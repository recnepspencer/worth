# Policy, Tenant, and Relationship-Proof Narrowing

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](ordinary-application-front-door.md).

## What This Feature Is

Policy and tenant narrowing **shrinks what a query may see or assert** under declared policy/tenant basis and relationship-proof descriptors—before and during read composition. The shipped path is **runtime-backed descriptor admission and masking validation**, not full policy-aware execution/live/historical parity across every surface.

Relationship-proof admission validates **descriptors** on the runtime-backed path; **host authorization callbacks** are forbidden on the public contract.

## Why You Use It

- attach policy/tenant basis to queries without bypassing admission
- produce narrowed policy query artifacts and authorized projection receipts
- admit relationship-proof descriptors with an honest support profile
- avoid conflating “policy narrowed at plan time” with “every live/historical lane is policy-aware today”

## Core Mental Model

Four cooperating areas in the crate (not four separate apps):

| Module area | Role |
|-------------|------|
| `policy_narrowing/` | Masking, influence validation, narrowed policy query artifacts |
| `policy_basis/` / `tenant_basis/` | Basis tokens tied to policy/tenant lanes |
| `relationship_proof/` | Descriptor admission, runtime-backed proof profile |
| Read composition | Operator receipts that record policy/tenant narrowing outcomes |

Narrowing **reduces** admissible shape; it does not replace domain authorization in host code. Store-backed policy durability is **blocked on worth-store**, not a silent fallback.

## Main Entry Points

Facade (`exports_policy.rs` and related):

- `narrow_policy_query` — narrowed policy query artifact path
- `admit_policy_tenant_context` — policy/tenant context admission
- `classify_saved_query_policy_tenant_reuse` / `classify_saved_policy_narrowing_reuse` — reuse classification
- `admit_relationship_proofs` — relationship-proof descriptor admission
- `runtime_backed_relationship_proof_support_profile()` — honesty table

Read composition records narrowing via runtime surface receipts (`runtime/surface/read_composition.rs`); see [read composition](../authoring/read-composition.md) for compose/execute—**policy detail lives here**.

## Typical Flow

1. Declare or load policy/tenant basis (see [basis capability lifecycle](../capabilities/basis-capability-lifecycle.md)).
2. `admit_policy_tenant_context` (or saved-query reuse classification) for the workspace posture.
3. `narrow_policy_query` where the plan requires a narrowed policy artifact.
4. For relationship proofs: `admit_relationship_proofs` with descriptor admission only—no host callback channel.
5. Execute read composition; inspect receipts for masking/authorized projection outcomes.

```text
Basis (policy/tenant)
  → admit context / narrow policy query
  → relationship-proof descriptors (if any)
  → read composition execute
  → receipts (masking, projection admission)
```

## How It Relates

- [Read composition](../authoring/read-composition.md) — execution surface; cross-link for policy receipts only
- [Basis capability lifecycle](../capabilities/basis-capability-lifecycle.md) — phase-typed basis tokens
- [Intent Admission](../execution/intent-admission.md) — intent families vs policy-aware execution debt
- [Support matrix and admission](support-matrix-and-admission.md) — `PolicyTenantAdmissionSupportProfile`

## Good to Know

- **Verified** today: authorized projection, masked influence validation, relationship-proof **descriptor** admission, narrowed policy query artifacts (runtime-backed).
- **Deferred**: policy-aware execution, live, historical diff, delivery—parity across surfaces is not shipped.
- **BlockedOnWorthStore**: store-backed durability for policy narrowing.
- Relationship-proof does not expose a public “call my authz hook” API; descriptors + runtime profile define the contract.

## Anti-Patterns

- Assuming live subscriptions automatically honor the same policy narrowing as a one-shot read plan.
- Using relationship-proof admission as proof that an external IAM system approved the request.
- Importing store APIs from domain code to “persist policy context” while matrix rows are blocked.

## Current Limits

`runtime_backed_policy_narrowing_support_profile()` (`policy_narrowing/support.rs`):

| Surface | Status |
|---------|--------|
| AuthorizedProjection | **Verified** |
| MaskedInfluenceValidation | **Verified** |
| RelationshipProofDescriptorAdmission | **Verified** |
| NarrowedPolicyQueryArtifact | **Verified** |
| PolicyAwareExecution | **Deferred** |
| PolicyAwareLive | **Deferred** |
| PolicyAwareHistoricalDiff | **Deferred** |
| PolicyAwareDelivery | **Deferred** |
| StoreBackedDurability | **BlockedOnWorthStore** |

Relationship-proof: descriptors **verified** on runtime-backed profile; host authorization callbacks **forbidden** on public API.

## Related Docs

- [Read composition](../authoring/read-composition.md)
- [Basis capability lifecycle](../capabilities/basis-capability-lifecycle.md)
- [Support matrix and admission](support-matrix-and-admission.md)
- [Historical diff and basis](../capabilities/historical-diff-and-basis.md) — historical lane; policy-aware historical diff deferred
