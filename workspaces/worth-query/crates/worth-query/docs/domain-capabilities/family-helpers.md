# Family Helpers

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Family helpers are the domain-native front door over stable generic Query
surfaces.

Use them when you already know the declaration family and want the call site
to read like the domain operation you are actually performing.

Helpers do not own a second runtime. They lower onto the same canonical Query
lanes the generic APIs use.

## Geometry Helper Jobs

The geometry helper surface currently helps with four kinds of work:

- progress active-face-selection declarations
- prepare signal-facing continuation work from that selection
- attach declaration-scoped contributions to that selection
- author local neighborhoods around that selection

That last job now includes more than grouped orchestration. Geometry helpers
can also drive grouped routes, receipts, envelopes, and grouped contribution
composition once the grouped declaration exists.

## Stable Entry Points

Start here:

- `family_helpers()`
- `geometry_helpers()`

Common geometry calls:

- `progress_active_face_selection(...)`
- `prepare_preview_for_active_face_selection_outcome(...)`
- `prepare_runtime_route_for_active_face_selection(...)`
- `prepare_runtime_route_for_active_face_selection_outcome(...)`
- `prepare_runtime_route_for_active_face_selection_checked(...)`
- `prepare_runtime_route_for_active_face_selection_proof(...)`
- `prepare_current_truth_view_for_active_face_selection_outcome(...)`
- `prepare_historical_truth_view_for_active_face_selection(...)`
- `prepare_historical_truth_view_for_active_face_selection_outcome(...)`
- `prepare_historical_truth_view_for_active_face_selection_checked(...)`
- `prepare_historical_truth_view_for_active_face_selection_proof(...)`
- `orchestrate_material_attachment_for_active_face_selection_outcome(...)`
- `local_neighborhood_for_active_face_selection(...)`
- `declare_local_neighborhood_for_active_face_selection(...)`
- `orchestrate_local_neighborhood_for_active_face_selection_outcome(...)`
- `grouped_routes_for_active_face_selection_checked(...)`
- `grouped_receipt_for_active_face_selection_checked(...)`
- `grouped_envelope_for_active_face_selection_checked(...)`
- `grouped_contributions_for_active_face_selection_checked(...)`

## Mental Model

Think of helpers as typed builders plus family-gated projections.

They do two things:

1. keep the public callsite specific to one real declaration family
2. lower onto canonical Query artifacts and orchestration lanes

That means:

- continuation helpers still reuse signal-compatibility orchestration
- material-attachment helpers still reuse contribution-composed orchestration
- grouped neighborhood helpers still reuse grouped declaration and grouped
  product surfaces

If the generic surface would return a typed `Deferred`, `WrongWorld`,
`ContributionDenied`, `Prepared`, or `Bound`, the helper surface should expose
that same posture rather than hiding it behind helper-local status strings.

## Grouped Helper Workflow

The grouped geometry flow now looks like this:

```rust
let declaration = handle
    .geometry_helpers()
    .declare_local_neighborhood_for_active_face_selection(
        handle
            .geometry_helpers()
            .local_neighborhood_for_active_face_selection(seed_face)
            .with_member(neighbor_a)
            .with_atomicity(WorthQueryGroupedAtomicity::Atomic)
            .with_grouping_intent(WorthQueryGroupedIntent::Authoritative)
            .with_continuity_assumption(
                WorthQueryGroupedContinuityAssumption::PreserveNeighborhood,
            )
            .with_shared_rationale("preserve a stable local cut"),
    )?;

let envelopes = handle
    .geometry_helpers()
    .grouped_envelope_for_active_face_selection_checked(declaration.clone());

let grouped = handle
    .geometry_helpers()
    .orchestrate_local_neighborhood_for_active_face_selection_outcome(declaration);
```

The helper surface gives you the domain-native way to write that neighborhood,
but the retained truth still lives on the canonical grouped declaration and
grouped product artifacts.

## Grouped Contribution Workflow

Grouped contribution authoring stays in the same style:

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

That still lowers through the canonical contribution-composed seam. The helper
surface just makes the family-specific grouped authoring flow easier to write.

## Inspection And Recovery

Use the returned canonical artifacts for inspection:

- signal-facing helper results:
  - `class()`
  - `signal_execution_family()`
- material-attachment helper results:
  - `contribution_composition()`
  - `contributions()`
- grouped helper results:
  - `declaration().atomicity()`
  - `declaration().grouping_intent()`
  - `declaration().aspect_participation()`
  - `members()[i].role()`
  - `members()[i].aspect_record()`
  - `members()[i].product()`

For repair:

- use `recover_from_outcome(...)` for the compact ordinary lane
- use `recover_from_grouped_orchestration_checked(...)` or
  `recover_from_grouped_orchestration_proof(...)` when you need member-local
  grouped context

## When Not To Use Helpers

Skip helpers when you need:

- a cross-family generic tool
- direct control over the lower generic input type
- a surface that the helper family has not exposed yet

Helpers are for expression and discoverability. They are not a substitute for
understanding the canonical Query lanes underneath.

## Related Docs

- [Grouped Authoring](./grouped-authoring.md)
- [Signal Compatibility Orchestration](./signal-compatibility-orchestration.md)
- [Contribution-Composed Orchestration](./contribution-composed-orchestration.md)
- [Typed Stops And Remediation Guidance](./typed-stops-and-remediation-guidance.md)
