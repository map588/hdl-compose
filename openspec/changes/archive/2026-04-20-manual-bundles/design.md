## Context

The existing bundle story comes from two places: sidecar files authored by the module owner, and a name-prefix heuristic (e.g. all `m_axi_*` ports on a module auto-bundle as `m_axi`). Both live in the parser and are baked into the `ModuleDef` at parse time. The canvas's `BundlePinItem` reads `PortDef.bundle` on the module and groups the layout accordingly.

Manual bundling is a different beast: the user's decision to group pins is *per instance* (the same module may be used twice with different bundling preferences) and is independent of the module's source. The bundle mapping has to live next to the instance in the `Schematic`, not on the `ModuleDef`. That means a new `Instance` field, a new project-file version, and a new overlay at render time: for every pin on an instance, first check if the instance has a manual bundle that covers this port; if so, route the pin through `BundlePinItem` with the manual group's name; otherwise fall back to the module's auto-detected bundle; otherwise render as a plain pin.

## Goals / Non-Goals

**Goals:**

- Shift-click a set of pins on one instance, right-click → "Group into interface…", name the group, see it collapse into a fat pin.
- Persist manual bundles in `.hdlc` so they survive save/reload.
- Ungroup a manual bundle; pins render individually again.
- Co-exist cleanly with auto-detected bundles: if a pin is in both, the manual one wins.

**Non-Goals:**

- Cross-instance bundle recognition (e.g. "this group matches another instance's `m_axi` so match-by-name can connect them") — follow-up.
- Editing a bundle's members after creation (for now: ungroup + regroup). A later change can add "add to bundle" / "remove from bundle".
- Bundle promotion to top-level (i.e. surface a bundle as a grouped top-level port) — follow-up.

## Decisions

### 1. Where the metadata lives

**Decision:** on the `Instance`, not the `Schematic`. Each instance has its own `manual_bundles: HashMap<String /* bundle_name */, Vec<String /* port_names */>>`. Different instances of the same module can have different groupings.

**Alternative considered:** put it on the `ModuleDef` at parse time. Rejected — that'd make the grouping a property of the module, not the usage; and it'd fight with the sidecar/auto-detect path that already owns module-level bundling.

### 2. Port-to-bundle index

**Decision:** the canonical form is `manual_bundles: HashMap<bundle_name, Vec<port_name>>`. At render time, compute the reverse index `HashMap<port_name, bundle_name>` once per instance paint. O(pins × bundles) per render; instances don't have thousands of pins so acceptable.

### 3. Serde / project-file version

**Decision:** bump `version` in `.hdlc` from `2` to `3`. The loader accepts both; a v2 file with no `manual_bundles` field deserializes with an empty map. On save, always write `version: 3` with the field.

**Alternative considered:** additive at v2 with `#[serde(default)]`. Cheaper, but the project's existing versioning convention is to bump on schema additions; stay consistent.

### 4. Selection + context menu

**Decision:** `PortPinItem` gains `ItemIsSelectable` explicitly (today it inherits the non-selectable default). Shift+left-click extends the scene selection across multiple pins. Right-click on any pin when 2+ pins on the *same* instance are selected offers `Group into interface…`. When the action is chosen, all selected pins on the source instance are grouped; pins on other instances in the same scene selection are ignored with a status-bar note.

### 5. Shift-click vs rubber-band

**Decision:** rubber-band does NOT select pins — they remain part of the instance visually but the rubber-band only selects `InstanceItem` and `WireItem` (today's behavior). Pins are selected exclusively via direct shift+click. This avoids "I wanted to rubber-band two boxes but picked up all their pins too".

### 6. Bundle rendering override

**Decision:** in `InstanceItem::layoutPins`, for each port, check the instance's manual bundle map first. If the port is in a manual bundle, add it to that bundle's collected list. Otherwise use the module's existing bundle value. Manual bundles paint using the existing `BundlePinItem`; no new class.

## Risks / Trade-offs

- **[Risk]** A manual bundle's member port may conflict with an auto-detected bundle (same port is claimed by both). → **Mitigation:** manual bundle wins. Document it in the user-facing dialog.
- **[Risk]** Project-file v3 breaks older binaries. → **Mitigation:** the loader already refuses unknown versions; acceptable pre-v1.
- **[Trade-off]** Pins ignoring rubber-band means the user can't "select everything on screen" in one drag. Explicit shift-click keeps the selection semantics clean at the cost of a few extra clicks when grouping large sets.

## Open Questions

- Should the manual-bundle dialog let the user choose between displaying as a collapsed fat pin by default vs expanded header? v1: always collapsed at creation, user toggles as with any bundle.
