pub mod codegen;
pub mod groups;
pub mod gui;
pub mod nets;
pub mod project;
pub mod routing;
pub mod schematic;
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
