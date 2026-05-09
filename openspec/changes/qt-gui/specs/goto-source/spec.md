## ADDED Requirements

### Requirement: Open HDL source in external editor
The application SHALL launch a user-configured external editor with the HDL source file path when the user triggers goto-source on an instance or module.

#### Scenario: Goto source from sidebar
- **WHEN** the user right-clicks an instance in the sidebar and selects "Goto Source"
- **THEN** the configured editor SHALL open with the module's `source_path`

#### Scenario: Goto source from canvas
- **WHEN** the user double-clicks an instance on the canvas (or uses a keyboard shortcut)
- **THEN** the configured editor SHALL open with the module's `source_path`

#### Scenario: Editor not configured
- **WHEN** goto-source is triggered and no editor is configured
- **THEN** the application SHALL show a dialog prompting the user to set an editor in Preferences

### Requirement: Configurable editor command
The editor command SHALL be configurable in Preferences and persisted to a config file.

#### Scenario: Common editors
- **WHEN** the user sets the editor to `zed`
- **THEN** goto-source SHALL execute `zed <source_path>`

#### Scenario: Custom command
- **WHEN** the user sets the editor to `nvim --remote-tab`
- **THEN** goto-source SHALL execute `nvim --remote-tab <source_path>`

### Requirement: Goto-source is separate from mini editor
Goto-source opens the full HDL source file for editing. The mini editor handles port map syntax only.

#### Scenario: Distinction
- **WHEN** an instance is selected
- **THEN** the mini editor SHALL show port maps, and goto-source SHALL open the full .vhd/.v file — they are independent actions
