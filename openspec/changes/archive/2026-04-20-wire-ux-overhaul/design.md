## Context

Wiring is the single most-used interaction in hdl-compose. The first wire implementation (in-flight `canvas-wires` change) landed a click-to-wire state machine in `WireTool`, Manhattan routing in `WireItem`, right-click alias rename, and live reroute on drag. Manual-verify exposed that the interaction model does not match user expectations:

- Only click-click is supported — no drag-to-wire.
- Armed pin has no visual feedback.
- Wires cannot be selected or deleted.
- Rubber-band selection does not exist.
- Pin hit region is `boundingRect()`-shaped → accidental wire arming on misclicks.
- `port_type_width()` conflates scalar and unresolved-width vector, so `std_logic_vector(WIDTH-1 downto 0) → std_logic` is silently accepted.
- Removing an instance leaves dangling `InstancePort` values in sibling `port_map`s; re-instantiating with the same name resurrects phantom connections.
- Bit-slicing (`din[3:0] => slv[7:4]`) has no way to be expressed through the UI — users can wire whole ports only.

The underlying model (`Schematic`, `NetRef`, `ModuleDef`) and codegen are sound. The view/controller layer needs to be rebuilt around a cleaner selection and wire-commit state machine, and the `NetRef` enum needs one new variant to carry slice information.

## Goals / Non-Goals

**Goals:**

- Wire selection + deletion with Delete/Backspace.
- Armed-port visual state so users can see which pin is the in-flight source.
- Drag-to-wire as the primary path; click-click retained.
- Esc cancels wire commit and clears selection.
- Rubber-band selection on empty-canvas drag.
- Selection highlight for instances and wires.
- Reject type-mismatched wires even when width is unresolved (generic-sized vectors).
- Right-click → "Connect slice..." dialog for bit-slice associations, round-tripped through `.hdlc` save/load and codegen.
- Wipe stale port_map references when an instance is removed.
- One coherent change — no piecemeal patches.

**Non-Goals:**

- Undo/redo (v1 accepts).
- Evaluating generic expressions to concrete widths for full compatibility checks. Mismatch detection here takes the conservative line: unknown-width vs scalar is a mismatch; unknown-width vs unknown-width passes through (best we can do without generic eval).
- Multi-wire routing optimization (crossings, minimization) — existing Manhattan routing is fine.
- Moving wires by dragging their midpoints.

## Decisions

### 1. State machine: selection vs wiring

**Decision:** The canvas has two modes expressed implicitly through `QGraphicsView::DragMode`:

- **Default mode**: `RubberBandDrag`. Mouse-down on empty canvas begins a rubber-band selection. Mouse-down on an `InstanceItem` begins a drag-move (unchanged). Mouse-down on a `PortPinItem` begins a drag-wire.
- **Wiring mode**: transient; entered by mouse-down on a pin. `WireTool` tracks the provisional wire as the cursor moves. Mouse-up decides commit vs cancel based on what's under the cursor.

Click-click wiring still works: a short mouse-down-then-up on a pin without dragging keeps the pin armed. A subsequent click on a compatible target pin commits.

**Alternative considered:** explicit "wire tool" and "select tool" toggles in the toolbar. Rejected — adds UI complexity and doesn't match user expectations from Vivado/Quartus style tools where wiring is modeless.

### 2. Wire selection and deletion

**Decision:** `WireItem` sets `QGraphicsItem::ItemIsSelectable`. The shape used for hit-testing is a thick-stroked path (≥8 px) so users can actually click the cosmetic-pen wire. Selection paints the stroke in a high-contrast color. Delete/Backspace handled by `CanvasView::keyPressEvent`:

1. Collect selected `WireItem`s.
2. For each: call a new `AppState::clear_port_map_entry(instance_name, port_name)` invokable that sets the load side's `port_map[port]` to `None`.
3. If that entry's driver was aliased and no other port_map entry references the same driver, remove the alias too.

**Alternative considered:** right-click → "Delete wire" menu item. Rejected as the only option — users will hit Delete/Backspace first. Menu item can still be offered alongside.

### 3. Armed-port visual

**Decision:** `WireTool` emits `armed_pin_changed(pin)` when arming state changes. `PortPinItem::paint()` checks `m_armed_state` and draws an additional outline glow when true. No new signals across the cxx bridge — this is pure Qt.

### 4. Drag-to-wire

**Decision:** `PortPinItem::mousePressEvent` arms and creates a provisional `QGraphicsPathItem` owned by `WireTool`. `CanvasView::mouseMoveEvent` updates the provisional path endpoint to the cursor's scene pos. `mouseReleaseEvent`:

- If cursor is over a `PortPinItem` that passes `compatibilityError`, commit via `set_port_map_entry`.
- Else cancel and discard provisional path.

Provisional path renders in a muted color so it's clearly distinct from committed wires.

### 5. Esc cancel

**Decision:** `CanvasView` reimplements `keyPressEvent` and on `Qt::Key_Escape`: clears scene selection, then calls `m_wire_tool->cancel()`. This single place handles the two "cancel" semantics.

### 6. Pin hit region

**Decision:** `PortPinItem::shape()` returns a `QPainterPath` of the pin-tip triangle only (≤12 × 12 px). The label and row background are NOT part of `shape()`. Clicks on the row text fall through to `InstanceItem` for drag/select. `boundingRect()` remains the full row for paint coverage, but no longer gates hit-testing.

### 7. Type-mismatch detection

**Decision:** `port_type_width()` in `bridge.rs` changes:

```rust
match t {
    PortType::StdLogic => 0,                                 // scalar
    PortType::StdLogicVector(Range { high: Literal(h), low: Literal(l), .. }) =>
        ((h - l).abs() as i32 + 1).max(1),                  // resolved vector, N ≥ 1
    PortType::StdLogicVector(_) => -1,                       // unresolved-width vector
    _ => 0,
}
```

`WireTool::compatibilityError` then rejects `0 ↔ -1` (scalar vs vector) as a mismatch with tooltip `"type mismatch: scalar cannot drive vector"`. `-1 ↔ -1` (two unresolved vectors) passes — we can't prove mismatch without generic eval, and blocking this would prevent legitimate wiring of parameterized modules.

### 8. Bit-slice connections

**Decision:** Extend `NetRef` with two variants:

```rust
pub enum NetRef {
    TopPort(String),
    InstancePort(String, String),
    // NEW:
    InstancePortSlice(String, String, SliceExpr),
    TopPortSlice(String, SliceExpr),
}

pub enum SliceExpr {
    Bit(i32),                   // e.g. port[3]
    Range { high: i32, low: i32 }, // e.g. port[7:4]
}
```

Right-click on a pin or wire shows a "Connect slice..." dialog that collects the slice spec. The dialog is only available when the target port is multi-bit. `set_port_map_entry_slice(load_inst, load_port, driver_inst, driver_port, slice)` is the new bridge invokable. Codegen emits `driver_port(3)` / `driver_port(7 downto 4)` for VHDL, `driver_port[3]` / `driver_port[7:4]` for SV.

**Alternative considered:** free-form text field for RHS expression. Rejected — error-prone, inconsistent with the structured model.

### 9. Stale-ref cleanup on delete

**Decision:** `Schematic::remove_instance(name)` does the removal, then iterates all other instances' `port_map`s. For each entry whose value is `Some(NetRef::InstancePort(inst, _))` or the slice variant and `inst == name`, it sets the value to `None`. Aliases whose key referenced the removed instance are also dropped.

Bridge adds a signal `port_map_changed_bulk()` fired once after the sweep so the canvas re-renders everything at once.

### 10. Bridge invokables added

- `AppState::clear_port_map_entry(instance_name, port_name)` — for wire deletion.
- `AppState::set_port_map_entry_slice(load_inst, load_port, driver_inst, driver_port, slice_high, slice_low)` — for slice connect. `slice_high == slice_low` means single-bit.
- `AppState::remove_instance_and_sweep(name)` — supersedes current `remove_instance` if one exists; cleans stale refs.
- Signal `port_map_changed_bulk()`.

## Risks / Trade-offs

- **[Risk]** Drag-to-wire coexisting with drag-to-move-instance creates ambiguity when the user mouse-downs on a pin and the drag distance is ≤5 px: is this a click-arm or an abandoned drag-wire? → **Mitigation:** drag-wire is always provisional; if mouse-up happens without having moved past 3 px, fall back to click-arm semantics. User experience: short drag = click-arm; long drag = drag-wire. Same 3–5 px threshold already used for drag-to-move-instance.

- **[Risk]** Thick hit-stroke on `WireItem` makes adjacent wires hard to distinguish when they run parallel and close. → **Mitigation:** keep visual render at cosmetic pen; only `shape()` is thick. Selection visual still renders at 1 px to avoid "fat wires". Users still see thin wires; the ≥8 px hit region is invisible.

- **[Risk]** `.hdlc` files containing slice variants cannot be opened by older binaries. → **Mitigation:** serde's default enum behavior fails loudly on unknown variants. Version field in `.hdlc` is already `2`; we don't bump unless absolutely necessary. Slice variants are additive — old files without them still load. New saves with slice variants fail to load on pre-change binaries, which is acceptable given project is pre-v1.

- **[Risk]** Stale-ref sweep fires `port_map_changed_bulk` which rebuilds all wires. On designs with hundreds of instances, that's a spike. → **Mitigation:** pre-v1 workloads are small (< 50 instances). Revisit if/when it becomes measurable.

- **[Trade-off]** `-1 ↔ -1` (two unresolved vectors) is allowed through. In theory the two parameterized modules could have different generics resolved to different widths. We accept this false-positive bucket because without generic eval we can't prove mismatch, and the alternative (blocking all parameterized wiring) is worse than runtime reporting.

## Open Questions

- Should Esc cancel wire + deselect in one press, or in two? (Current decision: one press, matching Vivado and Quartus.)
- Should slice dialog show current resolved width when available, to help users pick valid ranges? (Defer; straightforward followup.)
- Should wire deletion also offer "disconnect from driver and move to unconnected pile" vs full delete? (Defer; single behavior for now.)
