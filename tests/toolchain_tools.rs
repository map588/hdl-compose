//! Tests that run the external tools (oss-cad-suite) on a real project.
//!
//! A test whose tools are not on PATH prints a message and passes: the
//! suite must stay green on a host without the tools. Run `hdlc doctor` to
//! see which tools the host has.

mod common;

use std::path::Path;

use hdl_compose::codegen;
use hdl_compose::toolchain::{self, StagedBuild, ToolchainConfig};
use hdl_compose::types::{Language, ModuleDef};

/// True when each tool is on PATH. Else a message names the missing tools.
fn have(tools: &[&str]) -> bool {
    let missing: Vec<&&str> = tools
        .iter()
        .filter(|tool| !toolchain::tool_on_path(tool))
        .collect();
    if !missing.is_empty() {
        eprintln!("skipped: not on PATH: {missing:?}");
    }
    missing.is_empty()
}

/// Write the sources into `dir`, make a project whose top passes each port
/// of the module `dut` through, and stage it.
fn stage_passthrough(dir: &Path, sources: &[(&str, &str)], dut: &str) -> StagedBuild {
    let mut library: Vec<ModuleDef> = Vec::new();
    for (name, text) in sources {
        let path = dir.join(name);
        std::fs::write(&path, text).unwrap();
        library.extend(hdl_compose::parse_file(&path).unwrap());
    }
    let module = library.iter().find(|m| m.name == dut).unwrap();
    let mut schematic = common::build_passthrough_schematic(module, Language::SystemVerilog);
    for (name, _) in sources {
        schematic.add_library_path(dir.join(name));
    }
    let design = codegen::generate_design(&schematic, &library).expect("codegen");
    toolchain::stage(
        &schematic,
        &library,
        &design,
        &ToolchainConfig::default(),
        dir,
    )
    .unwrap()
}

const INNER: &str =
    "module inner (input logic [7:0] d, output logic q);\n  assign q = ^d;\nendmodule\n";

/// `outer` connects `width` bits to the 8-bit port of `inner`.
fn outer(connection: &str) -> String {
    format!(
        "module outer (input logic [7:0] d, output logic q);\n  inner u_inner (.d({connection}), .q(q));\nendmodule\n"
    )
}

/// slang reports a port connection of the wrong width only as a warning,
/// and its warnings are off by default: plain `slang --top` exits 0 on this
/// design. `verilator --lint-only`, which `check` used before, stops on it.
/// The mismatch is inside a library module, where `Schematic::validate`
/// does not look.
#[test]
fn check_fails_on_a_port_width_mismatch_inside_a_module() {
    if !have(&["slang"]) {
        return;
    }
    let good = tempfile::tempdir().unwrap();
    let staged = stage_passthrough(
        good.path(),
        &[("inner.sv", INNER), ("outer.sv", &outer("d"))],
        "outer",
    );
    assert_eq!(toolchain::run_check(&staged), Ok(true));

    let bad = tempfile::tempdir().unwrap();
    let staged = stage_passthrough(
        bad.path(),
        &[("inner.sv", INNER), ("outer.sv", &outer("d[6:0]"))],
        "outer",
    );
    assert_eq!(toolchain::run_check(&staged), Ok(false));
}

/// A package and a struct port: `read_verilog -sv` cannot read this design.
const PACKAGED: &str = "package bus_pkg;\n  typedef struct packed { logic [3:0] hi; logic [3:0] lo; } pair_t;\nendpackage\n\nmodule swap import bus_pkg::*; (input logic [7:0] d, output logic [7:0] q);\n  pair_t p;\n  assign p = d;\n  assign q = {p.lo, p.hi};\nendmodule\n";

#[test]
fn synth_reads_a_package_and_a_struct_through_slang() {
    if !have(&["yosys"]) {
        return;
    }
    let project = tempfile::tempdir().unwrap();
    let staged = stage_passthrough(project.path(), &[("swap.sv", PACKAGED)], "swap");
    assert_eq!(toolchain::run_synth(&staged), Ok(true));
    // The first variant passed, so the fallback did not run.
    let record = std::fs::read_to_string(staged.dir.join("steps.log")).unwrap();
    assert_eq!(record.matches("$ yosys").count(), 1, "{record}");
    assert!(record.contains("-m slang"), "{record}");
}
