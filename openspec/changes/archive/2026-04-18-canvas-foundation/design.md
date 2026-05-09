## Context

qt-gui change shipped Sections 1–4: Qt toolchain, app shell, cxx-qt bridge (AppState QObject), sidebar. The sidebar lists instances and a module library, with drag-to-canvas already wired (drops call `add_instance`). But the canvas is still a placeholder `QGraphicsScene` with no content — drops succeed at the model layer but the user sees nothing.

This design lays the canvas foundation that closes that gap. It is the first of three canvas changes; port pins (`canvas-port-pins`) and wires (`canvas-wires`) follow.

Relevant constraints from the existing codebase:
- cxx-qt AppState QObject owns all schematic state. C++ reads via invokables; writes go through invokables that emit change signals.
- `Instance.position: (f32, f32)` already exists on the Rust side and is persisted in `.hdlc`.
- C++ side uses Qt 6.11.0, Fusion style with a dark material QPalette.

## Goals / Non-Goals

**Goals:**
- Every instance in the current Schematic is drawn as a rectangle on the canvas, labeled with its name and module.
- Dragging an instance moves it smoothly and persists the new position on drop.
- Selecting an instance on the canvas highlights it and syncs the sidebar tree selection.
- Pan (middle-click drag, two-finger scroll) and zoom (Ctrl+scroll, cursor-centered) behave like every other diagramming tool.
- Dirty instances are visually distinct (red outline).

**Non-Goals:**
- Port pins on instance boxes — that is `canvas-port-pins`.
- Wires between instances — that is `canvas-wires`.
- Mini-editor update on selection — that is qt-gui Section 6.
- Rubber-band multi-select, alignment guides, snap-to-grid, or auto-layout — v2 candidates, not scoped here.

## Decisions

### 1. QGraphicsScene with a custom QGraphicsRectItem subclass

Use `QGraphicsScene` + `QGraphicsView` directly and subclass `QGraphicsRectItem` as `InstanceItem`. QGraphicsScene has battle-tested hit-testing, z-ordering, dragging, and rubber-band infrastructure — all the machinery we would otherwise reinvent.

**Alternative rejected:** rendering everything via a single `QWidget::paintEvent` — loses hit-testing and selection for free. `QQuickItem` / QML — heavier dependency, harder to integrate with the existing QWidget shell.

### 2. Scene coordinates in Schematic-native units

Use scene coordinates that correspond 1:1 with `Instance.position`. `InstanceItem::setPos(x, y)` directly reflects the `f32` pair. `QGraphicsView`'s transform handles zoom; scene coordinates stay unchanged by zoom/pan.

Rationale: any unit conversion is a bug magnet and a constant source of off-by-pixel errors. Scene coordinates == model coordinates is simpler and costs nothing.

### 3. Position persistence on drop, not during drag

Hook `InstanceItem::itemChange` for `ItemPositionHasChanged` but only call `AppState::set_instance_position` once per drag, on mouse-release. Calling it on every position change would flood the bridge, emit hundreds of `instance_moved` signals, and force re-validation on each pixel.

### 4. Selection sync is bidirectional via AppState signal

Canvas click → `AppState::set_selected_instance(name)` → `selection_changed(name)` signal → sidebar row highlight. Sidebar click also calls `set_selected_instance`. Both views subscribe to the same signal, so they stay in sync without direct canvas ↔ sidebar coupling.

Alternative rejected: direct C++ pointer coupling between tree view and canvas — creates a back-edge in the dependency graph; violates the "views talk to model, not each other" rule established in the bridge layer.

### 5. Dirty outline painted in InstanceItem::paint

Override `paint()` on `InstanceItem` to check `AppState::instance_is_dirty(index)` once per paint and apply a red `QPen` when true. No separate decoration item — keeps the visual tied to the rectangle.

Currently `instance_is_dirty` always returns false; qt-gui Section 9 wires the real dirty tracking. This change delivers the painting infrastructure; the flip to true happens downstream.

### 6. Pan/zoom via QGraphicsView built-ins

- Pan — set `QGraphicsView::setDragMode(ScrollHandDrag)` is already in place, but that hijacks left-click. Switch to handling middle-button press with a transient `ScrollHandDrag` mode, falling back to `NoDrag` on middle-release. Two-finger scroll via default `QAbstractScrollArea` behavior.
- Zoom — override `wheelEvent` on a `QGraphicsView` subclass. Detect Ctrl modifier, apply `scale(factor, factor)` with `AnchorUnderMouse` to zoom at cursor.

### 7. Introduce a `CanvasView` subclass for event handling

Required to handle middle-click pan, Ctrl+wheel zoom, and click-to-select without polluting the existing drop-filter. The subclass also accepts drops directly, letting us retire the `CanvasDropFilter` added in qt-gui Section 4.

## Risks / Trade-offs

- **[Drag + drop fighting each other]** → The drop target needs `setAcceptDrops(true)` on both the view and its viewport. Drag (pan) uses middle-button; drop uses left-button external drag. Test both flows to ensure no accidental interception.
- **[Many instances slow rendering]** → QGraphicsScene handles thousands of items with built-in indexing. Structural wrappers rarely exceed ~50 instances. Not a concern for v1.
- **[Zoom accumulates floating-point drift]** → Apply zoom as `scale(factor, factor)` repeatedly is fine; Qt's `QTransform` is double-precision. Clamp zoom between 0.2× and 5× to avoid pathological cases.
- **[Position changes flood .hdlc dirty tracking]** → Only set dirty on drag-release, not on every pixel. Already in Decision 3.
