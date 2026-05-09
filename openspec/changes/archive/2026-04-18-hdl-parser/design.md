## Context

The block editor needs structured data about HDL modules before it can do anything useful. Two mature Rust crates exist for this: `vhdl_lang` (VHDL) and `sv-parser` (Verilog/SystemVerilog). Both parse full files but we only need header-level information (entity/module declarations, ports, generics/parameters). The parser module sits between raw HDL files on disk and the in-memory `ModuleDef` data model.

## Goals / Non-Goals

**Goals:**
- Extract `ModuleDef` from `.vhd`/`.vhdl` files via `vhdl_lang`.
- Extract `ModuleDef` from `.v`/`.sv` files via `sv-parser`.
- Map parsed types into the unified `PortType` and `Direction` enums.
- Detect port bundles (AXI, APB, AXI-Stream, generic prefix groups).
- Compute `source_hash` for change detection.
- Return actionable errors for unparseable files without panicking.

**Non-Goals:**
- Full semantic analysis, elaboration, or type checking.
- Parsing architectures, process bodies, or behavioral code.
- Linting or style enforcement.
- Supporting VHDL-87 or Verilog-95 edge cases beyond what the upstream crates handle.
- Record type resolution across files (records are stored as `Record(String)` by name).

## Decisions

### 1. Thin wrapper over upstream crates, not a custom parser

Parse with `vhdl_lang` / `sv-parser`, then walk the AST to extract only what we need. No custom grammar, no tree-sitter, no hand-rolled tokenizer for the parser itself.

**Rationale:** Both crates are actively maintained and handle the full language grammar. Writing our own parser for two HDL languages would be months of work with no upside. We only need the entity/module declaration subset.

**Alternative considered:** tree-sitter grammars exist for both languages but would add a C dependency and the Rust bindings are less ergonomic for structured extraction.

### 2. Single `parse_file(path) -> Result<Vec<ModuleDef>, ParseError>` entry point

One function, dispatches on file extension. Returns a `Vec` because a file can contain multiple entities/modules. Errors are per-file, not per-entity.

**Rationale:** Callers (library scan, file-watch re-parse) don't care which parser ran. `Vec` handles the multi-entity-per-file case in VHDL cleanly.

### 3. PortType mapping strategy

- `std_logic` / single-bit wire → `StdLogic`
- `std_logic_vector` / multi-bit wire/reg → `StdLogicVector(Range)`
- VHDL record types / SV struct types → `Record(type_name_string)`
- Anything else (integer, enum, custom) → `Other(String)` with the raw type text

**Rationale:** Perfect type fidelity across both languages is a rabbit hole. The editor needs direction, width, and "is it compatible?" — not full type resolution. `Other(String)` is the escape hatch.

### 4. Bundle detection runs as a post-pass on the port list

After parsing, a separate function scans the `Vec<PortDef>` and assigns `bundle: Option<String>`. Priority: built-in AXI/APB/AXI-Stream patterns first, then generic prefix heuristic (≥3 ports sharing `prefix_suffix` form).

**Rationale:** Keeps parsing and bundle detection orthogonal. Bundle rules change independently of the parser. Sidecar `.bundles.yaml` override is a file read, not a parse concern.

### 5. source_hash via file content hash, not mtime

Hash the file bytes (e.g., `xxhash64` or `seahash`) rather than relying on filesystem mtime.

**Rationale:** mtime is unreliable across git operations, network drives, and copy. Content hash is deterministic.

## Risks / Trade-offs

- **[`vhdl_lang` API churn]** → Pin a specific version. The crate's public API has changed between releases. Wrap extraction logic so internal changes don't leak.
- **[`sv-parser` incomplete SV coverage]** → Some SystemVerilog 2017+ constructs may not parse. Acceptable — report the error and skip the file. Users can file upstream issues.
- **[Record/struct types are opaque]** → We store the type name string but can't validate member-level compatibility across instances. This is explicitly a v1 non-goal per ARCHITECTURE.md.
- **[Bundle detection false positives]** → The generic prefix heuristic (≥3 ports) could group unrelated ports. Mitigation: sidecar `.bundles.yaml` override, and bundle grouping is cosmetic (doesn't affect connectivity correctness).
