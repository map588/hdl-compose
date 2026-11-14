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
            // `high` and `low` are the numeric bounds. An ascending range
            // starts at the low bound: `(0 to 7)`.
            match range.dir {
                RangeDir::Downto => format!("std_logic_vector({high} downto {low})"),
                RangeDir::To => format!("std_logic_vector({low} to {high})"),
            }
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

/// Evaluate an integer expression: integers, `+ - * /`, `**`, a sign,
/// parentheses, and `$clog2(...)`. This is sufficient for the bounds that
/// HDL modules use (`WIDTH-1`, `2*N-1`, `$clog2(DEPTH)-1`) after
/// `substitute_generics`.
///
/// Returns None for other text (a name with no value, a function call) and
/// for an overflow, a division by zero, or a negative exponent.
fn eval_int_expr(expr: &str) -> Option<i64> {
    let mut parser = IntExpr { rest: expr };
    let value = parser.sum()?;
    parser.rest.trim_start().is_empty().then_some(value)
}

/// Recursive-descent parser for `eval_int_expr`. `rest` is the text that
/// is not yet read.
struct IntExpr<'a> {
    rest: &'a str,
}

impl IntExpr<'_> {
    /// Read `token` if the text continues with it.
    fn eat(&mut self, token: &str) -> bool {
        match self.rest.trim_start().strip_prefix(token) {
            Some(rest) => {
                self.rest = rest;
                true
            }
            None => false,
        }
    }

    /// `product (('+' | '-') product)*`
    fn sum(&mut self) -> Option<i64> {
        let mut value = self.product()?;
        loop {
            if self.eat("+") {
                value = value.checked_add(self.product()?)?;
            } else if self.eat("-") {
                value = value.checked_sub(self.product()?)?;
            } else {
                return Some(value);
            }
        }
    }

    /// `signed (('*' | '/') signed)*`
    fn product(&mut self) -> Option<i64> {
        let mut value = self.signed()?;
        loop {
            if self.eat("*") {
                value = value.checked_mul(self.signed()?)?;
            } else if self.eat("/") {
                value = value.checked_div(self.signed()?)?;
            } else {
                return Some(value);
            }
        }
    }

    /// `('-' | '+') signed | primary ('**' signed)?`. The sign applies to
    /// the full power, as in VHDL: `-2 ** 2` is -4.
    fn signed(&mut self) -> Option<i64> {
        if self.eat("-") {
            return self.signed()?.checked_neg();
        }
        if self.eat("+") {
            return self.signed();
        }
        let base = self.primary()?;
        if self.eat("**") {
            let exponent = u32::try_from(self.signed()?).ok()?;
            return base.checked_pow(exponent);
        }
        Some(base)
    }

    /// `'(' sum ')' | '$clog2' '(' sum ')' | integer`
    fn primary(&mut self) -> Option<i64> {
        if self.eat("(") {
            let value = self.sum()?;
            return self.eat(")").then_some(value);
        }
        if self.eat("$clog2") {
            if !self.eat("(") {
                return None;
            }
            let value = self.sum()?;
            if !self.eat(")") {
                return None;
            }
            // IEEE 1800: the ceiling of log2, and 0 for the values 0 and 1.
            return match value {
                i64::MIN..=-1 => None,
                0 | 1 => Some(0),
                _ => Some(i64::from(64 - (value - 1).leading_zeros())),
            };
        }
        let rest = self.rest.trim_start();
        let len = rest
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(rest.len());
        let (digits, tail) = rest.split_at(len);
        // `8abc` is not an integer.
        if tail.starts_with(is_identifier_char) {
            return None;
        }
        let value = digits.parse().ok()?;
        self.rest = tail;
        Some(value)
    }
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

/// True when the expression `text` uses the identifier `name`.
fn mentions_identifier(text: &str, name: &str) -> bool {
    text.split(|c| !is_identifier_char(c))
        .any(|word| word == name)
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
            RangeExpr::Expr(text) => mentions_identifier(text, name),
        })
}

/// True when a generic_map value uses the identifier `name`: the value is
/// the name, or an expression that contains it. A quoted string is text, so
/// a word in it is not an identifier.
pub(crate) fn generic_value_mentions(value: &str, name: &str) -> bool {
    let value = value.trim();
    !value.starts_with('"') && mentions_identifier(value, name)
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
        out.push(TopIntermediate {
            port_name: port.name.clone(),
            sig_name: net.name.clone(),
            port_type: top_signal_type(port, net),
            direction: port.direction.clone(),
        });
    }
    out
}

/// The type of the signal for a net with a top-level port.
///
/// The intermediate stands in for the PORT, so it takes the port's own type
/// whenever that type is fully literal. The net's resolved type (driver-side
/// generics applied) is only for ports whose bounds reference generics — a
/// sliced or narrower driver must not shrink or widen the port-adjacent
/// signal.
fn top_signal_type(port: &PortDef, net: &crate::nets::Net) -> PortType {
    if crate::nets::has_literal_bounds(&port.port_type) {
        port.port_type.clone()
    } else {
        net.port_type
            .clone()
            .unwrap_or_else(|| port.port_type.clone())
    }
}

/// The part of its signal that a reference selects, in the index numbers of
/// the signal, and true when the signal is ascending. `None` is the full
/// signal.
///
/// `Nets::bits` gives offsets from the lowest bit of the net. The signal of
/// the net can start at a different index (`(15 downto 8)`), and a slice of
/// an ascending signal must be ascending too.
pub(crate) fn signal_slice(
    schematic: &Schematic,
    nets: &crate::nets::Nets,
    reference: &NetRef,
) -> Option<(SliceExpr, bool)> {
    let offsets = nets.bits(schematic, reference)?;
    let signal_type = nets.net_for(reference).and_then(|net| {
        let top_port = net.members.iter().find_map(|member| match member {
            NetRef::TopPort(name) => schematic.top_ports.iter().find(|p| &p.name == name),
            _ => None,
        });
        match top_port {
            Some(port) => Some(top_signal_type(port, net)),
            None => net.port_type.clone(),
        }
    });
    let (lowest, ascending) = signal_type.as_ref().map_or((0, false), |t| {
        (crate::nets::lowest_index(t), crate::nets::is_ascending(t))
    });
    let slice = match offsets {
        SliceExpr::Bit(i) => SliceExpr::Bit(i + lowest),
        SliceExpr::Range { high, low } => SliceExpr::Range {
            high: high.max(low) + lowest,
            low: high.min(low) + lowest,
        },
    };
    Some((slice, ascending))
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

    // --- eval_int_expr ---

    #[test]
    fn integer_expressions_follow_the_usual_precedence() {
        for (text, value) in [
            ("7", 7),
            ("8 - 1", 7),
            ("8-1", 7),
            // Regression: the bound `(2 * 16) - 1` stayed an expression.
            ("(2 * 16) - 1", 31),
            ("2 * 16 - 1", 31),
            ("2 + 3 * 4", 14),
            ("8 - 1 - 1", 6),
            ("16 / 4 / 2", 2),
            ("7 / 2", 3),
            ("2 ** 10", 1024),
            ("2 * 3 ** 2", 18),
            ("-4 - 1", -5),
            ("-2 ** 2", -4),
            ("((3))", 3),
            ("$clog2(1024) - 1", 9),
            ("$clog2(1000)", 10),
            ("$clog2(1)", 0),
        ] {
            assert_eq!(eval_int_expr(text), Some(value), "{text}");
        }
    }

    #[test]
    fn text_that_is_not_an_integer_expression_has_no_value() {
        for text in [
            "",
            "WIDTH - 1",
            "8 -",
            "(8",
            "8)",
            "8abc",
            "8 8",
            "1 / 0",
            "2 ** -1",
            "9223372036854775807 + 1",
            "f(8)",
        ] {
            assert_eq!(eval_int_expr(text), None, "{text}");
        }
    }

    /// An expression tree, and the reference for its value.
    #[derive(Debug, Clone)]
    enum Tree {
        Int(i64),
        Neg(Box<Tree>),
        Binary(Box<Tree>, &'static str, Box<Tree>),
    }

    impl Tree {
        fn text(&self) -> String {
            match self {
                Tree::Int(v) => v.to_string(),
                Tree::Neg(t) => format!("-({})", t.text()),
                Tree::Binary(a, op, b) => format!("({}) {op} ({})", a.text(), b.text()),
            }
        }

        fn value(&self) -> Option<i64> {
            match self {
                Tree::Int(v) => Some(*v),
                Tree::Neg(t) => t.value()?.checked_neg(),
                Tree::Binary(a, op, b) => {
                    let (a, b) = (a.value()?, b.value()?);
                    match *op {
                        "+" => a.checked_add(b),
                        "-" => a.checked_sub(b),
                        "*" => a.checked_mul(b),
                        "/" => a.checked_div(b),
                        _ => a.checked_pow(u32::try_from(b).ok()?),
                    }
                }
            }
        }
    }

    fn tree() -> impl proptest::strategy::Strategy<Value = Tree> {
        use proptest::prelude::*;
        let leaf = (0i64..2000).prop_map(Tree::Int);
        leaf.prop_recursive(5, 32, 2, |inner| {
            prop_oneof![
                inner.clone().prop_map(|t| Tree::Neg(Box::new(t))),
                (
                    inner.clone(),
                    prop_oneof![Just("+"), Just("-"), Just("*"), Just("/"), Just("**")],
                    inner
                )
                    .prop_map(|(a, op, b)| Tree::Binary(
                        Box::new(a),
                        op,
                        Box::new(b)
                    )),
            ]
        })
    }

    proptest::proptest! {
        /// The value of the text of a tree is the value of the tree. A
        /// tree with an overflow, a division by zero or a negative
        /// exponent has no value.
        #[test]
        fn integer_expression_value_matches_the_tree(t in tree()) {
            proptest::prop_assert_eq!(eval_int_expr(&t.text()), t.value());
        }
    }

    #[test]
    fn generic_value_that_is_an_integer_expression_gives_a_literal_bound() {
        assert_eq!(resolved_high(Some("2 * 16")), RangeExpr::Literal(31));
    }

    // --- Vectors whose lowest index is not 0, and ascending vectors ---

    fn vector_range(high: i64, low: i64, dir: RangeDir) -> PortType {
        PortType::StdLogicVector(Range {
            high: RangeExpr::Literal(high),
            low: RangeExpr::Literal(low),
            dir,
        })
    }

    /// `u_x.dout` has the type `dout_type`, and `u_g.ena` (one bit) takes
    /// the bit `bit` of it. `connect` can put `u_x.dout` on a top-level port.
    fn bit_of_a_vector(
        language: Language,
        dout_type: PortType,
        bit: i32,
        connect: impl FnOnce(&mut Schematic),
    ) -> String {
        let mut s = Schematic::new("top", language);
        s.add_instance("u_x", "src").unwrap();
        s.add_instance("u_g", "gate").unwrap();
        s.set_port_map_entry(
            "u_g",
            "ena",
            Some(NetRef::InstancePortSlice(
                "u_x".into(),
                "dout".into(),
                SliceExpr::Bit(bit),
            )),
        )
        .unwrap();
        connect(&mut s);
        let lib = vec![
            module("src", vec![port("dout", Direction::Out, dout_type)]),
            module("gate", vec![port("ena", Direction::In, PortType::StdLogic)]),
        ];
        let errors: Vec<_> = s
            .validate(&lib)
            .into_iter()
            .filter(|d| d.is_error())
            .collect();
        assert!(errors.is_empty(), "{errors:?}");
        generate_design(&s, &lib).unwrap().top_code
    }

    /// Regression: bit 9 of a `(15 downto 8)` port was "out of range".
    #[test]
    fn slice_uses_the_index_numbers_of_the_referenced_port() {
        let hdl = bit_of_a_vector(
            Language::Vhdl,
            vector_range(15, 8, RangeDir::Downto),
            9,
            |_| {},
        );
        assert!(
            hdl.contains("signal u_x_dout : std_logic_vector(15 downto 8);"),
            "{hdl}"
        );
        assert!(hdl.contains("ena => u_x_dout(9)"), "{hdl}");
    }

    /// The signal of the net is the top-level port `data(7 downto 0)`, so
    /// bit 9 of the `(15 downto 8)` port is bit 1 of the signal.
    #[test]
    fn slice_is_translated_to_the_index_numbers_of_the_signal() {
        let hdl = bit_of_a_vector(
            Language::Vhdl,
            vector_range(15, 8, RangeDir::Downto),
            9,
            |s| {
                s.top_ports.push(port(
                    "data",
                    Direction::Out,
                    vector_range(7, 0, RangeDir::Downto),
                ));
                s.set_port_map_entry("u_x", "dout", Some(NetRef::TopPort("data".into())))
                    .unwrap();
            },
        );
        assert!(hdl.contains("ena => data_s(1)"), "{hdl}");
    }

    /// `u_x.dout` is on `wide[15:8]`. Bit 9 of the `(15 downto 8)` port is
    /// the second bit of the pin, which is bit 9 of `wide`.
    #[test]
    fn slice_of_a_pin_on_a_slice_counts_from_the_lowest_index_of_the_pin() {
        let hdl = bit_of_a_vector(
            Language::Vhdl,
            vector_range(15, 8, RangeDir::Downto),
            9,
            |s| {
                s.top_ports.push(port(
                    "wide",
                    Direction::Out,
                    vector_range(15, 0, RangeDir::Downto),
                ));
                s.set_port_map_entry(
                    "u_x",
                    "dout",
                    Some(NetRef::TopPortSlice(
                        "wide".into(),
                        SliceExpr::Range { high: 15, low: 8 },
                    )),
                )
                .unwrap();
            },
        );
        assert!(hdl.contains("dout => wide_s(15 downto 8)"), "{hdl}");
        assert!(hdl.contains("ena => wide_s(9)"), "{hdl}");
    }

    /// Regression: an ascending VHDL range was written as `7 to 0`, which
    /// is a null range. The parser stores `(0 to 7)` as high 7, low 0, To.
    #[test]
    fn ascending_range_is_written_from_low_to_high() {
        let ascending = vector_range(7, 0, RangeDir::To);
        assert_eq!(port_type_to_vhdl(&ascending), "std_logic_vector(0 to 7)");

        let hdl = bit_of_a_vector(Language::Vhdl, ascending, 2, |_| {});
        assert!(
            hdl.contains("signal u_x_dout : std_logic_vector(0 to 7);"),
            "{hdl}"
        );
        assert!(hdl.contains("ena => u_x_dout(2)"), "{hdl}");
    }

    /// A slice of an ascending signal must be ascending too.
    #[test]
    fn slice_of_an_ascending_signal_is_ascending() {
        for (language, expected) in [
            (Language::Vhdl, "din => u_x_dout(2 to 5)"),
            (Language::SystemVerilog, ".din(u_x_dout[2:5])"),
        ] {
            // SystemVerilog `[0:7]` is stored as written: high 0, low 7.
            let dout_type = match language {
                Language::Vhdl => vector_range(7, 0, RangeDir::To),
                Language::SystemVerilog => vector_range(0, 7, RangeDir::Downto),
            };
            let mut s = Schematic::new("top", language);
            s.add_instance("u_x", "src").unwrap();
            s.add_instance("u_n", "nibble").unwrap();
            s.set_port_map_entry(
                "u_n",
                "din",
                Some(NetRef::InstancePortSlice(
                    "u_x".into(),
                    "dout".into(),
                    SliceExpr::Range { high: 5, low: 2 },
                )),
            )
            .unwrap();
            let lib = vec![
                module("src", vec![port("dout", Direction::Out, dout_type)]),
                module("nibble", vec![port("din", Direction::In, vector(3))]),
            ];
            let errors: Vec<_> = s
                .validate(&lib)
                .into_iter()
                .filter(|d| d.is_error())
                .collect();
            assert!(errors.is_empty(), "{errors:?}");
            let hdl = generate_design(&s, &lib).unwrap().top_code;
            assert!(hdl.contains(expected), "{hdl}");
        }
    }
}
