# Signal Compatibility Orchestration

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Signal compatibility orchestration is the Query-owned surface that composes two
existing retained surfaces into one signal-facing next-step result:

- retained declaration signal compatibility
- optional prepared continuation

Use it when you want Query to answer a signal-facing question from one retained
declaration story without making you manually stitch together:

- signal compatibility classification
- optional continuation preparation
- ordinary outcome projection

This feature does not execute Signal work. It classifies compatibility, may
prepare continuation when you explicitly ask for that next step, and otherwise
stops honestly at retained compatibility.

## Why You Use It

- ask for the signal-facing next step from one retained declaration story
- keep `Compatible`, `Prepared`, and later explicit execution as separate states
- preserve wrong-world, wrong-handle, deferred, basis-mismatch, and denied
  posture as typed outcomes
- start from retained signal-compatibility input after declaration entry has
  already produced envelope-backed truth
- keep the concise `..._outcome(...)` lane on the shared ordinary outcome
  vocabulary instead of learning a signal-only convenience error family

## Stable Entry Points

Core orchestration types:

- `WorthQuerySignalCompatibilityOrchestrationInput`
- `WorthQuerySignalCompatibilityOrchestration`
- `WorthQuerySignalCompatibilityOrchestrationClass`
- `WorthQuerySignalCompatibilityOrchestrationOutcome`
- `WorthQuerySignalCompatibilityOrchestrationChecked`
- `WorthQuerySignalCompatibilityOrchestrationTranscript`

Admitted-handle entry points:

- `orchestrate_signal_compatibility(...)`
- `orchestrate_signal_compatibility_outcome(...)`
- `orchestrate_signal_compatibility_checked(...)`
- `orchestrate_signal_compatibility_proof(...)`

Good to know:

- this is a composition boundary, not a second signal-compatibility engine
- it reuses retained declaration signal-compatibility truth when it prepares
  continuation
- it can stop at retained compatibility without preparing continuation

## Core Mental Model

Think of this feature as one signal-facing orchestration question:

"Given this retained declaration story, does Query stop at signal compatibility,
or can it also prepare the next continuation step right now?"

That gives you two success-shaped outcomes that mean different things:

- `Compatible`
  - Query proved the declaration story is signal-compatible
  - Query did not prepare continuation
- `Prepared`
  - Query proved signal compatibility
  - Query also prepared one continuation artifact

The important rule is:

- compatible does not mean prepared
- prepared does not mean executed
- deferred or denied signal compatibility does not advance into prepared
  continuation just because a bridge request was supplied

Future-bearing declarations stay on this same lane too. Temporal and async
subjects can stop at `Deferred` or `Denied` before any prepared continuation
artifact exists, and ordinary declarations do not inherit that posture unless
their retained future projection actually activates it.

If you want explicit execution after preparation, move to the continuation
pipeline.

## How It Executes

The orchestration path is:

1. accept `WorthQuerySignalCompatibilityOrchestrationInput`
2. start from retained declaration signal-compatibility input
3. produce one retained signal-compatibility checked result
4. if no bridge request is present:
   - stop at retained compatibility
5. if a bridge request is present:
   - reuse the retained signal truth
   - prepare continuation through the continuation pipeline
6. return:
   - `Compatible`
   - `Prepared`
   - or typed non-success posture

The important cost boundary is that the bridge-request path does not rediscover
signal compatibility from scratch after it already has retained compatibility
truth.

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

match outcome {
    WorthQuerySignalCompatibilityOrchestrationOutcome::Bound(value) => {
        assert_eq!(
            value.class(),
            WorthQuerySignalCompatibilityOrchestrationClass::Compatible,
        );
        let _ = value.signal_execution_family();
        let _ = value.basis_families();
    }
    other => panic!("unexpected signal orchestration outcome: {:?}", std::mem::discriminant(&other)),
}
```

Use this when declaration entry already gave you envelope-backed truth and you
want the signal-facing classification without manually stitching together the
compatibility lane and the optional continuation lane yourself.

## Real Example

```rust
let envelope = handle.declare_review_progress_describe_plan_receipt_and_envelope(
    geometry_session.prepare_preview_for_active_face_selection()?,
)?;

let outcome = handle.orchestrate_signal_compatibility(
    WorthQuerySignalCompatibilityOrchestrationInput::new(
        WorthQueryDeclarationSignalCompatibilityInput::enveloped(envelope),
    )
    .with_bridge_request(
        WorthQueryDeclarationBridgeContinuationRequest::new(
            WorthQueryDeclarationBridgeContinuationMode::RuntimeRoute,
            WorthQueryDeclarationBridgeTruthContext::Current,
        ),
    ),
);

match outcome {
    WorthQuerySignalCompatibilityOrchestrationOutcome::Bound(value) => {
        assert_eq!(
            value.class(),
            WorthQuerySignalCompatibilityOrchestrationClass::Prepared,
        );
        let _ = value.signal_execution_family();
        let _ = value.basis_families();
        let _ = value.route_plan_digest();
        let _ = value.receipt_digest();
        let _ = value.envelope_digest();
    }
    WorthQuerySignalCompatibilityOrchestrationOutcome::BasisMismatch(reason) => {
        panic!("basis repair required: {reason}");
    }
    other => panic!("unexpected signal orchestration outcome: {:?}", std::mem::discriminant(&other)),
}
```

What this example is showing:

- the public input starts from retained declaration truth
- declaration entry gets you to envelope truth
- declaration signal compatibility consumes envelope-backed truth
- this feature composes retained signal truth into either `Compatible` or
  `Prepared`
- the optional bridge request asks Query to continue one step farther
- the result still stays explicit about whether it stopped at compatibility or
  advanced into preparation
- no Signal execution happened here

## How It Relates To Other Features

- [Declaration Signal Compatibility](./declaration-signal-compatibility.md)
  remains the authority for retained compatibility, signal execution family,
  required basis families, and compatibility denial causes.
- [Continuation Pipeline](./continuation-pipeline.md) remains the authority for
  prepared continuation and explicit execution.
- [Ordinary Outcomes](./ordinary-outcomes.md) provide the concise shared result
  vocabulary for `orchestrate_signal_compatibility_outcome(...)`.
- [Typed Stops And Remediation Guidance](./typed-stops-and-remediation-guidance.md) is the next-step surface when
  this lane stops at basis mismatch, wrong world, wrong handle, or other typed
  non-success posture.
- [Typed Stops And Remediation Guidance](./typed-stops-and-remediation-guidance.md)
  is the action-oriented follow-up guide for those stops.
- [Family Helpers](./family-helpers.md) expose geometry-specific preview,
  runtime-route, and truth-view helper verbs that lower onto this same
  signal-facing orchestration surface after declaration progression succeeds
  and checked envelope truth has been re-established.
- [Declaration Entry Orchestration](./declaration-entry-orchestration.md)
  remains the generic declaration-entry front door through the envelope
  ceiling.

Use declaration signal compatibility when you need the retained compatibility
artifact directly. Use this feature when you want Query to compose that
compatibility truth into one signal-facing next-step result.

## Inspection And Debugging

Use the bound orchestration artifact when you need the signal-facing retained
surface:

- `class()`
- `signal_execution_family()`
- `basis_families()`
- `handle_identity_digest()`
- `operating_context_identity_digest()`
- `declaration_digest()`
- `progression_digest()`
- `route_plan_digest()`
- `receipt_digest()`
- `envelope_digest()`

Use the proof-visible lane when you need to know exactly why the orchestration
stopped:

- `request()`
- `outcome()`
- `witness_checks()`
- `narrowing_decisions()`
- `orchestration_digest()`
- `linked_artifacts()`

Use the ordinary lane when you want the compact public result but still need
the checked-topology link:

- `orchestrate_signal_compatibility_outcome(...)`

Use the recovery lane when the app now needs one typed repair answer:

- `recover_from_outcome(...)`
- `recover_from_signal_compatibility_checked(...)`
- `recover_from_signal_compatibility_proof(...)`

## Anti-Patterns

- treating `Compatible` as if continuation was already prepared
- treating `Prepared` as if continuation was already executed
- recomputing signal compatibility manually after Query already returned
  retained compatibility truth
- lowering helper progression by hand inside app code instead of reusing the
  checked-envelope seam before entering this feature
- flattening `BasisMismatch`, `WrongWorld`, and `WrongHandle` into generic
  "signal not ready"
- using this feature when you actually need explicit continuation execution

## Current Limits

- this feature does not execute Signal work
- this feature does not execute continuation work
- this surface currently supports signal-facing orchestration from retained
  signal compatibility input only
- grouped or neighborhood signal orchestration is not part of this surface
- this feature reuses bridge requests only for continuation preparation, not
  for later execution

## Related Docs

- [Declaration Signal Compatibility](./declaration-signal-compatibility.md)
- [Continuation Pipeline](./continuation-pipeline.md)
- [Ordinary Outcomes](./ordinary-outcomes.md)
- [Typed Stops And Remediation Guidance](./typed-stops-and-remediation-guidance.md)
- [Family Helpers](./family-helpers.md)
- [Declaration Entry Orchestration](./declaration-entry-orchestration.md)
- Runtime-Installed Domain Handles
- [Domain Capabilities](./README.md)
