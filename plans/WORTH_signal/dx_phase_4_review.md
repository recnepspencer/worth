# WORTH Signal DX Phase 4 Review

## Verdict

Phase 4 is complete.

## What Phase 4 Needed To Prove

Phase 4 was about condensation, not just cleanup.

The question was whether `worth-signal` still forced users to remember
multi-step choreography for the main workflows, or whether the public API had
started to offer clearer guided forms.

## What Changed

### Runtime setup stayed simple

The direct runtime constructors remained the normal path:

- `SignalRuntime::build_for::<Ctx>(graph)`
- `SignalRuntime::operational_for::<Ctx>(graph)`
- `SignalRuntime::forensic_for::<Ctx>(graph)`

The builder still exists for abnormal setup, but it no longer defines the main
product story.

### Batch updates became guided

The transaction-owned batch flow is now a real guided session:

- `tx.batch_changes().mark(...).mark_regions(...).apply()?`

This keeps the production path batch-first without removing the lower-level
scalar APIs.

### Execution got a guided request shape

The public surface now has a more memorable execution shape:

- `runtime.target(node).read(...)`
- `runtime.target(node).run(...)`
- `runtime.targets(nodes).read_many(...)`
- `tx.target(node).read(...)`
- `tx.target(node).run(...)`

The older raw execution methods still exist, but they are no longer the main
story in docs and examples.

### Computation definition got a better public shape

The public name is now `Recipe`, not `ComputationSpec`.

The guided runtime declaration path is:

- `runtime.define(Recipe::new(...))`

This is materially better than exposing an internal-feeling `Spec` object as the
main public noun.

### Merge became a guided specialist flow

The specialist merge flow now has a real plan/execute shape:

- `runtime.merge().from(source).into(target).plan()?`
- `planned.execute()?`

This is a better specialist memory shape than forcing users to think in raw
merge packets first.

### Docs and examples now teach the condensed path

The publish-facing docs, crate rustdoc, README, and examples now emphasize:

- `build_for::<Ctx>(...)`
- `tx.batch_changes()`
- `runtime.target(...).read(...)`
- `tx.target(...).run/read(...)`
- `runtime.diagnostics()`
- `runtime.history()`
- `runtime.merge().from(...).into(...).plan()?.execute()?`
- `runtime.define(Recipe::new(...))`

## Verification

Verification on the Phase 4 batch:

- `cargo check -p worth-signal`
  - passed cleanly
- `cargo test -p worth-signal`
  - `477 passed`
  - `0 failed`
  - `23 ignored`
- doc tests
  - `3 passed`

## Remaining Raw Surfaces

Phase 4 does not remove all raw power, and it should not.

The following still intentionally remain:

- explicit evaluation plans
- explicit prepared execution
- explicit executor selection
- raw graph-level authoring and dependency wiring
- lower-level merge, planner, and proof-heavy specialist types

The important point is that these no longer need to define the default memory
shape.

## Exit Criteria Check

Phase 4 exit criteria were:

- guided paths exist for the major high-friction workflows
- raw paths are clearly lower-level and specialist
- condensation decisions are concrete, not hand-wavy

These are satisfied.

## Next Phase

Phase 5: Rationalize Policy Surfaces

The main remaining fragmentation risk is no longer workflow ceremony.

It is policy overlap and control-surface sprawl across runtime policy,
comparators, tier policy, checkpoint policy, diagnostics retention, and related
advanced knobs.
