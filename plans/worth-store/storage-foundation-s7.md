# Storage Foundation S.7 Engineering Spec: Native Blob/Object Chunk Store

## Goal

Make blobs native physical database objects: chunk-tree addressed, streaming,
checksummed, scope-preserving, resumable, deduped, retention-safe, and
placement-aware on the same Store physical foundation as pages, WAL,
checkpoints, manifests, and recovery evidence.

## Why This Milestone Exists

Roadmap 2 makes native blobs a physical foundation requirement before the
product-facing blob milestone expands the API surface. If S.7 treats blob bytes
as sidecar files with metadata in Store, later replication, backup, PITR,
repair, tiering, budgets, and certification will inherit a second storage
system that cannot share Store's integrity, memory, security-scope, I/O, and
recovery laws.

S.7 therefore turns the existing S.5 future chunk stability placeholders, S.5.1
blob security-scope readiness, S.6 blob I/O readiness handoffs, and S.4.5
simulation harness extension slots into real blob lifecycle authority.

## Governing Summaries

- `MENTALITY.md` protects adversarial-first foundation design. S.7 must solve
  multi-GB streaming, interruption, dedupe, corruption, and reclaim first,
  before ergonomic blob APIs can hide the hard path.
- `arch_laws.md` protects proof-bearing phase transitions and authority
  preservation. S.7 must expose sealed proof types for raw bytes, chunk
  candidates, scoped chunks, chunk-tree roots, resumable sessions, reachability,
  placement, and lifecycle receipts.
- `composition_laws.md` protects named semantic steps. S.7 code and tests must
  split chunk identity, streaming, digesting, dedupe admission, reachability,
  placement, export/import, and certification instead of growing a single blob
  service or one giant test harness file.
- `domain_structure_laws.md` protects responsibility-shaped crate topology.
  S.7 must keep Store-owned blob lifecycle law in lower production crates, keep
  certification as the courtroom, and keep test support at the narrowest
  reusable simulation boundary.
- `perf_laws.md` protects visible cost and memory honesty. S.7 must publish
  exact chunk, byte, allocation, residency, dedupe, reachability, placement,
  and reclaim counters, with tests that fail if whole-object materialization
  sneaks in.
- `physical-database-roadmap.md` protects the physical database foundation.
  Roadmap 2 requires S.7 to make multi-GB blobs native, constant-memory,
  content-addressed, secure-scope-preserving, partially replicable, and
  retention-safe before S.8 layout discipline and later S.10/S.11/S.12 work.

## Adversarial Constraint

An admitted multi-GB blob must be ingested, read, verified, resumed after
interruption, deduped, exported, imported, tier-moved, partially replicated,
corrupted, and reclaimed through Store-owned chunk lifecycle authority without
whole-object memory residency, without a sidecar storage system, without
digest-only authority, and without losing S.5.1 tenant/key/authenticity/custody
scope.

## Product Decision Lock

- S.7 is a physical Store foundation milestone, not the product blob API.
- Blob bytes are Store physical objects, not files beside Store with metadata
  inside Store.
- Chunk digest equality is never sufficient authority for dedupe across
  security scopes.
- Placement changes where chunks live; it does not change blob identity,
  security scope, authenticity, or retention truth.
- S.7 may decide chunk size, chunk-tree fanout, chunk metadata shape,
  placement class vocabulary, streaming path counters, and blob-local
  compaction rules. S.8 owns global artifact layout families, access-path
  indexing, physical locality strategy, cross-artifact layout policy, and the
  layout optimization/cost model.
- Certification proves S.7 law with hostile scenarios; certification does not
  own the law.
- JSON, serde, terminal projections, imported manifests, and CLI summaries are
  declarations or hostile inputs only. They must be readmitted before producing
  blob witnesses.

## Blob Identity And Generation Semantics

S.7 must define blob identity positively, not only by saying what digest,
chunk-tree root, and receipts cannot do.

- `BlobObjectId` is the Store durable object identity for one blob object.
- `BlobGeneration` is one published physical generation of that blob object.
- `ChunkTreeRoot` is the physical byte-layout identity for one generation.
- `LogicalContentDigest` is the derived digest of the canonical logical
  plaintext byte stream for one generation.
- `StoredChunkDigest` is the derived digest of the stored chunk-frame bytes
  under the admitted chunking/storage rule.
- `AuthenticatedFrameDigest` is reserved for S.11 authenticated/encrypted frame
  evidence and must remain distinct from logical content identity.
- `LifecycleReceipt` proves that a lifecycle stage executed. It is not blob
  identity.

Blob objects are versioned by generation. Published generations are immutable.
Changing blob bytes, chunking rule, transform posture, or authoritative/derived
classification produces a new `BlobGeneration` for the same `BlobObjectId` or
a new `BlobObjectId` when object identity itself changes. Product-level
semantic references may point to a blob object/generation pair, but relational
semantic references are not physical blob identity.

Authoritative and derived blobs are distinct classifications:

- `AuthoritativeBlob` is a primary durable artifact whose bytes are
  source-of-truth inside Store's physical foundation.
- `DerivedBlob` is rebuildable from other authority and may carry different
  retention, repair, export, and corruption handling posture.

Derived blob corruption may be repaired by rebuild when the rebuild source is
admitted and current. Authoritative blob corruption requires restore, repair,
quarantine, or typed degraded-truth reporting. Dedupe between authoritative and
derived blobs requires explicit policy because sharing storage must not erase
authority classification.

## Blob Publication Atomicity

S.7 publication is a named atomic protocol, not a side effect of bytes existing
on disk. The only ordinary publication event is `BlobGenerationPublished`.

The publication protocol is:

1. chunk bytes written
2. chunk checksums admitted
3. chunk-tree nodes durable
4. root candidate formed
5. reachability edges staged
6. blob generation publication record durably committed
7. semantic visibility handoff emitted, if a semantic owner consumes it
8. resume session closed

Chunks may exist before publication, but they are not reachable blob content.
Root candidates may exist before publication, but they are not visible blob
generations. Reachability edges staged before publication must not be consumed
as live reachability until the publication record commits. A crash at any point
must recover into one of: resume, deny, quarantine, reclaim-abandoned, or fully
published generation.

## WAL, Checkpoint, Manifest, And Recovery Integration

S.7 blob facts must live inside Store's physical durability model.

- Chunk byte writes produce Store physical write evidence and integrity
  evidence.
- Chunk-tree node writes are Store physical metadata writes.
- Resume checkpoints are WAL-backed or checkpoint-backed Store recovery facts.
- Blob generation publication records are WAL/recovery replay facts and
  checkpoint materialization inputs.
- Reachability edges and placement observations are manifest-visible physical
  metadata.
- Recovery replays publication records and resume checkpoints; it does not
  trust unreferenced backend residue as blob content.

Recovery must distinguish chunk bytes present without integrity admission,
integrity admission without a durable frontier, durable frontier without root
candidate, root candidate without generation publication, generation
publication without closed session, and closed session with orphan residue.

## Chunk Transform And Digest Ordering

S.7 stores canonical raw byte-stream chunks. Compression is not implemented in
S.7. If a later transform layer adds compression, it must enter as an explicit
transform posture before stored-frame digest derivation and must not change the
S.7 logical-content digest contract silently.

S.7 reserves transform ordering explicitly:

1. logical plaintext byte stream
2. canonical chunking rule
3. chunk integrity framing
4. stored chunk frame bytes
5. later S.11 authentication/encryption frame evidence

`LogicalContentDigest`, `StoredChunkDigest`, and `AuthenticatedFrameDigest`
must remain separate. S.7 dedupe and content identity are defined over admitted
logical content plus scope/dedupe policy. Stored-frame digests verify stored
bytes. Authenticated frame digests remain S.11 posture and cannot be guessed by
S.7.

The chunking rule version must name fixed chunk size, tail behavior,
content-defined algorithm posture, minimum/maximum chunk size when relevant,
rolling-hash parameters when relevant, transform ordering, canonical ordering,
and digest algorithm slots.

## Universal No-Whole-Object-Materialization Rule

No S.7 certification path may require materializing the full logical blob in
heap, one scalar buffer, one temp sidecar file, or one expected-byte artifact.
This applies to ingest, read, export bundle assembly, import validation, digest
comparison, corruption fixture generation, offline verification, temporary
file staging, future transform buffers, and heavy multi-GB qualification.

## Counter Strength Model

S.7 counters must state their evidence strength.

- `ExactCounter`: required for lifecycle correctness, chunk counts, byte counts,
  allocation counts, resident memory peaks, publication records, reachability
  edges, dedupe references, reclaim operations, corruption localization, and
  heavy qualification.
- `MonotonicCounter`: allowed where the value only needs to prove non-decrease,
  such as cumulative bytes streamed across one executed session.
- `SampledCounter`: allowed only for diagnostic pressure observation, never for
  lifecycle closeout, memory bounds, or publication authority.
- `DerivedCounter`: allowed when computed from exact counters with a named
  derivation basis.
- `DiagnosticCounter`: support-facing only and never authority.
- `CertificationOnlyCounter`: allowed for harness evidence when it is tied to
  executed production surfaces and cannot be consumed by runtime APIs.

Any counter used to prove constant memory, no whole-object materialization,
publication atomicity, reachability, dedupe sharing, reclaim, corruption
localization, or multi-GB execution must be exact unless the phase explicitly
names a weaker strength and why that weaker strength cannot affect authority.

## Concrete Foundational And Proof Usage

S.7 must use `worth-foundational` only where Store is crossing a shared
boundary, producing canonical/export evidence, carrying profile/support truth,
or reporting counter-backed performance evidence. Store-owned blob lifecycle
authority remains in Store crates.

Required Foundational surfaces:

- `worth_foundational::aspects()` for aspect-native declarations at import,
  export, evidence, and support boundaries.
- `worth_foundational::compatibility().json()` only for hostile/readmission
  tests that prove JSON cannot become authority without native lowering.
- `worth_foundational::canonicalization()` plus
  `worth_foundational::canonicalization_api::{common_path, lower_lane,
  stronger_lane}` for chunk-tree canonical basis, export bundles, digest
  derivation, comparison, boundary bridging, and readmission.
- `worth_foundational::boundary_evidence()` plus
  `worth_foundational::boundary_evidence_api::{common_path, lower_lane,
  stronger_lane}` for executed receipts, completed receipts, provenance,
  lineage, support attachments, attachment bundles, and readmitted support
  evidence.
- `worth_foundational::performance_api::{common_path, lower_lane,
  stronger_lane}` for descriptive performance claims, policy-admission
  receipts, canonical performance bundles, counter-backed performance receipts,
  report materialization, certification, and readmitted performance bundles.
- `worth_foundational::profiles()` and `worth_foundational::profiles_api` for
  profile, support posture, compatibility posture, materialization posture, and
  proof-bearing artifact profile attachment.

Required Proof surfaces:

- `use worth_proof::prelude::*;` for the ordinary S.7 progression lane.
- `recipe(...)`, `proof_flow()`, `.resolve_with(...)`, `.lower_with(...)`,
  `.ready_with(...)`, `.execute()`, and checked variants such as
  `.try_resolve_ready(...)`, `.try_lower_ready(...)`, `.try_ready_now(...)`,
  and `.try_execute()` for blob declaration -> admitted scope -> lowered
  execution -> execution-ready -> executed lifecycle progression.
- `ProofOutcome` / `ProofOutcomeKind` and `TransitionOutcome` categories to
  preserve success, denial, deferred, stale, rebind-required, and failed
  outcomes without flattening into one error.
- `AuthorityWitness<_>` and `CapabilityWitness<_>` for Store-owned authority
  and capability transitions.
- `.bridge_trust_boundary()`, `.rebind_with(...)`, and `.readmit_with(...)`
  for export/import and restored evidence crossing trust boundaries.
- `pair(...)`, `non_empty(...)`, `join_ready(...)`, and `compose_ready(...)`
  only where S.7 has fixed-shape proof composition, such as chunk plus scope,
  chunk-tree plus reachability, or lifecycle plus placement readiness.
- `worth_proof::raw::*` only when the pleasant lane would hide a real
  adversarial proof boundary.

## Phase Plan

### Phase 1: Blob Lifecycle Authority Boundary

Phase 1 freezes the noun system and public authority boundary for native blob
objects before any streaming path can claim completion.

**Relevant subsystems**
- `worth-store-blob-chunks`
- `worth-store-readiness`
- `worth-store-authority`
- `worth-store-claim-boundaries`

**Relevant APIs**
- `S7BlobChunkSecurityHandoff`
- `S6ClosedS7PlacementAdmissionSeed`
- `S7PlacementIoReadinessSeed`
- existing `BlobChunkIdentity`, `BlobChunkSecurityScope`, lifecycle receipt,
  and non-claim surfaces
- `worth_proof::prelude::*` with `recipe(...)`, `.resolve_with(...)`,
  `.lower_with(...)`, `.ready_with(...)`, `.execute()`, `ProofOutcomeKind`,
  `AuthorityWitness<_>`, and `CapabilityWitness<_>` for the phase progression
  skeleton
- `worth_foundational::boundary_evidence()` and
  `worth_foundational::boundary_evidence_api::lower_lane::receipts` for the
  eventual executed and completed receipt vocabulary carried by closeout

**Warnings**
- Do not make `BlobChunkIdentity` or a digest string equivalent to blob
  lifecycle authority.
- Do not let S.3 chunk integrity reports satisfy S.7 reachability,
  resumability, retention, or dedupe receipts.
- Do not expose raw-field constructors for lifecycle receipts.

**Test requirements**
- Adversarial equivalence: the same admitted chunk lifecycle replay produces
  the same sealed lifecycle boundary artifacts and counters across repeated
  certification runs.
- Adversarial denial: copied digest strings, copied counters, S.3 integrity
  reports, terminal projection rows, and imported manifest text cannot
  construct S.7 lifecycle receipts.
- Compile-fail: external crates cannot construct scoped chunk, lifecycle,
  reachability, or placement proof types from raw fields.

**Engineering decisions**
- Define S.7 public facades around capability construction, not raw structs.
- Use private fields and sealed constructors for every type that claims a
  proven lifecycle stage.
- Keep Store-specific lifecycle authority in `worth-store-blob-chunks`; use
  `worth-foundational` and `worth-proof` only for shared vocabulary and proof
  reporting.
- Model the lifecycle as Proof progression: raw declaration -> resolved Store
  authority -> lowered chunk lifecycle plan -> execution-ready admitted work ->
  executed receipt. Store types carry semantic law; Proof carries the
  progression law.
- Materialize Foundational boundary evidence only after Store has produced the
  executed lifecycle receipt.

**Open questions**
- None.

### Phase 2: Security-Scope Admission And Chunk Metadata Law

Phase 2 promotes the S.5.1 blob security handoff into mandatory chunk metadata
before chunk identity, streaming, dedupe, placement, export, import, or reclaim
can rely on it.

**Relevant subsystems**
- `worth-store-blob-chunks`
- `worth-store-security`
- `worth-store-readiness`
- `worth-store-authority`

**Relevant APIs**
- `S7BlobChunkSecurityHandoff`
- `BlobChunkSecurityScope`
- S.5.1 key scope, key version, tenant scope, authenticity class, and custody
  posture readiness surfaces
- `worth_foundational::aspects()` for aspect-native security-scope evidence
  at boundary materialization
- `AspectKey`, `AspectValue`, `StructAspectValue`,
  `ContractValidatedAspectValue`, `AuthoritativeRecordAspectStateAdmitted`,
  `ProjectionMask`, `MutationMask`, and `DiagnosticMask`
- `worth_foundational::canonicalization_api::lower_lane::basis` for
  canonical security-scope basis entries used in evidence bundles
- `worth_proof::prelude::{AuthorityWitness, ProofOutcomeKind}` for
  authority-gated scope admission

**Warnings**
- JWT subject, application organization id, IAM role, KMS key id, operator id,
  terminal projection label, and raw string tenant labels are not blob security
  scope.
- Security scope must not be reconstructed from serialized chunk metadata.
  Deserialization may create declarations only; readmission creates witnesses.
- Derived blobs and authoritative blobs may share streaming mechanics but not
  authority classification.

**Test requirements**
- Adversarial preservation: every admitted chunk proof carries key scope, key
  version, tenant scope, authenticity class, and custody posture through the
  phase-local witness.
- Adversarial denial: wrong tenant, wrong key scope, stale key version,
  unsupported custody posture, identity-provider claims, and deserialized
  metadata declarations fail before chunk witness construction.
- Compile-fail: callers cannot build blob chunk security scope from raw
  strings, JWT claims, KMS ids, IAM role names, or operator identities.

**Engineering decisions**
- Every S.7 chunk witness must carry or reference an admitted S.5.1 blob
  security-scope witness.
- Use distinct types for tenant scope, key scope, key version, authenticity
  class, custody posture, authoritative blob classification, and derived blob
  classification.
- Author native scope evidence through `aspects()` when S.7 materializes
  shared boundary evidence; do not route native scope through
  `compatibility().json()`.
- If JSON appears in tests, force it through `compatibility().json()` and prove
  it lands as a declaration that still requires Store readmission.

**Open questions**
- None.

### Phase 3: Chunk Byte Identity, Checksum, And Physical Integrity

Phase 3 defines the physical chunk byte unit, chunk id, checksum slot, chunking
rule version, and per-chunk integrity proof before any chunk tree or stream can
publish a blob root.

**Relevant subsystems**
- `worth-store-blob-chunks`
- `worth-store-physical-integrity`
- `worth-store-aspect-native`
- `worth-store-physical-format`

**Relevant APIs**
- `BlobChunkIdentity`
- S.3 chunk integrity reports
- Foundational canonical basis and digest vocabulary
- Store physical frame and chunk format vocabulary
- `worth_foundational::canonicalization()`
- `worth_foundational::canonicalization_api::lower_lane::{basis, comparison,
  digest}`
- `CanonicalBasisReadyArtifact`, `CanonicalBasisEntry`,
  `CanonicalDigestAlgorithmSlot`, `CanonicalDigestDerivationReadyArtifact`,
  `CanonicalDerivedDigest`, `CanonicalComparisonOutcome`
- `prepare_canonical_basis_sequence(...)`,
  `prepare_canonical_basis_bundle(...)`, `derive_canonical_digest(...)`, and
  `compare_canonical_basis(...)`
- `worth_proof::prelude::{pair, non_empty, join_ready}` for fixed-shape
  composition of chunk bytes, checksum evidence, and scope evidence

**Warnings**
- A checksum proves local byte integrity under a format rule; it is not
  authenticity, dedupe authority, or full-blob identity by itself.
- A full-content digest identifies a logical byte stream only after ordered
  chunk-tree proof; it must not erase chunk boundaries.
- Identical byte content across different security scopes must remain
  physically distinguishable until explicit dedupe admission.

**Test requirements**
- Adversarial equivalence: chunking the same stream with the same admitted
  rule produces the same chunk identities, chunk checksums, chunk-tree root,
  and full-content digest.
- Adversarial denial: reordered chunks, duplicated middle chunks, missing tail
  chunks, checksum-only evidence, and digest-only evidence fail before blob
  lifecycle publication.
- Collision lane: a forced digest-equivalence fixture still requires
  chunk-by-chunk verification and scope admission before dedupe.

**Engineering decisions**
- Store chunk checksums, content digests, canonical chunking rule version, and
  chunk-tree root as separate types.
- Make chunk-tree root publication consume ordered scoped chunk proofs, not raw
  byte buffers or unscoped digests.
- Record exact counters for bytes chunked, chunks emitted, checksums computed,
  digest updates, and chunk-tree nodes materialized.
- Use Foundational canonicalization for exported canonical basis and digest
  evidence only. Internal chunk-tree authority remains Store-owned.
- Digest derivation must consume canonical-basis-ready S.7 chunk-tree evidence;
  it must not accept raw digest strings or checksum rows.

**Open questions**
- Decide the initial canonical chunk size classes and whether any content
  defined chunking mode belongs in S.7 or later product expansion.

### Phase 4: Chunk Tree Root, Content Digest, And Canonical Basis

Phase 4 builds the ordered chunk-tree root and full-content digest from scoped
chunk proofs, then materializes Foundational canonical evidence for boundary
reporting.

**Relevant subsystems**
- `worth-store-blob-chunks`
- `worth-store-aspect-native`
- `worth-store-physical-format`
- `worth-store-physical-integrity`

**Relevant APIs**
- scoped chunk proof surfaces from Phase 3
- `worth_foundational::canonicalization()`
- `worth_foundational::canonicalization_api::lower_lane::{basis, comparison,
  digest}`
- `CanonicalBasisReadyArtifact`, `CanonicalBasisEntry`,
  `CanonicalDigestAlgorithmSlot`, `CanonicalDigestDerivationReadyArtifact`,
  `CanonicalDerivedDigest`, `CanonicalComparisonOutcome`
- `prepare_canonical_basis_sequence(...)`,
  `prepare_canonical_basis_bundle(...)`, `derive_canonical_digest(...)`, and
  `compare_canonical_basis(...)`
- `worth_proof::prelude::{non_empty, join_ready, compose_ready}` for ordered
  non-empty chunk-tree proof composition

**Warnings**
- A full-content digest identifies a logical byte stream only after ordered
  chunk-tree proof; it must not erase chunk boundaries.
- Digest derivation is derived compression, not Store authority.
- Identical byte content across different security scopes must remain
  distinguishable until explicit dedupe admission.

**Test requirements**
- Adversarial equivalence: the same ordered scoped chunks produce the same
  chunk-tree root, content digest, canonical basis, and comparison outcome.
- Adversarial denial: reordered chunks, duplicated middle chunks, missing tail
  chunks, checksum-only evidence, digest-only evidence, and empty chunk sets
  fail before root publication.
- Collision lane: forced digest-equivalence fixtures still require
  chunk-by-chunk canonical comparison and scope admission before dedupe can be
  considered.

**Engineering decisions**
- Make chunk-tree root publication consume ordered scoped chunk proofs, not raw
  byte buffers or unscoped digests.
- Use Foundational canonicalization for exported canonical basis and digest
  evidence only. Internal chunk-tree authority remains Store-owned.
- Digest derivation must consume canonical-basis-ready S.7 chunk-tree evidence;
  it must not accept raw digest strings or checksum rows.

**Open questions**
- None.

### Phase 5: Blob Object Identity And Generation Registry

Phase 5 introduces `BlobObjectId`, `BlobGeneration`, authoritative/derived blob
classification, and semantic-reference handoff boundaries before publication,
reachability, export, or replication can refer to "the blob."

**Relevant subsystems**
- `worth-store-blob-chunks`
- `worth-store-authority`
- `worth-store-readiness`
- `worth-store-claim-boundaries`
- `worth-store-physical-format`

**Relevant APIs**
- `BlobObjectId`
- `BlobGeneration`
- `AuthoritativeBlob`
- `DerivedBlob`
- `ChunkTreeRoot`
- `LogicalContentDigest`
- `StoredChunkDigest`
- `LifecycleReceipt`
- Foundational identity/canonicalization surfaces for boundary evidence
- `worth_proof::prelude::{recipe, ProofOutcomeKind, AuthorityWitness}`

**Warnings**
- Same bytes do not imply same blob object.
- Same chunk-tree root does not imply same blob generation unless the Store
  generation registry binds it.
- A semantic object reference from `worth-relational` may point at Store blob
  identity, but it is not Store blob identity.
- Derived blob rebuildability must not weaken authoritative blob retention or
  repair rules.

**Test requirements**
- Adversarial equivalence: two observations of the same published
  `BlobObjectId`/`BlobGeneration` resolve to the same chunk-tree root, logical
  digest, authority classification, and lifecycle receipt.
- Adversarial denial: digest equality, chunk-tree equality, copied lifecycle
  receipts, semantic reference ids, and raw generation numbers cannot construct
  blob identity.
- Classification lane: derived blob corruption can select rebuild posture only
  when admitted rebuild authority exists; authoritative blob corruption cannot
  silently downgrade into derived repair.

**Engineering decisions**
- Published generations are immutable. Updates create new generations.
- `BlobObjectId` and `BlobGeneration` must be distinct types even if their
  representation is identical to another id/generation type.
- The generation registry owns the binding from blob object/generation to
  chunk-tree root, digest evidence, classification, publication receipt, and
  current visibility state.

**Open questions**
- Decide whether initial `BlobObjectId` allocation lives in
  `worth-store-blob-chunks` directly or behind a narrower authority module.

### Phase 6: Blob Publication Atomicity And Visibility Commit

Phase 6 implements the atomic publication protocol that turns durable chunks
and a root candidate into one visible immutable blob generation.

**Relevant subsystems**
- `worth-store-blob-chunks`
- `worth-store-wal`
- `worth-store-recovery-physics`
- `worth-store-physical-isolation`
- `worth-store-readiness`

**Relevant APIs**
- `BlobGenerationPublished`
- root candidate publication surfaces
- reachability edge staging surfaces
- resume session closeout surfaces
- WAL publication record surfaces
- `worth_proof::prelude::{proof_flow, ProofOutcomeKind, join_ready}`

**Warnings**
- Chunk bytes durable is not publication.
- Chunk-tree root candidate is not publication.
- Staged reachability is not live reachability until the generation
  publication record commits.
- Semantic visibility handoff must happen after physical generation
  publication, not before it.

**Test requirements**
- Adversarial atomicity: crashes after chunk write, checksum admission,
  chunk-tree node durability, root candidate formation, reachability staging,
  publication record write, and session close each recover to exactly one
  typed state.
- Adversarial denial: root candidates, staged reachability rows, copied
  publication records, and semantic references cannot make a blob visible
  without `BlobGenerationPublished`.
- Visibility lane: semantic handoff consumers see either the previous
  generation or the newly published generation, never a partial generation.

**Engineering decisions**
- Define `BlobGenerationPublished` as the ordinary atomic publication event.
- Bind publication to Store authority, replay identity, root candidate,
  reachability staging identity, security scope, generation id, and counter
  receipt identity.
- Recovery must replay publication records, not infer publication from residue.

**Open questions**
- None.

### Phase 7: Blob WAL, Checkpoint, Manifest, And Recovery Records

Phase 7 makes blob lifecycle facts explicit members of Store's WAL,
checkpoint, manifest, and recovery vocabulary instead of backend-private
residue.

**Relevant subsystems**
- `worth-store-blob-chunks`
- `worth-store-wal`
- `worth-store-recovery-physics`
- `worth-store-physical-format`
- `worth-store-physical-backend`

**Relevant APIs**
- chunk write evidence
- chunk-tree metadata write evidence
- resume checkpoint records
- blob generation publication records
- blob reachability manifest rows
- blob placement manifest rows
- recovery replay receipts

**Warnings**
- Recovery must not trust orphan files, object-store keys, backend paths, or
  unreferenced chunk bytes as blob content.
- Resume checkpoints must be replayable Store facts, not process-local upload
  state.
- Checkpointing a root without publication and reachability state creates
  ambiguous recovery and is forbidden.

**Test requirements**
- Adversarial replay: WAL/checkpoint replay reconstructs published blob
  generations, resume sessions, reachability edge staging, and placement
  observations without scanning backend residue as authority.
- Adversarial denial: chunk bytes without integrity admission, integrity
  without frontier checkpoint, frontier without root candidate, root candidate
  without publication, and publication without manifest agreement produce
  distinct typed recovery outcomes.
- Manifest lane: reachability and placement manifests detect missing external
  chunks, stale generation rows, and orphaned placement residue.

**Engineering decisions**
- Chunk writes produce Store physical write evidence and integrity evidence.
- Chunk-tree nodes are Store metadata writes.
- Blob generation publication records are WAL/recovery replay facts and
  checkpoint materialization inputs.
- Reachability edges and placement observations are manifest-visible physical
  metadata.

**Open questions**
- Decide the exact WAL record families for chunk append, root candidate,
  publication, and session checkpoint records.

### Phase 8: Blob Harness Skeleton, Profiles, And Shortcut Taxonomy

Phase 8 extends the S.4.5 harness early with blob scenario profile vocabulary,
shortcut denials, and local/CI/heavy profile identity before implementation
phases rely on bespoke tests.

**Relevant subsystems**
- `worth-store-physical-certification`
- `worth-store-test-support`
- `worth-store-blob-chunks`
- `worth-store-budgets`

**Relevant APIs**
- S.4.5 scenario definition, plan, lowering, transcript, replay bundle, and
  oracle family surfaces
- `worth_foundational::profiles()` and
  `worth_foundational::profiles_api::{common_path, lower_lane,
  stronger_lane::readiness}`
- `FoundationalProfileIdentity`, `MaterializedFoundationalProfileArtifact`,
  profile compatibility and materialization posture surfaces
- `worth_proof::prelude::{proof_flow, ProofOutcomeKind}` for scenario plan ->
  lowered driver -> execution-ready -> executed evidence progression

**Warnings**
- Do not wait until closeout to design blob harness topology.
- Do not create a separate blob harness outside S.4.5.
- This phase defines only profile taxonomy, shortcut taxonomy, scenario
  identity shape, and counter topology. It must not prebuild production actors,
  fault injection, oracle families, or coverage matrices before the production
  lifecycle surfaces exist.
- Scenario summaries, logs, and same-run self-comparison are not proof.

**Test requirements**
- Adversarial replay: a seed blob scenario lowers into a stable S.4.5 plan
  shape and replay identity before production blob operations exist.
- Adversarial denial: tiny blobs, whole-object helpers, missing chunk counters,
  logs-as-proof, synthetic success rows, and private mutation of harness state
  fail as shortcut lanes.
- Profile lane: local, CI memory-envelope-exceeding, and heavy multi-GB
  profiles share one profile taxonomy and counter topology.

**Engineering decisions**
- Add blob fixture/profile vocabulary early: blob size class, memory envelope,
  chunk count, chunk size class, placement class, security scope, access mode,
  failure point, and actor mix.
- Attach Foundational profiles to scenario families so test scale is explicit
  materialization posture rather than runner folklore.
- Expose only stable scenario identity and counter-topology hooks here. Phase
  22 fills actors, faults, oracles, and coverage with production surfaces.

**Open questions**
- Decide the first profile names and which profiles are mandatory in local CI.

### Phase 9: Constant-Memory Streaming Ingest

Phase 9 establishes the ordinary bounded-memory write path for blob ingest,
including allocation, residency, chunk-write, and S.6 I/O pressure evidence.

**Relevant subsystems**
- `worth-store-blob-chunks`
- `worth-store-buffer-pool`
- `worth-store-budgets`
- `worth-store-io-scheduler`
- `worth-store-physical-backend`

**Relevant APIs**
- blob streaming residency proof surfaces
- S.2 resident-byte and allocation budget surfaces
- S.6 foreground reservation and blob pressure handoffs
- physical backend write capability surfaces
- `worth_foundational::performance_api::common_path::performance()`
- `worth_foundational::performance_api::lower_lane::{policy, receipts,
  reports}`
- `FoundationalPolicyAdmissionReceipt`,
  `FoundationalCounterBackedPerformanceReceipt`,
  `FoundationalPerformanceCounterRow`, `FoundationalPerformanceContractName`,
  and `FoundationalPerformanceCounterSpec`
- `worth_proof::prelude::{ready_now, gate_ready, ProofOutcomeKind}` for
  admission and denial-preserving streaming readiness

**Warnings**
- A helper that receives `Vec<u8>` for the full blob is not a streaming ingest
  path.
- Local CI profiles may be smaller than multi-GB, but they must still exceed
  the configured memory envelope.
- Policy admission is not executed streaming proof.

**Test requirements**
- Adversarial equivalence: streaming ingest emits the same scoped chunk
  sequence, chunk counters, residency proof, and content frontier regardless
  of bounded window size.
- Adversarial denial: any path that materializes the full blob, exceeds the
  declared resident-byte or allocation envelope, hides chunk counters, or uses
  a scalar read/write API for certification fails.
- Pressure lane: foreground page/WAL writes remain admitted through S.6
  reservation while blob ingest pressure yields, paces, or denies as required.

**Engineering decisions**
- Implement bounded streaming windows and allocation scopes as first-class
  lifecycle objects.
- Publish `BlobStreamingResidencyProof` or equivalent only from executed
  streaming sessions.
- Counters must include peak resident bytes, allocation count, chunk-read,
  chunk-write, bytes-streamed, scheduler wait, and background-yield events.
- Use Foundational performance claims for declared memory and streaming
  contracts, then strengthen only executed sessions into
  `FoundationalCounterBackedPerformanceReceipt`.
- Preserve `Denied`, `Deferred`, and `Stale` streaming admission outcomes
  through Proof checked progression instead of collapsing them into one
  storage error.

**Open questions**
- Decide the default local memory envelope used by deterministic CI profiles.

### Phase 10: Streaming Read And Verification

Phase 10 establishes bounded-memory read, verify, and chunk traversal paths as
separate from ingest so read amplification, verification, and foreground read
protection can be tested directly.

**Relevant subsystems**
- `worth-store-blob-chunks`
- `worth-store-buffer-pool`
- `worth-store-io-scheduler`
- `worth-store-physical-integrity`
- `worth-store-physical-isolation`

**Relevant APIs**
- blob streaming read proof surfaces
- chunk-tree traversal surfaces
- S.5 read stability and future chunk movement barriers
- S.6 foreground reservation surfaces
- `worth_foundational::performance_api::lower_lane::receipts`
- `worth_proof::prelude::{join_ready, ProofOutcomeKind}` for chunk-tree plus
  read-stability readiness

**Warnings**
- Read verification must not reuse ingest-side expected buffers as proof.
- Chunk-tree traversal must not hide broad scans or whole-object buffering.
- Verification success is not dedupe admission or authenticity success.

**Test requirements**
- Adversarial equivalence: streaming read verifies byte equality, chunk order,
  chunk checksums, full-content digest, and exact read counters without
  depending on read buffer size.
- Adversarial denial: missing chunk, reordered chunk, corrupted chunk, stale
  read hold, unavailable cold chunk, or whole-object expected buffer fails
  before verified read publication.
- Foreground lane: point reads and page/WAL reads remain inside admitted S.6
  reservations while blob reads proceed or deny with typed pressure evidence.

**Engineering decisions**
- Keep read verification counters distinct from write counters.
- Expose read amplification, chunk-read, verification, scheduler-wait, and
  protected-read-denial counters.
- Preserve `Denied`, `Deferred`, and `Stale` outcomes through Proof checked
  progression rather than one storage error.

**Open questions**
- None.

### Phase 11: Chunk Corruption Localization And Quarantine State

Phase 11 makes corrupt chunks a first-class lifecycle state with distinct
behavior for read, scrub, dedupe, cold placement, import, capsule creation,
rebuild, repair, and reclaim.

**Relevant subsystems**
- `worth-store-blob-chunks`
- `worth-store-physical-integrity`
- `worth-store-physical-isolation`
- `worth-store-retention`
- `worth-store-offline-verifier`

**Relevant APIs**
- chunk integrity reports
- quarantine hold surfaces
- corrupted chunk locality reports
- derived blob rebuild posture surfaces
- authoritative blob degraded-truth posture surfaces
- `worth_foundational::boundary_evidence_api::lower_lane::{receipts, support}`
- `worth_proof::prelude::{ProofOutcomeKind, join_ready}`

**Warnings**
- Corruption detected during read, scrub, dedupe, cold fetch, import, or
  capsule creation may have different consequences and must not collapse into
  one error.
- A corrupt shared deduped chunk affects every admitted reference edge until
  localized, quarantined, rebuilt, or repaired.
- Quarantine is a hold and a visibility posture, not reclaim permission.

**Test requirements**
- Adversarial localization: corrupt chunk during read, scrub, cold fetch,
  import, and partial capsule materialization localizes to the chunk, blob
  generation, placement class, and affected reference edges.
- Adversarial denial: corrupt chunks cannot satisfy dedupe, export, import
  readmission, capsule readiness, or verified read publication.
- Classification lane: derived blob corruption may choose rebuild only with an
  admitted rebuild source; authoritative blob corruption enters quarantine,
  repair, restore, or degraded-truth reporting.

**Engineering decisions**
- Add explicit corruption states for detected-unquarantined, quarantined,
  rebuildable-derived, repair-required-authoritative, cold-unavailable-corrupt,
  and import-corrupt declarations.
- Quarantine holds participate in reachability and block reclaim until released
  by admitted repair/rebuild/restore authority.
- Corruption counters must include detection source, chunk id, blob generation,
  placement class, sharing scope, quarantine hold, rebuild eligibility, and
  export/import/capsule denial counts.

**Open questions**
- Decide whether derived blob rebuild execution belongs entirely in S.7 or is
  represented as readiness for a later product rebuild surface.

### Phase 12: Resumable Write Sessions And Interrupted Ingest Recovery

Phase 12 makes interrupted blob ingest an ordinary lifecycle path instead of an
exceptional cleanup problem.

**Relevant subsystems**
- `worth-store-blob-chunks`
- `worth-store-recovery-physics`
- `worth-store-wal`
- `worth-store-physical-isolation`
- `worth-store-physical-certification`

**Relevant APIs**
- resumable blob write receipt surfaces
- recovery replay receipt surfaces
- S.5 future chunk stability and reclaim barriers
- S.4.5 crash/fault scenario surfaces
- `worth_proof::prelude::{recipe, proof_flow, ProofOutcomeKind}`
- `.bridge_trust_boundary()`, `.rebind_with(...)`, and `.readmit_with(...)`
  where replay or restored resume evidence crosses a trust boundary
- `worth_foundational::boundary_evidence_api::lower_lane::{receipts,
  provenance, lineage}`
- `FoundationalBoundaryEvidenceExecutedReceiptArtifact`,
  `FoundationalBoundaryEvidenceCompletedReceiptArtifact`,
  `FoundationalBoundaryEvidenceProvenanceArtifact`, and
  `FoundationalBoundaryEvidenceReplayDerivedLineageArtifact`

**Warnings**
- A resumable session token is not blob authority; it is an admitted recovery
  handle for an unfinished write.
- Partially durable chunks must not become reachable merely because their bytes
  exist on disk.
- Cleanup of abandoned sessions must preserve corruption localization and
  forensic visibility.
- Resume must distinguish `SessionDeclared`, `SessionAdmitted`,
  `ChunkAppendStarted`, `ChunkBytesDurable`, `ChunkIntegrityAdmitted`,
  `FrontierCheckpointed`, `RootCandidateBuilt`, `RootPublicationReady`,
  `BlobPublished`, `SessionClosed`, `SessionAbandoned`, and
  `SessionReclaimed`.

**Test requirements**
- Adversarial replay: crash after chunk write, after session checkpoint, after
  chunk-tree node write, and before root publication; replay either resumes to
  the same final blob or denies with localized unfinished state.
- Adversarial denial: stale session ids, wrong security scope, copied resume
  checkpoints, missing chunk tails, and session checkpoints from another Store
  authority cannot resume or publish a blob.
- Orphan lane: abandoned partial chunks remain unreached and become reclaimable
  only through the S.7 reclaim proof path.
- State-machine lane: chunk bytes without checksum admission, checksum
  admission without durable frontier, durable frontier without root node, root
  node without reachability staging, and closed session with orphan chunks
  produce distinct recovery outcomes.

**Engineering decisions**
- Model resumable ingest as raw declaration -> admitted session -> executed
  chunk append -> checkpointed session -> publishable chunk tree -> closed
  blob receipt.
- Persist resume checkpoints with Store authority, replay identity, chunking
  rule, security scope, current chunk frontier, and counter receipt identity.
- Keep resume mechanics separate from final blob identity and reachability.
- Use Proof basis weakening for interrupted/replayed session evidence. A
  resumed session must regain current Store authority through explicit
  readmission before it can publish a chunk-tree root.
- Use Foundational boundary evidence for replay-derived lineage and closeout
  receipts, not as session authority.
- Store the resumable ingest state machine as typed states, not one mutable
  session struct with flags.

**Open questions**
- Decide whether unfinished sessions are indexed by upload id, chunk-tree
  frontier, or both.

### Phase 13: Dedupe Admission, Collision Handling, And Scope Barriers

Phase 13 admits dedupe as a Store-owned physical optimization with explicit
equivalence and security-scope barriers.

**Relevant subsystems**
- `worth-store-blob-chunks`
- `worth-store-security`
- `worth-store-budgets`
- `worth-store-physical-integrity`

**Relevant APIs**
- `BlobChunkDedupeCandidate`
- `BlobChunkCanonicalEquivalence`
- `BlobDedupeReceipt`
- S.5.1 security-scope readiness and rejection surfaces
- `worth_foundational::canonicalization_api::lower_lane::{comparison,
  digest}`
- `CanonicalEquivalenceBasis`, `CanonicalEquivalentBasis`,
  `CanonicalMismatchBasis`, `CanonicalMismatchKind`,
  `CanonicalComparisonOutcome`, and `CanonicalDerivedDigest`
- `worth_proof::prelude::{pair, join_ready, compose_ready, ProofOutcomeKind}`
  for fixed-shape chunk+scope and chunk+canonical-equivalence admission

**Warnings**
- Dedupe saves space; it does not merge tenant authority.
- A dedupe index is derived placement/reuse structure unless explicitly
  classified otherwise.
- Hash equality must never bypass chunk verification, scope admission, or
  collision handling.
- Dedupe policy modes are explicit: `NoDedupe`,
  `SameBlobGenerationOnly`, `SameTenantSameKeyScope`,
  `SameTenantDifferentKeyScopeWithExplicitPolicy`, `CrossTenantDenied`, and
  `CrossTenantExplicitlyAdmittedLater`.
- A dedupe receipt must bind into reachability/reference accounting; otherwise
  shared chunks can be reclaimed incorrectly.

**Test requirements**
- Adversarial equivalence: same-scope chunks with the same canonical chunking
  rule, verified bytes, admitted security scope, and canonical equivalence
  produce a dedupe receipt and exact dedupe-hit counters.
- Adversarial denial: cross-tenant, cross-key, stale-key-version,
  authenticity-mismatch, custody-mismatch, copied equivalence, and digest-only
  candidates cannot share physical blob claims.
- Collision lane: collision fixtures force byte comparison and produce either
  verified equivalence or collision denial with localized counters.
- Collision posture lane: forced digest collisions produce one of
  `VerifiedEquivalent`, `DigestCollisionDenied`,
  `DigestAlgorithmQuarantined`, `DedupeIndexPartitioned`, or
  `ChunkRewrittenUnderNewDigestBasis`.
- Reclaim lane: a deduped chunk cannot be reclaimed until every admitted
  reference edge across all admitted sharing scopes is absent or separately
  denied.

**Engineering decisions**
- Require a move-only dedupe candidate that consumes scoped chunk evidence and
  emits either a dedupe receipt or typed denial.
- Keep dedupe index maintenance behind the blob-chunk facade; do not expose
  index internals as public authority.
- Record chunk-level dedupe hits, misses, collision probes, byte-verify probes,
  cross-scope denials, and index updates.
- Use Foundational canonical comparison vocabulary to describe equivalence and
  mismatch evidence after Store has performed byte and scope checks.
- Do not let Foundational digest evidence serve as the dedupe admission input;
  it is reporting/comparison evidence, not Store dedupe authority.
- Reserve strongest-retention/security-wins semantics for any future admitted
  cross-scope dedupe. The shared chunk must preserve the strongest active hold
  among all admitted references.

**Open questions**
- Decide whether the first dedupe index is per-scope only, or whether explicit
  cross-scope policy can be admitted in the same milestone after denial lanes
  exist.

### Phase 14: Chunk Reachability And Reference Accounting

Phase 14 establishes reachable chunk authority and reference accounting before
any reclaim, export, or capsule can depend on liveness.

**Relevant subsystems**
- `worth-store-blob-chunks`
- `worth-store-retention`
- `worth-store-reclaim-policy`
- `worth-store-physical-isolation`
- `worth-store-operations`

**Relevant APIs**
- `BlobReachabilityReceipt`
- `BlobRetentionReceipt`
- S.5 reachability barriers and read plan holds
- `worth_foundational::boundary_evidence_api::lower_lane::{attachments,
  receipts}`
- `FoundationalBoundaryEvidenceAttachmentBundle`,
  `FoundationalBoundaryEvidenceExecutedReceiptArtifact`
- `worth_proof::prelude::{non_empty, ProofOutcomeKind}` for non-empty
  protected-reference proof sets

**Warnings**
- Existence on disk is not reachability.
- Refcount equality is not enough unless the counted references are admitted,
  scoped, replayable, and tied to Store authority.
- Export, capsule, and reclaim phases must consume reachability proof rather
  than re-infer liveness from rows.
- Reachability edge types are explicit: `PrimaryBlobReference`,
  `DerivedBlobReference`, `ResumeSessionReference`, `ExportHoldReference`,
  `ExternalConsumerHoldReference`, `ReplicationCapsuleReference`,
  `ReadPlanHoldReference`, `QuarantineHoldReference`, and
  `PlacementMoveReference`.
- `BackupHoldReference` is reserved for S.10 and may appear in S.7 only as a
  typed placeholder supplied by an S.10 handoff or future compatibility proof.

**Test requirements**
- Adversarial convergence: equivalent reference churn, replayed after restart,
  converges to the same reachable chunk set, reference edges, and exact
  counters.
- Adversarial denial: copied refcount rows, empty reference proofs, wrong blob
  authority, and stale generation edges cannot mint reachability.
- Protection lane: active read plans, checkpoint holds, quarantine holds,
  backup/export holds, and unfinished resume sessions remain visible in
  reachability state.
- Dedupe lane: dedupe receipts create or update admitted reference edges so
  shared chunks remain reachable until every admitted sharing edge is absent or
  separately denied.

**Engineering decisions**
- Separate reachable chunk authority, derived reference indexes, orphan
  candidates, and protected holds.
- Counters must include reachable chunks, reference edges, protected holds,
  orphan candidates, and stale-reference denials.
- Use Foundational boundary evidence for support-facing reachability evidence,
  not as reachability authority.
- Use Proof non-empty/fixed-shape collections where empty protected-reference
  sets would make a proof claim dishonest.
- Reference accounting must compute from admitted edge types, not from raw
  refcount equality.

**Open questions**
- Decide the first reference edge vocabulary for primary blobs, derived blobs,
  export capsules, and partial replication capsules.

### Phase 15: Retention-Safe Orphan Reclaim

Phase 15 consumes reachability and S.6 reclaim posture to make orphan reclaim
safe under protected reads, abandoned sessions, residue, and tier movement.

**Relevant subsystems**
- `worth-store-blob-chunks`
- `worth-store-retention`
- `worth-store-reclaim-policy`
- `worth-store-physical-isolation`
- `worth-store-operations`

**Relevant APIs**
- `BlobRetentionReceipt`
- S.5 reachability barriers and read plan holds
- S.6 reclaim and tier-movement I/O posture handoffs
- `worth_foundational::boundary_evidence_api::lower_lane::{receipts, support}`
- `FoundationalBoundaryEvidenceSupportCloseoutArtifact`
- `FoundationalBoundaryEvidenceSupportResidualDebtSet`
- `worth_foundational::performance_api::lower_lane::receipts` for reclaim and
  protected-denial counter receipts
- `worth_proof::prelude::{non_empty, ProofOutcomeKind}` for non-empty
  orphan-candidate proof sets

**Warnings**
- Reclaim must not observe, move, trim, punch holes, or delete chunks under
  active read protection.
- Orphan candidates are not reclaim permits.
- Residue must be localized and reported, not promoted into blob content.
- S.7 retention policy is physical and minimal: generation hold, time-window
  hold, export hold, capsule hold, read-plan hold, quarantine hold,
  resume-session hold, placement-move hold, tenant/custody hold, and
  S.10-supplied backup hold placeholder.
- Derived blobs may have more aggressive retention only when admitted rebuild
  authority and policy allow it; authoritative blobs cannot be reclaimed on
  derived-blob rules.

**Test requirements**
- Adversarial equivalence: repeated reclaim planning over the same
  reachable/orphan set produces the same permits, denials, residue report, and
  counters.
- Adversarial denial: chunks protected by active read plans, checkpoint holds,
  quarantine holds, backup/export holds, unfinished resume sessions, or missing
  S.6 reclaim posture cannot be reclaimed.
- Residue lane: bytes left by abandoned sessions or failed reclaim are reported
  as localized residue, not promoted into blob content.
- Retention lane: time-window, generation, export, capsule, quarantine,
  read-plan, tenant/custody, and resume-session holds independently block
  reclaim and produce distinct denial counters.

**Engineering decisions**
- Make reclaim consume S.5 stability/reachability proof and S.6 reclaim I/O
  posture before touching physical bytes.
- Counters must include orphan candidates, reclaimed chunks,
  protected-denial counts, residue-localization counts, and reclaim I/O posture
  denials.
- Use Foundational support evidence for support-facing orphan/residue reports,
  while keeping reclaim authority in Store.

**Open questions**
- None.

### Phase 16: Placement Admission And Residency Classes

Phase 16 introduces inline, external, and cold placement classes plus placement
admission without claiming movement correctness or backup/tier semantics.

**Relevant subsystems**
- `worth-store-blob-chunks`
- `worth-store-tiering`
- `worth-store-io-scheduler`
- `worth-store-physical-backend`
- `worth-store-layout-indexes`

**Relevant APIs**
- `S7PlacementIoReadinessSeed`
- `S6ClosedS7PlacementAdmissionSeed`
- S.6 backend capability and queue admission surfaces
- S.8 layout family readiness placeholders
- `worth_foundational::performance_api::common_path` for layout intent claims
- `FoundationalLayoutIntentClaim`, `FoundationalPerformanceLayoutIntent`,
  `FoundationalPerformanceAccessPatternPosture`,
  `FoundationalPerformanceAllocationPosture`,
  `FoundationalCounterBackedPerformanceReceipt`
- `worth_proof::prelude::{recipe, ready_now, ProofOutcomeKind}` for placement
  plan -> admitted placement progression

**Warnings**
- Cold placement may require fetch or deny; it may not claim backup, restore,
  archival durability, or product tier policy.
- Placement may change residency and cost, not blob identity, digest,
  authority, or security scope.
- External placement is still Store physical foundation work, not sidecar
  ownership.
- External placement means Store-owned physical storage outside the primary
  page file or segment, still governed by Store witnesses, manifests, security
  scope, reachability, reclaim, and recovery.
- Filesystem paths, object-store keys, URLs, and external metadata databases
  are not blob authority.
- Cold placement states are explicit: `HotAvailable`, `ColdAvailable`,
  `ColdFetchRequired`, `ColdFetchInProgress`, `ColdUnavailable`, `ColdStale`,
  `ColdScopeDenied`, and `ColdRebindRequired`.

**Test requirements**
- Adversarial parity: inline, external, and admitted cold placement produce the
  same blob identity, security scope, reachability, digest, and lifecycle
  receipts while reporting distinct placement and I/O counters.
- Adversarial denial: unsupported backend placement, stale S.6 readiness,
  copied placement seeds, unavailable cold chunks, and external sidecar paths
  without Store authority fail before read/write publication.
- Non-claim lane: placement evidence explicitly cannot satisfy S.10 backup,
  restore, or archival semantics.
- External recovery lane: external placement manifest evidence, recovery probe,
  missing-denial, orphan scan, and cleanup receipt are required before external
  placement can claim Store-owned recoverability.

**Engineering decisions**
- Model placement plan -> placement admission as separate from placement
  movement execution.
- Use S.6 readiness only for I/O admission and pacing; S.7 owns lifecycle
  placement correctness.
- Counters must include placement class, placement moves, cold fetches,
  unavailable chunks, inline/external reads, and tier-move protected denials.
- Use Foundational layout intent to describe representation/access/allocation
  posture; do not treat layout intent as proof of placement execution.
- Strengthen placement performance claims only from executed Store placement
  receipts into counter-backed Foundational performance receipts.
- Placement admission must produce recovery obligations for external chunks,
  including manifest rows, probe strategy, missing-denial behavior, orphan scan
  posture, and cleanup evidence.

**Open questions**
- Decide the minimum external placement backend shape that still counts as
  Store-owned physical storage rather than a product blob sidecar.

### Phase 17: Placement Movement And Read-During-Move Stability

Phase 17 executes placement movement and proves reads, verification, reclaim,
and reachability remain stable while chunks move between admitted residency
classes.

**Relevant subsystems**
- `worth-store-blob-chunks`
- `worth-store-tiering`
- `worth-store-io-scheduler`
- `worth-store-physical-isolation`
- `worth-store-physical-certification`

**Relevant APIs**
- executed placement receipt surfaces
- S.5 read stability and future chunk movement barriers
- S.6 queue admission and foreground reservation surfaces
- `FoundationalCounterBackedPerformanceReceipt`
- `worth_proof::prelude::{join_ready, ProofOutcomeKind}` for lifecycle plus
  placement readiness

**Warnings**
- Movement execution cannot replace placement admission.
- Read-during-move must return stable bytes, typed retry, or typed denial;
  never half-moved chunks.
- Placement move counters do not prove lifecycle correctness unless bound to
  chunk-tree and reachability evidence.
- Read consistency contracts are explicit: read old placement until move
  publishes, read new placement only after move publishes, dual-read with
  verification when admitted, retry until placement observation stabilizes, or
  deny when movement crosses unavailable cold tier.

**Test requirements**
- Adversarial parity: read-before, read-during, and read-after admitted
  movement preserve blob identity, security scope, reachability, digest, and
  verified bytes.
- Adversarial denial: stale movement plan, missing S.5 read hold, copied
  execution receipt, unavailable cold chunk, and foreground reservation
  violation deny or retry before exposed bytes.
- Crash lane: restart during move either resumes from an admitted movement
  receipt or localizes residue without publishing a mixed placement state.
- Cold lane: cold unavailable, stale, scope-denied, and rebind-required states
  produce distinct read/export/capsule/materialization outcomes.

**Engineering decisions**
- Model executed placement receipt -> published placement observation as a
  separate proof stage after admission.
- Counters must include placement class, placement moves, cold fetches,
  unavailable chunks, inline/external reads, tier-move retries, and protected
  denials.
- Strengthen placement performance claims only from executed Store placement
  receipts into counter-backed Foundational performance receipts.

**Open questions**
- None.

### Phase 18: Blob Compaction And Chunk-Tree Rewrite

Phase 18 compacts blob chunk trees, dedupe indexes, placement residue, and
orphaned chunk topology without changing blob object identity, generation
visibility, security scope, or logical content.

**Relevant subsystems**
- `worth-store-blob-chunks`
- `worth-store-maintenance`
- `worth-store-physical-isolation`
- `worth-store-reclaim-policy`
- `worth-store-io-scheduler`
- `worth-store-physical-certification`

**Relevant APIs**
- chunk-tree rewrite plan surfaces
- S.5 read stability and reachability barriers
- S.6 compaction/background pacing and foreground reservation surfaces
- dedupe reference edge and reclaim permit surfaces
- `worth_foundational::performance_api::lower_lane::receipts`
- `worth_proof::prelude::{recipe, join_ready, ProofOutcomeKind}`

**Warnings**
- Blob compaction may rewrite physical chunk-tree layout and placement residue;
  it must not mutate a published `BlobGeneration`.
- Compaction is not dedupe admission, reclaim permission, backup, export, or
  S.8 global layout optimization.
- Compaction must not use whole-object materialization as a rewrite shortcut.

**Test requirements**
- Adversarial equivalence: compacted and uncompacted representations of the
  same blob generation verify to the same `BlobObjectId`, `BlobGeneration`,
  logical digest, security scope, reachability, and exported canonical basis.
- Adversarial denial: compaction cannot proceed across active read holds,
  quarantine holds, stale dedupe references, unavailable cold chunks,
  unsupported S.6 pacing, or missing reachability proof.
- Crash lane: restart during blob compaction either resumes an admitted rewrite
  plan, rolls back to the pre-compaction placement, or localizes residue without
  publishing mixed chunk-tree state.

**Engineering decisions**
- Model compaction as plan -> admission -> protected rewrite -> publication of
  replacement physical layout observation, never as blob generation mutation.
- Preserve old chunk-tree readability until the compacted observation
  publishes and all admitted readers can see a stable old or new view.
- Counters must include chunks scanned, chunks rewritten, dedupe edges
  preserved, references transferred, bytes moved, foreground yields, residue
  localized, and compaction denials.

**Open questions**
- None.

### Phase 19: Export Bundle Canonicalization

Phase 19 produces aspect-native, canonical export bundles from current S.7
lifecycle evidence without claiming import readmission or S.10 backup
correctness.

**Relevant subsystems**
- `worth-store-blob-chunks`
- `worth-store-operations`
- `worth-store-aspect-native`
- `worth-store-offline-verifier`

**Relevant APIs**
- S.7 chunk-tree, digest, reachability, and placement receipts
- S.10 backup/export readiness placeholders
- `worth_foundational::aspects()` for native export declarations
- `worth_foundational::canonicalization_api::lower_lane::{export, digest}`
- `prepare_canonical_export_bundle(...)`,
  `CanonicalExportReadyArtifact`, and `CanonicalExportReadinessProofs`
- `worth_proof::prelude::{recipe, ProofOutcomeKind}`

**Warnings**
- Export is not backup correctness; S.10 owns backup/PITR/repair semantics.
- Export bundles are portable declarations plus evidence, not live witnesses.
- Terminal projections may summarize exports but cannot re-enter as authority.
- Export layers are distinct: `ExportManifest`, `ExportEvidenceBundle`,
  `ExportedChunkBytes`, `ExportCustodyReceipt`, `ImportDeclaration`,
  `ImportReadmissionReceipt`, and `ImportedBlobWitness`.
- Export canonical basis is a boundary representation of current Store
  evidence. It must not become the internal chunk-tree format or runtime
  authority model.

**Test requirements**
- Adversarial equivalence: equivalent current lifecycle evidence produces the
  same canonical export bundle, manifest shape, digest evidence, and export
  counters.
- Adversarial denial: stale reachability, missing chunk, terminal projection
  rows, placement-only evidence, and copied export rows fail before export
  bundle publication.
- Offline lane: offline verifier can inspect exported chunk bundles and report
  declarations, checksums, and digest evidence without minting current Store
  authority.

**Engineering decisions**
- Treat export bundles as aspect-native declarations plus Store-owned physical
  evidence, not as live witnesses.
- Use Foundational canonical export surfaces to package boundary-crossing
  evidence, not to create blob authority.
- Counters must include exported chunks, exported bytes, skipped chunks,
  missing chunks, and terminal-projection denial counts.
- Keep export manifest, evidence bundle, chunk bytes, and custody receipt as
  separate artifacts with separate authority and readmission behavior.
- Architecture boundary: implement export as evidence collection ->
  canonical classification -> transition verification -> receipt/bundle
  construction. Do not put those responsibilities in one god function.
- Architecture boundary: export modules must be organized by artifact layer
  (`manifest`, `evidence_bundle`, `chunk_bytes`, `custody_receipt`,
  `denials`, and `facade`) rather than by generic helper bins.
- Architecture boundary: public export APIs expose only the next valid export
  capability; raw chunk rows, copied receipt ids, terminal projections, and
  offline-verifier observations remain inputs to classification or denial, not
  constructors for export authority.

**Open questions**
- Decide which export bundle fields are shared with S.10 and which remain
  S.7-only until backup/PITR semantics exist.

### Phase 20: Import Readmission After Trust Boundary

Phase 20 consumes exported or restored declarations after a trust boundary and
requires current Store authority, current security scope, and chunk
verification before any imported blob witness exists.

**Relevant subsystems**
- `worth-store-blob-chunks`
- `worth-store-security`
- `worth-store-aspect-native`
- `worth-store-operations`

**Relevant APIs**
- S.5.1 security-scope readmission surfaces
- `worth_foundational::compatibility().json()` for hostile JSON readmission
  tests
- `bridge_canonical_export_trust_boundary(...)`,
  `readmit_canonical_export_after_boundary(...)`,
  `BoundaryBridgedCanonicalExportArtifact`, and
  `CanonicalExportReadmissionAuthority`
- `worth_proof::prelude::{recipe, ProofOutcomeKind}` plus
  `.bridge_trust_boundary()`, `.rebind_with(...)`, and `.readmit_with(...)`

**Warnings**
- Imported chunk manifests are raw declarations until readmitted against
  current Store authority and current security scope.
- Offline export/import, different deployment, different Store instance,
  different key generation, different tenant authority, different custody
  domain, and backup restoration after key rotation are trust boundaries.
- Compatibility JSON lowering may create declarations only; it cannot create
  S.7 witnesses.
- Import placement inputs are explicit: inline in bundle, external by
  reference, cold-unavailable, already present locally, deduped locally,
  requires fetch, and scope-denied.

**Test requirements**
- Adversarial equivalence: export followed by readmitted import reconstructs
  the same chunk-tree identity, digests, scoped chunk metadata, reachability
  basis, and counters without trusting terminal projection text.
- Adversarial denial: imported JSON, stale key generation, wrong tenant
  authority, copied export rows, mismatched custody domain, missing chunk, and
  placement-only evidence fail before blob witness construction.
- Readmission lane: bridged canonical export evidence must pass explicit Store
  readmission before becoming current-basis S.7 evidence.
- Placement lane: import readmission produces a placement admission plan before
  it can produce an imported blob witness.

**Engineering decisions**
- Make import readmission consume current S.5.1 security readiness and S.7
  chunk verification before publishing blob lifecycle authority.
- Use Proof trust-boundary weakening so imported evidence cannot retain
  current authority by accident.
- Counters must include imported declarations, readmitted chunks,
  stale-scope denials, missing-chunk denials, and terminal-projection re-entry
  denials.
- Import readmission must distinguish `ImportDeclaration`,
  `ImportReadmissionReceipt`, and `ImportedBlobWitness`.
- Architecture boundary: implement import as boundary declaration parsing ->
  trust-boundary classification -> current-scope readmission -> chunk
  verification -> placement admission -> witness construction. Each step must
  live in a named module or transition function.
- Architecture boundary: deserialization, JSON compatibility, manifest parsing,
  and terminal projection lowering may create declarations only. They must not
  sit in the same module that constructs current Store witnesses.
- Architecture boundary: the public import facade must make it mechanically
  obvious which type is a raw declaration, which is readmitted evidence, and
  which is a current Store witness.

**Open questions**
- None.

### Phase 21: Partial Replication And Capsule Readiness

Phase 21 publishes blob-bearing capsule readiness without implementing full
Roadmap 1 replication or S.10 backup semantics.

**Relevant subsystems**
- `worth-store-blob-chunks`
- `worth-store-replication`
- `worth-store-operations`
- `worth-store-readiness`
- `worth-store-certification`

**Relevant APIs**
- blob chunk lifecycle receipts
- partial replication capsule readiness placeholders
- aspect-native capsule declaration surfaces
- Proof evidence and receipt reporting vocabulary
- `worth_foundational::boundary_evidence_api::lower_lane::{attachments,
  provenance, receipts}`
- `FoundationalBoundaryEvidenceAttachmentTarget`,
  `FoundationalBoundaryEvidenceAttachmentBundle`,
  `FoundationalBoundaryEvidenceProvenanceArtifact`,
  `FoundationalBoundaryEvidenceExecutedReceiptArtifact`
- `worth_foundational::canonicalization_api::lower_lane::basis` for capsule
  canonical basis preparation
- `worth_proof::prelude::{non_empty, join_ready, compose_ready,
  ProofOutcomeKind}` for chunk-subset readiness composition

**Warnings**
- Partial replication readiness is not replica convergence, backup success, or
  product replication API correctness.
- A capsule may reference blob chunks only through scoped chunk-tree and
  reachability evidence.
- Missing, quarantined, cold-unavailable, or scope-denied chunks must produce
  typed readiness denial.
- A capsule is a positive physical artifact model: a manifest plus an admitted
  chunk-tree slice, parent/root basis, scoped chunk references, placement
  requirements, reachability snapshot, and materialization/readiness evidence.
- A capsule may be planned without materializing bytes, but materialized byte
  bundles must carry separate evidence from planning-only capsules.

**Test requirements**
- Adversarial equivalence: two capsule materializations over the same admitted
  reachable blob subset produce the same capsule readiness evidence and exact
  chunk counters.
- Adversarial denial: copied capsule rows, digest-only chunk references,
  missing chunks, stale security scope, quarantined chunks, and cold placement
  unavailable for capsule materialization cannot publish readiness.
- Non-claim lane: S.7 capsule readiness explicitly reports that full
  replication, backup, restore, and product blob API semantics remain later
  milestones.
- Slice lane: subset cuts through internal chunk-tree nodes, missing
  parent/root basis, cold chunk unavailable, deduped chunk shared across
  scopes, and reachability changes during capsule creation all produce
  distinct denial or retry outcomes.

**Engineering decisions**
- Represent capsule readiness as a typed handoff that consumes S.7 lifecycle
  receipts and emits Proof/Foundational evidence without moving lifecycle law
  into certification.
- Keep partial replication references chunk-granular and security-scope
  preserving.
- Counters must include capsule chunks, skipped chunks, denied chunks,
  replicated-byte declarations, and partial-readiness denials.
- Attach Foundational provenance and boundary evidence to capsule readiness so
  later replication can inspect what was prepared without treating the capsule
  as full replication authority.
- Use Proof fixed-shape or non-empty collections so empty or unordered capsule
  subsets cannot masquerade as admitted partial replication readiness.
- Capsule readiness must consume reachability and placement evidence and must
  not infer liveness or fetchability from chunk ids alone.
- Architecture boundary: implement capsule readiness as slice selection ->
  reachability/placement classification -> fixed-shape proof composition ->
  readiness construction. Do not let capsule planning, byte materialization,
  non-claim reporting, and readiness construction collapse into one function.
- Architecture boundary: keep planning-only capsule declarations separate from
  materialized byte bundles and from future replication authority.
- Architecture boundary: the public capsule facade must teach the non-claim:
  S.7 may prepare blob-bearing capsule readiness, but it does not certify full
  replication, backup, restore, or convergence.

**Open questions**
- Decide whether capsule readiness lives in `worth-store-blob-chunks` directly
  or in a narrow handoff module consumed by `worth-store-replication`.

### Phase 22: Blob Harness Actors, Faults, Oracles, And Coverage

Phase 22 fills the S.4.5 blob harness skeleton with production actors, fault
injections, oracle families, coverage rows, and shortcut denials for the
completed S.7 lifecycle.

**Relevant subsystems**
- `worth-store-physical-certification`
- `worth-store-test-support`
- `worth-store-blob-chunks`
- `worth-store-certification`
- `worth-store-budgets`

**Relevant APIs**
- S.4.5 scenario definition, plan, lowering, transcript, replay bundle, and
  oracle family surfaces
- S.6 I/O pressure driver and coverage extension surfaces
- S.7 blob chunk lifecycle receipts and counters
- `worth_foundational::profiles_api::{common_path, lower_lane,
  stronger_lane::readiness}`
- `worth_foundational::boundary_evidence_api::lower_lane::{attachments,
  receipts, support}`
- `worth_foundational::performance_api::lower_lane::{basis, receipts, reports}`
- `worth_proof::prelude::{proof_flow, ProofOutcomeKind, join_ready}` for
  scenario plan -> lowered driver -> execution-ready -> executed evidence
  progression

**Warnings**
- Do not create a separate blob harness outside S.4.5.
- Harness fixtures must exercise production boundary code, not convenience
  in-memory blob paths.
- Harness coverage must not be one giant blob suite; actors, faults, oracles,
  and profiles are separate responsibilities.
- Scenario summaries, logs, and same-run self-comparison are not proof.

**Test requirements**
- Adversarial replay: the same blob scenario replayed through S.4.5 yields the
  same scenario identity, plan shape, transcript, oracle verdict, and executed
  evidence bundle.
- Adversarial denial: tiny blobs, whole-object helpers, missing chunk counters,
  logs-as-proof, synthetic success rows, and private mutation of harness state
  fail as shortcut lanes.
- Coverage lane: generated rows include blob size class, chunk count, chunk
  size class, security scope, placement class, access mode, failure point,
  memory envelope, and actor mix.

**Engineering decisions**
- Add blob-specific actors for ingest, read, verify, resume, dedupe, export,
  import, placement move, corruption, partial replication, and reclaim.
- Add blob-specific faults for crash after chunk write, after session
  checkpoint, after root publication, during tier move, during export, and
  during reclaim.
- Add blob-specific oracles for byte equality, chunk ordering, digest/checksum
  distinction, no cross-scope dedupe, reachability, constant memory, and
  no sidecar path.
- Attach Foundational profiles to scenario families so local, CI, and heavy
  profiles are explicit materialization postures rather than runner folklore.
- Use Foundational performance receipts for executed scenario counters and
  Foundational support evidence for shortcut-denial reports.
- Architecture boundary: harness code must be organized by actors, faults,
  oracles, profiles, transcripts, replay, and coverage. Generic fixture worlds
  and all-purpose scenario helpers are phase blockers.
- Architecture boundary: production actors must call production facades; test
  authority may prepare legal synthetic setup only behind explicit
  test-authority modules and cannot mint production witnesses.
- Architecture boundary: oracles must compare executed evidence and counters,
  not logs, summaries, fixture internals, or same-run self-reports.

**Open questions**
- Decide the first coverage matrix axes that must be mandatory in local CI
  versus heavy qualification.

### Phase 23: Real Multi-GB Blob Qualification Lane

Phase 23 adds an actual multi-GB blob lane that proves Store can move real
large objects through the production chunk path without memory cheating.

**Relevant subsystems**
- `worth-store-test-support`
- `worth-store-physical-certification`
- `worth-store-blob-chunks`
- `worth-store-physical-backend`
- `worth-store-budgets`

**Relevant APIs**
- deterministic streaming blob generator with seed, byte length, chunk profile,
  byte-pattern profile, expected digest basis, and expected chunk count
- large-object streaming profiles
- resident memory and allocation counter receipts
- S.4.5 heavy profile execution surfaces
- `worth_foundational::profiles_api::lower_lane::{identity, materialization,
  certification}`
- `FoundationalProfileIdentity`,
  `MaterializedFoundationalProfileArtifact`,
  `EvidenceBackedCertifiedProofBearingArtifact`
- `worth_foundational::performance_api::stronger_lane::{certified,
  readiness}`
- `FoundationalCertifiedPerformanceBundle`,
  `FoundationalPerformanceCertifiedReadmissionAuthority`,
  `FoundationalCounterBackedPerformanceReceipt`
- `worth_proof::prelude::{ready_now, ProofOutcomeKind}` for heavy-profile
  admission and execution readiness

**Warnings**
- Sparse-only logical length does not prove multi-GB streaming. Sparse files
  are useful as hostile deception fixtures, not as the main qualification
  proof.
- A zipped `target/` directory, current workspace cache, build artifact tree, or
  any other local incidental corpus is not the canonical heavy blob authority.
  It is nondeterministic, machine-local, rebuild-sensitive, and may only be
  used as an optional chaos/stress corpus after the deterministic qualification
  lane already passes.
- The test must not require loading the blob into heap, building a full
  expected byte vector, or comparing through one scalar buffer.
- The qualification source must not depend on committed multi-GB fixtures,
  developer-local directory contents, same-run self-comparison, or a regenerated
  archive whose contents are not described by the scenario identity.
- Heavy lanes must be explicitly named and gated, but S.7 cannot close without
  at least one real executed multi-GB profile.

**Test requirements**
- Adversarial qualification: stream an actual multi-GB deterministic blob
  through ingest, forced interruption, resume, verify, export, import, tier
  move, partial replication readiness, corruption localization, and reclaim
  while resident memory remains bounded by the declared envelope.
- Adversarial denial: sparse-only proof, logical-size-only proof,
  whole-object expected buffer, missing disk-space preflight, hidden temporary
  sidecar file, and missing cleanup fail the qualification lane.
- Scale lane: the same scenario family runs as small deterministic local,
  CI memory-envelope-exceeding, and heavy multi-GB profiles with the same
  counter topology.
- Exact evidence lane: heavy qualification records input generator seed,
  deterministic byte-pattern profile, declared byte length, expected chunk
  count, expected digest basis, actual bytes streamed, peak resident memory,
  peak allocation count, temporary file bytes, disk bytes written, chunk count,
  verification pass basis, cleanup receipt, and backend profile.
- Deception lane: sparse files, hidden staging files, synthetic byte counters,
  and generated expected-byte artifacts are valid hostile fixtures but cannot
  satisfy the real multi-GB evidence requirement.
- Pattern lane: run the same heavy fixture contract over incompressible seeded
  bytes, highly-compressible repeated spans, chunk-boundary adversarial bytes,
  repeated-chunk dedupe pressure, and sparse/deceptive source declarations as a
  denial profile. The canonical pass/fail proof must be deterministic from
  seed and profile, not from ambient filesystem contents.
- Optional chaos-corpus lane: a streamed archive of `target/` or another large
  local directory may be admitted only as a non-canonical stress input with a
  separate profile label. It cannot satisfy the S.7 closeout evidence because
  its contents are not stable across machines, rebuilds, or time.

**Engineering decisions**
- Implement deterministic streaming generation and streaming verification by
  bounded windows and rolling/chunked digests.
- Define a `HeavyBlobFixturePlan` or equivalent proof-bearing plan whose
  identity includes seed, byte length, chunk size/fanout profile,
  byte-pattern profile, materialization mode, backend profile, and expected
  digest/chunk-count basis.
- Support two execution modes: stream-only generation for CI and ordinary
  proof replay, and opt-in real temporary file materialization under a named
  heavy fixture directory such as `target/worth-store-heavy-fixtures/` for
  local qualification. Both modes must use the same deterministic plan and
  evidence schema.
- Add disk-space preflight, cleanup receipt, and platform/backend capability
  recording for heavy profiles.
- Require exact counters for actual bytes streamed, chunk count, disk bytes
  written, temp bytes, allocations, residency, cleanup, and verification basis.
- Gate heavy execution by an explicit profile flag or environment variable,
  while keeping the spec closeout dependent on recorded heavy evidence.
- Require the heavy gate to declare its byte length explicitly, for example
  through a named heavy profile or environment variable, so a runner cannot
  silently downgrade a multi-GB qualification into a small local smoke test.
- Materialize the heavy profile as a Foundational profile artifact so the
  evidence states hardware/backend/profile assumptions explicitly.
- Certify multi-GB performance only from executed counter-backed receipts; do
  not let a profile declaration or policy admission receipt satisfy the heavy
  evidence requirement.
- Architecture boundary: heavy qualification must be a profile-driven
  production scenario, not a bespoke test shortcut. Generation, disk preflight,
  execution, verification, cleanup, and evidence materialization must be named
  stages.
- Architecture boundary: resident-memory, allocation, disk, temp-file, and
  cleanup counters are transition evidence, not after-the-fact diagnostics.
- Architecture boundary: sparse/deception fixtures must live in hostile lanes
  separate from the real multi-GB qualification lane.
- Architecture boundary: local filesystem chaos corpora are diagnostic stress
  inputs, not source authority. The canonical heavy source is the deterministic
  fixture plan plus executed byte/chunk/digest evidence.

**Open questions**
- Decide the initial required local qualification size. The default should be a
  true multiple-GB size selected by available qualification hardware, with
  smaller profiles reserved only for CI and smoke coverage labels that cannot
  close Phase 23.

### Phase 24: Certification Closeout And Shortcut Rejection

Phase 24 materializes S.7 evidence and proves that only executed blob lifecycle
authority can satisfy S.7 closeout.

**Relevant subsystems**
- `worth-store-certification`
- `worth-store-physical-certification`
- `worth-store-blob-chunks`
- `worth-store-readiness`
- `worth-store-claim-boundaries`

**Relevant APIs**
- S.7 lifecycle evidence bundle
- S.4.5 simulation replay bundle
- Proof progression and receipt vocabulary
- Foundational completed-boundary and performance receipt vocabulary
- `worth_foundational::boundary_evidence_api::stronger_lane::{readiness,
  readmission}`
- `FoundationalBoundaryEvidenceProductionTestReadyArtifact`,
  `FoundationalBoundaryEvidenceCompletedReceiptArtifact`,
  `FoundationalBoundaryEvidenceSupportCloseoutArtifact`
- `worth_foundational::performance_api::stronger_lane::readiness`
- `FoundationalPerformanceProductionTestReadyArtifact`
- `worth_foundational::canonicalization_api::stronger_lane::readiness`
- `CanonicalProductionTestReadyArtifact`
- `worth_proof::prelude::{proof_flow, ProofOutcomeKind}` and
  `worth_proof::raw::*` only for closeout proof stages whose raw topology must
  stay visible

**Warnings**
- Certification must verify executed production surfaces; it must not become
  the place where blob authority is invented.
- Closing S.7 on small deterministic profiles alone would leave the actual
  multi-GB claim unproven.
- Shortcut rejection must include compile-fail, runtime denial, counter proof,
  and harness shortcut lanes.

**Test requirements**
- Adversarial materialization: executed S.7 scenarios produce a sealed evidence
  bundle binding Store authority, security scope, replay identity, chunk-tree
  identity, digest/checksum evidence, reachability, placement, counters, and
  Proof progression.
- Adversarial denial: copied receipts, copied chunk rows, copied proof ids,
  S.6 placement readiness alone, S.5 future chunk placeholders alone, terminal
  projections, and raw counters cannot mint S.7 closeout.
- Cross-milestone lane: S.8, S.10, S.11, and S.12 handoffs receive typed S.7
  readiness plus explicit non-claims for layout discipline, backup/repair,
  key lifecycle, and full certification.

**Engineering decisions**
- Define a sealed `S7NativeBlobStoreCloseout` or equivalent that is built only
  from executed S.7 lifecycle evidence.
- Keep `worth-store-certification` as materializer and verifier; lower Store
  crates own the request, witness, receipt, and denial types.
- Publish closeout evidence as aspect-native and Proof-bearing, never JSON or
  terminal projection authority.
- Closeout must bind Store-owned lifecycle receipts to Foundational canonical,
  boundary-evidence, profile, and performance readiness artifacts.
- Closeout must preserve Proof non-success topology, especially `Stale`,
  `RebindRequired`, `Denied`, and `Deferred`, so certification cannot blur
  incomplete evidence into ordinary failure.
- Architecture boundary: closeout materialization must read as executed
  evidence source -> S.7 materialized evidence bundle -> sealed closeout
  certificate -> downstream readiness/non-claim handoffs. Certification is the
  courtroom for this sequence, not the crate that invents blob law.
- Architecture boundary: each cross-milestone handoff must consume one sealed
  closeout capability or typed non-claim. It must not reconstruct readiness
  from copied lifecycle receipts, copied counters, copied proof ids, or
  certification rows.
- Architecture boundary: closeout code must keep request, classifier,
  verifier, certificate construction, and public facade responsibilities
  separate enough that a reviewer can audit the proof transition without
  re-deriving every predicate.

**Open questions**
- None.

## Must Ship

- Store-owned S.7 blob lifecycle authority boundary with sealed proof-bearing
  types and no raw-field construction for lifecycle receipts.
- `BlobObjectId`, `BlobGeneration`, authoritative/derived blob
  classification, chunk identity, checksum, logical content digest, stored
  chunk digest, authenticated frame digest placeholder, chunk-tree root,
  canonical chunking rule, and lifecycle receipt vocabulary as distinct typed
  surfaces.
- Atomic `BlobGenerationPublished` publication protocol binding durable chunk
  bytes, admitted checksums, durable chunk-tree nodes, root candidate,
  staged reachability edges, publication record, optional semantic visibility
  handoff, and closed resume session.
- WAL, checkpoint, manifest, and recovery integration for chunk writes,
  chunk-tree metadata writes, resume checkpoints, publication records,
  reachability rows, placement rows, replay receipts, and backend-residue
  denials.
- Separate constant-memory streaming ingest and streaming read/verify paths
  with independent counters and denial lanes.
- Resumable blob writes with crash/restart recovery, stale-session denial, and
  abandoned-session reclaim paths over the named resumable-ingest state
  machine.
- Mandatory S.5.1 security-scope metadata on blob chunk witnesses: key scope,
  key version, tenant scope, authenticity class, and custody posture.
- Dedupe admission with explicit policy modes, canonical equivalence,
  collision outcomes, dedupe receipts, reclaim interaction, and cross-scope
  rejection.
- Reachability and reference tracking from named admitted edge types before
  retention-safe orphan reclaim.
- Minimal physical retention holds for generation, time-window, export,
  capsule, read-plan, quarantine, resume-session, placement-move, tenant, and
  custody posture, with S.10 backup holds reserved as typed future inputs.
- Inline, external, and cold placement admission with Store-owned external
  placement, cold availability states, recovery obligations, and S.6 I/O
  readiness consumed only as pacing/admission evidence.
- Executed placement movement and read-during-move stability proof.
- Blob compaction for chunk trees, dedupe indexes, placement residue, and
  orphan topology without changing blob object identity, generation
  visibility, security scope, or logical content.
- Aspect-native export bundle canonicalization and separate import readmission
  after trust-boundary crossing.
- Export/import artifact layering: `ExportManifest`, `ExportEvidenceBundle`,
  `ExportedChunkBytes`, `ExportCustodyReceipt`, `ImportDeclaration`,
  `ImportReadmissionReceipt`, `ImportedBlobWitness`, and import placement
  admission plan.
- Partial replication and positive capsule-readiness handoffs for blob-bearing
  artifacts, with non-claims for full replication and backup semantics.
- Early S.4.5 blob harness skeleton with profile/shortcut taxonomy, followed
  by production actors, faults, oracles, coverage rows, transcripts, replay
  bundles, and shortcut denials.
- Real multi-GB heavy qualification lane using executed production chunk paths,
  deterministic streaming generation, disk-space preflight, bounded memory,
  exact evidence fields, and cleanup evidence.
- S.7 certification closeout bundle binding Store authority, security scope,
  replay identity, chunk-tree identity, digest/checksum evidence, reachability,
  placement, counters, and Proof progression.
- Explicit Foundational materialization for `aspects()`, canonicalization,
  boundary evidence, profiles, and performance APIs at the phases that cross
  shared boundaries.
- Explicit Proof progression, checked outcome, trust-boundary readmission, and
  fixed-shape composition usage at every phase that changes authority,
  readiness, or proof shape.

## Must Preserve

- Store owns physical blob lifecycle law.
- Certification proves S.7 law; it does not define or mint the law.
- S.5 owns read stability and future chunk placeholders only until S.7 promotes
  real chunk lifecycle authority.
- S.5.1 owns security-scope readiness vocabulary consumed by blob chunks.
- S.6 owns I/O admission, pacing, and backend capability posture; it does not
  prove blob lifecycle correctness.
- S.7 owns chunk size, chunk-tree fanout, chunk metadata shape, placement class
  vocabulary, streaming path counters, and blob-local compaction. S.8 owns
  global artifact layout families, access-path indexing, physical locality
  strategy, cross-artifact layout policy, and the layout optimization/cost
  model.
- S.10 owns backup, PITR, repair, disaster recovery, and forensics.
- S.11 owns full key lifecycle, encryption hierarchy, identity admission, audit,
  and cryptographic erasure.
- Digest equality, terminal projections, JSON, serde, CLI summaries, copied
  counters, copied proof ids, and imported manifests are never authority.
- Primary blobs may be authoritative artifacts; derived blobs remain
  rebuildable and accuracy-classed.
- External placement remains Store-owned physical storage governed by Store
  witnesses, manifests, security scope, reachability, reclaim, and recovery.
  Filesystem paths, object-store keys, URLs, and external metadata databases
  are never blob lifecycle authority.
- Export canonical basis is a boundary representation of current Store
  evidence, not the internal chunk-tree format or runtime authority model.

## Acceptance Evidence

- Compile-fail suites prove external crates cannot construct S.7 lifecycle,
  security-scope, dedupe, reachability, placement, capsule, or closeout
  witnesses from raw fields, strings, digest values, copied counters, or
  terminal projections.
- Runtime suites prove streaming ingest/read/verify/export/import/resume,
  dedupe, collision handling, scope preservation, reachability, reclaim,
  placement movement, partial replication readiness, and corruption
  localization through production surfaces.
- S.4.5 simulation suites prove blob scenarios lower into shared scenario
  plans, execute through deterministic actors and faults, replay to the same
  evidence, and reject synthetic shortcuts.
- Heavy qualification proves an actual multi-GB blob can complete the declared
  S.7 lifecycle with bounded resident memory, bounded allocations, exact chunk
  counters, input generator seed, declared byte length, actual bytes streamed,
  temp-file bytes, disk bytes written, verification basis, backend profile, and
  cleanup evidence.
- Counter receipts expose exact chunk reads, chunk writes, bytes streamed,
  checksums, digest updates, dedupe hits/misses, collision probes, scope
  denials, reachable chunks, orphans, reclaim operations, placement moves,
  cold fetches, capsule chunks, and peak resident memory.
- Counter receipts classify strength as `ExactCounter`, `MonotonicCounter`,
  `SampledCounter`, `DerivedCounter`, `DiagnosticCounter`, or
  `CertificationOnlyCounter`; lifecycle, publication, reachability, reclaim,
  corruption, memory-bound, and heavy qualification claims consume exact
  counters unless a phase explicitly proves why weaker evidence cannot become
  authority.
- Foundational evidence reports aspect-native source material, canonical basis,
  completed-boundary receipts, profile/support posture, and counter-backed
  performance receipts where those are shared platform vocabulary.
- Proof evidence reports fixed-shape progression from declaration to admitted
  scope, streamed chunks, chunk-tree publication, lifecycle receipt, harness
  replay, and closeout certification.
- Foundational canonical evidence includes `CanonicalBasisReadyArtifact`,
  `CanonicalExportReadyArtifact`, `CanonicalDerivedDigest`,
  `CanonicalComparisonOutcome`, and readmitted canonical export evidence where
  blobs cross trust boundaries.
- Foundational boundary evidence includes executed receipts, completed
  receipts, provenance, lineage, support closeout, and attachment bundles for
  S.7 lifecycle evidence.
- Foundational performance evidence includes policy-admission receipts only as
  pre-execution budget outcomes and `FoundationalCounterBackedPerformanceReceipt`
  / certified performance bundles only after executed Store counters exist.
- Proof evidence preserves `ProofOutcomeKind` and `TransitionOutcome`
  categories through S.7 so stale, rebind-required, denied, deferred, and
  failed outcomes cannot be blurred into a generic error.

## Sequencing Notes

S.7 belongs immediately after S.6 because native blobs need admitted I/O
pacing, backend capability posture, foreground reservation, and background
blob pressure accounting before large-object streaming can be honest.

S.7 belongs before S.8 because chunk trees and large-object streaming must
exist before the broader access-path and layout discipline milestone can
classify every durable artifact family honestly.

S.7 belongs before S.10, S.11, and S.12 because backup/repair, full security
and key lifecycle, and final physical certification all need real blob chunks,
scope-preserving metadata, reachability, placement, and blob-scale evidence
rather than placeholders.

## Known Risks

- Blob identity may blur with content digest unless `BlobObjectId` and
  `BlobGeneration` remain the only positive blob object identity model.
- Published generations may accidentally become mutable if root replacement,
  placement movement, compaction, or import readmission is not typed as a
  separate lifecycle event.
- External placement may recreate sidecar storage unless Store ownership,
  manifest evidence, recovery probes, missing-denials, orphan scans, and
  cleanup receipts are strict.
- Chunking, compression, encryption, and authentication ordering may affect
  future dedupe, digest, export, and authenticity semantics unless
  `LogicalContentDigest`, `StoredChunkDigest`, and `AuthenticatedFrameDigest`
  stay distinct.
- Reachability and refcount models may fail under dedupe unless reference edge
  types and dedupe receipts remain explicit.
- Multi-GB qualification may be gamed through sparse files, hidden staging,
  synthetic byte counters, or whole-object expected artifacts unless exact
  evidence is required.
- Blob compaction may accidentally mutate generation identity unless rewrite
  publication stays separate from blob generation publication.
- Certification may drift into law ownership unless lower Store crates own
  request, witness, receipt, and denial types.
