# Grouped Support And Readiness

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

Grouped support/readiness tells you whether an admitted grouped claim is
supportable before you treat it as a later execution or projection fact.

Use `grouped_support_report(...)` when your app needs to know:

- whether grouped declaration authoring is supported
- whether grouped route, receipt, and envelope projections are supported
- whether grouped contribution composition is supported
- whether the current grouped atomicity, intent, continuity, and shared
  posture claims are supportable

This surface starts from a grouped declaration artifact. In other words:

- declare the grouped neighborhood first
- then ask whether its stronger grouped claims are supportable
- then decide whether to continue into grouped route, receipt, envelope,
  contributions, or orchestration

The important consumer-facing behavior is that shared claims can be too strong
for the retained grouped truth. When that happens, the report preserves it
through:

- `status_for(...)`
- `unsupported_claims()`

That gives you a typed way to distinguish:

- "the neighborhood exists"
- "the neighborhood runs"
- "this stronger shared claim is not supportable as written"

Use this after grouped declaration admission and before later grouped execution
or projection when you want to keep unsupported grouped posture out of later
repair flows.
