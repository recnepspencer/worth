# Quickstart

This is the fastest honest path through `worth-relational`.

## 1. Build a runtime

```rust
use worth_relational::facade::{
    runtime::RelationalRuntimeApi,
    schema::RelationalSchemaRegistry,
};

let runtime = RelationalRuntimeApi::builder()
    .runtime_setup(|runtime| {
        runtime.runtime_name("quickstart");
    })
    .schema_setup(|schema| {
        schema.schema_registry(RelationalSchemaRegistry::new());
    })
    .build();
```

The normal setup shape is:

- start with `RelationalRuntimeApi::builder()`
- optionally shape grouped setup sections
- provide a schema registry through `schema_setup(...)`
- add storage / durability / strategy setup only when you actually need them
- `build()`

## 2. Write truth

```rust
use worth_relational::facade::{
    mvcc::RelationalTransactionIntent,
    transactions::WorkerIntentBatch,
};

let main = runtime.main_branch_identity();
let (_descriptor, basis) = runtime.observe_branch(&main)?;
let mut tx = runtime.begin_branch_transaction(
    &basis,
    RelationalTransactionIntent::ordinary(),
)?;
tx.push_batch(WorkerIntentBatch::new("seed"))?;
let commit = tx.commit(&runtime)?;
```

That is the default write story:

- explicitly choose and observe the exact owner branch basis
- begin a branch-bound transaction
- push batch
- commit

There is no optional target or ambient `"main"` fallback. The complete
prepare/compare-and-publish/settle form is the runnable
`branch_local_mvcc` example.

## 3. Read truth

```rust
let truth = runtime.read_truth();
```

`read_truth()` is the current-truth door.

If you need a stable pinned view instead of "whatever is current right now",
observe the selected branch and open the snapshot from its exact observation:

```rust
let main = runtime.main_branch_identity();
let (_descriptor, basis) = runtime.observe_branch(&main)?;
let snapshot = runtime
    .snapshots()
    .snapshot_for_observation(&basis.observation())?;

// Read through the snapshot view, then release its retention obligation.
runtime.snapshots().release_snapshot(&snapshot)?;
```

## 4. Inspect what happened

```rust
let inspection = runtime.inspect_what_happened();
let publication = runtime.publication();
let history = runtime.history();
let replay = runtime.replay();
```

Use:

- `inspect_what_happened()` for inspection
- `publication()` for publication-facing outputs
- `history()` when you need past truth
- `replay()` when you need verification or reconstruction

Fork and lifecycle are owner operations, not history-authority shortcuts. Use
`observe_fork_source` plus `fork_branch`, then `archive_branch` or
`delete_branch` when that branch reaches its lifecycle boundary.

## 5. Reach for deeper lanes only when needed

```rust
let validation = runtime.validation();
let compiled = runtime.compiled_artifacts();
let retention = runtime.retention();
let indexes = runtime.index_access();
```

Those are real public lanes.

They are just not the first five minutes of normal usage.

After this file, the fastest concrete references are the runnable examples:

- `basic_runtime`
- `snapshots`
- `history_and_replay`
- `derived_indexes`
- `validation_and_durability`
- `branch_local_mvcc`

Then read [`BRANCH_LOCAL_MVCC.md`](./BRANCH_LOCAL_MVCC.md). If you are
integrating a composite owner, continue with
[`OWNER_COMPONENT_PORT.md`](./OWNER_COMPONENT_PORT.md).

