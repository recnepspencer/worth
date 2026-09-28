# WORTH Runtime Bridge Docs Publication Audit

## Purpose

This document classifies the current `worth-runtime-bridge` docs by publication
value.

The goal is not to preserve every existing bridge document.

The goal is to decide:

- what should become the public product story
- what should remain as technical reference
- what should stay internal design history

This audit reflects the current bridge direction:

- one bridge product identity
- one guided setup and execution story
- one diagnostics door
- one certification story that proves the bridge as a causal protocol boundary
- one public docs stack shaped around jobs, not milestone archaeology

---

## Classification Legend

- `Rewrite For Publish`
  - should exist in the published docs set
  - current material is too architectural, too roadmap-shaped, or too internal
    to ship as-is
- `Keep As Reference`
  - useful and materially valid
  - not first-read material, but worth preserving for advanced readers and
    maintainers
- `Internal History`
  - valuable to maintainers
  - should not define the public onboarding or product story

---

## Current State

Right now `worth-runtime-bridge` does not have a real publish-facing docs set
under
[`crates/worth-runtime-bridge`](../../crates/worth-runtime-bridge).

Most meaningful writing lives under
[`plans/worth-runtime-bridge`](./),
and it is mostly:

- vision and roadmap control
- milestone engineering specs and closeouts
- certification requirements
- DX hardening docs

That means this audit has two jobs:

1. classify what already exists
2. define the docs set we should actually publish

---

## Publish-Facing Docs To Rewrite

These topics should absolutely exist in the published docs set, but the current
docs are too architecture-first, too historical, or too certification-shaped
to ship as the public bridge story.

### [`worth_runtime_bridge_vision.md`](./WORTH_runtime_bridge_vision.md)

Classification:

- `Rewrite For Publish`

Why:

- this is the right source of truth for what the bridge is
- it is too large and architecture-dense to be the first public landing page
- it should feed a tighter public story rather than ship raw

Target replacement:

- crate `README`
- `QUICKSTART.md`
- `API_OVERVIEW.md`

### [`worth_runtime_bridge_roadmap.md`](./WORTH_runtime_bridge_roadmap.md)

Classification:

- `Rewrite For Publish`

Why:

- useful engineering sequencing doc
- not a user-facing explanation of how to use or trust the bridge
- should inform feature-status notes, not serve as public onboarding

Target replacement:

- capability-status notes inside `API_OVERVIEW.md`
- release notes or changelog material later

### [`test-requirements.md`](./test-requirements.md)

Classification:

- `Rewrite For Publish`

Why:

- this is a trust and certification control doc
- it is too suite-shaped and internal to ship as onboarding
- its guarantees should become a readable public trust story

Target replacement:

- `CERTIFICATION_AND_HARNESS.md`
- `CAUSAL_BUNDLES_AND_GUARANTEES.md`

---

## Docs To Keep As Reference

These documents are useful and materially valid, but they should not carry the
public onboarding load.

### [`milestone-12b.md`](./milestone-12b.md)

Classification:

- `Keep As Reference`

Why:

- strong protocol doc for multi-family writeback identity and mapper
  containment
- important for advanced readers evaluating bridge authority boundaries
- too milestone-shaped for the public first-read stack

### [`milestone-13.md`](./milestone-13.md)

Classification:

- `Keep As Reference`

Why:

- defines the certification summit and the pricing-shock reference workload
- excellent advanced reference for trust posture
- should inform public docs rather than become them directly

### Prior milestone engineering specs and closeouts

Classification:

- `Keep As Reference`

Files:

- [`milestone-1.md`](./milestone-1.md)
- [`milestone-1-closeout.md`](./milestone-1-closeout.md)
- [`milestone-2.md`](./milestone-2.md)
- [`milestone-2-closeout.md`](./milestone-2-closeout.md)
- [`milestone-2-envelope-and-planning-hardening.md`](./milestone-2-envelope-and-planning-hardening.md)
- [`milestone-3.md`](./milestone-3.md)
- [`milestone-3-closeout.md`](./milestone-3-closeout.md)
- [`milestone-4.md`](./milestone-4.md)
- [`milestone-4-closeout.md`](./milestone-4-closeout.md)
- [`milestone-5.md`](./milestone-5.md)
- [`milestone-6.md`](./milestone-6.md)
- [`milestone-7.md`](./milestone-7.md)
- [`milestone-7-closeout.md`](./milestone-7-closeout.md)
- [`milestone-8.md`](./milestone-8.md)
- [`milestone-8-closeout.md`](./milestone-8-closeout.md)
- [`milestone-9.md`](./milestone-9.md)
- [`milestone-9-closeout.md`](./milestone-9-closeout.md)
- [`milestone-10.md`](./milestone-10.md)
- [`milestone-10-closeout.md`](./milestone-10-closeout.md)
- [`milestone-11.md`](./milestone-11.md)
- [`milestone-11-closeout.md`](./milestone-11-closeout.md)
- [`milestone-12.md`](./milestone-12.md)

Why:

- valuable implementation and proof history
- useful for advanced readers tracing how bridge capability matured
- too roadmap-shaped and too detailed for first-line product docs

---

## Internal Design History

These docs should guide maintainers and cleanup work, but they should not be
treated as publish-facing product docs.

### DX planning docs

- [`dx_plan.md`](./dx_plan.md)
- [`dx_canonical_surface_spec.md`](./dx_canonical_surface_spec.md)
- [`dx_boundary_spec.md`](./dx_boundary_spec.md)
- [`dx_boundary_cleanup_spec.md`](./dx_boundary_cleanup_spec.md)

Why:

- these are execution scaffolding for the DX program
- they should shape the public docs, not become the public docs

### Vision, roadmap, and certification control docs

- [`worth_runtime_bridge_vision.md`](./WORTH_runtime_bridge_vision.md)
- [`worth_runtime_bridge_roadmap.md`](./WORTH_runtime_bridge_roadmap.md)
- [`test-requirements.md`](./test-requirements.md)

Why:

- these are strong maintainer docs
- they are too broad, too future-facing, or too certification-controlled for
  first-line publication

---

## Recommended Published Docs Set

This is the docs set I would actually target for publication.

It follows the strong framework pattern:

- landing page first
- quickstart second
- task-oriented guides before deep concepts
- diagnostics and troubleshooting as first-class product docs
- advanced protocol depth available, but not forced on day one

### Tier 1: first-read product docs

- `README.md`
- `QUICKSTART.md`
- `DAILY_WORKFLOWS.md`
- `API_OVERVIEW.md`
- `DIAGNOSTICS.md`

### Tier 2: important specialist docs

- `ROUTING_AND_EVALUATION.md`
- `BRANCHING_AND_SPECULATION.md`
- `WRITEBACK_AND_PROMOTION.md`
- `HISTORY_AND_REPLAY.md`
- `RUNTIME_POLICY.md`

### Tier 3: deeper reference docs

- `CHANGE_STREAMS_AND_SOURCES.md`
- `MAPPING_CONTINUITY_AND_REMAP.md`
- `MERGE_AND_STRUCTURAL_COMPARISON.md`
- `CERTIFICATION_AND_HARNESS.md`
- `CAUSAL_BUNDLES_AND_GUARANTEES.md`
- `HOST_ADAPTERS.md`

---

## Rewrite Priority

### Priority 0

Must exist before publication:

- `README.md`
- `QUICKSTART.md`
- `DAILY_WORKFLOWS.md`
- `API_OVERVIEW.md`
- `DIAGNOSTICS.md`

### Priority 1

Should exist before publication:

- `ROUTING_AND_EVALUATION.md`
- `BRANCHING_AND_SPECULATION.md`
- `WRITEBACK_AND_PROMOTION.md`
- `HISTORY_AND_REPLAY.md`

### Priority 2

Can follow immediately after:

- `RUNTIME_POLICY.md`
- `CHANGE_STREAMS_AND_SOURCES.md`
- `MAPPING_CONTINUITY_AND_REMAP.md`
- `MERGE_AND_STRUCTURAL_COMPARISON.md`
- `CERTIFICATION_AND_HARNESS.md`
- `CAUSAL_BUNDLES_AND_GUARANTEES.md`
- `HOST_ADAPTERS.md`

---

## Bottom Line

The bridge has a lot of strong writing already.

What it does not have yet is a public docs stack that feels like an intentional
product.

So the job is not to polish the existing milestone and DX docs until they look
less internal.

The job is to build a new publish-facing docs set that matches the bridge we
actually intend to ship:

- one causal boundary
- one setup story
- one ordinary route and evaluate story
- one speculative story
- one diagnostics story
- one trust story
