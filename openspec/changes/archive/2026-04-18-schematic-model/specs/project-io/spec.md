## ADDED Requirements

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
The project file SHALL include a `"version": 2` field. The loader SHALL reject files with unknown versions.

#### Scenario: Load version 2 file
- **WHEN** a file with `"version": 2` is loaded
- **THEN** loading SHALL succeed

#### Scenario: Reject unknown version
- **WHEN** a file with `"version": 99` is loaded
- **THEN** loading SHALL return an error indicating unsupported version
