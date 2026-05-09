## Why

Real designs have one-to-many nets. A top-level clock drives every register in the wrapper; a reset fans out to every submodule; a chip-select goes to multiple consumers. Today `WireTool::compatibilityError` rejects input-to-input pin pairs as a "direction mismatch", forcing users to manually route through a top-port or an aliased net. That's wrong — the HDL language has no such restriction, and it's the most common wiring pattern in structural code. Users should be able to click two inputs and have the tool do the right thing: wire both loads to the same driver net.

## What Changes

- **Relax `WireTool` direction check**: two inputs can be paired. When the user wires two input pins:
  1. If either pin is already connected (its port_map entry is `Some(NetRef::…)`), the second pin's port_map is set to the same driver. Both inputs then share the net.
  2. If neither is connected, prompt the user: *"Both pins are inputs — they need an external driver. Create a top-level input named `<port>` to drive them?"* Yes → promote one of the pins to a top-level input and wire both to it. No → prompt for a signal name and create an alias-only net (both pins share that name as their driver — codegen treats it as an internal signal declared by the alias).
- `NetRef` gains a fourth variant (or reuses alias infrastructure) to express "driven by a named internal signal" for the no-top-port case. Actually, the existing alias mechanism already handles this if we allow the alias key to be a synthetic `NetRef::Signal(name)` — but the simpler route is to always require a concrete driver pin or top-port. Decision captured in `design.md`.
- **Validation update**: `Schematic::validate` already accepts multi-load nets (several port_map entries can point at the same driver). Confirm no regressions.
- **Wire rendering**: the canvas gains support for N wires sharing a driver anchor point (render each as its own Manhattan path from the same source scene point).

## Capabilities

### New Capabilities

- `multi-load-nets`: UI path for input↔input wiring, including the top-port-promotion prompt and the internal-signal prompt.

### Modified Capabilities

- `wire-interaction`: `compatibilityError` no longer flags input↔input as a direction mismatch; width/type checks still apply.
- `schematic-model`: if a new internal-signal NetRef variant lands, the enum grows.

## Impact

- **Code**: `src/gui/app.cpp` (`WireTool::compatibilityError`, `WireTool::tryCommit`, new prompt dialogs), `src/gui/bridge.rs` (possibly new invokable for "create internal signal").
- **Model**: may or may not extend `NetRef` — see `design.md`.
- **Codegen**: any internal-signal net must render as a `signal` / `wire` declaration in the generated HDL. If we reuse aliases, existing codegen already handles this.
- **Out of scope**: output↔output (two drivers on one net) — real design error; keep rejecting.
