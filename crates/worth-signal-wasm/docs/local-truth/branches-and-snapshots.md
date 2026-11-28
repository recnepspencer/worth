# Branches And Snapshots

## The Frame

The easy path is one `main` branch. Fork only when two lines of work genuinely
need independent application values. Local Truth then gives each branch an
immutable head and snapshot without turning Signal's derived branch machinery
into a second truth store.

This is still browser-process-local authority. If a branch must survive a
restart, be shared by users, or participate in durable audit, use Query and
Relational instead.

## Mental Model

```text
genesis -- main commit -- main commit
              \
               experiment commit -- experiment commit
```

A branch receipt identifies the branch and carries its current `basis`. A
basis names the exact authority, schema, branch, head commit, snapshot, and
revision observed by the caller. It is a capability issued by this runtime,
not a version number to fabricate.

Every admitted commit:

1. checks the runtime-issued expected basis;
2. validates all declared aspect operations before publishing any of them;
3. seals one immutable snapshot and commit;
4. advances exactly one branch head;
5. projects the committed value into Signal.

If the head moved, the request is denied as stale. Read the branch again,
decide whether the operation still makes sense, and submit a new request.

## Fork A Branch

```ts
const main = await truth.branch();
if (main.posture !== "success") throw new Error(main.message);

const fork = await truth.forkBranch({
  parentBranchId: main.value.id,
  expectedParentBasis: main.value.basis,
  name: "smaller-drive-gear",
});
if (fork.posture !== "success") throw new Error(fork.message);
```

The child starts at the parent's exact fork commit and snapshot. Later parent
and child commits do not overwrite one another.

## Commit One Semantic Change

```ts
const child = await truth.branch(fork.value.id);
if (child.posture !== "success") throw new Error(child.message);

const changed = await truth.commit({
  requestId: crypto.randomUUID(),
  branchId: child.value.id,
  expectedBasis: child.value.basis,
  operations: [{ entityId: "gear", aspectId: "teeth", value: 16 }],
});
if (changed.posture !== "success") throw new Error(changed.message);
```

Use one `requestId` for one intended mutation. Replaying the same request is
idempotent; reusing it for different content is denied.

Multiple operations in one request are atomic at the Local Truth boundary. An
invalid entity, aspect, value type, or basis means no operation publishes.

## Read Current And Historical Values

Use `branch(id)` when you need a current basis. Use `inspect()` for diagnostic
views across all branches. Use `historicalSnapshot(...)` only for a retained
commit on that branch's ancestry:

```ts
const past = await truth.historicalSnapshot({
  branchId: fork.value.id,
  commitId: changed.value.commit.id,
});

if (past.posture === "success") {
  renderReadOnlyGear(past.value.values.gear);
}
```

Historical reads do not move a head and do not create a new writable basis.
A commit from a sibling branch is not part of this branch's history and is
denied even if both branches contain equivalent values.

## Signal Is Downstream

Each Local Truth branch gets a disposable Signal projection. The Local Truth
snapshot is authoritative; the Signal branch is derived execution state. If
projection fails after a commit, the branch head stays committed and
`derivation(branchId)` reports `RebuildRequired` or `Failed`.

Never read a Signal input and reconstruct an authority basis from it. Rebuild
the projection from the Local Truth snapshot instead.

## Practical Boundaries

- Ordinary branches currently have no public delete operation. Long-lived
  abandoned branches remain active and can prevent global history compaction.
- Branch names are labels, not identities or authorization.
- A snapshot is immutable process memory, not persisted storage.
- Commit `metadata` is a host assertion, not authenticated actor identity.
- Forking is not collaboration or synchronization between browser processes.

## Next

- [History, Compaction, And Rebuild](./history-and-rebuild.md)
- [Authority Boundaries](./authority-boundaries.md)
- [Local Truth API Reference](./api-reference.md)
