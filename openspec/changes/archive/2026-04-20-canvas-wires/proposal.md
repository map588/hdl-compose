## Why

With `canvas-foundation` drawing boxes and `canvas-port-pins` drawing pins, the canvas shows topology visually but static — the user can see what is connected in the sidebar but not draw or rename connections on the canvas. This change completes the canvas by adding wire rendering, click-to-wire interaction, and in-place net renaming.

Click-port-click-port wiring is a shortcut for mini-editor text edits (per ARCHITECTURE.md: canvas edits emit equivalent text edits under the hood). Implementing the canvas shortcut here is orthogonal to the editor — they both call the same `set_port_map_entry` invokable.

## What Changes

- `WireItem` (QGraphicsPathItem subclass) — Manhattan-routed path between two `PortPinItem`s. One wire per driver → one-or-more loads.
- Wires re-route when endpoints move: subscribe to `instance_moved` signal from the bridge, recompute path.
- Click-port-click-port wiring — first click on an output pin "arms" the wire tool; second click on a compatible input pin creates the connection via `set_port_map_entry`.
- Invalid connection rejection — if the second click targets an incompatible pin (wrong direction, type mismatch, width mismatch), flash red briefly and show a tooltip with the reason. No connection made.
- Right-click wire → QInputDialog → set alias via new `set_alias(net_key, alias)` invokable. Alias drives the generated signal name.
- Bridge additions — enumerate current connections for rendering: `wire_count()`, `wire_source(i)`, `wire_target(i)` returning pin references (`<instance>.<port>` or `top:<name>`).

## Capabilities

### New Capabilities
- `canvas-wires`: Manhattan-routed wire rendering, live re-routing on instance move, click-port-click-port wire creation with direction/type/width validation, right-click wire rename to alias.

### Modified Capabilities
(none)

## Impact

- **src/gui/app.cpp** — `WireItem` class, wire-tool state machine for click-to-wire, right-click handler, drop-in wire re-route on `instance_moved` signal.
- **src/gui/bridge.rs** — invokables `wire_count()`, `wire_source(i) -> QString`, `wire_target(i) -> QString`, `set_alias(net_key: &QString, alias: &QString) -> bool`. `alias_changed(key)` signal emitted by `set_alias`.
- **src/schematic.rs** — no changes (set_alias / remove_alias already exist).
- **Dependencies** — requires `canvas-foundation` + `canvas-port-pins`.
