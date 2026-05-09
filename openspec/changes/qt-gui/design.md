## Context

hdl-compose backend is complete: parser (VHDL + Verilog), schematic model with validation, codegen (VHDL + SV), project I/O (.hdlc), and CLI. 55 tests passing. The first attempt at a GUI used egui + egui_snarl and failed on all UX fronts (interaction, visuals, layout). This attempt uses Qt via cxx-qt.

The core thesis: text and canvas are two live views of one in-memory Schematic. Canvas shows topology. Wiring happens in a per-instance VHDL-shaped mini editor. Canvas click-click wiring is a shortcut that emits the equivalent text edit. Neither view is canonical — the model is.

## Goals / Non-Goals

**Goals:**
- Professional-quality desktop application that an FPGA engineer would use daily.
- Material color scheme, polished UI, smooth scrolling and navigation.
- Block diagram canvas with draggable instances, port pins, wire routing, pan/zoom.
- Per-instance mini editor with VHDL-shaped syntax, autocomplete, inline diagnostics.
- Sidebar with hierarchy tree and module library.
- Bidirectional sync: edits in either view update the model, which re-renders both views.
- Goto-source: open HDL files in user's preferred external editor.
- Opt-in match-by-name auto-connect.
- File-watch with dirty marking on module source changes.

**Non-Goals:**
- Hierarchical editing (double-click instance to open sub-schematic) — v2.
- Undo/redo — v2.
- Windows support — deferred.
- Behavioral HDL editing (processes, always blocks).
- Synthesis, simulation, or linting.
- Custom wire routing algorithms — start with Manhattan routing, improve later.

## Decisions

### 1. Qt6 via cxx-qt (KDAB)

Use cxx-qt for Rust ↔ Qt interop. Rust owns the data model (`Schematic`, parser, codegen). Qt owns the presentation. Communication via cxx-qt bridge: Rust structs exposed as QObject properties and invokable methods.

**Rationale:** Qt provides QGraphicsScene (industry-standard node graph canvas), QTreeView, QPlainTextEdit, QCompleter, QStyle theming — all production-grade and battle-tested. cxx-qt is KDAB-maintained, production-quality, and avoids writing raw C++ for most cases.

**Alternative rejected:** iced (pure Rust) — would require building every widget from scratch. egui — failed in v1. GTK4 — weaker on macOS.

### 2. Architecture: Rust model, Qt views

```
┌─────────────────────────────────────────────────┐
│                    Qt Layer                      │
│                                                  │
│  ┌──────────┐  ┌──────────────┐  ┌───────────┐  │
│  │ Sidebar  │  │    Canvas    │  │   Mini     │  │
│  │ QTreeView│  │ QGraphics   │  │  Editor    │  │
│  │          │  │ Scene/View  │  │ QPlainText │  │
│  └────┬─────┘  └──────┬──────┘  └─────┬─────┘  │
│       │               │               │         │
│       └───────────────┼───────────────┘         │
│                       │                          │
│              ┌────────▼────────┐                 │
│              │  Bridge Layer   │                 │
│              │  (cxx-qt)       │                 │
│              └────────┬────────┘                 │
└───────────────────────┼──────────────────────────┘
                        │
┌───────────────────────▼──────────────────────────┐
│                  Rust Core                        │
│                                                   │
│  Schematic │ Parser │ Codegen │ Validation │ I/O  │
│                                                   │
│  (untouched — same library as CLI)                │
└───────────────────────────────────────────────────┘
```

All mutations go through the bridge to the Rust Schematic. Views re-render on model change signals.

### 3. Canvas: QGraphicsScene with custom items

- `InstanceItem` (QGraphicsRectItem subclass): draggable rectangle with port pins.
- `PortPinItem`: small circle/rect at instance edge. Direction arrow, width badge. Clickable for wiring.
- `WireItem` (QGraphicsPathItem): Manhattan-routed path between connected ports.
- `BundlePinItem`: fat pin that expands on click to show member pins.
- Pan/zoom via QGraphicsView::setDragMode and wheelEvent.

**Rationale:** QGraphicsScene handles hit-testing, z-ordering, item selection, rubber-band select, and efficient rendering out of the box. Custom items just need paint() and shape().

### 4. Mini editor: QPlainTextEdit + custom completer

Per-instance buffer rendered when an instance is selected. Content mirrors VHDL `generic map` / `port map` syntax. On text change, parse the buffer and update the Schematic model. On model change from canvas, regenerate the buffer text.

Autocomplete via QCompleter triggered after `=>`:
- Top-level port names (matching direction)
- Existing aliases
- `<instance>.<port>` for all driving outputs in scope
- After `<instance>.`: ports of that instance filtered by direction compatibility

Validation squiggles via QSyntaxHighlighter: red underline for width mismatch, type mismatch, unknown reference.

### 5. Goto-source: configurable external editor

User configures editor command in preferences (persisted in config file): `neovim`, `zed`, `code`, or custom command string. On "goto source," launch `QProcess::startDetached(editor, [source_path])`. Separate from mini editor — goto-source opens the full HDL file, mini editor shows port maps only.

### 6. Module re-parse via file watching

Use `QFileSystemWatcher` on all library paths. On file change:
1. Re-parse the changed file.
2. Diff new port list against stored `ModuleDef` (compare name, direction, type, width).
3. For each instance of that module: drop connections to changed/removed ports, mark instance dirty.
4. Dirty = red dot in sidebar, red outline on canvas, diagnostic comment in mini editor.
5. New ports appear unconnected. User fixes explicitly.

No auto-migration. Breakage is surfaced because silent rewiring is a correctness hazard.

### 7. Material color scheme

Use Qt's QPalette system with a custom dark material palette. Define in a `theme.rs` / `theme.cpp` module. Colors for: background, surface, primary, secondary, error, on-surface, etc. Consistent across sidebar, canvas, mini editor.

## Risks / Trade-offs

- **[cxx-qt learning curve]** → KDAB documentation is good. Start with a minimal QMainWindow, add widgets incrementally.
- **[Qt6 system dependency]** → Users must install Qt6. Mitigate with clear build instructions and consider bundling via `aqtinstall` in CI.
- **[Canvas performance with many instances]** → QGraphicsScene is optimized for thousands of items. Should not be an issue for structural wrappers (typically <100 instances).
- **[Mini editor sync complexity]** → The parse-on-edit / regen-on-model-change loop is the hardest part. Start with one-way (editor → model) and add model → editor regen after.
- **[macOS Qt rendering]** → Qt on macOS uses native rendering. Looks decent but not perfectly native. Material theme helps unify across platforms.
- **[No undo/redo in v1]** → Acknowledged risk for usability. Mutations are destructive. Mitigation: frequent auto-save of .hdlc.
