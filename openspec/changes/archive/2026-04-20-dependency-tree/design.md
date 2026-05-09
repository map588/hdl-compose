## Context

Existing parser (`src/vhdl.rs`, `src/verilog.rs`) produces `ModuleDef` from the entity/module header only — name, generics, ports. It does not look at body contents. This is sufficient for composing a structural wrapper around leaf modules, but misses dependency information when user-authored modules are themselves composed of sub-modules.

The sidebar currently renders each canvas instance as a direct child of the project root. No nesting.

Both parser libraries already walk the full AST — `vhdl_lang` produces `DesignFile` which contains architecture bodies; `sv-parser` exposes the full CST. Extending them to gather component references is an incremental pass over the already-built tree, not a separate parse.

## Goals / Non-Goals

**Goals:**
- For each instance on the canvas, show its module's list of unique sub-module names as child rows in the sidebar.
- Each child row indicates whether its module is currently in the library (green/normal) or missing (red with warning icon).
- When the library changes (add/remove source), presence flags re-evaluate and the tree refreshes.

**Non-Goals:**
- Unrolling generate loops or `for-generate` statements into individual sub-instance entries. A `for I in 0 to N-1 generate u_ff : flipflop` references `flipflop` once; we show `flipflop` once, not N times.
- Recursing into sub-modules' sub-modules. The tree shows only one level of dependency under each canvas instance. A fuller recursion is a v2 concern.
- Persisting dependency data to `.hdlc`. Always re-derived from the source.
- Detecting mismatched generics or width/type incompatibilities at the dependency level. Validation is its own concern (handled elsewhere by `Schematic::validate`).

## Decisions

### 1. Parser extension via AST walk, not text scan

For VHDL, use `vhdl_lang`'s AST visitor to find `ConcurrentStatement::Instance` nodes in each architecture body. For Verilog/SystemVerilog, use `sv-parser`'s event iterator to find `ModuleInstantiation` nodes.

Text scanning would be simpler but fragile — comments, conditionals, quoted strings, and vendor-specific keywords all muddy a regex-based approach. AST walk is authoritative.

### 2. Collect unique names per-module, not per-architecture

A VHDL entity can have multiple architectures in a file (`rtl`, `behavioral`, etc.). For v1 we union the component references across all architectures — a "max" view. The user can see all potential deps.

Alternative rejected: parse the `configuration` statement to pick the effective architecture. Realistic .hdlc workflows don't use configurations; overkill for v1.

### 3. Dependency data lives on ModuleDef, not separately

`ModuleDef.dependencies: Vec<String>` is the single source of truth, populated at parse time. Bridge invokables just index into this vector after joining through the library.

Alternative rejected: a separate `DependencyIndex` map. Adds a global mutable structure for no gain — `ModuleDef` already owns per-module data.

### 4. Red state computed on demand in the bridge

`instance_dependency_present(i, j)` checks `AppState.library` contents at call time. No caching, no separate "presence table" — library is small (tens of modules), lookup is O(library_size) per call, tree has O(dep_count) rows. Total O(M×N×K) per rebuild, where M=instances, N=deps per module, K=library size. All small constants.

Alternative rejected: precompute a Set<String> of library names, attach to AppState. Optimization for a case that isn't slow.

### 5. Sidebar rendering via QStandardItem with per-row QColor

Each dependency row is a `QStandardItem` with `setData(QVariant(QColor(220, 60, 60)), Qt::ForegroundRole)` when missing. No custom delegate required.

### 6. Rebuild on library_changed

Already hooked in current code — `library_changed` signal triggers `refresh_sidebar`. Tree rebuild from scratch is cheap (tens of rows); keep this pattern rather than differential updates.

## Risks / Trade-offs

- **[vhdl_lang API surface]** → We already depend on `vhdl_lang`. The `ConcurrentStatement` enum is stable; AST walk adds ~20 lines of code. Low risk.
- **[sv-parser AST complexity]** → SystemVerilog's event tree is more involved than VHDL's AST. Start with simple module instance detection, skip interface instances and parameterized types. Can extend later.
- **[Vendor primitives mistaken for missing deps]** → If user instantiates a vendor primitive like `Xilinx.BUFG`, we'll mark it red. Accepted — no easy way to distinguish. Users can suppress by adding a stub `.vhd` or by Future Work: a per-project "builtin blacklist" setting.
- **[Nested dependencies not shown]** → User with `top → n_register → flipflop` sees `n_register` with `flipflop` as child, but if `flipflop` itself depends on something missing, we won't flag that. One-level view is the explicit v1 scope; second-level would require recursive tree expansion which conflicts with "sidebar stays flat under each instance."
