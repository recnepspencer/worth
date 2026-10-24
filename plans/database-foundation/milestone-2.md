# Milestone 2: Relational Cutover to Store

Governing roadmap: [Database Foundation Roadmap](roadmap.md#milestone-2-relational-cutover-to-store).
Builds on: [Milestone 1](milestone-1.md).

Relational's only durable path is Store. A commit becomes visible in live
state only after Store has made it durable. A fresh process reconstructs the
runtime from Store's checkpoint and journal through Relational's own
readmission. The segmented local-filesystem engine is deleted.

## Decisions

**D1. The seam is a Relational port that Store implements.** Relational
defines `RelationalDurableMedium`, a public object-safe trait with one
operation: publish one `RelationalCanonicalCommitArtifact` at its position and
report `Committed`, `ProvenNoEffect` or `Indeterminate`. Relational never
depends on Store. A new Store-workspace crate, `worth-store-relational`,
depends on Relational and Store, implements the port over the Milestone 1
journal, and owns opening, recovery sequencing and checkpoint publication for a
Relational home (Milestone 1 D1). This follows the Store vision's structural
rule (the runtime produces artifacts; Store persists them and owns the durable
lifecycle) and keeps Signal and Store's certification crates out of
Relational's dependency graph.

**D2. Durable before live.** With a medium attached, publication runs in this
order: preparation and validation against the exact admitted basis; the
next patch position is reserved; the positioned commit is encoded as its
canonical artifact and published to the medium; only a `Committed` answer lets
the live cutover install the new head; settlement follows. Live state never
selects a commit the medium has not made durable, and acknowledgment follows
settlement as before.

- `ProvenNoEffect` is a pre-effect denial: the position is not consumed, the
  cutover does not run and the transaction returns
  `TransactionCommitError::PublicationDenied` with
  `RelationalPublicationDenial::DurablePublicationRefused`.
- `Indeterminate` returns `TransactionCommitError::DurabilityUnresolved` with
  the position, commit id and artifact digest. The cutover does not run. The
  runtime refuses every later publication with the same outcome until it is
  reopened; after reopen, recovery and the journal's `resolve` establish the
  fate.
- A commit that is durable but whose settlement does not finish returns
  `TransactionCommitError::PerformedButSettlementDeferred` (renamed from
  `PerformedButDurabilityDeferred`; settlement no longer appends to any
  medium). Its repair finishes settlement under the original identity; a crash
  instead recovers it from the journal. Neither path re-executes a mutation
  handler or issues a replacement commit.
- Memory-only runtimes keep their in-memory canonical log and do not change.

**D3. Recovery is Store-sequenced and Relational-readmitted.** The home reads
the selected checkpoint (if any) and the journal positions after its coverage,
and passes the bytes to Relational as a `RelationalDurableRecoverySource`.
`runtime.durability().recovery_plan_from_source(..)` readmits every artifact
through `RelationalCanonicalCommitArtifact::readmit`, requires the tail to be
contiguous from the checkpoint's covered position, requires the checkpoint's
own coverage to equal the position Store recorded for it, and builds the
existing `RecoveryPlan`. The existing `recover` rebuilds the runtime with its
schema, profile, runtime-name and codec continuity checks. The medium is
attached only after the rebuild, so replay never republishes. Any refusal is
typed and happens before service.

**D4. Checkpoints are anchored Store records.** `home.checkpoint()` captures
Relational's native checkpoint and publishes it as one record under the anchor
`relational.checkpoint`, framed with its covered position, in one root
decision. It then completes a Store physical checkpoint, so Store's own
recovery replays only the WAL after it. Recovery is bounded by both
checkpoints and the journal after the Relational one. The journal is retained
in this milestone; covered-journal reclamation belongs to Milestone 4.

**D5. Home identity is an anchored descriptor.** Initialization publishes
`relational.home` (format version, runtime name, profile) once. Opening a home
whose descriptor disagrees with the requested runtime refuses before recovery.
A directory with no Store is initialized; anything else is recovered or
refused. An unreadable or damaged descriptor, checkpoint or journal is never
treated as an empty home.

**D6. Store adds a generic anchored-record door.** The Milestone 1 anchor
mechanism gains `ServingPhysicalRuntime::anchored_records()`: read the record
an anchor names, and publish a replacement at `generation + 1` with the same
durable-root and fate semantics as the journal. The journal keeps its own API.
A home uses three of the four anchor slots.

**D7. Identity continuity.** Record identity facts (generation high-water
marks, reusable slots, append frontiers, outstanding reservations) are restored
from the checkpoint and then advanced by each tail commit's recorded
allocations. Commit, version and lineage frontiers are restored as one past the
largest durable value. An identity that was neither durable nor acknowledged
when the process died was never authority, so it is not a reservation whose loss
permits unsafe reuse. A committed or deleted identity is never reissued; the
oracle checks this by re-executing the suffix on the recovered runtime.

**D8. Branch lifecycle scope.** A fork is a branch-reference transition, not a
canonical commit. In this milestone a branch becomes durable with its first
commit (the envelope carries the branch cell's prior state) or when a
checkpoint covers it. Durable fork, archive and delete as their own operations
are Milestone 4. The oracle forks and then commits on the new branch.

**D9. Residency.** The native working state stays fully resident. Store holds
the only authoritative copy; memory holds working state derived from it, so
there is no second authoritative database. A commit writes only its own
artifact, never the record set. Reopen materializes the checkpoint image and
replays only the tail after it, not all history. Partition-granular cold access
behind fallible guards requires replacing about 230 infallible partition reads
and is Milestone 4's cold-data work. In this milestone every read of committed
data from Store (descriptor, checkpoint, journal) is fallible and typed and
refuses service rather than skipping data.

**D10. Interruption controls.** Store fault schedules use the existing
certification authority. The home crate forwards it as its own
`certification-test-authority` feature (inside Store's one-build exemption),
which adds two hidden doors: open a home on a Store the test opened with a
fault schedule, and reach the home's Store to park a publication at a mutation
seam. Both run the production admission path. Parking at
`AfterRootReplacement` stops a commit after Store made it durable and before
Relational installs it live. Relational's own durability tests use a
`cfg(test)` in-crate medium; it is not a second durable path.

## Durable artifacts

| Artifact | Classification | Owner |
|---|---|---|
| Canonical commit artifact (journal payload) | Authoritative | Relational format; Store journal ordering |
| Native checkpoint (anchored record payload) | Derived (reconstructible from the journal) | Relational format; Store framing and anchor |
| Home descriptor | Authoritative | Store home |

## Relational changes

- `RelationalDurableMedium` and its publication answer types, exported through
  the facade.
- The runtime carries an optional medium. The builder attaches one with
  `durable_medium(..)`; recovery transfers it to the rebuilt runtime.
- Durable publication inside the position reservation, before cutover (D2).
- `RelationalDurableRecoverySource` and `recovery_plan_from_source` (D3).
- `RelationalNativeCheckpoint::covered_position()`.
- Deleted: `DurabilityMode::PersistedSegmentedLocalFs`, `DurableStoreLayout`,
  `DurableStore`, segment and checkpoint manifests, compaction types, the
  local store, segment codec, file locking, persisted recovery planning and
  filesystem checkpoint persistence, with their builder setters and config
  fields, and `DurabilityMode` itself: whether a medium is attached is the
  only mode, so a separate mode would be a second source of truth.
- Durability contract tests that asserted recovered semantics run against the
  in-crate medium through the public recovery door. Tests that only exercised
  segment files, manifests or locks are deleted.
- README, owner-component and durability guidance describe commitment,
  recovery and the typed outcomes.

## Store changes

- `anchored_records()` (D6).
- `worth-store-relational`:
  - `RelationalStoreHome::open(request)` initializes a new home or recovers an
    existing one through the production recovery entry, readmits the
    descriptor, checkpoint and tail, and returns a live runtime with the
    journal medium attached and fresh owner capabilities.
  - `checkpoint()`, `resolve(position, digest)`, `close()`.
  - The medium adapter maps journal outcomes to the port: `Committed` and an
    identical `AlreadyCommitted` are durable; proven-no-effect failures before
    the append are `ProvenNoEffect`; everything else is `Indeterminate`.

## Crash-consistency oracle

Location: `worth-store-relational` tests.

- **Reference.** A memory-only runtime executes a seeded sequence of creates,
  aspect edits, relations, deletes, a schema transition, a fork followed by
  commits on the fork, and checkpoints. Its artifacts and state are the
  expected result.
- **State comparison.** Canonical artifacts embed the producing runtime's
  process-local instance identity (branch cell checkpoints and fork
  provenance), so a durable run's artifacts are not byte-equal to the
  reference's. After recovery the home must hold exactly the artifacts the
  crashed run acknowledged, byte for byte, plus at most the one in-flight
  commit, present exactly when `resolve` says it is committed; and every
  record and relation on every durable branch, read through Relational's
  ordinary transaction reads, must equal the reference's after that many
  commits. The recovered runtime then runs the rest of the script and must
  reach the reference's final records; new creates must never reuse an
  identity that was ever committed, including deleted ones.
- **Media sweep.** Store fault schedules interrupt every media operation of a
  commit and of a checkpoint with `FailBefore` and `IndeterminateAfterEffect`.
- **Real processes.** A child is killed before commitment (before the WAL
  append), at each later Store mutation seam including after root replacement
  (durable, not yet live), during a checkpoint, and on both sides of the first
  checkpoint.
- **Recovery interruption.** Recovery is interrupted and repeated; a second and
  third recovery reach the same state.
- **Refusals.** A stale candidate, a conflicting write and an invalid
  transaction publish nothing. A home with a foreign descriptor, a damaged
  checkpoint or a damaged journal entry refuses before service.

## Completion

Milestone 2 is complete when:

1. No `PersistedSegmentedLocalFs` path, type, setter or test remains, and the
   medium is Relational's only durable path.
2. Commits are durable before live installation, with the D2 outcomes covered
   by tests.
3. A home survives process death on both sides of the first checkpoint, before
   commitment and after durable publication before live installation, with
   exact records, identities, schema and canonical history, and supports
   another edit and safe allocation afterwards.
4. The oracle's generated sequences pass against the memory-only reference.
5. Relational owner lanes, Store lanes, boundary-check, agent-context,
   formatting and line caps pass with no new failures against the base commit.
6. An independent reviewer has certified the work against this specification
   and the roadmap.
