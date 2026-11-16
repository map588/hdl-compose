# hdl-compose — Architecture

hdl-compose is a block editor for structural HDL. The user writes the leaf
modules in VHDL or SystemVerilog. The tool reads their ports, lets the user
place instances and connect them, and writes the structural module that
instantiates them. It can then run the open-source FPGA tools on the result.

This document describes the design as the code is on 2026-11-15. It replaces
the earlier version, which described the system before groups, the net
model with union-find, and the tool flows.

This document holds the layers, the ownership of data, the invariants, the
data flow, and the decisions with their reasons. The other documents hold
the rest:

| Subject | Document |
|---|---|
| How to build and use the tool; the CLI; the GUI | [README.md](README.md) |
| The `.hdlc` file format | [docs/FORMAT.md](docs/FORMAT.md) |
| How an agent writes a design | [docs/AGENT_GUIDE.md](docs/AGENT_GUIDE.md) |
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
- Not a mixed-language editor. A design is VHDL or SystemVerilog.

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
   | Model:        types, schematic, project, reserved, designs        | |
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
    language: Language,            // Vhdl | SystemVerilog, one for a design
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

- A design with no group gives one file. A design with groups gives one
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
- `GeneratedDesign::write_to` replaces only a file that hdlc generated (the
  first line says so), and writes no file when it refuses one. A design or
  a group can have the name of a hand-written source; that source stays.

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

1. **The `AppState` QObject.** More than 100 methods that C++ calls, and 15
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

A drag of a block with the pointer is not a change until the release. While
the button is down, the canvas moves the block with the pointer and draws
each wire of the block as one straight line; it calculates no route and
moves no other block. The release puts the block on the nearest column,
moves the other blocks clear, writes the positions as one undo step, and
calculates the routes one time (`CanvasLayer::endDrag`).

Everything recalculates on each change. This is a decision: the picture then
depends only on the model, not on the history of edits.

**Size.** The cost of that decision, measured on 2026-11-15 (release build,
Apple silicon, generated designs, one net for each block). The time of the
Qt scene is not in these numbers.

| Blocks | Edit (validation, wire list) | Reads of the canvas | Wire routes |
|---|---|---|---|
| 50 | 0.2 ms | 0.1 ms | 0.2 ms |
| 200 | 0.9 ms | 1.0 ms | 1.6 ms |
| 500 | 3.2 ms | 4.3 ms | 8.4 ms |
| 1000 | 12 ms | 17 ms | 29 ms |

A design of 500 blocks recalculates in about 16 ms, and a design of 1000
blocks in about 60 ms. The wire routes are the part that grows fastest:
their time grows almost with the square of the number of blocks (103 ms for
2000 blocks). The router compares each bridge with the pins and the bridges
of all gutters that it crosses. The design is not for many thousands of
blocks.

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
   change and of its undo step.

A block of the canvas shows the ports and the generics of its module. When
these are different for a module that an instance uses, `AppState` sends
`modules_changed`. The window answers with `refresh_canvas`, through a
queued connection, and the canvas builds its items again. A reload builds
the canvas again itself; `refresh_canvas` then does nothing.

**Open** (GUI and CLI) has no earlier library to compare with, because the
design file holds no module definition. A connection that no longer agrees
with its module stays in the model, and validation reports it. The same
occurs after an undo that brings back a connection that a comparison
removed. In the two cases `dirty` is not set; the error of validation stops
codegen. The block has no pin for a port that is gone, so the context menu
of the block has "Remove Stale Connections"
(`Schematic::remove_stale_connections`): it removes each connection that
names such a port, as one undo step.

## Designs of a folder

A design is one `.hdlc` file. Its name is the name of the file and the name
of the module that it generates. The generated top file at its default
place, `<folder>/<name>.<ext>`, makes the design a module for each other
design of the folder. A user thus builds a hierarchy of designs and opens
no file dialog. `src/designs.rs` reads these facts from the folder; no file
holds them.

- **The list.** `folder_designs` gives each `.hdlc` file of the folder, not
  below it (a build directory holds copies). `AppState` reads the folder at
  an open, after a save with a new name, after Generate, and at a reload of
  the library. The sidebar shows the list; a click opens a design.
- **The offer.** The open design can place a design of the folder when that
  design has the same language, has generated HDL at the default place,
  and does not contain the open design (`FolderDesign::obstacle`). The
  library list shows such a design with the mark "(design)".
- **The placement.** `add_instance` with the module of such a design puts
  the generated top file, the generated group files and the library sources
  of the child into the `library_paths` of the parent, in the undo step of
  the instance. From then on the child is a module as each other module.
  The parent file holds each path itself. Thus a `.hdlc` file is complete
  alone, and the loader, the CLI and the build do not know the folder.
- **Names.** New and Save As give the design the name of its file. They
  refuse a name that is not a module name of the language and a name that
  another design of the folder has (`design_name`, `check_name_is_free`).
  File > New with a design open writes into the folder of that design. An
  older file whose module name is not its file name loads as before.
- **Guards.** `Schematic::validate` reports an instance of the design
  itself, an instance of a module that contains the design, and a module
  name that two sources define. Generate replaces only its own files (see
  "Code generation").
- **A child that changes.** The parent reads the generated HDL of the
  child, as it reads each library source. The window holds one design, so
  the parent is closed while the user changes the child. The parent thus
  gets the handling of an open (see "Library and module changes"): the
  connections stay, validation reports them, and the user removes them
  with "Remove Stale Connections".
- **The build.** A tool run of a parent takes `vhdl_libraries`,
  `extra_sources` and `exclude_sources` from the toolchain config of each
  design of the folder whose module the parent uses
  (`ToolchainConfig::load_with_children`). The flags, the VHDL standard,
  the board and the clocks come from the config of the parent only.

A group and a design are two things. A group is a fold in one design: it
has one use, and it has no `.hdlc` file of its own. A design has its file,
and other designs can use it many times.

## Undo and redo

Undo is a stack of snapshots, 100 entries at most. Each method that changes
the schematic calls `push_snapshot` before the change. `open_project`,
`recover`, and a save that gives the design a new name clear the two
stacks. A new design is an open of its new file.

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
  in place. `hdlc` can make them again. A generated top file at its default
  place is also an input: each design that places the design reads it.
- The text of a design file is a function of the design: each map is
  written with its keys in sorted order. Two saves of one design give the
  same file.
- The GUI shows the warnings of the loader in the diagnostics (prefix
  `load:`) until the next save. The loader can change the model: it removes
  a reference to an instance that is not in the file.
- Only a save of the user changes the design file. Between two saves, the
  GUI keeps a copy of the model in a recovery file,
  `.<design file name>.autosave` next to the design. A change starts a
  timer of 12 seconds. When the timer ends, `autosave` writes the file,
  and only when the model is different from the file. Many changes in that
  time give one write.
- A save removes the recovery file. Discard, in the question about changes
  that are not saved, removes it also. Thus the file is on the disk after
  an open only when the program stopped with changes that were not saved.
  The window then asks. Recover puts the changes into the model, as changes
  that are not saved. Discard removes the file.
- A panic in a method of the bridge stops the application (see "Known
  weaknesses", item 2).

## Tool flows

A tool command (`check`, `synth`, `sim`, `build`, `flash`) has three steps:

1. **Stage.** `toolchain::stage` generates the HDL and puts it, with the
   library sources, into `build/<top>/` next to the design. `hdlc` deletes
   that directory only when it made it (marker file `.hdlc-build`).
2. **Step table.** A pure function makes the list of commands for the
   language and the board (`check_stages`, `synth_stages`, `build_stages`,
   `netlist_stages`). A stage can have more than one variant, for a fallback
   (ghdl plugin, then ghdl netlist; `read_slang`, then `read_verilog -sv`).
   `hdlc build --dry-run` prints the table as a shell script.
3. **Run.** `run_stages` runs the commands, prints each one, and records it
   with its exit status in `steps.log`.

The settings of a design for the tools are in `<design>.toolchain.json`
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
  SystemVerilog, with nested groups). For the same schematics, the parser
  must give the interface of the design back from the generated top file:
  a parent design reads a child design from that file.
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
- `cargo test --features gui-tests` (`make test`) also runs
  `tests/gui_offscreen.rs`: the real `MainWindow` on the Qt `offscreen`
  platform, with clicks, drags and keys from Qt Test
  (`src/gui/gui_tests.cpp`). No window comes on the screen, and the
  settings of the test are in a temporary directory. The 25 tests cover
  the settings file of the tests, a wire from two clicks, the view of the
  canvas at undo, at Zoom to Fit and behind the recovery question, undo
  and redo from the menu, the drag of a block
  and of a selection, a click with a small move, the Delete key, the
  context menu, the color of a net, the pins of a
  changed source, the recovery file, the save key, the questions on close
  and on open, a new design in the folder, the list of designs, the
  placement of a design, and the removal of stale connections (see "Known
  weaknesses", item 3).
- `make test-ui` runs the package `ui-tests`: the real `hdlc` program on a
  headless screen, with the test library of the `headless-computer-use`
  project. A test does the actions of a user and reads the window and the
  files of a temporary folder. The 26 tests cover open, new, save,
  Generate HDL, the library and the list of designs, rename and delete,
  the question about changes that are not saved, the recovery file after
  a stop with `kill -9`, the recent designs, and on the canvas a drag, a
  wire from two clicks, and the Delete key. For these tests each block,
  pin and wire of the canvas has a name in `data(Qt::AccessibleTextRole)`
  (a wire: `<source key> -> <target key>`), and `HDLC_SETTINGS_DIR` gives
  the application a folder for its settings. The package is not part of
  the main package: its path dependency is not on each machine.

## Decisions that stay

| Decision | Reason |
|---|---|
| One in-process Rust model behind a cxx-qt bridge | One language for all rules; no protocol to a second process |
| Text and canvas are views of one model | No state to keep in step between views |
| A reference is connectivity only; nets by union-find | A hand-written file and the GUI can write a connection from either end |
| Module definitions are never saved | A changed source is always seen |
| A parent holds the paths of the files of a child | A `.hdlc` file is complete alone; the loader, the CLI and the build need no rule for the folder |
| The name of a design is the name of its file | Two designs of one folder never generate one file |
| One `generate_design` for the CLI and the GUI | The two cannot give different files |
| Snapshot undo | Simple and correct for a model of this size |
| Global route calculation on each change | The picture depends only on the model |
| External tools as subprocesses, from PATH | No tool is linked into the binary; the user controls the versions |
| An independent tool checks code generation | ghdl and slang accept a file that connects the wrong pins when the widths agree |

## Known weaknesses

A review on 2026-11-14 found eight weaknesses. The corrected ones are now in
the sections above (undo and redo, the net of a wire, the library
comparison, the build with no Qt, the save, the recovery file, the pins of
a changed module, the tests of the window, the time of the wire routes).
These stay:

1. **Editing rules are in the bridge.** `src/gui/bridge.rs` (about 4,600
   lines of code and 1,750 lines of tests) holds wiring rules, the grammar
   of the top-level buffer, undo and batching, as methods of the Qt object.
   A test reaches them only through `AppState`.
2. **A panic stops the application.** A panic in a method of the bridge
   stops the program. The recovery file keeps all changes but those of the
   last 12 seconds. The property test found no panic in 4,000 random
   sequences of edits; that is evidence, not proof.
3. **The tests of the GUI run the main flows only.** The offscreen tests
   and the tests of the application cover the window and the canvas. The
   text editors, the drag of a module from the library, the groups, the
   tool commands and the Chip View (`floorplan.h`) are checked by reading
   only.
4. **Routing is global.** Each change calculates all wire routes again, and
   that time grows almost with the square of the number of blocks (see
   "How a change flows"). A drag of a block calculates them one time, at
   the release. A drag of a top-level port calculates them at each move of
   the pointer.
5. **A design is a module through its generated file only.** A parent uses
   the HDL that a child generated last, and nothing shows that the child
   changed after that. The paths that a parent got at the placement do not
   follow a new library source or a new group of the child; a new instance
   of the child adds them. A switch between two designs is an open: it
   asks about changes that are not saved, and the undo history of the
   design that the user leaves is lost.

## Out of scope

- Mixed-language designs.
- Bus rippers and slice adapters beyond the inline slice forms.
- Manual positions for wire segments.
- Editing the inside of a library module that is an HDL source. (A module
  that is a design of the folder is edited as that design.)
- One stored library for a folder, several designs in memory, and a build
  of a parent that generates its children.

## Stack

- Rust 2024, one package. The tests of the application are a second
  package, `ui-tests`, that the main package does not use.
- Qt 6 (Widgets, Gui) through [cxx-qt](https://github.com/KDAB/cxx-qt) 0.8,
  only with the cargo feature `gui` (on by default). A build with
  `--no-default-features` is the CLI only and needs no Qt.
- `vhdl_lang` for VHDL and `sv-parser` for Verilog and SystemVerilog module
  headers.
- `serde` / `serde_json` for the design file, `schemars` for its schema,
  `clap` for the CLI, `tracing` for logs.
- `proptest` and `tempfile` for tests. Qt Test for the tests of the window,
  only with the cargo feature `gui-tests` (off by default). The test
  library of `headless-computer-use` for the package `ui-tests` only.
