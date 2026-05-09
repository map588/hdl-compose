## Why

The just-archived `codegen-roundtrip` change explicitly scoped out generic-default round-trip because no public helper lifted a parsed module's generics into the wrapper schematic's `top_generics` field. The model field exists, both backends already emit `top_generics` correctly, and the parsers already populate `ModuleDef::generics` with `name` / `type_name` / `default_value`. The only missing piece is the test-side wiring that says "when you build a passthrough wrapper for a parameterised module, also lift the module's generics onto the wrapper top". Without it, the round-trip happily passes for `counter.vhd` and `fifo_sync.vhd` even though the generated wrappers silently drop their `generic` clauses — a regression in the codegen's generic emission would not be caught.

## What Changes

- **Lift generics in `build_passthrough_schematic`**: copy `module.generics` into the resulting `Schematic`'s `top_generics` field. No change to instance generic maps (the passthrough wrapper does not override the child's defaults).
- **Extend `assert_shape_eq`**: also assert that every generic in `original.generics` appears in the regenerated module's `generics` with matching `name` and `default_value`, order-insensitive. Generic `type_name` is **not** compared — neither parser normalises type spelling (e.g. VHDL `integer` vs `INTEGER`, SV `parameter` vs `parameter int`), and forcing a string-equal contract on it would be a parser-spelling test, not a codegen contract test. If the parser ever adds a normalised `type_name` field, this can be tightened.
- **Replace the existing scenario** in `codegen-roundtrip` spec that says "intentionally not asserted" / "out of scope" for generics with two new scenarios that pin the lift behaviour.
- **No GUI / CLI changes.** No new public API beyond what the model already exposes (`top_generics: Vec<GenericDef>` is already a `pub` field).

## Capabilities

### Modified Capabilities

- `codegen-roundtrip`: the round-trip contract now also covers generic survival.

## Impact

- **Tests**: `tests/common/mod.rs` gains generic lift in `build_passthrough_schematic` and a generic-comparison loop in `assert_shape_eq`. The existing `vhdl_roundtrip_counter` / `vhdl_roundtrip_fifo_sync` tests then exercise the new path without source changes (their fixtures already declare `WIDTH` / `DEPTH` generics). `sv_roundtrip_counter` exercises it for Verilog `parameter WIDTH = 8`.
- **Source**: only modified if a round-trip surfaces a real codegen bug. None expected — the codegen for `top_generics` already exists and is unit-tested. If a bug *is* found, it gets a unit test in `src/codegen/{sv,vhdl}.rs` plus a fix in a separate commit, per the protocol the parent change established.
- **No new dependencies.**
- **Out of scope**: round-tripping `generic_map` *overrides* on instances (passthrough wrapper has no overrides — child uses its defaults). Round-tripping per-instance generics in the wrapper. Comparing generic `type_name` strings (parser-normalisation gap, separate concern).
