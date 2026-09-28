# WORTH Query Runtime API Next Batch Closeout

## Status

The WORTH Query runtime API next batch is closed as of 2026-04-27 for the
runtime-backed facade, automatic live subscription installation, query-shaped
delivery, nested computed handles, effects, branch/preview reuse, intent
commits, inspection, migration, and shortcut-rejection scope described in
[runtime-api-next-batch-implementation-plan.md](./runtime-api-next-batch-implementation-plan.md).

This closeout covers the runtime-backed surface that consumes Milestones 9.1
through 9.3. It does not claim temporal/async query semantics from Milestones
9.4 through 9.7, store-backed execution parity from Milestone 10, durable
subscription replay from Milestone 11, or domain-specific DSL behavior.

## Governing Source Summary

- `MENTALITY.md`: the ideal developer experience was treated as the
  adversarial constraint. The implementation repaired runtime seams and added
  proof boundaries rather than weakening the facade into implementation-shaped
  helper calls.
- `arch_laws.md`: closure required facade-first access, proof-bearing phase
  chains, framework-owned resource lifecycle, authority/derivation separation,
  sealed construction, typed errors, and explicit boundary crossings.
- `perf_laws.md`: closure required delivery, subscription, computed, effect,
  branch, intent, and inspection work to stay counter-visible and bounded by
  declared semantic surfaces rather than raw CDC breadth or registered-view
  breadth.
- `domain_laws.md`: runtime facade, backend assembly, live installation,
  delivery, computed resources, effects, branch/preview isolation, intents,
  inspection, compatibility migration, and shortcut tests remain separate
  responsibilities rather than one runtime god file.
- `worth_query_vision.md`: the closed surface advances the promise that one
  typed query shape can become read, live, branch, preview, derived, effected,
  delivered, and inspected work without asking ordinary consumers to wire lower
  runtimes.
- `worth_query_roadmap.md`: the batch preserves the roadmap rule: declare query
  intent once, lower it once, execute it against canonical truth.
- `test-requirements.md`: the closeout evidence is certification-shaped:
  typed denials, compile-fail shortcut rejection, runtime support metadata,
  query-shaped delivery, branch/preview isolation, and full-crate regression
  runs.
- `milestone-9.1-closeout.md`: subscription declaration, bridge lowering,
  admission, and `SubscriptionActivationInput` are consumed as the live
  declaration proof chain.
- `milestone-9.2-closeout.md`: active lifecycle, sharing, query-shaped
  delivery, continuation, preview isolation, and lifecycle closeout are
  consumed as runtime facade internals, not exposed as ordinary app work.
- `milestone-9.3.md`: automatic subscription diagnostics, bridge parity,
  support reporting, and runtime family certification remain the explanation
  substrate that inspection and runtime support evidence build on.

## Adversarial Constraint Closed

An ordinary consumer can now declare and compose runtime facade handles without
manual subscription, bridge, signal, grouped-baseline, active-lane, or CDC
wiring, while the internal path remains proof-bearing and inspectable.

The closed runtime surface rejects the naive failure modes this batch was
designed to prevent:

- live views no longer rely on caller-installed subscription plumbing
- patch draining no longer treats raw CDC or affected-view guesses as the
  ordinary delivery contract
- nested computed resources are declared handles with dependency and
  authority-lane evidence rather than hidden callback routes
- effects are declaration-owned resources with phase, policy, condition,
  delivery, and write-intent evidence rather than ambient host callbacks
- branch and preview reuse binds handles to explicit basis and effect-policy
  evidence instead of mutating authoritative handles in place
- intents cross sealed authority-lane boundaries and cannot turn derived state
  into truth without admission
- inspection explains retained handle and receipt artifacts instead of
  re-running lowering or exposing mutable internals
- memory-backed scaffolds and demo consumers use the runtime facade without
  pretending memory is the architectural source of truth

## Shipped Scope

### Runtime Facade And Backend Assembly

Closed:

- `WORTHQueryRuntime` is the ordinary runtime facade.
- `WORTHQueryRuntimeBackend`, `WORTHQueryRuntimeBackendParts`, and
  `WORTHQueryBridgeBackedRuntimeBackend` form the runtime backend seam.
- `WORTHQueryRuntimeSupportProfile` and facade-family support metadata expose
  whether read, live, computed, effect, branch, preview, write, intent, and
  inspection families are admitted.
- Runtime builder assembly rejects missing backend parts and support overclaims
  with typed errors.
- Memory-backed assembly is explicitly scaffold posture, not primary
  architecture.

### Automatic Live Subscription Installation

Closed:

- runtime live declaration installs the subscription proof chain automatically
  through family selection, declaration, bridge lowering, admission,
  activation input, active lane opening, and consumer attachment.
- live handles retain query, live-shape, subscription family, declaration,
  bridge, admission, activation, basis, signal strategy, active lane, consumer
  attachment, support, counter, and diagnostic evidence for inspection.
- unsupported families, slices, grouped metadata, policy/basis drift, and
  missing backend support deny before activation.

### Query-Shaped Delivery And Grouped Baselines

Closed:

- runtime patch draining emits `QueryDeliveryBatch` /
  `WORTHQueryRuntimeDeliveryBatch` evidence instead of making app code consume
  raw lower-runtime events.
- grouped live declaration obtains backend-owned grouped baseline membership;
  ordinary facade users cannot inject grouped baselines.
- delivery counters and patch families are tied to semantic change surfaces,
  with refresh/gap paths remaining explicit evidence rather than hidden broad
  fallback.

### Nested Computed Handles

Closed:

- `WORTHQueryDerivedViewHandle` and maintained derived views support live and
  computed dependencies through explicit dependency indexes.
- nested computed ordering, irrelevant-update suppression, cycle denial, and
  branch/preview binding are covered by tests.
- computed outputs carry authority-lane evidence, including
  `DerivedRuntimeState`, so a produced aspect cannot be mistaken for committed
  truth.

### Effects And Conditional Nodes

Closed:

- effect declarations are handle resources with triggers, conditions, delivery
  classes, write-intent lowering, suppression, pending-intent delivery, and
  phase evidence.
- expression and condition failures produce typed effect failures without
  corrupting computed state.
- effect policy and authority-lane override construction is sealed from the
  public surface.

### Branch And Preview Isolation

Closed:

- branch and preview sessions can bind live, computed, effect, write, and
  intent declarations to explicit branch/preview basis evidence.
- preview and branch options expose named safe effect policies rather than raw
  authoritative-policy constructors.
- preview discard, promotion, residue, and typed effect-policy behavior are
  covered for live, computed, effect, write, and intent surfaces.
- preview and branch constructors return typed support denials when the backend
  does not admit the family, rather than panicking.

### Intent Commits

Closed:

- intent declarations are facade resources that lower through runtime admission
  into relational or bridge authority.
- direct writes, branch/preview intents, effect-triggered intents, invariant
  denials, idempotent no-ops, and loop-prevention evidence are covered.
- intent execution artifacts cannot be spent as runtime commit receipts, and
  derived-to-truth shortcuts are blocked by sealed source-lane boundaries.

### Inspection

Closed:

- runtime inspection now explains live handles, computed handles, effects,
  intent receipts, preview/branch sessions, and feedback phase graphs using
  retained artifacts and stable digests.
- inspection shows authority lanes, basis lanes, dependency contracts, effect
  policy, loop-prevention, idempotence, support, counters, and denial/debt
  posture without granting construction authority.

### Surface Hardening And Migration

Closed:

- WORTH UI's todo workspace uses the runtime facade over an explicit
  memory-backed scaffold and has read/live/write regression coverage.
- ambiguous `in_memory_collections` builder construction is gone; compile-fail
  fixtures now prove those builder methods are missing rather than merely
  discouraged.
- compile-fail fixtures reject raw CDC subscription, raw bridge activation,
  host observer callbacks, direct active-lane mutation, raw preview lane
  sharing, grouped baseline injection, raw authoritative preview policy,
  intent receipt bypass, and derived-to-truth shortcuts.

## Acceptance Mapping

This batch is considered closed against:

- [runtime-api-next-batch-implementation-plan.md](./runtime-api-next-batch-implementation-plan.md)
- [worth_query_vision.md](./worth_query_vision.md)
- [worth_query_roadmap.md](./worth_query_roadmap.md)
- [test-requirements.md](./test-requirements.md)
- [milestone-9.1-closeout.md](./milestone-9.1-closeout.md)
- [milestone-9.2-closeout.md](./milestone-9.2-closeout.md)
- [milestone-9.3.md](./milestone-9.3.md)

because the runtime facade now consumes the subscription, lifecycle,
diagnostic, bridge-parity, computed, effect, branch/preview, intent, and
inspection proof surfaces directly, while ordinary callers stay on the facade
path.

## Final QA Outcome

The final QA passes specifically corrected these closure risks:

- preview and branch constructors could panic on unsupported backends instead
  of returning typed denials
- missing backend messages still pointed users toward ambiguous memory
  construction rather than explicit scaffold construction
- public-looking effect and intent lane override paths needed stricter
  compile-fail proof
- grouped baseline helpers, intent execution artifacts, and derived-to-truth
  shortcuts needed explicit shortcut-rejection evidence
- WORTH UI's todo example needed to demonstrate the runtime facade path rather
  than depending on lower-level memory-workspace behavior
- the test suite needed one adversarial composed runtime surface proving live
  subscriptions, nested computeds, conditional pending-intent effects,
  authoritative/effect/branch intents, preview isolation, inspection, and phase
  feedback together instead of only in separate happy-path rows
- refresh-fallback computed inspection needed to prove actual fallback posture
  rather than exercising an incremental maintainer while naming fallback

After those corrections, I do not see a remaining meaningful gap for the
runtime-backed Batch 1 through Batch 9 scope. Remaining gaps are explicitly
later temporal/async, store-backed, durable, or product-domain work.

## Explicit Deferred Scope

This closeout does not claim:

- temporal query basis semantics from Milestone 9.4
- async/resource query families from Milestone 9.5
- mixed truth/time/async delivery from Milestone 9.6
- temporal/async certification closure from Milestone 9.7
- store-backed execution parity from Milestone 10
- durable saved-query, cursor, subscription, diagnostic, or continuation reload
  from Milestone 11
- blob/media-backed delivery from Milestone 12
- domain-specific DSL, geometry kernel, table, workflow, or UI semantics inside
  `worth-query`
- removal of low-level query/subscription modules needed for crate-internal
  certification and lower-runtime development

## What Later Work May Now Assume

Later work may safely assume:

- the runtime facade is the ordinary consumer path
- live declaration installs query subscriptions automatically
- runtime live handles carry inspectable subscription and active-lane evidence
- patch draining is query-shaped
- grouped baselines are backend-owned and not app-injected through the ordinary
  facade
- nested computeds and effects are declaration-owned resources
- branch and preview reuse bind declarations to explicit basis and safe effect
  policy evidence
- intent commits cross sealed authority-lane and receipt boundaries
- inspection is the debugging surface for generated DSL output and runtime
  facade behavior
- unsupported combinations fail typed and early rather than widening, falling
  back to raw CDC, or relying on host callbacks

Later work must not assume:

- temporal/async surfaces already exist
- store-backed or durable restart semantics are admitted
- memory-backed scaffolds are the source of architectural truth
- low-level subscription, active-lane, bridge, signal, or relational shortcuts
  are appropriate for ordinary app or DSL code
- derived runtime state may become authoritative truth without explicit intent
  or commit authority

## Verification Baseline

The closeout state is grounded in:

- `cargo fmt -p worth-query -p worth-ui`
- `cargo check -p worth-query --tests`
- `cargo test --manifest-path crates/worth-query/Cargo.toml --test phase_boundaries_compile_fail`
- `cargo test -p worth-query`
- `cargo test -p worth-ui`
- `git diff --check`

The final full `worth-query` test run passed 757 tests. The final `worth-ui`
test run passed. `git diff --check` passed with only existing CRLF
normalization warnings in the working tree.

## Operational Conclusion

The runtime API batch is now the normative runtime-backed public API foundation
for ordinary `worth-query` consumers.

The important closure is not that every future capability exists. The closure
is that the public shape is now defended: consumers compose facade handles, and
the runtime owns subscription installation, derived computation, effects,
branch/preview isolation, intents, delivery, inspection, support metadata, and
typed denial. When future temporal, async, store-backed, or durable work lands,
it must extend this proof-bearing facade rather than route around it.

The follow-on stabilization spec is
[runtime-api-public-stabilization-plan.md](./runtime-api-public-stabilization-plan.md).
It freezes the final ordinary public API vocabulary, golden DX transcript
tests, async-safe state model, support gates, and inspection contract before
Milestones 9.4 through 9.7 extend the surface with temporal and async/resource
semantics.
