# History, Replay, And Runtime Branches

The history surface records runtime execution state, snapshots, lineage, and
runtime branches. Use it to inspect or restore the runtime's own derived world.

Do not confuse that with durable application history. Runtime history can be
extremely exact and still belong to a process-local execution engine.

## Stable Entry Points

- `signals.history()`
- `history.replay_for(id)`
- `history.lineage_for(id)`
- `history.snapshot()`
- `history.restore_exact_snapshot(snapshot)`
- `history.discard_exact_artifact(artifact)`
- `history.current_branch()`
- `history.create_branch(name)`
- `history.switch_branch(branchId)`
- `history.plan_merge_branches(...)`
- `history.merge_branches(...)`
- `graph.inspectHistory()`

## Inspect Replay And Lineage

```ts
const replay = await signals.history().replay_for(total.id);
const lineage = await signals.history().lineage_for(total.id);

console.log(replay);
console.log(lineage);
```

Replay describes retained execution for a runtime node. Lineage explains how
that node relates to its execution ancestry. At a published boundary,
`graph.inspectHistory()` is usually easier to read because it uses public names.

## Capture And Restore An Exact Snapshot

```ts
const history = signals.history();
const snapshot = await history.snapshot();

await count.set(9);
await history.restore_exact_snapshot(snapshot);
```

The exact snapshot carries a same-runtime restore token. Treat that token as an
authority artifact, not a generic JSON backup format.

Exact wire restore tokens are single-use transfer authorities. Restoring
consumes the token. Each JavaScript/worker realm keeps its 64 most recent
pending exact restore artifacts: minting the 65th evicts the oldest pending
one, and redeeming an evicted, consumed, or discarded token fails with
`restoreTokenNotPending` (a token the realm never issued fails with
`invalidInput`). Every `history.snapshot()`, `history.branch_snapshot()`,
`history.branch_snapshot_envelope()`, and `adapters.exportRuntimeEnvelope()`
call mints one, and a worker-first root refreshes its cached artifacts after
every mutation, so hold on to the artifact you intend to restore and restore
it before minting 64 more. Release an artifact you will not restore with
`history.discard_exact_artifact(artifact)` or
`adapters.discardExactRuntimeEnvelope(envelope)` (the raw module exposes the
same release as `discardRestoreToken(token)` and the current count as
`pendingRestoreTokenCount()`). Portable wire artifacts do not occupy this
registry and never expire.

A runtime snapshot belongs to the branch it was captured on. Restoring it
reactivates that branch first, so a snapshot taken before a branch was
created, switched to, or merged remains restorable afterwards. A restore that
is refused (unknown snapshot, callback unavailability) leaves the active
branch unchanged.

Capturing a snapshot, branch snapshot, branch-state proof, or replay view is
an observation. It never changes what a later merge adopts: edits made on a
branch stay merge-visible however many artifacts were minted in between
(the worker-first root mints them after every mutation). Restoring a snapshot
is different: it reinstates the merge boundary the snapshot was captured at.
Importing a portable snapshot artifact also refreshes the history views
(`replay`, `lineage`, diagnostics) so they describe the imported runtime.

Exact same-runtime restore artifacts are root-branch evidence. While a child
branch is active, `history.snapshot()` still works, but
`adapters.exportRuntimeEnvelope()` and `graph.exportSnapshot()` throw
`invalidInput` ("exact runtime restore artifacts require the root branch to
be active") on every deployment; a worker-first root reports the same denial
from its cached artifacts instead of minting on a child branch. Switch to the
root branch, or merge first, then export.

## Runtime Branches

```ts
const history = signals.history();
const main = await history.current_branch();
const experiment = await history.create_branch("experiment");

await history.switch_branch(experiment.id);
await count.set(12);

const plan = await history.plan_merge_branches(experiment.id, main.id);
console.log(plan);
```

A runtime branch stages an alternative runtime state and retains branch
ancestry. It is useful for execution experiments, replay, and exact runtime
restore.

After a merge the merged parent's published outputs are recomputed from the
merged inputs when you switch to it (they are standing demand), so the first
read sees the merged truth. Other computeds recompute on their next read.

## Runtime Merge Versus Application Merge

Runtime merge operates on native Signal branch state. It does not understand
that `teeth`, `thickness`, or `approval` are application aspects requiring a
human decision.

Use [Local Truth](../local-truth/README.md) when application values need declared
aspects, stale-basis denial, conflict alternatives, and manual resolution.
Signal then consumes the committed result as derivation.

## Worker-First Parity

Worker-first history owns branch lifecycle and exact snapshot restore in the
worker runtime. Calls may be asynchronous, so awaiting history operations is the
portable application posture.

## Anti-Patterns

- Do not store business records only in runtime snapshots.
- Do not use a runtime branch ID as an application-value merge decision.
- Do not edit restore tokens.
- Do not mint an exact artifact you will not restore; discard the ones you
  abandon, and do not expect an artifact minted 64 exports ago to still be
  pending.
- Do not describe retained process history as durable cross-process audit
  history.

## Related Docs

- [Diagnostics And Explanation](./diagnostics.md)
- [Graphs And Controllers](./graphs-and-controllers.md)
- [Branch Merge And Manual Resolution](../local-truth/branch-merge.md)
