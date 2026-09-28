# Relational Branch-Local MVCC

`worth-relational` publishes truth by moving one explicit branch reference from
one complete immutable root to another. A branch name, commit ID, version,
snapshot handle, serialized descriptor, or ambient idea of "main" is not enough
to authorize that movement.

The runnable public-facade example is
[`examples/branch_local_mvcc.rs`](./examples/branch_local_mvcc.rs). It is
executed, not merely compiled, by `cargo test`. Every Rust snippet this guide
would otherwise print lives either in that example or in the rustdoc named
below, so that a compiler proves each claim rather than this file asserting it.

## Mental model

Keep four things distinct:

- A **branch identity** selects one runtime-owned mutable reference cell.
- A **basis descriptor** describes one exact observed target and generation. It
  is portable data, not authority.
- An **admitted basis** is the owner-issued, retained authority to read or begin
  work from that exact observation.
- A **branch root** is the complete immutable state selected by the reference.
  Readers never observe a partially installed root.

The ordinary write progression is:

```text
explicit identity
  -> observe exact owner basis
  -> detached branch-bound transaction
  -> validated proposal
  -> prepared single-use candidate
  -> branch-local compare-and-publish
  -> performed owner commit
  -> settlement and exact successor basis
```

`main_branch_identity()` is an explicit owner observation helper. Governed
operations never accept `None`, a string named `"main"`, or a missing target as
permission to choose it.

## Stable public entry points

Import through `worth_relational::facade`:

- `branch` owns identities, descriptors, admitted bases, the fork service, fork
  outcomes, lifecycle outcomes, and external retention leases.
- `mvcc` owns exact observations, branch-bound transactions, prepared
  candidates, publication outcomes, and the preparation, publication, and
  settlement services.
- `snapshots` opens a pinned read only from an exact owner observation.
- `history` and `publication` inspect canonical committed artifacts; they do
  not provide a second branch-currentness authority.

### The four owner services

Governed movement is split across four services. Each is obtained from a
*shared* borrow of the runtime and is `Clone + Send + Sync`, so unrelated
branches progress at once without any caller excluding another and without
wrapping the runtime in a mutex. The runtime is not the unit of exclusion; the
branch reference cell is.

| Service | Facade route | Operations |
| --- | --- | --- |
| Preparation | `mvcc::RelationalPreparationPort` | `prepare_branch_transaction`, `discard_prepared_candidate` |
| Fork | `branch::RelationalForkPort` | `observe_fork_source`, `fork_branch` |
| Publication | `mvcc::RelationalPublicationPort` | `compare_and_publish` |
| Settlement | `mvcc::RelationalSettlementPort` | `settle_performed_publication`, `repair_deferred_publication_settlement`, `repair_pending_publication_settlement` |

Obtain them with `preparation_port()`, `fork_port()`, `publication_port()`, and
`settlement_port()`. The rustdoc on `preparation_port` carries the compiled
prepare-then-discard progression.

The remaining ordinary runtime methods take `&self`, so an open transaction,
candidate, or service clone never excludes another owner:

- `main_branch_identity`, `branch_identity`, and `observe_branch`;
- `readmit_branch_basis` and `readmit_retained_branch_basis`;
- `begin_branch_transaction`, `prepare_branch_transaction`, and
  `commit_branch_transaction`; and
- `retain_component_basis` and `release_component_basis`.

The shared-borrow guarantee covers those four services and these ordinary
methods, not the runtime's whole surface. Lifecycle and maintenance keep
exclusive receivers: `archive_branch`, `delete_branch`, and
`run_branch_root_reclamation_pass` take `&mut RelationalRuntime` and must be
sequenced outside concurrent owner work. See
[Owner services](./OWNER_COMPONENT_PORT.md#owner-services) for the normative
statement.

## Exact reads

`observe_branch` returns a serializable `RelationalBranchBasisDescriptor` and
an `AdmittedRelationalBranchBasis`. The descriptor is transportable evidence,
not permission; the admitted basis is the authority that opens governed work.
Use the admitted basis's `RelationalBranchObservation` to open a snapshot, and
release that snapshot exactly once.

The compiled read progression is the doctest on `RelationalRuntime::observe_branch`
(`src/branch/basis_observation.rs`).

Moving the branch later does not change that snapshot. A transported descriptor
must pass `readmit_branch_basis`; deserialization cannot restore authority.

## Detached transactions and preparation

`begin_branch_transaction` validates runtime identity, branch identity,
lifecycle, exact currentness, and retention before returning a detached
`BranchBoundRelationalTransaction`. The transaction owns its overlay, declared
read/write footprint, and retained basis. It holds no borrow of the runtime for
its lifetime; the owner operations that consume it take `&RelationalRuntime`,
so an open transaction never excludes another owner.

Preparation is its own service. It performs the fallible work before effects:
schema and invariant validation, footprint validation, canonical commit
assembly, immutable-root materialization, and resource checks. Those checks are
where the bounded budgets are refused, before any candidate exists and before
any effect: `prepare_branch_transaction` returns
`Err(TransactionCommitError::PublicationDeferred { .. })` carrying
`PublishedSnapshotCapacityExhausted { maximum_handles }`,
`CandidateCapacityExhausted { maximum_candidates }`, or
`RetentionBackpressure`.

A `PreparedRelationalCommitCandidate` is opaque, runtime-affine, branch-bound,
single-use, and retained. Preparing, discarding, expiring, or losing a
candidate does not move a public reference; `discard_prepared_candidate` is the
explicit way to consume one without publishing, and the `preparation_port`
doctest proves the branch does not move across it.

The convenience `transaction.commit(&runtime)` follows the same
prepare/publish/settle path through the same services. It is not a second
authority; it simply does not hand back the explicit linearization outcome.
Integration owners that need that outcome use the publication service described
in [`OWNER_COMPONENT_PORT.md`](./OWNER_COMPONENT_PORT.md).

## Linearization and conflict

`RelationalPublicationPort::compare_and_publish` compares the candidate's full
expected observation with the selected branch cell inside one bounded
branch-local critical section. It installs one complete next root and advances
that reference's generation exactly once.

The terminal outcomes are intentionally distinct:

- `Performed` means the branch reference moved and carries the canonical commit
  plus successor admitted basis. The owner must still settle durability and
  optional projections.
- `Stale` means another movement won; it carries complete expected and observed
  descriptors and moves nothing for this candidate.
- `Denied` means owner, runtime, branch, or lifecycle admission failed.
- `Interrupted` means cancellation or timeout won before linearization.
- `Deferred` means one bounded mechanical condition stopped this attempt:
  `PatchPositionReservationContended`, `RetentionBackpressure`, or
  `CandidateLifetimeExpired`. The two capacity variants of
  `RelationalPublicationDeferred` are refused earlier, during preparation, and
  never arrive here.
- `Failed` means an owner invariant or required publication step failed before
  movement.

The candidate is consumed on every one of these outcomes. No outcome hands back
a reusable candidate.

### The next action after a deferred patch position

`Deferred(PatchPositionReservationContended)` is a typed no-movement terminal
outcome, not an error and not a rebase authority. It means this publication lost
a bounded, nonblocking global reservation to a simultaneous publisher.

The caller's next action is explicit and owned by the caller: begin a **fresh
transaction** and perform a **fresh preparation**. The same still-admitted basis
may be reused, because a contended mechanical reservation does not invalidate
the caller's observation and does not require paying for a second one. The owner
never retries, never rebases, and never reuses a candidate on the caller's
behalf. Retries must be bounded and must fail by name when the bound is
exhausted; `examples/branch_local_mvcc.rs` implements exactly this loop.

## Concurrency boundary

What is concurrent:

- Unrelated branch cells. Different branches do not share an ordinary
  publication lock, so two branches can prepare, publish, and settle at the
  same time through their own clones of the owner services.
- Observation, transaction admission, preparation, and settlement, which take
  shared receivers throughout.

What is not:

- Same-reference contenders. Two candidates expecting the same observation have
  exactly one winner; the loser receives `Stale` and moves nothing.
- Global identity and patch-position reservations. These are bounded,
  nonblocking, and counted separately. They are mechanical reservations, not
  branch coordination, and a contended one is reported rather than waited on.
- Branch lifecycle and maintenance. `archive_branch`, `delete_branch`, and
  `run_branch_root_reclamation_pass` take `&mut RelationalRuntime` and must be
  sequenced outside concurrent owner work.

Wrapping the runtime in `Arc<Mutex<_>>` does not add concurrency and removes
what exists; the services are already the shared-access mechanism.

This milestone makes no claim that Signal's mutation engine becomes
concurrently borrowable, and no claim of composite Relational-plus-Signal
publication.

## Cancellation

Cancellation is observed at named boundaries. Before the critical section it
returns `Interrupted` and the old reference remains current. Once the owner has
crossed the linearization point, cancellation cannot turn performed work into
a failure: `PerformedRelationalCommit::late_interruption()` records the event,
and settlement must still complete or enter its typed recovery lane.

Prepared candidates and performed-but-unsettled commits retain their required
roots until their terminal path transfers or releases the obligation. Dropping
work is not a substitute for reporting success.

## Fork, retention, and lifecycle

Fork is a separate owner transition with its own service. `observe_fork_source`
issues a linear, fork-only source basis; an empty branch produces none, so a
branch must have committed before it can be forked. `fork_branch` consumes that
token, creates a fresh reference cell, and shares the exact immutable source
root and canonical commit artifact. It does not clone authoritative truth or
infer a source from a branch label.

Every admitted observation, snapshot, transaction, candidate, performed
settlement, and external component holder has a bounded retention obligation.
Use `retain_component_basis` when another component must keep an exact basis
resident, and consume the lease through `release_component_basis`. The lease
does not claim the retained basis is still current; it only preserves the
immutable target for exact readmission.

Archive advances lifecycle metadata and denies new ordinary work. Deletion and
its interaction with immutable retention are specified by the owner port; see
[Deletion and immutable retention](./OWNER_COMPONENT_PORT.md#deletion-and-immutable-retention).
Reclamation scans belong to the explicit maintenance lane, never transaction
admission or ordinary publication.

## Structural sharing and cost

Branch roots use persistent copy-on-write regions. Fork performs constant
reference/cell/retention work and copies zero authoritative truth bytes and
zero canonical commit envelopes. A write materializes only touched regions and
root paths; unchanged regions, schema registries, and ancestry remain shared by
allocation identity.

`observe_branch_sharing` keeps these claims separable:

- logical bytes reachable from each branch;
- unique physical authoritative bytes;
- reused and newly materialized regions;
- copied truth and commit-envelope counts;
- branch-local wait/contact counters; and
- ordinary work versus maintenance reconstruction work.

Each reported metric documents its own scope in rustdoc. In particular
`branch_metadata_bytes` counts shallow inline branch reference-state values for
the selected observation; it is not a total resident-memory measurement.

Ordinary observation, transaction admission, and publication do not scan
unrelated branches or retained history. Historical reconstruction and
reclamation are explicit cold paths.

## Authority boundaries

- Foundational branch-reference values describe exact meaning but never own
  currentness, liveness, or movement.
- Proof carriers encode progression law, while Relational's concrete
  owner-sealed marker types decide which values can open governed operations.
- Signal owns its independent component basis and graph state.
- The Bridge may retain and coordinate owner-issued component bases, but it
  may not mint Relational authority or mutate a Relational branch cell.
- Query carries component or composite artifacts supplied by their owners; it
  does not become component history authority.

## Current limits

The owner catalog, immutable roots, branch cells, and retention accounting are
memory-resident. Restart durability for this owner model is deferred to Worth
Store integration.

Published-snapshot handles are bounded by
`publication.policy.max_published_snapshot_handles`. *Preparation* reserves one
handle for every candidate it admits, so an exhausted budget is refused there
rather than at publication: `prepare_branch_transaction` returns
`Err(TransactionCommitError::PublicationDeferred { .. })` carrying
`PublishedSnapshotCapacityExhausted { maximum_handles }`, and no candidate is
created. `compare_and_publish` cannot produce that variant, because every
candidate it can be handed already holds its handle. An exhausted handle budget
therefore never leaves a moved reference unsettled. The same bound limits the
pending-settlement registry; see
[`OWNER_COMPONENT_PORT.md`](./OWNER_COMPONENT_PORT.md#preinstalled-bounded-pending-settlement).

This milestone does not provide composite Relational plus Signal currentness,
cross-owner atomic publication, automatic rebase, semantic merge, distributed
movement, or offline synchronization.

For the semantic certification model and its reusable extension rules, see
[`TESTING_WORLDS.md`](./TESTING_WORLDS.md).
