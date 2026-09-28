# Grouped Authoring

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Grouped authoring is the Query surface for neighborhood-shaped work where the
group itself is part of the meaning.

Use it when your app means:

- "these members belong to one local neighborhood"
- "this neighborhood has one shared rationale or posture"
- "I need group-level and member-level truth retained together"

Do not use it as a prettier `Vec<I>` loop. If the members stand alone, keep
them on the single-declaration path.

## What You Can Do

Grouped authoring now gives you one grouped declaration plus several grouped
projections over the existing declaration-entry product stack:

- grouped declaration admission
- grouped orchestration over member envelopes
- grouped route products
- grouped receipt products
- grouped envelope products
- grouped support/readiness reporting
- grouped contribution composition with shared and member-local contributions

Every grouped surface reuses retained member progression truth. Query does not
invent a second grouped authority system.

## Stable Entry Points

Most callers start here:

- `WorthQueryGroupedDeclarationInput::local_neighborhood(...)`
- `with_member(...)`
- `with_atomicity(...)`
- `with_grouping_intent(...)`
- `with_continuity_assumption(...)`
- `with_shared_posture_claim(...)`
- `with_shared_rationale(...)`
- `declare_grouped(...)`
- `orchestrate_grouped_outcome(...)`
- `grouped_route_checked(...)`
- `grouped_receipt_checked(...)`
- `grouped_envelope_checked(...)`
- `grouped_support_report(...)`
- `grouped_contributions_checked(...)`

If you prefer the geometry front door, the parallel helper entry points live on
`geometry_helpers()`.

## Grouped Meaning

One grouped declaration now retains more than member order.

It records:

- grouped semantics
- ordering
- atomicity
- grouping intent
- continuity assumption
- shared posture claims
- shared rationale
- grouped aspect contract and grouped aspect participation summary
- member roles and member-local aspect coverage

These values are part of the grouped digest because they change the meaning of
the neighborhood.

That means Query can distinguish:

- one exploratory neighborhood from one authoritative neighborhood
- one atomic grouped claim from one member-independent grouped claim
- one neighborhood that assumes continuity from one that does not

## Typical Workflow

```rust
let declaration = handle
    .declare_grouped(
        handle
            .geometry_helpers()
            .local_neighborhood_for_active_face_selection(seed_face)
            .with_member(neighbor_a)
            .with_member(neighbor_b)
            .with_atomicity(WorthQueryGroupedAtomicity::Atomic)
            .with_grouping_intent(WorthQueryGroupedIntent::Authoritative)
            .with_continuity_assumption(
                WorthQueryGroupedContinuityAssumption::PreserveNeighborhood,
            )
            .with_shared_posture_claim(
                WorthQueryGroupedSharedPostureClaim::SharedSelectionFocus,
            )
            .with_shared_rationale("preserve a stable local cut"),
    )?;

let grouped = handle.orchestrate_grouped_outcome(declaration);
```

Once you have the grouped declaration, pick the next grouped product that
matches the question:

- use grouped orchestration when you want the compact envelope-facing grouped
  execution lane
- use grouped routes when you want route posture per member
- use grouped receipts when you want retained crossing truth per member
- use grouped envelopes when you want the public crossing artifact per member
- use grouped support when you want to know whether the grouped claim itself is
  supportable

## Grouped Contributions

Grouped contribution composition is for neighborhoods that need both shared
group posture and member-local additions.

You can attach:

- shared support contributions
- shared explanation contributions
- shared workflow contributions
- member-local contributions by member index

Example:

```rust
let input = handle
    .geometry_helpers()
    .local_neighborhood_for_active_face_selection(seed_face)
    .with_member(neighbor_a)
    .with_shared_support_contribution(
        WorthQuerySupportContributionAuthoring::declaration_support(
            "geometry.trace",
            "track the neighborhood through grouped authoring",
        ),
    )
    .with_member_contribution(
        1,
        WorthQueryContributionIntent::explanation(
            WorthQueryExplanationContributionAuthoring::explains_fallback(
                "geometry.member.fallback",
                "neighbor-a needs a local fallback explanation",
            ),
        ),
    );

let grouped = handle.grouped_contributions_checked(input)?;
```

This still lowers onto the canonical contribution-composed seam. The grouped
surface just preserves which contributions were shared and which belonged to a
specific member.

## Support And Readiness

Use `grouped_support_report(...)` when your app needs to know whether the
grouped claim itself is honest before or beside execution.

The report answers whether Query currently supports:

- grouped declaration authoring
- grouped routes
- grouped receipts
- grouped envelopes
- grouped contribution composition
- grouped atomicity and grouped intent posture
- continuity assumptions
- shared posture claims

If a shared posture claim is too strong for the current retained grouped truth,
the report keeps that explicit through `unsupported_claims()`.

## Recovery

The compact ordinary grouped lane is still useful, but use the richer grouped
checked or proof recovery entry points when you care about neighborhood-local
repair:

- `recover_from_grouped_orchestration_checked(...)`
- `recover_from_grouped_orchestration_proof(...)`

Those preserve:

- whether the stop was group-level alignment or member-level execution
- stopped member index
- stopped member role
- stopped member aspect record

That is the lane you want when you need to tell:

- stale basis from wrong world
- one failed member from one broken grouped claim
- one refresh/rebind action from one manual inspection case

## Inspection

Useful grouped declaration accessors:

- `group_digest()`
- `atomicity()`
- `grouping_intent()`
- `continuity_assumption()`
- `shared_posture_claims()`
- `aspect_record()`
- `aspect_participation()`
- `members()`

Useful grouped product accessors:

- `declaration()`
- `members()`
- `member_index()`
- `role()`
- `aspect_record()`
- `product()`

Useful grouped contribution accessors:

- `members()[i].0.shared_contribution_count()`
- `members()[i].0.member_contribution_count()`
- `members()[i].1.composition_digest()`

## How It Relates To Other Features

- [Family Helpers](./family-helpers.md) provide the geometry-native entry
  points for grouped neighborhood work.
- [Contribution-Composed Orchestration](./contribution-composed-orchestration.md)
  owns the single-declaration contribution engine that grouped contributions
  reuse.
- [Typed Stops And Remediation Guidance](./typed-stops-and-remediation-guidance.md) owns the next-step guidance surface
  when grouped orchestration stops.
- [Orchestration Inventory](./orchestration-inventory.md) documents the grouped
  helper orchestration lane and its semantic attachments.

## Current Limits

- grouped semantics are still neighborhood-shaped, not arbitrary graph-shaped
- the helper front door is currently geometry-oriented
- grouped recovery is richest on grouped orchestration; grouped routes,
  receipts, envelopes, and grouped contributions are primarily inspection
  surfaces today
- grouped merge resolution is still out of scope; the current goal is to
  preserve enough grouped/member/aspect/basis truth to support it later

## Related Docs

- [Family Helpers](./family-helpers.md)
- [Contribution-Composed Orchestration](./contribution-composed-orchestration.md)
- [Typed Stops And Remediation Guidance](./typed-stops-and-remediation-guidance.md)
- [Orchestration Inventory](./orchestration-inventory.md)
