# codegen-roundtrip Specification

## Purpose
TBD - created by archiving change codegen-roundtrip. Update Purpose after archive.
## Requirements
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

### Requirement: Width-resolution policy is documented and uniform

The codegen pipeline SHALL document its width-resolution policy and the
round-trip test infrastructure SHALL apply that policy uniformly when
comparing original and regenerated port shapes.

The selected policy is **RESOLVE (eager / literal)**: at code-emission
time, `codegen::resolve_port_type` substitutes any `RangeExpr::Expr`
range bound that can be evaluated against the source module's
`GenericDef::default_value`s plus the per-instance `Instance::generic_map`
overrides, replacing it with `RangeExpr::Literal`. Bounds that cannot be
resolved (e.g. a generic with no default and no override) survive
verbatim and are emitted as-is, with the schematic validator surfacing
unresolved references separately.

The PRESERVE alternative — emit the symbolic form verbatim, never
substitute at codegen time — is rejected for this iteration. See
`design.md` for the file-level impact analysis behind the choice.

#### Scenario: codegen emits literal widths when the source default is known
- **WHEN** a passthrough schematic promotes a child port whose
  `port_type` carries `RangeExpr::Expr("WIDTH-1")` for a child generic
  `WIDTH` with `default_value = Some("8")`
- **AND** the instance has no `generic_map` override for `WIDTH`
- **AND** `generate_sv` or `generate_vhdl` is invoked
- **THEN** the emitted top-port type SHALL be the literal range form
  (e.g. `[7:0]` for SV, `(7 downto 0)` for VHDL)
- **AND** no `WIDTH-1` text SHALL appear in the emitted top-port range
  bounds

#### Scenario: codegen emits the per-instance override when present
- **WHEN** a schematic instance carries `generic_map["WIDTH"] = "16"`
- **AND** the source module's `WIDTH` default is `Some("8")`
- **AND** `generate_sv` or `generate_vhdl` is invoked on a wrapper
  whose top port type resolves through that instance's context
- **THEN** the emitted top-port type SHALL use the override value
  (e.g. `[15:0]` / `(15 downto 0)`)
- **AND** the `inst.generic_map` SHALL also appear in the emitted
  parameter / generic map block of that instance

#### Scenario: unresolvable expressions survive emission verbatim
- **WHEN** a port carries `RangeExpr::Expr("UNKNOWN_PARAM-1")` and
  neither the module's generics nor the instance's `generic_map`
  define `UNKNOWN_PARAM`
- **AND** `codegen::resolve_port_type` is invoked
- **THEN** the returned `PortType` SHALL retain the original
  `RangeExpr::Expr("UNKNOWN_PARAM-1")` bound unchanged
- **AND** validation (handled separately) SHALL surface the unresolved
  reference as a diagnostic

#### Scenario: round-trip shape comparison applies the same policy to both sides
- **WHEN** `assert_shape_eq` compares an original parsed `[PortDef]`
  against the regenerated `[PortDef]` for a passthrough wrapper
- **THEN** both sides SHALL be normalized through
  `codegen::resolve_port_type` against the same generic-substitution
  map (the original module's `GenericDef`s with no instance overrides)
  before equality comparison
- **AND** the doc comment on `assert_shape_eq` SHALL frame this
  normalization as "consistent with the documented RESOLVE policy",
  not as a workaround

### Requirement: Generic round-trip preserves names and defaults
For every fixture module that declares generics (Verilog `parameter`, VHDL `generic`), the regenerated top module produced by the passthrough round-trip MUST expose the same set of generics as the original parsed module — comparing generic name and `default_value`, ignoring source order. Generic `type_name` is NOT part of the contract because parser-side type-name spelling is not normalised across backends.

#### Scenario: SV generic shape survives a round-trip
- **WHEN** `parse_file("tests/fixtures/counter.v")` returns a `ModuleDef` for `counter` with `generics = [GenericDef { name: "WIDTH", default_value: Some("8"), .. }]`
- **AND** a passthrough schematic is built that promotes every `counter` port to a top-level port AND lifts every generic in `counter.generics` into `Schematic::top_generics`
- **AND** `codegen::sv::generate_sv` is invoked on that schematic
- **AND** the resulting text is re-parsed via `parse_file` against a temp file with extension `.sv`
- **THEN** the regenerated `counter_passthrough` module's `generics` SHALL contain a `GenericDef` with `name == "WIDTH"` and `default_value == Some("8")`

#### Scenario: VHDL generic shape survives a round-trip
- **WHEN** `parse_file("tests/fixtures/fifo_sync.vhd")` returns a `ModuleDef` for `fifo_sync` with `generics = [{name: "DEPTH", default_value: Some("256"), ..}, {name: "WIDTH", default_value: Some("8"), ..}]`
- **AND** a passthrough schematic is built that promotes every `fifo_sync` port to a top-level port AND lifts every generic in `fifo_sync.generics` into `Schematic::top_generics`
- **AND** `codegen::vhdl::generate_vhdl` is invoked on that schematic
- **AND** the resulting text is re-parsed via `parse_file` against a temp file with extension `.vhd`
- **THEN** the regenerated `fifo_sync_passthrough` module's `generics` SHALL contain `GenericDef`s with `(name, default_value)` pairs `("DEPTH", Some("256"))` and `("WIDTH", Some("8"))`, order-insensitive

