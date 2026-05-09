## ADDED Requirements

### Requirement: Parse subcommand
The CLI SHALL provide a `parse` subcommand that accepts an HDL file path and prints extracted module definitions.

#### Scenario: Parse a VHDL file
- **WHEN** `hdl-compose parse counter.vhd` is run
- **THEN** the output SHALL list each entity found with its name, generics, and ports (name, direction, type)

#### Scenario: Parse a Verilog file
- **WHEN** `hdl-compose parse counter.v` is run
- **THEN** the output SHALL list each module found with its name, parameters, and ports

#### Scenario: Parse error
- **WHEN** `hdl-compose parse broken.vhd` is run on an unparseable file
- **THEN** the CLI SHALL print an error message to stderr and exit with code 2

#### Scenario: File not found
- **WHEN** `hdl-compose parse nonexistent.vhd` is run
- **THEN** the CLI SHALL print an error message to stderr and exit with code 2

### Requirement: New subcommand
The CLI SHALL provide a `new` subcommand that creates an empty `.hdlc` project file.

#### Scenario: Create VHDL project
- **WHEN** `hdl-compose new my_design --language vhdl` is run
- **THEN** a file `my_design.hdlc` SHALL be created with an empty schematic, language set to Vhdl, and version 2

#### Scenario: Create SystemVerilog project
- **WHEN** `hdl-compose new my_design --language sv` is run
- **THEN** a file `my_design.hdlc` SHALL be created with language set to SystemVerilog

#### Scenario: File already exists
- **WHEN** `hdl-compose new my_design --language vhdl` is run and `my_design.hdlc` already exists
- **THEN** the CLI SHALL print an error and exit with code 1 without overwriting

### Requirement: Validate subcommand
The CLI SHALL provide a `validate` subcommand that loads a `.hdlc` project, parses its library, runs validation, and prints diagnostics.

#### Scenario: Valid project
- **WHEN** `hdl-compose validate project.hdlc` is run on a valid project
- **THEN** the output SHALL print "No errors" and exit with code 0

#### Scenario: Project with errors
- **WHEN** `hdl-compose validate project.hdlc` is run on a project with validation errors
- **THEN** the output SHALL list each error/warning with instance and port context, and exit with code 1

#### Scenario: Project with warnings only
- **WHEN** a project has warnings but no errors
- **THEN** the CLI SHALL print warnings and exit with code 0

### Requirement: Codegen subcommand
The CLI SHALL provide a `codegen` subcommand that loads a project, validates it, and generates structural HDL output.

#### Scenario: Generate to stdout
- **WHEN** `hdl-compose codegen project.hdlc` is run on a valid project
- **THEN** the generated HDL SHALL be printed to stdout

#### Scenario: Generate to file
- **WHEN** `hdl-compose codegen project.hdlc -o output.vhd` is run
- **THEN** the generated HDL SHALL be written to `output.vhd`

#### Scenario: Codegen blocked by errors
- **WHEN** `hdl-compose codegen project.hdlc` is run on a project with validation errors
- **THEN** the CLI SHALL print errors to stderr and exit with code 1 without generating output

### Requirement: Inspect subcommand
The CLI SHALL provide an `inspect` subcommand that prints a summary of a project.

#### Scenario: Inspect project
- **WHEN** `hdl-compose inspect project.hdlc` is run
- **THEN** the output SHALL show: top-level name, language, number of instances, list of instances with module refs, number of library paths, and any library resolution issues

### Requirement: Verbose flag
The CLI SHALL accept a `--verbose` or `-v` global flag that increases log output detail.

#### Scenario: Verbose mode
- **WHEN** any subcommand is run with `--verbose`
- **THEN** debug-level tracing output SHALL be printed to stderr

#### Scenario: Default mode
- **WHEN** a subcommand is run without `--verbose`
- **THEN** only warnings and errors SHALL appear on stderr

### Requirement: Exit codes
The CLI SHALL use consistent exit codes: 0 for success, 1 for validation/logic errors, 2 for I/O or parse errors.

#### Scenario: Success exits 0
- **WHEN** any command completes successfully
- **THEN** the exit code SHALL be 0

#### Scenario: Validation error exits 1
- **WHEN** validate or codegen encounters validation errors
- **THEN** the exit code SHALL be 1

#### Scenario: I/O error exits 2
- **WHEN** a file cannot be read or parsed
- **THEN** the exit code SHALL be 2
