# Declaration Signal Compatibility

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Declaration signal compatibility is the Query-owned boundary that turns one
retained declaration envelope into one retained compatibility artifact for
later Signal-backed derived execution.

This surface freezes whether later derived execution is structurally admitted,
explicitly deferred, denied, or failed without pretending Query already
executed through `worth-signal`.

The public lane is envelope-backed only. It does not start from
raw declarations, canonical declarations, legality evidence, foundational
evidence, route plans, or receipts alone.

## Why You Use It

- freeze later Signal-backed execution eligibility as one retained Query
  artifact instead of a loose posture flag
- preserve admitted-world identity, basis-family requirements, and retained
  route or receipt posture while classifying derived-execution compatibility
- keep mixed-authority declarations honest by treating signal compatibility as
  a modifier over retained declaration truth instead of a fake peer authority
  family
- get one signal-compatibility digest that converges when retained truth
  matches and diverges when world, basis family, authority posture, or denial
  topology changes
- hand later consumers one stable compatibility artifact so they do not reopen
  route, receipt, envelope, relational, or bridge meaning

## Stable Entry Points

- `WorthQueryDeclarationSignalCompatibilityInput`
- `WorthQueryDeclarationSignalCompatibility`
- `WorthQueryDeclarationSignalCompatibilityChecked`
- `WorthQueryDeclarationSignalCompatibilityDeferred`
- `WorthQueryDeclarationSignalCompatibilityDenied`
- `WorthQueryDeclarationSignalCompatibilityFailed`
- `WorthQueryDeclarationSignalCompatibilityTerminalError`
- `WorthQueryDeclarationEntrySignalCompatibilityError`
- `WorthQueryDeclarationSignalCompatibilityClass`
- `WorthQueryDeclarationSignalCompatibilityContract`
- `WorthQueryDeclarationSignalCompatibilityDenialCause`
- `WorthQueryDeclarationSignalCompatibilityExplanation`
- `WorthQueryDeclarationSignalCompatibilitySupportReport`
- `WorthQueryDeclarationSignalCompatibilitySupportRow`
- `WorthQueryDeclarationSignalCompatibilitySupportStatus`
- `WorthQueryDeclarationSignalExecutionFamily`
- `WorthQueryInstalledDomainDeclarationContext::signal_compatibility(...)`
- `WorthQueryInstalledDomainDeclarationContext::signal_compatibility_checked(...)`
- `WorthQueryInstalledDomainDeclarationContext::signal_compatibility_from_progressed(...)`
- `WorthQueryInstalledDomainDeclarationContext::signal_compatibility_from_progressed_with_intent(...)`
- `WorthQueryInstalledDomainDeclarationContext::declare_review_progress_describe_plan_receipt_envelope_and_check_signal_compatibility(...)`
- `WorthQueryInstalledDomainDeclarationContext::signal_compatibility_support::<I>()`
- `WorthQueryInstalledDomainDeclarationContext::orchestrate_signal_compatibility(...)`
- `WorthQueryInstalledDomainDeclarationContext::orchestrate_signal_compatibility_outcome(...)`
- `WorthQueryInstalledDomainDeclarationContext::orchestrate_signal_compatibility_checked(...)`
- `WorthQueryInstalledDomainDeclarationContext::orchestrate_signal_compatibility_proof(...)`
- `WorthQueryInstalledDomainDeclarationContext::declaration_entry_crossing_inventory::<I>()`
- `WorthQueryInstalledDomainDeclarationContext::declaration_entry_readiness::<I>()`

Good to know:

- the ordinary public lane is envelope-backed only
- common-lane helpers exist only for structurally signal-compatible
  declaration families
- deferred and incompatible families remain support-visible and checked-visible
- this surface freezes compatibility only; it does not execute, invalidate,
  recompute, or schedule Signal work

## API Reference

Signal-compatibility input constructors:

- `WorthQueryDeclarationSignalCompatibilityInput::enveloped(envelope)`
- `WorthQueryDeclarationSignalCompatibilityInput::deferred(envelope)`
- `WorthQueryDeclarationSignalCompatibilityInput::denied(envelope)`
- `WorthQueryDeclarationSignalCompatibilityInput::failed(envelope)`
- `WorthQueryDeclarationSignalCompatibilityInput::envelope_checked(checked)`

Admitted-handle signal-compatibility entry points:

- `signal_compatibility(subject) -> Result<WorthQueryDeclarationSignalCompatibility<D, I>, WorthQueryDeclarationSignalCompatibilityTerminalError<D, I>>`
- `signal_compatibility_checked(subject) -> WorthQueryDeclarationSignalCompatibilityChecked<D, I>`
- `signal_compatibility_from_progressed(progressed) -> Result<WorthQueryDeclarationSignalCompatibility<D, I>, WorthQueryDeclarationSignalCompatibilityTerminalError<D, I>>`
- `signal_compatibility_from_progressed_with_intent(progressed, intent) -> Result<WorthQueryDeclarationSignalCompatibility<D, I>, WorthQueryDeclarationSignalCompatibilityTerminalError<D, I>>`
- `declare_review_progress_describe_plan_receipt_envelope_and_check_signal_compatibility(input) -> Result<WorthQueryDeclarationSignalCompatibility<D, I>, WorthQueryDeclarationEntrySignalCompatibilityError<D, I>>`
- `signal_compatibility_support::<I>() -> WorthQueryDeclarationSignalCompatibilitySupportReport<D, I>`

Checked signal-compatibility outcomes:

- `WorthQueryDeclarationSignalCompatibilityChecked::Compatible(WorthQueryDeclarationSignalCompatibility<D, I>)`
- `WorthQueryDeclarationSignalCompatibilityChecked::Deferred(WorthQueryDeclarationSignalCompatibilityDeferred<D, I>)`
- `WorthQueryDeclarationSignalCompatibilityChecked::Denied(WorthQueryDeclarationSignalCompatibilityDenied<D, I>)`
- `WorthQueryDeclarationSignalCompatibilityChecked::Failed(WorthQueryDeclarationSignalCompatibilityFailed<D, I>)`

Signal-compatibility classes:

- `Compatible`
- `Deferred`
- `Denied`
- `Failed`

Future-bearing declarations use those same classes. Temporal and async
declarations do not get a side compatibility API. Query classifies them on the
same lane and now preserves their retained future posture directly on the
compatibility artifact through `future_projection()`.

Signal execution families:

- `RuntimeDerivedExecution`
- `HistoricalDerivedExecution`
- `PreviewDerivedExecution`
- `MixedDerivedExecution`

Support-report inspection:

- `WorthQueryDeclarationSignalCompatibilitySupportReport::declaration_family_key()`
- `WorthQueryDeclarationSignalCompatibilitySupportReport::rows()`
- `WorthQueryDeclarationSignalCompatibilitySupportReport::support_digest()`
- `WorthQueryDeclarationSignalCompatibilitySupportRow::execution_family()`
- `WorthQueryDeclarationSignalCompatibilitySupportRow::basis_family()`
- `WorthQueryDeclarationSignalCompatibilitySupportRow::required_dependency_aspects()`
- `WorthQueryDeclarationSignalCompatibilitySupportRow::produced_aspects()`
- `WorthQueryDeclarationSignalCompatibilitySupportRow::available_aspect_slice()`
- `WorthQueryDeclarationSignalCompatibilitySupportRow::aspect_fit()`
- `WorthQueryDeclarationSignalCompatibilitySupportRow::aspect_mismatch()`
- `WorthQueryDeclarationSignalCompatibilitySupportRow::status()`
- `WorthQueryDeclarationSignalCompatibilitySupportRow::reason()`

Support statuses:

- `Admitted`
- `Deferred`
- `Unsupported`
- `InvalidBasis`

## Core Mental Model

Think of signal compatibility as the declaration-side answer to one narrow
question:

"Can this retained declaration crossing story later continue into
Signal-backed derived execution, and if so, under which basis-family
requirements?"

That means:

1. envelopes still own the public crossing story
2. relational routing still owns relational truth binding
3. bridge routing still owns bridge continuation binding
4. signal compatibility freezes later derived-execution eligibility without
   re-running those earlier boundaries

The compatibility artifact is not a live Signal execution, schedule, or
invalidation surface. It is the proof-bearing Query artifact that says whether
later Signal execution is structurally admitted and which basis-family
requirements that later execution must respect.

The important shift is that signal compatibility now classifies from
the envelope's published semantic slice plus the declaration family's signal
contract. A structurally signal-capable family does not automatically satisfy
the dependency aspects later derived execution would need.

Later seam-ledger surfaces then use that retained compatibility artifact in
two different ways:

- readiness uses it as one family-level seam row projection
- inspection uses it as one retained lower seam posture when the signal
  compatibility boundary was actually crossed

## How It Executes

The advanced lane executes in this order:

1. start from `WorthQueryDeclarationSignalCompatibilityInput`
2. call `signal_compatibility(...)` or
   `signal_compatibility_checked(...)`
3. Query:
   - verifies the envelope belongs to the current admitted handle and world
   - verifies the retained envelope still represents covered crossing truth
   - reads the declaration family's signal-compatibility posture and contract
   - checks the envelope publication against the dependency aspect slice that
     later Signal-backed execution would require
   - derives one execution family and one required basis-family set from
     retained proof plus the family contract
   - checks the admitted-handle support snapshot for required capability and
     config posture
   - derives one signal-compatibility digest from retained proof and basis
     requirements
4. Query returns a compatible, deferred, denied, or failed compatibility
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
9. signal compatibility classification

## Small Example

```rust
use worth_query::facade::foundation::{
    WorthQueryDeclarationSignalCompatibilityChecked,
    WorthQueryDeclarationSignalCompatibilityInput,
};

let envelope = handle.envelope_routes_from_progressed(
    handle.declare_review_and_progress(
        geometry_session.prepare_preview_for_active_face_selection()?,
    )?,
)?;

match handle.signal_compatibility_checked(
    WorthQueryDeclarationSignalCompatibilityInput::enveloped(envelope),
) {
    WorthQueryDeclarationSignalCompatibilityChecked::Compatible(compatibility) => {
        let _ = compatibility.basis_families();
        let _ = compatibility.explain();
    }
    other => panic!(
        "unexpected compatibility outcome: {:?}",
        std::mem::discriminant(&other)
    ),
}
```

This is the smallest honest example because it starts from retained envelope
truth and keeps the checked boundary visible instead of pretending signal
execution already happened.

## Real Example

```rust
let compatibility = handle
    .declare_review_progress_describe_plan_receipt_envelope_and_check_signal_compatibility(
        geometry_session.prepare_preview_for_active_face_selection()?,
    )?;

assert_eq!(
    compatibility.execution_family(),
    WorthQueryDeclarationSignalExecutionFamily::RuntimeDerivedExecution,
);
let _ = compatibility.primary_authority_family();
let _ = compatibility.basis_families();
let _ = compatibility.signal_compatibility_digest();
let _ = compatibility.explain();
```

What this example is showing:

- the public lane still lowers through the full declaration proof chain
- the returned artifact says which Signal execution family is structurally in
  play later
- the returned artifact says which basis families later Signal execution must
  respect
- the returned artifact freezes dependency and produced-aspect posture instead
  of leaving later execution to rediscover it from envelope identity alone
- the returned digest is separate from the envelope digest because basis
  posture and signal compatibility are distinct retained facts

## Aspect Semantics

Signal compatibility speaks in dependency aspects, produced aspects, and
basis-sensitive aspect requirements rather than only family-level compatibility
posture.

This surface preserves a real semantic-slice compatibility artifact so later
execution and binding surfaces can reuse retained signal meaning instead of
approximating it locally.

The compatibility artifact and support rows expose:

- `aspect_contract()`
- `aspect_coverage()`
- `aspect_coverage_basis()`
- `aspect_fit()`
- `dependency_aspects()`
- `produced_aspects()`

That lets you distinguish "this family is signal-aware" from "this retained
envelope is missing the dependency slice and will report an
`AuthorityAspectGap`."

That contract now also stays honest in declaration-entry inspection: when a
denied compatibility artifact did not actually prove one execution family or
basis-family set, the inspection posture leaves those accessors empty instead
of filling them from broad signal family posture.

## How It Relates To Other Features

- [Declaration Boundary Envelopes](./declaration-boundary-envelopes.md) are
  the required public input to this surface
- [Declaration Relational Truth Routing](./declaration-relational-truth-routing.md)
  still owns relational truth binding; signal compatibility does not replace
  it
- [Declaration Bridge Continuation Routing](./declaration-bridge-continuation-routing.md)
  still owns bridge continuation binding and truth-context-sensitive bridge
  request selection
- [Continuation Pipeline](./continuation-pipeline.md) carries retained signal
  posture forward when Query prepares one continuation artifact for later
  explicit execution
- [Signal Compatibility Orchestration](./signal-compatibility-orchestration.md)
  composes this retained compatibility truth into one signal-facing next-step
  result that can stop at `Compatible` or advance into `Prepared`
- Runtime-Installed Domain Handles still own the
  support snapshot and admitted-world identity that compatibility classification
  must not rediscover
- `basis_lifecycle` still owns basis-family vocabulary; signal compatibility
  reuses it instead of inventing a second signal context grammar

Use envelopes when you need the public crossing story. Use bridge routing when
you need one real bridge continuation binding. Use signal compatibility when
you need to freeze whether that retained declaration story can later continue
into derived execution at all.

## Inspection And Debugging

Use these surfaces when inspecting a compatibility artifact:

- `class()`
- `execution_family()`
- `primary_authority_family()`
- `basis_families()`
- `aspect_contract()`
- `aspect_coverage()`
- `aspect_coverage_basis()`
- `aspect_fit()`
- `dependency_aspects()`
- `produced_aspects()`
- `declaration_family_key()`
- `handle_identity_digest()`
- `operating_context_identity_digest()`
- `declaration_digest()`
- `progression_digest()`
- `route_plan_digest()`
- `receipt_digest()`
- `envelope_digest()`
- `signal_compatibility_digest()`
- `envelope()`
- `route_denial_cause()`
- `receipt_denial_cause()`
- `evidence_origin()`
- `explain()`

Use `signal_compatibility_support::<I>()` when you need the family-scoped
readiness row before attempting compatibility classification.

That support row is the fastest way to answer:

- whether the family exposes any signal-compatibility contract at all
- which basis family a later derived-execution lane would require
- whether the current admitted world exposes the required Query capability
  families
- whether the current admitted world enables the `Signal` config section

## Anti-Patterns

- trying to start signal compatibility from receipts without envelope truth
- treating signal compatibility as if it already executed through
  `worth-signal`
- reconstructing basis requirements from family labels or host-local folklore
- inventing a second signal truth-context grammar instead of reusing basis
  lifecycle vocabulary
- assuming mixed-authority declarations become a fake peer signal authority
  family
- using common-lane helpers on non-signal-compatible declaration families

## Current Limits

Declaration signal compatibility now freezes retained declaration truth into
one Query-owned compatibility artifact. It still does not provide:

- actual `worth-signal` execution
- invalidation, recomputation, or scheduling semantics
- grouped declaration signal-compatibility batches
- a later unified inspection surface over all routed or compatible declaration
  artifacts
- public signal-compatibility entry from earlier declaration artifacts on the
  ordinary lane

The retained compatibility artifact tells you whether later Signal-backed
execution is structurally in play. It does not claim that derived execution
already ran.

When you want Query to reuse that retained signal posture while preparing one
explicit continuation artifact for lower bridge execution, move to
[Continuation Pipeline](./continuation-pipeline.md).

When you want one public signal-facing orchestration lane that can stop at
retained compatibility or advance into prepared continuation, move to
[Signal Compatibility Orchestration](./signal-compatibility-orchestration.md).

## Related Docs

- [Declaration Entry Inspection](./declaration-entry-inspection.md)
- [Declaration Entry Readiness](./declaration-entry-readiness.md)
- Runtime-Installed Domain Handles
- [Signal Compatibility Orchestration](./signal-compatibility-orchestration.md)
- [Declaration Boundary Envelopes](./declaration-boundary-envelopes.md)
- [Declaration Relational Truth Routing](./declaration-relational-truth-routing.md)
- [Declaration Bridge Continuation Routing](./declaration-bridge-continuation-routing.md)
- [Declaration Boundary Receipts](./declaration-boundary-receipts.md)
- Declaration Foundational Evidence
- [Domain Capabilities](./README.md)
