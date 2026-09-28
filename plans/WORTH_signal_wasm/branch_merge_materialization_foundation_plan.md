# Branch Merge Materialization And Aspect Policy Foundation Plan

> **Status:** Superseded design record
>
> **Superseded by:**
> [branch_merge_resolution_and_materialization_plan.md](./branch_merge_resolution_and_materialization_plan.md)
>
> The replacement corrects this plan's authority placement. Native
> `worth-signal` owns derived execution, not application-value merge truth.
> Standalone value history and manual resolution live in the explicit
> TypeScript local truth authority.
>
> **Roadmap parent:** [wasm_product_roadmap.md](./wasm_product_roadmap.md)
>
> **Prerequisite milestone:**
> [resource_response_lens_contracts_plan.md](./resource_response_lens_contracts_plan.md)
>
> **Shared vocabulary prerequisite:**
> [plans/worth-foundational/milestone-9.md](../worth-foundational/milestone-9.md)
>
> **Successor milestone:**
> [resource_mutation_response_reconciliation_plan.md](./resource_mutation_response_reconciliation_plan.md)
>
> **Certification parent:** [test-requirements.md](./test-requirements.md)

## Goal

Make branch merge execution a real web-product foundation: native merge proof
decides semantic merge truth, and `worth-signals-wasm` materializes that truth
into web-visible state through declared, proof-bearing materialization
strategies.

The target outcome is:

- native `worth-signal` admits scoped merge/cherry-pick requests as
  proof-bearing merge semantics lowered through shared `worth-foundational`
  scoped merge vocabulary rather than leaving wasm to filter native merge
  records after the fact
- web callers can request every native branch merge policy dimension required
  for branch-native products, including source-only policy and aspect policy
  bindings
- merge preview and merge execution consume one canonical policy request shape
- selected-node and selected-aspect merge scopes are planned by native merge
  authority and surfaced through wasm/product proof
- merge preview can produce an executable merge intent bound to source/target
  branch basis proof
- wasm branch-local store materialization consumes native merge result proof
  instead of choosing source or target values ad hoc
- the first admitted materialization strategy composes plain object fields from
  aspect decisions
- unsupported value shapes, topology shapes, or stale preview bases emit typed
  unavailable or denial posture before branch/store mutation
- branch creation can explicitly fork from a declared branch and snapshot basis
- later demos, resources, controllers, and mutation-response lanes can consume
  branch merge proof without building a second merge engine

This milestone is not a gear demo milestone. The gear demo is one certification
consumer. The architectural milestone is branch merge materialization.

## Why This Milestone Exists

Milestone 10 correctly made resource effects branch-native and declared that
the wasm product surface exposes the native branch/merge dimensions resource
effects require.

The current substrate is close, but the remaining gap is structural:

- native `worth-signal` already has branch merge policy hooks such as
  `RuntimeMerge::aspect_policy_named(...)`
- native merge planning already lowers policy and aspect decision proof
- wasm merge requests do not expose all native policy dimensions
- wasm branch-local store reconciliation can still adopt whole source values
  where a narrower materialization strategy is required
- branch creation is active-branch based instead of explicit-basis based
- UI code can therefore be tempted to compute merged values locally

If we only patch the plain object case, we risk solving the demo while leaving
the architecture too narrow. Real applications will use branch merge for:

- aspect-scoped object edits
- controller or graph contract branches
- resource line and response-lens effect branches
- partial/cherry-picked merge surfaces
- source-only introductions and target-only preservation
- deletion, tombstone, and identity correspondence
- branch timelines that must replay from proof, not from visual snapshots

This milestone defines the foundation so those later uses are natural
extensions rather than new local merge engines.

## Governing Source Summaries

- `MENTALITY.md`
  Protects hard-problem-first engineering. The runtime foundation must solve
  branch merge materialization generally enough that demos are cheap consumers,
  not fake implementations.
- `arch_laws.md`
  Protects lowered execution plans, proof-bearing phase transitions, and
  separation of policy resolution from execution. Merge materialization must
  consume native proof rather than deciding merge policy during store writes.
- `composition_laws.md`
  Protects semantic compilation units. Request normalization, native policy
  forwarding, merge intent construction, materialization strategy selection,
  store execution, and proof adaptation must remain separate file owners.
- `domain_structure_laws.md`
  Protects authority boundaries. Native `worth-signal` owns merge semantics;
  wasm owns web request translation and web-visible materialization; UI owns
  rendering only.
- `perf_laws.md`
  Protects cost honesty. Merge materialization must name branch-state capture,
  source/target lookup, strategy selection, reconstruction, downstream
  invalidation, and fallback breadth.
- `web_runtime_spec.md`
  Protects the rule that package surfaces consume runtime truth and do not
  become a second reactive or merge engine.
- `wasm_product_roadmap.md`
  Protects sequencing. This milestone closes the branch/merge policy exposure
  and materialization gap after Milestone 10 and before mutation-response
  reconciliation.
- `worth_signal_vision.md`
  Protects deterministic, transactional, aspect-aware, inspectable derived
  computation. Branch merge outcomes must be explainable and replayable.
- `worth_signal/test-requirements.md`
  Protects hostile branch, replay, restore, provenance, and granularity proof.
- `worth_signal_wasm/test-requirements.md`
  Protects resource/API product convergence and forbids second client truth
  engines.
- `worth-foundational/milestone-9.md`
  Protects shared scoped merge and cherry-pick vocabulary. Native `worth-signal`
  scoped merge proof must lower into this language before wasm treats it as the
  source of truth for materialization.

## Current Runtime Evidence

Native support already exists for the policy side:

- `RuntimeMerge::aspect_policy_named(...)` exists in
  `crates/worth-signal/src/logic/transaction/runtime/state/guided.rs`.
- Native merge requests carry `aspect_policy_bindings`.
- Native merge planning resolves those bindings in
  `crates/worth-signal/src/logic/transaction/runtime/state/branching/merge_runtime.rs`.
- Native merge plans and results carry `aspect_policy_plan` and
  `aspect_decision_plan`.
- Native merge candidate selection currently starts from the source branch
  mutation journal as a whole. There is no request-level scoped merge or
  cherry-pick selection proof for selected nodes or selected aspects.

The wasm/product boundary currently weakens that authority:

- `MergePolicyPreviewRequest` in
  `crates/worth-signal-wasm/src/runtime/core/state.rs` does not expose
  `source_only_policy_name` or `aspect_policy_bindings`.
- `plan_merge_policy_preview_raw(...)` and
  `merge_branches_policy_preview_raw(...)` in
  `crates/worth-signal-wasm/src/runtime/core/merge.rs` do not forward those
  dimensions.
- `merge_branch_store(...)` in the same file currently copies source values
  for adopted records, which is not a general materialization strategy.
- `SignalsTransaction.setWithAspects(...)` already exists, so aspect-scoped
  writes are not the missing piece.

Normative consequence:

- do not add cherry-pick by filtering native merge records in wasm
- do not fix merge in React
- do not treat object-field composition as the whole architecture
- do not infer merge policy from raw values
- do not count preview support as complete unless execution consumes the same
  request shape
- do not let unsupported materialization silently fall back to broad source
  adoption when proof requires narrower behavior

## Adversarial Constraint

A long-lived web application can fork branches from arbitrary retained
snapshots, mutate disjoint and overlapping semantic surfaces, preview a merge,
change one branch before execution, restore a branch to an earlier snapshot,
and replay the history later.

The merged target may include:

- one object signal with independently changed aspects
- one controller public contract with several input handles
- one resource line plus derived outputs
- one source-only introduced node
- one deleted or tombstoned node
- one partial merge that intentionally accepts only selected aspects or nodes

If two semantically equivalent histories can produce:

- different final merged web-visible state
- different downstream computed/resource invalidation
- source adoption that loses target-side disjoint truth
- conflict choices that are collected but not executable by the runtime
- branch children created from the wrong parent basis
- preview artifacts that can execute after source or target drift
- local UI code computing merged values
- or replay/export artifacts that cannot reconstruct the merge outcome

then this milestone has failed.

## Architectural Model

### Native Merge Authority

Native `worth-signal` owns branch merge semantics and lowers shared boundary
meaning through `worth-foundational`:

- source and target branch identity
- merge scope selection
- selected node scope
- selected aspect scope
- merge base selection
- source-only policy
- conflict policy
- conflict isolation policy
- identity matcher policy
- deletion policy
- aspect policy bindings
- native merge plan and result proof
- aspect policy and aspect decision plans

Wasm may request policy and consume proof. Wasm must not decide policy.

Scoped merge and cherry-pick belong here. Selecting only one node, one
semantic surface, or one aspect changes the native merge candidate set,
conflict set, source-only/deletion posture, and counters. A wasm layer that
filters native records after planning would be changing the meaning of the
merge without native proof.

The native merge request must therefore grow an explicit scoped merge family
after `worth-foundational` owns the shared vocabulary and before wasm can claim
cherry-pick support:

- all source journal candidates
- selected source nodes
- selected aspects for selected source nodes
- selected aspects by identity correspondence when source and target node ids
  differ

The native merge plan/result must carry:

- scope declaration digest
- admitted selected nodes
- admitted selected aspects
- selected-but-unchanged no-op records
- skipped-out-of-scope records or counters
- scope denial reason for unknown, ambiguous, deleted, or non-adoptable
  selections
- scope breadth counters

### Merge Intent Authority

The wasm product layer must introduce an executable merge intent between
preview and execution.

The intent binds:

- source branch id
- target branch id
- source head basis
- target head basis
- native merge scope declaration
- selected policy names
- aspect policy bindings
- optional preview/proof digest
- intended materialization scope when partial merge is requested

Execution consumes the intent and denies if the basis no longer matches.

### Materialization Strategy Authority

`worth-signals-wasm` owns the strategy that maps native merge proof into
web-visible state.

The first admitted strategy is:

- plain object field materialization from aspect decisions

The architecture must allow later strategies:

- whole-value materialization
- graph/controller public contract materialization
- resource line materialization
- collection/entity/map materialization
- deletion/tombstone materialization
- identity-correspondence materialization
- partial/cherry-pick materialization

Unsupported strategies must produce typed unavailable posture.

### Branch-Local Store Authority

The wasm runtime owns branch-local `RuntimeStoreSnapshot` materialization.

It must:

- capture source and target branch store state
- consume native merge result proof
- select a declared materialization strategy
- reconstruct only the admitted web-visible state
- invalidate derived recipes/outputs through runtime truth
- record materialization breadth

It must not:

- choose source or target from values alone
- mutate active branch state before admission
- silently apply broad replacement when a narrower materialization strategy is
  required

### Product Facade Authority

The TypeScript facade owns ergonomic request shapes, declarations, and docs.

It must expose:

- policy request fields
- scoped merge/cherry-pick request fields that map directly to native scope
  proof
- executable merge intent
- branch fork basis API
- proof summaries
- typed unavailable/denial results

It must not hide merge policy behind an untyped object bag.

## Product Decision Lock

- Branch merge materialization is the milestone; aspect object merge is the
  first admitted strategy.
- Cherry-pick and partial merge are native merge semantics. They are not wasm
  filters, UI filters, or materialization shortcuts.
- Preview and execution share one policy request model.
- Execution must be basis-checked against preview or declared branch heads.
- Selected-node and selected-aspect scopes must be planned by native merge
  authority before materialization can run.
- Materialization strategy selection happens after native merge proof and
  before store mutation.
- Unsupported materialization is typed unavailable, not source adoption.
- Partial merge/cherry-pick is a first-class direction, but admitted support
  starts only where native scoped merge proof and wasm materialization strategy
  both exist.
- Branch fork basis is explicit.
- UI and demos render proof; they never compute merged truth.

## Phases

### Phase 1: Adopt Foundational Scoped Merge Vocabulary

Purpose:

- make `worth-signal` depend on the shared scoped merge vocabulary before it
  adds native scoped merge execution

This phase must ship in `worth-signal`:

- dependency on `worth-foundational` scoped merge vocabulary once Milestone 9
  exists
- native adapter functions that can translate branch ids, node ids, aspect ids,
  merge basis, and scope loci into foundational boundary vocabulary
- compile-time or facade-level proof that scoped merge code can name
  foundational scope request, admitted scope, skipped scope, selected no-op,
  denial, and unavailable posture types

Phase 1 gate:

- `worth-signal` can materialize foundational scoped merge boundary vocabulary
  without yet changing merge behavior.

### Phase 2: Native Merge Scope Request Type

Purpose:

- add native request vocabulary for full-branch, selected-node, and
  selected-aspect scope

This phase must ship in `worth-signal`:

- `BranchMergeScope` or equivalent proof-bearing request type with:
  - all source journal candidates
  - selected source nodes
  - selected aspects for selected source nodes
  - selected aspects through identity correspondence when source and target
    node ids differ
- request builder support beside the existing native merge builder methods
- default full-branch scope for existing merge requests
- request normalization that rejects empty selected-node and selected-aspect
  scopes before planning
- lowering from native request scope into foundational scope request vocabulary

Phase 2 gate:

- native merge preview can receive a scoped request and report normalized scope
  proof without filtering merge records after planning.

### Phase 3: Native Candidate Scope Lowering

Purpose:

- apply selected scope before merge policy, identity, deletion, and aspect
  decisions are planned

This phase must ship in `worth-signal`:

- candidate-set lowering that consumes scope before policy resolution,
  identity correspondence, conflict isolation, source-only handling, deletion
  handling, and aspect decision planning
- planned candidate set that distinguishes requested, admitted, skipped, and
  selected no-op candidates
- scope breadth counters for requested, admitted, skipped, no-op, and
  conflict-check breadth
- native tests proving selected-node scope changes the candidate set before
  policy resolution
- native tests proving selected-aspect scope narrows aspect decision planning
  before materialization

Phase 3 gate:

- native scoped merge changes the planned candidate/decision set upstream of
  execution, not downstream of result records.

### Phase 4: Native Scope Denial And Unavailable Posture

Purpose:

- make unsafe or unsupported scoped merge fail before merge execution

This phase must ship in `worth-signal`:

- typed denial for unknown selected node, ambiguous identity correspondence,
  deleted selected target, non-adoptable selected source, unknown aspect, and
  aspect scope without aspect-capable node proof
- typed unavailable posture for native scope families that cannot be admitted
  under the current runtime strategy
- lowering of all denials/unavailable posture into foundational scope denial
  vocabulary
- no-side-effect tests proving denial preserves branch, diagnostics, and merge
  history truth

Phase 4 gate:

- native scoped merge cannot become generic merge failure or broad fallback.

### Phase 5: Native Scope Proof In Plans And Results

Purpose:

- carry scoped merge proof through preview, execution summary, result, replay,
  and diagnostics

This phase must ship in `worth-signal`:

- branch merge plan/result fields carrying scope declaration digest, admitted
  scope digest, skipped scope digest, selected no-op digest, denied scope
  digest, unavailable scope digest, and scope breadth counters
- proof report fields for scope digests
- replay/merge parity checks that compare scope proof
- native tests proving selected-node and selected-aspect merge semantics before
  any wasm materialization code is allowed to depend on them

Phase 5 gate:

- native preview and execution can prove selected-node and selected-aspect
  merge scopes through foundational vocabulary without wasm filtering native
  records after planning.

### Phase 6: Merge Policy Request Boundary

Purpose:

- expose missing native policy dimensions through wasm and product request
  surfaces

This phase must ship:

- scoped merge/cherry-pick request fields that forward to native
  `BranchMergeScope`
- `source_only_policy_name` on wasm merge policy requests
- `aspect_policy_bindings` on wasm merge policy requests
- validation for duplicate aspect policy bindings
- validation for out-of-range aspect ids
- forwarding into preview and execution paths
- product facade camelCase request support
- TypeScript declarations for aspect policy bindings and source-only policy
- worker-first and main-thread parity where history merge policy APIs exist

Phase 6 gate:

- request-named aspect policies appear in native lowered policy proof for both
  preview and execution
- selected merge scope appears in native lowered scope proof for both preview
  and execution
- malformed policy requests deny before branch mutation

### Phase 7: Executable Merge Intent

Purpose:

- create the authority artifact that connects preview, user-facing conflict
  review, and execution

This phase must ship:

- merge intent type carrying branch ids, branch head basis, selected policies,
  aspect bindings, and optional preview/proof digest
- intent construction from merge preview
- intent construction without preview for direct execution when no conflict
  review is needed
- selected merge scope basis checking
- stale source basis denial
- stale target basis denial
- WORTHd or mismatched preview digest denial where digest is supplied
- compact intent diagnostics

Phase 7 gate:

- execution cannot consume conflict choices without a valid intent
- branch drift after preview denies before native merge execution

### Phase 8: Materialization Strategy Taxonomy

Purpose:

- define the materialization strategy vocabulary before planning or execution

This phase must ship:

- materialization strategy taxonomy:
  - whole value
  - plain object aspect fields
  - source-only introduction
  - target preservation
  - deletion/tombstone unavailable
  - identity-correspondence unavailable
  - scoped merge/cherry-pick
  - scoped merge/cherry-pick materialization unavailable
  - unsupported topology unavailable
- public/internal type definitions for each strategy outcome
- typed unavailable posture for every strategy not admitted in v1

Phase 8 gate:

- materialization planning has a named strategy vocabulary before store
  mutation code changes.

### Phase 9: Materialization Strategy Planning

Purpose:

- separate native merge result proof from wasm store mutation through an
  explicit materialization plan

This phase must ship:

- lowered materialization plan type
- strategy selection from native merge result proof and wasm catalog metadata
- no-side-effect denial before store mutation when no strategy admits
- materialization breadth counters

Phase 9 gate:

- store mutation cannot run without a lowered materialization plan
- unsupported shapes produce typed unavailable posture instead of broad source
  adoption

### Phase 10: Whole-Value And Existing Behavior Preservation

Purpose:

- keep existing broad merge behavior honest while introducing strategy planning

This phase must ship:

- whole-value materialization strategy for existing non-aspect-producing signals
- proof that existing full-branch broad merge behavior remains compatible where
  native proof admits broad replacement
- denial that broad source adoption cannot run when native scoped/aspect proof
  requires narrower materialization

Phase 10 gate:

- existing broad behavior is preserved only through an explicit strategy.

### Phase 11: Plain Object Aspect Metadata

Purpose:

- give wasm enough declared metadata to compose object fields from aspect proof

This phase must ship:

- aspect-to-field metadata in wasm signal catalog or equivalent declaration
  proof
- duplicate field binding denial
- missing metadata denial
- preservation of produced-aspect metadata and per-aspect version summaries

Phase 11 gate:

- object aspect materialization has declared metadata before reconstruction
  logic exists.

### Phase 12: Plain Object Aspect Materialization

Purpose:

- admit the first concrete materialization strategy needed for object-shaped
  app state and the gear demo

This phase must ship:

- plain object source/target field composition from aspect decision proof
- denial for non-object values and identity-changing composition where identity
  is declared
- reconstruction counters for field count and object copy breadth

Phase 12 gate:

- disjoint aspect edits on one object signal merge into one composed object
- same-aspect edits require admitted policy resolution
- non-aspect-producing signals preserve existing whole-value behavior

### Phase 13: Explicit Branch Fork Basis

Purpose:

- make branch creation from parent branch/snapshot basis explicit

This phase must ship:

- native `worth-signal` branch creation from declared parent branch and optional
  snapshot basis when core cannot already express that operation directly
- wasm runtime API equivalent to
  `create_branch_from({ name, branch_id, snapshot_id? })`
- product facade equivalent to `history.createBranchFrom(...)`
- branch and snapshot validation before mutation
- restoration of previously active branch after success or denial
- returned proof naming parent branch id, fork snapshot id, created branch id,
  and restored active branch id

Phase 13 gate:

- multiple children can fork from one parent snapshot without inheriting the
  latest active branch by accident

### Phase 14: Selected Whole-Node Materialization

Purpose:

- admit selected-node materialization for the v1 whole-value strategy

This phase must ship:

- selected whole node materialization when native scope proof selects a whole
  source node and the target strategy admits whole-value replacement
- no-op selected node preservation
- skipped-out-of-scope preservation
- tests proving unselected source candidates do not mutate target store

Phase 14 gate:

- selected-node scope can execute for admitted whole-value shapes without broad
  source adoption.

### Phase 15: Selected Aspect Materialization

Purpose:

- admit selected-aspect materialization for plain object aspect fields

This phase must ship:

- selected aspect field composition from native aspect decision proof
- no-op selected aspect preservation
- skipped-out-of-scope preservation
- tests proving selected teeth/thickness-style aspect changes do not overwrite
  unselected radius-style target aspects

Phase 15 gate:

- selected-aspect scope can execute for admitted plain-object fields without
  mutating unselected fields.

### Phase 16: Unsupported Scoped Materialization Posture

Purpose:

- prevent scoped native merge support from implying every web topology can
  already materialize partial results

This phase must ship:

- typed unavailable posture for scoped materialization shapes not admitted in
  v1, while preserving native scoped merge proof
- proof that unavailable scoped materialization denies before branch/store
  mutation
- docs explaining future strategy slots for controllers, graph contracts,
  resources, entity stores, maps, deletion, tombstones, and identity
  correspondence

Phase 16 gate:

- the public model can execute scoped merges only where both native scope proof
  and wasm materialization proof exist, and can decline the rest without
  mutating branch truth

### Phase 17: Product Facade And Type Surface

Purpose:

- expose the admitted runtime capabilities through stable package APIs

This phase must ship:

- product facade request types for source-only policy, aspect policy bindings,
  and selected merge scopes
- type declarations for merge intent, scoped merge proof, materialization
  outcome, and unavailable posture
- type-smoke coverage for admitted and denied public capability boundaries

Phase 17 gate:

- application code can request admitted scoped merge behavior without touching
  raw wasm internals.

### Phase 18: Product Integration And Demo Readiness

Purpose:

- make demos and later products consume the foundation rather than substituting
  for it

This phase must ship:

- gear demo modeled as one aspect-producing signal
- slider commits through `setWithAspects(...)`
- branch tabs through explicit fork basis
- merge preview and execution through merge intent
- conflict choices lowered to aspect policy bindings
- cherry-pick controls lowered to native selected-node or selected-aspect scope
- no UI-side merged value construction
- proof summaries read from history/signals

Phase 18 gate:

- demo-visible branch/aspect behavior is backed by runtime execution proof

## Required Named Proof Families

### The Native Scoped Merge Authority Test

Proves selected-node and selected-aspect merge scopes are native merge
semantics, not wasm-side filtering.

Pass condition:

- emit scope request digest, admitted scope digest, skipped-out-of-scope digest,
  selected-no-op digest, scope denial digest, native candidate breadth envelope,
  and no-wasm-filter proof

### The Merge Policy Boundary Test

Proves source-only policy and aspect policy bindings pass through product,
wasm, native preview, and native execution with matching proof.

Pass condition:

- emit request digest, policy binding digest, selected policy basis digest,
  selected scope digest, denial digest, and no-execution-on-denial proof

### The Executable Merge Intent Test

Proves preview-to-execution basis safety.

Pass condition:

- emit intent digest, source basis digest, target basis digest, preview digest,
  stale-basis denial digest, and no-side-effect proof

### The Materialization Strategy Planning Test

Proves store execution consumes a lowered materialization plan.

Pass condition:

- emit native merge result digest, strategy selection digest, unavailable
  posture digest, and materialization breadth envelope

### The Plain Object Aspect Materialization Test

Proves object fields compose from source/target aspect decisions.

Pass condition:

- emit source value digest, target value digest, aspect decision digest,
  composed value digest, metadata digest, and reconstruction breadth envelope

### The Disjoint Branch Merge Convergence Test

Proves multiple disjoint aspect edits preserve all admitted changes across
merge orders, restore, and replay.

Pass condition:

- equivalent histories produce identical merged values and proof digests

### The Same Aspect Conflict Resolution Test

Proves overlapping aspect edits require explicit policy resolution.

Pass condition:

- require-conflict, prefer-source, and prefer-target produce distinct
  proof-visible outcomes with no side effects on denial

### The Explicit Fork Basis Test

Proves branch children inherit the declared parent branch/snapshot basis.

Pass condition:

- child branch proof names parent basis and denial preserves active branch
  truth

### The Partial Merge Honesty Test

Proves selected-aspect or selected-node native merge proof materializes only
where a declared wasm strategy admits it, and unsupported strategies deny
honestly.

Pass condition:

- emit native selected scope digest, admitted/unavailable materialization
  strategy digest, and no-side-effect denial proof

### The Product No-Shortcut Test

Proves demos and product examples do not compute merge truth locally.

Pass condition:

- instrumented demo operations show final values come from runtime signal reads
  after history execution

## Must Ship

- native `worth-signal` scoped merge/cherry-pick request, plan, result, denial,
  and counter proof lowered into `worth-foundational` scope vocabulary
- wasm/product merge request support for source-only policy and aspect policy
  bindings, plus forwarding for native scoped merge request fields
- executable merge intent with basis checking
- materialization strategy taxonomy and lowered plan
- plain object aspect-field materialization strategy
- explicit branch fork basis API
- selected-node and selected-aspect materialization for admitted v1 shapes
- scoped merge unavailable fallback for unsupported materialization shapes
- no-side-effect denials for malformed policy, stale intent, unsupported
  materialization, unknown branch, unavailable snapshot, unknown selected node,
  and unsupported selected aspect
- proof/cost artifacts for policy forwarding, intent admission, strategy
  selection, scoped candidate lowering, store reconstruction, and branch-state
  restoration

## Must Preserve

- native `worth-signal` remains merge semantics authority
- existing branch and merge APIs remain backward compatible
- full-branch merge remains the default when no scope is declared
- whole-value materialization remains valid when proof admits it
- non-aspect-producing signals are not forced through object composition
- unsupported topologies do not silently downgrade to broad source adoption
- wasm never implements cherry-pick by filtering native records after native
  planning
- UI, React, docs examples, and demos never become merge authorities

## Acceptance Evidence

This milestone is complete only when:

- native merge preview and execution prove selected-node and selected-aspect
  scopes
- merge policy dimensions are exposed and proof-visible through preview and
  execution
- executable merge intents deny stale source or target bases
- store mutation consumes a lowered materialization plan
- plain object aspect merges preserve disjoint source/target changes
- same-aspect conflicts require admitted policy
- branch fork basis is explicit and replayable
- scoped merge/cherry-pick executes only through native scope proof and declared
  wasm materialization strategy
- unsupported materialization emits typed unavailable posture
- equivalent merge/restore/replay histories converge
- product demos can consume merge proof without local merge logic

## Performance And Cost Contracts

Required counters or proof breadth:

- request validation breadth
- native scope candidate filtering breadth
- selected-node and selected-aspect breadth
- skipped-out-of-scope breadth
- native policy forwarding breadth
- merge intent basis-check breadth
- branch-state capture and restore breadth
- materialization strategy selection breadth
- source and target store lookup breadth
- aspect decision breadth
- reconstructed field breadth
- unavailable fallback breadth
- downstream invalidation breadth after materialization

Any branch merge helper that hides broad store scans, rich proof materialization,
or object reconstruction behind cheap-looking execution is out of spec unless
the cost is named and certified.

## Out Of Scope

- generic aspect-capacity rewrite
- full nested object merge inference
- arbitrary user-supplied merge functions
- wasm-only cherry-pick filtering
- network transport ownership
- UI toast/banner/modal execution
- Three.js visual design
- full resource mutation-response reconciliation
- full controller/resource/entity materialization strategies beyond declared
  unavailable posture

## Sequencing Notes

This milestone belongs immediately after Milestone 10 because it closes a
branch/merge exposure and materialization gap in the branch-native substrate.

It intentionally includes a small native `worth-signal` expansion before wasm
work begins. Scoped merge/cherry-pick changes candidate selection and conflict
semantics, so the parent runtime must own that proof before the web package can
materialize it.

It belongs before mutation-response reconciliation because create/update/remove
response reconciliation will depend on branch merge, rollback, replay,
diagnostics, and materialized store truth that must not lose disjoint aspect
changes.

If native merge result proof lacks a field needed for safe materialization,
native proof must be extended first. Wasm must not infer missing authority from
raw values.

If native merge request proof lacks a scope needed for cherry-pick or partial
merge, native request/planning/result proof must be extended first. Wasm must
not add scoped merge by filtering source records, target records, or materialized
store values after native planning.

## Self-Check

- Does this milestone solve a real structural problem?
  Yes. It turns native merge proof into web-visible materialized branch truth
  without a UI-side merge engine or wasm-side cherry-pick filter.
- Is the adversarial constraint precise and load-bearing?
  Yes. Disjoint, overlapping, stale, partial, and replayed branch merges break
  naive whole-value adoption.
- Does the milestone preserve crate authority boundaries?
  Yes. Native owns merge policy and scope; wasm owns materialization; UI owns
  rendering.
- Does the milestone define proof obligations?
  Yes. It names policy, intent, strategy, object materialization, fork basis,
  partial merge, and no-shortcut proof families.
- Could a competent engineer map this spec into honest types, modules, and
  tests?
  Yes. The phases define concrete request, intent, strategy, materialization,
  native scope, fork, and certification artifacts.
- Does the milestone belong in this roadmap sequence?
  Yes. It closes the merge foundation before mutation-response reconciliation
  and before demos claim branch/aspect convergence.
