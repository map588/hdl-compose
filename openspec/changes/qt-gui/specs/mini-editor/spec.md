## ADDED Requirements

### Requirement: Per-instance VHDL-shaped buffer
When an instance is selected, the mini editor SHALL display a buffer showing the instance's generic map and port map in VHDL instantiation syntax.

#### Scenario: Editor content
- **WHEN** instance `u_fifo : fifo_sync` is selected with `DEPTH => 1024` and `clk => clk_sys`
- **THEN** the editor SHALL show:
  ```
  u_fifo : fifo_sync
    generic map (
      DEPTH => 1024
    )
    port map (
      clk   => clk_sys,
      din   => u_adc.data_out,
      dout  => open,
      full  => open,
      empty => open
    );
  ```

### Requirement: RHS grammar — three forms
The editor SHALL accept three forms on the right-hand side of `=>`:

#### Scenario: Top-level port reference
- **WHEN** the user types `clk => clk_sys` where `clk_sys` is a top-level port
- **THEN** the model SHALL update with `NetRef::TopPort("clk_sys")`

#### Scenario: Instance.port reference
- **WHEN** the user types `din => u_adc.data_out`
- **THEN** the model SHALL update with `NetRef::InstancePort("u_adc", "data_out")`

#### Scenario: Open (unconnected)
- **WHEN** the user types `full => open`
- **THEN** the model SHALL update with `None` for that port

#### Scenario: Invalid RHS
- **WHEN** the user types an unrecognized identifier on the RHS
- **THEN** a red squiggle SHALL appear under the invalid text

### Requirement: Autocomplete after =>
After the user types `=>`, the editor SHALL offer completions via QCompleter.

#### Scenario: Completions include top ports
- **WHEN** the cursor is after `=>` on an input port
- **THEN** completions SHALL include all top-level output/inout ports and existing aliases

#### Scenario: Completions include instance.port
- **WHEN** the cursor is after `=>`
- **THEN** completions SHALL include `<instance>.<port>` for all output ports from all other instances

#### Scenario: Dot-triggered port completions
- **WHEN** the user types `u_adc.`
- **THEN** completions SHALL list ports of `u_adc` filtered by direction compatibility

### Requirement: Width and type mismatch squiggles
The editor SHALL show inline diagnostic squiggles for incompatible connections.

#### Scenario: Width mismatch
- **WHEN** an 8-bit port is connected to a 16-bit driver
- **THEN** a red squiggle SHALL appear with hover text explaining the mismatch

#### Scenario: Direction mismatch
- **WHEN** an input port references another input port as its driver
- **THEN** a red squiggle SHALL appear indicating the driver is not an output

### Requirement: Bidirectional sync with model
Edits in the mini editor SHALL update the Schematic model, and model changes from the canvas SHALL regenerate the editor buffer.

#### Scenario: Editor to model
- **WHEN** the user types `din => u_adc.data_out` in the editor
- **THEN** the Schematic port map SHALL update and a wire SHALL appear on the canvas

#### Scenario: Model to editor
- **WHEN** the user creates a connection via click-click on the canvas
- **THEN** the mini editor buffer SHALL regenerate to show the new connection

### Requirement: Generic map editing
The generic map section of the editor SHALL allow editing generic values.

#### Scenario: Change generic value
- **WHEN** the user changes `DEPTH => 1024` to `DEPTH => 2048`
- **THEN** the Schematic generic map SHALL update to `("DEPTH", "2048")`
