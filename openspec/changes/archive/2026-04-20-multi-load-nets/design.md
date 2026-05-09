## Context

The current wire model treats every `NetRef` as "driven by this (top or instance) output". A port_map entry `Some(NetRef::InstancePort("u_pll", "clk_out"))` on `u_counter.clk` says "u_counter.clk is driven by u_pll.clk_out". Codegen's `collect_internal_nets` collects all `InstancePort` values referenced by any port_map and emits `signal u_pll_clk_out` (with alias if set). If two inputs both reference the same driver, they share the signal — no model change needed.

The hard case is two inputs with *no existing driver*: neither is connected. Today `WireTool` rejects this outright. We want to accept it, but we need *some* driver in the model or codegen has nothing to emit. Two routes:

- **Route A — Require a concrete driver.** Prompt the user: "Create a top-level input to drive these?" If yes, promote one pin to a top-level port (via the `top-port-promotion` change) and wire both. If no, cancel the operation. Simple, no model change.
- **Route B — Internal-signal NetRef.** Allow an unnamed/anonymous signal to be the driver. `NetRef::Signal(name)` — stored only as a string. Both inputs reference `NetRef::Signal("spi_cs")`. Codegen emits `signal spi_cs : <type>;` (type inferred from the inputs). Flexible; enables real internal signals without promoting to top-level.

Route B is more correct: users sometimes want a true internal signal (e.g. a handshake between two submodule inputs fed by combinatorial logic that is *not* an instance). But for v1 we reject that case because the model has no way to express "signal fed by external logic" without an actual driver. For now: always require a concrete driver — Route A.

## Goals / Non-Goals

**Goals:**

- `WireTool` allows input↔input pairing.
- If one input is already driven, the second joins that same net.
- If neither is driven, prompt the user to promote a new top-level port or pick an existing top-port.
- Validate that multi-load nets pass existing checks (width/type identical on all loads; driver exists).

**Non-Goals:**

- Internal-signal NetRef variant (deferred; needs a fuller design around "how is width/type inferred from loads").
- Output↔output wiring (multiple drivers) — still rejected.
- Relaxing direction checks on output↔input (already the common case and already works).

## Decisions

### 1. Direction check relaxation

**Decision:** `compatibilityError` continues to reject output↔output. Input↔input becomes a valid pair. For input↔input, downstream logic chooses a *driver rule* (next decision).

### 2. Driver resolution for input↔input

**Decision:** three cases at commit time:

1. One pin is already connected: the other pin's port_map is set to that same `NetRef` (join the existing net). No prompt.
2. Neither pin is connected: show a modal dialog with two choices:
   - **"Promote to top-level input"** (default): creates a new top-port via the `top-port-promotion` invokable with name = current pin's port name (or the first-clicked pin's name), wires both pins to it.
   - **"Pick existing top-port…"** shows a combo of existing compatible top-level inputs; picking one wires both pins to it. Cancel aborts.
3. Both pins are already connected (to different nets): show an error tooltip — the user should delete one wire first rather than silently overwrite. This keeps destructive actions explicit.

### 3. Dependency on `top-port-promotion`

**Decision:** hard dependency. `multi-load-nets` cannot archive before `top-port-promotion` lands, because the "Promote to top-level input" path in case 2 calls the new invokable.

### 4. Direction-mismatch error clarity

**Decision:** when the user clicks two outputs (still rejected), the tooltip becomes `"output-to-output: only one driver per net allowed"`. Clearer than the old `"direction mismatch: cannot pair these pins"`.

## Risks / Trade-offs

- **[Risk]** Users may not expect a confirmation dialog mid-wiring. → **Mitigation:** the dialog appears only in the no-existing-driver case; when joining a net (case 1) the action is silent, matching user intent.
- **[Trade-off]** Route A leaves "pure internal signal with no pin driver" unimplementable until a later change. Accepted; that case is rare in structural wrapper work and the model would need wider redesign anyway (width inference, signal declaration ownership).

## Open Questions

- Should the "Pick existing top-port" chooser filter by resolvable type compatibility, or show all inputs? Default: filter by type (std_logic → std_logic; vector → matching vector or unresolved).
