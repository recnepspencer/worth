# Binding Vs Orchestration Vs Helpers

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../../foundations/ordinary-application-front-door.md).

## What This Page Helps You Choose

Use this page when you know you need Query to help with the next step, but you
are not sure whether that job is:

- selecting the next explicit input
- lowering one declaration through the declaration-entry pipeline
- using a family-native helper over those same canonical surfaces

The three surfaces are related, but they do different jobs.

## Why You Use It

- avoid binding when you really need orchestration
- avoid orchestration when you really need candidate selection
- avoid helper-local wrappers when the generic surface is the better fit
- choose the lane that preserves the right checked, proof, and recovery story

## Surfaces Compared

- [Typed Binding Pipeline](../typed-binding-pipeline.md)
- [Declaration Entry Orchestration](../declaration-entry-orchestration.md)
- [Family Helpers](../family-helpers.md)

## Core Mental Model

Think of these surfaces as three different questions:

1. binding: "what is the next admissible input from the context or retained
   artifact I already have?"
2. orchestration: "given this declaration input, can Query lower it through
   the declaration-entry pipeline?"
3. helpers: "can I get a family-native call shape over those same canonical
   Query lanes?"

The most important distinction is this:

- binding selects or denies the next explicit input
- orchestration lowers a chosen input
- helpers are ergonomic projections, not a second engine

## How To Choose

Choose **binding** when:

- you have current context or a retained target
- multiple candidates may exist
- aspect fit, world identity, or authority posture may deny the next step
- you want one typed request such as a route, receipt, envelope, or
  continuation input

Choose **orchestration** when:

- you already know the declaration input
- the next job is legality, progression, route, receipt, or envelope lowering
- you want the ordinary, checked, or proof-visible declaration-entry lanes

Choose **helpers** when:

- your app already knows the declaration family
- the generic surface is correct but too mechanical for ordinary use
- you want family-native methods that still lower onto signal compatibility,
  grouped authoring, or contribution-composed orchestration

## Small Example

Use binding when the next job is selecting the next explicit route input:

```rust
let request = WorthQueryRouteBindingRequest::new(
    candidates,
    AttachFaceMaterial::aspect_contract(),
    allowed_sources,
);

let route_input = handle.bind_route_request_from_context(request)?;
```

Use orchestration when the declaration input already exists:

```rust
let envelope = handle
    .declare_review_progress_describe_plan_receipt_and_envelope(trim_request)?;
```

Use helpers when you want a family-native call shape over canonical lowering:

```rust
let progressed = handle
    .geometry_helpers()
    .progress_active_face_selection(face_selection_input)?;

let prepared = handle
    .geometry_helpers()
    .prepare_preview_for_active_face_selection(progressed);
```

## Real Example

If your app has an active face selection and needs a material-edit workflow:

- choose binding when the main risk is selecting the wrong retained target from
  current context
- choose orchestration when the active selection has already been turned into a
  declaration input and you want Query to lower it to an envelope
- choose helpers when your app wants the active-face-selection call shape
  directly and the declaration family is already known
- helper continuation preparation still starts from admitted progression, so
  the helper lane keeps the same retained-proof shape as the generic surface

## How It Relates To Other Features

- [Ordinary Outcomes](../ordinary-outcomes.md) gives all three lanes a compact
  shared result vocabulary where applicable.
- [Typed Stops And Remediation Guidance](../typed-stops-and-remediation-guidance.md) explains what to do next when
  any of these surfaces stop.
- [Orchestration Inventory](../orchestration-inventory.md) is the registry for
  the public orchestration and helper surfaces.

## Inspection And Debugging

Use binding proof when you need:

- candidate comparisons
- aspect-fit reports
- narrowing decisions

Use orchestration proof when you need:

- step records
- stop stage
- materialization and authority publication posture

Use helper parity docs and tests when you need to confirm a helper is only a
projection over the canonical generic lane.

## Anti-Patterns

- using orchestration to guess the next input from loose context
- using binding as if it already lowered the declaration-entry pipeline
- teaching helpers as if they were helper-local runtimes
- skipping the generic surface docs and assuming a helper owns different
  semantics

## Current Limits

- helpers are intentionally family-scoped, not a replacement for the generic
  lanes
- binding does not execute later declaration-entry or continuation work
- orchestration does not resolve user intent or UI selection for you

## Related Docs

- [Typed Binding Pipeline](../typed-binding-pipeline.md)
- [Declaration Entry Orchestration](../declaration-entry-orchestration.md)
- [Family Helpers](../family-helpers.md)
- [Ordinary Outcomes](../ordinary-outcomes.md)
- [Typed Stops And Remediation Guidance](../typed-stops-and-remediation-guidance.md)
