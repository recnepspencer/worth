# Milestone 5.3 Closeout: Frontier-Aware Planning And Deterministic Parallel Admission

## Status

Milestone 5.3 is closed as of 2026-04-16 for the runtime-backed
frontier-aware planning and deterministic parallel-admission scope.

`worth-query` now has a real frontier-planning substrate layered on top of the
Milestone 3 planning/basis boundary, the Milestone 4 collection/result-family
boundary, the Milestone 5 live-promotion boundary, and the Milestone 5.1
locality-bearing live boundary. Frontier posture, packet identity, bundle basis
proof, deterministic parallel admission, typed serial fallback, typed
parallel/serial executor entrypoints, route-level breadth accounting, parity
bundles, milestone-native certification, and a production-facing closeout
matrix are no longer executor folklore, host-authored hints, or harness-only
reconstruction. They are crate-owned, digest-bearing, counter-explained,
compile-time hardened, and certification-proven surfaces.

The semantic center shipped in this milestone is:

the same admitted canonical query meaning can lower once into a planner-owned
frontier posture, execute through explicit serial, parallel-admitted, or typed
serial-fallback routes without changing result meaning, and close with
machine-checkable parity, drift, and denial evidence instead of executor-side
heuristics or hidden fallback.

## Shipped Scope

Milestone 5.3 delivered:

- planner-owned frontier posture, packet identity, bundle basis proof, route
  posture, parity bundles, and counter snapshots in
  [crates/worth-query/src/frontier_planning](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/frontier_planning)
- lower-runtime frontier-evidence translation in
  [crates/worth-query/src/frontier_signal_adapter.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/frontier_signal_adapter.rs)
- route-typed planning wrappers and facade exposure in
  [crates/worth-query/src/planning/mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/planning/mod.rs)
  and
  [crates/worth-query/src/facade.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/facade.rs)
- route-typed execution entrypoints in
  [crates/worth-query/src/execution/mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/execution/mod.rs)
- milestone-native frontier certification artifacts, rejection rows, and
  closeout mapping under
  [crates/worth-query/src/harness/frontier_certification](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/frontier_certification)
- frontier lowering, bundle, signal-backed evidence, drift, and parity harness
  coverage under
  [crates/worth-query/src/harness/frontier_planning.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/frontier_planning.rs)
- compile-fail proof-boundary tests for route forging, wrong-entrypoint
  execution, and private frontier artifacts under
  [crates/worth-query/tests/ui](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui)

## Acceptance Mapping

Milestone 5.3 is considered closed against
[milestone-5.3.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/milestone-5.3.md),
[worth_query_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/worth_query_roadmap.md),
and
[test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/test-requirements.md)
because the required frontier-planning and deterministic parallel-admission
acceptance surfaces now exist directly.

### `Frontier Planning And Parallel Admission Parity Test`

Covered by:

- [frontier_certification/mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/frontier_certification/mod.rs)
- [frontier_certification/model.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/frontier_certification/model.rs)
- [frontier_certification/tests.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/frontier_certification/tests.rs)

What is proven:

- the named certification artifact exists as a first-class Milestone 5.3
  closeout surface
- required canonical rows are present:
  - `frontier-serial-control`
  - `parallel-admitted-parity`
  - `serial-fallback-parity`
  - `predicted-vs-realized-breadth`
  - `bundle-route-posture-parity`
  - `exact-basis-bundle-parity`
  - `work-avoided-counter-parity`
- required rejection rows are present:
  - `unsupported-frontier-family`
  - `unsupported-bundle-composition`
  - `mixed-basis-bundle-denied`
  - `forbidden-executor-speculative-admission`
  - `forbidden-hidden-serial-fallback`
  - `invalid-route-posture-override`
  - `forbidden-serial-route-on-parallel-entrypoint`
- serial, parallel-admitted, serial-fallback, serial-bundle, and
  parallel-bundle lanes all emit production-owned parity bundles rather than
  harness-local synthetic digests
- exact bundle counters are asserted for both parallel-admitted bundle posture
  and serial-fallback bundle posture
- `executor_parallel_rediscovery_count` closes at zero for admitted lanes
- the frontier certification artifact and closeout artifact are deterministic
  and offline-readable

### `Planner-owned frontier posture and packet identity`

Covered by:

- `frontier_planning::lower_preflight_to_frontier_plan`
- `frontier_planning::lower_live_plan_to_frontier_plan`
- `frontier_planning::FrontierAwarePlan`
- `frontier_planning::PlannedWorkPacket`
- `frontier_planning::PacketMergeBoundary`
- frontier lowering harness tests in
  [crates/worth-query/src/harness/frontier_planning.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/frontier_planning.rs)

What is proven:

- admitted execution preflights and admitted live plans lower through one
  frontier-planning boundary rather than a second planner
- planner-owned packet identity, packet digests, packet merge contracts, and
  route posture digests are derived from canonical query meaning and basis
  proof rather than executor chunking
- exact bundle basis proof is explicit, digest-bearing, and required for
  bundle lowering
- unsupported families and unsupported mixed preflight/live bundle
  compositions fail typed and early
- mixed-basis bundles fail typed and early even when the broader basis class
  would otherwise look similar

### `Deterministic parallel admission, typed serial fallback, and drift honesty`

Covered by:

- `frontier_planning::lower_preflight_to_parallel_admission_route`
- `frontier_planning::lower_preflight_to_serial_fallback_route`
- `frontier_planning::ParallelAdmissionRoute`
- `frontier_planning::SerialFallbackRoute`
- `frontier_planning::FrontierPredictionDriftOutcome`
- frontier route harness tests and compile-fail tests

What is proven:

- parallel admission is a proof-bearing route artifact, not a boolean hint
- serial fallback is a proof-bearing route artifact, not a hidden executor
  recovery path
- bounded materialization cannot be fed into the parallel-admission route
  surface and ordered collections cannot be fed into the serial-fallback route
  surface without explicit typed denial
- `SerialFallbackRequired` changes route posture explicitly rather than
  surviving as a passive counter-only event
- `DeniedByDrift` blocks executable route construction rather than silently
  admitting a successful lane
- the wrong route class cannot be executed through the wrong executor entry
  point

### `Performance truth is production-owned rather than harness-reconstructed`

Covered by:

- `frontier_planning::FrontierCounterSnapshot`
- `frontier_planning::FrontierParityBundle`
- `frontier_planning::FrontierRouteReport`
- `frontier_certification` artifact aggregation and tests

What is proven:

- 5.3-required counters are production-owned and exposed through route/parity
  artifacts, including:
  - predicted breadth
  - realized breadth
  - parallel route and batch counts
  - serial fallback plan and execution counts
  - bundle route counts
  - mixed-basis denial counts
  - packet merge width and reduction counts
  - prediction drift counts
  - work-avoided and work-preserved counts
- parity/certification code consumes those production artifacts rather than
  reverse-engineering the counters from lower-level execution data
- the closeout matrix now binds canonical and rejection proof explicitly,
  rather than hand-waving denial coverage

## Explicit Deferred Scope

Milestone 5.3 is closed for runtime-backed frontier-aware planning and
deterministic parallel-admission semantics only.

The following remain explicit later-milestone work, not implied completeness:

- preview-session basis lifecycle and preview-versus-promoted query semantics,
  which remain Milestone 5.2 authority
- structural correspondence and historical materialization-path metadata,
  which belong to Milestone 5.4
- query-authored workflow, mutation, merge, and writeback lowering, which
  belong to Milestone 5.5
- unified application facade and unified runtime configuration closure, which
  belong to Milestone 5.6
- store-backed frontier parity, store-backed historical parity, and durable
  route-posture continuation
- broader query-family admission beyond the currently admitted ordered
  collection, bounded materialization, admitted live, and admitted
  locality-bearing live families
- richer future frontier cost models beyond the current explicit predicted and
  realized breadth surfaces

The current Milestone 5.3 surface is intentionally narrow in admitted family
count and broad in route-posture honesty.

## What Milestones 5.4 Through 5.6 May Now Assume

Milestones 5.4 through 5.6 may safely assume:

- frontier posture is already a planner-owned boundary rather than executor
  folklore
- deterministic parallel admission and typed serial fallback already exist as
  proof-bearing query-route surfaces
- route-typed executor entrypoints already prevent serial/parallel path mixing
- bundle posture already exists for both parallel-admitted and serial-fallback
  lanes with exact basis proof
- predicted breadth, realized breadth, drift outcomes, bundle route counts,
  and work-avoided/work-preserved counters already exist as production-owned
  parity artifacts
- milestone-native frontier certification and a spec-to-code closeout matrix
  already exist as acceptance-proof surfaces

Milestones 5.4 through 5.6 must not assume:

- preview-session lifecycle semantics are part of 5.3 authority
- store-backed frontier parity
- durable route-posture replay or restart-stable continuation
- broader frontier admission beyond the currently admitted family matrix
- any executor-side speculative admission, hidden fallback, or host-authored
  parallel safety claim

## Verification Baseline

Milestone 5.3 closeout was verified with:

- `cargo test --manifest-path crates/worth-query/Cargo.toml`

This passes cleanly and includes:

- unit and harness coverage for frontier lowering, signal-backed evidence,
  bundle posture, drift outcomes, and route parity
- milestone-native frontier certification artifact and closeout artifact tests
- shared certification-core tests
- trybuild compile-fail tests for route forging, wrong-entrypoint execution,
  private frontier artifacts, and signal-adapter privacy boundaries

## Operational Conclusion

Milestone 5.3 is now closed at the runtime-backed frontier-planning and
deterministic parallel-admission level.

`worth-query` no longer depends on executor-local frontier rediscovery,
host-authored parallel safety hints, implicit serial fallback, executor-defined
packetization, or harness-only parity reconstruction to make frontier-aware
bulk/live planning work. It now has a planner-owned frontier posture boundary,
proof-bearing parallel and serial route classes, exact-basis bundle posture,
typed drift outcomes, production-owned counter and parity artifacts,
compile-time route-boundary hardening, and named Milestone 5.3 acceptance
evidence that Milestones 5.4, 5.5, and 5.6 can build on safely.
