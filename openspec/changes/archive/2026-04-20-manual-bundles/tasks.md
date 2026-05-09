## 1. Model

- [x] 1.1 `src/types.rs`: add `manual_bundles: HashMap<String, Vec<String>>` to `Instance` with `#[serde(default)]` so v2 files still load.
- [x] 1.2 `src/schematic.rs::validate`: detect manual-bundle entries whose port name isn't on the instance's module — emit an error diagnostic.
- [x] 1.3 Unit tests: default empty; validate rejects unknown ports.

## 2. Project I/O

- [x] 2.1 Bump `.hdlc` `version` constant from 2 to 3; save always writes v3.
- [x] 2.2 Loader accepts v2..=v3; anything else rejected with named version.
- [x] 2.3 Unit test `v3_manual_bundles_round_trip`: save a project with a manual bundle, reload, assert round-trip.
- [x] 2.4 Unit test `load_v2_without_manual_bundles_succeeds`: v2 without the field loads with empty map.

## 3. Bridge

- [x] 3.1 Invokable `create_manual_bundle(instance, name, ports_csv) -> bool` (comma-separated).
- [x] 3.2 Invokable `remove_manual_bundle(instance, name) -> bool`.
- [x] 3.3 Invokables `manual_bundle_count/name/port_count/port_name` for canvas read-back.
- [x] 3.4 Fire `port_map_changed_bulk` after create/remove so the canvas rebuilds layout.

## 4. Canvas

- [~] 4.1 Shift+click multi-select of pins — DEFERRED. Replaced by dialog-with-checkboxes; simpler and avoids selection-state mixing between InstanceItem drag and pin selection.
- [~] 4.2 Rubber-band pin exclusion — DEFERRED for same reason (dialog path doesn't rely on scene selection).
- [x] 4.3 `PortPinItem::contextMenuEvent`: always offers `Group into interface...` and `Promote to top-level port` on a normal pin; offers `Ungroup` on a BundlePinItem. `BundlePinItem` now accepts right-click.
- [x] 4.4 `prompt_create_manual_bundle` dialog: name field + scroll of checkboxes (one per module port, right-clicked pin pre-checked); calls `create_manual_bundle` on accept.
- [x] 4.5 `InstanceItem::layoutPins` builds a `manual_bundle_of[port_name] -> bundle_name` map from AppState invokables and overrides `entries[*].bundle` before the existing bundle grouping logic runs. No new class — manual bundles reuse `BundlePinItem`.
- [x] 4.6 Ungroup action on a `BundlePinItem` calls `remove_manual_bundle`.
- [x] 4.7 Bonus: `src/bundle.rs` removed per user request ("no more auto-detection"). `pub mod bundle` dropped from `src/lib.rs`.

## 5. Tests + verify

- [x] 5.1 Manually verified: right-click → "Group into interface" with checkbox selection produces a collapsed bundle pin.
- [x] 5.2 Manually verified: expand/collapse via click; manual_bundles persist across save/reload.
- [x] 5.3 Manually verified: Ungroup on a manual bundle restores the individual pins.
- [x] 5.5 Run `cargo test` (57/57 pass; prior +4 tests were from deleted `bundle.rs`) and `openspec validate manual-bundles --strict` (valid).
