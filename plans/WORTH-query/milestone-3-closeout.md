# Milestone 3 Closeout: Query Planning, Snapshot-Bound Execution, And Binding Parity

## Status

Milestone 3 is closed as of 2026-04-14 for the runtime-backed one-shot
execution scope.

`worth-query` now has a real proof-bearing execution substrate between
validated query meaning and later collection/live/history work. Planning,
basis resolution, binding fulfillment, execution, and certification are no
longer implied by happy-path reads or host glue. They are explicit crate-owned
artifacts with sealed boundaries, deterministic digests, exact counters, and
named certification evidence.

The semantic center shipped in this milestone is:

validated query meaning lowers once into planner-owned execution artifacts,
binding fulfillment resolves once through query-owned descriptors, basis
identity resolves once into an explicit snapshot proof, execution runs once
against that admitted basis, and the certification harness proves parity,
difference, and rejection using canonical machine-checkable bundles rather than
ad hoc test logic.

## Shipped Scope

Milestone 3 delivered:

- planner-owned artifacts and lowering in
  [crates/worth-query/src/planning](../../workspaces/worth-query/crates/worth-query/src/planning)
- explicit basis intent, basis resolution, and preflight coupling in
  [crates/worth-query/src/basis](../../workspaces/worth-query/crates/worth-query/src/basis)
- query-owned binding fulfillment and resolution in
  [crates/worth-query/src/binding](../../workspaces/worth-query/crates/worth-query/src/binding)
- runtime-backed execution envelopes and exact execution counters in
  [crates/worth-query/src/execution](../../workspaces/worth-query/crates/worth-query/src/execution)
- plan/basis/binding/result digest authority in
  [crates/worth-query/src/identity](../../workspaces/worth-query/crates/worth-query/src/identity)
- compile-fail proof-boundary tests for planned, basis, and execution
  artifacts under
  `crates/worth-query/tests/ui`
- a shared certification core and requirements registry under
  [crates/worth-query/src/harness/certification](../../workspaces/worth-query/crates/worth-query/src/harness/certification)
- phase-aligned fixture layers under
  [crates/worth-query/src/harness/fixtures](../../workspaces/worth-query/crates/worth-query/src/harness/fixtures)
- the named Milestone 3 certification surface under
  [crates/worth-query/src/harness/planning_certification](../../workspaces/worth-query/crates/worth-query/src/harness/planning_certification)
  and
  [crates/worth-query/src/harness/planning_matrix](../../workspaces/worth-query/crates/worth-query/src/harness/planning_matrix)

## Acceptance Mapping

Milestone 3 is considered closed against
[milestone-3.md](./milestone-3.md),
[worth_query_roadmap.md](./WORTH_query_roadmap.md),
and
[test-requirements.md](./test-requirements.md)
because the required runtime-backed acceptance surfaces are now covered
directly.

### `Planner / Executor / Binding Parity Test`

Covered by:

- [planning_certification/mod.rs](../../workspaces/worth-query/crates/worth-query/src/harness/planning_certification/mod.rs)
- [planning_certification/tests.rs](../../workspaces/worth-query/crates/worth-query/src/harness/planning_certification/tests.rs)
- [planning_matrix](../../workspaces/worth-query/crates/worth-query/src/harness/planning_matrix)

What is proven:

- the named certification artifact exists as a first-class closeout surface
- required canonical rows are present:
  - `direct-runtime-plan-parity`
  - `replanned-runtime-parity`
  - `type-bound-runtime-parity`
  - `runtime-basis-repeatability`
  - `identity-bearing-binding-difference`
  - `basis-difference`
  - `route-semantic-difference`
- required rejection rows are present:
  - `unsupported-backend-route`
  - `unsupported-fallback-shape`
  - `binding-fulfillment-conflict`
  - `snapshot-basis-resolution-failure`
- `bundle_completeness_report` now derives from the shared requirements
  registry and closes with no missing required rows or assertion classes
- the aggregate certification artifact is deterministic and offline-readable

### `Planner-owned runtime route distinction`

Covered by:

- `planning::plan_validated_bundle`
- `harness::planning::traversal_bearing_runtime_queries_lower_to_expanded_runtime_route`
- `route-semantic-difference` in the Milestone 3 certification suite

What is proven:

- runtime-backed planning now admits two semantic runtime plan shapes:
  narrow snapshot reads and expanded snapshot reads
- route selection is planner-owned and digest-bearing rather than test-only
  metadata
- traversal/predicate/ordering-bearing validated queries lower onto the
  expanded runtime route
- intentionally different admitted runtime route semantics change certification
  outputs without pretending store support exists

### `Snapshot-bound execution and basis honesty`

Covered by:

- `basis::resolve_snapshot_basis`
- `basis::preflight_execution_basis`
- `execution::execute_preflight_bundle`
- harness tests in
  `crates/worth-query/src/harness/planning.rs`

What is proven:

- execution runs only after basis intent resolves into an admitted snapshot
  basis
- plan/basis compatibility is checked before execution starts
- execution envelopes carry `plan_digest`, `basis_digest`, and `result_digest`
- basis resolution and execution counters are exact and test-asserted
- executor semantic rediscovery remains exactly zero on admitted paths

### `One semantic ingress for direct and bound invocation`

Covered by:

- `planning_request_context_for_direct`
- `planning_request_context_for_bound`
- `BindingRequirements`, `BoundBindings`, and `BindingResolution`
- `type-bound-runtime-parity` and `identity-bearing-binding-difference`

What is proven:

- bound invocation does not invent a second planner authority path
- direct and bound execution converge through one `PlanningRequestContext`
  model
- identity-bearing binding changes alter plan semantics explicitly
- missing/conflicting binding fulfillment fails typed and early

### `Harness closure before Milestone 4`

Covered by:

- shared certification grammar under
  [harness/certification](../../workspaces/worth-query/crates/worth-query/src/harness/certification)
- Milestone 1, 2, and 3 completeness reports deriving from the shared
  requirements registry
- shared phase fixtures under
  [harness/fixtures](../../workspaces/worth-query/crates/worth-query/src/harness/fixtures)

What is proven:

- the crate now has one endorsed certification grammar instead of milestone-
  local mini-frameworks
- required row coverage and required assertion-class coverage are mechanically
  checked for Milestones 1-3
- Milestone 4 can build on the shared fixture/certification architecture
  instead of inventing a third harness style

## Explicit Store-Gated Debt

Milestone 3 is closed for runtime-backed one-shot execution only.

The following remain explicit debt, not implied completeness:

- store-backed execution parity
- store-backed basis equivalence and pushdown
- durable snapshot-plus-tail restore semantics

Unsupported store requests fail typed and early today. That is the honest
Milestone 3 state.

## What Milestone 4 May Now Assume

Milestone 4 may safely assume:

- the planned artifact boundary is frozen
- runtime-backed one-shot execution is canonical
- basis identity is explicit before reads execute
- direct and bound invocation share one planner-owned semantic ingress
- the certification harness architecture is the model for future milestones

Milestone 4 must not assume:

- admitted store parity
- durable cursor or saved-query semantics
- live maintenance, history, or diff behavior

## Verification Baseline

Milestone 3 closeout was verified with:

- `cargo test --manifest-path crates/worth-query/Cargo.toml -q`

This passes cleanly and includes:

- unit and harness coverage for planning, basis, binding, and execution
- the shared certification-core tests
- phase-layer fixture tests
- Milestone 1, 2, and 3 certification artifact tests
- trybuild compile-fail tests for proof-boundary privacy

## Operational Conclusion

Milestone 3 is now closed at the runtime-backed planning and one-shot
execution level.

`worth-query` no longer depends on ambient truth reads, host-local binding
semantics, planner/executor rediscovery, or milestone-local certification
ceremony to make one-shot execution honest. It now has a frozen planned
artifact boundary, explicit basis proof, runtime-backed execution envelope,
shared certification architecture, and named Milestone 3 acceptance evidence
that Milestone 4 can build on safely.
