# Projection Consumption Vs Inspection

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../../foundations/ordinary-application-front-door.md).

## What This Page Helps You Choose

Use when you need **facts from a projection or receipt** versus **general retained inspection evidence**.

## When Projection Consumption

- you consume **receipt-first projection facts** from admitted projection/read-composition paths
- truth is anchored to operator receipts and authorized projection narrowing
- policy/tenant narrowing may apply—see [policy narrowing](../../foundations/policy-tenant-and-relationship-proof-narrowing.md)

## When Inspection

- you need workspace **inspect** for per-target retained evidence ([inspection](../../capabilities/inspection.md))
- debugging “what is retained” without going through a specific projection receipt chain
- causal cross-runtime questions → [causal inspection](../../capabilities/cross-runtime-causal-inspection.md), not inspect alone

## Quick Rules

- **Projection consumption** = follow receipts and admitted projection artifacts.
- **Inspection** = `workspace.inspections()?.inspect` retained evidence surface.
- Do not use inspect to stand in for authorized projection when policy masking applies.
- Mutation/write proof → [authoritative mutation evidence](../../capabilities/authoritative-mutation-evidence.md), not generic inspect.

## Related Docs

- [Read composition](../../authoring/read-composition.md)
- [Inspection](../../capabilities/inspection.md)
- [Policy, tenant, and relationship-proof narrowing](../../foundations/policy-tenant-and-relationship-proof-narrowing.md)
- [Authoritative mutation evidence](../../capabilities/authoritative-mutation-evidence.md)
