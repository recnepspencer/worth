# History, Compaction, And Rebuild

## The Frame

The easy path is `inspect()` for the current state. Reach for retained history
when a browser-local workflow needs explanation, a read-only past view, or a
known snapshot from which to rebuild derived Signal state.

This history is deliberately bounded and process-local. It is not an event
store, persistence format, backup, restart protocol, or regulated audit log.
Those responsibilities belong in the full platform.

## Two Different Lifecycles

```text
Local Truth commit and snapshot  --authoritative for this process--+
                                                                |
                                                                v
                                                  Signal derivation
                                                  (disposable/rebuildable)
```

A Local Truth commit can succeed while Signal delivery fails. That is an
honest partial outcome: application truth advanced, derived execution did not.
The runtime does not undo a true commit to make a projection look current.

`derivation(branchId)` reports one of:

- `Current`: projection matches the Local Truth head;
- `CommittedDerivationPending`: truth committed and delivery is not complete;
- `RebuildRequired`: recreate the projection from authoritative values;
- `Unavailable`: no usable projection exists;
- `Failed`: projection work failed with a recorded reason.

## Rebuild A Projection

```ts
const posture = await truth.derivation(branchId);

if (posture.posture === "RebuildRequired" ||
    posture.posture === "Unavailable" ||
    posture.posture === "Failed") {
  const rebuilt = await truth.rebuildDerivation(branchId);
  if (rebuilt.posture !== "Current") {
    reportProjectionFailure(rebuilt);
  }
}
```

`destroyDerivation(branchId)` intentionally removes disposable Signal state.
`rebuildDerivation(branchId)` reconstructs it from the current Local Truth
snapshot. Destroying or rebuilding a derivation must not change the truth
digest, branch head, values, or commit history.

## Read Retained History

```ts
const history = await truth.history(branchId);
if (history.posture !== "success") throw new Error(history.message);

for (const commit of history.value.commits) {
  renderCommit(commit.id, commit.kind, commit.operations);
}
```

A history segment contains an optional checkpoint plus the bounded commits
retained after it. `historicalSnapshot({ branchId, commitId })` reconstructs a
read-only value only when that commit is retained and belongs to the branch's
ancestry. It does not move the head or create a writable basis.

## Checkpoints And Compaction

```ts
const checkpoint = await truth.checkpoint(branchId);
if (checkpoint.posture !== "success") {
  reportCheckpointFailure(checkpoint);
}
```

Checkpointing records this branch even when the authority cannot compact yet.
Compaction is coordinated across the authority and occurs only after every
active branch has checkpointed its current head. This prevents one branch from
erasing ancestry another active branch still needs. Use `history(...)` and the
inspection counters to observe the retained boundary; do not interpret one
successful `checkpoint(...)` call as proof that global compaction occurred.

After compaction:

- the checkpoint preserves sealed values, lineage, locus heads, compacted
  digests, and a segment digest;
- the checkpoint head remains available as the history boundary;
- later commits remain individually retained;
- older individual commits before the checkpoint are intentionally
  unavailable through `historicalSnapshot`.

Ordinary branches currently have no public delete operation, so abandoned
branches can delay authority-wide compaction. Treat branch creation as a
bounded product decision.

## What A Checkpoint Is Not

There is no public export-and-restore API for a Local Truth checkpoint. The
checkpoint object is retained in this running authority; it is not a promised
wire format or durable recovery artifact. Serializing it yourself does not
create a supported restart protocol.

Do not call process-local history durable or restart-stable.

For durable history, concurrency, retention policy, and recovery, use:

```text
Query -> Relational -> Bridge -> Signal
```

Relational owns the durable commit and history contract. The bridge transports
committed causal change. Signal remains rebuildable derived state.

## Inspection Checklist

When debugging a browser-local workflow, record:

- `inspection.supportPosture` (`"inMemoryProcessLocal"`);
- the Local Truth branch head and digest;
- the derivation posture and reason;
- the retained history boundary and checkpoint digest.

Do not substitute a Signal value, UI cache, or DOM state for the Local Truth
basis. Those are consumers of authority, not sources of it.

## Related Docs

- [Branches And Snapshots](./branches-and-snapshots.md)
- [Authority Boundaries](./authority-boundaries.md)
- [Local Truth API Reference](./api-reference.md)
