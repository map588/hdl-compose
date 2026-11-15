//! Netlist check: compare the generated HDL with the nets of the schematic.
//!
//! Codegen is the one part of hdlc that no external tool checked: ghdl and
//! slang accept a generated top level that connects the wrong pins, as long
//! as the widths agree. This module gets the connections two times, from
//! two independent sources, and compares them.
//!
//! - `expected` comes from `resolve_nets` on the schematic, with the groups
//!   ignored. A group must not change what is connected.
//! - `observed` comes from yosys. yosys reads only the generated files, with
//!   each library module as a black box, flattens the group modules, and
//!   writes the netlist as JSON (see `toolchain::netlist_stages`).
//!
//! Both are a `Connectivity`: the sets of pin bits that are on one wire.
//! Constant ties are not compared.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::codegen::resolve_port_type;
use crate::nets::{is_ascending, resolve_nets};
use crate::types::*;

/// One bit of a pin. A pin is a port of an instance or a top-level port.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Endpoint {
    /// The instance. `None` for a top-level port.
    pub instance: Option<String>,
    pub port: String,
    /// The place of the bit in the port, counted from the right-hand end of
    /// the range: 0 is index 0 of `(7 downto 0)` and index 7 of `(0 to 7)`.
    /// yosys numbers the bits of a port the same way.
    pub bit: u32,
}

impl std::fmt::Display for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.instance {
            Some(instance) => write!(f, "{instance}.{} bit {}", self.port, self.bit),
            None => write!(f, "top port {} bit {}", self.port, self.bit),
        }
    }
}

/// A pin: the instance (`None` for the top level) and the port.
type Pin = (Option<String>, String);

/// The sets of pin bits that are on one wire. A bit that is alone on its
/// wire is in no set.
pub type Connectivity = BTreeSet<BTreeSet<Endpoint>>;

/// The connections that the schematic asks for.
pub struct Expected {
    pub nets: Connectivity,
    /// The pins of each net that has a pin with no known width (a record
    /// type, or a bound that is not a number at the default values of the
    /// top-level generics). The check cannot place the bits of such a net,
    /// so it ignores these pins on both sides.
    pub unchecked: BTreeSet<Pin>,
}

/// VHDL names have no case and ghdl gives them in lower case.
fn fold(name: &str, fold_case: bool) -> String {
    if fold_case {
        name.to_lowercase()
    } else {
        name.to_string()
    }
}

/// The index numbers of a type with literal bounds.
struct Indexes {
    low: i64,
    high: i64,
    /// True when the index numbers go up from left to right.
    ascending: bool,
}

impl Indexes {
    fn of(port_type: &PortType) -> Option<Indexes> {
        match port_type {
            PortType::StdLogic => Some(Indexes {
                low: 0,
                high: 0,
                ascending: false,
            }),
            PortType::StdLogicVector(range) => {
                let (RangeExpr::Literal(a), RangeExpr::Literal(b)) = (&range.high, &range.low)
                else {
                    return None;
                };
                Some(Indexes {
                    low: *a.min(b),
                    high: *a.max(b),
                    ascending: is_ascending(port_type),
                })
            }
            PortType::Record(_) | PortType::Other(_) => None,
        }
    }

    /// The place of an index, counted from the right-hand end of the range.
    fn position(&self, index: i64) -> Option<u32> {
        if index < self.low || index > self.high {
            return None;
        }
        let from_right = if self.ascending {
            self.high - index
        } else {
            index - self.low
        };
        u32::try_from(from_right).ok()
    }
}

/// The bits of a pin that the schematic connects, as places in the port, in
/// the order of the index numbers. A consumer slice connects only a part of
/// the port. `None` when the width is not known.
fn connected_positions(port_type: &PortType, consumer: Option<&SliceExpr>) -> Option<Vec<u32>> {
    let indexes = Indexes::of(port_type)?;
    let (first, last) = match consumer {
        None => (indexes.low, indexes.high),
        Some(SliceExpr::Bit(i)) => (i64::from(*i), i64::from(*i)),
        Some(SliceExpr::Range { high, low }) => {
            (i64::from(*high.min(low)), i64::from(*high.max(low)))
        }
    };
    (first..=last)
        .map(|index| indexes.position(index))
        .collect()
}

/// The connections of the schematic, from `resolve_nets`. The groups have
/// no effect. A bound that uses a top-level generic is evaluated at the
/// default value of that generic, because yosys elaborates the top level
/// with the default values.
pub fn expected(schematic: &Schematic, library: &[ModuleDef]) -> Expected {
    let fold_case = schematic.language == Language::Vhdl;
    let modules: HashMap<&str, &ModuleDef> = library.iter().map(|m| (m.name.as_str(), m)).collect();
    let at_defaults = |port_type: &PortType| {
        resolve_port_type(port_type, &schematic.top_generics, &HashMap::new())
    };

    // The name of a member pin, and the places of the bits that it connects.
    let describe = |member: &NetRef| -> Option<(Pin, Option<Vec<u32>>)> {
        match member {
            NetRef::TopPort(name) => {
                let port = schematic.top_ports.iter().find(|p| &p.name == name);
                let positions =
                    port.and_then(|p| connected_positions(&at_defaults(&p.port_type), None));
                Some(((None, fold(name, fold_case)), positions))
            }
            NetRef::InstancePort(instance_name, port_name) => {
                let pin = (
                    Some(fold(instance_name, fold_case)),
                    fold(port_name, fold_case),
                );
                let positions = schematic
                    .instances
                    .iter()
                    .find(|i| &i.name == instance_name)
                    .and_then(|instance| {
                        let module = modules.get(instance.module_ref.as_str())?;
                        let port = module.ports.iter().find(|p| &p.name == port_name)?;
                        let port_type = at_defaults(&resolve_port_type(
                            &port.port_type,
                            &module.generics,
                            &instance.generic_map,
                        ));
                        connected_positions(&port_type, instance.consumer_slices.get(port_name))
                    });
                Some((pin, positions))
            }
            _ => None,
        }
    };

    let nets = resolve_nets(schematic, library);
    let mut wires: BTreeMap<(usize, i64), BTreeSet<Endpoint>> = BTreeMap::new();
    let mut unchecked: BTreeSet<Pin> = BTreeSet::new();
    for (net_index, net) in nets.nets.iter().enumerate() {
        let pins: Vec<(&NetRef, Pin, Option<Vec<u32>>)> = net
            .members
            .iter()
            .filter_map(|member| describe(member).map(|(pin, positions)| (member, pin, positions)))
            .collect();
        if pins.iter().any(|(_, _, positions)| positions.is_none()) {
            unchecked.extend(pins.into_iter().map(|(_, pin, _)| pin));
            continue;
        }
        for (member, (instance, port), positions) in pins {
            // The first bit of the net that the pin is on. `None` from
            // `Nets::bits` is the full net.
            let base = match nets.bits(schematic, member) {
                None => 0,
                Some(SliceExpr::Bit(i)) => i64::from(i),
                Some(SliceExpr::Range { high, low }) => i64::from(high.min(low)),
            };
            for (offset, bit) in (base..).zip(positions.into_iter().flatten()) {
                wires
                    .entry((net_index, offset))
                    .or_default()
                    .insert(Endpoint {
                        instance: instance.clone(),
                        port: port.clone(),
                        bit,
                    });
            }
        }
    }
    Expected {
        nets: wires.into_values().filter(|wire| wire.len() > 1).collect(),
        unchecked,
    }
}

/// The connections in the netlist that yosys wrote as JSON for the module
/// `top`. Each cell of the module must be an instance of a library module.
pub fn observed(json: &str, top: &str, fold_case: bool) -> Result<Connectivity, String> {
    let netlist: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("the netlist is not JSON: {e}"))?;
    let module = netlist["modules"]
        .as_object()
        .and_then(|modules| {
            modules
                .iter()
                .find(|(name, _)| fold(name, fold_case) == fold(top, fold_case))
        })
        .map(|(_, module)| module)
        .ok_or_else(|| format!("the netlist has no module {top}"))?;

    let mut wires: BTreeMap<u64, BTreeSet<Endpoint>> = BTreeMap::new();
    let mut connect = |instance: Option<&str>, port: &str, bits: &serde_json::Value| {
        for (place, bit) in (0u32..).zip(bits.as_array().into_iter().flatten()) {
            // A number is a wire. A text ("0", "1", "x", "z") is a constant.
            if let Some(wire) = bit.as_u64() {
                wires.entry(wire).or_default().insert(Endpoint {
                    instance: instance.map(|name| fold(name, fold_case)),
                    port: fold(port, fold_case),
                    bit: place,
                });
            }
        }
    };
    for (name, port) in module["ports"].as_object().into_iter().flatten() {
        connect(None, name, &port["bits"]);
    }
    for (name, cell) in module["cells"].as_object().into_iter().flatten() {
        let cell_type = cell["type"].as_str().unwrap_or_default();
        if cell_type.starts_with('$') {
            return Err(format!(
                "the top level has logic of its own (a `{cell_type}` cell)"
            ));
        }
        // yosys names a cell of a flattened group `u_<group>.<instance>`.
        let instance = name.rsplit('.').next().unwrap_or(name);
        for (port, bits) in cell["connections"].as_object().into_iter().flatten() {
            connect(Some(instance), port, bits);
        }
    }
    Ok(wires.into_values().filter(|wire| wire.len() > 1).collect())
}

/// One line for each pin bit that the two sides connect differently.
pub fn diff(expected: &Connectivity, observed: &Connectivity) -> Vec<String> {
    fn index(nets: &Connectivity) -> BTreeMap<&Endpoint, &BTreeSet<Endpoint>> {
        nets.iter()
            .flat_map(|wire| wire.iter().map(move |endpoint| (endpoint, wire)))
            .collect()
    }
    fn others(endpoint: &Endpoint, wire: Option<&&BTreeSet<Endpoint>>) -> String {
        match wire {
            None => "nothing".to_string(),
            Some(wire) => wire
                .iter()
                .filter(|other| *other != endpoint)
                .map(Endpoint::to_string)
                .collect::<Vec<_>>()
                .join(", "),
        }
    }
    let (in_schematic, in_hdl) = (index(expected), index(observed));
    let endpoints: BTreeSet<&Endpoint> =
        in_schematic.keys().chain(in_hdl.keys()).copied().collect();
    endpoints
        .into_iter()
        .filter(|endpoint| in_schematic.get(endpoint) != in_hdl.get(endpoint))
        .map(|endpoint| {
            format!(
                "{endpoint}: the schematic connects it to {}; the generated HDL connects it to {}",
                others(endpoint, in_schematic.get(endpoint)),
                others(endpoint, in_hdl.get(endpoint))
            )
        })
        .collect()
}

/// The result of the check.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The generated HDL connects what the schematic connects.
    Same {
        /// The number of wires that were compared.
        wires: usize,
        /// The number of pins that the check ignored (see `Expected`).
        unchecked: usize,
    },
    /// The two sides do not agree. One line for each pin bit.
    Different(Vec<String>),
}

/// Compare the schematic with the netlist that yosys wrote as JSON.
/// `Err` is a reason why the netlist cannot be compared.
pub fn compare(
    schematic: &Schematic,
    library: &[ModuleDef],
    json: &str,
) -> Result<Outcome, String> {
    let fold_case = schematic.language == Language::Vhdl;
    let expected = expected(schematic, library);
    let mut observed = observed(json, &schematic.top_name, fold_case)?;
    if !expected.unchecked.is_empty() {
        observed = observed
            .into_iter()
            .map(|wire| {
                wire.into_iter()
                    .filter(|e| {
                        !expected
                            .unchecked
                            .contains(&(e.instance.clone(), e.port.clone()))
                    })
                    .collect::<BTreeSet<Endpoint>>()
            })
            .filter(|wire| wire.len() > 1)
            .collect();
    }
    let lines = diff(&expected.nets, &observed);
    Ok(if lines.is_empty() {
        Outcome::Same {
            wires: expected.nets.len(),
            unchecked: expected.unchecked.len(),
        }
    } else {
        Outcome::Different(lines)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bits(high: i64, low: i64) -> PortType {
        PortType::StdLogicVector(Range {
            high: RangeExpr::Literal(high),
            low: RangeExpr::Literal(low),
            dir: RangeDir::Downto,
        })
    }

    fn port(name: &str, direction: Direction, port_type: PortType) -> PortDef {
        PortDef {
            name: name.to_string(),
            direction,
            port_type,
            bundle: None,
        }
    }

    fn module(name: &str, ports: Vec<PortDef>) -> ModuleDef {
        ModuleDef {
            name: name.to_string(),
            generics: Vec::new(),
            ports,
            source_path: format!("{name}.sv").into(),
            source_hash: 0,
            dependencies: Vec::new(),
        }
    }

    fn pin(instance: &str, port: &str) -> NetRef {
        NetRef::InstancePort(instance.to_string(), port.to_string())
    }

    fn end(instance: Option<&str>, port: &str, bit: u32) -> Endpoint {
        Endpoint {
            instance: instance.map(str::to_string),
            port: port.to_string(),
            bit,
        }
    }

    fn wire(endpoints: &[Endpoint]) -> BTreeSet<Endpoint> {
        endpoints.iter().cloned().collect()
    }

    /// `src.q` (4 bits) drives `dst.d` (4 bits); `clk` goes to both.
    fn design() -> (Schematic, Vec<ModuleDef>) {
        let library = vec![
            module(
                "source",
                vec![
                    port("clk", Direction::In, PortType::StdLogic),
                    port("q", Direction::Out, bits(3, 0)),
                ],
            ),
            module(
                "sink",
                vec![
                    port("clk", Direction::In, PortType::StdLogic),
                    port("d", Direction::In, bits(3, 0)),
                ],
            ),
        ];
        let mut s = Schematic::new("top", Language::SystemVerilog);
        s.top_ports = vec![port("clk", Direction::In, PortType::StdLogic)];
        s.add_instance("src", "source").unwrap();
        s.add_instance("dst", "sink").unwrap();
        let clk = Some(NetRef::TopPort("clk".to_string()));
        s.set_port_map_entry("src", "clk".to_string(), clk.clone())
            .unwrap();
        s.set_port_map_entry("dst", "clk".to_string(), clk).unwrap();
        s.set_port_map_entry("dst", "d".to_string(), Some(pin("src", "q")))
            .unwrap();
        (s, library)
    }

    /// The netlist of `design`, as yosys writes it. `d_bits` are the wires
    /// on the four bits of `dst.d`.
    fn netlist(d_bits: &str) -> String {
        format!(
            r#"{{"modules": {{"top": {{
                "ports": {{"clk": {{"direction": "input", "bits": [2]}}}},
                "cells": {{
                    "src": {{"type": "source", "connections": {{"clk": [2], "q": [3, 4, 5, 6]}}}},
                    "dst": {{"type": "sink", "connections": {{"clk": [2], "d": {d_bits}}}}}
                }}
            }}}}}}"#
        )
    }

    #[test]
    fn expected_has_one_wire_for_each_bit_of_a_net() {
        let (s, library) = design();
        let expected = expected(&s, &library);
        assert!(expected.unchecked.is_empty());
        assert_eq!(expected.nets.len(), 5);
        assert!(expected.nets.contains(&wire(&[
            end(None, "clk", 0),
            end(Some("dst"), "clk", 0),
            end(Some("src"), "clk", 0),
        ])));
        for bit in 0..4 {
            assert!(expected.nets.contains(&wire(&[
                end(Some("dst"), "d", bit),
                end(Some("src"), "q", bit)
            ])));
        }
    }

    #[test]
    fn netlist_that_connects_the_same_pins_is_the_same() {
        let (s, library) = design();
        assert_eq!(
            compare(&s, &library, &netlist("[3, 4, 5, 6]")),
            Ok(Outcome::Same {
                wires: 5,
                unchecked: 0
            })
        );
    }

    #[test]
    fn netlist_with_two_swapped_bits_is_different() {
        let (s, library) = design();
        let Ok(Outcome::Different(lines)) = compare(&s, &library, &netlist("[4, 3, 5, 6]")) else {
            panic!("the swap was not found");
        };
        assert_eq!(
            lines,
            [
                "dst.d bit 0: the schematic connects it to src.q bit 0; the generated HDL connects it to src.q bit 1",
                "dst.d bit 1: the schematic connects it to src.q bit 1; the generated HDL connects it to src.q bit 0",
                "src.q bit 0: the schematic connects it to dst.d bit 0; the generated HDL connects it to dst.d bit 1",
                "src.q bit 1: the schematic connects it to dst.d bit 1; the generated HDL connects it to dst.d bit 0",
            ]
        );
    }

    #[test]
    fn netlist_with_an_open_pin_is_different() {
        let (s, library) = design();
        let Ok(Outcome::Different(lines)) = compare(&s, &library, &netlist("[3, 4, 5, 9]")) else {
            panic!("the open bit was not found");
        };
        assert_eq!(
            lines,
            [
                "dst.d bit 3: the schematic connects it to src.q bit 3; the generated HDL connects it to nothing",
                "src.q bit 3: the schematic connects it to dst.d bit 3; the generated HDL connects it to nothing",
            ]
        );
    }

    #[test]
    fn netlist_with_a_wire_that_the_schematic_does_not_have_is_different() {
        let (mut s, library) = design();
        s.set_port_map_entry("dst", "d".to_string(), None).unwrap();
        let Ok(Outcome::Different(lines)) = compare(&s, &library, &netlist("[3, 4, 5, 6]")) else {
            panic!("the extra wires were not found");
        };
        assert_eq!(lines.len(), 8);
        assert!(lines[0].starts_with("dst.d bit 0: the schematic connects it to nothing;"));
    }

    #[test]
    fn a_constant_in_the_netlist_is_not_a_wire() {
        let (mut s, library) = design();
        s.set_port_map_entry(
            "dst",
            "d".to_string(),
            Some(NetRef::Constant("4'b1010".to_string())),
        )
        .unwrap();
        assert_eq!(
            compare(&s, &library, &netlist(r#"["0", "1", "0", "1"]"#)),
            Ok(Outcome::Same {
                wires: 1,
                unchecked: 0
            })
        );
    }

    #[test]
    fn a_reference_with_a_slice_puts_the_pin_on_those_bits_of_the_net() {
        let (mut s, mut library) = design();
        library[1].ports[1].port_type = bits(1, 0);
        s.set_port_map_entry(
            "dst",
            "d".to_string(),
            Some(NetRef::InstancePortSlice(
                "src".to_string(),
                "q".to_string(),
                SliceExpr::Range { high: 3, low: 2 },
            )),
        )
        .unwrap();
        let expected = expected(&s, &library);
        assert!(
            expected
                .nets
                .contains(&wire(&[end(Some("dst"), "d", 0), end(Some("src"), "q", 2)]))
        );
        assert!(
            expected
                .nets
                .contains(&wire(&[end(Some("dst"), "d", 1), end(Some("src"), "q", 3)]))
        );
        assert_eq!(expected.nets.len(), 3);
    }

    /// A slice uses the index numbers of the port: bits 9 and 8 of a
    /// `(11 downto 8)` port are its two lowest bits.
    #[test]
    fn a_slice_of_a_port_that_does_not_start_at_0_uses_the_index_numbers() {
        let (mut s, mut library) = design();
        library[0].ports[1].port_type = bits(11, 8);
        library[1].ports[1].port_type = bits(1, 0);
        s.set_port_map_entry(
            "dst",
            "d".to_string(),
            Some(NetRef::InstancePortSlice(
                "src".to_string(),
                "q".to_string(),
                SliceExpr::Range { high: 9, low: 8 },
            )),
        )
        .unwrap();
        let expected = expected(&s, &library);
        assert!(
            expected
                .nets
                .contains(&wire(&[end(Some("dst"), "d", 0), end(Some("src"), "q", 0)]))
        );
        assert!(
            expected
                .nets
                .contains(&wire(&[end(Some("dst"), "d", 1), end(Some("src"), "q", 1)]))
        );
    }

    #[test]
    fn a_consumer_slice_connects_only_those_bits_of_the_own_port() {
        let (mut s, mut library) = design();
        library[1].ports[1].port_type = bits(7, 0);
        s.instances
            .iter_mut()
            .find(|i| i.name == "dst")
            .unwrap()
            .consumer_slices
            .insert("d".to_string(), SliceExpr::Range { high: 7, low: 4 });
        let expected = expected(&s, &library);
        for bit in 0..4 {
            assert!(expected.nets.contains(&wire(&[
                end(Some("dst"), "d", bit + 4),
                end(Some("src"), "q", bit)
            ])));
        }
        assert_eq!(expected.nets.len(), 5);
    }

    #[test]
    fn a_width_from_a_top_level_generic_uses_the_default_value() {
        let (mut s, mut library) = design();
        s.top_generics = vec![GenericDef {
            name: "W".to_string(),
            type_name: "integer".to_string(),
            default_value: Some("4".to_string()),
        }];
        for m in &mut library {
            m.generics = vec![GenericDef {
                name: "WIDTH".to_string(),
                type_name: "integer".to_string(),
                default_value: Some("1".to_string()),
            }];
            m.ports[1].port_type = PortType::StdLogicVector(Range {
                high: RangeExpr::Expr("WIDTH-1".to_string()),
                low: RangeExpr::Literal(0),
                dir: RangeDir::Downto,
            });
        }
        for instance in &mut s.instances {
            instance
                .generic_map
                .insert("WIDTH".to_string(), "W".to_string());
        }
        assert_eq!(
            compare(&s, &library, &netlist("[3, 4, 5, 6]")),
            Ok(Outcome::Same {
                wires: 5,
                unchecked: 0
            })
        );
    }

    #[test]
    fn a_net_with_a_pin_of_no_known_width_is_not_checked() {
        let (s, mut library) = design();
        library[0].ports[1].port_type = PortType::Record("bus_t".to_string());
        // The netlist differs on that net, and the check cannot know.
        assert_eq!(
            compare(&s, &library, &netlist("[4, 3, 5, 6]")),
            Ok(Outcome::Same {
                wires: 1,
                unchecked: 2
            })
        );
    }

    #[test]
    fn vhdl_names_compare_without_case() {
        let (mut s, library) = design();
        s.language = Language::Vhdl;
        s.top_name = "Top".to_string();
        s.instances[0].name = "Src".to_string();
        s.instances[1]
            .port_map
            .insert("d".to_string(), Some(pin("Src", "q")));
        assert_eq!(
            compare(&s, &library, &netlist("[3, 4, 5, 6]")),
            Ok(Outcome::Same {
                wires: 5,
                unchecked: 0
            })
        );
    }

    #[test]
    fn a_cell_of_a_flattened_group_has_the_name_of_the_instance() {
        let (s, library) = design();
        let json = netlist("[3, 4, 5, 6]").replace(r#""src": {"#, r#""u_core.src": {"#);
        assert_eq!(
            compare(&s, &library, &json),
            Ok(Outcome::Same {
                wires: 5,
                unchecked: 0
            })
        );
    }

    #[test]
    fn logic_in_the_top_level_cannot_be_compared() {
        let (s, library) = design();
        let json = netlist("[3, 4, 5, 6]").replace(r#""type": "sink""#, r#""type": "$and""#);
        let error = compare(&s, &library, &json).unwrap_err();
        assert!(error.contains("`$and` cell"), "{error}");
        assert!(compare(&s, &library, "{").is_err());
        assert!(compare(&s, &library, r#"{"modules": {}}"#).is_err());
    }

    #[test]
    fn ascending_index_numbers_count_from_the_right_hand_end() {
        let to = PortType::StdLogicVector(Range {
            high: RangeExpr::Literal(7),
            low: RangeExpr::Literal(0),
            dir: RangeDir::To,
        });
        assert_eq!(
            connected_positions(&to, Some(&SliceExpr::Range { high: 1, low: 0 })),
            Some(vec![7, 6])
        );
        assert_eq!(
            connected_positions(&bits(15, 8), None),
            Some((0..8).collect())
        );
        assert_eq!(
            connected_positions(&PortType::StdLogic, None),
            Some(vec![0])
        );
        // An index outside the port has no place.
        assert_eq!(
            connected_positions(&bits(3, 0), Some(&SliceExpr::Bit(4))),
            None
        );
    }
}
