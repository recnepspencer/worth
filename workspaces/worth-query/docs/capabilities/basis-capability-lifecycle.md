# Basis Capability Lifecycle

## What This Feature Is

The basis capability lifecycle lets an application choose which version of
truth a Query operation may use. A **basis** is that chosen world: the current
head, a branch or snapshot, a preview, retained history, or a tenant- and
policy-scoped view. You declare the world and the operation; Query returns one
sealed capability for that operation or a typed denial.

## Why You Use It

- Pin observation, inspection, replay, or materialization to the intended
  truth world.
- Prepare a mutation without allowing a different basis to enter its plan.
- Bind subscription declaration and activation to the same admitted basis.
- Reject stale previews, inaccessible branches, and policy or tenant conflicts
  before operational work begins.
- Preserve one inspectable authority chain through lower-runtime evidence and
  the final use receipt.

## Stable Entry Points

The lifecycle is owned by Admission. Hosts and certification code import it
from the Host facade:

```rust
use worth_query_host::facade::admission::basis::basis_lifecycle;
```

Choose a world with `current_head()`, `branch_head(...)`,
`branch_snapshot(...)`, `preview(...)`, `runtime_snapshot(...)`,
`historical_snapshot(...)`, `historical_commit(...)`, `tenant_scoped(...)`, or
`policy_scoped(...)`.

Choose what the caller intends to do with that world:

- `observe()`
- `prepare_mutation()`
- `replay()`
- `inspect()`
- `materialize()`
- `declare_subscription()`
- `close_preview()`

The corresponding `for_*() -> admit() -> scope()` path is the advanced form of
the same transition. Use it when infrastructure code needs to inspect the
admission phase. It is not a separate authority route.

Use `discover_basis_lifecycle_support(...)` when support may be advisory,
deferred, or unsupported.

## Core Mental Model

A declaration describes what the caller wants. It is not authority. Query
normalizes the declaration, checks the requested operation, and mints a scoped
capability whose fields cannot be assembled by the caller.

The capability remembers:

- the admitted basis family
- which subsystem supplied authority for that world
- lifecycle and visibility posture
- the exact operation lane
- any expected lower-runtime binding
- a structural identity for this admitted capability

Digests and labels exposed by the capability are reporting projections. They
help caching, evidence, and debugging; they cannot be turned back into
operational authority.

## How It Executes

```text
basis declaration
  -> normalization
  -> support and eligibility decision
  -> sealed operation-scoped capability
  -> lower-runtime evidence binding, when required
  -> basis use receipt
  -> self-describing evidence envelope
```

A denial stops before the scoped capability exists. Lower-runtime evidence is
checked against the capability rather than trusted because its label or digest
looks familiar.

## Small Example

```rust
use worth_query_host::facade::admission::basis::basis_lifecycle;

let observation_basis = basis_lifecycle()
    .current_head()
    .observe()?;

assert_eq!(observation_basis.family().as_str(), "current_head");
```

This is the smallest honest call. `observation_basis` is already the sealed
proof required by the observation lane; the application does not assemble or
stamp it.

## Selection is not snapshot admission

Selecting a world, for example `runtime_snapshot(snapshot, binding)`, declares
which world the operation must use and records the expected lower-runtime
binding. It does not observe the snapshot or prove that it exists. The scoped
capability is authority for the declared lane only.

Proof that a snapshot exists can only come from the lower runtime that actually
resolved it. `readmit_lower_runtime_evidence` checks a
`LowerRuntimeBasisEvidence` against the capability's expected binding before
`emit_observation_basis_receipt` emits a use receipt. Construct that evidence
only from an owner-issued observation: `LowerRuntimeBasisEvidence::from_runtime_bridge_facade`
stores the identity, digest and version it is given and proves nothing by
itself. No Host route issues Runtime Bridge snapshot evidence for this
readmission today, so applications use the scoped capability directly, as in
the small example, and do not emit lower-runtime basis-use receipts.

## Subscriptions

For subscriptions, derive activation from the admitted declaration proof:

```rust
use worth_query_host::facade::admission::basis::{
    activate_subscription_basis, basis_lifecycle,
};

let declaration_basis = basis_lifecycle()
    .current_head()
    .declare_subscription()?;
let activation_basis = activate_subscription_basis(&declaration_basis);
```

This keeps subscription activation tied to the exact declaration authority.

## Policy, tenant and relationship-proof admission

Admission also owns the policy and tenant context a query runs under and the
relationship-proof descriptors that narrow it. Host re-exports them as
`worth_query_host::facade::admission::{policy, tenant, relationship}`.

- `policy::admit_policy_tenant_context(query, policy, tenant, branch, schema,
  mode)` admits an `AdmittedPolicyTenantContext` for one
  `CanonicalQueryArtifact`, or refuses with `PolicyTenantAdmissionError`.
- `relationship::admit_relationship_proofs(query, &admitted, &descriptors)`
  admits a `RelationshipProofDescriptorSet` against that context. Descriptors
  are built with `direct_edge`, `bounded_ancestor`, `bounded_descendant` and
  `tenant_membership`.
- The topology descriptors must cover the query's canonical traversal exactly:
  each traversal step is matched by a descriptor for the same relation with
  enough depth, and every topology descriptor is used. Anything else refuses
  with `QueryShapeMismatch`. A descriptor's policy digest must match the
  admitted policy (`PolicyMismatch`), and a tenant-membership descriptor must
  match the admitted tenant schema (`TenantSchemaMismatch`).
- The set's `RelationshipProofBudget` bounds descriptor count and topology
  width (`RelationshipProofBudgetExceeded`, `UnboundedProofTopology`).
  Unbounded recursive walks and host callback descriptors always refuse
  (`UnboundedRecursiveWalk`, `HostCallbackForbidden`).

A successful admission proves only that the descriptors are legal for this
query, policy and tenant context. It does not evaluate the relationship at
runtime or prove that it holds:
`relationship::runtime_backed_relationship_proof_support_profile()` reports
descriptor admission and the four topologies as verified, runtime proof
evaluation as deferred, and host callback proofs as forbidden. It also grants
no request permission. Each request still passes its own capability
authorization; see
[Application Authorization And Emergency Elevation](./application-authorization-and-emergency-elevation.md).

## How It Relates To Other Features

- [Cross-Runtime Causal Inspection](./cross-runtime-causal-inspection.md)
  requires inspection authority (`ScopedInspectionBasis`) in addition to causal
  observation evidence.
- [Branches And Previews](../foundations/branches-and-previews.md) covers the
  product branches whose heads and snapshots a basis may select.

## Inspection And Debugging

Use typed errors first:

- `BasisLifecycleDeclarationError::intent_denial()` reports malformed or
  unsupported declarations.
- `BasisLifecycleDeclarationError::eligibility_denial()` reports why a valid
  declaration could not enter the requested lane.
- `discover_basis_lifecycle_support(...)` reports admitted, advisory,
  deferred, or unsupported posture for a family and operation.
- Scoped capability getters report family, authority, lifecycle, and identity
  without exposing construction fields.
- A basis use receipt and envelope report which lower-runtime evidence was
  actually consumed.

## Anti-Patterns

- Passing branch, snapshot, commit, preview, tenant, or policy strings directly
  into operational code.
- Treating a declaration draft, label, or digest as a scoped capability.
- Rebuilding admission, scoping, or capability identity in a consumer crate.
- Activating a subscription from independently selected basis information.
- Building `LowerRuntimeBasisEvidence` from caller-chosen strings and reporting
  the resulting receipt as evidence of an observed snapshot.
- Importing certification tooling as an ordinary runtime API.
- Treating a deferred store or durable lane as runtime-backed support.

## Current Limits

- Runtime-backed current, branch, preview, historical, tenant, and policy lanes
  are usable only where support discovery admits the requested operation.
- Preview-derived inspection may be advisory and cannot be promoted silently.
- Store-backed parity and durable capability reload remain deferred.
- Temporal and async-resource basis families are not ordinary operational
  entry points.

## Related Docs

- [Cross-Runtime Causal Inspection](./cross-runtime-causal-inspection.md)
- [Branches And Previews](../foundations/branches-and-previews.md)
