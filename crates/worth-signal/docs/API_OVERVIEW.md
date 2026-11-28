# API Overview

This is the map of the public surface.

There are two normal entry paths:

- `worth_signal::easy` for the shortest path
- `worth_signal::facade::*` for the broader runtime surface

They are not two different systems.
`easy` is a simpler way into the same runtime story.

If you only remember five areas, remember these:

- easy
- graph
- runtime
- transaction
- diagnostics

There is now a sixth area worth remembering when you are wiring app-facing
reactions:

- observation

The intended import path is:

```rust
use worth_signal::facade::*;
```

If you want the shortest path in, start with `worth_signal::easy::*`.

The short-path names to remember are:

- `SignalApp`
- `InputSignal<T>`
- `ComputedSignal<T>`
- `SignalContext`

And the short-path observation names are:

- `watch(...)`
- `effect(...)`
- `unobserve(...)`

## Read This With The Docs Tree

If you are learning the library in order:

- `GETTING_STARTED.md`
- `core-concepts/`
- `guides/`
- `reference/`

## Core

This is the graph surface you use all the time.

- `SignalGraph`
- `NodeBuilder`
- `DependencyEdge`
- `mark_changed(...)`
- `mark_changed_with_regions(...)`
- `mark_dirty_batch(...)`

If you are in `easy`, this lower layer is still what the system runs on top of.
You just do not have to start here.

## Runtime

This is the main runtime surface.

- `SignalRuntime`
- `SignalRuntime::build_for::<Ctx>(...)`
- `SignalRuntime::operational_for::<Ctx>(...)`
- `SignalRuntime::forensic_for::<Ctx>(...)`
- `RuntimePolicy`
- `SignalTransaction`
- `BatchChange`
- `BatchChangeResult`
- `Recipe`
- `History`
- `RuntimeMerge`

If you want one stable computation shape instead of rebuilding the same thing by
hand, use:

- `runtime.define(Recipe::new(...))`
- `.keyed(...)`
- `.run(...)`
- `.read(...)`

If you started in `easy`, this is the surface you grow into.

## Observation

This is the runtime-backed observation surface.

- `runtime.observe_nodes(...)`
- `runtime.unobserve(...)`
- `ObservationPolicy`
- `ObservationTrigger`
- `ObservationHandle`
- `ObservationNotice`
- `ObservationReadContext`
- `ObservedNodeSet`

The important semantic point is that observation is commit-bounded.

- one committed transaction yields at most one boundary per matching observer
- rollback suppresses normal delivery
- matching can be based on touched, recomputed, or meaningful-change policy

On the short path, `SignalApp::watch(...)` and `SignalApp::effect(...)` build on
the same substrate instead of inventing a second local callback model.

## Diagnostics

This is where you go when the runtime does something you did not expect.

- `runtime.diagnostics()`
- `diagnostics_for_graph(...)`
- `diagnostics_for_runtime(...)`

Start with:

- `why(node)`
- `explain(node)`
- `health_now()`
- `health_view().current_now()`
- `latest_observation_summary()`
- `latest_flow_diagnostics()`
- `inspect().graph()`
- `compare().reports(...)`

Observation boundaries now live here too.
If an observer fired, or a rollback-suppressed boundary was retained, diagnostics
is where you inspect the latest committed truth.

## Advanced

Use this when you are taking deliberate control, not when you are still trying
to get the system working.

- `EvaluationPlan`
- `RunMode`
- `StageExecutor`
- `ReadView`
- `PlannedRun`
- `ComparatorPolicy`

If you need to tune advanced behavior, look for the grouped edit points first:

- `builder.adjust_runtime_policy(...)`
- `builder.adjust_checkpoints(...)`
- `builder.adjust_fallback_comparator(...)`
- `runtime.adjust_runtime_policy(...)`
- `runtime.adjust_checkpoint_policy(...)`
- `runtime.adjust_tier_policy(...)`
- `runtime.adjust_fallback_comparator(...)`

## Integration

This is for bridge authors and deeper integration work.

- event subscribers
- effect mapping
- checkpoint evaluators

It matters. It just is not where most users should start.

## History

History is exposed through the runtime:

- `runtime.history()`
- `runtime.target(node)`
- `tx.target(node)`

Do not start by memorizing lineage internals.

## Policy shape

The short version:

- choose a runtime preset first
- refine `RuntimePolicy` only when you need more control
- treat `reset_runtime_policy_to_tier(...)` as the stock preset reset, not the
  main advanced owner
