# gui

## Overview

`gui` is the Qt 6 desktop application. It has two parts in one process:

- **Rust side.** `bridge.rs` defines one QObject, `AppState`, with cxx-qt. It
  holds the design state and gives the C++ side its methods and signals.
  `editor.rs` is the grammar of the instance editor buffer, as pure
  functions. `mod.rs` starts the application.
- **C++ side.** `app.cpp` is the main window, the dialogs and the theme.
  `canvas.h` / `canvas.cpp` are the canvas. `items.h` / `items.cpp` are the
  items of the scene. `floorplan.h` is the Chip View.
  `canvas_constants.h` holds the layout constants.
- **Tests of the C++ side.** `gui_tests.h` / `gui_tests.cpp` run the real
  window with Qt Test. They are in the build only with the cargo feature
  `gui-tests`.

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
- `gui::run_tests() -> i32` (`mod.rs`, feature `gui-tests`): runs the tests
  of the C++ side and returns the number of tests that failed. It calls
  the C function `run_gui_tests`, which `gui_tests.cpp` defines.
  `tests/gui_offscreen.rs` is its only caller.

`AppState` (`bridge.rs`), the QObject that C++ uses:

- **Properties:** `project_name`, `dirty`.
- **Signals:** `project_loaded`, `project_saved`, `instance_added`,
  `instance_removed`, `instance_moved`, `port_map_changed`,
  `port_map_changed_bulk`, `alias_changed`, `validation_changed`,
  `selection_changed`, `library_changed`, `groups_changed`,
  `history_changed`, `modules_changed`, `designs_changed`.
- **Batch:** `begin_batch`, `end_batch`.
- **Tests:** `new_app_state` makes an `AppState` in Rust with no window.
  The application makes its `AppState` in C++.
- **Design:** `new_project_at`, `open_project`, `save_project`,
  `save_project_as`, `generate_code`, `suggest_codegen_path`,
  `project_language`, `has_project`, `current_project_path`, `last_error`,
  `project_board`, `set_project_board`.
- **Recovery file:** `autosave`, `recovery_path`, `recover`,
  `discard_recovery`.
- **View:** `refresh_canvas`.
- **Designs of the folder:** `design_count`, `design_path`, `design_module`,
  `design_note`.
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
  `remove_top_port`, `match_by_name`, `stale_connection_count`,
  `remove_stale_connections`, `set_alias`, `net_alias`,
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
  `wire_target`, `wire_width`, `wire_net`), the validation read methods
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
- `gui_tests.h`: `GuiTests`, one private slot for each test.

## Invariants

- The design state is in Rust: `AppStateRust` holds the schematic, the
  library, the diagnostics, the wire list, the undo stacks and the parse
  cache [`bridge.rs:AppStateRust`].
- A pin key is `<inst>.<port>` and a top-level port key is `top:<name>`.
  Each can have a `[h]` or `[h:l]` suffix. On the C++ side, `NetKey` is the
  only code that makes and parses these strings [`items.h:NetKey`]. On the
  Rust side the wire list uses the same form [`bridge.rs:AppStateRust`,
  field `wires`].
- Each method that changes the schematic calls `push_snapshot` before the
  change. The snapshot is pending first. It becomes an undo step, and
  clears the redo stack, only when the model is different after the call;
  a call that changes nothing leaves the history as it is. `undo`, `redo`,
  `can_undo` and `can_redo` decide that before they read the history
  [`bridge.rs:push_snapshot`, `bridge.rs:settle_snapshot`,
  `bridge.rs:pending_changed`].
- `AppState` sends `history_changed` when a method takes a snapshot, in
  `undo` and `redo`, and when the stacks are cleared. The history is final
  only when the method returns, so the window uses a queued connection
  [`bridge.rs:push_snapshot`, `app.cpp:MainWindow` constructor].
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
- `rebuild_library_and_validate` is the one function that brings a library
  in. It compares the ports of each module with the library of its last
  call; for a changed port it removes the entries and sets the `dirty` flag
  of the instance. `open_project` and `recover` clear the library first,
  so a design is not compared with the library of another design
  [`bridge.rs:rebuild_library_and_validate`, `bridge.rs:open_project`].
- `reload_library` takes a snapshot only when the comparison changes an
  instance [`bridge.rs:reload_library`].
- An open has no earlier library, so a connection to a port that a module
  no longer has stays in the model. `remove_stale_connections` removes each
  such connection of one instance as one undo step, and
  `stale_connection_count` is the number that it removes
  [`bridge.rs:remove_stale_connections`,
  `schematic.rs:Schematic::remove_stale_connections`,
  `items.cpp:InstanceItem::contextMenuEvent`].
- `AppState` reads the designs of the folder of the open design at an open,
  after a save with a new name, after a good `generate_code`, and at a
  reload of the library. It sends `designs_changed` only when the list is
  different [`bridge.rs:refresh_designs`].
- `design_note` is empty for a design that the open design can place.
  `add_instance` with the module of such a design puts the files of that
  design into `library_paths`, in the undo step of the instance, and
  builds the library again before it sends `instance_added`. The open
  design is not a module for itself [`bridge.rs:design_sources`,
  `bridge.rs:add_instance`, `designs.rs:FolderDesign::sources`].
- The name of a design is the name of its file. `new_project_at` writes
  the file and then opens it, so the open design stays when the new one is
  refused; it does not replace a file. `save_project_as` with a new file
  name changes `top_name` and makes the history empty, because each
  snapshot holds the old name [`bridge.rs:new_project_at`,
  `bridge.rs:save_project_as`, `bridge.rs:design_name_for`].
- When the ports or the generics of a module that an instance uses are
  different from those of the last library, `AppState` sends
  `modules_changed`. The window answers with `refresh_canvas`, through a
  queued connection. `refresh_canvas` sends `groups_changed`, which builds
  the canvas again in place, only when no full rebuild came after the
  signal [`bridge.rs:rebuild_library_and_validate`,
  `bridge.rs:refresh_canvas`, `app.cpp:MainWindow` constructor].
- Only a save of the user changes the design file. `autosave` writes the
  recovery file, `.<design file name>.autosave` next to the design, only
  when the design has changes that are not saved and the file does not
  have the model yet. A save and `discard_recovery` remove the file. Thus
  `recovery_path` gives a file after an open only when the program stopped
  with changes that were not saved [`bridge.rs:autosave`,
  `bridge.rs:save_to`, `bridge.rs:discard_recovery`,
  `bridge.rs:recovery_path`].
- `recover` replaces the model with the content of the recovery file, sets
  `dirty` and clears the two undo stacks. The file stays until the next
  save or discard [`bridge.rs:recover`].
- The window starts a timer of 12 seconds on `history_changed`, when the
  timer does not run, and calls `autosave` when the timer ends. Many
  changes in that time give one write [`app.cpp:kAutosaveDelayMs`,
  `app.cpp:MainWindow` constructor].
- The tests of the C++ side run only on the Qt `offscreen` platform. The
  entry point sets the platform itself and stops when Qt uses another one.
  It also puts `QSettings` into a temporary directory, so a test does not
  read or change the settings of the user [`gui_tests.cpp:run_gui_tests`].
- A wire is one `port_map` entry: its source is the pin that the entry
  references, which is not always the driver. `wire_net` gives the net of
  the wire from `nets::resolve_nets`: two wires have the same value when,
  and only when, they are on one net. The canvas uses it for the color, the
  highlight and the junction dots [`bridge.rs:net_key`,
  `bridge.rs:rebuild_wires`, `items.h:WireItem`,
  `canvas.h:CanvasLayer::setHoveredNet`].
- A net has one alias. `set_alias` removes the alias of each other pin of
  the net, and `net_alias` finds the alias on each pin of the net
  [`bridge.rs:set_alias`, `bridge.rs:net_alias`].
- The canvas refuses a connection that is an error (`connect_pins`). The
  text editor accepts each line that it can parse. Validation reports each
  connection that the canvas refuses
  [`bridge.rs:connect_pins`, `bridge.rs:commit_instance_buffer`, test
  `validation_reports_each_connection_that_the_canvas_refuses`].
- The parse cache has one entry for each library path. An entry is valid
  while the modification time of the file is the same. A path that is no
  longer in the design leaves the cache [`bridge.rs:resolve_library_cached`].
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

- `AppState::open_project` reads the design file (`project::load_project`),
  parses each library source, reads each `.hdlc` file of the folder, and
  clears the two undo stacks. The warnings of
  the loader go into the diagnostics with the prefix `load:` and stay
  until the next save [`bridge.rs:open_project`,
  `bridge.rs:rebuild_library_and_validate`, `bridge.rs:save_to`].
- `AppState::save_project` and `save_project_as` write the design file
  (`project::save_project`) and remove the recovery file
  [`bridge.rs:save_to`].
- `AppState::autosave` writes the recovery file, `AppState::recover` reads
  it, and `AppState::discard_recovery` removes it [`bridge.rs:autosave`,
  `bridge.rs:recover`, `bridge.rs:discard_recovery`].
- `MainWindow` asks about changes that are not saved before it closes,
  opens or makes a design. Discard removes the recovery file. After an
  open, `MainWindow` offers a recovery file that is on the disk
  [`app.cpp:MainWindow::maybeSaveDirty`, `app.cpp:MainWindow::offerRecovery`].
- `AppState::generate_code` writes the HDL files, and replaces only a file
  that hdlc generated [`bridge.rs:generate_code`].
- `AppState::new_project_at` writes a new `.hdlc` file
  [`bridge.rs:new_project_at`].
- File > New with a design open writes into the folder of that design and
  opens no file dialog. A click in the list of designs opens that design
  through the same path as File > Open [`app.cpp:MainWindow::onNewProject`,
  `app.cpp:MainWindow` constructor].
- `AppState::set_project_board` writes `<design>.toolchain.json`
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
  tests in `bridge.rs` also make an `AppState` with `new_app_state`; a
  signal handler in the test does the work of `CanvasLayer::settleAll`.
- The property test `history_follows_the_model` in `bridge.rs` runs random
  sequences of 26 kinds of edit, with batches, undo and redo. It compares
  the history with the model after each call, and it calls the reads of
  the canvas and the code generation on each final model.
  `PROPTEST_CASES=4000 cargo test --lib history_follows_the_model` runs
  more sequences.
- Run with the tests of the C++ side: `cargo test --features gui-tests`
  (`make test`). `tests/gui_offscreen.rs` makes the real `MainWindow` on
  the Qt `offscreen` platform; no window comes on the screen. `GuiTests`
  sends clicks, drags and keys with Qt Test and answers each menu and
  message box from a timer. A run stops after 120 seconds at most.
- The 17 tests of `GuiTests` cover: a wire from clicks on two pins, undo
  and redo from the menu, the drag of a block, the Delete key, the lock
  entry of the context menu, the color and the highlight of a net, the
  pins of a changed source, the recovery file, the save key, the questions
  on close and on open, a new design in the folder of the open design, a
  click in the list of designs, the placement of a design from the library
  list, and the removal of stale connections from the context menu.
- The text editors, the library list, the groups, the tool commands and
  the Chip View have no test. Their code is compiled and checked by
  reading.
