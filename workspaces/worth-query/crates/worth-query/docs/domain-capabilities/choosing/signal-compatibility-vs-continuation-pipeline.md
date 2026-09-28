# Signal Compatibility Vs Continuation Pipeline

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../../foundations/ordinary-application-front-door.md).

## What This Page Helps You Choose

Use this page when you have retained declaration truth and need to choose
between:

- freezing signal-facing eligibility
- preparing the next continuation artifact

These surfaces are adjacent, but they answer different questions.

## Why You Use It

- avoid preparing continuation work when you only need compatibility posture
- avoid using compatibility as if it already prepared execution
- keep signal-facing and continuation-facing jobs separate

## Surfaces Compared

- [Declaration Signal Compatibility](../declaration-signal-compatibility.md)
- [Signal Compatibility Orchestration](../signal-compatibility-orchestration.md)
- [Continuation Pipeline](../continuation-pipeline.md)

## Core Mental Model

Think of the flow this way:

1. declaration signal compatibility: "is this retained declaration story
   structurally eligible for later Signal-backed derived execution?"
2. signal compatibility orchestration: "can Query answer the next signal-facing
   question directly, including optional preparation?"
3. continuation pipeline: "given retained continuation-ready truth or an
   explicit continuation-binding request, can Query prepare one continuation
   artifact and optionally execute it?"

The signal surfaces are about compatibility and signal-facing next steps. The
continuation pipeline is about actual continuation preparation and execution.

## How To Choose

Choose **declaration signal compatibility** when:

- you need the retained compatibility artifact itself
- you care about execution family, basis-family requirements, dependency
  aspects, or produced aspects

Choose **signal compatibility orchestration** when:

- you want a higher-level signal-facing next-step answer
- the result may stop at `Compatible` or advance into `Prepared`
- you want one ordinary, checked, or proof-visible lane over that signal-facing
  decision

Choose **continuation pipeline** when:

- the next job is preparing one continuation artifact
- you are starting from a retained envelope target or an explicit continuation
  binding request
- you need retained bridge, basis, workspace, and readmission posture
- you may optionally execute the prepared continuation

## Small Example

Use signal compatibility to freeze compatibility:

```rust
let compatibility = handle.signal_compatibility_checked(subject);
```

Use signal compatibility orchestration to ask the next signal-facing question:

```rust
let result = handle.orchestrate_signal_compatibility_checked(input);
```

Use the continuation pipeline when you are ready to prepare or execute the next
continuation step:

```rust
let prepared = handle.prepare_continuation_from_target_checked(request);
```

## Real Example

If your app has an envelope-backed declaration and wants to know whether it can
support derived execution later:

- choose declaration signal compatibility when you need the precise retained
  compatibility artifact
- choose signal compatibility orchestration when the UI wants one signal-facing
  answer such as "compatible", "prepared", or one typed stop
- choose the continuation pipeline when you are actually preparing the next
  continuation step and need basis/readmission truth plus optional execution;
  that can start from a retained target or from explicit continuation context

## How It Relates To Other Features

- [Declaration Bridge Continuation Routing](../declaration-bridge-continuation-routing.md)
  is one of the lower-authority surfaces continuation preparation may depend on.
- [Typed Stops And Remediation Guidance](../typed-stops-and-remediation-guidance.md) can preserve different next-step
  actions for signal stops vs continuation stops.
- [Ordinary Outcomes](../ordinary-outcomes.md) provides the compact result lane
  for orchestration and continuation entry points.

## Inspection And Debugging

Use declaration signal compatibility when you need:

- execution family
- basis families
- dependency aspects
- produced aspects

Use signal compatibility orchestration when you need:

- one signal-facing transcript or checked stop
- compatibility-vs-prepared distinction

Use continuation proof when you need:

- basis posture
- evidence strength
- degraded recovery posture
- readmission and execution-facing stop context

## Anti-Patterns

- treating signal compatibility as if it already prepared a continuation
- treating continuation preparation as if it were only a signal eligibility
  check
- skipping the signal-facing orchestration lane when your app wants one compact
  next-step answer

## Current Limits

- signal compatibility does not execute `worth-signal`
- signal compatibility orchestration does not itself execute continuation
- continuation preparation does not replace the declaration-side compatibility
  artifact

## Related Docs

- [Declaration Signal Compatibility](../declaration-signal-compatibility.md)
- [Signal Compatibility Orchestration](../signal-compatibility-orchestration.md)
- [Continuation Pipeline](../continuation-pipeline.md)
- [Typed Stops And Remediation Guidance](../typed-stops-and-remediation-guidance.md)
