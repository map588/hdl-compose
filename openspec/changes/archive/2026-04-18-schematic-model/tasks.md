## 1. Foundation

- [x] 1.1 Add `thiserror` dependency and migrate `ParseError` to use `#[derive(thiserror::Error)]`
- [x] 1.2 Define `Language` enum (`Vhdl`, `SystemVerilog`) with serde support
- [x] 1.3 Define `NetRef` enum (`TopPort(String)`, `InstancePort(String, String)`) with serde support
- [x] 1.4 Define `Instance` struct (name, module_ref, generic_map, port_map, position) with serde support
- [x] 1.5 Define `Schematic` struct (top_name, language, top_generics, top_ports, instances, aliases, library_paths)

## 2. Schematic Operations

- [x] 2.1 Implement `Schematic::new(name, language)` constructor
- [x] 2.2 Implement `add_instance` with duplicate name check
- [x] 2.3 Implement `remove_instance` by name
- [x] 2.4 Implement `set_port_map_entry` on an instance (set/clear a single port's NetRef)
- [x] 2.5 Implement `set_generic_map_entry` on an instance
- [x] 2.6 Implement `set_alias` and `remove_alias` for net naming
- [x] 2.7 Implement `resolve_modules` — parse library paths, match instance module_refs to ModuleDefs

## 3. Validation

- [x] 3.1 Define `Diagnostic` type (error vs warning, message, location context)
- [x] 3.2 Validate: duplicate instance names
- [x] 3.3 Validate: NetRef references nonexistent instance
- [x] 3.4 Validate: NetRef references nonexistent port on instance's module
- [x] 3.5 Validate: direction mismatch (input port mapped to another input port's driver)
- [x] 3.6 Validate: width mismatch between connected ports
- [x] 3.7 Validate: duplicate alias names (two nets aliased to same string)
- [x] 3.8 Validate: unresolved module references (instance's module not in library)
- [x] 3.9 Validate: unconnected ports produce warnings (not errors)
- [x] 3.10 Implement `Schematic::validate(&self, library: &[ModuleDef]) -> Vec<Diagnostic>`

## 4. Project I/O

- [x] 4.1 Define `ProjectFile` serde struct with `version: u32` field and schematic data
- [x] 4.2 Implement `save_project(schematic, path)` — serialize to .hdlc JSON
- [x] 4.3 Implement `load_project(path) -> Result<Schematic>` — deserialize, check version, re-parse library paths
- [x] 4.4 Handle missing library files on load (warn, don't fail)
- [x] 4.5 Test round-trip: save then load produces identical schematic

## 5. VHDL Code Generation

- [x] 5.1 Create `src/codegen/mod.rs` and `src/codegen/vhdl.rs` module structure
- [x] 5.2 Emit header comment
- [x] 5.3 Emit `library ieee; use ieee.std_logic_1164.all;` preamble
- [x] 5.4 Emit entity declaration from top-level generics and ports
- [x] 5.5 Emit `architecture structural of <name> is` with signal declarations for internal nets
- [x] 5.6 Derive signal names: alias if set, otherwise `<instance>_<port>`
- [x] 5.7 Emit component declarations (one per unique referenced module)
- [x] 5.8 Emit instance statements with generic map and port map, alphabetical order
- [x] 5.9 Map NetRef to signal names in port map entries; None → `open`
- [x] 5.10 Refuse codegen if validation has errors
- [x] 5.11 Render PortType back to VHDL type strings (StdLogic → `std_logic`, StdLogicVector → `std_logic_vector(N downto M)`, etc.)

## 6. SystemVerilog Code Generation

- [x] 6.1 Create `src/codegen/sv.rs`
- [x] 6.2 Emit header comment (// style)
- [x] 6.3 Emit module declaration with parameters and ports
- [x] 6.4 Emit wire declarations for internal nets
- [x] 6.5 Emit instance statements with parameter overrides and named port connections
- [x] 6.6 Map NetRef to wire names; None → empty parens `.port()`
- [x] 6.7 Refuse codegen if validation has errors
- [x] 6.8 Render PortType back to SystemVerilog type strings (`logic`, `logic [N:0]`, etc.)

## 7. Tests

- [x] 7.1 Unit tests for Schematic operations — add/remove instance, set port/generic map, aliases
- [x] 7.2 Unit tests for validation — each diagnostic type (direction, width, missing ref, duplicate alias, unconnected)
- [x] 7.3 Unit tests for project I/O — save/load round-trip, version check, missing library warning
- [x] 7.4 Integration test: parse real VHDL fixtures → build schematic → generate VHDL → verify output compiles with GHDL (if available)
- [x] 7.5 Integration test: parse real Verilog fixtures → build schematic → generate SV → verify output
- [x] 7.6 Golden-file tests: compare codegen output against checked-in expected files for determinism
- [x] 7.7 Test codegen refusal on validation errors
