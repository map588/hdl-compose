use std::path::{Path, PathBuf};

use hdl_compose::codegen;
use hdl_compose::schematic::DiagnosticLevel;
use hdl_compose::types::*;

const COUNTER_VHD: &str = "tests/fixtures/counter.vhd";
const COUNTER_V: &str = "tests/fixtures/counter.v";
const FIFO_VHD: &str = "tests/fixtures/fifo_sync.vhd";

#[test]
fn parse_vhdl_fixture() {
    let modules = hdl_compose::parse_file(Path::new(COUNTER_VHD)).unwrap();
    assert_eq!(modules.len(), 1);

    let m = &modules[0];
    assert_eq!(m.name, "counter");
    assert_eq!(m.generics.len(), 1);
    assert_eq!(m.generics[0].name, "WIDTH");
    assert_eq!(m.ports.len(), 5);
    assert_eq!(m.ports[0].name, "clk");
    assert_eq!(m.ports[0].direction, Direction::In);
}

#[test]
fn parse_verilog_fixture() {
    let modules = hdl_compose::parse_file(Path::new(COUNTER_V)).unwrap();
    assert_eq!(modules.len(), 1);

    let m = &modules[0];
    assert_eq!(m.name, "counter");
    assert_eq!(m.generics.len(), 1);
    assert_eq!(m.generics[0].name, "WIDTH");
    assert_eq!(m.ports.len(), 4);
}

#[test]
fn parse_fifo_fixture() {
    let modules = hdl_compose::parse_file(Path::new(FIFO_VHD)).unwrap();
    assert_eq!(modules.len(), 1);

    let m = &modules[0];
    assert_eq!(m.name, "fifo_sync");
    assert_eq!(m.generics.len(), 2);
    assert_eq!(m.ports.len(), 8);
}

#[test]
fn schematic_codegen_from_fixtures() {
    // Parse fixtures
    let counter_mods = hdl_compose::parse_file(Path::new(COUNTER_VHD)).unwrap();
    let fifo_mods = hdl_compose::parse_file(Path::new(FIFO_VHD)).unwrap();

    let mut library = Vec::new();
    library.extend(counter_mods);
    library.extend(fifo_mods);

    // Build schematic
    let mut s = Schematic::new("test_top", Language::Vhdl);
    s.top_ports.push(PortDef {
        name: "clk".into(),
        direction: Direction::In,
        port_type: PortType::StdLogic,
        bundle: None,
    });
    s.top_ports.push(PortDef {
        name: "rst_n".into(),
        direction: Direction::In,
        port_type: PortType::StdLogic,
        bundle: None,
    });

    s.add_instance("u_counter", "counter").unwrap();
    s.set_port_map_entry("u_counter", "clk", Some(NetRef::TopPort("clk".into())))
        .unwrap();
    s.set_port_map_entry("u_counter", "rst_n", Some(NetRef::TopPort("rst_n".into())))
        .unwrap();
    s.set_generic_map_entry("u_counter", "WIDTH", "8").unwrap();

    s.add_instance("u_fifo", "fifo_sync").unwrap();
    s.set_port_map_entry("u_fifo", "clk", Some(NetRef::TopPort("clk".into())))
        .unwrap();
    s.set_port_map_entry("u_fifo", "rst_n", Some(NetRef::TopPort("rst_n".into())))
        .unwrap();
    s.set_generic_map_entry("u_fifo", "DEPTH", "512").unwrap();
    s.set_generic_map_entry("u_fifo", "WIDTH", "8").unwrap();

    // Connect counter.count -> fifo.din
    s.set_port_map_entry(
        "u_fifo",
        "din",
        Some(NetRef::InstancePort("u_counter".into(), "count".into())),
    )
    .unwrap();

    // Validate
    let diags = s.validate(&library);
    let errors: Vec<_> = diags.iter().filter(|d| d.is_error()).collect();
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");

    // Codegen
    let output = codegen::vhdl::generate_vhdl(&s, &library, &diags).unwrap();
    assert!(output.contains("entity test_top is"));
    assert!(output.contains("architecture structural of test_top is"));
    assert!(output.contains("u_counter : counter"));
    assert!(output.contains("u_fifo : fifo_sync"));
    assert!(output.contains("signal u_counter_count"));
    assert!(output.contains("din => u_counter_count"));
}

#[test]
fn validate_with_known_errors() {
    let mut s = Schematic::new("bad_top", Language::Vhdl);
    s.add_instance("u_missing", "nonexistent_module").unwrap();
    s.set_port_map_entry(
        "u_missing",
        "data",
        Some(NetRef::InstancePort("u_ghost".into(), "out".into())),
    )
    .unwrap();

    let diags = s.validate(&[]);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();

    assert!(!errors.is_empty());
    // Should have: unresolved module + missing instance reference
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("not found in library"))
    );
    assert!(errors.iter().any(|d| d.message.contains("u_ghost")));
}

#[test]
fn codegen_refuses_on_errors() {
    let mut s = Schematic::new("bad", Language::Vhdl);
    s.add_instance("u_x", "nonexistent").unwrap();

    let diags = s.validate(&[]);
    let result = codegen::vhdl::generate_vhdl(&s, &[], &diags);
    assert!(result.is_err());
}

#[test]
fn resolve_modules_collects_partial_results() {
    // Mix of valid + missing paths must NOT discard the valid modules.
    let mut s = Schematic::new("partial", Language::Vhdl);
    s.library_paths.push(PathBuf::from(COUNTER_VHD));
    s.library_paths
        .push(PathBuf::from("/tmp/this/path/does/not/exist.vhd"));
    s.library_paths.push(PathBuf::from(FIFO_VHD));

    let (modules, errors) = s.resolve_modules();

    let module_names: Vec<&str> = modules.iter().map(|m| m.name.as_str()).collect();
    assert!(module_names.contains(&"counter"), "got: {module_names:?}");
    assert!(module_names.contains(&"fifo_sync"), "got: {module_names:?}");
    assert_eq!(errors.len(), 1, "expected exactly one parse error");
    assert!(
        errors[0].0.ends_with("exist.vhd"),
        "wrong path in error: {:?}",
        errors[0].0
    );
}

#[test]
fn resolve_modules_all_good_returns_empty_errors() {
    let mut s = Schematic::new("clean", Language::Vhdl);
    s.library_paths.push(PathBuf::from(COUNTER_VHD));
    s.library_paths.push(PathBuf::from(FIFO_VHD));

    let (modules, errors) = s.resolve_modules();
    assert_eq!(modules.len(), 2);
    assert!(errors.is_empty());
}
