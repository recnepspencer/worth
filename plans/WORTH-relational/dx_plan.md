# WORTH Relational DX Plan

## Purpose

This is the execution plan for making `worth-relational` publication-ready from
a DX perspective.

This plan starts from a specific new reality:

- performance hardening is substantially complete
- coverage and hotspot work have produced a real public-surface inventory
- the next major product risk is not raw runtime viability
- the next major product risk is letting the current internal decomposition
  define the public identity
- the relational-to-signal bridge should be built on top of a deliberate
  relational facade, not used as an excuse to preserve an accidental one

This is pre-bridge productization work.

If we build the bridge before the relational facade is coherent, the bridge
will freeze today's internal seams into tomorrow's public contract.

---

## Signal DX Inputs

The WORTH Signal DX program already established the right pattern for this kind
of work. The most relevant documents are:

- [`plans/worth_signal/dx_plan.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/dx_plan.md)
- [`plans/worth_signal/dx_canonical_surface_spec.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/dx_canonical_surface_spec.md)
- [`plans/worth_signal/dx_boundary_spec.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/dx_boundary_spec.md)
- [`plans/worth_signal/dx_api_matrix.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/dx_api_matrix.md)
- [`plans/worth_signal/dx_export_inventory.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/dx_export_inventory.md)
- [`plans/worth_signal/dx_export_decision_matrix.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/dx_export_decision_matrix.md)
- [`plans/worth_signal/dx_exposure_cleanup_strategy.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/dx_exposure_cleanup_strategy.md)
- [`plans/worth_signal/dx_condensation_map.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/dx_condensation_map.md)
- [`plans/worth_signal/dx_diagnostics_product_map.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/dx_diagnostics_product_map.md)
- [`plans/worth_signal/dx_compatibility_transition_plan.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/dx_compatibility_transition_plan.md)
- [`plans/worth_signal/dx_wording_map.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/dx_wording_map.md)
- [`plans/worth_signal/docs_publication_audit.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/docs_publication_audit.md)
- [`plans/worth_signal/dx_phase_0_2_review.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/dx_phase_0_2_review.md)
- [`plans/worth_signal/dx_phase_4_review.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/dx_phase_4_review.md)
- [`plans/worth_signal/dx_phase_5_plan.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/dx_phase_5_plan.md)
- [`plans/worth_signal/dx_phase_5_policy_inventory.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/dx_phase_5_policy_inventory.md)
- [`plans/worth_signal/dx_phase_5_review.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/dx_phase_5_review.md)

The important takeaway is not "copy Signal literally."

The important takeaway is:

- inventory first
- classify deliberately
- define a canonical memory shape
- separate daily use from specialist use
- condense high-friction flows
- productize diagnostics
- record compatibility strategy while cleanup is happening

---

## Relational Inputs

This plan builds on:

- [`plans/engineering/worth_relational_coverage_and_api_inventory.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/engineering/worth_relational_coverage_and_api_inventory.md)
- [`plans/worth-relational/worth_relational_vision.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/worth_relational_vision.md)
- [`plans/worth-relational/worth_relational_roadmap.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/worth_relational_roadmap.md)
- [`plans/worth-relational/test-requirements.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/test-requirements.md)
- [`plans/worth-relational/dx_phase_0_5_review.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/dx_phase_0_5_review.md)
- [`plans/worth-relational/dx_phase_1_plan.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/dx_phase_1_plan.md)
- [`plans/worth-relational/dx_phase_1_review.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/dx_phase_1_review.md)
- [`plans/worth-relational/dx_phase_2_review.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/dx_phase_2_review.md)
- [`plans/worth-relational/dx_phase_3_review.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/dx_phase_3_review.md)
- [`plans/worth-relational/dx_phase_4_review.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/dx_phase_4_review.md)
- [`plans/worth-relational/dx_phase_5_review.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/dx_phase_5_review.md)
- [`crates/worth-relational/src/lib.rs`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/lib.rs)
- [`crates/worth-relational/src/facade.rs`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/facade.rs)
- [`crates/worth-relational/src/presentation/api.rs`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/presentation/api.rs)

---

## North Star

The published library should feel like this:

- the truth-runtime story is obvious
- the first success path is fast
- runtime construction feels guided rather than sprawling
- the transaction and read story feels like one product, not a bag of access
  facets
- history, diagnostics, and publication are powerful without dominating day-one
  usage
- merge, replay, lineage, durability, and commit strategies remain available,
  but clearly specialist
- the API expresses truth-runtime jobs, not crate-internal subsystem layout
- the bridge can target a clean relational boundary instead of a moving one

The dominant public mental model should be:

- authoritative truth runtime for transactional graph state with history,
  inspection, and replay

Not:

- a pile of subsystems
- a certification harness
- a schema experiment
- a bridge substrate

Those things matter, but they must not own the first impression.

---

## Current DX Reality

`worth-relational` is in a different starting state than Signal was.

Things already working in our favor:

- one official crate boundary exists: [`facade.rs`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/facade.rs)
- a small top-level API exists through [`api.rs`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/presentation/api.rs)
- the runtime has real performance certification breadth
- the crate already distinguishes many serious specialist surfaces instead of
  pretending they do not exist

Things that were not yet productized at the start of this DX pass:

- the facade currently mirrors internal subsystem decomposition very directly
- there is no canonical public memory shape yet
- there is no explicit daily-use versus specialist-use boundary policy
- runtime construction is inventoryable, but not yet clearly condensed into one
  obvious production setup story
- diagnostics, inspection, history, publication, replay, and merge all exist,
  but the relationship between them is not yet framed as a deliberate product
  journey
- harness exposure is still visible in the main facade
- there is no written compatibility transition plan for facade cleanup

This is exactly the right moment to do the DX pass.

Current status after the Phase 0-0.5 closeout:

- the inventory standard now exists
- the canonical shape now exists
- the boundary target now exists
- the program has now closed Phases 0 through 5 and can move into the final
  publication gate before bridge work

Operational checkpoint:

- [`plans/worth-relational/dx_phase_0_5_review.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/dx_phase_0_5_review.md)
- [`plans/worth-relational/dx_phase_1_plan.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/dx_phase_1_plan.md)
- [`plans/worth-relational/dx_phase_1_review.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/dx_phase_1_review.md)
- [`plans/worth-relational/dx_phase_2_review.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/dx_phase_2_review.md)

---

## Non-Negotiable Standards

1. Do not let bridge needs define the relational public identity.
2. Do not publish internal support or certification scaffolding as core product
   API.
3. Do not preserve fragmented subsystem-first organization when a job-oriented
   surface should exist.
4. Do not make users memorize internal runtime decomposition to do ordinary
   truth work.
5. Do not let specialist surfaces crowd out daily-use surfaces.
6. Do not ship a public boundary we would be embarrassed to support through the
   bridge era.

---

## Completion Criteria

DX work is only done when all of the following are true:

- the public facade is deliberate
- every public export family has been consciously classified
- daily-use surfaces are clearly separated from specialist surfaces
- runtime setup, mutation, read, and diagnostics flows each have one obvious
  story
- docs are organized by product journey rather than internal modules
- examples cover the main truth-runtime workflows cleanly
- compatibility strategy exists for moved, contained, renamed, or removed
  surfaces
- the bridge can be designed against a stable relational facade contract

In addition, these artifacts must exist and stay current:

- relational DX export inventory
- relational DX export decision matrix
- relational DX exhaustive audit
- relational DX method decision matrix
- relational DX phase 0-0.5 review
- relational canonical surface spec
- relational boundary spec
- relational boundary cleanup list
- relational condensation map
- relational diagnostics product map
- relational wording map
- relational docs publication audit
- relational phase 3 review
- relational compatibility transition plan
- relational phase 4 review
- relational phase 5 review

---

## Canonical Public Memory Shapes

These are the shapes the DX work should drive toward.

### 1. Canonical Import Path

Primary path:

- `worth_relational::facade`

Target property:

- users should not need to reason about crate internals outside the facade
- `RelationalRuntimeApi` may remain a convenience door, but the facade must stay
  the canonical memory boundary

### 2. Canonical Production Setup Flow

The normal setup story should revolve around:

- `RelationalRuntimeApi::builder()`
- named runtime profiles
- guided builder refinement
- `build()`

Target property:

- one obvious path for creating a production runtime
- profile-first defaults
- deeper knobs only when a user means to leave the default path

### 3. Canonical Truth-Mutation Flow

The normal mutation story should revolve around:

- transaction entry
- batched intents
- commit result
- publication-visible truth effects

Target property:

- mutation should feel like one coherent truth-authority workflow, not separate
  knowledge of transactions, publication, and history internals

### 4. Canonical Read And Query Flow

The normal read story should revolve around:

- direct record reads
- bulk query and traversal
- explicit historical reads when requested

Target property:

- current-truth reads should feel distinct from historical, replay, and
  inspection work
- ordinary read/query use should not require a tour of specialist access facets

### 5. Canonical Diagnostics And Inspection Flow

The normal inspection story should revolve around:

- runtime diagnostics
- current graph/commit inspection
- recent mutation inspection
- retention and structural inspection when needed

Target property:

- diagnostics answer operator questions cleanly
- inspection feels productized, not debug-only

### 6. Canonical Specialist Flow

These should remain explicitly specialist:

- merge
- replay
- lineage
- durability and recovery
- commit strategies
- bridge-facing coordination surfaces

Target property:

- powerful and real
- discoverable after the core truth-runtime story
- not the first thing ordinary users have to stitch together

---

## Target Public Layering

The facade should converge on a few clear product layers.

### Layer 1: Everyday Truth Runtime

Should own:

- runtime construction
- identity vocabulary needed daily
- transactions and commit results
- current-truth reads
- common query and traversal
- schema registration surfaces that are required for ordinary use

### Layer 2: Advanced Runtime Control

Should own:

- profile and policy control
- snapshots and branches
- publication control where it is still part of ordinary operations
- deeper query planning and execution control
- complexity and runtime-tuning surfaces that are legitimate but not first-use

### Layer 3: Specialist Infrastructure

Should own:

- merge
- replay
- lineage
- durability and recovery
- commit strategies
- specialist publication and CDC control
- bridge-facing integration contracts

### Layer 4: Internal Support

Should not define the public product boundary.

Most likely candidates:

- harness-first scaffolding
- support-only fixtures
- certification helpers
- architecture-only bookkeeping surfaces

---

## Likely Relational Boundary Policy

This is the current proposed classification direction. It should be converted
into a full decision matrix in Phase 0.

### Keep In The Main Daily-Use Story

- `runtime`
- `transactions`
- `query`
- `identity`
- `schema`
- `payloads`
- `config`

### Keep But Contain As Advanced

- `history`
- `snapshots`
- `publication`
- `inspection`
- `diagnostics`
- `indexes`

### Keep But Contain As Specialist

- `merge`
- `replay`
- `lineage`
- `durability`
- `commit_strategies`

### Remove From The Main Public Story

- `harness`

That does not necessarily mean immediate privatization, but it does mean it
should stop participating in the main product identity.

---

## Phase 0: Freeze The Standard

### Goal

Create the relational equivalent of Signalâ€™s export and decision discipline so
future cleanup is systematic rather than taste-driven.

### Tasks

- turn [`worth_relational_coverage_and_api_inventory.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/engineering/worth_relational_coverage_and_api_inventory.md)
  into the seed for a strict relational DX export inventory
- produce `dx_export_inventory.md`
- produce `dx_export_decision_matrix.md`
- produce `dx_export_exhaustive_audit.md` directly from the live facade
- classify each facade family and major public type by:
  - daily-use
  - advanced
  - specialist
  - internal/support only
- require every new public surface to justify its layer and whether it is raw
  or guided

### Exit Criteria

- relational public API work references the decision matrix before adding or
  promoting surface area

---

## Phase 0.5: Define The Canonical Product Shape

### Goal

Decide what users and AI agents are actually supposed to reach for first.

### Required Deliverables

- `dx_canonical_surface_spec.md`
- `dx_boundary_spec.md`
- `dx_boundary_cleanup_list.md`

### It Must Define

- the one obvious import path
- the one obvious runtime setup flow
- the one obvious transaction / commit flow
- the one obvious current-truth read / query flow
- the one obvious diagnostics / inspection entry flow
- the one obvious historical / replay / merge specialist escalation path
- the explicit role of `RelationalRuntimeApi`

### Exit Criteria

- we have a target public shape, not just a cleanup intention

---

## Phase 1: Establish The Published Product Boundary

### Goal

Make the facade feel like a product boundary instead of a mirror of internal
subsystems.

### Tasks

- decide whether the facade should stay module-based but be regrouped, or
  whether a curated nested boundary is needed
- explicitly define:
  - default daily-use path
  - advanced path
  - specialist path
  - support-only path
- decide the exact public role of:
  - `RelationalRuntimeApi`
  - `facade::runtime`
  - `facade::diagnostics`
  - `facade::inspection`
  - `facade::history`

### Mandatory Cleanup

- stop letting `harness` participate in the main public narrative
- stop treating every subsystem module as equally first-class in user memory
- ensure the bridge will have a stable relational boundary to target

### Exit Criteria

- the public boundary model is explicit and documented

---

## Phase 2: Remove Internal Leakage And Specialist Noise

### Goal

Reduce visible noise before condensation starts.

### Tasks

- contain or de-emphasize `harness`
- review whether performance-only or certification-only data types are leaking
  through otherwise legitimate namespaces
- identify public types that are real but too internal-facing for the default
  docs and examples
- move support-only or narrow-author surfaces out of the default path

### Exit Criteria

- ordinary users can look at the public story without being forced through
  support or specialist scaffolding

---

## Phase 3: Condense The Core Truth-Runtime Flows

### Goal

Turn the current inventory into a small number of memorable workflows.

### Tasks

- condense runtime setup around profile-first builder flows
- condense transaction and commit flows so batching, authority, and results feel
  like one workflow
- condense current-truth read and query entry so explicit reads and bulk query
  are easy to discover
- condense diagnostics and inspection around operator jobs
- define the narrow escalation path from:
  - current truth
  - to history
  - to replay
  - to merge

### Required Deliverables

- `dx_condensation_map.md`
- `dx_diagnostics_product_map.md`

### Exit Criteria

- core runtime workflows are coherent enough to teach without walking module by
  module through the crate

---

## Phase 4: Productize Naming, Wording, And Documentation

### Goal

Make the public story read like a product rather than a closeout archive.

### Tasks

- create a wording map for the public vocabulary
- standardize around consistent terms for:
  - truth
  - transaction
  - commit
  - publication
  - inspection
  - historical reads
  - replay
  - merge
  - schema contracts
  - specialist versus daily-use language
- audit existing docs for:
  - publish-facing docs to rewrite
  - reference docs to keep
  - internal history docs
- rewrite examples and docs around product journeys:
  - quickstart runtime setup
  - commit and read workflows
  - inspection and diagnostics
  - history and snapshots
  - CDC and publication
  - specialist merge and replay

### Required Deliverables

- `dx_wording_map.md`
- `docs_publication_audit.md`

### Exit Criteria

- the docs teach the intended product shape instead of preserving historical
  implementation order

---

## Phase 5: Record Compatibility And Transition Strategy

### Goal

Prevent cleanup paralysis.

### Tasks

- create `dx_compatibility_transition_plan.md`
- decide where to:
  - deprecate
  - contain
  - rename
  - remove immediately
- prefer guided replacements before long deprecation ladders
- use immediate removal for clearly support-only or certification-only exposure

### Special Rule Before Bridge Work

No bridge-facing public design should depend on facade noise that this phase
already intends to remove or contain.

Bridge integration must target the post-cleanup relational boundary, not the
pre-cleanup one.

### Exit Criteria

- API cleanup has an explicit migration posture
- bridge work can proceed against a stable intended facade

---

## Phase 6: Publication Gate Before Bridge

### Goal

Declare the relational facade stable enough that bridge work can safely lock to
it.

### Must Be True

- canonical runtime setup flow exists
- canonical truth-mutation flow exists
- canonical read/query flow exists
- diagnostics and inspection have a productized entry story
- specialist surfaces are contained and named deliberately
- harness/support exposure is no longer shaping the public identity
- docs and examples reflect the curated facade
- compatibility strategy is written down

### Outcome

At that point, the bridge can be designed as:

- relational truth runtime
- signal derived-computation runtime
- explicit integration layer

Instead of:

- one runtime with another runtimeâ€™s internal seams glued onto it

---

## First Concrete Deliverables

The highest-leverage immediate sequence is:

1. write `dx_export_inventory.md`
2. write `dx_export_decision_matrix.md`
3. generate `dx_export_exhaustive_audit.md`
4. write `dx_method_decision_matrix.md`
5. write `dx_boundary_cleanup_list.md`
6. write `dx_canonical_surface_spec.md`
7. write `dx_boundary_spec.md`
8. do the first facade cleanup pass against those decisions

That is the same shape that worked for Signal, and Relational now has the
inventory maturity to do it cleanly.
