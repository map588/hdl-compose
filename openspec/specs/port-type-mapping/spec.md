## ADDED Requirements

### Requirement: Map std_logic and single-bit types
The parser SHALL map VHDL `std_logic` and Verilog single-bit `wire`/`reg` to `PortType::StdLogic`.

#### Scenario: VHDL std_logic port
- **WHEN** a VHDL port is declared as `clk : in std_logic`
- **THEN** `PortDef.port_type` SHALL be `StdLogic`

#### Scenario: Verilog single-bit wire
- **WHEN** a Verilog port is declared as `input wire clk`
- **THEN** `PortDef.port_type` SHALL be `StdLogic`

### Requirement: Map vector types with ranges
The parser SHALL map VHDL `std_logic_vector(N downto M)` and Verilog `[N:M]` to `PortType::StdLogicVector(Range)`.

#### Scenario: VHDL std_logic_vector
- **WHEN** a VHDL port is declared as `data : out std_logic_vector(7 downto 0)`
- **THEN** `PortDef.port_type` SHALL be `StdLogicVector` with range high=7, low=0

#### Scenario: Verilog multi-bit port
- **WHEN** a Verilog port is declared as `output reg [15:0] data`
- **THEN** `PortDef.port_type` SHALL be `StdLogicVector` with range high=15, low=0

#### Scenario: Parameterized width
- **WHEN** a port width depends on a generic/parameter (e.g., `std_logic_vector(WIDTH-1 downto 0)`)
- **THEN** `PortDef.port_type` SHALL be `StdLogicVector` with the range expression preserved (not evaluated)

### Requirement: Map record and struct types
The parser SHALL map VHDL record types and SystemVerilog struct types to `PortType::Record(String)` with the type name.

#### Scenario: VHDL record type port
- **WHEN** a VHDL port is declared as `cfg : in config_record_t`
- **THEN** `PortDef.port_type` SHALL be `Record("config_record_t")`

### Requirement: Map unrecognized types to Other
The parser SHALL map any type that does not fit the above categories to `PortType::Other(String)` with the raw type text.

#### Scenario: VHDL integer port
- **WHEN** a VHDL port is declared as `count : out integer range 0 to 255`
- **THEN** `PortDef.port_type` SHALL be `Other("integer range 0 to 255")`

#### Scenario: Unknown Verilog type
- **WHEN** a Verilog port uses a user-defined type not recognized as a struct
- **THEN** `PortDef.port_type` SHALL be `Other` with the raw type string

### Requirement: Map direction consistently
The parser SHALL map port directions to `Direction::In`, `Direction::Out`, or `Direction::InOut` for both languages.

#### Scenario: VHDL direction keywords
- **WHEN** a VHDL port uses `in`, `out`, or `inout`
- **THEN** `PortDef.direction` SHALL be the corresponding `Direction` variant

#### Scenario: Verilog direction keywords
- **WHEN** a Verilog port uses `input`, `output`, or `inout`
- **THEN** `PortDef.direction` SHALL be the corresponding `Direction` variant
