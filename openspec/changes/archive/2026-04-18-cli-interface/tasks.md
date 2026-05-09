## 1. Project Setup

- [x] 1.1 Add `clap` (with derive feature), `tracing`, `tracing-subscriber` to Cargo.toml
- [x] 1.2 Create `src/main.rs` with clap CLI skeleton (top-level struct + subcommand enum)
- [x] 1.3 Create `.gitignore` (target/, *.hdlc, editor files)
- [x] 1.4 Create `rustfmt.toml` with formatting config
- [x] 1.5 Fix any existing clippy warnings across the codebase

## 2. Tracing Setup

- [x] 2.1 Initialize tracing-subscriber in main with env filter (default warn, verbose = debug)
- [x] 2.2 Add `--verbose` / `-v` global flag to CLI struct
- [x] 2.3 Add `tracing::debug!` / `tracing::info!` calls to key library functions (parse_file, validate, codegen)

## 3. Parse Subcommand

- [x] 3.1 Implement `parse` subcommand — accept file path, call `parse_file`, print results
- [x] 3.2 Format output: module name, generics (name: type = default), ports (name: direction type)
- [x] 3.3 Handle errors: file not found → exit 2, parse error → exit 2

## 4. New Subcommand

- [x] 4.1 Implement `new` subcommand — accept name + `--language` flag (vhdl|sv)
- [x] 4.2 Create empty Schematic, save as `<name>.hdlc` via save_project
- [x] 4.3 Refuse to overwrite existing file → exit 1

## 5. Validate Subcommand

- [x] 5.1 Implement `validate` subcommand — load project, parse library, run validation
- [x] 5.2 Print each diagnostic with level, instance, port, message
- [x] 5.3 Exit 0 if no errors (warnings OK), exit 1 if errors

## 6. Codegen Subcommand

- [x] 6.1 Implement `codegen` subcommand — load project, validate, generate HDL
- [x] 6.2 Default output to stdout, `--output` / `-o` flag writes to file
- [x] 6.3 Print errors to stderr and exit 1 if validation fails

## 7. Inspect Subcommand

- [x] 7.1 Implement `inspect` subcommand — load project, print summary
- [x] 7.2 Show: top name, language, instance count, instance list (name: module), library path count, resolution issues

## 8. Test Fixtures and Integration Tests

- [x] 8.1 Create `tests/fixtures/counter.vhd` — simple VHDL entity with generics and ports
- [x] 8.2 Create `tests/fixtures/counter.v` — equivalent Verilog module
- [x] 8.3 Create `tests/fixtures/fifo_sync.vhd` — second module for multi-instance testing
- [x] 8.4 Integration test: parse fixture → verify ModuleDef fields
- [x] 8.5 Integration test: build schematic from fixtures → codegen → verify output contains expected strings
- [x] 8.6 Integration test: validate a schematic with known errors → verify diagnostics
- [x] 8.7 Verify `cargo clippy -- -D warnings` passes clean
- [x] 8.8 Verify `cargo fmt --check` passes clean
