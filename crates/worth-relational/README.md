# worth-relational

`worth-relational` is a standalone truth runtime for graph-shaped state.

It is for systems that need state to be authoritative, transactional,
inspectable, replayable, and durable, rather than a mutable data structure
whose correctness rests on convention.

It is meant for more than CRUD apps. The target is systems like order and
inventory management, account ledgers, document approval workflows, planning
systems, and other runtimes where history, determinism, replay, validation,
branching, and durable recovery are first-class.

The library is built around a few obvious jobs:

- build runtime
- write truth
- read truth
- manage snapshots and branches when you need controlled views of truth
- inspect what happened
- go to history or replay when you need past truth
- use merge, validation, compiled artifacts, retention, indexes, and recovery
  when the workload gets deep

Start here:

- [`QUICKSTART.md`](./QUICKSTART.md)
- [`DAILY_WORKFLOWS.md`](./DAILY_WORKFLOWS.md)
- [`API_OVERVIEW.md`](./API_OVERVIEW.md)
- [`BRANCH_LOCAL_MVCC.md`](./BRANCH_LOCAL_MVCC.md)
- [`OWNER_COMPONENT_PORT.md`](./OWNER_COMPONENT_PORT.md)
- [`TESTING_WORLDS.md`](./TESTING_WORLDS.md)

Examples:

- `cargo run -p worth-relational --example basic_runtime`
- `cargo run -p worth-relational --example snapshots`
- `cargo run -p worth-relational --example history_and_replay`
- `cargo run -p worth-relational --example derived_indexes`
- `cargo run -p worth-relational --example validation_and_durability`
- `cargo run -p worth-relational --example branch_local_mvcc`

## Minimal shape

```rust
use worth_relational::facade::{
    mvcc::RelationalTransactionIntent,
    runtime::RelationalRuntimeApi,
    schema::RelationalSchemaRegistry,
    transactions::WorkerIntentBatch,
};

let runtime = RelationalRuntimeApi::builder()
    .schema_registry(RelationalSchemaRegistry::new())
    .build();

let main = runtime.main_branch_identity();
let (_descriptor, basis) = runtime.observe_branch(&main)?;
let mut tx = runtime.begin_branch_transaction(
    &basis,
    RelationalTransactionIntent::ordinary(),
)?;
tx.push_batch(WorkerIntentBatch::new("example"))?;
let _outcome = tx.commit(&runtime)?;

let _truth = runtime.read_truth();
let _snapshots = runtime.snapshots();
let _history = runtime.history();
let _inspection = runtime.inspect_what_happened();
```

## Mental model

- `RelationalRuntimeApi::builder()` is the setup door
- an explicit identity plus owner-admitted basis is the branch-selection door
- branch-bound transactions are the write-truth door
- `read_truth()` is the explicitly current standalone-truth door
- an exact observation plus `snapshots()` is the repeatable-view door
- `inspect_what_happened()` and `publication()` are the readback doors
- `history()` and `replay()` are the past-truth doors
- `validation()`, `compiled_artifacts()`, `retention()`, `durability()`,
  `commit_strategies()`, and indexes are the deep-system doors

If you find yourself reaching into crate internals instead of
[`facade`](./src/facade.rs), you are probably leaving the intended public
surface.

The owner catalog, branch cells, roots, and retention accounting are currently
memory-resident. Restart durability for this branch-owner model is deferred to
Worth Store integration.

## Validated transitions before recovered installation

`RelationalNativeCheckpoint` retains immutable byte backing. Clones share that
backing and preserve the selected byte region and any runtime capture-section
metadata. `from_untrusted_bytes_region` accepts `ExecutionImmutableBytes` so an enclosing
checkpoint can pass the same backing without copying its payload. Wrapping a
moved external box allocates an Arc header, not another payload buffer. Imported
native bytes remain uncharged; an admitted enclosing backing retains its whole
payload reservation through region clones until the last shared owner drops. Equality compares
selected byte values. Byte custody does not confer recovery authority; each
restore still authenticates and readmits the native checkpoint.

`restore_native_checkpoint_with_authority` issues linear recovery authority
for the verified runtime's exact branch images. Applications that must change
that image before installing their recovered world can prepare a normal native
commit and pass it, with that authority, to
`durability_recovery().commit_checkpoint_transition(...)`.

The recovery owner checks the candidate's exact recovered predecessor and uses
native publication and settlement. An acknowledged result yields the commit
receipt and recovery authority for the performed successor. Ordinary commits
do not refresh recovery authority. Relational attests native lineage; the
application still owns predecessor selection and target migration validation.

Refusal returns the original authority. A durable append failure returns a
linear deferred transition, which grants no successor authority. Pass it to
`repair_checkpoint_transition(...)` to repair the existing native settlement
route without publishing another commit. Failed repair returns the same custody
for retry; successful repair carries the performed successor, even if current
branch state has moved since that performance. These types are available through
`worth_relational::facade::durability`.

## Aspect-Precise Publication

Committed patches are interpreted against the installed schema before they
cross a runtime boundary. The publication facade exposes
`PublishedAuthoritativeAspectChange`, which retains:

- aspect key, opaque identity, and contract revision
- the exact entity, relation, endpoint, structural, or lifecycle binding
- whole-aspect, field, endpoint, structural, lifecycle, or opaque change kind
- an optional canonical field path
- `Exact` or explicitly declared widening precision

This is the authoritative change meaning. Relational does not allocate Signal
aspects, and downstream callers must not reinterpret raw patch fields into
their own change taxonomy.

Use `worth_relational::facade::publication` and
`worth_relational::facade::schema`. Equal labels or diagnostic digests do not
replace the typed aspect identity and binding.

#### Who consumes this

The Bridge reads committed changes through
`worth_relational::facade::change_source`. It selects a commit, has Relational
mint a change receipt, which proves the publication is consistent, and lowers
that receipt into its own envelope. Relational does not depend on the Bridge.
For Query-installed conditional operations, the flow is:

```text
Relational commit
  -> aspect-precise authoritative publication and change receipt
  -> Bridge installed correspondence
  -> Signal invalidation and decision
  -> Query consequence
```

