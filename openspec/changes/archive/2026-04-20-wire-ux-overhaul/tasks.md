## 1. Schematic model: slice + stale-ref cleanup

- [x] 1.1 Add `SliceExpr { Bit(i32), Range { high: i32, low: i32 } }` in `src/types.rs` (or co-located with `NetRef`).
- [x] 1.2 Extend `NetRef` enum with `InstancePortSlice(String, String, SliceExpr)` and `TopPortSlice(String, SliceExpr)` variants.
- [x] 1.3 Update serde (re)tests so `.hdlc` round-trips all four `NetRef` variants.
- [x] 1.4 Rewrite `Schematic::remove_instance` to sweep sibling `port_map`s and drop alias entries referencing the removed instance; add unit tests.
- [x] 1.5 Extend `Schematic::validate` to (a) emit scalar-vs-unresolved-vector mismatch; (b) emit slice-out-of-range on resolved-width drivers. Add unit tests.

## 2. Bridge layer

- [x] 2.1 Fix `port_type_width` in `src/gui/bridge.rs` to return `-1` for non-literal-bound vectors.
- [x] 2.2 Add invokable `clear_port_map_entry(inst, port)`.
- [x] 2.3 Add invokable `set_port_map_entry_slice(load_inst, load_port, driver_inst, driver_port, slice_high, slice_low)` (use `slice_high == slice_low` for single bit).
- [x] 2.4 Replace/supplement `remove_instance` with `remove_instance_and_sweep(name)` that delegates to the new `Schematic::remove_instance` and fires `port_map_changed_bulk()`.
- [x] 2.5 Add signal `port_map_changed_bulk()` to the cxx-qt Q_SIGNALS.

## 3. Canvas port pins (hit region + armed state)

- [x] 3.1 Narrow `PortPinItem::shape()` to the pin-tip triangle only (≤ 12 × 12 px); keep `boundingRect()` wide for paint coverage.
- [x] 3.2 Add armed-visual state to `PortPinItem::paint()` (outline glow / accent color when `m_armed == true`).
- [x] 3.3 Wire `WireTool` to call `PortPinItem::update()` and set `m_armed` on the newly armed pin; clear on the previously armed pin.

## 4. Wire tool: drag-to-wire + commit + cancel

- [x] 4.1 Implement `PortPinItem::mousePressEvent` to enter wire-drag mode: create a provisional `QGraphicsPathItem` owned by `WireTool`, record the arm pin, set cursor shape.
- [x] 4.2 Implement `CanvasView::mouseMoveEvent` to update the provisional path endpoint to the current scene-pos while the button is held.
- [x] 4.3 Implement `CanvasView::mouseReleaseEvent` to commit (compatible target) or cancel (anywhere else) the provisional wire.
- [x] 4.4 Retain click-to-wire: if the release happens within 3 px of press and over the same pin, arm (don't drag); if a pin is already armed, a second click on a compatible pin commits.
- [x] 4.5 Update `WireTool::compatibilityError` to surface the new `-1` width semantics with tooltip `"type mismatch: scalar cannot drive vector"`; allow `-1 ↔ -1` through.
- [x] 4.6 Add `CanvasView::keyPressEvent` handling `Qt::Key_Escape`: `wire_tool->cancel()` + clear scene selection.

## 5. Wire selection and deletion

- [x] 5.1 Set `QGraphicsItem::ItemIsSelectable` on `WireItem` in its constructor.
- [x] 5.2 Override `WireItem::shape()` to return a stroked path ≥ 8 px wide for reliable hit-testing while keeping cosmetic-pen rendering.
- [x] 5.3 Render selected state in `WireItem::paint()` with a distinct selection color (honor `QStyleOptionGraphicsItem::state & QStyle::State_Selected`).
- [x] 5.4 Handle `Delete`/`Backspace` in `CanvasView::keyPressEvent`: iterate selected `WireItem`s, call `clear_port_map_entry` for each, and drop aliases when their last reference is gone.

## 6. Rubber-band and selection highlight

- [x] 6.1 Set `QGraphicsView::setDragMode(RubberBandDrag)` on `CanvasView`.
- [x] 6.2 Confirm `InstanceItem` and `WireItem` have `ItemIsSelectable` and `ItemIsFocusable` flags.
- [x] 6.3 Render the InstanceItem selected state with the selection color (honoring Shift-extend via Qt defaults).
- [x] 6.4 Verify clicks on empty canvas clear selection; clicks on pins start drag-wire (not rubber-band).

## 7. Bit-slice dialog and codegen

- [x] 7.1 `prompt_connect_slice` dialog: driver text + slice text (accepts `[7:4]`, `7:4`, `3`, etc — brackets stripped). Validation via `toInt`.
- [x] 7.2 `Connect slice...` context menu action added on `PortPinItem` when `m_width != 0` (multi-bit or unresolved vector).
- [x] 7.3 Dialog result calls `AppState::set_port_map_entry_slice(inst, port, driver_inst, driver_port, high, low)`.
- [x] 7.4 VHDL codegen emits `name(i)` / `name(h downto l)` via `render_rhs_vhdl`.
- [x] 7.5 SV codegen emits `name[i]` / `name[h:l]` via `render_rhs_sv`.
- [x] 7.6 Codegen tests `slice_variants_render_vhdl` + `slice_variants_render_sv` cover single-bit + range for both languages.

## 8. Integration and manual verification

- [x] 8.1 Ensure `AppStateRust::wire_cache` rebuilds on `port_map_changed_bulk()` as well as `port_map_changed`.
- [x] 8.2 Manually verified by user: drag-to-wire, click-to-wire, armed-glow, Esc cancel, rubber-band multi-select, Delete removes wires, instance delete + re-add leaves no phantom wires, mismatch tooltip, slice round-trip via Connect slice... dialog.
- [x] 8.3 Run `cargo test` (56/56 pass) and `openspec validate wire-ux-overhaul --strict` (valid).
- [ ] 8.4 Archive in-flight `canvas-wires` as superseded by this change once the manual tests listed above pass.
