# Continuation Pipeline

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

The continuation pipeline is the Query-owned surface that turns a retained
continuation binding result into one prepared continuation artifact, and then
optionally executes that prepared artifact through the next lower continuation
boundary.

Use it when you already have one of these:

- a retained declaration envelope
- a continuation binding request from current context
- a prepared continuation artifact that is ready for explicit execution

This feature solves the next step after typed binding. Binding answers
"which continuation input is the right one?" The continuation pipeline answers
"what can Query prepare from that input, and what can it execute explicitly
without lying about basis, world, or runtime posture?"

In this doc, "basis" means the concrete runtime or truth-view identity that the
prepared continuation was built against. If that identity drifts before
execution, Query should tell you that directly instead of collapsing it into a
generic failure.

## Why You Use It

- prepare continuation without rebuilding bridge, basis, and workspace
  handoff glue yourself
- keep prepared and executed continuation as separate typed states
- preserve stale, deferred, denied, unsupported, wrong-world, wrong-handle,
  authority-mismatch, and basis-mismatch posture on the prepared lane
- preserve world-mismatch, support-mismatch, stale-basis, basis-mismatch,
  authority-mismatch, handle-mismatch, async-request drift, replay drift,
  remask drift, stale completion, and preview-crossed residue posture on the
  explicit execution lane
- inspect which bridge continuation family, truth context, workspace contract,
  and runtime contract Query selected
- keep concise `..._outcome(...)` calls on the same shared ordinary outcome
  vocabulary as the rest of declaration entry

## Stable Entry Points

Prepared/executed continuation types:

- `WorthQueryPreparedContinuation`
- `WorthQueryPreparedContinuationChecked`
- `WorthQueryPreparedContinuationTranscript`
- `WorthQueryPreparedContinuationOutcome`
- `WorthQueryContinuationExecution`
- `WorthQueryContinuationExecutionChecked`
- `WorthQueryContinuationExecutionTranscript`
- `WorthQueryContinuationExecutionOutcome`

Prepared continuation contract vocabulary:

- `WorthQueryPreparedContinuationFamily`
- `WorthQueryContinuationTruthContext`
- `WorthQueryContinuationBasisPosture`
- `WorthQueryContinuationWorkspaceContract`
- `WorthQueryContinuationRuntimeContract`
- `WorthQueryPreparedContinuationExecutionMode`
- `WorthQueryPreparedContinuationSignalPosture`

Request families:

- `WorthQueryPreparedContinuationRequest`
- `WorthQueryExecutePreparedContinuationRequest`

The exported execution request type exists for continuation-pipeline plumbing,
but the public admitted-handle execution lane accepts the prepared artifact
directly:

- `execute_prepared_continuation(prepared)`

Admitted-handle preparation entry points:

- `prepare_continuation_from_target(...)`
- `prepare_continuation_from_target_outcome(...)`
- `prepare_continuation_from_target_checked(...)`
- `prepare_continuation_from_target_proof(...)`
- `prepare_continuation_from_context(...)`
- `prepare_continuation_from_context_outcome(...)`
- `prepare_continuation_from_context_checked(...)`
- `prepare_continuation_from_context_proof(...)`

Admitted-handle execution entry points:

- `execute_prepared_continuation(...)`
- `execute_prepared_continuation_outcome(...)`
- `execute_prepared_continuation_checked(...)`
- `execute_prepared_continuation_proof(...)`

Good to know:

- preparation consumes the typed binding pipeline
- execution consumes only a prepared continuation artifact
- the concise `..._outcome(...)` lane reuses `WorthQueryOrdinaryOutcome<T>`
- this surface currently prepares and executes bridge-backed continuation
  families only

## Core Mental Model

Think about the continuation pipeline as two separate boundaries:

1. prepare continuation
2. execute prepared continuation

Preparation is where Query freezes the retained continuation truth it already
knows:

- which bridge continuation family is in play
- which truth context is in play
- which basis families are required
- which workspace and runtime contract later execution would need
- whether signal posture is compatible, deferred, denied, or failed

Preparation also freezes one execution-readmission witness. That witness is the
retained proof execution will later use to revalidate the concrete basis
identity, lower-runtime authority, and required capability support before it
admits execution.

When preparation starts from an already active subscription, the retained proof
also carries the active future-bearing lane posture and the active checkpoint
identity. That keeps temporal or async continuation from silently collapsing
 back into an ordinary lane later.

The same rule now applies at preview closeout: preview-owned temporal wakes,
async completion posture, and mixed-cause residue stay preview-local until
discard or authoritative re-admission proves a clean rebinding boundary.
Successful promotion records that rebinding digest explicitly, and rebinding
required denials stay typed instead of being flattened into generic promotion
failure.

Execution is a separate step that consumes that proof-bearing prepared artifact
and rechecks:

- admitted-world alignment
- the capability families the prepared continuation still requires
- retained lower-runtime basis freshness
- retained lower-runtime basis identity
- retained lower-runtime authority expectations
- admitted-handle alignment

Execution does not re-plan continuation semantics, rebuild bridge routing, or
re-run signal compatibility from scratch. Preparation derives one retained
execution-readmission witness once, and execution revalidates the world,
support, basis, and authority facts that witness carries forward.

The important rule is:

- prepared does not mean executed

If you only prepared continuation, Query has not claimed:

- runtime side effects already happened
- workspace mutations already happened
- lower bridge work already ran
- signal execution already ran

## How It Executes

The preparation path is:

1. bind continuation from a retained target or current context
2. recover the retained envelope-backed bridge subject
3. evaluate target-specific signal compatibility
4. route retained bridge continuation
5. derive the prepared continuation contract
6. return a prepared or typed non-success outcome

The execution path is:

1. accept one `WorthQueryPreparedContinuation`
2. verify admitted-world alignment
3. verify the prepared continuation's required capability families are still admitted
4. verify retained lower-runtime basis freshness
5. verify retained lower-runtime basis identity
6. verify retained lower-runtime authority alignment
7. verify handle alignment
8. derive one execution digest and execution artifact
9. return executed, wrong-world, unsupported, stale, basis-mismatch,
   authority-mismatch, async-request drift, replay drift, remask drift,
   stale-completion, preview-crossed-residue, or wrong-handle

Preparation does not build a second planning system. It reuses:

- the typed binding pipeline
- retained bridge continuation routing truth
- retained signal compatibility truth

Execution does not accept raw ids, envelopes, or ad hoc bags. It accepts the
prepared artifact directly on the public lane so the type system keeps the
ordering honest.

## Small Example

```rust
let envelope = handle.envelope_routes_from_progressed(
    handle.declare_review_and_progress(
        geometry_session.publish_boundary_change_for_active_face()?,
    )?,
)?;

let prepared = match handle.prepare_continuation_from_target(
    WorthQueryResolveContinuationFromTargetRequest::new(
        envelope,
        PublishBoundaryChange::aspect_contract(),
    ),
) {
    WorthQueryPreparedContinuationOutcome::Prepared(prepared) => prepared,
    other => panic!("unexpected continuation preparation outcome: {:?}", std::mem::discriminant(&other)),
};
```

This is the smallest honest example because it starts from retained envelope
truth and stops at preparation instead of implying later execution happened.

## Real Example

```rust
let envelope = handle.declare_review_progress_describe_plan_receipt_and_envelope(
    geometry_session.prepare_preview_for_active_face_selection()?,
)?;

let prepared = match handle.prepare_continuation_from_target(
    WorthQueryResolveContinuationFromTargetRequest::new(
        envelope,
        PreparePreviewForActiveFaceSelection::aspect_contract(),
    )
    .with_bridge_request(
        WorthQueryDeclarationBridgeContinuationRequest::new(
            WorthQueryDeclarationBridgeContinuationMode::RuntimeRoute,
            WorthQueryDeclarationBridgeTruthContext::Current,
        ),
    ),
) {
    WorthQueryPreparedContinuationOutcome::Prepared(prepared) => prepared,
    other => panic!("unexpected preparation outcome: {:?}", std::mem::discriminant(&other)),
};

assert_eq!(
    prepared.family(),
    WorthQueryPreparedContinuationFamily::BridgeRuntimeRoute,
);
assert_eq!(
    prepared.truth_context(),
    WorthQueryContinuationTruthContext::Current,
);
assert_eq!(
    prepared.runtime_contract(),
    WorthQueryContinuationRuntimeContract::RuntimeRoute,
);
assert_eq!(
    prepared.workspace_contract(),
    WorthQueryContinuationWorkspaceContract::RuntimeWorkspace,
);

let executed = match handle.execute_prepared_continuation(prepared) {
    WorthQueryContinuationExecutionOutcome::Executed(executed) => executed,
    other => panic!("unexpected execution outcome: {:?}", std::mem::discriminant(&other)),
};

let _ = executed.bridge_binding_surface();
let _ = executed.signal_execution_family();
let _ = executed.execution_digest();
```

What this example is showing:

- the envelope is still the authoritative public declaration crossing artifact
- preparation freezes continuation-family, truth-context, basis, workspace,
  runtime, and signal posture on one retained artifact
- execution is explicit and only starts from that prepared artifact
- execution revalidates world, support, basis, and authority against the
  retained readmission witness instead of trusting preparation blindly
- the final execution artifact still links back to prepared continuation truth

## How It Relates To Other Features

- [Typed Binding Pipeline](./typed-binding-pipeline.md) chooses or denies the
  continuation-ready input before preparation begins.
- [Ordinary Outcomes](./ordinary-outcomes.md) provide the compact public result
  lane for both preparation and execution.
- [Typed Stops And Remediation Guidance](./typed-stops-and-remediation-guidance.md) is the next-step surface when
  preparation or execution stopped and your app needs a typed repair answer.
- [Typed Stops And Remediation Guidance](./typed-stops-and-remediation-guidance.md) explains how
  continuation readmission posture shows up on that recovery lane.
- [Declaration Bridge Continuation Routing](./declaration-bridge-continuation-routing.md)
  remains the authority for which bridge continuation family and truth context
  Query selected.
- [Declaration Signal Compatibility](./declaration-signal-compatibility.md)
  remains the authority for derived-execution signal posture and basis-family
  requirements.
- [Signal Compatibility Orchestration](./signal-compatibility-orchestration.md)
  is the signal-facing composition lane that can reuse retained signal truth
  and optionally stop at `Compatible` before continuation preparation begins.
- Runtime-Installed Domain Handles remain the
  admitted-world boundary that preparation and execution both verify.

Use bridge routing when you want the retained bridge continuation artifact
directly. Use the continuation pipeline when you want Query to turn that
retained truth into one prepared continuation artifact and optional explicit
execution step. Use signal-compatibility orchestration when the public question
is still "compatible or prepared?" rather than "prepare or execute now."
Use [Async Resources And Result State](../capabilities/async-resources-and-result-state.md)
when you want the broader model for where async request identity, retained
async result-state, and async drift posture come from before continuation
classifies replay, remask, or stale-completion stops.

## Inspection And Debugging

Use the prepared artifact when you need to inspect what Query froze before
execution:

- `family()`
- `truth_context()`
- `basis_posture()`
- `workspace_contract()`
- `runtime_contract()`
- `execution_mode()`
- `required_basis_families()`
- `execution_readmission()`
- `signal_posture()`
- `signal_execution_family()`
- `signal_compatibility_digest()`
- `future_projection()`
- `basis_lifecycle_support_digest()`
- active future-bearing lane posture and active checkpoint identity when the
  prepared continuation came from an already active subscription
- `prepared_digest()`
- `bridge_routing()`
- `declaration_digest()`
- `progression_digest()`
- `route_plan_digest()`
- `receipt_digest()`
- `envelope_digest()`

Use the proof-visible lanes when you need to inspect why preparation or
execution stopped:

- `WorthQueryPreparedContinuationTranscript::{request, outcome, witness_checks, narrowing_decisions, prepared_digest, linked_artifacts}`
- `WorthQueryContinuationExecutionTranscript::{request, outcome, witness_checks, execution_digest, linked_artifacts}`

The execution proof currently explains only alignment and explicit execution:

Prepared continuation now carries the retained future-sensitive declaration
posture forward instead of re-deriving it later from family contracts or raw
declaration entries. If retained signal compatibility is `Deferred`, `Denied`,
or `Failed`, Query stops on the prepared lane before bridge preparation
materializes.

- `Executed`
- `WrongWorld`
- `Stale`
- `AsyncRequestDrift`
- `ReplayDrift`
- `RemaskDrift`
- `PreviewCrossedResidue`
- `StaleCompletion`
- `BasisMismatch`
- `AuthorityMismatch`
- `Unsupported`
- `WrongHandle`

The retained execution-readmission surface can also explain what execution was
checking:

- `execution_readmission().basis_witness().kind()`
- `execution_readmission().basis_witness().basis_identity_digest()`
- `execution_readmission().basis_witness().expected_lower_runtime_binding_digest()`
- `execution_readmission().basis_witness().source_basis_identity_digest()`
- `execution_readmission().authority_witness()`
- `execution_readmission().freshness_posture()`

Use the ordinary lane when you want the compact public result but still need a
checked-topology link:

- `prepare_continuation_from_target_outcome(...)`
- `prepare_continuation_from_context_outcome(...)`
- `execute_prepared_continuation_outcome(...)`

Use the recovery lane when you want Query to classify who owns the fix:

- `recover_from_outcome(...)`
- `recover_from_prepared_continuation_checked(...)`
- `recover_from_prepared_continuation_proof(...)`
- `recover_from_continuation_execution_checked(...)`
- `recover_from_continuation_execution_proof(...)`

Execution and recovery now keep the same typed continuation-drift vocabulary.
Temporal/async continuation drift is not flattened into generic stale or
unsupported posture:

- async request drift stays rebind-owned
- replay drift and stale completion stay basis-refresh owned
- remask drift stays support-owned
- preview-crossed residue stays explicit-handoff owned

When those retained drift classes later feed write-adjacent follow-on work,
Query routes them through the same effect-triggered intent lane used by other
pending write intents. Continuation does not own a second callback-shaped
mutation path.

## Anti-Patterns

- treating preparation as if it already executed lower bridge or runtime work
- rebuilding continuation-mode, truth-context, or basis rules by hand from the
  envelope instead of using the prepared artifact
- skipping the prepared artifact and trying to execute from raw envelopes or
  ids
- flattening prepared-lane `Stale`, `AuthorityMismatch`, and `BasisMismatch`
  into one generic continuation failure
- pretending the explicit execution lane still trusts prepared basis and
  authority posture without revalidation
- using this feature as if it replaced bridge routing or signal compatibility
  authority

## Current Limits

- this surface currently prepares and executes bridge-backed continuation
  families only
- signal posture is carried forward into the prepared artifact, but this
  feature
  does not execute Signal work
- active future-bearing checkpoint identity is preserved on the prepared lane,
  but durable replay and store-backed restart are still deferred
- preparation does not create a broad ambient runtime or workspace session by
  itself
- grouped or neighborhood continuation preparation is not part of this surface
- this feature does not replace the explicit lower-authority bridge and signal
  docs when you need their direct artifacts

## Related Docs

- Runtime-Installed Domain Handles
- [Typed Binding Pipeline](./typed-binding-pipeline.md)
- [Ordinary Outcomes](./ordinary-outcomes.md)
- [Declaration Bridge Continuation Routing](./declaration-bridge-continuation-routing.md)
- [Declaration Signal Compatibility](./declaration-signal-compatibility.md)
- [Signal Compatibility Orchestration](./signal-compatibility-orchestration.md)
- [Async Resources And Result State](../capabilities/async-resources-and-result-state.md)
- [Typed Stops And Remediation Guidance](./typed-stops-and-remediation-guidance.md)
- [Declaration Boundary Envelopes](./declaration-boundary-envelopes.md)
- [Declaration Entry Orchestration](./declaration-entry-orchestration.md)
- [Domain Capabilities](./README.md)
