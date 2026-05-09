## ADDED Requirements

### Requirement: Removing an instance cleans sibling port maps
`Schematic::remove_instance(name)` SHALL remove the named instance AND sweep every other instance's `port_map`, setting to `None` every entry whose value references the removed instance by name. Aliases whose driver referenced the removed instance SHALL also be removed.

#### Scenario: Sibling port map cleared on removal
- **WHEN** instance `u_foo` is removed and another instance `u_bar` has a `port_map` entry `Some(NetRef::InstancePort("u_foo", "out"))`
- **THEN** after removal, `u_bar`'s entry for that port SHALL be `None`

#### Scenario: Alias dropped when its driver is removed
- **WHEN** instance `u_pll` is removed and the aliases map contained a key derived from `("u_pll", "clk_out") → "sys_clk"`
- **THEN** after removal, the alias SHALL be absent from the aliases map

#### Scenario: Re-adding an instance with the same name starts fresh
- **WHEN** an instance named `u_foo` is removed and a new instance with name `u_foo` is added
- **THEN** its `port_map` SHALL be empty and no sibling SHALL have any `NetRef::InstancePort("u_foo", _)` entry unless the user re-creates it explicitly

## MODIFIED Requirements

### Requirement: Port map uses NetRef for connections
Each entry in an instance's port map SHALL be `Option<NetRef>` where `NetRef` is one of: `TopPort(name)`, `InstancePort(instance_name, port_name)`, `TopPortSlice(name, slice)`, `InstancePortSlice(instance_name, port_name, slice)`, or `None` (unconnected / open). `slice` is a `SliceExpr` which is either a single bit index `Bit(i)` or a descending range `Range { high, low }`.

#### Scenario: Connect to top-level port
- **WHEN** instance port `clk` is mapped to `NetRef::TopPort("clk_sys")`
- **THEN** the port map entry for `clk` SHALL be `Some(NetRef::TopPort("clk_sys"))`

#### Scenario: Connect to another instance's output
- **WHEN** instance `u_fifo` port `din` is mapped to `NetRef::InstancePort("u_adc", "data_out")`
- **THEN** the port map entry for `din` SHALL be `Some(NetRef::InstancePort("u_adc", "data_out"))`

#### Scenario: Connect to a single bit of another instance's output
- **WHEN** instance `u_led` port `led` is mapped to `NetRef::InstancePortSlice("u_counter", "count", SliceExpr::Bit(0))`
- **THEN** the port map entry SHALL be `Some(NetRef::InstancePortSlice("u_counter", "count", SliceExpr::Bit(0)))`

#### Scenario: Connect to a bit range of a top-level port
- **WHEN** instance port `data_in` is mapped to `NetRef::TopPortSlice("bus", SliceExpr::Range { high: 7, low: 4 })`
- **THEN** the port map entry SHALL be `Some(NetRef::TopPortSlice("bus", SliceExpr::Range { high: 7, low: 4 }))`

#### Scenario: Port left unconnected
- **WHEN** instance port `full` is not mapped
- **THEN** the port map entry for `full` SHALL be `None`

### Requirement: Validate schematic for correctness
The schematic SHALL provide a validation function that checks for errors without preventing invalid intermediate states during editing. Validation SHALL treat scalar-vs-unresolved-vector width pairs as mismatches and slice-out-of-range as an error.

#### Scenario: Direction mismatch detected
- **WHEN** an instance input port is mapped to a `NetRef::InstancePort` that references another instance's input port (not an output)
- **THEN** validation SHALL report a direction mismatch diagnostic

#### Scenario: Width mismatch detected
- **WHEN** an instance port of width 8 is connected to a net whose driver port has width 16
- **THEN** validation SHALL report a width mismatch diagnostic

#### Scenario: Scalar-to-vector mismatch detected even when vector width is unresolved
- **WHEN** a scalar (`std_logic`) port is connected to a vector port whose width is unresolved (generic-sized)
- **THEN** validation SHALL report a width mismatch diagnostic

#### Scenario: Slice out of range detected
- **WHEN** a port map entry uses `SliceExpr::Range { high: 15, low: 8 }` against a driver port of resolved width 8
- **THEN** validation SHALL report a slice-out-of-range diagnostic

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
