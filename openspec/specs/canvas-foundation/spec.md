## Purpose

Defines the baseline canvas experience: `InstanceItem` rendering, drag-to-move with persistent position, click selection synced with the sidebar, pan/zoom navigation, and rubber-band multi-select. Wire and pin behavior are specified in sibling capabilities; this file covers the scene plumbing they sit on top of.
## Requirements
### Requirement: Instance rendering on canvas
The canvas SHALL render every instance in the active Schematic as a labeled rectangle. Each rectangle MUST display the instance name and module reference.

#### Scenario: Opening a project with instances
- **WHEN** the user opens a `.hdlc` file containing N instances
- **THEN** the canvas renders N rectangles, each positioned at the instance's stored `position`, each labeled with `<instance-name>` above `: <module-ref>`

#### Scenario: Creating an instance via library drag
- **WHEN** the user drags a module from the library pane onto the canvas
- **THEN** a new rectangle appears at the drop position, and `Instance.position` is set to the drop coordinates

#### Scenario: Removing an instance
- **WHEN** the user invokes `remove_instance` (via sidebar context menu or model API)
- **THEN** the corresponding rectangle disappears from the canvas

### Requirement: Draggable instances with persistent position
Instances SHALL be draggable within the canvas. The new position MUST be written back to the Schematic on drag-release (not on every pixel during drag).

#### Scenario: Dragging an instance to a new position
- **WHEN** the user presses and holds the left mouse button on an instance rectangle and moves the cursor
- **THEN** the rectangle follows the cursor smoothly

#### Scenario: Persisting the new position
- **WHEN** the user releases the mouse button after dragging
- **THEN** `Instance.position` is updated exactly once via `AppState::set_instance_position(name, x, y)`, the `instance_moved(name, x, y)` signal fires, and `dirty` is set to true

#### Scenario: Abandoning a drag
- **WHEN** the user presses mouse-down on an instance, moves less than 5 pixels, and releases
- **THEN** the instance's position is unchanged and `set_instance_position` is NOT called

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

### Requirement: Pan and zoom navigation
The canvas SHALL support pan and zoom gestures without interfering with drag-to-move on instances.

#### Scenario: Pan with middle-click drag
- **WHEN** the user presses the middle mouse button and drags
- **THEN** the view scrolls such that the world point under the cursor on mouse-down stays under the cursor throughout the drag

#### Scenario: Pan with two-finger scroll
- **WHEN** the user performs a two-finger scroll gesture on a trackpad
- **THEN** the view scrolls by the gesture's delta in the same direction

#### Scenario: Zoom with Ctrl+scroll
- **WHEN** the user holds Ctrl and scrolls the wheel
- **THEN** the view zooms in or out around the cursor position, with zoom factor clamped between 0.2× and 5×

### Requirement: Dirty instances show red outline
Instances whose `instance_is_dirty(index)` returns true SHALL paint with a red outline.

#### Scenario: Rendering a dirty instance
- **WHEN** `instance_is_dirty(i)` returns true for instance i
- **THEN** the instance's rectangle is outlined with a red pen in its `paint()` override

#### Scenario: Clean instance
- **WHEN** `instance_is_dirty(i)` returns false
- **THEN** the instance's rectangle is outlined with the default pen

### Requirement: AppState exposes position and selection invokables
The AppState QObject SHALL provide invokables and signals to mutate and observe instance position and selection state.

#### Scenario: Setting a position
- **WHEN** C++ calls `AppState::set_instance_position(name, x, y)` with an existing instance name
- **THEN** the Schematic's `Instance.position` is updated, `dirty` is set to true, and `instance_moved(name, x, y)` is emitted

#### Scenario: Reading a position
- **WHEN** C++ calls `AppState::instance_pos_x(i)` and `AppState::instance_pos_y(i)` for a valid index
- **THEN** the calls return the current f64 x/y coordinates stored on `Instance.position`

#### Scenario: Setting the selection
- **WHEN** C++ calls `AppState::set_selected_instance(name)`
- **THEN** `selection_changed(name)` is emitted; subsequent `AppState::selected_instance()` returns the given name

### Requirement: Removing an instance cleans up wires and references
When an instance is removed, the canvas SHALL remove its rectangle and drop any wire or port-map reference that pointed at it.

#### Scenario: Removing an instance
- **WHEN** the user invokes `remove_instance` (via sidebar context menu, canvas Delete key, or model API)
- **THEN** the corresponding rectangle disappears from the canvas, every wire ending at one of its pins is removed, and every other instance's `port_map` entries that referenced this instance are set to `None`

#### Scenario: Re-adding an instance with the same name
- **WHEN** an instance named `u_foo` is removed and a new instance named `u_foo` is dropped onto the canvas
- **THEN** the new instance has an empty `port_map` and no phantom wires appear — previously-connected ports of siblings stay unconnected until the user wires them again

