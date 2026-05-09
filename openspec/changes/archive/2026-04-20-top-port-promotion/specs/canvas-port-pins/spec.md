## ADDED Requirements

### Requirement: Pin context menu offers port promotion
`PortPinItem` SHALL provide a right-click context menu with a `Promote to top-level port` action. The action SHALL delegate to `AppState::promote_port_to_top`.

#### Scenario: Right-click shows promote action
- **WHEN** the user right-clicks an instance pin
- **THEN** a context menu appears with a `Promote to top-level port` entry (among any other pin-context actions)

#### Scenario: Choosing the action promotes the port
- **WHEN** the user chooses the promote action on a pin
- **THEN** `AppState::promote_port_to_top(instance, port)` is invoked
- **AND** on success the canvas rebuilds to show the new top-port connector and wire
