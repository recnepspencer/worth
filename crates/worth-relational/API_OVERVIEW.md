# API Overview

This is the public shape of `worth-relational` as a standalone library.

## Public center of gravity

Import through:

```rust
use worth_relational::facade::*;
```

The product center is:

- `runtime`
- `transactions`
- `query`
- `schema`
- `snapshots`
- `history`
- `inspection`
- `publication`

## Main doors

### Setup

- `RelationalRuntimeApi::builder()`
- grouped setup sections on the builder:
  - `runtime_setup(...)`
  - `schema_setup(...)`
  - `storage_setup(...)`
  - `durability_setup(...)`
  - `strategy_setup(...)`

### Write truth

- choose an explicit `RelationalBranchIdentity`
- `runtime.observe_branch(...)`
- `runtime.begin_branch_transaction(...)`
- `tx.push_batch(...)`
- `tx.commit(&runtime)` for the ordinary convenience path
- or `runtime.prepare_branch_transaction(...)` followed by
  `runtime.publication_port().compare_and_publish(...)` and owner settlement

The admitted basis, not a branch name or optional selector, opens the governed
write path. See [`BRANCH_LOCAL_MVCC.md`](./BRANCH_LOCAL_MVCC.md).

### Read truth

- `runtime.read_truth()`
- `runtime.snapshots()`
- `runtime.snapshots().snapshot_for_observation(...)` for an exact pinned basis

`read_truth()` is an explicitly current standalone-runtime view. Work that must
remain attached to a selected branch version uses the observation carried by
an owner-admitted basis.

### Branches and component retention

- `runtime.main_branch_identity()` or `runtime.branch_identity(...)`
- `runtime.observe_branch(...)` and `runtime.readmit_branch_basis(...)`
- `runtime.observe_fork_source(...)` then `runtime.fork_branch(...)`
- `runtime.retain_component_basis(...)` and
  `runtime.release_component_basis(...)`
- `runtime.archive_branch(...)` and `runtime.delete_branch(...)`

Choosing the configured main branch is explicit. No governed entry point treats
an absent branch, `None`, or the string `"main"` as authority.

### Inspect what happened

- `runtime.inspect_what_happened()`
- `runtime.publication()`

### Publish semantic aspect changes

Use `facade::publication` when another runtime needs the authoritative meaning
of a committed change. `PublishedAuthoritativeAspectChange` retains aspect
identity and revision, Relational binding, change kind, optional field path,
and exact or declared-widening precision.

Runtime Bridge consumes this publication for Query-installed semantic
correspondence. Downstream callers should not derive their own aspect-change
taxonomy from raw patch fields.

### Past truth

- `runtime.history()`
- `runtime.history_authority()`
- `runtime.replay()`

These are canonical history, maintenance, and reconstruction lanes. They are
not alternate branch-head selection or publication doors.

## Contained real lanes

These are real public capabilities, but not the default first-contact story:

- `runtime.validation()`
- `runtime.compiled_artifacts()`
- `runtime.compiled_artifacts_authority()`
- `runtime.retention()`
- `runtime.index_access()`
- `runtime.index_authority()`
- `runtime.durability()`
- `runtime.durability_authority()`
- `runtime.commit_strategies()`
- `runtime.commit_strategies_authority()`
- `runtime.merge()`
- `runtime.replay_authority()`

## Published runtime lanes

The standalone library surface now uses the job-shaped runtime names directly:

- `read_truth()`
- `snapshots()`
- `inspect_what_happened()`
- `publication()`
- `history()`
- `history_authority()`
- `replay()`
- `validation()`
- `compiled_artifacts()`
- `retention()`
- `durability()`
- `merge()`

The exact component artifacts and outcomes available to a later composite
owner are frozen in [`OWNER_COMPONENT_PORT.md`](./OWNER_COMPONENT_PORT.md).

