## ADDED Requirements

### Requirement: Port pins on instance edges
Each instance rectangle SHALL display its module's ports as small graphical items anchored to the rectangle's left and right edges. Input ports MUST be on the left edge, output ports on the right edge, inout ports on whichever edge has fewer pins.

#### Scenario: Rendering pins for an instance
- **WHEN** an instance of a module with M inputs and N outputs is on the canvas
- **THEN** M pin items appear on the instance's left edge and N pin items appear on the right edge, each labeled with its port name

#### Scenario: Re-rendering after module re-parse changes the port list
- **WHEN** the module's port list changes via re-parse (dropping connections, adding new ports)
- **THEN** the pins on every instance of that module refresh to match the new list

### Requirement: Direction arrows and width badges
Pins SHALL indicate direction with a shape (input: right-pointing triangle on left edge; output: right-pointing triangle on right edge; inout: diamond). Multi-bit ports MUST display a width badge in the format `[N]` for an N-bit vector or `[H:L]` for an explicit range.

#### Scenario: Single-bit input pin
- **WHEN** a port is a single-bit `std_logic` input
- **THEN** its pin renders as a right-pointing triangle on the left edge with no width badge

#### Scenario: Multi-bit output pin
- **WHEN** a port is a `std_logic_vector(7 downto 0)` output
- **THEN** its pin renders as a right-pointing triangle on the right edge with a `[7:0]` width badge in monospace font

#### Scenario: Inout pin
- **WHEN** a port has `InOut` direction
- **THEN** its pin renders as a diamond shape

### Requirement: Bundle fat-pins with expand/collapse
Bundle ports (detected via bundle convention) SHALL render as a single fat pin labeled with the bundle name. Clicking a collapsed bundle pin MUST expand it to show its member pins; clicking again MUST collapse back.

#### Scenario: Collapsed bundle rendering
- **WHEN** a module has an `m_axi` bundle of 20 member ports
- **THEN** the instance shows a single fat pin labeled `m_axi` rather than 20 individual pins

#### Scenario: Expanding a bundle
- **WHEN** the user clicks a collapsed bundle pin
- **THEN** the bundle reveals its member pins stacked below the bundle header, and the instance rectangle grows vertically to accommodate them

#### Scenario: Collapsing a bundle
- **WHEN** the user clicks an expanded bundle header
- **THEN** the members disappear, the instance rectangle shrinks to its collapsed height, and only the bundle header remains

#### Scenario: Bundle state is not persisted
- **WHEN** the user closes and reopens a project that had an expanded bundle
- **THEN** the bundle opens in collapsed state (expansion is view state only, never written to `.hdlc`)

### Requirement: Top-level ports on canvas boundary
Top-level ports SHALL render as edge connectors on the scene boundary. Input ports MUST appear on the left boundary, output ports on the right boundary.

#### Scenario: Top-level port placement
- **WHEN** the project's top schematic has inputs `clk`, `rst_n` and output `led`
- **THEN** `clk` and `rst_n` render as edge connectors on the canvas's left boundary, and `led` renders on the right boundary

#### Scenario: Adding or removing top-level ports
- **WHEN** the top port list changes
- **THEN** the boundary connectors re-render to match the current list

### Requirement: AppState exposes port metadata invokables
AppState SHALL expose per-port metadata for instances and for top-level ports.

#### Scenario: Instance port metadata
- **WHEN** C++ calls `AppState::instance_port_count(i)`, `instance_port_name(i, p)`, `instance_port_direction(i, p)`, `instance_port_width(i, p)`, `instance_port_bundle(i, p)`
- **THEN** the calls return the port count, name, direction code (0=In, 1=Out, 2=InOut), width (0 for scalar, N>0 for vector of width N), and bundle name (empty if none)

#### Scenario: Top-level port metadata
- **WHEN** C++ calls `AppState::top_port_count()`, `top_port_name(i)`, `top_port_direction(i)`, `top_port_width(i)`
- **THEN** the calls return equivalent metadata for the top-level port list
