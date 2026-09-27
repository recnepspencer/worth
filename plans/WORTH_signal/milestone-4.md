# worth-signal Milestone 4

> **Status:** Active engineering spec
>
> **Roadmap parent:** [performance.md](./performance.md)
>
> **Related implementation surfaces:**
> - [entry.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/node/entry.rs)
> - [trace.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/trace.rs)
> - [entries.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/storage/entries.rs)
> - [effect.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/runtime/effect.rs)
> - [apply.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/evaluation/engine/apply.rs)
> - [stage.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/apply/stage.rs)
> - [mod.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/model/mod.rs)
> - [milestone-3.md](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/milestone-3.md)
> - [milestone-2.md](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/milestone-2.md)

## Goal

Milestone 4 converts worth-signal's node/runtime storage from a semantically
clean but still pointer-broad object model into a locality-oriented,
proof-carrying storage architecture with compile-time separation between:

- hot authoritative node state
- hot derived operational artifact state
- warm operational-but-not-inner-loop state
- cold retained diagnostic richness
- serialized authority images
- rollback/branch-restore deltas

The implementation goal is not "do a storage refactor" or "make structs
smaller." The goal is to make the hot staged execution lanes structurally
unable to pay for broad node access, cold payload traversal, or semantically
generic update forms when the runtime already knows a narrower truth.

This milestone must preserve:

- semantic truth
- rollback truth
- branch restore truth
- merge truth
- replay truth
- retained vs reconstructed artifact parity
- serial vs parallel equivalence

while making the hot path credible for chip-simulator-grade churn and
aerospace-kernel-grade locality pressure.

## Current Closure State

The milestone remains active overall, but the first two closure gates now have
named implementation artifacts and code certification surfaces:

- Gate 1, persistence and restore contract closure
  status: closed
  evidence: [milestone-4-access-matrix.md](./milestone-4-access-matrix.md),
  [checkpoint_image.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/node/checkpoint_image.rs),
  [mod.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/state/mod.rs)
- Gate 2, access-discipline closure
  status: closed
  evidence: [milestone-4-access-matrix.md](./milestone-4-access-matrix.md),
  [milestone-4-interior-heat-audit.md](./milestone-4-interior-heat-audit.md),
  [phase1_api.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/tests/phase1_api.rs)
- Gate 3, artifact and node storage split closure
  status: closed
  evidence: [trace.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/trace.rs),
  [entry.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/node/entry.rs),
  [entries.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/storage/entries.rs),
  [observer.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/runtime/observer.rs),
  [serial_batch.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/apply/serial_batch.rs),
  [stage.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/apply/stage.rs),
  [semantic/mod.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/semantic/mod.rs),
  [phase1_api.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/tests/phase1_api.rs)

The remaining gates were executed in order on top of the now-closed
persistence, access-discipline, storage-split, and proof-bearing execution
boundaries. Gates 1 through 6 are now closed.

- Gate 4, proof-bearing execution closure
  status: closed
  evidence: [proof.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/proof.rs),
  [trace.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/trace.rs),
  [workspace.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/apply/workspace.rs),
  [serial_batch.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/apply/serial_batch.rs),
  [stage.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/apply/stage.rs),
  [semantic/mod.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/semantic/mod.rs),
  [model/mod.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/model/mod.rs),
  [phase1_api.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/tests/phase1_api.rs)

Gate 4 is closed because the planner/apply/finalize execution boundary now
keeps proof-carrying forms intact instead of collapsing back to generic packet
or field-bag representations:

- stage-owned pending dependency snapshots no longer collapse back to a generic
  `SnapshotBatchCommit` before publication inside the planner hot path
- [workspace.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/apply/workspace.rs)
  stores `ClassifiedSnapshotBatchCommit` in `StageScratch`
- [serial_batch.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/apply/serial_batch.rs)
  classifies pending stage snapshots before the finalize boundary instead of
  carrying only a generic batch form
- [stage.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/apply/stage.rs)
  commits the already-classified proof form through
  `apply_classified_snapshot_batch_commit`
- [proof.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/proof.rs)
  seals stable-shape and replacement snapshot proof entry fields so those
  proof-bearing entry forms are no longer WORTHable by public struct literal
- [trace.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/trace.rs)
  no longer derives `Default` for `RuntimeArtifactFinalizeImage`, which
  prevents synthetic construction of a finalize carrier that claims runtime
  capture without runtime capture having occurred
- [model/mod.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/model/mod.rs)
  seals `LoweredTaskExecution`, `LoweredTask`, and `LoweredStagePlan` behind
  constructors, accessors, and owned decomposition methods instead of leaving
  planner execution as open field bags
- [semantic/mod.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/semantic/mod.rs)
  seals `SemanticTaskUpdate`, `SemanticSegment`, and `StageSemanticBatch`
  behind constructors / owned transitions instead of open packet assembly
- [serial_batch.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/apply/serial_batch.rs)
  establishes `ReadySerialFinalizeBatch` through a constructor after
  stage-width and snapshot-ownership checks, instead of open struct assembly
- [workspace.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/apply/workspace.rs)
  seals grouped-apply workspace packets behind constructors and owned
  decomposition methods, including `ConcurrentWorkerInput`,
  `ConcurrentApplyGroupInput`, `GroupLocalTaskCommit`, `GroupLocalApplyPacket`,
  and `StageScratch`
- [stage.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/apply/stage.rs)
  constructs and consumes lowered execution, semantic publication, finalize
  readiness, and grouped-apply workspace packets through those transition
  methods instead of open field assembly, so both serial and parallel hot paths
  now follow the same proof-bearing execution discipline

Gate 3 is closed because the split now exists as runtime structure instead of
only classification intent:

- [trace.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/trace.rs)
  separates `RuntimeArtifactHot`, `RuntimeArtifactWarm`, and the compatibility
  carrier `RuntimeArtifactState`, and adds the narrower
  `RuntimeArtifactFinalizeImage` for planner/finalize use
- [entry.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/node/entry.rs)
  structurally splits `NodeEntry` into `NodeHotData`, `NodeWarmData`, and boxed
  cold payload while preserving the serialized compatibility boundary
- [entries.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/storage/entries.rs)
  exposes explicit hot, warm, and finalize-image accessors instead of forcing
  hot callers through broad node entry assembly
- [serial_batch.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/apply/serial_batch.rs),
  [stage.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/apply/stage.rs),
  and [semantic/mod.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/semantic/mod.rs)
  no longer carry broad `RuntimeArtifactState` snapshots through the main
  apply/finalize planner path
- [observer.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/runtime/observer.rs)
  surfaces node and artifact lane inline-size inventory so the split is
  observable without pretending the node-side lane separation is already a
  fully independent physical store

Gate 5 is now closed, with the following explicit closure work landed:

- [patch_buffer.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/patch_buffer.rs)
  now stores canonical `CheckpointNodeImage` rollback packets instead of raw
  `NodeEntry` clones, so transaction rollback restores touched nodes through an
  explicit authority-image boundary
- [entries.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/storage/entries.rs)
  now exposes `node_checkpoint_image`,
  now exposes `create_node_from_checkpoint_image` and
  `replace_entry_from_checkpoint_image` so rollback and merge paths can operate
  on checkpoint-carried authority images without smuggling broad entry objects
  across the boundary
- [execute.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/merge/execute.rs)
  now captures and reapplies source/target authority through checkpoint node
  images when adopting or rewriting merge candidates, and applies carry-policy
  mutation directly to the checkpoint image instead of bouncing back through
  `NodeEntry::from_checkpoint_image(...)`
- [merge_runtime.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/branching/merge_runtime.rs)
  now rewrites existing-target reconciliation through checkpoint images and
  derives merge comparability / lineage / merge authority from explicit hot and
  warm artifact lane accessors instead of broad `RuntimeArtifactState` reads
- [phase1_api.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/tests/phase1_api.rs)
  now carries a Gate 5 regression barrier asserting that rollback and merge use
  checkpoint node images as their authority-transfer seam and that merge
  planning derives comparable state from explicit hot/warm lanes instead of
  broad runtime artifact reads
- [artifacts.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/diagnostics_access/artifacts.rs)
  now keeps snapshot restore proof-bearing by carrying the classified
  checkpoint-carried dependency snapshot rebuild batch in
  `SnapshotRestorePlan` and rebuilding dependency snapshot state through
  `apply_classified_snapshot_batch_commit` rather than the generic snapshot
  commit surface or late execution-time reclassification, while rewiring
  diagnostics pull scoped versions through a narrowed version accessor instead
  of a broad entry read
- [snapshotting.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/branching/snapshotting.rs)
  now consumes that already-classified restore-plan batch on the runtime branch
  restore path as well, preventing restore execution from silently
  reclassifying the checkpoint batch late
- [branches.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/branching/branches.rs),
  [runtime_state.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/runtime_state.rs),
  [snapshotting.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/branching/snapshotting.rs),
  and [merge_runtime.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/branching/merge_runtime.rs)
  now keep branch authority / derived / ancestry / mutation-ledger state behind
  mediated `BranchState` and `BranchAncestryState` accessors instead of open
  field reach-through, and stored branch state now keys itself from sealed
  ancestry rather than a caller-supplied branch id so restore/merge/fork paths
  cannot silently file a branch state under the wrong branch key
- active branch restore and active-target merge no longer duplicate full
  `BranchState` authority just to preserve ancestry and mutation-journal truth;
  [branches.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/branching/branches.rs)
  now keeps branch runtime metadata separately from stored inactive full-state
  payloads so the active runtime can move authority once and retain
  branch-local proof state without a second heavyweight clone
- Gate 5 certification evidence now includes:
  `cargo test -p worth-signal phase5_state -- --nocapture`,
  `cargo test -p worth-signal merge_adoption -- --nocapture`,
  `cargo test -p worth-signal --lib`,
  `cargo test -p worth-signal --lib -- --test-threads=1`,
  `cargo test -p worth-signal performance_profiles -- --ignored --nocapture --test-threads=1`,
  `cargo check -p worth-signal --features parallel`,
  and `cargo test -p worth-signal --lib --features parallel`
- allocation and footprint certification surface now exists in
  [performance_support.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/tests/performance_support.rs):
  the ignored perf harness runs under a process-wide `stats_alloc`
  instrumentation boundary, serializes sample execution with `PERF_ALLOC_LOCK`,
  and emits per-sample `allocation_metrics` including `allocated_bytes`,
  `deallocated_bytes`, `live_bytes`, and `peak_live_bytes` alongside elapsed
  wall-clock metrics for milestone certification

## Closure Model

Milestone 4 is intentionally broad in architecture and intentionally narrow in
execution sequencing. It is not one continuous refactor. It is a parent
milestone composed of multiple closure gates that must be completed in order.

The closure gates are:

1. persistence and restore contract closure
2. access-discipline closure
3. artifact and node storage split closure
4. proof-bearing execution closure
5. rollback / branch / merge synchronization closure
6. transitional compatibility closure

The governing rule is:

```rust
A later closure gate may not begin implementation until the prior gate is
certification-complete, except for explicitly enumerated dependencies recorded
in this milestone document.
```

This is not bureaucratic process. It is the mechanism that prevents Milestone 4
from bleeding across storage, apply/finalize, rollback, persistence, and merge
all at once until it becomes emotionally and architecturally unfinishable.

## Why This Milestone Exists

Milestones 1 through 3 move the runtime toward a batch-native and proof-carrying
execution model, but the current storage surfaces still leave too much locality
on the floor:

- [NodeEntry](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/node/entry.rs)
  is still the broad authority aggregate for many operations whose actual
  read/write set is much smaller
- [RuntimeArtifactState](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/trace.rs)
  is conceptually hot, but still contains fields whose dominant consumers are
  warm or cold
- the serial hot lane still reaches through broad entry access for state,
  versions, snapshot ids, and artifact facts that could live in a tighter
  locality-oriented store
- rollback and branch-restore truth currently inherit the broad object layout
  rather than an explicit split-store contract
- persistence derives from in-memory layout more directly than this milestone
  can tolerate
- hot-path access discipline is still too dependent on convention instead of
  sealed access surfaces

The current code already points in the right direction. `NodeColdData` is boxed
away from the main entry, hot/cold artifact language already exists, and
Milestone 3 establishes stronger proof-carrying snapshot paths. The problem is
that the storage topology still allows hot-path code to borrow semantically
broad runtime objects when it should be consuming compact, phase-appropriate,
proof-bearing views.

Milestone 4 closes that gap.

## Adversarial Constraint

Under staged serial and staged parallel churn with rotating-window pressure,
stable-shape dependency updates, structural replacement bursts, suppression
cascades, branch restore, merge reconciliation, and retained diagnostics still
enabled:

- apply, finalize, suppression, and snapshot commit must operate primarily on
  compact index-addressed hot state
- hot loops must not traverse cold retained payloads or pointer-rich warm
  metadata by default
- warm or cold escalation must be explicit, measurable, and uncommon on the
  representative hot lanes
- rollback and restore must remain exact even after storage is split into
  multiple physical lanes
- persistence compatibility must survive the in-memory layout transition
- a value that has already been proven stable-shape, commit-ready, or
  finalize-ready must carry that proof in its type and must not collapse back
  into an unclassified generic form

The naive failure mode is a runtime that is asymptotically reasonable but still
performs like an app server because every hot operation rehydrates broad
per-node objects, re-branches over semantically distinct cases, and drags warm
or cold payload adjacency through the cache footprint.

## Architectural Position

This milestone is governed by three hard laws from the coding guidelines:

1. Storage topology must reflect workload traversal, not abstract object
   convenience.
2. Hot paths must consume proof-bearing forms instead of re-deriving facts.
3. Invalid broad access from hot modules must be made structurally impossible
   or extremely narrow, not merely discouraged.

That means:

- no trait-object dispatch in hot storage access
- no "temperature taxonomy" that exists only in docs
- no code-review-only discipline around broad entry access
- no in-memory layout change without an explicit persistence bridge
- no split-store model without an explicit rollback synchronization contract

## What This Milestone Is Not

This milestone is not:

- a full semantic redesign of node authority
- a rewrite of diagnostics retention rules
- a replacement of the snapshot proof model introduced in Milestone 3
- a speculative vectorization pass
- a permission slip to erase semantic distinctions in the name of locality

The milestone changes representation, access discipline, and proof carriage. It
does not change what the runtime means.

## Target Runtime Topology

The runtime will move to a three-lane in-memory storage model plus a separate
serialized authority image contract:

```rust
pub struct NodeStorage {
    hot: NodeHotStore,
    warm: NodeWarmStore,
    cold: NodeColdStore,
}

pub struct SerializedNodeImage {
    // canonical persisted shape; not equal to in-memory SoA layout
}
```

The categories are intentionally different from the current `NodeEntry` layout:

- `hot` is for fields touched in apply, finalize, suppression, invalidation, and
  snapshot commit on representative workloads
- `warm` is for operational state that is authoritative or operationally
  meaningful, but not required in the tightest loops
- `cold` is for retained diagnostics, causality, trace stamps, and other
  explanation/provenance-facing payloads
- `SerializedNodeImage` is the persistence contract for checkpoints, restore
  images, branch snapshots, and any other stored authority boundary

In-memory topology and persistence topology are explicitly decoupled. The SoA
split is an execution optimization. It must not silently become the persisted
schema contract.

## Canonical Classification Rule

Milestone 4 still requires a full field inventory and classification pass, but
the classification is an engineering analysis artifact, not the enforcement
mechanism. We will maintain a field inventory document and internal tables that
classify each field into one of these categories:

```rust
enum StorageClass {
    HotAuthoritative,
    HotDerivedOperational,
    WarmAuthoritative,
    WarmDerivedOperational,
    ColdDerivedDiagnostic,
    SerializedAuthorityOnly,
}
```

This classification exists to drive migration and audit. It does **not** by
itself enforce anything. Enforcement will come from:

- concrete store/view types
- module visibility
- sealed constructors for proof-bearing forms
- hot-path APIs that cannot return broad entry objects

If a field inventory row is not backed by one of those mechanisms, it is only a
design note and must be treated as such.

Classification is governed by the following rule:

```rust
A field's default lane is determined by semantic role first and representative
workload majority second. Rare access from a hotter lane does not promote the
field. Rare access is served by explicit escalation. A field moves hotter only
when both:
1. representative hot workloads touch it frequently enough to matter, and
2. its internal representation is itself locality-compatible.
```

Corollaries:

- workload-specific exceptions do not automatically reclassify a field
- merge-heavy or restore-heavy reads of a warm field do not by themselves make
  it hot
- conditional fields such as `dirty_partition_scopes` remain governed by this
  rule rather than by implementation convenience
- hot/warm/cold classification is reviewed at closure gates, not continuously
  re-litigated during implementation

## Initial Field Inventory

The following classification is the initial expected split based on the current
hot surfaces in:

- [entry.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/node/entry.rs)
- [trace.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/trace.rs)
- [effect.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/runtime/effect.rs)
- [apply.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/evaluation/engine/apply.rs)

`NodeEntry`-owned fields expected to live in the hot lane:

- `state`
- `dirty_aspects`
- `dirty_partition_scope_aspects`
- `aspect_version_header`
- `dependencies_id`
- `subscribers_id`
- `dep_snapshot_id`

`NodeEntry`-owned fields expected to live in the warm lane:

- `eval_config`
- `tombstoned`
- `aspect_version_overrides`
- `dirty_partition_scope_payload`
- `runtime_artifact_state`

`NodeEntry`-owned fields expected to live in the cold lane:

- retained artifact record
- causality metadata
- execution trace stamp

`RuntimeArtifactState` facts expected to become `RuntimeArtifactHot`:

- `output_hash`
- `output_change`
- `recomputed`
- `dependency_count`
- `meaningful_input_changes`
- `changed_partition_count`
- `propagation_suppressed`
- `changed_scopes`

`RuntimeArtifactState` facts expected to become `RuntimeArtifactWarm`:

- `output_identity`
- `continuity_token`
- `memoized_origin`
- `reuse_basis`
- `reuse_origin`
- `reuse_boundary_authority`
- `lineage_artifact_id`
- `merge_authority`

This list is provisional until the access matrix is complete. No field moves to
hot storage purely because it is "small." It moves because hot-path access
frequency and semantic role justify it.

## Interior Heat Audit

Field classification alone is insufficient. A field can be frequently touched
and still be a bad hot-lane resident if its interior representation is broad,
pointer-rich, variably shaped, or branch-heavy.

Any field promoted into hot storage must therefore pass an interior heat audit.
At minimum, this audit must examine:

- `PartitionVersionMap`
- dirty partition scope payload shape
- dependency and subscriber handle dereference patterns
- any hot artifact substructure promoted into `RuntimeArtifactHot`

The governing rule is:

```rust
No field may be promoted into the hot lane solely by outer access frequency if
its interior representation remains pointer-rich, variably shaped, or
branch-heavy in a way that defeats locality.
```

For each audited type, the milestone must record:

- whether the type is already locality-compatible
- whether the type needs an internal split or narrower header form
- whether the type should remain warm with explicit hot-lane escalation instead

This audit is a required input to Phase 3, not optional follow-up work.

## Concrete In-Memory Types

The concrete storage now converges toward the following physically split lane
shapes:

```rust
pub struct NodeArena {
    nodes: Vec<Slot>,
    hot: Vec<Option<NodeHotData>>,
    warm: Vec<NodeWarmData>,
    cold: Vec<Option<Box<NodeColdData>>>,
}

pub struct NodeHotData {
    state: NodeState,
    dirty_aspects: AspectMask,
    dirty_partition_scope_aspects: AspectMask,
    aspect_version_header: AspectVersionHeader,
    dependencies_id: DependencySetId,
    subscribers_id: SubscriberSetId,
    dep_snapshot_id: DependencySnapshotId,
}

pub struct NodeWarmData {
    tombstoned: bool,
    aspect_version_overrides: PartitionVersionOverrides,
    dirty_partition_scope_payload:
        SmallVec<[(Aspect, PartitionSubscription); HOT_VEC_INLINE_CAPACITY]>,
    runtime_artifact_state: Option<RuntimeArtifactState>,
    eval_config: NodeEvaluationConfig,
}

pub struct NodeColdData {
    retained_artifact: Option<RetainedDiagnosticArtifact>,
    causality: Option<CausalityMetadata>,
    execution_trace: Option<ExecutionTraceStamp>,
}
```

The important landed property is physical lane separation:

- slot metadata is no longer the same object as node payload
- hot, warm, and cold node facts now live in separate index-addressed lane
  arrays
- fixed-width hot headers stay in `NodeHotData`
- partition-version overrides and scoped-dirty payloads stay in the warm lane
  behind explicit escalation

These shapes are still not a license to treat every inner field as equally hot.
In particular, `Vec<Option<T>>` is acceptable as a lane representation but is
not automatically the final locality-hardened answer for sparse or skewed
artifact presence.

Candidate final encodings for optional side-lane data include:

- `Vec<Option<T>>` where density and access skew justify it
- dense side-store plus presence bitset
- packed side arena keyed by node slot
- small inline header plus side payload

The governing rule is:

```rust
Milestone 4 does not commit to a final optional-field physical encoding until
presence density and access skew are measured on representative workloads.
```

The hot artifact structure should be split from its warm companion:

```rust
bitflags! {
    pub struct RuntimeArtifactTruthFlags: u8 {
        const RECOMPUTED = 0b0000_0001;
        const PROPAGATION_SUPPRESSED = 0b0000_0010;
    }
}

bitflags! {
    pub struct RuntimeArtifactWarmPresenceFlags: u8 {
        const HAS_OUTPUT_IDENTITY = 0b0000_0001;
        const HAS_CONTINUITY_TOKEN = 0b0000_0010;
        const HAS_REUSE_BOUNDARY_AUTHORITY = 0b0000_0100;
        const HAS_LINEAGE_KEY = 0b0000_1000;
    }
}

pub struct RuntimeArtifactHot {
    pub output_hash: StableHashValue,
    pub output_change: OutputChange,
    pub recomputed: bool,
    pub dependency_count: u32,
    pub meaningful_input_changes: u32,
    pub changed_partition_count: u32,
    pub propagation_suppressed: bool,
    pub changed_scopes: CompactChangedScopeProof,
}

pub struct RuntimeArtifactWarm {
    pub output_identity: Option<OutputIdentity>,
    pub continuity_token: ContinuityAuthorityToken,
    pub memoized_origin: MemoizedResultOrigin,
    pub reuse_basis: ReuseOperationalBasis,
    pub reuse_origin: ReuseOrigin,
    pub reuse_boundary_authority: Option<ReuseBoundaryAuthority>,
    pub lineage_artifact_id: ArtifactTransitionKey,
    pub merge_authority: ArtifactMergeAuthority,
}
```

The important landed rule is still mandatory even though the final
representation uses separate typed fields instead of bitflags: operational truth
and warm-presence semantics must remain physically and semantically separated.

## Authority and Derivation Contract

Milestone 4 must preserve the authority/derivation boundary explicitly:

- node state, aspect versions, dependency set handles, subscriber set handles,
  snapshot ids, tombstone state, and evaluation config are authoritative runtime
  state
- hot artifact facts are derived operational truth retained to serve runtime
  semantics
- warm artifact metadata is derived operational support state
- cold retained artifacts are derived diagnostic richness
- serialized authority images are the canonical persistence boundary

The critical rule is:

```rust
Destroying warm and cold stores must not destroy the ability to restore
authoritative node truth.
```

The separate but equally important rule is:

```rust
Destroying hot artifact state may degrade hot reuse/suppression continuity, but
must not make authoritative node state non-reconstructable.
```

Any design decision that confuses these categories is architecturally invalid.

## Persistence Compatibility Bridge

This spec adds a required Phase 0 before any in-memory split is allowed to
land.

The current runtime derives `Serialize` / `Deserialize` on broad node-facing
types. A direct SoA split would otherwise make persistence format an accidental
casualty of an execution optimization. That is not acceptable.

Milestone 4 therefore introduces a canonical persistence boundary:

```rust
pub struct SerializedNodeImage {
    pub state: NodeState,
    pub dirty_aspects: AspectMask,
    pub dirty_partition_scopes: Vec<(Aspect, PartitionSubscription)>,
    pub aspect_versions: PartitionVersionMap,
    pub dependencies_id: DependencySetId,
    pub subscribers_id: SubscriberSetId,
    pub dep_snapshot_id: DependencySnapshotId,
    pub tombstoned: bool,
    pub runtime_artifact_hot: Option<SerializedRuntimeArtifactHot>,
    pub runtime_artifact_warm: Option<SerializedRuntimeArtifactWarm>,
    pub retained_artifact: Option<RetainedDiagnosticArtifact>,
    pub causality: Option<CausalityMetadata>,
    pub execution_trace: Option<ExecutionTraceStamp>,
    pub eval_config: NodeEvaluationConfig,
}
```

The exact struct may differ, but the contract is fixed:

- persistence serialization happens through explicit image types, not through
  direct serialization of split stores
- loading from old broad layouts and loading from new image layouts must both be
  supported until migration closure
- `NodeEntry` may remain as a compatibility assembly surface during migration,
  but persisted images must not depend on the long-term in-memory layout

Checkpoint and branch image readers/writers must therefore be updated before the
hot-store split lands.

This phase is a hard closure gate. No hot-lane storage split, visibility
narrowing, or apply/finalize migration work may land ahead of it except for
strictly additive bridge scaffolding.

## Rollback and Branch-Restore Contract

The split-store architecture changes physical storage, so rollback and branch
restore need an explicit cross-lane synchronization contract.

Rollback and restore must not operate as "restore whichever fields the current
code happened to touch." They must restore coherent per-node store state through
a lane-aware delta type:

```rust
pub struct NodeStoreDelta {
    pub node: NodeId,
    pub hot: Option<NodeHotDelta>,
    pub warm: Option<NodeWarmDelta>,
    pub cold: Option<NodeColdDelta>,
}

pub struct NodeStoreCheckpointImage {
    pub hot: NodeHotImage,
    pub warm: NodeWarmImage,
    pub cold: NodeColdImage,
}
```

Contractually:

- rollback granularity remains per semantic operation, not per storage field
- a transaction restoring node state must restore all affected lanes in one
  deterministic delta application
- branch restore and merge adoption must either restore all required lanes or
  explicitly reconstruct omitted derived lanes from authority according to the
  existing branch/persistence truth model
- no lane may be restored from a different logical version of the node than the
  others

If rollback can restore `NodeHotStore` but accidentally leave `NodeWarmStore`
or `NodeColdStore` at a different logical revision, the split-store design is
invalid.

## Merge Contract

Merge truth is preserved by explicit lane semantics, not by assuming all
retained runtime state is equally authoritative.

The merge contract for split storage is:

- hot authoritative node state remains canonical for merge truth
- warm state is merge-reconstructable by default unless a specific warm field is
  explicitly declared merge-semantic
- cold retained richness is never authoritative for merge truth
- when merge parents differ in warm or cold richness, the merge path must
  explicitly choose one of:
  1. preserve one parent's retained richness,
  2. drop and reconstruct from authority and hot operational truth, or
  3. adopt by an explicit merge-authority rule

The milestone must therefore document which warm fields, if any, are
merge-semantic. Everything else is treated as reconstructable support state.

`merge_authority` is the existing clue that such semantics exist, but Milestone
4 must make the storage implications explicit:

- whether warm artifact continuity can be reconstructed during merge
- whether any continuity metadata is required for canonical merge adoption
- whether conflicting warm state is preserved, dropped, or re-derived

Merge-heavy workloads are therefore part of the required access matrix and
certification plan, not a named-but-deferred concern.

## Access Discipline

The hot path will not use trait objects or broad per-node aggregate access.
Hot-lane storage access must be concrete and index-addressed.

The core access surfaces should look like:

```rust
pub struct HotNodeView<'a> {
    pub node: NodeId,
    pub state: &'a NodeState,
    pub dirty_aspects: &'a AspectMask,
    pub dirty_partition_scopes: &'a DirtyScopeInlineSet,
    pub aspect_versions: &'a PartitionVersionMap,
    pub dependencies_id: &'a DependencySetId,
    pub subscribers_id: &'a SubscriberSetId,
    pub dep_snapshot_id: &'a DependencySnapshotId,
    pub artifact_hot: Option<&'a RuntimeArtifactHot>,
}

pub struct HotNodeViewMut<'a> { /* concrete mutable references */ }
pub struct WarmNodeView<'a> { /* concrete warm references */ }
pub struct ColdNodeView<'a> { /* concrete cold references */ }
```

And the owning store should expose concrete methods:

```rust
impl NodeStorage {
    #[inline]
    pub fn hot(&self, node: NodeId) -> Result<HotNodeView<'_>, SignalError> { ... }

    #[inline]
    pub fn hot_mut(&mut self, node: NodeId) -> Result<HotNodeViewMut<'_>, SignalError> { ... }

    #[inline]
    pub fn warm(&self, node: NodeId) -> Result<WarmNodeView<'_>, SignalError> { ... }

    #[inline]
    pub fn cold(&self, node: NodeId) -> Result<ColdNodeView<'_>, SignalError> { ... }
}
```

The important property here is not the exact struct shape. The important
property is:

- concrete accessors
- no vtable dispatch in hot access
- no broad `NodeEntry` borrow required for hot loops
- no silent warm/cold adjacency when a function asked only for hot access

Traits may still exist above the hot lane for testing or non-hot orchestration,
but they are not the primary hot-path abstraction.

## Visibility Enforcement

This milestone rejects code-review-only enforcement. Broad entry access must be
structurally narrowed.

The end-state target is:

- hot modules no longer import or call broad `get_entry()` / `get_entry_mut()`
  accessors
- broad entry accessors become visibility-restricted to non-hot administrative,
  serialization, or migration modules
- explicit escape hatches, if any, are named as such and live outside the hot
  execution modules

The visibility direction should converge toward:

```rust
pub(in crate::data) fn get_entry(...)
pub(in crate::data) fn get_entry_mut(...)
```

or a similarly narrow boundary, with hot execution code consuming `NodeStorage`
views instead.

If the broad accessors remain effectively public to the same hot modules, the
milestone is not architecturally closed.

## Proof-Carrying Type Model

Milestone 4 builds on Milestone 3 by extending proof-carrying types into the
storage and execution boundaries.

Facts that are expensive, correctness-sensitive, or phase-sensitive must be
carried structurally once proven.

Representative proof-carrying forms:

```rust
pub struct StableShapeSnapshotProof { /* sealed */ }
pub struct ReplacementShapeSnapshotProof { /* sealed */ }

pub struct ProvenStableShapeCommit {
    node: NodeId,
    previous_snapshot_id: DependencySnapshotId,
    proof: StableShapeSnapshotProof,
    versions: VersionVector,
    delta: SnapshotDeltaRecord,
}

pub struct ProvenReplacementCommit {
    node: NodeId,
    previous_snapshot_id: DependencySnapshotId,
    update: ReplacementSnapshotUpdate,
    delta: SnapshotDeltaRecord,
}

pub enum ProvenSnapshotCommit {
    Stable(ProvenStableShapeCommit),
    Replacement(ProvenReplacementCommit),
}

pub struct HotArtifactBeforeImage { /* compact hot-only snapshot */ }
pub struct HotNodeBeforeImage { /* compact hot-only snapshot */ }

pub struct LoweredHotTask {
    pub node: NodeId,
    pub before: HotNodeBeforeImage,
    pub prepared: PreparedEvaluation,
    pub dependency_plan: ProvenDependencyPlan,
    pub artifact_policy: ResolvedArtifactPolicy,
}

pub struct AppliedHotTask {
    pub node: NodeId,
    pub before: HotNodeBeforeImage,
    pub after: HotNodeBeforeImage,
    pub verdict: EvaluationVerdict,
    pub snapshot_commit: Option<ProvenSnapshotCommit>,
    pub semantic_seed: CompactSemanticSeed,
}

pub struct FinalizeReadyBatch {
    stage_index: u32,
    tasks: Vec<AppliedHotTask>,
}
```

The constructor rules are mandatory:

- proof-bearing constructors are sealed to the proving module
- no downstream module may synthesize "stable-shape" or "finalize-ready" forms
  by assembling raw fields
- a function that accepts `FinalizeReadyBatch` must never need to re-check
  whether hot apply already completed
- a function that accepts `ProvenStableShapeCommit` must never branch back into
  replacement semantics

This is the Milestone 4 application of Law 41: a type must encode what has been
proven, not merely what fields happen to be present.

## Snapshot Commit Contract

Milestone 3 made stable-shape snapshot handling stronger. Milestone 4 requires
that the storage path stop collapsing back into generic forms inside the commit
lane.

This section distinguishes two different concerns that must not be conflated:

- authoritative snapshot commit
- commit-adjacent continuity, artifact, and reporting assembly

The target is:

```rust
authoritative snapshot commit == HotOnly where structurally possible
commit-adjacent continuity/report assembly == explicit HotPlusWarm escalation
when required
```

If continuity, durable audit, or artifact-reporting boundaries require warm
state, that is not a failure. It becomes a separately named post-commit or
side-commit phase rather than an unmodeled leak into authoritative commit.

The batch commit contract becomes:

```rust
pub struct StableShapeCommitBatch {
    entries: Vec<ProvenStableShapeCommit>,
}

pub struct MixedShapeCommitBatch {
    stable: Vec<ProvenStableShapeCommit>,
    replacement: Vec<ProvenReplacementCommit>,
}
```

And the graph/storage API should converge toward:

```rust
fn apply_stable_shape_commit_batch(
    &mut self,
    batch: StableShapeCommitBatch,
) -> Result<(), SignalError>;

fn apply_mixed_shape_commit_batch(
    &mut self,
    batch: MixedShapeCommitBatch,
) -> Result<(), SignalError>;
```

The commit executor is allowed to optimize around stable-shape-first behavior
because the batch type itself already encodes that classification. Late
re-classification of generic commit enums inside the hot path is a design
failure.

## Apply / Finalize Contract

The serial hot path should lower into compact, phase-bearing batch forms rather
than broad task packets with incidental rich fields:

```rust
pub struct SerialHotApplyBatch {
    stage_index: u32,
    authority_policy: AuthorityPolicy,
    maintenance_strategy: ResolvedMaintenanceStrategy,
    dirty_delta: StructuralDelta,
    tasks: Vec<LoweredHotTask>,
}

pub struct HotApplyResult {
    stage_index: u32,
    tasks: Vec<AppliedHotTask>,
    pending_snapshots: Option<StableOrMixedBatch>,
}
```

The critical rule is that `before_artifact_state: Option<RuntimeArtifactState>`
must not remain the default currency for hot apply/finalize. That broad value
is too semantically rich and too easy to widen accidentally. It should be
replaced by compact hot before-images plus explicit warm/cold escalation where
needed.

Semantic finalize may still produce rich public records, but the input boundary
to finalize must be narrowed first.

## Explicit Warm / Cold Escalation

This milestone needs an explicit escalation model. A function that needs more
than hot state must say so in its API.

Representative access classes:

```rust
pub enum AccessClass {
    HotOnly,
    HotPlusWarm,
    HotPlusWarmPlusCold,
}
```

This enum is not a runtime branch inserted into the hot loop. It is a design
audit tool that determines which concrete storage view a function is permitted
to request. In practice that means separate functions or separate view types.

Examples:

- suppression scan should be `HotOnly`
- snapshot commit should be `HotOnly`
- finalize should be `HotOnly` first, with explicit warm escalation only where
  reporting fields require it
- explain resolver is `HotPlusWarmPlusCold`

The rule is:

```rust
Warm or cold access is not illegal. Implicit warm or cold access from APIs that
look hot-only is illegal.
```

## Optimization Guardrails

Milestone 4 is not a license for premature low-level cleverness. The first
closure target is access discipline and lane separation, not maximal density.

The following guardrails are mandatory:

- physical lane separation beats micro-packing
- semantic clarity and proof clarity beat custom encodings
- no custom allocators, handle-width rewrites, or aggressive bit-packing before
  counters and benchmark deltas prove the simpler split is insufficient
- no speculative vectorization-style transforms
- no representation cleverness that weakens rollback, restore, merge, or replay
  transparency
- no "while we are here" rewrites of unrelated hot structures without explicit
  milestone scope admission

The rule is:

```rust
Do not optimize representation before access boundaries are enforced and
measured.
```

## Serialization and In-Memory Assembly Boundary

`NodeEntry` may continue to exist temporarily as a compatibility assembly type,
but its role changes during the milestone:

- it stops being the mandatory hot-path carrier
- it remains a migration surface for image load/store and broad administrative
  operations until the split-store work closes
- after migration closure, it should either become a compatibility-only image
  assembly helper or be replaced entirely by explicit serialized image types

This means:

- we do not require an all-at-once deletion of `NodeEntry`
- we do require a one-way architectural shift in which hot modules stop
  depending on `NodeEntry`

## Node Handle and Arena Constraints

The milestone does not change the external meaning of `NodeId`. Storage still
remains index-addressed and generation-validated. The SoA split must preserve:

- generation validation semantics
- stale-handle error behavior
- branch/restore identity rules
- compatibility with existing arena ownership assumptions

Any narrowing of handle width or compaction of internal storage must remain
within the current address-space guarantees already assumed by the runtime.

## Instrumentation Contract

Milestone 4 must be measured with counters that explain meaningful locality and
access-boundary work. We explicitly do **not** want permanent per-field-read
counters in release hot paths.

Permanent acceptance counters should focus on:

- `hot_node_broad_entry_access_count`
- `hot_runtime_artifact_warm_escalation_count`
- `hot_runtime_artifact_cold_escalation_count`
- `stable_shape_commit_batch_count`
- `mixed_shape_commit_batch_count`
- `snapshot_commit_version_only_entry_count`
- `snapshot_commit_replacement_entry_count`
- `hot_path_cold_materialization_bypass_count`

Temporary debug-only or benchmark-only probes may be added for inventory work:

- field access sampling
- wide accessor call-site enumeration
- per-lane touch counts

Those probes must be gated behind debug or targeted benchmark instrumentation
and must not become permanent production hot-path counters.

Counters are necessary but not sufficient. Reduced broad-access or escalation
counts do not by themselves prove the runtime became faster or cheaper overall.

Milestone 4 performance certification must therefore include:

- wall-clock benchmark deltas on representative workloads
- allocation deltas on representative workloads
- memory footprint deltas on representative workloads
- structural counter deltas

Optional hardware-proxy metrics such as cache-miss or branch-miss sampling may
inform diagnosis where available, but they are diagnostic aids rather than hard
closure requirements.

The governing rule is:

```rust
No Milestone 4 performance claim is accepted from structural counters alone if
representative end-to-end benchmark deltas do not improve.
```

## Required Access Matrix

Before moving fields, the implementation must produce a concrete access matrix
covering at least these lanes:

- serial apply
- serial semantic finalize
- suppression walk
- snapshot commit
- invalidation narrowing
- branch restore
- rollback application
- merge reconciliation
- explain/reporting materialization

For each lane, the matrix must state:

- fields read
- fields written
- whether the field is authoritative or derived
- whether the field is hot, warm, or cold
- whether the access is mandatory, conditional, or accidental legacy access

This matrix is not optional. It is the proof that the storage split is grounded
in actual workload behavior rather than taste.

The access matrix must also record, for each field touched by a hot lane:

- whether the field's interior representation passed the interior heat audit
- whether the field is accessed directly or through a compatibility path
- whether any warm or cold escalation is expected, exceptional, or accidental

## Migration Plan

### Phase 0: Persistence Bridge

Before any storage split lands:

- introduce canonical serialized node image types
- support reading current persisted forms into the canonical image bridge
- update checkpoint / branch / restore code to serialize through image types,
  not direct in-memory layout
- add compatibility tests proving old saved state can still load

No SoA layout change lands before this phase is closed.

Closure gate:

- canonical image bridge exists
- old persisted shapes load successfully through the bridge
- checkpoint / branch / restore surfaces no longer depend directly on the
  future in-memory layout

### Phase 1: Access Inventory and Enforcement Seams

- build the lane-by-lane access matrix
- add temporary debug probes where needed to confirm hot-path access reality
- introduce concrete `NodeStorage` access methods and view types alongside the
  current broad entry path
- begin narrowing module visibility so hot modules can migrate off broad entry
  access

Deliverable: hot modules can start consuming concrete views without requiring
the full physical split yet.

Closure gate:

- access matrix is complete for all named lanes
- interior heat audit is complete for all candidate hot fields
- no new hot-path code may be introduced on broad entry accessors after this
  gate closes
- all explicit dependencies that require broad compatibility access are
  enumerated in this document

### Phase 2: Artifact Hot/Warm Split

- split `RuntimeArtifactState` into `RuntimeArtifactHot` and
  `RuntimeArtifactWarm`
- replace broad artifact snapshots in lowered/apply/finalize structures with
  compact hot before-images
- preserve `TraceSummary` / historical artifact assembly through explicit
  reconstruction bridges
- split operational truth flags from warm-presence flags

Deliverable: hot apply/finalize no longer carry broad runtime artifact state by
default.

Closure gate:

- broad runtime artifact snapshots are removed from the main hot apply/finalize
  structs
- operational truth flags and warm-presence flags are physically separated
- trace / historical reconstruction remains parity-safe

### Phase 3: Hot Node Store Extraction

- move clearly hot node fields into `NodeHotStore`
- keep `NodeWarmStore` and `NodeColdStore` physically separate
- update hot execution lanes to use concrete hot views
- maintain compatibility assembly for non-hot callers until migration closure

Deliverable: staged hot lanes stop borrowing broad `NodeEntry`.

Closure gate:

- main serial hot lanes consume concrete hot views
- no hot module may depend on both compatibility entry assembly and split-store
  views at the end of this gate
- candidate hot fields have passed interior heat audit or remained warm by
  explicit decision

### Phase 4: Proof-Carrying Batch and Commit Narrowing

- seal stable-shape and replacement-shape commit forms end to end
- ensure hot apply emits `ProvenSnapshotCommit` variants rather than generic
  snapshot update carriers
- ensure finalize consumes only finalize-ready batch forms

Deliverable: semantically distinct commit and finalize paths stay separated by
type all the way through execution.

Closure gate:

- stable-shape and replacement commit forms remain type-separated through commit
- authoritative commit and commit-adjacent warm escalation are structurally
  separated where applicable
- finalize consumes only finalize-ready forms

### Phase 5: Rollback / Branch-Restore Store Synchronization

- implement lane-aware rollback delta/image types
- ensure restore applies coherent multi-lane updates atomically at the logical
  node level
- add rollback and branch-restore certification tests over the split-store
  model

Deliverable: split storage is proven restore-safe and rollback-safe.

Closure gate:

- rollback is lane-coherent
- branch restore is lane-coherent
- merge lane rules are documented and certified for the split-store model

### Phase 6: Visibility Closure and Debt Removal

- restrict or remove broad hot-path accessors
- migrate remaining hot modules off compatibility entry assembly
- remove transitional duplication where metrics and certification prove closure
- explicitly mark any remaining broad-path compatibility debt

Deliverable: hot-path discipline is enforced by API and visibility rather than
convention.

Closure gate:

- all remaining compatibility paths are explicitly enumerated
- no non-enumerated hot-module broad-access dependency remains
- milestone closure is blocked until transitional dual-path hot access is gone

Gate 6 is now closed with the following visibility and compatibility work
landed:

- [entries.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/storage/entries.rs)
  now keeps `get_entry` and `get_entry_mut` crate-visible instead of public, so
  broad `NodeEntry` assembly is no longer part of the external API surface
- [entries.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/storage/entries.rs)
  now exposes explicit boundary accessors for condition reads, runtime artifact
  presence, lineage id, reuse-boundary authority, retained/cold artifact lanes,
  execution-trace stamps, and lineage/execution stamping so boundary modules do
  not need to reach through broad entry state by default
- [dot.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/presentation/outputs/dot.rs),
  [bridge.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/presentation/harness/bridge.rs),
  [execution_flow.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/diagnostics/runtime/execution_flow.rs),
  [recorder.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/diagnostics/runtime/recorder.rs),
  [history.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/diagnostics/inspection/history.rs),
  and [summary.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/diagnostics/model/summary.rs)
  now consume those named graph accessors instead of relying on public broad
  entry assembly
- [effect.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/runtime/effect.rs),
  [context.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/context.rs),
  [context_resolution.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/evaluation/reuse/context_resolution.rs),
  and [routing.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/invalidation/routing.rs)
  now use named graph transitions and narrowed read accessors instead of
  directly assembling broad `NodeEntry` state on execution and invalidation
  paths
- [observer.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/runtime/observer.rs)
  now keeps broad runtime artifact compatibility state crate-visible, and
  [facade.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/facade.rs)
  no longer re-exports `NodeEntry` or `RuntimeArtifactState` on the public API
- [phase1_api.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/tests/phase1_api.rs)
  now carries a Gate 6 regression barrier asserting that broad entry access is
  crate-visible only and that representative boundary modules use explicit
  accessors

Enumerated remaining broad-path compatibility seams:

- storage-internal graph maintenance, topology mutation, compaction, and
  snapshot-commit internals inside `crate::data::graph::*`
- diagnostics and explain materialization boundaries that intentionally assemble
  rich authority from multiple retained and reconstructed lanes
- convenience-only easy-mode runtime code in
  [easy/runtime.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/easy/runtime.rs)
- crate-internal tests that deliberately mutate or inspect full entries to
  validate invariants and migration boundaries

Gate 6 certification evidence now includes:

- `cargo test -p worth-signal phase1_api -- --nocapture`
- `cargo check -p worth-signal`
- `cargo check -p worth-signal --features parallel`
- `cargo test -p worth-signal --lib`
- `cargo test -p worth-signal --lib --features parallel`
- `cargo test -p worth-signal performance_profiles -- --ignored --nocapture --test-threads=1`

Milestone 4 certification is now green across behavioral parity,
architectural certification, and the required performance evidence surface.

## Testing and Certification

Milestone 4 is not complete until all three certification layers are green.

Behavioral parity:

- serial library sweep
- parallel library sweep
- retained vs reconstructed artifact parity
- snapshot restore parity
- branch restore parity
- merge reconciliation parity
- merge parent richness divergence parity

Architectural certification:

- compile-time proof that stable-shape-only consumers cannot receive generic
  replacement updates
- compile-time proof that finalize-only consumers cannot receive pre-apply task
  forms
- visibility tests or module-privacy structure proving hot modules cannot
  casually call broad entry accessors

Performance certification:

- ignored perf suite with `--test-threads=1`
- representative staged serial lanes
- representative stable-shape and replacement-shape lanes
- before/after counter review for warm/cold escalation and broad-access counts
- before/after wall-clock benchmark deltas
- before/after allocation deltas
- before/after memory footprint deltas
- committed certification baseline artifact at
  `crates/worth-signal/src/tests/performance_baseline.json`
- baseline-gated perf harness that compares current ignored-profile summaries
  against the committed artifact instead of reporting samples only

## Acceptance Criteria

Milestone 4 is accepted only when all of the following are true:

- staged hot execution lanes operate primarily through concrete hot-store views
- broad `NodeEntry` access is no longer available to the main hot modules
- `RuntimeArtifactHot` and `RuntimeArtifactWarm` are physically and semantically
  separated
- stable-shape and replacement snapshot commit forms remain type-separated into
  commit execution
- persistence compatibility is preserved through canonical serialized image
  bridges
- rollback and branch restore restore coherent split-store state
- permanent counters show materially reduced warm/cold escalation and reduced
  broad hot-path access on representative workloads
- no semantic parity regressions appear in replay, restore, branch, merge, or
  diagnostics truth

## Definition of Done

Milestone 4 is done when the runtime can be described honestly as follows:

The hot staged execution lanes operate on compact, index-addressed hot node and
artifact stores; semantically distinct commit and finalize phases consume
proof-bearing types that encode what has already been proven; warm and cold
state are reachable only through explicit non-hot escalation boundaries;
persistence is mediated by canonical serialized image types instead of in-memory
layout; and rollback / branch restore preserve exact logical node truth across
the split-store model.

If any part of that statement is still only convention, the milestone is not
closed.
