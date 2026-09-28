# Milestone 8 Closeout: Scale Query, Parallel Read, and Bulk Mutation Completion

## Status

Milestone 8 is closed as of 2026-03-30.

The runtime now treats scale query, immutable parallel read, and bulk mutation
as truth-grade architectural surfaces rather than as convenience APIs layered
on top of authority-era foundations.

The semantic center shipped in this milestone is:

read/query and bulk mutation now pass through proof-bearing planning,
admission, deterministic reduction, typed fallback, workload-derived
instrumentation, and certification-grade tracing, while preserving serialized
authority, lineage, provenance, persistent naming, and replay/durability
honesty under hostile scheduling and hostile index conditions.

This is not "queries got faster."

The runtime now owns:

- proof-bearing query planning as the only supported growth path
- snapshot-bound immutable read admission
- reducer-owned deterministic bulk read execution
- staged-parallel immutable read execution with canonical reduction
- typed index admissibility and typed storage fallback
- bounded deterministic sampled parity
- first-class accelerated query families for entity and relation payload-field
  equality and `AnyOf`
- bulk mutation planning with explicit naming, lineage, provenance, and
  locality artifacts
- proof-widened bulk mutation admission before serialized authority
- workload-derived, deterministic read-side telemetry
- memory-hardened fragment and index scratch reuse with runtime-lifetime
  cleanup
- a hard break from the legacy packet/result fallback growth surfaces

## Shipped Scope

Milestone 8 delivered:

- proof-bearing query planning through:
  - `QueryPlanContextId`
  - `QueryScope`
  - `QueryLocalityClass`
  - `QueryOrderingContract`
  - `QueryFallbackContract`
  - `PlannedQueryPacket`
  - `SnapshotPinnedQueryPlan`
- deterministic query execution through:
  - `QueryWorkerFragment`
  - `QueryFragmentCounters`
  - `CanonicalQueryResult`
  - `QueryComplexitySummary`
  - `QueryExecutionOutcome`
- reducer-owned canonical read behavior for:
  - explicit-target reads
  - entity kind scans
  - relation kind scans
  - aspect-filtered entity scans
  - aspect-filtered relation scans
  - outgoing neighborhood traversal
  - incoming neighborhood traversal
  - bounded connectivity traversal
- immutable staged-parallel read execution with:
  - packetized partitioned read work
  - deterministic fragment identities
  - traversal visit-key reduction
  - scheduler-independent observable result order
- true typed index/fallback surfaces through:
  - `IndexQueryRejectionClass`
  - `QueryAccessPath`
  - `FallbackParityMode`
  - `FallbackParityVerifiedQueryOutcome`
- real accelerated query lanes for:
  - `EntityPayloadFieldEquals`
  - `EntityPayloadFieldAnyOf`
  - `RelationPayloadFieldEquals`
  - `RelationPayloadFieldAnyOf`
- bounded deterministic `SampledParity` selection derived from stable workload
  identity rather than thread or scheduler state
- bulk mutation planning through:
  - `BulkMutationScope`
  - `BulkMutationLocalityFootprint`
  - `BulkMutationNamingPlan`
  - `BulkMutationLineagePlan`
  - `BulkMutationProvenancePlan`
  - `PlannedBulkMutationBatch`
- proof-widened bulk mutation admission through:
  - `NamingStableBulkMutationBatch`
  - `LineageSafeBulkMutationBatch`
  - `ProvenanceCompleteBulkMutationBatch`
- commit-boundary tracing for proof-admission failures
- workload-derived query scratch reuse counters and runtime-scoped index scratch
  reuse counters
- runtime-lifetime cleanup for index scratch hint state
- complexity contracts and proof tests for the shipped scale lanes
- hostile certification over query, parity, publication, fintech, and
  concurrency carriers

Before closeout, the implementation also removed or sealed the remaining paths
that would have weakened milestone honesty if left in place:

- `QueryWorkPacket` was removed rather than left as a compatibility-growth bag
- `PacketResult` was removed from the growth path
- `IndexedReadOutcome` was removed
- `RelationalReadView::execute_packet` was removed from the supported growth
  path
- `IndexAccess::read_with_storage_fallback(...)` was removed from the supported
  growth path
- recovered-runtime tests and harness helpers were migrated onto
  `PlannedQueryPacket` so proof context cannot be smuggled across runtime
  boundaries

## Phase Completion Map

Milestone 8 is considered closed because each phase now has a shipped
implementation surface, verification basis, and no open architectural blocker.

### Phase 1: Query Packet Modeling Upgrade

Closed by:

- [query/data/mod.rs](../../crates/worth-relational/src/query/data/mod.rs)
- [facade.rs](../../crates/worth-relational/src/facade.rs)
- [planning.rs](../../crates/worth-relational/src/tests/query/planning.rs)

What is proven:

- query ingress is proof-bearing instead of bag-shaped
- context, locality, ordering, fallback, and plan identity are first-class
- explicit-target construction no longer relies on a weak compatibility packet

### Phase 2: Snapshot Admission and Planning Proof

Closed by:

- [reader.rs](../../crates/worth-relational/src/visibility/materialization/read_records/reader/mod.rs)
- `history/logic/access.rs`
- [planning.rs](../../crates/worth-relational/src/tests/query/planning.rs)

What is proven:

- query plans bind to exact runtime, snapshot, version, schema, and descriptor
  semantics context
- WORTHd or mismatched planning context is rejected
- genesis and historical schema evidence are explicit rather than silent
  fallback

### Phase 3: Reducer-Based Query Execution

Closed by:

- [reader.rs](../../crates/worth-relational/src/visibility/materialization/read_records/reader/mod.rs)
- [mod.rs](../../crates/worth-relational/src/query/data/mod.rs)
- [execution.rs](../../crates/worth-relational/src/tests/query/execution/mod.rs)
- [concurrency.rs](../../crates/worth-relational/src/tests/query/concurrency.rs)

What is proven:

- canonical result order is reducer-owned
- traversal reduction is deterministic under overlap and future parallel fanout
- immutable reads match serial truth under staged-parallel execution
- pinned reads remain snapshot-stable under hot rewrite pressure

### Phase 4: True Index/Fallback Architecture

Closed by:

- `access.rs`
- [indexes.rs](../../crates/worth-relational/src/tests/query/indexes/mod.rs)
- [visibility_budgets.rs](../../crates/worth-relational/src/tests/complexity/contracts/visibility_budgets/mod.rs)

What is proven:

- index use is explicit, typed, and rejectable
- storage remains authoritative under all rejection conditions
- certification parity and sampled parity are distinct architectural modes
- accelerated query lanes preserve canonical storage parity

### Phase 5: Bulk Mutation Planning

Closed by:

- [primitives.rs](../../crates/worth-relational/src/transactions/data/primitives.rs)
- `mod.rs`
- [mod.rs](../../crates/worth-relational/src/tests/transactions/core/mod.rs)

What is proven:

- bulk mutation locality, naming, lineage, and provenance are planned before
  execution
- topology-heavy mutation does not flatten identity semantics into counts
- persistent naming participates in planning rather than cleanup

### Phase 6: Bulk Mutation Admission Proof Chain

Closed by:

- [primitives.rs](../../crates/worth-relational/src/transactions/data/primitives.rs)
- `mod.rs`
- [pipeline.rs](../../crates/worth-relational/src/authority/commit/pipeline/mod.rs)

What is proven:

- scale mutation cannot reach authority without naming, lineage, and
  provenance admission
- admission failures are traced at the commit boundary
- admission is semantically pure until commit time

### Phase 7: Locality, Memory, and Counter Honesty

Closed by:

- [reader.rs](../../crates/worth-relational/src/visibility/materialization/read_records/reader/mod.rs)
- `access.rs`
- [mod.rs](../../crates/worth-relational/src/performance/data/mod.rs)
- [visibility_budgets.rs](../../crates/worth-relational/src/tests/complexity/contracts/visibility_budgets/mod.rs)

What is proven:

- query breadth, packet shape, and emissions are measured honestly
- scratch reuse telemetry is workload-derived rather than scheduler-shaped
- index scratch state is runtime-scoped and runtime-lifetime-cleaned
- hot read paths avoid hidden full snapshot materialization for the shipped
  accelerated lanes

### Phase 8: Parallel Read Expansion and Accelerated Query Family Completion

Closed by:

- [reader.rs](../../crates/worth-relational/src/visibility/materialization/read_records/reader/mod.rs)
- `access.rs`
- [execution.rs](../../crates/worth-relational/src/tests/query/execution/mod.rs)
- [indexes.rs](../../crates/worth-relational/src/tests/query/indexes/mod.rs)
- [concurrency.rs](../../crates/worth-relational/src/tests/query/concurrency.rs)

What is proven:

- staged-parallel immutable reads preserve deterministic result order
- traversal remains canonical under overlap and worker-count variation
- accelerated entity and relation payload-field families preserve parity,
  branch scoping, partition bounding, and recovery stability

### Phase 9: Certification Closure

Closed by:

- [planning.rs](../../crates/worth-relational/src/tests/query/planning.rs)
- [execution.rs](../../crates/worth-relational/src/tests/query/execution/mod.rs)
- [concurrency.rs](../../crates/worth-relational/src/tests/query/concurrency.rs)
- [indexes.rs](../../crates/worth-relational/src/tests/query/indexes/mod.rs)
- [observability.rs](../../crates/worth-relational/src/tests/publication/observability/mod.rs)
- [visibility_budgets.rs](../../crates/worth-relational/src/tests/complexity/contracts/visibility_budgets/mod.rs)
- [commit_budgets.rs](../../crates/worth-relational/src/tests/complexity/contracts/commit_budgets/mod.rs)
- [workflows](../../crates/worth-relational/src/tests/domains/fintech/workflows)

What is proven:

- bulk query and traversal stress preserve truth
- index non-authority is fail-closed under drift, corruption, and
  incompatibility
- observable diagnostics remain deterministic under hostile scheduling
- pinned concurrent reads remain stable under hot rewrite pressure

## Verification Evidence

Milestone 8 closeout verification was completed against:

- full `worth-relational` test matrix
- compile-fail phase-boundary suite
- UI boundary suite
- hostile query certification lanes
- hostile publication/observability lanes
- hostile fintech carrier workflows
- complexity contracts for query and bulk-mutation hot paths

Final closeout verification included:

```text
cargo test -p worth-relational tests::query::execution -- --nocapture
cargo test -p worth-relational tests::query::indexes -- --nocapture
cargo test -p worth-relational tests::complexity::contracts::visibility_budgets -- --nocapture
cargo test -p worth-relational tests::query::concurrency -- --nocapture
cargo test -p worth-relational tests::publication::observability -- --nocapture
cargo test -p worth-relational
```

All passed at closeout.

## Hard-Break Surface Cleanup

Milestone 8 intentionally took a hard API break rather than preserving
pre-production compatibility debt.

Closed removals:

- `QueryWorkPacket`
- `PacketResult`
- `IndexedReadOutcome`
- legacy fallback wrapper growth surfaces

Outcome:

- there is no remaining public packet/result fallback growth path competing
  with `PlannedQueryPacket`
- harness and certification helpers now use the same proof-bearing query path
  as production code

## Residual Debt

No known Milestone 8 blocking debt is carried at closeout.

Non-blocking note:

- the deeper engineering rationale, historical critique, and design evolution
  remain in
  [milestone-8-plan.md](./milestone-8-plan.md)
  as the companion engineering spec

## Summary

Milestone 8 closed the scale contract of `worth-relational`.

The runtime now has:

- proof-bearing read/query planning
- deterministic immutable parallel reads
- typed, honest index acceleration
- first-class bulk mutation planning and admission
- workload-derived tracing and counter honesty
- hard-broken legacy query growth surfaces

This milestone is therefore closed not because more APIs exist, but because the
read and bulk-mutation side of the runtime now meets the same truth-grade bar
already expected of authority, replay, durability, lineage, and merge.
