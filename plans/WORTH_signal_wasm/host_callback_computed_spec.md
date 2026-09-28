# Host Callback Computed Nodes Spec

> **Status:** Completed 2026-04-29
>
> **Parent:** [web_runtime_spec.md](./web_runtime_spec.md)
>
> **React parent:** [react_adapter_spec.md](./react_adapter_spec.md)
>
> **Core vision:** [plans/worth_signal/worth_signal_vision.md](../WORTH_signal/WORTH_signal_vision.md)
>
> **Core test requirements:** [plans/worth_signal/test-requirements.md](../WORTH_signal/test-requirements.md)
>
> **Wasm follow-on roadmap:** [wasm_product_roadmap.md](./wasm_product_roadmap.md)
>
> **Primary architectural driver:** make JavaScript callback-backed computed
> signals first-class runtime nodes with dynamic dependency capture and
> diagnostics parity, so normal TypeScript code can write
> `computed(() => count() * 2)` without creating a second reactive engine.

## Goal

Make host-computed signals a core WORTH Signal capability and make the primary
`worth-signals-wasm` computed authoring experience callback first:

```ts
const count = signal(1);
const doubleCount: Signal<number> = computed(() => count() * 2);
```

The callback surface must:

- accept ordinary JavaScript and TypeScript closures
- track signal reads automatically
- support dynamic dependency changes between evaluations
- lower into real WORTH runtime nodes
- preserve committed observation, rollback, history, branch, and diagnostics
  behavior
- keep the existing AST/spec recipe surface available as a portable advanced
  lane rather than the default application authoring path

This milestone is not complete if it only hides AST recipes behind a prettier
TypeScript wrapper.

This milestone is also not complete if the core host-computed lifecycle lives
only in `worth-signals-wasm`. The JavaScript callback is host-specific; the
host-computed node contract is core runtime infrastructure.

## Why This Spec Exists

The current app-first wasm surface has the right product nouns:

- `input`
- `computed`
- `output`
- `watch`
- `effect`
- `transaction`

But `computed` still asks normal web code to write a serialized recipe:

```ts
signals.computed("doubled", {
  reads: ["count"],
  expr: {
    kind: "multiply",
    args: [
      { kind: "read", id: "count" },
      { kind: "value", value: 2 },
    ],
  },
});
```

That is useful for portability, export, and compatibility, but it is not the
primary authoring shape a React or TypeScript application should learn first.

The honest fix is not a JavaScript-local computed cache. The honest fix is a
host callback computed node:

- TypeScript owns ergonomic callable handles and callback invocation.
- `worth-signals-wasm` owns callback registration, marshalling, and lowered
  node declarations.
- `worth-signal` continues to own derived execution, dependency graph
  mutation, invalidation, observation, rollback, and diagnostics truth.

After QA, this spec makes one sequencing correction explicit:

- `worth-signal` must own the generic host-computed substrate.
- `worth-signals-wasm` must implement that substrate for JavaScript callbacks.

The wasm package may prototype integration details, but the milestone is not
architecturally complete if dynamic dependency replacement, host-computed
failure semantics, and dependency patch commit/rollback live only in wasm glue.

## Hard Part

The hard part is dynamic dependency truth.

This must work:

```ts
const label = computed(() => enabled() ? name() : "disabled");
```

When `enabled` is false, the live dependency set is only `enabled`. When it
turns true, `name` becomes a dependency. When it turns false again, `name`
must stop being an active dependency for that computed node.

The runtime must not:

- keep stale dependencies forever
- under-declare dependencies because the first run did not read a branch
- recompute broad downstream surfaces because dependency replacement is easier
  than dependency patching
- let React keep a private derived value that disagrees with runtime truth
- make diagnostics explain the AST lane honestly but callback lane vaguely

Dynamic callback dependency replacement is therefore a runtime capability
boundary, not just package polish.

## Adversarial Constraint

This milestone must survive the following hostile condition:

> A long-lived web runtime with callback-backed computed nodes whose read sets
> change across branches, transactions, rollback, snapshot restore, React mount
> churn, and watcher churn must converge to the same committed values, dynamic
> dependency graph, observation boundaries, performance counters, and
> diagnostics explanations as an equivalent runtime-authored graph, without
> preserving stale dependencies or inventing React-local derived truth.

Concretely, the design must remain correct when:

- callback computed nodes switch dependency branches repeatedly
- callbacks throw during initial declaration and during later evaluation
- a callback reads a computed that itself has dynamic dependencies
- transactions roll back after dependency capture has staged a new read set
- watcher/effect delivery would have fired if rollback had not occurred
- snapshot restore returns to a prior dependency shape
- React components mount and unmount while callback computations continue
- diagnostics tier changes retained richness but not canonical runtime truth

If any path produces the right final value with the wrong dependency graph, the
milestone has failed.

## QA Findings That Shape The Final Spec

This section records the traps this milestone must not dodge.

### Finding 1: WASM-Only Callback Semantics Would Be The Wrong Authority

If host-computed dependency patching and rollback live only in
`worth-signals-wasm`, the core runtime will have two computed stories:
runtime-native computed nodes and wasm-private callback computed nodes.

Why it matters:

- diagnostics, replay, branch restore, and observation would have to special
  case wasm callback artifacts
- future non-JS hosts would reimplement the same lifecycle badly
- the core runtime would not actually own derived computation semantics for
  one of its most important easy-mode shapes

Required correction:

- add a core `worth-signal` host-computed substrate with sealed proof types for
  host evaluation requests, captured dependency sets, dependency patches,
  staged host-computed results, and committed host-computed artifacts
- make wasm the JavaScript evaluator implementation, not the semantic owner

### Finding 2: Callback Purity Cannot Be Hand-Waved

JavaScript callbacks can read anything: globals, dates, mutable arrays,
closures, DOM state, random values, local variables, and network caches. Only
signal reads are visible to WORTH.

Why it matters:

- `computed(() => count() + window.innerWidth)` looks reactive but has a hidden
  dependency the runtime cannot invalidate
- replay and diagnostics can only be honest about captured signal reads
- hidden host reads turn callback computed nodes into nondeterministic caches

Required correction:

- define the milestone callback-computed contract as deterministic over
  captured signal reads
- treat undeclared host reads as non-reactive by contract
- provide diagnostics and development assertions that can identify callback
  evaluations with no captured reads, changing outputs without dependency
  changes, or declared impurity posture
- if a future explicit host capability lane is added, it must be specified as a
  separate typed resource family with its own ownership, invalidation,
  replay/restore, and diagnostics contract rather than being silently folded
  into ordinary closure capture
- reserve compiler transforms or linting as future enforcement, not a hidden
  requirement for this milestone

### Finding 3: Reentrancy And Cycles Are The Failure Mode, Not A Detail

Callbacks can read other computed callbacks. They can also accidentally read
themselves through an alias or trigger a lazy evaluation chain that cycles.

Why it matters:

- active collectors can be corrupted by nested evaluation
- self-dependency can deadlock or produce partially captured read sets
- callback failure in an inner computed can leave the outer computed with a
  bogus dependency story if the evaluation stack is not proof-typed

Required correction:

- add an evaluation stack with `HostEvaluationFrameId`, node identity, branch
  identity, and collector identity
- detect self-read and dynamic cycles before committing captured dependencies
- classify reentrancy denials separately from callback exceptions
- prove nested callback reads preserve the right parent/child collector
  boundaries

### Finding 4: TypeScript Structural Typing Can WORTH Handles Unless Branded

An object with `{ id, get }` can satisfy many TypeScript handle-like types.
That is convenient, but not safe enough for the callback authoring surface.

Why it matters:

- WORTHd handles can bypass runtime identity checks until runtime failure
- cross-runtime reads can look type-correct
- `set` can appear on a value that should be read-only inside computed
  evaluation

Required correction:

- public TypeScript handles must be branded with private `unique symbol`
  markers or equivalent opaque constructors
- writable input handles and read-only signal handles must be distinct
  categories
- callback read contexts must expose only read capability, never transaction or
  write capability
- runtime checks still exist at the JS/Wasm trust boundary, but ordinary TS
  code should not be able to WORTH a valid signal handle accidentally

### Finding 5: Sync Computed Must Reject Async And Reference-Leaking Values

If a callback returns a `Promise`, a DOM object, a mutable class instance, or a
reference retained by the app, the runtime cannot honestly compare, snapshot,
or replay the value as `SignalValue`.

Why it matters:

- async semantics belong to the resource substrate, not sync computed
- mutable object references can change after commit without a transaction
- identity and output-change classification become heuristics

Required correction:

- sync callback computed must return values that marshal into the supported
  canonical value model
- `Promise` return values are a typed denial, not an implicit async computed
- non-canonical host objects are denied or explicitly wrapped by a future
  host-object capability lane outside this milestone
- committed callback values are snapshots, not live references

## Governing Summaries

- `MENTALITY.md`
  protects the requirement to solve the adversarial constraint first. This
  spec therefore starts from dynamic dependency replacement, rollback, and
  diagnostics parity rather than from a pretty wrapper around `computed`.
- `arch_laws.md`
  most strongly shapes this spec through authority/derivation separation,
  phase-typed observation, framework-owned resource lifecycle, declarative
  resource definition, proof-carrying phase boundaries, and facade-only public
  access. Callback functions may be host code, but callback computed truth must
  remain runtime-owned derived state.
- `perf_laws.md`
  protects breadth honesty. Callback evaluation, dependency capture, dependency
  replacement, and JS/Wasm marshalling must expose counters and must scale with
  callback read breadth plus changed dependency delta, not with whole graph
  size or active React subscriber count.
- `domain_laws.md`
  protects responsibility-shaped structure. Callback registry, dependency
  capture, callback evaluation, TypeScript authoring, React consumption, and
  compatibility AST recipes must remain separate responsibility spaces.
- `worth_signal_vision.md`
  protects the thesis that `worth-signal` owns derived computation, not truth
  storage. The callback lane must compile to the same derived-computation
  runtime story as native/easy computed signals.
- `worth_signal_temporal_async_roadmap.md`
  protects the rule that later layers consume runtime semantics rather than
  redefining them. Even though this milestone is wasm-focused, callback
  computed nodes must not become adapter-local lifecycle truth.
- `dx_plan.md`
  protects the product standard: the default API should express semantic
  intent instead of internal ceremony. Callback computed is the intended
  default authoring form; AST recipes remain an advanced lane.
- `test-requirements.md`
  protects hostile parity testing. This milestone must extend the existing
  replay, observation, granularity, diagnostics, and boundedness proof style to
  host callback computations.
- `web_runtime_spec.md`
  protects the framework-agnostic app-first runtime. Callback computed belongs
  in the wasm runtime surface itself, not only in the React adapter.
- `react_adapter_spec.md`
  protects React as a consumer of runtime truth. React hooks may subscribe to
  callback computed signals, but they may not recompute callback values as a
  separate selector engine.
- `milestone-11-closeout.md`
  protects committed observation semantics. Callback computed evaluation must
  stage observation through the same commit-bounded and rollback-suppressed
  substrate.
- `milestone-a-plan.md`
  protects the broader rule that host callbacks cannot own semantics the
  runtime must replay. This callback milestone may invoke host code for value
  production, but runtime-visible dependencies, commit artifacts, and
  diagnostics must be canonical.
- `milestone-b-closeout.md`
  protects callback-era lifecycle discipline from the async substrate: host
  work can exist, but admission, denial, lifecycle records, diagnostics, and
  boundedness must be runtime-owned where they affect runtime truth.

## Product Decision Lock

- callback computed is the primary TypeScript authoring shape
- AST/spec computed remains supported as an advanced, portable, export-friendly
  recipe lane
- callback computed nodes are real runtime nodes, not TypeScript-local caches
- generic host-computed dependency patching and rollback semantics belong in
  `worth-signal`; JavaScript callback invocation belongs in
  `worth-signals-wasm`
- callable signal handles are the default value-read syntax for the callback
  surface
- dependency capture is runtime-visible and diagnostic-bearing
- dynamic dependency replacement is in scope for the first complete milestone
- callback evaluation errors are typed runtime boundary failures with rollback
  behavior, not raw uncaught JS exceptions that leave partial state
- sync callback computed nodes reject `Promise` and non-canonical host object
  returns with typed denials
- callback computed purity for this milestone is defined as determinism over
  captured signal reads
- any future explicit host-capability lane is a separate typed resource family,
  not an ambient closure-read exception
- React consumes callback computed nodes through the existing React store and
  committed observation substrate
- output callback support should use the same substrate once computed callback
  support is real; output is not a separate callback engine

Normative consequence:

- any implementation that supports only stable dependency sets is incomplete
- any implementation that keeps stale dependency edges after branch changes is
  out of spec
- any implementation that computes callback values in React state rather than
  WORTH runtime state is out of spec
- any implementation that hides callback dependency capture from diagnostics is
  out of spec
- any implementation that lets callback dependency patching exist only as a
  wasm-private side path is out of spec
- any implementation that treats `Promise` returns as sync computed values is
  out of spec
- any implementation that commits mutable host object references as runtime
  values is out of spec
- any implementation that makes callback computed values unavailable to
  history, snapshot, restore, or compatibility reads is out of spec unless it
  emits an explicit typed non-support artifact for a deliberately deferred lane

## Scope

### In Scope

- package-root TypeScript authoring APIs for callable signals and callback
  computed nodes
- core `worth-signal` host-computed substrate for dynamic dependency capture,
  dependency patching, rollback, observation, and diagnostics
- callback registration and lifecycle ownership in `worth-signals-wasm`
- host callback invocation with typed return-value marshalling
- active dependency capture across callable signal reads
- dynamic dependency replacement after each successful evaluation
- rollback-safe staging of callback values and dependency changes
- diagnostics, performance counters, and explanation artifacts for callback
  computed nodes
- React harness coverage proving callback computed nodes behave like runtime
  truth
- compatibility documentation that distinguishes callback and AST recipe lanes

### Explicitly Out Of Scope

- async/resource callback semantics
- Suspense, route loaders, forms, or framework resource APIs
- cross-tab persistence of callback function identities
- worker migration of live JavaScript closures
- serializing callback source code as a replay artifact
- replacing the AST recipe lane
- async computed or resource-returning callback semantics
- mutable host-object value lanes

Callback closures are process-local host capabilities. The runtime may retain
their committed values, dependency facts, and diagnostics artifacts, but it may
not claim that a JavaScript function body is portable runtime data.

## Public API Model

The target primary authoring surface is:

```ts
import { computed, signal, type Signal } from "worth-signals-wasm";

const count = signal(1);
const doubleCount: Signal<number> = computed(() => count() * 2);

batch(() => {
  count.set(2);
});
```

The existing app-object form remains valid:

```ts
const signals = createSignals();
const count = signals.input("count", 1);
const doubleCount = signals.computed(() => count() * 2);
```

The exact naming of global versus instance helpers may be finalized during
implementation, but the semantic requirements are locked:

- `Signal<T>` is callable and returns the current committed value
- public handles are opaque and branded; plain objects with `{ id, get }` are
  compatibility targets, not valid callback-authoring handles
- `InputSignal<T>` and read-only `Signal<T>` are distinct types even when the
  same underlying runtime node is involved
- input signals expose an explicit mutation lane such as `.set(...)` or
  transaction-scoped `tx.set(...)`
- computed callbacks read other signals by calling them
- `computedSpec(...)` or an equivalent advanced spelling keeps the serialized
  recipe path available without making AST authoring the default story
- callback computed handles can be passed to `watch`, `effect`,
  `useSignalValue`, diagnostics, history reads, and compatibility reads where
  those surfaces already accept computed handles

### Identity And Naming

The callback surface must support both generated and explicit ids.

Generated ids are acceptable for local app ergonomics, but explicit ids must
exist for:

- diagnostics readability
- stable React/store subscription keys
- snapshot and history inspection
- integration tests and exported runtime envelopes

Acceptable shapes include:

```ts
const doubleCount = computed("doubleCount", () => count() * 2);
const doubleCount = computed(() => count() * 2, { id: "doubleCount" });
```

The implementation must choose one canonical public form and document the
other only if it genuinely improves readability without broadening overload
ambiguity.

### Callback Purity Posture

Callback computed nodes are reactive only to captured signal reads.

This is valid:

```ts
const total = computed(() => price() * quantity());
```

This is non-reactive with respect to `taxRate` unless `taxRate` is also a
signal:

```ts
let taxRate = 0.08;
const total = computed(() => subtotal() * (1 + taxRate));
```

The implementation must make that boundary visible:

- docs must state that ordinary closure variables are not tracked
- diagnostics must expose captured read sets so hidden dependencies are
  inspectable
- a callback with no captured reads must be classified as constant or
  host-opaque rather than silently treated as signal-reactive
- this milestone does not admit an ambient "declared host capability" escape
  hatch for ordinary callback purity; any future host capability lane must
  introduce typed descriptors, lifecycle ownership, invalidation evidence, and
  replay/restore honesty as a separately specified resource family

This milestone does not require a TypeScript compiler transform or linter to
detect hidden closure reads. It does require the runtime and docs to avoid
pretending those reads are reactive.

## Architecture

## Core And WASM Responsibility Matrix

This milestone has two implementation ownership lanes.

`worth-signal` must own:

- host-computed node descriptors
- host evaluation request/admission proof types
- captured read-set proof types
- dependency patch proof types
- staged host-computed result artifacts
- committed host-computed artifacts
- rollback of host-computed value and dependency shape
- observation and diagnostics integration
- performance counters for host-computed dependency capture and patching
- failure/denial taxonomy that is independent of JavaScript

`worth-signals-wasm` must own:

- JavaScript callback registry
- JavaScript callback invocation
- JavaScript thrown-exception capture and conversion into core host-computed
  failure types
- JavaScript value canonicalization into the supported signal value model
- TypeScript callable signal handles
- TypeScript handle branding
- active JS read collector bridge into the core captured-read-set contract
- package exports and React harness coverage

`worth-signals-wasm` ownership here does not permit ambient process-global
callback folklore. Watch/effect and compute-callback capability registries must
still be runtime-scoped, generation-aware, lifecycle-inspectable resources
rather than bare global id maps that happen to live in wasm glue.

`worth-signals-wasm` must not own:

- dependency patch commit semantics
- rollback semantics for dependency replacement
- observation semantics
- diagnostics truth
- a separate host-computed lifecycle model

Normative reading rule:

If an implementation plan can be completed by editing only
`crates/worth-signal-wasm`, it has misread this spec.

### Callback Computed Is A Runtime Recipe Family

The core runtime must distinguish at least two recipe families:

- serialized expression recipes
- host callback recipes

Both families lower into runtime-managed computed nodes. They may differ in how
values are produced, but they must not differ in:

- committed value storage
- dependency graph authority
- version and aspect updates
- output-change classification
- observation delivery
- rollback behavior
- diagnostics availability

The generic host-computed family belongs in `worth-signal`. It should have
explicit internal vocabulary. Exact names may change, but the architecture
must preserve distinct concepts equivalent to:

- `HostComputedCallbackId`
- `HostComputedDescriptor`
- `HostComputedEvaluationRequest`
- `HostComputedReadCollector`
- `HostComputedReadSet`
- `HostComputedDependencyPatch`
- `HostComputedEvaluationOutcome`
- `HostComputedFailure`
- `CommittedHostComputedArtifact`

`worth-signals-wasm` then implements a JavaScript evaluator adapter for that
generic host-computed family.

### Dependency Capture

Callable signal reads must register with an active read collector before
returning the current value.

Capture must record:

- signal id
- signal kind if known
- selected aspect or aspect set
- partition or region scope if supported by the read
- read ordinal for canonical ordering
- branch/runtime identity

Capture must not depend on parsing JavaScript source or transforming callback
code.

Capture must also include enough context to reject or explain invalid reads:

- evaluation frame id
- current computed node id
- callback descriptor id
- runtime generation
- branch epoch
- active collector generation

### Dynamic Dependency Replacement

After each successful callback evaluation, the runtime must compare the
captured read set to the currently committed dependency set for that computed
node.

The result is a dependency patch:

- retained dependencies
- added dependencies
- removed dependencies
- aspect/scope changes
- canonical digest before and after

That patch is staged with the computed value update and committed atomically.

Rollback must restore both:

- prior committed value/version/identity
- prior committed dependency set

The patch must operate over dependency edge indexes, not over a reconstructed
whole graph snapshot. Dependency replacement is allowed to touch:

- the callback node's prior dependency footprint
- the callback node's newly captured dependency footprint
- subscriber/invalidation index entries corresponding to added and removed
  edges

It is not allowed to rebuild unrelated dependency indexes as the ordinary path.

### Callback Invocation Boundary

Callback invocation is a boundary crossing and must be explicit in API,
diagnostics, and counters.

The callback evaluator must:

- install the active read collector
- invoke the JavaScript function
- marshal its return value into `SignalValue` or the supported generic value
  model
- reject `Promise` and non-canonical host object returns for sync computed
  nodes
- capture thrown JS exceptions as typed callback failures
- always clear the active collector before returning across the boundary
- deny nested illegal mutation from inside computed evaluation

Mutation inside computed evaluation must be structurally rejected. A computed
callback may read committed values; it may not call `set`, `transaction`, or
otherwise mutate runtime state during evaluation.

### Reentrancy, Nesting, And Cycle Safety

Callback evaluation must be stack-aware.

The runtime must track a host-computed evaluation stack containing:

- evaluation frame id
- node id
- callback descriptor id
- branch epoch
- collector id
- parent frame id, if this evaluation was triggered by a nested read

The evaluation stack must classify:

- legal nested reads of other computed nodes
- self-read attempts
- dynamic cycles discovered through nested callback evaluation
- callback exceptions
- collector corruption or missing-frame errors

Self-read and dynamic-cycle cases are runtime denials, not ordinary callback
exceptions.

Nested callback evaluation must preserve parent and child collectors
separately. A child callback's captured reads may influence the child node's
dependency patch, but may not leak into the parent node's captured read set
except as the parent reading the child signal.

### Canonical Value Boundary

The sync callback lane commits canonical value snapshots.

Allowed callback return values must marshal into the package's supported value
model:

- null
- boolean
- number
- string
- arrays of supported values
- plain objects whose fields are supported values

The implementation must deny:

- `Promise`
- `Date`, `Map`, `Set`, DOM objects, functions, symbols, class instances, and
  other non-canonical host objects
- cyclic object graphs
- values that cannot be canonicalized for identity/output-change comparison

The committed value must not retain live mutable references owned by the host
application.

### Diagnostics Parity

Callback computed nodes must explain themselves through the same diagnostic
story as expression recipes.

At minimum diagnostics must expose:

- callback node id
- callback recipe family
- latest captured read set
- dependency patch summary
- output-change classification
- callback invocation count
- callback failure count
- last failure classification
- JS/Wasm marshalling breadth
- dynamic dependency replacement breadth

The explanation must answer:

- why this callback computed node was dirtied
- which captured dependencies were considered
- whether the callback ran
- whether the output was replaced, refreshed, or unchanged
- whether downstream propagation was suppressed
- whether dependency shape changed during evaluation

Diagnostics richness may vary by diagnostics policy, but canonical dependency
truth may not.

### History, Snapshot, Restore, And Replay

Callback functions themselves are not serializable runtime truth.

The runtime must be honest about that boundary:

- committed callback values are runtime state
- committed callback dependency sets are runtime state
- callback ids and descriptor metadata are runtime state
- JavaScript function bodies are host capabilities

Snapshot, restore, replay, export, and import must not be collapsed into one
undifferentiated portability claim. These lanes cross different trust and
capability boundaries and must remain distinct in both artifacts and
diagnostics:

- live snapshot/restore in a runtime where the required callback capabilities
  remain registered
- same-host replay where equivalent callback capabilities are reattached before
  resumed evaluation
- export/import or cross-host restore where callback bodies are unavailable and
  only committed callback-bearing artifacts can be carried forward honestly

Because JavaScript function bodies are host capabilities rather than portable
runtime data, the runtime must ship self-describing callback-bearing artifacts
that record:

- committed callback-derived values
- committed dependency shape
- callback descriptor identity and generation metadata
- required callback capability references
- explicit capability-availability and compatibility markers

The contract must freeze at least these artifact/result variants:

- `live_restore_artifact`: restores committed callback-derived value and
  dependency shape in a runtime that still owns the callback capability
- `reattached_replay_artifact`: resumes equivalent callback-bearing runtime
  truth only after an explicit capability reattachment step succeeds
- `callback_unavailability_artifact`: carries committed callback-derived
  snapshots plus the typed reason live reevaluation is unavailable

If a restore, replay, export, or import lane lacks the callback capability
required to evaluate a host callback node, it must emit the explicit typed
unavailability artifact rather than silently recomputing from stale values,
pretending the callback body itself was portable runtime data, or blurring the
difference between live restore and callback-missing import.

### React Adapter Boundary

The React adapter remains a consumer.

React may:

- subscribe to callback computed handles
- read their committed snapshots through `useSignalValue`
- observe diagnostics snapshots
- benefit from store-level fanout

React may not:

- rerun callback computed functions as selector logic
- own dependency capture
- decide dynamic dependency replacement
- maintain a separate derived cache that can disagree with runtime truth

## Required Architecture Changes

### Core Host-Computed Substrate

Add a generic host-computed substrate to `worth-signal`.

It must own:

- host-computed node descriptors
- evaluation request/admission proof types
- dynamic read-set proof types
- dependency patch proof types
- staged result and rollback artifacts
- committed host-computed artifacts
- host-computed diagnostics summaries and counters

It must not own:

- JavaScript `Function` values
- TypeScript handle objects
- React store subscriptions
- host-specific callback registries
- semantics that depend on monkey-patching shared public facade objects after
  construction
- ordinary app-read behavior that must fall back through compatibility lanes to
  stay correct
- public diagnostics truth that exists only as opaque `JsValue` or `unknown`
  blobs at the package boundary

This substrate must expose a narrow evaluator boundary that a host adapter can
implement. The evaluator boundary must receive a proof-bearing evaluation
request and return either:

- a canonical value plus captured read-set evidence
- or a typed host-computed failure/denial

The core runtime remains responsible for committing or rolling back the result.

### Phase-Separated Forms

The implementation must preserve this minimum phase split:

- callback authoring intent
- registered callback capability
- frozen host computed descriptor
- admitted host evaluation request
- active evaluation frame
- captured read set
- validated captured read set
- dependency patch
- staged callback computed result
- committed callback computed artifact
- diagnostics/explanation projection

Raw JavaScript `Function` values must not flow directly into transaction apply
or diagnostics surfaces. They must be represented through callback ids and
capability-owned registry entries.

### Callback Registry

Add or extend a wasm callback registry for compute callbacks.

The registry must own:

- callback id allocation
- callback disposal
- invocation counters
- failure counters
- active callback count
- lookup by callback id
- callback generation or disposal epoch

The registry must not own:

- committed computed values
- dependency graph truth
- observation delivery
- React subscriptions

### Read Collector

Add a host-computed read collector with a strict lifecycle.

The collector must be a proof producer:

- only an admitted host evaluation request can install it
- only the active evaluation frame can append reads
- only the collector close step can produce a validated captured read set
- only a validated captured read set can enter dependency patch construction
- failed evaluations consume and retire the collector without producing a
  dependency patch

The collector should fail loudly on:

- reading a signal from a different runtime
- mutating during callback evaluation
- attempting to use a disposed signal handle
- dependency capture after callback evaluation has ended

### Runtime Core Integration

The existing runtime core currently evaluates serialized expression recipes by
reading declared dependencies and evaluating an expression environment.

Host computed support must add a sibling path that:

- reads the committed descriptor for a host callback recipe
- asks the host evaluator adapter to evaluate an admitted host request
- receives a canonical value plus validated captured read set
- stages a dependency patch and output change together
- uses the same version, output identity, observation, and diagnostics
  semantics as expression recipes

The implementation may keep JavaScript invocation inside
`worth-signals-wasm`, but the semantic path for dependency patching, commit,
rollback, and diagnostics must be in `worth-signal`.

### Required Module Ownership

The implementation should create responsibility-shaped modules rather than
folding callback support into existing broad files.

Expected ownership:

- `worth-signal` host-computed data model
- `worth-signal` host-computed evaluation admission
- `worth-signal` host-computed dependency patching
- `worth-signal` host-computed diagnostics/certification
- `worth-signals-wasm` JavaScript callback registry
- `worth-signals-wasm` JavaScript read collector bridge
- `worth-signals-wasm` TypeScript callable authoring facade
- `worth-signals-wasm/react` consumption tests and adapter typing

The exact filenames may differ, but a single "callback helpers" file or
monolithic wasm facade expansion is out of spec.

## Phases

### Phase 1: Core Host-Computed Substrate

Deliver:

- generic `worth-signal` host-computed descriptor model
- sealed host evaluation request and response proof types
- validated read-set and dependency patch proof types
- staged and committed host-computed artifact types
- host-computed failure/denial taxonomy
- host-computed counters and diagnostics summary skeleton

Must prove:

- dynamic dependency patching is not wasm-private
- external code cannot WORTH host evaluation requests, read sets, dependency
  patches, staged results, or committed artifacts
- the core runtime can represent host-computed success, denial, and failure
  without knowing JavaScript exists

### Phase 2: Callback Capability Registries And Invocation Boundaries

Deliver:

- compute callback registry
- watch/effect callback registry cleanup so observation callbacks follow the
  same runtime-scoped lifecycle discipline instead of ambient global id maps
- callback lifecycle and disposal semantics
- typed callback invocation result and failure artifacts
- JS/Wasm marshalling for callback return values
- invocation/failure counters
- generation-aware callback capability identity for both compute and
  observation/effect callback families

Must prove:

- callback failures cannot leave active collectors installed
- disposed callbacks cannot be invoked as live runtime truth
- callback invocation cost is counted separately from runtime evaluation cost
- callback generations prevent stale callback ids from invoking newly
  registered callbacks after disposal/reuse
- watch/effect callback lifecycles are no longer modeled as ambient process
  globals unrelated to runtime ownership

### Phase 3: Read Collector And Callable Signal Handles

Deliver:

- active read collector
- callable signal read path
- runtime/branch scoped read validation
- read-set canonicalization with read ordinals
- denial of cross-runtime reads and mutation during evaluation
- frame-aware nested callback collection
- sync value canonicalization and denial of `Promise` or non-canonical returns

Must prove:

- `computed(() => count() * 2)` captures `count`
- conditional callbacks capture only the branch actually read on that
  evaluation
- nested computed reads preserve a canonical dependency story
- the collector clears after both success and failure
- self-read and dynamic cycle attempts produce typed denials
- non-canonical callback return values cannot commit

### Phase 4: Host Computed Runtime Recipe Family

Deliver:

- host callback recipe storage beside expression recipe storage
- runtime evaluator branch for callback recipes
- value, identity, version, and output-change handling through existing runtime
  semantics
- computed and output callback support on one substrate

Must prove:

- callback computed values are readable through the same handle/read surfaces
  as expression computed values
- output-change classification matches expression recipes for equivalent value
  transitions
- callback values participate in watch/effect delivery only through committed
  runtime observation
- output callback support does not create a second host-callback engine or a
  second diagnostics story

### Phase 5: Dynamic Dependency Patching

Deliver:

- committed dependency-set tracking for host callback nodes
- dependency patch construction after each successful evaluation
- atomic staging of dependency patch plus output update
- rollback restoration of prior dependency shape
- stale dependency removal from downstream invalidation indexes
- dependency edge patching through node-local and subscriber indexes rather
  than graph-wide rebuilds

Must prove:

- `enabled() ? name() : "disabled"` adds and removes `name` as the branch
  changes
- removed dependencies no longer dirty the callback computed node
- added dependencies dirty and recompute the node on future relevant changes
- rollback restores both value and dependency graph truth
- dependency patch counters exactly name added, removed, retained, and touched
  subscriber-index entries

### Phase 6: Callback-Bearing Artifacts, Restore, And Compatibility Truth

Deliver:

- callback-bearing staged/committed artifact projections
- live snapshot/restore support for callback-bearing runtimes
- replay/export/import through typed callback-bearing artifacts that distinguish
  capability-reattached replay from callback-unavailable transport lanes
- typed unavailability artifacts whenever required callback capabilities are
  missing or incompatible
- compatibility reads that return callback computed committed values without
  redefining app-first truth

Must prove:

- live snapshot restore preserves callback dependency shape and callback
  capability requirements
- replay with explicit capability reattachment reconstructs equivalent
  callback-bearing runtime truth
- export/import or callback-missing restore lanes fail explicitly with typed
  capability-unavailability artifacts rather than pretending reevaluation
  occurred
- compatibility lanes can observe committed callback truth without becoming the
  required happy path for ordinary app-first consumers

### Phase 7: Diagnostics And Measurement Boundaries

Deliver:

- callback dependency capture diagnostics
- callback dependency patch diagnostics
- callback failure diagnostics
- latest observation/latest flow parity for callback computed changes
- typed public diagnostics and adapter artifacts at the package boundary
- explicit public measurement boundaries for callback costs, patch breadth,
  lifecycle activity, and observation fanout

Must prove:

- diagnostics can explain callback-computed dirtiness, evaluation, dependency
  replacement, output suppression, and failure
- diagnostics tier changes retained richness but not canonical dependency truth
- hidden host-dependency posture is visible through captured read summaries and
  host-opaque/constant classifications
- typed consumers can inspect callback diagnostics and performance artifacts
  without reconstructing semantics from `JsValue`/`unknown` blobs
- acceptance proofs assert against named public measurement surfaces rather than
  private internal counters

### Phase 8: Public Authoring Contract And Handle Hardening

Deliver:

- the public callback-computed authoring contract
- typed `Signal<T>` callable handle model
- explicit advanced AST/spec recipe naming
- callback id and descriptor vocabulary
- callback mutation denial rule
- stronger branded/opaque handle categories
- stricter runtime/store ownership cues at the package surface

Must prove:

- callback computed authoring is not a wrapper over React state
- AST recipes remain available but no longer define the default product story
- the public types keep input, computed, output, disposable, and transaction
  categories distinct
- TypeScript signal handles are branded/opaque enough that ordinary structural
  objects cannot masquerade as callback-authoring handles
- writable inputs cannot be confused with computed or output handles
- raw wasm handles do not silently substitute for product-facing callable
  handles

### Phase 9: React Adapter, Package Prep, And Public Documentation

Deliver:

- React harness tests using callback computed nodes
- package root exports for callback-first authoring
- generated declaration updates
- docs replacing AST-first examples with callback-first examples where
  appropriate
- compatibility docs retaining the serialized recipe lane
- React store integration that does not monkey-patch shared `Signals` facade
  methods at runtime
- normal React/package read paths that do not require compatibility-lane reads
  for ordinary callback-first consumption

Must prove:

- `useSignalValue(computed(() => count() * 2), store)` rerenders only after
  committed meaningful changes
- React mount/unmount churn does not resurrect stale callback subscriptions
- multiple components reading the same callback computed signal use runtime
  observation fanout rather than React-local recomputation
- installing a React store does not rewrite the behavior of unrelated consumers
  sharing the same `Signals` instance
- the normal app-first read path remains correct without routing ordinary reads
  through compatibility-only APIs

## Must Ship

- generic `worth-signal` host-computed substrate
- callback-first `computed` authoring for normal JavaScript/TypeScript code
- callable signal handles with typed value reads
- branded/opaque TypeScript handle categories
- host compute callback registry and lifecycle
- active read collector and canonical captured read sets
- reentrancy and dynamic-cycle denial semantics
- sync canonical value boundary with `Promise` and non-canonical host-object
  denials
- dynamic dependency replacement with dependency patches
- rollback-safe staging of callback values and dependency changes
- diagnostics parity with expression recipes
- history/snapshot honesty for live callback-registered runtimes
- explicit unavailability artifacts where callback functions are not available
  for replay or restore
- React harness and type smoke coverage for callback computed usage
- package/docs updates that teach callback computed first and AST recipes as
  advanced compatibility

## Must Preserve

- `worth-signal` remains derived computation runtime, not truth storage
- `worth-signals-wasm` remains framework-agnostic
- React remains a consumer of runtime truth
- transactions remain the only write boundary
- callback evaluation remains read-only
- rollback suppresses normal delivery
- diagnostics richness policy cannot change committed dependency truth
- AST/spec recipes remain available for portable and export-friendly use
- hidden closure variables remain non-reactive unless represented by an
  explicit host capability
- sync computed does not absorb async/resource lifecycle semantics

## Acceptance Evidence

This milestone is complete only when the following named test families pass.

### The Callback Computed Dynamic Dependency Parity Test

Build callback and expression-equivalent graphs with conditional dependency
branches.

Verify:

- final values match
- dependency graph snapshots match the active branch
- removed dependencies no longer invalidate the callback node
- added dependencies do invalidate the callback node
- explanation digests identify the same causal dependency branch

### The Callback Failure Rollback Test

Inject callback failures during declaration seeding and later recomputation.

Verify:

- no partial dependency patch commits
- no partial value/version update commits
- no observer delivery escapes failed evaluation
- active collectors clear after failure
- diagnostics expose the typed callback failure

Also verify:

- thrown JS exceptions, self-read denials, dynamic cycle denials, disposed
  callback denials, `Promise` return denials, and non-canonical value denials
  are distinct failure classes
- the node's post-failure state is explicitly classified as preserved,
  failed-but-retained, or unavailable according to the chosen runtime policy
  rather than left implicit

### The Callback Observation And React Equivalence Test

Render multiple React components reading input, callback computed, and output
callback/projection handles.

Verify:

- React snapshots equal direct runtime reads
- rerenders follow committed meaningful-change observation
- rollback suppresses rerenders
- store fanout does not multiply runtime watch handles unnecessarily
- React never invokes the compute callback directly as selector logic

### The Callback Snapshot Restore Honesty Test

Create a graph whose callback dependencies change, capture snapshots before and
after the change, restore them in a runtime where callbacks remain registered,
and attempt a replay/restore path where callbacks are missing.

Verify:

- live restore recovers committed value and dependency shape
- equivalent suffixes converge to identical value/dependency digests
- missing-callback lanes produce typed unavailability artifacts
- stale callback values are not silently treated as recomputed truth

### The Callback Purity And Hidden Dependency Test

Create callbacks that read:

- only signals
- no signals
- mutable closure variables

Verify:

- signal-only callbacks expose complete captured read sets
- no-read callbacks are classified as constant or host-opaque
- closure-variable changes do not pretend to be reactive signal invalidations
- diagnostics make the purity posture inspectable

### The Callback Reentrancy And Cycle Test

Create nested callback-computed graphs including:

- parent reads child
- child reads grandchild
- self-read through alias
- two-node dynamic cycle
- inner callback failure while outer callback is evaluating

Verify:

- legal nested reads preserve separate collector frames
- parent read sets include the child signal, not the child's internal reads
- self-read and dynamic cycles are denied before dependency patch commit
- inner failure does not leak partial reads into the parent collector

### The Callback Diagnostics Parity Test

Compare expression recipe diagnostics and callback computed diagnostics for
equivalent graphs.

Verify:

- latest observation and latest flow agree on committed boundaries
- explain output names read dependencies, dependency replacements, output
  changes, and failures
- diagnostics tier changes retained detail only
- callback counters are attributable by node and API family

### The Callback Boundary Boundedness Test

Stress a large graph with a narrow dynamic callback dependency frontier.

Verify counters for:

- callback invocation count
- callback failure count
- captured read count
- dependency additions
- dependency removals
- retained dependency count
- dependency patch breadth
- subscriber-index touched entry count
- JS/Wasm return-value serialization breadth
- callback broad-scan denial count
- callback allocation/reuse count
- React subscriber count and runtime watch fanout

Pass condition:

- callback evaluation cost scales with captured read breadth and dependency
  patch breadth
- dependency replacement does not scan unrelated graph nodes
- React subscription cost remains bounded by runtime observation fanout
- dependency patching touches only prior/new dependency footprint plus
  corresponding subscriber-index entries

## Performance Contracts

The milestone must name and expose counters for:

- host-computed descriptor registration
- host evaluation request admission
- callback registration
- callback disposal
- callback generation mismatch denial
- callback invocation
- callback failure
- active read collector installation
- active evaluation frame count
- captured read count
- validated captured read-set count
- dynamic dependency additions
- dynamic dependency removals
- dynamic dependency retained count
- subscriber-index touched entry count
- dependency patch breadth
- callback return-value serialization breadth
- callback output-change classification
- rollback-restored callback dependency count
- missing-callback unavailability count
- self-read denial count
- dynamic-cycle denial count
- non-canonical return denial count
- promise-return denial count
- host-opaque/no-read classification count
- callback broad-scan denial count
- callback allocation count
- callback reuse count

These counters must not live only in internal tracing. They must be exposed
through explicit public measurement boundaries whose names match the claim being
made:

- callback evaluation and dependency-capture counters through the callback
  diagnostics/explanation surface
- dependency patch breadth and subscriber-index touch counters through the
  runtime inspection or boundedness artifact surface
- callback capability lifecycle counters through registry/lifecycle inspection
  surfaces
- replay/restore/import unavailability counters through the callback-bearing
  envelope or restore result surface
- React-facing observation fanout counters through the public observation or
  diagnostics boundary rather than hidden adapter-local instrumentation

Acceptance tests must assert against those named public artifacts or entrypoints
rather than against private counters harvested from internals after the fact.

The milestone must declare complexity contracts for:

- host evaluation request admission
- callback registration
- callback invocation and marshalling
- callable signal read with active collection
- read-set canonicalization
- dependency patch construction
- dependency patch commit
- dependency patch rollback
- reentrancy/cycle detection
- callback diagnostics summary read
- React subscription to callback computed signals

Each contract must state cost in terms of the real surface:

- captured read count
- current dependency count
- added dependency count
- removed dependency count
- retained dependency count
- subscriber-index touched entry count
- callback return-value breadth
- evaluation stack depth
- matching observer count
- React subscriber fanout

No contract may describe callback computed work as "constant" unless it names
the fixed maximum that makes the claim true.

## Architectural Notes

- Callback computed support should reuse as much existing expression recipe
  value/version/output-change machinery as possible.
- Dynamic dependency replacement should be represented as an explicit patch,
  not as broad graph rebuild.
- The callback registry should be a capability registry, not a storage engine.
- The read collector should be treated as an evaluation-scoped proof producer.
- Package-root callback authoring should be a thin product facade over the
  app-first runtime, not a forked global runtime unless the implementation
  explicitly chooses and documents a default singleton model.
- Generated ids are ergonomic, but explicit ids are required for serious
  diagnostics and integration scenarios.

## Sequencing Notes

This milestone belongs after the current web runtime and React adapter specs
because those specs established the app-first surface and React consumption
boundary. It must land before any serious form/resource/router product surface
because those layers will naturally depend on callback computed ergonomics and
dynamic dependency truth.

The milestone also belongs before presenting `worth-signals-wasm` as a polished
React state library. Without callback computed support, the package still asks
normal React users to author internal expression trees for everyday derived
state.

## Finish-Line Closeout Map

This spec now defines two separate finish lines:

- `milestone closeout`
  callback-backed computed nodes are semantically complete, runtime-honest,
  diagnostics-visible, and replay/restore/export/import-honest through one
  self-describing callback-bearing runtime-envelope story
- `crate maturity closeout`
  the package surface, capability vocabulary, handle model, diagnostics UX,
  and docs are polished enough that ordinary React and TypeScript users are
  naturally guided onto the right product surface and away from expert/raw
  lanes

These finish lines are intentionally separate. The milestone may be complete
before the crate feels mature, but the document should carry both obligations
forward so the follow-on work is not rediscovered from scratch later.

### Milestone Closeout Gate

The callback milestone is not closed until all of the following are true:

- callback-first `computed(() => ...)` is the normal authored path
- dynamic dependency rewiring is correct under branch churn, rollback, and
  restore
- callback functions, callback ids/generations, and callback lifecycle are
  framework-owned resources rather than ambient JS conventions
- diagnostics, latest observation, latest flow, replay/lineage summaries, and
  explanation surfaces treat callback-origin nodes as first-class runtime
  entities rather than second-tier annotations
- callback-bearing snapshot, restore, replay, export, and import all flow
  through one self-describing runtime-envelope contract with typed capability
  availability and unavailability artifacts
- the public package surface can demonstrate the callback lane from normal app
  code without dropping through raw wasm-bindgen surfaces

### Crate Maturity Closeout Gate

The crate is not mature merely because the milestone closes. Crate maturity
requires all of the following:

- one obvious product surface for ordinary React and TypeScript app code
- an explicit and teachable separation between product-facing surfaces and
  raw/compatibility/runtime-adjacent expert surfaces
- callback ids, callback ownership, callback disposal, and callback generation
  semantics that read as intentional product vocabulary rather than leftover
  plumbing
- diagnostics that feel first-class at the product boundary, not merely
  available if a user already understands the internals
- public diagnostics and adapter artifacts that are typed and self-describing at
  the package boundary rather than leaking critical semantics through
  `JsValue`/`unknown` blobs
- TypeScript handle categories strong enough that ordinary misuse becomes
  unrepresentable or materially harder to express
- legacy ambient callback registries are replaced or contained behind
  runtime-scoped capability objects with explicit lifecycle and generation
  semantics
- package docs, examples, and adapter surfaces that reinforce one canonical
  happy path instead of several half-equal ones

### Current Status

At the time of this revision, the callback-computed core is substantially done.

That means the document should no longer speak as though the main remaining work
is "make `computed(() => ...)` exist at all." The real remaining work is now:

- finish the last public-truth and replay/export surfaces
- close the package/publication proof lane
- make diagnostics feel first-class at the product boundary
- replace the remaining ambient callback-registry patterns with runtime-owned
  capability registries
- make `input`, `computed`, and `output` read like one coherent product surface
- remove React-store monkey-patching and other shared-facade rewrite shortcuts
- decouple normal app-first reads from compatibility-lane dependency
- harden the type and capability boundaries so the common misuse classes become
  materially harder or impossible to express

The remaining phases below are therefore not just cosmetic closeout. Some are
still foundation hardening phases that protect the crate shell from
convenience-era shortcuts even though the callback-computed core itself is
substantially working.

## Post-Milestone Crate Hardening

Finishing this milestone honestly is necessary, but it is not the same thing as
making `worth-signals-wasm` feel mature as a public crate. The milestone should
close the runtime-truth, rollback, dependency, snapshot, and diagnostics
obligations first. After that, the crate should receive an explicit hardening
pass aimed at product quality and long-term maintainability rather than merely
feature completion.

That hardening pass should be tracked as a follow-on effort, not smuggled in as
incidental cleanup. The goal is to make the crate feel inevitable and coherent,
not transitional or wrapper-heavy.

The hardening pass is expected to produce material React-facing DX gains, not
just internal tidiness. In the intended end state:

- ordinary React app code should see one obvious product surface for
  `input(...)`, `computed(...)`, `output(...)`, and store consumption
- those authoring forms should read like one coherent language rather than a
  mix of registration APIs and callback-first APIs
- normal app code should be able to organize signal logic into domain
  controller factories instead of forcing every feature into one monolithic
  graph declaration object
- advanced/raw/runtime-adjacent surfaces should remain available but should not
  leak into the default app-authoring path
- signal handles should be easy to inspect, easy to pass around, and difficult
  to misuse
- callback-backed derived state should feel like a native runtime feature, not
  a wrapper over lower-level wasm-bindgen objects
- callback failures, dependency rewires, runtime ownership mistakes, and
  lifecycle issues should become easier to diagnose from the public crate
  surface without deep runtime knowledge

Priority hardening targets:

- make the raw wasm boundary versus the product-facing package surface
  architecturally explicit rather than merely convenient
- unify authoring ergonomics so `input`, `computed`, and `output` read as one
  family of declarations rather than three different styles of runtime call
- move the primary app-authoring story toward controller-first signal
  composition with explicit graph publication, instead of giant node-registry
  objects with embedded compute closures
- refine callback capability vocabulary, callback id generation, and lifecycle
  ownership so they read as first-class crate concepts instead of leftover
  plumbing
- simplify and clarify signal handle categories, callable handle ergonomics,
  disposal semantics, and authoring overloads
- improve diagnostics discoverability so users can quickly tell what kind of
  signal they are holding, which runtime owns it, and why a callback failed or
  rewired
- tighten naming and module boundaries anywhere the current implementation
  still feels rushed, transitional, or over-coupled
- update docs/examples so the callback-first happy path is obvious and the raw
  or compatibility lanes are clearly intentional expert surfaces

The hardening pass should also explicitly target classes of bugs that are
currently possible but should become unrepresentable or substantially harder to
express in normal React application code:

- mixing signals, stores, or subscriptions across different runtime instances
- mutating computed or output handles as though they were writable inputs
- structurally forging signal-like objects that satisfy TypeScript shape checks
  but do not represent real runtime-owned handles
- passing raw wasm handles where product-facing callable handles are required,
  or vice versa, without crossing an intentional boundary
- leaking callback capability identity/lifecycle details into ordinary app code
  in ways that permit stale-use, disposal confusion, or accidental reuse
- treating callback-backed derived state as though it were React-local selector
  logic rather than committed runtime truth
- using compatibility or expert-only lanes by accident because the public API
  does not clearly separate product and raw surfaces
- teaching users an ID-first or spec-first style by accident because the
  guided surface does not present one obvious value-first callback-first
  authoring story

When possible, these protections should be compiler-enforced through branded or
opaque handle categories, runtime-owned capability construction, and stricter
surface separation rather than being left to documentation or runtime warnings
alone.

### Hardening Phase 1: Product Boundary And Capability Vocabulary

Deliver:

- one explicit product-facing package surface for ordinary app code
- one explicit raw/compatibility lane for expert or migration use
- one coherent authoring story for source state, derived state, and exposed
  state
- one coherent controller-first composition story for feature-level signal
  logic
- cleaned callback capability vocabulary for registration, identity,
  generation, and disposal
- removal of accidental overlap where raw and guided surfaces teach competing
  stories

The intended authoring shape should converge toward something like:

```ts
const count = signals.input(1, { id: "count" });
const doubled = signals.computed(() => count() * 2, { id: "doubled" });
const label = signals.output(() => `${doubled()}`, { id: "label" });
```

Exact spellings may vary, but the product requirement does not: `input`,
`computed`, and `output` should read as sibling declarations, with ids treated
as metadata rather than as the dominant first impression of the API.

The broader authoring target should also converge toward ordinary composed code
that looks like this:

```ts
export function createEditSessionController(signals: SignalNamespace) {
  const serverItemData = signals.input("serverItemData", null);
  const draftEdits = signals.input("draftEdits", {});

  const effectiveItemData = signals.computed("effectiveItemData", () => ({
    ...(serverItemData() ?? {}),
    ...(draftEdits() ?? {}),
  }));

  const dirtyState = signals.computed("dirtyState", () => ({
    isDirty: Object.keys(draftEdits()).length > 0,
  }));

  return {
    serverItemData,
    draftEdits,
    effectiveItemData,
    dirtyState,
  };
}

export function createWorkflowController(
  signals: SignalNamespace,
  editSession: ReturnType<typeof createEditSessionController>,
) {
  const submitReadiness = signals.computed("submitReadiness", () => {
    const item = editSession.effectiveItemData();
    const dirty = editSession.dirtyState();

    return {
      enabled: dirty.isDirty && Boolean(item.workflow_target_state_id),
      targetStateId: item.workflow_target_state_id ?? null,
    };
  });

  return {
    submitReadiness,
  };
}

const signals = createSignals();

const editSession = createEditSessionController(signals);
const workflow = createWorkflowController(signals, editSession);

export const itemDetailGraph = signals.graph("itemDetail", {
  outputs: {
    effectiveItemData: editSession.effectiveItemData,
    dirtyState: editSession.dirtyState,
    submitReadiness: workflow.submitReadiness,
  },
});
```

This code shape is not merely an example. It represents the desired product
direction:

- controllers author inputs and computed nodes as ordinary code
- controllers compose through returned typed handles rather than string lookup
- `signals.graph(...)` becomes the explicit publication boundary for outputs
- graph-shaped node registries remain available as compatibility/import/export
  lanes, not the default app-authoring story

Must prove:

- a React user can identify the correct authoring surface from the docs and
  types without reading runtime internals
- a new user can infer the difference between source state, derived state, and
  exposed state without first learning three unrelated call shapes
- a feature author can decompose signal logic into multiple controller factories
  without losing the runtime-owned graph/publication story
- callback ids and callback lifecycle semantics are explainable through public
  names alone
- raw surfaces remain available without defining the crate's first impression

### Hardening Phase 2: Make Misuse Unrepresentable

Deliver:

- stronger branded or opaque handle categories
- runtime-owned capability construction for callback-backed authoring forms
- stricter separation between writable inputs, read-only computed handles,
  output handles, and disposable resources
- stronger runtime ownership markers across store, signal, and observation
  surfaces
- authoring overloads that favor value-first / callback-first guided usage and
  demote registration-heavy shapes to explicit expert or compatibility lanes

Must prove:

- cross-runtime/store misuse becomes harder or impossible to type accidentally
- computed and output handles cannot masquerade as writable inputs
- structural signal WORTHry is blocked or sharply constrained at the normal
  product boundary
- raw wasm handles cannot silently stand in for product-facing callable handles
  without crossing an intentional conversion boundary
- the nicest-looking overloads are also the correct overloads for ordinary app
  code, so users are not nudged toward lower-level forms by aesthetics alone

### Hardening Phase 3: Diagnostics As A Product Job

Deliver:

- diagnostics entrypoints organized around user jobs rather than internal
  export families
- public answers for:
  - what kind of signal is this
  - which runtime owns it
  - why did it recompute
  - why did it not recompute
  - why did the callback fail
  - why did the dependency set rewire
- clearer callback-origin summaries at the package surface

Must prove:

- a user can diagnose common callback failures and ownership mistakes without
  dropping into raw runtime internals
- callback-origin diagnostics are at least as legible as expression-recipe
  diagnostics for equivalent graphs
- diagnostics richness remains strictly separate from runtime truth and hot-path
  correctness

### Hardening Phase 4: Documentation, Examples, And Publication Shape

Deliver:

- docs and examples that teach the callback-first happy path first
- intentional explanation of expert-only lanes and callback capability
  requirements
- package examples that exercise the guided product surface instead of raw
  internal compatibility flows
- a publication-ready package identity with coherent README, references, and
  adapter guidance

Must prove:

- examples do not accidentally train users onto deprecated, raw, or
  compatibility-first authoring paths
- guided tests cover the public product workflows we actually want users to
  adopt
- package identity feels singular and intentional rather than transitional

This hardening pass should happen after the remaining milestone truth-critical
work is complete, especially:

- runtime-scoped callback capability cleanup
- React/app-first boundary cleanup
- diagnostics parity
- replay/restore honesty
- output callback support
- cycle/reentrancy/failure hardening

The sequencing rule is: finish the truth-critical runtime semantics first, then
perform a deliberate crate-quality pass. Do not mistake "milestone complete"
for "crate finished."

## Practical Closeout Record

This section records the operational closeout map that carried the milestone
from "the feature works" to "the package is honest, publishable, and mature."

These phases were intentionally practical. They served as the implementation
and QA checklist for closing the remaining crate-shell and product-boundary
gaps after callback-computed core support was already largely working.

### Phase 0: Public Package Integrity And Release Proof

**Status:** Completed

**Why this phase exists**

The package cannot be treated as done if the prepared npm artifact omits
required root files, omits declaration files, or ships examples/docs that do
not match the real import surface.

**Deliver**

- a prepared npm artifact whose root `index.js`, `raw_surface.js`,
  `product/*`, `types/*`, and React subpath all exist in the packed tarball
- declaration files that describe the same contract the runtime JS actually
  exports
- package metadata and docs that teach the public package name and import paths
- a release-proof script path that rebuilds, prepares, packs, and verifies the
  artifact

**Must prove**

- `npm pack` contains every file transitively required by the public entrypoints
- a clean temp consumer can import:
  - `worth-signals-wasm`
  - `worth-signals-wasm/react`
- a clean temp TypeScript consumer can type-check the canonical happy path
- package docs do not point users back at obsolete private or scoped package
  names unless intentionally describing a separate lane

**Closeout note**

This phase is complete only once the release-proof lane is scripted rather than
remembered. The package must have a mechanical verifier and a release-gate path
that can rebuild, prepare, pack, and prove the artifact before any real
`npm publish` step is attempted.

### Phase 1: Runtime-Scoped Callback Capability Cleanup

**Status:** Completed

**Why this phase exists**

The crate still contains older callback-management patterns outside the new
host-computed substrate, especially observation/effect callback registration
that can drift into ambient global-id folklore if it is not deliberately
replaced or contained.

**Deliver**

- runtime-scoped callback capability registries for watch/effect and compute
  families
- explicit generation/disposal semantics for every callback capability family
- lifecycle inspection surfaces for callback capability state
- removal or containment of ambient global callback-id maps that are not tied to
  runtime ownership

**Must prove**

- watch/effect callback lifecycle follows the same ownership discipline as
  compute callbacks
- stale callback ids cannot silently bind to newly registered capabilities
- callback capability state is inspectable through runtime-owned artifacts
  rather than ambient globals

### Phase 2: React And App-First Boundary Cleanup

**Status:** Completed

**Why this phase exists**

The React adapter and app-first shell are still where convenience-era shortcuts
can sneak semantics sideways: monkey-patching shared facades, depending on
compatibility reads for normal usage, or quietly rewriting behavior once a
store is installed.

**Deliver**

- React store integration that does not monkey-patch shared `Signals` facade
  methods
- an app-first read path that does not require compatibility-lane reads for
  ordinary callback-first consumption
- explicit ownership and lifecycle boundaries between React consumer state and
  runtime truth

**Must prove**

- installing a React store does not rewrite the behavior of unrelated consumers
  sharing the same `Signals` instance
- normal app-first reads remain correct without fallback through
  compatibility-only APIs
- React remains a pure consumer of runtime truth rather than a second cache or
  orchestration layer

### Phase 3: Diagnostics Product Closeout

**Status:** Completed

**Why this phase exists**

Diagnostics exist, but they still need to feel like a first-class product job
for normal users rather than an internal capability users can discover only if
they already know what to ask for.

**Deliver**

- one clear diagnostics story for:
  - why did this recompute
  - why did this not recompute
  - why did this callback fail
  - why did the dependency set rewire
  - what just delivered to observers/effects
  - what is the current runtime cost posture
- docs and examples that teach diagnostics from the product surface
- public diagnostics typing that matches the real callback-origin runtime model
- elimination of critical product-boundary diagnostics surfaces that rely on
  opaque `JsValue`/`unknown` blobs where typed callback artifacts are required

**Must prove**

- callback-origin diagnostics are as legible as expression-origin diagnostics
- the public docs include both simple and richer diagnostics examples
- React-facing users can discover diagnostics without dropping into raw runtime
  internals
- typed consumers can inspect callback diagnostics, latest-flow summaries, and
  performance artifacts without reconstructing semantics from untyped blobs

### Phase 4: Authoring Surface Symmetry

**Status:** Completed

**Why this phase exists**

`computed(() => ...)` now reads like normal code. `input(...)` and `output(...)`
still carry too much of the older registration-first shape. That leaves the
crate feeling assembled from different eras instead of designed as one
language.

**Deliver**

- one coherent authoring family for source, derived, and exposed state
- simple forms and richer forms that share the same grammar
- ids treated as metadata rather than as the dominant first impression of the
  API
- explicit positioning of spec/compat forms as advanced lanes

**Target shape**

```ts
const count = signals.input(1, { id: "count" });
const doubled = signals.computed(() => count() * 2, { id: "doubled" });
const label = signals.output(() => `${doubled()}`, { id: "label" });
```

Exact spellings may evolve, but the authoring surface must read with this level
of symmetry and ease.

**Must prove**

- new users do not have to learn three unrelated call styles for `input`,
  `computed`, and `output`
- the most attractive-looking API forms are the correct normal usage forms
- callback-first forms and richer option-bearing forms remain mechanically
  consistent across the family

### Phase 5: Type, Capability, And Misuse Hardening

**Status:** Completed

**Why this phase exists**

The runtime is already enforcing a lot, but the TypeScript/public package
surface still allows too much confusion about handle categories, runtime
ownership, and callback capability identity.

**Deliver**

- stronger branded/opaque handle categories
- stricter runtime/store ownership cues at the package surface
- callback capability vocabulary that reads as intentional product semantics
- tighter separation between guided product surfaces and raw/compatibility
  surfaces

**Must prove**

- writable inputs cannot be confused with computed or output handles
- raw wasm handles do not silently substitute for product-facing callable
  handles
- callback/stale capability misuse becomes materially harder to express
- cross-runtime/store confusion is visibly and mechanically constrained

### Phase 6: Documentation, Examples, And Publish-Ready Product Identity

**Status:** Completed

**Why this phase exists**

The package should not merely be technically correct. It should be teachable,
demoable, and publication-grade.

**Deliver**

- docs organized around real usage, not internal taxonomy alone
- simple and complex examples for each major public surface
- diagnostics examples that feel like real debugging workflows
- README/examples that reinforce one obvious center of gravity
- release notes / deprecation notes where needed for broken or obsolete
  published artifacts

**Must prove**

- the docs teach the public package the way a new React/TypeScript user will
  actually consume it
- examples cover:
  - basic state and derivation
  - callback rewiring
  - diagnostics
  - React store usage
  - advanced/spec compatibility lanes where relevant
- a release candidate can be installed, imported, and used from another app
  without tribal knowledge

## Explicit Deferrals

- a general host-capability resource lane for callback purity beyond captured
  signal reads
- portable serialization of JavaScript callback bodies
- worker migration of live callback functions
- async/resource callbacks
- Suspense or router integration
- advanced compiler transforms that infer AST recipes from callback source

Those are separate products or optimizations. They may not block callback
computed nodes from becoming honest runtime-managed derived state.

## Milestone Done When

This milestone is done only when ordinary TypeScript can write:

```ts
const doubleCount: Signal<number> = computed(() => count() * 2);
```

and the result is a real WORTH runtime computed node with:

- dynamic dependency capture
- runtime-owned invalidation
- rollback-safe commits
- committed observation delivery
- React subscription parity
- diagnostics parity
- callback lifecycle and callback-id/generation semantics that are framework-
  owned and observable instead of ambient JS folklore
- callback-bearing history, replay, export, and import handled through honest
  artifact families:
  - exact same-process restore tokens for live callback-bearing restore
  - explicit callback-unavailable portable transport and import artifacts when
    callback bodies are not present
- bounded and attributable callback costs

At that point, serialized AST recipes become what they should have been all
along: a powerful advanced lane, not the tax every app developer pays for basic
derived state.

## Closeout Record

This milestone is closed.

It completed the product and runtime work needed to make callback-first
`computed(() => ...)` a real `worth-signals-wasm` authoring lane rather than a
JavaScript-local convenience cache.

What is now true:

- callback-first `computed(() => ...)` lowers into runtime-owned derived nodes
  with dynamic dependency capture and rewiring
- watch/effect and compute callback lifecycle is runtime-scoped,
  generation-aware, and mechanically denied when stale
- React consumes runtime truth without monkey-patching shared `Signals`
  behavior or depending on compatibility reads for ordinary usage
- diagnostics, history, replay, and merge surfaces are exposed as typed product
  artifacts rather than blob-shaped boundary folklore
- the npm artifact, runtime tests, and release-proof path certify the public
  package that users actually install

The trust-boundary split is also now explicit:

- same-process exact restore of callback-bearing runtime state uses
  runtime-local restore tokens and exact restore artifacts
- portable export/import and cross-host transport do not pretend JavaScript
  callback bodies are portable runtime data; they surface explicit typed
  callback-unavailable outcomes instead

This milestone intentionally did not solve general host capability. For this
completed scope, callback purity means captured signal reads only. Any future
host-capability lane must be specified as a separately typed resource family
with its own lifecycle, invalidation, replay/restore, diagnostics, and
measurement contracts. That follow-on work now lives in
[wasm_product_roadmap.md](./wasm_product_roadmap.md).

## Crate Maturity Done When

`worth-signals-wasm` is mature only when, after the milestone closes, ordinary
React and TypeScript users can learn one obvious callback-first product surface
and are mechanically steered away from the main misuse classes this document
identified.

In practice that means:

- the normal path is callback-first, typed, and easy to inspect
- `input`, `computed`, and `output` read as one coherent authoring language,
  with simple forms for quick usage and richer forms for explicit ids and
  options
- ordinary app code can be organized as controller factories that return signal
  handles and compose directly through those handles
- raw and compatibility lanes are explicit expert surfaces rather than ambient
  alternative truths
- callback ids, callback functions, and callback lifecycle feel productized
  instead of incidental
- diagnostics are a first-class product capability rather than a hidden expert
  affordance
- the common misuse classes are unrepresentable or materially constrained:
  - mixing runtimes or stores accidentally
  - mutating computed or output handles
  - forging signal-like objects structurally
  - confusing raw wasm handles with product-facing callable handles
  - treating callback-backed derived state as React-local selector logic
- docs, examples, and package exports all reinforce the same center of gravity

That is the actual finish line for the crate, even though the semantic callback
milestone should close first.

## Practical Finish Checklist

The crate should not be called finished until all of the following are true at
the same time:

- callback-computed runtime truth is stable and already certified
- the public npm artifact is entrypoint-complete and temp-consumer-proven
- root and React import surfaces both work at runtime and in TypeScript
- diagnostics are documented and demonstrated as a first-class product surface
- every major public concept has:
  - a simple example
  - a more realistic example
- `input`, `computed`, and `output` share one guided authoring language
- raw/spec/compatibility lanes are still available but no longer define the
  product's first impression
- the remaining misuse classes are mechanically constrained enough that the
  crate can be recommended without a giant footnote
