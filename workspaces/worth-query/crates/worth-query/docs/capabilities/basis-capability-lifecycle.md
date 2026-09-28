# Basis Capability Lifecycle

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

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

Import the lifecycle from the foundation facade:

```rust
use worth_query::facade::foundation::basis_lifecycle;
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
deferred, or unsupported. Certification-only inventories and matrices live in
`worth_query::facade::certification`, not in ordinary product code.

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
use worth_query::facade::foundation::basis_lifecycle;

let observation_basis = basis_lifecycle()
    .current_head()
    .observe()?;

assert_eq!(observation_basis.family().as_str(), "current_head");
```

This is the smallest honest call. `observation_basis` is already the sealed
proof required by the observation lane; the application does not assemble or
stamp it.

## Real Example

```rust
use worth_query::facade::foundation::{
    basis_lifecycle, emit_observation_basis_receipt, envelope_basis_use,
    readmit_lower_runtime_evidence, LowerRuntimeBasisEvidence,
};

let scoped = basis_lifecycle()
    .runtime_snapshot(
        "workspace-generation-42",
        "runtime:workspace-generation-42",
    )
    .observe()?;

let bound = readmit_lower_runtime_evidence(
    scoped,
    LowerRuntimeBasisEvidence::from_runtime_bridge_facade(
        "runtime:workspace-generation-42",
        "workspace-observation-evidence",
        1,
    ),
)?;

let receipt = emit_observation_basis_receipt(bound);
let envelope = envelope_basis_use(receipt);

assert_eq!(envelope.receipt().basis_family().as_str(), "runtime_snapshot");
```

Query owns admission and capability identity. The runtime bridge contributes
evidence for the snapshot it actually resolved. Readmission proves that the
evidence matches the sealed capability before a use receipt is emitted. The
envelope can be inspected or reported, but its digest does not replace the
capability it describes.

For subscriptions, derive activation from the admitted declaration proof:

```rust
use worth_query::facade::foundation::{
    activate_subscription_basis, basis_lifecycle,
};

let declaration_basis = basis_lifecycle()
    .current_head()
    .declare_subscription()?;
let activation_basis = activate_subscription_basis(&declaration_basis);
```

This keeps subscription activation tied to the exact declaration authority.

## How It Relates To Other Features

- [Historical Diff And Basis](./historical-diff-and-basis.md) binds validated
  queries and comparisons to admitted historical worlds.
- [Subscription Selection And Diagnostics](./subscription-selection-and-diagnostics.md)
  consumes declaration and activation proofs from this lifecycle.
- [Cross-Runtime Causal Inspection](./cross-runtime-causal-inspection.md)
  requires inspection authority in addition to causal observation evidence.
- [Projection Consumption](./projection-consumption.md) can carry facts from a
  basis-bound result into another runtime without reopening the basis.
- Preview execution and drift use the scoped preview-live binding described in
  [Branches And Previews](../foundations/branches-and-previews.md).

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

- [Historical Diff And Basis](./historical-diff-and-basis.md)
- [Support Matrix And Admission](../foundations/support-matrix-and-admission.md)
- [Policy, Tenant, And Relationship-Proof Narrowing](../foundations/policy-tenant-and-relationship-proof-narrowing.md)
- [Branches And Previews](../foundations/branches-and-previews.md)
- [Projection Consumption](./projection-consumption.md)
