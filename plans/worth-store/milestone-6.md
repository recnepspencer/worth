# Milestone 6 Engineering Spec: Aspect-Aware Physical Layout And Content-Addressed Structural Blocks

> **Status:** Draft
>
> **Roadmap parent:** [worth_store_roadmap.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/worth_store_roadmap.md)
>
> **Vision parent:** [worth_store_vision.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/worth_store_vision.md)
>
> **Test requirements:** [test-requirements.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/test-requirements.md)
>
> **Prerequisite milestones:**
> - [milestone-1.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-1.md)
> - [milestone-2.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-2.md)
> - [milestone-3.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-3.md)
> - [milestone-3.5-3.6.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-3.5-3.6.md)
> - [milestone-4.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-4.md)
> - [milestone-5.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-5.md)
> - [milestone-5-closeout.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-5-closeout.md)
>
> **Concurrent milestones:**
> - `Milestone 7` (`Durable Schema, Lineage, Cursor, And Checkpoint Artifacts`)
> - `Milestone 9` (`Deterministic Bulk Ingest And Bulk Transform Paths`) once the chunk model is frozen honestly
>
> **Impacted later milestone:** `Milestone 8: Live-Query Substrate And Durable Sync Basis`
>
> **Primary architectural driver:** make aspect-scoped reads, CDC narrowing,
> and cross-branch physical reuse honest derived storage programs while freezing
> a chunk-aligned structural block model that Milestone 9 can consume without
> redefining canonical commit authority

## Goal

Make aspect-aware physical layout and content-addressed structural blocks
first-class derived store families so admitted partial reads, CDC narrowing,
and cross-branch reuse stay narrow, rebuildable, and replay-safe instead of
degrading into hidden whole-state scans or copied structural payloads.

## Why This Milestone Exists

Milestone 6 is not "add some indexes and dedup tables."

It is the milestone that decides whether `worth-store` can claim aspect-local
read efficiency and structural reuse honestly, or whether later live-query,
bulk ingest, replication, and retention work will stand on backend-local
layouts that nobody can explain back through canonical authority.

Milestone 1 locked canonical durable authority.

Milestone 2 locked operating-mode ownership.

Milestone 3 and Milestone 3.5/3.6 locked the durable crash boundary, media
semantics, and recovery-source precedence.

Milestone 4 locked snapshots as basis-explicit, non-authoritative restore
substrates.

Milestone 5 locked branch-delta layering as a replay-parity physical branch
program instead of copied full-state storage.

Milestone 6 now has to lock a different physical honesty boundary:

- what an admitted aspect-scoped read is allowed to touch physically
- what exact storage family is allowed to carry reused structural regions
- what exact sameness contract lets one branch reuse another branch's physical
  structure without turning reuse into authority
- what exact block and chunk vocabulary later live-query and bulk programs are
  allowed to inherit
- what exact fallback and rebuild paths remain visible when narrowing or reuse
  cannot be admitted honestly

If this milestone is weak, later work will pay for it immediately:

- live-query narrowing will call itself selective while still decoding broad
  branch images
- CDC surfaces will pretend to be aspect-local while whole-state scans happen
  underneath
- content-addressed reuse will become an opaque cache that nobody can certify
  or rebuild
- Milestone 9 bulk chunking will freeze its own physical units ad hoc instead
  of inheriting one store-wide honest block/chunk model

This milestone exists to make physical narrowing and structural reuse explicit
before later programs start depending on them.

## Hard Part

The hard part is not physically separating bytes by aspect.

The hard part is preserving one exact separation among five things naive
storage designs constantly collapse:

- canonical authoritative commit history
- branch-delta layering that already defines physical branch change structure
- aspect-aware read slices derived from that history and layering
- content-addressed structural blocks that reuse physical regions across branch
  scopes
- chunk-aligned publication units that Milestone 9 wants to consume for bulk
  ingest and transform without becoming a second commit language

The design fails if:

- an aspect-scoped read still has to decode broad branch state on the admitted
  fast path
- structural blocks are keyed by backend serialization residue instead of a
  semantic equivalence contract
- deduplication changes replay, restore, or branch-visible conclusions because
  block identity outranked canonical authority
- chunk boundaries needed by Milestone 9 are discovered opportunistically by
  ingest code instead of frozen as part of the physical-layout contract here
- support-artifact work from Milestone 7 or bulk-path work from Milestone 9
  has to understand backend-local aspect layout trivia to stay meaningful

Milestone 6 therefore has to make narrowing cheap enough to matter, structural
reuse explicit enough to certify, and chunk identity stable enough to share
across milestones without letting any of those derived programs become shadow
authority.

## Explicit Assumptions

- Milestone 1 authoritative artifact families remain the only semantic durable
  truth authority.
- Milestone 2 operating-mode boundaries remain unchanged; Milestone 6 is a
  physical-layout and reuse milestone, not a mode or lifecycle milestone.
- Milestone 3 and Milestone 3.5/3.6 already make publication, recovery, and
  source precedence exact enough that aspect-layout and block families may ship
  as derived durable artifacts without softening restart honesty.
- Milestone 4 snapshots remain independent basis-explicit derived artifacts;
  Milestone 6 may accelerate later snapshot-family reads physically, but it
  does not redefine snapshot meaning or restore semantics.
- Milestone 5 already froze shared-base creation, branch-delta identity,
  rewrite lineage, and replay-parity branch reads. Milestone 6 must build on
  that branch/frontier vocabulary rather than inventing a second branch-layout
  ontology.
- `worth-relational` still owns commit semantics, branch semantics, aspect
  semantics, schema semantics, lineage semantics, and canonical replay meaning.
- Milestone 7 proceeds concurrently on schema, lineage, cursor, and checkpoint
  durability and must remain branch/frontier based rather than aspect-layout or
  block-layout aware.
- Milestone 9 will be built concurrently once Milestone 6 freezes an honest
  chunk model, but Milestone 9 still owns bulk orchestration, resumable
  progress checkpoints, and bounded-memory execution semantics.
- retention, compaction, replication, and advanced derived-family programs
  remain later milestones even if Milestone 6 reserves the physical vocabularies
  they will need.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects is solving the hostile physical honesty
  problem before later feature work treats it as ambient infrastructure.
  Milestone 6 therefore starts from hidden broad reads and fake reuse, not from
  "partial reads would be nice."
- `arch_laws.md`
  The most important thing it protects here is categorical separation between
  authority and derivation plus proof-bearing phase progression. Law 33 is
  load-bearing: aspect layouts, structural blocks, and chunk maps must stay
  derived durable artifacts rebuildable from authority. Law 41 matters too:
  aspect-read admission, published block families, dedup-admitted reuse, and
  chunk-stable publication units must be distinct proof-bearing types.
- `perf_laws.md`
  The most important thing it protects is explicit breadth control. Milestone 6
  therefore has to encode narrowing, decode breadth, block reuse, and fallback
  scope as named contracts with exact counters instead of hiding whole-state
  work behind a selective-sounding API.
- `domain_laws.md`
  The most important thing it protects is decomposition by reason-to-change.
  Aspect-scope planning, layout publication, structural-block identity, dedup
  reuse, chunk-map definition, and certification evidence must be separate
  subdomains rather than one storage-optimization module.
- `worth_store_vision.md`
  The most important thing it protects is that store persists canonical truth
  once and builds physical acceleration around it without redefining semantics.
  Milestone 6 must therefore make aspect-aware layout and structural blocks
  rebuildable from canonical commits and branch-delta authority rather than
  treating physical layout as meaning.
- `worth_store_roadmap.md`
  The most important thing it protects is sequencing. Milestone 6 belongs after
  branch-delta layering because physical narrowing and structural reuse need an
  already-honest branch substrate, and Milestone 9 may overlap only after the
  chunk model here is explicit.
- `worth-store/test-requirements.md`
  The most important thing it protects is certification-grade proof that
  physical acceleration changes cost only, not truth. Milestone 6 is not
  closeable until the `Aspect-Layout Narrowing And Structural-Block Dedup
  Integrity Test` proves admitted fast paths match authority, fallback remains
  explicit, and dedup does not change replay or restore conclusions.
- `milestone-4.md`
  The most important thing it protects is derived-family non-authority with
  basis-explicit rebuildability. Milestone 6 should mirror that pattern for
  aspect slices and structural blocks instead of allowing backend-local layout
  residue to become a hidden restore substrate.
- `milestone-5.md`
  The most important thing it protects is a real branch/frontier and delta-layer
  model. Milestone 6 must inherit Milestone 5's branch basis, replay parity,
  replacement lineage, and control-lane discipline instead of smuggling in a
  second branch representation through layout tables or dedup blocks.
- `milestone-5-closeout.md`
  The most important thing it protects is that shared-base branch creation and
  replay-parity delta reads are already closed. Milestone 6 should therefore
  optimize within that substrate, not reopen copied-base, hidden-replay, or
  Milestone-7-boundary questions.
- `milestone-7.md`
  The most important thing it protects is that support-artifact meaning stays
  branch/frontier explicit and independent from backend-local physical layout.
  Milestone 6 must preserve that boundary while still letting Milestone 7 and
  later Milestone 8 consume narrow read surfaces.
- `worth_store_dependency_map.md`
  The most important thing it protects is the real unlock shape: Milestone 6
  and Milestone 7 together unlock Milestone 8, while Milestone 9 can overlap
  only once Milestone 6 freezes the chunk model honestly enough for canonical
  bulk chunking.

## Adversarial Constraint

Milestone 6 must survive this hostile condition:

> A store with deep branch history, high aspect sparsity, repeated CDC
> narrowing, heavy cross-branch structural overlap, deleted and rebuilt layout
> artifacts, and concurrent Milestone 9 chunked bulk work must preserve the
> same branch-visible truth, replay conclusions, and restore conclusions as a
> control lane that ignores aspect-layout and structural-block acceleration and
> reconstructs the same result from canonical authoritative history plus
> admitted branch-delta replay alone.

## Product Decision Lock

- aspect-aware physical layout is always classified as a derived durable
  storage family
- content-addressed structural blocks are always classified as derived durable
  storage families
- block identity and reuse must derive from an explicit semantic equivalence
  contract, not backend byte coincidence alone
- admitted aspect-scoped reads must name their requested aspect scope and
  target frontier explicitly; "read whatever is cheap" is out of spec
- fallback from admitted narrow reads to broader control paths must stay
  explicit in results, diagnostics, and counters
- materialization of Milestone 6 layout support is an explicit lane choice, not
  an ambient side effect of asking for a read or certification surface
- chunk-aligned physical publication units frozen here are physical derivation
  units only; they do not become a second commit model or a bulk-only truth
  format
- Milestone 6 must export chunk and block vocabulary that Milestone 9 can
  consume, but it must not absorb resumable ingest orchestration, bulk
  checkpoint semantics, or bounded-memory scheduling into this milestone
- deleting all aspect-layout and structural-block artifacts must leave
  authoritative replay, snapshot rebuild, branch-delta rebuild, and later bulk
  parity intact even if the fallback is slower
- Milestone 7-facing and Milestone 9-facing references must remain expressed in
  branch/frontier and chunk/block vocabulary, not backend-local table or file
  internals

Normative consequence:

- any implementation that claims aspect-local reads while decoding broad branch
  state on the admitted fast path is out of spec
- any implementation that silently auto-materializes layout support on a
  cheap-looking read path without the caller or an explicit policy choosing
  that lane is out of spec
- any implementation that makes dedup or block reuse authoritative for replay,
  restore, or branch-head meaning is out of spec
- any implementation that forces Milestone 9 to invent its own chunk identity
  because Milestone 6 never froze one honest physical unit is out of spec
- any implementation that requires Milestone 7 support-artifact readers to
  understand aspect-table or block-layout shape is out of spec

## Scope

### In Scope

- aspect-aware physical layout derived from canonical commits and branch-delta
  authority
- explicit aspect-scope vocabulary for admitted narrow reads and CDC narrowing
- content-addressed structural block identity and publication
- cross-branch deduplication over structural blocks with explicit equivalence
  contracts
- rebuild of aspect-layout and structural-block families from authoritative
  artifacts
- explicit chunk-map and chunk-identity vocabulary suitable for concurrent
  Milestone 9 bulk chunking
- result-envelope, diagnostics, and counter surfaces that expose fallback,
  decode breadth, block reuse, and chunk width honestly
- certification bundles for narrowing parity and dedup integrity

### Explicitly Out Of Scope

- schema-boundary, lineage, cursor, and checkpoint durability semantics owned
  by Milestone 7
- live-query continuation execution semantics beyond the narrow read substrate
  later needed by Milestone 8
- bulk-ingest orchestration, resumable progress checkpoints, transform
  semantics, or bounded-memory scheduling beyond the chunk model Milestone 9
  will consume
- retention, compaction, reclaim, or replication policy beyond the minimum
  needed to keep derived-layout status honest
- correspondence indexes, locality clustering, and accuracy-taxonomy work from
  later milestones
- any second commit language, backend-private truth cache, or branch-head
  authority surface

## Physical Layout Authority Model

### Layout Non-Authority Rule

Aspect-aware layout tables, structural blocks, dedup indexes, and chunk maps
are derived durable artifacts.

They are allowed to accelerate:

- aspect-scoped reads
- CDC narrowing
- cross-branch physical reuse
- later bulk chunking and live-query narrowing

They are not allowed to define:

- canonical commit meaning
- branch-head authority
- ordered parent meaning
- schema or lineage meaning
- cursor or checkpoint meaning

Normative rule:

- if all Milestone 6 layout and block families are deleted, the store must
  still be able to reconstruct the same truth through authoritative replay,
  branch-delta replay, snapshot rebuild, and later bulk parity control lanes
- if a narrow read or dedup-backed read disagrees with canonical control
  reconstruction, the layout family is wrong and must be rejected or rebuilt;
  authority is not allowed to bend toward the acceleration layer

This is the anti-shadow-authority line for Milestone 6.

### Admitted Aspect Scope Rule

Milestone 6 must freeze the admitted vocabulary for physical narrowing rather
than leaving "aspect-aware" as a backend slogan.

Minimum required scope objects:

- `AspectScope`
- `AspectLayoutTarget`
- `AspectProjectionSet`
- `CdcNarrowingScope`
- `ChunkedPhysicalSlice`

Rules:

- every admitted narrow read must declare its aspect scope and target frontier
- every scope object must map to a branch/frontier authority basis rather than
  to raw backend row ranges
- scope equality must be explicit and canonicalized so counters and cache
  surfaces speak the same vocabulary
- a scope object may drive physical narrowing, but it may not redefine which
  truth is visible at the declared frontier

### Initial Admitted Scope Classes Rule

Milestone 6 must define an initial admitted set of narrow-read shapes rather
than claiming universal aspect selectivity on day one.

Minimum admitted first-ship scope classes:

- `SingleEntityAspectScope`
  one declared entity identity plus one or more declared aspects
- `EntitySetUniformAspectScope`
  a declared entity-id set plus one declared aspect set applied uniformly
- `CdcTouchedAspectScope`
  the touched entity/aspect pairs already proven by the canonical commit or
  branch-delta control path

Required rules:

- any requested scope outside the admitted first-ship classes must produce an
  explicit typed fallback or typed rejection
- the implementation may widen the admitted scope catalog later, but Milestone
  6 must freeze the initial set now so certification and counters test a real
  boundary instead of a slogan
- the first implementation may not claim "general aspect reads" while secretly
  routing all interesting shapes through broad fallback

This is the anti-fake-selectivity rule.

### Structural Block Equivalence Rule

Milestone 6 must define structural-block sameness mechanically.

Required sameness basis:

- branch/frontier authority basis of the source material
- aspect projection or structural slice included in the block
- canonical fragment ordering inside the block
- block family version and canonicalization version
- comparator rules for semantically equivalent structural fragments
- digest basis used for block identity

Rules:

- identical bytes without semantic equivalence proof are not enough
- equivalent structural slices across branches must canonicalize to the same
  block identity or fail typed
- duplicate fragment orderings must canonicalize deterministically or reject
  explicitly
- later reuse surfaces may not invent a looser sameness contract than the one
  frozen here

This is the Law 26 protection against equivalence drift for block reuse.

### Layout Support Lane Rule

Milestone 6 must expose layout-support posture as an explicit lane choice
rather than letting callers discover it accidentally through whichever helper
they called first.

Minimum lane vocabulary:

- `ProofOnlyLayoutLane`
  admitted planning, control parity, and witness construction are allowed, but
  no durable Milestone 6 layout families are published by that choice
- `OnDemandMaterializedLayoutLane`
  admitted planning is followed by explicit publication or fetch of the durable
  Milestone 6 layout materialization and its derived scope/block/chunk families

Optional later policy lane:

- `PolicyEagerMaterializedLayoutLane`
  an operator or workload policy may choose eager publication for hot branches
  or repeatedly demanded scopes, but this remains an explicit policy posture,
  not an ambient read-path side effect

Rules:

- proof-only is a valid Milestone 6 posture and must remain visible as such in
  evidence, diagnostics, and complexity status
- on-demand materialization is the verified durable lane for Milestone 6
  support families
- later eager policies must compile to one of the explicit materialized lanes;
  they may not create a silent third behavior
- calling a narrow-read or certification API must not silently publish durable
  layout support unless the caller or a resolved operator policy explicitly
  selected a materialized lane
- if an implementation opportunistically materializes because "the data was
  already in hand," that materialization must still be represented as an
  explicit lane or policy decision in counters and evidence

This is the anti-ambient-materialization rule.

### Block And Chunk Identity Rule

Milestone 6 must define canonical identities distinct from commit, branch,
snapshot, checkpoint, and delta-layer identity.

Required identities:

- `StructuralBlockId`
- `PhysicalChunkId`
- `AspectLayoutSliceId`

Rules:

- each identity is assigned before publication of the complete derived family
- block identity corresponds to one semantically canonical structural slice
- chunk identity corresponds to one declared physical publication unit suitable
  for later deterministic bulk chunking
- rebuild either preserves the original identity under one global rule or emits
  explicit rebuild lineage under one global rule; the implementation may not mix
  both models opportunistically
- chunk identity must not mean "the commit" or "the bulk job"; it is a physical
  derived publication unit only

Milestone 6 should prefer explicit rebuild lineage whenever identity reuse
could blur operator or certification understanding.

### Block Boundary Determinism Rule

Milestone 6 must freeze what is allowed to determine a structural block
boundary.

Allowed boundary inputs:

- declared aspect scope shape
- declared branch/frontier authority basis
- canonical fragment ordering
- declared chunk-shape version

Not allowed as the sole boundary determinant:

- "whatever fit in a page"
- backend-local compression choices
- current disk pressure
- runtime ingest batch shape
- nondeterministic hash-map iteration or parallel discovery order

Rules:

- equivalent source truth under the same admitted boundary version must produce
  the same block boundaries and block identities across equivalent lanes
- if the implementation needs size caps or packing rules, those rules must be
  explicit in the chunk-shape or block-family version rather than ambient
  backend behavior

This is the anti-"dedup worked differently this run" rule.

### Aspect-Layout Publication Rule

Milestone 6 must publish aspect-layout families as complete derived units, not
as best-effort background residue.

One admitted publication unit must coherently cover:

- declared aspect scope and target frontier
- layout slice records
- structural block references or inline block payloads
- integrity and digest records
- narrow-read admissibility marker
- chunk-map metadata when the published family participates in chunked physical
  layout

Required rule:

- either the layout family is not admitted for narrow reads, or the full
  scope-plus-layout-plus-integrity unit is present and verifiable

Forbidden states:

- published layout slice with no declared scope basis
- block references with no admitted equivalence contract version
- chunk-map rows marked usable for later bulk chunking while block membership is
  incomplete
- the newest partially built layout family being selected because it happens to
  exist on disk

### Narrow-Read Admission Rule

Milestone 6 admits narrow reads only through explicit planning and explicit
fallback classes.

Required read classes:

- `AspectNarrowRead`
  read one declared aspect scope through an admitted layout family
- `AspectNarrowReplayControlRead`
  reconstruct the same requested truth through canonical authority plus
  branch-delta control paths
- `AspectBroadFallbackRead`
  explicit fallback when the requested narrow path is not admitted honestly

Rules:

- narrow reads must name the requested aspect scope and target frontier
- control reads must remain available for replay-parity verification and
  certification
- broad fallback may exist, but it must return an explicit fallback class and
  cost surface rather than masquerading as a normal narrow-read success
- execution may not silently decode whole-state structures and still report the
  result as a verified narrow read

### Chunk-Model Export Rule

Milestone 6 must define the physical chunk model once so Milestone 9 can build
bulk ingest and transform honestly on top of it.

Required chunk concepts:

- `PhysicalChunkShape`
- `ChunkMembership`
- `ChunkFrontierRange`
- `ChunkAuthorityBasisDigest`
- `ChunkDeterminismWitness`

Rules:

- a chunk is a derived physical publication unit over declared structural slices
  and authority basis, not a second commit or transaction record
- chunk shape and chunk membership must be deterministic from the declared
  authority basis and canonicalization rules
- Milestone 9 may schedule, resume, and checkpoint bulk work over these chunk
  units, but Milestone 6 owns the physical chunk identity and determinism
  contract itself
- Milestone 6 must not absorb bulk job orchestration, progress checkpoint
  semantics, or retry policy just because chunk identity is needed there

This is the concurrency boundary that lets Milestone 9 overlap honestly.

### Result-Envelope Performance Rule

Performance accounting must be embedded in narrow-read, block-lookup, and
chunk-export results rather than living only in internal diagnostics.

Minimum required result-envelope fields:

- `strategy`
- `scope_class`
- `complexity_status`
- `fallback_class`
- `layout_slices_read`
- `blocks_decoded`
- `control_replay_breadth`
- `chunk_count`

Rules:

- `complexity_status` must distinguish at least `Verified` and `Debt`
- `fallback_class` must distinguish `None` from each named fallback family
- callers must be able to tell from the returned value whether the operation
  remained inside the admitted Milestone 6 cost envelope

### Local Budget Contract Rule

Milestone 6 must define local architectural budgets now even though global
admission control lands later.

Minimum local budget contracts:

- `MaxAdmittedAspectSlicesPerRead`
- `MaxAdmittedBlockDecodeBreadth`
- `MaxAdmittedControlReplayBreadthForParity`
- `MaxDeterministicChunkWidth`

Rules:

- exceeding one of these budgets must produce a typed fallback, typed reject,
  or explicit `Debt` classification
- budgets may be configuration-backed later, but the architectural meaning of
  each budget must be fixed in this milestone
- no admitted fast path may quietly exceed its local budget and still report as
  ordinary verified success

### Compile-Time Boundary Rule

The highest-risk Milestone 6 honesty boundaries must be enforced by type
construction, not only by runtime counters.

Required compile-time posture:

- `AdmittedAspectLayoutReadPlan` may be constructed only from proof that the
  requested scope belongs to an admitted scope class and is bound to one exact
  target frontier
- `DedupAdmittedBlockReuse` may be constructed only from proof that the block
  family and equivalence contract versions match
- `ChunkModelFrozenPhysicalLayout` may be constructed only from proof that the
  chunk boundary version and authority basis digest are complete
- Milestone 7-facing reference types may not carry layout-slice ids,
  block-local packing details, or chunk placement internals
- Milestone 9-facing reference types may not carry authority-bearing commit or
  branch-head mutation rights just because they carry chunk identity

Required proof surface:

- compile-fail tests for illegal narrow-read planning from unsupported scope
  classes
- compile-fail tests for dedup reuse from raw block digests without equivalence
  proof
- compile-fail tests for chunk export from unstable or partial chunk metadata
- compile-fail tests for Milestone 7- and Milestone 9-facing references that
  attempt to smuggle forbidden internals or authority

## Proof-Carrying Layout Pipeline

Law 41 is load-bearing here too.

Minimum intended phase sequence:

- `SelectedAspectScope`
- `AspectLayoutPlan`
- `PublishedAspectLayoutFamily`
- `PublishedStructuralBlockFamily`
- `DedupAdmittedBlockReuse`
- `AdmittedAspectNarrowRead`
- `ChunkModelFrozenPhysicalLayout`
- `VerifiedLayoutParityOutcome`

Rules:

- each later type consumes the prior proof-bearing type or an explicit sibling
  proof where the branch splits
- constructors for proof-bearing layout and block types must be crate-sealed
- narrow-read execution must not accept weaker inputs than
  `AdmittedAspectNarrowRead`
- dedup-backed reuse must not accept raw block digests without the declared
  equivalence and authority-basis proofs
- Milestone 9-facing chunk references should consume
  `ChunkModelFrozenPhysicalLayout`, not raw backend records

This is what makes "partial layout family," "dedup from coincident bytes," and
"bulk chunking over unstable physical units" structurally harder to express.

## Public Surface

Milestone 6 must keep the public facade explicit and scope-oriented.

Representative surface:

```rust
pub struct AspectLayoutReadRequest { ... }
pub struct StructuralBlockLookupRequest { ... }
pub struct LayoutRebuildRequest { ... }

pub struct PublishedAspectLayoutHandle { ... }
pub struct PublishedStructuralBlockHandle { ... }
pub struct AspectLayoutReadPlan { ... }
pub struct AspectLayoutReadResult { ... }

impl WorthStore {
    pub fn plan_aspect_layout_read(
        &self,
        request: AspectLayoutReadRequest,
    ) -> Result<AspectLayoutReadPlan, AspectLayoutPlanningError>;

    pub fn execute_aspect_layout_read(
        &self,
        plan: AspectLayoutReadPlan,
    ) -> Result<AspectLayoutReadResult, AspectLayoutReadError>;

    pub fn lookup_structural_block(
        &self,
        request: StructuralBlockLookupRequest,
    ) -> Result<PublishedStructuralBlockHandle, StructuralBlockLookupError>;

    pub fn rebuild_aspect_layout(
        &mut self,
        request: LayoutRebuildRequest,
    ) -> Result<PublishedAspectLayoutHandle, AspectLayoutRebuildError>;
}
```

Surface rules:

- read APIs must expose scope, frontier, and fallback vocabulary directly
- plan and execute should remain separate if that is what keeps admissibility
  mechanically enforceable
- block lookup must remain in store-owned block vocabulary, not raw backend row
  or file coordinates
- chunk-model export surfaces may exist for Milestone 9 consumers, but they
  must expose physical chunk vocabulary rather than backend-local storage trivia
- no API may imply that aspect layout or structural blocks are authoritative
  truth rather than derived physical families

## Required Internal Subsystems

Milestone 6 must decompose by responsibility:

- `layout/scope/`
  aspect-scope identity, frontier binding, and read admission
- `layout/slices/`
  layout-slice persistence and publication
- `layout/blocks/`
  structural block identity, equivalence, and block-family persistence
- `layout/dedup/`
  cross-branch reuse planning and block-reference admission
- `layout/chunks/`
  chunk-map definition, deterministic chunk identity, and Milestone 9 export
  vocabulary
- `layout/read/`
  narrow-read planning, execution, fallback classification, and control-lane
  parity
- `layout/rebuild/`
  rebuild from authoritative artifacts
- `layout/evidence/`
  counters, integrity bundles, and certification output
- `backend/`
  backend support for layout and block families without owning their semantics

This is the `domain_laws.md` line for Milestone 6: scope selection, slice
publication, block identity, dedup planning, chunk modeling, read execution,
and rebuild do not change for the same reasons and must not share one
optimization blob.

## Invariant Allocation Table

| Invariant | Proving Phase | Enforcing Subsystem | Failure Family | Certification Surface |
| --- | --- | --- | --- | --- |
| admitted aspect-scope reads stay within declared scope | read planning and execution | `layout/scope/` and `layout/read/` | `AspectScopeBroadeningViolation` | `truth_digest` and `diagnostics_digest` |
| layout families are never admitted partially | publication | `layout/slices/` | `AspectLayoutPublicationGap` | `artifact_digest` |
| structural block identity reflects declared semantic equivalence | block publication and verification | `layout/blocks/` | `StructuralBlockEquivalenceViolation` or `StructuralBlockDigestMismatch` | `artifact_digest` |
| cross-branch dedup preserves replay and restore conclusions | dedup planning and parity verification | `layout/dedup/` and `layout/read/` | `StructuralBlockDedupParityViolation` | `truth_digest` and `restore_digest` |
| chunk identity is deterministic from declared authority basis | chunk modeling | `layout/chunks/` | `PhysicalChunkDeterminismViolation` | `artifact_digest` and chunk-model bundle |
| Milestone 9-facing chunk exports do not encode a second commit language | chunk-model export review and tests | `layout/chunks/` | `ConcurrentBulkBoundaryViolation` | concurrency parity bundle |
| deleted layout and block families remain rebuildable from authority | rebuild | `layout/rebuild/` | `AspectLayoutRebuildFailure` or `StructuralBlockRebuildFailure` | rebuild parity bundle |
| Milestone 7-facing reads do not depend on backend-local layout shape | cross-milestone boundary review and tests | `layout/scope/` and adapters for support-artifact reads | `ConcurrentSupportBoundaryViolation` | compile-fail boundary proof |
| broad fallback remains explicit rather than hidden inside admitted narrow paths | read result envelope and counter verification | `layout/read/` | `HiddenWholeStateFallbackViolation` | `diagnostics_digest` and `counter_snapshot` |

## Failure Taxonomy

Milestone 6 must ship an explicit typed error family matrix at minimum
covering:

- `AspectScopeAmbiguous`
- `AspectScopeUnsupported`
- `AspectLayoutPublicationGap`
- `AspectLayoutDigestMismatch`
- `AspectScopeBroadeningViolation`
- `AspectLayoutReadTargetIllegal`
- `AspectLayoutFallbackRequired`
- `HiddenWholeStateFallbackViolation`
- `StructuralBlockEquivalenceViolation`
- `StructuralBlockDigestMismatch`
- `StructuralBlockDedupParityViolation`
- `StructuralBlockRebuildFailure`
- `AspectLayoutRebuildFailure`
- `PhysicalChunkDeterminismViolation`
- `ConcurrentBulkBoundaryViolation`
- `ConcurrentSupportBoundaryViolation`
- `AspectLayoutFamilyVersionUnsupported`
- `StructuralBlockIntegrityFailure`

Rules:

- plan, read, dedup, chunk-export, rebuild, and verification paths must map
  failures into these families or explicit refinements of them
- backend-driver failures must not leak as the public semantic failure taxonomy
- typed failures must remain stable enough for certification bundles and later
  operator diagnostics

## Complexity Contracts

Milestone 6 must name the hot-path and rebuild-path cost basis explicitly.

Milestone 6 must also encode performance in the architecture itself, not as a
later optimization pass over an otherwise shape-ambiguous storage layer.

That means:

- admitted fast paths must be constructible only from proof-bearing inputs
- expensive fallback paths must have distinct types and distinct public
  outcomes
- traversal direction must be reflected in subsystems and access structures
- expensive facts proven once upstream must be carried forward rather than
  rediscovered inside every narrow read
- chunk determinism must be published as part of the physical artifact model,
  not merely asserted in certification

### Performance Encoding Rules

#### Regime-Typed Read Rule

Milestone 6 must make read-cost regimes explicit in the type system and public
result surface.

Minimum required regimes:

- `DirectLayoutSliceRead`
- `BlockReuseBackedRead`
- `AspectReplayControlRead`
- `ExplicitBroadFallbackRead`

Rules:

- the public read result must report one exact regime, not a vague "read
  succeeded" outcome
- execution may not silently shift from an admitted narrow regime into broad
  fallback while preserving the same result type
- if two regimes have materially different cost surfaces, they must not be
  collapsed behind one uniform cheap-looking abstraction

This is the anti-cost-dishonest abstraction rule for Milestone 6.

#### Proof-Bearing Narrow Admission Rule

An admitted narrow read must be unconstructable without the proofs that make
its boundedness claim honest.

Required proof inputs for `AdmittedAspectLayoutReadPlan`:

- proof that the requested scope belongs to one admitted scope class
- proof that one exact target frontier is bound
- proof that layout-slice breadth stays within the admitted local budget
- proof that block-decode breadth stays within the admitted local budget
- proof that any control-lane parity work required by the regime stays within
  the admitted parity budget

Rules:

- if one of those proofs is absent, the system must construct a typed fallback
  or typed rejection plan instead
- execution may not "discover later" that the narrow path was actually broad
  while still claiming the admitted regime

#### Distinct Fallback Type Rule

Broad fallback must be a separate architectural object, not merely an admitted
result plus a boolean or string note.

Required posture:

- `AdmittedAspectLayoutReadPlan`
- `ExplicitBroadFallbackPlan`
- `RejectedAspectLayoutReadPlan`

Rules:

- a caller must be able to tell from the type and public result whether they
  received the Milestone 6 admitted path or an explicit broad fallback
- internal helpers may share implementation where honest, but public planning
  and execution boundaries must preserve the distinction

#### Directional Access Rule

Milestone 6 must encode the expected traversal directions architecturally.

Required directional surfaces:

- aspect layout lookup:
  - `(scope class, authority basis) -> layout slices`
- block lookup:
  - `structural block id -> block payload`
- dedup lookup:
  - `(scope slice, equivalence contract version) -> reusable block refs`
- chunk lookup:
  - `authority basis -> chunk membership`
  - `chunk id -> chunk artifact`

Rules:

- the spec must not allow one generic normalized storage family with later
  rediscovered access patterns while still claiming Milestone 6 complexity
  bounds
- if a backend cannot support one admitted access direction honestly, that path
  must be marked `Debt` and surfaced in certification output

#### Required Physical Access Structures

Milestone 6 must name the access structures it expects implementations to
provide, even if exact backend syntax varies.

Minimum required access structures:

- aspect-layout scope index over:
  - `(scope_class, branch_id, target_frontier, aspect_set_digest)`
- layout-slice identity index over:
  - `aspect_layout_slice_id`
- layout-slice membership index over:
  - `(branch_id, target_frontier, entity_or_entity_set_key, aspect_set_digest)`
- structural-block identity index over:
  - `structural_block_id`
- structural-block equivalence index over:
  - `(equivalence_contract_version, authority_basis_digest, structural_slice_digest)`
- block-reference membership index over:
  - `aspect_layout_slice_id -> structural_block_id`
- dedup-candidate index over:
  - `(scope_slice_digest, equivalence_contract_version) -> reusable block refs`
- chunk-membership index over:
  - `(branch_id, target_frontier, chunk_shape_version, authority_basis_digest)`
- chunk-identity index over:
  - `physical_chunk_id`

Rules:

- implementations may add richer indexes or caches
- implementations may not omit the above and replace them with full scans plus
  post-filtering while still claiming the Milestone 6 complexity contracts
- if one backend cannot honestly provide one of these structures, the affected
  path must be marked `Debt` and the missing structure must be named in
  certification output

Minimum row-identity expectations:

- `aspect_layout_slice_id` must be sufficient to fetch one published slice
  family member without rescanning sibling slices
- `structural_block_id` must be sufficient to fetch one canonical block payload
  and its family/version metadata without rescanning unrelated blocks
- `physical_chunk_id` must be sufficient to fetch one chunk artifact and its
  membership metadata without rescanning sibling chunks

This is the anti-"generic table with selective marketing copy" rule.

#### Access-Structure / Path Mapping Rule

Milestone 6 must also make explicit which hot paths depend on which access
structures so backend debt cannot hide behind vague indexing language.

Minimum mapping:

- `aspect_layout_read`
  depends on:
  - aspect-layout scope index
  - layout-slice membership index
  - block-reference membership index
- `structural_block_lookup`
  depends on:
  - structural-block identity index
- `dedup_backed_read`
  depends on:
  - dedup-candidate index
  - structural-block identity index
  - block-reference membership index
- `chunk_model_export`
  depends on:
  - chunk-membership index
  - chunk-identity index

Rules:

- any backend lane that lacks one of the required structures for a named path
  must mark that path `Debt`
- a backend may not claim `Verified` for a path if it still broad-scans one of
  the authoritative or derived families listed above

#### Working-Set Versus Rebuildable Index Rule

Milestone 6 must distinguish:

- physical access structures that are durably published derived families
- hot-path in-memory working sets populated from those families
- temporary rebuild-time helpers that are not admitted runtime hot paths

Required classification for the minimum access structures:

- durably published derived families:
  - aspect-layout scope index
  - layout-slice identity index
  - layout-slice membership index
  - structural-block identity index
  - structural-block equivalence index
  - block-reference membership index
  - dedup-candidate index
  - chunk-membership index
  - chunk-identity index
- admitted in-memory working sets:
  - hot scope-to-slice lookup caches derived from the aspect-layout scope index
  - hot block payload cache derived from the structural-block identity index
  - hot chunk-membership cache derived from the chunk-membership index
- rebuild-only helpers:
  - temporary broad replay maps
  - temporary canonicalization scratch indexes
  - temporary chunk assembly maps used only during publication or rebuild

Rules:

- no admitted runtime hot path may require a rebuild-only helper in order to
  claim `Verified`
- in-memory working sets may accelerate hot paths, but they must rebuild from
  the corresponding durable derived families alone and may not become required
  semantic authority
- if an admitted hot path can only be honest while a process-local cache is
  warm, that path is `Debt` until the durable derived family is sufficient on
  its own
- rebuild-only helpers must not leak into public API types, public evidence
  claims, or Milestone 7 / Milestone 9-facing reference surfaces

This is the anti-"the cache forgot it was a cache" rule for Milestone 6.

#### Warm/Cold Path Declaration Rule

Milestone 6 must declare which named hot paths are expected to remain honest
when the process starts cold and only durable derived families are present.

Minimum cold-start expectations:

- `structural_block_lookup`
  must remain admitted from the structural-block identity index without a warm
  in-memory block cache
- `chunk_model_export`
  must remain admitted from chunk-membership and chunk-identity indexes without
  a warm chunk cache

Minimum warm-path-optional expectations:

- `aspect_layout_read`
  may use hot scope-to-slice caches for speed, but must remain functionally
  honest from durable layout families alone or be marked `Debt`
- `dedup_backed_read`
  may use hot reuse caches for speed, but the underlying dedup-candidate and
  structural-block identities must remain sufficient to preserve parity

Rules:

- certification output should be able to state whether the path was exercised
  in cold or warm regime
- a path may not claim `Verified` in general if it is only verified in warm
  regime and becomes broad-scan dependent when cold

#### Access-Structure Rebuild Rule

Every durably published Milestone 6 access structure must have an explicit
rebuild basis and rebuild boundary.

Minimum rebuild basis mapping:

- aspect-layout scope and membership indexes rebuild from:
  - published layout slices
  - declared scope identity
  - authority basis digest
- structural-block identity and equivalence indexes rebuild from:
  - published structural blocks
  - block-family version
  - canonical structural slice digest
- chunk-membership and chunk-identity indexes rebuild from:
  - published chunk artifacts
  - chunk-shape version
  - authority basis digest

Rules:

- rebuild must not require ambient process memory, warm caches, or unpublished
  scratch state
- if one access structure cannot be rebuilt from its declared durable basis,
  then it is either missing required durable inputs or is shadow authority
- rebuild of an access structure may differ in counter cost, but not in lookup
  conclusions for equivalent lanes

#### Carry-Proof Forward Rule

Expensive facts proven by canonical append or branch-delta derivation must be
carried forward into Milestone 6 planning rather than rediscovered repeatedly.

Minimum carried-forward facts where admitted:

- CDC-touched entity/aspect sets
- canonical fragment ordering for block publication
- branch/frontier-local structural slice boundaries
- chunk membership derived during layout publication

Rules:

- if a trusted upstream phase already proved one of these facts, narrow-read
  or dedup planning may not re-scan broad source truth merely to rediscover it
- later phases should consume proof-bearing forms or explicit summaries rather
  than raw payloads whenever the proof already exists inside the same trust
  boundary

This is the anti-repeated-rediscovery rule for Milestone 6.

#### Version-Typed Equivalence Rule

Structural block equivalence and chunk-boundary rules must be version-coupled
to the identities and witnesses that use them.

Required posture:

- block identity, block equivalence proof, and chunk determinism witness must
  each carry the family or boundary version they were derived under
- dedup reuse and chunk export may only consume matching versions unless an
  explicit compatibility bridge exists

Rules:

- a block produced under one boundary or canonicalization version may not be
  silently reused under another
- "the bytes happened to match" is insufficient when the governing equivalence
  version differs

#### Chunk Witness Rule

Milestone 9 must consume a published chunk-determinism contract from Milestone
6 rather than recomputing or guessing chunk identity ad hoc.

Required rule:

- `ChunkDeterminismWitness` is produced when the chunk model is frozen for one
  admitted authority basis
- Milestone 9-facing chunk references must require that witness or a proof type
  derived from it
- chunk export without a determinism witness is not an admitted Milestone 6
  fast path

This is the anti-"bulk invented its own physical units" rule.

#### Complexity-Status Surface Rule

Milestone 6 evidence must publish path-local complexity status rather than one
rolled-up milestone status.

Minimum named paths:

- `aspect_layout_read`
- `structural_block_lookup`
- `dedup_backed_read`
- `chunk_model_export`

Rules:

- each path must declare at least `Verified` or `Debt`
- any path marked `Debt` must name the missing proof, missing access
  structure, or unresolved broadening condition
- certification output must not let one honest path hide another dishonest one

#### Compile-Time Performance Boundary Rule

The highest-risk performance boundaries must remain compile-time enforced even
after runtime counters exist.

Required compile-time posture:

- `AdmittedAspectLayoutReadPlan` may not be caller-synthesized from raw scope
  request fields
- `DedupAdmittedBlockReuse` may not be constructed from raw digest equality
  alone
- `ChunkModelFrozenPhysicalLayout` may not be constructed from partial chunk
  metadata
- Milestone 7-facing and Milestone 9-facing references must fail to compile if
  they attempt to carry forbidden layout internals or authority-bearing rights

Required proof surface:

- compile-fail tests for illegal admitted-plan construction
- compile-fail tests for raw-digest dedup witness construction
- compile-fail tests for partial chunk-freeze construction
- compile-fail tests for forbidden cross-milestone reference leakage

Minimum contracts:

- admitted aspect-layout read cost is proportional to:
  - layout slices touched for the declared aspect scope
  - structural blocks decoded for the declared scope
  - declared control-lane parity breadth for verification
- structural-block lookup cost is proportional to:
  - one block identity lookup
  - block references and block payloads read for that exact identity
  - equivalence-basis validations required for the admitted family
- dedup-backed read cost is proportional to:
  - block reuse hits consulted for the declared scope
  - block decode breadth actually needed for the read
  - not total branch-state width
- rebuild cost is proportional to:
  - authoritative branch/frontier range replayed into layout and block families
  - layout slices and blocks re-emitted
- chunk-model export cost is proportional to:
  - chunks enumerated for the declared authority basis
  - chunk-membership rows emitted
  - not total historical bulk-job count

Forbidden fallback work that must be made mechanically visible:

- hidden whole-state decode during admitted narrow reads
- hidden canonical replay fallback on every narrow-read path
- hidden broad block-scan search for block lookup
- hidden chunk reshaping during Milestone 9-facing export that changes physical
  units under the caller

Minimum counters:

- `aspect_layout_read_count`
- `aspect_layout_scope_lookup_count`
- `aspect_layout_slice_rows_read`
- `aspect_layout_whole_state_fallback_count`
- `aspect_layout_decode_breadth`
- `structural_block_lookup_count`
- `structural_block_reuse_hit_count`
- `structural_block_reuse_miss_count`
- `structural_block_decode_breadth`
- `structural_block_rebuild_count`
- `structural_block_integrity_failure_count`
- `aspect_layout_rebuild_count`
- `aspect_layout_rebuild_slice_count`
- `physical_chunk_export_count`
- `physical_chunk_width_count`
- `physical_chunk_determinism_violation_count`
- `concurrent_bulk_boundary_rejection_count`
- `concurrent_support_boundary_rejection_count`

Required counter assertions:

- `aspect_layout_whole_state_fallback_count` must remain zero for the
  representative admitted narrow-read lanes; any non-zero lane must be named as
  explicit fallback or explicit debt
- `structural_block_reuse_hit_count` and `structural_block_reuse_miss_count`
  must distinguish dedup lanes rather than collapsing into one generic lookup
  counter
- `physical_chunk_determinism_violation_count` must remain zero for admitted
  Milestone 6 chunk-model exports
- `concurrent_bulk_boundary_rejection_count` and
  `concurrent_support_boundary_rejection_count` must remain zero for admitted
  Milestone 9- and Milestone 7-facing reference surfaces

Debt posture:

- if the first implementation still needs broad decode or control-lane replay
  on some admitted paths, those paths must be marked `Debt` with the exact
  triggering conditions named
- Milestone 6 may not imply verified narrowing while relying on silent broad
  scans

## Phases

### Phase 1: Lock Aspect Scope, Block Sameness, And Chunk Vocabulary

Phase 1 defines what physical narrowing and structural reuse are allowed to
mean before backend-specific layout work lands.

Required work:

- define aspect-scope identity and target-frontier vocabulary
- define the initial admitted scope classes
- define the layout non-authority rule and structural-block equivalence
  contract
- define block, layout-slice, and chunk identities
- define block-boundary determinism and local budget contracts
- define the proof-bearing layout pipeline
- define narrow-read, control-read, and explicit fallback classes
- define compile-time witness types and compile-fail proof boundaries
- define the concurrency boundary with Milestone 7 and Milestone 9 in
  structural terms

Exit condition:

- layout and block families have one exact authority relationship
- chunk identity and determinism vocabulary are frozen
- Milestone 9 has an honest physical unit to inherit without taking over this
  milestone

### Phase 2: Persist Aspect Layout Families And Structural Blocks

Phase 2 makes aspect-layout slices and structural blocks real as derived
artifact families.

Required work:

- implement aspect-layout slice publication with scope and integrity records
- implement structural-block publication with canonical equivalence and digest
  basis
- implement publication atomicity and admissibility boundaries
- expose typed publication and integrity failures
- emit exact slice, block, and chunk-model counters

Exit condition:

- layout and block families can be durably published as complete derived
  artifacts
- partial or damaged families are not admitted for narrow reads or chunk-model
  export
- block identity and chunk identity are machine-checkable

### Phase 3: Expose Narrow Reads, Dedup Reuse, And Control Lanes

Phase 3 turns layout and block families into real store read surfaces instead
of latent bytes on disk.

Required work:

- implement aspect narrow-read planning and execution
- implement explicit broad-fallback and control-lane read surfaces
- implement explicit proof-only and on-demand-materialized layout-support lane
  selection without hidden auto-publication
- implement structural-block lookup and cross-branch dedup reuse planning
- expose strategy-typed result envelopes with fallback and breadth surfaces
- expose typed illegal-target, broadening, and parity failures
- emit exact read, reuse-hit, reuse-miss, and decode-breadth counters

Exit condition:

- admitted narrow reads remain scope-explicit and cost-honest
- dedup-backed reads are mechanically comparable with control-lane replay
- broad fallback is explicit rather than ambient

### Phase 4: Freeze Chunk-Model Export And Rebuild Boundaries

Phase 4 makes the Milestone 6 physical contract reusable by later programs.

Required work:

- implement deterministic chunk-map export from admitted layout families
- prove chunk identity stability for equivalent authority bases
- implement rebuild of layout and structural-block families from authority
- expose Milestone 9-facing chunk references that remain physical and
  non-authoritative
- expose Milestone 7-facing read/reference boundaries that remain independent
  from layout internals
- prove compile-time boundary enforcement with UI or compile-fail tests
- emit chunk-width, rebuild, and boundary counters

Exit condition:

- Milestone 9 can start from a frozen chunk model instead of inventing one
- deleted layout and block families are rebuildable from authority
- support-artifact and bulk consumers remain insulated from backend-local layout
  shape

### Phase 5: Prove Narrowing Integrity And Structural-Block Dedup Parity

Phase 5 turns Milestone 6 into a certifiable physical-layout substrate rather
than an optimistic optimization pass.

Required work:

- run the Milestone 6 named suite:
  `Aspect-Layout Narrowing And Structural-Block Dedup Integrity Test`
- compare admitted narrow-read lanes against control reconstruction lanes
- compare dedup and non-dedup lanes across branch overlap cases
- compare rebuilt layout/block families against originally published truth
  surfaces
- prove compile-time boundary cases for admitted scope classes, dedup witness
  construction, and chunk-model export witnesses
- emit machine-checkable truth, artifact, diagnostics, and counter bundles

Exit condition:

- admitted fast paths match authoritative truth
- fallback broadening remains explicit
- block dedup does not change replay or restore conclusions
- Milestone 6 closeout evidence exists in machine-checkable form

## Must Ship

- aspect-aware physical layout with explicit aspect-scope vocabulary
- content-addressed structural block family with declared semantic equivalence
- cross-branch deduplication over structural blocks
- explicit narrow-read, broad-fallback, and control-lane read surfaces
- explicit proof-only and on-demand-materialized layout-support lanes
- deterministic chunk-model export suitable for concurrent Milestone 9 work
- rebuild of layout and block families from canonical authoritative artifacts
- stable Milestone 7- and Milestone 9-facing boundary surfaces that do not
  expose backend-local layout internals
- compile-time witness and compile-fail enforcement for admitted scope, dedup,
  and chunk-export boundaries
- typed layout, block, dedup, chunk, and rebuild failures
- exact counters and machine-checkable Milestone 6 certification output

## Must Preserve

- canonical commit history remains the only semantic durability authority
- branch/frontier truth remains replay-stable and subordinate only to canonical
  authority plus admitted branch-delta control paths
- aspect layout, structural blocks, dedup indexes, and chunk maps remain
  derived durable artifacts
- block reuse changes cost only, not truth meaning
- chunk-model export does not become a second commit or bulk-truth language
- Milestone 7 support-artifact meaning remains independent from physical-layout
  internals
- backend variation does not change truth-visible read, replay, or restore
  conclusions

## Acceptance Evidence

Milestone 6 is complete only when the store satisfies the named Milestone 6
suite:

- `Aspect-Layout Narrowing And Structural-Block Dedup Integrity Test`

Required machine-checkable outputs:

- `truth_digest`
- `artifact_digest`
- `diagnostics_digest`
- `counter_snapshot`

Milestone-specific proof obligations:

- admitted fast paths match authoritative truth
- fallback broadening remains explicit rather than hidden
- proof-only versus on-demand-materialized posture remains explicit in evidence
  and never changes silently under the caller
- block dedup does not change replay or restore conclusions
- rebuilt layout and block families match original truth-visible meaning
- Milestone 9-facing chunk exports remain deterministic and non-authoritative
- Milestone 7-facing consumers do not inherit physical-layout coupling
- invalid narrow-read, dedup, and chunk-export constructions are mechanically
  rejected rather than only failing at runtime

Milestone 6 is not closed by "partial reads were faster" or "dedup saved disk"
tests.

## Architectural Notes

- The smart abstraction is not "columnar-ish storage." The smart abstraction is
  one exact aspect-scope-plus-structural-block contract with explicit fallback,
  control-lane parity, and rebuildability.
- Layout shape may vary by backend, but scope identity, block equivalence,
  chunk determinism, and non-authority rules may not.
- Chunk-model export is part of Milestone 6 because physical units must be
  frozen before Milestone 9 can build resumable bulk execution honestly.
- Milestone 6 should prefer deterministic block and chunk lineage over hidden
  rewrite or cache residue whenever operator or certification understanding
  would otherwise blur.
- Milestone 8 should consume the narrow read substrate defined here instead of
  renegotiating what "aspect-local" means.

## Sequencing Notes

This milestone belongs immediately after Milestone 5 because physical
narrowing, block reuse, and chunk identity need an already-honest branch-delta
substrate before they can be trusted.

- `Milestone 7` should continue in parallel because support-artifact meaning is
  branch/frontier based and must not depend on aspect-layout or block-layout
  internals.
- `Milestone 9` should begin concurrently only after Phase 4 freezes the chunk
  model, because bulk orchestration needs stable physical units but must not
  define them independently.
- `Milestone 8` depends on this milestone together with `Milestone 7` because
  live-query continuation needs both honest physical narrowing and honest
  durable support truth.
- `Milestone 10` depends on this milestone together with `Milestone 4` and
  `Milestone 5` because retention and compaction cannot reason honestly about
  pruning and rebuild until snapshot, branch-delta, and layout/block families
  are all explicit.
