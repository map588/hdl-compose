## ADDED Requirements

### Requirement: Rubber-band selection on empty canvas
Mouse-down on empty canvas followed by a drag SHALL begin a rubber-band selection rectangle. Any `InstanceItem` or `WireItem` intersected by the rectangle on mouse-up SHALL become selected.

#### Scenario: Rubber-band selects multiple items
- **WHEN** the user presses on empty canvas, drags a rectangle over two instances and one wire, and releases
- **THEN** those two instances and that wire become the current selection, each rendered in the selection style

#### Scenario: Rubber-band with no items intersected
- **WHEN** the user drags a rubber-band rectangle that intersects nothing
- **THEN** the current selection is cleared on mouse-up

#### Scenario: Shift+rubber-band extends selection
- **WHEN** the user holds Shift during rubber-band
- **THEN** intersected items are added to the existing selection rather than replacing it

## MODIFIED Requirements

### Requirement: Click selection syncs with sidebar
Clicking an instance on the canvas SHALL select it. Selection MUST be reflected in the sidebar tree. Selection state MUST also be visible in the canvas rendering with a distinct selection color.

#### Scenario: Canvas-originated selection
- **WHEN** the user clicks an instance rectangle
- **THEN** the rectangle paints with the selection-highlight border, and the corresponding sidebar tree row becomes the current selection

#### Scenario: Sidebar-originated selection
- **WHEN** the user clicks an instance row in the sidebar tree
- **THEN** the corresponding canvas rectangle paints with the selection-highlight border

#### Scenario: Multi-select via rubber-band
- **WHEN** multiple instances are selected via rubber-band
- **THEN** each selected instance renders with the selection-highlight border; sidebar reflects the last-focused item

#### Scenario: Clicking empty canvas clears selection
- **WHEN** the user clicks empty canvas without initiating a rubber-band drag
- **THEN** the current selection is cleared

## ADDED Requirements

### Requirement: Removing an instance cleans up wires and references
When an instance is removed, the canvas SHALL remove its rectangle and drop any wire or port-map reference that pointed at it.

#### Scenario: Removing an instance
- **WHEN** the user invokes `remove_instance` (via sidebar context menu, canvas Delete key, or model API)
- **THEN** the corresponding rectangle disappears from the canvas, every wire ending at one of its pins is removed, and every other instance's `port_map` entries that referenced this instance are set to `None`

#### Scenario: Re-adding an instance with the same name
- **WHEN** an instance named `u_foo` is removed and a new instance named `u_foo` is dropped onto the canvas
- **THEN** the new instance has an empty `port_map` and no phantom wires appear — previously-connected ports of siblings stay unconnected until the user wires them again
