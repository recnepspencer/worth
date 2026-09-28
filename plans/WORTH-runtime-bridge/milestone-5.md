# Milestone 5 Engineering Spec: Bridge Planning, Bulk Routing, And Parallel-Ready Scale Path

> **Status:** Planned engineering spec
>
> **Roadmap parent:** [worth_runtime_bridge_roadmap.md](./WORTH_runtime_bridge_roadmap.md)
>
> **Vision parent:** [worth_runtime_bridge_vision.md](./WORTH_runtime_bridge_vision.md)
>
> **Prior milestone:** [milestone-4.md](./milestone-4.md)
>
> **Prior closeout:** [milestone-4-closeout.md](./milestone-4-closeout.md)
>
> **Primary architectural driver:** make bridge scale behavior explicit, planned, and replay-safe before large change sets, bulk truth-view reads, and admitted parallel preparation turn the bridge into an opaque per-item routing tax
>
> **Companion docs:**
> - [worth_relational_roadmap.md](../WORTH-relational/WORTH_relational_roadmap.md)
> - [worth_signal_vision.md](../WORTH_signal/WORTH_signal_vision.md)
> - [worth_signals2.md](../WORTH_signal/WORTH_signals2.md)
> - [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)
> - [architectural_guidelines.md](../../docs/coding-guidelines/arch_laws.md)
> - [domain_standards.md](../../docs/coding-guidelines/domain_structure_laws.md)
> - [performance_guidelines.md](../../docs/coding-guidelines/perf_laws.md)

## Summary

Milestones 1 through 4 established that:

- committed truth enters the bridge through one canonical envelope
- fine-grained truth deltas lower into canonical subscription slices
- continuity across identity evolution is explicit and lineage-backed
- historical and branch-aware evaluation select one explicit truth view and replay it safely

That is enough to make the bridge correct.

It is not enough to make the bridge scale honestly under the workloads the
bridge explicitly claims it must survive:

- large topology edits that produce wide patchsets
- branch-local and historical requests that fan into broad read packets
- subscription populations where matching and reduction breadth matter as much
  as semantic correctness
- host integrations that need bulk propagation instead of per-item bridge calls
- future admission of parallel preparation without allowing the executor to
  rediscover planning semantics on the hot path

Milestone 5 exists because the bridge cannot keep treating scale as an
implementation detail after correctness is established.

The bridge must be able to say:

`this exact canonical patchset, truth-view basis, mapping registry, and continuity context lowered once into this exact bulk bridge plan, this exact packet set, this exact reduced invalidation artifact, and this exact legality basis for admitted parallel preparation`

not:

`the bridge iterated until the right outputs appeared and happened to be fast enough this time`

The bridge still does not own truth semantics, truth mutation authority,
historical retention authority, signal scheduling, or signal execution policy.
It owns:

- bulk planning vocabulary
- planning-context identity
- canonical work-packet construction
- deterministic reduction and artifact ordering
- explicit legality and profitability classification for what bridge work may
  remain serial and what may be prepared in parallel
- scale counters, decision records, and replay-safe planning artifacts

## Goal

Make bridge planning, bulk routing, packetized reads, and admitted
parallel-ready preparation first-class so large bridge workloads stay
deterministic, bounded, replay-safe, and honest about cost.

## Why This Milestone Exists

Milestone 5 belongs immediately after Milestone 4 because Milestone 4 supplied
the strongest truth-view substrate the scale path must optimize without
weakening:

- canonical route identity
- canonical slice identity
- canonical continuity identity
- canonical truth-view authority
- replay-safe historical and branch-local evaluation artifacts

Without Milestone 4, bulk planning would be tempted to optimize only current
head-state routing and treat historical or branch-local flows as special cases.
That would be architectural dishonesty.

Without Milestone 5, every later bridge ambition inherits an unproven scale
surface:

- reactive source contracts would expose read surfaces before packet breadth and
  bulk planning are explicit
- structural-identity-aware remapping would add more matching work to a bridge
  whose scale path is still opaque
- merge-aware bridge semantics would increase candidate breadth before the
  bridge has a canonical reduction model
- speculative branch coordination would need admitted parallel preparation
  without a legality basis
- end-to-end certification would still be missing a trustworthy answer to "why
  did this wide change set cost this much and reduce this way?"

Milestone 5 therefore earns its place in the roadmap by solving the next real
structural problem after explicit truth-view authority: scale-path honesty.

## Adversarial Constraint

Milestone 5 must survive the following hostile condition:

> A long-lived system with large committed patchsets, wide fine-grained surface
> sets, branch-local and historical truth-view requests, overlapping
> subscriptions, continuity remaps, diagnostics tiers that vary by environment,
> and replay after restart must lower the same canonical bridge workload into
> the same canonical bulk plan, packet set, reduction result, and parallel
> admission basis every time, while preserving fine-grained routing precision,
> while bounding bridge work by semantic delta rather than host iteration
> patterns, and while preventing the execution path from rediscovering planning
> semantics or legality decisions.

Concretely, the design must remain correct when all of the following are true:

- one committed patchset contains many entities, aspects, slice categories, and
  continuity-relevant identities
- one bridge request fans into multiple read packets across current, historical,
  or branch-local truth views
- the same workload is registered under different host insertion orders across runs
- overlapping mapping rules and continuity results require deterministic reduction
- diagnostics richness changes between environments
- replay occurs from canonical bridge planning artifacts after restart
- some packet groups are structurally independent and may be prepared in
  parallel, while others must remain serial
- some host integrations can execute bulk packet reads directly while others
  must use bridge-planned packet batching over narrower adapter seams

If any supported path:

- widens work from semantic delta to host iteration breadth
- performs per-item routing when a canonical bulk plan already exists
- re-decides packet grouping, reduction, or legality during delivery
- loses fine-grained precision under load
- uses nondeterministic reduction order for overlapping slices or continuity
  outcomes
- claims parallel safety without a canonical legality basis
- cannot explain routed item count, reduction width, packet breadth, or fallback
  behavior from first-class counters

then Milestone 5 has failed.

## Product Decision Lock

The following decisions are explicit and not open questions for this milestone:

- bulk routing is a first-class bridge subsystem, not an optimization hidden
  inside existing route planning
- planning, packetization, reduction, and parallel-admission classification are
  all bridge-owned proof-bearing phases
- the executor consumes lowered bridge plans only; it does not rediscover
  grouping, reduction, or legality
- legality and profitability are separate decisions; legal parallel preparation
  is not automatically profitable parallel preparation
- canonical packetized reads are the authority for broad bridge read surfaces;
  scalar convenience reads remain layered on top
- parallelism in this milestone is preparation-only and legality-bearing; it
  does not make authoritative truth mutation or final truth publication parallel
- admitted parallel work is classified from structural disjointness proofs
  carried by the plan, not by speculative runtime locking
- reduced routing outputs remain deterministic and replay-safe even when the
  bridge chooses a wider bulk path
- counters and decision-log truth are part of the operational contract, not
  debug garnish

Normative consequence:

- host-local batching wrappers around per-item route planning are out of spec
- delivery-time packet coalescing or regrouping is out of spec
- thread-pool use without canonical legality markers is out of spec
- hidden coarsening under "bulk mode" is out of spec
- bridge APIs that hide broad planning and packetization behind getter-shaped
  calls are out of spec

## Scope

### In Scope

- bridge-owned bulk planning vocabulary and artifacts
- canonical workload summaries derived once from route, slice, continuity, and
  truth-view inputs
- packetized bridge planning for large routing and read surfaces
- canonical reduction of overlapping invalidation, continuity, and truth-view
  planning outputs
- explicit legality classification for admitted serial work versus admitted
  parallel preparation work
- counters and decision records for routed width, reduction width, packet
  breadth, fallback classes, and parallel-admission outcomes
- replay-safe bulk planning artifacts and reduction records
- harness certification for large patchsets, broad packet sets, deterministic
  reduction, diagnostics-tier invariance, and admitted parallel-preparation
  parity

### Explicitly Out Of Scope

- general multi-consumer change stream protocol productization
- generalized reactive source protocol productization across all host shapes
- merge-aware bridge semantics across multi-parent truth histories
- speculative truth-branch to signal-branch coordination beyond the legality
  groundwork needed here
- bridge-mediated writeback or commit strategies
- scheduler-owned parallel execution semantics inside `worth-signal`

Milestone 5 must remain focused on the bridge scale path itself, not absorb
later protocol and speculative features.

## Governing Design Rules

### 1. Bulk Planning Must Be Authoritative Bridge Truth

Milestone 5 must introduce one bridge-owned bulk planning boundary that lowers:

- canonical committed patch truth
- canonical normalized truth-delta surfaces
- canonical continuity context
- canonical truth-view authority
- frozen mapping authority

into one canonical workload summary and one canonical bulk bridge plan.

The bridge must answer:

- what work items exist?
- how were they packetized?
- how were overlapping outcomes reduced?
- what legality basis admits serial or parallel preparation?

It must not answer:

- what truth mutation semantically means
- what the signal scheduler should do after delivery
- whether parallel execution in the compute runtime is desirable

The governing operational rule is the same one `worth-relational` already uses
successfully:

`parallelize disposable work, serialize authority`

### 1.1 Planning Context Identity Must Be Explicit

Milestone 5 must not let packet proofs, legality proofs, or reduced artifacts
float free from the exact planning context that produced them.

The bridge must introduce a first-class canonical planning identity carrying at least:

- workload digest
- mapping registry digest
- truth-view authority digest
- continuity semantics digest
- packetization semantics version
- reduction semantics version

Required rule:

- if any planning-context identity-bearing field changes, the bridge must treat
  prior packet proofs and legality decisions as invalid and require replanning

Replay and diagnostics must be able to answer not only "what plan ran?" but
also "what exact planning context made that plan legal and canonical?"

Milestone 5 must also introduce a separate admission/profile identity for
profitability and execution-profile-sensitive decisions.

That separate identity may include:

- adapter capability digest
- diagnostics tier
- runtime profitability policy digest
- bounded buffering ceiling profile

Rules:

- canonical packetization, canonical reduction, and replay-visible plan truth
  must depend only on canonical planning identity
- legality may depend on canonical planning identity plus proof-bearing
  disjointness artifacts
- profitability and selected mode may depend on admission/profile identity
- diagnostics-tier changes must not change canonical plan identity unless the
  diagnostics policy itself changes semantic planning authority, which Milestone
  5 does not admit

### 2. Packetization Must Be Planned, Canonical, And Cost-Honest

Milestone 5 must treat packetization as a first-class bridge artifact, not as a
temporary vector built inside planning.

Admitted packet classes should include at least:

- `TruthDeltaRoutingPacket`
- `TruthViewMaterializationPacket`
- `ContinuityRemapPacket`
- `FallbackAggregationPacket`
- `InvalidationReductionPacket`

Each admitted packet class must define:

- identity-bearing fields
- canonical ordering key
- deduplication basis
- digest basis
- explanatory-only fields excluded from identity

Rules:

- packet breadth is derived during planning, not discovered during delivery
- packet identity remains stable across diagnostics tiers
- packetization may batch work broadly only when the resulting packet still
  reflects honest semantic scope
- scalar convenience surfaces must compile down to packetized bridge truth, not
  vice versa
- packet proofs are valid only for the exact planning context that produced
  them; changing canonical workload identity, mapping authority, truth-view
  authority, or continuity basis invalidates packet reuse and requires replanning

### 2.1 Packet Key Contracts Must Be Written Before Implementation

Milestone 5 must follow the same standard `worth-relational` used for Phase 8:
each packet family owns an explicit canonical key contract before code lands.

Required packet key contracts for Milestone 5:

- `TruthDeltaRoutingPacket`
  - `(workload_identity, route_member_identity, truth_surface_kind, truth_surface_scope, mapped_slice_scope, packet_index)`
- `TruthViewMaterializationPacket`
  - `(workload_identity, truth_view_member_identity, branch_identity, snapshot_selector_kind, snapshot_identity, packet_index)`
- `ContinuityRemapPacket`
  - `(workload_identity, continuity_member_identity, prior_slice_scope, continuity_class, successor_scope_class, packet_index)`
- `FallbackAggregationPacket`
  - `(workload_identity, originating_member_identity, fallback_class, bounded_scope_identity, packet_index)`
- `InvalidationReductionPacket`
  - `(workload_identity, reduction_family, reduced_target_scope, reduced_target_identity, packet_index)`

Rules:

- `packet_index` exists only to preserve canonical ordering among otherwise
  equal members inside the same packet family; it must never be the only
  distinguishing field
- `originating_member_identity` must resolve to one canonical upstream route,
  slice, continuity, or truth-view member identity carried by the workload request
- every packet family must define which fields are identity-bearing versus
  explanatory-only before implementation begins
- packet key tuples may not be widened or reordered during implementation
  without updating the spec first

### 3. Reduction Must Be Deterministic And Explicit

Milestone 5 must not allow overlapping mapping hits, continuity outcomes,
truth-view requests, or fallback routes to reduce by host iteration order.

Reduction must define:

- ordered input basis
- reduction grouping key
- duplicate collapse rule
- conflict or overlap classification
- canonical output ordering
- digest basis for reduced artifacts

At minimum, canonical reduction must cover:

- duplicate routing entries
- duplicate subscription slices
- continuity-driven remap overlap
- repeated truth-view packet requests
- fallback and unsupported-path aggregation
- packet-local and plan-wide reduction summaries

If a later replay cannot reconstruct why N input items became M reduced outputs,
the reduction contract is incomplete.

Reducers own final observable ordering. Worker completion order, packet arrival
order, and thread scheduling may not affect any canonical bridge artifact.

### 3.1 Reduction Identity Must Be Separate From Packet Identity

Milestone 5 must explicitly distinguish:

- packet identity: "what worker-local or packet-local unit of planned work exists?"
- reduction identity: "what canonical observable bridge output does that work reduce into?"

Required rule:

- multiple packets may reduce into one reduction identity only when the reducer
  contract declares that merge explicitly
- same reduction identity with incompatible payloads is a typed
  `ReductionIdentityConflict`, not a best-effort merge
- packet overlap in a supposedly disjoint planning region is a typed
  `PacketOverlapDetected`, not a reducer concern

### 3.1.1 Reduced Output Identity Contracts Must Be Written Before Implementation

Milestone 5 must define exact reduced-output identity tuples before any packet
family implementation begins.

Required reduced-output identity contracts:

- `ReducedRoutingTargetIdentity`
  - `(workload_identity, truth_surface_scope, mapped_slice_scope, canonical_target_identity)`
- `ReducedTruthViewIdentity`
  - `(workload_identity, truth_view_member_identity, branch_identity, snapshot_identity, canonical_truth_view_scope)`
- `ReducedContinuityIdentity`
  - `(workload_identity, continuity_member_identity, prior_slice_scope, continuity_class, canonical_successor_scope)`
- `ReducedFallbackIdentity`
  - `(workload_identity, fallback_class, bounded_scope_identity, originating_family)`
- `ReducedPublicationIdentity`
  - `(workload_identity, publication_family, canonical_target_identity, canonical_truth_view_scope, canonical_continuity_scope)`

Rules:

- reduced-output identities must be sufficient to reconstruct why multiple
  packet-local outputs collapsed into one observable bridge output
- same reduced-output identity with mismatched payload is always a typed
  `ReductionIdentityConflict`
- reduced-output identity tuples define observable reduction truth; packet
  families may not substitute alternative ad hoc identities during implementation

### 3.2 Reducer Topology Must Be Explicit

Milestone 5 must not leave cross-family reduction topology as an implementation
detail.

The reducer topology for this milestone is:

1. family-local reduction
2. canonical cross-family publication reduction

Rules:

- each packet family performs its own local canonical reduction first
- family-local reducers emit typed reduced outputs with explicit reduction identities
- cross-family publication reduction consumes those typed reduced outputs only
- cross-family publication reduction owns the final observable ordering across
  routing, truth-view, continuity, fallback, and invalidation families
- no unified catch-all reducer may rediscover family semantics after local reduction

Required family-local reducer outputs:

- `TruthDeltaRoutingPacket` -> `ReducedRoutingTargetIdentity`
- `TruthViewMaterializationPacket` -> `ReducedTruthViewIdentity`
- `ContinuityRemapPacket` -> `ReducedContinuityIdentity`
- `FallbackAggregationPacket` -> `ReducedFallbackIdentity`
- `InvalidationReductionPacket` -> `ReducedPublicationIdentity`

### 4. Parallel Admission And Profitability Must Be Proof-Carrying And Narrow

Milestone 5 may admit parallel preparation only through explicit lowered
legality proofs and explicit profitability decisions.

Required classes:

- `SerialRequired`
- `ParallelPreparationAdmitted`
- `ParallelPreparationRejected`

Required decision dimensions:

- `parallel_legality`
- `parallel_profitability`
- `selected_mode`
- `fallback_reason`

Required proof-bearing legality artifacts:

- `ParallelPreparationLegalityProof`
- `DisjointPacketRegionSet`
- `AdmittedPreparationPartitionSet`

Required typed admission and fallback reasons:

- `SerialExecutor`
- `BelowMinWorkloadWidth`
- `BelowPolicyWorkThreshold`
- `TruthViewMaterializationHeavy`
- `CrossBranchSharedSurface`
- `AdapterDoesNotSupportParallelPreparation`
- `PacketOverlapDetected`
- `AdmittedOperational`
- `AdmittedDevelopment`
- `AdmittedForensic`

Rules:

- legality is derived from structural disjointness of bridge-owned work packets
- legality proofs are carried by the bulk plan
- `AdmittedBridgeExecutionPlan` must be impossible to construct without a
  legality proof artifact produced by planning
- profitability is evaluated from explicit workload width, packet width,
  reduction width, diagnostics tier, and locality facts already available in planning
- delivery consumes legality-bearing plans and must not upgrade a rejected class
  to admitted at runtime
- admitted parallel preparation must not change canonical ordering or reduction truth
- authoritative truth mutation, canonical truth publication, and final signal
  scheduling remain outside this milestone's parallelization claim

Milestone 5 does not admit speculative locking or contention-based discovery as
parallel-safety authority.

Legal but unprofitable workloads must fall back explicitly to serial preparation
with a typed recorded reason, not silently.

### 4.1 Locality Must Participate In Legality And Profitability

Milestone 5 must not evaluate legality or profitability from semantic
disjointness alone.

The bridge must carry a first-class locality footprint, such as:

```rust
pub struct BridgeLocalityFootprint {
    branch_scope: CanonicalBranchScope,
    snapshot_scope: CanonicalSnapshotScope,
    mapping_partition_scope: CanonicalMappingPartitionScope,
    continuity_scope: CanonicalContinuityScope,
    truth_view_scope: CanonicalTruthViewScope,
}
```

Rules:

- legality may depend on structural disjointness plus locality boundaries
- profitability must consider locality, not only packet count or workload width
- plans that are semantically disjoint but mechanically scattered may be legal
  and still explicitly unprofitable

### 5. Plan / Packetize / Reduce / Admit / Deliver Separation Is Mandatory

Milestone 5 extends the bridge proof chain:

- canonical bridge workload request
- normalized workload summary
- planned packet set
- reduced workload artifact
- legality-bearing execution plan
- delivered bulk routing result

Packetization must not be rediscovered during reduction.
Reduction must not be rediscovered during delivery.
Parallel legality must not be rediscovered during execution.

### 6. Scale-Path Types Must Carry Proof

Milestone 5 must continue satisfying Architectural Laws 30 and 41.

Representative progression:

```rust
pub struct CanonicalBridgeWorkloadRequest { ... }
pub struct NormalizedBridgeWorkloadSummary { ... }
pub struct PlannedBridgePacketSet { ... }
pub struct ReducedBridgeWorkloadArtifact { ... }
pub struct AdmittedBridgeExecutionPlan { ... }
pub struct DeliveredBulkBridgeResult { ... }
```

Rules:

- constructors for proof-bearing scale-path packets must be sealed to proving
  modules
- later phases must consume the exact proof-bearing type produced upstream
- packetization breadth, reduction classes, fallback classes, and legality
  outcomes become part of the proof chain rather than explanation-only side channels
- delivery cannot accept weaker route bags or raw packet vectors once a lowered
  plan exists

Worker-local preparation outputs, if parallel preparation is admitted, may emit
only packet-local observations, packet-local counters, and packet-local
fragments. Reducers produce the only authoritative observable bridge outputs.

### 6.1 Buffering And Diagnostics Ceilings Must Be Explicit

Milestone 5 must define operational ceilings so scale-path observability does
not become a hidden second hot path.

Required bounded surfaces:

- packet queue depth
- reducer input buffering
- explanation fragment counts
- decision-log fragment counts
- in-flight packet-local artifact counts

Rules:

- if a ceiling is exceeded, the bridge must emit a typed, bounded fallback or
  rejection rather than silently allocating wider
- diagnostics expansion must remain policy-shaped and must never alter canonical
  routing, reduction, or legality truth

### 7. Canonicality Must Be Mechanically Declared

For every Milestone 5 canonical artifact, the spec must define:

- ordered input set
- ordering key
- deduplication rule
- digest basis
- identity-bearing versus explanatory-only fields

Canonicality must cover at least:

- workload request ordering
- workload-summary ordering
- packet-set ordering
- packet-entry ordering
- reduction grouping and output ordering
- legality decision ordering
- plan-wide counter and decision-log record ordering
- replay record ordering

If any scale-path artifact can vary because a host uses a different map, packet
builder, thread count, or registration order, the design is out of spec.

### 8. Cost Must Be Visible At The Bulk Boundary

Milestone 5 must satisfy the performance guidelines structurally.

The bridge must surface first-class counters for at least:

- routed item count
- normalized workload width
- reduced output width
- packet count
- packet entry count
- packet coalescing count
- fallback count
- unsupported-path count
- serial-required count
- parallel-preparation-admitted count
- parallel-preparation-rejected count
- replay mismatch count

Rules:

- counters belong to canonical plan/result artifacts
- exact counter assertions are required in certification scenarios
- a performance claim without a named counter and a proof test remains out of spec
- diagnostics richness may add explanation detail, but it must not alter counter truth

### 9. Precision Must Survive Scale

Milestone 5 may batch and reduce, but it may not erase fine-grained meaning to
do so.

Rules:

- fine-grained slice identity remains the reduction substrate
- continuity-backed remaps remain explicit after reduction
- truth-view authority remains explicit after packetization
- fallback classes remain typed and visible after bulk planning
- bulk mode must not silently widen to entity-wide or branch-wide routing unless
  an already-admitted typed fallback class requires it

### 10. Diagnostics Are Derived From Canonical Plan Truth

Operational truth for Milestone 5 is:

- canonical workload identity
- packet-set identity
- reduced workload identity
- legality identity
- counters
- typed fallback and rejection classes

Rich explanations, packet annotations, and performance narratives remain derived
under diagnostics policy.

## Target Runtime Model

### 1. Public Surface Growth

Milestone 5 should extend `worth-runtime-bridge` with scale-path concepts such as:

```rust
pub struct BridgeBulkWorkloadRequest { ... }
pub struct BridgeBulkPlan { ... }
pub struct BridgeBulkPlanResult { ... }
pub struct BridgePacketSet { ... }
pub struct ReducedBridgeArtifact { ... }
pub struct BridgeParallelAdmission { ... }

pub enum BridgeParallelAdmissionClass {
    SerialRequired,
    ParallelPreparationAdmitted,
    ParallelPreparationRejected,
}
```

Design rules:

- planning and delivery remain separate public boundary crossings
- public surfaces expose bridge nouns only
- bulk planning extends existing route, continuity, and truth-view authority;
  it does not create a separate host-local batching API
- callers must not be able to trigger hidden broad planning through innocent
  getter-shaped surfaces

### 2. Bulk Workload Input Contract

Milestone 5 needs one canonical workload request built from prior milestone
artifacts.

Representative shape:

```rust
pub struct CanonicalBridgeWorkloadRequest {
    workload_identity: BridgeWorkloadIdentity,
    route_members: Vec<BridgeRouteIdentity>,
    slice_members: Vec<BridgeSliceArtifactIdentity>,
    continuity_members: Vec<BridgeContinuityIdentity>,
    truth_view_members: Vec<BridgeTruthViewIdentity>,
    commit_members: Vec<TruthCommitIdentity>,
    snapshot_members: Vec<TruthSnapshotIdentity>,
    branch_members: Vec<TruthBranchIdentity>,
    workload_segments: Vec<BridgeWorkloadSegment>,
    digest: BridgeWorkloadDigest,
}
```

Rules:

- the workload request is derived exactly once from canonical upstream bridge truth
- identity-bearing fields must be sufficient for replay and certification
- the request must not depend on diagnostics-only richness or host batching hints
- scalar route, slice, continuity, truth-view, branch, commit, and snapshot
  identities are derived members of the workload request rather than the
  top-level request shape itself
- the top-level request must remain a true batch declaration rather than a
  scalar route event wrapped in a bulk shell

Milestone 5 should also introduce a companion planning-context artifact shaped like:

```rust
pub struct BridgeCanonicalPlanningIdentity {
    workload_digest: BridgeWorkloadDigest,
    mapping_registry_digest: BridgeMappingRegistryDigest,
    truth_view_digest: BridgeTruthViewDigest,
    continuity_digest: Option<BridgeContinuityDigest>,
    packetization_semantics_version: u16,
    reduction_semantics_version: u16,
}

pub struct BridgeAdmissionProfileIdentity {
    adapter_capability_digest: BridgeAdapterCapabilityDigest,
    diagnostics_tier: BridgeDiagnosticsTier,
    profitability_policy_digest: BridgeProfitabilityPolicyDigest,
    buffering_ceiling_profile: BridgeBufferingCeilingProfileDigest,
}
```

### 3. Workload Summary And Packet Contract

Milestone 5 needs explicit normalized workload summaries and packet sets.

Representative shape:

```rust
pub struct NormalizedBridgeWorkloadSummary {
    workload_identity: BridgeWorkloadIdentity,
    routing_entries: CanonicalRoutingEntries,
    read_scope: CanonicalReadScope,
    continuity_scope: CanonicalContinuityScope,
    counters: BridgeBulkPlanningCounters,
}

pub struct PlannedBridgePacketSet {
    workload_identity: BridgeWorkloadIdentity,
    routing_packets: Vec<TruthDeltaRoutingPacket>,
    truth_view_packets: Vec<TruthViewMaterializationPacket>,
    continuity_packets: Vec<ContinuityRemapPacket>,
    fallback_packets: Vec<FallbackAggregationPacket>,
    reduction_packets: Vec<InvalidationReductionPacket>,
    counters: BridgeBulkPlanningCounters,
}
```

Rules:

- the workload summary derives shared facts exactly once and passes them forward immutably
- packet sets carry enough proof to make delivery monomorphic
- packet ordering is canonical and replay-safe
- packet classes remain structurally separate rather than collapsing into one
  generic `Vec<Packet>`

### 4. Reduction And Legality Contract

Milestone 5 needs explicit reduced artifacts and legality-bearing plans.

Representative shape:

```rust
pub struct ReducedBridgeWorkloadArtifact {
    workload_identity: BridgeWorkloadIdentity,
    reduced_slices: CanonicalSubscriptionSlices,
    reduced_truth_views: CanonicalTruthViewPackets,
    reduced_fallbacks: CanonicalFallbackClasses,
    counters: BridgeBulkPlanningCounters,
}

pub struct AdmittedBridgeExecutionPlan {
    workload_identity: BridgeWorkloadIdentity,
    reduced_artifact: ReducedBridgeWorkloadArtifact,
    parallel_admission: BridgeParallelAdmission,
    selected_mode: BridgePreparationMode,
    planning_identity: BridgeCanonicalPlanningIdentity,
    admission_profile: BridgeAdmissionProfileIdentity,
    legality_proof: ParallelPreparationLegalityProof,
    counters: BridgeBulkPlanningCounters,
}
```

Rules:

- reduction output is canonical bridge truth
- legality classification is explicit and typed
- reduction work on shared surfaces is limited to deterministic publication-only
  merge/reduction, never semantic rediscovery
- execution consumes admitted plans only
- the legality surface must be narrow enough that later optimized implementations
  can change execution strategy without changing the public proof chain

### 5. Bridge-Owned Adapter Growth

Milestone 5 should continue depending on narrow bridge-owned contracts.

Representative shape:

```rust
pub trait BridgeBulkPlanningAdapter {
    type Error;

    fn build_workload_summary(
        &self,
        request: &CanonicalBridgeWorkloadRequest,
    ) -> Result<NormalizedBridgeWorkloadSummary, Self::Error>;

    fn build_packet_set(
        &self,
        summary: &NormalizedBridgeWorkloadSummary,
    ) -> Result<PlannedBridgePacketSet, Self::Error>;
}

pub trait BridgeBulkDeliveryAdapter {
    type Error;

    fn deliver_bulk_plan(
        &self,
        plan: AdmittedBridgeExecutionPlan,
    ) -> Result<DeliveredBulkBridgeResult, Self::Error>;
}
```

Rules:

- the bridge owns the bulk planning and delivery adapter contracts
- parent runtimes implement only the narrow seams required for canonical
  packetization and delivery
- broad facade reach-through remains out of spec

## Phases

### Phase 1: Canonical Workload And Packet Authority

Phase 1 exists to make scale-path work structurally representable before any
parallel or performance claims are made.

Milestone 5 must first define:

- one canonical bulk workload request surface
- one normalized workload-summary surface
- admitted packet classes and packet identity rules
- exact canonical ordering, deduplication, and digest bases for workload and packet truth
- the closed vocabulary of fallback and unsupported-path aggregation classes

This phase leaves the system in a coherent state where:

- large bridge workloads are described once through bridge-owned artifacts
- packet breadth is explicit before delivery exists
- no host-local batching wrapper can honestly substitute for bridge planning truth

### Phase 2: Deterministic Reduction And Parallel-Admission Planning

Phase 2 exists to turn canonical workload and packet truth into executable,
bounded, legality-bearing bridge plans.

Milestone 5 must then implement:

- deterministic packet-set planning from normalized workload summaries
- deterministic reduction of overlapping slices, truth-view requests,
  continuity outcomes, and fallback classes
- legality classification into serial-required, parallel-preparation-admitted,
  or parallel-preparation-rejected
- exact scale-path counters and canonical decision-log records
- admitted execution plans that freeze packetization, reduction, and legality before delivery

This phase leaves the system in a coherent state where:

- identical canonical workloads lower to identical reduced plans
- broad bridge work is bounded by explicit packet sets and reduction summaries
- parallel-preparation claims are explicit, narrow, and replay-safe

### Phase 3: Bulk Delivery, Replay, And Scale Certification

Phase 3 exists to prove that the scale path is trustworthy instead of merely
plausible.

Milestone 5 must finally ship:

- bulk delivery surfaces that consume admitted execution plans only
- canonical plan records, replay records, and explanation reconstruction
- hostile harness suites covering large patchsets, wide packet sets,
  deterministic reduction, diagnostics-tier invariance, and admitted
  parallel-preparation parity
- exact counter assertions for named workload-width, packet-width, reduction-width,
  and legality-class scenarios

This phase leaves the system in a coherent state where:

- the bridge can certify scale behavior mechanically
- replay after restart validates planning, reduction, and legality parity directly
- later protocol, merge-aware, and speculative milestones can extend a trustworthy scale substrate

## Must Ship

- canonical bulk workload request and workload-summary artifacts
- canonical packet-set planning for routing, snapshot reads, continuity work,
  truth-view work, and reduction work
- deterministic reduction artifacts and overlap classifications
- typed legality classification for serial versus admitted parallel preparation
- counters and decision-log records for workload width, packet breadth,
  reduction width, fallback classes, and legality outcomes
- typed failures for workload-summary construction, packet construction,
  reduction mismatch, unsupported packet class, invalid legality basis,
  bulk delivery rejection, and replay mismatch
- replay-safe canonical plan records and derived explanations
- harness certification lanes for large patchsets, deterministic reduction,
  diagnostics-tier invariance, and parallel-preparation parity

## Must Preserve

- truth runtime remains the authority for truth semantics, history, retention,
  and mutation
- signal runtime remains the authority for dependency ownership, scheduling,
  and execution
- no live mutable truth reads during bulk planning or delivery
- no hidden loss of fine-grained precision under load
- no executor rediscovery of packetization, reduction, or legality semantics
- canonical ordering and replay-safe identities
- explicit branch and truth-view authority through the scale path
- clean facade boundaries rather than wide parent-runtime reach-through

## Acceptance Evidence

Milestone 5 is complete only when the bridge harness can prove:

- identical canonical large workloads lower to identical bulk plans and reduced artifacts
- large patchsets route through planned bulk paths rather than per-item ad hoc handlers
- packet breadth and reduction width remain explicit and diagnostics-tier-invariant
- fine-grained routing precision survives bulk planning and reduction
- legality-bearing admitted parallel preparation remains parity-safe with the
  serial-required path where both are admitted
- hostile packet completion orders and worker schedules produce byte-identical
  reduced artifacts and canonical counters
- replay from canonical bulk plan artifacts matches original planning and
  reduction behavior
- unsupported packet classes or invalid legality bases fail explicitly and typed
- counters explain routed width, packet width, reduction width, fallback
  behavior, and legality outcomes honestly

## Architectural Notes

### Expected Internal Subdomains

Milestone 5 should extend the bridge crate with subdomains such as:

- `planning/workload/`
- `planning/packets/`
- `planning/reduction/`
- `planning/admission/`
- `planning/counters/`
- `delivery/bulk/`
- `diagnostics/plans/`
- `harness/fixtures/bulk_patchsets.rs`
- `harness/fixtures/reduction_overlap.rs`
- `harness/fixtures/parallel_admission.rs`

This follows workspace domain standards:

- workload normalization is not the same responsibility as packet construction
- packet construction is not the same responsibility as reduction
- legality classification is not the same responsibility as delivery
- canonical plan records are not the same responsibility as explanation reconstruction

### Minimum Counter Floor

Milestone 5 must add counters such as:

- `bulk_workload_count`
- `bulk_routed_item_count`
- `bulk_normalized_workload_width`
- `bulk_packet_count`
- `bulk_packet_entry_count`
- `bulk_packet_queue_depth_peak`
- `bulk_reducer_input_buffer_peak`
- `bulk_reduction_input_count`
- `bulk_reduction_output_count`
- `bulk_fallback_count`
- `bulk_unsupported_path_count`
- `bulk_serial_required_count`
- `bulk_parallel_legal_count`
- `bulk_parallel_profitable_count`
- `bulk_parallel_preparation_admitted_count`
- `bulk_parallel_preparation_rejected_count`
- `bulk_parallel_fallback_to_serial_count`
- `bulk_replay_mismatch_count`

Exact names may refine during implementation, but the structural floor is not
optional.

### Explicit Legality And Failure Policy

Milestone 5 must carry legality and failure outcomes structurally rather than
narratively.

Required legality classes:

- `SerialRequired`
- `ParallelPreparationAdmitted`
- `ParallelPreparationRejected`

Required failure classes:

- `UnsupportedPacketClass`
- `InvalidReductionBasis`
- `InvalidParallelAdmissionBasis`
- `ParallelPreparationNotProfitable`
- `PacketOverlapDetected`
- `ReductionIdentityConflict`
- `ReducerBufferCeilingExceeded`
- `DiagnosticsFragmentCeilingExceeded`
- `BulkPlanReplayMismatch`
- `BulkDeliveryRejected`

Rules:

- every canonical workload receives exactly one legality outcome
- legality remains visible in canonical plan truth
- failure must include the planning boundary that failed
- legality rejection must not degrade into untracked "best effort serial mode"

## Test And Harness Model

Milestone 5 must follow the same structural testing discipline as earlier
bridge milestones.

Expected first-class test surfaces:

- large patchset routing scenarios
- wide packet-set planning scenarios
- reduction-overlap scenarios
- legality admission and rejection scenarios
- hostile scheduling and packet completion order scenarios
- diagnostics-tier invariance scenarios
- replay parity and replay drift scenarios
- counter certification scenarios

Milestone 5 is not complete with only direct fixture tests. It must establish a
real scale-path certification surface on top of `worth-harness`.

Expected harness surfaces:

- `ScenarioPlan` and `ScenarioFixture` for wide bridge workloads
- `MutationBatch` for large committed patchsets, overlap-heavy routes, and
  historical/branch-local bulk requests
- `ExecutionRequest` for workload planning, packet construction, reduction,
  delivery, replay, and diagnostics capture
- `ExecutionProfile` for deterministic, replay, diagnostics-tier, and
  admitted-parallel-preparation sweeps
- `ParitySuite` for profile-to-profile bulk-plan parity
- `CertificationMatrix` for adversarial workload-width and legality coverage

Minimum certification families:

- fixed deterministic large-patch workload fixtures
- fixed deterministic reduction-overlap fixtures
- seeded wide-packet and mixed-branch workload matrices
- hostile scheduling matrices where packet completion order is intentionally perturbed
- replay-after-restart bulk-plan certification from canonical plan artifacts
- invalid legality-basis rejection certification
- exact counter assertions for named workload-width and packet-width scenarios

Minimum representative test names:

- `tests::planning::large_patchset_lowers_to_canonical_bulk_plan`
- `tests::planning::overlapping_slice_routes_reduce_deterministically`
- `tests::planning::packetized_truth_view_requests_remain_canonical_under_host_reordering`
- `tests::planning::parallel_preparation_admission_remains_parity_safe_with_serial_required_path`
- `tests::planning::hostile_packet_completion_order_preserves_canonical_reduction`
- `tests::planning::replayed_bulk_plan_matches_original_canonical_artifact`

## Target API And Module Plan

### New Files Expected

- `crates/worth-runtime-bridge/src/planning/workload/mod.rs`
- `crates/worth-runtime-bridge/src/planning/workload/request.rs`
- `crates/worth-runtime-bridge/src/planning/workload/summary.rs`
- `crates/worth-runtime-bridge/src/planning/packets/mod.rs`
- `crates/worth-runtime-bridge/src/planning/packets/routing.rs`
- `crates/worth-runtime-bridge/src/planning/packets/snapshot.rs`
- `crates/worth-runtime-bridge/src/planning/packets/continuity.rs`
- `crates/worth-runtime-bridge/src/planning/packets/reduction.rs`
- `crates/worth-runtime-bridge/src/planning/reduction/mod.rs`
- `crates/worth-runtime-bridge/src/planning/reduction/grouping.rs`
- `crates/worth-runtime-bridge/src/planning/reduction/lowering.rs`
- `crates/worth-runtime-bridge/src/planning/admission/mod.rs`
- `crates/worth-runtime-bridge/src/planning/admission/legality.rs`
- `crates/worth-runtime-bridge/src/planning/admission/counters.rs`
- `crates/worth-runtime-bridge/src/delivery/bulk.rs`
- `crates/worth-runtime-bridge/src/diagnostics/plans.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/bulk_patchsets.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/reduction_overlap.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/parallel_admission.rs`
- `crates/worth-runtime-bridge/src/tests/planning/bulk_workloads.rs`
- `crates/worth-runtime-bridge/src/tests/planning/reduction.rs`
- `crates/worth-runtime-bridge/src/tests/planning/admission.rs`
- `crates/worth-runtime-bridge/src/tests/planning/replay.rs`

### Existing Files Expected To Change

- [facade.rs](../../crates/worth-runtime-bridge/src/facade.rs)
- [planning.rs](../../crates/worth-runtime-bridge/src/routing/planning/mod.rs)
- [lowering.rs](../../crates/worth-runtime-bridge/src/routing/lowering/mod.rs)
- [packet.rs](../../crates/worth-runtime-bridge/src/snapshot/packet.rs)
- [mod.rs](../../crates/worth-runtime-bridge/src/harness/mod.rs)
- [adapter.rs](../../crates/worth-runtime-bridge/src/harness/adapter/mod.rs)

## Implementation Phases

Milestone 5 must execute in strict order. Later phases may reopen earlier ones,
but no phase may bypass unfinished scale foundations with host-local batching
glue or runtime-only parallel heuristics.

### Phase M5.0 - Workload Taxonomy And Bulk Boundary Lock

Purpose:

- define the canonical workload request and packet taxonomy
- lock what scale semantics the bridge owns versus what remains runtime authority
- define explicit unsupported packet and legality classes

Required work:

- inventory the route, continuity, and truth-view surfaces that must
  participate in one bulk workload request
- define the first closed Milestone 5 packet taxonomy
- define canonical workload and packet digest bases
- define the legality vocabulary and rejection policy

Exit criteria:

- the workload boundary is singular and explicit
- unsupported packet classes are named rather than deferred
- there is no unresolved ambiguity about whether bulk planning lives in the bridge

### Phase M5.1 - Workload Summary And Packet Planning

Purpose:

- derive shared workload facts once and freeze packet breadth before reduction exists

Required work:

- define `CanonicalBridgeWorkloadRequest`
- define `NormalizedBridgeWorkloadSummary`
- define `PlannedBridgePacketSet`
- canonicalize duplicate packet entries
- define exact packet ordering and digest bases

Exit criteria:

- the bridge can derive one canonical packet set per workload
- packet breadth is explicit before delivery and reduction
- later phases do not need to rediscover planning breadth

### Phase M5.2 - Canonical Reduction And Parallel-Admission Classification

Purpose:

- lower packet truth into reduced artifacts and legality-bearing execution plans

Required work:

- define `ReducedBridgeWorkloadArtifact`
- define `AdmittedBridgeExecutionPlan`
- define deterministic overlap reduction and fallback aggregation
- classify workloads into serial-required, parallel-preparation-admitted, or
  parallel-preparation-rejected
- record legality and profitability as separate decisions with explicit serial
  fallback when work is legal but not profitable
- define typed admission reasons and locality-footprint participation in admission
- add exact counters and decision-log records

Exit criteria:

- identical packet sets lower to identical reduced plans
- legality outcomes are typed and replay-safe
- counters and digest bases are specified and test-covered

### Phase M5.3 - Bulk Delivery, Replay, And Certification

Purpose:

- make the scale path certifiable rather than plausible

Required work:

- add bulk delivery integration that consumes admitted plans only
- add canonical plan and replay records
- add explanation reconstruction over canonical scale truth
- add `worth-harness` fixtures, parity suites, and certification matrices for
  wide-workload planning and legality coverage

Exit criteria:

- all roadmap acceptance evidence is covered by bridge-native harness scenarios
- replay validates bulk-plan parity directly
- diagnostics-tier changes richness only, not planning, reduction, or legality truth

## Explicit Failure Taxonomy For Milestone 5

Milestone 5 must ship typed bridge failures for at least:

- unsupported packet class
- workload summary construction failure
- packet digest inconsistency
- invalid reduction basis
- invalid parallel-admission basis
- packet overlap detected
- reduction identity conflict
- legal-but-unprofitable parallel preparation fallback
- reducer buffer ceiling exceeded
- diagnostics fragment ceiling exceeded
- bulk delivery rejection
- bulk plan decode or compatibility failure
- bulk plan replay mismatch

These are bridge failures, not raw parent-runtime strings.

## Anti-Patterns Explicitly Rejected

- implementing bulk routing as a loop around per-item Milestone 1 or 2 planning calls
- treating packet vectors as internal scratch state with no canonical identity
- performing reduction by map iteration order or host insertion order
- claiming parallel safety from runtime lock behavior instead of lowered legality proofs
- hiding coarsening or fallback expansion inside bulk mode
- re-deciding packet grouping, reduction, or legality during delivery
- exposing parent-runtime internals as the bulk planning API
- reporting scale behavior only through elapsed time instead of explanatory counters

## Sequencing Notes

Milestone 5 must land before:

- reactive source protocol productization, because source contracts should
  expose packetized and bulk-honest read shapes
- structural-identity-aware remapping, because added remap breadth should build
  on a canonical reduction substrate
- merge-aware bridge semantics, because merge-bearing histories will widen
  routing and reduction breadth
- speculative truth-branch to signal-branch coordination, because that work
  needs explicit legality-bearing parallel preparation rather than heuristics
- end-to-end causality certification, because the bridge cannot be fully
  certifiable while its scale path remains opaque

Milestone 5 must not attempt to pre-solve:

- multi-consumer protocol contracts
- merge ontology
- speculative branch lifecycle
- bridge-mediated writeback
- scheduler execution policy inside `worth-signal`

Those become stronger because Milestone 5 exists; they do not need to be
smuggled into it.

## Self-Check

This milestone solves a real structural problem rather than packaging work
cosmetically because the bridge cannot honestly claim product-grade scale until
planning breadth, reduction semantics, and legality admissions are explicit.

The adversarial constraint is load-bearing because it forbids the easy failure
mode of per-item loops, hidden coarsening, delivery-time rediscovery, and
performance claims with no explanatory counters.

The milestone preserves authority boundaries because truth still owns truth and
history semantics, signal still owns execution semantics, and the bridge owns
only the scale-path contract and canonical plan artifacts between them.

The milestone defines proof obligations rather than implementation chores
because deterministic packetization, deterministic reduction, explicit legality
classification, replay parity, and counter-certified scale behavior are required
for closeout.

A competent engineer should be able to map this spec into honest types,
planning subdomains, packet sets, counters, legality proofs, and harness suites
without inventing the architecture during implementation.

## Closeout Standard

Milestone 5 is complete only when all of the following are true:

- canonical route, slice, continuity, and truth-view inputs lower into one
  canonical bulk workload request per workload
- workload planning derives one canonical packet set and one canonical reduced
  artifact without host-order drift
- legality classification for serial versus admitted parallel preparation is
  explicit, typed, and replay-safe
- bulk delivery consumes lowered execution plans only
- fine-grained routing precision survives bulk planning and reduction
- counters and decision records explain scale behavior honestly
- bulk planning truth is replay-safe and diagnostics-tier-invariant
- harness certification proves large-workload parity, deterministic reduction,
  admitted parallel-preparation parity, and explicit failure behavior under
  hostile breadth pressure

If code lands but wide bridge workloads still depend on per-item routing loops,
hidden regrouping, nondeterministic reduction, explanation-only legality
decisions, or unmeasured scale claims, Milestone 5 is not complete.
