# worth-signals-wasm Opaque Identity And Ergonomic Authoring Plan

> **Status:** Planned engineering spec
>
> **Roadmap parent:** [wasm_product_roadmap.md](./wasm_product_roadmap.md)
>
> **Lifecycle prerequisite:** [controller_scope_and_graph_lifecycle_plan.md](./controller_scope_and_graph_lifecycle_plan.md)
>
> **Core vision:** [plans/worth_signal/worth_signal_vision.md](../../../plans/worth_signal/worth_signal_vision.md)
>
> **Core DX reference:** [plans/worth_signal/dx_plan.md](../../../plans/worth_signal/dx_plan.md)
>
> **Core test requirements:** [plans/worth_signal/test-requirements.md](../../../plans/worth_signal/test-requirements.md)

## Goal

Make the main `worth-signals-wasm` app-authoring lane substantially easier
without reducing runtime truth, graph truth, diagnostics truth, or restore
truth.

The target outcome is:

- ordinary app authoring does not require user-authored signal ids
- internal signal identity is runtime-owned and opaque
- optional debug names remain diagnostic metadata only
- public graph contract names remain explicit
- portability/spec lanes keep explicit structural names where they genuinely
  matter
- simple CRUD-style code and complicated controller-composed code both become
  less ceremonious without collapsing architectural boundaries

This milestone is not a rollback from graph-owned lifecycle. It is the
ergonomic layer that should sit on top of that ownership model.

## Why This Milestone Exists

The current wasm package is strong architecturally, but it still asks ordinary
app code to care about too much identity ceremony too early:

- explicit `id` on nearly every authored signal
- local ids that feel too close to public contract names
- CRUD-style authoring that still feels like runtime paperwork
- controller composition that is structurally correct but heavier than it needs
  to be for ordinary application code

That friction is survivable in expert code and worth it at public boundaries,
but it is not the right default for the product path that should eventually
carry basic CRUD apps, edit sessions, and ordinary application state.

Rust `worth-signal::easy` proved one useful product truth:

- the shortest path should feel like state authoring, not runtime assembly

This milestone adopts that lesson without porting Rust `easy` one-to-one.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects here is solving the real structural
  problem instead of papering over ceremony. This milestone therefore makes
  authored identity runtime-owned rather than merely adding more convenience
  overloads on top of string ids.
- `arch_laws.md`
  The most important laws here are 7, 20, 33, 40, and 41. Internal identity,
  public contract names, exported names, and debug names must remain distinct
  meanings, and the type surface must encode which identity facts have been
  proven.
- `perf_laws.md`
  The most important thing it protects is API honesty. Cheap-looking local
  authoring must not hide broad string registries, repeated name resolution, or
  reconstructive graph walks.
- `domain_laws.md`
  The most important thing it protects is decomposition by responsibility.
  Opaque identity, public contract naming, mutation ergonomics, and controller
  builders must not collapse into one god surface.
- `worth_signal_vision.md`
  The most important thing it protects is that `worth-signal` remains derived
  runtime infrastructure, never truth. This milestone can improve ergonomics,
  but it must still compile down to the same runtime truth model.
- `test-requirements.md`
  The most important thing it protects is proof-grade equivalence. A more
  ergonomic authoring lane must produce the same committed truth, replay truth,
  and diagnostics truth as the explicit lane.
- `dx_plan.md`
  The most important thing it protects is that the first success path must be
  fast, while advanced control remains available without polluting the default
  path.
- `wasm_product_roadmap.md`
  The most important thing it protects is sequencing. Forms and resources
  should inherit a better ergonomic substrate rather than becoming the place
  where CRUD ergonomics are invented under pressure.
- `controller_scope_and_graph_lifecycle_plan.md`
  The most important thing it protects is that graph lifecycle and controller
  contracts are already real. This milestone must simplify authoring on top of
  that, not replace it with a second graph system.

## Adversarial Constraint

This milestone must survive the following hostile condition:

> A long-lived TypeScript application with many local signals, repeated
> controller instances, graph-owned publication, branch/restore activity,
> diagnostics inspection, and public graph inputs/outputs must converge to the
> same committed truth, graph contract, exported/imported truth, and replay
> artifacts after the package has fully moved from authored string ids on the
> main lane to the new opaque-identity ergonomic lane.

Concretely, the design must remain correct when:

- many internal signals are authored without explicit ids
- multiple local signals share the same optional debug name
- repeated controllers are instantiated in one runtime
- a graph publishes explicit public names from opaque internal handles
- diagnostics must explain both opaque runtime identity and public boundary
  names honestly
- spec/compatibility lanes still rely on explicit structural names
- export/import and restore preserve graph contract truth even though internal
  authoring did not use user-authored ids

If the ergonomic lane produces:

- hidden identity collisions
- unstable public names
- unstable exported/imported names
- diagnostics that cannot explain what a handle really is
- or a second convenience truth model

then the milestone has failed.

## Product Decision Lock

- the main app-authoring lane must stop treating authored string ids as the
  primary identity mechanism
- internal signal identity must be runtime-owned, opaque, and generationally
  grounded
- handles, not user-authored strings, must be the primary authoring identity
  currency for ordinary app code
- opaque runtime identity must be non-portable and non-user-addressable; it is
  never serialized as the public identity of a graph artifact
- optional local debug names may exist for readability and diagnostics, but
  debug names are not identity
- public graph contract names remain explicit and deliberate
- explicit structural names remain required for portable/spec/compatibility
  lanes where names are the contract
- graph publication remains the first place where many users need to choose
  stable public names
- this milestone assumes we do not need backward compatibility for the current
  main-lane authored-id ergonomics; if the old lane obstructs a cleaner
  architecture, it should be removed rather than preserved
- controller composition must become lighter without ceasing to be controller
  composition
- mutation ergonomics must feel application-shaped while still lowering to one
  canonical runtime mutation envelope
- this milestone must not create a second easy runtime, second graph engine, or
  adapter-local shadow truth model
- this milestone intentionally excludes a docs-journey overhaul as a primary
  scope item; it is about ergonomic architecture and API shape first

Normative consequence:

- any solution that merely adds more id overloads without changing identity
  ownership is out of spec
- any solution that makes ids optional but still treats them as the true
  internal identity when present is out of spec
- any solution that keeps ordinary `input` / `computed` / `output` authoring on
  the same `id`-first overload family and relies only on docs or taste to steer
  callers toward the opaque lane is out of spec
- any solution that keeps the current main-lane authored-id ergonomics alive
  primarily for compatibility anxiety is out of spec
- any solution that lets debug names silently become public contract names is
  out of spec
- any solution that exports or restores opaque runtime identity as if it were a
  portable structural name is out of spec
- any solution that makes simple code easier by weakening graph/public/export
  truth is out of spec

## Architectural Model

### Identity categories

This milestone freezes four distinct identity categories:

1. **Runtime identity**
   - opaque
   - runtime-owned
   - generationally grounded
   - never authored manually on the normal app lane
2. **Author debug name**
   - optional
   - human-facing
   - useful for diagnostics and local readability
   - not required to be globally unique
   - never treated as contract identity
   - never a globally addressable mutation or query key
3. **Public contract name**
   - explicit
   - graph-owned
   - chosen only when something crosses a graph boundary
4. **Portable structural name**
   - explicit
   - used where recipe/spec/compatibility surfaces require stable named
     structure

Type consequence:

- the package surface should eventually expose distinct branded categories or
  equivalent proof-bearing wrappers for:
  - opaque authored handles
  - public contract names
  - portable structural names
- compile-time surfaces must not accept a debug name where a public contract
  name is required
- compile-time surfaces must not accept opaque runtime identity as a portable
  import/export key
- compile-time surfaces must not offer debug-name-addressed mutation or
  contract lookup APIs on the main lane

The main ergonomic improvement is not "optional ids." It is that the runtime
owns category 1 fully, while users choose category 3 only when they actually
publish a boundary.

### Main-lane authoring model

The desired authoring direction is:

```ts
const signals = createSignals();

const server = signals.input<Item | null>(null);
const draft = signals.input<Partial<Item>>({});
const effective = signals.computed(() => ({
  ...(server() ?? {}),
  ...draft(),
}));
const dirty = signals.computed(() => Object.keys(draft()).length > 0);
```

Optional debug names may exist:

```ts
const draft = signals.input<Partial<Item>>({}, { debugName: "draft" });
const dirty = signals.computed(() => Object.keys(draft()).length > 0, {
  debugName: "dirty",
});
```

But these debug names do not own identity, uniqueness, publication, or export.

They may support diagnostic filtering or local display, but they must never
become a globally addressable query or mutation vocabulary. In other words:

- a debug name may be searchable in diagnostics
- a debug name may be shown in explanations and inspector UIs
- a debug name may not become the stable key for `get`, `set`, `patch`,
  `reset`,
  export, import, or public graph contract lookup
- duplicate debug names must remain a normal, harmless condition

### Boundary publication model

Graph publication remains explicit and deliberate:

```ts
const itemDetailGraph = createSignals().graph("itemDetail", (graph) => {
  const editSession = createEditSessionController(graph.scope("editSession"));

  return graph.expose({
    inputs: {
      serverItemData: editSession.inputs.server,
      draftEdits: editSession.inputs.draft,
    },
    outputs: {
      effectiveItemData: editSession.outputs.effective,
      dirtyState: editSession.outputs.dirty,
    },
  });
});
```

That is where:

- public contract names become explicit
- authority classes become explicit
- export/import naming becomes explicit

Public input contracts should also become more precise than one undifferentiated
bag. The intended direction is that graph-owned public inputs can express
requiredness and optionality explicitly at the boundary, for example through
required and optional public-input entry forms or equivalent type-bearing
declaration shapes.

That means the graph contract should eventually be able to say, mechanically:

- this public input is required
- this public input is optional
- this public input is writable, read-only, or imported

without forcing later forms/resources work to reinvent those categories.

### Portable/spec lane model

This milestone does **not** erase names from:

- `computedSpec(...)`
- `outputSpec(...)`
- compatibility definitions
- import/export graph artifacts
- any other lane where structural portability depends on stable names

The package should therefore expose two honest identity postures:

- **ergonomic app lane**
  - opaque runtime identity
  - optional debug names
  - explicit names only at public boundaries
- **portable/spec lane**
  - explicit structural names remain required

Addressability rule:

- local app-lane addressing must happen by handle
- graph-boundary addressing must happen by explicit public contract name
- portable/spec addressing must happen by explicit structural name
- debug names may participate only in diagnostic search/filter helpers that
  return ambiguity explicitly and always surface the real identity categories
  beside the debug name

Import/export rule:

- exported definitions and snapshots may describe public contract names and
  portable structural names only
- opaque runtime identity may support same-process diagnostics and runtime-owned
  equivalence checks, but it is not a serialized public contract
- imported graphs must never require or expose authored opaque ids as the
  caller's addressing vocabulary

### Ergonomic improvement categories

This milestone intentionally focuses on four ergonomic categories:

1. opaque internal identity
2. lighter controller/graph authoring
3. application-shaped mutation ergonomics
4. linked writable derived-state primitives

It explicitly does **not** treat docs-story redesign as the main milestone
deliverable, because the user already excluded that category from scope.

## Phases

Phase ordering is strict. No phase may begin implementation until the prior
phase's code shape, type shape, and proof obligations exist. This milestone is
not an any-order buffet.

### Phase 1: Freeze The Opaque Identity Law

Purpose:

- define the non-negotiable identity model before any helper API is built

Required work:

- define runtime-owned opaque signal identity as the primary authored identity
- define the exact distinction among runtime identity, author debug name,
  public contract name, and portable structural name
- define where explicit structural naming remains legal and required, and where
  authored `id` on the app lane becomes forbidden
- define the cutover posture from the current main-lane authored-id surface to
  opaque internal identity with no requirement to preserve the old ergonomic
  call forms
- define how diagnostics are allowed to surface opaque identity without leaking
  unstable implementation nonsense into product docs
- define the eventual compile-time categories that prevent `debugName`, public
  contract name, portable structural name, and opaque runtime identity from
  collapsing into one stringly bag
- define the explicit rule that debug names may support diagnostics search but
  never globally addressable query or mutation APIs

Concrete target shape:

- normal app-lane authoring uses handle identity plus optional
  `{ debugName }`
- graph exposure names public contracts through explicit object keys or explicit
  public-name wrappers
- portable/spec surfaces continue to require named definitions through distinct
  APIs rather than ambient reuse of local authoring overloads
- the current app-lane `id`-centric overloads are expected to disappear from
  the primary surface rather than being treated as equal-status historical
  alternatives

Exit criteria:

- the package has one naming law for the ergonomic lane and one for the
  portable/spec lane
- no later phase needs to rediscover what `id`, `debugName`, and public
  contract name mean
- later phases have a frozen target for which API families may accept explicit
  structural names and which ones may not

Must not start Phase 2 until:

- the spec names where explicit authored `id` is removed from the app lane and
  where explicit structural naming still survives
- the spec names the compile-time distinction the surface is expected to encode

### Phase 2: Introduce Opaque Internal Identity In The Callable Surface

Purpose:

- make local signal authoring work without explicit ids while preserving
  runtime truth and proof-bearing identities

Required work:

- add id-less authoring forms for local `input`, `computed`, and `output`
- ground internal identity in runtime-owned generational identity
- expose optional debug names as non-authoritative metadata only
- ensure handles remain the primary composition currency
- prevent ordinary app-lane authoring from accidentally falling back onto the
  old string-identity overload family
- replace the current main-lane authored-id ergonomics rather than carrying
  them forward as equal-status alternatives
- preserve explicit naming only in the distinct public-boundary and
  portable/spec lanes where names are structurally required

Concrete target shape:

```ts
const count = signals.input(1);
const displayText = signals.computed(() => `${count()}`);

const named = signals.spec.input("count", 1);
const specOutput = signals.spec.output("summary", () => ...);
```

The exact surface spelling may change, but the code shape must preserve one
important constraint:

- ordinary local authoring and explicit named/spec authoring must become
  visibly distinct API families, not two equally primary overload clusters on
  the same call
- debug names remain metadata, such as
  `signals.input(1, { debugName: "count" })`, not identity
- opaque authored handles must be the only thing ordinary local composition
  depends on
- no main-lane API may let callers address that signal later by `"count"` just
  because a debug name was present

Exit criteria:

- ordinary local app code can author signals without ids
- repeated local authoring does not collide
- the ordinary app lane no longer relies on authored string ids
- explicit structural naming exists only in the APIs whose contract genuinely
  requires names
- compile-time surfaces no longer encourage accidental string identity on the
  main lane

Must not start Phase 3 until:

- local handles compose without user-authored ids
- explicit naming has a segregated surface shape instead of a doc-only warning
- the current main-lane authored-id call shapes are no longer the preferred or
  co-equal path the implementation must carry forever

### Phase 3: Rebuild Controller Authoring Around Opaque Identity

Purpose:

- make controller composition lighter without weakening controller contracts

Required work:

- add a lighter controller builder or equivalent condensation layer
- preserve branded controller artifacts and internal/public distinctions
- allow controller-local authoring without explicit ids on the normal lane
- ensure controller internals still map to stable runtime-owned opaque identity
- keep public contract naming explicit only when graphs expose controller
  members

Concrete target shape:

```ts
const editSession = graph.controller("editSession", ({ input, computed }) => {
  const server = input<Item | null>(null);
  const draft = input<Partial<Item>>({});
  const effective = computed(() => ({ ...(server() ?? {}), ...draft() }));

  return {
    inputs: { server, draft },
    outputs: { effective },
  };
});
```

or an equivalent builder that is just as explicit about:

- controller artifact construction
- internal vs public contract categories
- handle-based local authoring

The phase is not satisfied by "return a plain object and brand it later."

Exit criteria:

- controller code is structurally lighter than the current ceremony-heavy lane
- controller artifacts remain real package-understood units
- the lighter builder still produces mechanically validated controller artifacts

Must not start Phase 4 until:

- controller builders prove they can preserve controller artifact enforcement
- no controller helper relies on local string ids as its internal addressing
  mechanism

### Phase 4: Add Graph-Boundary Naming And Mutation Condensation

Purpose:

- keep graphs explicit where they should be explicit, while making graph use
  feel less like runtime paperwork

Required work:

- preserve explicit graph `inputs` / `outputs` naming as the public boundary
- add lighter graph exposure helpers where they reduce repeated boilerplate
- preserve authority classes on public inputs
- condense graph-native input operations into more app-shaped helpers while
  keeping one canonical mutation envelope underneath
- allow graph transactions and patch/reset flows to feel CRUD-shaped

Concrete target shape:

```ts
const graph = createSignals().graph("itemDetail", (graph) => {
  const edit = createEditSessionController(graph.scope("editSession"));

  return graph.expose({
    inputs: {
      serverItemData: graph.input.required(edit.inputs.server),
      draftEdits: graph.input.optional(edit.inputs.draft),
    },
    outputs: {
      effectiveItemData: edit.outputs.effective,
      dirtyState: edit.outputs.dirty,
    },
  });
});

graph.writeInputs({ draftEdits: { name: "Ada" } });
graph.patchInputs({ draftEdits: { done: true } });
graph.resetInputs(["draftEdits"]);
```

or ergonomic aliases that lower to the same canonical graph mutation envelope.

The phase is not satisfied by scattering a bag of helper methods whose
relationship to `apply(...)` has to be rediscovered in every implementation.

Public input contracts in this phase must be capable of carrying, at minimum:

- public name
- required vs optional status
- authority class
- handle binding

Exit criteria:

- graph publication remains explicit
- graph mutation ergonomics improve without creating a second write engine
- graph helpers lower through one visibly canonical mutation plan/envelope

Must not start Phase 5 until:

- graph boundary naming is still explicit under the new ergonomic lane
- mutation helpers have a single canonical lowering path

### Phase 5: Linked Writable Derived-State Primitive

Purpose:

- add one principled dependent-writable-state primitive without prematurely
  building the full forms product

Required work:

- add a linked writable derived-state primitive or equivalent surface for state
  that normally follows a reactive source but may be locally overridden and
  re-anchored
- allow the linked-state primitive to account for previous value where that is
  required to preserve still-valid user intent under source change
- keep this primitive as a consumer of controller/graph/runtime truth, not as a
  separate state model

Concrete target shape:

- helpers must compile to controller artifacts, graph-native operations, or
  both
- helpers may return a structured helper artifact, but that artifact must
  expose its authority and derivation categories explicitly
- helpers must not invent a hidden lifecycle, hidden validation engine, or
  hidden submit state model that later forms work would have to undo

Illustrative target shape:

```ts
const selectedOption = signals.linked(
  () => shippingOptions()[0],
  {
    debugName: "selectedOption",
  },
);

const preservedSelection = signals.linked({
  source: () => shippingOptions(),
  computation: (options, previous) =>
    options.find((option) => option.id === previous?.value?.id) ?? options[0],
  debugName: "preservedSelection",
});
```

The exact surface spelling may evolve, but the semantic requirements are:

- writable state may remain linked to upstream reactive state
- source changes may reset or re-anchor that writable state
- previous value may be consulted explicitly when preserving still-valid local
  intent
- this primitive must still reduce to the same runtime-owned authority and
  derivation model rather than inventing a private mini-store

The phase is not satisfied by shipping convenience objects whose semantics are
only explainable by reading their implementation.

Exit criteria:

- dependent writable state becomes shorter and less ceremonial
- the primitive does not become a proto-forms engine with private semantics
- the primitive remains mechanically reducible to the existing controller /
  graph / mutation substrate
- linked writable derived state exists as a principled primitive instead of
  being reinvented ad hoc in helper families

Must not start Phase 6 until:

- linked writable derived-state artifacts can be explained entirely in terms of
  existing authority and derivation categories
- the primitive has not become a stealth second store or stealth forms engine

### Phase 6: Diagnostics, Export, And Equivalence Hardening

Purpose:

- prove that the ergonomic lane is simpler without changing truth

Required work:

- make diagnostics explain opaque runtime identity and public boundary naming
  coherently
- preserve export/import/public contract naming truth
- certify equivalence between:
  - direct opaque ergonomic authoring
  - controller-composed opaque authoring
  - controller-composed graph publication
- add package-proof coverage so the tarball certifies the ergonomic lane too
- certify that exported/imported artifacts never require opaque runtime identity
  as caller vocabulary

Concrete target shape:

- diagnostics may show opaque runtime identity, debug names, public contract
  names, and portable structural names together
- each of those categories must remain separately named in the artifact model
- imported graphs must be addressable through public contract names and other
  explicit portable names only

Exit criteria:

- the ergonomic lane is certified as a thinner syntax over the same runtime and
  graph truth
- the import/export surface remains honest about what is and is not portable

## Must Ship

- runtime-owned opaque internal identity for the main app-authoring lane
- optional author debug names that are explicitly non-authoritative
- debug names restricted to readability and diagnostics search rather than globally
  addressable identity
- explicit preserved public contract naming at graph boundaries
- explicit preserved structural naming for portable/spec lanes
- id-less `input`, `computed`, and `output` authoring on the main lane
- removal of authored string-id requirements from the normal app lane
- lighter controller authoring on top of real controller artifacts
- lighter graph exposure and mutation ergonomics on top of real graph
  contracts
- required and optional public input contract forms at graph boundaries
- a linked writable derived-state primitive on the main lane
- diagnostics-visible identity truth for opaque handles and public boundary
  names
- export/import naming truth preserved under the ergonomic lane
- package-proof and runtime-proof equivalence between ergonomic and explicit
  authoring

## Must Preserve

- runtime truth remains runtime-owned
- graph/public/export truth remains explicit and stable
- controller contracts remain real package-understood artifacts
- public graph boundaries remain the authority for public naming
- portable/spec lanes remain explicit where names are the actual contract
- one canonical graph mutation envelope remains the underlying write path
- this milestone must not become a forms implementation by stealth

## Acceptance Evidence

This milestone is complete only when all of the following are true:

- local signal authoring no longer requires explicit ids on the main lane
- opaque internal identity never collides under repeated local/controller use
- public graph inputs/outputs still have explicit, stable names
- diagnostics can explain opaque runtime identity, debug names, and public
  contract identity separately
- debug names never become the only globally queryable string for authored
  signals
- export/import and restore preserve public contract truth under ergonomic
  authoring
- graph-native mutation ergonomics remain semantically identical to the
  existing mutation substrate
- required and optional public input contracts remain explicit and type-visible
- linked writable derived state remains authority/derivation-honest under
  source change and override flows
- direct opaque authoring, controller-composed opaque authoring, and
  graph-published opaque authoring converge to the same committed truth, replay
  truth, and diagnostics truth

## Required Named Test Families

- `The Opaque Local Identity Collision Test`
  Proves id-less local authoring remains collision-safe under repeated
  controller and graph composition.
- `The Debug Name Is Not Identity Test`
  Proves duplicate debug names do not collide and never silently become public
  contract identity.
- `The Debug Name Is Not Addressability Test`
  Proves debug names cannot be used as stable mutation, export/import, or graph
  contract lookup keys, and that any diagnostics search by debug name returns
  ambiguity explicitly.
- `The Public Input Requiredness Contract Test`
  Proves required and optional public input forms remain distinct, type-visible,
  and operationally honest at the graph boundary.
- `The Opaque Authoring Equivalence Test`
  Proves direct opaque authoring, controller-composed opaque authoring, and
  graph-published opaque authoring converge to the same committed truth and
  diagnostics truth.
- `The Public Boundary Naming Truth Test`
  Proves public graph names remain explicit, stable, and export/import-honest
  even when internal authoring used no explicit ids.
- `The Portable Lane Explicit Naming Test`
  Proves spec/compatibility lanes still require explicit structural names where
  they genuinely own portability.
- `The Ergonomic Mutation Envelope Equivalence Test`
  Proves lighter patch/reset/set helpers lower to the same canonical mutation
  envelope as the existing graph/runtime surfaces.
- `The Linked Writable Derived State Test`
  Proves linked writable derived state preserves override intent honestly under
  source changes, previous-value-aware recomputation, and reset/re-anchor
  behavior without inventing a second authority model.

## Architectural Notes

- This milestone belongs after graph-owned lifecycle because it relies on
  controller artifacts and graph boundaries already being real.
- This milestone belongs before forms/resources because they should inherit
  better ergonomics rather than compensating for current ceremony.
- This milestone intentionally improves the complicated version too. The goal is
  not a toy shortcut; it is a thinner authoring layer over the same substrate.

## Sequencing Notes

The intended order becomes:

1. host capability closeout
2. composition API and graph publication
3. scoped controller identity and graph-owned lifecycle
4. opaque identity and ergonomic authoring
5. forms product surface
6. API surface

That order keeps forms/resources from becoming the first place where CRUD-scale
ergonomics are invented, while still leaving the actual forms product for the
later milestone the user explicitly wants to reserve.
