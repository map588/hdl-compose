## ADDED Requirements

### Requirement: SystemVerilog codegen output is parseable
For every fixture module that the SystemVerilog parser accepts, the SystemVerilog codegen output for a passthrough schematic of that module SHALL be accepted without error by the same SystemVerilog parser.

#### Scenario: SV passthrough wrapper for `counter.v` re-parses
- **WHEN** `parse_file("tests/fixtures/counter.v")` returns a `ModuleDef` for `counter`
- **AND** a passthrough schematic is built that promotes every `counter` port to a top-level port and wires every instance port through to its same-named top port
- **AND** `codegen::sv::generate_sv` is invoked on that schematic with the original module in the library and an empty diagnostics list
- **THEN** the resulting text SHALL parse without error via `parse_file` against a temp file with extension `.sv`

### Requirement: VHDL codegen output is parseable
For every fixture entity that the VHDL parser accepts, the VHDL codegen output for a passthrough schematic of that entity SHALL be accepted without error by the same VHDL parser.

#### Scenario: VHDL passthrough wrapper for `counter.vhd` re-parses
- **WHEN** `parse_file("tests/fixtures/counter.vhd")` returns a `ModuleDef` for `counter`
- **AND** a passthrough schematic is built that promotes every `counter` port to a top-level port and wires every instance port through to its same-named top port
- **AND** `codegen::vhdl::generate_vhdl` is invoked on that schematic with the original module in the library and an empty diagnostics list
- **THEN** the resulting text SHALL parse without error via `parse_file` against a temp file with extension `.vhd`

#### Scenario: VHDL passthrough wrapper for `fifo_sync.vhd` re-parses
- **WHEN** `parse_file("tests/fixtures/fifo_sync.vhd")` returns a `ModuleDef` for `fifo_sync`
- **AND** a passthrough schematic is built that promotes every `fifo_sync` port to a top-level port and wires every instance port through to its same-named top port
- **AND** `codegen::vhdl::generate_vhdl` is invoked on that schematic with the original module in the library and an empty diagnostics list
- **THEN** the resulting text SHALL parse without error via `parse_file` against a temp file with extension `.vhd`

### Requirement: Round-trip preserves port shape
For every fixture module, the regenerated top module MUST expose the same set of ports as the original parsed module — comparing port name, direction, and port_type, ignoring source order and ignoring the auto-detected `bundle` field.

#### Scenario: SV port shape survives a round-trip
- **WHEN** the SV passthrough wrapper for a fixture is generated and re-parsed
- **AND** the regenerated top module is located in the re-parsed `Vec<ModuleDef>` by its `<original>_passthrough` name
- **THEN** the multiset of `(port.name, port.direction, port.port_type)` triples on the regenerated module SHALL equal the multiset on the original module

#### Scenario: VHDL port shape survives a round-trip
- **WHEN** the VHDL passthrough wrapper for a fixture is generated and re-parsed
- **AND** the regenerated top module is located in the re-parsed `Vec<ModuleDef>` by its `<original>_passthrough` name
- **THEN** the multiset of `(port.name, port.direction, port.port_type)` triples on the regenerated module SHALL equal the multiset on the original module

### Requirement: Reusable round-trip helpers
The integration test suite SHALL expose a shared `tests/common/mod.rs` module providing the following public helpers, so a new round-trip case for a new fixture can be added by writing one `#[test]` of fewer than 10 lines.

#### Scenario: A new fixture can be added with one helper call per assertion
- **GIVEN** `tests/common/mod.rs` exposes `pub fn build_passthrough_schematic(module: &ModuleDef, language: Language) -> Schematic`
- **AND** `pub fn assert_sv_parses(text: &str) -> Vec<ModuleDef>` (or VHDL equivalent) that writes the text to a temp file and re-parses it, panicking on parse error
- **AND** `pub fn assert_shape_eq(expected: &[PortDef], actual: &[PortDef])` that panics if the sets differ
- **WHEN** a new fixture file is added under `tests/fixtures/`
- **THEN** a new `#[test]` consists of: `parse_file → build_passthrough_schematic → generate_<lang> → assert_<lang>_parses → assert_shape_eq`, with no other glue code
