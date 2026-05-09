## Why

Users composing a structural wrapper often need to surface a specific instance port as a top-level port of the design — an SPI chip-select, a UART TX line, a clock that comes from a board pin. Today the only way to do that is to manually edit the top-level port list via dialogs and then wire the instance port to it. That workflow is tedious for the common case and invisible from the canvas. A single "Promote to top-level port" action on an instance pin makes the intent obvious and one-click.

## What Changes

- **Right-click menu on `PortPinItem`** gains a `Promote to top-level port` action.
- Picking it (a) adds a new entry to `Schematic::top_ports` matching the instance port's direction, type, and bundle; (b) sets the instance's `port_map[port]` to `NetRef::TopPort(top_name)`; (c) refreshes the canvas so the new top-port chevron appears on the scene boundary.
- Name collision: if a top-port with the chosen name already exists and differs in direction/type, append a numeric suffix.
- Bulk variant on `InstanceItem` context menu: `Promote all unconnected inputs...` / `Promote all unconnected outputs...` (both are scoped follow-up, not gated on this change).

## Capabilities

### New Capabilities

- `top-port-promotion`: the right-click-to-promote action, name resolution, and post-promotion canvas/sidebar state.

### Modified Capabilities

- `canvas-port-pins`: pin context menu gains the promote action.

## Impact

- **Code**: `src/gui/app.cpp` (PortPinItem context menu, new lambda), `src/gui/bridge.rs` (new invokable `promote_port_to_top(instance, port, top_name)`), `src/schematic.rs` (no model change — top_port + set_port_map_entry already exist).
- **Project files**: no schema change.
- **Dependencies**: none.
- **Out of scope**: bulk promotion, automatic renaming of existing top-ports.
