# Cross-Runtime Causal Inspection

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Cross-runtime causal inspection explains why an observed Query result changed,
was suppressed, was denied, or was replayed across Query, bridge, relational,
and signal boundaries. It starts from an observation receipt plus a sealed
inspection-basis capability, then produces an admitted explanation plan and an
inspectable artifact.

This is different from `workspace.inspections()?.inspect(...)`, which explains one retained
workspace target. Use causal inspection when the question crosses runtime
boundaries.

## Why You Use It

- Explain a result change through Query, bridge, truth, and signal evidence.
- Investigate temporal wakes, async completions, remasking, replay drift, or a
  resume mismatch.
- Ask for reference-only evidence without materializing every lower-runtime
  detail.
- Preserve a typed denial or advisory when requested evidence is unavailable
  or too expensive.
- Keep the explanation tied to the same scoped authority as the observation.

## Stable Entry Points

Import the basis capability from the foundation facade and causal inspection
from the runtime facade:

```rust
use worth_query::facade::{
    foundation::{basis_lifecycle, ScopedInspectionBasis},
    runtime::{CausalInspection, QueryObservationReceipt},
};
```

The ordinary builder surface is:

- `CausalInspection::for_observation(receipt, inspection_basis)`
- `why_changed()`, `why_suppressed()`, `why_denied()`,
  `why_replayed()`, or `why_previewed()`
- `why_temporal_wake()`, `why_async_completion()`, `why_remasked()`, or
  `why_resume_mismatch()` for common cross-runtime questions
- `reference_only()` or `materialized_detail()`
- `plan()`
- `plan.materialize_with_bridge(...)`
- `CausalInspection::support()`

The observation receipt alone is not inspection authority. The scoped
inspection basis is required and must match the basis retained by the receipt.

## Core Mental Model

The receipt proves that an observation happened. The inspection basis proves
that this caller may inspect that truth world. Neither artifact substitutes for
the other.

`CausalInspection::for_observation(...)` keeps both together. Planning anchors
the observation, resolves retained evidence references, checks support and
cost, and creates an admitted, advisory, or denied proof flow. Materialization
then asks the bridge for the detail allowed by that plan.

Reference-only evidence is the common supported lane. Materialized detail is
advisory because availability, redaction, and lower-runtime cost can narrow the
answer.

## How It Executes

```text
observation receipt + matching ScopedInspectionBasis
  -> causal question and evidence selection
  -> evidence-reference resolution
  -> support and cost admission
  -> causal inspection plan
  -> optional bridge materialization
  -> Query-owned causal inspection artifact
```

Basis mismatch and missing required evidence stop during planning, before a
bridge explanation is assembled.

## Small Example

```rust
use worth_query::facade::{
    foundation::ScopedInspectionBasis,
    runtime::{CausalInspection, QueryObservationReceipt},
};

fn plan_change_explanation(
    receipt: QueryObservationReceipt,
    inspection_basis: ScopedInspectionBasis,
) -> Result<_, worth_query::facade::runtime::CausalInspectionPlanError> {
    CausalInspection::for_observation(receipt, inspection_basis)
        .why_changed()
        .reference_only()
        .include_all_retained_evidence()
        .plan()
}
```

The function requires both proof artifacts. A caller cannot authorize
inspection by supplying a receipt or matching-looking digest alone.

## Real Example

```rust
use worth_query::facade::runtime::CausalInspection;

let plan = CausalInspection::for_observation(receipt, inspection_basis)
    .why_async_completion()
    .reference_only()
    .include_all_retained_evidence()
    .plan()?;

let artifact = plan.materialize_with_bridge(&bridge)?;

record_causal_evidence(
    artifact.receipt(),
    artifact.authority_bindings(),
    artifact.performance_envelope(),
);
```

The observation and inspection capability are authoritative inputs. The bridge
owns its lower-runtime evidence and Query owns the public explanation artifact.
The artifact's references and digests are inspection projections, not new
authority inputs.

## How It Relates To Other Features

- [Inspection](./inspection.md) explains retained Query targets without
  assembling a cross-runtime causal envelope.
- [Basis Capability Lifecycle](./basis-capability-lifecycle.md) creates the
  `ScopedInspectionBasis` required here.
- [Authoritative Mutation Evidence](./authoritative-mutation-evidence.md)
  explains what a write changed; causal inspection explains why an observed
  result followed.
- [Lower-Runtime Capability Routing](../domain-capabilities/lower-runtime-capability-routing.md)
  owns bridge-facing materialization boundaries.
- Lower-runtime explanation contributions describe
  domain explanation posture; they do not execute this runtime lane.

## Inspection And Debugging

Before materialization, inspect:

- `plan.support_posture()`
- `plan.required_evidence()`
- `plan.decision_trace()`
- `plan.estimated_cost()`
- `plan.inspection_basis()`
- `plan.explain()`

After materialization, inspect:

- `artifact.primary_result()`
- `artifact.warnings()`
- `artifact.authority_bindings()`
- `artifact.evidence()`
- `artifact.performance_envelope()`
- `artifact.receipt()`
- `artifact.denial_reason()` or `artifact.advisory_reason()`

Use `CausalInspection::support()` to distinguish supported, advisory, and
deferred explanation families before authoring a production workflow.

## Anti-Patterns

- Calling `CausalInspection::for_observation(...)` without a scoped inspection
  basis.
- Reconstructing inspection authority from receipt, basis, or evidence digests.
- Calling `workspace.inspections()?.inspect(...)` and describing it as cross-runtime causal
  inspection.
- Importing bridge or signal internals into product code to assemble a local
  explanation.
- Treating advisory materialized detail as guaranteed support.
- Using domain explanation contributions in place of causal inspection.

## Current Limits

- Cross-runtime reference-only explanation is supported.
- Materialized detail is advisory and can narrow under evidence, redaction, or
  bridge constraints.
- Durable causal archives and store-backed replay reconstruction are deferred.
- Causal inspection explains retained evidence; it does not create truth,
  replay authority, or durable history.

## Related Docs

- [Inspection](./inspection.md)
- [Basis Capability Lifecycle](./basis-capability-lifecycle.md)
- [Inspection Vs Cross-Runtime Explanation](../domain-capabilities/choosing/inspection-vs-cross-runtime-explanation.md)
- [Support Matrix And Admission](../foundations/support-matrix-and-admission.md)
- [Authoritative Mutation Evidence](./authoritative-mutation-evidence.md)
