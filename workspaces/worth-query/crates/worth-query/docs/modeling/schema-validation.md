# Schema Validation

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

For laws enforced when an application changes its graph, see
[Relation laws](./relation-laws.md).

Schema validation is Worth Query's legality gate for canonical query bundles.
It confirms that a query's projections, predicates, ordering, traversal, and
result-shape bindings are legal for the schema basis it claims to target, then
freezes a stable validated identity for the bundle.

## Why You Use It

- you need an explicit legality pass before planning or execution
- you want structured rejection reasons instead of broad "invalid query"
  failures
- you need validated query identity to change when schema basis or result shape
  meaning changes
- you want compatibility and widening checks to be enforced by infrastructure

## Stable Entry Points

- `validate_canonical_bundle(...)`
- `ValidatedQueryBundle`
- `QueryValidationReport`
- `QueryValidationCounters`
- `ValidationEvent`
- `ValidationRejectionMatrix`
- `QueryValidationError`
- `ValidationFailureClass`

## Core Mental Model

Validation is not a cosmetic schema check. It is the point where Worth Query
decides whether a canonical bundle is semantically legal for a schema basis and
whether its meaning can be safely frozen for downstream planning.

What validation proves:

- selected aspects exist and are queryable
- ordering fields exist and are orderable
- predicate families are legal for the targeted field and context
- traversal requests respect relation and depth legality
- result-shape bindings do not silently widen or contradict the canonical query
- the declared schema basis is compatible with the bundle being validated

What validation produces:

- a `ValidatedQueryBundle`
- a deterministic validation report and event stream
- a rejection matrix when validation fails
- a validated identity that includes both query meaning and result-shape
  meaning

## How It Executes

1. Authoring and composition produce a canonical query bundle.
2. `validate_canonical_bundle(...)` walks projection, ordering, predicate,
   traversal, and result-shape legality against the requested schema basis.
3. Each successful stage emits a `ValidationEvent`.
4. If all stages succeed, validation freezes canonical compatibility and emits
   `IdentityFrozen`.
5. If any stage fails, validation returns a typed `QueryValidationError` and a
   failure class such as projection, predicate, traversal, or compatibility.

Validation is intentionally earlier than planning. A plan should never have to
guess whether a canonical bundle was legal.

## Small Example

```rust
use worth_query::facade::runtime::validate_canonical_bundle;

let canonical = canonical_bundle;

let validated = validate_canonical_bundle(&canonical, "tasks-table")?;

assert_eq!(
    validated.report().events().last().unwrap(),
    &ValidationEvent::IdentityFrozen {
        query_digest: validated.query_digest().to_string(),
        result_shape_digest: validated.result_shape_digest().to_string(),
    }
);
```

This is the smallest honest example because it shows the contract boundary:
canonical meaning goes in, validated meaning comes out, and identity is frozen
only after legality succeeds.

## Real Example

```rust
use worth_query::facade::runtime::{validate_canonical_bundle, QueryValidationError};

let canonical = workflow_canonical_bundle;

match validate_canonical_bundle(&canonical, "workflow-runtime") {
    Ok(validated) => {
        assert_eq!(validated.report().counters().compatibility_success_count(), 1);
    }
    Err(QueryValidationError::IllegalWorkflowPredicateCapabilityOrContextShape {
        ..
    }) => {
        // Workflow-scoped predicate capability was used against an illegal
        // context shape.
    }
    Err(QueryValidationError::ProjectionWideningDenied { .. }) => {
        // Result-shape or projection binding attempted to widen beyond the
        // canonical query contract.
    }
    Err(other) => panic!("unexpected validation failure: {other:?}"),
}
```

This is where the feature earns its keep: a query can be structurally
well-formed and still be illegal because a predicate family, result-shape
binding, or schema-basis compatibility rule was violated.

## How It Relates To Other Features

- Use [Query Expressions And Result Shapes](../authoring/query-expressions-and-result-shapes.md)
  to author canonical meaning.
- Use [Scopes, Templates, Saved Queries, And View Shapes](../authoring/scopes-templates-saved-queries-and-view-shapes.md)
  when the canonical bundle was composed rather than written directly.
- Validation happens before [Historical Basis, Diff, And Comparison Queries](../capabilities/historical-diff-and-basis.md)
  or view-shape planning can safely reuse a query.

Validation is the legality spine beneath those higher-level features.

## Inspection And Debugging

`QueryValidationReport` is the main debugging artifact. Look at:

- ordered `ValidationEvent`s to see which stage succeeded before failure
- `ValidationRejectionMatrix` to see which failure family accumulated
- `ValidationFailureClass` to group failures by concern
- validation counters and digests to confirm deterministic legality

Useful events include:

- `PredicateValidated`
- `ProjectionValidated`
- `OrderingValidated`
- `TraversalValidated`
- `ResultShapeBindingValidated`
- `CompatibilityEstablished`
- `IdentityFrozen`

## Anti-Patterns

- Treating validation as optional because a query already canonicalized.
- Assuming unknown or non-queryable fields should be tolerated and fixed later.
- Smuggling result-shape widening through validation and expecting planning to
  catch it.
- Treating structured content as queryable everywhere instead of only where the
  schema basis explicitly admits it.

## Current Limits

### Candidate Structural Source Changes

The canonical-validation examples above target the internal engine audience.
Application invariants use `worth-query-decl` and `worth-query-host`, rather
than the engine facade. The
[compiled application preparation/evaluation example](../../../../../../docs/build-an-application.md#prepare-candidate-structural-source-changes)
shows the public handoff: retain paired Native source custody in `prepare_scope`,
keep `WorthQueryPreparedInvariantDerivedChanges` in the rule's owned `Scope`,
then reborrow it during `evaluate`.

Native acquires the planner's real record frontier and both before/after relation
endpoints, then uses its existing inverse-prefix evaluator. Empty support remains
retained. Preparation requires complete declared read and affected-kind coverage
of the recipe. Evaluation rechecks the exact installed definition, original
proposal, both view versions and current reader audience; a retained proof from
another proposal or a narrower reader refuses. The proof retains source backing
and its original payer, without retaining an Execution request or granting reads
from descriptive coordinates.

Three focused Native source courts and the owning compiled-example inventory
test pass for this structural custody prerequisite. The courts cover paired
pullback, empty support, exact proposal/read audiences and incomplete affected
declarations. It supplies structural pullback only: scalar projection changes
still need the projection's per-entry
dependency routing. It does not yet stage incremental managed provider roots,
add managed-index reads to `DecisionReader`, or publish graph/index roots
atomically. Those remain required Phase 2 work. Complete conservative Source
dependencies remain in force; Phases 3–6 remain unfinished.

The current backing uses immutable node pages in place of whole retained node
arrays. An isolated edit shares untouched pages and allocates changed
pages and directory paths under the current request. Prefix insertion/removal
and keyed box edits/refits belong to their pure value owners. These mechanisms
do not acquire projected rows, schedule validation, or publish a graph root.
Two Query backing courts, eight consumer cases and the owning compiled inventory
case pass for this bounded slice. In particular, neither
the structural proof nor a cold Source ordinal supplies the stable full-generation
occurrence identity or admitted scalar reads needed for maintained roots.

Native still applies a proposal and runs its first invariant family before
Query's existing managed-view publication preparation. The next integration must
prepare admitted provider roots at that actual candidate boundary and attach
them to the single-use publication owner. A later publication callback cannot
provide that contract. Committed `DecisionReader` access, candidate invariant
access and the Source/maintenance/scope-inclusive growth proof remain required.

### Canonical Query Validation

- Validation is stable for canonical legality and identity freezing.
- `ValidationWarning` currently has no public warning families; legality is
  modeled primarily through success events or typed failures.
- This feature validates meaning. It does not execute the query, materialize
  history, or lower subscriptions on its own.

## Related Docs

- [Query Expressions And Result Shapes](../authoring/query-expressions-and-result-shapes.md)
- [Scopes, Templates, Saved Queries, And View Shapes](../authoring/scopes-templates-saved-queries-and-view-shapes.md)
- [Historical Basis, Diff, And Comparison Queries](../capabilities/historical-diff-and-basis.md)


