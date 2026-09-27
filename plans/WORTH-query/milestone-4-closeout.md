# Milestone 4 Closeout: Collection Semantics, Ordering, Pagination, And Bounded Traversal

## Status

Milestone 4 is closed as of 2026-04-14 for the runtime-backed collection
semantics scope.

`worth-query` now has a real collection execution substrate layered on top of
the Milestone 3 planned/basis/execution boundary. Ordered collections, bounded
traversal/materialization, admitted aggregate and rollup families, admitted
derived-field semantics, and CDC-shaped result families are no longer host
helpers or delivery-time conventions. They are planner-owned, digest-bearing,
counter-explained, and certification-proven runtime-backed surfaces.

The semantic center shipped in this milestone is:

validated collection meaning lowers once into collection-owned planning
artifacts, ordering and cursor semantics stay basis-bound, traversal and
breadth stay explicit, result-family shaping stays planner-owned, execution
consumes those lowered plans without rediscovery, and the certification harness
proves parity, difference, and rejection through canonical machine-checkable
bundles.

## Shipped Scope

Milestone 4 delivered:

- planner-owned collection artifacts and collection digest authority in
  [crates/worth-query/src/collection](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/collection)
- collection-aware plan lowering in
  [crates/worth-query/src/planning](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/planning)
- runtime-backed collection execution envelopes and collection-specific
  counters in
  [crates/worth-query/src/execution](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/execution)
- collection-specific harness and acceptance artifacts under
  [crates/worth-query/src/harness/collection_certification](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/collection_certification)
  and
  [crates/worth-query/src/harness/collection_matrix](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/collection_matrix)
- shared fixture coverage for collection preflights under
  [crates/worth-query/src/harness/fixtures](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/fixtures)
- compile-fail proof-boundary tests for collection artifacts under
  [crates/worth-query/tests/ui](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui)

## Acceptance Mapping

Milestone 4 is considered closed against
[milestone-4.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/milestone-4.md),
[worth_query_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/worth_query_roadmap.md),
and
[test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/test-requirements.md)
because the required runtime-backed collection acceptance surfaces are now
covered directly.

### `Collection, Cursor, Rollup, And CDC Shape Parity Test`

Covered by:

- [collection_certification/mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/collection_certification/mod.rs)
- [collection_certification/tests.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/collection_certification/tests.rs)
- [collection_matrix](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/collection_matrix)

What is proven:

- the named certification artifact exists as a first-class closeout surface
- required canonical rows are present:
  - `ordered-collection-parity`
  - `cursor-advance-repeatability`
  - `bounded-traversal-parity`
  - `aggregate-rollup-parity`
  - `derived-field-parity`
  - `cdc-shaped-result-parity`
- required rejection rows are present:
  - `unsupported-ordering-family`
  - `unstable-cursor-shape`
  - `unsupported-traversal-bound`
  - `unsupported-aggregate-family`
  - `unsupported-cdc-result-family`
- `bundle_completeness_report` closes with no missing required rows or
  assertion classes
- the aggregate certification artifact is deterministic and offline-readable

### `Planner-owned collection semantics`

Covered by:

- `collection::CollectionPlanBundle`
- `planning::plan_validated_bundle`
- `planning::plan_validated_bundle_for_collection_family`
- collection harness coverage in
  [crates/worth-query/src/harness/planning.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/planning.rs)

What is proven:

- collection planning is no longer hidden in generic planning residue
- ordering direction changes alter both `CollectionPlanDigest` and `PlanDigest`
- collection result-family changes alter both collection and plan identity
- detail queries do not silently acquire collection planning artifacts
- collection breadth, traversal depth, and CDC-family selection are planner-owned

### `Stable cursor and bounded breadth honesty`

Covered by:

- `collection::OpaquePageCursor`
- `collection::CursorBoundaryDigest`
- `execution::ExecutionCounters`
- collection certification bundles and planning harness tests

What is proven:

- cursor progression is basis-bound and plan-bound rather than ambient offset
  state
- page width, cursor advancement, traversal breadth, aggregate input breadth,
  rollup breadth, and derived-field evaluation are explicit counter surfaces
- collection execution remains `executor_semantic_rediscovery_count == 0`
- unsupported ordering/cursor/traversal shapes fail typed and early

### `Admitted aggregate, rollup, derived-field, and CDC families`

Covered by:

- `RequestedAggregateFamily::CountRows`
- `RollupEdgeClass::RootCollection`
- `RequestedDerivedFieldFamily::DisplayLabel`
- `DerivedFieldComputationClass::DisplayLabelFromIdentityAndProfile`
- `CollectionResultFamily::{OrdinaryCollection, CdcCollection}`

What is proven:

- one admitted aggregate family exists as a planner-owned semantic family,
  not a host-local recomputation pass
- one admitted rollup class exists as an explicit planned shape
- one admitted derived-field family exists as an explicit planned shape
- CDC-shaped output remains a planned result family over canonical query meaning
- unsupported aggregate families still fail typed and early instead of falling
  back into approximate behavior

## Explicit Deferred Scope

Milestone 4 is closed for runtime-backed collection semantics only.

The following remain explicit later-milestone work, not implied completeness:

- live promotion and incremental maintenance
- historical and diff-aware collection replay
- durable cursor persistence and restart-stable continuation
- store-backed collection parity and store pushdown
- broader aggregate DSL coverage
- broader derived-field families

The current Milestone 4 surface is intentionally narrow in family count and
broad in semantic honesty.

## What Milestone 5 May Now Assume

Milestone 5 may safely assume:

- collection planning is a first-class extension of the Milestone 3 plan boundary
- runtime-backed ordered collection execution is canonical
- cursor advancement semantics are basis-bound and planner-owned
- bounded traversal/materialization breadth is explicit and counter-proven
- admitted aggregate, rollup, derived-field, and CDC result families already
  lower through one collection planning substrate
- the collection certification harness is the model for later live/history
  suites

Milestone 5 must not assume:

- durable cursor resume
- historical collection parity
- store-backed collection execution parity
- broad aggregate or derived-field family coverage beyond the admitted families

## Verification Baseline

Milestone 4 closeout was verified with:

- `cargo test --manifest-path crates/worth-query/Cargo.toml -q`

This passes cleanly and includes:

- unit and harness coverage for collection planning and execution
- collection certification artifact tests
- shared certification-core tests
- trybuild compile-fail tests for proof-boundary privacy

## Operational Conclusion

Milestone 4 is now closed at the runtime-backed collection semantics level.

`worth-query` no longer depends on host-local collection loops, offset-shaped
pagination claims, best-effort traversal breadth, aggregate/derived
recomputation in delivery glue, or CDC formatting hacks to make collection
reads work. It now has a planner-owned collection boundary, basis-bound cursor
proof, explicit breadth counters, admitted result-family semantics, and named
Milestone 4 acceptance evidence that Milestone 5 can build on safely.
