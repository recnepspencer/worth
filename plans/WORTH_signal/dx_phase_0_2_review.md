# WORTH Signal DX Phase 0-2 Review

## Purpose

This is the production-grade review checkpoint for:

- Phase 0
- Phase 0.5
- Phase 1
- Phase 2

The goal is to decide whether these phases are complete enough to be closed
officially so later work can proceed linearly.

---

## Verdict

- Phase 0: Complete
- Phase 0.5: Complete
- Phase 1: Complete
- Phase 2: Complete

These phases are not “perfect forever.”

They are complete in the only way that matters for linear execution:

- the standards exist
- the canonical product shape exists
- the published boundary is now clear enough to guide work
- internal/certification leakage no longer defines the visible product identity

Any remaining issues belong to later phases, especially 3 through 5.

---

## Phase 0 Review

### Requirement

- the DX standard is frozen
- public API work references the decision matrix instead of convenience

### Evidence

- [`dx_export_decision_matrix.md`](./dx_export_decision_matrix.md)
  exists and has been used as the classification basis for the cleanup work
- later design artifacts exist and align to it:
  - [`dx_boundary_spec.md`](./dx_boundary_spec.md)
  - [`dx_canonical_surface_spec.md`](./dx_canonical_surface_spec.md)
  - [`dx_diagnostics_product_map.md`](./dx_diagnostics_product_map.md)
  - [`dx_compatibility_transition_plan.md`](./dx_compatibility_transition_plan.md)
- the actual code work has followed the classification logic:
  - guided surface promoted
  - specialist surface contained
  - test-only compatibility used instead of reopening production leakage

### Decision

Complete.

---

## Phase 0.5 Review

### Requirement

- we have a concrete canonical product shape
- the team knows what users are supposed to memorize

### Evidence

- [`dx_canonical_surface_spec.md`](./dx_canonical_surface_spec.md)
  now matches the current implementation direction
- the canonical import path is explicit:
  - `use worth_signal::facade::*;`
- the canonical setup path is explicit and implemented:
  - `SignalRuntime::build_for::<Ctx>(graph)`
- the canonical batch mutation path is explicit and implemented:
  - `tx.batch_changes().mark(...).apply()?`
- the canonical diagnostics entry is explicit and implemented:
  - `runtime.diagnostics()`
  - `diagnostics.compare().*`
  - `diagnostics.health_now()`
- the role of `easy` is clear enough for later phases:
  - it remains a top-level guided path, not the full product identity

### Decision

Complete.

---

## Phase 1 Review

### Requirement

- the published boundary is clear
- the canonical import and starting point are not ambiguous
- `diagnostics` and `easy` do not undermine `facade`

### Evidence

- [`lib.rs`](../../crates/worth-signal/src/lib.rs)
  states the main import path directly
- [`facade.rs`](../../crates/worth-signal/src/facade.rs)
  documents the intended public shape directly in code
- [`docs/README.md`](../../crates/worth-signal/docs/README.md)
  starts with the curated product journey and names `facade` as the main import
  path
- `worth_signal::easy` remains public, but the docs do not position it as the
  main production/runtime identity
- `worth_signal::diagnostics` remains public, but it is now intentionally
  organized and subordinate to the guided diagnostics story
- the non-test `facade::*` export set has been thinned so the daily-use path is
  more curated and specialist clutter is reduced

### Decision

Complete.

---

## Phase 2 Review

### Requirement

- internal/certification/support surface no longer defines the visible product
  identity

### Evidence

- `facade::harness` is no longer part of the non-test public boundary
- the harness/deployment/metrics compatibility exports in
  [`facade.rs`](../../crates/worth-signal/src/facade.rs)
  are gated to `#[cfg(test)]`
- [`lib.rs`](../../crates/worth-signal/src/lib.rs)
  keeps `presentation` private in non-test builds and only exposes it under
  tests
- the docs place certification and harness material in lower-level reference,
  not in the first-path product story:
  - `CERTIFICATION_AND_HARNESS.md`
  - [`docs/README.md`](../../crates/worth-signal/docs/README.md)
- the user-facing README and crate docs do not pitch harness/certification as
  the front door

### Decision

Complete.

---

## Non-Blocking Remaining Work

These are real, but they belong to later phases and do not block closing 0-2:

- facade thinning is not finished
- policy ownership and overlapping knobs are not fully rationalized
- some specialist namespaces remain broader than ideal
- examples still need expansion beyond the first tranche
- naming and ergonomics sweeps remain

Those are Phase 3+ concerns, not reasons to keep 0-2 open forever.

---

## Closing Rule

After this checkpoint:

- do not reopen Phase 0 through 2 unless a later phase exposes a real
  regression
- continue linearly from Phase 3 onward
