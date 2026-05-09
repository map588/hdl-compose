## 1. Project Setup

- [x] 1.1 Create `parser` module/crate structure within the workspace
- [x] 1.2 Add `vhdl_lang`, `sv-parser`, and hash crate (`seahash` or `xxhash-rust`) to `Cargo.toml`
- [x] 1.3 Define `ModuleDef`, `PortDef`, `GenericDef`, `PortType`, `Direction` structs/enums (or import from shared data model crate)
- [x] 1.4 Define `ParseError` type and `parse_file(path) -> Result<Vec<ModuleDef>, ParseError>` entry point with extension dispatch

## 2. VHDL Parser

- [x] 2.1 Implement VHDL entity extraction using `vhdl_lang` — parse file, walk AST to find entity declarations
- [x] 2.2 Extract entity name into `ModuleDef.name`
- [x] 2.3 Extract generics into `Vec<GenericDef>` (name, type, default value)
- [x] 2.4 Extract ports into `Vec<PortDef>` (name, direction, raw type info)
- [x] 2.5 Handle multiple entities per file (return `Vec<ModuleDef>`)
- [x] 2.6 Handle parse errors gracefully — return `Err` with descriptive message, empty file returns `Ok(vec![])`

## 3. Verilog/SystemVerilog Parser

- [x] 3.1 Implement Verilog module extraction using `sv-parser` — parse file, walk syntax tree
- [x] 3.2 Extract module name into `ModuleDef.name`
- [x] 3.3 Extract parameters into `Vec<GenericDef>`
- [x] 3.4 Extract ports into `Vec<PortDef>` — support both ANSI and non-ANSI port declarations
- [x] 3.5 Handle `.v` and `.sv` extensions identically
- [x] 3.6 Handle parse errors gracefully — same contract as VHDL parser

## 4. Port Type Mapping

- [x] 4.1 Map VHDL `std_logic` → `StdLogic`, `std_logic_vector(range)` → `StdLogicVector(Range)`
- [x] 4.2 Map Verilog single-bit wire/reg → `StdLogic`, multi-bit `[N:M]` → `StdLogicVector(Range)`
- [x] 4.3 Map VHDL record types → `Record(type_name)`, SV structs → `Record(type_name)`
- [x] 4.4 Map unrecognized types → `Other(raw_string)`
- [x] 4.5 Preserve parameterized width expressions in `StdLogicVector` range (don't evaluate)

## 5. Source Hashing

- [x] 5.1 Implement content-based file hashing (read bytes, compute hash)
- [x] 5.2 Populate `ModuleDef.source_path` and `ModuleDef.source_hash` in both parsers

## 6. Bundle Detection

- [x] 6.1 Implement AXI-Full bundle detection (match `m_axi_*`/`s_axi_*` with required signal set)
- [x] 6.2 Implement AXI-Lite bundle detection
- [x] 6.3 Implement AXI-Stream bundle detection (`m_axis_*`/`s_axis_*` with `tvalid`/`tready`/`tdata`)
- [x] 6.4 Implement APB bundle detection
- [x] 6.5 Implement generic prefix heuristic (≥3 ports sharing `prefix_suffix`, not claimed by built-in)
- [x] 6.6 Enforce priority: built-in conventions before generic heuristic
- [x] 6.7 Implement sidecar `.bundles.yaml` file reading and override logic

## 7. Tests

- [x] 7.1 Unit tests for VHDL parsing — simple entity, multi-entity file, generics, ports, error cases
- [x] 7.2 Unit tests for Verilog parsing — ANSI ports, non-ANSI ports, parameters, `.sv` extension, error cases
- [x] 7.3 Unit tests for port type mapping — all `PortType` variants, both languages
- [x] 7.4 Unit tests for source hashing — stability, change detection
- [x] 7.5 Unit tests for bundle detection — each built-in convention, generic heuristic, sidecar override, priority ordering
- [x] 7.6 Add test HDL fixture files covering the spec scenarios
