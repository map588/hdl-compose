# manual-bundles Specification

## Purpose
TBD - created by archiving change manual-bundles. Update Purpose after archive.
## Requirements
### Requirement: Create a manual bundle via right-click
Right-clicking an instance `PortPinItem` SHALL offer a `Group into interface...` action. Choosing it SHALL open a dialog with a bundle-name field and a checkbox per module port (the right-clicked port pre-checked). Submitting with a non-empty name and at least two checked ports SHALL persist the group on the instance.

#### Scenario: Create a new bundle
- **WHEN** on `spi_0` the user right-clicks `mosi`, chooses `Group into interface...`, names it `spi`, checks `mosi`, `miso`, `sclk`, `cs_n`, and submits
- **THEN** `spi_0.manual_bundles["spi"]` contains `["mosi", "miso", "sclk", "cs_n"]`
- **AND** the canvas re-renders `spi_0` with a single collapsible `spi` bundle pin in place of the four individual pins

#### Scenario: Bundle with fewer than two ports is rejected
- **WHEN** the user submits the dialog with 0 or 1 ports checked
- **THEN** no change is made

### Requirement: Ungroup a manual bundle
Right-clicking a manual-bundle pin SHALL offer an `Ungroup` action that removes the bundle entry and restores the pins' individual rendering.

#### Scenario: Ungroup restores individual pins
- **WHEN** the user right-clicks the `spi` bundle on `spi_0` and chooses `Ungroup`
- **THEN** `spi_0.manual_bundles` no longer has a `spi` entry
- **AND** the canvas re-renders with the four pins as individual `PortPinItem`s

### Requirement: Manual bundles persist through save/reload
`.hdlc` project files SHALL serialize and deserialize `Instance.manual_bundles`. A v2 file with no field loads with an empty map; a v3 file round-trips.

#### Scenario: Round-trip v3
- **WHEN** a schematic with `spi_0.manual_bundles["spi"] = ["mosi", "miso", "sclk", "cs_n"]` is saved and reopened
- **THEN** `spi_0.manual_bundles` matches exactly

#### Scenario: v2 file loads with empty map
- **WHEN** a v2 `.hdlc` without the `manual_bundles` field is opened
- **THEN** every instance has an empty `manual_bundles` map

