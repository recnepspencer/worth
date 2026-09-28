# Storage Foundation S.0 Engineering Spec: Shipped Store Reconciliation And Capability Reclassification

> **Status:** Planned
>
> **Roadmap parent:** [physical-database-roadmap.md](./physical-database-roadmap.md)
>
> **Vision parent:** [worth_store_vision.md](./worth_store_vision.md)
>
> **Test requirements:**
> - [test-requirements.md](./test-requirements.md)
> - [test-requirements-2.md](./test-requirements-2.md)
>
> **Prerequisite roadmap state:** `Milestone 13.3` semantic subscription-support
> cleanup is closed and is the handoff point from Roadmap 1 into Roadmap 2.
>
> **Follow-on storage-foundation sequence:** `S.1`
>
> **Primary architectural driver:** stop Store from overclaiming database-grade
> physical behavior before the physical substrate exists.

## Goal

Reconcile the Store work already completed or planned in Roadmap 1 against the
Roadmap 2 physical database foundation, and classify every backend, milestone
claim, closeout claim, and certification lane by the kind of evidence it
actually proves.

S.0 is complete when a reader can tell, without inference, which Store claims
are already semantically true and which physical database guarantees are still
deferred to `S.1` through `S.12`.

## Why This Sequence Exists

The first Worth Store roadmap built a strong semantic durability program:
canonical authority, operating modes, WAL-shaped recovery, snapshots, branch
deltas, layout materialization, schema and cursor support, retention,
maintenance, compatibility, tiering, and first-class subscription-support
artifacts.

That work is real. It should not be thrown away.

But it still leaves a dangerous ambiguity: a semantic persistence harness can
pass impressive replay, restart, retention, and subscription-support tests while
still not being a physical database. It can depend on heap-shaped state,
serde-loaded full objects, backend-local residue, SQLite/file behavior, broad
scans, or unbounded memory while using language such as "production-grade",
"platform-grade", or "database".

S.0 exists to remove that ambiguity before S.1 starts building the physical byte
substrate.

If this sequence is skipped, every later Roadmap 2 milestone inherits bad
language and bad proof boundaries:

- S.1 cannot tell which backend is supposed to expose pages, segments, extents,
  manifests, and physical references.
- S.2 cannot prove bounded memory if earlier tests are allowed to heap-load the
  store and still call the result database evidence.
- S.3 cannot localize corruption if earlier digest or deserialize failures are
  still allowed to count as physical integrity evidence.
- S.4 cannot define recovery physics if existing WAL/recovery language already
  implies database-grade LSN/checkpoint behavior.
- S.12 cannot certify physical readiness if the earlier roadmap has never
  separated semantic correctness from physical substrate readiness.

S.0 is therefore the honesty pass. It backtracks without denigrating the work
already done.

## Governing Summaries

- `MENTALITY.md`
  protects adversarial, hard-problem-first architecture. S.0 follows it by
  naming the foundation gap before building more platform features on top of
  unclear physical claims.
- `arch_laws.md`
  protects proof-bearing boundaries and authority separation. S.0 must separate
  semantic authority, physical byte authority, backend capability, and
  certification evidence into distinct typed claims.
- `perf_laws.md`
  protects visible, testable cost. S.0 must identify where existing evidence
  proves replay or parity but not resident-memory bounds, allocation bounds,
  physical access breadth, recovery breadth, or foreground interference.
- `domain_laws.md`
  protects decomposition by responsibility. S.0 treats backend classification,
  milestone claim audit, closeout evidence audit, deferred guarantee mapping,
  and terminology cleanup as separate responsibilities because they fail and
  change for different reasons.
- `worth_store_vision.md`
  protects the thesis that Store makes truth survive without owning truth
  semantics. S.0 preserves that semantic survival work while clarifying that
  physical media survival, bounded memory, native chunks, and operator-grade
  repair are Roadmap 2 responsibilities.
- `runtime-integration-roadmap.md`
  defines Part II as a fresh consumer of the closed physical foundation. S.0
  must prevent retired semantic implementation claims from being mistaken for
  physical database readiness or earned runtime-integration credit.
- `physical-database-roadmap.md`
  protects the physical database backtrack. S.0 is the first gate because the
  project needs claim hygiene before building pages, buffers, integrity,
  recovery physics, I/O, blob chunks, backup, security, and certification.
- `test-requirements.md`
  protects machine-checkable closeout. S.0 must map the existing milestone
  suites to what they prove semantically and what they do not prove physically.
- `test-requirements-2.md`
  protects adversarial physical harness realism. S.0 must classify harness
  maturity and name the evidence bundle shape that later `S.*` sequences will
  consume.
- `milestone-13.md`
  protects tiering as non-authority. S.0 must preserve that result while
  clarifying that tier movement is not yet backed by Roadmap 2 physical read
  stability, I/O isolation, or page/chunk integrity.
- `milestone-13.1.md`
  protects durable subscription-support identity and resume classification. S.0
  must preserve that semantic support contract while clarifying that persisted
  support artifacts still need physical database substrate classification.
- `milestone-13.2.md`
  protects subscription-support participation through retention, compatibility,
  replication, and maintenance. S.0 must treat it as semantic/operational
  support work that still depends on Roadmap 2 for platform-grade physical
  claims.
- `milestone-13.3.md` and `milestone-13.3-closeout.md`
  protect role-scoped subscription-support trust classification and
  certification. S.0 must preserve that closed semantic trust result while
  treating its own closeout language seriously: Milestone 13.3 does not claim
  physical database readiness, durable certification-run persistence, or
  Roadmap 2 foundation closure.
- `storage-foundation-s1.md`
  protects the first physical substrate implementation step. S.0 must produce
  the backend tiers, deferred guarantee map, and forbidden-claim list that S.1
  consumes.

## Adversarial Constraint

S.0 must survive this hostile condition:

> A reader, implementer, closeout reviewer, or later milestone author audits
> Store after the `13.x` semantic arc and must not be able to mistake semantic
> durability evidence, heap/file/SQLite bootstrap persistence, serde-loaded
> object reconstruction, backend-local residue, happy-path reopen behavior, or
> digest-level artifact parity for platform-grade physical database behavior.

If any milestone, closeout, backend, test suite, or roadmap phrase still lets
heap-shaped state, full-object decode, broad scans, backend-private residue, or
unverified media assumptions masquerade as database-grade physical evidence,
S.0 is not closed.

## Product Decision Lock

- earlier semantic Store work remains valid and valuable
- Roadmap 2 is a physical database foundation gate, not a repudiation of the
  first roadmap
- semantic durability evidence and physical database evidence are separate
  claim families
- backend capability tiers are claim boundaries, not value judgments
- no current heap/file/SQLite path may be described as platform-grade unless it
  satisfies the Roadmap 2 gates it claims
- "production-grade embedded backend" language must be rewritten or annotated
  wherever it implies physical database posture not yet proven
- S.0 may classify, audit, report, and clean terminology; it must not implement
  S.1 page/segment/extent substrate work
- later milestones may proceed only with explicit non-platform-grade debt if
  they depend on unclosed Roadmap 2 guarantees
- S.0 artifacts, evidence bundles, handoff readiness, claim reports, backend
  matrices, and migration notes must be typed aspect-native Store/Foundational
  surfaces; JSON, serde-shaped objects, debug strings, display names, raw
  bytes, and producer-private names may appear only as hostile inputs,
  rejected compatibility-origin fixtures, or explicitly named migration debt
- no S.1 consumer may treat parseable JSON, a schema-compatible object, or a
  copied report as evidence; S.1 may consume only admitted native witnesses,
  aspect-native evidence artifacts, and proof-bearing handoff readiness

## Scope

### In Scope

- backend capability-tier vocabulary for existing and future Store backends
- audit of Roadmap 1 milestones and closeouts through `Milestone 13.3`
- audit of Roadmap 1 and Roadmap 2 wording that implies platform-grade physical
  behavior
- separation of already-earned semantic guarantees from deferred physical
  guarantees
- mapping of deferred physical guarantees to `S.1` through `S.12`
- migration notes for tests that prove semantic parity but not physical
  boundedness, integrity, recovery physics, or I/O isolation
- certification-harness maturity classification for Roadmap 2 evidence
- terminology cleanup for overbroad "production-grade", "platform-grade",
  "database", "embedded backend", "WAL", "recovery", "durability", "blob", and
  "replication" claims where those words imply unproven physical behavior
- machine-checkable S.0 evidence bundle shape

### Explicitly Out Of Scope

- physical pages, segments, extents, manifests, and record framing, which are
  S.1
- buffer pool, resident memory, page leases, and allocation envelopes, which are
  S.2
- checksums, scrub, quarantine, and corruption localization, which are S.3
- LSN/pageLSN/checkpoint recovery physics, which are S.4
- physical read isolation, latches, epochs, and stable read plans, which are S.5
- I/O QoS and hardware capability qualification, which are S.6
- native blob chunk trees and streaming large-object substrate, which are S.7
- per-artifact-family layout strategy discipline, which is S.8
- formal models, operator repair, security, tenant boundaries, and final
  physical certification, which are S.9 through S.12
- reworking semantic Store architecture already proven by Roadmap 1

## Required Contracts And Counters

### Canonical S.0 Artifact Set Rule

S.0 must produce canonical artifacts that later code and specs can consume
without rereading prose.

Required artifact directory:

- `plans/worth-store/artifacts/storage-foundation-s0/`

Required typed aspect-native artifacts:

- `backend-capability-matrix.aspec`
- `milestone-physical-status-matrix.aspec`
- `semantic-physical-claim-report.aspec`
- `deferred-physical-guarantee-map.aspec`
- `terminology-risk-report.aspec`
- `test-migration-notes.aspec`
- `harness-maturity-report.aspec`
- `s1-handoff-readiness.aspec`
- `s0-evidence-bundle.aspec`

Required implementation-facing surfaces:

- `StorageFoundationS0EvidenceBundle`
- `BackendCapabilityMatrix`
- `MilestonePhysicalStatusMatrix`
- `SemanticPhysicalClaimReport`
- `DeferredPhysicalGuaranteeMap`
- `TerminologyRiskReport`
- `HarnessMaturityReport`
- `StorageFoundationS1Handoff`

Rules:

- typed aspect-native artifacts are the closeout source of truth; prose
  summarizes them
- each artifact must lower through `worth-foundational` aspec-native value,
  identity, locator, profile, diagnostic, canonical-basis, receipt, and
  performance vocabulary where those surfaces exist
- each artifact must include a schema version, source revision, generated-at
  policy, and deterministic digest over semantically relevant aspec fields
- timestamps, local paths, and host names may appear only in explicitly
  excluded nondeterministic metadata sections
- S.1 may not consume hand-written summaries when the canonical S.0 artifact is
  present
- S.0 closeout must fail if any required artifact is missing, not admitted as
  aspect-native evidence, schema-incompatible, digest-inconsistent, or backed
  by JSON-shaped objects, debug strings, raw bytes, or producer-private names

JSON compatibility stance:

- JSON may appear only as a hostile input, rejected compatibility-origin
  fixture, or explicitly named migration debt record.
- JSON may not be the canonical source of truth, closeout artifact format,
  evidence bundle shape, digest basis, S.1 handoff payload, or proof witness.
- Parseable JSON is not evidence. Only typed aspect-native admission plus
  canonical-basis proof can satisfy S.0 closeout.

Naive trap this prevents:

- producing a beautiful audit document that nobody can import, diff, validate,
  or use as an implementation gate.

### Canonical Artifact Schema Floor Rule

Every S.0 artifact must share a common envelope and each matrix row must have a
stable identity.

Required common artifact fields:

- `schema_version`
- `artifact_kind`
- `source_revision`
- `roadmap_parent_digest`
- `generated_by`
- `deterministic_digest`
- `nondeterministic_metadata`
- `rows`

Required row fields for all matrix-style artifacts:

- `row_id`
- `subject_kind`
- `subject_path_or_symbol`
- `classification`
- `evidence_refs`
- `forbidden_claims`
- `deferred_s_sequences`
- `status`
- `notes`

Rules:

- `row_id` must be stable across reruns unless the subject itself is renamed
- `row_id` must not include line numbers, timestamps, host names, or generated
  counters
- `evidence_refs` must point to local docs, source files, test suites, or
  evidence bundles; free text is not enough
- `deferred_s_sequences` must be empty only when the row has no physical debt or
  when the physical gap is explicitly `NotApplicable`
- `notes` may explain but may not carry the only copy of an enforceable fact
- status matrices must be sortable without changing their digest

Naive trap this prevents:

- shipping JSON that is technically parseable but still impossible to diff,
  merge, or use as a stable gate because the important facts live in prose
  strings.

### S.0 Complexity Contract Rule

Every S.0 operation that touches repo-scale inputs must declare a named
complexity contract, exact counters, and a `Verified` or `Debt` status.

Required complexity surfaces:

- `S0ComplexityContract`
- `S0ComplexityStatus`
- `S0AuditInputManifest`
- `S0AuditBreadthSummary`
- `S0ScanCostSurface`
- `S0ArtifactValidationCostSurface`
- `S0DigestCostSurface`
- `S0EvidenceResolutionCostSurface`
- `S0HandoffCostSurface`

Minimum named contracts:

- `s0_input_manifest_construction`
  Cost is proportional to declared scan roots plus matched files, not total
  workspace files.
- `s0_terminology_scan`
  Cost is proportional to scanned bytes plus risky phrase matches, not number of
  artifact rows times scanned bytes.
- `s0_backend_inventory`
  Cost is proportional to declared backend inventory roots plus discovered
  backend declarations, not all source files.
- `s0_milestone_status_matrix_build`
  Cost is proportional to milestone docs plus closeout docs plus emitted rows,
  not all Worth docs.
- `s0_evidence_reference_resolution`
  Cost is proportional to unique evidence refs, not total references after
  duplicates.
- `s0_deferred_guarantee_validation`
  Cost is proportional to deferred guarantee rows plus referenced `S.*`
  sequence ids.
- `s0_artifact_schema_validation`
  Cost is proportional to artifact bytes plus row count.
- `s0_digest_construction`
  Cost is proportional to canonicalized row bytes, with sorting cost declared
  separately as `O(row_count log row_count)` unless rows arrive pre-sorted with
  proof.
- `s0_s1_handoff_validation`
  Cost is proportional to accepted input digests and blocking predicates, not
  raw source scan breadth.

Rules:

- each contract must name its input units and forbidden hidden breadth
- any `Debt` status must identify the exact unresolved breadth risk and the
  sequence or phase that must remove it
- S.0 closeout cannot mark the named suite complete if any required contract is
  missing
- `Debt` is allowed for exploratory tooling but not for required closeout lanes
- result envelopes must carry the complexity status for every required
  operation class

Naive trap this prevents:

- building a correct audit whose cost grows as `artifact rows * repo size`
  because each report rediscovers the same files and references.

### Audit Input Manifest Rule

S.0 must derive scan breadth once at the batch boundary and pass that summary to
all audit programs.

Required surfaces:

- `S0AuditInputManifest`
- `S0DeclaredScanRoot`
- `S0MatchedInputFile`
- `S0InputFileKind`
- `S0InputFileDigest`
- `S0InputManifestWitness`
- `S0InputManifestDelta`
- `S0ScanScopeRejection`

Rules:

- terminology scanning, milestone audit, backend inventory, evidence ref
  validation, and test migration notes must consume the same input manifest
  where their scopes overlap
- scan roots must be declared before traversal starts
- generated, target, vendor, and irrelevant workspace paths must be excluded by
  explicit scope rules rather than accidental omission
- file digests must be computed once per input file per run and reused by all
  downstream S.0 reports
- stale manifest use must fail typed when file digest, schema version, scan
  scope, or source revision changes
- S.0 may support incremental reruns, but an incremental result must publish
  which files were reused, rescanned, added, removed, or rejected

Naive trap this prevents:

- every S.0 sub-report walking the filesystem independently and producing
  slightly different ideas of what was audited.

### Capability-Tier Taxonomy Rule

Every backend and evidence lane must declare the strongest claim it is allowed
to make.

Required tiers:

- `Bootstrap`
  Useful for development, transition, or minimal persistence, but not allowed
  to satisfy semantic certification or physical foundation claims unless paired
  with separate proof.
- `SemanticCertification`
  Proves canonical authority, replay, retention, compatibility, subscription
  support, or other semantic Store behavior, but does not prove Roadmap 2
  physical database behavior.
- `Compatibility`
  Preserves old artifact or backend behavior so migration remains possible, but
  carries explicit forbidden claims for new platform-grade physical evidence.
- `PhysicalFoundation`
  Implements one or more Roadmap 2 physical substrate gates, with exact `S.*`
  capability rows naming which gates are satisfied.
- `PlatformGrade`
  Satisfies the declared Roadmap 2 foundation gates required for the platform
  claim being made, including certification evidence and hardware/backend
  assumptions.

Required surfaces:

- `StoreBackendCapabilityTier`
- `BackendCapabilityDeclaration`
- `BackendForbiddenClaim`
- `PhysicalFoundationCapabilityRow`
- `PlatformGradeReadinessClaim`
- `BackendCapabilityEvidence`

Rules:

- a backend may be valuable and non-platform-grade at the same time
- capability tiers are monotonic only through evidence, not naming
- a backend cannot claim `PlatformGrade` because it passes semantic replay or
  restart tests
- a backend that depends on whole-store heap materialization, full-object serde
  decode, or backend-local residue must not exceed `SemanticCertification` or
  `Compatibility` for physical claims
- `PhysicalFoundation` must name the exact `S.*` rows it satisfies; it is not a
  vague halfway tier

Naive trap this prevents:

- keeping an existing backend around for useful semantic tests, then letting
  demos, docs, or closeouts describe it as the physical database backend because
  no tier system says otherwise.

### Claim Promotion Typestate Rule

Backend and milestone claims must progress through proof-bearing states. A
platform-grade claim must not be constructible from raw strings, enum variants,
or documentation rows.

Required typestate surfaces:

- `UnclassifiedBackendClaim`
- `ClassifiedBackendClaim`
- `ForbiddenClaimAudited`
- `Roadmap2EvidenceBound`
- `PlatformGradeClaimAdmitted`
- `SemanticOnlyClaimWitness`
- `PhysicalDebtWitness`
- `FoundationEvidenceWitness`
- `PlatformGradeEvidenceWitness`
- `ClaimPromotionRejection`

Required promotion chain:

1. raw backend or milestone claim enters as `UnclassifiedBackendClaim`
2. S.0 classification produces `ClassifiedBackendClaim`
3. forbidden-claim audit produces `ForbiddenClaimAudited`
4. deferred guarantee mapping produces either `SemanticOnlyClaimWitness` or
   `PhysicalDebtWitness`
5. only closed Roadmap 2 evidence may produce `FoundationEvidenceWitness`
6. only the relevant set of foundation witnesses may produce
   `PlatformGradeEvidenceWitness`
7. only `PlatformGradeEvidenceWitness` may produce `PlatformGradeClaimAdmitted`

Rules:

- constructors for `PlatformGradeEvidenceWitness` and
  `PlatformGradeClaimAdmitted` must be sealed behind S.0 or later Roadmap 2
  certification authorities
- string-based labels such as `"platform-grade"` may be parsed only into raw
  unclassified claims, never directly into admitted claims
- a claim with any unmapped physical debt cannot progress past
  `PhysicalDebtWitness`
- a semantic-only claim may be strong, but it cannot be promoted by adding
  stronger wording
- compile-fail coverage must prove external callers cannot synthesize platform
  witnesses, skip forbidden-claim audit, or promote physical debt as
  platform-grade

Naive trap this prevents:

- creating a `PlatformGrade` enum variant and trusting callers to use it only
  when the right Roadmap 2 evidence exists.

### Concrete First-Audit Baseline Rule

S.0 must ship an initial classification baseline for the backend and evidence
families already known to exist.

Minimum required first-audit rows:

- `AbsentMode`
- `InMemoryHarness`
- `EmbeddedMode`
- `DurableMode`
- `LocalFileBackend`
- `SqliteBackend`
- `SemanticCertificationHarness`
- `SubscriptionSupportTrustEvidence`
- `Roadmap2PhysicalBackendCandidate`
- `FuturePlatformGradeBackend`

Each row must declare:

- capability tier
- valid use
- forbidden claims
- required evidence before promotion
- known semantic guarantees
- known physical gaps
- dependent Roadmap 2 sequences

Required first-audit posture:

- `AbsentMode` may prove optional-store boundaries, not physical persistence
- `InMemoryHarness` may prove semantic behavior, not durable survival
- `EmbeddedMode` may prove lifecycle and artifact reception semantics, not
  platform-grade physical database posture
- `DurableMode` may prove semantic durable-mode orchestration, not S.4 recovery
  physics until Roadmap 2 evidence exists
- `LocalFileBackend` and `SqliteBackend` remain non-platform-grade until they
  expose the relevant Roadmap 2 physical gates or are explicitly classified as
  compatibility paths
- `SubscriptionSupportTrustEvidence` is closed semantic trust evidence from
  Milestone 13.3, not physical database readiness
- `FuturePlatformGradeBackend` is a target row with required evidence, not a
  present claim

Rules:

- S.0 may add more rows, but it may not close with fewer than the required
  first-audit rows
- a row may be marked not present only if the inventory proves the family is
  absent from the current repo
- every row must include at least one forbidden claim unless the row is already
  admitted platform-grade by closed Roadmap 2 evidence

Naive trap this prevents:

- letting each implementer decide what "existing backend" means and quietly
  skipping the awkward paths that most need claim fences.

### Roadmap Sequence Consistency Rule

S.0 must audit roadmap dependency and status consistency, not only individual
milestone claims.

Required surfaces:

- `RoadmapSequenceStatusMatrix`
- `MilestoneStatusDeclaration`
- `MilestoneCloseoutStatus`
- `MilestonePrerequisiteEdge`
- `MilestoneSequenceInconsistency`
- `ClosedWithUnclosedPrerequisite`
- `SpecCloseoutStatusMismatch`
- `RoadmapGateReadinessWitness`

Required checks:

- every milestone with a closeout must reconcile closeout status with spec
  status
- every closed milestone must have each prerequisite closed, explicitly waived,
  or marked as intentionally out-of-order with a typed rationale
- every roadmap gate must name the exact predecessor evidence it consumes
- a milestone spec marked `Planned` with a closeout marked closed must produce a
  `SpecCloseoutStatusMismatch` until the status is reconciled or the closeout
  explains the divergence
- a later milestone closeout that depends on an unclosed prior milestone must
  produce `ClosedWithUnclosedPrerequisite` unless an explicit prerequisite waiver
  exists
- Roadmap 2 S.0 may consume closed semantic 13.3 evidence only after the
  sequence matrix explains the 13.1 -> 13.2 -> 13.3 status chain

Rules:

- sequence inconsistencies block S.0 closeout unless they are explicitly
  classified as semantic-only documentation drift with a remediation owner
- sequence waivers must be typed artifacts, not prose in a note
- S.1 handoff must include the roadmap gate readiness witness

Naive trap this prevents:

- treating a closeout file as authoritative while its prerequisite chain or spec
  status still says the work has not actually reached that state.

### Semantic-Versus-Physical Claim Rule

Every prior milestone claim must be classified by what it proves.

Required claim families:

- `SemanticAuthorityClaim`
- `RecoverySemanticsClaim`
- `RetentionSemanticsClaim`
- `SubscriptionSupportClaim`
- `CompatibilitySemanticsClaim`
- `TieringPlacementClaim`
- `ReplicationSemanticsClaim`
- `PhysicalSubstrateClaim`
- `PhysicalBoundednessClaim`
- `PhysicalIntegrityClaim`
- `PhysicalRecoveryPhysicsClaim`
- `PhysicalIsolationClaim`
- `PhysicalIoClaim`
- `PhysicalOperationalSafetyClaim`
- `PhysicalSecurityClaim`

Rules:

- a claim may belong to multiple families, but each family needs its own
  evidence status
- semantic replay equivalence is not physical recovery physics
- artifact digest equality is not page/frame/chunk integrity
- reopen from persisted records is not bounded-memory operation unless memory
  counters prove it
- restart correctness is not crash-harness evidence unless live heap state was
  discarded and recovery came from persisted bytes
- replication/capsule parity is not physical portability unless physical
  manifests, integrity, backend assumptions, and chunk behavior are included

Naive trap this prevents:

- treating one impressive test result as broad proof because the words
  "recovery", "durability", or "replication" sound physically complete.

### Prior-Milestone Physical Status Rule

Milestones 1 through 13.3 must each receive one physical-status row.

Required row fields:

- milestone id
- semantic capability proven
- closeout document or planned closeout source
- named suite from `test-requirements.md`
- backend or lanes used as evidence
- physical substrate status
- bounded-memory status
- physical integrity status
- recovery-physics status
- I/O/QoS status
- native blob/chunk status where applicable
- operator/security status where applicable
- forbidden claims
- deferred `S.*` dependencies
- required wording cleanup

Allowed physical status values:

- `NotApplicable`
- `NotStarted`
- `SemanticOnly`
- `BootstrapPhysical`
- `PhysicalDebt`
- `PartiallyFoundationBacked`
- `FoundationBacked`
- `PlatformGrade`

Rules:

- `PlatformGrade` is illegal before the relevant Roadmap 2 gates have
  machine-checkable evidence
- `BootstrapPhysical` must name why the path is useful but insufficient
- `PhysicalDebt` must map to at least one `S.*` sequence
- planned milestones such as 13.2 and 13.3 may have planned rows, but the row
  must still separate semantic intent from physical database posture

Naive trap this prevents:

- closing S.0 with a narrative paragraph instead of a mechanically auditable
  status matrix.

### Deferred Physical Guarantee Map Rule

Every physical guarantee deferred by S.0 must map to one or more Roadmap 2
sequences.

Minimum mapping categories:

- page/segment/extent substrate -> `S.1`
- memory and allocation boundedness -> `S.2`
- page/frame/chunk integrity and corruption localization -> `S.3`
- WAL/checkpoint/LSN recovery physics -> `S.4`
- physical read stability during maintenance -> `S.5`
- hardware-aware I/O and foreground QoS -> `S.6`
- native blob/object chunk store -> `S.7`
- index/layout/access-path discipline -> `S.8`
- formal crash/concurrency models -> `S.9`
- backup, PITR, offline verifier, repair, and forensics -> `S.10`
- security, tenant boundaries, keys, and auditability -> `S.11`
- physical database certification and performance -> `S.12`

Rules:

- no deferred guarantee may remain unmapped
- a guarantee may map to multiple `S.*` sequences when the physical behavior is
  cross-cutting
- deferred guarantees must be phrased as concrete missing proof, not vague
  future improvement
- any later milestone depending on a deferred guarantee must either wait for the
  relevant `S.*` closeout or name non-platform-grade debt explicitly

Naive trap this prevents:

- saying "we need a real database foundation later" without pinning each missing
  guarantee to the sequence that will actually earn it.

### Terminology Cleanup Rule

S.0 must make overbroad language mechanically visible.

Required terminology audit targets:

- `production-grade`
- `platform-grade`
- `database`
- `embedded backend`
- `WAL`
- `crash recovery`
- `durability`
- `physical`
- `blob`
- `replication`
- `certification`
- `bounded`
- `integrity`
- `repair`

Required surfaces:

- `TerminologyRiskReport`
- `OverclaimedPhysicalPosture`
- `AllowedSemanticPhrase`
- `RequiredPhysicalQualifier`
- `RoadmapCleanupPatch`

Rules:

- wording cleanup must preserve valid semantic claims rather than weakening
  them lazily
- if a phrase is valid only semantically, it must say so
- if a phrase is physical debt, it must name the `S.*` sequence that owns the
  future proof
- "production-grade embedded backend" may refer only to the semantic embedded
  contract until Roadmap 2 physical gates close; platform-grade physical
  readiness requires explicit qualification

Naive trap this prevents:

- replacing precise overclaims with vague underclaims and accidentally erasing
  the value of Roadmap 1 instead of clarifying its evidence boundary.

### Mechanical Terminology Scanner Rule

S.0 terminology cleanup must be enforced by a deterministic scanner, not by
human memory.

Required surfaces:

- `TerminologyScanPlan`
- `TerminologyScanScope`
- `TerminologyPhraseFinding`
- `TerminologyAllowedUse`
- `TerminologyRequiredQualifier`
- `TerminologyCleanupRejection`
- `TerminologyScanDigest`

Required scan scopes:

- `plans/worth-store/runtime-integration-roadmap.md`
- `plans/worth-store/physical-database-roadmap.md`
- `plans/worth-store/milestone-*.md`
- `plans/worth-store/*-closeout.md`
- `plans/worth-store/test-requirements.md`
- `plans/worth-store/test-requirements-2.md`
- `crates/worth-store/src/**/*.rs`
- `crates/worth-store/tests/**/*.rs`

Rules:

- every risky phrase occurrence must classify as `AllowedSemanticUse`,
  `QualifiedPhysicalDebt`, `ClosedFoundationEvidence`, or
  `OverclaimedPhysicalPosture`
- `OverclaimedPhysicalPosture` findings block S.0 closeout
- `QualifiedPhysicalDebt` findings must name the exact `S.*` sequence that owns
  the missing proof
- `ClosedFoundationEvidence` findings must reference the evidence bundle that
  admits the claim
- allowlist entries must be line-scoped or symbol-scoped, not file-wide
- adding a new risky phrase without a classification must change the
  terminology scan digest and fail the suite

Naive trap this prevents:

- doing one cleanup pass, then reintroducing unqualified "database-grade" or
  "production-grade" language in the next spec or code comment.

### Scanner And Evidence Resolution Budget Rule

S.0 scanners and evidence resolvers must reject or mark debt before they broaden
into expensive global work.

Required surfaces:

- `S0ScanBudget`
- `S0ScanBudgetClass`
- `S0EvidenceResolutionBudget`
- `S0UniqueEvidenceRefIndex`
- `S0EvidenceRefResolutionReceipt`
- `S0BroadScanRejection`
- `S0ResolutionDebt`

Budget classes:

- `DeclaredScope`
- `MilestoneLocal`
- `WorthStoreDocs`
- `WorthStoreSource`
- `Roadmap2Harness`
- `RejectedWorkspaceGlobal`

Rules:

- no required S.0 closeout lane may use `RejectedWorkspaceGlobal`
- evidence refs must be canonicalized and deduplicated before resolution
- resolving the same evidence ref twice inside one S.0 run is a performance
  defect unless a receipt proves reuse was impossible across a trust boundary
- terminology scanning may not broaden from Worth Store docs/source into all
  crates unless the broadened scope is explicit, countered, and marked debt or
  separately admitted
- broad scan attempts must produce `S0BroadScanRejection` instead of silently
  succeeding with high cost
- result bundles must report requested scan scope, admitted scan scope, rejected
  scan scope, scanned file count, scanned byte count, unique evidence ref count,
  and reused receipt count

Naive trap this prevents:

- "just scan everything" becoming the hidden implementation strategy for every
  future S.0 closeout run.

### Harness Maturity Classification Rule

S.0 must classify the Roadmap 2 harness requirements before S.1 consumes them.

Required surfaces:

- `HarnessSubsystemMaturity`
- `HarnessCoverageMatrix`
- `SequenceHarnessDependency`
- `EvidenceBundleReadiness`
- `ForbiddenShortcutDetectionStatus`

Maturity levels:

- `Missing`
- `Exists`
- `SmokeWorks`
- `CiCertifiable`
- `ReleaseCertifiable`

Rules:

- S.0 does not need every harness subsystem to be `CiCertifiable`
- S.0 must identify which harness pieces are required for S.1 and whether they
  exist
- missing harness pieces must become explicit prerequisites or debt for S.1
  planning, not invisible test optimism
- forbidden-shortcut detection must exist for S.0's own suite:
  overclaim detection, backend tier mismatch, unmapped deferred guarantee,
  and missing physical-status row

Naive trap this prevents:

- starting S.1 implementation with no evidence system capable of proving the
  implementation is actually using the new physical substrate.

### S.1 Handoff Gate Rule

S.0 must emit a handoff artifact that S.1 treats as an input gate, not
background reading.

Before S.0 may consume the aspect-native gate output, the input must be a typed
`S0AspectNativeGateHandoff` built from `StoreS0ReadinessHandoffArtifact`.
That artifact carries a native canonical basis, completed boundary receipts,
diagnostic support reports, and counter-backed performance receipts. Terminal
JSON projection output, unclassified residue, raw string identity, generic
serde authority, and non-native digest basis are not handoff inputs; each must
be denied by `S0HandoffGateProofEvidence`. That evidence is produced from the
current JSON residue scan, terminal projection boundary classification,
Foundational adoption map, public facade proof, and native harness proof rather
than from logs, markdown, terminal JSON, or symbolic labels.

Required handoff fields:

- accepted backend tier matrix digest
- accepted deferred guarantee map digest
- accepted terminology scan digest
- accepted S.0 audit input manifest digest
- accepted S.0 complexity contract summary digest
- required S.1 forbidden shortcuts
- required S.1 harness subsystems and maturity levels
- S.1 allowed backend candidates
- legacy backend fences S.1 must preserve
- S.1 compile-fail fixtures required by S.0
- S.1 non-platform-grade debt rows that remain legal only as debt

Required S.1 blocking predicates:

- no backend tier matrix -> S.1 cannot close
- no deferred guarantee map -> S.1 cannot close
- no terminology scan digest -> S.1 cannot close
- no forbidden shortcut list -> S.1 cannot close
- no S.1 harness readiness rows -> S.1 cannot close
- any S.0 `OverclaimedPhysicalPosture` -> S.1 cannot close
- any unmapped physical guarantee -> S.1 cannot close

Rules:

- S.1 may begin exploratory implementation before S.0 closeout, but S.1 may not
  close or claim platform-grade substrate evidence without consuming the
  S.0 handoff artifact
- S.1 must report the S.0 handoff digest in its evidence bundle
- S.1 must report whether S.0 closeout lanes were `Verified` or carried
  performance debt
- if S.1 changes a backend tier or deferred guarantee, it must update the S.0
  artifacts or produce a typed stale-handoff rejection
- if S.1 broadens any inherited scan scope or evidence resolution scope, it must
  publish a new complexity contract rather than silently inheriting S.0's
  verified cost posture

Naive trap this prevents:

- treating S.0 as a planning document and then letting S.1 implementation drift
  away from the audit it was supposed to consume.

### Release And CI Claim Gate Rule

S.0 must fence public-facing and automation-facing claims, not only engineering
spec prose.

Required surfaces:

- `ReleaseClaimScanPlan`
- `CiClaimGate`
- `PackageMetadataClaim`
- `ReadmeBadgeClaim`
- `ChangelogClaim`
- `ReleaseNoteClaim`
- `AutomationPlatformClaim`
- `PublicClaimRejection`

Required scan scopes:

- `.github/**`
- `Cargo.toml`
- `crates/worth-store/Cargo.toml`
- root `README*` files if present
- crate-level `README*` files if present
- changelog or release-note files if present
- packaging, publish, or release scripts if present

Rules:

- no CI workflow, release script, package metadata, README, badge, changelog, or
  release note may claim beta, production, platform-grade, physical database,
  financial-platform, aerospace-grade, or database-ready posture without an
  admitted platform-grade evidence witness
- semantic claims may appear only with qualifiers that bind them to the semantic
  guarantee family
- release claim checks must run from canonical S.0 terminology and claim data,
  not a second hand-maintained word list
- hostile lanes must prove a public-facing overclaim is rejected before release
  or publication metadata is treated as valid

Naive trap this prevents:

- keeping the specs honest while the package metadata, README, badge, or CI
  release job tells users the store is already a production database.

### Evidence Staleness And Provenance Rule

S.0 evidence must become stale when its source inputs change. A stale S.0
artifact is not a soft warning; it is an invalid handoff input.

Required surfaces:

- `S0EvidenceProvenance`
- `S0SourceRevision`
- `S0InputDigestSet`
- `S0ArtifactStalenessReport`
- `S0StaleEvidenceRejection`
- `S0RegenerationRequirement`
- `S0AcceptedEvidenceBundleWitness`

Rules:

- every canonical S.0 artifact must record the exact source revision and input
  manifest digest used to generate it
- changing any input file included by the S.0 input manifest invalidates the
  accepted evidence bundle unless an incremental rerun updates the affected
  artifact and digest chain
- S.1 handoff construction must reject stale S.0 artifacts
- generated S.0 artifacts may not be edited manually after generation unless the
  edit is represented as a new generated artifact with an updated provenance
  record
- provenance must distinguish generated artifacts, hand-authored specs,
  closeout evidence, and source code inputs
- S.0 closeout must publish a regeneration command or equivalent reproducible
  procedure for rebuilding the evidence bundle from source inputs

Naive trap this prevents:

- generating a clean S.0 bundle once, then continuing to change Roadmap 1,
  Roadmap 2, tests, or code while S.1 consumes stale evidence.

### Compile-Time Boundary Rule

The highest-risk S.0 conclusions must be represented in code with sealed
constructors and compile-fail tests.

Required compile-time boundaries:

- external code cannot construct `PlatformGradeClaimAdmitted`
- external code cannot construct `PlatformGradeEvidenceWitness`
- external code cannot promote `PhysicalDebtWitness` into platform evidence
- external code cannot build a backend capability declaration without a tier
- external code cannot omit forbidden claims from a non-platform-grade backend
  row
- external code cannot build a milestone physical-status row without deferred
  `S.*` mappings for physical debt
- external code cannot build an S.1 handoff without accepted matrix, deferred
  map, terminology scan, and harness maturity digests

Required trybuild-style fixtures:

- `s0_platform_grade_claim_constructor_private.rs`
- `s0_platform_grade_evidence_witness_constructor_private.rs`
- `s0_physical_debt_cannot_promote_to_platform.rs`
- `s0_backend_declaration_requires_tier.rs`
- `s0_non_platform_backend_requires_forbidden_claims.rs`
- `s0_physical_debt_requires_s_sequence_mapping.rs`
- `s0_s1_handoff_requires_accepted_digests.rs`

Rules:

- this spec document may be approved before the fixtures exist, but the S.0
  sequence may not close until the compile-time boundary fixtures exist and pass
- S.1 may not close if any S.0 compile-time boundary fixture is missing,
  ignored, or marked debt
- runtime validation may supplement these boundaries, but it may not replace
  sealed construction for platform-grade promotion

Naive trap this prevents:

- making the audit conceptually correct while leaving future code free to
  synthesize the exact forbidden states the audit was designed to prevent.

### Implementation Decomposition Rule

S.0 implementation must be decomposed by audit responsibility. It must not land
as one broad `s0.rs`, one "audit helper", or a doc-only script with private
semantics.

Required production module shape:

- `storage_foundation/s0/capability/`
  backend tiers, forbidden claims, capability evidence, and promotion gates
- `storage_foundation/s0/claims/`
  semantic-versus-physical claim families and claim classification
- `storage_foundation/s0/milestones/`
  prior milestone status rows, closeout references, and status matrix digest
- `storage_foundation/s0/deferred/`
  deferred physical guarantee map and `S.*` dependency validation
- `storage_foundation/s0/terminology/`
  scanner plans, findings, allowlist classifications, and cleanup reports
- `storage_foundation/s0/harness/`
  harness maturity rows and S.1-required harness readiness
- `storage_foundation/s0/handoff/`
  S.1 handoff construction and stale-handoff rejection
- `storage_foundation/s0/evidence/`
  canonical artifact envelopes, evidence bundles, schema validation, and
  deterministic digests
- `storage_foundation/s0/counters/`
  exact S.0 counter snapshot construction

Required test shape:

- focused unit tests beside each subdomain
- suite-level S.0 certification tests that consume only public or facade
  surfaces
- trybuild fixtures for sealed witness and typestate boundaries
- scanner regression tests with both allowed and rejected terminology examples
- stale digest and missing artifact negative lanes

Rules:

- terminology scanning may use filesystem traversal, but classification logic
  belongs in the terminology subdomain, not in shell scripts
- digest construction belongs in the evidence subdomain, not repeated ad hoc in
  tests
- S.1 handoff validation must consume public S.0 evidence types, not private
  parser internals
- each subdomain must have a single reason to change and its own failure tests

Naive trap this prevents:

- implementing S.0 as an impressive one-off audit generator that cannot become
  a stable Store subsystem or be reused by S.1 through S.12.

### S.0 Counter Contract

Required counters:

- `s0_required_artifact_count`
- `s0_missing_required_artifact_count`
- `s0_schema_incompatible_artifact_count`
- `s0_complexity_contract_count`
- `s0_missing_complexity_contract_count`
- `s0_complexity_debt_count`
- `s0_input_manifest_file_count`
- `s0_input_manifest_byte_count`
- `s0_input_manifest_reused_file_count`
- `s0_input_manifest_rescanned_file_count`
- `s0_backend_classification_count`
- `s0_backend_forbidden_claim_count`
- `s0_required_first_audit_row_count`
- `s0_missing_first_audit_row_count`
- `s0_roadmap_sequence_edge_count`
- `s0_sequence_inconsistency_count`
- `s0_unwaived_sequence_inconsistency_count`
- `s0_spec_closeout_status_mismatch_count`
- `s0_closed_with_unclosed_prerequisite_count`
- `s0_milestone_status_row_count`
- `s0_missing_milestone_status_row_count`
- `s0_semantic_claim_count`
- `s0_physical_claim_count`
- `s0_overclaimed_physical_phrase_count`
- `s0_terminology_phrase_finding_count`
- `s0_unclassified_terminology_finding_count`
- `s0_requested_scan_scope_count`
- `s0_admitted_scan_scope_count`
- `s0_rejected_scan_scope_count`
- `s0_scanned_file_count`
- `s0_scanned_byte_count`
- `s0_broad_scan_rejection_count`
- `s0_release_claim_scan_count`
- `s0_public_claim_rejection_count`
- `s0_unqualified_release_claim_count`
- `s0_unique_evidence_ref_count`
- `s0_duplicate_evidence_ref_count`
- `s0_evidence_ref_receipt_reuse_count`
- `s0_evidence_ref_reresolution_count`
- `s0_digest_canonicalized_row_byte_count`
- `s0_digest_sort_row_count`
- `s0_stale_evidence_rejection_count`
- `s0_manual_artifact_edit_rejection_count`
- `s0_regeneration_requirement_count`
- `s0_cleanup_patch_count`
- `s0_deferred_guarantee_count`
- `s0_unmapped_deferred_guarantee_count`
- `s0_test_migration_note_count`
- `s0_harness_maturity_row_count`
- `s0_s1_blocking_prerequisite_count`
- `s0_s1_unmet_blocking_prerequisite_count`
- `s0_compile_time_boundary_count`
- `s0_missing_compile_time_boundary_count`
- `s0_platform_grade_claim_rejection_count`
- `s0_exact_status_matrix_digest_count`

Required zero assertions:

- `s0_missing_required_artifact_count == 0`
- `s0_schema_incompatible_artifact_count == 0`
- `s0_missing_complexity_contract_count == 0`
- `s0_complexity_debt_count == 0` in required closeout lanes
- `s0_missing_first_audit_row_count == 0`
- `s0_unwaived_sequence_inconsistency_count == 0`
- `s0_unqualified_release_claim_count == 0`
- `s0_missing_milestone_status_row_count == 0`
- `s0_unmapped_deferred_guarantee_count == 0`
- `s0_unclassified_terminology_finding_count == 0`
- `s0_rejected_scan_scope_count` matches hostile broad-scan lanes exactly
- `s0_evidence_ref_reresolution_count == 0` inside one trusted S.0 run
- `s0_s1_unmet_blocking_prerequisite_count == 0`
- `s0_platform_grade_claim_rejection_count` matches hostile overclaim lanes
- `s0_overclaimed_physical_phrase_count == 0` after accepted cleanup patches

Rules:

- counters must be emitted in the S.0 evidence bundle
- the status matrix digest must change if a milestone row, backend tier,
  deferred guarantee, or forbidden claim changes
- S.0 closeout may not rely on human review of prose alone

## Phases

### Phase 1: Establish Claim Vocabulary And Backend Capability Tiers

Phase 1 defines the language that keeps the audit honest.

Required work:

- define the canonical S.0 artifact directory and required typed
  aspect-native artifact set
- define the named complexity contracts and required cost surfaces
- define the audit input manifest and scan budget vocabulary
- define capability tiers for existing and future backends
- define the claim promotion typestate chain
- define roadmap sequence consistency and release/public claim gates
- define semantic and physical claim families
- define physical-status row fields and allowed values
- define forbidden platform-grade claim rules
- define the S.0 evidence bundle shape
- define the first deferred physical guarantee categories
- define exact S.0 counters and zero assertions

Exit condition:

- an existing backend, milestone, test lane, or roadmap phrase can be classified
  without inventing new vocabulary during the audit.

### Phase 2: Inventory Existing Backends, Modes, And Evidence Lanes

Phase 2 classifies the actual implementation and test evidence the first
roadmap has been using.

Required work:

- inventory durable, embedded, absent, local-file, SQLite, in-memory, harness,
  compatibility, and any bootstrap persistence paths currently present
- build the shared audit input manifest before backend and evidence inventory
  consumes repository inputs
- populate all required first-audit baseline rows or mark absent rows with
  inventory evidence
- build the roadmap sequence status matrix and identify prerequisite/status
  inconsistencies
- classify each backend by capability tier
- record which named suites and closeouts use which backend or lane
- identify where evidence depends on heap state, full-object serde decode,
  backend-local residue, broad scans, or missing resource counters
- produce forbidden-claim rows for every backend that must not be used as
  platform-grade physical evidence
- preserve valid semantic evidence rows for existing closeouts
- emit backend inventory cost surfaces and mark any broadening as debt or
  rejection

Exit condition:

- every existing Store backend and evidence lane has a tier, a valid-use
  statement, and a forbidden-claim list.

### Phase 3: Audit Roadmap 1 Milestones And Closeouts Through 13.3

Phase 3 produces the status matrix that makes backtracking precise.

Required work:

- create one physical-status row for each Roadmap 1 milestone from `Milestone 1`
  through `Milestone 13.3`
- consume the shared audit input manifest rather than rewalking all docs
- include closeout/planned-closeout references where they exist
- classify each milestone's valid semantic guarantees
- classify each milestone's unproven physical guarantees
- map each deferred guarantee to `S.1` through `S.12`
- mark planned or absent closeout material explicitly instead of inferring
  completion
- identify milestone language that may overstate physical database posture
- emit milestone-matrix build complexity status and exact row/digest counters
- emit sequence consistency reports for spec/closeout status mismatch,
  unclosed prerequisites, and typed waivers

Exit condition:

- no Roadmap 1 milestone can be read as platform-grade physical proof unless
  the row explicitly names Roadmap 2 evidence supporting that claim.

### Phase 4: Clean Roadmap And Test Language Without Weakening Earned Semantics

Phase 4 patches documentation so the roadmap teaches the correct claim
boundary.

Required work:

- update Roadmap 1 language that implies platform-grade physical behavior before
  Roadmap 2 closes
- update Roadmap 2 S.0 references to point at this spec
- implement the deterministic terminology scanner and line-scoped allowlist
- enforce scanner budgets and reject workspace-global broadening
- run release and CI claim scanning over workflow, package, README, changelog,
  and release surfaces
- update test-requirement wording where a suite proves semantic parity but not
  physical database behavior
- add migration notes for closeouts whose proof is semantic-only or
  bootstrap-physical
- keep earned semantic claims explicit instead of replacing them with vague
  caveats
- emit terminology cleanup counters and status matrix digest updates
- emit scanned file/byte counters and evidence-ref receipt reuse counters

Exit condition:

- the roadmap can be read without implying that heap/file/SQLite bootstrap paths
  already satisfy the physical database foundation.

### Phase 5: Define S.1 Handoff Inputs And Harness Readiness

Phase 5 makes S.0 useful to the implementation that follows.

Required work:

- produce the deferred physical guarantee map consumed by S.1 and later
  sequences
- produce the backend tier matrix consumed by S.1 legacy capability fences
- produce the S.1 handoff readiness artifact with accepted input digests and
  blocking predicates
- include S.0 complexity contract and audit input manifest digests in the S.1
  handoff
- include roadmap gate readiness and accepted evidence provenance in the S.1
  handoff
- identify which harness subsystems S.1 requires and their maturity status
- identify any forbidden-shortcut detectors that must exist before S.1 closeout
- define compile-time boundary fixtures required before S.1 may close
- define non-platform-grade debt language for any post-13.3 work that proceeds
  before Roadmap 2 closes
- publish S.1 entry criteria in the S.0 evidence bundle
- publish which S.0 performance contracts are verified and prove required
  closeout lanes carry no performance debt

Exit condition:

- S.1 can start with known backend tiers, known forbidden shortcuts, known
  deferred guarantees, known harness gaps, required compile-time boundaries,
  and a digestable handoff artifact.

### Phase 6: Prove Shipped Store Capability Reclassification

Phase 6 closes S.0 with machine-checkable evidence.

Required work:

- run the Roadmap 2 named suite:
  `Shipped store capability reclassification test`
- parse and validate every required canonical S.0 artifact
- validate every required S.0 complexity contract
- prove one shared input manifest feeds overlapping audit scopes
- prove stale evidence and manual artifact edits are rejected
- prove evidence refs are deduplicated and resolved by reusable receipts
- emit capability-tier matrix
- emit semantic-versus-physical claim report
- emit deferred physical guarantee map
- emit terminology cleanup report
- emit test migration notes
- emit harness maturity report
- emit exact counter snapshot
- emit complexity and cost-surface summaries
- prove every prior milestone has a physical-status row
- prove every required first-audit baseline row exists or is explicitly absent
  with inventory evidence
- prove roadmap sequence status is consistent or explicitly waived with typed
  rationale
- prove every deferred physical guarantee maps to an `S.*` sequence
- prove unqualified public/release claims are rejected
- prove hostile platform-grade overclaims are rejected typed
- prove S.1 blocking predicates reject stale, missing, or incomplete handoff
  inputs
- prove compile-time boundary fixtures exist or are marked as S.0
  implementation debt that blocks S.1 closeout

Exit condition:

- S.0 closeout evidence can be checked mechanically, and the project has a
  clean foundation for S.1 rather than a pile of ambiguous prior claims.

## Must Ship

- canonical S.0 artifact directory:
  `plans/worth-store/artifacts/storage-foundation-s0/`
- required typed aspect-native artifacts for backend tiers, milestone status,
  claim reports, deferred guarantees, terminology findings, test migration
  notes, harness maturity, S.1 handoff readiness, and the S.0 evidence bundle
- common artifact envelope schema and stable row schema for all matrix-style
  artifacts
- named S.0 complexity contracts for input manifest construction, terminology
  scanning, backend inventory, milestone matrix building, evidence resolution,
  deferred guarantee validation, artifact validation, digest construction, and
  S.1 handoff validation
- shared audit input manifest with file digests, scan roots, matched files,
  incremental delta posture, and stale-manifest rejection
- scan and evidence-resolution budgets with broad-scan rejection and reusable
  evidence-ref receipts
- implementation decomposition by capability, claims, milestones, deferred
  guarantees, terminology, harness, handoff, evidence, and counters
- backend capability-tier taxonomy
- capability-tier matrix for existing Store backends and evidence lanes
- first-audit baseline rows for absent, in-memory, embedded, durable,
  local-file, SQLite, semantic-harness, subscription-support-trust,
  physical-backend-candidate, and future-platform backend families
- roadmap sequence status matrix that reconciles spec status, closeout status,
  prerequisite edges, waivers, and gate readiness
- proof-bearing claim promotion typestate from raw claims to admitted
  platform-grade claims
- semantic-versus-physical claim family taxonomy
- physical-status matrix for Roadmap 1 milestones through `Milestone 13.3`
- explicit list of semantic guarantees already earned
- explicit list of physical guarantees deferred to Roadmap 2
- deferred physical guarantee map from each missing guarantee to `S.1` through
  `S.12`
- deterministic terminology scanner with line-scoped or symbol-scoped
  classifications for risky physical language
- release and CI claim gate over workflows, package metadata, README, changelog,
  release, badge, and automation surfaces
- evidence staleness and provenance model with source revision, input digest
  set, stale-evidence rejection, and reproducible regeneration procedure
- terminology risk report and cleanup patches for overbroad physical language
- migration notes for tests and closeouts that prove semantic parity but not
  physical boundedness or database-grade physical behavior
- Roadmap 2 harness maturity report
- S.1 handoff inputs:
  - backend tiers
  - forbidden shortcut list
  - deferred guarantee rows
  - harness readiness rows
  - terminology scan digest
  - audit input manifest digest
  - complexity contract summary digest
  - roadmap gate readiness witness
  - accepted evidence provenance
  - compile-time boundary fixture list
  - entry criteria
- compile-time boundary plan or fixtures for platform-grade claim admission,
  physical-debt promotion rejection, backend declaration completeness,
  milestone row completeness, and S.1 handoff completeness
- exact S.0 counters and status-matrix digest
- exact S.0 cost counters for scanned files, scanned bytes, unique evidence
  refs, receipt reuse, digest row bytes, and broad-scan rejection
- machine-checkable S.0 certification bundle

## Must Preserve

- canonical commit envelopes remain semantic authority
- `worth-relational` remains owner of truth semantics, transaction semantics,
  branch meaning, MVCC, identity, and lineage semantics
- existing Roadmap 1 semantic work remains valid unless the S.0 audit finds an
  actual contradiction
- useful bootstrap, compatibility, and semantic-certification backends remain
  allowed under honest tiers
- S.0 does not rename physical debt into semantic failure
- S.0 does not implement S.1 through S.12 work prematurely
- later roadmap work may name non-platform-grade debt, but may not hide it
- platform-grade promotion remains sealed behind evidence witnesses rather than
  public enum construction
- S.1 closeout must consume the S.0 handoff artifact instead of rediscovering
  audit conclusions
- S.0 audit tooling remains bounded by declared scan scopes and evidence-ref
  receipts; "scan the whole workspace" is not an accepted closeout strategy
- S.0 evidence must be current for the source revision and input digest set S.1
  consumes
- public release and automation surfaces may not outrun the admitted claim tier

## Acceptance Evidence

S.0 is complete only when the store satisfies the Roadmap 2 named suite:

- `Shipped store capability reclassification test`

Required machine-checkable outputs:

- `capability_tier_matrix`
- `semantic_physical_claim_report`
- `deferred_physical_guarantee_map`
- `terminology_cleanup_report`
- `test_migration_note_report`
- `harness_maturity_report`
- `s1_handoff_readiness`
- `complexity_contract_summary`
- `audit_input_manifest`
- `roadmap_sequence_status_matrix`
- `release_claim_report`
- `evidence_provenance_report`
- `counter_snapshot`
- `failure_digest`

Minimum certification matrix rows:

- `all_existing_backends_classified`
  Every backend and evidence lane has exactly one declared tier and a valid-use
  statement.
- `canonical_artifact_set_aspect_native`
  Every required S.0 aspect-native artifact exists, admits through typed
  Foundational value/identity/locator/canonical-basis surfaces, matches schema
  version, rejects JSON-shaped substitutes, and has a deterministic digest.
- `canonical_artifact_rows_stable`
  Matrix rows have stable row ids, local evidence references, sortable order,
  and no digest dependence on nondeterministic metadata.
- `complexity_contracts_verified`
  Every required S.0 operation class has a complexity contract, verified status
  in required closeout lanes, and exact cost counters.
- `audit_input_manifest_reused`
  Terminology scan, milestone audit, backend inventory, evidence resolution,
  and test migration notes consume the shared input manifest where scopes
  overlap.
- `evidence_refs_deduplicated`
  Duplicate evidence references resolve through reusable receipts rather than
  repeated filesystem or parser work.
- `workspace_global_scan_rejected`
  A hostile lane requests workspace-global scanning and receives
  `S0BroadScanRejection`.
- `digest_cost_surface_exact`
  Digest construction reports canonicalized row bytes and sort row counts, and
  does not allocate or sort outside the declared contract.
- `s0_module_decomposition_enforced`
  S.0 implementation surfaces are split by capability, claims, milestone
  status, deferred guarantees, terminology, harness, handoff, evidence, and
  counters rather than one audit mega-module.
- `first_audit_baseline_complete`
  All required first-audit backend and evidence-family rows are present or
  explicitly absent with inventory evidence.
- `roadmap_sequence_consistency_verified`
  Spec status, closeout status, prerequisite edges, and gate readiness are
  consistent or explicitly waived with typed rationale.
- `spec_closeout_status_mismatch_rejected`
  A milestone spec marked planned while its closeout claims closed status
  produces a blocking mismatch unless reconciled or waived.
- `closed_with_unclosed_prerequisite_rejected`
  A closed milestone depending on an unclosed prerequisite produces a blocking
  inconsistency unless an explicit waiver exists.
- `release_claim_gate_rejects_overclaim`
  CI, package, README, changelog, badge, release, or automation surfaces cannot
  publish unqualified platform-grade claims without evidence witnesses.
- `stale_evidence_rejected`
  Changing an input file or manually editing a generated artifact causes S.0
  handoff construction to reject stale evidence.
- `legacy_heap_paths_not_platform_grade`
  Heap-shaped, full-object, or in-memory bootstrap paths are rejected for
  platform-grade physical claims.
- `file_sqlite_paths_claim_fenced`
  File and SQLite paths are classified by the physical gates they actually
  satisfy and forbidden from claiming unearned Roadmap 2 posture.
- `milestones_1_through_13_3_status_rows_complete`
  Every Roadmap 1 milestone through `13.3` has a physical-status row.
- `semantic_guarantees_preserved`
  Existing semantic guarantees remain listed and are not weakened by physical
  reclassification.
- `physical_guarantees_deferred`
  Missing physical database guarantees are listed as deferred work rather than
  implied completion.
- `deferred_guarantees_map_to_s_sequences`
  Every deferred guarantee maps to at least one `S.*` sequence.
- `production_grade_language_qualified`
  Phrases such as "production-grade embedded backend" are either qualified as
  semantic claims or mapped to Roadmap 2 physical debt.
- `terminology_scanner_rejects_unclassified_phrase`
  Adding a risky phrase without line-scoped or symbol-scoped classification
  changes the scan digest and fails the suite.
- `platform_grade_overclaim_rejected`
  A hostile lane attempts to mark an unqualified backend or milestone
  platform-grade and receives a typed rejection.
- `platform_grade_claim_unconstructable_without_witness`
  Platform-grade claim admission cannot be constructed without sealed Roadmap 2
  evidence witnesses.
- `physical_debt_cannot_promote_to_platform`
  A claim carrying physical debt cannot be promoted into platform evidence.
- `test_evidence_scope_declared`
  Each existing named suite is marked as semantic, bootstrap-physical,
  foundation-backed, or platform-grade according to its actual evidence.
- `harness_maturity_rows_present`
  Required Roadmap 2 harness subsystems have maturity rows and S.1-required
  gaps are visible.
- `s1_handoff_blocks_missing_inputs`
  S.1 handoff construction rejects missing matrix, deferred map, terminology
  scan, harness maturity, or forbidden shortcut inputs.
- `s1_handoff_carries_cost_posture`
  S.1 handoff includes the accepted audit input manifest digest and complexity
  contract summary digest.
- `status_matrix_digest_changes_on_claim_change`
  Changing a backend tier, milestone row, forbidden claim, or deferred guarantee
  changes the matrix digest.
- `unmapped_deferred_guarantee_forbidden`
  A deferred physical guarantee without an `S.*` mapping fails the suite.
- `missing_milestone_row_forbidden`
  Removing one required milestone status row fails the suite.
- `roadmap_readability_gate`
  The first roadmap can be read without implying that bootstrap persistence is
  already the platform-grade physical database substrate.

Milestone-specific proof obligations:

- every prior milestone has a physical-status row
- every backend has a capability tier and forbidden-claim list
- every deferred physical guarantee maps to `S.1` through `S.12`
- overbroad physical terminology is either cleaned up or explicitly qualified
- S.1 has concrete handoff inputs from S.0
- S.1 handoff includes accepted digests for backend tiers, deferred guarantees,
  terminology scan, audit input manifest, complexity contract summary, harness
  maturity, roadmap gate readiness, evidence provenance, and forbidden shortcuts
- roadmap sequence status is consistent or typed-waived before S.0 closeout
- release and CI claim surfaces are fenced by the same platform-grade evidence
  witness model as code and docs
- stale evidence cannot be consumed by S.1 handoff
- every required S.0 operation has a named complexity contract and verified
  closeout status
- shared input manifest prevents repeated filesystem rediscovery across
  overlapping audit programs
- evidence reference receipts prevent repeated reference resolution inside one
  trusted S.0 run
- broad scan requests are rejected or explicitly marked outside required
  closeout lanes
- semantic guarantees remain visible and valuable
- platform-grade claims require Roadmap 2 evidence rather than semantic
  closeout evidence
- platform-grade claim construction is sealed behind evidence witnesses
- terminology cleanup is scanner-enforced, not grep-and-memory enforced
- S.0 evidence is machine-checkable and not a prose-only audit
- `s0_missing_required_artifact_count` remains zero
- `s0_missing_complexity_contract_count` remains zero
- `s0_complexity_debt_count` remains zero in required closeout lanes
- `s0_missing_first_audit_row_count` remains zero
- `s0_unwaived_sequence_inconsistency_count` remains zero
- `s0_unqualified_release_claim_count` remains zero
- `s0_missing_milestone_status_row_count` remains zero
- `s0_unmapped_deferred_guarantee_count` remains zero
- `s0_unclassified_terminology_finding_count` remains zero
- `s0_evidence_ref_reresolution_count` remains zero inside one trusted S.0 run
- `s0_s1_unmet_blocking_prerequisite_count` remains zero
- `s0_overclaimed_physical_phrase_count` remains zero after cleanup

S.0 is not closed by "we know what we meant" or by a narrative audit that cannot
be checked against exact backend, milestone, claim, and deferred-guarantee rows.

## Architectural Notes

- The smart abstraction is not "legacy versus new backend." The smart
  abstraction is a claim boundary: what evidence does this path actually prove?
- S.0 should be slightly uncomfortable. If no language changes, no forbidden
  claims, and no deferred guarantees are found, the audit probably failed to be
  adversarial enough.
- Do not punish Roadmap 1 for being semantic. The mistake would be pretending
  semantic persistence evidence is the same as database substrate evidence.
- S.0's status matrix should become living input for S.1 through S.12 rather
  than a one-time spreadsheet-shaped artifact.
- The capability-tier vocabulary should appear in code and tests where possible
  so unsupported platform claims fail mechanically.

## Sequencing Notes

S.0 belongs immediately after the `13.x` semantic cleanup arc and immediately
before S.1.

- It follows closed `Milestone 13.3` because subscription-support accuracy and
  certification is the last semantic support cleanup before platform expansion,
  and its closeout explicitly hands off physical database readiness to Roadmap
  2.
- It precedes S.1 because page, segment, extent, manifest, and physical-reference
  work needs known backend tiers and forbidden shortcut boundaries.
- It gates `Milestone 14` and later platform-grade work because replication,
  extensibility, blobs, budgets, repair, and certification must not build on
  ambiguous physical claims.
- It may be drafted while late `13.x` cleanup is being reviewed, but it may not
  close until the through-13.3 status matrix consumes the 13.3 spec and closeout
  as closed semantic evidence and non-platform-grade physical handoff evidence.

## Required Self-Check

- Does the sequence solve a real structural problem or just package work
  cosmetically?
  Yes. It separates semantic Store evidence from physical database evidence so
  Roadmap 2 has a truthful starting line.
- Is the adversarial constraint precise and load-bearing?
  Yes. Every contract and phase prevents heap-shaped persistence, full-object
  decode, backend residue, or overbroad language from masquerading as
  platform-grade physical evidence.
- Does the sequence preserve crate authority boundaries?
  Yes. It preserves runtime semantic authority and Store semantic durability
  while isolating physical byte authority as Roadmap 2 work.
- Does the sequence define proof obligations, not just implementation tasks?
  Yes. It names the required suite, evidence outputs, status rows, counters,
  zero assertions, hostile overclaim lanes, and deferred guarantee mapping.
- Could a competent engineer map this spec into honest types, modules, and
  tests?
  Yes. It names tiers, claim families, status matrices, reports, counters,
  suite rows, and S.1 handoff artifacts.
- Does the sequence belong in this roadmap sequence, or is it out of order?
  Yes. It is the necessary backtrack between Roadmap 1's semantic durability
  program and Roadmap 2's physical database substrate.
