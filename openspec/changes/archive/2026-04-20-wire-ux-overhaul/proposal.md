## Why

The wire system is the single most-used interaction in hdl-compose — composing structural wrappers *is* wiring instances together. Manual-verify (2026-04-19) exposed that the current click-to-wire flow is unreliable and hostile: users cannot tell which port is armed, cannot select or delete wires, cannot drag to wire, have no rubber-band selection, get silent type-mismatched connections accepted, get phantom port-map entries when deleting and re-instantiating, and cannot connect individual bits of a vector. If this app is to be usable for its core purpose, these have to be fixed together — piecemeal patches keep regressing the mental model.

## What Changes

- **Wire selection + deletion**: `WireItem` becomes selectable. Delete/Backspace on a selected wire clears the load-side `port_map` entry and drops the alias if it was the last reference.
- **Armed-port visual feedback**: when `WireTool` has an armed pin, that `PortPinItem` paints with a distinct glow/outline so the user always knows which pin is active.
- **Drag-to-wire**: mouse-down on a pin starts a provisional wire that tracks the cursor; mouse-up on a compatible pin commits; release on empty space or incompatible target cancels. Click-click still works.
- **Escape cancels**: `Esc` in the canvas cancels a pending wire, clears armed state, and deselects all items.
- **Rubber-band selection**: mouse-down + drag on empty canvas starts a QGraphicsView rubber-band that multi-selects `InstanceItem`s and `WireItem`s.
- **Selection highlight**: selected instances and wires render with a distinct border/color so multi-selection is visible.
- **Tighter pin hit region**: `PortPinItem::shape()` specifies a small, explicit pin-tip rectangle; clicks outside it fall through to the `InstanceItem` so users stop wiring pins by accident when trying to drag/select.
- **Type-mismatch detection fix**: `port_type_width()` distinguishes scalar (0) from unresolved-width vector (-1). `WireTool::compatibilityError` rejects scalar↔unresolved-vector as a mismatch with an explicit tooltip. **BREAKING** to existing `.hdlc` files that were saved with phantom mismatched connections — loader will emit warnings but not drop entries.
- **Bit-slice connections**: right-click a pin or wire opens a "Connect slice..." dialog that lets the user specify slice indices (e.g. `din[3:0] => slv[7:4]`, `din => slv[0]`). `NetRef` gains a slice variant; VHDL and SV codegen emit correct sliced association.
- **Stale-reference cleanup on delete**: when an instance is removed, every other instance's `port_map` is swept; any `InstancePort` value referring to the deleted instance name is set to `None`. Fixes phantom connections after delete + re-add with same name.

## Capabilities

### New Capabilities

- `wire-interaction`: selection, deletion, drag-to-wire, armed-port feedback, rubber-band multi-select, Esc-cancel, selection highlight, slice-connect dialog. Supersedes the in-flight `canvas-wires` change with a revised behavior model.

### Modified Capabilities

- `canvas-foundation`: adds rubber-band-selection scenarios to empty-canvas mouse handling and selection-highlight rendering for `InstanceItem`.
- `canvas-port-pins`: tightens pin hit-region scenarios and adds armed-pin visual-state scenario.
- `schematic-model`: adds slice `NetRef` variant and adds stale-reference cleanup scenarios for instance deletion.

## Impact

- **Affected code**:
  - `src/gui/app.cpp` — `WireTool`, `WireItem`, `PortPinItem`, `InstanceItem`, `CanvasLayer`, `CanvasView`.
  - `src/gui/bridge.rs` — `port_type_width`, `set_port_map_entry` (slice values), new `remove_instance_and_sweep`, `set_port_map_slice` invokable.
  - `src/schematic.rs` — `NetRef` gains `InstancePortSlice` / `TopPortSlice` variants; `remove_instance` sweeps `port_map` of siblings.
  - `src/codegen/vhdl.rs`, `src/codegen/sv.rs` — emit slice associations.
  - `src/types.rs` — if slice representation lives in the type layer.
- **Project-file compatibility**: `.hdlc` v2 gains slice variant in `NetRef` serde enum. Backward-compatible read (unknown slice variants not yet present in old files). Existing saves remain valid.
- **In-flight change `canvas-wires`**: its remaining manual-verify items are subsumed by this change. Recommend archiving `canvas-wires` as superseded once `wire-ux-overhaul` lands.
- **Dependencies**: no new crates. Still Qt 6 + cxx-qt 0.8.
- **Out of scope**: undo/redo (v1 accepts), generic-value resolution for concrete vector widths (separate concern; mismatch detection here handles the unresolved case conservatively).
