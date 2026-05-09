## Why

The codegen-roundtrip integration test (just merged at `7465a4e`) surfaced a
behavioral asymmetry between the parser and the codegen for vector port
widths derived from generic / parameter expressions:

- The **parser** preserves the symbolic form when it cannot fully evaluate
  the expression. A SystemVerilog port declared `[WIDTH-1:0]` parses to
  `PortType::StdLogicVector { high: RangeExpr::Expr("WIDTH-1"), low:
  RangeExpr::Literal(0) }` (see `try_eval_expr` in `src/parser/vhdl.rs:222`
  and `eval_simple_expr` in `src/parser/verilog.rs:57`).
- The **codegen** eagerly substitutes the source module's default generic
  value (or any per-instance override from `inst.generic_map`) and emits a
  literal range. `WIDTH=8` default becomes `[7:0]` in the generated wrapper
  text. See `codegen::resolve_port_type` in `src/codegen/mod.rs:152` and
  its three call sites — `collect_internal_nets`,
  `collect_top_intermediates`, and the per-language top-port emit blocks
  in `src/codegen/sv.rs:128` and `src/codegen/vhdl.rs:157`.

The round-trip test masks the asymmetry by normalizing both the original
parsed `PortDef` and the regenerated `PortDef` through `resolve_port_type`
against the same generics before comparison (see `tests/common/mod.rs:91`
and the design.md decision history at
`openspec/changes/archive/2026-05-09-codegen-roundtrip/design.md`).

The masked asymmetry is a *scoping decision*, not a bug to fix in place:

- If "preserve symbolic widths through codegen" is the intended product
  contract — so a downstream synthesizer or human reader sees `[WIDTH-1:0]`
  rather than a baked-in literal — then eager resolution is a real defect
  the round-trip test hides, and `resolve_port_type` should be narrowed.
- If eager resolution is intentional — so the generated wrapper is
  self-contained, the literal width is unambiguous to a tool that doesn't
  resolve generics, and the user's instance-level override actually shows
  up in the emitted text — then the round-trip's normalization step is
  correct as written and we should document the contract so future
  contributors don't try to "fix" it.

The decision matters because both interpretations have real downstream
consequences for the GUI parameter-override flow (`set_generic_map_entry`
in `src/gui/bridge.rs:1654`), the bridge's width readout
(`resolve_port_width` in `src/gui/bridge.rs:1993`), and the way new
codegen fixtures should be designed.

## What Changes

This change captures a **decision point**, not a unit of work:

- Frame the question in `proposal.md`.
- Enumerate the two policies — PRESERVE (lazy / symbolic) and RESOLVE
  (eager / literal) — in `design.md` with concrete file-level impact for
  each.
- Recommend one policy with rationale grounded in the existing codebase
  (not preference).

The follow-up implementation lives in a separate change once the user
picks a policy. This change does **not** include `specs/` or `tasks.md`:
no requirement is being added or modified yet, and there is no work to
schedule until a policy is chosen.

## Capabilities

### New Capabilities

- None. This is a decision artifact.

### Modified Capabilities

- None. The relevant capability (`sv-codegen` / `vhdl-codegen` /
  `codegen-roundtrip`) will be amended in a follow-up change once a
  policy is selected.

## Impact

- **Specs**: no change in this proposal. The follow-up change will amend
  one of `sv-codegen`, `vhdl-codegen`, or `codegen-roundtrip` depending
  on the policy chosen.
- **Source**: zero edits. Investigation only.
- **Tests**: zero edits. The existing round-trip normalization stays in
  place until a policy is picked; if PRESERVE wins, the normalization
  will be removed by the follow-up. If RESOLVE wins, the normalization
  is correct and will be documented as such.
- **GUI**: zero edits. The `set_generic_map_entry` plumbing already
  feeds `inst.generic_map` into `resolve_port_type` — both policies keep
  that wiring; only the *output* of the resolver at emit time is
  affected.
