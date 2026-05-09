## 1. Bridge wire-enumeration and alias invokables

- [x] 1.1 Add invokable `wire_count() -> i32` — sum of connected port_map entries across all instances
- [x] 1.2 Add invokable `wire_source(i: i32) -> QString` — returns `top:<name>` or `<inst>.<port>` for the i-th wire's driver
- [x] 1.3 Add invokable `wire_target(i: i32) -> QString` — returns `<inst>.<port>` for the i-th wire's load side
- [x] 1.4 Internal: cache the flattened `Vec<(NetRef, NetRef)>` on AppStateRust; rebuild on `port_map_changed`/`project_loaded` signals
- [x] 1.5 Add invokable `set_alias(net_key: &QString, alias: &QString) -> bool` — parses key with `NetRef::from_key`, calls `Schematic::set_alias` or `remove_alias` (if alias is empty), emits `alias_changed(net_key)`

## 2. WireItem

- [x] 2.1 Create `WireItem` (QGraphicsPathItem subclass) holding source/target keys (as QString) and AppState pointer for rename
- [x] 2.2 Implement `routeBetween()` — Manhattan H-V-H routing between two scene points
- [x] 2.3 Implement `paint()` via QGraphicsPathItem default with a cosmetic `QPen`
- [ ] 2.4 Mid-segment offset when midline passes through an InstanceItem's bounding rect — deferred (v1 accepts occasional overlap)

## 3. Wire layer manager

- [x] 3.1 In `CanvasLayer`, add `std::vector<WireItem*>` for current wires
- [x] 3.2 Implement `rebuildWires()`: clear existing; iterate `wire_count()` + `wire_source`/`wire_target`; resolve keys via `resolveKey` to scene points
- [x] 3.3 Hook signals: `project_loaded` → rebuild; `port_map_changed` → rebuild; `instance_removed` → rebuild (removes touching wires as side effect)
- [x] 3.4 Hook `instance_moved(name, x, y)` → `rerouteWiresFor(name)` updating just the wires that touch the moved instance

## 4. Click-to-wire state machine

- [x] 4.1 Add `WireTool` class holding `AppState*` and `PortPinItem* m_armed`
- [x] 4.2 Override `PortPinItem::mousePressEvent`: call `WireTool::onPinClicked(this)` on left click
- [x] 4.3 Implement `WireTool::onPinClicked`: state machine — idle→arm, armed→same=cancel, armed→other=commit if compatible else feedback
- [x] 4.4 Visual arm indicator — pin repaints via `update()` on arm (subtle; could brighten further in polish)
- [x] 4.5 Canvas keypress: Escape clears source_pin via `WireTool::cancel`
- [ ] 4.6 Canvas empty-space click: clears source_pin — deferred (Esc handles cancel)

## 5. Compatibility check and feedback

- [x] 5.1 Implement `WireTool::compatibilityError(a, b)` — returns empty on compatible, reason string on direction/width mismatch
- [x] 5.2 On incompatible: `PortPinItem::flashRed(500ms)` using `QTimer::singleShot` that toggles `m_flash` and triggers repaint
- [x] 5.3 Show tooltip at cursor with `QToolTip::showText(QCursor::pos(), reason)`

## 6. Right-click wire rename

- [x] 6.1 `WireItem::setAcceptedMouseButtons(Qt::RightButton)` (PortPinItem uses LeftButton for wiring; right-click on wires goes to WireItem)
- [x] 6.2 Override `WireItem::contextMenuEvent` — `QMenu` with "Rename..."
- [x] 6.3 On Rename: open `QInputDialog` seeded with the net key
- [x] 6.4 On Accept non-empty: call `AppState::set_alias(net_key, text)`
- [x] 6.5 On Accept empty: same invokable with empty string — bridge treats as remove

## 7. Update qt-gui tasks.md

- [x] 7.1 Section 5 already replaced with pointer block in canvas-foundation; no further edit needed for canvas-wires

## 8. Verification

- [x] 8.1 `cargo build` clean
- [ ] 8.2 Manual: open a project with existing connections — wires render as Manhattan paths
- [ ] 8.3 Manual: drag an instance — wires re-route live
- [ ] 8.4 Manual: click output pin → click compatible input pin → wire appears; port map updated (confirm via `hdl-compose validate` CLI or mini-editor later)
- [ ] 8.5 Manual: click output → click another output → red flash + tooltip "direction mismatch"
- [ ] 8.6 Manual: right-click wire → Rename → enter alias → wire name persists in generated HDL (spot-check via `cargo run -- codegen`)
