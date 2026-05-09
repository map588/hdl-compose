## ADDED Requirements

### Requirement: Instances render as labeled boxes with port pins
Each instance SHALL render as a rectangular block on the canvas with the instance name and module name as labels, and port pins along the edges.

#### Scenario: Instance appearance
- **WHEN** an instance `u_fifo : fifo_sync` exists with 8 ports
- **THEN** the canvas SHALL show a rectangle labeled `u_fifo` (and `fifo_sync`), with 8 port pins arranged along left (inputs) and right (outputs) edges

#### Scenario: Port pin labels
- **WHEN** a port pin is rendered
- **THEN** it SHALL show the port name, a direction arrow (→ in, ← out, ↔ inout), and a width badge (e.g., `[8]`) for multi-bit ports

### Requirement: Instances are draggable
The user SHALL be able to drag instances to reposition them on the canvas. Position is persisted in the Schematic model.

#### Scenario: Drag instance
- **WHEN** the user drags `u_fifo` to a new position
- **THEN** the instance position SHALL update in the Schematic and persist on save

### Requirement: Wires render between connected ports
For each connection in the port map, a wire SHALL render from the driving port pin to the consuming port pin.

#### Scenario: Wire between instances
- **WHEN** `u_fifo.din` is connected to `NetRef::InstancePort("u_adc", "data_out")`
- **THEN** a wire SHALL render from `u_adc`'s `data_out` pin to `u_fifo`'s `din` pin

#### Scenario: Wire to top-level port
- **WHEN** `u_fifo.clk` is connected to `NetRef::TopPort("clk")`
- **THEN** a wire SHALL render from the top-level `clk` port indicator to `u_fifo`'s `clk` pin

### Requirement: Click-port-click-port wiring
The user SHALL be able to click a port pin, then click another port pin, to create a connection. This emits the equivalent mini editor text edit.

#### Scenario: Create connection via canvas
- **WHEN** the user clicks `u_adc.data_out` then clicks `u_fifo.din`
- **THEN** the port map SHALL update: `u_fifo.din => NetRef::InstancePort("u_adc", "data_out")`, and the mini editor SHALL reflect the change

#### Scenario: Invalid connection rejected
- **WHEN** the user tries to connect two output ports
- **THEN** the connection SHALL be rejected with a visual/audible indication

### Requirement: Pan and zoom
The canvas SHALL support panning (middle-click drag or scroll) and zooming (Ctrl+scroll or pinch).

#### Scenario: Zoom in/out
- **WHEN** the user scrolls with Ctrl held
- **THEN** the canvas SHALL zoom in or out centered on the cursor position

#### Scenario: Pan
- **WHEN** the user middle-click drags
- **THEN** the canvas SHALL pan to follow the drag

### Requirement: Click instance to select
Clicking an instance on the canvas SHALL select it, highlight it, and open its port map in the mini editor.

#### Scenario: Select instance
- **WHEN** the user clicks on `u_fifo` on the canvas
- **THEN** `u_fifo` SHALL be highlighted, selected in the sidebar, and its port map SHALL appear in the mini editor

### Requirement: Bundle ports render as expandable fat pins
Ports with a `bundle` value SHALL render as a single combined pin. Clicking the bundle pin expands to show individual member pins.

#### Scenario: Collapsed bundle
- **WHEN** an instance has ports `m_axi_awvalid`, `m_axi_awready`, etc. with bundle `m_axi`
- **THEN** they SHALL render as a single fat pin labeled `m_axi`

#### Scenario: Expanded bundle
- **WHEN** the user clicks the `m_axi` fat pin
- **THEN** individual member pins SHALL appear and be individually connectable

### Requirement: Right-click net to rename
Right-clicking a wire on the canvas SHALL offer a "Rename" option to set an alias for the net.

#### Scenario: Set alias via canvas
- **WHEN** the user right-clicks a wire and enters alias `sys_clk`
- **THEN** the alias SHALL be set in the Schematic and the signal name SHALL update in codegen output

### Requirement: Dirty instances have red outline
Instances marked dirty SHALL have a red outline on the canvas.

#### Scenario: Dirty visual
- **WHEN** an instance is dirty (module source changed incompatibly)
- **THEN** its rectangle on the canvas SHALL have a red border
