# Region-Scoped Live Invalidation and Stream Contracts

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Region-scoped live ties **locality predicates** to live query plans: admit a region-scoped plan, execute region-scoped live changes, and **lower** execution to **query-shaped stream contracts**—not raw CDC event feeds. Region narrowing is **query-planner-owned**; stream lowering delivers projected, admission-class-aware delivery.

## Why You Use It

- narrow live invalidation to entity regions, partitions, or bounded materialization classes
- obtain `RegionScopedLivePlan` with explicit admission and stream-lowering posture
- subscribe with digests that include locality + admission class
- stay honest about durable subscription replay (deferred in subscription matrix)

## Core Mental Model

`live/region_scoped.rs`:

1. `admit_region_scoped_live_plan(live, locality)` — derives semantic basis, scope admission, admission class, stream-lowering admission.
2. `execute_region_scoped_live_change` — applies change under locality budgets (breadth, widening, stream member/window widths).
3. `lower_region_scoped_execution_to_stream_contract` — **query-shaped** stream delivery.

Admission classes (representative): `DetailRegion`, `DetailPartition`, `OrderedCollectionPartition`, `BoundedMaterializationRegion`—each maps to cost postures and stream lowering (e.g. single-slice vs CDC projected patch vs bounded materialization deferred).

```text
LiveQueryPlan + LocalityPredicateContract
  → admit_region_scoped_live_plan
  → execute_region_scoped_live_change
  → lower_region_scoped_execution_to_stream_contract
```

## Main Entry Points

- `admit_region_scoped_live_plan`, `execute_region_scoped_live_change`, `lower_region_scoped_execution_to_stream_contract` (`live/region_scoped.rs`)
- Facade: `exports_foundation.rs` — `RegionScopedLivePlan`, `LocalityPredicateContract`, subscription identity digests
- Tests: `harness/region_live_certification/tests.rs`, `view_shape_live/tests.rs`
- Subscription matrix: `subscription/support/matrix.rs` — durable replay deferred

## Typical Flow

1. Build a `LiveQueryPlan` with a relevance contract compatible with locality derivation.
2. Provide a `LocalityPredicateContract` (scope kind, region/partition semantics).
3. `admit_region_scoped_live_plan` → plan with `RegionScopedSubscriptionIdentity` digest.
4. On invalidation/update: `execute_region_scoped_live_change` within budgets.
5. Deliver updates via lowered stream contract to consumers expecting **query-shaped** patches.

Compare with [live views](live-views.md) for general live surface admission; use [subscription selection](../capabilities/subscription-selection-and-diagnostics.md) when choosing subscription vs retained live.

This locality-aware stream surface is not the installed-operation consumer
invalidation authority. A region-scoped plan narrows query-shaped live stream
delivery. An installed bound projection instead compiles operation semantics,
retains a managed owner and exact lease, and emits a capability-bound delta
that must be readmitted before a consumer consequence is attached. Neither
surface can be reconstructed from the other's digest or locality metadata.

## How It Relates

- [Live views](live-views.md) — general live admission and workspace live API
- [Subscription selection and diagnostics](../capabilities/subscription-selection-and-diagnostics.md) — sharing, continuation, durable replay debt
- [Choosing: live view vs subscription](../domain-capabilities/choosing/live-view-vs-subscription.md)
- [Collections, cursors, ordering](../authoring/collections-cursors-ordering-and-aggregations.md) — ordered collection partition admission class
- [Bound Projection Lifecycle, Sharing, And Consumer Invalidation](../domain-capabilities/bound-projection-sharing-and-invalidation.md) — installed-operation owner, lease, and consequence authority

## Good to Know

- `locality_subscription_digest` hashes subscription + locality + admission class—use for identity, not offset pagination.
- `OrderedCollectionPartition` may use `StreamLoweringCostPosture::CdcPatchWithProjectedDeltas`—still **projected**, not arbitrary CDC.
- `BoundedMaterializationRegion` may defer stream lowering (`BoundedMaterializationDeferred`) per admission path.

## Anti-Patterns

- Treating lowered streams as raw database CDC topics.
- Widening locality without respecting `LocalityWideningBudget` (e.g. `deny_all()` on partition paths).
- Assuming region-scoped subscriptions survive restart with durable replay while matrix rows are deferred.

## Current Limits

| Concern | Status |
|---------|--------|
| Runtime-backed region admit + execute + stream lowering | **Verified** on certified paths |
| Query-shaped stream delivery | **Verified** (not raw CDC) |
| Durable subscription replay / restart metadata | **Deferred** (`subscription/support/matrix.rs`) |
| Unbounded materialization stream lowering | **Deferred** per admission class |

## Related Docs

- [Live views](live-views.md)
- [Subscription selection and diagnostics](../capabilities/subscription-selection-and-diagnostics.md)
- [Live view vs subscription (chooser)](../domain-capabilities/choosing/live-view-vs-subscription.md)
- [Support matrix and admission](../foundations/support-matrix-and-admission.md)
