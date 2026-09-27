# Causal Inspection

Causal inspection answers questions like “why did this observation change?” from an existing `QueryObservationReceipt` and the Query-minted inspection basis that authorizes that scope.

Before building a plan, callers can inspect the supported families and later-milestone debt:

```rust
let support = CausalInspection::support();
let rows = support.rows();
let explanation = support.explain();
```

Use the common path when you want a finished Query inspection artifact:

```rust
let inspection_basis = basis_lifecycle()
    .current_head()
    .inspect()?;

let plan = CausalInspection::for_observation(receipt, inspection_basis)
    .why_changed()
    .reference_only()
    .include_all_retained_evidence()
    .plan()?;

let artifact = plan.materialize_with_bridge(&bridge)?;
```

The receipt and scoped basis must match the pairing captured when the observation receipt was created. A mismatch returns a typed `BasisMismatch` before anchoring, evidence resolution, or bridge-envelope assembly.

The plan is intentionally cheap. It anchors the observation, resolves Query evidence references, builds the inspection target, creates the inspection request, and runs Query admission. It does not ask the runtime bridge to assemble a causal envelope until `materialize_with_bridge`.

## Plan Before Materialize

Inspect the plan when a caller needs to understand the posture before touching bridge-owned retained records:

```rust
let posture = plan.support_posture();
let evidence = plan.required_evidence();
let trace = plan.decision_trace();
let cost = plan.estimated_cost();
let explanation = plan.explain();
let basis = plan.inspection_basis();
```

`estimated_cost()` exposes the planned anchor, reference-resolution, admission,
bridge-assembly, and evidence-reference counts. A denied Query-admission plan
reports zero bridge-envelope assembly before materialization.

Read-only reporting projections are available for certification and diagnostics:

```rust
let anchor = plan.anchor_for_reporting();
let references = plan.reference_set_digest();
let request = plan.request_for_reporting();
let admission = plan.admission_digest();
```

## Outcomes

Admitted plans materialize through the runtime bridge and produce admitted Query artifacts.

Advisory plans are still materializable, but the plan exposes the narrowed posture before bridge materialization. A materialized-detail request currently narrows to reference-only until the bridge envelope is assembled.

Denied Query-admission plans materialize directly as denied Query artifacts. They do not assemble a runtime bridge envelope.

If Query admission succeeds but runtime bridge envelope assembly is denied, materialization returns a denied Query artifact carrying the bridge denial kind, family, and digest.

## Reading Artifacts

The artifact enum exposes the common inspection surface without requiring callers to stitch Query, bridge, signal, or relational APIs together:

```rust
let result = artifact.primary_result();
let warnings = artifact.warnings();
let trace = artifact.decision_trace();
let bindings = artifact.authority_bindings();
let evidence = artifact.evidence();
let integrity = artifact.integrity();
let receipt = artifact.receipt();
let denial = artifact.denial_reason();
let advisory = artifact.advisory_reason();
```

Authority bindings and evidence are Query-owned artifact views over the bridge envelope bindings. Relational and signal evidence remain owned by their authority crates; Query reports references and bridge-bound evidence summaries rather than rebuilding those authority APIs.

## Advanced Inspection

Use the common `CausalInspection` builder for product, operator, and downstream-domain workflows. Query keeps observation anchoring, reference resolution, request assembly, and admission transitions internal so they cannot become a second public authority path. Certification code exercises those boundaries through its dedicated tooling surface.

## Boundary Rule

Do not stitch runtime bridge, signal, or relational APIs directly to answer causal-inspection questions. The common path preserves the boundary:

- Query owns scoped inspection admission, observation anchoring, evidence references, request admission, and Query artifacts.
- Runtime bridge owns causal envelope assembly.
- Signal and relational crates own their evidence and authority records.

## Later Debt

Durable causal archives, store-backed replay reconstruction, restart-stable reload, and persisted causal narrative materialization are deferred. Current causal inspection reports those families as denied or later-milestone posture instead of inventing a partial archive.
