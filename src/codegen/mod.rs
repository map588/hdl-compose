pub mod sv;
pub mod vhdl;

use crate::schematic::{Diagnostic, DiagnosticLevel};
use crate::types::*;

#[derive(Debug, thiserror::Error)]
pub enum CodegenError {
    #[error("schematic has validation errors; fix them before generating code")]
    ValidationErrors(Vec<Diagnostic>),

    #[error("dirty instances present (source re-parse dropped connections): {0:?}")]
    DirtyInstances(Vec<String>),
}

/// Refuse codegen when any instance carries an unresolved dirty flag from a
/// prior library re-parse. Forces the user to review and acknowledge the
/// dropped connections before they land in the generated HDL.
pub fn check_no_dirty_instances(schematic: &Schematic) -> Result<(), CodegenError> {
    let dirty: Vec<String> = schematic
        .instances
        .iter()
        .filter(|i| i.dirty)
        .map(|i| i.name.clone())
        .collect();
    if dirty.is_empty() {
        Ok(())
    } else {
        Err(CodegenError::DirtyInstances(dirty))
    }
}

/// Render a PortType as a VHDL type string.
pub fn port_type_to_vhdl(pt: &PortType) -> String {
    match pt {
        PortType::StdLogic => "std_logic".to_string(),
        PortType::StdLogicVector(range) => {
            let high = range_expr_to_string(&range.high);
            let low = range_expr_to_string(&range.low);
            let dir = match range.dir {
                RangeDir::Downto => "downto",
                RangeDir::To => "to",
            };
            format!("std_logic_vector({high} {dir} {low})")
        }
        PortType::Record(name) => name.clone(),
        PortType::Other(s) => s.clone(),
    }
}

/// Render a PortType as a SystemVerilog type string.
pub fn port_type_to_sv(pt: &PortType) -> String {
    match pt {
        PortType::StdLogic => "logic".to_string(),
        PortType::StdLogicVector(range) => {
            let high = range_expr_to_string(&range.high);
            let low = range_expr_to_string(&range.low);
            format!("logic [{high}:{low}]")
        }
        PortType::Record(name) => name.clone(),
        PortType::Other(s) => s.clone(),
    }
}

/// Render a PortType as a VHDL signal declaration type (same as port type for VHDL).
pub fn port_type_to_vhdl_signal(pt: &PortType) -> String {
    port_type_to_vhdl(pt)
}

/// Render a PortType as a SystemVerilog wire declaration type.
pub fn port_type_to_sv_wire(pt: &PortType) -> String {
    match pt {
        PortType::StdLogic => "wire".to_string(),
        PortType::StdLogicVector(range) => {
            let high = range_expr_to_string(&range.high);
            let low = range_expr_to_string(&range.low);
            format!("wire [{high}:{low}]")
        }
        PortType::Record(name) => name.clone(),
        PortType::Other(s) => s.clone(),
    }
}

fn range_expr_to_string(expr: &RangeExpr) -> String {
    match expr {
        RangeExpr::Literal(n) => n.to_string(),
        RangeExpr::Expr(s) => s.clone(),
    }
}

/// Evaluate a `RangeExpr` against a substitution map of generic names → integer
/// values. Supports bare integers, lone identifiers, and simple `NAME ± const`
/// patterns — enough for the common `WIDTH-1` / `N+1` bounds in HDL modules.
pub fn eval_range_expr(
    expr: &RangeExpr,
    generics: &std::collections::HashMap<&str, i64>,
) -> Option<i64> {
    match expr {
        RangeExpr::Literal(v) => Some(*v),
        RangeExpr::Expr(raw) => {
            let s = raw.trim();
            if let Ok(v) = s.parse::<i64>() {
                return Some(v);
            }
            if let Some(v) = generics.get(s).copied() {
                return Some(v);
            }
            for op in ['-', '+'] {
                if let Some(pos) = s.rfind(op) {
                    let lhs = s[..pos].trim();
                    let rhs = s[pos + 1..].trim();
                    let lv = generics
                        .get(lhs)
                        .copied()
                        .or_else(|| lhs.parse::<i64>().ok())?;
                    let rv = rhs.parse::<i64>().ok()?;
                    return Some(if op == '-' { lv - rv } else { lv + rv });
                }
            }
            None
        }
    }
}

/// Build a generic-substitution map from a module's defaults plus per-instance
/// overrides. Instance overrides win.
pub fn build_generic_substitutions<'a>(
    module_generics: &'a [GenericDef],
    instance_generic_map: &'a std::collections::HashMap<String, String>,
) -> std::collections::HashMap<&'a str, i64> {
    let mut subs = std::collections::HashMap::new();
    for g in module_generics {
        if let Some(default) = &g.default_value
            && let Ok(v) = default.trim().parse::<i64>()
        {
            subs.insert(g.name.as_str(), v);
        }
    }
    for (k, v) in instance_generic_map {
        if let Ok(n) = v.trim().parse::<i64>() {
            subs.insert(k.as_str(), n);
        }
    }
    subs
}

/// Return a copy of `port_type` where any non-literal `StdLogicVector` range
/// bounds are evaluated against the supplied generic substitutions and replaced
/// with `RangeExpr::Literal`. Bounds that cannot be resolved are left as-is so
/// codegen still emits something compilable (the validator surfaces unresolved
/// references separately).
pub fn resolve_port_type(
    port_type: &PortType,
    module_generics: &[GenericDef],
    instance_generic_map: &std::collections::HashMap<String, String>,
) -> PortType {
    let PortType::StdLogicVector(range) = port_type else {
        return port_type.clone();
    };
    if matches!(range.high, RangeExpr::Literal(_)) && matches!(range.low, RangeExpr::Literal(_)) {
        return port_type.clone();
    }
    let subs = build_generic_substitutions(module_generics, instance_generic_map);
    let high = eval_range_expr(&range.high, &subs)
        .map(RangeExpr::Literal)
        .unwrap_or_else(|| range.high.clone());
    let low = eval_range_expr(&range.low, &subs)
        .map(RangeExpr::Literal)
        .unwrap_or_else(|| range.low.clone());
    PortType::StdLogicVector(Range {
        high,
        low,
        dir: range.dir.clone(),
    })
}

/// Per-top-port intermediate signal that codegen routes loads/drivers through
/// instead of the bare entity port. Created for every connected non-inout top
/// port to give all internal references a single canonical name.
pub struct TopIntermediate {
    pub port_name: String,
    pub sig_name: String,
    pub port_type: PortType,
    pub direction: Direction,
}

/// Collect the per-top-port intermediates that need a `signal` declaration +
/// pass-through assignment. Skips ports with no connection (no need for an
/// unused signal) and `InOut` ports (would create multi-driver conflict).
/// Skips ports whose intermediate name collides with the port name (e.g. an
/// alias set to the same string) so codegen doesn't emit `<x> <= <x>;`.
pub fn collect_top_intermediates(
    schematic: &Schematic,
    library: &[ModuleDef],
) -> Vec<TopIntermediate> {
    use std::collections::HashSet;
    let mut referenced: HashSet<String> = HashSet::new();
    for inst in &schematic.instances {
        for entry in inst.port_map.values() {
            if let Some(net) = entry
                && let NetRef::TopPort(name) = net.base()
            {
                referenced.insert(name);
            }
        }
    }
    let mut out = Vec::new();
    for port in &schematic.top_ports {
        if !referenced.contains(&port.name) {
            continue;
        }
        if matches!(port.direction, Direction::InOut) {
            continue;
        }
        let sig_name = schematic.signal_name(&NetRef::TopPort(port.name.clone()));
        if sig_name == port.name {
            continue;
        }
        let port_type = match find_top_port_context(schematic, library, &port.name) {
            Some((mg, gm)) => resolve_port_type(&port.port_type, mg, gm),
            None => port.port_type.clone(),
        };
        out.push(TopIntermediate {
            port_name: port.name.clone(),
            sig_name,
            port_type,
            direction: port.direction.clone(),
        });
    }
    out
}

/// Find a context (module + instance generic_map) that can resolve the type of
/// the given top-level port. Scans instances looking for any port_map entry
/// whose net base is `TopPort(top_port_name)`. Returns the first match.
pub fn find_top_port_context<'a>(
    schematic: &'a Schematic,
    library: &'a [ModuleDef],
    top_port_name: &str,
) -> Option<(&'a [GenericDef], &'a std::collections::HashMap<String, String>)> {
    use std::collections::HashMap;
    let lib_map: HashMap<&str, &ModuleDef> = library.iter().map(|m| (m.name.as_str(), m)).collect();
    for inst in &schematic.instances {
        for net_opt in inst.port_map.values() {
            let Some(net) = net_opt else { continue };
            if let NetRef::TopPort(name) = net.base()
                && name == top_port_name
                && let Some(module) = lib_map.get(inst.module_ref.as_str())
            {
                return Some((module.generics.as_slice(), &inst.generic_map));
            }
        }
    }
    None
}


/// Collect all internal nets (driven by instance outputs that are referenced by at least one port).
/// Returns (net_ref, signal_name, port_type) tuples.
pub fn collect_internal_nets(
    schematic: &Schematic,
    library: &[ModuleDef],
) -> Vec<(NetRef, String, PortType)> {
    use std::collections::{HashMap, HashSet};

    let lib_map: HashMap<&str, &ModuleDef> = library.iter().map(|m| (m.name.as_str(), m)).collect();

    // Find all instance-output nets referenced in any port map (normalize slices to base drivers).
    let mut referenced_nets: HashSet<NetRef> = HashSet::new();
    for inst in &schematic.instances {
        for net_ref_opt in inst.port_map.values() {
            if let Some(net_ref) = net_ref_opt {
                let base = net_ref.base();
                if matches!(base, NetRef::InstancePort(_, _)) {
                    referenced_nets.insert(base);
                }
            }
        }
    }

    // For each referenced net, find the driver's port type and resolve any
    // generic-derived range bounds against the driving instance's generic_map.
    let mut nets = Vec::new();
    for net_ref in &referenced_nets {
        if let NetRef::InstancePort(inst_name, port_name) = net_ref
            && let Some(driver_inst) = schematic.instances.iter().find(|i| i.name == *inst_name)
            && let Some(module) = lib_map.get(driver_inst.module_ref.as_str())
            && let Some(port) = module.ports.iter().find(|p| p.name == *port_name)
        {
            let sig_name = schematic.signal_name(net_ref);
            let resolved =
                resolve_port_type(&port.port_type, &module.generics, &driver_inst.generic_map);
            nets.push((net_ref.clone(), sig_name, resolved));
        }
    }

    // Sort by signal name for deterministic output
    nets.sort_by(|a, b| a.1.cmp(&b.1));
    nets
}

/// Pre-validate and return errors if any exist.
pub fn check_errors(diagnostics: &[Diagnostic]) -> Result<(), CodegenError> {
    let errors: Vec<Diagnostic> = diagnostics
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .cloned()
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(CodegenError::ValidationErrors(errors))
    }
}
