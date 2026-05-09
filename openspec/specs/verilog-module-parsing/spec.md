## ADDED Requirements

### Requirement: Parse Verilog module name
The parser SHALL extract the module name from a Verilog/SystemVerilog source file and populate `ModuleDef.name`.

#### Scenario: Simple module declaration
- **WHEN** a `.v` file contains `module counter (...); ... endmodule`
- **THEN** `ModuleDef.name` SHALL equal `"counter"`

#### Scenario: SystemVerilog file extension
- **WHEN** a `.sv` file contains a module declaration
- **THEN** the parser SHALL parse it identically to a `.v` file

### Requirement: Parse Verilog parameters
The parser SHALL extract all parameter declarations and populate `ModuleDef.generics` as `Vec<GenericDef>`.

#### Scenario: Module with parameter list
- **WHEN** a module declares `#(parameter WIDTH = 8, parameter DEPTH = 256)`
- **THEN** `ModuleDef.generics` SHALL contain two entries with names `"WIDTH"` and `"DEPTH"`

#### Scenario: Module with no parameters
- **WHEN** a module has no `#(...)` parameter clause
- **THEN** `ModuleDef.generics` SHALL be an empty `Vec`

### Requirement: Parse Verilog ports
The parser SHALL extract all port declarations and populate `ModuleDef.ports` as `Vec<PortDef>`.

#### Scenario: ANSI-style port declarations
- **WHEN** a module declares `(input wire clk, output reg [7:0] data, inout wire sda)`
- **THEN** `ModuleDef.ports` SHALL contain three entries with directions `In`, `Out`, `InOut` and appropriate types/widths

#### Scenario: Non-ANSI port declarations
- **WHEN** a module uses separate port list and direction declarations (`module foo(a, b); input a; output b;`)
- **THEN** the parser SHALL still extract both ports with correct directions

#### Scenario: Port ordering preserved
- **WHEN** a module declares ports `a`, `b`, `c` in that order
- **THEN** `ModuleDef.ports` SHALL list them in the same order

### Requirement: Store source path and hash
The parser SHALL populate `ModuleDef.source_path` and `ModuleDef.source_hash` identically to the VHDL parser.

#### Scenario: Consistent hashing across languages
- **WHEN** a `.v` file is parsed
- **THEN** `source_hash` SHALL use the same hash algorithm as VHDL files

### Requirement: Report errors for unparseable Verilog
The parser SHALL return an `Err` with a descriptive message when a `.v`/`.sv` file cannot be parsed, without panicking.

#### Scenario: Syntax error in module
- **WHEN** a `.v` file contains `module broken (input clk` (missing closing)
- **THEN** `parse_file` SHALL return `Err` with a message indicating the parse failure

#### Scenario: Empty file
- **WHEN** a `.v` file is empty
- **THEN** `parse_file` SHALL return `Ok` with an empty `Vec`
