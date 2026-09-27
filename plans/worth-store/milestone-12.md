# Milestone 12 Engineering Spec: Artifact Format Evolution And Rolling Compatibility

> **Status:** Closed via [milestone-12-closeout.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-12-closeout.md)
>
> **Closeout:** [milestone-12-closeout.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-12-closeout.md)
>
> **Roadmap parent:** [worth_store_roadmap.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/worth_store_roadmap.md)
>
> **Vision parent:** [worth_store_vision.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/worth_store_vision.md)
>
> **Test requirements:** [test-requirements.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/test-requirements.md)
>
> **Prerequisite milestones:**
> - [milestone-7.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-7.md)
> - [milestone-8.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-8.md)
> - [milestone-9.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-9.md)
> - [milestone-10.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-10.md)
> - [milestone-11.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-11.md)
>
> **Concurrent milestone context:**
> - [milestone-13.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-13.md) is already closed and supplies placement and recall vocabulary that compatibility must preserve.
>
> **Impacted later milestones:**
> - `Milestone 14` (`Replication, Capsules, And Integrity Verification`)
> - `Milestone 15` (`Extensible Durable Artifact Families And Storage Strategies`)
> - `Milestone 20` (`Native Blob And Object Storage`)
> - `Milestone 22` (`Operator Repair, Audit, And Forensic Recovery Tooling`)
>
> **Primary architectural driver:** make durable artifact compatibility an
> explicit, proof-bearing store subsystem so format versions, reader capability,
> rolling upgrades, derived-family rebuild invalidation, backup/restore, and
> disaster recovery cannot drift into partial truth acceptance.

## Goal

Make authoritative and derived artifact families evolvable across rolling
upgrades, backup/restore, and disaster-recovery windows without allowing
deserialization success, backend-local tolerance, or opportunistic rebuilds to
stand in for semantic compatibility.

## Why This Milestone Exists

Milestone 12 is not "add version numbers."

It is the milestone that decides whether `worth-store` can survive long-lived
artifact evolution after the store already has real authority, support,
retention, maintenance, live-query, bulk, and placement surfaces.

Milestone 7 made schema, lineage, cursor, and checkpoint support artifacts
durable.

Milestone 8 made live-query continuation depend on stable basis and schema
support compatibility.

Milestone 9 made bulk ingest and transform progress durable enough that
interrupted bulk programs can leave restart-visible support artifacts.

Milestone 10 made retention, compaction, reclaim, and rebuild debt explicit.

Milestone 11 made maintenance execution typed, paced, restart-visible, and
foreground-safe.

Milestone 13 made placement and recall cost-only, manifest-driven, and
non-authoritative.

Milestone 12 must now answer the next hard store question:

- how authoritative artifact families evolve without changing historical
  meaning
- how old readers reject new artifacts without silently accepting partial truth
- how new readers interpret old artifacts through declared compatibility rules
  rather than best-effort decoding
- how derived durable families are invalidated, rebuilt, or rejected when their
  format, basis, or accuracy assumptions drift
- how rolling upgrades across processes, stores, replicas, backups, and restore
  targets remain explicit about admitted version windows
- how maintenance, tiering, and later replication can consume compatibility
  evidence instead of inventing local version checks

If this milestone is weak, later milestones spread ambiguity:

- replication will ship artifacts whose recipient compatibility is inferred
  only by whether bytes decode
- extension families will register format contracts without a platform
  compatibility authority
- blob and object storage will invent a parallel version policy for tiered
  payloads
- operator repair tools will face mixed-version damage without knowing whether
  to rebuild, reject, quarantine, or restore

This milestone exists to make compatibility a first-class durable contract
before artifacts are exported widely or extension-defined families multiply.

## Hard Part

The hard part is not choosing a version integer.

The hard part is keeping seven things separate that naive systems collapse:

- artifact byte format
- semantic artifact meaning
- reader capability
- writer capability
- compatibility decision evidence
- derived-family rebuild invalidation
- rolling-upgrade and backup/restore admission policy

The design fails if:

- an old reader skips unknown fields and still reports clean success when those
  fields changed authoritative meaning
- a new reader accepts an old artifact because it decodes, without proving the
  old artifact's semantic meaning maps into the current interpretation
- derived artifacts stay resident after their format or basis assumptions
  changed and are consumed as still exact
- compatibility checks are embedded in each backend or reader as scattered
  booleans instead of one typed compatibility plan
- rolling upgrade lanes rely on process order or deployment folklore rather
  than declared writer, reader, and replica capability windows
- backup/restore treats "can open the file" as sufficient proof that restored
  truth is semantically exact

Milestone 12 therefore has to define one compatibility authority model, one
version-skew admission model, and one derived-family invalidation/rebuild model
that later replication, extension, blob, and operator tooling inherit.

## Explicit Assumptions

- Milestone 1 canonical commit envelopes, version DAG records, and branch heads
  remain authoritative truth.
- Milestone 7 support artifacts for schema, lineage, cursor, and checkpoint
  meaning remain authoritative for their declared support roles.
- Milestone 8 stable-basis and continuation compatibility surfaces remain the
  owner of live-query continuation meaning; Milestone 12 may define artifact
  compatibility windows they depend on, but it may not redefine continuation.
- Milestone 10 remains the owner of retention legality, reclaim eligibility,
  compaction cutover, basis survival, and rebuild debt.
- Milestone 11 remains the owner of background pacing, restart readmission,
  foreground reservations, and maintenance debt escalation.
- Milestone 13 remains the owner of placement, tier residency, recall, and
  working-set classification semantics.
- `worth-relational` still owns truth, schema semantics, identity semantics, and
  transaction meaning; store owns only durable artifact compatibility and
  survival.
- compatibility decisions must be explicit over artifact family, format version,
  semantic version, reader capability, writer capability, and admitted upgrade
  window.
- Milestone 9 bulk progress, chunk, and resume support artifacts are
  compatibility-bearing store artifacts when present.
- deserialization is a byte-level operation and never sufficient compatibility
  proof by itself.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects is identifying the hostile condition
  before the feature shape. Milestone 12 therefore starts from mixed-version
  partial truth acceptance, not from ergonomic migration helpers.
- `arch_laws.md`
  The most important thing it protects here is proof-bearing plan/execute
  separation and authority-versus-derivation discipline. Compatibility
  decisions, reader admission, writer admission, derived invalidation, and
  restore acceptance must be distinct typed phases rather than scattered decode
  checks.
- `perf_laws.md`
  The most important thing it protects is cost and breadth honesty.
  Compatibility checking must expose artifact families inspected, manifests
  checked, rebuilds forced, and rejected lanes rather than hiding upgrade work
  behind "open succeeded."
- `domain_laws.md`
  The most important thing it protects is decomposition by reason-to-change.
  Format manifests, reader capability, writer capability, rolling-upgrade
  admission, derived-family invalidation, and restore compatibility must be
  separate subdomains instead of one versioning module.
- `worth_store_vision.md`
  The most important thing it protects is that store persists canonical
  artifacts without redefining runtime semantics. Milestone 12 must therefore
  preserve old authoritative meaning through declared compatibility contracts
  and reject incompatible artifacts sharply.
- `worth_store_roadmap.md`
  The most important thing it protects is sequencing. Milestone 12 belongs
  after retention and maintenance rules are stable and before replication,
  capsules, extension families, and advanced artifact proliferation spread
  durable bytes across machines.
- `test-requirements.md`
  The most important thing it protects is certification-grade proof. Milestone
  12 is not closed until `Artifact Format Evolution And Rolling Compatibility
  Test` proves old/new/mixed lanes are semantically exact where admitted and
  typed failures where not admitted.
- `worth_store_dependency_map.md`
  The most important thing it protects is unlock shape. Milestone 10 unlocks
  Milestone 12, and Milestone 12 unlocks Milestone 14 because replication needs
  explicit version windows before artifacts can be shipped honestly.
- `milestone-7.md`
  The most important thing it protects is durable support truth for schema,
  lineage, cursor, and checkpoint artifacts. Milestone 12 must version and
  admit those support families without turning schema compatibility into store
  semantics.
- `milestone-8.md`
  The most important thing it protects is stable-basis and continuation
  compatibility. Milestone 12 must supply artifact-family compatibility evidence
  that continuation can consume without absorbing live-query meaning.
- `milestone-9.md`
  The most important thing it protects is deterministic, resumable bulk work
  with canonical artifact parity. Milestone 12 must version bulk progress,
  chunking, and resume support artifacts so interrupted upgrades cannot resume a
  bulk program under a different interpretation than the control lane.
- `milestone-10.md`
  The most important thing it protects is retention, compaction, reclaim, and
  rebuild honesty. Milestone 12 must force derived-family rebuild or
  invalidation when compatibility drift makes retained acceleration unsafe.
- `milestone-11.md`
  The most important thing it protects is one typed maintenance runtime.
  Milestone 12 should route compatibility rebuilds, migration-adjacent
  maintenance, and rolling-format work through that runtime rather than creating
  another worker loop.
- `milestone-11-closeout.md`
  The most important thing it protects is that scheduler containers for derived
  rebuild, snapshot refresh, replication preparation, and audit now exist.
  Milestone 12 can consume those containers for compatibility rebuild and audit
  work without redefining scheduler semantics.
- `milestone-13.md`
  The most important thing it protects is placement non-authority. Milestone 12
  must preserve compatibility across hot/warm/cold manifests and recall records
  without making tier residency part of semantic version meaning.
- `milestone-13-closeout.md`
  The most important thing it protects is that placement, recall, restart
  reconstruction, and typed tier records are closed and machine-certified.
  Milestone 12 must include those records in compatibility reporting because
  they now travel with real stores and future replicas.

## Adversarial Constraint

Milestone 12 must survive this hostile condition:

> A store with authoritative commits, schema/lineage/cursor support artifacts,
> retained and compacted derived families, live-query bases, maintenance backlog,
> tier-residency manifests, backups, and replicas is read and written across
> rolling code versions such that old artifacts with new readers, new artifacts
> with old readers, mixed-version stores, restored backups, and partially
> upgraded replicas either converge to the same semantic truth and derived
> rebuild posture as a single-version control lane or fail explicitly and typed
> before any partial truth is accepted.

## Product Decision Lock

- every durable artifact family must have an explicit format identity, semantic
  compatibility identity, and reader/writer capability window
- authoritative artifact compatibility is decided before artifact meaning is
  exposed to replay, restore, replication preparation, or live-query basis
  planning
- derived artifact compatibility is decided before reuse; incompatible derived
  artifacts are invalidated, rebuilt, or rejected through typed evidence
- compatibility decisions are artifacts of policy and manifest evaluation, not
  byproducts of decoding bytes
- old readers must reject new authoritative artifacts unless the new writer
  declared a downgrade-compatible semantic window and the reader proves it can
  understand that window
- new readers must prove old authoritative artifacts map into current semantic
  expectations rather than assuming old bytes are harmless
- rolling upgrades are admitted through declared `WriterCapability` and
  `ReaderCapability` contracts, not through deployment order folklore
- backup/restore and disaster-recovery windows must carry compatibility
  manifests strong enough to reject incompatible restores before publication
- optional migration convenience tooling may remain `Debt`, but compatibility
  truth and typed rejection may not

Normative consequence:

- any implementation that treats successful deserialization as compatibility is
  out of spec
- any implementation that accepts an unknown authoritative field by ignoring it
  without a semantic compatibility witness is out of spec
- any implementation that lets derived artifacts survive version drift without
  rebuild, invalidation, or typed compatibility proof is out of spec
- any implementation that runs compatibility migration work outside Milestone 11
  maintenance admission is out of spec
- any implementation that restores a backup into a reader without checking the
  backup's artifact-family compatibility manifest is out of spec

## Scope

### In Scope

- compatibility manifests for authoritative and derived durable artifact
  families
- reader and writer capability declarations
- semantic compatibility windows for old-reader/new-writer and
  new-reader/old-writer lanes
- derived artifact invalidation, rebuild, and rejection decisions caused by
  format or semantic version drift
- rolling upgrade admission and mixed-version store/replica reporting
- backup, restore, and disaster-recovery compatibility posture
- compatibility surfaces for tier-residency manifests, recall records,
  maintenance summaries, and replication-preparation inputs already present in
  the store
- machine-checkable compatibility evidence, exact counters, and typed failures

### Explicitly Out Of Scope

- changing runtime schema semantics, transaction semantics, identity semantics,
  or lineage semantics
- implementing replication capsules or cross-machine transfer semantics, which
  remain Milestone 14 work
- defining extension-family registration, which remains Milestone 15 work
- implementing blob/object storage compatibility, which remains Milestone 20
  work, though it must later inherit this compatibility model
- operator repair-plan semantics, which remain Milestone 22 work
- convenience migration tooling beyond the minimum compatibility, rebuild, and
  rejection paths required for correctness

## Practical Implementation Shape

Milestone 12 should introduce one compatibility subsystem rather than scattering
version checks across backend readers.

Expected first-ship subdomains:

- `compatibility/catalog/`
  first-ship artifact family declarations, authority classification, family
  kind, counter family id, and certification lane id
- `compatibility/manifests/`
  durable manifest records, manifest publication units, manifest digests, and
  manifest recovery summaries
- `compatibility/decoding/`
  raw bytes, framed records, quarantined decoded artifacts, and decode-stage
  failure taxonomy
- `compatibility/admission/`
  reader capability, writer capability, compatibility relations, lowered
  admission plans, and compatibility witnesses
- `compatibility/authoritative/`
  authoritative read/write compatibility rules for commits, branches, WAL,
  schema, lineage, cursors, checkpoints, and embedded checkpoint authority
- `compatibility/derived/`
  derived reuse, invalidation, rebuild-required, and degraded non-authority
  posture
- `compatibility/rolling/`
  rolling-upgrade windows, mixed-version store posture, replica posture, and
  maintenance-worker posture
- `compatibility/restore/`
  backup manifests, restore compatibility plans, disaster-recovery windows, and
  restore publication witnesses
- `compatibility/evidence/`
  compatibility matrices, version-skew reports, counter contracts, complexity
  surfaces, and certification bundles

Rules:

- backend code may frame and decode bytes, but it may not decide semantic
  compatibility locally
- replay, restore, live-query, cursor, branch-head, maintenance, and tiering code
  consume compatibility-witnessed types rather than raw decoded records
- compatibility evidence is emitted from the compatibility subsystem, not
  reconstructed after the fact by tests

## Required Contracts And Counters

### First-Ship Artifact Family Catalog Rule

Milestone 12 must freeze a concrete first-ship compatibility catalog so
implementation cannot claim generic "artifact compatibility" while checking only
the easiest family.

Required first-ship authoritative families:

- canonical commit envelopes
- version DAG records
- branch head records
- WAL publication and recovery records that may survive restart
- schema-boundary support artifacts
- lineage support artifacts
- durable cursor and subscriber-checkpoint artifacts
- embedded checkpoint authority records

Required first-ship derived and support families:

- immutable snapshot records and snapshot basis records
- branch-delta layer and rewrite-lineage records
- Milestone 6 aspect-layout slices, structural blocks, chunk manifests, and
  layout-support records
- Milestone 8 stable-basis and continuation descriptor records
- Milestone 9 bulk progress, chunk-boundary, and resume support records
- Milestone 10 retention, compaction, reclaim, basis-survival, and rebuild-debt
  records
- Milestone 11 maintenance descriptors, queue summaries, reservation summaries,
  debt summaries, and completed receipts
- Milestone 13 tier-residency manifests, transfer records, recall records,
  working-set observation windows, and hotness classification records

Rules:

- every family in this catalog must have a manifest, capability posture,
  compatibility decision path, and certification lane
- adding a new durable family must fail compile-time exhaustiveness checks until
  compatibility posture, restore posture, derived invalidation posture, counters,
  and tests are declared
- missing compatibility coverage for any first-ship family is a milestone
  blocker, not optional debt
- future families may remain absent, but shipped families may not remain outside
  the compatibility catalog

This is the anti-"we versioned commits but forgot queue summaries" rule.

### Artifact Family Manifest Rule

Every durable artifact family must declare a compatibility manifest before it is
admitted through compatibility-aware reads, writes, restore, or replication
preparation.

Required manifest vocabulary:

- `ArtifactFamilyId`
- `ArtifactFormatVersion`
- `ArtifactSemanticVersion`
- `ArtifactCompatibilityWindow`
- `ArtifactFamilyCompatibilityManifest`
- `AuthoritativeCompatibilityManifest`
- `DerivedCompatibilityManifest`
- `CompatibilityManifestDigest`

Rules:

- authoritative and derived manifests are separate types
- an artifact family may not be read through the compatibility facade without a
  manifest
- semantic compatibility is distinct from byte-format compatibility
- manifest digests participate in certification bundles and later replication
  preparation
- manifest parsing success is not compatibility acceptance

### Compatibility Relation Rule

Milestone 12 must not infer semantic compatibility from numeric version
arithmetic.

Required compatibility relation variants:

- `NativeCompatible`
- `ForwardCompatible`
- `BackwardCompatible`
- `DeterministicAdapterRequired`
- `DerivedRebuildRequired`
- `RejectIncompatible`

Required relation vocabulary:

- `CompatibilityRelation`
- `DeclaredCompatibilityEdge`
- `CompatibilityEdgeProof`
- `CompatibilityAdapterId`
- `CompatibilityAdapterDigest`
- `CompatibilityAdapterParityWitness`

Rules:

- no version number implies compatibility by ordering, prefix, major/minor
  equality, or string comparison
- every non-native acceptance must name one declared compatibility edge
- deterministic adapters are admitted only when they are named, digest-bound,
  and parity-proven against a control lane
- if no declared edge exists, the result is `RejectIncompatible`
- derived artifacts may never use a deterministic adapter as proof of authority;
  adapter output is either rebuilt derived state or rejected
- convenience migration tooling may remain `Debt`, but any adapter that ships in
  the milestone's compatibility path must be certified

This is the anti-`version.major == current.major` rule.

### Decode Quarantine Rule

Decoding bytes must produce quarantined records, not semantic artifacts.

Required decode-stage vocabulary:

- `RawArtifactBytes`
- `FramedArtifactRecord`
- `QuarantinedDecodedArtifact`
- `CompatibilityCheckedArtifact`
- `CompatibilityAdmittedArtifact`
- `SemanticArtifactView`

Rules:

- `QuarantinedDecodedArtifact` exposes only family id, format id, semantic id,
  manifest digest, structural digest, and diagnostic context
- semantic accessors are available only on `CompatibilityAdmittedArtifact` or a
  stronger phase-typed wrapper
- byte framing, checksum validation, decode, compatibility admission, and
  semantic exposure are distinct phases
- typed failures distinguish malformed bytes, unsupported format, unsupported
  semantic meaning, missing compatibility edge, adapter failure, and partial
  truth rejection
- no backend may return a decoded authoritative record directly to replay,
  restore, live-query, cursor, or branch-head code

This is the anti-"decode first and remember to check version nearby" rule.

### Compatibility Registry Exhaustiveness Rule

Artifact-family compatibility must be declared through one registry surface that
the compiler can audit.

Required registry surfaces:

- `CompatibilityRegistry`
- `CompatibilityFamilyDeclaration`
- `AuthoritativeFamilyDeclaration`
- `DerivedFamilyDeclaration`
- `CompatibilityFamilyKind`
- `CompatibilityRegistrySnapshot`

Required declaration fields:

- artifact family id
- authority classification
- format version set
- semantic version set
- reader capability set
- writer capability set
- declared compatibility edges
- restore compatibility posture
- rolling-upgrade posture
- derived invalidation or rebuild posture when applicable
- replication-export posture for later Milestone 14 consumption
- counter family id
- certification lane id

Rules:

- declarations are data, not scattered match arms
- adding a `CompatibilityFamilyKind` variant must require updating manifest
  loading, read admission, write admission, restore admission, rolling upgrade,
  certification evidence, and counter mapping
- external callers may not register authoritative families; later extension work
  may register only derived families under Milestone 15 containment rules
- registry snapshots are immutable inputs to compatibility planning

This is the anti-"one more family got added without compatibility coverage"
rule.

### Manifest Publication And Recovery Rule

Compatibility manifests are durable support artifacts and must have their own
publication and restart rules.

Required publication surfaces:

- `CompatibilityManifestPublicationUnit`
- `ManifestPublicationWitness`
- `ManifestRecoverySummary`
- `ManifestPublicationGap`
- `ManifestDigestMismatch`

Rules:

- authoritative family manifests required to interpret a committed artifact must
  publish in the same durable publication unit as the first artifact that needs
  them, or the artifact publication must be rejected
- restart must reject or degrade typed when an artifact exists without its
  required compatibility manifest
- manifest updates must be append-only and frontier-scoped; mutating an old
  manifest in place is forbidden
- manifest identity must be deterministic from family id, version window, and
  semantic compatibility declaration
- recovery may rebuild derived compatibility summaries from manifests, but it
  may not invent missing authoritative compatibility manifests from decoder
  behavior

This is the anti-"we will backfill manifests later" rule.

### First-Ship Compatibility Policy Rule

Milestone 12 must define conservative admitted policies before any flexible
upgrade story is claimed.

Required first-ship admitted policies:

- native same-format/same-semantic reads and writes for all catalog families
- new-reader/old-authority acceptance only through explicit
  `ForwardCompatible` edges
- old-reader/new-authority acceptance only through explicit
  `BackwardCompatible` edges
- derived-family invalidation when format or semantic basis drift lacks a
  certified reuse edge
- derived-family rebuild from retained compatible authority when the retained
  basis survives and rebuild is already admitted by earlier milestones
- rolling upgrade windows with at most one old reader capability set and one new
  writer capability set per admitted lane
- restore publication only after all included authoritative families pass
  compatibility planning

Explicit first-ship debt:

- multi-hop compatibility chains
- arbitrary adapter composition
- in-place authoritative migration
- mixed fleets with more than two writer capability sets in one admitted window
- heuristic downgrade of authoritative artifacts
- compatibility inference from domain schema versions without store-level
  family manifests

Rules:

- unsupported policies must reject typed or remain marked `Debt`
- no `Debt` policy may be used to publish authoritative truth
- first-ship certification must cover the admitted conservative policies before
  broader rolling-upgrade ergonomics are added

This is the anti-"the versioning framework is flexible, therefore it works"
rule.

### Reader And Writer Capability Rule

Compatibility must be decided from explicit reader and writer capabilities, not
from ambient crate version strings or deployment assumptions.

Required capability vocabulary:

- `ReaderCapabilitySet`
- `WriterCapabilitySet`
- `CompatibilityReadIntent`
- `CompatibilityWriteIntent`
- `CompatibilityAdmissionPlan`
- `CompatibilityDecision`
- `CompatibilityRejection`

Rules:

- readers declare exactly which format and semantic windows they can interpret
- writers declare exactly which format and semantic windows their outputs require
- a write may not publish artifacts whose declared writer requirement exceeds
  the admitted rolling window
- a read may not expose artifact meaning until compatibility has produced an
  accepting `CompatibilityDecision`
- "same crate version" may be useful diagnostics, but it is not the authority
  basis for compatibility admission

### Authoritative Semantic Compatibility Rule

Authoritative artifacts may evolve only through declared semantic windows that
preserve replay, branch, schema-support, cursor-support, lineage-support, and
restore meaning.

Required authoritative surfaces:

- `AuthoritativeCompatibilityWitness`
- `SemanticMeaningPreservationWitness`
- `ForwardReadCompatibilityWitness`
- `BackwardReadCompatibilityWitness`
- `UnsupportedAuthoritativeVersion`
- `PartialTruthRejection`

Rules:

- new code reading old authority must prove the old artifact's meaning maps into
  the current semantic contract
- old code reading new authority must reject unless the new artifact explicitly
  declares backward-compatible meaning for that reader capability
- unknown authoritative fields are fatal unless covered by a semantic
  preservation witness
- canonical replay, branch head resolution, support-artifact fetch, cursor
  resume, and snapshot-tail restore may consume only compatibility-proven
  authoritative artifacts

### Derived Artifact Invalidation Rule

Derived artifacts must declare how compatibility drift affects reuse.

Required derived surfaces:

- `DerivedCompatibilityWitness`
- `DerivedInvalidationPlan`
- `DerivedRebuildCompatibilityPlan`
- `DerivedCompatibilityReuseWitness`
- `CompatibilityRebuildDebt`
- `StaleDerivedVersionRejection`

Rules:

- derived artifacts with incompatible format or semantic basis may not be reused
  as exact accelerators
- exact derived families must rebuild or reject when their declared basis
  version is incompatible
- conservative, approximate, heuristic, or advisory derived families may degrade
  only through explicit compatibility policy and result-surface classification
- rebuild-required decisions must emit Milestone 10 rebuild debt and enter
  Milestone 11 maintenance admission if background execution is needed
- compatibility invalidation must identify the artifact family and retained
  basis it depends on

### Rolling Upgrade Admission Rule

Rolling upgrades must be admitted by version-window plans that account for
readers, writers, stores, replicas, and maintenance workers.

Required rolling-upgrade surfaces:

- `RollingUpgradeWindow`
- `MixedVersionStorePosture`
- `ReplicaCompatibilityPosture`
- `MaintenanceCompatibilityPosture`
- `UpgradeAdmissionWitness`
- `UpgradeSkewRejection`

Rules:

- a mixed-version lane must name the active reader and writer capability sets
- a writer may not publish artifacts that admitted readers in the same window
  must reject unless the write is isolated from those readers by explicit policy
- maintenance workers may not rewrite or rebuild artifacts into a format outside
  the admitted rolling window
- tier movement and recall may move bytes only after compatibility for the
  underlying artifact family is accepted; movement does not convert format
  compatibility
- rolling upgrade state must be restart-visible and reportable, not only
  process-local

### Backup, Restore, And Disaster-Recovery Compatibility Rule

Backups and disaster-recovery restores must carry enough compatibility evidence
to reject unsafe restores before publication.

Required restore surfaces:

- `BackupCompatibilityManifest`
- `RestoreCompatibilityPlan`
- `DisasterRecoveryCompatibilityWindow`
- `RestorePublicationWitness`
- `RestoreVersionRejection`

Rules:

- restore planning checks every included authoritative family before any restored
  authority is published
- derived families included in backup may be restored only if compatible;
  otherwise they are invalidated or rebuilt from compatible retained authority
- a restore target may not publish a branch head, cursor checkpoint, schema
  support artifact, lineage support artifact, tier manifest, or snapshot family
  whose compatibility decision is still pending
- disaster-recovery windows must distinguish "truth restorable" from "derived
  acceleration restorable"
- restore evidence must be machine-checkable without needing the original
  producing process

### Failure Topology Rule

Compatibility failures must be typed by cause and artifact boundary, not merged
into one unsupported-version error.

Required failure families:

- `MalformedArtifactBytes`
- `ArtifactFrameVersionUnsupported`
- `CompatibilityManifestMissing`
- `CompatibilityManifestDigestMismatch`
- `ArtifactFamilyUndeclared`
- `ArtifactFormatUnsupported`
- `ArtifactSemanticVersionUnsupported`
- `CompatibilityEdgeMissing`
- `CompatibilityAdapterParityFailure`
- `AuthoritativePartialTruthRejected`
- `DerivedReuseVersionRejected`
- `DerivedRebuildBasisIncompatible`
- `RollingUpgradeWindowRejected`
- `RestoreCompatibilityRejected`
- `DisasterRecoveryWindowRejected`

Rules:

- every failure carries artifact family id, format version, semantic version,
  reader capability, writer capability where relevant, and the decision boundary
  that rejected it
- malformed bytes, unsupported format, unsupported semantic meaning, and missing
  compatibility edge are distinct failures
- restore and rolling-upgrade failures identify whether truth authority,
  support authority, or derived acceleration caused rejection
- failure digests participate in the certification bundle

This is the anti-"unsupported version" catch-all rule.

### Compatibility Plan Lowering Rule

Execution must consume lowered compatibility plans rather than rediscovering
version posture inside readers, writers, rebuilders, or restore paths.

Required plan families:

- `AuthoritativeReadCompatibilityPlan`
- `AuthoritativeWriteCompatibilityPlan`
- `DerivedReuseCompatibilityPlan`
- `DerivedRebuildRequiredPlan`
- `RollingUpgradeCompatibilityPlan`
- `RestoreCompatibilityPlan`
- `CompatibilityAuditPlan`

Rules:

- compatibility policy resolution occurs before artifact decoding exposes
  semantic meaning
- execution may not re-decide compatibility from raw version fields after a plan
  exists
- rejected plans carry typed reasons and exact artifact-family scope
- rebuild and audit plans route through the Milestone 11 scheduler containers
  when they need background execution

### Compile-Time Boundary Rule

The highest-risk compatibility boundaries must be compiler-enforced rather than
left as doc-only rules.

Required proof-bearing surfaces:

- `CompatibilityAdmissionPlan`
- `AuthoritativeCompatibilityWitness`
- `DerivedCompatibilityWitness`
- `SemanticMeaningPreservationWitness`
- `RestorePublicationWitness`
- `UpgradeAdmissionWitness`

Required compile-time posture:

- decoded records are quarantined and may not expose semantic accessors before
  compatibility admission
- authoritative readers may not consume raw decoded records directly; they must
  consume compatibility-witnessed records
- derived reuse may not consume a raw artifact id or raw format version; it must
  consume a `DerivedCompatibilityWitness`
- restore publication may not consume a generic "restore succeeded" flag; it
  must consume `RestorePublicationWitness`
- rolling upgrade writes may not publish artifacts without
  `UpgradeAdmissionWitness`
- compatibility witnesses may not be publicly constructible by callers or
  deserialization code
- adding a new durable artifact family must require updating the compatibility
  registry and evidence mapping before the crate compiles

Required proof surface:

- compile-fail tests for semantic access from quarantined decoded artifacts
- compile-fail tests for raw authoritative read after decode without witness
- compile-fail tests for synthetic compatibility witness construction
- compile-fail tests for derived artifact reuse without compatibility proof
- compile-fail tests for restore publication without restore compatibility proof
- compile-fail tests for rolling-upgrade writer publication without admission
- compile-fail tests for adding a compatibility family without manifest,
  restore, counter, and evidence declarations

### Performance-Shaping Types Rule

Milestone 12 must encode compatibility breadth into lowered types so hot readers
do not repeatedly rediscover version posture.

Required performance-shaping surfaces:

- `CompatibilityManifestSummary`
- `ArtifactFamilyVersionSummary`
- `ReaderWriterSkewSummary`
- `DerivedInvalidationSummary`
- `CompatibilityRebuildSummary`
- `RestoreVersionSummary`
- `CompatibilityAuditSummary`
- `CompatibilityAdmissionReceipt`
- `ArtifactFamilyCompatibilityIndex`
- `CompatibilityBatchScope`
- `CompatibilityAdapterCostClass`
- `RestoreCompatibilityBreadthBudget`

Required posture:

- manifest summaries are loaded once per admitted compatibility boundary
- family version posture is carried into reads, writes, restore, and rebuild
  paths instead of recomputed per artifact row
- derived invalidation breadth is summarized before rebuild or reject execution
- rolling-upgrade skew is summarized by participating capability sets and
  artifact families
- hot reads consume an immutable `CompatibilityAdmissionReceipt` tied to one
  registry snapshot, reader capability set, artifact family, and version window
- repeated reads in the same admitted batch reuse the admission receipt instead
  of re-running manifest lookup, relation resolution, or adapter selection
- compatibility indexes are built from manifest summaries and registry snapshots,
  not from backend inventory walks
- deterministic adapters declare a cost class before admission
- restore planning declares a breadth budget before any target publication work

Rules:

- readers may not scan the full store manifest inventory for every artifact read
- readers may not allocate new compatibility decision graphs per artifact when a
  batch admission receipt already exists
- compatibility admission receipts are proof-carrying values, not caches; they
  expire only when the registry snapshot, manifest digest, reader capability set,
  writer capability set, or artifact-family version window changes
- restore may not publish incrementally while compatibility summary is still
  incomplete
- restore may not inspect target-store families outside the declared restore
  scope except through explicit conflict or publication-precondition units
- compatibility adapters may not run on hot read paths unless their declared cost
  class is `InlineBounded` and their parity proof is already admitted
- allocation for manifest loading, edge resolution, and adapter planning must
  belong to an explicit compatibility planning scope rather than per-artifact
  heap churn
- compatibility audit may broaden deliberately, but it must expose that breadth
  in counters and summaries

### Admission Receipt Reuse Rule

Compatibility proof may be carried forward inside one trusted read, write,
restore, or rebuild boundary instead of revalidated per artifact row.

Required receipt surfaces:

- `ReadCompatibilityReceipt`
- `WriteCompatibilityReceipt`
- `DerivedReuseCompatibilityReceipt`
- `RestoreCompatibilityReceipt`
- `RollingWindowCompatibilityReceipt`

Rules:

- receipts are constructed only from a `CompatibilityAdmissionPlan` and immutable
  registry snapshot
- receipts carry the exact manifest digest, compatibility relation, reader
  capability, writer capability when relevant, artifact family id, and version
  window they admit
- receipts are consumed by artifact-family readers and writers as proof that
  compatibility was already resolved for the batch boundary
- receipts may not be serialized as durable authority; they are phase-local proof
  wrappers
- if a batch crosses artifact families, it carries one receipt per family rather
  than one global "compatible" flag

This is the anti-recheck-every-row rule.

### Compatibility Index Locality Rule

Manifest and capability lookup must be indexed by the dimensions used by hot
compatibility checks.

Required index keys:

- `(ArtifactFamilyId, ArtifactFormatVersion, ArtifactSemanticVersion)`
- `(ReaderCapabilitySetId, ArtifactFamilyId, ArtifactSemanticVersion)`
- `(WriterCapabilitySetId, ArtifactFamilyId, ArtifactSemanticVersion)`
- `(CompatibilityAdapterId, source semantic version, target semantic version)`

Rules:

- hot read admission must resolve through indexed family/version keys, not
  linear scans over manifests or capability lists
- rolling-upgrade admission may inspect every family in the declared rolling
  window, but not families outside that window
- restore compatibility may inspect every family in the backup scope, but not
  unrelated target-store families unless a publication conflict is declared
- index reconstruction cost must be proportional to manifest entries and
  declared compatibility edges, not stored artifact count

### Adapter Cost Boundary Rule

Compatibility adapters are not free and must not become hidden decode cost.

Required adapter cost classes:

- `InlineBounded`
- `BatchBounded`
- `MaintenanceScheduled`
- `RejectedForHotPath`

Rules:

- `InlineBounded` adapters must have exact input and output width counters and
  may touch only the artifact currently being admitted
- `BatchBounded` adapters must run over a declared `CompatibilityBatchScope`
  with exact item counts before execution begins
- `MaintenanceScheduled` adapters must enter Milestone 11 scheduling through a
  compatibility work unit
- hot read paths reject adapters classified as `BatchBounded` or
  `MaintenanceScheduled`
- adapter output must include counters for records read, records emitted,
  allocation scope used, and fallback/rejection count

This is the anti-"compatibility adapter accidentally became migration engine on
the read path" rule.

### Lowered Work Unit Families Rule

Compatibility work must be partitioned by semantic family and execution purpose
before scheduling or execution.

Required lowered work units:

- `AuthoritativeCompatibilityCheckUnit`
- `DerivedCompatibilityCheckUnit`
- `DerivedCompatibilityRebuildUnit`
- `RollingUpgradeAdmissionUnit`
- `RestoreCompatibilityUnit`
- `CompatibilityAuditUnit`
- `CompatibilityReportUnit`

Rules:

- every unit ties to one artifact family or one declared rolling window
- authoritative and derived units remain distinct even if one executor checks
  both
- rebuild units remain separate from compatibility-check units
- reporting units may not mutate compatibility state
- Milestone 11 may schedule units, but Milestone 12 defines their meaning

### Read And Result Cost Surface Rule

Every compatibility-aware read, write, rebuild, restore, and upgrade admission
must expose compatibility posture and cost evidence in its result envelope.

Required result surfaces:

- accepted or rejected compatibility decision
- artifact family and version window inspected
- reader and writer capability posture
- derived invalidation or reuse classification
- rebuild-debt delta
- rolling-upgrade skew classification
- restore publication classification
- compatibility audit breadth

Rules:

- callers can tell whether success came from exact native compatibility,
  forward compatibility, backward compatibility, derived rebuild, or explicit
  degraded non-authority posture
- zero-work native acceptance and real compatibility migration or rebuild are
  distinguishable
- incompatible artifacts fail with typed reasons before semantic meaning is
  exposed

### Complexity-Status Surface Rule

Milestone 12 evidence must publish path-local complexity status rather than one
rolled-up compatibility verdict.

Minimum named paths:

- `compatibility_manifest_load`
- `compatibility_index_reconstruction`
- `compatibility_admission_receipt_build`
- `authoritative_read_compatibility_check`
- `authoritative_write_compatibility_check`
- `derived_reuse_or_invalidation`
- `compatibility_adapter_execution`
- `rolling_upgrade_admission`
- `restore_version_check`
- `compatibility_audit`

Rules:

- each path declares at least `Verified` or `Debt`
- any `Debt` path names the unresolved breadth, unsupported migration shape, or
  convenience tooling gap explicitly

Minimum contracts:

- compatibility manifest load cost is proportional to:
  - artifact-family manifests loaded
  - declared version windows summarized
  - not total artifact row count
- compatibility index reconstruction cost is proportional to:
  - manifest entries loaded
  - declared compatibility edges indexed
  - declared adapter records indexed
  - not stored artifact count
- compatibility admission receipt build cost is proportional to:
  - artifact families in the admitted batch
  - capability windows evaluated for those families
  - declared compatibility edges selected
  - not artifacts read inside the batch
- authoritative read compatibility check cost is proportional to:
  - artifact families read
  - capability windows evaluated
  - admission receipts consumed
  - not artifact rows read after receipt construction
- authoritative write compatibility check cost is proportional to:
  - writer families emitted
  - admitted reader capability windows checked
  - not unrelated stored families
- derived reuse or invalidation cost is proportional to:
  - derived families inspected
  - basis-version comparisons required
  - invalidation or rebuild plans emitted
- rolling upgrade admission cost is proportional to:
  - participating capability sets
  - artifact families in the rolling window
  - maintenance workers admitted for rewrite or rebuild
- restore version check cost is proportional to:
  - artifact families in the backup or disaster-recovery scope
  - version windows evaluated
  - declared publication conflicts checked
  - not target store size outside the restore scope
- compatibility audit cost is proportional to:
  - audited manifests
  - audited artifact-family records
  - declared broadened audit scope
- compatibility adapter execution cost is proportional to:
  - declared adapter input records
  - declared adapter output records
  - declared adapter allocation scope
  - not ambient reader count or unrelated artifacts

Forbidden hidden work:

- full history replay to determine whether one artifact family version is
  admitted
- accepting unknown authoritative fields because a decoder ignored them
- per-artifact manifest scans in hot read paths when a trusted family summary is
  already available
- rebuilding compatibility indexes from stored artifact rows instead of manifest
  and registry entries
- recomputing compatibility relations once a valid admission receipt exists for
  the batch
- running batch or maintenance-class adapters inline on hot read paths
- adapter allocation outside a declared planning, batch, or maintenance scope
- background rebuild work that bypasses maintenance admission
- restoring derived artifacts as exact after incompatible basis drift
- treating tier-residency metadata as format conversion evidence

Minimum counters:

- `compatibility_manifest_load_count`
- `compatibility_manifest_rejection_count`
- `compatibility_manifest_publication_count`
- `compatibility_manifest_gap_count`
- `compatibility_registry_family_count`
- `compatibility_registry_exhaustiveness_rejection_count`
- `compatibility_index_rebuild_count`
- `compatibility_index_manifest_entry_count`
- `compatibility_index_artifact_row_scan_count`
- `compatibility_admission_receipt_count`
- `compatibility_receipt_reuse_count`
- `compatibility_relation_resolution_count`
- `compatibility_relation_recheck_after_receipt_count`
- `quarantined_decode_count`
- `quarantined_semantic_access_rejection_count`
- `compatibility_accept_count`
- `compatibility_typed_reject_count`
- `compatibility_edge_accept_count`
- `compatibility_edge_missing_rejection_count`
- `compatibility_adapter_inline_count`
- `compatibility_adapter_batch_count`
- `compatibility_adapter_maintenance_scheduled_count`
- `compatibility_adapter_hot_path_rejection_count`
- `compatibility_adapter_input_record_count`
- `compatibility_adapter_output_record_count`
- `compatibility_adapter_allocation_scope_count`
- `compatibility_adapter_parity_failure_count`
- `authoritative_forward_compat_accept_count`
- `authoritative_backward_compat_accept_count`
- `authoritative_partial_truth_rejection_count`
- `unknown_authoritative_field_rejection_count`
- `derived_compatibility_accept_count`
- `derived_invalidation_count`
- `derived_rebuild_required_count`
- `compatibility_rebuild_debt_count`
- `rolling_upgrade_window_admission_count`
- `rolling_upgrade_skew_rejection_count`
- `mixed_version_store_lane_count`
- `mixed_version_replica_lane_count`
- `maintenance_compatibility_rebuild_admission_count`
- `restore_compatibility_accept_count`
- `restore_version_rejection_count`
- `restore_scope_family_count`
- `restore_target_out_of_scope_family_scan_count`
- `disaster_recovery_window_check_count`
- `compatibility_audit_family_count`
- `compatibility_truth_parity_failure_count`
- `compatibility_restore_parity_failure_count`

Required counter assertions:

- `compatibility_truth_parity_failure_count` remains zero for admitted
  compatibility lanes
- `compatibility_restore_parity_failure_count` remains zero for admitted
  restore lanes
- `compatibility_manifest_gap_count` remains zero in admitted publication and
  restart lanes
- `compatibility_index_artifact_row_scan_count` remains zero for representative
  index reconstruction lanes
- `compatibility_relation_recheck_after_receipt_count` remains zero once an
  admission receipt exists for the batch
- `compatibility_receipt_reuse_count` increments for repeated artifact reads in
  the same family/window batch
- `compatibility_adapter_hot_path_rejection_count` increments when a hot read
  attempts to use a `BatchBounded` or `MaintenanceScheduled` adapter
- `restore_target_out_of_scope_family_scan_count` remains zero in representative
  scoped restore lanes
- `quarantined_semantic_access_rejection_count` increments in compile-fail or
  hostile proof lanes that attempt semantic access before compatibility
  admission
- `compatibility_edge_missing_rejection_count` increments when numeric version
  proximity exists but no declared compatibility edge exists
- `compatibility_adapter_parity_failure_count` remains zero for admitted
  certified adapters and increments in hostile adapter-drift lanes
- `authoritative_partial_truth_rejection_count` increments in hostile lanes that
  attempt partial old-reader acceptance of new authoritative meaning
- `unknown_authoritative_field_rejection_count` increments when unknown fields
  lack semantic preservation witnesses
- `derived_invalidation_count` and `derived_rebuild_required_count` distinguish
  discard from rebuild-required posture
- `maintenance_compatibility_rebuild_admission_count` exactly matches rebuild
  work routed through Milestone 11 in representative lanes
- `rolling_upgrade_skew_rejection_count` increments only where the declared
  rolling window cannot safely admit participating readers or writers

## Phases

### Phase 1: Lock Compatibility Vocabulary, Manifests, And Authority Boundaries

Phase 1 defines what compatibility is allowed to mean before any reader,
writer, restore path, or rebuild path consumes versioned artifacts.

Required work:

- define the first-ship artifact family catalog and compatibility registry
- define artifact-family manifest vocabulary for authoritative and derived
  families
- define byte-format version, semantic version, and compatibility-window
  identities as distinct types
- define explicit compatibility relation variants and forbid numeric
  version-order inference
- define raw, framed, quarantined, admitted, and semantic artifact type stages
- define compatibility admission receipts and their invalidation basis
- define compatibility index keys and reconstruction boundaries
- define adapter cost classes and hot-path rejection posture
- define restore breadth budgets and scoped publication conflict checks
- define reader and writer capability sets
- define compatibility admission plans and typed compatibility decisions
- define authoritative semantic compatibility witnesses
- define derived compatibility, invalidation, rebuild, and reuse witnesses
- define rolling-upgrade windows and mixed-version posture vocabulary
- define backup, restore, and disaster-recovery compatibility manifests
- define manifest publication and restart recovery rules
- define compatibility failure topology
- define compatibility counters and certification bundle shape
- define compile-time witness privacy requirements

Exit condition:

- compatibility is a typed store subsystem, not scattered decoder behavior
- authoritative and derived compatibility cannot be confused
- deserialization has no path to expose semantic meaning without compatibility
  evidence
- adding a durable artifact family without compatibility coverage is
  mechanically blocked

### Phase 2: Implement Reader, Writer, And Manifest Admission

Phase 2 turns compatibility declarations into machine-checkable admission for
normal artifact reads and writes.

Required work:

- implement manifest persistence and restart-visible manifest summaries
- implement manifest publication units and manifest gap recovery failures
- implement compatibility index reconstruction from manifests and registry
  entries, not stored artifact rows
- implement decode quarantine from raw bytes to quarantined decoded artifacts
- implement reader capability admission for authoritative artifact reads
- implement writer capability admission for authoritative artifact publication
- implement admission receipt construction and reuse for family/window batches
- implement compatibility registry snapshots and family exhaustiveness checks
- implement old-reader/new-writer and new-reader/old-writer decision lanes
- implement declared compatibility edge checks and deterministic adapter parity
  gates where adapters are admitted
- implement adapter cost-class rejection for hot reads and Milestone 11 admission
  for maintenance-scheduled adapters
- implement typed rejections for unsupported format, unsupported semantic
  version, missing edge, unknown authoritative fields, malformed bytes,
  manifest gaps, and partial truth attempts
- implement result envelopes that expose native, forward, backward, rejected,
  and degraded non-authority compatibility posture
- emit exact manifest, quarantine, edge, acceptance, rejection, and
  partial-truth counters

Exit condition:

- reads and writes cannot rely on raw decode success
- accepted authoritative artifacts carry explicit semantic compatibility proof
- incompatible authoritative artifacts fail typed before meaning is exposed
- numeric version proximity cannot admit compatibility without a declared edge
- repeated reads in one family/window batch reuse admission receipts instead of
  re-resolving manifests and compatibility edges
- compatibility index rebuild is manifest-bounded rather than artifact-row-scan
  based

### Phase 3: Invalidate, Rebuild, Or Reject Derived Families Under Version Drift

Phase 3 makes compatibility exact for snapshots, compaction products, layout
families, live-query bases, maintenance summaries, tier manifests, and other
derived or support families already present in the store.

Required work:

- implement derived-family compatibility checks against format and semantic
  basis versions
- implement derived invalidation plans for incompatible reusable artifacts
- implement compatibility rebuild-required plans for exact derived families
- implement first-ship catalog coverage for snapshots, branch deltas, Milestone
  6 layout families, Milestone 8 basis records, Milestone 9 bulk support,
  Milestone 10 retention records, Milestone 11 maintenance records, and
  Milestone 13 tier records
- route compatibility rebuild work through Milestone 11 maintenance admission
- preserve Milestone 10 rebuild-debt evidence when rebuild is deferred
- preserve Milestone 13 placement and recall non-authority while checking tier
  records and residency manifests
- expose typed stale-derived, incompatible-basis, and rebuild-admission failures
- emit exact derived accept, invalidation, rebuild, and rebuild-debt counters

Exit condition:

- derived artifacts cannot survive version drift as hidden exact accelerators
- incompatible derived families either rebuild from retained compatible authority
  or fail typed
- compatibility rebuild work shares the typed maintenance runtime instead of
  creating a side worker path

### Phase 4: Admit Rolling Upgrades, Backups, Restores, And Disaster-Recovery Windows

Phase 4 turns compatibility from local read/write checks into an operational
store posture.

Required work:

- implement rolling-upgrade window admission over reader and writer capability
  sets
- implement the first-ship two-capability rolling window and typed rejection for
  unsupported multi-hop or multi-writer windows
- implement mixed-version store and replica compatibility reporting
- implement maintenance-worker compatibility posture for rebuild and format
  rewrite work
- implement backup compatibility manifests
- implement restore compatibility plans and restore publication witnesses
- implement restore breadth budgets over backup-scope families and explicit
  publication-conflict units
- implement disaster-recovery compatibility windows that distinguish truth
  restoration from derived acceleration restoration
- reject unsafe restore publication before branch heads, cursor checkpoints,
  schema support, lineage support, tier manifests, or snapshots become visible
- emit exact rolling-upgrade, mixed-version, restore, and disaster-recovery
  counters

Exit condition:

- rolling upgrades are admitted or rejected from declared capability windows
- backups carry compatibility evidence strong enough for future restore targets
- restores cannot publish semantically ambiguous truth
- restore compatibility remains scoped to backup families plus declared
  publication conflicts rather than target-store scans

### Phase 5: Prove Artifact Format Evolution And Rolling Compatibility

Phase 5 turns compatibility into a certifiable store surface.

Required work:

- run the Milestone 12 named suite:
  `Artifact Format Evolution And Rolling Compatibility Test`
- compare old-artifact/new-reader lanes against single-version control lanes
- compare new-artifact/old-reader rejection lanes against typed failure
  expectations
- compare mixed-version store and mixed-version replica lanes against admitted
  rolling-upgrade windows
- include catalog-completeness lanes for every first-ship artifact family
- compare derived-family invalidation and rebuild lanes against retained
  authority rebuild control lanes
- include backup, restore, and disaster-recovery lanes across admitted version
  windows
- include hostile lanes where deserialization succeeds but semantic
  compatibility is not declared
- include hostile lanes where numeric version proximity exists but no declared
  compatibility edge exists
- include hostile lanes where a manifest is missing, mismatched, or recovered
  after the artifact that required it
- include hot-read receipt reuse lanes proving relation checks are not repeated
  per artifact row
- include index reconstruction lanes proving no stored artifact row scan is used
  to build compatibility indexes
- include adapter cost-class lanes proving batch and maintenance adapters reject
  on hot read paths
- include scoped restore lanes proving out-of-scope target families are not
  scanned
- include compile-fail coverage for quarantine, synthetic witnesses, and
  registry exhaustiveness
- emit machine-checkable artifact, failure, compatibility, version-skew,
  diagnostics, and counter bundles

Exit condition:

- compatible artifacts preserve authoritative meaning across version windows
- incompatible artifacts fail typed before partial truth acceptance
- derived families rebuild, invalidate, or reject exactly where compatibility
  requires
- backup and restore compatibility is machine-checkable
- Milestone 12 closeout evidence exists in machine-checkable form

## Must Ship

- concrete first-ship artifact family compatibility catalog
- compatibility registry with exhaustive family declarations
- explicit artifact-family compatibility manifests for authoritative and derived
  families
- distinct byte-format, semantic-version, and compatibility-window identities
- explicit compatibility relation variants with declared compatibility edges
- decode quarantine from raw bytes to compatibility-admitted semantic views
- durable manifest publication and restart recovery rules
- compatibility admission receipts for batch-local proof reuse
- manifest-backed compatibility indexes with no artifact-row reconstruction
- adapter cost classes and hot-path adapter rejection
- restore breadth budgets scoped to backup families and declared publication
  conflicts
- reader and writer capability declarations
- compatibility admission plans and typed decisions
- authoritative semantic compatibility witnesses
- derived compatibility, invalidation, rebuild, and reuse witnesses
- deterministic compatibility adapter gates where any adapter is admitted
- rolling-upgrade windows and mixed-version posture reporting
- concrete first-ship rolling-upgrade policy with explicit unsupported-policy
  debt markers
- backup, restore, and disaster-recovery compatibility manifests
- restore publication witnesses
- compatibility rebuild routing through Milestone 11 maintenance admission
- Milestone 10 rebuild-debt preservation for deferred compatibility rebuilds
- Milestone 13 placement and recall compatibility reporting for tier manifests
  and residency records
- compile-fail boundary coverage for raw decoded authority, synthetic
  compatibility witnesses, semantic access from quarantined records, derived
  reuse without proof, registry incompleteness, restore publication without
  proof, and rolling-upgrade writer publication without admission
- typed compatibility, version-skew, restore, and stale-derived failures
- exact compatibility, rejection, invalidation, rebuild, rolling-upgrade, and
  restore counters
- exact receipt reuse, relation recheck, index rebuild, adapter width, adapter
  allocation-scope, and restore out-of-scope scan counters
- machine-checkable Milestone 12 certification output

## Must Preserve

- canonical authoritative meaning remains owned by canonical artifacts and
  runtime semantics, not by compatibility adapters
- older authoritative meaning may not drift when new fields or format versions
  appear
- deserialization success never substitutes for compatibility proof
- schema, lineage, cursor, checkpoint, and live-query basis meaning remains
  owned by the support and live-query surfaces that introduced them
- retention, rebuild debt, maintenance scheduling, and tier placement remain
  owned by their prior milestones
- compatibility changes admission, rebuild, reject, and reporting posture; it
  does not redefine truth semantics
- compatibility checks remain bounded by declared family/window/batch scope and
  do not scale with artifact rows after admission receipts exist
- later replication, extension families, blobs, and operator tooling inherit one
  compatibility model

## Acceptance Evidence

Milestone 12 is complete only when the store satisfies the named Milestone 12
suite:

- `Artifact Format Evolution And Rolling Compatibility Test`

Required machine-checkable outputs:

- `artifact_digest`
- `failure_digest`
- `compatibility_matrix`
- `version_skew_report`
- `diagnostics_digest`
- `counter_snapshot`

Minimum certification matrix rows:

- `native_control`
  current reader, current writer, native current artifacts
- `old_authority_new_reader_forward_compatible`
  old authoritative artifacts accepted by new reader through declared
  `ForwardCompatible` edge
- `new_authority_old_reader_backward_compatible`
  new authoritative artifacts accepted by old reader only through declared
  `BackwardCompatible` edge
- `new_authority_old_reader_partial_truth_rejected`
  new authoritative artifact decodes but contains meaning the old reader cannot
  interpret
- `numeric_proximity_without_edge_rejected`
  versions appear close by number or name but no declared compatibility edge
  exists
- `manifest_gap_rejected`
  artifact exists without the manifest required to interpret it
- `manifest_digest_mismatch_rejected`
  artifact and manifest both decode but their digest binding disagrees
- `hot_read_receipt_reuse`
  repeated reads in one family/window batch reuse admission receipts and do not
  re-resolve compatibility relations per artifact row
- `compatibility_index_manifest_bounded`
  index reconstruction touches manifest entries and declared edges, not stored
  artifact rows
- `batch_adapter_hot_path_rejected`
  a batch-class adapter is rejected from an authoritative hot read path
- `derived_reuse_accepted`
  derived family remains reusable through exact declared basis compatibility
- `derived_invalidated`
  derived family decodes but basis or semantic version drift requires discard
- `derived_rebuild_required`
  exact derived family rebuilds from retained compatible authority and records
  rebuild debt if deferred
- `bulk_resume_skew_rejected`
  Milestone 9 bulk progress or chunk records cannot resume under a different
  interpretation than the control lane
- `maintenance_rebuild_admitted`
  compatibility rebuild work enters Milestone 11 admission with matching
  counters
- `tier_manifest_skew_rejected`
  Milestone 13 residency or transfer records decode but are incompatible with
  the reader's placement manifest window
- `rolling_upgrade_two_capability_admitted`
  one old reader capability set and one new writer capability set are admitted
  by the first-ship rolling policy
- `rolling_upgrade_multi_writer_rejected`
  unsupported multi-writer rolling window rejects typed rather than guessing
- `restore_safe_window_published`
  restore publishes only after all authoritative families pass compatibility
  planning
- `restore_scope_bounded`
  restore compatibility inspects backup-scope families plus declared publication
  conflicts and does not scan unrelated target-store families
- `restore_unsafe_window_rejected`
  restore refuses publication before branch heads, cursors, schema support,
  lineage support, tier manifests, or snapshots become visible
- `disaster_recovery_truth_only`
  DR lane restores trusted authority while rejecting or invalidating incompatible
  derived acceleration

Milestone-specific proof obligations:

- every first-ship artifact family has manifest, read/write admission, restore
  posture, counters, and certification coverage
- decoded artifacts remain quarantined until compatibility witnesses admit
  semantic access
- compatible old artifacts read by new code preserve authoritative meaning
- incompatible new artifacts read by old readers fail typed before partial truth
  exposure
- numeric version proximity does not imply compatibility without a declared
  compatibility edge
- deterministic adapters, if shipped, are digest-bound and parity-proven
- hot read paths consume admission receipts and do not repeat relation checks per
  artifact row
- compatibility indexes rebuild from manifests and registry edges, not artifact
  inventory
- adapter cost classes reject broad adapters from hot reads and route scheduled
  adapters through maintenance
- restore planning stays scoped to backup families and declared publication
  conflicts
- mixed-version store lanes are admitted only inside declared reader/writer
  capability windows
- mixed-version replica lanes report explicit compatibility posture for later
  Milestone 14 consumption
- derived artifacts are invalidated, rebuilt, or rejected exactly where version
  drift requires
- compatibility rebuild work enters the Milestone 11 scheduler boundary
- backup and restore lanes reject unsafe version windows before publication
- disaster-recovery lanes distinguish truth-restorable from
  derived-acceleration-restorable scopes
- manifest publication and recovery lanes reject gaps and digest drift before
  artifact meaning is exposed
- deserialization-success hostile lanes do not count as compatibility success
- exact counters prove compatibility breadth and typed rejection behavior

Milestone 12 is not closed by "old files open" or "new files deserialize" tests.

## Architectural Notes

- The smart abstraction is not "migration." The smart abstraction is one
  compatibility authority model layered over already-classified durable artifact
  families.
- Compatibility is about semantic meaning, not only bytes. A byte reader that
  does not understand meaning must reject.
- Quarantine decoded records aggressively. The only safe decoded artifact is one
  that still cannot be used semantically until compatibility admission produces
  a witness.
- Avoid numeric-version comfort. Compatibility is a declared relation with proof,
  not a property of numbers looking close.
- The first-ship catalog matters. A compatibility system that covers canonical
  commits but misses maintenance summaries, tier manifests, or bulk resume
  records will fail during exactly the kind of rolling upgrade this milestone is
  supposed to survive.
- Derived-family compatibility is a reuse claim. Reuse without declared basis
  compatibility is shadow authority.
- Rolling upgrade support is a store contract, not a deployment checklist.
- Compatibility rebuild work should use Milestone 11 maintenance containers and
  Milestone 10 rebuild debt. Milestone 12 defines why the work is required; it
  does not own general scheduling or retention policy.
- Placement records from Milestone 13 must be versioned and checked, but tier
  residency never converts artifact meaning.

## Sequencing Notes

This milestone belongs after Milestone 10 and Milestone 11 because artifact
compatibility cannot honestly rebuild, invalidate, or schedule derived work
until retained authority, rebuild debt, and maintenance admission are already
explicit.

- Milestone 12 should consume Milestone 7 and Milestone 8 support/basis
  vocabulary so schema, lineage, cursor, checkpoint, and live-query
  compatibility remain grounded in durable support truth.
- Milestone 12 should consume Milestone 13 placement records now that tiering is
  closed, but it must preserve placement as cost-only posture.
- Milestone 14 should wait for this milestone because replication and capsules
  spread artifacts across machines; without explicit compatibility manifests
  and rolling-version rejection, replication would export ambiguity.
- Milestone 15 should inherit this milestone's compatibility contracts for
  extension-defined derived families.
- Milestone 20 should inherit this milestone's compatibility posture for blob
  manifests and tiered blob placement.
- Milestone 22 should inherit compatibility matrices and version-skew reports
  for operator repair, audit, and forensic recovery.
