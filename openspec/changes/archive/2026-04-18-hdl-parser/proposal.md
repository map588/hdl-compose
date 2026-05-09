## Why

The block editor needs to extract module definitions (entity/module name, generics/parameters, ports with direction/type/width) from existing VHDL and Verilog source files. Without a parser, there is no `ModuleDef` data to populate the library pane, no port list for instance placement, and no type information for connection validation. This is the foundational input layer — everything else (canvas, mini editor, codegen) depends on parsed module headers.

## What Changes

- Add a `parser` crate/module that accepts an HDL source file path and returns a `ModuleDef` struct.
- VHDL parsing via `vhdl_lang` crate — extract entity declarations (name, generics, ports).
- Verilog/SystemVerilog parsing via `sv-parser` crate — extract module declarations (name, parameters, ports).
- Port type mapping into the `PortType` enum (`StdLogic`, `StdLogicVector(Range)`, `Record(String)`, etc.).
- Direction mapping (`In`, `Out`, `InOut`) for both languages.
- Bundle detection on parsed port lists (AXI, APB, AXI-Stream prefix conventions + generic prefix heuristic).
- Source hash computation (`source_hash: u64`) for change detection.
- Error reporting for unparseable files (not a lint tool — report and skip, don't block).

## Capabilities

### New Capabilities
- `vhdl-entity-parsing`: Extract entity name, generics, and ports from VHDL source files using `vhdl_lang`.
- `verilog-module-parsing`: Extract module name, parameters, and ports from Verilog/SystemVerilog source files using `sv-parser`.
- `port-type-mapping`: Map parsed port types from both languages into the unified `PortType`/`Direction` data model.
- `bundle-detection`: Detect AXI/APB/AXI-Stream bundles and generic prefix groups from parsed port lists.

### Modified Capabilities

(none — no existing specs)

## Impact

- **New dependency**: `vhdl_lang` and `sv-parser` crates added to `Cargo.toml`.
- **Data model**: Consumes and produces the `ModuleDef`, `PortDef`, `GenericDef` structs defined in ARCHITECTURE.md.
- **Library paths**: The `Schematic.library_paths` entries are the input — each path is parsed on project load and on file-watch events.
- **Downstream**: Canvas, mini editor, codegen, and re-parse/dirty-detection all depend on parser output.
