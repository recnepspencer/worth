# WORTH Relational DX Phase 1 Plan

## Purpose

Phase 1 is the published-boundary cleanup pass.

This is where we stop saying "the facade should feel more productized" and
start making the actual public boundary behave that way.

The goal is not to rename everything in one giant thrash.

The goal is:

- make the primary public doors obvious
- stop fake seams from pretending to be real lanes
- promote the real under-documented lanes that actually matter
- stop support or specialist structure from setting the public vibe

This phase is where Relational stops being "audited and well-described" and
starts being "boundary-shaped on purpose."

---

## Inputs

This phase builds on:

- [`dx_plan.md`](./dx_plan.md)
- [`dx_phase_0_5_review.md`](./dx_phase_0_5_review.md)
- [`dx_export_decision_matrix.md`](./dx_export_decision_matrix.md)
- [`dx_method_decision_matrix.md`](./dx_method_decision_matrix.md)
- [`dx_boundary_cleanup_list.md`](./dx_boundary_cleanup_list.md)
- [`dx_canonical_surface_spec.md`](./dx_canonical_surface_spec.md)
- [`dx_boundary_spec.md`](./dx_boundary_spec.md)
- [`dx_phase_1_boundary_delta.md`](./dx_phase_1_boundary_delta.md)

---

## North Star

After Phase 1, the public boundary should feel like this:

- there is one obvious setup door
- there is one obvious write-truth door
- there is one obvious read-truth door
- there is one obvious inspect-what-happened door
- specialist lanes are still real, but they do not hijack first contact
- support scaffolding is not shaping the public product identity

An AI agent looking at the surface should have a good shot at choosing the
right door on the first try.

That is the bar.

---

## Scope

Phase 1 is boundary work.

It is not yet the big condensation phase.

### In Scope

- runtime seam cleanup
- removal of fake or empty public backdoors
- promotion of real lanes that already have coherent public jobs
- public-surface role clarification for:
  - `RelationalRuntimeApi`
  - `facade::runtime`
  - `facade::history`
  - `facade::inspection`
  - `facade::publication`
  - `facade::diagnostics`
- `harness` de-emphasis or removal from the main public story
- facade and docs updates needed so the boundary story matches the code

### Out Of Scope

- full flow condensation for setup, transactions, query, and diagnostics
- final naming sweep across the whole crate
- compatibility strategy and deprecation mapping
- broad docs rewrite
- bridge-facing API design

Those come next.

---

## Boundary Targets

Phase 1 should leave us with a boundary that has these layers in practice:

### Layer 1. Primary Runtime Story

- setup
- write truth
- read truth
- inspect what happened

### Layer 2. Guided Operational Readback

- history
- publication
- inspection
- validation readback
- config inspection

### Layer 3. Contained Specialist Power

- merge
- replay
- durability
- commit strategies
- compiled artifact workflows
- retention control

### Layer 4. Not Part Of The Public Story

- harness-first support scaffolding
- empty shells
- runtime backdoors
- substrate mutation leakage

---

## Current Starting Point

Phase 1 is not starting from zero.

We already made a first code cleanup pass that:

- removed `MergeAccess::runtime()`
- narrowed `publication_authority()` to crate-only
- narrowed `storage_authority()` to crate-only
- narrowed `lineage_access()` to crate-only
- narrowed `lineage_authority()` to crate-only
- promoted runtime-facing exports for:
  - `InvariantAccess`
  - `SimulationAccess`
  - `SimulationAuthority`
  - `VisibilityReadContext`
  - `VisibilityRetentionAuthority`

That work matters because it means Phase 1 is already started in code.

What is still missing is the cohesive public-boundary pass that makes those
calls add up to one visible product story.

Live-code checkpoint:

- [`dx_phase_1_boundary_delta.md`](./dx_phase_1_boundary_delta.md)

---

## Mandatory Questions To Close

Phase 1 has to answer these directly:

1. Is `harness` still part of the main public facade story?
2. What is the official read-truth lane?
3. What is the official validation readback lane?
4. What is the official compiled-artifact lane?
5. What is the official retention lane?
6. Which runtime helper seams are real public lanes versus leftovers?
7. Does the facade still present too many equally loud subsystem doors?

If any of those are still fuzzy, Phase 1 is still open.

---

## Work Plan

Phase 1 should run in strict order.

### Step 1. Freeze The Live Boundary Delta

Capture what has already changed in code versus what is still just planned.

Output:

- a short implementation status section added to the phase review later

Exit condition:

- we know which boundary decisions are already real in code
- we know which ones are still docs-only

### Step 2. Finish Fake-Seam Removal

Confirm the removal set is fully handled and consistent:

- `publication_authority`
- `storage_authority`
- `lineage_access`
- `lineage_authority`
- `MergeAccess::runtime`

Audit for any remaining helper seams that have the same smell.

Output:

- no obvious public backdoors left unclassified

Exit condition:

- fake seams are either gone or explicitly queued with a reason

### Step 3. Promote Real Lanes Into Honest Boundary Concepts

Take the promoted seam set and make the public story admit them on purpose:

- `visibility_reads`
- `invariant_access`
- `simulation_access`
- `simulation_authority`
- `retention_authority`

This does not require final perfect names yet.

It does require deciding where they live in the public boundary and how they
are described.

Output:

- explicit lane ownership for each promoted seam

Exit condition:

- the promoted seams no longer feel like random runtime side doors

### Step 4. Set The Runtime Role Hierarchy

Make the role of these surfaces explicit in code-facing docs and public shape:

- `RelationalRuntimeApi`
- `facade::runtime`
- `facade::history`
- `facade::inspection`
- `facade::publication`
- `facade::diagnostics`

This is where we make the boundary hierarchy visible instead of leaving every
module equally loud.

Output:

- clear primary versus contained lane hierarchy

Exit condition:

- a new contributor or AI agent can tell which doors are first-path versus
  escalation-path

### Step 5. De-Emphasize Or Remove `harness`

This is the one support surface that most clearly pollutes the public identity.

The phase has to make a real call here.

Options:

- remove it from the main facade story
- gate it more narrowly
- move it behind test/support-only exposure if the code supports that cleanly

Output:

- `harness` no longer shapes the main public story

Exit condition:

- we can describe the public product without mentioning `harness` in the first
  breath

### Step 6. Align Docs With The Live Boundary

Once the code-side calls are stable enough:

- update the phase docs if any lane naming or placement shifted
- make sure `dx_boundary_spec.md` and `dx_canonical_surface_spec.md` still
  match the actual boundary direction

Output:

- docs and code say the same thing

Exit condition:

- we are no longer carrying docs that describe a cleaner boundary than the code
  actually has

### Step 7. Write The Phase 1 Review

The phase ends with a review doc that says:

- what changed
- what boundary leaks were removed
- what lanes were promoted
- what remains intentionally deferred to Phase 2 and Phase 3

Output:

- `dx_phase_1_review.md`

Exit condition:

- we can say with a straight face that the published boundary is explicit and
  documented

---

## Deliverables

Phase 1 is only done when these exist and are current:

- [`dx_boundary_spec.md`](./dx_boundary_spec.md)
- [`dx_boundary_cleanup_list.md`](./dx_boundary_cleanup_list.md)
- `dx_phase_1_review.md`

And the code should reflect the main cleanup calls, not just describe them.

---

## High-Risk Smells

If any of these remain at the end, Phase 1 is not really done:

- an AI agent still sees several equally plausible runtime doors for the same
  job
- promoted seams are technically public but still undocumented and ownerless
- `harness` still reads like part of the main product contract
- fake helper shells still leak out of runtime
- docs describe a boundary hierarchy that the code does not back up
- the read-truth lane is still ambiguous
- the validation lane is still ambiguous

---

## Working Naming Direction

Phase 1 does not need the final word on every literal symbol name.

But it should keep steering toward job-shaped names and lanes.

Good directional pressure:

- `read_truth`
- `write_truth`
- `inspect_what_happened`
- `validation`
- `compiled_artifacts`
- `retention`

This is not about making the API shallow.

It is about making the boundary legible.

---

## Exit Criteria

Phase 1 is complete only when all of the following are true:

- the public boundary hierarchy is explicit
- the fake runtime seams are removed or intentionally resolved
- the promoted runtime seams have real lane ownership
- `harness` no longer shapes the main public identity
- the primary versus contained versus specialist split is visible
- the docs match the live direction closely enough that they can guide code
  work without lying

If any of those are false, the phase is still open.
