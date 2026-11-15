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

/// Optional per-project tool settings, loaded from
/// `<project>.toolchain.json` next to the `.hdlc` file. Covers what the
/// schematic itself cannot know: VHDL sources that must be analyzed into a
/// named library (packages, vendor stubs) and extra analysis units (e.g.
/// sim-only implementations of a module). Paths are relative to the project
/// directory.
#[derive(Default, serde::Deserialize)]
pub struct ToolchainConfig {
    #[serde(default)]
    pub vhdl_libraries: Vec<VhdlLibrary>,
    /// Analyzed into `work` BEFORE the project's library sources — leaf
    /// dependencies of library modules and sim-only implementations.
    #[serde(default)]
    pub extra_sources: Vec<PathBuf>,
    /// Project library paths to leave out of tool runs (e.g. a
    /// vendor-flow-only file another `extra_sources` entry stands in for).
    #[serde(default)]
    pub exclude_sources: Vec<PathBuf>,
    /// Extra flags for the ghdl-yosys-plugin during `synth`/`build`
    /// (e.g. "--latches" for designs with intentional latches).
    #[serde(default)]
    pub ghdl_synth_flags: Vec<String>,
}

#[derive(serde::Deserialize)]
pub struct VhdlLibrary {
    pub name: String,
    pub files: Vec<PathBuf>,
}

impl ToolchainConfig {
    /// Load `<stem>.toolchain.json` for a project path; absent file → default.
    pub fn load_for(project_path: &Path) -> Result<ToolchainConfig, String> {
        let path = project_path.with_extension("toolchain.json");
        if !path.exists() {
            return Ok(ToolchainConfig::default());
        }
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        serde_json::from_str(&text).map_err(|e| format!("invalid {}: {e}", path.display()))
    }
}

/// A project's complete HDL staged into a scratch directory: library sources
/// copied in, generated files written out, everything addressed by relative
/// filename so tool command lines stay simple.
pub struct StagedBuild {
    pub dir: PathBuf,
    /// Filenames (relative to `dir`) in analysis order: config extra sources,
    /// then library sources with dependencies first, then generated group
    /// files, then the top file.
    pub sources: Vec<String>,
    /// Named-library prelude: (library name, staged filenames), analyzed
    /// before `sources` (ghdl `--work=<name>`).
    pub libraries: Vec<(String, Vec<String>)>,
    /// Config's extra ghdl flags for the yosys plugin.
    pub ghdl_synth_flags: Vec<String>,
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
/// `project_dir` anchors the config's relative paths.
pub fn stage(
    schematic: &Schematic,
    library: &[ModuleDef],
    design: &GeneratedDesign,
    config: &ToolchainConfig,
    project_dir: &Path,
) -> Result<StagedBuild, String> {
    let dir = std::env::temp_dir().join(format!("hdl-compose-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;

    // A path in the config is relative to the project directory. A library
    // source is not: `load_project` already resolved it.
    let from_project = |path: &Path| {
        if path.is_relative() {
            project_dir.join(path)
        } else {
            path.to_path_buf()
        }
    };
    // `./rtl/a.vhd` and `rtl/a.vhd` are the same file.
    let plain = |path: &Path| -> PathBuf {
        path.components()
            .filter(|c| !matches!(c, std::path::Component::CurDir))
            .collect()
    };

    let mut used: Vec<String> = Vec::new();
    let mut copy_in = |src: &Path| -> Result<String, String> {
        let mut fname = src
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| format!("bad source path: {}", src.display()))?
            .to_string();
        // Two different files with the same basename: disambiguate.
        while used.contains(&fname) {
            fname = format!("_{fname}");
        }
        std::fs::copy(src, dir.join(&fname))
            .map_err(|e| format!("cannot copy {}: {e}", src.display()))?;
        used.push(fname.clone());
        Ok(fname)
    };

    let mut libraries = Vec::new();
    for lib in &config.vhdl_libraries {
        let mut files = Vec::new();
        for f in &lib.files {
            files.push(copy_in(&from_project(f))?);
        }
        libraries.push((lib.name.clone(), files));
    }

    let mut sources = Vec::new();
    for src in &config.extra_sources {
        sources.push(copy_in(&from_project(src))?);
    }
    let excluded: Vec<PathBuf> = config
        .exclude_sources
        .iter()
        .map(|e| plain(&from_project(e)))
        .collect();
    for src in dependency_ordered_sources(library) {
        if excluded.contains(&plain(&src)) {
            continue;
        }
        sources.push(copy_in(&src)?);
    }
    for (fname, code) in design.files.iter().chain(std::iter::once(&(
        design.top_filename.clone(),
        design.top_code.clone(),
    ))) {
        std::fs::write(dir.join(fname), code).map_err(|e| format!("cannot write {fname}: {e}"))?;
        sources.push(fname.clone());
    }

    Ok(StagedBuild {
        dir,
        sources,
        libraries,
        ghdl_synth_flags: config.ghdl_synth_flags.clone(),
        top_name: schematic.top_name.clone(),
        language: schematic.language.clone(),
    })
}

/// One external command of a flow. It runs in the build directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub tool: String,
    pub args: Vec<String>,
    /// The file in the build directory that gets the stdout of the tool.
    /// `None` sends the stdout to the terminal.
    pub stdout_to: Option<String>,
}

impl Step {
    fn new<I, S>(tool: &str, args: I) -> Step
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Step {
            tool: tool.to_string(),
            args: args.into_iter().map(Into::into).collect(),
            stdout_to: None,
        }
    }
}

/// One way to do a stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variant {
    /// `run_stages` prints this text when it uses the variant because the
    /// variant before it failed.
    pub note: &'static str,
    pub steps: Vec<Step>,
}

/// One part of a flow. `run_stages` tries the variants in order and uses the
/// first one that passes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage {
    pub name: &'static str,
    pub variants: Vec<Variant>,
}

impl Stage {
    fn single(name: &'static str, steps: Vec<Step>) -> Stage {
        Stage {
            name,
            variants: vec![Variant { note: "", steps }],
        }
    }
}

/// Facts about the installed tools that a step table needs. The functions
/// that build a table get them as data, so the functions call no tool and a
/// test can run them on a host without the tools.
#[derive(Debug, Clone, Default)]
pub struct FlowEnv {
    /// The library directory of the standalone ghdl (see `ghdl_prefix`).
    pub ghdl_prefix: Option<String>,
}

impl FlowEnv {
    /// Ask the installed tools. Only a VHDL flow needs an answer.
    pub fn detect(language: &Language) -> FlowEnv {
        FlowEnv {
            ghdl_prefix: match language {
                Language::Vhdl => ghdl_prefix(),
                Language::SystemVerilog => None,
            },
        }
    }
}

/// Run one step. Returns whether the tool exited successfully. A tool that
/// is not on PATH is an error, not a failed step.
fn run_step(step: &Step, dir: &Path) -> Result<bool, String> {
    let mut cmd = Command::new(&step.tool);
    cmd.args(&step.args).current_dir(dir);
    let spawn_error = |e: std::io::Error| {
        if e.kind() == std::io::ErrorKind::NotFound {
            format!("{} {PATH_HINT}", step.tool)
        } else {
            format!("failed to run {}: {e}", step.tool)
        }
    };
    match &step.stdout_to {
        None => Ok(cmd.status().map_err(spawn_error)?.success()),
        Some(file) => {
            let out = cmd.stderr(Stdio::inherit()).output().map_err(spawn_error)?;
            if !out.status.success() {
                return Ok(false);
            }
            std::fs::write(dir.join(file), out.stdout)
                .map_err(|e| format!("cannot write {file}: {e}"))?;
            Ok(true)
        }
    }
}

/// Run steps in order. Stop at the first step that fails.
fn run_steps(steps: &[Step], dir: &Path) -> Result<bool, String> {
    for step in steps {
        if !run_step(step, dir)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Run a step table in the directory `dir`. Each stage must pass. A stage
/// passes when one of its variants passes; the variants run in order.
pub fn run_stages(stages: &[Stage], dir: &Path) -> Result<bool, String> {
    'stages: for stage in stages {
        for (i, variant) in stage.variants.iter().enumerate() {
            if i > 0 {
                println!("note: {}", variant.note);
            }
            if run_steps(&variant.steps, dir)? {
                continue 'stages;
            }
        }
        return Ok(false);
    }
    Ok(true)
}

/// The ghdl steps that analyze the named-library sources of the config
/// (`--work=<name>`). They must run before the work-library files. Empty
/// without a toolchain config.
fn library_steps(staged: &StagedBuild) -> Vec<Step> {
    staged
        .libraries
        .iter()
        .map(|(name, files)| {
            let mut args = vec![
                "-a".to_string(),
                "--std=08".to_string(),
                format!("--work={name}"),
            ];
            args.extend(files.iter().cloned());
            Step::new("ghdl", args)
        })
        .collect()
}

/// The stage that analyzes the named libraries. Only a VHDL design with
/// named libraries has it.
fn library_stage(staged: &StagedBuild) -> Option<Stage> {
    let steps = library_steps(staged);
    (matches!(staged.language, Language::Vhdl) && !steps.is_empty())
        .then(|| Stage::single("libraries", steps))
}

/// The ghdl step that analyzes every work source.
fn analyze_step(staged: &StagedBuild) -> Step {
    let mut args = vec!["-a".to_string(), "--std=08".to_string()];
    args.extend(staged.sources.iter().cloned());
    Step::new("ghdl", args)
}

/// Elaborate the staged design with a real frontend: ghdl for VHDL,
/// `verilator --lint-only` for SystemVerilog.
pub fn check_stages(staged: &StagedBuild) -> Vec<Stage> {
    let steps = match staged.language {
        Language::Vhdl => {
            let mut steps = library_steps(staged);
            steps.push(analyze_step(staged));
            steps.push(Step::new(
                "ghdl",
                ["-e", "--std=08", staged.top_name.as_str()],
            ));
            steps
        }
        Language::SystemVerilog => {
            let mut args = vec![
                "--lint-only".to_string(),
                "-sv".to_string(),
                "--top-module".to_string(),
                staged.top_name.clone(),
            ];
            args.extend(staged.sources.iter().cloned());
            vec![Step::new("verilator", args)]
        }
    };
    vec![Stage::single("check", steps)]
}

/// Run `check_stages`. Returns whether the design passed.
pub fn run_check(staged: &StagedBuild) -> Result<bool, String> {
    run_stages(&check_stages(staged), &staged.dir)
}

/// The ghdl-yosys-plugin's embedded ghdl does not resolve its std/ieee
/// library path the way the standalone binary does; ask the standalone ghdl
/// where its libraries live so the plugin can be pointed at them.
fn ghdl_prefix() -> Option<String> {
    let out = Command::new("ghdl").arg("--disp-config").output().ok()?;
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .find_map(|l| {
            l.strip_prefix("library directory:")
                .map(|v| v.trim().to_string())
        })
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("GHDL_PREFIX").ok())
}

/// The steps of the fallback VHDL frontend, for a design that the
/// ghdl-yosys-plugin cannot read (known gaps: out-port defaults, some ROM
/// shapes). The standalone ghdl analyzes the sources and writes a Verilog
/// netlist to `design_net.v` in the build directory for yosys to read.
fn netlist_steps(staged: &StagedBuild) -> Vec<Step> {
    let mut args = vec!["synth".to_string(), "--std=08".to_string()];
    if !staged.libraries.is_empty() {
        args.push("-P.".to_string());
    }
    args.extend(staged.ghdl_synth_flags.iter().cloned());
    args.push("--out=verilog".to_string());
    args.push(staged.top_name.clone());
    vec![
        analyze_step(staged),
        Step {
            tool: "ghdl".to_string(),
            args,
            stdout_to: Some("design_net.v".to_string()),
        },
    ]
}

/// Argument string for the ghdl-yosys-plugin: std, library prefix, search
/// path for pre-analyzed named libraries, then every work source.
fn ghdl_plugin_flags(staged: &StagedBuild, env: &FlowEnv) -> String {
    let mut flags = String::from("--std=08 ");
    if let Some(p) = &env.ghdl_prefix {
        flags.push_str(&format!("--PREFIX={p} "));
    }
    if !staged.libraries.is_empty() {
        flags.push_str("-P. ");
    }
    for f in &staged.ghdl_synth_flags {
        flags.push_str(f);
        flags.push(' ');
    }
    flags.push_str(&staged.sources.join(" "));
    flags
}

/// One way to read the staged design into yosys.
struct Frontend {
    note: &'static str,
    /// The steps that must run before the yosys step.
    pre: Vec<Step>,
    /// The yosys plugin that `read` needs (`-m <plugin>`).
    plugin: Option<&'static str>,
    /// The yosys commands that read the design.
    read: String,
    /// True when `read` also selects the top module.
    selects_top: bool,
}

/// The frontends for the language of the design, the preferred one first.
fn frontends(staged: &StagedBuild, env: &FlowEnv) -> Vec<Frontend> {
    match staged.language {
        Language::Vhdl => vec![
            Frontend {
                note: "",
                pre: Vec::new(),
                plugin: Some("ghdl"),
                read: format!(
                    "ghdl {} -e {}",
                    ghdl_plugin_flags(staged, env),
                    staged.top_name
                ),
                selects_top: true,
            },
            Frontend {
                note: "ghdl-yosys-plugin failed, retrying via ghdl synth -> verilog netlist",
                pre: netlist_steps(staged),
                plugin: None,
                read: "read_verilog design_net.v".to_string(),
                selects_top: false,
            },
        ],
        Language::SystemVerilog => vec![Frontend {
            note: "",
            pre: Vec::new(),
            plugin: None,
            read: format!("read_verilog -sv {}", staged.sources.join(" ")),
            selects_top: false,
        }],
    }
}

/// A stage with one yosys run for each frontend. `script` gives the yosys
/// script for a frontend.
fn yosys_stage(
    name: &'static str,
    staged: &StagedBuild,
    env: &FlowEnv,
    script: impl Fn(&Frontend) -> String,
) -> Stage {
    let variants = frontends(staged, env)
        .into_iter()
        .map(|frontend| {
            let mut args: Vec<String> = Vec::new();
            if let Some(plugin) = frontend.plugin {
                args.extend(["-m".to_string(), plugin.to_string()]);
            }
            args.extend(["-p".to_string(), script(&frontend)]);
            let mut steps = frontend.pre;
            steps.push(Step::new("yosys", args));
            Variant {
                note: frontend.note,
                steps,
            }
        })
        .collect();
    Stage { name, variants }
}

/// Generic yosys synthesis of the staged design, with `stat` at the end.
/// It proves that the design can be synthesized, nothing more.
pub fn synth_stages(staged: &StagedBuild, env: &FlowEnv) -> Vec<Stage> {
    let top = &staged.top_name;
    let mut stages: Vec<Stage> = library_stage(staged).into_iter().collect();
    stages.push(yosys_stage("synth", staged, env, |frontend| {
        if frontend.selects_top {
            format!("{}; synth; stat", frontend.read)
        } else {
            format!("{}; hierarchy -top {top}; synth; stat", frontend.read)
        }
    }));
    stages
}

/// Run `synth_stages`. yosys prints its warnings and `stat` to the terminal.
pub fn run_synth(staged: &StagedBuild) -> Result<bool, String> {
    let env = FlowEnv::detect(&staged.language);
    run_stages(&synth_stages(staged, &env), &staged.dir)
}

/// The file that nextpnr writes and the pack tool reads.
fn pnr_output(family: FpgaFamily) -> &'static str {
    match family {
        FpgaFamily::Ice40 => "pnr.asc",
        FpgaFamily::Ecp5 => "pnr.config",
        FpgaFamily::Gowin => "pnr.json",
    }
}

/// The place-and-route step for a board. `constraints` is the absolute path
/// of the constraints file of the board.
fn pnr_step(board: &crate::board::Board, constraints: &str) -> Step {
    let output = pnr_output(board.family);
    let (tool, mut args): (&str, Vec<String>) = match board.family {
        FpgaFamily::Ice40 => (
            "nextpnr-ice40",
            vec![
                format!("--{}", board.device),
                "--package".into(),
                board.package.clone(),
                "--json".into(),
                "design.json".into(),
                "--pcf".into(),
                constraints.into(),
                "--asc".into(),
                output.into(),
            ],
        ),
        FpgaFamily::Ecp5 => (
            "nextpnr-ecp5",
            vec![
                format!("--{}", board.device),
                "--package".into(),
                board.package.clone(),
                "--json".into(),
                "design.json".into(),
                "--lpf".into(),
                constraints.into(),
                "--textcfg".into(),
                output.into(),
            ],
        ),
        FpgaFamily::Gowin => (
            "nextpnr-himbaechel",
            vec![
                "--device".into(),
                board.device.clone(),
                "--vopt".into(),
                format!("cst={constraints}"),
                "--json".into(),
                "design.json".into(),
                "--write".into(),
                output.into(),
            ],
        ),
    };
    // The chip view reads these three files. The Gowin flow already
    // writes `pnr.json` as the input of its pack step.
    if !matches!(board.family, FpgaFamily::Gowin) {
        args.extend(["--write".into(), "pnr.json".into()]);
    }
    args.extend(["--report", "report.json", "--placed-svg", "placed.svg"].map(String::from));
    Step::new(tool, args)
}

/// The name of the bitstream in the build directory.
const BITSTREAM: &str = "out.bit";

/// The step that packs the nextpnr output into a bitstream.
fn pack_step(board: &crate::board::Board) -> Step {
    let input = pnr_output(board.family).to_string();
    match board.family {
        FpgaFamily::Ice40 => Step::new("icepack", [input, BITSTREAM.to_string()]),
        FpgaFamily::Ecp5 => {
            let mut args = board.pack_args.clone();
            args.push(input);
            args.push(BITSTREAM.into());
            Step::new("ecppack", args)
        }
        FpgaFamily::Gowin => {
            let mut args = board.pack_args.clone();
            args.extend(["-o".into(), BITSTREAM.into(), input]);
            Step::new("gowin_pack", args)
        }
    }
}

/// The bitstream flow for a board: yosys `synth_<family>`, nextpnr, pack.
/// `constraints` is the absolute path of the constraints file of the board.
pub fn build_stages(
    staged: &StagedBuild,
    board: &crate::board::Board,
    constraints: &str,
    env: &FlowEnv,
) -> Vec<Stage> {
    let synth_cmd = match board.family {
        FpgaFamily::Ice40 => "synth_ice40",
        FpgaFamily::Ecp5 => "synth_ecp5",
        FpgaFamily::Gowin => "synth_gowin",
    };
    let top = &staged.top_name;
    let mut stages: Vec<Stage> = library_stage(staged).into_iter().collect();
    stages.push(yosys_stage("synth", staged, env, |frontend| {
        if frontend.selects_top {
            format!("{}; {synth_cmd} -json design.json", frontend.read)
        } else {
            format!(
                "{}; {synth_cmd} -top {top} -json design.json",
                frontend.read
            )
        }
    }));
    stages.push(Stage::single("pnr", vec![pnr_step(board, constraints)]));
    stages.push(Stage::single("pack", vec![pack_step(board)]));
    stages
}

/// Run `build_stages` in the build directory, then copy the bitstream to
/// `out_bit`. The timing report of nextpnr goes to the terminal.
pub fn run_build(
    staged: &StagedBuild,
    board: &crate::board::Board,
    out_bit: &Path,
) -> Result<bool, String> {
    let constraints = board
        .constraints
        .canonicalize()
        .map_err(|e| format!("cannot resolve {}: {e}", board.constraints.display()))?;
    let env = FlowEnv::detect(&staged.language);
    let stages = build_stages(staged, board, &constraints.to_string_lossy(), &env);
    if !run_stages(&stages, &staged.dir)? {
        return Ok(false);
    }

    std::fs::copy(staged.dir.join(BITSTREAM), out_bit)
        .map_err(|e| format!("cannot write {}: {e}", out_bit.display()))?;
    // A chip view failure must not fail a build that made a bitstream.
    match write_floorplan(&staged.dir, &floorplan_path(out_bit)) {
        Ok(()) => println!("chip view written to {}", floorplan_path(out_bit).display()),
        Err(msg) => println!("warning: no chip view: {msg}"),
    }
    Ok(true)
}

/// The chip view file that belongs to a bitstream: `<top>.floorplan.json`.
pub fn floorplan_path(bitstream: &Path) -> PathBuf {
    bitstream.with_extension("floorplan.json")
}

/// Merge the nextpnr outputs in `dir` into one chip view file at `out`.
fn write_floorplan(dir: &Path, out: &Path) -> Result<(), String> {
    let read = |name: &str| {
        std::fs::read_to_string(dir.join(name)).map_err(|e| format!("cannot read {name}: {e}"))
    };
    let plan = crate::floorplan::Floorplan::from_nextpnr(
        &read("pnr.json")?,
        &read("placed.svg")?,
        read("report.json").ok().as_deref(),
    )?;
    let unmatched = plan.unmatched_tiles();
    if !unmatched.is_empty() {
        println!(
            "warning: chip view: {} tiles hold cells that the drawing does not show (first: X{}/Y{})",
            unmatched.len(),
            unmatched[0].0,
            unmatched[0].1
        );
    }
    let text = serde_json::to_string(&plan).map_err(|e| e.to_string())?;
    std::fs::write(out, text).map_err(|e| format!("cannot write {}: {e}", out.display()))
}

/// Program a bitstream with openFPGALoader. SRAM load by default (volatile);
/// `persistent` writes config flash.
pub fn run_flash(
    bitstream: &Path,
    board: &crate::board::Board,
    persistent: bool,
) -> Result<bool, String> {
    require_tools(&["openFPGALoader"])?;
    let mut args: Vec<String> = board.prog_args.clone();
    if persistent {
        args.push("--write-flash".into());
    }
    args.push(bitstream.to_string_lossy().into_owned());
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    run_tool("openFPGALoader", &args, &cwd)
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
            if !run_steps(&library_steps(staged), &staged.dir)? {
                return Ok(None);
            }
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

#[derive(Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
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
         \thdlc codegen $(PROJECT) -o $@\n\
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
            |p| {
                format!(
                    "# LOCATE COMP \"{p}\" SITE \"<site>\";\n# IOBUF PORT \"{p}\" IO_TYPE=LVCMOS33;\n"
                )
            },
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
            matches!(p.direction, Direction::In) && matches!(p.port_type, PortType::StdLogic) && {
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
                 -- `hdlc sim` runs it with --stop-time=1us.\n\
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
        assert_eq!(
            order,
            vec![PathBuf::from("x.vhd"), PathBuf::from("util.vhd")]
        );
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
        assert!(sc.makefile.contains("hdlc codegen $(PROJECT)"));
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

#[cfg(test)]
mod flow_tests {
    use super::*;
    use crate::board::Board;

    fn staged(language: Language) -> StagedBuild {
        let ext = match language {
            Language::Vhdl => "vhd",
            Language::SystemVerilog => "sv",
        };
        StagedBuild {
            dir: PathBuf::from("/nonexistent"),
            sources: vec![format!("leaf.{ext}"), format!("top.{ext}")],
            libraries: Vec::new(),
            ghdl_synth_flags: Vec::new(),
            top_name: "top".to_string(),
            language,
        }
    }

    fn env() -> FlowEnv {
        FlowEnv {
            ghdl_prefix: Some("/ghdl/lib".to_string()),
        }
    }

    fn board(family: FpgaFamily, device: &str, package: &str) -> Board {
        Board {
            name: "test".to_string(),
            family,
            device: device.to_string(),
            package: package.to_string(),
            constraints: PathBuf::from("pins"),
            pack_args: vec!["--compress".to_string()],
            prog_args: Vec::new(),
        }
    }

    fn step(tool: &str, args: &[&str]) -> Step {
        Step::new(tool, args.iter().copied())
    }

    fn single(name: &'static str, steps: Vec<Step>) -> Stage {
        Stage::single(name, steps)
    }

    #[test]
    fn vhdl_check_analyzes_then_elaborates() {
        assert_eq!(
            check_stages(&staged(Language::Vhdl)),
            vec![single(
                "check",
                vec![
                    step("ghdl", &["-a", "--std=08", "leaf.vhd", "top.vhd"]),
                    step("ghdl", &["-e", "--std=08", "top"]),
                ]
            )]
        );
    }

    #[test]
    fn sv_check_runs_verilator_lint() {
        assert_eq!(
            check_stages(&staged(Language::SystemVerilog)),
            vec![single(
                "check",
                vec![step(
                    "verilator",
                    &[
                        "--lint-only",
                        "-sv",
                        "--top-module",
                        "top",
                        "leaf.sv",
                        "top.sv"
                    ]
                )]
            )]
        );
    }

    #[test]
    fn named_libraries_are_analyzed_first_and_put_on_the_search_path() {
        let mut s = staged(Language::Vhdl);
        s.libraries = vec![("loot".to_string(), vec!["pkg.vhd".to_string()])];
        let library = step("ghdl", &["-a", "--std=08", "--work=loot", "pkg.vhd"]);

        assert_eq!(check_stages(&s)[0].variants[0].steps[0], library);

        let stages = synth_stages(&s, &env());
        assert_eq!(stages[0], single("libraries", vec![library]));
        let plugin = &stages[1].variants[0].steps[0];
        assert_eq!(
            plugin.args[3],
            "ghdl --std=08 --PREFIX=/ghdl/lib -P. leaf.vhd top.vhd -e top; synth; stat"
        );
        let netlist = &stages[1].variants[1].steps[1];
        assert_eq!(
            netlist.args,
            ["synth", "--std=08", "-P.", "--out=verilog", "top"]
        );
    }

    #[test]
    fn vhdl_synth_has_the_plugin_then_the_netlist_variant() {
        let mut s = staged(Language::Vhdl);
        s.ghdl_synth_flags = vec!["--latches".to_string()];
        assert_eq!(
            synth_stages(&s, &env()),
            vec![Stage {
                name: "synth",
                variants: vec![
                    Variant {
                        note: "",
                        steps: vec![step(
                            "yosys",
                            &[
                                "-m",
                                "ghdl",
                                "-p",
                                "ghdl --std=08 --PREFIX=/ghdl/lib --latches leaf.vhd top.vhd -e top; synth; stat"
                            ]
                        )],
                    },
                    Variant {
                        note: "ghdl-yosys-plugin failed, retrying via ghdl synth -> verilog netlist",
                        steps: vec![
                            step("ghdl", &["-a", "--std=08", "leaf.vhd", "top.vhd"]),
                            Step {
                                stdout_to: Some("design_net.v".to_string()),
                                ..step(
                                    "ghdl",
                                    &["synth", "--std=08", "--latches", "--out=verilog", "top"]
                                )
                            },
                            step(
                                "yosys",
                                &[
                                    "-p",
                                    "read_verilog design_net.v; hierarchy -top top; synth; stat"
                                ]
                            ),
                        ],
                    },
                ],
            }]
        );
    }

    #[test]
    fn sv_synth_reads_verilog() {
        assert_eq!(
            synth_stages(&staged(Language::SystemVerilog), &FlowEnv::default()),
            vec![single(
                "synth",
                vec![step(
                    "yosys",
                    &[
                        "-p",
                        "read_verilog -sv leaf.sv top.sv; hierarchy -top top; synth; stat"
                    ]
                )]
            )]
        );
    }

    /// The commands of `run_build` before the step table, for the flow that
    /// is verified on hardware (VHDL, ECP5).
    #[test]
    fn vhdl_ecp5_build_commands() {
        let stages = build_stages(
            &staged(Language::Vhdl),
            &board(FpgaFamily::Ecp5, "25k", "CABGA256"),
            "/abs/board.lpf",
            &env(),
        );
        let names: Vec<&str> = stages.iter().map(|s| s.name).collect();
        assert_eq!(names, ["synth", "pnr", "pack"]);
        assert_eq!(
            stages[0].variants[0].steps,
            vec![step(
                "yosys",
                &[
                    "-m",
                    "ghdl",
                    "-p",
                    "ghdl --std=08 --PREFIX=/ghdl/lib leaf.vhd top.vhd -e top; synth_ecp5 -json design.json"
                ]
            )]
        );
        assert_eq!(
            stages[0].variants[1].steps[2],
            step(
                "yosys",
                &[
                    "-p",
                    "read_verilog design_net.v; synth_ecp5 -top top -json design.json"
                ]
            )
        );
        assert_eq!(
            stages[1],
            single(
                "pnr",
                vec![step(
                    "nextpnr-ecp5",
                    &[
                        "--25k",
                        "--package",
                        "CABGA256",
                        "--json",
                        "design.json",
                        "--lpf",
                        "/abs/board.lpf",
                        "--textcfg",
                        "pnr.config",
                        "--write",
                        "pnr.json",
                        "--report",
                        "report.json",
                        "--placed-svg",
                        "placed.svg",
                    ]
                )]
            )
        );
        assert_eq!(
            stages[2],
            single(
                "pack",
                vec![step("ecppack", &["--compress", "pnr.config", "out.bit"])]
            )
        );
    }

    #[test]
    fn sv_ice40_build_commands() {
        let stages = build_stages(
            &staged(Language::SystemVerilog),
            &board(FpgaFamily::Ice40, "up5k", "sg48"),
            "/abs/board.pcf",
            &FlowEnv::default(),
        );
        assert_eq!(
            stages[0],
            single(
                "synth",
                vec![step(
                    "yosys",
                    &[
                        "-p",
                        "read_verilog -sv leaf.sv top.sv; synth_ice40 -top top -json design.json"
                    ]
                )]
            )
        );
        assert_eq!(
            stages[1].variants[0].steps[0],
            step(
                "nextpnr-ice40",
                &[
                    "--up5k",
                    "--package",
                    "sg48",
                    "--json",
                    "design.json",
                    "--pcf",
                    "/abs/board.pcf",
                    "--asc",
                    "pnr.asc",
                    "--write",
                    "pnr.json",
                    "--report",
                    "report.json",
                    "--placed-svg",
                    "placed.svg",
                ]
            )
        );
        // icepack takes no arguments from the board file.
        assert_eq!(
            stages[2].variants[0].steps[0],
            step("icepack", &["pnr.asc", "out.bit"])
        );
    }

    #[test]
    fn gowin_build_packs_the_json_that_nextpnr_writes() {
        let stages = build_stages(
            &staged(Language::SystemVerilog),
            &board(FpgaFamily::Gowin, "GW1NR-LV9QN88PC6/I5", ""),
            "/abs/board.cst",
            &FlowEnv::default(),
        );
        assert_eq!(
            stages[1].variants[0].steps[0],
            step(
                "nextpnr-himbaechel",
                &[
                    "--device",
                    "GW1NR-LV9QN88PC6/I5",
                    "--vopt",
                    "cst=/abs/board.cst",
                    "--json",
                    "design.json",
                    "--write",
                    "pnr.json",
                    "--report",
                    "report.json",
                    "--placed-svg",
                    "placed.svg",
                ]
            )
        );
        assert_eq!(
            stages[2].variants[0].steps[0],
            step("gowin_pack", &["--compress", "-o", "out.bit", "pnr.json"])
        );
    }

    #[cfg(unix)]
    mod run {
        use super::*;

        fn variants(tools: &[&str]) -> Stage {
            Stage {
                name: "stage",
                variants: tools
                    .iter()
                    .map(|tool| Variant {
                        note: "next variant",
                        steps: vec![step(tool, &[])],
                    })
                    .collect(),
            }
        }

        #[test]
        fn a_stage_passes_when_a_later_variant_passes() {
            let dir = tempfile::tempdir().unwrap();
            assert_eq!(
                run_stages(&[variants(&["false", "true"])], dir.path()),
                Ok(true)
            );
        }

        #[test]
        fn a_stage_fails_when_each_variant_fails() {
            let dir = tempfile::tempdir().unwrap();
            assert_eq!(
                run_stages(&[variants(&["false", "false"])], dir.path()),
                Ok(false)
            );
        }

        #[test]
        fn a_failed_stage_stops_the_flow() {
            let dir = tempfile::tempdir().unwrap();
            let marker = Step {
                stdout_to: Some("ran".to_string()),
                ..step("true", &[])
            };
            let stages = [variants(&["false"]), single("later", vec![marker])];
            assert_eq!(run_stages(&stages, dir.path()), Ok(false));
            assert!(!dir.path().join("ran").exists());
        }

        #[test]
        fn a_missing_tool_is_an_error_and_no_other_variant_runs() {
            let dir = tempfile::tempdir().unwrap();
            let error =
                run_stages(&[variants(&["hdlc-no-such-tool", "true"])], dir.path()).unwrap_err();
            assert!(
                error.starts_with("hdlc-no-such-tool not found on PATH"),
                "{error}"
            );
        }

        #[test]
        fn stdout_of_a_step_goes_to_its_file() {
            let dir = tempfile::tempdir().unwrap();
            let echo = Step {
                stdout_to: Some("out.txt".to_string()),
                ..step("echo", &["netlist"])
            };
            assert_eq!(
                run_stages(&[single("echo", vec![echo])], dir.path()),
                Ok(true)
            );
            assert_eq!(
                std::fs::read_to_string(dir.path().join("out.txt")).unwrap(),
                "netlist\n"
            );
        }
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;
    use crate::codegen::GeneratedDesign;

    /// `stage` gives each call in one process the same directory. The tests
    /// that call it must not run at the same time.
    static STAGE: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn stage_honors_config_order_libraries_and_excludes() {
        let _stage = STAGE.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = std::env::temp_dir().join(format!("hdlc-cfgtest-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join("rtl")).unwrap();
        for f in [
            "rtl/pkg.vhd",
            "rtl/leaf.vhd",
            "rtl/mod_a.vhd",
            "rtl/vendor.v",
        ] {
            std::fs::write(tmp.join(f), "-- test").unwrap();
        }

        let schematic = Schematic::new("top", Language::Vhdl);
        // `load_project` gives each library path with the project directory.
        let library = vec![
            ModuleDef {
                name: "mod_a".into(),
                generics: vec![],
                ports: vec![],
                source_path: tmp.join("rtl/mod_a.vhd"),
                source_hash: 0,
                dependencies: vec![],
            },
            ModuleDef {
                name: "vendor".into(),
                generics: vec![],
                ports: vec![],
                source_path: tmp.join("rtl/vendor.v"),
                source_hash: 0,
                dependencies: vec![],
            },
        ];
        let design = GeneratedDesign {
            files: vec![],
            top_filename: "top.vhd".into(),
            top_code: "-- top".into(),
        };
        let config: ToolchainConfig = serde_json::from_str(
            r#"{
                "vhdl_libraries": [{"name": "loot", "files": ["rtl/pkg.vhd"]}],
                "extra_sources": ["rtl/leaf.vhd"],
                "exclude_sources": ["rtl/vendor.v"]
            }"#,
        )
        .unwrap();

        let staged = stage(&schematic, &library, &design, &config, &tmp).unwrap();
        assert_eq!(
            staged.libraries,
            vec![("loot".to_string(), vec!["pkg.vhd".to_string()])]
        );
        assert_eq!(staged.sources, vec!["leaf.vhd", "mod_a.vhd", "top.vhd"]);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// Regression: `hdlc check tests/projects/hdlc/voice_chain.hdlc` from
    /// the repository root stopped with "cannot copy
    /// tests/projects/hdlc/tests/projects/hdlc/rtl/...". `load_project`
    /// resolves a library path against the directory of the project, and
    /// `stage` added that directory a second time.
    #[test]
    fn stage_takes_library_sources_as_the_loader_resolved_them() {
        let _stage = STAGE.lock().unwrap_or_else(|e| e.into_inner());
        // The working directory of a test is the package root. `tempfile`
        // gives an absolute path; the defect needs a relative one.
        let project = tempfile::Builder::new()
            .prefix("hdlc-stage-")
            .tempdir_in(".")
            .unwrap();
        let project_dir = PathBuf::from(project.path().file_name().unwrap());
        let project_dir = project_dir.as_path();
        assert!(project_dir.is_relative() && project_dir.is_dir());
        std::fs::create_dir_all(project_dir.join("rtl")).unwrap();
        for f in ["rtl/mod_a.vhd", "rtl/vendor.v"] {
            std::fs::write(project_dir.join(f), "-- test").unwrap();
        }

        let module = |name: &str, file: &str| ModuleDef {
            name: name.into(),
            generics: vec![],
            ports: vec![],
            source_path: project_dir.join(file),
            source_hash: 0,
            dependencies: vec![],
        };
        let library = vec![
            module("mod_a", "rtl/mod_a.vhd"),
            module("vendor", "rtl/vendor.v"),
        ];
        let design = GeneratedDesign {
            files: vec![],
            top_filename: "top.vhd".into(),
            top_code: "-- top".into(),
        };
        let config: ToolchainConfig =
            serde_json::from_str(r#"{"exclude_sources": ["rtl/vendor.v"]}"#).unwrap();

        let staged = stage(
            &Schematic::new("top", Language::Vhdl),
            &library,
            &design,
            &config,
            project_dir,
        )
        .unwrap();
        assert_eq!(staged.sources, vec!["mod_a.vhd", "top.vhd"]);
        let _ = std::fs::remove_dir_all(&staged.dir);
    }

    #[test]
    fn missing_config_is_default() {
        let cfg = ToolchainConfig::load_for(Path::new("/nonexistent/proj.hdlc")).unwrap();
        assert!(cfg.vhdl_libraries.is_empty() && cfg.extra_sources.is_empty());
    }
}
