# Milestone A Closeout Acceptance Map: Temporal Runtime Substrate

> **Status:** Closeout candidate
>
> **Spec:** [milestone-a-plan.md](./milestone-a-plan.md)
>
> **Roadmap parent:** [worth_signal_temporal_async_roadmap.md](./worth_signal_temporal_async_roadmap.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Prerequisite closeout:** [milestone-11-closeout.md](./milestone-11-closeout.md)

## Purpose

This document maps the Milestone A spec to concrete implementation and
certification evidence.

It is intentionally not a product announcement. It is the closeout ledger for
the hostile question:

> Can `worth-signal` now own time as deterministic, replay-honest runtime
> truth instead of treating time as host callback folklore?

## Closeout Summary

Milestone A is implemented as a core temporal substrate in `worth-signal`.

The implementation now includes:

- runtime-owned monotonic clock basis and validated clock advance requests
- sealed temporal policy vocabulary for `After`, `AtOrAfter`, `Debounce`,
  `Throttle`, `StaleAfter`, and `Interval`
- runtime-owned scheduled, ready, retired, rescheduled, and reused wake state
- deterministic due and ready frontier ordering
- node-owned temporal wake admission from declared temporal conditions
- temporal lowering artifacts consumed by execution rather than resolver
  callbacks
- transaction-staged temporal evidence
- previous-value access gated by ready temporal wake capability and branch epoch
- branch and snapshot restore of temporal state from retained runtime state
- reconstructability and certification artifacts with canonical digests
- diagnostics-visible temporal provenance and cost contracts
- named temporal counters and named prohibited performance failure modes

## Primary Implementation Surfaces

Temporal type vocabulary:

- [clock.rs](../../crates/worth-signal/src/data/temporal/clock.rs)
- [condition.rs](../../crates/worth-signal/src/data/temporal/condition.rs)
- [units.rs](../../crates/worth-signal/src/data/temporal/units.rs)
- [wake.rs](../../crates/worth-signal/src/data/temporal/wake.rs)
- [frontier.rs](../../crates/worth-signal/src/data/temporal/frontier.rs)
- [eligibility.rs](../../crates/worth-signal/src/data/temporal/eligibility.rs)
- [previous_value.rs](../../crates/worth-signal/src/data/temporal/previous_value.rs)

Runtime ownership and lifecycle:

- [temporal.rs](../../crates/worth-signal/src/logic/transaction/runtime/state/temporal.rs)
- [transaction_evaluation.rs](../../crates/worth-signal/src/logic/transaction/runtime/execution/transaction_evaluation.rs)
- [runtime_execution.rs](../../crates/worth-signal/src/logic/transaction/runtime/execution/runtime_execution.rs)
- [transaction_keyed.rs](../../crates/worth-signal/src/logic/transaction/runtime/execution/transaction_keyed.rs)
- [branches.rs](../../crates/worth-signal/src/logic/transaction/runtime/state/branching/branches.rs)
- [snapshotting.rs](../../crates/worth-signal/src/logic/transaction/runtime/state/branching/snapshotting.rs)

Lowering, transaction evidence, and reconstructability:

- [precompute/mod.rs](../../crates/worth-signal/src/logic/planner/precompute/mod.rs)
- [transaction_types.rs](../../crates/worth-signal/src/logic/transaction/runtime/transaction/transaction_types.rs)
- [finalize.rs](../../crates/worth-signal/src/logic/transaction/runtime/transaction/transaction_commit/finalize.rs)
- [reconstructability.rs](../../crates/worth-signal/src/logic/transaction/runtime/state/reconstructability.rs)

Diagnostics, facade, and counters:

- [telemetry.rs](../../crates/worth-signal/src/data/telemetry.rs)
- [summary.rs](../../crates/worth-signal/src/diagnostics/model/summary.rs)
- [observer.rs](../../crates/worth-signal/src/logic/transaction/runtime/state/observer.rs)
- [facade.rs](../../crates/worth-signal/src/facade.rs)

Certification tests:

- [temporal_runtime.rs](../../crates/worth-signal/src/tests/temporal_runtime.rs)
- [temporal_api.rs](../../crates/worth-signal/src/tests/temporal_api.rs)
- [temporal compile-fail fixtures](../../crates/worth-signal/tests/ui)

## Must-Ship Acceptance Map

| Spec requirement | Implementation evidence | Certification evidence |
| --- | --- | --- |
| Runtime-owned temporal subsystem | `TemporalRuntimeState` owns clock basis, scheduled/ready/owner frontiers, retired wakes, and previous-value capability epoch in `state/temporal.rs` | `scheduling_temporal_wake_assigns_monotonic_identity_and_updates_summary`; `owned_temporal_wake_lifecycle_preserves_owner_across_schedule_ready_retire_and_reschedule` |
| Explicit clock basis and domains | `RuntimeClockBasis`, `ClockAdvanceRequest`, `ValidatedClockAdvance`, authoritative-domain validation in `clock.rs` | `clock_advance_rejects_metadata_only_domains`; `clock_advance_rejects_monotonic_regression`; `branch_switch_preserves_clock_basis_identity` |
| Sealed temporal policy family | `TemporalCondition` enum in `condition.rs` contains `After`, `AtOrAfter`, `Debounce`, `Throttle`, `StaleAfter`, `Interval` | `sealed_temporal_policy_family_uses_node_owned_runtime_wakes_without_resolver`; `graph_only_sealed_temporal_execution_cannot_use_host_resolver_truth` |
| Node declarations own wakes | `admit_node_temporal_wake_with_summary` and plan/node admission helpers in `state/temporal.rs` and transaction execution | `node_owned_after_declaration_schedules_defers_admits_and_consumes_one_wake`; `sealed_temporal_policy_family_uses_node_owned_runtime_wakes_without_resolver` |
| Scheduled wake ownership and deterministic ready ordering | `ScheduledTemporalWake`, `ReadyTemporalWake`, `TemporalWakeOwner`, `TemporalFrontierSnapshot`, `scheduled_frontier`, `ready_frontier`, owner frontier | `due_temporal_wake_batch_promotion_is_canonical_by_due_tick_then_ordinal`; `due_temporal_wake_batch_promotion_leaves_future_frontier_entries_scheduled` |
| Wake retirement, supersession, reschedule, and reuse | `RetiredTemporalWake`, `TemporalWakeReschedule`, `TemporalWakeReuse`, owner retirement APIs, node replacement/unregistration hooks | `runtime_unregister_node_structurally_retires_owned_temporal_wakes`; `rewriting_node_evaluation_config_supersedes_owned_temporal_wakes`; `debounce_admission_summary_records_each_burst_supersession_without_extra_live_wakes`; `throttle_admission_summary_records_reuse_without_window_drift` |
| Lowered temporal eligibility artifacts | `LoweredTemporalEligibility`, `DeferredTemporalEligibility`, `ReadyTemporalEligibility`, `TemporalLoweringContext`, `lower_temporal_condition` | `runtime_clock_backed_temporal_ready_is_reported_as_runtime_authority`; `runtime_execute_prepared_plan_uses_clock_basis_for_at_or_after_without_temporal_resolver`; `runtime_target_execution_uses_clock_basis_for_at_or_after_without_temporal_resolver` |
| No host temporal truth for sealed policies | Sealed policies lower through runtime clock/wake facts; `ResolverFallback` remains only a distinct authority posture, not sealed-policy authority | `graph_only_sealed_temporal_execution_cannot_use_host_resolver_truth`; `sealed_temporal_policy_family_uses_node_owned_runtime_wakes_without_resolver`; closeout bundle asserts zero resolver fallback |
| Transaction temporal staging | `TemporalTransactionEvidence` and `TransactionTemporalScratch` carry scheduled, ready, retired, rescheduled, reused, interval, eligibility, and previous-value evidence | `transaction_temporal_evidence_freezes_wake_and_reconstructability_artifacts`; `transaction_debounce_burst_records_supersession_evidence_and_digest`; `transaction_throttle_burst_records_reuse_evidence_and_digest` |
| Transactional previous-value access | `TemporalPreviousValueAccess`, `TemporalPreviousValueReference`, branch id, restore epoch, active ready-wake checks, committed lineage capture | `ready_temporal_wake_grants_previous_value_access_and_captures_committed_state`; `previous_value_reads_committed_branch_truth_after_failed_transaction`; `previous_value_access_is_branch_scoped`; `previous_value_access_is_rejected_after_restore_epoch_changes` |
| Branch, restore, and replay temporal state | Branch state carries temporal runtime state and telemetry; snapshot restore restores temporal state and bumps capability epoch; reconstructability artifacts carry temporal digests | `branch_switch_preserves_temporal_wake_state`; `active_temporal_snapshot_restore_counts_restore_and_reinstates_frontier`; `temporal_snapshot_restore_preserves_ready_wake_frontier_without_rebuild_scan`; `temporal_replay_parity_survives_snapshot_restore_of_ready_frontier` |
| Diagnostics-visible temporal provenance | `TemporalDiagnosticsSummary`, `TemporalCostContractSummary`, retained temporal artifact, frontier, wake summary, telemetry | `temporal_diagnostics_summary_exposes_artifact_without_tier_deciding_truth` |
| Public honest clock/time APIs | Facade exports clock request, validated advance, temporal wake summaries, promotion/admission summaries, and diagnostics summaries | `clock_advance_summary_is_cost_honest_and_does_not_promote_wakes`; compile-fail fixtures for private proof fields |
| Named counters and cost contracts | `TemporalTelemetry`, `TemporalCostContractSummary`, `TemporalPerformanceFailureMode` | `temporal_diagnostics_summary_exposes_artifact_without_tier_deciding_truth`; `ready_promotion_summary_reports_frontier_width_and_broad_scan_denial`; closeout bundle checks prohibited failure modes |

## Phase Acceptance Map

| Phase | Closeout status | Owning evidence |
| --- | --- | --- |
| Phase 1: Temporal Contract Freeze | Closed | Dedicated temporal modules, `TemporalRuntimeState`, sealed condition vocabulary, facade exports, compile-fail privacy tests |
| Phase 2: Clock Basis And Domain Semantics | Closed | `RuntimeClockBasis`, authoritative-domain validation, monotonic regression rejection, branch clock identity tests |
| Phase 3: Sealed Temporal Policy Family | Closed | `TemporalCondition` sealed family plus per-policy scheduling/admission tests |
| Phase 4: Temporal Proof Types And Capability Surfaces | Closed | private-field trybuild fixtures for temporal duration, wakes, frontier, summaries, previous-value access, and lowered eligibility |
| Phase 5: Wake Storage And Frontier Indexing | Closed | scheduled/ready/owner frontiers, canonical ready ordering tests, broad-scan denial counter tests |
| Phase 6: Interval Regeneration And Wake Lifecycle | Closed | interval anchor/missed-tick policy types, regeneration evidence, large-jump tests, node lifecycle retirement tests |
| Phase 7: Temporal Eligibility Lowering And Execution Admission | Closed | precompute lowering, runtime clock/wake-backed eligibility, no sealed-policy resolver fallback tests |
| Phase 8: Transaction Staging And Previous-Value Semantics | Closed | transaction scratch/evidence lanes, committed previous-value references, rollback and branch epoch rejection tests |
| Phase 9: Branch Restore And Replay Integration | Closed | temporal reconstructability artifacts, replay parity reports, branch/snapshot restore tests |
| Phase 10: Diagnostics, Facade, And Certification Surface | Closed as substrate | temporal diagnostics summary, cost contracts, failure mode enum, certification bundle builder and parity tests |

## Required Certification Families

Milestone A requires four named certification lanes. They are represented by
`TemporalCertificationFamily` and are enforced by `TemporalCertificationBuilder`
and `temporal_certification_bundle`.

| Required family | Code surface | Certification tests |
| --- | --- | --- |
| `temporal_eligibility_replay_parity` | `TemporalCertificationFamily::TemporalEligibilityReplayParity`; temporal replay parity report | `temporal_eligibility_replay_parity_certification_family_records_exact_digest_match`; closeout bundle |
| `temporal_branch_restore_equivalence` | `TemporalCertificationFamily::TemporalBranchRestoreEquivalence`; replay parity over restored artifact | `temporal_branch_restore_equivalence_certifies_full_bundle_parity`; closeout bundle |
| `temporal_wake_boundedness` | `TemporalCertificationFamily::TemporalWakeBoundedness`; interval regeneration evidence and missed interval counters | `temporal_wake_boundedness_certification_family_covers_large_interval_jumps`; closeout bundle |
| `previous_value_time_gated_equivalence` | `TemporalCertificationFamily::PreviousValueTimeGatedEquivalence`; previous-value reference digest | `previous_value_time_gated_equivalence_certification_family_captures_committed_lineage`; closeout bundle |

The direct closeout gate is:

- `milestone_a_closeout_bundle_covers_hostile_temporal_certification_paths`

That test builds a complete required certification bundle after exercising:

- multiple sealed temporal policies in one runtime
- zero resolver fallback for sealed temporal declarations
- debounce burst supersession
- throttle wake reuse
- branch fork before temporal readiness
- branch-local restore from retained temporal state
- large interval jumps across missed-tick policies
- previous-value access through a ready temporal wake
- diagnostics tier variation without temporal truth drift

## Hostile Condition Map

| Required hostile condition | Evidence |
| --- | --- |
| Multiple pending wakes in one runtime | `sealed_temporal_policy_family_uses_node_owned_runtime_wakes_without_resolver`; closeout bundle |
| Branch fork before readiness | `branch_switch_preserves_temporal_wake_state`; closeout bundle |
| Node replacement or condition rewrite while wakes are pending | `rewriting_node_evaluation_config_supersedes_owned_temporal_wakes`; `replacing_node_checkpoint_image_supersedes_owned_temporal_wakes`; `stale_ready_owned_wake_is_superseded_before_temporal_lowering` |
| Restore to checkpoints before and after temporal admission | `active_temporal_snapshot_restore_counts_restore_and_reinstates_frontier`; `temporal_snapshot_restore_preserves_ready_wake_frontier_without_rebuild_scan`; closeout bundle |
| Long idle periods with only time advance | `stale_after_expires_without_upstream_writes_under_runtime_owned_time`; interval large-jump tests |
| Threshold-boundary behavior for previous-value-sensitive nodes | `previous_value_reads_committed_branch_truth_after_failed_transaction`; `previous_value_time_gated_equivalence_certification_family_captures_committed_lineage`; closeout bundle |
| Throttle and debounce oscillation under bursty invalidation | `debounce_burst_supersedes_owned_wake_and_waits_for_new_quiet_period`; `throttle_burst_reuses_original_window_without_reschedule`; transaction evidence variants |
| Stale-after expiry without upstream writes | `stale_after_expires_without_upstream_writes_under_runtime_owned_time` |
| Large time jumps across interval periods | `interval_regeneration_collapse_to_one_skips_missed_boundaries_into_future_successor`; `interval_regeneration_skip_to_latest_materializes_one_latest_immediate_successor`; `interval_regeneration_catch_up_all_requires_explicit_repeated_catch_up_steps`; `temporal_wake_boundedness_certification_family_covers_large_interval_jumps` |
| Interval replay after restore with missed-tick policy evidence | closeout bundle plus `temporal_replay_parity_survives_snapshot_restore_of_ready_frontier` |
| Diagnostics-tier variation across equivalent runs | `temporal_diagnostics_summary_exposes_artifact_without_tier_deciding_truth`; closeout bundle |

## Canonical Artifact Map

The certification bundle must include canonical digests for the temporal truth
surfaces below. `TemporalReconstructabilityArtifact` owns the digest fields.

| Required digest surface | Artifact field or source |
| --- | --- |
| Clock checkpoints | `clock_checkpoint_digest`; `clock_basis` |
| Clock-domain declarations | temporal condition descriptors and clock basis inside evidence digests |
| Scheduled wake sets | `scheduled_wake_digest`; `scheduled_wake_count` |
| Ready ordering | `ready_wake_digest`; `ready_wake_count`; ready frontier ordering |
| Retired and superseded wakes | `retired_wake_digest`; `retired_wake_count`; `rescheduled_wake_digest`; `rescheduled_wake_count` |
| Interval regeneration decisions | `interval_regeneration_digest`; `interval_regeneration_count` |
| Temporal eligibility decisions | `temporal_eligibility_digest`; `eligibility_fact_count` |
| Previous-value references | `previous_value_reference_digest`; `previous_value_reference_count` |
| Committed outputs | previous-value references include branch, node, revision, aspect version, and output identity |
| Branch/restore temporal state | wake summary, clock basis, restore counters, retained temporal state |
| Diagnostics/explanation artifacts | `certification_digest`, temporal diagnostics summary, retained artifact expansion |

## Counter And Complexity Map

Named temporal counters are exposed through `TemporalTelemetry`:

- `temporal_wake_count`
- `deferred_by_time_count`
- `scheduled_frontier_width`
- `ready_queue_width`
- `retired_wake_count`
- `rescheduled_wake_count`
- `interval_wake_regeneration_count`
- `missed_interval_count`
- `temporal_eligibility_lowering_count`
- `previous_value_reference_count`
- `branch_local_temporal_restore_count`
- `temporal_replay_parity_check_count`
- `temporal_broad_scan_denial_count`
- `wake_allocation_count`
- `wake_reuse_count`
- `branch_restore_temporal_rebuild_denial_count`

Named complexity contracts are exposed through `TemporalCostContractSummary`:

- temporal registration lowering
- clock advance
- ready-node selection
- interval regeneration
- wake retirement and reschedule
- previous-value lookup
- branch restore
- diagnostics expansion

Named prohibited performance failure modes are machine-visible through
`TemporalPerformanceFailureMode`:

- `TemporalBroadScan`
- `IntervalCatchUpExplosion`
- `WakeAllocationChurn`
- `BranchRestoreTemporalRebuild`
- `RescheduleBreadthLeak`

The cost-honesty tests currently assert:

- clock advance does not hide ready promotion
- ready promotion reports frontier width and broad-scan denial
- interval large jumps are charged through missed-tick policy outcomes
- branch restore increments rebuild-denial evidence when retained temporal
  state is consumed
- diagnostics expansion does not re-decide temporal readiness

## Final Verification Snapshot

Most recent verification for this closeout map:

```powershell
cargo fmt -p worth-signal --check
cargo check -p worth-signal --tests --message-format short
$env:CARGO_PROFILE_TEST_OPT_LEVEL='0'; $env:CARGO_PROFILE_TEST_DEBUG='0'; cargo test -p worth-signal temporal_runtime -- --nocapture
$env:CARGO_PROFILE_TEST_OPT_LEVEL='0'; $env:CARGO_PROFILE_TEST_DEBUG='0'; cargo test -p worth-signal -- --nocapture
git diff --check
```

Observed result:

- temporal runtime suite: `66 passed`
- full `worth-signal` suite: `651 passed`, `0 failed`, `23 ignored`
- doctests: `3 passed`
- formatting/checking: passed
- diff whitespace check: passed with only existing LF/CRLF warnings

## Residual Risk

No Milestone A substrate blocker is known from this acceptance map.

The remaining areas to watch before the next async/resource milestone are:

- public API curation so certification machinery does not become the main
  product identity
- further allocation-shape hardening if temporal wake volume becomes large in
  production workloads
- longer branch-history stress around mixed interval policies and restore
  churn
- accidental reintroduction of resolver-based temporal truth through future
  convenience APIs
- accidental broadening of clock advance into ready selection or diagnostics
  recomputation

These are now guarded by named counters, typed failure modes, and closeout
tests, but they remain the most likely future regression classes.

## Closeout Decision

Milestone A is code-complete as a temporal runtime substrate candidate.

The runtime now admits temporal meaning once, lowers it once, wakes it through
runtime-owned structures, and can replay and certify the resulting temporal
story with machine-checkable artifacts.

The next required work before async/resource substrate should be a public API
curation pass, not another temporal semantics implementation pass.
