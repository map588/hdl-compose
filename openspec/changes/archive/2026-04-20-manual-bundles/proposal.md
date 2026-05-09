## Why

hdl-compose already detects port bundles (AXI, AHB, APB, etc.) via the sidecar convention — a module declares its bundle prefixes and the canvas renders those ports as a single collapsible fat pin. That's great when the module's author has annotated them, but it's useless for ad-hoc or in-progress modules where the user *knows* which pins belong together but hasn't (or can't) edit the HDL source to add sidecar metadata. Users should be able to shift-select a set of pins on any instance, right-click, and say "group these into an interface" — hdl-compose then treats that group as a bundle for display, selection, and (eventually) matching purposes.

## What Changes

- **Shift-click multi-select** on `PortPinItem` extends the scene selection (rubber-band already selects, but pins aren't currently targeted by it; explicit shift+left-click on pins accumulates them).
- **`Group into interface...`** context menu on a multi-pin selection or on any individual pin in the group, prompts for a bundle name. Writes a new `manual_bundles: HashMap<String, ManualBundle>` entry to the `Instance` (or the `Schematic`, see `design.md`), overriding auto-detected bundles for those pins.
- **Bundle rendering** for manual bundles uses the same collapse/expand `BundlePinItem` as auto-detected bundles; the group is addressable as `<inst>.<group>` (collapsed) or `<inst>.<group>.<port>` (expanded member).
- **Ungroup** action on a manual bundle restores the pins to their individual rendering.
- **Project file v3** adds `manual_bundles` to each instance's JSON.

## Capabilities

### New Capabilities

- `manual-bundles`: shift-select, group/ungroup dialog, bundle metadata model, persistence, canvas rendering override.

### Modified Capabilities

- `canvas-port-pins`: bundle-pin layout gains a manual-override code path; pin hit-testing supports shift-click multi-select.
- `schematic-model`: `Instance` gains `manual_bundles`; `.hdlc` serde version bumps to 3.
- `project-io`: migration from v2 → v3 (loader accepts missing `manual_bundles` field → empty map).

## Impact

- **Code**: `src/types.rs` (new `ManualBundle` type + `Instance.manual_bundles`), `src/project.rs` (v3 format, v2 migration), `src/gui/bridge.rs` (new invokables `create_manual_bundle` / `remove_manual_bundle`), `src/gui/app.cpp` (pin multi-select, context menu, layout).
- **Project files**: one-way migration — opening a v2 `.hdlc` produces a v3 on save.
- **Codegen**: no change — port_map keys still use individual port names. The bundle is a view/UX concept.
- **Out of scope**: automatic bundle detection from common prefixes (that's the existing sidecar-driven path). Inter-module bundle matching is a follow-up to `match-by-name`.
