# Envelope To Signal Or Continuation

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../../foundations/ordinary-application-front-door.md).

## What This Workflow Is

This workflow starts from one retained envelope and answers the next
runtime-facing question:

- do you need a signal-facing compatibility answer?
- do you need explicit continuation preparation?
- do you need explicit continuation execution?

## Why You Use It

- keep signal compatibility separate from continuation preparation
- keep preparation separate from execution
- preserve basis, world, and authority posture instead of flattening everything
  into generic runtime readiness
- avoid rerunning declaration-entry work after envelope truth already exists

## Stable Entry Points

- `signal_compatibility(...)`
- `signal_compatibility_checked(...)`
- `orchestrate_signal_compatibility(...)`
- `orchestrate_signal_compatibility_outcome(...)`
- `prepare_continuation_from_target(...)`
- `prepare_continuation_from_target_checked(...)`
- `execute_prepared_continuation(...)`
- `execute_prepared_continuation_checked(...)`

## Core Mental Model

The envelope is the public declaration crossing artifact. From there, you
choose one of two lanes:

1. signal-facing classification
2. explicit continuation preparation and optional execution

Choose signal compatibility orchestration when you want Query to answer "is
this signal-compatible, and can it also prepare the next continuation step?"

Choose the continuation pipeline when you already know you need explicit
continuation preparation or execution.

## How It Executes

The signal-facing lane can stop at:

- `Compatible`
- `Prepared`
- one typed non-success result

The continuation pipeline splits into:

1. prepared continuation
2. executed continuation

That separation is important. Prepared does not mean executed.

## Small Example

```rust
let envelope = handle.declare_review_progress_describe_plan_receipt_and_envelope(
    geometry_session.prepare_preview_for_active_face_selection()?,
)?;

let outcome = handle.orchestrate_signal_compatibility(
    WorthQuerySignalCompatibilityOrchestrationInput::new(
        WorthQueryDeclarationSignalCompatibilityInput::enveloped(envelope),
    ),
);
```

Use this when you want the shortest signal-facing answer from one retained
envelope story.

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
    other => panic!(
        "unexpected preparation outcome: {:?}",
        std::mem::discriminant(&other)
    ),
};

let executed = handle.execute_prepared_continuation(prepared);
```

Use this when your app needs explicit prepared and executed continuation states
instead of a signal-facing combined answer.

## How It Relates To Other Features

- [Declaration Signal Compatibility](../declaration-signal-compatibility.md)
  is the retained compatibility surface behind the signal-facing lane.
- [Signal Compatibility Orchestration](../signal-compatibility-orchestration.md)
  is the combined signal-facing next-step surface.
- [Continuation Pipeline](../continuation-pipeline.md) owns prepared and
  executed continuation truth.
- [Typed Stops And Remediation Guidance](../typed-stops-and-remediation-guidance.md) is the next guide when either lane
  stops.

## Inspection And Debugging

Use signal compatibility when you need:

- signal execution family
- basis-family requirements
- a retained compatibility artifact without preparing continuation

Use the continuation pipeline when you need:

- the prepared continuation family
- truth context
- workspace contract
- runtime contract
- explicit execution-readmission posture

## Anti-Patterns

- treating `Compatible` as if continuation is already prepared
- treating `Prepared` as if runtime work already executed
- running continuation execution from raw envelope truth instead of from a
  prepared continuation artifact

## Current Limits

- signal compatibility does not execute Signal work
- continuation preparation and execution are currently bridge-backed only

## Related Docs

- [Signal Compatibility Orchestration](../signal-compatibility-orchestration.md)
- [Continuation Pipeline](../continuation-pipeline.md)
- [Typed Stops And Remediation Guidance](../typed-stops-and-remediation-guidance.md)
