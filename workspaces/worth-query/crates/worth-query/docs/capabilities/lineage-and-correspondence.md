# Lineage And Correspondence

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Lineage and correspondence queries let Worth Query ask identity questions that
go beyond ordinary data retrieval: what record corresponds to this one, whether
identity continuity is singular or ambiguous, and how identity evolves across
predecessor, successor, replacement, split, merge, branch-local, or historical
contexts.

## Why You Use It

- you need identity continuity, not just record equality
- you want typed outcomes for unique matches, ambiguity, disagreement, or
  denial
- you need lineage traversal and correspondence comparison to share a coherent
  execution model
- you want historical envelopes that preserve both payload and support posture

## Stable Entry Points

- `CorrespondenceEvaluationRequest`
- `resolve_correspondence_evidence(...)`
- `CorrespondenceEvidenceResolved`
- `IdentityEvolutionQueryContext`
- `LineageTraversalDescriptor`
- `CorrespondenceIdentityComparison`
- `admit_identity_evolution_query(...)`
- `execute_admitted_identity_evolution_query(...)`
- `IdentityEvolutionResultBundle`

Important outcome types:

- `CorrespondenceOutcome`
- `LineageContinuity`
- `LineageStructuralDisagreement`
- `AdvisoryStructuralUnique`
- `AdvisoryStructuralAmbiguous`
- `SingularIdentityContinuityResult`
- `PluralIdentitySuccessorSet`
- `AdvisoryIdentityCandidateSet`
- `IdentityEvolutionAmbiguityBundle`
- `IdentityEvolutionIdentityBreakBundle`
- `IdentityEvolutionDeniedBundle`

## Core Mental Model

Correspondence and lineage are related but not identical.

Correspondence asks:

- which other identity most likely corresponds to this one
- whether the evidence is unique, ambiguous, structurally inconsistent, or
  denied

Lineage asks:

- what happened to this identity over time or across branches
- is the next identity singular, plural, merged, replaced, split, or broken

Worth Query keeps these concerns explicit because a record can correspond
structurally without proving clean lineage continuity, and lineage can deny or
branch even when correspondence candidates exist.

## How It Executes

1. Build either a correspondence request or an identity evolution query
   context.
2. Resolve correspondence evidence or admit the identity evolution query.
3. Execute the admitted query against the requested comparison or traversal
   context.
4. Return a typed outcome bundle that preserves continuity, ambiguity, break,
   or denial posture.
5. For historical composition, wrap the result in an envelope that keeps
   metadata and denial posture aligned with the payload.

## Small Example

```rust
use worth_query::facade::foundation::{
    CorrespondenceEvaluationRequest, resolve_correspondence_evidence,
};
use worth_query::facade::foundation::{
    StructuralCandidateDiscoveryPlan, StructuralCandidateOrderingContract,
};

let request = CorrespondenceEvaluationRequest::structural_only(
    vec!["task-42".to_string()],
    StructuralCandidateDiscoveryPlan::IndexBackedBounded,
    4,
    StructuralCandidateOrderingContract::StableFingerprintOrder,
);
let evidence = resolve_correspondence_evidence(request)?;

if let Some(unique) = evidence.outcome().as_advisory_structural_unique() {
    assert!(!unique.advisory_candidate().is_empty());
}
```

This is the smallest honest example because it shows that correspondence is not
just yes-or-no matching. The result carries a specific identity outcome family.

## Real Example

```rust
use worth_query::facade::foundation::{
    admit_identity_evolution_query, execute_admitted_identity_evolution_query,
    IdentityEvolutionQueryContext, LineageTraversalDescriptor,
};

let lineage = IdentityEvolutionQueryContext::lineage_traversal(
    LineageTraversalDescriptor::direct_split_successors("part-17")
);

let admitted = admit_identity_evolution_query(&lineage)?;
let result = execute_admitted_identity_evolution_query(&admitted)?;

let bundle = result.result_bundle();

if let Some(successors) = bundle.as_plural_identity_successor_set() {
    assert!(successors.successor_identities().len() > 1);
} else if let Some(single) = bundle.as_singular_identity_continuity() {
    assert!(!single.authoritative_identity().is_empty());
} else if let Some(ambiguous) = bundle.as_ambiguity() {
    assert_ne!(ambiguous.ambiguity_reason().as_str(), "");
} else if let Some(identity_break) = bundle.as_identity_break() {
    assert_ne!(identity_break.identity_break_reason().as_str(), "");
} else if let Some(denied) = bundle.as_denied() {
    assert_ne!(denied.denial_reason().as_str(), "");
}
```

That example matters because identity evolution is where real systems get
messy: splits, merges, replacements, branch-local rewrites, and genuine breaks
in continuity all need first-class outcomes.

## How It Relates To Other Features

- Pair this with [Historical Basis, Diff, And Comparison Queries](historical-diff-and-basis.md)
  when identity continuity must be evaluated across time-bound snapshots.
- Use [Schema Validation](../modeling/schema-validation.md) and the canonical query stack
  when lineage work sits beside ordinary query legality and planning.
- Correspondence historical envelopes are a close neighbor when you need
  metadata-preserving success, ambiguity, disagreement, or denial composition.

## Inspection And Debugging

The most useful debugging surfaces are the typed result bundles and historical
envelopes. Inspect:

- whether the result is continuity, plural successor, ambiguity, break, or
  denial
- comparison-basis family for identity comparison requests
- lineage traversal descriptor for predecessor/successor/replacement/split/merge
- historical envelope class: success, ambiguity, disagreement, or denied
- parity and replay metadata when identity work is composed with historical
  execution

If the result is surprising, first ask whether the system is proving continuity
or merely surfacing correspondence candidates. Those are intentionally
different promises.

## Anti-Patterns

- Treating correspondence as proof of lineage continuity.
- Collapsing plural successor or merge outcomes into fake singular continuity.
- Hiding ambiguity by auto-picking a candidate without preserving the advisory
  result family.
- Assuming branch-local identity evolution means the same thing as authoritative
  cross-history continuity.

## Current Limits

- The public identity-evolution and correspondence surfaces are query-layer
  infrastructure, not part of the ordinary `workspace.live_view(...)` runtime
  facade.
- Historical envelope composition preserves metadata and denial posture, but it
  does not bypass the support or admission rules of the underlying historical
  path.
- Durable restart and store-backed historical identity semantics remain bounded
  by the same deferred support posture as their historical neighbors.

## Related Docs

- [Historical Basis, Diff, And Comparison Queries](historical-diff-and-basis.md)
- [Schema Validation](../modeling/schema-validation.md)
- [Scopes, Templates, Saved Queries, And View Shapes](../authoring/scopes-templates-saved-queries-and-view-shapes.md)


