# worth-signals-wasm Controller Scope And Graph Lifecycle Plan

> **Status:** Planned engineering spec
>
> **Roadmap parent:** [wasm_product_roadmap.md](./wasm_product_roadmap.md)
>
> **Web runtime parent:** [web_runtime_spec.md](./web_runtime_spec.md)
>
> **Callback-computed parent:** [host_callback_computed_spec.md](./host_callback_computed_spec.md)
>
> **Composition prerequisite:** [composition-api-plan.md](./composition-api-plan.md)
>
> **Host capability prerequisite:** [host_capability_spec.md](./host_capability_spec.md)
>
> **Core vision:** [plans/worth_signal/worth_signal_vision.md](../WORTH_signal/WORTH_signal_vision.md)
>
> **Core test requirements:** [plans/worth_signal/test-requirements.md](../WORTH_signal/test-requirements.md)
>
> **Primary architectural driver:** make controller-authored graphs mature
> enough for serious application architecture by adding scoped controller
> identity, graph-owned lifecycle, explicit public input/output contracts,
> graph-owned operations, richer contract semantics, and graph-native
> historical/export truth before forms and resource products are allowed to
> build on the current wasm surface.

## Goal

Make `worth-signals-wasm` support a mature application-graph surface where:

- controller-local names are actually local
- repeated feature instances do not collide
- graph construction owns lifecycle and public contract declaration
- published graph inputs and outputs are explicit product artifacts
- controller contracts distinguish public surface from internal staging
- public graph contracts are operational, not merely descriptive
- graph diagnostics/history/export surfaces explain the public contract directly
- the `input` / `computed` / `output` authoring language converges further
- later forms and resource products can consume one scoped composition model
  instead of inventing their own

The target product direction is no longer merely:

```ts
const signals = createSignals();
const editSession = createEditSessionController(signals);
const workflow = createWorkflowController(signals, editSession);

const itemDetailGraph = signals.graph("itemDetail", {
  outputs: {
    effectiveItemData: editSession.outputs.effectiveItemData,
    dirtyState: editSession.outputs.dirtyState,
    submitReadiness: workflow.outputs.submitReadiness,
  },
});
```

The target product direction for significant application code is:

```ts
export const itemDetailGraph = createSignals().graph("itemDetail", (graph) => {
  const editSession = createEditSessionController(graph.scope("editSession"));
  const workflow = createWorkflowController(graph.scope("workflow"), editSession);

  return graph.expose({
    inputs: {
      ...editSession.inputs,
    },
    outputs: {
      ...editSession.outputs,
      ...workflow.outputs,
    },
  });
});
```

The exact final spelling may still evolve, but the required semantics are
locked:

- graph construction is the authoring boundary
- controller scopes are first-class
- graph-owned lifecycle replaces runtime-global id folklore
- graph `inputs` and `outputs` are explicit public contract surfaces
- controller contracts can mark internal structure intentionally
- graph contract consumers can operate through graph-native surfaces instead of
  dropping immediately to raw handle mechanics
- repeated and dynamic instance identity is explicit enough for future forms and
  resources

## Why This Milestone Exists

The shipped composition milestone was real and worthwhile.

It gave the package:

- `SignalNamespace`
- controller-first authoring
- real `signals.graph(...)`
- deterministic output publication
- graph-scoped diagnostics/history/compatibility surfaces

But it intentionally stopped before solving the next structural problem:

- signal ids are still effectively runtime-global
- controller-local names are only local by naming discipline
- repeated feature instances can collide unless users prefix ids manually
- graphs publish outputs, but they do not yet own construction scope or public
  input contracts
- later forms/resources would have to invent scope, lifecycle, and contract
  discipline privately if this milestone does not land first

That means the current composition surface is good enough for:

- one feature graph
- one runtime
- one instance of a controller family
- disciplined authors who are willing to prefix ids by hand

It is not mature enough for:

- repeated feature instances in one runtime
- side-by-side editors
- modal + page copies of the same feature
- list rows with per-row controllers
- forms and resources that need graph-owned public input/output contracts

This milestone exists to close that gap before higher product layers hard-code
their own solutions.

This spec is intentionally more ambitious than the first closed version of the
milestone.

If we stop at scoped ids plus graph publication, we still leave too much of the
real application boundary unowned:

- controllers still improvise their public/internal contract shape
- graph inputs are visible but not yet a graph-native operational surface
- repeated and dynamic instances remain only partially specified
- diagnostics can describe nodes without fully describing boundary contracts
- export/import remains graph-filtered runtime truth rather than graph-native
  boundary truth
- forms and resources would still need to invent some of their own contract and
  lifecycle mechanics

This revised milestone treats that remaining boundary work as belonging here,
not as accidental follow-on polish.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects here is solving the hard structural
  problem before the attractive product layers. This spec therefore treats
  scope, ownership, and public graph contracts as foundation work for
  forms/resources, not as post-polish.
- `arch_laws.md`
  The most important laws here are 7, 16, 21, 33, 34, 40, and 41. Graph
  construction must produce self-describing boundary artifacts, lifecycle must
  be framework-owned, and distinct meanings like controller scope, graph
  identity, local ids, and public contract names must stay distinct.
- `perf_laws.md`
  The most important thing it protects is boundary honesty. Scope resolution,
  graph exposure, and public-contract reads must not conceal broad scans,
  global registries, or rediscovery of graph topology that looks local at the
  API.
- `domain_laws.md`
  The most important thing it protects is responsibility-shaped structure.
  Scoped authoring, graph construction, public contract exposure, diagnostics
  alignment, and compatibility/export alignment must remain decomposed by
  reason to change.
- `worth_signal_vision.md`
  The most important thing it protects is that `worth-signal` remains derived
  computation infrastructure rather than truth storage. This milestone must
  strengthen product composition without creating a second graph engine beside
  the runtime.
- `wasm_product_roadmap.md`
  The most important thing it protects is sequencing. Forms and resources must
  inherit one scoped composition/lifecycle model rather than becoming its first
  real owners.
- `test-requirements.md`
  The most important thing it protects is hostile equivalence and proof-grade
  closure. The scoped graph model must prove the same committed truth,
  diagnostics, and replay/restore posture as the current flat/manual-scope
  lanes.
- `dx_plan.md`
  The most important thing it protects is that the primary surface must feel
  inevitable and teachable. A mature graph authoring model must not depend on
  tribal scope-prefix conventions forever.
- `web_runtime_spec.md`
  The most important thing it protects is the app-first product boundary. This
  milestone deepens that promise from signal primitives into graph-owned
  application architecture.
- `host_callback_computed_spec.md`
  The most important thing it protects is crate maturity beyond callback
  support alone. This milestone is the ownership/lifecycle layer that makes
  callback-first authoring actually scalable.
- `composition-api-plan.md`
  The most important thing it protects is that composition/publication are
  already real. This milestone must extend that surface structurally rather
  than replacing it with another docs-only dream.

## Architectural Extension Lock

This milestone now explicitly includes all of the following architectural
extension tracks:

1. **Controller contract standardization**
   - controllers must have a principled way to distinguish public inputs,
     public outputs, and internal signals
2. **Graphs own lifecycle completely**
   - graph boundaries must become the owner of authoring, contract exposure,
     operational input mutation, transactionality, and historical boundary
     semantics
3. **Public contract richer than `inputs` and `outputs` alone**
   - the architecture must preserve room for internal state and future contract
     families such as forms/resources without collapsing everything into one bag
4. **Public input ergonomics become operational**
   - graph inputs must be usable through graph-native mutation and transaction
     surfaces, not only by grabbing raw handles
5. **Repeated and dynamic instance architecture**
   - repeated controller families, nested scopes, and future dynamic subgraphs
     must have a real identity story
6. **Authoring grammar convergence**
   - `input`, `computed`, and `output` must converge toward one coherent
     language rather than three eras of API design
7. **Contract-level diagnostics and dependency introspection**
   - the graph boundary must be able to explain public inputs, public outputs,
     dependency relationships, and contract-local version/change posture
8. **Graph-native export/import**
   - graphs must be able to export and restore their own boundary truth as
     first-class artifacts instead of only consuming runtime-wide filtered
     exports

This milestone now also explicitly includes these further refinements:

9. **Controller artifacts, not just controller object shapes**
   - controller contracts should become branded, package-understood artifacts
     rather than remaining only a conventionally nice return shape
10. **Public contract vs operational contract distinction**
   - the architecture must distinguish what a graph exposes publicly from what
     it admits as graph-native operations
11. **Typed scope identity**
   - scope and instance identity must become explicit semantic values rather
     than stringly ambient conventions hidden behind ergonomic helpers
12. **Contract delta and diff artifacts**
   - graph contracts should be able to explain how and whether the public
     contract changed over time
13. **Unified graph mutation envelope**
   - ergonomic graph input helpers should lower to one canonical graph-mutation
     contract rather than a pile of unrelated mutator methods
14. **Authority classes for public inputs**
   - the milestone must preserve room for read-only, writable, imported, or
     otherwise differently-authorized public input lanes
15. **Canonical naming law**
   - local ids, runtime ids, public contract names, and exported/imported names
     must have one explicit relationship rather than being reinterpreted phase
     by phase
16. **Negative ergonomics protection**
   - the architecture should not only optimize the happy path; it should also
     make the wrong path feel explicit, awkward, or denied on purpose

## Adversarial Constraint

This milestone must survive the following hostile condition:

> A long-lived TypeScript application with many repeated feature instances,
> controller factories, callback-backed computed state, branch/restore
> activity, diagnostics inspection, explicit public inputs and outputs, and
> later forms/resources layered on top must converge to the same committed
> truth, public graph contract, diagnostics story, and replay/restore-visible
> artifacts whether the feature is authored as one flat scoped script, as
> multiple controller factories with explicit scopes, or as graph-owned scoped
> construction through one graph boundary.

Concretely, the design must remain correct when:

- the same controller family is instantiated many times in one runtime
- nested controllers and sibling controllers use overlapping local names
- one controller consumes another controller's outputs
- public graph inputs and outputs are exposed while internal staging signals
  remain private
- branch restore returns to an earlier graph-owned contract state
- compatibility/export surfaces need to describe graph-local identity instead
  of runtime-global folklore
- forms and resources later consume the same graph boundary without inventing a
  second scope model

If any supported path produces the right final values with:

- colliding signal ids
- unstable graph-local identity
- ambiguous public input/output naming
- different diagnostics naming for equivalent histories
- or hidden runtime-global behavior behind graph-local-looking API

then the milestone has failed.

## Current-State Assessment

The package is not starting from zero.

### What already exists

The current wasm product already has:

- callback-first signal authoring
- controller-first composition
- `SignalNamespace`
- real `signals.graph(...)`
- deterministic output publication
- graph-scoped diagnostics/history/compatibility surfaces

That is enough to make graph authoring feel real.

### What does not exist yet

The missing structural layer is ownership:

- no first-class controller scope model
- no graph-owned construction boundary
- no graph-owned public input contract
- no sealed graph lifecycle that makes local names truly local
- no official significant-code answer better than manual string prefixing

### Why this is a real gap instead of aesthetic polish

Without this milestone:

- serious codebases must hand-roll scope strings forever
- graph construction remains partially incidental instead of authoritative
- public graph inputs remain implied rather than explicit
- forms/resources will either build on a collision-prone substrate or invent
  their own ownership model

## Product Decision Lock

- controller-local names must become structurally local, not merely socially
  local
- graph construction must become the lifecycle owner for scoped controller
  authoring
- the significant-code path must not require raw string prefixing forever
- graph `inputs` and graph `outputs` are distinct public contract categories
- controller factories may still author ordinary code, but they should do so
  through a graph-owned scope surface rather than the whole runtime namespace
- internal controller signals and public graph contract signals must be allowed
  to differ explicitly
- controllers should converge on one principled contract shape instead of each
  feature inventing its own return-object folklore
- controller contracts should become real package-understood artifacts rather
  than remaining only nice object shapes
- graphs must become the primary operational surface for public inputs and
  public outputs rather than a purely descriptive publication wrapper
- public contract and operational contract must be allowed to differ
- scope and instance identity must have an explicit typed semantic model under
  the ergonomic string-facing authoring surface
- repeated controller families and future dynamic subgraphs must have explicit
  instance identity semantics
- graph-native mutations should lower to one canonical operation envelope
- public inputs may have different authority classes; not every exposed input is
  necessarily equally writable
- graph-native export/import, restore, and diagnostics surfaces must speak in
  graph-contract terms directly
- contract deltas and historical contract change artifacts should be first-class
  enough that future forms/resources can reason about boundary evolution
- canonical naming law across local ids, runtime ids, public names, and
  exported names must be explicit
- the architecture should deliberately protect against bad-path ergonomics
  instead of making every unsafe path equally pleasant
- the authoring grammar for `input`, `computed`, and `output` should converge
  further here so future forms/resources do not inherit avoidable product debt
- forms and resources must inherit this graph-owned ownership model rather than
  inventing private scope/lifecycle machinery
- this milestone is allowed before generic aspect-capacity work because it
  changes naming, lifecycle, and public-boundary semantics rather than aspect
  breadth semantics

Normative consequence:

- any implementation that keeps repeated controller composition dependent on
  manual unique-string discipline alone is out of spec
- any implementation that makes graph-owned `inputs` only a documentation
  convention is out of spec
- any implementation that leaves graph scope as implicit local JS metadata with
  no diagnostics/export meaning is out of spec
- any implementation that leaves controller contracts as unstandardized feature
  folklore is out of spec
- any implementation that exposes public graph inputs without a graph-native
  operational surface is out of spec
- any implementation that leaves controller identity and contract proof as mere
  object-shape folklore is out of spec
- any implementation that treats all public inputs as one undifferentiated
  writable lane is out of spec
- any implementation that hides the relationship among local ids, runtime ids,
  public names, and exported names is out of spec
- any implementation whose ergonomic helpers bypass one canonical mutation
  envelope is out of spec
- any implementation that treats repeated-instance identity as a future app
  concern instead of a runtime/package boundary concern is out of spec
- any implementation that keeps graph export/import as merely filtered
  runtime-wide truth without graph-native boundary meaning is out of spec
- any implementation that makes forms/resources the first owners of scoped
  controller lifecycle is out of spec

## Public API Model

### Primary authoring model

The package should converge on three aligned layers:

- **scoped controller authoring**
  - ordinary functions over a scoped graph authoring surface
  - local ids are local to that scope
- **graph-owned construction**
  - one graph boundary owns scope creation and lifecycle
- **public graph contract**
  - explicit `inputs` and `outputs` define what the graph exposes

The intended direction is in this family:

```ts
export const itemDetailGraph = createSignals().graph("itemDetail", (graph) => {
  const editSession = createEditSessionController(graph.scope("editSession"));
  const workflow = createWorkflowController(graph.scope("workflow"), editSession);

  return graph.expose({
    inputs: {
      ...editSession.inputs,
    },
    outputs: {
      ...editSession.outputs,
      ...workflow.outputs,
    },
  });
});
```

The exact public spelling may become:

- `signals.graph("id", (graph) => ...)`
- `createGraph("id", (graph) => ...)`
- or an equivalent facade

but the semantic shape is locked.

### Required surface categories

At minimum, this milestone must define real product vocabulary for:

- `ScopedSignalNamespace` or equivalent scoped controller surface
- `GraphScope` or equivalent graph-owned authoring scope
- `ControllerContract` or equivalent branded controller artifact
- `PublishedGraphInputs`
- `PublishedGraphOutputs`
- `PublishedGraphOperationalSurface`
- graph construction/exposure request type
- graph mutation/apply request type
- graph contract-delta artifact type
- graph lifecycle/public-contract artifact type

The exact names may evolve. The semantic categories may not.

### Identity model

The runtime/public model must distinguish at least:

- **controller-local ids**
  - authoring names inside a scope
- **runtime-owned canonical ids**
  - globally unique identities after lowering
- **graph public input/output names**
  - explicit contract names chosen for consumers
- **exported/imported contract names**
  - names that survive graph-native export/import boundaries
- **scope or instance identity values**
  - semantic graph/controller instance identity beyond raw display strings

Those categories must not collapse into one string that means three different
things in three different APIs.

Canonical naming law:

- local ids are authoring-local and may repeat across sibling scopes
- runtime ids are canonical lowered identities and must be globally unique
- public contract names are consumer-facing boundary names and may intentionally
  differ from local ids
- exported/imported contract names must either equal public contract names or
  carry an explicit machine-readable remapping artifact
- diagnostics/history/export must never force consumers to guess which of those
  names is being shown

### Public contract rule

Graph `inputs` and `outputs` must become real public contract surfaces.

That means:

- a graph may expose source handles as named public inputs
- a graph may expose output handles or synthesize public outputs from readable
  handles where that remains honest
- internal controller signals may remain unexposed
- diagnostics/history/export surfaces must be able to explain the public graph
  contract directly

### Controller contract rule

The package should stop treating controller return shapes as incidental local
objects.

This milestone should converge on a standard controller contract shape in this
family:

```ts
return {
  inputs: {
    serverItemData,
    draftEdits,
  },
  outputs: {
    effectiveItemData,
    dirtyState,
  },
  internal: {
    // future staging, validation, provenance, undo, or resource-local signals
  },
};
```

Exact helper spelling may evolve, but the semantic categories are locked:

- `inputs`
  - public source-state contract
- `outputs`
  - public derived/publication contract
- `internal`
  - authored graph structure that must not accidentally become public API

The stronger long-term direction is:

- controller contracts should be brandable/constructible through a package-owned
  helper such as `defineController(...)` or `graph.controller(...)`
- later phases may keep plain-object authoring ergonomics on top, but the
  runtime/package should understand one canonical controller artifact

### Graph operational rule

If a graph exposes public `inputs`, then the graph surface must eventually own
real operational affordances for them instead of requiring every consumer to
immediately drop down to raw handles.

This milestone should therefore leave room for graph-native operations in this
family:

- `graph.readInputs()`
- `graph.writeInputs(...)`
- `graph.patchInputs(...)`
- `graph.transaction(...)`
- `graph.resetInputs()`
- `graph.apply(...)`

The exact surface may stage in over phases, but the milestone is only
architecturally complete if the graph can act as the operational boundary for
its public contract.

Those helpers should converge on one canonical lower-level mutation envelope in
this family:

```ts
graph.apply({
  writes: {
    serverItemData: nextServerItem,
  },
  patches: {
    draftEdits: nextDraftPatch,
  },
  reset: ["draftEdits"],
});
```

Ergonomic helpers may remain, but they must lower to one contract-bearing graph
mutation artifact.

### Public input authority rule

Public graph inputs are not required to be one undifferentiated writable class.

This milestone should preserve explicit room for authority categories in this
family:

- writable public inputs
- read-only public inputs
- imported or externally-owned public inputs
- future policy-gated or command-mediated public inputs

The exact type names may evolve, but the architecture must not assume:

> exposed input == freely writable input

### Contract change rule

Graph contracts should be able to describe how the public boundary changed over
time.

This milestone should therefore preserve space for graph-native contract
artifacts in this family:

- `graph.contract()`
- `graph.contractDelta(since)`
- `graph.contractHistory()`

The exact surface can stage in, but the architecture should not force future
products to rediscover contract evolution indirectly from raw node history.

### Repeated-instance rule

The scoped graph model must be explicit about repeated and dynamic instance
architecture.

It must define enough identity law that future code can safely express:

- side-by-side editors
- tabbed copies of the same feature
- modal + page copies of the same feature
- repeated row/item controllers
- nested subgraphs
- future dynamic graph families

without reducing correctness to “choose different strings carefully.”

### Significant-code bridge rule

The current manual scope-string pattern is an acceptable transitional bridge
for users today. It is not the end-state product model.

This milestone is complete only when the product surface gives serious app code
something better than:

```ts
signals.input(`${scope}.serverItemData`, null);
```

## Required Architecture Changes

### 1. Add a dedicated scope/lifecycle responsibility space

Do not hide scoped identity inside generic `signals.js` helpers.

Expected responsibility split:

- scoped controller authoring
- graph-owned lifecycle and scope creation
- public graph contract exposure
- graph diagnostics/history/export alignment

Likely file families:

- `package-src/product/scopes.ts`
- `package-src/product/graphs.ts`
- `package/types/graph_surface.d.ts`
- supporting type modules as needed

The exact filenames can vary, but scoped identity must be a named
responsibility, not accidental string manipulation.

### 2. Lower scope once

Scoped identity must follow the same structural pattern as the rest of the
package:

- authoring intent
- validated scope/graph declaration
- lowered canonical runtime identities
- committed graph/public-contract artifact
- derived diagnostics/history/export summaries

The implementation must not let every consumer invent its own prefixing or
re-discover graph-local identity independently.

### 3. Preserve one product facade, not one ownership god file

The package may still expose one public facade, but:

- scoped authoring
- graph construction
- public contract exposure
- diagnostics alignment
- history/export alignment

must remain decomposed internally by reason to change.

### 4. Keep compatibility explicit

The current composition API and manual-scope transitional path remain important
compatibility surfaces during migration.

This milestone must:

- keep existing controller-first composition usable
- make the graph-owned scope model the preferred path
- subordinate manual runtime-global naming to explicit compatibility guidance

## Phases

Phase ordering is strict.

This milestone is not an any-order buffet because later phases consume proof
and vocabulary established by earlier ones:

- controller contract categories must exist before graph-owned construction can
  standardize them
- scoped identity and instance identity must exist before graph operations can
  safely act on public inputs
- graph-owned construction must exist before grammar cleanup can know which
  surface it is cleaning up
- graph-native export/import must not be designed before graph contracts,
  diagnostics, and operational surfaces are already concrete

If a later phase can be implemented without depending on earlier phase outputs,
the earlier phase is underspecified.

### Phase 1: Controller Contract And Vocabulary Lock

Purpose:

- freeze the boundary vocabulary before more helpers accumulate

Required work:

- define standard controller contract categories:
  - `inputs`
  - `outputs`
  - `internal`
- define the branded controller artifact direction
- define scoped controller identity categories
- define graph-owned lifecycle categories
- define public graph input/output contract categories
- define public contract vs operational contract categories
- define canonical distinctions among local ids, runtime ids, and public names
- define exported/imported name posture
- define public input authority classes
- define graph-native operational boundary categories for public inputs
- define validation/denial classes for collisions, illegal reuse, and
  cross-runtime misuse

Exit criteria:

- scoped authoring is a named product concept
- controller contracts are a named product concept
- public input/output contract is a named product concept
- the docs no longer imply manual prefixing or ad hoc controller return shapes
  are the final architecture

Concrete target shape:

```ts
type ControllerContract<TInputs, TOutputs, TInternal> = {
  inputs: TInputs;
  outputs: TOutputs;
  internal: TInternal;
};

type GraphContractShape<TInputs, TOutputs> = {
  inputs: TInputs;
  outputs: TOutputs;
};

type GraphOperationalContract<TWrites, TPatches, TCommands> = {
  writes: TWrites;
  patches: TPatches;
  commands: TCommands;
};
```

Primary code directions:

- `package/types/controller_surface.d.ts`
- `package/types/graph_surface.d.ts`
- `package-src/product/scopes.ts`
- `package-src/product/graphs.ts`

Must not start Phase 2 until:

- controller contract categories are frozen tightly enough that later phases do
  not have to redesign every example and handle grouping shape
- public vs operational contract categories are explicit enough that later
  graph operations do not retroactively redefine public contract meaning
- validation and denial classes for scope and graph ownership are explicit
  enough to become runtime proof types later

### Phase 2: Scoped Authoring And Instance Identity

Purpose:

- give serious app code a real scoped authoring surface with an explicit
  repeated-instance story

Required work:

- add `scope(...)` or equivalent scoped authoring entrypoint
- make `input`, `computed`, and `output` inside a scope author local ids
- enforce collision rules through scope + graph lowering instead of hoping
- preserve typed handle categories through scoped authoring
- add typed scope/instance identity internally even if public authoring still
  starts from strings
- define repeated-instance and nested-instance identity expectations
- define dynamic-subgraph identity expectations strongly enough that future
  forms/resources do not improvise them privately

Exit criteria:

- repeated controller families can be instantiated in one runtime without
  manual unique-string discipline
- nested and repeated scopes have an explicit identity story, not just working
  examples
- scoped authoring is real, typed, and tested

Concrete target shape:

```ts
const graph = createSignals().graph("itemDetail", (graph) => {
  const editSession = graph.scope("editSession");
  const workflow = graph.scope("workflow");

  const row0 = graph.scope("rows").scope("row-0");
  const row1 = graph.scope("rows").scope("row-1");
});
```

Primary code directions:

- `package-src/product/scopes.ts`
- `package-src/product/symbols.ts`
- `package/types/callable_surface.d.ts`
- `package-src/product/signals.runtime.test.mjs`

Proof target:

- canonical id lowering proves:
  - graph id
  - scope path
  - local id
  remain distinct machine-readable categories
- scope identity is not just a display string; it becomes a semantic value the
  runtime/package can carry through diagnostics and export surfaces

Must not start Phase 3 until:

- repeated and nested instance identity is explicit enough that graph-owned
  construction can safely assume scope-local names really are local
- same-runtime and same-graph ownership proof hooks exist for builder surfaces

### Phase 3: Graph-Owned Construction And Rich Public Contract

Purpose:

- make the graph itself the lifecycle and contract owner for serious
  composition

Required work:

- add graph-owned construction callback/builder form
- add `graph.expose(...)` or equivalent explicit public contract declaration
- support explicit public `inputs` and `outputs`
- support explicit internal controller structure intentionally
- support controller artifact composition instead of only plain object
  composition
- standardize controller contract composition across graph boundaries
- ensure graph artifact identity includes public contract visibility
- ensure the graph contract model leaves room for future contract families
  without collapsing them into one bag

Exit criteria:

- graph construction is no longer just "wrap already-global handles later"
- public graph inputs/outputs are explicit product artifacts
- controller internals can remain internal without accidental public exposure
- graph contract composition is a real product pattern, not feature folklore

Concrete target shape:

```ts
export const itemDetailGraph = createSignals().graph("itemDetail", (graph) => {
  const editSession = createEditSessionController(graph.scope("editSession"));
  const workflow = createWorkflowController(graph.scope("workflow"), editSession);

  return graph.expose({
    inputs: {
      ...editSession.inputs,
    },
    outputs: {
      ...editSession.outputs,
      ...workflow.outputs,
    },
  });
});
```

Primary code directions:

- `package-src/product/graphs.ts`
- `package/types/graph_surface.d.ts`
- `package/types/controller_surface.d.ts`
- `package/types-smoke.ts`

Proof target:

- builder-form graphs reject ambient same-runtime handles that were not authored
  through the owning graph scope
- builder-form graphs understand controller artifacts as boundary-bearing
  composition units, not just bags to spread

Must not start Phase 4 until:

- builder graphs are branded boundary artifacts rather than plain objects
- graph exposure is the only admitted route for public contract construction

### Phase 4: Graph Operational Surface

Purpose:

- make public graph inputs and transactions graph-native operational surfaces

Required work:

- add graph-native read/write/patch/transaction/reset surfaces where the graph
  contract can honestly own them
- add one canonical mutation envelope under those helpers
- keep the graph operational surface aligned with runtime-owned transaction and
  authority semantics
- preserve public input authority classes through graph-native operations
- ensure graph operations are boundary-honest and do not conceal broad graph
  rediscovery or global scans
- prove that graph contract consumers do not need to drop to raw handles for
  ordinary public-input workflows

Exit criteria:

- public inputs are operationally first-class at the graph boundary
- graph lifecycle ownership is real in use, not only descriptive in shape

Concrete target shape:

```ts
itemDetailGraph.readInputs();
itemDetailGraph.writeInputs({
  serverItemData: nextServerItem,
});
itemDetailGraph.patchInputs({
  draftEdits: nextDraftPatch,
});
itemDetailGraph.transaction((tx) => {
  tx.set(itemDetailGraph.inputs.draftEdits, nextDraft);
});
itemDetailGraph.resetInputs();

itemDetailGraph.apply({
  writes: {
    serverItemData: nextServerItem,
  },
  patches: {
    draftEdits: nextDraftPatch,
  },
});
```

Primary code directions:

- `package-src/product/graphs.ts`
- `package-src/product/transactions.ts`
- `package/types/graph_surface.d.ts`
- graph-operation-specific runtime proof tests

Performance and honesty target:

- every graph-native input operation must state whether it is:
  - O(number of named public inputs touched)
  - O(number of affected runtime nodes)
- no graph-native operation is allowed to rediscover the graph boundary by
  whole-runtime scan

Must not start Phase 5 until:

- graph operations are real enough that grammar cleanup can target the actual
  serious path rather than a hypothetical one

### Phase 5: Authoring Grammar Convergence

Purpose:

- make `input`, `computed`, and `output` read like one coherent language before
  forms/resources inherit the surface

Required work:

- converge on one canonical happy-path grammar
- keep value-first string-safe authoring honest
- make ids feel like metadata rather than registration ceremony
- demote older expert/compat forms without deleting them where they still
  matter

Exit criteria:

- the primary authoring language reads coherently across source, derived, and
  exposed state
- new users do not have to learn three unrelated surface dialects

Concrete target shape:

```ts
const serverItemData = graph.scope("editSession").input(null, {
  id: "serverItemData",
});

const effectiveItemData = graph.scope("editSession").computed(
  () => ({ ...(serverItemData() ?? {}), ...draftEdits() }),
  { id: "effectiveItemData" },
);

const submitReadiness = graph.scope("workflow").output(
  () => ({
    enabled: dirtyState().isDirty && Boolean(effectiveItemData()?.workflow_target_state_id),
  }),
  { id: "submitReadiness" },
);
```

Primary code directions:

- `package-src/product/signals.ts`
- `package/types/callable_surface.d.ts`
- `README.md`
- `docs/app_surface_reference.md`

Must not start Phase 6 until:

- one canonical happy path is chosen and reflected in docs/tests
- compatibility lanes are explicitly secondary instead of silently co-equal

### Phase 6: Contract Diagnostics And Dependency Introspection

Purpose:

- make the public graph contract explainable as a boundary, not only as a set
  of node internals

Required work:

- surface graph-owned identity in diagnostics where relevant
- surface public input/output contract in history where relevant
- add contract-delta and contract-history artifact direction
- add graph-level dependency explanation:
  - which public outputs depend on which public inputs
  - which contract-local versions changed
  - why a contract-local output changed or did not change
- define contract summaries and contract-level version/change surfaces

Exit criteria:

- scoped graph identity is not local helper lore
- diagnostics/history can explain the graph contract directly
- contract-local dependency explanation is available without raw runtime graph
  spelunking

Concrete target shape:

```ts
const contract = itemDetailGraph.contract();
const diagnostics = itemDetailGraph.inspectDiagnostics();

contract.inputs;
contract.outputs;
diagnostics.input("draftEdits");
diagnostics.output("submitReadiness");
diagnostics.dependenciesForOutput("submitReadiness");
diagnostics.contractSummary();
itemDetailGraph.contractDelta(previousContractSnapshot);
```

Primary code directions:

- `package-src/product/graphs.ts`
- `package-src/product/diagnostics.ts`
- `package/types/graph_surface.d.ts`
- `docs/diagnostics_and_history_reference.md`

Must not start Phase 7 until:

- contract-level names and dependency explanations are stable enough to be used
  as historical/export contract vocabulary

### Phase 7: Graph-Native Export, Import, And Historical Truth

Purpose:

- make graphs first-class historical/export boundaries instead of filtered
  runtime-wide stories

Required work:

- define graph-native export artifacts
- define graph-native import/restore/hydrate posture
- preserve canonical naming law explicitly through export/import
- define how scoped graph identity survives restore/replay/compatibility reads
- define graph contract delta/history posture under restore/import
- prove that equivalent flat/manual-scope/scoped-graph authoring yields the
  same committed truth and explainable public boundary
- ensure graph-native export/import remains runtime-truth consuming rather than
  inventing a second graph engine

Exit criteria:

- graph export/import is graph-native rather than an incidental filtered
  runtime artifact
- historical boundary truth is stable across equivalent authoring styles

Concrete target shape:

```ts
const exported = itemDetailGraph.exportDefinition();
const snapshot = itemDetailGraph.exportSnapshot();

const restored = createSignals().importGraph(exported, snapshot);
restored.contract();
restored.contractHistory();
restored.inspectHistory();
```

Primary code directions:

- `package-src/product/graphs.ts`
- `package-src/product/history.ts`
- `package/types/graph_surface.d.ts`
- package verifier runtime/type smoke

Proof target:

- graph-native export/import artifacts preserve:
  - graph id
  - public input names
  - public output names
  - scoped identity lineage
  - contract-level historical explanation

Must not start Phase 8 until:

- graph-native export/import is no longer just filtered runtime export with a
  different label
- restore/hydrate semantics are specific enough to document and certify

### Phase 8: Documentation, Examples, And Certification Closeout

Purpose:

- make the mature graph authoring surface teachable and regression-protected

Required work:

- update docs to teach the graph-owned scoped model as the primary path
- add simple and realistic examples across:
  - repeated instance feature composition
  - forms-style draft/effective/validation flows
  - resource/query-style input/resource contract flows
  - workflow/state orchestration
- add proof tests for collisions, graph-owned lifecycle, public input/output
  contract parity, graph-native operational surfaces, and graph-native
  historical/export truth
- add clean-consumer package proof where appropriate

Exit criteria:

- a user can discover the mature pattern from docs alone
- forms/resources can treat this as stable substrate instead of future hope
- the milestone is certification-complete for the larger application graph
  boundary model, not just the earlier scoped-publication subset

Required example families:

- repeated row-level editor instances
- modal + page copy of one controller family
- form-style draft/effective/validation graph
- resource/query-style public input plus derived readiness graph
- workflow/status projection graph

Required package-proof families:

- local workspace runtime proof
- clean-consumer tarball runtime proof
- clean-consumer tarball type proof
- authoring-style equivalence proof across:
  - flat
  - manual scope
  - graph-owned scoped construction

## Must Ship

- standardized controller contract shape
- branded or package-understood controller artifact direction
- scoped controller/graph authoring vocabulary
- real scoped authoring entrypoint such as `scope(...)` or equivalent
- graph-owned construction boundary
- explicit public graph `inputs` and `outputs`
- explicit public contract vs operational contract distinction
- graph-native operational surface for public inputs
- one canonical graph mutation envelope beneath ergonomic helpers
- canonical distinction among local ids, runtime ids, and public contract names
- explicit naming law for exported/imported graph identities
- collision-safe repeated controller composition
- explicit repeated/dynamic instance identity model
- typed scope/instance identity model
- further-converged `input` / `computed` / `output` authoring grammar
- contract-level diagnostics and dependency introspection
- contract delta/history artifact direction
- graph-native export/import and historical boundary artifacts
- diagnostics/history/export alignment for scoped graph identity
- docs and examples that teach the mature significant-code path first
- certification-grade tests for repeated-instance safety and public-contract
  parity

## Must Preserve

- runtime truth remains runtime-owned
- controller authoring remains ordinary code rather than a second local graph
  engine
- graph-native operational helpers remain consumers of runtime authority rather
  than shadow mutation engines
- compatibility/manual-scope lanes remain available during transition
- output publication remains an explicit public boundary
- forms/resources consume this ownership model rather than redefining it
- this milestone must not smuggle generic aspect-capacity semantics into the
  wasm package

## Acceptance Evidence

This milestone is complete only when all of the following are true:

- repeated instances of one controller family can coexist in one runtime
  without collision
- graph-owned scoped authoring and equivalent flat/manual-scope scripts produce
  the same committed public truth
- public graph `inputs` and `outputs` are explicit, typed, and
  diagnostics-visible
- controller contracts preserve internal/public distinctions without accidental
  leakage
- controller artifacts compose as real package-understood boundary units instead
  of remaining only object-shape folklore
- graph public inputs are operationally usable through graph-native surfaces
  instead of only via raw handles
- graph-native operations lower to one canonical mutation contract
- public input authority classes remain explicit and stable under graph
  operations
- repeated and dynamic instance identity is explicit enough for future
  forms/resources
- canonical naming law remains stable across authoring, diagnostics, export,
  import, and restore
- `input`, `computed`, and `output` read as one guided authoring language
- diagnostics/history/export surfaces can explain scoped graph identity and
  public contract names honestly
- contract-level dependency explanations identify which public outputs depend on
  which public inputs
- contract delta/history surfaces can explain how the graph boundary changed
  over time
- graph-native export/import and restore/hydrate surfaces preserve the same
  contract truth as equivalent authoring styles
- forms/resources no longer need to invent their own scope or lifecycle model

### Required Named Test Families

- `The Repeated Controller Instance Collision Test`
  Proves repeated instances of the same controller family do not collide in one
  runtime.
- `The Scoped Graph And Manual Scope Equivalence Test`
  Proves scoped graph-owned authoring and the equivalent manually scoped script
  converge to the same committed public truth.
- `The Public Graph Input And Output Contract Test`
  Proves graph `inputs` and `outputs` are explicit, typed, and stable across
  diagnostics/history/export reads.
- `The Graph-Owned Lifecycle Boundary Test`
  Proves graph-owned construction, disposal, and public-contract identity stay
  coherent under restore/replay and repeated composition.
- `The Controller Contract Internal Boundary Test`
  Proves controller `internal` signals do not leak into public graph contracts
  unless explicitly exposed.
- `The Controller Artifact Composition Test`
  Proves branded or package-understood controller artifacts compose across graph
  boundaries without collapsing back into ad hoc object folklore.
- `The Graph-Native Input Operations Test`
  Proves graph-owned read/write/patch/transaction/reset surfaces stay aligned
  with runtime authority and public contract identity.
- `The Graph Mutation Envelope Equivalence Test`
  Proves ergonomic graph input helpers lower to one canonical mutation contract
  without semantic drift.
- `The Repeated And Dynamic Instance Identity Test`
  Proves repeated families, nested scopes, and dynamic instance creation remain
  collision-safe and historically explainable.
- `The Public Input Authority Class Test`
  Proves writable, read-only, imported, or otherwise differently-authorized
  public inputs remain distinct and enforced.
- `The Authoring Grammar Convergence Test`
  Proves the primary `input` / `computed` / `output` grammar remains coherent,
  string-safe, and guided toward the right path.
- `The Canonical Naming Law Test`
  Proves local ids, runtime ids, public contract names, and exported/imported
  names remain distinct, stable, and correctly mapped.
- `The Contract Dependency Explanation Test`
  Proves public contract diagnostics can explain which public outputs depend on
  which public inputs and why contract-local changes happened.
- `The Contract Delta And History Test`
  Proves graph contracts can explain how the public boundary changed over time
  without forcing consumers to reconstruct that from raw node history.
- `The Graph-Native Export And Restore Equivalence Test`
  Proves graph-native export/import/restore surfaces preserve the same public
  contract truth as equivalent flat/manual-scope/scoped-graph authoring.
- `The Forms And Resources Dependency Readiness Test`
  Proves the scoped lifecycle surface provides the contract forms/resources need
  without private scope reinvention.

## Adversarial Test Requirements

This milestone must not be certified by a handful of happy-path runtime tests.

We already know the package can look elegant while still shipping awkward bugs
at the public boundary:

- id/value overload ambiguity
- output naming mismatches
- graph publication artifacts that look right in local examples but degrade in
  real packaged consumption
- controller-local intent collapsing into runtime-global accidents

The test strategy for this milestone therefore has to flush out both:

- **simple product bugs**
  - the sorts of bugs users hit immediately in ordinary app code
- **deep structural bugs**
  - the sorts of bugs that only appear under repeated composition, replay,
    export/import, or package-boundary consumption

The required bar is:

- local runtime tests
- local type-surface tests
- clean-consumer tarball runtime tests
- clean-consumer tarball type tests
- adversarial equivalence tests across authoring styles
- boundary-honesty tests for diagnostics/history/export

### Test Matrix

Every major capability added by this milestone must be stressed across these
five dimensions:

1. **Authoring-shape variation**
   - flat/manual-scoped authoring
   - controller-first scoped authoring
   - graph-owned builder authoring
2. **Identity pressure**
   - overlapping local ids
   - repeated controller families
   - nested scopes
   - dynamic instance creation
3. **Boundary variation**
   - local runtime use
   - packaged clean-consumer use
   - diagnostics/history reads
   - export/import/restore use
4. **Contract variation**
   - public inputs only
   - public outputs only
   - mixed public inputs/outputs
   - internal-only controller signals coexisting with public ones
5. **Value-shape variation**
   - string values
   - object values
   - partial/patch-style updates
   - optional/null-bearing values
   - structured workflow/form/resource-style projections

If a requirement is only tested in one of those dimensions, it is not closed.

### Required Bug-Class Coverage

The test suite for this milestone must explicitly cover these bug classes:

#### 1. Id parsing and value-shape ambiguity

This exists because we already hit a real product bug here.

Must cover:

- `input(value, { id })` with string values
- `input(value, { id })` with empty-string values
- explicit id-first and value-first forms coexisting in one graph
- output names that differ from local authoring ids
- graph publication from handles whose local id and public output name are
  intentionally different

Pass condition:

- no overload path may silently reinterpret value as id or id as value
- graph contract names and authored local ids stay distinct and stable

#### 2. Public contract leakage

Must cover:

- controller `internal` signals not exposed by default
- explicit re-export of selected controller outputs only
- public graph inputs coexisting with non-public internal staging signals
- diagnostics/history/export surfaces not accidentally surfacing internal-only
  signals as public contract

Pass condition:

- contract surfaces show only what `graph.expose(...)` actually exposed

#### 3. Repeated-instance and nested-scope stress

Must cover:

- two instances of one controller family in one graph
- many instances of one controller family in one graph
- nested repeated scopes
- sibling scopes with the same local ids
- repeated instances surviving restore/replay and export/import

Pass condition:

- no collision
- no aliasing
- no loss of graph-local identity
- no ambiguity in diagnostics/history/export

#### 4. Graph-owned lifecycle honesty

Must cover:

- builder graphs rejecting handles authored outside the owning graph scope
- disposal/teardown of graph-owned contracts
- branch/restore preserving graph-owned contract identity
- public input operations staying inside the owning graph boundary

Pass condition:

- graph-owned construction is a real authority boundary, not a stylistic
  wrapper

#### 5. Graph-native operational surface honesty

Must cover:

- `writeInputs(...)` writing only named public inputs
- `patchInputs(...)` against structured object inputs
- graph-scoped transaction use
- reset semantics
- operation denial on unknown public input names
- operation denial on foreign graph handles

Pass condition:

- graph operations never silently broaden to runtime-global mutations
- graph operations fail explicitly and typed when contract membership is wrong

#### 6. Contract diagnostics and dependency explanation

Must cover:

- output depending on one public input
- output depending on several public inputs
- contract-local outputs that do not change after one public input change
- repeated instances whose diagnostics names must remain distinct
- history and diagnostics agreement on graph contract identity

Pass condition:

- diagnostics can answer:
  - which public inputs exist
  - which public outputs exist
  - which outputs depend on which inputs
  - why one output changed or did not change

#### 7. Graph-native export/import and restore equivalence

Must cover:

- export/import of a graph with only public outputs
- export/import of a graph with public inputs and outputs
- restore/hydrate after repeated-instance composition
- equivalence across flat/manual-scope/scoped-builder authoring
- explicit denial paths where live/runtime-local semantics cannot be carried

Pass condition:

- graph-native historical/export truth is stable and contract-honest
- the system never silently reinterprets graph identity during import/restore

#### 8. Forms/resources readiness, not toy readiness

Must cover:

- draft/effective/dirty form-style controller shapes
- resource/query-style controllers with public request inputs and derived
  readiness or status outputs
- repeated form/resource controller families in one graph
- package-consumer examples using these shapes directly

Pass condition:

- downstream product teams can adopt the boundary model without inventing
  missing contract/lifecycle semantics

### Required Proof Lanes

The following proof lanes are mandatory for this milestone:

#### A. Local runtime adversarial tests

Use package runtime tests to stress:

- collisions
- scope nesting
- graph-owned lifecycle denials
- contract leakage
- graph-native operations
- export/import/restore equivalence

#### B. Local type-surface adversarial tests

Use strict type-smoke tests to stress:

- controller contract shapes
- graph builder forms
- graph-native operational surfaces
- output/public-name vs local-id separation
- read-only vs writable distinctions

#### C. Clean-consumer tarball runtime tests

The packed npm artifact must prove:

- scoped/controller/graph builder authoring works after install
- forms/resource-style examples work after install
- graph diagnostics/history/export paths work after install
- no file-layout or entrypoint omission reappears under the packaged boundary

#### D. Clean-consumer tarball type tests

The packed npm artifact must type-check:

- controller contracts
- graph builder forms
- graph-native input operations
- graph-native contract/dx/history/export surfaces

#### E. Equivalence certification tests

For the same feature graph, prove equivalence across:

- flat manual scope
- controller-first scoped authoring
- graph-owned builder authoring

The equivalence package must compare:

- public input contract
- public output contract
- committed values
- diagnostics summaries
- history summaries
- export/import truth

### Required Failure-First Tests

This milestone must also include tests whose whole purpose is proving the
system says "no" correctly.

At minimum, include:

- duplicate local id denial
- duplicate public contract name denial
- foreign-runtime handle denial
- foreign-graph handle denial
- unknown public input operation denial
- malformed controller contract denial
- graph expose misuse denial
- export/import compatibility denial where required

If the wrong thing can happen and still “mostly works,” it needs a denial test.

### Required Self-Check

This milestone is not done if it only:

- wraps manual prefixing in a prettier helper
- renames runtime-global ids without changing ownership truth
- adds public `inputs` as a docs-only convention
- leaves diagnostics/history/export blind to graph-owned identity
- claims forms/resources are unblocked while they still need to invent private
  scope or lifecycle semantics

## Sequencing Notes

This milestone belongs immediately after the shipped composition milestone and
before forms/resources.

Why:

- the current composition API is real, but its ownership model is not yet
  strong enough for repeated serious application composition
- forms need public graph inputs, graph-owned lifecycle, and scoped controller
  identity
- resources need the same thing for route/resource-local state and explicit
  public contracts
- if either product lands first, it will be forced to invent the missing scope
  and lifecycle model privately

This milestone is intentionally allowed before the generic aspect-capacity
rewrite because:

- it solves ownership, naming, and boundary truth
- it does not depend on wider aspect-width semantics to be honest
- the current 32-aspect default is sufficient for this ownership milestone's
  proof obligations

The intended order is now:

1. host capability closeout
2. composition API and graph publication
3. scoped controller identity and graph-owned lifecycle
4. forms product surface
5. API surface

That order keeps the next two application-facing products from becoming the
first real owners of scope, lifecycle, and public graph contracts.


