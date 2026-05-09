## Why

The original qt-gui change put 16 canvas tasks into a single section. Shipping them together is a weeks-long monolithic patch — too big to review, too big to roll back. The canvas work splits naturally into three orthogonal concerns: the drawing surface and instance boxes (foundation), the port decorations on those boxes (pins), and the wires between them. This change delivers the foundation so later changes can build on it.

Without a canvas-foundation landing first, a user opening a .hdlc can't see their design — instances are only listed in the sidebar. Getting blocks drawn and draggable is the first GUI milestone where the tool starts looking like what it is.

## What Changes

- Replace the placeholder `QGraphicsScene`/`QGraphicsView` in `src/gui/app.cpp` with a functional block-diagram canvas.
- Add `InstanceItem` (QGraphicsRectItem subclass) — rectangle with two labels: instance name (top) and module reference (subtitle).
- Draggable instances — movement updates `Instance.position` in the Schematic via a new `set_instance_position(name, x, y)` invokable. Position persists in the `.hdlc` file.
- Clicking an instance selects it: visual highlight on the canvas, row-sync in the sidebar tree, placeholder editor hook (real editor lands in qt-gui Section 6).
- Pan — middle-click drag and two-finger scroll pan the view.
- Zoom — `Ctrl+scroll` zooms centered on the cursor.
- Dirty instances (Section 9 wires the real flag) render with a red outline. Uses the existing `instance_is_dirty(i)` stub.
- Remove this scope from qt-gui change (Section 5 tasks 5.1, 5.2, 5.5, 5.8, 5.11, 5.12, 5.15 replaced by pointers to this change).

## Capabilities

### New Capabilities
- `canvas-foundation`: block-diagram rendering surface, draggable instance items with position persistence, click-to-select, pan/zoom navigation, dirty visual state.

### Modified Capabilities
(none — this is a new presentation capability layered on the existing `qt-app-shell` and `sidebar`)

## Impact

- **src/gui/app.cpp** — remove placeholder canvas, add `InstanceItem` class, event handlers for drag/zoom/pan, selection wiring.
- **src/gui/bridge.rs** — new invokables `set_instance_position(name, x, y)`, `instance_pos_x(i) -> f64`, `instance_pos_y(i) -> f64`; new signals `instance_moved(name, x, y)`, `instance_selected(name)`.
- **src/schematic.rs** — no changes (Instance already has `position`).
- **qt-gui/tasks.md** — Section 5 tasks 5.1/5.2/5.5/5.8/5.11/5.12/5.15 struck through with a pointer to this change.
- **Dependencies** — unlocks `canvas-port-pins` (pins anchor to InstanceItem edges) and `canvas-wires` (wires need pins, which need boxes).
