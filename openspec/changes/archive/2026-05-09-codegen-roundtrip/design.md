## Context

`hdl_compose::parse_file` returns `Vec<ModuleDef>` for `.v`/`.sv`/`.vhd`/`.vhdl`. `hdl_compose::codegen::sv::generate_sv` and `hdl_compose::codegen::vhdl::generate_vhdl` consume a `Schematic` + library + diagnostics and return a `String`. Today's `tests/integration.rs` exercises the *forward* direction only — it parses a fixture, builds a hand-crafted schematic, calls codegen, and `contains`-matches strings in the output. There is no path that takes the generator's output and asserts the parser accepts it.

The sibling scratch tree (`~/git/hdl_tooling/hdl_block_editor`, superseded architecture) already explored this pattern and surfaced two real codegen bugs that the string-match tests missed. We are transferring the *pattern*, not the code, to this canonical project.

## Goals / Non-Goals

**Goals:**

- Detect malformed codegen output the moment it lands, not in a synthesizer six months later.
- Pin the existing codegen contract: same parsed module → same regenerated port shape.
- Make adding a new round-trip case for a new fixture a 5-line addition.

**Non-Goals:**

- Round-trip preservation of internal connections, aliases, manual bundles, or instance generic overrides. The passthrough wrapper has no internal connections — every port is wired straight through to a sibling top port — so these surfaces are not exercised here. They have their own existing unit tests in `src/codegen/`.
- Round-trip preservation of generic / parameter defaults. The `Schematic` model has `top_generics: Vec<GenericDef>`, but the existing codegen only uses it when populated — there is no helper that lifts a parsed module's generics into the wrapper. Building that lift would be feature work, not a test.
- Round-tripping behavioral / RTL bodies. The parser only extracts interfaces; the generator only emits structural wrappers. There is no body to round-trip.
- Verifying *behavior* (simulation equivalence). This is a syntactic / structural contract, not a semantic one.

## Decisions

### 1. Passthrough schematic shape

**Decision:** For each parsed `ModuleDef`, build a wrapper that:

- Names the wrapper top `<module_name>_passthrough`.
- Promotes every port of the parsed module to a top-level port with the same `name`, `direction`, and `port_type` (so vector widths survive verbatim — including unresolved `WIDTH-1 downto 0` expressions).
- Adds one instance of the parsed module under the name `dut`.
- Wires every instance port to its same-named top port via `NetRef::TopPort(port.name)`.

**Why not also propagate generics?** See goals. The model field exists but is not auto-populated by anything in the public API — adding that wiring is a separate change.

**Rationale for shape:** This is the minimal schematic that exercises every entity-port emission path, every component-decl emission path, and every port-map line. It does *not* exercise internal-net naming, slice rendering, alias rendering, or multi-instance dependency ordering — all of which are covered by existing unit tests in `src/codegen/sv.rs` and `src/codegen/vhdl.rs`.

### 2. Output validation: re-parse via `parse_file` against a temp file

**Decision:** Write the generated text to a `tempfile::NamedTempFile` with the appropriate extension (`.sv` for SV, `.vhd` for VHDL) and call `hdl_compose::parse_file` on it. Assert success, then locate the regenerated top module in the returned `Vec<ModuleDef>` by name.

**Why not call the parsers directly on a string?** `parse_file` is the public API. Going through it exercises the same code path the GUI / CLI use, including the extension dispatch. `tempfile` is already a dev-dep, so no new crates.

### 3. Shape comparison: order-insensitive on port name + direction + port_type

**Decision:** `assert_shape_eq` compares two `[PortDef]` slices by collecting them into a `BTreeMap<name, PortDef>` on each side and asserting the maps are equal under `PartialEq`. `PortDef`, `Direction`, and `PortType` all derive `PartialEq` already.

**Why ignore order?** Codegen sorts top-level ports alphabetically, while the original fixture is in source-order. Order isn't the contract; the *set* of ports is. (Slice / generic ordering inside a `port_type` is part of `PortType`'s `PartialEq` and stays meaningful.)

**Bundle field:** `PortDef::bundle` is allowed to differ. The parser may auto-detect bundles from naming heuristics; the generated output does not preserve them. Bundles aren't part of the syntactic round-trip contract.

### 4. Helpers live in `tests/common/mod.rs`, included via `mod common;`

**Decision:** Cargo's per-test-binary pattern. Each `tests/*_roundtrip.rs` file declares `mod common;` and uses `common::*`. The helpers compile once per test binary (a known Cargo trade-off) but the alternative — a `tests/common.rs` non-module file — produces an "unused crate-level item" lint that is louder than the duplicate compile.

### 5. Bug-fix protocol

If a round-trip assertion fails on a malformed-output bug:

1. Add a unit test in the relevant `src/codegen/<lang>.rs` `mod tests` that captures the bug minimally (smaller than the full fixture round-trip).
2. Fix the codegen.
3. Commit unit-test + fix together with `fix(codegen): <one-line description>`.
4. The round-trip test then passes "for free."

This keeps the round-trip tests as integration-level regressions and the unit tests as the developer-facing reproductions.

## Risks / Trade-offs

- **Re-parsing has measurable cost** — each round-trip is two parser invocations + one codegen + temp-file I/O. Acceptable: we have ~5 fixture cases and `cargo test` runs in well under a second on this project. If it ever exceeds a few hundred ms, the temp-file step can be replaced with the parser-on-string entry point (none currently exists publicly; would need a small `lib.rs` addition).
- **Hidden coverage gaps** — passthrough wrappers don't exercise slice / alias / multi-instance paths. Mitigation: existing unit tests in `src/codegen/` already cover those; this layer adds the *whole-pipeline* signal that string-matching missed.
- **Parser strictness drift** — if `vhdl_lang` or `sv-parser` ever loosens (accepting input that today rejects), a real codegen bug could slip through unnoticed. Mitigation: not in scope. We pin the parser version in `Cargo.toml`; intentional version bumps will exercise these tests.
