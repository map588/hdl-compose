//! Shared helpers for codegen round-trip integration tests.
//!
//! Each public helper is documented inline; in aggregate they let a new
//! round-trip test for a fresh fixture be a 5-line `#[test]`.
//!
//! `dead_code` is allowed because Cargo's per-test-binary compile model
//! recompiles `mod common` in every `tests/*_roundtrip.rs` file, and not all
//! of them use every helper (the SV file doesn't call `assert_vhdl_parses`
//! and vice versa).
#![allow(dead_code)]

use std::collections::{BTreeMap, HashMap};
use std::io::Write as _;

use hdl_compose::codegen::resolve_port_type;
use hdl_compose::types::{GenericDef, Language, ModuleDef, NetRef, PortDef, Schematic};

/// Re-parse generated SystemVerilog text via the public `parse_file` API.
///
/// Writes the text to a temp file with `.sv` extension and parses it.
/// Panics with a numbered dump of the input text on parse failure so the
/// developer doesn't have to round-trip through stdout to see what the
/// generator emitted.
pub fn assert_sv_parses(text: &str) -> Vec<ModuleDef> {
    parse_text(text, "sv")
}

/// Re-parse generated VHDL text via the public `parse_file` API.
///
/// See [`assert_sv_parses`] for behavior.
pub fn assert_vhdl_parses(text: &str) -> Vec<ModuleDef> {
    parse_text(text, "vhd")
}

fn parse_text(text: &str, ext: &str) -> Vec<ModuleDef> {
    let mut tmp = tempfile::Builder::new()
        .suffix(&format!(".{ext}"))
        .tempfile()
        .expect("create temp file");
    tmp.write_all(text.as_bytes()).expect("write temp file");
    tmp.flush().expect("flush temp file");

    match hdl_compose::parse_file(tmp.path()) {
        Ok(modules) => modules,
        Err(e) => {
            let numbered = number_lines(text);
            panic!("re-parse of generated {ext} failed: {e}\n--- generated text ---\n{numbered}");
        }
    }
}

fn number_lines(text: &str) -> String {
    text.lines()
        .enumerate()
        .map(|(i, l)| format!("{:4}  {l}", i + 1))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Build a passthrough schematic for `module`: every port of `module` is
/// promoted to a top-level port (same name, direction, port_type, bundle),
/// every generic of `module` is lifted into `top_generics` (same name,
/// type_name, default_value), one instance named `dut` of the module is
/// added, and every instance port is wired to its same-named top port via
/// [`NetRef::TopPort`].
///
/// The schematic top is named `<module.name>_passthrough` so the regenerated
/// top can be located unambiguously after re-parsing.
pub fn build_passthrough_schematic(module: &ModuleDef, language: Language) -> Schematic {
    let top_name = format!("{}_passthrough", module.name);
    let mut s = Schematic::new(&top_name, language);
    s.top_generics = module.generics.clone();
    s.top_ports = module.ports.clone();
    s.add_instance("dut", &module.name)
        .expect("dut is the only instance, can't be a duplicate");
    for port in &module.ports {
        s.set_port_map_entry(
            "dut",
            port.name.clone(),
            Some(NetRef::TopPort(port.name.clone())),
        )
        .expect("dut exists, set_port_map_entry can't fail");
    }
    s
}

/// Compare the port shape of `actual` against `expected`, order-insensitive.
///
/// Two ports are considered equal if their `name`, `direction`, and
/// `port_type` match. The `bundle` field is intentionally ignored — bundles
/// are auto-detected by the parser from naming heuristics and are not part
/// of the syntactic round-trip contract.
///
/// Both sides are normalized through `codegen::resolve_port_type` against
/// `expected_generics` (with no instance overrides) before comparison. This
/// is necessary because the codegen *itself* resolves `Expr("WIDTH-1")` to
/// `Literal(N)` using the source module's default generic values before
/// emission, so the regenerated module has literal ranges while the
/// originally-parsed module preserves the symbolic form. Comparing them raw
/// would always mismatch on parameterized vectors.
///
/// Panics with a clear diff on the first mismatch.
pub fn assert_shape_eq(expected: &[PortDef], actual: &[PortDef], expected_generics: &[GenericDef]) {
    let exp_map: BTreeMap<&str, &PortDef> = expected.iter().map(|p| (p.name.as_str(), p)).collect();
    let act_map: BTreeMap<&str, &PortDef> = actual.iter().map(|p| (p.name.as_str(), p)).collect();

    let exp_names: Vec<&str> = exp_map.keys().copied().collect();
    let act_names: Vec<&str> = act_map.keys().copied().collect();
    assert_eq!(
        exp_names, act_names,
        "port-name set differs after round-trip\nexpected: {exp_names:?}\nactual:   {act_names:?}"
    );

    let empty_overrides: HashMap<String, String> = HashMap::new();
    for (name, exp_port) in &exp_map {
        let act_port = act_map[name];
        assert_eq!(
            exp_port.direction, act_port.direction,
            "port {name}: direction differs (expected {:?}, got {:?})",
            exp_port.direction, act_port.direction
        );
        let exp_resolved =
            resolve_port_type(&exp_port.port_type, expected_generics, &empty_overrides);
        let act_resolved =
            resolve_port_type(&act_port.port_type, expected_generics, &empty_overrides);
        assert_eq!(
            exp_resolved, act_resolved,
            "port {name}: port_type differs (after generic resolution)\nexpected: {exp_resolved:?}\nactual:   {act_resolved:?}"
        );
    }
}

/// Compare the generic shape of `actual` against `expected`, order-insensitive.
///
/// Two generics are considered equal if their `name` and `default_value`
/// match. The `type_name` field is intentionally NOT compared — the parsers
/// capture the raw source spelling (`integer` vs `INTEGER`, `parameter` vs
/// `parameter int`, etc.) and the codegen does not normalise it. Forcing a
/// type-name match would couple this test to incidental parser-spelling
/// behaviour rather than the codegen contract.
///
/// Panics with a clear diff on the first mismatch.
pub fn assert_generics_eq(expected: &[GenericDef], actual: &[GenericDef]) {
    let exp_map: BTreeMap<&str, &GenericDef> =
        expected.iter().map(|g| (g.name.as_str(), g)).collect();
    let act_map: BTreeMap<&str, &GenericDef> =
        actual.iter().map(|g| (g.name.as_str(), g)).collect();

    let exp_names: Vec<&str> = exp_map.keys().copied().collect();
    let act_names: Vec<&str> = act_map.keys().copied().collect();
    assert_eq!(
        exp_names, act_names,
        "generic-name set differs after round-trip\nexpected: {exp_names:?}\nactual:   {act_names:?}"
    );

    for (name, exp_gen) in &exp_map {
        let act_gen = act_map[name];
        assert_eq!(
            exp_gen.default_value, act_gen.default_value,
            "generic {name}: default_value differs (expected {:?}, got {:?})",
            exp_gen.default_value, act_gen.default_value
        );
    }
}
