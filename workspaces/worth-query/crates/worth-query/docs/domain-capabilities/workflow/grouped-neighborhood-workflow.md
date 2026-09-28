# Grouped Neighborhood Workflow

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../../foundations/ordinary-application-front-door.md).

## What This Workflow Is

This workflow is for neighborhood-shaped work where the group itself is part of
the meaning.

Use it when the job is not just "run these declarations in a loop," but
something closer to:

- "these faces belong to one local neighborhood"
- "this group has one shared rationale"
- "some contribution meaning is shared across the group"

## Why You Use It

- keep group-level meaning separate from member-level meaning
- retain grouped aspect posture, member roles, and member-local aspect context
- inspect grouped route, receipt, and envelope truth without inventing a second
  grouped engine
- compose shared and member-local contributions on the same grouped boundary

## Stable Entry Points

Geometry-helper entry points:

- `local_neighborhood_for_active_face_selection(...)`
- `declare_local_neighborhood_for_active_face_selection(...)`
- `orchestrate_local_neighborhood_for_active_face_selection_outcome(...)`
- `grouped_routes_for_active_face_selection_checked(...)`
- `grouped_receipt_for_active_face_selection_checked(...)`
- `grouped_envelope_for_active_face_selection_checked(...)`
- `grouped_contributions_for_active_face_selection_checked(...)`

Admitted-handle grouped support entry point:

- `grouped_support_report(...)`

## Core Mental Model

Grouped work has four common steps:

1. define the grouped neighborhood
2. admit the grouped declaration
3. choose the next grouped product or orchestration lane
4. optionally add shared or member-local contributions

The grouped declaration is the anchor. Later grouped products and grouped
contributions reuse that retained grouped truth.

## How It Executes

The grouped lane keeps:

- group-level posture such as atomicity and grouping intent
- continuity assumptions and shared posture claims
- member roles
- member-local aspect coverage

From there you can ask for:

- grouped orchestration
- grouped route products
- grouped receipt products
- grouped envelope products
- grouped support/readiness from the admitted grouped declaration
- grouped contribution composition

## Small Example

```rust
let declaration = handle
    .geometry_helpers()
    .declare_local_neighborhood_for_active_face_selection(
        handle
            .geometry_helpers()
            .local_neighborhood_for_active_face_selection(seed_face)
            .with_member(neighbor_a)
            .with_member(neighbor_b)
            .with_atomicity(WorthQueryGroupedAtomicity::Atomic)
            .with_grouping_intent(WorthQueryGroupedIntent::Authoritative)
            .with_shared_rationale("preserve a stable local cut"),
    )?;

let grouped = handle
    .geometry_helpers()
    .orchestrate_local_neighborhood_for_active_face_selection_outcome(
        declaration.clone(),
    );
```

Use this when the neighborhood itself is meaningful and grouped orchestration is
the next question.

## Real Example

```rust
let input = handle
    .geometry_helpers()
    .local_neighborhood_for_active_face_selection(seed_face)
    .with_member(neighbor_a)
    .with_shared_explanation_contribution(
        WorthQueryExplanationContributionAuthoring::requires_context(
            "geometry.group.context",
            "the neighborhood depends on shared active-face context",
        ),
    )
    .with_member_contribution(
        1,
        WorthQueryContributionIntent::workflow(
            WorthQueryWorkflowContributionAuthoring::preview_only(
                "geometry.group.preview",
                "keep neighbor-a in preview-only posture",
            ),
        ),
    );

let grouped = handle
    .geometry_helpers()
    .grouped_contributions_for_active_face_selection_checked(input)?;
```

Use this when the neighborhood also carries shared or member-local contribution
meaning.

## How It Relates To Other Features

- [Grouped Authoring](../grouped-authoring.md) is the feature reference for the
  grouped boundary itself.
- [Grouped Products](../grouped-products.md) go deeper on grouped route,
  receipt, and envelope surfaces.
- [Grouped Contributions](../grouped-contributions.md) go deeper on shared and
  member-local contribution composition.
- [Typed Stops And Remediation Guidance](../typed-stops-and-remediation-guidance.md) is the next guide when grouped
  orchestration stops.

## Inspection And Debugging

Use grouped declaration inspection when you need:

- `atomicity()`
- `grouping_intent()`
- `continuity_assumption()`
- `shared_posture_claims()`
- `aspect_participation()`

Use grouped checked or proof recovery when you need:

- the stopped member index
- the stopped member role
- member-local aspect context
- group-level vs member-level stop ownership

## Anti-Patterns

- using grouped authoring as a prettier `Vec<I>` loop
- flattening shared group contributions and member-local contributions into one
  untyped bag
- treating grouped support/readiness as if it created the grouped declaration

## Current Limits

- grouped recovery is richest on grouped orchestration
- grouped work is still neighborhood-shaped, not arbitrary graph-shaped
- grouped merge resolution is still out of scope

## Related Docs

- [Grouped Authoring](../grouped-authoring.md)
- [Grouped Products](../grouped-products.md)
- [Grouped Contributions](../grouped-contributions.md)
- [Grouped Support And Readiness](../grouped-support-readiness.md)
