## Why

The `codegen-roundtrip` change pinned the contract that codegen output for a *passthrough* schematic re-parses cleanly. It explicitly skipped `tests/fixtures/fixture_project.vhd` because that file is generator output, not a source-shape entity. It did not however close the matching gap on the *input* side: `tests/fixtures/fixture_project.hdlc` is a saved schematic — exactly what the user produces from the GUI — and nothing in the existing test suite drives the load → codegen → re-parse path end-to-end on a real project file.

The result is that a regression in the `.hdlc` deserializer (a renamed field, a dropped `#[serde(default)]`, a new variant the loader doesn't handle, a `cleanup_stale_refs` pass that drops live references) can ship with `cargo test` green, and only surface when an end user opens an existing project and finds their connections gone or, worse, sees a build pass produce malformed HDL. We need a single test that exercises the full user-facing pipeline on a real saved project.

## What Changes

- **Add an integration test** `tests/project_roundtrip.rs::roundtrip_fixture_project` that: loads `tests/fixtures/fixture_project.hdlc` via `hdl_compose::project::load_project`, resolves the saved library paths to a `ModuleDef` library, validates the loaded schematic, generates HDL via the appropriate backend (`codegen::vhdl::generate_vhdl` or `codegen::sv::generate_sv` per `schematic.language`), and re-parses the generated text via `hdl_compose::parse_file` to assert no parse errors.
- **Reuse existing test helpers.** `tests/common/mod.rs::assert_vhdl_parses` and `assert_sv_parses` already do the temp-file write + parse dance with a useful failure dump. The new test imports them via `mod common;` — no helper additions.
- **Portability shim in the test, not the fixture.** The fixture stores absolute `library_paths` from the original author's machine. Per the per-task instruction "do not rewrite fixtures," the test rewrites each `library_path` in memory after `load_project` returns to point at the matching file in `tests/fixtures/`. The fixture file is left untouched.
- **No new external crates.**
- **Fix any bugs surfaced.** If the load path or codegen against the loaded schematic produces invalid HDL, fix in scope and document under `design.md`. (None surfaced on the current `master`.)

## Capabilities

### New Capabilities

- `project-load-roundtrip`: the test-only contract that loading a saved `.hdlc` project, resolving its library, and running codegen produces HDL that re-parses without error.

### Modified Capabilities

- None. No production behavior is changing — this layer pins existing contracts.

## Impact

- **Tests**: new file `tests/project_roundtrip.rs` plus a copy of `tests/fixtures/fixture_project.hdlc` (already present in the parent checkout, copied into this worktree's tree). No changes to `tests/common/mod.rs`. Existing tests untouched.
- **Source**: no changes on the current `master`.
- **No new dependencies.**
- **Out of scope**: shape-comparing the regenerated entity's ports against a "source" `ModuleDef` (no such canonical shape exists for a multi-instance project — the project IS the source). Round-tripping the saved-then-loaded schematic byte-for-byte (the existing `project::round_trip` unit test in `src/project.rs` covers in-process save/load equality). Generator-output → parser → save-back symmetry. Behavioral / simulation equivalence.
