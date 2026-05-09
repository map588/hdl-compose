## 1. Bridge additions

- [x] 1.1 Add invokable `set_instance_position(name: &QString, x: f64, y: f64) -> bool` to `bridge.rs`; update `Instance.position` and set dirty
- [x] 1.2 Add invokable `instance_pos_x(index: i32) -> f64` and `instance_pos_y(index: i32) -> f64`
- [x] 1.3 Add signal `instance_moved(name: QString, x: f64, y: f64)`; emit from `set_instance_position`
- [x] 1.4 Add invokable `set_selected_instance(name: &QString)` and `selected_instance() -> QString`; track in `AppStateRust`
- [x] 1.5 Add signal `selection_changed(name: QString)`; emit from `set_selected_instance`

## 2. CanvasView subclass

- [x] 2.1 Create `CanvasView` (QGraphicsView subclass) in `src/gui/app.cpp` — accepts drops internally (replaces the standalone `CanvasDropFilter`)
- [x] 2.2 Override `wheelEvent`: Ctrl+scroll zooms around cursor, clamped to `[0.2x, 5x]` via `setTransformationAnchor(AnchorUnderMouse)`
- [x] 2.3 Override `mousePressEvent`/`mouseReleaseEvent`: middle-button toggles `ScrollHandDrag` temporarily for pan
- [x] 2.4 Swap the existing `QGraphicsView` construction in `run_gui` for `CanvasView`; retire `CanvasDropFilter` + its `installEventFilter` call

## 3. InstanceItem

- [x] 3.1 Create `InstanceItem` (QGraphicsRectItem subclass) holding `instance_name` (QString) and `module_ref` (QString) fields
- [x] 3.2 Implement `paint()`: fill, border, two text labels — name (top, bold), `: module_ref` (subtitle, smaller)
- [x] 3.3 Set fixed box size (e.g. 160×80 placeholder); pins in later change adjust height
- [x] 3.4 Enable `ItemIsMovable` + `ItemSendsScenePositionChanges`; register for `ItemPositionHasChanged` notifications in `itemChange`
- [x] 3.5 Override `mousePressEvent` to record press position; in `mouseReleaseEvent` call `AppState::set_instance_position(name, pos.x, pos.y)` ONLY if the moved distance exceeded 5 px
- [x] 3.6 Override `mouseReleaseEvent` (when no drag) to call `AppState::set_selected_instance(name)` on click
- [x] 3.7 In `paint()`: if `AppState::instance_is_dirty(my_index)` returns true, paint border with red `QPen`; otherwise default pen
- [x] 3.8 In `paint()`: if selected (track via `selection_changed` signal hook or `isSelected()`), paint a thicker highlight border

## 4. Canvas layer manager

- [x] 4.1 Add a `CanvasLayer` helper struct in `app.cpp` holding `QGraphicsScene*`, `AppState*`, and a `QHash<QString, InstanceItem*>` for lookup by name
- [x] 4.2 Implement `rebuild_canvas()`: clears scene, iterates `AppState::instance_count()`, creates `InstanceItem` for each with `setPos(instance_pos_x(i), instance_pos_y(i))`
- [x] 4.3 Hook signals: `project_loaded` → `rebuild_canvas`; `instance_added(name)` → add one item; `instance_removed(name)` → remove one item
- [x] 4.4 Hook `selection_changed(name)`: find item by name, call `setSelected(true)`, trigger repaint; clear other selections
- [x] 4.5 Hook sidebar `tree_view::clicked` → call `AppState::set_selected_instance` (replaces the stub status-bar message)

## 5. Update qt-gui tasks.md

- [x] 5.1 In `openspec/changes/qt-gui/tasks.md`, replace tasks 5.1, 5.2, 5.5, 5.8, 5.11, 5.12, 5.15 with a note pointing to `canvas-foundation`

## 6. Verification

- [x] 6.1 `cargo build` — clean, no new warnings beyond pre-existing ranlib noise
- [x] 6.2 Manual: open a test `.hdlc`, verify every instance appears as a box at its stored position
- [x] 6.3 Manual: drag an instance, release, close-reopen the file — verify the new position persists
- [x] 6.4 Manual: click an instance on canvas → verify sidebar selection moves; click a sidebar row → verify canvas highlight moves
- [x] 6.5 Manual: Ctrl+scroll zooms at cursor; middle-drag pans; both work without fighting left-drag-to-move
