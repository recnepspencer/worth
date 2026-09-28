# Retained Artifact To Next Step

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../../foundations/ordinary-application-front-door.md).

## What This Workflow Is

This workflow starts from retained Query truth you already have and moves it to
the next explicit public step.

Use it when you already hold progression, route, receipt, or envelope truth and
do not want to replay the whole declaration-entry path by hand.

## Why You Use It

- keep retained artifacts as the source of truth for later steps
- move forward without rebuilding earlier declaration-entry inputs manually
- choose between compact product-target orchestration and explicit binding
- preserve aspect fit, specificity, stale, and rebind-required posture as typed
  outcomes

## Stable Entry Points

- `plan_routes_from_progressed(...)`
- `receipt_routes_from_progressed(...)`
- `envelope_routes_from_progressed(...)`
- `bind_route_from_target(...)`
- `bind_receipt_from_target(...)`
- `bind_envelope_from_target(...)`
- `bind_continuation_from_target(...)`
- their matching `..._checked(...)`, `..._proof(...)`, and `..._outcome(...)`
  variants

## Core Mental Model

There are two common jobs here:

1. move one retained artifact straight into the next product
2. ask Query to bind one retained artifact into the next explicit input

Choose orchestration when the next product is obvious. Choose binding when
Query still needs to decide or deny the next explicit input.

## How It Executes

Progression can move forward into:

- route
- receipt
- envelope

Envelope truth can move forward into:

- signal compatibility
- continuation binding and preparation

The key rule is that later surfaces should reuse retained targets instead of
pretending the earlier declaration work never happened.

For post-mutation consumption, the same rule applies one step earlier: if you
already have a retained batch-write receipt and the next step needs the
matching inspection plus one exact retained derived artifact, use
`materialize_batch_write_artifact_binding(...)` so Query owns that retained
package instead of caller code reopening separate inspection and materialize
steps by hand.

## Small Example

```rust
let progressed = handle.declare_review_and_progress(
    geometry_session.publish_boundary_change_for_active_face()?,
)?;

let envelope = handle.envelope_routes_from_progressed(progressed)?;
```

Use this when the next public product is obvious and you want the shortest
retained-artifact path.

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

let _ = binding.outcome();
let _ = binding.aspect_fit_report();
let _ = binding.resolved_target();
```

Use the binding lane when Query still needs to prove that one retained target
is the right subject for the next continuation step.

## How It Relates To Other Features

- [Typed Binding Pipeline](../typed-binding-pipeline.md) is the authoritative
  reference for retained-target binding requests and outcomes.
- [Declaration Entry Orchestration](../declaration-entry-orchestration.md)
  provides the compact product-target orchestration variants.
- [Continuation Pipeline](../continuation-pipeline.md) consumes retained
  continuation binding truth after this workflow.

## Inspection And Debugging

Use the proof-visible binding lane when you need:

- witness checks
- aspect-fit reasoning
- narrowing decisions
- the exact resolved target

Use the checked or proof orchestration lane when you need:

- the retained route, receipt, or envelope step records
- the product-target orchestration digest

## Anti-Patterns

- rebuilding raw declaration-entry inputs from scratch when progression, route,
  receipt, or envelope truth already exists
- using binding when the next product is already obvious
- using compact orchestration when the main question is still candidate
  selection

## Current Limits

- the retained-target ladder is intentionally narrow and typed
- continuation still requires its own preparation or signal-facing lane after
  binding succeeds

## Related Docs

- [Typed Binding Pipeline](../typed-binding-pipeline.md)
- [Declaration Entry Orchestration](../declaration-entry-orchestration.md)
- [Envelope To Signal Or Continuation](./envelope-to-signal-or-continuation.md)
