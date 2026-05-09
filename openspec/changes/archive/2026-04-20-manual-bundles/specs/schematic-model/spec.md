## ADDED Requirements

### Requirement: Instance holds manual bundle metadata
Each `Instance` SHALL hold a `manual_bundles: HashMap<String, Vec<String>>` mapping a bundle name to the ordered list of port names that belong to it.

#### Scenario: Default is empty
- **WHEN** a new `Instance` is created via `Schematic::add_instance`
- **THEN** `manual_bundles` SHALL be an empty map

#### Scenario: Add a manual bundle
- **WHEN** `spi_0.manual_bundles.insert("spi", vec!["mosi", "miso", "sclk", "cs_n"])`
- **THEN** the map contains exactly that entry

#### Scenario: Ports in a manual bundle must exist on the instance's module
- **WHEN** a manual bundle references a port name not present on the module
- **THEN** `Schematic::validate` SHALL emit an error diagnostic identifying the manual bundle and the missing port
