# WORTH Query Aspect API Finalization Plan

> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./worth_query_vision.md)
>
> **Companion closeout:** [runtime-api-public-stabilization-closeout.md](./runtime-api-public-stabilization-closeout.md)
>
> **Shipped closeout:** [aspect-api-finalization-closeout.md](./aspect-api-finalization-closeout.md)
>
> **Purpose:** freeze the public aspect-native authoring surface for writes and
> authoritative mutation before the deeper JSON-substrate removal begins, so
> downstream runtimes can build on WORTH Query now without deprecating their own
> code when `worth-relational`, `worth-store`, and the runtime bridge are
> rewritten under the hood.
>
> **Bias:** make the aspect model the executable public truth. If the current
> runtime seams still speak JSON internally, contain that fact behind the facade.
> Do not teach JSON as the authoring model, do not let payload shape define
> public semantics, and do not weaken the API to protect transitional internals.

## Goal

Finalize the public WORTH Query mutation API around aspect-native authoring,
receipts, and inspection before beginning the deeper storage and engine rewrite.

After this plan closes, ordinary downstream code should be able to:

- create and update truth through aspect-native declarations
- reason about mutation in the same language as live views, computeds, effects,
  branching, inspection, and support admission
- avoid `serde_json::Value` payload blobs in normal product-facing write code
- rely on a stable public facade while lower crates are free to replace the
  JSON-backed substrate

This plan intentionally freezes the public write and mutation authoring shape.
It does **not** remove JSON from the internal engine yet.

## Why This Plan Exists

WORTH Query now has a stabilized runtime-backed facade for live views,
computeds, effects, reads, observation, materialization, state, inspection,
branching, preview, and support admission. That public surface is close enough
to beautiful that downstream runtimes can start building on it.

The mutation side is the remaining mismatch.

Today the public runtime story is aspect-shaped almost everywhere:

- live views project aspects
- computeds declare `reads(...)` and `produces(...)`
- effects trigger from aspects
- receipts and invalidation route by touched meaning
- inspection and state vocabulary speak in aspect and lane terms

But ordinary inserts still teach a payload-first shape. That leaves three bad
outcomes:

- the public DX teaches a weaker mental model than the runtime actually uses
- generated runtime code would depend on a transitional JSON-shaped facade
- the later aspect-native substrate rewrite would either require public API
  churn or force long-lived compatibility baggage around the wrong abstraction

This plan exists to cut that knot now: stabilize the right public mutation
surface first, then let the deeper storage rewrite happen underneath it.

## Governing Summaries

- `MENTALITY.md`: the hard problem is the ontology mismatch between
  aspect-native semantics and JSON-shaped mutation authoring. Solve that
  foundation before building more runtime features on top.
- `arch_laws.md`: the facade is the only surface, authority and derivation must
  stay explicit, and invalid paths should fail mechanically rather than by
  convention. The public write API must align with aspects, lanes, and
  inspection rather than leaking substrate details.
- `perf_laws.md`: mutation, invalidation, and recomputation breadth must scale
  with semantic delta. Public write shapes must preserve touched-aspect meaning
  rather than hiding it inside opaque payload blobs.
- `domain_laws.md`: WORTH Query must stay domain-neutral. The plan can be
  pressure-tested against geometry, workflow, and table examples, but it must
  not introduce product-domain semantics into the query crate.
- `worth_query_vision.md`: Query is the daily-driver query framework where
  developer experience lives or dies. Public writes must speak the same aspect
  vocabulary as the rest of the framework.
- `worth_query_roadmap.md`: the guiding rule remains `declare query intent
  once, lower it once, execute it against canonical truth`. Public mutation
  authoring should follow the same rule instead of inventing a separate
  payload-first story.
- `test-requirements.md`: no mutation-surface claim is honest without canonical
  bundles, typed denials, adversarial lanes, and support-matrix enforcement.
- `runtime-api-public-stabilization-plan.md` and closeout: the stable workspace,
  handle, state, inspection, and support posture already exists. This plan must
  extend that public model instead of creating a sibling mutation model.

## Adversarial Constraint

An ordinary developer must be able to build a serious runtime-backed feature
against WORTH Query using aspect-native writes now, and that same code must
remain valid when the storage engine, runtime bridge, and store later become
fully aspect-native internally.

The public API must survive these hostile conditions:

- a workflow DSL generator emits hundreds of create/update operations across
  sections, groups, controls, validation nodes, and branch-local previews
- a geometry runtime emits invariant-preserving edits, preview branches,
  rollback-capable commits, and merge-ready receipts
- an Excel-grade table runtime emits frequent narrow updates, batched edits,
  formula-driven derived state, and undo/redo pressure
- all of those use the current runtime-backed Query facade before the internal
  JSON substrate is removed
- later lower-crate work replaces JSON-backed entity payload truth with
  aspect-native storage and diffing

If any of the following become likely, this plan has failed:

- downstream code must later rewrite aspect-native calls because the public API
  was only a thin wrapper over JSON payload semantics
- public authoring requires callers to think in `serde_json::Value` instead of
  aspect paths and typed aspect values
- the runtime cannot explain inserts, updates, commits, branch-local mutations,
  or touched-surface routing in aspect terms
- public writes and public reads speak two different ontologies
- JSON payload shape becomes part of the stable public contract
- later substrate work needs to preserve JSON-shaped public baggage just to keep
  downstream runtimes compiling

## Non-Negotiable Public DX

The final public write story should feel like aspect-native mutation authoring
that composes naturally with live views, computeds, effects, branching, state,
and inspection.

Illustrative shape:

```rust
let mut workspace = runtime.workspace("garage")?;

let receipt = workspace.insert("Car", |car| {
    car.aspect("identity.id", "car-1")
        .aspect("make.value", "Honda")
        .aspect("model.value", "Civic");
})?;

let rename = workspace.update("Person", "person-1", |person| {
    person.aspect("name.value", "Ava Chen");
})?;

let branch_receipt = branch.intent("apply-preview-rule", |intent| {
    intent.entity("Section", section_id)
        .aspect("validation.rule", new_rule);
})?;

let explanation = workspace.inspect(&receipt)?;
```

The exact surface names may adapt to local WORTH Query naming style, but the
public DX properties are non-negotiable:

- ordinary writes are authored in aspects, not JSON payload trees
- inserts, updates, batched edits, and intent-shaped writes stay inside the
  same aspect vocabulary
- authoritative mutation, branch-local mutation, preview-local mutation, and
  staged intent residue remain distinguishable by lane and inspection evidence
- receipts explain touched aspects, touched surfaces, and resulting commit
  posture
- support admission remains the gate for future or deferred mutation families
- JSON may exist behind the facade temporarily, but it is not the story users
  read, copy, or generate against

## Stable Public Concepts

The public mutation API must freeze these concepts as long-lived vocabulary.

### Mutation Family

The facade should distinguish at least these public mutation families:

- insert/create
- update existing entity aspects
- batched multi-entity mutation
- delete
- intent-shaped authority crossing
- effect-staged pending write intent

Family meaning must remain visible in the API shape. One generic
`write_anything(...)` escape hatch is not an acceptable public daily-driver
surface.

### Aspect Mutation Builder

Aspect mutation builders are the public authoring unit for entity changes.

Required properties:

- aspect paths are declared explicitly
- the builder can express multiple aspect writes in one entity mutation
- the builder can express typed removal/reset where supported
- the builder can later lower into native aspect-state writes without changing
  the public contract
- the builder must not require callers to assemble nested JSON trees

### Mutation Receipt

Receipts are the public canonical artifact for write consequences.

At minimum a receipt must carry or inspectably reference:

- mutation family
- target entity or collection identity
- touched aspect set
- touched live-view ids
- touched derived-view ids
- authority lane and basis lane
- commit / branch-local / preview-local posture
- support posture where relevant
- digest-bound inspection identity

### Mutation State And Lane Posture

Mutation outcomes must remain aligned with the stabilized public state and lane
model:

- authoritative writes are not the same thing as pending write intent
- branch-local truth is not the same thing as preview truth
- derived state is not authoritative truth
- future temporal or async neighbors must extend the same posture vocabulary

### Mutation Inspection

`inspect()` must explain public writes in the same vocabulary the user authored:

- declared aspects
- admitted family
- lane posture
- touched-surface routing
- denial reason where unsupported
- commit/branch/preview/intention evidence

Inspection is part of the mutation contract, not a debugging afterthought.

## Phases

### Phase 1: Freeze The Public Mutation Ontology

Purpose:

- explicitly declare that aspect-native mutation is the public truth
- forbid JSON payload shape from defining public semantics
- settle the public family vocabulary before implementation work hardens around
  provisional names

Must ship:

- a final naming decision for create/insert, update, batch mutation, delete,
  intent mutation, and staged effect intent consumption
- a public ontology statement that ordinary mutation authoring is aspect-native
- a lower-level or deletion posture for any remaining payload-first command
  names
- explicit statement that JSON may remain an internal lowering adapter during
  transition, but is not the public semantic model

Must preserve:

- no claim that lower-crate storage is already aspect-native
- no domain semantics inside WORTH Query
- no API names that hide orchestration boundaries or family meaning

Acceptance evidence:

- this plan is implemented as named public surfaces or explicit closeout debt
- docs and tests no longer present JSON payload blobs as the preferred public
  mutation story
- support/admission metadata recognizes the stabilized mutation families

Forbidden shortcuts:

- treating `serde_json::Value` payloads as the public write currency
- adding one generic dynamic `workspace.write(...)` overload that erases family
  meaning
- declaring the ontology in prose while leaving all golden examples
  payload-first

### Phase 2: Introduce Aspect-Native Public Builders Over The Existing Runtime

Purpose:

- ship the final public mutation authoring DX now
- allow the current runtime-backed implementation to lower aspect builders into
  the existing substrate while the deeper rewrite is still pending

Must ship:

- public aspect-native insert/create builder
- public aspect-native update builder
- public batch mutation surface where one semantic operation changes multiple
  entities or aspect groups
- aspect-native delete/clear surface where supported
- lowering from public aspect builders into the current runtime-backed write
  path without changing public semantics

Must preserve:

- the lowering layer remains internal or lower-level-scoped
- lower-runtime JSON plumbing is not reachable from ordinary public examples
- touched-aspect evidence is preserved through lowering

Acceptance evidence:

- ordinary public examples compile and execute without `json!(...)`
- compile-fail or visibility proof prevents new public code from relying on
  internal lowering helpers
- canonical receipts and inspection reflect aspect meaning, not payload shape

Forbidden shortcuts:

- public examples that build nested maps and call them "aspects"
- leaking transitional JSON helper types into the stable facade
- losing touched-aspect fidelity during lowering

### Phase 3: Align Receipts, Inspection, And State With Mutation Semantics

Purpose:

- make public mutation explainable in the same aspect/lane/basis vocabulary as
  reads, computeds, and effects
- ensure inserts are not second-class explanation surfaces compared with
  `UpdateAspect`

Must ship:

- stable inspection shape for create, update, batch, delete, and intent
  receipts
- touched-aspect reporting that survives both direct writes and lowered writes
- lane/basis evidence for authoritative, branch-local, preview-local, and
  pending-intent outcomes
- denial bundles for unsupported mutation neighbors

Must preserve:

- one canonical artifact drives downstream observation, routing, and
  explanation
- diagnostics richness cannot change operational meaning
- inspection remains read-only and proof-bearing

Acceptance evidence:

- tests assert meaningful touched-aspect, touched-surface, and lane evidence
- receipts can explain why a write woke one surface and not another
- typed denial bundles localize unsupported mutation family or support posture

Forbidden shortcuts:

- receipts that only say success/failure without semantic delta
- inspection that requires re-deriving meaning from substrate payloads at read
  time
- branch/preview writes that collapse into the same explanation shape as
  authoritative writes

### Phase 4: Replace Public Docs, Golden Examples, And Generated Reference Shapes

Purpose:

- make the stable public mutation story teach the right thing everywhere
- give AI and downstream runtime authors a clean aspect-native reference set

Must ship:

- updated product docs for:
  - workspace overview
  - writes and intent boundaries
  - minimal CRUD runtime example
  - any other public doc still teaching payload-first writes
- at least one golden CRUD/runtime transcript using aspect-native writes
- one workflow-shaped and one geometry-shaped example proving the write surface
  holds up under more than trivial CRUD

Must preserve:

- docs stay product-facing rather than milestone prose
- examples remain domain-neutral pressure tests, not Query-owned product logic
- deferred intent/async/store semantics remain explicitly gated

Acceptance evidence:

- no P0/P1 public feature doc teaches JSON blobs as the preferred write path
- examples prove real touched-surface routing and receipt meaning
- doc examples match executable public API names

Forbidden shortcuts:

- leaving JSON in public docs "temporarily"
- updating examples without updating the support and limits language
- teaching internal lower-level APIs as the preferred story

### Phase 5: Certification, Support Matrix, And Legacy-Surface Closure

Purpose:

- make the public mutation freeze honest and mechanically enforced
- mark exactly what is stable now versus what remains deferred

Must ship:

- support-matrix rows for aspect-native mutation families
- alternate-name or deletion list for payload-first public names and examples
- compile-fail or visibility tests preventing new public dependencies on
  transitional payload surfaces where possible
- certification suites for public mutation parity and denial behavior

Must preserve:

- method presence is not a support claim
- unsupported future neighbors fail typed and early
- stable runtime-backed mutation support does not overclaim store-backed,
  durable, temporal, or async mutation behavior

Acceptance evidence:

- admitted mutation families execute canonically
- non-admitted mutation combinations fail typed and early
- docs, capability advertisement, and executable admission behavior stay in
  sync

Forbidden shortcuts:

- silently preserving payload-first APIs as co-equal public paths forever
- support claims inferred from old method presence
- stable claims that outrun certification

### Phase 6: Closeout And Handoff To Substrate Rewrite

Purpose:

- produce the dependency contract downstream runtime work can safely build on
- create a clean handoff line before aspect-native storage work starts

Must ship:

- aspect API finalization closeout
- explicit "safe to build now" list for downstream runtimes
- explicit "must not assume yet" list covering internal substrate semantics,
  store-backed behavior, temporal, async, and durable restart
- migration notes for remaining lower-level surfaces

Must preserve:

- the public facade is stable even if lower crates churn aggressively next
- no claim that JSON has already been removed internally
- no claim that lower-runtime rewrites are optional if the substrate still
  remains payload-first

Acceptance evidence:

- targeted public mutation tests pass
- support matrix enforcement stays honest
- closeout answers the substrate-handoff questions explicitly

Forbidden shortcuts:

- calling the public surface finalized while docs still teach JSON
- calling the API finalized while receipts cannot explain aspect-native writes
- using the public freeze to postpone the deeper substrate rewrite indefinitely

## Must Ship

- final public aspect-native mutation vocabulary
- public create, update, batch, and delete authoring surfaces that do not
  require JSON payload blobs
- receipt and inspection contracts aligned with touched aspects, touched
  surfaces, authority lanes, and basis evidence
- support-matrix rows and typed denial behavior for admitted and deferred
  mutation families
- updated public docs and golden examples that teach aspect-native writes
- lower-level or deletion posture for payload-first public surfaces
- closeout guidance that downstream runtimes can cite before the deeper
  aspect-native storage rewrite begins

## Must Preserve

- WORTH Query remains the public facade, not the truth/storage authority
- `worth-relational`, `worth-store`, and the runtime bridge remain free to
  change internally after this plan closes
- one canonical write artifact continues to drive routing, observation, and
  explanation
- public mutation semantics stay aligned with aspect projection, computed
  dependencies, effect triggers, lane posture, and inspection
- temporal, async, store-backed, and durable semantics remain explicit later
  gates where not yet shipped

## Explicit Non-Goals

- removing JSON-backed truth storage from `worth-relational`
- redesigning `worth-store` around native aspect persistence
- redesigning the runtime bridge around canonical aspect deltas
- certifying store-backed parity or durable restart/reload semantics
- implementing temporal or async/resource mutation execution semantics
- embedding geometry, workflow, table, or DSL semantics into WORTH Query

Those are important next steps, but they are not this plan. This plan freezes
the public API that the deeper rewrite must preserve.

## Acceptance Evidence

This plan is complete only when WORTH Query can prove:

- ordinary public mutation examples no longer require `json!(...)` or
  `serde_json::Value` payload assembly
- aspect-native create and update paths produce canonical receipts and
  inspection evidence aligned with touched aspects and touched surfaces
- admitted mutation families are support-matrix-backed and certification-backed
- unsupported mutation neighbors fail typed and early
- lower-level surfaces are explicitly marked rather than silently remaining
  co-equal public paths
- downstream runtimes can build against the stable facade without depending on
  payload-shaped internals

Required verification should include:

- `cargo fmt -p worth-query`
- `cargo check -p worth-query --tests`
- targeted public mutation API tests
- targeted support matrix enforcement tests
- targeted compile-fail or visibility-boundary tests
- `cargo test -p worth-query`
- `git diff --check`

## Roadmap Placement

This plan belongs after the runtime API public stabilization closeout and
before the full internal aspect-native substrate rewrite.

It belongs here because:

- the public runtime-backed facade is now strong enough for downstream use
- the remaining public mismatch is mutation authoring, not live/computed/effect
  composition
- downstream runtimes want to start building now, especially for wasm and
  domain runtimes
- the internal storage and bridge rewrite should happen beneath a stable public
  mutation contract rather than in parallel with public API churn

This plan is therefore the last public-API freeze line before the deeper
`worth-relational` / `worth-store` / bridge rewrite begins.

## Architectural Notes

- Public writes must speak the same ontology as public reads.
- Aspects are the mutation contract, not just the invalidation contract.
- Mutation receipts are canonical artifacts, not convenience return values.
- JSON may temporarily survive as an internal lowering adapter, but it must not
  define the public API, docs, or examples.
- The facade must stay strong enough that lower crates can be broken and
  rebuilt underneath it without forcing downstream runtime churn.

## Self-Check

- Does this solve a real structural problem? Yes. It closes the remaining
  public mismatch between aspect-native runtime semantics and payload-first
  mutation authoring.
- Is the adversarial constraint load-bearing? Yes. It specifically protects
  downstream runtime code from future substrate churn.
- Does the plan preserve crate authority boundaries? Yes. Query freezes the
  public authoring surface while lower crates remain the replaceable semantic
  and storage authorities underneath it.
- Does the plan define proof obligations, not just feature work? Yes. It
  requires support metadata, typed denials, canonical receipts, inspection
  evidence, doc replacement, and legacy-surface closure.
- Could a competent engineer map this into honest modules, tests, and closeout
  artifacts? Yes. Each phase names the surface, boundary, and proof needed.
- Does this belong in the roadmap sequence? Yes. It is the public mutation
  freeze that should happen immediately before the deeper aspect-native storage
  rewrite.
