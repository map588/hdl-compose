# wire-interaction Specification

## Purpose
TBD - created by archiving change wire-ux-overhaul. Update Purpose after archive.
## Requirements
### Requirement: Drag-to-wire
Mouse-down on a `PortPinItem` SHALL enter wire-drag mode. A provisional wire path tracks the cursor. On mouse-up over a compatible pin the wire SHALL commit; over any other target it SHALL cancel silently.

#### Scenario: Commit via drag-to-wire
- **WHEN** the user presses mouse-down on the output pin `counter_0.count`, drags to the input pin `fifo_0.din` (a compatible target), and releases
- **THEN** the provisional wire disappears, `AppState::set_port_map_entry("fifo_0", "din", "counter_0.count")` is called, and the committed wire renders Manhattan-routed between the two pins

#### Scenario: Cancel drag-to-wire on empty space
- **WHEN** the user presses mouse-down on a pin, drags to empty canvas, and releases
- **THEN** the provisional wire disappears and no `port_map` entry is created

#### Scenario: Cancel drag-to-wire on incompatible pin
- **WHEN** the user drags from an output pin and releases over an input pin whose direction or width mismatches
- **THEN** the provisional wire disappears, the target pin flashes red with a tooltip explaining the mismatch, and no `port_map` entry is created

### Requirement: Click-to-wire retained
Click-to-wire SHALL remain functional as an alternative path: a short mouse-press on a pin (released within 3 pixels of the down position) arms the pin. A subsequent click on a compatible pin commits.

#### Scenario: Two-click commit
- **WHEN** the user clicks output pin `counter_0.count` (without dragging), then clicks input pin `fifo_0.din`
- **THEN** the wire commits exactly as in drag-to-wire

#### Scenario: Click on armed pin cancels
- **WHEN** a pin is armed and the user clicks the same pin again
- **THEN** the armed state clears and no wire is created

### Requirement: Armed-pin visual feedback
When `WireTool` has an armed source pin, that pin SHALL render with a distinct visual state (outline glow or accent color) so the user knows which pin is the in-flight source.

#### Scenario: Arming paints glow
- **WHEN** the user arms a pin
- **THEN** that pin's painted glyph adds a visible outline glow or accent color

#### Scenario: Cancel removes glow
- **WHEN** the armed state clears (via commit, Esc, or clicking the same pin)
- **THEN** the pin returns to its default rendering in the next paint

### Requirement: Escape cancels pending wire and selection
Pressing `Esc` while the canvas has focus SHALL cancel any pending wire in `WireTool` AND clear the scene's selection.

#### Scenario: Esc during drag-to-wire
- **WHEN** the user is in the middle of a drag-to-wire (button still held)
- **THEN** pressing Esc aborts the drag, discards the provisional wire, and clears armed state

#### Scenario: Esc with only selection active
- **WHEN** one or more items are selected and no wire is pending
- **THEN** pressing Esc clears the selection

#### Scenario: Esc with armed-click pending
- **WHEN** a pin is armed (click-to-wire mid-flight) and no drag is in progress
- **THEN** pressing Esc clears the armed state

### Requirement: Wire selection and deletion
`WireItem` SHALL be selectable. `Delete` or `Backspace` on a selected wire SHALL remove the connection by clearing the load-side `port_map` entry.

#### Scenario: Select a wire
- **WHEN** the user clicks on the rendered wire path
- **THEN** the wire paints in its selected style (distinct high-contrast color) and the scene reports one selected item

#### Scenario: Delete selected wire
- **WHEN** one or more wires are selected and the user presses `Delete` or `Backspace`
- **THEN** `AppState::clear_port_map_entry(load_inst, load_port)` is called for each selected wire, the wires disappear from the canvas, and the load-side port is reported unconnected

#### Scenario: Delete last wire to an aliased net
- **WHEN** the last wire referencing an aliased net is deleted
- **THEN** that alias is also removed from `Schematic.aliases`

### Requirement: Selection highlight
Selected `InstanceItem`s AND selected `WireItem`s SHALL render with a distinct selection color so multi-selection is visible.

#### Scenario: Multi-select renders consistent highlight
- **WHEN** the user rubber-band-selects two instances and one wire
- **THEN** all three items paint with the selection accent color

### Requirement: Bit-slice connect via right-click
Right-clicking a pin or a wire SHALL offer a "Connect slice..." action when the target port is multi-bit. Selecting it SHALL open a dialog that captures a bit index or range; accepting SHALL create a port_map entry with a slice expression.

#### Scenario: Slice dialog for multi-bit target
- **WHEN** the user right-clicks the multi-bit input pin `fifo_0.din` (width 8)
- **THEN** the context menu contains `Connect slice...`; choosing it opens a dialog with driver-selection and slice-spec fields (single bit or range)

#### Scenario: Single-bit slice commit
- **WHEN** the user completes the dialog with driver `counter_0.count` and slice `[0]`
- **THEN** the port_map entry for `fifo_0.din` SHALL be `Some(NetRef::InstancePortSlice("counter_0", "count", SliceExpr::Bit(0)))`

#### Scenario: Range slice commit
- **WHEN** the user completes the dialog with driver `counter_0.count` and slice `[7:4]`
- **THEN** the port_map entry SHALL be `Some(NetRef::InstancePortSlice("counter_0", "count", SliceExpr::Range { high: 7, low: 4 }))`

#### Scenario: Slice not offered for scalar target
- **WHEN** the user right-clicks a scalar (`std_logic`) pin
- **THEN** the context menu SHALL NOT contain `Connect slice...`

#### Scenario: Slice round-trips through `.hdlc`
- **WHEN** a project with a slice port_map entry is saved and reopened
- **THEN** the entry deserializes to the identical `NetRef::InstancePortSlice` variant

#### Scenario: Slice emitted in VHDL codegen
- **WHEN** codegen runs on a schematic where `fifo_0.din` has a slice entry `Range(7, 4)` against `counter_0.count`
- **THEN** the emitted association SHALL be `din => u_counter_0_count(7 downto 4)` (or equivalent using the alias/derived name)

#### Scenario: Slice emitted in SystemVerilog codegen
- **WHEN** the same schematic targets SV
- **THEN** the emitted association SHALL be `.din(u_counter_0_count[7:4])`

