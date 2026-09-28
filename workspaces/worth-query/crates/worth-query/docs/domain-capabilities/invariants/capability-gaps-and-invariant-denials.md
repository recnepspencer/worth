# Capability Gaps And Invariant Denials

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../../foundations/ordinary-application-front-door.md).

## What This Feature Is

This feature covers domain-authored capability-gap and invariant-denial posture
that materializes into canonical Query runtime artifacts instead of local
domain wrappers.

## Why You Use It

- you want to explain why a geometry operation cannot proceed because a runtime
  capability is missing
- you need a typed invariant denial artifact for graph composition or related
  runtime seams
- you want these outcomes to participate in certification and support posture

## Stable Entry Points

Proof-facing authoring:

- `WorthQueryInvariantCapabilityContributionAuthoring::graph_capability_gap(...)`
- `WorthQueryInvariantCapabilityContributionAuthoring::graph_invariant_denial(...)`

Canonical materializers:

- `materialize_graph_composition_capability_support_row(...)`
- `materialize_graph_composition_domain_invariant_denial(...)`

Boundary acquisition:

- obtain `WorthQueryLowerRuntimeBoundaryEnvelope` values from real Query
  boundary receipts, or pass those receipts through
  `for_lower_runtime_boundary_source(...)`
- do not construct lower-runtime boundary envelopes directly; constructors are
  intentionally private

For ordinary invariant registration, use the separate runtime and domain
registration surfaces documented in
Registering Domain Invariants Through Query.

## Core Mental Model

These surfaces are for runtime-facing capability and invariant posture, not for
catalog registration itself.

A capability gap says:
- this runtime seam is missing a needed capability

An invariant denial says:
- this operation must not proceed because a named invariant would be broken

## How It Executes

1. author capability-gap or invariant-denial meaning
2. bind it to the allowed lower-runtime target family through a real boundary
   envelope or boundary-envelope source
3. progress it through the contribution lifecycle
4. materialize the canonical support row or denial artifact

## Small Example

```rust
let requested =
    WorthQueryInvariantCapabilityContributionAuthoring::graph_invariant_denial(
        "graph.non_manifold_edge_split",
        runtime_semantics,
    );
```

This is the smallest honest example because this feature starts on a sharper
runtime-facing proof lane, not an ordinary registration convenience verb.

## Real Example

```rust
let row = materialize_graph_composition_capability_support_row(
    ready_capability_gap_contribution,
)?;

let denial = materialize_graph_composition_domain_invariant_denial(
    ready_invariant_denial_contribution,
)?;
```

For a geometry kernel, this is the category you use when a split, merge, or
rebind operation needs to say:

- the graph composition floor does not yet support this target combination
- this edge split would violate the non-manifold invariant

## How It Relates To Other Features

- use Registering Domain Invariants Through Query
  to install runtime invariants
- use Lower-Runtime Support And Boundary Traceability
  when you only need support posture rather than a capability gap or denial
- use Continuity Contributions And Authoritative Successors
  when the invariant story depends on predecessor and successor truth

## Inspection And Debugging

- capability support rows and invariant denials have distinct digest families
- do not collapse them into one "invariant problem" bucket in downstream code

## Anti-Patterns

- using runtime-facing denial artifacts as a substitute for registration
- flattening invariant context into one diagnostic string
- binding these surfaces to declaration targets when the runtime seam is the
  real authority boundary
- fabricating a lower-runtime boundary envelope instead of obtaining one from a
  Query boundary receipt

## Current Limits

- the strongest runtime-facing use here is lower-runtime-bound
- ordinary common-lane coverage is centered on registration; capability-gap and
  denial surfaces remain sharper runtime tools

## Related Docs

- Registering Domain Invariants Through Query
- Lower-Runtime Support And Boundary Traceability
- Continuity Contributions And Authoritative Successors
