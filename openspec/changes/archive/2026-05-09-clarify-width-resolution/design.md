## Context

`hdl_compose` extracts module interfaces (parser → `ModuleDef` with
`PortDef::port_type: PortType`), wires them into a graph (`Schematic`
with `Instance::generic_map: HashMap<String, String>`), and emits a
top-level wrapper (`generate_sv` / `generate_vhdl`). For vector ports
whose width is parameterized (`[WIDTH-1:0]`, `std_logic_vector(N-1
downto 0)`), there are two places in the pipeline that can decide
whether to keep the symbolic expression or substitute a concrete
integer:

1. **At parse time.** The parser already attempts simple constant
   evaluation (`try_eval_expr` for VHDL, `eval_simple_expr` for
   SystemVerilog). When evaluation fails — typically because a generic
   appears in the expression — it falls back to `RangeExpr::Expr(s)`
   carrying the original textual fragment.
2. **At emit time.** `codegen::resolve_port_type` walks the
   `PortType::StdLogicVector` range and replaces any non-literal bound
   with `RangeExpr::Literal(n)` if it can be resolved against the
   substitution map produced by `build_generic_substitutions`. The
   substitution map is built from (a) the source module's
   `GenericDef::default_value`s, then (b) overlaid with the per-instance
   `inst.generic_map` entries set via the GUI's `set_generic_map_entry`.

Today's behavior is option (2) — eager resolution at emit time. The
round-trip test in `tests/sv_roundtrip.rs` and `tests/vhdl_roundtrip.rs`
must therefore call `resolve_port_type` on **both** sides of the
comparison (`tests/common/mod.rs:111-127`) before checking equality,
because the original parsed `PortDef` still carries
`RangeExpr::Expr("WIDTH-1")` while the regenerated `PortDef` carries
`RangeExpr::Literal(7)`.

The two policies below are mutually exclusive contracts. Only one can be
correct.

## Policy A — PRESERVE (lazy / symbolic)

The codegen emits the same syntactic form the parser captured. If the
source said `[WIDTH-1:0]`, the wrapper says `[WIDTH-1:0]`. Resolution
only happens for callers that explicitly need an integer (validation,
width-mismatch diagnostics, the GUI badge).

### (a) Contract

- Generated HDL preserves parameterized widths textually.
- A user who sets `WIDTH=16` on an instance via the GUI sees the
  override in the generated **`generic map` / `#(.WIDTH(16))`** block —
  *not* baked into the port-type ranges of the wrapper.
- The wrapper's own top ports keep whatever symbolic form the schematic
  carries; if the schematic was built by promoting a child port whose
  type was `[WIDTH-1:0]`, the wrapper port stays `[WIDTH-1:0]` and the
  wrapper exposes `WIDTH` as its own top-level generic so the form is
  legal.
- `resolve_port_type` becomes an internal helper for *width-checking*
  and *GUI display*, not for code emission.

### (b) Files that must change

- `src/codegen/mod.rs` — drop the three `resolve_port_type` calls in
  `collect_top_intermediates` (line 220), `collect_internal_nets` (line
  292), and remove the helper from the public emit-time path. Keep it
  exported for the bridge's `resolve_port_width` use.
- `src/codegen/sv.rs:128` and `src/codegen/vhdl.rs:157` — emit
  `p.port_type` directly instead of the resolved variant.
- `src/codegen/sv.rs` and `src/codegen/vhdl.rs` — when an internal net
  carries a symbolic width, the wrapper must declare the relevant
  generics on its own port (or fail validation). New helper:
  lift any `RangeExpr::Expr` referenced by an internal net into
  `Schematic::top_generics` if not already present, or surface a new
  `Diagnostic::Error` if the user hasn't promoted them.
- `src/types.rs` / `src/schematic.rs` — possibly extend `Schematic`
  with a "promote-source-generics" helper so the GUI can keep symbolic
  widths viable end-to-end. Without this, every parameterized
  passthrough becomes a validation error.

### (c) Tests that would shift

- **Codegen unit tests will break.** Two named tests assert the eager
  literal output today and codify RESOLVE as the contract:
  - `top_port_resolves_generic_default` (`src/codegen/vhdl.rs:667`) —
    asserts `dout : out std_logic_vector(7 downto 0)` for the default
    `WIDTH=8`.
  - `top_port_resolves_generic_override` (`src/codegen/vhdl.rs:683`) —
    asserts `dout : out std_logic_vector(15 downto 0)` for an override
    `WIDTH=16`.
  - `internal_net_resolves_generic_override` (`src/codegen/vhdl.rs:697`)
    — asserts `signal u_drv_bus : std_logic_vector(15 downto 0);`.
  All three names contain the word *resolves*. Under PRESERVE these
  must be rewritten to assert `WIDTH-1 downto 0` and to verify the
  override appears in the `generic map` block instead.
- **Round-trip tests are simplified.** The normalization step in
  `tests/common/mod.rs:111-127` is removed; ports compare directly.

### (d) Impact on the GUI parameter override flow

- `set_generic_map_entry` (`src/gui/bridge.rs:1654`) still updates
  `inst.generic_map`. The override **does** appear in the generated
  text — but only inside the instance's `#( .WIDTH(16) )` /
  `generic map (WIDTH => 16)` block, not by mutating the surrounding
  port-type widths.
- `resolve_port_width` (`src/gui/bridge.rs:1993`) and the canvas pin
  width badge are unaffected — they were already calling the resolver
  for *display*, not for emission.
- Risk: the GUI today relies on `port_map_changed_bulk` after a generic
  edit to redraw widths. If the wrapper port type stays symbolic, a
  width-changed override will not change the wrapper's signature, so
  the canvas only needs to redraw the affected instance pins. That is a
  net simplification.

### (e) Impact on round-trip tests

- `tests/common/mod.rs` loses its `resolve_port_type` import and the
  empty-overrides plumbing. `assert_shape_eq` becomes a straight
  `PortDef` equality comparison.
- The round-trip then *actually* asserts that "what the parser sees,
  the generator must emit verbatim" — which is a stronger contract than
  what we have today.

### (f) Risks

- **Validation surface grows.** Today a wrapper that promotes a child
  port with `[WIDTH-1:0]` works because the codegen quietly bakes in
  the default. Under PRESERVE, the wrapper must legally declare
  `WIDTH` itself or the emitted HDL is malformed. The codegen pipeline
  has no auto-promotion of child generics today; that would be net new
  feature work.
- **Tooling that doesn't resolve generics may break.** If a user feeds
  the generated wrapper to a tool that has its own (different) idea of
  what `WIDTH` defaults to, behavior shifts. With RESOLVE, the wrapper
  is self-contained.
- **Override visibility.** A user inspecting the generated wrapper to
  confirm "did my `WIDTH=16` actually take effect?" no longer sees
  `[15:0]` in the wrapper port list — they must scan the `generic map`
  block. This is arguably what HDL idiom expects, but it is a UX
  change.

## Policy B — RESOLVE (eager / literal) — STATUS QUO

The codegen substitutes generic defaults and instance overrides at emit
time. Generated HDL contains literal widths only. This is what the code
does today.

### (a) Contract

- Generated HDL has only literal range bounds for any port whose width
  is resolvable from `module_generics + inst.generic_map`.
- A user override on an instance changes both (i) the literal width in
  the surrounding port-type emission for any wrapper top port whose
  context resolves through that instance and (ii) the generic-map block
  itself.
- Unresolvable bounds (a generic with no default and no override, or an
  expression the toy evaluator doesn't handle) survive as `RangeExpr::
  Expr` and emit verbatim. The validator surfaces unresolved references
  as a separate diagnostic — codegen does not silently drop the
  symbolic form when it can't substitute.
- Round-trip equality is not a syntactic-sameness contract; it is a
  *resolved-shape-sameness* contract.

### (b) Files that must change

- `src/codegen/mod.rs` — add a doc-comment to `resolve_port_type`
  explicitly stating the policy and listing the three call sites that
  depend on it. No code change.
- `tests/common/mod.rs` — keep the existing normalization. Update the
  doc comment from "necessary because the codegen *itself* resolves"
  (which currently reads as a workaround note) to "consistent with the
  codegen RESOLVE policy: round-trip compares the resolved form on both
  sides."
- A new spec capability — `width-resolution-policy` — or an addition to
  the `sv-codegen` and `vhdl-codegen` specs that documents RESOLVE as
  the contract.

### (c) Tests that would shift

- **Zero behavior changes.** All three `top_port_resolves_generic_*` /
  `internal_net_resolves_generic_*` tests already encode this policy.
- A small new unit test in `src/codegen/mod.rs` confirming
  `resolve_port_type` leaves an unresolvable `RangeExpr::Expr`
  unchanged would be welcome but is not strictly required.

### (d) Impact on the GUI parameter override flow

- The override flow is *already* end-to-end correct under RESOLVE:
  `set_generic_map_entry` writes `inst.generic_map`, codegen reads it
  via `resolve_port_type`, and the user's `WIDTH=16` shows up as
  `[15:0]` in the wrapper's port list (visible confirmation).
- `resolve_port_width` and the pin width badge already use the same
  resolver, so the GUI display and the emitted text agree.
- No change.

### (e) Impact on round-trip tests

- The normalization step is correct; the comment in `tests/common/mod.rs`
  needs to be promoted from "workaround" framing to "policy" framing.
- New round-trip fixtures with parameterized widths are easy to add —
  the helpers handle the resolution automatically.

### (f) Risks

- **Confusion if undocumented.** Today a contributor reading
  `tests/common/mod.rs` reasonably wonders "is the codegen wrong, or is
  the test wrong?" because nothing in the codebase calls out RESOLVE as
  a deliberate choice. That is the chief risk and the chief deliverable
  of this change.
- **Loss of the symbolic form.** A downstream tool that *wants* the
  symbolic `[WIDTH-1:0]` (e.g. a documentation extractor or a
  parametric-IP exporter) cannot recover it from the generated text.
  Mitigation: if such a tool ever appears, it can read the schematic
  JSON, which still carries the symbolic form on `top_ports`.
- **Override semantics surprise.** A user who has *not* set an instance
  override sees the source module's default substituted into the
  wrapper. If they then `WIDTH=16` the override at the parent level
  (i.e. on the wrapper's own top generic), the wrapper's port list does
  not update because the wrapper's `top_generics` is independent of the
  child's defaults. This is a real edge case and is the strongest
  argument for PRESERVE — but it only bites users who try to
  parameterize the wrapper itself, which is unsupported feature work
  today (`Schematic::top_generics` exists but is not auto-populated;
  see `archive/2026-05-09-codegen-roundtrip/design.md` non-goal at
  line 18). Until top-generic auto-population lands, the surprise has
  no concrete user-facing trigger.

## Recommendation

**Adopt Policy B (RESOLVE) as the documented contract. Document, do not
re-architect.**

### Rationale grounded in the codebase

1. **The codegen unit tests already encode RESOLVE as the contract.**
   `top_port_resolves_generic_default`,
   `top_port_resolves_generic_override`, and
   `internal_net_resolves_generic_override` (`src/codegen/vhdl.rs:667-710`)
   each assert a literal width string. The test names use the word
   *resolves*. These tests were written before the round-trip change
   and represent the deliberate prior decision. PRESERVE would require
   rewriting them.

2. **The GUI parameter-override flow is already end-to-end RESOLVE.**
   `set_generic_map_entry` → `inst.generic_map` →
   `codegen::resolve_port_type` → emitted literal width. The bridge's
   `resolve_port_width` (`src/gui/bridge.rs:1993`) uses the same
   resolver for the canvas pin badge, so the displayed width and the
   emitted width are guaranteed to agree. PRESERVE breaks that
   alignment unless the GUI is rewired to also display symbolic forms.

3. **PRESERVE has no near-term enabler.** It would require auto-
   promoting child generics into `Schematic::top_generics` for the
   wrapper to remain legal HDL when symbolic widths are kept. That
   plumbing was explicitly called out as a non-goal in the just-
   archived `codegen-roundtrip` change
   (`archive/2026-05-09-codegen-roundtrip/design.md:18`). PRESERVE
   under today's pipeline would emit broken HDL whenever a child
   module's generic is not also a wrapper-level generic.

4. **The "round-trip masks a bug" framing is incorrect under closer
   reading.** The round-trip's normalization is comparing
   *equivalent-by-policy* port shapes, not papering over a defect. The
   test asserts what the policy says it should: that two `PortType`s
   resolve to the same literal range under the same substitutions.
   That is exactly the contract a structural codegen should pin.

The defect is one of **documentation**, not behavior. The follow-up
change should:

- Add a RESOLVE policy statement to the relevant codegen spec(s).
- Promote the `tests/common/mod.rs:91-97` doc comment from "necessary
  because the codegen itself resolves" to "consistent with the
  documented RESOLVE policy."
- Optionally add one positive unit test in `src/codegen/mod.rs`
  confirming that `RangeExpr::Expr` survives when no substitution is
  available (today's behavior, currently untested in isolation).

If, in the future, a concrete use case for symbolic-width preservation
appears (e.g. a parametric-IP exporter), it can be served by a separate
"emit unresolved" codegen mode rather than by flipping the default —
preserving today's GUI invariants and existing test coverage.
