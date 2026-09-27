# Milestone 5.1 Closeout: Region-Scoped Live Narrowing And Stream-Contract Delivery

## Status

Milestone 5.1 is closed as of 2026-04-16 for the locality-aware live
hardening scope defined in
[milestone-5.1.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/milestone-5.1.md).

`worth-query` now has a real region-scoped extension of the Milestone 5 live
substrate. Locality-bearing live admission, region and partition slice
matching, off-region suppression, explicit widening admission and denial,
query-shaped stream-contract lowering, replay-safe counter-bearing artifacts,
and milestone-native certification are no longer host glue or optimistic
runtime folklore. They are crate-owned, digest-bearing, compile-time hardened,
and certification-proven surfaces.

The semantic center shipped in this milestone is:

an admitted Milestone 5 live plan can attach a planner-owned locality contract,
classify lower-runtime changes as in-region, off-region, widening-admitted, or
widening-denied, suppress irrelevant churn before visible delivery, and lower
the same query-shaped meaning into a formal bridge stream contract without
silently broadening to aspect-level invalidation, raw partition events, or
transport-local consumer glue.

## Shipped Scope

Milestone 5.1 delivered:

- region-scoped live planning, locality admission, locality-aware execution,
  widening policy, stream-contract lowering, and replay-bearing delivery
  artifacts in
  [crates/worth-query/src/live](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/live)
- explicit region-scoped lifecycle decomposition in
  [crates/worth-query/src/live/region_scoped.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/live/region_scoped.rs)
  rather than leaving Milestone 5.1 responsibilities collapsed into one live
  mega-module
- locality and stream proof artifacts including `RegionScopedLivePlan`,
  `LocalityAwareRelevanceContract`, `StreamLoweredDeliveryContract`,
  `RegionScopedReplayBundle`, `QueryDeliveryContract`,
  `DeliveryContractLowering`, `StreamMemberProjection`,
  `StreamWindowCompatibility`, and `DeliveryContractReplayRecord` in
  [crates/worth-query/src/live/mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/live/mod.rs)
- milestone-native region-live certification artifacts, row catalogs, and
  scenario coverage under
  [crates/worth-query/src/harness/region_live_certification](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/region_live_certification)
- typed certification taxonomy shared through
  [crates/worth-query/src/harness/live_certification/model.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/live_certification/model.rs)
- compile-fail proof-boundary tests for locality-bearing and stream-lowered
  artifacts under
  [crates/worth-query/tests/ui](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui)

## Acceptance Mapping

Milestone 5.1 is considered closed against
[milestone-5.1.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/milestone-5.1.md),
[worth_query_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/worth_query_roadmap.md),
and
[test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/test-requirements.md)
because the required locality-aware live and stream-contract acceptance surface
now exists directly.

### `Region-Scoped Live Narrowing And Stream Contract Test`

Covered by:

- [region_live_certification/mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/region_live_certification/mod.rs)
- [region_live_certification/row_catalog.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/region_live_certification/row_catalog.rs)
- [region_live_certification/tests.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/region_live_certification/tests.rs)

What is proven:

- the named certification artifact exists as a first-class Milestone 5.1
  closeout surface
- required canonical rows are present and asserted through a shared catalog:
  - `region-live-convergence`
  - `off-region-suppression-parity`
  - `broad-vs-region-narrowing-control`
  - `stream-contract-delivery-parity`
  - `locality-breadth-budget-enforcement`
  - `stream-member-width-budget-enforcement`
  - `locality-work-avoided-counter-parity`
- representative concrete lanes are present:
  - `detail-region-hit`
  - `detail-off-region-suppressed`
  - `collection-partition-hit`
  - `collection-cross-partition-denied`
  - `bounded-materialization-region-hit`
  - `cdc-stream-lowered-parity`
  - `raw-stream-member-forbidden`
- required hostile rows are present and typed:
  - `unsupported-locality-family`
  - `unsupported-locality-predicate`
  - `unsupported-stream-consumer-contract`
  - `raw-partition-event-leakage-forbidden`
  - `raw-stream-member-leakage-forbidden`
  - `raw-stream-member-forbidden`
  - `forbidden-locality-widening`
  - `forbidden-broad-success-lane`
  - `forbidden-stream-width-overflow-success`
  - `forbidden-stream-window-overflow-success`
  - `bridge-slice-incompatibility-denied`
- exact counters, family identity, outcome kind, digest relationships, and
  typed failure taxonomy are all asserted rather than inferred from row names
- the certification artifact is deterministic and offline-readable

### `Locality-bearing live plans as a plan-derived proof boundary`

Covered by:

- `live::admit_region_scoped_live_plan`
- `live::RegionScopedLivePlan`
- `live::LocalityAwareRelevanceContract`
- region-scoped tests in
  [crates/worth-query/src/live/mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/live/mod.rs)
  and
  [crates/worth-query/src/live/region_scoped.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/live/region_scoped.rs)

What is proven:

- locality-bearing live plans are derived from admitted Milestone 5 live plans
  rather than a second live runtime or alternate subscription builder
- locality semantics are planner-owned through explicit artifacts such as
  `LocalitySemanticBasis`, `LocalityScopeAdmission`,
  `LocalityAwareRelevanceContract`, `LocalityWideningPolicy`, and
  `RegionScopedPlanningReport`
- region- and partition-aware maintenance consumes bridge-admitted slice
  categories rather than host-authored topic routing or callback filtering
- non-admitted locality families, predicates, and bridge slice pairings fail
  typed and early
- proof-bearing locality artifacts remain externally unconstructable through
  private fields and compile-fail coverage

### `Locality-sensitive execution, suppression, and widening honesty`

Covered by:

- `live::execute_region_scoped_live_change`
- `live::RegionScopedExecutionReport`
- `live::RegionScopedLiveCounters`
- `live::LocalityWideningDecision`

What is proven:

- locality-sensitive changes are classified explicitly as:
  - in-region
  - off-region suppressed
  - widening-admitted
  - widening-denied
- off-region churn suppresses before visible delivery and increments exact
  locality counters
- admitted widening is planner-owned, narrow, typed, and counted
- denied widening remains an explicit rejection path rather than hidden broad
  refresh or soft fallback
- locality breadth, widening budgets, and work-avoided counters are carried as
  first-class proof surfaces instead of telemetry-only afterthoughts

### `Query-shaped stream-contract lowering rather than raw stream leakage`

Covered by:

- `live::lower_region_scoped_execution_to_stream_contract`
- `live::StreamLoweredDeliveryContract`
- `live::QueryDeliveryContract`
- `live::DeliveryContractLowering`
- `live::StreamMemberProjection`
- `live::StreamWindowCompatibility`

What is proven:

- query-shaped delivery can be emitted directly or lowered into a bridge stream
  contract without changing query meaning
- raw partition events and raw stream members are not public query delivery
  contracts
- stream admission, denial, member-width budgets, and window-width budgets are
  explicit and exact
- stream-lowered delivery artifacts own their own counter snapshot instead of
  relying on harness-side reconstruction
- unsupported consumer-shape or stream-width pairings fail typed and early

### `Replay, parity, and counter ownership`

Covered by:

- `live::RegionScopedReplayBundle`
- `live::DeliveryContractReplayRecord`
- region-live replay and parity tests
- counter assertions in
  [crates/worth-query/src/harness/region_live_certification/tests.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/region_live_certification/tests.rs)

What is proven:

- region-aware replay bundles carry `query_digest`, `delivery_digest`,
  `replay_digest`, and `counter_snapshot`
- replay compares region-scoped live evolution both against fresh re-execution
  and against the broader aspect control lane where both are admitted
- required locality and stream counters now belong to the shipped proof
  artifacts rather than existing only as harness-side observations
- `locality_executor_rediscovery_count` remains zero on admitted paths

## Explicit Deferred Scope

Milestone 5.1 is closed for runtime-backed region-scoped live narrowing and
stream-contract delivery semantics only.

The following remain explicit later-milestone work, not implied completeness:

- durable stream continuation, persisted checkpoints, and restart-stable stream
  resume
- store-backed locality-aware live parity or store-backed stream continuation
- preview-session composition, speculative branch semantics, or policy masking
  as a legality source
- broader locality predicate families beyond the currently admitted region and
  partition vocabulary
- broader stream consumer shapes beyond the closed admitted matrix
- historical, diff-aware, lineage-aware, or correspondence-aware live parity

The current Milestone 5.1 surface is intentionally narrow in admitted family
count and broad in semantic honesty.

## What Milestone 5.2 May Now Assume

Milestone 5.2 and later milestones may safely assume:

- locality-bearing narrowing is a first-class extension of the Milestone 5
  live plan boundary
- admitted region- and partition-scoped live families execute through
  query-owned locality contracts rather than host-local heuristics
- query-shaped delivery and stream-lowered delivery are aligned but distinct
  proof-bearing surfaces
- widening, suppression, replay parity, and counter ownership are explicit and
  certification-proven
- the region-live certification harness is the model for later preview and
  policy-composed suites

Later milestones must not assume:

- durable restart-stable stream continuation
- arbitrary user-authored locality languages
- broader locality-admitted family coverage than the named 5.1 matrix
- stream lowering as permission to expose raw bridge events as query contracts

## Verification Baseline

Milestone 5.1 closeout was verified with:

- `cargo test --manifest-path crates/worth-query/Cargo.toml live::tests -- --nocapture`
- `cargo test --manifest-path crates/worth-query/Cargo.toml region_live -- --nocapture`
- `cargo test --manifest-path crates/worth-query/Cargo.toml --test phase_boundaries_compile_fail -- query_phase_boundaries_are_compile_time_private --nocapture`
- `cargo test --manifest-path crates/worth-query/Cargo.toml -q`

This passes cleanly and includes:

- unit and harness coverage for region-scoped live planning, execution,
  widening, stream lowering, and replay
- milestone-native region-live certification artifact tests
- trybuild compile-fail tests for locality-bearing and stream-lowered proof
  boundary privacy
- full-crate verification after the final region-scoped decomposition

## Operational Conclusion

Milestone 5.1 is now closed at the runtime-backed region-scoped live narrowing
and stream-contract delivery level.

`worth-query` no longer depends on broad aspect invalidation by default, host
topic heuristics, raw partition events, raw stream members, hidden widening, or
decorative stream vocabulary to make locality-aware live delivery work. It now
has planner-owned locality admission, bridge-compatible slice matching, typed
widening policy, query-shaped stream-contract lowering, replay-safe
counter-bearing artifacts, compile-time hardened proof boundaries, and named
Milestone 5.1 acceptance evidence that later milestones can build on safely.
