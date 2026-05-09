## ADDED Requirements

### Requirement: Opt-in match-by-name auto-connect
The application SHALL provide a button or keyboard shortcut on the selected instance that auto-connects ports by matching names with top-level ports, aliases, or other instance outputs.

#### Scenario: Matching ports connected
- **WHEN** the user selects `u_counter` and triggers match-by-name, and a top-level port `clk` exists matching the instance's `clk` port in name, direction, and type
- **THEN** the instance's `clk` port SHALL be connected to `NetRef::TopPort("clk")`

#### Scenario: Non-matching ports left unconnected
- **WHEN** match-by-name runs and a port has no matching name in scope
- **THEN** that port SHALL remain unconnected (no change)

#### Scenario: Never automatic on placement
- **WHEN** a new instance is created by dragging from the library
- **THEN** match-by-name SHALL NOT run automatically — all ports start unconnected

#### Scenario: Only compatible matches
- **WHEN** match-by-name finds a name match but direction or type is incompatible
- **THEN** that port SHALL NOT be connected
