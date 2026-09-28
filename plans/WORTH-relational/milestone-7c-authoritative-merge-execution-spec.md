# Milestone 7C Authoritative Merge Execution Spec

## Purpose

This document is the build specification for Milestone 7C.

It is not a roadmap summary. It defines the concrete execution model, proof
chain, type surfaces, integration points, failure topology, observability
requirements, and certification strategy for authoritative merge execution in
`worth-relational`.

The governing rule remains:

`parallelize disposable work, serialize authority`

Merge execution is therefore not a special side path. It is a serialized truth
commit that must consume pre-lowered merge semantics and publish the same
canonical artifacts, replay guarantees, diagnostics guarantees, and durability
guarantees as any other authoritative commit.

## Problem Statement

Milestone 7B made merge semantics explicit and replayable at planning time.
Milestone 7C must make merge execution authoritative without allowing merge
meaning to leak back into procedural execution code.

The naive failure mode is predictable:

- planning classifies records, but execution re-decides what those
  classifications mean
- merge identity matching is treated as advisory during planning and
  authoritative during execution
- policy resolution is recomputed from raw values during apply
- branch heads move between planning and commit, but execution still proceeds
- merge commits publish partial truth or diagnostics inconsistent with replay
- merge behaves like a "transaction with extra parents" instead of a distinct
  truth operation

This milestone exists to make those failure modes structurally impossible.

## Adversarial Constraints

The design must survive all of the following simultaneously:

1. A merge request may be planned against one target/source head pair and
   executed later under branch movement pressure. Execution must reject stale
   plans explicitly.
2. Replay of merge-bearing histories must produce identical snapshots, patch
   artifacts, diagnostics summaries, branch heads, lineage surfaces, and query
   surfaces.
3. Merge must remain fail-closed. If any record remains blocked or rejected, no
   authoritative merge truth may publish.
4. Hot-path execution must not rediscover identity matching, policy choice,
   conflict classification, causal disposition, or authorized aspect value
   surfaces.
5. Diagnostics must localize merge behavior with the same truth-grade rigor as
   ordinary commits. "merge failed" is not a diagnosis.
6. Cost must be visible. A merge that scans broad branch deltas or large visible
   record sets must expose that breadth explicitly and never masquerade as a
   cheap history helper.

## Existing Baseline We Must Preserve

Milestone 7C builds on the current 7B and commit pipeline surfaces already
present in the crate:

- merge planning ontology in `crates/worth-relational/src/merge`
- canonical planning artifact and digest basis
- lowered record decisions and authorized aspect value surfaces
- causal metadata and causal annotations
- schema-declared merge policies
- canonical commit envelopes and merge-ready parent lists
- serialized commit pipeline in `crates/worth-relational/src/authority/commit/pipeline.rs`

The current architecture already contains one crucial contract:

- planning emits `LoweredRecordDecision`
- execution is only authorized to consume lowered record decisions

Milestone 7C must preserve and strengthen that contract. It must not weaken it
by threading raw classifications into authority-path mutation.

## Scope

Milestone 7C ships authoritative execution for merge requests that are fully
execution-ready under the canonical lowered plan.

Execution-ready means:

- every lowered record is `Admitted`
- no record is `Blocked`
- no record is `Rejected`
- source and target branch heads still match the heads seen during planning
- the merge base and schema semantics seen during planning still match the
  current authoritative state

Milestone 7C does not invent manual merge tooling. Any plan requiring manual
resolution, deletion-specific resolution, or relation-endpoint rewiring
resolution remains an explicit typed failure. That is not a missing edge case;
it is the required fail-closed behavior for this milestone.

## Architectural Decision

Authoritative merge execution will be modeled as a distinct commit mode with
its own proof-carrying input, not as a normal transaction that happens to set
`merge_parent_branches`.

That distinction must be made explicit because the two operations have
different authorities:

- a normal transaction authority is "caller supplied these mutation intents"
- a merge authority is "the runtime supplied this canonical lowered merge plan"

Conflating those authorities would permit host-side heuristics to become
accidental truth.

## Implementation Structure

This section is the actual build spine for Milestone 7C. The rest of the
document defines the semantic and architectural constraints. This section
defines the order we will implement them, where each piece lands in the crate,
what each phase is allowed to depend on, and what must be true before we move
to the next phase.

The milestone must be executed in dependency order. We are not allowed to start
at commit integration and "fill in the types later." That would recreate
implicit semantics through convenience code.

Implementation phases:

1. Phase A: Execution substrate and sealed proof types
2. Phase B: Freshness binding and authority-context verification
3. Phase C: Execution-plan compilation and value materialization
4. Phase D: Merge-to-mutation derivation
5. Phase E: Commit pipeline integration
6. Phase F: Diagnostics, counters, and artifact ownership
7. Phase G: Certification and hostile-path hardening

Only one phase may be considered in progress at a time for architectural
completion. Parallel coding is acceptable only when write scopes do not overlap
and the dependency direction remains intact.

### Phase A: Execution Substrate And Sealed Proof Types

#### Goal

Create the type-level boundary that prevents non-executable merge plans from
reaching authority-path execution.

#### Why This Phase Comes First

Everything else depends on one core fact: execution must accept only a
runtime-produced proof object. If we begin with pipeline integration or helper
APIs before sealing the proof boundary, the codebase will accumulate ad hoc
"temporary" merge execution entrypoints that later become difficult to remove.

#### Code Ownership

Primary modules:

```text
crates/worth-relational/src/merge/data/execution.rs
crates/worth-relational/src/merge/data/execution_errors.rs
crates/worth-relational/src/merge/facade.rs
crates/worth-relational/src/facade.rs
```

#### Types Added In This Phase

Public:

- `MergeExecutionRequest`
- `PreparedMergeExecution`
- `MergeExecutionPreparationError`
- `MergeExecutionError`
- `MergeExecutionFreshnessPolicy`

Crate-private or sealed:

- `PreparedMergeExecutionToken`
- `ExecutionReadyLoweredMergePlan`
- constructor seals for `ExecutionReadyLoweredMergePlan`

#### Required Construction Rules

- only `MergeAccess` or merge authority code may construct
  `PreparedMergeExecution`
- only `ExecutionReadyLoweredMergePlan::try_from_lowered(...)` may create the
  execution-ready proof
- any lowered plan containing `Blocked` or `Rejected` decisions fails here
- the public API must not accept `MergePlanningArtifactCore` directly for
  execution

#### Tests Added In This Phase

- execution-ready conversion succeeds only for fully admitted lowered plans
- blocked lowered plans cannot produce prepared execution
- rejected lowered plans cannot produce prepared execution
- public merge execution APIs do not expose constructors that permit caller
  synthesis

#### Exit Criteria

We do not leave Phase A until the illegal state "blocked or rejected merge
artifact can enter execution" is unrepresentable in code.

### Phase B: Freshness Binding And Authority-Context Verification

#### Goal

Bind prepared merges to the authority context that produced them and define the
exact 7C freshness contract.

#### Why This Phase Comes Second

Without authority binding, a sealed proof object is still too weak. It prevents
callers from fabricating merge readiness, but it does not yet prevent stale or
cross-context execution.

#### Code Ownership

Primary modules:

```text
crates/worth-relational/src/merge/data/execution.rs
crates/worth-relational/src/merge/logic/execution_preflight.rs
crates/worth-relational/src/history/logic/access.rs
crates/worth-relational/src/schema/data/mod.rs
```

#### Types Added Or Extended

- `MergeExecutionAuthorityBinding`
- `RuntimeInstanceId` or equivalent runtime-scoped authority identity
- preflight result surface for freshness verification

#### Required Checks Implemented In This Phase

- target head parity
- source head parity
- merge base parity
- schema semantic snapshot digest parity
- runtime/authority instance parity

#### Required Behavior

- preflight verification is read-only
- preflight emits typed failures only
- preflight does not attempt partial continuation
- preflight does not replan
- preflight does not broaden the admitted record set

#### Tests Added In This Phase

- prepared merge from runtime instance A cannot execute on runtime instance B
- target-head drift rejects before mutation
- source-head drift rejects before mutation
- merge-base drift rejects before mutation
- schema-semantic drift rejects before mutation

#### Exit Criteria

We do not leave Phase B until the illegal state "prepared merge executes in a
different authority context or against drifted branch/schema state" is rejected
before mutation derivation starts.

### Phase C: Execution-Plan Compilation And Value Materialization

#### Goal

Compile the execution-ready lowered plan into a representation that is narrow
enough for authority-path consumption and explicit enough about value carriage
that cost and semantics remain honest.

#### Why This Phase Exists

This phase is allowed to exist only if it earns its keep. It must either:

- eliminate merge-aware semantic branching from downstream code
- bind value-materialization policy and authority-context details in a way the
  lower-level proof type does not

If implementation reveals that it does neither, this phase must be collapsed
into `ExecutionReadyLoweredMergePlan` and the extra type removed.

#### Code Ownership

Primary modules:

```text
crates/worth-relational/src/merge/data/execution.rs
crates/worth-relational/src/merge/data/execution_artifacts.rs
crates/worth-relational/src/merge/logic/execution_compilation.rs
```

#### Types Added In This Phase

- `BoundExecutableMergePlan`
- `BoundExecutableMergeRecordPlan`
- `AdoptSourceRecordPlan`
- `PreserveSharedRecordPlan`
- `ReconcileRecordPlan`
- `ExecutableAspectPlan`
- `MaterializedAspectValue`
- `MergeValueMaterialization`

#### Allowed Inputs

Compilation may consume:

- `ExecutionReadyLoweredMergePlan`
- current runtime read surfaces needed to materialize authorized values
- schema/aspect plans already frozen by the planning artifact

Compilation may not consume:

- raw merge conflict classification as a new authority source
- new identity matching logic
- new policy resolution logic
- broad branch traversal beyond what is required to materialize already
  authorized execution inputs

#### Concrete Work

For each admitted lowered record:

- map lowered execution bundle kind to one executable record plan variant
- map each admitted lowered aspect intent to one executable aspect plan
- choose explicit value-materialization policy for each value-bearing aspect
- retain policy/resolution provenance needed for diagnostics

#### Tests Added In This Phase

- admitted `KeepSourceAddition` lowers to `AdoptSource`
- admitted `KeepExactSharedTruth` lowers to `PreserveShared`
- admitted `ReconcileSchemaCorrespondence` lowers to `Reconcile`
- compilation does not accept blocked/rejected aspect outcomes
- value-materialization policy is encoded explicitly in compiled aspect plans

#### Exit Criteria

We do not leave Phase C until downstream code can consume executable record
plans without needing to branch on raw merge conflict classes or raw policy
records.

### Phase D: Merge-To-Mutation Derivation

#### Goal

Derive ordinary authoritative mutation intents from the compiled executable
merge plan.

#### Why This Phase Is Separate

This is the point where merge semantics stop being merge-native and become
ordinary runtime mutation. That boundary must be explicit. If we collapse it
into commit application logic, merge semantics will leak into the hot path.

#### Code Ownership

Primary modules:

```text
crates/worth-relational/src/merge/logic/execution_mutation_plan.rs
crates/worth-relational/src/transactions/data/outcomes.rs
crates/worth-relational/src/transactions/data/intents/*
```

#### Types Added In This Phase

- `MergeCommitMutationPlan`
- `MergeExecutionStructuralSummary`
- `MergeExecutionSummary`

#### Concrete Work

- derive parent commit ordering for the pairwise merge request
- derive `merged_intents` from executable record plans only
- derive any lineage-affecting intents required by executable reconciliation
- derive structural summaries needed by commit/invariant/artifact phases
- prohibit any mutation intent that was not authorized by the compiled plan

#### Required Invariants

- `merged_intents` are runtime-derived, never caller-supplied
- pairwise merge parent ordering is deterministic
- source branch head does not advance
- target branch head is the sole advanced branch head on success

#### Tests Added In This Phase

- executable source adoption produces only source-authorized create intents
- exact-shared execution does not fabricate mutating intents
- reconcile execution produces only target-converging intents for the 7C
  executable reconciliation class
- derived mutation does not depend on re-reading raw planning classifications

#### Exit Criteria

We do not leave Phase D until a complete `MergeCommitMutationPlan` can be
produced from a prepared merge without any caller staging and without any
execution-time semantic reinterpretation.

### Phase E: Commit Pipeline Integration

#### Goal

Thread merge execution through the existing serialized commit lifecycle without
forking publication, durability, replay, or invariant semantics.

#### Why This Phase Comes After Mutation Derivation

The commit pipeline should not know how to interpret merge semantics. It should
know how to consume an authority input that has already been lowered into
commit-ready form.

#### Code Ownership

Primary modules:

```text
crates/worth-relational/src/authority/commit/pipeline.rs
crates/worth-relational/src/authority/commit/phases/merge_prepare.rs
crates/worth-relational/src/authority/commit/phases/merge_history.rs
crates/worth-relational/src/history/logic/authority.rs
crates/worth-relational/src/replay/data/mod.rs
crates/worth-relational/src/durability/authority.rs
```

#### Types Added Or Extended

- `CommitAuthorityInput`
- merge-specific history resolution input shape
- merge-specific publication summary enrichments where required

#### Lifecycle Rules

Authority-strategy-specific:

- merge preflight
- merge mutation-plan input
- merge diagnostics enrichment

Lifecycle-common:

- invariant pre-check
- authoritative mutation
- history publication lifecycle
- artifact assembly
- durable append
- final publication

#### Concrete Work

- add merge authority entrypoint that produces `CommitAuthorityInput::Merge`
- reuse existing commit pipeline lifecycle where possible
- ensure merge parent list and merge base list are published through canonical
  history and envelope surfaces
- ensure replay and durability read merge-produced histories without special
  casing the observable result

#### Tests Added In This Phase

- successful merge commit produces ordered multi-parent commit reference
- merge commit publishes through canonical envelope path
- merge commit survives durability append and recovery
- merge-produced history satisfies existing merge-ready history expectations

#### Exit Criteria

We do not leave Phase E until merge execution uses the same publication and
durability truth path as ordinary commits, with no side publication channel and
no merge-only replay semantics.

### Phase F: Diagnostics, Counters, And Artifact Ownership

#### Goal

Make merge execution operationally inspectable and cost-honest.

#### Why This Phase Is Not Optional Polish

Without this phase, 7C could appear correct while still violating the
architecture's requirement that cost and authority reasoning be visible at the
boundary.

#### Code Ownership

Primary modules:

```text
crates/worth-relational/src/merge/data/execution_artifacts.rs
crates/worth-relational/src/diagnostics/data/mod.rs
crates/worth-relational/src/performance/data/mod.rs
crates/worth-relational/src/performance/logic/access.rs
```

#### Concrete Work

- add planning/readiness artifact ownership rules
- add execution artifact ownership rules
- add merge execution failure artifact ownership rules
- add all required merge execution counters
- add digest ownership rules so later phases reuse earlier digest bases where
  valid

#### Required Diagnostic Separation

- planning/readiness artifacts may contain admitted, blocked, and rejected rows
- execution artifacts may contain executed rows only
- failure artifacts may contain stale/drift/commit failure evidence only

#### Tests Added In This Phase

- execution diagnostics cannot represent blocked/rejected rows
- stale execution emits failure artifact without mutation artifact leakage
- merge execution counters reflect compilation breadth and freshness breadth
- digest surfaces remain stable across replay/recovery for merge-produced
  histories

#### Exit Criteria

We do not leave Phase F until merge execution can be inspected without mixing
preparation truth, execution truth, and failure truth.

### Phase G: Certification And Hostile-Path Hardening

#### Goal

Prove that the implemented merge path is truth-grade under replay, recovery,
history shape, and hostile drift conditions.

#### Code Ownership

Primary modules:

```text
crates/worth-relational/src/tests/history/
crates/worth-relational/src/tests/publication/
crates/worth-relational/src/tests/complexity/contracts/
crates/worth-relational/src/tests/support/
```

#### Certification Work

- add the authoritative merge execution certification suite
- ensure merge-produced histories satisfy hostile replay equivalence
- ensure merge-produced histories satisfy durable recovery equivalence
- ensure merge-produced histories satisfy merge-ready history shape tests
- add complexity proof tests for the merge execution boundary

#### Exit Criteria

Milestone 7C is not complete when the code compiles. It is complete when the
new certification suite passes and the existing replay/recovery/history suites
still pass on real merge-produced histories.

## Public Surface

We will add a dedicated merge execution surface rather than extending ordinary
transaction staging with hidden merge semantics.

```rust
pub struct MergeExecutionRequest {
    pub target_branch: BranchId,
    pub source_branch: BranchId,
    pub merge_intent: MergeIntent,
}

pub struct PreparedMergeExecution {
    pub request: MergeExecutionRequest,
    pub artifact: MergePlanningArtifactCore,
    // Opaque to callers; constructed only by merge authority.
    execution_token: PreparedMergeExecutionToken,
}

pub struct MergeExecutionOutcome {
    pub commit: CommitResult,
    pub execution_summary: MergeExecutionSummary,
}
```

`PreparedMergeExecution` is the key public proof type. Callers may inspect the
planning artifact, but they may not synthesize the execution token. Only the
runtime may produce that token after validating that the lowered plan is fully
execution-ready.

Public entrypoints:

```rust
impl RelationalRuntime {
    pub fn prepare_merge_execution(
        &self,
        request: MergeExecutionRequest,
    ) -> Result<PreparedMergeExecution, MergeExecutionPreparationError>;

    pub fn execute_prepared_merge(
        &mut self,
        prepared: PreparedMergeExecution,
    ) -> Result<MergeExecutionOutcome, MergeExecutionError>;
}
```

This two-step API makes two implicit assumptions explicit:

- merge planning and merge execution are distinct phases
- execution consumes a runtime-produced proof object, not a caller-assembled
  "merge configuration"

## Internal Proof Chain

The internal proof chain will be:

```rust
MergePlanningRequest
-> LoweredMergePlan
-> ExecutionReadyLoweredMergePlan
-> BoundExecutableMergePlan
-> MergeCommitMutationPlan
-> CommitResult
```

Each transition exists to make one specific property explicit.

### `ExecutionReadyLoweredMergePlan`

This type proves that every record in the lowered plan is admissible for
execution.

```rust
struct ExecutionReadyLoweredMergePlan {
    request: MergePlanningRequest,
    target_head: CommitReference,
    source_head: CommitReference,
    merge_base: ResolvedMergeBase,
    ancestry: MergeAncestrySummary,
    schema_snapshot: MergeSchemaSnapshotDigestBasis,
    authority_contract: MergeExecutionAuthorityContract,
    lowered_records: Arc<[LoweredMergePlanRecord]>,
    decision_log: MergePlanningDecisionLog,
    digest_basis: MergeArtifactDigestBasis,
}
```

Construction rules:

- sealed constructor
- only produced if `fully_execution_ready == true`
- only produced if every record decision is `LoweredRecordDecision::Execute`
- only produced if the authority contract matches the runtime's supported
  execution contract exactly

This type eliminates the illegal state "executor receives a lowered plan that
contains blocks or rejections."

### `BoundExecutableMergePlan`

This type exists only if it proves two things that
`ExecutionReadyLoweredMergePlan` does not already prove:

1. the prepared merge is bound to a specific authority context and freshness
   basis
2. the executor no longer needs merge-aware semantic branching in order to
   derive commit-ready mutation

If implementation discovers that this stage does not eliminate those illegal
states concretely, it must be collapsed back into
`ExecutionReadyLoweredMergePlan`. This stage is not allowed to survive as a
ceremonial repackaging layer.

When it does exist, it compiles lowered intents into concrete runtime actions
without touching authority yet.

```rust
struct BoundExecutableMergePlan {
    request: MergeExecutionRequest,
    authority_binding: MergeExecutionAuthorityBinding,
    target_head: CommitReference,
    source_head: CommitReference,
    merge_base: ResolvedMergeBase,
    schema_snapshot: MergeSchemaSnapshotDigestBasis,
    parent_order: OrderedParentList,
    record_plans: Arc<[BoundExecutableMergeRecordPlan]>,
    structural_summary: MergeExecutionStructuralSummary,
    diagnostics_plan: MergeExecutionDiagnosticsPlan,
    digest: ExecutableMergePlanDigest,
}

enum BoundExecutableMergeRecordPlan {
    AdoptSource(AdoptSourceRecordPlan),
    PreserveShared(PreserveSharedRecordPlan),
    Reconcile(ReconcileRecordPlan),
}
```

The important rule is that this stage is still plan compilation, not mutation.
Its job is to convert abstract execution bundles into concrete record-level
operations that the commit pipeline can apply deterministically.

### Record Plan Shapes

```rust
struct AdoptSourceRecordPlan {
    source_record: RecordRef,
    record_kind: VisibleMergeRecordKind,
    source_visible_snapshot: VisibleMergeRecordSnapshot,
    aspect_plan: Arc<[ExecutableAspectPlan]>,
}

struct PreserveSharedRecordPlan {
    record: RecordRef,
    target_record: Option<RecordRef>,
    equality_witness: SharedTruthWitness,
    aspect_plan: Arc<[ExecutableAspectPlan]>,
}

struct ReconcileRecordPlan {
    source_record: RecordRef,
    target_record: RecordRef,
    identity_basis: ReconciledIdentityBasis,
    causal_disposition: MergeRecordCausalDisposition,
    aspect_plan: Arc<[ExecutableAspectPlan]>,
}

enum ExecutableAspectPlan {
    AdoptSourceValue {
        aspect_key: AspectKey,
        source_value: MaterializedAspectValue,
    },
    PreserveSharedValue {
        aspect_key: AspectKey,
        shared_value_digest: AspectValueDigest,
    },
    ReconcileValue {
        aspect_key: AspectKey,
        source_value: Option<MaterializedAspectValue>,
        target_value: Option<MaterializedAspectValue>,
        base_value: Option<MaterializedAspectValue>,
        resolved_value: MaterializedAspectValue,
        resolution_basis: AspectResolutionBasis,
    },
}
```

These types make several assumptions explicit that are currently only implicit
in `LoweredAspectExecutionIntent`:

- the executor needs canonical values, not just authorization classes
- preserving equality is a proof-bearing path, not "just pick either side"
- reconciliation must record why a value was chosen, not only what was chosen

The first bullet must not be implemented naively. "Needs canonical values" does
not automatically authorize "embed copied values directly into the prepared
artifact."

### Authority Binding

Prepared merge execution must bind structurally to the authority context that
produced it. Opaqueness alone is not sufficient.

```rust
struct MergeExecutionAuthorityBinding {
    runtime_instance_id: RuntimeInstanceId,
    target_head_commit_id: CommitId,
    source_head_commit_id: CommitId,
    merge_base_commit_id: CommitId,
    schema_snapshot_digest: String,
    freshness_policy: MergeExecutionFreshnessPolicy,
}
```

This binding exists to prevent the illegal state "prepared merge from one
authority context is executed in another authority context that only looks
equivalent."

### Value Materialization Policy

Executable merge plans must carry an explicit value materialization policy
rather than silently embedding large concrete values into the prepared artifact.

```rust
enum MergeValueMaterialization {
    EqualityWitnessDigest,
    SnapshotPinnedRead,
    InternedCanonicalValueHandle,
    EagerInlineCanonicalValue,
}

struct MaterializedAspectValue {
    policy: MergeValueMaterialization,
    payload: MaterializedAspectValuePayload,
}
```

For 7C, the default policy should be the lightest one that preserves exact
execution semantics for the executable record class in question.

This section exists because otherwise the prepared merge artifact can quietly
turn into a large frozen data bundle, which would change the runtime's memory,
serialization, replay, and durability cost profile without the architecture
admitting it.

## Merge-to-Commit Integration

The current commit pipeline is correct and should be reused. We will not create
an alternate publication path for merges.

Integration will happen by extending the commit authority with a second input
mode:

```rust
enum CommitAuthorityInput {
    Mutation(MergedCommitPlan),
    Merge(MergeCommitMutationPlan),
}
```

`MergeCommitMutationPlan` is the bridge object between merge execution planning
and the existing authoritative mutation phase.

```rust
struct MergeCommitMutationPlan {
    transaction_id: TransactionId,
    target_branch: BranchId,
    source_branch: BranchId,
    parent_commits: OrderedParentList,
    merge_base_commits: Arc<[CommitId]>,
    executable_plan: BoundExecutableMergePlan,
    merged_intents: Vec<MutationIntent>,
    merge_execution_summary: MergeExecutionSummary,
}
```

Important rule:

- `merged_intents` are derived from `ExecutableMergePlan`, never from caller
  staging

The commit pipeline then consumes `CommitAuthorityInput::Merge` through the
same explicit phases:

1. preflight validation of branch-head freshness and schema digest parity
2. invariant pre-check using the derived merge mutation plan
3. authoritative mutation apply
4. history resolution using explicit parent list and merge base list
5. invariant post-check over merged truth
6. artifact assembly, durability append, publication

This split must not fork the lifecycle into two mostly duplicated pipelines.
The lifecycle remains shared. What differs is only:

- authority-specific preflight
- authority-specific mutation-plan derivation
- authority-specific diagnostics enrichment

Invariant execution, history publication lifecycle, artifact assembly,
durability append, and final publication remain lifecycle-common.

## Freshness and Staleness Rules

The executor must reject stale prepared merges before any mutation work begins.

We will add:

```rust
enum MergeExecutionPreparationError {
    Planning(MergePlanningError),
    NotExecutionReady(MergeExecutionReadinessReport),
}

enum MergeExecutionError {
    StaleBranchHead {
        branch: BranchId,
        planned: CommitId,
        current: Option<CommitId>,
    },
    MergeBaseDrift {
        planned: CommitId,
        current: Option<CommitId>,
    },
    SchemaSemanticDrift {
        planned: MergeSchemaSnapshotDigestBasis,
        current: MergeSchemaSnapshotDigestBasis,
    },
    Commit(TransactionCommitError),
}
```

Freshness checks are mandatory and must be exact for Milestone 7C:

- target branch current head commit must equal planned target head commit
- source branch current head commit must equal planned source head commit
- recomputed merge base for those heads must equal planned merge base
- recomputed touched schema semantic snapshot must equal planned snapshot digest

If any check fails, execution fails before authoritative mutation. There is no
"best effort continue if close enough."

This is a milestone-local execution policy, not a declaration that all future
merge execution must use whole-plan invalidation. To make that explicit, the
freshness policy must be named in the type system:

```rust
enum MergeExecutionFreshnessPolicy {
    ExactAuthorityParity,
}
```

7C uses `ExactAuthorityParity` because it is the simplest fail-closed policy
consistent with truth authority. It must not be treated as proof that future
scoped revalidation is architecturally invalid.

## Parent Ordering

Parent ordering is authoritative history structure and must be deterministic.

Rule for 7C:

- parent `0` is always the target branch head
- parent `1` is the source branch head

The commit envelope, history node, replay surfaces, durability surfaces, and
diagnostics must all reflect this exact ordering.

We are intentionally keeping the execution request single-source in this
milestone because the current 7B request and planning ontology are pairwise.
This keeps the architecture honest. We are not going to fake multi-source merge
execution by repeatedly replaying pairwise merges inside one commit boundary.

The code must make this limitation explicit through type shape rather than
documentation.

## Mapping Lowered Decisions to Concrete Mutation

The translation table is fixed for the 7C executable subset:

- `KeepSourceAddition` -> create the source-visible record in target truth
- `KeepExactSharedTruth` -> emit no semantic mutation for the record, but retain
  equality evidence in diagnostics and merge execution summary
- `ReconcileSchemaCorrespondence` -> update the existing target record in place
  using the resolved aspect plans; this preserves target storage identity while
  recording merge lineage and diagnostics

This mapping is important: merge execution does not introduce a new mutation
language. It lowers into the existing mutation language where possible and
records merge-specific authority in diagnostics and envelopes.

The storage-identity rule above is milestone-local:

- for the 7C executable reconciliation class, reconciliation converges into the
  existing target record

This must not be interpreted as a permanent statement that all future
reconciliation semantics preserve target storage identity. Future merge classes
may require different authority artifacts even if physical slot reuse still
occurs.

### Executable Subset vs Full Merge Ontology

The 7C executable set is intentionally smaller than the long-term merge
ontology.

```rust
enum MergeResolutionClass {
    SourceOnlyAddition,
    ExactSharedTruth,
    SchemaDeclaredCorrespondence,
    Deletion,
    RelationEndpointRewiring,
    ManualConflict,
}

enum MergeExecutableClass {
    AdoptSourceAddition,
    PreserveExactSharedTruth,
    ReconcileDeclaredCorrespondence,
}
```

This distinction exists to prevent the 7C trichotomy from hardening into the
runtime's permanent ontology. Deletion and rewiring are real merge classes in
the ontology even when they remain non-executable in 7C.

## Merge Lineage and History Semantics

Merge execution must not smuggle merge identity through ordinary replace
semantics.

Rules:

- merge parentage is history authority, not lineage authority
- lineage changes occur only when the lowered merge record plan explicitly
  requires a lineage-affecting mutation
- branch-head advancement moves only the target branch
- source branch head remains unchanged
- replay of the merge commit must reconstruct the same parent list, merge base
  set, and branch-head advancement

## Diagnostics and Artifact Additions

We will add a merge execution artifact family parallel to the 7B planning
artifacts.

```rust
struct MergeExecutionSummary {
    request: MergeExecutionRequest,
    target_head: CommitId,
    source_head: CommitId,
    merge_base: CommitId,
    executed_record_count: usize,
    adopted_source_record_count: usize,
    preserved_shared_record_count: usize,
    reconciled_record_count: usize,
    diagnostics_digest: String,
    execution_digest: String,
}

struct MergeExecutionDiagnosticArtifact {
    request: MergeExecutionRequest,
    plan_digest: String,
    executed_records: Arc<[ExecutedMergeRecordDiagnosticRow]>,
}
```

Even on successful execution, the runtime must retain enough merge execution
evidence to answer:

- which records were adopted
- which records were preserved as equal
- which existing target records were reconciled
- which aspect policies were applied
- which authorized value surfaces were consumed

Blocked and rejected rows do not belong in successful execution diagnostics.
Those are preparation/readiness artifacts.

The artifact families must stay phase-honest:

- planning/readiness artifact: may include admitted, blocked, and rejected rows
- execution artifact: executed rows only
- execution failure artifact: stale, drift, or commit-stage failure evidence

## Complexity Contracts

Milestone 7C adds one new named contract:

- `runtime.merge.execution`

Declared budget summary:

- branch-head freshness checks: `O(1)`
- executable plan compilation: `O(admitted_records + admitted_aspects)`
- merge mutation derivation: `O(executed_records + executed_aspects)`
- authoritative apply: bounded by derived mutation breadth, not by total branch
  history breadth

The contract must distinguish the following scaling surfaces explicitly:

- ancestry breadth
- touched-record breadth
- visible source-record breadth
- target-side identity index breadth
- schema snapshot breadth
- executable aspect breadth
- derived mutation breadth

Only some merge phases are allowed to scale with each surface:

- planning may scale with ancestry breadth and touched-record breadth
- identity discovery may scale with target-side identity index breadth
- schema snapshotting may scale with schema snapshot breadth
- execution compilation may scale with executable aspect breadth
- authoritative apply must scale with derived mutation breadth

The implementation must not hide broad planning or compilation work behind a
narrow final apply budget claim.

Required counters:

- `merge_execution_requests`
- `merge_execution_record_plans_compiled`
- `merge_execution_aspect_plans_compiled`
- `merge_execution_records_adopted`
- `merge_execution_records_preserved`
- `merge_execution_records_reconciled`
- `merge_execution_stale_head_rejections`
- `merge_execution_schema_drift_rejections`
- `merge_execution_elapsed_nanos`

Additional explanatory counters:

- `merge_execution_freshness_head_checks`
- `merge_execution_freshness_schema_checks`
- `merge_execution_visible_source_records_materialized`
- `merge_execution_target_records_revalidated`
- `merge_execution_values_inlined`
- `merge_execution_values_snapshot_pinned`

These counters must appear in `RuntimeComplexityCounters` and be asserted by
named proof tests.

## Module Decomposition

This milestone must not become a single `merge_execution.rs` blob.

Required module structure:

```text
crates/worth-relational/src/merge/
  data/
    execution.rs
    execution_artifacts.rs
    execution_errors.rs
  logic/
    execution_preflight.rs
    execution_compilation.rs
    execution_mutation_plan.rs
    execution_diagnostics.rs
```

Commit-side integration should live in commit-specific modules, not inside
`merge/logic`:

```text
crates/worth-relational/src/authority/commit/phases/
  merge_prepare.rs
  merge_history.rs
```

If normal commit and merge commit code share lifecycle but differ in strategy,
extract the shared lifecycle and parameterize the strategy. Do not collapse the
domains into one generic "apply commit mode" file unless the lifecycle can be
named honestly.

## Explicit Assumptions We Are Making Structural

The implementation must encode all of the following explicitly:

- merge execution is pairwise in 7C
- manual-resolution merges are not executable in 7C
- deletion-specific merge semantics are not executable in 7C
- relation-endpoint rewiring merges are not executable in 7C
- branch-head drift invalidates prepared execution
- schema semantic drift invalidates prepared execution
- merge execution consumes runtime-produced proof objects only
- no partial merge truth may publish

The implementation must also encode the following as explicit milestone-local
policies rather than ontological truths:

- whole-plan invalidation on freshness drift
- in-place target convergence for executable reconciliation
- the current executable subset of merge classes

None of these may remain as comments or caller expectations.

## Implementation Red Flags

The architecture is strong enough to support 7C cleanly, but several failure
modes remain likely during implementation because they arise from "helpful"
local decisions that quietly violate the authority model or widen the cost
surface.

This section exists to name those risks before code is written.

### Red Flag 1: Execution Reinterprets Planning

The single biggest semantic risk is allowing execution to do semantic work that
belongs to planning.

Execution must not:

- rediscover correspondence
- re-run merge policy selection
- reclassify conflicts from raw visible state
- normalize aspect values differently from the prepared artifact
- broaden what counts as executable

In 7C, execution is authorized to do exactly four kinds of work:

- verify freshness and authority binding
- compile already-admitted plan state into commit-ready mutation
- run the normal authoritative commit lifecycle
- emit diagnostics from the same authority inputs consumed by commit

If execution needs to "just check one more thing" about merge meaning, that is
almost certainly planning work leaking forward.

### Red Flag 2: Broad Truth Reload During Execution

The single biggest performance risk is paying planning cost again during
execution.

Execution and mutation derivation must not repeatedly walk:

- branch delta breadth
- visible source record sets
- schema surfaces
- relation neighborhoods
- lineage context

unless that breadth was explicitly carried forward as an authorized execution
input.

The implementation smell to watch for is any per-record or per-aspect execution
loop that performs "just one more lookup" into broad runtime state.

### Red Flag 3: Storage Identity Quietly Becomes Merge Truth Identity

For 7C, executable reconciliation converges into the existing target record.
That is an execution choice, not a general law of merge truth.

The implementation must not let the 7C storage rule leak into:

- lineage semantics
- diagnostics language
- helper names
- future merge ontology

Target-slot preservation in 7C must remain explicitly milestone-local.

### Red Flag 4: Prepared Artifacts Become Heavy Frozen Data Bundles

Prepared execution artifacts must not silently become large concrete value
bundles.

The implementation must make value carriage explicit through
`MergeValueMaterialization` and count it structurally. In particular:

- eager inline value carriage must be deliberate
- snapshot-pinned value reads must be deliberate
- digest-only equality witnesses must remain available where sufficient

If the prepared artifact begins to carry full copied values for every reconciled
aspect by default, memory, serialization, replay, and recovery costs will drift
upward without the architecture admitting it.

### Red Flag 5: Stage Proliferation Without Enforcement Gain

Multiple proof stages are only justified if each stage makes a concrete illegal
state unrepresentable or materially changes representation for the next phase.

If the code accumulates a chain of `from_*` conversions that mostly reshuffle
the same arrays and metadata, the proof chain has become ceremony.

Each stage must answer:

- what illegal state was possible before this stage?
- why is it impossible after this stage?

If that answer is weak, collapse the stage.

### Red Flag 6: Phase-Honesty Breaks in Diagnostics

Planning/readiness artifacts, execution artifacts, and failure artifacts must
stay distinct.

Signs that phase honesty is drifting:

- successful execution diagnostics contain blocked or rejected rows
- execution diagnostics reconstruct "why" from final state instead of from the
  authority inputs consumed during execution
- replay/debug tooling needs to infer lifecycle phase from mixed artifacts

Diagnostics must be authority-derived, not observationally reconstructed.

### Red Flag 7: Freshness Logic Becomes Either Too Clever or Too Expensive

For 7C, freshness policy is exact and fail-closed. The semantic failure is
trying to weaken that with "close enough" continuation.

The performance failure is implementing exact freshness with broad repeated
work:

- branch-head equality must stay cheap
- merge-base validation must be bounded and instrumented
- schema semantic drift checks must be scoped and instrumented

Do not trade semantic clarity for operational cleverness, and do not trade
operational simplicity for hidden broad scans.

### Red Flag 8: Per-Record Structural Work Replaces Batch-Derived Work

The architecture wants one compiled executable plan and one derived mutation
plan. It does not want a thousand tiny self-sufficient merge interpreters.

Avoid per-record repetition of:

- schema checks
- lineage enrichment
- digest computation
- diagnostics shape construction
- visibility revalidation

When a structural fact is shared across records or aspects, derive it once and
carry it forward.

### Red Flag 9: Digest Rehashing Without Clear Ownership

Canonical digests are required, but repeated rehashing of large artifact bases
across planning, readiness sealing, execution compilation, mutation derivation,
artifact assembly, replay, and certification can become hidden overhead.

Digest ownership must be explicit:

- which phase computes a digest
- which later phases reuse it
- which later phases may refine it
- which later phases are forbidden to recompute the same semantic basis

The architecture should prove once and reuse, not prove the same thing at every
boundary because it feels safer.

## Enforcement Rules Derived From The Red Flags

The build is not allowed to rely on review discipline alone to avoid the red
flags above. The implementation should enforce them mechanically where
possible.

Required enforcement rules:

- execution entrypoints must accept only sealed prepared merge proof types
- blocked or rejected lowered plans must be unconstructable as executable plans
- execution diagnostics types must not have fields for blocked or rejected rows
- value materialization mode must be encoded in the execution plan type rather
  than implied by helper choice
- complexity counters must expose freshness-check breadth, value
  materialization choice, and execution compilation breadth explicitly
- shared digest bases must be carried forward by type, not silently rebuilt by
  helper functions

## Certification Plan

Milestone 7C requires a dedicated certification suite in addition to preserving
existing tests.

New suite:

- `Authoritative merge execution certification test`

Required scenarios:

- exact shared record merge with no-op semantic mutation but real multi-parent
  history publication
- source-only addition merge
- schema-declared correspondence merge with `PreferRicher`
- stale prepared merge rejected after target branch advances
- stale prepared merge rejected after source branch advances
- schema semantic drift rejected after planning
- replay parity for successful merge execution
- durable recovery parity for successful merge execution

Required machine-checkable outputs:

- `merge_execution_digest`
- `merge_execution_diagnostics_digest`
- `merge_execution_truth_digest`
- `merge_execution_replay_digest`
- `merge_execution_recovery_digest`
- `merge_execution_branch_heads_digest`

Existing suites that must continue to pass on real merge-produced histories:

- `Hostile commit/replay equivalence test`
- `Durable recovery and schema mismatch test`
- `Merge-ready history shape test`

## Build Order

The authoritative build order for this milestone is the phase sequence defined
in `Implementation Structure`.

The condensed dependency chain is:

1. Phase A: sealed proof substrate
2. Phase B: freshness and authority binding
3. Phase C: bound executable-plan compilation
4. Phase D: merge-to-mutation derivation
5. Phase E: commit pipeline integration
6. Phase F: diagnostics, counters, and artifact ownership
7. Phase G: certification and hostile-path hardening

Do not start by wiring a host-facing merge API directly into commit history.
That would recreate the exact implicitness Milestone 7B was built to eliminate.

## Completion Standard

Milestone 7C is complete only when all of the following are true:

- no authoritative merge path exists that bypasses prepared lowered plans
- all successful merges publish canonical envelopes with ordered parents and
  merge base metadata
- all stale or unsupported merges fail before authoritative mutation
- replay and durability remain exact on merge-bearing histories
- the merge execution certification suite emits machine-checkable artifacts
- complexity counters prove execution cost at the merge facade boundary

That is the bar. Anything weaker would produce a merge feature, but not a
truth-grade merge authority.
