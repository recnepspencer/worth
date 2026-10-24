# Milestone 2: Relational Cutover to Store

Governing roadmap: [Database Foundation Roadmap](roadmap.md#milestone-2-relational-cutover-to-store).
Builds on: [Milestone 1](milestone-1.md).

Relational's only durable path is Store. A commit, and every branch lifecycle
transition, becomes visible in live state only after Store has made it
durable. A fresh process reconstructs the runtime from Store through
Relational's own readmission without materializing all records or all history:
committed data is read from Store on demand behind fallible residency guards.
The segmented local-filesystem engine is deleted.

## Adversarial constraint

A home is killed at any durable I/O boundary of any operation, including
commits, branch transitions, checkpoints and recovery itself, and is reopened
any number of times. Each reopen must:

- recover exactly the acknowledged prefix plus at most the one in-flight
  operation, decided from durable state;
- reach records, identities, schema boundaries and branches equal to a
  memory-only reference at that prefix, with canonical artifacts byte-identical
  to the reference's;
- never reissue a committed identity;
- cost work proportional to the checkpoint manifest and the journal after it,
  not to the number of records or the length of history. A home holding N
  partitions and H commits that reopens and reads one partition loads one
  partition and no historical envelope beyond the tail.

A storage read that fails is a typed refusal, never absence. An unresolved fate
blocks only what depends on it and is decided by recovery alone.

## Decisions

**D1. The seam is a Relational port that Store implements, owned by one
runtime.** Relational defines the medium port: publish a canonical commit
artifact, publish a branch transition artifact, and read committed data back
(D9). Answers are `Committed`, `ProvenNoEffect` and `Indeterminate`.
Relational never depends on Store. The Store-workspace crate
`worth-store-relational` implements the port and owns opening, recovery
sequencing and checkpoint publication for a home (Milestone 1 D1).

A medium is moved into exactly one runtime when that runtime is built or
recovered. It is not cloneable through the builder, and a forked runtime never
inherits it, so two runtimes cannot publish to one medium. The builder never
holds it: `build_durable(Box<dyn RelationalDurableMedium>)` moves it into the
runtime it builds. Recovery moves it out of the recovering runtime into the
rebuilt one; the attachment it left refuses every later publication with the
pre-effect `MediumMoved`.

**D2. Durable before live.** With a medium attached, publication runs in this
order: preparation and validation against the exact admitted basis;
reservation of the next patch position; publication of the positioned commit's
canonical artifact; only on `Committed`, the live cutover; then settlement.
Live state never selects a commit the medium has not made durable.

- `ProvenNoEffect` is a pre-effect denial
  (`PublicationDenied(DurablePublicationRefused)`). The position is not
  consumed and nothing is installed.
- `Indeterminate` returns `DurabilityUnresolved` with the position, commit id
  and artifact digest. Nothing is installed, and later publications refuse with
  the same answer until the home is reopened. Then recovery decides the fate,
  and `resolve` answers it by Relational's digest.
- A commit waiting for the patch-position reservation while another commit's
  durable publication is in flight waits within its execution deadline. It is
  not deferred because a concurrent publisher holds the reservation for I/O.
  Waiters are served first come, first served and observe the request's
  cancellation and deadline; a stopped waiter returns `RequestStopped` before
  any effect, holding no position. The wait cannot deadlock: a publisher takes
  its own branch's critical section before the reservation, and the holder
  takes no other branch's while holding it. Memory-only runtimes keep their
  existing reservation behavior.
- The settlement-time append and its "performed but durability deferred"
  repair path are deleted: durability can no longer fail after the cutover.
  Settlement that fails for another reason keeps the durable commit under its
  original identity; a crash recovers it from the journal. Neither path
  re-executes a mutation handler or issues a replacement commit.
- Memory-only runtimes keep their in-memory canonical log.

**D3. Recovery is Store-sequenced and Relational-readmitted.** The home reads
the checkpoint manifest, the journal tail after its coverage, and the branch
transitions after its coverage, then hands them to Relational as a
`RelationalDurableRecoverySource`. Relational readmits every artifact, requires
the tail to be contiguous from the checkpoint's coverage, and rebuilds through
its existing continuity checks. The medium is bound only after the rebuild, so
replay never republishes. Any refusal is typed and happens before service.

**D4. Checkpoints are a manifest plus content-addressed regions.** A checkpoint
is a small manifest record under `relational.checkpoint` that names its covered
position, branch cells, schema images, identity allocation state, lineage,
index definitions, symbols and a region directory. Each partition region and
mirror partition is its own Store record, addressed by its content digest. A
checkpoint writes only the regions whose digest changed since the previous
checkpoint and reuses the rest. Publishing the manifest is one root decision.
The home then completes a Store physical checkpoint, so Store's own recovery is
bounded too. A crash at any point of a checkpoint, first or later, leaves the
previous or the new checkpoint selected and a home that serves.

**D5. Home identity binds runtime and schema.** Initialization publishes
`relational.home` with the format version, runtime name and profile. The
schema a home serves is the one its recovered main branch head records: the
initial schema, or the target of the last durable schema transition. Opening
refuses before service when the requested runtime name or profile differs
from the descriptor, or when the declared schema registry differs from the
recovered main head's schema, including a different version label with
identical contracts. Only a durable schema transition changes a home's schema.
A directory with no Store is initialized; anything else is recovered or
refused. Damaged or unreadable home data is never treated as an empty home.

**D6. Store adds a generic anchored-record door** (`anchored_records()`), with
the journal's fate semantics. A home uses three anchors: descriptor,
checkpoint manifest and branch transitions. The canonical journal is the
fourth.

**D7. Identity continuity.** Record identity facts (generation high-water
marks, reusable slots, append frontiers, outstanding reservations) are restored
from the checkpoint and advanced by each tail commit's recorded allocations.
Commit, version and lineage frontiers resume one past the largest durable
value. An identity that was never durable or acknowledged was never authority,
so its loss on process death is not unsafe reuse. A committed or deleted
identity is never reissued; the oracle checks this after every crash.

**D8. Branch lifecycle transitions are durable before live.** Fork, archive,
deletion-requested and delete each produce a runtime-neutral
`RelationalBranchTransitionArtifact`. It is framed like the canonical commit
artifact and carries:

- its own contiguous transition sequence;
- the patch position it follows;
- for a fork, the source branch's generation, truth version and selected head;
- for archive and delete, the branch's prior generation and truth version.

The design keeps transitions apart from commits:

- **Own sequence.** Transitions do not consume patch positions, so the commit
  stream and its tail rules stay contiguous.
- **Own journal.** They are stored in a second ordered journal,
  `relational.branch-transitions`, the fourth anchor. Store's journal is
  generalized to a named journal with per-name unresolved state.
- **Durable step.** A transition publishes inside the patch-position
  reservation with D2 waiting, after every fallible preparation step and before
  any live effect. Its fates are those of D2.
- **One blocking slot.** An unresolved transition and an unresolved commit
  share one slot that blocks both kinds until reopen. This is required: a later
  commit must not overtake an unresolved delete.
- **Typed stop.** Fork, archive and delete return
  `RelationalBranchTransitionStop<Denial>`, either `Denied` (including
  `DurablePublicationRefused`) or `DurabilityUnresolved`.
- **Resolution.** The home resolves a transition's fate by sequence and digest.

Checkpoints record the transitions they cover, every branch's lifecycle
posture and the retired branch names. Recovery restores all three and no
longer clears retired names, so an archive, a deletion or a retired name
survives a crash.

Recovery merges the transition tail into the commit tail: before replaying
commit q it applies every transition that follows a position below q. Each
replayed transition is validated against the live registry. A fork is the only
source of a branch's existence, so a commit carrying the cell of an unknown
branch is refused. Milestone 4 adds fork-by-sharing at scale, retained readers
of deleted branches and retention.

**D9. Committed data is read on demand behind residency guards.** Reopen
loads the checkpoint manifest and replays the tail. Partition regions, mirror
partitions and historical envelopes are read from Store only when an operation
needs them, through a Relational-defined read port the home implements.

An operation acquires residency for what it will read before it reads it: a
transaction or observation at basis admission, a mirror read at edition
acquisition. A residency failure is a typed refusal of that operation
(transaction outcome, read denial or recovery stop), never absence or empty
data. Resident working state is derived from Store, the only authoritative
copy, so there is no second authoritative database.

Partition identity and placement never change with residency. Cold and warm
reads must agree. Milestone 4 extends the same guards to bounded chunks,
eviction, larger-than-memory serving and historical selection. Detailed types
are in "Residency" below.

**D10. Interruption controls.** Store fault schedules use the existing
certification authority, forwarded by the home crate's
`certification-test-authority` feature inside Store's one-build exemption. Its
hidden doors run the production admission path. Relational's own durability
tests use a `cfg(test)` in-crate medium; that is not a second durable path.

**D11. Canonical artifacts are runtime-neutral.** A canonical artifact's bytes
depend only on durable history, never on which process produced it. Every
runtime identity an envelope records is recorded in one canonical runtime
frame: a reserved runtime instance id that no live runtime is issued, with the
same text width as live ids. That covers the branch cell state
(`branch_cell_checkpoint`, including fork provenance), a required output's
producer basis and a codec retirement's source basis. A recorded basis
descriptor also omits its process-local root identity, which is a per-process
allocation rather than durable history. Envelope checks that compare recorded
identities with each other compare within the frame; a check against live
state first moves the live side into the frame. Recovery rebinds recorded cells
to the recovering runtime, as it already does for checkpointed cells.
Checkpoint images stay runtime-affine: they are derived, not canonical.
Re-executing the same operations in any process, recovered or not, produces
byte-identical artifacts.

## Durable artifacts

| Artifact | Classification | Owner |
|---|---|---|
| Canonical commit artifact (journal payload) | Authoritative | Relational format; Store journal ordering |
| Branch transition artifact | Authoritative | Relational format; Store anchored log |
| Checkpoint manifest | Derived (reconstructible from journal and transitions) | Relational format; Store anchor |
| Partition region record | Derived | Relational format; Store content-addressed placement |
| Home descriptor | Authoritative | Store home |

## Residency

**Checkpoint layout.** `relational.checkpoint` holds a manifest. Its member
records are published with it in one root decision and are carried forward,
not rewritten, while their content is unchanged.

- **Manifest (eager).** The manifest holds:
  - coverage and runtime name;
  - branch cells, schema images and record-identity facts;
  - aspect contracts and index definitions;
  - for each root, its commit id, schema carrier digest and region leaves
    `{partition_id, content_digest, slot counts, unavailable count, member}`;
  - mirror leaves;
  - the commit index: one small entry per commit with its id, version,
    position, parents, branch, root descriptor, payload digest and schema
    version.
- **Members (lazy, content-addressed by SHA-256).** A member is one of:
  - a region head image and a region history image (`RelationalRegionImage`:
    partition, schema carrier digest, kind, payload);
  - a symbol page;
  - a lineage page;
  - a derived-index image.

  Equal content is stored once across roots, the mirror and checkpoints.
- **Verification without loading.** Reopen recomputes each root's region
  commitment from its leaves and compares it with the root descriptor. Each
  member is verified against its leaf when it loads.
- **Embedded checkpoints** used by the Query host carry the same members
  inline. Placement is an artifact-policy parameter of one format.

**Read port.** `RelationalDurableColdSource` sits beside the medium:
`read_member(locator, limit)` and `read_commit(position, limit)`. The home
implements it over anchored-record members and the journal. The recovery
source carries it, because recovery needs it before the medium is bound.

**Residency cells** (`storage/residency`):

- `RelationalRegionResidency` is shared by every root and the mirror that hold
  the region, and is `Cold`, `Resident` or sticky `Damaged`. One load serves
  concurrent demands.
- A load reads, checks the frame and digest, decodes under that root's schema,
  readmits required text, rebuilds kind buckets and verifies the content
  digest.
- Residency never relocates an identity or changes a slot directory.

**Guards.** Loads happen only in operation orchestrators, never behind a
pure-looking accessor:

- **Declared demand** where the footprint is known: a point read demands its
  partition, a mirror write its touched set, and scans, maintenance and
  inspection the whole root or mirror.
- **Probe and retry** where the footprint depends on data, as in commit
  preparation, visibility reads and invariant reads:
  - A round runs over views whose cold entries record a miss.
  - A round with any miss is discarded, its misses are loaded (the fallible
    step) and the round runs again. Only a round without misses is accepted.
  - Retries are bounded by the distinct regions read, and a policy ceiling
    escalates to the whole root.
  - Discarding a round equals abandoning a candidate, which never reuses an
    identity (D7).

**Residency leases.** Every accepted demand returns a counted
`RelationalResidencyLease` for the regions it made resident. The operation
holds the lease for as long as it can read those regions: a transaction until
it commits or is dropped, an observation or read view for its lifetime, an
accepted probe round for the candidate it produced, a retained root while it
is retained. Reads go through views that are only obtainable from a lease, so
reading resident data without one is unrepresentable. Milestone 2 never evicts
anything, but every reader of a residency cell is already counted. Milestone 4
adds release and eviction without retrofitting a lease onto any reader.

**Head and history are separate.** A region's checkpoint content is two
member kinds:

- a head image: each slot's current record version, the slot directory,
  lifecycle and adjacency;
- a history image: earlier record versions.

A region cell is resident at `Head` or `Full`. Current-state reads,
validation and commit preparation demand `Head`. Version-filtered reads,
historical observations, aspect history and retention demand `Full`.

A head-resident region is exposed through a head-only view that has no
history accessors, so a historical read against a head-only region is
unrepresentable rather than silently answered from the latest version.
Writes append history in memory to a resident region of either level, and
capture writes the changed head and history members.

**Typed failures** (`RelationalStorageUnavailable{subject, cause}`, with
subject region, envelope, symbols, lineage or index, and cause source
unavailable, damaged, integrity mismatch, limit exceeded or memory exhausted):

- transaction reads: `RelationalTransactionReadDenial::StorageUnavailable`;
- commit and merge preparation:
  `CommitPreparationReason::StorageUnavailable`, before any effect;
- visibility and invariant reads:
  `RelationalExecutionDenialCause::StorageUnavailable`;
- storage statistics, retention passes and inspection: `Result` with the same
  failure;
- tail replay: `RecoveryFailureClass::ColdStorageUnavailable`;
- pinning requires a resident witness, so pinning a cold partition is
  unrepresentable.

**History on demand.**

- The commit catalog keeps index entries only.
- Envelopes live in `RelationalCommitEnvelopeResidency` cells. One is held
  resident only while it is a branch head, an unsettled route or in flight;
  otherwise it is read from the journal, readmitted and checked against the
  index digest.
- History queries (patch streams, aspect history, envelope surfaces,
  retained export, index builds) read through the cold source and become
  fallible and bounded.
- Memory-only runtimes keep envelopes resident: they have no other copy.

**Symbols and lineage.**

- Field text and lineage events are paged as members too:
  - symbol pages are keyed by id range for reads, with hash-bucket pages
    giving text-to-id lookup for interning;
  - lineage pages are keyed by node range.
- Reopen loads neither the full symbol table nor all lineage. A new text is
  interned after its bucket page is resident, so deduplication still holds.

**Capture.** A checkpoint walks leaves and never loads. It carries a region
that has a locator and is unchanged, encodes a resident region that changed,
and records new locators once publication completes. A one-partition commit
followed by a checkpoint writes O(1) new members.

**Placed by the roadmap in Milestone 4, not deferred from this one:**

- kind and adjacency summaries, so that a first scan need not warm a whole
  branch;
- moving pins and lifecycle out of the mirror so it stops being a separate
  working copy of main;
- eviction and larger-than-memory serving.

This milestone builds the cells, port, guards and member format that those
extend without reshaping.

## Relational changes

- The medium port (commit and branch-transition publication, committed-data
  reads), its answer types and the recovery source, exported through the
  facade. The runtime owns its medium (D1).
- Durable publication inside the position reservation, before the cutover,
  with waiting instead of deferral when a medium is attached (D2).
- Runtime-neutral branch cell state in envelopes (D11).
- Durable branch transitions (D8) and the schema identity check on recovery
  (D5).
- The split checkpoint and residency guards (D4, D9).
- Deleted: the segmented filesystem engine, `DurabilityMode`, its config and
  setters, and the settlement-time append with its deferred-durability
  carrier, repair API, test hooks and downstream branches in World and Query.
- Durability contract tests run against the in-crate medium through the public
  recovery door. Tests that only exercised retired storage or the retired
  deferral are deleted.
- README, owner-component and durability guidance describe commitment,
  recovery, residency and the typed outcomes.

## Store changes

- `anchored_records()` (D6). Recovery adjudicates an interrupted Store
  physical checkpoint so that a crash during one never leaves a Store that
  requires inspection forever.
- `worth-store-relational`: `RelationalStoreHome::open`, `checkpoint`,
  `resolve`, `close`, the medium adapter, the region and history read port, and
  the branch transition log.

## Crash-consistency oracle

Location: `worth-store-relational` tests.

- **Reference.** A memory-only runtime executes a seeded script of creates,
  edits, relations, deletes, a schema transition, forks, archives, deletes of
  branches, commits on every live branch, and checkpoints. Its artifacts and
  records are the expected result. Every seed in a tested range generates.
- **State comparison.** After recovery the home holds the reference's artifacts
  byte for byte (D11) up to the acknowledged prefix, plus at most the one
  in-flight operation, present exactly when `resolve` says it is committed.
  Every record and relation on every durable branch, read through ordinary
  transaction reads both cold (just reopened) and warm, equals the reference.
  The rest of the script then runs to the reference's final state, and new
  creates never reuse a committed identity.
- **Residency.** A reopened home with many partitions and a long history loads
  only the manifest and the partitions an operation touches (measured through
  the read port with headroom, not exact counts). A region or history read that
  fails refuses the operation with its typed outcome. Cold and warm reads agree.
- **Media sweep.** Every media operation of a commit, a branch transition, the
  first checkpoint and a later checkpoint, and a region load, with
  `FailBefore` and `IndeterminateAfterEffect`.
- **Real processes.** A child is killed at every Store mutation seam of a
  commit (before the WAL append through durable-not-yet-live), during a branch
  transition, during a checkpoint including Store's physical checkpoint, and
  idle on both sides of the first checkpoint.
- **Recovery interruption.** Recovery is killed at each Store recovery stage
  and during Relational readmission. Repeated recoveries reach the same state.
- **Refusals.** A stale candidate, a conflicting write and an invalid
  transaction publish nothing. A foreign runtime, a foreign schema, a damaged
  descriptor, manifest, region or journal entry each refuse before service.
  A medium cannot be shared by two runtimes.

## Completion

Milestone 2 is complete when:

1. The medium is Relational's only durable path. No filesystem engine, mode,
   deferred-durability path or test remains.
2. Commits and branch transitions are durable before they are live, with the D2
   outcomes covered by tests.
3. A home survives process death at every boundary in the oracle, on both sides
   of the first checkpoint, with records, identities, schema, branches and
   byte-identical canonical history equal to the reference. It then supports
   further edits and safe allocation.
4. Reopen and ordinary operations read committed data on demand. The residency
   tests show that untouched partitions and history are not loaded, and that
   read failures are typed refusals.
5. The oracle's generated sequences pass.
6. Relational, World, Query and Store owner lanes, boundary-check,
   agent-context, formatting and line caps pass with no new failures against
   the base commit.
7. An independent reviewer certifies the work against this specification and
   the roadmap.
