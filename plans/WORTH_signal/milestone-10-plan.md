# Milestone 10 Engineering Spec: Extensible Merge Strategies

> **Status:** Active engineering spec
>
> **Roadmap parent:** [signal_architecture2.md](./signal_architecture2.md)
>
> **Primary architectural driver:** `S10 - Merge-Forward Expansion`
>
> **Related implementation surfaces:**
> - [facade.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/facade.rs)
> - [builder.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/builder.rs)
> - [merge_runtime.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/branching/merge_runtime.rs)
> - [core.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/merge/core.rs)
> - [policy.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/merge/policy.rs)
> - [plan.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/merge/plan.rs)
> - [result.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/merge/result.rs)
> - [journal.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/merge/journal.rs)
> - [contract.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/node/contract.rs)
> - [facade.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/facade.rs)
> - [facade.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/commit_strategies/facade.rs)
> - [registration.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/commit_strategies/data/registration.rs)
> - [lowering.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/commit_strategies/data/lowering.rs)
> - [frozen_registry.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/commit_strategies/logic/frozen_registry.rs)
> - [test-requirements.md](./test-requirements.md)

## Summary

Milestone 10 makes merge behavior in `worth-signal` extensible without
re-opening any of the architectural shortcuts that S9.15 just closed.

This milestone is not "add more merge policies."

It is:

- host-declared merge semantics instead of planner-local hardcoding
- frozen, versioned, replay-visible merge strategy registration
- proof-bearing lowering of merge semantics before planning and execution
- explicit separation between merge policy resolution and merge execution
- bounded candidate discovery preserved under richer merge behavior
- diagnostics and replay artifacts that remain canonical as strategy surfaces grow
- certification that extensibility does not reintroduce broad scans, merge-time
  late binding, or diagnostics-truth collapse

The governing rule is:

`declare once, freeze once, lower once, execute once`

Merge strategy variability must not leak into the executor as dynamic branching.
If the executor is still making semantic decisions after lowering, the design is
incomplete.

## 1. Adversarial Constraint

This milestone must survive the following hostile condition:

> A long-lived branched runtime with geometry-kernel-scale dependency graphs and
> chip-simulator-scale replay requirements must support multiple host-specific
> merge semantics while keeping merge planning bounded by branch-carried proof,
> replay-deterministic across process restarts, and diagnostically truthful
> under repeated merge / restore / re-merge cycles.

Concretely, the design must remain correct when all of the following are true:

- node identity is not reducible to storage `NodeId`
- different aspects on the same node merge under different semantics
- deletion/removal semantics differ by host domain
- merge-base selection is not globally uniform
- only part of the overlap is conflicting
- diagnostic retention policy changes between environments
- replay occurs after code restart and registry reload
- performance-only indexes are absent, rebuilt, or churned

If any supported path falls back to whole-live branch inspection, ambient policy
lookup, or executor-side semantic branching under those conditions, the
milestone has failed.

## 2. Current-State Assessment

The current `worth-signal` merge substrate is in a much better place than a
fresh system. S9.15 already established the bounded merge floor:

- merge planning lowers through `MergeBoundaryWitness`,
  `StructuralMergeJournalSlice`, `ProofMinimalOverlapBasis`,
  `ConservativeOverlapExpansion`, `PlannedMergeCandidateSet`, and
  `LoweredMergePlan`
- executor-side candidate discovery is removed from the supported path
- merge counters already expose the correct boundedness surfaces
- conflict evidence and `resolution_plan` are typed
- merge result envelopes already preserve replay-relevant artifacts

However, the semantic control plane is still effectively hardcoded:

- [policy.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/merge/policy.rs)
  defines a fixed `BranchMergeReconciliationPolicy` with three coarse enums
- [merge_runtime.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/branching/merge_runtime.rs)
  constructs the reconciliation policy inline during planning
- identity matching is currently storage-identity-first and planner-owned
- merge-base selection is implicit in branch ancestry instead of a named lowered
  strategy surface
- supported conflict auto-resolution is encoded as direct logic against
  `ConflictMergePolicy`
- per-aspect semantics do not yet exist as declared contract-bearing forms
- runtime construction has no registry/freeze boundary for merge strategies
- replay artifacts do not yet carry a semantically versioned merge-strategy
  registry basis

This means the runtime currently has bounded merge truth, but not extensible
merge truth.

Milestone 10 closes that gap.

## 2.1 Locked Product Decisions

The following decisions are now explicit and are not open design questions for
this milestone:

- `worth-signal` will gain a first-class schema registry for merge semantics.
  Merge strategy ownership will not be modeled as node-contract-only metadata.
- pre-S10 merge, replay, snapshot, and lineage compatibility is out of scope.
  This code exists only in the demo/prototype environment, so S10 may replace
  prior merge artifact shapes outright instead of carrying compatibility debt.
- Milestone 10 must include at least one highly adversarial web-demo scenario
  that doubles as both product showcase and certification workload.

Normative consequence:

- any implementation path that preserves old merge artifact shapes by
  compatibility layering rather than replacing them is a design regression
- any implementation path that defers schema registry introduction in favor of
  â€œnode contracts first, registry laterâ€ is out of spec

## 3. Governing Design Rules

### 3.1 Extensibility Must Follow The Relational Pattern

`worth-relational` already solved the shape of extensible domain behavior:

- registration type
- frozen registry
- runtime facade access
- deterministic lowering provenance
- private proof-bearing lowered forms
- replay-visible descriptor digests

Milestone 10 should copy that pattern structurally, not aesthetically.

Required consequence:

- merge strategy configuration is not a bag of closures on `SignalRuntimeBuilder`
- it is a typed registration surface plus a frozen runtime-owned registry

### 3.2 Merge Policy Resolution Must Be Pre-Execution

All host variability must resolve before merge planning begins, or at worst
before merge execution begins if the choice depends on the already-bounded
planning artifact.

Acceptable:

- lowering identity matching strategy at runtime construction
- selecting merge-base strategy during planning and recording the result in a
  lowered plan
- lowering per-aspect merge semantics into executable policy records before the
  executor mutates target authority

Not acceptable:

- consulting ambient callbacks from inside merge execution
- switching on aspect names inside the executor
- discovering identity-match rules from runtime state at merge time
- recomputing strategy meaning during replay

### 3.3 Strategy Identity Must Be Durable

Every merge-semantic decision that can change truth must have durable identity.

That includes:

- identity-matching strategy identity
- conflict-resolution strategy identity
- merge-base strategy identity
- per-aspect merge policy identity
- deletion/removal policy identity
- conflict-isolation-granularity identity

Those identities must be:

- host-declared
- frozen at runtime construction
- semantically versioned
- recorded in canonical merge planning artifacts
- comparable during replay and diagnostics

### 3.4 Boundedness Dominates Richness

No extensibility feature may widen merge breadth beyond branch-carried proof
unless the widened breadth is itself explicitly admitted by a lowered strategy
and surfaced in counters as an intentional, explainable cost.

Examples:

- identity matching may use host-declared correspondence indexes
- it may not silently scan all live nodes in the target branch
- partial-conflict isolation may expand from node to aspect or region
- it may not implicitly widen to the whole overlapping branch

### 3.5 Diagnostics Must Remain Derived

Strategy richness must not become hot-path truth coupling.

Operational merge truth commits:

- lowered strategy identities
- lowered decision surfaces
- merge execution summary
- lineage-relevant records
- counters

Rich explanatory artifacts remain derived under runtime policy.

Changing diagnostics tier must not change:

- merge candidate construction
- merge execution decisions
- merge lineage
- replay equivalence

## 4. Scope

### 4.1 In Scope

- host-configurable identity matching
- host-configurable conflict resolution policy vocabulary
- host-configurable merge-base selection
- host-configurable per-aspect merge semantics
- host-configurable deletion/removal semantics
- host-configurable conflict-isolation granularity
- frozen merge strategy registry at runtime construction
- lowered merge strategy artifacts and replay-visible provenance
- facade and builder surfaces for the above
- crate-level certification for boundedness, replay, and diagnostics truth

### 4.2 Explicitly Out Of Scope

- heuristic name matching
- ambient runtime mutation of merge policy after build
- executor-owned semantic branching
- host-specific geometry or chip semantics inside the runtime core
- broad three-way semantic solver logic beyond the lowered strategy surface
- UI or CLI tooling for authoring strategies

## 5. Architecture Corrections Required Before Implementation

### 5.1 Replace `BranchMergeReconciliationPolicy` As The Primary Strategy Surface

The current shape:

```rust
pub struct BranchMergeReconciliationPolicy {
    pub existing_target: ExistingTargetMergePolicy,
    pub source_only: SourceOnlyMergePolicy,
    pub conflict: ConflictMergePolicy,
}
```

is too narrow and too execution-oriented.

It should become a lowered execution artifact, not the primary declaration
surface.

Required split:

- declaration layer: host-declared merge strategy registrations
- frozen layer: runtime-owned frozen merge strategy registry
- lowered layer: proof-bearing merge strategy packets attached to
  `LoweredMergePlan`
- execution layer: executor consumes lowered strategy packets only

### 5.2 Separate Strategy Families By Responsibility

Do not create one giant `MergeStrategyDescriptor` that claims to own all merge
semantics. These responsibilities differ in failure topology and cost model.

They must be separate families:

- identity matching
- merge-base selection
- conflict resolution
- aspect merge behavior
- deletion/removal behavior
- conflict isolation

Each family can share common descriptor and registry patterns, but each must
remain a distinct semantic domain.

### 5.3 Introduce Freeze And Registry Digest At Runtime Construction

`SignalRuntimeBuilder` currently freezes checkpoint and comparator policy, but
it does not own a merge strategy registry boundary.

Runtime construction must fail if:

- duplicate merge strategy identities are registered
- multiple default strategies are declared for the same scope without explicit
  precedence
- required built-in strategy families are missing
- registry semantic versions conflict

The built runtime must expose:

- frozen merge strategy registry
- registry digest
- deterministic lookup surfaces

### 5.4 Extend `LoweredMergePlan` With Strategy Provenance

`LoweredMergePlan` already carries `merge_strategy`,
`reconciliation_policy`, `merge_base`, and `resolution_plan`, but it does not
yet encode the full S10 strategy provenance.

It must gain lowered proof-bearing fields for:

- lowered identity matching record
- lowered merge-base selection record
- lowered conflict resolution policy records
- lowered aspect merge policy records
- lowered deletion policy records
- lowered conflict-isolation record
- strategy-registry digest and version basis

### 5.5 Preserve Proof Privacy

As with relational strategy lowering:

- registration types may be public
- frozen registry may be public read-only
- lowered merge strategy forms must be constructed only by owning modules
- `LoweredMergePlan` fields remain private
- external callers cannot synthesize lowered strategy packets

## 6. Target Runtime Model

### 6.1 Declaration Layer

Add a new merge strategy configuration subsystem under
`crates/worth-signal/src/logic/transaction/runtime/state/merge/strategies/`
with these responsibility groups:

- `descriptor.rs`
- `registration.rs`
- `frozen_registry.rs`
- `identity_matching/`
- `merge_base/`
- `conflict_resolution/`
- `aspect_policy/`
- `deletion_policy/`
- `conflict_isolation/`

Representative public declaration types:

```rust
pub struct MergeStrategySemanticName(String);
pub struct MergeStrategyFamilyName(String);
pub struct MergeStrategyVersion {
    major: u16,
    minor: u16,
}

pub struct MergeStrategyDescriptor {
    strategy_id: MergeStrategyId,
    semantic_name: MergeStrategySemanticName,
    family_name: MergeStrategyFamilyName,
    version: MergeStrategyVersion,
    artifact_name: PersistentArtifactName,
    cost_class: MergeStrategyCostClass,
    replay_semantics_version: MergeReplaySemanticsVersion,
}

pub struct IdentityMatchingStrategyRegistration { ... }
pub struct MergeBaseStrategyRegistration { ... }
pub struct ConflictResolutionStrategyRegistration { ... }
pub struct AspectMergePolicyRegistration { ... }
pub struct DeletionMergePolicyRegistration { ... }
pub struct ConflictIsolationRegistration { ... }
```

Design rules:

- family-specific registration types wrap a shared descriptor
- registration validation is structural and eager
- names and versions must be non-empty and deterministic
- per-family registration types carry only declarations, never execution state

### 6.2 Frozen Registry Layer

Add:

```rust
pub struct FrozenMergeStrategyRegistry {
    identity_matching: FrozenIdentityMatchingRegistry,
    merge_base: FrozenMergeBaseRegistry,
    conflict_resolution: FrozenConflictResolutionRegistry,
    aspect_policy: FrozenAspectMergePolicyRegistry,
    deletion_policy: FrozenDeletionPolicyRegistry,
    conflict_isolation: FrozenConflictIsolationRegistry,
    registry_digest: MergeStrategyRegistryDigest,
}
```

The frozen registry is built exactly once by `SignalRuntimeBuilder`.

Required invariants:

- deterministic iteration order
- duplicate semantic-name rejection
- duplicate family-version rejection where disallowed
- scope ambiguity rejection
- stable digest independent of registration insertion order

### 6.3 First-Class Schema Registry And Contract Attachment

Milestone 10 requires merge semantics to attach to a first-class schema
registry owned by the runtime, not to ambient runtime choices and not to
node-contract-only declarations.

The registry is authoritative for:

- strategy-family registration
- scope ownership
- schema-default merge policy selection
- aspect merge semantics
- deletion semantics
- conflict-isolation granularity
- precedence resolution inputs

`NodeContract` remains an override and narrowing surface, not the root
authority.

Add a new schema subsystem, expected under:

- `crates/worth-signal/src/schema/mod.rs`
- `crates/worth-signal/src/schema/facade.rs`
- `crates/worth-signal/src/schema/data/`
- `crates/worth-signal/src/schema/logic/`

Representative schema surfaces:

```rust
pub struct SignalSchemaRegistry { ... }
pub struct MergeSchemaScopeId(u32);
pub struct MergeSchemaRegistration { ... }
pub struct MergeSchemaDigest([u8; 32]);
```

The resolution precedence must be explicit and canonical:

1. schema-declared required merge semantics
2. schema-declared family defaults for the owning scope
3. node-contract override, only where the schema marks the field overrideable
4. runtime built-in family default, only for families explicitly declared
   globally defaultable

No other precedence path is valid.

This precedence order must lower into proof-bearing ownership artifacts such as:

- `ResolvedMergePolicyOwnership`
- `ResolvedAspectPolicyOwnership`
- `ResolvedIdentityScopeOwnership`

Extend [`NodeContract`](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-signal\src\data\node\contract.rs)
with a merge override section:

```rust
pub struct NodeMergeContract {
    pub identity_matching_scope: IdentityMatchingScopeKey,
    pub merge_base_scope: MergeBaseScopeKey,
    pub conflict_resolution_scope: ConflictResolutionScopeKey,
    pub aspect_policy_scope: AspectPolicyScopeKey,
    pub deletion_policy_scope: DeletionPolicyScopeKey,
    pub conflict_isolation_scope: ConflictIsolationScopeKey,
}
```

Rules:

- merge contract values are declarations, not executable closures
- node-contract merge declarations cannot exist without a registry-owned schema
  scope
- overrideability is schema-declared, not implied
- if a node/aspect lacks a registry-resolved merge contract, planning must fail
  explicitly on any path that would need that decision
- builder completeness should enforce presence of a schema registry before any
  merge-capable runtime can be built

### 6.4 Lowered Strategy Layer

Add proof-bearing lowered forms:

```rust
pub struct LoweredIdentityMatchingPlan { ... }
pub struct LoweredMergeBasePlan { ... }
pub struct LoweredConflictResolutionPlan { ... }
pub struct LoweredAspectMergePlan { ... }
pub struct LoweredDeletionMergePlan { ... }
pub struct LoweredConflictIsolationPlan { ... }

pub struct LoweredMergeStrategyBundle {
    registry_digest: MergeStrategyRegistryDigest,
    identity_matching: LoweredIdentityMatchingPlan,
    merge_base: LoweredMergeBasePlan,
    conflict_resolution: LoweredConflictResolutionPlan,
    aspect_policies: Vec<LoweredAspectMergePlan>,
    deletion_policy: LoweredDeletionMergePlan,
    conflict_isolation: LoweredConflictIsolationPlan,
}
```

`LoweredMergePlan` becomes:

```rust
pub struct LoweredMergePlan {
    // existing bounded merge proof
    ...
    strategy_bundle: LoweredMergeStrategyBundle,
}
```

Rules:

- all strategy choices are frozen into the lowered bundle before execution
- replay and diagnostics compare lowered strategy bundle digests
- the executor may reject an inconsistent lowered bundle, but may not
  re-resolve semantics

Canonicality requirements:

- every lowered family artifact must define a canonical ordering basis
- `aspect_policies: Vec<LoweredAspectMergePlan>` must be sorted by canonical
  aspect key before digesting or execution
- every lowered bundle digest must incorporate:
  - merge strategy registry digest
  - schema registry digest
  - lowering semantics version
  - canonicalization semantics version
- any future unordered collection inside a lowered merge artifact is a design
  defect unless it is wrapped in a `Canonical*` proof-bearing form

### 6.5 Execution Layer

The executor consumes:

- bounded merge candidate proof
- lowered strategy bundle
- lowered conflict resolution plan
- lowered node plan

The executor may do only:

- target mutation
- dependency remap
- topology repair
- lineage emission
- canonical counter recording

The executor may not:

- pick a different merge-base
- consult the frozen registry directly
- inspect aspect names to decide behavior
- change deletion semantics
- widen conflict isolation granularity

## 7. Family-Specific Design

### 7.1 Identity Matching

Current state:

- effectively storage-node equality plus branch-carried overlap proof

Target state:

```rust
pub enum IdentityBasisKind {
    StorageNodeId,
    PersistentCorrespondence,
    StructuralFingerprint,
    HostDeclaredCorrespondence,
    LineageArtifactIdentity,
}

pub struct IdentityMatchingDeclaration {
    pub bases_in_priority_order: Vec<IdentityBasisKind>,
    pub ambiguity_policy: IdentityAmbiguityPolicy,
    pub required_scope: IdentityMatchingScope,
}
```

Rules:

- `StorageNodeId` remains a valid built-in basis, not the only basis
- host-declared correspondence evidence must enter through a bounded index or
  explicit declaration surface
- identity matching cannot scan all target nodes
- ambiguity policy must be explicit and replay-visible

Required counters:

- identity candidate breadth
- identity evidence source count
- identity ambiguity rejection count
- identity fallback basis count

Required proof carriers:

- `IdentityEvidenceSummary`
- `IdentityCandidateSet`
- `CanonicalIdentityMatchSet`

Normative rule:

- if a chosen identity basis cannot produce bounded proof through one of the
  admitted evidence carriers, planning must fail explicitly rather than
  widening into live-branch discovery

### 7.2 Merge-Base Selection

Current state:

- effectively branch ancestry and fork snapshot semantics

Target state:

```rust
pub enum MergeBaseStrategyKind {
    ForkSnapshotBoundary,
    MaxCommonSnapshotAncestor,
    HostDeclaredBoundary,
}
```

Rules:

- the selected strategy must be named and recorded
- the resulting merge-base proof must become part of `LoweredMergePlan`
- planning may use the strategy result, but execution may not recompute it

### 7.3 Conflict Resolution

Current state:

- current conflict resolution policy is coarse and partially encoded by direct
  branching in `merge_runtime.rs`

Target state:

```rust
pub enum ConflictResolutionStrategyKind {
    Reject,
    SourceWinsComparable,
    TargetWinsComparable,
    AdoptSourceRuntimeArtifact,
    PreserveTargetRuntimeArtifact,
    ReplaySourceDependencySnapshot,
    PreferRicherStructure,
}
```

Rules:

- strategies are selected by lowered conflict family and scope
- supported and unsupported conflict families remain explicit
- no conflict resolution behavior may exist without a named strategy identity
- if a family is unsupported, the lowered plan records typed rejection, not a
  soft fallback

### 7.4 Per-Aspect Merge Semantics

Current state:

- node-level merge semantics only

Target state:

```rust
pub enum AspectMergePolicyKind {
    SourceWins,
    TargetWins,
    AdditiveSet,
    MonotonicCounter,
    ReplaceWholeAspect,
    RejectOnConflict,
}
```

Rules:

- aspect policy is attached to aspect declarations or node contract merge scope
- aspect names are not used as hardcoded switches
- planning lowers aspect policy per affected aspect
- partial-conflict acceptance may depend on aspect policy, but only through
  lowered isolation proof

### 7.5 Deletion / Removal Semantics

Target state:

```rust
pub enum DeletionMergePolicyKind {
    HardDelete,
    Tombstone,
    SoftRetire,
    OrphanCascade,
    RejectDeletionConflict,
}
```

Rules:

- deletion semantics are host-declared
- merge artifacts record which deletion policy applied
- unsupported deletion semantics must fail before execution

### 7.6 Conflict Isolation Granularity

Target state:

```rust
pub enum ConflictIsolationGranularity {
    PerNode,
    PerAspect,
    PerSubgraph,
    HostDeclaredRegion,
}
```

Rules:

- isolation granularity is a lowered strategy input
- candidate broadening required by isolation must be counted explicitly
- no hidden widening to whole-overlap scope is allowed

Required proof carriers:

- `ConflictIsolationWitness`
- `RegionIsolationSummary`
- `ConservativeIsolationExpansion`

Normative rule:

- missing isolation proof must fail explicitly; it must never degrade into a
  node-global or branch-global fallback unless that wider boundary was itself
  the selected lowered strategy

## 8. Builder And Facade Changes

### 8.1 `SignalRuntimeBuilder`

Add builder methods analogous to relational strategy registration:

- `merge_identity_strategy(...)`
- `merge_base_strategy(...)`
- `merge_conflict_resolution_strategy(...)`
- `merge_aspect_policy(...)`
- `merge_deletion_policy(...)`
- `merge_conflict_isolation_policy(...)`

Also add aggregate forms:

- `merge_strategy_bundle(...)`
- `with_default_merge_strategies()`

Build-time behavior:

- registrations are validated
- frozen registry is created
- registry digest becomes part of runtime config/state
- build fails on invalid registry shape
- build fails if no schema registry is supplied for a merge-capable runtime

`with_default_merge_strategies()` is permitted only as a convenience for tests,
examples, and the demo shell, and it must still produce explicit frozen
registrations with stable semantic names and versions.

Normative rule:

- `with_default_merge_strategies()` must not silently fill missing schema-owned
  merge declarations in production-grade paths

### 8.2 Public Facade

`crate::facade::adapters` is the correct home for declaration-layer types.

Publicly expose:

- registration types
- descriptor types
- frozen registry read surfaces
- strategy digest types

Do not publicly expose:

- lowered merge strategy packet constructors
- internal lowering tokens
- executor-only packets

## 9. Planning Pipeline Rewrite

The planning pipeline must become explicitly phased:

1. boundary witness acquisition
2. structural journal slicing
3. proof-minimal overlap derivation
4. conservative overlap expansion
5. candidate set derivation
6. strategy lookup and scope binding
7. strategy lowering
8. identity resolution
9. merge-base lowering
10. conflict classification
11. conflict policy lowering
12. per-node and per-aspect reconciliation lowering
13. lowered merge bundle assembly

Required new phase artifact families:

- `Bound*`
- `Resolved*`
- `Lowered*`

Examples:

- `BoundIdentityMatchingScope`
- `ResolvedIdentityMatchingStrategy`
- `LoweredIdentityMatchingPlan`
- `ResolvedAspectMergePolicy`
- `LoweredConflictIsolationPlan`

Normative rule:

- no later phase may accept raw declarations if an earlier phase already bound
  scope and strategy identity

## 10. Replay, Diagnostics, And Provenance Requirements

### 10.1 Replay

Replay must compare:

- merge strategy registry digest
- lowered strategy bundle digests
- identity-match decisions
- merge-base decision
- aspect policy decisions
- conflict-resolution decisions
- deletion policy decisions
- final merge result

Replay mismatch classes must gain merge-strategy-specific categories:

- registry digest mismatch
- strategy semantic version mismatch
- lowered identity strategy mismatch
- lowered aspect policy mismatch
- lowered deletion policy mismatch
- lowered conflict isolation mismatch

Compatibility rule:

- pre-S10 merge and replay artifacts are not supported through compatibility
  shims
- once S10 lands, the runtime may reject older merge/replay artifacts with a
  typed `LegacyMergeArtifactUnsupported` or equivalent failure class
- no implementation time should be spent preserving or auto-upgrading prior
  prototype merge artifacts

### 10.2 Diagnostics

Diagnostics must explain:

- why a strategy was selected
- what scope it was bound to
- why a candidate was matched or rejected under identity rules
- why a conflict family was auto-resolved or rejected
- which aspect or deletion policy controlled the outcome

But those artifacts must remain tiered.

Operational hot paths retain:

- strategy identities
- counters
- final decisions

Richer policy explanation may be reconstructed if policy allows.

### 10.3 Canonical Merge Artifact

`BranchMergeExecutionSummary` remains the canonical merge truth source, but it
must grow strategy provenance so that all downstream consumers derive from one
artifact.

It must include:

- registry digest
- lowered strategy bundle digest
- selected strategy semantic names and versions
- per-family decision summaries

## 11. Testing And Certification Plan

### 11.1 Compile-Time / API Boundary Tests

Extend `phase1_api.rs` and add compile-fail coverage to prove:

- external code cannot construct lowered merge strategy bundles
- external code cannot mutate frozen registry internals
- builder must initialize required merge registry state before build
- `LoweredMergePlan` cannot be assembled by struct literal outside owning
  modules

Suggested tests:

- `tests::phase1_api::merge_strategy_lowered_packets_are_private`
- `tests::phase1_api::signal_runtime_builder_requires_frozen_merge_registry`
- `tests::phase1_api::merge_executor_consumes_lowered_strategy_bundle_only`

### 11.2 Boundedness Tests

Add crate-level merge tests proving:

- identity matching uses bounded declared evidence rather than whole-live scans
- strategy changes do not reintroduce `MergeCandidateScope`
- per-aspect isolation remains distinct from node-wide fallback
- strategy registry churn does not change candidate breadth on unchanged policy

Suggested tests:

- `tests::merge_strategies::identity_matching_uses_declared_evidence_not_live_branch_scan`
- `tests::merge_strategies::aspect_isolation_does_not_widen_to_whole_overlap`
- `tests::merge_strategies::strategy_registry_digest_change_without_selected_scope_change_does_not_change_candidates`

### 11.3 Replay / Determinism Tests

Add hostile replay tests proving:

- identical frozen registries yield identical lowered merge artifacts
- strategy semantic version drift is detected explicitly
- replay refuses silent registry mismatch
- restore / re-merge remains deterministic with configured strategies

Suggested tests:

- `tests::merge_strategies::replay_detects_merge_strategy_registry_digest_mismatch`
- `tests::merge_strategies::lowered_merge_strategy_bundle_is_replay_stable`
- `tests::merge_strategies::restore_then_merge_preserves_strategy_provenance`

### 11.3.a Web Demo Adversarial Showcase

Milestone 10 must include one adversarial scenario implemented on top of the
existing parametric gear demo at
[apps/worth-signal-demo](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\apps\worth-signal-demo),
not as a separate demo app.

The current app already gives us the exact scaffolding we need:

- branch creation and merge buttons in
  [App.tsx](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/apps/worth-signal-demo/src/App.tsx)
- worker-driven branch / merge / scrub flows in
  [demo-worker.ts](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/apps/worth-signal-demo/src/gear-scene/worker/demo-worker.ts)
- an explicit runtime layer with `planMerge()` and `executeMerge()` in
  [runtime.ts](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/apps/worth-signal-demo/src/gear-scene/core/runtime.ts)
- a timeline, replay scrubber, node inspect panel, and live HUD already wired
  into the app shell

Scenario name:

- `Adversarial Gear Merge Arena`

The point is not to invent a generic merge visualizer. The point is to turn the
existing gear scene into a hostile merge demonstration with enough semantic
structure to prove S10.

#### Existing scene model we will build on

The current scene model already decomposes into source and derived layers:

- source inputs:
  `gearTeeth`, `gearOuterRadius`, `gearInnerRadius`, `gearThickness`,
  `gearRotation`, `lightIntensity`, camera/light positions
- derived aspect nodes:
  `gearDimensionsModel`, `gearProfileModel`, `gearTopologyModel`,
  `gearMeshModel`, `lightingModel`, `viewportProjectionModel`,
  `viewportShadingModel`
- output:
  `hudModel`
- keyed family:
  `gearToothModel::tooth-N`

This is good enough to represent:

- structural identity pressure
- per-aspect merge pressure
- localized conflict isolation
- replay and lineage pressure

#### Required demo implementation plan

The adversarial scenario should be built as an explicit â€œscenario modeâ€ inside
the existing app rather than as free-form manual slider play.

Add a new demo mode toggle in
[App.tsx](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/apps/worth-signal-demo/src/App.tsx):

- `Manual Gear`
- `Adversarial Merge Arena`

When `Adversarial Merge Arena` is selected, the worker should run a fixed,
scripted sequence using the existing command pipeline instead of relying on the
user to improvise the edits.

#### Exact branch script

Use the existing `main` branch and `what-if` branch.

Script on `main`:

1. start from the current default scene
2. apply a â€œtopology-heavyâ€ patch sequence:
   - `gear.teeth += 4`
   - `gear.outerRadius += 0.18`
   - `gear.innerRadius += 0.04`
3. then apply a â€œrender-only / shadingâ€ patch sequence:
   - `light.intensity += 0.35`
   - `gear.rotation += 0.2`

Script on `what-if`:

1. fork after the baseline
2. apply a conflicting â€œtopology-heavyâ€ patch sequence:
   - `gear.teeth -= 2`
   - `gear.outerRadius += 0.10`
   - `gear.innerRadius += 0.09`
3. then apply a different â€œrender-only / shadingâ€ patch sequence:
   - `light.intensity -= 0.15`
   - `gear.rotation -= 0.35`

This gives us:

- overlapping topology changes
- overlapping dimensional changes
- overlapping but separable render-policy changes
- keyed tooth-family churn because `gear.teeth` changes the
  `gearToothModel::tooth-N` family shape

#### How merge semantics should map onto the current app

The demo should explicitly surface three semantic zones, all based on the
existing node graph:

1. topology zone:
   `gearDimensionsModel`, `gearProfileModel`, `gearTopologyModel`,
   `gearMeshModel`, and the tooth family
   This zone is where structural fingerprint and persistent correspondence
   matching are demonstrated.

2. render zone:
   `lightingModel`, `viewportProjectionModel`, `viewportShadingModel`
   This zone is where per-aspect merge policy is demonstrated. We should be
   able to show a case where render policy resolves successfully even while
   topology policy rejects or requires richer reconciliation.

3. output / explain zone:
   `hudModel`, lineage, timeline, and node inspection
   This zone proves that diagnostics and replay tell the same story as the
   merge engine.

#### UI additions required in the existing app

Add a dedicated â€œMerge Arenaâ€ panel in
[App.tsx](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/apps/worth-signal-demo/src/App.tsx)
using the existing sidebar pattern. That panel must expose:

- scenario mode selector
- `Run Adversarial Script`
- `Plan Merge`
- `Execute Merge`
- `Replay Merge`
- diagnostics tier selector

Add a merge diagnostics surface next to the existing Live HUD showing:

- schema registry digest
- lowered strategy bundle digest
- merge plan summary
- merge result summary
- candidate breadth
- identity evidence breadth
- isolation expansion breadth
- reconciliation breadth

Add a semantic-zone legend tied to the existing node tree:

- topology nodes
- render nodes
- output nodes
- conflicted nodes
- resolved nodes

The node tree already exists in
[App.tsx](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/apps/worth-signal-demo/src/App.tsx);
this work should extend it rather than replace it.

#### Worker orchestration changes

Extend the existing worker command protocol in
[protocol.ts](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/apps/worth-signal-demo/src/gear-scene/worker/protocol.ts)
with scripted scenario commands:

- `runAdversarialMergeScenario`
- `planScenarioMerge`
- `executeScenarioMerge`
- `replayScenarioMerge`
- `setDiagnosticsTier`

Implementation should live in
[demo-worker.ts](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/apps/worth-signal-demo/src/gear-scene/worker/demo-worker.ts)
and should reuse the existing:

- `applyScenePatch()`
- `createBranch()`
- `computeMergePlan()`
- `mergeActiveBranch()`
- `scrubTimeline()`

Do not build a second orchestration path outside the worker. The scripted
scenario must use the same worker authority path as normal interaction.

#### Runtime-layer changes

Extend
[runtime.ts](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/apps/worth-signal-demo/src/gear-scene/core/runtime.ts)
to expose:

- schema registry digest
- lowered merge strategy bundle digest
- merge-plan counters
- merge-result counters
- any per-family strategy summaries needed by the UI

These should come from the runtimeâ€™s canonical merge/report surfaces, not from
demo-local recomputation.

#### What exactly the scenario must prove

The scenario is successful only if the UI can show all of the following in one
run:

- changing `gear.teeth` causes tooth-family identity pressure and visibly
  changes merge candidate scope
- topology-zone conflicts and render-zone conflicts are distinguished rather
  than collapsed into one generic â€œmerge conflictâ€
- at least one zone resolves and at least one zone rejects or remains blocked
- replaying the same scripted merge produces the same schema-registry digest,
  lowered strategy bundle digest, merge-result digest, and lineage digest
- changing diagnostics tier changes retained richness only, not merge outcome
- scrubbing the timeline and re-inspecting `gearTopologyModel`, one tooth node,
  and `hudModel` preserves a coherent causal story

#### Concrete node-inspection requirements

The scripted scenario must automatically inspect these nodes after planning and
after merge:

- `gearTopologyModel`
- `gearMeshModel`
- `gearToothModel::tooth-0`
- `lightingModel`
- `hudModel`

Those are the minimum proof points because they cover:

- structural topology
- mesh consequence
- keyed-family identity
- per-aspect render semantics
- final output integrity

#### Required demo-backed certification outputs

The scenario must emit machine-checkable artifacts, not just UI text:

- canonical schema registry digest
- canonical lowered strategy bundle digest
- canonical merge plan digest
- canonical merge result digest
- canonical lineage digest
- candidate breadth counters
- identity evidence counters
- conflict-isolation counters
- replay parity assertion

These artifacts should be consumable both by the web app UI and by crate-level
tests in
[merge_strategies.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/tests/merge_strategies.rs),
so the demo and certification are exercising the same scenario truth.

### 11.4 Diagnostics-Tier Tests

Add tests proving:

- diagnostics tier affects retained richness only
- lowered strategy decisions remain canonical across tiers
- cold reconstruction can explain strategy choice without changing merge truth

Suggested tests:

- `tests::merge_strategies::diagnostics_tier_does_not_change_lowered_strategy_bundle`
- `tests::merge_strategies::strategy_explanation_reconstruction_preserves_operational_truth`

### 11.5 Domain-Shaped Certification

Before declaring the milestone closed, add at least two hostile domain-shaped
certification suites:

- fake geometry adapter:
  persistent correspondence, region isolation, aspect-specific merge policy,
  deletion/removal pressure
- fake chip/simulation adapter:
  lineage-sensitive identity, merge-base variation, conflict-isolation pressure,
  replay determinism across long branch histories

## 12. Implementation Phases

This milestone must be executed in strict phase order. Later phases are allowed
to discover issues in earlier phases, but they must not bypass an unfinished
foundation by shipping ad hoc stopgaps.

Normative rule:

- no phase may start coding its core deliverable until the previous phase exit
  criteria are satisfied or explicitly reopened

### Phase M10.0 - Phase Inventory And Replacement Cut Line

Purpose:

- eliminate ambiguity about what is being replaced versus extended
- establish the no-backward-compatibility cut line for pre-S10 merge artifacts
- create a concrete workboard before implementation begins

Entry criteria:

- the milestone spec is accepted as the current authority

Required work:

- inventory all existing merge-related public and internal types under
  `crates/worth-signal/src/logic/transaction/runtime/state/merge/`
- classify each as:
  - preserved unchanged
  - preserved but extended
  - replaced
  - deleted
- identify every current hardcoded semantic decision in
  [merge_runtime.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/branching/merge_runtime.rs)
- identify every current public or semi-public artifact that will become legacy
  and unsupported after S10
- produce a checklist mapping each strategy family in this spec to the file(s)
  that currently hardcode it

Exit criteria:

- there is an explicit replacement list for all pre-S10 merge artifacts
- there is an explicit â€œpreserve vs replaceâ€ list for all merge files
- there is no unresolved ambiguity about whether S10 is additive or replacing
  the prototype merge control plane

Must not start before:

- none; this is the required first phase

### Phase M10.1 - First-Class Schema Registry Foundation

Purpose:

- introduce the first-class schema authority required by S10
- make schema-owned merge semantics a build-time reality rather than a future
  extension point

Entry criteria:

- M10.0 replacement inventory is complete

Required work:

- create `SignalSchemaRegistry` and related data/logic/facade modules
- define schema digest and deterministic canonical ordering
- define merge schema registration surfaces and scope ids
- define which merge families are schema-owned
- define overrideability rules for node-contract merge overrides
- wire schema registry into runtime build

Required outputs:

- `SignalSchemaRegistry`
- `MergeSchemaRegistration`
- `MergeSchemaScopeId`
- `MergeSchemaDigest`
- builder and facade exposure for schema registration

Exit criteria:

- merge-capable runtimes cannot build without a schema registry
- schema registry digest is exposed as a canonical runtime artifact
- schema registrations are deterministic and duplicate-safe

Must not start before:

- M10.0 complete

### Phase M10.2 - Strategy Descriptor, Registration, And Frozen Registry

Purpose:

- add the declaration and freeze boundary for strategy families

Entry criteria:

- M10.1 complete

Required work:

- define shared merge strategy descriptor types
- define family-specific registration types
- define frozen family registries and top-level frozen registry
- define registry digest computation
- define build-time validation and duplicate rejection
- wire strategy registration into `SignalRuntimeBuilder`

Required outputs:

- descriptor types
- registration types
- frozen registry types
- builder registration methods
- read-only facade exposure

Exit criteria:

- runtime build freezes the merge strategy registry exactly once
- duplicate, ambiguous, or conflicting registrations fail at build time
- frozen registry digest is stable and order-independent

Must not start before:

- M10.1 complete

### Phase M10.3 - Ownership Resolution And Scope Binding

Purpose:

- make implicit policy precedence explicit and proof-bearing

Entry criteria:

- M10.2 complete

Required work:

- add `NodeMergeContract` as schema-scoped override surface
- implement the canonical precedence order:
  - schema required semantics
  - schema family defaults
  - node-contract overrides where allowed
  - explicit globally defaultable built-ins only
- define proof-bearing ownership artifacts for every family
- define missing-scope and disallowed-override failure surfaces

Required outputs:

- `Resolved*Ownership` forms
- scope-binding artifacts
- precedence resolution rules in code and tests

Exit criteria:

- planning never uses ambient defaults without lowered ownership proof
- override precedence is deterministic, tested, and replay-visible
- unsupported missing scope fails explicitly

Must not start before:

- M10.2 complete

### Phase M10.4 - Lowering Architecture And Canonical Bundle

Purpose:

- lower all selected strategy semantics into one canonical executable bundle

Entry criteria:

- M10.3 complete

Required work:

- define `Resolved*` and `Lowered*` family packet types
- define `LoweredMergeStrategyBundle`
- extend `LoweredMergePlan` with schema digest, registry digest, and lowered
  strategy bundle
- define canonical ordering and digest basis for every lowered family
- add privacy boundaries so lowered forms are not WORTHable externally

Required outputs:

- lowered family artifacts
- canonical bundle digest
- updated `LoweredMergePlan`
- compile-time privacy tests

Exit criteria:

- merge executor receives only lowered strategy forms
- replay artifacts carry schema digest, registry digest, and bundle digest
- unordered lowered collections are eliminated or wrapped in canonical forms

Must not start before:

- M10.3 complete

### Phase M10.5 - Identity Matching Family

Purpose:

- make identity matching extensible without reopening broad scans

Entry criteria:

- M10.4 complete

Required work:

- define identity basis family and ambiguity policy
- define bounded identity evidence carriers
- implement lowered identity matching plan
- integrate identity matching into merge planning
- add identity counters and complexity assertions

Required outputs:

- `IdentityEvidenceSummary`
- `IdentityCandidateSet`
- `CanonicalIdentityMatchSet`
- lowered identity strategy records

Exit criteria:

- non-`NodeId` identity matching is possible
- missing bounded identity evidence fails explicitly
- identity matching cannot silently widen into live-branch discovery

Must not start before:

- M10.4 complete

### Phase M10.6 - Merge-Base Selection Family

Purpose:

- make merge-base selection explicit, named, and replay-visible

Entry criteria:

- M10.5 complete

Required work:

- define merge-base strategy family
- implement lowered merge-base plan
- record merge-base selection in canonical artifacts
- add replay and diagnostics surfaces for merge-base choice

Exit criteria:

- merge-base selection is named, lowered, and replay-visible
- execution no longer re-derives merge-base semantics

Must not start before:

- M10.5 complete

### Phase M10.7 - Conflict Resolution Family

Purpose:

- replace coarse hardcoded conflict policy with lowered typed family semantics

Entry criteria:

- M10.6 complete

Required work:

- define conflict resolution family descriptors and registrations
- map conflict families to supported and unsupported resolution families
- lower conflict strategy selection into executable records
- replace direct `ConflictMergePolicy` branching with lowered conflict records

Exit criteria:

- supported conflict families lower into typed execution plans
- unsupported conflict families remain typed rejection
- no conflict resolution behavior exists without named strategy identity

Must not start before:

- M10.6 complete

### Phase M10.8 - Per-Aspect Merge Policy Family

Purpose:

- make aspect-level merge semantics first-class and enforceable

Entry criteria:

- M10.7 complete

Required work:

- define aspect merge policy family
- attach aspect policies to schema scopes
- lower affected aspect policies into canonical execution records
- enforce canonical aspect ordering and per-aspect decision reporting

Exit criteria:

- per-aspect merge semantics are no longer hardcoded or node-global by default
- aspect merge policy selection is replay-visible and digest-stable

Must not start before:

- M10.7 complete

### Phase M10.9 - Deletion And Conflict Isolation Families

Purpose:

- add the remaining family semantics required for truthful partial acceptance

Entry criteria:

- M10.8 complete

Required work:

- define deletion/removal policy family
- define conflict isolation family
- add explicit proof carriers for isolation widening
- integrate deletion and isolation into lowered bundle and execution
- add counters for isolation expansion breadth

Exit criteria:

- deletion semantics are host-declared and recorded in artifacts
- isolation widening is explicit, bounded, and counted
- missing isolation proof fails explicitly instead of degrading silently

Must not start before:

- M10.8 complete

### Phase M10.10 - Runtime Surfaces, Diagnostics, And Replay

Purpose:

- expose the new merge semantics through canonical runtime surfaces

Entry criteria:

- M10.9 complete

Required work:

- extend merge result and execution summary artifacts
- add schema digest, registry digest, bundle digest, and family summaries
- add replay mismatch classes for merge-strategy drift
- add diagnostics-tier-safe strategy explanation surfaces
- add legacy-artifact rejection path for pre-S10 merge/replay artifacts

Exit criteria:

- canonical merge artifacts contain the new strategy provenance
- replay detects registry or lowering drift explicitly
- diagnostics tier changes richness only, not merge truth

Must not start before:

- M10.9 complete

### Phase M10.11 - Complexity Contracts And Crate-Level Certification

Purpose:

- convert performance and boundedness claims into enforced proof

Entry criteria:

- M10.10 complete

Required work:

- declare merge complexity contracts and named counters
- add exact boundedness tests for candidate breadth, identity breadth,
  isolation breadth, and reconciliation breadth
- add replay and diagnostics certification tests
- add compile-time boundary tests for lowered artifact privacy and builder
  completeness

Exit criteria:

- merge complexity contracts are declared and tested
- boundedness regressions fail tests deterministically
- compile-time and crate-level proof surfaces cover every strategy family

Must not start before:

- M10.10 complete

### Phase M10.12 - Adversarial Web Demo Scenario

Purpose:

- prove the milestone in the real demo shell using the same runtime surfaces

Entry criteria:

- M10.11 complete

Required work:

- implement `Adversarial Gear Merge Arena` in the existing demo app
- add scripted worker commands and scenario-mode UI
- expose schema digest, lowered strategy bundle digest, and merge counters in
  the demo
- wire replay and timeline scrubbing to the scripted scenario

Exit criteria:

- the demo runs the scripted adversarial scenario end-to-end
- the demo surfaces canonical digests and counters from runtime truth
- the demo proves replay parity and diagnostics-tier invariance visibly

Must not start before:

- M10.11 complete

### Phase M10.13 - Closeout And Architecture Update

Purpose:

- finish the milestone with explicit preserved/deferred scope

Entry criteria:

- M10.12 complete

Required work:

- update `signal_architecture2.md` with final S10 landed status
- explicitly mark supported merge families and unsupported deferred families
- remove or delete superseded prototype merge semantics
- write closeout evidence references

Exit criteria:

- all S10 supported behaviors are explicitly documented
- no remaining supported path uses planner-local hardcoded merge semantics
- certification and demo evidence are linked from the architecture docs

Must not start before:

- M10.12 complete

## 13. File Plan

### New files expected

- `crates/worth-signal/src/logic/transaction/runtime/state/merge/strategies/mod.rs`
- `crates/worth-signal/src/logic/transaction/runtime/state/merge/strategies/descriptor.rs`
- `crates/worth-signal/src/logic/transaction/runtime/state/merge/strategies/registration.rs`
- `crates/worth-signal/src/logic/transaction/runtime/state/merge/strategies/frozen_registry.rs`
- `crates/worth-signal/src/logic/transaction/runtime/state/merge/strategies/identity_matching.rs`
- `crates/worth-signal/src/logic/transaction/runtime/state/merge/strategies/merge_base.rs`
- `crates/worth-signal/src/logic/transaction/runtime/state/merge/strategies/conflict_resolution.rs`
- `crates/worth-signal/src/logic/transaction/runtime/state/merge/strategies/aspect_policy.rs`
- `crates/worth-signal/src/logic/transaction/runtime/state/merge/strategies/deletion_policy.rs`
- `crates/worth-signal/src/logic/transaction/runtime/state/merge/strategies/conflict_isolation.rs`
- `crates/worth-signal/src/tests/merge_strategies.rs`

### Existing files expected to change

- `crates/worth-signal/src/facade.rs`
- `crates/worth-signal/src/logic/transaction/mod.rs`
- `crates/worth-signal/src/logic/transaction/runtime/state/builder.rs`
- `crates/worth-signal/src/data/node/contract.rs`
- `crates/worth-signal/src/logic/transaction/runtime/state/merge/core.rs`
- `crates/worth-signal/src/logic/transaction/runtime/state/merge/policy.rs`
- `crates/worth-signal/src/logic/transaction/runtime/state/merge/plan.rs`
- `crates/worth-signal/src/logic/transaction/runtime/state/merge/result.rs`
- `crates/worth-signal/src/logic/transaction/runtime/state/branching/merge_runtime.rs`
- `crates/worth-signal/src/tests/phase1_api.rs`
- `crates/worth-signal/src/tests/merge_adoption.rs`
- `crates/worth-signal/src/tests/adversarial_diagnostics.rs`

## 14. Anti-Patterns Explicitly Rejected

- building S10 as a bigger `match` inside `merge_runtime.rs`
- using `NodeId` equality as the only identity-matching truth
- storing host merge policy as ad hoc closures without durable identity
- looking up merge strategy from runtime-global mutable state during execution
- using diagnostics artifacts as the only place where strategy identity exists
- allowing convenience indexes to change lowered merge semantics
- implementing per-aspect policy by string comparisons on aspect names
- allowing strategy registry contents to mutate after `SignalRuntime::build()`

## 15. Closeout Standard

Milestone 10 is complete only when all of the following are true:

- every supported merge behavior that can differ by host domain is declared,
  frozen, lowered, and replay-visible
- merge execution consumes lowered strategy packets only
- merge candidate construction remains bounded by branch-carried proof
- strategy extensibility does not change canonical merge truth across
  diagnostics tiers
- replay detects registry or strategy drift explicitly
- the certification suite proves boundedness, determinism, and diagnostics
  truth under hostile branch evolution

If implementation lands but the runtime still relies on planner-local semantic
branching or merge-time policy discovery, the milestone is not complete.
