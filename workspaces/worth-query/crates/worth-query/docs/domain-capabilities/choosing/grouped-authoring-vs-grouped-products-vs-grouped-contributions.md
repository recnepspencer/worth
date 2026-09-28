# Grouped Authoring Vs Grouped Products Vs Grouped Contributions

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../../foundations/ordinary-application-front-door.md).

## What This Page Helps You Choose

Use this page when you are working on one neighborhood or local group of
declaration meaning and need to choose between:

- defining the grouped meaning itself
- reading grouped route, receipt, or envelope truth
- composing shared or member-local contributions onto that grouped meaning

## Why You Use It

- keep grouped declaration meaning separate from grouped product inspection
- keep grouped contributions separate from the grouped declaration itself
- preserve the right grouped recovery and aspect-native story for neighborhood
  work

## Surfaces Compared

- [Grouped Authoring](../grouped-authoring.md)
- [Grouped Products](../grouped-products.md)
- [Grouped Contributions](../grouped-contributions.md)
- [Grouped Support And Readiness](../grouped-support-readiness.md)

## Core Mental Model

Think of grouped work as three layers:

1. grouped authoring: define one grouped neighborhood meaning
2. grouped products: inspect grouped route, receipt, or envelope truth
3. grouped contributions: attach shared or member-local contributions to that
   grouped neighborhood

The group itself is part of the meaning, so Query keeps group-level posture and
member-level aspect witness separate.

## How To Choose

Choose **grouped authoring** when:

- the group itself is part of the declaration meaning
- you need grouped atomicity, grouping intent, continuity assumptions, or
  shared posture claims
- you want grouped checked or proof orchestration

Choose **grouped products** when:

- the grouped declaration already exists
- you need grouped route, receipt, or envelope artifacts
- you want one group summary plus member-level product truth

Choose **grouped contributions** when:

- the grouped neighborhood also carries shared support, explanation, workflow,
  or member-local contribution authoring
- you need to distinguish group-level contributions from member-local ones

Choose **grouped support/readiness** when:

- the grouped declaration already exists
- you need to know whether its grouped semantics, grouped products, or grouped
  contributions are supportable before the next grouped step

## Small Example

Use grouped authoring to declare one neighborhood:

```rust
let declaration = handle
    .geometry_helpers()
    .local_neighborhood_for_active_face_selection(seed)
    .with_member(neighbor_a)
    .with_member(neighbor_b);

let grouped = handle
    .geometry_helpers()
    .declare_local_neighborhood_for_active_face_selection(declaration)?;
```

Use grouped products after admission:

```rust
let envelope = handle.grouped_envelope_checked(grouped.clone());
```

Use grouped contributions when the neighborhood also carries shared or
member-local contribution meaning:

```rust
let result = handle.grouped_contributions_checked(input)?;
```

## Real Example

For a local CAD neighborhood edit:

- choose grouped authoring when the neighborhood itself is meaningful, such as
  "these adjacent faces form one local cut"
- choose grouped products when you need the grouped route, receipt, or envelope
  truth that follows from that neighborhood
- choose grouped contributions when the neighborhood also carries shared
  explanation or support contributions plus member-local geometry intent

If one member later stops, recovery can preserve member index, role, basis
posture, and member-local aspect context because those grouped surfaces stay
separate.

## How It Relates To Other Features

- [Family Helpers](../family-helpers.md) gives you geometry-native grouped
  entry points over the same canonical grouped surfaces.
- [Typed Stops And Remediation Guidance](../typed-stops-and-remediation-guidance.md) preserves grouped member context
  when a grouped checked or proof lane stops.
- [Contribution-Composed Orchestration](../contribution-composed-orchestration.md)
  is the canonical lower seam grouped contributions reuse.

## Inspection And Debugging

Use grouped authoring when you need:

- grouped semantic posture
- grouped aspect contract
- grouped checked or proof stop ownership

Use grouped products when you need:

- grouped route, receipt, or envelope artifacts
- member-level product truth

Use grouped contributions when you need:

- shared vs member-local contribution distinction
- grouped contribution denial posture
- retained contribution summaries

## Anti-Patterns

- looping over single declarations when the group itself is part of the meaning
- treating grouped products as if they defined grouped meaning
- mixing shared group contributions and member-local contributions without
  using the grouped contribution surface
- teaching grouped helpers as a second grouped engine

## Current Limits

- grouped collaborative merge resolution is still outside the direct public
  surface
- grouped ordinary outcomes stay compact; richer grouped member witness lives on
  the checked and proof lanes

## Related Docs

- [Grouped Authoring](../grouped-authoring.md)
- [Grouped Products](../grouped-products.md)
- [Grouped Contributions](../grouped-contributions.md)
- [Grouped Support And Readiness](../grouped-support-readiness.md)
- [Family Helpers](../family-helpers.md)
