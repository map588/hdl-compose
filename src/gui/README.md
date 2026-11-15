# gui

## Overview

`gui` is the Qt 6 desktop application. It has two parts in one process:

- **Rust side.** `bridge.rs` defines one QObject, `AppState`, with cxx-qt. It
  holds the project state and gives the C++ side its methods and signals.
  `editor.rs` is the grammar of the instance editor buffer, as pure
  functions. `mod.rs` starts the application.
- **C++ side.** `app.cpp` is the main window, the dialogs and the theme.
  `canvas.h` / `canvas.cpp` are the canvas. `items.h` / `items.cpp` are the
  items of the scene. `floorplan.h` is the Chip View.
  `canvas_constants.h` holds the layout constants.

The Rust side decides; the C++ side draws and takes input. Placement and
wire routing are in `crate::routing`, and `bridge.rs` only converts the
types [`bridge.rs:plan_routes_ffi`, comment "logic lives in crate::routing"].
The editor grammar is in `editor.rs`, and the `QPlainTextEdit` handling stays
in C++ [`editor.rs` module comment].

The C++ side also starts the `hdlc` binary as a subprocess for the tool
commands (Check, Synth, Build, Flash). That path does not go through
`AppState`.

## Interface

Entry:

- `gui::run(project: Option<&Path>) -> i32` (`mod.rs`): starts the Qt event
  loop. It calls the C function `run_gui`, which `app.cpp` defines.

`AppState` (`bridge.rs`), the QObject that C++ uses:

- **Properties:** `project_name`, `dirty`.
- **Signals:** `project_loaded`, `project_saved`, `instance_added`,
  `instance_removed`, `instance_moved`, `port_map_changed`,
  `port_map_changed_bulk`, `alias_changed`, `validation_changed`,
  `selection_changed`, `library_changed`, `groups_changed`.
- **Batch:** `begin_batch`, `end_batch`.
- **Tests:** `new_app_state` makes an `AppState` in Rust with no window.
  The application makes its `AppState` in C++.
- **Project:** `new_project`, `open_project`, `save_project`,
  `save_project_as`, `generate_code`, `suggest_codegen_path`,
  `project_language`, `has_project`, `current_project_path`, `last_error`,
  `project_board`, `set_project_board`.
- **Top-level buffer:** `top_level_buffer`, `commit_top_level_buffer`. Its
  grammar is in `bridge.rs` (crate only: `parse_top_level_buffer`,
  `ParsedTopLevel`, `resolve_top_port_name`).
- **Instances:** `add_instance`, `rename_instance`, `remove_instance`,
  `set_instance_position`, `set_selected_instance`, `selected_instance`,
  `instance_locked`, `set_instance_locked`, and read methods
  (`instance_count`, `instance_name`, `instance_module`, `instance_index`,
  `instance_pos_x`, `instance_pos_y`, `instance_is_dirty`,
  `instance_is_dirty_name`, `instance_source_path`,
  `instance_dependency_count`, `instance_dependency_name`,
  `instance_dependency_present`).
- **Instance editor buffer:** `instance_buffer`, `check_instance_buffer`,
  `commit_instance_buffer`, `editor_rhs_candidates`,
  `editor_port_candidates`.
- **Connections:** `connect_pins`, `set_port_map_entry`,
  `set_port_map_entry_slice`, `clear_port_map_entry`, `set_consumer_slice`,
  `clear_consumer_slice`, `consumer_slice`, `promote_port_to_top`,
  `remove_top_port`, `match_by_name`, `set_alias`, `net_alias`,
  `port_map_entry`, `set_generic_map_entry`, `generic_map_entry`,
  `module_generic_count`, `module_generic_name`, `module_generic_default`,
  and the manual bundle methods (`create_manual_bundle`,
  `remove_manual_bundle`, `manual_bundle_count`, `manual_bundle_name`,
  `manual_bundle_port_count`, `manual_bundle_port_name`).
- **Library:** `add_library_path`, `remove_library_path`, `reload_library`,
  `clear_instance_dirty`, and read methods (`library_module_count`,
  `library_module_name`, `library_path_count`, `library_path`).
- **Undo:** `undo`, `redo`, `can_undo`, `can_redo`.
- **Groups:** `create_group`, `remove_group`, `set_group_collapsed`,
  `is_group_block`, `instance_group`, `group_count`, `group_name`,
  `group_expanded_visible`, `group_hull_items`.
- **Data for the canvas:** the port, top-port and wire read methods
  (`instance_port_*`, `top_port_*`, `wire_count`, `wire_source`,
  `wire_target`, `wire_width`), the validation read methods
  (`validation_error_count`, `validation_warning_count`,
  `validation_messages`, `pin_issue_level`, `pin_issue_message`), and the pin
  notes (`pin_exposed_note`, `pin_origin_note`, `pin_direction_code`).

Free functions for C++ (`bridge.rs`). Their shared structs are `FfiNet`,
`FfiEndpoint`, `FfiRect`, `FfiPoint`, `FfiWire`, `FfiDot`, `FfiRouteParams`,
`FfiRouteResult`, `FfiColItem`, `FfiColMove`, `FfiPlaceNode`,
`FfiPlaceEdge`, `FfiPlaceResult`, `FfiCompletionContext`. Two more structs
are results of `AppState` methods: `FfiConnectResult`, `FfiEditorIssues`.

- `plan_routes_ffi`: wire routes for the nets of the canvas.
- `legalize_columns_ffi`: positions with no overlap in each column.
- `optimize_positions_ffi`: positions for the "Optimize" command.
- `completion_context_ffi`: the completion context at the cursor.

`editor.rs` (pure functions):

- `render_instance_buffer`, `parse_instance_buffer`, `ParsedBuffer`
- `is_constant_literal`, `split_consumer_slice`
- `completion_context`, `CompletionCtx`

C++ classes:

- `app.cpp`: `MainWindow`, `LibraryView`, `MiniEditorHighlighter`,
  `TabAcceptFilter`, `FocusOutFilter`, `FileOpenFilter`.
- `canvas.h`: `WireTool`, `CanvasView`, `CanvasLayer`.
- `items.h`: `NetKey`, `InstanceItem`, `PortPinItem`, `BundlePinItem`,
  `TopPortItem`, `WireItem`, `JunctionDotItem`, `GroupHullItem`.
- `floorplan.h` (namespace `chipview`): the Chip View.

## Invariants

- The project state is in Rust: `AppStateRust` holds the schematic, the
  library, the diagnostics, the wire list, the undo stacks and the parse
  cache [`bridge.rs:AppStateRust`].
- A pin key is `<inst>.<port>` and a top-level port key is `top:<name>`.
  Each can have a `[h]` or `[h:l]` suffix. On the C++ side, `NetKey` is the
  only code that makes and parses these strings [`items.h:NetKey`]. On the
  Rust side the wire list uses the same form [`bridge.rs:AppStateRust`,
  field `wires`].
- Each method that changes the schematic calls `push_snapshot` before the
  change. A snapshot clears the redo stack [`bridge.rs:push_snapshot`].
- While Rust sends `project_loaded` or `groups_changed`, no change takes a
  snapshot. The view puts each block on the grid in the handler of these
  signals, and those moves are a part of the change that sent the signal
  [`bridge.rs:rebuild_view`, `bridge.rs:push_snapshot`].
- Each undo stack holds 100 entries at most [`bridge.rs:UNDO_STACK_LIMIT`].
- `begin_batch` and `end_batch` can nest. Only the outer pair acts. The
  first change in a batch takes the one snapshot of the batch; a batch with
  no change takes none. When a change occurred, the end of the batch runs
  one validation and sends one `port_map_changed_bulk`
  [`bridge.rs:begin_batch`, `bridge.rs:end_batch`,
  `bridge.rs:push_snapshot`].
- After a change, the wire list is made again before the signal is sent,
  because the canvas reads the wire list in its signal handler
  [`bridge.rs:finish_mutation`].
- `rebuild_library_and_validate` runs after each change of connections,
  instances, generics, library paths or groups. It replaces the library,
  the diagnostics, the pin notes and the wire list, and then sends
  `library_changed` and `validation_changed` each time
  [`bridge.rs:rebuild_library_and_validate`]. It also runs after an alias
  change [`bridge.rs:set_alias`]. It does not run after
  `set_instance_position` and `set_instance_locked`.
- The parse cache has one entry for each library path. An entry is valid
  while the modification time of the file is the same. A path that is no
  longer in the project leaves the cache [`bridge.rs:resolve_library_cached`].
- An index that the canvas passes addresses the view: the visible instances
  first, then the blocks of the collapsed groups [`bridge.rs`, comment "Read
  helpers"].
- A method that can fail returns `false` and puts the message where
  `last_error` reads it [`bridge.rs:open_project`, `bridge.rs:save_to`,
  `bridge.rs:generate_code`].
- Code generation in the GUI uses `codegen::generate_design` and
  `GeneratedDesign::write_to`, the same as the CLI
  [`bridge.rs:generate_code`].
- The functions of `editor.rs` are pure. The grammar of the top-level buffer
  is not there; it is in `bridge.rs` [`editor.rs` module comment].
- `canvas_constants.h` uses no Qt type [`canvas_constants.h` file comment].
- `gui::run` gives each argument of the process to Qt
  [`mod.rs:qt_arguments`].

## Side Effects

- `AppState::open_project` reads the project file and each library source
  (`project::load_project`), clears the two undo stacks, and discards the
  warnings of the loader [`bridge.rs:open_project`].
- `AppState::save_project` and `save_project_as` write the project file
  (`project::save_project`) [`bridge.rs:save_to`].
- `AppState::generate_code` writes the HDL files [`bridge.rs:generate_code`].
- `AppState::set_project_board` writes `<project>.toolchain.json`
  (`ToolchainConfig::set_board`) [`bridge.rs:set_project_board`].
- After each change, the bridge reads the modification time of each library
  file and parses a file that changed [`bridge.rs:resolve_library_cached`].
- The `project_loaded` and `groups_changed` handlers of `MainWindow` call
  `CanvasLayer::settleAll`, which writes the position of each block that it
  moved to the model. These writes take no undo snapshot
  [`app.cpp:MainWindow` constructor, `canvas.h:CanvasLayer::settleAll`,
  `bridge.rs:rebuild_view`].
- `app.cpp` starts processes with `QProcess`: the `hdlc` binary itself for
  the tool commands (path from `QCoreApplication::applicationFilePath`), and
  the editor command of the user for "go to source".
- `app.cpp` keeps settings with `QSettings`: `recent_projects`,
  `default_open_dir`, `editor_command`, `window/geometry`,
  `window/splitter`, `window/editor_visible`.
- `app.cpp` watches the library files with `QFileSystemWatcher` and calls
  `AppState::reload_library` when a file changes.
- `floorplan.h` reads `<top>.floorplan.json` with `QFile` and
  `QJsonDocument`.

## Verification

- Run: `cargo test`
- The tests in `bridge.rs` and `editor.rs` cover the free functions. The
  tests in `bridge.rs` also make an `AppState` with `new_app_state` and run
  its undo, redo, batch and alias methods; a signal handler in the test
  does the work of `CanvasLayer::settleAll`.
- No test runs the C++ classes. The batch calls in `app.cpp` for the
  library path commands are compiled but not run by a test.
