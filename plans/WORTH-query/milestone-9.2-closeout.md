# Milestone 9.2 Closeout: Subscription-Family-Backed Active Lifecycle, Sharing, Delivery, Continuation, And Preview Isolation

## Status

Milestone 9.2 is closed as of 2026-04-23 for the runtime-backed active query
subscription surface in `worth-query`.

This closeout covers:

- activation only from admitted `SubscriptionActivationInput`
- registry-owned active lane lifecycle and equivalent sharing
- consumer-local attachment, pacing, acknowledgement, and backpressure state
- query-shaped delivery windows, lowered maintenance deltas, patch groups, and
  delivery receipts
- typed continuation/remap
- preview isolation, discard residue proof, and promotion handoff
- hot-path performance encoding through typed widths, lookup posture, density
  posture, allocation posture, work packets, and performance receipts
- lifecycle closeout support and shipped lifecycle certification, including
  preview/support closure

This closeout does not claim durable checkpoint continuation, store-backed
restart survival, runtime replay, or richer automatic subscription diagnostics.
Those remain later-milestone work exactly as Milestone 9.2 specified.

## Governing Source Summary

- `MENTALITY.md`: the milestone is closed only where lifecycle truth is
  mechanically enforced, not merely narrated.
- `arch_laws.md`: closure required proof-bearing lifecycle phases, sealed
  construction, framework-owned resources, phase-typed delivery artifacts, and
  self-describing certification output.
- `perf_laws.md`: closure required explicit hot-path width types, posture
  encoding, performance receipts, and exact counters rather than benchmark-only
  claims.
- `domain_laws.md`: closure required lifecycle, attachment, delivery,
  continuation, preview, closeout, support, and certification to remain
  separated responsibilities rather than collapsing into a runtime bag.
- `milestone-9.2.md`: the shipped/runtime surface and the milestone harness now
  both close against the adversarial constraint and closeout standard.

## Shipped Scope

Milestone 9.2 delivered:

- active lane admission, registry ownership, handles, sharing, budgets, lookup
  posture, lifecycle posture, delivery posture, and counters in
  [crates/worth-query/src/subscription/active.rs](../../workspaces/worth-query/crates/worth-query/src/subscription/active.rs),
  [crates/worth-query/src/subscription/active_lane.rs](../../workspaces/worth-query/crates/worth-query/src/subscription/active_lane.rs),
  [crates/worth-query/src/subscription/active_registry.rs](../../workspaces/worth-query/crates/worth-query/src/subscription/active_registry.rs),
  [crates/worth-query/src/subscription/active_runtime.rs](../../workspaces/worth-query/crates/worth-query/src/subscription/active_runtime.rs), and
  [crates/worth-query/src/subscription/active_budget.rs](../../workspaces/worth-query/crates/worth-query/src/subscription/active_budget.rs)
- consumer attachment, acknowledgement frontier, backpressure, fanout, and
  delivery cursor semantics in
  [crates/worth-query/src/subscription/attachment.rs](../../workspaces/worth-query/crates/worth-query/src/subscription/attachment.rs),
  [crates/worth-query/src/subscription/acknowledgement.rs](../../workspaces/worth-query/crates/worth-query/src/subscription/acknowledgement.rs), and
  [crates/worth-query/src/subscription/fanout.rs](../../workspaces/worth-query/crates/worth-query/src/subscription/fanout.rs)
- query-shaped delivery windows, maintenance deltas, work packets, batches,
  patch groups, and raw CDC / raw bridge invalidation denials in
  [crates/worth-query/src/subscription/delivery_window.rs](../../workspaces/worth-query/crates/worth-query/src/subscription/delivery_window.rs),
  [crates/worth-query/src/subscription/maintenance_delta.rs](../../workspaces/worth-query/crates/worth-query/src/subscription/maintenance_delta.rs),
  [crates/worth-query/src/subscription/delivery_work_packet.rs](../../workspaces/worth-query/crates/worth-query/src/subscription/delivery_work_packet.rs), and
  [crates/worth-query/src/subscription/patch_group.rs](../../workspaces/worth-query/crates/worth-query/src/subscription/patch_group.rs)
- typed continuation/remap evidence and report lowering in
  `crates/worth-query/src/subscription/continuation.rs`
- preview isolation, discard closeout, promotion handoff, residue classes, and
  residue reports in
  [crates/worth-query/src/subscription/preview_isolation.rs](../../workspaces/worth-query/crates/worth-query/src/subscription/preview_isolation.rs)
- lifecycle closeout support and terminal runtime closure in
  [crates/worth-query/src/subscription/closeout.rs](../../workspaces/worth-query/crates/worth-query/src/subscription/closeout.rs) and
  `crates/worth-query/src/subscription/support.rs`
- phase-local performance receipts in
  [crates/worth-query/src/subscription/performance_receipt.rs](../../workspaces/worth-query/crates/worth-query/src/subscription/performance_receipt.rs)
- shipped lifecycle certification, including preview/support closure, in
  [crates/worth-query/src/subscription/certification.rs](../../workspaces/worth-query/crates/worth-query/src/subscription/certification.rs)
- milestone certification in
  [crates/worth-query/src/harness/milestone_nine_two_certification](../../workspaces/worth-query/crates/worth-query/src/harness/milestone_nine_two_certification)
- compile-fail phase boundaries in
  `crates/worth-query/tests/ui`

## Acceptance Mapping

Milestone 9.2 is considered closed against:

- [milestone-9.2.md](./milestone-9.2.md)
- [worth_query_roadmap.md](./worth_query_roadmap.md)
- [worth_query_vision.md](./worth_query_vision.md)
- [test-requirements.md](./test-requirements.md)
- [milestone-9.1-closeout.md](./milestone-9.1-closeout.md)

because the runtime-backed active lifecycle boundary now exists directly and is
certified through both a shipped proof-bearing API and a milestone harness that
consumes that shipped certification surface for lifecycle rows.

### `Subscription Lifecycle Sharing And Preview Parity Test`

Covered by:

- [crates/worth-query/src/harness/milestone_nine_two_certification/mod.rs](../../workspaces/worth-query/crates/worth-query/src/harness/milestone_nine_two_certification/mod.rs)
- `crates/worth-query/src/harness/milestone_nine_two_certification/builders.rs`
- [crates/worth-query/src/harness/milestone_nine_two_certification/tests.rs](../../workspaces/worth-query/crates/worth-query/src/harness/milestone_nine_two_certification/tests.rs)
- [crates/worth-query/src/harness/certification/requirements.rs](../../workspaces/worth-query/crates/worth-query/src/harness/certification/requirements.rs)

What is proven:

- the named certification suite exists as
  `Subscription Lifecycle Sharing And Preview Parity Test`
- required canonical rows are present for active lifecycle delivery,
  equivalent-lane sharing, grouped query-shaped delivery, continuation remap,
  preview discard, preview promotion, posture-sensitive performance receipts,
  and width-bounded scale evidence
- required rejection rows are present for masked sharing, raw CDC fallback,
  raw bridge invalidation fallback, preview-authoritative sharing, preview
  discard residue denial, dense refresh denial, and store-backed restart denial
- rows prove equality, inequality, typed failure, and zero-residue assertion
  classes
- preview discard and promotion rows now certify preview/support closure
  through the shipped lifecycle certification surface rather than harness-local
  post-hoc digest assembly
- required verification outputs are present, including lifecycle, delivery,
  preview, counter, scale, compile-fail, and support digests

### `Shipped lifecycle certification`

Covered by:

- [crates/worth-query/src/subscription/certification.rs](../../workspaces/worth-query/crates/worth-query/src/subscription/certification.rs)
- `crates/worth-query/src/subscription/tests/certification.rs`

What is proven:

- shipped lifecycle certification binds admission, activation, scale, active
  lane admission, lane handle, consumer attachment, delivery window, lowered
  maintenance delta, delivery work packet, delivery batch, acknowledgement
  frontier, optional continuation, optional preview evidence, and lifecycle
  closeout on one coherent runtime-backed source
- preview discard certification requires aligned isolation, residue, discard
  closeout, and preview lifecycle closeout
- preview promotion certification requires aligned isolation, residue,
  promotion handoff, and preview lifecycle closeout
- support closure is part of shipped certification output via
  `support_matrix_digest`
- external code cannot mint the admitted lifecycle context constructor
  directly; proof-bearing context remains crate-owned

### `Closeout Standard`

The Milestone 9.2 closeout standard in
[milestone-9.2.md](./milestone-9.2.md) is satisfied because:

- active lifecycle starts only from admitted `SubscriptionActivationInput`
- active lane identity preserves declaration, equivalence, basis, policy,
  tenant, relationship proof, view shape, bridge declaration, and signal
  strategy meaning
- equivalent subscriptions can share one maintenance lane while retaining
  separate consumer-local attachment and acknowledgement state
- active delivery emits query-shaped batches for detail and collection/grouped
  families
- hot-path performance is structurally encoded through typed widths, lookup
  class, density posture, allocation posture, work packets, and receipts
- continuation/remap is typed, patch-visible, and denial-aware
- preview subscriptions are isolated; discard proves zero authoritative
  residue; promotion crosses an explicit authority boundary
- durable checkpoint and store-backed restart claims remain explicit typed debt

## Verification Baseline

The closeout state is verified by:

- `cargo fmt -p worth-query`
- `cargo test -p worth-query subscription::tests::certification`
- `cargo test -p worth-query harness::milestone_nine_two_certification`
- `cargo test -p worth-query --test phase_boundaries_compile_fail`
- `cargo test -p worth-query`
- `git diff --check -- crates/worth-query/src/subscription crates/worth-query/src/harness/milestone_nine_two_certification crates/worth-query/tests/ui memory/2026-04-23.md plans/worth-query/milestone-9.2-closeout.md`

## Deferred Scope That Remains Explicitly Deferred

The following are not part of Milestone 9.2 closeout:

- automatic subscription diagnostics expansion
- automatic subscription path inspection
- richer bridge-parity explanation
- durable continuation checkpoints
- store-backed restart-stable active subscriptions
- store-backed replay or checkpoint-plus-tail subscription recovery

These remain 9.3 / 10 / 11 work exactly as the milestone spec declared.
