# Milestone 17 Engineering Spec: Temporal And Async Bridge Basis, Causality, And Certification

> **Status:** Draft
>
> **Roadmap parent:** [worth_runtime_bridge_roadmap.md](./worth_runtime_bridge_roadmap.md)
>
> **Vision parent:** [worth_runtime_bridge_vision.md](./worth_runtime_bridge_vision.md)
>
> **Bridge certification companion:** [test-requirements.md](./test-requirements.md)
>
> **Primary predecessors:** [milestone-15.md](./milestone-15.md), [milestone-16.md](./milestone-16.md), and WORTH Signal temporal/async milestones A through D
>
> **Purpose:** merge roadmap Milestones 17 through 19 into one bridge milestone so temporal basis binding, async completion causality, mixed-cause ordering, restart/resume basis, and end-to-end certification close as one lower-authority integration contract before Query builds its public temporal/async surface on top.

## Goal

Make the bridge own one coherent temporal/async cross-runtime contract that:

- binds relational truth basis to signal temporal basis without stealing clock
  authority from `worth-signal`
- binds bridge async source declarations to signal async lifecycle truth
  without letting adapters define completion meaning
- defines canonical mixed truth/time/async cause ordering, restart/resume
  basis, and offline diagnosis artifacts
- leaves Query free to productize temporal/async semantics without inventing
  its own lower-authority basis or causality model

## Why This Milestone Exists

WORTH Signal already settled the core runtime law:

- Milestone A made temporal eligibility, scheduled wakes, previous-value
  access, and replay runtime-owned truth
- Milestone B made async/resource lifecycle runtime-owned truth
- Milestone C made retry, timeout, cancellation, supersession, revalidation,
  and diagnostics descriptor-backed runtime policy
- Milestone D made async a capability on ordinary nodes rather than a separate
  runtime species

The bridge now has to close the integration law above that substrate.

Without this milestone:

- Query would be forced to invent its own temporal basis vocabulary before the
  bridge freezes how truth basis and signal temporal basis compose
- Query would be forced to invent its own async completion causality before the
  bridge freezes how source identity, truth-view basis, signal generation, and
  stale completion denial compose
- server-facing delivery work would risk consuming a half-Query, half-Bridge
  mixed-cause semantics split
- restart, replay, preview discard, promotion, and offline diagnostics would
  remain integration folklore instead of bridge-owned certification artifacts

This milestone is therefore the lower-authority temporal/async contract that
Query and later Server work must inherit rather than redefine.

## Governing Summaries

- `MENTALITY.md`: close the hard integration truth first. Do not let Query or
  host adapters guess the basis/causality model and retrofit it later.
- `arch_laws.md`: bridge artifacts must be self-describing, phase-typed,
  authority-preserving, and explicit about boundary crossings. The bridge binds
  truth and compute; it does not become either authority.
- `composition_laws.md`: temporal basis binding, async source admission,
  completion causality, mixed-cause ordering, restart/resume, diagnostics, and
  certification are separate responsibilities and must not collapse into one
  â€œtemporal async helperâ€ subsystem.
- `domain_structure_laws.md`: the tree must preserve distinct homes for
  temporal basis, async lifecycle binding, mixed-cause delivery, restart
  basis, diagnostics, and certification instead of burying them in the earlier
  subscription modules.
- `perf_laws.md`: cause binding, completion admission, replay, and resume must
  stay bounded by declared cause width, basis width, inflight width, and
  delivery width rather than hidden whole-registry or whole-history scans.
- `worth_runtime_bridge_vision.md`: the bridge owns causal protocol boundaries,
  not truth authority, not signal scheduling, and not host-local conventions.
- `worth_runtime_bridge_roadmap.md`: old milestones 17 through 19 are one
  coherent cross-runtime temporal/async integration surface, not four
  unrelated features.
- `worth_signal_temporal_async_roadmap.md`: bridge temporal/async work must
  consume Signalâ€™s closed substrate law. It may not redefine time, lifecycle,
  policy, or async capability shape.
- `milestone-15.md`: active subscription identity, delivery, fanout,
  continuation, checkpoint, replay, and preview residue are already real and
  must be extended rather than replaced.
- `milestone-16.md`: certification bundles and reference workloads already
  exist as a bridge pattern and should be extended, not reinvented.
- `test-requirements.md`: temporal/async bridge closure is not honest until
  basis equivalence, stale completion denial, mixed-cause replay parity,
  restart/resume parity, and offline diagnostics all emit canonical
  machine-checkable bundles.

## Adversarial Constraint

Under branch divergence, historical basis variation, replay, restart,
preview discard/promotion, temporal wakes, async completion races, retry,
cancellation, supersession, diagnostics-tier variation, and hostile adapter
variation, the same canonical bridge declaration and basis inputs must produce
the same temporal/async bridge basis, the same accepted or denied completion
set, the same mixed-cause delivery ordering, and the same offline diagnosis
artifacts unless the scenario is intentionally semantically different.

If any supported path:

- lets host-local clocks or wall-clock metadata define temporal authority
- lets adapters define async lifecycle state through transport-local strings or
  callback order
- loses the distinction between truth basis, signal temporal basis, async
  generation, subscription instance, and delivery basis
- allows mixed-cause ordering to vary with host event arrival order
- requires live runtime memory or logs to diagnose stale completion, preview
  residue, resume mismatch, or mixed-cause drift
- or forces Query to reopen the same basis/causality questions at a higher
  layer

then this milestone has failed.

## Product Decision Lock

- This milestone absorbs old roadmap Milestones 17 through 19 into one bridge
  milestone with internal phases.
- Bridge temporal/async artifacts are lower-authority integration artifacts,
  not public product semantics. Query will later project them into product
  surfaces without redefining them.
- `worth-signal` remains authoritative for temporal execution basis, wake
  scheduling, previous-value semantics, async lifecycle, retry, timeout,
  cancellation, supersession, and policy-family behavior.
- `worth-relational` remains authoritative for truth basis, snapshots,
  branches, historical identity, commits, CDC cursors, lineage, and retained
  authoritative history.
- The bridge owns:
  - cross-runtime temporal basis records
  - async source declaration binding
  - completion causality artifacts
  - mixed-cause ordering artifacts
  - restart/resume basis records
  - offline diagnostic bundles
  - reference workload certification bundles
- Query must not freeze its own temporal/async basis or mixed-cause law ahead
  of these bridge artifacts.
- Mixed truth/time/async delivery ordering is part of this bridge milestone,
  not a later Query-only concern.
- Restart/resume basis for temporal and inflight async state is part of this
  bridge milestoneâ€™s runtime-backed contract, even if durable store-backed
  continuation remains later work.

## Phase Plan

1. **Phase 1: Temporal Basis Artifact Boundary**
   Freeze the bridge-owned artifact that binds truth-view basis, signal clock
   basis, temporal wake evidence, and branch/snapshot/CDC identity into one
   canonical temporal bridge basis.

   **Relevant subsystems**
   - `worth-runtime-bridge` truth-view basis and subscription basis artifacts
   - `worth-runtime-bridge` routing, boundary-envelope, and causal evidence surfaces
   - `worth-relational` snapshot, branch-head, historical, and CDC cursor basis surfaces
   - `worth-signal` temporal basis, clock-domain, wake-evidence, and previous-value substrate from Milestone A

   **Relevant `worth-signal` APIs**
   - `facade::core::{ClockAdvanceRequest, ClockTick, RuntimeClockBasis, TemporalPreviousValueReference}`
   - `facade::runtime::{SignalRuntime, TemporalFrontierSnapshot, TemporalWakeAdmissionSummary, TemporalWakeSummary}`
   - `SignalRuntime::{clock_basis, temporal_wake_summary}`

   **Relevant `worth-proof` APIs**
   - `assumption::{FreshnessScopedBasis, CurrentValidity, AuthorityRevalidationRequired}`
   - `assumption::readmission::BoundaryBridged`
   - use these as the model for basis freshness, weakening, and explicit readmission rather than inventing bridge-local freshness folklore

   **Relevant `worth-foundational` APIs**
   - `canonicalization::front_doors::basis::CanonicalBasisFrontDoor`
   - `canonicalization::digest_slots::evidence::{CanonicalDigestBasisBundle, CanonicalDigestBasisSequence}`
   - use these as the model for canonical basis construction and digest-basis participation for temporal bridge basis artifacts

   **Warnings**
   - Do not let bridge-local timestamps, wall-clock metadata, or transport timing become temporal authority.
   - Do not collapse truth-view basis and signal temporal basis into one undifferentiated "current time" artifact.

   **Test requirements**
   - This phase is not done until `Temporal Bridge Basis Equivalence Test` passes for authoritative, historical, branch-head, and CDC-cursor truth bases paired with equivalent signal temporal basis artifacts.
   - Completion requires canonical digests for truth-view basis, signal clock basis, temporal wake evidence, and the merged temporal bridge basis, with explicit typed failures for stale, missing, wrong-domain, and cross-branch evidence.

   **Engineering decisions**
   - The temporal bridge basis will be a first-class bridge artifact rather than an inferred diagnostics reconstruction.
   - The artifact will always carry separate fields for truth-view basis, signal clock basis, and temporal wake evidence; no flattened composite-only representation is allowed.
   - Wall-clock and presentation-time metadata, if retained at all, will be explicitly non-authoritative metadata and excluded from basis equivalence.
   - The first shipping artifact family will admit authoritative, branch-head, historical, and CDC-cursor truth bases under one native temporal basis contract rather than splitting CDC support into a second-class follow-up lane.
   - Temporal bridge basis fields must remain named native basis carriers; they may project digest text for comparison, but they cannot degrade into string-built basis packets.

   **Open questions**
   - None. The first shipping slice should include CDC-cursor truth basis inside the same temporal bridge basis family so Query and later Server do not inherit a false "historical but not resumable" split.

2. **Phase 2: Time-Aware Subscription Admission Boundary**
   Extend bridge subscription admission so time-aware families can only activate
   when truth basis, temporal basis, wake evidence, and admitted temporal
   family posture all line up explicitly.

   **Relevant subsystems**
   - `worth-runtime-bridge` subscription declaration, admission, activation, and readiness surfaces from Milestones 14 and 15
   - `worth-runtime-bridge` support and failure-localization surfaces
   - `worth-signal` temporal policy admission substrate from Milestone A
   - `worth-relational` branch, historical, and preview truth-basis surfaces

   **Relevant `worth-signal` APIs**
   - `facade::core::{DeferredTemporalEligibility, LoweredTemporalEligibility, ReadyTemporalEligibility, TemporalCondition}`
   - `facade::runtime::{SignalRuntime, TemporalEligibilityFact, TemporalTransactionEvidence}`
   - `SignalRuntime::{clock_basis, temporal_wake_summary}`

   **Warnings**
   - Do not let subscription admission succeed on implicit or ambient temporal posture.
   - Do not treat unsupported truth-basis and temporal-basis combinations as soft degradation cases.

   **Test requirements**
   - This phase is not done until `Time-Aware Subscription Basis Rejection Test` passes across authoritative, historical, branch-local, and preview-scoped truth views.
   - Completion requires admitted combinations to emit explicit subscription basis artifacts and unsupported combinations to reject before activation with localized typed failures.

   **Engineering decisions**
   - Time-aware admission will fail before activation when any required truth, temporal, or family posture evidence is missing or incompatible.
   - Admission artifacts will record the admitted temporal family explicitly rather than relying on declaration replay to rediscover it.
   - Unsupported basis combinations will deny typed; they will not silently degrade to non-temporal subscription behavior.
   - The first admitted matrix will include authoritative, branch-head, preview-scoped, and pinned-historical truth views whenever the temporal family has an explicit native bridge basis and wake-evidence contract.
   - There are no separate pinned-historical family exceptions inside the admitted temporal set; if a temporal family cannot replay from retained native basis, wake evidence, and normalized previous-value evidence, it is not admitted as a historical-capable family at all.

   **Open questions**
   - None. Pinned-historical support is part of the admission bar for every admitted temporal family that claims historical readiness.

3. **Phase 3: Time-Only Cause Routing Boundary**
   Define how non-patch temporal causes route through the bridge when signal
   time advances derived eligibility without a new relational commit.

   **Relevant subsystems**
   - `worth-runtime-bridge` routing, delivery, and cause-record surfaces
   - `worth-runtime-bridge` active subscription protocol surfaces from Milestone 15
   - `worth-signal` temporal wake and ready-node execution surfaces
   - `worth-relational` retained truth-view basis surfaces used by time-only reevaluation

   **Relevant `worth-signal` APIs**
   - `facade::core::{ClockAdvanceRequest, TemporalClockAdvanceSummary, TemporalReadyPromotionSummary}`
   - `facade::runtime::{ReadyTemporalWake, SignalRuntime, TemporalWakeId}`
   - `SignalRuntime::{advance_clock, promote_temporal_wake_ready}`

   **Warnings**
   - Do not misreport time-only delivery as if a relational patch caused it.
   - Do not let host callback order decide mixed truth-versus-time routing order.

   **Test requirements**
   - This phase is not done until `Truth Patch Plus Clock Advance Replay Parity Test` proves that mixed truth-patch and clock-advance lanes replay to the same routing and delivery digests under host-order variation.
   - Completion requires typed denial for duplicate or stale clock submissions and explicit cause records that preserve time-only versus truth-plus-time distinction.

   **Engineering decisions**
   - Time-only cause routing will use a bridge-native cause class distinct from truth-patch cause classes.
   - Mixed truth-plus-time updates will preserve both cause identities in one canonical cause record rather than collapsing to a winner-take-all label.
   - Duplicate or stale clock submissions will be retained as denied causes, not ignored.
   - This phase will not introduce a separate ordinal family just for time-only causes; canonical mixed-cause ordering artifacts will own ordering once Phase 11 lands.

   **Open questions**
   - None. Stable mixed-cause ordering is enough; Phase 3 should freeze cause identity and cause class, not create a second ordering law that Phase 11 would later have to collapse.

4. **Phase 4: Historical Truth With Temporal Readiness Boundary**
   Freeze the law for combining pinned historical truth basis with advancing
   temporal readiness so current truth churn cannot leak into historical
   time-aware bridge flows.

   **Relevant subsystems**
   - `worth-runtime-bridge` historical evaluation, replay, and truth-view basis surfaces
   - `worth-relational` historical snapshot, lineage, retention, and replay basis surfaces
   - `worth-signal` temporal replay and restore substrate from Milestone A

   **Relevant `worth-signal` APIs**
   - `facade::runtime::{SignalRuntime, TemporalReconstructabilityArtifact, TemporalReplayParityReport, TemporalStateRebuildProof}`
   - `facade::specialist::{RuntimeSnapshot, SnapshotRestoreIntent}`
   - `SignalRuntime::{checkpoint, restore_snapshot, restore_branch_snapshot}`

   **Relevant `worth-proof` APIs**
   - `assumption::{FreshnessScopedBasis, StaleReadable, RebindRequired}`
   - `assumption::readmission::{BoundaryBridgedStaleReadableBasis, BoundaryBridgedRebindRequiredBasis}`
   - use these as the model for retained historical basis that may be replay-readable but not silently current

   **Warnings**
   - Do not let current truth churn leak into historical time-aware evaluation.
   - Do not make historical temporal replay depend on ambient process time or unrecorded host memory.

   **Test requirements**
   - This phase is not done until `Historical Truth With Temporal Wake Replay Test` passes under retained-history, retention-truncated, restore, and replay lanes.
   - Completion requires historical truth basis, temporal wake readiness, and previous-value evidence to remain pinned to canonical retained artifacts rather than current truth or ambient time.

   **Engineering decisions**
   - Historical time-aware flows will pin truth basis first and then advance temporal readiness against that pinned basis.
   - Retention truncation will fail closed when the retained historical basis or temporal evidence is insufficient for replay.
   - Previous-value evidence used in historical temporal replay will be treated as branch- and checkpoint-scoped, never ambient runtime state.
   - The minimum first-ship retained packet is pinned truth-view basis, temporal bridge basis, wake/previous-value evidence, and the lineage or retention proof needed to show those artifacts still belong to the admitted historical view.
   - Every admitted temporal family will use one normalized retained previous-value carrier family; family-specific needs can appear as typed optional sections inside that carrier, but we will not create multiple previous-value artifact families.

   **Open questions**
   - None. One normalized retained previous-value carrier family is the contract.

5. **Phase 5: Async Source Declaration Boundary**
   Define the bridge-owned declaration families for async/resource-backed
   sources that lower into admitted `worth-signal` async lifecycle families.

   **Relevant subsystems**
   - `worth-runtime-bridge` reactive source protocol and declaration-family surfaces
   - `worth-runtime-bridge` subscription declaration and lowering surfaces
   - `worth-signal` async/resource declaration substrate from Milestone B
   - `worth-signal` capability attachment substrate from Milestone D

   **Relevant `worth-signal` APIs**
   - `facade::core::{AsyncNodeCapabilityDeclaration, ResourceNodeDeclaration, ResourcePolicyDescriptor}`
   - `facade::runtime::{LoweredAsyncNodeCapabilityBundle, LoweredResourceDescriptor, ResourceDeclarationReport}`
   - `NodeBuilder::attach_async_capability`

   **Warnings**
   - Do not let Bridge invent source families that bypass Signal lifecycle law.
   - Do not hide declaration-family differences behind one generic "async source" lowering.

   **Test requirements**
   - This phase is not done until `Async Source Lifecycle Bridge Parity Test` proves that equivalent bridge source declarations lower into equivalent admitted Signal lifecycle families across adapter variation.
   - Completion requires distinct declaration digests per admitted family and explicit typed rejection for unsupported source capability or source-family mismatch.

   **Engineering decisions**
   - Bridge async sources will lower only through admitted Signal resource or async-capability declaration families; there will be no bridge-local lifecycle family.
   - Source declaration identity will be descriptor-backed and digest-bearing from the first shipping implementation.
   - Adapter-specific source behaviors that cannot be represented through the admitted declaration family will be rejected rather than hidden behind host callbacks.
   - The first shipping slice will include both request-response and subscription-backed async source families because Query's runtime-backed surface needs both point lookups and continued observation under one bridge law.

   **Open questions**
   - None. Both core families should ship first; a one-family launch would force Query to grow an avoidable second lowering path later.

6. **Phase 6: Async Request Identity Binding Boundary**
   Bind bridge source declaration, truth-view basis, subscription instance,
   temporal basis where relevant, and signal async generation into one explicit
   request-identity contract.

   **Relevant subsystems**
   - `worth-runtime-bridge` source binding, basis artifacts, and subscription instance surfaces
   - `worth-runtime-bridge` continuation and replay identity surfaces
   - `worth-signal` request identity, generation, attempt, and epoch substrate from Milestone B
   - `worth-relational` truth-view basis and branch/preview identity surfaces

   **Relevant `worth-signal` APIs**
   - `facade::core::{AsyncNodeRequestIntent, ResourceAttemptId, ResourceBranchEpoch, ResourceGeneration, ResourceRequestHandle, ResourceRequestIntent}`
   - `facade::runtime::{AsyncNodeRequestAdmissionReport, InFlightResourceRequest, ResourceRequestAdmissionReport}`
   - `SignalRuntime::{admit_async_node_request, admit_resource_request, in_flight_resource_request}`

   **Relevant `worth-proof` APIs**
   - `transition::rejection::PreConstructionGate`
   - `transition::outcomes::TransitionOutcome`
   - use these as the model for separating pre-admission rejection from post-admission stale or failed identity progression

   **Warnings**
   - Do not blur declaration identity, truth-view basis, subscription instance, and signal generation into one loose request id.
   - Do not let Query later reopen request identity because Bridge failed to freeze it here.

   **Test requirements**
   - This phase is not done until `Async Source Lifecycle Bridge Parity Test` and `Out-Of-Order Completion Truth-Basis Supersession Test` both prove that request identity is bound to source declaration, truth-view basis, subscription instance where applicable, and Signal generation.
   - Completion requires canonical request-identity artifacts that compare equal across equivalent lanes and reject branch-crossed, preview-crossed, or generation-drifted reuse.

   **Engineering decisions**
   - Request identity will be a bridge-owned artifact distinct from source declaration identity.
   - Subscription instance identity is part of request identity whenever a source is attached to an active subscription context.
   - Truth-view basis drift, branch drift, preview drift, and Signal generation drift will all be independent typed denial reasons.
   - Temporal basis will participate in request identity only for source families whose admission or replay semantics explicitly depend on temporal posture; non-temporal families will not gain artificial identity churn from unrelated clock movement.
   - Request identity must remain a named native basis composition, not a string-concatenated digest recipe over declaration, branch, aspect, or generation text.

   **Open questions**
   - None. Temporal basis should enter request identity only when the admitted source family is temporally meaningful.

7. **Phase 7: Completion Admission And Denial Boundary**
   Freeze the bridge-visible artifact families for fulfilled, rejected,
   cancelled, timed-out, superseded, and stale-denied completion
   outcomes.

   **Relevant subsystems**
   - `worth-runtime-bridge` completion routing, diagnostics, and failure taxonomy surfaces
   - `worth-runtime-bridge` retained receipt, envelope, and explanation surfaces
   - `worth-signal` completion admission and denial substrate from Milestone B
   - `worth-signal` policy-family substrate from Milestone C

   **Relevant `worth-signal` APIs**
   - `facade::core::{AdmittedResourceCompletion, CompletionDenialClass, DeniedResourceCompletion, RawCompletionEnvelope, ValidatedCompletionEnvelope}`
   - `facade::runtime::{CommittedResourceCompletionArtifact, ResourceCompletionAdmissionReport, ResourceCompletionBatchAdmissionReport}`
   - `runtime.diagnostics()` and resource completion admission/report surfaces

   **Relevant `worth-proof` APIs**
   - `transition::rejection::TransitionReadiness`
   - `transition::outcomes::{TransitionOutcome, DenialTransitionOutcome}`
   - use these as the model for preserving denied, deferred, stale, rebind-required, and failed completion categories without flattening them

   **Warnings**
   - Do not treat adapter success/failure labels as completion truth.
   - Do not collapse denied completion classes into one generic transport or host error.

   **Test requirements**
   - This phase is not done until `Async Source Lifecycle Bridge Parity Test` proves that fulfilled, rejected, cancelled, timed-out, superseded, and stale-denied outcomes remain typed and replay-stable.
   - Completion requires separate admission and denial artifacts plus diagnostics that distinguish transport failure, source-family rejection, and Signal lifecycle denial.

   **Engineering decisions**
   - Completion admission and completion denial will remain separate artifact families with no shared generic outcome wrapper.
   - Denied completions will be retained as canonical lifecycle evidence, not diagnostics-only side effects.
   - Bridge-visible completion classes will map from Signal-owned lifecycle truth plus bridge-owned source and basis context, never from adapter labels.
   - "Transport never produced a valid envelope" will remain a source-family rejection or diagnostics lane, not a first-class admitted completion class, because no canonical completion artifact exists to classify.
   - Retry will not be modeled as a Phase 7 completion class because Signal does not admit "retried" as a completion-denial class; retry scheduling and retry causality remain owned by Phase 9.

   **Open questions**
   - None. Invalid-envelope transport failure should stay outside the admitted completion-class family.

8. **Phase 8: Stale Completion Supersession Boundary**
   Define the precise stale/superseded denial law when truth basis, branch,
   preview posture, subscription instance, temporal basis, or signal generation
   has moved on before a completion arrives.

   **Relevant subsystems**
   - `worth-runtime-bridge` supersession, continuation, preview, and replay identity surfaces
   - `worth-runtime-bridge` denial-report and failure-localization surfaces
   - `worth-signal` stale and superseded completion law from Milestones B and C
   - `worth-relational` branch, historical, and preview truth-basis identity surfaces

   **Relevant `worth-signal` APIs**
   - `facade::core::{CompletionDenialClass, DeniedResourceCompletion, ResourceSupersessionRecord}`
   - `facade::runtime::{ResourceCompletionAdmissionReport, ResourceSupersessionDecisionPlan, ResourceSupersessionDecisionClass}`
   - `SignalRuntime::{admit_resource_request, in_flight_resource_request}` together with denied-completion reporting

   **Relevant `worth-proof` APIs**
   - `assumption::{StaleReadable, RebindRequired}`
   - `transition::outcomes::FreshnessTransitionOutcome`
   - use these as the model for stale-versus-rebind-required distinction where a completion crossed a basis boundary

   **Warnings**
   - Do not admit stale completions merely because their payload matches the current output.
   - Do not let branch, preview, or subscription-instance drift disappear into a generic superseded status.

   **Test requirements**
   - This phase is not done until `Out-Of-Order Completion Truth-Basis Supersession Test` passes with adversarial physical completion order, same-output-lookalike payloads, branch switches, and preview discard lanes.
   - Completion requires denial artifacts to name the superseded truth basis, Signal generation, and subscription instance where applicable, with replay preserving the same accepted and denied completion set.

   **Engineering decisions**
   - Payload equality will never rescue a stale completion.
   - Supersession evidence will always record which newer admitted basis and generation displaced the old completion authority.
   - Preview-discarded and branch-crossed completions will deny in the same stale-causality family, but with distinct typed sub-classes.
   - Subscription-instance supersession, truth-basis supersession, branch drift, preview drift, and generation drift will live under one stale or supersession family with typed sub-variants so downstream Query consumes one stable denial band.

   **Open questions**
   - None. One supersession family with sub-variants is the better first-ship shape.

9. **Phase 9: Retry, Revalidation, And Timeout Causality Boundary**
   Freeze how bridge source identity composes with Signal's retry/backoff,
   revalidation, timeout, and cancellation law without moving scheduling
   authority into the bridge.

   **Relevant subsystems**
   - `worth-runtime-bridge` source causality, lifecycle diagnostics, and temporal basis surfaces
   - `worth-signal` retry/backoff, timeout/deadline, cancellation, revalidation, and policy descriptors from Milestone C
   - `worth-signal` temporal substrate from Milestone A where retry timing depends on runtime-owned time

   **Relevant `worth-signal` APIs**
   - `facade::core::{ResourceRevalidationIntent, ResourceRetryPolicyDeclaration, ResourceTimeoutPolicyDeclaration}`
   - `facade::runtime::{ResourceRetryAdmissionReport, ResourceRetryScheduleReport, ResourceRevalidationReport, ResourceTimeoutReport}`
   - `SignalRuntime::{admit_resource_timeout, cancel_resource_request, revalidate_resource_node}`

   **Warnings**
   - Do not move retry, timeout, or cancellation scheduling authority into Bridge.
   - Do not let retry and revalidation collapse into one causal class.

   **Test requirements**
   - This phase is not done until `Async Retry And Revalidation Causality Test` proves retry, timeout, cancellation, and revalidation preserve canonical causality and compare equal to no-failure control lanes where they should.
   - Completion requires boundedness evidence, temporal retry basis evidence, and typed rejection for retry or revalidation over stale truth basis or stale Signal generation.

   **Engineering decisions**
   - Retry and revalidation will remain separate bridge causality families even when they converge to the same downstream derived output.
   - Timeout and cancellation will remain distinct from retry admission and completion denial.
   - Bridge will consume Signal retry and timeout scheduling evidence but will own the cross-runtime causality link back to source declaration and truth basis.
   - Ordinary operational bundles will retain retry admission, attempt-lineage head, timeout or cancellation disposition, and final revalidation outcome; detailed scheduler traces and full backoff ladders can remain diagnostics-only unless a certification suite proves they affect parity.
   - Exact restart parity will not require retaining full retry scheduling ladders; retry timing will be recomputed from retained basis, lineage head, and policy descriptors rather than checkpointing scheduler internals.

   **Open questions**
   - None. Full retry scheduling evidence is not part of the resume contract.

10. **Phase 10: Async Completion Writeback Boundary**
    Close the optional writeback path so admitted async completions can flow
    back into authoritative truth only through explicit bridge-mediated
    writeback families with causality transfer, idempotence, and loop
    prevention.

    **Relevant subsystems**
    - `worth-runtime-bridge` writeback families and causality-transfer surfaces from Milestones 12 and 12b
    - `worth-runtime-bridge` source binding and completion-causality surfaces
    - `worth-relational` authoritative commit and write authority surfaces
    - `worth-signal` async lifecycle and retry/supersession substrate where completion meaning must remain intact across writeback

    **Relevant `worth-signal` APIs**
    - `facade::runtime::{CommittedResourceCompletionArtifact, ResourceCompletionCommitReport, ResourceCompletionRollbackReport, StagedResourceCompletionEffect}`
    - `facade::core::{ResourceHostCancellationAdvisory, ResourceOldHostWorkCancellationAdvisory}`
    - resource completion staging/commit/rollback reports plus `runtime.diagnostics()` for writeback-local causality evidence

    **Warnings**
    - Do not let async completion writeback bypass relational commit authority.
    - Do not leave loop prevention as host discipline or mapper folklore.

    **Test requirements**
    - This phase is not done until `Async Completion Writeback Loop Prevention Test` passes across idempotent duplicate completion, preview-branch completion, truth-changed completion, mapper failure, and relational writeback rejection lanes.
    - Completion requires loop-prevention evidence linking completion, writeback intent, resulting truth commit where admitted, downstream invalidation, and zero authoritative residue on rejection.

   **Engineering decisions**
   - Async completion writeback will remain optional and family-gated rather than becoming the default completion path.
   - Every admitted writeback will emit causality-transfer evidence linking completion identity to resulting authoritative truth mutation.
   - Duplicate completion handling will be family-specific but always explicit: idempotent-noop or typed denial, never silent replay.
   - The first shipping slice will admit authoritative commit-producing writeback families only; preview-local publication, branch-hopping writeback, and convenience fanout writeback lanes stay deferred until they can prove zero-residue and loop-law parity.

   **Open questions**
   - None. The first slice should ship one canonical authoritative completion-to-commit family and defer additional writeback family shapes until that contract is proven.

11. **Phase 11: Mixed-Cause Ordering Boundary**
    Define one canonical ordering law for truth patches, clock advances,
    temporal wakes, async completions, retries, cancellations, and
    supersessions so host call order cannot change bridge delivery meaning.

    **Relevant subsystems**
    - `worth-runtime-bridge` delivery, replay, causality, and ordering artifact surfaces
    - `worth-runtime-bridge` active subscription protocol from Milestone 15
    - `worth-relational` patch, commit, and lineage cause surfaces
    - `worth-signal` temporal wake, async lifecycle, retry, cancellation, and supersession surfaces

   **Relevant `worth-signal` APIs**
   - `facade::runtime::{ObservationBoundarySummary, ObservationBoundaryOutcome, ResourceCompletionAdmissionReport, TemporalClockAdvanceSummary}`
   - `facade::core::{OutputChange, TemporalClockAdvanceSummary as CoreTemporalClockAdvanceSummary}`
   - `SignalRuntime::{advance_clock, admit_resource_request, admit_resource_timeout, cancel_resource_request}`

    **Relevant `worth-foundational` APIs**
    - `canonicalization::front_doors::basis::CanonicalBasisFrontDoor`
    - `canonicalization::digest_slots::evidence::{CanonicalDigestBasisBundle, CanonicalDigestInputEvidence}`
    - use these as the model for canonical mixed-cause order evidence and replay-comparable ordering digests

    **Warnings**
    - Do not treat mixed-cause ordering as an implementation detail or replay-only artifact.
    - Do not let consumer pacing or host arrival order define semantic delivery order.

    **Test requirements**
    - This phase is not done until `Mixed Cause Delivery Ordering Parity Test` proves canonical ordering across truth patches, clock advances, temporal wakes, async completions, retries, cancellations, and supersessions.
    - Completion requires replay-stable mixed-cause order digests plus explicit proof that stale or duplicate causes do not create extra delivery.

    **Engineering decisions**
    - Mixed-cause ordering will be a bridge-owned semantic law, not an incidental consequence of runtime call order.
    - Ordering artifacts will be retained in canonical bundle form rather than reconstructed from logs.
    - Duplicate and stale causes will participate in ordering as denied or suppressed artifacts, not vanish from the causal story.
    - Phase 11 should emit an explicit total-order artifact over admitted and denied causes; a digest-only or partial-order-only story would leave too much room for Query or Server to reinterpret delivery order later.

    **Open questions**
    - None. We should freeze one explicit total-order artifact here.

12. **Phase 12: Shared-Consumer Temporal/Async Fanout Boundary**
    Extend the earlier subscription fanout model so temporal/async-backed
    subscriptions can fan out to multiple consumers without collapsing shared
    pacing or coalescing into subscription meaning.

    **Relevant subsystems**
    - `worth-runtime-bridge` shared-consumer contracts, fanout layout, and coalesced delivery surfaces from Milestone 15
    - `worth-runtime-bridge` mixed-cause delivery and consumer-contract surfaces
    - `worth-signal` delivery strategy and observation scheduling substrate
    - `worth-relational` truth-basis continuity where shared consumers remain attached to one admitted view

    **Relevant `worth-signal` APIs**
    - `facade::runtime::{MatchingObserverSet, ObservationHandle, ObservationPolicy, ObservationRegistrySummary, ObservationTrigger}`
    - `facade::core::{PartitionSubscription, PartitionToken}`
    - `runtime.diagnostics()` and `runtime.history()` observation/delivery summaries

    **Warnings**
    - Do not let shared-consumer fanout redefine subscription meaning.
    - Do not let coalescing rules erase cause identity or basis identity.

    **Test requirements**
    - This phase is not done until `Temporal Async Subscription Bundle Equivalence Test` and `Mixed Cause Delivery Ordering Parity Test` both pass shared-consumer and coalesced-delivery lanes.
    - Completion requires consumer-contract and coalescing artifacts to vary delivery shape only through admitted contracts, never by mutating subscription meaning.

    **Engineering decisions**
    - Shared-consumer fanout will remain a consumer-contract concern layered over one canonical subscription meaning.
    - Coalescing will be explicit, family-aware, and artifact-bearing rather than implicit scheduler behavior.
    - Separate-but-equivalent consumers and true shared consumers must remain distinguishable in artifacts even when their delivered outputs match.
    - First-ship mandatory coalescing includes duplicate-cause suppression, same-basis same-consumer delivery collapse, and shared-consumer pacing artifacts needed for replay parity; transport or presentation optimizations remain deferrable.
    - Deeper cross-consumer bundle sharing is not parity-critical; it is an optimization layer that must not affect canonical delivery, replay, or certification artifacts.

    **Open questions**
    - None. Cross-consumer bundle sharing is optimization-only.

13. **Phase 13: Restart And Resume Basis Boundary**
    Freeze the checkpoint/resume artifact family that must carry truth cursor,
    truth-view basis, signal temporal basis, async generation, and subscription
    delivery basis for runtime-backed temporal/async restart.

    **Relevant subsystems**
    - `worth-runtime-bridge` checkpoint, resume, replay, and retained lifecycle surfaces from Milestones 15 and 16
    - `worth-runtime-bridge` certification bundle and replay comparison surfaces
    - `worth-relational` CDC cursor, branch/head, snapshot, and historical basis surfaces
    - `worth-signal` temporal basis and async generation replay/restore substrate

    **Relevant `worth-signal` APIs**
    - `facade::runtime::{ResourceBranchRestoreReport, ResourceReplayReconstructionReport, SignalRuntime, TemporalReconstructabilityArtifact}`
    - `facade::specialist::{RuntimeSnapshot, SnapshotRestoreIntent}`
    - `SignalRuntime::{checkpoint, reconstruct_resource_replay_summary, restore_branch_snapshot, restore_snapshot}`

    **Relevant `worth-proof` APIs**
    - `assumption::readmission::{BoundaryBridgedAuthorityRevalidationRequiredBasis, BoundaryBridgedStaleReadableBasis}`
    - `assumption::{AuthorityRevalidationRequired, StaleReadable}`
    - use these as the model for resume artifacts that crossed a trust boundary and must be explicitly re-readmitted

    **Relevant `worth-foundational` APIs**
    - `boundary_evidence::attachment_front_doors::FoundationalBoundaryEvidenceAttachmentFrontDoor`
    - readmission and current-basis/support-basis attachment docs in `worth-foundational`
    - use these as the model for basis-bearing resume evidence that remains explicit about replay-derived versus current-basis posture

    **Warnings**
    - Do not reduce resume basis to raw stream offsets, raw clock ticks, or transport handles.
    - Do not leave inflight async and temporal wake recovery to broad rediscovery.

    **Test requirements**
    - This phase is not done until `Restart Resume With Clock And Inflight Basis Test` passes for pending wakes, ready wakes, inflight async requests, partial delivery, and shared-consumer checkpoints.
    - Completion requires typed failure for stale, truncated, incompatible, or cross-branch basis and exact parity with uninterrupted control lanes when canonical basis is complete.

    **Engineering decisions**
    - Resume basis will explicitly carry truth cursor, truth-view basis, temporal basis, inflight async basis, and subscription delivery basis as separate fields.
    - Restart and resume will fail closed on basis incompatibility rather than attempting best-effort reconstruction.
    - Inflight async recovery and temporal wake recovery will come from retained basis artifacts, not broad rescans or host-local handles.
    - The minimum retained inflight packet will include request identity, source declaration identity, truth-view basis, temporal basis when applicable, signal generation or attempt-lineage head, and the delivery checkpoint needed to prove exact resume.
    - No additional retained retry scheduling evidence is required beyond the lineage head and policy descriptors already fixed in earlier phases.

    **Open questions**
    - None. The resume contract is complete without scheduler-history retention.

14. **Phase 14: Preview Residue, Promotion, And Discard Boundary**
    Close the preview-local lifecycle for temporal wakes, inflight async work,
    completion residue, and promotion/discard boundaries so preview-local
    temporal/async state cannot leak into authoritative flows.

    **Relevant subsystems**
    - `worth-runtime-bridge` preview basis, preview work, promotion, discard, and residue-proof surfaces from Milestones 10, 11, and 15
    - `worth-runtime-bridge` continuation and failure-localization surfaces
    - `worth-relational` branch and preview truth-basis authority surfaces
    - `worth-signal` temporal and async residue-bearing lifecycle substrate

    **Relevant `worth-signal` APIs**
    - `facade::runtime::{ResourceBranchRestoreReport, ResourceRuntimeSummary, TemporalWakeSummary}`
    - `facade::specialist::{RuntimeBranch, RuntimeBranchId, SnapshotRestoreIntent}`
    - `SignalRuntime::{checkpoint, restore_branch_snapshot, restore_snapshot}`

    **Warnings**
    - Do not let preview-local temporal or async residue leak into authoritative flows.
    - Do not make promotion/discard semantics depend on hidden host-local caches.

    **Test requirements**
    - This phase is not done until preview discard and promotion hostile lanes inside `Temporal Async Failure Taxonomy Localization Test` and `End-To-End Temporal Async Reference Workload Sufficiency Test` both pass.
    - Completion requires zero authoritative residue after discard and typed localization for promotion-boundary mismatch, preview-crossed completion, or branch-local temporal evidence drift.

    **Engineering decisions**
    - Preview-local temporal wakes and inflight async work will be treated as preview-owned lifecycle artifacts until an explicit promotion boundary transfers them.
    - Discard will require zero-authoritative-residue proof rather than relying on absence of obvious downstream effects.
    - Promotion mismatch and preview-crossed completion will be typed bridge failures, not generic branch errors.
    - Preview-local temporal and async work should re-admit on the authoritative side rather than structurally promoting as if preview residue were already authoritative; only non-authoritative diagnostics and proof attachments may carry forward directly.
    - No preview-owned helper artifact is worth carrying across promotion as a semantic requirement; explanation after authoritative re-admission must be reconstructed from retained proof attachments and the authoritative artifact set.

    **Open questions**
    - None. Preview helper artifacts do not cross the promotion boundary.

15. **Phase 15: Failure Taxonomy And Offline Diagnosis Boundary**
    Extend bridge-native failure localization and diagnostics so missing
    temporal basis, stale completion, incompatible resume basis, preview
    residue, promotion mismatch, and mixed-cause replay drift are explainable
    offline from canonical artifacts alone.

    **Relevant subsystems**
    - `worth-runtime-bridge` diagnostics facade, causal explanation, and certification bundle surfaces
    - `worth-runtime-bridge` replay, resume, preview, and writeback failure taxonomy surfaces
    - `worth-relational` retained truth authority evidence surfaces
    - `worth-signal` retained temporal and async lifecycle evidence surfaces

    **Relevant `worth-signal` APIs**
    - `runtime.diagnostics()` and `runtime.history()`
    - `facade::runtime::{ResourceDiagnosticsExpansionBudget, ResourceDiagnosticsExpansionDenial, ResourceDiagnosticsSummary, TemporalReplayParityReport}`
    - `facade::core::{CompletionDenialClass, ResourceRetainedHistoryAvailabilityClass, ResourceRetainedRetryLineageAvailabilityClass}`

    **Relevant `worth-proof` APIs**
    - `transition::outcomes::TransitionOutcome`
    - `transition::rejection::TransitionReadiness`
    - use these as the model for typed failure families that preserve denial, stale, rebind-required, and failed distinctions

    **Relevant `worth-foundational` APIs**
    - `boundary_evidence::attachment_front_doors::FoundationalBoundaryEvidenceAttachmentFrontDoor`
    - `canonicalization::front_doors::export::CanonicalExportFrontDoor`
    - use these as the model for offline-diagnosis attachments, provenance materialization, and cross-bundle comparison surfaces

    **Warnings**
    - Do not turn failure localization into a generic diagnostics formatting exercise.
    - Do not require live runtime memory or host logs to explain stale completion, resume mismatch, or mixed-cause drift.

    **Test requirements**
    - This phase is not done until `Temporal Async Failure Taxonomy Localization Test` proves that temporal, async, truth, delivery, subscription, preview, resume, and writeback failures localize to typed bridge-native classes.
    - Completion requires offline artifacts alone to distinguish failed boundaries and rejected artifacts across replay and adapter variation.

    **Engineering decisions**
    - Failure taxonomy will be bridge-native and typed even when parent-runtime provenance is attached.
    - Offline diagnosis will be a first-class product contract, not a diagnostics-richness bonus path.
    - Failure localization matrices will be canonical outputs of certification runs rather than ad hoc debugging reports.
    - The first shipping taxonomy should stabilize top-level failure bands and typed subcodes rather than exploding every edge case into a public top-level class.
    - The first shipping subcode inventory is fixed as:
      `temporal_basis.{missing,incompatible,stale,cross_branch,history_truncated}`
      `temporal_readiness.{not_ready,wake_missing,previous_value_missing}`
      `async_identity.{source_mismatch,basis_mismatch,preview_mismatch,generation_drift,subscription_instance_drift}`
      `completion_admission.{envelope_invalid,transport_rejected,lifecycle_denied}`
      `supersession.{truth_basis,preview,branch,subscription_instance,generation}`
      `retry_revalidation.{retry_rejected,revalidation_rejected,timeout,cancelled}`
      `ordering.{duplicate_cause,suppressed_cause,replay_drift}`
      `resume_basis.{missing,incompatible,stale,truncated}`
      `preview_boundary.{discard_residue,promotion_mismatch,preview_crossed_completion}`
      `policy_remask.{tenant_drift,policy_drift,schema_context_drift}`
      `writeback_boundary.{authority_rejected,mapper_failed,loop_prevented,idempotent_noop}`

    **Open questions**
    - None. Additions after first ship should be exceptional and backwards-conscious.

16. **Phase 16: Temporal/Async Certification Bundle Boundary**
    Define one canonical bridge bundle shape spanning temporal basis, async
    lifecycle, mixed-cause delivery, restart/resume posture, and diagnostic
    sufficiency.

    **Relevant subsystems**
    - `worth-runtime-bridge` certification bundle, digest, and comparison surfaces from Milestone 16
    - `worth-runtime-bridge` delivery, replay, preview, and failure-localization artifact families
    - `worth-relational` truth-basis evidence surfaces included in the bundle
    - `worth-signal` temporal and async lifecycle evidence surfaces included in the bundle

    **Relevant `worth-signal` APIs**
    - `facade::runtime::{ResourceCertificationBundle, ResourceCertificationBundleParityReport, TemporalCertificationBundle, TemporalCertificationBundleParityReport}`
    - `facade::core::{resource_certification_bundle, temporal_certification_bundle}`
    - milestone certification runs: `resource_milestone_b_certification_run`, `resource_milestone_c_certification_run`, and `async_node_milestone_d_certification_run`

    **Relevant `worth-foundational` APIs**
    - `canonicalization::front_doors::basis::CanonicalBasisFrontDoor`
    - `canonicalization::front_doors::digest::CanonicalDigestFrontDoor`
    - `canonicalization::front_doors::export::CanonicalExportFrontDoor`
    - use these as the model for canonical bundle basis preparation, digest derivation, export naming, and comparison law

    **Warnings**
    - Do not produce bundle shapes that hide which runtime owned which cause or basis component.
    - Do not let certification depend on richer diagnostics tiers to recover missing operational truth.

    **Test requirements**
    - This phase is not done until `Temporal Async Subscription Bundle Equivalence Test` and `End-To-End Temporal Async Reference Workload Sufficiency Test` both pass with canonical machine-checkable bundle outputs.
    - Completion requires bundle digests to preserve traceability for truth basis, temporal basis, async lifecycle, mixed-cause ordering, subscription identity, and failure localization without relying on live diagnostics expansion.

    **Engineering decisions**
    - One canonical temporal/async bridge bundle family will exist for comparison, replay, and offline audit; diagnostics-richness variation may add detail but cannot change bundle truth.
    - Bundle artifacts will preserve owning-runtime provenance for every basis and cause component.
    - Bundle comparison will be a bridge-native capability, not an external auditor-only reconstruction flow.
    - The public shape should be one top-level bundle composed from named sub-bundles rather than one flat monolith, so Query and later Server can consume stable sections without re-splitting a giant artifact.

    **Open questions**
    - None. The public surface should expose one top-level composed bundle.

17. **Phase 17: Temporal/Async Reference Workload Boundary**
    Assemble one Rust-only end-to-end workload covering authoritative,
    historical, branch-local, preview, time-only, async-backed, shared-consumer,
    restart, and replay lanes without relying on UI or host-local logging.

    **Relevant subsystems**
    - `worth-runtime-bridge` harness and certification workload surfaces from Milestone 16
    - `worth-runtime-bridge` subscription declaration, active delivery, continuation, preview, replay, and diagnostics surfaces
    - `worth-relational` authoritative, historical, and branch-local truth-basis surfaces
    - `worth-signal` temporal and async runtime substrate consumed through the bridge

    **Relevant `worth-signal` APIs**
    - `facade::runtime::{SignalRuntime, SignalRuntimeBuilder, ResourceMilestoneBCertificationRun, ResourceMilestoneCCertificationRun, AsyncNodeMilestoneDCertificationRun}`
    - `facade::core::{SignalGraph, NodeBuilder}`
    - `SignalRuntime::{advance_clock, admit_async_node_request, admit_resource_request, revalidate_async_node, reconstruct_resource_replay_summary}`

    **Relevant `worth-foundational` APIs**
    - `canonicalization::production_readiness::report::CanonicalProductionReadinessReport`
    - `canonicalization::production_readiness::vocabulary::{CanonicalFixtureManifestEvidence, CanonicalPhaseGateEvidence}`
    - use these as the model for fixture-manifest and phase-gate evidence attached to the reference workload

    **Warnings**
    - Do not let the reference workload become a domain-specific product story instead of a bridge certification fixture.
    - Do not omit hostile lanes just because the happy-path workload already looks realistic.

    **Test requirements**
    - This phase is not done until `End-To-End Temporal Async Reference Workload Sufficiency Test` passes with authoritative, historical, branch-local, preview, time-only, async-backed, shared-consumer, restart, and replay lanes all present.
    - Completion requires one coherent artifact story that supports offline diagnosis, equivalence comparison, and hostile failure localization without host logs or live runtime access.

    **Engineering decisions**
    - The reference workload will be one Rust-only bridge fixture, not a UI-assisted or transport-assisted demo.
    - The workload must include hostile lanes from the start; they are not a later hardening pass.
    - The workload will certify integration truth, not teach domain semantics, so it must stay synthetic enough to isolate bridge law.
    - The first workload skin should extend the existing pricing-shock certification aspect workload because it already exercises authoritative, historical, preview, fanout, restart, merge, and hostile certification lanes while remaining bridge-owned rather than product-owned.
    - We do not need a second companion fixture next to pricing-shock for milestone closeout; targeted hostile minimization can happen through focused pricing-shock sub-suites and smaller harness helpers without promoting a second first-class workload.

    **Open questions**
    - None. Pricing-shock remains the only first-class reference workload for this milestone.

18. **Phase 18: Temporal/Async Certification Closure Boundary**
    Close the merged milestone with the old Milestone 19 style proof bar:
    hostile lanes, replay lanes, restart lanes, unsupported-neighbor lanes, and
    machine-checkable bundle equivalence across the full temporal/async bridge
    surface.

    **Relevant subsystems**
    - `worth-runtime-bridge` full certification and support-matrix surfaces
    - `worth-runtime-bridge` diagnostics entrypoint and bundle comparison surfaces
    - `worth-relational` truth authority evidence needed for final equivalence and rejection lanes
    - `worth-signal` temporal and async authority evidence needed for final equivalence and rejection lanes

    **Relevant `worth-signal` APIs**
    - `facade::runtime::{ResourceCertificationSummary, TemporalCertificationSummary, AsyncNodeMilestoneDCertificationRunSummary}`
    - bundle parity and replay comparison reports: `ResourceCertificationBundleParityReport`, `TemporalCertificationBundleParityReport`, `TemporalReplayParityReport`
    - required-scenario constants and certification families exported through `facade::core` and `facade::runtime`

    **Relevant `worth-foundational` APIs**
    - `canonicalization::production_readiness::report::CanonicalProductionReadinessReport`
    - `canonicalization::front_doors::readiness::CanonicalReadinessFrontDoor`
    - `boundary_evidence::attachment_front_doors::FoundationalBoundaryEvidenceAttachmentFrontDoor`
    - use these as the model for final closeout evidence, phase-gate reporting, and support-grade bundle attachments

    **Warnings**
    - Do not declare the milestone closed without passing the full old-suite closure set from Milestones 17 through 19.
    - Do not let unsupported-neighbor behavior degrade into undocumented best-effort handling.

    **Test requirements**
    - This phase is not done until suites `38` through `50` all pass with canonical machine-checkable outputs across original execution, replay, restart, hostile adapter variation, hostile clock and async ordering variation, and diagnostics-tier variation where admitted.
    - Completion requires final closure evidence showing that intentionally equivalent lanes compare equal, intentionally different lanes compare unequal, and unsupported-neighbor behavior fails explicitly and typed.

    **Engineering decisions**
    - This merged milestone closes only as one batch; partial suite closure does not count as milestone completion.
    - The final closeout must treat old Milestones `17`, `18`, and `19` as internal proof bands, not separate releasable states.
    - Unsupported-neighbor and unsupported-basis behavior must remain explicit typed failure in the final support matrix.
    - Final closure should emit an explicit merged-milestone closeout artifact that summarizes suites `38` through `50`, bundle parity, replay parity, restart parity, and unsupported-neighbor behavior in one machine-checkable packet.

    **Open questions**
    - None. The merged milestone should emit its own explicit closeout artifact.

## Sequencing Notes

This milestone belongs immediately after the current subscription/certification
story because:

- Milestone 15 already closed activation, delivery, fanout, continuation,
  checkpoint, replay, and preview residue for ordinary subscription protocol
- Milestone 16 already establishes the certification and reference-workload
  pattern the temporal/async bridge surface should extend
- Signal A through D already closed the runtime substrate law, so the next
  honest step is freezing the bridge law that composes truth basis and compute
  basis

This milestone belongs before the merged Query temporal/async milestone because
Query should project a closed lower-authority basis/causality model rather than
inventing one.

## Notes For Follow-On Work

- Query should later consume this milestone as a prerequisite and should treat:
  - temporal bridge basis
  - async request identity
  - completion causality
  - mixed-cause ordering
  - restart/resume basis
  as fixed lower-authority contracts.
- Server should later consume the delivery and resume artifacts from this
  milestone through Query-facing projection layers rather than reopening bridge
  semantics at the transport boundary.
