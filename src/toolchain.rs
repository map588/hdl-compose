//! oss-cad-suite integration: stage a project's HDL into a scratch directory
//! and drive external tools (ghdl, verilator, yosys, iverilog, gtkwave/surfer)
//! against it. Tools are resolved from PATH only — the user is expected to
//! have activated their oss-cad-suite environment.

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::codegen::{self, GeneratedDesign};
use crate::types::{Direction, Language, ModuleDef, PortType, Schematic};

const PATH_HINT: &str =
    "not found on PATH — activate oss-cad-suite (source <oss-cad-suite>/environment) first";

/// True when `name` can be spawned. Exit status is irrelevant; only "does the
/// binary exist on PATH" matters.
pub fn tool_on_path(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

pub fn require_tools(names: &[&str]) -> Result<(), String> {
    for n in names {
        if !tool_on_path(n) {
            return Err(format!("{n} {PATH_HINT}"));
        }
    }
    Ok(())
}

/// Run a tool with inherited stdio (its messages reach the user verbatim).
/// Returns whether it exited successfully.
fn run_tool<I, S>(name: &str, args: I, cwd: &Path) -> Result<bool, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let status = Command::new(name)
        .args(args)
        .current_dir(cwd)
        .status()
        .map_err(|e| format!("failed to run {name}: {e}"))?;
    Ok(status.success())
}

/// A project's complete HDL staged into a scratch directory: library sources
/// copied in, generated files written out, everything addressed by relative
/// filename so tool command lines stay simple.
pub struct StagedBuild {
    pub dir: PathBuf,
    /// Filenames (relative to `dir`) in analysis order: library sources with
    /// dependencies first, then generated group files, then the top file.
    pub sources: Vec<String>,
    pub top_name: String,
    pub language: Language,
}

/// Order library source files so that every module is analyzed after its
/// in-library dependencies (VHDL cares; harmless elsewhere). Cycles fall back
/// to original order.
pub fn dependency_ordered_sources(library: &[ModuleDef]) -> Vec<PathBuf> {
    let name_idx: HashMap<&str, usize> = library
        .iter()
        .enumerate()
        .map(|(i, m)| (m.name.as_str(), i))
        .collect();

    let mut emitted = vec![false; library.len()];
    let mut order: Vec<usize> = Vec::new();
    while order.len() < library.len() {
        let mut progressed = false;
        for (i, m) in library.iter().enumerate() {
            if emitted[i] {
                continue;
            }
            let ready = m.dependencies.iter().all(|d| {
                name_idx
                    .get(d.as_str())
                    .is_none_or(|&j| emitted[j] || j == i)
            });
            if ready {
                emitted[i] = true;
                order.push(i);
                progressed = true;
            }
        }
        if !progressed {
            for (i, e) in emitted.iter_mut().enumerate() {
                if !*e {
                    *e = true;
                    order.push(i);
                }
            }
        }
    }

    // Module order → file order: a file is placed where its LAST module lands,
    // so every module in it has its dependencies analyzed beforehand.
    let mut files: Vec<PathBuf> = Vec::new();
    for &i in order.iter().rev() {
        let p = &library[i].source_path;
        if !files.contains(p) {
            files.push(p.clone());
        }
    }
    files.reverse();
    files
}

/// Codegen the project and lay everything out in a scratch dir.
pub fn stage(
    schematic: &Schematic,
    library: &[ModuleDef],
    design: &GeneratedDesign,
) -> Result<StagedBuild, String> {
    let dir = std::env::temp_dir().join(format!("hdl-compose-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;

    let mut sources = Vec::new();
    for src in dependency_ordered_sources(library) {
        let mut fname = src
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| format!("bad library path: {}", src.display()))?
            .to_string();
        // Two different library files with the same basename: disambiguate.
        while sources.contains(&fname) {
            fname = format!("_{fname}");
        }
        std::fs::copy(&src, dir.join(&fname))
            .map_err(|e| format!("cannot copy {}: {e}", src.display()))?;
        sources.push(fname);
    }
    for (fname, code) in design
        .files
        .iter()
        .chain(std::iter::once(&(design.top_filename.clone(), design.top_code.clone())))
    {
        std::fs::write(dir.join(fname), code)
            .map_err(|e| format!("cannot write {fname}: {e}"))?;
        sources.push(fname.clone());
    }

    Ok(StagedBuild {
        dir,
        sources,
        top_name: schematic.top_name.clone(),
        language: schematic.language.clone(),
    })
}

/// Elaborate the staged design with a real frontend: ghdl for VHDL,
/// `verilator --lint-only` for SystemVerilog. Returns whether it passed.
pub fn run_check(staged: &StagedBuild) -> Result<bool, String> {
    match staged.language {
        Language::Vhdl => {
            require_tools(&["ghdl"])?;
            let mut args = vec!["-a".to_string(), "--std=08".to_string()];
            args.extend(staged.sources.iter().cloned());
            if !run_tool("ghdl", &args, &staged.dir)? {
                return Ok(false);
            }
            run_tool(
                "ghdl",
                ["-e", "--std=08", &staged.top_name],
                &staged.dir,
            )
        }
        Language::SystemVerilog => {
            require_tools(&["verilator"])?;
            let mut args = vec![
                "--lint-only".to_string(),
                "-sv".to_string(),
                "--top-module".to_string(),
                staged.top_name.clone(),
            ];
            args.extend(staged.sources.iter().cloned());
            run_tool("verilator", &args, &staged.dir)
        }
    }
}

/// The ghdl-yosys-plugin's embedded ghdl does not resolve its std/ieee
/// library path the way the standalone binary does; ask the standalone ghdl
/// where its libraries live so the plugin can be pointed at them.
fn ghdl_prefix() -> Option<String> {
    let out = Command::new("ghdl").arg("--disp-config").output().ok()?;
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .find_map(|l| l.strip_prefix("library directory:").map(|v| v.trim().to_string()))
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("GHDL_PREFIX").ok())
}

/// Generic yosys synthesis of the staged design; prints yosys output
/// (warnings + `stat`) directly. Proves synthesizability, nothing more.
pub fn run_synth(staged: &StagedBuild) -> Result<bool, String> {
    require_tools(&["yosys"])?;
    match staged.language {
        Language::Vhdl => {
            let prefix = ghdl_prefix()
                .map(|p| format!("--PREFIX={p} "))
                .unwrap_or_default();
            let script = format!(
                "ghdl --std=08 {prefix}{} -e {}; synth; stat",
                staged.sources.join(" "),
                staged.top_name
            );
            run_tool("yosys", ["-m", "ghdl", "-p", &script], &staged.dir)
        }
        Language::SystemVerilog => {
            let script = format!(
                "read_verilog -sv {}; hierarchy -top {}; synth; stat",
                staged.sources.join(" "),
                staged.top_name
            );
            run_tool("yosys", ["-p", &script], &staged.dir)
        }
    }
}

/// Simulate `<top>_tb` (generated on demand next to the project) with ghdl or
/// iverilog. The wave file lands next to the project. Returns the wave path
/// on success so the caller can open a viewer.
pub fn run_sim(
    staged: &StagedBuild,
    schematic: &Schematic,
    project_dir: &Path,
) -> Result<Option<PathBuf>, String> {
    let ext = match staged.language {
        Language::Vhdl => "vhd",
        Language::SystemVerilog => "sv",
    };
    let tb_name = format!("{}_tb", staged.top_name);
    let tb_file = format!("{tb_name}.{ext}");
    let tb_path = project_dir.join(&tb_file);
    if !tb_path.exists() {
        std::fs::write(&tb_path, testbench_skeleton(schematic))
            .map_err(|e| format!("cannot write {}: {e}", tb_path.display()))?;
        println!(
            "Generated testbench skeleton {} — edit it to drive your design.",
            tb_path.display()
        );
    }
    std::fs::copy(&tb_path, staged.dir.join(&tb_file))
        .map_err(|e| format!("cannot copy {}: {e}", tb_path.display()))?;

    match staged.language {
        Language::Vhdl => {
            require_tools(&["ghdl"])?;
            let wave = project_dir.join(format!("{tb_name}.ghw"));
            let mut args = vec!["-a".to_string(), "--std=08".to_string()];
            args.extend(staged.sources.iter().cloned());
            args.push(tb_file.clone());
            if !run_tool("ghdl", &args, &staged.dir)? {
                return Ok(None);
            }
            if !run_tool("ghdl", ["-e", "--std=08", &tb_name], &staged.dir)? {
                return Ok(None);
            }
            let wave_arg = format!("--wave={}", wave.display());
            // Clock generators never stop on their own; bound the run.
            if !run_tool(
                "ghdl",
                ["-r", "--std=08", &tb_name, &wave_arg, "--stop-time=1us"],
                &staged.dir,
            )? {
                return Ok(None);
            }
            Ok(Some(wave))
        }
        Language::SystemVerilog => {
            require_tools(&["iverilog", "vvp"])?;
            let mut args = vec![
                "-g2012".to_string(),
                "-o".to_string(),
                "sim.vvp".to_string(),
                "-s".to_string(),
                tb_name.clone(),
            ];
            args.extend(staged.sources.iter().cloned());
            args.push(tb_file.clone());
            if !run_tool("iverilog", &args, &staged.dir)? {
                return Ok(None);
            }
            let vvp_path = staged.dir.join("sim.vvp");
            // cwd = project dir so the tb's relative $dumpfile lands there.
            if !run_tool("vvp", [vvp_path.as_os_str()], project_dir)? {
                return Ok(None);
            }
            Ok(Some(project_dir.join(format!("{tb_name}.vcd"))))
        }
    }
}

/// Open a wave file in surfer (preferred) or gtkwave, detached.
pub fn open_wave_viewer(wave: &Path) -> Result<(), String> {
    let viewer = if tool_on_path("surfer") {
        "surfer"
    } else if tool_on_path("gtkwave") {
        "gtkwave"
    } else {
        return Err(format!("surfer/gtkwave {PATH_HINT}"));
    };
    Command::new(viewer)
        .arg(wave)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("failed to launch {viewer}: {e}"))?;
    println!("Opened {} in {viewer}", wave.display());
    Ok(())
}

#[derive(Clone, Copy)]
pub enum FpgaFamily {
    Ice40,
    Ecp5,
    Gowin,
}

pub struct FpgaScaffold {
    pub makefile: String,
    pub constraints_name: String,
    pub constraints: String,
}

/// Emit a Makefile (yosys → nextpnr → pack → openFPGALoader) plus a
/// constraints placeholder listing the top-level ports. Everything a board
/// needs to change sits in the variables at the top of the Makefile.
pub fn fpga_scaffold(
    schematic: &Schematic,
    library: &[ModuleDef],
    family: FpgaFamily,
    project_file_name: &str,
) -> FpgaScaffold {
    let top = &schematic.top_name;
    let (gen_ext, lang_vars, read_cmd) = match schematic.language {
        Language::Vhdl => (
            "vhd",
            // The ghdl-yosys-plugin's embedded ghdl can't always find its
            // std/ieee libraries; point it at the standalone ghdl's.
            "GHDL_PREFIX ?= $(shell ghdl --disp-config 2>/dev/null | sed -n 's/^library directory: //p')\n\
             GHDL_FLAGS  := $(if $(GHDL_PREFIX),--PREFIX=$(GHDL_PREFIX))\n",
            "yosys -m ghdl -p \"ghdl --std=08 $(GHDL_FLAGS) $(SRCS) -e $(TOP); SYNTH -json $@\""
                .to_string(),
        ),
        Language::SystemVerilog => (
            "sv",
            "",
            "yosys -p \"read_verilog -sv $(SRCS); SYNTH -top $(TOP) -json $@\"".to_string(),
        ),
    };

    let lib_srcs: Vec<String> = dependency_ordered_sources(library)
        .iter()
        .map(|p| p.display().to_string())
        .collect();
    let gen_srcs: Vec<String> = schematic
        .groups
        .iter()
        .map(|g| format!("$(BUILD)/{}.{gen_ext}", g.name))
        .chain(std::iter::once(format!("$(BUILD)/$(TOP).{gen_ext}")))
        .collect();

    let (synth_cmd, device_line, cst_var, pnr, pack, bit) = match family {
        FpgaFamily::Ice40 => (
            "synth_ice40",
            "# ice40 devices: lp1k hx1k hx8k up5k u4k ...\nDEVICE  := up5k\nPACKAGE := sg48",
            ("PCF", "$(TOP).pcf"),
            "$(BUILD)/$(TOP).asc: $(BUILD)/$(TOP).json $(PCF)\n\
             \tnextpnr-ice40 --$(DEVICE) --package $(PACKAGE) --json $< --pcf $(PCF) --asc $@",
            "$(BUILD)/$(TOP).bin: $(BUILD)/$(TOP).asc\n\ticepack $< $@",
            "$(BUILD)/$(TOP).bin",
        ),
        FpgaFamily::Ecp5 => (
            "synth_ecp5",
            "# ecp5 devices: 12k 25k 45k 85k um5g-25k ...\nDEVICE  := 25k\nPACKAGE := CABGA256",
            ("LPF", "$(TOP).lpf"),
            "$(BUILD)/$(TOP).config: $(BUILD)/$(TOP).json $(LPF)\n\
             \tnextpnr-ecp5 --$(DEVICE) --package $(PACKAGE) --json $< --lpf $(LPF) --textcfg $@",
            "$(BUILD)/$(TOP).bit: $(BUILD)/$(TOP).config\n\tecppack $< $@",
            "$(BUILD)/$(TOP).bit",
        ),
        FpgaFamily::Gowin => (
            "synth_gowin",
            "# gowin: DEVICE is the full part number, FAMILY the die family\nDEVICE  := GW1NR-LV9QN88PC6/I5\nFAMILY  := GW1N-9C",
            ("CST", "$(TOP).cst"),
            "$(BUILD)/$(TOP)_pnr.json: $(BUILD)/$(TOP).json $(CST)\n\
             \tnextpnr-himbaechel --device $(DEVICE) --vopt family=$(FAMILY) --vopt cst=$(CST) --json $< --write $@",
            "$(BUILD)/$(TOP).fs: $(BUILD)/$(TOP)_pnr.json\n\tgowin_pack -d $(FAMILY) -o $@ $<",
            "$(BUILD)/$(TOP).fs",
        ),
    };
    let synth_line = read_cmd.replace("SYNTH", synth_cmd);
    let (cst_name, cst_val) = cst_var;

    let makefile = format!(
        "# FPGA build flow for {top} — generated by hdl-compose. Edit freely.\n\
         # Requires oss-cad-suite on PATH: source <oss-cad-suite>/environment\n\
         # Set DEVICE/PACKAGE for your board and fill in the constraints file.\n\
         \n\
         TOP     := {top}\n\
         PROJECT := {project_file_name}\n\
         BUILD   := build\n\
         {device_line}\n\
         {cst_name}     := {cst_val}\n\
         # add board/cable flags, e.g. -b <board>\n\
         PROG    := openFPGALoader\n\
         {lang_vars}\
         \n\
         LIB_SRCS := {lib_srcs}\n\
         GEN_SRCS := {gen_srcs}\n\
         SRCS     := $(LIB_SRCS) $(GEN_SRCS)\n\
         \n\
         all: {bit}\n\
         \n\
         # codegen writes the group files alongside the top file\n\
         $(BUILD)/$(TOP).{gen_ext}: $(PROJECT)\n\
         \t@mkdir -p $(BUILD)\n\
         \thdl-compose codegen $(PROJECT) -o $@\n\
         \n\
         $(BUILD)/$(TOP).json: $(BUILD)/$(TOP).{gen_ext} $(LIB_SRCS)\n\
         \t{synth_line}\n\
         \n\
         {pnr}\n\
         \n\
         {pack}\n\
         \n\
         prog: {bit}\n\
         \t$(PROG) $<\n\
         \n\
         clean:\n\
         \trm -rf $(BUILD)\n\
         \n\
         .PHONY: all prog clean\n",
        lib_srcs = lib_srcs.join(" "),
        gen_srcs = gen_srcs.join(" "),
    );

    let mut constraints = String::new();
    let (header, line): (&str, fn(&str) -> String) = match family {
        FpgaFamily::Ice40 => (
            "# Pin constraints — uncomment and set a pin per top-level port.\n",
            |p| format!("# set_io {p} <pin>\n"),
        ),
        FpgaFamily::Ecp5 => (
            "# Pin constraints — uncomment and set a site per top-level port.\n",
            |p| format!("# LOCATE COMP \"{p}\" SITE \"<site>\";\n# IOBUF PORT \"{p}\" IO_TYPE=LVCMOS33;\n"),
        ),
        FpgaFamily::Gowin => (
            "# Pin constraints — uncomment and set a pin per top-level port.\n",
            |p| format!("# IO_LOC \"{p}\" <pin>;\n"),
        ),
    };
    constraints.push_str(header);
    for p in &schematic.top_ports {
        constraints.push_str(&line(&p.name));
    }

    FpgaScaffold {
        makefile,
        constraints_name: cst_val.replace("$(TOP)", top),
        constraints,
    }
}

fn find_clock(schematic: &Schematic) -> Option<&str> {
    schematic
        .top_ports
        .iter()
        .find(|p| {
            matches!(p.direction, Direction::In)
                && matches!(p.port_type, PortType::StdLogic)
                && {
                    let n = p.name.to_lowercase();
                    n.contains("clk") || n.contains("clock")
                }
        })
        .map(|p| p.name.as_str())
}

fn vhdl_input_init(pt: &PortType) -> &'static str {
    match pt {
        PortType::StdLogic => " := '0'",
        PortType::StdLogicVector(_) => " := (others => '0')",
        _ => "",
    }
}

/// Minimal testbench: signals for every top port, top instantiated, optional
/// clock toggle, empty stimulus block. Meant to be edited by the user.
pub fn testbench_skeleton(schematic: &Schematic) -> String {
    let top = &schematic.top_name;
    let clock = find_clock(schematic);
    match schematic.language {
        Language::Vhdl => {
            let mut s = String::new();
            s.push_str(&format!(
                "-- Testbench skeleton for {top}, generated by hdl-compose. Edit freely.\n\
                 -- `hdl-compose sim` runs it with --stop-time=1us.\n\
                 library ieee;\nuse ieee.std_logic_1164.all;\n\n\
                 entity {top}_tb is\nend entity {top}_tb;\n\n\
                 architecture tb of {top}_tb is\n"
            ));
            for p in &schematic.top_ports {
                let init = if matches!(p.direction, Direction::In) {
                    vhdl_input_init(&p.port_type)
                } else {
                    ""
                };
                s.push_str(&format!(
                    "  signal {} : {}{init};\n",
                    p.name,
                    codegen::port_type_to_vhdl(&p.port_type)
                ));
            }
            s.push_str("begin\n");
            s.push_str(&format!("  uut: entity work.{top}\n    port map (\n"));
            let maps: Vec<String> = schematic
                .top_ports
                .iter()
                .map(|p| format!("      {} => {}", p.name, p.name))
                .collect();
            s.push_str(&maps.join(",\n"));
            s.push_str("\n    );\n");
            if let Some(clk) = clock {
                s.push_str(&format!("\n  {clk} <= not {clk} after 5 ns;\n"));
            }
            s.push_str(
                "\n  stim: process\n  begin\n    -- drive inputs here\n    wait;\n  end process;\n",
            );
            s.push_str("end architecture tb;\n");
            s
        }
        Language::SystemVerilog => {
            let mut s = String::new();
            s.push_str(&format!(
                "// Testbench skeleton for {top}, generated by hdl-compose. Edit freely.\n\
                 `timescale 1ns/1ps\n\nmodule {top}_tb;\n"
            ));
            for p in &schematic.top_ports {
                let init = if matches!(p.direction, Direction::In) {
                    " = '0"
                } else {
                    ""
                };
                s.push_str(&format!(
                    "  {} {}{init};\n",
                    codegen::port_type_to_sv(&p.port_type),
                    p.name
                ));
            }
            s.push_str(&format!("\n  {top} uut (\n"));
            let maps: Vec<String> = schematic
                .top_ports
                .iter()
                .map(|p| format!("    .{}({})", p.name, p.name))
                .collect();
            s.push_str(&maps.join(",\n"));
            s.push_str("\n  );\n");
            if let Some(clk) = clock {
                s.push_str(&format!("\n  always #5 {clk} = ~{clk};\n"));
            }
            s.push_str(&format!(
                "\n  initial begin\n    $dumpfile(\"{top}_tb.vcd\");\n    $dumpvars(0, {top}_tb);\n    // drive inputs here\n    #1000 $finish;\n  end\nendmodule\n"
            ));
            s
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Direction, PortDef, PortType};

    fn module(name: &str, file: &str, deps: &[&str]) -> ModuleDef {
        ModuleDef {
            name: name.to_string(),
            generics: Vec::new(),
            ports: Vec::new(),
            source_path: PathBuf::from(file),
            source_hash: 0,
            dependencies: deps.iter().map(|d| d.to_string()).collect(),
        }
    }

    fn port(name: &str, direction: Direction) -> PortDef {
        PortDef {
            name: name.to_string(),
            direction,
            port_type: PortType::StdLogic,
            bundle: None,
        }
    }

    #[test]
    fn sources_ordered_dependencies_first() {
        let lib = vec![
            module("c", "c.vhd", &["a"]),
            module("a", "a.vhd", &["b"]),
            module("b", "b.vhd", &[]),
        ];
        let order = dependency_ordered_sources(&lib);
        assert_eq!(
            order,
            vec![
                PathBuf::from("b.vhd"),
                PathBuf::from("a.vhd"),
                PathBuf::from("c.vhd")
            ]
        );
    }

    #[test]
    fn shared_file_placed_after_its_last_module_dep() {
        // "util.vhd" holds both `b` (no deps) and `a` (depends on `x` from
        // another file) — the file must land after x.vhd.
        let lib = vec![
            module("b", "util.vhd", &[]),
            module("a", "util.vhd", &["x"]),
            module("x", "x.vhd", &[]),
        ];
        let order = dependency_ordered_sources(&lib);
        assert_eq!(order, vec![PathBuf::from("x.vhd"), PathBuf::from("util.vhd")]);
    }

    #[test]
    fn cyclic_dependencies_do_not_hang() {
        let lib = vec![module("a", "a.vhd", &["b"]), module("b", "b.vhd", &["a"])];
        let order = dependency_ordered_sources(&lib);
        assert_eq!(order.len(), 2);
    }

    #[test]
    fn vhdl_testbench_skeleton_shape() {
        let mut s = Schematic::new("blinky", Language::Vhdl);
        s.top_ports = vec![port("clk", Direction::In), port("led", Direction::Out)];
        let tb = testbench_skeleton(&s);
        assert!(tb.contains("entity blinky_tb is"));
        assert!(tb.contains("signal clk : std_logic := '0';"));
        assert!(tb.contains("signal led : std_logic;"));
        assert!(tb.contains("uut: entity work.blinky"));
        assert!(tb.contains("clk => clk"));
        assert!(tb.contains("clk <= not clk after 5 ns;"));
    }

    #[test]
    fn sv_testbench_skeleton_shape() {
        let mut s = Schematic::new("blinky", Language::SystemVerilog);
        s.top_ports = vec![port("clk", Direction::In), port("led", Direction::Out)];
        let tb = testbench_skeleton(&s);
        assert!(tb.contains("module blinky_tb;"));
        assert!(tb.contains("logic clk = '0;"));
        assert!(tb.contains("blinky uut ("));
        assert!(tb.contains(".clk(clk)"));
        assert!(tb.contains("always #5 clk = ~clk;"));
        assert!(tb.contains("$dumpfile(\"blinky_tb.vcd\");"));
    }

    #[test]
    fn ice40_scaffold_shape() {
        let mut s = Schematic::new("blinky", Language::Vhdl);
        s.top_ports = vec![port("clk", Direction::In), port("led", Direction::Out)];
        let lib = vec![module("pulse", "pulse.vhd", &[])];
        let sc = fpga_scaffold(&s, &lib, FpgaFamily::Ice40, "blinky.hdlc");
        assert_eq!(sc.constraints_name, "blinky.pcf");
        assert!(sc.makefile.contains("synth_ice40"));
        assert!(sc.makefile.contains("nextpnr-ice40"));
        assert!(sc.makefile.contains("icepack"));
        assert!(sc.makefile.contains("yosys -m ghdl"));
        assert!(sc.makefile.contains("LIB_SRCS := pulse.vhd"));
        assert!(sc.makefile.contains("hdl-compose codegen $(PROJECT)"));
        assert!(sc.constraints.contains("# set_io clk <pin>"));
        assert!(sc.constraints.contains("# set_io led <pin>"));
    }

    #[test]
    fn ecp5_sv_scaffold_shape() {
        let s = Schematic::new("blinky", Language::SystemVerilog);
        let sc = fpga_scaffold(&s, &[], FpgaFamily::Ecp5, "blinky.hdlc");
        assert_eq!(sc.constraints_name, "blinky.lpf");
        assert!(sc.makefile.contains("synth_ecp5"));
        assert!(sc.makefile.contains("read_verilog -sv"));
        assert!(sc.makefile.contains("ecppack"));
        assert!(!sc.makefile.contains("ghdl"));
    }
}
