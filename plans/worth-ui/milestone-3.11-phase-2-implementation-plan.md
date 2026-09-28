# WORTH UI 3.11 Phase 2 Implementation Plan

> Historical QA policy (2026-08-22): proof, closure, migration, acceptance,
> and phase ledgers described below are frozen historical records. They are not
> active implementation or release gates, are not updated or reopened, and a
> ledger-only failure does not block current work. Current evidence follows
> [the QA review guide](../../docs/coding-guidelines/qa_review_guide.md) and
> [testing laws](../../docs/coding-guidelines/testing_laws.md): specifications state QA
> considerations in prose, tests and repository checks run against the current
> commit, and code review decides whether the evidence is adequate. This note
> does not retire product-domain ledgers that are part of runtime behavior.

## Closure Claim

An explicitly admitted visual request pins one exact retained mounted
presentation, crosses an authority-checked asynchronous host-capture boundary,
returns an epoch-bound coordinate observation and the requested typed pixel
posture, and releases every capture-owned resource without claiming semantic
point resolution.

Phase 2 does not build spatial identity indexes, trace declarations, adjudicate
points, or publish overlays. It establishes the retained and mechanical basis
those later phases must consume.

## Authority and Truth Owners

- The mounted presentation receipt owns the runtime binding between
  presentation attempt, frame, semantic surface, host surface, binding
  generation, and the host-issued presentation epoch.
- Mounted retention owns bounded snapshot and overlay pin classes. It does not
  own pixel payloads or inspection meaning.
- The runtime visual-snapshot domain owns request admission, registered capture
  state, exact-frame pinning, host-observation validation, pixel retention,
  disposal, and receipt sealing.
- The host contract owns mechanical request, pending/completed observation,
  transform, realized-region, epoch, and byte-payload types.
- The egui adapter owns its presentation epochs, screenshot request
  correlation, screenshot-event translation, client/viewport transform, and
  mechanical pixel copying. It never mints runtime receipts or semantic
  identity.
- The public facade owns the linear begin/poll/cancel/dispose workflow.

## Batch 2A — Epoch and Host Authority Seam

1. Make every successful mounted-surface completion carry a host-issued
   `UiHostPresentationEpoch`.
2. Retain that epoch in the completed mounted presentation receipt together
   with attempt, frame, semantic surface, host surface, and binding.
3. Preserve the completed presentation receipt in mounted retained-frame truth.
4. Add an authority-checked visual-capture operation to
   `WorthUiOperationalHostAdapter` and `UiHostAdapterSessionAuthority`.
5. Add an explicit mechanical `Pending` posture so asynchronous adapters do
   not manufacture synchronous completion.
6. Reject request/session/surface/binding/presentation affinity before adapter
   effects where the runtime or adapter already has the necessary fact.

Gate: deterministic host-contract tests prove exact-epoch completion,
superseded-before-readback, indeterminate affinity, and foreign request
rejection. Existing presentation tests remain green.

## Batch 2B — Admission, Pinning, and Registered Linear Lifecycle

1. Carry one application-declared `UiVisualInspectionPolicy` from builder to
   prepared application and seal it into the active session.
2. Scope concrete geometry, pixel, and overlay grants to the session, audience,
   admitted surfaces, byte limits, retention limits, and expiry posture.
3. Derive current and retained surface targets only from live mounted
   inspection receipts with an unambiguous completed surface presentation.
4. Replace `FutureSnapshot` with explicit `VisualSnapshot` and `VisualOverlay`
   mounted retention classes and independent budgets/usages.
5. Pin the exact retained frame and completed presentation before host request.
6. Register each capture in session-owned state. Begin, poll, cancel, timeout,
   dispose, and shutdown consume one handle and return the only legal successor.
7. Enforce one in-flight capture per host surface, capture-count, structural
   bytes, retained pixel bytes, deadline, disclosure, and foreign-session
   admission.
8. Seal geometry-only, optional-pixel, and required-pixel outcomes without
   exposing an impossible accessor.

Gate: public compile twins and runtime integration tests prove authority
separation, target provenance, linearity, before/after-host cancellation,
deadline split, capacity, historical pixel omission, and complete disposal.

## Batch 2C — Real Egui Readback and Mechanical Validation

1. Advance per-surface egui presentation epochs only on successful
   presentations and retain exact prepared mechanics per epoch.
2. Submit `ViewportCommand::Screenshot` with an opaque request correlation and
   return mechanical `Pending`.
3. Consume only matching `Event::Screenshot` observations; copy pixels while
   checking the requested epoch remains current.
4. Return `SupersededBeforeReadback` if the epoch advances before copying, and
   `CaptureAffinityIndeterminate` with no pixels when the copy epoch cannot be
   proved.
5. Report nonzero client origin, physical dimensions, logical viewport,
   fractional scale, orientation, and pixel-center rounding.
6. Validate request identity, attempt, frame, surface, binding, epoch,
   transform, region rows, pixel dimensions, stride, and byte limit before
   sealing runtime evidence.
7. Account pixel bytes requested/transferred/retained, coordinate transforms,
   leases, and retained structure. An ordinary frame without a capture request
   keeps all visual-snapshot counters zero.

Gate: the real egui adapter passes the ordinary exact-epoch contract and
fractional-transform/required-pixel cases. Deterministic schedules cover the
uncontrollable replacement races.

## Scenario Evidence

- VS-02: consolidated `application_contracts::visual_snapshot` deterministic
  schedules plus the same exact-epoch contract against `WorthUiHostEgui`.
- VS-05 capture/transform: two-surface rejection and selected-surface
  isolation in deterministic integration; real egui fractional scale and
  nonzero client origin.
- VS-06 capture lifecycle: the existing mounted retention state-machine owner
  plus consolidated begin/poll/cancel/timeout/capacity/dispose/shutdown cases.
- VS-07 runtime authority/protocol: the existing two-session compile owner and
  consolidated foreign-session, disclosure, expiry, and protocol cases.

## Required Final Evidence

- focused host-contract, runtime visual-snapshot, mounted-retention, egui
  capture, compile-contract, and application-contract tests;
- full WORTH UI topology and application contract targets;
- `cargo fmt --check`;
- workspace clippy with warnings denied;
- dirty function scrutiny and the Rust 400-line guard;
- boundary-check and agent-context checks; and
- a machine-audited Phase 2 proof ledger with no `OPEN` row.

## QA Re-plan After Batch 2B

The first Batch 2B implementation established policy carriage, concrete grant
types, exact presentation targets, bounded capture registration, cancellation,
timeout, disposal, and shutdown invalidation. QA found four remaining honesty
gaps that must close before Phase 2 can be claimed:

1. The named internal capture phases exist, but the production handle still
   carries the admitted request directly. The real pipeline must consume
   `admitted -> pinned -> host-requested -> host-observed -> validated`
   authority; a type-only test over unused aliases is not evidence.
2. `VisualOverlay` has an independent budget and report row, but mounted
   retention does not yet admit a lease through that class. The common
   visual-lease admission owner must accept a sealed snapshot/overlay class and
   return the matching typed lease.
3. The host observation's realized region rows are discarded. Phase 2 must
   retain the exact presented static-paint mechanical basis, validate every
   host row against frame, surface, binding, mounted receipt, bounds, clip,
   order, and participation, then carry the validated rows forward. Phase 3
   still owns index construction and semantic adjudication.
4. The egui adapter still reports `Unsupported`. It must use the real egui
   `ViewportCommand::Screenshot(UserData)` / `Event::Screenshot` lifecycle,
   correlate one exact runtime request, check the binding's presentation epoch
   immediately before copying, and return no usable bytes on superseded or
   indeterminate affinity.

### Re-planned Batch 2B.1 — Production Typestate and Retention

1. Replace the pending handle's stored admitted request with a private enum
   whose variants own either a pinned capture or a requested-host capture.
2. Make pinning consume admission and own the exact presentation basis,
   mounted snapshot lease, registry lease, and retained mechanical-row basis.
3. Make first poll consume the pinned phase into a requested-host phase. A
   subsequent host observation consumes that requested phase; validation
   consumes the observation; receipt sealing accepts only the validated phase.
4. Remove any unused phase aliases or tests that prove only nominal
   distinctness.
5. Make mounted visual retention admission generic over sealed snapshot and
   overlay class markers. Each marker fixes its enum class at compile time, and
   tests must admit and release both independent budgets.
6. Make registry admission atomic: compute count and byte successors before
   inserting; completion must prove actual retained bytes do not exceed the
   reservation.
7. Extend the native-shell shutdown receipt with visual pending, retained, and
   pixel-byte disposition rather than dropping the runtime report.

Gate: warnings-denied runtime tests, compile contracts, retention state-machine
tests, and consolidated lifecycle scenarios. P2-06 through P2-10 and P2-18 may
close only from these results.

### Re-planned Batch 2C.1 — Real Egui Capture State Machine

1. Retain, per registered binding, the exact successful presentation epoch and
   translated static-paint mechanics used for native paint.
2. On the first exact request, reject foreign registration/binding/session
   before effects, reject a non-current epoch as superseded, register one
   opaque egui screenshot correlation, send one real screenshot command, and
   return `Pending`.
3. On later polls, accept only the matching screenshot event. Check the
   requested binding epoch again immediately before copying the event image.
4. Convert the image once into owned top-down unmultiplied RGBA bytes within
   the request byte budget. A changed epoch returns
   `SupersededBeforeReadback`; an unprovable viewport or correlation returns
   `CaptureAffinityIndeterminate`; neither returns pixels.
5. Observe client origin from the current viewport inner rectangle, physical
   dimensions from the screenshot image, logical dimensions and fractional
   scale from egui input, and explicitly report top-left orientation and
   pixel-center-nearest rounding.
6. Cancellation removes the registered correlation. Because the egui command
   has already crossed the backend boundary, its honest posture is
   `ReadbackMayHaveBegun`; later unmatched events are drained by ordinary egui
   input and never re-enter a runtime capture.

Gate: focused real-adapter tests exercise command emission, exact correlated
event completion, wrong correlation, fractional transform, epoch advance,
capacity, cancellation, and zero request/zero command behavior.

### Re-planned Batch 2C.2 — Runtime Validation and Outcomes

1. Retain exact per-surface static-paint mechanics with the mounted frame
   retention evidence. Capture pinning snapshots those immutable rows before a
   replacement can advance current state.
2. Validate finite positive transform dimensions and scale, client/logical
   dimension coherence under the declared rounding posture, request identity,
   copy epoch, pixel dimensions, stride, byte length, byte budget, color-space
   posture, and every realized region row.
3. Preserve distinct geometry-only, optional-pixel, required-pixel, omitted,
   unsupported, superseded, denied, deadline-before-effect,
   timeout-after-effect, and indeterminate outcomes. A pixel-capable host that
   cannot prove exact epoch returns affinity indeterminate, not generic
   unsupported.
4. Add a transfer counter between requested and retained pixel bytes. Snapshot
   cost must expose requested, transferred, retained, transforms, leases, and
   retained structure; the ordinary frame receipt remains the all-zero
   projection.
5. Add deterministic predecessor-copy/successor-publish and
   successor-before-copy schedules, two-surface rejection/selection,
   transform/pixel/region mismatch matrices, and required/optional/geometry
   outcome matrices.

Gate: VS-02, capture/transform VS-05, capture-lifecycle VS-06, and runtime
authority/protocol VS-07 records name their exact world, boundary, schedule,
typed result, independent oracle, mutation control, cleanup, structural cost,
and cost lane. P2-11 through P2-19 remain open until those records and tests
exist.

### Ledger QA Rule

The ledger is not a progress narrative. A row changes to `PROVED` only when its
evidence cell names a passing test or compile contract, the tested world
actually crosses the claimed production boundary, the oracle is independent
of the result under test, and cleanup/cost consequences are asserted where the
claim includes them. Phase 2 remains incomplete while any row is `OPEN`; Phase
3 work may not begin merely because the remaining work appears adjacent.
