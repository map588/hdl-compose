//! The designs of one folder.
//!
//! A design is one `.hdlc` file. Its generated HDL is a module, and each
//! other design of the same folder can place that module. This file reads
//! the facts that the folder holds. It stores nothing: the `.hdlc` files
//! and the generated files are the only state.

use std::path::{Path, PathBuf};

use crate::types::Language;

/// One design of a folder, as its `.hdlc` file gives it.
#[derive(Debug, Clone, PartialEq)]
pub struct FolderDesign {
    /// The `.hdlc` file.
    pub path: PathBuf,
    /// The name of the module that the design generates.
    pub top_name: String,
    pub language: Language,
    /// True when the generated top file is at `module_file`. Only then is
    /// the design a module for other designs.
    pub generated: bool,
    /// The module of each instance of the design.
    pub uses: Vec<String>,
    /// The name of each group. Each group is a generated file of its own.
    pub groups: Vec<String>,
    /// The library sources of the design.
    pub library_paths: Vec<PathBuf>,
}

impl FolderDesign {
    /// The place of the generated top file: `<folder>/<top_name>.<ext>`,
    /// where Generate HDL writes it. A file that "Generate HDL As..." or
    /// `hdlc codegen -o` wrote to another place is an export; it does not
    /// make the design a module of the folder.
    pub fn module_file(&self) -> PathBuf {
        self.generated_file(&self.top_name)
    }

    fn generated_file(&self, module: &str) -> PathBuf {
        let extension = match self.language {
            Language::Vhdl => "vhd",
            Language::SystemVerilog => "sv",
        };
        self.path.with_file_name(format!("{module}.{extension}"))
    }

    /// The files that a parent needs to use the design as a module: the
    /// generated top file, the generated file of each group that is on the
    /// disk, and each library source of the design.
    pub fn sources(&self) -> Vec<PathBuf> {
        let groups = self
            .groups
            .iter()
            .map(|group| self.generated_file(group))
            .filter(|file| is_generated_file(file));
        std::iter::once(self.module_file())
            .chain(groups)
            .chain(self.library_paths.iter().cloned())
            .collect()
    }

    /// The reason that a design with the name `parent` and the language
    /// `language` cannot place this design, as a short text for the list
    /// of designs. `None` when it can.
    pub fn obstacle(
        &self,
        designs: &[FolderDesign],
        parent: &str,
        language: &Language,
    ) -> Option<&'static str> {
        if self.language != *language {
            Some("other language")
        } else if !self.generated {
            Some("not generated")
        } else if self.contains(designs, parent) {
            Some("contains this design")
        } else {
            None
        }
    }

    /// True when this design places the design `name`, directly or through
    /// other designs of the folder.
    fn contains(&self, designs: &[FolderDesign], name: &str) -> bool {
        let mut seen: Vec<&str> = Vec::new();
        let mut pending: Vec<&FolderDesign> = vec![self];
        while let Some(design) = pending.pop() {
            if seen.contains(&design.top_name.as_str()) {
                continue;
            }
            seen.push(&design.top_name);
            for module in &design.uses {
                if module == name {
                    return true;
                }
                pending.extend(designs.iter().filter(|d| d.top_name == *module));
            }
        }
        false
    }
}

/// True when `path` is a file that hdlc generated.
fn is_generated_file(path: &Path) -> bool {
    std::fs::read_to_string(path).is_ok_and(|text| crate::codegen::is_generated(&text))
}

/// The designs in `dir`, in the order of their file names. A design below
/// `dir` is not in the list: a build directory holds copies. A file that
/// does not load is not in the list.
pub fn folder_designs(dir: &Path) -> Vec<FolderDesign> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "hdlc") && path.is_file())
        .collect();
    paths.sort();
    paths
        .into_iter()
        .filter_map(|path| {
            let (schematic, _warnings) = crate::project::load_project(&path).ok()?;
            let mut design = FolderDesign {
                path,
                top_name: schematic.top_name,
                language: schematic.language,
                generated: false,
                uses: schematic
                    .instances
                    .into_iter()
                    .map(|i| i.module_ref)
                    .collect(),
                groups: schematic.groups.into_iter().map(|g| g.name).collect(),
                library_paths: schematic.library_paths,
            };
            design.generated = is_generated_file(&design.module_file());
            Some(design)
        })
        .collect()
}

/// The name of the design in the file `path`: the name of the file with no
/// extension. The name goes into the generated HDL as the module name, so
/// it must be a name of the language.
pub fn design_name(path: &Path, language: &Language) -> Result<String, String> {
    let name = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default();
    let mut chars = name.chars();
    let starts_with_letter = chars.next().is_some_and(|c| c.is_ascii_alphabetic());
    if !starts_with_letter || !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(format!(
            "'{name}' is not a name for a design: the name of the file is the name of the \
             module, so it starts with a letter and has only letters, digits and '_'"
        ));
    }
    if crate::reserved::is_reserved_word(language, name) {
        let language = match language {
            Language::Vhdl => "VHDL",
            Language::SystemVerilog => "SystemVerilog",
        };
        return Err(format!(
            "'{name}' is not a name for a design: it is a reserved word in {language}"
        ));
    }
    Ok(name.to_string())
}

/// Refuse the name `name` for the design file `path` when another design
/// of that folder has it. Two designs with one name generate one file. The
/// comparison ignores the case: VHDL does, and so does the file system of
/// macOS.
pub fn check_name_is_free(path: &Path, name: &str) -> Result<(), String> {
    let dir = path.parent().unwrap_or(Path::new(""));
    match folder_designs(dir)
        .into_iter()
        .find(|d| d.path.file_name() != path.file_name() && d.top_name.eq_ignore_ascii_case(name))
    {
        Some(other) => Err(format!(
            "the name '{name}' is the name of the design in {}",
            other.path.display()
        )),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Schematic;

    /// Write a design with the module name `top_name` into `dir/<file>`.
    fn write_design(dir: &Path, file: &str, top_name: &str, language: Language) -> PathBuf {
        let path = dir.join(file);
        crate::project::save_project(&Schematic::new(top_name, language), &path).unwrap();
        path
    }

    #[test]
    fn folder_designs_gives_each_design_of_the_folder_and_none_below_it() {
        let dir = tempfile::tempdir().unwrap();
        let b = write_design(dir.path(), "b.hdlc", "b", Language::SystemVerilog);
        let a = write_design(dir.path(), "a.hdlc", "a", Language::Vhdl);
        // Not designs of this folder.
        std::fs::create_dir_all(dir.path().join("build/a")).unwrap();
        write_design(&dir.path().join("build/a"), "a.hdlc", "a", Language::Vhdl);
        std::fs::write(dir.path().join("broken.hdlc"), "{ not json").unwrap();
        std::fs::write(dir.path().join("a.vhd"), "entity a is end;").unwrap();
        std::fs::create_dir(dir.path().join("folder.hdlc")).unwrap();

        let designs = folder_designs(dir.path());
        let facts: Vec<(&Path, &str, &Language, bool)> = designs
            .iter()
            .map(|d| {
                (
                    d.path.as_path(),
                    d.top_name.as_str(),
                    &d.language,
                    d.generated,
                )
            })
            .collect();
        // `a.vhd` is at the place of the generated file of `a`, but hdlc
        // did not write it.
        assert_eq!(
            facts,
            [
                (a.as_path(), "a", &Language::Vhdl, false),
                (b.as_path(), "b", &Language::SystemVerilog, false),
            ]
        );
        assert_eq!(folder_designs(&dir.path().join("no_such_folder")), []);
    }

    #[test]
    fn design_name_is_the_name_of_the_file() {
        let name = |file: &str, language| design_name(Path::new(file), &language);
        assert_eq!(
            name("/x/uart_rx2.hdlc", Language::Vhdl),
            Ok("uart_rx2".into())
        );
        assert_eq!(name("top.hdlc", Language::SystemVerilog), Ok("top".into()));

        for file in [
            "2fast.hdlc",
            "my-design.hdlc",
            "_x.hdlc",
            "a b.hdlc",
            ".hdlc",
            "é.hdlc",
        ] {
            let error = name(file, Language::Vhdl).unwrap_err();
            assert!(error.contains("starts with a letter"), "{file}: {error}");
        }
        // A reserved word of the language of the design only.
        assert!(
            name("signal.hdlc", Language::Vhdl)
                .unwrap_err()
                .contains("reserved word in VHDL")
        );
        assert_eq!(
            name("signal.hdlc", Language::SystemVerilog),
            Ok("signal".into())
        );
    }

    #[test]
    fn name_of_another_design_of_the_folder_is_not_free() {
        let dir = tempfile::tempdir().unwrap();
        // A file from before the rule: its module name is not its file name.
        let old = write_design(dir.path(), "old.hdlc", "Uart", Language::Vhdl);

        let error = check_name_is_free(&dir.path().join("uart.hdlc"), "uart").unwrap_err();
        assert!(
            error.contains("'uart'") && error.contains("old.hdlc"),
            "{error}"
        );
        // The file that the save replaces is not another design.
        assert_eq!(check_name_is_free(&old, "uart"), Ok(()));
        assert_eq!(
            check_name_is_free(&dir.path().join("spi.hdlc"), "spi"),
            Ok(())
        );
    }

    const HEADER: &str = "-- Generated by hdl-compose.\n";

    /// A design `name` in `dir` that places the modules `uses`, with its
    /// generated top file.
    fn generated_design(dir: &Path, name: &str, uses: &[&str]) -> PathBuf {
        let mut s = Schematic::new(name, Language::Vhdl);
        for (i, module) in uses.iter().enumerate() {
            s.add_instance(format!("u{i}"), *module).unwrap();
        }
        let path = dir.join(format!("{name}.hdlc"));
        crate::project::save_project(&s, &path).unwrap();
        std::fs::write(dir.join(format!("{name}.vhd")), HEADER).unwrap();
        path
    }

    fn design<'a>(designs: &'a [FolderDesign], name: &str) -> &'a FolderDesign {
        designs.iter().find(|d| d.top_name == name).unwrap()
    }

    /// A design is a module for another design only when it has the same
    /// language, its generated file is at the default place, and it does
    /// not contain that design.
    #[test]
    fn obstacle_gives_the_reason_that_a_design_cannot_be_placed() {
        let dir = tempfile::tempdir().unwrap();
        generated_design(dir.path(), "leaf", &["fifo"]);
        generated_design(dir.path(), "mid", &["leaf", "fifo"]);
        generated_design(dir.path(), "root", &["mid"]);
        write_design(dir.path(), "draft.hdlc", "draft", Language::Vhdl);
        write_design(dir.path(), "sv.hdlc", "sv", Language::SystemVerilog);
        std::fs::write(dir.path().join("sv.sv"), "// Generated by hdl-compose.\n").unwrap();
        let designs = folder_designs(dir.path());
        let obstacle = |child: &str, parent: &str| {
            design(&designs, child).obstacle(&designs, parent, &Language::Vhdl)
        };

        assert_eq!(obstacle("leaf", "mid"), None);
        assert_eq!(obstacle("leaf", "root"), None);
        assert_eq!(obstacle("mid", "leaf"), Some("contains this design"));
        assert_eq!(obstacle("root", "leaf"), Some("contains this design"));
        assert_eq!(obstacle("draft", "root"), Some("not generated"));
        assert_eq!(obstacle("sv", "root"), Some("other language"));
        assert!(design(&designs, "sv").generated);
    }

    /// Two designs that place each other in their files do not make the
    /// search run without end.
    #[test]
    fn designs_that_place_each_other_end_the_search() {
        let dir = tempfile::tempdir().unwrap();
        generated_design(dir.path(), "a", &["b"]);
        generated_design(dir.path(), "b", &["a"]);
        generated_design(dir.path(), "c", &[]);
        let designs = folder_designs(dir.path());

        assert_eq!(
            design(&designs, "a").obstacle(&designs, "c", &Language::Vhdl),
            None
        );
        assert_eq!(
            design(&designs, "a").obstacle(&designs, "b", &Language::Vhdl),
            Some("contains this design")
        );
    }

    /// A parent gets the generated top file, the generated file of each
    /// group that is on the disk, and the library sources of the child.
    #[test]
    fn sources_are_the_generated_files_and_the_library_of_the_design() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = Schematic::new("child", Language::Vhdl);
        s.library_paths = vec![dir.path().join("rtl/fifo.vhd")];
        for name in ["made", "later", "by_hand"] {
            s.groups.push(crate::types::Group {
                name: name.into(),
                members: Vec::new(),
                parent: None,
                collapsed: false,
                position: (0.0, 0.0),
            });
        }
        let path = dir.path().join("child.hdlc");
        crate::project::save_project(&s, &path).unwrap();
        std::fs::write(dir.path().join("child.vhd"), HEADER).unwrap();
        std::fs::write(dir.path().join("made.vhd"), HEADER).unwrap();
        // The group `later` came after the last Generate: it has no file.
        // `by_hand.vhd` is a source of the user with the name of a group.
        std::fs::write(dir.path().join("by_hand.vhd"), "entity by_hand is end;").unwrap();

        let designs = folder_designs(dir.path());
        assert_eq!(
            designs[0].sources(),
            [
                dir.path().join("child.vhd"),
                dir.path().join("made.vhd"),
                dir.path().join("rtl/fifo.vhd"),
            ]
        );
    }
}
