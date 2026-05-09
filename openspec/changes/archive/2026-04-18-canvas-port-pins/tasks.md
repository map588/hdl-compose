## 1. Bridge port-metadata invokables

- [x] 1.1 Add `instance_port_count(index: i32) -> i32` — resolves instance's module in the cached library and returns `ports.len()`
- [x] 1.2 Add `instance_port_name(instance_index: i32, port_index: i32) -> QString`
- [x] 1.3 Add `instance_port_direction(instance_index: i32, port_index: i32) -> i32` — returns 0/1/2 for In/Out/InOut
- [x] 1.4 Add `instance_port_width(instance_index: i32, port_index: i32) -> i32` — returns 0 for scalar, N for `std_logic_vector(N-1 downto 0)` etc.
- [x] 1.5 Add `instance_port_bundle(instance_index: i32, port_index: i32) -> QString` — returns empty if not bundled
- [x] 1.6 Add top-level analogues: `top_port_count() -> i32`, `top_port_name(i)`, `top_port_direction(i)`, `top_port_width(i)`

## 2. PortPinItem (C++)

- [x] 2.1 Create `PortPinItem` (QGraphicsItem subclass) holding parent `InstanceItem*`, `port_index`, direction, width, `name`
- [x] 2.2 Implement `boundingRect()` — small rectangle bounding the pin shape + label
- [x] 2.3 Implement `paint()` — triangle (left-edge for input, right-edge for output), diamond for inout; label text adjacent
- [x] 2.4 Render width badge `[N]` or `[H:L]` in monospace font, smaller size, elide with `...` if too wide
- [x] 2.5 Tooltip: full `<name> : <type>` for hover hints

## 3. InstanceItem layout update

- [x] 3.1 Extend InstanceItem to compute height from `max(input_count, output_count) * PIN_SLOT_HEIGHT + HEADER_HEIGHT`
- [x] 3.2 On construction, query `instance_port_count(i)` and for each port create a `PortPinItem` as child; position on left/right edge by direction
- [x] 3.3 Re-layout pins after port-metadata changes — listen for `instance_added` / `project_loaded`

## 4. BundlePinItem

- [x] 4.1 Create `BundlePinItem` subclass of `PortPinItem` with `bool expanded` and `std::vector<int> member_port_indices`
- [x] 4.2 Modify InstanceItem pin-layout pass: group ports by `instance_port_bundle()`, create one BundlePinItem per unique bundle, with members hidden in collapsed state
- [x] 4.3 Override `mousePressEvent` on BundlePinItem header: toggle `expanded`; relay out parent InstanceItem
- [x] 4.4 Collapsed render: fat rectangle (~2x pin width) with bundle name inside. Expanded: header + member pins stacked below

## 5. Top-level port boundary connectors

- [x] 5.1 Create `TopPortItem` (QGraphicsItem subclass) using same PortPinItem visual vocabulary but free-standing
- [x] 5.2 In `CanvasLayer`, on `project_loaded` / `instance_added` / `instance_removed`, rebuild a `QVector<TopPortItem*>` anchored to the scene's bounding rect
- [x] 5.3 Layout: input ports on left (x = scene_rect.left - MARGIN), output ports on right (x = scene_rect.right + MARGIN), stacked vertically

## 6. Update qt-gui tasks.md

- [x] 6.1 Replace qt-gui Section 5 tasks 5.3, 5.4, 5.13, 5.16 with a note pointing to `canvas-port-pins`

## 7. Verification

- [x] 7.1 `cargo build` clean
- [x] 7.2 Manual: open a project with an instance of a module with varied ports (1-bit in, 8-bit out, inout) — verify all three render with correct shapes, arrows, badges
- [x] 7.3 Manual: open a project with an AXI module — verify `m_axi` collapses to one fat pin; click to expand shows members; click header to collapse
- [x] 7.4 Manual: top-level ports appear as edge connectors on left/right boundary of canvas
