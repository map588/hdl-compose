## 1. Bridge helpers

- [x] 1.1 `AppState::module_generic_count(instance_index)` reads the instance's module's generics length.
- [x] 1.2 `AppState::module_generic_name(instance_index, generic_index)` returns the generic name.
- [x] 1.3 `AppState::module_generic_default(instance_index, generic_index)` returns the raw default-value expression string.
- [x] 1.4 `AppState::generic_map_entry(instance, generic_name)` returns the current override or empty.

## 2. Buffer rendering

- [x] 2.1 `build_instance_buffer(AppState*, const QString &instance) -> QString` emits the VHDL component-instantiation form.
- [~] 2.2 Dirty diagnostic header — SIMPLIFIED: prints a generic 2-line comment when the instance is dirty; does NOT enumerate the specific dropped ports. Enumerating would need a new invokable that snapshots the delta at re-parse time. Deferred.
- [x] 2.3 Generic-map block rendered only when module has ≥1 generic; port-map block always.
- [x] 2.4 Monospace font + `leftJustified` alignment for the `=>` column.

## 3. Editor wiring

- [~] 3.1 Mini editor is implemented as procedural wiring on the existing `QPlainTextEdit` + helper lambdas, not a standalone class. Sufficient for current scope.
- [x] 3.2 Menlo font with monospace hint.
- [x] 3.3 On `selection_changed`: commit any pending edit against the previous instance, then repopulate for the new selection.
- [x] 3.4 On `port_map_changed` / `port_map_changed_bulk` / `project_loaded`: repopulate only when not actively editing.
- [x] 3.5 Focus-out → `commit_editor` via `QObject::eventFilter`. Also bound to `Ctrl+Return` via `QShortcut`.
- [x] 3.6 Successful clean commit on a dirty instance calls `clear_instance_dirty` implicitly.

## 4. Parser and commit

- [x] 4.1 Line-based: `extract_binding` returns `(lhs, rhs)` for `=>` lines and skips comments / section headers / close-parens.
- [x] 4.2 `parse_editor_line` validates RHS via a single regex covering identifier, `inst.port`, bracket-slice forms, and literal `open` (→ empty rhs).
- [x] 4.3 Commit is all-or-nothing: on any parse error a `QMessageBox` lists the offending lines and the model is not mutated.
- [x] 4.4 `Ctrl+Return` shortcut wired.

## 5. Completer

- [ ] 5.1 Completer — DEFERRED. Current editor is typable; completion is polish.
- [ ] 5.2 DEFERRED
- [ ] 5.3 DEFERRED
- [ ] 5.4 DEFERRED

## 6. Syntax highlighter

- [x] 6.1 `MiniEditorHighlighter` (QSyntaxHighlighter subclass) lives in `src/gui/app.cpp`. Inline `WaveUnderline` in red on the RHS of any port-map line that fails `parse_editor_line`. Replaces the modal `QMessageBox` that was shown on commit.
- [x] 6.2 Editor `textChanged` recomputes the error set every keystroke (via `parse_editor_buffer`) and pushes it to the highlighter; status bar shows live `Mini editor: N parse error(s)` while errors are present.
- [x] 6.3 `commit_editor` refuses non-clean buffers silently (status-bar `fix to commit` message, no modal) — model untouched, editor stays as-is for the user to fix.
- [x] 6.4 `selection_changed` silently discards an in-progress edit; switching instances is never blocked by parse errors.

## 7. Tests + manual verify

- [ ] 7.1 Unit test for RHS parser — DEFERRED (C++ side; not straightforward to hook into `cargo test`).
- [x] 7.2 Manual verify: select instance → buffer matches; edit `rst_n => clk` + Ctrl+Return → canvas wire from top-port `clk` to `counter_0.rst_n` rendered. Verified 2026-04-25.
- [ ] 7.3 Manual verify: dirty instance → comment header visible; commit clears flag. PENDING user (no dirty instance currently in fixture).
- [x] 7.4 `cargo test` (59 lib + 8 integration pass) + `openspec validate mini-editor --strict` (valid).
