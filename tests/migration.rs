//! .hdlc format-version migration tests.
//!
//! `load_project` migrates older supported versions in-process (missing
//! fields fill via serde defaults); `save_project` always writes
//! CURRENT_VERSION. `hdl-compose migrate` is just load + save, so these
//! tests pin the CLI's behavior too.

use std::path::PathBuf;

use hdl_compose::project::{ProjectError, load_project, save_project};
use tempfile::TempDir;

/// A minimal v2 project: predates `manual_bundles` (v3) and
/// `consumer_slices` (v4). Both must deserialize as empty defaults.
const V2_PROJECT: &str = r#"{
  "version": 2,
  "top_name": "old_top",
  "language": "Vhdl",
  "top_generics": [],
  "top_ports": [
    { "name": "clk", "direction": "In", "port_type": "StdLogic", "bundle": null }
  ],
  "instances": [
    {
      "name": "u_a",
      "module_ref": "mod_a",
      "generic_map": {},
      "port_map": { "clk": { "TopPort": "clk" } },
      "position": [0.0, 0.0]
    }
  ],
  "aliases": {},
  "library_paths": []
}"#;

fn write_temp(dir: &TempDir, name: &str, contents: &str) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, contents).unwrap();
    path
}

#[test]
fn v2_loads_with_defaulted_fields() {
    let dir = TempDir::new().unwrap();
    let path = write_temp(&dir, "old.hdlc", V2_PROJECT);

    let (schematic, _warnings) = load_project(&path).unwrap();
    assert_eq!(schematic.top_name, "old_top");
    assert_eq!(schematic.instances.len(), 1);
    let inst = &schematic.instances[0];
    assert!(inst.manual_bundles.is_empty(), "v3 field must default");
    assert!(inst.consumer_slices.is_empty(), "v4 field must default");
    assert!(!inst.dirty);
}

#[test]
fn migrate_rewrites_at_current_version() {
    let dir = TempDir::new().unwrap();
    let path = write_temp(&dir, "old.hdlc", V2_PROJECT);

    // load + save is exactly what `hdl-compose migrate` does.
    let (schematic, _) = load_project(&path).unwrap();
    save_project(&schematic, &path).unwrap();

    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(json["version"], 5, "save must stamp CURRENT_VERSION");

    // Idempotent: a second migration produces byte-identical output.
    let first = std::fs::read_to_string(&path).unwrap();
    let (schematic, _) = load_project(&path).unwrap();
    save_project(&schematic, &path).unwrap();
    let second = std::fs::read_to_string(&path).unwrap();
    assert_eq!(first, second, "migration must be idempotent");
}

#[test]
fn migrated_project_round_trips() {
    let dir = TempDir::new().unwrap();
    let path = write_temp(&dir, "old.hdlc", V2_PROJECT);

    let (schematic, _) = load_project(&path).unwrap();
    save_project(&schematic, &path).unwrap();
    let (reloaded, _) = load_project(&path).unwrap();
    assert_eq!(reloaded.top_name, schematic.top_name);
    assert_eq!(reloaded.instances.len(), schematic.instances.len());
    assert_eq!(
        reloaded.instances[0].port_map,
        schematic.instances[0].port_map
    );
}

#[test]
fn unsupported_versions_are_rejected() {
    let dir = TempDir::new().unwrap();
    for bad in [1u32, 0, 99] {
        let contents = V2_PROJECT.replace("\"version\": 2", &format!("\"version\": {bad}"));
        let path = write_temp(&dir, &format!("v{bad}.hdlc"), &contents);
        match load_project(&path) {
            Err(ProjectError::UnsupportedVersion(v)) => assert_eq!(v, bad),
            other => panic!("version {bad} should be rejected, got: {other:?}"),
        }
    }
}

#[test]
fn v3_fixture_migrates_to_v4() {
    // The checked-in fixtures are v3 on purpose — loading them exercises the
    // live migration path. Saving must bump them to current.
    let dir = TempDir::new().unwrap();
    let (schematic, _) =
        load_project(std::path::Path::new("tests/fixtures/fixture_project.hdlc")).unwrap();
    let out = dir.path().join("migrated.hdlc");
    save_project(&schematic, &out).unwrap();
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out).unwrap()).unwrap();
    assert_eq!(json["version"], 5);
}
