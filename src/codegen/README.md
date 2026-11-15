# codegen

## Overview

`codegen` writes the structural HDL of a project. It takes a `Schematic` and
the module library and returns the text of each output file. It has one entry
point, `generate_design`, for flat projects and for projects with groups. The
CLI commands (`codegen`, `check`, `synth`, `sim`, `build`) and the GUI all use
this entry point.

The module has three files:

- `mod.rs`: `generate_design`, the shared helpers, and the errors.
- `vhdl.rs`: the VHDL emitter.
- `sv.rs`: the SystemVerilog emitter.

The emitters do not calculate connectivity. They read the nets from
`crate::nets::resolve_nets` and print them.

## Interface

Entry points:

- `generate_design(schematic, library) -> Result<GeneratedDesign, DesignError>`
  (`mod.rs`): generates each output file of a project.
- `GeneratedDesign` (`mod.rs`): `files` (file name and text of each group
  module), `top_filename`, `top_code`.
- `GeneratedDesign::write_to(top_path)` (`mod.rs`): writes the files to disk.
- `vhdl::generate_vhdl(schematic, library, diagnostics)` and
  `sv::generate_sv(schematic, library, diagnostics)`: one module from one
  schematic that has no group.

Errors:

- `CodegenError` (`mod.rs`): `ValidationErrors`, `DirtyInstances`.
- `DesignError` (`mod.rs`): `GroupDiagnostics`, `Group { name, source }`,
  `Top`.

Checks before generation:

- `check_errors(diagnostics)` (`mod.rs`)
- `check_no_dirty_instances(schematic)` (`mod.rs`)

Helpers that other modules use:

- `resolve_port_type(port_type, module_generics, instance_generic_map)`:
  the type of a port with the generics of one instance resolved.
- `find_top_port_context(schematic, library, top_port_name)`
- `TopIntermediate`, `collect_top_intermediates(schematic, nets)`
- `collect_internal_nets(nets)`
- `format_generic_value(value, generic_def, top_generics)`
- `port_type_to_vhdl`, `port_type_to_sv`, `port_type_to_vhdl_signal`,
  `port_type_to_sv_wire`
- Crate only: `signal_slice`, `port_type_mentions`, `generic_value_mentions`

## Invariants

- `generate_design` does not write to the filesystem
  [`mod.rs:generate_design`].
- An emitter stops when the diagnostics hold an error, and when an instance
  has the `dirty` flag [`mod.rs:check_errors`,
  `mod.rs:check_no_dirty_instances`; called first in
  `vhdl.rs:generate_vhdl` and `sv.rs:generate_sv`].
- For a project with groups, `generate_design` stops on an error of
  `groups::validate_groups` before it calls `groups::expand_hierarchy`
  [`mod.rs:generate_design`].
- `GeneratedDesign.files` has the module of a child group before the module
  of its parent group. This is the order that the HDL tools need
  [`mod.rs:GeneratedDesign`].
- A file name is the module name plus `.vhd` or `.sv`
  [`mod.rs:generate_design`].
- Each connected top-level port that is not `InOut` goes through an
  intermediate signal. Each reference in the module uses that signal, not the
  port [`mod.rs:collect_top_intermediates`].
- Declarations and port connections come from one result of `resolve_nets`
  for each module [`vhdl.rs:generate_vhdl`, `sv.rs:generate_sv`].
- Instances are written in the order of their names
  [`vhdl.rs:generate_vhdl`, `sv.rs:generate_sv`].
- VHDL: an instance of a module with no port gets no `port map`, because VHDL
  does not accept an empty one [`vhdl.rs:emit_instance`].
- A pin with no `port_map` entry that is on a net connects to that net. A pin
  on no net is `open` in VHDL [`vhdl.rs:emit_instance`].
- The first line of each output file says that the file is generated
  [`vhdl.rs:generate_vhdl`, `sv.rs:generate_sv`].

## Side Effects

- `GeneratedDesign::write_to` writes each group file and the top-level file
  with `std::fs::write`. It replaces a file that exists. The write is not
  atomic [`mod.rs:GeneratedDesign::write_to`].
- All other functions have no side effect.

## Verification

- Run: `cargo test`
- The round trip through yosys is in `tests/netlist_roundtrip.rs`. It needs
  yosys with the ghdl and slang plugins and skips with a message when they
  are missing.
