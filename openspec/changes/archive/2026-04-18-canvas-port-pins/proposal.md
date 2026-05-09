## Why

Once `canvas-foundation` lands, the canvas shows instance boxes but no ports. That is visually incomplete and leaves no anchor for wires. This change decorates each instance with its port pins, arrows, and width badges — and renders top-level ports as edge connectors on the canvas boundary. Depends on `canvas-foundation`.

Bundle rendering (AXI, APB, etc.) is included here because bundles must degrade gracefully into a single fat pin that expands to members — pin rendering and bundle rendering are the same concern.

## What Changes

- Add `PortPinItem` (QGraphicsItem subclass) — small shape anchored to an InstanceItem edge. Inputs on the left edge, outputs on the right, inouts on whichever edge fits.
- Pin appearance — direction arrow (▶ for in, ◀ for out, ◆ for inout), width badge (`[N]`) for multi-bit ports, port name label adjacent to the shape.
- `BundlePinItem` — a fat pin that represents a detected bundle (e.g. `m_axi`). Click-to-expand reveals member pins; click-to-collapse hides them.
- Top-level ports render on the canvas boundary as edge connectors, visually distinct from instance pins, on the side matching their direction (inputs on the left boundary, outputs on the right).
- Bridge additions — module/instance port metadata invokables: port count, port name, direction, width, bundle membership. Top-level port analogues.

## Capabilities

### New Capabilities
- `canvas-port-pins`: pin rendering on instance edges, direction arrows, width badges, bundle fat-pins with expand/collapse, top-level port edge connectors.

### Modified Capabilities
(none)

## Impact

- **src/gui/app.cpp** — new `PortPinItem`, `BundlePinItem` classes; InstanceItem layout adjusts to host pins; top-level-port overlay on canvas boundary.
- **src/gui/bridge.rs** — invokables: `instance_port_count(i)`, `instance_port_name(i, p)`, `instance_port_direction(i, p)`, `instance_port_width(i, p)`, `instance_port_bundle(i, p)`; top-level analogues `top_port_count()`, `top_port_name(p)`, `top_port_direction(p)`, `top_port_width(p)`.
- **Dependencies** — requires `canvas-foundation`. Unlocks `canvas-wires` (wires connect between pins).
