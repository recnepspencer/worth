# Milestone 9.3.3 Closeout: Authority-Scoped Effect Execution Pipeline

## Status

Milestone 9.3.3 is closed as of 2026-05-13 for the Query-owned
authority-scoped effect execution pipeline in `worth-query`, with execution
authority preserved through existing relational and runtime-bridge facades.

This closeout covers:

- phase-typed effect progression from `RawEffectIntent` through normalized
  intent, typed eligibility, admitted effect proof, authority-scoped planning,
  lowered execution plans, execution receipts, self-describing envelopes, and
  unified certification
- runtime-backed relational mutation, relational merge, bridge-backed
  writeback, and ordered batch execution lanes under one shared proof chain
- typed denial, rebind-required, deferred, advisory, mismatch, and hostile
  execution postures before lower-runtime execution occurs
- receipt-first execution outcomes with transition rules, source-bearing
  envelopes, diagnostics materialization, and explicit deferred neighbors for
  replay/projection/export follow-ons
- exact counters and slope digests for normalization, eligibility, lowering,
  execution, receipt materialization, envelope materialization, and support
  lookup, including explicit `executor_rediscovery_count`,
  `batch_lowering_count`, `batch_basis_reuse_count`, and
  `authority_reopen_count`
- independent oracle verification for relational and bridge-backed execution
  plus seeded replay/parity certification
- compile-fail boundaries preventing external construction of normalized
  intents, admitted effects, authority-scoped plans, lowered execution plans,
  receipts, envelopes, support rows, and certification bundles
- one public production certification gate:
  `certify_effect_execution_pipeline()`

This closeout does not claim store-backed effect execution parity, durable
workflow continuation, restart-stable effect envelopes, portable receipt
import/export, or durable replay. Those remain Milestone 10, Milestone 11, or
later scope exactly as the 9.3.3 spec declared.

## Governing Source Summary

- `MENTALITY.md`: closure required making executor-side rediscovery impossible,
  not merely discouraged.
- `arch_laws.md`: closure required planner-owned decisions, executor-owned
  consumption, typed receipts, and compile-time proof boundaries.
- `composition_laws.md`: closure required separate responsibility for
  normalization, eligibility, planning, lowering, execution, receipt shaping,
  and certification.
- `domain_structure_laws.md`: closure required effect planning, execution,
  receipt/envelope shaping, oracle verification, and certification to remain
  physically locatable.
- `perf_laws.md`: closure required exact counters and slope evidence for every
  claimed bounded phase and explicit anti-rediscovery accounting.
- `dx_laws.md`: closure required family-complete public stories, including
  bridge-backed writeback as a first-class common path rather than a hidden
  lower-runtime escape hatch.
- `milestone-9.3.3.md`: the shipped surface now satisfies the phase contract,
  finished-surface contract, required verification outputs, and closeout
  standard.

## Adversarial Constraint Closed

Milestone 9.3.3 had to survive downstream callers attempting to execute
mutation, merge, batch, or writeback work by supplying raw effect intent,
ambient basis, preview-local posture, host-selected strategy, or ad hoc
artifact policy.

The closed surface enforces one typed progression:

1. `RawEffectIntent`
2. `NormalizedEffectIntent`
3. `EffectAuthorityEligibility` or typed non-admitted posture
4. `AdmittedEffectIntent`
5. `AuthorityScopedEffectPlan`
6. `LoweredEffectExecutionPlan`
7. `EffectExecutionReceipt`
8. `SelfDescribingEffectEnvelope`
9. `EffectExecutionCertificationBundle`

Query now owns public effect normalization, eligibility, planning, lowering
orchestration, receipt shaping, envelope derivation, support metadata, DX
inventory, oracle verification, and certification. It does not mint relational
truth mutation semantics, merge semantics, preview lifecycle semantics, bridge
writeback semantics, or durable replay behavior locally.

## Shipped Scope

Milestone 9.3.3 delivered:

- effect lifecycle implementation in
  [crates/worth-query/src/effect_lifecycle](../../crates/worth-query/src/effect_lifecycle)
- unified closeout certification in
  [crates/worth-query/src/effect_lifecycle/certification/closeout](../../crates/worth-query/src/effect_lifecycle/certification/closeout)
- independent relational and bridge oracle verification in
  [crates/worth-query/src/effect_lifecycle/oracle](../../crates/worth-query/src/effect_lifecycle/oracle)
- public-surface inventory, support matrix, and DX closure in
  [inventory.rs](../../crates/worth-query/src/effect_lifecycle/inventory.rs),
  [inventory_rows.rs](../../crates/worth-query/src/effect_lifecycle/inventory_rows.rs),
  [support_matrix.rs](../../crates/worth-query/src/effect_lifecycle/support_matrix.rs), and
  [support_contract.rs](../../crates/worth-query/src/effect_lifecycle/support_contract.rs)
- receipt, envelope, diagnostics, and transition rules in
  [receipt.rs](../../crates/worth-query/src/effect_lifecycle/receipt.rs),
  [envelope.rs](../../crates/worth-query/src/effect_lifecycle/envelope.rs),
  [diagnostics.rs](../../crates/worth-query/src/effect_lifecycle/diagnostics.rs), and
  [receipt_transitions.rs](../../crates/worth-query/src/effect_lifecycle/receipt_transitions.rs)
- facade exports in
  [crates/worth-query/src/facade/exports_foundation.rs](../../crates/worth-query/src/facade/exports_foundation.rs)
- compile-fail proof boundaries in
  [crates/worth-query/tests/phase_boundaries_effect_lifecycle_compile_fail.rs](../../crates/worth-query/tests/phase_boundaries_effect_lifecycle_compile_fail.rs) and
  [crates/worth-query/tests/ui/effect_lifecycle](../../crates/worth-query/tests/ui/effect_lifecycle)

## Acceptance Mapping

Milestone 9.3.3 is considered closed against:

- [milestone-9.3.3.md](./milestone-9.3.3.md)
- [worth_query_roadmap.md](./worth_query_roadmap.md)
- [worth_query_vision.md](./worth_query_vision.md)
- [test-requirements.md](./test-requirements.md)
- [test-requirements-milestone-9_3-and-runtime-gates.md](./test-requirements-milestone-9_3-and-runtime-gates.md)
- [milestone-9.3.2-closeout.md](./milestone-9.3.2-closeout.md)

because Query effect execution is now represented by one proof-bearing
pipeline with machine-checkable certification, oracle verification, receipt
closure, and family-complete public DX evidence.

### `Authority-Scoped Effect Execution Pipeline Test`

Covered by:

- [crates/worth-query/src/effect_lifecycle/tests/authoring](../../crates/worth-query/src/effect_lifecycle/tests/authoring)
- [crates/worth-query/src/effect_lifecycle/tests/batch](../../crates/worth-query/src/effect_lifecycle/tests/batch)
- [crates/worth-query/src/effect_lifecycle/tests/execution](../../crates/worth-query/src/effect_lifecycle/tests/execution)
- [crates/worth-query/src/effect_lifecycle/tests/closeout](../../crates/worth-query/src/effect_lifecycle/tests/closeout)
- [crates/worth-query/src/effect_lifecycle/tests/support.rs](../../crates/worth-query/src/effect_lifecycle/tests/support.rs)
- [crates/worth-query/tests/phase_boundaries_effect_lifecycle_compile_fail.rs](../../crates/worth-query/tests/phase_boundaries_effect_lifecycle_compile_fail.rs)
- [crates/worth-query/tests/ui/effect_lifecycle](../../crates/worth-query/tests/ui/effect_lifecycle)

What is proven:

- equivalent effect authoring paths normalize to the same admitted and lowered
  execution meaning
- intentionally different family, authority lane, basis lane, scope, policy,
  or strategy identities change the declared digest fields
- stale, preview-read-only, authority-mismatched, deferred, unsupported,
  rebind-required, and host-override attempts fail typed and early before
  execution artifacts exist
- ordinary callers cannot execute anything weaker than
  `LoweredEffectExecutionPlan`
- ordered batch execution lowers once per batch, denies mixed-lane batches
  before execution, and does not fall back to scalar basis re-admission
- support/discovery remains a first-class certified surface rather than a doc
  habit, and support metadata stays synchronized with executable behavior
- receipts are the canonical operational artifact, envelopes are derived from
  receipts, and diagnostics are derived from receipt-plus-authority evidence
- relational and bridge oracle lanes agree with Query-shaped execution receipts
- the unified closeout bundle emits the full required verification output set,
  including DX, oracle, replay, proof-shape, boundary, failure, and slope
  evidence
- public callers cannot construct proof-bearing lifecycle artifacts or bypass
  the single public production certification gate

## Final Audit Finding Closed

The final closeout audit found two remaining DX-certification honesty gaps:

- the writeback golden transcript claimed a branch-head basis story even though
  the runtime-backed writeback evidence was tenant-scoped
- the public-surface inventory did not have an explicit writeback common-path
  row, so target-DX certification could not fail if writeback common-path
  support regressed

That gap is closed by:

- binding the writeback common-path transcript to the real tenant-scoped
  runtime-backed evidence
- adding `EffectPublicSurfaceKind::WritebackCommonPath` to the public-surface
  inventory
- requiring that row in the target-DX digest
- adding certification and inventory tests that fail if writeback common-path
  support disappears or drifts from the certified transcript

## Closeout Standard

The Milestone 9.3.3 closeout standard is satisfied because:

- the spec phases were implemented in order across authoring, eligibility,
  planning, lowering, execution, receipt/envelope shaping, and certification
- every ordinary admitted runtime-backed Query effect surface executes from a
  lowered plan rather than from raw intent, ambient basis, or executor-local
  rediscovery
- admitted relational, merge, bridge writeback, and ordered batch lanes share
  one proof chain from raw intent through receipt and envelope
- support metadata, executable behavior, DX transcripts, and certification
  coverage agree for admitted, advisory, denied, deferred, mismatch, and
  rebind-required postures
- compile-fail boundaries prove public callers cannot mint proof-bearing
  effect lifecycle artifacts directly
- performance counters and slope digests are enforced for every claimed phase,
  including anti-rediscovery and anti-scalarization counters
- roadmap and test-requirement references now point at the 9.3.3 spec and
  this closeout record accurately
- store-backed, durable, and portable follow-ons remain explicit later-
  milestone scope rather than hidden debt

## Verification Baseline

The closeout state is verified by:

- `cargo fmt --package worth-query`
- `cargo test -p worth-query effect_lifecycle::tests::certification --quiet`
- `cargo test -p worth-query effect_lifecycle::tests::inventory --quiet`
- `cargo test -p worth-query effect_lifecycle::tests::oracle --quiet`
- `cargo test -p worth-query effect_lifecycle --quiet`
- `cargo test -p worth-query --test phase_boundaries_effect_lifecycle_compile_fail --quiet`
- `cargo test -p worth-query --quiet`

## Deferred Scope That Remains Explicitly Deferred

The following are not part of Milestone 9.3.3 closeout:

- store-backed effect execution parity
- durable workflow continuation
- durable replay
- restart-stable effect envelopes
- portable effect receipt import/export

These remain Milestone 10, Milestone 11, or later effect-lifecycle scope
exactly as the milestone spec declared.
