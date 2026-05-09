## Purpose

Defines the `.hdlc` project file format and the versioned load/save contract. The loader accepts v2 and v3; saves always emit the current version. v3 adds per-instance `manual_bundles`.
## Requirements
### Requirement: Save schematic to .hdlc JSON file
The system SHALL serialize a `Schematic` to a JSON file with `.hdlc` extension, including all instances, port maps, generic maps, aliases, top-level ports/generics, and library paths.

#### Scenario: Save and verify structure
- **WHEN** a schematic with 2 instances and 3 library paths is saved to `design.hdlc`
- **THEN** the file SHALL contain valid JSON with `"version": 2`, all instances with their port/generic maps, and all library paths

#### Scenario: ModuleDef data is NOT stored
- **WHEN** a schematic is saved
- **THEN** the file SHALL NOT contain parsed port definitions, type information, or source hashes from ModuleDef — only module reference names

### Requirement: Load schematic from .hdlc JSON file
The system SHALL deserialize a `.hdlc` file back into a `Schematic`, then re-parse all library paths to rebuild the module library.

#### Scenario: Round-trip fidelity
- **WHEN** a schematic is saved to a file and then loaded back
- **THEN** the loaded schematic SHALL be identical to the original in all fields (instances, port maps, generic maps, aliases, top-level ports, library paths)

#### Scenario: Library re-parse on load
- **WHEN** a `.hdlc` file is loaded
- **THEN** the system SHALL parse all files listed in `library_paths` to rebuild the `ModuleDef` library

#### Scenario: Missing library file on load
- **WHEN** a library path in the `.hdlc` file points to a nonexistent file
- **THEN** the system SHALL report a warning but still load the schematic — the affected instances will fail validation

### Requirement: Version field for forward compatibility
`.hdlc` project files SHALL carry a top-level `version` field. The loader SHALL accept v2 and v3; saves SHALL emit v3. v3 adds the optional-at-load `manual_bundles` field on each `Instance`; v2 files without it SHALL deserialize as an empty map. Unknown versions outside the supported range SHALL be rejected with an error naming the version.

#### Scenario: Save writes v3
- **WHEN** a project is saved
- **THEN** the resulting `.hdlc` contains `"version": 3`

#### Scenario: Load v2 succeeds with empty manual_bundles
- **WHEN** a v2 `.hdlc` without `manual_bundles` is loaded
- **THEN** every instance's `manual_bundles` is an empty map

#### Scenario: Load v3 round-trips manual_bundles
- **WHEN** a v3 `.hdlc` with `spi_0.manual_bundles` is loaded and re-saved
- **THEN** the saved file contains the same `manual_bundles` entries in the same order

#### Scenario: Unknown version rejected
- **WHEN** a `.hdlc` with `version: 99` is loaded
- **THEN** loading SHALL fail with an error message naming the unsupported version

