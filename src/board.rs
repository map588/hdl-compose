//! Board definition files: a small JSON description of an FPGA board
//! (family, device, package, constraints file, tool flags) that `build` and
//! `flash` consume. Referenced by path — boards live wherever the user keeps
//! them, typically next to the board's own repo.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::toolchain::FpgaFamily;
use crate::types::{PortType, RangeExpr, Schematic};

#[derive(Deserialize)]
pub struct Board {
    pub name: String,
    pub family: FpgaFamily,
    pub device: String,
    pub package: String,
    /// Constraints file (.lpf/.pcf/.cst); relative paths resolve against the
    /// board file's directory.
    pub constraints: PathBuf,
    #[serde(default)]
    pub pack_args: Vec<String>,
    #[serde(default)]
    pub prog_args: Vec<String>,
}

impl Board {
    pub fn load(path: &Path) -> Result<Board, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read board file {}: {e}", path.display()))?;
        let mut board: Board = serde_json::from_str(&text)
            .map_err(|e| format!("invalid board file {}: {e}", path.display()))?;
        if board.constraints.is_relative() {
            let dir = path.parent().unwrap_or_else(|| Path::new("."));
            board.constraints = dir.join(&board.constraints);
        }
        if !board.constraints.exists() {
            return Err(format!(
                "board '{}': constraints file {} not found",
                board.name,
                board.constraints.display()
            ));
        }
        Ok(board)
    }
}

/// Pull the constrained port names out of a constraints file. Line-based, not
/// a grammar: LPF `LOCATE COMP "<name>"`, PCF `set_io <name> <pin>`,
/// CST `IO_LOC "<name>"`.
pub fn constraint_port_names(text: &str, family: FpgaFamily) -> HashSet<String> {
    let mut names = HashSet::new();
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        match family {
            FpgaFamily::Ecp5 => {
                if let Some(rest) = line.strip_prefix("LOCATE COMP")
                    && let Some(name) = quoted(rest)
                {
                    names.insert(name);
                }
            }
            FpgaFamily::Ice40 => {
                // set_io [-flag value ...] <name> <pin> — name is second-to-last
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.first() == Some(&"set_io") && parts.len() >= 3 {
                    names.insert(parts[parts.len() - 2].to_string());
                }
            }
            FpgaFamily::Gowin => {
                if let Some(rest) = line.strip_prefix("IO_LOC")
                    && let Some(name) = quoted(rest)
                {
                    names.insert(name);
                }
            }
        }
    }
    names
}

fn quoted(s: &str) -> Option<String> {
    let start = s.find('"')?;
    let rest = &s[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// Compare the schematic's top-level ports against the board's constrained
/// names. Returns human-readable warnings for ports the board has no pin for;
/// vector ports check each bit (`led[0]`…) unless their bounds are unresolved.
pub fn pin_check(schematic: &Schematic, names: &HashSet<String>) -> Vec<String> {
    let mut warnings = Vec::new();
    for port in &schematic.top_ports {
        let n = &port.name;
        match &port.port_type {
            PortType::StdLogicVector(range) => {
                let (RangeExpr::Literal(high), RangeExpr::Literal(low)) =
                    (&range.high, &range.low)
                else {
                    continue;
                };
                if names.contains(n) {
                    continue;
                }
                let (lo, hi) = (*low.min(high), *low.max(high));
                let missing: Vec<String> = (lo..=hi)
                    .filter(|i| !names.contains(&format!("{n}[{i}]")))
                    .map(|i| format!("{n}[{i}]"))
                    .collect();
                if missing.len() as i64 == hi - lo + 1 {
                    warnings.push(format!("top port '{n}' has no pin on this board"));
                } else if !missing.is_empty() {
                    warnings.push(format!(
                        "top port '{n}' bits without a pin: {}",
                        missing.join(", ")
                    ));
                }
            }
            _ => {
                if !names.contains(n) {
                    warnings.push(format!("top port '{n}' has no pin on this board"));
                }
            }
        }
    }
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Direction, Language, PortDef, Range, RangeDir};

    fn vector(name: &str, high: i64, low: i64) -> PortDef {
        PortDef {
            name: name.to_string(),
            direction: Direction::Out,
            port_type: PortType::StdLogicVector(Range {
                high: RangeExpr::Literal(high),
                low: RangeExpr::Literal(low),
                dir: RangeDir::Downto,
            }),
            bundle: None,
        }
    }

    fn scalar(name: &str) -> PortDef {
        PortDef {
            name: name.to_string(),
            direction: Direction::In,
            port_type: PortType::StdLogic,
            bundle: None,
        }
    }

    const LPF: &str = r#"
SYSCONFIG CONFIG_IOVOLTAGE=3.3;
# Clock
LOCATE COMP "clk" SITE "M1";
IOBUF  PORT "clk" IO_TYPE=LVCMOS33;
LOCATE COMP "led[0]" SITE "E13";
LOCATE COMP "led[1]" SITE "D14";
"#;

    #[test]
    fn lpf_names_extracted() {
        let names = constraint_port_names(LPF, FpgaFamily::Ecp5);
        assert_eq!(
            names,
            HashSet::from(["clk".to_string(), "led[0]".to_string(), "led[1]".to_string()])
        );
    }

    #[test]
    fn pcf_names_extracted() {
        let names = constraint_port_names(
            "# comment\nset_io clk 35\nset_io -pullup yes btn 10\n",
            FpgaFamily::Ice40,
        );
        assert_eq!(names, HashSet::from(["clk".to_string(), "btn".to_string()]));
    }

    #[test]
    fn pin_check_flags_missing_and_partial() {
        let names = constraint_port_names(LPF, FpgaFamily::Ecp5);
        let mut s = Schematic::new("t", Language::Vhdl);
        s.top_ports = vec![scalar("clk"), scalar("rst"), vector("led", 2, 0)];
        let warnings = pin_check(&s, &names);
        assert_eq!(warnings.len(), 2);
        assert!(warnings[0].contains("'rst' has no pin"));
        assert!(warnings[1].contains("led[2]"));
        assert!(!warnings[1].contains("led[0]"));
    }
}
