# Declaration Boundary Envelopes

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Declaration boundary envelopes are the public Query-owned artifacts that carry
one complete declaration crossing story forward.

They are receipt-backed only on the public lane. Query does not build them
directly from declarations, legality evidence, foundational evidence, or route
plans alone.

An envelope composes retained truth that already exists:

- retained declaration truth from foundational evidence
- retained route truth from route planning
- retained crossing posture from declaration boundary receipts

This keeps the public story honest:

- foundational evidence still owns retained declaration description
- route plans still own lower-authority participation planning
- receipts still own crossing posture
- envelopes own the one self-describing public artifact over those retained
  truths

## Why You Use It

- hand downstream callers one public artifact instead of making them reassemble
  evidence, route, and receipt truth
- preserve route-denial and receipt-denial topology on non-success paths
- keep admitted-world identity attached to the public crossing story
- get one envelope digest that converges when retained truth matches and
  diverges when world, route posture, receipt posture, or denial topology
  changes
- keep later inspection and recovery work from reopening route or receipt
  reasoning

## Stable Entry Points

- `WorthQueryDeclarationEnvelopeInput`
- `WorthQueryDeclarationEnvelope`
- `WorthQueryDeclarationEnvelopeChecked`
- `WorthQueryDeclarationEnvelopeDeferred`
- `WorthQueryDeclarationEnvelopeDenied`
- `WorthQueryDeclarationEnvelopeFailed`
- `WorthQueryDeclarationEnvelopeTerminalError`
- `WorthQueryDeclarationEntryEnvelopeError`
- `WorthQueryDeclarationEnvelopeClass`
- `WorthQueryDeclarationEnvelopeEvidenceOrigin`
- `WorthQueryDeclarationEnvelopeExplanation`
- `WorthQueryInstalledDomainDeclarationContext::envelope_routes(...)`
- `WorthQueryInstalledDomainDeclarationContext::envelope_routes_checked(...)`
- `WorthQueryInstalledDomainDeclarationContext::envelope_routes_from_progressed(...)`
- `WorthQueryInstalledDomainDeclarationContext::envelope_routes_from_progressed_with_intent(...)`
- `WorthQueryInstalledDomainDeclarationContext::envelope_routes_from_progressed(...)`
- `WorthQueryInstalledDomainDeclarationContext::envelope_routes_from_progressed_with_intent(...)`
- `WorthQueryInstalledDomainDeclarationContext::declare_review_progress_describe_plan_receipt_and_envelope(...)`

Good to know:

- the public envelope lane starts from receipt truth only
- deferred, denied, and failed receipt posture still produce Query-owned
  envelope artifacts
- envelopes preserve foundational evidence origin even though the ordinary lane
  does not begin from foundational evidence directly
- envelopes are Query-owned artifacts; they are not a cloned foundational
  envelope surface

## API Reference

Envelope input constructors:

- `WorthQueryDeclarationEnvelopeInput::issued(receipt)`
- `WorthQueryDeclarationEnvelopeInput::deferred(receipt)`
- `WorthQueryDeclarationEnvelopeInput::denied(receipt)`
- `WorthQueryDeclarationEnvelopeInput::failed(receipt)`
- `WorthQueryDeclarationEnvelopeInput::receipt_checked(checked)`

Admitted-handle envelope entry points:

- `envelope_routes(subject) -> Result<WorthQueryDeclarationEnvelope<D, I>, WorthQueryDeclarationEnvelopeTerminalError<D, I>>`
- `envelope_routes_checked(subject) -> WorthQueryDeclarationEnvelopeChecked<D, I>`
- `envelope_routes_from_progressed(progressed) -> Result<WorthQueryDeclarationEnvelope<D, I>, WorthQueryDeclarationEnvelopeTerminalError<D, I>>`
- `envelope_routes_from_progressed_with_intent(progressed, intent) -> Result<WorthQueryDeclarationEnvelope<D, I>, WorthQueryDeclarationEnvelopeTerminalError<D, I>>`
- `envelope_routes_from_progressed(progressed) -> Result<WorthQueryDeclarationEnvelope<D, I>, WorthQueryDeclarationEnvelopeTerminalError<D, I>>`
- `envelope_routes_from_progressed_with_intent(progressed, intent) -> Result<WorthQueryDeclarationEnvelope<D, I>, WorthQueryDeclarationEnvelopeTerminalError<D, I>>`
- `bind_continuation_request_from_context(request) -> WorthQueryBindingOutcome<WorthQueryContinuationBindingInput<D, I>>`
- `bind_continuation_request_from_context_checked(request) -> WorthQueryBindingChecked<WorthQueryContinuationBindingInput<D, I>>`
- `bind_continuation_request_from_context_proof(request) -> WorthQueryBindingTranscript<WorthQueryContinuationBindingInput<D, I>>`
- `bind_continuation_from_target(request) -> WorthQueryBindingOutcome<WorthQueryContinuationBindingInput<D, I>>`
- `bind_continuation_from_target_checked(request) -> WorthQueryBindingChecked<WorthQueryContinuationBindingInput<D, I>>`
- `bind_continuation_from_target_proof(request) -> WorthQueryBindingTranscript<WorthQueryContinuationBindingInput<D, I>>`
- `declare_review_progress_describe_plan_receipt_and_envelope(input) -> Result<WorthQueryDeclarationEnvelope<D, I>, WorthQueryDeclarationEntryEnvelopeError<D, I>>`

Envelope inspection:

- `class() -> WorthQueryDeclarationEnvelopeClass`
- `declaration_family_key() -> &'static str`
- `handle_identity_digest() -> &str`
- `operating_context_identity_digest() -> &str`
- `declaration_digest() -> &str`
- `progression_digest() -> &str`
- `route_plan_digest() -> &str`
- `receipt_digest() -> &CanonicalDerivedDigest`
- `envelope_digest() -> &CanonicalDerivedDigest`
- `binding_target() -> WorthQueryDeclarationEnvelopeBindingTarget`
- `foundational_evidence() -> &WorthQueryDeclarationFoundationalEvidence<D, I>`
- `receipt() -> &WorthQueryDeclarationReceipt<D, I>`
- `route_plan() -> &WorthQueryDeclarationRoutePlan<D, I>`
- `route_denial_cause() -> Option<WorthQueryDeclarationRoutePlanDenialCause>`
- `receipt_denial_cause() -> Option<WorthQueryDeclarationReceiptDenialCause>`
- `evidence_origin() -> WorthQueryDeclarationEnvelopeEvidenceOrigin`
- `aspect_contract() -> &WorthQueryDeclarationAspectContract`
- `aspect_publication() -> &WorthQueryDeclarationAspectPublication`
- `explain() -> &WorthQueryDeclarationEnvelopeExplanation`

Checked envelope outcomes:

- `WorthQueryDeclarationEnvelopeChecked::Enveloped(WorthQueryDeclarationEnvelope<D, I>)`
- `WorthQueryDeclarationEnvelopeChecked::Deferred(WorthQueryDeclarationEnvelopeDeferred<D, I>)`
- `WorthQueryDeclarationEnvelopeChecked::Denied(WorthQueryDeclarationEnvelopeDenied<D, I>)`
- `WorthQueryDeclarationEnvelopeChecked::Failed(WorthQueryDeclarationEnvelopeFailed<D, I>)`

Envelope classes:

- `CoveredCrossing`
- `DeferredCrossing`
- `DeniedCrossing`
- `FailedCrossing`

Evidence-origin values:

- `AdmittedProgression`
- `ProgressionDeferred`
- `ProgressionDenied`
- `ProgressionStale`
- `ProgressionRebindRequired`
- `ProgressionFailed`
- `LegalityEvidence`
- `LegalityDenial`

## Core Mental Model

Think of declaration boundary envelopes as the public crossing-story wrapper
over already-proved declaration truth:

1. progression proves whether the declaration can continue
2. foundational evidence publishes retained declaration truth
3. route planning decides what lower-authority participation is honestly in
   play
4. receipt materialization records the crossing posture Query is willing to
   claim
5. envelope construction publishes one self-describing artifact over that
   receipt-backed truth

That means envelopes do not invent new route or receipt meaning. They preserve
and present retained meaning that earlier declaration-entry surfaces already
proved.

The retained envelope artifact is also now one shared binding target. Later
relational-routing, bridge-continuation, signal-compatibility, and grouped
binding work should bind from this envelope identity rather than reopening the
declaration-entry seam.

That public seam is explicitly aspect-scoped:

- `aspect_contract()` tells later consumers which semantic crossing contract
  the envelope is actually publishing
- `aspect_publication()` tells them which slices are visible and which remain
  masked at the public boundary
- the envelope inherits that truth from the retained receipt instead of
  widening it from foundational evidence on its own

Declaration-entry orchestration, inspection, and readiness now consume that
published slice directly. They may summarize later relational, bridge, and
signal posture as retained consequence, but they do not treat those summaries
as proof that lower-authority routing already ran.

That also means later surfaces must stay honest about absence. If the retained
envelope alone does not prove one concrete lower-authority truth claim,
continuation family, or signal execution family, downstream inspection and
readiness surfaces may summarize the public slice but must not fabricate those
more specific facts.

## How It Executes

The advanced lane executes in this order:

1. produce admitted declaration progression
2. materialize one declaration boundary receipt from that retained route truth
3. wrap the issued, deferred, denied, or failed receipt in
   `WorthQueryDeclarationEnvelopeInput`
4. call `envelope_routes(...)` or `envelope_routes_checked(...)`
5. Query:
   - reuses retained foundational evidence through the receipt
   - reuses retained route explanation or route denial through the receipt
   - reuses retained receipt explanation or receipt denial directly
   - preserves the receipt-scoped aspect publication instead of widening the
     public story back to broader declaration semantics
   - derives one envelope digest from retained proof and denial topology
6. Query returns one enveloped, deferred, denied, or failed envelope artifact

The common lane preserves the same structure. It still lowers through:

1. admission
2. canonicalization
3. legality
4. progression
5. foundational evidence
6. route planning
7. receipt materialization
8. envelope construction

## Small Example

```rust
use worth_query::facade::foundation::{
    WorthQueryDeclarationEnvelopeChecked, WorthQueryDeclarationEnvelopeInput,
};

let receipt = handle.receipt_routes_from_progressed(
    handle.declare_review_and_progress(
        geometry_session.reclassify_active_boundary_loop_as_structural_opening()?,
    )?,
)?;

match handle.envelope_routes_checked(
    WorthQueryDeclarationEnvelopeInput::issued(receipt),
) {
    WorthQueryDeclarationEnvelopeChecked::Enveloped(envelope) => {
        let _ = envelope.receipt_digest();
        let _ = envelope.explain();
    }
    other => panic!("unexpected envelope outcome: {:?}", std::mem::discriminant(&other)),
}
```

This is the smallest honest example because it starts from retained receipt
truth and finishes with the public envelope artifact while still keeping the
checked envelope boundary visible.

## Real Example

```rust
use worth_query::facade::foundation::{
    WorthQueryDeclarationEnvelopeChecked, WorthQueryDeclarationEnvelopeInput,
    WorthQueryDeclarationFoundationalEvidenceInput, WorthQueryDeclarationRouteIntent,
    WorthQueryDeclarationReceiptInput, WorthQueryDeclarationRoutePlanInput,
};

let progressed = handle.progress_declaration(
    handle.declare_and_review(
        geometry_session.attach_material_for_active_face_selection()?,
    )?,
)?;

let evidence = handle.describe_foundational(
    WorthQueryDeclarationFoundationalEvidenceInput::admitted_progression(
        progressed.clone(),
    ),
)?;

let route_plan = handle.plan_routes(
    WorthQueryDeclarationRoutePlanInput::with_intent(
        progressed,
        evidence,
        WorthQueryDeclarationRouteIntent::Auto,
    ),
)?;

let receipt = handle.receipt_routes(
    WorthQueryDeclarationReceiptInput::planned(route_plan),
)?;

match handle.envelope_routes_checked(
    WorthQueryDeclarationEnvelopeInput::issued(receipt),
) {
    WorthQueryDeclarationEnvelopeChecked::Enveloped(envelope) => {
        assert_eq!(
            envelope.declaration_family_key(),
            "attach-face-material-assignment"
        );
        let _ = envelope.evidence_origin();
        let _ = envelope.route_plan_digest();
        let _ = envelope.receipt_digest();
        let _ = envelope.envelope_digest();
        let _ = envelope.explain();
    }
    WorthQueryDeclarationEnvelopeChecked::Deferred(envelope) => {
        let _ = envelope.reason();
    }
    WorthQueryDeclarationEnvelopeChecked::Denied(envelope) => {
        let _ = envelope.route_cause();
        let _ = envelope.receipt_cause();
    }
    WorthQueryDeclarationEnvelopeChecked::Failed(envelope) => {
        let _ = envelope.reason();
    }
}
```

What this example is showing:

- the envelope starts from retained receipt truth
- foundational evidence origin is still visible on the public story
- route posture and receipt posture stay distinct
- the envelope digest is separate from the receipt digest
- richer orchestration publication may expose more metadata, but it does not
  change the retained envelope truth being published

The main DX point is that the caller is still working from active geometry
context. Any canonical loop, face, or material identifiers stay inside the
retained declaration artifact instead of becoming the primary public targeting
story.

## How It Relates To Other Features

- [Declaration Boundary Receipts](./declaration-boundary-receipts.md) provide
  the retained crossing posture the envelope wraps
- Declaration Route Plans provide the retained
  route truth the receipt and envelope carry forward
- Declaration Foundational Evidence
  provides the retained declaration truth and evidence-origin posture the
  envelope preserves
- Runtime-Installed Domain Handles retain the
  admitted-world identity the envelope must not rediscover

Use receipts when you need the operational crossing artifact itself. Use
envelopes when you need the one self-describing public artifact that keeps
retained evidence, route truth, and receipt truth together for later
inspection or recovery work. Use
[Declaration Entry Orchestration](./declaration-entry-orchestration.md) when
you want Query to own the full declaration-entry lowering path through the
current envelope ceiling and produce a Query-owned orchestration plan,
outcome, or proof-visible transcript over that same retained envelope
boundary. Orchestration may stop honestly at `RoutePlanned` or
`ReceiptIssued` before any envelope is produced; use direct envelope surfaces
when you already know you want the receipt-backed crossing artifact itself. Use
[Declaration Relational Truth Routing](./declaration-relational-truth-routing.md)
when that same public crossing story needs to bind into relational truth
authority. Use
[Declaration Bridge Continuation Routing](./declaration-bridge-continuation-routing.md)
when it needs to bind into bridge continuation authority. Use
[Declaration Signal Compatibility](./declaration-signal-compatibility.md)
when you need to freeze whether that retained crossing story can later
continue into Signal-backed derived execution.

Use [Typed Binding Pipeline](./typed-binding-pipeline.md) when you already have
current-envelope context or one retained envelope target and want Query to
prepare the next continuation-ready input without pretending continuation
already executed.

## Inspection And Debugging

Use these surfaces when inspecting an envelope:

- `class()`
- `declaration_family_key()`
- `handle_identity_digest()`
- `operating_context_identity_digest()`
- `declaration_digest()`
- `progression_digest()`
- `route_plan_digest()`
- `receipt_digest()`
- `envelope_digest()`
- `binding_target()`
- `foundational_evidence()`
- `receipt()`
- `route_plan()`
- `route_denial_cause()`
- `receipt_denial_cause()`
- `evidence_origin()`
- `explain()`

Use the terminal envelope wrappers when the public crossing story did not land
as a covered crossing:

- `WorthQueryDeclarationEnvelopeDeferred::reason()`
- `WorthQueryDeclarationEnvelopeDenied::route_cause()`
- `WorthQueryDeclarationEnvelopeDenied::receipt_cause()`
- `WorthQueryDeclarationEnvelopeDenied::reason()`
- `WorthQueryDeclarationEnvelopeFailed::reason()`

These surfaces help answer:

- whether the envelope records a covered, deferred, denied, or failed crossing
- whether divergence came from admitted world, route posture, or receipt
  posture
- whether route denial or receipt denial is the real public blocker
- whether the public envelope preserved the same retained truth as the receipt
- which retained public crossing identity later continuation-oriented features
  should bind from directly

## Aspect Semantics

Envelopes publish one self-describing aspect-scoped crossing story. Continuation
and grouped-authoring surfaces should be
able to bind from what the envelope really published, including what it masked,
without reopening the lower receipt or route artifacts just to rediscover the
semantic slice that crossed. The envelope surface exposes
`aspect_contract()` and `aspect_publication()` for that purpose.

## Anti-Patterns

- attempting envelope construction from route plans without receipt truth
- attempting envelope construction from foundational evidence alone
- rebuilding route or receipt meaning from family labels or payload folklore
- treating envelopes as if they were allowed to hide denial topology
- treating envelopes and orchestration transcripts as interchangeable artifacts
- using public crossing-story APIs as if they were allowed to return free-form
  text or no artifact at all

## Current Limits

Declaration boundary envelopes now provide one self-describing public artifact
over retained receipt truth. They still do not provide:

- direct construction from foundational evidence or route plans on the public
  lane
- relational truth routing on their own; use the dedicated relational-routing
  surface when the envelope needs to bind into lower relational truth
- bridge continuation routing on their own; use the dedicated bridge-routing
  surface when the envelope needs to bind into lower continuation authority
- grouped declaration envelopes
- lower-authority execution internals beyond the retained route and receipt
  artifacts they compose

## Related Docs

- Runtime-Installed Domain Handles
- [Typed Binding Pipeline](./typed-binding-pipeline.md)
- [Declaration Boundary Receipts](./declaration-boundary-receipts.md)
- [Declaration Entry Orchestration](./declaration-entry-orchestration.md)
- [Declaration Relational Truth Routing](./declaration-relational-truth-routing.md)
- [Declaration Bridge Continuation Routing](./declaration-bridge-continuation-routing.md)
- Declaration Foundational Evidence
- Declaration Route Plans
- Declaration Progression
- [Domain Capabilities](./README.md)
