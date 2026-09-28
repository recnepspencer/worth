# Milestone 13.3 Engineering Spec: Subscription Support Accuracy Taxonomy And Certification

> **Status:** Completed. Closeout: [milestone-13.3-closeout.md](./milestone-13.3-closeout.md).
>
> **Roadmap parent:** `worth_store_roadmap.md`
>
> **Vision parent:** [worth_store_vision.md](./worth_store_vision.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Prerequisite milestone:**
> - [milestone-13.1.md](./milestone-13.1.md)
> - [milestone-13.1-closeout.md](./milestone-13.1-closeout.md)
> - [milestone-13.2.md](./milestone-13.2.md)
>
> **Follow-on milestone:**
> - `Roadmap 2 S.0` (`Shipped Store Reconciliation And Capability Reclassification`)
> - `Milestone 14` (`Replication, Capsules, And Integrity Verification`) after the Roadmap 2 gate
>
> **Primary architectural driver:** make subscription-support trust posture
> explicit, enforceable, and certifiable for each declared support role without
> letting exact support artifacts become truth authority or subscription
> authority.

## Goal

Classify and certify first-class subscription-support artifacts so every
admitted support family carries an enforced, family-aware trust posture for its
declared support role.

The milestone is complete only when exact, degraded, rebuilt, replicated,
stale, and policy-rejected subscription-support variants can be audited from
machine-checkable evidence, and no consumer can use a weaker support posture as
though it proved exact resumability.

## Why This Milestone Exists

Milestone 13.1 made subscription-support artifacts durable, basis-linked,
family-aware, and restart-visible.

Milestone 13.2 threads those artifacts through retention, compatibility,
replication, import/export, and maintenance programs so operational actions
publish typed support consequences instead of leaving resumability to be
guessed from missing rows.

Milestone 13.3 closes the cleanup arc by assigning final trust posture to the
results of those earlier milestones.

Without this milestone:

- `ExactResume` can become a local classification rather than a certifiable
  platform claim
- degraded or rebuilt support can be operationally valid but trust-ambiguous
- replicated support can preserve identity while losing proof strength
- certification can prove subscription-support rows exist without proving what
  their support role is allowed to mean
- later replication, extensibility, derived-artifact accuracy, budget, and
  operator-repair work can consume subscription-support artifacts without a
  shared trust vocabulary

This milestone exists to make support posture a typed, evidence-backed contract:
what the artifact proves, for which family, against which basis, after which
operational actions, with which accuracy/trust class, and under which
certification coverage.

## Hard Part

The hard part is not adding another enum named `Exact`.

The hard part is preventing five adjacent facts from collapsing into one vague
resume signal:

- a Milestone 13.1 resume classification such as `ExactResume`
- a Milestone 13.2 operational verdict such as `ExactResumePreserved`
- the artifact family's declared support role
- the accuracy or trust class that says how strong the support evidence is
- the certification row proving that class under hostile lanes

The design fails if:

- any rebuilt, replicated, migrated, stale, or partially retained support
  artifact can be consumed as exact without an accuracy witness
- a support artifact exact for one family or role is treated as exact for
  another family or role
- certification counts row coverage without checking family, role, basis,
  compatibility, operational verdict, and trust posture together
- "degraded but recoverable" support is allowed to look like "exact but slower"
- trust posture is inferred from logs, artifact presence, or same-run
  comparison instead of proof-bearing classification inputs
- generic and domain certification omit subscription-support lanes while later
  roadmap milestones assume subscription durability is platform-grade

Milestone 13.3 therefore defines the final support trust taxonomy, the legal
translation from earlier resume and operational surfaces into that taxonomy,
and the certification evidence required before downstream systems may consume
subscription-support posture as a platform fact.

## Explicit Assumptions

- `worth-relational` owns canonical truth, schema semantics, lineage semantics,
  and transaction meaning.
- `worth-signal`, `worth-runtime-bridge`, `worth-query`, and later server
  layers own subscription semantics, dependency evaluation, lowering, delivery,
  fanout, and lifecycle.
- Milestone 13.1 owns durable support identity, family catalog, basis/cursor
  linkage, restart reconstruction, and initial resume classification.
- Milestone 13.2 owns retention, compatibility, portability, import, and
  maintenance consequences for support artifacts.
- Milestone 13 owns placement and recall cost posture; tiering never changes
  support trust posture.
- Milestone 14 will own final replication/capsule integrity for all artifacts,
  but it must consume the support trust posture defined here.
- Roadmap 2 follows this milestone before platform-grade post-13.3 work can
  honestly proceed; 13.3 classifies semantic subscription-support trust, not
  physical database readiness.
- Unsupported subscription-support family variants may remain absent. Shipped
  support families may not remain unclassified, uncertified, or trust-ambiguous.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects is adversarial, hard-problem-first
  design. Milestone 13.3 therefore starts from stale and overclaimed resume
  proof rather than from a convenient certification report.
- `arch_laws.md`
  The most important thing it protects is proof-bearing authority separation.
  Resume classification, operational survival, support trust, and certification
  coverage must be distinct typed phases, and support exactness may never become
  truth or subscription authority.
- `perf_laws.md`
  The most important thing it protects is visible cost and bounded proof.
  Support trust classification, drift detection, and certification coverage must
  expose exact counters for classified artifacts, stale detections, rebuild
  completions, coverage rows, and forbidden broad scans.
- `domain_laws.md`
  The most important thing it protects is decomposition by responsibility.
  Trust taxonomy, role binding, certification coverage, drift audit, generic
  certification, and domain certification must remain separate subdomains
  rather than one broad subscription-certification module.
- `worth_store_vision.md`
  The most important thing it protects is that store makes runtime truth and
  support artifacts survive without becoming the runtime. This milestone
  classifies durable support evidence while refusing to own query or delivery
  semantics.
- `worth_store_roadmap.md`
  The most important thing it protects is sequencing. Milestone 13.3 closes the
  13.x subscription-support arc and must land before Roadmap 2 and later
  platform claims consume subscription-support durability as complete.
- `test-requirements.md`
  The most important thing it protects is certification-grade evidence.
  Milestone 13.3 is not closeable until `Subscription-Support Accuracy And
  Certification Test` proves explicit classification, stale/rebuilt downgrade,
  and generic/domain certification coverage.
- `test-requirements-2.md`
  The most important thing it protects is realistic physical proof for Roadmap
  2. This milestone must not pretend its semantic support certification also
  proves physical database readiness; that proof is the next gate.
- `milestone-13.md` and `milestone-13-closeout.md`
  The most important thing they protect is placement non-authority. Support
  artifacts may be hot, warm, cold, or recalled, but trust posture is decided by
  support evidence, not residency.
- `milestone-13.1.md` and `milestone-13.1-closeout.md`
  The most important thing they protect is durable family-aware support identity
  and resume classification. Milestone 13.3 must consume those surfaces without
  weakening the exact-resume proof chain.
- `milestone-13.2.md`
  The most important thing it protects is operational participation through
  retention, compatibility, replication, and maintenance. Milestone 13.3 must
  classify those operational verdicts rather than rediscovering facts from raw
  rows or logs.
- `worth_store_dependency_map.md`
  The most important thing it protects is unlock shape. Milestone 13.2 unlocks
  this milestone; this milestone finishes the subscription-support cleanup arc
  before the Roadmap 2 foundation gate and later Milestone 14 replication work.

## Adversarial Constraint

Milestone 13.3 must survive this hostile condition:

> A store containing exact, degraded, rebuilt, replicated, migrated, partially
> retained, stale, and policy-rejected subscription-support artifacts across
> multiple admitted support families must classify, certify, and audit each
> artifact's trust posture for its declared support role without allowing any
> stale, rebuilt, degraded, incompatible, omitted, or role-mismatched artifact
> to be consumed as exact resumability proof.

## Product Decision Lock

- subscription-support trust posture is scoped to a declared family and support
  role; it is not universal subscription truth
- exact support trust requires the full Milestone 13.1 proof chain and the
  Milestone 13.2 operational verdicts that preserve that proof
- degraded, rebuilt, stale, replicated, migrated, omitted, and policy-rejected
  variants must carry trust posture that prevents exact-resume consumption
- accuracy/trust classification is a proof-bearing phase after resume
  classification and operational verdict translation, not a flag on raw rows
- certification coverage must be family-aware, role-aware, basis-aware,
  compatibility-aware, and operational-verdict-aware
- generic and domain certification must include first-class
  subscription-support lanes before store certification may claim support
  durability closure
- Roadmap 2 physical foundation work remains separate; this milestone may emit
  semantic support certification but may not claim platform-grade physical
  database posture
- future extension-defined support families must register into this taxonomy
  rather than inventing their own support exactness vocabulary

Normative consequence:

- any implementation that treats `ExactResumePreserved` as enough to construct
  support trust without family, role, basis, compatibility, and certification
  evidence is out of spec
- any implementation that lets a rebuilt or migrated artifact retain exact
  support trust without an explicit equivalence witness is out of spec
- any implementation that reports certification coverage by row count alone is
  out of spec
- any implementation that consumes logs, artifact presence, or raw capsule rows
  as support-trust proof is out of spec

## Scope

### In Scope

- subscription-support trust taxonomy for admitted support families
- role-scoped accuracy/trust classification for exact, degraded, rebuilt,
  replicated, migrated, stale, omitted, and policy-rejected support variants
- legal translation from Milestone 13.1 resume classifications and Milestone
  13.2 operational verdicts into trust posture
- certification coverage model for family, role, basis, compatibility,
  operational action, drift, rebuild, replication, and domain scenarios
- generic store certification updates for subscription-support trust posture
- domain certification rows for geometry/CAD, web/data, AI, chip/simulation,
  and offline/collaborative support resumability
- compile-time boundaries preventing weaker trust posture from constructing
  exact-resume handles or exact-support certification rows
- exact counters and machine-checkable evidence bundles for the Milestone 13.3
  named suite

### Explicitly Out Of Scope

- defining new subscription semantics, query semantics, delivery policy, fanout,
  retry windows, or network lifecycle behavior
- final replication/capsule integrity closure for all store artifacts, which is
  Milestone 14 after the Roadmap 2 gate
- extension-defined durable support families beyond typed deferral and future
  registration requirements
- physical page, chunk, buffer-pool, I/O, security, and operator-forensics
  database readiness, which belongs to Roadmap 2
- general derived-artifact accuracy taxonomy for all non-subscription derived
  artifact families, which remains Milestone 17

## Required Contracts And Counters

### Support Trust Taxonomy Rule

Milestone 13.3 must define one enforced trust taxonomy for
subscription-support artifacts in their declared support role.

Required surfaces:

- `SubscriptionSupportTrustClass`
- `SupportRoleTrustPosture`
- `SupportTrustClassificationPlan`
- `SupportTrustClassificationReport`
- `SupportTrustClassificationWitness`
- `SupportTrustDowngradeReason`

Required trust classes:

- `ExactSupportTrusted`
- `DegradedSupportTrusted`
- `RebuildDerivedSupport`
- `ReplicatedSupportTrusted`
- `MigratedSupportTrusted`
- `StaleSupportRejected`
- `PolicyRejectedSupport`
- `UnsupportedSupportTrust`

Rules:

- trust posture is a two-axis model, not one overloaded enum:
  - `SupportTrustStrength` answers how strong the support proof is
  - `SupportTrustProvenance` answers how this artifact state was produced
- `SupportTrustStrength` must at least distinguish:
  - `Exact`
  - `Degraded`
  - `RebuildOnly`
  - `Rejected`
  - `Unsupported`
- `SupportTrustProvenance` must at least distinguish:
  - `NativePublished`
  - `Rebuilt`
  - `Migrated`
  - `Replicated`
  - `Imported`
  - `Omitted`
  - `PolicyExpired`
- the combined trust report names family id, support role, basis id,
  cursor/checkpoint id, compatibility window, operational verdict, trust
  strength, and provenance
- `ExactSupportTrusted`, `ReplicatedSupportTrusted`,
  `MigratedSupportTrusted`, and similar public classifications are facade
  report labels derived from the two-axis model; they are not the internal
  authority shape
- `ExactSupportTrusted` is constructible only from exact resume proof plus an
  exact-preserved operational verdict plus any required equivalence evidence
- `RebuildDerivedSupport`, `ReplicatedSupportTrusted`, and
  `MigratedSupportTrusted` may claim exact support only through an explicit
  equivalence witness
- stale or policy-rejected variants may not lower into resumable handles
- role mismatch always rejects, even if the artifact digest and basis match
- `Imported + Exact` and `Replicated + Exact` are legal combinations only when
  target-side admission and portability evidence preserve the declared support
  role; `Imported` or `Replicated` alone is never evidence strength
- `RebuildOnly` is not a degraded resume handle; it is a proof that rebuild work
  may be scheduled or reported, not consumed as resume support

Naive trap this prevents:

- treating support exactness as an artifact property instead of a family-role
  proof.
- creating one enum that hides whether a support artifact is weak because its
  evidence is degraded or because its provenance is transformed.

### Runtime Trust Versus Certification Closure Rule

Production trust classification and certification closure are separate phases.
Runtime code may need to classify a support artifact before a certification
bundle exists, while platform claims require certification coverage before the
result can be advertised as certified.

Required surfaces:

- `OperationalSupportTrustReport`
- `CertifiedSupportTrustReport`
- `SupportTrustCertificationStamp`
- `SupportCertificationCorpusVersion`
- `SupportCertificationCoverageRequirement`
- `UncertifiedSupportTrustPosture`

Rules:

- operational trust classification consumes resume, operational, equivalence,
  compatibility, basis, cursor/checkpoint, and drift evidence
- certification closure consumes an operational trust report plus coverage
  evidence and produces a certified trust report
- an operational trust report may be exact but uncertified; it may be used only
  inside store-local resume classification paths that do not claim platform
  certification
- exported evidence, generic certification, domain certification, and Milestone
  14 handoff must consume `CertifiedSupportTrustReport`, not raw operational
  reports
- certification stamps name corpus version, suite version, family id, support
  role, trust strength, provenance, row id, and evidence bundle digest
- stale or missing certification stamps reject platform trust claims without
  changing the underlying local operational classification

Naive trap this prevents:

- making production exactness depend on a test row id, or the opposite mistake:
  treating a local exact classification as a certified platform claim.

### Resume And Operational Translation Rule

Milestone 13.3 trust classification consumes earlier proof-bearing outputs. It
does not inspect raw support rows.

Required surfaces:

- `SupportResumeTrustInput`
- `SupportOperationalTrustInput`
- `SupportTrustTranslationPlan`
- `ExactSupportTrustWitness`
- `DegradedSupportTrustWitness`
- `RebuildDerivedTrustWitness`
- `RejectedSupportTrustWitness`

Rules:

- Milestone 13.1 resume classifications and Milestone 13.2 operational verdicts
  must translate through a pre-resolved plan before trust classification
- exact trust is illegal unless both resume classification and operational
  verdict preserve exactness
- degraded or rebuild-required translation cannot construct an exact trust
  witness
- missing operational participation evidence downgrades or rejects; it does not
  default to exact support because the support artifact still exists
- translation failures are typed and carry suppressed causes when multiple
  earlier proofs disagree

Naive trap this prevents:

- letting later code bypass the 13.1 and 13.2 proof chain by re-reading durable
  rows and making a fresh "looks exact" decision.

### Trust Classification Typestate Rule

Trust classification must be encoded as a proof-widening pipeline. Each phase
must consume the exact proof type established by the prior phase.

Required typestate surfaces:

- `RawSupportTrustRequest`
- `SupportTrustRequestAdmitted`
- `SupportTrustInputsTranslated`
- `SupportTrustDriftChecked`
- `SupportTrustEquivalenceChecked`
- `OperationalSupportTrustClassified`
- `SupportTrustCoverageChecked`
- `CertifiedSupportTrustClassified`

Required transitions:

1. `admit_support_trust_request`
   consumes raw family, role, support identity, and requested use.
2. `translate_support_trust_inputs`
   consumes Milestone 13.1 resume classification and Milestone 13.2 operational
   verdict inputs.
3. `check_support_trust_drift`
   consumes translated inputs and produces drift-checked support evidence.
4. `check_support_trust_equivalence`
   consumes drift-checked evidence and any rebuild, migration, replication, or
   import provenance requirements.
5. `classify_operational_support_trust`
   consumes equivalence-checked evidence and produces local operational trust.
6. `check_support_trust_coverage`
   consumes operational trust and certification corpus requirements.
7. `certify_support_trust`
   consumes coverage-checked evidence and emits certified trust.

Rules:

- no phase may accept raw support rows, raw capsule rows, raw action envelopes,
  raw compatibility manifests, or raw maintenance descriptors as substitutes for
  its input proof type
- exact support handles consume only `OperationalSupportTrustClassified` when
  the call is store-local, and only `CertifiedSupportTrustClassified` when the
  call crosses store, replication, export, generic-certification, or
  domain-certification boundaries
- each typestate struct has private fields and is constructed only by its
  proving function
- skipping drift or equivalence checks must be a compile-time error, not a
  runtime assertion

Naive trap this prevents:

- writing one large `classify_support_trust(...)` function that internally
  branches over raw data and silently forgets which proofs were actually
  established.

### Concrete Input Receipt Rule

Trust classification must consume concrete, inspectable receipts from earlier
milestones rather than vague "evidence" parameters.

Required receipts:

- `ResumeClassificationReceipt`
- `OperationalVerdictReceipt`
- `SupportFamilyRoleReceipt`
- `SupportBasisReceipt`
- `SupportCursorCheckpointReceipt`
- `SupportCompatibilityReceipt`
- `SupportPortabilityReceipt`
- `SupportMaintenanceReceipt`
- `SupportRetentionReceipt`
- `SupportImportAdmissionReceipt`

Rules:

- every receipt carries an origin milestone, source artifact id, digest,
  version, and proof status
- receipts are stable inputs to digest construction; display text or debug
  strings may not participate in trust identity
- missing receipts produce typed denial before classification, not degraded
  best effort
- receipt reuse inside a batch is explicit through a batch receipt cache and
  exact reuse counters

Naive trap this prevents:

- passing loosely typed structs named "evidence" around until trust depends on
  whatever fields happen to be present.

### Support Trust Equivalence Rule

Rebuilt, migrated, replicated, and imported artifacts require explicit
equivalence proof before they can inherit the trust class of their source.

Required surfaces:

- `SupportTrustEquivalenceContract`
- `SupportRebuildEquivalenceWitness`
- `SupportMigrationEquivalenceWitness`
- `SupportReplicationEquivalenceWitness`
- `SupportImportEquivalenceWitness`
- `SupportEquivalenceFailure`

Rules:

- equivalence basis includes family id, support role, declaration digest,
  artifact digest basis, stable basis, cursor/checkpoint identity,
  compatibility window, operational verdict, and portability scope
- a support artifact may preserve identity but still lose exact trust if its
  equivalence contract is incomplete
- same digest is not enough to prove same trust when role, compatibility window,
  or operational action differs
- equivalence witnesses are sealed and constructed only by the corresponding
  rebuild, migration, replication, or import proving path

Naive trap this prevents:

- assuming that artifact digest equality after replication or migration proves
  exact support trust on the target.

### Drift And Staleness Rule

Support trust classification must localize drift before exposing support
posture.

Required surfaces:

- `SupportTrustDriftScanPlan`
- `SupportTrustDriftReport`
- `SupportStalenessVerdict`
- `SupportTrustSuppressedCause`
- `SupportTrustDriftLocality`

Required drift families:

- family drift
- support role drift
- basis drift
- cursor/checkpoint drift
- support digest drift
- compatibility drift
- operational verdict drift
- portability drift
- certification coverage drift
- placement-cost drift

Rules:

- placement-cost drift may change diagnostics and counters but not trust class
  unless support evidence was actually lost
- certification coverage drift rejects platform trust even when local resume
  classification still succeeds
- multi-drift reports retain primary and suppressed causes in deterministic
  order
- drift detection cost must be bounded by the classification plan's family,
  basis, or certification scope, not by global support history

Naive trap this prevents:

- treating any successful resume classification as enough after compatibility,
  portability, or certification coverage has gone stale.

### Epoch And Freshness Rule

Trust posture is not timeless. It is valid only against the support catalog,
compatibility corpus, operational ledger, and certification corpus versions that
were checked.

Required surfaces:

- `SupportTrustEpoch`
- `SupportCatalogEpoch`
- `SupportOperationalLedgerEpoch`
- `SupportCompatibilityEpoch`
- `SupportCertificationEpoch`
- `SupportTrustFreshnessWitness`
- `SupportTrustExpiredReport`

Rules:

- every operational trust report carries catalog, operational-ledger,
  compatibility, and input-artifact epochs
- every certified trust report additionally carries certification corpus and
  suite-version epochs
- changing support family declarations, resume classifiers, compatibility
  windows, operational verdict rules, or certification coverage requirements
  invalidates earlier trust reports unless an explicit epoch-translation witness
  is produced
- cached trust reports may be reused only when their epochs match the current
  request context
- stale epoch reuse rejects before exact support handle construction

Naive trap this prevents:

- caching an exact support trust result and reusing it after the family catalog,
  compatibility window, or certification suite changed.

### Trust Cache And Reuse Rule

Any trust cache is derived state with an explicit equivalence contract.

Required surfaces:

- `SupportTrustCacheKey`
- `SupportTrustCacheEntry`
- `SupportTrustCacheEquivalence`
- `SupportTrustCacheHitWitness`
- `SupportTrustCacheInvalidationReport`

Rules:

- cache identity includes family id, support role, support identity, basis id,
  cursor/checkpoint id, compatibility epoch, operational ledger epoch,
  provenance, requested trust strength, and certification epoch where certified
  trust is requested
- a cache hit may skip recomputation only by producing
  `SupportTrustCacheHitWitness`
- cache entries are derived and rebuildable; losing them changes cost, not
  trust truth
- cache invalidation breadth is counted and must be proportional to affected
  family, role, basis, or epoch, not global support history

Naive trap this prevents:

- adding a convenient "last trust classification" cache that survives catalog or
  compatibility changes and becomes accidental authority.

### Access Structure And Hot-Path Rule

The required suite may not pass through broad scans or foreground
certification-building work hidden behind resume calls.

Required access structures:

- lookup by support identity
- lookup by family and support role
- lookup by basis id
- lookup by cursor/checkpoint id
- lookup by operational ledger action id
- lookup by compatibility epoch
- lookup by certification row id
- lookup by domain scenario id

Required path classes:

- `ForegroundResumeTrustPath`
- `BatchCertificationPath`
- `DomainCertificationPath`
- `RoadmapHandoffPath`
- `TrustCacheRebuildPath`

Rules:

- foreground resume trust paths may classify local operational trust but may not
  build missing certification matrices, run domain certification, or scan
  global support history
- missing required access structures produce typed `SupportTrustAccessDebt` or
  rejection in required lanes, not fallback scans
- domain certification runs only through batch/domain certification paths with
  explicit workload scenarios and counters
- certification matrix construction is batch work, not a per-resume side effect

Naive trap this prevents:

- making every resume call "helpfully" refresh certification coverage or scan
  all support rows to prove exactness.

### Payload And Evidence Size Rule

Trust classification and certification must reject oversized evidence before
decoding, allocation-heavy materialization, or domain scenario construction.

Required surfaces:

- `SupportTrustEvidenceBudget`
- `SupportCertificationMatrixBudget`
- `DomainScenarioEvidenceBudget`
- `SupportTrustPayloadBudgetRejection`
- `SupportTrustAllocationScope`

Rules:

- budgets apply to support payload receipts, operational receipts, equivalence
  reports, certification rows, domain scenario evidence, and handoff reports
- budget rejection occurs before payload interpretation or certification row
  construction
- allocation scopes are separate for foreground trust classification, batch
  certification, domain certification, and handoff export
- exact allocation counters are required for representative certification lanes

Naive trap this prevents:

- allowing certification proof to become an unbounded blob of convenient
  evidence because "it only runs in tests."

### Layout And Access-Path Discipline Rule

The trust layer must declare its physical/logical access layout before
classification and certification paths consume it.

Required layout families:

- `SupportTrustPrimaryIndex`
  direct lookup by support identity
- `SupportTrustFamilyRoleIndex`
  family and role scoped lookup
- `SupportTrustBasisIndex`
  basis scoped lookup for drift and domain scenarios
- `SupportTrustCursorCheckpointIndex`
  cursor/checkpoint scoped lookup
- `SupportTrustOperationalActionIndex`
  action-envelope and operational-ledger lookup
- `SupportTrustEpochIndex`
  catalog, compatibility, operational-ledger, and certification epoch lookup
- `SupportTrustCertificationRowIndex`
  row id, family, role, scenario, and trust class lookup
- `SupportTrustDomainScenarioIndex`
  domain scenario id and required family coverage lookup

Rules:

- every trust classification plan names the exact access structures it will
  touch before execution
- access paths are declared as point lookup, bounded range, batch lookup, or
  rejected; there is no implicit scan access path in required lanes
- each index declares its rebuild authority, epoch invalidation basis, and read
  amplification counter
- certification row lookup must be keyed by row identity and coverage scope, not
  by filtering all rows after load
- domain certification scenarios consume predeclared scenario indexes rather than
  scanning every domain row for a matching label
- missing indexes reject or report debt before opening the data path

Naive trap this prevents:

- satisfying "bounded" claims by loading a vector of all support rows and
  filtering it in memory because the test corpus is small.

### Batch Cardinality And Density Rule

Trust work must be shaped around honest cardinality. Scalar APIs may exist only
as thin single-item plans over the same batch machinery.

Required batch surfaces:

- `SupportTrustBatchPlan`
- `SupportTrustBatchScope`
- `SupportTrustAffectedSet`
- `SupportTrustDensityClass`
- `SupportTrustBatchReceiptCache`
- `SupportTrustBatchResult`

Required density classes:

- `SingleSupportArtifact`
- `FamilyLocal`
- `RoleLocal`
- `BasisLocal`
- `CursorCheckpointLocal`
- `OperationalActionLocal`
- `CertificationScopeLocal`
- `DomainScenarioLocal`
- `StoreGlobalRejected`

Rules:

- every batch plan declares its density class, affected-set width, expected index
  probes, receipt reuse strategy, and allocation scope before execution
- scalar classification calls lower to `SingleSupportArtifact` batch plans
  rather than running a separate scalar-only path
- family, basis, certification, and domain work must amortize receipt loading
  through `SupportTrustBatchReceiptCache`
- `StoreGlobalRejected` is a real density class for attempted whole-store trust
  operations and must not execute in required certification lanes
- if sparse tracking overhead exceeds declared benefit, the batch plan must
  switch to an explicit dense family-local plan rather than silently scanning
  unrelated support history

Naive trap this prevents:

- implementing trusted single-artifact classification well, then building
  certification by looping over that scalar path and rediscovering the same
  receipts for every row.

### Lowered Performance Plan Rule

Execution must consume lowered performance plans. It may not decide lookup
strategy, batch width, receipt reuse, or allocation behavior while classifying
trust.

Required lowered plans:

- `LoweredSupportTrustClassificationPlan`
- `LoweredSupportTrustDriftPlan`
- `LoweredSupportTrustEquivalencePlan`
- `LoweredSupportTrustCertificationPlan`
- `LoweredSupportTrustDomainPlan`
- `SupportTrustPlanRejection`

Rules:

- planning resolves access structures, density class, required receipts, cache
  eligibility, epoch requirements, allocation budget, and expected counters
  before execution
- execution cannot fall back from point lookup to broad scan
- execution cannot widen from one support family or basis to another without a
  new lowered plan
- rejected plans carry typed causes and expected skipped counters
- result envelopes echo the plan id and observed counter snapshot so planning
  claims can be checked against execution evidence

Naive trap this prevents:

- putting strategy branches inside the classifier where a missing receipt or
  index quietly broadens the work.

### Allocation And Clone Discipline Rule

Foreground trust classification must be allocation-bounded and clone-hostile.

Required surfaces:

- `SupportTrustScratchArena`
- `SupportTrustReceiptSlice`
- `SupportTrustRowSlice`
- `SupportTrustDigestInputSlice`
- `SupportTrustMoveOnlyEvidencePacket`
- `SupportTrustAllocationReport`

Rules:

- foreground operational trust classification uses caller-provided or
  operation-scoped scratch storage
- certification and domain paths may allocate only inside their declared batch
  allocation scope
- support receipts, row evidence, and digest inputs are passed as borrowed
  slices or move-only packets where possible
- cloning evidence packets requires an explicit semantic boundary such as
  control-versus-hostile comparison, export bundle construction, or offline
  verifier handoff
- any clone of support evidence increments a clone-boundary counter with a
  reason code
- per-row heap allocation inside certification matrix construction is debt
  unless the row family explicitly proves it is outside the hot or batch path

Naive trap this prevents:

- building certification rows by allocating and cloning rich evidence structs per
  row until the suite passes but the real platform cannot scale.

### Counter Contract Exactness Rule

Every performance claim must have a named contract with exact expected counters
for representative lanes.

Required contract surfaces:

- `SupportTrustClassificationComplexityContract`
- `SupportTrustBatchComplexityContract`
- `SupportTrustAccessPathContract`
- `SupportTrustAllocationContract`
- `SupportTrustCertificationComplexityContract`
- `SupportTrustDomainComplexityContract`

Minimum contract shapes:

- single operational trust classification:
  proportional to one support identity lookup, required receipt lookups, drift
  checks, and equivalence checks for that artifact
- family-local certification:
  proportional to covered family-role rows plus unique receipt sets, not total
  support history
- basis-local drift detection:
  proportional to support artifacts linked to that basis and relevant epochs,
  not total artifacts
- replicated/imported equivalence:
  proportional to portability scope entries and target admission receipts, not
  source store breadth
- domain certification:
  proportional to declared scenario rows and required first-ship family coverage,
  not all domain certification history

Rules:

- each contract is marked `Verified` or `Debt`
- required 13.3 lanes may not close with `Debt` for access-path, allocation, or
  global-scan avoidance contracts
- exact expected values must be asserted for zero broad scans, zero forbidden
  exact overclaims, index probe counts, receipt reuse counts, allocation counts,
  clone counts, cache hits/misses, and certification row counts

Naive trap this prevents:

- claiming a path is "bounded" without saying bounded by what, or proving only
  that the counter was nonzero.

### Certification Coverage Rule

Certification coverage must prove trust posture, not merely artifact existence.

Required surfaces:

- `SubscriptionSupportCertificationCoveragePlan`
- `SupportCertificationRow`
- `SupportCertificationCoverageMatrix`
- `SupportCertificationCoverageWitness`
- `SupportCertificationGapReport`
- `SupportCertificationSummary`

Rules:

- every admitted family and support role has at least one exact, degraded,
  stale/rejected, and operationally transformed row where applicable
- coverage rows must name their control, hostile, and rebuild/replay lanes
- row evidence includes artifact digest, support digest, diagnostics digest,
  counter snapshot, trust class, primary drift cause, and zero-forbidden counts
- missing rows, duplicate rows, role-mismatched rows, or row-label/evidence
  mismatch reject the bundle
- certification may not compare a row only to itself from the same run

Naive trap this prevents:

- passing certification because rows were emitted, without proving the row
  actually covered the claimed support trust posture.

### First-Ship Family Coverage Rule

Milestone 13.3 must certify the first-ship families that Milestone 13.1
admitted. It may not close with a family-agnostic trust taxonomy.

Required first-ship family coverage:

- `BasisBoundContinuationSupport`
  - exact native publication over a stable retained basis
  - stale basis rejection
  - cursor/checkpoint drift rejection
  - operational exact-preserved to certified-exact transition
- `MaterializedNarrowingSupport`
  - exact native publication over a basis-linked narrowing descriptor
  - replicated full-scope exact equivalence
  - partial replication omission and non-exact target posture
  - compatibility drift downgrade or rejection
- `DegradedContinuationSupport`
  - degraded native publication
  - degraded handle construction
  - degraded-as-exact compile-fail and runtime rejection
  - policy rejection when degradation is not admitted by the family role
- `ExtensionDefinedSupport`
  - typed unsupported-family rejection until Milestone 15 registration exists

Rules:

- every first-ship family row names the exact family kind and support role it
  covers
- a generic "exact support" row cannot satisfy first-ship family coverage
- absent richer bridge/server support families are explicit debt rows with
  `Milestone15Required`, `ServerLayerRequired`, or another concrete future owner
- closing 13.3 with only `BasisBoundContinuationSupport` exact lanes is out of
  spec

Naive trap this prevents:

- certifying the taxonomy on the easiest continuation family while silently
  leaving admitted narrowing and degraded families trust-ambiguous.

### Generic And Domain Certification Rule

Milestone 13.3 must update certification posture so subscription-support trust
is visible in both generic and domain-facing store evidence.

Required surfaces:

- `GenericStoreSubscriptionSupportCertification`
- `DomainSubscriptionSupportCertification`
- `SubscriptionSupportDomainScenario`
- `SupportTrustCertificationExport`
- `SubscriptionSupportCertificationHandoffReport`

Required domain scenario classes:

- geometry/CAD session continuation:
  retained branch-head continuation using `BasisBoundContinuationSupport`, with
  an exact retained basis lane and a stale-basis rejection lane
- web/data durable subscription resume:
  restart plus partial replication using `MaterializedNarrowingSupport`, with
  full-scope exact preservation and omitted-scope non-exact target posture
- AI branch workspace continuation:
  support degradation using `DegradedContinuationSupport`, proving degraded
  trust stays resumable only through the degraded handle family
- chip/simulation long-history continuation:
  rebuild-derived support over a retained analysis basis, proving rebuild
  provenance is visible and exact trust requires equivalence
- offline/collaborative capsule import:
  capsule import with support omission, target-side import admission, and
  explicit non-exact trust for omitted support

Rules:

- generic certification proves the taxonomy and family-role matrix
- domain certification proves the taxonomy maps to real Store product contexts
- domain rows may use conservative first-ship support families; absence of more
  advanced families must be explicit debt, not implied coverage
- certification handoff to Roadmap 2 must distinguish semantic support trust
  from physical database readiness

Naive trap this prevents:

- closing subscription-support certification in a generic harness while domain
  certification can still consume trust-ambiguous support artifacts.

### Certification Bundle Shape Rule

The certification bundle must be structured enough to be checked offline
without reconstructing live store state.

Required bundle records:

- `SupportTrustRunHeader`
- `SupportTrustCoverageMatrixReport`
- `SupportTrustRowEvidence`
- `SupportTrustCounterSnapshot`
- `SupportTrustDriftMatrix`
- `SupportTrustEquivalenceMatrix`
- `GenericSupportCertificationReport`
- `DomainSupportCertificationReport`
- `Roadmap2SupportTrustHandoffReport`

Required row evidence fields:

- row id
- family id
- support role
- trust strength
- provenance
- operational verdict
- resume classification
- basis id
- cursor/checkpoint id
- compatibility epoch
- operational ledger epoch
- certification epoch
- control lane digest
- hostile lane digest
- rebuild or replay lane digest
- artifact digest
- subscription-support digest
- diagnostics digest
- counter digest
- primary drift cause
- suppressed drift causes
- exact-overclaim count
- global-scan debt count

Rules:

- bundle validation recomputes row digests from structured fields, not display
  strings
- unsupported or debt rows are explicit records with debt reason, blocked trust
  strength, and required future milestone
- bundle comparison is stable under map iteration order and platform-specific
  path formatting
- any omitted row must appear in the gap report and block closeout unless the
  family is explicitly out of first-ship scope

Naive trap this prevents:

- emitting a pretty certification summary that humans can read but machines
  cannot independently validate.

### Failure Taxonomy Rule

Trust failures must be typed by cause and recovery posture.

Required failure families:

- `SupportTrustFamilyMismatch`
- `SupportTrustRoleMismatch`
- `SupportTrustBasisMismatch`
- `SupportTrustCursorCheckpointMismatch`
- `SupportTrustCompatibilityMismatch`
- `SupportTrustOperationalVerdictMismatch`
- `SupportTrustPortabilityMismatch`
- `SupportTrustEquivalenceMissing`
- `SupportTrustEpochExpired`
- `SupportTrustCoverageMissing`
- `SupportTrustAccessStructureDebt`
- `SupportTrustPayloadBudgetExceeded`
- `SupportTrustForbiddenExactOverclaim`

Rules:

- every failure carries family id, support role, support identity, requested
  trust strength, provenance, and recoverability posture
- recoverability posture distinguishes:
  - retry with fresher receipts
  - rebuild trust cache
  - rerun certification
  - wait for Milestone 14 or Roadmap 2 evidence
  - unsupported by current family catalog
  - permanently rejected by policy
- failure conversion to public errors preserves typed cause and suppressed
  causes

Naive trap this prevents:

- collapsing every trust problem into "not certified" and leaving callers unable
  to tell whether they should retry, rebuild, rerun certification, or stop.

### Compile-Time Boundary Rule

The highest-risk trust boundaries must be compiler-enforced.

Required proof-bearing surfaces:

- `ExactSupportTrustWitness`
- `SupportTrustClassificationWitness`
- `SupportCertificationCoverageWitness`
- `SupportTrustEquivalenceWitness`
- `CertifiedSubscriptionSupportBundle`

Required compile-time posture:

- exact trust witnesses cannot be publicly synthesized
- degraded, rebuild-required, not-resumable, stale, or policy-rejected trust
  cannot construct exact resume handles
- raw support rows, raw capsule rows, raw maintenance descriptors, and raw
  artifact digests cannot construct certification rows
- certification bundles cannot be marked complete without coverage witnesses
- role-scoped trust witnesses cannot be reused across support families or roles

Required proof surface:

- compile-fail tests for synthetic exact trust witness construction
- compile-fail tests for degraded or rebuild-derived trust used as exact resume
- compile-fail tests for raw row certification
- compile-fail tests for cross-family trust witness reuse
- compile-fail tests for incomplete certification bundle construction

### Complexity And Counter Rule

Milestone 13.3 evidence must publish path-local complexity status rather than a
single rolled-up certification verdict.

Minimum named paths:

- `support_trust_classification`
- `support_trust_translation`
- `support_trust_equivalence`
- `support_trust_drift_detection`
- `support_certification_coverage`
- `generic_support_certification`
- `domain_support_certification`

Minimum counters:

- `subscription_support_exact_trust_count`
- `subscription_support_degraded_trust_count`
- `subscription_support_rebuild_derived_trust_count`
- `subscription_support_replicated_trust_count`
- `subscription_support_migrated_trust_count`
- `subscription_support_stale_rejection_count`
- `subscription_support_policy_rejection_count`
- `subscription_support_role_mismatch_rejection_count`
- `subscription_support_trust_drift_detection_count`
- `subscription_support_equivalence_witness_count`
- `subscription_support_equivalence_failure_count`
- `subscription_support_epoch_expiration_count`
- `subscription_support_certification_stamp_count`
- `subscription_support_operational_uncertified_count`
- `subscription_support_trust_cache_hit_count`
- `subscription_support_trust_cache_invalidation_count`
- `subscription_support_access_structure_debt_count`
- `subscription_support_foreground_certification_rejection_count`
- `subscription_support_payload_budget_rejection_count`
- `subscription_support_allocation_scope_violation_count`
- `subscription_support_index_probe_count`
- `subscription_support_batch_plan_count`
- `subscription_support_store_global_rejection_count`
- `subscription_support_receipt_reuse_count`
- `subscription_support_clone_boundary_count`
- `subscription_support_dense_plan_count`
- `subscription_support_sparse_plan_count`
- `subscription_support_read_amplification_count`
- `subscription_support_certification_row_count`
- `subscription_support_certification_gap_count`
- `subscription_support_generic_certification_row_count`
- `subscription_support_domain_certification_row_count`
- `subscription_support_forbidden_exact_overclaim_count`
- `subscription_support_global_scan_debt_count`

Required counter assertions:

- `subscription_support_forbidden_exact_overclaim_count` remains zero in all
  required certification lanes
- `subscription_support_global_scan_debt_count` remains zero in the required
  named suite
- exact, degraded, rebuild-derived, replicated, migrated, stale, and rejected
  counters match the certification matrix rows exactly
- equivalence witness counts increment only in rebuild, migration, replication,
  and import lanes that prove equivalence
- certification gap counts increment for intentionally incomplete bundles and
  remain zero for the required closeout bundle
- operational-uncertified counts increment when local exact trust exists without
  platform certification and remain absent from exported certified bundles
- cache hit counts increment only when all trust epochs match
- foreground certification rejection counts increment when resume paths attempt
  certification-matrix or domain-row work inline
- payload budget and allocation scope counters match hostile oversized-evidence
  lanes exactly
- index probe and read-amplification counters match the declared access path for
  single, family-local, basis-local, certification-scope, and domain-scenario
  lanes
- receipt reuse counters prove batch lanes do not rediscover identical
  compatibility, basis, cursor/checkpoint, operational, or portability receipts
- store-global rejection counters increment for attempted whole-store trust
  operations and remain zero for admitted required lanes
- clone-boundary counters increment only for declared comparison, export, or
  offline-verifier handoff boundaries
- sparse and dense plan counters reflect the selected density strategy rather
  than hiding a density shift inside execution

## Architectural Shape

Milestone 13.3 should be implemented as a dedicated trust and certification
layer inside the existing subscription-support domain, not as scattered checks
inside resume classification, retention, replication, or tests.

Preferred module responsibilities:

- `subscription_support/trust/taxonomy/`
  trust strength, trust provenance, role-scoped posture, and downgrade reasons
- `subscription_support/trust/receipts/`
  concrete receipts consumed from Milestones 13.1 and 13.2
- `subscription_support/trust/translation/`
  legal translation from resume classifications and operational verdicts
- `subscription_support/trust/typestate/`
  proof-widening pipeline types and sealed witnesses
- `subscription_support/trust/equivalence/`
  rebuild, migration, replication, and import equivalence contracts
- `subscription_support/trust/drift/`
  drift scans, staleness, epoch freshness, and suppressed-cause reports
- `subscription_support/trust/cache/`
  derived trust cache keys, hit witnesses, and invalidation reports
- `subscription_support/trust/certification/`
  coverage matrices, row validation, certified bundles, and certification stamps
- `subscription_support/trust/domain_certification/`
  first-ship domain scenario rows and explicit debt rows
- `subscription_support/trust/evidence/`
  counters, complexity surfaces, budget reports, and Roadmap handoff reports

Rules:

- Milestone 13.1 classification code may call into trust translation, but it may
  not own trust taxonomy or certification closure
- Milestone 13.2 operational participation code may emit receipts, but it may
  not decide final trust posture
- backend persistence adapters may store trust reports and certification
  bundles, but they may not synthesize trust witnesses from decoded rows
- test code may assemble hostile lanes, but it may not use test-only constructors
  to bypass production witness creation

Naive trap this prevents:

- implementing 13.3 as a pile of certification helpers that know too much about
  earlier modules and are impossible to reuse from Milestone 14, Milestone 15,
  or Milestone 17.

## Phases

### Phase 1: Lock Trust Taxonomy, Role Scope, And Translation Boundaries

Phase 1 defines what subscription-support trust is allowed to mean before any
certification row or trust witness ships.

Required work:

- define `SubscriptionSupportTrustClass` and role-scoped trust posture
- split trust into `SupportTrustStrength` and `SupportTrustProvenance`
- define exact, degraded, rebuild-derived, replicated, migrated, stale,
  policy-rejected, and unsupported trust classes
- define operational trust versus certified trust report families
- define trust classification plans and reports over family id, support role,
  basis, cursor/checkpoint, compatibility, and operational verdicts
- define translation plans from Milestone 13.1 resume classifications and
  Milestone 13.2 operational verdicts
- define sealed trust witnesses for exact, degraded, rebuild-derived, rejected,
  and unsupported trust posture
- define support trust epochs for catalog, operational ledger, compatibility,
  and certification corpus versions
- define downgrade reasons, drift families, deterministic drift precedence, and
  suppressed-cause reporting
- define the typed trust failure taxonomy and recovery posture vocabulary
- define required trust access layout families and access-path contracts
- define density classes and batch cardinality rules
- define lowered performance plan families for classification, drift,
  equivalence, certification, and domain work
- define allocation scopes, scratch arenas, move-only evidence packets, and
  clone-boundary reason codes
- define initial complexity contracts, counters, and bundle shape
- define compile-time boundaries for exact-trust witness construction and
  cross-family trust reuse

Exit condition:

- exact support trust has one proof chain and one role-scoped meaning
- operational verdicts and resume classifications cannot be mistaken for final
  trust posture
- weaker support posture cannot construct exact support trust by API shape

### Phase 2: Classify Support Trust From Resume And Operational Evidence

Phase 2 makes trust classification executable while preserving the earlier
proof chains.

Required work:

- implement the trust typestate pipeline from raw trust request to operational
  trust classification
- implement concrete receipts for resume classification, operational verdict,
  family role, basis, cursor/checkpoint, compatibility, portability,
  maintenance, retention, and import admission
- implement scalar calls as `SingleSupportArtifact` batch plans over shared
  batch machinery
- implement lowered operational trust classification plans that resolve access
  paths, receipt sets, epoch requirements, cache eligibility, allocation scope,
  and expected counters before execution
- implement trust translation from exact, degraded, rebuild-required,
  not-resumable, and policy-rejected inputs
- implement classification reports for exact, degraded, rebuild-derived,
  replicated, migrated, stale, policy-rejected, and unsupported support
- reject role mismatch and family mismatch before trust witness construction
- reject missing operational participation evidence before exact trust
  classification
- implement epoch freshness checks before cache reuse or exact trust handle
  construction
- implement public result cost surfaces for classification breadth, rows read,
  proof receipts consumed, and drift checks performed
- emit exact trust translation and classification counters
- add unit and compile-fail coverage for synthetic exact trust, degraded-as-exact
  misuse, and cross-family trust reuse

Exit condition:

- support artifacts receive trust posture only through proof-bearing resume and
  operational inputs
- exact support trust is impossible to construct from raw durable rows,
  degraded support, or rebuild-required posture

### Phase 3: Prove Rebuild, Migration, Replication, And Import Equivalence

Phase 3 prevents operationally transformed support artifacts from inheriting
trust strength without explicit equivalence proof.

Required work:

- define support trust equivalence contracts for rebuild, migration,
  replication, and import lanes
- implement sealed equivalence witnesses for each transformed support path
- distinguish identity preservation from trust preservation
- reject digest-only equivalence claims when role, basis, compatibility,
  operational verdict, or portability scope differs
- classify rebuilt and migrated support as exact only when equivalence proves
  digest basis and resume classifier equivalence
- classify replicated and imported support as exact only when portability and
  target-side admission evidence preserve the same support role
- emit equivalence witness, equivalence failure, and transformed-trust counters

Exit condition:

- transformed support artifacts can preserve, downgrade, or reject trust
  explicitly
- replicated or imported support cannot claim stronger trust than source and
  target evidence prove

### Phase 4: Detect Drift, Staleness, And Coverage Gaps

Phase 4 makes trust posture hostile to stale evidence.

Required work:

- implement family, role, basis, cursor/checkpoint, digest, compatibility,
  operational-verdict, portability, and certification-coverage drift detection
- implement deterministic primary and suppressed drift cause reports
- implement stale-support rejection and policy-rejection reports
- ensure placement-cost drift changes diagnostics and counters only unless
  support evidence was actually lost
- implement bounded drift plans by family, basis, support identity, or
  certification scope
- implement trust cache keys, cache hit witnesses, and cache invalidation
  reports with epoch-aware equivalence
- implement access-structure verification for support identity, family/role,
  basis, cursor/checkpoint, operational action, compatibility epoch,
  certification row, and domain scenario lookups
- implement read-amplification and index-probe counters for each required
  access structure
- reject global support-history scans in required trust classification lanes
- reject foreground resume paths that try to build certification matrices,
  domain rows, or missing access structures inline
- enforce evidence, matrix, domain-scenario, and handoff payload budgets before
  decode or allocation-heavy materialization
- emit drift detection, stale rejection, coverage drift, and global-scan debt
  counters

Exit condition:

- stale or coverage-incomplete support cannot report platform-trusted exactness
- multi-drift trust failures are deterministic, audited, and bounded by the
  classification plan

### Phase 5: Build Certification Coverage And Evidence Bundles

Phase 5 turns trust classification into machine-checkable certification
evidence.

Required work:

- implement subscription-support certification rows and coverage matrices
- require control, hostile, and rebuild/replay lanes for each matrix row
- implement family-local, basis-local, certification-scope-local, and
  domain-scenario-local certification batch plans
- implement receipt reuse across certification rows and exact reuse counters
- implement structured bundle records for run header, coverage matrix, row
  evidence, counter snapshot, drift matrix, equivalence matrix, generic report,
  domain report, and Roadmap 2 handoff report
- validate row labels against family, support role, trust class, drift cause,
  operational verdict, and required counters
- reject duplicate rows, missing rows, mislabeled rows, row self-comparison, and
  incomplete bundle construction
- reject row evidence whose structured fields do not recompute to the declared
  row digest
- emit artifact, subscription-support, diagnostics, counter, and certification
  summary outputs
- expose certification gap reports and forbidden exact-overclaim counters
- add compile-fail coverage for raw row certification and incomplete certified
  bundle construction

Exit condition:

- certification proves support trust posture instead of artifact presence
- required support families and roles have audited coverage with machine-checkable
  pass/fail evidence

### Phase 6: Thread Support Trust Into Generic And Domain Certification

Phase 6 makes the 13.3 result visible to later store certification.

Required work:

- update generic store certification surfaces to include subscription-support
  trust coverage
- define domain-facing subscription-support scenarios for geometry/CAD,
  web/data, AI, chip/simulation, and offline/collaborative workloads
- implement conservative first-ship domain certification rows over admitted
  support families
- implement required first-ship family coverage for
  `BasisBoundContinuationSupport`, `MaterializedNarrowingSupport`,
  `DegradedContinuationSupport`, and `ExtensionDefinedSupport` rejection
- implement the required first-ship scenario rows using
  `BasisBoundContinuationSupport`, `MaterializedNarrowingSupport`, and
  `DegradedContinuationSupport`
- implement domain scenario batch plans with declared scenario width,
  family-role row width, index probes, receipt reuse, and allocation budget
- mark absent advanced support families as explicit debt rather than implied
  coverage
- emit certification handoff reports for Milestone 14 and Roadmap 2
- distinguish semantic support trust from physical database readiness in the
  handoff evidence
- expose exact generic and domain certification counters

Exit condition:

- generic and domain certification can audit subscription-support trust posture
- later roadmap work consumes one support trust vocabulary instead of raw rows,
  operational logs, or certification folklore

### Phase 7: Prove Subscription-Support Accuracy And Certification

Phase 7 closes the milestone with the named certification suite.

Required work:

- run the Milestone 13.3 named suite:
  `Subscription-Support Accuracy And Certification Test`
- include exact, degraded, rebuilt, migrated, replicated, imported, stale,
  omitted, and policy-rejected support variants
- include family-role mismatch and cross-family witness misuse lanes
- include stale support, compatibility drift, operational verdict drift,
  portability drift, and certification coverage drift lanes
- include generic and domain certification rows
- include certification gap, duplicate-row, mislabeled-row, and row
  self-comparison rejection lanes
- include forbidden exact-overclaim and forbidden global-scan lanes
- include compile-fail coverage for sealed trust witnesses, raw row
  certification, degraded-as-exact misuse, cross-family witness reuse, and
  incomplete certified bundles
- emit machine-checkable artifact, subscription-support, diagnostics, counter,
  and certification summary bundles

Exit condition:

- every shipped subscription-support family has explicit trust posture and
  certification coverage
- stale, rebuilt, degraded, incompatible, omitted, or role-mismatched support
  cannot masquerade as exact resumability proof
- generic and domain certification can consume subscription-support trust
  posture without reinterpreting raw operational evidence

## Must Ship

- enforced subscription-support trust taxonomy for admitted support families,
  split into `SupportTrustStrength` and `SupportTrustProvenance`
- distinct operational trust and certified trust report families
- role-scoped trust classification plans, reports, and sealed witnesses
- proof-widening trust typestate pipeline from raw request to certified trust
- legal translation from Milestone 13.1 resume classifications and Milestone
  13.2 operational verdicts into trust posture
- concrete receipt types for resume, operational, family-role, basis,
  cursor/checkpoint, compatibility, portability, maintenance, retention, and
  import-admission evidence
- explicit trust classes:
  - `ExactSupportTrusted`
  - `DegradedSupportTrusted`
  - `RebuildDerivedSupport`
  - `ReplicatedSupportTrusted`
  - `MigratedSupportTrusted`
  - `StaleSupportRejected`
  - `PolicyRejectedSupport`
  - `UnsupportedSupportTrust`
- support trust equivalence contracts for rebuild, migration, replication, and
  import
- support trust epoch and freshness model across catalog, operational ledger,
  compatibility, and certification corpus versions
- derived trust cache keys, hit witnesses, invalidation reports, and
  equivalence contracts
- dedicated `subscription_support/trust/` module family with taxonomy,
  receipts, translation, typestate, equivalence, drift, cache, certification,
  domain-certification, and evidence responsibilities
- declared trust access layout families and index rebuild authorities
- batch cardinality model with density classes from `SingleSupportArtifact`
  through `StoreGlobalRejected`
- lowered performance plan families for classification, drift, equivalence,
  certification, and domain scenarios
- scalar APIs implemented as single-item batch plans over the same machinery
- scratch arenas, borrowed slices, move-only evidence packets, clone-boundary
  counters, and allocation reports
- drift and staleness reports for family, role, basis, cursor/checkpoint,
  digest, compatibility, operational verdict, portability, and certification
  coverage
- access-structure requirements for support identity, family/role, basis,
  cursor/checkpoint, operational action, compatibility epoch, certification row,
  and domain scenario lookups
- path-class restrictions separating foreground resume trust, batch
  certification, domain certification, Roadmap handoff, and trust-cache rebuild
  work
- payload, matrix, domain-scenario, handoff, and allocation budgets
- typed trust failure taxonomy with recoverability posture
- exact complexity contracts for operational classification, batch
  classification, access paths, allocation, certification, and domain
  certification
- subscription-support certification coverage matrix and row validation
- structured certification bundle records with offline-checkable row evidence
- certification gap reports and forbidden exact-overclaim counters
- generic store subscription-support certification surfaces
- domain subscription-support certification rows for first-ship product
  scenarios
- required first-ship family coverage for `BasisBoundContinuationSupport`,
  `MaterializedNarrowingSupport`, `DegradedContinuationSupport`, and typed
  `ExtensionDefinedSupport` rejection
- certification handoff report separating semantic support trust from Roadmap 2
  physical database readiness
- compile-fail boundary coverage for trust witness construction, weaker posture
  misuse, raw row certification, cross-family witness reuse, and incomplete
  bundle construction
- exact trust classification, drift, equivalence, certification, generic-row,
  domain-row, epoch, cache, access-debt, budget-rejection, and
  forbidden-overclaim counters
- machine-checkable Milestone 13.3 certification output

## Must Preserve

- canonical truth remains authoritative; subscription-support artifacts never
  become truth authority
- runtime bridge, `worth-signal`, `worth-query`, and server layers remain owners
  of subscription meaning, lowering, delivery, fanout, and lifecycle semantics
- Milestone 13.1 durable support identity and resume classification remain the
  support proof chain consumed here
- Milestone 13.2 operational verdicts remain the source of retention,
  compatibility, replication, import, and maintenance consequences
- placement and recall remain cost posture only
- support trust posture remains family-aware and role-scoped
- rebuilt, degraded, stale, replicated, migrated, omitted, or policy-rejected
  support cannot claim exact resumability unless its declared proof rules allow
  that exact claim
- Roadmap 2 remains the gate for physical database posture; this milestone does
  not overclaim physical boundedness, media durability, or platform-grade
  backend readiness

## Acceptance Evidence

Milestone 13.3 is complete only when the store satisfies the named Milestone
13.3 suite:

- `Subscription-Support Accuracy And Certification Test`

Required machine-checkable outputs:

- `artifact_digest`
- `subscription_support_digest`
- `diagnostics_digest`
- `counter_snapshot`
- `certification_summary`

Minimum certification matrix rows:

- `exact_support_trusted_control`
  classifies an admitted exact support artifact as exact only after resume,
  operational, role, compatibility, and basis evidence align, then produces a
  certified exact report only after coverage evidence is checked.
- `degraded_support_trusted`
  classifies degraded support as degraded and proves it cannot construct exact
  resume trust.
- `rebuild_derived_support_exact_equivalence`
  rebuilds support and preserves exact trust only through rebuild equivalence
  evidence.
- `rebuild_derived_support_downgraded`
  rebuilds support with incomplete equivalence and downgrades or rejects exact
  trust.
- `replicated_support_identity_not_enough`
  preserves support identity across replication but rejects exact trust when
  portability evidence is incomplete.
- `replicated_support_exact_equivalence`
  preserves exact trust across replication through source and target
  equivalence evidence.
- `migrated_support_exact_equivalence`
  migrates a support format while preserving exact trust through digest-basis
  and resume-classifier equivalence.
- `imported_support_missing_basis_not_resumable`
  imports support without required basis evidence and rejects trusted exactness.
- `stale_support_rejected`
  detects stale support evidence and prevents exact trust consumption.
- `policy_rejected_support`
  classifies policy-rejected support as non-resumable for trust purposes.
- `family_role_mismatch_rejected`
  rejects a support artifact whose family or declared role differs from the
  trust request.
- `compatibility_drift_rejects_exact_trust`
  proves decode success cannot preserve exact trust when compatibility evidence
  drifted.
- `operational_verdict_drift_rejects_exact_trust`
  proves stale or contradictory Milestone 13.2 operational evidence blocks
  exact trust.
- `portability_drift_rejects_exact_trust`
  proves partial replication or omitted support scope cannot report exact trust
  on the target.
- `coverage_drift_rejects_platform_trust`
  proves local classification success is insufficient when required
  certification coverage is missing or stale.
- `multi_drift_precedence_deterministic`
  injects multiple trust failures and proves primary and suppressed causes are
  deterministic.
- `certification_matrix_complete`
  emits a complete family-role coverage matrix with exact counter assertions.
- `certification_missing_row_rejected`
  rejects a bundle with an omitted required row.
- `certification_duplicate_row_rejected`
  rejects a bundle with duplicate or conflicting row evidence.
- `certification_mislabeled_row_rejected`
  rejects row labels that do not match family, role, trust class, or drift
  evidence.
- `certification_self_comparison_rejected`
  proves same-run self-comparison is not accepted as certification proof.
- `generic_certification_includes_support_trust`
  proves generic store certification includes subscription-support trust rows.
- `domain_geometry_support_trust`
  proves geometry/CAD session continuation uses role-scoped support trust.
- `domain_web_data_support_trust`
  proves web/data restart and partial replication lanes classify support trust
  honestly.
- `domain_ai_degraded_support_trust`
  proves AI branch workspace continuation can report degraded support without
  exact overclaim.
- `domain_chip_rebuild_support_trust`
  proves chip/simulation long-history support can be rebuild-derived without
  becoming shadow truth.
- `domain_offline_omitted_support_trust`
  proves offline/collaborative capsule import reports omitted support as
  non-exact on the target.
- `forbidden_exact_overclaim_zero`
  proves exact overclaim counters remain zero across required hostile lanes.
- `global_scan_debt_forbidden`
  proves required trust classification and certification lanes do not fall back
  to global support-history scans.
- `roadmap_2_handoff_physical_debt_explicit`
  emits handoff evidence that separates semantic support trust closure from
  physical database readiness.

Milestone-specific proof obligations:

- every shipped subscription-support family has explicit trust posture
- every trust posture is scoped to family, role, basis, cursor/checkpoint,
  compatibility, operational verdict, and certification row evidence
- exact support trust is constructible only through legal resume,
  operational-verdict, equivalence, and coverage witnesses
- operational exact trust and certified exact trust remain distinct; exported,
  replicated, generic-certification, domain-certification, and Milestone 14
  handoff paths cannot consume uncertified operational reports
- degraded, rebuild-derived, stale, policy-rejected, unsupported, or
  role-mismatched support cannot construct exact resume trust
- rebuilt, migrated, replicated, and imported support preserve exact trust only
  through explicit equivalence witnesses
- stale support, compatibility drift, operational drift, portability drift, and
  certification coverage drift localize to typed failures
- epoch changes invalidate cached trust unless an explicit epoch translation
  witness exists
- required access structures are present or the required lane rejects with typed
  access debt rather than scanning global support history
- every required lane consumes a lowered performance plan that names access
  path, density class, receipt reuse, allocation scope, and expected counters
  before execution
- scalar trust APIs are proven to lower to `SingleSupportArtifact` batch plans
  rather than duplicating scalar-only proof logic
- family-local, basis-local, certification-scope-local, and domain-scenario
  certification lanes amortize receipt loading through batch receipt caches
- foreground resume paths cannot build certification matrices, domain rows, or
  Roadmap handoff reports inline
- oversized evidence, certification matrices, domain scenario records, and
  handoff reports reject before decode or allocation-heavy materialization
- foreground trust classification uses operation-scoped scratch storage and
  does not allocate per receipt or per row in required lanes
- evidence packet clones occur only at declared comparison, export, or offline
  verifier handoff boundaries
- certification rows prove trust posture rather than artifact existence
- certification bundles reject missing, duplicate, mislabeled, self-comparing,
  or coverage-incomplete evidence
- certification bundle row digests are recomputable from structured fields and
  stable under map iteration order and platform path formatting
- first-ship family coverage includes basis-bound continuation,
  materialized narrowing, degraded continuation, and typed extension-family
  rejection lanes
- generic and domain certification include first-class subscription-support
  trust lanes
- absent advanced support families are marked explicit debt rather than implied
  coverage
- no certification lane relies on logs or same-run self-comparison as proof
- `subscription_support_forbidden_exact_overclaim_count` remains zero
- `subscription_support_global_scan_debt_count` remains zero in required
  certification lanes
- `subscription_support_foreground_certification_rejection_count`,
  `subscription_support_payload_budget_rejection_count`, and
  `subscription_support_access_structure_debt_count` match their hostile lanes
  exactly
- index probe, receipt reuse, allocation, clone-boundary, sparse-plan,
  dense-plan, and read-amplification counters match the declared complexity
  contracts exactly
- compile-fail tests prevent sealed-witness synthesis, weaker-posture exact
  misuse, raw-row certification, cross-family witness reuse, and incomplete
  bundle completion

Milestone 13.3 is not closed by "the support artifact still exists," "the
resume classifier returned exact once," or "the certification bundle emitted a
row" tests.

## Architectural Notes

- The smart abstraction is not an "accuracy flag." The smart abstraction is a
  role-scoped trust witness built from resume proof, operational survival proof,
  equivalence proof, and certification coverage.
- Exact support trust should be intentionally hard to obtain. The system is
  healthier when degraded, rebuild-derived, stale, or unsupported outcomes are
  explicit rather than laundered into exactness.
- Certification belongs in the support subsystem as a proof consumer, not as a
  log formatter at the end of tests.
- Milestone 13.3 should make Milestone 14 simpler: replication can consume
  support trust posture instead of inventing subscription-support exactness
  during capsule integrity work.
- This milestone deliberately stops before Roadmap 2 physical database claims.
  Semantic support trust can close while physical database posture remains a
  named next gate.

## Sequencing Notes

This milestone belongs immediately after Milestone 13.2 because 13.2 produces
the operational verdicts that trust classification must consume.

- Milestone 13.1 supplies durable support identity, basis linkage, and resume
  classification.
- Milestone 13.2 supplies operational survival, compatibility, portability,
  import, and maintenance consequences.
- Milestone 13.3 assigns final trust posture and certification coverage over
  those surfaces.
- Roadmap 2 follows as the physical database foundation gate before post-13.3
  platform milestones proceed as platform-grade work.
- Milestone 14 should consume 13.3 support trust posture when deciding how
  replication, capsules, and integrity verification carry subscription-support
  artifacts.
- Milestone 15 and Milestone 17 should inherit this support-specific taxonomy
  for extension-defined support families and the broader derived-artifact
  accuracy model.

## Required Self-Check

- Does the milestone solve a real structural problem or just package work
  cosmetically?
  Yes. It closes the trust-posture gap left after durable identity and
  operational participation so exact subscription resumability cannot be
  overclaimed by stale, transformed, or under-certified support artifacts.
- Is the adversarial constraint precise and load-bearing?
  Yes. Every phase prevents a weaker support variant from masquerading as exact
  trust under family, role, drift, transformation, or coverage pressure.
- Does the milestone preserve crate authority boundaries?
  Yes. Store classifies durable support evidence for declared roles while
  runtime/query/server layers continue owning subscription semantics and
  delivery behavior.
- Does the milestone define proof obligations, not just implementation tasks?
  Yes. It names the required suite, matrix rows, machine-checkable outputs,
  exact counters, zero-overclaim assertions, and compile-fail boundaries.
- Could a competent engineer map this spec into honest types, modules, and
  tests?
  Yes. The spec names taxonomy, translation, equivalence, drift, certification,
  generic/domain certification, phases, counters, and hostile lanes.
- Does the milestone belong in this roadmap sequence, or is it out of order?
  Yes. It follows 13.2 operational participation and must close before the
  Roadmap 2 gate and later Milestone 14 consume subscription-support durability
  as complete.
