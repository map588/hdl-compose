//! Tests that run the external tools (oss-cad-suite) on a real project.
//!
//! A test whose tools are not on PATH prints a message and passes: the
//! suite must stay green on a host without the tools. Run `hdlc doctor` to
//! see which tools the host has.

mod common;

use std::path::Path;

use hdl_compose::codegen;
use hdl_compose::toolchain::{self, StagedBuild, ToolchainConfig};
use hdl_compose::types::{Language, ModuleDef};

/// True when each tool is on PATH. Else a message names the missing tools.
fn have(tools: &[&str]) -> bool {
    let missing: Vec<&&str> = tools
        .iter()
        .filter(|tool| !toolchain::tool_on_path(tool))
        .collect();
    if !missing.is_empty() {
        eprintln!("skipped: not on PATH: {missing:?}");
    }
    missing.is_empty()
}

/// Write the sources into `dir`, make a project whose top passes each port
/// of the module `dut` through, and stage it.
fn stage_passthrough(dir: &Path, sources: &[(&str, &str)], dut: &str) -> StagedBuild {
    let mut library: Vec<ModuleDef> = Vec::new();
    for (name, text) in sources {
        let path = dir.join(name);
        std::fs::write(&path, text).unwrap();
        library.extend(hdl_compose::parse_file(&path).unwrap());
    }
    let module = library.iter().find(|m| m.name == dut).unwrap();
    let mut schematic = common::build_passthrough_schematic(module, Language::SystemVerilog);
    for (name, _) in sources {
        schematic.add_library_path(dir.join(name));
    }
    let design = codegen::generate_design(&schematic, &library).expect("codegen");
    toolchain::stage(
        &schematic,
        &library,
        &design,
        &ToolchainConfig::default(),
        dir,
    )
    .unwrap()
}

const INNER: &str =
    "module inner (input logic [7:0] d, output logic q);\n  assign q = ^d;\nendmodule\n";

/// `outer` connects `width` bits to the 8-bit port of `inner`.
fn outer(connection: &str) -> String {
    format!(
        "module outer (input logic [7:0] d, output logic q);\n  inner u_inner (.d({connection}), .q(q));\nendmodule\n"
    )
}

/// slang reports a port connection of the wrong width only as a warning,
/// and its warnings are off by default: plain `slang --top` exits 0 on this
/// design. `verilator --lint-only`, which `check` used before, stops on it.
/// The mismatch is inside a library module, where `Schematic::validate`
/// does not look.
#[test]
fn check_fails_on_a_port_width_mismatch_inside_a_module() {
    if !have(&["slang"]) {
        return;
    }
    let good = tempfile::tempdir().unwrap();
    let staged = stage_passthrough(
        good.path(),
        &[("inner.sv", INNER), ("outer.sv", &outer("d"))],
        "outer",
    );
    assert_eq!(toolchain::run_check(&staged), Ok(true));

    let bad = tempfile::tempdir().unwrap();
    let staged = stage_passthrough(
        bad.path(),
        &[("inner.sv", INNER), ("outer.sv", &outer("d[6:0]"))],
        "outer",
    );
    assert_eq!(toolchain::run_check(&staged), Ok(false));
}

/// A package and a struct port: `read_verilog -sv` cannot read this design.
const PACKAGED: &str = "package bus_pkg;\n  typedef struct packed { logic [3:0] hi; logic [3:0] lo; } pair_t;\nendpackage\n\nmodule swap import bus_pkg::*; (input logic [7:0] d, output logic [7:0] q);\n  pair_t p;\n  assign p = d;\n  assign q = {p.lo, p.hi};\nendmodule\n";

#[test]
fn synth_reads_a_package_and_a_struct_through_slang() {
    if !have(&["yosys"]) {
        return;
    }
    let project = tempfile::tempdir().unwrap();
    let staged = stage_passthrough(project.path(), &[("swap.sv", PACKAGED)], "swap");
    assert_eq!(toolchain::run_synth(&staged), Ok(true));
    // The first variant passed, so the fallback did not run.
    let record = std::fs::read_to_string(staged.dir.join("steps.log")).unwrap();
    assert_eq!(record.matches("$ yosys").count(), 1, "{record}");
    assert!(record.contains("-m slang"), "{record}");
}

// --- Netlist check -----------------------------------------------------------

use hdl_compose::netcheck::{self, Outcome};
use hdl_compose::types::{Group, NetRef, Schematic, SliceExpr};

/// True when yosys can load the plugin.
fn have_plugin(plugin: &str) -> bool {
    let loads = std::process::Command::new("yosys")
        .args(["-q", "-m", plugin, "-p", ""])
        .output()
        .is_ok_and(|out| out.status.success());
    if !loads {
        eprintln!("skipped: yosys cannot load the {plugin} plugin");
    }
    loads
}

const VHDL_SOURCES: &[(&str, &str)] = &[
    (
        "source.vhd",
        "library ieee;\nuse ieee.std_logic_1164.all;\n\
         entity source is\n  generic (WIDTH : integer := 4);\n\
         port (clk : in std_logic; q : out std_logic_vector(WIDTH-1 downto 0));\nend entity;\n\
         architecture rtl of source is\nbegin\n  q <= (others => clk);\nend architecture;\n",
    ),
    (
        "sink.vhd",
        "library ieee;\nuse ieee.std_logic_1164.all;\n\
         entity sink is\n\
         port (clk : in std_logic; d : in std_logic_vector(7 downto 0); ok : out std_logic);\nend entity;\n\
         architecture rtl of sink is\nbegin\n  ok <= d(0) and clk;\nend architecture;\n",
    ),
    (
        "narrow.vhd",
        "library ieee;\nuse ieee.std_logic_1164.all;\n\
         entity narrow is\n\
         port (d : in std_logic_vector(3 downto 0); ok : out std_logic);\nend entity;\n\
         architecture rtl of narrow is\nbegin\n  ok <= d(3);\nend architecture;\n",
    ),
];

const SV_SOURCES: &[(&str, &str)] = &[
    (
        "source.sv",
        "module source #(parameter WIDTH = 4) (input logic clk, output logic [WIDTH-1:0] q);\n\
         assign q = {WIDTH{clk}};\nendmodule\n",
    ),
    (
        "sink.sv",
        "module sink (input logic clk, input logic [7:0] d, output logic ok);\n\
         assign ok = d[0] & clk;\nendmodule\n",
    ),
    (
        "narrow.sv",
        "module narrow (input logic [3:0] d, output logic ok);\n\
         assign ok = d[3];\nendmodule\n",
    ),
];

/// A design with the cases that codegen must get right: a width from a
/// generic, a reference to a slice of a net, a net with three pins, and a
/// group whose output goes to two instances outside it.
///
/// `src.q` (8 bits) drives `dst.d` and, with its bits 3 to 0, `lo.d`.
fn netlist_design(dir: &Path, language: Language) -> (Schematic, Vec<ModuleDef>) {
    let sources = match language {
        Language::Vhdl => VHDL_SOURCES,
        Language::SystemVerilog => SV_SOURCES,
    };
    let mut library: Vec<ModuleDef> = Vec::new();
    let mut s = Schematic::new("top", language);
    for (name, text) in sources {
        let path = dir.join(name);
        std::fs::write(&path, text).unwrap();
        library.extend(hdl_compose::parse_file(&path).unwrap());
        s.add_library_path(path);
    }
    let module = |name: &str| library.iter().find(|m| m.name == name).unwrap();
    let top_port = |name: &str, of: &str| {
        module(of)
            .ports
            .iter()
            .find(|p| p.name == name)
            .unwrap()
            .clone()
    };
    let mut ok2 = top_port("ok", "narrow");
    ok2.name = "ok2".to_string();
    s.top_ports = vec![top_port("clk", "sink"), top_port("ok", "sink"), ok2];

    for (instance, of) in [("src", "source"), ("dst", "sink"), ("lo", "narrow")] {
        s.add_instance(instance, of).unwrap();
    }
    s.instances[0]
        .generic_map
        .insert("WIDTH".to_string(), "8".to_string());
    let top = |name: &str| Some(NetRef::TopPort(name.to_string()));
    let q = NetRef::InstancePort("src".to_string(), "q".to_string());
    let low_half = NetRef::InstancePortSlice(
        "src".to_string(),
        "q".to_string(),
        SliceExpr::Range { high: 3, low: 0 },
    );
    for (instance, port, net) in [
        ("src", "clk", top("clk")),
        ("dst", "clk", top("clk")),
        ("dst", "d", Some(q)),
        ("dst", "ok", top("ok")),
        ("lo", "d", Some(low_half)),
        ("lo", "ok", top("ok2")),
    ] {
        s.set_port_map_entry(instance, port.to_string(), net)
            .unwrap();
    }
    s.groups.push(Group {
        name: "core".to_string(),
        members: vec!["src".to_string()],
        parent: None,
        collapsed: false,
        position: (0.0, 0.0),
    });
    (s, library)
}

/// Stage the design and get its netlist from yosys.
fn netlist_of(dir: &Path, schematic: &Schematic, library: &[ModuleDef]) -> String {
    let design = codegen::generate_design(schematic, library).expect("codegen");
    let staged = toolchain::stage(
        schematic,
        library,
        &design,
        &ToolchainConfig::default(),
        dir,
    )
    .unwrap();
    toolchain::run_netlist(&staged)
        .unwrap()
        .expect("yosys reads the generated files")
}

fn netlist_tools(language: &Language) -> bool {
    match language {
        Language::Vhdl => have(&["yosys", "ghdl"]) && have_plugin("ghdl"),
        Language::SystemVerilog => have(&["yosys"]) && have_plugin("slang"),
    }
}

#[test]
fn netlist_of_the_generated_files_connects_what_the_schematic_connects() {
    for language in [Language::Vhdl, Language::SystemVerilog] {
        if !netlist_tools(&language) {
            continue;
        }
        let dir = tempfile::tempdir().unwrap();
        let (schematic, library) = netlist_design(dir.path(), language.clone());
        let json = netlist_of(dir.path(), &schematic, &library);
        // clk, eight bits of q, ok, ok2.
        assert_eq!(
            netcheck::compare(&schematic, &library, &json),
            Ok(Outcome::Same {
                wires: 11,
                unchecked: 0
            }),
            "{language:?}"
        );
    }
}

/// `lo` has no output in use. yosys knows the port directions of a
/// SystemVerilog black box, and `opt_clean` removed such an instance from
/// the netlist. The check then reported that `lo.d` had no connection.
#[test]
fn netlist_check_keeps_an_instance_whose_outputs_have_no_connection() {
    for language in [Language::Vhdl, Language::SystemVerilog] {
        if !netlist_tools(&language) {
            continue;
        }
        let dir = tempfile::tempdir().unwrap();
        let (mut schematic, library) = netlist_design(dir.path(), language.clone());
        schematic
            .set_port_map_entry("lo", "ok".to_string(), None)
            .unwrap();
        schematic.top_ports.retain(|port| port.name != "ok2");
        let json = netlist_of(dir.path(), &schematic, &library);
        // clk, eight bits of q, ok.
        assert_eq!(
            netcheck::compare(&schematic, &library, &json),
            Ok(Outcome::Same {
                wires: 10,
                unchecked: 0
            }),
            "{language:?}"
        );
    }
}

/// The netlist of one design, compared with a schematic that differs in one
/// connection: `lo.d` on bits 7 to 4 of the net, not on bits 3 to 0. This
/// is the result of a codegen that connects the wrong slice.
#[test]
fn netlist_check_finds_a_connection_to_the_wrong_slice() {
    for language in [Language::Vhdl, Language::SystemVerilog] {
        if !netlist_tools(&language) {
            continue;
        }
        let dir = tempfile::tempdir().unwrap();
        let (mut schematic, library) = netlist_design(dir.path(), language.clone());
        let json = netlist_of(dir.path(), &schematic, &library);

        let high_half = NetRef::InstancePortSlice(
            "src".to_string(),
            "q".to_string(),
            SliceExpr::Range { high: 7, low: 4 },
        );
        schematic
            .set_port_map_entry("lo", "d".to_string(), Some(high_half))
            .unwrap();
        let Ok(Outcome::Different(lines)) = netcheck::compare(&schematic, &library, &json) else {
            panic!("{language:?}: the wrong slice was not found");
        };
        assert!(
            lines.contains(
                &"lo.d bit 0: the schematic connects it to dst.d bit 4, src.q bit 4; \
                  the generated HDL connects it to dst.d bit 0, src.q bit 0"
                    .to_string()
            ),
            "{language:?}: {lines:#?}"
        );
    }
}

// --- The built-in gates ---------------------------------------------------

use hdl_compose::gates::{self, Gate, Op};
use hdl_compose::types::{Direction, PortDef, PortType, Range, RangeDir, RangeExpr};

/// The value of a gate, written with no use of the code generators.
fn gate_value(op: Op, a: bool, b: bool) -> bool {
    match op {
        Op::And => a && b,
        Op::Or => a || b,
        Op::Xor => a != b,
        Op::Not => !a,
        Op::Nand => !(a && b),
        Op::Nor => !(a || b),
        Op::Xnor => a == b,
    }
}

/// The name of an operation in the port names of `gates_design`.
fn op_name(op: Op) -> String {
    format!("{op:?}").to_lowercase()
}

/// The bits of the two vector inputs, most significant bit first. The four
/// bit positions have the four combinations of two input bits.
const VA: [bool; 4] = [true, true, false, false];
const VB: [bool; 4] = [true, false, true, false];

/// A design with each built-in gate. The scalar gates read the top-level
/// ports `a` and `b`, the vector gates (4 bits) read `va` and `vb`. The
/// gate of the operation `and` writes `y_and` or `v_and`.
fn gates_design(language: Language) -> (Schematic, Vec<ModuleDef>) {
    let vector = PortType::StdLogicVector(Range {
        high: RangeExpr::Literal(3),
        low: RangeExpr::Literal(0),
        dir: RangeDir::Downto,
    });
    let mut s = Schematic::new("gates", language);
    let top_port = |s: &mut Schematic, name: String, direction, port_type: &PortType| {
        s.top_ports.push(PortDef {
            name,
            direction,
            port_type: port_type.clone(),
            bundle: None,
        });
    };
    for (name, port_type) in [
        ("a", &PortType::StdLogic),
        ("b", &PortType::StdLogic),
        ("va", &vector),
        ("vb", &vector),
    ] {
        top_port(&mut s, name.to_string(), Direction::In, port_type);
    }
    for op in gates::OPS {
        for is_vector in [false, true] {
            let gate = Gate {
                op,
                vector: is_vector,
            };
            let (prefix, port_type) = if is_vector {
                ("v", &vector)
            } else {
                ("y", &PortType::StdLogic)
            };
            let output = format!("{prefix}_{}", op_name(op));
            let instance = format!("g{output}");
            top_port(&mut s, output.clone(), Direction::Out, port_type);
            s.add_instance(&instance, gate.module_name()).unwrap();
            if is_vector {
                s.set_generic_map_entry(&instance, gates::WIDTH, "4")
                    .unwrap();
            }
            for input in op.inputs() {
                let source = if is_vector {
                    format!("v{input}")
                } else {
                    input.to_string()
                };
                s.set_port_map_entry(&instance, *input, Some(NetRef::TopPort(source)))
                    .unwrap();
            }
            s.set_port_map_entry(&instance, gates::OUTPUT, Some(NetRef::TopPort(output)))
                .unwrap();
        }
    }
    (s, gates::modules())
}

fn bits(values: impl IntoIterator<Item = bool>) -> String {
    values
        .into_iter()
        .map(|bit| if bit { '1' } else { '0' })
        .collect()
}

/// A testbench for `gates_design`. It gives each combination of `a` and
/// `b`, and stops with an error when an output is not the value of
/// `gate_value`.
fn gates_testbench(language: &Language) -> String {
    let outputs: Vec<String> = gates::OPS
        .iter()
        .flat_map(|op| ["y", "v"].map(|prefix| format!("{prefix}_{}", op_name(*op))))
        .collect();
    let vector_value = |op: Op| bits(VA.iter().zip(VB).map(|(a, b)| gate_value(op, *a, b)));
    let mut checks = String::new();
    for (a, b) in [(false, false), (false, true), (true, false), (true, true)] {
        let (a_bit, b_bit) = (bits([a]), bits([b]));
        match language {
            Language::Vhdl => {
                checks += &format!("    a <= '{a_bit}'; b <= '{b_bit}'; wait for 1 ns;\n");
                for op in gates::OPS {
                    let name = op_name(op);
                    checks += &format!(
                        "    assert y_{name} = '{}' report \"y_{name} at a={a_bit} b={b_bit}\" severity failure;\n",
                        bits([gate_value(op, a, b)])
                    );
                    checks += &format!(
                        "    assert v_{name} = \"{}\" report \"v_{name}\" severity failure;\n",
                        vector_value(op)
                    );
                }
            }
            Language::SystemVerilog => {
                checks += &format!("    a = 1'b{a_bit}; b = 1'b{b_bit}; #1;\n");
                for op in gates::OPS {
                    let name = op_name(op);
                    checks += &format!(
                        "    if (y_{name} !== 1'b{}) $fatal(1, \"y_{name} at a={a_bit} b={b_bit}\");\n",
                        bits([gate_value(op, a, b)])
                    );
                    checks += &format!(
                        "    if (v_{name} !== 4'b{}) $fatal(1, \"v_{name}\");\n",
                        vector_value(op)
                    );
                }
            }
        }
    }
    let (va, vb) = (bits(VA), bits(VB));
    match language {
        Language::Vhdl => {
            let signals: String = outputs
                .iter()
                .map(|name| {
                    let kind = if name.starts_with('v') {
                        "std_logic_vector(3 downto 0)"
                    } else {
                        "std_logic"
                    };
                    format!("  signal {name} : {kind};\n")
                })
                .collect();
            let map: String = outputs
                .iter()
                .map(|name| format!(", {name} => {name}"))
                .collect();
            format!(
                "library ieee;\nuse ieee.std_logic_1164.all;\n\nentity gates_tb is\nend entity;\n\n\
                 architecture sim of gates_tb is\n  signal a, b : std_logic;\n\
                 \x20 signal va : std_logic_vector(3 downto 0) := \"{va}\";\n\
                 \x20 signal vb : std_logic_vector(3 downto 0) := \"{vb}\";\n{signals}begin\n\
                 \x20 dut : entity work.gates port map (a => a, b => b, va => va, vb => vb{map});\n\
                 \x20 process\n  begin\n{checks}    wait;\n  end process;\nend architecture;\n"
            )
        }
        Language::SystemVerilog => {
            let signals: String = outputs
                .iter()
                .map(|name| {
                    let kind = if name.starts_with('v') {
                        "logic [3:0]"
                    } else {
                        "logic"
                    };
                    format!("  {kind} {name};\n")
                })
                .collect();
            let map: String = outputs
                .iter()
                .map(|name| format!(", .{name}({name})"))
                .collect();
            format!(
                "module gates_tb;\n  logic a, b;\n  logic [3:0] va = 4'b{va};\n\
                 \x20 logic [3:0] vb = 4'b{vb};\n{signals}\
                 \x20 gates dut (.a(a), .b(b), .va(va), .vb(vb){map});\n\
                 \x20 initial begin\n{checks}    $finish;\n  end\nendmodule\n"
            )
        }
    }
}

/// The generated HDL of each gate has the value of the gate, for each
/// combination of the input bits, in the two languages. The reference is
/// `gate_value`; a simulator runs the generated HDL.
#[test]
fn each_gate_has_its_truth_table_in_a_simulation() {
    for language in [Language::Vhdl, Language::SystemVerilog] {
        let tools: &[&str] = match language {
            Language::Vhdl => &["ghdl"],
            Language::SystemVerilog => &["iverilog", "vvp"],
        };
        if !have(tools) {
            continue;
        }
        let dir = tempfile::tempdir().unwrap();
        let (schematic, library) = gates_design(language.clone());
        let design = codegen::generate_design(&schematic, &library).expect("codegen");
        let staged = toolchain::stage(
            &schematic,
            &library,
            &design,
            &ToolchainConfig::default(),
            dir.path(),
        )
        .unwrap();
        let ext = if language == Language::Vhdl {
            "vhd"
        } else {
            "sv"
        };
        std::fs::write(
            dir.path().join(format!("gates_tb.{ext}")),
            gates_testbench(&language),
        )
        .unwrap();
        let wave = toolchain::run_sim(&staged, &schematic, dir.path()).unwrap();
        assert!(
            wave.is_some(),
            "{language:?}: the simulation stopped with an error\n{}",
            design.top_code
        );
    }
}

/// The frontend of each language accepts a design with gates, and the
/// netlist check accepts the logic cells that yosys makes from them.
#[test]
fn check_and_netlist_check_accept_a_design_with_gates() {
    for language in [Language::Vhdl, Language::SystemVerilog] {
        let check_tool: &[&str] = match language {
            Language::Vhdl => &["ghdl"],
            Language::SystemVerilog => &["slang"],
        };
        if !have(check_tool) || !netlist_tools(&language) {
            continue;
        }
        let dir = tempfile::tempdir().unwrap();
        let (schematic, library) = gates_design(language.clone());
        let design = codegen::generate_design(&schematic, &library).expect("codegen");
        let staged = toolchain::stage(
            &schematic,
            &library,
            &design,
            &ToolchainConfig::default(),
            dir.path(),
        )
        .unwrap();
        assert_eq!(toolchain::run_check(&staged), Ok(true), "{language:?}");
        let json = toolchain::run_netlist(&staged)
            .unwrap()
            .expect("yosys reads the generated files");
        // Each net has one top-level port and pins of gates only, and the
        // check does not compare the pins of a gate.
        assert_eq!(
            netcheck::compare(&schematic, &library, &json),
            Ok(Outcome::Same {
                wires: 0,
                unchecked: 0
            }),
            "{language:?}"
        );
    }
}
