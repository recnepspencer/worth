# worth-signal Refactor Engineering Spec

> **Status:** Pre-production. No public consumers. All changes are breaking-change-safe.
>
> **Scope:** Systematic structural overhaul of the `worth-signal` crate to eliminate duplication, enforce invariants at compile time, and create composable primitives that make wrong code unwritable.

---

## Table of Contents

1. [R1: `NodeEntry` State Machine Transitions](#r1-nodeentry-state-machine-transitions)
2. [R2: `PartitionScoped` Trait for Scope Matching](#r2-partitionscoped-trait-for-scope-matching)
3. [R3: `DependencyKey` and Named Snapshot Entries](#r3-dependencykey-and-named-snapshot-entries)
4. [R4: `ScratchGuard` RAII Lease](#r4-scratchguard-raii-lease)
5. [R5: Edge Mutation Ceremony Extraction](#r5-edge-mutation-ceremony-extraction)
6. [R6: Declarative Dependency Reconciliation](#r6-declarative-dependency-reconciliation)
7. [R7: Invalidation Pass Pipeline](#r7-invalidation-pass-pipeline)
8. [**Phase 2**: R8: Zero-Allocation Planner & Prepared Cursor](#r8-zero-allocation-planner--prepared-cursor)
9. [**Phase 2**: R9: Feature-Gated Pipeline Execution Engine](#r9-feature-gated-pipeline-execution-engine)
10. [**Phase 2**: R10: Epoch-Driven Amortized Garbage Collection](#r10-epoch-driven-amortized-garbage-collection)
11. [R11: Unified `SegmentedStore<T, Id>`](#r11-unified-segmentedstoret-id)
12. [R12: Telemetry Sub-Struct Decomposition](#r12-telemetry-sub-struct-decomposition)
13. [R13: Diagnostics Replay Filter Consolidation](#r13-diagnostics-replay-filter-consolidation)
14. [R14: Stale Utility Deduplication](#r14-stale-utility-deduplication)
15. [**Phase 2**: R15: Partition-Aware MaybeStale Validation](#r15-partition-aware-maybestale-validation)
16. [R19: `PhaseGuard<P>` â€” Cross-Epoch Re-entrancy Prevention](#r19-phaseguardp--cross-epoch-re-entrancy-prevention)
17. [R20: Observation Purity â€” `&self` Diagnostic Enforcement](#r20-observation-purity--self-diagnostic-enforcement)
18. [R21: Single Source of Truth â€” Representational Drift Prevention](#r21-single-source-of-truth--representational-drift-prevention)
19. [R22: Transactional Mutation â€” Rollback Amnesia Prevention](#r22-transactional-mutation--rollback-amnesia-prevention)
20. [Meta-Abstractions](#meta-abstractions)
21. [**Phase 3**: R16â€“R18: Compile-Time Safety](./signal_compile_time_safety.md) *(separate document)*
22. [**Deferred**: Bug Classes & Compile-Time Enforcement](./signal_compile_time_safety.md) *(separate document)*
23. [Verification](#verification)
24. [Sequencing](#sequencing)

---

## R1: `NodeEntry` State Machine Transitions

### Problem

Node state transitions require manually coordinating 3â€“5 fields (`set_state`, `set_dirty_aspects`/`add_dirty_aspect`, `clear_dirty_partition_scopes`/`clear_dirty_partition_scopes_for`/`add_dirty_partition_scope`). This ceremony is duplicated across 5+ locations with subtle semantic differences.

### Exhaustive Inventory of Duplication

| Transition                    | File                                                                                                                                                                  | Lines     | Fields Touched                                                                                                            | Telemetry                         |
| ----------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- | ------------------------------------------------------------------------------------------------------------------------- | --------------------------------- |
| Source â†’ Dirty                | [invalidation.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/invalidation.rs#L76-L84)                         | L76â€“L84   | `set_state(Dirty)`, `add_dirty_aspect`, `merge_dirty_partition_scopes`                                                    | none                              |
| Direct Sub â†’ Dirty/MaybeStale | [invalidation.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/invalidation.rs#L163-L177)                       | L163â€“L177 | `set_state(Dirty\|MaybeStale)`, `add_dirty_aspect`, `merge_dirty_partition_scopes`                                        | `invalidation_nodes_visited` L189 |
| Transitive Sub â†’ MaybeStale   | [invalidation.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/invalidation.rs#L438-L450)                       | L438â€“L450 | `set_state(MaybeStale)`, `add_dirty_aspect`, `clear_dirty_partition_scopes_for`                                           | `invalidation_nodes_visited` L426 |
| Eval Result â†’ Clean           | [result_apply.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/evaluation/engine/result_apply.rs#L116-L123)     | L116â€“L123 | `set_aspect_version`, `set_trace_summary`, `set_state(Clean)`, `set_dirty_aspects(EMPTY)`, `clear_dirty_partition_scopes` | `nodes_recomputed` L127           |
| Comparator Skip â†’ Clean       | [prepared_apply.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/evaluation/engine/prepared_apply.rs#L123-L130) | L123â€“L130 | `set_state(Clean)`, `set_dirty_aspects(EMPTY)`, `clear_dirty_partition_scopes`                                            | `skipped_by_comparator` L124      |
| Condition Skip â†’ Clean        | [prepared_apply.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/evaluation/engine/prepared_apply.rs#L132-L141) | L132â€“L141 | `set_state(Clean)`, `set_dirty_aspects(EMPTY)`, `clear_dirty_partition_scopes`                                            | none                              |
| Condition Defer â†’ MaybeStale  | [prepared_apply.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/evaluation/engine/prepared_apply.rs#L143-L146) | L143â€“L146 | `set_state(MaybeStale)`                                                                                                   | none                              |

### Design

Add transition methods to `NodeEntry` ([data/node/entry.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/node/entry.rs)):

```rust
impl NodeEntry {
    /// Atomically transition to Clean. All dirty tracking is reset.
    pub fn transition_clean(&mut self) {
        self.set_state(NodeState::Clean);
        self.set_dirty_aspects(AspectMask::EMPTY);
        self.clear_dirty_partition_scopes();
    }

    /// Atomically transition to Dirty for a specific aspect with scoped regions.
    pub fn transition_dirty(
        &mut self,
        aspect: Aspect,
        scopes: &[PartitionSubscription],
    ) {
        let was_clean = matches!(*self.get_state(), NodeState::Clean);
        let already_dirty = self.get_dirty_aspects().contains(AspectMask::from_aspect(aspect));
        self.set_state(NodeState::Dirty);
        self.add_dirty_aspect(aspect);
        merge_dirty_partition_scopes(self, aspect, scopes, was_clean, already_dirty);
    }

    /// Atomically transition to MaybeStale for a specific aspect.
    pub fn transition_maybe_stale(&mut self, aspect: Aspect) {
        let was_clean = matches!(*self.get_state(), NodeState::Clean);
        let already_dirty = self.get_dirty_aspects().contains(AspectMask::from_aspect(aspect));
        self.set_state(NodeState::MaybeStale);
        self.add_dirty_aspect(aspect);
        if was_clean || !already_dirty {
            self.clear_dirty_partition_scopes_for(aspect);
        }
    }
}
```

`merge_dirty_partition_scopes` moves from `invalidation.rs` L480â€“L505 into `data/node/entry.rs` as a private helper.

### Files Modified

| File                                                                                                                                                        | Change                                                                                                                                                                                                          |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [data/node/entry.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/node/entry.rs)                       | Add `transition_clean`, `transition_dirty`, `transition_maybe_stale`; absorb `merge_dirty_partition_scopes`                                                                                                     |
| [invalidation.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/invalidation.rs)                       | Replace L76â€“L84, L163â€“L177, L438â€“L450 with single transition calls; delete `merge_dirty_partition_scopes`                                                                                                       |
| [result_apply.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/evaluation/engine/result_apply.rs)     | Replace L116â€“L123 with `entry.transition_clean()`                                                                                                                                                               |
| [prepared_apply.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/evaluation/engine/prepared_apply.rs) | Collapse `revert_to_clean` + `revert_to_clean_due_to_condition` into one function calling `transition_clean`; `defer_due_to_condition` calls `set_state(MaybeStale)` directly (single field, no wrapper needed) |

### Lines Saved: ~50

### Bug Class Eliminated

"Inconsistent state transition" â€” WORTHtting to clear scopes, WORTHtting to add dirty aspect, using wrong clearing method for the transition type.

---

## R2: `PartitionScoped` Trait for Scope Matching

### Problem

The same partition-scope matching logic exists in **5 separate functions** across 3 files:

| Function                                                                                                                                                                        | File              | Lines     | Input Types                                                       |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------- | --------- | ----------------------------------------------------------------- |
| [partition_scope_matches](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/invalidation.rs#L329-L343)                         | `invalidation.rs` | L329â€“L343 | `&PartitionSubscription` Ã— `&PartitionSubscription`               |
| [interned_partition_scope_matches](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/invalidation.rs#L295-L309)                | `invalidation.rs` | L295â€“L309 | `InternedPartitionSubscription` Ã— `InternedPartitionSubscription` |
| [partition_subscription_matches](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/evaluation/engine/suppression.rs#L111-L125) | `suppression.rs`  | L111â€“L125 | `&PartitionSubscription` Ã— `&ChangedRegion`                       |
| [partition_scope_touched](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/evaluation/engine/suppression.rs#L85-L102)         | `suppression.rs`  | L85â€“L102  | wraps above with trace lookup                                     |
| [partition_scope_untouched](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/planner/plan_builder.rs#L343-L360)               | `plan_builder.rs` | L343â€“L360 | **Yet another copy**                                              |

All five implement the same semantic: "does partition A overlap with partition B, considering WholePartition vs PartitionAndDetail?"

### Design

#### New trait in `data/output.rs`

```rust
/// Anything that can participate in partition scope matching.
pub trait PartitionScoped {
    fn partition_token(&self) -> &PartitionToken;
    fn detail_token(&self) -> Option<&DetailToken>;
    fn match_mode(&self) -> PartitionMatchMode;
}

impl PartitionScoped for PartitionSubscription { ... }
impl PartitionScoped for InternedPartitionSubscription { ... }
impl PartitionScoped for ChangedRegion { ... }
```

#### One function

```rust
pub fn scopes_overlap(a: &impl PartitionScoped, b: &impl PartitionScoped) -> bool {
    if a.partition_token() != b.partition_token() { return false; }
    match (a.match_mode(), b.match_mode()) {
        (WholePartition, _) | (_, WholePartition) => true,
        (PartitionAndDetail, PartitionAndDetail) => a.detail_token() == b.detail_token(),
    }
}
```

#### Higher-level helpers stay as thin wrappers

```rust
pub fn scope_touched_by_trace(trace: Option<&TraceSummary>, scope: &PartitionSubscription) -> bool {
    let Some(trace) = trace else { return false };
    if trace.output_change == OutputChange::Unchanged { return false; }
    if trace.changed_regions.is_empty() { return true; }
    trace.changed_regions.iter().any(|region| scopes_overlap(scope, region))
}
```

### Files Modified

| File                                                                                                                                                  | Change                                                                    |
| ----------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| [data/output.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/output.rs)                         | Add `PartitionScoped` trait, impls, `scopes_overlap`                      |
| [invalidation.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/invalidation.rs)                 | Delete L295â€“L343; replace with `scopes_overlap` calls                     |
| [suppression.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/evaluation/engine/suppression.rs) | Delete L85â€“L125; replace with `scope_touched_by_trace` + `scopes_overlap` |
| [plan_builder.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/planner/plan_builder.rs)         | Delete L343â€“L360; import shared function                                  |

### Lines Saved: ~80

### Bug Class Eliminated

"Scope matching semantic drift" â€” five functions that could diverge independently (and one already handles `detail: None` differently from the others).

---

## R3: `DependencyKey` and Named Snapshot Entries

### Problem

`DependencySnapshot` stores entries as bare tuples:

```rust
entries: Vec<(NodeId, Aspect, u64, Option<PartitionSubscription>)>
```

This results in:

1. **Positional access throughout the codebase** â€” `snapshot.0`, `snapshot.1`, `snapshot.2`, `snapshot.3` with no field names.
2. **Three separate comparison functions** that extract the same sort key from different representations:

| Function                                                                                                                                                                         | File              | Lines     | Extracts From                       |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------- | --------- | ----------------------------------- |
| [compare_snapshot_entries](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/dependency.rs#L205-L223)                            | `dependency.rs`   | L205â€“L223 | `(NodeId, Aspect, u64, Option<PS>)` |
| [compare_snapshot_identity](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/dependency.rs#L225-L235)                           | `dependency.rs`   | L225â€“L235 | `(NodeId, Aspect, u64, Option<PS>)` |
| [compare_dependency_to_snapshot](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/evaluation/engine/result_apply.rs#L230-L251) | `result_apply.rs` | L230â€“L251 | `DependencyEdge` vs tuple           |

3. **Implicit sort key agreement** â€” `DependencyEdge` and `DependencySnapshot` entries must sort in the same order for binary searching, but nothing in the type system guarantees this.

### Design

#### Named snapshot entry

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DependencySnapshotEntry {
    pub source: NodeId,
    pub aspect: Aspect,
    pub cached_version: u64,
    pub scope: Option<PartitionSubscription>,
}
```

#### Shared sort key

```rust
/// Canonical sort key for any dependency reference.
/// Implement Ord once, correct everywhere.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DependencySortKey {
    source_index: u32,
    source_generation: u32,
    aspect_index: u8,
    scope: Option<PartitionSubscription>,
}

impl DependencyEdge {
    pub fn sort_key(&self) -> DependencySortKey { ... }
}

impl DependencySnapshotEntry {
    pub fn sort_key(&self) -> DependencySortKey { ... }
}
```

#### Impact on `DependencySnapshot`

```rust
pub struct DependencySnapshot {
    entries: Vec<DependencySnapshotEntry>,  // was: Vec<(NodeId, Aspect, u64, Option<PS>)>
}
```

All three comparison functions are replaced by `a.sort_key().cmp(&b.sort_key())`.

### Files Modified

| File                                                                                                                                                    | Change                                                                                                                                              |
| ------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| [dependency.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/dependency.rs)                        | Introduce `DependencySnapshotEntry`, `DependencySortKey`; replace tuple with struct, delete `compare_snapshot_entries`, `compare_snapshot_identity` |
| [result_apply.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/evaluation/engine/result_apply.rs) | Delete `compare_dependency_to_snapshot` (L230â€“L251), use `DependencySortKey`                                                                        |
| [storage.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/graph/storage.rs)                        | Replace `compare_dependency_edges` with `DependencyEdge::sort_key()`                                                                                |
| All consumers of `.entries()`                                                                                                                           | Change from `(NodeId, Aspect, u64, Option<PS>)` to `DependencySnapshotEntry` field access                                                           |

### Lines Saved: ~60

---

## R4: `ScratchGuard` RAII Lease

### Problem

[TraversalScratch](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/graph/scratch.rs) uses a runtime-checked `acquire`/`restore` protocol. The lease kind must match on restore. WORTHtting to call `restore` leaks the scratch buffer, causing subsequent operations to fail with `ScratchReentryError`.

Current usage pattern in [invalidation.rs L39â€“L49](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/invalidation.rs#L39-L49):

```rust
let mut scratch = graph.acquire_scratch(ScratchLeaseKind::Invalidation)?;
// ... 60 lines of work that could early-return via ? ...
let result = mark_dirty_with_scratch(graph, &mut scratch, ...);
graph.restore_scratch(ScratchLeaseKind::Invalidation, scratch)?;  // must not WORTHt this
```

### Design

Because `ScratchGuard` cannot hold `&mut SignalGraph` alongside the scratch (the graph owns the scratch slot), use a **closure-based** pattern matching `BRepWorkspace::as_parts_mut()` from WORTH-kernel:

```rust
impl SignalGraph {
    /// Borrow the scratch buffer for the duration of `f`. Automatically restores on return.
    pub fn with_scratch<R>(
        &mut self,
        kind: ScratchLeaseKind,
        f: impl FnOnce(&mut SignalGraph, &mut TraversalScratch) -> Result<R, SignalError>,
    ) -> Result<R, SignalError> {
        let mut scratch = self.acquire_scratch(kind)?;
        let result = f(self, &mut scratch);
        self.restore_scratch(kind, scratch)?;
        result
    }
}
```

> [!NOTE]
> This requires the graph to temporarily store a placeholder in the scratch slot during `f`, since `self` is passed into the closure. The existing `acquire_scratch` already does this (it swaps with `Default::default()`).

### Files Modified

| File                                                                                                                                       | Change                                                                      |
| ------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------- |
| [signal_graph.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/graph/signal_graph.rs) | Add `with_scratch` method                                                   |
| [invalidation.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/invalidation.rs)      | Replace L39â€“L49 with `graph.with_scratch(kind, \|graph, scratch\| { ... })` |
| All other `acquire_scratch`/`restore_scratch` call sites                                                                                   | Same closure conversion                                                     |

### Bug Class Eliminated

"Unreturned scratch lease" â€” impossible, because `with_scratch` restores on both Ok and Err paths.

---

## R5: Edge Mutation Ceremony Extraction

### Problem

Every edge mutation in [storage.rs L252â€“L405](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/graph/storage.rs#L252-L405) follows the same 5-step ceremony:

```
1. Read current edges into Vec      (snapshot)
2. Mutate the Vec                   (add/remove/filter)
3. Insert mutated Vec into store    (persist)
4. Update NodeEntry with new SetId  (wire)
5. Call maybe_compact_graph_storage (compact)
```

This ceremony appears in 6 functions:

| Function                           | Lines     | Mutation Type              |
| ---------------------------------- | --------- | -------------------------- |
| `add_dependency_edge`              | L252â€“L268 | sorted insert              |
| `remove_dependency_edge`           | L282â€“L301 | filter                     |
| `remove_dependency_edges_matching` | L303â€“L329 | filter with scope matching |
| `remove_dependencies_on`           | L331â€“L350 | filter by source           |
| `add_subscriber_edge`              | L359â€“L374 | sorted insert              |
| `remove_subscriber_edge`           | L387â€“L405 | filter                     |

### Design

Following WORTH-kernel's `BRepWorkspace::as_parts_mut()` pattern â€” destructure the graph into the parts the mutation needs, then operate on them independently.

Two alternatives:

**Option A: Closure-based `mutate_edges`**

```rust
impl SignalGraph {
    fn mutate_deps(
        &mut self,
        node: NodeId,
        mutate: impl FnOnce(&mut Vec<DependencyEdge>) -> bool,
    ) -> Result<bool, SignalError> {
        let mut edges = self.dependencies_of(node)?.to_vec();
        if !mutate(&mut edges) { return Ok(false); }
        let id = self.dependency_edges.insert_from_slice(&edges);
        self.get_entry_mut(node)?.set_dependencies_id(id);
        self.maybe_compact_graph_storage();
        Ok(true)
    }

    fn mutate_subs(
        &mut self,
        node: NodeId,
        mutate: impl FnOnce(&mut Vec<NodeId>) -> bool,
    ) -> Result<bool, SignalError> {
        // identical pattern for subscribers
    }
}
```

**Option B: Use reconciliation (see R6)** â€” if R6 is implemented, most of these functions become unnecessary entirely.

### Recommendation

Implement Option A as an incremental step. When R6 lands, the individual add/remove methods become thin callers of `mutate_deps`; several can be deprecated entirely.

### Files Modified

| File                                                                                                                             | Change                                                                       |
| -------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| [storage.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/graph/storage.rs) | Extract `mutate_deps`/`mutate_subs`; rewrite 6 functions as 1-3 line callers |

### Lines Saved: ~100

---

## R6: Declarative Dependency Reconciliation

### Problem

Dependency management is imperative: callers must manually track what to add and what to remove. The evaluation engine already computes the desired dependency set via `PreparedDependencyCapture` and then [apply_prepared_dependencies](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/evaluation/engine/prepared_apply.rs#L90-L121) manually diffs old vs new.

This is the Kubernetes reconciliation pattern buried inside the evaluation engine instead of being the primary API.

### Design

Promote reconciliation to a first-class graph API (following WORTH-kernel's `OperationResult<T>` envelope pattern for reporting what changed):

```rust
#[derive(Debug, Default)]
pub struct ReconciliationReport {
    pub added: u32,
    pub removed: u32,
    pub unchanged: u32,
}

impl SignalGraph {
    pub fn reconcile_dependencies(
        &mut self,
        node: NodeId,
        desired: &[DependencyEdge],
    ) -> Result<ReconciliationReport, SignalError> {
        let current = self.dependencies_of(node)?.to_vec();
        let mut report = ReconciliationReport::default();
        // diff current vs desired, issue add/remove operations, return report
        ...
    }
}
```

`apply_prepared_dependencies` becomes:

```rust
let edges: Vec<DependencyEdge> = capture.as_edges(graph)?;
let report = graph.reconcile_dependencies(node, &edges)?;
```

### Migration

The existing imperative API (`connect_dependency_capture`, `disconnect_dependency_edge`) is kept initially but marked `#[deprecated]`. New code uses `reconcile_dependencies` exclusively.

---

## R7: Invalidation Pass Pipeline

### Problem

[mark_dirty_with_scratch](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/invalidation.rs#L61-L125) is a 65-line monolith that performs 5 sequential phases:

1. **Mark source** â€” transitions source node to Dirty (L68â€“L86)
2. **Collect subscribers** â€” gathers live direct subscribers (L97â€“L98)
3. **Detect cycles** â€” DFS cycle check on the reachable subgraph (L99)
4. **Mark direct subscribers** â€” partition-aware dirty/maybe-stale marking (L100â€“L117)
5. **Propagate transitive** â€” BFS frontier marking all reachable nodes MaybeStale (L119â€“L124)

Each phase reads from and writes to shared mutable state (`graph` + `scratch`). Testing phase 3 (cycle detection) requires setting up all state for phases 1â€“2. Adding a new pass (e.g., "skip OnDemand-gated subscribers") requires modifying the monolith.

### Design

Restructure into a pipeline struct carrying intermediate state:

```rust
pub struct InvalidationPipeline<'a> {
    graph: &'a mut SignalGraph,
    scratch: &'a mut TraversalScratch,
    source: NodeId,
    aspect: Aspect,
    changed_scopes: Vec<PartitionSubscription>,
    changed_scope_ids: Vec<InternedPartitionSubscription>,
}

impl<'a> InvalidationPipeline<'a> {
    pub fn new(graph, scratch, source, aspect, regions) -> Self { ... }
    pub fn mark_source(&mut self) -> Result<(), SignalError> { ... }
    pub fn collect_subscribers(&mut self) -> Result<(), SignalError> { ... }
    pub fn detect_cycles(&self) -> Result<(), SignalError> { ... }
    pub fn mark_direct(&mut self) -> Result<InvalidationStats, SignalError> { ... }
    pub fn propagate_transitive(&mut self) -> Result<(), SignalError> { ... }
}
```

The top-level function becomes:

```rust
fn mark_dirty_with_scratch(...) -> Result<(), SignalError> {
    let mut pipeline = InvalidationPipeline::new(graph, scratch, source, aspect, regions);
    pipeline.mark_source()?;
    pipeline.collect_subscribers()?;
    pipeline.detect_cycles()?;
    let stats = pipeline.mark_direct()?;
    graph.record_invalidation_diagnostics(stats);
    pipeline.propagate_transitive()
}
```

### Why This Matters

Each pass can be unit-tested with a minimal graph + scratch setup. Future passes (tier-filtered propagation, OnDemand gating) are new methods, not edits to a monolith.

---

## R8: Zero-Allocation Planner & Prepared Cursor

### Problem

The evaluation planner ([plan_builder.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/planner/plan_builder.rs)) allocates a new `EvaluationPlan` per evaluation. This struct contains a `Vec<ExecutionStage>`, which itself contains a `Vec<EvaluationTask>`.
Similarly, the execution engine produces an `ExecutionReport` containing a fresh `Vec<StageExecutionRecord>`, each containing a `Vec<TaskExecutionRecord>`.

These are heap allocations occurring on _every_ transaction commit, bypassing the arena storage entirely. For a graph executing hundreds of micro-transactions per second, this results in significant heap churn and fragmentation.

### Design

Apply the **Areana-backed Cursor Pattern** (similar to ECS query iterators).

Instead of the planner returning owned `Vec`s, the planner writes task schedules into a pre-allocated graph-owned scratch buffer (e.g., `graph.task_scratch`), and returns an `EvaluationCursor`â€”a lightweight struct containing only index ranges.

```rust
pub struct EvaluationCursor {
    start_index: usize,
    end_index: usize,
    stage_boundaries: Range<usize>, // index into a separate stage boundaries buffer
}

// The execution engine consumes the cursor, reading directly from the graph's pre-allocated buffers:
pub fn execute_prepared_plan(
    graph: &mut SignalGraph,
    cursor: EvaluationCursor,
    // ...
)
```

The `ExecutionReport` telemetry should also be accumulated into graph-owned buffers, moving the rich struct out of the hot path and only materializing it if trace logging is enabled.

### Bug Class Eliminated

"Transaction-rate GC pressure" â€” Removes unbounded dynamic allocations from the hottest path in the system.

---

## R15: Partition-Aware MaybeStale Validation

### Problem

There is a semantic disconnect between how `invalidation.rs` pushes `MaybeStale` states, and how `result_apply.rs` (and the execution engine generally) validates if a `MaybeStale` node actually needs to re-evaluate.

During invalidation, the engine correctly checks if a subscriber's `PartitionSubscription` overlaps with the `changed_scopes` of the upstream node. If they don't overlap, the subscriber remains `Clean`.

However, when a subscriber is marked `MaybeStale` (e.g., due to a transitive dependency chain), and the executor goes to validate it, [count_meaningful_input_changes()](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/evaluation/engine/result_apply.rs#L168-L179) compares the cached dependency version against the current `AspectVersion` of the upstream node.

**The Bug:** The upstream `AspectVersion` is a flat integer slot. It increments if _any_ partition changes.
If Node A (partition `East`) changes, its `AspectVersion` increments.
Node B (subscribes to partition `West` on A) is transitive downstream. It was marked `MaybeStale` through a different path.
The executor checks if B needs to re-evaluate by looking at A. It sees A's `AspectVersion` is higher than cached.
B re-evaluates, even though the only part of A that changed was `East`, which B doesn't care about!

### Design

Abstract the combination of `<AspectVersion, PartitionScope>` into a `ScopeVersionMap`.

Instead of `AspectVersion` being a single integer array, it must track versions _per partition scope_.

```rust
// A structural trie or flat-map stored on the NodeEntry mapping scopes to versions
#[derive(Debug, Clone, Default)]
pub struct PartitionVersionMap {
    global_aspect_version: AspectVersion, // fallback/wildcard
    partition_versions: HashMap<PartitionToken, AspectVersion>, // or a faster flat map
}
```

- When Node A emits changes for `East`, only the `East` version in its map increments.
- The `DependencySnapshotEntry` stores the specific version for the scope it subscribed to, not the global version.
- `count_meaningful_input_changes()` now compares apples to apples: "Did the version of the _specific partition I care about_ change?"

### Bug Class Eliminated

"Partition-scoped invalidation Ã— MaybeStale validation mismatch" (Over-evaluation of transitive dependencies due to global flat versioning of partitioned outputs).

---

## R19: `PhaseGuard<P>` â€” Cross-Epoch Re-entrancy Prevention

### Problem

GC epochs, checkpoint barriers, compaction triggers, and evaluation stages all define different "phase boundaries." They assume mutual exclusion: you don't compact during evaluation, you don't GC during invalidation, you don't flush events during planning. But nothing in the type system enforces this. If a future code path calls `mark_dirty` from inside an `on_checkpoint` handler, which triggers `stage_mark_dirty_candidates`, which calls `maybe_compact_graph_storage`, you get compaction inside event flushing inside transaction commit.

Each subsystem's invariants hold in isolation but break under re-entrant composition. Rust's borrow checker prevents some of this, but not all â€” mutable borrows through different accessor methods on the same struct can compose in ways that violate phase assumptions.

### Design

Wrap the graph in a typestate that restricts which operations are legal in each phase:

```rust
pub struct GraphHandle<Phase> {
    graph: SignalGraph,
    _phase: PhantomData<Phase>,
}

pub struct Idle;
pub struct Evaluating;
pub struct Invalidating;
pub struct Observing;

impl GraphHandle<Idle> {
    pub fn begin_evaluation(self) -> GraphHandle<Evaluating> { ... }
    pub fn begin_invalidation(self) -> GraphHandle<Invalidating> { ... }
    pub fn begin_observation(&self) -> GraphHandle<Observing> { ... }
    pub fn run_gc(&mut self) { ... }   // only on Idle
    pub fn compact(&mut self) { ... }  // only on Idle
}

impl GraphHandle<Evaluating> {
    pub fn evaluate_node(&mut self, ...) { ... }
    pub fn finish(self) -> GraphHandle<Idle> { ... }
    // mark_dirty is NOT available here â€” compile error
    // compact is NOT available here â€” compile error
}

impl GraphHandle<Invalidating> {
    pub fn mark_dirty(&mut self, ...) { ... }
    pub fn propagate(&mut self, ...) { ... }
    pub fn finish(self) -> GraphHandle<Idle> { ... }
    // evaluate_node is NOT available here â€” compile error
}

impl GraphHandle<Observing> {
    // Only &self methods â€” diagnostics, explain, telemetry reads
    pub fn explain(&self, ...) { ... }
    pub fn replay_events(&self) -> &[ReplayEvent] { ... }
    // No mutation methods exist here at all
}
```

### Why This Matters

R8 (Zero-Allocation Planner) and R9 (Pipeline Execution Engine) define the evaluation and invalidation entry points. If they are built carelessly, they can make a later phase-guard layer harder to introduce. **R19 is a design constraint on R8/R9's public shape, but it is not a blocker for the current internal planner/executor refactor.** The immediate rule is: do not widen public mutation entry points or bake new raw `&mut SignalGraph` escape hatches into Batch C.

### Bug Class Eliminated

"Cross-Epoch Re-entrancy" â€” calling GC during evaluation, compaction during invalidation, mark_dirty during planning.

---

## R20: Observation Purity â€” `&self` Diagnostic Enforcement

### Problem

Reading the graph for diagnostics, telemetry, explain queries, or metrics can mutate shared state. The partition interner grows when you intern a new scope during a diagnostic query. The edge store interner rebuilds when you call `dependencies_of()` for an explain trace. The scratch buffer gets leased for a diagnostic traversal.

If any of these side effects alter the graph's behavior on the next real computation, then observing the graph changes the graph. The nastiest form: a diagnostic query triggers `maybe_compact_graph_storage()`, which remaps all edge IDs, which invalidates cached lookups in an in-flight evaluation.

### Design

All diagnostic, telemetry, and explain methods must take `&self`, not `&mut self`. Any lazy initialization (interner rebuild, compaction) must happen *before* the observation phase â€” during a preparation step that takes `&mut self`.

```rust
impl SignalGraph {
    /// Call before entering observation mode. Takes &mut self.
    pub fn prepare_for_observation(&mut self) {
        self.rebuild_interner_if_needed();
        self.compact_graph_storage_if_needed();
    }

    /// All diagnostic methods are &self â€” pure reads.
    pub fn explain(&self, node: NodeId) -> ExplainTrace { ... }
    pub fn replay_events(&self) -> &[ReplayEvent] { ... }
    pub fn dependencies_of(&self, node: NodeId) -> &[DependencyEdge] { ... }
}
```

This is tightly related to R19 (`PhaseGuard`): the `Observing` phase only exposes `&self` methods. The preparation happens during the `Idle â†’ Observing` transition.

### Why This Is Foundational

R12 (Telemetry Decomposition) is restructuring how diagnostics and telemetry are accessed. If those methods aren't designed as `&self` from the start, the entire diagnostics API will need a second refactor. **R20 constrains R12's design.**

### Bug Class Eliminated

"Observation Contamination" â€” reading the graph for diagnostics mutates state that affects subsequent computation.

---

## R21: Single Source of Truth â€” Representational Drift Prevention

### Problem

The codebase maintains two supposedly-equivalent representations of the same truth: dependency edges and subscriber edges are duals of the same relationship. Interned partition IDs and string-based partition tokens represent the same concept. Edge store flat segments and interner HashMap keys describe the same set of edges.

Each should be derivable from the other. But mutations update one representation and reconstruct the other lazily (e.g., `rebuild_interner_if_needed`). If the rebuild happens at the wrong time, or if a mutation path updates one but not the other, the two representations disagree.

Unlike Topological Dementia (dead references), both representations contain valid, live data â€” they just say different things about the same relationship.

### Design

Make one representation the canonical source, and make the other a derived view that is structurally impossible to mutate independently.

```rust
/// Canonical: dependencies are the single source of truth.
/// Subscribers are computed, never stored independently.
impl SignalGraph {
    /// Returns a fresh view derived from dependency edges.
    /// Not cached. Computed on demand.
    pub fn subscribers_of(&self, node: NodeId) -> Vec<NodeId> {
        self.all_dependencies()
            .filter(|edge| edge.source() == node)
            .map(|edge| edge.owner())
            .collect()
    }
}
```

For performance, the derived view can be cached â€” but the cache must carry a branded epoch that expires when the canonical data mutates:

```rust
struct SubscriberCache<'epoch> {
    data: Vec<NodeId>,
    _epoch: PhantomData<&'epoch ()>,
}

impl SignalGraph {
    /// Cache is valid only while the graph is not mutated.
    /// Any &mut self call advances the epoch, invalidating the cache borrow.
    pub fn cached_subscribers_of(&self) -> SubscriberCache<'_> { ... }
}
```

### Why This Is Foundational

R5 (Edge Mutation Ceremony) and R6 (Declarative Reconciliation) are already extracting the edge mutation surface. If the ceremony doesn't enforce that dependencies and subscribers are kept in sync *structurally* (not by convention), you're locking in the exact dual-representation problem. **R21 is R5/R6's core concern â€” it must be stated as an explicit design constraint.**

### Bug Class Eliminated

"Representational Drift" â€” two supposedly-equivalent representations of the same relationship silently diverge.

---

## R22: Transactional Mutation â€” Rollback Amnesia Prevention

### Problem

The `SparsePatchBuffer` captures `NodeEntry` snapshots, but diagnostics state, partition interner growth, edge store segments, and memo cache mutations are tracked through separate rollback paths. If any one of those paths has a gap, rollback produces a graph state that never existed â€” not the pre-transaction state and not the post-transaction state, but a Frankenstate.

The insidious part: the graph looks valid. It just contains data from two different timelines.

### Design

All graph mutation must flow through a `TransactionalMut<'tx>` wrapper that auto-records undo entries. The raw `&mut SignalGraph` is not accessible during a transaction.

```rust
/// During a transaction, this is the ONLY way to mutate the graph.
/// Every mutation method auto-records an undo entry.
pub struct TransactionalMut<'tx> {
    graph: &'tx mut SignalGraph,
    undo_log: &'tx mut UndoLog,
}

impl<'tx> TransactionalMut<'tx> {
    pub fn transition_dirty(&mut self, node: NodeId, aspect: Aspect, scopes: &[PartitionSubscription]) {
        let old = self.graph.get_entry(node).unwrap().clone();
        self.undo_log.record(node, old);
        self.graph.get_entry_mut(node).unwrap().transition_dirty(aspect, scopes);
    }

    // Every mutating method follows the same pattern:
    // 1. Snapshot old state
    // 2. Record in undo log
    // 3. Apply mutation
}
```

For catching new fields at compile time, use `#[must_use]` on mutation results:

```rust
#[must_use = "This mutation must be recorded in the transaction undo log"]
pub struct MutationReceipt<T> { revert_data: T }
```

### Why This Is Foundational

R1 (NodeEntry Transitions) restricts the mutation surface. If R1 doesn't also consider how mutations feed into the transaction layer, the transition methods will need to be refactored a second time when undo-token generation is wired in. **R22 is a design constraint on R1's mutation API.**

### Bug Class Eliminated

"Rollback Amnesia" â€” transaction layer fails to record some piece of state in its undo log, producing a Frankenstate on rollback.

---

## Meta-Abstractions

> [!NOTE]
> Several of the refactoring items in this spec share underlying type-system patterns. These two meta-abstractions are worth calling out because instantiating them once and reusing them across items reduces total implementation work and keeps the codebase consistent.

### `BrandedHandle<'scope, T>` â€” Generative Lifetime Branding

The pattern of wrapping a value in a struct with a `PhantomData<&'scope T>` so that the Rust borrow checker ensures the handle cannot outlive its originating scope. Used by:

| Instantiation | R-Item | Bug Class |
|---|---|---|
| `NodeRef<'graph>` | R16 | Topological Dementia |
| `SubscriberCache<'epoch>` | R21 | Representational Drift |
| `Version<'timeline>` | Deferred | Monotonicity Violation |
| `FrameValue<'epoch>` | Deferred | Temporal Aliasing |
| `BranchConfig<'branch>` | Deferred | Branch State Leakage |

All five are the same generic pattern. Implementing `NodeRef<'graph>` and `SubscriberCache<'epoch>` first gives the codebase a reusable branded-handle idiom for the deferred items.

### `PhaseGuard<P>` â€” Typestate Phase Restriction

The pattern of wrapping a resource in a typestate that restricts which methods are available. Used by:

| Phase Set | R-Item | Bug Class |
|---|---|---|
| `Idle / Evaluating / Invalidating / Observing` | R19 | Cross-Epoch Re-entrancy |
| `&self` restriction on `Observing` phase | R20 | Observation Contamination |
| `TransactionalMut` vs raw `&mut` | R22 | Rollback Amnesia |

All three are instances of "restrict which operations are legal based on the current phase." R19 is the graph-level equivalent of what R1 does at the node level.

---

## R9: Feature-Gated Pipeline Execution Engine

### Problem

[runtime_execution.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/runtime_execution.rs) and [execution.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/planner/execution.rs) contain massive monolithic execution functions (`execute_prepared_plan_with_policy` is ~400 lines).
The code is heavily littered with `#[cfg(feature = "parallel")]` conditional compilation blocks interleaving directly with core business logic.

This makes the executor incredibly difficult to read, test in isolation, or extend.

### Design

Adopt the `WORTH-kernel` **FeaturePipeline** concept. Define a standard `ExecutionPass` trait, and construct the execution pipeline dynamically.

```rust
pub trait ExecutionPass {
    fn run(&self, ctx: &mut ExecutionContext) -> Result<(), SignalError>;
}

pub struct ExecutionContext<'a> {
    pub graph: &'a mut SignalGraph,
    pub cursor: EvaluationCursor,
    pub report: &'a mut ExecutionReportBuilder,
    pub resolver: &'a mut dyn ComparatorPolicyResolver,
}
```

Implement standard passes: `SnapshotPass`, `SerialPrecomputePass`, `SemanticFinalizePass`.
In a separate module gated by `#[cfg(feature = "parallel")]`, implement `ParallelPrecomputePass` and `FullParallelPatchPass`.

The executor simply runs the configured pipeline. Conditional compilation is restricted to pipeline _assembly_, entirely removing it from the business logic.

### Bug Class Eliminated

"Conditional compilation rot" â€” Ensure sequential and parallel executors share the identical architectural seams, preventing them from diverging purely because of `#cfg` spaghetti.

---

## R10: Epoch-Driven Amortized Garbage Collection

### Problem

[lifecycle.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/graph/lifecycle.rs) implements `run_gc_epoch()` as a synchronous, "stop-the-world" sweep. When the `tombstone_count` exceeds a threshold, the graph halts to dynamically re-allocate massive `Vec` arrays for `gc_liveness_generations` and `gc_liveness_alive` bitsets into the scratch buffer, iterates over every single node in the graph, and explicitly rebuilds subscriber edges.

This creates unpredictable latency spikes in tail latencies for large graphs.

### Design

Migrate to **Amortized Generational Reclamation**.

Instead of sweeping the entire graph, GC constraints are evaluated incrementally during other graph traversals (like Evaluation or Invalidation).
If a traversal touches an edge pointing to a tombstoned generation, the edge is dropped _lazily_.

The formal `run_gc_epoch` is simplified to only perform the `compact_graph_storage()` (which rewrites the backing stores), and even that can be broken into background chunks (e.g., compacting 10% of the stores per transaction once the threshold is reached).

Liveness checking arrays (`gc_liveness_generations`) are entirely eliminated because dead edge detection happens organically via `.is_alive()` checks during normal reads.

### Bug Class Eliminated

"Latency jitter" â€” Eliminates O(N) full-graph traversals triggered unexpectedly during transactions.

---

## R11: Unified `SegmentedStore<T, Id>`

### Problem

[DependencyEdgeStore](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/graph/edge_store.rs#L58-L123) and [SubscriberEdgeStore](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/graph/edge_store.rs#L125-L191) are structurally identical. Both maintain `Vec<T>`, `Vec<Segment>`, and `HashMap<u64, Vec<Id>>`. The `rebuild_interner_if_needed`, `get`, `insert_from_slice`, `storage_counts`, and `live_segment_count` methods are copy-pasted with only the element type and id type changed.

[DependencySnapshotStore](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/dependency.rs#L144-L198) is a third instance of the same pattern, differing only in that it uses `HashMap<DependencySnapshot, Id>` (equality-based) instead of `HashMap<u64, Vec<Id>>` (hash-based). This store should remain separate unless the interning strategy can be unified.

### Evidence

| Struct                | Lines                | Element Type     | Id Type           |
| --------------------- | -------------------- | ---------------- | ----------------- |
| `DependencyEdgeStore` | L58â€“L123 (65 lines)  | `DependencyEdge` | `DependencySetId` |
| `SubscriberEdgeStore` | L125â€“L191 (66 lines) | `NodeId`         | `SubscriberSetId` |

Method-by-method diff:

| Method                       | Dep version | Sub version | Difference                                                     |
| ---------------------------- | ----------- | ----------- | -------------------------------------------------------------- |
| `rebuild_interner_if_needed` | L67â€“L79     | L134â€“L147   | `edges` â†’ `subscribers`, `DependencySetId` â†’ `SubscriberSetId` |
| `get`                        | L81â€“L89     | L149â€“L157   | `edges` â†’ `subscribers`                                        |
| `insert_from_slice`          | L91â€“L113    | L159â€“L181   | `edges` â†’ `subscribers`, id types differ                       |
| `live_segment_count`         | L120â€“L122   | L188â€“L190   | identical                                                      |

The `DependencySetId` and `SubscriberSetId` types (L9â€“L23 and L36â€“L50) are also identical except for name.

### Design

#### New trait: `SetHandle`

```rust
// data/graph/edge_store.rs
pub trait SetHandle: Copy + Eq + Hash + Serialize + DeserializeOwned {
    const EMPTY: Self;
    fn from_index(index: usize) -> Self;
    fn index(self) -> Option<usize>;
}
```

Both `DependencySetId` and `SubscriberSetId` implement `SetHandle`. Their struct definitions remain unchanged (they carry semantic meaning in the type system), but the trait eliminates duplicated method bodies.

#### New struct: `SegmentedStore<T, Id>`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentedStore<T: Hash + Clone + PartialEq, Id: SetHandle> {
    items: Vec<T>,
    segments: Vec<Segment>,
    #[serde(skip, default)]
    interner: HashMap<u64, Vec<Id>>,
}
```

The five existing methods (`rebuild_interner_if_needed`, `get`, `insert_from_slice`, `storage_counts`, `live_segment_count`) move onto `impl<T, Id> SegmentedStore<T, Id>`. No behavioral changes.

#### Type aliases

```rust
pub type DependencyEdgeStore = SegmentedStore<DependencyEdge, DependencySetId>;
pub type SubscriberEdgeStore = SegmentedStore<NodeId, SubscriberSetId>;
```

### Files Modified

| File                                                                                                                                   | Change                                                         |
| -------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------- |
| [edge_store.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/graph/edge_store.rs) | Replace two impl blocks with generic struct + two type aliases |

### Lines Saved: ~65

---

## R12: Telemetry Sub-Struct Decomposition

### Problem

[RuntimeTelemetry](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/telemetry.rs) has **46 flat fields**. These same fields are **manually copied** into two additional structs:

| Struct             | File                                                                                                                                                      | Lines     | Fields                      |
| ------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- | --------------------------- |
| `RuntimeTelemetry` | [data/telemetry.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/telemetry.rs)                       | L5â€“L132   | 46                          |
| `GraphMetrics`     | [presentation/metrics.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/presentation/metrics.rs#L7-L55)    | L7â€“L55    | 48 (46 telemetry + 2 extra) |
| `RuntimeMetrics`   | [presentation/metrics.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/presentation/metrics.rs#L118-L166) | L118â€“L166 | 46                          |

The [from_runtime_telemetry](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/presentation/metrics.rs#L57-L113) conversion is **54 lines** of `field: telemetry.field`. Adding a single counter requires edits to all three structs plus the conversion function. Missing a copy is silent data loss.

### Design

Decompose `RuntimeTelemetry` into cohesive sub-structs grouped by domain:

```rust
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluationCounters {
    pub evaluation_calls: u64,
    pub evaluation_nanos: u128,
    pub nodes_evaluated: u64,
    pub nodes_recomputed: u64,
    pub skipped_by_comparator: u64,
    pub suppressed_downstream_propagations: u64,
    pub output_identity_unchanged_count: u64,
    pub memoization_hits: u64,
    pub memoization_misses: u64,
    pub condition_skip_count: u64,
    pub ondemand_deferred_count: u64,
    pub debounce_deferred_count: u64,
    pub evaluation_stack_peak: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvalidationCounters {
    pub invalidation_nodes_visited: u64,
    pub partition_scoped_invalidation_checks: u64,
    pub partition_match_dirty_count: u64,
    pub detail_match_dirty_count: u64,
    pub partition_scope_revert_clean_count: u64,
    pub partition_interner_growth_delta: u64,
    pub partition_aware_recomputations: u64,
    pub keyed_evaluation_count: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionCounters {
    pub transaction_begin_count: u64,
    pub transaction_commit_count: u64,
    pub transaction_rollback_count: u64,
    pub transaction_poison_count: u64,
    pub staged_node_patch_count: u64,
    pub max_touched_nodes_in_txn: u64,
    pub transaction_mark_dirty_candidate_visits: u64,
    pub rolled_back_created_node_count: u64,
    pub rollback_count: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannerCounters {
    pub plans_built: u64, pub stages_built: u64,
    pub tasks_scheduled: u64, pub tasks_pruned_before_execution: u64,
    pub maybe_stale_validation_tasks: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionCounters {
    pub stage_execution_count: u64, pub stage_execution_nanos: u128,
    pub parallel_stage_dispatch_count: u64, pub max_tasks_in_stage: u64,
    pub serial_executor_usage_count: u64, pub parallel_executor_usage_count: u64,
    pub execution_snapshots_built: u64, pub execution_snapshot_nanos: u128,
    pub prepared_evaluations_produced: u64, pub prepared_evaluations_applied: u64,
    pub dependency_capture_updates: u64, pub rewiring_apply_count: u64,
    pub serial_precompute_task_count: u64, pub parallel_precompute_task_count: u64,
    pub stage_precompute_nanos: u128, pub stage_apply_nanos: u128,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageCounters {
    pub gc_epoch_count: u64, pub gc_epoch_nanos: u128,
    pub graph_storage_compaction_count: u64,
    pub graph_storage_dependency_segments_rewritten: u64,
    pub graph_storage_subscriber_segments_rewritten: u64,
    pub graph_storage_snapshot_rewrites: u64,
    pub subscriber_index_rebuild_count: u64,
    pub scratch_reentry_error_count: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointCounters {
    pub event_flushes: u64, pub event_flush_nanos: u128,
    pub checkpoint_flushes: u64, pub checkpoint_flush_nanos: u128,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeTelemetry {
    pub evaluation: EvaluationCounters,
    pub invalidation: InvalidationCounters,
    pub transaction: TransactionCounters,
    pub planner: PlannerCounters,
    pub execution: ExecutionCounters,
    pub storage: StorageCounters,
    pub checkpoint: CheckpointCounters,
}
```

`GraphMetrics` and `RuntimeMetrics` become **views that embed the sub-structs directly** instead of copying field-by-field:

```rust
pub struct GraphMetrics {
    pub evaluation: EvaluationCounters,
    pub invalidation: InvalidationCounters,
    pub planner: PlannerCounters,
    pub execution: ExecutionCounters,
    pub storage: StorageCounters,
    pub partition_interner_size: usize,
}

impl GraphMetrics {
    pub fn from_telemetry(t: &RuntimeTelemetry, interner_size: usize) -> Self {
        Self {
            evaluation: t.evaluation,
            invalidation: t.invalidation,
            planner: t.planner,
            execution: t.execution,
            storage: t.storage,
            partition_interner_size: interner_size,
        }
    }
}
```

### Files Modified

| File                                                                                                                                            | Change                                                         |
| ----------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------- |
| [data/telemetry.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/telemetry.rs)             | Decompose 46-field flat struct into 7 sub-structs + composite  |
| [presentation/metrics.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/presentation/metrics.rs) | Delete 54-line `from_runtime_telemetry`; embed sub-structs     |
| All `graph.telemetry_mut().field += 1` call sites (~40)                                                                                         | Change to `graph.telemetry_mut().invalidation.field += 1` etc. |

### Lines Saved: ~100 (54-line copy function eliminated; metrics structs shrink by ~50 lines)

### Bug Class Eliminated

"Missing field in telemetry copy" â€” impossible, because sub-structs are embedded by value.

---

## R13: Diagnostics Replay Filter Consolidation

### Problem

Multiple replay filter methods in `diagnostics_access.rs` each follow the same pattern: filter `replay_events()` with a predicate, collect, wrap in `ReplaySlice`.

### Design

```rust
impl SignalGraph {
    pub fn replay_where(&self, pred: impl Fn(&ReplayEvent) -> bool) -> ReplaySlice {
        ReplaySlice {
            start: None, end: None,
            frames: self.replay_events().iter().filter(|e| pred(e)).cloned().collect(),
        }
    }
}
```

Existing methods become one-liner wrappers.

### Lines Saved: ~60

---

## R14: Stale Utility Deduplication

### Problem

A `stale_error` helper function is duplicated in two files:

| Function      | File                                                                                                                                       |
| ------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| `stale_error` | [signal_graph.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/graph/signal_graph.rs) |
| `stale_error` | [storage.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/graph/storage.rs)           |

### Fix

Delete one copy. Keep the canonical version in `signal_graph.rs` and import it in `storage.rs`.

### Lines Saved: ~5

---

## Verification

### Existing Test Suite

The `worth-signal` crate has extensive test coverage across multiple test modules:

```bash
# Run all tests (includes ignored tests)
cargo test -p worth-signal --all-features -- --include-ignored
```

Each refactor in this spec is intended as internal restructuring with no behavioral change. If the existing test suite passes after each change, correctness is preserved.

### Per-Refactor Verification Strategy

| Refactor             | Verification                                                                       | Risk                                                                   |
| -------------------- | ---------------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| R1 (Transitions)     | All invalidation and evaluation tests                                              | Medium â€” behavior now centralized, subtle edge cases in scope clearing |
| R2 (PartitionScoped) | All partition-related tests, especially `phase3_partitions` and `phase5_workflows` | Medium â€” multiple matching semantics being unified                     |
| R3 (DependencyKey)   | Existing snapshot and comparison tests                                             | Low â€” named fields, same semantics                                     |
| R4 (ScratchGuard)    | All tests that trigger invalidation or evaluation (they all use scratch)           | Low â€” closure wrapper, same acquire/restore semantics                  |
| R5 (Edge ceremony)   | All dependency/subscriber modification tests                                       | Low â€” internal extraction                                              |
| R6 (Reconciliation)  | Evaluation + dependency capture tests                                              | Medium â€” new API, existing tests run through old API                   |
| R7 (Pipeline)        | All invalidation tests                                                             | Medium â€” structural change to core algorithm                           |
| R11 (SegmentedStore) | Existing edge_store tests must pass unchanged                                      | Low â€” type alias, no behavior change                                   |
| R12 (Telemetry)      | All tests that assert on telemetry counters or metrics                             | Medium â€” field paths change everywhere                                 |
| R13 (Replay)         | Diagnostic replay tests                                                            | Low â€” trivial wrapper                                                  |
| R14 (stale_error)    | Compilation                                                                        | Trivial                                                                |
| R19 (PhaseGuard)     | All evaluation and invalidation tests â€” entry points change                        | High â€” rewrites how the graph is entered for mutation                  |
| R20 (Observation)    | All diagnostic/explain tests, telemetry assertions                                 | Medium â€” methods change from `&mut self` to `&self`                    |
| R21 (Single Source)  | All subscriber-related tests, dependency/subscriber consistency assertions         | High â€” subscriber storage model changes                                |
| R22 (Transactional)  | All transaction/rollback tests, especially multi-step mutation sequences            | High â€” mutation API surface changes                                    |

---

## Sequencing

### Dependency Rules

If the goal is to reach `R8`â€“`R10` as early as possible **without building Phase 2 on weak foundations**, the order has to follow abstraction dependencies rather than cosmetic cleanup.

#### Hard prerequisites for Phase 2

- `R1` must land before `R7` and materially before `R8`
  - planner/execution refactors should not preserve duplicated node-state transition ceremony
  - `R22` is a design constraint on R1 â€” transition methods must be compatible with transactional undo recording
- `R2` must land before `R7`, `R6`, and effectively before `R8`
  - zero-allocation planning and pipeline execution need one canonical partition-matching semantic
- `R3` must land before `R6` and strongly before `R8`
  - cursor/planner work should not be built on anonymous snapshot tuples and ad hoc comparison rules
- `R4` must land before `R7`
  - pass pipelines should not retain manual scratch-lease ceremony
- `R5` and `R6` must land before `R8`/`R9`
  - planner/execution refactors need explicit dependency mutation/reconciliation seams
  - `R21` is a design constraint on R5/R6 â€” edge ceremony must enforce single source of truth
- `R7` should land before `R9`
  - pipeline execution should compose over a decomposed invalidation pipeline, not another monolith
- `R19` does not block Batch C, but it constrains it
  - planner/executor work should avoid widening public mutation entry points in ways that would fight a later phase-guard layer
- `R20` should land before or alongside `R12`
  - telemetry/diagnostic methods must be `&self` from the start
- `R11` is helpful for `R10`, but not a hard prerequisite
  - amortized GC should avoid deep coupling to duplicated store internals so `R11` can still land cleanly afterward if needed

#### Not prerequisites for Phase 2

- `R14` is cleanup only
- `R13` is diagnostics convenience only
- `R12` is observability architecture, but it does not unblock `R8`â€“`R10` (though `R20` constrains its API)

### Recommended Execution Order

#### Batch A â€” Semantic Foundation

These are the minimum structural foundations required before Phase 2 is worth touching.

```text
R1   NodeEntry state machine transitions (with R22 design constraint)
R2   PartitionScoped trait for scope matching
R3   DependencyKey + named snapshot entries
R4   ScratchGuard / with_scratch
```

Why first:

- these changes make wrong state handling, wrong scope matching, and wrong dependency identity harder to write
- `R8` and `R9` built before these would lock in the same weak invariants under a faster engine

#### Batch B â€” Mutation and Pipeline Seams

These expose the core seams the planner/executor refactor needs.

```text
R5   Edge mutation ceremony extraction (with R21 single-source-of-truth constraint)
R6   Declarative dependency reconciliation (with R21 constraint)
R7   Invalidation pass pipeline
```

Why second:

- `R8` should target reconciliation and explicit mutation contracts, not imperative edge surgery
- `R9` should assemble pipelines out of decomposed passes, not monolithic execution and invalidation blobs

#### Batch C â€” Phase 2 Acceleration

Once B is complete, Phase 2 can begin safely.

```text
R8   Zero-allocation planner & prepared cursor
R9   Feature-gated pipeline execution engine
R10  Epoch-driven amortized garbage collection
```

Why here:

- `R8` now has stable dependency identity, scope semantics, state transitions, and mutation seams
- `R9` can assemble over explicit pipeline stages instead of preserving `#cfg` monoliths
- `R10` benefits from the clearer execution/storage seams and should not be designed against duplicated stores if that can be avoided
- `R19` should guide the public shape of this batch, but should not delay internal planner/executor work already underway

#### Batch D â€” Storage and Observability Consolidation

These are still important, but they should no longer block Phase 2.

```text
R11  Unified SegmentedStore<T, Id>
R12  Telemetry sub-struct decomposition (with R20 observation purity constraint)
R20  Observation Purity (land alongside R12 â€” constrains diagnostic API)
R13  Diagnostics replay filter consolidation
R14  stale_error deduplication
```

Why later:

- `R11` is strategically important, especially for `R10`, but it is not required to begin `R8`/`R9`
- `R12` is worthwhile after engine churn settles, otherwise telemetry paths change twice
- `R13` and `R14` are cleanup and should not delay engine work

### Practical Batch Plan

```text
Batch A1 â€” semantic correctness floor
  R1  NodeEntry transitions (design with R22 transactional constraint)
  R2  PartitionScoped trait

Batch A2 â€” dependency identity floor
  R3  DependencyKey + named snapshot entries
  R4  ScratchGuard / with_scratch

Batch B1 â€” mutation seam extraction
  R5  Edge mutation ceremony extraction (design with R21 single-source constraint)
  R6  Declarative dependency reconciliation (design with R21 constraint)

Batch B2 â€” invalidation decomposition
  R7  Invalidation pass pipeline

Batch C1 â€” planner hot path
  R8  Zero-allocation planner & prepared cursor

Batch C2 â€” execution hot path
  R9  Feature-gated pipeline execution engine

Batch C3 â€” lifecycle hot path
  R10 Epoch-driven amortized garbage collection

Batch D â€” cleanup and consolidation
  R11 Unified SegmentedStore<T, Id>
  R12 Telemetry sub-struct decomposition
  R20 Observation Purity (constrains R12 diagnostic API)
  R13 Diagnostics replay filter consolidation
  R14 stale_error deduplication
  R19 PhaseGuard<P> (harden public entry points after Batch C seams stabilize)
  R22 Transactional Mutation (wire up undo recording over R1 transitions)

Batch E â€” Phase 2 Semantic Correctness
  R15 Partition-Aware MaybeStale Validation
  R21 Single Source of Truth (verify subscriber derivation after R5/R6)

Batch F â€” Phase 3 Compile-Time Safety (see signal_compile_time_safety.md)
  R16 Branded NodeRef<'g>
  R17 ScopedVersion witness type
  R18 Private state setters

Batch G â€” Deferred Compile-Time Safety (see signal_compile_time_safety.md)
  R23â€“R34 (see deferred items document)
```

### Recommended Rule

If there is tension between â€œclean every layerâ€ and â€œreach Phase 2 quickly,â€ use this rule:

- do the minimum foundation work that prevents Phase 2 from encoding bad invariants
- begin `R8` immediately after those foundations and mutation seams are in place
- defer cosmetic DRY work and telemetry reshaping until the engine architecture stabilizes

That means:

- do **not** start with `R11`, `R13`, or `R14`
- do **not** defer `R1`, `R2`, or `R3`
- do **not** start `R9` before `R7`
- do **not** start `R10` before the planner/execution direction is clear
- do **not** let late compile-time or typestate ideas derail the current internal Batch C seam work unless they force a public API decision right now

> [!IMPORTANT]
> Run `cargo test -p worth-signal` after **every individual refactor**. Run `cargo test -p worth-signal --all-features -- --include-ignored` once the known mutually-exclusive core-profile test-lane issue is fixed. Do not batch test runs. A regression caught early maps to exactly one change.

---

## Summary

| #   | Refactor                      | Lines Saved | Bug Class Eliminated                     |
| --- | ----------------------------- | ----------- | ---------------------------------------- |
| R1  | `NodeEntry::transition_*`     | 50          | Inconsistent state transitions           |
| R2  | `PartitionScoped` trait       | 80          | Scope matching semantic drift            |
| R3  | `DependencyKey` + named entry | 60          | Positional access, comparison drift      |
| R4  | `ScratchGuard` RAII           | 20          | Unreturned scratch leases                |
| R5  | Edge mutation ceremony        | 100         | Forgotten compact/wire step              |
| R6  | Declarative reconciliation    | 40          | Stale edge retention                     |
| R7  | Invalidation pipeline         | 30          | Untestable monolith                      |
| R8  | Zero-Allocation Planner       | 150         | Transaction-rate GC pressure             |
| R9  | Pipeline Execution Engine     | 120         | Conditional compilation rot              |
| R10 | Amortized Garbage Collection  | 80          | Latency jitter                           |
| R11 | `SegmentedStore<T, Id>`       | 65          | Store divergence                         |
| R12 | Telemetry sub-structs         | 100         | Missing field in telemetry copy          |
| R13 | `replay_where`                | 60          | N/A (DRY)                                |
| R14 | `stale_error` dedup           | 5           | N/A (DRY)                                |
| R15 | `PartitionVersionMap`         | -100        | PartitionÃ—MaybeStale evaluation mismatch |
| R16 | Branded `NodeRef<'g>`         | -50         | Topological Dementia (ghost edges)       |
| R17 | `ScopedVersion` witness       | -30         | Granularity False Negatives              |
| R18 | Private state setters         | 0           | State Machine Fracture                   |
| R19 | `PhaseGuard<P>`               | -20         | Cross-Epoch Re-entrancy                  |
| R20 | Observation Purity            | -10         | Observation Contamination                |
| R21 | Single Source of Truth         | -40         | Representational Drift                   |
| R22 | Transactional Mutation        | -30         | Rollback Amnesia                         |
|     | **Total**                     | **~580**    | **18 bug classes**                       |
