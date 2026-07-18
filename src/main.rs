use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use tracing::{debug, error, info, warn};

use hdl_compose::codegen;
use hdl_compose::project;
use hdl_compose::schematic::DiagnosticLevel;
use hdl_compose::types::{Language, Schematic};

#[derive(Parser)]
#[command(
    name = "hdl-compose",
    version,
    about = "Structural HDL composition tool"
)]
struct Cli {
    /// Enable verbose (debug) output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Parse an HDL file and print extracted module definitions
    Parse {
        /// Path to HDL source file (.vhd, .vhdl, .v, .sv)
        file: PathBuf,
    },

    /// Create a new empty .hdlc project file
    New {
        /// Project name (creates <name>.hdlc)
        name: String,

        /// Target HDL language
        #[arg(short, long)]
        language: LangArg,

        /// Populate the project with a working example: two wired modules
        /// (with HDL sources written alongside), a constant tie, a net
        /// alias, and a top-generic passthrough
        #[arg(long)]
        example: bool,
    },

    /// Validate a .hdlc project and report diagnostics.
    /// Exit code: 0 = clean, 1 = warnings only, 2 = errors.
    Validate {
        /// Path to .hdlc project file
        project: PathBuf,
    },

    /// Generate structural HDL from a .hdlc project
    Codegen {
        /// Path to .hdlc project file
        project: PathBuf,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Print a summary of a .hdlc project
    Inspect {
        /// Path to .hdlc project file
        project: PathBuf,
    },

    /// Migrate .hdlc project files to the current format version (in place)
    Migrate {
        /// Paths to .hdlc project files
        projects: Vec<PathBuf>,
    },

    /// Print the JSON Schema for the .hdlc project file format
    Schema {
        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Elaborate the generated HDL with a real frontend (ghdl / verilator).
    /// Requires oss-cad-suite (or the tools) on PATH.
    /// Exit code: 0 = clean, 1 = tool reported errors, 2 = tool missing or codegen failed.
    Check {
        /// Path to .hdlc project file
        project: PathBuf,
    },

    /// Run a generic yosys synthesis of the generated design and print stats.
    /// Requires oss-cad-suite (yosys, plus ghdl plugin for VHDL) on PATH.
    Synth {
        /// Path to .hdlc project file
        project: PathBuf,
    },

    /// Simulate the design via <top>_tb (a skeleton is generated next to the
    /// project if missing) using ghdl / iverilog. Requires oss-cad-suite on PATH.
    Sim {
        /// Path to .hdlc project file
        project: PathBuf,

        /// Open the wave file in surfer/gtkwave after the run
        #[arg(long)]
        wave: bool,
    },

    /// Emit an FPGA build Makefile + constraints skeleton
    /// (yosys → nextpnr → pack → openFPGALoader) next to the project
    Fpga {
        /// Path to .hdlc project file
        project: PathBuf,

        /// Target FPGA family
        #[arg(long)]
        family: FamilyArg,

        /// Output directory (default: the project's directory)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Overwrite an existing Makefile / constraints file
        #[arg(long)]
        force: bool,
    },

    /// Launch the Qt GUI (default when no subcommand is given)
    Gui,
}

#[derive(Clone, ValueEnum)]
enum FamilyArg {
    Ice40,
    Ecp5,
    Gowin,
}

impl From<FamilyArg> for hdl_compose::toolchain::FpgaFamily {
    fn from(f: FamilyArg) -> Self {
        use hdl_compose::toolchain::FpgaFamily;
        match f {
            FamilyArg::Ice40 => FpgaFamily::Ice40,
            FamilyArg::Ecp5 => FpgaFamily::Ecp5,
            FamilyArg::Gowin => FpgaFamily::Gowin,
        }
    }
}

#[derive(Clone, ValueEnum)]
enum LangArg {
    Vhdl,
    Sv,
}

impl From<LangArg> for Language {
    fn from(l: LangArg) -> Self {
        match l {
            LangArg::Vhdl => Language::Vhdl,
            LangArg::Sv => Language::SystemVerilog,
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let level = if cli.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::WARN
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_writer(std::io::stderr)
        .init();

    match cli.command {
        None | Some(Command::Gui) => cmd_gui(),
        Some(Command::Parse { file }) => cmd_parse(&file),
        Some(Command::New {
            name,
            language,
            example,
        }) => cmd_new(&name, language.into(), example),
        Some(Command::Validate { project }) => cmd_validate(&project),
        Some(Command::Codegen { project, output }) => cmd_codegen(&project, output.as_deref()),
        Some(Command::Inspect { project }) => cmd_inspect(&project),
        Some(Command::Migrate { projects }) => cmd_migrate(&projects),
        Some(Command::Schema { output }) => cmd_schema(output.as_deref()),
        Some(Command::Check { project }) => cmd_check(&project),
        Some(Command::Synth { project }) => cmd_synth(&project),
        Some(Command::Sim { project, wave }) => cmd_sim(&project, wave),
        Some(Command::Fpga {
            project,
            family,
            output,
            force,
        }) => cmd_fpga(&project, family.into(), output.as_deref(), force),
    }
}

const EXAMPLE_PULSE_GEN_VHDL: &str = r#"library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

entity pulse_gen is
  generic (
    WIDTH : integer := 8;
    DIV   : integer := 1000
  );
  port (
    clk   : in  std_logic;
    ena   : in  std_logic;
    count : out std_logic_vector(WIDTH-1 downto 0)
  );
end entity pulse_gen;

architecture rtl of pulse_gen is
  signal cnt : unsigned(WIDTH-1 downto 0) := (others => '0');
begin
  process (clk)
  begin
    if rising_edge(clk) then
      if ena = '1' then
        cnt <= cnt + 1;
      end if;
    end if;
  end process;
  count <= std_logic_vector(cnt);
end architecture rtl;
"#;

const EXAMPLE_LED_DRIVER_VHDL: &str = r#"library ieee;
use ieee.std_logic_1164.all;

entity led_driver is
  port (
    clk   : in  std_logic;
    value : in  std_logic_vector(7 downto 0);
    led   : out std_logic
  );
end entity led_driver;

architecture rtl of led_driver is
begin
  led <= value(7);
end architecture rtl;
"#;

const EXAMPLE_PULSE_GEN_SV: &str = r#"module pulse_gen #(
  parameter int WIDTH = 8,
  parameter int DIV   = 1000
) (
  input  logic             clk,
  input  logic             ena,
  output logic [WIDTH-1:0] count
);
  always_ff @(posedge clk)
    if (ena) count <= count + 1'b1;
endmodule
"#;

const EXAMPLE_LED_DRIVER_SV: &str = r#"module led_driver (
  input  logic       clk,
  input  logic [7:0] value,
  output logic       led
);
  assign led = value[7];
endmodule
"#;

/// Build the `new --example` project: writes the two library HDL sources
/// next to the project file and returns a schematic that exercises the
/// format — an instance-to-instance net with an alias, top-port routing, a
/// constant tie, and a top-generic passthrough into a generic map.
fn build_example_project(name: &str, language: &Language) -> Result<Schematic, String> {
    use hdl_compose::types::{Direction, GenericDef, NetRef, PortDef, PortType};

    let (ext, pulse_src, led_src, ena_const) = match language {
        Language::Vhdl => ("vhd", EXAMPLE_PULSE_GEN_VHDL, EXAMPLE_LED_DRIVER_VHDL, "'1'"),
        Language::SystemVerilog => ("sv", EXAMPLE_PULSE_GEN_SV, EXAMPLE_LED_DRIVER_SV, "1'b1"),
    };
    let pulse_path = PathBuf::from(format!("pulse_gen.{ext}"));
    let led_path = PathBuf::from(format!("led_driver.{ext}"));
    for p in [&pulse_path, &led_path] {
        if p.exists() {
            return Err(format!("'{}' already exists", p.display()));
        }
    }
    std::fs::write(&pulse_path, pulse_src).map_err(|e| e.to_string())?;
    std::fs::write(&led_path, led_src).map_err(|e| e.to_string())?;

    let mut s = Schematic::new(name, language.clone());
    s.top_generics.push(GenericDef {
        name: "CLK_DIV".into(),
        type_name: "integer".into(),
        default_value: Some("1000".into()),
    });
    s.top_ports.push(PortDef {
        name: "clk".into(),
        direction: Direction::In,
        port_type: PortType::StdLogic,
        bundle: None,
    });
    s.top_ports.push(PortDef {
        name: "led".into(),
        direction: Direction::Out,
        port_type: PortType::StdLogic,
        bundle: None,
    });
    s.library_paths.push(pulse_path);
    s.library_paths.push(led_path);

    {
        let inst = s.add_instance("u_pulse", "pulse_gen").map_err(|e| e.to_string())?;
        inst.position = (80.0, 80.0);
        // Top-generic passthrough: DIV follows the top-level CLK_DIV.
        inst.generic_map.insert("DIV".into(), "CLK_DIV".into());
        inst.port_map
            .insert("clk".into(), Some(NetRef::TopPort("clk".into())));
        // Constant tie instead of an illegal open input.
        inst.port_map
            .insert("ena".into(), Some(NetRef::Constant(ena_const.into())));
    }
    {
        let inst = s.add_instance("u_led", "led_driver").map_err(|e| e.to_string())?;
        inst.position = (360.0, 80.0);
        inst.port_map
            .insert("clk".into(), Some(NetRef::TopPort("clk".into())));
        // Reference direction is free — this also connects u_pulse.count.
        inst.port_map.insert(
            "value".into(),
            Some(NetRef::InstancePort("u_pulse".into(), "count".into())),
        );
        inst.port_map
            .insert("led".into(), Some(NetRef::TopPort("led".into())));
    }
    // Net alias: the generated signal is named pulse_count, not u_pulse_count.
    s.set_alias(
        NetRef::InstancePort("u_pulse".into(), "count".into()),
        "pulse_count",
    );
    Ok(s)
}

/// Load each project (load_project applies in-process migrations, e.g. v3 →
/// v4 fills default `consumer_slices` / `manual_bundles`) and save it back at
/// CURRENT_VERSION. Idempotent: files already current are rewritten as-is.
fn cmd_migrate(projects: &[PathBuf]) -> ExitCode {
    if projects.is_empty() {
        error!("usage: hdl-compose migrate <path.hdlc> [<path.hdlc> ...]");
        return ExitCode::from(2);
    }
    let mut failed = 0;
    for path in projects {
        match project::load_project(path) {
            Ok((schematic, warnings)) => {
                for w in &warnings {
                    warn!("{}: {w}", path.display());
                }
                if let Err(e) = project::save_project(&schematic, path) {
                    error!("{}: save failed: {e}", path.display());
                    failed += 1;
                } else {
                    println!("migrated {}", path.display());
                }
            }
            Err(e) => {
                error!("{}: load failed: {e}", path.display());
                failed += 1;
            }
        }
    }
    if failed > 0 { ExitCode::from(1) } else { ExitCode::SUCCESS }
}

fn cmd_gui() -> ExitCode {
    let code = hdl_compose::gui::run();
    if code == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(code as u8)
    }
}

fn cmd_parse(file: &std::path::Path) -> ExitCode {
    debug!("Parsing file: {}", file.display());

    match hdl_compose::parse_file(file) {
        Ok(modules) => {
            if modules.is_empty() {
                println!("No modules found in {}", file.display());
                return ExitCode::SUCCESS;
            }
            for module in &modules {
                println!("module {}", module.name);

                if !module.generics.is_empty() {
                    println!("  generics:");
                    for g in &module.generics {
                        if let Some(default) = &g.default_value {
                            println!("    {} : {} := {}", g.name, g.type_name, default);
                        } else {
                            println!("    {} : {}", g.name, g.type_name);
                        }
                    }
                }

                if !module.ports.is_empty() {
                    println!("  ports:");
                    for p in &module.ports {
                        let dir = match p.direction {
                            hdl_compose::types::Direction::In => "in",
                            hdl_compose::types::Direction::Out => "out",
                            hdl_compose::types::Direction::InOut => "inout",
                        };
                        let type_str = codegen::port_type_to_vhdl(&p.port_type);
                        println!("    {} : {} {}", p.name, dir, type_str);
                    }
                }
                println!();
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            error!("{e}");
            eprintln!("error: {e}");
            ExitCode::from(2)
        }
    }
}

fn cmd_schema(output: Option<&std::path::Path>) -> ExitCode {
    let schema = project::hdlc_schema_json();
    match output {
        Some(path) => match std::fs::write(path, &schema) {
            Ok(()) => {
                println!("Written to {}", path.display());
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::from(2)
            }
        },
        None => {
            println!("{schema}");
            ExitCode::SUCCESS
        }
    }
}

fn cmd_new(name: &str, language: Language, example: bool) -> ExitCode {
    let filename = format!("{name}.hdlc");
    let path = PathBuf::from(&filename);

    if path.exists() {
        eprintln!("error: '{filename}' already exists");
        return ExitCode::FAILURE;
    }

    let lang_str = match language {
        Language::Vhdl => "VHDL",
        Language::SystemVerilog => "SystemVerilog",
    };

    let schematic = if example {
        match build_example_project(name, &language) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        Schematic::new(name, language)
    };

    match project::save_project(&schematic, &path) {
        Ok(()) => {
            info!("Created project: {filename}");
            println!("Created {lang_str} project: {filename}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            error!("{e}");
            eprintln!("error: {e}");
            ExitCode::from(2)
        }
    }
}

fn cmd_validate(project_path: &std::path::Path) -> ExitCode {
    let (schematic, load_warnings) = match load_project(project_path) {
        Ok(v) => v,
        Err(code) => return code,
    };

    for w in &load_warnings {
        warn!("{w}");
        eprintln!("warning: {w}");
    }

    let (library, lib_errors) = schematic.resolve_modules();
    if !lib_errors.is_empty() {
        for (path, e) in &lib_errors {
            error!("{}: {}", path.display(), e);
            eprintln!("error: failed to parse {}: {}", path.display(), e);
        }
        return ExitCode::from(2);
    }

    let mut diagnostics = schematic.validate(&library);
    diagnostics.extend(hdl_compose::groups::validate_groups(&schematic, &library));

    if diagnostics.is_empty() {
        println!("No errors.");
        return ExitCode::SUCCESS;
    }

    let mut has_errors = false;
    for d in &diagnostics {
        println!("{d}");
        if d.level == DiagnosticLevel::Error {
            has_errors = true;
        }
    }

    // 0 = clean, 1 = warnings only, 2 = errors — scriptable severity split.
    if has_errors {
        ExitCode::from(2)
    } else {
        ExitCode::from(1)
    }
}

/// Emit a grouped project: each group becomes its own source file next to
/// the top module's output (`<group>.<ext>`); without `-o`, everything is
/// concatenated to stdout with separator comments.
/// Print a `DesignError` and map it to the CLI exit code. Grouped-project
/// failures exit 2; flat-project codegen failures keep the historical exit 1.
fn report_design_error(e: codegen::DesignError, grouped: bool) -> ExitCode {
    match e {
        codegen::DesignError::GroupDiagnostics(diags) => {
            for d in &diags {
                eprintln!("error: {d}");
            }
            ExitCode::from(2)
        }
        codegen::DesignError::Group { name, source } => {
            eprintln!("error: group '{name}': {source}");
            ExitCode::from(2)
        }
        codegen::DesignError::Top(e) if grouped => {
            eprintln!("error: {e}");
            ExitCode::from(2)
        }
        codegen::DesignError::Top(codegen::CodegenError::ValidationErrors(errs)) => {
            eprintln!("error: schematic has validation errors");
            for d in &errs {
                eprintln!("  {d}");
            }
            ExitCode::FAILURE
        }
        codegen::DesignError::Top(codegen::CodegenError::DirtyInstances(names)) => {
            eprintln!(
                "error: dirty instances present (source re-parse dropped \
                 connections). Review and reconnect: {}",
                names.join(", ")
            );
            ExitCode::FAILURE
        }
    }
}

/// Load a project and resolve its module library, printing warnings/errors.
fn load_project_and_library(
    project_path: &std::path::Path,
) -> Result<(Schematic, Vec<hdl_compose::types::ModuleDef>), ExitCode> {
    let (schematic, load_warnings) = load_project(project_path)?;

    for w in &load_warnings {
        warn!("{w}");
        eprintln!("warning: {w}");
    }

    let (library, lib_errors) = schematic.resolve_modules();
    if !lib_errors.is_empty() {
        for (path, e) in &lib_errors {
            eprintln!("error: failed to parse {}: {}", path.display(), e);
        }
        return Err(ExitCode::from(2));
    }
    Ok((schematic, library))
}

fn cmd_codegen(project_path: &std::path::Path, output: Option<&std::path::Path>) -> ExitCode {
    let (schematic, library) = match load_project_and_library(project_path) {
        Ok(v) => v,
        Err(code) => return code,
    };

    let grouped = !schematic.groups.is_empty();
    let design = match codegen::generate_design(&schematic, &library) {
        Ok(d) => d,
        Err(e) => return report_design_error(e, grouped),
    };

    match output {
        Some(out_path) => {
            let dir = out_path.parent().unwrap_or_else(|| std::path::Path::new("."));
            for (fname, code) in &design.files {
                let p = dir.join(fname);
                if let Err(e) = std::fs::write(&p, code) {
                    eprintln!("error: {e}");
                    return ExitCode::from(2);
                }
                println!("Written to {}", p.display());
            }
            if let Err(e) = std::fs::write(out_path, &design.top_code) {
                eprintln!("error: {e}");
                return ExitCode::from(2);
            }
            println!("Written to {}", out_path.display());
        }
        None => {
            let comment = match schematic.language {
                Language::Vhdl => "--",
                Language::SystemVerilog => "//",
            };
            for (fname, code) in &design.files {
                println!("{comment} ==== {fname} ====");
                print!("{code}");
            }
            if grouped {
                println!("{comment} ==== top ====");
            }
            print!("{}", design.top_code);
        }
    }
    ExitCode::SUCCESS
}

/// Load, resolve, codegen and stage a project into a scratch dir for an
/// external-tool run. Any failure is printed and mapped to exit code 2.
fn stage_project(
    project_path: &std::path::Path,
) -> Result<(Schematic, hdl_compose::toolchain::StagedBuild), ExitCode> {
    let (schematic, library) = load_project_and_library(project_path)?;
    let design = match codegen::generate_design(&schematic, &library) {
        Ok(d) => d,
        Err(e) => {
            report_design_error(e, !schematic.groups.is_empty());
            return Err(ExitCode::from(2));
        }
    };
    let staged = hdl_compose::toolchain::stage(&schematic, &library, &design).map_err(|e| {
        eprintln!("error: {e}");
        ExitCode::from(2)
    })?;
    Ok((schematic, staged))
}

fn tool_result(result: Result<bool, String>, pass_msg: &str, fail_msg: &str) -> ExitCode {
    match result {
        Ok(true) => {
            println!("{pass_msg}");
            ExitCode::SUCCESS
        }
        Ok(false) => {
            eprintln!("error: {fail_msg} (see tool output above)");
            ExitCode::FAILURE
        }
        Err(msg) => {
            eprintln!("error: {msg}");
            ExitCode::from(2)
        }
    }
}

fn cmd_check(project_path: &std::path::Path) -> ExitCode {
    let (schematic, staged) = match stage_project(project_path) {
        Ok(v) => v,
        Err(code) => return code,
    };
    tool_result(
        hdl_compose::toolchain::run_check(&staged),
        &format!("check passed: {} elaborates cleanly", schematic.top_name),
        "check failed",
    )
}

fn cmd_synth(project_path: &std::path::Path) -> ExitCode {
    let (_, staged) = match stage_project(project_path) {
        Ok(v) => v,
        Err(code) => return code,
    };
    tool_result(
        hdl_compose::toolchain::run_synth(&staged),
        "synth passed",
        "synth failed",
    )
}

fn cmd_sim(project_path: &std::path::Path, wave: bool) -> ExitCode {
    let (schematic, staged) = match stage_project(project_path) {
        Ok(v) => v,
        Err(code) => return code,
    };
    let project_dir = project_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."));
    match hdl_compose::toolchain::run_sim(&staged, &schematic, project_dir) {
        Ok(Some(wave_file)) => {
            println!("sim finished, wave written to {}", wave_file.display());
            if wave
                && let Err(msg) = hdl_compose::toolchain::open_wave_viewer(&wave_file)
            {
                eprintln!("error: {msg}");
                return ExitCode::from(2);
            }
            ExitCode::SUCCESS
        }
        Ok(None) => {
            eprintln!("error: sim failed (see tool output above)");
            ExitCode::FAILURE
        }
        Err(msg) => {
            eprintln!("error: {msg}");
            ExitCode::from(2)
        }
    }
}

fn cmd_fpga(
    project_path: &std::path::Path,
    family: hdl_compose::toolchain::FpgaFamily,
    output: Option<&std::path::Path>,
    force: bool,
) -> ExitCode {
    let (schematic, library) = match load_project_and_library(project_path) {
        Ok(v) => v,
        Err(code) => return code,
    };
    let project_dir = project_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."));
    let out_dir = output.unwrap_or(project_dir);
    if let Err(e) = std::fs::create_dir_all(out_dir) {
        eprintln!("error: cannot create {}: {e}", out_dir.display());
        return ExitCode::from(2);
    }

    let project_file_name = project_path
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_else(|| project_path.display().to_string());
    let scaffold =
        hdl_compose::toolchain::fpga_scaffold(&schematic, &library, family, &project_file_name);

    let makefile_path = out_dir.join("Makefile");
    if makefile_path.exists() && !force {
        eprintln!(
            "error: {} exists — pass --force to overwrite",
            makefile_path.display()
        );
        return ExitCode::from(2);
    }
    let constraints_path = out_dir.join(&scaffold.constraints_name);
    if let Err(e) = std::fs::write(&makefile_path, &scaffold.makefile) {
        eprintln!("error: {e}");
        return ExitCode::from(2);
    }
    println!("Written to {}", makefile_path.display());
    if constraints_path.exists() && !force {
        println!("Kept existing {}", constraints_path.display());
    } else {
        if let Err(e) = std::fs::write(&constraints_path, &scaffold.constraints) {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
        println!("Written to {}", constraints_path.display());
    }
    ExitCode::SUCCESS
}

fn cmd_inspect(project_path: &std::path::Path) -> ExitCode {
    let (schematic, load_warnings) = match load_project(project_path) {
        Ok(v) => v,
        Err(code) => return code,
    };

    for w in &load_warnings {
        eprintln!("warning: {w}");
    }

    let lang_str = match schematic.language {
        Language::Vhdl => "VHDL",
        Language::SystemVerilog => "SystemVerilog",
    };

    println!("Project: {}", schematic.top_name);
    println!("Language: {lang_str}");
    println!("Instances: {}", schematic.instances.len());

    if !schematic.instances.is_empty() {
        for inst in &schematic.instances {
            println!("  {} : {}", inst.name, inst.module_ref);
        }
    }

    println!("Library paths: {}", schematic.library_paths.len());
    for p in &schematic.library_paths {
        let status = if p.exists() { "ok" } else { "MISSING" };
        println!("  {} [{}]", p.display(), status);
    }

    // Try to resolve and report issues
    let (library, lib_errors) = schematic.resolve_modules();
    for (path, e) in &lib_errors {
        println!("Library parse failed for {}: {}", path.display(), e);
    }
    let diags = schematic.validate(&library);
    let errors = diags.iter().filter(|d| d.is_error()).count();
    let warnings = diags.iter().filter(|d| !d.is_error()).count();
    println!(
        "Validation: {} errors, {} warnings (+ {} library parse errors)",
        errors,
        warnings,
        lib_errors.len()
    );

    ExitCode::SUCCESS
}

fn load_project(path: &std::path::Path) -> Result<(Schematic, Vec<String>), ExitCode> {
    debug!("Loading project: {}", path.display());
    project::load_project(path).map_err(|e| {
        eprintln!("error: {e}");
        ExitCode::from(2)
    })
}
