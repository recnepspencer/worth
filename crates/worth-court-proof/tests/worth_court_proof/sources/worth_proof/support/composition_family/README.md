Milestone 6 support topology

- `compile.rs` owns the named compile-fail and compile-pass matrices for the
  static fork/join and composition-family suite.
- `digests.rs` owns proof extraction for transition, composition/proof-shape,
  and failure digests.
- `representatives.rs` owns the representative type aliases used by digest and
  codegen certification.
- `codegen.rs` owns the representative size/layout/drop honesty report for the
  fixed-arity and family-lowering lanes.
- `closeout.rs` owns the residual debt surface for the milestone.
- `mod.rs` is only the thin facade that re-exports the milestone-local support
  contracts.

This split is intentional:
- compile matrices stay separate from proof extraction
- proof extraction stays separate from representative fixture naming
- codegen proof stays separate from residual debt/closeout vocabulary
