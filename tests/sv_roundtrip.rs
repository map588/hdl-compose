//! Round-trip tests for the SystemVerilog backend.
//!
//! For each fixture, parse the source, build a passthrough schematic,
//! generate SV from it, and re-parse the generated text. Then assert the
//! regenerated top module's port shape matches the original module's.

mod common;

use std::path::Path;

use hdl_compose::codegen;
use hdl_compose::types::{Language, ModuleDef};

const COUNTER_V: &str = "tests/fixtures/counter.v";
const FIFO_V: &str = "tests/fixtures/fifo_sync.v";

fn roundtrip_sv(fixture: &str, module_name: &str) {
    let modules = hdl_compose::parse_file(Path::new(fixture))
        .unwrap_or_else(|e| panic!("parse {fixture}: {e}"));
    let original: &ModuleDef = modules
        .iter()
        .find(|m| m.name == module_name)
        .unwrap_or_else(|| panic!("{module_name} module is present in {fixture}"));

    let s = common::build_passthrough_schematic(original, Language::SystemVerilog);
    let library = vec![original.clone()];
    let diags = s.validate(&library);
    let errors: Vec<_> = diags.iter().filter(|d| d.is_error()).collect();
    assert!(errors.is_empty(), "schematic has errors: {errors:?}");

    let text = codegen::sv::generate_sv(&s, &library, &diags).expect("generate_sv on passthrough");

    let regenerated = common::assert_sv_parses(&text);
    let expected_top = format!("{module_name}_passthrough");
    let top = regenerated
        .iter()
        .find(|m| m.name == expected_top)
        .unwrap_or_else(|| {
            panic!(
                "expected `{expected_top}` module in regenerated output, got: {:?}",
                regenerated.iter().map(|m| &m.name).collect::<Vec<_>>()
            )
        });

    common::assert_shape_eq(&original.ports, &top.ports, &original.generics);
    common::assert_generics_eq(&original.generics, &top.generics);
}

#[test]
fn sv_roundtrip_counter() {
    roundtrip_sv(COUNTER_V, "counter");
}

#[test]
fn sv_roundtrip_fifo_sync() {
    roundtrip_sv(FIFO_V, "fifo_sync");
}
