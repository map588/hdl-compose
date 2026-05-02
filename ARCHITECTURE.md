# hdl-compose — Architecture

A GUI tool for composing structural VHDL/SystemVerilog top-level wrappers from
user-authored modules. Parses existing HDL source, lets the user drop instances
into a block diagram, wire them, and emits a structural architecture file.

This document supersedes the April 2026 version. The data model + parser
crates carried forward; the GUI stack moved from egui to Qt (cxx-qt), and the
codegen routes top-level ports through intermediate signals.

## Non-goals

- Not a gate-level / truth-table schematic editor. Leaf modules are
  user-written HDL.
- Not a synthesis tool. Output is source for an existing flow (yosys, GHDL,
  vendor tools).
- Not IP-XACT. No metadata beyond what's in the HDL source.
- Not a waveform viewer, simulator, or linter.
- Not a mixed-language editor. Each `.hdlc` project is VHDL **or** SystemVerilog.

## The central design decision

Prior attempts at this tool fail because dragging wires between pins is slower
than typing a port map. If the GUI isn't faster than the keyboard for the
common case, the tool has no reason to exist.

**Text and canvas are two live views of one model.** The canvas shows
topology. Wiring happens in a per-instance VHDL-shaped mini editor. Canvas
edits emit equivalent text edits; text edits re-render the canvas. Neither
view is canonical — the in-memory `Schematic` is.

## Data model

```rust
pub struct ModuleDef {
    name: String,
    generics: Vec<GenericDef>,
    ports: Vec<PortDef>,
    source_path: PathBuf,
    source_hash: u64,        // for change detection
    dependencies: Vec<String>, // sub-modules referenced; not persisted
}

pub struct PortDef {
    name: String,
    direction: Direction,    // In | Out | InOut
    port_type: PortType,     // StdLogic | StdLogicVector(Range) | Record | Other
    bundle: Option<String>,
}

pub struct Schematic {
    top_name: String,
    language: Language,      // Vhdl | SystemVerilog (per-project)
    top_generics: Vec<GenericDef>,
    top_ports: Vec<PortDef>,
    instances: Vec<Instance>,
    aliases: HashMap<NetId, String>,
    library_paths: Vec<PathBuf>,
}

pub struct Instance {
    name: String,
    module_ref: String,
    generic_map: HashMap<String, String>,
    port_map: HashMap<String, Option<NetRef>>,
    dirty: bool,             // module source changed incompatibly
    position: (f32, f32),
    manual_bundles: HashMap<String, Vec<String>>,
}

pub enum NetRef {
    TopPort(String),
    InstancePort(String, String),
    TopPortSlice(String, SliceExpr),
    InstancePortSlice(String, String, SliceExpr),
}
```

**Net identity is the driver.** A net is named by its source: top-level port
name for externally-driven, `<inst>.<port>` for internally-driven. Loads
reference the driver. No separate signal-declaration concept in the model;
codegen synthesizes signals where needed.

## Sidebar — hierarchy + library

Left pane, two-section split:

- **Project tree:** root is the top-level schematic. Each instance appears as
  `u_name : module_name`. Each instance's known sub-module dependencies render
  as child rows (one level deep today; recursive traversal is open work).
  Dirty instances get a red dot.
- **Library pane:** parsed modules not yet placed. Drag onto the canvas to
  create an instance.

Sidebar is navigation + library only, not a wiring surface.

## Canvas — topology view

`QGraphicsScene` rendering. Instances as boxes with pins. Wires as orthogonal
paths. Top-level ports as chevrons on the canvas edges.

- Drag any instance to reposition; all wires reroute live, going around any
  instance whose body falls in the wire's corridor. Push-and-shove is greedy
  per-wire (no global solver).
- Click a port → click another port to create a connection (commits the
  equivalent text edit under the hood).
- All pin chevrons point right (signal flow L→R always). Top-input chevron
  base sits outside the canvas; top-output chevron tip extends past the
  anchor. Top-port labels render OUTSIDE the canvas — left of inputs, right
  of outputs — to avoid wire overlap.
- Right-click a wire → set an alias (used as the signal name in codegen).
- Multi-bit wires render with a `--/--` slash + width number.
- Empty-canvas click deselects. Wheel-zoom proportional (trackpad-friendly).
  Cmd+Z / Cmd+Shift+Z undo/redo (snapshot-based, cap 100).

## Per-instance mini editor

Right pane. One editor buffer per selected instance, VHDL-shaped:

```vhdl
u_fifo : fifo_sync
  generic map (
    DEPTH => 1024,
    WIDTH => 16,
  )
  port map (
    clk   => clk_sys,
    rst_n => rst_n,
    din   => u_adc.data_out,
    dout  => u_dsp.data_in,
    full  => open,
  );
```

RHS grammar:

1. `<identifier>` — top-level port or alias.
2. `<instance>.<port>` — references the net driven by that instance's output.
3. `<head>[<i>]` or `<head>[<h>:<l>]` — slice of the above.
4. `open` — intentionally unconnected.

Anything else → red squiggle (inline `QSyntaxHighlighter` `WaveUnderline`).
Parse runs after a 300 ms idle debounce on every text change; never moves the
cursor while the user is typing. Tab on the completer popup accepts. Commit
on Ctrl+Return or focus-out. Refused commits stay in the buffer for the user
to fix; status bar surfaces the count.

## Top-level mini editor

Toggle button at the top of the right pane. Free-form grammar — the parser
silently ignores `entity ... is`, `port (`, `);`, `end entity` decoration:

```
entity my_top is
  generic (
    WIDTH : integer := 8,
  );
  port (
    clk  : in logic,
    rst  : in,                  -- type defaults to std_logic
    dout : out logic[7:0],
  );
end entity my_top;
```

Heuristic: lines with `in`/`out`/`inout` as first word after `:` are ports;
otherwise generics. Brief SV-style types (`logic`, `logic[h:l]`) preferred;
legacy `std_logic`, `std_logic_vector(h downto l)`, `slv(h:l)` all parse for
paste compatibility. `:` inside the type brackets means `downto`.

Removing a port cascade-clears any `port_map` entries (and aliases) that
referenced it. Top entity name lives in `Schematic.top_name` and isn't
editable from this view.

## Module re-parse semantics

When a module source changes (file watch or manual reload), for each instance
of that module:

- Diff each port by **name, direction, type, width**. Any difference = break.
- Drop every connection referencing a changed-or-removed port from
  `port_map`.
- Mark the instance `dirty` (red dot in sidebar, red outline on canvas).
- The mini editor shows a comment header noting the source change.
- Codegen is blocked on dirty instances (project save is not — JSON still
  serializes).

No string-matching auto-migration. Silent rewiring on a port change is a
correctness hazard, not a convenience.

## Code generation

Deterministic, readable, no tool-added metadata. One file per schematic.
File → Generate HDL... in the GUI; `hdl-compose codegen` from the CLI.

### Top-level intermediate signals

Every connected non-`InOut` top port routes through an intermediate signal
named `<alias>` if the user set one, else `<name>_s`. Codegen emits:

- For inputs:  `<sig> <= <port>;`
- For outputs: `<port> <= <sig>;`

All instance `port_map` entries that referenced the top port resolve to the
intermediate. `InOut` skipped (multi-driver hazard with naive routing).

### Generic-derived widths

Top-level entity ports promoted from a child instance carry the child's
generic-derived range expression (e.g. `WIDTH-1 downto 0`). At codegen time,
those bounds are evaluated against the driving instance's `generic_map` plus
the module's generic defaults — so the entity emits `(7 downto 0)` for
`WIDTH=8`. Internal signal declarations get the same treatment. Component
declarations stay symbolic so child parameterization survives.

### Instance ordering

Alphabetical by instance name for stable diffs. `open` for intentionally
unconnected. Header comment:
`-- Generated by hdl-compose. Edit the source .hdlc project file, not this file.`

### Language dispatch

`Schematic.language` switches between `codegen::vhdl::generate_vhdl` and
`codegen::sv::generate_sv`. SV uses `wire`/`assign`/`logic`/`[h:l]`/`#(.PARAM(v))`
shapes. The mini editor stays VHDL-shaped regardless — punctuation
(commas/semicolons) is uniform in the editor regardless of project language.

## Project file format

JSON via serde, extension `.hdlc`. Schema version 3. `ModuleDef` data is
never persisted — always re-derived from source on load.

```json
{
  "version": 3,
  "top_name": "my_top",
  "language": "Vhdl",
  "top_generics": [],
  "top_ports": [...],
  "instances": [...],
  "aliases": {...},
  "library_paths": ["rtl/fifo_sync.vhd", ...]
}
```

Loader supports v2 (no `manual_bundles`) and v3. Older versions rejected.
Loader also runs `cleanup_stale_refs` to drop port_map / alias entries
pointing at instances that no longer exist (defensive against pre-fix
saves).

## Stack

- **Rust 2024 edition** — single static binary, no runtime deps.
- **Qt 6** via [cxx-qt 0.8](https://github.com/KDAB/cxx-qt) — `QGraphicsScene`
  canvas, `QPlainTextEdit` + `QSyntaxHighlighter` + `QCompleter` for the mini
  editor, `QFileDialog` for project + codegen IO.
- **vhdl_lang 0.86** — VHDL entity/architecture extraction.
- **sv-parser 0.13** — Verilog/SystemVerilog module-header extraction.
- **serde / serde_json** — project file.
- **clap** — CLI.
- **tracing / tracing_subscriber** — logging.

## Undo/redo

Snapshot-based, cap 100 entries per stack. Every mutator pushes a JSON-serialized
schematic snapshot to the undo stack before mutating; the redo stack clears
on any new edit. Cmd+Z / Cmd+Shift+Z. `set_instance_position` is intentionally
skipped (drag noise would saturate the stack on every release).

## Out of scope for v1

- Hierarchical editing (open a sub-schematic inside an instance).
- Mixed-language schematics (VHDL instance of an SV module or vice versa).
- Bus rippers / slice adapters beyond inline `[h:l]` / `(h downto l)`.
- Tree-sitter grammar for the mini editor.
- Wire-segment manual repositioning (push-shove handles it implicitly today).

## Known open work

- Top-level mini editor: SV-grammar variant for SV projects (currently always
  VHDL-shaped).
- Project tree pane: recursive traversal of sub-component dependencies (one
  level today).
- Verilog/SV codegen: less manual GUI verification than VHDL path.

## Repository layout

```
src/
  main.rs              — CLI entry + dispatch
  lib.rs               — module declarations, parse_file
  types.rs             — Schematic, ModuleDef, NetRef, etc.
  schematic.rs         — Schematic methods, validate(), replace_top_level()
  project.rs           — load/save .hdlc
  vhdl.rs              — vhdl_lang AST → ModuleDef
  verilog.rs           — sv-parser AST → ModuleDef
  codegen/
    mod.rs             — shared helpers (resolve_port_type,
                          collect_internal_nets, collect_top_intermediates)
    vhdl.rs            — VHDL emitter
    sv.rs              — SystemVerilog emitter
  gui/
    bridge.rs          — cxx-qt AppState + invokables (Rust↔Qt)
    app.cpp            — Qt UI (canvas, mini-editor, menus, painters)
    mod.rs             — gui::run() entry
tests/
  integration.rs       — end-to-end parser + codegen tests
  fixtures/            — counter.{vhd,v}, fifo_sync.vhd
openspec/              — feature change proposals + archived specs
```
