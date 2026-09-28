# Consumption Cost Evidence

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Consumption cost evidence lets you inspect how much structural work Query
actually performed while resolving, binding, executing, and settling an
installed operation. Use it when a consumer needs proof of bounded work rather
than elapsed-time measurements or optional diagnostics.

## Why You Use It

- Verify that adding unrelated domains, views, rows, or consumers does not make
  a bound operation more expensive.
- Explain an early denial and confirm that later planning, execution, access,
  or lifecycle work stayed at zero.
- Export Query's exact operational rows into Foundational's shared performance
  receipt vocabulary for reporting or certification support.

## Stable Entry Points

- `WorthQuerySettledDomainProjection::consumption_cost_snapshot()`
- `WorthQuerySettledWorkflowProjection::consumption_cost_snapshot()`
- `WorthQueryConsumptionCostSnapshot::{rows, row}`
- `WorthQueryConsumptionCostSnapshot::materialize_foundational_receipt()`
- Boundary-local `counters()` methods on lookup, binding, support, execution,
  native access, dependency impact, sharing, lifecycle, invalidation, window,
  patch, Relational lowering, Bridge correspondence, and Signal conditional
  outcomes

The settled snapshot is stable for installed direct and workflow operations.
It does not replace the local counters attached to later native reads, live
refreshes, or delivery results.

## Core Mental Model

Query owns the measurement. Each boundary counts only the work it performs and
attaches that snapshot to its result or denial. A settled consumption snapshot
then seals the counters accumulated by the installed-operation journey.

Foundational owns the shared names and receipt/report vocabulary. Exporting a
Query snapshot derives a counter-backed Foundational receipt; it does not turn
that receipt into Query execution, lifecycle, or consumer authority.
Read-only execution rows remain authoritative observation work; only operations
whose admitted commit posture advances truth are classified as authoritative
mutation. The mixed settled journey is traversal-local, not a point lookup.

Counter names identify both the boundary and the operation, such as
`query.lookup.indexed_operation_lookups` or
`query.dependency.compiled_dependency_count`. A zero is meaningful: it says
that boundary did no such work.

## How It Executes

1. The installed operation is resolved with an indexed lookup.
2. Binding, support admission, execution, dependency compilation, and optional
   native binding each retain their own exact counters.
3. Publication consumption settles the projection and seals those rows in a
   `WorthQueryConsumptionCostSnapshot`.
4. Later work keeps separate local evidence. For example, each native value
   access and each live refresh carries its own counters.
5. A caller may explicitly materialize the settled snapshot as a Foundational
   counter-backed receipt. Rich report materialization remains a separate
   support operation and cannot modify the Query snapshot.

## Small Example

```rust
let settled = executed
    .publish()?
    .consume(consumer, requested_facts)?
    .settle()?;

let costs = settled.consumption_cost_snapshot();
let lookup = costs
    .row("query.lookup.indexed_operation_lookups")
    .expect("settled installed operations retain lookup work");

assert_eq!(lookup.observed_count(), 1);
```

This is the smallest honest use: inspect a snapshot minted by a real settled
operation rather than assembling rows from copied diagnostics.

## Real Example

```rust
let selected = settled.native_value(&declared_key, row_index)?;
assert_eq!(selected.counters().indexed_accesses, 1);
assert_eq!(selected.counters().refinement_checks, 1);
assert_eq!(selected.counters().row_scans, 0);

let costs = settled.consumption_cost_snapshot();
let receipt = costs.materialize_foundational_receipt()?;

assert_eq!(receipt.counter_rows().len(), costs.rows().len());
assert_eq!(
    receipt.bundle().counter_specs().len(),
    costs.rows().len(),
);
```

The native read reports its own point-access work because it occurs after
settlement. The settled snapshot reports the earlier operation journey. The
Foundational receipt is derived reporting evidence; the original `settled`
capability remains the object that authorizes Query work.

## How It Relates To Other Features

- Pair it with declaration-indexed native access to prove `O(1)` access per
  admitted key and `O(k)` work for `k` requested keys.
- Pair it with dependency impact and shared live execution to distinguish
  affected dependency breadth from admitted lease fan-out.
- Conditional results separately count request admission, contract lookup,
  dependency observation and comparison, condition resolution, output-version
  reads, runtime dependency capture, application, semantic classification,
  comparator work, compute, reverted-clean outcomes, semantic change,
  deferral, reuse, and delivery.
- Relational and Runtime Bridge counters prove authoritative target lowering,
  admitted correspondence matches, and actual Signal fan-out without making
  Query a second truth authority.

## Inspection And Debugging

Start with the counter snapshot on the result or denial closest to the failed
boundary. Use the settled snapshot for the completed installed-operation
journey. Materialize a Foundational receipt or report only when another tool
needs shared performance vocabulary.

When checking a bound, grow unrelated state deliberately and compare exact
snapshots. Do not infer bounded behavior from similar response times.

## Anti-Patterns

- Timing an end-to-end request and calling that a semantic breadth proof.
- Recomputing counters from logs, diagnostics, collection lengths, or report
  rows.
- Aggregating unrelated source execution into a downstream access boundary.
- Treating a Foundational claim, policy receipt, counter-backed receipt, or
  materialized report as Query authority.
- Collapsing condition checks, comparator checks, compute, and delivery into a
  single "evaluated nodes" count.

## Current Limits

- A settled snapshot covers the installed direct or workflow journey through
  settlement; later accesses and live deliveries remain boundary-local.
- Rich Foundational reports are deliberately materialized off the ordinary hot
  path.
- Counter evidence proves structural work performed. It does not promise wall-
  clock latency, scheduler behavior, or hardware-specific throughput.

## Related Docs

- [Runtime-Installed Domains And Operations](./runtime-installed-domains.md)
- [Conditional Installed Operations](./conditional-installed-operations.md)
- [Bound Projection Lifecycle, Sharing, And Consumer Invalidation](./bound-projection-sharing-and-invalidation.md)
- [Projection Consumption](../capabilities/projection-consumption.md)
