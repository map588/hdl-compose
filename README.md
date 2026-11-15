# hdl-compose

A block editor for structural HDL. Drop instances of your VHDL or
SystemVerilog modules onto a canvas, wire them together, and generate the
top-level entity/module that instantiates them. The generated file is plain
structural HDL: use it as your top level, or as a submodule in a larger
design. A design that you generate is a module for each other design in its
folder, so you build a hierarchy of designs with no file dialog.

The goal is narrow: make top-level wrapper authoring faster than hand-typing
port maps. It is not a synthesis tool, not IP-XACT, and not a waveform
viewer. It does integrate with the open-source toolchain (ghdl, verilator,
yosys, nextpnr, openFPGALoader) so a design can go from schematic to
elaboration check, simulation, or a flashed FPGA without leaving the tool.

## Status

Pre-1.0, developed and tested on macOS with Qt 6 from Homebrew. Linux should
build with Qt 6 installed but is not regularly tested. The VHDL path is the
most exercised; SystemVerilog has full codegen and a parser but has seen less
real-world use. The FPGA flow has been verified end to end on one board
(Icepi Zero, Lattice ECP5). Expect rough edges and report them.

## Build

Requires Rust (2024 edition) and Qt 6.

```sh
brew install qt           # macOS; provides qmake6
make build                # cargo build (debug)
make run                  # launch the GUI
make test                 # all tests, with the offscreen tests of the GUI
make help                 # all targets
```

`cargo build` alone also works; `cxx-qt-build` locates Qt via `qmake`.

The GUI is the cargo feature `gui`, which is on by default. A build with
no Qt gives the CLI only:

```sh
cargo build --no-default-features   # no Qt necessary; `hdlc gui` reports an error
cargo test --no-default-features    # all tests but those of the GUI
```

The cargo feature `gui-tests` adds the tests of the window. They run on the
Qt `offscreen` platform with input from Qt Test, so no window comes on the
screen:

```sh
cargo test --features gui-tests     # `make test` runs this
```

On macOS, `make app` installs `/Applications/hdl-compose.app`. The bundle
starts from Finder and opens `.hdlc` files on double-click.

## Quick start

```sh
hdlc new demo -l vhdl --example   # working two-module example design
hdlc codegen demo.hdlc            # print the generated top level
hdlc check demo.hdlc              # elaborate it with ghdl (needs oss-cad-suite)
hdlc demo.hdlc                    # open it in the GUI
```

## CLI

```sh
hdlc [design.hdlc]                # launch GUI; no path reopens the most recent design
hdlc parse <file.vhd|.sv|.v>      # print a file's module ports + generics
hdlc new <name> -l vhdl|sv        # create a design (--example adds demo modules)
hdlc validate <design.hdlc>       # report diagnostics (exit 0/1/2)
hdlc codegen <design.hdlc> [-o out.vhd]
hdlc inspect <design.hdlc>        # summary + library status
hdlc migrate <a.hdlc> [b.hdlc ..] # rewrite older designs at the current version
hdlc schema                       # JSON Schema for the .hdlc format
hdlc guide                        # what an agent must know to write a design by hand
```

## Toolchain commands

These drive external tools against the generated HDL. They resolve tools from
PATH only, so activate your
[oss-cad-suite](https://github.com/YosysHQ/oss-cad-suite-build) environment
first (or have the individual tools installed).

```sh
hdlc doctor                       # which tools are on PATH, their versions, the commands that run them
hdlc check <design.hdlc>          # elaborate: ghdl (VHDL) / slang (SV)
hdlc synth <design.hdlc>          # generic yosys synthesis + cell stats
hdlc sim <design.hdlc> [--wave]   # run <top>_tb via ghdl / iverilog; --wave opens surfer or gtkwave
hdlc fpga <design.hdlc> --family ice40|ecp5|gowin    # start a board: Makefile, board file, constraints skeleton
hdlc build <design.hdlc> [--board <x.board.json>]    # bitstream: yosys, nextpnr, pack
hdlc flash <design.hdlc> [--board <x.board.json>]    # program via openFPGALoader (SRAM; --flash writes config flash)
```

Notes:

- The tool commands copy the sources to `build/<top>/` next to the design
  and run the tools there. The directory keeps the intermediate files until
  the next run: the netlist (`design.json`), the nextpnr output (`pnr.json`,
  `report.json`), `yosys.log`, `nextpnr.log`, and `steps.log` (each command
  line and its exit status). `hdlc` deletes the directory at the start of
  each run. It stops with an error when the directory holds files and no
  `.hdlc-build` marker, because then `hdlc` did not make it.
- `build --dry-run` runs no tool. It copies the sources to the build
  directory and prints the tool commands as a shell script
  (`hdlc build x.hdlc --dry-run > build.sh`). The commands of a fallback are
  comments. The script makes the bitstream but not the chip view file.
- `check` has a second part, the netlist check. yosys reads only the
  generated files, with each library module as a black box, and writes the
  netlist. `hdlc` compares its connections, bit for bit, with the nets of
  the design, with the groups flattened. A difference fails `check` (exit
  code 1) and is a defect of hdlc, not of your design. The part is skipped,
  with a message, when yosys or its plugin is missing, or when the generated
  files do not elaborate alone (a port with a type from your own package).
  Constant ties, and nets with a pin of no known width, are not compared.
- `sim` generates an editable `<top>_tb` skeleton next to the design on
  first run and never overwrites an existing one.
- For VHDL synthesis the ghdl-yosys-plugin is tried first; if it fails on a
  construct it does not support, the tool falls back to `ghdl synth
  --out=verilog` and feeds the netlist to yosys, with a note in the output.
- For SystemVerilog synthesis the yosys slang plugin (`read_slang`) is tried
  first. It reads packages, structs and interfaces. If it fails, the tool
  falls back to `read_verilog -sv`, with a note in the output.
- `check` on a SystemVerilog design runs `slang` when it is on PATH, with a
  port connection of the wrong width as an error. Without `slang` it runs
  `verilator --lint-only`. slang is stricter about the language than
  verilator; verilator reports more style warnings inside your modules.
- `build` also writes `<top>.floorplan.json` next to the bitstream. It
  records where nextpnr placed each cell, the device utilization, and the
  maximum clock frequency. The GUI Chip View reads it.
- `fpga` starts a board for a design. It writes `<top>.board.json` with a
  usual part of the family, a constraints placeholder with one line for
  each top-level port, and a Makefile: `make` runs `hdlc build`, `make prog`
  runs `hdlc flash`. It also sets the `board` key of the toolchain config
  when the config names no board. Set `device` and `package` in the board
  file for your part and fill in the pins; nextpnr stops on a port without
  a pin. The Makefile holds no tool command of its own, so the flow has one
  definition; `hdlc build --dry-run` prints the commands. `--force`
  overwrites the three files.

### Board definitions

`build` and `flash` take a board definition file: plain JSON you keep
wherever you like, typically next to the board's own repository. Name the
file with `--board`, or one time with the `board` key of the
[toolchain config](#per-design-toolchain-config) of the design. The flag
has priority.

```json
{
  "name": "icepi-zero",
  "family": "ecp5",
  "device": "25k",
  "package": "CABGA256",
  "constraints": "icepi-zero.lpf",
  "pack_args": ["--compress"],
  "prog_args": ["-b", "icepi-zero"]
}
```

`constraints` (.lpf/.pcf/.cst) resolves relative to the board file.
`pack_args` go to ecppack/icepack/gowin_pack, `prog_args` to openFPGALoader.
The optional `pnr_args` go to nextpnr; a Gowin board uses them for
`["--vopt", "family=GW1N-9C"]`.
Before place and route, `build` checks every top-level port against the
constraints file and warns about ports the board has no pin for. A working
example for the [Icepi Zero](https://github.com/cheyao/icepi-zero) is at
`tests/hardware/icepi-zero.board.json`. The constraints file next to it is an
unmodified copy from that repository (`gateware/icepi-zero.lpf`, revision
`f3da0ad`); its license is in `tests/hardware/icepi-zero.lpf.LICENSE`.

### Per-design toolchain config

The `.hdlc` file lists module sources; some designs need more than that for
the tools to work. Add `<name>.toolchain.json` next to `<name>.hdlc`:

```json
{
  "vhdl_libraries": [{ "name": "loot", "files": ["rtl/loot/midi_lut_pkg.vhd"] }],
  "extra_sources": ["rtl/registers/dflipflop.vhd", "sim/selectio_sim.vhd"],
  "exclude_sources": ["rtl/audio/clocked_data_out.v"],
  "ghdl_synth_flags": ["--latches"],
  "board": "boards/icepi-zero.board.json",
  "clocks": { "clk": 50 },
  "synth_args": ["-abc9"],
  "pnr_args": ["--seed", "3"],
  "vhdl_std": "08"
}
```

Each key is optional. Paths are relative to the folder of the design.

- `vhdl_libraries`: sources analyzed into a named library (`--work=<name>`),
  for packages and vendor stubs.
- `extra_sources`: files the tools need that the schematic does not reference
  directly (leaf dependencies of your modules, simulation-only
  implementations). Analyzed before the module sources.
- `exclude_sources`: library entries that only make sense in a vendor flow.
- `ghdl_synth_flags`: extra flags for ghdl during synthesis.
- `board`: the board definition file for `build` and `flash`.
- `clocks`: top-level port name to clock frequency in MHz. `build` gives
  them to nextpnr as an SDC file (`build/<top>/clocks.sdc`), which each FPGA
  family reads. An ECP5 constraints file with a `FREQUENCY PORT` line for
  the same port has priority in nextpnr; `build` prints a warning then.
- `synth_args`: extra options for the yosys `synth_<family>` command of
  `build`. `hdlc synth` runs the generic `synth` command, which does not
  know all of them, so it does not use them.
- `pnr_args`: extra arguments for nextpnr, after the `pnr_args` of the
  board.
- `vhdl_std`: the VHDL standard for ghdl (`--std=<value>`). Default `08`.

A design that uses other designs of its folder does not repeat their
entries. A tool run takes `vhdl_libraries`, `extra_sources` and
`exclude_sources` from the config of each design of the folder whose module
the design uses. The other keys (the flags, `vhdl_std`, the board, the
clocks) come from the config of the design itself only.

`tests/projects/hdlc/` is a complete example: a two-voice synthesizer with a
named library, mixed leaf dependencies, and both `.hdlc` designs checked and
synthesized by the tools above. `synth_top` uses the design `voice_chain`.

## GUI

The window opens maximized with the canvas front and center.

- **Left sidebar:** the designs of the folder, the instance tree, and the
  module library. Drag a module from the library onto the canvas to add an
  instance, or double-click it. **File > Add HDL Folder** adds every HDL
  source below a directory.
- **Designs:** the list shows each `.hdlc` file in the folder of the open
  design, with the open design in bold. Click another design to open it;
  the GUI asks first when the open design has changes that are not saved.
  **Generate HDL** makes the open design a module of the folder. A design
  that has generated HDL and the same language is then in the library list
  of each other design, with the mark "(design)". Place it as any module:
  the block has the top-level ports of that design, and its generated files
  and its library sources go into the library of the open design, in the
  same undo step. A note in the list says why a design is not in the
  library: "not generated", "other language", or "contains this design".
- **A child that changed:** a parent reads the generated HDL of a child, so
  it shows the child as of its last Generate. When a port of the child is
  gone, the next open of the parent reports each connection to that port,
  and Generate HDL stops. Right-click the block and choose **Remove Stale
  Connections** to remove them.
- **Canvas:** instances as boxes, top-level ports on the edges, wires routed
  through gutters between columns. Drag a block to move it. The block
  follows the pointer, and its wires show as straight lines. When you
  release it, the block goes to the nearest column, the blocks in its way
  make room, and the wires get their routes. Click a pin, then click
  another pin to connect, or drag pin to pin. While a pin is armed (dashed
  cyan ring,
  crosshair cursor) you can also click any existing wire to join that net;
  clicking anything else cancels. Hold `Shift` and hover a pin or wire to
  highlight its whole net. `F` / `Ctrl+0` zoom to fit, `Ctrl`+wheel zoom,
  middle-drag pan,
  `Ctrl+L` tidies the layout (including top-port placement), `Esc` cancels,
  `Delete` removes the selection. Pinch on a trackpad zooms.
- **Toolchain menu:** Check, Synth, Build Bitstream, and Flash run the CLI
  commands above against the saved design and stream their output into a
  dock at the bottom. Select Board picks a `.board.json` and writes it as
  the `board` key into the toolchain config of the design, so the CLI uses
  the same board. This rewrites `<name>.toolchain.json` in the standard
  layout of the JSON writer (sorted keys); each other key keeps its value.
- **Toolchain > Chip View** shows the device after place and route. Free
  bels are grey outlines. Each tile that holds cells of the design has a
  colored square: logic, register, I/O, DSP, memory, or clock. Select an
  instance on the canvas to mark its tiles on the chip. Click a tile to
  select the instance that its cells come from. Hover a tile to list its
  cells. `Ctrl`+wheel or pinch zooms, drag pans, double-click fits. Build
  Bitstream opens it. The view shows placement, not routing.
  The list above the device holds the slowest path for each clock pair, with
  its delay and its slack against the constraint. The view draws the chosen
  path: a line for a net between two tiles, a ring for a delay inside a
  cell, from yellow to red with the share of the delay. The hops are in the
  list below the device with the net name and its HDL source line. Click a
  hop to select the instance that owns its cells.
- **Editor panel** (hidden by default, `Ctrl+\` or the Editor toolbar
  button): per-instance `port map` / `generic map` in a VHDL-shaped text
  buffer. Commits on Ctrl+Return or focus-out, Tab completes signal names.
  The Top Level toggle edits the top entity's ports and generics the same
  way.
- **File > New** asks for the design name and the language. With a design
  open, the new design goes into the same folder, with no dialog for the
  place; with no design open, a dialog asks where to save. The name of a
  design is the name of its `.hdlc` file and of its module, so it must be a
  name of the language, and no other design of the folder can have it. A
  design always has a file, so source paths stay relative to it.
- **Save and recovery:** only **File > Save** (Cmd+S) and **Save As...**
  (Cmd+Shift+S) change the `.hdlc` file. While a design has changes that
  are not saved, the GUI keeps them in a recovery file,
  `.<name>.hdlc.autosave` next to the design. It writes that file 12
  seconds after a change, and one time for all changes in those 12
  seconds. A save removes the recovery file. When you close a design with
  changes that are not saved, the GUI asks: Save, Discard or Cancel. When
  the program stopped with changes that were not saved, the next open of
  the design offers to recover them. **Save As...** with a new file name
  gives the design that name, and the undo history starts again.
- **File > Generate HDL** (Cmd+G) writes the structural wrapper to
  `<top>.vhd` / `<top>.sv` next to the design file, plus one file for each
  group. It does not ask for a path. **Generate HDL As...** (Cmd+Shift+G)
  lets you choose one; that file is an export, and only the file at the
  default place makes the design a module of its folder. Generate replaces
  only a file that hdl-compose generated: a hand-written source with the
  name of the design or of a group stays, and Generate reports it.
- **Goto Source** (right-click an instance) opens the module's source file
  with a command you configure in Preferences. The command runs through your
  shell exactly as typed in a terminal; `{file}` is replaced with the source
  path. Examples: `code -g {file}`, `zed {file}`, `kitty -e nvim {file}`.

## Top-level mini-editor

Free-form grammar; the parser ignores `entity ... is`, `port (`, `);`, and
`end entity` decoration:

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

Lines with `in`/`out`/`inout` after `:` are ports; everything else with `:`
is a generic. Slice notation `[high:low]` matches SV; `std_logic_vector(7
downto 0)` and `slv(7:0)` parse too. The top entity name comes from the
design and is not editable from this view. Removing a port from the buffer
also clears any instance `port_map` entries that referenced it.

## Codegen model

Each top-level port routes through an intermediate signal: codegen emits
`<port>_s <= <port>;` for inputs (or `<port> <= <port>_s;` for outputs) and
all instance port maps reference `<port>_s`. Aliases (right-click a wire)
replace the default `_s` name. `inout` ports connect directly, skipping the
intermediate, to avoid multi-driver conflicts.

Generic-derived ranges in promoted top-port types resolve to literals at
generation time using the driving instance's `generic_map` (for example
`std_logic_vector(WIDTH-1 downto 0)` with `WIDTH=8` becomes `(7 downto 0)`).
Component declarations stay symbolic so child parameterization survives.

Instances can be grouped (right-click a selection); each group becomes its
own generated module file, and the top level instantiates it.

## Design file (`.hdlc`)

JSON via serde. Current schema version is 5 (adds hierarchical groups);
versions 2 through 4 load with defaults and `hdlc migrate` rewrites
them in place. Module definitions are never persisted; they are re-derived
from source on every load so the tool notices when a source file changed.

```json
{
  "version": 5,
  "top_name": "my_top",
  "language": "Vhdl",
  "top_generics": [...],
  "top_ports": [...],
  "instances": [...],
  "aliases": {...},
  "library_paths": ["rtl/fifo_sync.vhd", ...],
  "groups": [...]
}
```

`library_paths` are relative to the design file's directory; run the CLI
from there.

## Stack

- Rust 2024
- Qt 6 via [cxx-qt](https://github.com/KDAB/cxx-qt)
- [vhdl_lang](https://crates.io/crates/vhdl_lang) for VHDL parsing
- [sv-parser](https://crates.io/crates/sv-parser) for Verilog/SV parsing
- serde + serde_json (design file), clap (CLI), tracing (logging)

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for the design discussion: the data
model, nets and groups, the split between Rust and C++, how a change flows,
the tool flows, and the known weaknesses.

## Local Interface & Invariants

The files in the repository root that have an effect on the build:

- `Cargo.toml`: one package, `hdl-compose`, with the library `hdl_compose`
  (`src/lib.rs`) and the binary `hdlc` (`src/main.rs`). The cargo feature
  `gui` is on by default. Only with it, the build has the module `gui`,
  the cxx-qt dependencies and the C++ code, and needs Qt 6. With no `gui`
  feature, `hdlc gui` and `hdlc` with no command return exit code 2
  [`Cargo.toml`, `src/lib.rs`: `pub mod gui`, `src/main.rs:cmd_gui`].
- `build.rs`: with the `gui` feature, gives `src/gui/bridge.rs` to cxx-qt
  and compiles `src/gui/app.cpp`, `src/gui/canvas.cpp` and
  `src/gui/items.cpp` against the Qt modules Widgets and Gui. The list of
  C++ files is explicit: a new `.cpp` file must be added there. With no
  `gui` feature it does nothing [`build.rs:main`].
- `.cargo/config.toml`: holds the C++ flags for the macOS arm64 workaround
  that `build.rs` names in its comment.
- `Makefile`: `build`, `release`, `run` / `gui`, `check`, `test`, `fmt`,
  `clippy`, `clean`, and the macOS bundle targets `app`, `app-debug`,
  `app-release`, `app-bundle`.

## Subdirectories & System Boundaries

- src: [src/README.md](./src/README.md) - the Rust package: the model, the
  net resolution, groups, code generation, the tool flows, the CLI, and the
  Qt application. Its children have their own contracts:
  [src/codegen/README.md](./src/codegen/README.md) and
  [src/gui/README.md](./src/gui/README.md).
- tests: integration and round-trip tests, fixtures, and the example
  folder `tests/projects/hdlc/` with two designs (no contract file).
- docs: `AGENT_GUIDE.md`, `FORMAT.md` and `hdlc.schema.json`; `hdlc guide`
  and `hdlc schema` print them (no contract file).
- packaging: the macOS `Info.plist` of the application bundle (no contract
  file).

## Global Invariants

- The `Schematic` in memory is the one model. The CLI and the GUI generate
  HDL through the same function, `codegen::generate_design`, and write it
  through `GeneratedDesign::write_to`
  [`src/codegen/mod.rs:generate_design`, `src/gui/bridge.rs:generate_code`].
- A `port_map` entry states that two pins are connected; its direction has
  no effect. The nets come from `nets::resolve_nets` (union-find), and
  codegen and validation both use that result [`src/nets.rs`].
- The generated HDL must connect what the schematic connects. `hdlc check`
  compares the two through yosys, and `tests/netlist_roundtrip.rs` does the
  same for random designs [`src/netcheck.rs:compare`].
- A group must not change what is connected [`src/netcheck.rs:expected`,
  `src/groups.rs:expand_hierarchy`].
- A design has one language, VHDL or SystemVerilog
  [`src/types.rs:Language`, field `Schematic.language`].
- A design is one `.hdlc` file, and its name is the name of that file. Its
  generated top file at `<folder>/<name>.<ext>` is a module for each other
  design of the folder. No file holds the list of the designs: the folder
  is the list [`src/designs.rs:folder_designs`,
  `src/designs.rs:FolderDesign::module_file`].
- A design never contains itself, and one module name has one source
  [`src/schematic.rs:Schematic::validate`].
- Generate replaces only a file that hdlc generated
  [`src/codegen/mod.rs:GeneratedDesign::write_to`].
- The `.hdlc` file holds no module definition. The loader reads the modules
  from the library sources. It accepts format versions 2 to 5
  [`src/project.rs:load_project`].
- The text of a `.hdlc` file is a function of the design: each map is
  written with its keys in sorted order, so two saves of one design give
  the same file [`src/types.rs:serialize_sorted`].
- Decisions are in Rust; the C++ side draws and takes input. Placement and
  routing are in `src/routing.rs`, which has no Qt type
  [`src/routing.rs` module comment].
- The tool commands of the GUI start the `hdlc` binary as a subprocess.
  They do not go through the bridge [`src/gui/app.cpp:MainWindow`].
- The external tools come from PATH only [`src/toolchain.rs` module
  comment].
- `hdlc` deletes a build directory only when it is empty or holds the
  marker file `.hdlc-build` [`src/toolchain.rs:reset_build_dir`].

## Global Side Effects

- **Files next to a design:** the `.hdlc` file, `<name>.toolchain.json`,
  the recovery file `.<name>.hdlc.autosave` (only while the GUI has changes
  that are not saved) [`src/gui/bridge.rs:autosave`], the generated
  `<top>.vhd` or `<top>.sv` and one file for each group, the directory
  `build/<top>/`, the bitstream, `<top>.floorplan.json`, the
  testbench `<top>_tb`, and the wave file of a simulation.
- **Writes:** the design file and `<name>.toolchain.json` are replaced in
  one step, so a failure during a save leaves the old file complete
  [`src/lib.rs:replace_file`, `src/project.rs:save_project`,
  `src/toolchain.rs:ToolchainConfig::set_board`]. The generated files are
  written in place with `std::fs::write`; `hdlc` can make them again
  [`src/codegen/mod.rs:GeneratedDesign::write_to`].
- **External processes:** ghdl, slang, verilator, yosys, nextpnr, the pack
  tools, iverilog, surfer or gtkwave, and the editor command of the user.
  `hdlc flash` programs hardware with openFPGALoader.
- **User settings:** the GUI keeps the recent designs, the window layout
  and the preferences with `QSettings` [`src/gui/app.cpp`].
- **`make app`, `app-debug`, `app-release`:** copy the binary into
  `/Applications/hdl-compose.app`, stop each running `hdlc` process, and
  register the bundle with the operating system [`Makefile`].

## System Verification

- Run: `cargo test`
- With the offscreen tests of the window: `cargo test --features gui-tests`
- Run: `cargo fmt --check`
- Run: `cargo clippy --all-targets -- -D warnings`
- With no Qt: `cargo test --no-default-features` and
  `cargo clippy --all-targets --no-default-features -- -D warnings`

## Known limitations

- One language per design: VHDL or SystemVerilog, never both. A Verilog
  file can sit in a VHDL design's library for a vendor flow, but the
  open-source tool commands need a `toolchain.json` exclusion for it.
- The `sim` command expects `<top>_tb` next to the design file; designs
  that keep testbenches elsewhere run them outside the tool for now.
- Tool discovery is PATH-only. The GUI runs toolchain commands through your
  login shell, so the tools must be on the PATH that your shell profile sets
  (for example, source the oss-cad-suite `environment` file there).
- No bus rippers or slice adapters beyond inline `[h:l]` / `(h downto l)`.
- A parent design uses the HDL that a child design generated last. Nothing
  shows that the child was changed after that; generate the child again.
- The window holds one design. A switch to another design of the folder is
  an open: it asks about changes that are not saved, and the undo history
  of the design that you leave is lost.
- A parent holds the paths of the files of a child from the time that you
  placed it. When the child gets a new library source or a new group, place
  an instance of the child again; that adds the new paths.

## License

MIT. See [LICENSE](LICENSE).
