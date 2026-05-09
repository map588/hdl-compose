## ADDED Requirements

### Requirement: .gitignore covers build artifacts
The project SHALL have a `.gitignore` that excludes `target/`, `*.hdlc` (generated projects), and common editor files.

#### Scenario: Cargo build output ignored
- **WHEN** `cargo build` is run
- **THEN** `target/` SHALL be excluded from git tracking

### Requirement: rustfmt configuration
The project SHALL have a `rustfmt.toml` with consistent formatting settings.

#### Scenario: Format check passes
- **WHEN** `cargo fmt --check` is run
- **THEN** all source files SHALL pass without changes needed

### Requirement: Clippy passes clean
All source code SHALL pass `cargo clippy` without warnings.

#### Scenario: Clean clippy run
- **WHEN** `cargo clippy -- -D warnings` is run
- **THEN** no warnings SHALL be reported

### Requirement: Integration test fixtures
The project SHALL include HDL fixture files in `tests/fixtures/` for integration testing.

#### Scenario: VHDL fixture exists
- **WHEN** integration tests run
- **THEN** `tests/fixtures/counter.vhd` SHALL exist and be a valid, parseable VHDL entity

#### Scenario: Verilog fixture exists
- **WHEN** integration tests run
- **THEN** `tests/fixtures/counter.v` SHALL exist and be a valid, parseable Verilog module
