# Milestone 13.1 Engineering Spec: Durable Subscription-Support Artifacts And Resume Contracts

> **Status:** Closed
>
> **Closeout:** [milestone-13.1-closeout.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-13.1-closeout.md)
>
> **Roadmap parent:** [worth_store_roadmap.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/worth_store_roadmap.md)
>
> **Vision parent:** [worth_store_vision.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/worth_store_vision.md)
>
> **Test requirements:** [test-requirements.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/test-requirements.md)
>
> **Prerequisite milestones:**
> - [milestone-7.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-7.md)
> - [milestone-7-closeout.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-7-closeout.md)
> - [milestone-8.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-8.md)
> - [milestone-10.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-10.md)
> - [milestone-11.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-11.md)
> - [milestone-11-closeout.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-11-closeout.md)
> - [milestone-12.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-12.md)
> - [milestone-12-closeout.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-12-closeout.md)
> - [milestone-13.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-13.md)
> - [milestone-13-closeout.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-13-closeout.md)
>
> **Follow-on milestones:**
> - `Milestone 13.2` (`Subscription Support Through Retention, Compatibility, Replication, And Maintenance`)
> - `Milestone 13.3` (`Subscription Support Accuracy Taxonomy And Certification`)
> - `Milestone 14` (`Replication, Capsules, And Integrity Verification`)
> - `Milestone 15` (`Extensible Durable Artifact Families And Storage Strategies`)
>
> **Primary architectural driver:** make subscription-support artifacts first-class,
> durable, family-aware, basis-linked support records so store restart and rebuild
> can report exact resumability posture without absorbing subscription semantics,
> delivery policy, or host-local session memory.

## Goal

Make first-class subscription-support artifacts durable and basis-exact so the
store can preserve what an admitted subscription family needs to resume
honestly, without turning subscription semantics, delivery fanout, or network
policy into store-owned authority.

## Why This Milestone Exists

Milestone 13.1 is not "persist subscription state."

Milestone 7 already made durable cursor and subscriber-checkpoint truth real.
Milestone 8 already defined stable-basis continuation and exact versus degraded
continuation posture. Milestone 13 already made placement and recall cost-only.

Those surfaces are necessary, but they are not yet enough for first-class
subscription support.

A subscription family may need support artifacts that are more specific than a
raw cursor and more durable than host-local delivery memory:

- the declared subscription family or strategy kind it was lowered through
- the exact stable basis and cursor/checkpoint support truth it depends on
- the support artifact identity and digest that prove what can be resumed
- the distinction between exact resume, degraded resume, rebuild-required, and
  non-resumable posture
- drift diagnostics that say whether failure came from basis drift, cursor
  drift, subscription-support drift, compatibility drift, or lost session memory

Without this milestone, later retention, replication, compatibility, extension,
and certification work would be forced to infer subscription resumability from
cursor folklore, missing records, or server delivery state. That would leak
subscription meaning into the store by accident while still failing to give the
store enough typed evidence to report resumability honestly.

This milestone therefore inserts a narrow support-artifact layer between the
existing live-query substrate and the later platform programs that must retain,
replicate, classify, or certify subscription-support families.

## Hard Part

The hard part is keeping five concepts separate:

- durable cursor/checkpoint truth from Milestone 7
- stable-basis continuation truth from Milestone 8
- subscription-support artifacts that are exact only for a declared support role
- runtime, bridge, and query-layer subscription semantics above the store
- session-local delivery, fanout, and pacing state that must remain ephemeral

The design fails if:

- a subscription-support artifact is treated as a universal resume token across
  different subscription families
- a cursor checkpoint alone can masquerade as exact subscription resume
- store code embeds bridge lowering rules or query delivery policy in order to
  decide resumability
- restart can recover canonical truth but cannot distinguish exact resume from
  degraded or rebuild-required support posture
- subscription-support drift, basis drift, cursor drift, and session loss
  collapse into one vague "cannot resume" error
- rebuilt or missing support records silently fall back to best-effort cursor
  resume while reporting exact resumability

Milestone 13.1 therefore must define one family-aware support identity model,
one basis-linkage model, one resume-classification model, and one restart/rebuild
story that remains subordinate to the runtime stack.

## Explicit Assumptions

- `worth-relational` owns committed truth, schema semantics, lineage semantics,
  and transaction meaning.
- `worth-signal`, the runtime bridge, and later query/server layers own
  subscription meaning, lowering, delivery policy, and lifecycle semantics.
- Milestone 7 cursor and checkpoint artifacts remain durable support truth, but
  cursor truth alone is not subscription-support truth.
- Milestone 8 stable-basis and continuation artifacts remain the durable
  read/continuation substrate consumed here.
- Milestone 10 retention and rebuild rules remain the authority for survival
  legality, but this milestone must publish vocabulary later retention can use.
- Milestone 11 maintenance scheduling is closed and can host future rebuild or
  refresh work, but Milestone 13.1 only defines the subscription-support meaning
  and first restart/rebuild classification.
- Milestone 12 compatibility is closed and supplies artifact-family manifest and
  version-window vocabulary, but Milestone 13.1 only needs first-ship
  compatibility binding for its own support records; broader retention,
  replication, and rolling-compatibility participation is Milestone 13.2.
- Milestone 13 placement and recall are cost-only and must not change whether a
  support artifact is exact, degraded, rebuild-required, or non-resumable.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects is stating the hostile condition before
  designing the feature. Milestone 13.1 therefore starts from cursor folklore
  and host-local session drift, not from a convenient "subscription resume"
  helper.
- `arch_laws.md`
  The most important thing it protects here is authority separation and
  proof-bearing phase progression. Subscription support, cursor truth, stable
  basis truth, runtime subscription semantics, and delivery memory must be
  distinct types and phases.
- `perf_laws.md`
  The most important thing it protects is cost and breadth visibility.
  Subscription-support fetch and resume classification must expose direct
  lookup, support rows read, drift checks, and denial counters instead of
  hiding broad searches behind cheap-looking resume APIs.
- `domain_laws.md`
  The most important thing it protects is decomposition by reason-to-change.
  Family catalog, basis linkage, artifact persistence, resume classification,
  drift diagnostics, restart reconstruction, and certification evidence must be
  separate subdomains rather than one subscription metadata module.
- `worth_store_vision.md`
  The most important thing it protects is that store owns durable survival, not
  runtime semantics. Milestone 13.1 persists subscription-support artifacts
  faithfully while refusing to become a subscription manager.
- `worth_store_roadmap.md`
  The most important thing it protects is sequencing. Milestone 13.1 belongs
  after Milestone 13 because placement, recall, compatibility, maintenance, and
  basis vocabulary are stable enough to name subscription-support durability
  explicitly before the 13.2 and 13.3 cleanup arc.
- `test-requirements.md`
  The most important thing it protects is certification-grade proof. Milestone
  13.1 is not closeable until `Durable Subscription-Support Artifact And
  Resume-Contract Test` proves restart/rebuild parity and typed resumability
  classification.
- `milestone-7.md` and `milestone-7-closeout.md`
  The most important thing they protect is durable cursor, checkpoint, schema,
  and lineage support truth. Milestone 13.1 must consume cursor and checkpoint
  support truth without pretending it is enough to prove subscription resume.
- `milestone-8.md`
  The most important thing it protects is stable-basis continuation equivalence.
  Milestone 13.1 must link subscription support to stable basis and continuation
  posture without redefining live-query semantics.
- `milestone-10.md`
  The most important thing it protects is retained truth and rebuild honesty.
  Milestone 13.1 must publish support-family survival vocabulary for later
  retention work but must not decide retention policy here.
- `milestone-11.md` and `milestone-11-closeout.md`
  The most important thing they protect is one typed maintenance runtime.
  Milestone 13.1 may define rebuild-required posture, but future rebuild work
  must enter that scheduler instead of inventing a subscription worker loop.
- `milestone-12.md` and `milestone-12-closeout.md`
  The most important thing they protect is manifest-backed compatibility
  admission. Subscription-support records introduced here must have family
  manifests and compatibility binding rather than relying on decode success.
- `milestone-13.md` and `milestone-13-closeout.md`
  The most important thing they protect is placement non-authority. Tier
  movement may change support-artifact access cost, but not resumability
  meaning.
- `worth_store_dependency_map.md`
  The most important thing it protects is unlock shape. Milestone 13 now
  enables this cleanup arc, but the dependency map should later be amended to
  include Milestones 13.1 through 13.3 as explicit post-13 gates before final
  replication/certification assumptions depend on subscription support.

## Adversarial Constraint

Milestone 13.1 must survive this hostile condition:

> After restart, rebuild, tier recall, or handoff between runtime instances, a
> store containing durable cursor/checkpoint support truth, stable-basis
> continuation artifacts, and first-class subscription-support records must
> preserve and report the same subscription-support identity, family binding,
> basis linkage, compatibility posture, and resumability conclusion for each
> admitted support lane as a control lane, without collapsing that lane into raw
> cursor resume, host-local delivery memory, or store-owned subscription
> semantics.

## Product Decision Lock

- subscription-support artifacts are durable support artifacts for a declared
  role, not canonical truth and not delivery sessions
- every support artifact is keyed to one admitted subscription family or kind
- cursor truth alone is insufficient to prove exact subscription resume
- stable-basis linkage is mandatory for any exact resume claim
- resumability classification is explicit and typed:
  - `ExactResume`
  - `DegradedButRecoverable`
  - `RebuildRequired`
  - `NotResumable`
- support drift, cursor drift, basis drift, compatibility drift, and delivery
  session loss are distinct diagnostic families
- rebuilt support artifacts may report exact resume only if their declared
  family rules and basis evidence prove exactness
- store may persist and classify support artifacts, but may not lower queries,
  choose delivery policy, fan out subscribers, or manage network delivery
- later Milestones 13.2 and 13.3 must inherit these identity and classification
  surfaces rather than introducing a second subscription-support model

Normative consequence:

- any implementation that treats a cursor checkpoint as exact subscription
  resume without a subscription-support artifact is out of spec
- any implementation that accepts one support artifact across incompatible
  subscription families is out of spec
- any implementation that stores server delivery memory as durable subscription
  support is out of spec
- any implementation that reports generic resume failure without localizing
  support, basis, cursor, compatibility, or session drift is out of spec
- any implementation that lets missing support records silently degrade into
  best-effort resume while reporting exactness is out of spec

## Scope

### In Scope

- first-class durable subscription-support artifact families
- family and kind catalog for admitted first-ship subscription-support lanes
- support artifact identity, digest, basis linkage, cursor/checkpoint linkage,
  and compatibility binding
- explicit resumability classification surfaces
- typed restart and rebuild-required classification for support records
- drift diagnostics that distinguish support, basis, cursor, compatibility, and
  session-local loss
- public facade surfaces for storing, fetching, and classifying
  subscription-support artifacts
- exact counters, compile-time boundaries, and machine-checkable certification
  output for the Milestone 13.1 named suite

### Explicitly Out Of Scope

- query semantics, filter evaluation, subscription lowering, and dependency DAG
  interpretation
- subscriber fanout, delivery pacing, network delivery, retry windows, or server
  connection lifecycle
- retention, compaction, reclaim, compatibility migration, replication, or
  capsule inclusion behavior for subscription-support artifacts beyond the
  minimal first-ship binding required to persist and reopen them honestly
- extension-defined subscription-support families
- final subscription-support accuracy taxonomy and generic/domain certification
  coverage, which are Milestone 13.3 work

## Subscription-Support Authority Model

### Support-Role Non-Authority Rule

A subscription-support artifact is not truth authority and is not subscription
authority.

It is a durable support record whose authority is scoped to one question:

> For this declared subscription-support family, against this exact basis and
> cursor/checkpoint support context, what resumability posture can the store
> prove after restart or rebuild?

It may prove:

- support identity
- family/kind binding
- basis and cursor/checkpoint linkage
- compatibility binding
- digest and reconstruction evidence
- resume classification

It may not prove:

- what a query means
- how a dependency graph is evaluated
- which subscriber receives a delivery
- what network class or delivery schedule applies
- whether upper-layer subscription semantics are satisfied beyond the declared
  support role

Required classification:

- canonical commit records remain `Authoritative`
- Milestone 7 schema, lineage, cursor, and checkpoint records remain
  authoritative support families for their declared role
- Milestone 8 stable-basis and continuation records remain authoritative
  support families for their declared role
- Milestone 13.1 subscription-support records are durable support artifacts
  subordinate to cursor, basis, and runtime subscription semantics
- delivery sessions, connection ids, in-memory fanout queues, and transient
  retry windows remain `Ephemeral`

If a support artifact cannot state its declared support role without naming a
server delivery mechanism, it is not a store artifact.

### First-Ship Family Catalog Rule

Milestone 13.1 must ship a conservative first-ship catalog so "subscription
support" does not become a vague universal bucket.

Required first-ship catalog fields:

- `SubscriptionSupportFamilyId`
- `SubscriptionSupportKind`
- `SubscriptionSupportRole`
- `SubscriptionSupportFamilyVersion`
- `RequiredBasisPosture`
- `RequiredCursorPosture`
- `CompatibilityBinding`
- `ResumeClassifier`
- `SupportArtifactDigestBasis`

Minimum admitted first-ship family kinds:

- `BasisBoundContinuationSupport`
  Support for resuming a declared stable-basis continuation lane when the
  subscription family requires no additional derived resume material beyond the
  Milestone 8 basis and Milestone 7 cursor/checkpoint support truth.
- `MaterializedNarrowingSupport`
  Support for resuming a declared narrowed subscription family that depends on a
  durable, basis-linked narrowing descriptor or support materialization.
- `DegradedContinuationSupport`
  Support for a family that can resume with explicit degradation and must report
  that degradation rather than claiming exactness.

Explicit first-ship debt:

- bridge-specific subscription families that require richer dependency strategy
  proofs may remain absent
- server delivery bundles remain absent
- cross-process fanout session restoration remains absent
- extension-defined subscription-support families remain absent until
  Milestone 15

Rules:

- every admitted family has one explicit role and one resume classifier
- unsupported family kinds fail typed instead of falling back to cursor-only
  resume
- adding a new family must require catalog, manifest, persistence,
  classification, counter, and certification coverage before it can publish

### External Support Declaration Rule

Store must not infer subscription family meaning from query text, bridge
strategy internals, delivery configuration, or runtime callback state.

The upstream owner of subscription meaning must hand store one already-lowered
declaration envelope. Store validates, persists, links, and classifies that
envelope; it does not lower it.

Required declaration surface:

- `SubscriptionSupportDeclaration`
- `SubscriptionSupportDeclarationDigest`
- `SubscriptionSupportDeclarationVersion`
- `DeclaredSubscriptionFamily`
- `DeclaredSupportRole`
- `DeclaredResumeClassifier`
- `DeclaredSupportScope`
- `OpaqueSupportPayloadDigest`
- `UpstreamSupportAuthority`

Required declaration fields:

- family id and kind
- support role
- family version
- stable-basis id expected by the declaration
- cursor or checkpoint identity expected by the declaration
- declared support scope in canonical order
- opaque payload digest
- compatibility manifest id
- upstream authority tag identifying the owner that lowered the declaration
- declaration digest over all semantic inputs

Rules:

- store may compare declaration fields, digests, compatibility posture, basis
  linkage, and cursor/checkpoint linkage
- store may not inspect query predicates, dependency graph internals, subscriber
  fanout topology, delivery windows, or bridge strategy internals to infer
  support family meaning
- store may persist opaque support payload bytes only behind the declaration
  digest and family catalog entry
- a declaration whose upstream authority is unknown, whose family is not in the
  catalog, or whose declared scope is not canonical fails before artifact
  publication
- a declaration cannot be admitted by "same subscriber id" or "same cursor id"
  alone; family, role, basis, cursor/checkpoint, and declaration digest all
  participate

Compile-time consequence:

- `SubscriptionSupportDeclaration` constructors are crate-owned or upstream
  facade-owned; store-internal code consumes `AdmittedSubscriptionSupportDeclaration`
  after catalog and compatibility admission
- no store module outside the declaration admission path may construct a
  declaration digest or declaration admission witness

This is the anti-"store quietly becomes query lowering" rule.

### Durable Record Shape Rule

The first ship must name the durable record shape and access structures. "Persist
support artifact" is too weak to implement honestly.

Minimum durable record families:

- `SubscriptionSupportFamilyRecord`
  Declares admitted family id, kind, role, version, compatibility binding,
  declared upstream authority, and first-ship debt posture.
- `SubscriptionSupportArtifactRecord`
  Stores support artifact id, family id, kind, declaration digest, support
  artifact digest, basis id, cursor/checkpoint identity, support scope digest,
  compatibility manifest id, publication sequence, and artifact state.
- `SubscriptionSupportLinkageRecord`
  Stores stable-basis linkage, authority basis digest, support context digest,
  schema boundary id, cursor frontier or checkpoint sequence, and linkage
  verification digest.
- `SubscriptionSupportClassificationRecord`
  Stores the latest classification conclusion for an artifact identity, the
  classifier version, inspected linkage digest, failure precedence winner, and
  suppressed lower-precedence drift causes.
- `SubscriptionSupportRestartRecord`
  Stores restart reconstruction disposition, recovered manifest sequence,
  recovered artifact count, and rebuild-basis planning status.

Minimum access structures:

- family catalog lookup by `SubscriptionSupportFamilyId`
- artifact lookup by `(SubscriptionSupportFamilyId, SubscriptionSupportArtifactId)`
- declaration lookup by `SubscriptionSupportDeclarationDigest`
- basis lookup by `(StableBasisId, SubscriptionSupportFamilyId)`
- cursor lookup by `(DurableCursorIdentity, SubscriptionSupportFamilyId)`
- checkpoint lookup by `(SubscriberCheckpointIdentity, SubscriptionSupportFamilyId)`
- classification lookup by `(SubscriptionSupportArtifactId, ClassifierVersion)`
- restart recovery lookup by manifest sequence and unresolved disposition

Rules:

- a backend may choose physical tables or encoded records, but it must provide
  these traversal directions honestly
- if a backend cannot provide one access structure without a scan, that path is
  `Debt` and the certification bundle must name the missing structure
- startup reconstruction may load manifests and unresolved restart records; it
  may not walk delivery sessions, subscriber populations, or all cursor records
  to rediscover support artifacts
- support artifact state is one of:
  - `Published`
  - `ClassificationCurrent`
  - `ClassificationStale`
  - `RebuildBasisPlanned`
  - `Rejected`

### Publication Pipeline Rule

Publishing subscription support must be a proof-widening pipeline, not one
helper that accepts raw ids.

Required phase types:

- `RawSubscriptionSupportDeclaration`
- `CatalogAdmittedSubscriptionSupportDeclaration`
- `CompatibilityAdmittedSubscriptionSupportDeclaration`
- `BasisLinkedSubscriptionSupportDeclaration`
- `CursorLinkedSubscriptionSupportDeclaration`
- `PublishableSubscriptionSupportArtifact`
- `PublishedSubscriptionSupportArtifact`
- `ClassifiedSubscriptionSupportArtifact`

Required pipeline:

1. catalog admission proves the family/kind/role is admitted
2. compatibility admission proves the declaration and family version may be read
   and published by the current store
3. basis linkage proves the stable basis id, authority digest, schema boundary,
   and support-context digest match durable Milestone 8 basis truth
4. cursor/checkpoint linkage proves the declared cursor or checkpoint belongs to
   the admitted basis and has not regressed
5. artifact construction binds declaration digest, support payload digest,
   linkage digest, and family catalog record
6. publication writes artifact, linkage, and initial classification evidence in
   one durable publication unit
7. classification consumes only the published proof type

Rules:

- no phase may accept a weaker type than the previous phase produced
- publication is not allowed as asynchronous backfill after cursor advancement
  if the support artifact claims to prove exact resume for that cursor frontier
- if a support artifact is produced after the cursor/checkpoint it references,
  exact resume requires a publication-order witness proving the cursor frontier
  was still current and equivalent at the support publication boundary
- duplicate publication with the same declaration digest and linkage digest must
  converge to the same artifact id, not create a second support record
- duplicate publication with the same artifact id but different digest fails as
  `SubscriptionSupportDigestMismatch`

Compile-time consequence:

- public publish APIs accept only the earliest raw/declaration request type
  appropriate for the facade; internal persistence accepts only
  `PublishableSubscriptionSupportArtifact`
- backend persistence modules cannot synthesize `PublishedSubscriptionSupportArtifact`
  from decoded rows without restart reconstruction proving the durable record
  chain

### Subscription-Support Identity Rule

Every durable subscription-support artifact must have deterministic identity
derived from its support meaning.

Required identity inputs:

- subscription-support family id
- support kind
- stable basis id
- basis support context digest
- cursor or subscriber-checkpoint identity
- cursor frontier or checkpoint sequence
- declared subscription support scope
- family version and compatibility binding
- support artifact semantic digest

Rules:

- equivalent lanes must produce identical support artifact identities and
  digests
- identity may not include host-local process ids, connection ids, queue
  addresses, or transient delivery handles
- two support artifacts with the same cursor but different family/kind binding
  must not collide
- two support artifacts with the same family but different basis support context
  must not collide
- retry, restart, and rebuild must not mint a second durable identity for the
  same admitted support fact

### Basis And Cursor Linkage Rule

Exact subscription resume requires both stable-basis evidence and durable
cursor/checkpoint evidence.

Required linkage fields:

- `StableBasisId`
- `AuthorityBasisDigest`
- `BasisSupportContextDigest`
- `BasisSchemaBoundaryId`
- `DurableCursorIdentity`
- `SubscriberCheckpointIdentity`
- `ContinuationFrontier`
- `SupportCompatibilityWindow`

Rules:

- a support artifact with cursor evidence but no basis linkage cannot classify
  as `ExactResume`
- a support artifact with basis evidence but no cursor/checkpoint linkage cannot
  classify as `ExactResume`
- basis drift and cursor drift are separate failure families
- schema-boundary mismatch inherited from the basis support context must not be
  reported as generic subscription drift
- compatibility binding must be checked before support meaning is exposed after
  decode

### Resume Classification Rule

Milestone 13.1 must make resume posture a typed decision, not a boolean.

Required classification variants:

- `ExactResume`
  The support artifact, basis, cursor/checkpoint, compatibility binding, and
  family classifier all prove the declared support family can resume exactly.
- `DegradedButRecoverable`
  The support family cannot resume on its exact lane, but retained authority and
  support evidence allow a declared degraded path without pretending exactness.
- `RebuildRequired`
  The support artifact is missing, stale, or non-materialized but can be rebuilt
  from retained authority and support truth later through Milestone 11
  maintenance.
- `NotResumable`
  The basis, cursor/checkpoint, family version, support digest, or retained
  inputs no longer admit resume.

Rules:

- classification happens before any resume handle is exposed
- degraded and rebuild-required results may not be acknowledged through an exact
  resume path
- `RebuildRequired` names the family and rebuild basis; it does not schedule
  work directly in this milestone unless an already-existing scheduler entry
  point is deliberately consumed
- `NotResumable` must carry typed failure context, not only a diagnostic string

### Classification Precedence Rule

Multi-failure resume attempts must produce deterministic classification and
diagnostics. The implementation may retain suppressed causes, but it may not
choose whichever error it discovers first.

Required precedence, highest to lowest:

1. `SubscriptionSupportFamilyMismatch`
2. `SubscriptionSupportCompatibilityDrift`
3. `SubscriptionSupportBasisDrift`
4. `SubscriptionSupportSchemaDrift`
5. `SubscriptionSupportCursorDrift`
6. `SubscriptionSupportCheckpointDrift`
7. `SubscriptionSupportDigestMismatch`
8. `SubscriptionSupportPlacementUnavailable`
9. `SubscriptionSupportSessionMemoryMissing`

Rules:

- the highest-precedence failing condition determines the primary
  classification failure
- lower-precedence failures are retained as suppressed drift causes in the
  classification record and diagnostics bundle
- `SessionMemoryMissing` never upgrades an otherwise exact durable support lane
  to `NotResumable`; it affects session restoration posture only
- `PlacementUnavailable` may produce `RebuildRequired` or deferred recall only
  if retained basis and family rules still admit the support artifact
- basis drift prevents cursor evidence from being consumed as exact, even when
  the cursor digest itself still matches
- compatibility drift prevents support payload interpretation, so support digest
  drift inside the payload cannot outrank it

Certification consequence:

- the named suite must include at least one lane with simultaneous basis,
  cursor, and support digest drift and must prove the primary failure is basis
  drift while cursor/support digest drift remain suppressed causes
- the named suite must include simultaneous compatibility and digest drift and
  prove compatibility drift wins

### Drift Localization Rule

The store must distinguish why a support artifact cannot prove its claimed
resumability.

Required drift families:

- `SubscriptionSupportDrift`
- `SubscriptionSupportDigestMismatch`
- `SubscriptionSupportFamilyMismatch`
- `SubscriptionSupportCompatibilityDrift`
- `SubscriptionSupportBasisDrift`
- `SubscriptionSupportCursorDrift`
- `SubscriptionSupportCheckpointDrift`
- `SubscriptionSupportSchemaDrift`
- `SubscriptionSupportPlacementUnavailable`
- `SubscriptionSupportSessionMemoryMissing`

Rules:

- basis drift outranks cursor drift when the cursor belongs to the wrong basis
- cursor drift outranks support artifact drift when the support artifact points
  at a missing or regressed cursor/checkpoint
- support digest drift localizes to the support artifact after basis and cursor
  linkage are valid
- session memory loss is never a durable truth failure; it is a delivery-layer
  loss that may prevent session restoration but may not invalidate durable
  support truth by itself
- placement unavailability may change access cost or produce explicit recall
  posture, but it does not change support resumability classification unless
  the support artifact cannot be legally recalled or rebuilt

### Restart And Rebuild Rule

Restart and rebuild must reconstruct subscription-support conclusions from
durable support records, not from host memory or backend-local residue.

Required restart surfaces:

- `SubscriptionSupportRestartSummary`
- `RecoveredSubscriptionSupportRecord`
- `SubscriptionSupportRebuildBasis`
- `SubscriptionSupportRecoveryDisposition`
- `SubscriptionSupportResumeReport`

Rules:

- restart loads support manifests and records through compatibility-admitted
  paths
- restart verifies basis and cursor/checkpoint linkage before classifying resume
- missing support artifacts are classified as `RebuildRequired` or
  `NotResumable`, not clean success
- corrupted support records fail typed and localize to support family and basis
- rebuild classification may name Milestone 11 scheduler work classes, but it
  does not run unbounded rebuild work inside restart

### Rebuild-Basis Honesty Rule

`RebuildRequired` is only legal when the store can name a retained rebuild basis
and the family catalog says rebuilding that support role is admitted.

Required rebuild-basis fields:

- subscription-support family id
- support artifact id or declaration digest
- stable basis id
- retained authority basis digest
- required cursor/checkpoint identity
- required support scope digest
- required compatibility window
- rebuild work class expected by Milestone 11
- missing or stale durable record families

Rules:

- if retained authority, stable basis, cursor/checkpoint, or family catalog
  evidence is missing, the outcome is `NotResumable`, not `RebuildRequired`
- if the family catalog marks the family as not rebuildable in Milestone 13.1,
  missing support is `NotResumable`
- if rebuild depends on retention, replication, or maintenance participation not
  yet in scope, the report must name explicit `Milestone13_2Required` debt
  rather than imply first-ship rebuild execution exists
- rebuild-basis planning may produce a scheduler descriptor preview, but it may
  not enqueue or execute work unless it enters the Milestone 11 admission
  boundary through a typed work descriptor

This prevents the naive trap where every missing support record becomes a vague
rebuild promise with no retained inputs.

### Resume Handle Boundary Rule

Classification reports and executable resume handles are distinct.

Required handle types:

- `ExactSubscriptionResumeHandle`
- `DegradedSubscriptionResumeHandle`
- `SubscriptionSupportRebuildPlanHandle`
- `SubscriptionResumeDeniedReport`

Rules:

- only `ExactResume` may produce `ExactSubscriptionResumeHandle`
- `DegradedButRecoverable` may produce only a degraded handle that carries the
  degradation class and requires explicit caller acknowledgment
- `RebuildRequired` may produce only a rebuild-plan handle, not a resume handle
- `NotResumable` produces only a denied report
- no public API may return one generic "resume token" whose internals decide
  later whether it is exact, degraded, rebuild, or denied

Compile-time consequence:

- exact-only acknowledgment paths accept only `ExactSubscriptionResumeHandle`
- degraded acknowledgment paths accept only `DegradedSubscriptionResumeHandle`
  and must expose the degradation class
- rebuild work submission accepts only `SubscriptionSupportRebuildPlanHandle`
  and still requires Milestone 11 scheduler admission

### Public Surface Rule

The public facade should expose support artifacts as store support records, not
subscription managers.

Representative surface:

```rust
pub struct SubscriptionSupportArtifact { ... }
pub struct SubscriptionSupportPublishRequest { ... }
pub struct SubscriptionSupportFetchRequest { ... }
pub struct SubscriptionSupportResumeRequest { ... }
pub struct SubscriptionSupportResumeReport { ... }

impl WorthStore {
    pub fn publish_subscription_support(
        &mut self,
        request: SubscriptionSupportPublishRequest,
    ) -> Result<SubscriptionSupportArtifact, SubscriptionSupportError>;

    pub fn fetch_subscription_support(
        &self,
        request: SubscriptionSupportFetchRequest,
    ) -> Result<SubscriptionSupportArtifact, SubscriptionSupportError>;

    pub fn classify_subscription_resume(
        &self,
        request: SubscriptionSupportResumeRequest,
    ) -> Result<SubscriptionSupportResumeReport, SubscriptionSupportError>;
}
```

Surface rules:

- publish consumes a family catalog entry, stable-basis proof, cursor/checkpoint
  proof, and compatibility binding
- fetch consumes family and identity keys, not a vague subscriber id
- classify returns typed resume posture and drift localization
- no facade method may accept server connection ids, network delivery classes,
  or raw host session handles as durable support proof

## Representative First-Ship Flow

The first implementation should prove one concrete lane before broadening the
catalog.

### Flow: Basis-Bound Continuation Support

Scenario:

- Milestone 8 publishes a `StableBasisHandle` for branch `design-main` at
  frontier `C42`.
- Milestone 7 persists durable cursor identity `cursor:panel-a` and checkpoint
  sequence `7` at the same continuation frontier.
- The upstream subscription owner lowers a subscription into
  `BasisBoundContinuationSupport` and hands store a
  `SubscriptionSupportDeclaration`.
- Store publishes a durable subscription-support artifact and later restarts.
- After restart, store classifies the lane as `ExactResume` without reading any
  host-local delivery memory.

Required implementation steps:

1. catalog admits `BasisBoundContinuationSupport`
2. compatibility admits declaration version and family version
3. basis linkage verifies `StableBasisId`, `AuthorityBasisDigest`,
   `BasisSupportContextDigest`, and `BasisSchemaBoundaryId`
4. cursor linkage verifies `DurableCursorIdentity`, checkpoint sequence, and
   continuation frontier against the admitted basis
5. artifact identity is derived from family, basis, cursor/checkpoint,
   declaration digest, support scope digest, compatibility binding, and payload
   digest
6. artifact, linkage, classification, and restart records publish in one durable
   publication unit
7. restart reconstructs `PublishedSubscriptionSupportArtifact`
8. classification produces `ExactSubscriptionResumeHandle`

Expected exact counters for the simple exact lane:

- `subscription_support_declaration_admission_count = 1`
- `subscription_support_declaration_reject_count = 0`
- `subscription_support_family_catalog_lookup_count = 1`
- `subscription_support_publish_count = 1`
- `subscription_support_basis_link_check_count = 1`
- `subscription_support_cursor_link_check_count = 1`
- `subscription_support_checkpoint_link_check_count = 1`
- `subscription_support_compatibility_check_count = 1`
- `subscription_support_classification_precedence_eval_count = 1`
- `subscription_support_exact_resume_count = 1`
- `subscription_support_degraded_resume_count = 0`
- `subscription_support_rebuild_required_count = 0`
- `subscription_support_not_resumable_count = 0`
- `subscription_support_cursor_only_resume_reject_count = 0`
- `subscription_support_suppressed_drift_cause_count = 0`
- `subscription_support_truth_parity_failure_count = 0`

Forbidden shortcuts in this flow:

- accepting subscriber id plus cursor id as enough to publish support
- constructing artifact identity before scope canonicalization
- storing the delivery session id in the artifact identity
- reconstructing the support artifact after restart by scanning all cursors for
  `cursor:panel-a`
- returning a generic resume token instead of `ExactSubscriptionResumeHandle`
- reporting exact resume when the support artifact is missing but cursor truth
  still exists

## Required Contracts And Counters

### Compile-Time Boundary Rule

The highest-risk support boundaries must be compiler-enforced.

Required proof-bearing surfaces:

- `AdmittedSubscriptionSupportDeclaration`
- `AdmittedSubscriptionSupportFamily`
- `SubscriptionSupportBasisWitness`
- `SubscriptionSupportCursorWitness`
- `SubscriptionSupportCheckpointWitness`
- `SubscriptionSupportCompatibilityWitness`
- `PublishableSubscriptionSupportArtifact`
- `PublishedSubscriptionSupportArtifact`
- `ExactSubscriptionResumeWitness`
- `DegradedSubscriptionResumeWitness`
- `SubscriptionSupportRebuildWitness`

Required compile-time posture:

- raw cursor ids may not construct exact subscription support
- raw stable basis ids may not construct exact subscription support
- exact resume witnesses may not be caller-synthesized
- degraded or rebuild-required reports may not flow through exact-only
  acknowledgment paths
- delivery session memory may not satisfy durable support witness requirements
- decoded backend rows may not become published support artifacts without
  restart reconstruction witnesses
- a raw declaration digest may not stand in for catalog and compatibility
  admission

Required proof surface:

- compile-fail tests for backend decoded rows used as published artifacts
- compile-fail tests for raw declaration digests used as admitted declarations
- compile-fail tests for cursor-only exact subscription resume
- compile-fail tests for synthetic exact resume witness construction
- compile-fail tests for cross-family support artifact reuse
- compile-fail tests for delivery-session handles used as durable support
- compile-fail tests for degraded/rebuild-required reports passed to exact-only
  resume acknowledgment surfaces

### Performance-Shaping Types Rule

Milestone 13.1 must encode the dominant cost and locality facts in typed
surfaces.

Required performance-shaping surfaces:

- `SubscriptionSupportLookupKey`
- `SubscriptionSupportFamilyScope`
- `SubscriptionSupportAccessStructureReport`
- `SubscriptionSupportBasisSummary`
- `SubscriptionSupportCursorSummary`
- `SubscriptionSupportCheckpointSummary`
- `SubscriptionSupportClassificationPlan`
- `SubscriptionSupportPrecedenceReport`
- `SubscriptionSupportDriftSummary`
- `SubscriptionSupportPayloadBudget`
- `SubscriptionSupportAllocationScope`
- `SubscriptionSupportRestartShard`
- `SubscriptionSupportResultCostSurface`
- `SubscriptionSupportCounterContract`

Rules:

- support fetch starts from family plus support identity, not from scanning all
  subscribers
- resume classification consumes pre-resolved basis and cursor summaries rather
  than rediscovering support context from arbitrary history
- family/kind matching is decided before artifact payload interpretation
- result envelopes report support rows read, basis rows read, cursor rows read,
  compatibility checks, and drift checks

### Pre-Resolved Classification Plan Rule

Classification must consume one lowered plan. It may not rediscover strategy,
access path, or payload policy while deciding the resume result.

Required plan types:

- `ExactResumeClassificationPlan`
- `DegradedResumeClassificationPlan`
- `RebuildRequiredClassificationPlan`
- `DeniedResumeClassificationPlan`
- `PlacementDeferredClassificationPlan`

Required pre-resolved fields:

- support lookup path
- basis lookup path
- cursor or checkpoint lookup path
- compatibility posture
- payload admission posture
- placement/recall posture
- rebuild-basis posture
- failure precedence table version
- result handle family allowed by the plan

Rules:

- planning resolves which classification family may execute before payload
  interpretation starts
- execution consumes the plan and may not switch from exact to degraded, rebuild,
  or denied by mutating a result flag; a different outcome requires a distinct
  plan type
- placement-deferred plans are not exact or degraded resume plans; they are cost
  admission plans that must later produce one of the semantic plan families
- each plan carries the access structures it is allowed to touch

Compile-time consequence:

- `classify_subscription_resume` internally lowers to one plan enum whose
  variants carry distinct proof types; exact handle construction is impossible
  from non-exact variants
- payload interpretation functions accept only compatibility-admitted,
  payload-budget-admitted plan types

### Payload Budget And Allocation Rule

Opaque support payloads must still have explicit cost shape.

Required budget fields:

- `MaxSupportPayloadBytes`
- `MaxSupportScopeItems`
- `MaxSupportLinkageRows`
- `MaxSuppressedDriftCauses`
- `MaxRestartShardRecords`

Required allocation scopes:

- `PublishLocalSupportArena`
- `ClassificationLocalSupportArena`
- `RestartShardSupportArena`

Rules:

- publication rejects or degrades before allocating a payload larger than the
  admitted family budget
- classification allocates from a classification-local scope that is released
  after the result envelope is built
- restart reconstruction processes bounded shards rather than materializing all
  support records into one unbounded vector
- suppressed drift causes are bounded and counted; overflow produces a typed
  diagnostics truncation flag without changing the primary failure
- support scope canonicalization must report item count and canonicalization
  work before artifact identity construction

This prevents a small-looking support record from becoming an unbounded memory
or allocation path.

### Directional Storage And Shard Rule

Access structures must reflect the dominant traversal directions and restart
shape.

Required directional stores:

- family-to-artifacts store
- artifact-id-to-linkage store
- basis-to-support-artifacts store
- cursor-to-support-artifacts store
- checkpoint-to-support-artifacts store
- declaration-digest-to-artifact store
- restart-manifest-sequence-to-shard store

Rules:

- classification may touch at most one artifact identity lane plus its declared
  basis and cursor/checkpoint lanes unless a typed broadened diagnostic plan is
  produced
- restart reconstruction processes `SubscriptionSupportRestartShard` units with
  declared maximum records and emits shard-local counters
- a missing reverse index is not silently compensated by scanning a forward
  store; it marks the affected complexity path as `Debt`
- access-structure verification runs at backend open and participates in the
  Milestone 13.1 evidence bundle

### Result Cost Surface Rule

Every public result must carry the structural cost of how it was produced.

Required result fields:

- resolved classification plan family
- support rows read
- basis rows read
- cursor rows read
- checkpoint rows read
- compatibility checks executed
- payload bytes inspected
- support scope items canonicalized
- placement recall units touched
- restart shard records inspected where applicable
- suppressed drift causes retained
- complexity status for the path

Rules:

- diagnostics may add detail, but these fields live in the public result
  envelope or its machine-checkable evidence, not only logs
- exact and degraded handles carry their cost surface forward so acknowledgment
  or follow-on work can see whether the lane stayed inside budget
- zero-work reuse, direct resident read, cold recall, rebuild-plan creation, and
  denied classification are distinct cost postures

### Density Regime Rule

First ship is sparse and identity-first. Dense subscription-support workloads
must be explicit debt until the store has a batch classification path.

Required density classes:

- `SparseIdentityClassification`
- `FamilyBatchClassificationDebt`
- `RestartShardBatchClassification`

Rules:

- single-lane classification is verified only for sparse identity lookup
- classifying many support artifacts for one family in one call is not admitted
  unless it uses a batch plan with family-local amortized lookups
- any temporary bulk audit helper must report `FamilyBatchClassificationDebt`
  rather than pretending scalar loops are the batch architecture

### Complexity Contracts

Minimum named paths:

- `subscription_support_publish`
- `subscription_support_fetch`
- `subscription_support_plan_lowering`
- `subscription_resume_classification`
- `subscription_support_restart_reconstruction`
- `subscription_support_rebuild_basis_planning`
- `subscription_support_payload_admission`
- `subscription_support_scope_canonicalization`

Minimum contracts:

- subscription-support publish cost is proportional to:
  - one admitted family entry
  - one stable-basis proof
  - one cursor/checkpoint proof
  - support artifact bytes published for that family
  - not total subscriber count
- subscription-support fetch cost is proportional to:
  - one support lookup key
  - support rows for that exact identity
  - compatibility checks for the support family
  - not total support artifact volume
- subscription-support plan lowering cost is proportional to:
  - one support artifact identity lane
  - one basis summary
  - one cursor or checkpoint summary
  - one compatibility posture
  - not payload bytes or branch history
- resume classification cost is proportional to:
  - one support artifact
  - one stable-basis linkage check
  - one cursor/checkpoint linkage check
  - one family classifier execution
  - not total branch history or total active subscriptions
- restart reconstruction cost is proportional to:
  - restart shards admitted
  - records inside each shard
  - unresolved rebuild/disposition records
  - not host session count or delivery queue breadth
- rebuild-basis planning cost is proportional to:
  - support families requiring rebuild
  - retained basis and cursor/checkpoint proofs needed for those families
  - not all retained history
- payload admission cost is proportional to:
  - payload bytes declared in the admitted support payload
  - support scope items declared in the support scope
  - not decoded query predicates or delivery session state
- scope canonicalization cost is proportional to:
  - declared support scope items
  - family-specific canonical ordering comparisons
  - not subscriber population or branch history

Forbidden hidden work:

- scanning all durable cursors to guess which one matches a subscription
- replaying arbitrary branch history to infer support family identity
- reading host-local delivery queues during durable support classification
- falling back from missing support records to cursor-only exact resume
- treating tier recall as ordinary fetch without reporting placement cost
- decoding opaque payload bytes before payload budget admission
- scalar-looping over many support artifacts while reporting a batch path
- constructing unbounded suppressed-cause vectors during multi-drift reporting

Minimum counters:

- `subscription_support_declaration_admission_count`
- `subscription_support_declaration_reject_count`
- `subscription_support_publish_count`
- `subscription_support_family_catalog_lookup_count`
- `subscription_support_identity_collision_count`
- `subscription_support_fetch_count`
- `subscription_support_lookup_key_count`
- `subscription_support_rows_read`
- `subscription_support_plan_lowering_count`
- `subscription_support_basis_link_check_count`
- `subscription_support_cursor_link_check_count`
- `subscription_support_checkpoint_link_check_count`
- `subscription_support_compatibility_check_count`
- `subscription_support_payload_budget_check_count`
- `subscription_support_payload_bytes_admitted`
- `subscription_support_payload_budget_reject_count`
- `subscription_support_scope_item_count`
- `subscription_support_scope_canonicalization_count`
- `subscription_support_allocation_scope_count`
- `subscription_support_classification_precedence_eval_count`
- `subscription_support_suppressed_drift_cause_count`
- `subscription_support_suppressed_drift_truncation_count`
- `subscription_support_exact_resume_count`
- `subscription_support_degraded_resume_count`
- `subscription_support_rebuild_required_count`
- `subscription_support_not_resumable_count`
- `subscription_support_rebuild_basis_missing_count`
- `subscription_support_cursor_only_resume_reject_count`
- `subscription_support_family_mismatch_count`
- `subscription_support_compatibility_drift_count`
- `subscription_support_basis_drift_count`
- `subscription_support_cursor_drift_count`
- `subscription_support_checkpoint_drift_count`
- `subscription_support_schema_drift_count`
- `subscription_support_digest_mismatch_count`
- `subscription_support_session_memory_loss_count`
- `subscription_support_restart_recovered_count`
- `subscription_support_restart_shard_count`
- `subscription_support_restart_shard_record_count`
- `subscription_support_restart_global_scan_count`
- `subscription_support_rebuild_basis_count`
- `subscription_support_tier_recall_count`
- `subscription_support_batch_classification_debt_count`
- `subscription_support_truth_parity_failure_count`

Required counter assertions:

- `subscription_support_cursor_only_resume_reject_count` increments in hostile
  cursor-only resume lanes
- `subscription_support_truth_parity_failure_count` remains zero in equivalent
  restart, rebuild, and recall lanes
- `subscription_support_exact_resume_count` increments only when basis,
  cursor/checkpoint, compatibility, and family support all match
- `subscription_support_degraded_resume_count`,
  `subscription_support_rebuild_required_count`, and
  `subscription_support_not_resumable_count` remain distinct and are never
  collapsed into generic failure totals
- `subscription_support_session_memory_loss_count` may increment without
  changing support truth digests
- `subscription_support_tier_recall_count` changes diagnostics and counters,
  not truth or resume classification, for admitted recall lanes
- `subscription_support_suppressed_drift_cause_count` increments in multi-drift
  hostile lanes and is zero in single-cause lanes
- `subscription_support_rebuild_basis_missing_count` increments when missing
  retained inputs correctly turn an apparent rebuild lane into `NotResumable`
- declaration admission and rejection counts exactly match catalog-compatible
  and catalog-incompatible declaration lanes in certification
- `subscription_support_plan_lowering_count` increments before classification
  execution in every classified lane
- `subscription_support_payload_budget_reject_count` increments before payload
  decoding in oversized-payload hostile lanes
- `subscription_support_restart_global_scan_count` remains zero in admitted
  restart lanes
- `subscription_support_restart_shard_record_count` equals the records in
  admitted restart shards, not total support history
- `subscription_support_batch_classification_debt_count` increments for any
  scalar-loop batch helper that has not shipped a family-batch plan
- `subscription_support_suppressed_drift_truncation_count` increments only when
  suppressed causes exceed the declared diagnostics budget

## Required Internal Subsystems

Milestone 13.1 should decompose by responsibility:

- `subscription_support/catalog/`
  admitted family/kind declarations, support roles, manifest identity, and
  first-ship debt markers
- `subscription_support/identity/`
  deterministic support artifact identity, digest basis, canonical ordering, and
  collision rejection
- `subscription_support/linkage/`
  stable-basis, cursor, checkpoint, schema, and compatibility witnesses
- `subscription_support/artifacts/`
  durable support records, publication units, fetch records, and backend
  persistence adapters
- `subscription_support/classification/`
  exact, degraded, rebuild-required, and non-resumable classification plans
- `subscription_support/drift/`
  support, basis, cursor, checkpoint, schema, compatibility, placement, and
  session-loss drift localization
- `subscription_support/restart/`
  restart reconstruction, rebuild-basis planning, and recovery dispositions
- `subscription_support/evidence/`
  counters, complexity surfaces, certification bundles, and suite reports

Rules:

- backend persistence may store bytes and indexes, but it may not decide support
  semantics locally
- classification code consumes proof-bearing basis and cursor summaries rather
  than raw rows
- drift diagnostics are emitted from the subscription-support subsystem, not
  reconstructed only inside tests

## Phases

### Phase 1: Lock Family Catalog, Identity, And Non-Authority Boundaries

Phase 1 defines what subscription support is allowed to mean before any durable
record ships.

Required work:

- define the subscription-support family catalog and first-ship family kinds
- define upstream `SubscriptionSupportDeclaration` envelope fields and opaque
  payload digest rules
- define support role, support kind, family version, and compatibility binding
  vocabulary
- define deterministic support artifact identity and digest basis
- define minimum durable record families and access structures
- define basis, cursor, checkpoint, schema, and compatibility linkage witnesses
- define the proof-widening publication pipeline from raw declaration to
  classified support artifact
- define non-authority rules separating support artifacts from runtime
  subscription semantics and delivery sessions
- define resume classification variants and drift localization families
- define deterministic classification precedence and suppressed-cause reporting
- define exact, degraded, rebuild-plan, and denied resume handle families
- define compile-time witness privacy for exact, degraded, and rebuild-required
  support flows
- define counter contracts and certification bundle shape
- define pre-resolved classification plan families, payload budgets,
  allocation scopes, restart shards, result cost surfaces, and density classes

Exit condition:

- subscription-support artifacts have one exact durable role
- cursor/checkpoint truth and stable-basis truth are required inputs, not
  substitutes for subscription support
- unsupported family kinds fail typed rather than becoming universal resume
  tokens

### Phase 2: Persist Durable Subscription-Support Artifacts

Phase 2 makes subscription-support records durable, fetchable, and restart
visible.

Required work:

- implement family catalog admission for first-ship support kinds
- implement declaration admission from upstream-lowered support envelopes
- implement support scope canonicalization and payload budget admission before
  artifact identity construction
- implement support artifact publication from admitted family, basis, cursor or
  checkpoint, and compatibility witnesses
- persist support identity, digest basis, family/kind binding, stable-basis
  linkage, cursor/checkpoint linkage, and compatibility binding
- persist family records, artifact records, linkage records, initial
  classification records, and restart records
- verify backend access structures at open and mark missing structures as
  explicit `Debt`
- implement allocation scopes for publish-local and classification-local support
  work
- implement direct support fetch by family and support identity
- implement manifest-backed compatibility admission before semantic exposure
- implement deterministic duplicate/retry handling for equivalent publication
  lanes
- expose typed family mismatch, identity collision, malformed support, missing
  manifest, and compatibility drift failures
- emit exact publication, fetch, family lookup, row read, and collision counters

Exit condition:

- support artifacts survive restart as first-class durable records
- fetch and publish never depend on host-local session state
- equivalent retry and reopen lanes preserve support artifact identity and digest

### Phase 3: Classify Resume Posture From Basis And Cursor Evidence

Phase 3 turns support records into typed resumability conclusions.

Required work:

- implement resume classification planning from one support artifact, one stable
  basis, and one cursor/checkpoint support context
- lower every resume attempt into a pre-resolved classification plan before
  payload interpretation
- implement `ExactResume`, `DegradedButRecoverable`, `RebuildRequired`, and
  `NotResumable` classification results
- implement exact, degraded, rebuild-plan, and denied handle construction from
  classification reports
- implement deterministic failure precedence with suppressed drift causes
- reject cursor-only exact resume attempts
- reject cross-family support artifact reuse
- distinguish basis drift, cursor drift, checkpoint drift, schema drift,
  compatibility drift, support digest drift, and session-memory loss
- preserve placement/tier recall as cost posture rather than resume meaning
  where recall is legally admitted
- expose exact classification result envelopes with cost counters and drift
  diagnostics
- carry `SubscriptionSupportResultCostSurface` into exact/degraded/rebuild/denied
  results

Exit condition:

- every resume attempt produces one typed resumability conclusion
- degraded, rebuild-required, and non-resumable outcomes cannot masquerade as
  exact resume
- session loss and durable support drift remain separate phenomena

### Phase 4: Reconstruct Support Posture Across Restart, Rebuild, And Handoff

Phase 4 makes the support model operationally honest across process boundaries.

Required work:

- implement restart reconstruction of subscription-support records from durable
  manifests and persisted support rows
- process restart reconstruction through bounded `SubscriptionSupportRestartShard`
  units rather than one global support scan
- implement rebuild-basis planning for missing or stale support artifacts
- reject `RebuildRequired` when retained authority, basis, cursor/checkpoint, or
  catalog evidence is missing
- report `Milestone13_2Required` debt where future participation rules are
  required before rebuild can be operationally executed
- classify missing support records as `RebuildRequired` or `NotResumable`
  according to retained basis and family rules
- preserve family-aware support posture during handoff between runtime instances
  without persisting delivery-session memory
- emit restart summaries, recovery dispositions, and support resume reports
- map corrupted, missing, or incompatible records into typed drift and recovery
  failures
- expose exact restart, rebuild-basis, and handoff counters
- emit restart shard counters and prove global restart scans remain zero

Exit condition:

- restart can explain subscription-support resumability without reading host
  memory
- rebuild-required posture names its family and basis without scheduling hidden
  unbounded work
- handoff changes runtime ownership only, not durable support meaning

### Phase 5: Prove Durable Subscription-Support Artifact And Resume Contracts

Phase 5 turns subscription-support durability into certifiable store behavior.

Required work:

- run the Milestone 13.1 named suite:
  `Durable Subscription-Support Artifact And Resume-Contract Test`
- compare control, restart, rebuild-required, handoff, and tier-recall lanes
  against equivalent support-truth digests
- include exact, degraded, rebuild-required, and non-resumable resume lanes
- include hostile cursor-only exact resume attempts
- include hostile cross-family reuse attempts
- include basis drift, cursor drift, support digest drift, compatibility drift,
  and session-memory-loss lanes
- include multi-drift precedence lanes proving primary and suppressed causes
- include rebuild-basis-missing lanes proving apparent rebuild work becomes
  `NotResumable`
- include declaration rejection lanes for unknown upstream authority,
  non-canonical support scope, and unsupported family kind
- include backend access-structure debt lanes for a missing support lookup index
- include oversized-payload rejection before decode
- include restart-shard lanes proving no global scan is used
- include scalar-loop batch debt lanes for family-wide classification helpers
- include result-cost-surface assertions for exact, degraded, rebuild, denied,
  and placement-deferred outcomes
- include compile-fail coverage for synthetic exact witnesses, cursor-only
  exact support, cross-family support reuse, raw declaration-digest admission,
  decoded-row publication, and delivery-session handles used as durable support
  proof
- emit machine-checkable truth, artifact, subscription-support, replay,
  diagnostics, and counter bundles

Exit condition:

- subscription-support identity survives restart and rebuild classification
- cursor truth alone cannot masquerade as exact subscription resume
- basis drift and subscription-support drift are mechanically distinct
- multi-failure precedence is deterministic and audited
- implementation has concrete durable records and access structures to build
- performance strategy is carried by plan types and public cost surfaces
- payload, allocation, restart, and density limits are explicit
- Milestone 13.1 closeout evidence exists in machine-checkable form

## Must Ship

- conservative first-ship subscription-support family catalog
- upstream subscription-support declaration envelope with opaque payload digest
- minimum durable record families and required access structures
- durable subscription-support artifact records with deterministic identity
- proof-widening publication pipeline from raw declaration to classified support
  artifact
- explicit family/kind tagging for admitted support families
- support artifact digest basis and compatibility binding
- stable-basis, cursor, checkpoint, schema, and compatibility linkage witnesses
- pre-resolved classification plan families
- payload budgets, allocation scopes, restart shards, density classes, and result
  cost surfaces
- direct support artifact publication and fetch surfaces
- typed resume classification:
  - `ExactResume`
  - `DegradedButRecoverable`
  - `RebuildRequired`
  - `NotResumable`
- drift taxonomy that distinguishes support, basis, cursor, checkpoint, schema,
  compatibility, placement, and session-memory loss
- deterministic classification precedence with suppressed-cause reporting
- exact, degraded, rebuild-plan, and denied resume handle families
- restart reconstruction and rebuild-basis planning for support artifacts
- handoff-safe support posture that excludes host-local delivery memory
- compile-fail boundary coverage for witness privacy and cursor-only misuse
- exact support publication, fetch, classification, drift, restart, and rebuild
  counters
- exact payload, plan-lowering, allocation, restart-shard, and batch-debt
  counters
- machine-checkable Milestone 13.1 certification output

## Must Preserve

- canonical commit history remains the only semantic durable truth authority
- Milestone 7 cursor and checkpoint support truth remains necessary but not
  sufficient for exact subscription resume
- Milestone 8 stable-basis continuation remains the read/continuation substrate
  and is not redefined here
- runtime bridge, `worth-signal`, `worth-query`, and later server layers remain
  owners of subscription semantics, lowering, lifecycle, delivery, and fanout
- session-local delivery state remains ephemeral
- placement and recall change access cost only
- later Milestones 13.2 and 13.3 inherit the support identity and resume
  classification model rather than inventing another one

## Acceptance Evidence

Milestone 13.1 is complete only when the store satisfies the named Milestone
13.1 suite:

- `Durable Subscription-Support Artifact And Resume-Contract Test`

Required machine-checkable outputs:

- `truth_digest`
- `artifact_digest`
- `subscription_support_digest`
- `replay_digest`
- `diagnostics_digest`
- `counter_snapshot`

Minimum certification matrix rows:

- `exact_resume_control`
  publishes support against an admitted family, basis, and cursor/checkpoint
  context and classifies as exact
- `restart_exact_resume`
  reopens from durable records and produces the same support identity,
  subscription-support digest, and exact classification
- `rebuild_required_missing_support`
  removes or withholds a rebuildable support record and reports
  `RebuildRequired` with its family and basis
- `degraded_but_recoverable`
  preserves retained authority and cursor/basis truth but reports a declared
  degraded path rather than exact resume
- `not_resumable_basis_drift`
  changes the basis support context and localizes failure to basis drift
- `not_resumable_cursor_drift`
  regresses or mismatches cursor/checkpoint evidence and localizes failure to
  cursor or checkpoint drift
- `support_digest_drift`
  corrupts the support artifact after basis and cursor linkage remain valid and
  localizes failure to support digest drift
- `compatibility_drift`
  decodes a support artifact whose compatibility window is not admitted and
  rejects before semantic exposure
- `cursor_only_exact_resume_rejected`
  proves durable cursor/checkpoint truth alone cannot construct exact
  subscription resume
- `cross_family_reuse_rejected`
  proves a support artifact for one admitted family cannot resume another family
- `session_memory_loss_non_authoritative`
  loses host-local delivery memory and reports session loss without changing
  durable support truth
- `tier_recall_cost_only`
  recalls a support artifact from colder placement and changes diagnostics or
  counters without changing support truth or resume classification
- `runtime_handoff_equivalence`
  hands support posture between runtime instances without persisting delivery
  sessions or changing support identity
- `unknown_upstream_authority_rejected`
  submits a declaration from an unadmitted upstream authority and rejects before
  support artifact construction
- `non_canonical_scope_rejected`
  submits a declaration whose support scope is not canonicalized and rejects
  before identity calculation
- `multi_drift_basis_precedence`
  injects basis, cursor, and support digest drift together and proves basis
  drift is primary while cursor and digest drift are suppressed causes
- `multi_drift_compatibility_precedence`
  injects compatibility and support digest drift together and proves
  compatibility drift is primary while digest drift is suppressed
- `rebuild_basis_missing_not_resumable`
  removes retained rebuild inputs and proves a missing support artifact reports
  `NotResumable` instead of vague `RebuildRequired`
- `backend_access_structure_debt`
  opens a backend missing the exact support lookup structure and proves the
  path reports `Debt` rather than silently scanning all support records
- `decoded_row_publication_rejected`
  proves backend-decoded rows cannot be treated as published support artifacts
  without restart reconstruction witnesses
- `oversized_payload_rejected_before_decode`
  submits an opaque support payload over the admitted family budget and proves
  rejection occurs before payload interpretation or classification execution
- `restart_shard_bounded_reconstruction`
  reconstructs support records from bounded restart shards and proves
  `subscription_support_restart_global_scan_count` remains zero
- `result_cost_surface_exact`
  proves an exact resume result carries plan family, rows read, payload bytes,
  scope items, compatibility checks, and complexity status
- `batch_classification_debt`
  exercises a family-wide scalar-loop classification helper and proves it
  reports batch-classification debt rather than verified batch support

Milestone-specific proof obligations:

- support artifact identity is deterministic across publish, retry, restart, and
  rebuild-classification lanes
- upstream declarations are admitted only through catalog, compatibility, basis,
  cursor/checkpoint, and canonical-scope checks
- every admitted support artifact is bound to one family/kind and one declared
  support role
- exact resume requires support artifact, stable basis, cursor/checkpoint, and
  compatibility evidence
- cursor-only resume attempts fail typed
- basis drift, cursor drift, support drift, compatibility drift, and session
  loss remain mechanically distinguishable
- multi-drift classification follows the explicit precedence order and retains
  suppressed causes
- `RebuildRequired` appears only when retained rebuild basis and family
  rebuildability are proven
- payload budget admission happens before payload decode or interpretation
- restart reconstruction is shard-bounded and does not scan global support
  history
- exact, degraded, rebuild, denied, and placement-deferred results carry public
  result cost surfaces
- family-wide classification is either a real batch plan or explicit debt
- delivery-session memory never becomes durable support proof
- rebuild-required posture names the family and basis required to rebuild
- tier placement and recall affect cost evidence only
- compile-fail tests prevent synthetic exact witnesses, raw declaration digest
  admission, decoded-row publication, cursor-only exact resume, and
  cross-boundary misuse

Milestone 13.1 is not closed by "a subscriber resumed once" or "a cursor loaded
after restart" tests.

## Architectural Notes

- The smart abstraction is not a durable subscriber. The smart abstraction is a
  family-aware support artifact whose exactness is tied to basis and cursor
  evidence.
- Keep the word "subscription" narrow in store code. If a type starts needing
  delivery policy, network pacing, or query semantics, it belongs above the
  store.
- `ExactResume` should be rare and proof-heavy. Degraded and rebuild-required
  are honest outcomes, not failures to hide.
- Milestone 13.1 should reuse Milestone 12 compatibility manifests and Milestone
  13 placement vocabulary, but it should not absorb Milestone 13.2 retention,
  replication, compatibility-propagation, or maintenance participation.
- The first-ship catalog can be conservative. The dangerous thing is not narrow
  family coverage; the dangerous thing is pretending one cursor-shaped support
  record works for every subscription family.

## Sequencing Notes

This milestone belongs immediately after Milestone 13 because the store now has
the durability, basis, maintenance, compatibility, and placement vocabulary
needed to make subscription-support artifacts explicit without turning them into
shadow authority.

- Milestone 7 and Milestone 8 provide the cursor/checkpoint and stable-basis
  support truth this milestone consumes.
- Milestone 10 and Milestone 11 provide the retention/rebuild and maintenance
  vocabulary that later Milestone 13.2 will thread these support families
  through.
- Milestone 12 provides compatibility manifests and admitted semantic exposure
  rules.
- Milestone 13 provides placement and recall vocabulary so support artifact
  location remains cost-only.
- Milestone 13.2 should follow by proving these support artifacts survive,
  degrade, rebuild, or reject through retention, compatibility, replication,
  and maintenance programs.
- Milestone 13.3 should follow by assigning the final accuracy/trust posture and
  certification coverage required for generic and domain store certification.
- Milestone 14 should not treat subscription-support replication as complete
  until 13.1 through 13.3 have made the support families durable, operationally
  propagated, and certifiably classified.
