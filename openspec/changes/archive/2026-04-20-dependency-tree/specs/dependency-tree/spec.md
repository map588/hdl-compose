## ADDED Requirements

### Requirement: Parser extracts unique component references per module
The parser SHALL collect a list of unique module names referenced in each parsed module's body. The list MUST be attached to the corresponding `ModuleDef`.

#### Scenario: VHDL module with one component instantiation
- **WHEN** a VHDL file defines entity `n_register` whose architecture instantiates `flipflop` in a `for-generate` loop
- **THEN** `ModuleDef.dependencies` for `n_register` SHALL contain `["flipflop"]`

#### Scenario: VHDL module with multiple component instantiations
- **WHEN** a VHDL file defines entity `datapath` whose architecture instantiates `alu`, `register_file`, and `alu` again
- **THEN** `ModuleDef.dependencies` for `datapath` SHALL contain exactly `["alu", "register_file"]` (deduplicated)

#### Scenario: Verilog module with instance references
- **WHEN** a SystemVerilog file defines module `counter` whose body contains `flipflop ff1 (...)`, `flipflop ff2 (...)`, `gate_and g1 (...)`
- **THEN** `ModuleDef.dependencies` for `counter` SHALL contain `["flipflop", "gate_and"]` (deduplicated, order-preserving)

#### Scenario: Leaf module with no instantiations
- **WHEN** a file defines a module whose body contains only signal assignments and processes, no instantiations
- **THEN** `ModuleDef.dependencies` SHALL be an empty vector

### Requirement: AppState exposes per-instance dependency enumeration
AppState SHALL expose invokables to enumerate the dependencies of the module associated with each canvas instance, and check whether each dependency is present in the current library.

#### Scenario: Instance of a module with one dependency
- **WHEN** instance `u_reg_0` references module `n_register` whose dependencies are `["flipflop"]`
- **THEN** `instance_dependency_count(index_of_u_reg_0) == 1` and `instance_dependency_name(index_of_u_reg_0, 0) == "flipflop"`

#### Scenario: Present dependency
- **WHEN** the library contains a `ModuleDef` named `flipflop`
- **THEN** `instance_dependency_present(instance_index, flipflop_dep_index) == true`

#### Scenario: Missing dependency
- **WHEN** the library does not contain a `ModuleDef` named `flipflop`
- **THEN** `instance_dependency_present(instance_index, flipflop_dep_index) == false`

#### Scenario: Instance of a module not resolved in the library
- **WHEN** `u_foo`'s module reference resolves to no library entry
- **THEN** `instance_dependency_count(index_of_u_foo) == 0` (no dependencies to enumerate)

### Requirement: Sidebar renders dependencies as child rows with presence state
The sidebar tree SHALL display each canvas instance's dependencies as child rows beneath the instance row. Missing dependencies MUST render with a red foreground color and a warning prefix icon. Present dependencies MUST render with the default foreground.

#### Scenario: Instance with all dependencies present
- **WHEN** `u_reg_0` (module `n_register`) has dependency `flipflop` and `flipflop` is in the library
- **THEN** the sidebar tree shows `u_reg_0 : n_register` with one child row `flipflop` in default color

#### Scenario: Instance with missing dependency
- **WHEN** `u_reg_0` (module `n_register`) has dependency `flipflop` and the library does NOT contain `flipflop`
- **THEN** the sidebar tree shows `u_reg_0 : n_register` with one child row `⚠ flipflop` rendered in red foreground; the tooltip SHALL read `Module not in library`

#### Scenario: Adding a missing source turns red child green
- **WHEN** a dependency is missing (red in the tree) and the user then adds the corresponding `.vhd`/`.sv` to the library via File → Add HDL Source
- **THEN** `library_changed` fires, the tree rebuilds, and the dependency row renders in the default color

#### Scenario: Removing a source turns green child red
- **WHEN** a dependency is present (default color) and the user removes the corresponding source from `library_paths`
- **THEN** the tree rebuilds and the dependency row renders red with the warning prefix
