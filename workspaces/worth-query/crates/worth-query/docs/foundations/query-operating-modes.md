# Query Operating Modes

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](ordinary-application-front-door.md).

## What This Feature Is

Query operating modes describe **how execution, artifacts, and subscriptions are backed** in the current runtime—not a second public API surface. Today the honest default is **runtime-backed**: plans, receipts, live state, and inspection evidence live in the in-process Query runtime. **Store-backed execution**, **durable cursors**, and **restart-stable subscription metadata** are explicit deferred debt, not implied by type names or facade exports.

Use this doc when you need to know whether a capability assumes process-local runtime state or durable store semantics.

## Why You Use It

- choose APIs that match what is actually admitted today
- distinguish supported runtime-local recovery from restart-durable recovery
  that remains `Deferred` or `BlockedOnWorthStore`
- read support-matrix rows with the right mental model (`Verified` vs debt)
- explain why saved-query persistence and query-context reload behave as ephemeral/runtime-owned

## Core Mental Model

Three layers stack:

1. **Capability families** (`WorthQueryCapabilityFamily` in application support) — coarse lanes like `DurableArtifacts` marked deferred at the registry level.
2. **Runtime public support** (`WorthQueryRuntimePublicSupportMatrix` / `runtime/support/profile.rs`) — `StoreBackedExecution`, `DurableArtifacts`, and related rows.
3. **Lane-specific profiles** — saved-query (`runtime_backed_saved_query_support_profile`), query-context, subscription matrix rows.

**Runtime-backed** means admission, execution, and evidence are coherent inside the current workspace/runtime instance. **Store-backed** means reload, cross-process continuity, or worth-store durability—largely **not** verified on the public path today.

Application-aftermath recovery is runtime-backed today. Its exact handle is
process-local and receipt-bound. Store-backed handle reload and cross-process
recovery authority remain deferred; an opaque wire identity is not a live
handle.

## Main Entry Points

| Area | Symbols / modules |
|------|-------------------|
| Capability registry | `application/support/registry.rs` — `WorthQueryCapabilityFamily` |
| Runtime profile | `runtime/support/profile.rs` — `StoreBackedExecution`, `DurableArtifacts` |
| Saved queries | `saved_query/support.rs` — `SavedQueryPersistenceFamily::EphemeralProcessOwned` (debt) |
| Query context | `query_context/support.rs` — reload/store rows |
| Facade workspace | `WorthQueryRuntime`, `WorthQueryWorkspace` — same runtime instance for compose/execute/live |

There is no separate `query_operating_mode()` switch on the facade; mode is **derived from support profiles** and admission receipts, not a user-facing enum you set at startup.

## Typical Flow

1. Open or build a `WorthQueryRuntime` / workspace (runtime-backed instance).
2. Admit and execute through normal compose/execute or live/subscription lanes.
3. If you need durability, read `runtime_backed_*_support_profile()` for the lane **before** assuming persistence.
4. For saved queries or query context, treat process-owned artifacts as **convenience**, not restart contracts, until store-backed rows flip to `Verified`.

```text
Host app
  └── WorthQueryRuntime (process-local)
        ├── plans / receipts / live graph
        ├── inspection evidence (retained)
        └── [deferred] worth-store reload / durable cursors
```

## How It Relates

- [Support matrix and admission](support-matrix-and-admission.md) — authoritative Verified / Deferred / Forbidden tables
- [Saved query and query context](../authoring/read-composition.md) — authoring that may imply persistence; check profiles first
- [Subscription selection and diagnostics](../capabilities/subscription-selection-and-diagnostics.md) — durable replay deferred in subscription matrix
- [Basis capability lifecycle](../capabilities/basis-capability-lifecycle.md) — basis envelopes: store-backed reload deferred

- [Application aftermath, external effects, and recovery](../execution/application-aftermath-and-recovery.md) — runtime-local recovery and its durable boundary

## Good to Know

- API names containing “saved”, “context”, or “subscription” do **not** guarantee worth-store durability.
- Certification and harness tests often run runtime-backed; do not generalize to multi-process deployment without checking matrix rows.
- `DurableArtifacts` at the capability-family level is deferred—treat durable artifact stories as roadmap debt unless a lane-specific profile says `Verified`.

- A supported lost-response or safe-retry journey inside one runtime does not
  imply restart durability.

## Anti-Patterns

- Assuming subscription identity survives process restart without checking subscription support matrix.
- Building “write to DB then reload query context from store” on `BlockedOnWorthStore` rows.
- Treating `Debt` or `Deferred` in a profile as “works in dev only”—it means **not a supported public contract** yet.

## Current Limits

From application/runtime/saved-query/query-context support profiles (representative rows):

| Surface / family | Status |
|------------------|--------|
| Runtime-backed execution, live, inspection | **Verified** (default honest path) |
| Runtime-local receipt-bound application-aftermath recovery | **Verified** |
| Store-backed recovery-handle reload | **Deferred** |
| `WorthQueryCapabilityFamily::DurableArtifacts` | **Deferred** |
| `StoreBackedExecution` (runtime public matrix) | **Deferred** / store-blocked neighbors |
| Saved query `EphemeralProcessOwned` persistence | **Debt** |
| Query-context store reload | **Deferred** / blocked per `query_context/support.rs` |
| Durable subscription replay metadata | **Deferred** (see subscription matrix) |

Temporal/time-aware runtime semantics are already carried by ordinary live
handles. Their separate durable/store-backed neighbors remain governed by the
[support matrix](support-matrix-and-admission.md).

## Related Docs

- [Support matrix and admission](support-matrix-and-admission.md)
- [Policy, tenant, and relationship-proof narrowing](policy-tenant-and-relationship-proof-narrowing.md)
- [Region-scoped live invalidation and stream contracts](../runtime-surfaces/region-scoped-live-invalidation-and-stream-contracts.md)
- [AI agent orientation](../AI_README.md) — Operating modes category
