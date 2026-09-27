# Milestone 5 Closeout: Lowering And Execution Readiness Boundary

## Status

Closed.

Milestone 5 now has a machine-checkable named certification surface and an
explicit closure record for what later milestones may assume about canonical
lowered forms, execution-ready admission, runtime readmission, and
representative executed-form lifecycle proof.

## Implemented Surface

- Canonical lowering, readiness, and execution carriers:
  - `Recipe<Lowered, T, B>`
  - `ExecutionReadyRecipe<T, B>`
  - `ExecutedRecipe<T, B>`
- Canonical readiness and execution transitions:
  - `AdmitExecutionReadyRecipeTransition`
  - `CheckedAdmitExecutionReadyRecipeTransition`
  - `ExecuteReadyRecipeTransition`
  - `admit_ready_and_execute_recipe(...)`
  - `checked_admit_ready_and_execute_recipe(...)`
- Canonical runtime-readmission and shifted-basis lowering recovery:
  - `LoweredReadmissionContext<...>`
  - `LoweredReadmissionReadiness<...>`
  - `ReadmitLoweredForExecutionReadyTransition`
  - `CheckedReadmitLoweredForExecutionReadyTransition`
  - `readmit_ready_and_execute_recipe(...)`
  - `checked_readmit_ready_and_execute_recipe(...)`
- Canonical readiness posture preserved from Milestone 4:
  - success
  - denied
  - deferred
  - stale
  - rebind-required
  - failed

## Certification Surface

Named suite:

- `Lowering And Execution Readiness Boundary Test`

Primary test:

- [lowering_and_execution_readiness_boundary_certification.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/lowering_and_execution_readiness_boundary_certification.rs)

Supporting evidence module:

- [tests/support/milestone5/mod.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/support/milestone5/mod.rs)

Machine-checkable outputs:

- `transition_digest`
- `basis_digest`
- `failure_digest`
- `compile_fail_bundle`
- `compile_pass_bundle`
- `codegen_honesty_report`
- `residual_debt_report`

## Hostile Coverage

The closeout suite now owns the hostile lanes required by the milestone:

- lowered recipes cannot execute without readiness admission
- resolved recipes cannot enter readiness admission before lowering
- boundary-bridged lowered recipes cannot skip readmission and enter readiness
- shifted-basis ready recipes cannot be treated as original-basis ready forms
- checked readiness preserves explicit denied, deferred, stale,
  rebind-required, and failed categories
- runtime admission facts terminate back into static ready or executed forms
  rather than ambient executor state
- direct current-basis ready execution and same-basis runtime-readmission
  execution converge on the same executed result
- shifted-basis runtime readmission diverges explicitly when the resulting basis
  is intentionally different

Compile-fail fixtures:

- [lowered_recipe_cannot_execute_without_readiness.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/ui/milestone5/compile_fail/lowered_recipe_cannot_execute_without_readiness.rs)
- [resolved_recipe_cannot_enter_execution_readiness.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/ui/milestone5/compile_fail/resolved_recipe_cannot_enter_execution_readiness.rs)
- [boundary_bridged_lowered_cannot_enter_execution_readiness.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/ui/milestone5/compile_fail/boundary_bridged_lowered_cannot_enter_execution_readiness.rs)
- [shifted_basis_ready_recipe_cannot_be_treated_as_original_basis.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/ui/milestone5/compile_fail/shifted_basis_ready_recipe_cannot_be_treated_as_original_basis.rs)

Compile-pass fixtures:

- [explicit_lowered_ready_executed_progression_compiles.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/ui/milestone5/compile_pass/explicit_lowered_ready_executed_progression_compiles.rs)
- [checked_readiness_progression_compiles.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/ui/milestone5/compile_pass/checked_readiness_progression_compiles.rs)
- [same_basis_runtime_readmission_progression_compiles.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/ui/milestone5/compile_pass/same_basis_runtime_readmission_progression_compiles.rs)
- [shifted_basis_readiness_progression_compiles.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/ui/milestone5/compile_pass/shifted_basis_readiness_progression_compiles.rs)

## Residual Debt

- Milestone 5 closes the canonical lowered-versus-ready execution boundary with
  representative executed-state and runtime-readmission hooks, but it does not
  yet certify multi-artifact composition pressure or cross-crate migration
  parity. Those belong to Milestone 6 and Milestone 7 respectively.

This is explicit sequencing debt, not a hidden Milestone 5 gap.

## Verification

Verified with:

- `cargo fmt -p worth-proof`
- `cargo test -p worth-proof lowering_and_execution_readiness_boundary_certification -- --nocapture`
- `cargo test -p worth-proof`
- `git diff --check`

## What Later Milestones May Assume

Milestone 6 and Milestone 7 may now assume:

- canonical lowered, ready, and executed proof-bearing wrappers already exist
- execution-facing APIs consume only execution-ready forms, never merely
  lowered forms
- runtime-gated readiness can terminate back into static proof-bearing ready or
  executed forms
- same-basis runtime readmission and direct ready execution can converge on the
  same executed result when their semantics are intentionally identical
- shifted-basis runtime readmission remains explicitly basis-distinct
- readiness categories preserve denied, deferred, stale, rebind-required, and
  failed divergence rather than collapsing into generic runtime folklore
- representative execution-boundary surfaces remain allocation-free and
  virtual-dispatch-free within the certified scope
