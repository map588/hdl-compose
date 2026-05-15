//! Migrate older .hdlc project files to the latest version.
//!
//! Usage: `cargo run --bin migrate-hdlc -- path/to/proj1.hdlc [path/to/proj2.hdlc ...]`
//!
//! For each path, load_project() applies any in-process migrations (e.g. v3 ->
//! v4 fills in default `consumer_slices` and `manual_bundles`). save_project()
//! writes the file back at CURRENT_VERSION. Files already at the current
//! version are still rewritten (idempotent).

use std::path::Path;
use std::process::ExitCode;

use hdl_compose::project::{load_project, save_project};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!(
            "usage: migrate-hdlc <path.hdlc> [<path.hdlc> ...]\n\
             Loads each project (applying migrations) and saves it back at the\n\
             current .hdlc version."
        );
        return ExitCode::from(2);
    }

    let mut failed = 0;
    for arg in &args {
        let path = Path::new(arg);
        match load_project(path) {
            Ok((schematic, warnings)) => {
                for w in &warnings {
                    eprintln!("{}: warning: {w}", path.display());
                }
                if let Err(e) = save_project(&schematic, path) {
                    eprintln!("{}: save failed: {e}", path.display());
                    failed += 1;
                } else {
                    println!("migrated {}", path.display());
                }
            }
            Err(e) => {
                eprintln!("{}: load failed: {e}", path.display());
                failed += 1;
            }
        }
    }
    if failed > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
