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

/// Everything codegen produces for a project: per-group module files
/// (children before parents, matching required analysis order) plus the
/// top-level module. Filenames carry the language extension.
pub struct GeneratedDesign {
    pub files: Vec<(String, String)>,
    pub top_filename: String,
    pub top_code: String,
}

impl GeneratedDesign {
    /// Write the design to disk: the top-level module to `top_path`, and
    /// each group module next to it under its own filename. Returns the
    /// written paths, group files first.
    ///
    /// The CLI and the GUI both write through this function, so the two
    /// always produce the same set of files.
    pub fn write_to(&self, top_path: &std::path::Path) -> std::io::Result<Vec<std::path::PathBuf>> {
        let dir = top_path.parent().unwrap_or(std::path::Path::new(""));
        let mut written = Vec::with_capacity(self.files.len() + 1);
        for (fname, code) in &self.files {
            let p = dir.join(fname);
            std::fs::write(&p, code)?;
            written.push(p);
        }
        std::fs::write(top_path, &self.top_code)?;
        written.push(top_path.to_path_buf());
        Ok(written)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DesignError {
    #[error("group validation failed")]
    GroupDiagnostics(Vec<crate::schematic::Diagnostic>),

    #[error("group '{name}': {source}")]
    Group { name: String, source: CodegenError },

    #[error(transparent)]
    Top(#[from] CodegenError),
}

/// Generate every output file for a project — flat or grouped — without
/// touching the filesystem. Single entry point shared by `codegen`, `check`,
/// `synth` and `sim`.
pub fn generate_design(
    schematic: &Schematic,
    library: &[ModuleDef],
) -> Result<GeneratedDesign, DesignError> {
    let ext = match schematic.language {
        Language::Vhdl => "vhd",
        Language::SystemVerilog => "sv",
    };
    let top_filename = format!("{}.{ext}", schematic.top_name);

    if schematic.groups.is_empty() {
        let diags = schematic.validate(library);
        let top_code = match schematic.language {
            Language::Vhdl => vhdl::generate_vhdl(schematic, library, &diags),
            Language::SystemVerilog => sv::generate_sv(schematic, library, &diags),
        }?;
        return Ok(GeneratedDesign {
            files: Vec::new(),
            top_filename,
            top_code,
        });
    }

    let group_diags = crate::groups::validate_groups(schematic, library);
    if group_diags
        .iter()
        .any(|d| d.level == DiagnosticLevel::Error)
    {
        return Err(DesignError::GroupDiagnostics(group_diags));
    }

    let plan = crate::groups::expand_hierarchy(schematic, library);
    let generate = |s: &Schematic| -> Result<String, CodegenError> {
        let diags = s.validate(&plan.library);
        match s.language {
            Language::Vhdl => vhdl::generate_vhdl(s, &plan.library, &diags),
            Language::SystemVerilog => sv::generate_sv(s, &plan.library, &diags),
        }
    };

    let mut files = Vec::new();
    for (name, gs) in &plan.groups {
        let code = generate(gs).map_err(|source| DesignError::Group {
            name: name.clone(),
            source,
        })?;
        files.push((format!("{name}.{ext}"), code));
    }
    let top_code = generate(&plan.top)?;
    Ok(GeneratedDesign {
        files,
        top_filename,
        top_code,
    })
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
    nets: &crate::nets::Nets,
) -> Vec<TopIntermediate> {
    let mut out = Vec::new();
    for port in &schematic.top_ports {
        let pin = NetRef::TopPort(port.name.clone());
        let Some(net) = nets.net_for(&pin) else {
            continue; // unconnected top port
        };
        if matches!(port.direction, Direction::InOut) {
            continue;
        }
        if net.name == port.name {
            continue;
        }
        // The intermediate stands in for the PORT, so it takes the port's
        // own type whenever that type is fully literal. The net's resolved
        // type (driver-side generics applied) is only for ports whose bounds
        // reference generics — a sliced or narrower driver must not shrink
        // or widen the port-adjacent signal.
        let port_type = if crate::nets::has_literal_bounds(&port.port_type) {
            port.port_type.clone()
        } else {
            net.port_type
                .clone()
                .unwrap_or_else(|| port.port_type.clone())
        };
        out.push(TopIntermediate {
            port_name: port.name.clone(),
            sig_name: net.name.clone(),
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
) -> Option<(
    &'a [GenericDef],
    &'a std::collections::HashMap<String, String>,
)> {
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

/// Collect the internal signal declarations: one per resolved net that does
/// not route through a top-port intermediate. Returns (signal_name, port_type)
/// pairs sorted by name.
pub fn collect_internal_nets(nets: &crate::nets::Nets) -> Vec<(String, PortType)> {
    nets.nets
        .iter()
        .filter(|n| !n.has_top)
        .filter_map(|n| n.port_type.clone().map(|pt| (n.name.clone(), pt)))
        .collect()
}

/// Render a generic-map value. String-typed generics get quotes added when
/// the user wrote a bare word (`OR` → `"OR"`) — hand-escaping is not needed.
/// Values that are already quoted, or that name a top-level generic (a
/// passthrough, not a literal), are emitted verbatim.
pub fn format_generic_value(
    value: &str,
    generic_def: Option<&GenericDef>,
    top_generics: &[GenericDef],
) -> String {
    let v = value.trim();
    let is_string_type =
        generic_def.is_some_and(|g| g.type_name.trim().eq_ignore_ascii_case("string"));
    if is_string_type && !v.starts_with('"') && !top_generics.iter().any(|g| g.name == v) {
        format!("\"{v}\"")
    } else {
        value.to_string()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn vector(high: i64) -> PortType {
        PortType::StdLogicVector(Range {
            high: RangeExpr::Literal(high),
            low: RangeExpr::Literal(0),
            dir: RangeDir::Downto,
        })
    }

    fn port(name: &str, direction: Direction, port_type: PortType) -> PortDef {
        PortDef {
            name: name.into(),
            direction,
            port_type,
            bundle: None,
        }
    }

    fn module(name: &str, ports: Vec<PortDef>) -> ModuleDef {
        ModuleDef {
            name: name.into(),
            generics: Vec::new(),
            ports,
            source_path: format!("{name}.vhd").into(),
            source_hash: 0,
            dependencies: Vec::new(),
        }
    }

    /// The project from the defect report. Two 8-bit outputs drive the two
    /// halves of the 16-bit top-level port `wide`, and `u_led.value` (8 bits)
    /// references the output `u_hi.count`, which is on `wide[15:8]`.
    fn slice_reference_project(language: Language) -> (Schematic, Vec<ModuleDef>) {
        let mut s = Schematic::new("slices", language);
        s.top_ports.push(port("wide", Direction::Out, vector(15)));
        s.top_ports
            .push(port("led", Direction::Out, PortType::StdLogic));
        for (name, high, low) in [("u_lo", 7, 0), ("u_hi", 15, 8)] {
            s.add_instance(name, "byte_source").unwrap();
            s.set_port_map_entry(
                name,
                "count",
                Some(NetRef::TopPortSlice(
                    "wide".into(),
                    SliceExpr::Range { high, low },
                )),
            )
            .unwrap();
        }
        s.add_instance("u_led", "led_driver").unwrap();
        s.set_port_map_entry(
            "u_led",
            "value",
            Some(NetRef::InstancePort("u_hi".into(), "count".into())),
        )
        .unwrap();
        s.set_port_map_entry("u_led", "led", Some(NetRef::TopPort("led".into())))
            .unwrap();
        let lib = vec![
            module(
                "byte_source",
                vec![port("count", Direction::Out, vector(7))],
            ),
            module(
                "led_driver",
                vec![
                    port("value", Direction::In, vector(7)),
                    port("led", Direction::Out, PortType::StdLogic),
                ],
            ),
        ];
        (s, lib)
    }

    /// Regression: the reference connected `value` to all 16 bits of `wide_s`.
    #[test]
    fn reference_to_a_pin_on_a_slice_connects_to_that_slice() {
        let (s, lib) = slice_reference_project(Language::Vhdl);
        let diags = s.validate(&lib);
        assert!(diags.is_empty(), "{diags:?}");
        let vhdl = generate_design(&s, &lib).unwrap().top_code;
        assert!(vhdl.contains("value => wide_s(15 downto 8)"), "{vhdl}");
        assert!(vhdl.contains("count => wide_s(15 downto 8)"), "{vhdl}");
        assert!(vhdl.contains("count => wide_s(7 downto 0)"), "{vhdl}");

        let (s, lib) = slice_reference_project(Language::SystemVerilog);
        let sv = generate_design(&s, &lib).unwrap().top_code;
        assert!(sv.contains(".value(wide_s[15:8])"), "{sv}");
    }

    /// Regression: two 4-bit outputs drive the two halves of the 8-bit input
    /// `u_led.value`. The signal took the width of one of the outputs.
    #[test]
    fn net_that_outputs_drive_in_slices_has_the_width_of_the_full_pin() {
        let mut s = Schematic::new("inner", Language::Vhdl);
        s.top_ports
            .push(port("led", Direction::Out, PortType::StdLogic));
        for (name, high, low) in [("u_lo", 3, 0), ("u_hi", 7, 4)] {
            s.add_instance(name, "nibble_source").unwrap();
            s.set_port_map_entry(
                name,
                "count",
                Some(NetRef::InstancePortSlice(
                    "u_led".into(),
                    "value".into(),
                    SliceExpr::Range { high, low },
                )),
            )
            .unwrap();
        }
        s.add_instance("u_led", "led_driver").unwrap();
        s.set_port_map_entry("u_led", "led", Some(NetRef::TopPort("led".into())))
            .unwrap();
        let lib = vec![
            module(
                "nibble_source",
                vec![port("count", Direction::Out, vector(3))],
            ),
            module(
                "led_driver",
                vec![
                    port("value", Direction::In, vector(7)),
                    port("led", Direction::Out, PortType::StdLogic),
                ],
            ),
        ];
        let diags = s.validate(&lib);
        assert!(diags.is_empty(), "{diags:?}");
        let vhdl = generate_design(&s, &lib).unwrap().top_code;
        let signals: Vec<&str> = vhdl
            .lines()
            .filter(|l| l.trim_start().starts_with("signal ") && l.contains("count"))
            .collect();
        assert_eq!(signals.len(), 1, "{vhdl}");
        assert!(
            signals[0].ends_with(": std_logic_vector(7 downto 0);"),
            "{vhdl}"
        );
    }
}
