# WORTH Runtime Bridge DX Compatibility Transition Plan

## Purpose

This document defines how the bridge should move from its current public shape
to the intended DX shape without fear-driven hesitation and without accidental
breakage.

The bridge already has real users in tests, docs, and internal workflows.

That means DX cleanup needs a compatibility strategy, not just a desired end
state.

---

## Governing Rule

We should preserve meaningful compatibility where it protects real users.

We should not preserve awkwardness forever just because it already exists.

The compatibility goal is:

- staged convergence toward the intended product boundary

not:

- permanent dual-surface sprawl

---

## Compatibility Principles

### 1. Protect The Canonical Future Shape

If there is tension between:

- preserving every old seam forever
- making the bridge read like one deliberate product

the canonical future shape wins.

### 2. Add Before Removing When The Old Surface Is Still Used

When an existing raw or awkward path is still in active use:

- add the guided replacement first
- move docs and examples to it
- move integration tests to it
- then demote or de-emphasize the old path

### 3. Do Not Let Specialist Surfaces Pretend To Be Everyday

A specialist surface can remain public and stable.

That does not mean:

- it should still appear in Tier 1 docs
- it should still be presented as the normal path

### 4. Compatibility Is Not Just API Signatures

The bridge also has compatibility obligations for:

- docs
- examples
- test memory shape
- diagnostics naming
- replay/certification expectations

---

## Surface Classes

### Class A: Canonical Going Forward

These should gain the strongest stability expectations.

Examples:

- `RuntimeBridge::builder()`
- `with_truth_source(...)`
- `with_compute_sink(...)`
- `route(...)`
- `evaluate_current(...)`
- `evaluate(...)`
- `speculate(...)`
- session `discard(...)`
- session `promote(...)`
- `diagnostics()`

Plan:

- use in docs first
- use in integration tests first
- stabilize naming aggressively

### Class B: Advanced But Supported

These remain public and meaningful, but not day-one.

Examples:

- explicit truth-view selectors
- bulk planning
- stream control
- structural comparison
- merge orchestration
- advanced writeback configuration

Plan:

- preserve
- document in advanced guides
- keep out of Tier 1 docs

### Class C: Specialist And Raw

These remain public where protocol or certification value requires it.

Examples:

- `validate_*`
- `admit_*`
- `lower_*`
- `canonicalize_*`
- `replay_*`
- raw record-family queries

Plan:

- preserve where real
- classify explicitly as specialist
- stop teaching them as ordinary workflow tools

### Class D: Support-Only Or Non-Product

Examples:

- fixture-only helpers
- harness substrate details
- support seams with no real external product meaning

Plan:

- do not teach publicly as bridge identity
- keep out of first-read docs
- avoid depending on them in ordinary end-to-end tests

---

## Transition Order

### Stage 1: Add The Canonical Path

Do this first:

- ship guided wrappers
- ship diagnostics convenience methods
- ship happy-path examples

### Stage 2: Move Teaching Surfaces

Then:

- rewrite docs to the canonical path
- rewrite integration tests to the canonical path
- rewrite the pricing-shock reference workload to the canonical path

### Stage 3: Demote Old Surfaces

Then:

- mark raw paths as advanced or specialist in docs
- stop using them in ordinary examples
- stop introducing new everyday tests against them

### Stage 4: Tighten And Prune

Only after the canonical path is clearly established:

- consider deprecation notes
- consider narrower exports
- consider stronger grouping/containment

---

## What Must Migrate First

These should migrate to the canonical surface earliest because they shape user
memory the fastest:

1. first-read docs
2. Milestone 13 pricing-shock workload
3. ordinary integration tests
4. diagnostics examples

If these still use old seams, the cleanup is not real yet.

---

## What Can Stay Raw Longer

These can remain raw longer without harming the product boundary, as long as
they are clearly specialist:

- canonical record replay helpers
- merge proof surfaces
- structural reduction details
- family-aware mapper internals
- specialist certification bundle internals

The key is:

- real and public is fine
- accidentally ordinary is not

---

## Deprecation Heuristic

An old surface becomes a deprecation candidate when all of the following are
true:

1. there is a better canonical replacement
2. docs and examples no longer teach the old path
3. ordinary integration tests no longer rely on it
4. the old path adds confusion rather than capability

Until then, demotion in docs is often enough.

---

## Testing Rule During Transition

During the transition, tests should be intentionally split:

- ordinary integration and end-to-end tests target Class A surfaces
- advanced workflow tests target Class B surfaces
- specialist certification tests may target Class C surfaces directly

This prevents the old surface from continuing to define the bridge by inertia.

---

## Documentation Rule During Transition

Tier 1 docs must never show:

- raw phase sequencing as the normal path
- record-family spelunking as the normal diagnostics path
- specialist writeback or replay internals as the normal promotion path

Tier 2 and Tier 3 docs may show those, but only after the canonical story is
already clear.

---

## Immediate Bridge Actions

Based on the current state of the bridge, the next compatibility moves should
be:

1. finish the diagnostics product surface
2. move public docs and examples to the standard path
3. keep raw phase verbs public but clearly specialist
4. avoid adding new ordinary tests against raw phase APIs
5. use the pricing workload as the compatibility audit for whether the
   canonical path is truly sufficient

---

## Completion Test

The compatibility transition is healthy when:

1. the bridge has one clearly taught public memory model
2. old seams still exist only where they add real advanced or specialist value
3. new work naturally lands on the canonical path
4. cleanup can continue without fear that we are breaking the bridge identity

If we still feel compelled to preserve every old awkward path as equally
primary, the compatibility strategy has failed.
