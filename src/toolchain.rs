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

/// What `hdlc doctor` found for one external tool.
pub struct ToolReport {
    /// The command of the tool, e.g. `yosys` or `yosys -m ghdl`.
    pub name: &'static str,
    /// The version line of the tool, or a short word when the tool prints
    /// no version. `None`: the tool is not on PATH.
    pub found: Option<String>,
    /// The hdlc commands that run the tool.
    pub used_by: &'static str,
}

/// Each tool that hdlc can run: its name, the arguments that make it print
/// its version (empty when it has none), and the hdlc commands that run it.
const TOOLS: &[(&str, &[&str], &str)] = &[
    ("ghdl", &["--version"], "check, synth, build, sim (VHDL)"),
    ("yosys", &["-V"], "synth, build"),
    ("verilator", &["--version"], "check (SystemVerilog)"),
    ("iverilog", &["-V"], "sim (SystemVerilog)"),
    ("vvp", &["-V"], "sim (SystemVerilog)"),
    ("nextpnr-ice40", &["--version"], "build (ice40)"),
    ("nextpnr-ecp5", &["--version"], "build (ecp5)"),
    ("nextpnr-himbaechel", &["--version"], "build (gowin)"),
    ("icepack", &[], "build (ice40)"),
    ("ecppack", &["--version"], "build (ecp5)"),
    ("gowin_pack", &[], "build (gowin)"),
    ("openFPGALoader", &["-V"], "flash"),
    ("surfer", &["--version"], "sim --wave"),
    (
        "gtkwave",
        &["--version"],
        "sim --wave, when surfer is missing",
    ),
];

/// The yosys plugins that hdlc loads: the name in the report, the plugin,
/// and the hdlc commands that load it.
const YOSYS_PLUGINS: &[(&str, &str, &str)] = &[("yosys -m ghdl", "ghdl", "synth, build (VHDL)")];

/// The first line with text in the output of a tool. When the line gives
/// the version as `(Version <text>)`, as nextpnr does, only `<text>`.
fn first_line(output: &[u8]) -> String {
    let text = String::from_utf8_lossy(output);
    let line = text
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or_default();
    match line.split_once("(Version ") {
        Some((_, version)) => version.trim_end_matches(')').to_string(),
        None => line.to_string(),
    }
}

/// The file of a tool in the first of `dirs` that has it.
fn find_in(dirs: impl Iterator<Item = PathBuf>, name: &str) -> Option<PathBuf> {
    dirs.map(|dir| dir.join(name)).find(|file| file.is_file())
}

fn find_on_path(name: &str) -> Option<PathBuf> {
    find_in(std::env::split_paths(&std::env::var_os("PATH")?), name)
}

/// Look for each tool that hdlc can run and ask it for its version.
pub fn doctor() -> Vec<ToolReport> {
    let output = |name: &str, args: &[&str]| {
        Command::new(name)
            .args(args)
            .stdin(Stdio::null())
            .output()
            .ok()
    };
    let mut reports: Vec<ToolReport> = TOOLS
        .iter()
        .map(|&(name, version_args, used_by)| {
            let found = find_on_path(name).map(|_| {
                let version = match version_args {
                    [] => String::new(),
                    args => output(name, args)
                        .map(|out| first_line(&[out.stdout, out.stderr].concat()))
                        .unwrap_or_default(),
                };
                if version.is_empty() {
                    "found".to_string()
                } else {
                    version
                }
            });
            ToolReport {
                name,
                found,
                used_by,
            }
        })
        .collect();
    let has_yosys = find_on_path("yosys").is_some();
    for &(name, plugin, used_by) in YOSYS_PLUGINS {
        // yosys stops with an error when it cannot load a plugin.
        let loads = has_yosys
            && output("yosys", &["-q", "-m", plugin, "-p", ""]).is_some_and(|o| o.status.success());
        reports.push(ToolReport {
            name,
            found: loads.then(|| "loads".to_string()),
            used_by,
        });
    }
    reports
}

/// The report of `hdlc doctor` as text: one line for each tool.
pub fn doctor_table(reports: &[ToolReport]) -> String {
    // A version line can be long (yosys prints its compiler).
    const STATUS_WIDTH: usize = 60;
    let status = |report: &ToolReport| -> String {
        match &report.found {
            Some(version) => version.chars().take(STATUS_WIDTH).collect(),
            None => "missing".to_string(),
        }
    };
    let name_width = reports.iter().map(|r| r.name.len()).max().unwrap_or(0);
    let status_width = reports
        .iter()
        .map(|r| status(r).chars().count())
        .max()
        .unwrap_or(0);
    let mut table = String::new();
    for report in reports {
        table.push_str(&format!(
            "{:<name_width$}  {:<status_width$}  {}\n",
            report.name,
            status(report),
            report.used_by
        ));
    }
    if reports.iter().any(|r| r.found.is_none()) {
        table.push_str(
            "\nA missing tool is not on PATH. Activate oss-cad-suite first: source <oss-cad-suite>/environment\n",
        );
    }
    table
}

/// Optional per-project tool settings, loaded from
/// `<project>.toolchain.json` next to the `.hdlc` file. Covers what the
/// schematic itself cannot know: VHDL sources that must be analyzed into a
/// named library (packages, vendor stubs), extra analysis units (e.g.
/// sim-only implementations of a module), the board, and options for the
/// tools. Paths are relative to the project directory.
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
    /// The board definition file for `build` and `flash`. The `--board`
    /// flag has priority over it.
    #[serde(default)]
    pub board: Option<PathBuf>,
    /// The VHDL standard for each ghdl step (`--std=<value>`). Default: 08.
    #[serde(default)]
    pub vhdl_std: Option<String>,
    /// Extra options for the yosys `synth_<family>` command of `build`
    /// (e.g. "-abc9"). The generic `synth` command does not know them all,
    /// so `hdlc synth` does not use them.
    #[serde(default)]
    pub synth_args: Vec<String>,
    /// Extra arguments for nextpnr. They follow the arguments of the board.
    #[serde(default)]
    pub pnr_args: Vec<String>,
    /// Clock constraints for nextpnr: top-level port name to frequency in
    /// MHz.
    #[serde(default)]
    pub clocks: std::collections::BTreeMap<String, f64>,
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

    /// The board file of the config, resolved against the project directory.
    pub fn board_path(&self, project_dir: &Path) -> Option<PathBuf> {
        self.board.as_ref().map(|board| {
            if board.is_relative() {
                project_dir.join(board)
            } else {
                board.clone()
            }
        })
    }

    /// The form of a board path in the config: relative to the project
    /// directory when the board is below it, else as given.
    fn stored_board(project_path: &Path, board: &Path) -> PathBuf {
        let relative = project_path
            .parent()
            .filter(|dir| !dir.as_os_str().is_empty())
            .and_then(|dir| board.strip_prefix(dir).ok())
            .unwrap_or(board);
        // `./x.board.json` and `x.board.json` are the same file.
        relative
            .components()
            .filter(|c| !matches!(c, std::path::Component::CurDir))
            .collect()
    }

    /// Set the `board` key in `<stem>.toolchain.json` of a project. This is
    /// the only function that writes that file. A board below the project
    /// directory is stored relative to it, so the project stays portable;
    /// a board elsewhere is stored as given. Each other key keeps its
    /// value. The file gets the standard layout of `serde_json`.
    pub fn set_board(project_path: &Path, board: &Path) -> Result<(), String> {
        let path = project_path.with_extension("toolchain.json");
        let mut config: serde_json::Value = if path.exists() {
            let text = std::fs::read_to_string(&path)
                .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
            serde_json::from_str(&text).map_err(|e| format!("invalid {}: {e}", path.display()))?
        } else {
            serde_json::Value::Object(serde_json::Map::new())
        };
        let keys = config
            .as_object_mut()
            .ok_or_else(|| format!("invalid {}: not a JSON object", path.display()))?;
        keys.insert(
            "board".to_string(),
            serde_json::Value::String(
                Self::stored_board(project_path, board)
                    .to_string_lossy()
                    .into_owned(),
            ),
        );
        let text = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
        std::fs::write(&path, text + "\n")
            .map_err(|e| format!("cannot write {}: {e}", path.display()))
    }
}

/// A project's complete HDL staged into its build directory: library sources
/// copied in, generated files written out, everything addressed by relative
/// filename so tool command lines stay simple.
pub struct StagedBuild {
    /// The build directory (see `build_dir`). The tools run in it and write
    /// their output files to it.
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
    /// The VHDL standard for each ghdl step.
    pub vhdl_std: String,
    /// Config's extra options for `synth_<family>`.
    pub synth_args: Vec<String>,
    /// Config's extra arguments for nextpnr.
    pub pnr_args: Vec<String>,
    /// The top-level ports that have a clock frequency in the config. When
    /// there is one, `dir` holds the clock constraints file `clocks.sdc`.
    pub clock_ports: Vec<String>,
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

/// The file that marks a directory as a build directory that hdlc made.
const BUILD_MARKER: &str = ".hdlc-build";

/// The build directory of a project: `build/<top>` next to the project file.
/// The intermediate files of a tool run stay there until the next run.
pub fn build_dir(project_dir: &Path, top_name: &str) -> PathBuf {
    project_dir.join("build").join(top_name)
}

/// Make `dir` an empty build directory. The directory is in the project of
/// the user, so this deletes it only when it is empty or holds the marker
/// file of an earlier run.
fn reset_build_dir(dir: &Path) -> Result<(), String> {
    if dir.exists() {
        let is_empty = dir
            .read_dir()
            .is_ok_and(|mut entries| entries.next().is_none());
        if !is_empty && !dir.join(BUILD_MARKER).is_file() {
            return Err(format!(
                "{} exists and hdlc did not make it (no {BUILD_MARKER} file); move it or delete it",
                dir.display()
            ));
        }
        std::fs::remove_dir_all(dir).map_err(|e| format!("cannot clean {}: {e}", dir.display()))?;
    }
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    std::fs::write(
        dir.join(BUILD_MARKER),
        "hdlc made this directory. hdlc deletes it at the start of each tool run.\n",
    )
    .map_err(|e| format!("cannot write {}: {e}", dir.join(BUILD_MARKER).display()))
}

/// Codegen the project and lay everything out in its build directory.
/// `project_dir` anchors the config's relative paths and the build directory.
pub fn stage(
    schematic: &Schematic,
    library: &[ModuleDef],
    design: &GeneratedDesign,
    config: &ToolchainConfig,
    project_dir: &Path,
) -> Result<StagedBuild, String> {
    let dir = build_dir(project_dir, &schematic.top_name);
    reset_build_dir(&dir)?;

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

    write_clocks(schematic, config, &dir)?;

    Ok(StagedBuild {
        dir,
        sources,
        libraries,
        ghdl_synth_flags: config.ghdl_synth_flags.clone(),
        vhdl_std: config.vhdl_std.clone().unwrap_or_else(|| "08".to_string()),
        synth_args: config.synth_args.clone(),
        pnr_args: config.pnr_args.clone(),
        clock_ports: config.clocks.keys().cloned().collect(),
        top_name: schematic.top_name.clone(),
        language: schematic.language.clone(),
    })
}

/// The file in the build directory with the clock constraints.
const SDC_FILE: &str = "clocks.sdc";

/// Write the clocks of the config as an SDC file in `dir`. nextpnr reads the
/// same file format for each FPGA family. A config without clocks gets no
/// file.
fn write_clocks(schematic: &Schematic, config: &ToolchainConfig, dir: &Path) -> Result<(), String> {
    if config.clocks.is_empty() {
        return Ok(());
    }
    let mut text = String::new();
    for (port, mhz) in &config.clocks {
        if !schematic.top_ports.iter().any(|p| &p.name == port) {
            return Err(format!(
                "clocks: '{port}' is not a top-level port of {}",
                schematic.top_name
            ));
        }
        if !(mhz.is_finite() && *mhz > 0.0) {
            return Err(format!(
                "clocks: the frequency of '{port}' must be more than 0 MHz"
            ));
        }
        text.push_str(&format!(
            "create_clock -period {:.3} [get_ports {{{port}}}]\n",
            1000.0 / mhz
        ));
    }
    std::fs::write(dir.join(SDC_FILE), text).map_err(|e| format!("cannot write {SDC_FILE}: {e}"))
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

/// Quote an argument for a POSIX shell. A shell that reads the result gets
/// the argument back.
fn sh_quote(arg: &str) -> String {
    let plain = !arg.is_empty()
        && arg
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_@%+=:,./-".contains(c));
    if plain {
        arg.to_string()
    } else {
        format!("'{}'", arg.replace('\'', r"'\''"))
    }
}

/// The step as a shell command line.
impl std::fmt::Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", sh_quote(&self.tool))?;
        for arg in &self.args {
            write!(f, " {}", sh_quote(arg))?;
        }
        if let Some(file) = &self.stdout_to {
            write!(f, " > {}", sh_quote(file))?;
        }
        Ok(())
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

/// The record of a tool run in the build directory: each command line and
/// its exit status.
const STEPS_LOG: &str = "steps.log";

/// Add a line to the record. The record is a help for the user; a run does
/// not fail when the line cannot be written.
fn record(dir: &Path, line: &str) {
    use std::io::Write;
    if let Ok(mut log) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(STEPS_LOG))
    {
        let _ = writeln!(log, "{line}");
    }
}

/// Run one step. Returns whether the tool exited successfully. A tool that
/// is not on PATH is an error, not a failed step. The command line goes to
/// the terminal and, with the exit status, to the record.
fn run_step(step: &Step, dir: &Path) -> Result<bool, String> {
    println!("$ {step}");
    record(dir, &format!("$ {step}"));
    let mut cmd = Command::new(&step.tool);
    cmd.args(&step.args).current_dir(dir);
    let spawn_error = |e: std::io::Error| {
        if e.kind() == std::io::ErrorKind::NotFound {
            format!("{} {PATH_HINT}", step.tool)
        } else {
            format!("failed to run {}: {e}", step.tool)
        }
    };
    let status = match &step.stdout_to {
        None => cmd.status().map_err(spawn_error)?,
        Some(file) => {
            let out = cmd.stderr(Stdio::inherit()).output().map_err(spawn_error)?;
            if out.status.success() {
                std::fs::write(dir.join(file), out.stdout)
                    .map_err(|e| format!("cannot write {file}: {e}"))?;
            }
            out.status
        }
    };
    record(
        dir,
        &match status.code() {
            Some(code) => format!("exit {code}"),
            None => "stopped by a signal".to_string(),
        },
    );
    Ok(status.success())
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

/// A step table as shell commands, for the user to read or to run by hand.
/// The first variant of each stage is a command line. The other variants
/// are comments, because the shell would run them only after a failure.
pub fn flow_script(stages: &[Stage], dir: &Path) -> String {
    let mut script = format!("cd {}\n", sh_quote(&dir.to_string_lossy()));
    for stage in stages {
        for (i, variant) in stage.variants.iter().enumerate() {
            if i == 0 {
                script.push_str(&format!("# {}\n", stage.name));
            } else {
                script.push_str(&format!(
                    "# {}, when a command above fails ({}):\n",
                    stage.name, variant.note
                ));
            }
            for step in &variant.steps {
                let prefix = if i == 0 { "" } else { "#   " };
                script.push_str(&format!("{prefix}{step}\n"));
            }
        }
    }
    script
}

/// The ghdl steps that analyze the named-library sources of the config
/// (`--work=<name>`). They must run before the work-library files. Empty
/// without a toolchain config.
fn library_steps(staged: &StagedBuild) -> Vec<Step> {
    staged
        .libraries
        .iter()
        .map(|(name, files)| {
            let mut args = vec!["-a".to_string(), std_flag(staged), format!("--work={name}")];
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

/// The ghdl option that selects the VHDL standard of the design.
fn std_flag(staged: &StagedBuild) -> String {
    format!("--std={}", staged.vhdl_std)
}

/// The ghdl step that analyzes every work source.
fn analyze_step(staged: &StagedBuild) -> Step {
    let mut args = vec!["-a".to_string(), std_flag(staged)];
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
                ["-e", &std_flag(staged), staged.top_name.as_str()],
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
    let mut args = vec!["synth".to_string(), std_flag(staged)];
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
    let mut flags = std_flag(staged) + " ";
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
            // yosys writes its messages to the terminal and to this file.
            let mut args = vec!["-l".to_string(), "yosys.log".to_string()];
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
fn pnr_step(staged: &StagedBuild, board: &crate::board::Board, constraints: &str) -> Step {
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
    args.extend(["--log", "nextpnr.log"].map(String::from));
    if !staged.clock_ports.is_empty() {
        args.extend(["--sdc", SDC_FILE].map(String::from));
    }
    args.extend(board.pnr_args.iter().cloned());
    args.extend(staged.pnr_args.iter().cloned());
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
    let synth_cmd = std::iter::once(synth_cmd)
        .chain(staged.synth_args.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" ");
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
    stages.push(Stage::single(
        "pnr",
        vec![pnr_step(staged, board, constraints)],
    ));
    stages.push(Stage::single("pack", vec![pack_step(board)]));
    stages
}

/// The step table of `build` for this host: `build_stages` with the
/// absolute path of the constraints file and the facts of the installed
/// tools.
fn build_flow(staged: &StagedBuild, board: &crate::board::Board) -> Result<Vec<Stage>, String> {
    let constraints = board
        .constraints
        .canonicalize()
        .map_err(|e| format!("cannot resolve {}: {e}", board.constraints.display()))?;
    let env = FlowEnv::detect(&staged.language);
    Ok(build_stages(
        staged,
        board,
        &constraints.to_string_lossy(),
        &env,
    ))
}

/// The commands that `run_build` runs, as a shell script. The paths are
/// absolute, so the script does not depend on the working directory. The
/// script does not make the chip view file; only `hdlc build` writes that.
pub fn build_script(
    staged: &StagedBuild,
    board: &crate::board::Board,
    out_bit: &Path,
) -> Result<String, String> {
    let absolute = |path: &Path| {
        std::path::absolute(path).map_err(|e| format!("cannot resolve {}: {e}", path.display()))
    };
    let mut script = flow_script(&build_flow(staged, board)?, &absolute(&staged.dir)?);
    script.push_str(&format!(
        "cp {BITSTREAM} {}\n",
        sh_quote(&absolute(out_bit)?.to_string_lossy())
    ));
    Ok(script)
}

/// Run `build_stages` in the build directory, then copy the bitstream to
/// `out_bit`. The timing report of nextpnr goes to the terminal.
pub fn run_build(
    staged: &StagedBuild,
    board: &crate::board::Board,
    out_bit: &Path,
) -> Result<bool, String> {
    if !run_stages(&build_flow(staged, board)?, &staged.dir)? {
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
            let std = std_flag(staged);
            let mut args = vec!["-a".to_string(), std.clone()];
            args.extend(staged.sources.iter().cloned());
            args.push(tb_file.clone());
            if !run_tool("ghdl", &args, &staged.dir)? {
                return Ok(None);
            }
            if !run_tool("ghdl", ["-e", &std, &tb_name], &staged.dir)? {
                return Ok(None);
            }
            let wave_arg = format!("--wave={}", wave.display());
            // Clock generators never stop on their own; bound the run.
            if !run_tool(
                "ghdl",
                ["-r", &std, &tb_name, &wave_arg, "--stop-time=1us"],
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

#[derive(Clone, Copy, serde::Deserialize, serde::Serialize)]
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
    /// The name of the board file: `<top>.board.json`.
    pub board_name: String,
    /// The board file as JSON text.
    pub board: String,
}

/// The files that start a board for a project: a Makefile that calls
/// `hdlc build` and `hdlc flash`, a board file with a usual part of the
/// family, and a constraints placeholder with the top-level ports. The
/// Makefile has no tool command of its own, so the flow has one definition.
pub fn fpga_scaffold(
    schematic: &Schematic,
    family: FpgaFamily,
    project_file_name: &str,
) -> FpgaScaffold {
    let top = &schematic.top_name;
    let (extension, device, package, hint) = match family {
        FpgaFamily::Ice40 => (
            "pcf",
            "up5k",
            "sg48",
            "ice40 devices: lp1k hx1k hx8k up5k u4k ...",
        ),
        FpgaFamily::Ecp5 => (
            "lpf",
            "25k",
            "CABGA256",
            "ecp5 devices: 12k 25k 45k 85k um5g-25k ...",
        ),
        FpgaFamily::Gowin => (
            "cst",
            "GW1NR-LV9QN88PC6/I5",
            "",
            "gowin: \"device\" is the full part number; \"pnr_args\" and \"pack_args\" name the die family",
        ),
    };
    let constraints_name = format!("{top}.{extension}");
    let board_name = format!("{top}.board.json");

    // nextpnr and gowin_pack both need the die family of a Gowin part.
    let gowin_family = |option: &str, value: &str| -> Vec<String> {
        match family {
            FpgaFamily::Gowin => vec![option.to_string(), value.to_string()],
            _ => Vec::new(),
        }
    };
    let board = crate::board::Board {
        name: top.clone(),
        family,
        device: device.to_string(),
        package: package.to_string(),
        constraints: PathBuf::from(&constraints_name),
        pack_args: gowin_family("-d", "GW1N-9C"),
        prog_args: Vec::new(),
        pnr_args: gowin_family("--vopt", "family=GW1N-9C"),
    };
    let board = serde_json::to_string_pretty(&board).expect("a board serializes") + "\n";

    let makefile = format!(
        "# Bitstream flow for {top} — generated by hdl-compose. Edit freely.\n\
         # Requires oss-cad-suite on PATH: source <oss-cad-suite>/environment\n\
         #\n\
         # `hdlc build` reads the board from {board_name}. Set \"device\" and\n\
         # \"package\" there for your part, fill in {constraints_name}, and put the\n\
         # flags of your programmer in \"prog_args\" (for example [\"-b\", \"<board>\"]).\n\
         # {hint}\n\
         # `hdlc build $(PROJECT) --dry-run` prints the tool commands.\n\
         \n\
         PROJECT := {project_file_name}\n\
         \n\
         all:\n\
         \thdlc build $(PROJECT)\n\
         \n\
         prog: all\n\
         \thdlc flash $(PROJECT)\n\
         \n\
         clean:\n\
         \trm -rf build\n\
         \n\
         .PHONY: all prog clean\n"
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
        constraints_name,
        constraints,
        board_name,
        board,
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
        let sc = fpga_scaffold(&s, FpgaFamily::Ice40, "blinky.hdlc");
        assert_eq!(sc.constraints_name, "blinky.pcf");
        assert!(sc.constraints.contains("# set_io clk <pin>"));
        assert!(sc.constraints.contains("# set_io led <pin>"));
        assert_eq!(sc.board_name, "blinky.board.json");
        let board: crate::board::Board = serde_json::from_str(&sc.board).unwrap();
        assert_eq!(board.name, "blinky");
        assert!(matches!(board.family, FpgaFamily::Ice40));
        assert_eq!(
            (board.device.as_str(), board.package.as_str()),
            ("up5k", "sg48")
        );
        assert_eq!(board.constraints, PathBuf::from("blinky.pcf"));
    }

    /// The Makefile calls hdlc. It must not hold a second copy of the flow.
    #[test]
    fn scaffold_makefile_calls_hdlc_and_runs_no_tool_itself() {
        let s = Schematic::new("blinky", Language::SystemVerilog);
        let sc = fpga_scaffold(&s, FpgaFamily::Ecp5, "blinky.hdlc");
        assert!(sc.makefile.contains("PROJECT := blinky.hdlc"));
        assert!(sc.makefile.contains("\thdlc build $(PROJECT)\n"));
        assert!(sc.makefile.contains("\thdlc flash $(PROJECT)\n"));
        assert!(sc.makefile.contains("--dry-run"));
        for tool in ["yosys", "nextpnr", "ecppack", "ghdl", "openFPGALoader"] {
            let runs_tool = sc
                .makefile
                .lines()
                .any(|line| !line.starts_with('#') && line.contains(tool));
            assert!(!runs_tool, "the Makefile runs {tool}");
        }
    }

    #[test]
    fn ecp5_scaffold_board() {
        let s = Schematic::new("blinky", Language::SystemVerilog);
        let sc = fpga_scaffold(&s, FpgaFamily::Ecp5, "blinky.hdlc");
        assert_eq!(sc.constraints_name, "blinky.lpf");
        let board: crate::board::Board = serde_json::from_str(&sc.board).unwrap();
        assert!(matches!(board.family, FpgaFamily::Ecp5));
        assert_eq!(
            (board.device.as_str(), board.package.as_str()),
            ("25k", "CABGA256")
        );
        assert_eq!(board.constraints, PathBuf::from("blinky.lpf"));
        assert!(board.pnr_args.is_empty() && board.pack_args.is_empty());
    }

    /// nextpnr and gowin_pack both need the die family of a Gowin part.
    #[test]
    fn gowin_scaffold_board_names_the_die_family_for_nextpnr_and_pack() {
        let s = Schematic::new("blinky", Language::Vhdl);
        let sc = fpga_scaffold(&s, FpgaFamily::Gowin, "blinky.hdlc");
        assert_eq!(sc.constraints_name, "blinky.cst");
        let board: crate::board::Board = serde_json::from_str(&sc.board).unwrap();
        assert_eq!(board.device, "GW1NR-LV9QN88PC6/I5");
        assert_eq!(board.pnr_args, ["--vopt", "family=GW1N-9C"]);
        assert_eq!(board.pack_args, ["-d", "GW1N-9C"]);
    }
}

#[cfg(test)]
mod doctor_tests {
    use super::*;

    #[test]
    fn version_is_the_first_line_with_text() {
        assert_eq!(
            first_line(b"\n\n  GHDL 7.0.0-dev \nmore\n"),
            "GHDL 7.0.0-dev"
        );
        assert_eq!(first_line(b""), "");
    }

    /// nextpnr puts its version at the end of a long line, after the cut.
    #[test]
    fn version_in_parentheses_replaces_the_line() {
        assert_eq!(
            first_line(
                b"\"nextpnr-ecp5\" -- Next Generation Place and Route (Version nextpnr-0.10-100-gfb95bb8e)\n"
            ),
            "nextpnr-0.10-100-gfb95bb8e"
        );
    }

    #[test]
    fn a_tool_is_found_in_the_first_directory_that_has_it() {
        let (a, b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        std::fs::write(b.path().join("yosys"), "").unwrap();
        let dirs = || [a.path().to_path_buf(), b.path().to_path_buf()].into_iter();
        assert_eq!(find_in(dirs(), "yosys"), Some(b.path().join("yosys")));
        assert_eq!(find_in(dirs(), "ghdl"), None);
        // A directory with the name of the tool is not the tool.
        std::fs::create_dir(a.path().join("ghdl")).unwrap();
        assert_eq!(find_in(dirs(), "ghdl"), None);
    }

    #[test]
    fn table_has_one_aligned_line_for_each_tool_and_a_hint_for_a_missing_tool() {
        let reports = [
            ToolReport {
                name: "ghdl",
                found: Some("GHDL 7.0.0-dev".to_string()),
                used_by: "check (VHDL)",
            },
            ToolReport {
                name: "nextpnr-ecp5",
                found: None,
                used_by: "build (ecp5)",
            },
        ];
        assert_eq!(
            doctor_table(&reports),
            "ghdl          GHDL 7.0.0-dev  check (VHDL)\n\
             nextpnr-ecp5  missing         build (ecp5)\n\
             \n\
             A missing tool is not on PATH. Activate oss-cad-suite first: source <oss-cad-suite>/environment\n"
        );
        assert!(!doctor_table(&reports[..1]).contains("missing"));
    }

    #[test]
    fn a_long_version_line_is_cut() {
        let reports = [ToolReport {
            name: "yosys",
            found: Some("x".repeat(200)),
            used_by: "synth",
        }];
        let line = doctor_table(&reports);
        assert_eq!(line, format!("yosys  {}  synth\n", "x".repeat(60)));
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
            vhdl_std: "08".to_string(),
            synth_args: Vec::new(),
            pnr_args: Vec::new(),
            clock_ports: Vec::new(),
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
            pnr_args: Vec::new(),
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
            plugin.args[5],
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
                                "-l",
                                "yosys.log",
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
                                    "-l",
                                    "yosys.log",
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
                        "-l",
                        "yosys.log",
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
                    "-l",
                    "yosys.log",
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
                    "-l",
                    "yosys.log",
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
                        "--log",
                        "nextpnr.log",
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
                        "-l",
                        "yosys.log",
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
                    "--log",
                    "nextpnr.log",
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
                    "--log",
                    "nextpnr.log",
                ]
            )
        );
        assert_eq!(
            stages[2].variants[0].steps[0],
            step("gowin_pack", &["--compress", "-o", "out.bit", "pnr.json"])
        );
    }

    #[test]
    fn vhdl_std_goes_to_each_ghdl_step() {
        let mut s = staged(Language::Vhdl);
        s.vhdl_std = "93".to_string();
        s.libraries = vec![("loot".to_string(), vec!["pkg.vhd".to_string()])];

        let check = &check_stages(&s)[0].variants[0].steps;
        assert_eq!(check[0].args[..3], ["-a", "--std=93", "--work=loot"]);
        assert_eq!(check[1].args[..2], ["-a", "--std=93"]);
        assert_eq!(check[2].args, ["-e", "--std=93", "top"]);

        let synth = &synth_stages(&s, &env())[1];
        let script = synth.variants[0].steps[0].args.last().unwrap();
        assert!(script.starts_with("ghdl --std=93 "), "{script}");
        assert_eq!(synth.variants[1].steps[0].args[..2], ["-a", "--std=93"]);
        assert_eq!(synth.variants[1].steps[1].args[..2], ["synth", "--std=93"]);
    }

    /// `synth_args` are options of `synth_<family>`. The generic `synth`
    /// command does not know them all, so `hdlc synth` does not get them.
    #[test]
    fn synth_args_go_to_the_family_synth_command_only() {
        let mut s = staged(Language::Vhdl);
        s.synth_args = vec!["-abc9".to_string(), "-retime".to_string()];

        let build = build_stages(
            &s,
            &board(FpgaFamily::Ecp5, "25k", "CABGA256"),
            "/abs/board.lpf",
            &env(),
        );
        let scripts: Vec<&String> = build[0]
            .variants
            .iter()
            .map(|v| v.steps.last().unwrap().args.last().unwrap())
            .collect();
        assert!(
            scripts[0].ends_with("-e top; synth_ecp5 -abc9 -retime -json design.json"),
            "{}",
            scripts[0]
        );
        assert_eq!(
            scripts[1],
            "read_verilog design_net.v; synth_ecp5 -abc9 -retime -top top -json design.json"
        );

        let generic = &synth_stages(&s, &env())[0].variants[0].steps[0];
        assert!(
            generic
                .args
                .last()
                .unwrap()
                .ends_with("-e top; synth; stat")
        );
    }

    #[test]
    fn nextpnr_gets_the_clocks_then_the_arguments_of_the_board_then_of_the_project() {
        let mut s = staged(Language::SystemVerilog);
        s.clock_ports = vec!["clk".to_string()];
        s.pnr_args = vec!["--seed".to_string(), "3".to_string()];
        let mut b = board(FpgaFamily::Gowin, "GW1NR-LV9QN88PC6/I5", "");
        b.pnr_args = vec!["--vopt".to_string(), "family=GW1N-9C".to_string()];

        let stages = build_stages(&s, &b, "/abs/board.cst", &FlowEnv::default());
        let args = &stages[1].variants[0].steps[0].args;
        assert_eq!(
            args[args.len() - 8..],
            [
                "--log",
                "nextpnr.log",
                "--sdc",
                "clocks.sdc",
                "--vopt",
                "family=GW1N-9C",
                "--seed",
                "3"
            ]
        );
    }

    #[test]
    fn script_has_the_first_variant_as_commands_and_the_others_as_comments() {
        let stages = [
            Stage {
                name: "synth",
                variants: vec![
                    Variant {
                        note: "",
                        steps: vec![step("yosys", &["-p", "synth; stat"])],
                    },
                    Variant {
                        note: "plugin failed",
                        steps: vec![
                            Step {
                                stdout_to: Some("n.v".to_string()),
                                ..step("ghdl", &["synth", "top"])
                            },
                            step("yosys", &["-p", "read_verilog n.v"]),
                        ],
                    },
                ],
            },
            single("pack", vec![step("ecppack", &["pnr.config", "out.bit"])]),
        ];
        assert_eq!(
            flow_script(&stages, Path::new("/proj/build/my top")),
            "cd '/proj/build/my top'\n\
             # synth\n\
             yosys -p 'synth; stat'\n\
             # synth, when a command above fails (plugin failed):\n\
             #   ghdl synth top > n.v\n\
             #   yosys -p 'read_verilog n.v'\n\
             # pack\n\
             ecppack pnr.config out.bit\n"
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
        fn steps_log_records_each_command_and_its_exit_status() {
            let dir = tempfile::tempdir().unwrap();
            let echo = Step {
                stdout_to: Some("out.txt".to_string()),
                ..step("echo", &["two words", "plain"])
            };
            let stages = [variants(&["false", "true"]), single("echo", vec![echo])];
            assert_eq!(run_stages(&stages, dir.path()), Ok(true));
            assert_eq!(
                std::fs::read_to_string(dir.path().join(STEPS_LOG)).unwrap(),
                "$ false\nexit 1\n$ true\nexit 0\n$ echo 'two words' plain > out.txt\nexit 0\n"
            );
        }

        proptest::proptest! {
            /// A shell that reads a quoted argument gets the argument back.
            #[test]
            fn sh_quote_round_trips_through_a_shell(arg in "[^\\x00]{0,24}") {
                let out = Command::new("sh")
                    .args(["-c", &format!("printf %s {}", sh_quote(&arg))])
                    .output()
                    .unwrap();
                proptest::prop_assert_eq!(String::from_utf8_lossy(&out.stdout), arg);
            }
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

    #[test]
    fn stage_honors_config_order_libraries_and_excludes() {
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
    }

    /// A project directory with one generated file, staged with no config.
    fn stage_in(project_dir: &Path) -> Result<StagedBuild, String> {
        let design = GeneratedDesign {
            files: vec![],
            top_filename: "top.vhd".into(),
            top_code: "-- top".into(),
        };
        stage(
            &Schematic::new("top", Language::Vhdl),
            &[],
            &design,
            &ToolchainConfig::default(),
            project_dir,
        )
    }

    #[test]
    fn stage_writes_to_build_top_next_to_the_project() {
        let project = tempfile::tempdir().unwrap();
        let staged = stage_in(project.path()).unwrap();
        assert_eq!(staged.dir, project.path().join("build").join("top"));
        assert_eq!(staged.dir, build_dir(project.path(), "top"));
        assert!(staged.dir.join("top.vhd").is_file());
        assert!(staged.dir.join(BUILD_MARKER).is_file());
    }

    #[test]
    fn stage_replaces_the_build_directory_of_the_run_before() {
        let project = tempfile::tempdir().unwrap();
        let staged = stage_in(project.path()).unwrap();
        std::fs::write(staged.dir.join("design.json"), "{}").unwrap();

        let staged = stage_in(project.path()).unwrap();
        assert!(!staged.dir.join("design.json").exists());
        assert!(staged.dir.join("top.vhd").is_file());
    }

    #[test]
    fn stage_uses_an_empty_directory() {
        let project = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(build_dir(project.path(), "top")).unwrap();
        assert!(stage_in(project.path()).is_ok());
    }

    /// The build directory is in the project of the user. `stage` must not
    /// delete a directory there that it did not make.
    #[test]
    fn stage_does_not_delete_a_directory_that_hdlc_did_not_make() {
        let project = tempfile::tempdir().unwrap();
        let dir = build_dir(project.path(), "top");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("notes.txt"), "mine").unwrap();

        let error = stage_in(project.path()).err().unwrap();
        assert!(error.contains("hdlc did not make"), "{error}");
        assert_eq!(
            std::fs::read_to_string(dir.join("notes.txt")).unwrap(),
            "mine"
        );
    }

    fn clock_project(clocks: &str) -> (tempfile::TempDir, Result<StagedBuild, String>) {
        let project = tempfile::tempdir().unwrap();
        let mut schematic = Schematic::new("top", Language::Vhdl);
        schematic.top_ports = vec![crate::types::PortDef {
            name: "clk".to_string(),
            direction: Direction::In,
            port_type: PortType::StdLogic,
            bundle: None,
        }];
        let design = GeneratedDesign {
            files: vec![],
            top_filename: "top.vhd".into(),
            top_code: "-- top".into(),
        };
        let config: ToolchainConfig =
            serde_json::from_str(&format!(r#"{{"clocks": {clocks}}}"#)).unwrap();
        let staged = stage(&schematic, &[], &design, &config, project.path());
        (project, staged)
    }

    #[test]
    fn clocks_become_an_sdc_file_in_the_build_directory() {
        let (_project, staged) = clock_project(r#"{"clk": 50}"#);
        let staged = staged.unwrap();
        assert_eq!(staged.clock_ports, ["clk"]);
        assert_eq!(
            std::fs::read_to_string(staged.dir.join("clocks.sdc")).unwrap(),
            "create_clock -period 20.000 [get_ports {clk}]\n"
        );
    }

    #[test]
    fn a_project_without_clocks_has_no_sdc_file() {
        let (_project, staged) = clock_project("{}");
        let staged = staged.unwrap();
        assert!(staged.clock_ports.is_empty());
        assert!(!staged.dir.join("clocks.sdc").exists());
    }

    #[test]
    fn a_clock_on_a_name_that_is_not_a_top_level_port_is_an_error() {
        let (_project, staged) = clock_project(r#"{"clock": 50}"#);
        let error = staged.err().unwrap();
        assert!(error.contains("'clock' is not a top-level port"), "{error}");
    }

    #[test]
    fn a_clock_frequency_must_be_more_than_zero() {
        for bad in ["0", "-12.5"] {
            let (_project, staged) = clock_project(&format!(r#"{{"clk": {bad}}}"#));
            let error = staged.err().unwrap();
            assert!(error.contains("must be more than 0 MHz"), "{error}");
        }
    }

    #[test]
    fn config_defaults_keep_the_flow_of_a_project_without_a_sidecar() {
        let project = tempfile::tempdir().unwrap();
        let staged = stage_in(project.path()).unwrap();
        assert_eq!(staged.vhdl_std, "08");
        assert!(staged.synth_args.is_empty() && staged.pnr_args.is_empty());
        assert!(staged.clock_ports.is_empty());
    }

    #[test]
    fn board_path_is_relative_to_the_project_directory() {
        let config: ToolchainConfig =
            serde_json::from_str(r#"{"board": "boards/x.board.json"}"#).unwrap();
        assert_eq!(
            config.board_path(Path::new("proj")),
            Some(PathBuf::from("proj/boards/x.board.json"))
        );
        let config: ToolchainConfig =
            serde_json::from_str(r#"{"board": "/abs/x.board.json"}"#).unwrap();
        assert_eq!(
            config.board_path(Path::new("proj")),
            Some(PathBuf::from("/abs/x.board.json"))
        );
        assert_eq!(
            ToolchainConfig::default().board_path(Path::new("proj")),
            None
        );
    }

    #[test]
    fn set_board_makes_the_sidecar_and_stores_a_path_below_the_project_as_relative() {
        let project = tempfile::tempdir().unwrap();
        let hdlc = project.path().join("demo.hdlc");
        let board = project.path().join("boards").join("x.board.json");

        ToolchainConfig::set_board(&hdlc, &board).unwrap();

        let text = std::fs::read_to_string(project.path().join("demo.toolchain.json")).unwrap();
        assert_eq!(text, "{\n  \"board\": \"boards/x.board.json\"\n}\n");
        let config = ToolchainConfig::load_for(&hdlc).unwrap();
        assert_eq!(config.board_path(project.path()), Some(board));
    }

    #[test]
    fn stored_board_is_relative_to_the_project_directory() {
        let stored = |project: &str, board: &str| {
            ToolchainConfig::stored_board(Path::new(project), Path::new(board))
        };
        assert_eq!(
            stored("/p/demo.hdlc", "/p/b/x.board.json"),
            Path::new("b/x.board.json")
        );
        assert_eq!(
            stored("/p/demo.hdlc", "/q/x.board.json"),
            Path::new("/q/x.board.json")
        );
        // A project in the working directory: the board path is already
        // relative to the project directory.
        assert_eq!(
            stored("demo.hdlc", "./x.board.json"),
            Path::new("x.board.json")
        );
        assert_eq!(
            stored("demo.hdlc", "b/x.board.json"),
            Path::new("b/x.board.json")
        );
    }

    #[test]
    fn set_board_keeps_a_path_outside_the_project_as_given() {
        let project = tempfile::tempdir().unwrap();
        let hdlc = project.path().join("demo.hdlc");
        ToolchainConfig::set_board(&hdlc, Path::new("/elsewhere/x.board.json")).unwrap();
        let config = ToolchainConfig::load_for(&hdlc).unwrap();
        assert_eq!(config.board, Some(PathBuf::from("/elsewhere/x.board.json")));
    }

    #[test]
    fn set_board_refuses_a_sidecar_that_is_not_an_object() {
        let project = tempfile::tempdir().unwrap();
        let hdlc = project.path().join("demo.hdlc");
        let sidecar = project.path().join("demo.toolchain.json");
        std::fs::write(&sidecar, "[1, 2]").unwrap();
        assert!(ToolchainConfig::set_board(&hdlc, Path::new("x.board.json")).is_err());
        assert_eq!(std::fs::read_to_string(&sidecar).unwrap(), "[1, 2]");
    }

    proptest::proptest! {
        /// `set_board` changes the `board` key only. Each other key of the
        /// sidecar, also one that hdlc does not know, keeps its value.
        #[test]
        fn set_board_keeps_each_other_key(
            unknown in proptest::collection::btree_map("x_[a-z]{0,6}", "[ -~]{0,12}", 0..5),
            flags in proptest::collection::vec("--[a-z]{1,8}", 0..3),
            old_board in proptest::option::of("[a-z]{1,8}\\.board\\.json"),
            new_board in "[a-z]{1,8}\\.board\\.json",
        ) {
            let project = tempfile::tempdir().unwrap();
            let hdlc = project.path().join("demo.hdlc");
            let sidecar = project.path().join("demo.toolchain.json");
            let mut before = serde_json::Map::new();
            for (key, value) in &unknown {
                before.insert(key.clone(), serde_json::Value::from(value.clone()));
            }
            before.insert("ghdl_synth_flags".to_string(), serde_json::Value::from(flags.clone()));
            if let Some(board) = &old_board {
                before.insert("board".to_string(), serde_json::Value::from(board.clone()));
            }
            std::fs::write(&sidecar, serde_json::to_string(&before).unwrap()).unwrap();

            ToolchainConfig::set_board(&hdlc, Path::new(&new_board)).unwrap();

            let text = std::fs::read_to_string(&sidecar).unwrap();
            let mut after: serde_json::Map<String, serde_json::Value> =
                serde_json::from_str(&text).unwrap();
            proptest::prop_assert_eq!(
                after.remove("board"),
                Some(serde_json::Value::from(new_board.clone()))
            );
            before.remove("board");
            proptest::prop_assert_eq!(after, before);
            let config = ToolchainConfig::load_for(&hdlc).unwrap();
            proptest::prop_assert_eq!(config.board, Some(PathBuf::from(&new_board)));
            proptest::prop_assert_eq!(config.ghdl_synth_flags, flags);
        }
    }

    #[test]
    fn missing_config_is_default() {
        let cfg = ToolchainConfig::load_for(Path::new("/nonexistent/proj.hdlc")).unwrap();
        assert!(cfg.vhdl_libraries.is_empty() && cfg.extra_sources.is_empty());
    }
}
