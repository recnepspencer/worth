# Declaration Relational Truth Routing

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Declaration relational truth routing is the Query-owned boundary that turns one
retained declaration envelope into one retained relational authority binding.

This is the Query boundary where retained declaration proof becomes one
relational-authority binding. Query still owns the public artifact and the
orchestration boundary. `worth-relational` still owns relational truth
semantics.

The public lane is envelope-backed only. It does not start from
raw declarations, canonical declarations, legality evidence, foundational
evidence, route plans, or receipts alone.

## Why You Use It

- route an already-proved declaration into the relational authority family it
  actually needs
- preserve admitted-world identity, route posture, receipt posture, and
  evidence origin while crossing into relational truth
- keep mixed route plans honest by lowering only the relational slice at this
  boundary
- get one relational-routing digest that converges when retained truth matches
  and diverges when world, route posture, receipt posture, or truth claim
  changes
- hand later consumers one Query-owned artifact instead of making them rediscover
  which relational surface was selected

## Stable Entry Points

- `WorthQueryDeclarationRelationalRoutingInput`
- `WorthQueryDeclarationRelationalRouting`
- `WorthQueryDeclarationRelationalRoutingChecked`
- `WorthQueryDeclarationRelationalRoutingDeferred`
- `WorthQueryDeclarationRelationalRoutingDenied`
- `WorthQueryDeclarationRelationalRoutingFailed`
- `WorthQueryDeclarationRelationalRoutingTerminalError`
- `WorthQueryDeclarationEntryRelationalRoutingError`
- `WorthQueryDeclarationRelationalRoutingClass`
- `WorthQueryDeclarationRelationalBinding`
- `WorthQueryDeclarationRelationalTruthContract`
- `WorthQueryDeclarationRelationalTruthClaim`
- `WorthQueryDeclarationRelationalAuthorityFamily`
- `WorthQueryDeclarationRelationalRoutingExplanation`
- `WorthQueryDeclarationRelationalRoutingSupportReport`
- `WorthQueryDeclarationRelationalRoutingSupportRow`
- `WorthQueryDeclarationRelationalTruthRoutingSupportStatus`
- `WorthQueryInstalledDomainDeclarationContext::route_relational_truth(...)`
- `WorthQueryInstalledDomainDeclarationContext::route_relational_truth_checked(...)`
- `WorthQueryInstalledDomainDeclarationContext::route_relational_truth_from_progressed(...)`
- `WorthQueryInstalledDomainDeclarationContext::route_relational_truth_from_progressed_with_intent(...)`
- `WorthQueryInstalledDomainDeclarationContext::declare_review_progress_describe_plan_receipt_envelope_and_route_relational_truth(...)`
- `WorthQueryInstalledDomainDeclarationContext::relational_truth_support::<I>()`

Good to know:

- the public lane is envelope-backed only
- common-lane helpers exist only for structurally relational declaration
  families
- mixed route plans still route here, but only the relational slice is
  lowered now
- deferred, denied, and failed envelopes remain first-class typed outcomes

## API Reference

Relational-routing input constructors:

- `WorthQueryDeclarationRelationalRoutingInput::enveloped(envelope)`
- `WorthQueryDeclarationRelationalRoutingInput::deferred(envelope)`
- `WorthQueryDeclarationRelationalRoutingInput::denied(envelope)`
- `WorthQueryDeclarationRelationalRoutingInput::failed(envelope)`
- `WorthQueryDeclarationRelationalRoutingInput::envelope_checked(checked)`

Admitted-handle relational-routing entry points:

- `route_relational_truth(subject) -> Result<WorthQueryDeclarationRelationalRouting<D, I>, WorthQueryDeclarationRelationalRoutingTerminalError<D, I>>`
- `route_relational_truth_checked(subject) -> WorthQueryDeclarationRelationalRoutingChecked<D, I>`
- `route_relational_truth_from_progressed(progressed) -> Result<WorthQueryDeclarationRelationalRouting<D, I>, WorthQueryDeclarationRelationalRoutingTerminalError<D, I>>`
- `route_relational_truth_from_progressed_with_intent(progressed, intent) -> Result<WorthQueryDeclarationRelationalRouting<D, I>, WorthQueryDeclarationRelationalRoutingTerminalError<D, I>>`
- `declare_review_progress_describe_plan_receipt_envelope_and_route_relational_truth(input) -> Result<WorthQueryDeclarationRelationalRouting<D, I>, WorthQueryDeclarationEntryRelationalRoutingError<D, I>>`
- `relational_truth_support::<I>() -> WorthQueryDeclarationRelationalRoutingSupportReport<D, I>`

Checked relational-routing outcomes:

- `WorthQueryDeclarationRelationalRoutingChecked::Routed(WorthQueryDeclarationRelationalRouting<D, I>)`
- `WorthQueryDeclarationRelationalRoutingChecked::Deferred(WorthQueryDeclarationRelationalRoutingDeferred<D, I>)`
- `WorthQueryDeclarationRelationalRoutingChecked::Denied(WorthQueryDeclarationRelationalRoutingDenied<D, I>)`
- `WorthQueryDeclarationRelationalRoutingChecked::Failed(WorthQueryDeclarationRelationalRoutingFailed<D, I>)`

Truth claims:

- `AuthoritativeCurrentTruth`
- `Identity`
- `Lineage`
- `HistoricalTruth`
- `InvariantTruth`
- `GroupedTruth`
- `StrategyTruth`

Authority families:

- `Runtime`
- `History`
- `GroupedTruth`
- `CommitStrategies`
- `BridgeSource`

## Core Mental Model

Think of relational truth routing as the binding step between declaration proof
and relational authority:

1. envelopes prove one public crossing story already exists
2. relational routing asks whether that envelope still admits a relational slice
3. Query selects one relational truth claim and one relational authority family
4. Query returns one retained routing artifact that points at the real lower
   relational surface without pretending Query owns relational semantics

That means the routing artifact is not a live relational runtime. It is the
proof-bearing Query boundary artifact that says which relational authority lane
this declaration now binds to.

The important shift is that relational routing now binds from the
envelope's published semantic slice, not from broad declaration identity
alone. Query is answering, "which relational truth slice is actually safe to
lower right now?"

## How It Executes

The advanced lane executes in this order:

1. start from `WorthQueryDeclarationRelationalRoutingInput`
2. call `route_relational_truth(...)` or `route_relational_truth_checked(...)`
3. Query:
   - verifies the envelope is covered crossing truth
   - verifies the retained route plan still includes a relational slice
   - narrows the declaration contract to the relational slice the envelope is
     actually publishing
   - reads the declaration family's relational truth contract
   - binds that truth contract to one real lower surface family:
     - `worth_relational::facade::runtime`
     - `worth_relational::facade::history`
     - `worth_runtime_bridge::facade::RelationalGroupedProjectionArtifact`
     - `worth_relational::facade::commit_strategies`
     - `worth_relational::facade::bridge::RuntimeBridgeRelationalSource`
   - derives one relational-routing digest from retained proof
4. Query returns a routed, deferred, denied, or failed relational-routing
   artifact

The common lane preserves the same structure. It still lowers through:

1. admission
2. canonicalization
3. legality
4. progression
5. foundational evidence
6. route planning
7. receipt materialization
8. envelope construction
9. relational truth routing

## Small Example

```rust
use worth_query::facade::foundation::{
    WorthQueryDeclarationRelationalRoutingChecked,
    WorthQueryDeclarationRelationalRoutingInput,
};

let envelope = handle.envelope_routes_from_progressed(
    handle.declare_review_and_progress(
        geometry_session.attach_material_for_active_face_selection()?,
    )?,
)?;

match handle.route_relational_truth_checked(
    WorthQueryDeclarationRelationalRoutingInput::enveloped(envelope),
) {
    WorthQueryDeclarationRelationalRoutingChecked::Routed(routing) => {
        let _ = routing.binding();
        let _ = routing.explain();
    }
    other => panic!("unexpected routing outcome: {:?}", std::mem::discriminant(&other)),
}
```

This is the smallest honest example because it starts from retained envelope
truth and ends at the first lower-authority truth-routing artifact without
hiding the checked boundary.

## Real Example

```rust
let routing = handle
    .declare_review_progress_describe_plan_receipt_envelope_and_route_relational_truth(
        geometry_session.attach_material_for_active_face_selection()?,
    )?;

assert_eq!(
    routing.truth_claim(),
    WorthQueryDeclarationRelationalTruthClaim::GroupedTruth,
);
let _ = routing.authority_family();
let _ = routing.binding();
let _ = routing.relational_routing_digest();
let _ = routing.explain();
```

What this example is showing:

- the public lane still lowers through the full declaration proof chain
- the routed artifact tells you which relational truth claim was selected
- the routed artifact tells you which lower authority family now owns truth
  semantics
- the routed artifact freezes the envelope-backed aspect slice, coverage basis,
  and fit before relational binding happens
- later consumers can keep using retained proof instead of rediscovering route or
  receipt meaning

## Aspect Semantics

Relational routing aligns directly with relational `required_aspects()` and
aspect-filtered truth access. Query does not invent a broader local notion of
relational truth here.

Routing success and denial expose which relational semantic slices were
required, covered, or missing so later consumers can trust the routed artifact
without rediscovering relational granularity themselves.

The routed artifact and support rows preserve:

- `aspect_contract()`
- `aspect_coverage()`
- `aspect_coverage_basis()`
- `aspect_fit()`

That means a family can be admitted structurally while still reporting an
`AuthorityAspectGap` or another slice-level mismatch before routing succeeds.

The same retained-truth rule now carries through declaration-entry inspection:
if a denied relational-routing artifact did not actually prove one concrete
truth claim or relational authority family, the inspection posture keeps those
fields empty instead of synthesizing a best guess from family posture alone.

## How It Relates To Other Features

- [Declaration Boundary Envelopes](./declaration-boundary-envelopes.md) are
  the required public input to this surface
- [Declaration Boundary Receipts](./declaration-boundary-receipts.md) still own
  crossing posture; routing does not replace them
- Declaration Route Plans still own route
  selection; routing only consumes the retained relational slice
- Runtime-Installed Domain Handles still own the
  public orchestration lane and support snapshot

Use envelopes when you need one public crossing artifact. Use relational truth
routing when you need to bind that crossing artifact to real relational truth
authority.

## Inspection And Debugging

Use these surfaces when inspecting a routed artifact:

- `class()`
- `truth_claim()`
- `authority_family()`
- `binding()`
- `aspect_contract()`
- `aspect_coverage()`
- `aspect_coverage_basis()`
- `aspect_fit()`
- `declaration_family_key()`
- `handle_identity_digest()`
- `operating_context_identity_digest()`
- `declaration_digest()`
- `progression_digest()`
- `route_plan_digest()`
- `receipt_digest()`
- `envelope_digest()`
- `relational_routing_digest()`
- `envelope()`
- `route_denial_cause()`
- `receipt_denial_cause()`
- `evidence_origin()`
- `explain()`

Use `relational_truth_support::<I>()` when you need the family-scoped readiness
row before attempting routing.

That support row is also where you inspect:

- `required_aspect_slice()`
- `available_aspect_slice()`
- `aspect_fit()`
- `aspect_mismatch()`

## Anti-Patterns

- trying to start relational truth routing from receipts without envelope truth
- trying to reconstruct relational target selection from family labels or route
  folklore
- treating the routed artifact as a live relational runtime
- assuming mixed route plans become pure relational successes at this boundary
- using common-lane helpers on non-relational families

## Current Limits

Declaration relational truth routing now binds retained envelope truth to one
real relational authority family. It still does not provide:

- bridge continuation routing for the mixed-authority slice; use the dedicated
  bridge-routing surface when the crossing story needs lower continuation
  authority too
- signal execution
- signal compatibility classification
- grouped declaration batching
- a later unified inspection surface over routed declaration artifacts
- direct execution of lower relational semantics inside Query

## Related Docs

- Runtime-Installed Domain Handles
- [Declaration Boundary Envelopes](./declaration-boundary-envelopes.md)
- [Declaration Boundary Receipts](./declaration-boundary-receipts.md)
- Declaration Route Plans
- Declaration Foundational Evidence
- [Declaration Bridge Continuation Routing](./declaration-bridge-continuation-routing.md)
- [Declaration Signal Compatibility](./declaration-signal-compatibility.md)
- [Domain Capabilities](./README.md)
