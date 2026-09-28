# worth-signal Architecture V2 — Structural Redesign

> **Status:** Pre-production. All changes are breaking-change-safe.
>
> **Scope:** Architectural redesign of `worth-signal` applying the same rigor as the [relational architecture doc](../WORTH-relational/relational_architecture.md) — type-as-contract, contract duality, declarative effects, subsystem decomposition, state-derived context, and commit result envelopes.
>
> **Relationship to V1:** This document supersedes Batches C–G of [signal_architecture.md](./signal_architecture.md). Batches A and B (R1–R7) and the landed items from Batch D (R11, R12, R13) are *preserved* — they form the foundation this document builds on.
>
> **Note:** Source links in this document pointed at files that have since moved or been removed from `crates/worth-signal`. They are kept as plain paths for history.

---

## Table of Contents

1. [What Landed from V1](#what-landed-from-v1)
2. [Phase S1 — Subsystem Decomposition](#phase-s1--subsystem-decomposition)
3. [Phase S2 — Contract System](#phase-s2--contract-system)
4. [Phase S3 — Declarative Effects & Computation Model](#phase-s3--declarative-effects--computation-model)
5. [Phase S4 — Transaction Architecture](#phase-s4--transaction-architecture)
6. [Phase S5 — Pipeline & Performance](#phase-s5--pipeline--performance)
7. [Phase S6 — Safety Architecture](#phase-s6--safety-architecture)
8. [Phase S7 — API Surface & Facade](#phase-s7--api-surface--facade)
9. [Phase S8 — Context-Aware Computation](#phase-s8--context-aware-computation)
10. [Phase S9 — Performance Enforcement Addendum](#phase-s9--performance-enforcement-addendum)
11. [What Must Be Preserved](#what-must-be-preserved)
12. [Sequencing](#sequencing)

---

## What Landed from V1

These items are complete and form the structural floor for this document:

| V1 Item | What It Did | Current File |
|---|---|---|
| R1 | `transition_clean/dirty/maybe_stale` on `NodeEntry` | [entry.rs](../../crates/worth-signal/src/data/node/entry.rs) |
| R2 | `PartitionScoped` trait + `scopes_overlap` | [output.rs](../../crates/worth-signal/src/data/output.rs) |
| R3 | `DependencySnapshotEntry` + `DependencySortKey` | [dependency.rs](../../crates/worth-signal/src/data/dependency.rs) |
| R4 | `with_scratch` closure-based lease | [graph.rs](../../crates/worth-signal/src/data/graph/runtime/graph.rs) |
| R5/R6 | `reconcile_dependencies` + edge ceremony | [mutation.rs](../../crates/worth-signal/src/data/graph/topology/mutation.rs) |
| R7 | `InvalidationTraversal` pipeline struct | [routing.rs](../../crates/worth-signal/src/logic/invalidation/routing.rs) |
| R11 | `SegmentedStore<T, Id>` + type aliases | [segmented.rs](../../crates/worth-signal/src/data/graph/storage/segmented.rs) |
| R12 | Telemetry sub-structs (7 domain groups) | [telemetry.rs](../../crates/worth-signal/src/data/telemetry.rs) |
| R13 | `replay_where` filter consolidation | [replay.rs](../../crates/worth-signal/src/data/graph/diagnostics_access/replay.rs) |

> [!NOTE]
> R14 (stale_error dedup) is trivial and should be done alongside any Batch D cleanup. It is not tracked in this document.

---

## Phase S1 — Subsystem Decomposition

**Kernel reference:** [Relational C1 — Runtime Subsystems](../WORTH-relational/relational_architecture.md).

### Problem

`SignalGraph` ([graph.rs](../../crates/worth-signal/src/data/graph/runtime/graph.rs)) is a 13-field god struct mixing:

```rust
pub struct SignalGraph {
    // Arena (identity management)
    nodes: Vec<Slot>,
    free_list: Vec<u32>,
    free_slots: DenseBitset,
    active_nodes: u32,
    compaction: CompactionState,

    // Traversal infrastructure
    scratch: TraversalScratch,
    scratch_lease: Option<ScratchLeaseKind>,

    // Edge storage (topology)
    dependency_edges: DependencyEdgeStore,
    subscriber_edges: SubscriberEdgeStore,
    dependency_snapshots: DependencySnapshotStore,

    // Runtime state (observation/interning)
    partition_interner: PartitionInterner,
    telemetry: RuntimeTelemetry,
    diagnostics: DiagnosticsState,
}
```

Five distinct concerns share one mutable borrow. Adding a method to diagnostics forces reasoning about arena liveness. Adding an edge store method forces reasoning about scratch state.

Above this, `SignalRuntime<D, I, E, Ctx, T>` (`crates/worth-signal/src/logic/transaction/runtime/state/runtime_state.rs`) is 698 lines mixing graph ownership, branch management, snapshot capture/restore, event bus, diagnostics passthrough, tier configuration, keyed computation, and transactions — all behind 5 type parameters.

### Design

#### S1.1 — `SignalGraph` Subsystem Split

Decompose `SignalGraph` into subsystem structs accessible through a `GraphParts` destructuring pattern (matching WORTH-kernel's `BRepWorkspace::as_parts_mut()`):

```rust
pub struct NodeArena {
    nodes: Vec<Slot>,
    free_list: Vec<u32>,
    free_slots: DenseBitset,
    active_nodes: u32,
    compaction: CompactionState,
}

pub struct EdgeTopology {
    dependency_edges: DependencyEdgeStore,
    subscriber_edges: SubscriberEdgeStore,
    dependency_snapshots: DependencySnapshotStore,
}

pub struct TraversalResources {
    scratch: TraversalScratch,
    scratch_lease: Option<ScratchLeaseKind>,
}

pub struct RuntimeObservation {
    partition_interner: PartitionInterner,
    telemetry: RuntimeTelemetry,
    diagnostics: DiagnosticsState,
}

impl SignalGraph {
    /// Destructure for concurrent borrowing of independent subsystems.
    pub(crate) fn as_parts_mut(&mut self) -> (
        &mut NodeArena,
        &mut EdgeTopology,
        &mut TraversalResources,
        &mut RuntimeObservation,
    ) { ... }
}
```

The public `SignalGraph` type stays — it is the composed whole. But internal code calls `as_parts_mut()` to borrow only what it needs, eliminating false borrow conflicts.

#### S1.2 — `SignalRuntime` Subsystem Split

`SignalRuntime` currently delegates ~30 methods directly to `self.graph.method()`. These passthrough methods exist because the runtime cannot expose the graph without exposing everything. With subsystem decomposition:

```rust
pub struct SignalRuntime<D, I, E, Ctx, T> {
    graph: SignalGraph,
    config: SignalRuntimeConfig<T>,
    checkpoint: CheckpointRuntime<D, I>,
    event_bus: EventBus<E, D, Ctx>,
    branches: BranchManager<D, I, T>,   // extracted from inline BTreeMaps
    telemetry: RuntimeTelemetry,
}
```

`BranchManager` replaces the inline `BTreeMap<SignalBranchId, RuntimeBranchState>` + `BTreeMap<SignalSnapshotId, RuntimeBranchState>` and absorbs the `capture_branch_state` / `load_branch_state` / `synchronize_branch_catalogs` methods that currently clutter `runtime_state.rs`.

### Files Modified

| File | Change |
|---|---|
| [graph.rs](../../crates/worth-signal/src/data/graph/runtime/graph.rs) | Split fields into `NodeArena`, `EdgeTopology`, `TraversalResources`, `RuntimeObservation`; add `as_parts_mut()` |
| `crates/worth-signal/src/logic/transaction/runtime/state/runtime_state.rs` | Extract `BranchManager`; reduce passthrough surface |
| All internal callers of `&mut SignalGraph` | Use `as_parts_mut()` where borrowing independent subsystems |

---

## Phase S2 — Contract System

**Kernel reference:** [Relational F2 — RecordProjection](../WORTH-relational/relational_architecture.md) and [Relational D4 — Intent Contracts](../WORTH-relational/relational_architecture.md).

> [!NOTE]
> Phase S9 extends this phase. `NodeContract` becomes the main performance
> contract and grows explicit equivalence, path-class, maintenance, and
> artifact-policy fields.

### Problem

In worth-relational, **write contracts** (`MutationIntent::invariant_contract()`) and **read contracts** (`RecordProjection::required_aspects()`) are declared up front on the type. The pipeline uses their intersection for aspect-aware invalidation.

In worth-signal, the equivalent information exists but is scattered:

- **What a node reads** (its dependency subscriptions) is only known after evaluation — it is a *side effect* of `PreparedDependencyCapture`, not a declaration.
- **What a node produces** (its output aspects and partition scopes) is only known from the `NodeEvaluationResult` returned at runtime.
- **What invalidation propagates** (dirty aspects and scopes) is inferred on the fly via `subscribes_to_aspect()`.

None of this is declared up front. The pipeline has no way to skip unnecessary work because it doesn't know what a node cares about until it runs.

### Design

#### S2.1 — `NodeContract` Trait

The read-path equivalent of `RecordProjection`. A node declares its dependency contract up front:

```rust
/// Declared on node registration — what this node reads and produces.
pub struct NodeContract {
    /// Which aspects this node subscribes to on its dependencies.
    pub reads: AspectMask,
    /// Which aspects this node may produce when evaluated.
    pub produces: AspectMask,
    /// Which partition scopes this node cares about, if any.
    /// None = all partitions (whole-partition subscription).
    pub partition_scope: Option<Vec<PartitionSubscription>>,
}
```

This is registered at node creation time via the `NodeBuilder`:

```rust
let node = graph.node()
    .with_contract(NodeContract {
        reads: AspectMask::from_aspect(Aspect(0)),
        produces: AspectMask::from_aspect(Aspect(0)),
        partition_scope: None,
    })
    .build();
```

Nodes that don't register a contract default to a wildcard contract (`reads: ALL, produces: ALL, scope: None`) — backward compatible, no behavioral change.

#### S2.2 — Contract Duality: Invalidation × Evaluation

The invalidation path pushes *what changed* (aspect + scopes). The evaluation path checks *what a node depends on*. These are duals:

| Direction | Contract | Current Location | After S2 |
|---|---|---|---|
| **Write** (invalidation) | "I changed aspect A in scope S" | `mark_dirty_with_regions(source, aspect, regions)` | Same, but planner can skip nodes whose `reads` mask doesn't intersect `aspect` |
| **Read** (evaluation) | "I depend on aspect A in scope S" | Implicit in `PreparedDependencyCapture` | Declared in `NodeContract.reads` + `.partition_scope` |

The pipeline uses contract intersection to prune the plan: if a node's `reads` mask doesn't intersect the combined `changed_aspects` mask of its dirty dependencies, the planner can skip it without evaluation.

#### S2.3 — Aspect-Aware Planner Pruning

Currently [plan_builder.rs](../../crates/worth-signal/src/logic/planner/planning/mod.rs) includes all `Dirty` and `MaybeStale` nodes. With contracts:

```rust
fn should_include_in_plan(
    graph: &SignalGraph,
    node: NodeId,
    dirty_aspects: AspectMask,
) -> bool {
    let contract = graph.get_contract(node);
    // If the node's reads don't overlap with what changed, skip it
    contract.reads.intersects(dirty_aspects)
}
```

This is the same optimization as relational's D4 topology inference — the `union_mask` determines which checks to run.

#### S2.4 — Context-Type in Contracts

A node's contract should also declare which **domain context** it requires for evaluation. Signal currently has `Ctx` as a type parameter on `SignalRuntime<D, I, E, Ctx, T>`, but evaluation closures never receive it. Different computations need different contexts — a geometry kernel evaluation needs a model snapshot, a dashboard aggregation needs cross-project summaries, an admin metric needs system-level state.

The contract declares this:

```rust
pub struct NodeContract {
    pub reads: AspectMask,
    pub produces: AspectMask,
    pub partition_scope: Option<Vec<PartitionSubscription>>,
    /// Which context layer this node requires.
    /// None = context-free (pure computation from dependencies).
    pub required_context: Option<ContextRequirement>,
}

pub enum ContextRequirement {
    /// Node needs the domain context (Ctx parameter)
    DomainContext,
    /// Node needs a relational snapshot (bridge integration)
    RelationalSnapshot,
    /// Node is context-free — pure function of its inputs
    None,
}
```

The planner uses this to verify that the required context is available before scheduling evaluation. If a node requires `RelationalSnapshot` but the transaction was started without a bridge, the planner reports a contract violation at planning time instead of a runtime panic during evaluation.

This is the signal equivalent of the frontend's `ProjectContextService` — each computation declares its context dependency, and the framework verifies availability before execution.

### Files Modified

| File | Change |
|---|---|
| [entry.rs](../../crates/worth-signal/src/data/node/entry.rs) | Add `NodeContract` field to `NodeEntry` |
| [construction](../../crates/worth-signal/src/data/graph/construction) | Add `.with_contract()` to `NodeBuilder` |
| [planning/mod.rs](../../crates/worth-signal/src/logic/planner/planning/mod.rs) | Use `NodeContract.reads` to prune plan; verify `required_context` |
| [routing.rs](../../crates/worth-signal/src/logic/invalidation/routing.rs) | Use `NodeContract.reads` to skip subscribers that don't care about the changed aspect |

---

## Phase S3 — Declarative Effects & Computation Model

**Kernel reference:** [Relational B5 — Declarative Effect Assembly](../WORTH-relational/relational_architecture.md).

> [!NOTE]
> Phase S9 extends this phase by splitting hot operational effect data from
> optional diagnostic materialization inputs.

### Problem

`crates/worth-signal/src/logic/evaluation/engine/result_apply.rs` `apply_evaluation_result_with_policy()` is a 137-line function that mixes:
1. **Comparison logic** — output identity/continuity token checks (L53–L71)
2. **Dependency snapshot building** — `build_dep_snapshot` + `count_meaningful_input_changes` (L72–L73)
3. **Trace assembly** — constructing `TraceSummary` from 15 fields (L79–L113)
4. **State transition** — `entry.transition_clean()` (L119)
5. **Telemetry** — incrementing counters conditionally (L123–L134)
6. **Downstream suppression** — `suppress_downstream_if_identity_unchanged` (L128–L130)

Every new evaluation behavior requires editing this monolith. Adding telemetry means adding more branches. Adding a new comparison policy means adding more conditions.

### Design

#### S3.1 — `EvaluationEffect` Struct

Separate domain-level computation outcome from framework bookkeeping:

```rust
/// The pure result of evaluating a signal node.
/// Contains what changed — not how to apply it.
pub struct EvaluationEffect {
    pub node: NodeId,
    pub aspect_version: AspectVersion,
    pub output_change: OutputChange,
    pub output_identity: Option<OutputIdentity>,
    pub continuity_token: Option<OutputIdentity>,
    pub changed_regions: Vec<ChangedRegion>,
    pub labels: Vec<String>,
    pub dependency_snapshot: DependencySnapshot,
    pub meaningful_input_changes: u32,
    pub recomputed: bool,
    pub memoized_origin: MemoizedResultOrigin,
    pub keyed_context: Option<PreparedKeyedContext>,
}
```

#### S3.2 — `apply_effect` Pipeline

The current monolith becomes a pipeline of small, testable phases:

```rust
impl SignalGraph {
    pub(crate) fn apply_effect(
        &mut self,
        effect: EvaluationEffect,
        comparator: VersionComparatorPolicy,
    ) -> Result<AppliedEffectReport, SignalError> {
        let comparison = self.compare_output(&effect, comparator);
        let trace = self.build_trace(&effect, &comparison);
        self.transition_node_clean(effect.node, effect.aspect_version, trace)?;
        self.commit_dependency_snapshot(effect.node, effect.dependency_snapshot)?;
        let suppressed = self.suppress_if_unchanged(effect.node, &comparison)?;
        self.record_effect_telemetry(&effect, &comparison, suppressed);
        Ok(AppliedEffectReport { comparison, suppressed })
    }
}
```

Each phase is a separate method that can be unit-tested. Adding a new comparison policy means adding a new `compare_output` variant, not editing a 137-line function.

#### S3.3 — Commit Ceremony Extraction (Transaction)

`crates/worth-signal/src/logic/transaction/runtime/transaction/transaction_commit.rs` has the **same 30-line rollback ceremony copy-pasted 3 times** (L25–L65, L69–L109, L176–L213):

```
1. compute rollback_patch_count
2. event_bus.rollback(runtime_ctx)
3. rollback_graph_state()
4. construct RollbackDiagnostic
5. construct FailureSummary
6. push replay events
7. increment poison count
8. finalize_semantic_delta(true)
9. return error
```

Extract into:

```rust
fn fail_and_rollback(
    &mut self,
    runtime_ctx: &mut Ctx,
    reason: &str,
    error: SignalError,
) -> Result<TransactionOutcome, SignalError> {
    let rollback_patch_count = self.rollback_patch_count();
    self.event_bus.rollback(runtime_ctx);
    self.rollback_graph_state()?;
    // ... single copy of the ceremony
}
```

#### S3.4 — Evaluation Verdicts

**Kernel reference:** [Relational D5 — Three-State Verdicts](../WORTH-relational/relational_architecture.md).

Signal evaluation has outcomes that are actually ternary, but this is inferred from scattered booleans across `result_apply.rs` and `prepared_apply.rs`:

- `recomputed == true` → the closure ran and produced new output
- `propagation_suppressed == true` → output identity matched, downstream propagation skipped
- on-demand / condition-deferred → the node was skipped entirely by condition gating

Formalize this as a first-class verdict:

```rust
pub enum EvaluationVerdict {
    /// Closure ran, output changed, downstream propagation needed.
    Recomputed,
    /// Closure ran, output identity unchanged, downstream suppressed.
    Suppressed { reason: SuppressionReason },
    /// Node was not evaluated due to condition/on-demand gating.
    Deferred { reason: DeferralReason },
}

pub enum SuppressionReason {
    OutputIdentityUnchanged,
    ContinuityTokenUnchanged,
    ComparatorMatch,
}

pub enum DeferralReason {
    ConditionNotMet,
    OnDemandNotRequested,
    DebounceWindow,
}
```

The verdict is attached to `EvaluationEffect` and flows into `TransactionResult` (S4), making the pipeline self-describing. The caller knows not just *that* a node was evaluated, but *what the evaluation decided*.

#### S3.5 — `defineComputation` Pattern

**Frontend reference:** Inspired by frontend `defineCrudResource` / `useCrudResource` patterns. If the agent working on this wants to see examples, ask the user — the frontend code is in a separate workspace.

Currently, defining a computation requires multi-step ceremony:

```rust
// Current: 6 separate calls to set up one computation
let family = runtime.register_computation_family("volumes");
let node = runtime.keyed_node(&family, "body_42");
runtime.set_node_tier(node, Tier::OnDemand);
runtime.set_fallback_comparator(OutputIdentity);
// ... then separately wire up the evaluator closure in the transaction
```

This is the same problem the frontend had before `defineCrudResource` — scattered setup that must be kept in sync manually.

`defineComputation` bundles everything into a single declaration:

```rust
let volumes = runtime.define_computation(ComputationSpec {
    family: "volumes",
    contract: NodeContract {
        reads: AspectMask::from_aspect(GEOMETRY),
        produces: AspectMask::from_aspect(METRICS),
        partition_scope: None,
        required_context: Some(ContextRequirement::RelationalSnapshot),
    },
    tier: Tier::OnDemand,
    comparator: VersionComparatorPolicy::OutputIdentity,
    evaluator: |ctx, deps| {
        // compute volumes from relational snapshot
        Ok(NodeEvaluationResult { ... })
    },
});

// Later: just use it
let node = volumes.keyed("body_42");
let result = volumes.evaluate(node)?;
```

The `ComputationSpec` is the signal equivalent of the frontend's `CrudResourceDefinition` — a single source of truth for everything the framework needs to know about a computation.

> [!NOTE]
> `defineComputation` is a convenience API built on top of `NodeContract` (S2.1) and context requirements (S2.4). It does not introduce new primitives — it composes existing ones into a zero-boilerplate surface.

### Files Modified

| File | Change |
|---|---|
| `crates/worth-signal/src/logic/evaluation/engine/result_apply.rs` | Replace monolith with `EvaluationEffect` + `apply_effect` pipeline; add `EvaluationVerdict` |
| `crates/worth-signal/src/logic/transaction/runtime/transaction/transaction_commit.rs` | Extract `fail_and_rollback`, collapse 3 copies to 3 one-line calls |
| [prepared_apply.rs](../../crates/worth-signal/src/logic/evaluation/engine/prepared_apply.rs) | Construct `EvaluationEffect` and call `graph.apply_effect()` |
| [config.rs](../../crates/worth-signal/src/logic/transaction/runtime/config.rs) | Add `ComputationSpec`, `define_computation()`, and computation registry |
| `crates/worth-signal/src/logic/transaction/runtime/state/runtime_state.rs` | Expose `define_computation()` on `SignalRuntime` |

---

## Phase S4 — Transaction Architecture

**Kernel reference:** [Relational E1 — Commit Decision Log](../WORTH-relational/relational_architecture.md) and [Relational E2 — Commit Result Envelope](../WORTH-relational/relational_architecture.md).

### Problem

`TransactionOutcome` is currently a bare enum:

```rust
pub enum TransactionOutcome {
    Committed,
    RolledBack,
    Poisoned,
}
```

After a commit, the caller has **no structured information** about what happened: how many nodes were evaluated, which aspects changed, what the execution report contained, whether any suppression occurred, how many event epochs completed. The caller must query diagnostics separately, after the fact, hoping the right data was retained.

### Design

#### S4.1 — `TransactionResult` Envelope

```rust
pub struct TransactionResult {
    pub outcome: TransactionOutcome,
    pub execution_report: Option<ExecutionReport>,
    pub timing: TransactionTiming,
    pub touched_nodes: u32,
    pub evaluation_summary: EvaluationSummary,
    pub event_epochs: Vec<EventEpochSummary>,
    pub rollback: Option<RollbackDiagnostic>,
    pub warnings: Vec<AdvisoryRecord>,
    pub decision_summary: DecisionSummary,
    pub decision_log: DecisionLog,
    pub integrity_markers: IntegrityMarkers,
    pub performance_accounting: PerformanceCounterSurface<'static>,
}

pub struct TransactionTiming {
    pub total_nanos: u128,
    pub evaluation_nanos: u128,
    pub event_flush_nanos: u128,
    pub commit_nanos: u128,
}

pub struct EvaluationSummary {
    pub nodes_evaluated: u32,
    pub nodes_recomputed: u32,
    pub nodes_suppressed: u32,
    pub plans_built: u32,
    pub stages_executed: u32,
}
```

The caller gets a self-describing transaction result without querying diagnostics
or producer internals:

```rust
let result = runtime.transaction(ctx, |txn| {
    txn.mark_dirty(node, aspect)?;
    Ok(())
})?;

// Structured result immediately available
println!("evaluated {} nodes", result.evaluation_summary.nodes_evaluated);
```

#### S4.2 — `SemanticDelta` Consolidation

`TransactionSemanticDelta` currently stores replay events as `Vec<(ReplayEventKind, String, Option<..>, Option<..>)>` tuples. Replace with named struct:

```rust
pub struct TransactionReplayEntry {
    pub kind: ReplayEventKind,
    pub detail: String,
    pub execution_record_id: Option<ExecutionRecordId>,
    pub semantic_segment_id: Option<SemanticSegmentId>,
}
```

### Files Modified

| File | Change |
|---|---|
| `crates/worth-signal/src/logic/transaction/runtime/transaction/transaction_commit.rs` | Return `TransactionResult` instead of bare `TransactionOutcome` |
| [transaction_types.rs](../../crates/worth-signal/src/logic/transaction/runtime/transaction/transaction_types.rs) | Add `TransactionResult`, `TransactionTiming`, `EvaluationSummary`, `TransactionReplayEntry` |
| All callers of `.commit()` / `.transaction()` | Receive `TransactionResult` |

---

## Phase S5 — Pipeline & Performance

**Kernel reference:** Relational D4 (topology inference) and frontier patterns from WORTH-kernel.

> [!NOTE]
> Phase S9 is the enforcement completion layer for this phase. S5 establishes
> the pipeline; S9 makes the pipeline consume proof-bearing forms and
> batch-first contracts by default.

> [!NOTE]
> This phase subsumes V1's R8 (zero-allocation planner), R9 (feature-gated execution), and R10 (amortized GC). The designs are refined to align with the subsystem and contract patterns from S1–S2.

### Cross-Cutting Rule — Batch-Scoped Structural Maintenance

> [!IMPORTANT]
> Amortize structural maintenance across a batch boundary whenever
> intermediate states have no semantic value.

This is now a core WORTH Signal architecture rule, not an optional
optimization pattern.

The staged/session-backed execution model already gives the system natural
batch boundaries: planning sessions, evaluation stages, transaction commit,
rollback repair, scenario assembly, and any topology reconciliation pass that
derives one final committed truth. When only that final truth is observable,
WORTH Signal must not pay to maintain every intermediate structural state as if
it were semantically meaningful.

The rule is simple:

- if only the final batch or stage truth is observable
- and intermediate structural states do not affect semantics in-flight
- then structural work must be accumulated first
- and committed once per affected node, source, set, or stage boundary

This applies especially to:

- dependency rewiring and subscriber membership maintenance
- staged prepared-apply pipelines
- scenario/setup graph assembly
- rollback and repair passes
- snapshot, topology, or diagnostics artifacts that can be derived once from a
  merged batch result

**Operational contract**

Batch-first surfaces are the architectural default. Per-edit structural
mutation is not an acceptable hot-path contract unless the caller can prove
that intermediate states are semantically observable and required.

In practice, that means:

- batch/session APIs are the primary operational path
- per-edit topology maintenance is a low-level implementation detail, not a
  design surface
- downstream diagnostics should prefer batch-derived summaries over repeated
  structural reconstruction

**Review rule**

When reviewing hot-path code, ask:

- is any intermediate structural state observable outside this batch?
- are we rewriting storage more than once for the same affected set?
- could this work be accumulated by node, source, or stage and committed once?
- are downstream consumers reconstructing structure that the batch boundary
  already knew?

### S5.1 — Contract-Driven Plan Pruning

After S2 lands, the planner has `NodeContract.reads` available. The `populate_plan_buffers` function in [planning/mod.rs](../../crates/worth-signal/src/logic/planner/planning/mod.rs) currently includes all `Dirty`/`MaybeStale` nodes. With contracts, nodes whose `reads` mask doesn't intersect the propagated `changed_aspects` mask are excluded at planning time, before any evaluation runs.

This is the signal equivalent of relational's D4 — the contract mask determines which pipeline phases execute.

### S5.2 — Zero-Allocation Planner (V1 R8, Redesigned)

V1's R8 proposed arena-backed cursors. The design is refined:

The planner already has a `build_evaluation_session_with_policy_resolver` that writes into `TraversalScratch`-owned buffers (`scratch.planner_targets`, `scratch.planner_tasks`, `scratch.planner_stages`). The runtime execution path already uses this through `EvaluationSession`.

What remains is ensuring the `EvaluationSession` path is the **primary** path, and the allocating `EvaluationPlan` path is only used for diagnostics/inspection. This is already partially done — it needs completion, not redesign.

### S5.3 — Execution Pipeline Decomposition (V1 R9, Redesigned)

V1's R9 proposed an `ExecutionPass` trait with `#[cfg(feature)]` isolation. Redesigned through S1 subsystem lens:

Instead of a trait, decompose the execution path into subsystem calls with `as_parts_mut()`:

```rust
fn execute_stage(
    graph: &mut SignalGraph,
    session: &EvaluationSession,
    stage_index: usize,
    resolver: &mut impl ComparatorPolicyResolver,
) -> Result<StageExecutionRecord, SignalError> {
    let (arena, topology, traversal, observation) = graph.as_parts_mut();
    // precompute phase: reads arena + topology, doesn't need traversal
    let snapshots = precompute_stage(arena, topology, session, stage_index)?;
    // evaluate phase: uses arena + topology
    let results = evaluate_stage(arena, topology, &snapshots, resolver)?;
    // apply phase: mutates topology + observation
    let report = apply_stage_effects(arena, topology, observation, results)?;
    Ok(report)
}
```

The `#[cfg(feature = "parallel")]` isolation becomes:

```rust
#[cfg(feature = "parallel")]
fn precompute_stage_parallel(...) -> ... { /* rayon parallel map */ }

#[cfg(not(feature = "parallel"))]
fn precompute_stage_serial(...) -> ... { /* sequential */ }

fn precompute_stage(...) -> ... {
    #[cfg(feature = "parallel")]
    if should_parallelize(stage) {
        return precompute_stage_parallel(...);
    }
    precompute_stage_serial(...)
}
```

Conditional compilation is restricted to **function dispatch**, not interleaved within business logic.

### S5.4 — Amortized GC (V1 R10, Redesigned)

V1's R10 proposed incremental GC during traversals. Refined through S1:

With `NodeArena` as a separate subsystem, GC becomes a method on `NodeArena` that doesn't need to reason about edge topology or diagnostics. The arena tracks its own tombstone count and compacts within its own boundary.

Edge cleanup happens in `EdgeTopology` via a separate `prune_dead_edges()` method that runs lazily when the tombstone ratio exceeds a threshold. Since `NodeArena` and `EdgeTopology` are independent subsystems, they can compact independently without stop-the-world coordination.

### S5.5 — Execution Path Collapse

**Frontend reference:** Inspired by frontend component collapse patterns (create-dialog + edit-dialog → single parameterized form). If the agent working on this wants to see examples, ask the user — the frontend code is in a separate workspace.

[runtime_execution.rs](../../crates/worth-signal/src/logic/transaction/runtime/execution/runtime_execution.rs) has `execute_for_commit` (~100 lines) and `execute_for_on_demand` (~100 lines) that are **structurally identical** — they differ only in:
- Request mode (`Default` vs `ForceOnDemand`)
- Target selection (all dirty via `staged_dirty` vs explicit node list)
- Whether they report execution timing to `semantic_delta`

The logic — build plan, precompute snapshots, evaluate stage, apply results, record diagnostics — is the same.

Collapse into a single `execute_evaluation` parameterized by an `ExecutionIntent`:

```rust
pub(crate) enum ExecutionIntent<'a> {
    Commit { dirty_targets: &'a DenseBitset },
    OnDemand { targets: &'a [NodeId] },
}

fn execute_evaluation(
    graph: &mut SignalGraph,
    intent: ExecutionIntent<'_>,
    config: &SignalRuntimeConfig<T>,
    ...
) -> Result<ExecutionReport, SignalError> {
    let (targets, request_mode) = match intent {
        ExecutionIntent::Commit { dirty_targets } => (
            dirty_targets.collect_set_bits(),
            EvaluationRequestMode::Default,
        ),
        ExecutionIntent::OnDemand { targets } => (
            targets.to_vec(),
            EvaluationRequestMode::ForceOnDemand,
        ),
    };
    // Single implementation of the evaluation pipeline
    ...
}
```

This is the same pattern as the frontend collapsing `DialogCreateComponent` and `DialogEditComponent` into a single `DialogFormComponent` parameterized by mode.

### Files Modified

| File | Change |
|---|---|
| [planning/mod.rs](../../crates/worth-signal/src/logic/planner/planning/mod.rs) | Add contract-based pruning to `visit_node` |
| [runtime_execution.rs](../../crates/worth-signal/src/logic/transaction/runtime/execution/runtime_execution.rs) | Collapse `execute_for_commit` / `execute_for_on_demand` into `execute_evaluation`; decompose into subsystem calls |
| `crates/worth-signal/src/logic/planner/execution.rs` | Isolate `#[cfg(feature)]` to dispatch functions |
| [lifecycle.rs](../../crates/worth-signal/src/data/graph/lifecycle) | Subsystem-scoped compaction |

---

## Phase S6 — Safety Architecture

> [!NOTE]
> Phase S9 extends this phase with allocation lifetime scopes, single-consumer
> packet rules, and phase-typed fast-exit progression.

> [!NOTE]
> This phase subsumes V1's R15 (partition-aware validation), R19 (PhaseGuard), R20 (observation purity), R21 (single source of truth), and R22 (transactional mutation). The designs are refined to build on S1–S3 rather than being standalone compile-time safety items.

### S6.1 — Partition-Aware Version Tracking (V1 R15, Unchanged)

The bug from V1 R15 is real and the design is sound. Move `AspectVersion` from a flat integer to a `PartitionVersionMap` so that `count_meaningful_input_changes` compares scope-specific versions:

```rust
pub struct PartitionVersionMap {
    global_version: AspectVersion,
    partition_versions: HashMap<PartitionToken, AspectVersion>,
}
```

> [!IMPORTANT]
> This should land **before S5** (pipeline performance). Optimizing a pipeline that over-evaluates due to false version matches is wasted optimization.

### S6.2 — Phase-Typed Graph Access (V1 R19 + R20, Redesigned)

V1 proposed `GraphHandle<Phase>` with `PhantomData` typestates. With S1's subsystem split, the approach is simpler: each phase borrows only the subsystems it needs.

Invalidation borrows `(&mut NodeArena, &EdgeTopology, &mut TraversalResources, &mut RuntimeObservation)`. Evaluation borrows `(&mut NodeArena, &mut EdgeTopology, &mut TraversalResources, &mut RuntimeObservation)`. Observation borrows `(&NodeArena, &EdgeTopology, &RuntimeObservation)` — all `&self`.

The type system enforces phase restrictions through borrow patterns, not through wrapper types:

```rust
impl SignalGraph {
    /// Observation-only borrow. All fields are &self.
    pub fn observe(&self) -> GraphObserver<'_> {
        GraphObserver {
            arena: &self.arena,
            topology: &self.topology,
            observation: &self.observation,
        }
    }
}

impl<'a> GraphObserver<'a> {
    pub fn explain(&self, node: NodeId) -> Result<NodeExplanation, SignalError> { ... }
    pub fn replay_events(&self) -> &[ReplayEvent] { ... }
    // No mutation methods exist here — compile error if attempted
}
```

V1's R20 (observation purity) falls out naturally: `GraphObserver` only has `&self` references.

### S6.3 — Single Source of Truth (V1 R21, Redesigned)

With `EdgeTopology` as a subsystem, the dual-representation problem (dependencies ↔ subscribers) is contained. `EdgeTopology` owns both stores and enforces that mutations always update **both** through `reconcile_dependencies` (already landed via R6).

The remaining risk is **stale subscriber edges after topology changes**. With S1, this becomes a subsystem invariant: `EdgeTopology` exposes an `assert_bidirectional_consistency(&self)` debug assertion that verifies deps↔subs agreement. This runs in debug builds and tests, not in production.

### S6.4 — Transactional Mutation Safety (V1 R22, Redesigned)

V1 proposed wrapping `&mut SignalGraph` in a `TransactionalMut<'tx>`. With S3's `EvaluationEffect` struct, the design is simpler: mutations during evaluation produce effects, and effects are applied atomically. The transaction only needs to undo `NodeEntry` patches (already handled by `SparsePatchBuffer`) and rollback created nodes (already handled by `rollback_created_nodes`).

The remaining gap is **edge store rollback** — if dependencies are reconciled during evaluation and the transaction rolls back, the edge topology must revert. With `EdgeTopology` as a subsystem, the rollback tracks `(NodeId, old_dependency_set_id)` tuples and restores them atomically.

### S6.5 — Typed Error Hierarchy

**Kernel reference:** [Relational A3 — Typed Error Hierarchy](../WORTH-relational/relational_architecture.md).

`SignalError` is currently string-based:

```rust
// Current: all errors are strings
SignalError::invalid_input(format!("stale NodeId: {id}"))
SignalError::internal("signal scratch is already leased")
SignalError::invalid_input(format!("cycle detected at {node}"))
```

Catching these requires string matching. Relational's A3 proposed typed variants; signal has even more distinct error classes:

```rust
pub enum SignalError {
    /// Node handle has been recycled or refers to a freed slot.
    StaleHandle { node: NodeId, expected_generation: u32 },
    /// Cycle detected during planning or invalidation.
    CycleDetected { path: Vec<NodeId> },
    /// Re-entrant scratch lease (concurrent invalidation/evaluation).
    ScratchReentry { active: ScratchLeaseKind, attempted: ScratchLeaseKind },
    /// Scratch lease mismatch during restore.
    ScratchMismatch { expected: ScratchLeaseKind, restored: ScratchLeaseKind },
    /// Contract violation: node requires context that isn't available.
    ContractViolation { node: NodeId, requirement: ContextRequirement },
    /// Transaction used after commit/rollback.
    TransactionFinished,
    /// Transaction poisoned by a prior error.
    TransactionPoisoned,
    /// Event bus flush failed.
    EventFlushFailed { subscriber: String, source: String },
    /// Snapshot compatibility check failed.
    IncompatibleSnapshot { reason: String },
    /// Generic internal error (escape hatch for truly unexpected conditions).
    Internal { message: String },
}
```

Callers can now match on error kind:

```rust
match result {
    Err(SignalError::StaleHandle { node, .. }) => {
        // Handle stale reference gracefully
    }
    Err(SignalError::CycleDetected { path }) => {
        // Report cycle with full path
    }
    Err(e) => return Err(e),
}
```

### S6.6 — Builder Completeness

**Kernel reference:** [Relational C4 — Fork-Safe Construction](../WORTH-relational/relational_architecture.md).

`SignalRuntimeBuilder` currently accepts all configuration as optional:

```rust
// Current: everything optional, defaults silently applied
let runtime = SignalRuntime::builder(graph)
    // forgot checkpoint policy? defaults to no-op
    // forgot fallback comparator? defaults to Exact
    // forgot diagnostics profile? defaults to minimal
    .build();
```

If you WORTHt to set a checkpoint policy, checkpoints silently become no-ops. If you WORTHt a fallback comparator, on-demand nodes may use the wrong comparison policy.

Enforce critical configuration at compile time using typestate:

```rust
pub struct SignalRuntimeBuilder<Checkpoint = (), Ctx = ()> {
    graph: SignalGraph,
    checkpoint: Checkpoint,
    ctx: PhantomData<Ctx>,
    // optional fields that have safe defaults
    fallback_comparator: VersionComparatorPolicy,
    diagnostics_profile: DiagnosticsProfile,
}

impl SignalRuntimeBuilder<(), ()> {
    pub fn with_checkpoint<D, I>(
        self,
        policy: CheckpointPolicy<D>,
    ) -> SignalRuntimeBuilder<CheckpointRuntime<D, I>, ()> { ... }
}

impl<D, I> SignalRuntimeBuilder<CheckpointRuntime<D, I>, ()> {
    // build() is only available when checkpoint is configured
    pub fn build(self) -> SignalRuntime<D, I, (), (), ()> { ... }
}
```

This is the same pattern as relational's C4 — the builder type tracks which required subsystems have been configured, and `build()` is only available when all required subsystems are present.

### Files Modified

| File | Change |
|---|---|
| [entry.rs](../../crates/worth-signal/src/data/node/entry.rs) | Replace flat `AspectVersion` with `PartitionVersionMap` |
| [graph.rs](../../crates/worth-signal/src/data/graph/runtime/graph.rs) | Add `observe()` method returning `GraphObserver` |
| [mutation.rs](../../crates/worth-signal/src/data/graph/topology/mutation.rs) | Add `assert_bidirectional_consistency` |
| `crates/worth-signal/src/logic/evaluation/engine/result_apply.rs` | Track edge set IDs for rollback |
| [error.rs](../../crates/worth-signal/src/data/error.rs) | Replace string-based `SignalError` with typed enum variants |
| [builder.rs](../../crates/worth-signal/src/logic/transaction/runtime/state/builder.rs) | Add typestate to `SignalRuntimeBuilder` |

---

## Phase S7 — API Surface & Facade

> [!NOTE]
> Phase S9 extends this phase by making batch-first, bulk-first API shape a
> normative architecture rule rather than a style preference.

**Kernel reference:** [Relational F1 — Facade Namespace Organization](../WORTH-relational/relational_architecture.md).

### Problem

[facade.rs](../../crates/worth-signal/src/facade.rs) has ~120 flat `pub use` re-exports. Consumers must know which of the 120 types they need. No organization by domain.

### Design

#### S7.1 — Grouped Facade Namespaces

```rust
// facade.rs
pub mod types {
    // Core types: NodeId, Aspect, AspectMask, AspectVersion, NodeState, etc.
}

pub mod graph {
    // Graph construction and access: SignalGraph, NodeBuilder, NodeContract
}

pub mod evaluation {
    // Evaluation primitives: EvaluationContext, EvaluationEffect, NodeEvaluationResult
}

pub mod planning {
    // Plan types: EvaluationPlan, EvaluationTask, ExecutionReport, etc.
}

pub mod transaction {
    // Transaction types: SignalRuntime, SignalTransaction, TransactionResult
}

pub mod diagnostics {
    // All diagnostic types, replay, explain, lineage
}

pub mod harness {
    // Test harness: SignalHarnessAdapter, SignalScenario, etc.
}

// Top-level re-exports for the most common types
pub use types::{NodeId, Aspect, AspectMask, NodeState};
pub use graph::{SignalGraph, NodeBuilder};
pub use transaction::{SignalRuntime, SignalTransaction, TransactionResult};
```

#### S7.2 — State-Derived Evaluation Strategy

**Kernel reference:** [Relational D6 — State-Derived Invariant Context](../WORTH-relational/relational_architecture.md).

The evaluation engine currently applies the same strategy regardless of graph state. With a state-derived context, the runtime adapts:

```rust
pub struct EvaluationStrategy {
    pub parallelism: ParallelismHint,
    pub gc_pressure: GcPressure,
    pub observation_policy: ObservationLevel,
}

impl SignalGraph {
    pub fn derive_evaluation_strategy(&self) -> EvaluationStrategy {
        let node_count = self.active_nodes();
        let tombstone_ratio = self.tombstone_ratio();
        EvaluationStrategy {
            parallelism: if node_count > 1000 {
                ParallelismHint::Preferred
            } else {
                ParallelismHint::Serial
            },
            gc_pressure: if tombstone_ratio > 0.3 {
                GcPressure::CompactAfterEvaluation
            } else {
                GcPressure::Deferred
            },
            observation_policy: if self.diagnostics_profile().is_verbose() {
                ObservationLevel::Full
            } else {
                ObservationLevel::Minimal
            },
        }
    }
}
```

The transaction uses this to configure its evaluation pass dynamically.

---

## Phase S8 — Context-Aware Computation

**Frontend reference:** Inspired by frontend ambient context patterns (`ProjectContextService`, operations mode, admin mode). If the agent working on this wants to see examples, ask the user — the frontend code is in a separate workspace.

### Problem

`SignalRuntime<D, I, E, Ctx, T>` has a `Ctx` type parameter, but it only flows to the event bus during `commit()`. **Evaluation closures never receive the domain context.** This means:

- Geometry kernel evaluations need a model snapshot → must be captured in a closure upvalue, losing transactional safety
- Dashboard aggregations need cross-project summaries → must be threaded manually
- Admin metrics need system-level state → same manual threading

As the system scales to support projects, operations, and administrative modes (each with different context shapes), this problem multiplies. Every new context type requires a new way to smuggle state into evaluation closures.

In the frontend, this was solved by `ProjectContextService` — context is injected by the framework, not threaded by the consumer. Signal needs the same pattern.

### Design

#### S8.1 — Ambient Evaluation Context

Extend `EvaluationContext` to carry the domain context:

```rust
pub struct EvaluationContext<'graph, Ctx> {
    graph: &'graph SignalGraph,
    node: NodeId,
    domain_context: &'graph Ctx,
}

impl<'graph, Ctx> EvaluationContext<'graph, Ctx> {
    pub fn graph(&self) -> &SignalGraph { self.graph }
    pub fn node(&self) -> NodeId { self.node }
    pub fn domain(&self) -> &Ctx { self.domain_context }
}
```

The transaction injects the domain context at `begin()` time, and it propagates to every evaluation automatically:

```rust
runtime.transaction(ctx, |txn| {
    txn.mark_dirty(node, GEOMETRY)?;
    // ctx is available to all evaluation closures during commit
    Ok(())
})?;
```

#### S8.2 — Multi-Context Support

Different parts of the system operate in different contexts:

| Mode | Context Type | What It Provides |
|---|---|---|
| **Project** | `ProjectContext` | Relational snapshot for a single project |
| **Operations** | `OperationsContext` | Cross-project summaries, fleet metrics |
| **Administrative** | `AdminContext` | System configuration, tenant boundaries |
| **Kernel** | `ModelContext` | BRep workspace, topology access |
| **Simulation** | `SimulationContext` | Tick state, entity world |

The `Ctx` type parameter on `SignalRuntime` already supports this — different runtime instances can have different context types. The key design decision is: **do different computations within the same runtime need different context types?**

Two approaches:

**Option A: Homogeneous context** — all computations in one runtime share one `Ctx`. This is the current model. Different modes use different runtime instances.

**Option B: Heterogeneous context via trait objects** — a single runtime carries a context registry, and computations request their specific context layer:

```rust
pub trait ContextProvider: 'static {
    fn provide<T: 'static>(&self) -> Option<&T>;
}

impl<'graph> EvaluationContext<'graph, dyn ContextProvider> {
    pub fn require<T: 'static>(&self) -> Result<&T, SignalError> {
        self.domain_context.provide::<T>()
            .ok_or(SignalError::ContractViolation { ... })
    }
}
```

> [!IMPORTANT]
> **Recommendation: start with Option A** (homogeneous context per runtime). This is simpler, fully type-safe, and matches how the frontend handles it (separate services per mode, not one service with dynamic dispatch). Option B can be added later if multi-mode runtime becomes a real requirement.

#### S8.3 — Context-Scoped Evaluation

With ambient context, evaluation closures become pure functions of (context + dependencies) instead of closures that capture external state:

```rust
// Before: closure captures model snapshot unsafely
let snapshot = Arc::clone(&model_snapshot);
runtime.transaction(ctx, |txn| {
    txn.evaluate(volumes_node, |eval_ctx| {
        let snap = snapshot.lock();  // lock contention, transactional danger
        compute_volumes(&snap, eval_ctx.dependencies())
    })?;
    Ok(())
})?;

// After: context injected by framework
runtime.transaction(model_ctx, |txn| {
    txn.evaluate(volumes_node, |eval_ctx| {
        let snap = eval_ctx.domain().relational_snapshot();  // safe, transactional
        compute_volumes(snap, eval_ctx.dependencies())
    })?;
    Ok(())
})?;
```

The framework owns the context lifetime. The closure doesn't need to manage `Arc`/`Mutex`/lifetime gymnastics.

With `defineComputation` (S3.5), this becomes even cleaner:

```rust
let volumes = runtime.define_computation(ComputationSpec {
    family: "volumes",
    contract: NodeContract {
        reads: GEOMETRY.into(),
        produces: METRICS.into(),
        partition_scope: None,
        required_context: Some(ContextRequirement::DomainContext),
    },
    tier: Tier::OnDemand,
    comparator: VersionComparatorPolicy::OutputIdentity,
    evaluator: |ctx: &EvaluationContext<ModelContext>, deps| {
        let snapshot = ctx.domain().relational_snapshot();
        compute_volumes(snapshot, deps)
    },
});
```

### Files Modified

| File | Change |
|---|---|
| [context.rs](../../crates/worth-signal/src/logic/context.rs) | Extend `EvaluationContext` with generic `Ctx` parameter |
| `crates/worth-signal/src/logic/transaction/runtime/state/runtime_state.rs` | Thread `Ctx` from `transaction()` through evaluation pipeline |
| [runtime_execution.rs](../../crates/worth-signal/src/logic/transaction/runtime/execution/runtime_execution.rs) | Accept `&Ctx` and pass to evaluation closures |
| [transaction_evaluation.rs](../../crates/worth-signal/src/logic/transaction/runtime/execution/transaction_evaluation.rs) | Inject `Ctx` into evaluation closures |

---

## Phase S9 — Performance Enforcement Addendum

> **Status:** Mandatory completion layer for V2, not an optional appendix.
>
> **Purpose:** This phase extends S2, S3, S5, S6, and S7 with concrete
> architectural forms that enforce performance structurally. The earlier phases
> remain authoritative about subsystem shape and pipeline composition; this
> phase defines how those phases become performance-enforced rather than merely
> performance-aware.

### S9.1 — Performance Enforcement Model

Performance in `worth-signal` is enforced in three layers:

1. **Compile-time enforced** through types, ownership, capability boundaries,
   lifecycle scopes, and API surface shape.
2. **Policy/runtime enforced** through resolved strategies and explicit mode
   selection performed before the hot path.
3. **Counter/test enforced** through boundary-local counters, certification
   workloads, and scale-sensitive regression checks.

The default rule is simple:

> If a performance law can be enforced by shape, `worth-signal` should enforce
> it by shape rather than by reviewer memory.

This addendum exists because the current V2 design already points in this
direction, but still leaves too many performance-critical truths encoded as raw
collections, late branching, or optional discipline.

### S9.2 — Proof-Bearing Pipeline Forms

Once a phase establishes a costly fact, later phases must consume that fact as
an explicit proof-bearing form instead of rediscovering it.

The required architectural families are:

- `Canonical*` for canonicalized, sorted, or deduplicated collections
- `Lowered*` for planner-to-execution lowered forms
- `Resolved*` for policy or strategy selected before execution
- `*Delta` for semantically narrowed change payloads
- `*Summary` for batch-derived structural proofs

Examples that should exist in Signal after this rewrite:

```rust
pub struct CanonicalDependencies(SmallVec<[DependencyEdge; 8]>);
pub struct CanonicalChangedRegions(SmallVec<[ChangedRegion; 4]>);
pub struct DirtyDelta { ... }
pub struct TouchedScopeSummary { ... }
pub struct LoweredStagePlan { ... }
pub struct ResolvedExecutionStrategy { ... }
```

Normative rule:

- later phases may not accept raw `Vec`s if an earlier phase already proved
  canonical order, deduplication, or narrowing
- planner and apply code may not re-sort, re-deduplicate, or re-canonicalize
  collections that should already be in proof-bearing form
- performance-sensitive paths must consume `Canonical*`, `Lowered*`,
  `Resolved*`, `*Delta`, and `*Summary` forms directly

This is the main architectural encoding for laws `10`, `22`, `25`, and `26`.

### S9.3 — Contracts That Must Grow Beyond Current S2

`NodeContract` is no longer only a dependency/read declaration. It becomes the
central performance contract for a node.

The V2 `NodeContract` form from S2 is extended to include:

```rust
pub struct NodeContract {
    pub reads: AspectMask,
    pub produces: AspectMask,
    pub partition_scope: Option<Vec<PartitionSubscription>>,
    pub required_context: Option<ContextRequirement>,
    pub projection_contract: ProjectionContract,
    pub equivalence: EquivalenceContract,
    pub path_class: PathClass,
    pub maintenance_mode: MaintenanceMode,
    pub artifact_policy: ArtifactPolicyClass,
    pub authority_policy: AuthorityPolicy,
}
```

Required supporting forms:

```rust
pub struct ProjectionContract {
    pub consumes: AspectMask,
    pub consumes_partitions: Option<Vec<PartitionSubscription>>,
}

pub struct EquivalenceContract {
    pub identity_basis: IdentityBasis,
    pub suppression_basis: SuppressionBasis,
    pub canonical_dependency_order: CanonicalDependencyOrder,
    pub comparator_basis: ComparatorBasis,
}

pub enum PathClass {
    Operational,
    Rich,
}

pub enum MaintenanceMode {
    IncrementalOnly,
    RebuildAllowed,
    DensityAdaptive,
}

pub enum ArtifactPolicyClass {
    OperationalMinimal,
    DevelopmentRetained,
    ForensicReconstructable,
}

pub enum AuthorityPolicy {
    AuthoritativeOnly,
    SpeculativeThenReconcile,
}
```

These forms are required because reuse, suppression, caching, and hot/cold path
separation must not remain distributed across output identity, continuity
tokens, comparator policy, diagnostics profile, and incidental code branching.

This section amends S2 directly. After S9 lands:

- S2 becomes the home of equivalence contracts and path classification
- planner pruning still uses `reads`, but apply, projection, and reuse behavior
  also consume `projection_contract`, `equivalence`, `path_class`,
  `maintenance_mode`, `artifact_policy`, and `authority_policy`
- `CanonicalDependencyOrder` is a contract field, not an implementation detail
- authority handling is an explicit contract choice, not an emergent runtime
  habit
- write-path contracts and read-path contracts are duals of the same truth and
  must be declared on types rather than rediscovered at runtime
- framework-owned resources must be declared through contract-bearing
  registration forms rather than scattered coordination calls

Identity representation is also part of the performance contract, not an
incidental storage detail. Performance-critical runtime entities should default
to dense arena-backed generational IDs rather than rich object identity or
pointer-shaped handles:

```rust
pub struct NodeId(u32);
pub struct SegmentId(u32);
pub struct SnapshotId(u32);
```

Normative rule:

- performance-critical runtime domains default to generational arena IDs
- ID width is an architectural decision because cache density, bandwidth, and
  adjacency storage density change with it
- wider or richer identity forms require explicit justification at the phase
  that introduces them

Speculative application is also part of the performance contract. If a node or
path can cheaply reconcile with authoritative truth, the architecture should
prefer reflecting speculative effects immediately rather than waiting for final
authority before showing any state movement.

Normative rule:

- authority policy must be explicit in the contract or resolved path policy
- paths that can speculatively apply and cheaply reconcile should default to
  `SpeculativeThenReconcile`
- `AuthoritativeOnly` is reserved for domains whose semantics or failure mode
  make speculative reflection unacceptable

This is the architectural attachment point for the speculative-then-reconcile
law, and it also reinforces laws `3`, `8`, `20`, `21`, and `34`.

This is the main architectural attachment point for law `36`, and it also
reinforces laws `7`, `19`, `31`, and `33`.

This is the primary encoding for laws `7`, `19`, `20`, `28`, and `29`.

### S9.4 — Canonical Collections and Narrowed Deltas

Hot-path structural operations currently still move too much meaning through raw
lists. That stops here.

The following wrapper types are mandatory for the next architecture rewrite:

```rust
pub struct CanonicalDependencies(SmallVec<[DependencyEdge; 8]>);
pub struct CanonicalChangedRegions(SmallVec<[ChangedRegion; 4]>);
pub struct DedupedNodeBatch(SmallVec<[NodeId; 16]>);
pub struct SortedSourceBatch(SmallVec<[NodeId; 16]>);
pub struct DirtyDelta {
    pub changed_aspects: AspectMask,
    pub changed_regions: CanonicalChangedRegions,
    pub touched_nodes: DedupedNodeBatch,
}
pub struct StructuralDelta { ... }
pub struct DesiredState<T> { ... }
pub struct PatchPlan { ... }
pub struct PendingSnapshotBatch(SmallVec<[PendingDependencySnapshot; 16]>);
pub struct SubscriberRepairBatch(SmallVec<[SubscriberRepair; 16]>);
```

Normative rule:

- topology, snapshot, invalidation, and batch-commit paths must accept these
  wrapper forms rather than raw `Vec`s
- `DirtyDelta` is the only acceptable invalidation input after narrowing
- batch maintenance should consume `PendingSnapshotBatch`,
  `SubscriberRepairBatch`, and `SortedSourceBatch` directly
- when computing desired truth is cheaper than applying it, producers emit
  `DesiredState<T>` and the framework owns `StructuralDelta` and `PatchPlan`
- cross-phase facts such as touched scope, contract masks, and topological
  impact must be derived exactly once at the batch boundary as `*Summary` forms

Relationship storage must also follow traversal shape rather than normalization
shape. Signal is a directional graph runtime, so dependency and subscriber
relationships are stored to serve graph traversal, invalidation, rewiring, and
batch repair directly:

```rust
pub struct DependencyAdjacency { ... }   // node -> dependencies
pub struct SubscriberAdjacency { ... }   // source -> subscribers
pub struct SnapshotAdjacency { ... }     // node -> dependency snapshot
```

Normative rule:

- relationship storage must match dominant traversal pattern
- Signal must not normalize graph relationships as though they were relational
  join tables if the hot path consumes them directionally
- dependency, subscriber, and snapshot storage should optimize for traversal
  locality, batch rewiring, and narrow repair rather than abstract schema
  symmetry

This is the main architectural attachment point for law `37`, and it also
reinforces laws `6`, `10`, `22`, `25`, `30`, and `31`.

This section extends S5 and strengthens S2/S6. It is the main architectural
encoding for laws `1`, `6`, `10`, `22`, `25`, and `26`.

### S9.5 — Lowered Stage Plans as the Only Execution Input

The planner must not hand execution a loosely interpreted bag of tasks. Serial
and parallel execution must consume the same lowered form.

Required architectural forms:

```rust
pub struct LoweredStagePlan {
    pub tasks: Vec<LoweredTask>,
    pub apply_groups: Vec<DisjointApplyGroup>,
    pub dirty_delta: DirtyDelta,
    pub execution_strategy: ResolvedExecutionStrategy,
    pub maintenance_strategy: ResolvedMaintenanceStrategy,
    pub authority_policy: AuthorityPolicy,
    pub decision_summary: DecisionSummary,
}

pub struct LoweredTask {
    pub node: NodeId,
    pub contract: NodeContract,
    pub projection_contract: ProjectionContract,
    pub dependency_inputs: CanonicalDependencies,
    pub rewiring: Option<RewiringPlan>,
    pub path_class: PathClass,
    pub authority_policy: AuthorityPolicy,
}

pub struct DisjointApplyGroup {
    pub tasks: Vec<LoweredTask>,
    pub footprint: ApplyFootprint,
}

pub struct ApplyFootprint {
    pub touched_nodes: DedupedNodeBatch,
    pub touched_sources: SortedSourceBatch,
    pub locality: LocalityFootprint,
}
```

Execution must consume `LoweredStagePlan`, not re-decide:

- strategy
- artifact path class
- maintenance mode
- dependency canonicalization
- rewiring intent
- parallel admission safety
- whether the stage is speculative-first or authority-blocking
- which policy decisions were already resolved and recorded in the decision log

Typestate also applies here. The target architecture is not just “typed data,”
but phase-typed construction:

```rust
pub struct CandidateTask { ... }
pub struct EligibleTask { ... }
pub struct LoweredTask { ... }
pub struct ExecutedTask { ... }
```

Illegal transitions between these forms must be uncallable, not merely checked
after the fact. Invalid operational states are architecture bugs.

This is the strongest protection against serial/parallel drift and late
branching. It extends S5 directly and encodes laws `3`, `8`, `16`, and `21`.

### S9.6 — Operational Effect vs Diagnostic Envelope

S3 currently proposes `EvaluationEffect` as a single effect shape. That is no
longer sufficient.

The architecture now distinguishes:

```rust
pub struct DomainEffect {
    pub primary_result: OperationalEffect,
    pub structural_delta: StructuralDelta,
    pub rollback_effect: RollbackEffect,
}

pub struct OperationalEffect {
    pub node: NodeId,
    pub aspect_version: AspectVersion,
    pub output_change: OutputChange,
    pub dependency_snapshot: DependencySnapshot,
    pub meaningful_input_changes: u32,
    pub verdict: EvaluationVerdict,
}

pub struct SpeculativeEffect {
    pub operational: OperationalEffect,
    pub authority_policy: AuthorityPolicy,
}

pub struct ReconciliationOutcome {
    pub node: NodeId,
    pub confirmed: bool,
    pub adjusted: bool,
    pub rolled_back: bool,
}

pub struct RollbackEffect {
    pub node: NodeId,
    pub undo_patch: PatchPlan,
}

pub struct DiagnosticEnvelope {
    pub changed_regions: CanonicalChangedRegions,
    pub labels: SmallVec<[String; 2]>,
    pub output_identity: Option<OutputIdentity>,
    pub continuity_token: Option<OutputIdentity>,
    pub memoized_origin: MemoizedResultOrigin,
}
```

Normative rule:

- the hot path commits `OperationalEffect`
- if the active authority policy is `SpeculativeThenReconcile`, the hot path may
  commit `SpeculativeEffect` first and reconcile with authoritative truth later
- reconciliation must be cheap, explicit, and structurally separated from rich
  diagnostics
- domain handlers produce declarative `DomainEffect` truth; the framework
  derives rollback, observability routing, and publication from that effect
- rollback must be structurally derivable from `RollbackEffect` and effect
  records alone
- rich diagnostics are retained or reconstructed only if the resolved
  `ArtifactPolicyClass` allows it
- `DiagnosticEnvelope` must not be required for purely operational execution

`EvaluationEffect` may remain as an internal assembly convenience during
transition, but the target architecture is explicit operational/rich path
separation plus explicit speculative/authoritative reconciliation.

This section extends S3 and encodes laws `2`, `15`, and `20`.

### S9.7 — Boundary Envelopes and Decision Logs

Every boundary crossing in Signal must produce a self-describing envelope, and
every authority-path decision must be structurally recorded.

Semantic-purity rule:

- every truth-bearing field and accessor must mean exactly one thing
- ids are truth; labels are display metadata
- derivation, restoration, invalidation, and presentation must not be merged
  into one convenience field or helper
- if a helper collapses more than one ontology, it is an architecture defect,
  not a convenience

Required forms:

```rust
pub struct TransactionResult {
    pub outcome: TransactionOutcome,
    pub warnings: Vec<AdvisoryRecord>,
    pub decision_summary: DecisionSummary,
    pub decision_log: DecisionLog,
    pub integrity_markers: IntegrityMarkers,
    pub performance_accounting: PerformanceCounterSurface<'static>,
}

pub enum DecisionDetail {
    TransactionOutcome { outcome: TransactionOutcome },
    StageAuthorityPolicy { authority_policy: AuthorityPolicy },
    StageParallelAdmission { admission_reason: String },
    Rollback { reason: String },
    Failure { phase: ExecutionFailurePhase, message: String },
}

pub struct DecisionRecord {
    pub stage_index: Option<u32>,
    pub detail: DecisionDetail,
}
```

Normative rule:

- a consumer must be able to reconstruct an operation from its envelope without
  querying producer internals
- conflict resolution, invariant overrides, cascade triggers, and authority
  outcomes must be recorded in `DecisionLog`
- `DecisionSummary` is the batch-derived form that crosses phase boundaries; the
  full log remains queryable and span-aware

This section extends S3 and S13 and encodes laws `7`, `8`, and `32`.

### S9.7.a — Lineage Semantic Purity

Signal lineage is artifact lineage over time within a branched execution
runtime, not execution logging.

Required lineage laws:

- stable artifact identity remains explicit through `LineageArtifactId`
- true derivational parentage remains distinct from restoration reference,
  invalidation reference, and display metadata
- branch ids are truth; branch names are optional presentation only
- invalidation cause must converge toward typed causality, not free-form text
- restore records must stay semantically distinct from recomputation
- UI labels may summarize lineage, but must never define lineage semantics

Normative rule:

- no lineage accessor may return “some related artifact” under a parentage name
- if a record needs multiple artifact relations, those relations must be named
  separately and precisely
- a lineage surface is incomplete if it requires string parsing to recover
  branch identity or invalidation cause

This section extends S5 and S13 and encodes laws `8`, `15`, and `32`.

### S9.8 — Cardinality-Matched API Surface

S7 now has a normative API-shape rule:

> Bulk semantics must cross subsystem boundaries as bulk types.

Required batch-first operational forms:

```rust
pub struct DependencyBatchEdit { ... }
pub struct DirtyBatch { ... }
pub struct SemanticBatchCommit { ... }
pub struct SnapshotBatchCommit { ... }
```

Normative rule:

- scalar orchestration over semantically bulk work is forbidden as a primary API
  surface
- scalar mutation helpers may exist only as low-level implementation detail
- batch/session APIs are the operational contract

This extends S7 directly and encodes laws `6`, `9`, `17`, and `18`.

### S9.9 — Locality and Parallel Disjointness Contracts

Locality and parallel safety must be represented structurally, not inferred
late.

Required architectural forms:

```rust
pub struct LocalityFootprint {
    pub partitions: PartitionScopeSet,
    pub nodes: DedupedNodeBatch,
    pub sources: SortedSourceBatch,
}

pub struct PartitionScopeSet(SmallVec<[PartitionSubscription; 8]>);
```

Normative rule:

- parallel apply admission must depend on `DisjointApplyGroup` and
  `ApplyFootprint`, not only executor policy
- supported grouped-concurrent apply must consume a proof-bearing lowered plan
  and derive worker-local packets before any shared publication step
- stages that would require shared-surface suppression or local rewiring beyond
  the proof-safe concurrent envelope must lower honestly to serial execution
  with an explicit rejection reason instead of keeping a fake `FullParallel`
  execution label
- locality boundaries must travel with lowered plans and repair summaries
- touched or affected scope must be carried as `*Summary` or scope-set forms
  instead of rediscovered later
- relationship storage and apply-group formation must be aligned so the same
  directional adjacency forms support both traversal locality and conflict
  detection instead of forcing normalized reassembly at admission time

This extends S5 and S6 and encodes laws `5`, `21`, `30`, and `33`.

### S9.10 — Allocation Lifetime and Single-Consumer Flow

Lifecycle scope must become visible in the architecture, not only in local
scratch helpers.

Required workspace families:

```rust
pub struct GraphScratch { ... }
pub struct SessionScratch { ... }
pub struct StageScratch { ... }
pub struct TransactionScratch { ... }
```

Normative rule:

- hot-path work must execute within an explicit lifecycle-managed scope
- effect packets and structural batches should be move-oriented and non-`Clone`
  by default
- if a packet is `Clone`, the architecture must justify the second observer of
  the pre-mutation truth
- framework-owned resources such as computations, subscriptions, observers,
  projections, and caches must be registered and disposed through framework
  lifecycle boundaries, not consumer convention

Supporting form:

```rust
pub struct SingleConsumer<T>(T);
```

`SingleConsumer<T>` is not a required literal final name, but the architecture
must provide an equivalent move-only signal for pipeline packets whose cloning
would be structural waste.

This extends S6 and encodes laws `24`, `31`, `32`, and `35`.

### S9.11 — Fast-Exit and Phase-Typed Eligibility

Cheap rejection must happen before expensive construction.

Required pipeline progression:

```rust
pub struct CandidateTask { ... }
pub struct EligibleTask { ... }
pub struct LoweredTask { ... }
pub struct ExecutedTask { ... }
```

Normative rule:

- admission, contract mismatch, path mismatch, and cheap disqualifiers must be
  evaluated before dependency-input assembly, artifact shaping, rewiring
  planning, or apply-group construction
- later pipeline phases may only consume `EligibleTask` or `LoweredTask`, never
  raw candidates

This extends S5 and S6 and encodes law `34`.

### S9.12 — Authority, Derivation, Checkpoints, and Reconstructability

Authoritative truth and derived runtime state are categorically different
objects. Derived state must be reproducible from authority alone.

Required forms:

```rust
pub struct AuthorityState { ... }
pub struct DerivedState { ... }
pub struct CheckpointRecord { ... }
pub struct JournalSegment { ... }
```

Normative rule:

- every derived structure in Signal must be destroyable and rebuildable from
  `AuthorityState` plus a bounded journal since the last checkpoint
- speculative state is never authoritative; it is either confirmed, adjusted,
  or rolled back during reconciliation
- checkpoints and journals are independent subsystems with independent
  lifecycles and must not require write-path suspension to exist

This section extends S3, S6, and S7 and encodes laws `19`, `33`, and `36`.

### S9.13 — Measurement Boundaries Required by Architecture

Because S9 defines all three enforcement layers, the architecture itself must
name the mandatory measurement boundaries and counters that every rewritten path
must expose.

Required architecture-level counters and certification hooks:

- batch width
- dirty delta breadth
- rewiring count
- snapshot batch size
- subscriber repair breadth
- incremental vs rebuild choice
- apply-group width
- apply-group disjointness statistics
- hot-path artifact retention count
- hot-path artifact reconstruction count
- decision-log event count
- checkpoint size and journal replay span
- structural-delta size and patch-application breadth

Normative rule:

- a rewritten path is not complete until these counters exist at the boundary
  the path claims to optimize
- serial and parallel lowered execution paths must emit comparable counters
- performance certification workloads must consume these counters alongside
  elapsed time so claims remain interpretable

This section ties S9 back to the performance baseline and encodes the
architecture-level part of law `13`.

### S9.14 — Concrete Integration with Existing Phases

This addendum modifies earlier phases as follows:

- **S2** becomes the home of `NodeContract`, `EquivalenceContract`,
  `ProjectionContract`, `PathClass`, `MaintenanceMode`, authority policy, and
  artifact-path classification.
- **S3** becomes the home of `DomainEffect`, `OperationalEffect`,
  `DiagnosticEnvelope`, boundary envelopes, decision logs, and effect-application
  capability boundaries.
- **S5** becomes the home of `Canonical*`, `Lowered*`, `Resolved*`, `*Delta`,
  `*Summary`, disjoint apply groups, and adaptive incremental/rebuild choice.
- **S6** becomes the home of lifecycle-scoped scratch types, single-consumer
  packet rules, checkpoints/journals, and phase-typed eligibility progression.
- **S7** becomes the home of batch-first operational APIs and the explicit
  demotion of scalar mutation surfaces plus framework-owned resource lifecycle.

### S9.15 — Branched Runtime Reconciliation and Merge Lineage

Branching is incomplete without a way to reconcile accepted branch-local work
back into an authoritative branch.

Landed closeout status for the supported S9 envelope:

- supported merge planning is now proof-driven through `MergeBoundaryWitness`,
  `StructuralMergeJournalSlice`, `ProofMinimalOverlapBasis`,
  `ConservativeOverlapExpansion`, `PlannedMergeCandidateSet`, and
  `LoweredMergePlan`
- `MergeCandidateScope`, including whole-live supported scope, is retired from
  the supported merge path
- merge executor and merge reporting now consume lowered proof-bearing merge
  packets rather than ambient candidate discovery
- merge counters expose `boundary_witness_kind`, `source_slice_breadth`,
  `proof_minimal_overlap_breadth`,
  `conservative_overlap_expansion_breadth`, `final_candidate_breadth`, and
  `reconciliation_breadth`
- repeated merge, restore-after-merge, and convenience-index churn are
  certified against the bounded merge substrate

Canonical supported result shape:

```rust
pub struct BranchMergeResult {
    pub source_branch: SignalBranchId,
    pub target_branch: SignalBranchId,
    pub boundary_witness: MergeBoundaryWitness,
    pub proof_minimal_overlap: ProofMinimalOverlapBasis,
    pub conservative_overlap: ConservativeOverlapExpansion,
    pub planned_candidates: PlannedMergeCandidateSet,
    pub merge_kind: BranchMergeKind,
}

pub enum BranchMergeKind {
    FastForward,
    Applied,
    ConflictResolved,
}
```

Implementation is intentionally staged inside S9.15 itself. Merge hardening is
not a generic cleanup pass after S9.x; it is part of making merge a real
product capability.

#### S9.15.0 — Merge Substrate Foundation

Required completion:

- source-only node adoption into target authority through explicit merge-time
  introduction semantics
- canonical `BranchMergeExecutionSummary` as the single truth source for replay,
  lineage, and merge reporting
- branch-owned mutation ledger rather than graph-wide rediscovery as the
  primary merge candidate proof surface
- merge candidate scope narrowing from branch-local mutation proof instead of
  unconditional whole-live branch scans
- explicit target identity allocation and dependency remap truth for introduced
  nodes

Normative rule:

- merge substrate is not complete until repeated merges can stay bounded by
  branch-carried proof instead of cumulative whole-branch inspection

#### S9.15.1 — Reconciliation Semantics and Conflict Surfaces

Required completion:

- typed divergence classification between `FastForward`, `Applied`, and true
  conflict-required merge cases
- typed merge failure surfaces instead of generic invalid-input routing
- explicit reconciliation policy for existing-target replacement, source
  adoption, preserved target state, and non-adoptable branch-local work
- truthful conflict boundaries: `ConflictResolved` may exist only when real
  conflict resolution semantics exist

Normative rule:

- planner and executor must agree on merge meaning from lowered typed policy,
  not rediscover it from runtime state at execution time

#### S9.15.2 — Structural Mutation Journal

The branch mutation ledger must evolve from node-granular proof to structural
merge truth.

Required completion:

- first-class branch-local records for:
  - node introduction and eventual branch-local removal semantics
  - dependency edge add/remove
  - dependency snapshot delta
  - authoritative artifact transition
  - merge-relevant scope/region truth where required
- merge planning driven from journal truth rather than graph-state comparison
- boundary advancement semantics so repeated merges stay bounded by "since last
  merge boundary" rather than "anything ever touched on this branch"

Normative rule:

- if merge planning must rescan broad graph state to reconstruct structural
  delta, the mutation journal is incomplete

#### S9.15.3 — Production-Grade Hardening and Certification

Merge is not production-grade until breadth, traceability, and replay
coherence are certified under repeated history evolution.

Required completion:

- repeated-merge boundedness certification
- snapshot/restore coherence across merge histories
- replay/lineage stability across longer branch lifetimes
- performance certification for:
  - candidate-node merge breadth
  - dependency remap breadth
  - subscriber repair breadth
  - merge snapshot capture breadth
- diagnostics-only churn must remain excluded from merge execution breadth
- no accidental whole-graph subscriber rebuilds or branch-wide merge scans may
  remain in hot merge paths except as explicit fallback when no narrower proof
  exists

Normative rule:

- merge is not done when it is merely truthful; it is done when truthful merge
  semantics also remain operationally bounded under geometry-kernel-scale
  workloads

Normative rule:

- `MergedFrom` must not be emitted as a decorative branch event without real
  merge semantics
- merge lineage must eventually include both branch-level reconciliation and
  artifact-level adoption/replacement semantics
- speculative or branch-local parallel work is not complete until accepted work
  can reconcile back into the authoritative branch through an explicit merge
  boundary

This section extends S5 and S9.7.a and encodes laws `7`, `15`, and `32`.

#### S9.15.4 — Conflict Reconciliation Semantics

Conflict reporting is necessary, but it is not enough. Merge becomes a real
product capability only when at least some conflicting shared-state cases can
be reconciled through typed policy rather than blanket rejection.

Required completion:

- add typed conflict-reconciliation plans derived from conflict evidence, not
  host-side interpretation or prose-only diagnostics
- define which conflict families are supported in v1 reconciliation:
  - artifact/runtime-state reconciliation
  - dependency-topology reconciliation
  - dependency-snapshot reconciliation
  - authority/adoptability reconciliation
- emit `ConflictResolved` only from genuinely executed reconciliation behavior
- preserve typed rejection for unsupported conflict families; do not soften
  `RejectSharedStateConflict` into convenience fallback

Completion criteria:

- unsupported conflicts still fail with structured evidence and required
  resolution
- supported conflicts lower into explicit reconciliation plans before mutation
- successful resolved conflicts produce truthful merge lineage distinct from
  ordinary artifact derivation

#### S9.15.5 — Final Merge Certification

Once conflict reconciliation exists, merge must be certified as a stable
product capability under longer histories and hostile branch evolution.

Required completion:

- repeated merge/restore certification across longer branch histories
- breadth and cost regression tests for planning, remap, repair, and conflict
  classification
- proof that failure, success, and restore paths do not fabricate branch-merge
  history
- proof that reconciliation work remains bounded by structural journal truth

Completion criteria:

- repeated histories remain bounded and replay/lineage coherent
- cost-sensitive tests fail if broad scans or broad repairs reappear
- resolved conflicts and rejected conflicts both leave truthful diagnostics

#### S9.15.6 — Scope Decision and Closeout

S9.15 closes only after the remaining merge scope is made explicit rather than
implicitly drifting into future work.

Required completion:

- explicitly mark which merge behaviors are complete in S9.15
- explicitly defer unsupported-but-real merge behaviors to S10
- do not leave persistent-identity matching, per-aspect merge, deletion/removal
  semantics, or richer identity correspondence as implied future cleanup

Completion criteria:

- S9.15 ends with an explicit supported merge envelope
- deferred behaviors are documented as S10 work, not soft TODOs

Closeout statement:

- S9.15 is considered closed for the supported merge envelope because supported
  merge candidate construction is now a pure function of carried proof plus
  proof-authorized indexes, whole-live supported merge scope is no longer
  representable on the supported path, repeated merge and restore preserve
  bounded boundary truth, and convenience subscriber-index rebuilds do not
  change lowered merge candidates

Supported S9.15 merge envelope:

- `FastForward`, `Applied`, and `ConflictResolved` are real runtime outcomes
- source-only adoptable nodes can be introduced into target authority with
  fresh target node identity, remapped dependencies, and merge-specific
  lineage rather than fake derivation ancestry
- merge planning is bounded by branch-owned mutation truth via the branch
  mutation ledger and structural mutation journal; missing bounded proof must
  fail explicitly rather than falling back to branch-wide live scans
- snapshot/restore preserves merge ledger boundaries and does not fabricate
  pending merge delta or false merge history
- failed merges emit typed conflict evidence, typed reconciliation plans,
  failure diagnostics, and failure replay without emitting false
  `BranchMerged` history
- successful conflict-resolved merges retain typed `resolution_plan`
  information in results, execution summaries, replay detail, and lineage
- v1 supported conflict resolution families are:
  - runtime/comparable state reconciliation by adopting source authority when
    structure and merge authority remain within the supported envelope
  - dependency-snapshot reconciliation by adopting the source dependency
    snapshot without fabricating artifact adoption
- v1 unsupported conflict families remain typed rejection:
  - dependency-topology conflicts
  - merge-authority/adoptability conflicts
  - broader three-way or policy-driven structural reconciliation

Deferred to S10:

- persistent-identity node matching across different `NodeId`s
- per-aspect merge into differently shaped or differently allocated target
  nodes
- deletion/removal semantics for nodes, edges, and aspects
- richer edge-level merge result surfaces
- partial-conflict acceptance and broader conflict-resolution policy families

### S9.16 — Geometry-Kernel Performance Hardening Program

`S9.16` assumes the `S9.15` merge substrate is complete enough to support
performance hardening without re-opening merge-truth shortcuts. The next
merge-expansion work after `S9.16` is tracked explicitly under `S10`, not left
as implied future cleanup.

If `worth-signal` is expected to support geometry kernels for next-generation
aircraft, performance hardening must target the real failure modes of that
workload rather than generic “incremental runtime” benchmarks.

Geometry-kernel pressure profile:

- very large dependency graphs
- high fan-out invalidation over localized semantic deltas
- expensive artifacts with large retained payloads
- partitioned or region-scoped outputs whose changed frontier is much smaller
  than the object carrying them
- strong replay, lineage, and explainability requirements under industrial
  debugging and certification pressure

Normative rule:

- no hot-path optimization may trade away reconstructability, transactional
  rollback, semantic locality, or provenance truth
- no diagnostic surface may remain on the operational hot path by accident
- no geometry-kernel performance claim is valid without scale-sensitive
  certification workloads that exercise large artifact graphs and hostile
  branch/restore/replay history

Cross-phase invariants:

| Invariant | Meaning |
| --- | --- |
| Hot paths never require cold richness | Operational execution, planning, merge, invalidation, and reuse must run from hot runtime truth alone |
| Snapshot policy affects richness, not truth | Retention/storage policy may change retained payload and reconstruction capability, never operational semantics |
| Invalidation breadth is bounded by canonical delta | Propagation breadth must come from mutation-time proof, not broad rediscovery |
| Reuse requires explicit equivalence contract | No reuse from ad hoc “close enough” field comparisons |
| Diagnostic tier affects availability, not semantics | Lower tiers may drop richness, never alter meaning |
| Certification uses runtime counters, not log scraping | Performance/correctness proof must consume canonical counters and summaries |

Required workstreams:

### S9.16.1 — Hot/Cold Artifact Separation

Operational state, retained diagnostic artifacts, and historical explainability
must become physically and architecturally separate lanes.

Required target forms:

```rust
pub struct RuntimeArtifactState { ... }
pub struct RetainedDiagnosticArtifact { ... }
pub struct HistoricalArtifactRecord { ... }
```

Semantic role:

- `RuntimeArtifactState` is the canonical operational packet for hot-path
  artifact truth
- `RetainedDiagnosticArtifact` is derived retained artifact carrying optional
  cold diagnostic richness
- `HistoricalArtifactRecord` is a cold assembled history view for explanation,
  lineage expansion, and retained reporting

Normative rule:

- apply, invalidation, and planner hot paths must consume only
  `RuntimeArtifactState`
- retained explanations, lineage expansions, and replay attachments must be
  stored in a cold path that can be degraded by policy
- large geometry artifacts must not be cloned merely to satisfy diagnostics or
  retained history

### S9.16.2 — Structural-Sharing Snapshots and Dependency State

Snapshotting and dependency-state retention must stop scaling with whole owned
payload size when the semantic delta is narrow.

Required target forms:

```rust
pub struct SharedDependencySnapshot { ... }
pub struct SnapshotDeltaRecord { ... }
pub struct ArtifactRetentionPolicy { ... }
```

Semantic role:

- `SharedDependencySnapshot` is storage-oriented shared dependency snapshot
  backing, not a semantic shortcut
- `SnapshotDeltaRecord` is a proof-bearing snapshot delta packet for narrow
  dependency-state change
- `ArtifactRetentionPolicy` is explicit richness-retention policy, not an
  operational truth selector

Normative rule:

- dependency snapshots and snapshot restores must converge toward
  structural-sharing or delta-oriented storage
- restoring a prior artifact identity, rewinding active state, and seeding
  recomputation from prior state must remain distinct semantics even if they
  share storage internals
- shared snapshot backing is a storage strategy only; restore, identity, and
  reuse semantics remain defined by explicit snapshot and dependency contracts,
  not by pointer-sharing or backing reuse
- whole-snapshot clone behavior is acceptable only for compact or explicitly
  bounded profiles, never as the universal geometry-kernel path

### S9.16.3 — Locality-First Invalidation and Frontier Execution

The invalidation engine must evolve from batch-amortized traversal into a true
locality-first multi-source frontier engine.

Required target forms:

```rust
pub(crate) struct AdmittedSourceRecompute { ... }
pub(crate) struct PreparedDirectInvalidation { ... }
pub(crate) struct CommittedDirectInvalidation { ... }
pub(crate) struct AdmittedStructuralRecompute { ... }
pub(crate) struct ResolvedInvalidationWork { ... }
pub(crate) struct LoweredInvalidationBatch { ... }
pub(crate) struct ReadyInvalidationBatch { ... }
pub(crate) struct ExecutedInvalidationBatch { ... }
pub struct InvalidationPlanningEstimate { ... }
pub struct SignalInvalidationExecutionReceipt { ... }
```

Semantic role:

- `AdmittedSourceRecompute` is current persisted/readmitted root work and does
  not claim a producer output commit
- `PreparedDirectInvalidation` is unperformed immediate-edge admission and
  cannot authorize scheduling or execution
- `CommittedDirectInvalidation` exists only after the exact output/cause packet
  is atomically performed
- `AdmittedStructuralRecompute` exists only after performed topology mutation
  establishes a current structural obligation
- `ResolvedInvalidationWork` is the sealed convergence point that retains the
  exact source, dependency, or structural origin
- `LoweredInvalidationBatch` binds committed work to current topology and
  canonical stage/order
- `ReadyInvalidationBatch` is the only executor input
- `ExecutedInvalidationBatch` retains performed execution truth
- `InvalidationPlanningEstimate` is caller-visible predicted evidence only
- `SignalInvalidationExecutionReceipt` is derived realized evidence and cannot
  reenter operational authority

Normative rule:

- the owner-specific progression must use `worth-proof` phase carriers,
  private authority/capability witnesses, binding axes, freshness outcomes,
  typed transition outcomes, and `Performed` at actual effects
- only the performed output-commit owner may promote prepared immediate-edge
  admission into committed direct invalidation truth
- source-recompute and structural-recompute work use separate current-basis
  owner admissions and cannot masquerade as dependency-commit truth
- each committed producer delta may query only a rebuildable producer-local
  aspect/scope reverse-subscription index, then validate the authoritative
  direct edges it returns; further work exists only after another performed
  producer commit
- propagation cost must scale with the realized semantic frontier plus the
  smallest declared indexed candidate/order granule, never the reachable
  descendant closure or total graph size
- the reverse index is derived from dependency topology, updated with topology
  mutation, rebuildable after restore, and incapable of minting causes
- aspect and partition rejection occurs before dirty mutation and ready enqueue
- ready work is process-local derived state, is not checkpoint authority, and
  must be rebuilt/readmitted from M12 authority after restore or rebind
- Signal owns operational causes, work, effects, and observed counters;
  `worth-foundational` owns canonical case/report identity and counter-backed
  descriptive evidence only
- predicted and realized counters are different artifacts; only performed
  realized rows can satisfy a counter-backed execution receipt
- reachability-shaped `FrontierPlan`/wave constructors are removed from public
  integration facades; compatibility views, if required, are descriptive-only
  and cannot satisfy operational or performed-evidence bounds

`S9.16.3` implementation status after Milestone 13:

- Milestone 12 is complete: root intent, performed per-aspect/per-scope output
  deltas, consumer-specific causes, exact binding axes, pending precedence,
  comparator separation, persistence/readmission, and the financial semantic
  courtroom are accepted
- structural transitive summaries are aspect-free and scope-free, so
  reachability cannot mint descendant meaning
- ordinary routing queries the rebuildable `ProducerAspectKey` reverse index
  and its unscoped/partition/detail buckets, validates every candidate against
  current edge and snapshot authority, and creates further work only after the
  immediate producer performs another output commit
- source, dependency-commit, and structural origins retain distinct current
  bases before converging through resolved, lowered, ready, and executed Proof
  phases; graph/revision/origin/epoch/stage drift is denied before effects
- ready queues and work bindings remain derived process-local state and are
  rebuilt from readmitted M12 source/cause authority after restore
- public integration exposes predicted `InvalidationPlanningEstimate` and
  performed `SignalInvalidationExecutionReceipt` rather than operational
  reachability-plan/wave constructors
- Signal records the realized 24-row counter contract and successful executed
  work bindings; Foundational attaches counter-backed descriptive evidence and
  canonicalizes bounded financial case/report identity without acquiring
  operational authority
- the six-family financial locality courtroom seals independent
  `Q/C/K/U/E/S/P`, exact structural slopes, lifecycle evidence, and same-work-
  stream serial/parallel strategy evidence at the declared ordinary and
  scheduled scale contracts
- cycle preflight, M12 direct-cause atomicity, deterministic stage order,
  condition/async pending precedence, trace policy separation, and branch/
  restore/replay truth remain inherited guarantees that Milestone 13 preserves

The numbered repair and certification sequence is:

1. [Milestone 12 - Aspect-Causal Invalidation](./milestone-12-plan.md) separates
   unresolved root recompute intent from atomically committed per-aspect/per-
   scope producer deltas, binds resolved causes to the immediate dependency and
   its logical revision, establishes canonical pending cause storage, expands
   the fintech financial world, and seals semantic equivalence evidence during
   implementation.
2. [Milestone 13 - Locality-First Frontier Execution](./milestone-13-plan.md)
   replaces reachable-closure walking with direct-hop semantic admission and a
   replaceable ready-work scheduling boundary, then seals financial cost-slope
   and strategy-readiness evidence during implementation.

Milestones 12 and 13 jointly close the invalidation portion of `S9.16.3`:
Milestone 12 owns semantic causality and Milestone 13 owns locality, phase
progression, and realized cost evidence. Later parallel work may consume only
the sealed canonical work stream and measured envelopes; it may not reinterpret
aspect, scope, cause, or readiness authority.

### S9.16.4 — Geometry-Scale Equivalence and Reuse Contracts

Reuse is mandatory for geometry scale, but it must stay truth-grade.

Required target forms:

```rust
pub struct ArtifactEquivalenceContract { ... }
pub struct ReuseBoundaryContext { ... }
pub struct ReuseBasis { ... }
pub enum ReuseOrigin { ... }
pub struct ReuseCertificationRecord { ... }
```

Semantic role:

- `ArtifactEquivalenceContract` is the explicit equivalence contract that
  defines when artifact reuse is semantically legal
- `ReuseBoundaryContext` is the structured runtime evidence packet used to
  evaluate legality at the decision boundary
- `ReuseBasis` is the lowered compact hot-path admission basis derived from
  declared boundaries, not a second semantic owner
- `ReuseOrigin` is the realized runtime outcome and must distinguish fresh
  compute, suppression, memoized reuse, snapshot restore, reconciliation,
  cross-identity persistent reuse, and partial artifact splice
- `ReuseCertificationRecord` is the cold certification record explaining why
  reuse was valid

Normative rule:

- expensive artifact reuse, suppression, memoization, and cache hits must be
  justified by explicit artifact equivalence contracts
- legality is planner/prepared-stage truth; apply/execution may realize or
  reject an admitted strategy, but it must not rediscover legality from broad
  runtime state
- comparator/output equivalence may support suppression semantics, but it must
  never independently authorize artifact reuse
- cross-identity reuse requires explicit persistent correspondence evidence and
  remains distinct from lineage continuity or merge identity
- partial artifact splicing is composition semantics with explicit region-basis
  legality and mixed-provenance lineage, not fake whole-artifact reuse
- geometry kernels must be able to certify that reused artifacts did not cross
  invalid semantic boundaries such as topology regime, tolerance regime, or
  semantic region identity

### S9.16.5 — Diagnostic Tiering Without Semantic Drift

Diagnostics must be tiered by policy without changing the operational meaning
of the run.

Required target forms:

```rust
pub struct DiagnosticsTier { ... }
pub struct RetentionBudget { ... }
pub struct ReconstructionBudget { ... }
pub enum DiagnosticsAvailability { ... }
```

Semantic role:

- `DiagnosticsTier` is the access-lane and richness-class contract, not a
  semantic mode switch
- `RetentionBudget` is the eager retained envelope for bounded history, replay,
  and retained artifact detail
- `ReconstructionBudget` is the explicit cold-work allowance for explanation,
  provenance, and deep replay/history reconstruction
- `DiagnosticsAvailability` is the typed answer surface for retained,
  reconstructed, omitted, denied, and unavailable detail

Tier invariant:

- operational, development, and forensic tiers may differ in retained richness,
  retained depth, and cold reconstruction allowance
- they must remain equal in canonical runtime outcome, reuse/invalidation
  classification, lineage relation meaning, and replay/history conclusion sets
- retained envelopes bound observability cost and richness; they do not define
  alternate runtime truths

Access-lane rules:

- ordinary access returns retained bounded summaries only and performs zero cold
  reconstruction
- retained forensic access may expose richer retained evidence without becoming
  a second semantic authority
- explicit cold materialization must remain named, budgeted, and observable in
  counters

Required observability counters:

- explicit cold materialization requests
- retained forensic reads
- cold explanation reconstructions
- cold provenance reconstructions
- retained artifact reads
- reconstructed artifact reads
- denied reconstruction by tier
- denied reconstruction by budget
- denied reconstruction by API family

Normative rule:

- operational, development, and forensic tiers may differ in retained
  richness but must not differ in execution semantics
- a lower diagnostics tier may reconstruct less history, but it must not report
  different truth for the same boundary envelope or lineage event
- diagnostics tier reduction may remove retained richness, but must not
  introduce hidden broad reconstruction as a default access path for ordinary
  operational or observational queries
- reconstruction work must be budgeted explicitly rather than happening as
  hidden lazy cost on first access

### S9.16.6 — Integrated Invalidation Certification And Performance Proof

The runtime must earn invalidation credibility with hostile financial
scenarios during implementation, not with a later certification milestone or
microbenchmark optimism. Geometry remains a future consumer and requires its
own domain evidence before geometry readiness can be claimed.

Required certification families:

- quote-to-price-to-risk aspect translation with matched/unmatched filters
- heterogeneous exact, tolerance, and installed comparator consumers of one
  producer delta, including policy-separation and legacy-upgrade twins
- tolerance-suppressed and genuinely changed repricing twins
- producer-local market-factor aspect collisions
- rates/credit partition and detail locality with multiple committed scopes
  accumulated while a consumer is gated
- condition-gated repricing and dynamic instrument dependency rewiring,
  including same-shaped edge recreation under a new dependency revision
- sparse, medium, and dense portfolio frontiers
- overlapping price, FX, curve, and volatility shocks
- branch/restore/replay histories with financial-truth equivalence checks

Mandatory measurement boundaries:

- independent fresh financial recompute equivalence
- independent financial necessity manifest agreement
- invalidation frontier width and narrowed frontier width
- edges examined and candidates rejected before enqueue
- ready work enqueued, popped, and deduplicated
- nodes evaluated, changed outputs, and suppression stops
- non-semantic visits, topology-churn work, and peak/retained memory
- replay suffix cost by checkpoint span
- branch restore and reconciliation breadth

Numbered implementation owners:

- [Milestone 12 - Aspect-Causal Invalidation](./milestone-12-plan.md) owns the
  authentic immutable financial-world definition, causally complete baseline,
  named semantic financial scenarios, fresh-recompute and necessity oracles,
  financial equivalence reports, and
  `FinancialAspectCausalityCertificationRun`
- [Milestone 13 - Locality-First Frontier Execution](./milestone-13-plan.md)
  owns the named scale/locality financial scenarios, structural cost slopes,
  Foundational counter-backed receipts, same-work-stream traversal comparisons,
  `FinancialFrontierLocalityCertificationRun`, and typed strategy decision
- Phase 1 must first establish the authentic financial baseline, independent
  oracle cores, and named inherited red control; after a phase
  cuts over the authority exercised by an assigned financial scenario, neither
  milestone may close that phase with the scenario red or absent
- both milestones explicitly exclude tree-only order-maintenance assumptions
  and do not pre-authorize a priority queue or other traversal implementation

Milestone 12 implementation state:

- `ChangeBatchAdmission` is root recompute admission, not output-commit truth;
  deprecated commit-shaped aliases lower to that same admission
- producer output equivalence is decided once at canonical output-commit
  preparation; consumer dependency comparison is a separate admission policy
- committed dependency causes retain producer, consumer, aspect, correlated
  scope, dependency revision, snapshot version, output ordinal, committed
  version, graph instance, and edge identity through persistence/readmission
- direct root bases and dependency cause sets are distinct persisted
  authorities; derived dirty caches are rebuilt from them and cannot mint them
- immediate pending causes resolve before ordinary, temporal, on-demand,
  custom, installed, or async condition admission
- structural transitive reachability carries pending-revalidation posture only;
  its plan and execution summaries contain no copied root aspect or root scope
- the eight-scenario financial courtroom seals one
  `FinancialAspectCausalityCertificationRun`; it remains test-only and does not
  put financial vocabulary on the domain-neutral facade

Execution order:

1. expand the production-shaped financial world and independent oracles
2. implement and certify aspect causality phase by phase
3. implement and certify locality/cost phase by phase
4. seal the semantic and locality runs before parallel execution begins

Anti-goals:

- do not hide broad artifact clones behind ergonomic getters
- do not make forensic richness the default operational cost
- do not claim geometry readiness from elapsed-time wins alone
- do not claim geometry readiness from financial-world evidence alone
- do not introduce compatibility shims that preserve cost-dishonest surfaces
- do not let bridge or host integration become the only place where artifact
  causality and reuse truth can be recovered

This section extends S5, S6, S9.6, S9.9, S9.10, S9.12, and S9.13 and encodes
laws `1`, `2`, `5`, `7`, `10`, `12`, `20`, `21`, `24`, `27`, `28`, `29`,
`32`, `35`, and `36`.

This is mandatory. The new structural rules are not advisory preferences.

### S9.17 — Deterministic And Portable Parallel Execution

`worth-signal` must be able to exploit all causally independent work without
making physical scheduling, worker count, platform, or consumer domain part of
signal meaning.

The numbered implementation sequence is:

1. [Milestone 14 - Deterministic Parallel Execution Foundation](./milestone-14-plan.md)
2. [Milestone 15 - Proof-Carrying Graph Parallelism](./milestone-15-plan.md)
3. [Milestone 16 - Structured Partitioned Parallelism](./milestone-16-plan.md)
4. [Milestone 17 - Portable Execution Backends And Distributed Coordination](./milestone-17-plan.md)

The sequence follows Milestones 12-13 because parallel execution may not
amplify or hide a causally false or breadth-dishonest invalidation frontier.

Cross-phase invariants:

| Invariant | Meaning |
| --- | --- |
| Parallelism never grants semantic safety | Callers and backends cannot assert disjointness, readiness, or control-order legality |
| One lowered meaning | Serial, native parallel, WASM-worker, accelerator, and remote execution consume the same semantic plan |
| One resource authority | Graph and nested partition work subdivide one bounded lease and cannot create independent capacity |
| Workers are non-authoritative | Workers consume immutable inputs and return local packets; graph truth changes only through canonical publication |
| Determinism is explicit | Bitwise, contract-equivalent, and relaxed execution remain distinct contracts and cannot be silently weakened |
| Portability does not erase boundaries | Worker, device, process, and network crossings expose capability, transfer, failure, cancellation, and recovery |
| Domain neutrality is structural | Geometry, imaging, simulation, finance, and other consumers depend on generic partition/execution contracts; they do not enter core meaning |

#### S9.17.1 — Resource Authority, Determinism, And Publication

Parallel execution must be admitted through a runtime-owned resource authority.
The configured worker budget must become a strict hierarchical lease, not a
chunking hint. Nested work subdivides that lease. It may not allocate another
pool or exceed the parent's concurrency, memory, deadline, or cancellation
envelope.

Required target forms:

```rust
pub struct ExecutionRequestPolicy { ... }
pub struct ResolvedExecutionCapabilities { ... }
pub struct ExecutionResourceLease { ... }
pub enum DeterminismContract { ... }
pub struct PreparedExecutionBatch { ... }
pub struct CanonicalPublicationPlan { ... }
pub struct ExecutionOutcomeEnvelope { ... }
```

Normative rules:

- worker-count, memory, queue, and nested-concurrency bounds are enforced and
  reported at runtime
- task-stealing completion order never becomes canonical publication order
- worker-local code cannot mutate authoritative graph state
- cancellation and timeout expose exact progress and do not claim rollback
- unsupported or unprofitable parallel work resolves before dispatch through
  an explicit plan outcome
- a target with no parallel capability executes the same plan serially when
  policy permits, without semantic drift

Milestone 14 owns this subsection.

#### S9.17.2 — Proof-Carrying Graph Parallelism

Graph concurrency requires three independent proofs:

1. settled dependency versions
2. control-order safety in the presence of dynamic rewiring
3. disjoint mutation footprints or an explicit deterministic reconciliation

A topological stage or lock is not a substitute for those proofs.

Required target forms:

```rust
pub struct SettledDependencySet { ... }
pub struct ControlOrderProof { ... }
pub struct GraphMutationFootprint { ... }
pub struct DisjointGraphBatch { ... }
pub struct OrderedConflictPartition { ... }
pub struct DependencyRewriteProposal { ... }
pub struct GraphEpochPublication { ... }
```

Normative rules:

- Milestone 13's causality owner admits invalidation edges before parallel
  scheduling; parallel mechanics cannot reinterpret aspects or partitions
- rewiring is worker-local proposal until cycle, subscription, and readiness
  reconciliation succeeds
- later work whose legality could change because of a proposed edge waits for
  the next epoch unless narrower independence is proved
- every synchronous graph, snapshot, subscription, lineage, observation, and
  publication surface participates in conflict footprints
- queues and unpublished packets remain bounded by the resource lease
- graph-parallel and serial histories must be differentially equivalent under
  adversarial schedule perturbation

Milestone 15 owns this subsection.

#### S9.17.3 — Structured Partitioned Parallelism

Computation authors need domain-neutral structured patterns for work inside one
node. The foundational family is map, reduce, scan, fork/join, and
bulk-synchronous iterative rounds.

Required target forms:

```rust
pub struct StableWorkPartition { ... }
pub struct PartitionReadSet { ... }
pub struct PartitionWriteSet { ... }
pub struct PartitionComputation<I, O> { ... }
pub struct DisjointPartitionBatch { ... }
pub struct DeterministicReductionPlan { ... }
pub struct ScanPlan { ... }
pub struct SynchronousRoundPlan { ... }
```

Normative rules:

- declarations describe semantic partition identity, access, reduction, and
  convergence; the framework alone mints execution authority
- worker index, completion order, raw spawning, and concrete backend types are
  not computation meaning
- deterministic reductions fix partition and join order where required;
  floating-point associativity is never inferred
- scans preserve declared order and are not encoded as unordered reduction
- iterative rounds publish one complete round image before the next reads it
- partition and graph work share one hierarchical lease
- serial-only platforms execute the same declaration and proof topology

Geometry is a future consumer and adversarial scale workload, not an API or
module axis. Milestone 16 owns this subsection.

#### S9.17.4 — Portable Backends And Distributed Coordination

Portable backends consume only versioned prepared work. Capability negotiation
must precede expensive transfer, and returned results are untrusted derived
input until core readmission validates plan, version, epoch, integrity,
determinism, footprint, and idempotency identity.

Required target forms:

```rust
pub struct BackendCapabilityDescriptor { ... }
pub struct BackendRequirementSet { ... }
pub struct PortableComputationIdentity { ... }
pub struct PreparedBackendBatch { ... }
pub struct BackendSubmissionEnvelope { ... }
pub struct BackendResultEnvelope { ... }
pub struct BackendResultReadmission { ... }
pub struct RemoteExecutionRecoveryHandle { ... }
```

Normative rules:

- native pools, browser workers, device runtimes, process transports, and
  network transports are adapters behind the core backend port
- closures, callbacks, pointers, graph handles, credentials, ambient context,
  and runtime authority cannot cross the portable boundary
- remote work carries stable idempotency and input-epoch identity; duplicate,
  delayed, stale, corrupt, or incompatible results cannot publish
- indeterminate remote execution has a recovery handle and never claims
  rollback
- remote executors compute derived packets and do not become authoritative
  graph replicas
- `worth-signal-wasm` preserves its existing worker placement, readmission,
  worker-first posture, and explicit fallback authority
- accelerator support is claimed only after a real adapter passes numerical,
  memory, cancellation, transfer, failure, and semantic conformance

Milestone 17 owns this subsection.

#### S9.17.5 — Certification And Performance Truth

Parallel performance claims must report both semantic proof and the physical
cost that limits speedup.

Mandatory measurement boundaries include:

- total work, span, and critical-path depth
- active workers versus leased workers
- queue width, steals, barriers, and conflict partitions
- worker-local packet and canonical publication breadth
- nested lease breadth and oversubscription denials
- partitions, reductions, scans, rounds, and synchronization depth
- transient, peak, retained, transferred, and resident bytes
- serialization, device, process, and network transfer bytes and round trips
- cancellation points, retries, duplicate results, recovery actions, and
  fallback reasons
- workload distribution, scale axes, runtime/backend versions, hardware,
  cold/warm posture, repetitions, variance, and percentiles

No elapsed-time result may compensate for semantic drift, resource-bound
violation, hidden serial work, scope leakage, or missing external-boundary
evidence. Serial equivalence remains an independent correctness oracle; it is
not itself a speedup claim.

This section extends S4, S5, S6, S9.5, S9.9, S9.10, S9.13, and S9.16 and
encodes architectural laws `4`, `7`, `15`, `16`, `17`, `21`, and `22` plus the
parallel, boundary, allocation, and measurement performance laws.

### S10 — Merge-Forward Expansion

Once `S9.15` closes and `S9.16` no longer requires merge-substrate churn, the
next merge-expansion work must be tracked explicitly rather than rediscovered ad
hoc.

The main unsupported-but-real product behaviors are:

- persistent-identity node matching across different `NodeId`s, so merge can
  reconcile logically stable nodes that were reallocated or independently
  introduced on different branches
- per-aspect merge semantics for multi-aspect nodes, including cases where one
  branch carries richer aspect authority than the other
- true conflict resolution policies, not just conflict classification and typed
  failure
- three-way structural reconciliation over node state, dependency topology, and
  artifact state
- first-class deletion/removal semantics for nodes, edges, and eventually
  aspects
- edge-level merge results and lineage, not only node-centered merge summaries
- partial-conflict acceptance where non-conflicting regions can reconcile
  without pretending the conflicting region was resolved
- typed merge policies such as target-wins/source-wins or
  topology-vs-artifact resolution modes, but only once the semantics are real
  rather than convenience shims
- keyed/persistent-name based identity mapping, if and only if it is promoted
  into real merge identity truth rather than display metadata
- richer historical/explain query surfaces for why a merge produced the final
  target shape

Normative rule:

- none of these may be implemented by heuristic name matching, string labels,
  or compatibility adapters that collapse semantic identity, artifact
  derivation, and branch reconciliation into one surface

#### Domain-Agnostic Configurability Requirement

`worth-signal` is a domain-agnostic runtime. Geometry kernels, web
applications, financial engines, and chip simulators must all be first-class
merge consumers. No S10 feature may hardcode a single domain's merge strategy
into the runtime substrate.

Governing design rule:

- every S10 merge behavior that could reasonably differ across application
  domains must be host-configurable through explicit contract or schema
  registration, frozen at construction time, semantically versioned, and
  lowered into executable runtime form before merge execution begins

This is the same registration discipline already used for custom invariants,
custom evaluation contracts, and artifact equivalence contracts. S10 extends
that discipline to merge.

#### Required configurability surfaces

Identity matching strategy:

- identity matching must not use a single hardcoded resolution algorithm
- the host must declare an identity matching strategy per node contract or per
  schema scope that specifies which identity bases apply (structural
  fingerprint, persistent name, lineage identity, storage identity,
  host-declared correspondence) and in what priority order
- ambiguity policy (reject, prefer first match, require unanimous) must be
  host-declared, not runtime-defaulted
- the identity matching strategy must be frozen at construction and recorded in
  merge planning artifacts for replay determinism

Conflict resolution policy:

- conflict resolution policies must be declared per contract or per aspect
  through the schema registration surface, not supplied as ad hoc closures or
  ambient runtime configuration at merge time
- the runtime must ship a classified set of built-in resolution strategies
  (fail-on-conflict, source-wins, target-wins, last-writer-wins with causal
  evidence, additive-set, monotonic-counter, prefer-richer-structure) as the
  standard policy vocabulary
- host-registered custom conflict resolution policies must follow the same
  semantic versioning and freeze-at-construction rules as custom invariants
- resolution policy identity must be recorded in merge artifacts so replay and
  certification can verify that the same policy was applied

Merge-base selection:

- merge-base selection must be a named, pluggable strategy rather than a single
  hardcoded algorithm
- the default `MaxCommitIdCommonAncestor` must be one variant of an explicit
  strategy surface, not the only path
- custom merge-base strategies must be host-registrable and frozen at
  construction

Per-aspect merge semantics:

- aspect-level merge behavior must be declared per aspect through the schema or
  node contract surface
- the runtime must not assume all aspects on a node merge under the same policy
- aspect merge declarations must lower into typed executable policy forms
  before merge execution, not resolve dynamically at merge time

Deletion and removal semantics:

- deletion classification (tombstone, hard-delete, soft-retire, orphan-cascade)
  must be host-declared per node contract or schema scope
- the runtime must not assume a single deletion model across all domains
- deletion policy must be recorded in merge artifacts

Partial-conflict region isolation:

- the granularity of conflict isolation (per-node, per-aspect, per-subgraph,
  per-host-declared-region) must be configurable through the contract surface
- the runtime must not hardcode a single conflict isolation boundary

#### Required anti-patterns

The following implementation patterns are explicitly prohibited in S10:

- hardcoding identity matching to storage `NodeId` equality as the only
  supported path, with "custom matching" as a future TODO
- implementing conflict resolution as `match` arms inside the merge executor
  with no host-extensibility surface
- treating merge-base selection as an internal implementation detail rather than
  a named, observable, replaceable strategy
- implementing per-aspect merge by checking aspect names against a hardcoded
  list of "known mergeable aspects"
- implementing deletion semantics as a single boolean tombstone flag with no
  host-declared policy
- implementing partial-conflict acceptance with a hardcoded per-node isolation
  boundary and no contract-level configurability

#### Registration and lowering pattern

All S10 configurable behaviors must follow this lifecycle:

1. **declare** at schema or contract registration time through a typed
   declaration surface
2. **freeze** at runtime construction; no merge-time mutation of policy
3. **lower** into executable form before merge planning begins
4. **record** in canonical merge planning artifacts for replay determinism
5. **version** semantically so durable recovery and replay can detect policy
   drift

This is not an optional extension surface. If a future implementer ships an S10
feature without the corresponding host-configurable registration and lowering
path, the feature is incomplete regardless of whether it works for one domain

### Files Modified

| File | Change |
|---|---|
| [signal_architecture2.md](./signal_architecture2.md) | Append S9 addendum; extend S2/S3/S5/S6/S7 by reference |

## What Must Be Preserved

| Pattern | Current Location | Must Remain |
|---|---|---|
| `transition_clean/dirty/maybe_stale` | `NodeEntry` | Canonical state machine, used in 5+ call sites |
| `PartitionScoped` + `scopes_overlap` | `output.rs` | Single partition-matching semantic |
| `DependencySnapshotEntry` + `DependencySortKey` | `dependency.rs` | Named fields, canonical sort key |
| `with_scratch` closure | `SignalGraph` | RAII scratch lease, no manual acquire/restore |
| `reconcile_dependencies` | `EdgeTopology` (after S1) | Declarative dependency management |
| `InvalidationTraversal` pipeline | `routing.rs` | Decomposed invalidation phases |
| `SegmentedStore<T, Id>` | `segmented.rs` | Unified edge store |
| Telemetry sub-structs | `telemetry.rs` | 7 domain-grouped counters |
| `replay_where` | `replay.rs` | Filter consolidation |
| `SparsePatchBuffer` rollback | `patch_buffer.rs` | Transaction undo mechanism |
| Branch capture/restore symmetry | `runtime_state.rs` | Fork/join on branch state |
| `CheckpointRuntime` separation | `checkpoint/` | Checkpoint stays independent of graph |

---

## Sequencing

### Dependency Rules

```
S1 (subsystem split) → unlocks S3, S5, S6
S2 (contracts) → unlocks S5.1 (contract-driven pruning), S8.1 (ambient context)
S3 (declarative effects) → unlocks S4 (transaction result envelope), S3.5 (defineComputation)
S6.1 (partition versions) → should precede S5 (pipeline perf)
S6.5 (error hierarchy) → should precede S6.6 (builder completeness)
S7 (facade) → after S1–S4 stabilize
S8 (context) → after S2.4 (context in contracts) and S3.5 (defineComputation)
S9 (performance enforcement addendum) → extends S2/S3/S5/S6/S7 and should be
written before any V2.1 rewrite work begins
S9.16.3 causal/local invalidation repair and S9.16.6 certification
  -> precede S9.17 parallel-execution expansion
S9.17.1 resource/determinism foundation -> precedes S9.17.2 graph parallelism
S9.17.2 graph parallelism -> precedes S9.17.3 nested partition parallelism
S9.17.3 structured partition proof -> precedes S9.17.4 portable backends
```

### Recommended Execution Order

```text
Batch 1 — Structural Foundation
  S1.1  SignalGraph subsystem split (NodeArena, EdgeTopology, etc.)
  S1.2  BranchManager extraction from SignalRuntime
  S6.5  Typed error hierarchy (enables match-based error handling throughout)

Batch 2 — Effect Pipeline
  S3.1  EvaluationEffect struct
  S3.2  apply_effect pipeline (replaces result_apply monolith)
  S3.3  Commit ceremony extraction (fail_and_rollback)
  S3.4  EvaluationVerdict (three-state outcome enum)

Batch 3 — Contract System
  S2.1  NodeContract trait + NodeBuilder integration
  S2.2  Contract duality documentation
  S2.3  Aspect-aware planner pruning
  S2.4  Context-type in contracts (ContextRequirement)

Batch 4 — Correctness
  S6.1  PartitionVersionMap (fixes over-evaluation bug)
  S6.3  EdgeTopology bidirectional consistency assertion
  S6.4  Edge store rollback tracking
  S6.6  Builder completeness (typestate on SignalRuntimeBuilder)

Batch 5 — Transaction Surface
  S4.1  TransactionResult envelope
  S4.2  SemanticDelta consolidation (named replay entries)

Batch 6 — Computation Model
  S8.1  Ambient evaluation context (Ctx threaded to evaluation closures)
  S8.2  Multi-context design decision (Option A: homogeneous per runtime)
  S3.5  defineComputation pattern (ComputationSpec)
  S5.5  Execution path collapse (merge commit/on-demand paths)

Batch 7 — Performance
  S5.1  Contract-driven plan pruning
  S5.2  EvaluationSession as primary path (zero-alloc completion)
  S5.3  Execution pipeline subsystem decomposition + cfg isolation
  S5.4  Subsystem-scoped amortized GC

Batch 8 — Safety & Surface
  S6.2  GraphObserver (phase-typed observation)
  S7.1  Grouped facade namespaces
  S7.2  State-derived evaluation strategy
  S8.3  Context-scoped evaluation (framework-owned context lifetime)

Batch 9 — Performance Enforcement Rewrite Layer
  S9.1  Performance enforcement model
  S9.2  Proof-bearing pipeline forms
  S9.3  Performance-expanded NodeContract
  S9.4  Canonical collections and narrowed deltas
  S9.5  LoweredStagePlan as the only execution input
  S9.6  OperationalEffect / DiagnosticEnvelope split
  S9.7  Boundary envelopes and decision logs
  S9.8  Cardinality-matched API surface
  S9.9  Locality and disjoint parallel apply contracts
  S9.10 Allocation lifetime scopes and single-consumer packets
  S9.11 Fast-exit and phase-typed eligibility
  S9.12 Authority, derivation, checkpoints, and reconstructability
  S9.13 Architecture-mandated measurement boundaries
  S9.15 Branched runtime reconciliation and merge lineage
  S9.16 Geometry-kernel performance hardening program
  S9.17 Deterministic and portable parallel execution
```

### Practical Rule

The same rule as the relational doc: if there is tension between "clean every layer" and "reach performance work quickly," do the minimum structural work that prevents performance work from encoding bad invariants. That means:

- Do **not** start S5 before S1 (subsystem split prevents false borrow conflicts in execution decomposition)
- Do **not** start S5 before S6.1 (don't optimize a pipeline that over-evaluates)
- Do **not** start S7 before S1–S4 (API surface should reflect stabilized internals)
- **Do** start S3 immediately after S1 (effect pipeline is high-impact, low-risk)

> [!IMPORTANT]
> Run `cargo test -p worth-signal` after **every individual refactor**, same as V1. A regression caught early maps to exactly one change.

---

## Summary

| Phase | Items | Key Pattern Source |
|---|---|---|
| S1 | Subsystem decomposition | Relational C1 (god struct split) |
| S2 | NodeContract + contract duality + context requirements | Relational F2 + D4 · Frontend `ProjectContextService` |
| S3 | EvaluationEffect + verdicts + `defineComputation` | Relational B5 + D5 · Frontend `defineCrudResource` |
| S4 | TransactionResult envelope | Relational E1/E2 (commit result envelope) |
| S5 | Pipeline + performance + path collapse | Relational D4 · Frontend component collapse |
| S6 | Safety + error hierarchy + builder completeness | Relational A3 + C3 + C4 |
| S7 | Facade + state-derived strategy | Relational F1 + D6 |
| S8 | Context-aware computation (ambient, multi-mode) | Frontend `ProjectContextService` / operations mode |
| S9 | Performance enforcement addendum (proof-bearing forms, lowered execution, bulk-first API shape) | Relational performance enforcement model + proof-bearing pipeline forms |


