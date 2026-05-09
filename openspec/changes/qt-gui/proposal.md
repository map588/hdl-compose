## Why

hdl-compose has a complete backend — parser, schematic model, validation, codegen, project I/O, and CLI. But the tool's thesis is that a *GUI* can make structural HDL composition faster than hand-typing. The CLI proves the data model works; the GUI proves the tool is worth using. Without it, hdl-compose is a library with a shell wrapper, not the block diagram editor it's designed to be.

The first attempt at this tool used egui with egui_snarl for the canvas. It failed — interaction, visuals, and layout were all subpar. This attempt uses Qt via cxx-qt, giving us QGraphicsScene for the canvas, QTreeView for the sidebar, QPlainTextEdit for the mini editor, and Qt's theming system for material-quality aesthetics. The Rust core (Schematic, parser, codegen) stays untouched; Qt is purely the presentation layer.

## What Changes

- Add `cxx-qt` and Qt dependencies to the project.
- Create a QMainWindow-based application shell with material color scheme.
- **Sidebar**: QTreeView hierarchy tree showing instances as children of the top-level design, plus a library pane showing parsed but unplaced modules. Drag from library onto canvas to create instances.
- **Canvas**: QGraphicsScene/QGraphicsView block diagram. Instances as draggable boxes with labeled port pins (direction arrows, width badges). Wires between connected ports. Pan/zoom/scroll navigation. Click-port-click-port to create a connection (emits equivalent text edit). Bundle ports render as expandable fat pins.
- **Mini editor**: QPlainTextEdit per selected instance. VHDL-shaped syntax showing `generic map` / `port map`. RHS accepts: `<identifier>` (top port/alias), `<instance>.<port>` (driver reference), `open`. QCompleter autocomplete after `=>` filtered by direction/type compatibility. Inline squiggles for width/type mismatches.
- **Goto-source**: Configurable external editor command (neovim, zed, code). Opens HDL source file at the module definition. Separate from mini editor — mini editor handles port maps, external editor handles HDL source.
- **Match-by-name**: Opt-in button per instance. Auto-connects ports with matching names/types. Never automatic on placement.
- **Module re-parse**: File watch on library paths. On source change, diff port list against stored ModuleDef. Break connections to changed/removed ports, mark instance dirty (red dot sidebar, red outline canvas).
- **Bidirectional sync**: Canvas edits and mini editor edits both mutate the same in-memory Schematic. Neither view is canonical.

## Capabilities

### New Capabilities
- `qt-app-shell`: QMainWindow, material theming, menu bar, toolbar, application lifecycle.
- `sidebar`: QTreeView hierarchy + library pane, drag-to-instantiate.
- `canvas`: QGraphicsScene block diagram — instances, ports, wires, pan/zoom, click-to-wire.
- `mini-editor`: Per-instance VHDL-shaped text editor with autocomplete, validation squiggles, and bidirectional model sync.
- `goto-source`: External editor integration for viewing/editing HDL source files.
- `match-by-name`: Opt-in auto-connect by port name matching.
- `module-reparse`: File-watch, port diff, dirty marking, connection breakage on incompatible changes.

### Modified Capabilities

(none — all existing capabilities are backend; GUI is a new layer)

## Impact

- **New dependencies**: `cxx-qt`, `cxx-qt-lib`, `cxx-qt-build`, Qt6 (system dependency).
- **Build system**: `build.rs` for cxx-qt code generation. Qt must be installed on the build machine.
- **New files**: `src/gui/` module tree — app.rs, sidebar.rs, canvas.rs, editor.rs, theme.rs, etc.
- **Platform**: macOS primary, Linux secondary. Windows deferred.
- **Binary**: The existing CLI stays as-is. GUI launches when no subcommand is given, or via `hdl-compose gui`.
- **Downstream**: This is the final major feature for v1. After GUI, it's polish, testing, and release.
