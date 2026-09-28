# Lower-Runtime Capability Routing

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Lower-runtime capability routing is Query’s **declared path for contacting** relational, bridge, and signal runtimes: route plans, boundary envelopes, eligibility receipts, and `worth_query_lower_runtime_support_matrix()`—without domain code importing lower crates directly. **Compatibility debt** rows still exist; `TemporalQueryBasisRoutingNeighbor` is **deferred**.

## Why You Use It

- route live, subscription, write-authority, and signal invalidation through boundary receipts
- inspect closeout and boundary summaries for agents (`inspect_lower_runtime_boundary`)
- stay on certification-tested public inventory rows
- avoid bypassing envelopes to call `worth-relational` / `worth-signal` from domain modules

## Core Mental Model

`lower_runtime_routing/` modules:

| Piece | Role |
|-------|------|
| `plans/` / `protocol/` | Route plans and protocol shapes |
| `envelopes/` | `WorthQueryLowerRuntimeBoundaryEnvelope`, cost/failure topology |
| `eligibility/` | Capability eligibility and posture |
| `receipts/` | Boundary receipts (live, subscription, write authority, signal) |
| `support/` | `worth_query_lower_runtime_support_matrix()` |
| `dx/` | `inspect_lower_runtime_boundary`, `summarize_lower_runtime_boundary` |

This is the **execution contact** layer; [declaration bridge continuation routing](declaration-bridge-continuation-routing.md) covers declaration-entry continuation—link both, do not merge.

```text
Domain declaration (no lower imports)
  → lower_runtime route plan + eligibility
  → boundary envelope + receipt
  → relational / bridge / signal runtime
```

## Main Entry Points

Public exports (`lower_runtime_routing/mod.rs`):

- Adapters: `LiveViewDeclarationAdmissionBoundaryReceipt`, `SubscriptionActivationBoundaryReceipt`, `WriteAuthorityExecutionReceipt`, `SignalInvalidationBoundaryReceipt`
- DX: `inspect_lower_runtime_boundary`, `WorthQueryLowerRuntimeRoutingInspection`, `WorthQueryLowerRuntimeBoundarySummary`
- Support: `worth_query_lower_runtime_support_matrix()`
- Certification suite types (for harness/CI, not app hot path)

Tests: `lower_runtime_routing/certification/tests.rs`, UI `tests/ui/lower_runtime_routing/`.

## Typical Flow

1. Admit domain capability with lower-runtime eligibility for the target family.
2. Build route plan with boundary envelope category and cost posture.
3. Execute through boundary receipt (live view, subscription activation, or write authority).
4. On failure, read failure topology from envelope—do not bypass to raw runtime APIs.
5. For diagnostics: `inspect_lower_runtime_boundary` / `summarize_lower_runtime_boundary`.

## How It Relates

- [Declaration bridge continuation routing](declaration-bridge-continuation-routing.md) — continuation at declaration entry
- [Live views](../runtime-surfaces/live-views.md) — live admission consumes boundary receipts
- [Region-scoped live](../runtime-surfaces/region-scoped-live-invalidation-and-stream-contracts.md) — locality + stream lowering
- [Cross-runtime causal inspection](../capabilities/cross-runtime-causal-inspection.md) — materialized detail vs bridge envelopes

## Good to Know

- `certify_lower_runtime_routing` and acceptance suite are certification artifacts—apps use receipts and support matrix, not closeout manifests, in production paths.
- Focused boundary tests enforce **non-bypass** from domain code, while
  ordinary integration tests exercise the receipt path. Add a compiler fixture
  only when compiler rejection itself is the public guarantee.

## Anti-Patterns

- `use worth_relational::*` (or signal/bridge) inside domain capability modules.
- Treating routing as permission to skip Query admission/support profiles.
- Assuming temporal query basis routing works because a neighbor name exists—check deferred row.

## Current Limits

From `worth_query_lower_runtime_support_matrix()` (see `lower_runtime_routing/support.rs` for full rows):

| Neighbor / concern | Status |
|--------------------|--------|
| Relational / bridge / signal boundary contact (runtime-backed) | **Admitted** on verified rows |
| Compatibility debt families | **Debt** — check matrix before claiming parity |
| `TemporalQueryBasisRoutingNeighbor` | **Deferred** |
| Direct lower-crate import from domains | **Forbidden** (certification non-bypass) |

## Related Docs

- [Declaration bridge continuation routing](declaration-bridge-continuation-routing.md)
- [Support matrix and admission](../foundations/support-matrix-and-admission.md)
- [Contribution composed orchestration](contribution-composed-orchestration.md)
- [Conditional Installed Operations](conditional-installed-operations.md)
- [Public doc coverage](public-doc-coverage.md)
