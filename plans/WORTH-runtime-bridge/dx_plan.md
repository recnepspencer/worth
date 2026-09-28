# WORTH Runtime Bridge DX Plan

## Purpose

This is the execution-grade DX implementation guide for making
`worth-runtime-bridge` publication-grade from a boundary, naming, workflow,
diagnostics, and testing perspective.

This plan replaces the narrower pre-certification-only framing.

The decision is now explicit:

- we do not want the minimum bridge cleanup that happens to unblock
  Milestone 13
- we want the maximum bridge DX hardening so the bridge gets one deliberate
  public shape and we do not have to come back later to unwind accidental
  boundary debt

This plan still matters immediately for Milestone 13.
But it is not scoped only to Milestone 13.

It is the bridge equivalent of the stronger DX programs already executed for
`worth-signal` and `worth-relational`.

This document is not only a design statement.
It is the required execution order for the remaining bridge DX hardening work.
If implementation work and this sequence disagree, this document must be
updated first.

---

## Starting Reality

The bridge is not starting from scratch.

The current baseline now includes:

- milestones 1 through 12 landed as real bridge protocol work
- [`milestone-12b.md`](./milestone-12b.md)
  establishing bridge-native extensible writeback families and mapper
  containment
- a strong internal subsystem decomposition
- a real facade boundary in code
- a large diagnostics and replay artifact story
- a real standard path for build, route, evaluate, speculate, discard or
  promote, and inspect
- a real docs spine and rustdoc teaching layer
- thin pricing-shock workload lanes already proving the hardened path against
  real bridge behavior

What it still lacks is a strict execution order for the remaining work.

Without that execution order, the most likely failure mode is:

- the bridge keeps shipping real capability
- tests keep proving real internal correctness
- but code-surface cleanup, docs, and certification work keep proceeding in
  parallel
- Milestone 13 then certifies whichever seams happened to be convenient
- and later cleanup becomes more expensive because tests, examples, and
  integrations all depend on today’s accidental sequence

That is exactly the failure mode this plan is designed to prevent.

---

## Inputs

This plan builds on:

- [`plans/worth-runtime-bridge/worth_runtime_bridge_roadmap.md`](./WORTH_runtime_bridge_roadmap.md)
- [`plans/worth-runtime-bridge/worth_runtime_bridge_vision.md`](./WORTH_runtime_bridge_vision.md)
- [`plans/worth-runtime-bridge/test-requirements.md`](./test-requirements.md)
- [`plans/worth-runtime-bridge/milestone-12b.md`](./milestone-12b.md)
- [`plans/worth-runtime-bridge/milestone-13.md`](./milestone-13.md)
- [`plans/worth-runtime-bridge/dx_canonical_surface_spec.md`](./dx_canonical_surface_spec.md)
- [`plans/worth-runtime-bridge/dx_boundary_spec.md`](./dx_boundary_spec.md)
- [`plans/worth-runtime-bridge/dx_boundary_cleanup_spec.md`](./dx_boundary_cleanup_spec.md)
- [`plans/worth-runtime-bridge/dx_standard_path_spec.md`](./dx_standard_path_spec.md)
- [`plans/worth-runtime-bridge/dx_diagnostics_product_map.md`](./dx_diagnostics_product_map.md)
- [`plans/worth-runtime-bridge/dx_wording_map.md`](./dx_wording_map.md)
- [`plans/worth-runtime-bridge/dx_compatibility_transition_plan.md`](./dx_compatibility_transition_plan.md)
- [`plans/worth-runtime-bridge/dx_public_surface_audit.md`](./dx_public_surface_audit.md)
- [`plans/worth_signal/dx_plan.md`](../WORTH_signal/dx_plan.md)
- [`plans/worth_signal/dx_boundary_spec.md`](../WORTH_signal/dx_boundary_spec.md)
- [`plans/worth-relational/dx_plan.md`](../WORTH-relational/dx_plan.md)
- [`plans/worth-relational/dx_boundary_spec.md`](../WORTH-relational/dx_boundary_spec.md)
- [`crates/worth-runtime-bridge/src/lib.rs`](../../crates/worth-runtime-bridge/src/lib.rs)
- [`crates/worth-runtime-bridge/src/facade.rs`](../../crates/worth-runtime-bridge/src/facade.rs)

---

## Adversarial Constraint

The bridge DX program must survive this hostile condition:

> A new engineer or AI agent must be able to build a bridge, route truth
> changes, evaluate against current or branch-local truth views, open and
> manage speculative sessions, understand multi-family writeback promotion
> boundaries, inspect diagnostics, replay canonical bridge artifacts, and write
> end-to-end certification workflows without learning the internal subsystem
> tree first and without stitching phase-level methods together by guesswork.
> If ordinary bridge jobs still require direct assembly of validation,
> admission, lowering, canonicalization, replay, or record-specific diagnostics
> methods, the public bridge boundary is still accidental and the DX program has
> failed.

This constraint is load-bearing because:

- Milestone 12b expanded the real bridge surface materially
- Milestone 13 will certify the bridge end to end
- a half-cleaned boundary would simply move the later cleanup cost into a more
  brittle part of the roadmap

The execution-order version of that same constraint is:

> The bridge team must be able to finish the remaining DX hardening without
> wandering between docs, facade cleanup, and certification work out of order.
> If the next step is not obvious, the plan is incomplete. If later phases can
> start before earlier phases establish their public boundary decisions and
> stop conditions, the bridge will regress into accidental API growth and
> accidental certification targets.

---

## North Star

The published bridge should feel like this:

- one obvious import path
- one obvious setup flow
- one obvious route and evaluate story
- one obvious speculative-session story
- one obvious discard or promote story
- one obvious diagnostics and certification entrypoint
- advanced bridge control exists without polluting the first impression
- writeback-family power, merge semantics, structural comparison, replay, and
  stream protocol remain strong but clearly specialist
- end-to-end tests exercise the real public bridge, not its accidental seams

The dominant mental model should be:

- causal protocol boundary between authoritative truth and derived computation

Not:

- a bag of milestone nouns
- a re-export inventory
- a test harness that happens to have a facade

---

## Non-Negotiable Standards

1. Do not let host adapters define the bridge product boundary.
2. Do not let certification fixtures define the bridge product boundary.
3. Do not keep raw subsystem export walls as a substitute for design.
4. Do not make users memorize milestone-local phase chains for ordinary work.
5. Do not let record inventory dominate the bridge product story.
6. Do not preserve awkward public APIs just because they already exist.
7. Do not settle for "enough DX to unblock Milestone 13" if the shape is still
   obviously wrong.
8. Do not let docs run ahead of code stability.
9. Do not reopen broad DX polish unless real workload pressure exposes a
   concrete boundary problem.

---

## Completion Criteria

The DX work is only done when all of the following are true:

- the bridge has a deliberate canonical surface
- the bridge has a deliberate boundary spec
- ordinary bridge jobs have obvious guided flows
- advanced and specialist flows are powerful but contained
- diagnostics are organized around user jobs rather than record families
- naming and grouping are coherent enough that new engineers and AI agents do
  not have to infer usage by subsystem vibes
- examples and tests target the intended public bridge flows
- Milestone 13’s pricing-shock reference workload can be expressed through the
  intended public bridge path
- the bridge is hard enough to support publicly without embarrassment

In addition, these artifacts must exist and stay current:

- bridge DX canonical surface spec
- bridge DX boundary spec
- bridge DX boundary cleanup spec
- bridge DX public surface audit
- bridge standard path spec
- bridge wording map
- bridge diagnostics product map
- bridge compatibility transition plan
- bridge docs publication audit
- bridge docs information architecture
- this DX plan

In addition, the plan is only complete as an implementation guide when:

- every remaining phase has a clear goal
- every phase has explicit must-ship outputs
- every phase has explicit stop conditions
- every phase says what work is forbidden until it finishes
- Milestone 13 implementation can point to one current phase instead of
  vaguely "continuing DX"

---

## Canonical Daily-Use Jobs

The bridge boundary must optimize for these jobs first:

1. build bridge
2. route truth change
3. evaluate against truth view
4. open speculative session
5. discard or promote speculative outcome
6. inspect what happened
7. export or compare bridge certification artifacts

If those jobs are not smooth, the bridge is not done from a DX perspective.

The standard-path ergonomic target for those jobs is defined in:

- [`dx_standard_path_spec.md`](./dx_standard_path_spec.md)

---

## Canonical Public Layers

### Layer 1: Everyday Bridge Operations

This should define the bridge product.

Required characteristics:

- short import story
- obvious setup path
- obvious route, evaluate, and speculate verbs
- obvious diagnostics door

### Layer 2: Advanced Runtime Control

This is for:

- policy refinement
- branch and historical view selection
- bulk planning
- stream coordination
- structural comparison
- merge-aware reads
- advanced writeback configuration

### Layer 3: Specialist Infrastructure

This is for:

- writeback-family internals
- stream protocol detail
- raw replay and canonicalization detail
- merge and structural proof surfaces
- host-adapter authoring
- certification bundle internals

### Layer 4: Internal Support

This should not define the product.

Includes:

- test-only harness substrate
- support-only helpers
- scaffolding that does not correspond to a real bridge job

---

## Foundation Work Already Complete

The following foundation work already exists and is treated as completed DX
baseline rather than future planning:

- canonical surface definition
- boundary definition
- cleanup-target definition
- diagnostics product map
- wording map
- compatibility transition plan
- public surface audit
- initial Tier 1 and Tier 2 docs spine
- standard-path facade implementation
- initial pricing-shock workload slices

This matters because the next execution phases are not "invent the bridge DX
strategy."
They are "finish the remaining hardening in the right order."

---

## Execution Policy

The remaining bridge DX work must now follow this rule:

1. finish the current phase
2. prove its exit criteria
3. only then start the next phase

Do not open broad new docs work, new public API growth, or new certification
matrix breadth just because there is momentum.

If work appears to belong to multiple phases, it belongs to the earliest phase
whose exit criteria are not yet satisfied.

---

## Execution Phases

## Phase 1: Freeze The Product Boundary

### Why this phase exists

The bridge can now describe its product shape, but if we continue changing code
and tests without a locked public-surface decision, we will keep accidentally
teaching compatibility baggage and specialist seams.

### Goal

Freeze what the bridge is trying to be before widening implementation or
certification breadth again.

### Must Ship

- treat the canonical surface, boundary, cleanup, wording, diagnostics, and
  compatibility docs as one active authority set
- treat [`dx_public_surface_audit.md`](./dx_public_surface_audit.md)
  as the authoritative classification of:
  - canonical
  - advanced
  - specialist
  - compatibility-heavy
- ensure all remaining public-surface edits can be justified against that
  classification
- ensure Milestone 13 sequencing points to this plan as the execution-order
  authority

### Must Preserve

- no regression of the existing standard-path surface
- no silent promotion of specialist APIs into ordinary examples
- no new product-defining surfaces from harness-only needs

### Exit Criteria

- one engineer can answer "what is canonical versus advanced versus specialist"
  without inspecting the code tree
- new bridge-facing work has one authority set to consult before edits
- no remaining DX task depends on guessing phase order

### Forbidden Until Done

- do not widen the pricing certification matrix
- do not add broad new public docs
- do not add new top-level public API families

---

## Phase 2: Finish Code-Surface Hardening

### Why this phase exists

The standard path is much stronger now, but the bridge is not done until the
advanced and specialist lanes are also deliberately contained and the
compatibility surface stops dominating discovery.

### Goal

Make the code surface itself stable enough that the docs and certification work
are describing a boundary we actually trust.

### Must Ship

- finish containment and intent-bearing rustdoc across the remaining advanced
  and specialist facade surfaces
- keep [`facade.rs`](../../crates/worth-runtime-bridge/src/facade.rs)
  as the single teaching surface, with standard-path, explicit-control, and
  specialist categories explained inside that one root API
- de-emphasize compatibility-heavy flat exports rather than letting them become
  the default memory path
- add compile-checked examples for the highest-value advanced flows:
  - policy control
  - truth-view or historical control
  - stream coordination
  - source materialization
  - structural or merge comparison where truly public
- ensure raw phase verbs remain clearly specialist

### Must Preserve

- compatibility remains available during transition
- specialist power remains real for replay, proof, and certification work
- no false simplification that hides cost or authority boundaries

### Exit Criteria

- the code surface teaches the intended public layers without needing the
  markdown docs to compensate for confusion
- advanced examples compile and reflect real supported usage
- no new ordinary tests are written against raw phase APIs
- the flat compatibility wall is no longer the dominant discovery path

### Forbidden Until Done

- do not call bridge DX finished
- do not treat the markdown docs as stable final teaching material
- do not expand Milestone 13 beyond thin workload-pressure lanes

---

## Phase 3: Prove The Boundary With Real Workloads

### Why this phase exists

A hardened boundary that only looks good in docs is not hardened.
The real judge is whether the pricing-shock reference workload and hostile
certification lanes can use the intended public shape without reaching through
awkward seams.

### Goal

Shift from DX-by-opinion to DX-by-real-use, using the Milestone 13 reference
workload as the boundary judge.

### Must Ship

- keep growing the Rust-only pricing-shock workload only through:
  - canonical flows for ordinary jobs
  - explicitly advanced flows where the scenario truly requires them
  - specialist flows only when certifying specialist behavior itself
- turn the existing workload slices into a certification-oriented matrix for:
  - live high-fanout churn
  - speculative branch isolation
  - discard zero-residue
  - commit promotion
  - replay and restart parity
  - diagnostics-tier variation
  - failure injection
- converge those lanes into one top-level pricing-shock workload certification
  bundle so Phase 4 docs teach one real artifact model instead of a bag of
  scenario names
- use workload friction as the only valid reason to reopen the DX surface
- keep integration and end-to-end tests as the center of gravity, with only
  minimal unit tests for pure load-bearing logic

### Must Preserve

- the bridge remains boundary-honest rather than domain-owning
- truth authority stays in `worth-relational`
- compute authority stays in `worth-signal`
- diagnostics richness never changes causal meaning

### Exit Criteria

- the pricing-shock matrix can express the standard bridge story without
  ad hoc subsystem stitching
- the pricing-shock workload exposes one nested bundle shape for ordinary,
  hostile, lifecycle, fanout, replay, and restart evidence
- workload failures localize to explicit bridge surfaces rather than "the docs
  were misleading"
- DX changes become exception-driven rather than open-ended

### Forbidden Until Done

- do not widen the public docs or reference story again unless workload
  pressure reveals a real mismatch
- do not introduce new guided API families without workload evidence

---

## Phase 4: Finalize Publication And Certification Teaching

### Why this phase exists

Only after the code surface and workload pressure agree do we get to call the
docs and certification teaching spine stable.

### Goal

Finish the public teaching layer around a boundary that has already survived
real workload pressure.

### Must Ship

- keep the Tier 1, Tier 2, and reference docs aligned with the final hardened
  public surface
- maintain compile-checked rustdoc for canonical and important advanced flows
- ensure examples and docs teach jobs first and specialist detail second
- ensure the certification docs and public diagnostics story match the actual
  Milestone 13 harness and artifact surfaces
- ensure Phase 4 docs describe the real pricing-shock workload bundle shape now
  produced by the harness

### Must Preserve

- docs must not get ahead of code stability
- examples must not silently rely on compatibility baggage
- publication quality must not erase architectural honesty

### Exit Criteria

- public docs, rustdoc, and certification lanes all teach the same bridge
  boundary
- the bridge can be taught without caveating that the "real API" lives
  somewhere else
- DX hardening is complete enough that future work is normal product evolution,
  not boundary rescue

---

## Execution Order Summary

The remaining bridge DX hardening must proceed in this order:

1. Phase 1: freeze the product boundary
2. Phase 2: finish code-surface hardening
3. Phase 3: prove the boundary with real workloads
4. Phase 4: finalize publication and certification teaching

This is now the official execution order.

Milestone 13 implementation is allowed to pressure Phase 2 and Phase 3.
It is not allowed to skip them.

---

## Immediate Next-Step Rule

As of the current bridge state, the plan assumes:

- the foundation DX docs exist
- the standard path exists
- much of the code-surface hardening is already in progress
- thin pricing-shock workload lanes already exist

Therefore the active execution focus is:

- finish Phase 2 where the facade still needs containment or advanced examples
- then continue directly into Phase 3 using the pricing-shock certification
  matrix as the judge

Do not reopen broad docs ideation unless the Phase 3 workload exposes a real
boundary problem.

---

## Bottom Line

The bridge no longer needs a tiny "just enough DX" pass.

It needs the same kind of deliberate, maximal DX hardening that signal and
relational already got:

- deliberate boundary
- deliberate flows
- deliberate diagnostics
- deliberate naming
- deliberate tests
- deliberate execution order

That is the standard this plan now encodes.
