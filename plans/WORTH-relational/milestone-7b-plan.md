# Milestone 7B Engineering Spec: Merge Artifact Ontology and Reconciliation Semantics

## Summary

Milestone 7B is not a merge-execution milestone.

It is the milestone that makes merge semantics explicit enough that later
execution can be honest.

The runtime must move from:

- "history is merge-ready because ordered parent lists exist"

to:

- "merge semantics are represented as explicit, proof-bearing runtime objects
  with canonical identity matching, causal evidence, conflict classification,
  policy resolution, lowered execution plans, explanation artifacts, and
  machine-checkable sameness contracts"

This milestone is not "add a richer `inspect_merge()`."

It is:

- merge-domain structural split
- causal metadata introduction at merge-planning granularity
- schema-declared merge policy introduction
- identity matching and reconciliation ontology
- conflict taxonomy completion
- lowered merge-plan completion
- merge explanation artifact completion
- certification-grade serialization, digest, and replay/durability parity for
  merge-planning artifacts

No authoritative merged truth is committed in this milestone. That is
Milestone 7C. Milestone 7B exists so that 7C is forced to execute against
canonical merge ontology rather than rediscovering merge meaning procedurally
inside the authority hot path.

## Governing Rules

The rules for this milestone are:

- ordered parent lists remain history truth, not merge truth
- merge semantics must be represented as first-class typed artifacts, not as
  comments around helper methods
- causal reasoning, identity matching, conflict classification, and policy
  resolution are distinct phases and must not be collapsed into one "merge
  check" blob
- every phase must produce a proof-bearing type that the next phase consumes
- every important merge decision must be reconstructable from a canonical
  artifact, not from re-running merge logic against live runtime state
- schema policy resolution must be explicit and recorded; policy may not be a
  closure passed at call time
- branch-scoped and direction-scoped merge reasoning must be explicit in the
  request type
- later phases may enrich explanation and metrics, but may not reinterpret the
  legality or semantic class of earlier decisions
- all merge cost claims must be visible at named boundaries with structural
  counters

The most important discipline is Principle 41 from the coding guidelines:

- a merge-planning type must encode what has been proven, not merely what data
  it contains
- a later merge phase must not be constructible from weaker earlier data
- a runtime check for a property already guaranteed by the type is dead code
- if a type claims "policy resolved," "causally annotated," or "execution
  lowered," but external code can synthesize that type directly, the type is a
  lie

## Phase Split Retention Criteria

`MENTALITY.md` requires future-proofing for foundations, but it does not
justify decorative wrappers.

For this milestone, a phase split is justified only if all three are true:

- the later type carries a distinct new proof, not merely the same data in a
  new wrapper
- that proof is expected to remain load-bearing under future merge,
  reconciliation, and policy work
- later phases should be structurally unable to proceed without that proof

If any adjacent pair cannot satisfy those conditions in the implementation, the
phases should be collapsed.

This rule exists to balance two failure modes:

- under-structuring: future load-bearing distinctions are collapsed too early
- over-structuring: wrappers exist on paper but do not correspond to durable
  semantic boundaries

Milestone 7B therefore keeps:

- conflict classification separate from policy resolution
- causal annotation separate from policy resolution
- policy resolution separate from lowering

Milestone 7B deliberately does **not** require a separate
candidate-discovery wrapper before validated identity output unless the
implementation can prove a distinct load-bearing boundary between candidate
discovery and validated mapping.

## Architecture Goal

At closeout, merge planning must have the same structural honesty standard as
lineage promotion:

- one explicit request boundary
- one explicit ancestry and branch-scope boundary
- one explicit identity-matching boundary
- one explicit causal-annotation boundary
- one explicit conflict-classification boundary
- one explicit policy-resolution boundary
- one explicit lowered-plan boundary
- one canonical merge-planning artifact
- all summaries, explanations, serialization, replay parity, and certification
  digests derived from that artifact

No later consumer may reconstruct merge meaning by rescanning raw branch heads,
raw patch deltas, or raw lineage state.

## What Exists Today and What It Really Means

The current codebase already has several useful substrates, but they are not
yet the merge ontology.

### Existing substrates that remain valid

- [`CommitReference.parents`](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/history/data/mod.rs)
  is the authoritative ordered-parent history surface
- [`HistoryAccess::inspect_merge(...)`](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/history/logic/access.rs)
  is a useful history/overlap inspection helper
- lineage authority phase types in
  [`phase_types.rs`](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/lineage/logic/authority/phase_types.rs)
  are a strong precedent for proof-bearing lifecycle design
- aspect declarations and lowered aspect plans in
  [`aspect_semantics.rs`](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/schema/data/aspect_semantics.rs)
  prove the repo already accepts declaration-surface to lowered-plan
  transitions as an architectural pattern

### Existing surfaces that must not be mistaken for merge ontology

- `HistoryAccess::inspect_merge(...)` currently means:
  - choose a merge-base using the runtime's current ancestor-selection rule
  - compute branch-unique commit closures
  - classify conflicts as overlapping touched record identity

It does **not** mean:

- identity matching across different record ids
- aspect-aware reconciliation
- causal dependency classification
- schema-declared merge policy resolution
- relation endpoint rewiring semantics
- deletion-as-merge-result semantics
- execution-lowered merge planning

- `merge_parent_branches` in commit/publication/replay surfaces is contextual
  provenance only. It is not merge semantic authority.

- `MergedCommitPlan` in transactions is a same-transaction mutation-intent
  normalization artifact. It is not a branch merge plan.

### Existing naming debt that must be corrected first

The current `src/authority/merge/` module is about mutation-intent merge inside
one transaction, not branch merge semantics.

Milestone 7B must make this explicit.

Required correction:

- rename or structurally relocate the current `authority/merge/` subsystem into
  a responsibility-honest name such as:
  - `authority/intent_merge/`
  - `authority/mutation_plan_merge/`
  - `authority/commit_plan_normalization/`

This must happen before the new branch-merge ontology lands. Keeping both
under the same term would create semantic collapse and almost guarantee wrong
future design choices.

Implementation note:

- this rename is complete in production code as `src/authority/intent_merge/`
- no path alias or compatibility shim should remain because the project has no
  production backward-compatibility obligation

## Non-Goals

This milestone does not do the following:

- commit authoritative merged truth
- advance branch heads through successful merge commits
- mutate storage through merge execution
- publish merge commits into CDC as accepted truth
- resolve merge requests with ad hoc host closures
- collapse lineage identity, storage identity, and structural identity into one
  matching bucket
- infer merge policy from aspect names or payload shape
- pretend the current `latest_common_ancestor_between_branches(...)` helper is
  already the final merge-base ontology
- admit partially merged authoritative truth within one merge request scope
- hardcode web/resource semantics, chip/net semantics, or geometry/topology
  semantics into the generic merge core

## Domain Neutrality Rules

This library must remain generic enough for chip simulators, geometry kernels,
web/data systems, and domains not yet anticipated.

Milestone 7B must therefore preserve domain neutrality by enforcing:

- identity bases are declared or runtime-provided as typed surfaces, not
  hardcoded for one domain vocabulary
- merge semantics are aspect-driven and relation-aware, not blob-diff-driven
- policies are schema-governed, not host-closure-governed
- record truth and relation truth are equal citizens in the ontology
- canonical merge artifacts explain runtime decisions in typed terms, not
  domain-specific prose
- no merge rule assumes that storage identity, display name, payload field
  label, or UI/resource semantics are the default identity model

This means the runtime may offer multiple identity substrates and multiple
policy families, but it may not silently privilege one domain's intuitions as
the generic baseline.

## Required Production Structure

Milestone 7B must begin with a structural split. The implementation may not
accumulate into `history`, `transactions`, or broad "merge helper" files.

### Production structure

- `src/merge/`
- `facade.rs`
- facade only

- `src/merge/data/`
- `requests.rs`
  - `MergePlanningRequest`
  - `MergeIntent`
- `ancestry.rs`
  - `MergeBaseSelectionRule`
  - `ResolvedMergeBase`
  - `BranchCommitDelta`
- `identity.rs`
  - `MergeRecordIdentity`
  - `IdentityMatchCandidate`
  - `IdentityMatchClass`
  - `IdentityResolutionReason`
- `causal.rs`
  - `BranchCausalDot`
  - `CausalFrontier`
  - `CommitCausalMetadata`
  - `MergeCausalRelation`
  - `MergeCausalEvidence`
- `conflicts.rs`
  - `MergeConflictClass`
  - `MergeConflictRecord`
  - `MergeConflictReason`
  - `ConflictLocality`
- `policy.rs`
  - `AspectMergePolicyDeclaration`
  - `AspectMergePolicyKind`
  - `LoweredAspectMergePolicy`
  - `MergePolicyResolution`
- `plans.rs`
  - `LoweredMergePlan`
  - `LoweredMergeTargetRecordPlan`
  - `LoweredMergeRelationPlan`
- `artifacts.rs`
  - `MergePlanningArtifactCore`
  - `MergeDecisionLog`
  - `MergeArtifactDigestBasis`
- `explanations.rs`
  - `MergeExplanationRecord`
  - `MergeExplanationSurface`
- `metrics.rs`
  - merge counters and boundary summaries
- `mod.rs`
  - wiring only

- `src/merge/logic/access/`
- `request_validation.rs`
- `ancestry_resolution.rs`
- `identity_matching.rs`
- `causal_annotation.rs`
- `conflict_classification.rs`
- `policy_resolution.rs`
- `plan_lowering.rs`
- `explanation.rs`
- `serialization.rs`
- `mod.rs`

- `src/schema/data/merge_semantics.rs`
- per-aspect merge policy declarations, identity-basis declarations, and
  custom merge policy descriptor types

- `src/schema/logic/merge_policies.rs`
- schema declaration lowering into executable merge-policy catalogs

- `src/history/data/causal.rs`
- commit-causal metadata shapes that belong to history truth rather than only
  merge logic

- `src/history/logic/causal.rs`
- causal comparison and frontier derivation over the version graph

### Test structure

- `src/tests/merge/`
- `request_validation.rs`
- `ancestry_resolution.rs`
- `identity_matching.rs`
- `causal_annotation.rs`
- `conflict_classification.rs`
- `policy_resolution.rs`
- `plan_lowering.rs`
- `artifact_roundtrip.rs`
- `certification.rs`
- `mod.rs`

- `tests/ui/merge/`
- compile-fail coverage for:
  - sealed constructors
  - invalid policy use without causal capability
  - execution-lowered types not constructible externally
  - direction-less or branch-less merge request misuse

### Structural rules

- `mod.rs` files are wiring-only
- `facade.rs` is public API only
- no "merge_manager," "merge_helpers," "merge_utils," or mixed
  "merge_contracts" file is allowed
- each file must map to one phase responsibility or one data concept
- no new merge behavior may be added to `history/logic/access.rs` beyond
  narrow compatibility shims once the merge subsystem exists

## Canonical Domain Vocabulary

Milestone 7B must make several implicit assumptions explicit by naming them as
real runtime concepts.

### 1. Merge request direction is explicit

Today it is easy to speak loosely about "merging branch A into branch B." The
runtime must not carry this as conversational meaning.

Required type:

```rust
pub struct MergePlanningRequest {
    target_branch: BranchId,
    source_branch: BranchId,
    merge_intent: MergeIntent,
}
```

`MergePlanningRequest` must not be constructible from unordered branch pairs.
The direction is encoded by the typed `source_branch` and `target_branch`
fields themselves.

### 2. Merge base selection rule is explicit

Today the runtime uses `max_commit_id_common_ancestor`. That is a real 7A
history fact. It is not enough to say "merge base."

Required type:

```rust
pub enum MergeBaseSelectionRule {
    MaxCommitIdCommonAncestor,
}
```

```rust
pub struct ResolvedMergeBase {
    rule: MergeBaseSelectionRule,
    commit_id: CommitId,
    supporting_left_ancestors: Arc<[CommitId]>,
    supporting_right_ancestors: Arc<[CommitId]>,
}
```

This makes explicit which algorithm was used and what evidence supports the
selection.

### 3. Storage identity, lineage identity, and structural identity are separate

Current merge inspection treats overlap mostly as record identity overlap. That
is not sufficient for CAD, chip, or schema-rich application workloads.

Required type:

```rust
pub enum MergeRecordIdentity {
    StorageRecord(RecordRef),
    Lineage(LineageId),
    StructuralFingerprint(StructuralFingerprint),
}
```

The merge system must never return a single untyped "record identity." The
identity basis is itself part of the semantic claim.

### 3.5 Schema-declared identity basis is explicit

Merge identity cannot remain purely heuristic if the runtime is meant to be
domain-agnostic.

The schema must be able to declare which identity surfaces are legitimate for
correspondence and reconciliation.

Required types:

```rust
pub struct IdentityBasisDeclaration {
    pub scope: IdentityBasisScope,
    pub basis: IdentityBasisKind,
}
```

```rust
pub enum IdentityBasisScope {
    EntityKind(KindId),
    RelationKind(KindId),
    AspectKey(AspectKey),
}
```

```rust
pub enum IdentityBasisKind {
    StorageIdentity,
    LineageIdentity,
    StructuralFingerprint,
    DeclaredKeySet(Arc<[AspectKey]>),
    Custom(CustomIdentityBasisIdentity),
}
```

Rules:

- storage identity is never assumed sufficient just because it exists
- display names, UI labels, or arbitrary payload fields are not identity
  unless declared as such
- custom identity bases must be semantically versioned and schema-declared
- every accepted source/target correspondence must record which declared
  identity basis justified it

### 4. Conflict classification and policy resolution are distinct

A conflict is a semantic statement about divergence. Policy resolution is a
statement about whether that divergence is resolvable under declared rules.

Required types:

```rust
pub enum MergeConflictClass {
    NoCorrespondingTarget,
    StructuralConflict,
    AspectConflict,
    RelationTopologyConflict,
    DeletionConflict,
    DeletionVsModificationConflict,
    DeletionVsRewireConflict,
    IdentityAmbiguity,
    SchemaConflict,
    PolicyUnavailable,
    CausalOrderRequired,
}
```

```rust
pub enum MergePolicyResolution {
    AutoResolved,
    RequiresManualResolution,
    Reject,
}
```

No single enum may merge these two ontologies together.

### 4.5 Deletion and removal semantics are explicit

Deletion and removal are not just ordinary absence. They are merge-relevant
semantic states.

Required types:

```rust
pub enum DeletionMergeClass {
    DeletedOnSourceOnly,
    DeletedOnTargetOnly,
    DeletedOnBoth,
    DeletedVsModified,
    DeletedVsRewired,
    DeletedVsRicherStructure,
}
```

Rules:

- deletion must be compared three-way against the merge base
- deletion must be classifiable before policy resolution
- deletion may participate in policy resolution, but only after it has been
  classified as a deletion-specific merge condition
- a missing record must not be treated as generic "no corresponding target"
  once the runtime has enough evidence to classify it as deletion-relative

### 5. Richer-aspect reconciliation is explicit

The roadmap already calls this out. It must become a typed concept now, not an
explanation string later.

Required reason type:

```rust
pub enum IdentityResolutionReason {
    ExactStorageIdentity,
    ExactLineageIdentity,
    ExactStructuralFingerprint,
    PreferRicherAspectShape,
    SchemaDeclaredCorrespondence,
}
```

If the runtime claims "this source record maps onto that target record because
the source carries a richer declared aspect structure," that reason must appear
as a first-class typed decision, not as a prose note.

## Type-System Foundation

Milestone 7B must encode merge planning as a proof-widening chain.

The exact names may vary, but the phase structure must not.

Merge planning in this milestone is explicitly three-way.

Every semantic decision must be grounded in:

- base truth at the selected merge base
- source branch truth
- target branch truth

This applies not only to record state, but also to:

- aspect deltas
- relation topology
- deletion/removal semantics

The implementation must not collapse into a two-way "source vs target" diff
that merely consults the merge base opportunistically.

### Required phase chain

The implementation currently uses:

- one explicit request boundary
- six internal proof-bearing plan types
- one canonical artifact closure

That is the current honest runtime shape for 7B. The milestone should be
described as an eight-stage planning chain, not as a nine-phase chain.

1. `RequestedMergePlan`
- request is syntactically valid
- branch ordering and direction are explicit
- no ancestry or policy assumptions yet

2. `HistoryScopedMergePlan`
- source and target heads exist
- merge-base selection rule and result are explicit
- branch-local commit deltas are explicit
- still no identity matching or causal claims

3. `ValidatedIdentityMatchedMergePlan`
- corresponding record candidates have been identified
- each candidate carries explicit identity basis and reason
- identity ambiguity rejected or partitioned into explicit conflict classes
- source/target mapping is shape-valid
- relation endpoint candidate mapping is coherent

4. `CausallyAnnotatedMergePlan`
- branch commits and candidate record changes carry explicit causal evidence
- concurrency vs ancestry is explicit
- still no policy resolution

5. `ConflictClassifiedMergePlan`
- every candidate divergence is classified into a typed conflict taxonomy
- no unresolved "other conflict" bucket is allowed

6. `PolicyResolvedMergePlan`
- schema-declared merge policies have been applied to each resolvable conflict
- non-resolvable conflicts remain typed and explicit
- no execution lowering yet

7. `LoweredMergePlan`
- monomorphic execution input
- exact target record actions are fixed
- exact relation endpoint rewires are fixed
- exact deletion/adoption/reconciliation semantics are fixed
- this is the strongest pre-execution form

8. `MergePlanningArtifactCore`
- canonical serializable artifact derived from `LoweredMergePlan`
- carries decision log, digest basis, and core metrics
- this is the canonical merge-ontology artifact for 7B

Implementation note:

- the current internal phase types are `HistoryScopedMergePlan`,
  `IdentityScopedMergePlan`, `ConflictClassifiedMergePlan`,
  `CausallyAnnotatedMergePlan`, `PolicyResolvedMergePlan`, and
  `LoweredMergePlan`
- `RequestedMergePlan` is represented by the validated
  `MergePlanningRequest` boundary rather than a dedicated wrapper type

### Constructors and visibility rules

- later-phase constructors are `pub(crate)` and owned only by their phase
  modules
- no blanket `From` or `Into` conversions from raw collections into later
  phase types
- no external caller may construct `ValidatedIdentityMatchedMergePlan`,
  `CausallyAnnotatedMergePlan`, `PolicyResolvedMergePlan`, `LoweredMergePlan`,
  or `MergePlanningArtifactCore`
- accessor methods expose read-only views only
- fields on later-phase wrappers remain private

### Compile-time verification

UI tests must prove:

- external code cannot construct `LoweredMergePlan`
- external code cannot skip from request type to policy-resolved type
- merge planning functions cannot accept unordered branch pairs
- `LastWriterWins` or any policy requiring causal evidence cannot be lowered
  without the causal-capability proof type

## Required Phase Types

The following types define the engineering contract. Exact field spelling may
change, but the semantic ownership must remain.

### Phase 1: request

```rust
pub struct MergePlanningRequest {
    target_branch: BranchId,
    source_branch: BranchId,
    merge_intent: MergeIntent,
}
```

```rust
pub enum MergeIntent {
    ReconcileIntoTarget,
}
```

`MergePlanningRequest` means only:

- the caller wants the runtime to compute canonical merge semantics for a
  directed source/target branch pair

It does not mean:

- merge-base resolved
- mergeable
- policy-supported
- execution-ready

### Phase 2: history-scoped

```rust
pub struct HistoryScopedMergePlan {
    request: MergePlanningRequest,
    target_head: CommitReference,
    source_head: CommitReference,
    merge_base: ResolvedMergeBase,
    target_delta: BranchCommitDelta,
    source_delta: BranchCommitDelta,
}
```

```rust
pub struct BranchCommitDelta {
    branch_id: BranchId,
    commits: Arc<[CommitId]>,
    touched_records: Arc<[MergeConflictRecord]>,
}
```

This phase makes explicit the current hidden assumptions in `inspect_merge()`:

- which branch heads were used
- which merge-base algorithm was used
- which commit deltas are being compared

### Phase 3: validated identity match

```rust
pub struct ValidatedIdentityMatchedMergePlan {
    history: HistoryScopedMergePlan,
    discovered_candidates: Arc<[IdentityMatchCandidate]>,
    validated_candidates: Arc<[ValidatedIdentityMatch]>,
    rejected_candidates: Arc<[RejectedIdentityMatch]>,
}
```

```rust
pub struct IdentityMatchCandidate {
    source: MergeRecordIdentity,
    target: MergeRecordIdentity,
    match_class: IdentityMatchClass,
    reason: IdentityResolutionReason,
}
```

```rust
pub enum IdentityMatchClass {
    Exact,
    Reconciliable,
    Ambiguous,
    MissingTarget,
}
```

```rust
pub struct ValidatedIdentityMatch {
    source: MergeRecordIdentity,
    target: MergeRecordIdentity,
    mapping_kind: MergeMappingKind,
}
```

```rust
pub enum MergeMappingKind {
    OneToOne,
    SourceDeletionAgainstTarget,
    SourceIntroductionIntoTarget,
    RelationRewireCandidate,
}
```

```rust
pub struct RejectedIdentityMatch {
    source: MergeRecordIdentity,
    reason: MergeConflictClass,
}
```

This phase must reject:

- one source mapping onto multiple targets
- one target claiming multiple incompatible sources
- relation endpoint candidate sets that cannot be made coherent

This phase is where the runtime stops pretending that "same record" means
"same storage id."

It is also where three-way identity interpretation begins:

- what existed at base but diverged into distinct source/target continuations
- what was introduced independently after base
- what was deleted on one side and evolved on the other

### Phase 4: causal annotation

```rust
pub struct CausallyAnnotatedMergePlan {
    validated: ValidatedIdentityMatchedMergePlan,
    source_causal: CommitCausalMetadata,
    target_causal: CommitCausalMetadata,
    candidate_evidence: Arc<[MergeCausalEvidence]>,
}
```

```rust
pub struct CommitCausalMetadata {
    observed_frontier: CausalFrontier,
    produced_dot: BranchCausalDot,
    concurrent_frontier: Arc<[BranchCausalDot]>,
}
```

```rust
pub struct MergeCausalEvidence {
    candidate_id: MergeCandidateId,
    relation: MergeCausalRelation,
    supporting_frontier: CausalFrontier,
}
```

```rust
pub enum MergeCausalRelation {
    SourceBeforeTarget,
    TargetBeforeSource,
    Concurrent,
    Equal,
}
```

This phase makes these currently implicit assumptions explicit:

- whether divergence is concurrent or causally ordered
- whether a policy requiring causal order is even meaningful
- which frontier the runtime is talking about when it explains "why this wins"

### Phase 5: conflict classification

```rust
pub struct ConflictClassifiedMergePlan {
    causal: CausallyAnnotatedMergePlan,
    classified_records: Arc<[ClassifiedMergeRecord]>,
}
```

```rust
pub struct ClassifiedMergeRecord {
    candidate_id: MergeCandidateId,
    source: MergeRecordIdentity,
    target: Option<MergeRecordIdentity>,
    conflict_class: MergeConflictClass,
    locality: ConflictLocality,
}
```

```rust
pub enum ConflictLocality {
    RecordLocal,
    RelationEndpointLocal,
    AspectLocal,
    SchemaLocal,
    CrossRecordStructural,
}
```

No residual "unclassified merge item" bucket is allowed after this phase.

### Phase 6: policy resolution

```rust
pub struct PolicyResolvedMergePlan {
    classified: ConflictClassifiedMergePlan,
    resolved_records: Arc<[PolicyResolvedMergeRecord]>,
}
```

```rust
pub struct PolicyResolvedMergeRecord {
    candidate_id: MergeCandidateId,
    conflict_class: MergeConflictClass,
    policy: LoweredAspectMergePolicy,
    resolution: MergePolicyResolution,
    resolved_action: Option<ResolvedMergeAction>,
}
```

```rust
pub enum ResolvedMergeAction {
    AdoptSourceAspectDelta,
    PreserveTargetAspectDelta,
    SumCounterDelta,
    UnionObservedRemoveSet,
    PreferRicherShape,
    PreserveDeletion,
    RewireRelationEndpoints,
}
```

This phase must never be implemented as "if/else over enum plus closure." The
policy and the resolution must both be canonical typed values.

### Phase 7: lowering

```rust
pub struct LoweredMergePlan {
    request: MergePlanningRequest,
    merge_base: ResolvedMergeBase,
    source_parent: CommitReference,
    target_parent: CommitReference,
    target_record_plans: Arc<[LoweredMergeTargetRecordPlan]>,
    relation_plans: Arc<[LoweredMergeRelationPlan]>,
    unresolved_conflicts: Arc<[ClassifiedMergeRecord]>,
}
```

```rust
pub struct LoweredMergeTargetRecordPlan {
    target_identity: MergeRecordIdentity,
    record_action: LoweredRecordMergeAction,
}
```

```rust
pub enum LoweredRecordMergeAction {
    KeepTarget,
    IntroduceSourceAsNewRecord,
    ReconcileIntoExistingTarget,
    RetireTargetByMergedDeletion,
}
```

```rust
pub struct LoweredMergeRelationPlan {
    relation_identity: MergeRecordIdentity,
    relation_action: LoweredRelationMergeAction,
}
```

```rust
pub enum LoweredRelationMergeAction {
    KeepRelation,
    IntroduceRelation,
    RewireEndpoints,
    RetireRelation,
}
```

`LoweredMergePlan` is the only shape 7C is allowed to execute.

## Schema-Declared Merge Policies

Milestone 7B must introduce merge policy declarations as a schema concern,
parallel to aspect declarations and invariant declarations.

### Declaration surface

Required schema type:

```rust
pub struct AspectMergePolicyDeclaration {
    pub aspect_key: AspectKey,
    pub policy: AspectMergePolicyKind,
}
```

```rust
pub enum AspectMergePolicyKind {
    FailOnConflict,
    LastWriterWins,
    MonotonicCounter,
    AdditiveSet,
    PreferRicher,
    Custom(CustomMergePolicyIdentity),
}
```

`CustomMergePolicyIdentity` must be semantically versioned exactly the way
custom invariant identity is semantically versioned. Merge policy identity is
semantic identity, not runtime closure identity.

Implementation note:

- the current 7B implementation admits only `FailOnConflict` and
  `PreferRicher`
- `LastWriterWins`, `MonotonicCounter`, `AdditiveSet`, and `Custom(_)` are not
  yet supported in the planning runtime and must be rejected at schema
  registration time rather than silently degrading to manual resolution

### Lowered policy surface

Required lowered type:

```rust
pub struct LoweredAspectMergePolicy {
    pub aspect_key: AspectKey,
    pub policy: LoweredAspectMergePolicyKind,
}
```

```rust
pub enum LoweredAspectMergePolicyKind {
    FailOnConflict,
    LastWriterWins(CausalCapabilityProof),
    MonotonicCounter,
    AdditiveSet,
    PreferRicher,
    Custom(LoweredCustomMergePolicy),
}
```

`CausalCapabilityProof` is not a bool. It is a proof-bearing type created only
by the causal-annotation phase. If the runtime has not proven causal evidence
for the request, the `LastWriterWins` lowered form must be unconstructible.

### Policy lowering rules

- policy declarations lower once per schema registry snapshot
- merge planning consumes lowered policy catalogs, not raw declarations
- a merge request may not invent or override policy at call time
- custom merge policy registration must be frozen at runtime construction
- policy lowering must record declaration revision and semantic identity

### Schema evolution and reconciliation rules

Merge policy declarations are schema declarations and must participate in the
same truth-grade schema transition machinery as other load-bearing schema
surfaces.

Required rules:

- adding a merge policy to a previously policy-less aspect is a schema
  transition and must appear in schema transition artifacts
- changing a merge policy between schema revisions is a schema transition and
  must be classified explicitly during schema reconciliation
- replay and durable recovery must compare the effective merge-policy
  declaration snapshot, not merely aspect presence
- schema reconciliation descriptors must classify merge-policy divergence using
  the same explicit compatibility/rejection rules as other schema semantics

## Causal Metadata

Milestone 7B introduces causal metadata as a first-class merge-planning
concept and as a history-adjacent artifact family.

In the initial 7B implementation, causal evidence is branch-history-derived
merge causality over the existing version DAG, not a general distributed
causality model.

### Required causal types

```rust
pub struct BranchCausalDot {
    pub branch_id: BranchId,
    pub commit_id: CommitId,
}
```

```rust
pub struct CausalFrontier {
    pub dots: Arc<[BranchCausalDot]>,
}
```

```rust
pub enum CommitCausalRelation {
    Before,
    After,
    Equal,
    Concurrent,
}
```

### Required causal rules

- causal comparison must use version-graph history semantics, not wall-clock
  time
- the runtime must define one canonical ordering for frontier serialization
- causal metadata must be digestable and round-trippable
- merge explanation artifacts must cite causal evidence by typed relation, not
  prose

### Honest limitation rule

The first 7B implementation may use branch/head ancestry over the existing
history DAG as the causal substrate. If it does, that limitation must be named
in the complexity contracts and marked as debt where appropriate.

What is forbidden is pretending that a weaker ancestry-derived causal model is
already a fully general distributed vector-clock semantics if it is not.

## Identity Matching and Reconciliation Ontology

The merge planner must not jump directly from branch deltas to "these
conflicts exist." It must first answer what identity relationship is being
claimed.

### Required candidate classes

```rust
pub enum IdentityResolutionReason {
    ExactStorageIdentity,
    ExactLineageIdentity,
    ExactStructuralFingerprint,
    SchemaDeclaredCorrespondence,
    PreferRicherAspectShape,
    AdvisoryCorrespondenceRejected,
}
```

This explicitly separates:

- true identity continuity
- schema-declared correspondence
- richer-structure reconciliation
- rejected advisory correspondence

### Required anti-collapse rules

- advisory correspondence must never silently become merge identity authority
- lineage identity and merge identity are related but distinct concepts
- structural fingerprint matching must remain a distinct basis, not a fallback
  string field
- richer-aspect reconciliation must not be encoded as fake storage continuity

Implementation note:

- the current 7B implementation admits `StorageIdentity`, `LineageIdentity`,
  and `DeclaredKeySet(...)`
- `StructuralFingerprint` and `Custom(...)` identity bases are not yet
  supported in merge planning and must be rejected at schema registration time
  rather than counted as live capability

## Relation Semantics

Relation merge semantics are load-bearing and must be modeled with the same
seriousness as record merge semantics.

### Required relation distinctions

The ontology must explicitly distinguish:

- endpoint identity continuity
- relation continuity identity
- rewired-but-continuous relation semantics
- retired-and-reintroduced relation semantics
- endpoint ambiguity as an input to relation classification
- relation-local conflict vs cross-record structural conflict

Required types:

```rust
pub enum RelationContinuityClass {
    PreserveRelationIdentity,
    RetireAndIntroduceSuccessor,
}
```

```rust
pub enum EndpointContinuityClass {
    EndpointsStable,
    SourceEndpointRewired,
    TargetEndpointRewired,
    BothEndpointsRewired,
}
```

```rust
pub struct RelationMergeCandidate {
    relation_identity: MergeRecordIdentity,
    endpoint_continuity: EndpointContinuityClass,
    relation_continuity: RelationContinuityClass,
}
```

```rust
pub enum RelationIdentityBasis {
    StorageRelationIdentity,
    LineageRelationIdentity,
    StructuralRelationFingerprint,
    DeclaredRelationKeySet(Arc<[AspectKey]>),
}
```

```rust
pub enum RelationConflictPropagation {
    RelationLocalOnly,
    EscalatesToRecordConflict,
    EscalatesToTopologyRegionConflict,
}
```

### Required relation rules

- rewiring endpoints does not automatically imply relation identity continuity
- preserving relation identity across endpoint rewiring must be an explicit
  typed decision
- endpoint ambiguity must be able to poison relation reconciliation rather than
  being silently ignored
- relation-local topology conflicts and cross-record topology conflicts must be
  classified separately even if both eventually block resolution
- relation identity basis must be explicit for accepted relation mappings
- relation conflicts must be able to propagate outward into record or topology
  classification when the ontology requires it
- three-way relation reconciliation must compare:
  - base endpoints
  - source endpoints
  - target endpoints
  - relation-local aspect deltas

## Canonical Merge Artifact Rule

There must be exactly one canonical merge-planning artifact in this milestone:

- `MergePlanningArtifact`

Everything else derives from it.

### Required contents

```rust
pub struct MergePlanningArtifactCore {
    pub request: MergePlanningRequest,
    pub merge_base: ResolvedMergeBase,
    pub lowered_plan: LoweredMergePlan,
    pub decision_log: MergeDecisionLog,
    pub metrics: MergePlanningMetrics,
    pub digest_basis: MergeArtifactDigestBasis,
}
```

```rust
pub struct MergePlanningArtifact {
    pub core: MergePlanningArtifactCore,
    pub attached_views: MergePlanningAttachedViews,
}
```

```rust
pub struct MergePlanningAttachedViews {
    pub explanation_surface: MergeExplanationSurface,
}
```

### Decision-log contract

Required decision record:

```rust
pub struct MergeDecisionRecord {
    pub candidate_id: MergeCandidateId,
    pub phase: MergePlanningPhase,
    pub source: MergeRecordIdentity,
    pub target: Option<MergeRecordIdentity>,
    pub causal_relation: Option<MergeCausalRelation>,
    pub conflict_class: Option<MergeConflictClass>,
    pub policy_resolution: Option<MergePolicyResolution>,
    pub action: Option<ResolvedMergeAction>,
}
```

The decision log must be sufficient to answer:

- why two records were considered corresponding
- why a record was classified as conflicting
- whether the conflict depended on causal evidence
- which schema policy resolved it, if any
- why a richer structure was adopted or rejected

If a future consumer cannot answer those questions from the canonical artifact,
the milestone is incomplete.

The lowered plan remains the sole actionable semantic authority.

The decision log is explanatory and evidentiary only. It may explain why the
plan exists, but it must never become a parallel actionable representation of
merge semantics.

## Partial-Conflict Deferral and Locality Preservation

Partial-conflict acceptance is not part of Milestone 7B.

This milestone plans merge semantics over the full request scope. If the plan
contains unresolved conflicts, the request is not execution-ready for 7C.

However, the ontology must preserve enough locality information that a later
milestone can introduce region-scoped or locality-scoped partial merge
admission without redesigning the core taxonomy.

Required preservation rules:

- conflict records must carry locality classification
- relation conflicts must be able to escalate to region/topology-local conflict
  classes
- lowered plans must distinguish resolved actions from unresolved conflicts
  without erasing locality
- no current type may imply that "full-request conflict" is the only possible
  future admission model

### Derived surfaces

The following must derive from `MergePlanningArtifactCore`:

- explanation reports
- serialization roundtrip parity
- certification digests
- preview/inspection APIs
- future 7C execution input

No later consumer may reconstruct merge meaning by rescanning history and
re-running identity matching heuristics.

## Public API Design

Milestone 7B adds a read/planning surface, not authority execution.

### Required public facade

`RelationalRuntime` should expose:

```rust
pub fn merge_access(&self) -> MergeAccess<'_>
```

### Required public APIs

```rust
pub fn plan_merge(
    &self,
    request: MergePlanningRequest,
) -> Result<MergePlanningArtifact, MergePlanningError>;
```

```rust
pub fn explain_merge(
    &self,
    request: MergePlanningRequest,
) -> Result<MergeExplanationSurface, MergePlanningError>;
```

### Signature rules

- no API may accept only `source_branch` and `target_branch` without a typed
  request object
- no API may return only a bool or a string explanation
- no API may shape merge planning as a cheap accessor
- result types must carry metrics and explicit planning phase outputs

## Complexity Contracts and Measurement Boundaries

Merge planning must be cost-visible from the start.

### Required boundaries

- request validation boundary
- ancestry resolution boundary
- identity matching boundary
- causal annotation boundary
- conflict classification boundary
- policy resolution boundary
- lowered-plan artifact boundary

### Required counters

At minimum:

- `merge_request_validation_count`
- `merge_history_nodes_visited`
- `merge_history_parent_checks`
- `merge_candidate_match_count`
- `merge_candidate_ambiguity_count`
- `merge_lineage_match_count`
- `merge_structural_fingerprint_match_count`
- `merge_causal_comparison_count`
- `merge_concurrent_candidate_count`
- `merge_conflict_classification_count`
- `merge_policy_resolution_count`
- `merge_auto_resolved_count`
- `merge_unresolved_conflict_count`
- `merge_relation_rewire_candidate_count`
- `merge_richer_shape_resolution_count`
- `merge_planning_elapsed_nanos`

### Required complexity contracts

Add named contracts for:

- `merge::ancestry::merge_base_resolution`
- `merge::identity::candidate_matching`
- `merge::causal::branch_delta_causal_annotation`
- `merge::classification::conflict_taxonomy_assignment`
- `merge::policy::schema_declared_policy_resolution`
- `merge::lowering::execution_plan_lowering`

Each contract must declare:

- exact complexity statement
- boundedness basis
- counters used to justify the claim
- verified vs debt status
- proof test name

### Required honesty rules

- if identity matching uses whole-branch scans in the current implementation,
  the contract must say so and mark debt
- if causal comparison is ancestry-derived rather than vector-clock complete,
  the contract must say so and mark debt where appropriate
- if structural fingerprint matching requires broad scan fallback, that fallback
  must be explicit and counted

`merge_planning_elapsed_nanos` is a diagnostic boundary metric, not the proof
basis for complexity claims. Structural counters remain the authoritative cost
model; elapsed time exists to help distinguish where the measured cost is being
paid.

## Phase-by-Phase Implementation Plan

This milestone should be built in ordered phases. Each phase lands only after
its types, tests, and counters exist.

### Phase 1: structural split and naming correction

Goal:

- create the new `merge/` subsystem
- rename the current `authority/merge/` intent-normalization subsystem to an
  honest name
- prevent semantic collision before new branch-merge work lands

Close condition:

- no remaining production code uses `authority/merge` to mean two unrelated
  things

### Phase 2: request, ancestry, and causal vocabulary

Goal:

- land `MergePlanningRequest`, `MergeBaseSelectionRule`,
  `ResolvedMergeBase`, `BranchCausalDot`, `CausalFrontier`, and
  `CommitCausalMetadata`
- land schema-declared identity-basis types

Close condition:

- merge planning requests, merge-base selection, and causal claims are all
  represented as first-class types

### Phase 3: identity matching ontology

Goal:

- land `MergeRecordIdentity`, `IdentityMatchCandidate`,
  `IdentityResolutionReason`, `IdentityMatchClass`, and
  `ValidatedIdentityMatchedMergePlan`
- land deletion-classification and three-way identity interpretation rules

Close condition:

- no merge path relies on raw `RecordRef` overlap alone as the only identity
  basis

### Phase 4: schema policy declarations and lowering

Goal:

- add merge-policy declarations to schema
- lower them into `LoweredAspectMergePolicy`
- enforce causal-capability requirements mechanically
- define schema-transition and reconciliation behavior for merge-policy changes

Close condition:

- the runtime can state exactly which policy applies to which aspect, and the
  lowered policy forms are sealed

### Phase 5: conflict classification and policy resolution

Goal:

- implement typed conflict taxonomy and explicit policy resolution
- no residual "merge conflict unknown" escape hatch
- make relation-local vs topology-region conflict propagation explicit

Close condition:

- every candidate record is either resolved into a typed action or left behind
  as a typed unresolved conflict

### Phase 6: lowering and canonical artifact

Goal:

- lower the resolved plan into `LoweredMergePlan`
- build `MergePlanningArtifactCore`
- add digest basis and decision log

Close condition:

- the merge subsystem emits one canonical artifact that later 7C execution can
  consume directly

### Phase 7: certification and parity

Goal:

- add artifact roundtrip tests
- add merge-ontology certification carrier
- prove the canonical artifact is serializable, replayable, and durable enough
  for later execution consumption

Close condition:

- 7B merge ontology surfaces are machine-checkable and deterministic

## Test and Certification Requirements

Milestone 7B should add a named certification requirement if the requirements
document does not already contain one:

- `Merge ontology and reconciliation semantics certification test`

It must verify:

- canonical merge planning requests round-trip without semantic drift
- merge-base rule and evidence serialize and replay identically
- identity matching reasons are deterministic
- causal metadata is deterministic and explanation-stable
- conflict classification is deterministic
- policy resolution is deterministic
- lowered merge plans are deterministic
- richer-aspect reconciliation is expressed canonically, not heuristically
- schema-declared identity basis participation is deterministic
- deletion/removal classification is deterministic
- relation three-way reconciliation classification is deterministic

Required machine-checkable outputs:

- `merge_request_digest`
- `merge_base_digest`
- `merge_identity_digest`
- `merge_causal_digest`
- `merge_conflict_digest`
- `merge_policy_digest`
- `merge_lowered_plan_digest`
- `merge_decision_log_digest`
- `merge_identity_basis_digest`
- `merge_deletion_semantics_digest`

### Required regression suites

- reversed source/target direction changes the artifact in expected canonical
  ways
- current merge-base rule identity appears in the artifact
- `LastWriterWins` without causal capability is rejected
- ambiguous identity matching produces explicit typed conflicts
- advisory correspondence remains non-authoritative
- richer-aspect reconciliation is represented as `PreferRicher` or another
  explicit typed resolution, never as fake continuity

## Important Public and Internal Interfaces

### New public types

- `MergePlanningRequest`
- `MergeIntent`
- `MergePlanningArtifact`
- `MergeExplanationSurface`
- `MergeConflictClass`
- `AspectMergePolicyDeclaration`
- `AspectMergePolicyKind`
- `IdentityBasisDeclaration`
- `IdentityBasisKind`
- `DeletionMergeClass`
- `CausalFrontier`
- `CommitCausalMetadata`

### New internal proof-bearing types

- `HistoryScopedMergePlan`
- `ValidatedIdentityMatchedMergePlan`
- `CausallyAnnotatedMergePlan`
- `ConflictClassifiedMergePlan`
- `PolicyResolvedMergePlan`
- `LoweredMergePlan`
- `LoweredMergeTargetRecordPlan`
- `LoweredMergeRelationPlan`
- `CausalCapabilityProof`

### Public interface constraints

- no public raw planner internals
- no public late-phase wrappers
- no public API that accepts loose branch ids plus loose options instead of a
  typed request
- no public API that conflates history-shape inspection and merge semantics

## Assumptions and Defaults

- Milestone 7B begins with the structural split and naming correction before
  semantic behavior expansion
- the current merge-base rule is represented honestly as
  `MaxCommitIdCommonAncestor` until changed by a later milestone
- lineage phase types are the implementation precedent for proof-bearing merge
  phase types
- aspect merge policy declarations belong to schema and lower into executable
  merge-policy catalogs
- 7B emits planning artifacts, not authoritative merged commits
- 7C execution must consume `LoweredMergePlan` and may not rediscover merge
  semantics from weaker types
- any path that cannot yet be compile-time enforced must be guarded by module
  visibility plus compile-fail tests and recorded as explicit debt

## Completion Standard

Milestone 7B is complete only when:

- merge semantics are represented as explicit proof-bearing phase types
- identity matching, causal annotation, conflict classification, and policy
  resolution are structurally distinct and mechanically enforced
- schema-declared merge policies lower into sealed runtime policy types
- one canonical `MergePlanningArtifactCore` exists and all merge explanation and
  certification surfaces derive from it
- cost claims are visible through named counters and complexity contracts
- the current branch-merge semantics stop relying on conversational meaning and
  start relying on typed evidence
- the resulting design leaves 7C with one honest job:
  execute the lowered merge plan without re-deciding what the merge means
