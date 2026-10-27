# Cross-Runtime Causal Inspection

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Cross-runtime causal inspection explains why an observed Query result changed,
was suppressed, was denied, or was replayed across Query, bridge, relational,
and signal boundaries. It starts from an observation receipt plus a sealed
inspection-basis capability, then produces an admitted explanation plan and an
inspectable artifact.

The operational implementation lives in
`worth-query-execution/src/domain_computation/primary_graph/causal_inspection/`.
The older engine preserves its supported receipt constructors through a
directional adapter containing one Execution receipt; it retains only old-receipt
translation and historical certification. Anchoring, reference resolution,
admission and materialization have one Execution owner.

Completed application-query publication receipts retain an opaque native
observation when their disclosed source owner supplies one. It carries original
source, original bound interpretation, parameter, scope and basis evidence,
without a snapshot pin or current
permission. The ordinary Decl/Host inspection entrance and fresh governed
disclosure remain unfinished in milestone 9.18.1 Phase 9. The older diagnostic
builder described here does not provide that application admission.

Output-readiness delivery also privately retains the actual Native conditional
observation. Its immutable Bridge backing preserves the real Signal decision,
semantic observations, admitted snapshot identity and Signal-issued invalidation
observation, without keeping the operational snapshot reader or reentry seed.
The unique performed invalidation receipt still transfers only once; the
read-only observer shares its original funded backing and grants no execution or
disclosure authority. This retention does not yet connect the complete producer
chain to an application inspection entrance. See
[Native observation custody](../../../../../../crates/worth-runtime-bridge/README.md#read-only-native-observation-custody)
for ownership, funding and release details.

### Original Query Binding Custody

An ordinary installed query binding carries the actual immutable catalog binding
into its query, including after `into_query`. Completed row and result-set sources
share that interpretation with their original admitted parameters in one capsule.
Cloning or retyping a source preserves it; retaining the application query
observation does not acquire a Native reader. No descriptor or parameter values
are copied for this binding handoff.

Two bindings can name the same query while differing in input, result, scope or
principal meaning. A matching query identity cannot substitute for the original
catalog binding. Queries issued through the raw certification owner retain no
binding, and later inspection cannot infer one. The carrier stays opaque: it
exposes neither protected metadata nor permission through the application receipt.
The [Native query court](../../../worth-query-execution/src/domain_computation/primary_graph/causal_inspection/application_observation/declared_query/tests.rs)
executes two actual same-query bindings and an unbound query, preserving their
distinct custody after operational query bases have closed.

This prerequisite supplies the original interpretation for fresh Host admission;
it does not complete that admission, producer-chain association or Phase 9.

### Original Query Row Evidence

Query completion retains the exact immutable source meaning for every completed
row, once per invocation alongside the original binding and admitted parameters.
Each row source and the result-set source share that capsule. The opaque query
observation therefore preserves the original field and adjacency reads after
business rows and operational query bases have dropped. This capture copies no
business rows or footprint facts and retains no Native reader.

The roster owns fixed pointer storage admitted by the original execution request;
cloning a receipt shares that same backing and reservation. A failed capture
returns the original request refusal or a typed physical allocation refusal,
including its known byte quote. It never publishes incomplete observation evidence.
The [native custody court](../../../worth-query-execution/src/domain_computation/primary_graph/application_query/observed_source/retained_parameters/tests.rs)
checks actual backing funding, meaning sharing and release at the last receipt.

Result-set membership remains a separate meaning. Its selection footprint is not
a union of projected row reads: many rows retain their own exact field footprints,
and an empty result has an empty row roster with its original selection evidence.
The [many and empty query court](../../../worth-query-execution/src/domain_computation/primary_graph/causal_inspection/application_observation/declared_query/tests.rs)
executes both through actual query admission and completion. A future inspector
must admit requested metadata before traversing these retained footprints. This
capture grants no fresh permission and does not yet associate a producer chain.

### Optional Query Inspection Metadata Meaning

The declaration facade exposes
`ApplicationQueryInspectionDefinition<Schema, QueryBinding>::new(metadata_contract)`
and the schema builder's `application_query_inspection(definition)`. This attaches
optional metadata meaning to the existing complete query binding; it requires no
additional inspection marker. See the
[typed declaration and installation court](../../../worth-query-installation/src/application_principal_binding_tests/inspection.rs).

`ApplicationQueryInspectionMetadataRef<Schema, QueryBinding>` selects one of six
closed facets: `ObservationIdentity`, `EvidenceLinks`, `EvidenceCounts`,
`EvidenceAvailability`, `Explanation` and `DiagnosticFields`. A governed contract
uses `disclose_inspection_metadata_by` for every facet exactly once. Counts need
`Count` influence and the other facets need `Explanation`. Each rule belongs to
the full binding, including its input, query, result, scope and principal meaning;
matching names cannot transfer a rule to another binding. Explicit public metadata
uses `ApplicationQueryDisclosureContract::public()` without rules.

This is portable meaning only. Installation resolves every definition against the
actual compiled query bindings, then returns typed `InspectionUnavailable` until
the fresh Host issuer is implemented. The ordinary binding without this optional
definition continues to install. Payload disclosure grants no metadata access,
and a diagnostic-fields metadata rule grants no field values: those still require
the separately admitted current subject's diagnostic mask. The Archive encoding
adds member45/v1 and selector8/v1 without changing unextended schema bytes or
identity. These declarations do not complete the Host inspection entrance.

An application mutation may independently declare its original source-disclosure
meaning with `ApplicationSourceDisclosureBinding<Schema>` and the mutation macro's
`inspection_disclosure Binding,` clause. The binding names the actual source and
installed disclosure query, and maps the original typed input to query parameters.
Installation checks that query's scope and principal interpretation against the
source. `ApplicationQueryBindingReference` is the portable descriptive association;
it grants no permission and must resolve against the actual installed inventory.

Original source capture retains the genuine scope and admitted disclosure
parameters on the source receipt's existing owner. Accounted requests, including
an actual accounted ancestor, fund framework retention through their original
execution request; an opaque application
parameter mapper's returned heap is admitted after materialization, before
framework retention. Unaccounted serial requests keep their native posture and
original control checkpoints, without a fabricated accounting reservation.
Inspection alone retains neither the business input nor a correction contract.
Correction keeps its additional input/inverse custody where required. These
captures provide evidence for later fresh admission; the ordinary and governed
Host inspection entrances and connected producer-chain journey remain unfinished.

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
