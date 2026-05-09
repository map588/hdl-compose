# multi-load-nets Specification

## Purpose
TBD - created by archiving change multi-load-nets. Update Purpose after archive.
## Requirements
### Requirement: Input-to-input wiring creates a shared net
When the user wires two input pins, `WireTool` SHALL NOT reject the pairing on direction grounds. Instead it SHALL route both ports onto the same net identity per the rules below. No driver dialog appears — the user can later add a real driver (top-port, instance output) by wiring into any member of the net.

#### Scenario: One input already driven
- **WHEN** the user wires input pin `a` (already connected to a net) to input pin `b` (unconnected)
- **THEN** `b`'s port_map entry becomes the same `NetRef` as `a`'s
- **AND** no prompt appears

#### Scenario: Both inputs already on the same net
- **WHEN** the user re-wires two inputs that already share a net
- **THEN** the action is a silent no-op

#### Scenario: Neither input driven — undriven shared signal
- **WHEN** the user wires two unconnected input pins
- **THEN** both pins' port_map entries are set to `NetRef::InstancePort(first_pin.inst, first_pin.port)` so they share a net identity
- **AND** codegen emits a single `signal` declaration; the net has no driver until one is added
- **AND** validation emits a warning (not an error) noting the net is undriven

#### Scenario: Sticky after commit for net building
- **WHEN** a multi-load commit succeeds
- **THEN** the first pin remains armed so the user can click additional input pins to extend the same net without re-arming
- **AND** pressing Esc or clicking the armed pin cancels

### Requirement: Output-to-output pairing is rejected
`WireTool::compatibilityError` SHALL reject a wire between two instance outputs.

#### Scenario: Two instance outputs clicked
- **WHEN** the user wires two instance output pins
- **THEN** the destination pin flashes red
- **AND** the tooltip reads `output-to-output: only one driver per net allowed`

#### Scenario: Top-port involved bypasses the output-to-output check
- **WHEN** either or both pins are top-level ports
- **THEN** the direction check passes (top-port direction semantics are flipped from inside the wrapper) and the wire commits via direct `set_port_map_entry` on the instance side

