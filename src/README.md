# src

## Overview

`src` holds the one Rust package of hdl-compose. It builds a library crate
(`hdl_compose`, root `lib.rs`) and one binary (`hdlc`, root `main.rs`). The
binary is the CLI, and with no command or with `gui` it starts the Qt
application.

The modules have four roles:

1. **Model:** `types`, `schematic`, `project`, `reserved`. The `Schematic`
   and its checks, and the `.hdlc` file.
2. **Derived data:** `nets`, `groups`, `codegen`, `routing`, the parsers
   `vhdl` and `verilog`. Functions on the model. Only the parsers read a
   file.
3. **Tools:** `toolchain`, `board`, `floorplan`, `netcheck`. External
   commands (ghdl, slang, yosys, nextpnr and others) and their files.
4. **Front ends:** `main.rs` (CLI) and `gui` (Qt application).

The roles are not strict layers. These are the uses between the modules in
the code that is not test code:

- `types`, `routing` and `floorplan` use no other module of the crate.
- `schematic`, `nets`, `codegen` and `groups` use each other: `schematic`
  uses `nets` and `codegen`; `nets` uses `codegen`; `codegen` uses
  `schematic`, `nets` and `groups`; `groups` uses `schematic`, `nets` and
  `codegen`.
- `toolchain` and `board` use each other. `toolchain` also uses `codegen`
  and `floorplan`.
- `netcheck` uses `nets` and `codegen`. It does not use `toolchain`.
- `gui` uses the model and the derived data, and from the tools only
  `toolchain` (for the board setting of a project).
- No module but `main.rs` uses `gui`.

## Interface

Subdirectories:

- [codegen](./codegen/README.md): the structural HDL of a project.
- [gui](./gui/README.md): the Qt application and the bridge between Rust
  and C++.

Crate root (`lib.rs`):

- `parse_file(path) -> Result<Vec<ModuleDef>, ParseError>`: reads the modules
  of one HDL source file. `.vhd` and `.vhdl` go to `vhdl::parse_vhdl`; `.v`
  and `.sv` go to `verilog::parse_verilog`.
- `ParseError`

Model:

- `types.rs`: `Schematic`, `Instance`, `Group`, `ModuleDef`, `PortDef`,
  `GenericDef`, `PortType`, `Range`, `RangeExpr`, `RangeDir`, `Direction`,
  `Language`, `NetRef` (`base`, `instance_name`, `to_key`, `from_key`),
  `SliceExpr` (`to_suffix`), `NetId`.
- `schematic.rs`: the methods of `Schematic` (`new`, `add_instance`,
  `rename_instance`, `remove_instance`, `set_port_map_entry`,
  `set_generic_map_entry`, `set_alias`, `remove_alias`, `add_library_path`,
  `remove_library_path`, `replace_top_level`, `apply_library_update`,
  `clear_instance_dirty`, `cleanup_stale_refs`, `resolve_modules`,
  `signal_name`, `validate`, `get_instance`, `get_instance_mut`),
  `Diagnostic` (`error`, `warning`, `is_error`), `DiagnosticLevel`,
  `SchematicError`.
- `project.rs`: `load_project`, `save_project`, `hdlc_schema_json`,
  `ProjectError`.
- `reserved.rs`: `is_reserved_word`.

Derived data:

- `nets.rs`: `resolve_nets`, `Nets` (`net_for`, `name_for`, `bits`,
  `drivers_conflict`), `Net` (`aliases`), `pin_direction`,
  `instance_pin_type`, `lowest_index`, `is_ascending`; crate only:
  `has_literal_bounds`.
- `groups.rs`: `validate_groups`, `expand_hierarchy`, `HierarchyPlan`,
  `derive_boundary`, `BoundaryPort`, `transitive_members`.
- `routing.rs`: `plan_routes`, `legalize_columns`, `optimize_positions`,
  `resolve_clear_y`, and their data types (`Endpoint`, `Net`, `Rect`,
  `Params`, `Point`, `Dot`, `RouteResult`, `ColItem`, `PlaceNode`,
  `PlaceEdge`).
- `vhdl.rs`: `parse_vhdl`. `verilog.rs`: `parse_verilog`.

Tools:

- `toolchain.rs`:
  - Staging: `stage`, `StagedBuild`, `build_dir`,
    `dependency_ordered_sources`.
  - Step tables: `Step`, `Variant`, `Stage`, `FlowEnv`, `check_stages`,
    `synth_stages`, `build_stages`, `netlist_stages`, `run_stages`,
    `flow_script`.
  - Runs: `run_check`, `run_synth`, `run_build`, `build_script`,
    `run_netlist`, `run_sim`, `run_flash`, `open_wave_viewer`.
  - Project settings: `ToolchainConfig` (`load_for`, `board_path`,
    `set_board`), `VhdlLibrary`.
  - Other: `doctor`, `doctor_table`, `ToolReport`, `tool_on_path`,
    `require_tools`, `fpga_scaffold`, `FpgaScaffold`, `FpgaFamily`,
    `testbench_skeleton`, `floorplan_path`.
- `board.rs`: `Board` (`load`), `constraint_port_names`,
  `constraint_clock_ports`, `pin_check`.
- `floorplan.rs`: `Floorplan` (`from_nextpnr`, `unmatched_tiles`),
  `PlacedCell`, `CellKind`, `Usage`, `CriticalPath`, `Hop`, `parse_bel`,
  `parse_cells`, `TILE_PITCH`.
- `netcheck.rs`: `expected`, `observed`, `diff`, `compare`, `Outcome`,
  `Expected`, `Endpoint`, `Connectivity`.

CLI (`main.rs`), the commands of `hdlc`: `parse`, `new`, `validate`,
`codegen`, `inspect`, `migrate`, `schema`, `guide`, `check`, `synth`, `sim`,
`build`, `flash`, `fpga`, `doctor`, `gui`.

## Invariants

- A `port_map` entry is a statement that two pins are connected. Its
  direction has no effect. `resolve_nets` merges all pins that reference each
  other into one net with union-find. Codegen and validation both use this
  result [`nets.rs` module comment, `nets.rs:resolve_nets`].
- `Nets::bits` gives the bits that a reference selects as offsets from the
  lowest bit of the net. A slice in a reference uses the index numbers of
  the port that it names [`nets.rs:Nets::bits`].
- A name from the `.hdlc` file goes into the generated HDL as written, so
  validation rejects a reserved word of the project language
  [`reserved.rs` module comment].
- The `.hdlc` file holds the schematic and a version number. The module
  definitions are not in it; the loader reads them from the library sources
  [`project.rs:ProjectFile`, `project.rs:load_project`].
- The loader accepts versions `MIN_SUPPORTED_VERSION` (2) to
  `CURRENT_VERSION` (5) and rejects each other version. `save_project`
  writes `CURRENT_VERSION` [`project.rs:load_project`,
  `project.rs:save_project`].
- `save_project` writes a library path below the project directory as a
  relative path, and `load_project` resolves it against the project
  directory [`project.rs:save_project`].
- `expand_hierarchy` must run only after `validate_groups` reports no error.
  It collapses child groups before their parents
  [`groups.rs:expand_hierarchy`].
- A boundary port of a group is the full net. In the group, each reference
  to a pin on a boundary net becomes a reference to the boundary port and
  keeps its bits [`groups.rs:derive_boundary`, `groups.rs:expand_hierarchy`].
- A group must not change what is connected. The netlist check ignores the
  groups on the schematic side for this reason [`netcheck.rs` module
  comment, `netcheck.rs:expected`].
- `routing.rs` has no Qt type and no access to the scene. The caller passes
  positions in. The nets given to `plan_routes` must be in a stable order
  [`routing.rs` module comment, `routing.rs:plan_routes`].
- The external tools come from PATH only [`toolchain.rs` module comment].
- The functions that build a step table are pure. They get the facts about
  the installed tools as data in `FlowEnv` [`toolchain.rs:FlowEnv`].
- `run_stages` tries the variants of a stage in order and uses the first one
  that passes. Each stage must pass [`toolchain.rs:run_stages`].
- The build directory of a project is `build/<top>` next to the project
  file. `reset_build_dir` deletes a directory only when it is empty or holds
  the marker file `.hdlc-build` [`toolchain.rs:build_dir`,
  `toolchain.rs:reset_build_dir`, `toolchain.rs:BUILD_MARKER`].
- `ToolchainConfig::set_board` is the only function that writes
  `<project>.toolchain.json` [`toolchain.rs:ToolchainConfig::set_board`].
- The netlist check runs yosys in `build/<top>/netcheck`. That directory
  must hold no ghdl library file, or the library modules are no longer black
  boxes [`toolchain.rs:NETLIST_DIR`].
- The netlist script sets `keep` on each instance before `opt_clean`
  [`toolchain.rs:KEEP_INSTANCES`, `toolchain.rs:netlist_stages`].
- `CellKind::of` checks the cell types in a fixed order, and the order has
  an effect on the result [`floorplan.rs:CellKind::of`].
- A CLI command that fails returns exit code 2. `validate` returns 1 for
  warnings only and 2 for errors [`main.rs:cmd_validate`,
  `main.rs:load_project`].

## Side Effects

- **Project file:** `project::load_project` reads the `.hdlc` file and each
  library source. `project::save_project` replaces the `.hdlc` file with
  `std::fs::write`; the write is not atomic [`project.rs`].
- **HDL sources:** `vhdl::parse_vhdl` and `verilog::parse_verilog` read the
  source file [`vhdl.rs`, `verilog.rs`].
- **Build directory:** `toolchain::stage` deletes and makes `build/<top>`,
  copies the library sources into it, and writes the generated files and
  `clocks.sdc` [`toolchain.rs:stage`, `toolchain.rs:reset_build_dir`,
  `toolchain.rs:write_clocks`].
- **External commands:** `run_stages` starts each tool with
  `std::process::Command`, prints the command line, and adds it and its exit
  status to `steps.log` [`toolchain.rs:run_step`, `toolchain.rs:record`].
  `doctor` and `tool_on_path` also start tools. `FlowEnv::detect` asks ghdl
  for its prefix for a VHDL project; for SystemVerilog it only looks for
  `slang` on PATH.
- **Bitstream:** `run_build` copies the bitstream to the output path and
  writes `<top>.floorplan.json` next to it [`toolchain.rs:run_build`,
  `toolchain.rs:write_floorplan`].
- **Simulation:** `run_sim` writes the testbench next to the project when it
  is missing, and copies the wave file there [`toolchain.rs:run_sim`].
- **Hardware:** `run_flash` programs a board with openFPGALoader
  [`toolchain.rs:run_flash`].
- **Detached process:** `open_wave_viewer` starts surfer or gtkwave
  [`toolchain.rs:open_wave_viewer`].
- **Settings file:** `ToolchainConfig::set_board` reads and replaces
  `<project>.toolchain.json` [`toolchain.rs:ToolchainConfig::set_board`].
- **Board file:** `Board::load` reads a board file [`board.rs:Board::load`].
- **Environment:** `find_on_path` reads `PATH`; `ghdl_prefix` reads an
  environment variable and asks ghdl [`toolchain.rs`].
- `types`, `schematic`, `nets`, `groups`, `routing`, `netcheck`, `reserved`
  and `floorplan` (apart from the files that `toolchain` reads for it) have
  no side effect.

## Verification

- Run: `cargo test`
- Lint: `cargo clippy --all-targets -- -D warnings`
- The tests in `tests/toolchain_tools.rs` and `tests/netlist_roundtrip.rs`
  need the external tools. They skip with a message when a tool is missing.
