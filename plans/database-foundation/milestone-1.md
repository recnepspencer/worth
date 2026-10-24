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

**D5. Heads are root-referenced anchors.** A record anchor table, referenced from
the durable root manifest, maps a fixed anchor name to (record identity,
expected generation). A record batch may carry anchor advances; they publish in
the batch's root generation and are checked against the expected generation
under the anchor's lock. The canonical journal is the first anchor
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
| Journal record (Store frame: owner tag, position, artifact digest, spine, payload) | Authoritative | Store |
| Record anchor table and its root-manifest reference | Authoritative | Store |

## Relational: canonical commit artifact

- Frame: magic, artifact version, `PatchStreamPosition`, `CommitId`, SHA-256
  of the payload, then the authoritative envelope payload
  (`encode_authoritative_payload`, excluding diagnostics and derived index
  lanes).
- `RelationalCanonicalCommitArtifact::from_positioned(&PositionedCanonicalCommit)`
  encodes; `readmit(bytes, ExecutionRequest)` verifies frame, version and
  digest, decodes, runs the existing envelope readmission chain and returns a
  readmitted positioned commit. Unknown versions, digest mismatch and
  header/envelope disagreement refuse; nothing falls back.
- `RelationalRuntime` exposes the retained positioned commits in position order
  as artifacts, so callers obtain real envelopes without reaching into history
  internals.
- The segmented local-filesystem path is untouched here; Milestone 2 deletes it.

## Store: anchors

- `worth-store-physical-format`: an anchor table block (versioned, checksummed,
  bounded by the declared anchor names) and a new root-manifest layout that
  references it. Stores without the layout are refused; there is no
  compatibility decode for earlier development stores.
- Every successor-root construction path (ordinary publication, group
  publication, checkpoint, recovery staging and publication, maintenance and
  retirement) carries the anchor table reference forward unchanged unless the
  publishing batch advances an anchor.
- Anchor advances join `RecordAppendBatch` submission. Preparation takes anchor
  locks in name order, checks expected generations, and holds them until root
  publication or a pre-effect refusal. Two advances on one anchor never share a
  publication group.
- A publication failure after WAL append leaves the anchor unsettled: further
  advances and reads of that anchor refuse until recovery.
- Recovery redo carries anchor advances; a recovered root exposes the anchor
  table it selected.

## Store: canonical commit journal

`ServingPhysicalRuntime::canonical_commits()` returns `CanonicalCommitJournal`:

- `publish(CanonicalCommitPublication) -> fate`: owner payload bytes (streamed),
  position, artifact digest, durability request. Builds one journal record with
  the spine from the current head, submits it with the anchor advance as one
  record batch, and returns the existing typed mutation fates.
- `head()`: the selected position and record identity, or empty.
- `resolve(position, digest)`: D4.
- `read(position, limits)`: point access returning a bounded payload session.
- `range(from, to, limits)`: descending bounded iteration, resumable by
  position.
- Allocation, cancellation, deadline and barrier failures keep their existing
  effect posture; a position gap, stale head or unsettled anchor refuses before
  effects.

## Recovery and checkpoints

- Recovery before and after the first checkpoint selects the journal anchor from
  the recovered root; the journal is readable without scanning history.
- A second crash during recovery reaches the same journal head and the same
  `resolve` answers.
- WAL reclamation and checkpoint coverage keep every journal record reachable
  from the anchor; retention of journal records is unconditional in this
  milestone.

## Crash-consistency oracle

Location: `worth-store-certification` tests (already a dev-dependent of
`worth-relational`).

- **Reference.** A memory-only Relational runtime executes a seeded, generated
  sequence of transactions (record creates and edits, relations, aspect writes,
  a schema transition) and yields its positioned commits as artifacts.
- **Media sweep.** For each media operation ordinal reached while publishing
  the sequence (and while checkpointing at seeded points), run a fresh home with
  `FailBefore`, `IndeterminateAfterEffect` and `PanicAfter` at that ordinal,
  drop the runtime, recover, and check:
  - the journal is exactly a prefix of the reference containing every
    acknowledged commit, byte-equal and readmitted by Relational;
  - `resolve` agrees with the prefix for every reference position;
  - publishing the remaining suffix succeeds and the final journal equals the
    reference.
- **Recovery interruption.** Repeat recovery with a recovery yieldpoint crash and
  a second recovery; conclusions must match the first.
- **Real process lane.** A child process publishes a seeded sequence and is
  killed at seeded points; a fresh process recovers and applies the same checks.
- Cheap seeds run in the owner lane; the full sweep runs in the scheduled
  process-scenario lane through `store-test-runner`.

## Completion

Milestone 1 is complete when:

1. Relational exports the artifact and readmission door with refusal tests.
2. The anchor table and journal publish, recover and serve point/range access
   through the production entry and recovery facade.
3. The oracle's media sweep, recovery interruption and real-process lanes pass
   with real Relational envelopes, covering the first-checkpoint boundary,
   publication before acknowledgment and interrupted recovery.
4. `resolve` answers every reference position correctly after every
   interruption, and a retry never publishes a second record for a committed
   position.
5. Store and Relational owner lanes, the Store UI and process-scenario lanes,
   boundary-check, agent-context, formatting and line caps pass.
