# WORTH Signal Docs Publication Audit

## Purpose

This document classifies the current `worth-signal` docs by publication value.

The goal is not to preserve every existing doc. The goal is to decide:

- what should become the public product story
- what should remain as technical reference
- what should be treated as internal design history

This audit reflects the current code and DX direction:

- curated `facade::*` public boundary
- guided daily-use surface
- domain-agnostic runtime identity
- diagnostics as a product feature

---

## Classification Legend

- `Rewrite For Publish`
  - should exist in the published docs set
  - current doc is too drifted, too broad, or too internally framed to ship as-is
- `Keep As Reference`
  - useful and substantially valid
  - may need edits, but not a ground-up rewrite
- `Internal History`
  - valuable to maintainers
  - should not be part of the public onboarding or product narrative

---

## Publish-Facing Docs To Rewrite

These are the docs that should define the public product experience, but they do
not yet match the current intended surface strongly enough.

### `crates/worth-signal/docs/API_SURFACE.md`

Classification:

- `Rewrite For Publish`

Why:

- still reads like a broad inventory rather than a productized path
- overweights executor and lower-level surface early
- does not fully reflect the current canonical center of gravity
- should become the main public map, so it needs a cleaner story

Target replacement:

- `QUICKSTART.md`
- `DAILY_WORKFLOWS.md`
- `API_OVERVIEW.md`

### `crates/worth-signal/docs/ADVANCED_PATTERNS.md`

Classification:

- `Rewrite For Publish`

Why:

- useful topic area, but currently feels like a catch-all
- should be reorganized around advanced jobs, not leftovers

Target replacement:

- `ADVANCED_WORKFLOWS.md`

### `crates/worth-signal/docs/TRANSACTIONS_AND_KEYED_RUNTIME.md`

Classification:

- `Rewrite For Publish`

Why:

- valuable content, but too easy to collapse several different stories together
- transaction and keyed runtime should be documented as deliberate advanced
  workflows, not as a single â€œmiscellaneous power surfaceâ€

Target replacement:

- `TRANSACTIONS.md`
- `KEYED_COMPUTATION.md`

### `crates/worth-signal/docs/SNAPSHOTS_BRANCHES_AND_REPLAY.md`

Classification:

- `Rewrite For Publish`

Why:

- topic is important
- current framing is still state-surface oriented rather than job-oriented
- should reflect guided `history()` / `merge()` / replay usage more clearly

Target replacement:

- `HISTORY_AND_REPLAY.md`

### `crates/worth-signal/docs/HARNESS_AND_CERTIFICATION.md`

Classification:

- `Rewrite For Publish`

Why:

- this should likely not be part of the main public story
- the current doc is already acknowledging that
- if retained publicly at all, it should be clearly marked as specialist support

Target replacement:

- `CERTIFICATION_AND_HARNESS.md` in an advanced or reference section

### `crates/worth-signal/docs/CHECKPOINTS_AND_TIERS.md`

Classification:

- `Rewrite For Publish`

Why:

- useful specialist topic
- should be framed as advanced policy control instead of a flat type tour

Target replacement:

- `RUNTIME_POLICY.md`

### `crates/worth-signal/docs/LINEAGE_MODEL.md`

Classification:

- `Rewrite For Publish`

Why:

- concept is real and important
- â€œlineage modelâ€ is too architecture-first for the main product story
- should sit underneath a more approachable history/replay document

Target replacement:

- part of `HISTORY_AND_REPLAY.md`
- optional deeper `LINEAGE_REFERENCE.md`

---

## Docs To Keep As Reference

These are still useful and close enough to current truth that they should be
kept, though they may need cleanup edits.

### `crates/worth-signal/docs/CONDITIONS_AND_COMPARATORS.md`

Classification:

- `Keep As Reference`

Why:

- focused concept doc
- concrete and likely still useful
- complements, rather than competes with, a quickstart

Edits needed:

- align examples and terminology to current guided wording

### `crates/worth-signal/docs/ARTIFACT_ACCESS_MATRIX.md`

Classification:

- `Keep As Reference`

Why:

- compact and practically useful
- addresses a subtle area users will actually need help with
- good as a diagnostics/reference companion

Edits needed:

- align naming to the newer diagnostics entry story

### `crates/worth-signal/docs/LIFECYCLE_AND_GC.md`

Classification:

- `Keep As Reference`

Why:

- focused operational topic
- not first-read material, but good reference material

Edits needed:

- verify current API examples and terminology

---

## Internal Design History

These documents are valuable, but they should not be treated as public product
docs.

### DX planning and audit docs

- `plans/worth_signal/dx_api_matrix.md`
- `plans/worth_signal/dx_boundary_spec.md`
- `plans/worth_signal/dx_canonical_surface_spec.md`
- `plans/worth_signal/dx_compatibility_transition_plan.md`
- `plans/worth_signal/dx_condensation_map.md`
- `plans/worth_signal/dx_diagnostics_product_map.md`
- `plans/worth_signal/dx_export_decision_matrix.md`
- `plans/worth_signal/dx_export_inventory.md`
- `plans/worth_signal/dx_exposure_cleanup_strategy.md`
- `plans/worth_signal/dx_plan.md`
- `plans/worth_signal/dx_wording_map.md`

Why:

- internal execution scaffolding
- should guide implementation, not public onboarding

### Vision and architecture docs

- `plans/worth_signal/worth_signal_vision.md`
- `plans/worth_signal/worth_signals2.md`
- `plans/worth_signal/signal_architecture.md`
- `plans/worth_signal/signal_architecture2.md`
- `plans/worth_signal/signal_compile_time_safety.md`

Why:

- important maintainer docs
- too architecture-heavy and historically layered for the main public story

### Milestone and substrate docs

- `plans/worth_signal/milestone-2.md`
- `plans/worth_signal/milestone-2-field-classification.md`
- `plans/worth_signal/milestone-3.md`
- `plans/worth_signal/milestone-4.md`
- `plans/worth_signal/milestone-4-access-matrix.md`
- `plans/worth_signal/milestone-4-interior-heat-audit.md`
- `plans/worth_signal/s9_16_acceptance_map.md`
- `plans/worth_signal/s9_missing_substrate_completion.md`
- `plans/worth_signal/test-requirements.md`

Why:

- implementation history and engineering control docs
- useful to maintainers only

### Performance planning docs

- `plans/worth_signal/performance.md`
- `plans/worth_signal/signal_performance.md`
- `plans/worth_signal/signal_performance_architecture.md`
- `plans/worth_signal/signal_performance_baseline.md`

Why:

- important engineering references
- not appropriate as first-line publish docs

---

## Recommended Published Docs Set

This is the docs set I would actually target for publication.

### Tier 1: first-read product docs

- `README` / crate-level landing docs
- `QUICKSTART.md`
- `DAILY_WORKFLOWS.md`
- `API_OVERVIEW.md`
- `DIAGNOSTICS.md`

### Tier 2: important specialist docs

- `CONDITIONS_AND_COMPARATORS.md`
- `RUNTIME_POLICY.md`
- `TRANSACTIONS.md`
- `KEYED_COMPUTATION.md`
- `HISTORY_AND_REPLAY.md`

### Tier 3: deeper reference docs

- `ARTIFACT_ACCESS_MATRIX.md`
- `LIFECYCLE_AND_GC.md`
- `CERTIFICATION_AND_HARNESS.md`
- optional `LINEAGE_REFERENCE.md`

---

## Rewrite Priority

### Priority 0

Must exist before publication:

- `QUICKSTART.md`
- `DAILY_WORKFLOWS.md`
- `DIAGNOSTICS.md`
- rewritten `API_OVERVIEW.md`

### Priority 1

Should exist before publication:

- `TRANSACTIONS.md`
- `KEYED_COMPUTATION.md`
- `HISTORY_AND_REPLAY.md`
- `RUNTIME_POLICY.md`

### Priority 2

Can follow immediately after:

- `CERTIFICATION_AND_HARNESS.md`
- `LINEAGE_REFERENCE.md`
- cleanup edits to `ARTIFACT_ACCESS_MATRIX.md`
- cleanup edits to `LIFECYCLE_AND_GC.md`

---

## Bottom Line

The docs do need a significant rewrite.

The code and tests are now in a stable enough place that we should stop trying
to make old docs â€œgood enoughâ€ and instead build a new publish-facing docs set
that matches the product shape we actually intend to ship.
