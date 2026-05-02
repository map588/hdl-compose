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
    },

    /// Validate a .hdlc project and report diagnostics
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
        Some(Command::New { name, language }) => cmd_new(&name, language.into()),
        Some(Command::Validate { project }) => cmd_validate(&project),
        Some(Command::Codegen { project, output }) => cmd_codegen(&project, output.as_deref()),
        Some(Command::Inspect { project }) => cmd_inspect(&project),
    }
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

fn cmd_new(name: &str, language: Language) -> ExitCode {
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

    let schematic = Schematic::new(name, language);

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

    let diagnostics = schematic.validate(&library);

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

    if has_errors {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
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
