## ADDED Requirements

### Requirement: File watch on library paths
The application SHALL watch all files in `Schematic.library_paths` for changes using QFileSystemWatcher.

#### Scenario: Source file modified externally
- **WHEN** a library HDL file is saved externally (e.g., user edits in neovim)
- **THEN** the application SHALL detect the change and re-parse the file

### Requirement: Port diff on re-parse
On re-parse, the application SHALL diff the new port list against the stored ModuleDef, comparing name, direction, type, and width for each port.

#### Scenario: Port unchanged
- **WHEN** a port exists in both old and new ModuleDef with identical name, direction, type, and width
- **THEN** connections to that port SHALL remain intact

#### Scenario: Port removed
- **WHEN** a port exists in the old ModuleDef but not in the new one
- **THEN** all connections to that port SHALL be dropped from every instance of that module

#### Scenario: Port changed (width, direction, or type)
- **WHEN** a port exists in both but its direction, type, or width differs
- **THEN** all connections to that port SHALL be dropped from every instance of that module

#### Scenario: Port added
- **WHEN** a port exists in the new ModuleDef but not in the old one
- **THEN** it SHALL appear unconnected in all instances of that module

### Requirement: Mark dirty instances
Instances whose module source changed incompatibly SHALL be marked dirty.

#### Scenario: Dirty marking
- **WHEN** at least one port was dropped due to a re-parse diff
- **THEN** the instance SHALL be marked dirty: red dot in sidebar, red outline on canvas

#### Scenario: Mini editor shows diagnostic
- **WHEN** a dirty instance is selected in the mini editor
- **THEN** broken port entries SHALL show the RHS cleared with a comment: `-- WAS: u_adc.data_out (port width changed 8 → 16)`

### Requirement: No auto-migration
The application SHALL NOT attempt to auto-migrate or string-match renamed ports. All breakage is explicit.

#### Scenario: Renamed port
- **WHEN** a port is renamed from `din` to `data_in` in the source
- **THEN** the old `din` connection SHALL be dropped and `data_in` SHALL appear unconnected — no "did you mean" suggestion

### Requirement: Save blocked on dirty instances for codegen
The codegen action SHALL be blocked when dirty instances exist. The project file (.hdlc) can still save.

#### Scenario: Codegen blocked
- **WHEN** the user triggers codegen with dirty instances present
- **THEN** the application SHALL show an error listing the dirty instances and refuse to generate

#### Scenario: Project save allowed
- **WHEN** the user saves the .hdlc project with dirty instances present
- **THEN** the save SHALL succeed (dirty state is preserved in the project)
