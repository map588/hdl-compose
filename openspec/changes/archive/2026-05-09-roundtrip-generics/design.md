## Context

`codegen-roundtrip` (archived 2026-05-09) introduced parser → codegen → parser tests for both backends, intentionally dropping generic survival because the test helper `build_passthrough_schematic` did not populate `Schematic::top_generics`. Both backends' codegen DOES emit `top_generics` (see `emit_module_header` in `src/codegen/sv.rs:88-112` and `emit_entity` in `src/codegen/vhdl.rs:117-141`), and both parsers DO populate `ModuleDef::generics` (see `extract_parameters` in `src/verilog.rs:124` and `extract_generics` in `src/vhdl.rs:136`). The gap is purely on the test-helper side.

## Goals / Non-Goals

**Goals:**

- Pin the contract that a parsed module's generic *names* and *default values* survive a round-trip through the codegen.
- Keep the test helper trivial — the lift is a one-line copy.
- Reuse the existing fixtures (`counter.v`, `counter.vhd`, `fifo_sync.vhd`) which already have generics. No new fixtures.

**Non-Goals:**

- Asserting `type_name` survives. Parser-side normalisation is inconsistent (VHDL `integer` is captured verbatim from source casing; SV `parameter int` vs bare `parameter` differ in `type_name` content). Forcing a string match would either fail spuriously or require synthetic re-normalisation in the test, which is parser-spelling territory, not codegen-contract territory.
- Round-tripping instance-level `generic_map` overrides. The passthrough wrapper has no overrides — the `dut` instance uses the child's defaults. Instance generic maps are already covered by unit tests in `src/codegen/vhdl.rs::tests::top_port_resolves_generic_override` and friends.
- Adding a new public method on `Schematic`. The struct's `top_generics` field is already `pub` and the existing test helper already mutates `top_ports` directly via field access (`s.top_ports = module.ports.clone()`). Mirroring that pattern for `top_generics` is the smallest possible change.

## Decisions

### 1. Lift in `build_passthrough_schematic`, not in `Schematic::new`

**Decision:** Add `s.top_generics = module.generics.clone();` to `build_passthrough_schematic` next to the existing `s.top_ports = module.ports.clone();` line.

**Why not push generics in `Schematic::new`?** `Schematic::new` doesn't take a `ModuleDef` — it builds an empty schematic. Wiring generics through there would change the constructor signature (a public API) for one test helper's benefit. The passthrough lift is exactly where the test pretends "this wrapper is a transparent shim around the source entity", so it's the right place to copy *all* of the source's interface (ports + generics).

### 2. Compare names + defaults only, order-insensitive

**Decision:** `assert_shape_eq` (or a new sibling) collects generics from each side into a `BTreeMap<&str, &GenericDef>` keyed on `name`, asserts the key sets match, and per-key asserts `default_value` matches.

**Why order-insensitive?** Codegen does not (currently) sort generics — it emits them in `top_generics` insertion order. Both parsers emit in source order. The lift preserves source order. So in practice, order *would* match on the round-trip — but asserting on order would couple the test to that incidental behaviour. Order-insensitive is what the existing port comparison does, so keep the contract uniform.

**Why not compare `type_name`?** Already covered above: parser-spelling drift would produce false negatives. If the project later normalises type names through a shared mapper, this can be tightened in a separate change.

### 3. Field-level write rather than `add_top_generic` method

**Decision:** Use direct `s.top_generics = ...` assignment in the helper. Do not add an `add_top_generic` method.

**Rationale:** The codebase consistently exposes `top_ports` and `top_generics` as `pub Vec<...>` fields and never gates them behind a setter. Adding a one-call helper would inflate the public API for no enforcement value (there's no per-name uniqueness to check that direct push could violate, since a parsed `ModuleDef` already has unique generic names — the parser would have rejected duplicates). The brief allows for adding such a method only if `add_top_port` exists; in this codebase it does not.

### 4. Spec delta: MODIFY the existing scope-out scenario, ADD the new contract

**Decision:** The current spec at `openspec/specs/codegen-roundtrip/spec.md` does not have an explicit "generics not asserted" scenario — the carve-out lives in the archived change's design.md. So the delta is purely **ADDED**: a new requirement "Generic round-trip preserves names and defaults" with two scenarios (one SV, one VHDL). No `## MODIFIED Requirements` block is needed because no prior scenario contradicts the new contract.

**Why the brief mentions MODIFIED:** The brief said *"replacing the 'intentionally not asserted' scenario"* — that scenario was promised to land in the spec but didn't (the archived change shipped without inserting an explicit carve-out scenario, only the prose note in `design.md` and `proposal.md`). Treat the brief's instruction as descriptive, not literal: the *intent* is to replace the carve-out, the *mechanism* is to add the positive requirement. If the scope-out had been promoted into the spec as a scenario, this would be `## MODIFIED Requirements`; since it wasn't, `## ADDED Requirements` is correct.

## Risks / Trade-offs

- **Verilog `parameter` default-value spelling drift.** The SV parser captures `WIDTH = 8` as `default_value = Some("8")`. The codegen emits `parameter WIDTH = 8`. The re-parser reads `"8"` back. This works for integer literals. If a fixture ever uses a string default like `parameter S = "hello"`, the quotes round-trip is parser-dependent — out of scope today; revisit if a fixture surfaces it.
- **VHDL default-value formatting.** `vhdl_lang` returns `format!("{}", expr.item)` which canonicalises spacing (`8` not `   8`). Codegen emits `WIDTH : integer := 8`. Round-trips on all current fixtures. A pathological default like `(others => '0')` may stringify differently across two passes; not a current fixture concern.
- **Hidden coverage gap if codegen ever stops emitting top_generics.** That regression is exactly what this change protects against, so the risk is *catching* it, not running into it.
