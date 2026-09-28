# Declaration Boundary Receipts

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Declaration boundary receipts are the Query-owned operational artifacts that
record what crossing posture Query is willing to claim after route planning.

They are built from retained declaration truth, not from fresh discovery:

- admitted-world proof from the handle
- retained declaration truth from foundational evidence
- retained route truth from route planning
- foundational boundary-receipt materialization primitives

They are not the same thing as foundational descriptive receipts. Foundational
evidence receipts describe retained declaration truth. Declaration boundary
receipts record the Query crossing posture that followed from that truth.

## Why You Use It

- keep one public artifact for declaration crossing posture
- preserve deferred, denied, and failed crossing outcomes instead of dropping
  them
- carry forward route explanation and typed route-denial causes
- hand later envelope, inspection, and recovery features one stable receipt
  surface
- get one receipt digest that converges when retained truth matches and
  diverges when admitted world or crossing posture changes

## Stable Entry Points

- `WorthQueryDeclarationReceiptInput`
- `WorthQueryDeclarationReceipt`
- `WorthQueryDeclarationReceiptChecked`
- `WorthQueryDeclarationReceiptDeferred`
- `WorthQueryDeclarationReceiptDenied`
- `WorthQueryDeclarationReceiptFailed`
- `WorthQueryDeclarationReceiptTerminalError`
- `WorthQueryDeclarationEntryReceiptError`
- `WorthQueryDeclarationReceiptClass`
- `WorthQueryDeclarationReceiptKind`
- `WorthQueryDeclarationReceiptExplanation`
- `WorthQueryDeclarationReceiptDenialCause`
- `WorthQueryInstalledDomainDeclarationContext::receipt_routes(...)`
- `WorthQueryInstalledDomainDeclarationContext::receipt_routes_checked(...)`
- `WorthQueryInstalledDomainDeclarationContext::receipt_routes_from_progressed(...)`
- `WorthQueryInstalledDomainDeclarationContext::receipt_routes_from_progressed_with_intent(...)`
- `WorthQueryInstalledDomainDeclarationContext::receipt_routes_from_progressed(...)`
- `WorthQueryInstalledDomainDeclarationContext::receipt_routes_from_progressed_with_intent(...)`
- `WorthQueryInstalledDomainDeclarationContext::declare_review_progress_describe_plan_and_receipt(...)`

Good to know:

- boundary receipts start from retained route truth, not from legality evidence
  or foundational evidence alone
- the admitted handle remains the public entry surface because admitted-world
  identity is part of the retained proof chain
- denied and deferred crossings still produce Query-owned receipt artifacts
- signal-only route plans are not treated as successful receipt kinds in the
  current implementation

## API Reference

Receipt input constructors:

- `WorthQueryDeclarationReceiptInput::planned(plan)`
- `WorthQueryDeclarationReceiptInput::deferred(plan)`
- `WorthQueryDeclarationReceiptInput::denied(plan)`
- `WorthQueryDeclarationReceiptInput::failed(plan)`
- `WorthQueryDeclarationReceiptInput::route_checked(checked)`

Admitted-handle receipt entry points:

- `receipt_routes(subject) -> Result<WorthQueryDeclarationReceipt<D, I>, WorthQueryDeclarationReceiptTerminalError<D, I>>`
- `receipt_routes_checked(subject) -> WorthQueryDeclarationReceiptChecked<D, I>`
- `receipt_routes_from_progressed(progressed) -> Result<WorthQueryDeclarationReceipt<D, I>, WorthQueryDeclarationReceiptTerminalError<D, I>>`
- `receipt_routes_from_progressed_with_intent(progressed, intent) -> Result<WorthQueryDeclarationReceipt<D, I>, WorthQueryDeclarationReceiptTerminalError<D, I>>`
- `receipt_routes_from_progressed(progressed) -> Result<WorthQueryDeclarationReceipt<D, I>, WorthQueryDeclarationReceiptTerminalError<D, I>>`
- `receipt_routes_from_progressed_with_intent(progressed, intent) -> Result<WorthQueryDeclarationReceipt<D, I>, WorthQueryDeclarationReceiptTerminalError<D, I>>`
- `bind_envelope_from_target(request) -> WorthQueryBindingOutcome<WorthQueryDeclarationEnvelopeInput<D, I>>`
- `bind_envelope_from_target_checked(request) -> WorthQueryBindingChecked<WorthQueryDeclarationEnvelopeInput<D, I>>`
- `bind_envelope_from_target_proof(request) -> WorthQueryBindingTranscript<WorthQueryDeclarationEnvelopeInput<D, I>>`
- `declare_review_progress_describe_plan_and_receipt(input) -> Result<WorthQueryDeclarationReceipt<D, I>, WorthQueryDeclarationEntryReceiptError<D, I>>`

Checked receipt outcomes:

- `WorthQueryDeclarationReceiptChecked::Issued(WorthQueryDeclarationReceipt<D, I>)`
- `WorthQueryDeclarationReceiptChecked::Deferred(WorthQueryDeclarationReceiptDeferred<D, I>)`
- `WorthQueryDeclarationReceiptChecked::Denied(WorthQueryDeclarationReceiptDenied<D, I>)`
- `WorthQueryDeclarationReceiptChecked::Failed(WorthQueryDeclarationReceiptFailed<D, I>)`

Receipt classes:

- `CoveredCrossing`
- `DeferredCrossing`
- `DeniedCrossing`
- `FailedCrossing`

Receipt kinds:

- `Relational`
- `Bridge`
- `Mixed`
- `Deferred`
- `Denied`
- `Failed`

Receipt-denial causes:

- `MissingRoutePlan`
- `UnsupportedReceiptKind`
- `ReceiptMaterializationMismatch`
- `RouteIntegrityMismatch`

Receipt inspection:

- `class() -> WorthQueryDeclarationReceiptClass`
- `kind() -> WorthQueryDeclarationReceiptKind`
- `declaration_family_key() -> &'static str`
- `handle_identity_digest() -> &str`
- `operating_context_identity_digest() -> &str`
- `declaration_digest() -> &str`
- `progression_digest() -> &str`
- `route_plan_digest() -> &str`
- `receipt_digest() -> &CanonicalDerivedDigest`
- `binding_target() -> WorthQueryDeclarationReceiptBindingTarget`
- `foundational_evidence() -> &WorthQueryDeclarationFoundationalEvidence<D, I>`
- `route_plan() -> &WorthQueryDeclarationRoutePlan<D, I>`
- `route_denial_cause() -> Option<WorthQueryDeclarationRoutePlanDenialCause>`
- `aspect_contract() -> &WorthQueryDeclarationAspectContract`
- `aspect_coverage() -> &WorthQueryDeclarationAspectCoverage`
- `aspect_publication() -> &WorthQueryDeclarationAspectPublication`
- `explain() -> &WorthQueryDeclarationReceiptExplanation`
- `descriptive_receipt() -> Option<&FoundationalBoundaryEvidenceCompletedReceiptArtifact>`
- `boundary_receipt() -> &FoundationalBoundaryReceipt`

## Core Mental Model

Think of declaration boundary receipts as the first operational artifact after
planning:

1. progression proves the declaration can continue
2. foundational evidence publishes retained declaration truth
3. route planning decides which lower-authority participation is honestly in
   play
4. receipt materialization records the crossing posture Query is willing to
   claim

That means:

- successful planned route participation becomes an issued crossing receipt
- deferred route participation becomes a deferred receipt artifact
- denied route participation becomes a denied receipt artifact
- failed route participation becomes a failed receipt artifact

The receipt stays Query-owned even when the lower authority has its own local
receipt or evidence language.

The retained receipt artifact is also now one shared binding target. Envelope
construction and later continuation surfaces should bind from this receipt
identity directly instead of rebuilding crossing posture from lower-level
artifacts.

The retained receipt is explicitly route-scoped:

- `aspect_contract()` records which semantic slices the crossing claim is
  actually about
- `aspect_coverage()` records which retained slices were present, masked, or
  conflicting at the crossing seam
- `aspect_publication()` records what this receipt is willing to publish at its
  own materialization tier without widening beyond route-backed truth

Declaration boundary envelopes are the next public crossing-story boundary.
Bridge continuation routing is the later lower-authority continuation boundary
that can consume those retained envelope truths after relational routing or on
bridge-authoritative family paths.

## How It Executes

The advanced lane executes in this order:

1. produce admitted declaration progression
2. describe matching foundational evidence from that progressed declaration
3. produce route truth through route planning
4. wrap planned, deferred, denied, or failed route truth in
   `WorthQueryDeclarationReceiptInput`
5. call `receipt_routes(...)` or `receipt_routes_checked(...)`
6. Query:
   - reuses the retained foundational evidence
   - reuses the retained route explanation or route-denial cause
   - narrows the receipt aspect contract to the route-backed slice instead of
     reusing the broader declaration contract verbatim
   - materializes one foundational boundary receipt surface
   - derives one Query receipt digest from retained proof and crossing posture
7. Query returns one issued, deferred, denied, or failed receipt artifact

The common lane preserves the same structure. It still lowers through:

1. admission
2. canonicalization
3. legality
4. progression
5. foundational evidence
6. route planning
7. receipt materialization

## Small Example

```rust
use worth_query::facade::foundation::{
    WorthQueryDeclarationReceiptChecked, WorthQueryDeclarationReceiptInput,
};

let progressed = handle.declare_review_and_progress(
    geometry_session.attach_face_material_for_active_selection()?,
)?;

match handle.receipt_routes_checked(
    WorthQueryDeclarationReceiptInput::planned(
        handle.plan_routes_from_progressed(progressed)?,
    ),
) {
    WorthQueryDeclarationReceiptChecked::Issued(receipt) => {
        let _ = receipt.boundary_receipt();
    }
    other => panic!("unexpected receipt outcome: {:?}", std::mem::discriminant(&other)),
}
```

This is the smallest honest example because it starts from admitted progression
and finishes with a receipt artifact, but it still keeps the checked receipt
boundary visible.

## Real Example

```rust
use worth_query::facade::foundation::{
    WorthQueryDeclarationFoundationalEvidenceInput, WorthQueryDeclarationReceiptChecked,
    WorthQueryDeclarationReceiptInput, WorthQueryDeclarationRouteIntent,
    WorthQueryDeclarationRoutePlanInput,
};

let progressed = handle.progress_declaration(
    handle.declare_and_review(AttachFaceMaterialAssignment {
        face_ref: "face:loading-bay-west",
        material_profile_ref: "material-profile:fire-rated-primer",
    })?,
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

match handle.receipt_routes_checked(
    WorthQueryDeclarationReceiptInput::planned(route_plan),
) {
    WorthQueryDeclarationReceiptChecked::Issued(receipt) => {
        assert_eq!(receipt.declaration_family_key(), "attach-face-material");
        let _ = receipt.receipt_digest();
        let _ = receipt.explain();
        let _ = receipt.boundary_receipt();
    }
    WorthQueryDeclarationReceiptChecked::Deferred(receipt) => {
        let _ = receipt.reason();
    }
    WorthQueryDeclarationReceiptChecked::Denied(receipt) => {
        let _ = receipt.route_cause();
        let _ = receipt.receipt_cause();
    }
    WorthQueryDeclarationReceiptChecked::Failed(receipt) => {
        let _ = receipt.reason();
    }
}
```

What this example is showing:

- receipt construction begins from retained route truth
- the receipt retains foundational evidence instead of rematerializing it
- the receipt keeps route explanation and denial topology visible
- the receipt exposes both Query-level inspection and the underlying
  foundational boundary receipt artifact
- orchestration may publish the same retained receipt truth through a leaner
  or richer policy without changing the receipt's semantic crossing story

## How It Relates To Other Features

- Declaration Progression produces the admitted
  declaration proof the receipt pipeline ultimately depends on
- Declaration Foundational Evidence
  publishes the retained declaration truth the receipt must carry forward
- Declaration Route Plans produce the crossing
  posture and route explanation receipts consume
- Runtime-Installed Domain Handles retain the
  admitted-world identity receipts must not rediscover
- [Declaration Boundary Envelopes](./declaration-boundary-envelopes.md) bind
  their public crossing story from this retained receipt target
- [Typed Binding Pipeline](./typed-binding-pipeline.md) turns a retained receipt
  target into the next envelope input while preserving wrong-world,
  wrong-handle, and aspect-fit denial posture

Use route planning when you need to decide what lower-authority participation
is in play. Use boundary receipts when you need the public operational artifact
that records what Query was actually willing to claim from that plan. Use
[Declaration Boundary Envelopes](./declaration-boundary-envelopes.md) when you
need the one self-describing public artifact that carries receipt truth, route
truth, and retained evidence together.

## Inspection And Debugging

Use these surfaces when inspecting a receipt:

- `class()`
- `kind()`
- `declaration_family_key()`
- `handle_identity_digest()`
- `operating_context_identity_digest()`
- `declaration_digest()`
- `progression_digest()`
- `route_plan_digest()`
- `receipt_digest()`
- `binding_target()`
- `foundational_evidence()`
- `route_plan()`
- `route_denial_cause()`
- `explain()`
- `descriptive_receipt()`
- `boundary_receipt()`

Use the terminal receipt wrappers when the crossing did not issue:

- `WorthQueryDeclarationReceiptDeferred::reason()`
- `WorthQueryDeclarationReceiptDenied::route_cause()`
- `WorthQueryDeclarationReceiptDenied::receipt_cause()`
- `WorthQueryDeclarationReceiptDenied::reason()`
- `WorthQueryDeclarationReceiptFailed::reason()`

These surfaces help answer:

- whether the receipt records a covered, deferred, denied, or failed crossing
- whether denial came from route truth or from receipt-boundary materialization
- whether two equivalent retained-proof paths converged to the same receipt
  digest
- whether divergence came from admitted world, route posture, or receipt kind
- which retained receipt identity later envelope or continuation consumers
  should bind to directly

## Aspect Semantics

Receipts state which semantic slices the crossing claim
actually covers. A receipt is no longer allowed to overclaim broad crossing
truth when only some route-backed slices truly crossed. Later binding and
envelope publication must be able to distinguish covered, masked, and merely
adjacent semantics from the retained receipt itself. The receipt surface
exposes `aspect_contract()`, `aspect_coverage()`, and
`aspect_publication()` so later layers can consume that retained crossing truth
directly.

## Anti-Patterns

- attempting receipt construction from legality evidence alone
- attempting receipt construction from foundational evidence without route
  truth
- rebuilding receipt meaning from family labels or payload folklore
- treating foundational descriptive receipts as if they were already Query
  crossing receipts
- using successful receipt APIs as if they were allowed to return free-form
  text or no artifact at all

## Current Limits

Declaration boundary receipts now provide one Query-owned crossing artifact
over retained route truth. They still do not provide:

- public boundary envelopes over that retained receipt truth
- public relational truth routing from retained envelope truth
- public bridge continuation routing from retained envelope truth
- public signal compatibility classification from retained envelope truth
- grouped declaration crossing receipts
- successful signal-only receipt kinds
- lower-authority execution internals outside the retained route plan and
  foundational receipt surfaces

## Related Docs

- Runtime-Installed Domain Handles
- [Typed Binding Pipeline](./typed-binding-pipeline.md)
- [Declaration Boundary Envelopes](./declaration-boundary-envelopes.md)
- Declaration Foundational Evidence
- [Declaration Relational Truth Routing](./declaration-relational-truth-routing.md)
- [Declaration Bridge Continuation Routing](./declaration-bridge-continuation-routing.md)
- [Declaration Signal Compatibility](./declaration-signal-compatibility.md)
- Declaration Route Plans
- Declaration Progression
- [Domain Capabilities](./README.md)
