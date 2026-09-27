# Milestone 5 Closeout: Live Query Promotion, Query-Shaped Patches, And Convergence

## Status

Milestone 5 is closed as of 2026-04-15 for the runtime-backed live-promotion
scope.

`worth-query` now has a real live query substrate layered on top of the
Milestone 3 planning/basis boundary and the Milestone 4 collection semantics
boundary. Live promotion, relevance classification, query-shaped live patches,
suppression, replay progression, replay parity, refresh/coalescing policy, and
milestone-native certification are no longer host glue or subscription folklore.
They are crate-owned, digest-bearing, counter-explained, compile-time hardened,
and certification-proven runtime-backed surfaces.

The semantic center shipped in this milestone is:

the same admitted canonical query can execute once or promote into live mode
without changing query meaning, live maintenance stays query-shaped instead of
event-shaped, irrelevant changes suppress before consumer delivery, replay and
fresh execution compare through machine-checkable artifacts, and forbidden live
paths fail typed and early instead of degrading into raw CDC or hidden refresh.

## Shipped Scope

Milestone 5 delivered:

- live promotion, live execution, replay, patch envelopes, replay bundles, and
  live policy counters in
  [crates/worth-query/src/live](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/live)
- live performance contracts, width budgets, coalescing admission, and refresh
  status vocabulary in
  [crates/worth-query/src/live_performance](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/live_performance)
- planning-to-live lowering through `ExecutionPlanBundle` in
  [crates/worth-query/src/planning](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/planning)
- milestone-native live certification artifacts and matrix coverage under
  [crates/worth-query/src/harness/live_certification](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/live_certification)
- live fixtures and hostile preflights under
  [crates/worth-query/src/harness/fixtures](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/fixtures)
- compile-fail proof-boundary tests for live artifacts and forbidden raw-CDC
  live payload leakage under
  [crates/worth-query/tests/ui](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui)

## Acceptance Mapping

Milestone 5 is considered closed against
[milestone-5.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/milestone-5.md),
[worth_query_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/worth_query_roadmap.md),
and
[test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/test-requirements.md)
because the required runtime-backed live-promotion acceptance surface now
exists directly.

### `Live Promotion Convergence And Suppression Test`

Covered by:

- [live_certification/mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/live_certification/mod.rs)
- [live_certification/tests.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/live_certification/tests.rs)
- [live_certification/model.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/live_certification/model.rs)

What is proven:

- the named certification artifact exists as a first-class Milestone 5 closeout
  surface
- required canonical rows are present:
  - `detail-live-convergence`
  - `ordered-collection-live-convergence`
  - `bounded-materialization-live-convergence`
  - `irrelevant-update-suppression`
  - `refresh-fallback-equivalence`
  - `coalesced-sequence-replay-parity`
  - `patch-width-budget-overflow-policy`
  - `work-avoided-counter-parity`
- required rejection rows are present:
  - `unsupported-live-family`
  - `unsupported-patch-family`
  - `raw-cdc-leakage-forbidden`
  - `invalid-live-basis-promotion`
  - `forbidden-refresh-escape-hatch`
  - `non-monotonic-change-sequence`
  - `forbidden-coalescing-class`
  - `forbidden-width-budget-overflow-behavior`
- replay-specific parity is no longer forced through generic equality:
  - end-state replay parity is explicit
  - stepwise replay parity is explicit
- the live certification artifact is deterministic and offline-readable

### `Live promotion as a plan-derived proof boundary`

Covered by:

- `live::promote_preflight_bundle_to_live`
- `planning::ExecutionPlanBundle::live_promotion`
- live promotion tests in
  [crates/worth-query/src/live/mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/live/mod.rs)

What is proven:

- live plans are derived from already-admitted one-shot plans rather than a
  second query language
- start basis, progress basis, change-sequence identity, replay digest, and
  subscription digest are explicit artifacts rather than ambient runtime state
- non-admitted promotion inputs fail typed and early:
  - store-backed preflights are rejected
  - CDC-shaped collection families are rejected
- live proof-bearing artifacts stay externally unconstructable through private
  fields and compile-fail coverage

### `Query-shaped live maintenance rather than raw event interpretation`

Covered by:

- `live::execute_live_change`
- `live::LivePatchEnvelope`
- `live::LivePatchPayload`
- live family outcome tests and compile-fail tests

What is proven:

- admitted live families produce query-shaped payloads for:
  - detail reads
  - ordered collections
  - bounded materialization
- no raw CDC variant exists on the public live patch payload surface
- family mismatch and raw-CDC leakage are both explicit rejection paths rather
  than soft conventions
- live patch identity is semantic/query-shaped instead of subscription-scoped

### `Replay, suppression, and policy honesty`

Covered by:

- `live::replay_live_sequence`
- `live::LiveProgressBasis`
- `live::RefreshAdmissionMatrix`
- `live_performance::*`
- live certification rows and replay tests

What is proven:

- replay progression is explicit and monotonic within one change sequence
- gapful sequences and backward sequences are distinguished and counted
- irrelevant changes suppress before delivery
- refresh fallback and coalescing remain explicit policy outcomes instead of
  hidden implementation shortcuts
- fresh execution and replay compare through `query_digest`, `result_digest`,
  `delivery_digest`, `replay_digest`, and `counter_snapshot`

## Explicit Deferred Scope

Milestone 5 is closed for runtime-backed live-promotion semantics only.

The following remain explicit later-milestone work, not implied completeness:

- branch-scoped, historical, and diff-aware live parity
- live + policy masking parity
- durable subscription resume, persisted checkpoints, and restart-stable live
  continuation
- store-backed live replay and store-backed live execution parity
- broader suppression classes beyond the currently admitted and certified lanes
- broader non-detail/non-collection live-family coverage

The current Milestone 5 surface is intentionally narrow in admitted family
count and broad in semantic honesty.

## What Milestone 6 May Now Assume

Milestone 6 may safely assume:

- live promotion is a first-class extension of the one-shot plan boundary
- runtime-backed live execution is canonical for admitted detail, ordered
  collection, and bounded materialization families
- replay progression, replay parity, and query-shaped live artifacts already
  exist as proof-bearing types
- refresh/coalescing/width policy is explicit and counter-proven rather than
  transport folklore
- raw CDC is structurally excluded from the live consumer contract
- the live certification harness is the model for later historical and
  policy-composed suites

Milestone 6 must not assume:

- durable restart-stable live continuation
- store-backed live parity
- historical or diff-aware live semantics
- policy masking integrated into the live lane

## Verification Baseline

Milestone 5 closeout was verified with:

- `cargo test --manifest-path crates/worth-query/Cargo.toml -q`

This passes cleanly and includes:

- unit and harness coverage for live promotion, live patching, replay, and
  policy
- milestone-native live certification artifact tests
- shared certification-core tests
- trybuild compile-fail tests for live proof-boundary privacy and no-raw-CDC
  live payload leakage

## Operational Conclusion

Milestone 5 is now closed at the runtime-backed live-promotion level.

`worth-query` no longer depends on subscription-only builders, host-local
relevance heuristics, client-side collection repair, raw CDC disguised as live
query output, or hidden refresh behavior to make live reads work. It now has a
plan-derived live boundary, query-shaped patch families, explicit replay-safe
progression, policy-owned suppression and fallback, milestone-native
certification, and named Milestone 5 acceptance evidence that Milestone 6 can
build on safely.
