# Walks in installed invariants

Use an invariant walk when a rule must inspect a dependency history, hierarchy,
or other graph condition beyond a fixed number of hops. A walk follows one
stored relation and returns each reachable entity with its minimum distance
from the starting roots. Cycles and duplicate paths are allowed.

## Calling the API

Import the installed APIs from
`worth_query_host::facade::application_invariants`. Resolve the relation binding
with the schema resolver supplied to the invariant factory. Its source and
target must have the same entity kind.

| Read | Call |
| --- | --- |
| Walk from one root | `view.walk(&binding, direction, &root)` |
| Walk from several roots | `view.closure(&binding, direction, &roots)` |
| Test forward reachability | `view.reaches(&binding, &from, &to)` |

```rust,ignore
let history = view.walk(&previous, RelationDirection::Forward, &tip)?;
for step in &history {
    inspect(step.entity(), step.distance());
}
let depends = view.reaches(&previous, &later, &earlier)?;
let shared = view.closure(&previous, RelationDirection::Forward, &roots)?;
```

`Forward` follows source to target; `Reverse` follows target to source. If a
feature points to its previous feature, forward follows history toward earlier
features. `RelationDirection` is also exported by the declaration facade. It
replaces `ApplicationQueryRootPathDirection` with the same meanings and wire
encodings.

Roots appear once at distance zero. Other entities appear once at their minimum
distance from any root. Visits are ordered by distance, then native entity
identity. `walk.entities()` yields the members; `walk.relations()` yields every
matching physical edge, including self edges, back edges and edges that did
not discover a new entity.

Reachability is reflexive for a valid local entity. `reaches` currently performs
a complete forward walk before answering, even for a reflexive query. Failure
to complete returns an error rather than `false`.

## Preparing the rule's read scope

Preparation must identify every affected root from the domain's relationships.
A deep edit may leave the root untouched. Each preparation walk admits every
entity and physical edge it encounters. Evaluation checks subsequent reads
against this frozen scope.

For example, an active group may forbid reachable blocked nodes. Preparation
can find the group's root through its authored membership relation and walk
its path relation. Evaluation can then read the path again:

```rust,ignore
let reachable = view
    .walk(&self.next, RelationDirection::Forward, &root.entity)
    .map_err(access)?;
for step in &reachable {
    checkpoint(budget)?;
    if view.field(&self.blocked, step.entity()).map_err(access)? == Some(true) {
        let blocked = view.field(&self.key, step.entity()).map_err(access)?
            .ok_or_else(|| failure("blocked key absent"))?;
        return Ok(WorthQueryApplicationInvariantVerdict::Violation(Refusal {
            root: root.key,
            blocked,
            distance: step.distance() as u64,
        }));
    }
}
```

The [rule callback](../../../worth-query-execution/src/domain_computation/primary_graph/tests/application_attempt/invariant_walks/rule/evaluation.rs)
and [schema](../../../worth-query-execution/src/domain_computation/primary_graph/tests/application_attempt/invariant_walks/schema.rs)
show the surrounding bindings and preparation. The membership relation and
walked path serve different purposes: finding a root does not by itself prepare
all of its descendants.

Declare the required entity, field and relation reads as for other invariant
reads. Complete traversal does not supply a domain's root-selection policy.
Native relation laws can require acyclicity when the domain needs it; walks
also work on cyclic graphs.

## Identity, cost and failures

Use identities from the view being read. `proposed()` and `committed()` describe
different sides of a validation. A handle from another schema binding,
candidate, posture or branch is not a traversal grant. Keeping a walk alive
does not make its identities valid in a later candidate.

Native traversal enforces visibility, kind access and the original request's
work and memory limits. Query records actual acquisitions and accounts for its
prepared-scope storage with the original invariant budget. Checked callbacks
must also account for their own computation and owned storage.

A returned walk owns its native result and retains its allocation payer.
Borrowed iteration creates typed values without another graph read or another
result collection. Drop the result when its facts are no longer needed; other
owners may still retain the same payer.

There is no fixed depth limit. A successful walk exhausts its frontier;
cancellation, deadline or resource exhaustion returns no partial result.
Propagate that refusal rather than returning an empty walk or a false
reachability answer.

`planner.reads()` reports preparation acquisitions. `context.reads()` includes
preparation and the current evaluation. Acquired adjacency lists, relation
records and entity metadata remain counted when a later read fails. Counts
neither grant access nor replace a budget.

Inspect `WorthQueryInvariantAccessDenial::kind()` to distinguish foreign
binding/view, undeclared access, wrong kind and unprepared scope from work,
memory and interruption failures. `Interrupted` preserves cancellation versus
deadline. `MemoryAdmission` carries the original native allocation refusal.

These methods currently belong to installed invariant views and follow one
stored endorelation. Snapshot projections, operation readers and declared
queries have separate read APIs.

See [invariants and read observations](invariants-and-read-observations.md),
[relation laws](../modeling/relation-laws.md), and
[build an application](../../../../../../docs/build-an-application.md).
