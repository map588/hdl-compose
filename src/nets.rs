//! Derived net resolution.
//!
//! A `port_map` entry is a connectivity statement, not a naming convention:
//! `B.din => A.dout` and `A.dout => B.din` describe the same wire. This module
//! merges all mutually-referencing pins into nets with union-find, then
//! resolves each net's driver, canonical signal name, and declaration type.
//! Codegen and validation both consume this — reference direction in the
//! `.hdlc` file never matters.

use std::collections::HashMap;

use crate::codegen::resolve_port_type;
use crate::types::*;

/// One resolved net: a maximal set of pins connected through port_map refs.
#[derive(Debug, Clone)]
pub struct Net {
    /// Base (non-slice) refs of every attached pin, sorted by key.
    pub members: Vec<NetRef>,
    /// Members capable of driving the net: instance `Out`/`InOut` ports and
    /// top-level `In`/`InOut` ports.
    pub drivers: Vec<NetRef>,
    /// Resolved signal name (alias > top-port intermediate > driver-derived).
    pub name: String,
    /// Declaration type, resolved against the owning instance's generics.
    /// `None` when no member's port definition could be found in the library.
    pub port_type: Option<PortType>,
    /// True when any member is a top-level port (net routes through a
    /// top-port intermediate; no separate internal signal is declared).
    pub has_top: bool,
}

impl Net {
    /// All distinct aliases attached to any member of this net.
    pub fn aliases<'a>(&self, schematic: &'a Schematic) -> Vec<&'a str> {
        let mut out: Vec<&str> = self
            .members
            .iter()
            .filter_map(|m| schematic.aliases.get(m).map(|s| s.as_str()))
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }
}

/// All nets of a schematic plus a pin → net index.
pub struct Nets {
    pub nets: Vec<Net>,
    index: HashMap<NetRef, usize>,
}

impl Nets {
    /// Net containing the given pin (slice refs are normalized to base).
    pub fn net_for(&self, r: &NetRef) -> Option<&Net> {
        self.index.get(&r.base()).map(|i| &self.nets[*i])
    }

    /// Signal name for the net containing the given pin.
    pub fn name_for(&self, r: &NetRef) -> Option<&str> {
        self.net_for(r).map(|n| n.name.as_str())
    }
}

/// Union-find with path halving.
struct Uf {
    parent: Vec<usize>,
}

impl Uf {
    fn new(n: usize) -> Self {
        Uf {
            parent: (0..n).collect(),
        }
    }

    fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]];
            x = self.parent[x];
        }
        x
    }

    fn union(&mut self, a: usize, b: usize) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra != rb {
            self.parent[rb] = ra;
        }
    }
}

/// Direction of a pin, looked up from the schematic / library.
pub fn pin_direction(
    schematic: &Schematic,
    lib_map: &HashMap<&str, &ModuleDef>,
    r: &NetRef,
) -> Option<Direction> {
    match r {
        NetRef::TopPort(name) => schematic
            .top_ports
            .iter()
            .find(|p| &p.name == name)
            .map(|p| p.direction.clone()),
        NetRef::InstancePort(inst_name, port_name) => {
            let inst = schematic.instances.iter().find(|i| &i.name == inst_name)?;
            let module = lib_map.get(inst.module_ref.as_str())?;
            module
                .ports
                .iter()
                .find(|p| &p.name == port_name)
                .map(|p| p.direction.clone())
        }
        _ => None, // base() never returns slice variants
    }
}

/// Can this pin drive a net? Instance outputs and top-level inputs push
/// values onto internal signals; InOut can go either way.
fn is_driver_capable(dir: &Direction, is_top: bool) -> bool {
    matches!(
        (is_top, dir),
        (false, Direction::Out | Direction::InOut) | (true, Direction::In | Direction::InOut)
    )
}

/// Declaration type contributed by one member pin, resolved against the
/// owning instance's generic map where possible.
fn member_type(
    schematic: &Schematic,
    lib_map: &HashMap<&str, &ModuleDef>,
    r: &NetRef,
) -> Option<PortType> {
    match r {
        NetRef::TopPort(name) => schematic
            .top_ports
            .iter()
            .find(|p| &p.name == name)
            .map(|p| p.port_type.clone()),
        NetRef::InstancePort(inst_name, port_name) => {
            let inst = schematic.instances.iter().find(|i| &i.name == inst_name)?;
            let module = lib_map.get(inst.module_ref.as_str())?;
            let port = module.ports.iter().find(|p| &p.name == port_name)?;
            Some(resolve_port_type(
                &port.port_type,
                &module.generics,
                &inst.generic_map,
            ))
        }
        _ => None,
    }
}

/// The target a driver pin pushes onto: the base ref from its own port_map
/// entry plus the slice of that target it occupies (None = the whole thing).
/// Non-instance drivers (top In ports) and self-references drive their base
/// in full.
pub fn driver_target(schematic: &Schematic, driver: &NetRef) -> (NetRef, Option<SliceExpr>) {
    if let NetRef::InstancePort(inst_name, port_name) = driver
        && let Some(inst) = schematic.instances.iter().find(|i| &i.name == inst_name)
        && let Some(Some(target)) = inst.port_map.get(port_name)
    {
        return match target {
            NetRef::TopPortSlice(n, s) => (NetRef::TopPort(n.clone()), Some(s.clone())),
            NetRef::InstancePortSlice(i, p, s) => {
                (NetRef::InstancePort(i.clone(), p.clone()), Some(s.clone()))
            }
            other => (other.base(), None),
        };
    }
    (driver.base(), None)
}

/// Do two drivers conflict? Only when they push onto the same target base
/// with overlapping bits; disjoint slices of one vector port are fine.
/// Different bases merged into one net stay conservative (conflict).
pub fn drivers_conflict(
    schematic: &Schematic,
    a: &NetRef,
    b: &NetRef,
) -> bool {
    let (base_a, slice_a) = driver_target(schematic, a);
    let (base_b, slice_b) = driver_target(schematic, b);
    if base_a != base_b {
        return true;
    }
    fn bits(s: &SliceExpr) -> (i32, i32) {
        match s {
            SliceExpr::Bit(i) => (*i, *i),
            SliceExpr::Range { high, low } => (*low.min(high), *high.max(low)),
        }
    }
    match (&slice_a, &slice_b) {
        (Some(x), Some(y)) => {
            let (al, ah) = bits(x);
            let (bl, bh) = bits(y);
            al <= bh && bl <= ah
        }
        _ => true,
    }
}

pub(crate) fn has_literal_bounds(pt: &PortType) -> bool {
    match pt {
        PortType::StdLogicVector(r) => {
            matches!(r.high, RangeExpr::Literal(_)) && matches!(r.low, RangeExpr::Literal(_))
        }
        _ => true,
    }
}

/// Resolve all nets of the schematic.
pub fn resolve_nets(schematic: &Schematic, library: &[ModuleDef]) -> Nets {
    let lib_map: HashMap<&str, &ModuleDef> =
        library.iter().map(|m| (m.name.as_str(), m)).collect();

    // Intern every pin seen in a port_map relationship and unite the two
    // sides of each entry.
    let mut ids: HashMap<NetRef, usize> = HashMap::new();
    let mut atoms: Vec<NetRef> = Vec::new();
    let intern = |r: NetRef, atoms: &mut Vec<NetRef>, ids: &mut HashMap<NetRef, usize>| {
        *ids.entry(r.clone()).or_insert_with(|| {
            atoms.push(r);
            atoms.len() - 1
        })
    };

    let mut edges: Vec<(usize, usize)> = Vec::new();
    for inst in &schematic.instances {
        for (port_name, entry) in &inst.port_map {
            let Some(net_ref) = entry else { continue };
            // Constant ties are direct literal associations, not nets.
            if matches!(net_ref, NetRef::Constant(_)) {
                continue;
            }
            let owner = NetRef::InstancePort(inst.name.clone(), port_name.clone());
            let target = net_ref.base();
            let a = intern(owner, &mut atoms, &mut ids);
            // Self-references only mark net identity; the atom alone suffices.
            let b = intern(target, &mut atoms, &mut ids);
            edges.push((a, b));
        }
    }

    let mut uf = Uf::new(atoms.len());
    for (a, b) in edges {
        uf.union(a, b);
    }

    let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..atoms.len() {
        let root = uf.find(i);
        groups.entry(root).or_default().push(i);
    }

    let mut nets: Vec<Net> = Vec::new();
    for member_ids in groups.into_values() {
        let mut members: Vec<NetRef> = member_ids.iter().map(|&i| atoms[i].clone()).collect();
        members.sort_by_key(|r| r.to_key());

        let drivers: Vec<NetRef> = members
            .iter()
            .filter(|r| {
                let is_top = matches!(r, NetRef::TopPort(_));
                pin_direction(schematic, &lib_map, r)
                    .is_some_and(|d| is_driver_capable(&d, is_top))
            })
            .cloned()
            .collect();

        let has_top = members.iter().any(|r| matches!(r, NetRef::TopPort(_)));

        // Canonical pin for naming: aliased member (drivers preferred) >
        // top-port member > driver > first member. `signal_name` then applies
        // alias / top-intermediate / driver-derived naming for that pin.
        let canonical = drivers
            .iter()
            .chain(members.iter())
            .find(|r| schematic.aliases.contains_key(r))
            .or_else(|| members.iter().find(|r| matches!(r, NetRef::TopPort(_))))
            .or_else(|| drivers.first())
            .or_else(|| members.first())
            .cloned()
            .expect("net groups are never empty");
        let name = schematic.signal_name(&canonical);

        // Declaration type: prefer the driver's resolved type, then any
        // member that resolves to literal bounds, then anything available.
        let mut candidates = drivers
            .iter()
            .chain(members.iter())
            .filter_map(|r| member_type(schematic, &lib_map, r));
        let first = candidates.next();
        let port_type = match &first {
            Some(pt) if !has_literal_bounds(pt) => {
                candidates.find(has_literal_bounds).or(first)
            }
            _ => first,
        };

        nets.push(Net {
            members,
            drivers,
            name,
            port_type,
            has_top,
        });
    }

    // Deterministic order for codegen output and diagnostics.
    nets.sort_by(|a, b| a.name.cmp(&b.name));

    let mut index = HashMap::new();
    for (i, net) in nets.iter().enumerate() {
        for m in &net.members {
            index.insert(m.clone(), i);
        }
    }

    Nets { nets, index }
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

    fn ab_library() -> Vec<ModuleDef> {
        vec![
            module("mod_a", vec![("dout", Direction::Out)]),
            module("mod_b", vec![("din", Direction::In)]),
        ]
    }

    fn ab_schematic() -> Schematic {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.add_instance("u_a", "mod_a").unwrap();
        s.add_instance("u_b", "mod_b").unwrap();
        s
    }

    #[test]
    fn forward_reference_resolves_single_net() {
        let mut s = ab_schematic();
        s.set_port_map_entry(
            "u_b",
            "din",
            Some(NetRef::InstancePort("u_a".into(), "dout".into())),
        )
        .unwrap();
        let nets = resolve_nets(&s, &ab_library());
        assert_eq!(nets.nets.len(), 1);
        let net = &nets.nets[0];
        assert_eq!(net.members.len(), 2);
        assert_eq!(net.drivers, vec![NetRef::InstancePort("u_a".into(), "dout".into())]);
        assert_eq!(net.name, "u_a_dout");
    }

    #[test]
    fn mutual_references_merge_into_one_net() {
        // The classic footgun: A.dout => B.din AND B.din => A.dout.
        let mut s = ab_schematic();
        s.set_port_map_entry(
            "u_a",
            "dout",
            Some(NetRef::InstancePort("u_b".into(), "din".into())),
        )
        .unwrap();
        s.set_port_map_entry(
            "u_b",
            "din",
            Some(NetRef::InstancePort("u_a".into(), "dout".into())),
        )
        .unwrap();
        let nets = resolve_nets(&s, &ab_library());
        assert_eq!(nets.nets.len(), 1, "mutual refs must not split the net");
        let net = &nets.nets[0];
        assert_eq!(net.drivers.len(), 1);
        assert_eq!(net.name, "u_a_dout", "named from the driver, not the load");
    }

    #[test]
    fn backward_reference_names_from_driver() {
        // Output references the load ("wrong" direction) — still one net,
        // still named from the driving pin.
        let mut s = ab_schematic();
        s.set_port_map_entry(
            "u_a",
            "dout",
            Some(NetRef::InstancePort("u_b".into(), "din".into())),
        )
        .unwrap();
        let nets = resolve_nets(&s, &ab_library());
        assert_eq!(nets.nets.len(), 1);
        assert_eq!(nets.nets[0].name, "u_a_dout");
        assert_eq!(
            nets.name_for(&NetRef::InstancePort("u_b".into(), "din".into())),
            Some("u_a_dout")
        );
    }

    #[test]
    fn slice_refs_unite_base_nets() {
        let mut lib = ab_library();
        lib[0].ports[0].port_type = PortType::StdLogicVector(Range {
            high: RangeExpr::Literal(7),
            low: RangeExpr::Literal(0),
            dir: RangeDir::Downto,
        });
        let mut s = ab_schematic();
        s.set_port_map_entry(
            "u_b",
            "din",
            Some(NetRef::InstancePortSlice(
                "u_a".into(),
                "dout".into(),
                SliceExpr::Bit(3),
            )),
        )
        .unwrap();
        let nets = resolve_nets(&s, &lib);
        assert_eq!(nets.nets.len(), 1);
        assert_eq!(nets.nets[0].name, "u_a_dout");
        assert!(matches!(
            nets.nets[0].port_type,
            Some(PortType::StdLogicVector(_))
        ));
    }

    #[test]
    fn undriven_multi_load_net_has_no_driver() {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.add_instance("u_b1", "mod_b").unwrap();
        s.add_instance("u_b2", "mod_b").unwrap();
        // Multi-load idiom: both inputs share a self-rooted net.
        s.set_port_map_entry(
            "u_b1",
            "din",
            Some(NetRef::InstancePort("u_b1".into(), "din".into())),
        )
        .unwrap();
        s.set_port_map_entry(
            "u_b2",
            "din",
            Some(NetRef::InstancePort("u_b1".into(), "din".into())),
        )
        .unwrap();
        let nets = resolve_nets(&s, &ab_library());
        assert_eq!(nets.nets.len(), 1);
        assert!(nets.nets[0].drivers.is_empty());
        assert_eq!(nets.nets[0].members.len(), 2);
    }

    #[test]
    fn alias_wins_regardless_of_which_member_carries_it() {
        let mut s = ab_schematic();
        s.set_port_map_entry(
            "u_b",
            "din",
            Some(NetRef::InstancePort("u_a".into(), "dout".into())),
        )
        .unwrap();
        // Alias on the LOAD pin, not the driver — merged net still picks it up.
        s.set_alias(NetRef::InstancePort("u_b".into(), "din".into()), "data_bus");
        let nets = resolve_nets(&s, &ab_library());
        assert_eq!(nets.nets[0].name, "data_bus");
    }

    #[test]
    fn top_port_net_uses_intermediate_name() {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.top_ports.push(PortDef {
            name: "led".into(),
            direction: Direction::Out,
            port_type: PortType::StdLogic,
            bundle: None,
        });
        s.add_instance("u_a", "mod_a").unwrap();
        s.set_port_map_entry("u_a", "dout", Some(NetRef::TopPort("led".into())))
            .unwrap();
        let nets = resolve_nets(&s, &ab_library());
        assert_eq!(nets.nets.len(), 1);
        let net = &nets.nets[0];
        assert!(net.has_top);
        assert_eq!(net.name, "led_s");
        // Instance output drives; top Out port is a load.
        assert_eq!(net.drivers, vec![NetRef::InstancePort("u_a".into(), "dout".into())]);
    }

    #[test]
    fn chained_references_merge_transitively() {
        // b1.din => a.dout, b2.din => b1.din — all one net.
        let mut s = ab_schematic();
        s.add_instance("u_b2", "mod_b").unwrap();
        s.set_port_map_entry(
            "u_b",
            "din",
            Some(NetRef::InstancePort("u_a".into(), "dout".into())),
        )
        .unwrap();
        s.set_port_map_entry(
            "u_b2",
            "din",
            Some(NetRef::InstancePort("u_b".into(), "din".into())),
        )
        .unwrap();
        let nets = resolve_nets(&s, &ab_library());
        assert_eq!(nets.nets.len(), 1);
        assert_eq!(nets.nets[0].members.len(), 3);
        assert_eq!(nets.nets[0].name, "u_a_dout");
    }
}
