# Workflow Guides

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../../foundations/ordinary-application-front-door.md).

These guides show how real Query tasks fit together across multiple public
surfaces.

Use this section when you already know the feature pages exist, but you want a
short path for one job instead of reading each surface in isolation.

## Table Of Contents

- [Start Here](#start-here)
  Task-first workflow guides by job.
- [Quick Rules](#quick-rules)
  One-line heuristics for choosing the right workflow guide.
- [Related Docs](#related-docs)
  Feature references and older workflow contribution docs.

## Start Here

- [Single Declaration To Envelope](./single-declaration-to-envelope.md)
  Go from one declaration input to one public envelope artifact.
- [Retained Artifact To Next Step](./retained-artifact-to-next-step.md)
  Start from progression, route, receipt, or envelope truth and move forward
  without rebuilding the earlier declaration path by hand.
- [Envelope To Signal Or Continuation](./envelope-to-signal-or-continuation.md)
  Choose whether the next job is signal-facing compatibility or explicit
  continuation preparation and execution.
- [Grouped Neighborhood Workflow](./grouped-neighborhood-workflow.md)
  Author one neighborhood-shaped operation, inspect grouped products, and add
  shared or member-local contributions.
- [Typed Stops And Remediation Guidance](../typed-stops-and-remediation-guidance.md)
  Interpret one ordinary, checked, or proof-visible stop and choose a next
  application action.

## Quick Rules

- start with the single-declaration workflow when one declaration stands on its
  own
- start with the retained-artifact workflow when you already hold progression,
  route, receipt, or envelope truth
- start with the signal or continuation workflow when envelope truth already
  exists and you need the next runtime-facing step
- start with the grouped workflow when the neighborhood itself is part of the
  meaning
- start with remediation guidance when the main question is "what do I do
  next?"

## Related Docs

- [Domain Capabilities](../README.md)
- [Choosing The Right Surface](../choosing/README.md)
- [Typed Binding Pipeline](../typed-binding-pipeline.md)
- [Typed Stops And Remediation Guidance](../typed-stops-and-remediation-guidance.md)
- Preview Inspection And Mutation Planning
- [Runtime-Preflight Workflow Contributions](./runtime-preflight-workflow-contributions.md)
- Workflow Lanes: Common, Checked, Proof, And Raw
