pub mod board;
pub mod codegen;
pub mod floorplan;
pub mod groups;
pub mod gui;
pub mod netcheck;
pub mod nets;
pub mod project;
pub mod reserved;
pub mod routing;
pub mod schematic;
pub mod toolchain;
pub mod types;
pub mod verilog;
pub mod vhdl;

use std::path::Path;
use types::ModuleDef;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("unsupported file extension: {0}")]
    UnsupportedExtension(String),

    #[error("VHDL parse error: {0}")]
    VhdlParse(String),

    #[error("Verilog parse error: {0}")]
    VerilogParse(String),
}

pub fn parse_file(path: &Path) -> Result<Vec<ModuleDef>, ParseError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "vhd" | "vhdl" => vhdl::parse_vhdl(path),
        "v" | "sv" => verilog::parse_verilog(path),
        other => Err(ParseError::UnsupportedExtension(other.to_string())),
    }
}

/// Replace the file at `path` with `contents` in one step. The function
/// writes a temporary file in the same directory and then renames it onto
/// `path`. A failure or a stop of the program during the write leaves the
/// old file complete. `std::fs::write` empties the file before it writes,
/// so a failure there can leave no copy of the data.
///
/// Use this for a file that is the only copy of the work of the user. A
/// file that `hdlc` can generate again does not need it.
pub(crate) fn replace_file(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    // Write through a symbolic link, as `std::fs::write` does.
    let target = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let name = target.file_name().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "path has no file name")
    })?;
    // A rename can replace a file that has no write permission.
    // `std::fs::write` refuses such a file; do the same.
    let old = std::fs::metadata(&target).ok();
    if old.as_ref().is_some_and(|m| m.permissions().readonly()) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "the file is read-only",
        ));
    }
    let mut tmp_name = std::ffi::OsString::from(".");
    tmp_name.push(name);
    tmp_name.push(format!(".{}.tmp", std::process::id()));
    let tmp = target.with_file_name(tmp_name);

    let write = || {
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(contents)?;
        // The new file gets the permissions of the file that it replaces.
        if let Some(old) = &old {
            file.set_permissions(old.permissions())?;
        }
        // The data must be on the disk before the rename.
        file.sync_all()?;
        std::fs::rename(&tmp, &target)
    };
    let result = write();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}
