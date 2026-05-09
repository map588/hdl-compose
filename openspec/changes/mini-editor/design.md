## Context

The canvas already does the heavy lifting for wire editing — click, drag, right-click, multi-select, delete, slice-connect, promote, group. What the canvas cannot do well is bulk editing: changing a dozen generic values on a bunch of instances, pasting a port map from an external reference, or annotating an interface with comments. The mini editor is where those flows live. It also has to handle the reverse direction: when the user edits via the canvas, the text buffer must update to match — otherwise the text view drifts out of sync and becomes a source of confusion.

Key design pressure: **avoid surprising the user**. When they're typing in the editor, the canvas's edits shouldn't overwrite their buffer. When they commit a buffer change, canvas + sidebar should reflect immediately. The existing signal-order rule (rebuild cache → fire signal) already ensures the other panes react correctly once we call the model mutators.

## Goals / Non-Goals

**Goals:**

- Text pane shows the selected instance's bindings in readable VHDL-style form.
- User edits in the pane can change port/generic maps.
- Canvas edits to the same instance update the pane, when the pane is not being actively typed into.
- Error feedback inline (red underline on the offending RHS, tooltip with the diagnostic).
- Autocomplete for RHS: top-port names, alias names, instance-output drivers.
- Support for `open`, slice syntax (`foo[0]`, `foo[7:4]`), and the four current `NetRef` variants.

**Non-Goals:**

- General-purpose text editor — this is a structured form, not a code editor.
- Undo/redo history in the mini editor (use the QPlainTextEdit default).
- Multi-instance edit (selecting multiple instances shows nothing; single-select only for v1).
- Rename refactoring (e.g. renaming a port and having all drivers follow).

## Decisions

### 1. Buffer format

**Decision:** VHDL component instantiation. Each instance renders as:

```
u_counter : counter
  generic map (
    WIDTH => 8
  )
  port map (
    clk   => clk,
    rst_n => rst_n,
    en    => en,
    count => u_counter_count
  );
```

Whitespace, trailing commas, and the blank line after `);` are cosmetic — the parser accepts any reasonable variant. Generic map section is omitted if the module has no generics.

**Alternative considered:** SystemVerilog `.name(value)` form. Rejected — VHDL form is language-neutral to the user (they can read it regardless of project language; we just parse the `<name> => <rhs>` shape). For the SV target, codegen still emits SV; the editor is an intermediate abstraction.

### 2. Parse-on-commit, not parse-on-keystroke

**Decision:** apply the buffer to the model only on focus-out, explicit Ctrl+Return, or when the selected instance changes. Every keystroke would be too chatty for the bridge (full rebuild every char) and the partial-parse errors would flicker. Focus-out means the user finishes a thought before we commit.

**Alternative considered:** debounced parse-on-keystroke (e.g. 300 ms). Good UX but noisy for the model. Defer.

### 3. Error reporting

**Decision:** parser reports a list of `(line, col, message)` diagnostics. Each invalid RHS keeps the pre-edit model value — we never clobber a good value with a broken one. Diagnostics feed the `QSyntaxHighlighter` for underlines + the status bar.

On commit: if *any* RHS has a grammatical error, block the whole commit and show a dialog listing the offending lines. Partial commits are confusing (some entries updated, others silently ignored).

### 4. Focus awareness for sync-back

**Decision:** if the editor has focus and the user is typing, `AppState::port_map_changed` signals suppress the auto-repopulate. Track a `m_user_editing` flag set on `textChanged` and cleared on focus-out. The next repopulate happens on focus-out (user done editing, apply + resync) or on selection change.

**Alternative considered:** always auto-repopulate on model changes. Rejected — steals the user's in-progress edit.

### 5. Completer scope

**Decision:** QCompleter attached to the editor with a custom `QAbstractItemModel`:

- After `=>` (and after whitespace following `=>`): completion set = top-port names + alias names + `<instance>.<port>` strings for every instance output / top-output-as-load.
- After `<identifier>.` (dot-triggered): completion set = that instance's ports only.
- Other positions: no completer popup.

Filter the combined set by current width + direction compatibility when the target port is known (we do know — we're inside a `port_map` entry, and the LHS is the target port).

### 6. Diagnostic comments for dirty instances

**Decision:** when `instance_is_dirty` is true, the buffer is prepended with:

```
-- Source file changed. Previously-connected ports that no longer exist:
--   WAS: <port_name> => <old_rhs>
```

followed by the normal instantiation. The comment lines are parsed as comments (skipped) on commit. After the user edits + commits, if no error, we call `AppState::clear_instance_dirty(name)` implicitly.

### 7. Slice-in-RHS parse

**Decision:** parse `<inst>.<port>[<slice>]` and `<top_name>[<slice>]` — slice is `<int>` or `<int>:<int>`. Map to `NetRef::InstancePortSlice` / `NetRef::TopPortSlice`. Backward compat: the existing `parse_net_rhs` handles non-slice; extend it OR add a new `parse_net_rhs_slice` for this path.

Simpler: extend `parse_net_rhs` itself so both invokables (string and slice) go through one parser. Then `set_port_map_entry` becomes a single bridge invokable and slice dialog can also use it.

## Risks / Trade-offs

- **[Risk]** Parser covers the happy path but not pathological input (nested parens, comments inside map entries). → **Mitigation:** simple line-based parser; anything that doesn't fit the `<port> => <rhs>` grammar is a diagnostic, not a crash.

- **[Risk]** QCompleter + QSyntaxHighlighter are both stateful and must not clash with the user's in-progress text. → **Mitigation:** stock Qt patterns; the `textChanged` signal handles both. Keep them in the same file so behavior stays local.

- **[Trade-off]** Parse-on-commit means typos don't show red until focus-out. Users used to live IDEs may expect sooner. Acceptable for v1; can add a 300ms debounced check later without breaking the contract.

- **[Trade-off]** No multi-instance edit means users who want to e.g. rename `clk` everywhere still use the canvas or the file. Rename refactoring is a separate feature.

## Open Questions

- Should the editor show read-only headers (`u_counter : counter`) or make the instance name editable too? v1: read-only header; renaming happens via sidebar context menu.
- Should generic map default values (when the user clears an override) show the module default as a greyed placeholder? Nice-to-have; defer.
