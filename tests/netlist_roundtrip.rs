//! Round trip of codegen through an independent tool.
//!
//! For a random valid schematic: write the library modules as HDL, read
//! them back with the parser, generate the top level (and the group
//! modules), let yosys elaborate the generated files, and compare the
//! connections in its netlist with the nets of the schematic.
//!
//! The property is `netlist(codegen(schematic)) == nets(schematic)`. yosys
//! is the reference: it shares no code with hdlc.
//!
//! The test needs yosys with the ghdl plugin (VHDL) and the slang plugin
//! (SystemVerilog). When a tool is missing it prints a message and tests
//! only the language that has its tools.
//!
//! The generator does not make two designs for which codegen has a known
//! defect. `CLOCK` and `may_drive` in `design` give the conditions. Remove
//! each limit together with the correction of its defect.

use std::path::Path;

use hdl_compose::codegen;
use hdl_compose::netcheck::{self, Outcome};
use hdl_compose::toolchain::{self, ToolchainConfig};
use hdl_compose::types::{
    Direction, GenericDef, Group, Language, ModuleDef, NetRef, PortDef, PortType, Range, RangeDir,
    RangeExpr, Schematic, SliceExpr,
};
use proptest::prelude::*;

/// The choices of the generator, one byte for each choice. A design is a
/// function of the bytes, so proptest shrinks a failure by making bytes
/// smaller; choice 0 is always the simplest one.
struct Choices<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Choices<'_> {
    /// A number below `count`. After the last byte each choice is 0.
    fn pick(&mut self, count: usize) -> usize {
        let byte = self.bytes.get(self.at).copied().unwrap_or(0);
        self.at += 1;
        usize::from(byte) % count
    }

    fn flip(&mut self) -> bool {
        self.pick(2) == 1
    }
}

/// The name of the generic that each library module with a generic has.
const MODULE_GENERIC: &str = "WIDTH";
/// The default value of `MODULE_GENERIC`.
const MODULE_DEFAULT: i64 = 4;
/// The name of the top-level generic.
const TOP_GENERIC: &str = "TW";
/// The name of the clock input of each library module, and of the top-level
/// input that drives each of them. With it each group has a port: codegen
/// writes a VHDL group module with no port as `port map ()`, which is not
/// legal VHDL.
const CLOCK: &str = "clk";

/// The type of a port of a library module.
#[derive(Clone, Copy)]
enum Shape {
    Bit,
    /// `(low + width - 1 downto low)`.
    Vector {
        width: i64,
        low: i64,
    },
    /// `(WIDTH - 1 downto 0)`.
    Generic,
}

struct PortSpec {
    name: String,
    output: bool,
    shape: Shape,
}

struct ModuleSpec {
    name: String,
    has_generic: bool,
    ports: Vec<PortSpec>,
}

fn vhdl_type(shape: Shape) -> String {
    match shape {
        Shape::Bit => "std_logic".to_string(),
        Shape::Vector { width, low } => {
            format!("std_logic_vector({} downto {low})", low + width - 1)
        }
        Shape::Generic => format!("std_logic_vector({MODULE_GENERIC}-1 downto 0)"),
    }
}

fn sv_type(shape: Shape) -> String {
    match shape {
        Shape::Bit => "logic".to_string(),
        Shape::Vector { width, low } => format!("logic [{}:{low}]", low + width - 1),
        Shape::Generic => format!("logic [{MODULE_GENERIC}-1:0]"),
    }
}

/// The source of a library module with no content. The netlist check
/// makes each library module a black box, so only the ports have an effect.
fn module_source(module: &ModuleSpec, language: &Language) -> String {
    match language {
        Language::Vhdl => {
            let clock = format!("    {CLOCK} : in std_logic");
            let ports: Vec<String> = std::iter::once(clock)
                .chain(module.ports.iter().map(|p| {
                    let mode = if p.output { "out" } else { "in" };
                    format!("    {} : {mode} {}", p.name, vhdl_type(p.shape))
                }))
                .collect();
            let generic = if module.has_generic {
                format!("  generic ({MODULE_GENERIC} : integer := {MODULE_DEFAULT});\n")
            } else {
                String::new()
            };
            format!(
                "library ieee;\nuse ieee.std_logic_1164.all;\n\nentity {name} is\n{generic}  port (\n{ports}\n  );\nend entity;\n\narchitecture rtl of {name} is\nbegin\nend architecture;\n",
                name = module.name,
                ports = ports.join(";\n")
            )
        }
        Language::SystemVerilog => {
            let clock = format!("  input logic {CLOCK}");
            let ports: Vec<String> = std::iter::once(clock)
                .chain(module.ports.iter().map(|p| {
                    let mode = if p.output { "output" } else { "input" };
                    format!("  {mode} {} {}", sv_type(p.shape), p.name)
                }))
                .collect();
            let parameter = if module.has_generic {
                format!(" #(\n  parameter int {MODULE_GENERIC} = {MODULE_DEFAULT}\n)")
            } else {
                String::new()
            };
            format!(
                "module {}{parameter} (\n{}\n);\nendmodule\n",
                module.name,
                ports.join(",\n")
            )
        }
    }
}

/// A pin that can drive a net, with the numbers of its type.
struct Source {
    reference: NetRef,
    /// The instance of the pin. None for a top-level input.
    from: Option<usize>,
    /// 0 for a pin that is not a vector.
    width: i64,
    /// The lowest index number of the type.
    low: i64,
}

fn vector(high: RangeExpr, low: i64) -> PortType {
    PortType::StdLogicVector(Range {
        high,
        low: RangeExpr::Literal(low),
        dir: RangeDir::Downto,
    })
}

/// A literal that ties an input of the given type to zero.
fn zero(is_vector: bool, language: &Language) -> NetRef {
    NetRef::Constant(
        match (language, is_vector) {
            (Language::Vhdl, false) => "'0'",
            (Language::Vhdl, true) => "(others => '0')",
            (Language::SystemVerilog, false) => "1'b0",
            (Language::SystemVerilog, true) => "'0",
        }
        .to_string(),
    )
}

/// Make a design from the choices and write the sources of its library
/// modules into `dir`.
fn design(choices: &mut Choices, language: Language, dir: &Path) -> (Schematic, Vec<ModuleDef>) {
    let top_default = choices.flip().then(|| 2 + choices.pick(7) as i64);

    // Library modules.
    let modules: Vec<ModuleSpec> = (0..2 + choices.pick(2))
        .map(|m| {
            let has_generic = choices.flip();
            let outputs = 1 + choices.pick(2);
            let inputs = 1 + choices.pick(3);
            let ports = (0..outputs + inputs)
                .map(|p| {
                    let output = p < outputs;
                    let shape = match choices.pick(4) {
                        1 => Shape::Vector {
                            width: 2 + choices.pick(7) as i64,
                            low: 0,
                        },
                        2 => Shape::Vector {
                            width: 2 + choices.pick(7) as i64,
                            low: 1 + choices.pick(4) as i64,
                        },
                        3 if has_generic => Shape::Generic,
                        _ => Shape::Bit,
                    };
                    PortSpec {
                        name: format!("{}{p}", if output { "o" } else { "i" }),
                        output,
                        shape,
                    }
                })
                .collect();
            ModuleSpec {
                name: format!("m{m}"),
                has_generic,
                ports,
            }
        })
        .collect();

    let extension = match language {
        Language::Vhdl => "vhd",
        Language::SystemVerilog => "sv",
    };
    let mut library: Vec<ModuleDef> = Vec::new();
    let mut s = Schematic::new("top", language.clone());
    for module in &modules {
        let path = dir.join(format!("{}.{extension}", module.name));
        std::fs::write(&path, module_source(module, &language)).unwrap();
        library.extend(hdl_compose::parse_file(&path).expect("the parser reads the module"));
        s.add_library_path(path);
    }
    if let Some(default) = top_default {
        s.top_generics.push(GenericDef {
            name: TOP_GENERIC.to_string(),
            type_name: "integer".to_string(),
            default_value: Some(default.to_string()),
        });
    }

    // Instances. `widths[i]` is the value of the generic of instance `i`;
    // `from_top[i]` is true when that value is the top-level generic.
    let count = 2 + choices.pick(3);
    let mut of: Vec<&ModuleSpec> = Vec::new();
    let mut widths: Vec<i64> = Vec::new();
    let mut from_top: Vec<bool> = Vec::new();
    for i in 0..count {
        let module = &modules[choices.pick(modules.len())];
        let name = format!("u{i}");
        s.add_instance(&name, &module.name).unwrap();
        s.set_port_map_entry(
            &name,
            CLOCK.to_string(),
            Some(NetRef::TopPort(CLOCK.to_string())),
        )
        .unwrap();
        let (width, top) = match (module.has_generic, choices.pick(3)) {
            (true, 1) => {
                let width = 2 + choices.pick(7) as i64;
                s.instances[i]
                    .generic_map
                    .insert(MODULE_GENERIC.to_string(), width.to_string());
                (width, false)
            }
            (true, 2) if top_default.is_some() => {
                s.instances[i]
                    .generic_map
                    .insert(MODULE_GENERIC.to_string(), TOP_GENERIC.to_string());
                (top_default.unwrap(), true)
            }
            _ => (MODULE_DEFAULT, false),
        };
        of.push(module);
        widths.push(width);
        from_top.push(top);
    }
    // The width and the lowest index number of a pin of instance `i`.
    let numbers = |i: usize, shape: Shape| match shape {
        Shape::Bit => (0, 0),
        Shape::Vector { width, low } => (width, low),
        Shape::Generic => (widths[i], 0),
    };

    // Groups: each instance is in the first group, the second group, or no
    // group. The second group can be inside the first.
    let group_of: Vec<usize> = (0..count).map(|_| choices.pick(3)).collect();
    let nested = choices.flip() && group_of.contains(&1) && group_of.contains(&2);
    // Known defect of `expand_hierarchy`: a net from the inner group that
    // goes to the outer group and to an instance outside the two groups can
    // get no connection to the port of the outer group. The generated HDL
    // then has one name for a port and for a signal, and no tool reads it.
    // Until the correction, a net from the inner group goes only to the two
    // groups and to top-level ports.
    let may_drive = |driver: usize, consumer: usize| {
        !(nested && group_of[driver] == 2) || group_of[consumer] != 0
    };

    // Top-level inputs.
    s.top_ports.push(PortDef {
        name: CLOCK.to_string(),
        direction: Direction::In,
        port_type: PortType::StdLogic,
        bundle: None,
    });
    let mut sources: Vec<Source> = Vec::new();
    for t in 0..1 + choices.pick(2) {
        let name = format!("tin{t}");
        let width = match choices.pick(2) {
            0 => 0,
            _ => 2 + choices.pick(7) as i64,
        };
        s.top_ports.push(PortDef {
            name: name.clone(),
            direction: Direction::In,
            port_type: match width {
                0 => PortType::StdLogic,
                w => vector(RangeExpr::Literal(w - 1), 0),
            },
            bundle: None,
        });
        sources.push(Source {
            reference: NetRef::TopPort(name),
            from: None,
            width,
            low: 0,
        });
    }

    // Each output of an instance can drive a net. Some go to a top-level
    // output with the type of the pin.
    for (i, module) in of.iter().enumerate() {
        for port in module.ports.iter().filter(|p| p.output) {
            let (width, low) = numbers(i, port.shape);
            let reference = NetRef::InstancePort(format!("u{i}"), port.name.clone());
            if choices.pick(3) == 1 {
                let name = format!("tout_u{i}_{}", port.name);
                let high = match port.shape {
                    Shape::Generic if from_top[i] => RangeExpr::Expr(format!("{TOP_GENERIC}-1")),
                    _ => RangeExpr::Literal(low + width - 1),
                };
                s.top_ports.push(PortDef {
                    name: name.clone(),
                    direction: Direction::Out,
                    port_type: match width {
                        0 => PortType::StdLogic,
                        _ => vector(high, low),
                    },
                    bundle: None,
                });
                s.set_port_map_entry(
                    &format!("u{i}"),
                    port.name.clone(),
                    Some(NetRef::TopPort(name)),
                )
                .unwrap();
            }
            sources.push(Source {
                reference,
                from: Some(i),
                width,
                low,
            });
        }
    }

    // Each input of an instance gets a driver, or a constant. VHDL does not
    // accept an input with no connection. A wider driver gives a slice,
    // with the index numbers of the driver.
    for (i, module) in of.iter().enumerate() {
        for port in module.ports.iter().filter(|p| !p.output) {
            let (width, _) = numbers(i, port.shape);
            let source = match choices.pick(sources.len() + 1) {
                0 => None,
                n => Some(&sources[n - 1]),
            }
            .filter(|source| source.from.is_none_or(|driver| may_drive(driver, i)));
            let slice = |slice: SliceExpr, source: &Source| match &source.reference {
                NetRef::TopPort(name) => NetRef::TopPortSlice(name.clone(), slice),
                NetRef::InstancePort(instance, pin) => {
                    NetRef::InstancePortSlice(instance.clone(), pin.clone(), slice)
                }
                other => other.clone(),
            };
            let entry = match source {
                Some(source) if source.width == width => source.reference.clone(),
                // One bit of a vector for an input that is not a vector.
                Some(source) if width == 0 && source.width > 0 => {
                    let index = source.low + choices.pick(source.width as usize) as i64;
                    slice(SliceExpr::Bit(index as i32), source)
                }
                Some(source) if width > 0 && source.width > width => {
                    let first =
                        source.low + choices.pick((source.width - width + 1) as usize) as i64;
                    slice(
                        SliceExpr::Range {
                            high: (first + width - 1) as i32,
                            low: first as i32,
                        },
                        source,
                    )
                }
                _ => zero(width > 0, &language),
            };
            s.set_port_map_entry(&format!("u{i}"), port.name.clone(), Some(entry))
                .unwrap();
        }
    }

    for g in 1..=2 {
        let members: Vec<String> = (0..count)
            .filter(|i| group_of[*i] == g)
            .map(|i| format!("u{i}"))
            .collect();
        if members.is_empty() {
            continue;
        }
        s.groups.push(Group {
            name: format!("g{}", g - 1),
            members,
            parent: (g == 2 && nested).then(|| "g0".to_string()),
            collapsed: false,
            position: (0.0, 0.0),
        });
    }
    (s, library)
}

/// True when yosys is on PATH and can load the plugin for the language.
fn has_tools(language: &Language) -> bool {
    let (plugin, tools): (&str, &[&str]) = match language {
        Language::Vhdl => ("ghdl", &["yosys", "ghdl"]),
        Language::SystemVerilog => ("slang", &["yosys"]),
    };
    tools.iter().all(|tool| toolchain::tool_on_path(tool))
        && std::process::Command::new("yosys")
            .args(["-q", "-m", plugin, "-p", ""])
            .output()
            .is_ok_and(|out| out.status.success())
}

/// Each case is one yosys run of about 0.1 s.
const CASES: u32 = 32;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn netlist_of_the_generated_hdl_equals_the_nets_of_the_schematic(
        bytes in proptest::collection::vec(any::<u8>(), 96),
        vhdl in any::<bool>(),
    ) {
        let language = if vhdl { Language::Vhdl } else { Language::SystemVerilog };
        if !has_tools(&language) {
            eprintln!("skipped: no yosys with the plugin for {language:?} on PATH");
            return Ok(());
        }
        let dir = tempfile::tempdir().unwrap();
        let mut choices = Choices { bytes: &bytes, at: 0 };
        let (schematic, library) = design(&mut choices, language, dir.path());
        let project = serde_json::to_string_pretty(&schematic).unwrap();

        // The generator makes valid designs. An error here is a defect of
        // the generator or of `validate`.
        let errors: Vec<String> = schematic
            .validate(&library)
            .into_iter()
            .filter(|d| d.is_error())
            .map(|d| d.to_string())
            .collect();
        prop_assert!(errors.is_empty(), "validate: {errors:#?}\n{project}");

        let generated = codegen::generate_design(&schematic, &library);
        prop_assert!(generated.is_ok(), "codegen failed: {:?}\n{project}", generated.err());
        let staged = toolchain::stage(
            &schematic,
            &library,
            &generated.unwrap(),
            &ToolchainConfig::default(),
            dir.path(),
        )
        .unwrap();
        let json = toolchain::run_netlist(&staged).unwrap();
        prop_assert!(json.is_some(), "yosys cannot read the generated files\n{project}");

        let outcome = netcheck::compare(&schematic, &library, &json.unwrap());
        prop_assert!(
            matches!(outcome, Ok(Outcome::Same { unchecked: 0, .. })),
            "{outcome:#?}\n{project}"
        );
    }
}
