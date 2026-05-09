## Why

The parser layer extracts `ModuleDef` from HDL source files, but there is no model to represent a design — instances of those modules, their wiring, top-level ports, or the relationships between them. Without a `Schematic` model, nothing else can be built: no project files, no code generation, no GUI. This is the core data structure that everything views and mutates.

## What Changes

- Add `Schematic`, `Instance`, `NetRef`, and `Language` types that represent an in-memory block design.
- Net identity follows the "driver is the name" rule: each net is identified by its driving source (`InstancePort("u_pll", "clk_out")` or `TopPort("clk_sys")`), with optional user-chosen aliases for generated signal names.
- Add `.hdlc` project file serialization/deserialization via serde_json. Version 2 schema. `ModuleDef` data is never stored — always re-derived from source on load.
- Add deterministic VHDL code generation from a `Schematic` — entity declaration, signal declarations, component declarations, instance statements with full generic/port maps.
- Add deterministic SystemVerilog code generation — module declaration, wire declarations, instance statements. Per-project language selection (never mixed).
- Add validation: detect width mismatches, direction errors, references to nonexistent instances/ports, duplicate instance names.
- Upgrade error handling to `thiserror`.

## Capabilities

### New Capabilities
- `schematic-model`: The in-memory design representation — Schematic, Instance, NetRef, net resolution, and design validation.
- `project-io`: Save/load `.hdlc` project files. Round-trip fidelity. Library path resolution and re-parse on load.
- `vhdl-codegen`: Generate deterministic, readable structural VHDL from a Schematic.
- `sv-codegen`: Generate deterministic, readable structural SystemVerilog from a Schematic.

### Modified Capabilities

(none)

## Impact

- **New types in `src/types.rs`**: `Schematic`, `Instance`, `NetRef`, `Language`, alias map.
- **New modules**: `src/schematic.rs` (model + validation), `src/project.rs` (I/O), `src/codegen/vhdl.rs`, `src/codegen/sv.rs`.
- **Dependencies**: Add `thiserror`. Existing `serde`/`serde_json` used for project files.
- **Downstream**: GUI (canvas, sidebar, mini editor) will be views of the `Schematic` model built here. CLI will invoke codegen and project I/O.
