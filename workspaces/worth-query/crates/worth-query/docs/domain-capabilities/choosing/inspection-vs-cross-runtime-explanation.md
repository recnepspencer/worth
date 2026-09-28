# Inspection Vs Cross-Runtime Explanation

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../../foundations/ordinary-application-front-door.md).

## What This Page Helps You Choose

Use when you need **evidence or explanation** after a run and are mixing up `workspace.inspections()?.inspect`, `CausalInspection`, and **explanation contributions**.

## When Workspace Inspect

- per-target **retained inspection evidence** on the workspace
- “what does Query retain for this target right now?”
- see [inspection](../../capabilities/inspection.md)

## When Cross-Runtime Causal Inspection

- **`CausalInspection` lane**: bind the originating receipt to a
  `ScopedInspectionBasis`, then admit, plan, and materialize that artifact
- `CrossRuntimeCausalExplanation` (reference-only supported; materialized detail advisory)
- temporal wakes, async completions, mixed-cause suppressions, preview remasks,
  replay drift, and resume mismatch
- **Not** `workspace.inspections()?.inspect`—see [cross-runtime causal inspection](../../capabilities/cross-runtime-causal-inspection.md)

## When Explanation Contributions

- domain **declaration posture** for explanation (lower-runtime explanation contributions)
- does not replace causal inspection APIs or general inspection reads

## Quick Rules

- **Inspect** = retained per-target evidence.
- **Causal inspection** = cross-runtime explanation families + admission/materialization.
- **Explanation contributions** = declare how domain attaches explanation posture.
- Durable causal archive / store-backed replay = **deferred** on causal support table.
- If the question starts with "why did this temporal/async runtime event happen
  or get replayed/suppressed/remasked?", choose causal inspection.

## Related Docs

- [Inspection](../../capabilities/inspection.md)
- [Cross-runtime causal inspection](../../capabilities/cross-runtime-causal-inspection.md)
- Lower-runtime explanation contributions
- [Typed stops and remediation guidance](../typed-stops-and-remediation-guidance.md)
