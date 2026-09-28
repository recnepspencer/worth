# Daily Workflows

This library gets much easier if you think in jobs instead of modules.

## Build runtime

Use:

- `RelationalRuntimeApi::builder()`
- `profile(...)` if you want a predefined setup shape
- `runtime_setup(...)`
- `schema_setup(...)`
- `storage_setup(...)`
- `durability_setup(...)`
- `strategy_setup(...)`
- `build()`

## Write truth

Use:

- an explicit branch identity
- `observe_branch(...)`
- `begin_branch_transaction(...)`
- `push_batch(...)`
- `commit(&runtime)`

For larger staged writes, `plan_bulk_mutation_batch(...)` is the main advanced
helper.

For integration or controlled publication work, split the same path at
`prepare_branch_transaction(...)`, pass the opaque candidate only to
`publication_port().compare_and_publish(...)`, and settle a `Performed`
outcome through its owner. Prepared, stale, denied, interrupted, deferred, and
failed work is not a commit.

## Read truth

Use:

- `read_truth()`
- `snapshots()`
- query APIs when you need larger or more selective reads

Use `read_truth()` only when explicitly current standalone truth is the job.
When work already carries an admitted branch basis, read from its exact
`RelationalBranchObservation` instead of asking for whichever root is current
later.

Do not start with history, replay, or storage helpers unless that is actually
the job.

When you need a pinned read basis, call
`snapshots().snapshot_for_observation(&basis.observation())` and release the
returned handle exactly once.

## Inspect what happened

Use:

- `inspect_what_happened()`
- `publication()`

Think of this as the operator readback zone:

- what changed?
- what published?
- what is wrong?
- what is retained?

## Go to history

Use:

- `history()`
- `history_authority()` only for its explicit canonical-history maintenance
  operations

This is the next door after current truth when the question becomes "what did
truth look like before?"

History does not create, select, or move ordinary branch authority. Fork with
`observe_fork_source(...)` and `fork_branch(...)`; manage lifecycle with
`archive_branch(...)` and `delete_branch(...)`.

## Go to replay

Use:

- `replay()`

This is the deeper verification lane after history.

## Deep lanes

Use these only when the job really calls for them:

- `validation()`
- `compiled_artifacts()`
- `retention()`
- `index_access()` / `index_authority()`
- `merge()`
- `durability()`
- `durability_authority()`
- `commit_strategies()`
- `commit_strategies_authority()`

That is not because they are hidden.

It is because they are real high-power lanes and should feel intentional.

For the complete flow, use the runnable `branch_local_mvcc` example and read
[`BRANCH_LOCAL_MVCC.md`](./BRANCH_LOCAL_MVCC.md). Component integrators should
also read [`OWNER_COMPONENT_PORT.md`](./OWNER_COMPONENT_PORT.md).
