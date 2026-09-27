# Milestone 6 Closeout: Static Fork And Join Progression

## Status

Closed.

Milestone 6 now has a machine-checkable named certification surface and an
explicit closure record for what later milestones may assume about canonical
fixed-arity fork/join progression, multi-input short-circuit composition, and
deterministic same-family symbolic lowering.

## Implemented Surface

- Canonical fixed-arity composition carriers:
  - `ForkOutputs2<L, R>`
  - `JoinInputs2<L, R>`
- Canonical artifact-level composition helpers:
  - `fork_artifact_pair(...)`
  - `join_artifact_pair(...)`
- Canonical multi-input transition composition:
  - `compose_transition_outcome(...)`
  - `compose_success_transition(...)`
  - `compose_join_transition_outcome(...)`
  - `compose_join_success_transition(...)`
- Canonical lowered-versus-ready-sensitive join posture:
  - `join_ready_recipe_pair(...)`
  - `compose_join_ready_recipe_pair(...)`
- Canonical same-family symbolic and deterministic lowering surfaces:
  - `CompositionFamilySymbol<S>`
  - `AuthoritativeFamilyMember<A>`
  - `FamilyResolvedReference<S, A>`
  - `FamilyLifecycleAction<S, A, P>`
  - `LoweredFamilyProgram2<S, A, P>`
  - `resolve_family_symbol(...)`
  - `lower_deterministic_family_pair(...)`

## Certification Surface

Named suite:

- `Static Fork/Join And Composition Family Test`

Primary test:

- [static_fork_join_and_composition_family_certification.rs](/C:/Users/shepworth/Documents/programming/WORTH/crates/worth-proof/tests/static_fork_join_and_composition_family_certification.rs)

Supporting evidence module:

- [tests/support/milestone6/mod.rs](/C:/Users/shepworth/Documents/programming/WORTH/crates/worth-proof/tests/support/milestone6/mod.rs)

Supporting topology note:

- [tests/support/milestone6/README.md](/C:/Users/shepworth/Documents/programming/WORTH/crates/worth-proof/tests/support/milestone6/README.md)

Machine-checkable outputs:

- `transition_digest`
- `composition_digest`
- `proof_shape_digest`
- `failure_digest`
- `compile_fail_bundle`
- `compile_pass_bundle`
- `codegen_honesty_report`
- `residual_debt_report`

## Hostile Coverage

The closeout suite now owns the hostile lanes required by the milestone:

- raw broad join shapes cannot satisfy fixed-arity join contracts
- raw tuple-like fork substitutes cannot satisfy fixed-arity fork contracts
- lowered recipes cannot satisfy ready-only join surfaces
- left-lane denial skips right-lane evaluation in representative multi-input
  composition
- right-lane denial skips next-step execution in representative multi-input
  composition
- symbolic family references cannot enter authoritative-identity APIs unchanged
- same-family lifecycle pressure explicitly covers:
  - create
  - rewrite
  - supersede
  - retire
- deterministic create/retire family lowering converges for semantically
  equivalent reversed inputs
- deterministic rewrite/supersede family lowering converges for semantically
  equivalent reversed inputs
- representative fork/join and family-lowering carriers remain
  allocation-free and virtual-dispatch-free within the certified scope

Compile-fail fixtures:

- [raw_vec_cannot_satisfy_join_inputs.rs](/C:/Users/shepworth/Documents/programming/WORTH/crates/worth-proof/tests/ui/milestone6/compile_fail/raw_vec_cannot_satisfy_join_inputs.rs)
- [raw_tuple_cannot_satisfy_fork_outputs.rs](/C:/Users/shepworth/Documents/programming/WORTH/crates/worth-proof/tests/ui/milestone6/compile_fail/raw_tuple_cannot_satisfy_fork_outputs.rs)
- [lowered_recipe_cannot_satisfy_ready_join.rs](/C:/Users/shepworth/Documents/programming/WORTH/crates/worth-proof/tests/ui/milestone6/compile_fail/lowered_recipe_cannot_satisfy_ready_join.rs)
- [symbolic_family_reference_cannot_satisfy_authoritative_api.rs](/C:/Users/shepworth/Documents/programming/WORTH/crates/worth-proof/tests/ui/milestone6/compile_fail/symbolic_family_reference_cannot_satisfy_authoritative_api.rs)

Compile-pass fixtures:

- [explicit_fixed_arity_fork_join_progression_compiles.rs](/C:/Users/shepworth/Documents/programming/WORTH/crates/worth-proof/tests/ui/milestone6/compile_pass/explicit_fixed_arity_fork_join_progression_compiles.rs)
- [checked_multi_input_ordering_and_ready_join_compiles.rs](/C:/Users/shepworth/Documents/programming/WORTH/crates/worth-proof/tests/ui/milestone6/compile_pass/checked_multi_input_ordering_and_ready_join_compiles.rs)
- [explicit_family_symbol_resolution_and_lowering_compiles.rs](/C:/Users/shepworth/Documents/programming/WORTH/crates/worth-proof/tests/ui/milestone6/compile_pass/explicit_family_symbol_resolution_and_lowering_compiles.rs)

## Residual Debt

- Milestone 6 closes canonical fixed-arity fork/join progression and
  deterministic same-family lowering for representative static lanes, but it
  does not yet certify broader N-ary composition pressure or cross-crate
  migration parity. Those belong to Milestone 7.

This is explicit sequencing debt, not a hidden Milestone 6 gap.

## Verification

Verified with:

- `cargo fmt --all`
- `cargo test -p worth-proof`

## What Later Milestones May Assume

Milestone 7 may now assume:

- canonical fixed-arity fork and join carriers already exist
- representative multi-input composition preserves explicit non-success
  short-circuiting instead of hiding it behind a generic batch runtime
- Milestone 5 lowered-versus-ready boundaries remain authoritative even when
  several artifacts participate
- same-family composition-local symbols are distinct from authoritative family
  identity and cannot cross authority-only APIs unchanged
- representative same-family lifecycle pressure for create, rewrite,
  supersede, and retire is already encoded in one canonical family substrate
- deterministic family lowering no longer depends on caller iteration folklore
  after lowered family form exists
- representative static fork/join and family-lowering lanes remain
  allocation-free and virtual-dispatch-free within the certified scope
