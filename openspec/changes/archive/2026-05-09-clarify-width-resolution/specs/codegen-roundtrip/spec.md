## ADDED Requirements

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
