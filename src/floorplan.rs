//! Chip view data: where nextpnr placed the design on the device.
//!
//! nextpnr writes three files during `build`:
//! - `pnr.json` (`--write`): each cell has a `NEXTPNR_BEL` attribute.
//! - `placed.svg` (`--placed-svg`): one rectangle for each bel of the device.
//! - `report.json` (`--report`): utilization and maximum clock frequency.
//!
//! This module merges them into one small JSON file that the GUI draws.
//! The link between a cell and the drawing is the tile: a bel `X3/Y26/...`
//! is in tile (3, 26), and the SVG puts that tile at `TILE_PITCH * (3, 26)`.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

/// Size of one device tile in nextpnr SVG units.
pub const TILE_PITCH: f32 = 50.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellKind {
    Logic,
    Register,
    Io,
    Dsp,
    Memory,
    Clock,
    Other,
}

impl CellKind {
    /// Classify a nextpnr cell type (`TRELLIS_FF`, `ICESTORM_LC`, `DCCA`).
    /// The order matters: `SB_GB_IO` is a clock buffer, not an I/O cell.
    pub fn of(cell_type: &str) -> Self {
        let t = cell_type.to_ascii_uppercase();
        let has = |keys: &[&str]| keys.iter().any(|k| t.contains(k));
        if has(&["MULT", "ALU54", "DSP", "MAC16"]) {
            Self::Dsp
        } else if has(&["RAM", "DP16K", "PDPW"]) {
            Self::Memory
        } else if has(&["PLL", "DCC", "DCS", "OSC", "CLKDIV", "ECLK", "_GB", "BUFG"]) {
            Self::Clock
        } else if has(&["IO"]) {
            Self::Io
        } else if has(&["FF"]) {
            Self::Register
        } else if has(&["COMB", "LUT", "_LC", "SLICE", "ALU", "CCU2"]) {
            Self::Logic
        } else {
            Self::Other
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlacedCell {
    pub name: String,
    pub cell_type: String,
    pub kind: CellKind,
    /// Tile column and row.
    pub x: u32,
    pub y: u32,
    /// Bel name inside the tile (`SLICEA.K0`).
    pub bel: String,
    /// Hierarchy path of the cell: each segment of the name before the
    /// last. A schematic instance owns the cell when its name is a segment.
    pub path: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    pub used: u64,
    pub available: u64,
}

/// One step of a timing path: a delay inside a cell, or a net between two
/// cells.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hop {
    /// nextpnr step type: `clk-to-q`, `routing`, `logic`, `setup`, `source`.
    pub kind: String,
    pub delay_ns: f32,
    pub from_cell: String,
    /// Tile column and row of `from_cell`.
    pub from: (u32, u32),
    pub to_cell: String,
    pub to: (u32, u32),
    /// Net name. Only a `routing` hop has one.
    pub net: Option<String>,
    /// HDL source location of the net (`top.v:8.13-8.14`).
    pub source: Option<String>,
}

/// The slowest path between two clock events, from the nextpnr report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CriticalPath {
    /// Launch event (`posedge clk`, or `<async>` for an input).
    pub from: String,
    /// Capture event.
    pub to: String,
    /// Sum of the hop delays.
    pub delay_ns: f32,
    /// Clock period of the constraint. Present only when one clock both
    /// launches and captures the path; other paths have no period.
    pub budget_ns: Option<f32>,
    pub hops: Vec<Hop>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Floorplan {
    pub tile_pitch: f32,
    pub width: f32,
    pub height: f32,
    /// Outlines of the free bels: `x, y, w, h` for each rectangle.
    pub free: Vec<f32>,
    /// Outlines of the bels the design uses, same layout as `free`.
    pub used: Vec<f32>,
    pub cells: Vec<PlacedCell>,
    /// Bel type to usage, from the nextpnr report.
    pub utilization: BTreeMap<String, Usage>,
    /// Clock net to achieved frequency in MHz.
    pub fmax: BTreeMap<String, f32>,
    /// Slowest path for each pair of clock events, in report order.
    #[serde(default)]
    pub critical_paths: Vec<CriticalPath>,
}

/// Split a bel name into tile and local name. nextpnr-ecp5 and
/// nextpnr-ice40 write `X3/Y26/SLICEA.K0`; himbaechel writes `X3Y26/LUT0`.
pub fn parse_bel(bel: &str) -> Option<(u32, u32, &str)> {
    let rest = bel.strip_prefix('X')?;
    let (x, rest) = split_number(rest)?;
    let rest = rest.strip_prefix('/').unwrap_or(rest).strip_prefix('Y')?;
    let (y, rest) = split_number(rest)?;
    Some((x, y, rest.strip_prefix('/')?))
}

fn split_number(s: &str) -> Option<(u32, &str)> {
    let end = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    Some((s[..end].parse().ok()?, &s[end..]))
}

/// Read the cells that have a placement from nextpnr's `--write` JSON.
pub fn parse_cells(pnr_json: &str) -> Result<Vec<PlacedCell>, String> {
    let root: serde_json::Value =
        serde_json::from_str(pnr_json).map_err(|e| format!("pnr.json: {e}"))?;
    let mut cells = Vec::new();
    let modules = root["modules"].as_object().into_iter().flatten();
    for (name, cell) in modules.flat_map(|(_, m)| m["cells"].as_object().into_iter().flatten()) {
        let Some((x, y, bel)) = cell["attributes"]["NEXTPNR_BEL"]
            .as_str()
            .and_then(parse_bel)
        else {
            continue;
        };
        let cell_type = cell["type"].as_str().unwrap_or_default();
        // Names that start with `$` are tool-made and have no owner.
        let mut path: Vec<String> = if name.starts_with('$') {
            Vec::new()
        } else {
            name.split('.').map(str::to_owned).collect()
        };
        path.pop();
        cells.push(PlacedCell {
            name: name.clone(),
            cell_type: cell_type.to_owned(),
            kind: CellKind::of(cell_type),
            x,
            y,
            bel: bel.to_owned(),
            path,
        });
    }
    cells.sort_by(|a, b| (a.y, a.x, &a.bel).cmp(&(b.y, b.x, &b.bel)));
    Ok(cells)
}

fn parse_hop(hop: &serde_json::Value) -> Option<Hop> {
    let end = |key: &str| {
        let loc = &hop[key]["loc"];
        let tile = (loc[0].as_u64()? as u32, loc[1].as_u64()? as u32);
        Some((hop[key]["cell"].as_str()?.to_owned(), tile))
    };
    let (from_cell, from) = end("from")?;
    let (to_cell, to) = end("to")?;
    Some(Hop {
        kind: hop["type"].as_str()?.to_owned(),
        delay_ns: hop["delay"].as_f64()? as f32,
        from_cell,
        from,
        to_cell,
        to,
        net: hop["net"].as_str().map(str::to_owned),
        source: hop["sources"][0].as_str().map(str::to_owned),
    })
}

/// Read `critical_paths` from the nextpnr report. A hop that does not have
/// the expected fields is left out; the path keeps its other hops.
fn parse_critical_paths(report: &serde_json::Value) -> Vec<CriticalPath> {
    let paths = report["critical_paths"].as_array().into_iter().flatten();
    paths
        .map(|p| {
            let from = p["from"].as_str().unwrap_or_default().to_owned();
            let to = p["to"].as_str().unwrap_or_default().to_owned();
            let hops: Vec<Hop> = p["path"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(parse_hop)
                .collect();
            // `from` is `posedge <clock net>`; the constraint is in MHz.
            let clock = from.split_once(' ').map_or("", |(_, net)| net);
            let budget_ns = report["fmax"][clock]["constraint"]
                .as_f64()
                .filter(|mhz| from == to && *mhz > 0.0)
                .map(|mhz| (1000.0 / mhz) as f32);
            CriticalPath {
                delay_ns: hops.iter().map(|h| h.delay_ns).sum(),
                from,
                to,
                budget_ns,
                hops,
            }
        })
        .collect()
}

fn attr<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let start = line.find(&format!(" {key}=\""))? + key.len() + 3;
    let len = line[start..].find('"')?;
    Some(&line[start..start + len])
}

/// One `<rect>` line of the SVG as `([x, y, w, h], used)`. nextpnr writes
/// negative sizes for bels it draws right to left; the result has the
/// top-left corner and a positive size.
fn parse_rect(line: &str) -> Option<([f32; 4], bool)> {
    let num = |key| attr(line, key)?.parse::<f32>().ok();
    let (x, y, w, h) = (num("x")?, num("y")?, num("width")?, num("height")?);
    let rect = [x.min(x + w), y.min(y + h), w.abs(), h.abs()];
    // Two decimals keep the output file small.
    let rect = rect.map(|v| (v * 100.0).round() / 100.0);
    Some((rect, attr(line, "fill")? != "none"))
}

fn tile_of(rect: &[f32]) -> (u32, u32) {
    ((rect[0] / TILE_PITCH) as u32, (rect[1] / TILE_PITCH) as u32)
}

impl Floorplan {
    /// Merge the three nextpnr outputs. `report_json` is optional because
    /// the drawing does not need it.
    pub fn from_nextpnr(
        pnr_json: &str,
        placed_svg: &str,
        report_json: Option<&str>,
    ) -> Result<Self, String> {
        let mut plan = Floorplan {
            tile_pitch: TILE_PITCH,
            cells: parse_cells(pnr_json)?,
            ..Default::default()
        };
        // The background rectangle has a width of `100%`; `parse_rect`
        // rejects it.
        for (rect, used) in placed_svg.lines().filter_map(parse_rect) {
            plan.width = plan.width.max(rect[0] + rect[2]);
            plan.height = plan.height.max(rect[1] + rect[3]);
            if used {
                plan.used.extend(rect);
            } else {
                plan.free.extend(rect);
            }
        }
        if plan.free.is_empty() && plan.used.is_empty() {
            return Err("placed.svg: no bel rectangles found".into());
        }
        if let Some(text) = report_json {
            let report: serde_json::Value =
                serde_json::from_str(text).map_err(|e| format!("report.json: {e}"))?;
            for (bel_type, u) in report["utilization"].as_object().into_iter().flatten() {
                let usage = Usage {
                    used: u["used"].as_u64().unwrap_or(0),
                    available: u["available"].as_u64().unwrap_or(0),
                };
                plan.utilization.insert(bel_type.clone(), usage);
            }
            for (clock, f) in report["fmax"].as_object().into_iter().flatten() {
                if let Some(mhz) = f["achieved"].as_f64() {
                    plan.fmax.insert(clock.clone(), mhz as f32);
                }
            }
            plan.critical_paths = parse_critical_paths(&report);
        }
        Ok(plan)
    }

    /// Tiles that hold a cell but have no used rectangle in the drawing.
    /// An empty result shows that the tile rule (`TILE_PITCH`) agrees with
    /// this nextpnr version and device.
    pub fn unmatched_tiles(&self) -> Vec<(u32, u32)> {
        let (rects, _) = self.used.as_chunks::<4>();
        let drawn: BTreeSet<_> = rects.iter().map(|rect| tile_of(rect)).collect();
        let placed: BTreeSet<_> = self.cells.iter().map(|c| (c.x, c.y)).collect();
        placed.difference(&drawn).copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const PNR: &str = r#"{"modules":{"top":{"cells":{
        "u_blink.c_TRELLIS_FF_Q":{"type":"TRELLIS_FF","attributes":{"NEXTPNR_BEL":"X3/Y26/SLICED.FF0"}},
        "grp.u_mul.p_MULT":{"type":"MULT18X18D","attributes":{"NEXTPNR_BEL":"X18/Y12/MULT18_0"}},
        "$gbuf$clk":{"type":"DCCA","attributes":{"NEXTPNR_BEL":"X31/Y0/TDCC0"}},
        "led[0]$tr_io":{"type":"TRELLIS_IO","attributes":{"NEXTPNR_BEL":"X0/Y14/PIOB"}},
        "unplaced":{"type":"TRELLIS_COMB","attributes":{}}
    }}}}"#;

    // Lines copied from a nextpnr-0.10 ECP5 run. The last one is the I/O
    // bel at X0/Y14, which nextpnr writes with a negative width.
    const SVG: &str = r##"<svg viewBox="0 0 3692.5 2597.5" xmlns="http://www.w3.org/2000/svg">
<rect x="0" y="0" width="100%" height="100%" stroke="#fff" fill="#fff"/>
<rect x="75.500000" y="75.500000" width="19.500000" height="19.500000" stroke="#808080" fill="none"/>
<rect x="196.100000" y="1335.600000" width="0.300000" height="0.700000" stroke="#FF3030" fill="#FF8080"/>
<rect x="917.499939" y="602.500000" width="3.999996" height="9.500000" stroke="#FF3030" fill="#FF8080"/>
<rect x="1575.500000" y="7.000000" width="1.000023" height="2.000000" stroke="#FF3030" fill="#FF8080"/>
<rect x="12.000000" y="707.500000" width="-9.500000" height="3.999996" stroke="#FF3030" fill="#FF8080"/>
</svg>"##;

    // The path hops are copied from a nextpnr-0.10 report.
    const REPORT: &str = r#"{"fmax":{"clk":{"achieved":122.25,"constraint":12}},
        "utilization":{"TRELLIS_FF":{"available":24288,"used":42}},
        "critical_paths":[
          {"from":"posedge clk","to":"posedge clk","path":[
            {"delay":0.5,"type":"clk-to-q",
             "from":{"cell":"a_FF","loc":[17,12],"port":"Q"},
             "to":{"cell":"a_FF","loc":[17,12],"port":"Q"}},
            {"delay":1.25,"type":"routing","net":"a[4]","sources":["top.v:8.13-8.14"],
             "from":{"cell":"a_FF","loc":[17,12],"port":"Q"},
             "to":{"cell":"u_mul.b_MULT","loc":[17,13],"port":"A4"}},
            {"delay":4.0,"type":"logic","from":{"cell":"u_mul.b_MULT"}}
          ]},
          {"from":"<async>","to":"posedge clk","path":[]}
        ]}"#;

    #[test]
    fn merges_the_three_files() {
        let plan = Floorplan::from_nextpnr(PNR, SVG, Some(REPORT)).unwrap();
        assert_eq!(plan.cells.len(), 4, "the unplaced cell is not in the plan");
        assert_eq!(plan.free.len(), 4);
        assert_eq!(plan.used.len(), 16);
        assert_eq!(plan.unmatched_tiles(), vec![]);
        assert_eq!(
            plan.utilization["TRELLIS_FF"],
            Usage {
                used: 42,
                available: 24288
            }
        );
        assert_eq!(plan.fmax["clk"], 122.25);

        let ff = plan.cells.iter().find(|c| c.bel == "SLICED.FF0").unwrap();
        assert_eq!((ff.x, ff.y, ff.kind), (3, 26, CellKind::Register));
        assert_eq!(ff.path, ["u_blink"]);
        let mul = plan.cells.iter().find(|c| c.kind == CellKind::Dsp).unwrap();
        assert_eq!(mul.path, ["grp", "u_mul"]);
        let buf = plan
            .cells
            .iter()
            .find(|c| c.kind == CellKind::Clock)
            .unwrap();
        assert!(buf.path.is_empty(), "a `$` name has no owner");
    }

    #[test]
    fn critical_paths_keep_tiles_delay_and_source() {
        let plan = Floorplan::from_nextpnr(PNR, SVG, Some(REPORT)).unwrap();
        let [clocked, input] = &plan.critical_paths[..] else {
            panic!("two paths expected");
        };
        assert_eq!(clocked.hops.len(), 2, "the hop with no tile is left out");
        assert_eq!(clocked.delay_ns, 1.75);
        // 12 MHz is a period of 83.33 ns.
        assert!((clocked.budget_ns.unwrap() - 83.333).abs() < 0.01);
        let net = &clocked.hops[1];
        assert_eq!((net.from, net.to), ((17, 12), (17, 13)));
        assert_eq!(net.to_cell, "u_mul.b_MULT");
        assert_eq!(net.net.as_deref(), Some("a[4]"));
        assert_eq!(net.source.as_deref(), Some("top.v:8.13-8.14"));
        assert_eq!(clocked.hops[0].net, None);
        assert_eq!(input.budget_ns, None, "an input path has no clock period");
    }

    #[test]
    fn an_old_chip_view_file_loads_without_paths() {
        let mut value = serde_json::to_value(Floorplan::default()).unwrap();
        value.as_object_mut().unwrap().remove("critical_paths");
        let plan: Floorplan = serde_json::from_value(value).unwrap();
        assert!(plan.critical_paths.is_empty());
    }

    #[test]
    fn negative_width_rect_gets_a_top_left_corner() {
        let plan = Floorplan::from_nextpnr(PNR, SVG, None).unwrap();
        let io = &plan.used[12..16];
        assert_eq!(io, [2.5, 707.5, 9.5, 4.0]);
        assert_eq!(tile_of(io), (0, 14));
    }

    #[test]
    fn a_cell_tile_with_no_rect_is_reported() {
        let svg = SVG.replace(r#"x="1575.500000""#, r#"x="2575.500000""#);
        let plan = Floorplan::from_nextpnr(PNR, &svg, None).unwrap();
        assert_eq!(plan.unmatched_tiles(), vec![(31, 0)]);
    }

    #[test]
    fn an_svg_without_rects_is_an_error() {
        assert!(Floorplan::from_nextpnr(PNR, "<svg></svg>", None).is_err());
    }

    #[test]
    fn kinds() {
        for (cell_type, kind) in [
            ("TRELLIS_COMB", CellKind::Logic),
            ("ICESTORM_LC", CellKind::Logic),
            ("LUT4", CellKind::Logic),
            ("TRELLIS_FF", CellKind::Register),
            ("DFFRE", CellKind::Register),
            ("TRELLIS_IO", CellKind::Io),
            ("SB_IO", CellKind::Io),
            ("IOLOGIC", CellKind::Io),
            ("SB_GB_IO", CellKind::Clock),
            ("EHXPLLL", CellKind::Clock),
            ("rPLL", CellKind::Clock),
            ("DCCA", CellKind::Clock),
            ("MULT18X18D", CellKind::Dsp),
            ("ALU54B", CellKind::Dsp),
            ("ICESTORM_DSP", CellKind::Dsp),
            ("DP16KD", CellKind::Memory),
            ("ICESTORM_RAM", CellKind::Memory),
            ("TRELLIS_RAMW", CellKind::Memory),
            ("GSR", CellKind::Other),
        ] {
            assert_eq!(CellKind::of(cell_type), kind, "{cell_type}");
        }
    }

    proptest! {
        #[test]
        fn bel_name_round_trip(x in 0u32..500, y in 0u32..500, name in "[A-Za-z][A-Za-z0-9_.]{0,12}") {
            let slash = format!("X{x}/Y{y}/{name}");
            prop_assert_eq!(parse_bel(&slash), Some((x, y, name.as_str())));
            let himbaechel = format!("X{x}Y{y}/{name}");
            prop_assert_eq!(parse_bel(&himbaechel), Some((x, y, name.as_str())));
        }

        /// A rect covers the same area for each sign of its size.
        #[test]
        fn rect_sign_does_not_move_the_area(
            x in 0u32..4000, y in 0u32..3000, w in 1u32..60, h in 1u32..60,
            flip_w: bool, flip_h: bool,
        ) {
            let (x, y, w, h) = (x as f32, y as f32, w as f32, h as f32);
            let (sx, sw) = if flip_w { (x + w, -w) } else { (x, w) };
            let (sy, sh) = if flip_h { (y + h, -h) } else { (y, h) };
            let line = format!(
                r##"<rect x="{sx:.6}" y="{sy:.6}" width="{sw:.6}" height="{sh:.6}" stroke="#808080" fill="none"/>"##
            );
            prop_assert_eq!(parse_rect(&line), Some(([x, y, w, h], false)));
        }
    }
}
