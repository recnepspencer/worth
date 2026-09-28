# WORTH Relational Docs Publication Audit

## Purpose

This document classifies the current Relational docs by publication value.

The goal is not to preserve every existing doc.

The goal is to decide:

- what should become the public product story
- what should remain as technical reference
- what should stay internal design history

This audit reflects the current DX direction:

- facade-first public boundary
- one setup story
- one write-truth story
- one read-truth story
- operator readback organized around jobs
- specialist lanes kept real, but contained

---

## Classification Legend

- `Rewrite For Publish`
  - should exist in the published docs set
  - current material is too architectural, too stale, or too internally framed
    to ship as-is
- `Keep As Reference`
  - useful and materially valid
  - not first-read material, but worth keeping accessible
- `Internal History`
  - valuable to maintainers
  - should not define the public onboarding or product story

---

## Current State

Right now `worth-relational` does not have a clean publish-facing docs set
under [`crates/worth-relational`](../../crates/worth-relational).

Most of the meaningful writing lives under
[`plans/worth-relational`](./).

So this audit is doing two jobs:

1. classify what already exists
2. define the replacement docs set that should exist for publication

---

## Publish-Facing Docs To Rewrite

These topics should absolutely be in the published docs set, but the existing
material is too architecture-first or too execution-history-shaped to ship as
the product story.

### [`worth_relational_vision.md`](./WORTH_relational_vision.md)

Classification:

- `Rewrite For Publish`

Why:

- the ideas are strong, but the doc is huge and architecture-heavy
- great maintainer vision doc, bad first product landing doc
- should feed a tighter public landing story instead of shipping raw

Target replacement:

- crate `README`
- `QUICKSTART.md`
- `API_OVERVIEW.md`

### [`worth_relational_roadmap.md`](./WORTH_relational_roadmap.md)

Classification:

- `Rewrite For Publish`

Why:

- useful engineering control doc
- not a public-facing explanation of how to use the runtime
- should inform public "what exists" docs, not be one

Target replacement:

- feature-status notes in reference docs where needed

### [`relational_architecture.md`](./relational_architecture.md)

Classification:

- `Rewrite For Publish`

Why:

- real architecture reference, but too internal to serve as onboarding
- current public docs should not start with architecture decomposition

Target replacement:

- `API_OVERVIEW.md`
- optional `ARCHITECTURE_REFERENCE.md`

### [`test-requirements.md`](./test-requirements.md)

Classification:

- `Rewrite For Publish`

Why:

- important trust doc, but it is certification-facing, not user-facing
- should inspire a shorter "correctness guarantees" section, not ship raw as
  onboarding

Target replacement:

- part of `API_OVERVIEW.md`
- optional `CORRECTNESS_AND_GUARANTEES.md`

---

## Docs To Keep As Reference

These are useful and close enough to valid truth that they should remain
available, but they should not carry the public onboarding load.

### [`phase-8.md`](./phase-8.md)

Classification:

- `Keep As Reference`

Why:

- milestone closeout/reference material
- useful for maintainers and advanced readers
- not a first-read product doc

### Milestone plans and closeouts

Classification:

- `Keep As Reference`

Files:

- [`milestone-1-closeout.md`](./milestone-1-closeout.md)
- [`milestone-2-plan.md`](./milestone-2-plan.md)
- [`milestone-2-closeout.md`](./milestone-2-closeout.md)
- [`milestone-3-closeout.md`](./milestone-3-closeout.md)
- [`milestone-4-closeout.md`](./milestone-4-closeout.md)
- [`milestone-5-plan.md`](./milestone-5-plan.md)
- [`milestone-5-closeout.md`](./milestone-5-closeout.md)
- [`milestone-6-plan.md`](./milestone-6-plan.md)
- [`milestone-6-closeout.md`](./milestone-6-closeout.md)
- [`milestone-6.5-plan.md`](./milestone-6.5-plan.md)
- [`milestone-6.5-closeout.md`](./milestone-6.5-closeout.md)
- [`milestone-7a.md`](./milestone-7a.md)
- [`milestone-7b-plan.md`](./milestone-7b-plan.md)
- [`milestone-7b-closeout.md`](./milestone-7b-closeout.md)
- [`milestone-7c-authoritative-merge-execution-spec.md`](./milestone-7c-authoritative-merge-execution-spec.md)
- [`milestone-7d-deletion-and-topology-merge-execution-spec.md`](./milestone-7d-deletion-and-topology-merge-execution-spec.md)
- [`milestone-7d-closeout.md`](./milestone-7d-closeout.md)
- [`milestone-8-plan.md`](./milestone-8-plan.md)
- [`milestone-8-closeout.md`](./milestone-8-closeout.md)
- [`milestone-8.5-plan.md`](./milestone-8.5-plan.md)

Why:

- valuable implementation and proof history
- good reference for advanced users and maintainers
- too detailed and too roadmap-shaped for product onboarding

### [`relational_compile_time_safety.md`](./relational_compile_time_safety.md)

Classification:

- `Keep As Reference`

Why:

- focused advanced concept doc
- useful if someone is specifically evaluating type-safety posture
- too deep for first-read docs

---

## Internal Design History

These docs should guide maintainers and cleanup work, but they should not be
treated as public product docs.

### DX planning and review docs

- [`dx_plan.md`](./dx_plan.md)
- [`dx_export_inventory.md`](./dx_export_inventory.md)
- [`dx_export_decision_matrix.md`](./dx_export_decision_matrix.md)
- [`dx_export_exhaustive_audit.md`](./dx_export_exhaustive_audit.md)
- [`dx_method_decision_matrix.md`](./dx_method_decision_matrix.md)
- [`dx_boundary_cleanup_list.md`](./dx_boundary_cleanup_list.md)
- [`dx_canonical_surface_spec.md`](./dx_canonical_surface_spec.md)
- [`dx_boundary_spec.md`](./dx_boundary_spec.md)
- [`dx_condensation_map.md`](./dx_condensation_map.md)
- [`dx_diagnostics_product_map.md`](./dx_diagnostics_product_map.md)
- [`dx_wording_map.md`](./dx_wording_map.md)
- [`dx_phase_0_5_review.md`](./dx_phase_0_5_review.md)
- [`dx_phase_1_plan.md`](./dx_phase_1_plan.md)
- [`dx_phase_1_boundary_delta.md`](./dx_phase_1_boundary_delta.md)
- [`dx_phase_1_review.md`](./dx_phase_1_review.md)
- [`dx_phase_2_review.md`](./dx_phase_2_review.md)
- [`dx_phase_3_review.md`](./dx_phase_3_review.md)

Why:

- these are execution scaffolding
- they should shape the product docs, not become the product docs

### Architecture and future-history docs

- [`worth_relational_vision.md`](./WORTH_relational_vision.md)
- [`worth_relational_roadmap.md`](./WORTH_relational_roadmap.md)
- [`relational_architecture.md`](./relational_architecture.md)

Why:

- strong maintainer docs
- too broad, too historical, or too architectural for first-line publication

---

## Recommended Published Docs Set

This is the set I would actually target for publication once we start writing
real ship-facing docs.

### Tier 1: first-read product docs

- `README.md`
- `QUICKSTART.md`
- `DAILY_WORKFLOWS.md`
- `API_OVERVIEW.md`
- `OPERATOR_READBACK.md`

### Tier 2: important specialist docs

- `HISTORY_AND_REPLAY.md`
- `MERGE.md`
- `VALIDATION.md`
- `RETENTION_AND_RECOVERY.md`
- `COMMIT_STRATEGIES.md`

### Tier 3: deeper reference docs

- `SCHEMA_AND_CONTRACTS.md`
- `CONFIG_AND_RUNTIME_PROFILES.md`
- `CDC_AND_PUBLICATION.md`
- `COMPILED_ARTIFACTS.md`
- `CORRECTNESS_AND_GUARANTEES.md`

---

## Rewrite Priority

### Priority 0

Must exist before publication:

- `README.md`
- `QUICKSTART.md`
- `DAILY_WORKFLOWS.md`
- `API_OVERVIEW.md`

### Priority 1

Should exist before publication:

- `OPERATOR_READBACK.md`
- `HISTORY_AND_REPLAY.md`
- `VALIDATION.md`
- `RETENTION_AND_RECOVERY.md`

### Priority 2

Can follow immediately after:

- `MERGE.md`
- `COMMIT_STRATEGIES.md`
- `CDC_AND_PUBLICATION.md`
- `CONFIG_AND_RUNTIME_PROFILES.md`
- `COMPILED_ARTIFACTS.md`
- `SCHEMA_AND_CONTRACTS.md`
- `CORRECTNESS_AND_GUARANTEES.md`

---

## Bottom Line

Relational has plenty of writing.

It does not yet have a real publish-facing docs set.

So the Phase 4 job is not "polish the old docs a little."

It is to build a new public docs stack that matches the product shape we have
now actually chosen.
