# Milestone B Closeout Acceptance Map: Async And Resource Node Runtime Substrate

> **Status:** Completed
>
> **Spec:** [milestone-b-plan.md](./milestone-b-plan.md)
>
> **Roadmap parent:** [worth_signal_temporal_async_roadmap.md](./worth_signal_temporal_async_roadmap.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Prerequisite closeout:** [milestone-a-closeout.md](./milestone-a-closeout.md)

## Purpose

This document maps the Milestone B spec to concrete implementation and
certification evidence.

It is the closeout ledger for the hostile question:

> Can `worth-signal` now own async/resource lifecycle truth as a deterministic,
> replay-honest runtime substrate instead of leaving pending, fulfilled,
> cancelled, timed-out, stale, superseded, and malformed work as adapter-local
> folklore?

## Closeout Summary

Milestone B is implemented as a core async/resource substrate in
`worth-signal`.

The implementation now includes:

- runtime-owned resource declarations and lowered descriptors
- frozen policy descriptor identity and built-in policy lowering
- proof-bearing request identity with request id, generation, attempt, branch
  epoch, and ordinal categories
- hot in-flight request ownership keyed by request identity
- runtime-owned request admission, supersession, cancellation, timeout, retry,
  and revalidation semantics
- scalar and batch completion admission with typed denial artifacts
- transactional completion staging, commit, rollback, and observation
  suppression
- branch/snapshot restore rekeying of in-flight handles
- replay reconstruction of descriptors, lifecycle summaries, denied
  completions, retained history, and in-flight state
- diagnostics summaries with explicit budgeted cold reconstruction
- retained summary reads that prove zero cold reconstruction
- lifecycle-retention compaction and retained-history-unavailable denial
- boundary performance envelopes on public async/resource reports
- allocation posture, density strategy, cost contract, and cost posture
  accounting
- compile-fail proof boundaries for constructor privacy, invalid protocol
  spending, and facade-only runtime access
- sealed milestone certification artifacts, scenario matrix, hostile completion
  evidence, performance closeout, and final certification run

The direct closeout gate is:

- `resource_milestone_b_certification_run_requires_complete_passing_bundle`

That test builds the complete required certification bundle, the nine-row
scenario matrix, the nine-claim performance closeout, and the final
`ResourceMilestoneBCertificationRun`.

## Primary Implementation Surfaces

Resource data model:

- [declaration.rs](../../crates/worth-signal/src/data/resource/declaration.rs)
- [descriptor.rs](../../crates/worth-signal/src/data/resource/descriptor.rs)
- [policy.rs](../../crates/worth-signal/src/data/resource/policy.rs)
- [policy_registry.rs](../../crates/worth-signal/src/data/resource/policy_registry.rs)
- [request.rs](../../crates/worth-signal/src/data/resource/request.rs)
- [lifecycle.rs](../../crates/worth-signal/src/data/resource/lifecycle.rs)
- [denial.rs](../../crates/worth-signal/src/data/resource/denial.rs)
- [completion.rs](../../crates/worth-signal/src/data/resource/completion.rs)
- [summary.rs](../../crates/worth-signal/src/data/resource/summary.rs)
- [diagnostics.rs](../../crates/worth-signal/src/data/resource/diagnostics.rs)
- [certification.rs](../../crates/worth-signal/src/data/resource/certification.rs)

Runtime ownership and transaction integration:

- [resource.rs](../../crates/worth-signal/src/logic/transaction/runtime/state/resource.rs)
- [runtime_state.rs](../../crates/worth-signal/src/logic/transaction/runtime/state/runtime_state.rs)
- [transaction_resource.rs](../../crates/worth-signal/src/logic/transaction/runtime/transaction/transaction_resource.rs)
- [transaction_observation.rs](../../crates/worth-signal/src/logic/transaction/runtime/transaction/transaction_observation.rs)
- [branching state](../../crates/worth-signal/src/logic/transaction/runtime/state/branching)

Diagnostics, counters, facade, and tests:

- [telemetry.rs](../../crates/worth-signal/src/data/telemetry.rs)
- [facade.rs](../../crates/worth-signal/src/facade.rs)
- [resource_runtime.rs](../../crates/worth-signal/src/tests/resource_runtime.rs)
- [resource_api.rs](../../crates/worth-signal/src/tests/resource_api.rs)
- [resource compile-fail fixtures](../../crates/worth-signal/tests/ui)

## Must-Ship Acceptance Map

| Spec requirement | Implementation evidence | Certification evidence |
| --- | --- | --- |
| Runtime-owned async/resource subsystem | `ResourceRuntimeState` owns descriptors, active request index, in-flight request map, retained lifecycle history, denied completion history, restore reports, and replay reports | resource declaration/admission tests; `resource_milestone_b_certification_run_requires_complete_passing_bundle` |
| Lifecycle vocabulary | `ResourceLifecycleClass`, `ResourceLifecycleTransition`, `ResourceOutputContinuity`, `ResourceInitialLifecycleClass` | lifecycle policy hostile deserialization tests; lifecycle transition privacy trybuild |
| Request identity, generation, attempt, branch epoch, and ordinals | `ResourceRequestHandle`, `ResourceRequestId`, `ResourceGeneration`, `ResourceAttemptId`, `ResourceBranchEpoch`, completion/cancellation/retry/timeout ordinals | stale generation/branch/attempt completion tests; raw id cannot replace handle compile-fail boundaries |
| Runtime-owned declaration lowering and policy descriptors | `LoweredResourceDescriptor`, `ResourceResolvedPolicyBundle`, `FrozenResourcePolicyRegistry`, descriptor digests and compatibility posture | policy lowering tests; unknown/duplicate policy denial tests |
| In-flight registration and supersession | active request by node, in-flight by request, retained supersession records | `out_of_order_completion_supersession` certification family; supersession timeout retirement tests |
| Cancellation, timeout, retry, and revalidation | sealed cancellation, timeout, retry schedule/admission, revalidation reports and denial types | focused cancellation/timeout/retry/revalidation tests; closeout bundle families |
| Completion admission and denial | `RawCompletionEnvelope`, `ValidatedCompletionEnvelope`, `AdmittedResourceCompletion`, `DeniedResourceCompletion`, typed `CompletionDenialClass` | hostile completion scenario evidence rows; scalar and batch completion tests |
| Transactional completion apply | staged admitted and denied effects, committed artifacts, rollback reports, transaction commit helpers | completion observation rollback tests; compile-fail fixtures preventing raw/admitted misuse |
| Observation remains commit-bounded | transaction resource completion stages observation through existing observation substrate | completion observation delivery vs rollback suppression tests |
| Branch, snapshot, and replay honesty | branch restore report, replay reconstruction report, restore epoch rekeying, retained summaries | branch/restore replay certification family; restore stale completion tests |
| Diagnostics-visible async/resource provenance | `ResourceDiagnosticsSummary`, replay reconstruction debt, diagnostics expansion budget and denial | diagnostics summary tests; performance closeout diagnostics claims |
| Bounded in-flight tracking | hot request lookup, frontier width counters, retained compaction, broad rebuild denial | in-flight boundedness certification family; performance closeout scenario claim |
| Boundary performance envelopes | `ResourceBoundaryPerformanceEnvelope` on public resource reports | nine performance closeout claims |
| Scalar and batch completion admission | `ResourceCompletionAdmissionReport`, `ResourceCompletionBatchAdmissionReport` | batch duplicate/contradictory/flood tests; batch private/move-only compile-fail |
| Compile-time protocol boundaries | sealed proof types and facade exports only | `resource_compile_fail_boundaries_hold` |

## Required Certification Families

Milestone B requires five named certification families. They are represented by
`ResourceCertificationFamily` and enforced by `ResourceCertificationBuilder`
and `resource_certification_bundle`.

| Required family | Code surface | Certification role |
| --- | --- | --- |
| `async_resource_lifecycle_parity` | `with_async_resource_lifecycle_parity` consumes a `ResourceReplayReconstructionReport` | proves lifecycle, descriptor, denial, retained history, and in-flight replay digests |
| `out_of_order_completion_supersession` | `with_out_of_order_completion_supersession` consumes a superseding `ResourceRequestAdmissionReport` | proves newer request admission supersedes older in-flight truth |
| `async_rollback_observation_equivalence` | `with_async_rollback_observation_equivalence` consumes `ResourceCompletionRollbackReport` | proves rollback evidence and observation suppression are explicit |
| `async_branch_restore_replay_equivalence` | `with_async_branch_restore_replay_equivalence` consumes restore and replay reports | proves branch restore state and replay digests agree |
| `async_inflight_boundedness` | `with_async_inflight_boundedness` consumes retained summary plus boundary performance | proves bounded hot in-flight admission evidence |

The final certification run requires:

- one passing record for every required certification family
- one scenario row for every required scenario
- one hostile completion evidence row for every required hostile scenario
- one performance claim for every required closeout performance claim
- matching bundle, scenario matrix, and performance closeout digests

## Scenario Matrix

`ResourceMilestoneBScenarioMatrix` has nine required rows:

| Scenario | Evidence kind | Purpose |
| --- | --- | --- |
| `LifecycleReplayParity` | certification family | replay reconstructs the lifecycle story, not only values |
| `OutOfOrderSupersession` | certification family | newer admissions retire older completion authority |
| `RollbackObservationEquivalence` | certification family | failed completion work does not leak committed observation |
| `BranchRestoreReplayEquivalence` | certification family | branch restore and replay retain the same resource story |
| `InflightBoundedness` | certification family | request-local hot in-flight state remains bounded |
| `LateCompletionAfterSupersessionRejected` | hostile completion denial | stale success after supersession is denied as `Superseded` |
| `LateCompletionAfterCancellationRejected` | hostile completion denial | late success after cancellation is denied as `Cancelled` |
| `LateCompletionAfterTimeoutRejected` | hostile completion denial | late success after timeout is denied as `TimedOut` |
| `MalformedCompletionRejected` | hostile completion denial | payload-contract drift is denied as `Malformed` |

The matrix is deliberately narrower than every unit-level hostile condition.
It certifies the representative acceptance families and the highest-risk stale
completion lanes. The broader hostile grammar is covered by regression and
compile-fail tests below.

## Performance Closeout

`ResourceMilestoneBPerformanceCloseout` has nine required claims:

| Claim | Evidence bound | Contract checked |
| --- | --- | --- |
| `LifecycleReplayParityDebtBounded` | lifecycle replay scenario row | replay reconstruction is explicitly diagnostics-only debt |
| `OutOfOrderSupersessionAdmissionBounded` | supersession scenario row | one admitted request, two lifecycle transitions, bursty density, exact allocation lanes |
| `RollbackObservationRollbackBounded` | rollback scenario row | rollback performs no lifecycle or retained-history work |
| `BranchRestoreReplayRestoreBounded` | branch restore scenario row | retained summaries plus broad rebuild denial are explicit; diagnostics work remains zero |
| `InflightBoundednessAdmissionBounded` | in-flight boundedness scenario row | sparse one-request admission with exact hot/retained/facade allocation lanes |
| `RuntimeSummaryReadZeroColdReconstruction` | `ResourceRuntimeSummaryReadReport` | retained summary read performs no cold reconstruction |
| `DiagnosticsExpansionBudgetedColdReconstruction` | `ResourceDiagnosticsSummary` | diagnostics cold reconstruction is budget-admitted and marked `Debt` |
| `DiagnosticsExpansionBudgetDenial` | `ResourceDiagnosticsExpansionDenial` | denied diagnostics fallback performs no reconstruction and records `DeniedFallback` |
| `HostileCompletionDenialsScalarBounded` | four hostile scenario rows | denied hostile completions stay scalar and non-transitioning |

Policy-extensibility note:

- these are **Milestone B first-ship closeout claims**, not universal policy
  laws for every future policy family
- future Milestone C policy variants may add richer profiles, but they must
  expose their different cost shape as new named claims, not silently weaken
  these first-ship claims

## Hostile Condition Map

| Hostile condition | Evidence |
| --- | --- |
| Multiple admissions before completion | supersession tests and `OutOfOrderSupersession` scenario |
| Out-of-order completion | supersession scenario and stale/superseded denial tests |
| Duplicate completion delivery | duplicate-after-commit retired test; batch duplicate denial tests |
| Success after timeout | `LateCompletionAfterTimeoutRejected` hostile row |
| Failure after supersession | supersession stale completion denial tests |
| Cancellation racing completion | `LateCompletionAfterCancellationRejected` hostile row |
| Retry racing fresh admission | stale retry and retry-superseded denial tests |
| Broken or delayed completion delivery | unknown, stale, retained-history-unavailable, and partial completion tests |
| Contradictory completion reports | contradictory scalar/batch denial tests and counters |
| Partial payload delivery | partial completion denial test and counter |
| Missing/corrupted request identity | stale/unknown/malformed completion tests |
| Unknown request completion | unknown completion denial and retained denial history tests |
| Retired request completion | duplicate-after-commit retired denial test |
| Cancelled request completion | hostile cancellation row and late cancellation tests |
| Superseded request completion | hostile supersession row and out-of-order tests |
| Completion that lies about generation, attempt, or branch epoch | stale identity tests, pre-restore epoch tests |
| Lost completion and ghost in-flight state | runtime summary, compaction, and in-flight boundedness tests |
| Long-session acquire/supersede/cancel/retry/retention churn | lifecycle compaction, retained-history pruning, retry, cancellation, timeout, and branch restore tests |
| Branch fork with in-flight work | branch restore report and replay reconstruction tests |
| Snapshot restore before and after completion | restore rekeying and pre-restore stale completion tests |
| Diagnostics-tier variation across equivalent runs | diagnostics summary is derived from retained truth; diagnostics expansion is budgeted and non-authoritative |

## Counter And Complexity Map

Named resource counters are exposed through `ResourceTelemetry` and aggregate
through `record_boundary_performance_envelope`.

Key counter families:

- declaration and policy resolution counts
- request admission, cancellation, timeout, retry, and revalidation counts
- scalar and batch completion admission counts
- completion staging, denial staging, commit, and rollback counts
- descriptor, hot in-flight, frontier width, supersession, and branch restore
  counts
- replay reconstruction width and retained-history-unavailable counts
- hot in-flight compaction, retired/reclaimed record, retained write, and
  retained prune counts
- retained summary read, diagnostics expansion, diagnostics cold
  reconstruction, and diagnostics denial counts
- typed completion denial counts for stale, superseded, malformed, partial,
  contradictory, duplicate, unknown, retained-history-unavailable, cancelled,
  and timed-out completions
- temporal wake footprints for timeout and retry
- boundary envelope, broad-scan denial, hot lookup, allocation lane, and
  density strategy counts

Named complexity contracts are exposed by `ResourceBoundaryKind`,
`ResourceCostContractId`, `ResourceCostPosture`, and the per-boundary
`ResourceBoundaryPerformanceEnvelope`:

- declaration lowering
- request admission
- cancellation
- timeout admission
- retry scheduling and admission
- revalidation admission
- scalar completion admission
- batch completion admission
- completion staging, commit, rollback, and denial staging
- branch restore
- replay reconstruction
- retained summary read
- diagnostics expansion and denied diagnostics fallback
- lifecycle retention compaction

## Compile-Fail Boundary Map

`resource_compile_fail_boundaries_hold` covers the public API boundaries.

Important fixture classes include:

- external code cannot construct admitted, cancelled, timed-out, validated, or
  denied resource proof objects
- external code cannot construct lifecycle transition or report proof fields
- external code cannot stage raw completions or denied completions through
  admitted paths
- external code cannot commit admitted completions without staging
- staged and committed completion artifacts are move-only where required
- cancellation proofs cannot be staged as completion proofs
- resource runtime internals are not publicly reachable outside the facade
- initial lifecycle policy cannot be WORTHd into pending or terminal states
- milestone certification, scenario, hostile-evidence, performance-closeout,
  and final-run artifacts cannot be WORTHd or deserialized as proof objects

## Explicit Non-Claims And Deferrals

Milestone B closes the substrate, not every async policy product.

Explicit non-claims:

- wasm, React, route-resource, form, and query replacement ergonomics remain
  out of scope
- network transport and external work execution remain host-owned
- domain-specific cache products remain out of scope
- Milestone C owns richer async/resource policy-family certification
- diagnostics richness may vary by budget, but diagnostics may not redefine
  lifecycle truth
- the performance closeout exactness applies to the first-ship Milestone B
  profile and should be extended through named policy profiles rather than
  weakened for future variants

No known substrate blocker remains for Milestone B closeout.

## Final Verification

Final verification commands:

```powershell
cargo fmt -p worth-signal
cargo test -p worth-signal resource_milestone_b_certification_run_requires_complete_passing_bundle -- --nocapture
cargo check -p worth-signal --tests
cargo test -p worth-signal resource_compile_fail_boundaries_hold -- --nocapture
cargo test -p worth-signal
git diff --check -- plans/worth_signal/milestone-b-closeout.md plans/worth_signal/worth_signal_temporal_async_roadmap.md plans/worth_signal/milestone-b-plan.md
```

Final result at closeout:

- `cargo check -p worth-signal --tests`: passed
- focused closeout test: passed
- resource compile-fail boundary suite: passed
- full `cargo test -p worth-signal`: `714 passed`, `0 failed`, `23 ignored`
- doc-tests: `3 passed`

## Residual Risk

The most sensitive future regression areas are:

- broadening completion admission into graph-wide or all-resource scans
- weakening performance closeout exactness without adding named policy profiles
- treating diagnostics expansion as lifecycle authority
- adding adapter-local resource truth above the substrate
- letting future policy variants decide completion legality after admission
- allowing retained history pruning to degrade into silent completion drops

These risks are guarded by the certification run, scenario matrix, performance
closeout, compile-fail boundaries, and targeted regression tests.

## Closeout Decision

Milestone B is complete and can be treated as closed.

Phase 3 / Milestone C should proceed from this substrate by expanding
descriptor-backed async/resource policy families, not by redefining lifecycle
truth.
