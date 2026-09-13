# Granular Live Invalidation

## What This Feature Is

Granular live invalidation updates an already-live Query result from the exact
truth change that affected it. Use it when a committed field, partition, or
record change should update one projection, collection membership decision,
ordering key, group, or bounded window without rerunning unrelated queries.

## Why You Use It

- Reprice one risk projection after a single curve bucket changes.
- Remove or reorder one portfolio row without rebuilding the collection.
- Share one maintenance execution across consumers while applying each
  consumer's current purpose, disclosure, lease, and backpressure policy.

## Stable Entry Points

Import the maintained surfaces from `worth_query::facade::domain`:

- `bind_primary_runtime_granular_invalidations`
- `maintain_primary_runtime_granular_invalidations`
- `maintain_primary_runtime_granular_batch`
- `maintain_primary_runtime_granular_collection_batch`
- `bind_shared_primary_runtime_granular_invalidations`
- `maintain_shared_primary_runtime_granular_batch`

The application runtime supplies the matching installation through
`granular_invalidation_installation()`. The binding and each delivery batch are
runtime-affine: a stale or foreign runtime is denied before Query reads or
publishes.

## Core Mental Model

Relational owns the committed truth change. Runtime Bridge matches that change
to installed semantic dependencies. Signal performs only the derived work that
is actually required. Query then decides which query roles changed and applies
the result-shaped consequence.

```text
committed truth
    -> installed Bridge correspondence
    -> optional performed Signal consequence
    -> Query impact roles
    -> Query-owned maintenance
    -> authorized consumer publication
```

A direct truth delivery and a performed Signal delivery are different facts.
A comparator may suppress Signal work while the truth change remains real.
Conversely, Query may use direct truth for a local field patch without claiming
that Signal executed.

Product-bound delivery adds one identity rule: every existing patch granule
carries the committed product branch and composite commit occurrence that
authorized it. Query admits delivery only against the matching selected
product. It never publishes a half-current view assembled from a new
Relational head and an old Signal basis. A retained Signal component may still
evaluate a Relational-only publication without advancing its own component
basis; the delivery evidence records those as separate contacts.

Matching, nonmatching, delayed, and slow-consumer paths retain the existing
bounded queue and backpressure policy. Exhaustion after `Performed` delays or
retains the committed artifact for typed projection retry; it does not
reclassify the publication or rerun authoritative effects. Closing a live
consumer releases its product observation, evaluation binding, and queued
delivery custody.

## How It Executes

1. Install Query semantic dependencies and their Bridge correspondence.
2. Promote a settled projection or collection into a live owner.
3. Bind that owner to the current primary-runtime invalidation installation.
4. Observe a committed host operation and take its granular batch.
5. Query revalidates runtime, source snapshot, dependency, locality, policy,
   and consumer authority.
6. Query previews the patch against its retained current result.
7. Publication succeeds before the retained result advances.

A failed or stale attempt does not consume queued live delivery state and does
not advance the comparison baseline.

## Small Example

```rust
use worth_query::facade::domain::{
    bind_primary_runtime_granular_invalidations,
    maintain_primary_runtime_granular_invalidations,
};

let binding = bind_primary_runtime_granular_invalidations(
    &live_projection,
    application.granular_invalidation_installation(),
);

let outcome = maintain_primary_runtime_granular_invalidations(
    &live_projection,
    &mut workspace,
    &binding,
    &mut accepted_observation,
)?;
```

This is the smallest honest form because the runtime-owned observation still
carries direct truth, optional performed Signal evidence, and the exact source
read basis. Do not unpack it into raw field events first.

## Real Example

For an ordered live portfolio, retain collection state and use the collection
entry point:

```rust
use worth_query::facade::domain::{
    bind_primary_runtime_granular_invalidations,
    maintain_primary_runtime_granular_collection_batch,
};

let installation = application.granular_invalidation_installation();
let binding = bind_primary_runtime_granular_invalidations(
    &live_portfolio,
    installation,
);
let batch = accepted_observation.take_granular_invalidation_batch();

let outcome = maintain_primary_runtime_granular_collection_batch(
    &live_portfolio,
    &mut retained_collection_window,
    &mut workspace,
    &binding,
    batch,
)?;
```

A projected-value change can remain a local field patch. A membership change
can remove or insert one row. An ordering or window change can update the
affected row and the bounded replacement neighbor. The retained collection is
advanced only after Query publishes the performed patch.

## How It Relates To Other Features

- Pair it with live projection promotion; it does not create the initial result.
- Shared live owners execute maintenance once and publish per current consumer.
- Runtime reconstruction requires explicit Bridge reconstitution and Query
  source rebinding. Old batches and bindings are rejected.
- Replay and reconstruction evidence stay in certification lanes. Ordinary
  maintenance consumes current runtime products only.

## Inspection And Debugging

Inspect the owner-separated observations:

- Bridge counters explain candidate lookup, rejection, widening, and delivery.
- Signal realized counters report performed execution, not predicted work.
- Query admission counters report selected deliveries and semantic roles.
- Query maintenance counters report projection, membership, ordering, grouping,
  window, authorization, and publication work.

If a change produces no patch, first distinguish irrelevant, already settled,
comparator-suppressed, stale, and unauthorized outcomes.

Match the public outcome rather than inferring success from an empty patch:

- `NoRelevantChange` reports irrelevant, duplicate, already-settled, or
  suppressed work without claiming publication.
- `Performed` carries Query-owned deliveries and the admission and maintenance
  counters for work that actually ran.
- `ForeignPrimaryRuntime`, admission, execution, maintenance, and publication
  denials identify the boundary that must be rebound or repaired. Do not retry
  one of these denials by copying identities into a new request.

## Anti-Patterns

- Do not treat a Bridge delivery as proof that Signal executed.
- Do not feed raw CDC or copied aspect/scope tuples directly into Query.
- Do not reuse a binding, batch, snapshot basis, or consumer lease after
  reinstall, restore, or rebind.
- Do not rebuild the full collection when the admitted role names a bounded
  field, membership, ordering, group, or window consequence.
- Do not move semantic locality into physical shard or worker identifiers.

## Current Limits

The stable semantic locality model is unscoped, whole partition, and exact
detail. A future semantic hierarchy can extend that model without turning
physical shard or region placement into authority. Granular invalidation
already keeps semantic scope separate from execution placement, so that
expansion does not change the Query maintenance contract.

## Related Docs

- [Region-Scoped Live Invalidation And Stream Contracts](./region-scoped-live-invalidation-and-stream-contracts.md)
- [Bound Projection Lifecycle, Sharing, And Consumer Invalidation](../domain-capabilities/bound-projection-sharing-and-invalidation.md)
- [Cross-Runtime Causal Inspection](../capabilities/cross-runtime-causal-inspection.md)
- [Query Operating Modes](../foundations/query-operating-modes.md)
- [Aspects And Authority Lanes](../modeling/aspects-and-authority-lanes.md)
- [Downstream Runtime Integration](../foundations/downstream-runtime-integration.md)
