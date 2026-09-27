# WORTH Signal DX Phase 5 Policy Inventory

## Purpose

This document is the source-of-truth working artifact for the first four Phase
5 steps:

1. build the policy inventory
2. map real decision ownership
3. choose canonical owners
4. choose the target public shape

This is the document the code cleanup should follow.

Phase 8 supersedes the transitional compatibility notes below: the tier-named
parallel threshold fields and `set_diagnostics_profile(...)` aliases have been
removed. Current code uses execution-objective threshold fields and
`reset_runtime_policy_to_tier(...)` or `set_runtime_policy(...)`.

It is intentionally practical.

The point is not to restate every type in academic language.

The point is to answer:

- what knobs exist today
- what each knob really controls
- where ownership is split
- which surface should win
- what shape we should implement next

---

## Working Position

Phase 4 already gave `worth-signal` better workflow shapes.

Phase 5 should make the policy story match those shapes.

The normal mental model should be:

- pick a runtime posture
- build the runtime
- work normally
- only open one advanced section when you actually need finer control

Not:

- remember which of several policy knobs is the "real" one
- decide whether builder, runtime, graph, diagnostics, or merge owns the same
  behavior
- learn internal decomposition just to ask for sane defaults

---

## Policy Families Covered

This audit covers the policy families named in the Phase 5 plan:

- runtime posture
- diagnostics richness and retention
- execution and parallelism
- comparator and semantic equality
- conditions and evaluation gating
- tiers and checkpoints
- history, restore, and merge policy

---

## Inventory

## 1. Runtime Posture

### `SignalRuntime::build_for::<Ctx>(graph)`

- Owner module:
  [runtime_state/mod.rs](C:/forge_workspace/worktree_3/crates/worth-signal/src/logic/transaction/runtime/state/runtime_state/mod.rs)
- Current public path:
  `worth_signal::facade::SignalRuntime::build_for`
- What it really controls:
  the recommended default runtime posture for the typed context path
- Current default:
  `development`
- Likely audience:
  daily user

### `SignalRuntime::{development_for, operational_for, web_development_for, fintech_for, forensic_for}`

- Owner module:
  [runtime_state/construction/context_presets.rs](C:/forge_workspace/worktree_3/crates/worth-signal/src/logic/transaction/runtime/state/runtime_state/construction/context_presets.rs)
- Current public path:
  `worth_signal::facade::SignalRuntime::*_for`
- What it really controls:
  named runtime posture presets
- Current default:
  explicit preset chosen by caller
- Likely audience:
  daily user, advanced runtime user

### `SignalRuntimeBuilder::runtime_policy(...)`

- Owner module:
  [builder.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/builder.rs)
- Current public path:
  `worth_signal::facade::runtime::SignalRuntimeBuilder::runtime_policy`
- What it really controls:
  runtime posture before build
- Current default:
  `SignalRuntimePolicy::default()` which is `development`
- Likely audience:
  advanced runtime user

### `SignalRuntime::set_runtime_policy(...)`

- Owner module:
  [observation.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/observation.rs)
- Current public path:
  `worth_signal::facade::SignalRuntime::set_runtime_policy`
- What it really controls:
  runtime posture after build
- Current default:
  no-op unless called
- Likely audience:
  advanced runtime user

### `SignalRuntime::try_set_runtime_policy(...)`

- Owner module:
  [observation.rs](C:/forge_workspace/worktree_3/crates/worth-signal/src/logic/transaction/runtime/state/observation.rs)
- Current public path:
  `worth_signal::facade::SignalRuntime::try_set_runtime_policy`
- What it really controls:
  result-bearing runtime-policy admission and installation after build
- Current default:
  no-op unless called
- Likely audience:
  advanced runtime user and integrations accepting caller configuration

### `SignalGraph::set_runtime_policy(...)`

- Owner module:
  [runtime.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/diagnostics_access/runtime.rs)
- Current public path:
  crate-internal support; external tooling must use the canonical
  `SignalRuntime` builder or runtime mutation front door
- What it really controls:
  graph-local diagnostics and retention posture
- Current default:
  `SignalRuntimePolicy::default()` which is `development`
- Likely audience:
  internal/support tooling only

### `SignalRuntimePolicy`

- Owner module:
  [definition.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/runtime_policy/definition.rs)
- Current public path:
  `worth_signal::facade::SignalRuntimePolicy`
- What it really controls:
  the main posture bundle for diagnostics richness, retention, replay richness,
  restore lineage mode, and parallel admission thresholds
- Current default:
  `development`
- Likely audience:
  daily user, advanced runtime user

---

## 2. Diagnostics Richness And Retention

### `DiagnosticsTier`

- Owner module:
  [profile.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/diagnostics/policy/profile.rs)
- Current public path:
  `worth_signal::facade::DiagnosticsTier`
- What it really controls:
  the high-level diagnostics richness class
- Current default:
  `Development`
- Likely audience:
  daily user, advanced runtime user

### Removed: `SignalGraph::set_diagnostics_profile(...)`

- Owner module:
  [runtime.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/diagnostics_access/runtime.rs)
- Current public path:
  removed in M10 Phase 8
- What it really controls:
  resets full runtime policy to `SignalRuntimePolicy::for_tier(profile)`
- Current default:
  no-op unless called
- Likely audience:
  advanced runtime user, internal/support

### Removed: `SignalRuntime::set_diagnostics_profile(...)`

- Owner module:
  [observation.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/observation.rs)
- Current public path:
  removed in M10 Phase 8
- What it really controls:
  resets full runtime policy to the chosen tier preset
- Current default:
  no-op unless called
- Likely audience:
  advanced runtime user

### `SignalRuntimePolicy::{operational, development, forensic, web_development, fintech, kernel, game_engine}`

- Owner module:
  [presets.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/runtime_policy/presets.rs)
- Current public path:
  `worth_signal::facade::SignalRuntimePolicy::*`
- What it really controls:
  named posture bundles
- Current default:
  explicit preset chosen by caller
- Likely audience:
  daily user, advanced runtime user

### `SignalRuntimePolicy::{with_explanation_retention, with_provenance_retention, with_replay_detail, with_semantic_retention, with_history_limit, with_detail_limit, with_history_details, with_snapshot_restore_lineage_mode}`

- Owner module:
  [presets.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/runtime_policy/presets.rs)
- Current public path:
  `worth_signal::facade::SignalRuntimePolicy::*`
- What it really controls:
  bounded overrides inside the main runtime posture bundle
- Current default:
  inherited from the chosen preset
- Likely audience:
  advanced runtime user

### `RetentionBudget` and `ReconstructionBudget`

- Owner module:
  [definition.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/diagnostics/policy/definition.rs)
- Current public path:
  `worth_signal::facade::RetentionBudget`,
  `worth_signal::facade::ReconstructionBudget`
- What they really control:
  low-level retention and cold-materialization details inside runtime posture
- Current default:
  chosen indirectly through `SignalRuntimePolicy`
- Likely audience:
  advanced runtime user, integration author

---

## 3. Execution And Parallelism

### `ParallelAdmissionPolicy`

- Owner module:
  [parallel.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/runtime_policy/parallel.rs)
- Current public path:
  `worth_signal::facade::ParallelAdmissionPolicy`
- What it really controls:
  when the runtime will admit staged or full parallel work for each execution
  objective
- Current default:
  embedded in `SignalRuntimePolicy`
- Likely audience:
  advanced runtime user

### `SignalRuntimePolicy::with_parallel_admission(...)`

- Owner module:
  [presets.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/runtime_policy/presets.rs)
- Current public path:
  `worth_signal::facade::SignalRuntimePolicy::with_parallel_admission`
- What it really controls:
  runtime-level parallel admission thresholds
- Current default:
  preset-specific defaults
- Likely audience:
  advanced runtime user

### `StageExecutor`

- Owner module:
  [facade.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/facade.rs)
- Current public path:
  `worth_signal::facade::advanced::StageExecutor`
- What it really controls:
  explicit executor choice for prepared plan and request-based execution
- Current default:
  request APIs can run without caller supplying an executor
- Likely audience:
  advanced runtime user

### `ParallelExecutionPolicy`

- Owner module:
  [facade.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/facade.rs)
- Current public path:
  `worth_signal::facade::advanced::ParallelExecutionPolicy`
- What it really controls:
  detailed executor topology for parallel planning/apply
- Current default:
  executor-specific defaults
- Likely audience:
  advanced runtime user

---

## 4. Comparator And Semantic Equality

### `VersionComparatorPolicy`

- Owner module:
  [comparator.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/comparator.rs)
- Current public path:
  `worth_signal::facade::advanced::VersionComparatorPolicy`
- What it really controls:
  how version changes are treated as meaningful or ignorable
- Current default:
  `Exact`
- Likely audience:
  advanced runtime user, integration author

### `SignalRuntimeBuilder::fallback_comparator(...)`

- Owner module:
  [builder.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/builder.rs)
- Current public path:
  `worth_signal::facade::runtime::SignalRuntimeBuilder::fallback_comparator`
- What it really controls:
  runtime-wide fallback comparator before build
- Current default:
  `Exact`
- Likely audience:
  advanced runtime user

### `SignalRuntime::set_fallback_comparator(...)`

- Owner module:
  [observation.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/observation.rs)
- Current public path:
  `worth_signal::facade::SignalRuntime::set_fallback_comparator`
- What it really controls:
  runtime-wide fallback comparator after build
- Current default:
  no-op unless called
- Likely audience:
  advanced runtime user

### `TierPolicy::with_default_comparator(...)`

- Owner module:
  [tier.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/tier.rs)
- Current public path:
  `worth_signal::facade::runtime::TierPolicy::with_default_comparator`
- What it really controls:
  tier-specific comparator default
- Current default:
  `Exact`
- Likely audience:
  advanced runtime user

### `Recipe::with_comparator(...)`

- Owner module:
  [computation.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/computation.rs)
- Current public path:
  `worth_signal::facade::runtime::Recipe::with_comparator`
- What it really controls:
  per-defined-computation comparator override
- Current default:
  falls back to tier or runtime fallback comparator
- Likely audience:
  advanced runtime user

### `NodeContract::with_comparator_override(...)`

- Owner module:
  [contract.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/node/contract.rs)
- Current public path:
  specialist node contract API
- What it really controls:
  per-node comparator override in the low-level path
- Current default:
  no override
- Likely audience:
  integration author

---

## 5. Conditions And Evaluation Gating

### `EvaluationCondition`

- Owner module:
  [facade.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/facade.rs)
- Current public path:
  `worth_signal::facade::EvaluationCondition`
- What it really controls:
  node-level admission rules
- Current default:
  node declaration default
- Likely audience:
  daily user, advanced runtime user

### `ConditionResolver` and related advanced condition types

- Owner module:
  [facade.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/facade.rs)
- Current public path:
  `worth_signal::facade::advanced::*`
- What they really control:
  custom condition evaluation plumbing
- Current default:
  runtime default resolver path
- Likely audience:
  integration author, advanced runtime user

### `DependencyMode`, `DirtyPropagation`, `EvaluationTrigger`

- Owner module:
  [tier.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/tier.rs)
- Current public path:
  `worth_signal::facade::runtime::*`
- What they really control:
  tier-level evaluation and propagation behavior
- Current default:
  chosen by each `TierPolicy`
- Likely audience:
  advanced runtime user

---

## 6. Tiers And Checkpoints

### `TierPolicy`

- Owner module:
  [tier.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/tier.rs)
- Current public path:
  `worth_signal::facade::runtime::TierPolicy`
- What it really controls:
  tier-level dependency, propagation, trigger, and default comparator behavior
- Current default:
  explicit per-tier policy chosen by caller
- Likely audience:
  advanced runtime user

### `SignalRuntimeBuilder::tier_policy(...)`

- Owner module:
  [builder.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/builder.rs)
- Current public path:
  `worth_signal::facade::runtime::SignalRuntimeBuilder::tier_policy`
- What it really controls:
  seed tier policy at build time
- Current default:
  no seeded tier policies
- Likely audience:
  advanced runtime user

### `SignalRuntime::set_tier_policy(...)`

- Owner module:
  [observation.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/observation.rs)
- Current public path:
  `worth_signal::facade::SignalRuntime::set_tier_policy`
- What it really controls:
  add or replace one tier policy after build
- Current default:
  no-op unless called
- Likely audience:
  advanced runtime user

### `SignalRuntime::set_node_tier(...)`

- Owner module:
  [observation.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/observation.rs)
- Current public path:
  `worth_signal::facade::SignalRuntime::set_node_tier`
- What it really controls:
  per-node tier assignment
- Current default:
  no tier assignment
- Likely audience:
  advanced runtime user

### `CheckpointBarrier`

- Owner module:
  [checkpoint.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/checkpoint.rs)
- Current public path:
  `worth_signal::facade::runtime::CheckpointBarrier`
- What it really controls:
  when deferred checkpoint work is allowed to flush
- Current default:
  `PerOperation` through builder defaults
- Likely audience:
  advanced runtime user

### `CheckpointPolicy`

- Owner module:
  [checkpoint_policy.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/checkpoint_policy.rs)
- Current public path:
  `worth_signal::facade::runtime::CheckpointPolicy`
- What it really controls:
  per-domain checkpoint barrier overrides
- Current default:
  `PerOperation` for all domains when built via default builder flow
- Likely audience:
  advanced runtime user

### `SignalRuntimeBuilder::{checkpoint_barrier, checkpoint_policy}`

- Owner module:
  [builder.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/builder.rs)
- Current public path:
  `worth_signal::facade::runtime::SignalRuntimeBuilder::*`
- What they really control:
  shorthand versus full checkpoint configuration before build
- Current default:
  builder defaults to `CheckpointBarrier::PerOperation`
- Likely audience:
  advanced runtime user

---

## 7. History, Restore, And Merge Policy

### `RuntimeHistory`

- Owner module:
  [guided.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/guided.rs)
- Current public path:
  `worth_signal::facade::runtime::RuntimeHistory`
- What it really controls:
  guided access to branch, snapshot, replay, and lineage workflows
- Current default:
  runtime-owned guided entry point
- Likely audience:
  advanced runtime user

### `SnapshotRestoreIntent` and related snapshot restore types

- Owner modules:
  [state/mod.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/state/mod.rs),
  [facade.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/facade.rs)
- Current public path:
  `worth_signal::facade::history::*`
- What they really control:
  restore behavior, artifact retention policy on restore, dependency-state
  restore behavior, and restore planning
- Current default:
  runtime/story defaults unless caller selects explicit restore intent
- Likely audience:
  advanced runtime user, integration author

### `SnapshotRestoreLineageMode`

- Owner module:
  [mod.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/diagnostics/policy/mod.rs)
- Current public path:
  `worth_signal::facade::SnapshotRestoreLineageMode`
- What it really controls:
  how restore history is recorded in lineage
- Current default:
  compact global for operational/development, per-node for forensic only when
  explicitly selected
- Likely audience:
  advanced runtime user

### `RuntimeMerge`

- Owner module:
  [guided.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/guided.rs)
- Current public path:
  `worth_signal::facade::runtime::RuntimeMerge`
- What it really controls:
  guided specialist merge entry
- Current default:
  no merge unless used
- Likely audience:
  specialist, integration author

### `BranchMergeReconciliationPolicy` and raw merge policy types

- Owner module:
  [policy.rs](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/merge/policy.rs)
- Current public path:
  `worth_signal::facade::integration::*`
- What they really control:
  raw merge reconciliation behavior
- Current default:
  driven by merge planning/execution internals unless caller drops to raw merge
  types
- Likely audience:
  specialist, integration author

---

## Ownership Problems

These are the real split-owner and duplicate-official problems surfaced by the
inventory.

## 1. Runtime posture is split across presets, builder, runtime mutation, and graph mutation

Competing controls:

- `SignalRuntime::build_for`
- `SignalRuntime::*_for`
- `SignalRuntimeBuilder::runtime_policy`
- `SignalRuntime::set_runtime_policy`

Problem:

- the graph mutator is crate-internal and must not become a second public
  policy front door

Target owner:

- runtime posture should be owned by `SignalRuntime` preset constructors on the
  normal path
- builder runtime policy should be the advanced setup owner
- graph mutation should be contained as lower-level support, not co-equal

---

## 2. Diagnostics richness has one runtime-policy owner

Target owner:

- diagnostics richness should be owned by `SignalRuntimePolicy`
- tier/profile selection should be expressed through named posture presets, not
  as a competing runtime mutation API

---

## 3. Comparator control is spread across three layers plus raw node overrides

Competing controls:

- runtime fallback comparator
- tier default comparator
- recipe-defined comparator
- raw node contract comparator override

Problem:

- these are legitimate layered controls, but the ownership hierarchy is not
  clear in the public story
- users can easily see several knobs without knowing which one is the right
  level for the decision they mean

Target owner:

- recipe-level comparator owns per-computation override
- tier policy owns grouped runtime behavior
- runtime fallback comparator is the specialist last-resort default
- raw node override stays specialist-only

---

## 4. Checkpoint configuration has a shorthand owner and a full owner

Competing controls:

- `checkpoint_barrier(...)`
- `checkpoint_policy(...)`

Problem:

- this one is less harmful than the runtime posture split
- still, the public shape should make it obvious that `checkpoint_barrier(...)`
  is just the shorthand for the ordinary case and `checkpoint_policy(...)` is
  the real advanced owner

Target owner:

- `CheckpointPolicy` is the canonical owner
- `checkpoint_barrier(...)` is a convenience front door for the common one-rule
  case

---

## 5. Tier control is split between build-time seeding and runtime mutation

Competing controls:

- `SignalRuntimeBuilder::tier_policy(...)`
- `SignalRuntime::set_tier_policy(...)`
- `SignalRuntime::set_node_tier(...)`

Problem:

- build-time and runtime-time tier control are both real
- what is missing is a clear story: build-time for setup, runtime-time for
  live refinement

Target owner:

- `TierPolicy` remains the canonical policy type
- builder seeding owns setup-time configuration
- runtime mutation remains legitimate advanced control for live systems

---

## 6. Execution parallelism is split between posture and explicit executor plumbing

Competing controls:

- `SignalRuntimePolicy::with_parallel_admission(...)`
- explicit `StageExecutor`
- explicit `ParallelExecutionPolicy`

Problem:

- posture and execution tuning are both real, but they affect different layers
- the runtime policy controls whether parallel work is admitted
- executor types control how explicitly requested work runs

Target owner:

- runtime posture owns admission defaults
- explicit executor types own request-level execution topology

---

## 7. History and restore policy is split between runtime posture and raw restore types

Competing controls:

- `SignalRuntimePolicy::with_snapshot_restore_lineage_mode(...)`
- `SnapshotRestoreIntent` and related restore-mode types

Problem:

- one group controls retained history richness and restore-lineage style
- the other controls actual restore behavior
- both are valid, but the docs need to make the split obvious so users do not
  confuse "how restore is recorded" with "how restore behaves"

Target owner:

- runtime posture owns restore-history richness
- history/restore types own restore behavior itself

---

## Ownership Map

This is the canonical owner map Phase 5 should implement.

| Decision | Canonical owner | Supporting surfaces |
| --- | --- | --- |
| Runtime posture | `SignalRuntime` preset constructors and `SignalRuntimePolicy` | builder runtime policy for abnormal setup |
| Diagnostics richness and retention | `SignalRuntimePolicy` | `DiagnosticsTier` as a preset label only |
| Parallel admission | `SignalRuntimePolicy` | explicit executor types for request-level override |
| Explicit execution topology | `StageExecutor` / `ParallelExecutionPolicy` | runtime posture only gates admission |
| Comparator fallback | runtime advanced setup/config | tier and recipe overrides sit above it |
| Tier-scoped comparator behavior | `TierPolicy` | runtime fallback below, recipe override above |
| Per-computation comparator behavior | `Recipe` | raw node contract override only for specialist path |
| Condition gating | node declaration and advanced resolver APIs | not runtime posture |
| Tier scheduling and propagation | `TierPolicy` | node tier assignment selects which tier applies |
| Node-to-tier assignment | runtime advanced control | tier policy defines behavior, assignment selects it |
| Checkpoint barrier behavior | `CheckpointPolicy` | `checkpoint_barrier(...)` as shorthand |
| Restore history richness | `SignalRuntimePolicy` | history APIs read the result |
| Restore behavior | `RuntimeHistory` plus restore intent types | runtime posture should not co-own this |
| Merge behavior | `RuntimeMerge` guided flow | raw merge policy types stay specialist |

---

## Target Public Shape

This is the public shape we should code toward.

## 1. Runtime posture

Target shape:

- preset first
- bounded advanced override second

Published story:

- normal path:
  `SignalRuntime::build_for::<Ctx>(graph)`
- explicit posture choice:
  `SignalRuntime::{operational_for, development_for, forensic_for, ...}`
- abnormal setup:
  builder plus one runtime policy object

Policy:

- do not present graph-level runtime policy mutation as a normal owner

---

## 2. Diagnostics richness and retention

Target shape:

- preset plus bounded advanced section

Published story:

- start with a named runtime posture
- refine explanation/provenance/replay/history limits only if needed

Policy:

- `SignalRuntimePolicy` stays public
- `RetentionBudget` and `ReconstructionBudget` stay public but specialist
- `DiagnosticsTier` stays as a preset label, not a competing top-level owner

---

## 3. Execution and parallelism

Target shape:

- runtime posture owns defaults
- advanced execution objects own explicit override

Published story:

- most users do not touch executor policy
- advanced users can pass explicit `StageExecutor`
- specialists can tune `ParallelExecutionPolicy`

Policy:

- do not lead docs with executor topology
- do not move executor detail into the normal runtime setup path

---

## 4. Comparator behavior

Target shape:

- bounded advanced section

Published story:

- most users accept defaults
- if you want one family/computation to behave differently, use recipe-level
  comparator override
- if you want a broader class of nodes to behave differently, use tier policy
- runtime fallback comparator is specialist-only

Policy:

- comparator resolver plumbing stays advanced
- raw node contract comparator override stays specialist

---

## 5. Conditions and evaluation gating

Target shape:

- declaration-time primary
- advanced resolver support secondary

Published story:

- ordinary users express gating on the node/computation
- condition resolver types are for advanced/runtime-bridge work

---

## 6. Tiers and checkpoints

Target shape:

- bounded advanced section

Published story:

- tiers and checkpoints are real differentiators, but not day-one material
- `TierPolicy` and `CheckpointPolicy` are the canonical owners
- builder convenience methods stay, but they should read like setup helpers, not
  rival policy systems

---

## 7. History, restore, and merge policy

Target shape:

- specialist guided surface with raw support underneath

Published story:

- use `runtime.history()` for branch/snapshot/replay/restore work
- use `runtime.merge()` for guided merge work
- raw restore and merge policy structs stay public only where they encode real
  specialist control

Policy:

- do not let raw merge policy types define the normal runtime story

---

## Consolidation Decisions For The First Code Tranche

These are the decisions the next implementation pass should follow.

## Runtime posture

- Keep:
  preset constructors and `SignalRuntimePolicy`
- Contain:
  graph-level runtime-policy mutation
- Removed in Phase 8:
  `set_diagnostics_profile(...)`

## Diagnostics richness

- Keep:
  `SignalRuntimePolicy` posture presets and bounded `with_*` overrides
- Keep but contain:
  raw budget structs
- Keep but contain:
  tier/profile selection as a preset input; it does not own parallel admission

## Comparators

- Keep:
  `VersionComparatorPolicy`, `TierPolicy`, `Recipe::with_comparator(...)`
- Keep but contain:
  fallback comparator and resolver plumbing

## Tiers and checkpoints

- Keep:
  `TierPolicy`, `CheckpointPolicy`
- Keep as shorthand:
  `checkpoint_barrier(...)`
- Keep as advanced live control:
  runtime tier assignment and runtime tier-policy mutation

## History and merge

- Keep:
  `runtime.history()` and `runtime.merge()`
- Keep but contain:
  raw restore-mode types and raw merge policy types

---

## Compatibility Notes

These are the compatibility rules the implementation phase should respect.

## 1. Do not break the preset story

If we change anything, the normal path should still be:

- `build_for::<Ctx>(...)`
- named presets when needed
- builder only for abnormal setup

## 2. The former diagnostics-profile compatibility path is removed

Use `reset_runtime_policy_to_tier(...)` for a stock preset reset and
`set_runtime_policy(...)` for a complete policy change. There is no separate
diagnostics-profile mutation alias.

## 3. Builder and runtime mutation can both remain where they encode different moments

We do not need to delete all duplication mechanically.

What matters is that:

- builder owns setup-time configuration
- runtime mutation owns live refinement
- docs and naming make that split obvious

## 4. Specialist raw types can remain public without dominating the product

Phase 5 should prefer containment and stronger guided ownership before hard
removal.

The main problem is not lack of power.

It is unclear ownership and too many equally respectable-looking knobs.

---

## Exit For This Batch

This first batch is complete when we can answer these questions without
guessing:

1. What knob owns runtime posture?
2. What knob owns diagnostics richness?
3. What knob owns comparator behavior at each layer?
4. What knob owns checkpoint behavior?
5. What knob owns restore behavior versus restore-history richness?
6. Which APIs are normal, advanced, specialist, or transitional?

This document is the answer set the next code tranche should implement.
