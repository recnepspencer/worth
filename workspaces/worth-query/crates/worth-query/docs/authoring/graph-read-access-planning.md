# Graph Read Access Planning

## What This Feature Is

Graph read access planning is Query's internal admission step for proving that
an installed application read has the required access structures, a bounded
cost posture, and available capacity. Application authors declare a typed read
shape. Query derives and consumes one plan for current, continuation,
historical, preview, and live execution.

Use the inspection surfaces when you need to understand why an installed read
was admitted, denied, or assigned a particular cost posture. Do not construct a
planning input or invoke raw plan review from application code.

## Why You Use It

- Detect unsupported adjacency, predicate, ordering, traversal, or live
  maintenance requirements before execution.
- Prevent caller-owned graph loops and hidden N+1 work.
- Bind the admitted plan to the exact installed query graph, branch, basis,
  capacity reservation, and managed session.
- Expose honest cost and work counters without making inspection executable.

## Stable Entry Points

Application authors use:

- typed application-query declarations from `worth_query_declaration`;
- installed-query inspection through `worth_query_host::facade::domain`;
- ordinary host query preparation and execution; and
- mutation-handler `DecisionReader` field and relation methods for declared,
  dependency-tracked decision reads; and
- terminal/publication inspection for actual work counters.

The admission facade exposes read-only requirement, cost, budget, inventory,
and review vocabulary. Raw planning input, the review transition, and admitted
plan construction are hidden one-way integration seams. Compiler tests enforce
that ordinary consumers cannot import the retired public planning constructors
or the monolith admitted application-query plan.

## Core Mental Model

The installed read graph is the semantic source. Admission derives its access
requirements rather than accepting a caller-authored list.

```text
installed canonical read graph
  -> derived requirement set
  -> runtime inventory match
  -> structural cost and byte budget
  -> read-only review
  -> capacity-reserved admitted graph-work plan
  -> session-owned execution
  -> terminal consumption and release evidence
```

The review explains admission. The admitted plan is move-only authority. The
review cannot execute, and a plan from another runtime, installation
generation, query, branch, basis, or session cannot substitute.

## How It Executes

1. Installation binds one canonical read graph to the installed query.
2. Obligation selection selects its `GraphRead` row.
3. Admission derives exact access requirements from the bound graph.
4. Query matches those requirements against the current graph-index inventory.
5. Cost and budget evaluation produces an admitted or typed denied posture.
6. Query reserves provider capacity and seals one graph-work plan.
7. The managed session consumes that plan through its private read port.
8. The read terminal reports actual work and exact capacity/basis release.

Every application-query lane enters at step 1. A live or historical lane does
not have its own planner.

Installed mutation handlers use the same owner-tracked graph truth through
`DecisionReader`. `field`, `relations_from`, and `related_one` enforce the
operation's declared read permissions, runtime and basis affinity, endpoint
availability, relation cardinality, and finite traversal work. Empty adjacency
is still a dependency. `related_one` reports missing or multiple targets instead
of turning either condition into an empty result, and exhausted traversal never
returns a successful prefix. `mutation_target` can carry an entity observed by
that completed decision read set into the candidate phase without adding a
duplicate scalar identity field.

## Small Example

Application code declares the read shape and executes the installed query:

```rust
let activity = bank.query::<AccountActivity>(
    &principal,
    AccountActivityInput { account, page },
    AccessPurpose::AccountServicing,
).await?;
```

The host resolves the installed query, derives and admits its graph-read plan,
opens the managed session, and returns only the shaped result.

## Real Example

For a query with a root predicate, one relation traversal, descending ordering,
and a bounded result shape, the derived requirements can include:

| Requirement | Reason |
|---|---|
| `directional_adjacency` | traverse the installed relation in its declared direction |
| `predicate_support` | apply the installed root predicate before widening work |
| `ordering_support` | produce the declared order without caller sorting |
| `traversal_workset` | bound frontier work |
| `visited_set` and `dedup_set` | avoid repeated work when paths converge |
| `result_buffer` | account for retained result memory |

The Bank current, continuation, historical, preview, and live journeys execute
the same installed planning meaning. Lane-parity tests prove the graph meaning
is equal while attempt plans, sessions, and bases remain distinct.

## How It Relates To Other Features

- [Read Composition](read-composition.md) owns typed query and result-shape
  declaration.
- [Graph Touch Obligation Authority](graph-touch-obligation-authority.md) owns
  the installed `GraphRead` requirement and required terminal.
- [Canonical Graph Obligation Progression](../domain-capabilities/canonical-graph-obligation-progression.md)
  owns selection, plan, session, owner execution, and publication.
- Generic workspace reads preserve their existing access-accountability
  machinery, but they are not a second application-query plan.

## Inspection And Debugging

Inspect:

- canonical installed read-graph identity;
- derived requirement-set identity and rows;
- graph-index inventory probes and match results;
- structural cost, byte estimate, budget class, and admission posture;
- reserved plan, session, branch, and basis identities;
- traversal, projection, delivery, and live-maintenance counters; and
- capacity and basis release receipts.

The important no-N+1 claim is an actual terminal counter, not a promise in a
helper name. Adding unrelated graph records, grants, or consumers must not
widen ordinary work except along the declared work axis.

## Anti-Patterns

- Constructing `WorthQueryCanonicalGraphReadPlanningInput` in application code.
- Calling the raw review transition or executing a review object.
- Importing the monolith admitted graph-read plan.
- Giving one lane a direct Relational graph handle.
- Falling back to per-row neighbor queries after admission denies.
- Treating a budget denial as permission to increase an arbitrary limit.
- Recomputing canonical digests or scanning registries on the warm path.

## Current Limits

- A shape may require persistent indexing, paged streaming, async
  materialization, store support, or domain capability registration instead of
  ordinary inline execution.
- The current application progression does not add durability or restart
  recovery.
- Multiple branch heads and concurrent branch writers remain outside the
  current contract.

## Related Docs

- [Read Composition](read-composition.md)
- [Graph Touch Obligation Authority](graph-touch-obligation-authority.md)
- [Canonical Graph Obligation Progression](../domain-capabilities/canonical-graph-obligation-progression.md)
- [Provider Sessions And Decision Read-Sets](../domain-capabilities/provider-sessions-and-decision-read-sets.md)
