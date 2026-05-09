//! Round-trip tests for the VHDL backend.
//!
//! For each fixture, parse the source, build a passthrough schematic,
//! generate VHDL from it, and re-parse the generated text. Then assert the
//! regenerated top module's port shape matches the original module's.

mod common;

use std::path::Path;

use hdl_compose::codegen;
use hdl_compose::types::{Language, ModuleDef};

const COUNTER_VHD: &str = "tests/fixtures/counter.vhd";
const FIFO_VHD: &str = "tests/fixtures/fifo_sync.vhd";

fn roundtrip_vhdl(fixture: &str, module_name: &str) {
    let modules = hdl_compose::parse_file(Path::new(fixture))
        .unwrap_or_else(|e| panic!("parse {fixture}: {e}"));
    let original: &ModuleDef = modules
        .iter()
        .find(|m| m.name == module_name)
        .unwrap_or_else(|| panic!("{module_name} module is present in {fixture}"));

    let s = common::build_passthrough_schematic(original, Language::Vhdl);
    let library = vec![original.clone()];
    let diags = s.validate(&library);
    let errors: Vec<_> = diags.iter().filter(|d| d.is_error()).collect();
    assert!(errors.is_empty(), "schematic has errors: {errors:?}");

    let text =
        codegen::vhdl::generate_vhdl(&s, &library, &diags).expect("generate_vhdl on passthrough");

    let regenerated = common::assert_vhdl_parses(&text);
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
fn vhdl_roundtrip_counter() {
    roundtrip_vhdl(COUNTER_VHD, "counter");
}

#[test]
fn vhdl_roundtrip_fifo_sync() {
    roundtrip_vhdl(FIFO_VHD, "fifo_sync");
}
