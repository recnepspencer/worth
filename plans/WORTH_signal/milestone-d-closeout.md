# Milestone D Closeout Acceptance Map: Async Capability On Arbitrary Nodes

> **Status:** Completed
>
> **Spec:** [milestone-d-plan.md](./milestone-d-plan.md)
>
> **Roadmap parent:** [worth_signal_temporal_async_roadmap.md](./worth_signal_temporal_async_roadmap.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Prerequisite closeouts:**
> - [milestone-a-closeout.md](./milestone-a-closeout.md)
> - [milestone-b-closeout.md](./milestone-b-closeout.md)
> - [milestone-c-closeout.md](./milestone-c-closeout.md)

## Purpose

This document maps the Milestone D spec to concrete implementation,
certification, and hostile-test evidence.

It is the closeout ledger for the hostile question:

> Can `worth-signal` now treat async as a capability attachable to ordinary
> nodes, including keyed families, interior graph gates, and hierarchical async
> chains, without inventing a second truth model for lifecycle, replay,
> explanation, or restore?

## Closeout Summary

Milestone D is implemented as a capability-first async substrate attached to
ordinary node identities in `worth-signal`.

The implementation now includes:

- typed async-capability declarations, validation, frozen descriptors, lowered
  bundles, and compatibility alias proofs for ordinary `NodeId` values
- proof-bearing public handles for attached async-capable nodes and keyed
  async-capable family members
- condition-, temporal-, previous-value-, aspect-, and partition-aware async
  admission and revalidation that remain distinct from lifecycle truth
- first-class interior async gate state and downstream continuity semantics
- hierarchical async replay, cancellation, and historical parity artifacts
- capability-first historical parity and capability/legacy equivalence reports
  carrying replay, observation, diagnostics, and explanation lineage
- strict compile-time and typed-boundary enforcement around capability-only
  surfaces and sealed proof artifacts
- sealed Milestone D certification artifacts:
  - `AsyncNodeCompileTimeBoundaryProof`
  - `AsyncNodeMilestoneDScenarioMatrix`
  - `AsyncNodeMilestoneDPerformanceCloseout`
  - `AsyncNodeMilestoneDCertificationRun`
- direct strict-suite coverage for branch churn, public rediscovery, keyed
  lineage drift, interior gate visibility, hierarchy stale-authority races, and
  mixed restore/explanation pressure

The direct closeout gates are:

- `async_node_milestone_d_certification_run_builds_from_real_async_capability_reports`
- `async_node_milestone_d_certification_run_rejects_duplicate_scenario_coverage`
- `async_node_milestone_d_certification_run_rejects_WORTHd_performance_envelope`

Those tests prove the final certification run is built from real runtime
artifacts and fails closed when scenario ownership or performance-envelope
truth is WORTHd.

## Primary Implementation Surfaces

Capability model and public handles:

- [async_node/mod.rs](../../crates/worth-signal/src/data/async_node/mod.rs)
- [async_node/declaration.rs](../../crates/worth-signal/src/data/async_node/declaration.rs)
- [async_node/descriptor.rs](../../crates/worth-signal/src/data/async_node/descriptor.rs)
- [async_node/request.rs](../../crates/worth-signal/src/data/async_node/request.rs)
- [async_node/admission.rs](../../crates/worth-signal/src/data/async_node/admission.rs)
- [async_node/capable.rs](../../crates/worth-signal/src/data/async_node/capable.rs)
- [async_node/family.rs](../../crates/worth-signal/src/data/async_node/family.rs)
- [async_node/gate.rs](../../crates/worth-signal/src/data/async_node/gate.rs)
- [async_node/hierarchy.rs](../../crates/worth-signal/src/data/async_node/hierarchy.rs)

Historical parity, equivalence, and closeout artifacts:

- [async_node/history.rs](../../crates/worth-signal/src/data/async_node/history.rs)
- [async_node/equivalence.rs](../../crates/worth-signal/src/data/async_node/equivalence.rs)
- [async_node/keyed_history.rs](../../crates/worth-signal/src/data/async_node/keyed_history.rs)
- [async_node/keyed_equivalence.rs](../../crates/worth-signal/src/data/async_node/keyed_equivalence.rs)
- [async_node/hierarchy_history.rs](../../crates/worth-signal/src/data/async_node/hierarchy_history.rs)
- [async_node/certification/mod.rs](../../crates/worth-signal/src/data/async_node/certification/mod.rs)
- [async_node/certification/compile_time.rs](../../crates/worth-signal/src/data/async_node/certification/compile_time.rs)
- [async_node/certification/matrix.rs](../../crates/worth-signal/src/data/async_node/certification/matrix.rs)
- [async_node/certification/performance.rs](../../crates/worth-signal/src/data/async_node/certification/performance.rs)
- [async_node/certification/run.rs](../../crates/worth-signal/src/data/async_node/certification/run.rs)

Runtime ownership and lowering:

- [async_keyed.rs](../../crates/worth-signal/src/logic/transaction/runtime/async_keyed.rs)
- [async_capability/mod.rs](../../crates/worth-signal/src/logic/transaction/runtime/state/async_capability/mod.rs)
- [async_capability/declaration.rs](../../crates/worth-signal/src/logic/transaction/runtime/state/async_capability/declaration.rs)
- [async_capability/admission.rs](../../crates/worth-signal/src/logic/transaction/runtime/state/async_capability/admission.rs)
- [async_capability/gate.rs](../../crates/worth-signal/src/logic/transaction/runtime/state/async_capability/gate.rs)
- [async_capability/hierarchy.rs](../../crates/worth-signal/src/logic/transaction/runtime/state/async_capability/hierarchy.rs)
- [async_capability/history.rs](../../crates/worth-signal/src/logic/transaction/runtime/state/async_capability/history.rs)
- [async_capability/equivalence.rs](../../crates/worth-signal/src/logic/transaction/runtime/state/async_capability/equivalence.rs)
- [async_capability/keyed_history.rs](../../crates/worth-signal/src/logic/transaction/runtime/state/async_capability/keyed_history.rs)
- [async_capability/keyed_equivalence.rs](../../crates/worth-signal/src/logic/transaction/runtime/state/async_capability/keyed_equivalence.rs)
- [async_capability/hierarchy_history.rs](../../crates/worth-signal/src/logic/transaction/runtime/state/async_capability/hierarchy_history.rs)
- [runtime_state.rs](../../crates/worth-signal/src/logic/transaction/runtime/state/runtime_state.rs)
- [resource.rs](../../crates/worth-signal/src/logic/transaction/runtime/state/resource.rs)
- [summary.rs](../../crates/worth-signal/src/data/resource/summary.rs)
- [telemetry.rs](../../crates/worth-signal/src/data/telemetry.rs)
- [facade.rs](../../crates/worth-signal/src/facade.rs)

Test and compile-fail surfaces:

- [async_node_runtime.rs](../../crates/worth-signal/src/tests/async_node_runtime.rs)
- [async_node_family_runtime.rs](../../crates/worth-signal/src/tests/async_node_family_runtime.rs)
- [async_node_gate_runtime.rs](../../crates/worth-signal/src/tests/async_node_gate_runtime.rs)
- [async_node_gate_visibility_runtime.rs](../../crates/worth-signal/src/tests/async_node_gate_visibility_runtime.rs)
- [async_node_gate_churn_runtime.rs](../../crates/worth-signal/src/tests/async_node_gate_churn_runtime.rs)
- [async_node_hierarchy_runtime.rs](../../crates/worth-signal/src/tests/async_node_hierarchy_runtime.rs)
- [async_node_hierarchy_race_runtime.rs](../../crates/worth-signal/src/tests/async_node_hierarchy_race_runtime.rs)
- [async_node_history_runtime.rs](../../crates/worth-signal/src/tests/async_node_history_runtime.rs)
- [async_node_equivalence_runtime.rs](../../crates/worth-signal/src/tests/async_node_equivalence_runtime.rs)
- [async_node_closeout_runtime.rs](../../crates/worth-signal/src/tests/async_node_closeout_runtime.rs)
- [async_node_public_api_runtime.rs](../../crates/worth-signal/src/tests/async_node_public_api_runtime.rs)
- [async_node_public_api_restore_runtime.rs](../../crates/worth-signal/src/tests/async_node_public_api_restore_runtime.rs)
- [async_node_public_branch_churn_runtime.rs](../../crates/worth-signal/src/tests/async_node_public_branch_churn_runtime.rs)
- [async_node_public_hierarchy_branch_runtime.rs](../../crates/worth-signal/src/tests/async_node_public_hierarchy_branch_runtime.rs)
- [async_node_restore_lineage_runtime.rs](../../crates/worth-signal/src/tests/async_node_restore_lineage_runtime.rs)
- [async_node_nightmare_runtime.rs](../../crates/worth-signal/src/tests/async_node_nightmare_runtime.rs)
- [async_node_certification_runtime.rs](../../crates/worth-signal/src/tests/async_node_certification_runtime.rs)
- [resource_api.rs](../../crates/worth-signal/src/tests/resource_api.rs)
- [async-node compile-fail fixtures](../../crates/worth-signal/tests/ui)

## Must-Ship Acceptance Map

| Spec requirement | Implementation evidence | Certification / test evidence |
| --- | --- | --- |
| Explicit async-capability attachment on ordinary nodes | typed capability declaration, validation, freeze, lower, and attach surfaces rooted in ordinary `NodeId` values | `async_node_capability_declaration_lowers_into_runtime_owned_descriptor`; `async_node_capability_equivalence_report_matches_legacy_runtime_truth_for_rich_leaf_workload` |
| Capability-aware node declaration and public vocabulary | `AsyncCapableNode`, keyed attach/rediscovery helpers, node-first request/revalidation intents | `attach_async_capability_returns_handle_that_owns_public_intent_building`; `keyed_attach_async_capability_reads_like_node_capability_not_subsystem_switch` |
| Condition-, temporal-, and previous-value-aware async admission | typed admission/revalidation classifications distinct from lifecycle state | `async_node_condition_gated_request_blocks_without_mutating_lifecycle_truth`; `async_node_temporal_request_admission_blocks_until_clock_reaches_ready_tick`; `async_node_previous_value_drift_blocks_request_admission`; `async_node_revalidation_can_refresh_when_new_lineage_condition_is_blocked` |
| Aspect-/partition-aware async capability | aspect-local and partition-local refresh classification and bounded locality telemetry | `async_node_partition_local_revalidation_blocks_when_changed_region_misses_contract_scope`; `async_node_partition_local_revalidation_matches_contract_scope_and_records_locality` |
| Interior async graph gates | gate-state reports, continuity visibility policy, legality drift, and downstream dependency truth | gate runtime, visibility, churn, public branch-churn, and combined nightmare suites listed below |
| Hierarchical async composition | replay summaries, cancellation footprints, hierarchy historical parity, and restore-local cancellation law | hierarchy runtime, race, closeout, public hierarchy branch, and combined nightmare suites listed below |
| Historical parity and legacy compatibility | historical parity, keyed historical parity, capability equivalence, keyed equivalence, hierarchy historical parity | history/equivalence/closeout runtime suites plus certification rows |
| Compile-time and typed-boundary enforcement | sealed proof artifacts, private constructors/fields, typed denials before execution | `resource_compile_fail_boundaries_hold`; `async_node_milestone_d_compile_time_boundary_proof_rejects_missing_required_fixture` |
| Final milestone-level certification run | sealed scenario matrix, performance closeout, and final run built from real reports | `async_node_milestone_d_certification_run_builds_from_real_async_capability_reports`; WORTHd-row and WORTHd-envelope rejection tests |

## Milestone D Certification Surface

Milestone D closes with four milestone-level proof artifacts:

- `AsyncNodeCompileTimeBoundaryProof`
- `AsyncNodeMilestoneDScenarioMatrix`
- `AsyncNodeMilestoneDPerformanceCloseout`
- `AsyncNodeMilestoneDCertificationRun`

The runtime-backed proof tests that guard these surfaces are:

- `async_node_milestone_d_certification_run_builds_from_real_async_capability_reports`
- `async_node_milestone_d_compile_time_boundary_proof_rejects_missing_required_fixture`
- `async_node_milestone_d_scenario_matrix_rejects_gate_historical_node_mismatch`
- `async_node_milestone_d_scenario_matrix_rejects_keyed_explanation_lineage_drift`
- `async_node_milestone_d_certification_run_rejects_duplicate_scenario_coverage`
- `async_node_milestone_d_certification_run_rejects_WORTHd_performance_envelope`

These tests ensure the final milestone artifact is not merely row-shaped:

- scenario coverage must be exact
- compile-time fixture ownership must be exact
- gate/historical/keyed explanation evidence must agree semantically
- performance rows must bind real named boundary envelopes

## Crucial Requirement Coverage Map

Milestone D directly introduces `test-requirements.md` clauses `21` through
`25`. This section makes the direct-vs-combined ownership explicit, as required
by section `14.2` of the spec.

### Directly blocking and directly satisfied

| Requirement | Owning suite(s) |
| --- | --- |
| `21. Async capability attachment equivalence test` | `async_node_capability_equivalence_report_matches_legacy_runtime_truth_for_rich_leaf_workload`; `async_node_capability_alias_lowering_matches_legacy_resource_truth`; `async_capable_node_public_rediscovery_after_restore_preserves_parity_and_explanation_truth`; `keyed_public_handles_fail_closed_after_restore_rebind_and_require_rediscovered_lineage` |
| `22. Interior async node gate equivalence test` | `async_node_interior_gate_report_tracks_dependency_shape_and_restores_identically`; `async_node_interior_gate_pending_visibility_reflects_output_continuity_policy`; `async_node_interior_gate_rejection_visibility_changes_without_forging_lifecycle_truth`; `async_node_interior_gate_timeout_visibility_reflects_output_continuity_policy`; `async_node_active_gate_legality_drift_revalidates_without_new_lineage_and_replays_after_restore`; `public_async_gate_rediscovery_is_branch_local_and_visibility_honest_under_restore_churn` |
| `23. Hierarchical async capability replay and cancellation test` | `async_node_hierarchy_cancellation_propagates_and_replay_summary_restores_identically`; `async_node_hierarchy_restore_is_branch_local_and_checkpoint_honest`; `async_node_hierarchy_late_descendant_completion_switches_from_cancelled_to_stale_across_restore`; `async_node_hierarchy_historical_parity_report_preserves_restore_honesty`; `public_rediscovery_keeps_gate_visibility_and_hierarchy_explanations_branch_honest` |
| `25. Async capability compile-time boundary test` | `resource_compile_fail_boundaries_hold`; `async_node_milestone_d_compile_time_boundary_proof_rejects_missing_required_fixture` |

### Important requirements satisfied through stronger direct and combined suites

| Requirement | Owning suite(s) |
| --- | --- |
| `24. Condition-gated async admission parity test` | directly satisfied by `async_node_condition_gated_request_blocks_without_mutating_lifecycle_truth`, `async_node_temporal_request_admission_blocks_until_clock_reaches_ready_tick`, `async_node_previous_value_drift_blocks_request_admission`, and `async_node_revalidation_can_refresh_when_new_lineage_condition_is_blocked`; strengthened by `async_node_active_gate_legality_drift_revalidates_without_new_lineage_and_replays_after_restore` and `async_node_nightmare_workload_preserves_combined_capability_truth_across_restore` |
| `Aspect-Scoped Async Capability Test` | directly satisfied by `async_node_partition_local_revalidation_blocks_when_changed_region_misses_contract_scope` and `async_node_partition_local_revalidation_matches_contract_scope_and_records_locality`; strengthened by the combined nightmare and milestone certification rows |
| `Previous-Value And Temporal Async Capability Parity Test` | directly satisfied by the temporal and previous-value admission tests in `async_node_runtime.rs`; strengthened by the legality-drift gate churn lane and both nightmare workloads |
| `Legacy Resource Alias Compatibility Test` | directly satisfied by `async_node_capability_alias_lowering_matches_legacy_resource_truth`; strengthened by history/equivalence/public rediscovery suites and the milestone certification matrix |

## Direct Strict-Suite Ownership

The strict Milestone D suites that turned the milestone from â€œfeature-completeâ€
into â€œcloseout-completeâ€ are:

- `async_node_nightmare_workload_preserves_combined_capability_truth_across_restore`
- `async_node_nightmare_restore_lineage_keeps_hierarchy_honest_and_rebinds_keyed_explanations`
- `public_async_gate_rediscovery_is_branch_local_and_visibility_honest_under_restore_churn`
- `public_rediscovery_keeps_gate_visibility_and_hierarchy_explanations_branch_honest`

Together they force the hardest combined seams to share one coherent truth
across:

- capability-first vs legacy lowering
- keyed/family-local lineage and rebind churn
- interior async gates in the middle of the graph
- hierarchical cancellation and restore
- public handle rediscovery
- observation and explanation continuity
- branch-local restore and replay honesty

## Compile-Time Boundary Map

Milestone D compile-time and typed-boundary proof is owned by:

- `resource_compile_fail_boundaries_hold`
- `AsyncNodeCompileTimeBoundaryProof`
- the async-node `tests/ui` fixtures covering:
  - sealed declaration / descriptor / lowered bundle types
  - sealed public handles and keyed bindings
  - sealed gate / hierarchy / history / equivalence reports
  - sealed milestone-level certification artifacts
  - private request/revalidation intent constructors

That closes the spec requirement that capability-only surfaces not be reachable
through ordinary node declarations, WORTHd lowered artifacts, or bypassed
compatibility aliases.

## Final Verification

The final Milestone D verification surface is:

- `cargo test -p worth-signal async_node_public_hierarchy_branch_ -- --nocapture`
- `cargo test -p worth-signal async_node_milestone_d_ -- --nocapture`
- `cargo test -p worth-signal async_node_ -- --nocapture`
- `cargo test -p worth-signal resource_compile_fail_boundaries_hold -- --nocapture`
- `cargo test -p worth-signal`

Most recent result:

- `cargo test -p worth-signal` -> **923 passed, 23 ignored**

`git diff --check` is clean apart from the pre-existing LF/CRLF warnings.

## Closeout Decision

Milestone D is closed.

The runtime now supports async as an attachable node capability rather than as
a mentally separate node species, while preserving:

- separate graph dirtiness and async lifecycle truth
- honest condition/temporal/previous-value admission semantics
- bounded aspect/partition locality
- interior graph-gate behavior without hidden dependency models
- hierarchical replay/restore/cancellation law
- capability-first and legacy compatibility parity
- proof-bearing compile-time and milestone-level certification boundaries

Remaining work belongs to future product-facing layers and future specs, not to
Milestone D substrate closure.

## Successor Handoff

[Milestone 12 - Aspect-Causal Invalidation](./milestone-12-plan.md) is the next
numbered core-runtime milestone. It inherits Milestone D's condition, temporal,
previous-value, partition, branch, replay, and async-capability guarantees and
repairs the shared aspect-invalidation substrate without redefining async
lifecycle truth.
