# Milestone 13.2 Engineering Spec: Subscription Support Through Retention, Compatibility, Replication, And Maintenance

> **Status:** Planned
>
> **Roadmap parent:** [worth_store_roadmap.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/worth_store_roadmap.md)
>
> **Vision parent:** [worth_store_vision.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/worth_store_vision.md)
>
> **Test requirements:** [test-requirements.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/test-requirements.md)
>
> **Prerequisite milestone:**
> - [milestone-13.1.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-13.1.md)
> - [milestone-13.1-closeout.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-13.1-closeout.md)
>
> **Follow-on milestone:**
> - `Milestone 13.3` (`Subscription Support Accuracy Taxonomy And Certification`)
>
> **Primary architectural driver:** make first-class subscription-support
> artifacts participate explicitly in retention, compatibility, replication,
> and maintenance programs without letting any of those programs become shadow
> subscription authority.

## Goal

Thread the durable subscription-support artifacts from Milestone 13.1 through
retention, compatibility, replication, and maintenance so the store can report
one typed support posture after those programs act:

- `ExactResumePreserved`
- `DegradedResumePreserved`
- `RebuildRequired`
- `NotResumable`
- `RejectedByPolicy`

The milestone is complete only when support artifacts stop being "special rows
that happen to persist" and become admitted participants in the store programs
that can keep, compact, migrate, replicate, rebuild, or reject durable
artifacts.

## Why This Milestone Exists

Milestone 13.1 made subscription-support artifacts durable, family-aware,
basis-linked, compatibility-bound, and honest about restart-time resume
classification.

That closed the first hard gap, but it intentionally left named debt:

- retention and reclaim do not yet know how to preserve or degrade support
  posture
- compatibility and version-skew do not yet propagate support-family outcomes
  through rolling readers
- replication and capsules do not yet include, omit, or reject
  subscription-support scopes with explicit portability evidence
- maintenance can name `Milestone13_2Required` rebuild debt, but cannot yet
  execute, refresh, or degrade support families through the common scheduler
  boundary

Milestone 13.2 exists to remove that debt without widening store authority.

The store still does not own query semantics, subscription lowering, subscriber
fanout, or delivery sessions. It owns the survival and portability of the
support artifacts it admitted in Milestone 13.1. When another store program
touches those artifacts, the result must be typed and auditable rather than
inferred later from missing records, stale cursors, or operator memory.

## Hard Part

The hard part is that four existing store programs can all change the physical
or operational status of a support artifact while none of them is allowed to
change subscription meaning:

- retention can keep, compact, reclaim, or expire supporting records
- compatibility can admit, migrate, degrade, or reject family versions
- replication and capsules can include, omit, defer, or fail support bundles
- maintenance can refresh, rebuild, defer, or escalate support-family work

The design fails if these programs produce independent, incompatible stories.

For example:

- retention says an artifact was legally reclaimed
- replication omits that artifact without a portability report
- compatibility accepts the artifact version as decodable
- resume classification still reports `ExactResume`

That is shadow authority by inconsistency.

Milestone 13.2 must therefore define one participation ledger per support
artifact family, one support-survival verdict model, one support portability
model, and one maintenance admission story. Each store program can still own its
domain, but they must publish subscription-support consequences through the same
typed result vocabulary.

## Explicit Assumptions

- Milestone 13.1 durable support identity, family catalog, basis linkage,
  compatibility binding, restart shards, and resume classification are the
  foundation this milestone consumes.
- Milestone 10 remains the owner of retention, compaction, reclaim, retained
  authority, and rebuild legality.
- Milestone 11 remains the owner of maintenance scheduling, pacing, foreground
  isolation, debt escalation, and restart-recovered work admission.
- Milestone 12 remains the owner of artifact format evolution, version windows,
  compatibility manifests, rolling compatibility, and restore compatibility.
- Milestone 13 remains the owner of placement, recall, hot/warm/cold residency,
  and cost-only tier posture.
- Milestone 14 will own final replication and capsule integrity, but Milestone
  13.2 must provide the subscription-support inclusion and portability rules
  that Milestone 14 consumes.
- Milestone 13.3 will own final trust/accuracy classification and certification
  posture. Milestone 13.2 must produce the operational evidence that 13.3
  classifies.
- Unsupported subscription-support families may remain absent. Admitted families
  may not remain ambiguous about retention, compatibility, replication, or
  maintenance participation.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects is adversarial design before feature
  packaging. This spec therefore starts from exact-resume loss under retention,
  compatibility, replication, and maintenance pressure, rather than from a
  helper to "include subscription artifacts."
- `arch_laws.md`
  The most important thing it protects is authority separation through
  proof-bearing phases. This milestone must make support survival, portability,
  compatibility, and maintenance verdicts typed outputs of their owners instead
  of rediscovered runtime facts.
- `perf_laws.md`
  The most important thing it protects is named cost boundaries. Support
  retention, capsule inclusion, compatibility admission, and rebuild scheduling
  must expose exact breadth counters rather than hiding scans behind cheap
  resume or export APIs.
- `domain_laws.md`
  The most important thing it protects is decomposition by reason-to-change.
  Retention participation, compatibility participation, replication
  portability, maintenance rebuild, and operator reporting must be distinct
  subdomains with a shared verdict vocabulary, not one generic subscription
  persistence module.
- `worth_store_vision.md`
  The most important thing it protects is that store makes truth and support
  artifacts survive without owning runtime semantics. Milestone 13.2 preserves
  that boundary by making operational programs publish support consequences
  without deciding what subscriptions mean.
- `worth_store_roadmap.md`
  The most important thing it protects is sequencing. Milestone 13.2 belongs
  after 13.1 because support artifacts now have durable identity, and before
  13.3 and 14 because trust classification and replication need participation
  rules first.
- `test-requirements.md`
  The most important thing it protects is machine-checkable proof. Milestone
  13.2 is not closeable until `Subscription-Support Retention, Replication,
  Compatibility, And Maintenance Test` proves support posture is preserved or
  explicitly degraded through hostile lanes.
- `milestone-13.md` and `milestone-13-closeout.md`
  The most important thing they protect is placement non-authority. Tier recall
  can change cost and maintenance timing for support artifacts, but not their
  resume classification.
- `milestone-13.1.md` and `milestone-13.1-closeout.md`
  The most important thing they protect is family-aware durable support identity
  and typed resume classification. Milestone 13.2 must consume those surfaces
  and remove the named `Milestone13_2Required` debt.
- `worth_store_dependency_map.md`
  The most important thing it protects is unlock shape. Milestone 13.1 unlocks
  this milestone; this milestone unlocks 13.3 by making operational survival
  explicit enough to classify and certify.

## Adversarial Constraint

Milestone 13.2 must survive this hostile condition:

> A store under retention pressure, compaction and reclaim, rolling
> compatibility skew, capsule export/import, partial replication,
> background-maintenance delay, restart after maintenance interruption, and tier
> recall of support records must publish the same family-aware
> subscription-support consequence as a control lane: exact resume preserved,
> degraded resume preserved, rebuild required, not resumable, or policy
> rejected. No lane may silently report exact resumability after the support
> basis, compatibility window, retained inputs, or portability scope needed for
> exactness has been lost.

## Product Decision Lock

- subscription-support participation is an operational survival contract, not
  subscription authority
- retention owns keep/reclaim legality; subscription support owns the resulting
  resume posture report
- compatibility owns version admission; subscription support owns the
  family-aware consequence of admission, migration, degradation, or rejection
- replication owns capsule and artifact transport; subscription support owns the
  inclusion, omission, and portability verdict for admitted support families
- maintenance owns scheduling and pacing; subscription support owns rebuild,
  refresh, degradation-recovery, and debt descriptors for support families
- `ExactResumePreserved` requires retained basis, cursor/checkpoint linkage,
  support artifact identity, family compatibility, and portability scope
  evidence
- support loss must lower to `DegradedResumePreserved`, `RebuildRequired`,
  `NotResumable`, or `RejectedByPolicy`; it may not disappear into ambient
  fallback
- a replicated or imported support artifact may not claim stronger resumability
  than the source participation ledger proves
- rebuild work must enter the Milestone 11 maintenance admission boundary; no
  hidden subscription worker loop may execute unbounded rebuilds
- Milestone 13.3 must inherit the verdicts and evidence produced here rather
  than reinterpreting raw operational logs

## Scope

### In Scope

- retention-facing survival rules for admitted subscription-support families
- compaction and reclaim consequences for exact, degraded, rebuildable, and
  non-resumable support posture
- compatibility/version-skew participation rules for support-family records,
  manifests, declarations, and opaque payload versions
- subscription-support capsule and replication participation rules for full,
  partial, deferred, omitted, and rejected scopes
- maintenance work descriptors for support rebuild, refresh, compatibility
  migration, degradation recovery, and stale-support cleanup
- operator-visible support survival reports after retention, compatibility,
  replication, and maintenance action
- exact counters and certification bundle rows for the Milestone 13.2 named
  suite

### Explicitly Out Of Scope

- defining new subscription semantics or delivery policy
- server delivery sessions, fanout restoration, retry windows, or network
  lifecycle persistence
- extension-defined subscription-support families, except typed rejection or
  explicit deferral until Milestone 15
- final trust/accuracy taxonomy and generic/domain certification coverage,
  which are Milestone 13.3
- final replication/capsule integrity closure for all store artifacts, which is
  Milestone 14

## Required Contracts And Counters

### Resume Classification Translation Rule

Milestone 13.2 verdicts are not replacements for Milestone 13.1 resume
classifications. They are operational consequences that must lower back into
Milestone 13.1 classification inputs.

Required translation surfaces:

- `SubscriptionSupportOperationalVerdict`
- `ResumeClassificationTranslationPlan`
- `PostActionResumeClassificationInput`
- `ExactResumePreservationWitness`
- `DegradedResumePreservationWitness`
- `SupportRebuildAdmissionWitness`
- `SupportNonResumableWitness`
- `SupportPolicyRejectionWitness`

Rules:

- `ExactResumePreserved` may lower to `ExactResume` only through
  `ExactResumePreservationWitness`
- `DegradedResumePreserved` may lower only to `DegradedButRecoverable`
- `RebuildRequired` may lower only to a rebuild-plan handle, never a resume
  handle
- `NotResumable` and `RejectedByPolicy` may lower only to denied reports
- translation consumes proof-bearing retained basis, cursor/checkpoint,
  support identity, compatibility, and portability evidence
- translation may not inspect raw support rows, raw capsule entries, or raw
  maintenance queue records
- any operational verdict without a legal translation target is rejected before
  resume classification runs

Naive trap this prevents:

- treating `ExactResumePreserved` as a renamed `ExactResume` enum variant and
  letting code bypass the Milestone 13.1 exact-resume proof chain

### Subscription-Support Participation Ledger Rule

Every admitted support artifact family must have one operational ledger entry
that records how store programs are allowed to affect it.

Required surfaces:

- `SubscriptionSupportParticipationLedger`
- `SubscriptionSupportProgramParticipant`
- `SubscriptionSupportSurvivalVerdict`
- `SubscriptionSupportOperationalBasis`
- `SubscriptionSupportPostActionReport`

Required verdicts:

- `ExactResumePreserved`
- `DegradedResumePreserved`
- `RebuildRequired`
- `NotResumable`
- `RejectedByPolicy`

Rules:

- every verdict names family id, support role, basis id, cursor/checkpoint id,
  compatibility window, and action origin
- a verdict cannot be constructed from missing raw rows alone; it must consume
  proof-bearing basis, family, and compatibility evidence
- exact preservation is illegal if any required basis, cursor/checkpoint,
  support digest, compatibility, or portability witness is missing
- degraded or rebuild-required outcomes must carry the exact missing or weakened
  support condition
- operator reporting consumes ledger verdicts rather than reconstructing support
  posture from logs

Naive trap this prevents:

- storing a mutable `last_known_resume_state` field beside the support record
  and letting later programs update it opportunistically without replayable
  action evidence

### Retention And Reclaim Participation Rule

Retention may keep, compact, reclaim, or expire support artifacts only through a
family-aware support survival plan.

Required surfaces:

- `SubscriptionSupportRetentionPlan`
- `RetainedSupportArtifactSet`
- `ReclaimedSupportArtifactSet`
- `CompactedSupportBasis`
- `SupportRetentionSurvivalWitness`
- `SupportReclaimConsequence`

Rules:

- support retention planning consumes Milestone 10 retained-range and rebuild
  legality proof rather than inventing support-local retention policy
- reclaim may remove support artifacts only if the resulting support verdict is
  published before the reclaim is treated as complete
- compaction products remain non-authoritative support accelerators unless
  their retained basis proves exact resume preservation
- a support artifact whose retained rebuild basis has also been reclaimed is
  `NotResumable`, not `RebuildRequired`
- retention windows may differ by support family, but every difference must be
  family-declared and countered

Naive trap this prevents:

- reclaiming support payload bytes because canonical truth is still retained,
  then allowing exact resume to survive because the support row still has a
  cursor id

### Compatibility Propagation Rule

Compatibility checks must propagate through support declarations, family
catalog entries, manifests, opaque payload versions, and participation-ledger
records before a support artifact is semantically exposed after upgrade or
version skew.

Required surfaces:

- `SubscriptionSupportCompatibilityPlan`
- `SupportFamilyVersionWindow`
- `SupportManifestAdmissionWitness`
- `SupportCompatibilityMigrationPlan`
- `SupportCompatibilityDegradation`
- `SupportVersionSkewRejection`

Rules:

- decode success is not compatibility admission
- old readers reject new support families or payload versions explicitly
- new readers admit old support records only through declared family windows
- compatible migration may preserve exactness only when digest basis and
  declared resume classifier remain equivalent
- compatibility drift can degrade or reject support posture without changing
  canonical truth
- every version-skew rejection emits a typed failure with family and version
  context

Naive trap this prevents:

- accepting support artifacts because the row decodes, even though the decoded
  payload no longer proves the same support role or resume classifier

### Replication And Capsule Participation Rule

Replication and capsules must carry support artifacts through explicit
portability plans instead of copying whatever support rows happen to be local.

Required surfaces:

- `SubscriptionSupportPortabilityPlan`
- `ReplicatedSupportBundle`
- `CapsuleSupportManifest`
- `SupportPortabilityScope`
- `SupportOmissionReport`
- `SupportImportAdmissionWitness`

Rules:

- a capsule that includes a support artifact must include enough basis,
  cursor/checkpoint, family manifest, and compatibility evidence for the target
  to classify the support posture
- partial replication may omit support artifacts only through an explicit
  omission report
- omitted support cannot report exact resumability on the target
- imported support artifacts preserve source identity only when scope,
  compatibility, and digest basis match
- unsupported family portability fails typed rather than becoming cursor-only
  resume
- support portability verdicts are subordinate to replication integrity; they do
  not replace Milestone 14 digest-graph proof

Naive trap this prevents:

- copying subscription-support rows into a capsule while omitting the basis,
  cursor/checkpoint, family manifest, or omission report needed for the target
  to classify resumability honestly

### Support Action Atomicity Rule

Any store program that changes subscription-support posture must publish one
self-describing action envelope. The envelope is the replayable bridge between
the program's domain action and the support consequence ledger.

Required surfaces:

- `SubscriptionSupportActionIntent`
- `SubscriptionSupportActionPlan`
- `PlannedSupportAction`
- `ExecutedSupportAction`
- `SupportActionPublicationWitness`
- `SupportActionRollbackRecord`
- `SupportConsequenceEnvelope`

Required phases:

1. plan the program action and support consequence together
2. verify retained basis, compatibility, portability, or maintenance proofs
3. execute the physical program action
4. publish the support consequence envelope
5. cut over program completion only after the consequence envelope is durable
6. record rollback or recovery disposition if interruption occurs before cutover

Rules:

- retention, compatibility, replication, import, export, and maintenance may not
  mark their work complete until the support consequence envelope is durable
- a crash after physical action but before consequence publication must recover
  to either a completed envelope or a typed interrupted-action disposition
- support consequence envelopes are derived durable artifacts; they explain
  operational posture but do not become truth authority
- rollback must be derivable from the action envelope and program-owned physical
  action record, not from ad hoc backend inspection
- repeated restart must be quiescent once an action has a published consequence
  envelope

Naive trap this prevents:

- completing reclaim or capsule export first and writing the support verdict
  afterward, leaving crash windows where exact resume is unknowable

### Required Access Structures Rule

Milestone 13.2 must ship concrete lookup structures for the common support
participation paths. A backend that lacks one must report typed access-structure
debt and fail certification lanes that require verified boundedness.

Required access structures:

- family id -> admitted support artifacts
- support artifact id -> operational participation ledger entries
- stable basis id -> support artifacts that depend on that basis
- cursor/checkpoint id -> support artifacts that depend on that cursor or
  checkpoint
- compatibility manifest id -> support family versions admitted under that
  manifest
- support portability scope -> included and omitted support artifacts
- support maintenance key -> in-flight rebuild, refresh, or migration work
- support action id -> consequence envelope and recovery disposition

Rules:

- retention planning may not scan all support history to find affected support
  artifacts for one retained range
- import admission may not scan target global support rows to detect one capsule
  support identity conflict
- maintenance coalescing may not deduplicate by stringified debug output or
  queue position
- startup recovery may not infer incomplete support actions by walking raw
  backend files or row remnants
- missing required access structures are explicit `Debt` only where the affected
  certification lane is also marked unsupported; they cannot be debt for the
  required Milestone 13.2 suite

Naive trap this prevents:

- first shipping correct-looking support participation that is secretly O(total
  support history) and becomes unfixable once callers depend on cheap-looking
  APIs

### Compile-Time Boundary Rule

The highest-risk support-participation states must be impossible to synthesize
or pass out of order.

Required proof-bearing surfaces:

- `ExactResumePreservationWitness`
- `DegradedResumePreservationWitness`
- `SupportRebuildAdmissionWitness`
- `SupportNonResumableWitness`
- `SupportPolicyRejectionWitness`
- `SupportRetentionSurvivalWitness`
- `SupportManifestAdmissionWitness`
- `SupportImportAdmissionWitness`
- `SupportMaintenanceAdmissionWitness`
- `SupportActionPublicationWitness`

Required typestate flow:

```text
RawSupportProgramAction
  -> PlannedSupportAction
  -> ProofCheckedSupportAction
  -> ExecutedSupportAction
  -> PublishedSupportConsequence
  -> CompletedSupportProgramAction
```

Rules:

- support verdict constructors are crate-owned or sealed; external code cannot
  synthesize exact-preserved, rebuild-admitted, or import-admitted witnesses
- `CompletedSupportProgramAction` cannot be constructed without
  `SupportActionPublicationWitness`
- `ExactResumePreserved` cannot be constructed from raw support identity,
  retained range, cursor/checkpoint id, or compatibility manifest id alone
- `RebuildRequired` cannot be constructed unless retained-basis proof,
  family rebuildability proof, and maintenance-admission proof are present
- capsule import cannot expose support artifacts semantically before
  `SupportImportAdmissionWitness`
- maintenance execution cannot consume a support rebuild descriptor before
  `SupportMaintenanceAdmissionWitness`
- compatibility migration cannot publish exact preservation without a
  `SupportManifestAdmissionWitness` and classifier-equivalence proof
- retention/reclaim cannot retire physical support records before the
  consequence envelope is published

Required proof surface:

- compile-fail tests for constructing `ExactResumePreserved` from raw support
  ids
- compile-fail tests for constructing `RebuildRequired` without retained-basis
  and maintenance-admission witnesses
- compile-fail tests for completing reclaim before support consequence
  publication
- compile-fail tests for semantic import access before import admission
- compile-fail tests for using decoded compatibility rows as support
  compatibility proof
- compile-fail tests for enqueueing support maintenance work without a
  maintenance-admission witness
- compile-fail tests for translating `DegradedResumePreserved`,
  `RebuildRequired`, `NotResumable`, or `RejectedByPolicy` into exact resume
  handles
- compile-fail tests for direct mutation of participation ledgers without a
  support action envelope

Naive trap this prevents:

- correctly documenting the support lifecycle while leaving enough public
  constructors that a tired implementation can skip half the proof chain

### Maintenance Rebuild And Refresh Rule

Support rebuild and refresh work must enter the existing maintenance admission
boundary as explicit work descriptors.

Required surfaces:

- `SubscriptionSupportMaintenanceDescriptor`
- `SupportRebuildWorkDescriptor`
- `SupportRefreshWorkDescriptor`
- `SupportCompatibilityMigrationWork`
- `SupportDegradationRecoveryWork`
- `SupportMaintenanceAdmissionWitness`
- `SupportMaintenanceDebtReport`

Rules:

- no support rebuild may execute directly from resume classification
- rebuild descriptors name retained basis, support family, cursor/checkpoint,
  compatibility window, and missing record families
- refresh work is distinct from rebuild work; refreshing stale derived support
  cannot pretend to recreate missing exact support unless basis proof permits it
- maintenance delay changes operational posture or debt, not truth
- interrupted maintenance restarts through the same descriptor admission model
- duplicate rebuild or refresh requests coalesce by a support-maintenance key

### Operator Resumability Report Rule

Every store program that acts on support artifacts must produce an
operator-visible support consequence report.

Required surfaces:

- `SubscriptionSupportResumabilityReport`
- `SupportProgramActionOrigin`
- `SupportActionConsequenceMatrix`
- `SupportPolicyRejectionReport`
- `SupportMaintenanceDebtSummary`

Rules:

- reports distinguish exact preserved, degraded preserved, rebuild required,
  not resumable, and policy rejected
- reports distinguish retention loss, compatibility drift, replication omission,
  maintenance delay, maintenance failure, and tier recall cost
- diagnostics richness may add detail but may not change the verdict
- reports include enough machine-checkable fields for certification bundle
  comparison

### Performance-Shaping Types Rule

Milestone 13.2 must encode cost shape before execution. The planner must lower
support-affecting work into family-local, basis-local, scope-local, or explicit
store-global plans so the executor consumes bounded work instead of discovering
breadth while it runs.

Required performance-shaping surfaces:

- `SupportParticipationBreadthSummary`
- `SupportFamilyLocalityFootprint`
- `SupportBasisLocalityFootprint`
- `SupportPortabilityScopeFootprint`
- `SupportMaintenanceLocalityFootprint`
- `SupportActionBreadthBudget`
- `SupportProgramDensityClass`
- `SupportBatchAdmissionReceipt`
- `SupportAllocationScope`
- `SupportHotPathRejection`

Required density classes:

- `SingleSupportArtifact`
- `FamilyLocalBatch`
- `BasisLocalBatch`
- `PortabilityScopeBatch`
- `MaintenanceKeyBatch`
- `StoreGlobalDebt`

Rules:

- support-affecting execution consumes a lowered breadth summary; it may not
  query indexes repeatedly to rediscover the same affected set
- `StoreGlobalDebt` is an explicit debt-bearing density class and cannot close
  required Milestone 13.2 certification lanes
- a hot read or resume-classification path may not run retention, portability,
  compatibility migration, or maintenance batch work inline
- repeated compatibility, basis, or cursor/checkpoint checks inside one
  admitted batch must reuse a `SupportBatchAdmissionReceipt`
- allocation-heavy work must declare a `SupportAllocationScope` before reading
  opaque payloads, constructing reports, or materializing capsule support
  manifests
- `SupportHotPathRejection` is the required outcome when a caller attempts a
  batch, migration, import, or maintenance-class action through a cheap-looking
  resume/read API
- family-local and basis-local plans must carry canonical ordering so result
  digests and counter snapshots are stable across backends

Naive trap this prevents:

- implementing support participation as a loop over scalar 13.1 classification
  calls and reporting it as a batch-capable architecture

### Bounded Planning And Batch Reuse Rule

Every support-affecting program must separate planning breadth from execution
work and carry reusable proof across the batch boundary.

Required surfaces:

- `SupportRetentionBatchPlan`
- `SupportCompatibilityBatchPlan`
- `SupportPortabilityBatchPlan`
- `SupportMaintenanceBatchPlan`
- `SupportAffectedSet`
- `SupportAffectedSetDigest`
- `SupportBatchProofCache`
- `SupportBatchExecutionReceipt`

Rules:

- a batch plan materializes one canonical affected set before execution
- affected sets are typed by family, basis, portability scope, or maintenance
  key; raw `Vec<SupportArtifactId>` cannot cross from planning to execution
- support batch execution consumes the affected-set digest and proof cache,
  not the original broad query
- batch proof caches are phase-local proofs, not durable authority
- repeated support actions over the same affected set must either reuse the
  batch receipt or produce a typed invalidation reason
- a partial batch failure must report which affected set entries executed,
  which were rejected, and which remained untouched without rescanning the store

Naive trap this prevents:

- re-running support-family queries inside every phase and accidentally turning
  a scoped retention or capsule operation into O(scope x total_support_rows)

### Allocation And Payload Budget Rule

Support participation must reject oversized or allocation-hostile work before it
constructs rich artifacts.

Required surfaces:

- `SupportParticipationPayloadBudget`
- `SupportReportMaterializationBudget`
- `SupportCapsuleManifestBudget`
- `SupportActionEnvelopeBudget`
- `SupportBatchArena`
- `SupportPayloadBudgetRejection`

Rules:

- opaque support payloads are measured before decode, migration, import, or
  report materialization
- capsule support manifests declare entry count, payload-header bytes, omission
  count, and required-basis count before allocation
- action envelopes declare maximum diagnostic cause count and report size before
  publication
- batch-local allocations use `SupportBatchArena` or an equivalent lifecycle
  scope that is cleared after the admitted unit
- general-purpose per-artifact allocation inside family-local or scope-local
  loops is explicit `Debt`
- budget rejection happens before compatibility migration, payload decode, or
  report construction

Naive trap this prevents:

- building giant support reports or capsule manifests as unbounded vectors and
  discovering memory pressure after the store has already begun the action

### Path Separation Rule

Support participation must keep operational paths separate from foreground
resume and read paths.

Required path classes:

- `SupportForegroundResumePath`
- `SupportForegroundReadPath`
- `SupportOperationalPlanningPath`
- `SupportMaintenanceExecutionPath`
- `SupportReplicationExportPath`
- `SupportImportAdmissionPath`
- `SupportOperatorReportingPath`

Rules:

- foreground resume may consume an already-published operational verdict; it may
  not compute retention, replication, compatibility migration, or maintenance
  consequences inline
- foreground read may observe support posture and counters, but may not trigger
  support rebuild, refresh, or capsule manifest construction
- import admission and export planning are orchestration boundaries, not getter
  methods
- operator reporting may materialize rich diagnostics only after the operational
  verdict exists
- path class must be carried in result cost surfaces and counter snapshots

Naive trap this prevents:

- hiding slow support repair, export preparation, or compatibility migration
  behind a resume API that looks like a cheap classification call

### Complexity Contracts

Minimum named paths:

- `subscription_support_survival_planning`
- `subscription_support_retention_consequence`
- `subscription_support_compatibility_admission`
- `subscription_support_portability_planning`
- `subscription_support_import_admission`
- `subscription_support_maintenance_admission`
- `subscription_support_operator_reporting`
- `subscription_support_batch_plan_lowering`
- `subscription_support_affected_set_materialization`
- `subscription_support_batch_receipt_reuse`
- `subscription_support_payload_budget_admission`
- `subscription_support_path_class_rejection`

Minimum complexity contracts:

- support survival planning cost is proportional to admitted support families,
  retained-basis checks, and support artifacts in scope, not all subscription
  history
- retention consequence cost is proportional to support artifacts affected by
  the retention action and retained-basis checks needed
- compatibility admission cost is proportional to family manifest entries,
  version-window checks, and support payload headers admitted before semantic
  exposure
- portability planning cost is proportional to declared support scope,
  included support artifacts, omitted support artifacts, and required basis
  artifacts
- import admission cost is proportional to capsule support entries and
  compatibility windows checked, not target-store global support rows
- maintenance admission cost is proportional to unique support-maintenance keys
  and retained-basis descriptors
- operator reporting cost is proportional to emitted support action verdicts,
  not historical logs
- batch plan lowering cost is proportional to support families and locality
  footprints in the admitted program action
- affected-set materialization cost is proportional to affected support
  artifacts in the declared scope, not total support history
- batch receipt reuse cost is O(1) per repeated proof lookup inside the admitted
  batch
- payload budget admission cost is proportional to payload headers and manifest
  entry counts, not payload body bytes
- path class rejection cost is O(1) after API entry classification

Minimum counters:

- `subscription_support_retained_family_count`
- `subscription_support_reclaimed_family_count`
- `subscription_support_compacted_basis_count`
- `subscription_support_exact_preserved_count`
- `subscription_support_degraded_preserved_count`
- `subscription_support_rebuild_required_count`
- `subscription_support_not_resumable_count`
- `subscription_support_policy_rejection_count`
- `subscription_support_replicated_bundle_count`
- `subscription_support_capsule_inclusion_count`
- `subscription_support_capsule_omission_count`
- `subscription_support_import_admission_count`
- `subscription_support_version_skew_rejection_count`
- `subscription_support_compatibility_degradation_count`
- `subscription_support_maintenance_descriptor_count`
- `subscription_support_maintenance_rebuild_debt_count`
- `subscription_support_maintenance_refresh_count`
- `subscription_support_maintenance_coalesced_count`
- `subscription_support_hidden_exact_loss_count`
- `subscription_support_batch_plan_lowering_count`
- `subscription_support_affected_set_entry_count`
- `subscription_support_batch_receipt_reuse_count`
- `subscription_support_store_global_debt_count`
- `subscription_support_hot_path_rejection_count`
- `subscription_support_payload_budget_rejection_count`
- `subscription_support_batch_arena_allocation_count`
- `subscription_support_per_artifact_allocation_debt_count`

Required counter assertions:

- `subscription_support_hidden_exact_loss_count` remains zero in all admitted
  lanes
- `subscription_support_exact_preserved_count` increments only when all exact
  support evidence survives the action
- `subscription_support_rebuild_required_count` increments only when retained
  rebuild basis and family rebuildability are proven
- `subscription_support_not_resumable_count` increments when retained basis,
  cursor/checkpoint, family, or compatibility proof required for rebuild is
  missing
- `subscription_support_capsule_omission_count` increments for every omitted
  support family in partial replication or capsule scopes
- `subscription_support_version_skew_rejection_count` increments before
  semantic exposure of incompatible support records
- `subscription_support_maintenance_coalesced_count` increments when duplicate
  rebuild or refresh demand is suppressed
- `subscription_support_store_global_debt_count` remains zero in required
  Milestone 13.2 certification lanes
- `subscription_support_hot_path_rejection_count` increments when operational
  work is attempted through foreground resume/read APIs
- `subscription_support_batch_receipt_reuse_count` increments for repeated
  compatibility, basis, cursor/checkpoint, or portability checks inside one
  admitted batch
- `subscription_support_payload_budget_rejection_count` increments before
  payload decode or report materialization in oversized lanes
- `subscription_support_per_artifact_allocation_debt_count` remains zero in
  verified family-local and scope-local batch lanes

Additional required counters:

- `subscription_support_action_envelope_publish_count`
- `subscription_support_action_interrupted_recovery_count`
- `subscription_support_access_structure_debt_count`
- `subscription_support_global_scan_attempt_reject_count`
- `subscription_support_translation_rejection_count`
- `subscription_support_import_semantic_access_reject_count`

Additional required counter assertions:

- `subscription_support_action_envelope_publish_count` equals the number of
  completed support-affecting program actions
- `subscription_support_action_interrupted_recovery_count` increments for crash
  lanes between physical action and consequence publication
- `subscription_support_access_structure_debt_count` remains zero for required
  Milestone 13.2 certification lanes
- `subscription_support_global_scan_attempt_reject_count` increments in hostile
  lanes that attempt to recover support participation by scanning raw backend
  residue
- `subscription_support_translation_rejection_count` increments when an
  operational verdict is not legal input for the requested Milestone 13.1
  resume handle
- `subscription_support_import_semantic_access_reject_count` increments when
  imported support is accessed before import admission

## First-Ship Policy

Milestone 13.2 must ship a conservative policy model that is useful immediately
and hard to misread.

Required first-ship participation policies:

- `BasisBoundContinuationSupport`
  Exact preservation is allowed only when retained basis, cursor/checkpoint,
  family manifest, and support identity survive the action unchanged or through
  a proven compatible migration.
- `MaterializedNarrowingSupport`
  Exact preservation is allowed only when the materialized narrowing descriptor
  or a retained exact rebuild basis survives. If the descriptor is reclaimed and
  exact rebuild basis remains, the outcome is `RebuildRequired`.
- `DegradedContinuationSupport`
  Retention, compatibility, replication, and maintenance may preserve degraded
  posture, but must never upgrade it to exact.

Explicit first-ship debt:

- predictive preservation of support artifacts based on future subscriber
  demand
- cross-family reconstruction where one support family rebuilds another
  family's support material
- partial replication that tries to synthesize exact support on the target from
  cursor truth alone
- compatibility migrations that require inspecting query semantics or bridge
  strategy internals
- maintenance refresh that changes declared support role or family kind

Rules:

- unsupported policies fail typed or remain explicit `Debt`
- no first-ship debt may permit silent exact preservation
- adding a new support family requires participation policy, access structures,
  action-envelope handling, counters, and certification matrix rows

## Required Internal Subsystems

Milestone 13.2 should extend the existing `subscription_support` domain with
separate responsibilities rather than growing one operational catch-all module.

Required subdomains:

- `subscription_support/participation/`
  participation ledger records, operational verdicts, action origins, and
  support consequence envelopes
- `subscription_support/retention/`
  retention plans, reclaim consequences, compacted support basis, and
  retained-basis witnesses
- `subscription_support/compatibility/`
  support family version windows, manifest admission witnesses, migration
  plans, and version-skew rejections
- `subscription_support/portability/`
  capsule manifests, support portability scopes, replicated bundles, omission
  reports, and import admission witnesses
- `subscription_support/maintenance/`
  rebuild, refresh, compatibility migration, degradation recovery descriptors,
  coalescing keys, and maintenance admission witnesses
- `subscription_support/actions/`
  support action intents, typestate progression, publication witnesses,
  rollback records, and interrupted-action recovery
- `subscription_support/access/`
  family, basis, cursor/checkpoint, compatibility, portability, maintenance, and
  action lookup structures
- `subscription_support/reporting/`
  operator-visible resumability reports and action consequence matrices
- `subscription_support/evidence/milestone_13_2/`
  counters, complexity surfaces, certification bundles, and matrix validation

Rules:

- backend persistence may store rows, indexes, and action envelopes, but it may
  not decide operational support verdicts locally
- retention, compatibility, portability, and maintenance code consume
  proof-bearing support summaries rather than raw backend rows
- tests should follow the same responsibility split; one broad
  `subscription_support_operations.rs` test file is not acceptable for the
  final milestone

## Required Durable Record Families

Milestone 13.2 must be implementable as concrete durable record families, not
as ad hoc metadata bolted onto 13.1 support records.

Required record families:

- `SupportParticipationLedgerRecord`
  Append-only operational verdict record keyed by support artifact, family,
  basis, cursor/checkpoint, action origin, and consequence envelope id.
- `SupportActionEnvelopeRecord`
  Durable action envelope containing intent, planned consequence, proof summary,
  execution state, publication witness, and recovery disposition.
- `SupportRetentionParticipationRecord`
  Retention/reclaim/compaction participation row linking support artifacts to
  retained ranges, retained rebuild basis, and survival verdict.
- `SupportCompatibilityParticipationRecord`
  Version-window and manifest-admission row linking support artifacts to
  compatibility outcome, migration plan, degradation, or rejection.
- `SupportPortabilityManifestRecord`
  Capsule/replication participation row listing included support artifacts,
  omitted support artifacts, required basis artifacts, and import-admission
  requirements.
- `SupportMaintenanceDescriptorRecord`
  Maintenance descriptor row for rebuild, refresh, compatibility migration,
  degradation recovery, duplicate coalescing, and interrupted-work recovery.
- `SupportOperatorReportRecord`
  Machine-checkable report row derived from action envelopes and ledger records,
  not from free-form logs.

Rules:

- ledger and action envelope records are append-only for completed actions;
  correction requires a later compensating action envelope
- mutable backend convenience rows may exist, but cannot be the only durable
  explanation of a support consequence
- record identity must include family and support role; support artifact id
  alone is not enough
- record families must be rebuildable or auditable from the support action
  envelopes plus the relevant program-owned artifacts
- deleting support payload bytes may not delete the historical action envelope
  that explains why exact resume was degraded or denied

Forbidden implementation shortcuts:

- one nullable `resume_state` column on the support artifact row
- one JSON "metadata" blob that mixes retention, compatibility, portability,
  maintenance, and reporting facts
- inferring omission from absence in a capsule instead of writing an omission
  report
- treating a maintenance queue entry as the durable rebuild explanation
- using backend file presence or SQLite row presence as the support consequence
  ledger
- making operator reports by parsing human-readable log text

## Phases

### Phase 1: Lock Participation Ledger And Verdict Vocabulary

Phase 1 defines the single support consequence vocabulary all affected store
programs must publish.

Required work:

- define participation ledger records for admitted support families
- define operational action origins for retention, compatibility, replication,
  import, export, maintenance, restart, and tier recall
- define exact, degraded, rebuild-required, not-resumable, and policy-rejected
  verdicts
- define support operational basis fields consumed by all verdicts
- define translation between operational verdicts and Milestone 13.1 resume
  classification inputs
- define support action envelope typestate and publication witness rules
- define typed report structures for operator and certification evidence
- define compile-time construction boundaries for exact-preserved and
  rebuild-required verdicts
- define required access structures and missing-index debt behavior
- define performance-shaping types, density classes, path classes, allocation
  scopes, and batch-admission receipts
- define support-program counter contract and certification bundle shape

Exit condition:

- no store program can report support consequences with private booleans or
  missing-row inference
- exact preservation and rebuild-required outcomes require proof-bearing
  witnesses
- support-affecting actions have a single plan/proof/execute/publish/complete
  lifecycle
- every support-affecting program has a declared path class and density class
  before execution

### Phase 2: Thread Support Families Through Retention, Compaction, And Reclaim

Phase 2 makes support survival physically honest under Milestone 10 programs.

Required work:

- implement family-aware support retention planning
- implement `SupportRetentionBatchPlan` and `SupportAffectedSet` construction
  before retention execution
- implement retained, reclaimed, compacted, and expired support consequence
  records
- bind support retention to retained authority and rebuild-basis proof
- execute retention and reclaim through support action envelopes
- maintain support access structures for basis, cursor/checkpoint, and family
  reachability
- reject or degrade support exactness when required basis or support records are
  outside policy
- emit support survival reports from compaction and reclaim paths
- recover interrupted reclaim lanes without scanning raw backend remnants
- prove reclaimed exact support cannot still classify as exact
- reject store-global retention support sweeps from required certification lanes
  unless they carry explicit `StoreGlobalDebt`
- expose exact retention and reclaim counters

Exit condition:

- support artifacts can be kept, compacted, reclaimed, or expired only with a
  typed resumability consequence
- missing rebuild basis becomes `NotResumable`, not a vague rebuild promise
- retention support participation runs over typed affected sets rather than
  scalar rediscovery loops

### Phase 3: Propagate Support Compatibility And Version-Skew Outcomes

Phase 3 makes Milestone 12 compatibility rules family-aware for support
artifacts.

Required work:

- implement support family version-window admission
- implement `SupportCompatibilityBatchPlan` and batch admission receipt reuse for
  family/window checks
- implement manifest-backed support compatibility checks before semantic
  exposure
- implement compatible support migration plans where digest and classifier
  equivalence are proven
- execute compatibility migration through support action envelopes
- implement explicit compatibility degradation and version-skew rejection
  outcomes
- reject old-reader and unknown-family lanes typed
- reject decoded-row semantic access before manifest admission
- reject batch-class compatibility migration from foreground resume/read paths
- ensure compatibility drift updates the participation ledger and support
  consequence report
- expose exact compatibility counters

Exit condition:

- version skew can preserve, degrade, or reject support posture explicitly
- decode success cannot expose support artifacts without compatibility evidence
- repeated compatibility checks inside one admitted batch reuse proof receipts

### Phase 4: Define Replication, Capsule, And Import Participation

Phase 4 prepares support artifacts for Milestone 14 without closing all of
Milestone 14.

Required work:

- implement support portability plans for admitted families
- implement `SupportPortabilityBatchPlan`, manifest budgets, and portability
  scope footprints before capsule materialization
- implement capsule support manifests and replicated support bundles
- implement partial-scope omission reports
- implement target-side import admission witnesses
- implement action envelopes for export, import, and replication support
  consequences
- prove support identity preservation for admitted full-scope replication
- prove partial replication cannot report exact support for omitted artifacts
- reject target-side semantic access before support import admission
- reject oversized capsule support manifests before payload decode or allocation
- expose support inclusion, omission, replication, and import counters

Exit condition:

- replication and capsules can include, omit, defer, or reject support artifacts
  with typed portability evidence
- target stores classify imported support posture from capsule evidence rather
  than cursor folklore
- export and import costs are visible by scope, manifest entry count, omitted
  support count, and required basis count

### Phase 5: Admit Maintenance Rebuild, Refresh, And Degradation Recovery

Phase 5 removes the `Milestone13_2Required` operational debt left by 13.1.

Required work:

- define support rebuild, refresh, compatibility migration, and degradation
  recovery work descriptors
- implement `SupportMaintenanceBatchPlan` and maintenance-key locality
  footprints
- admit descriptors through the Milestone 11 maintenance boundary
- implement retained-basis checks before rebuild descriptor construction
- require `SupportMaintenanceAdmissionWitness` before maintenance execution
- implement duplicate rebuild/refresh coalescing
- reject maintenance-class work attempted from foreground read or resume paths
- implement interrupted-maintenance restart posture for support descriptors
- update missing-support classification to produce admitted maintenance
  descriptors where legal instead of `Milestone13_2Required`
- preserve maintenance delay as debt or degradation, not truth failure
- expose exact maintenance counters and debt reports

Exit condition:

- support rebuild and refresh work is schedulable through the common
  maintenance model
- hidden support-specific worker loops are unnecessary and out of spec
- 13.1 rebuild-required posture can become actionable when retained inputs
  exist
- maintenance coalescing and admission are keyed by typed locality, not debug
  strings or queue position

### Phase 6: Prove Subscription-Support Operational Participation

Phase 6 turns the milestone into certification-grade evidence.

Required work:

- run the Milestone 13.2 named suite:
  `Subscription-Support Retention, Replication, Compatibility, And Maintenance Test`
- include retained, compacted, reclaimed, and expired support lanes
- include full replication, partial replication, capsule omission, and import
  admission lanes
- include old-reader, new-reader, incompatible-family, and migrated-family
  compatibility lanes
- include maintenance rebuild, refresh, delayed, interrupted, and coalesced lanes
- include crash lanes between physical support action and consequence
  publication
- include access-structure debt and forbidden global-scan lanes
- include density-class lanes for single-artifact, family-local, basis-local,
  portability-scope, maintenance-key, and rejected store-global work
- include path-class rejection lanes for operational work attempted through
  foreground resume/read APIs
- include payload and allocation budget lanes for oversized support payloads,
  capsule manifests, action envelopes, and operator reports
- include batch-receipt reuse lanes proving repeated checks do not rediscover
  compatibility, basis, cursor/checkpoint, or portability proof
- include compile-fail coverage for sealed witnesses, typestate progression,
  import admission, maintenance admission, and exact-translation misuse
- compare exact, degraded, rebuild-required, not-resumable, and rejected lanes
- prove no lane reports exact resumability after support loss
- emit machine-checkable truth, artifact, subscription-support, failure, and
  counter bundles

Exit condition:

- store programs that act on support artifacts preserve or explicitly classify
  support resumability
- support participation cost is encoded in lowered plans, density classes,
  access structures, allocation scopes, and result cost surfaces before
  execution
- Milestone 13.2 closeout evidence exists in machine-checkable form

## Must Ship

- participation ledger for admitted subscription-support families
- operational-verdict translation plans back into Milestone 13.1 resume
  classification inputs
- performance-shaping types for breadth summaries, locality footprints, density
  classes, path classes, batch receipts, allocation scopes, and hot-path
  rejections
- typed support survival verdicts:
  - `ExactResumePreserved`
  - `DegradedResumePreserved`
  - `RebuildRequired`
  - `NotResumable`
  - `RejectedByPolicy`
- support action envelopes with plan/proof/execute/publish/complete typestate
- required access structures for family, basis, cursor/checkpoint,
  compatibility, portability, maintenance, and support-action lookups
- batch plans and affected-set proof wrappers for retention, compatibility,
  portability, and maintenance
- durable record families for participation ledgers, action envelopes,
  retention participation, compatibility participation, portability manifests,
  maintenance descriptors, and operator reports
- retention, compaction, reclaim, and expiration participation rules for support
  artifacts
- compatibility/version-skew participation rules for support family manifests,
  declarations, opaque payload versions, and ledger records
- support portability plans for replication, capsules, import, and partial-scope
  omission
- target-side support import admission witnesses
- maintenance descriptors for support rebuild, refresh, compatibility migration,
  and degradation recovery
- interrupted-maintenance restart posture for support descriptors
- operator-visible support resumability reports
- compile-time boundary coverage for exact-preserved, rebuild-required,
  import-admitted, maintenance-admitted, and action-publication witness
  construction
- exact counters for retention, reclaim, compatibility, portability, import,
  maintenance, action-envelope publication, access-structure debt, translation
  rejection, density class, batch reuse, allocation budget, hot-path rejection,
  and hidden exact-loss prevention
- machine-checkable Milestone 13.2 certification output

## Must Preserve

- canonical truth remains authoritative; support artifacts never become truth
  authority
- runtime bridge, `worth-signal`, `worth-query`, and server layers remain owners
  of subscription meaning, lowering, delivery, fanout, and lifecycle semantics
- Milestone 10 remains the authority for retention and reclaim legality
- Milestone 11 remains the authority for maintenance scheduling and pacing
- Milestone 12 remains the authority for compatibility admission
- Milestone 13 remains the authority for placement and recall cost posture
- Milestone 14 remains the authority for final replication integrity and capsule
  equivalence
- no compacted artifact, maintenance output, migrated support record, or
  replicated capsule becomes shadow subscription authority
- unsupported portability, rebuild, or compatibility paths fail typed rather
  than silently degrading into cursor-only resume

## Acceptance Evidence

Milestone 13.2 is complete only when the store satisfies the named Milestone
13.2 suite:

- `Subscription-Support Retention, Replication, Compatibility, And Maintenance Test`

Required machine-checkable outputs:

- `truth_digest`
- `artifact_digest`
- `subscription_support_digest`
- `failure_digest`
- `counter_snapshot`

Minimum certification matrix rows:

- `retention_exact_preserved`
  keeps all exact-support inputs and proves exact posture survives.
- `retention_degraded_preserved`
  retains enough support for degraded resume and proves exact is not reported.
- `reclaim_rebuild_required`
  reclaims support material while retaining rebuild basis and produces an
  admitted rebuild descriptor.
- `reclaim_not_resumable`
  reclaims support material and retained rebuild inputs, producing
  `NotResumable`.
- `compacted_support_basis_exact`
  compacts support basis while preserving exact resume through proof-bearing
  compacted basis evidence.
- `expired_support_policy_rejected`
  expires support by retention policy and reports `RejectedByPolicy`.
- `compatible_old_support_admitted`
  admits an older support family version through a declared compatibility
  window.
- `new_support_old_reader_rejected`
  rejects a newer support artifact before semantic exposure.
- `support_compatibility_degraded`
  admits a support record only as degraded after compatibility drift.
- `support_migration_exact_preserved`
  migrates support format while preserving digest basis and resume classifier
  equivalence.
- `full_replication_identity_preserved`
  replicates an admitted support scope and preserves support identity and
  digest posture on the target.
- `partial_replication_omission_reported`
  omits support artifacts by declared scope and proves exact resume is not
  reported on the target.
- `capsule_import_admitted_support`
  imports support from a capsule with basis, cursor/checkpoint, family, and
  compatibility evidence.
- `capsule_import_missing_basis_not_resumable`
  imports support without required basis evidence and reports `NotResumable`.
- `unsupported_family_portability_rejected`
  rejects support portability for an unsupported family rather than falling back
  to cursor-only resume.
- `maintenance_rebuild_descriptor_admitted`
  converts legal rebuild-required posture into Milestone 11 maintenance work.
- `maintenance_rebuild_basis_missing_denied`
  denies rebuild descriptor construction when retained basis is missing.
- `maintenance_refresh_degraded_support`
  refreshes stale support and reports degraded or exact posture according to
  proof, not best effort.
- `maintenance_interrupted_restart_recovered`
  restarts interrupted support maintenance through admitted descriptors.
- `maintenance_duplicate_coalesced`
  coalesces duplicate support rebuild or refresh demand.
- `family_local_batch_bounded`
  executes support consequences for one family-local batch and proves affected
  set breadth, proof reuse, and allocation counts match the declared batch.
- `basis_local_batch_bounded`
  executes support consequences for one stable-basis-local batch and proves no
  unrelated support family rows are touched.
- `portability_scope_batch_bounded`
  exports and imports a declared support portability scope and proves included,
  omitted, and required-basis counts match the manifest.
- `store_global_density_rejected`
  attempts support participation through an implicit whole-store sweep and proves
  the path is rejected or marked unsupported for the required suite.
- `foreground_resume_operational_work_rejected`
  attempts retention, compatibility migration, export planning, or maintenance
  execution through foreground resume and proves `SupportHotPathRejection`.
- `batch_receipt_reuse_verified`
  performs repeated compatibility, basis, cursor/checkpoint, and portability
  checks inside one admitted batch and proves receipt reuse counters.
- `payload_budget_rejected_before_materialization`
  submits oversized support payload, capsule manifest, action envelope, and
  operator report lanes and proves rejection before decode or allocation-heavy
  materialization.
- `action_publication_crash_recovered`
  crashes after physical support-affecting action but before support consequence
  publication and recovers to one typed interrupted or completed disposition.
- `access_structure_debt_rejected`
  removes a required support lookup structure and proves the required lane
  rejects or reports debt rather than scanning global support history.
- `global_scan_recovery_forbidden`
  attempts to recover support consequences from raw backend residue and proves
  the path is rejected.
- `operational_verdict_translation_rejected`
  attempts to translate degraded, rebuild-required, not-resumable, or
  policy-rejected verdicts into exact resume handles and proves rejection.
- `import_semantic_access_before_admission_rejected`
  attempts target-side semantic use of capsule support before import admission.
- `hidden_exact_loss_forbidden`
  injects support loss across retention, compatibility, replication, and
  maintenance lanes and proves exact resumability is never silently reported.

Milestone-specific proof obligations:

- exact support preservation requires retained basis, cursor/checkpoint,
  support digest, compatibility, and portability proof
- retention and reclaim publish typed support consequences before completion
- support artifacts with reclaimed rebuild bases are not resumable, not
  rebuild-required
- compatibility drift rejects or degrades before semantic exposure
- replication and capsules include or omit support artifacts through declared
  support scopes
- imported support cannot claim stronger posture than source and capsule
  evidence prove
- support-affecting store actions cannot complete before their consequence
  envelope is durable
- interrupted support actions recover without backend-residue guessing
- required access structures keep affected-family planning bounded and are not
  optional for the required named suite
- lowered breadth plans and affected-set wrappers prevent scalar rediscovery
  loops from masquerading as batch architecture
- density classes and path classes are carried into result cost surfaces and
  counter snapshots
- foreground resume/read paths reject operational work rather than hiding broad
  planning, migration, export, or maintenance cost
- payload, manifest, action-envelope, and report budgets reject before expensive
  materialization
- batch-admission receipts prevent repeated proof rediscovery inside one trusted
  batch boundary
- operational verdicts translate into Milestone 13.1 resume classifications only
  through legal proof-bearing translation plans
- rebuild and refresh work enters the Milestone 11 maintenance boundary through
  typed descriptors
- maintenance delay, interruption, and debt are operational posture, not hidden
  truth changes
- no certification lane relies on logs or same-run self-comparison as proof
- `subscription_support_hidden_exact_loss_count` remains zero
- `subscription_support_store_global_debt_count` remains zero in required
  certification lanes
- `subscription_support_hot_path_rejection_count`,
  `subscription_support_batch_receipt_reuse_count`, and
  `subscription_support_payload_budget_rejection_count` match the hostile and
  bounded performance lanes exactly
- `subscription_support_per_artifact_allocation_debt_count` remains zero in
  verified family-local, basis-local, portability-scope, and maintenance-key
  batch lanes
- compile-fail tests prevent sealed-witness synthesis, out-of-order action
  completion, import-before-admission, maintenance-before-admission, and
  exact-resume translation misuse

Milestone 13.2 is not closed by "a support row survived compaction" or "a
capsule included subscription metadata" tests.

## Architectural Notes

- The smart abstraction is not "subscription replication." The smart
  abstraction is family-aware operational participation for already-durable
  support artifacts.
- Retention, compatibility, replication, and maintenance should keep their own
  authority. This milestone defines the support-specific consequences they must
  publish.
- Exact resume is a fragile proof, not a default. Any weakened basis should
  degrade, rebuild, deny, or reject explicitly.
- Partial replication and capsule export are allowed to omit support artifacts,
  but omission must become target-visible support posture.
- Maintenance rebuild is an operational activity with debt and pacing. It is
  not permission for resume classification to run unbounded repair work inline.
- Milestone 13.3 should classify the trust posture produced here; it should not
  need to infer operational facts from raw support rows.

## Sequencing Notes

This milestone belongs immediately after Milestone 13.1 because 13.1 created
durable, family-aware support artifacts but deliberately left their operational
participation as named follow-on debt.

- Milestone 13.2 consumes 13.1 support identity, restart, and classification
  surfaces.
- Milestone 13.2 removes the `Milestone13_2Required` rebuild-execution debt by
  routing legal support rebuild and refresh work through Milestone 11
  maintenance descriptors.
- Milestone 13.2 must close before Milestone 13.3 can honestly assign final
  trust and accuracy posture to support artifacts.
- Milestone 14 should consume this milestone's support portability and capsule
  participation rules rather than defining subscription-support inclusion from
  scratch.
- Milestones 15, 17, 20, 21, and 22 may reference this milestone when deciding
  how extension families, derived accuracy, blobs, budgets, and operator repair
  interact with first-class subscription-support artifacts.

## Required Self-Check

- Does the milestone solve a real structural problem or just package work
  cosmetically?
  Yes. It closes the operational participation gap left by 13.1 so support
  artifacts cannot be retained, replicated, migrated, or rebuilt through
  incompatible private meanings.
- Is the adversarial constraint precise and load-bearing?
  Yes. Every phase maps back to preventing silent exact-resume claims after
  retention, compatibility, replication, or maintenance has weakened support
  evidence.
- Does the milestone preserve crate authority boundaries?
  Yes. Retention, maintenance, compatibility, placement, and replication remain
  owned by their existing store programs; subscription support publishes only
  support-specific consequences.
- Does the milestone define proof obligations, not just implementation tasks?
  Yes. It names the required certification suite, machine-checkable outputs,
  matrix rows, exact counters, and zero hidden-loss assertion.
- Could a competent engineer map this spec into honest types, modules, and
  tests?
  Yes. The spec names participation, survival, compatibility, portability,
  maintenance, and reporting surfaces plus ordered phases and test lanes.
- Does the milestone belong in this roadmap sequence, or is it out of order?
  Yes. It follows 13.1 durable support identity and precedes 13.3 trust
  classification and Milestone 14 replication closure.
