## ADDED Requirements

### Requirement: Hierarchy tree shows design structure
The sidebar SHALL display a QTreeView where the root is the top-level schematic and each instance appears as a child labeled `u_name : module_name`.

#### Scenario: Instances listed
- **WHEN** a schematic has instances u_fifo and u_counter
- **THEN** the tree SHALL show two children: `u_fifo : fifo_sync` and `u_counter : counter`

#### Scenario: Click instance to select
- **WHEN** the user clicks an instance in the tree
- **THEN** that instance SHALL be selected on the canvas and its port map SHALL appear in the mini editor

### Requirement: Library pane shows available modules
Below a divider in the sidebar, a library pane SHALL list all parsed modules from the library paths that are not yet placed as instances.

#### Scenario: Unplaced modules listed
- **WHEN** the library contains modules `counter`, `fifo_sync`, and `pll_main`, and only `counter` is instantiated
- **THEN** the library pane SHALL list `fifo_sync` and `pll_main`

#### Scenario: Drag to instantiate
- **WHEN** the user drags a module from the library pane onto the canvas
- **THEN** a new instance SHALL be created with a default name (e.g., `u_fifo_sync`) and placed at the drop position

### Requirement: Dirty instances have red dot
Instances marked dirty (module source changed incompatibly) SHALL display a red dot indicator in the sidebar.

#### Scenario: Dirty indicator
- **WHEN** an instance's module source changes and ports are incompatible
- **THEN** a red dot SHALL appear next to that instance in the hierarchy tree

### Requirement: Right-click context menu
Right-clicking an instance in the sidebar SHALL show a context menu with: Rename, Delete, Goto Source.

#### Scenario: Delete instance
- **WHEN** the user right-clicks an instance and selects Delete
- **THEN** the instance SHALL be removed from the schematic, and canvas and editor SHALL update
