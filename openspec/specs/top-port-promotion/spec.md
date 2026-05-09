# top-port-promotion Specification

## Purpose
TBD - created by archiving change top-port-promotion. Update Purpose after archive.
## Requirements
### Requirement: Promote instance pin to top-level port
A right-click context menu on an `PortPinItem` SHALL offer `Promote to top-level port`. Selecting it SHALL create a matching top-level port (if one does not already exist) and wire the pin to it.

#### Scenario: Fresh promotion creates a new top-port
- **WHEN** the user right-clicks `counter_0.clk` and chooses "Promote to top-level port" and no top-port named `clk` exists
- **THEN** `Schematic.top_ports` gains a new `PortDef` with name `clk`, direction `In`, the same `port_type` and `bundle` as the source pin
- **AND** the port_map entry for `counter_0.clk` becomes `Some(NetRef::TopPort("clk"))`
- **AND** the canvas shows a new top-port connector on the left boundary with a wire to `counter_0.clk`

#### Scenario: Promotion reuses an existing compatible top-port
- **WHEN** a top-port `clk` with matching direction / type / bundle already exists, and the user promotes `counter_1.clk`
- **THEN** no new top-port is created
- **AND** the port_map entry for `counter_1.clk` becomes `Some(NetRef::TopPort("clk"))`

#### Scenario: Name collision with incompatible top-port
- **WHEN** a top-port `clk` exists but with different direction or type, and the user promotes another pin named `clk`
- **THEN** the new top-port is named `clk_1` (or the next free `clk_N`)
- **AND** the status bar shows the resolved name, e.g. `Promoted as 'clk_1'`

### Requirement: Direction mirrors source pin
The new top-level port SHALL have the same direction as the source instance pin. Input pins promote to top-level inputs; output pins to top-level outputs; inout pins to top-level inouts.

#### Scenario: Output pin promotes to top-level output
- **WHEN** the user promotes `uart_0.tx` (Out) and no `tx` top-port exists
- **THEN** the new top-port `tx` has direction `Out`
- **AND** it renders on the right boundary of the canvas

### Requirement: AppState invokable for promotion
`AppState::promote_port_to_top(instance, port)` SHALL perform the promotion atomically and return the resolved top-port name. On failure it SHALL return an empty `QString` and set `last_error`.

#### Scenario: Invokable on unknown instance
- **WHEN** C++ calls `promote_port_to_top("u_ghost", "clk")` with no such instance
- **THEN** the call returns an empty QString and `last_error()` reports `instance not found`

