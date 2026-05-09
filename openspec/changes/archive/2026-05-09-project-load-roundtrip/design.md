## Context

`hdl_compose::project::load_project(&Path) -> Result<(Schematic, Vec<String>), ProjectError>` reads a `.hdlc` JSON file and returns the deserialized `Schematic` plus a list of warnings (missing library files, cleared stale references). `Schematic::resolve_modules() -> (Vec<ModuleDef>, Vec<(PathBuf, ParseError)>)` re-parses every `library_path` to rebuild the module library. The `Schematic` carries its target language as `language: Language { Vhdl | SystemVerilog }`. `codegen::vhdl::generate_vhdl` and `codegen::sv::generate_sv` consume `(&Schematic, &[ModuleDef], &[Diagnostic])` and return `Result<String, CodegenError>`.

The previous `codegen-roundtrip` change tested the codegen → parser arc for synthetically-built passthrough schematics. It deliberately did not exercise the project loader, because the canonical fixture (`fixture_project.hdlc`) was not in scope for "round-trip generator output of a source-shape entity." This change closes that gap by round-tripping the *saved schematic* end-to-end.

## Goals / Non-Goals

**Goals:**

- Fail `cargo test` the moment a `.hdlc` deserialization regression, a stale-ref cleanup over-step, or a codegen bug specific to multi-instance schematics with internal nets ships.
- Mirror the actual end-user path: open a saved project, regenerate HDL.

**Non-Goals:**

- Shape-comparing the regenerated entity's port list against any canonical "expected" shape. The project IS the source — there is no source `ModuleDef` to diff against.
- Re-saving the schematic and asserting JSON byte-equality. `src/project.rs::tests::round_trip` already pins in-process save → load equality at the model layer.
- Round-tripping the generated HDL back into a schematic. There is no parse-HDL-into-schematic path in the codebase.
- Adding new test helpers. `assert_vhdl_parses` / `assert_sv_parses` cover both languages.

## Decisions

### 1. Single test, language-dispatched at runtime

**Decision:** One `#[test] fn roundtrip_fixture_project()` that branches on `schematic.language` to pick the codegen backend and the matching `assert_*_parses` helper. Not two language-specific tests.

**Why:** The fixture is one file with one declared language. Testing both languages from one fixture is impossible; testing the *test* against both languages is feature work the per-task brief calls out as out of scope ("If the project file is for SystemVerilog instead, swap to `assert_sv_parses`. Detect from the file content rather than guessing"). A runtime `match schematic.language { Vhdl => ..., SystemVerilog => ... }` does that detection without hard-coding the language in the test name.

The fixture currently committed is `Language::Vhdl`. If a future SV fixture is added, that's a new test, not a fork of this one.

### 2. Library-path rewrite in the test, not the fixture

**Decision:** After `load_project` returns, walk `schematic.library_paths` and replace each entry with `tests/fixtures/<file_name>`. Do not edit the `.hdlc` file on disk.

**Why:** The fixture stores absolute paths from the author's machine (`/Users/matthewprock/...`). Without rewriting, the test passes only on that one machine and silently degrades on every other (the loader warns, doesn't fail; `resolve_modules` returns empty library; codegen reports module-not-in-library validation errors and refuses). Rewriting in-memory keeps the fixture untouched per the per-task instruction "If `fixture_project.hdlc` is malformed or references non-existent fixtures, pause and report rather than rewriting the fixture."

The rewrite uses `Path::file_name()` so the fixture's `library_paths` schema (absolute, relative, with directory components) doesn't matter to the test. Only the basename has to match a file in `tests/fixtures/`. That happens to be true today (`counter.vhd`, `fifo_sync.vhd`).

**Alternative considered:** Patching the `.hdlc` to use relative paths. Rejected: explicit instruction not to rewrite fixtures, plus a relative path would need to be resolved against either the `.hdlc`'s parent directory or the `cargo test` cwd, neither of which the loader does today. Adding that relative-path support would be a real source change, not a test addition.

### 3. Validation gate before codegen

**Decision:** Run `schematic.validate(&library)` and assert no errors before invoking codegen. Even though `generate_vhdl` / `generate_sv` will themselves refuse on validation errors via `check_errors`, asserting in the test gives a clearer failure message ("schematic has validation errors: [...]") than the generic `CodegenError::ValidationErrors(...)` panic from `.expect(...)`.

### 4. Light cross-check beyond "parses cleanly"

**Decision:** After re-parsing, assert that the regenerated top entity is present in the parsed `Vec<ModuleDef>` under `schematic.top_name`, and that its port count matches `schematic.top_ports.len()`. Stop there.

**Why not deeper shape comparison?** The per-task brief: "this test does NOT need to shape-compare against a source ModuleDef — there is no single source module, the project IS the source. Stop at 'generated HDL parses cleanly'. If you want stronger coverage, additionally compare module names emitted vs schematic instance names — but don't go further."

A first iteration tried "every distinct `module_ref` in the schematic appears as a regenerated module name." That fails because `vhdl_lang::parse_file` returns top-level entities only — `component` declarations inside an architecture body are not surfaced as separate `ModuleDef`s. So that check would always fail on a multi-instance VHDL project. Replaced with "regenerated top is present and exposes the expected port count."

## Risks / Trade-offs

- **The library-path rewrite assumes basename uniqueness in `tests/fixtures/`.** If a future fixture references two files with the same basename in different directories, the rewrite collapses them. Mitigation: not a real risk on the current fixture set, and the assertion `assert!(path.exists(), ...)` after the rewrite will catch any "rewritten path doesn't exist" case loudly.
- **The test is sensitive to validation diagnostics.** The current schematic has open ports (`counter_0.count → null`) which validate cleanly as warnings, not errors. If `cleanup_stale_refs` ever escalates an open port to an error, the test will fail — but that's a real behavioral change worth reviewing, not a flake.
- **No coverage of the "cleared N stale refs" warning path.** The warnings from `load_project` are captured but only printed in the `assert!` failure message. A regression that quietly drops live references would still produce parseable HDL with fewer instances; the port-count check on the top is the only safety net (and it would not trigger on instance loss, only top-port loss). Acceptable for this layer; deeper invariants are unit-test territory.
- **`vhdl_lang` strictness drift.** Same risk noted in the prior `codegen-roundtrip` change — a parser version bump that quietly relaxes acceptance could let a real codegen bug slip through. Pinned via `Cargo.lock`; not in scope.

## Bugs Surfaced

None. The load → codegen → re-parse pipeline runs cleanly on the committed fixture as of `master @ 7465a4e`.

## Notes

- **`*.hdlc` is in `.gitignore`.** The repo's `.gitignore` excludes `*.hdlc` as a rule (saved projects are user data, not source). The fixture is committed with `git add -f` to bypass that rule for the test asset specifically. If a future change reorganises `tests/fixtures/`, that `-f` step needs to repeat — there is no `!tests/fixtures/*.hdlc` un-ignore in `.gitignore` because adding one would risk re-ignoring everything in a sub-tree that another rule un-ignores. Single fixture, single `-f` add, simpler.
