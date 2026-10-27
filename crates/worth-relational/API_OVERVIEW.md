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

Prepared runtime integrations can inspect an unconsumed candidate with
`with_prepared_descriptive_touches(|graph| ...)`. It borrows the exact graph
already prepared for that candidate. `None` means its payload was consumed or
revoked; `Some` can contain an exact-empty or unavailable graph. The callback
must not reenter the candidate. Touches describe the candidate and confer no
source currentness or publication authority.

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

Downstream callers should not derive their own aspect-change taxonomy from raw
patch fields.

#### Who consumes this

The Bridge carries Query-installed semantic correspondence. It reads this
publication through `facade::change_source`, which checks the publication
before minting a change receipt.

#### Exact source identity and receipt authority

Use `facade::change_source` to retain the actual admitted observation, select
an exact or reachable native commit, and consume that `RelationalSelectedCommit`
with `mint_change_receipt`. The runtime that selected the commit must mint the
receipt; descriptive identifiers cannot recreate that authority. Read the
[source contract](src/facade/change_source.rs) for the executable entry sequence.

`facade::identity_authority` retains the concrete authority-category and family
kind types. The former Phase 1 family, root-break and future-test catalogs have
been removed: they were planning inventories, not issuance or executed compiler
proofs. Actual selection, receipt issuance and existing compile-boundary tests
remain their authorities.

### Past truth

- `runtime.history()`
- `runtime.history_authority()`
- `runtime.replay()`

These are canonical history, maintenance, and reconstruction lanes. They are
not alternate branch-head selection or publication doors.

### Bounded record and aspect history

Native consumers import the page types from `facade::history` and use the
runtime's `retain_record_history_subject(execution, &basis, record)` to resolve
an exact entity or relation generation in a genuinely admitted root. Retained
tombstones are valid subjects; zero-generation wildcards, unavailable subjects
and foreign owners are refused. The subject retains that original observation.

Select a commit with `select_exact_commit`, then pass the sealed selection,
retained subjects and selected aspect keys to
`record_aspect_history_cursor(execution, selected, &subjects, &aspects)`.
The selected commit is independent of each subject's original observation, so a
composition owner can inspect older component commits without substituting the
subject's provenance. The native cursor establishes no World product ancestry
or application disclosure permission.

`next_page(&runtime, execution, limits)` bounds entries, canonical record-patch
visits and newly owned page backing. It can stop inside a large commit or return
an empty incomplete page. Only selected keys are exposed; structural occurrences
remain observable even when no selected aspect changed. A retained native
`Updated` record with no canonical aspect operations also remains an occurrence,
including retained equal-value writes. Its changed-aspect list is empty: the
canonical source no longer attributes it to any authored aspect, so no selected
aspect is claimed to have changed. Nonempty unrelated-aspect patches stay
filtered. The immutable canonical envelope stays shared and
retained; its existing storage is not covered by the new-page byte limit.
Cancellation, deadline, allocation and owner refusals leave the accepted position
unchanged. A composition owner can use `prepare_next_page` to stage native work,
then `accept_page` after fresh authorization/disclosure. It returns the page and
a sealed successor cursor while leaving the original unchanged. Install the
aggregate continuation only after the composition's final original-request
handoff; dropping a prepared page or uninstalled successor declines advancement.
Acceptance rejects another cursor's proof or a stale position and uses its stored
originating request at handoff. Drop the
cursor, subjects and returned entries/pages to release their
native retention leases and charged backing. The older eager branch-wide history
selectors retain their existing contract.

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

