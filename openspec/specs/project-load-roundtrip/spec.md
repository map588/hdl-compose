# project-load-roundtrip Specification

## Purpose
TBD - created by archiving change project-load-roundtrip. Update Purpose after archive.
## Requirements
### Requirement: Saved `.hdlc` projects load into a `Schematic`
The project loader SHALL accept a saved `.hdlc` file produced by the project save path and return a `Schematic` whose `top_name`, `language`, `top_ports`, `instances`, `aliases`, and `library_paths` match the saved file's content. Missing library files produce a warning, not an error.

#### Scenario: `fixture_project.hdlc` loads
- **WHEN** `hdl_compose::project::load_project("tests/fixtures/fixture_project.hdlc")` is invoked
- **THEN** the call SHALL return `Ok((schematic, _warnings))` and `schematic.top_name` SHALL equal `"fixture_project"`
- **AND** `schematic.instances` SHALL contain at least one entry whose `module_ref` is one of `"counter"` or `"fifo_sync"`

### Requirement: Codegen accepts a loaded schematic
For every `Schematic` produced by the project loader whose `library_paths` resolve to a parseable module library, the codegen backend matching `schematic.language` SHALL produce HDL text without returning `CodegenError`, provided `schematic.validate(&library)` reports no errors.

#### Scenario: VHDL codegen runs on the loaded fixture project
- **WHEN** `fixture_project.hdlc` is loaded and its `library_paths` are resolved against the local `tests/fixtures/` directory
- **AND** `schematic.validate(&library)` returns no errors
- **AND** `schematic.language == Language::Vhdl`
- **THEN** `codegen::vhdl::generate_vhdl(&schematic, &library, &diags)` SHALL return `Ok(text)`

### Requirement: Codegen output for a loaded project re-parses cleanly
The HDL text produced by codegen for a saved-and-loaded `Schematic` SHALL be accepted without error by the same-language parser via `hdl_compose::parse_file`.

#### Scenario: VHDL output for `fixture_project.hdlc` re-parses
- **WHEN** the VHDL text generated for the loaded `fixture_project.hdlc` is written to a temp file with extension `.vhd`
- **AND** `hdl_compose::parse_file` is invoked on that temp file
- **THEN** the call SHALL return `Ok(modules)` with no parse error
- **AND** `modules` SHALL contain a `ModuleDef` whose `name` equals the loaded schematic's `top_name`
- **AND** that top `ModuleDef`'s `ports.len()` SHALL equal the loaded schematic's `top_ports.len()`

### Requirement: Test runs without modifying the fixture file
The integration test SHALL NOT mutate `tests/fixtures/fixture_project.hdlc` on disk. Path-portability concerns (the fixture's saved `library_paths` are absolute paths from the author's machine) SHALL be resolved by rewriting the in-memory `schematic.library_paths` after `load_project` returns, not by editing the fixture.

#### Scenario: Library paths are rewritten in memory
- **GIVEN** the fixture stores `library_paths` as absolute filesystem paths
- **WHEN** the integration test loads the fixture
- **THEN** the test SHALL replace each entry in `schematic.library_paths` with `tests/fixtures/<file_name>` before calling `Schematic::resolve_modules`
- **AND** the on-disk fixture file SHALL be left unchanged

