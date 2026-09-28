# Choosing The Right Surface

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../../foundations/ordinary-application-front-door.md).

These pages help you choose the right Query surface before you start wiring
code.

Use this section when you already know the general problem you are solving, but
the neighboring Query docs feel close together.

## Table Of Contents

- [Start Here](#start-here)
  The shortest chooser pages for common Query surface overlaps.
- [Quick Rules](#quick-rules)
  One-line heuristics for picking the right lane.
- [Related Docs](#related-docs)
  Feature references behind the chooser pages.

## Start Here

- [Binding Vs Orchestration Vs Helpers](./binding-vs-orchestration-vs-helpers.md)
  Choose between explicit next-input binding, declaration-entry lowering, and
  family-native helper ergonomics.
- [Typed Stops And Remediation Guidance](../typed-stops-and-remediation-guidance.md)
  Interpret a typed stop and choose the next ordinary application action.
- [Grouped Authoring Vs Grouped Products Vs Grouped Contributions](./grouped-authoring-vs-grouped-products-vs-grouped-contributions.md)
  Choose between defining one neighborhood meaning, reading grouped route or
  envelope truth, and composing shared or member-local contributions.
- [Signal Compatibility Vs Continuation Pipeline](./signal-compatibility-vs-continuation-pipeline.md)
  Choose between freezing signal-facing eligibility and preparing the next
  continuation step.
- [Live View Vs Subscription](./live-view-vs-subscription.md)
  Choose between retained live surface and subscription declaration family.
- [Inspection Vs Cross-Runtime Explanation](./inspection-vs-cross-runtime-explanation.md)
  Choose between `workspace.inspections()?.inspect`, `CausalInspection`, and explanation
  contributions.
- [Projection Consumption Vs Inspection](./projection-consumption-vs-inspection.md)
  Choose between receipt-first projection facts and general inspection.

## Quick Rules

- choose binding when you need Query to select or deny the next explicit input
- choose orchestration when you already know the declaration input and want
  Query to lower it through the declaration-entry pipeline
- choose helpers when you want family-native ergonomics over the same canonical
  surfaces
- choose readiness before a run, inspection for retained evidence, and
  remediation guidance after a typed stop
- choose grouped authoring when the group itself is part of the meaning
- choose grouped products when you need grouped route, receipt, or envelope
  artifacts
- choose grouped contributions when the grouped neighborhood also carries
  shared or member-local contribution authoring

## Related Docs

- [Domain Capabilities](../README.md)
- [Typed Binding Pipeline](../typed-binding-pipeline.md)
- [Declaration Entry Orchestration](../declaration-entry-orchestration.md)
- [Family Helpers](../family-helpers.md)
- [Typed Stops And Remediation Guidance](../typed-stops-and-remediation-guidance.md)
