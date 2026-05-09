## Context

`canvas-foundation` delivers draggable `InstanceItem` rectangles on the canvas. This change decorates each rectangle with port pins — the visual terminals where wires will attach in `canvas-wires`. It also renders top-level ports on the canvas boundary (where generated VHDL entity ports appear in a structural wrapper).

Port metadata comes from the resolved module library (`AppState::library_*` invokables). For any instance, the bridge needs to expose per-port queries: name, direction, width, bundle membership.

## Goals / Non-Goals

**Goals:**
- Every instance on the canvas shows all of its module's ports as small graphical items on the box edges.
- Input ports on the left edge; output ports on the right; inout on whichever side has fewer.
- Direction arrow and multi-bit width badge visible for each pin.
- Bundle ports (AXI, APB, etc.) render as a single fat pin; clicking expands into member pins; clicking again collapses.
- Top-level ports appear as edge connectors on the scene boundary, left side for inputs and right side for outputs.

**Non-Goals:**
- Wiring between pins — that is `canvas-wires`.
- Port renaming, pin rearrangement, or hover tooltips beyond what Qt provides by default.
- Bundle auto-wiring on instance placement — the "wire bundle from other instance?" prompt is a later phase.

## Decisions

### 1. PortPinItem is a child of InstanceItem

Parent-child relationship keeps the pin's position in scene coordinates automatic: when the instance moves, the pins move with it. No manual sync required.

**Alternative rejected:** free-standing pin items with an `instance_moved` subscription — extra wiring and guaranteed drift bug the first time someone forgets to unsubscribe.

### 2. Pin layout: fixed slot height, variable instance height

InstanceItem's height is determined by the larger of input-count and output-count times a per-pin slot height (e.g. 20 px). Pins stack top-to-bottom within the left or right edge. Instance name + module subtitle fit above the top pin slot.

This keeps the box compact for small modules and scales naturally for AXI-heavy blocks. Alternative — fixed instance size with scrollable pins — is fussy and unexpected on a canvas.

### 3. Direction encoding: shape + arrow

- Input: right-pointing triangle on the left edge.
- Output: right-pointing triangle on the right edge.
- InOut: diamond on whichever edge it is placed.

Using one shape family (triangle/diamond) keeps the visual language consistent with standard schematic symbols. Color is unused for direction — reserved for other semantics (dirty, selected, bundle).

### 4. Width badge as a short suffix label

`[8]`, `[31:0]` formatted right-aligned next to the port name. Omit for single-bit ports. Typography: monospace to keep alignment clean, smaller point size than the pin name.

### 5. Bundle fat-pin as a single PortPinItem variant

A `BundlePinItem` inherits `PortPinItem` with an `expanded` flag. Collapsed state: single fat shape labeled with the bundle name (e.g. `m_axi`). Expanded state: renders its member pins stacked below, with the fat shape acting as a header.

Click on the fat shape toggles `expanded`. Stored locally on the item — not persisted to Schematic (view state, not model state). InstanceItem lays out with expanded-bundle height = sum of member heights + header height.

### 6. Top-level ports as boundary connectors

Render on the scene boundary at negative-x (inputs) and boundary-max-x (outputs). Use the same `PortPinItem` visual vocabulary with different parent — free-standing rather than attached to an InstanceItem. Implements the "structural entity wrapper" visual: inputs enter from the left edge of the sheet, outputs exit to the right.

Position recomputed on every scene-rect change. Simple enough; no need for their own layout system.

## Risks / Trade-offs

- **[Wide buses make labels huge]** → `[31:0]` is the worst common case. Cap label width with eliding (`...`) and show full text in tooltip on hover. Qt's `QGraphicsTextItem` has built-in tooltip support.
- **[Many pins slow paint]** → Qt batches QGraphicsItem draws. Hundreds of pins across dozens of instances are fine. Bundle collapse is the lever if a user has an AXI-heavy design.
- **[Direction ambiguity for inout]** → Diamond shape on whichever edge has fewer pins. Consistent convention better than per-pin heuristic.
- **[Top-level ports overlap with instances placed near boundary]** → Reserve a margin (~30 px) inside the scene rect that instances can't be dragged into. Or accept overlap and rely on user to drag instances away — v1 accepts the latter to keep the code simple.
