# Worker Runtime Product Entrypoint Correction Closeout Acceptance Map

> **Status:** Closed
>
> **Spec:** [worker_runtime_product_entrypoint_correction_plan.md](./worker_runtime_product_entrypoint_correction_plan.md)
>
> **Roadmap parent:** [wasm_product_roadmap.md](./wasm_product_roadmap.md)
>
> **Substrate predecessor:** [worker_runtime_placement_closeout.md](./worker_runtime_placement_closeout.md)

## Purpose

This document is the closeout ledger for the worker-runtime product entrypoint
correction.

It answers the hostile package-level question the substrate closeout could not
answer by itself:

> Does the shipped `worth-signals-wasm` package now make `createSignals()` an
> honest worker-first front door, with explicit compatibility recovery,
> explicit worker-unavailable artifacts, aligned docs and types, and no hidden
> reintroduction of main-thread runtime authority?

## Current Closeout Read

The corrective implementation is materially complete.

The shipped package surface now includes:

- async `createSignals()` as the canonical worker-first entry lane
- explicit compatibility construction through
  `createSignals({ deployment: "mainThreadCompatibility" })`
- typed worker-unavailable construction rejection
- deployment planning and construction explanation surfaces
- worker-first callable app construction over the existing worker substrate
- worker-first forms, resources, API terminal realization, diagnostics,
  history, adapters, specialist, observation, and host-capability surfaces
- no compatibility-sidecar runtime inside the worker-first root
- README and package docs aligned to the async worker-first construction story
- canonical explicit compatibility recovery guidance
- named construction proof families at the package entry boundary

The final rebuilt aggregate verification is now captured. The remaining work is
only repository bookkeeping: commit/push and any parent-roadmap status updates
that reference this closed follow-on.

## Primary Implementation Surfaces

Public entrypoint planning and callable construction:

- [entrypoint_construction.ts](../../crates/worth-signal-wasm/package-src/product/entrypoint/construction/entrypoint_construction.ts)
- [worker_first_callable_signals.ts](../../crates/worth-signal-wasm/package-src/product/entrypoint/worker_first_callable_signals.ts)
- [worker_first_authoring_namespace.ts](../../crates/worth-signal-wasm/package-src/product/entrypoint/worker_first_authoring_namespace.ts)
- [worker_first_root_session.ts](../../crates/worth-signal-wasm/package-src/product/entrypoint/worker_first_root_session.ts)

Worker-root responsibility boundaries:

- [worker_first_root_history_lifecycle.ts](../../crates/worth-signal-wasm/package-src/product/entrypoint/worker_first_root_history_lifecycle.ts)
- [worker_first_root_mutation.ts](../../crates/worth-signal-wasm/package-src/product/entrypoint/worker_first_root_mutation.ts)
- [worker_first_root_runtime_replacement.ts](../../crates/worth-signal-wasm/package-src/product/entrypoint/worker_first_root_runtime_replacement.ts)
- [worker_first_root_cached_facades.ts](../../crates/worth-signal-wasm/package-src/product/entrypoint/worker_first_root_cached_facades.ts)
- [worker_first_root_graph.ts](../../crates/worth-signal-wasm/package-src/product/entrypoint/worker_first_root_graph.ts)
- [worker_first_root_history.ts](../../crates/worth-signal-wasm/package-src/product/entrypoint/worker_first_root_history.ts)
- [worker_first_root_observations.ts](../../crates/worth-signal-wasm/package-src/product/entrypoint/worker_first_root_observations.ts)

Worker-first authored runtime support:

- [worker_first_root_authored_runtime.ts](../../crates/worth-signal-wasm/package-src/product/entrypoint/sessions/support/authored/worker_first_root_authored_runtime.ts)
- [worker_first_authored_input_state.ts](../../crates/worth-signal-wasm/package-src/product/entrypoint/sessions/support/authored/worker_first_authored_input_state.ts)
- [worker_first_authored_readable_state.ts](../../crates/worth-signal-wasm/package-src/product/entrypoint/sessions/support/authored/worker_first_authored_readable_state.ts)
- [worker_first_authored_callback_authoring.ts](../../crates/worth-signal-wasm/package-src/product/entrypoint/sessions/support/authored/worker_first_authored_callback_authoring.ts)
- [worker_first_authored_readable_refresh.ts](../../crates/worth-signal-wasm/package-src/product/entrypoint/sessions/support/authored/worker_first_authored_readable_refresh.ts)
- [worker_first_host_dependency_records.ts](../../crates/worth-signal-wasm/package-src/product/entrypoint/sessions/support/authored/worker_first_host_dependency_records.ts)
- [worker_first_host_dependency_report.ts](../../crates/worth-signal-wasm/package-src/product/entrypoint/sessions/support/authored/worker_first_host_dependency_report.ts)

Compatibility and construction explanation surfaces:

- [signals.ts](../../crates/worth-signal-wasm/package-src/product/signals.ts)
- [construction_explanation_surface.test.mjs](../../crates/worth-signal-wasm/package/product/signals_runtime/entrypoint/construction/construction_explanation_surface.test.mjs)

Documentation surfaces:

- [README.md](../../crates/worth-signal-wasm/README.md)
- [install-and-publish.md](../../crates/worth-signal-wasm/docs/package/install-and-publish.md)
- [compatibility-surface.md](../../crates/worth-signal-wasm/docs/api-reference/compatibility-surface.md)

## Must-Ship Acceptance Map

| Spec requirement | Implementation evidence | Proof / doc evidence |
| --- | --- | --- |
| Async worker-first `createSignals()` contract | `entrypoint_construction.ts`, `worker_first_callable_signals.ts` | `async_worker_first_entrypoint_construction.test.mjs`, package type surface |
| Explicit compatibility deployment lane | `signals.ts`, deployment planning in `entrypoint_construction.ts` | `explicit_compatibility_construction.test.mjs`, README/install-guide recovery snippet |
| Typed worker-unavailable construction artifact | deployment planner + rejection surfaces | `worker_unavailable_construction_artifact.test.mjs` |
| No hidden main-thread fallback | explicit plan families only | `no_hidden_main_thread_fallback.test.mjs` |
| Worker-first callable surface over worker-owned truth | root session + worker-first facades + imported/root graph surfaces | worker-first callable construction suite under `package/product/signals_runtime/entrypoint/construction` |
| Worker-first semantic parity for supported workloads | worker-first root/history/resource/form/api surfaces | `explicit_compatibility_construction.test.mjs` plus worker-first construction proofs |
| Docs teach async worker-first as the normal lane | README + docs tree | `docs_and_package_surface_alignment.test.mjs` |
| Docs teach one canonical compatibility recovery path | README + install guide | `canonical_compatibility_recovery_documentation.test.mjs` |
| Advanced denial/unavailability inspection | construction explanation surface | `construction_explanation_surface.test.mjs` |
| No second main-thread runtime inside worker-first root | explicit denials of `compatibilityApp()` / `compatibilityRuntime()` on worker-first root; no hidden compatibility sidecar | worker-first construction and root-surface proofs, hostile QA record |

## Required Named Proof Families

The correction plan named these required proof families:

- `The Async Worker-First Entrypoint Construction Test`
- `The Explicit Compatibility Construction Test`
- `The Worker Unavailable Construction Artifact Test`
- `The No Hidden Main-Thread Fallback Test`
- `The Worker-First Entrypoint Semantic Parity Test`
- `The Docs And Package Surface Alignment Test`
- `The Construction Explanation Surface Test`
- `The Canonical Compatibility Recovery Documentation Test`

The current package proof surfaces are:

- [async_worker_first_entrypoint_construction.test.mjs](../../crates/worth-signal-wasm/package/product/signals_runtime/entrypoint/construction/async_worker_first_entrypoint_construction.test.mjs)
- [explicit_compatibility_construction.test.mjs](../../crates/worth-signal-wasm/package/product/signals_runtime/entrypoint/construction/explicit_compatibility_construction.test.mjs)
- [worker_unavailable_construction_artifact.test.mjs](../../crates/worth-signal-wasm/package/product/signals_runtime/entrypoint/construction/worker_unavailable_construction_artifact.test.mjs)
- [no_hidden_main_thread_fallback.test.mjs](../../crates/worth-signal-wasm/package/product/signals_runtime/entrypoint/construction/no_hidden_main_thread_fallback.test.mjs)
- [docs_and_package_surface_alignment.test.mjs](../../crates/worth-signal-wasm/package/product/signals_runtime/entrypoint/construction/docs_and_package_surface_alignment.test.mjs)
- [construction_explanation_surface.test.mjs](../../crates/worth-signal-wasm/package/product/signals_runtime/entrypoint/construction/construction_explanation_surface.test.mjs)
- [canonical_compatibility_recovery_documentation.test.mjs](../../crates/worth-signal-wasm/package/product/signals_runtime/entrypoint/construction/canonical_compatibility_recovery_documentation.test.mjs)

The semantic-parity obligation is discharged by the compatibility-vs-worker
construction and worker-first callable proof family rather than one file with
that exact name:

- [explicit_compatibility_construction.test.mjs](../../crates/worth-signal-wasm/package/product/signals_runtime/entrypoint/construction/explicit_compatibility_construction.test.mjs)
- [worker_first_callable_root_surfaces.test.mjs](../../crates/worth-signal-wasm/package/product/signals_runtime/entrypoint/construction/worker_first_callable_root_surfaces.test.mjs)
- [worker_first_callable_form.test.mjs](../../crates/worth-signal-wasm/package/product/signals_runtime/entrypoint/construction/worker_first_callable_form.test.mjs)
- [worker_first_callable_resource.test.mjs](../../crates/worth-signal-wasm/package/product/signals_runtime/entrypoint/construction/worker_first_callable_resource.test.mjs)
- [worker_first_callable_api.test.mjs](../../crates/worth-signal-wasm/package/product/signals_runtime/entrypoint/construction/worker_first_callable_api.test.mjs)

## Documentation Alignment Ledger

The package docs now explicitly teach:

- `await createSignals()` as the normal lane for ordinary app code
- explicit compatibility recovery through
  `await createSignals({ deployment: "mainThreadCompatibility" })`
- no assumption that worker-first roots expose `signals.compatibilityApp()` or
  `signals.compatibilityRuntime()`

The current alignment proof rejects:

- sync-looking `const signals = createSignals();`
- stale `createSignals().importGraph(...)`
- compatibility-surface docs that mention compatibility methods without also
  teaching explicit `mainThreadCompatibility`

## Final Verification Captured

The refreshed closeout verification record for the corrected worker-first
package surface is:

```sh
npm --prefix crates/worth-signal-wasm run typecheck:package-surface
git diff --check
/Users/spenstar/.nvm/versions/node/v22.13.1/bin/node --experimental-wasm-modules --test --test-force-exit \
  crates/worth-signal-wasm/package/product/signals_runtime/entrypoint/construction/*.test.mjs
wasm-pack build crates/worth-signal-wasm --target bundler --release --out-dir pkg
node scripts/wasm/prepare-worth-signals-wasm-package.mjs crates/worth-signal-wasm/pkg
/Users/spenstar/.nvm/versions/node/v22.13.1/bin/node --experimental-wasm-modules --test --test-force-exit \
  crates/worth-signal-wasm/package/product/signals.runtime.test.mjs
```

Captured result on 2026-05-21:

- `npm --prefix crates/worth-signal-wasm run typecheck:package-surface` passed
- `git diff --check` passed
- construction aggregate passed with `36/36`
- `wasm-pack build crates/worth-signal-wasm --target bundler --release --out-dir pkg` passed
- `node scripts/wasm/prepare-worth-signals-wasm-package.mjs crates/worth-signal-wasm/pkg` passed
- `signals.runtime.test.mjs` passed with `271/271`

The refreshed aggregate rerun also caught and closed two real post-closeout
drifts:

- scoped worker-first linked handles were dropping `relink()` during scope
  decoration
- forms closeout/doc tests were still resolving docs and metadata from the
  repository root instead of the crate-local docs tree

## Honest Residual Boundaries

The following remain intentionally explicit, not accidental unfinished work:

- worker-first roots do not expose `signals.compatibilityApp()` or
  `signals.compatibilityRuntime()`
- worker-first root graphs do not expose compatibility-sidecar graph runtime
  doors
- host-capability reads inside worker-first callback capture remain denied until
  a distinct explicit host-read lowering lane exists

Those boundaries are aligned with the placement spec and the correction planÃ¢â‚¬â„¢s
single-runtime-authority rule.
