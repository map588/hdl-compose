## 1. Test scaffolding

- [x] 1.1 In `tests/common/mod.rs`, update `build_passthrough_schematic(module, language)` to also copy `module.generics` into the resulting `Schematic`'s `top_generics` field. Mirror the existing `s.top_ports = module.ports.clone();` pattern.
- [x] 1.2 Add `assert_generics_eq(expected, actual)` sibling helper to `assert_shape_eq`. Asserts: (a) every generic name in `expected` appears in `actual`, order-insensitive; (b) per matching name, `default_value` matches. `type_name` is intentionally NOT compared — see design.md §2.
- [x] 1.3 Wire the new helper into `tests/sv_roundtrip.rs` and `tests/vhdl_roundtrip.rs` by calling `assert_generics_eq(&original.generics, &top.generics)` after the existing port-shape assertion.

## 2. SV round-trip exercises generics

- [x] 2.1 `sv_roundtrip_counter` (existing test) now also asserts `WIDTH = 8` survives the round-trip. No new test function needed — `counter.v` already declares `parameter WIDTH = 8`.
- [x] 2.2 The Verilog parser dropped trailing source whitespace into `default_value` and `type_name` (see commit `fix(parser): strip trailing trivia from verilog parameter type and default`). Caused malformed codegen output (`8 \n` baked into emitted text). Fixed by trimming both fields at the parser boundary; pinned by new unit test `parameter_default_and_type_strip_trailing_whitespace`. Committed separately from this change's test wiring per the parent-change protocol.

## 3. VHDL round-trip exercises generics

- [x] 3.1 `vhdl_roundtrip_counter` now also asserts `WIDTH := 8` survives.
- [x] 3.2 `vhdl_roundtrip_fifo_sync` now also asserts `DEPTH := 256` and `WIDTH := 8` survive.
- [x] 3.3 No VHDL codegen bugs surfaced — `vhdl_lang` returns expression strings already canonicalised, so trailing trivia did not leak.

## 4. Spec update

- [x] 4.1 Added new requirement `Generic round-trip preserves names and defaults` to the `codegen-roundtrip` capability spec, with one SV scenario (`counter.v`) and one VHDL scenario (`fifo_sync.vhd`).
- [x] 4.2 `openspec validate roundtrip-generics --strict` is clean.

## 5. Validation gates

- [x] 5.1 `cargo test` green: 72 lib + 8 integration + 1 sv_roundtrip + 2 vhdl_roundtrip + 0 doc = 83 tests passing.
- [x] 5.2 `cargo clippy --tests -- -D warnings`: clean for the touched files (`tests/common/mod.rs`, `tests/sv_roundtrip.rs`, `tests/vhdl_roundtrip.rs`, `src/verilog.rs`). Pre-existing warnings in unrelated files are out of scope per the parent-change task brief.
- [x] 5.3 `rustfmt --check` is clean for touched files.

## 6. Archive

- [ ] 6.1 Left active. Per the parent change's convention, archive happens after operator review/merge to master, not in the implementing commit.
