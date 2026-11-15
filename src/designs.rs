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
            Some(FolderDesign {
                path,
                top_name: schematic.top_name,
                language: schematic.language,
            })
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

        assert_eq!(
            folder_designs(dir.path()),
            [
                FolderDesign {
                    path: a,
                    top_name: "a".into(),
                    language: Language::Vhdl
                },
                FolderDesign {
                    path: b,
                    top_name: "b".into(),
                    language: Language::SystemVerilog
                },
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
}
