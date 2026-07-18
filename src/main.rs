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

    /// Launch the Qt GUI (default when no subcommand is given)
    Gui,
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
fn codegen_hierarchical(
    schematic: &Schematic,
    library: &[hdl_compose::types::ModuleDef],
    output: Option<&std::path::Path>,
) -> ExitCode {
    use hdl_compose::groups;

    let group_diags = groups::validate_groups(schematic, library);
    if group_diags.iter().any(|d| d.level == DiagnosticLevel::Error) {
        for d in &group_diags {
            eprintln!("error: {d}");
        }
        return ExitCode::from(2);
    }

    let plan = groups::expand_hierarchy(schematic, library);
    let (ext, comment) = match schematic.language {
        Language::Vhdl => ("vhd", "--"),
        Language::SystemVerilog => ("sv", "//"),
    };

    let generate = |s: &Schematic| -> Result<String, hdl_compose::codegen::CodegenError> {
        let diags = s.validate(&plan.library);
        match s.language {
            Language::Vhdl => codegen::vhdl::generate_vhdl(s, &plan.library, &diags),
            Language::SystemVerilog => codegen::sv::generate_sv(s, &plan.library, &diags),
        }
    };

    let mut files: Vec<(String, String)> = Vec::new();
    for (name, gs) in &plan.groups {
        match generate(gs) {
            Ok(code) => files.push((format!("{name}.{ext}"), code)),
            Err(e) => {
                eprintln!("error: group '{name}': {e}");
                return ExitCode::from(2);
            }
        }
    }
    let top_code = match generate(&plan.top) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };

    match output {
        Some(out_path) => {
            let dir = out_path.parent().unwrap_or_else(|| std::path::Path::new("."));
            for (fname, code) in &files {
                let p = dir.join(fname);
                if let Err(e) = std::fs::write(&p, code) {
                    eprintln!("error: {e}");
                    return ExitCode::from(2);
                }
                println!("Written to {}", p.display());
            }
            if let Err(e) = std::fs::write(out_path, &top_code) {
                eprintln!("error: {e}");
                return ExitCode::from(2);
            }
            println!("Written to {}", out_path.display());
        }
        None => {
            for (fname, code) in &files {
                println!("{comment} ==== {fname} ====");
                print!("{code}");
            }
            println!("{comment} ==== top ====");
            print!("{top_code}");
        }
    }
    ExitCode::SUCCESS
}

fn cmd_codegen(project_path: &std::path::Path, output: Option<&std::path::Path>) -> ExitCode {
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
            eprintln!("error: failed to parse {}: {}", path.display(), e);
        }
        return ExitCode::from(2);
    }

    // Grouped projects emit one file per group plus the top module.
    if !schematic.groups.is_empty() {
        return codegen_hierarchical(&schematic, &library, output);
    }

    let diagnostics = schematic.validate(&library);

    let result = match schematic.language {
        Language::Vhdl => codegen::vhdl::generate_vhdl(&schematic, &library, &diagnostics),
        Language::SystemVerilog => codegen::sv::generate_sv(&schematic, &library, &diagnostics),
    };

    match result {
        Ok(code) => {
            if let Some(out_path) = output {
                match std::fs::write(out_path, &code) {
                    Ok(()) => {
                        println!("Written to {}", out_path.display());
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("error: {e}");
                        ExitCode::from(2)
                    }
                }
            } else {
                print!("{code}");
                ExitCode::SUCCESS
            }
        }
        Err(codegen::CodegenError::ValidationErrors(errs)) => {
            eprintln!("error: schematic has validation errors");
            for d in &errs {
                eprintln!("  {d}");
            }
            ExitCode::FAILURE
        }
        Err(codegen::CodegenError::DirtyInstances(names)) => {
            eprintln!(
                "error: dirty instances present (source re-parse dropped \
                 connections). Review and reconnect: {}",
                names.join(", ")
            );
            ExitCode::FAILURE
        }
    }
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
