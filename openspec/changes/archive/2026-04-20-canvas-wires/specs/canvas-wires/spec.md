## ADDED Requirements

### Requirement: Wire rendering between connected pins
The canvas SHALL render a wire for every connected entry in any instance's port map. Wires MUST route with Manhattan geometry (horizontal and vertical segments only, no diagonals).

#### Scenario: Rendering wires on project load
- **WHEN** a `.hdlc` file with connected port maps is opened
- **THEN** one wire is drawn for each `Some(NetRef)` port-map entry, from the driver pin to the consumer pin, using Manhattan routing

#### Scenario: Driver with multiple loads
- **WHEN** three instance inputs are all driven by the same output pin
- **THEN** three separate wires render, one from the driver to each consumer

#### Scenario: No wires for open ports
- **WHEN** an instance port's port-map entry is `None` (open)
- **THEN** no wire is rendered for that entry

### Requirement: Live wire re-routing on instance move
Wires SHALL re-route when any endpoint's instance moves.

#### Scenario: Dragging an instance re-routes its wires
- **WHEN** an instance with N attached wires is dragged and released
- **THEN** all N wires recompute their Manhattan paths to the instance's new position

#### Scenario: Adding a new connection re-renders immediately
- **WHEN** `AppState::set_port_map_entry(...)` emits `port_map_changed`
- **THEN** the corresponding wire appears on the canvas without requiring a project reload

### Requirement: Click-port-click-port wiring
The canvas SHALL support creating connections by clicking one pin then another.

#### Scenario: Valid connection
- **WHEN** the user clicks an output pin (arming the wire tool) and then clicks a compatible input pin (matching direction, type, width)
- **THEN** `set_port_map_entry(target_instance, target_port, driver_rhs)` is called; the wire appears; the wire tool disarms

#### Scenario: Canceling a pending wire
- **WHEN** the wire tool is armed and the user presses Escape or clicks empty canvas
- **THEN** the wire tool disarms; no connection is created

#### Scenario: Clicking the same source pin cancels
- **WHEN** the wire tool is armed on pin P and the user clicks pin P again
- **THEN** the wire tool disarms

### Requirement: Invalid connection rejection with visual feedback
Attempts to create a connection between incompatible pins SHALL be rejected. Rejection MUST be communicated with a red visual flash and a tooltip that names the reason.

#### Scenario: Direction mismatch
- **WHEN** the user clicks an output pin, then clicks another output pin
- **THEN** the target pin flashes red for ~500 ms, a tooltip shows "direction mismatch: output cannot drive output", and no connection is created

#### Scenario: Width mismatch
- **WHEN** the user wires an 8-bit output to a 16-bit input
- **THEN** the target flashes red with tooltip "width mismatch: 8 → 16"; no connection is created

#### Scenario: Type mismatch
- **WHEN** the user wires a `std_logic` to a `std_logic_vector`
- **THEN** the target flashes red with tooltip "type mismatch: std_logic → std_logic_vector"; no connection is created

### Requirement: Right-click wire rename sets alias
Right-clicking a wire SHALL open a rename dialog. Accepting MUST call `AppState::set_alias(net_key, alias)`.

#### Scenario: Renaming a wire
- **WHEN** the user right-clicks a wire and chooses Rename, enters `clk_sys` in the QInputDialog, and confirms
- **THEN** `AppState::set_alias(net_key, "clk_sys")` is called; the `alias_changed` signal fires; the generated signal name is now `clk_sys`

#### Scenario: Empty alias clears
- **WHEN** the user accepts the dialog with an empty string
- **THEN** `AppState::remove_alias(net_key)` (or equivalent) is called and the alias is removed

#### Scenario: Canceled rename
- **WHEN** the user cancels the QInputDialog
- **THEN** the alias is unchanged and no signals fire

### Requirement: AppState exposes wire enumeration and alias invokables
AppState SHALL expose enumeration invokables for rendering current wires and an invokable to set aliases.

#### Scenario: Wire enumeration
- **WHEN** C++ calls `AppState::wire_count()` then loops `wire_source(i)` and `wire_target(i)` for each i
- **THEN** the calls return the total number of currently-connected port-map entries and, for each, the driver side as `top:<name>` or `<inst>.<port>` and the load side as `<inst>.<port>`

#### Scenario: Set alias
- **WHEN** C++ calls `AppState::set_alias(net_key, alias)` with a valid NetRef serialization
- **THEN** the Schematic's alias map is updated, `dirty` is set to true, and `alias_changed(net_key)` is emitted
