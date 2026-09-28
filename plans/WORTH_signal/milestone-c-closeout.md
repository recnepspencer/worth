# Milestone C Closeout Acceptance Map: Async Resource Policy Families

> **Status:** Completed
>
> **Spec:** [milestone-c-plan.md](./milestone-c-plan.md)
>
> **Roadmap parent:** [worth_signal_temporal_async_roadmap.md](./worth_signal_temporal_async_roadmap.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Prerequisite closeouts:**
> - [milestone-a-closeout.md](./milestone-a-closeout.md)
> - [milestone-b-closeout.md](./milestone-b-closeout.md)

## Purpose

This document maps the Milestone C spec to concrete implementation and
certification evidence.

It is the closeout ledger for the hostile question:

> Can `worth-signal` now own async/resource policy variation as deterministic,
> replay-honest runtime truth instead of leaving retry, timeout, cancellation,
> supersession, freshness, observation, retention, diagnostics, and replay
> compatibility as adapter folklore?

## Closeout Summary

Milestone C is implemented as a descriptor-backed async/resource policy
substrate in `worth-signal`.

The implementation now includes:

- deterministic frozen policy registries with canonical descriptor identity
- built-in policy families for:
  - retry and backoff
  - timeout and deadline
  - cancellation and supersession
  - revalidation and freshness
  - observation and output continuity
  - retention, diagnostics, and replay compatibility
- proof-bearing policy decision artifacts for every first-ship family
- replay-compatibility proofs and typed restore denials
- replay-availability outcomes distinguishing retained, reconstructed, omitted,
  unavailable, and denied access lanes
- typed retained-history availability for lifecycle, denied-completion, and
  retry-lineage compaction/pruning
- policy-specific boundary performance envelopes and telemetry
- sealed policy certification bundle, scenario matrix, performance closeout,
  and final `ResourceMilestoneCCertificationRun`
- direct hostile Phase 9 suites covering the crucial temporal and async
  production-grade grammars

The direct closeout gates are:

- `resource_milestone_c_policy_certification_bundle_and_scenario_matrix_use_production_reports`
- `resource_milestone_c_policy_scenario_matrix_rejects_wrong_restore_denial_class`

Those tests build the required bundle, scenario matrix, performance closeout,
and final certification run from real production reports rather than synthetic
placeholders, and they prove the final gate rejects incomplete or semantically
wrong evidence.

## Primary Implementation Surfaces

Policy model and registry:

- [policy.rs](../../crates/worth-signal/src/data/resource/policy.rs)
- [policy_registry.rs](../../crates/worth-signal/src/data/resource/policy_registry/mod.rs)
- [policy/retry.rs](../../crates/worth-signal/src/data/resource/policy/retry.rs)
- [policy/timeout.rs](../../crates/worth-signal/src/data/resource/policy/timeout.rs)
- [policy/cancellation.rs](../../crates/worth-signal/src/data/resource/policy/cancellation.rs)
- [policy/supersession.rs](../../crates/worth-signal/src/data/resource/policy/supersession.rs)
- [policy/revalidation.rs](../../crates/worth-signal/src/data/resource/policy/revalidation.rs)
- [policy/observation.rs](../../crates/worth-signal/src/data/resource/policy/observation.rs)
- [policy/output_continuity.rs](../../crates/worth-signal/src/data/resource/policy/output_continuity.rs)
- [policy/retention.rs](../../crates/worth-signal/src/data/resource/policy/retention.rs)
- [policy/diagnostics.rs](../../crates/worth-signal/src/data/resource/policy/diagnostics.rs)
- [policy/compatibility.rs](../../crates/worth-signal/src/data/resource/policy/compatibility.rs)
- [policy/replay.rs](../../crates/worth-signal/src/data/resource/policy/replay.rs)

Runtime ownership, replay, observation, and retention:

- [resource.rs](../../crates/worth-signal/src/logic/transaction/runtime/state/resource.rs)
- [runtime_state.rs](../../crates/worth-signal/src/logic/transaction/runtime/state/runtime_state/mod.rs)
- [resource_observation.rs](../../crates/worth-signal/src/logic/transaction/runtime/state/resource_observation.rs)
- [revalidation.rs](../../crates/worth-signal/src/data/resource/revalidation/mod.rs)
- [retention.rs](../../crates/worth-signal/src/data/resource/retention.rs)
- [replay_availability.rs](../../crates/worth-signal/src/data/resource/replay_availability.rs)
- [summary.rs](../../crates/worth-signal/src/data/resource/summary/mod.rs)
- [diagnostics.rs](../../crates/worth-signal/src/data/resource/diagnostics.rs)
- [observation.rs](../../crates/worth-signal/src/data/resource/observation.rs)
- [telemetry.rs](../../crates/worth-signal/src/data/telemetry.rs)

Certification and public surface:

- [certification.rs](../../crates/worth-signal/src/data/resource/certification/mod.rs)
- [facade.rs](../../crates/worth-signal/src/facade.rs)
- [resource_runtime.rs](../../crates/worth-signal/src/tests/resource_runtime/mod.rs)
- [resource_api.rs](../../crates/worth-signal/src/tests/resource_api.rs)
- [temporal_runtime.rs](../../crates/worth-signal/src/tests/temporal_runtime.rs)
- [resource compile-fail fixtures](../../crates/worth-signal/tests/ui)

## Must-Ship Acceptance Map

| Spec requirement | Implementation evidence | Certification evidence |
| --- | --- | --- |
| Frozen policy registries and canonical descriptor identity | frozen registry plus deterministic descriptor ids, semantic names, versions, parameter digests, and compatibility posture | `RegistryOrderCanonicalization` scenario row; bundle/matrix/final-run tests |
| Built-in retry/backoff policy families | retry declarations, lowered retry decision plans, deterministic jitter, retry budgets, duplicate retry coalescing | `Async Retry Budget And Backoff Certification Test`; retry storm and jitter parity rows |
| Built-in timeout/deadline policy families | fixed/per-attempt/total-lifetime/inherited deadline/heartbeat extension/terminal-vs-revalidation timeout policies | `Async Timeout Deadline Certification Test`; timeout race and heartbeat terminal denial rows |
| Built-in cancellation/supersession policy families | runtime-hard cancellation, host advisory, grace periods, dependent propagation, overlap posture, intent-equivalence coalescing | `Async Cancellation Supersession Policy Certification Test`; host cancellation failure and overlap identity rows |
| Built-in revalidation/freshness policy families | explicit, stale-after, dependency-change, observer-demand, terminal-state, fulfilled-only, forced active-handle, and coalesced revalidation | `Async Revalidation Freshness Certification Test`; forced revalidation and observer-demand rows |
| Built-in observation/output continuity families | lifecycle-only, lifecycle+output, denied-completion observation, retry-schedule observation, preserve/hide pending and terminal visibility | `Async Observation Output Continuity Certification Test`; pending visibility and denied-completion observation rows |
| Built-in retention/diagnostics/replay compatibility families | typed retention compaction, diagnostics budgets, replay compatibility/incompatibility, replay availability lanes | `Async Retention Replay Policy Certification Test`; retention availability, diagnostics denial, and replay compatibility rows |
| Typed policy denials before execution | unknown, duplicate, missing, incompatible, malformed, illegal, and budget-exhausted denials | registry boundary rows plus restore compatibility denials |
| Policy-specific boundary performance envelopes | policy decision counters, boundary envelopes, closeout claim rows, zero-cold denial separation | `ResourceMilestoneCPolicyPerformanceCloseout` plus closeout assertions |
| Final certification run | sealed bundle, scenario matrix, performance closeout, final summary digest | `ResourceMilestoneCCertificationRun` tests and compile-fail proof boundaries |
| Crucial mixed hostile workload proof | unified async nightmare grammar, async branch replay equivalence, temporal parity/boundedness, strengthened lifecycle/rollback/liveness lanes | Phase 9 direct suites listed below |

## Required Certification Families

Milestone C requires seven named certification families. They are represented
by `ResourceMilestoneCPolicyCertificationFamily` and enforced by
`resource_milestone_c_policy_certification_builder`.

| Required family | Owning surface | Certification role |
| --- | --- | --- |
| `async_resource_policy_family_certification` | policy bundle / scenario matrix rows | proves registry + descriptor + family identity truth |
| `async_retry_budget_and_backoff_certification` | retry/backoff runtime decisions | proves retry budgets, coalescing, deterministic jitter, and zero-wake denial |
| `async_timeout_deadline_certification` | timeout/deadline runtime decisions | proves timeout race classification, heartbeat extension, inherited deadline truth |
| `async_cancellation_supersession_policy_certification` | cancellation/supersession runtime decisions | proves advisory separation, overlap posture, cancellation/supersession law |
| `async_revalidation_freshness_certification` | revalidation proofs and coalescing | proves freshness lanes remain distinct and proof-gated |
| `async_observation_output_continuity_certification` | observation and output continuity runtime decisions | proves observation/reporting/visibility do not mutate lifecycle truth |
| `async_retention_replay_policy_certification` | retention, diagnostics, replay compatibility, replay availability | proves history richness, diagnostics cold-work, and replay denial/admission stay typed |

These families own the direct Milestone C acceptance surface described in
section `16` of the spec.

## Scenario Matrix

`ResourceMilestoneCPolicyScenarioMatrix` contains the spec-required scenario
rows from section `13.3`.

Representative required rows include:

- `RegistryOrderCanonicalization`
- `DuplicatePolicyIdentityRejected`
- `UnknownPolicyReferenceRejected`
- `RetryBudgetExhaustionRejected`
- `DeterministicJitterReplayParity`
- `RetryStormCoalescingBounded`
- `TimeoutSuccessRaceClassified`
- `HeartbeatExtensionTerminalDenied`
- `HostCancellationFailureLateCompletionDenied`
- `OverlappingGenerationIdentityPreserved`
- `IntentEquivalenceCoalescingPreservesLineage`
- `RetryAndRevalidationRemainDistinct`
- `ForcedRevalidationRequiresActiveHandle`
- `ObserverDemandUsesCommittedObservation`
- `PendingVisibilityDoesNotMutateLifecycle`
- `DeniedCompletionObservationCannotApply`
- `RetentionCompactionReportsUnavailableHistory`
- `DiagnosticsExpansionBudgetDeniedZeroCold`
- `ReplayCompatibilityExactDescriptorMatch`
- `ReplayCompatibilityIncompatibleVersionDenied`
- `ReplayCompatibilityMissingDescriptorDenied`

The direct owning test that proves production-evidence construction is:

- `resource_milestone_c_policy_certification_bundle_and_scenario_matrix_use_production_reports`

The direct test that proves the matrix rejects semantically wrong evidence is:

- `resource_milestone_c_policy_scenario_matrix_rejects_wrong_restore_denial_class`

## Performance Closeout

`ResourceMilestoneCPolicyPerformanceCloseout` contains the spec-required claims
from section `13.4`.

Representative required claims include:

- `RegistryFreezeOrderBounded`
- `RetryBudgetDenialZeroWake`
- `RetryStormCoalescingBounded`
- `DeterministicJitterReplayBounded`
- `TimeoutRaceFrontierBounded`
- `HostCancellationAdvisorySeparated`
- `SupersessionOverlapIdentityBounded`
- `RevalidationActiveHandleBounded`
- `ObservationVisibilityRollbackBounded`
- `DeniedCompletionObservationNonApplying`
- `RetentionCompactionAvailabilityBounded`
- `DiagnosticsBudgetDenialZeroCold`
- `ReplayCompatibilityDescriptorBounded`

Performance closeout rows bind:

- scenario row digest
- policy decision digest
- boundary performance envelope digest
- cost contract id
- cost posture
- allocation lane counts

That closes the spec requirement that no performance claim pass from family
name or scenario digest alone.

## Crucial Requirement Coverage Map

Phase 9 deliberately allowed stronger combined suites to satisfy some of the
harder `test-requirements.md` clauses. This is the explicit map required by the
spec.

### Directly blocking and directly satisfied

| Requirement | Owning suite(s) |
| --- | --- |
| `13. Temporal wake boundedness test` | `temporal_phase9_mixed_workload_preserves_parity_and_boundedness_across_branch_restore` |
| `15. Async resource lifecycle parity test` | `resource_async_lifecycle_and_rollback_workload_preserves_committed_truth_and_suppresses_observation`; strengthened certification builder parity checks |
| `17. Async rollback and observation equivalence test` | `resource_async_lifecycle_and_rollback_workload_preserves_committed_truth_and_suppresses_observation`; `resource_rollback_certification_rejects_control_observation_mismatch` |
| `18. Async branch restore and replay equivalence test` | `resource_async_branch_restore_replay_equivalence_converges_for_equivalent_hostile_suffixes` |
| `19A/19B. Worst async nightmare grammar / regulated-system adversarial rule` | `resource_async_nightmare_grammar_preserves_canonical_truth_across_restore_and_replay`; `resource_async_inflight_pressure_workload_keeps_matching_local_and_bounded`; `resource_async_liveness_failures_preserve_inflight_truth_and_reject_zombie_completion`; `resource_milestone_b_hostile_scenario_evidence_rejects_non_hostile_batch_denials` |

### Important requirements satisfied through stronger combined suites

| Requirement | Combined-suite coverage |
| --- | --- |
| `11. Temporal eligibility replay parity test` | satisfied by the mixed temporal Phase 9 workload, which asserts canonical eligibility, wake, replay-history, and restore parity across combined temporal policies |
| `12. Temporal branch restore equivalence test` | satisfied by the mixed temporal Phase 9 workload, which proves branch-local temporal restore parity and boundedness together |
| `14. Previous-value and time-gated node equivalence test` | satisfied by the mixed temporal Phase 9 workload plus Milestone A previous-value certification lanes |
| `16. Out-of-order completion supersession test` | satisfied by the unified async nightmare grammar plus Milestone B/C supersession hostile rows |
| `20. Async resource policy family certification test` | satisfied directly by the Milestone C policy bundle, scenario matrix, performance closeout, and final run |
| `20A. Async policy registry boundary test` | satisfied directly by registry boundary scenario rows, restore-compatibility denials, and compile-fail proof fixtures |

## Hostile Condition Map

Milestone C hostile rows and Phase 9 suites together cover:

- duplicate policy id / duplicate semantic-name registration
- unknown policy references during declaration lowering
- missing and incompatible restore policy descriptors
- deterministic jitter replay after branch restore
- retry budget exhaustion under retry storm pressure
- duplicate pending retry coalescing
- timeout racing success and cancellation
- timeout-triggered retry with exhausted retry window
- progress-heartbeat extension of a non-terminal request
- host cancellation signal failure followed by late completion
- cancellation racing completion
- supersession with old host work left running
- overlapping-generation admission without identity collapse
- intent-equivalence coalescing with distinct request identities
- stale-after revalidation after temporal restore
- observer-demand revalidation racing dependency-change revalidation
- forced revalidation without active-handle proof
- pending visibility preserved under one policy and hidden under another
- denied-completion observation without denied-completion apply
- retained history pruned before diagnostics expansion
- diagnostics budget denial with zero cold work
- replay-compatible restore
- replay-incompatible restore
- duplicate, contradictory, and unknown-request completions inside the mixed
  async nightmare grammar
- ghost inflight state and zombie completion after restore-era churn

## Compile-Fail Boundary Map

`resource_compile_fail_boundaries_hold` covers the public proof boundaries.

Required Milestone C fixture classes now include:

- external code cannot construct frozen policy descriptors
- external code cannot construct lowered policy bundles
- external code cannot construct admitted or denied policy decision artifacts
- external code cannot mutate private policy decision fields
- retry proof cannot be spent as revalidation proof
- host advisory proof cannot be spent as runtime cancellation proof
- active-handle revalidation proofs remain sealed
- diagnostics-expansion budget admissions remain sealed
- replay compatibility proofs remain sealed
- final certification rows and runs cannot be deserialized as proof objects
- policy certification bundle / matrix / performance closeout / final run
  private fields remain inaccessible

Representative fixture coverage is declared in:

- [resource_api.rs](../../crates/worth-signal/src/tests/resource_api.rs)

## Final Verification

Final closeout verification commands:

```powershell
cargo fmt -p worth-signal
cargo test -p worth-signal resource_milestone_c_policy -- --nocapture
cargo test -p worth-signal resource_policy_restore_compatibility -- --nocapture
cargo test -p worth-signal resource_replay_availability -- --nocapture
cargo test -p worth-signal resource_async_nightmare_grammar_preserves_canonical_truth_across_restore_and_replay -- --nocapture
cargo test -p worth-signal resource_async_branch_restore_replay_equivalence_converges_for_equivalent_hostile_suffixes -- --nocapture
cargo test -p worth-signal resource_async_lifecycle_and_rollback_workload_preserves_committed_truth_and_suppresses_observation -- --nocapture
cargo test -p worth-signal resource_async_inflight_pressure_workload_keeps_matching_local_and_bounded -- --nocapture
cargo test -p worth-signal resource_async_liveness_failures_preserve_inflight_truth_and_reject_zombie_completion -- --nocapture
cargo test -p worth-signal temporal_phase9_mixed_workload_preserves_parity_and_boundedness_across_branch_restore -- --nocapture
cargo test -p worth-signal resource_compile_fail_boundaries_hold -- --nocapture
cargo test -p worth-signal
git diff --check -- plans/worth_signal/milestone-c-closeout.md plans/worth_signal/milestone-c-plan.md plans/worth_signal/worth_signal_temporal_async_roadmap.md
```

Observed result at closeout:

- focused Milestone C policy certification tests: passed
- replay compatibility tests: passed
- replay availability tests: passed
- unified async nightmare grammar: passed
- async branch restore / replay equivalence: passed
- async lifecycle / rollback / observation workload: passed
- async inflight pressure boundedness: passed
- async liveness pressure regression: passed
- temporal Phase 9 mixed workload: passed
- compile-fail proof boundary suite: passed
- full `cargo test -p worth-signal`: `854 passed`, `0 failed`, `23 ignored`

## Residual Risk

No known Milestone C substrate blocker remains from this acceptance map.

The most sensitive future regression areas are:

- softening the nightmare grammar back into decorative scenario rows
- letting replay-compatible restore drift into implicit reinterpretation
- letting diagnostics richness redefine lifecycle or replay truth
- hiding policy cold work behind ordinary retained-summary or replay reads
- weakening exact row coverage in the scenario matrix or performance closeout
- reintroducing adapter-local policy semantics above the runtime substrate

These risks are guarded by the final certification run, explicit coverage map,
compile-fail proof boundaries, and the mixed hostile Phase 9 suites.

## Closeout Decision

Milestone C is complete and can be treated as closed.

The runtime now owns async/resource policy variation as a deterministic,
descriptor-backed, replay-honest substrate, and the crucial temporal/async
hostile workloads are certified directly rather than being left to product
layers to rediscover.

Phase D and later product-layer work should build on this substrate, not reopen
policy truth inside route, query, form, or adapter abstractions.
