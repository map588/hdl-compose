## Context

`canvas-foundation` + `canvas-port-pins` together draw boxes with pins. This change adds wires — the edges of the block diagram. It also makes the canvas *interactive* for wiring: click one pin, click another, and the model gains a connection.

ARCHITECTURE.md's thesis: canvas wiring is a shortcut that emits the equivalent text edit under the hood. Which means: canvas click-wiring just calls `AppState::set_port_map_entry(instance, port, rhs)` — the same invokable the mini-editor will call. Both views mutate the same model.

## Goals / Non-Goals

**Goals:**
- Every connection in the current port map renders as a wire on the canvas.
- Wires route Manhattan-style (horizontal + vertical segments, no diagonals).
- When an instance moves, wires touching it re-route automatically.
- Click-port-click-port creates a connection if directions/types/widths match.
- Incompatible connection attempts reject visually (red flash + tooltip) and do not mutate the model.
- Right-click a wire → QInputDialog for alias → persisted to `Schematic.aliases`.

**Non-Goals:**
- Intelligent wire routing that avoids crossings or other wires — plain Manhattan is enough for v1.
- Slice / part-select syntax on wiring (e.g. `wire[7:0] => port[15:8]`). Users needing that drop to the mini-editor.
- Bundle auto-wire prompt on instance placement — later feature.
- Wire deletion via canvas — happens via mini-editor's `=> open` or port-map clearing.

## Decisions

### 1. One WireItem per driver→load edge

Each entry in `Schematic.port_map` that is `Some(NetRef)` represents a load side of an edge. The driver side is either a top-level input port or another instance's output port. Render one `WireItem` per load — i.e. one line from driver pin to each consumer pin, even if multiple loads share a driver.

Alternative rejected: one "net" object spanning all loads with a tree geometry — prettier visually but much more routing code. Simple per-load lines are tolerable for v1.

### 2. Manhattan routing: horizontal-vertical-horizontal

Three segments: exit the driver pin horizontally, turn vertically midway, enter the load pin horizontally. For vertically-close pins, degrade to L-shape. For pins on the same y-coordinate, degrade to a single horizontal segment.

Midway turn x-coordinate: halfway between driver and load x unless that overlaps another instance's bounding rect — in which case offset by a constant. This is crude but visually acceptable for structural wrappers of <~50 instances. Better routing is explicitly out of scope.

### 3. Re-route via instance_moved signal

Subscribe the wire layer (a container QGraphicsItemGroup) to `AppState::instance_moved(name, x, y)`. On signal, walk every wire referencing that instance and recompute its path. Batched update — single `QGraphicsScene::update` at the end.

Alternative rejected: each WireItem subscribes individually — fan-out of N wires × 1 signal is cheap, but the subscribe/unsubscribe lifecycle on wire create/destroy adds complexity. One subscription in the manager is simpler.

### 4. Click-wiring state machine

`WireTool` holds a nullable `source_pin`. State transitions:

- Idle → click on output pin → `source_pin = pin`; pin glows, cursor changes to cross-hair.
- Armed → click on compatible input pin → `set_port_map_entry(target_inst, target_port, driver_rhs)`; `source_pin = null`.
- Armed → click on *incompatible* pin → red flash on target, tooltip "direction mismatch: ...", `source_pin` unchanged so user can retarget.
- Armed → click on empty canvas / Esc → cancel, `source_pin = null`.
- Armed → click on the same source pin → cancel.

Compatibility check happens purely C++-side using port metadata invokables already exposed by `canvas-port-pins`. No new bridge surface needed for the validation logic itself, only for the mutation (`set_port_map_entry`, already exists) and wire enumeration.

### 5. Wire selection + rename

Hit-testing on a `QGraphicsPathItem` is path-based, which works fine with Manhattan segments. Right-click emits `customContextMenuRequested`; a `QMenu` with "Rename..." opens a `QInputDialog` prepopulated with the current alias (or the derived signal name if no alias). Accept → `AppState::set_alias(net_key, new_alias)`.

`net_key` is the NetRef serialization already used internally (`top:<name>` or `<inst>.<port>`). Kept opaque to the user — they only see the alias text.

### 6. Wire enumeration invokables on AppState

The canvas needs to enumerate current wires to render. Add:

- `wire_count() -> i32` — total connected entries in all port_maps.
- `wire_source(i) -> QString` — driver side as `top:<name>` or `<inst>.<port>`.
- `wire_target(i) -> QString` — load side as `<inst>.<port>`.

These iterate over `Schematic.instances[].port_map` flattening all `Some(NetRef)` entries. Consistent ordering by `(instance_index, port_name)`.

Alternative considered: returning a struct/array per wire — cxx-qt works cleanly with primitive returns and loops, so that's the path of least friction.

## Risks / Trade-offs

- **[Manhattan routing creates ugly overlaps for dense designs]** → Accepted for v1. A scrollable canvas and zoom-out mitigate. Better routing (e.g. A* or orthogonal edge routing) is an obvious follow-up but tangential to correctness.
- **[Click-wiring misfires when user drags instead of clicks]** → Distinguish click from drag by mouse-press/release distance threshold (~5 px). Standard Qt idiom.
- **[Alias conflicts (two nets with same alias)]** → `set_alias` in `Schematic` currently overwrites silently per driver. Validation ideally flags duplicate aliases but that is a `Schematic::validate` concern, not this change's scope. Add a diagnostic if it comes up during testing.
- **[Wire enumeration cost on every repaint]** → Maintain a cached `Vec<(NetRef, NetRef)>` rebuilt on mutation signals (`port_map_changed`). O(n) rebuild, O(1) render. Add caching only if profiling shows it's a hotspot.
