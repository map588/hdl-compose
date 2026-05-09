## Why

The parser, schematic model, project I/O, and codegen all exist as a library — but there's no way to invoke them. A CLI makes the tool usable *today*: parse HDL files, create/validate projects, generate structural output, all from the terminal. This proves the vertical slice end-to-end before any GUI work begins, and provides a headless interface for scripting and CI pipelines.

## What Changes

- Add a `main.rs` binary entry point using `clap` for argument parsing.
- `hdl-compose parse <file>` — parse an HDL file and print extracted module definitions.
- `hdl-compose new <name> --language <vhdl|sv>` — create a new `.hdlc` project file.
- `hdl-compose validate <project.hdlc>` — validate a project and report diagnostics.
- `hdl-compose codegen <project.hdlc> [--output <file>]` — generate structural HDL from a project.
- `hdl-compose inspect <project.hdlc>` — print project summary (instances, connections, library status).
- Add `tracing` crate for structured logging with `--verbose` flag.
- Add integration tests with real HDL fixture files.
- Add `.gitignore`, `rustfmt.toml`, and clippy configuration for project hygiene.

## Capabilities

### New Capabilities
- `cli-commands`: The command-line interface — subcommands, argument parsing, output formatting, and error reporting.
- `project-hygiene`: Project setup items — .gitignore, rustfmt, clippy config, tracing integration.

### Modified Capabilities

(none)

## Impact

- **New files**: `src/main.rs`, test fixtures in `tests/fixtures/`.
- **New dependencies**: `clap` (with derive), `tracing`, `tracing-subscriber`.
- **Cargo.toml**: Add `[[bin]]` section and new deps.
- **Downstream**: GUI will eventually replace most CLI interaction, but CLI remains useful for scripting and validation.
