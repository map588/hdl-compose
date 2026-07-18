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
            diags.push(Diagnostic::error(format!("duplicate group name '{}'", g.name)));
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
            diags.push(Diagnostic::error(format!("group '{}' has no members", g.name)));
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

struct BoundaryPort {
    name: String,
    direction: Direction,
    port_type: PortType,
    /// One representative pin outside the group on this net (driver
    /// preferred) — what the parent-level group instance connects to.
    outside_ref: Option<NetRef>,
}

/// Collapse every group. Returns the per-group schematics (children first),
/// the rewritten top, and the library extended with synthesized group
/// modules. Call only after `validate_groups` reports no errors.
pub fn expand_hierarchy(schematic: &Schematic, library: &[ModuleDef]) -> HierarchyPlan {
    let mut cur = schematic.clone();
    let mut lib: Vec<ModuleDef> = library.to_vec();
    let mut out_groups: Vec<(String, Schematic)> = Vec::new();

    // Children first: deeper parent chains collapse earlier.
    let by_name: HashMap<&str, &Group> =
        schematic.groups.iter().map(|g| (g.name.as_str(), g)).collect();
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

        let lib_map: HashMap<&str, &ModuleDef> = lib.iter().map(|m| (m.name.as_str(), m)).collect();
        let nets = resolve_nets(&cur, &lib);

        // Derive boundary ports: nets with pins on both sides of the fence.
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
            let any_inout = net.members.iter().any(|r| {
                pin_direction(&cur, &lib_map, r) == Some(Direction::InOut)
            });
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
                port_for_pin.insert((*pin).clone(), pname.clone());
            }
            ports.push(BoundaryPort {
                name: pname,
                direction,
                port_type,
                outside_ref,
            });
        }
        ports.sort_by(|a, b| a.name.cmp(&b.name));

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
        // instance passes them straight through by name.
        let passthrough: Vec<GenericDef> = cur
            .top_generics
            .iter()
            .filter(|tg| {
                gs.instances
                    .iter()
                    .any(|i| i.generic_map.values().any(|v| v.trim() == tg.name))
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

    HierarchyPlan { groups: out_groups, top: cur, library: lib }
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
            module("mod_a", vec![("clk", Direction::In), ("dout", Direction::Out)]),
            module("mod_b", vec![("din", Direction::In), ("dout", Direction::Out)]),
        ];
        (s, lib)
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
        assert!(msgs.iter().any(|m| m.contains("not an instance")), "{msgs:?}");
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
        assert!(top.instances.iter().all(|i| i.name != "u_a" && i.name != "u_b"));
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
}
