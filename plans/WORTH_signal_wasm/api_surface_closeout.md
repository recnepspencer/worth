# API Surface Closeout Acceptance Map

> **Status:** Completed
>
> **Spec:** [api_surface_plan.md](./api_surface_plan.md)
>
> **Certification spec:** [test-requirements.md](./test-requirements.md)
>
> **Roadmap parent:** [wasm_product_roadmap.md](./wasm_product_roadmap.md)
>
> **Prerequisite spec:** [opaque_identity_and_ergonomic_authoring_plan.md](./opaque_identity_and_ergonomic_authoring_plan.md)

## Purpose

This document maps the `worth-signals-wasm` API-surface milestone to concrete
implementation and certification evidence.

It is the closeout ledger for the hostile question:

> Can `worth-signals-wasm` now host detail, collection, paged, request-shaped,
> patch-capable, diagnostics-rich, branch-aware, download-aware, delivery-aware,
> and externally-compatible resource lines as one coherent local product model
> without minting a second async, freshness, cache, or delivery truth engine?

## Closeout Summary

Milestone 6 is now implemented as a first-class resource/API product surface on
top of runtime-owned async, lifecycle, policy, branch, restore, and diagnostics
truth.

The implementation now includes:

- typed detail, collection, and paged resource families under one canonical
  line model
- typed parameter normalization and stable family/member identity
- one canonical line facade with:
  - `value()`
  - `status()`
  - `freshness()`
  - `request()`
  - `diagnostics()`
  - `diagnosticsSummary()`
  - `history()`
  - `download()`
  - `processing()`
  - `upload()`
  - line-scoped `view(...)`
- runtime-lowered refresh, revalidate, retry, timeout, supersession, and
  continuity behavior
- named request/auth/context/continuation/processing/upload posture
- declaration-driven item/aspect/summary reconciliation and honest broad
  fallback
- typed binary/download descriptor truth kept distinct from structured value
- transport-neutral live delivery plus explicit basis compatibility
- branch-aware history, exact restore, and exact replay line actions
- canonical verification-package emission for hostile proof comparison
- external-definition and external-delivery compatibility boundaries that
  converge on the same local materialization model
- a full hostile suite-0 convergence lane that compares forward, restore,
  retained-history, and replay modes without inventing a second product truth

The direct closeout gates are:

- [api_surface_plan.md](./api_surface_plan.md)
- [test-requirements.md](./test-requirements.md)
- [resource.runtime.test.mjs](../crates/worth-signal-wasm/package/product/resource.runtime.test.mjs)
- [resource_surface_usage.ts](../crates/worth-signal-wasm/package/resource_types_smoke/resource_surface_usage.ts)
- [resource_authoring_denials.ts](../crates/worth-signal-wasm/package/resource_types_smoke/resource_authoring_denials.ts)

Those five surfaces matter together:

- the engineering spec defines the intended product model
- the certification spec defines the hostile proof contract
- the runtime suite owns the runtime-hostile certification lanes, including
  suite 0
- the type-smoke and boundary surfaces close the compile-time and declaration
  boundary obligations the certification spec names explicitly

## Primary Implementation Surfaces

Resource product surface and family model:

- [package-src/product/signals.ts](../crates/worth-signal-wasm/package-src/product/signals.ts)
- [package-src/product/resource/facade.ts](../crates/worth-signal-wasm/package-src/product/resource/facade.ts)
- [package-src/product/resource/families/detail_family.ts](../crates/worth-signal-wasm/package-src/product/resource/families/detail_family.ts)
- [package-src/product/resource/families/collection_family.ts](../crates/worth-signal-wasm/package-src/product/resource/families/collection_family.ts)
- [package-src/product/resource/families/paged_family.ts](../crates/worth-signal-wasm/package-src/product/resource/families/paged_family.ts)
- [package-src/product/resource/families/materialization/materialized_family_factory.ts](../crates/worth-signal-wasm/package-src/product/resource/families/materialization/materialized_family_factory.ts)
- [package-src/product/resource/identity/runtime_line_identity.ts](../crates/worth-signal-wasm/package-src/product/resource/identity/runtime_line_identity.ts)

Canonical line execution, evidence, and history:

- [package-src/product/resource/lines/line_handle.ts](../crates/worth-signal-wasm/package-src/product/resource/lines/line_handle.ts)
- [package-src/product/resource/lines/line_patch_capable_handle.ts](../crates/worth-signal-wasm/package-src/product/resource/lines/line_patch_capable_handle.ts)
- [package-src/product/resource/lines/actions/line_reload_execution.ts](../crates/worth-signal-wasm/package-src/product/resource/lines/actions/line_reload_execution.ts)
- [package-src/product/resource/lines/actions/line_patch_execution.ts](../crates/worth-signal-wasm/package-src/product/resource/lines/actions/line_patch_execution.ts)
- [package-src/product/resource/lines/actions/line_delivery_execution.ts](../crates/worth-signal-wasm/package-src/product/resource/lines/actions/line_delivery_execution.ts)
- [package-src/product/resource/lines/history/line_history_restore.ts](../crates/worth-signal-wasm/package-src/product/resource/lines/history/line_history_restore.ts)
- [package-src/product/resource/lines/history/line_history_replay.ts](../crates/worth-signal-wasm/package-src/product/resource/lines/history/line_history_replay.ts)
- [package-src/product/resource/lines/history/line_verification_package.ts](../crates/worth-signal-wasm/package-src/product/resource/lines/history/line_verification_package.ts)
- [package-src/product/resource/lines/reads/line_history_read.ts](../crates/worth-signal-wasm/package-src/product/resource/lines/reads/line_history_read.ts)
- [package-src/product/resource/lines/reads/line_history_availability_read.ts](../crates/worth-signal-wasm/package-src/product/resource/lines/reads/line_history_availability_read.ts)

Compatibility, downloads, and delivery:

- [package-src/product/resource/compatibility/resource_compatibility_namespace.ts](../crates/worth-signal-wasm/package-src/product/resource/compatibility/resource_compatibility_namespace.ts)
- [package-src/product/resource/compatibility/resource_external_definition.ts](../crates/worth-signal-wasm/package-src/product/resource/compatibility/resource_external_definition.ts)
- [package-src/product/resource/compatibility/resource_external_delivery.ts](../crates/worth-signal-wasm/package-src/product/resource/compatibility/resource_external_delivery.ts)
- [package-src/product/resource/delivery/resource_delivery.ts](../crates/worth-signal-wasm/package-src/product/resource/delivery/resource_delivery.ts)
- [package-src/product/resource/downloads/resource_binary_descriptor.ts](../crates/worth-signal-wasm/package-src/product/resource/downloads/resource_binary_descriptor.ts)
- [package-src/product/resource/downloads/resource_binary_value.ts](../crates/worth-signal-wasm/package-src/product/resource/downloads/resource_binary_value.ts)
- [package-src/product/resource/downloads/resource_download.ts](../crates/worth-signal-wasm/package-src/product/resource/downloads/resource_download.ts)

Typed public surface:

- [package/types/resource/resource_namespace.d.ts](../crates/worth-signal-wasm/package/types/resource/resource_namespace.d.ts)
- [package/types/resource/resource_declarations.d.ts](../crates/worth-signal-wasm/package/types/resource/resource_declarations.d.ts)
- [package/types/resource/resource_family_surfaces.d.ts](../crates/worth-signal-wasm/package/types/resource/resource_family_surfaces.d.ts)
- [package/types/resource/resource_postures.d.ts](../crates/worth-signal-wasm/package/types/resource/resource_postures.d.ts)
- [package/types/resource/resource_reconciliation.d.ts](../crates/worth-signal-wasm/package/types/resource/resource_reconciliation.d.ts)
- [package/types/resource/resource_lifecycle.d.ts](../crates/worth-signal-wasm/package/types/resource/resource_lifecycle.d.ts)
- [package/types/resource/resource_verification.d.ts](../crates/worth-signal-wasm/package/types/resource/resource_verification.d.ts)
- [package/resource_types_smoke/resource_surface_usage.ts](../crates/worth-signal-wasm/package/resource_types_smoke/resource_surface_usage.ts)
- [package/resource_types_smoke/resource_authoring_denials.ts](../crates/worth-signal-wasm/package/resource_types_smoke/resource_authoring_denials.ts)

## Must-Ship Acceptance Map

| Spec requirement | Implementation evidence | Certification evidence |
| --- | --- | --- |
| Family-first resource authoring with canonical identity | family declarations, materialized family factory, runtime line identity | authoring identity and facade suites plus suite 0 |
| Runtime-owned lifecycle, continuity, retry, timeout, and revalidation truth | reload execution, diagnostics/freshness/status state, request state | lifecycle parity, timeout/retry, and async-first suites |
| Typed request/auth/context/continuation/processing/upload posture | request lowering, typed declaration surface, line request reads | request, continuation, processing, and upload suites |
| Narrow item/aspect/summary reconciliation with honest broad fallback | reconciliation declarations, patch lowering, patch execution | phase 4 reconciliation suites and no-side-effect denial coverage |
| Diagnostics/history/branch/restore/replay honesty | history availability, restore action, replay action, verification package | phase 5 suites, replay action suite, suite 0 |
| Binary/download and live-delivery convergence | binary/download vocabulary, delivery execution, basis history | phase 6 download and delivery suites |
| External-definition and external-delivery compatibility without a second engine | compatibility namespace, external definition/delivery lowering | phase 7 compatibility suites and suite 0 |

## Required Acceptance Evidence

The engineering spec names representative proof families, while the
certification authority in [test-requirements.md](./test-requirements.md)
defines suites 1 through 28 plus suite 0 as the actual closure bar.

Those obligations now map to two evidence classes:

- runtime-hostile certification lanes under
  [resource.runtime.test.mjs](../crates/worth-signal-wasm/package/product/resource.runtime.test.mjs)
- compile-time and declaration-boundary evidence under
  [resource_surface_usage.ts](../crates/worth-signal-wasm/package/resource_types_smoke/resource_surface_usage.ts)
  and
  [resource_authoring_denials.ts](../crates/worth-signal-wasm/package/resource_types_smoke/resource_authoring_denials.ts)

Representative closeout owners include:

- Phase 1 identity/facade/view lanes:
  - [family_identity_equivalence.test.mjs](../crates/worth-signal-wasm/package/product/resource_runtime/authoring/family_identity_equivalence.test.mjs)
  - [line_facade_stability.test.mjs](../crates/worth-signal-wasm/package/product/resource_runtime/authoring/line_facade_stability.test.mjs)
  - [line_view_ownership.test.mjs](../crates/worth-signal-wasm/package/product/resource_runtime/authoring/line_view_ownership.test.mjs)
- Phase 2 lifecycle/freshness/policy lanes:
  - [lifecycle_and_refresh.test.mjs](../crates/worth-signal-wasm/package/product/resource_runtime/lifecycle/lifecycle_and_refresh.test.mjs)
  - [retry_and_timeout.test.mjs](../crates/worth-signal-wasm/package/product/resource_runtime/lifecycle/retry_and_timeout.test.mjs)
  - [invalidation.test.mjs](../crates/worth-signal-wasm/package/product/resource_runtime/lifecycle/invalidation.test.mjs)
  - [async_first_history_parity.test.mjs](../crates/worth-signal-wasm/package/product/resource_runtime/lifecycle/async_first_history_parity.test.mjs)
- Phase 3 request/deferred/upload lanes:
  - [request_posture.test.mjs](../crates/worth-signal-wasm/package/product/resource_runtime/requests/request_posture.test.mjs)
  - [continuation_posture.test.mjs](../crates/worth-signal-wasm/package/product/resource_runtime/requests/continuation_posture.test.mjs)
  - [processing_job.test.mjs](../crates/worth-signal-wasm/package/product/resource_runtime/transfers/processing_job.test.mjs)
  - [upload_transport.test.mjs](../crates/worth-signal-wasm/package/product/resource_runtime/transfers/upload_transport.test.mjs)
- Phase 4 reconciliation lanes:
  - [patch_reconciliation_hardening.test.mjs](../crates/worth-signal-wasm/package/product/resource_runtime/reconciliation/patch_reconciliation_hardening.test.mjs)
  - [patch_reconciliation_mixed_history.test.mjs](../crates/worth-signal-wasm/package/product/resource_runtime/reconciliation/patch_reconciliation_mixed_history.test.mjs)
  - [patch_reconciliation_paged_summary_scope.test.mjs](../crates/worth-signal-wasm/package/product/resource_runtime/reconciliation/patch_reconciliation_paged_summary_scope.test.mjs)
- Phase 5 diagnostics/history/restore/replay lanes:
  - [phase5_history_closeout.test.mjs](../crates/worth-signal-wasm/package/product/resource_runtime/lifecycle/phase5_history_closeout.test.mjs)
  - [branch_restore_action_surface.test.mjs](../crates/worth-signal-wasm/package/product/resource_runtime/inspection/branch_restore_action_surface.test.mjs)
  - [replay_action_surface.test.mjs](../crates/worth-signal-wasm/package/product/resource_runtime/inspection/replay_action_surface.test.mjs)
- Phase 6 binary/download/delivery lanes:
  - [binary_download_surface.test.mjs](../crates/worth-signal-wasm/package/product/resource_runtime/downloads/binary_download_surface.test.mjs)
  - [live_delivery_convergence.test.mjs](../crates/worth-signal-wasm/package/product/resource_runtime/delivery/live_delivery_convergence.test.mjs)
  - [delivery_basis_history_closeout.test.mjs](../crates/worth-signal-wasm/package/product/resource_runtime/delivery/delivery_basis_history_closeout.test.mjs)
- Phase 7 compatibility lanes:
  - [external_definition_compatibility.test.mjs](../crates/worth-signal-wasm/package/product/resource_runtime/authoring/external_definition_compatibility.test.mjs)
  - [external_basis_refresh_compatibility.test.mjs](../crates/worth-signal-wasm/package/product/resource_runtime/delivery/external_basis_refresh_compatibility.test.mjs)
- Suite 0:
  - [full_resource_hostile_convergence.test.mjs](../crates/worth-signal-wasm/package/product/resource_runtime/closeout/full_resource_hostile_convergence.test.mjs)

## Closeout Matrix Coverage

The milestone closeout now owns the combined hostile lanes the spec demanded:

- stable family/member identity across rematerialization and publication
- runtime-owned lifecycle truth across refresh, retry, timeout, supersession,
  invalidation, and host-driven revalidation
- declaration-driven narrow reconciliation versus broad fallback
- branch, exact restore, retained-history unavailability, and exact replay
  separation
- binary/download truth without collapsing structured value into byte transport
- delivery basis progression, stale-basis denial, duplicate delivery denial,
  and local-refresh convergence
- signals-first versus external-definition convergence on the same local line
  model
- explicit external basis refresh compatibility without a second delivery engine
- suite-0 canonical verification packages across forward, restore,
  retained-history, and replay modes

## Verification At Closeout

Most recent closeout verification:

```powershell
node --test crates/worth-signal-wasm/package/product/resource.runtime.test.mjs
npm run check:types
npm run check:boundary
```

The full runtime suite currently closes 120 resource-runtime tests with zero
failures, and the boundary/type smoke checks close the compile-time posture
obligations named by the certification spec.

## Deferred Follow-On Work

Closing this milestone does not mean the wasm roadmap is finished.

What remains intentionally deferred to later roadmap milestones:

- the forms product surface
- any mutation/optimistic-write milestone that integrates through explicit
  reconciliation hooks rather than ambient resource side effects
- any broader external-system product beyond the typed compatibility boundary
- additional download or delivery transport classes that would require a new
  compatibility taxonomy

## Residual Risk

No open milestone blocker remains at closeout.

The most sensitive future regression class is still semantic drift under new
external-definition contracts, delivery packet kinds, or mutation-side
integration. The current hostile certification bundle protects the shipped API
surface, but any future product that adds a new authority class or new packet
grammar should add a named suite row instead of relying only on the current
baseline matrix.
