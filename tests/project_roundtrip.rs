//! Round-trip test for a saved `.hdlc` project file.
//!
//! Loads `tests/fixtures/fixture_project.hdlc`, regenerates HDL from the
//! loaded `Schematic`, and re-parses the generated text to assert the
//! generator produces syntactically-valid HDL of the project's language.
//!
//! Unlike `sv_roundtrip.rs` / `vhdl_roundtrip.rs` there is no source
//! `ModuleDef` to shape-compare against — the project itself is the source
//! of truth. The contract this test pins is just "load → codegen → parse"
//! does not produce malformed HDL.

mod common;

use std::path::{Path, PathBuf};

use hdl_compose::codegen;
use hdl_compose::project::load_project;
use hdl_compose::types::Language;

const FIXTURE_HDLC: &str = "tests/fixtures/fixture_project.hdlc";
const FIXTURES_DIR: &str = "tests/fixtures";

#[test]
fn roundtrip_fixture_project() {
    let (mut schematic, warnings) =
        load_project(Path::new(FIXTURE_HDLC)).expect("load fixture_project.hdlc");

    // The fixture stores library_paths as absolute paths from the original
    // author's machine. Rewrite each entry to point at the matching file in
    // this checkout's `tests/fixtures/` so the test is portable. We don't
    // edit the fixture itself per task instruction (don't rewrite fixtures).
    let fixtures_dir = PathBuf::from(FIXTURES_DIR);
    for path in &mut schematic.library_paths {
        if let Some(file_name) = path.file_name() {
            *path = fixtures_dir.join(file_name);
        }
    }

    // After rewriting, every library path must exist — if the fixture ever
    // references a file we don't have locally, fail loudly rather than
    // silently parsing an empty library.
    for path in &schematic.library_paths {
        assert!(
            path.exists(),
            "library path missing after rewrite: {} (warnings from load: {warnings:?})",
            path.display()
        );
    }

    let (library, parse_errors) = schematic.resolve_modules();
    assert!(
        parse_errors.is_empty(),
        "library re-parse failures: {parse_errors:?}"
    );

    // Cross-check: every instance's module_ref must be in the library, else
    // codegen will fail the validation step. This makes the failure message
    // useful when the fixture and library drift apart.
    for inst in &schematic.instances {
        assert!(
            library.iter().any(|m| m.name == inst.module_ref),
            "instance `{}` references module `{}` which is not in the resolved library",
            inst.name,
            inst.module_ref
        );
    }

    let diags = schematic.validate(&library);
    let errors: Vec<_> = diags.iter().filter(|d| d.is_error()).collect();
    assert!(
        errors.is_empty(),
        "loaded schematic has validation errors: {errors:?}"
    );

    // Generate HDL in the project's declared language and re-parse it.
    let generated = match schematic.language {
        Language::Vhdl => codegen::vhdl::generate_vhdl(&schematic, &library, &diags)
            .expect("generate_vhdl on loaded fixture project"),
        Language::SystemVerilog => codegen::sv::generate_sv(&schematic, &library, &diags)
            .expect("generate_sv on loaded fixture project"),
    };

    let regenerated = match schematic.language {
        Language::Vhdl => common::assert_vhdl_parses(&generated),
        Language::SystemVerilog => common::assert_sv_parses(&generated),
    };

    // Light additional check: the regenerated top should be present, and
    // expose exactly the same number of top-level ports as the loaded
    // schematic. Stops short of shape-comparing port directions / types
    // (project is the source — there's no canonical "expected" shape).
    // `parse_file` returns top-level entities only, so component
    // declarations inside the architecture do not appear here.
    let top = regenerated
        .iter()
        .find(|m| m.name == schematic.top_name)
        .unwrap_or_else(|| {
            panic!(
                "expected top module `{}` in regenerated output, got: {:?}",
                schematic.top_name,
                regenerated.iter().map(|m| &m.name).collect::<Vec<_>>()
            )
        });
    assert_eq!(
        top.ports.len(),
        schematic.top_ports.len(),
        "regenerated top exposes {} ports, schematic declared {}",
        top.ports.len(),
        schematic.top_ports.len()
    );
}
