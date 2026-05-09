## ADDED Requirements

### Requirement: Armed pin renders distinct visual state
When `WireTool` has armed a pin as the source of an in-flight wire, that pin SHALL render with an additional visual indicator (outline glow, accent color, or equivalent) distinct from its default state.

#### Scenario: Pin gains glow when armed
- **WHEN** a pin is armed by mouse-press or click
- **THEN** that pin paints with the armed visual state in its next paint cycle

#### Scenario: Pin returns to default when disarmed
- **WHEN** the armed state clears (commit, Esc, or click on same pin)
- **THEN** the pin paints without the armed visual state in its next paint cycle

## MODIFIED Requirements

### Requirement: Port pins on instance edges
Each instance rectangle SHALL display its module's ports as small graphical items anchored to the rectangle's left and right edges. Input ports MUST be on the left edge, output ports on the right edge, inout ports on whichever edge has fewer pins. Each pin's hit region MUST be limited to the pin tip only, not the full port row.

#### Scenario: Rendering pins for an instance
- **WHEN** an instance of a module with M inputs and N outputs is on the canvas
- **THEN** M pin items appear on the instance's left edge and N pin items appear on the right edge, each labeled with its port name

#### Scenario: Re-rendering after module re-parse changes the port list
- **WHEN** the module's port list changes via re-parse (dropping connections, adding new ports)
- **THEN** the pins on every instance of that module refresh to match the new list

#### Scenario: Clicking the port label row does not arm the pin
- **WHEN** the user clicks on the port label or an empty portion of the port's row
- **THEN** the click falls through to the `InstanceItem` for drag or selection; `WireTool` is NOT armed

#### Scenario: Clicking the pin tip arms the pin
- **WHEN** the user clicks on the small arrow/diamond glyph at the end of the port row
- **THEN** `WireTool` arms on that pin
