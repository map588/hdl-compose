## 1. Fixture availability

- [x] 1.1 Confirm `tests/fixtures/fixture_project.hdlc` is present in the worktree's `tests/fixtures/` directory (copied in from the parent checkout where it was untracked but present on disk).
- [x] 1.2 Read its contents to determine `language`, `top_name`, instance count, and which library files it references. Result: `language: "Vhdl"`, top `fixture_project`, two instances (`counter_0`, `fifo_sync_0`) referencing `counter` and `fifo_sync`. Library paths are absolute, point to `counter.vhd` and `fifo_sync.vhd` in the same directory.

## 2. Test scaffolding

- [x] 2.1 Create `tests/project_roundtrip.rs` with `mod common;` so the file shares `assert_vhdl_parses` / `assert_sv_parses` with the existing round-trip suites.
- [x] 2.2 Add `#[test] fn roundtrip_fixture_project()` that:
  - calls `hdl_compose::project::load_project(Path::new("tests/fixtures/fixture_project.hdlc"))`
  - rewrites every entry of `schematic.library_paths` to `tests/fixtures/<file_name>` so the test is portable across machines
  - asserts every rewritten path exists on disk
  - calls `schematic.resolve_modules()` and asserts no parse errors
  - asserts every instance's `module_ref` is in the resolved library
  - calls `schematic.validate(&library)` and asserts no errors
  - dispatches on `schematic.language` to call `codegen::vhdl::generate_vhdl` or `codegen::sv::generate_sv`
  - dispatches on `schematic.language` to call `common::assert_vhdl_parses` or `common::assert_sv_parses` on the generated text
  - locates the top module by `schematic.top_name` in the re-parsed `Vec<ModuleDef>` and asserts `ports.len()` matches `schematic.top_ports.len()`

## 3. Bugs surfaced

- [x] 3.1 None on `master @ 7465a4e`. The load → resolve → validate → codegen → re-parse pipeline runs end-to-end with no errors. Documented in `design.md` under "Bugs Surfaced."

## 4. Validation gates

- [x] 4.1 `cargo test` passes — full suite runs green, new test included.
- [x] 4.2 `cargo clippy --tests -- -D warnings` clean over the touched files (`tests/project_roundtrip.rs`).
- [x] 4.3 `cargo fmt -- --check` clean over the touched files.
- [x] 4.4 `openspec validate project-load-roundtrip` passes.

## 5. Archive

- [ ] 5.1 Left active per the per-task instruction "Do NOT archive — leave for operator merge." Operator can run `openspec archive project-load-roundtrip -y` after merge into `master`.
