# Milestone 6.5 Engineering Spec: Invariant Completion and Custom Invariant Support

## Summary

Milestone 6.5 completes the invariant subsystem as a truth-grade authority
surface.

The runtime must move from:

- "some structural rules exist and are checked through the validation engine"

to:

- "the full domain-agnostic invariant suite is represented as explicit,
  phase-typed authority contracts, custom structural invariants participate in
  the same planning and execution pipeline without becoming a type-erasure
  escape hatch, publication-boundary legality is explicit, and every important
  semantic and cost claim is mechanically enforced or machine-certifiable."

This milestone is not "add more validators."

It is:

- type-system completion for invariant authority
- native invariant completion
- custom structural invariant extensibility
- publication-boundary legality completion
- complexity-contract completion
- certification-grade artifact and diagnostics completion

The governing rules for Milestone 6.5 are:

- invariants are authority-path semantics, not test-only aftercare
- planning and execution are distinct phases with distinct proof-bearing types
- custom invariants are first-class participants in the invariant pipeline, not
  side hooks
- custom invariants may observe only structural truth surfaces, never signal or
  other derived state
- the wrong phase, wrong scope, wrong authority boundary, and wrong executable
  pairing must be made unrepresentable or uncompilable wherever possible
- every invariant cost claim must be tied to named counters and proof tests
- publication blocking is a first-class semantic state, not an accidental
  diagnostics side effect

This milestone keeps the original phased plan intact and strengthens it with
the architectural corrections required by critique:

- no `Any`-based semantic escape hatches at the framework boundary
- honest acyclicity complexity language
- traversal budgets enforced per rule-execution session, not per call
- explicit fork-join phase topology instead of pretending the subsystem is a
  purely linear chain
- semantic identity for custom rules separated from operational metadata
- explicit definitions for load-bearing planning and locality types
- explicit lifecycle semantics for committed-but-unpublished states

## Architecture Goal

At closeout, invariant authority must have the same architectural honesty
standard as lineage:

- one explicit registration surface
- one explicit planning/join boundary
- one explicit execution batch
- one canonical invariant artifact per execution boundary
- all summaries, diagnostics, replay digests, and publication decisions derived
  from that artifact

No later consumer may reconstruct invariant meaning by rescanning raw runtime
state.

## Phase Topology

Milestone 6.5 is not a single linear proof chain.

The real topology is a fork-join authority graph:

1. registration branch:
   - what invariant rules exist
   - what execution points they support
   - what groups, costs, and failure effects they carry

2. prepared-input branch:
   - what structural scope has been prepared for this execution request
   - what touched, partition-local, reachable, or publication-visible truth is
     available to consume

3. join boundary:
   - selection, legality filtering, scope ownership, and packet lowering

4. execution branch:
   - monomorphic packet execution only

5. artifact branch:
   - canonical result artifact
   - commit/publication/replay/diagnostics surfaces derived from it

The subsystem must encode this topology explicitly. It must not hide the merge
point inside a convenience function.

### Required authority graph

```text
InvariantRegistrationSet ----\
                             -> InvariantSelection -> LoweredInvariantPlan
PreparedInvariantInputs -----/                            |
                                                          v
                                           ExecutionReadyInvariantBatch
                                                          |
                                                          v
                                              InvariantExecutionArtifact
```

Ownership rule:

- `InvariantSelection` is the sole owner of the fork-join merge
- lowering consumes the selection object and produces the plan
- execution consumes only the lowered plan

No packet executor may independently look up registrations or reconstruct
prepared scope.

## Required Production Structure

Milestone 6.5 must preserve structural decomposition and may not accumulate
into broad existing files.

### Production structure

- `src/validation/data/`
- `rule_id.rs`
- `descriptor.rs`
- `native_rule.rs`
- `custom_rule.rs`
- `registration.rs`
- `execution.rs`
- `groups.rs`
- `results.rs`
- `metrics.rs`
- `mod.rs`

- `src/validation/planning/`
- `request.rs`
- `prepared_inputs.rs`
- `selection.rs`
- `scope.rs`
- `lowered_plan.rs`
- `locality.rs`
- `mod.rs`

- `src/validation/execution/`
- `dispatch.rs`
- `context.rs`
- `native/sidecar.rs`
- `native/relation_integrity.rs`
- `native/acyclicity.rs`
- `native/payload_schema.rs`
- `native/partition_isolation.rs`
- `native/connectivity_minimum.rs`
- `custom/adapter.rs`
- `custom/panic_boundary.rs`
- `custom/runtime.rs`
- `mod.rs`

- `src/validation/registry/`
- `custom_rules.rs`
- `descriptor_store.rs`
- `mod.rs`

- `src/validation/facade.rs`
- facade only

### Test structure

- `src/tests/validation/`
- `registration.rs`
- `selection.rs`
- `planning.rs`
- `native_acyclicity.rs`
- `native_payload_schema.rs`
- `native_partition_isolation.rs`
- `native_connectivity_minimum.rs`
- `custom_registration.rs`
- `custom_execution.rs`
- `custom_panic_isolation.rs`
- `publication_boundary.rs`
- `certification.rs`
- `mod.rs`

- `tests/ui/invariants/`
- compile-fail coverage for constructor sealing, invalid registration, and
  inaccessible internal authority surfaces

### Structural rules

- `mod.rs` files are wiring-only
- no new milestone behavior may land in existing broad matcher files
- no `validation_manager`, `invariant_helpers`, or mixed "contracts" files
- each file must map to one phase responsibility or one invariant family

## Type-System Foundation

Milestone 6.5 must replace the overly collapsed "enum + metadata methods"
shape with a three-layer invariant identity model.

### Public identity layer

```rust
pub enum InvariantRuleId {
    Native(NativeInvariantRuleId),
    Custom(CustomInvariantRuleId),
}
```

```rust
pub enum NativeInvariantRuleId {
    LiveRecordRequiresSidecarEntity,
    LiveRecordRequiresSidecarRelation,
    MaxMergedIntents,
    RelationIntegrityScopeBudget,
    MaxSnapshotEntities,
    UniqueEntityPayloadField,
    EndpointKindContract,
    CardinalityMaximumContract,
    CardinalityMinimumContract,
    UniquenessContract,
    SymmetryContract,
    EndpointDeletionIntegrityContract,
    AcyclicityContract,
    PayloadSchemaContract,
    PartitionIsolationContract,
    ConnectivityMinimumContract,
}
```

```rust
pub struct CustomInvariantRuleId(Arc<str>);
```

`CustomInvariantRuleId` is semantic identity, not operational identity.

### Custom semantic versioning

Custom rule identity stability must be explicit across binary versions.

Required types:

```rust
pub struct CustomInvariantSemanticVersion {
    pub major: u16,
    pub minor: u16,
}
```

```rust
pub struct CustomInvariantSemanticIdentity {
    pub rule_id: CustomInvariantRuleId,
    pub semantic_version: CustomInvariantSemanticVersion,
}
```

```rust
pub struct CustomInvariantOperationalMetadata {
    pub execution_point: InvariantExecutionPoint,
    pub groups: InvariantGroupSet,
    pub cost_class: InvariantCostClass,
    pub failure_effect: InvariantFailureEffect,
}
```

Rule identity rules:

- semantic identity is stable across binary versions unless the rule author
  explicitly changes semantic version
- operational metadata may change without changing semantic identity
- canonical persisted artifacts must carry semantic identity separately from
  operational metadata digest
- replay of historical artifacts must not require the current runtime to have a
  live executable for the historical custom rule id

### Public descriptor layer

```rust
pub struct InvariantRuleDescriptor {
    pub id: InvariantRuleId,
    pub execution_points: SupportedExecutionPoints,
    pub groups: InvariantGroupSet,
    pub cost_class: InvariantCostClass,
    pub failure_effect: InvariantFailureEffect,
    pub semantics: InvariantSemanticsClass,
}
```

```rust
pub enum InvariantSemanticsClass {
    NativeAlwaysOn,
    NativeSchemaLowered,
    CustomStructural,
}
```

### Internal executable layer

```rust
pub(crate) enum ExecutableInvariantRule {
    Native(NativeInvariantRule),
    Custom(RegisteredCustomInvariantRule),
}
```

`ExecutableInvariantRule` is never public.

## Phase 1: Structural Split and Canonical Domain Vocabulary

### Goal

Reshape validation into responsibility-aligned subdomains before behavior
expansion so Milestone 6.5 does not accumulate into existing validation files.

### Constructor and visibility rules

- `CustomInvariantRuleId::new` is public
- `InvariantRuleDescriptor` is not freely constructible by external code;
  callers obtain it only through:
  - schema lowering
  - custom registration constructors
  - facade-owned builder helpers
- `ExecutableInvariantRule`, `PreparedInvariantInputs`,
  `InvariantSelection`, `LoweredInvariantPlan`, and
  `ExecutionReadyInvariantBatch` constructors are `pub(crate)` and owned only
  by their phase modules
- no blanket `From<Vec<_>>`, `Into`, or collection-based convenience
  conversions into proof-bearing wrappers are allowed
- no rule evaluator may accept a raw descriptor, raw id, or raw registry lookup
  instead of the exact lowered packet type for its phase

### Compile-time verification

Compile-fail tests must prove:

- external code cannot construct `ExecutableInvariantRule`
- external code cannot construct `LoweredInvariantPlan`
- external code cannot call execution with descriptors instead of lowered plans
- unsupported execution-point combinations cannot be registered through public
  helpers
- custom invariants cannot be registered without explicit group, cost, and
  failure metadata
- internal late-phase constructors remain inaccessible outside their owning
  module

## Phase 2: Typed Authority Phase Graph

### Goal

Encode invariant authority as explicit fork-join phases so later phases cannot
accept weaker inputs and re-decide earlier questions.

### Required phase wrappers

#### 1. `InvariantRegistrationSet`

Authority basis only.

```rust
pub(crate) struct InvariantRegistrationSet {
    registrations: Arc<[InvariantRegistration]>,
}
```

Contains:

- schema-lowered native registrations
- runtime-registered custom registrations

#### 2. `PreparedInvariantInputs<'runtime>`

Prepared structural scopes only.

```rust
pub(crate) struct PreparedInvariantInputs<'runtime> {
    request: InvariantExecutionRequest<'runtime>,
    relation_integrity_scopes: Option<PreparedRelationIntegrityScopes>,
    acyclicity_scopes: Option<PreparedAcyclicityScopes>,
    payload_validation_targets: Option<PreparedPayloadValidationTargets>,
    partition_isolation_scopes: Option<PreparedPartitionIsolationScopes>,
    connectivity_scopes: Option<PreparedConnectivityScopes>,
    custom_scope_packets: Option<PreparedCustomInvariantScopes>,
}
```

#### 3. `InvariantSelection<'runtime>`

Explicit fork-join merge owner.

```rust
pub(crate) struct InvariantSelection<'runtime> {
    registrations: InvariantRegistrationSet,
    prepared_inputs: PreparedInvariantInputs<'runtime>,
}
```

Responsibilities:

- execution-point filtering
- group-mask filtering
- plan-contract filtering
- scope-ownership alignment
- cost-policy filtering
- packet lowering preparation

#### 4. `SelectedInvariantRegistrations`

```rust
pub(crate) struct SelectedInvariantRegistrations {
    registrations: Arc<[InvariantRegistration]>,
}
```

Contains only registrations authorized by:

- execution point
- plan contract
- group mask
- cost policy

#### 5. `LoweredInvariantPlan<'runtime>`

```rust
pub(crate) struct LoweredInvariantPlan<'runtime> {
    context: Arc<InvariantPlanningContext>,
    packets: Arc<[LoweredInvariantPacket<'runtime>]>,
    strategy: InvariantExecutionStrategy,
    proof_summary: InvariantProofBoundarySummary,
}
```

```rust
pub(crate) struct LoweredInvariantPacket<'runtime> {
    packet_id: InvariantPacketId,
    descriptor: InvariantRuleDescriptor,
    executable: &'runtime ExecutableInvariantRule,
    execution_point: InvariantExecutionPoint,
    locality: InvariantLocalityProof,
    inputs: LoweredInvariantInputs<'runtime>,
}
```

#### 6. `ExecutionReadyInvariantBatch<'runtime>`

```rust
pub(crate) struct ExecutionReadyInvariantBatch<'runtime> {
    plan: LoweredInvariantPlan<'runtime>,
}
```

Confirms:

- all packets are legal to execute under the chosen strategy
- all custom packet scope/executable pairing is already sealed

#### 7. `InvariantExecutionArtifact`

```rust
pub struct InvariantExecutionArtifact {
    pub execution_point: InvariantExecutionPoint,
    pub summary: InvariantExecutionSummary,
    pub results: Arc<[InvariantCheckResult]>,
    pub decision_log: Arc<[InvariantDecisionRecord]>,
    pub metrics: InvariantBoundaryMetrics,
    pub digest_basis: InvariantExecutionDigestBasis,
}
```

### Anti-bypass rules

- execution dispatch accepts only `ExecutionReadyInvariantBatch`
- packet evaluators accept only `LoweredInvariantPacket`
- commit/publication code consumes only `InvariantExecutionArtifact`
- no evaluator may look up a custom rule by string id at execution time
- no evaluator may reconstruct prepared scope by rescanning raw runtime state
- all semantic rejection based on metadata, legality, or phase support occurs
  before `LoweredInvariantPlan` exists

## Phase 3: Branchless Structural Access and Scope Preparation

### Goal

Define the structural scope-preparation model with exact access boundaries.

### Required load-bearing planning types

#### `InvariantGroupSet`

This must be defined, not treated as an ambient helper.

Required representation:

```rust
pub struct InvariantGroupSet(u64);
```

It is a closed bitset over a fixed enum of invariant groups.

Required behavior:

- descriptor-declared group membership is exact
- request-consumed groups intersect with may-break groups from the plan
- registration selection is based on exact bitset intersection
- no dynamic string group names

#### `InvariantLocalityProof`

This is a proof-bearing structural boundary summary carried by each lowered
packet.

Required type:

```rust
pub(crate) struct InvariantLocalityProof {
    pub observation_scope: InvariantObservationKind,
    pub partition_scope: InvariantPartitionScope,
    pub kind_scope: InvariantKindScope,
    pub traversal_basis: InvariantTraversalBasis,
    pub scope_class: InvariantScopeClass,
}
```

Where:

- `InvariantPartitionScope` is one of:
  - `TouchedPartitions`
  - `SinglePartition`
  - `AllObserved`
- `InvariantKindScope` is one of:
  - `SingleRelationKind`
  - `SingleRecordKind`
  - `KnownKindSet`
  - `CrossKind`
- `InvariantTraversalBasis` is one of:
  - `NoTraversal`
  - `TouchedOnly`
  - `AdjacencyReachable`
  - `PublicationVisibility`
  - `GlobalObserved`
- `InvariantScopeClass` is one of:
  - `TouchedScope`
  - `PartitionScope`
  - `ReachabilityScope`
  - `PublicationScope`
  - `GlobalScope`

This type proves what structural universe a packet is allowed to inspect.

#### `CustomInvariantScopePlanner<'runtime>`

This must be narrower than execution context, not broader.

Required type:

```rust
pub struct CustomInvariantScopePlanner<'runtime> {
    observation_kind: InvariantObservationKind,
    version_id: VersionId,
    touched: &'runtime TouchedStructuralSet,
    payloads: &'runtime StructuralPayloadView,
    relations: &'runtime StructuralRelationView,
    traversal: &'runtime SessionBoundTraversal<'runtime>,
    counts: &'runtime StructuralCountView,
}
```

Planner rules:

- it may derive structural scope
- it may not mutate runtime state
- it may not access signal or derived state
- it may not allocate unbounded scope packets without paying a tracked scope
  budget
- it may not widen scope silently; any widening must be reflected in packet
  locality proof and counters

### Session-bound traversal budget

Per-call traversal budgets are insufficient.

Milestone 6.5 must enforce traversal budgets per rule-execution session.

Required types:

```rust
pub struct TraversalBudgetSession {
    remaining_frontier: usize,
    remaining_steps: usize,
}
```

```rust
pub enum TraversalBudgetExceeded {
    FrontierExceeded { attempted: usize, remaining: usize },
    StepExceeded { attempted: usize, remaining: usize },
}
```

Traversal APIs must take a mutable session budget:

```rust
fn walk_outgoing_from(
    &self,
    seeds: &[EntityId],
    budget: &mut TraversalBudgetSession,
) -> Result<TraversalResult, TraversalBudgetExceeded>;
```

Rules:

- a custom rule may not refresh its own traversal budget per call
- total traversal cost is bounded across one prepare/evaluate session
- budget exhaustion is a typed failure, not a silent counter overrun

## Phase 4: Native Invariant Completion

### Goal

Finish the missing native invariant families with explicit lowered types and
execution-boundary semantics.

### Required new lowered types

```rust
pub struct LoweredAcyclicityContract {
    pub contract_id: Arc<str>,
    pub relation_kind_id: KindId,
    pub traversal_direction: DirectedTraversalKind,
    pub allowed_cycle_class: AllowedCycleClass,
    pub plan_revision: RelationIntegrityPlanRevision,
}
```

```rust
pub struct LoweredPayloadSchemaContract {
    pub contract_id: Arc<str>,
    pub record_kind: PayloadContractRecordKind,
    pub kind_id: KindId,
    pub schema_digest: PayloadSchemaDigest,
    pub required_fields: Arc<[PayloadFieldRequirement]>,
    pub field_constraints: Arc<[PayloadFieldConstraint]>,
}
```

```rust
pub struct LoweredPartitionIsolationContract {
    pub contract_id: Arc<str>,
    pub relation_kind_id: KindId,
    pub isolation_mode: PartitionIsolationMode,
}
```

```rust
pub struct LoweredConnectivityMinimumContract {
    pub contract_id: Arc<str>,
    pub source_kind_ids: Arc<[KindId]>,
    pub relation_kind_id: KindId,
    pub target_kind_ids: Arc<[KindId]>,
    pub minimum_reachable_targets: u32,
    pub enforcement_boundary: ConnectivityMinimumEnforcement,
}
```

### Required native executable variants

```rust
pub enum NativeInvariantRule {
    ...
    Acyclicity(LoweredAcyclicityContract),
    PayloadSchema(LoweredPayloadSchemaContract),
    PartitionIsolation(LoweredPartitionIsolationContract),
    ConnectivityMinimum(LoweredConnectivityMinimumContract),
}
```

### Acyclicity honesty requirements

The critique is correct that general cycle detection cannot honestly promise a
strictly local worst-case bound without a persistent incremental data
structure.

Milestone 6.5 therefore makes the following explicit choice:

- acyclicity is enforced via per-commit reachability search over the candidate
  relation kind graph
- the check starts from the newly introduced target and tests reachability back
  to source
- worst-case complexity is:
  - `O(reachable_vertices + reachable_edges)` within the candidate relation
    kind graph visible to the request
- this may equal the full candidate relation graph in the worst case

Required honesty rules:

- the subsystem must not claim "no full-graph DFS" as a hard law
- it must claim:
  - no hidden scans outside the candidate relation kind scope
  - no broadening across unrelated relation kinds
  - exact counters for visited vertices, visited edges, and frontier breadth
  - explicit contract status:
    - `Verified` if current topology guarantees the declared candidate-kind
      scope
    - `Debt` if current storage access still widens beyond the intended kind
      local boundary

If a future milestone introduces maintained topological-order or dynamic cycle
detection structures, that becomes a new explicit design, not a hidden
reinterpretation of this one.

### Native scope packet types

```rust
pub(crate) struct PreparedAcyclicityScopes {
    by_relation_kind: Arc<BTreeMap<KindId, PreparedAcyclicityScope>>,
}
```

```rust
pub(crate) struct PreparedPayloadValidationTargets {
    entity_targets: Arc<[PreparedPayloadTarget<EntityId>]>,
    relation_targets: Arc<[PreparedPayloadTarget<RelationId>]>,
}
```

```rust
pub(crate) struct PreparedPartitionIsolationScopes {
    relation_edges: Arc<[PreparedPartitionEdge]>,
}
```

```rust
pub(crate) struct PreparedConnectivityScopes {
    candidate_sources: Arc<[EntityId]>,
    reachable_frontiers: Arc<[PreparedConnectivityFrontier]>,
}
```

### Native execution-point rules

- acyclicity:
  - execution point: `CommitBoundary`
  - failure effect: `BlockCommit`
  - cost class: `Touched`

- payload schema validation:
  - execution point: `CommitBoundary`
  - failure effect: `BlockCommit`
  - cost class: `Touched`

- partition isolation:
  - execution point: `CommitBoundary`
  - failure effect: `BlockCommit`
  - cost class: `Touched`

- connectivity minimum:
  - execution point: `SnapshotPublication`
  - failure effect: `BlockPublication`
  - cost class: `Global` initially unless narrowed further by scope proof

### Publication-boundary legality semantics

Connectivity minimum and publication-boundary minimum-cardinality rules create
an explicit semantic state:

- committed authority
- publication eligibility
- published observability

These states must remain distinct.

Required behavior:

- a commit may succeed authoritatively and still fail publication eligibility
- the resulting `CommitResult` must include the publication-boundary invariant
  artifact directly
- publication status must explicitly encode "blocked by invariant"
- no blocked commit becomes visible to CDC/subscribers as published truth
- a later commit that repairs connectivity enables publication of a later
  coherent state; it does not retroactively publish the earlier blocked commit
- operators and inspection APIs must be able to ask why publication was blocked

This is not a vague diagnostics note. It is a product state.

## Phase 5: Custom Invariant Extensibility Without Type-Erasure Escape Hatches

### Goal

Add custom structural invariants without collapsing back into `Any`-driven
runtime type uncertainty.

### Public custom rule trait

The public rule trait must remain generic over its own scope type.

```rust
pub trait CustomInvariantRule:
    Send + Sync + std::panic::RefUnwindSafe + 'static
{
    type Scope: Send + Sync + 'static;

    fn descriptor(&self) -> CustomInvariantDescriptor;

    fn prepare_scope(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError>;

    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError>;
}
```

### Public descriptor and registration

```rust
pub struct CustomInvariantDescriptor {
    pub identity: CustomInvariantSemanticIdentity,
    pub display_name: Arc<str>,
    pub operational: CustomInvariantOperationalMetadata,
}
```

```rust
pub struct CustomInvariantRegistration {
    descriptor: CustomInvariantDescriptor,
    executable: Arc<dyn ErasedCustomInvariantRule>,
}
```

### Internal erased adapter model

Type erasure is permitted only at the framework storage boundary, not at the
semantic rule boundary.

Required internal types:

```rust
pub(crate) trait ErasedPreparedScope: Send + Sync {}
```

```rust
pub(crate) trait ErasedCustomInvariantRule: Send + Sync {
    fn descriptor(&self) -> &CustomInvariantDescriptor;
    fn prepare_erased(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Box<dyn ErasedPreparedScope>, CustomInvariantPreparationError>;
    fn evaluate_erased(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &dyn ErasedPreparedScope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError>;
}
```

Adapter rule:

- the generated erased adapter for a given custom rule owns both:
  - preparation of its concrete scope type
  - consumption of that same concrete scope type
- the framework never routes a prepared custom scope by id and later asks
  another executor to downcast it
- each lowered custom invariant packet must own:
  - exactly one erased executable
  - exactly one prepared scope produced by that executable

Therefore:

- no `PreparedCustomInvariantScope { inner: Arc<dyn Any> }` is allowed as a
  public or framework-wide semantic type
- if any downcast exists, it is contained inside the single generated adapter
  for that rule and cannot mismatch under correct packet construction
- the packet shape itself prevents "scope from rule A consumed by rule B"

### Custom packet ownership

Required packet form:

```rust
pub(crate) struct LoweredCustomInvariantPacket<'runtime> {
    descriptor: CustomInvariantDescriptor,
    executable: &'runtime RegisteredCustomInvariantRule,
    prepared_scope: PreparedErasedCustomScope,
    locality: InvariantLocalityProof,
}
```

The prepared scope never escapes its owning packet.

### Panic boundary

All custom preparation and execution must run through a dedicated panic
boundary:

```rust
pub(crate) fn run_custom_rule_safely<T>(
    identity: &CustomInvariantSemanticIdentity,
    phase: CustomInvariantRuntimePhase,
    run: impl FnOnce() -> T + std::panic::UnwindSafe,
) -> Result<T, CapturedCustomInvariantPanic>
```

Panic rules:

- panic must never crash the runtime
- panic becomes a typed invariant failure or preparation failure
- panic details are canonicalized into diagnostics artifacts
- panic count is included in invariant execution summary and counters

### Registry freeze rules

- duplicate custom semantic identity registration is a runtime build error
- unsupported execution point / failure effect combinations are rejected during
  registration
- the custom rule registry is frozen at runtime construction
- no post-build mutation of the registry is allowed

## Phase 6: Execution Context and Structural Access

### Goal

Expose enough structural state for geometry-grade custom invariants while
mechanically preventing boundary violations.

### Public custom execution context

```rust
pub struct CustomInvariantExecutionContext<'a> {
    observation_kind: InvariantObservationKind,
    execution_point: InvariantExecutionPoint,
    version_id: VersionId,
    touched: &'a TouchedStructuralSet,
    payloads: &'a StructuralPayloadView,
    relations: &'a StructuralRelationView,
    traversal: &'a SessionBoundTraversal<'a>,
    counts: &'a StructuralCountView,
}
```

### Required structural access helper types

```rust
pub struct TouchedStructuralSet { ... }
pub struct StructuralPayloadView { ... }
pub struct StructuralRelationView { ... }
pub struct SessionBoundTraversal<'a> { ... }
pub struct StructuralCountView { ... }
```

### Forbidden capabilities

This context must not expose:

- `RelationalRuntime`
- mutation authority
- signal graph access
- projection cache internals
- derived index internals as authority
- branch switching
- publication envelope mutation
- diagnostics construction APIs

Custom invariants must return domain results, not materialize framework
artifacts.

## Phase 7: Canonical Artifact and Derived Surface Rules

### Goal

Produce one canonical invariant artifact per execution boundary and derive all
other invariant surfaces from it.

### Canonical result artifact

Required result summary:

```rust
pub struct InvariantExecutionSummary {
    pub result_count: usize,
    pub pass_count: usize,
    pub advisory_count: usize,
    pub violation_count: usize,
    pub first_blocking_failure: Option<InvariantFailureRef>,
    pub first_publication_failure: Option<InvariantFailureRef>,
    pub panic_count: usize,
}
```

Required decision log:

```rust
pub struct InvariantDecisionRecord {
    pub packet_id: InvariantPacketId,
    pub rule_id: InvariantRuleId,
    pub execution_point: InvariantExecutionPoint,
    pub decision: InvariantDecisionKind,
    pub locality: InvariantLocalitySummary,
    pub metrics: InvariantRuleMetricsSnapshot,
}
```

```rust
pub enum InvariantDecisionKind {
    Passed,
    Advisory,
    Violated,
    SkippedByPlanContract,
    SkippedByMayBreakMask,
    SkippedByCostPolicy,
    RejectedDuringPreparation,
    PanicCaptured,
}
```

### Canonical ordering

Decision records must be canonically ordered by:

1. execution point
2. rule id canonical order
3. packet id
4. violation-field digest when needed for tie-breaking

No consumer may invent its own sort.

### Derived surfaces

The following must derive from `InvariantExecutionArtifact`:

- commit validation summary
- publication-blocked diagnostics
- replay digest inputs
- certification summaries
- inspection surfaces for blocked publication

`CommitLog` may derive traces from the artifact but may not become an
independent invariant authority surface.

## Phase 8: Complexity Contracts and Measurement Boundaries

### Goal

Make cost visible, honest, and testable.

### Required counters

At minimum add:

- `count_acyclicity_seed_edges`
- `count_acyclicity_frontier_expansions`
- `count_payload_validation_targets`
- `count_payload_field_constraints_checked`
- `count_partition_isolation_edges_checked`
- `count_connectivity_seed_entities`
- `count_connectivity_frontier_expansions`
- `count_custom_rule_preparations`
- `count_custom_rule_executions`
- `count_custom_rule_panics`
- `count_custom_scope_bytes`
- `count_invariant_packets_lowered`
- `count_invariant_packets_executed`
- `count_invariant_scope_widenings`

### Required complexity contracts

Add named contracts for:

- `validation::acyclicity::commit_boundary_reachability_cycle_detection`
- `validation::payload_schema::touched_payload_contract_validation`
- `validation::partition_isolation::touched_edge_partition_rejection`
- `validation::connectivity_minimum::publication_boundary_reachability_gate`
- `validation::custom::custom_rule_dispatch_overhead`

Each contract must declare:

- exact complexity statement
- boundedness basis
- measured counters
- verified vs debt status
- proof test name

### Required honesty statuses

- acyclicity:
  - `Verified` only if the actual implementation scope is candidate-kind
    bounded as claimed
  - otherwise `Debt`

- payload schema:
  - `Verified`

- partition isolation:
  - `Verified`

- connectivity minimum:
  - may begin as `Debt` if scope still broadens beyond candidate sources

- custom dispatch overhead:
  - `Verified`

No registration helper may silently infer `Global` cost for custom rules.
Cost class must be explicit.

## Phase 9: Certification and Domain Proof

### Goal

Close Milestone 6.5 with machine-checkable outputs and explicit new
certification coverage.

### Required compile-fail suites

UI tests must prove:

- custom rules cannot receive `RelationalRuntime`
- internal phase wrappers cannot be constructed externally
- execution cannot be invoked with registrations or descriptors instead of
  `ExecutionReadyInvariantBatch`
- unsupported registration helper usages fail to compile where helper APIs
  encode the restriction

### Required unit and integration suites

#### Native invariant suites

- hostile acyclicity creation sequences
- payload schema field/type/range violations
- partition-isolation rejection
- connectivity minimum publication blocking
- minimum-cardinality publication blocking remains correct after refactor

#### Custom invariant suites

- registration success and duplicate-id rejection
- custom preparation failure shaping
- custom execution failure shaping
- custom panic capture during preparation
- custom panic capture during execution
- savepoint rollback leaves zero custom-invariant residue
- packet-owned scope/executable pairing cannot mismatch

#### Publication-boundary suites

- committed-but-unpublished status is visible in `CommitResult`
- publication status encodes invariant blocking explicitly
- later repairing commit produces later publishable truth without retroactively
  publishing prior blocked commit

### Required certification addition

Add a new named requirement to `test-requirements.md`:

- `Invariant extensibility and structural legality certification test`

It must verify:

- native and custom parity in registration, planning, execution, and artifact
  shaping
- custom panic isolation
- hostile cycle-inducing rejection
- payload-schema rejection localization
- partition-isolation rejection under hostile writes
- connectivity and deferred minimum-cardinality publication semantics
- replay artifact parity for invariant-bearing histories

Required outputs:

- `invariant_artifact_digest`
- `custom_invariant_registry_digest`
- `invariant_decision_log_digest`
- `structural_legality_counter_snapshot`
- `custom_panic_capture_report`
- `publication_boundary_rejection_matrix`

## Important Public Interfaces and Types

### New public types

- `InvariantRuleId`
- `NativeInvariantRuleId`
- `CustomInvariantRuleId`
- `CustomInvariantSemanticVersion`
- `CustomInvariantSemanticIdentity`
- `CustomInvariantOperationalMetadata`
- `InvariantRuleDescriptor`
- `CustomInvariantDescriptor`
- `CustomInvariantRegistration`
- `CustomInvariantRule`
- `InvariantExecutionArtifact`
- `InvariantDecisionRecord`
- `CustomInvariantExecutionContext`
- `CustomInvariantScopePlanner`
- `TraversalBudgetSession`

### New internal proof-bearing types

- `InvariantRegistrationSet`
- `PreparedInvariantInputs`
- `InvariantSelection`
- `SelectedInvariantRegistrations`
- `LoweredInvariantPlan`
- `ExecutionReadyInvariantBatch`
- `LoweredInvariantPacket`
- `PreparedAcyclicityScopes`
- `PreparedPayloadValidationTargets`
- `PreparedPartitionIsolationScopes`
- `PreparedConnectivityScopes`
- `LoweredCustomInvariantPacket`
- `InvariantLocalityProof`

### Public interface constraints

- no public raw executable custom rule handles
- no public late-phase wrappers
- no public API that accepts loose runtime references for custom rule execution
- no public API that merges semantic identity and operational metadata into one
  opaque string bucket

## Assumptions and Defaults

- Milestone 6.5 begins with the structural split and phase wrappers before any
  new invariant behavior lands
- `SnapshotPublication` is the authority boundary for connectivity-minimum and
  deferred publication blocking
- replay truth compares canonical invariant artifacts, not live re-execution of
  custom code from the current binary
- custom invariants are registered at build time and the registry is frozen at
  runtime construction
- custom invariants are structural-only and may not observe signal or derived
  state
- acyclicity is implemented as a reachability-based commit-boundary check in
  this milestone, with explicit complexity honesty rather than false boundedness
  claims
- any path that cannot yet be compile-time enforced must be guarded by module
  visibility plus compile-fail tests and recorded as explicit debt

## Phase 10: Causal Metadata Endcap

### Goal

Immediately after the core Milestone 6.5 invariant work is complete, add
distributed-database-grade causal metadata so every authoritative mutation,
publication-boundary decision, and invariant outcome can be interpreted in
terms of:

- what this commit observed
- what this commit depended on
- what this commit superseded
- which concurrent or divergent histories were not observed

This is not a convenience tracing add-on. It is the causal authority substrate
needed for future intent reconciliation, branch-aware merge semantics,
conflict-classification honesty, and precise explanation of why two operations
can or cannot coexist.

### Why It Belongs Here

Milestone 6.5 is already making structural legality and publication eligibility
first-class authority concepts. Causal metadata is the natural endcap because:

- invariant failures often depend on what truth was visible at decision time
- publication-boundary rejection needs an honest causal basis, not only a point
  verdict
- custom invariants become much more valuable when their outputs are tied to an
  exact causal horizon
- future intent-reconciliation work will need causal ancestry and concurrency
  facts at the runtime boundary, not reconstructed after the fact

### Required Outcome

Every canonical commit and every invariant/publication artifact derived from it
must be able to answer:

- which branch head or version frontier this decision observed
- whether another commit is causally before, after, equal, or concurrent
- whether a rejection occurred because of true structural incompatibility or
  because the candidate decision was made against an older causal horizon
- whether two branch-local intent sets are in direct causal succession or are
  genuinely concurrent and therefore need reconciliation policy rather than
  overwrite semantics

### Required Types

```rust
pub struct CausalVersionDot {
    pub branch_id: BranchId,
    pub version_id: VersionId,
}
```

```rust
pub struct CausalFrontier {
    pub dots: Arc<[CausalVersionDot]>,
}
```

```rust
pub enum CausalRelation {
    Before,
    After,
    Equal,
    Concurrent,
}
```

```rust
pub struct CommitCausalMetadata {
    pub observed_frontier: CausalFrontier,
    pub produced_dot: CausalVersionDot,
    pub parent_commit_ids: Arc<[CommitId]>,
    pub concurrent_frontier: Arc<[CausalVersionDot]>,
}
```

```rust
pub struct InvariantCausalMetadata {
    pub execution_point: InvariantExecutionPoint,
    pub observed_frontier: CausalFrontier,
    pub target_version: VersionId,
    pub current_version: VersionId,
}
```

```rust
pub struct PublicationCausalMetadata {
    pub publication_frontier: CausalFrontier,
    pub blocked_by_versions: Arc<[VersionId]>,
}
```

### Structural Rules

- causal metadata must be part of canonical commit and invariant artifacts, not
  a best-effort debug log
- causal metadata must be derived exactly once at the commit/finalization
  boundary and then propagated as immutable proof data
- replay must compare causal metadata canonically; replay may not rediscover
  causal relation from ambient runtime state
- no consumer may infer concurrency by comparing timestamps or wall-clock order
- parent ordering and causal frontier ordering must be canonical and stable

### Public Surface Requirements

The runtime must expose:

- branch/version frontier inspection
- commit-to-commit causal comparison
- branch-head causal comparison
- invariant/publication artifact causal inspection
- explicit "concurrent vs ancestor vs descendant" answers as typed results, not
  bool flags

Required public types or equivalent:

- `CausalFrontier`
- `CausalRelation`
- `CommitCausalMetadata`
- `InvariantCausalMetadata`
- `PublicationCausalMetadata`

### Canonical Artifact Integration

`CanonicalCommitEnvelope` or its immediate derived companion must carry commit
causal metadata.

`InvariantExecutionArtifact` must carry invariant causal metadata.

Publication bundles and publication-blocked outcomes must carry publication
causal metadata.

This is required so later systems can reason about:

- whether a publication failure was based on the same observed frontier as a
  subsequent repair
- whether two branch-local changes are in succession or concurrency
- whether a future merge or intent-reconciliation layer is dealing with true
  conflict or stale-observation repair

### Complexity and Storage Requirements

Causal metadata must meet distributed-database-grade quality:

- causal comparison must be explicit about complexity and boundedness basis
- branch-local common cases must be cheap and directly represented
- multi-parent / merge-ready histories must preserve causal frontier truth
- storage must not require replay-from-genesis to answer ordinary causal
  comparison questions

If the implementation begins with branch-scoped frontiers rather than a fully
general vector-clock surface, that limitation must be named honestly in the
complexity contract and marked `Debt` where appropriate.

### Acceptance Requirements

This endcap is complete only when the runtime proves:

- canonical causal metadata is persisted through commit, replay, and durability
- replay reproduces identical causal metadata for equivalent histories
- branch-local commits compare causally without ambiguity
- concurrent branch-local commits are classified as `Concurrent` explicitly
- publication-boundary failures retain the exact observed frontier that caused
  the rejection
- invariant artifacts expose the causal horizon used during evaluation

### Required Certification Addition

Add a named certification carrier after the rest of Milestone 6.5 is complete:

- `Causal metadata and concurrency classification test`

It must verify:

- ancestor/descendant/equal/concurrent classification across branch-local
  histories
- canonical frontier persistence through durability and replay
- invariant/publication artifacts carry stable causal metadata
- merge-ready multi-parent fixtures preserve causal truth
- no timestamp-based or incidental ordering leaks into causal classification

Required outputs:

- `causal_frontier_digest`
- `causal_relation_matrix`
- `commit_causal_metadata_digest`
- `invariant_causal_metadata_digest`
- `publication_causal_metadata_digest`

### Scheduling Rule

This work is intentionally placed at the end of Milestone 6.5.

Execution order:

1. complete the core invariant type-system, planning, execution, and
   certification work
2. add canonical causal metadata to the resulting commit/invariant/publication
   artifacts
3. close the causal certification carrier

The point is to prevent causal metadata from becoming a speculative abstraction
detached from the now-hardened invariant and publication surfaces it must
actually explain.
