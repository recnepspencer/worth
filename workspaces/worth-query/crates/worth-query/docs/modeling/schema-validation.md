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

Structural paired custody retains empty support, exact proposal/read audiences
and complete affected declarations. It is not projected-row READ authority.
The installed managed Source compiler binds physical scalar projection fields
and stored/reverse/composed/union paths to Native's preparation audience.
Query's persistent per-entry dependency postings cover scalar/lifecycle reads;
Native supplies complete reverse scope closure from actual before/after changes.
Neither public sessions nor a caller's initiating scope define that closure.

Direct endpoint projections add schema-owned Native routes from each Source row
to its projected entity. Lifecycle, scalar and adjacency effects pull back on
both images through these routes and then through Source membership to scopes.
The child loan retains the actual prepared read and full-generation physical
relation occurrence; deduplicated endpoint sets cannot erase cardinality.
Every read rechecks the originating row, route, audience and actual observation.
Empty adjacency remains support. Child predicates, deeper projection trees and
child-field ordering refuse installation rather than changing Source meaning.

Maintained row maps and inverse entity postings use Execution's owned ordered
cells. Each copied path is admitted through the original request, and unchanged
cells retain their original grants until their last share closes. Projected row
payloads retain their separate backing; successor images retain no predecessor
grant chain. This applies to removal as well as insertion and endpoint refitting.

The current backing uses immutable node pages in place of whole retained node
arrays. An isolated edit shares untouched pages and allocates changed
pages and directory paths under the current request. Prefix insertion/removal
and keyed box edits/refits belong to their pure value owners. These mechanisms
do not themselves acquire projected rows, schedule validation, or publish a graph
root. Native-issued full-generation occurrences supply maintained row identity;
cold Source ordinals cannot address another scope or an incremental successor.

Native's sealed suspension custom reads use the actual logical before-image
and its older version; native checks still see the physical candidate. Actual
rematerialization does not substitute that logical image for the unavailable
committed physical Source. The managed Source interpretation policy is fixed
canonical meaning, not an application-selected posture. Live owner custody and
an observation identity do not alone admit a projected index read: exact installed
audience, complete Source preparation and publication remain required. Unsupported
Source shapes refuse installation; valid shapes bind the same configured catalog
and managed registry before bootstrap mutations and rule factory publication.
The [public generated Catalog court](../../../worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/managed_index/maintained/generated/court.rs)
composes this path with a real fixed source action and producer. Both actual
Directory scopes retain logical rows during suspension; current public reads
refuse before provider queries. The manifest-correct reconstructed duplicate
reaches its installed Source invariant, and its lawful twin restores warm reads.
Native may prove complete authoritative-record equality from actual restoration
custody and the applied candidate before preparing those roots. Its transient
correspondence selects only the exact suspended image through canonical
runtime/branch/registration/root/commit/position admission and the actual compiled
program and scope. Query verifies the typed provider installation before sharing
the immutable payload as proposed backing. No unavailable committed loan opens;
normal required-rule judgment and coordinated publication still apply. A changed
unprojected field declines this conservative proof and rebuilds under AdmitRebuild.
The court also cancels the original request after one real rule query and retries
from the returned suspension custody without moving the selected basis.
The [generated geometry counterpart](../../../worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/managed_index/maintained/geometry/generated/court.rs)
publishes all three Edge/Vertex roles and their actual relations together. Complete
Source queries expose the generated edge even outside the domain rule's fixed
probe. Original suspension cancellation follows a real committed/proposed query,
leaves Native/World selection available and retries. Exact reconstruction shares
backing; changed endpoint reconstruction reaches B's existing enclosure refusal.
The suspension entry preserves the complete owned in-phase result across its
outer control handoff. Its borrowed typed control projection shares preparation
execution classification with restoration and retains the original Native error.
This court does not qualify a late-success interruption or pending recovery run.
The [cold/history court](../../../worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/managed_index/maintained/geometry/generated/court/cold_history.rs)
first evicts previously attached B through admitted Parent cleanup and closes
that public registration. A real cold-node cancellation leaves availability,
selection and cold cache unchanged; retry reconstructs committed B before either
logical rule. Ordinary Source still reads available Native rows while the derived
cache is cold. Fresh retained ordinary queries resolve current principal/scope
before selecting old data, so suspension preserves the exact current principal
refusal. After restoring endpoint 20, fresh retained Source execution returns
endpoint 10 independently of the current result. Retained Parent mode remains
UnsupportedMode; this evidence does not widen historical managed support.

Native stages provider roots before its first dependent invariant. Each actual
observation reborrows its own Source loan. The existing SourceInvalidationOwner
seals one fixed graph/availability/Source-image compound effect. The unpublished
Native root receives the exact sealed image before movement; actual selected and
pending World basis custody retains that pair, without a Query history cache.
Failure or cancellation leaves the prior root unchanged and releases staging.
Public committed reads still perform principal/scope/READ admission separately.
Their complete Source condition is rebound at the real consuming candidate after
idempotency resolution; copied program names and retained material grant no READ.
A managed fork creates its association header through Native's actual fork
owner before target head retention and basis issuance. It shares authoritative
regions, schema and canonical envelope, uses the same live root identity issuer,
and retains the selected Source's canonical position. Query stages only a
pending branch lookup; durable target installation admits it. Refusal removes
that exact pending registration, with participant-owned cleanup deferred beyond
Native coordination, publication and registration guards. Actual target World
basis custody therefore selects the target association from its first read.
A public close releases its access attachment; invariant and retained basis
owners can still retain the canonical material.
Successful Query owner cleanup uses the actual Native deletion receipt to remove
that branch's live Source lookup. Receipt custody waits beyond the cleanup record
and its outer coordination guards, including partial cleanup and late request
interruption. Existing immutable bases extend retention only through their actual
current Native history-retention owner; matching descriptors cannot substitute
for it. Closing the original Native owner therefore preserves an existing
`ReuseExact` alias's read, eviction and refill paths, while fresh Native branch
admission, forks and writes still refuse.

The exact selected image owns a disposable material directory separately from
its immutable Source conditions. Cache eviction retains the condition and its
revision; a cold reconstruction with Native's complete unchanged disposition
reuses that condition even when its provider payload is new. Cold material is
independent of physical Source suspension. Historical Native/World images can
retain temporal material, and fresh selection never substitutes the Native head
for a lawful older `ReuseExact` component.

Directory cells use Execution's individually funded immutable ordered backing.
Their storage key is the declared index-specific Source path identity plus full
scope generation. Every access still checks the genuine live Program binding;
equal descriptive keys from independent registries refuse. A weak binding memo
belongs to the actual registered path owner, so clones and composition preserve
that audience while independently registered paths remain distinct. Memoization
contributes no descriptive schema or wire meaning.

Admitted eviction uses the original request and retains replaced backing in an
owned retirement value until runtime, directory and registration guards end.
Query prechecks cleanup status before the effect and completes it infallibly;
its actual completed result survives a later outer request interruption. A
concurrent attachment close does not turn completed Native cleanup into a
no-effect refusal. This cleanup supplies neither Source READ nor currentness.

The [compiled application examples](../../../../../../docs/build-an-application.md#maintain-a-provider-source-for-an-invariant)
show both the committed DecisionReader and installed candidate reader audiences.

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


