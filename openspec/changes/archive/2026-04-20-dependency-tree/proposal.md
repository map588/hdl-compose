## Why

The sidebar tree currently shows only top-level instances. If a user drags `n_register` onto the canvas and that module internally instantiates `flipflop`, there is no indication that the design *also* needs a `flipflop` module in the library. When codegen runs, the structural VHDL will reference `flipflop` as a component — and if `flipflop.vhd` was never imported, the generated file won't synthesize.

This change surfaces those nested dependencies in the tree so the user can see what's missing before codegen time. Missing dependencies render red; present ones render normally.

Scope decisions from the scope discussion:
- **Unique modules only** — for each instance, show its module's DISTINCT dependency set. No per-sub-instance enumeration, no generate-loop unrolling. Single pass through the file.
- **Recompute on library change** — dependency presence is rechecked whenever `add_library_path` / `remove_library_path` fires.

## What Changes

- Parser emits a per-module dependency list. For each parsed `.vhd`/`.sv`/`.v`:
  - VHDL: collect all component names used in architecture body (`component_instantiation_statement`).
  - Verilog/SV: collect all module references in module body (instance declarations).
- `ModuleDef` gains a `dependencies: Vec<String>` field (not persisted to `.hdlc` — always re-derived on load).
- AppState bridge exposes per-instance dependency enumeration:
  - `instance_dependency_count(instance_index) -> i32`
  - `instance_dependency_name(instance_index, dep_index) -> QString`
  - `instance_dependency_present(instance_index, dep_index) -> bool`
- Tree model renders each instance's dependencies as child rows under the instance:
  - Present: normal color, tree-leaf icon.
  - Missing: red foreground color, "⚠" prefix, tooltip "Module not in library".
- `library_changed` signal triggers full sidebar rebuild so red/normal state reflects current library contents.

## Capabilities

### New Capabilities
- `dependency-tree`: parse module-level dependencies from HDL sources; expose per-instance dependency list with presence flag; sidebar renders dependencies as child nodes under each instance with missing-module visual marker.

### Modified Capabilities
(none — extends schematic parsing and sidebar rendering without changing their existing contracts)

## Impact

- **src/vhdl.rs** — extend `parse_vhdl_file` to walk architecture bodies for `component_instantiation_statement` AST nodes. Currently only entity headers are parsed.
- **src/verilog.rs** — extend `parse_verilog_file` to scan module bodies for instance references.
- **src/types.rs** — `ModuleDef.dependencies: Vec<String>`.
- **src/gui/bridge.rs** — three new invokables.
- **src/gui/app.cpp** — `rebuild_tree_model` extended; dependency children added under each instance; red styling + warning icon.
- **No `.hdlc` schema change** — dependencies re-derived on parse.
- **No backend crate change** — unit tests added against fixtures with known component references.
