//! Round-trip tests for the SystemVerilog backend.
//!
//! For each fixture, parse the source, build a passthrough schematic,
//! generate SV from it, and re-parse the generated text. Then assert the
//! regenerated top module's port shape matches the original module's.

mod common;

use std::path::Path;

use hdl_compose::codegen;
use hdl_compose::types::Language;

const COUNTER_V: &str = "tests/fixtures/counter.v";

#[test]
fn sv_roundtrip_counter() {
    let modules = hdl_compose::parse_file(Path::new(COUNTER_V)).expect("parse counter.v");
    let counter = modules
        .iter()
        .find(|m| m.name == "counter")
        .expect("counter module is present");

    let s = common::build_passthrough_schematic(counter, Language::SystemVerilog);
    let library = vec![counter.clone()];
    let diags = s.validate(&library);
    let errors: Vec<_> = diags.iter().filter(|d| d.is_error()).collect();
    assert!(errors.is_empty(), "schematic has errors: {errors:?}");

    let text = codegen::sv::generate_sv(&s, &library, &diags).expect("generate_sv on passthrough");

    let regenerated = common::assert_sv_parses(&text);
    let top = regenerated
        .iter()
        .find(|m| m.name == "counter_passthrough")
        .unwrap_or_else(|| {
            panic!(
                "expected `counter_passthrough` module in regenerated output, got: {:?}",
                regenerated.iter().map(|m| &m.name).collect::<Vec<_>>()
            )
        });

    common::assert_shape_eq(&counter.ports, &top.ports, &counter.generics);
    common::assert_generics_eq(&counter.generics, &top.generics);
}
