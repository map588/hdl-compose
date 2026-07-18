use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::types::Schematic;

const CURRENT_VERSION: u32 = 4;
const MIN_SUPPORTED_VERSION: u32 = 2;

#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error(
        "unsupported project version: {0} (supported: {MIN_SUPPORTED_VERSION}..={CURRENT_VERSION})"
    )]
    UnsupportedVersion(u32),

    #[error("parse error while loading library: {0}")]
    LibraryParse(#[from] crate::ParseError),
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectFile {
    version: u32,
    #[serde(flatten)]
    schematic: Schematic,
}

/// Save a schematic to a .hdlc project file.
///
/// Library paths under the project's directory are written relative to it so
/// the project stays portable (load_project resolves them back against the
/// project dir). Paths elsewhere stay absolute.
pub fn save_project(schematic: &Schematic, path: &Path) -> Result<(), ProjectError> {
    let mut schematic = schematic.clone();
    if let Some(project_dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        for lib_path in schematic.library_paths.iter_mut() {
            if let Ok(rel) = lib_path.strip_prefix(project_dir) {
                *lib_path = rel.to_path_buf();
            }
        }
    }
    let project = ProjectFile {
        version: CURRENT_VERSION,
        schematic,
    };
    let json = serde_json::to_string_pretty(&project)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Load a schematic from a .hdlc project file.
/// Re-parses library paths to rebuild the module library.
/// Returns (schematic, warnings) where warnings include missing library files.
pub fn load_project(path: &Path) -> Result<(Schematic, Vec<String>), ProjectError> {
    let content = std::fs::read_to_string(path)?;
    let project: ProjectFile = serde_json::from_str(&content)?;

    if project.version < MIN_SUPPORTED_VERSION || project.version > CURRENT_VERSION {
        return Err(ProjectError::UnsupportedVersion(project.version));
    }

    let mut schematic = project.schematic;
    let mut warnings = Vec::new();

    // Resolve relative library paths against the project file's directory so
    // .hdlc projects can use relative paths and remain portable. Absolute
    // paths are preserved as-is. After this, every entry is an absolute path
    // (or unchanged, if already absolute).
    if let Some(project_dir) = path.parent() {
        for lib_path in schematic.library_paths.iter_mut() {
            if lib_path.is_relative() {
                *lib_path = project_dir.join(&lib_path);
            }
        }
    }

    // Verify library paths exist (warn on missing, don't fail)
    schematic.library_paths.retain(|lib_path| {
        if lib_path.exists() {
            true
        } else {
            warnings.push(format!("library file not found: {}", lib_path.display()));
            true // keep the path in the list so user can fix it
        }
    });

    // Clean up port_map / alias entries that reference instances that no
    // longer exist. Projects saved before `remove_instance` swept siblings
    // can carry dangling references that resurrect phantom wires when a
    // new instance is dropped with the same name.
    let cleared = schematic.cleanup_stale_refs();
    if cleared > 0 {
        warnings.push(format!(
            "cleared {cleared} stale port-map / alias reference(s) to instances that no longer exist"
        ));
    }

    Ok((schematic, warnings))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use tempfile::TempDir;

    #[test]
    fn round_trip() {
        let mut s = Schematic::new("top_level", Language::Vhdl);
        s.top_ports.push(PortDef {
            name: "clk".into(),
            direction: Direction::In,
            port_type: PortType::StdLogic,
            bundle: None,
        });
        s.add_instance("u_fifo", "fifo_sync").unwrap();
        s.set_port_map_entry("u_fifo", "clk", Some(NetRef::TopPort("clk".into())))
            .unwrap();
        s.set_generic_map_entry("u_fifo", "DEPTH", "1024").unwrap();
        s.set_alias(
            NetRef::InstancePort("u_fifo".into(), "dout".into()),
            "fifo_out",
        );

        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.hdlc");

        save_project(&s, &path).unwrap();
        let (loaded, _warnings) = load_project(&path).unwrap();

        assert_eq!(loaded.top_name, "top_level");
        assert_eq!(loaded.language, Language::Vhdl);
        assert_eq!(loaded.instances.len(), 1);
        assert_eq!(loaded.instances[0].name, "u_fifo");
        assert_eq!(
            loaded.instances[0].port_map.get("clk"),
            Some(&Some(NetRef::TopPort("clk".into())))
        );
        assert_eq!(
            loaded.instances[0].generic_map.get("DEPTH"),
            Some(&"1024".to_string())
        );
        assert_eq!(
            loaded
                .aliases
                .get(&NetRef::InstancePort("u_fifo".into(), "dout".into())),
            Some(&"fifo_out".to_string())
        );
    }

    #[test]
    fn reject_unknown_version() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("bad.hdlc");
        std::fs::write(&path, r#"{"version": 99, "top_name": "x", "language": "Vhdl", "top_generics": [], "top_ports": [], "instances": [], "aliases": {}, "library_paths": []}"#).unwrap();

        let err = load_project(&path).unwrap_err();
        assert!(matches!(err, ProjectError::UnsupportedVersion(99)));
    }

    #[test]
    fn save_writes_current_version() {
        let s = Schematic::new("top", Language::Vhdl);
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("v.hdlc");
        save_project(&s, &path).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains(&format!("\"version\": {CURRENT_VERSION}")));
    }

    #[test]
    fn load_v2_without_manual_bundles_succeeds() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("v2.hdlc");
        std::fs::write(
            &path,
            r#"{
              "version": 2,
              "top_name": "top",
              "language": "Vhdl",
              "top_generics": [],
              "top_ports": [],
              "instances": [
                {
                  "name": "u_a",
                  "module_ref": "mod_a",
                  "generic_map": {},
                  "port_map": {},
                  "position": [0.0, 0.0]
                }
              ],
              "aliases": {},
              "library_paths": []
            }"#,
        )
        .unwrap();
        let (loaded, _warnings) = load_project(&path).unwrap();
        assert_eq!(loaded.instances.len(), 1);
        assert!(loaded.instances[0].manual_bundles.is_empty());
    }

    #[test]
    fn v3_manual_bundles_round_trip() {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.add_instance("spi_0", "spi").unwrap();
        s.get_instance_mut("spi_0").unwrap().manual_bundles.insert(
            "spi".into(),
            vec![
                "mosi".into(),
                "miso".into(),
                "sclk".into(),
                "cs_n".into(),
            ],
        );
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("v3.hdlc");
        save_project(&s, &path).unwrap();
        let (loaded, _warnings) = load_project(&path).unwrap();
        let loaded_bundle = loaded
            .get_instance("spi_0")
            .unwrap()
            .manual_bundles
            .get("spi")
            .unwrap();
        assert_eq!(loaded_bundle, &vec![
            "mosi".to_string(),
            "miso".to_string(),
            "sclk".to_string(),
            "cs_n".to_string(),
        ]);
    }

    #[test]
    fn load_v3_without_consumer_slices_succeeds() {
        // v3 files predate the consumer_slices field; they must load with the
        // field defaulted to empty.
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("v3.hdlc");
        std::fs::write(
            &path,
            r#"{
              "version": 3,
              "top_name": "top",
              "language": "Vhdl",
              "top_generics": [],
              "top_ports": [],
              "instances": [
                {
                  "name": "u_a",
                  "module_ref": "mod_a",
                  "generic_map": {},
                  "port_map": {},
                  "position": [0.0, 0.0],
                  "manual_bundles": {}
                }
              ],
              "aliases": {},
              "library_paths": []
            }"#,
        )
        .unwrap();
        let (loaded, _warnings) = load_project(&path).unwrap();
        assert_eq!(loaded.instances.len(), 1);
        assert!(loaded.instances[0].consumer_slices.is_empty());
        // Re-saves as v4.
        save_project(&loaded, &path).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("\"version\": 4"));
    }

    #[test]
    fn consumer_slice_round_trip() {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.add_instance("u_b", "mod_b").unwrap();
        s.get_instance_mut("u_b").unwrap().consumer_slices.insert(
            "din".into(),
            crate::types::SliceExpr::Range { high: 3, low: 0 },
        );
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("rt.hdlc");
        save_project(&s, &path).unwrap();
        let (loaded, _warnings) = load_project(&path).unwrap();
        let cs = loaded
            .get_instance("u_b")
            .unwrap()
            .consumer_slices
            .get("din")
            .unwrap();
        assert!(matches!(
            cs,
            crate::types::SliceExpr::Range { high: 3, low: 0 }
        ));
    }

    #[test]
    fn library_paths_saved_relative_to_project_dir() {
        let dir = TempDir::new().unwrap();
        let lib = dir.path().join("rtl").join("counter.vhd");
        std::fs::create_dir_all(lib.parent().unwrap()).unwrap();
        std::fs::copy("tests/fixtures/counter.vhd", &lib).unwrap();

        let mut s = Schematic::new("top", Language::Vhdl);
        s.library_paths.push(lib.clone());
        let path = dir.path().join("proj.hdlc");
        save_project(&s, &path).unwrap();

        // On disk: relative, so the project dir can move wholesale.
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(
            content.contains("rtl/counter.vhd") && !content.contains(dir.path().to_str().unwrap()),
            "{content}"
        );
        // On load: resolved back to absolute.
        let (loaded, warnings) = load_project(&path).unwrap();
        assert_eq!(loaded.library_paths, vec![lib]);
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    #[test]
    fn missing_library_warns() {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.library_paths.push("/nonexistent/file.vhd".into());

        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.hdlc");

        save_project(&s, &path).unwrap();
        let (_loaded, warnings) = load_project(&path).unwrap();

        assert!(warnings.iter().any(|w| w.contains("not found")));
    }
}
