## 1. Qt / cxx-qt Setup

- [x] 1.1 Install Qt6 and verify `qmake` / `cmake` available on build machine
- [x] 1.2 Add `cxx-qt`, `cxx-qt-lib`, `cxx-qt-build` to Cargo.toml
- [x] 1.3 Create `build.rs` for cxx-qt code generation
- [x] 1.4 Create minimal QApplication + QMainWindow that opens an empty window — verify the toolchain works
- [x] 1.5 Wire CLI: `hdl-compose gui` subcommand launches the Qt app, default (no subcommand) also launches GUI

## 2. Application Shell

- [~] 2.1 Create `src/gui/mod.rs` module tree (app, theme, sidebar, canvas, editor) — DEFERRED. Current structure: `src/gui/mod.rs` re-exports `bridge`; `src/gui/bridge.rs` is the cxx-qt layer; `src/gui/app.cpp` is the C++ front-end (single file, not split). Splitting is cosmetic; the monolith works and matches the deployed binary.
- [x] 2.2 Implement material dark color palette via QPalette and apply to QApplication
- [x] 2.3 Create QMainWindow with three-pane layout: QSplitter(sidebar | canvas | editor)
- [x] 2.4 Implement menu bar: File (New, Open, Save, Save As, Exit)
- [x] 2.5 Implement File → Open: QFileDialog for .hdlc, load project, populate all panes
- [x] 2.6 Implement File → Save / Save As: serialize Schematic to .hdlc
- [x] 2.7 Implement File → New: dialog for name + language, create empty schematic
- [x] 2.8 Window title: show project name + dirty indicator (*)
- [x] 2.9 Preferences dialog: external editor command (persisted to config file)

## 3. Bridge Layer (cxx-qt)

- [x] 3.1 Define Rust QObject wrapper for Schematic — expose as properties/invokables
- [x] 3.2 Expose `add_instance`, `remove_instance`, `set_port_map_entry`, `set_generic_map_entry` as invokable methods
- [x] 3.3 Expose `validate` results as a signal/property for view updates
- [x] 3.4 Expose `resolve_modules` and module library as accessible data
- [x] 3.5 Define signals for model changes (instance added/removed, port map changed, alias changed) that views connect to

## 4. Sidebar

- [x] 4.1 Create QTreeView with custom model: root = top-level, children = instances (`u_name : module_name`)
- [x] 4.2 Click instance → emit selection signal → canvas highlights, editor shows port map
- [x] 4.3 Library pane below divider: list parsed but unplaced modules
- [x] 4.4 Drag from library pane onto canvas → create instance at drop position
- [x] 4.5 Dirty instances show red dot icon
- [x] 4.6 Right-click context menu: Rename, Delete, Goto Source

## 5. Canvas

Section 5 split into three dedicated changes:

- **`canvas-foundation`** — covers 5.1, 5.2, 5.5, 5.8, 5.11, 5.12, 5.15 (QGraphicsScene/View, InstanceItem, drag-to-persist, selection, pan/zoom, dirty outline).
- **`canvas-port-pins`** — covers 5.3, 5.4, 5.13, 5.16 (PortPinItem, direction arrows, width badges, bundle fat-pins, top-level boundary connectors).
- **`canvas-wires`** — covers 5.6, 5.7, 5.9, 5.10, 5.14 (WireItem, Manhattan routing, re-route on move, click-wiring, invalid-connection feedback, right-click rename to alias).

## 6. Mini Editor

- [ ] 6.1 Create QPlainTextEdit as right pane, populate on instance selection
- [ ] 6.2 Generate buffer text from instance's generic map + port map in VHDL syntax
- [ ] 6.3 Parse buffer on text change: extract RHS values, update Schematic port/generic maps
- [ ] 6.4 RHS grammar: recognize `<identifier>`, `<instance>.<port>`, `open`, and flag unknown as error
- [ ] 6.5 QCompleter after `=>`: offer top-level ports, aliases, `<instance>.<port>` drivers
- [ ] 6.6 Dot-triggered completions: after `<instance>.`, list compatible ports of that instance
- [ ] 6.7 QSyntaxHighlighter: red underline for width mismatch, type mismatch, unknown reference
- [ ] 6.8 Bidirectional sync: model → editor regen when canvas edits change the selected instance
- [ ] 6.9 Dirty instance diagnostic comments: `-- WAS: u_adc.data_out (port removed)`

## 7. Goto Source

- [x] 7.1 Editor command persisted in QSettings via Preferences dialog (`editor_command` + `editor_in_terminal` + `default_open_dir`).
- [x] 7.2 `launch_goto_source` in `src/gui/app.cpp` uses `QProcess::startDetached`; wraps in `osascript -e 'tell application "Terminal" ...'` when `editor_in_terminal` is set (required when app is launched as a `.app` bundle — no TTY).
- [x] 7.3 Wired to sidebar context menu "Goto Source"; double-click on sidebar rows opens the source.
- [x] 7.4 Info dialog prompts the user to set an editor command if none is configured.

## 8. Match-by-Name

- [x] 8.1 `AppState::match_by_name(instance)` invokable in `src/gui/bridge.rs`. For each currently-unconnected port on the instance, looks up a top-level port with same `name + direction + port_type` and sets the port_map to `Some(NetRef::TopPort(name))`. Returns count of matches.
- [x] 8.2 Edit → "Match Ports by Name" (Ctrl+M) invokes it on `selected_instance`. Status bar shows the resulting count or a "no matches" message.
- [x] 8.3 Only compatible matches (same name + direction + type) are connected; ambiguous or partial-match ports are left unchanged.
- [x] 8.4 Never auto-runs; only on explicit user action (menu / shortcut).

## 9. Module Re-parse

- [x] 9.1 `QFileSystemWatcher` wired in `run_gui`. Watches every existing library path. Re-registers paths on `library_changed` and `project_loaded` signals so freshly-added sources are tracked.
- [x] 9.2 `AppState::reload_library` snapshots the old library, re-parses via `resolve_modules`, then calls `Schematic::apply_library_update(old, new)`.
- [x] 9.3 `apply_library_update` drops `port_map` entries for ports removed from the module OR whose direction/type changed on the new version.
- [x] 9.4 `Instance.dirty: bool` field added with `#[serde(default, skip_serializing_if)]`. Set when a re-parse drops any entry on that instance. `instance_is_dirty` invokable reads it; the canvas / sidebar already paint dirty visuals.
- [x] 9.5 Ports present on the new module but absent from `port_map` remain unconnected (`None`) — current codegen emits `open` for those. No special-casing needed.
- [x] 9.6 Canvas outlines red (`InstanceItem::paint` already honors `instance_is_dirty`); sidebar red-dot is wired. Status bar shows `Source changed: <file> — reloading`. Editor diagnostic comments deferred with section 6 mini editor.
- [x] 9.7 `check_no_dirty_instances` added to `codegen::mod`. Both `generate_vhdl` and `generate_sv` call it before emitting code; `CodegenError::DirtyInstances(Vec<String>)` reported to the CLI. Forces an explicit "I've reviewed" before the change lands in generated HDL.
- [x] 9.8 `AppState::clear_instance_dirty(name)` invokable so UI can offer an "acknowledge re-parse fallout" action. (Wiring a button for it is minor polish; defer.)

## 10. Integration and Polish

- [ ] 10.1 End-to-end test: open .hdlc → see instances on canvas → edit in mini editor → codegen → verify output — blocked on section 6.
- [x] 10.2 Keyboard shortcuts: Ctrl+S save, Ctrl+O open, Ctrl+N new, Ctrl+Shift+O add source, Ctrl+R refresh, Delete remove instance / wire. Toolbar exposes File/Open/Add/Refresh/Save buttons.
- [x] 10.3 Status bar shows validation summary (error/warning counts) via `validation_changed` signal handler.
- [~] 10.4 Zoom animation — DEFERRED. Zoom works with Ctrl+scroll; smooth animated zoom is polish.
- [~] 10.5 Window geometry persistence — DEFERRED.
- [~] 10.6 README build instructions — DEFERRED.
