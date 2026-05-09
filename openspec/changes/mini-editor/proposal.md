## Why

The right-pane editor is still a placeholder (`Select an instance to edit its port map...`). Its job — per the original hdl-compose vision — is to be the third live view of the schematic: text and canvas are equally authoritative, neither is canonical. Today the canvas is the only interactive surface; without the mini editor, users who prefer keyboard editing, batch changes, or copy/paste from existing VHDL have no ergonomic path. It's also where re-parse diagnostics should surface in-context (e.g. "this port was renamed upstream"), alongside the actual entries the user needs to edit.

## What Changes

- **Populate on selection**: selecting an instance renders a VHDL component-instantiation buffer (generic map + port map) in the right pane. Deselecting clears it.
- **Parse on commit**: the editor accepts edits; on focus-out (or explicit "Apply"), the buffer is parsed back to `set_generic_map_entry` + `set_port_map_entry` calls. Syntax errors are reported but don't clobber the model.
- **RHS grammar** accepts: `<identifier>` (top-port name or alias), `<instance>.<port>` (instance-output driver), `<driver>[<slice>]` (bit / range slice), `open` (unconnected).
- **Diagnostic comments**: when an instance is `dirty`, the buffer includes `-- WAS: u_adc.data_out (port removed)` lines above the affected entries so the user can see what was dropped.
- **Bidirectional sync**: canvas edits that change the selected instance's port_map re-render the buffer (unless the user is mid-edit — track focus).
- **QCompleter**: after `=>`, offer top-level port names, alias names, and `<instance>.<port>` combinations for valid drivers. Dot-triggered after `<instance>.` lists that instance's ports.
- **QSyntaxHighlighter**: width mismatch / type mismatch / unknown reference → red underline on the RHS. Connects to the existing `validate` diagnostics for the selected instance.

## Capabilities

### New Capabilities

- `mini-editor`: the text-view of a selected instance's bindings, its parser, its live sync with the model, and its completer / highlighter behavior.

### Modified Capabilities

- None. The mini editor is a new pane; it calls existing bridge invokables (`set_port_map_entry`, `set_generic_map_entry`, `port_map_entry`, `match_by_name`, etc.).

## Impact

- **Code**: `src/gui/app.cpp` — new `MiniEditor` class (or functions) that owns a `QPlainTextEdit`, `QCompleter`, and `QSyntaxHighlighter`. Connected to `AppState::selection_changed` and `AppState::port_map_changed`.
- **Bridge**: may need a small helper invokable to enumerate an instance's generics in declaration order (the existing `instance_port_count/name` already handles ports). `module_generic_count(instance_index)` + `module_generic_name(instance_index, i)` + `module_generic_default(instance_index, i)`.
- **No model changes**: everything is a view. No `.hdlc` schema impact.
- **Out of scope**: multi-instance edits, rename refactoring, snippet expansion, multi-cursor editing.
