# Scopes, Templates, Saved Queries, And View Shapes

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

These features are Worth Query's query productization layer. They let you
compose canonical query meaning from reusable scopes and templates, admit the
result for a specific view shape, and freeze reusable saved-query artifacts
with explicit reuse and persistence posture.

## Why You Use It

- you want higher-order query building blocks instead of repeating canonical
  query authoring by hand
- you need a reusable query surface that still preserves legality and identity
- you want view-family planning to be explicit and query-aware
- you need reuse decisions that can distinguish "safe to reuse", "requires
  fresh freeze", and "deny"

## Stable Entry Points

- `GuidedCompositionPath`
- `ExpandedScopeArtifact`
- `TemplateInstantiationArtifact`
- `ComposedCanonicalQueryBundle`
- `admit_view_shape(...)`
- `validate_canonical_bundle_for_admitted_view_shape(...)`
- `plan_admitted_view_shape(...)`
- `freeze_direct_saved_query(...)`
- `freeze_composed_saved_query(...)`
- `evaluate_saved_query_reuse(...)`

Important public vocabulary:

- `QueryCompositionFamily`
- `PredicateScope`
- `OrderingScope`
- `ProjectionScope`
- `TraversalBoundScope`
- `BasisAwareScope`
- `DetailTemplate`
- `CollectionTemplate`
- `ObservedInspectorDetailTemplate`
- `FocusedInspectorDetailTemplate`
- `GroupedCollectionTemplate`
- `ViewShapeFamily`
- `SavedQueryReuseOutcome`
- `SavedQueryPersistenceClaim`

## Core Mental Model

This layer turns raw query authoring into reusable product surfaces.

The progression looks like this:

1. scopes add bounded reusable meaning to a query
2. templates instantiate larger reusable query shapes
3. canonicalization freezes the composed query meaning
4. view-shape admission proves that a query fits a UI or product surface
5. saved-query freeze captures that whole contract for later reuse

Named scope expansion on the runtime-backed path is now a verified composition
lane: admitted scope families must canonicalize to the same query meaning as
equivalent direct construction, with explicit scope lineage and no caller-owned
rewrite folklore.

Template instantiation on the runtime-backed path is now a verified
composition lane too: admitted detail and collection template families must
canonicalize to the same query meaning as equivalent direct construction, with
explicit binding digests, exact slot and binding counters, and typed fail-early
denial for malformed or deferred neighbors.

The public application support report now publishes that same closure through
the query-composition support profile. When `QueryComposition` is admitted,
downstream callers should see:

- `named_scope_expansion:verified`
- `template_instantiation:verified`

The deferred composition neighbors stay explicit instead of being silently
promoted:

- `observed_inspector_detail_template`
- `focused_inspector_detail_template`

Grouped collection templates are now part of the admitted runtime-backed
template lane rather than a deferred neighbor. They preserve canonical
collection identity and grouped future-bearing posture through the ordinary
template-instantiation authority path.

Reuse is not a blind cache hit. Worth Query preserves the difference between:

- same canonical query meaning
- same composition meaning
- same view-shape meaning
- same schema-basis and identity-consumption posture
- same temporal/async surface posture

Future-bearing reuse is fail-closed. A composition, view-shape, or saved-query
surface must either preserve temporal/async meaning, mark it explicitly
deferred, require a fresh freeze, or deny the reuse outright. It is not valid
to silently degrade that meaning into ordinary-only reuse.

## How It Executes

1. Expand named scopes or instantiate a template through `GuidedCompositionPath`.
2. Canonicalize the composed output into a `ComposedCanonicalQueryBundle`.
3. Admit a `ViewShapeDescriptor` for the intended detail, table, inspector, or
   grouped surface.
4. Validate the canonical bundle for that admitted view shape.
5. Plan the admitted view shape to get planning, delivery, patch, and identity
   posture.
6. Freeze the result as a direct or composed saved query.
7. Evaluate reuse later against the new schema basis, support profile,
   template-slot, and view-shape conditions.

## Small Example

```rust
use worth_query::facade::foundation::GuidedCompositionPath;
use worth_query::facade::runtime::freeze_composed_saved_query;

let composed = GuidedCompositionPath::new()
    .expand_collection_scopes(["open-items", "recent-first"])?
    .canonicalize_expanded()?;

let saved = freeze_composed_saved_query(&composed, freeze_context)?;

assert_eq!(saved.metadata().composition_family().as_str(), "named_scope_expansion");
```

This is the smallest honest example because it shows that reusable query work
starts with composition meaning, not with ad hoc post-hoc persistence.

## Real Example

```rust
use worth_query::facade::foundation::GuidedCompositionPath;
use worth_query::facade::runtime::{evaluate_saved_query_reuse, freeze_composed_saved_query};
use worth_query::facade::runtime::{SavedQueryFreezeContext, SavedQueryReuseOutcome};
use worth_query::facade::runtime::{
    admit_view_shape, plan_admitted_view_shape,
    validate_canonical_bundle_for_admitted_view_shape,
};

let template = GuidedCompositionPath::new()
    .instantiate_collection_template_with_query_bindings(
        "work-items-by-owner",
        [("owner_id", "person-7"), ("status", "ready")],
    )?
    .canonicalize_expanded()?;

let admitted_view = admit_view_shape("kanban-grouped-by-stage", &template)?;
let validated_view = validate_canonical_bundle_for_admitted_view_shape(
    &template,
    &admitted_view,
)?;
let plan = plan_admitted_view_shape(&validated_view)?;
let freeze_context = SavedQueryFreezeContext::new(
    "runtime-support-profile-digest",
    "kanban-grouped-capability-family",
);

assert_eq!(plan.family().as_str(), "kanban_grouped");

let frozen = freeze_composed_saved_query(&template, &plan, freeze_context)?;
let reuse = evaluate_saved_query_reuse(&frozen, &new_reuse_context)?;

match reuse {
    SavedQueryReuseOutcome::Admitted(decision) => {
        assert_ne!(decision.overall().as_str(), "");
    }
    SavedQueryReuseOutcome::Denied(denial) => {
        assert!(!denial.message().is_empty());
    }
}
```

This is the hard shape the feature is built for:

- reusable query composition
- grouped view-shape planning
- frozen saved-query identity
- explicit reuse law instead of hand-wavy caching

## How It Relates To Other Features

- Composition outputs should still pass [Schema Validation](../modeling/schema-validation.md).
- View-shape planning is often upstream of [Automatic Subscription Family Selection And Diagnostics](../capabilities/subscription-selection-and-diagnostics.md)
  because view family and future-bearing live posture together can change the
  selected subscription family or deny the live shape early.
- The admitted view family also constrains later mixed-cause delivery
  projection. Detail, table, grouped, and inspector surfaces consume one
  canonical ordered delivery stream, but they do not get to redefine Bridge
  ordering or collapse basis-bound delivery members for convenience.
- Saved-query reuse becomes especially important when paired with
  [Historical Basis, Diff, And Comparison Queries](../capabilities/historical-diff-and-basis.md)
  or identity-aware inspection surfaces.

## Inspection And Debugging

For composition:

- inspect the `QueryCompositionFamily`
- inspect which scopes or template slots participated
- compare canonical query digest with composition digest

For view shapes:

- inspect `ViewShapeFamily`
- inspect identity-consumption posture
- inspect grouped vs detail vs focused-inspector planning evidence
- inspect patch posture and invalidation posture

For saved queries:

- inspect persistence claim
- inspect reuse outcome and rebinding dimensions
- inspect schema-basis equivalence evidence
- inspect temporal/async surface posture on the frozen artifact
- inspect whether a miss is a hard denial or a legal fresh-freeze requirement

## Anti-Patterns

- Treating scopes and templates as string substitution instead of bounded query
  composition.
- Assuming saved-query reuse means "same canonical query digest only."
- Assuming scope expansion, template instantiation, or saved-query reload may
  erase temporal/async meaning and still count as valid reuse.
- Freezing a saved query with a durable claim that the runtime does not support.
- Ignoring view family and identity-consumption changes when deciding reuse.

## Current Limits

- Durable claims such as restart-stable continuation, import/export, and full
  durable reload are explicitly denied today.
- Runtime-backed temporal/async reuse is only admitted for the explicitly
  preserved surfaces. Table, detail, and `kanban_grouped` now remain
  future-preserving, while observed-inspector and focused-inspector
  future-bearing postures stay visible but deferred instead of being weakly
  admitted.
- Core runtime-backed view-family support rows are now verified for `table`,
  `detail`, `inspector_detail_observed`, `inspector_detail_focused`, and
  `kanban_grouped`.
- View-shape planning is strong for current detail, table, inspector, and
  grouped surfaces. Remaining grouped follow-on work in this area belongs to
  future durable/store-backed neighbors, not to runtime-backed grouped
  reusable composition/template closure or the `kanban_grouped` view-family
  row itself.
- Some reuse mismatches are legal but require a fresh freeze rather than
  outright denial. That distinction is part of the contract.

## Related Docs

- [Schema Validation](../modeling/schema-validation.md)
- [Historical Basis, Diff, And Comparison Queries](../capabilities/historical-diff-and-basis.md)
- [Automatic Subscription Family Selection And Diagnostics](../capabilities/subscription-selection-and-diagnostics.md)


