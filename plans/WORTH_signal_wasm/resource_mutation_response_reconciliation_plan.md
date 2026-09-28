# Resource Mutation Response Reconciliation And Detail Lenses Plan

> **Status:** Planned engineering spec
>
> **Roadmap parent:** [wasm_product_roadmap.md](./wasm_product_roadmap.md)
>
> **Prerequisite milestone:**
> [branch_merge_materialization_foundation_plan.md](./branch_merge_materialization_foundation_plan.md)
>
> **Branch-native substrate milestone:**
> [resource_response_lens_contracts_plan.md](./resource_response_lens_contracts_plan.md)
>
> **Predecessor feature closeouts:**
> [resource_response_auto_patching_remaining.md](./resource_response_auto_patching_remaining.md)
>
> **Resource certification parent:** [test-requirements.md](./test-requirements.md)

## Goal

Finish the resource response product surface so mutation responses, detail
resources, canonical server confirmations, creates, updates, removes, identity
migration, and multi-family reconciliation are as explicit and ergonomic as the
collection response lane.

Milestone 10 correctly made resource effects branch-native and made response
lenses lower topology into proof-bearing effect loci. The branch merge
materialization foundation then closes the discovered runtime/product gap
around merge policy exposure, executable merge intent, branch-local
materialization, and explicit fork basis. Those foundations still do not finish
the consumer-facing product story for writes and large details.

This milestone closes that gap.

The target outcome is:

- `api.url(...).response(...).create(...)`, `.update(...)`, and `.remove(...)`
  are first-class response-owned mutation lanes rather than denied future work
- mutation responses can explicitly reconcile detail, collection, paged,
  summary, and auxiliary read families
- detail resources support granular field, JSON path, and declared-region
  effect loci rather than only whole-response replacement
- creates can insert canonical server items into collection topologies with
  declared placement proof or explicit placement-unavailable/refetch posture
- updates can patch or replace related read-family truth without feature-local
  cache glue
- removes can delete from related read-family truth through declared topology
  proof or emit typed deletion-unavailable posture
- identity migration from temporary/client/draft ids to canonical server ids is
  proof-bearing and cannot corrupt related families
- partial and multi-family mutation responses reconcile through one canonical
  mutation-response effect plan
- every unsupported reconciliation path is labeled as a deferred product gap,
  not counted as ergonomic completion merely because it denies cleanly

This milestone is not a network transport milestone and not a UI callback
milestone. It is the product layer that makes server-confirmed write results
and detail-shaped read truth consume the same branch-native resource effect
model that collections already use.

## Why This Milestone Exists

The completed branch-native resource effects milestone and branch merge
materialization foundation ship a strong substrate:

- one canonical resource effect envelope
- branch-native optimistic lifecycle
- rollback and merge/rebase proof
- response-lens topology lowering
- JSON item aspects
- advanced collection topology loci
- whole-response detail and summary replacement
- proof-bearing merge policy exposure
- branch-local materialization planning
- aspect-aware plain object materialization
- explicit branch fork basis

But real application code still feels an asymmetry:

- read families can own response topology through `.response(...).list(...)`,
  `.paged(...)`, and `.detail(...)`
- collection item/aspect/JSON paths are pleasant and proof-bearing
- write families still do not own response topology in the same ergonomic way
- mutation responses do not first-class declare which read families they
  reconcile
- detail resources remain broad-grained, even when workflow/editor detail
  values are large enough that whole-response replacement is too blunt

That gap encourages exactly the product shape WORTH tries to eliminate:

- feature code manually trusts returned mutation values
- applications manually normalize server responses into local authority
- detail resources become giant opaque blobs
- create/update/delete reconciliation becomes route folklore
- server id migration and placement become bespoke feature logic
- refetch decisions live in app code instead of typed resource posture

This milestone exists so consumers do not get an elegant collection experience
and a substrate-shaped write/detail experience.

## Governing Source Summaries

- `MENTALITY.md`
  Protects hard-problem-first product closure. This spec must not let clean
  denial artifacts masquerade as completed ergonomics when the wanted product
  surface is still missing.
- `arch_laws.md`
  Protects dual write/read contracts, proof-bearing phase progression, typed
  error topology, lowered execution plans, and self-describing envelopes. A
  mutation response must structurally declare what read truth it confirms,
  replaces, patches, deletes, invalidates, or cannot reconcile.
- `composition_laws.md`
  Protects semantic compilation units. Mutation response planning, detail
  lenses, identity migration, placement, deletion, fallback, and diagnostics
  must not collapse into generic resource helpers or one mega response file.
- `domain_structure_laws.md`
  Protects authority boundaries. Mutation response authority, read-family
  authority, detail-lens topology, identity migration, derived diagnostics, and
  transport posture must remain structurally distinct.
- `perf_laws.md`
  Protects cost honesty. Detail-region updates, collection placement, identity
  migration, multi-family reconciliation, and refetch fallbacks must expose
  lookup, traversal, reconstruction, migration, and fanout counters.
- `web_runtime_spec.md`
  Protects runtime-owned truth and package surfaces as product layers over the
  runtime, not second semantic engines. This milestone must consume resource
  line and branch truth rather than inventing cache semantics in API glue.
- `wasm_product_roadmap.md`
  Protects sequencing. This milestone belongs after branch-native resource
  effects because it relies on effect envelopes, response-lens proof, and
  merge/rebase posture that Milestone 10 made available.
- `test-requirements.md`
  Protects hostile certification for resource/API behavior. This milestone
  must add named proof suites for mutation response reconciliation, detail
  lenses, identity migration, placement, deletion, fallback, and multi-family
  convergence.
- `api_surface_dx_plan.md`
  Protects the route-first ergonomic lane. Mutation response reconciliation
  must integrate with `url(...)` finalizers instead of forcing ordinary CRUD
  back to raw family declarations.
- `resource_response_lens_contracts_plan.md`
  Protects branch-native effect substrate and response topology lowering. This
  milestone extends that substrate to write responses and granular details
  without reopening the completed branch-native foundation.

## Adversarial Constraint

A long-lived workflow editor or data-heavy app with detail documents,
collection summaries, paged lists, grouped views, optimistic creates, server
assigned ids, canonical update responses, partial mutation payloads, deletes,
validation warnings, branch rollback, delivery packets, and route continuity
must be able to express every server-confirmed write result as a declared
mutation-response reconciliation plan.

If two semantically equivalent histories can produce:

- different detail line truth after a save response
- different collection membership after a create/update/remove response
- a temporary id that survives in one family while the canonical id appears in
  another
- manual feature code deciding whether a mutation response should replace a
  detail line, patch a collection item, update a summary, or refetch
- a broad detail replacement where a declared detail field, JSON path, or
  region effect was provable
- a create inserted into the wrong group, page, tree parent, connection edge,
  map key, or entity record
- a remove that hides deletion as invalidation or broad replacement when the
  topology could prove deletion
- partial server truth that silently overwrites unknown fields
- a multi-family write that updates one read family but leaves an equivalent
  related family stale without a typed fallback artifact
- rollback, replay, diagnostics, or merge evidence that cannot name the
  mutation response field and read-family locus it reconciled
- or an unsupported path that is counted as product closure merely because it
  throws a clean denial

then this milestone has failed.

## Product Decision Lock

- Mutation responses are first-class resource effects, not ad hoc returned
  values that feature code manually commits.
- Write response lenses are distinct authoring surfaces from read response
  lenses when the mutation payload shape differs from the read payload shape.
- A mutation route may reconcile zero, one, or many read-family targets, but
  every target must be declared explicitly.
- The normal route-first lane must support response-owned `.create(...)`,
  `.update(...)`, and `.remove(...)` finalizers when a response lens is attached.
- Detail resources must support granular declared effect loci:
  - field
  - JSON path
  - region/subtree
  - whole response
- Whole detail replacement remains legal, but it is not allowed to stand in for
  granular support when the spec claims the granular lane is complete.
- Create insertion requires declared placement proof or a typed
  placement-unavailable/refetch posture.
- Delete removal requires declared deletion proof or a typed
  deletion-unavailable/refetch posture.
- Identity migration is a proof-bearing effect family, not a caller-side
  normalization trick.
- Partial mutation responses must explicitly declare what they prove and what
  they leave stale, unknown, delivery-awaited, or refetch-required.
- Multi-family reconciliation must be atomic at the mutation-response plan
  boundary: either all declared targets admit, or the plan emits typed partial
  or unavailable posture before visible truth is mutated.
- Mutation response planning must stay a distinct proof-bearing phase from
  read-line patch execution. It is not permitted to smuggle multi-family write
  orchestration into ordinary line patch helpers, detail-line broad replace,
  or ad hoc route callbacks.
- Route-finalizer authoring state for response-owned writes must stay distinct
  from the existing direct-array and read-response lanes. The pleasant API may
  share vocabulary, but it must not pretend write reconciliation is a minor
  variation of collection patch authoring.
- Refetch and delivery-awaited fallbacks are first-class outcomes, not comments
  in feature code.
- Clean denial is necessary but not sufficient for milestone closure. A wanted
  product path is closed only when it has an admitted ergonomic happy path,
  hostile denials, diagnostics/history proof, and cost evidence.

Normative consequence:

- any implementation that makes app code manually decide how a mutation result
  updates canonical read lines is below the product bar
- any implementation that leaves detail resources whole-response-only while
  claiming detail lens closure is out of spec
- any implementation that treats temporary-to-canonical id migration as a
  feature-local map rewrite is out of spec
- any implementation that applies mutation reconciliation to one family without
  either updating, invalidating, or explicitly declining related declared
  families is out of spec
- any implementation that treats mutation-response reconciliation as a hidden
  extension of one existing line patch path is out of spec
- any implementation that overloads read response lens proof to mean mutation
  payload authority without a distinct mutation-response proof boundary is out
  of spec
- any implementation that counts typed unavailability as ergonomic completion
  without naming the deferred happy path is out of spec

## Architectural Model

### Authority Split

1. **Mutation response authority**
   - owns the admitted server response payload for a create, update, remove,
     action, or server confirmation
   - owns response-field proof, correlation with the submitted mutation, and
     mutation-response plan identity
   - does not directly mutate read-family truth
2. **Read family authority**
   - owns family identity, line identity, visible truth, lifecycle, freshness,
     diagnostics, history, branch posture, and delivery basis
   - admits reconciliation only through proof-bearing mutation-response plans
3. **Detail topology lowering**
   - owns field, JSON path, region, and whole-response replacement proof for
     detail-shaped resources
   - feeds resource effect loci; it does not own mutation intent or lifecycle
4. **Placement and deletion topology lowering**
   - owns where a created item can be inserted or an existing item can be
     removed inside a collection topology
   - emits placement/deletion proof or typed unavailable/refetch posture
5. **Identity migration planner**
   - converts temporary, client, draft, or imported ids into canonical server
     ids through a branch-native migration effect
   - binds every affected detail, collection, selection, summary, and
     diagnostics reference it claims to update
6. **Fallback and freshness planner**
   - owns explicit `refetchRequired`, `deliveryAwaited`,
     `placementUnavailable`, `deletionUnavailable`,
     `identityMigrationUnavailable`, and `partialReconciliation` artifacts
   - must not silently replace missing reconciliation with broad mutation
     folklore
7. **Per-target execution lowering**
   - consumes an admitted mutation-response plan and lowers it into exact
     read-family execution artifacts
   - may reuse existing line patch, delivery, invalidation, or replace
     mechanics after planning, but may not rediscover sibling targets,
     identity mappings, or mutation payload semantics during execution

### Mutation Response Plan

Every admitted mutation response must lower into a canonical
`mutation-response-reconciliation-plan` family before any read line changes.

This plan is an authority artifact, not a convenience callback. It must exist
before any line-level execution helper runs and must be rich enough that
execution can stay mechanical:

- planners decide which targets exist, which payload fields they consume, and
  whether the mutation admits exact reconciliation, partial reconciliation,
  invalidation, refetch, delivery-await, or denial
- execution applies the already-admitted target operations and records branch,
  rollback, and diagnostics truth
- execution is not allowed to discover additional targets, broaden payload
  meaning, or reinterpret mutation intent on the fly

The plan must carry:

- mutation route identity and request correlation
- submitted mutation effect id and previous optimistic effect id where present
- mutation response lens proof
- response payload digest and response field/region digest evidence
- target read-family declarations and line identities
- target operation per family:
  - replace detail
  - patch detail field
  - patch detail JSON path
  - patch detail region
  - replace collection item
  - insert collection item
  - delete collection item
  - patch collection summary
  - invalidate/refetch
  - await delivery
  - no-op with proof
- identity migration mapping when canonical ids differ from submitted ids
- placement, deletion, and migration posture
- atomicity posture across multiple targets
- rollback and inverse posture
- merge/rebase posture and conflict granularity
- diagnostics/history compact facts
- cost counters for response extraction, target lookup, topology traversal,
  reconstruction, target fanout, migration fanout, and fallback breadth

The first implementation of this plan should assume multiple downstream target
effects even when the happy path starts with one. A write response that only
touches one detail line today still belongs to the same canonical artifact
family as create-with-placement, delete-with-summary, or multi-family save.

This plan is the canonical artifact for diagnostics, history, rollback, replay,
branch merge, and UI-observable lifecycle facts. Feature code must not need to
remember that "this write returns canonical workflow detail" outside the
declaration.

### Response Lens Families

This milestone introduces two related but distinct lens families.

1. **Read response lenses**
   - describe the shape of canonical read-family truth
   - already exist for collection, paged, detail, summary, JSON item aspects,
     and advanced topology loci
2. **Mutation response lenses**
   - describe the shape of server responses returned by write routes
   - may be identical to a read response lens or may wrap the canonical value
     inside operation metadata, warnings, validation payloads, placement data,
     or delivery hints

These families may share topology vocabulary where the semantics truly match,
but they must not share authority blindly:

- read response lens proof names what a read-family value looks like and what
  read-side loci it can lower into
- mutation response lens proof names what a write response payload proves, what
  response fields or regions it exposes, and what reconciliation planning facts
  can be derived from it
- when a write response is structurally identical to a read response, the API
  may allow an explicit reuse path, but the lowered proof artifacts must still
  record that a mutation-response boundary was crossed

The product surface must make both cases natural:

```ts
const workflowDetail = signals.resource.response.detail<Workflow>();

const saveWorkflow = api
  .url("/workflows/:workflowId")
  .response(workflowDetail)
  .update({
    submit,
    reconciles: workflowDetailLine.replaceFromResponse(),
  });
```

and:

```ts
const saveResponse = signals.resource.mutation.response<{
  workflow: Workflow;
  validation: ValidationWarning[];
  version: string;
}>()({
  detail: (value) => value.workflow,
  warnings: (value) => value.validation,
});
```

The exact API may change during implementation, but the semantic split may not:
read topology and mutation payload topology are not the same authority.

### Detail Locus Vocabulary

Detail resources must admit declared granular loci:

- `detailResponse`
  Whole detail replacement.
- `detailField`
  One named top-level field or typed field projection.
- `detailJsonPath`
  A JSON path inside one detail field with the same hostile path, optional
  terminal, accessor-denial, identity, rollback, and cost discipline as JSON
  item aspects.
- `detailRegion`
  A named subtree or region such as `workflow.metadata`, `workflow.nodes`,
  `workflow.edges`, `workflow.permissions`, or `workflow.validation`.

Detail region declarations must define:

- how to read the region
- how to replace the region
- whether identity is inside or outside the region
- what merge/rebase granularity the region maps to
- what traversal and reconstruction cost the region implies

Granular detail loci are first-class proof families, not aliases for broad
replacement with extra metadata. Each declared field/path/region lane must own:

- its own capability row and effect-locus identity
- its own rollback or typed-unavailability posture
- its own diagnostics/history vocabulary
- its own traversal and reconstruction counters
- explicit identity-preservation rules when the region excludes the canonical
  object identity

Whole-response detail replacement remains available, but the docs and closeout
matrix must distinguish it from granular detail support.

### Placement, Deletion, And Refetch Posture

Create and remove reconciliation cannot be treated as item update with a
different verb.

Create placement must declare one of:

- exact index or exact topology position
- server ordering key insertion
- append/prepend policy
- group/page/tree/connection/entity/map placement hook
- placement supplied by mutation response field
- placement unavailable and refetch required
- placement unavailable and canonical delivery awaited

Remove deletion must declare one of:

- remove exact item from known topology location
- remove by canonical id from all declared visible locations
- tombstone or soft-delete aspect update
- deletion unavailable and refetch required
- deletion unavailable and canonical delivery awaited

Every placement or deletion posture must name its cost and failure mode.

### Identity Migration

Identity migration is required when a mutation response proves that the
canonical server identity differs from the submitted local identity.

Supported cases must include:

- optimistic create temp id to server id
- draft id to published id
- cloned resource id
- imported external id to canonical local id
- server-assigned child/node ids inside a detail document

Migration must be declared as a branch-native effect that can update every
declared target or emit typed partial/unavailable posture. It must not be a
caller-side map rewrite hidden behind response normalization.

## Phase Deliverable Standard

Every phase must ship:

- at least one public or internal type encoding the phase proof
- at least one planner/lowering/admission function that consumes the previous
  proof and produces the next proof
- at least one runtime-visible artifact in diagnostics, history, verification,
  lifecycle, or denial output
- hostile runtime tests for success and no-side-effect denial
- type-smoke or compile-denial coverage when public capability is exposed
- breadth/cost counters for every operation that can touch more than one
  response field, family line, topology container, identity mapping, or
  diagnostic artifact

Docs may describe the phase, but docs do not close the phase.

## Implementation Guidelines

The phases below are a dependency spine, not a menu. Implementations must move
through them in order because each phase produces proof the next phase consumes:

1. mutation response plans create the authority boundary
2. detail lenses create the missing target loci
3. update/save reconciliation proves canonical write-result replacement
4. identity migration proves id changes before placement depends on them
5. create placement consumes identity migration and topology placement proof
6. remove/delete consumes topology deletion proof
7. partial and multi-family reconciliation composes the single-target cases
8. diagnostics/history/rollback/merge closes the proof surface across time
9. product closeout certifies docs, types, examples, and denial honesty

Practical implementation rules:

- Start each phase by adding the phase's proof type and denial artifact before
  adding pleasant authoring helpers.
- Add one narrow happy path and one no-side-effect hostile denial before
  broadening topology coverage.
- Keep authoring, planning, topology lowering, execution, diagnostics, and
  verification in separate files named for those responsibilities.
- Do not add a public method until the runtime has an admitted path or a typed
  unavailable artifact for that method.
- Do not count an unavailable artifact as closing a wanted product path unless
  the phase explicitly says the unavailable posture is the intended product
  result for that topology.
- Every new route helper must have an equivalent raw-family or lower-level
  declaration proof so the pleasant lane cannot become a second resource model.
- Do not let response-owned `.create(...)`, `.update(...)`, or `.remove(...)`
  lower straight into plain detail-family write truth with ad hoc callbacks.
  They must lower into mutation-response plan authority first, even when the
  eventual target is one detail line.
- Do not teach existing line patch or delivery helpers to rediscover mutation
  payload meaning, sibling targets, placement, deletion, or identity migration.
  Those helpers may execute planned work, but the planning authority must live
  upstream in named mutation-response modules.
- Treat route-builder state, mutation-response proof, target planning,
  per-target lowering, and read-family execution as separate file owners. If a
  file starts owning both "which targets does this write affect?" and "how does
  one line patch execute?", the seam is in the wrong place.
- Preserve honest capability surfaces while the milestone is in flight. Detail
  lines should not suddenly overclaim generic `patch(...)` or
  `reconciliation()` just because mutation-response reconciliation exists; any
  new detail-granular capability must arrive with its own declared proof lane.
- Every target reconciliation helper must answer these questions in tests:
  - what response field or payload region did it consume?
  - which read-family line did it target?
  - what exact locus changed?
  - what happened if the target was stale, missing, unloaded, or ambiguous?
  - what did rollback or fallback preserve?
  - how much lookup, traversal, reconstruction, and fanout work occurred?

Recommended file ownership:

- mutation response declaration and authoring belong under a mutation-response
  response/declaration area, not in generic API route helpers
- target reconciliation planning belongs under resource effect planning
- detail field/path/region topology belongs beside response topology lowering,
  but not in collection topology files
- identity migration belongs in its own planner/execution files because it is
  neither ordinary placement nor ordinary item replacement
- placement and deletion topology belong in topology-specific files that can
  name direct-array, map, entity-store, grouped, sparse, connection, tree, and
  detail-child behavior separately
- fallback/refetch/delivery-awaited artifacts belong in freshness or fallback
  planning files, not in docs or UI-facing helpers
- verification packages belong in closeout/proof files, not in the hot-path
  execution modules

Minimum practical examples the docs and tests must exercise:

- save workflow detail, return canonical workflow, replace the detail line
- save workflow node, return `{ node, graphVersion }`, patch a detail region
  and a list summary
- create workflow from draft id, return server id, migrate identity, insert in
  workflow list, and materialize detail
- create task in a grouped list with server-provided group placement
- delete task with exact removal and summary decrement
- delete workflow with detail invalidation and list removal
- update item where the relevant paged/search result is unloaded and the plan
  emits refetch-required instead of pretending it patched
- server accepts mutation but declares canonical delivery will follow, yielding
  delivery-awaited posture

## Phases

### Phase 1: Mutation Response Plan Authority

Purpose:

- create the canonical mutation-response reconciliation artifact before adding
  individual target types

This phase must ship:

- sealed mutation response declaration and lowered reconciliation plan types
- mutation response lens proof artifacts distinct from read response lens proof
- explicit target family references and target line identity proof
- route-owned `.response(...).create(...)`, `.update(...)`, and `.remove(...)`
  surfaces that lower to denied or admitted mutation-response plans rather
  than disappearing as unsupported methods
- atomicity posture for zero, one, or many reconciliation targets
- typed fallback posture for refetch, delivery-awaited, partial
  reconciliation, and unsupported targets
- diagnostics/history facts derived from the mutation-response plan
- planning and execution breadth counters
- explicit route-builder state and lowering boundaries for response-owned write
  finalizers so the write lane does not masquerade as the existing direct-array
  or single-response read lanes
- per-target execution artifacts derived from the plan rather than discovered
  during line mutation

Phase 1 gate:

- no detail, create, update, delete, placement, or identity migration phase may
  begin until mutation responses lower into one canonical plan that can deny
  before read-line mutation and can explain every target it intends to touch.
- no implementation may satisfy Phase 1 by directly extending one existing
  line-patch effect plan with extra mutation fields. The mutation-response plan
  must be a distinct authority artifact with distinct proof and counters.

### Phase 2: Detail Field, JSON Path, And Region Lenses

Purpose:

- make detail resources granular enough for large editor/workflow documents

This phase must ship:

- `detailField` response loci with typed field reads and replacement
- `detailJsonPath` response loci with required/optional path policy,
  array-crossing policy, unsafe-segment denial, accessor-denial, non-JSON
  denial, identity preservation, rollback or unavailability, and path cost
  proof
- `detailRegion` response loci with named region declarations, replacement
  hooks, identity boundary policy, merge/rebase granularity, and region cost
  proof
- family-scoped detail patch helpers for local patches, delivery packets, and
  mutation-response reconciliation
- detail capability surfaces and verification artifacts that distinguish
  field/path/region support from whole-response support instead of widening
  every detail lane into generic patch capability
- whole-response detail replacement preserved as a separate broad effect
- hostile proof that broad replacement is not used when a declared detail locus
  can prove a narrower effect

Phase 2 gate:

- detail support is not considered complete until whole-response replacement,
  field patching, JSON path patching, and declared-region patching all have
  local, delivery, mutation-response, rollback, diagnostics/history, denial,
  and cost proof.
- detail granularity is not considered honest if the runtime still reports only
  broad `detailResponse` truth for field/path/region-capable declarations.

### Phase 3: Update And Save Response Reconciliation

Purpose:

- make canonical update/save responses replace or patch related read truth
  without feature-local normalization glue

This phase must ship:

- route-owned `update(...)` response reconciliation declarations
- detail replacement from returned canonical detail values
- detail field/path/region patching from partial mutation responses
- collection item replacement from returned canonical items
- summary patching from mutation response fields
- validation/warning mapping into typed resource diagnostics without mutating
  canonical value truth unless explicitly declared
- server confirmation classification that distinguishes preserved optimistic
  truth, consumed canonical truth, partial canonical truth, refetch required,
  and delivery awaited
- branch rollback and merge/rebase proof for update reconciliation targets

Phase 3 gate:

- a save/update route that returns canonical detail truth must be able to
  declare that truth as the replacement basis for the related detail line, and
  tests must prove no feature code needs to manually commit the returned value.

### Phase 4: Identity Migration Foundation

Purpose:

- make canonical id changes proof-bearing before create placement, update
  reconciliation, imports, clones, or draft publication depend on them

This phase must ship:

- temporary/client/draft/import id to canonical server id migration declarations
- submitted identity, response identity, and canonical identity proof artifacts
- migration target declarations for detail lines, collection lines, summaries,
  selection-like resource truth, and detail child ids
- stale migration denial when target line basis or submitted mutation basis no
  longer matches
- partial migration posture when explicitly allowed by the declaration
- typed migration-unavailable, refetch-required, and delivery-awaited artifacts
- rollback, replay, diagnostics/history, and merge/rebase proof for migration
  effects
- migration fanout counters for every target and reference rewritten

Phase 4 gate:

- no create placement phase may begin until id migration can prove the
  difference between submitted, temporary, draft, imported, and canonical
  identities and can either update every declared target or emit typed partial
  or unavailable posture.

### Phase 5: Create Response Placement

Purpose:

- make creates branch-native across canonical server identity, collection
  placement, detail materialization, and summaries

This phase must ship:

- route-owned `create(...)` response reconciliation declarations
- declared placement policy for direct arrays, object-items, custom
  collections, map-backed collections, entity stores, grouped collections,
  named/multiple collections, sparse pages, connections, discriminated tuples,
  recursive trees, detail-created children, and summaries where applicable
- canonical detail line creation or replacement from create responses
- collection insertion and summary updates from create responses
- typed placement-unavailable, migration-unavailable, refetch-required, and
  delivery-awaited artifacts
- rollback and merge/rebase proof for optimistic create plus canonical
  migration

Phase 5 gate:

- an optimistic create with a temporary id is not closed until server
  confirmation can migrate identity, update every declared read target, roll
  back exactly or with typed unavailability, and explain any target it cannot
  reconcile.

### Phase 6: Remove Response Deletion And Tombstone Reconciliation

Purpose:

- make remove/delete effects first-class instead of hidden invalidation or
  broad replacement folklore

This phase must ship:

- route-owned `remove(...)` response reconciliation declarations
- deletion proof for direct arrays, object-items, custom collections,
  map-backed collections, entity stores, grouped collections, named/multiple
  collections, sparse pages, connections, discriminated tuples, recursive
  trees, and detail child regions where applicable
- tombstone/soft-delete aspect posture when deletion means retained item with
  changed status
- detail-line removal, invalidation, or replaced-not-found posture where the
  deleted object has a detail resource
- summary patching for counts, status, version, and modified metadata
- typed deletion-unavailable, refetch-required, and delivery-awaited artifacts
- rollback and merge/rebase proof for optimistic deletion and canonical
  removal responses

Phase 6 gate:

- delete/remove is not closed until the product can distinguish exact deletion,
  tombstone update, detail invalidation, refetch-required, and delivery-awaited
  outcomes without feature-local cache code.

### Phase 7: Partial And Multi-Family Reconciliation

Purpose:

- make mutation responses that touch several read surfaces explicit and
  atomic enough for real feature modules

This phase must ship:

- partial response field mapping where mutation payloads contain only canonical
  fragments such as `id`, `version`, `updatedAt`, `status`, warnings, or
  server-derived metadata
- multi-target reconciliation declarations for detail lines, collection lines,
  paged lines, summaries, auxiliary reads, audit/history lists, and permission
  reads
- atomic all-or-none target admission where required
- typed partial admission where explicitly allowed
- per-target fallback posture:
  - reconciled exactly
  - reconciled partially
  - invalidated
  - refetch required
  - delivery awaited
  - unsupported by current declaration
- diagnostics/history that name every declared target and every target outcome
- cost counters for target fanout, target lookup, payload field extraction,
  topology traversal, reconstruction, and fallback breadth

Phase 7 gate:

- a mutation that affects multiple declared read families must not leave any
  declared target in an implicit stale state. Every target must be reconciled,
  invalidated, delivery-awaited, refetch-required, or explicitly declined with
  typed evidence.

### Phase 8: Mutation Diagnostics, History, Rollback, And Merge Closeout

Purpose:

- make mutation response reconciliation explainable across branch lifecycle,
  rollback, replay, and merge/rebase

This phase must ship:

- verification packages for mutation-response plans and per-target outcomes
- history entries derived from mutation-response plans rather than feature
  code
- rollback through exact branch restore, inverse target effects, or typed
  unavailable posture for:
  - detail field/path/region patches
  - detail replacements
  - collection item replacements
  - create insertions
  - identity migrations
  - deletion/tombstone effects
  - summary patches
  - multi-family reconciliation
- merge/rebase conflict projection from mutation response target loci to
  native branch evidence
- stale mutation response denial when the response no longer matches the
  submitted mutation, optimistic basis, target line basis, or identity mapping
- no-side-effect denial proof before value, diagnostics, lifecycle, branch, or
  history truth changes
- diagnostics summary surfaces that remain compact and do not materialize rich
  target replay by default

Phase 8 gate:

- mutation response reconciliation is not closed until rollback, replay,
  diagnostics, history, and merge/rebase can explain each reconciled target
  from the canonical plan without reading feature-local state.

### Phase 9: Product Closeout And Documentation Honesty

Purpose:

- prove this milestone is the complete write/detail counterpart to the
  collection response experience

This phase must ship:

- suite-0-style hostile convergence covering detail save, collection item
  update, create with identity migration, delete, partial response,
  multi-family reconciliation, refetch fallback, delivery-awaited fallback,
  rollback, branch restore, merge/rebase, diagnostics, and history
- public type surfaces for mutation response declarations, detail lenses,
  placement, deletion, identity migration, fallback, and multi-family target
  outcomes
- compile denials for overclaimed detail, create, update, remove, placement,
  deletion, identity migration, and multi-target capability
- product docs that teach:
  - save detail and replace detail line
  - update item and patch related collection/detail views
  - create item with placement and identity migration
  - delete item with exact removal or tombstone posture
  - partial response mapping
  - multi-family reconciliation
  - refetch/delivery fallback
- a closeout matrix that distinguishes:
  - supported ergonomic happy paths
  - supported precise denials
  - supported typed unavailable fallbacks
  - intentionally out-of-scope work
  - deferred product ergonomics, if any remain

Phase 9 gate:

- the milestone is not closed until the docs and closeout matrix no longer let
  "denied correctly" read as "ergonomically supported."

## Required Named Proof Families

### The Mutation Response Plan Authority Test

Purpose:

- prove create/update/remove responses lower into one canonical
  mutation-response reconciliation plan before read-line mutation

What to stress:

- create, update, remove, and custom action responses
- identical read and mutation response lenses
- mutation payloads that wrap canonical values in metadata
- zero-target, one-target, and multi-target plans
- stale submitted mutation basis
- WORTHd mutation response lens proof
- denied target admission

Pass condition:

- emit mutation route digest, submitted effect digest, response lens digest,
  target plan digest, atomicity digest, denial digest, and planning breadth
  envelope.

### The Detail Granular Response Lens Test

Purpose:

- prove detail resources are no longer whole-response-only

What to stress:

- detail field patch
- detail JSON path patch
- optional detail JSON terminal
- array-crossing detail JSON path
- detail region patch
- whole detail replacement
- hostile accessors, non-JSON values, identity-changing patches, malformed
  regions, and broad replacement attempts where narrow proof exists

Pass condition:

- emit detail declaration digest, field/path/region locus digest, rollback
  digest, denial digest, merge/rebase digest, and detail cost envelope.

### The Save Response To Detail Line Reconciliation Test

Purpose:

- prove update/save responses can replace or patch canonical detail truth
  without feature-local commit logic

What to stress:

- returned whole detail value
- partial detail field response
- partial detail region response
- validation/warning payloads beside canonical detail truth
- optimistic save followed by canonical transformed server truth
- stale save response after local or delivered drift

Pass condition:

- emit submitted detail basis digest, response extraction digest, target detail
  digest, canonicalization digest, diagnostics digest, and no-manual-commit
  proof.

### The Update Response To Collection And Summary Test

Purpose:

- prove update responses can patch related collection, paged, and summary truth
  through declared topology

What to stress:

- direct arrays, object-items, custom collections, maps, entity stores,
  grouped collections, named/multiple collections, sparse pages, connections,
  discriminated tuples, and trees
- returned canonical item
- returned partial item plus summary metadata
- off-page or unloaded item update
- duplicate visible item identity
- summary-only server response

Pass condition:

- emit topology declaration digest, item target digest, summary target digest,
  fallback digest, denial digest, and topology cost envelope.

### The Identity Migration Foundation Test

Purpose:

- prove canonical id changes are branch-native effects rather than caller-side
  map rewrites

What to stress:

- temp id to server id
- draft id to published id
- clone/import id mapping
- server-assigned child ids inside a detail region
- migration touching detail, collection, summary, and auxiliary read targets
- target basis drift before migration admission
- partial migration policy
- migration unavailable with refetch
- migration unavailable with delivery awaited

Pass condition:

- emit submitted identity digest, canonical identity digest, migration plan
  digest, target fanout digest, partial/unavailable posture digest, rollback
  digest, and migration cost envelope.

### The Create Placement Test

Purpose:

- prove optimistic creates converge from canonical server identity into declared
  collection/detail placement truth

What to stress:

- placement by exact position, server ordering key, append/prepend, group,
  page, tree parent, connection edge, map key, entity record, and response
  supplied placement
- placement unavailable with refetch
- placement unavailable with delivery awaited
- canonical detail line materialization after create
- summary updates after create
- rollback before and after canonical migration

Pass condition:

- emit canonical identity digest, placement digest, created detail digest,
  collection insertion digest, summary digest, rollback digest, and placement
  cost envelope.

### The Remove Deletion And Tombstone Test

Purpose:

- prove remove/delete responses can reconcile exact deletion, detail
  invalidation, tombstone updates, summaries, and fallbacks

What to stress:

- exact collection deletion
- tombstone/soft-delete aspect update
- detail line removal or invalidation
- summary count/status updates
- delete response with no body
- delete response with canonical deleted item
- delete response with operation metadata only
- stale delete and duplicate delete confirmations

Pass condition:

- emit deletion plan digest, tombstone digest, detail invalidation digest,
  summary digest, fallback digest, rollback digest, and no-side-effect denial
  proof.

### The Partial Mutation Response Mapping Test

Purpose:

- prove partial canonical payloads update only the fields, regions, summaries,
  or diagnostics they declare

What to stress:

- `{ id, updatedAt, version }`
- `{ workflow, warnings }`
- `{ node, graphVersion }`
- `{ accepted: true, deliveryExpected: true }`
- fields that are absent versus explicitly null
- unknown response fields

Pass condition:

- emit response field digest, declared mapping digest, unknown-field posture,
  partial target digest, stale-field artifact, and cost envelope.

### The Multi-Family Mutation Convergence Test

Purpose:

- prove one mutation can reconcile several read families without partial stale
  folklore

What to stress:

- detail plus collection item
- detail plus paged search result
- collection plus summary
- workflow detail plus workflow list plus audit/history list
- permissions or validation auxiliary read after save
- one target denying while others could admit
- explicit partial-admission policy

Pass condition:

- emit target set digest, target outcome digest, atomicity/partial posture
  digest, fallback digest, diagnostics/history digest, and fanout breadth
  envelope.

### The Refetch And Delivery Fallback Honesty Test

Purpose:

- prove unprovable reconciliation becomes explicit freshness posture rather
  than manual app knowledge

What to stress:

- unknown create placement
- unloaded sparse page update
- topology mapping unavailable
- identity migration unavailable
- server accepted but canonical delivery follows
- partial response that cannot prove canonical local truth

Pass condition:

- emit fallback reason digest, affected target digest, freshness posture
  digest, delivery-awaited digest, refetch-required digest, and no-hidden-
  mutation proof.

### The Full Mutation Response Reconciliation Convergence Test

Purpose:

- prove the whole milestone is one coherent product surface

What to stress:

- create with optimistic temp id
- update returning canonical detail and summary
- partial update returning warnings and version
- delete with tombstone or exact removal
- multi-family target set
- rollback, branch restore, replay, merge/rebase, delivery fallback, refetch
  fallback, duplicate confirmation, stale confirmation

Pass condition:

- emit mutation-response plan digest, read-family target digest, detail-locus
  digest, collection-locus digest, identity-migration digest, placement/deletion
  digest, fallback digest, rollback digest, merge/rebase digest,
  diagnostics/history digest, and boundary performance envelope. Equivalent
  histories must converge exactly when they mean the same thing.

## Acceptance Evidence

This milestone is complete only when the wasm product surface can prove:

- response-owned create/update/remove lanes exist and lower to mutation
  response reconciliation plans
- mutation response lenses are distinct from read response lenses where payload
  shape requires it
- write responses can replace or patch related detail lines without
  feature-local commit logic
- detail resources support field, JSON path, region, and whole-response loci
- update responses can patch related collection, paged, summary, and detail
  truth through declared topology
- create responses can insert into declared collection topologies or emit typed
  placement fallback
- remove responses can delete, tombstone, invalidate detail truth, or emit
  typed deletion fallback
- temporary/client/draft/import ids can migrate to canonical server ids through
  proof-bearing branch-native effects
- partial mutation responses update only declared canonical fragments and name
  unknown/stale/fallback posture explicitly
- multi-family mutation responses reconcile, invalidate, refetch, delivery-
  await, or explicitly decline every declared target
- rollback, replay, branch restore, diagnostics, history, and merge/rebase are
  derived from the mutation-response plan
- type denials and runtime denials agree on every mutation/detail/reconcile
  capability boundary
- cost counters name target fanout, response extraction, topology lookup,
  traversal, reconstruction, identity migration, placement, deletion, and
  fallback breadth
- docs and closeout evidence distinguish admitted ergonomic support from
  denial-only support and typed unavailable fallback

## Performance And Cost Contracts

Required cost posture:

- mutation response planning happens before read-family mutation
- response extraction cost is counted separately from target reconciliation
- target fanout cost is explicit for multi-family plans
- detail field, JSON path, and region traversal/reconstruction counters are
  distinct
- create placement lookup and reconstruction counters are topology-specific
- delete lookup and reconstruction counters are topology-specific
- identity migration counts every rewritten target, reference, and summary it
  claims to update
- fallback/refetch/delivery-awaited posture counts affected targets without
  materializing rich replay artifacts by default
- diagnostics summary must consume compact mutation-response facts rather than
  reconstructing every target plan unless rich diagnostics are requested

Any helper that makes a write response look like an O(1) local commit while it
extracts broad response payloads, scans target families, reconstructs detail
documents, migrates identities, or materializes diagnostics is out of spec
unless the cost is named and certified.

## Out Of Scope

- network transport ownership
- service-worker synchronization
- UI toast/banner/modal execution
- arbitrary response topology inference without declaration
- arbitrary item identity inference without declaration
- framework-specific cache integration
- core `worth-signal` branch or merge semantics not intentionally added to the
  native crate first

Important non-deferrals:

- detail field/path/region support is in scope
- create/update/remove response-owned lanes are in scope
- write-result-to-read-family reconciliation is in scope
- identity migration is in scope
- placement and deletion topology proof are in scope
- refetch and delivery-awaited fallback posture is in scope
- multi-family reconciliation is in scope

## Sequencing Notes

This milestone belongs immediately after
[resource_response_lens_contracts_plan.md](./resource_response_lens_contracts_plan.md).

Milestone 10 closed the substrate:

- resource effects are branch-native
- response lenses lower topology into effect loci
- advanced collection topologies have proof
- whole-response detail replacement exists
- rollback, merge/rebase, diagnostics, and cost proof exist

This milestone finishes the product surface that consumers feel next:

- mutation responses
- detail granularity
- create/update/delete ergonomics
- canonical write-result reconciliation
- identity migration
- refetch/delivery fallback
- multi-family target convergence

It should land before treating API resources as fully finished for workflow
editors, form submissions, router continuity, or external integration. Those
surfaces should consume write/detail reconciliation instead of manually
normalizing mutation results.

## Self-Check

- Does this milestone solve a real structural problem?
  Yes. It closes the asymmetry between elegant collection response topology and
  substrate-shaped write/detail reconciliation.
- Is the adversarial constraint precise and load-bearing?
  Yes. It names the failure modes: stale read families, manual save commits,
  temp id drift, broad detail replacement, unproved placement/deletion, partial
  response overreach, and denial-only false closure.
- Does the milestone preserve crate authority boundaries?
  Yes. Mutation responses plan reconciliation; read families own line truth;
  detail lenses own topology lowering; branch-native effects own lifecycle,
  rollback, and merge proof.
- Does the milestone define proof obligations, not just implementation tasks?
  Yes. Each phase has gates and named proof families with required emitted
  digests, denial artifacts, and cost envelopes.
- Could a competent engineer map this spec into honest types, modules, and
  tests?
  Yes. The spec names mutation response plans, mutation response lenses, detail
  loci, placement/deletion/migration planners, fallback artifacts, and
  verification packages.
- Does the milestone belong in this roadmap sequence?
  Yes. It depends on Milestone 10's branch-native effect and response-lens
  substrate, and it should precede any claim that API resources are complete
  for serious editor/workflow write surfaces.
