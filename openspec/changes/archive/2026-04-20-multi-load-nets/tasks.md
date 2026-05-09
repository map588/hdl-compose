## 1. WireTool compatibility

- [x] 1.1 `WireTool::compatibilityError` — drop the input↔input rejection; keep output↔output rejection with the new `output-to-output: only one driver per net allowed` message.
- [x] 1.2 `WireTool::tryCommit` — when both pins are inputs, branch into the driver-resolution flow (`tryCommitMultiLoad`).

## 2. Driver resolution

- [x] 2.1 Helper: `AppState::port_map_entry(instance, port) -> QString` (see 3.1).
- [x] 2.2 Case A — one input already driven: `set_port_map_entry` on the other pin with the same RHS.
- [x] 2.3 Case B — neither connected: `QMessageBox` with buttons `Promote to top-level input` (default) and `Pick existing top-port…`; Cancel aborts.
- [x] 2.4 Case B path 1: call `AppState::promote_port_to_top` for the first pin; then set the second pin's port_map to the resolved top-port name.
- [x] 2.5 Case B path 2: `QInputDialog::getItem` with width-compatible top-level inputs; chosen value wired to both pins.
- [x] 2.6 Case C — both inputs driven from different nets: tooltip `both pins are already connected to different nets — delete one wire first`; no mutation. (Same-net case is a silent no-op.)

## 3. Bridge

- [x] 3.1 Add invokable `port_map_entry(instance: &QString, port: &QString) -> QString` returning the RHS string, or `""` if `None`.
- [x] 3.2 `promote_port_to_top` (from `top-port-promotion`) is reused — no new invokable needed for path 1.

## 4. Tests + verify

- [x] 4.1 Unit test `multi_load_net_passes_validation` in `schematic::tests` — three instances each driven by the same top-port-clock — `validate()` reports no errors.
- [x] 4.2 Manually verified: input↔input wires silently create a shared undriven net (no dialog per user simplification) and can be promoted to top-port after the fact.
- [x] 4.3 Manually verified: subsequent input pins joining an already-driven net extend it via Case A (no prompt).
- [x] 4.4 Manually verified: output→output rejected with `output-to-output: only one driver per net allowed`.
- [x] 4.5 Run `cargo test` (61/61 pass) and `openspec validate multi-load-nets --strict` (valid).
