# Live View Vs Subscription

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../../foundations/ordinary-application-front-door.md).

## What This Page Helps You Choose

Use when you need **ongoing updates** but are unsure whether to use the **retained live view surface** or the **subscription declaration family** (sharing, continuation, diagnostics).

## When Live View

- you want a workspace-retained live query plan tied to [live views](../../runtime-surfaces/live-views.md)
- invalidation is driven through live admission and region-scoped neighbors
- consumers read live state from the runtime graph, not only a subscription envelope

## When Subscription

- you declare a subscription family with explicit sharing/continuation contracts
- you need [subscription selection and diagnostics](../../capabilities/subscription-selection-and-diagnostics.md) posture
- activation flows through lower-runtime boundary receipts ([routing doc](../lower-runtime-capability-routing.md))

## Quick Rules

- **Live view** = retained live surface + planner-owned invalidation paths.
- **Subscription** = declaration-scoped family with its own admission matrix (durable replay often **deferred**).
- **Region narrowing** applies to live plans ([region-scoped live](../../runtime-surfaces/region-scoped-live-invalidation-and-stream-contracts.md))—not a substitute for picking subscription vs live.
- Stream delivery is **query-shaped** in both neighborhoods—never assume raw CDC.

## Related Docs

- [Live views](../../runtime-surfaces/live-views.md)
- [Region-scoped live](../../runtime-surfaces/region-scoped-live-invalidation-and-stream-contracts.md)
- [Subscription selection and diagnostics](../../capabilities/subscription-selection-and-diagnostics.md)
- [Query operating modes](../../foundations/query-operating-modes.md)
