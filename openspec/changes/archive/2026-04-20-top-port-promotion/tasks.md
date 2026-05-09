## 1. Bridge

- [x] 1.1 Add invokable `promote_port_to_top(instance: &QString, port: &QString) -> QString` in `src/gui/bridge.rs`.
- [x] 1.2 Implementation: look up the source `PortDef`; compute a non-colliding top-port name (`<port>`, then `<port>_1` …); append to `Schematic.top_ports`; call `set_port_map_entry`; fire `project_loaded`; set dirty.
- [x] 1.3 Reuse path: if a matching top-port already exists (same direction + type + bundle), skip creation and only set port_map.

## 2. Canvas UI

- [x] 2.1 `PortPinItem::contextMenuEvent` adds `Promote to top-level port` action.
- [x] 2.2 On selection: call `AppState::promote_port_to_top`; if the returned name differs from the port name, show a `Promoted as '<name>'` message via QToolTip near the cursor (simpler than plumbing to the window's status bar from inside a QGraphicsItem).

## 3. Tests + verify

- [x] 3.1 Unit test: `resolve_top_port_name` reuse path (same name + direction + type + bundle returns existing, no create).
- [x] 3.2 Unit test: name-collision suffix path (mismatched direction → `clk_1`; walks to next free suffix when `_1`/`_2` taken).
- [x] 3.3 Manually verified: right-click pin → promotion creates top-port, all net members migrate to TopPort, wire renders without Refresh, persists across save/reload.
- [x] 3.4 Run `cargo test` (60/60 pass) and `openspec validate top-port-promotion --strict` (valid).
