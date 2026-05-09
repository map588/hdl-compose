## ADDED Requirements

### Requirement: Parse VHDL entity name
The parser SHALL extract the entity name from a VHDL source file and populate `ModuleDef.name`.

#### Scenario: Simple entity declaration
- **WHEN** a `.vhd` file contains `entity counter is ... end entity counter;`
- **THEN** `ModuleDef.name` SHALL equal `"counter"`

#### Scenario: Multiple entities in one file
- **WHEN** a `.vhd` file contains two entity declarations (`foo` and `bar`)
- **THEN** `parse_file` SHALL return a `Vec` with two `ModuleDef` entries, one per entity

### Requirement: Parse VHDL generics
The parser SHALL extract all generic declarations from the entity and populate `ModuleDef.generics` as `Vec<GenericDef>`.

#### Scenario: Entity with integer generics
- **WHEN** an entity declares `generic (WIDTH : integer := 8; DEPTH : integer := 256)`
- **THEN** `ModuleDef.generics` SHALL contain two entries with names `"WIDTH"` and `"DEPTH"`, their types, and default values

#### Scenario: Entity with no generics
- **WHEN** an entity has no `generic` clause
- **THEN** `ModuleDef.generics` SHALL be an empty `Vec`

### Requirement: Parse VHDL ports
The parser SHALL extract all port declarations from the entity and populate `ModuleDef.ports` as `Vec<PortDef>`.

#### Scenario: Ports with in/out/inout directions
- **WHEN** an entity declares `port (clk : in std_logic; data : out std_logic_vector(7 downto 0); sda : inout std_logic)`
- **THEN** `ModuleDef.ports` SHALL contain three entries with directions `In`, `Out`, `InOut` respectively

#### Scenario: Port ordering preserved
- **WHEN** an entity declares ports `a`, `b`, `c` in that order
- **THEN** `ModuleDef.ports` SHALL list them in the same order

### Requirement: Store source path and hash
The parser SHALL populate `ModuleDef.source_path` with the file path and `ModuleDef.source_hash` with a content-based hash of the file bytes.

#### Scenario: Hash changes on file edit
- **WHEN** a file is parsed, then its content changes, then it is parsed again
- **THEN** `source_hash` SHALL differ between the two parses

#### Scenario: Hash stable on unchanged file
- **WHEN** a file is parsed twice without modification
- **THEN** `source_hash` SHALL be identical both times

### Requirement: Report errors for unparseable VHDL
The parser SHALL return an `Err` with a descriptive message when a `.vhd` file cannot be parsed, without panicking.

#### Scenario: Syntax error in entity
- **WHEN** a `.vhd` file contains `entity broken is port (clk : in std_logic` (missing closing)
- **THEN** `parse_file` SHALL return `Err` with a message indicating the parse failure

#### Scenario: Empty file
- **WHEN** a `.vhd` file is empty
- **THEN** `parse_file` SHALL return `Ok` with an empty `Vec` (no entities found)
