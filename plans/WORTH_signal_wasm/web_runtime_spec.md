# worth-signals-wasm Web Runtime Spec

> **Status:** Completed 2026-04-29
>
> **Vision parent:** [plans/worth_signal/worth_signals2.md](../WORTH_signal/WORTH_signals2.md)
>
> **Core prerequisite:** [plans/worth_signal/milestone-11-closeout.md](../WORTH_signal/milestone-11-closeout.md)
>
> **Primary architectural driver:** ship a framework-agnostic web runtime that
> feels native in React, Angular, Vue, workers, and plain TypeScript without
> inventing a second reactive truth model or preserving the current collapsed
> wasm facade shape

## Goal

Make `worth-signals-wasm` a framework-agnostic web runtime product with an
app-first API:

- `createSignals()`
- `input`
- `computed`
- `output`
- `watch`
- `effect`
- `transaction`
- `nuke`

while also cleaning the crate into a law-compliant architecture whose public
surface, internal decomposition, typing, observation semantics, diagnostics,
and compatibility story are honest enough to be imported directly into a real
web codebase for state and subscription management.

## Why This Spec Exists

This spec is not "add a nicer wrapper to the existing wasm bindings."

This spec was written before the app-first runtime and callback-computed
closeout were finished. Its main architectural decisions have now landed, and
the current wasm crate is the implemented web runtime product this document was
trying to force into existence.

At the time of writing, the risk looked like this:

- the public surface is still kernel-first instead of app-first
- the main boundary is too collapsed, especially
  [facade.rs](../../crates/worth-signal-wasm/src/boundary/facade.rs)
- observation parity with the newly completed `worth-signal` substrate is
  incomplete
- `computed`, `effect`, and `output` are not first-class web concepts yet
- TypeScript ergonomics are not strong enough
- the package still feels like a low-level wasm export rather than something a
  web engineer would naturally import and build on

That means the crate is currently easy to underuse and easy to misuse.

If this spec is weak, the likely failure mode is not dramatic breakage.
It is slow semantic drift:

- apps build their own local subscription layer because wasm does not feel
  complete
- apps invent their own `computed` or `output` semantics because the package
  does not provide them
- `worth-signals-wasm` becomes a partial kernel wrapper plus a growing pile of
  helpers
- the collapsed facade becomes the new structural gravity well
- framework-specific glue quietly becomes the real product instead of the web
  runtime itself

This spec exists to prevent that.

## Hard Part

The hard part is not exposing more functions to JavaScript.

The hard part is freezing one honest relationship among four distinct things
that naive designs blur together:

- the core `worth-signal` runtime that owns derived computation and committed
  observation semantics
- the wasm binding layer that must marshal that runtime honestly into the web
  environment
- the app-facing web API that must feel natural in a TypeScript codebase
- the host callback and framework integration story that must not become a
  second reactive engine

The design fails if:

- the wasm crate defines app-facing semantics that the core runtime does not own
- `computed`, `output`, `watch`, or `effect` become a JS-local truth system
  disconnected from runtime observation and transaction boundaries
- cleanup is deferred and the new API is bolted onto the current collapsed
  facade
- `output` is treated as a naming alias instead of an actual public projection
  concept
- TypeScript types are weak enough that semantic categories collapse back into
  `any`
- framework authors cannot build stable adapters because subscription and
  snapshot semantics are still implicit

This spec therefore has to make the wasm package friendlier while making its
architecture stricter.

## Adversarial Constraint

This web runtime must survive this hostile condition:

> A web app with long-lived inputs, derived outputs, watcher churn, effect
> churn, rollback-producing transaction failures, branch and restore activity,
> and rich structured outputs must converge to the same committed values,
> observation boundaries, and diagnostics summaries whether it uses the new
> app-first web API, the lower-level compatibility API, or a control lane built
> directly on `worth-signal` itself.

## Explicit Assumptions

- `worth-signal` remains the owner of derived computation, transactions,
  rollback semantics, and committed observation semantics.
- `worth-signals-wasm` remains a framework-agnostic web runtime, not a
  React-only package.
- framework-specific adapters may come later, and they may live in separate
  in-crate domain folders so long as they do not redefine core semantics or
  collapse back into the framework-agnostic web runtime boundary.
- `output` belongs in v1 and is not deferred.
- aspect semantics must remain first-class on the web surface for node
  definition, dependency reads, invalidation, recomputation, and version
  reporting.
- the existing `source`, `recipe`, `source_family`, and `recipe_family`
  surfaces still matter for compatibility and advanced use, but they are not
  the primary product story.
- this spec may add a thin JS-facing authoring/product layer, but it must not
  create a second semantic engine that disagrees with core runtime truth.
- package ergonomics matter as much as raw wasm export parity; the crate is
  only successful if a web engineer can adopt it without first becoming an
  expert in internal WORTH runtime topology.

## Governing Summaries

- `MENTALITY.md`
  The important thing it protects here is solving the dangerous boundary first.
  This spec therefore starts from "what must the web runtime mean?" and "how do
  we avoid a second reactive engine?" rather than from "what JS API looks coolest?"
- `arch_laws.md`
  The dominant laws here are 1, 10, 11, 18, 30, 34, 35, 40, and especially 41.
  The wasm crate must not remain a monolithic facade, observation callbacks
  must remain phase-typed and read-only, and web-facing registration,
  lowering, delivery, and diagnostics forms must stay distinct proof-bearing
  types rather than one open packet bag.
- `perf_laws.md`
  The important thing it protects is that the web surface must not hide broad
  scans, repeated rediscovery, excessive JS/Wasm boundary churn, or rich-path
  allocation on operational paths. Registration matching, output recomputation,
  watch delivery, and serialization need named counters and bounded slopes.
- `domain_laws.md`
  The important thing it protects is that the crate cleanup must organize by
  responsibility, not by "all wasm stuff goes here." Web app API, compatibility
  surface, observation, diagnostics, history, export, and boundary helpers must
  live in separate responsibility spaces because they change for different
  reasons and fail independently.
- `worth_signals2.md`
  The important thing it protects is the split between truth runtime,
  derived-computation runtime, and integration layers. This spec must extend
  that split to the web package rather than smearing app ergonomics directly
  into the core crate.
- `milestone-11-closeout.md`
  The important thing it protects is that core observation semantics are now
  finished enough to serve as the single substrate. This spec must consume
  that substrate, not reinterpret it.

## Product Decision Lock

- the primary web entrypoint is `createSignals()`
- the package must not require a separate user-called wasm bootstrap function
  before `createSignals()`; initialization burden belongs to the package
- the primary app-facing concepts are:
  - `input`
  - `computed`
  - `output`
  - `watch`
  - `effect`
  - `transaction`
  - `nuke`
- `output` is a first-class public projection concept in v1, not a future idea
- `nuke(handle)` is allowed as the public web teardown verb, but the internal
  runtime and Rust-facing terminology remain precise (`unsubscribe`,
  `unobserve`, `dispose`) so internal semantics stay honest
- `transaction(...)` is the canonical committed-boundary primitive
- `batch(...)` may exist only as an exact ergonomic alias of `transaction(...)`;
  it is not allowed to become a weaker semantic lane
- the app-first surface is the canonical web product surface
- the existing kernel-like surface remains available only as an explicit
  compatibility/advanced surface
- the current exported names (`SignalApp`, `SignalRuntime`, `SignalDiagnostics`,
  `SignalHistory`, `SignalSpecialist`, `SignalAdapters`) remain compatibility
  surfaces during transition and must not be silently broken while the
  app-first surface is introduced
- framework-specific bindings are out of scope for this spec; the wasm package
  itself must be the framework-agnostic substrate they would sit on
- diagnostics, latest observation, latest flow, history, and branch-aware truth
  remain part of the product contract
- subscriptions stay node-scoped by default; aspect precision belongs in
  derivation and invalidation rather than the default observation surface

Normative consequence:

- any implementation that makes web `watch(...)` or `effect(...)` behave
  differently from core committed observation semantics is out of spec
- any implementation that turns `output` into a mere naming alias for
  `computed` without public-projection semantics is out of spec
- any implementation that preserves the current collapsed facade shape and only
  adds more exports is out of spec

## Scope

### In Scope

- app-first web runtime creation through `createSignals()`
- first-class web concepts for `input`, `computed`, `output`, `watch`,
  `effect`, `transaction`, and `nuke`
- explicit handle types and teardown semantics for watcher/effect lifecycles
- output registration and observation semantics suitable for structured view
  models, not just scalar values
- latest observation and latest flow diagnostics on the web surface
- history, branching, snapshot, export, and import surfaces that remain honest
  under the new app-facing API
- strong TypeScript typing for public app/runtime/observation/diagnostics
  surfaces
- crate cleanup and boundary decomposition required to satisfy architecture and
  domain laws
- compatibility/advanced surfaces for existing recipe/family-driven usage
- performance counters and named budgets for web registration, delivery, read,
  and serialization boundaries
- web-facing documentation and examples sufficient for framework-agnostic use
- package README updates aligned with the app-first surface

### Explicitly Out Of Scope

- React-specific hooks or adapter implementation in this spec
- Angular-specific service or signal wrapper implementation in this spec
- form, resource, workflow, permission, or async-resource product categories
- replacing the core runtime observation model with a JS-local store engine
- public API promises around SSR, service-worker persistence, or cross-tab sync
  beyond what the wasm package already honestly supports

## Public API Model

### Primary App Surface

The primary product surface is:

```ts
const signals = createSignals();

const count = signals.input(1, { id: "count" });
const doubled = signals.computed(() => count() * 2, { id: "doubled" });
const tableTrace = signals.output(() => ({
  count: count(),
  doubled: doubled(),
}), { id: "tableTrace" });

const watchHandle = signals.watch(tableTrace, (notice) => { ... });
const effectHandle = signals.effect(tableTrace, () => { ... });

signals.transaction((tx) => {
  tx.set(count, 2);
});

const latestObservation = signals.diagnostics().latestObservation();
const latestFlow = signals.diagnostics().latestFlow();

signals.nuke(watchHandle);
signals.nuke(effectHandle);
```

The exact callback and builder shape can still be refined during implementation,
but the semantic categories are locked:

- `input` is mutable app-owned source state
- `computed` is derived internal state
- `output` is a public projection intended for external consumption
- `watch` is explicit observation of committed change
- `effect` is host-side reaction to committed change
- `transaction` is the only committed write boundary
- `nuke` tears down managed observer/effect resources for future deliveries
- `watch` and `effect` accept either a signal handle or a string signal id

### Output Semantics

`output` is not just "computed with a different name."

`output` exists because web apps need a public projection concept for
structured, externally consumed values such as:

- table row view models
- trace payloads
- editor panel state
- summary objects
- complex nested derived UI state

An `output` must therefore have:

- stable public identity
- committed observation semantics
- diagnostics visibility as a public projection surface
- export/read friendliness for web consumers
- distinct typing from `computed` even if both lower into derived runtime work

### Observation Semantics

The web surface inherits the core Milestone 11 observation contract:

- one committed transaction yields at most one delivery boundary per matching
  watcher/effect handle
- rollback suppresses normal delivery
- `nuke(handle)` affects future deliveries only
- latest observation and latest flow remain inspectable after delivery
- branch, snapshot, restore, and merge semantics must remain consistent with
  core runtime behavior

Aspect-aware observation is not the default contract.
Instead:

- node definitions can declare produced aspects
- reads can subscribe to selected aspects
- writes and invalidation can target specific aspects
- node-level watchers/effects become more precise automatically because
  irrelevant aspect churn no longer forces downstream recompute

### Compatibility Surface

The existing kernel-style wasm surface remains in scope, but only as an
explicit compatibility/advanced surface.

The current exports:

- `source`
- `recipe`
- `source_family`
- `recipe_family`
- low-level read helpers
- diagnostics/history/specialist/adapters

must be retained only through a clearly named compatibility boundary.
They must not continue to dominate the crate's top-level product story.

### Transition Rule

The crate already exports kernel-first wasm-facing names such as:

- `SignalApp`
- `SignalRuntime`
- `SignalDiagnostics`
- `SignalHistory`
- `SignalSpecialist`
- `SignalAdapters`

This spec does not allow an unplanned breaking cutover.

The transition must therefore be explicit:

- `createSignals()` becomes the canonical entrypoint
- existing exports move under a clearly described compatibility story
- package docs must teach the app-first surface first and the compatibility
  surface second
- any eventual removal of compatibility exports belongs to a later,
  separately-admitted transition plan

## Architecture Corrections

### Current Structural Problem

The current wasm crate has some useful foldering, but the product boundary is
still too collapsed. The clearest example is
[facade.rs](../../crates/worth-signal-wasm/src/boundary/facade.rs),
which currently mixes too many reasons to change:

- app/runtime creation
- low-level definition registration
- reads
- transactions
- diagnostics
- history
- keyed/grid helpers
- export/import
- compatibility surfaces

That violates `domain_laws.md` and `arch_laws.md` directly.

### Required Decomposition

This spec requires the crate to reorganize around real responsibilities.

At minimum, the structure must separate:

- web app facade
- web observation and handle lifecycle
- web diagnostics surface
- web history and branch surface
- web export/import surface
- compatibility/advanced kernel surface
- low-level wasm boundary helpers (`errors`, `serde`, callback marshalling)
- package-facing documentation surfaces

The exact filenames can vary, but the crate must teach the domain by structure.
One giant facade file is not acceptable.

Framework-specific domains are allowed inside this crate only if they remain
structurally separate responsibility spaces, for example:

- `src/react`
- `src/angular`

Those folders must consume the framework-agnostic runtime surface rather than
mutating its semantics.

### Facade Rule

The crate still exposes one public package facade.
Internal modules must remain private behind that facade.

This means:

- `lib.rs` and the top-level exported JS surface stay small and intentional
- internal cleanup is not optional or decorative
- no external consumer should depend on deep internal module shape

### Package Documentation Rule

The package-level documentation is part of the product surface.

This spec therefore requires:

- a README that teaches `createSignals()` first
- examples that show `input`, `computed`, `output`, `watch`, `effect`,
  `transaction`, and `nuke`
- compatibility guidance that explains when a consumer would still use the
  lower-level exported runtime/kernel surfaces

Package docs are not allowed to remain a one-line "wasm bindings" description
once the crate is expected to be directly adopted in web codebases.

## Compile-Time And Lowering Discipline

This spec must not flatten web authoring, compatibility specs, runtime
registration, and committed delivery into one dynamically shaped bag.

The minimum proof-bearing phase split is:

- web authoring intent
- frozen web definition descriptor
- lowered runtime registration plan
- committed runtime definition / handle
- committed observation delivery summary

The main categories that need distinct types are:

- `InputHandle`
- `ComputedHandle`
- `OutputHandle`
- `WatchHandle`
- `EffectHandle`
- `DisposableHandle` or equivalent wrapper if `nuke(...)` accepts multiple
  handle families

Distinct meanings must remain distinct types even if they share internal
storage mechanics.

### Callback Capability Rule

Host callbacks must receive read-only committed views.

If the implementation exposes a host callback context, it must be a phase-typed
read capability, not a mutation-capable runtime bag.

Any follow-up mutation must happen through a new explicit `transaction(...)`
boundary.

## Performance Contracts

This spec must encode performance into the architecture, not as a later
optimization pass.

### Named Hot Paths

The following web runtime paths require named complexity contracts and counters:

- input writes
- committed `transaction(...)` delivery
- `watch(...)` registration matching
- `effect(...)` registration matching
- `output` recomputation and publication
- JS/Wasm serialization for reads and outputs
- compatibility surface registration and reads

### Required Boundedness

- watcher/effect matching must remain index-driven and bounded by changed
  semantic scope, not by active watcher count
- `output` publication must not force broad graph reads or whole-runtime export
- diagnostics access must not inject rich-path work into operational
  transactions
- compatibility surfaces must not broaden app-surface hot paths merely because
  they share a crate

### Measurement Boundaries

The wasm/web product surface must expose counters or internal cert surfaces for:

- active handle counts
- matched watcher breadth
- delivered watcher/effect count per transaction
- rollback-suppressed delivery count
- output serialization breadth
- JS callback invocation count
- compatibility read breadth where relevant

## Boundary Contract With Other WORTH Layers

`worth-signal`
- owns derived computation, transactions, invalidation, observation semantics,
  rollback semantics, and diagnostics truth

`worth-signals-wasm`
- owns web-facing authoring, host callback marshalling, typed handles,
  framework-agnostic web API shape, and app/public projection concepts such as
  `output`

future framework adapters
- own hooks, directives, or framework lifecycle glue

This spec must keep those lines clean.

The wasm package must not:

- redefine core runtime observation semantics
- become a React-specific convenience layer
- become a second source of truth for derived state

## Required Named Test Families

This spec is not closed until the wasm package has certification coverage for
at least these families:

- `The App-Surface And Core Observation Equivalence Test`
  Proves that `watch(...)` and `effect(...)` on the web surface converge to the
  same committed boundaries as the core runtime substrate.
- `The Output Projection Commitment Test`
  Proves that `output` behaves as a public committed projection rather than a
  local alias and remains consistent under structured object values.
- `The Nuke Lifecycle And Stale Handle Churn Test`
  Proves that torn-down web handles are not resurrected by branch churn,
  restore, or slot reuse.
- `The Host Callback Failure Rollback Test`
  Proves that callback failures do not create partial committed truth or
  illegal delivery behavior.
- `The Compatibility Surface Equivalence Test`
  Proves that the advanced/legacy wasm surface and the app-first surface agree
  on committed runtime truth where they overlap.
- `The Web Boundary Boundedness Test`
  Proves that operational paths stay within their declared matching and
  serialization breadth contracts.
- `The Diagnostics Observation Parity Test`
  Proves that latest observation and latest flow stay in sync on the web
  surface.

## Phases

### Phase 1: Boundary Cleanup And Product Skeleton

- split the current collapsed wasm facade into responsibility-shaped modules
- establish the one public package facade and explicit internal boundaries
- define the typed handle families and proof-bearing registration/lowering forms
- create the product-level `createSignals()` entry boundary
- move the legacy/kernel-style exports behind an explicit compatibility surface

This phase is mandatory first work because the feature surface should not be
added to the current collapsed facade shape.

### Phase 2: App-Facing Definitions

- add first-class `input`, `computed`, and `output`
- lock `output` as a distinct public projection concept
- add transaction-owned mutation APIs suitable for web state management
- define the read/export shape for app-facing values and outputs
- add TypeScript types for app primitives and handles

This phase is where the app-first web runtime becomes real.

### Phase 3: Observation, Watch, Effect, And Nuke

- expose committed observation through `watch(...)`
- expose host reaction through `effect(...)`
- implement `nuke(handle)` as the public teardown surface
- ensure all delivery semantics inherit the core observation contract
- expose latest observation and related diagnostics on the web surface

This phase closes the main parity gap between the core runtime and the web API.

### Phase 4: Diagnostics, History, Compatibility, And Package Completion

- expose diagnostics/history/branch/snapshot/export/import through the cleaned
  facade
- keep the advanced compatibility surface usable and explicitly secondary
- finish TS declaration quality
- finish packaging, initialization, and documentation for framework-agnostic
  adoption
- close the required named test families and performance contracts

## Acceptance Surface

This spec is done when all of these are true:

- a web engineer can import the package and start from `createSignals()`
- `input`, `computed`, `output`, `watch`, `effect`, `transaction`, and `nuke`
  are real first-class product concepts
- the current collapsed facade is decomposed into law-compliant responsibility
  spaces
- `output` is a real public projection concept, not an alias
- the app-first surface and compatibility surface agree on committed truth
- the web API inherits core observation semantics honestly
- latest observation and latest flow are visible and coherent on the web surface
- TS types are strong enough that handles and semantic categories do not
  collapse back into `any`
- required adversarial tests and performance certs pass

## Completion Note

This parent spec is complete. The app-first runtime, typed product/package
surface, observation parity, diagnostics/history lanes, and compatibility
truth all now exist in the crate.

The follow-on callback-first derived-state milestone that this spec pointed to
is also complete:
[host_callback_computed_spec.md](./host_callback_computed_spec.md).

Future wasm product work should now enter through
[wasm_product_roadmap.md](./wasm_product_roadmap.md), not by reopening the web
runtime foundation as if it were still unfinished.

## Follow-On Milestone: Host Callback Computed Nodes

The app-first runtime spec intentionally made `computed` a first-class product
concept, but the current concrete surface still uses serialized expression
recipes as the default authoring form.

That is not the final product shape.

[host_callback_computed_spec.md](./host_callback_computed_spec.md) is the
follow-on milestone that makes callback-backed computed nodes the normal
TypeScript authoring path:

```ts
const doubleCount: Signal<number> = computed(() => count() * 2);
```

That milestone belongs after this web runtime spec and the React adapter spec
because it depends on:

- app-first signal handles
- committed observation
- diagnostics/latest-flow surfaces
- React as a runtime-truth consumer rather than a second store engine

It must land before treating the wasm package as polished React state
infrastructure. Until callback computed nodes support dynamic dependencies and
diagnostics parity, the package still asks ordinary app code to author internal
expression recipes for basic derived state.

## Explicit Non-Goals

To keep this spec honest, it does not attempt to solve:

- React hooks
- Angular services or directives
- forms
- resources
- workflow abstractions
- permissions
- async job orchestration
- framework-specific suspense or rendering contracts

Those can come later.
This spec's job is to make the framework-agnostic web runtime strong enough
that those later layers are consumers, not substitute foundations.
