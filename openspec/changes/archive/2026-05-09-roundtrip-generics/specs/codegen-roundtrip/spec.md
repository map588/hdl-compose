## ADDED Requirements

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
