# Milestone 4 Engineering Spec: Snapshot Persistence And Point-In-Time Restore

> **Status:** Closed 2026-04-14
>
> **Roadmap parent:** `worth_store_roadmap.md`
>
> **Vision parent:** [worth_store_vision.md](./worth_store_vision.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Prerequisite milestones:**
> - [milestone-1.md](./milestone-1.md)
> - [milestone-1-closeout.md](./milestone-1-closeout.md)
> - [milestone-2.md](./milestone-2.md)
> - [milestone-2-closeout.md](./milestone-2-closeout.md)
> - [milestone-3.md](./milestone-3.md)
> - [milestone-3-closeout.md](./milestone-3-closeout.md)
>
> **Primary architectural driver:** make snapshots a fast, immutable, derived
> restore substrate without letting them become a second source of truth

## Goal

Make immutable snapshots and snapshot-plus-tail restore first-class derived
durable artifacts so the store can serve bounded historical reads and fast
restore without weakening canonical replay authority.

## Why This Milestone Exists

Milestone 4 is not "serialize state sometimes."

It is the milestone that decides whether `worth-store` can accelerate reads and
restore honestly, or whether it quietly creates a second truth format that
future code will start trusting because it is faster.

Milestone 1 locked canonical durable authority.

Milestone 2 locked who is allowed to host runtime execution and who is allowed
to persist externally produced artifacts.

Milestone 3 locked the durable crash boundary and recovery control lane.

Milestone 4 now has to lock a different but equally dangerous boundary:

- what a snapshot is allowed to mean
- what exact basis a snapshot captures
- what exact point-in-time read guarantees a snapshot can support
- what exact restore work may be skipped because a snapshot exists
- what exact truth still has to come from canonical replay

If this milestone is weak, later multi-resolution materialization, retention,
replication, tiering, and analysis lanes will all start depending on snapshots
as though they were authority. That is exactly the failure this milestone
exists to prevent.

## Hard Part

The hard part is not writing out a big blob of state.

The hard part is freezing one exact truth-preserving relationship among three
different things that naive designs constantly blur together:

- the canonical authoritative commit history
- the derived immutable state image at one exact basis
- the suffix replay needed to move from that basis to another point in history

The design fails if:

- a snapshot can be interpreted without its exact basis
- a point-in-time read can drift across multiple truth versions during capture
- restore is allowed to trust snapshot-local structure that cannot be rebuilt
  from canonical authority
- deleting snapshots changes what truth can be recovered
- later code starts using "latest snapshot" as a substitute for canonical
  replay or branch-head authority

Milestone 4 therefore has to make snapshots cheap to use, but structurally
incapable of becoming semantic authority.

## Explicit Assumptions

- Milestone 1 authoritative artifact families remain the only semantic durable
  truth authority.
- Milestone 2 operating-mode boundaries remain unchanged; snapshot capture and
  restore are store-owned durable capabilities, not embedded-checkpoint
  semantics.
- Milestone 3 durable recovery is already exact enough that snapshot capture
  may start from a recovery-verified store rather than an ambiguous restart
  state.
- `worth-relational` still owns truth semantics, MVCC snapshot meaning, branch
  history meaning, ordered parent meaning, and canonical replay semantics.
- snapshots in this milestone are full immutable snapshots for an admitted
  truth scope; multi-resolution or partial materializations remain later work.
- snapshots are derived durable artifacts and must remain destroyable and
  rebuildable from authoritative artifacts alone.
- point-in-time restore in this milestone means "restore from immutable
  snapshot basis plus canonical suffix history," not "skip replay entirely."
- retention, compaction, branch delta layering, and replication are still out
  of scope except where the spec must reserve honest boundaries for them.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects is solving the hostile boundary first.
  Milestone 4 therefore starts from restore exactness and snapshot
  non-authority, not from "read performance" or "startup speed."
- `arch_laws.md`
  The most important thing it protects here is proof-bearing phase separation.
  Law 41 matters especially: chosen snapshot basis, captured snapshot image,
  restore-admitted snapshot, and rebuilt snapshot equivalence must be distinct
  types with sealed transitions. Law 33 and Law 36 also matter directly:
  authoritative truth and derived state must stay categorically distinct, and a
  system must be reconstructable from a checkpoint plus bounded journal without
  letting the checkpoint become authority.
- `perf_laws.md`
  The most important thing it protects is honest boundedness. Snapshot capture,
  snapshot load, point-in-time read, and tail replay must expose named cost
  bases and exact counters rather than hiding amplification behind
  "restores are faster."
- `domain_laws.md`
  The most important thing it protects is responsibility-shaped decomposition.
  Milestone 4 must separate snapshot basis selection, snapshot image
  persistence, point-in-time reads, restore planning, restore execution, and
  certification evidence rather than burying them in one persistence helper.
- `worth_store_vision.md`
  The most important thing it protects is that snapshots are derived durable
  artifacts used for bounded reads and recovery acceleration, while canonical
  commit envelopes remain the authority.
- `worth_store_roadmap.md`
  The most important thing it protects is order. Milestone 4 belongs here
  because retention, replication, and later materialization families are
  dishonest until snapshot basis and restore semantics are frozen first.
- `worth-store/test-requirements.md`
  The most important thing it protects is exact restore equivalence. Milestone
  4 is not closed until the `Snapshot-Plus-Tail Restore Equivalence Test`
  proves restore parity, rebuild parity, and snapshot non-authority.
- `milestone-3.md`
  The most important thing it protects is a real durable crash boundary and a
  first-class rebuild control lane. Snapshot restore must build on that exact
  recovery substrate instead of creating a bypass around it.
- `milestone-3-closeout.md`
  The most important thing it protects is that durable publication, restart,
  and rebuild are already certifiable. Milestone 4 should therefore treat
  snapshots as a bounded acceleration lane over known-good authority, not a
  new recovery authority.
- `worth_relational_vision.md`
  The most important thing it protects is MVCC snapshot meaning and canonical
  replay from commit artifacts. Store snapshots must therefore preserve
  relational snapshot basis honestly instead of inventing storage-local state
  image semantics.
- `worth_relational_roadmap.md`
  The most important thing it protects is immutable snapshots, stable reads
  under mutation, and deterministic replay. Milestone 4 depends on those
  runtime truths instead of redefining them inside storage.
- `worth_runtime_bridge_vision.md`
  The most important thing it protects is snapshot-backed deterministic
  evaluation against stable truth. Store snapshots must therefore present
  branch-and-version-explicit read bases that future bridge consumers can trust
  without reading live mutable state.
- `worth_runtime_bridge_roadmap.md`
  The most important thing it protects is historical and branch-aware
  evaluation over intentional truth surfaces. Milestone 4 must preserve that
  shape by making point-in-time snapshot reads explicit and replay-safe rather
  than backend-local conveniences.

## Adversarial Constraint

Milestone 4 must survive this hostile condition:

> A store that has captured immutable snapshots, deleted some of them, rebuilt
> them from canonical authority, and restored from snapshot-plus-tail across
> branch-local history must converge to the same committed truth, branch-head
> conclusions, and replay-visible state as a control lane that replays the same
> canonical commit history without snapshots at all.

## Product Decision Lock

- snapshots are always classified as derived durable artifacts
- snapshot capture always binds to one exact canonical truth basis; "latest
  state at roughly this time" is out of spec
- snapshots are immutable once published
- point-in-time reads from snapshots are reads of one declared truth basis, not
  live views that may drift under concurrent mutation
- snapshot-plus-tail restore remains subordinate to canonical replay semantics;
  the tail is replayed through the same runtime truth rules as ordinary history
- deleting all snapshots must leave canonical truth replay and full
  authoritative rebuild intact
- snapshot rebuild must recreate derived snapshot meaning from authoritative
  artifacts rather than copying backend-local residue
- restore eligibility must depend on complete snapshot identity, basis, and
  integrity evidence; partial or half-published snapshots are never admitted as
  restore bases

Normative consequence:

- any implementation that treats a snapshot as a substitute for branch-head
  authority or canonical history authority is out of spec
- any implementation that restores from snapshot bytes without proving the
  declared basis and suffix history is out of spec
- any implementation whose rebuilt snapshots differ in truth-visible meaning
  from originally captured snapshots is out of spec

## Scope

### In Scope

- immutable persisted full snapshots for admitted store truth scopes
- exact snapshot basis records tying each snapshot to branch, commit frontier,
  and canonical authority identity
- point-in-time snapshot reads against an explicit immutable basis
- snapshot-plus-tail restore against canonical suffix history
- snapshot rebuild from canonical authoritative artifacts
- snapshot integrity records and certification bundles
- counters and diagnostics for snapshot capture, read, restore, rebuild, and
  fallback breadth
- backend support sufficient to persist snapshot image data together with basis
  and identity records

### Explicitly Out Of Scope

- partial or multi-resolution materialization families
- branch delta layering and delta-stack rewrite policy
- retention-driven snapshot pruning or compaction policy beyond the minimum
  needed to keep snapshot identity honest
- replication capsules and snapshot shipping across machines
- embedded-mode checkpoint semantics
- live-query continuation over snapshot bases
- analysis checkpoint lanes and other basis-pinned derived families that are
  not plain immutable truth snapshots

## Snapshot Authority Model

### Snapshot Non-Authority Rule

Snapshots are derived durable artifacts.

They are allowed to accelerate:

- point-in-time reads
- restore
- later replication and materialization programs

They are not allowed to define:

- canonical commit history
- branch-head authority
- ordered parent meaning
- lineage meaning
- schema-boundary meaning

Normative rule:

- if all snapshots are deleted, the store must still be able to recover the
  same canonical truth through authoritative replay and rebuild
- if a snapshot disagrees with canonical replay, the snapshot is wrong and must
  be rejected or rebuilt; canonical replay is not allowed to bend toward the
  snapshot

This is the anti-shadow-authority line for Milestone 4.

### Snapshot Basis Rule

Every admitted snapshot must bind to one exact immutable snapshot basis.

Minimum basis fields:

- `SnapshotId`
- `SnapshotBranchId`
- `SnapshotCommitFrontier`
- `SnapshotHistoryRange`
- `SnapshotCanonicalizationVersion`
- `SnapshotAuthorityDigest`

Required meaning:

- `SnapshotBranchId`
  identifies the branch-local truth scope whose point-in-time image is being
  captured
- `SnapshotCommitFrontier`
  identifies the exact canonical commit frontier whose visible truth the
  snapshot represents
- `SnapshotHistoryRange`
  identifies the closed authoritative history interval whose replay is already
  incorporated into the snapshot image
- `SnapshotCanonicalizationVersion`
  identifies the admitted canonicalization rules used for snapshot image
  identity and integrity
- `SnapshotAuthorityDigest`
  binds the snapshot to the authoritative artifact identity it claims to
  summarize

Normative rules:

- one snapshot basis corresponds to one exact point-in-time truth surface
- a snapshot may not be published against "current branch head at capture end";
  the branch and frontier must be selected before image publication
- if the branch head advances during capture, that later truth belongs to tail
  replay, not to the snapshot image

This is the line that prevents mixed-version snapshots.

### Snapshot Capture Source Rule

Milestone 4 must be explicit about what a snapshot is captured from.

Required rule:

- snapshot capture consumes one runtime-provided immutable truth basis, not a
  live mutable branch view

Admitted capture source in this milestone:

- a recovery-safe, branch-explicit, runtime truth snapshot whose visible state
  already corresponds to one exact canonical frontier

Not admitted as capture source:

- live mutable branch state that may continue changing while image extraction
  proceeds
- backend-local row scans that are not proven to correspond to one immutable
  runtime truth basis
- "whatever branch head is current when the last image record is written"

Normative consequence:

- if store cannot prove the runtime truth basis was immutable and frontier
  explicit before image publication, the snapshot is not admissible

This is the line that keeps store snapshots subordinate to relational MVCC
truth instead of creating a storage-local approximation of it.

### Snapshot Identity Rule

Milestone 4 must define one canonical `SnapshotId` that is distinct from:

- commit identity
- branch identity
- durable mutation identity
- checkpoint identity

Required rules:

- `SnapshotId` is assigned before snapshot image publication begins
- one complete immutable snapshot publication corresponds to one `SnapshotId`
- a rebuilt snapshot receives:
  - either the original `SnapshotId` only when the rebuilt artifact is bitwise
    and semantically the same admitted snapshot family instance, or
  - a distinct `SnapshotId` plus explicit `RebuildsSnapshotId` lineage when the
    implementation chooses to model rebuild as a new derived durable artifact
    instance
- whichever choice is taken, the rule must be global and mechanical; the store
  may not mix both models opportunistically
- `SnapshotId` is never overloaded to mean "the commit this snapshot is based
  on"

Milestone 4 should prefer explicit snapshot lineage over identity reuse if
there is any risk of operator or certification ambiguity.

### Snapshot Capture Atomicity Rule

Snapshot publication must be atomic at the snapshot-family boundary.

One admitted snapshot publication unit must coherently cover:

- selected snapshot basis
- immutable snapshot image records
- snapshot integrity and digest records
- restore-eligibility marker

Required rule:

- either the snapshot is not yet admitted for read or restore, or the full
  basis-plus-image-plus-integrity unit is present and verifiable

Forbidden states:

- published snapshot image without basis records
- basis records marked restore-eligible while the image is incomplete
- integrity records present for only part of the snapshot family
- point-in-time reads selecting a partially captured snapshot because it is the
  newest one on disk

This is the same structural honesty Milestone 3 applied to the durable commit
boundary, but now for derived snapshot publication.

### Point-In-Time Read Rule

Milestone 4 admits point-in-time reads only through explicit immutable
snapshot bases.

Required rule:

- a point-in-time snapshot read must declare:
  - the target branch or declared truth scope
  - the target snapshot basis
  - whether the caller expects pure snapshot truth or snapshot-plus-tail truth

Point-in-time read classes in this milestone:

- `PureSnapshotRead`
  read only the truth captured in one immutable snapshot basis
- `SnapshotTailRead`
  read snapshot basis truth plus replay the canonical suffix needed to reach a
  declared later frontier

Rules:

- `PureSnapshotRead` may not silently broaden into live branch-head reads
- `SnapshotTailRead` must declare the target frontier explicitly; "latest after
  snapshot" is too ambiguous for Milestone 4
- both read classes must return truth that can be restated in canonical
  branch-and-frontier terms

The store is not allowed to expose "snapshot-ish reads" that hide the basis.

### Snapshot-Plus-Tail Restore Rule

Snapshot-plus-tail restore is a restore acceleration lane, not a second replay
engine.

Required restore model:

`select admitted immutable snapshot basis -> load snapshot image ->
identify canonical suffix history after snapshot frontier -> replay suffix
through ordinary runtime truth semantics -> verify restored truth against the
declared target frontier`

Normative rules:

- the suffix is defined by canonical authoritative commit history, not by
  backend-local snapshot metadata alone
- restore may skip replay of the already-captured prefix, but it may not skip
  replay of the suffix
- if the suffix history is unavailable or inconsistent with the declared
  snapshot basis, restore must fail explicitly rather than "restoring as far as
  possible"
- branch-head movement after the snapshot basis must remain tail work unless
  the target frontier equals the snapshot frontier exactly

This is the anti-"snapshot restore as hidden truth shortcut" rule.

### Snapshot Restore Admissibility And Frontier Legality

Milestone 4 must make restore admissibility exact rather than convenient.

An admitted snapshot restore requires:

- one `PublishedSnapshotArtifact`
- one declared target branch scope
- one declared target frontier
- one exact canonical suffix history from the snapshot frontier to the target
  frontier

Allowed target-frontier classes in this milestone:

- `TargetEqualsSnapshotFrontier`
  pure snapshot restore with zero suffix replay
- `TargetDescendsFromSnapshotFrontierOnSameDeclaredBranchScope`
  snapshot-plus-tail restore through canonical suffix replay

Rejected target-frontier classes in this milestone:

- target frontier older than the snapshot frontier
- target frontier on a different branch scope without an explicit admitted
  cross-branch restore rule
- target frontier whose canonical suffix is ambiguous, unavailable, or not
  branch-consistent with the snapshot basis

Required restore outcomes:

- `SnapshotRestoreEquivalent`
- `SnapshotRestoreRejected`
- `SnapshotRestoreFailed`

Milestone 4 may not leave target-frontier legality as backend or caller
convention.

### Snapshot Rebuild Rule

Milestone 4 must define one explicit snapshot rebuild lane.

Snapshot rebuild consumes:

- canonical authoritative artifacts
- admitted snapshot basis specification
- the snapshot family format defined by this milestone

Snapshot rebuild does not consume:

- previously materialized snapshot-local acceleration metadata unless that
  metadata is itself declared part of the admitted snapshot family
- backend-private residue from a deleted or corrupted snapshot

Required rebuild conclusions:

- `RebuiltSnapshotEquivalent`
  the rebuilt snapshot matches the original snapshot's truth-visible meaning
- `SnapshotRebuildFailure`
  the rebuild cannot recreate the declared snapshot family honestly and must
  fail explicitly

Rebuild is allowed to differ in counters or low-level persistence mechanics.
It is not allowed to differ in point-in-time truth meaning.

## Proof-Carrying Snapshot Pipeline

Law 41 is load-bearing here.

Milestone 4 should encode snapshot work as a proof chain rather than as one
helper that "kind of knows" whether a snapshot is ready.

Minimum intended phase sequence:

- `SelectedSnapshotBasis`
- `CaptureAdmittedSnapshotPlan`
- `PersistedSnapshotImage`
- `PublishedSnapshotArtifact`
- `RestoreAdmittedSnapshot`
- `RebuiltSnapshotArtifact`
- `VerifiedSnapshotRestoreOutcome`

Rules:

- each later type consumes the prior proof-bearing type
- constructors for proof-bearing snapshot types must be crate-sealed
- fields carrying snapshot basis and integrity evidence must remain private
- restore execution must not accept a weaker type than
  `RestoreAdmittedSnapshot`
- rebuild equivalence checks must consume explicit `PublishedSnapshotArtifact`
  or `RebuiltSnapshotArtifact` proofs, not raw backend records

This is what makes "half-published snapshot," "snapshot with unknown basis,"
and "restore from unverified bytes" structurally harder to express.

### Snapshot Family Compatibility Rule

Milestone 4 must define version and family compatibility now rather than
letting future snapshot readers guess.

Required metadata:

- `snapshot_family_version`
- `snapshot_basis_version`
- `snapshot_image_format_version`

Rules:

- capture must publish these versions as part of the admitted snapshot family
- read, restore, and rebuild must either:
  - support the admitted older versions through explicit compatibility readers,
    or
  - reject them explicitly and typed
- future-added snapshot fields must not silently change the truth meaning of an
  older admitted snapshot family version
- rebuilt snapshots must preserve the declared compatibility story of the
  family they claim to recreate

This is the anti-"old snapshot bytes happened to deserialize" rule.

## Public Surface

Milestone 4 must keep the public facade explicit and basis-oriented.

Representative surface:

```rust
pub struct SnapshotCaptureRequest { ... }
pub struct SnapshotReadRequest { ... }
pub struct SnapshotRestoreRequest { ... }

pub struct PublishedSnapshotHandle { ... }
pub struct SnapshotRestorePlan { ... }
pub struct SnapshotRestoreOutcome { ... }

impl WorthStore {
    pub fn capture_snapshot(
        &mut self,
        request: SnapshotCaptureRequest,
    ) -> Result<PublishedSnapshotHandle, SnapshotCaptureError>;

    pub fn read_snapshot(
        &self,
        request: SnapshotReadRequest,
    ) -> Result<SnapshotReadResult, SnapshotReadError>;

    pub fn plan_snapshot_restore(
        &self,
        request: SnapshotRestoreRequest,
    ) -> Result<SnapshotRestorePlan, SnapshotRestorePlanningError>;

    pub fn execute_snapshot_restore(
        &self,
        plan: SnapshotRestorePlan,
    ) -> Result<SnapshotRestoreOutcome, SnapshotRestoreError>;

    pub fn rebuild_snapshot(
        &mut self,
        snapshot_id: SnapshotId,
    ) -> Result<PublishedSnapshotHandle, SnapshotRebuildError>;
}
```

Surface rules:

- snapshot APIs must expose basis and target-frontier vocabulary directly, not
  hide them behind "load latest snapshot" convenience
- restore planning and restore execution should remain distinct public concepts
  if the implementation needs an explicit admissibility boundary
- read and restore surfaces must stay in store-owned vocabulary, not raw
  backend row or file vocabulary
- no API may imply that a snapshot is authoritative truth rather than a
  declared derived basis artifact

## Required Internal Subsystems

Milestone 4 must decompose by responsibility:

- `snapshot/basis/`
  basis selection, branch/frontier binding, and basis digest identity
- `snapshot/image/`
  immutable snapshot image persistence and integrity
- `snapshot/read/`
  point-in-time read surfaces over admitted snapshot bases
- `snapshot/restore/`
  restore planning, suffix selection, restore execution, and restore parity
- `snapshot/rebuild/`
  rebuild from authoritative artifacts
- `snapshot/evidence/`
  counters, equivalence bundles, and certification output
- `backend/`
  backend support for snapshot families without owning snapshot semantics

This is the `domain_laws.md` line for Milestone 4: snapshot basis, image
publication, restore planning, restore execution, and rebuild do not change for
the same reasons and must not share one god-module.

## Invariant Allocation Table

| Invariant | Proving Phase | Enforcing Subsystem | Failure Family | Certification Surface |
| --- | --- | --- | --- | --- |
| snapshot basis identifies one exact branch-local truth frontier | basis selection | `snapshot/basis/` | `SnapshotBasisAmbiguous` or `SnapshotBasisUnsupported` | `artifact_digest` and `truth_digest` |
| partially captured snapshots are never restore-admitted | snapshot publication | `snapshot/image/` | `SnapshotPublicationStateGap` | `failure_digest` |
| snapshot image integrity matches declared basis digest | snapshot verification | `snapshot/image/` | `SnapshotDigestMismatch` | `artifact_digest` |
| pure snapshot reads do not drift beyond selected basis | read admission | `snapshot/read/` | `SnapshotReadBasisMismatch` | `truth_digest` |
| snapshot-plus-tail restore replays the exact canonical suffix | restore planning plus execution | `snapshot/restore/` | `SnapshotTailRangeGap` or `SnapshotRestoreParityViolation` | `restore_digest` |
| deleted snapshots remain rebuildable from authority | rebuild | `snapshot/rebuild/` | `SnapshotRebuildFailure` | rebuild equivalence bundle |
| rebuilt snapshots match original snapshot-visible truth | certification comparison | `snapshot/rebuild/` and `snapshot/evidence/` | `SnapshotRebuildParityViolation` | `truth_digest` and `restore_digest` |
| snapshots never override authoritative replay conclusions | restore verification | `snapshot/restore/` | `SnapshotShadowAuthorityViolation` | control-vs-restore parity bundle |

## Failure Taxonomy

Milestone 4 must ship an explicit typed error family matrix at minimum
covering:

- `SnapshotBasisAmbiguous`
- `SnapshotBasisUnsupported`
- `SnapshotCaptureSourceNotImmutable`
- `SnapshotPublicationStateGap`
- `SnapshotDigestMismatch`
- `SnapshotReadBasisMismatch`
- `SnapshotUnsupportedReadMode`
- `SnapshotRestoreTargetIllegal`
- `SnapshotTailRangeGap`
- `SnapshotRestoreParityViolation`
- `SnapshotRebuildFailure`
- `SnapshotRebuildParityViolation`
- `SnapshotShadowAuthorityViolation`
- `SnapshotFamilyVersionUnsupported`
- `SnapshotIntegrityFailure`

Rules:

- capture, read, restore, rebuild, and verification paths must map failures
  into these families or explicit refinements of them
- backend-driver or file-format failures must not leak as the public semantic
  error taxonomy
- typed failures must remain stable enough for certification bundles and later
  operator diagnostics

## Complexity Contracts

Milestone 4 must name the hot-path and restore-path cost basis explicitly.

Minimum contracts:

- snapshot capture cost is proportional to:
  - selected truth-scope width at the admitted basis
  - immutable image bytes written
  - snapshot digest work for the admitted family
- pure snapshot read cost is proportional to:
  - immutable image records read for the admitted scope
  - snapshot decode breadth
- snapshot-plus-tail restore cost is proportional to:
  - immutable image load breadth
  - canonical suffix commit count after the snapshot frontier
  - replay breadth for that suffix
- snapshot rebuild cost is proportional to:
  - authoritative history range replayed into the snapshot family
  - immutable image bytes re-emitted

Minimum counters:

- `snapshot_capture_count`
- `snapshot_capture_record_count`
- `snapshot_capture_byte_count`
- `snapshot_read_count`
- `snapshot_read_record_count`
- `snapshot_restore_count`
- `snapshot_restore_tail_commit_count`
- `snapshot_restore_tail_replay_count`
- `snapshot_rebuild_count`
- `snapshot_rebuild_record_count`
- `snapshot_integrity_failure_count`
- `snapshot_basis_mismatch_count`

Milestone 4 may add richer counters, but it may not hide the actual restore
amplification basis.

## Phases

### Phase 1: Lock Snapshot Basis, Identity, And Non-Authority Boundaries

Phase 1 defines what a snapshot is allowed to mean before any image bytes are
persisted.

Required work:

- define snapshot basis fields and identity basis
- define snapshot non-authority rule and rebuild rule
- define snapshot publication atomicity and restore admissibility
- define proof-bearing snapshot pipeline types
- define point-in-time read classes and restore classes

Exit condition:

- a snapshot has one exact basis vocabulary
- a snapshot cannot be confused with authority
- restore meaning is no longer ambiguous

### Phase 2: Persist Immutable Snapshot Artifact Families

Phase 2 makes the immutable snapshot family real as a backend-supported derived
artifact surface.

Required work:

- implement snapshot image persistence
- implement basis and integrity record persistence
- implement restore-admitted publication boundary for snapshots
- expose typed capture and integrity failures
- emit exact snapshot capture counters

Exit condition:

- snapshots can be durably published as complete immutable derived artifacts
- incomplete or damaged snapshots are not restore-admitted
- snapshot identity and basis are machine-checkable

### Phase 3: Expose Point-In-Time Snapshot Reads

Phase 3 turns immutable snapshots into explicit read surfaces instead of latent
bytes on disk.

Required work:

- implement pure snapshot reads against explicit bases
- implement snapshot-tail read planning against explicit target frontiers
- expose typed basis mismatch and unsupported-read failures
- emit exact snapshot read counters and fallback counters where admitted

Exit condition:

- point-in-time reads talk in declared basis/frontier vocabulary
- snapshot reads remain stable and branch-explicit
- snapshot reads do not silently broaden into live branch-head reads

### Phase 4: Implement Snapshot-Plus-Tail Restore And Rebuild

Phase 4 makes snapshots part of a real restore and rebuild program.

Required work:

- implement snapshot-plus-tail restore planning
- implement suffix replay through canonical runtime truth semantics
- implement snapshot rebuild from authoritative artifacts
- implement parity comparison between restore and control replay lanes
- emit restore and rebuild counters

Exit condition:

- snapshot-plus-tail restore reaches the same target frontier as canonical
  replay
- deleted snapshots can be rebuilt honestly from authority
- restore and rebuild remain distinct but comparable lanes

### Phase 5: Prove Restore Equivalence And Snapshot Non-Authority

Phase 5 turns snapshots into a certifiable acceleration surface rather than an
optimistic convenience.

Required work:

- run the Milestone 4 named suite:
  `Snapshot-Plus-Tail Restore Equivalence Test`
- compare snapshot restore against canonical replay
- compare rebuilt snapshots against originally captured snapshot-visible truth
- emit machine-checkable truth, restore, artifact, and counter bundles

Exit condition:

- snapshot restore matches canonical replay
- rebuilt snapshots match original snapshot-visible truth
- snapshot deletion does not change recoverable truth
- Milestone 4 closeout evidence exists in machine-checkable form

## Must Ship

- immutable full snapshot artifact family with explicit basis and integrity
  records
- point-in-time snapshot reads against explicit immutable bases
- snapshot-plus-tail restore through canonical suffix replay
- snapshot rebuild from canonical authoritative artifacts
- typed snapshot capture, read, restore, rebuild, and integrity failures
- snapshot counters and machine-checkable Milestone 4 certification output

## Must Preserve

- canonical commit history remains the only semantic durability authority
- snapshots remain derived durable artifacts
- point-in-time reads remain branch-and-frontier explicit
- restore conclusions remain subordinate to canonical replay semantics
- deleting snapshots never deletes recoverable truth
- backend variation does not change snapshot truth-visible meaning

## Acceptance Evidence

Milestone 4 is complete only when the store satisfies the named Milestone 4
suite:

- `Snapshot-Plus-Tail Restore Equivalence Test`

Required machine-checkable outputs:

- `truth_digest`
- `restore_digest`
- `artifact_digest`
- `counter_snapshot`

Milestone-specific proof obligations:

- snapshot-plus-tail restore matches canonical replay for the same target
  frontier
- rebuilt snapshots match original snapshot-visible truth
- deleting snapshots does not change authoritative replay conclusions
- snapshots remain explicitly non-authoritative under parity comparison

Milestone 4 is not closed by "restored from snapshot successfully" tests.

## Architectural Notes

- The smart abstraction is not "snapshot file format." The smart abstraction is
  one exact snapshot basis-and-restore contract with immutable derived artifact
  publication around it.
- Snapshot image layout may vary by backend, but snapshot basis, identity,
  admissibility, and restore parity may not.
- Restore planning and restore execution should stay separate subdomains even
  if one backend initially implements both.
- Multi-resolution materialization later must build on these snapshot basis
  rules rather than smuggling in a second snapshot ontology.
- Retention later may prune snapshots aggressively, but only because rebuild
  from authority is already explicit and proven here.

## Sequencing Notes

This milestone belongs immediately after Milestone 3 because it depends on an
already-honest crash boundary and rebuild control lane, but it must land before
retention and later materialization families can claim to be honest.

- `Milestone 5` may run in parallel because branch delta layering is another
  physical storage program that still depends on the already-frozen authority
  model, not on snapshot semantics.
- `Milestone 10` retention and compaction depends on snapshot identity and
  rebuild semantics already being explicit.
- `Milestone 12` replication and capsules depend on snapshots already being
  clearly derived and branch/frontier explicit.
- later analysis lanes and multi-resolution materialization must inherit this
  basis contract rather than renegotiating what a snapshot means.
