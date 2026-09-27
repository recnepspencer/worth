# Milestone 8 Engineering Spec

## Status

Milestone 8 is complete as of 2026-03-30.

This document now serves two roles:

- the architectural spec that governed implementation
- the closeout record for what actually shipped

Closeout verification completed against:

- full `worth-relational` test matrix
- compile-fail phase-boundary suite
- UI boundary suite
- Milestone 8 hostile query, parity, publication, fintech, and complexity lanes

Notable shipped closeout points:

- proof-bearing query planning is the only supported growth path
- immutable parallel read execution is reducer-owned and deterministic
- bounded deterministic `SampledParity` is implemented
- accelerated query support includes equality and `AnyOf` payload-field lanes
- bulk mutation planning and admission carry naming, lineage, and provenance
- legacy public packet/result fallback surfaces were hard-removed rather than
  left as compatibility debt

## 1. Milestone Intent

Milestone 8 completes the scale side of `worth-relational`.

This is not a "make queries faster" milestone and it is not an "add some
parallelism" milestone. It is the point where read/query and bulk mutation must
be elevated to the same architectural standard already expected of commit,
merge, validation, replay, and durability.

The runtime must emerge from this milestone able to make honest, certifiable
claims about:

- industrial bulk query
- immutable parallel read execution
- partition-aware and locality-aware planning
- bulk mutation as a first-class scale surface
- memory stability under long-lived workloads
- index acceleration without accidental authority
- deterministic observability under hostile scheduling
- preservation of lineage, provenance, and persistent naming under scale
  pressure

The governing rule remains:

`parallelize disposable work, serialize authority`

Nothing in Milestone 8 weakens that rule.

## 2. Current Reality In Code

The current runtime already has strong authority-side modeling. The weakest
surface relative to the target architecture is query/read execution and
scale-grade bulk mutation.

### 2.1 Strong Existing Substrate

The code already gives Milestone 8 a substantial base:

- namespaced public facade in
  [facade.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/facade.rs)
- explicit immutable read surfaces through `RelationalReadView` and
  `VisibilityProjectionView`
- explicit lineage, inspection, and structural identity surfaces in
  [inspection/data/mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/inspection/data/mod.rs)
- explicit commit, validation, and complexity artifacts in
  [outcomes.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/transactions/data/outcomes.rs)
- explicit complexity registry and broad runtime counters in
  [performance/data/mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/performance/data/mod.rs)
- partition-bounded visibility complexity tests in
  [visibility_budgets.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/tests/complexity/contracts/visibility_budgets.rs)
- existing bulk mutation intent substrate in
  [mutation_intent.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/transactions/data/intents/mutation_intent.rs)
- existing client-key normalization substrate in
  [client_keys.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/transactions/data/intents/client_keys.rs)

### 2.2 Historical Weak Seams Milestone 8 Addressed

These were the exact low-resolution seams at milestone start, and they are now
closed:

1. `QueryWorkPacket` was too weak as a scale contract and has been removed.
   `PlannedQueryPacket` is the surviving growth path.

2. `ReadPacketPlan` was only a chunk-touch summary.
   It remains a narrow storage-planning artifact, not a proof-bearing query
   contract.

3. `RelationalReadView::execute_packet` was not a scale-grade primary engine.
   The reducer-based planned-query execution path replaced it as the supported
   read surface.

4. `IndexAccess::read_with_storage_fallback` was not an honest dual-path
   surface.
   It has been replaced by typed admissibility, parity modes, and
   `FallbackParityVerifiedQueryOutcome`.

5. Bulk mutation existed mainly as intent shape rather than proof-bearing scale
   work.
   Planning and proof-widened admission are now first-class contracts.

6. Persistent naming existed as substrate but not planning contract.
   It now participates directly in bulk mutation planning and admission.

## 3. Architectural Laws For This Milestone

### 3.1 Rule 41 Is Central

A type must encode what has been proven about a value.

For Milestone 8, that means:

- a raw query request is not the same as a planned packet
- a planned packet is not the same as a snapshot-admitted packet
- a snapshot-admitted packet is not the same as a reducer-ready execution plan
- a bulk mutation intent collection is not the same as a naming-stable batch
- a naming-stable batch is not the same as a lineage-admitted batch
- a lineage-admitted batch is not the same as a provenance-complete batch

If a later phase relies on a fact that is not carried in the type it consumes,
the architecture is wrong.

### 3.2 Distinct Proof Types Must Block Distinct Illegal States

Rule 41 does not justify ornamental wrapper chains.

Each proof-bearing wrapper introduced in Milestone 8 must correspond to:

- distinct admission work
- distinct observable diagnostics or counters
- distinct illegal states that become unrepresentable after widening

If two wrappers do not block different classes of misuse, they should be
collapsed.

Initial intended proof boundaries:

- `PlannedQueryPacket`
  Blocks execution without explicit scope, locality, ordering, and fallback
  contract.
- `SnapshotPinnedQueryPlan`
  Blocks parallel execution over a non-immutable or wrong-context surface.
- `NamingStableBulkMutationBatch`
  Blocks scale execution before naming normalization and persistent naming basis
  capture.
- `LineageSafeBulkMutationBatch`
  Blocks scale execution before lineage transition admission is known.
- `ProvenanceCompleteBulkMutationBatch`
  Blocks scale execution before causally meaningful batch identity and worker
  provenance basis are attached.

If implementation reveals that `NamingStable`, `LineageSafe`, and
`ProvenanceComplete` do not each prevent a real illegal state, the chain must
be compressed rather than preserved ceremonially.

### 3.3 Determinism Is Non-Negotiable

For every new read or mutation scale surface:

- packet scheduling may vary
- worker completion order may vary
- partition dispatch order may vary internally

But:

- result ordering may not vary
- fallback semantics may not vary
- patch and publication order may not vary
- lineage and provenance artifacts may not vary
- diagnostics summaries on observable surfaces may not vary

### 3.4 Persistent Naming, Lineage, and Provenance Are Product Semantics

These are not optional metadata layers.

Bulk mutation must not become fast by silently degrading:

- persistent naming continuity
- lineage continuity
- causal provenance visibility

## 4. Adversarial Constraints

Milestone 8 must be designed against the following hostile conditions:

1. Hostile chip workload:
   snapshot-pinned readers scan and traverse hot connectivity truth while
   another branch performs rapid rewrites and derived index generations lag.

2. Hostile CAD workload:
   large topological rewrites produce many relation and identity changes, and
   the system must preserve naming continuity, lineage interpretability, and
   deterministic read surfaces.

3. Hostile scheduling:
   same workload, different worker counts and different packet completion
   orders, zero observable drift.

4. Hostile index conditions:
   missing, stale, partial, corrupted, or incompatible index generations must
   never change truth semantics.

5. Hostile bulk mutation:
   large cross-partition mutation batches must preserve canonical publication
   order, lineage, provenance, and persistent naming without devolving into
   hidden whole-state work.

## 5. Design Goals

Milestone 8 succeeds only if the runtime can honestly claim:

- bulk query is a primary API, not stitched scalar loops
- parallel read happens only over immutable snapshot truth
- storage fallback is always semantically authoritative
- index use is explicit, typed, and rejectable
- bulk mutation preserves identity semantics under scale
- cost claims are measurable and asserted
- results remain deterministic under hostile scheduling

## 6. Out Of Scope

Milestone 8 does not:

- weaken serialized commit authority
- add parallel mutation authority
- fuse planner and executor into heuristic runtime branching
- make derived indexes authoritative
- move merge meaning into generic query paths
- move persistent naming repair into query-time hidden reconstruction
- make lineage advisory during bulk mutation because the surface is large

## 7. New Type System

These types are the intended center of Milestone 8. They are designed to evolve
the current API rather than replace it in one break.

### 7.1 Query Planning Types

These should live under a structured query planning surface such as:

- `query/data/`
- `query/planning/`
- `query/execution/`
- `query/reduction/`
- `query/fallback/`

Core identifiers:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DeterministicQueryPlanKey(pub u128);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DeterministicQueryFragmentKey(pub u128);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryPlanContextId {
    pub runtime_instance_id: u64,
    pub snapshot_id: SnapshotId,
    pub version_id: VersionId,
    pub schema_version: SchemaVersionId,
    pub descriptor_semantics_version: DescriptorSemanticsVersion,
}
```

Reason:

- packet proofs must be invalid outside the exact planning context
- this mirrors the already-established proof-bearing planning discipline in the
  codebase

### 7.2 Query Scope Types

Replace the current target-only worldview.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryScope {
    ExplicitTargets {
        targets: Arc<[RecordRef]>,
    },
    EntityKindScan {
        kind_id: KindId,
        partition_scope: Option<Arc<[PartitionId]>>,
    },
    RelationKindScan {
        kind_id: KindId,
        partition_scope: Option<Arc<[PartitionId]>>,
    },
    EntityPayloadFieldEquals {
        field: String,
        value: String,
        partition_scope: Option<Arc<[PartitionId]>>,
    },
    EntityPayloadFieldAnyOf {
        field: String,
        values: Arc<[String]>,
        partition_scope: Option<Arc<[PartitionId]>>,
    },
    RelationPayloadFieldEquals {
        field: String,
        value: String,
        partition_scope: Option<Arc<[PartitionId]>>,
    },
    RelationPayloadFieldAnyOf {
        field: String,
        values: Arc<[String]>,
        partition_scope: Option<Arc<[PartitionId]>>,
    },
    AspectFilteredEntities {
        kind_id: Option<KindId>,
        aspect_filter: AspectFilter,
        partition_scope: Option<Arc<[PartitionId]>>,
    },
    AspectFilteredRelations {
        kind_id: Option<KindId>,
        aspect_filter: AspectFilter,
        partition_scope: Option<Arc<[PartitionId]>>,
    },
    OutgoingNeighborhood {
        seeds: Arc<[EntityId]>,
        relation_kind_scope: Option<Arc<[KindId]>>,
    },
    IncomingNeighborhood {
        seeds: Arc<[EntityId]>,
        relation_kind_scope: Option<Arc<[KindId]>>,
    },
    ConnectivityTraversal {
        seeds: Arc<[EntityId]>,
        relation_kind_scope: Option<Arc<[KindId]>>,
        max_depth: Option<u32>,
    },
}
```

### 7.3 Query Locality Types

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryLocalityClass {
    SinglePartition {
        partition_id: PartitionId,
    },
    PartitionBounded {
        partitions: Arc<[PartitionId]>,
    },
    CrossPartitionTraversal,
}
```

Reason:

- `PartitionHint` is too advisory and too weak
- locality class affects legality, profitability, and complexity honesty

### 7.4 Query Ordering Types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryOrderingContract {
    CanonicalEntityIdOrder,
    CanonicalRelationIdOrder,
    CanonicalRecordRefOrder,
    CanonicalTraversalOrder,
}
```

This is necessary but not sufficient for traversal semantics. See Section 8.

### 7.5 Query Fallback Types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryFallbackContract {
    StorageOnly,
    IndexAdmissibleStorageEquivalent,
}
```

### 7.6 Planned Query Packet

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedQueryPacket {
    pub label: String,
    pub context_id: QueryPlanContextId,
    pub scope: QueryScope,
    pub locality: QueryLocalityClass,
    pub ordering: QueryOrderingContract,
    pub fallback: QueryFallbackContract,
    pub execution_shape: QueryExecutionShape,
    pub reduction: ReductionDiscipline,
    pub plan_key: DeterministicQueryPlanKey,
    pub target_count_hint: usize,
}
```

This is the first real replacement for `QueryWorkPacket`.

### 7.7 Admission Types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryParallelLegality {
    LegalReadOnlySnapshot,
    RequiresSerialReduction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuerySerialReason {
    TinyPacket,
    SingleChunkSurface,
    BroadCrossPartitionCoordination,
    UnsupportedIndexPath,
    ReductionWouldDominateExecution,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryParallelProfitability {
    Profitable,
    SerialPreferred {
        reason: QuerySerialReason,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotPinnedQueryPlan {
    pub packet: PlannedQueryPacket,
    pub snapshot: SnapshotHandle,
    pub legality: QueryParallelLegality,
    pub profitability: QueryParallelProfitability,
}
```

This is the Rule 41 boundary for immutable parallel read.

### 7.8 Execution Fragment and Result Types

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryFragmentCounters {
    pub target_count: usize,
    pub entity_records_emitted: usize,
    pub relation_records_emitted: usize,
    pub touched_partitions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryWorkerFragment {
    pub plan_key: DeterministicQueryPlanKey,
    pub fragment_key: DeterministicQueryFragmentKey,
    pub ordering: QueryOrderingContract,
    pub entities: Vec<EntityReadRecord>,
    pub relations: Vec<RelationReadRecord>,
    pub counters: QueryFragmentCounters,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalQueryResult {
    pub execution_shape: QueryExecutionShape,
    pub ordering: QueryOrderingContract,
    pub entities: Vec<EntityReadRecord>,
    pub relations: Vec<RelationReadRecord>,
    pub reduction_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryComplexitySummary {
    pub packet_count: usize,
    pub fragment_count: usize,
    pub touched_partitions: usize,
    pub target_count: usize,
    pub entity_records_emitted: usize,
    pub relation_records_emitted: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryExecutionOutcome {
    pub plan: SnapshotPinnedQueryPlan,
    pub result: CanonicalQueryResult,
    pub complexity: QueryComplexitySummary,
}
```

This is the replacement for loose `PacketResult` on the scale path.

### 7.9 Index Path Types

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexQueryRejectionClass {
    MissingGeneration,
    IncompatibleVersion,
    IncompatibleBranch,
    CorruptPayload,
    UnsupportedScope,
    UnsupportedOrderingContract,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryAccessPath {
    AuthoritativeStorage,
    DerivedIndexGeneration {
        generation_id: DerivedIndexGenerationId,
    },
    DerivedIndexRejectedStorageFallback {
        rejection: IndexQueryRejectionClass,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FallbackParityVerifiedQueryOutcome {
    pub execution: QueryExecutionOutcome,
    pub access_path: QueryAccessPath,
    pub parity_basis_digest: String,
}
```

### 7.10 Bulk Mutation Planning Types

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BulkMutationScope {
    BulkEntityCreate,
    BulkRelationCreate,
    BulkMixedMutation,
    TopologyRegionRewrite,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BulkMutationLocalityFootprint {
    pub touched_partitions: Arc<[PartitionId]>,
    pub cross_partition_relation_count: usize,
    pub entity_target_count: usize,
    pub relation_target_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BulkMutationNamingPlan {
    pub normalized_client_keys: Arc<[InternedString]>,
    pub naming_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlannedLineageTransition {
    CreateEntity,
    ReplaceEntity {
        source: EntityId,
    },
    DeleteEntity {
        target: EntityId,
    },
    CreateRelation,
    DeleteRelation {
        target: RelationId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BulkMutationLineagePlan {
    pub transitions: Arc<[PlannedLineageTransition]>,
    pub lineage_scope_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BulkMutationProvenancePlan {
    pub batch_name: String,
    pub worker_batch_names: Arc<[String]>,
    pub provenance_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedBulkMutationBatch {
    pub transaction_id: TransactionId,
    pub scope: BulkMutationScope,
    pub locality: BulkMutationLocalityFootprint,
    pub naming: BulkMutationNamingPlan,
    pub lineage: BulkMutationLineagePlan,
    pub provenance: BulkMutationProvenancePlan,
    pub intents: Arc<[MutationIntent]>,
}
```

### 7.11 Bulk Mutation Proof-Widening Types

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamingStableBulkMutationBatch {
    pub planned: PlannedBulkMutationBatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageSafeBulkMutationBatch {
    pub naming_stable: NamingStableBulkMutationBatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceCompleteBulkMutationBatch {
    pub lineage_safe: LineageSafeBulkMutationBatch,
}
```

These wrappers are provisional in the sense described in Section 3.2. If any
two are found to block the same illegal state, they must be collapsed.

## 8. Traversal Canonicalization Rules

Traversal is the exact point where deterministic semantics and performance
honesty are most likely to drift.

`CanonicalTraversalOrder` is not enough unless the expansion rules are fixed.

For all neighborhood and connectivity traversal surfaces, canonical traversal
must mean:

1. Seed ordering:
   seeds are canonicalized before traversal begins
   - `EntityId` ascending for entity-seeded traversals
   - `RelationId` ascending if a future traversal begins from relation seeds

2. Partition tie-break:
   if expansion candidates span partitions, partition ordering is ascending
   `PartitionId`

3. Relation-kind tie-break:
   if multiple relation kinds are admissible, relation kind ordering is
   ascending `KindId`

4. Edge tie-break within one `(partition, kind)` bucket:
   relations are visited in ascending `RelationId`

5. Neighbor entity tie-break:
   when frontier expansion encounters multiple destination entities at the same
   traversal depth, entities are emitted in ascending `EntityId`

6. Breadth semantics:
   `CanonicalTraversalOrder` means breadth-first over canonicalized frontier
   buckets unless a specific traversal API declares otherwise in its contract

7. Reduction semantics:
   reducers may merge packet-local canonical fragments
   reducers may not invent a different traversal order than the packet contract

8. Performance honesty:
   if a traversal path requires a global re-sort beyond canonical merge of
   already-canonical packet-local fragments, that cost must be explicit in the
   plan and counters

This section exists because traversal is where "deterministic" often becomes
quietly expensive or semantically ambiguous.

## 9. Query Execution Memory Model

The prior plan correctly identified memory risk but was not explicit enough
about mechanics. This section fixes that.

### 9.1 Primary Scale Path Policy

The primary scale path should prefer:

- packet-local materialization from immutable snapshot surfaces
- reusable worker-local buffers for fragment assembly
- canonicalization at packet-local fragment construction time where feasible
- reducer merge over already-canonical fragments

The primary scale path should avoid:

- rebuilding large BTreeMap indexes per query packet
- per-record heap allocation where a packet-local buffer can be reused
- full read-view cloning as a prerequisite for scale execution
- late global canonicalization after large unsorted accumulation

### 9.2 Borrowing vs Ownership

For Milestone 8:

- worker execution may borrow immutable runtime state during scanning and lookup
- worker result fragments should own emitted `EntityReadRecord` and
  `RelationReadRecord` values at the fragment boundary
- reducers should merge owned fragments

This is deliberate:

- pure borrowing across packet execution would complicate cross-thread fragment
  lifetime management
- full pre-materialized snapshot cloning would be mechanically dishonest for
  many workloads

The expected compromise is:

- borrow while traversing immutable snapshot substrate
- materialize only selected output records into worker-local fragment buffers
- reduce owned fragment outputs deterministically

### 9.3 Fragment Buffer Reuse

Worker execution should be designed around reusable packet-local buffers.

At minimum:

- entity result buffers
- relation result buffers
- frontier buffers for traversal-heavy packets

must be reusable across packet execution within a query execution lane.

### 9.4 Projection Timing

Canonicalization should occur before projection-level ergonomic conversion where
possible.

Meaning:

- the scale path should reduce canonical record outputs first
- typed projection conversion may remain a higher ergonomic layer

This avoids creating a projection-specific canonicalization burden that can
silently distort costs or semantics.

## 10. Index/Fallback Parity Modes

The prior plan needed a sharper distinction here.

Parity verification must not silently turn into "run both paths all the time."

Milestone 8 distinguishes three modes:

1. Certification parity mode
   - both admissible paths may run
   - machine-checkable parity artifacts are emitted
   - used in certification and targeted hardening tests

2. Sampled parity mode
   - production-capable optional mode
   - both paths may run for a bounded sample of requests
   - emits diagnostic drift evidence without redefining the steady-state hot path

3. Production admissibility mode
   - one path executes
   - explicit typed access-path outcome is emitted
   - fallback semantics remain authoritative

`FallbackParityVerifiedQueryOutcome` therefore names the architectural contract,
not the guarantee that both paths were executed on every request.

The specific parity mode in effect should be explicit in diagnostics and, where
appropriate, in harness-visible metadata.

Current implementation note:

- `SampledParity` is implemented as a bounded deterministic production mode
- sampled verification selection is derived from stable workload identity
  rather than scheduler state or thread affinity
- `CertificationParity` remains the always-dual-run lane

## 11. Bulk Mutation Semantic Non-Flattening Rules

Bulk mutation planning is one of the strongest parts of Milestone 8, but it is
also at high risk of semantic flattening under implementation pressure.

The following must remain explicit:

- `BulkMixedMutation` must not become a bucket that hides the classes of entity
  replace, entity delete, relation create, relation delete, and topology-local
  rewrite that occurred
- `TopologyRegionRewrite` must not collapse into counts alone
- lineage planning must preserve exact transition classes needed by later
  inspection and replay
- relation-heavy paths must not flatten lineage into generic create/delete
  counts
- provenance must remain attributable at the batch and worker-batch level

Aggregate digests are required, but they are not allowed to replace exact
transition classification surfaces.

## 12. API Evolution Plan

### 12.1 Legacy Surface Break

Milestone 8 now assumes a hard break on the legacy public query and fallback
compatibility facade.

Completed removals and demotions:

- `QueryWorkPacket` is no longer part of the public facade growth path
- `PacketResult` is no longer part of the public facade growth path
- `IndexedReadOutcome` is removed
- `RelationalReadView::execute_packet` is removed from the growth path
- `IndexAccess::read_with_storage_fallback` is removed from the growth path

Remaining rule:

- no new scale-grade feature work may target legacy packet/result compatibility
  types directly, even internally, except where a sealed ingress adapter is
  still needed for narrow planning heuristics or test scaffolding

### 12.2 New Growth Path

All new Milestone 8 work should target:

- `PlannedQueryPacket`
- `SnapshotPinnedQueryPlan`
- `QueryExecutionOutcome`
- `FallbackParityVerifiedQueryOutcome`
- `PlannedBulkMutationBatch`
- `NamingStableBulkMutationBatch`
- `LineageSafeBulkMutationBatch`
- `ProvenanceCompleteBulkMutationBatch`

## 13. Module Layout

Do not create broad dumping-ground files like `scale.rs`, `performance.rs`, or
`milestone8.rs`.

Recommended shape:

```text
query/
  data/
    mod.rs
    planning.rs
    admission.rs
    execution.rs
    reduction.rs
    fallback.rs
    counters.rs

transactions/
  bulk_planning/
    mod.rs
    locality.rs
    naming.rs
    lineage.rs
    provenance.rs
    batch.rs

transactions/
  bulk_execution/
    mod.rs
    admission.rs
    apply.rs
```

If implementation reveals a better exact module layout, keep the same
responsibility split.

## 14. Phase Plan

Each phase has a distinct completion boundary. The work must not proceed as
ad hoc interleaving.

### Phase 0: Spec Freeze

Goal:

- freeze the type model and migration rules before coding

Deliver:

- this document
- public/private export decisions
- naming conventions for new types
- legacy compatibility policy

Completion condition:

- no Milestone 8 code lands before the phase boundaries are agreed

### Phase 1: Query Packet Modeling Upgrade

Status: shipped

Goal:

- introduce proof-bearing query planning types without breaking existing
  callers

Deliver:

- `QueryPlanContextId`
- `QueryScope`
- `QueryLocalityClass`
- `QueryOrderingContract`
- `QueryFallbackContract`
- `DeterministicQueryPlanKey`
- `PlannedQueryPacket`

Required code changes:

- add new types under `query/data`
- export them through the `query` facade
- replace explicit-target ingress with `PlannedQueryPacket::explicit_targets`

Completion condition:

- all new query work can target the new packet type
- old tests still pass without full migration

### Phase 2: Snapshot Admission And Query Planning

Goal:

- make all scale-grade query execution consume a snapshot-admitted,
  proof-bearing plan

Deliver:

- `QueryParallelLegality`
- `QueryParallelProfitability`
- `QuerySerialReason`
- `SnapshotPinnedQueryPlan`
- planning entrypoint on a query or visibility facade

Required code changes:

- planner consumes `SnapshotHandle`
- plan binds to exact snapshot/version/schema semantics
- packet proof becomes invalid outside its context

Completion condition:

- no new parallel query path accepts raw `PlannedQueryPacket`
- serial fallback reasons are explicit and observable

### Phase 3: Early Counter And Contract Scaffolding

Goal:

- land the minimum counter and complexity scaffolding before reducer execution
  hardens around hidden costs

Deliver:

- initial Milestone 8 counters in `RuntimeComplexityCounters`
- draft complexity contract entries for new query planning and execution paths
- harness-visible diagnostics for legality, profitability, and fallback

Completion condition:

- the first reducer-based execution path ships with counters already attached

This phase is intentionally early because several costs are hard to unwind once
fragment and reducer shapes are entrenched.

### Phase 4: Reducer-Based Query Execution

Goal:

- replace direct packet execution for scale surfaces with packet-local fragments
  plus deterministic reduction

Deliver:

- `QueryWorkerFragment`
- `QueryFragmentCounters`
- `CanonicalQueryResult`
- `QueryComplexitySummary`
- `QueryExecutionOutcome`

Required code changes:

- new query executor path over immutable snapshot data
- reducer contracts keyed by canonical fragment keys
- keep `RelationalReadView::execute_packet` as convenience only

Completion condition:

- supported packet classes have stable outputs under hostile scheduling
- canonical order is reducer-owned, not worker-owned

### Phase 5: True Index/Fallback Architecture

Goal:

- make index acceleration a real, explicit, typed dual-path system

Deliver:

- `IndexQueryRejectionClass`
- `QueryAccessPath`
- `FallbackParityVerifiedQueryOutcome`

Required code changes:

- replace the old storage-fallback helper with an explicit parity/admissibility
  surface
- add explicit admissibility evaluation
- add typed rejection for incompatible or unsupported index paths
- only report index use when the index path actually participates

Completion condition:

- index and storage paths are both real execution choices
- corrupted or missing indexes produce typed fallback, not silent behavior
- observable results stay identical

### Phase 6: Bulk Mutation Planning

Goal:

- promote bulk mutation from raw intent grouping to planned identity-safe scale
  work

Deliver:

- `BulkMutationScope`
- `BulkMutationLocalityFootprint`
- `BulkMutationNamingPlan`
- `BulkMutationLineagePlan`
- `BulkMutationProvenancePlan`
- `PlannedBulkMutationBatch`

Required code changes:

- integrate current client-key normalization into naming planning
- compute locality footprint before execution
- compute lineage expectations before execution
- attach provenance basis before execution

Completion condition:

- bulk entity and relation create entrypoints can produce
  `PlannedBulkMutationBatch`
- naming normalization is part of planning
- lineage and provenance are attached before execution begins

### Phase 7: Bulk Mutation Admission Proof Chain

Goal:

- require naming, lineage, and provenance proofs before scale execution

Deliver:

- `NamingStableBulkMutationBatch`
- `LineageSafeBulkMutationBatch`
- `ProvenanceCompleteBulkMutationBatch`

Required code changes:

- bulk execution APIs consume widened proof types, not raw planned batches
- relation-heavy paths preserve persistent naming and lineage continuity
  explicitly

Completion condition:

- impossible to run the scale mutation path without passing naming and lineage
  admission
- commit results remain reconstructive and deterministic

### Phase 8: Locality And Memory Honesty Pass

Goal:

- make the mechanical cost of query and bulk mutation match the semantic claims

Deliver:

- remaining Milestone 8 counters
- completed complexity contracts for new hot paths
- storage and packet execution adjustments where counters reveal dishonesty

Completion condition:

- each new fast-path claim has a named counter
- each contract has proof tests or is explicitly marked `Debt`

### Phase 9: Certification Closure

Goal:

- close Milestone 8 through hostile truth-grade tests

Required named tests:

- `Bulk query and traversal stress truth test`
- `Index non-authority corruption test`
- `Deterministic observability under hostile scheduling test`
- `Snapshot-stable concurrent read vs hot rewrite test`

Completion condition:

- supported Milestone 8 lanes pass certification with canonical artifact outputs

## 15. Counter And Complexity Plan

The prior plan introduced many counters too late. This version splits them into
early and late groups.

### 15.1 Early Counters

These must land by Phase 3 at the latest:

- `query_packet_count`
- `query_packet_item_count`
- `query_parallel_legal_count`
- `query_parallel_profitable_count`
- `query_serial_fallback_count`
- `query_entity_slot_scans`
- `query_relation_slot_scans`
- `query_entity_records_emitted`
- `query_relation_records_emitted`

### 15.2 Later Counters

These may land after the first reducer path exists:

- `query_packet_peak_width_total`
- `query_cross_partition_packet_count`
- `query_storage_path_count`
- `query_index_path_count`
- `query_index_rejection_count`
- `query_reducer_conflict_count`
- `bulk_mutation_batch_count`
- `bulk_mutation_entity_target_count`
- `bulk_mutation_relation_target_count`
- `bulk_mutation_cross_partition_relation_count`
- `bulk_mutation_naming_normalization_count`
- `bulk_mutation_lineage_transition_count`
- `bulk_mutation_provenance_record_count`

### 15.3 Recommended New Complexity Contracts

- `query.snapshot_explicit_targets`
- `query.partition_kind_scan`
- `query.relation_kind_scan`
- `query.connectivity_traversal_bounded`
- `query.index_storage_fallback_parity`
- `transactions.bulk_mutation_planning`
- `transactions.bulk_mutation_naming_normalization`
- `transactions.bulk_mutation_lineage_admission`

## 16. Critique Of Current Code Surfaces

### 16.1 `QueryWorkPacket`

Historical problem:

- it was a useful compatibility shape
- it was too weak for scale truth

Shipped outcome:

- removed
- replaced by direct `PlannedQueryPacket` construction

### 16.2 `ReadPacketPlan`

Problem:

- chunk-touch summary, not a true plan

Decision:

- keep for compatibility and chunk diagnostics
- do not treat it as the primary Milestone 8 plan type

### 16.3 `RelationalReadView::execute_packet`

Historical problem:

- it was an honest convenience surface
- it was mechanically dishonest as an industrial primary surface

Shipped outcome:

- removed from the supported growth path
- reducer-based planned-query execution is the canonical read surface

### 16.4 `IndexAccess::read_with_storage_fallback`

Historical problem:

- the name suggested a real dual-path semantic surface
- the behavior was mostly storage execution plus generation annotation

Shipped outcome:

- removed from the supported growth path
- replaced by
  `execute_query_plan_with_fallback_parity(...) -> FallbackParityVerifiedQueryOutcome`

### 16.5 Bulk Mutation Intents

Problem:

- good ingress command shapes
- not proof-bearing scale plans

Decision:

- keep them as input layer
- planning derives proof-bearing batches from them

## 17. Persistent Naming Plan

Persistent naming is a first-class milestone concern.

Current substrate:

- `client_key` on `EntitySpec` and `RelationSpec`
- `client_keys` on bulk create intents
- normalization in `transactions/data/intents/client_keys.rs`

Milestone 8 requirements:

- naming continuity must be planned before bulk execution
- normalized naming basis must be captured in `BulkMutationNamingPlan`
- naming digest must be observable in scale-grade artifact surfaces
- no bulk mutation execution path may bypass normalization
- large batches must not be allowed to weaken naming continuity for convenience

This does not require query results to expose client keys by default. It
requires mutation planning and later inspection/provenance surfaces to preserve
the naming continuity contract.

## 18. Lineage Plan

Lineage must remain structurally separate from storage identity and must
participate in scale planning.

Milestone 8 requirements:

- bulk mutation planning declares expected lineage transition classes
- relation-heavy scale mutation does not flatten lineage implications into
  generic create/delete counts
- query traversal across rebuilt or replaced surfaces remains deterministic and
  lineage-safe
- scale-grade outputs remain compatible with inspection and historical
  resolution surfaces already present in the runtime

## 19. Provenance Plan

Provenance must not be bolted on after execution.

Milestone 8 requirements:

- each scale mutation batch has explicit batch name and worker batch names
- provenance digest is derived during planning
- commit outputs and later inspection can align touched scope with the scale
  batch that produced it
- provenance collection must remain proportional to the semantic delta, not to
  whole-state breadth

## 20. Testing And Migration Strategy

### 20.1 Testing By Phase

Phase 1-2 tests:

- planning context identity changes invalidate reuse
- partition-local and cross-partition plans classify differently
- explicit target packets construct directly as proof-bearing planned packets

Phase 3 tests:

- early counters increment exactly on planning and admission paths
- serial fallback reasons are emitted deterministically

Phase 4 tests:

- same result under different fragment orderings
- same result under different worker counts
- reducer conflict handling is typed

Phase 5 tests:

- stale index generation falls back explicitly
- corrupt index payload falls back explicitly
- storage and index paths produce identical canonical output under certification
  parity mode

Phase 6-7 tests:

- client-key normalization always runs before bulk execution
- lineage transition plans exist for replace/delete-heavy bulk batches
- provenance digest is stable for identical batch inputs
- impossible to execute bulk scale path without admission types

Phase 8 tests:

- exact counter assertions for packet counts, fallback counts, emitted record
  counts
- no broad-scan regression in partition-bounded workloads
- no hidden whole-state fallback in explicit bounded scans

Phase 9 tests:

- the four required named certification tests
- machine-checkable digests emitted by every run

### 20.2 Migration Policy

The migration must not be random.

Order:

1. add new types
2. add planning and execution surfaces
3. migrate harness helpers and query tests
4. migrate storage and index access internals
5. migrate public ergonomic surfaces
6. only then de-emphasize old packet surfaces

### 20.3 Anti-Rot Rules For Compatibility Surfaces

Dual-path rot is a serious risk. To prevent it:

- no new scale-grade feature work may reintroduce a weak packet compatibility
  bag in place of `PlannedQueryPacket`
- no new scale-grade feature work may target `RelationalReadView::execute_packet`
- compatibility paths must be marked in code comments and docs as compatibility
  or convenience surfaces
- harness and certification helpers must migrate before general convenience
  helpers do
- any new test added for a Milestone 8 feature must use the new plan/execution
  path unless it is explicitly a compatibility test

## 21. Risks, Exit Criteria, And Summary

### 21.1 Highest-Risk Failure Modes

- turning `QueryWorkPacket` into a giant everything-bag instead of replacing it
  with proof-bearing types
- keeping `RelationalReadView::execute_packet` as the hidden real engine
- reporting index usage when the path was actually storage-only
- letting "bulk mutation" mean "loop of scalar apply operations with a nicer
  name"
- letting locality planning remain advisory instead of binding
- losing naming continuity under large batches because it would cost too much
- flattening lineage implications into counts
- global sorting of large fragments when reducer-owned canonical merge would
  suffice
- re-deriving planning facts inside execution workers
- adding counters but not proof tests
- treating inspection and diagnostics as the same thing
- making broad-scope provenance collection part of the hot path by default

### 21.2 Milestone 8 Exit Criteria

Milestone 8 is complete only when all are true:

- proof-bearing query planning types exist and are the growth path
- immutable parallel read execution uses reducer-owned canonical ordering
- true typed index/fallback semantics exist
- bulk mutation has naming, lineage, and provenance planning
- bulk mutation execution requires admitted proof-bearing batch types
- complexity contracts and counters exist for the new hot paths
- required certification tests pass with canonical artifact outputs

Current status:

- all exit criteria are satisfied
- full crate verification is green
- no legacy public query packet/result fallback growth surface remains
- deterministic diagnostics and workload-derived counters are in place for the
  shipped parallel read lanes

### 21.3 Summary

The core correction to earlier drafts was this:

Milestone 8 is not mainly about adding faster query code.

It is about bringing read/query and bulk mutation up to the same architectural
resolution that commit, merge, validation, and replay already have.

That means:

- proof-bearing types
- phase boundaries
- deterministic reduction
- explicit fallback contracts
- scale-honest counters
- naming, lineage, and provenance carried through planning
- traversal order defined precisely, not vaguely
- memory/allocation strategy explicit at the primary execution path
- migration that starts from the real current seams, not a greenfield fantasy

Milestone 8 produced a runtime that is broader, faster, and still truth-grade
under hostile CAD, chip, and branch-divergent workloads.
