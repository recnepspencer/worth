# WORTH Signal DX Plan

## Purpose

This is the linear execution plan for making `worth-signal` publication-ready
from a DX perspective.

This is **not** an MVP plan.

This plan assumes:

- we are willing to refactor
- we are willing to shrink or reorganize the visible API
- we are willing to clean up naming and layering
- we are willing to improve docs, examples, defaults, and guided workflows
- we are willing to do real production hardening before publish
If a surface is architecturally wrong, fragmented, overly ceremonial, or too
internal to expose confidently, the plan is to fix it rather than rationalize
it.

---

## Inputs

This plan builds on:

- [`plans/worth_signal/worth_signals2.md`](/Users/spenstar/Documents/programming/WORTH/WORTH/plans/worth_signal/worth_signals2.md)
- [`plans/worth_signal/signal_architecture2.md`](/Users/spenstar/Documents/programming/WORTH/WORTH/plans/worth_signal/signal_architecture2.md)
- [`plans/worth_signal/dx_api_matrix.md`](/Users/spenstar/Documents/programming/WORTH/WORTH/plans/worth_signal/dx_api_matrix.md)
- [`plans/worth_signal/dx_export_inventory.md`](/Users/spenstar/Documents/programming/WORTH/WORTH/plans/worth_signal/dx_export_inventory.md)
- [`plans/worth_signal/dx_exposure_cleanup_strategy.md`](/Users/spenstar/Documents/programming/WORTH/WORTH/plans/worth_signal/dx_exposure_cleanup_strategy.md)
- [`plans/worth_signal/dx_export_decision_matrix.md`](/Users/spenstar/Documents/programming/WORTH/WORTH/plans/worth_signal/dx_export_decision_matrix.md)

---

## North Star

The published library should feel like this:

- the primary surface is obvious
- the first success path is fast
- advanced control exists without polluting the default path
- integration power exists without defining the whole product identity
- diagnostics feel premium and trustworthy
- the API expresses semantic intent, not internal ceremony
- users are naturally guided toward batch-safe, policy-safe, and transaction-safe
  usage
- the public surface feels intentionally designed, not accidentally exposed

The dominant mental model for the product should be:

- production incremental runtime for derived computation with strong diagnostics

This is not a domain choice.

It is the center of gravity for the public identity.

Web, geometry, DSL/compiler, ML, and bridge integrations should prove breadth,
but they should not all co-own the first impression.

The library should feel elegant for:

- web development
- geometry kernel development
- DSL / compiler / query-system development
- ML and analysis pipelines
- runtime bridge integrations

---

## Non-Negotiable Standards

These standards apply throughout the plan:

1. Do not publish internal review/certification scaffolding as product API.
2. Do not expose raw internals merely because they exist.
3. Do not keep fragmented APIs when they should be condensed.
4. Do not leave naming debt in public types and methods.
5. Do not leave docs organized around implementation instead of usage.
6. Do not preserve accidental public APIs just to avoid cleanup.
7. Do not ship a product boundary that we would be embarrassed to support.

---

## Completion Criteria

The DX work is only done when all of the following are true:

- the public boundary is deliberate
- all public exports have been consciously classified
- `P3` / Layer `D` surfaces no longer define the product boundary
- high-friction companion-method flows have been condensed where needed
- docs are structured by product journey and layer
- examples cover the main usage modes cleanly
- defaults are safe and persuasive
- tests cover the guided public paths, not just internals
- release packaging and metadata support confident publication

In addition, these design artifacts must exist and be current:

- canonical surface spec
- condensation map
- diagnostics product map
- compatibility / transition plan

---

## Phase 0: Freeze The Standard

Status: Complete

### Goal

Lock the DX standard so later phases do not backslide into convenience-driven
exposure.

### Tasks

- treat [`dx_export_decision_matrix.md`](/Users/spenstar/Documents/programming/WORTH/WORTH/plans/worth_signal/dx_export_decision_matrix.md)
  as the current source of truth
- update it whenever public API decisions change
- require every new public symbol to justify:
  - its layer
  - its action
  - whether it should be raw or guided

### Exit Criteria

- no public API work proceeds without referencing the decision matrix

---

## Phase 0.5: Define The Canonical Product Shape

Status: Complete

### Goal

Commit to the positive canonical forms the user experience will revolve around.

Classification and cleanup are not enough. We need to decide what people are
supposed to memorize.

### Required Deliverable

- [`plans/worth_signal/dx_canonical_surface_spec.md`](/Users/spenstar/Documents/programming/WORTH/WORTH/plans/worth_signal/dx_canonical_surface_spec.md)

### It Must Define

- the one obvious import path
- the one obvious production setup flow
- the one obvious computation-definition flow
- the one obvious batch invalidation flow
- the one obvious diagnostics entry flow
- the one obvious specialist merge orchestration flow
- the explicit role of `easy`

### Exit Criteria

- we have a concrete target public shape, not only a cleanup philosophy

---

## Phase 1: Establish The Published Product Boundary

Status: Complete

### Goal

Define the actual product boundary we intend users to see first.

### Tasks

- decide the final role of:
  - `worth_signal::facade`
  - `worth_signal::easy`
  - `worth_signal::diagnostics`
- define the public top-level product layers:
  - semantic authoring
  - runtime and policy control
  - diagnostics
  - integration-authoring
- explicitly decide which namespaces are:
  - default public path
  - advanced/specialist path
  - internal/support only
- remove ambiguity around whether `facade` is truly the canonical import path

### Mandatory Cleanup

- stop letting broad direct exports undermine curated boundaries
- if `diagnostics` remains directly public, it must be intentionally organized
- `easy` must be positioned as guided entry, not accidental sidecar

### Exit Criteria

- the crate has a clear published boundary model
- there is no confusion about where users should start
- the boundary matches the canonical surface spec rather than only the export
  audit

---

## Phase 2: Remove Internal Leakage

Status: Complete

### Goal

Stop internal/certification/support surfaces from defining the public identity.

### Tasks

- remove or isolate `facade::harness`
- relocate or internalize:
  - harness runtime
  - scenario support
  - parity helpers
  - deployment/certification presets
  - internal boundary contract markers
  - support metrics intended for certification flows
- determine whether these move to:
  - another crate
  - feature-gated support
  - test/dev-only exposure
  - narrower internal modules

### Mandatory Cleanup

- do not leave `P3` surfaces in the main facade â€œfor nowâ€
- if they are not product API, remove them from the product boundary

### Exit Criteria

- Layer `D` is no longer visible in the main product surface

---

## Phase 3: Thin And Reorganize The Core Surface

Status: Complete

### Goal

Make the visible API architecture coherent and compact.

### Tasks

- thin `facade::types`
- stop mixing:
  - primitive vocabulary
  - proof-bearing forms
  - storage-profile tuning
  - reuse/equivalence internals
  - artifact/history plumbing
- split `facade::transaction` conceptually into:
  - runtime operations
  - advanced runtime policy/history
  - bridge/merge integration
- ensure specialist surfaces live under specialist namespaces
- decide whether `facade::performance` remains public, moves, or becomes
  subservient to better guided surfaces
- decide whether `facade::proof` remains public as-is or becomes more narrowly
  integration-facing

### Mandatory Cleanup

- eliminate namespaces that sound foundational but actually contain specialist
  or internal-heavy machinery
- do not preserve current grouping if the grouping itself harms usability

### Exit Criteria

- each namespace has a clear job
- the main facade feels intentionally curated
- namespace cleanup has not replaced the need for better guided public forms

---

## Phase 4: Condense High-Ceremony Flows

Status: Complete

### Goal

Replace memory-based multi-step usage with guided objects and intentional flows.

This phase is not allowed to stop at "less clutter."

The required output is superior canonical workflow shapes.

### Primary Targets

- runtime setup and configuration
- computation definition
- planning and execution orchestration
- batch invalidation orchestration
- branch/snapshot/restore flows
- merge and reconciliation flows
- diagnostics access and rendering flows

### Required Deliverable

- [`plans/worth_signal/dx_condensation_map.md`](/Users/spenstar/Documents/programming/WORTH/WORTH/plans/worth_signal/dx_condensation_map.md)

### The Condensation Map Must Specify

For each high-value workflow family:

- current raw flow
- why it is fragmented or ceremonial
- target abstraction shape
  - builder
  - session
  - request object
  - prepared operation
  - preset bundle
  - raw API retained below guided API
- which raw APIs remain public
- which raw APIs move down a layer

### Tasks

- identify companion-method smells across the public surface
- create guided forms where raw sequences are too easy to misuse
- prefer:
  - builders
  - sessions
  - request structs
  - scoped helpers
  - prepared operation objects
- preserve raw expert controls only where they are architecturally legitimate

### Examples Of What To Look For

- methods that should almost never be called alone
- several policy setters representing one conceptual decision
- raw batch forms used as if they were end-user APIs
- orchestration APIs that force users to remember sequencing

### Mandatory Cleanup

- do not merely document bad ceremony
- if the API shape is wrong, change the API shape

### Exit Criteria

- guided paths exist for the major high-friction workflows
- raw paths are clearly lower-level and specialist
- condensation decisions are concrete, not hand-wavy

---

## Phase 5: Rationalize Policy Surfaces

Primary execution doc:

- [`plans/worth_signal/dx_phase_5_plan.md`](/Users/spenstar/Documents/programming/WORTH/WORTH/plans/worth_signal/dx_phase_5_plan.md)

### Goal

Make policy configuration powerful without becoming fragmented or noisy.

### Tasks

- audit all policy/config knobs across:
  - runtime policy
  - comparator policy
  - condition policy
  - executor policy
  - tier policy
  - checkpoint policy
  - diagnostics retention policy
  - branch/restore/merge policy
- identify overlaps and competing control points
- collapse overlapping knobs into stronger configuration objects where possible
- ensure defaults tell a persuasive story
- ensure low-level policy details do not dominate day-one docs

### Mandatory Cleanup

- do not leave multiple equally â€œofficialâ€ ways to steer the same decision
- make sure each policy layer has a coherent owner

### Compatibility Requirement

- every major policy cleanup must state its migration path while the cleanup is
  being designed, not only at packaging time

### Exit Criteria

- policy surfaces are layered, coherent, and non-fragmented

---

## Phase 6: Rebuild The Diagnostics Experience

### Goal

Make diagnostics a premium capability rather than a flat wall of symbols.

Diagnostics should be designed around user jobs, not around export families.

### Tasks

- restructure diagnostics around clear jobs:
  - inspect
  - explain why this changed
  - compare two runs
  - inspect runtime health
  - trace replay / lineage / history
- decide what remains in the general diagnostics layer vs narrower forensic or
  lineage-focused namespaces
- create guided diagnostics access patterns where useful
- ensure renderers and summaries are easy to discover
- ensure retention/policy knobs are understandable and correctly scoped

### Required Deliverable

- [`plans/worth_signal/dx_diagnostics_product_map.md`](/Users/spenstar/Documents/programming/WORTH/WORTH/plans/worth_signal/dx_diagnostics_product_map.md)

### Mandatory Cleanup

- diagnostics should feel powerful, not sprawling
- direct `worth_signal::diagnostics` exposure must not remain more coherent than
  the curated public story

### Exit Criteria

- diagnostics are a selling point, not an intimidation point
- diagnostics are organized by jobs users actually perform

---

## Phase 7: Rebuild The Documentation Architecture

### Goal

Make docs reflect the product, not the internal module tree.

### Required Documentation Layers

- Quick Start
- Production Runtime
- Advanced Control
- Diagnostics and Forensics
- Integration Authoring
- State History: snapshots, replay, branches, merge
- Performance and batch-oriented execution

### Tasks

- rewrite top-level crate docs
- define the official import and getting-started story
- reorganize the existing docs set into a more progressive structure
- remove doc emphasis from internal/certification support
- ensure every major product path has at least one clean example
- ensure advanced docs clearly signal prerequisites and intended audience

### Mandatory Cleanup

- do not leave docs anchored on implementation detail or internal taxonomy
- do not assume users will infer the preferred path from symbol names alone
- the primary docs must reflect the canonical surface spec, not just the module
  tree

### Exit Criteria

- docs tell a progressive story with clear layer boundaries

---

## Phase 8: Build The Example Suite

### Goal

Demonstrate the product clearly across the domains we actually care about.

### Required Example Families

- minimal web/reactive usage
- production runtime with transactions and batching
- geometry-kernel style partial recomputation
- DSL/compiler/query-style incremental computation
- ML / analysis pipeline style staged evaluation
- diagnostics and explanation walkthrough
- integration/bridge-facing example

### Product Identity Rule

- examples should demonstrate breadth
- they should not make the library feel like it has five competing identities
- the examples should all reinforce the same core mental model:
  production incremental runtime for derived computation with strong diagnostics

### Tasks

- create examples that use the guided paths, not just raw internals
- ensure examples emphasize the intended mental model
- ensure advanced examples justify advanced APIs

### Mandatory Cleanup

- examples must not train users onto accidental or deprecated usage patterns

### Exit Criteria

- examples make the product legible in multiple domains

---

## Phase 9: Public API Naming And Ergonomics Sweep

### Goal

Remove naming debt and eliminate avoidable friction before publication.

### Tasks

- audit public type names
- audit public method names
- audit builder method names
- audit config/property naming symmetry
- normalize terminology across:
  - node
  - computation
  - plan
  - stage
  - transaction
  - snapshot
  - branch
  - merge
  - diagnostics
- remove names that expose implementation detail rather than semantic intent
- add convenience aliases or constructors where justified

### Mandatory Cleanup

- do not publish names that require internal architecture knowledge to decode

### Exit Criteria

- public naming feels cohesive and productized

---

## Phase 9.5: Lock The Role Of `easy`

### Goal

Resolve ambiguity around `easy` before publication.

### Decision Required

Choose one and commit:

1. `easy` is the explicit first-15-minutes guided path and intentionally not
   production-shaped
2. `easy` is a thin alias layer over the real product surface
3. `easy` is a subordinate demo-oriented sidecar

### Mandatory Rule

- do not publish with ambiguity about whether users should learn `easy` first
  and later relearn everything

### Exit Criteria

- the role of `easy` is explicit in code organization, docs, and examples

---

## Phase 10: Safety, Errors, And Defaults Hardening

### Goal

Ensure the public product is safe and graceful under misuse.

### Tasks

- audit typed error surfaces
- improve error messages for common misuses
- ensure builders prevent incomplete or misleading configuration where possible
- audit panic/expect usage in user-facing paths
- ensure default behavior is safe and unsurprising
- ensure raw/advanced paths do not silently undermine correctness

### Mandatory Cleanup

- do not rely on docs alone where the type system or API shape can prevent bad
  use

### Exit Criteria

- misuse paths fail clearly and predictably

---

## Phase 10.5: Compatibility And Transition Strategy

### Goal

Prevent cleanup work from stalling under fear of breakage.

### Required Deliverable

- [`plans/worth_signal/dx_compatibility_transition_plan.md`](/Users/spenstar/Documents/programming/WORTH/WORTH/plans/worth_signal/dx_compatibility_transition_plan.md)

### Tasks

- define unstable or specialist namespaces explicitly where needed
- define deprecation ladders where appropriate
- identify codemod-able renames and migration patterns
- ensure guided-path examples replace deprecated flows
- decide when to use:
  - deprecation
  - containment
  - immediate removal
  - feature-gated transition

### Exit Criteria

- cleanup decisions are not blocked by vague compatibility anxiety

---

## Phase 11: Test The Public Product, Not Just The Internals

### Goal

Make sure the actual published experience is protected by tests.

### Tasks

- add tests for the guided public API paths
- add tests for key docs/examples
- add regression tests around:
  - namespace exposure
  - guided runtime setup
  - batch workflows
  - diagnostics entry flows
  - branch/snapshot guided flows
- ensure cleanup work does not only preserve internals while breaking product UX

### Mandatory Cleanup

- test the paths we want users to use
- not only the raw subsystems

### Exit Criteria

- the intended public workflows are regression-protected

---

## Phase 12: Packaging And Publication Readiness

### Goal

Make the crate ready to publish confidently.

### Tasks

- finalize crate metadata
- audit feature flags
- decide publication shape for support surfaces
- ensure generated docs land in the right order
- ensure examples compile and represent current API
- ensure changelog / migration notes exist if needed
- ensure the crate boundary is coherent for future open source consumption

### Mandatory Cleanup

- do not publish with unresolved uncertainty about what the product boundary is

### Exit Criteria

- the crate is operationally ready to publish

---

## Phase 13: Final Release Gate

### Goal

Refuse to ship a product boundary that still feels accidental.

### Release Questions

Before publication, the answer to all of these must be yes:

1. Is the primary import path obvious?
2. Is the default workflow elegant?
3. Are advanced controls available without cluttering the default path?
4. Are bridge/integration surfaces contained rather than dominant?
5. Are internal/certification surfaces out of the product identity?
6. Are docs and examples strong enough to sell the product honestly?
7. Are naming, defaults, and errors good enough to support publicly?
8. Are we willing to maintain the visible surface we are publishing?

### Exit Criteria

- publication approval is an explicit decision, not inertia

---

## Practical Execution Rule

Work phase-by-phase, but do not use phase boundaries as excuses to leave obvious
debt behind.

If a refactor becomes clearly necessary while executing an earlier phase:

- do it
- update the decision docs
- keep the public boundary improving as we go

This plan is explicitly anti-procrastination.

---

## Suggested Immediate Starting Sequence

1. Phase 0.5: define the canonical product shape
2. Phase 1: establish the final product boundary model
3. Phase 2: remove internal leakage
4. Phase 3: thin and reorganize the core surface
5. Phase 4: condense high-ceremony flows
6. Phase 6: define diagnostics as product jobs early
7. Phase 7: rebuild docs in parallel once the boundary starts stabilizing

That sequence gives the highest leverage:

- first decide what the product is
- then stop leaking what it is not
- then make the remaining product elegant

---

## Bottom Line

The goal is not merely to make `worth-signal` acceptable to publish.

The goal is to make it feel inevitable:

- sharp boundary
- elegant API
- powerful control
- disciplined layering
- premium diagnostics
- no accidental public surface
