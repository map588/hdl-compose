## MODIFIED Requirements

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
