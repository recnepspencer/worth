# Host Capability Closeout Acceptance Map

> **Status:** Completed
>
> **Spec:** [host_capability_spec.md](./host_capability_spec.md)
>
> **Roadmap parent:** [wasm_product_roadmap.md](./wasm_product_roadmap.md)
>
> **Product guide:** [host_capabilities.md](./host_capabilities.md)
>
> **Prerequisite closeout:** [host_callback_computed_spec.md](./host_callback_computed_spec.md)

## Purpose

This document maps the host-capability milestone to concrete implementation and
certification evidence.

It is the closeout ledger for the hostile question:

> Can `worth-signals-wasm` now admit browser- and runtime-local facts through a
> typed product lane without letting ambient closure reads, React lifecycle,
> or portable transport shortcuts become a second reactive truth engine?

## Closeout Summary

Milestone 1 is implemented as a typed host-capability lane on top of the
callback-first wasm surface.

The implementation now includes:

- one explicit `hostCapabilityPlan(...)` registration boundary
- admitted product families for:
  - `visibility`
  - `viewport`
  - `online`
  - `clock`
  - `persistence`
- framework-owned capability lifecycle and invalidation ownership
- typed `signals.host.*` handles rather than ambient host reads
- capability read witness capture through callback execution artifacts
- explicit compatibility posture:
  - `LiveOnly`
  - `Reattachable`
  - `SnapshotPortable`
  - `ImportDenied`
- same-runtime exact restore kept distinct from portable import
- diagnostics-visible host events, lineage, breadth, and canonical digests
- package-facing transport reports for unavailable callback-bearing artifacts
- hostile certification suites covering ambient reads, lifecycle churn,
  mixed-family invalidation, transport honesty, React parity, and long-session
  diagnostics retention

The direct closeout gates are:

- [host_capabilities.certification.test.mjs](../package-src/product/host_capabilities.certification.test.mjs)
- [verify-worth-signals-wasm-package.mjs](../../../scripts/wasm/verify-worth-signals-wasm-package.mjs)

Those two surfaces matter together:

- the certification harness proves the hostile milestone grammar directly
- the package verifier proves the prepared npm artifact still exposes the same
  honest surface to a clean consumer

## Primary Implementation Surfaces

Product surface and lifecycle ownership:

- [package-src/product/host_capabilities.ts](../package-src/product/host_capabilities.ts)
- [package-src/product/signals.ts](../package-src/product/signals.ts)
- [package-src/product/callback_frames.ts](../package-src/product/callback_frames.ts)
- [package-src/product/transactions.ts](../package-src/product/transactions.ts)
- [package-src/product/diagnostics.ts](../package-src/product/diagnostics.ts)
- [package-src/product/host_capability_reports.ts](../package-src/product/host_capability_reports.ts)

Typed public surface:

- [package-src/index.ts](../package-src/index.ts)
- [package/types/callable_surface.d.ts](../package/types/callable_surface.d.ts)
- [package/types/diagnostics.d.ts](../package/types/diagnostics.d.ts)
- [package/types/raw_surface.d.ts](../package/types/raw_surface.d.ts)
- [package/types-smoke.ts](../package/types-smoke.ts)
Runtime and transport ownership:

- [src/runtime/compute_callbacks/invocation.rs](../src/runtime/compute_callbacks/invocation.rs)
- [src/runtime/compute_callbacks/types.rs](../src/runtime/compute_callbacks/types.rs)
- [src/runtime/core/state.rs](../src/runtime/core/state.rs)
- [src/runtime/core/diagnostics/callback_nodes.rs](../src/runtime/core/diagnostics/callback_nodes.rs)
- [src/runtime/core/diagnostics/why.rs](../src/runtime/core/diagnostics/why.rs)
- [src/runtime/core/envelopes.rs](../src/runtime/core/envelopes.rs)
- [src/runtime/adapters.rs](../src/runtime/adapters.rs)
- [src/runtime/summaries.rs](../src/runtime/summaries.rs)
- [src/boundary/restore_tokens.rs](../src/boundary/restore_tokens.rs)

## Must-Ship Acceptance Map

| Spec requirement | Implementation evidence | Certification evidence |
| --- | --- | --- |
| Sealed registration plan and non-WORTHable family entries | `hostCapabilityPlan(...)`, branded family constructors, framework-owned hidden host sources | runtime tests deny WORTHd plan entries and structural lookalikes |
| Typed callback-visible host reads | `signals.host.*`, callback capture frames, host-capability read artifacts | runtime and certification suites prove host reads appear explicitly while ambient reads do not |
| Family-local invalidation and lifecycle ownership | push-driven, polled, and manually committed family registries plus runtime-owned teardown | stale/zombie delivery tests and mixed-family churn certification |
| Honest restore/import/export posture | exact restore tokens, portable import denial/unavailability artifacts, transport reports | signals runtime tests plus package verifier |
| Diagnostics-visible host causality | `latestHostCapabilityEvent()`, `recentHostCapabilityEvents()`, `hostCapabilityReport()` | diagnostics assertions in runtime tests and hostile certification harness |
| Named public counters and boundedness | performance summary host counters, breadth/lineage digests, unavailability accounting | package verifier plus dedicated hostile breadth assertions |
| React remains a consumer | React store reads through runtime-owned `signals.read(...)` and diagnostics subscription | mount churn certification and runtime React store tests |

## Required Acceptance Evidence

The spec named five minimum evidence lanes. They now map to:

| Spec evidence lane | Owning suite(s) |
| --- | --- |
| `The Host Capability Purity Boundary Test` | `host capability certification keeps ambient reads non-reactive and bounds invalidation to the affected frontier` |
| `The Host Capability Invalidation Scope Test` | `host capability invalidation batches push churn and exposes counters honestly`; certification frontier assertions |
| `The Host Capability Restore And Replay Honesty Test` | `wrapSignals adapters wrapper marks same-runtime exact restore while preserving portable host-capability denial artifacts`; package verifier |
| `The Host Capability Product Boundary Typing Test` | `types-smoke.ts`; package verifier clean-consumer compile surface |
| `The Host Capability React Consumer Parity Test` | `host capability certification rejects zombie delivery and keeps React as a pure consumer under mount churn`; `react/store.runtime.test.mjs` |

## Adversarial Closeout Matrix Coverage

| Matrix row | Owning evidence |
| --- | --- |
| Ambient read rejection torture | `host capability certification keeps ambient reads non-reactive and bounds invalidation to the affected frontier` |
| Stale registration and zombie delivery | `host capability certification rejects zombie delivery and keeps React as a pure consumer under mount churn` |
| Fanout boundedness and frontier precision | certification breadth assertions plus `hostCapabilityReport().breadth` |
| Restore / replay / reattach honesty nightmare | `host capability certification preserves transport honesty, mixed-family attribution, and long-session report integrity`; package verifier |
| Host capability identity WORTHry | runtime denial tests for WORTHd plan entries and non-package handles |
| React consumer parity and mount churn | certification mount churn test plus `react/store.runtime.test.mjs` |
| Multi-family mixed churn | long-session mixed-family certification and package verifier mixed-family export/import checks |
| Long-session retention and diagnostics integrity | certification bounded-lineage assertions plus digest stability checks |

## Verification At Closeout

Most recent closeout verification:

```powershell
node crates/worth-signal-wasm/package-src/product/host_capabilities.certification.test.mjs
node crates/worth-signal-wasm/package-src/product/host_capabilities.runtime.test.mjs
node crates/worth-signal-wasm/package-src/product/signals.runtime.test.mjs
node crates/worth-signal-wasm/react/store.runtime.test.mjs
powershell -ExecutionPolicy Bypass -File scripts/wasm/publish-worth-signals-wasm.ps1 -SkipPublish
```

## Deferred Follow-On Work

Closing this milestone does not mean the wasm roadmap is finished.

What remains intentionally deferred to later roadmap milestones:

- forms built on top of host capability and async truth
- API surface product line
- additional host families beyond the admitted first set
- any future broadening of transport posture that would need a new capability
  taxonomy or new family-specific restore law

## Residual Risk

No open milestone blocker remains at closeout.

The most sensitive future regression class is still transport and lifecycle
drift under new capability families. The current hostile suite protects the
first shipped family set, but any future family that introduces a new
invalidity grammar, transport class, or registration shape should add its own
certification row instead of relying only on the baseline matrix.
