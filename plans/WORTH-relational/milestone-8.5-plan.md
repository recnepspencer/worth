# Milestone 8.5 Engineering Spec: Extensible Commit Strategies

## Summary

Milestone 8.5 adds extensible commit strategies to `worth-relational` without
creating a second truth-authority path.

This milestone is not "add a strategy trait."

It is:

- commit-plan generalization without authority-path duplication
- proof-bearing strategy request, planning, validation, and replay
- deterministic strategy identity, input, output, and lowering
- strategy-aware lineage, provenance, and persistent naming
- strategy-aware merge classification
- certification-grade containment for failing and hostile strategies

The governing rule remains:

`parallelize disposable work, serialize authority`

Strategy code may produce candidate effects. It may not mutate authority.

Rule 41 is central:

> A type must encode what has been proven about a value.

That means a strategy-bearing commit may not move through the pipeline as an
ordinary "bag of mutations" until the runtime has proven:

- which registered strategy produced it
- which canonical input it consumed
- which deterministic descriptor version it used
- which committed read contract it was allowed to observe
- which canonical output it produced
- which lowered commit plan it became
- which lineage/provenance/naming semantics it declared
- which invariant and replay boundaries it must satisfy

If any later phase can accept a weaker type and silently re-decide an earlier
question, the architecture is wrong.

## 1. Architecture Corrections From Critique

This plan is intentionally structured around the main weaknesses identified in
earlier drafts.

### 1.1 Lowering Boundary Is Explicit

The most important architectural decision is this:

- strategy output does **not** lower directly into plain `WorkerIntentBatch`
  and disappear
- strategy output remains a first-class commit-plan kind through validation,
  replay, and merge metadata
- only final authority execution consumes the ordinary executable mutation batch

Required consequence:

- we keep strategy semantics alive as truth-grade artifacts
- we do not demote strategy meaning into diagnostics-only metadata

The load-bearing lowering product is therefore:

- `LoweredCommitPlan::Strategy(StrategyLoweredCommitPlan)`

not:

- "strategy returns mutations and we WORTHt how they were produced"

### 1.2 Proof Chain Is Minimal, Not Ceremonial

The proof chain is compressed to the minimum honest set of illegal states:

1. `RawStrategyCommitRequest`
   Blocks nothing except malformed ingress.

2. `CanonicalStrategyCommitRequest`
   Proves canonical input, persistent naming, deterministic digest basis, and
   binding to a registered strategy descriptor.

3. `StrategyExecutionDraft`
   Proves the strategy ran against a phase-typed committed read surface and
   produced a candidate output, but proves nothing yet about legality.

4. `StrategyLoweredCommitPlan`
   Proves the draft was lowered into runtime-owned commit semantics with
   structural summaries, lineage plan, naming basis, and provenance attached.

5. `ValidatedStrategyCommitPlan`
   Proves invariants and commit-boundary legality were evaluated against the
   lowered plan.

6. `ExecutedStrategyCommit`
   Proves serialized authority execution succeeded and canonical strategy
   artifacts were persisted.

7. `ReplayVerifiedStrategyCommit`
   Proves replay re-invocation reproduced the same lowered output.

No additional wrappers are allowed unless they block a distinct illegal state.

### 1.3 Constructor Ownership Is Explicit

Every proof-bearing type has one owning module that alone may construct it.

Rules:

- all late-phase constructors are `pub(crate)` or private
- fields on proof-bearing wrappers are private
- transitions are implemented only by the owning phase module
- no blanket `From<Vec<_>>`, `Into`, or generic collection conversions into
  proof-bearing wrappers
- UI compile-fail tests must verify that external code cannot construct or
  bypass late-phase wrappers

### 1.4 Read Cost Is Declared Up Front

The previous draft left read cost too implicit.

This milestone requires strategy registration to declare a read contract:

- exact or bounded query scope classes
- permitted packet types
- allowed traversal basis
- locality class
- worst-case cost class

No strategy executes against an unrestricted read surface.

### 1.5 Determinism Basis Is Explicit

Replay parity is not based on "same strategy and same input" alone.

The deterministic basis must include:

- strategy descriptor digest
- canonical input digest
- input schema version
- runtime schema registry digest
- invariant catalog digest
- lowering semantics version
- canonicalization semantics version
- query planning semantics version
- observed snapshot/version id

Any of those changing must be visible in canonical artifacts.

### 1.6 Merge Semantics Are Structural, Not Taxonomic Only

Strategy-aware merge classification must consider:

- strategy family/class
- strategy semantic identity
- canonical intent scope
- causal metadata
- lineage-affecting declarations
- per-aspect merge policy
- schema reconciliation outcome where relevant

We do not accept "same record touched by different strategies" as sufficient
merge semantics.

### 1.7 Persistent Naming Uses A Real Ontology

Persistent naming is split into distinct domains:

- `CommitStrategySemanticName`
- `CommitStrategyFamilyName`
- `StrategyInputSchemaName`
- `StrategyOutputSchemaName`
- `StrategyIntentName`
- `PersistentArtifactName`

No Rust type name, module path, or display label may become durable identity.

### 1.8 The Second Reference Strategy Is Narrowed

The previous "constraint solver" second strategy was too broad for the first
proof implementation.

Milestone 8.5 uses:

- `IntentReconciliationStrategy`
- `ReplicaConvergenceStrategy`

The second strategy is intentionally Kubernetes-like: desired replica state for
deployments/pods/services is reconciled to current truth through deterministic,
bounded intent planning.

This keeps the adversarial test industrial and strategy-real without opening a
solver-shaped nondeterminism surface too early.

## 2. Milestone Goal

At closeout, the runtime must be able to say:

- authoritative truth still mutates through one serialized commit path
- commit strategies are registered, named, and versioned deterministically
- strategy-produced commits preserve lineage, provenance, and persistent naming
- replay can re-invoke strategies and verify exact lowered output
- merge can classify strategy-bearing histories honestly
- failing or panicking strategies leave zero authoritative residue

The runtime must not merely support extensibility. It must certify it.

## 3. Core Architectural Decision

The shared lifecycle is the abstraction. Strategy effect production is the
parameter.

The true phase chain is:

```text
RawStrategyCommitRequest
  -> CanonicalStrategyCommitRequest
  -> StrategyExecutionDraft
  -> StrategyLoweredCommitPlan
  -> ValidatedStrategyCommitPlan
  -> ExecutedStrategyCommit
  -> ReplayVerifiedStrategyCommit
```

The runtime's ordinary authority pipeline remains shared after the lowering
boundary.

### 3.1 Lowered Commit Plan Union

The main runtime commit-plan shape must become:

```rust
pub enum LoweredCommitPlan {
    Mutation(MutationOnlyCommitPlan),
    Strategy(StrategyLoweredCommitPlan),
}
```

This is the critical architectural correction.

It preserves:

- replay parity semantics
- merge-visible strategy identity
- lineage-aware strategy interpretation
- strategy-specific provenance surfaces

without creating a second executor.

## 4. Required Production Structure

Milestone 8.5 must not accumulate into broad transaction files.

### Production structure

- `src/commit_strategies/data/`
- `strategy_id.rs`
- `descriptor.rs`
- `request.rs`
- `draft.rs`
- `lowered_plan.rs`
- `artifacts.rs`
- `replay.rs`
- `merge.rs`
- `mod.rs`

- `src/commit_strategies/registry/`
- `store.rs`
- `freeze.rs`
- `mod.rs`

- `src/commit_strategies/execution/`
- `context.rs`
- `read_contract.rs`
- `panic_boundary.rs`
- `executor.rs`
- `mod.rs`

- `src/commit_strategies/lowering/`
- `mutation_program.rs`
- `lineage.rs`
- `naming.rs`
- `provenance.rs`
- `summary.rs`
- `mod.rs`

- `src/commit_strategies/replay/`
- `verification.rs`
- `mismatch.rs`
- `mod.rs`

- `src/commit_strategies/merge/`
- `classification.rs`
- `intent_scope.rs`
- `mod.rs`

- `src/commit_strategies/facade.rs`

### Test structure

- `src/tests/commit_strategies/`
- `registry.rs`
- `canonical_request.rs`
- `execution_containment.rs`
- `lowering.rs`
- `validation.rs`
- `replay.rs`
- `merge.rs`
- `intent_reconciliation.rs`
- `replica_convergence.rs`
- `certification.rs`
- `mod.rs`

- `tests/ui/commit_strategies/`
- compile-fail constructor and boundary tests

## 5. Type System Foundation

The type set below is the intended center of the milestone.

### 5.1 Identity, Naming, and Versioning

```rust
pub struct CommitStrategyId(u32);
```

```rust
pub struct CommitStrategySemanticName(Arc<str>);
```

```rust
pub struct CommitStrategyFamilyName(Arc<str>);
```

```rust
pub struct CommitStrategyVersion {
    pub major: u16,
    pub minor: u16,
}
```

```rust
pub struct StrategyInputSchemaName(Arc<str>);
pub struct StrategyOutputSchemaName(Arc<str>);
pub struct StrategyIntentName(Arc<str>);
pub struct PersistentArtifactName(Arc<str>);
```

```rust
pub struct CommitStrategyDescriptorDigest([u8; 32]);
pub struct CanonicalStrategyInputDigest([u8; 32]);
pub struct CanonicalStrategyOutputDigest([u8; 32]);
pub struct MutationProgramDigest([u8; 32]);
pub struct StrategyIntentScopeDigest([u8; 32]);
```

### 5.2 Descriptor Layer

```rust
pub struct CommitStrategyDescriptor {
    pub id: CommitStrategyId,
    pub semantic_name: CommitStrategySemanticName,
    pub family_name: CommitStrategyFamilyName,
    pub version: CommitStrategyVersion,
    pub intent_name: StrategyIntentName,
    pub input_schema_name: StrategyInputSchemaName,
    pub output_schema_name: StrategyOutputSchemaName,
    pub read_contract: StrategyReadContract,
    pub determinism_basis: StrategyDeterminismBasis,
    pub replay_policy: StrategyReplayPolicy,
    pub merge_semantics: StrategyMergeSemantics,
    pub digest: CommitStrategyDescriptorDigest,
}
```

```rust
pub struct StrategyDeterminismBasis {
    pub canonicalization_version: u16,
    pub lowering_version: u16,
    pub query_planning_version: u16,
    pub requires_canonical_seed: bool,
}
```

```rust
pub enum StrategyReplayPolicy {
    ReinvokeAndCompare,
}
```

### 5.3 Registry Layer

```rust
pub struct RegisteredCommitStrategy {
    descriptor: CommitStrategyDescriptor,
    executor: Arc<dyn CommitStrategyExecutor>,
}
```

```rust
pub struct FrozenCommitStrategyRegistry {
    descriptors_by_id: BTreeMap<CommitStrategyId, CommitStrategyDescriptor>,
    ids_by_name: BTreeMap<CommitStrategySemanticName, CommitStrategyId>,
    executors_by_id: BTreeMap<CommitStrategyId, Arc<dyn CommitStrategyExecutor>>,
    registry_digest: [u8; 32],
}
```

```rust
pub enum StrategyRegistryError {
    DuplicateId { id: CommitStrategyId },
    DuplicateSemanticName { name: CommitStrategySemanticName },
    DuplicateFamilyAndVersion {
        family: CommitStrategyFamilyName,
        version: CommitStrategyVersion,
    },
    RegistryFrozen,
    DescriptorDigestMismatch,
}
```

### 5.4 Read Contract Layer

```rust
pub struct StrategyReadContract {
    pub scope_class: StrategyReadScopeClass,
    pub locality_class: StrategyReadLocalityClass,
    pub traversal_basis: StrategyTraversalBasis,
    pub packet_contract: StrategyPacketContract,
    pub cost_class: StrategyReadCostClass,
}
```

```rust
pub enum StrategyReadScopeClass {
    ExplicitTargetsOnly,
    KindBoundedScan,
    PartitionBoundedScan,
    BoundedNeighborhood,
}
```

```rust
pub enum StrategyReadLocalityClass {
    SinglePartition,
    PartitionBounded,
    CrossPartitionBounded,
}
```

```rust
pub enum StrategyTraversalBasis {
    NoTraversal,
    AdjacencyBounded { max_depth: u16 },
}
```

```rust
pub enum StrategyPacketContract {
    ProjectionOnly,
    PlannedPacketOnly,
}
```

```rust
pub enum StrategyReadCostClass {
    ORequestedSurface,
    OPartitionBoundedSurface,
}
```

### 5.5 Request Layer

```rust
pub struct RawStrategyCommitRequest {
    pub strategy_name: CommitStrategySemanticName,
    pub input_bytes: Arc<[u8]>,
    pub caller_provenance: StrategyCallerProvenance,
}
```

```rust
pub struct CanonicalStrategyInputArtifact {
    pub schema_name: StrategyInputSchemaName,
    pub schema_version: SchemaVersionId,
    pub canonical_bytes: Arc<[u8]>,
    pub digest: CanonicalStrategyInputDigest,
    pub artifact_name: PersistentArtifactName,
}
```

```rust
pub struct CanonicalStrategyCommitRequest {
    strategy_id: CommitStrategyId,
    descriptor_digest: CommitStrategyDescriptorDigest,
    canonical_input: CanonicalStrategyInputArtifact,
    caller_provenance: StrategyCallerProvenance,
}
```

```rust
pub struct StrategyCallerProvenance {
    pub request_origin: StrategyRequestOrigin,
    pub actor_identity: Option<ActorId>,
    pub correlation_id: Option<CorrelationId>,
}
```

### 5.6 Execution Layer

```rust
pub struct StrategyObservationContext<'a> {
    snapshot: &'a SnapshotHandle,
    version_id: VersionId,
    schema_registry_digest: [u8; 32],
    invariant_catalog_digest: [u8; 32],
    visibility: StrategyVisibilityReadView<'a>,
    packet_executor: &'a StrategyPacketExecutionFacade,
}
```

```rust
pub trait CommitStrategyExecutor: Send + Sync + 'static {
    fn execute(
        &self,
        request: &CanonicalStrategyCommitRequest,
        observation: &StrategyObservationContext<'_>,
    ) -> Result<StrategyExecutionDraft, StrategyExecutionError>;
}
```

```rust
pub struct StrategyExecutionDraft {
    canonical_output: CanonicalStrategyOutputArtifact,
    mutation_program: StrategyMutationProgram,
    lineage_intents: Arc<[StrategyLineageIntent]>,
    intent_scope: StrategyIntentScope,
    execution_summary: StrategyExecutionSummary,
}
```

```rust
pub struct CanonicalStrategyOutputArtifact {
    pub schema_name: StrategyOutputSchemaName,
    pub canonical_bytes: Arc<[u8]>,
    pub digest: CanonicalStrategyOutputDigest,
    pub artifact_name: PersistentArtifactName,
}
```

### 5.7 Lowering Layer

```rust
pub struct StrategyMutationProgram {
    pub operations: Arc<[StrategyMutationOp]>,
}
```

```rust
pub enum StrategyMutationOp {
    CreateEntity(CreateEntityOp),
    UpdateEntity(UpdateEntityOp),
    DeleteEntity(DeleteEntityOp),
    CreateRelation(CreateRelationOp),
    UpdateRelation(UpdateRelationOp),
    DeleteRelation(DeleteRelationOp),
    ReplaceEntity(ReplaceEntityOp),
    DeclareCorrespondence(DeclareCorrespondenceOp),
}
```

```rust
pub struct StrategyIntentScope {
    pub touched_records: Arc<[RecordRef]>,
    pub touched_partitions: Arc<[PartitionId]>,
    pub touched_aspects: Arc<[AspectKey]>,
    pub digest: StrategyIntentScopeDigest,
}
```

```rust
pub struct StrategyCommitProvenance {
    pub strategy_id: CommitStrategyId,
    pub semantic_name: CommitStrategySemanticName,
    pub family_name: CommitStrategyFamilyName,
    pub version: CommitStrategyVersion,
    pub descriptor_digest: CommitStrategyDescriptorDigest,
    pub input_digest: CanonicalStrategyInputDigest,
    pub output_digest: CanonicalStrategyOutputDigest,
}
```

```rust
pub struct StrategyLoweredCommitPlan {
    provenance: StrategyCommitProvenance,
    canonical_request: CanonicalStrategyCommitRequest,
    canonical_output: CanonicalStrategyOutputArtifact,
    mutation_program_digest: MutationProgramDigest,
    intent_scope: StrategyIntentScope,
    executable_batch: WorkerIntentBatch,
    lineage_plan: LoweredLineagePlan,
    naming_plan: StrategyPersistentNamingPlan,
    replay_descriptor: StrategyReplayDescriptor,
    merge_descriptor: StrategyMergeDescriptor,
}
```

```rust
pub enum LoweredCommitPlan {
    Mutation(MutationOnlyCommitPlan),
    Strategy(StrategyLoweredCommitPlan),
}
```

### 5.8 Validation and Execution Layer

```rust
pub struct ValidatedStrategyCommitPlan {
    lowered: StrategyLoweredCommitPlan,
    invariant_result: InvariantExecutionArtifact,
    validation_summary: CommitValidationSummary,
}
```

```rust
pub struct ExecutedStrategyCommit {
    pub commit_result: CommitResult,
    pub strategy_artifacts: StrategyCommitArtifacts,
}
```

```rust
pub struct StrategyCommitArtifacts {
    pub provenance: StrategyCommitProvenance,
    pub canonical_input: CanonicalStrategyInputArtifact,
    pub canonical_output: CanonicalStrategyOutputArtifact,
    pub replay_descriptor: StrategyReplayDescriptor,
    pub merge_descriptor: StrategyMergeDescriptor,
    pub decision_trace: StrategyDecisionTrace,
}
```

### 5.9 Replay Layer

```rust
pub struct StrategyReplayDescriptor {
    pub strategy_id: CommitStrategyId,
    pub descriptor_digest: CommitStrategyDescriptorDigest,
    pub input_digest: CanonicalStrategyInputDigest,
    pub output_digest: CanonicalStrategyOutputDigest,
    pub mutation_program_digest: MutationProgramDigest,
    pub observed_snapshot_id: SnapshotId,
    pub observed_version_id: VersionId,
    pub schema_registry_digest: [u8; 32],
    pub invariant_catalog_digest: [u8; 32],
    pub determinism_basis: StrategyDeterminismBasis,
}
```

```rust
pub enum StrategyReplayMismatchClass {
    StrategyNotRegistered,
    DescriptorDigestMismatch,
    InputDigestMismatch,
    OutputDigestMismatch,
    MutationProgramDigestMismatch,
    IntentScopeDigestMismatch,
    LineagePlanDrift,
    NamingPlanDrift,
    LoweringSemanticsDrift,
}
```

```rust
pub struct ReplayVerifiedStrategyCommit {
    pub descriptor: StrategyReplayDescriptor,
    pub mismatch: Option<StrategyReplayMismatchClass>,
}
```

### 5.10 Merge Layer

```rust
pub struct StrategyMergeDescriptor {
    pub strategy_id: CommitStrategyId,
    pub family_name: CommitStrategyFamilyName,
    pub semantic_name: CommitStrategySemanticName,
    pub version: CommitStrategyVersion,
    pub intent_scope_digest: StrategyIntentScopeDigest,
    pub merge_semantics: StrategyMergeSemantics,
}
```

```rust
pub struct StrategyMergeSemantics {
    pub conflict_class: StrategyConflictClass,
    pub requires_causal_comparison: bool,
    pub respects_aspect_merge_policies: bool,
}
```

```rust
pub enum StrategyConflictClass {
    IntentReconciliation,
    ReplicaConvergence,
    WorkflowAdvancement,
    BridgeEvaluation,
}
```

## 6. Constructor Ownership And Visibility

The following constructor rules are mandatory:

- `RawStrategyCommitRequest::new(...)` is public
- `CanonicalStrategyCommitRequest` is constructible only in
  `commit_strategies/request`
- `StrategyExecutionDraft` is constructible only by
  `CommitStrategyExecutor` adapters
- `StrategyLoweredCommitPlan` is constructible only in
  `commit_strategies/lowering`
- `ValidatedStrategyCommitPlan` is constructible only in the commit validation
  module
- `ExecutedStrategyCommit` is constructible only in the authority execution
  module
- `ReplayVerifiedStrategyCommit` is constructible only in
  `commit_strategies/replay`

Compile-fail tests must prove:

- external code cannot construct `CanonicalStrategyCommitRequest`
- external code cannot construct `StrategyLoweredCommitPlan`
- external code cannot pass `StrategyExecutionDraft` directly to authority
  execution
- external code cannot bypass strategy registration freeze

## 7. Read Contract And Observation Model

The observation model must prevent hidden whole-graph planning.

### Observation rules

- strategies observe committed truth only
- strategies cannot access mutation authority
- strategies cannot request raw whole-snapshot iteration unless their declared
  read contract explicitly admits the needed bounded packet type
- strategies must use packetized read or projection read surfaces only
- the observation context does not expose branch switching, mutable caches, or
  index authority internals

### Required enforcement

The registry must reject a strategy descriptor whose read contract cannot be
honestly served by the runtime's packet/projection surfaces.

The planner must reject execution if the requested observation would exceed the
registered contract.

### Why this matters

Without this, a seemingly narrow intent strategy can silently become a
whole-state scan under CAD or chip-scale workloads.

## 8. Determinism Rules

Determinism is non-negotiable.

Forbidden nondeterminism sources:

- unordered map iteration
- ambient wall clock reads
- ambient randomness
- thread completion order
- allocation-address-derived ordering
- mutable runtime observation during strategy execution

If a strategy needs time or randomness, they must be supplied through
canonicalized input and represented in the input digest.

The runtime must compare canonical digests over:

- input artifact
- output artifact
- lowered mutation program
- intent scope
- lineage plan
- naming plan

## 9. Lineage, Provenance, And Persistent Naming

These are product semantics, not optional metadata.

### Lineage rules

- strategy-produced replace/split/correspondence semantics must be explicit in
  lowered artifacts
- lineage-affecting transitions may not be reconstructed from patch deltas
  later
- replay must compare lineage-bearing strategy artifacts canonically

### Provenance rules

Every executed strategy commit must persist:

- strategy semantic name
- strategy family name
- strategy version
- descriptor digest
- input schema name
- input digest
- output digest
- schema registry digest
- invariant catalog digest
- caller provenance

### Persistent naming rules

- semantic names are canonical durable identities
- display names are not durable identities
- implementation type names are not durable identities
- renames require explicit version transition and descriptor digest change

## 10. Strategy-Aware Merge Semantics

Strategy-bearing merge classification must consume:

- `StrategyMergeDescriptor`
- commit causal metadata
- per-aspect merge policy descriptors
- schema reconciliation outcome where relevant
- lineage plan summaries

At minimum, merge must distinguish:

- same-record ordinary conflict
- same-record same-strategy same-family conflict
- same-record different-strategy family conflict
- same-intent-scope divergent-output conflict
- same-output divergent-provenance conflict

This metadata must be canonical and persisted with merge-bearing artifacts.

## 11. Phase Plan

Each phase below is intended to be buildable in order.

### Phase 0: Spec Freeze

Goal:

- freeze the type model, constructor authority rules, read contract model, and
  determinism basis before code lands

Deliver:

- this document
- constructor ownership map
- compile-fail boundary test list

Completion condition:

- no milestone code lands before the proof chain and lowering boundary are
  accepted

### Phase 1: Registry, Naming, and Descriptor Foundation

Goal:

- build frozen strategy registration with persistent naming and descriptor
  digests

Deliver:

- `CommitStrategyId`
- naming/versioning types
- `CommitStrategyDescriptor`
- `FrozenCommitStrategyRegistry`

Required counters:

- `strategy_registry_count`
- `strategy_registry_freeze_count`

Completion condition:

- strategies can register only during runtime construction
- duplicate identity and naming failures are typed
- registry digest is stable across restart

### Phase 2: Canonical Request And Input Artifact Layer

Goal:

- convert raw intent into a canonical strategy request artifact

Deliver:

- `RawStrategyCommitRequest`
- `CanonicalStrategyInputArtifact`
- `CanonicalStrategyCommitRequest`

Required counters:

- `strategy_request_count`
- `strategy_request_canonicalization_count`

Completion condition:

- identical semantic input yields identical canonical digest
- canonical request is durable without host help

### Phase 3: Read Contract Admission And Contained Execution

Goal:

- execute registered strategies against phase-typed committed observation only

Deliver:

- `StrategyReadContract`
- `StrategyObservationContext`
- `CommitStrategyExecutor`
- panic boundary and typed failures

Required counters:

- `strategy_execution_count`
- `strategy_execution_rejection_count`
- `strategy_execution_panic_count`
- `strategy_packet_count`
- `strategy_packet_target_count`

Completion condition:

- a strategy cannot exceed its declared read contract
- a strategy cannot access mutation authority
- panic becomes typed failure, not runtime crash

### Phase 4: Lowering And Structural Summary

Goal:

- lower execution drafts into runtime-owned strategy commit plans

Deliver:

- `StrategyMutationProgram`
- `StrategyIntentScope`
- `StrategyCommitProvenance`
- `StrategyLoweredCommitPlan`
- `LoweredCommitPlan::Strategy`

Required counters:

- `strategy_lowering_count`
- `strategy_mutation_op_count`
- `strategy_touched_record_count`
- `strategy_lineage_intent_count`

Completion condition:

- lowering derives touched scope exactly once
- strategy semantics remain alive in commit plan form
- executable batch is available without erasing strategy provenance

### Phase 5: Invariant Integration And Validation

Goal:

- run strategy plans through ordinary invariant and validation boundaries

Deliver:

- `ValidatedStrategyCommitPlan`
- validation summaries carrying strategy provenance

Required counters:

- `strategy_validation_count`
- `strategy_validation_rejection_count`
- `strategy_publication_block_count`

Completion condition:

- illegal strategy output is rejected by invariant authority
- publication-blocked semantics remain explicit and canonical

### Phase 6: Executed Strategy Commit Artifacts

Goal:

- persist canonical strategy-bearing commit artifacts

Deliver:

- `ExecutedStrategyCommit`
- `StrategyCommitArtifacts`
- `StrategyDecisionTrace`
- strategy artifacts integrated into commit result and envelope-derived surfaces

Required counters:

- `strategy_commit_count`
- `strategy_artifact_emit_count`

Completion condition:

- durable strategy-bearing commits contain enough information for replay
  re-invocation
- provenance and naming are queryable without re-execution

### Phase 7: Replay Re-Invocation Verification

Goal:

- re-invoke strategies during replay and compare exact lowered output

Deliver:

- `StrategyReplayDescriptor`
- `StrategyReplayMismatchClass`
- `ReplayVerifiedStrategyCommit`

Required counters:

- `strategy_replay_count`
- `strategy_replay_mismatch_count`

Completion condition:

- nondeterministic strategies fail parity explicitly
- replay distinguishes descriptor drift, input drift, lowering drift, and
  lineage/naming drift

### Phase 8: Strategy-Aware Merge Classification

Goal:

- integrate strategy semantics into merge truth

Deliver:

- `StrategyMergeDescriptor`
- merge conflict classification over strategy-bearing commits

Required counters:

- `strategy_merge_conflict_count`
- `strategy_merge_same_family_conflict_count`
- `strategy_merge_cross_family_conflict_count`

Completion condition:

- merge can classify strategy-bearing overlaps without reducing them to raw
  touched-record collisions

### Phase 9: Reference Strategies

Goal:

- ship two real strategies proving the architecture

Deliver:

- `IntentReconciliationStrategy`
- `ReplicaConvergenceStrategy`

Completion condition:

- both strategies execute through the same canonical lifecycle
- both replay deterministically
- both emit lineage/provenance/naming artifacts

### Phase 10: Certification Closure

Goal:

- close the milestone with machine-checkable truth-grade artifacts

Deliver:

- named certification suite
- required artifact digests

Completion condition:

- all hostile scenarios pass with canonical artifact outputs

## 12. Reference Strategies

### 12.1 Intent Reconciliation Strategy

Purpose:

- reconcile a desired-state declaration against current truth and produce the
  minimal legal mutation program

Required input type:

```rust
pub struct IntentReconciliationInput {
    pub target_scope: IntentTargetScope,
    pub desired_state: CanonicalDesiredStateArtifact,
    pub reconciliation_policy: IntentReconciliationPolicy,
}
```

Required properties:

- idempotent
- deterministic canonical operation ordering
- respects aspect merge policies
- emits explicit lineage intents where replacement is chosen

### 12.2 Replica Convergence Strategy

Purpose:

- reconcile desired service/deployment replica state to current truth in a
  Kubernetes-style control-loop shape

Required input type:

```rust
pub struct ReplicaConvergenceInput {
    pub namespace: NamespaceId,
    pub deployment_name: PersistentArtifactName,
    pub desired_replicas: u32,
    pub pod_template_digest: [u8; 32],
    pub rollout_policy: ReplicaRolloutPolicy,
}
```

Required properties:

- deterministic pod-selection and replacement ordering
- bounded read scope
- explicit persistent naming for deployment, replica set, and pod surfaces
- explicit lineage on pod replacement
- exact no-op behavior when desired state is already satisfied

## 13. Adversarial Constraint

The adversarial condition for Milestone 8.5 is:

> A caller-supplied strategy running against a large, branch-divergent,
> replayable truth runtime must not be able to corrupt authoritative state,
> bypass invariants, hide nondeterminism, erase lineage/provenance/naming
> semantics, or make replay depend on host-specific ambient context.

## 14. Adversarial Certification Scenario

Milestone 8.5 must add a named certification suite:

- `Extensible commit strategy certification test`

Its primary hostile scenario must be Kubernetes-style intent convergence.

### Scenario: Kubernetes-Style Intent Commit Under Branch Divergence

Run a deterministic workload containing:

- registration of `IntentReconciliationStrategy`
- registration of `ReplicaConvergenceStrategy`
- creation of deployment, replica set, and pod truth records
- intent commits that scale replicas up and down
- branch-local alternate rollout plans
- branch-local pod replacement ordering divergence
- snapshot-pinned readers inspecting deployment truth mid-rollout
- savepoint rollback inside failed rollout attempts
- panic injection in one hostile strategy execution
- invariant rejection of an illegal rollout mutation
- replay and durable recovery over strategy-bearing histories

Must verify:

- failing strategy leaves zero authoritative residue
- panicking strategy becomes typed failure
- replay re-invokes strategy and reproduces identical mutation program
- persistent naming of deployment/replica set/pod surfaces remains stable
- lineage captures pod replacement correctly
- branch-local rollout history stays isolated
- merge classifies divergent rollout strategies with strategy-aware conflict
  metadata
- snapshot-pinned readers see stable truth during hot rollout activity

Required outputs:

- `strategy_registry_digest`
- `strategy_input_digest`
- `strategy_output_digest`
- `strategy_mutation_program_digest`
- `strategy_lineage_digest`
- `strategy_provenance_digest`
- `strategy_replay_digest`
- `strategy_merge_conflict_digest`
- `strategy_failure_containment_digest`
- `strategy_snapshot_stability_digest`

Pass condition:

Strategy-bearing histories remain deterministic, replay-verifiable,
lineage-aware, provenance-complete, and fail-contained under hostile rollout
pressure.

## 15. Required New Complexity Contracts

At minimum add:

- `commit_strategies.request.canonicalization`
- `commit_strategies.execution.read_contract_admission`
- `commit_strategies.execution.packet_observation`
- `commit_strategies.lowering.strategy_mutation_program_lowering`
- `commit_strategies.replay.reinvoke_and_compare`
- `commit_strategies.merge.strategy_conflict_classification`
- `commit_strategies.replica_convergence.partition_bounded_rollout_reconciliation`

Each must declare:

- exact complexity statement
- boundedness basis
- required counters
- verified vs debt status
- proof test name

## 16. Highest-Risk Failure Modes

- erasing strategy semantics too early by lowering straight into mutation-only
  commit plans
- allowing unrestricted read access and accidentally normalizing whole-graph
  scans into strategy execution
- treating persistent naming as display metadata
- using implementation type names as durable strategy identity
- replay comparing only stored outputs without re-invocation
- merge treating strategy-bearing conflicts as record overlaps only
- reconstructing lineage from patches instead of explicit lineage intents
- allowing late-phase proof types to be externally constructible
- adding a second broad strategy before the bounded Kubernetes-style one proves
  the architecture

## 17. Milestone 8.5 Exit Criteria

Milestone 8.5 is complete only when all are true:

- the runtime has a frozen strategy registry with deterministic descriptor
  digests
- canonical strategy requests are durable and replay-safe
- strategy execution is read-contract bounded and fail-contained
- strategy lowering preserves lineage, provenance, and persistent naming
- strategy commit plans remain first-class through validation and replay
- replay re-invokes strategies and compares exact lowered output
- merge classification consumes strategy semantics structurally
- both reference strategies ship and pass deterministic replay
- the extensible strategy certification suite passes with machine-checkable
  outputs

## 18. Summary

The core correction in this plan is simple:

Milestone 8.5 is not an extension hook milestone.

It is a truth-authority generalization milestone.

If we do it correctly:

- commit strategies become first-class, deterministic, replayable effect
  planners
- the runtime keeps one authority path
- lineage, provenance, and persistent naming remain product semantics
- merge and replay remain honest
- Kubernetes-style intent convergence becomes a certifiable strategy workload,
  not a host-side sidecar accident

If we do it incorrectly:

- we create a second mutation authority path hidden behind "extensibility"
- replay becomes trust-based
- merge loses semantic honesty
- performance degrades under broad strategy reads
- long-lived truth artifacts stop being durable contracts

This milestone is where extensibility must earn truth-grade status.

## 19. Implementation Closeout Addendum

This section records the delivered implementation status for Milestone 8.5.

### 19.1 Closeout Decision

Milestone 8.5 is considered complete.

The implementation now satisfies the milestone's load-bearing requirements:

- shared lowered commit-plan handling exists and strategy commits do not bypass
  the ordinary authority path
- canonical strategy request, execution, lowering, validation, replay, and
  merge artifacts exist as proof-bearing runtime surfaces
- replay re-invokes strategy logic and classifies hostile failure modes
- strategy-aware merge classification and manual boundaries are runtime-owned
- real reference strategies ship and participate in end-to-end replay and merge
  certification
- named certification suites exist and pass with machine-checkable outputs

### 19.2 Delivered Runtime Shape

The shipped runtime behavior now includes:

- deterministic frozen strategy registration and descriptor digests
- canonical request/input/output artifacts
- contained execution behind declared read contracts
- shared lowered commit-plan execution through the single serialized authority
  path
- validation artifacts with stale-basis rejection
- durable strategy replay and merge descriptors on canonical commit envelopes
- replay re-execution with explicit mismatch taxonomy
- strategy-aware merge classification for staged hostile histories
- Kubernetes-style staged controller certification with non-trivial outcomes
- accumulated-history recovery parity for earlier staged merge planning digests

### 19.3 Reference Strategies Shipped

The proving strategies are now real runtime-owned implementations, not
synthetic drafts:

- `IntentReconciliationStrategy`
- `ReplicaConvergenceStrategy`
- `EntityReplacementReconciliationStrategy`

These strategies now exercise:

- declared-field partial updates
- aspect-aware reconciliation
- persistent-name and lineage-sensitive replacement behavior
- deterministic replay through the ordinary strategy pipeline

### 19.4 Certification Surfaces Landed

The implementation ships named certification coverage for:

- general strategy-bearing replay, recovery, merge, and durable artifact truth
- hostile executor-unavailable and executor-failure replay outcomes
- Kubernetes-style staged intent/convergence history with:
  - conflict introduction
  - intent narrowing
  - re-broadening
  - controller re-convergence
  - exact-shared-truth end state
  - final accumulated-history recovery parity for earlier staged planning
    digests

### 19.5 Intentional Residual Constraint

One narrow equivalence rule was required to make accumulated-history recovery
digest parity honest and stable:

- in merge planning digest basis only, `BaseHistoricalWindow` and
  `BaseResolvedViewLookup` are treated as equivalent when base, source, and
  target all prove the record visible

This normalization does **not** erase raw planner evidence. The original
visibility evidence rows remain present in the planning artifact. Only the
digest-basis copy is normalized for parity where the two forms prove the same
semantic fact.

### 19.6 Closeout Statement

Milestone 8.5 is closed with the following interpretation:

- extensible commit strategies now have truth-grade status in
  `worth-relational`
- the runtime preserves one authority path
- replay and recovery are no longer trust-based for strategy-bearing commits
- merge semantics are strategy-aware and durable
- staged hostile controller history is certified, not merely demonstrated

Any future work in this area should be treated as post-8.5 refinement, domain
expansion, or new certification breadth, not unfinished milestone plumbing.
