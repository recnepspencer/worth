# WORTH Relational DX Phase 3 Review

## Purpose

This is the closeout checkpoint for Phase 3:

- condense the core truth-runtime flows

The question here is not whether every module is now tiny.

The question is:

- can the crate now be taught as a small number of workflows instead of a tour
  of subsystem names?

---

## Verdict

- Phase 3: Complete

The code has not been fully renamed around those workflows yet.

But the product workflow map is now explicit enough that future code and docs
can follow one stable story.

That is the actual Phase 3 bar.

---

## Requirement

Phase 3 is complete only if all of these are true:

- runtime setup has one obvious guided story
- write truth has one obvious guided story
- read truth has one obvious guided story
- operator readback has one obvious story
- history, replay, and merge escalation order is explicit
- promoted lanes have a place inside the workflow map instead of floating as
  isolated helper seams

---

## Evidence

### Runtime Setup Is Now Condensed Around One Spine

[`dx_condensation_map.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/dx_condensation_map.md)
locks the setup story to:

- `RelationalRuntimeApi::builder()`
- `profile(...)`
- `schema_registry(...)`
- `build()`

That means the crate no longer needs to be introduced as a flat builder knob
list.

### Write Truth Now Has One Canonical Story

The same condensation map locks the write path to:

- begin transaction
- push batch
- commit

And for larger writes:

- begin transaction
- plan bulk mutation batch
- commit

That preserves the real internal phases without making them the first thing the
public story teaches.

### Read Truth And Query Now Have An Explicit Escalation Order

Phase 3 explicitly records:

- current truth first
- query for bigger read jobs
- inspection for operator questions
- history for past truth

The current implementation seam is still `visibility_reads()`, but the product
direction is now stable enough to teach as `read_truth`.

### Inspection And Publication Now Form The Operator Readback Story

[`dx_diagnostics_product_map.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/dx_diagnostics_product_map.md)
does not fake a nonexistent unified diagnostics object.

Instead it locks the honest operator story to:

- `inspection_access()` for "what happened?"
- `publication_access()` for "what published?" and publication trouble
- `invariant_access()` as the contained validation lane

That is a real product shape, not just a list of nearby modules.

### Escalation Order Is Now Explicit

The condensation map and diagnostics product map now agree on the specialist
ladder:

- current truth
- history
- replay
- merge / validation / compiled artifacts / retention / durability

That matters because the crate can now be taught in a stable order instead of
as a flat pile of equally plausible helper doors.

---

## Decision

Complete.

Phase 3 does not require every code path to already have final job-shaped API
names.

It requires that the workflow condensation decisions are made, written down,
and coherent enough to guide Phase 4 wording and Phase 5 compatibility work.

That happened.

---

## What Remains But Belongs To Later Phases

These are real, but they are not Phase 3 blockers:

- productizing the wording into final public names
- docs-publication cleanup
- compatibility and migration strategy
- code renames or guided wrapper additions that may come later

Those belong to Phase 4 and Phase 5.

---

## Closing Rule

After this checkpoint:

- treat Phase 3 as closed
- do not reopen it just because some current implementation seams still have
  older names
- only reopen it if the workflow order itself becomes unclear again
