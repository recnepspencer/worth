# WORTH Relational DX Phase 2 Review

## Purpose

This is the closeout checkpoint for Phase 2:

- remove internal leakage and specialist noise

The question is not whether every broad namespace has been condensed yet.

The question is:

- does support-only or certification-shaped surface still define the visible
  product identity?

---

## Verdict

- Phase 2: Complete

The visible product boundary is still broad.

But it is no longer being defined by obvious support scaffolding.

That is the real Phase 2 bar.

---

## Requirement

Phase 2 is complete only if all of these are true:

- ordinary users can look at the public story without being forced through
  support scaffolding
- support-only or narrow-author seams are no longer shaping the visible
  product identity
- the remaining broad surfaces are broad because they are real architecture,
  not because support debris was left lying around

---

## Evidence

### `harness` Left The Non-Test Public Boundary

The biggest remaining leak is now gone from the non-test facade:

- [`facade::harness`](../../crates/worth-relational/src/facade.rs)
  is now behind `#[cfg(test)]`
- [`presentation::harness`](../../crates/worth-relational/src/presentation/mod.rs)
  is also test-only

That means fixture loading, harness planning, and harness adapters no longer
show up as part of the normal product contract.

### Harness Audit Surface Left The Non-Test Runtime Story

The other clear certification-shaped leak is also gone from the non-test public
surface:

- `HarnessAuditMode`
  is only re-exported for tests
- `InvariantAccess::harness_audit()`
  is only available in test builds

That matters because "harness audit" is exactly the kind of support and
certification language that should not sit in the normal runtime story.

### Follow-Up Audit Did Not Find Another Hidden Leak Cluster

[`dx_phase_1_boundary_delta.md`](./dx_phase_1_boundary_delta.md)
confirmed that after the first cleanup pass:

- there was not another equally bad batch of public helper backdoors waiting
  behind the first one

So the remaining broadness is now mostly about real architecture:

- schema
- transactions
- query
- history
- publication
- merge

Those still need condensation.

But they are not Phase 2 leaks.

### The Setup Story Is Less Noisy

[`RelationalRuntimeApi::runtime()`](../../crates/worth-relational/src/presentation/api.rs)
is gone.

That is partly a Phase 1 hierarchy fix, but it also reduces public noise by
removing a duplicate convenience door that was competing with the actual setup
story.

---

## Decision

Complete.

Phase 2 does not require every real subsystem to be small.

It requires that support-only and certification-shaped leakage stop acting like
the product.

That happened.

---

## What Remains But Belongs To Later Phases

These are real, but they are not Phase 2 blockers:

- condensing the broad primary lanes
- turning the promoted runtime seams into cleaner named lanes
- shrinking the noun clouds around schema, transactions, diagnostics, and
  publication
- productizing docs and examples

Those are Phase 3 and Phase 4 jobs.

---

## Closing Rule

After this checkpoint:

- treat Phase 2 as closed
- do not reopen it just because real architectural lanes are still broad
- only reopen it if support or certification scaffolding starts leaking back
  into the non-test public story
