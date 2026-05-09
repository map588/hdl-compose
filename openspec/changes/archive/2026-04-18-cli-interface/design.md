## Context

hdl-compose has a complete library layer: HDL parsing (VHDL + Verilog), schematic model with validation, project I/O (.hdlc JSON), and codegen (VHDL + SystemVerilog). Currently a `lib` crate only — no binary, no way to run it. Need a CLI to prove the vertical slice and provide a scripting interface.

## Goals / Non-Goals

**Goals:**
- Expose all existing library functionality via CLI subcommands.
- Structured error output that a human can act on.
- `--verbose` flag for debug tracing.
- Real HDL fixture files for integration testing.
- Project hygiene: .gitignore, rustfmt.toml, clippy.

**Non-Goals:**
- Interactive mode or REPL.
- File watching (that's a GUI concern).
- Project creation wizard beyond `new`.

## Decisions

### 1. clap with derive macros

Use `clap` derive for subcommand definitions. Clean, type-safe, self-documenting.

**Rationale:** Standard Rust CLI pattern. Derive macros eliminate boilerplate. Auto-generated `--help`.

### 2. Subcommand structure

```
hdl-compose parse <file>                         # show parsed module defs
hdl-compose new <name> --language vhdl|sv         # create empty .hdlc
hdl-compose validate <project.hdlc>              # run validation, print diagnostics
hdl-compose codegen <project.hdlc> [-o file]     # emit structural HDL
hdl-compose inspect <project.hdlc>               # print summary
```

**Rationale:** Each subcommand maps 1:1 to an existing library function. No new logic — pure wiring.

### 3. Exit codes

- 0: success
- 1: validation errors (codegen/validate)
- 2: I/O or parse errors

**Rationale:** Standard Unix convention. Scripts can check exit codes.

### 4. tracing for logging, not println

Use `tracing` + `tracing-subscriber` with `--verbose` / `-v` flag controlling log level. Default: warn. Verbose: debug.

**Rationale:** Structured logging scales better than scattered printlns. Can add file/span context later.

### 5. Output format

Default: human-readable text. Future: `--json` flag for machine-readable output. Not implementing `--json` in this change — keep it simple.

**Rationale:** Start human-readable. JSON can be added when someone actually needs it.

### 6. Fixture files for integration tests

Add `tests/fixtures/` with small, real HDL files covering common patterns: simple entity, module with generics, multi-entity file. Integration tests parse → build schematic → codegen → verify output.

**Rationale:** Tempfile-based tests are fine for unit tests, but integration tests should use stable, reviewable fixture files.

## Risks / Trade-offs

- **[Library is `lib` only, need `[[bin]]`]** → Add `src/main.rs` and `[[bin]]` to Cargo.toml. Library stays as `[lib]`.
- **[codegen requires a valid .hdlc with library paths pointing to real files]** → Integration tests create temp directories with both .hdlc and .vhd files.
- **[No interactive project editing via CLI]** → By design. CLI is for batch operations. Editing is hand-editing JSON or using the future GUI.
