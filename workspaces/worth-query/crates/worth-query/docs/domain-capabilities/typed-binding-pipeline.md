# Typed Binding Pipeline

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

The typed binding pipeline is the Query-owned surface that turns declared
context or retained declaration-entry artifacts into the next admissible Query
input without guessing silently.

Use it when your app already has meaningful context such as:

- an explicit selection
- the current admitted progression
- the current route plan
- the current receipt
- the current envelope

and you want Query to bind that context into the next declaration, route,
receipt, envelope, or continuation-ready input.

This is not the low-level `src/binding/` slot/canonicalization subsystem. It is
the retained-artifact and context-binding seam built on top of:

- shared retained binding targets from the declaration-entry seam
- explicit aspect contract and granularity semantics from the declaration-entry seam
- the existing route/receipt/envelope/orchestration boundaries

## Why You Use It

- keep context binding explicit instead of relying on ambient framework magic
- prefer aspect fit before source-order folklore when more than one candidate
  exists
- preserve wrong-world, wrong-handle, stale, rebind-required, and
  missing-aspect posture as typed outcomes
- bind retained artifacts into the next explicit Query input without reopening
  lower surfaces by hand
- get a proof-visible transcript when you need to inspect why one candidate won
  or why binding denied

## Stable Entry Points

- `WorthQueryDeclarationBindingRequest`
- `WorthQueryRouteBindingRequest`
- `WorthQueryReceiptBindingRequest`
- `WorthQueryEnvelopeBindingRequest`
- `WorthQueryContinuationBindingRequest`
- `WorthQueryResolveRouteFromTargetRequest`
- `WorthQueryResolveReceiptFromTargetRequest`
- `WorthQueryResolveEnvelopeFromTargetRequest`
- `WorthQueryResolveContinuationFromTargetRequest`
- `WorthQueryBindingOutcome<T>`
- `WorthQueryBindingChecked<T>`
- `WorthQueryBindingTranscript<T>`
- `WorthQueryBindingSourceKind`
- `WorthQueryBindingSpecificity`
- `WorthQueryBindingContextWitness`
- `WorthQueryBindingAuthorityWitness`
- `WorthQueryBindingBasisWitness`
- `WorthQueryBindingTargetWitnessSet`
- `WorthQueryBindingFamilyWitness`
- `WorthQueryFamilyBindingContract`
- `WorthQueryFamilyContextExtractorContract`
- `WorthQueryFamilyTargetResolverContract`

Admitted-handle entry points:

- `bind_declaration_from_context(...)`
- `bind_declaration_from_context_outcome(...)`
- `bind_declaration_from_context_checked(...)`
- `bind_declaration_from_context_proof(...)`
- `bind_route_request_from_context(...)`
- `bind_route_request_from_context_outcome(...)`
- `bind_route_request_from_context_checked(...)`
- `bind_route_request_from_context_proof(...)`
- `bind_receipt_request_from_context(...)`
- `bind_receipt_request_from_context_outcome(...)`
- `bind_receipt_request_from_context_checked(...)`
- `bind_receipt_request_from_context_proof(...)`
- `bind_envelope_request_from_context(...)`
- `bind_envelope_request_from_context_outcome(...)`
- `bind_envelope_request_from_context_checked(...)`
- `bind_envelope_request_from_context_proof(...)`
- `bind_continuation_request_from_context(...)`
- `bind_continuation_request_from_context_outcome(...)`
- `bind_continuation_request_from_context_checked(...)`
- `bind_continuation_request_from_context_proof(...)`
- `bind_route_from_target(...)`
- `bind_route_from_target_outcome(...)`
- `bind_route_from_target_checked(...)`
- `bind_route_from_target_proof(...)`
- `bind_receipt_from_target(...)`
- `bind_receipt_from_target_outcome(...)`
- `bind_receipt_from_target_checked(...)`
- `bind_receipt_from_target_proof(...)`
- `bind_envelope_from_target(...)`
- `bind_envelope_from_target_outcome(...)`
- `bind_envelope_from_target_checked(...)`
- `bind_envelope_from_target_proof(...)`
- `bind_continuation_from_target(...)`
- `bind_continuation_from_target_outcome(...)`
- `bind_continuation_from_target_checked(...)`
- `bind_continuation_from_target_proof(...)`

## Core Mental Model

Think of binding as a separate step that happens before orchestration or before
the next explicit artifact constructor:

1. Query gathers only the candidate sources the request allowed
2. Query rejects illegal candidates first
3. Query compares required aspect fit and coverage
4. Query applies explicit specificity rules
5. Query either binds one exact winner or returns a typed non-success outcome

The most important rule is:

- aspect fit beats source-order precedence

That means Query should prefer the candidate whose semantic slice best matches
the declared request. It should not silently choose whichever source happened
to be checked first.

## How It Executes

Context-binding requests carry:

- candidate sources
- required aspect contract
- allowed source kinds
- whether compatible supersets are acceptable
- whether partial fit should deny or return explicit narrowing required

Retained-target resolver requests carry:

- one typed retained artifact subject
- one required aspect contract
- optional route intent or continuation request narrowing

The public request families expose that configuration directly:

- `required_aspect_contract()`
- `allowed_sources()` on context-binding requests
- `allow_compatible_superset()`
- `partial_is_narrowing_required()`
- `route_intent()` on route/receipt/envelope resolver-style requests
- `bridge_request()` on continuation requests

The public family-scoped contract surface is inspectable too:

- `WorthQueryFamilyBindingContract::{family_key, required_aspect_contract}`
- `WorthQueryFamilyContextExtractorContract::{family_key, allowed_sources, required_aspect_contract}`
- `WorthQueryFamilyTargetResolverContract::{family_key, required_aspect_contract, route_intent, specificity_rank}`

The current source and subject families are intentionally narrow:

- declaration binding from explicit declaration candidates
- route/receipt/envelope request binding from current progression candidates
- continuation request binding from current envelope candidates
- route resolution from progression targets
- receipt resolution from progression or route-plan targets
- envelope resolution from progression or receipt targets
- continuation resolution from envelope targets

The public runtime proof covers the resolver-parity ladders that matter most
for this surface:

- `progression -> route`
- `route_plan -> receipt`
- `receipt -> envelope`
- `envelope -> continuation-ready input`

There is one more public layer on top of that checked/proof surface:

- `WorthQueryOrdinaryOutcome<T>`

Use the new `..._outcome(...)` entry points when you want one compact result
type without flattening the binding non-success taxonomy.

## Small Example

```rust
let progressed = handle.declare_review_and_progress(
    geometry_session.attach_material_for_active_face_selection()?,
)?;

let request = WorthQueryRouteBindingRequest::new(
    vec![WorthQueryProgressionContextCandidate::new(
        "current-progression",
        WorthQueryBindingSourceKind::CurrentProgression,
        WorthQueryBindingSpecificity::TypedCurrentArtifact,
        progressed,
    )],
    AttachFaceMaterial::aspect_contract(),
    vec![WorthQueryBindingSourceKind::CurrentProgression],
);

let route_input = match handle.bind_route_request_from_context(request) {
    WorthQueryBindingOutcome::Bound(input) => input,
    other => panic!("unexpected binding outcome: {:?}", std::mem::discriminant(&other)),
};

let route_plan = handle.plan_routes(route_input)?;
```

## Real Example

```rust
let envelope = handle.envelope_routes_from_progressed(
    handle.declare_review_and_progress(
        geometry_session.publish_boundary_change_for_active_face()?,
    )?,
)?;

let binding = handle.bind_continuation_from_target_proof(
    WorthQueryResolveContinuationFromTargetRequest::new(
        envelope,
        PublishBoundaryChange::aspect_contract(),
    ),
);

let _ = binding.request();
let _ = binding.outcome();
let _ = binding.candidates();
let _ = binding.witness_checks();
let _ = binding.aspect_fit_report();
let _ = binding.narrowing_decisions();
let _ = binding.resolved_target();
let _ = binding.linked_artifacts();
```

This is the intended shape:

- your app owns the current context or retained artifact
- Query owns the proof-bearing bind
- later route/receipt/envelope/continuation entry points still own actual
  lowering

## How It Relates To Other Features

- Runtime-Installed Domain Handles own the admitted
  world the binding pipeline checks first.
- [Continuation Pipeline](./continuation-pipeline.md) consumes
  continuation-ready binding results and turns them into prepared and executed
  continuation artifacts.
- [Ordinary Outcomes](./ordinary-outcomes.md) provide the compact public result
  lane over `WorthQueryBindingChecked<T>`.
- Declaration Progression introduces the first
  retained artifact most binding flows start from.
- Declaration Route Plans,
  [Declaration Boundary Receipts](./declaration-boundary-receipts.md), and
  [Declaration Boundary Envelopes](./declaration-boundary-envelopes.md) each
  expose `binding_target()` for later resolver surfaces.
- [Declaration Entry Orchestration](./declaration-entry-orchestration.md) is a
  separate boundary. Binding can prepare the next input or explain why it
  cannot; orchestration still owns declaration-entry lowering.

## Inspection And Debugging

Use the checked or proof-visible lane when you need to know:

- whether Query bound one exact winner or denied
- which candidates were considered
- which witness checks failed
- which aspect slices were missing, masked, or conflicting
- whether the result was ambiguous, stale, wrong-world, or rebind-required

Useful surfaces:

- `WorthQueryBindingOutcome::{Ambiguous, WrongWorld, WrongHandle, Stale, RebindRequired, MissingRequiredAspect, AspectConflict, AuthorityMismatch, BasisMismatch, ExplicitNarrowingRequired, Unsupported}`
- `WorthQueryBindingChecked::{outcome, binding_digest, linked_artifacts}`
- `WorthQueryBindingTranscript::candidates()`
- `WorthQueryBindingTranscript::witness_checks()`
- `WorthQueryBindingTranscript::aspect_fit_report()`
- `WorthQueryBindingTranscript::narrowing_decisions()`
- `WorthQueryBindingTranscript::resolved_target()`
- `WorthQueryBindingTranscript::binding_digest()`
- `WorthQueryBindingTranscript::linked_artifacts()`

## Anti-Patterns

- treating binding as if it were ambient dependency injection
- using binding to imply later execution happened
- letting source-order folklore decide between candidates with different aspect
  slices
- using raw ids or ad hoc bag objects when a retained target already exists
- flattening stale, wrong-world, and missing-aspect posture into one generic
  error path

## Current Limits

This surface does not yet provide:

- broad ambient UI probing outside the declared candidate set
- grouped or neighborhood binding
- arbitrary continuation execution
- prepared or executed continuation artifacts
- a second orchestration engine
- a replacement for explicit route/receipt/envelope constructors

## Related Docs

- Runtime-Installed Domain Handles
- [Continuation Pipeline](./continuation-pipeline.md)
- Declaration Progression
- Declaration Route Plans
- [Declaration Boundary Receipts](./declaration-boundary-receipts.md)
- [Declaration Boundary Envelopes](./declaration-boundary-envelopes.md)
- [Declaration Entry Orchestration](./declaration-entry-orchestration.md)
- [Domain Capabilities](./README.md)
