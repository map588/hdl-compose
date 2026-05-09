## ADDED Requirements

### Requirement: Application window with three-pane layout
The application SHALL display a QMainWindow with three panes: sidebar (left), canvas (center), and mini editor (right).

#### Scenario: Application launches
- **WHEN** `hdl-compose gui` or `hdl-compose` (no subcommand) is run
- **THEN** a window SHALL appear with sidebar, canvas, and editor panes visible

#### Scenario: Panes are resizable
- **WHEN** the user drags a splitter between panes
- **THEN** pane widths SHALL adjust and persist across sessions

### Requirement: Material color scheme
The application SHALL use a dark material-inspired color palette applied via QPalette across all widgets.

#### Scenario: Consistent theming
- **WHEN** the application is running
- **THEN** sidebar, canvas, mini editor, menus, and toolbars SHALL all use the material color scheme

### Requirement: Menu bar with standard actions
The application SHALL have a menu bar with File (New, Open, Save, Save As, Exit), Edit (future), and View menus.

#### Scenario: Open project
- **WHEN** the user selects File → Open and chooses a .hdlc file
- **THEN** the project SHALL load and all panes SHALL update to reflect its contents

#### Scenario: Save project
- **WHEN** the user selects File → Save
- **THEN** the current Schematic SHALL be saved to the loaded .hdlc file

#### Scenario: New project
- **WHEN** the user selects File → New
- **THEN** a dialog SHALL prompt for project name and language, then create an empty schematic

### Requirement: Title bar shows project name
The window title SHALL display the project name and a dirty indicator when unsaved changes exist.

#### Scenario: Dirty indicator
- **WHEN** the schematic is modified after the last save
- **THEN** the title bar SHALL show an asterisk (*) after the project name

### Requirement: Preferences for external editor
The application SHALL provide a preferences dialog where the user can set the external editor command (e.g., `neovim`, `zed`, `code`).

#### Scenario: Set editor preference
- **WHEN** the user opens Preferences and sets the editor to `zed`
- **THEN** goto-source actions SHALL launch `zed` with the file path
