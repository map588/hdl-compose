## ADDED Requirements

### Requirement: Buffer populates on selection
The mini editor SHALL render the selected instance's generic map and port map as a VHDL component-instantiation text buffer. Deselection SHALL clear the buffer.

#### Scenario: Selecting an instance populates the buffer
- **WHEN** the user selects `u_counter` in the sidebar or on the canvas
- **THEN** the mini editor shows a buffer beginning `u_counter : counter`, followed by (if applicable) a `generic map` block and a `port map` block, with one `<port> => <rhs>` line per currently-mapped port and `<port> => open` for unconnected ports

#### Scenario: Deselection clears the buffer
- **WHEN** the canvas/sidebar selection is cleared
- **THEN** the buffer becomes empty and the placeholder returns

### Requirement: RHS grammar covers concrete driver forms
The RHS of a `<port> => <rhs>` line SHALL accept exactly: a bare identifier (top-port name or alias), `<instance>.<port>`, `<driver>[<bit>]`, `<driver>[<high>:<low>]`, or `open`.

#### Scenario: Top-port RHS
- **WHEN** the user writes `clk => sys_clk`
- **THEN** commit sets the port_map entry to `Some(NetRef::TopPort("sys_clk"))`

#### Scenario: Instance-port RHS
- **WHEN** the user writes `din => u_adc.data_out`
- **THEN** commit sets the entry to `Some(NetRef::InstancePort("u_adc", "data_out"))`

#### Scenario: Bit-slice RHS
- **WHEN** the user writes `led => u_counter.count[0]`
- **THEN** commit sets the entry to `Some(NetRef::InstancePortSlice("u_counter", "count", SliceExpr::Bit(0)))`

#### Scenario: Range-slice RHS
- **WHEN** the user writes `din => bus[7:4]`
- **THEN** commit sets the entry to `Some(NetRef::TopPortSlice("bus", SliceExpr::Range { high: 7, low: 4 }))`

#### Scenario: Open RHS
- **WHEN** the user writes `full => open`
- **THEN** commit sets the entry to `None`

### Requirement: Parse on commit
The editor SHALL apply buffer edits to the model on focus-out, on explicit `Ctrl+Return`, or when the selection changes to a different instance.

#### Scenario: Focus-out commits a clean buffer
- **WHEN** the buffer has no parse errors and focus leaves the editor
- **THEN** every `<port> => <rhs>` line is applied via `set_port_map_entry` / `set_generic_map_entry`
- **AND** the canvas/sidebar/wire cache reflect the new state on the next paint

#### Scenario: Parse error blocks commit
- **WHEN** any `<rhs>` is unparseable (e.g. `count => foo(bar)`)
- **THEN** no partial commit is applied
- **AND** a dialog or status-bar message lists the error locations
- **AND** the previous model state is preserved

### Requirement: Bidirectional sync without clobbering active edits
When the model changes for the currently-selected instance, the editor SHALL re-render the buffer — UNLESS the editor has input focus and the user is mid-edit.

#### Scenario: Canvas edit while editor is not focused
- **WHEN** the editor is not focused and the canvas wires a new port on the selected instance
- **THEN** the buffer re-renders to include the new entry

#### Scenario: Canvas edit while editor is focused
- **WHEN** the editor has focus and model changes arrive
- **THEN** the buffer is NOT regenerated while the user is typing; the next regeneration happens after focus-out (and after any pending commit) or on selection change

### Requirement: Completer suggests valid drivers
A `QCompleter` attached to the editor SHALL offer RHS completions after `=>` and dot-triggered completions after `<instance>.`.

#### Scenario: Completion after =>
- **WHEN** the user types `clk =>` (and a space) and hits the completer key
- **THEN** the popup lists top-port names, alias names, and `<instance>.<port>` strings, filtered by compatibility with the LHS port when known

#### Scenario: Dot-triggered completion
- **WHEN** the user types `din => u_counter.` in RHS
- **THEN** the popup lists `u_counter`'s ports

### Requirement: Syntax highlighter shows errors inline
A `QSyntaxHighlighter` SHALL underline each RHS that fails validation — unparseable grammar, unknown reference, width mismatch, type mismatch, or slice-out-of-range — with a red wavy underline. Hovering over the underlined region SHALL show the diagnostic message.

#### Scenario: Unknown reference
- **WHEN** the user writes `clk => nonexistent_top`
- **THEN** the RHS renders with a red wavy underline
- **AND** the hover tooltip reads `unknown reference 'nonexistent_top'`

#### Scenario: Width mismatch
- **WHEN** the user writes `din => u_a.small_bus` where the driver is narrower than the load
- **THEN** the RHS renders underlined with a width-mismatch tooltip

### Requirement: Dirty instance diagnostic header
When the selected instance has `dirty == true`, the editor SHALL prepend a comment block listing the port_map entries that were dropped by the last re-parse.

#### Scenario: Dirty header present
- **WHEN** a library re-parse drops `data_out` from `u_a`, then the user selects `u_b` which had been connected to `u_a.data_out`
- **THEN** the buffer begins with
  `-- Source file changed. Previously-connected ports that no longer exist:`
  `--   WAS: din => u_a.data_out`
  `u_b : mod_b ...`

#### Scenario: Commit on a dirty instance clears the flag
- **WHEN** the user edits the dirty instance's buffer (even just acknowledging) and focus leaves
- **THEN** `AppState::clear_instance_dirty(instance)` is called once the parse succeeds
- **AND** the dirty indicator on the sidebar and canvas clears
