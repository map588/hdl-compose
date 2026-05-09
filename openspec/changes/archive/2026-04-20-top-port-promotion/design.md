## Context

Top-level ports are already first-class in the `Schematic` model: `top_ports: Vec<PortDef>`. Codegen renders them as the entity/module ports on the wrapper. The only thing missing is an ergonomic UI path to create one from an existing instance pin. Today the user has to (a) remember the port's name/direction/type, (b) open the top-port editor (not yet built in the Qt GUI — currently top-ports can only land via `.hdlc` import), (c) re-wire the pin to the new top-port. That's three steps for an action that should take one click.

## Goals / Non-Goals

**Goals:**

- Right-click a pin → "Promote to top-level port" adds a matching top-port and rewires the pin to it.
- Default name = instance port name; if collision, suggest `<port>_<n>` numbered suffix.
- Top-port's direction/type/bundle mirror the source pin. An instance input pin promotes to a top-level input; an instance output pin promotes to a top-level output.
- Canvas refreshes to show the new top-port connector on the scene boundary, with a wire to the promoted pin.

**Non-Goals:**

- Bulk "promote all unconnected" — separate change.
- A standalone top-port editor dialog — separate change.
- Renaming existing top-ports.

## Decisions

### 1. Direction mapping

**Decision:** instance-input → top-level-input; instance-output → top-level-output; instance-inout → top-level-inout. Same direction. This matches the usual "route this signal out of the wrapper" intent.

**Alternative considered:** invert direction (instance input → top output "because the top drives it in"). Rejected — codegen and validation already treat `NetRef::TopPort(name)` as "this instance port is wired to the top port `name`", and a top INPUT drives an instance INPUT just fine. Inversion would break the model.

### 2. Name collision policy

**Decision:** if a top-port with the chosen name already exists:

- If direction+type+bundle match exactly, reuse it (no new top-port created; just set the port_map).
- Else append `_1`, `_2`, … until unique.

Show the resolved name in a toast on the status bar so the user sees what happened.

### 3. Bridge API

**Decision:** new invokable `promote_port_to_top(instance_name: &QString, port_name: &QString) -> QString` returning the resolved top-port name (empty string on failure, `last_error()` holds the reason). The call:

1. Looks up the instance's module port def.
2. Computes a non-colliding top-port name.
3. Appends a `PortDef` to `Schematic.top_ports`.
4. Calls `set_port_map_entry(instance, port, NetRef::TopPort(name))`.
5. Fires `project_loaded` (whole-schematic refresh) so the canvas rebuilds top-ports and wires.

### 4. Menu location

**Decision:** add the action to `PortPinItem::contextMenuEvent` (right-click on pin). Always enabled. Not added to top-ports themselves (no sense promoting a top-port).

## Risks / Trade-offs

- **[Risk]** Firing `project_loaded` on every promotion is heavy (full scene rebuild). → **Mitigation:** acceptable for v1; optimize later if perf is measurable.
- **[Trade-off]** A promoted top-port that collides with a differing existing one gets a suffix. If the user wanted to merge, they can rename after the fact — but rename tooling doesn't exist yet. Accepted; follow-up change can add rename.

## Open Questions

- Should the action also offer a name-entry dialog (so the user can override before creation)? v1: no, just auto-name with collision suffix. Add a follow-up dialog if users ask.
