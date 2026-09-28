# worth-signals-wasm Composition API Plan

> **Status:** Closed engineering spec
>
> **Roadmap parent:** [wasm_product_roadmap.md](./wasm_product_roadmap.md)
>
> **Web runtime parent:** [web_runtime_spec.md](./web_runtime_spec.md)
>
> **Callback-computed parent:** [host_callback_computed_spec.md](./host_callback_computed_spec.md)
>
> **Core vision:** [plans/worth_signal/worth_signal_vision.md](../WORTH_signal/WORTH_signal_vision.md)
>
> **Core test requirements:** [plans/worth_signal/test-requirements.md](../WORTH_signal/test-requirements.md)
>
> **Primary architectural driver:** make controller-first signal authoring and
> explicit graph publication first-class product surfaces so app code can
> compose feature logic as ordinary functions without falling back to giant
> graph registries, string lookups, or a second local composition engine.

## Goal

Make `worth-signals-wasm` support a real controller-first composition API where
application code authors inputs and computed nodes as ordinary code, composes
feature controllers through typed signal handles, and publishes an explicit
graph boundary through a real `signals.graph(...)` product API.

The target product shape is:

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

This milestone is not complete if the target shape lives only in docs or
examples. It must become a real exported, typed, tested product surface.

## Why This Spec Exists

`worth-signals-wasm` is now in a much better place than the earlier callback
milestone:

- `createSignals()` is real
- `input`, `computed`, and `output` are real product surfaces
- callback-first `computed(() => ...)` is runtime-owned truth
- diagnostics, history, replay, and host capability are real package lanes

But the product still has one major gap between the documented direction and
the shipped API:

- signal authoring is code-first
- graph composition is not

Today, app code can author signals naturally, but it still does not have a
first-class product story for:

- grouping authored handles into a named graph boundary
- composing feature-level controllers as ordinary functions
- publishing outputs from those controllers without falling back to graph JSON
  or string-id registries
- carrying graph identity through diagnostics, history, and compatibility
  surfaces honestly

Without this milestone, the likely failure mode is not runtime breakage. It is
product drift:

- app authors keep writing giant node registries because only those feel like
  "real graphs"
- controller-style composition remains an undocumented convention instead of a
  supported product lane
- forms and resources build their own feature-level composition helpers before
  the package provides the right substrate
- `signals.graph(...)` stays spec theater instead of a first-class API
- the crate teaches better signal authoring than graph authoring, leaving the
  overall programming model only half-modernized

This spec exists to close that gap.

## Hard Part

The hard part is not adding one helper called `graph`.

The hard part is freezing one honest relationship among four distinct things
that naive designs blur together:

- signal authoring inside controllers
- graph publication as an explicit public boundary
- output authority as a real runtime/product concept
- compatibility/export/history lanes that still need graph-shaped artifacts

The design fails if:

- `signals.graph(...)` is only a JavaScript bagging helper with no typed or
  diagnostics-visible meaning
- publication requires string lookups even though controllers already hold
  typed signal handles
- outputs remain semantically second-class and publication silently smuggles
  computed handles across an authority boundary without recording that choice
- controller composition invents a second local graph model that drifts from
  runtime truth
- compatibility/import/export lanes stop matching the composed code-first lane
- forms/resources become the first place where feature-level composition is
  truly solved

This milestone therefore has to make composition more natural while making the
publication boundary more explicit and more honest.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects here is solving the real structural gap
  instead of polishing around it. This spec therefore treats graph publication
  and controller composition as product architecture, not as optional sugar.
- `arch_laws.md`
  The most important laws here are 1, 7, 16, 17, 21, 33, 34, 40, and 41.
  Controller composition must not collapse runtime authority boundaries,
  graph publication must produce a self-describing boundary artifact, and
  lowered/publication forms must remain proof-bearing rather than ambient bags.
- `perf_laws.md`
  The most important thing it protects is that the composition API must not
  hide broad graph scans, broad output synthesis, or repeated rediscovery of
  handles and outputs at publication time. Publication must scale with the
  declared output set, not with total runtime state.
- `domain_laws.md`
  The most important thing it protects is single responsibility. Controller
  authoring, graph publication, compatibility graph definitions, diagnostics,
  and export/history concerns must live in separate responsibility spaces.
- `worth_signal_vision.md`
  The most important thing it protects is that `worth-signal` owns derived
  truth while higher layers author and consume it. This spec must preserve that
  split: composition is a wasm/package product surface, not a new truth engine.
- `worth_signal_temporal_async_roadmap.md`
  The most important thing it protects is sequencing discipline. Later layers
  must consume runtime semantics rather than redefining them. This composition
  milestone must therefore land before forms/resources build higher-level
  feature composition on top of the wrong public model.
- `test-requirements.md`
  The most important thing it protects is proof-grade equivalence. The
  controller-first composition lane must certify the same committed truth,
  diagnostics, and replay/restore behavior as the already-shipped flat signal
  lane.
- `dx_plan.md`
  The most important thing it protects is product inevitability. The primary
  path should feel like the right path, and major usage modes should not remain
  fragmented, registration-heavy, or compatibility-shaped by accident.
- `wasm_product_roadmap.md`
  The most important thing it protects is roadmap sequencing for real package
  products. This composition milestone belongs before forms and API resources
  because those products need a controller-first publication model rather than
  inventing one privately.
- `web_runtime_spec.md`
  The most important thing it protects is the app-first runtime surface. This
  spec extends that app-first direction from signal authoring into
  feature-level graph composition and publication.
- `host_callback_computed_spec.md`
  The most important thing it protects is the callback-first product direction
  and the crate-maturity requirement for controller-first authoring with
  explicit graph publication. This spec turns that recorded target into a real
  executable milestone.

## Adversarial Constraint

This milestone must survive the following hostile condition:

> A long-lived TypeScript application with multiple feature controllers,
> callback-backed computed nodes, explicit outputs, branch/restore activity,
> diagnostics inspection, and compatibility/export reads must converge to the
> same committed values, published output authorities, graph identity,
> diagnostics summaries, and replay/restore-visible artifacts whether the same
> feature logic is authored as one flat runtime script or as composed
> controller factories published through `signals.graph(...)`.

Concretely, the design must remain correct when:

- multiple controllers share the same `SignalNamespace`
- one controller consumes handles returned by another controller
- publication outputs include both raw output handles and readable computed
  handles that need publication synthesis
- graph publication happens after many signals already exist
- branch restore and replay occur after controller-composed graphs have been
  published
- diagnostics and history are read before and after publication
- compatibility/export lanes need to describe the published graph boundary
- feature authors refactor one flat controller into multiple smaller
  controllers without changing runtime truth

If any supported path produces the same final values with a different published
graph boundary, different output authorities, or different diagnostics story,
the milestone has failed.

## Current-State Assessment

The package is not starting from zero.

### What already exists

The lower-level substrate needed for this product shape is already real:

- code-first signal authoring through `createSignals()`
- callback-first `computed(() => ...)`
- callable typed handles for `input`, `computed`, and `output`
- runtime-owned observation, diagnostics, history, replay, and host capability
  surfaces
- product/package entrypoints that are now real npm artifacts rather than
  semi-private bindings

In other words: the signal authoring half of the story is already there.

### What does not exist yet

The actual composition/publication layer is still missing:

- there is no real `signals.graph(...)` export today
- there is no public `SignalNamespace` product vocabulary
- there is no typed graph publication artifact on the package surface
- there is no explicit rule for how computed handles become published outputs
- there is no graph-level diagnostics/history/compatibility surface at the
  package boundary

That means the current package can author signals naturally, but it cannot yet
publish controller-composed feature graphs as a first-class product concept.

### Why this is a real gap instead of nice-to-have polish

Without this layer, the crate still teaches two different mental models:

- "write signals like ordinary code"
- "but think about graphs like object registries or compatibility definitions"

That split is exactly what the composition API needs to remove.

## Product Decision Lock

- controller-first authoring is the intended primary app-facing composition
  model
- `SignalNamespace` or its final equivalent is a real exported product type,
  not just a name used in docs
- `signals.graph(name, { outputs })` is a real exported product API, not a
  helper hidden in examples
- controller factories may author:
  - inputs
  - computeds
  - outputs
  and return typed handles
- graph publication accepts typed readable handles directly; it must not
  require string lookup to publish outputs
- graph publication may synthesize output authorities from readable computed or
  input handles when necessary, but that synthesis must be explicit in typed
  artifacts and diagnostics
- graph-shaped compatibility/import/export definitions remain supported, but
  they are no longer the primary app-authoring story
- forms, resources, and later app products must consume this composition API
  rather than inventing their own feature composition model

Normative consequence:

- any implementation that leaves `signals.graph(...)` as docs-only is out of
  spec
- any implementation that requires publication by string id when a typed handle
  already exists is out of spec
- any implementation that treats graph publication as pure local JS metadata
  with no diagnostics/history/export meaning is out of spec
- any implementation that makes forms/resources the first real owners of
  controller composition is out of spec

## Public API Model

### Primary authoring model

The package should converge on two aligned layers:

- **controller authoring**
  - ordinary functions over a `SignalNamespace`
  - return typed handles
- **graph publication**
  - one explicit `signals.graph(...)` boundary that declares which handles are
    exposed as named outputs

The controller layer is ordinary code.
The graph layer is the explicit public/projection boundary.

### Required surface categories

At minimum, the composition API must define real product vocabulary for:

- `SignalNamespace`
- `PublishedSignalGraph` or equivalent graph publication artifact
- graph output mapping input type
- graph diagnostics/history/compatibility surface if it differs from the base
  runtime object

The exact names may still evolve, but the semantic categories are locked.

### Exact target example

The exact target shape this spec is trying to make real is:

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

### Output publication rule

The `outputs` object in `signals.graph(...)` must accept typed readable handles
instead of requiring pre-authored output nodes.

That means the composition surface must support publication of:

- existing `OutputSignalHandle`
- `ComputedSignalHandle`
- optionally `InputSignalHandle` where publication of source state is desired

If publication needs to synthesize output authorities under the hood, that
synthesis must be runtime- and diagnostics-visible rather than a hidden local
alias trick.

### Graph publication rule

`signals.graph(...)` is not allowed to be just a bag of handles.

It must be a real boundary artifact with at least:

- graph identity
- canonical named outputs
- publication-time validation
- diagnostics/history/compatibility hooks where relevant

The graph publication artifact may remain package-owned rather than core-owned,
but it must still be self-describing enough that the rest of the package can
treat it as a real product object.

## Required Architecture Changes

### 1. Add a dedicated composition/publication responsibility space

Do not keep the composition API as ad hoc helper code inside
[signals.js](../../crates/worth-signal-wasm/package-src/product/signals.ts).

Expected package-level responsibility split:

- controller-facing authoring namespace
- graph publication / output synthesis
- graph publication types
- graph publication diagnostics/history/compatibility glue

Likely file families:

- `package-src/product/graphs.ts`
- `package/types/graph_surface.d.ts`
- supporting product/type modules as needed

The exact filenames can vary, but graph publication must be a named
responsibility, not an extra method hidden in the general signals file.

### 2. Preserve one facade, not one god file

The package may still expose one public facade, but:

- authoring
- handles
- history
- diagnostics
- transactions
- host capabilities
- graph publication

must remain decomposed internally by reason-to-change.

### 3. Publication must lower once

Graph publication must follow the same structural pattern as the rest of the
crate:

- authoring intent
- validated publication request
- lowered publication plan
- committed publication artifact
- derived diagnostics/history/export summaries

The implementation must not repeatedly rediscover output handles, synthesize
publication identities differently across APIs, or let each consumer interpret
publication metadata on its own.

### 4. Compatibility remains explicit

Object-graph definitions still matter for:

- compatibility surfaces
- import/export
- portable definitions

But they must remain the advanced or compatibility lane.

This milestone must not delete that lane.
It must subordinate it.

## Phases

### Phase 1: Composition Vocabulary And Boundary Lock

Purpose:

- turn the desired controller-first composition model into a real exported
  package vocabulary
- prevent `signals.graph(...)` from becoming one more convenience helper with
  vague semantics

Entry criteria:

- callback-computed, diagnostics, and host-capability closeout already shipped

Required work:

- define the public composition vocabulary:
  - `SignalNamespace`
  - graph publication request types
  - graph publication artifact type
- decide the exact allowed `outputs` input categories
- decide the exact publication return shape
- define publication-time validation errors and rejection classes
- define graph identity and output identity rules

Required outputs:

- exported composition types in the package surface
- one canonical docs/example shape for controller-first authoring
- compile-time/type-level distinction between signal authoring and graph
  publication

Exit criteria:

- controller-first composition is a named product concept
- `signals.graph(...)` has a typed contract instead of a vague future note

Must not start before:

- none; this is the required first phase

### Phase 2: Graph Publication Primitive And Output Synthesis

Purpose:

- implement the real `signals.graph(...)` API
- make publication of readable handles honest and deterministic

Entry criteria:

- Phase 1 complete

Required work:

- add `signals.graph(name, { outputs })` to the product JS surface
- add matching TypeScript declarations
- validate graph ids, output names, and output handle ownership
- allow publication from readable handles
- synthesize output authorities where publication requires them
- make that synthesis deterministic and inspectable

Required outputs:

- runtime-usable graph publication API
- typed output-publication rules
- deterministic output synthesis path

Exit criteria:

- ordinary controller code can publish a graph without string lookup
- publication from readable handles is real and tested
- hidden output synthesis, if used, is deterministic and diagnostics-visible

Must not start before:

- Phase 1 complete

### Phase 3: Controller Composition And Graph Artifact Truth

Purpose:

- make composed controller graphs first-class product artifacts instead of
  local helper conventions

Entry criteria:

- Phase 2 complete

Required work:

- define graph artifact identity and accessors
- expose published outputs through the graph artifact
- define how published graphs interact with:
  - diagnostics
  - history
  - specialist/graph summary surfaces
- make controller composition across multiple factories a supported test path
- define and enforce same-runtime ownership rules across published graphs

Required outputs:

- `PublishedSignalGraph` or equivalent artifact
- graph-aware diagnostics/history integration where relevant
- controller composition tests using multiple factories

Exit criteria:

- composed controllers produce a real graph artifact, not just a bag of
  returned handles
- graph identity is inspectable
- cross-controller composition stays same-runtime honest

Must not start before:

- Phase 2 complete

### Phase 4: Compatibility, Export, And Diagnostics Alignment

Purpose:

- keep the new composition lane aligned with the older graph-shaped lanes
- prevent publication metadata from becoming docs-only meaning

Entry criteria:

- Phase 3 complete

Required work:

- define how published graphs show up in compatibility/export surfaces
- define how publication affects diagnostics summaries and graph summaries
- define how graph identity survives history/replay/restore reads where
  relevant
- ensure the old graph/object compatibility lane and new composition lane agree
  on canonical output truth

Required outputs:

- compatibility/export alignment rules
- graph publication diagnostics surfaces
- history/restore/replay posture for published graph identity

Exit criteria:

- graph publication is not invisible to the rest of the product
- composition and compatibility lanes agree on committed output truth

Must not start before:

- Phase 3 complete

### Phase 5: Documentation, Examples, And Certification Closeout

Purpose:

- make the controller-first composition API teachable and regression-protected

Entry criteria:

- Phase 4 complete

Required work:

- update docs so controller-first composition is taught as the primary app
  story
- add both simple and more realistic examples
- add product tests for:
  - controller composition
  - publication from computed handles
  - diagnostics/history parity
  - compatibility/export parity
- add temp-consumer or package-level tests where appropriate

Required outputs:

- docs and examples for controller-first composition
- product/runtime/package tests proving the lane

Exit criteria:

- a new app author can discover the composition API from docs alone
- the target code shape is regression-protected

Must not start before:

- Phase 4 complete

## Must Ship

- real exported composition vocabulary
- real `signals.graph(...)` package API
- publication from typed readable handles
- deterministic output synthesis where needed
- graph publication artifact with identity
- graph-aware diagnostics/history/compatibility behavior
- controller-first docs and examples
- certification-grade tests for controller composition equivalence

## Must Preserve

- runtime truth remains owned by the existing runtime substrate
- controller composition does not create a second local store or graph engine
- output publication remains an explicit boundary rather than ambient handle
  aliasing
- compatibility/import/export graph definitions remain available
- diagnostics richness remains derived and must not change operational truth
- publication work remains bounded by declared outputs, not total graph state

## Required Named Test Families

- `The Controller Composition And Flat Runtime Equivalence Test`
  Proves that one flat authored runtime and one controller-composed runtime
  converge to the same committed outputs and diagnostics.
- `The Graph Publication Output Synthesis Test`
  Proves that publishing computed handles yields deterministic output authority
  artifacts and committed truth.
- `The Same-Runtime Controller Ownership Test`
  Proves that graph publication rejects foreign-runtime handles and preserves
  runtime ownership rules across controllers.
- `The Composition Diagnostics And History Parity Test`
  Proves that publication-aware diagnostics/history surfaces stay coherent with
  the base runtime truth.
- `The Composition And Compatibility Graph Equivalence Test`
  Proves that the controller-first composition lane and the graph-shaped
  compatibility lane agree on canonical published output truth.

## Acceptance Evidence

This milestone is complete only when all of the following are true:

- the exact target code shape in this spec is real, exported, and typed
- controller composition through returned handles is a supported product path
- `signals.graph(...)` publishes readable handles without string-id fallback
- published graph identity is visible enough that diagnostics/history and
  compatibility surfaces can explain it honestly
- composition does not create a second truth model beside the runtime
- the named test families above pass

## Closeout Status

This milestone is now closed.

The shipped package surface includes:

- exported composition vocabulary such as `SignalNamespace`
- real `signals.graph(...)` publication
- deterministic synthesized output authorities for published readable handles
- graph-scoped diagnostics/history inspection
- graph-scoped compatibility export
- controller-first docs and examples in the main package references
- package-level clean-consumer proof for controller composition

Later work may deepen higher-level products built on this surface, but the
composition API itself is no longer a planned gap.

## Sequencing Notes

This milestone belongs in the wasm roadmap before forms and API resources.

Why:

- forms want controller-first authoring naturally
- workflow/resource products want explicit graph publication naturally
- if they land first, they will be forced to invent their own composition
  model or keep leaning on older graph-object declarations

So the intended order is:

1. host capability closeout
2. composition API and graph publication
3. forms product surface
4. API surface

That order keeps the next application-facing products from becoming the first
owners of feature composition.


