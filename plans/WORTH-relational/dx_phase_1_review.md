# WORTH Relational DX Phase 1 Review

## Purpose

This is the closeout checkpoint for Phase 1:

- establish the published product boundary

The question here is simple:

- is the public boundary now explicit enough that we can stop debating what the
  product is and move on to noise cleanup and condensation?

---

## Verdict

- Phase 1: Complete

That does not mean the facade is perfectly condensed or finally named.

It means:

- the main boundary hierarchy is now explicit
- the ugliest fake seams were removed in code
- the setup story is no longer split between two equally respectable API doors
- the remaining work belongs to Phase 2 and Phase 3, not to boundary ambiguity

---

## Requirement

Phase 1 is complete only if all of these are true:

- the public boundary hierarchy is explicit
- fake runtime seams are removed or intentionally resolved
- promoted seams have real lane ownership
- `harness` no longer shapes the main public identity
- the primary versus contained versus specialist split is visible

---

## Evidence

### The Boundary Hierarchy Exists In Writing

These docs now define the intended boundary directly:

- [`dx_canonical_surface_spec.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/dx_canonical_surface_spec.md)
- [`dx_boundary_spec.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/dx_boundary_spec.md)
- [`dx_phase_1_plan.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/dx_phase_1_plan.md)

That means the hierarchy is no longer implied.

It is written down as:

- primary runtime story
- guided operational readback
- contained specialist power
- not part of the public story

### The Worst Fake Seams Were Actually Removed

The accepted removals are real in code:

- [`publication_authority()`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/publication/logic/authority.rs)
  is crate-only
- [`storage_authority()`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/storage/logic/authority.rs)
  is crate-only
- [`lineage_access()`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/lineage/logic/access/mod.rs)
  is crate-only
- [`lineage_authority()`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/lineage/logic/authority/mod.rs)
  is crate-only
- [`MergeAccess::runtime()`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/merge/logic/mod.rs)
  is gone

That was the highest-value boundary cleanup in the crate.

### The Promoted Lanes Are No Longer Denied

The runtime-facing exports now openly acknowledge these real lanes:

- [`InvariantAccess`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/validation/logic/mod.rs)
- [`SimulationAccess`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/simulation/logic/access.rs)
- [`SimulationAuthority`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/simulation/logic/authority.rs)
- [`VisibilityReadContext`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/visibility/materialization/read_records/mod.rs)
- [`VisibilityRetentionAuthority`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/visibility/retention/retention_authority.rs)

The naming is not final yet.

But the boundary is no longer pretending those lanes do not exist.

Their lane ownership is now also resolved in the DX docs:

- `visibility_reads()` belongs to the primary runtime read lane
- `invariant_access()` belongs to the contained `validation` lane
- `simulation_access()` and `simulation_authority()` belong to the contained
  `compiled_artifacts` lane
- `retention_authority()` belongs to the contained `retention` lane

That ownership now lives in:

- [`dx_boundary_spec.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/dx_boundary_spec.md)
- [`dx_boundary_cleanup_list.md`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/dx_boundary_cleanup_list.md)

### The Setup Story Is Cleaner

[`RelationalRuntimeApi`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/presentation/api.rs)
now exposes only:

- `builder()`

[`RelationalRuntimeApi::runtime()`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/presentation/api.rs)
was removed.

That matters because it kills the second quasi-official setup path and leaves
the builder flow as the one obvious setup door.

### `harness` No Longer Defines The Main Public Boundary

This work also crossed into the Phase 2 area, but it matters for Phase 1 too:

- [`facade::harness`](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/facade.rs)
  is now `#[cfg(test)]`

That means the non-test public facade no longer treats harness support as a
first-class product lane.

---

## Decision

Complete.

Phase 1 does not require the read lane, validation lane, compiled-artifact
lane, and retention lane to have their final forever names.

It requires that the boundary stop being fuzzy.

That happened.

The remaining work is now about:

- support-noise cleanup
- condensation
- naming refinement

Not about basic boundary legitimacy.

---

## What Remains But Does Not Block Closing Phase 1

These are real, but they belong to later phases:

- final literal naming for promoted lanes
- broader flow condensation for setup, mutation, read, and diagnostics
- docs rewrite around the final product journey
- compatibility and transition planning

Those are not reasons to keep Phase 1 open.

---

## Closing Rule

After this checkpoint:

- treat Phase 1 as closed
- do not reopen it just because later phases improve the lane names or condense
  the workflows further
- only reopen it if the live code drifts back into boundary ambiguity
