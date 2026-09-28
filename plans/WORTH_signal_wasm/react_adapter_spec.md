# worth-signals-wasm React Adapter Spec

> **Status:** Completed 2026-04-29
>
> **Parent:** [web_runtime_spec.md](web_runtime_spec.md)
>
> **Core prerequisite:** [plans/worth_signal/milestone-11-closeout.md](../WORTH_signal/milestone-11-closeout.md)
>
> **Primary architectural driver:** add a React domain inside
> `worth-signals-wasm` that feels native in a React codebase without inventing a
> second store engine, weakening committed observation semantics, or smearing
> framework glue back into the framework-agnostic runtime

## Goal

Add a React domain to `worth-signals-wasm` that provides a clean, minimal,
production-grade adapter surface for React apps:

- `createSignals()` remains the runtime entrypoint
- React consumes that runtime through hooks
- hooks are powered by `useSyncExternalStore`
- diagnostics are available through `useSignalsDiagnostics()`

The point is not to make React the owner of signal truth.

The point is to make React a disciplined consumer of the already-finished web
runtime so a React codebase can use WORTH for state, derivation, observation,
and diagnostics without writing its own adapter glue.

## Why This Spec Exists

This spec was written while the web runtime was still converging and the React
lane still needed to be forced into a disciplined consumer shape.

At the time of writing, the adoption gap looked like this:

- the runtime is framework-agnostic
- React still wants a stable subscription/snapshot layer
- teams will otherwise re-implement `useSyncExternalStore` glue ad hoc
- once that happens, React glue starts becoming the real product instead of a
  consumer of the runtime product

This spec exists to prevent that drift.

## Hard Part

The hard part is not "write some hooks."

The hard part is preserving one honest layering:

- `worth-signal` owns derived computation and committed observation semantics
- the wasm web runtime owns web-facing authoring, handles, observation
  marshalling, diagnostics, and history
- the React domain owns only React lifecycle glue and snapshot subscription

The design fails if:

- React hooks redefine what a committed change means
- React keeps its own derived cache that can diverge from runtime truth
- hooks over-subscribe and broaden observation matching beyond signal scope
- diagnostics hooks read richer paths by accident on ordinary render hot paths
- framework glue leaks back into the framework-agnostic runtime domain

## Adversarial Constraint

This adapter must survive this hostile condition:

> A React screen with multiple components reading the same input, computed, and
> output signals, with interleaved transactions, watcher churn, effect churn,
> restore activity, and component mount/unmount churn, must converge to the
> same committed values and latest diagnostics summaries as a non-React host
> consuming the same `Signals` instance directly.

## Explicit Assumptions

- the React adapter lives inside `worth-signals-wasm` in a separate `react`
  domain folder
- the adapter is secondary to the framework-agnostic runtime, not a new top
  level product surface
- `useSyncExternalStore` is the required subscription primitive
- the React adapter must accept the existing app-first runtime handles
- `useSignalsDiagnostics()` is in scope for v1 of the React adapter
- React-specific helpers must not make Angular/Vue/etc. harder to add later

## Governing Summaries

- `MENTALITY.md`
  The important thing it protects here is not hiding the dangerous boundary.
  This spec therefore defines React as a consumer of runtime truth, not as a
  place where truth is recomputed "more ergonomically."
- `arch_laws.md`
  The dominant laws here are separation, phase honesty, and proof-bearing
  boundaries. React lifecycle state, snapshot state, runtime handles, and
  diagnostics snapshots must not collapse into one ambiguous object.
- `perf_laws.md`
  The important thing it protects is that React convenience must not widen the
  operational cost surface. Subscription matching must stay node-local and
  rerender triggers must stay snapshot-driven rather than "rerun everything."
- `domain_laws.md`
  The important thing it protects is that `src/react` must be a separate domain
  folder with its own responsibility. The runtime domain remains the runtime
  domain.

## Product Decision Lock

- the React adapter lives in a distinct in-crate `react` domain
- the runtime still starts with `createSignals()`
- React hooks consume an existing `Signals` instance
- the minimal v1 React hook surface is:
  - `createReactSignalsStore(signals)`
  - `useSignalValue(signal)`
  - `useOutputValue(output)`
  - `useSignalsDiagnostics()`
- `useSignalsDiagnostics()` is not deferred
- React hooks must consume committed observation semantics already provided by
  the runtime
- no React hook may silently create its own derived cache or selector engine
- React hook naming should stay direct and boring; the playful naming lives on
  the runtime side with `nuke`, not in hook names

Normative consequence:

- any adapter that does not use `useSyncExternalStore` is out of spec
- any adapter that computes freshness from local React state instead of runtime
  subscription/snapshot truth is out of spec
- any adapter that makes diagnostics a one-off imperative helper instead of a
  React-readable subscription surface is out of spec

## Scope

### In Scope

- a separate React domain folder, likely `src/react`
- a store adapter around a `Signals` instance
- stable subscription and snapshot wiring for React
- reading `InputSignal`, `ComputedSignal`, and `OutputSignal` values through
  hooks
- diagnostics subscription and snapshot wiring through `useSignalsDiagnostics()`
- tests proving mount/unmount churn and committed-boundary correctness
- React-facing documentation with one concrete usage example

### Explicitly Out Of Scope

- Suspense resource modeling
- server rendering guarantees
- async resource hooks
- form abstractions
- router integration
- custom concurrent scheduler work beyond standard React subscription semantics

## Public API Model

The intended React-domain surface is:

```ts
import { createSignals } from "worth-signals-wasm";
import {
  createReactSignalsStore,
  useOutputValue,
  useSignalValue,
  useSignalsDiagnostics,
} from "worth-signals-wasm/react";

const signals = createSignals();
const store = createReactSignalsStore(signals);

const count = signals.input(1, { id: "count" });
const doubled = signals.computed(() => count() * 2, { id: "doubled" });
const panel = signals.output(() => ({
  count: count(),
  doubled: doubled(),
}), { id: "panel" });

function Counter() {
  const countValue = useSignalValue(count, store);
  const doubledValue = useSignalValue(doubled, store);
  const panelValue = useOutputValue(panel, store);
  const diagnostics = useSignalsDiagnostics(store);

  return ...
}
```

The exact export path can still be refined during implementation, but the
semantic categories are locked:

- the store owns React subscription glue
- `useSignalValue` reads input or computed handles
- `useOutputValue` reads public projection handles
- `useSignalsDiagnostics` exposes latest observation/flow/performance-facing
  diagnostics snapshots

## Store Model

The React domain should revolve around one explicit adapter object rather than
ambient global state.

The minimal model is:

- `createReactSignalsStore(signals)`

That store must own:

- subscription registration/unregistration against the runtime
- stable snapshot versioning for React
- fanout for multiple components watching the same signal
- a diagnostics subscription lane for `useSignalsDiagnostics()`

The store must not own:

- derived signal computation
- transaction semantics
- effect semantics
- independent diagnostic truth

## Hook Semantics

### `useSignalValue`

`useSignalValue(signal, store)`:

- accepts `InputSignal` or `ComputedSignal`
- returns the current committed runtime value
- rerenders only when the subscribed signal meaningfully changes across a
  committed boundary

### `useOutputValue`

`useOutputValue(output, store)`:

- accepts `OutputSignal`
- returns the current committed public projection value
- is the preferred hook for structured view-model outputs

### `useSignalsDiagnostics`

`useSignalsDiagnostics(store)`:

- returns a stable diagnostics snapshot object
- must at minimum expose:
  - `latestObservation`
  - `latestFlow`
  - `performanceSummary`
- must stay coherent with direct runtime diagnostics reads
- must update after committed runtime changes that change diagnostics state

This hook exists now because diagnostics are part of the product truth model,
not a debugging afterthought.

## Subscription Contract

React subscriptions must be derived from runtime observation, not from polling
or React-local derivation.

The adapter must guarantee:

- subscriptions observe committed runtime boundaries only
- rollback does not create false rerenders
- unsubscribe on unmount is deterministic
- repeated components reading the same signal reuse one runtime subscription
  path where practical
- diagnostics subscription remains separate from ordinary signal-value
  subscriptions so it does not accidentally broaden hot render paths

## Architecture Corrections

This spec requires a distinct React responsibility space, for example:

- `src/react/store.rs`
- `src/react/hooks.rs`
- `src/react/model.rs`
- `src/react/tests.rs`

The exact filenames may differ, but the adapter must not be smuggled into the
main runtime boundary files.

The React domain may depend on the runtime domain.
The runtime domain must not depend on the React domain.

## Compile-Time And Lowering Discipline

The React adapter should preserve the same phase honesty as the runtime:

- React authoring intent
- lowered store subscription request
- committed snapshot token / version
- React-rendered snapshot value

At minimum, distinct types should remain distinct for:

- React store adapter
- signal subscription record
- diagnostics subscription record
- signal snapshot
- diagnostics snapshot

## Performance Contracts

The React adapter must make performance visible, not hopeful.

### Required Boundedness

- component subscription must not widen to whole-runtime watch sets
- one component reading one signal must not cause broad diagnostics refresh
- multiple components reading the same signal should not multiply runtime
  subscriptions linearly when store-level fanout can avoid it
- diagnostics rerender triggers must be snapshot-based and bounded

### Required Measurement

The React domain must expose or certify:

- active React subscriber count
- runtime watch handle count attributable to the React store
- fanout ratio between React subscribers and runtime watchers
- diagnostics subscriber count

## Required Named Test Families

This adapter is not done until it has at least these test families:

- `The React And Runtime Observation Equivalence Test`
  Proves that React hook rerenders line up with committed runtime boundaries.
- `The React Mount Churn And Nuke Equivalence Test`
  Proves that mount/unmount churn tears down subscriptions honestly and does not
  resurrect stale listeners.
- `The React Shared Subscription Fanout Test`
  Proves that multiple components reading the same signal do not broaden into
  redundant runtime watch registrations.
- `The React Diagnostics Parity Test`
  Proves that `useSignalsDiagnostics()` stays coherent with direct runtime
  diagnostics reads.
- `The React Rollback Suppression Test`
  Proves that failed transactions do not produce illegal rerenders.

## Acceptance Surface

This spec is done when all of these are true:

- a React app can consume a `Signals` instance through a dedicated React store
- `useSignalValue`, `useOutputValue`, and `useSignalsDiagnostics` are present
  and coherent
- React rerenders follow committed runtime observation semantics
- diagnostics are available through a first-class hook now, not deferred
- React-domain structure remains separate from the framework-agnostic runtime
- required adversarial tests pass

## Completion Note

This adapter spec is complete. React now consumes an existing `Signals`
instance through a dedicated store and runtime-owned diagnostics subscription
path instead of monkey-patching shared runtime behavior or inventing a second
derived cache.

The React surface remains intentionally secondary to the framework-agnostic web
runtime and the callback-first product surface described in:

- [web_runtime_spec.md](./web_runtime_spec.md)
- [host_callback_computed_spec.md](./host_callback_computed_spec.md)

## Explicit Non-Goals

This spec does not attempt to solve:

- async resources
- forms
- suspense
- SSR
- Angular/Vue/etc. adapters

Its job is narrower:
make React a clean, honest consumer of the already-built web runtime.
