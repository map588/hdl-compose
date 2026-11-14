# hdl-compose

A block editor for structural HDL. Drop instances of your VHDL or
SystemVerilog modules onto a canvas, wire them together, and generate the
top-level entity/module that instantiates them. The generated file is plain
structural HDL: use it as your top level, or as a submodule in a larger
design.

The goal is narrow: make top-level wrapper authoring faster than hand-typing
port maps. It is not a synthesis tool, not IP-XACT, and not a waveform
viewer. It does integrate with the open-source toolchain (ghdl, verilator,
yosys, nextpnr, openFPGALoader) so a project can go from schematic to
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
make test                 # lib + integration + roundtrip tests
make help                 # all targets
```

`cargo build` alone also works; `cxx-qt-build` locates Qt via `qmake`.

On macOS, `make app` installs `/Applications/hdl-compose.app`. The bundle
starts from Finder and opens `.hdlc` files on double-click.

## Quick start

```sh
hdlc new demo -l vhdl --example   # working two-module example project
hdlc codegen demo.hdlc            # print the generated top level
hdlc check demo.hdlc              # elaborate it with ghdl (needs oss-cad-suite)
hdlc demo.hdlc                    # open it in the GUI
```

## CLI

```sh
hdlc [project.hdlc]               # launch GUI; no path reopens the most recent project
hdlc parse <file.vhd|.sv|.v>      # print a file's module ports + generics
hdlc new <name> -l vhdl|sv        # create a project (--example adds demo modules)
hdlc validate <project.hdlc>      # report diagnostics (exit 0/1/2)
hdlc codegen <project.hdlc> [-o out.vhd]
hdlc inspect <project.hdlc>       # summary + library status
hdlc migrate <a.hdlc> [b.hdlc ..] # rewrite older projects at the current version
hdlc schema                       # JSON Schema for the .hdlc format
hdlc guide                        # what an agent must know to write a project by hand
```

## Toolchain commands

These drive external tools against the generated HDL. They resolve tools from
PATH only, so activate your
[oss-cad-suite](https://github.com/YosysHQ/oss-cad-suite-build) environment
first (or have the individual tools installed).

```sh
hdlc check <project.hdlc>         # elaborate: ghdl (VHDL) / verilator --lint-only (SV)
hdlc synth <project.hdlc>         # generic yosys synthesis + cell stats
hdlc sim <project.hdlc> [--wave]  # run <top>_tb via ghdl / iverilog; --wave opens surfer or gtkwave
hdlc fpga <project.hdlc> --family ice40|ecp5|gowin   # emit a Makefile + constraints skeleton
hdlc build <project.hdlc> --board <x.board.json>     # bitstream: yosys, nextpnr, pack
hdlc flash <project.hdlc> --board <x.board.json>     # program via openFPGALoader (SRAM; --flash writes config flash)
```

Notes:

- `sim` generates an editable `<top>_tb` skeleton next to the project on
  first run and never overwrites an existing one.
- For VHDL synthesis the ghdl-yosys-plugin is tried first; if it fails on a
  construct it does not support, the tool falls back to `ghdl synth
  --out=verilog` and feeds the netlist to yosys, with a note in the output.
- `build` also writes `<top>.floorplan.json` next to the bitstream. It
  records where nextpnr placed each cell, the device utilization, and the
  maximum clock frequency. The GUI Chip View reads it.
- `fpga` writes a Makefile (codegen, yosys, nextpnr, pack, openFPGALoader)
  plus a per-port constraints placeholder. Edit the `DEVICE` / `PACKAGE`
  variables for your part. `--force` overwrites.

### Board definitions

`build` and `flash` take a board definition file: plain JSON you keep
wherever you like, typically next to the board's own repository.

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
Before place and route, `build` checks every top-level port against the
constraints file and warns about ports the board has no pin for. A working
example for the [Icepi Zero](https://github.com/cheyao/icepi-zero) is at
`tests/hardware/icepi-zero.board.json`. The constraints file next to it is an
unmodified copy from that repository (`gateware/icepi-zero.lpf`, revision
`f3da0ad`); its license is in `tests/hardware/icepi-zero.lpf.LICENSE`.

### Per-project toolchain config

The `.hdlc` file lists module sources; some projects need more than that for
the tools to work. Add `<name>.toolchain.json` next to `<name>.hdlc`:

```json
{
  "vhdl_libraries": [{ "name": "loot", "files": ["rtl/loot/midi_lut_pkg.vhd"] }],
  "extra_sources": ["rtl/registers/dflipflop.vhd", "sim/selectio_sim.vhd"],
  "exclude_sources": ["rtl/audio/clocked_data_out.v"],
  "ghdl_synth_flags": ["--latches"]
}
```

- `vhdl_libraries`: sources analyzed into a named library (`--work=<name>`),
  for packages and vendor stubs.
- `extra_sources`: files the tools need that the schematic does not reference
  directly (leaf dependencies of your modules, simulation-only
  implementations). Analyzed before the module sources.
- `exclude_sources`: library entries that only make sense in a vendor flow.
- `ghdl_synth_flags`: extra flags for ghdl during synthesis.

`tests/projects/hdlc/` is a complete example: a two-voice synthesizer with a
named library, mixed leaf dependencies, and both `.hdlc` projects checked and
synthesized by the tools above.

## GUI

The window opens maximized with the canvas front and center.

- **Left sidebar:** project tree (instances) above the module library. Drag a
  module from the library onto the canvas to add an instance, or double-click
  it. **File > Add HDL Folder** adds every HDL source below a directory.
- **Canvas:** instances as boxes, top-level ports on the edges, wires routed
  through gutters between columns. Click a pin, then click another pin to
  connect, or drag pin to pin. While a pin is armed (dashed cyan ring,
  crosshair cursor) you can also click any existing wire to join that net;
  clicking anything else cancels. Hold `Shift` and hover a pin or wire to
  highlight its whole net. `F` / `Ctrl+0` zoom to fit, `Ctrl`+wheel zoom,
  middle-drag pan,
  `Ctrl+L` tidies the layout (including top-port placement), `Esc` cancels,
  `Delete` removes the selection. Pinch on a trackpad zooms.
- **Toolchain menu:** Check, Synth, Build Bitstream, and Flash run the CLI
  commands above against the saved project and stream their output into a
  dock at the bottom. Select Board picks a `.board.json` and remembers it per
  project.
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
- **File > New** asks for the project name, the language, and where to save
  the `.hdlc` file. A project always has a file, so source paths stay
  relative to it.
- **File > Generate HDL** (Cmd+G) writes the structural wrapper to
  `<top>.vhd` / `<top>.sv` next to the project file, plus one file for each
  group. It does not ask for a path. **Generate HDL As...** (Cmd+Shift+G)
  lets you choose one.
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
project and is not editable from this view. Removing a port from the buffer
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

## Project file (`.hdlc`)

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

`library_paths` are relative to the project file's directory; run the CLI
from there.

## Stack

- Rust 2024
- Qt 6 via [cxx-qt](https://github.com/KDAB/cxx-qt)
- [vhdl_lang](https://crates.io/crates/vhdl_lang) for VHDL parsing
- [sv-parser](https://crates.io/crates/sv-parser) for Verilog/SV parsing
- serde + serde_json (project file), clap (CLI), tracing (logging)

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for the design discussion: data
model, net identity, mini-editor grammar, codegen rules, and the central
text-vs-canvas decision.

## Known limitations

- One language per project: VHDL or SystemVerilog, never both. A Verilog
  file can sit in a VHDL project's library for a vendor flow, but the
  open-source tool commands need a `toolchain.json` exclusion for it.
- The `sim` command expects `<top>_tb` next to the project file; projects
  that keep testbenches elsewhere run them outside the tool for now.
- Tool discovery is PATH-only. The GUI runs toolchain commands through your
  login shell, so the tools must be on the PATH that your shell profile sets
  (for example, source the oss-cad-suite `environment` file there).
- No bus rippers or slice adapters beyond inline `[h:l]` / `(h downto l)`.

## License

MIT. See [LICENSE](LICENSE).
