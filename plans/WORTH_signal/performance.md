# worth-signal Performance Milestones

> **Status:** Active implementation roadmap
>
> **Audience:** Runtime engineers working on the next performance architecture wave
>
> **Scope:** This doc is the concrete milestone plan for the remaining large performance architecture work after the `S9` missing-substrate program and the first hardening/tightening passes.
>
> **Related docs:**
> - [signal_performance.md](./signal_performance.md)
> - [signal_performance_architecture.md](./signal_performance_architecture.md)
> - [signal_performance_baseline.md](./signal_performance_baseline.md)
> - [signal_architecture2.md](./signal_architecture2.md)

## Goal

The runtime is now in a good enough semantic state that performance work can be treated as first-class architecture work instead of substrate rescue.

The remaining performance gap is concentrated mostly in:

- staged serial apply
- semantic finalize
- dependency snapshot churn under staged rotating-window workloads
- hot-path locality and hot/cold state separation

This document defines the next four major performance milestones:

1. batch-native staged apply/finalize
2. deeper hot/cold state separation
3. compact snapshot/version-delta model
4. storage locality pass

The point is not incremental polish. The point is to remove whole categories of hot-path cost.

## Entry Criteria

These milestones assume the following are already true:

- supported `S9` substrate work is closed enough that performance work will not reintroduce truth shortcuts
- serial and parallel library sweeps are green
- the runtime has honest performance counters and ignored perf harnesses
- topology perf profiles no longer include debug-only whole-graph audits

If any later change reopens substrate ambiguity, this roadmap pauses until that ambiguity is closed again.

## Performance Laws

These milestones should follow a few hard rules:

1. No performance gain is valid if it weakens semantic truth, rollback truth, merge truth, or reconstructability truth.
2. No hot-path improvement should depend on `easy/` surfaces, test-only shortcuts, or hidden debug-disable semantics.
3. If a batch law exists in reality, represent it structurally. Do not simulate a batch with many per-task mini-protocols.
4. Hot operational state and cold diagnostic richness must stay distinct.
5. Performance claims must be backed by the ignored perf suite plus targeted runtime counters, not intuition.

## Milestone 1: Batch-Native Staged Apply/Finalize

### Why

The remaining dominant bucket is still stage apply, followed by semantic finalize. The current runtime is already much better than before, but it still pays too much per-task orchestration overhead inside a lane that is conceptually batch-shaped.

The primary goal of this milestone is to stop paying a small protocol cost per task when the stage already knows the whole batch.

### Target Form

Staged execution should lower into a compact batch-native form with:

- stage-local apply input arrays
- stage-local effect/result arrays
- stage-local deferred snapshot arrays
- stage-local semantic record arrays
- one deterministic finalize/publication pass per stage

The hot serial lane should not need to build or reduce packet-shaped scaffolding that only exists to preserve abstractions from older execution forms.

### Main Code Surfaces

- [stage.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/apply/stage.rs)
- [prepared_apply.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/evaluation/engine/prepared_apply.rs)
- [apply.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/evaluation/engine/apply.rs)
- [effect.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/runtime/effect.rs)
- [semantic/mod.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/semantic/mod.rs)

### What We Need To Build

1. A compact serial batch execution form.
2. A compact serial batch finalize form.
3. Stage-local arrays or buffers for:
   - prepared apply inputs
   - effect publication results
   - semantic task updates
   - deferred snapshots
4. A smaller per-task hot payload.
5. A direct serial-fast-path publication/finalize loop that avoids generic packet abstraction costs.

### Concrete Work

- narrow `LoweredTaskExecution` and adjacent update structs so the serial path carries only the hot fields it truly needs
- stop building per-task semantic segment wrappers until the last moment, or represent the stage as one naturally ordered batch with compact per-task offsets
- move stage-level temporary collections onto pre-sized buffers that are filled once and reused
- reduce repeated node lookups after apply by carrying forward the minimum post-apply information
- batch trace stamping and semantic record construction where deterministic ordering already exists

### Acceptance Criteria

- `report_stage_apply_nanos` falls materially on the staged rotating-window profile
- `report_semantic_finalize_nanos` also falls materially
- no regression in:
  - branch restore
  - rewiring diagnostics
  - retained vs reconstructed artifact parity
  - serial vs parallel semantic equivalence

### Expected Difficulty

- medium-large
- highest payoff of the four milestones

## Milestone 2: Deeper Hot/Cold State Separation

### Why

The runtime already separates hot operational artifact state from colder retained diagnostics better than it did before, but cold richness still leaks too close to the hot path.

Chip-simulator-grade workloads will punish any design where bookkeeping-rich structures remain attached to the hottest loops.

### Target Form

The hot execution path should update only minimal operational state:

- version/change truth
- compact reuse truth
- compact output identity/continuity truth
- minimal suppression/meaningful-input truth

Cold state should move further out:

- retained diagnostic artifacts
- rich explanation/provenance payloads
- lineage-rich structures
- some reuse certification richness

### Main Code Surfaces

- [trace.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/trace.rs)
- [effect.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/runtime/effect.rs)
- [artifacts.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/diagnostics_access/artifacts.rs)
- [recorder.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/diagnostics/runtime/recorder.rs)
- [resolver.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/explain/resolver.rs)

### What We Need To Build

1. A smaller hot runtime artifact representation.
2. A stricter cold retained artifact path.
3. A cleaner handoff between hot-path truth and cold-path reconstruction.
4. More append-only or deferred diagnostic recording where possible.

### Concrete Work

- split `RuntimeArtifactState` further if needed so hot execution does not carry fields that only matter for later explain/provenance reconstruction
- make lineage stamping consume compact hot facts and emit richer cold artifacts outside the tightest execution path
- keep reconstruction-capable retained facts without forcing expensive rich explain materialization on every hot write
- ensure that semantic finalize can stamp compact truth cheaply and defer richer interpretation to read time

### Acceptance Criteria

- no loss of diagnostic truth under:
  - rewiring
  - branch restore
  - cross-identity reuse
  - partial splice
- lower hot-path artifact-retention cost on staged churn profiles
- retained vs reconstructed parity tests remain green

### Expected Difficulty

- medium
- very important for sustained throughput and cache pressure

## Milestone 3: Compact Snapshot / Version-Delta Model

### Why

The runtime has improved dependency snapshot handling, but stable-shape churn is still a major recurring workload. The fast path should be designed around compact stable-shape version updates as the common case, not around generic full replacement with an optimization branch.

### Target Form

Dependency snapshots should behave more like:

- a stable structural shape handle
- a compact version vector or delta payload
- a batch-friendly commit model for shared-shape updates

The common case should avoid rebuilding whole snapshot objects.

### Main Code Surfaces

- [apply.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/evaluation/engine/apply.rs)
- [entries.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/storage/entries.rs)
- dependency data types under [data](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data)
- topology/runtime dependency accessors under the graph runtime/topology storage layers

### What We Need To Build

1. A more explicit stable-shape snapshot form.
2. Cheaper version-only delta construction.
3. Cheaper batch commit for shared-shape updates.
4. Better structural sharing for rotating-window churn.

### Concrete Work

- reduce reliance on generic `DependencySnapshotUpdate::Replace(...)` in paths where stable shape is already known
- introduce or refine shape-handle plus version-vector style updates if current types are still too generic
- make snapshot commit paths optimize around the version-only case first
- keep delta accounting honest and replay-safe while removing unnecessary full snapshot construction

### Acceptance Criteria

- lower `dependency_input_build_nanos` on a benchmark that is actually stable-shape-dominant
- lower `dependency_reconcile_nanos` where the staged path is paying for snapshot-adjacent work without structural rewiring
- lower `snapshot_batch_commit_nanos` on stable-shape churn workloads
- no regression in snapshot restore, dependency restore batches, or subscriber integrity

> [!IMPORTANT]
> `perf_dependency_reconciliation_rotating_window_staged_serial` is **not** the primary Milestone 3 acceptance lane.
>
> We instrumented the post-M3 runtime and found that this workload is overwhelmingly structural-replacement heavy rather than stable-shape heavy:
>
> - `dependency_input_replacement_count`: `12288`
> - `dependency_input_stable_shape_count`: `64`
>
> That means the workload is a valuable staged serial stress test, but it is mostly measuring the cost of structural dependency-shape churn rather than the compact stable-shape version-delta path Milestone 3 was designed to improve.
>
> Milestone 3 must therefore be judged primarily against a benchmark that keeps dependency membership/order fixed and changes only upstream versions. The current dedicated lane for that is:
>
> - `perf_dependency_reconciliation_stable_shape_staged_serial`
>
> Keep the rotating-window staged benchmark in the suite, but interpret it as a structural-replacement pressure lane that informs Milestone 2 / Milestone 4 follow-up work rather than as the main Milestone 3 scorecard.

### Expected Difficulty

- medium-large
- correctness-sensitive, but very high value for churn-heavy workloads

## Milestone 4: Storage Locality Pass

### Why

Once the obvious framework overhead is removed, locality becomes a first-order concern. Chip-simulator-class pressure will punish pointer-heavy, mixed-hot-cold layouts even when the logic is otherwise correct.

### Target Form

The hottest state should become more locality-friendly:

- tighter hot node fields
- fewer cold payload traversals in hot loops
- more contiguous hot arrays or more structure-of-arrays-style access where it helps
- reduced pointer chasing in apply, finalize, invalidation, and snapshot lanes

### Main Code Surfaces

- node storage and hot node fields
- graph runtime hot traversal state
- dependency snapshot handle access
- hot artifact/access paths
- selected storage code under:
  - [entries.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/storage/entries.rs)
  - graph/node runtime code
  - traversal scratch and hot execution code

### What We Need To Build

1. A clear hot-field inventory.
2. A colder payload inventory.
3. A staged migration plan for layout changes.
4. Locality-aware benchmarks and counter interpretation.

### Concrete Work

- identify exactly which per-node fields are read/written in:
  - apply
  - semantic finalize
  - invalidation suppression
  - snapshot commit
- separate those hot fields from colder payloads where practical
- consider compact packed headers for node hot state
- avoid pulling full rich node/trace structures into loops that only need a few hot fields
- ensure any layout changes stay compatible with branch restore, rollback, and diagnostics truth

### Acceptance Criteria

- measurable gains on sustained churn and wide staged workloads
- no hidden semantic coupling introduced by storage specialization
- serial and parallel parity stays green
- no panic/error regressions from layout churn

### Expected Difficulty

- high
- most invasive of the four milestones

## Suggested Order

Recommended order by payoff vs risk:

1. batch-native staged apply/finalize
2. compact snapshot/version-delta model
3. deeper hot/cold state separation
4. storage locality pass

This order is not arbitrary:

- Milestone 1 attacks the current dominant cost directly.
- Milestone 3 attacks the most important recurring churn structure underneath it.
- Milestone 2 reduces hot-path semantic/bookkeeping drag as the batch model becomes stronger.
- Milestone 4 is the heaviest systems pass and should be informed by the earlier reshaping.

## Verification Plan

Every milestone should be verified with:

1. full serial library sweep
2. full parallel library sweep
3. ignored perf suite with `--test-threads=1`
4. before/after updates in [signal_performance_baseline.md](./signal_performance_baseline.md)
5. targeted regression lanes for the areas the milestone touches most

Required perf profiles to watch:

- `perf_dependency_reconciliation_rotating_window_staged_serial`
- `perf_dependency_reconciliation_stable_shape_staged_serial`
- `perf_dependency_reconciliation_rotating_window_serial`
- `perf_topology_rewiring_rotating_window_serial`
- `perf_fintech_mixed_fanout_profile_matrix`

If a milestone improves one hotspot while making another representative workload materially worse, it is not done.

## Current Read

As of the current roadmap checkpoint:

- the runtime looks semantically strong enough to treat performance as the main frontier
- the remaining gap is concentrated rather than diffuse
- the staged rotating-window serial lane is still an important pressure point, but it is now known to be mostly a structural-replacement workload rather than a stable-shape workload
- the new stable-shape staged reconciliation lane is the correct primary acceptance benchmark for Milestone 3
- the next best gains are likely to come from architectural restructuring, not local micro-tuning

That is a healthy place to be.
