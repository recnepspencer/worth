# Author A Grouped Neighborhood With Contributions

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../../foundations/ordinary-application-front-door.md).

## What This Recipe Covers

This recipe shows how to build one neighborhood-shaped operation with:

- shared grouped posture
- one or more neighborhood members
- shared or member-local contribution meaning

Use it when the neighborhood itself is part of the operation, not just a list
of independent declarations.

## When To Use It

Use this when:

- the group itself carries meaning
- you want shared neighborhood rationale or posture
- at least some contribution meaning is shared across the group or attached to
  one specific member

Do not use this when:

- each declaration stands alone
- you only need a single declaration with contributions
- you do not need grouped semantics at all

## The Smallest Useful Path

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

This is the grouped authoring path when the next question is grouped
orchestration or grouped products.

## Add Shared And Member-Local Contributions

```rust
let input = handle
    .geometry_helpers()
    .local_neighborhood_for_active_face_selection(seed_face)
    .with_member(neighbor_a)
    .with_member(neighbor_b)
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

Use this when the neighborhood also carries shared and member-local
contribution meaning.

## If It Goes Wrong

If grouped orchestration stops and you need member-local repair context:

```rust
let proof = handle
    .geometry_helpers()
    .orchestrate_local_neighborhood_for_active_face_selection_proof(
        declaration,
    );

let recovery = handle
    .recover_from_grouped_orchestration_proof(proof)
    .expect("grouped stop should yield recovery");

let _ = recovery.explanation().grouped_member_context();
```

Use grouped support/readiness when the grouped declaration already exists and
you want to know whether stronger grouped claims are supportable before the
next grouped step.

```rust
let support = handle.grouped_support_report(&declaration);

let _ = support.statuses();
let _ = support.unsupported_claims();
```

## What This Reuses

This recipe stays on the canonical grouped surfaces:

- grouped declaration authoring
- grouped orchestration or grouped products
- grouped contribution composition
- grouped recovery

The helper path only makes the neighborhood family-native.

## Related Docs

- [Grouped Authoring](../grouped-authoring.md)
- [Grouped Contributions](../grouped-contributions.md)
- [Grouped Neighborhood Workflow](../workflow/grouped-neighborhood-workflow.md)
