## MODIFIED Requirements

### Requirement: Bundle fat-pins with expand/collapse
Bundle ports declared via `Instance.manual_bundles` SHALL render as a single fat pin labeled with the bundle name. Clicking a collapsed bundle pin MUST expand it to show its member pins; clicking again MUST collapse back. Automatic bundle detection has been removed: `PortDef.bundle` is always `None` in parsed modules and grouping happens exclusively through the manual-bundles dialog.

#### Scenario: Collapsed bundle rendering
- **WHEN** an instance's `manual_bundles` declares `spi = [mosi, miso, sclk, cs_n]`
- **THEN** the instance shows a single fat pin labeled `spi` grouping the four member ports

#### Scenario: Expanding a bundle
- **WHEN** the user clicks a collapsed bundle pin
- **THEN** the bundle reveals its member pins stacked below the bundle header, and the instance rectangle grows vertically to accommodate them

#### Scenario: Collapsing a bundle
- **WHEN** the user clicks an expanded bundle header
- **THEN** the members disappear, the instance rectangle shrinks, and only the bundle header remains

#### Scenario: Bundle state is not persisted
- **WHEN** the user closes and reopens a project that had an expanded bundle
- **THEN** the bundle opens in collapsed state (expansion is view state only, never written to `.hdlc`)
