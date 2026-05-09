## ADDED Requirements

### Requirement: Detect AXI-Full bundles by naming convention
The bundle detector SHALL identify AXI-Full master/slave bundles when ports match the `m_axi_*` or `s_axi_*` naming pattern with required AXI signals (`awvalid`, `awready`, `awaddr`, etc.).

#### Scenario: AXI-Full master bundle
- **WHEN** a module has ports `m_axi_awvalid`, `m_axi_awready`, `m_axi_awaddr`, `m_axi_wdata`, `m_axi_wvalid`, `m_axi_wready`, etc.
- **THEN** all matching ports SHALL have `bundle` set to `Some("m_axi")` and the bundle type SHALL be AXI-Full

#### Scenario: Partial AXI signals do not match
- **WHEN** a module has `m_axi_awvalid` and `m_axi_awaddr` but is missing other required AXI signals
- **THEN** the built-in AXI-Full pattern SHALL NOT match (may still match via generic prefix heuristic)

### Requirement: Detect AXI-Lite bundles
The bundle detector SHALL identify AXI-Lite bundles using the same prefix convention but with the AXI-Lite signal subset.

#### Scenario: AXI-Lite slave bundle
- **WHEN** a module has ports matching `s_axi_lite_*` with the AXI-Lite required signals
- **THEN** matching ports SHALL have `bundle` set to `Some("s_axi_lite")`

### Requirement: Detect AXI-Stream bundles
The bundle detector SHALL identify AXI-Stream bundles when ports match `m_axis_*` or `s_axis_*` with `tvalid`, `tready`, `tdata`.

#### Scenario: AXI-Stream master
- **WHEN** a module has ports `m_axis_tvalid`, `m_axis_tready`, `m_axis_tdata`
- **THEN** matching ports SHALL have `bundle` set to `Some("m_axis")`

### Requirement: Detect APB bundles
The bundle detector SHALL identify APB bundles when ports match the APB naming convention with required signals (`psel`, `penable`, `pwrite`, `paddr`, `pwdata`, `prdata`).

#### Scenario: APB interface
- **WHEN** a module has ports `apb_psel`, `apb_penable`, `apb_pwrite`, `apb_paddr`, `apb_pwdata`, `apb_prdata`
- **THEN** matching ports SHALL have `bundle` set to `Some("apb")`

### Requirement: Generic prefix heuristic for unlabeled bundles
When no built-in convention matches, the bundle detector SHALL group ports sharing a `<prefix>_<suffix>` pattern if ≥3 ports share the same prefix and the prefix is not already claimed by a built-in bundle.

#### Scenario: Custom prefix grouping
- **WHEN** a module has ports `uart_tx`, `uart_rx`, `uart_cts`, `uart_rts`
- **THEN** all four ports SHALL have `bundle` set to `Some("uart")`

#### Scenario: Fewer than 3 ports with same prefix
- **WHEN** a module has ports `led_r`, `led_g` (only 2 with prefix `led`)
- **THEN** those ports SHALL NOT be grouped into a bundle

### Requirement: Built-in conventions take priority over generic heuristic
The bundle detector SHALL apply built-in AXI/APB/AXI-Stream detection before the generic prefix heuristic. Ports claimed by a built-in bundle SHALL NOT be re-grouped by the heuristic.

#### Scenario: AXI ports not re-grouped
- **WHEN** a module has a full set of `m_axi_*` ports matching AXI-Full
- **THEN** the generic prefix heuristic SHALL NOT create a second `m_axi` bundle or override the AXI-Full detection

### Requirement: Sidecar override file
The bundle detector SHALL read an optional `<modulefile>.bundles.yaml` sidecar file that can explicitly declare bundles or disable auto-detection for a module.

#### Scenario: Sidecar declares custom bundle
- **WHEN** `fifo_sync.vhd.bundles.yaml` exists and declares a bundle `data_bus` covering ports `din`, `dout`, `dvalid`
- **THEN** those ports SHALL have `bundle` set to `Some("data_bus")` regardless of auto-detection results

#### Scenario: Sidecar disables auto-detection
- **WHEN** the sidecar file sets `auto_detect: false`
- **THEN** only explicitly declared bundles from the sidecar SHALL be applied; no auto-detection SHALL run

#### Scenario: No sidecar file
- **WHEN** no `.bundles.yaml` sidecar exists for a module
- **THEN** auto-detection SHALL run normally
