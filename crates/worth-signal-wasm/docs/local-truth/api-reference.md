# Local Truth API Reference

## Scope

This reference covers the public TypeScript surface exported by
`worth-signals-wasm`. Local Truth is a process-local authority with a required
Signal projection. It is not a persistence or distributed-state API.

## Create A Schema

```ts
const schema = localTruthSchema<Entity>({
  id: "entity",
  version: 1,
  aspects: [{
    id: "status",
    field: "status",
    valueType: "string",
    equivalence: { kind: "exact" },
    costClass: "constant",
  }],
});
```

`declareLocalTruthSchema(...)` is the canonical function;
`localTruthSchema(...)` is its exported alias.

| Declaration | Accepted values |
| --- | --- |
| `valueType` | `"any"`, `"boolean"`, `"number"`, `"string"` |
| `equivalence` | `{ kind: "exact" }` or `{ kind: "numberEpsilon", epsilon }` |
| `costClass` | `"constant"` or `"linearInValue"` |

Each aspect ID and field must be unambiguous. Aspects map to declared top-level fields on plain objects.

## Create An Authority

```ts
const truth = signals.localTruth({
  authorityId,
  schema,
  initialEntities,
  bindings: [{ entityId, input, aspectMap }],
});

await truth.ready?.();
```

Initial entities must satisfy the schema. A binding connects each schema
aspect ID to a numeric Signal aspect produced by the input. The initial entity
and input value must agree.

The returned `LocalTruthAuthority<T>` has
`kind: "typescriptInMemoryLocalTruth"`. Call `terminate()` when the authority
is no longer needed.

## Outcomes

Most methods return `LocalTruthOutcome<T>`:

- `success`: admitted value;
- `advisory`: admitted value plus non-fatal advisories;
- `denied`: the request violated a contract and published nothing;
- `unavailable`: the requested retained artifact or capability is absent;
- `failed`: execution could not complete; inspect the code and evidence.

Handle outcomes explicitly. Do not use a failed response's evidence as a new
basis.

## Current Branch And Commit

### `branch(branchId?)`

Returns the named branch or the main branch. The receipt includes the current
runtime-issued `basis`, head commit, snapshot, parent/fork information, and
derivation receipt.

### `commit(request)`

```ts
truth.commit({
  requestId,
  branchId,
  expectedBasis,
  operations: [{ entityId, aspectId, value }],
  metadata,
});
```

All operations are validated and published atomically. `requestId` makes an
identical replay idempotent. `metadata` is an unauthenticated host assertion.
On success, the outcome contains the immutable commit and the downstream
derivation posture.

## Branches

### `forkBranch(request)`

Requires `parentBranchId`, a current `expectedParentBasis`, and a `name`.
Returns a child branch beginning at the exact parent head.

There is currently no public delete operation for ordinary branches.

## Projection Lifecycle

### `derivation(branchId?)`

Returns `Current`, `CommittedDerivationPending`, `RebuildRequired`,
`Unavailable`, or `Failed`.

### `destroyDerivation(branchId)`

Disposes the branch's Signal projection without changing Local Truth.

### `rebuildDerivation(branchId)`

Recreates derived Signal state from the authoritative current snapshot. It
does not reconstruct Local Truth from Signal.

## History

### `history(branchId)`

Returns a bounded segment: optional checkpoint plus individually retained
commits after it.

### `historicalSnapshot({ branchId, commitId })`

Returns read-only values for a retained ancestor commit. It does not move the
head or issue a writable historical basis. Sibling commits and compacted-away
individual commits are unavailable or denied.

### `checkpoint(branchId)`

Checkpoints the current branch head. Authority-wide compaction waits until all
active branches have checkpointed their current heads. There is no public
durable export/restore contract for the checkpoint.

## Inspection

### `inspect()`

Returns authority and schema identity, support posture, revision, branches,
heads, per-branch values, decision log, counters, optional bridge counters,
and a digest.

Inspection is diagnostic. Its collections and digests do not grant mutation
authority and do not constitute durable audit evidence.

## Common Denials

- stale, foreign, or fabricated basis;
- unknown or retired branch;
- unknown entity/aspect or invalid value type;
- request ID reused with different content;
- selection or alternative from another review/conflict;
- custom resolution touching the wrong locus or committing twice;
- historical commit outside the branch ancestry;
- checkpoint request for an unknown or ineligible branch.

## Platform Boundary

Use `Query -> Relational -> Bridge -> Signal` for durable, shared,
transactional, authenticated, or cross-process truth. See
[Local Truth And Platform Authority](./authority-boundaries.md).
