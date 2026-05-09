## ADDED Requirements

### Requirement: Schematic holds design state
The `Schematic` struct SHALL hold the complete state of a structural design: top-level name, language, top-level generics, top-level ports, instances, net aliases, and library paths.

#### Scenario: Create empty schematic
- **WHEN** a new `Schematic` is created with name `"top_level"` and language `Vhdl`
- **THEN** it SHALL have empty instances, empty aliases, and empty library paths

#### Scenario: Language is immutable after creation
- **WHEN** a `Schematic` is created with language `Vhdl`
- **THEN** the language field SHALL be `Vhdl` and codegen SHALL produce VHDL output

### Requirement: Instance references a module by name
Each `Instance` SHALL contain a unique name, a module reference (by name string), a generic map, a port map, and a canvas position.

#### Scenario: Add instance to schematic
- **WHEN** an instance `u_fifo` referencing module `fifo_sync` is added
- **THEN** the schematic SHALL contain one instance with name `"u_fifo"` and module_ref `"fifo_sync"`

#### Scenario: Duplicate instance name rejected
- **WHEN** an instance with name `"u_fifo"` already exists and another instance with the same name is added
- **THEN** the operation SHALL return an error indicating duplicate instance name

### Requirement: Port map uses NetRef for connections
Each entry in an instance's port map SHALL be `Option<NetRef>` where `NetRef` is either `TopPort(name)`, `InstancePort(instance_name, port_name)`, or the port is `None` (unconnected/open).

#### Scenario: Connect to top-level port
- **WHEN** instance port `clk` is mapped to `NetRef::TopPort("clk_sys")`
- **THEN** the port map entry for `clk` SHALL be `Some(NetRef::TopPort("clk_sys"))`

#### Scenario: Connect to another instance's output
- **WHEN** instance `u_fifo` port `din` is mapped to `NetRef::InstancePort("u_adc", "data_out")`
- **THEN** the port map entry for `din` SHALL be `Some(NetRef::InstancePort("u_adc", "data_out"))`

#### Scenario: Port left unconnected
- **WHEN** instance port `full` is not mapped
- **THEN** the port map entry for `full` SHALL be `None`

### Requirement: Net aliases provide user-chosen signal names
The schematic SHALL maintain an alias map from net identity (driver) to user-chosen name strings. Aliases affect generated signal names only, not connectivity.

#### Scenario: Set alias for internal net
- **WHEN** the net driven by `("u_pll", "clk_out")` is aliased to `"sys_clk"`
- **THEN** codegen SHALL use `sys_clk` as the signal name instead of `u_pll_clk_out`

#### Scenario: No alias uses derived name
- **WHEN** no alias exists for net driven by `("u_adc", "data_out")`
- **THEN** codegen SHALL derive the signal name as `u_adc_data_out`

### Requirement: Resolve module references against library
Given a set of library paths, the schematic SHALL resolve each instance's module_ref to a `ModuleDef` by parsing the library files and matching by name.

#### Scenario: Module found in library
- **WHEN** instance `u_fifo` references module `fifo_sync` and `fifo_sync.vhd` is in the library paths
- **THEN** resolution SHALL return the parsed `ModuleDef` for `fifo_sync`

#### Scenario: Module not found
- **WHEN** instance references module `nonexistent` and no library file contains that module
- **THEN** resolution SHALL report an error identifying the missing module

### Requirement: Validate schematic for correctness
The schematic SHALL provide a validation function that checks for errors without preventing invalid intermediate states during editing.

#### Scenario: Direction mismatch detected
- **WHEN** an instance input port is mapped to a `NetRef::InstancePort` that references another instance's input port (not an output)
- **THEN** validation SHALL report a direction mismatch diagnostic

#### Scenario: Width mismatch detected
- **WHEN** an instance port of width 8 is connected to a net whose driver port has width 16
- **THEN** validation SHALL report a width mismatch diagnostic

#### Scenario: Reference to nonexistent instance
- **WHEN** a port map contains `NetRef::InstancePort("u_ghost", "data")` but no instance named `u_ghost` exists
- **THEN** validation SHALL report a missing instance diagnostic

#### Scenario: Reference to nonexistent port
- **WHEN** a port map contains `NetRef::InstancePort("u_adc", "nonexistent")` but `u_adc`'s module has no port named `nonexistent`
- **THEN** validation SHALL report a missing port diagnostic

#### Scenario: Duplicate alias names
- **WHEN** two different nets are aliased to the same name `"sys_clk"`
- **THEN** validation SHALL report a duplicate alias diagnostic

#### Scenario: Unconnected ports are warnings, not errors
- **WHEN** an instance port has no mapping (None)
- **THEN** validation SHALL report a warning, not an error — codegen emits `open` for these

### Requirement: Generic map stores expression strings
Each instance's generic map SHALL map generic names to string expressions. No expression evaluation — values are passed through to generated HDL verbatim.

#### Scenario: Set generic value
- **WHEN** instance `u_fifo` sets generic `DEPTH` to `"1024"`
- **THEN** the generic map SHALL contain `("DEPTH", "1024")`

#### Scenario: Expression as generic value
- **WHEN** instance sets generic `WIDTH` to `"DATA_WIDTH * 2"`
- **THEN** the generic map SHALL store the expression string as-is
