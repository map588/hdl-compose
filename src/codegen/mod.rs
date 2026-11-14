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

/// Evaluate a bound that holds only integers: a bare integer, or the simple
/// `A - B` / `A + B` pattern. This is sufficient for the common `WIDTH-1` /
/// `N+1` bounds after `substitute_generics`.
fn eval_int_expr(expr: &str) -> Option<i64> {
    let s = expr.trim();
    if let Ok(v) = s.parse::<i64>() {
        return Some(v);
    }
    for op in ['-', '+'] {
        if let Some(pos) = s.rfind(op) {
            let lv = s[..pos].trim().parse::<i64>().ok()?;
            let rv = s[pos + 1..].trim().parse::<i64>().ok()?;
            return Some(if op == '-' { lv - rv } else { lv + rv });
        }
    }
    None
}

fn is_identifier_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Replace each generic name in a bound with the value that the instance
/// gives the generic: the generic_map value, else the default value.
///
/// A value that is not an integer (the name of a top-level generic, an
/// expression) goes into the bound as text. The default value must not
/// replace it: the default is not the width of this instance.
fn substitute_generics(
    expr: &str,
    module_generics: &[GenericDef],
    instance_generic_map: &std::collections::HashMap<String, String>,
) -> String {
    let value_of = |name: &str| {
        let generic = module_generics.iter().find(|g| g.name == name)?;
        instance_generic_map
            .get(name)
            .or(generic.default_value.as_ref())
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
    };
    let mut out = String::new();
    let mut rest = expr;
    while let Some(first) = rest.chars().next() {
        if !is_identifier_char(first) {
            out.push(first);
            rest = &rest[first.len_utf8()..];
            continue;
        }
        let len = rest.find(|c| !is_identifier_char(c)).unwrap_or(rest.len());
        let (word, tail) = rest.split_at(len);
        match value_of(word) {
            Some(v) if v.parse::<i64>().is_ok() || v.chars().all(is_identifier_char) => {
                out.push_str(v);
            }
            Some(v) => {
                out.push('(');
                out.push_str(v);
                out.push(')');
            }
            None => out.push_str(word),
        }
        rest = tail;
    }
    out
}

/// True when a bound of `port_type` uses the identifier `name`.
pub(crate) fn port_type_mentions(port_type: &PortType, name: &str) -> bool {
    let PortType::StdLogicVector(range) = port_type else {
        return false;
    };
    [&range.high, &range.low]
        .into_iter()
        .any(|bound| match bound {
            RangeExpr::Literal(_) => false,
            RangeExpr::Expr(text) => text
                .split(|c| !is_identifier_char(c))
                .any(|word| word == name),
        })
}

/// Return a copy of `port_type` with the generics of the module resolved for
/// one instance. A bound becomes a literal when its generics have integer
/// values. Else it keeps the text, with the names of the module generics
/// replaced by the values from the instance, so that the bound is valid in
/// the scope of the generated module. Used so internal signal declarations
/// have the width of the port they connect to.
pub fn resolve_port_type(
    port_type: &PortType,
    module_generics: &[GenericDef],
    instance_generic_map: &std::collections::HashMap<String, String>,
) -> PortType {
    let PortType::StdLogicVector(range) = port_type else {
        return port_type.clone();
    };
    let resolve = |bound: &RangeExpr| match bound {
        RangeExpr::Literal(_) => bound.clone(),
        RangeExpr::Expr(text) => {
            let text = substitute_generics(text, module_generics, instance_generic_map);
            match eval_int_expr(&text) {
                Some(v) => RangeExpr::Literal(v),
                None => RangeExpr::Expr(text),
            }
        }
    };
    PortType::StdLogicVector(Range {
        high: resolve(&range.high),
        low: resolve(&range.low),
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
        // Regression: the signal had the name of one of the two outputs
        // (`u_hi_count`). The pin on the full net gives the name.
        assert!(
            vhdl.contains("signal u_led_value : std_logic_vector(7 downto 0);"),
            "{vhdl}"
        );
        assert!(vhdl.contains("count => u_led_value(7 downto 4)"), "{vhdl}");
        assert!(vhdl.contains("count => u_led_value(3 downto 0)"), "{vhdl}");
        assert!(vhdl.contains("value => u_led_value,"), "{vhdl}");
        assert!(!vhdl.contains("u_hi_count"), "{vhdl}");
    }

    // --- Widths that depend on generics ---

    fn generic(name: &str, default_value: &str) -> GenericDef {
        GenericDef {
            name: name.into(),
            type_name: "integer".into(),
            default_value: Some(default_value.into()),
        }
    }

    /// `std_logic_vector(<generic> - 1 downto 0)`, as the parser stores it.
    fn vector_of(generic_name: &str) -> PortType {
        PortType::StdLogicVector(Range {
            high: RangeExpr::Expr(format!("{generic_name} - 1")),
            low: RangeExpr::Literal(0),
            dir: RangeDir::Downto,
        })
    }

    fn resolved_high(value: Option<&str>) -> RangeExpr {
        let generic_map = value
            .map(|v| ("WIDTH".to_string(), v.to_string()))
            .into_iter()
            .collect();
        match resolve_port_type(&vector_of("WIDTH"), &[generic("WIDTH", "8")], &generic_map) {
            PortType::StdLogicVector(range) => range.high,
            other => panic!("not a vector: {other:?}"),
        }
    }

    proptest::proptest! {
        /// An integer value of the generic gives a literal bound. The value
        /// in the generic_map has priority over the default value.
        #[test]
        fn integer_generic_value_gives_a_literal_bound(width in 1i64..100_000) {
            proptest::prop_assert_eq!(
                resolved_high(Some(&width.to_string())),
                RangeExpr::Literal(width - 1)
            );
        }
    }

    #[test]
    fn default_generic_value_gives_a_literal_bound() {
        assert_eq!(resolved_high(None), RangeExpr::Literal(7));
    }

    /// Regression: a generic_map value that is a name made the bound use the
    /// default value of the generic (7), which is not the width of the port.
    #[test]
    fn generic_value_that_is_a_name_stays_in_the_bound() {
        assert_eq!(
            resolved_high(Some("TOP_W")),
            RangeExpr::Expr("TOP_W - 1".into())
        );
        assert_eq!(
            resolved_high(Some("2 * TOP_W")),
            RangeExpr::Expr("(2 * TOP_W) - 1".into())
        );
    }

    /// Regression: `u_gen` gets its width from the top-level generic `TOP_W`.
    /// The signal for `u_gen.count` had the default width of the module.
    fn top_generic_width_project(language: Language) -> (Schematic, Vec<ModuleDef>) {
        let mut s = Schematic::new("passthrough", language);
        s.top_generics.push(generic("TOP_W", "16"));
        s.top_ports
            .push(port("led", Direction::Out, PortType::StdLogic));
        s.add_instance("u_gen", "pulse_gen").unwrap();
        s.set_generic_map_entry("u_gen", "WIDTH", "TOP_W").unwrap();
        s.add_instance("u_sink", "sink").unwrap();
        s.set_generic_map_entry("u_sink", "N", "TOP_W").unwrap();
        s.set_port_map_entry(
            "u_sink",
            "value",
            Some(NetRef::InstancePort("u_gen".into(), "count".into())),
        )
        .unwrap();
        s.set_port_map_entry("u_sink", "led", Some(NetRef::TopPort("led".into())))
            .unwrap();
        let mut pulse_gen = module(
            "pulse_gen",
            vec![port("count", Direction::Out, vector_of("WIDTH"))],
        );
        pulse_gen.generics.push(generic("WIDTH", "8"));
        let mut sink = module(
            "sink",
            vec![
                port("value", Direction::In, vector_of("N")),
                port("led", Direction::Out, PortType::StdLogic),
            ],
        );
        sink.generics.push(generic("N", "8"));
        (s, vec![pulse_gen, sink])
    }

    #[test]
    fn net_width_from_a_top_generic_stays_symbolic() {
        let (s, lib) = top_generic_width_project(Language::Vhdl);
        let diags = s.validate(&lib);
        assert!(diags.is_empty(), "{diags:?}");
        let vhdl = generate_design(&s, &lib).unwrap().top_code;
        assert!(
            vhdl.contains("signal u_gen_count : std_logic_vector(TOP_W - 1 downto 0);"),
            "{vhdl}"
        );

        let (s, lib) = top_generic_width_project(Language::SystemVerilog);
        let sv = generate_design(&s, &lib).unwrap().top_code;
        assert!(sv.contains("wire [TOP_W - 1:0] u_gen_count;"), "{sv}");
    }

    /// Regression: `u_a` connects only `count[28:24]`, and `u_b.value` (5
    /// bits) references that pin. Validation compared 5 bits with the 29
    /// bits of the port, and the signal had 29 bits.
    #[test]
    fn net_of_a_pin_with_a_consumer_slice_has_the_width_of_the_slice() {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.top_ports
            .push(port("led", Direction::Out, PortType::StdLogic));
        s.add_instance("u_a", "pulse_gen")
            .unwrap()
            .consumer_slices
            .insert("count".into(), SliceExpr::Range { high: 28, low: 24 });
        s.add_instance("u_b", "led_driver").unwrap();
        s.set_port_map_entry(
            "u_b",
            "value",
            Some(NetRef::InstancePort("u_a".into(), "count".into())),
        )
        .unwrap();
        s.set_port_map_entry("u_b", "led", Some(NetRef::TopPort("led".into())))
            .unwrap();
        let lib = vec![
            module("pulse_gen", vec![port("count", Direction::Out, vector(28))]),
            module(
                "led_driver",
                vec![
                    port("value", Direction::In, vector(4)),
                    port("led", Direction::Out, PortType::StdLogic),
                ],
            ),
        ];
        let diags = s.validate(&lib);
        assert!(diags.is_empty(), "{diags:?}");
        let vhdl = generate_design(&s, &lib).unwrap().top_code;
        assert!(
            vhdl.contains("signal u_a_count : std_logic_vector(4 downto 0);"),
            "{vhdl}"
        );
        assert!(vhdl.contains("count(28 downto 24) => u_a_count"), "{vhdl}");
        assert!(vhdl.contains("value => u_a_count"), "{vhdl}");
    }
}
