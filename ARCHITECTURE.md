# hdl-compose — Architecture

hdl-compose is a block editor for structural HDL. The user writes the leaf
modules in VHDL or SystemVerilog. The tool reads their ports, lets the user
place instances and connect them, and writes the structural module that
instantiates them. It can then run the open-source FPGA tools on the result.

This document describes the design as the code is on 2026-11-14. It replaces
the earlier version, which described the system before groups, the net
model with union-find, and the tool flows.

This document holds the layers, the ownership of data, the invariants, the
data flow, and the decisions with their reasons. The other documents hold
the rest:

| Subject | Document |
|---|---|
| How to build and use the tool; the CLI; the GUI | [README.md](README.md) |
| The `.hdlc` file format | [docs/FORMAT.md](docs/FORMAT.md) |
| How an agent writes a project | [docs/AGENT_GUIDE.md](docs/AGENT_GUIDE.md) |
| Contract of the Rust package | [src/README.md](src/README.md) |
| Contract of code generation | [src/codegen/README.md](src/codegen/README.md) |
| Contract of the Qt application and the bridge | [src/gui/README.md](src/gui/README.md) |

## Goal and non-goals

The goal is to make a top-level wrapper faster to write than a hand-typed
port map, and to check the result with the real tools.

Non-goals:

- Not a gate-level schematic editor. A leaf module is HDL that the user
  writes.
- Not a synthesis tool, a simulator or a place-and-route tool. hdl-compose
  runs external tools (ghdl, slang, verilator, yosys, nextpnr, iverilog,
  openFPGALoader) and shows their results. It has no such engine of its own.
- Not IP-XACT. The only metadata is what the HDL source holds.
- Not a mixed-language editor. A project is VHDL or SystemVerilog.

## The central decision: one model, two views

The `Schematic` in memory is the one model. The canvas and the text editor
are two views of it. A change in one view changes the model, and the other
view draws the model again. No view holds state that the model does not
have.

The first design said that wiring happens in the text editor and that the
canvas only shows topology. The product changed: the canvas is now the
primary wiring surface (click pin to pin, drag, join a wire), and the editor
panel is hidden by default. The text editor stays as the second view for
bulk edits. The canvas refuses a connection that is an error. The text
editor accepts each line that it can parse, and validation reports the
error. Validation is thus the one check that holds for the two views and for
a file from another source. A test keeps the rule that validation reports
each connection that the canvas refuses.

## System map

One Rust package builds one library (`hdl_compose`) and one binary (`hdlc`).
The binary is the CLI and the GUI.

```text
                 +--------------------- hdlc binary ---------------------+
                 |                                                       |
   CLI (main.rs) |        GUI (src/gui)                                  |
        |        |   C++: MainWindow, canvas, items, Chip View           |
        |        |        |  invokables, signals      ^ subprocess       |
        |        |        v  (cxx-qt bridge)          | + files          |
        |        |   Rust: AppState (bridge.rs)       |                  |
        v        |        |                           |                  |
   +----+--------+--------+---------------------------+----------------+ |
   | Model:        types, schematic, project, reserved                 | |
   | Derived data: nets, groups, codegen, routing, vhdl, verilog       | |
   | Tools:        toolchain, board, floorplan, netcheck               | |
   +--------------------------------------------------------------------+ |
                 +-------------------------------------------------------+
                          |
                          v
        external tools on PATH (ghdl, slang, yosys, nextpnr, ...)
```

The roles are not strict layers: `schematic`, `nets`, `codegen` and `groups`
use each other. [src/README.md](src/README.md) lists the uses between the
modules.

## Data model

```rust
pub struct Schematic {
    top_name: String,
    language: Language,            // Vhdl | SystemVerilog, one for a project
    top_generics: Vec<GenericDef>,
    top_ports: Vec<PortDef>,
    instances: Vec<Instance>,
    aliases: HashMap<NetId, String>,
    library_paths: Vec<PathBuf>,
    groups: Vec<Group>,
}

pub struct Instance {
    name: String,
    module_ref: String,
    generic_map: HashMap<String, String>,
    port_map: HashMap<String, Option<NetRef>>,
    consumer_slices: HashMap<String, SliceExpr>, // part of the own port
    position: (f32, f32),
    manual_bundles: HashMap<String, Vec<String>>,
    dirty: bool,                   // the module source changed
    locked: bool,                  // the layout commands do not move it
}

pub enum NetRef {
    TopPort(String),
    InstancePort(String, String),
    TopPortSlice(String, SliceExpr),
    InstancePortSlice(String, String, SliceExpr),
    Constant(String),              // a literal, written as given
}

pub struct Group {                 // a set of instances that becomes a module
    name: String,
    members: Vec<String>,
    parent: Option<String>,        // groups nest
    collapsed: bool,
    position: (f32, f32),
}
```

A `ModuleDef` (name, generics, ports of a library module) is never saved.
The loader reads it from the library sources each time, so a changed source
is seen.

### Nets

A `port_map` entry states that two pins are connected. Its direction has no
effect: `B.din => A.dout` and `A.dout => B.din` are the same wire.
`nets::resolve_nets` merges all pins that reference each other into nets
with union-find. For each net it gives the drivers, the signal name and the
declaration type. Codegen, validation, the groups and the netlist check all
use this one result.

- **Name of a net:** the alias, if the user set one. Else `<port>_s` for a
  net with a top-level port, else `<instance>_<port>` of a pin on the full
  net (`Schematic::signal_name`).
- **Bits:** a slice in a reference uses the index numbers of the port that
  it names. `Nets::bits` converts a reference to offsets from the lowest bit
  of the net.
- **Drivers:** the outputs of instances and the top-level inputs. Two
  drivers conflict when their bits overlap.

The model has no signal declaration. Codegen makes the signals from the
nets.

The canvas draws one wire for each `port_map` entry. The pin that an entry
references is not always the driver, so a wire alone does not give its net.
The canvas takes the net of each wire from the same result
(`AppState::wire_net`): the colors and the highlights show the nets of the
generated HDL. An alias belongs to the net: `set_alias` keeps one alias for
each net, on whichever pin the user set it.

### Groups

A group is a set of instances that codegen writes as its own module. A net
with pins in the group and pins outside becomes a port of that module; the
port is the full net. `groups::expand_hierarchy` collapses the groups, child
groups first, and gives one schematic for each group and one for the top
level. A group must not change what is connected: the flat schematic and the
schematic with groups give the same connections between the leaf pins.

## Code generation

`codegen::generate_design` is the one entry point, for the CLI and for the
GUI. It returns the text of each file and writes nothing.

- A project with no group gives one file. A project with groups gives one
  file for each group and the top-level file.
- Codegen stops when validation reports an error, and when an instance is
  `dirty`.
- Each connected top-level port that is not `inout` goes through an
  intermediate signal, so each reference in the module has one name.
- A width that comes from a generic becomes a literal when the generic has
  an integer value. A value that is a name (a top-level generic) stays in
  the bound.
- The output is deterministic: instances in the order of their names, no
  metadata of the tool.

[src/codegen/README.md](src/codegen/README.md) has the contract.

## The split between Rust and C++

**Rule:** Rust decides; C++ draws and takes input. A calculation that has a
correct answer (nets, validation, placement, wire routes, the grammar of the
editor buffer) is in plain Rust, where a test can run it without Qt. C++
holds the widgets, the scene and the events.

**Ownership:**

- Rust (`AppStateRust` in `src/gui/bridge.rs`) holds the schematic, the
  library, the diagnostics, the wire list, the undo stacks and the parse
  cache.
- C++ holds the Qt scene (items, wires as drawn), the window, and the user
  settings (`QSettings`).

**Three channels** connect the two sides:

1. **The `AppState` QObject.** More than 100 methods that C++ calls, and 13
   signals that Rust sends. Most read methods return one field for one
   index. A method that can fail returns `false`, and `last_error()` gives
   the message.
2. **Four free functions with shared structs** (`plan_routes_ffi`,
   `legalize_columns_ffi`, `optimize_positions_ffi`,
   `completion_context_ffi`). They pass geometry to `src/routing.rs` and
   text to `src/gui/editor.rs`.
3. **A subprocess and files.** For Check, Synth, Build and Flash, the GUI
   starts the `hdlc` binary itself and shows its output. The Chip View reads
   the file `<top>.floorplan.json` that `hdlc build` writes. The tool flows
   never go through the bridge.

**Encoding across the bridge:** a pin is the string `<inst>.<port>`, a
top-level port is `top:<name>`, each with an optional `[h]` or `[h:l]`
suffix. `NetKey` (C++) and `NetRef::to_key` / `from_key` (Rust) parse them.
Lists cross as strings, enums as integers.

[src/gui/README.md](src/gui/README.md) has the contract.

## How a change flows

For a change from the canvas or from the editor:

1. C++ calls a method of `AppState` (for example `connect_pins`).
2. The method takes a snapshot of the schematic (JSON), then changes the
   schematic. The snapshot becomes an undo step when the change is one.
3. `rebuild_library_and_validate` runs: it gets the library through the
   parse cache, validates, and makes the pin notes and the wire list again.
4. Rust sends a signal (`port_map_changed`, `instance_added`, ...).
5. The C++ handler reads the new data through the read methods and draws.
   The canvas calculates all wire routes again; routing is global, not
   incremental.

`begin_batch` / `end_batch` put several changes into one undo step, one
validation and one signal.

Everything recalculates on each change. This is a decision: the picture then
depends only on the model, not on the history of edits.

**Size.** The cost of that decision, measured on 2026-11-15 (release build,
Apple silicon, generated projects, one net for each block). The time of the
Qt scene is not in these numbers.

| Blocks | Edit (validation, wire list) | Reads of the canvas | Wire routes |
|---|---|---|---|
| 50 | 0.2 ms | 0.2 ms | 0.3 ms |
| 200 | 1.0 ms | 1.6 ms | 7 ms |
| 500 | 2.4 ms | 4.7 ms | 79 ms |
| 1000 | 8.5 ms | 18 ms | 575 ms |

The wire router is the limit: its time grows faster than the square of the
number of blocks. A project of 200 blocks recalculates in about 10 ms. At
500 blocks a user can see the time of each change. The design is not for
1000 blocks.

## Library and module changes

The library is the set of modules that the sources in `library_paths`
define. `rebuild_library_and_validate` is the one function that brings a
library in. It parses a source again when its modification time changed.
Then it compares the ports of each module with the library of the last call.
For an instance whose module changed a port (the port is gone, or has
another direction or type), it removes the entries of those ports and sets
`dirty`. Codegen stops on a `dirty` instance until the user confirms. The
reason: to change a connection in silence when a port changes is a
correctness hazard.

Two events start that function:

1. **Reload** (the file watcher, or Refresh Library): `reload_library`
   empties the parse cache first, so each source is parsed again. It takes
   an undo snapshot only when the comparison changes an instance; the file
   watcher calls it for each save of a source.
2. **Each other change of the model.** The comparison is then a part of that
   change and of its undo step. In this case the canvas keeps the old pins
   of the changed module until its next full rebuild (see "Known
   weaknesses", item 4).

**Open** (GUI and CLI) has no earlier library to compare with, because the
project file holds no module definition. A connection that no longer agrees
with its module stays in the model, and validation reports it. The same
occurs after an undo that brings back a connection that a comparison
removed. In the two cases `dirty` is not set; the error of validation stops
codegen.

## Undo and redo

Undo is a stack of snapshots, 100 entries at most. Each method that changes
the schematic calls `push_snapshot` before the change. `open_project` and
`new_project` clear the two stacks.

Three rules make one command of the user one undo step, and nothing else:

- **Pending snapshot.** A snapshot is pending first. It becomes an undo
  step, and clears the redo stack, only when the model is different after
  the call. A call that changes nothing (a name that is in use, a value
  that is the same) leaves the history as it is. No method needs a check of
  its own for that.
- **Batch.** Between `begin_batch` and `end_batch`, only the first change
  takes a snapshot.
- **View rebuild.** `project_loaded` and `groups_changed` make the canvas
  build its items again and put each block on the grid
  (`CanvasLayer::settleAll`). Rust sends these two signals through
  `rebuild_view`. While the handler runs, no change takes a snapshot: the
  moves are a part of the change that sent the signal. Without this rule,
  the settle after an undo clears the redo stack.

`AppState` sends `history_changed` where the history can change. The window
reads `can_undo` and `can_redo` on that signal, through a queued connection,
because the history is final only when the method returns.

## Files and failures

- `project::save_project` and `ToolchainConfig::set_board` replace their
  file in one step (`replace_file`: a temporary file in the same directory,
  then a rename). A failure during the save leaves the old file complete.
  These two files are the only copy of the work of the user.
- The generated HDL files and the files of the build directory are written
  in place. `hdlc` can make them again.
- The text of a project file is a function of the design: each map is
  written with its keys in sorted order. Two saves of one design give the
  same file.
- The GUI shows the warnings of the loader in the diagnostics (prefix
  `load:`) until the next save. The loader can change the model: it removes
  a reference to an instance that is not in the file.
- A panic in a method of the bridge stops the application (see "Known
  weaknesses", item 2).

## Tool flows

A tool command (`check`, `synth`, `sim`, `build`, `flash`) has three steps:

1. **Stage.** `toolchain::stage` generates the HDL and puts it, with the
   library sources, into `build/<top>/` next to the project. `hdlc` deletes
   that directory only when it made it (marker file `.hdlc-build`).
2. **Step table.** A pure function makes the list of commands for the
   language and the board (`check_stages`, `synth_stages`, `build_stages`,
   `netlist_stages`). A stage can have more than one variant, for a fallback
   (ghdl plugin, then ghdl netlist; `read_slang`, then `read_verilog -sv`).
   `hdlc build --dry-run` prints the table as a shell script.
3. **Run.** `run_stages` runs the commands, prints each one, and records it
   with its exit status in `steps.log`.

The settings of a project for the tools are in `<project>.toolchain.json`
(board, clocks, extra sources, tool options). The CLI and the GUI read the
same file.

**The netlist check** is the control of code generation. `hdlc check` lets
yosys read only the generated files, with each library module as a black
box, and compares the connections in that netlist, bit for bit, with the
nets of the schematic. yosys shares no code with hdlc, so a defect in
codegen or in the group collapse shows as a difference.

## Verification

- Unit tests next to the code, with a regression test for each defect.
- Property tests for the net model, routing and code generation.
- `tests/netlist_roundtrip.rs`: for random valid schematics, the netlist of
  the generated HDL must equal the nets of the schematic (VHDL and
  SystemVerilog, with nested groups).
- `tests/toolchain_tools.rs`: runs the real tools; skips when they are
  missing.
- Tests in `bridge.rs` make an `AppState` with no window
  (`new_app_state`). A signal handler in the test does the work of the C++
  handler.
- A property test (`history_follows_the_model`) runs random sequences of 26
  kinds of edit on such an `AppState`, with batches, undo and redo. For each
  call it compares the history with the model: a change is one undo step
  that undo and redo reverse exactly, and a call with no change leaves the
  history. For each final model, the reads of the canvas and the code
  generation must return.
- `cargo test --no-default-features` runs all tests but those of the GUI,
  on a machine with no Qt.
- No test runs the C++ classes (see "Known weaknesses", item 3).

## Decisions that stay

| Decision | Reason |
|---|---|
| One in-process Rust model behind a cxx-qt bridge | One language for all rules; no protocol to a second process |
| Text and canvas are views of one model | No state to keep in step between views |
| A reference is connectivity only; nets by union-find | A hand-written file and the GUI can write a connection from either end |
| Module definitions are never saved | A changed source is always seen |
| One `generate_design` for the CLI and the GUI | The two cannot give different files |
| Snapshot undo | Simple and correct for a model of this size |
| Global route calculation on each change | The picture depends only on the model |
| External tools as subprocesses, from PATH | No tool is linked into the binary; the user controls the versions |
| An independent tool checks code generation | ghdl and slang accept a file that connects the wrong pins when the widths agree |

## Known weaknesses

A review on 2026-11-14 found eight weaknesses. The corrected ones are now in
the sections above (undo and redo, the net of a wire, the library
comparison, the build with no Qt, the save, the size). These stay:

1. **Editing rules are in the bridge.** `src/gui/bridge.rs` (about 4,200
   lines of code and 1,100 lines of tests) holds wiring rules, the grammar
   of the top-level buffer, undo and batching, as methods of the Qt object.
   A test reaches them only through `AppState`.
2. **No recovery after a stop of the program.** A panic in a method of the
   bridge stops the application, and the changes after the last save are
   lost. No autosave or recovery file exists. The property test found no
   panic in 4,000 random sequences of edits; that is evidence, not proof.
3. **The C++ side has no test.** About 6,000 lines (`app.cpp`, `canvas.*`,
   `items.*`, `floorplan.h`) are checked by reading only.
4. **A changed source that comes in with an edit** (not with a reload)
   leaves the old pins of its module on the canvas until the next full
   rebuild. The model, the wires and the diagnostics are correct at once.
5. **The wire router limits the size** (see "How a change flows").

## Out of scope

- Mixed-language projects.
- Bus rippers and slice adapters beyond the inline slice forms.
- Manual positions for wire segments.
- Editing the inside of a library module.

## Stack

- Rust 2024, one package.
- Qt 6 (Widgets, Gui) through [cxx-qt](https://github.com/KDAB/cxx-qt) 0.8,
  only with the cargo feature `gui` (on by default). A build with
  `--no-default-features` is the CLI only and needs no Qt.
- `vhdl_lang` for VHDL and `sv-parser` for Verilog and SystemVerilog module
  headers.
- `serde` / `serde_json` for the project file, `schemars` for its schema,
  `clap` for the CLI, `tracing` for logs.
- `proptest` and `tempfile` for tests.
