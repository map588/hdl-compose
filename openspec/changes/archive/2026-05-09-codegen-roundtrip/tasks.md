## 1. Test scaffolding

- [x] 1.1 Add `tests/common/mod.rs` with: `assert_sv_parses(text)`, `assert_vhdl_parses(text)`, `build_passthrough_schematic(module, language)`, `assert_shape_eq(expected, actual, expected_generics)`.
- [x] 1.2 `assert_*_parses` writes the text to a `tempfile::NamedTempFile` with `.sv` / `.vhd` extension and calls `hdl_compose::parse_file`. Returns the parsed `Vec<ModuleDef>` on success; panics with the parse error and a numbered dump of the offending text on failure.
- [x] 1.3 `build_passthrough_schematic` produces a `Schematic` named `<module>_passthrough` with one instance `dut`, every port of the source promoted to a top port, every instance port mapped to its same-named top port via `NetRef::TopPort`.
- [x] 1.4 `assert_shape_eq` collects each side into a `BTreeMap<&str, &PortDef>` and compares (direction, port_type) per port — order-insensitive, ignores the auto-detected `bundle` field. Both port_types are first normalised through `codegen::resolve_port_type` against the source module's generics with no instance overrides, so the codegen's eager `WIDTH-1 → 7` resolution is not a false mismatch against the parser's symbolic form.

## 2. SV round-trip test

- [x] 2.1 New file `tests/sv_roundtrip.rs` with `mod common;`.
- [x] 2.2 `#[test] fn sv_roundtrip_counter()` for `tests/fixtures/counter.v`.
- [x] 2.3 No parse failures surfaced on the SV fixture — no codegen bugs to fix.

## 3. VHDL round-trip test

- [x] 3.1 New file `tests/vhdl_roundtrip.rs` with `mod common;`.
- [x] 3.2 `#[test] fn vhdl_roundtrip_counter()` for `tests/fixtures/counter.vhd`.
- [x] 3.3 `#[test] fn vhdl_roundtrip_fifo_sync()` for `tests/fixtures/fifo_sync.vhd`.
- [~] 3.4 Skipped `tests/fixtures/fixture_project.vhd` — that file is generator OUTPUT (a regenerated top with component declarations and an architecture body), not a source-shape entity. Putting it through the round-trip would round-trip generated code, which is not the contract; the contract is that generator output of a source-shape entity re-parses cleanly. Not a gap.
- [x] 3.5 No parse failures surfaced on the VHDL fixtures — no codegen bugs to fix.

## 4. Validation gates

- [x] 4.1 `cargo test` is green: 71 lib + 8 integration + 1 sv + 2 vhdl + 0 doc = 82 tests passing.
- [x] 4.2 `cargo clippy --tests -- -D warnings`: clean for the new files (`tests/common/`, `tests/sv_roundtrip.rs`, `tests/vhdl_roundtrip.rs`). 9 pre-existing warnings remain in `src/` (verilog.rs, schematic.rs, codegen/mod.rs, gui/bridge.rs, gui/mod.rs); per the per-task brief they are out of scope for chasing.
- [x] 4.3 `cargo fmt -- --check` is clean for the new files. Pre-existing fmt diffs in `src/` are out of scope.
- [x] 4.4 `openspec validate codegen-roundtrip` is clean.

## 5. Archive

- [ ] 5.1 Left active. The project's archive convention for non-trivial changes (every existing archived change in `openspec/changes/archive/`) is that they were archived at "feature complete + merged into the active source tree" boundaries. This change is feature-complete on the test side but is on a feature branch that has not yet been reviewed / merged into `master`, so per existing convention it stays active until the maintainer closes the loop. Operator can run `openspec archive codegen-roundtrip -y` after merge.
