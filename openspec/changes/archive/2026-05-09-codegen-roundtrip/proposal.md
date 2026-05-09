## Why

The existing integration tests parse fixtures, run codegen, and string-match the result. They never feed the *output* back through the parser, so a regression that emits malformed VHDL or SystemVerilog (mismatched parens, missing semicolons, illegal identifiers, `assign` to a non-net, etc.) will pass `cargo test` and only surface when a downstream tool — a synthesizer, a simulator, or even our own re-import flow — chokes on it. That is the worst place to discover the bug.

We need a guard at the codegen boundary: the generator's output, given any valid schematic, must always be syntactically-valid HDL of the same flavor. Round-trip parsing (parser → codegen → parser) is the smallest tool that proves this.

## What Changes

- **Add round-trip tests for both backends.** For each fixture module, parse it, build a passthrough schematic (every parsed port becomes a top-level port on the generated wrapper, every instance port wired through to that top port), generate the wrapper, parse the generated text, and shape-compare the regenerated top module's ports against the original.
- **Reusable test helpers.** Extract a `tests/common/mod.rs` module with `assert_sv_parses`, `assert_vhdl_parses`, `build_passthrough_schematic`, and `assert_shape_eq` so future round-trip cases can be added in a couple of lines.
- **Fix any codegen bugs the round-trip surfaces.** Each bug gets its own commit with a unit-test reproduction and the minimal fix. No unrelated cleanup.

## Capabilities

### New Capabilities

- `codegen-roundtrip`: the test-only contract that the codegen output is a valid input to the same-language parser, and that the regenerated top module preserves the original module's port shape.

### Modified Capabilities

- None. The codegen behavior itself is not changing — we are pinning the existing contract with a sharper test.

## Impact

- **Tests**: new files `tests/common/mod.rs`, `tests/sv_roundtrip.rs`, `tests/vhdl_roundtrip.rs`. Existing `tests/integration.rs` is untouched.
- **Source**: only modified if a round-trip surfaces a real codegen bug. Each fix is a separate commit.
- **No new dependencies.** All helpers use what's already in the dev deps (`tempfile`) or the lib API (`hdl_compose::parse_file`, `hdl_compose::codegen`).
- **Out of scope**: round-tripping parameter / generic *defaults* (the `top_generics` model exists but the generator does not currently propagate generic defaults from a parsed source module into the top-level wrapper's `top_generics` — that is feature work, tracked separately). Round-tripping internal connections / aliases / slices (the passthrough wrapper has no internal nets). Round-tripping the GUI / `.hdlc` project format.
