# hdl-compose

Structural HDL composition tool. Drop instances of user-authored VHDL/Verilog
modules onto a canvas, wire them, generate a clean structural top-level entity.

Not a synthesis tool, not IP-XACT, not a waveform viewer. One job: make
top-level wrapper authoring faster than hand-typing for the flat case, in both
VHDL and SystemVerilog.

## Status

Pre-1.0. VHDL path is exercised; SV path has codegen + parser but less manual
verification. macOS is the dev target (Qt 6 from homebrew). Linux should work
with Qt 6 installed, untested.

## Build

Requires Rust 2024 edition, Qt 6.

```sh
brew install qt           # macOS, provides qmake6
make build                # cargo build (debug)
make run                  # launch GUI
make test                 # full test suite (lib + integration + roundtrips)
make help                 # all targets
```

`cargo build` alone works — `cxx-qt-build` handles Qt linking via `qmake`.

## CLI

```sh
hdl-compose                              # launch GUI (default)
hdl-compose parse <file.vhd|.sv|.v>      # dump module ports + generics
hdl-compose new <name> -l vhdl|sv        # create empty .hdlc project
hdl-compose validate <project.hdlc>      # report diagnostics
hdl-compose codegen <project.hdlc> [-o out.vhd]
hdl-compose inspect <project.hdlc>       # summary + library status
hdl-compose migrate <a.hdlc> [b.hdlc ..] # rewrite older projects at the current version
```

### oss-cad-suite integration

These subcommands drive external tools against the generated HDL. They resolve
tools from PATH only — activate your [oss-cad-suite](https://github.com/YosysHQ/oss-cad-suite-build)
environment first (`source <oss-cad-suite>/environment`).

```sh
hdl-compose check <project.hdlc>         # elaborate generated HDL (ghdl / verilator --lint-only)
hdl-compose synth <project.hdlc>         # generic yosys synth + stat (ghdl plugin for VHDL)
hdl-compose sim <project.hdlc> [--wave]  # run <top>_tb via ghdl / iverilog; --wave opens surfer/gtkwave
hdl-compose fpga <project.hdlc> --family ice40|ecp5|gowin  # emit Makefile + constraints skeleton
```

`sim` generates an editable `<top>_tb` skeleton next to the project on first
run. `fpga` writes a `Makefile` (codegen → yosys → nextpnr → pack →
openFPGALoader) and a per-port constraints placeholder; edit the `DEVICE` /
`PACKAGE` variables for your board (`--force` overwrites).

## GUI in 30 seconds

- **Left sidebar:** project tree (instances) + library pane (parsed modules).
  Drag from library to canvas to add an instance.
- **Canvas:** instances as boxes, top-level ports on the edges, wires routed
  through the gutters between columns. Click a port → click another port to
  connect (or drag pin-to-pin). Drag instances; wires reroute live. Hover a
  wire to light up its whole net. `F` / `Ctrl+0` zoom-to-fit, `Ctrl`+wheel
  zoom, middle-drag pan, `Esc` cancels, `Delete` removes the selection,
  `Ctrl+\` restores the editor panel.
- **Right pane mini-editor:** per-instance VHDL-shaped `port map` /
  `generic map`. Edits commit on Ctrl+Return or focus-out. Tab-complete on RHS.
  Toggle "Top Level" to edit the top entity's port + generic list directly.
- **File → Generate HDL...** (Cmd+G) writes the structural wrapper.

## Top-level mini-editor

Free-form grammar — parser ignores `entity ... is`, `port (`, `);`,
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

Lines with `in`/`out`/`inout` after `:` are ports; everything else with `:` is
a generic. Slice notation `[high:low]` matches SV; legacy `std_logic_vector(7
downto 0)` and `slv(7:0)` parse too. Top entity name is taken from the
project; not editable from this view.

Removing a port from the buffer cascade-clears any instance `port_map` entries
that referenced it.

## Codegen model

Each top-level port routes through an intermediate signal: codegen emits
`<port>_s <= <port>;` for inputs (or `<port> <= <port>_s;` for outputs) and
all instance port-maps reference `<port>_s`. Aliases (right-click a wire) take
precedence over the default `_s` suffix. `inout` ports skip the intermediate
to avoid multi-driver conflicts.

Generic-derived ranges in promoted top-port types resolve to literals at gen
time using the driving instance's `generic_map` (e.g.
`std_logic_vector(WIDTH-1 downto 0)` with `WIDTH=8` → `(7 downto 0)`).
Component declarations stay symbolic — child parameterization survives.

## Project file (`.hdlc`)

JSON via serde. Schema version 4 (adds per-port `consumer_slices`; v2–v3
load with defaults, `hdl-compose migrate` rewrites them in place).
`ModuleDef` is never persisted — always re-derived from source on load
(re-parse detection stays honest).

```json
{
  "version": 4,
  "top_name": "my_top",
  "language": "Vhdl",
  "top_generics": [...],
  "top_ports": [...],
  "instances": [...],
  "aliases": {...},
  "library_paths": ["rtl/fifo_sync.vhd", ...]
}
```

## Stack

- Rust 2024
- Qt 6 via [cxx-qt 0.8](https://github.com/KDAB/cxx-qt)
- [vhdl_lang 0.86](https://crates.io/crates/vhdl_lang) — VHDL parsing
- [sv-parser 0.13](https://crates.io/crates/sv-parser) — Verilog/SV parsing
- serde + serde_json — project file
- clap — CLI
- tracing — logging

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for the full design discussion: data
model, net identity, mini-editor grammar, codegen rules, and the central
text-vs-canvas decision.

## Out of scope (for v1)

- Hierarchical sub-schematics
- Mixed-language projects (each project is VHDL **or** SV, never both)
- Bus rippers / slice adapters beyond inline `[h:l]` / `(h downto l)`
- Simulator / linter integration

## License

TBD.
