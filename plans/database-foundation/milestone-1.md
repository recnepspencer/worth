# Milestone 1: Canonical Durable Commits

Governing roadmap: [Database Foundation Roadmap](roadmap.md#milestone-1-canonical-durable-commits).

Store durably publishes and recovers Relational's canonical commits as one
ordered journal per home, with one durable decision per commit, fates a fresh
process can resolve, and bounded point and range access. Relational is not yet
cut over (Milestone 2); this milestone proves the boundary with real Relational
envelopes produced by the memory-only runtime.

## Decisions

**D1. Lifecycle ownership.** Store owns the durable commit decision and recovery
sequencing for an application home: when a commit is durable, the order in
which owners readmit their artifacts, and when service may begin. The Query
host is the application's only entry point and the composition root that opens
the Store-owned home; it never sequences durability itself. Semantic owners
receive their recovered artifacts from Store and readmit them. Milestones 2 and
3 build on this; neither may introduce a second recovery sequencer.

**D2. The canonical artifact.** Relational exports
`RelationalCanonicalCommitArtifact`: a sealed, self-describing byte form of one
positioned canonical commit. Relational owns its frame, version, digest and
readmission. Store treats the bytes as an opaque owner payload and never decodes
them.

**D3. The commit point.** A commit is durable when the Store root generation
that contains its journal record and the advanced journal anchor is selected
durably. Store acknowledges only after that point. Milestone 2 moves
Relational's live cutover after this point; this milestone must not depend on
today's install-then-append ordering.

**D4. Attempt identity is the commit position.** Relational assigns contiguous
positions. A journal publication is admitted only at `head + 1`. After recovery,
`resolve(position, artifact digest)` answers from durable state alone:

- `Committed` when the journal holds that position with that digest;
- `NotCommitted` when the recovered head is below the position;
- `Conflict` when the position holds a different digest.

Before recovery completes, the existing in-process fates (`Completed`,
`ProvenNoEffect`, `Indeterminate`, in-progress) apply. Journal records are
authoritative and are not subject to idempotency-binding expiry, so resolution
never expires while the commit is retained.

**D5. Heads are root-referenced anchors.** The durable root manifest holds a
small record anchor table mapping a fixed anchor name to (record identity,
generation). A record batch may carry one anchor advance; it publishes in the
batch's root generation and is checked against the expected generation before
any WAL effect. The canonical journal is the first anchor
(`relational.canonical-commits`). World and Query reuse the mechanism in
Milestone 3.

**D6. Journal access uses jump pointers, not a derived index.** Each journal
record stores the identities of the most recent earlier records at every
power-of-two alignment below it (its spine). Point lookup from the head takes
O(log n) bounded header reads; descending range iteration is one read per
record. The spine is authoritative and immutable, so there is no index to
rebuild or drift.

**D7. Interruption controls.** Store's workspace is exempt from the one-build
rule and already gates certification fault authority behind its existing
`certification-test-authority` feature. The oracle uses that authority; it adds
no new Cargo feature and no production behavior.

## Durable artifacts

| Artifact | Classification | Owner |
|---|---|---|
| `RelationalCanonicalCommitArtifact` bytes | Authoritative | Relational (format, readmission) |
| Journal record (Store frame: journal name, position, payload digest and length, spine, payload) | Authoritative | Store |
| Record anchor table inside the root manifest | Authoritative | Store |

## Relational: canonical commit artifact

- Frame: magic, artifact version, `PatchStreamPosition`, `CommitId`, SHA-256
  of the payload and its length, then the payload `{position, canonical}`: the
  position under the digest and every authoritative envelope field in the
  existing named-map codec. Diagnostics and derived index lanes carry no
  entries.
- `readmit(bytes, ExecutionRequest)` checks the frame, decodes, requires the
  header to agree with the payload, runs the existing envelope and positioned
  commit admission, and requires the payload to be exactly the admitted
  commit's canonical encoding, so one commit has one digest. Every refusal is
  typed; nothing falls back.
- `runtime.durability().canonical_commit_artifacts(policy)` yields the
  retained commits in position order.
- The segmented local-filesystem path is untouched; Milestone 2 deletes it.

## Store: anchors

- Root manifest schema 11 is the head-bound layout plus an inline table of at
  most four anchors (name, routed record, record digest, generation). A root
  without anchors keeps its existing layout. An anchor must name a record its
  root routes.
- Every successor-root builder carries the table forward; maintenance and
  residue publications must leave it unchanged.
- A record batch may request one anchor advance. Planning checks it against the
  source root before the WAL append (`RecordAnchorConflict`), and the request is
  part of the idempotency fingerprint. A group merge refuses two advances.
- The WAL recovery projection records the advance (`RecordAnchorAdvanced`).
  Recovery replays it into the staged root, refuses a verified drop of an
  anchored record, and validates published roots against it.

## Store: canonical commit journal

`ServingPhysicalRuntime::canonical_commits()` returns `CanonicalCommitJournal`:

- `publish(CanonicalCommitPublication)` admits only `head + 1`, writes one
  journal record carrying its spine, advances the anchor in the same root
  generation and returns the existing typed mutation fates. Publications are
  serialized; an indeterminate one blocks further publication until reopen.
  An identical retry answers `AlreadyCommitted` and writes nothing.
- `head()`, `resolve(position, digest)` (D4), `read(position, limit)` with
  digest verification, and `range_descending(at_or_below, limit)`.

## Recovery adjudication

The oracle showed that a crash at several ordinary media operations left a
store that recovered but could not serve. Recovery now settles each case:

- Root-protocol candidates a killed runtime left in record staging are removed
  after recovery's own publication.
- Physical-work obligations of the dead runtime that recovery adjudicated (WAL
  intervals, record ranges, artifact, catalog and namespace synchronization,
  record removal) are discharged. Checkpoint and WAL reclamation obligations
  and undecodable records remain and still require inspection.
- A root-protocol replacement interrupted after its selectors but before its
  catalog is completed by republishing the selected root.
- A never-written WAL origin segment is an empty WAL, and an empty successor
  root is a torn, unpublished candidate that staging completes only from an
  exact prefix.

## Crash-consistency oracle

Location: `worth-store-recovery-runtime` `production_entry` tests.

- **Reference.** A memory-only Relational runtime executes a seeded sequence of
  creates, edits, relations and deletes; its artifacts are the expected
  journal. Relational's own tests cover schema transitions.
- **Media sweep.** For the first and a later publication, every identified media
  operation of every role is counted, then rerun once per ordinal with
  `FailBefore` and `IndeterminateAfterEffect`. Each case abandons the runtime,
  recovers through the production path and requires an exact reference prefix
  containing every acknowledged entry, Relational readmission of every entry,
  `resolve` agreement, publication of the suffix and an identical second
  recovery.
- **Real process lane.** A child is killed before and above a checkpoint and at
  each publication seam (before the WAL append through root replacement); the
  durable-WAL seams roll forward exactly once.
- **Recovery interruption.** Recovery itself is killed at each of nine stages;
  the next recovery settles to the same journal.

## Completion

Milestone 1 is complete when:

1. Relational exports the artifact and readmission door with refusal tests.
2. The anchor table and journal publish, recover and serve point/range access
   through the production entry and recovery facade.
3. The media sweep, real-process lane and recovery interruption pass with real
   Relational envelopes, covering the first-checkpoint boundary, publication
   before acknowledgment and interrupted recovery.
4. `resolve` answers every reference position correctly after every
   interruption, and a retry never publishes a second record for a committed
   position.
5. Store and Relational owner lanes, the Store process-scenario lane,
   boundary-check, agent-context, formatting and line caps pass, with no new
   failures against the base commit.
