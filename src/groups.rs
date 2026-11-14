//! Hierarchical grouping: collapse named sets of instances into synthesized
//! sub-modules.
//!
//! Each group becomes its own entity/module — nets crossing the group
//! boundary become the group module's top ports (named by the net), and the
//! parent level replaces the member instances with a single instance of the
//! group module. Groups nest via `parent`; collapse runs children-first so a
//! parent group sees its child groups as ordinary instances.

use std::collections::{HashMap, HashSet};

use crate::nets::{pin_direction, resolve_nets};
use crate::schematic::Diagnostic;
use crate::types::*;

/// Fully expanded hierarchy: one schematic per group (children first) plus
/// the rewritten top. `library` includes the synthesized group ModuleDefs,
/// so both group and top schematics validate/codegen against it.
pub struct HierarchyPlan {
    /// (group name, schematic) in emit order — children before parents.
    pub groups: Vec<(String, Schematic)>,
    pub top: Schematic,
    pub library: Vec<ModuleDef>,
}

/// Structural checks on the group table itself. Codegen refuses on errors.
pub fn validate_groups(s: &Schematic, library: &[ModuleDef]) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let instance_names: HashSet<&str> = s.instances.iter().map(|i| i.name.as_str()).collect();
    let group_names: HashSet<&str> = s.groups.iter().map(|g| g.name.as_str()).collect();

    let mut seen_groups: HashSet<&str> = HashSet::new();
    let mut member_of: HashMap<&str, &str> = HashMap::new();
    for g in &s.groups {
        if !seen_groups.insert(&g.name) {
            diags.push(Diagnostic::error(format!(
                "duplicate group name '{}'",
                g.name
            )));
        }
        if instance_names.contains(g.name.as_str()) {
            diags.push(Diagnostic::error(format!(
                "group '{}' collides with an instance of the same name",
                g.name
            )));
        }
        if library.iter().any(|m| m.name == g.name) {
            diags.push(Diagnostic::error(format!(
                "group '{}' collides with library module of the same name",
                g.name
            )));
        }
        if g.members.is_empty() {
            diags.push(Diagnostic::error(format!(
                "group '{}' has no members",
                g.name
            )));
        }
        for m in &g.members {
            if !instance_names.contains(m.as_str()) {
                diags.push(Diagnostic::error(format!(
                    "group '{}' member '{m}' is not an instance",
                    g.name
                )));
            }
            if let Some(other) = member_of.insert(m, &g.name)
                && other != g.name
            {
                diags.push(Diagnostic::error(format!(
                    "instance '{m}' is in two groups: '{other}' and '{}'",
                    g.name
                )));
            }
        }
        if let Some(p) = &g.parent
            && !group_names.contains(p.as_str())
        {
            diags.push(Diagnostic::error(format!(
                "group '{}' parent '{p}' does not exist",
                g.name
            )));
        }
    }

    // Parent-chain cycles.
    let parent_of: HashMap<&str, &str> = s
        .groups
        .iter()
        .filter_map(|g| g.parent.as_deref().map(|p| (g.name.as_str(), p)))
        .collect();
    for g in &s.groups {
        let mut seen: HashSet<&str> = HashSet::new();
        let mut cur = g.name.as_str();
        while let Some(&p) = parent_of.get(cur) {
            if !seen.insert(p) {
                diags.push(Diagnostic::error(format!(
                    "group '{}' has a cyclic parent chain",
                    g.name
                )));
                break;
            }
            cur = p;
        }
    }
    diags
}

/// Depth of a group's parent chain (0 = top level).
fn depth(g: &Group, by_name: &HashMap<&str, &Group>) -> usize {
    let mut d = 0;
    let mut cur = g;
    while let Some(p) = cur.parent.as_deref().and_then(|p| by_name.get(p)) {
        d += 1;
        cur = p;
        if d > 64 {
            break; // cycle guard; validate_groups reports it
        }
    }
    d
}

pub struct BoundaryPort {
    pub name: String,
    pub direction: Direction,
    pub port_type: PortType,
    /// One representative pin outside the group on this net (driver
    /// preferred) — what the parent-level group instance connects to.
    pub outside_ref: Option<NetRef>,
}

/// Derive the boundary ports of a fence: nets with pins both inside and
/// outside `inside` become ports. Also returns the pin → port-name map for
/// every pin on a boundary net.
pub fn derive_boundary(
    s: &Schematic,
    library: &[ModuleDef],
    inside: &HashSet<String>,
) -> (Vec<BoundaryPort>, HashMap<NetRef, String>) {
    let lib_map: HashMap<&str, &ModuleDef> = library.iter().map(|m| (m.name.as_str(), m)).collect();
    let nets = resolve_nets(s, library);
    let is_inside = |r: &NetRef| match r {
        NetRef::InstancePort(i, _) => inside.contains(i),
        _ => false, // top ports and constants are outside
    };
    let mut ports: Vec<BoundaryPort> = Vec::new();
    let mut port_for_pin: HashMap<NetRef, String> = HashMap::new();
    let mut used_names: HashSet<String> = HashSet::new();
    for net in &nets.nets {
        let (ins, outs): (Vec<&NetRef>, Vec<&NetRef>) =
            net.members.iter().partition(|r| is_inside(r));
        if ins.is_empty() || outs.is_empty() {
            continue;
        }
        let any_inout = net
            .members
            .iter()
            .any(|r| pin_direction(s, &lib_map, r) == Some(Direction::InOut));
        let driver_inside = net.drivers.iter().any(&is_inside);
        let direction = if any_inout {
            Direction::InOut
        } else if driver_inside {
            Direction::Out
        } else {
            Direction::In
        };
        let outside_ref = net
            .drivers
            .iter()
            .find(|d| !is_inside(d))
            .or(outs.first().copied())
            .cloned();
        let port_type = net.port_type.clone().unwrap_or(PortType::StdLogic);
        // Port name: prefer the parent top port's bare name (a net named
        // clk_s would otherwise spawn a clk_s_s intermediate inside the
        // group); fall back to the net name on collision.
        let top_member = net.members.iter().find_map(|r| match r {
            NetRef::TopPort(n) => Some(n.clone()),
            _ => None,
        });
        let mut pname = top_member.unwrap_or_else(|| net.name.clone());
        if !used_names.insert(pname.clone()) {
            pname = net.name.clone();
            used_names.insert(pname.clone());
        }
        for pin in &net.members {
            port_for_pin.insert(pin.clone(), pname.clone());
        }
        ports.push(BoundaryPort {
            name: pname,
            direction,
            port_type,
            outside_ref,
        });
    }
    ports.sort_by(|a, b| a.name.cmp(&b.name));
    (ports, port_for_pin)
}

/// Transitive instance membership of a group: direct members plus members
/// of descendant groups. What the GUI hides when the group is collapsed.
pub fn transitive_members(s: &Schematic, group_name: &str) -> HashSet<String> {
    let mut out: HashSet<String> = HashSet::new();
    let mut queue: Vec<&str> = vec![group_name];
    let mut seen: HashSet<&str> = HashSet::new();
    while let Some(gname) = queue.pop() {
        if !seen.insert(gname) {
            continue;
        }
        if let Some(g) = s.groups.iter().find(|g| g.name == gname) {
            out.extend(g.members.iter().cloned());
        }
        for child in s
            .groups
            .iter()
            .filter(|c| c.parent.as_deref() == Some(gname))
        {
            queue.push(&child.name);
        }
    }
    out
}

/// Collapse every group. Returns the per-group schematics (children first),
/// the rewritten top, and the library extended with synthesized group
/// modules. Call only after `validate_groups` reports no errors.
pub fn expand_hierarchy(schematic: &Schematic, library: &[ModuleDef]) -> HierarchyPlan {
    let mut cur = schematic.clone();
    let mut lib: Vec<ModuleDef> = library.to_vec();
    let mut out_groups: Vec<(String, Schematic)> = Vec::new();

    // Children first: deeper parent chains collapse earlier.
    let by_name: HashMap<&str, &Group> = schematic
        .groups
        .iter()
        .map(|g| (g.name.as_str(), g))
        .collect();
    let mut order: Vec<&Group> = schematic.groups.iter().collect();
    order.sort_by_key(|g| std::cmp::Reverse(depth(g, &by_name)));

    // Effective member lists grow as child groups collapse into instances.
    let mut members: HashMap<String, Vec<String>> = schematic
        .groups
        .iter()
        .map(|g| (g.name.clone(), g.members.clone()))
        .collect();

    for g in order {
        let inside: HashSet<String> = members
            .get(&g.name)
            .map(|m| m.iter().cloned().collect())
            .unwrap_or_default();

        let (ports, port_for_pin) = derive_boundary(&cur, &lib, &inside);
        let is_inside = |r: &NetRef| match r {
            NetRef::InstancePort(i, _) => inside.contains(i),
            _ => false, // top ports and constants are outside
        };

        // Build the group schematic: member instances with outside refs
        // rewritten to the derived top ports.
        let mut gs = Schematic::new(&g.name, cur.language.clone());
        gs.library_paths = cur.library_paths.clone();
        gs.top_ports = ports
            .iter()
            .map(|p| PortDef {
                name: p.name.clone(),
                direction: p.direction.clone(),
                port_type: p.port_type.clone(),
                bundle: None,
            })
            .collect();
        for inst in cur.instances.iter().filter(|i| inside.contains(&i.name)) {
            let mut inst = inst.clone();
            for entry in inst.port_map.values_mut() {
                let Some(r) = entry else { continue };
                if matches!(r, NetRef::Constant(_)) || is_inside(&r.base()) {
                    continue;
                }
                // Outside pin → the boundary port carrying that net.
                if let Some(port_name) = port_for_pin.get(&r.base()) {
                    *entry = Some(match r {
                        NetRef::InstancePortSlice(_, _, s) | NetRef::TopPortSlice(_, s) => {
                            NetRef::TopPortSlice(port_name.clone(), s.clone())
                        }
                        _ => NetRef::TopPort(port_name.clone()),
                    });
                }
            }
            gs.instances.push(inst);
        }
        // Aliases whose pin lives inside move into the group.
        gs.aliases = cur
            .aliases
            .iter()
            .filter(|(k, _)| is_inside(&k.base()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        // Top generics referenced by member generic_maps travel with the
        // group: they become the group module's generics, and the parent
        // instance passes them straight through by name. The value can be
        // the name of the generic or an expression that uses it. A top generic in
        // the width of a boundary port travels too: the net can get its
        // width from an instance outside the group.
        let passthrough: Vec<GenericDef> = cur
            .top_generics
            .iter()
            .filter(|tg| {
                gs.instances.iter().any(|i| {
                    i.generic_map
                        .values()
                        .any(|v| crate::codegen::generic_value_mentions(v, &tg.name))
                }) || gs
                    .top_ports
                    .iter()
                    .any(|p| crate::codegen::port_type_mentions(&p.port_type, &tg.name))
            })
            .cloned()
            .collect();
        gs.top_generics = passthrough.clone();

        // Synthesized module definition for the parent library.
        lib.push(ModuleDef {
            name: g.name.clone(),
            generics: passthrough.clone(),
            ports: gs.top_ports.clone(),
            source_path: std::path::PathBuf::from(format!("{}.generated", g.name)),
            source_hash: 0,
            dependencies: Vec::new(),
        });

        // Rewrite the parent: members out, one group instance in.
        let centroid = {
            let pts: Vec<(f32, f32)> = cur
                .instances
                .iter()
                .filter(|i| inside.contains(&i.name))
                .map(|i| i.position)
                .collect();
            if g.position != (0.0, 0.0) || pts.is_empty() {
                g.position
            } else {
                let n = pts.len() as f32;
                (
                    pts.iter().map(|p| p.0).sum::<f32>() / n,
                    pts.iter().map(|p| p.1).sum::<f32>() / n,
                )
            }
        };
        cur.instances.retain(|i| !inside.contains(&i.name));
        // VHDL forbids an instance label matching the component name — use a
        // u_ prefix, uniquified against surviving instances.
        let mut inst_name = format!("u_{}", g.name);
        while cur.instances.iter().any(|i| i.name == inst_name) {
            inst_name.push('_');
        }
        let mut group_port_map: HashMap<String, Option<NetRef>> = HashMap::new();
        for p in &ports {
            // Nets driven outside (or via a top port either way) get an
            // explicit entry; inside-driven internal nets are connected by
            // the outside consumers referencing the group pin.
            if let Some(r) = &p.outside_ref
                && (p.direction != Direction::Out || matches!(r, NetRef::TopPort(_)))
            {
                group_port_map.insert(p.name.clone(), Some(r.clone()));
            }
        }
        cur.instances.push(Instance {
            name: inst_name.clone(),
            module_ref: g.name.clone(),
            generic_map: passthrough
                .iter()
                .map(|g| (g.name.clone(), g.name.clone()))
                .collect(),
            port_map: group_port_map,
            position: centroid,
            manual_bundles: HashMap::new(),
            consumer_slices: HashMap::new(),
            dirty: false,
            locked: false,
        });
        // Outside pins that referenced inside pins now reference the group pin.
        for inst in cur.instances.iter_mut() {
            if inst.name == inst_name {
                continue;
            }
            for entry in inst.port_map.values_mut() {
                let Some(r) = entry else { continue };
                if let NetRef::InstancePort(i, _) | NetRef::InstancePortSlice(i, _, _) = r
                    && inside.contains(i)
                    && let Some(port_name) = port_for_pin.get(&r.base())
                {
                    *entry = Some(match r {
                        NetRef::InstancePortSlice(_, _, s) => NetRef::InstancePortSlice(
                            inst_name.clone(),
                            port_name.clone(),
                            s.clone(),
                        ),
                        _ => NetRef::InstancePort(inst_name.clone(), port_name.clone()),
                    });
                }
            }
        }
        // Drop aliases that moved inside.
        cur.aliases.retain(|k, _| !is_inside(&k.base()));
        cur.groups.retain(|other| other.name != g.name);

        // Register this collapsed group's INSTANCE as a member of its parent.
        if let Some(parent) = &g.parent
            && let Some(pm) = members.get_mut(parent)
        {
            pm.push(inst_name.clone());
        }

        out_groups.push((g.name.clone(), gs));
    }

    HierarchyPlan {
        groups: out_groups,
        top: cur,
        library: lib,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn module(name: &str, ports: Vec<(&str, Direction)>) -> ModuleDef {
        ModuleDef {
            name: name.into(),
            generics: vec![],
            ports: ports
                .into_iter()
                .map(|(p, d)| PortDef {
                    name: p.into(),
                    direction: d,
                    port_type: PortType::StdLogic,
                    bundle: None,
                })
                .collect(),
            source_path: format!("{name}.vhd").into(),
            source_hash: 0,
            dependencies: Vec::new(),
        }
    }

    /// top clk -> u_a(mod_a: clk in, dout out) -> u_b(mod_b: din in, dout out) -> top led.
    /// u_a and u_b grouped as "chain".
    fn grouped_fixture() -> (Schematic, Vec<ModuleDef>) {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.top_ports.push(PortDef {
            name: "clk".into(),
            direction: Direction::In,
            port_type: PortType::StdLogic,
            bundle: None,
        });
        s.top_ports.push(PortDef {
            name: "led".into(),
            direction: Direction::Out,
            port_type: PortType::StdLogic,
            bundle: None,
        });
        s.add_instance("u_a", "mod_a").unwrap();
        s.add_instance("u_b", "mod_b").unwrap();
        s.add_instance("u_c", "mod_b").unwrap();
        s.set_port_map_entry("u_a", "clk", Some(NetRef::TopPort("clk".into())))
            .unwrap();
        s.set_port_map_entry(
            "u_b",
            "din",
            Some(NetRef::InstancePort("u_a".into(), "dout".into())),
        )
        .unwrap();
        // u_c stays outside the group, consuming an inside-driven net.
        s.set_port_map_entry(
            "u_c",
            "din",
            Some(NetRef::InstancePort("u_b".into(), "dout".into())),
        )
        .unwrap();
        s.set_port_map_entry("u_c", "dout", Some(NetRef::TopPort("led".into())))
            .unwrap();
        s.groups.push(Group {
            name: "chain".into(),
            members: vec!["u_a".into(), "u_b".into()],
            parent: None,
            collapsed: false,
            position: (0.0, 0.0),
        });
        let lib = vec![
            module(
                "mod_a",
                vec![("clk", Direction::In), ("dout", Direction::Out)],
            ),
            module(
                "mod_b",
                vec![("din", Direction::In), ("dout", Direction::Out)],
            ),
        ];
        (s, lib)
    }

    /// Regression: the GUI wrote only the top-level file for a grouped
    /// project. The CLI and the GUI now share `GeneratedDesign::write_to`.
    #[test]
    fn grouped_design_writes_one_file_for_each_group() {
        let (s, lib) = grouped_fixture();
        let design = crate::codegen::generate_design(&s, &lib).unwrap();
        let dir = tempfile::TempDir::new().unwrap();
        let top = dir.path().join("top.vhd");

        let written = design.write_to(&top).unwrap();

        assert_eq!(written, vec![dir.path().join("chain.vhd"), top.clone()]);
        let chain = std::fs::read_to_string(dir.path().join("chain.vhd")).unwrap();
        assert!(chain.contains("entity chain is"), "{chain}");
        let top_code = std::fs::read_to_string(&top).unwrap();
        assert!(top_code.contains("u_chain"), "{top_code}");
        // The flat emitter the GUI used before knows nothing about groups.
        let flat =
            crate::codegen::vhdl::generate_vhdl(&s, &lib, &s.validate(&lib)).unwrap_or_default();
        assert!(!flat.contains("u_chain"), "{flat}");
    }

    #[test]
    fn validate_groups_catches_bad_members_and_cycles() {
        let (mut s, lib) = grouped_fixture();
        s.groups.push(Group {
            name: "g2".into(),
            members: vec!["nope".into()],
            parent: Some("g3".into()),
            collapsed: false,
            position: (0.0, 0.0),
        });
        s.groups.push(Group {
            name: "g3".into(),
            members: vec!["u_c".into()],
            parent: Some("g2".into()),
            collapsed: false,
            position: (0.0, 0.0),
        });
        let diags = validate_groups(&s, &lib);
        let msgs: Vec<&str> = diags.iter().map(|d| d.message.as_str()).collect();
        assert!(
            msgs.iter().any(|m| m.contains("not an instance")),
            "{msgs:?}"
        );
        assert!(msgs.iter().any(|m| m.contains("cyclic")), "{msgs:?}");
    }

    #[test]
    fn expand_derives_boundary_ports() {
        let (s, lib) = grouped_fixture();
        assert!(validate_groups(&s, &lib).is_empty());
        let plan = expand_hierarchy(&s, &lib);
        assert_eq!(plan.groups.len(), 1);
        let (name, gs) = &plan.groups[0];
        assert_eq!(name, "chain");
        // Two boundary nets: clk (driven outside → In) and u_b.dout
        // (driven inside, consumed by u_c → Out).
        let mut dirs: Vec<(String, Direction)> = gs
            .top_ports
            .iter()
            .map(|p| (p.name.clone(), p.direction.clone()))
            .collect();
        dirs.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(
            dirs,
            vec![
                ("clk".to_string(), Direction::In),
                ("u_b_dout".to_string(), Direction::Out),
            ],
            "boundary ports: {dirs:?}"
        );
        // Inside refs rewritten to group-top ports.
        let u_a = gs.instances.iter().find(|i| i.name == "u_a").unwrap();
        assert_eq!(
            u_a.port_map.get("clk").cloned().flatten(),
            Some(NetRef::TopPort("clk".into()))
        );
        // Internal net untouched.
        let u_b = gs.instances.iter().find(|i| i.name == "u_b").unwrap();
        assert_eq!(
            u_b.port_map.get("din").cloned().flatten(),
            Some(NetRef::InstancePort("u_a".into(), "dout".into()))
        );
    }

    #[test]
    fn expand_rewrites_parent_level() {
        let (s, lib) = grouped_fixture();
        let plan = expand_hierarchy(&s, &lib);
        let top = &plan.top;
        // Members replaced by one instance of the synthesized module.
        assert!(
            top.instances
                .iter()
                .all(|i| i.name != "u_a" && i.name != "u_b")
        );
        let gi = top.instances.iter().find(|i| i.name == "u_chain").unwrap();
        assert_eq!(gi.module_ref, "chain");
        // Group instance connects its In port to the top clk.
        assert_eq!(
            gi.port_map.get("clk").cloned().flatten(),
            Some(NetRef::TopPort("clk".into()))
        );
        // Outside consumer now references the group pin.
        let u_c = top.instances.iter().find(|i| i.name == "u_c").unwrap();
        assert_eq!(
            u_c.port_map.get("din").cloned().flatten(),
            Some(NetRef::InstancePort("u_chain".into(), "u_b_dout".into()))
        );
        // Library gained the synthesized def.
        assert!(plan.library.iter().any(|m| m.name == "chain"));
        // Top validates clean against the extended library.
        let diags = top.validate(&plan.library);
        assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
        // Group schematic validates clean too.
        let (_, gs) = &plan.groups[0];
        let gdiags = gs.validate(&plan.library);
        assert!(gdiags.iter().all(|d| !d.is_error()), "{gdiags:?}");
    }

    #[test]
    fn nested_groups_collapse_children_first() {
        let (mut s, lib) = grouped_fixture();
        // Put u_c in an outer group that also contains the "chain" group.
        s.groups.push(Group {
            name: "outer".into(),
            members: vec!["u_c".into()],
            parent: None,
            collapsed: false,
            position: (0.0, 0.0),
        });
        s.groups[0].parent = Some("outer".into());
        let plan = expand_hierarchy(&s, &lib);
        assert_eq!(plan.groups.len(), 2);
        // chain collapses first, then outer swallows the chain instance.
        assert_eq!(plan.groups[0].0, "chain");
        assert_eq!(plan.groups[1].0, "outer");
        let outer = &plan.groups[1].1;
        assert!(outer.instances.iter().any(|i| i.name == "u_chain"));
        assert!(outer.instances.iter().any(|i| i.name == "u_c"));
        // Top holds only the outer instance now.
        assert_eq!(plan.top.instances.len(), 1);
        assert_eq!(plan.top.instances[0].name, "u_outer");
        let diags = plan.top.validate(&plan.library);
        assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
    }

    /// The width of the net `u_gen.count` comes from the top-level generic
    /// `T`. The net goes into the group `inner`, where no instance uses `T`:
    /// `u_sink` gets its width from the top-level generic `U`. The port of
    /// the group has the width of the net, so the group must get `T` too.
    #[test]
    fn top_generic_in_a_boundary_port_width_travels_with_the_group() {
        let generic = |name: &str, default_value: &str| GenericDef {
            name: name.into(),
            type_name: "integer".into(),
            default_value: Some(default_value.into()),
        };
        let vector_of = |generic_name: &str| {
            PortType::StdLogicVector(Range {
                high: RangeExpr::Expr(format!("{generic_name} - 1")),
                low: RangeExpr::Literal(0),
                dir: RangeDir::Downto,
            })
        };
        let port = |name: &str, direction: Direction, port_type: PortType| PortDef {
            name: name.into(),
            direction,
            port_type,
            bundle: None,
        };
        let mut pulse_gen = module("pulse_gen", vec![]);
        pulse_gen.generics.push(generic("WIDTH", "8"));
        pulse_gen
            .ports
            .push(port("count", Direction::Out, vector_of("WIDTH")));
        let mut sink = module("sink", vec![("led", Direction::Out)]);
        sink.generics.push(generic("N", "8"));
        sink.ports
            .push(port("value", Direction::In, vector_of("N")));

        let mut s = Schematic::new("top", Language::Vhdl);
        s.top_generics.push(generic("T", "16"));
        s.top_generics.push(generic("U", "16"));
        s.top_ports
            .push(port("led", Direction::Out, PortType::StdLogic));
        s.add_instance("u_gen", "pulse_gen").unwrap();
        s.set_generic_map_entry("u_gen", "WIDTH", "T").unwrap();
        s.add_instance("u_sink", "sink").unwrap();
        s.set_generic_map_entry("u_sink", "N", "U").unwrap();
        s.set_port_map_entry(
            "u_sink",
            "value",
            Some(NetRef::InstancePort("u_gen".into(), "count".into())),
        )
        .unwrap();
        s.set_port_map_entry("u_sink", "led", Some(NetRef::TopPort("led".into())))
            .unwrap();
        s.groups.push(Group {
            name: "inner".into(),
            members: vec!["u_sink".into()],
            parent: None,
            collapsed: false,
            position: (0.0, 0.0),
        });

        let design = crate::codegen::generate_design(&s, &[pulse_gen, sink]).unwrap();
        let (_, inner) = &design.files[0];
        assert!(
            inner.contains("u_gen_count : in std_logic_vector(T - 1 downto 0)"),
            "{inner}"
        );
        assert!(inner.contains("T : integer := 16"), "{inner}");
        assert!(design.top_code.contains("T => T"), "{}", design.top_code);
    }

    /// Regression: `u_t` is in the group and sets `DIV` to the expression
    /// `2 * TOP_DIV`. Only a value that was exactly the name of a top-level
    /// generic carried the generic into the group, so the group used
    /// `TOP_DIV` and did not declare it.
    #[test]
    fn top_generic_in_a_generic_map_expression_travels_with_the_group() {
        let generic = |name: &str, default_value: &str| GenericDef {
            name: name.into(),
            type_name: "integer".into(),
            default_value: Some(default_value.into()),
        };
        let mut timer = module(
            "timer",
            vec![("clk", Direction::In), ("tick", Direction::Out)],
        );
        timer.generics.push(generic("DIV", "1000"));

        let mut s = Schematic::new("top", Language::Vhdl);
        s.top_generics.push(generic("TOP_DIV", "500"));
        for (name, direction) in [("clk", Direction::In), ("tick", Direction::Out)] {
            s.top_ports.push(PortDef {
                name: name.into(),
                direction,
                port_type: PortType::StdLogic,
                bundle: None,
            });
        }
        s.add_instance("u_t", "timer").unwrap();
        s.set_generic_map_entry("u_t", "DIV", "2 * TOP_DIV")
            .unwrap();
        s.set_port_map_entry("u_t", "clk", Some(NetRef::TopPort("clk".into())))
            .unwrap();
        s.set_port_map_entry("u_t", "tick", Some(NetRef::TopPort("tick".into())))
            .unwrap();
        s.groups.push(Group {
            name: "g".into(),
            members: vec!["u_t".into()],
            parent: None,
            collapsed: false,
            position: (0.0, 0.0),
        });

        let design = crate::codegen::generate_design(&s, &[timer]).unwrap();
        let (_, group) = &design.files[0];
        assert!(group.contains("DIV => 2 * TOP_DIV"), "{group}");
        assert!(group.contains("TOP_DIV : integer := 500"), "{group}");
        assert!(
            design.top_code.contains("TOP_DIV => TOP_DIV"),
            "{}",
            design.top_code
        );
    }
}
