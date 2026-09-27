# S9 Missing-Substrate Completion Spec

## Summary

This document defines the completion work for the remaining unfinished substrate across `S9.9`, `S9.10`, `S9.12`, and `S9.15`.

These are not polish tasks. They are the remaining places where the runtime is architecturally honest about incompleteness, but not yet complete:

- `S9.12`: reconstructability is improved, but restore still is not fully forced through a canonical proof chain of `checkpoint + bounded journal + required derived rebuild`
- `S9.15`: whole-live merge fallback is gone, but bounded merge proof is not yet complete enough to make all supported merge flows purely proof-driven
- `S9.9`: proof-driven grouped concurrent apply is now closed for proof-safe static stages, and ineligible full-parallel stages lower honestly to serial execution with named rejection
- `S9.10`: rollback and lifecycle are now packetized and type-separated on the supported path, with only final sealing/doc closeout remaining

This spec is governed by:

- [MENTALITY.md](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/docs/coding-guidelines/MENTALITY.md)
- [architectural_guidelines.md](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/docs/coding-guidelines/architectural_guidelines.md)
- [performance_guidelines.md](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/docs/coding-guidelines/performance_guidelines.md)
- [signal_architecture2.md](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/signal_architecture2.md)

Primary adversarial constraint:

- under long-lived branch/snapshot churn, geometry-kernel-scale graphs, hostile restore/merge/replay workloads, and disjoint parallel batches, the runtime must preserve identical authoritative truth and semantically required derived truth while keeping hot-path breadth bounded by semantic delta and forbidding whole-live fallback, fake parallelism, policy-driven semantic drift, or baseline-bundle rollback as supported behavior

Implementation order:

1. shared proof substrate laws
2. `S9.12` reconstructability completion
3. `S9.15` bounded merge completion
4. `S9.9` true parallel apply completion
5. `S9.10` rollback and lifecycle completion
6. cross-phase certification closeout

## Execution Control Layer

This section is mandatory implementation scaffolding for the rest of the spec.
It exists to keep the work intelligible under active implementation pressure.

Every workstream below must be executable through the same control surfaces:

- migration table
  - current substrate
  - temporary bridge substrate
  - final proof-bearing substrate
  - supported callers during migration
  - surfaces that must stop compiling at closeout
- proof inventory row
  - producer
  - consumer
  - invalidation trigger
  - replay-verifiable yes/no
  - persistence lifetime
  - measurement boundary
  - negative-space condition
- closeout impossibility checklist
  - the exact legacy behaviors that must be unrepresentable when the workstream closes
- certification staging
  - compile-time barrier
  - local proof tests
  - adversarial/property tests
  - cross-phase equivalence tests
  - named harness debt if final harness work is deferred

No workstream is complete if it lands target nouns but leaves legacy execution
routes structurally valid.

## Global Closeout Impossibility Checklist

The following behaviors must be unrepresentable on supported paths by the end
of this program:

- reconstructability with `journal: None`
- restore from raw snapshot-like state bundles
- restore that mixes authority rebuild and diagnostic rebuild in one semantic phase
- whole-live merge candidate scope on supported merge paths
- candidate construction that depends on ambient branch inspection
- grouped concurrent apply that performs the majority of semantic authority work in serial reduction
- worker access to shared runtime surfaces during grouped concurrent apply
- rollback that depends on baseline state bundles rather than typed inverse authority effects
- routine lifecycle paths that can construct heavy capture witnesses
- diagnostics policy that changes authority truth, semantically required derived truth, or admissibility

## Global Proof Inventory Contract

Every proof-bearing type introduced by this spec must have an implementation-time
inventory row in code review or closeout notes using this shape:

| Proof type | Producer | Consumer | Invalidation trigger | Replay-verifiable | Lifetime | Measurement boundary | Negative space |
| --- | --- | --- | --- | --- | --- | --- | --- |

The purpose of the table is not documentation. It is to prevent silent drift in
what each subsystem means by "proof."

## Execution Phase Chain Template

Every workstream phase chain must be rendered using the following exact
questions:

1. What proof-bearing type is accepted at the entry boundary?
2. What narrower proof or typed packet does this phase produce?
3. What facts become illegal to rediscover after this phase?
4. What counter surface proves the phase remained bounded?
5. What caller-visible API is allowed to consume the output?

If a phase cannot answer those questions concretely, it is still descriptive
architecture rather than executable architecture.

## Shared Proof Substrate Laws

These laws are mandatory for every proof-bearing object introduced by this work. This section exists to prevent each subsystem from inventing its own private notion of "proof."

### 1. Proofs are boundary products, not late orchestration guesses

A proof object must be produced directly from canonical mutation-time, checkpoint-time, or lifecycle-time artifacts. It must not be synthesized later from arbitrary current state.

Allowed:

- merge proof derived from mutation ledger boundaries, merge-base checkpoint truth, and proof-authorized overlap indexes maintained from the same journal semantics
- reconstructability proof derived from checkpoint capture plus bounded retained journal segment
- rollback packets emitted during effect application
- disjoint apply proof derived from lowered task footprints against a specific planning basis

Forbidden:

- proof assembled by broad branch inspection after the fact
- proof inferred from convenience caches
- proof reconstructed from diagnostic summaries
- proof that depends on arbitrary ambient whole-live state discovery

### 2. Every proof type must encode five things

Every proof-bearing type must explicitly encode or imply through its type family:

- authority scope
  - what authority surface, branch, checkpoint, stage, or mutation basis it certifies
- boundedness scope
  - what exact breadth, cursor window, candidate set, or touched-surface set it certifies
- continuity basis
  - what chain or boundary makes the proof valid
- invalidation rule
  - what mutation or lifecycle event makes it stale
- reuse model
  - whether it is consumable once, inspectable, replay-stable, persistent until mutation, or snapshot-stable

### 3. Descriptive metadata is never admissibility proof

A struct with fields and labels is not proof unless supported execution can proceed from it without broad rediscovery.

If execution still needs:

- whole-branch inspection
- whole-graph rescans
- speculative runtime validation
- hidden "just to be safe" broad reconstruction

then the object is descriptive metadata, not admissibility proof.

### 4. Supported execution may consume only proof-authorized indexes

A proof-bearing path may consume indexes only if those indexes are themselves maintained from the same canonical mutation, checkpoint, or journal semantics that define the proof.

Allowed:

- branch overlap index maintained from mutation journal semantics
- dependency snapshot index maintained from authoritative dependency snapshot updates
- replay suffix index maintained from retained replay/journal packets

Forbidden:

- convenience branch index that silently widens merge planning
- whole-live helper index that effectively reintroduces broad branch scans
- diagnostic-only index used to satisfy semantic admissibility

### 5. Proofs must be replay-verifiable where the phase requires replay truth

For reconstructability, merge, and rollback, proof must be verifiable against retained canonical packets. Planner confidence is not enough.

### 6. Supported execution must consume lowered proof-bearing packets only

This follows Architectural Law 30 directly.

For every phase:

- phase `K` output type must be the only acceptable input to phase `K+1`
- raw collections, mutable state bundles, and convenience structs must not bypass the proof chain
- executors must not re-decide strategy, admissibility, or boundedness

### 7. Proof-bearing types must not silently unify cost or semantic boundaries

This follows Architectural Laws 29 and 40.

Do not collapse:

- bounded proof-minimal overlap and conservative expansion
- retained authority truth and policy-rich diagnostics
- concurrent group-local work and serial reduction work
- inverse authority effects and imperative undo commands

## Shared Runtime Classification

Every retained, rebuilt, or restored surface must belong to exactly one class.

### 1. Authoritative retained truth

Examples:

- graph authority
- node authority state
- checkpoint authority
- branch ancestry authority
- mutation journal authority
- merge-base authority truth

### 2. Semantically required derived truth

Examples:

- canonical dependency/topology indexes required for valid execution
- replay suffix structures required for reconstructability or merge truth
- merge support structures required for supported bounded reconciliation
- any derived structure whose absence would change admissibility or semantic outcome

### 3. Performance-only derived state

Examples:

- acceleration indexes
- bounded caches
- convenience lookup structures
- precomputed candidate maps that can be rebuilt without changing truth

### 4. Diagnostic richness

Examples:

- explanation artifacts
- provenance detail
- forensic summaries
- convenience inspection packets
- retained diagnostics beyond what semantic correctness requires

Hard rules:

- policy may affect diagnostic richness only
- policy may not affect authoritative truth
- policy may not affect semantically required derived truth
- policy may not affect merge/apply/restore admissibility
- restore, merge, rollback, and planning may depend only on authoritative or semantically required derived truth

## Workstream Migration Tables

The tables in this section are implementation control surfaces. They name the
legacy substrate that must be retired rather than merely surrounded by new
types.

### `S9.12` Reconstructability Migration Table

| Category | Current substrate | Temporary bridge substrate | Final substrate | Must stop compiling at closeout |
| --- | --- | --- | --- | --- |
| restore authority proof | `ReconstructabilityRecord` and snapshot-like metadata bundles | adapters that construct `ReconstructabilityProof` from existing checkpoint and retained journal capture | `ReconstructabilityProof` | restore helpers that accept raw snapshot bundles or partially descriptive metadata |
| journal truth | `JournalSegment` with optional attachment | bridge validation that rejects missing or discontinuous journal early | `BoundedJournalSegment` | any supported path with absent journal proof |
| rebuild classification | open-ended derived rebuild logic | explicit bridge mapper from existing rebuild lanes into closed semantic classes | `RequiredDerivedRebuildSet` | rebuild code that can mix required derived truth with policy-rich diagnostics |
| restore pipeline | mixed restore helpers | wrappers that call the new 3-phase chain internally | `restore_authority_from_checkpoint -> rebuild_required_derived_from_authority -> apply_diagnostic_policy_richness` | any supported restore helper that collapses those phases |

### `S9.15` Bounded Merge Migration Table

| Category | Current substrate | Temporary bridge substrate | Final substrate | Must stop compiling at closeout |
| --- | --- | --- | --- | --- |
| merge boundary | branch/snapshot-adjacent merge inputs | bridge constructor that lowers existing branch metadata into boundary witness form | `MergeBoundaryWitness` | merge helpers that infer admissibility from ambient branch state |
| source delta | `BranchMutationJournalSlice` without full proof split | source journal bridge with explicit boundary attachment | `StructuralMergeJournalSlice` | candidate construction from branch-state bundles alone |
| overlap | combined overlap/candidate shaping | temporary derivation functions that emit distinct overlap forms before final executor adoption | `ProofMinimalOverlapBasis` plus `ConservativeOverlapExpansion` | overlap and candidate scope collapsed into one concept |
| candidate scope | `MergeCandidateScope`, including whole-live scope | temporary narrow candidate adapters | `PlannedMergeCandidateSet` inside `LoweredMergePlan` | whole-live supported candidate scope |
| execution | merge plan plus ambient lookups | executor wrapper that only reads lowered plan fields and proof-authorized indexes | `LoweredMergePlan` | executor-side candidate discovery or ambient branch inspection |

### `S9.9` Parallel Apply Migration Table

| Category | Current substrate | Temporary bridge substrate | Final substrate | Must stop compiling at closeout |
| --- | --- | --- | --- | --- |
| grouping | disjoint grouping from footprints | explicit lowering from grouping result into proof-bearing concurrent admission object | `DisjointApplyProof` | grouped concurrent admission without proof object |
| execution truth | grouped planning with serial mutable apply reality | staged worker packet buffering behind current executor shape | `LoweredApplyPlan::GroupedConcurrent(ConcurrentApplyPlan)` | `FullParallel` execution that is semantically serial |
| worker output | mixed direct/shared writes or shared-surface temptation | bridge worker packets that isolate existing outputs before reduction | `GroupLocalApplyPacket` | worker access to shared runtime surfaces |
| reduction | broad serial post-pass | narrow deterministic publication-only reducer | `ConcurrentApplyReductionPlan` | reduction that redoes majority semantic work |

### `S9.10` Rollback and Lifecycle Migration Table

| Category | Current substrate | Temporary bridge substrate | Final substrate | Must stop compiling at closeout |
| --- | --- | --- | --- | --- |
| rollback truth | lazy rollback baselines and repair ceremony | effect-derived packet staging from patch and created-node surfaces | `TransactionRollbackPacket` | rollback via baseline bundles as supported semantic truth |
| branch move | branch-state load/store helpers | wrappers that materialize move-only transfer packets internally | `AuthorityTransferPacket` | branch switch APIs that imply duplication |
| branch duplicate | generic branch capture/fork helpers | explicit bridge for duplication-only call paths | `ExplicitBranchForkPacket` | branch fork without explicit duplicate truth packet |
| restore | branch restore from snapshot-shaped bundles | typed lifecycle transfer wrapper over proof-gated restore state | `BranchLifecycleTransfer::Restore(...)` | branch restore APIs that accept raw branch bundles |
| heavy capture | helper-accessible heavyweight capture | runtime-owned heavy-capture helpers with private witness construction | `HeavyCaptureWitness` | routine lifecycle code that can construct heavy capture directly |

## Proof Inventory by Workstream

This section is the minimum inventory expected during implementation.

### `S9.12` Required Proof Inventory

| Proof type | Producer | Consumer | Invalidation trigger | Replay-verifiable | Lifetime | Measurement boundary | Negative space |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `CheckpointBoundary` | checkpoint capture | restore phase 1 | mutation beyond checkpoint authority | yes | persistent | checkpoint facade counters | raw checkpoint metadata must not act as authority proof |
| `BoundedJournalSegment` | retained journal capture | restore phase 1 and replay verification | continuity break or truncation | yes | replay-stable | journal span and retained-bytes counters | optional journal attachment |
| `RequiredDerivedRebuildSet` | reconstructability lowering | restore phase 2 | mismatch with checkpoint or authority basis | phase-dependent | inspectable | required-derived breadth counters | policy-rich diagnostics inside required rebuild set |
| `ReconstructabilityProof` | checkpoint plus journal boundary capture | restore facade | mutation beyond end cursor or checkpoint mismatch | yes | restore-consumable | restore breadth counters | restore from snapshot bundle or descriptive record |

### `S9.15` Required Proof Inventory

| Proof type | Producer | Consumer | Invalidation trigger | Replay-verifiable | Lifetime | Measurement boundary | Negative space |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `MergeBoundaryWitness` | branch lifecycle plus mutation ledger plus merge-base proof | merge lowering | source or target mutation past boundary | yes | inspectable until mutation | boundary witness counters | merge admissibility from ambient branch truth |
| `StructuralMergeJournalSlice` | mutation ledger | overlap derivation | further source mutation | yes | inspectable | source slice breadth | whole-live source inspection |
| `ProofMinimalOverlapBasis` | merge lowering | overlap expansion and candidate lowering | boundary or index invalidation | yes | planning-only | proof-minimal breadth | heuristic overlap expansion inside minimal proof |
| `ConservativeOverlapExpansion` | merge lowering using proof-authorized indexes | candidate lowering | same as overlap basis | yes | planning-only | overlap expansion breadth | silent widening from convenience indexes |
| `LoweredMergePlan` | merge lowering | merge executor | mutation beyond witness basis | yes | single-use per merge execution | final candidate and reconciliation breadth | executor-side candidate discovery |

### `S9.9` Required Proof Inventory

| Proof type | Producer | Consumer | Invalidation trigger | Replay-verifiable | Lifetime | Measurement boundary | Negative space |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `DisjointApplyProof` | planner lowering from canonical footprints | concurrent executor | planning basis or policy change | indirectly | single-use | admission and group-local breadth counters | grouped concurrency without proof |
| `GroupLocalApplyPacket` | worker-local apply path | deterministic reducer | packet mutation or ordering mismatch | indirectly | single-use | group-local packet breadth | worker writes to shared surfaces |
| `ConcurrentApplyReductionPlan` | planner lowering | reducer only | planning basis or policy change | indirectly | single-use | reduction breadth counters | reduction as hidden serial semantic engine |

### `S9.10` Required Proof Inventory

| Proof type | Producer | Consumer | Invalidation trigger | Replay-verifiable | Lifetime | Measurement boundary | Negative space |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `TransactionRollbackPacket` | effect application | rollback/finalize path | wrong baseline or wrong transaction order | yes | single-use | rollback packet counters | rollback as imperative undo script or bundle |
| `AuthorityTransferPacket` | lifecycle move path | branch switch | mutation after packet creation | no | single-use move-only | move transfer counters | move that secretly duplicates |
| `ExplicitBranchForkPacket` | lifecycle duplicate path | branch fork | mutation after packet creation | no | single-use | explicit fork counters | duplicate truth created by generic switch/capture |
| `BranchLifecycleTransfer` | lifecycle owner | branch switch and active restore | wrong packet kind or stale branch state | no | single-use | move/restore counters | raw branch-state load on supported paths |
| `HeavyCaptureWitness` | sealed lifecycle owner | heavy capture path only | module boundary escape | no | scoped | heavy capture counters | routine path access to heavyweight capture |

## Workstream 1: `S9.12` Reconstructability Completion

### Adversarial constraint

A checkpointed, branch-heavy runtime under long history and repeated snapshot restore must be reconstructable from checkpoint plus a bounded journal suffix only. Restoring from snapshot-like state bundles, broad retained history, or diagnostic surfaces is architecturally invalid.

### Current defect

The runtime is now more honest:

- snapshot reconstructability records retained replay span
- journal proof is no longer silently absent

But reconstructability is still too record-like:

- restore is not yet forced to consume one canonical proof object
- the separation between authority restore, semantically required derived rebuild, and policy-rich diagnostics is not yet pinned tightly enough
- `RequiredDerivedRebuildSet` does not yet exist as a closed semantic class

### Required target forms

```rust
pub struct CheckpointBoundary {
    pub checkpoint_id: CheckpointId,
    pub authority_digest: AuthorityDigest,
    pub branch_identity: BranchIdentity,
    pub checkpoint_kind: CheckpointKind,
}

pub struct BoundedJournalSegment {
    pub start_cursor: ReplayCursor,
    pub end_cursor: ReplayCursor,
    pub retained_events: JournalEventSet,
    pub continuity_digest: JournalContinuityDigest,
    pub replay_stable: bool,
}

pub enum RequiredDerivedRebuildSet {
    DependencyIndexes(DependencyIndexRebuildProof),
    ReplaySuffix(ReplaySuffixRebuildProof),
    MergeSupport(MergeSupportRebuildProof),
}

pub struct ReconstructabilityProof {
    pub checkpoint: CheckpointBoundary,
    pub journal: BoundedJournalSegment,
    pub required_rebuild: Vec<RequiredDerivedRebuildSet>,
}
```

### Proof semantics

`CheckpointBoundary`

- producer: checkpoint subsystem at capture time
- certifies: authority boundary and branch identity
- invalidated by: mutation beyond the captured checkpoint state
- reuse model: persistent and inspectable

`BoundedJournalSegment`

- producer: retained journal/replay capture at snapshot time
- certifies: bounded suffix continuity and retained replay window
- invalidated by: continuity break or truncation beyond supported retained span
- replay-verifiable: yes
- reuse model: inspectable and replay-stable

`ReconstructabilityProof`

- producer: checkpoint capture plus bounded journal capture
- certifies: authority reconstruction scope and semantically required rebuild scope
- invalidated by: authority mutation beyond `journal.end_cursor` or mismatched checkpoint continuity
- replay-verifiable: yes
- reuse model: inspectable and restore-consumable

### Required classification lock

`RequiredDerivedRebuildSet` may include only semantically required derived truth:

- dependency snapshot/index structures required for later valid execution
- replay suffix structures required for equivalence or merge support
- bounded merge-support structures if they are required for supported merge semantics

It must exclude:

- convenience caches
- rich diagnostics
- policy-tier summaries
- optional forensic materializations

### Restore phase chain

Restore must become exactly three explicit phases:

1. `restore_authority_from_checkpoint`
   - input: `CheckpointBoundary`
   - output: authoritative runtime truth only

2. `rebuild_required_derived_from_authority`
   - input: authoritative runtime truth plus `RequiredDerivedRebuildSet`
   - output: semantically required derived truth only

3. `apply_diagnostic_policy_richness`
   - input: authority + semantically required derived truth + active diagnostics policy
   - output: policy-shaped diagnostic richness only

Hard rule:

- phase 3 may not mutate authority
- phase 3 may not mutate semantically required derived truth
- phase 3 may not alter merge/apply/restore admissibility
- phase 3 may not introduce quasi-authoritative caches required by later planning or execution

### Compile-time enforcement

- restore APIs accept `ReconstructabilityProof` only
- no supported restore helper accepts raw snapshot bundles or partially descriptive snapshot metadata
- `RequiredDerivedRebuildSet` is a closed type family owned by reconstructability code
- policy-rich diagnostic rebuild APIs are structurally separate from reconstructability restore APIs
- no supported snapshot path may produce `journal: None`

### Measurement boundaries

Checkpoint/reconstructability facade must expose:

- `journal_replay_span`
- `journal_event_count`
- `journal_retained_bytes`
- `restore_authority_breadth`
- `restore_required_derived_breadth`
- `restore_diagnostic_richness_breadth`

### Acceptance tests

- `checkpoint_plus_bounded_journal_is_sufficient_for_restore`
- `destroying_all_derived_state_and_rebuilding_from_reconstructability_proof_preserves_truth`
- `required_derived_rebuild_set_contains_no_policy_optional_richness`
- `restore_under_tier_matrix_preserves_authority_and_required_derived_truth`
- `discontinuous_or_truncated_journal_proof_fails_before_restore_mutation`
- `replay_suffix_equivalence_holds_across_long_snapshot_restore_churn`

### Closeout criteria

- checkpoint + bounded journal is the only supported reconstructability truth
- authority, semantically required derived truth, performance-only derived state, and diagnostic richness are structurally distinct
- restore no longer accepts snapshot-like bundles as authority proof

### Certification staging

Compile-time barrier:

- restore facade accepts `ReconstructabilityProof` only
- required-derived rebuild APIs cannot accept policy-rich diagnostic carriers

Local proof tests:

- `checkpoint_plus_bounded_journal_is_sufficient_for_restore`
- `required_derived_rebuild_set_contains_no_policy_optional_richness`

Adversarial/property tests:

- discontinuous retained journal rejection under churn
- destroy-all-derived-and-rebuild equivalence under repeated checkpoint/restore cycles

Cross-phase equivalence tests:

- restore under diagnostics tier matrix preserves authority and semantically required derived truth
- replay suffix equivalence across repeated restore churn

Allowed harness debt:

- large-scale slope harness work may remain open if named counters and exact local proof tests already exist

## Workstream 2: `S9.15` Bounded Merge Completion

### Adversarial constraint

Under long-lived branch divergence, repeated snapshot restore, and hostile overlapping mutation journals, merge planning must remain bounded by carried proof rather than ambient whole-branch knowledge. Supported merge flows must either lower to a bounded merge plan or fail before candidate construction.

### Closeout status

`S9.15` is closed for the supported S9 merge envelope.

The whole-live supported scope is retired, merge candidate construction is now
lowered through the proof chain below, and supported execution consumes only
`LoweredMergePlan`.

Retired legacy surface:

- `MergeCandidateScope`, including whole-live supported scope

Closeout evidence:

- merge boundary proof is required before candidate construction
- source journal truth is carried as `StructuralMergeJournalSlice`
- proof-minimal overlap and conservative expansion are distinct carried forms
- repeated merge and restore preserve future bounded merge boundary validity
- convenience performance-only index churn does not alter lowered merge candidates

### Required target forms

```rust
pub struct MergeBoundaryWitness {
    pub source_boundary: BranchMutationBoundaryId,
    pub target_boundary: BranchMutationBoundaryId,
    pub merge_base: CheckpointBoundary,
    pub continuity_digest: MergeContinuityDigest,
}

pub struct StructuralMergeJournalSlice {
    pub records: Vec<StructuralMergeRecord>,
    pub source_boundary: BranchMutationBoundaryId,
}

pub struct ProofMinimalOverlapBasis {
    pub overlapping_nodes: CanonicalNodeSet,
    pub overlapping_dependencies: CanonicalDependencySet,
}

pub struct ConservativeOverlapExpansion {
    pub expanded_nodes: CanonicalNodeSet,
    pub expansion_reason: OverlapExpansionReason,
}

pub struct PlannedMergeCandidateSet {
    pub candidates: CanonicalNodeSet,
}

pub struct LoweredMergePlan {
    pub boundary: MergeBoundaryWitness,
    pub source_slice: StructuralMergeJournalSlice,
    pub overlap_basis: ProofMinimalOverlapBasis,
    pub overlap_expansion: ConservativeOverlapExpansion,
    pub candidates: PlannedMergeCandidateSet,
    pub resolution_basis: MergeDecisionBasis,
}
```

### Semantic split of overlap

This is mandatory and must not collapse into one slice concept.

`ProofMinimalOverlapBasis`

- the smallest authority-relevant overlap implied directly by:
  - source journal slice
  - merge-base proof
  - target continuity semantics
- not a heuristic
- not a convenience expansion
- not an execution-ready candidate set

`ConservativeOverlapExpansion`

- the explicit bounded expansion needed for sound reconciliation beyond proof-minimal overlap
- must name why each expansion class is sound and necessary
- must remain bounded by proof-authorized indexes maintained from the same mutation journal semantics

`PlannedMergeCandidateSet`

- final candidate membership
- pure function of:
  - `MergeBoundaryWitness`
  - `StructuralMergeJournalSlice`
  - `ProofMinimalOverlapBasis`
  - `ConservativeOverlapExpansion`
  - proof-authorized bounded indexes only

### Hard merge law

For supported merge flows, candidate construction must be a pure function of lowered merge planning inputs plus canonical proof-authorized indexes whose maintenance is themselves bounded by the same mutation journal semantics.

Forbidden:

- whole-live branch scans
- convenience indexes that effectively encode whole-branch truth
- hidden ambient branch-state discovery during candidate construction

### Proof semantics

`MergeBoundaryWitness`

- producer: branch lifecycle + mutation ledger + merge-base authority proof
- certifies: one source-target-merge-base bounded merge relationship
- invalidated by: mutation past source or target boundary
- replay-verifiable: yes
- reuse model: inspectable until either side mutates past boundary

`StructuralMergeJournalSlice`

- producer: mutation ledger
- certifies: bounded structural source delta since boundary
- invalidated by: further source mutation
- reuse model: inspectable

`ProofMinimalOverlapBasis`

- producer: merge lowering from journal proof and merge-base truth
- certifies: minimal authority-relevant overlap only
- invalidated by: boundary invalidation or changed overlap indexes
- reuse model: planning-only

`ConservativeOverlapExpansion`

- producer: merge lowering using proof-authorized indexes
- certifies: bounded sound expansion beyond minimal overlap
- invalidated by: same as overlap basis
- reuse model: planning-only

### Merge planning chain

1. establish `MergeBoundaryWitness`
2. derive `StructuralMergeJournalSlice`
3. derive `ProofMinimalOverlapBasis`
4. derive `ConservativeOverlapExpansion`
5. derive `PlannedMergeCandidateSet`
6. lower `LoweredMergePlan`
7. execute merge from `LoweredMergePlan` only

### Compile-time enforcement

- merge executor accepts `LoweredMergePlan` only
- candidate-construction helpers require `MergeBoundaryWitness`
- overlap basis, expansion, and final candidates are distinct types
- supported merge variants cannot encode whole-live candidate scope
- unsupported merge families must reject before candidate construction

### Measurement boundaries

Merge planning/execution must expose:

- `source_slice_breadth`
- `proof_minimal_overlap_breadth`
- `conservative_overlap_expansion_breadth`
- `final_candidate_breadth`
- `reconciliation_breadth`
- `boundary_witness_kind`

### Acceptance tests

- `supported_merge_candidate_construction_is_purely_proof_driven`
- `proof_minimal_overlap_and_conservative_expansion_remain_distinct_and_bounded`
- `supported_merge_never_constructs_whole_live_scope`
- `merge_candidate_construction_is_identical_with_and_without_convenience_branch_indexes`
- `restore_after_merge_preserves_future_boundary_validity`
- `unsupported_merge_families_fail_before_candidate_construction`

Implemented crate-level evidence:

- `tests::merge_adoption::merge_branch_uses_branch_local_mutation_scope_instead_of_whole_live_scan`
- `tests::merge_adoption::proof_minimal_overlap_and_conservative_expansion_remain_distinct_and_bounded`
- `tests::merge_adoption::merge_candidate_construction_is_identical_with_and_without_convenience_branch_indexes`
- `tests::merge_adoption::active_restore_reinstates_branch_merge_ledger_boundary_for_later_fast_forward_merge`
- `tests::merge_adoption::repeated_merge_after_target_restore_stays_bounded_and_history_honest`
- `tests::merge_adoption::merge_branch_without_established_journal_boundary_fails_explicitly`

### Closeout criteria

- supported merge flows are always bounded by carried proof
- candidate construction is pure and proof-driven
- whole-live merge is unrepresentable on supported paths

Closeout statement:

- these criteria are satisfied for the supported S9 merge envelope; remaining
  richer merge-expansion work belongs to `S10`, not to `S9.15`

### Certification staging

Compile-time barrier:

- merge executor accepts `LoweredMergePlan` only
- supported merge variants cannot express whole-live candidate scope

Local proof tests:

- `supported_merge_candidate_construction_is_purely_proof_driven`
- `proof_minimal_overlap_and_conservative_expansion_remain_distinct_and_bounded`

Adversarial/property tests:

- overlapping mutation journals under repeated restore/merge churn
- convenience index presence or absence preserves identical candidate construction

Cross-phase equivalence tests:

- restore after merge preserves future boundary validity
- supported merge outcomes remain stable across diagnostics tier changes

Allowed harness debt:

- richer unsupported-family certification may remain open if unsupported flows already reject before candidate construction

## Workstream 3: `S9.9` True Parallel Apply Completion

### Adversarial constraint

Under wide disjoint stages on geometry-style graphs, the runtime must either execute real grouped concurrent apply with identical semantic outcomes to serial execution, or lower honestly to serial execution. It must not advertise full parallelism while funneling the majority of semantic work through a hidden serial phase.

### Current defect

The original defect is now retired on the supported S9 path.

The runtime now provides:

- proof-bearing grouped concurrent lowering through `DisjointApplyProof`
- real grouped concurrent worker packet derivation for proof-safe static stages
- deterministic reduction-only publication through `ConcurrentApplyReductionPlan`
- honest serial lowering with named rejection when a stage would require shared-surface suppression or local rewiring beyond the current proof-safe envelope

### Required target forms

```rust
pub struct DisjointApplyProof {
    pub planning_basis: PlanningAuthorityDigest,
    pub mutation_domain: MutationDomain,
    pub group_footprints: Vec<ApplyFootprint>,
    pub shared_surface_policy: SharedSurfacePolicy,
}

pub enum LoweredApplyPlan {
    Serial(SerialApplyPlan),
    GroupedConcurrent(ConcurrentApplyPlan),
}

pub struct GroupLocalApplyPacket {
    pub authority_delta: AuthorityDeltaPacket,
    pub topology_delta: TopologyDeltaPacket,
    pub replay_packet: LocalReplayPacket,
    pub lineage_packet: LocalLineagePacket,
    pub telemetry_packet: LocalTelemetryPacket,
    pub snapshot_packet: LocalSnapshotPacket,
}

pub struct ConcurrentApplyPlan {
    pub groups: Vec<DisjointApplyGroup>,
    pub proof: DisjointApplyProof,
    pub reduction: ConcurrentApplyReductionPlan,
}

pub struct ConcurrentApplyReductionPlan {
    pub ordering_contract: ReductionOrderingContract,
    pub allowed_work: ReductionWorkClass,
}
```

### Hard concurrency law

This plan adopts the safer model:

- workers may write only to group-local isolated buffers
- all shared runtime surfaces are reduction-only

That means `DisjointApplyProof` does not need to certify global disjointness over every shared surface. It must certify:

- disjointness over worker-writable authority/mutation space
- that all non-disjoint shared surfaces are excluded from worker writes and deferred to deterministic reduction

### Worker and reduction contract

Worker-allowed work:

- group-local authority mutation on disjoint slices
- local dependency reconciliation already proven local
- local topology/artifact/runtime deltas
- local replay/lineage/telemetry packets
- local snapshot/update packets

Reduction-allowed work:

- deterministic publication ordering
- insertion of replay packets into shared retained structures
- lineage publication
- telemetry aggregation
- snapshot commit publication
- merge-relevant derived index publication that cannot be isolated per worker

Reduction-forbidden work:

- broad dependency discovery
- recomputing disjointness
- late execution strategy selection
- majority of semantic authority work
- convenience artifact construction
- hidden rich-path reconstruction

### Parallel equivalence scope

Serial vs concurrent equivalence must cover:

- authoritative state
- replay packet ordering contract
- lineage/provenance graph
- semantically required diagnostic conclusions
- checkpoint eligibility and reconstructability consequences
- downstream merge-relevant truth

Matching only coarse summaries is insufficient.

### Proof semantics

`DisjointApplyProof`

- producer: lowered planning from canonical task footprints and planning basis
- certifies: group concurrency safety for one lowered stage under one mutation domain and shared-surface policy
- invalidated by: changed planning basis, changed footprints, or changed shared-surface policy
- replay-verifiable: indirectly through equivalence and packet-ordering tests
- reuse model: single-use per lowered stage

### Compile-time enforcement

- planner emits `LoweredApplyPlan::Serial` or `LoweredApplyPlan::GroupedConcurrent`
- executor consumes lowered plan variant only
- grouped concurrent path requires `DisjointApplyProof`
- worker code cannot access shared surfaces directly
- reduction consumes only `GroupLocalApplyPacket`s
- `SerialFallback` cannot remain as the semantic truth for grouped concurrent execution

### Measurement boundaries

Planner/apply must expose:

- `group_local_authority_breadth`
- `group_local_packet_breadth`
- `reduction_packet_breadth`
- `reduction_group_count`
- `shared_surface_publication_breadth`
- `parallel_admission_rejection_reason`

### Acceptance tests

- `grouped_concurrent_and_serial_apply_are_equivalent_across_all_semantic_surfaces`
- `workers_write_only_group_local_buffers`
- `reduction_breadth_scales_with_publication_packets_not_semantic_work`
- `no_grouped_concurrent_plan_exists_without_disjoint_apply_proof`
- `unsupported_mutable_engine_parallelism_lowers_to_serial_honestly`

### Closeout criteria

- `FullParallel` means real grouped concurrent apply or does not exist
- reduction is narrow publication, not a backdoor serial semantic engine
- planner, lowered plan, executor, and reporting all agree on execution truth

### Certification staging

Compile-time barrier:

- grouped concurrent execution requires `DisjointApplyProof`
- worker code cannot access shared runtime surfaces directly

Local proof tests:

- `workers_write_only_group_local_buffers`
- `no_grouped_concurrent_plan_exists_without_disjoint_apply_proof`

Adversarial/property tests:

- disjoint geometry-style subgraph workloads across varying batch width
- rejection of unsupported mutable-engine parallelism before execution

Cross-phase equivalence tests:

- grouped concurrent and serial apply are equivalent across authoritative, replay, lineage, and required diagnostic surfaces
- reduction breadth scales with publication packets rather than semantic work

Allowed harness debt:

- throughput slope reporting may remain open if equivalence and breadth counters are already certified locally

### Closeout note

- `S9.9` is closed for the supported S9 envelope
- retired legacy surface: `FullParallel` execution that was semantically serial on the supported path
- supported proof-driven path: `DisjointApplyProof -> LoweredApplyPlan::GroupedConcurrent(ConcurrentApplyPlan) -> GroupLocalApplyPacket -> ConcurrentApplyReductionPlan`
- named crate-level evidence:
  - `tests::adversarial_parallel::full_parallel_splits_wide_stage_into_deterministic_apply_groups`
  - `tests::adversarial_parallel::full_parallel_rewires_dynamic_dependencies_without_losing_parity`
  - `tests::adversarial_parallel::full_parallel_apply_failure_does_not_leak_partial_semantic_state`
  - `tests::adversarial_parallel::full_parallel_policy_matrix_preserves_semantic_artifacts_on_tolerance_heavy_partition_graph`
  - `tests::adversarial_parallel::logically_equivalent_region_orders_produce_identical_provenance_and_replay`
  - `tests::telemetry_contract::full_parallel_honest_serial_apply_emits_group_local_packet_and_reduction_counters`

## Workstream 4: `S9.10` Rollback and Lifecycle Completion

### Adversarial constraint

Under frequent transactions, injected failures, branch switching, branch creation, and repeated restore cycles, rollback and lifecycle transfer must preserve exact pre-operation semantic truth while avoiding eager broad baseline cloning and avoiding heavyweight branch-state capture as a routine orchestration primitive.

### Current state

The supported path is now substantially closed:

- rollback baseline bundles are retired from supported semantic truth
- rollback stages typed packets for config, diagnostics, graph patch rewind, created-node cleanup, and subscriber repair
- branch switching is move-only on the supported lifecycle path
- active restore runs through a typed restore transfer lane rather than a raw branch-state load
- heavy capture witness construction is private to runtime-owned helpers

Remaining hardening is now mostly seal-and-certify work:

- keep all supported lifecycle application on the `BranchLifecycleTransfer` lane
- avoid reintroducing raw packet construction at future callsites
- keep doc and acceptance matrices aligned with the sealed runtime-owned helper boundary

### Rollback truth boundary

After rollback, the runtime must be observationally equivalent to the pre-transaction state with respect to:

- authoritative truth
- semantically required derived truth

Diagnostic richness may be restored:

- exactly, or
- by deterministic rebuild according to policy

Rollback packets must not carry policy-optional forensic richness unless required for deterministic semantic rebuild.

### Required target forms

```rust
pub enum TransactionRollbackPacket {
    GraphPatches(GraphPatchRollbackDelta),
    CreatedNodes(CreatedNodeRollbackDelta),
    SubscriberRepair(SubscriberRepairRollbackDelta),
    Config(ConfigRollbackDelta),
    DiagnosticsRequired(DiagnosticsRollbackDelta),
}

pub struct AuthorityTransferPacket {
    pub branch_id: SignalBranchId,
    pub authority: AuthorityState,
    pub required_derived: DerivedSemanticState,
}

pub struct ExplicitBranchForkPacket {
    pub source_branch: SignalBranchId,
    pub forked_authority: AuthorityState,
    pub forked_required_derived: DerivedSemanticState,
}

pub struct HeavyCaptureWitness(());

pub enum BranchLifecycleTransfer {
    Move(AuthorityTransferPacket),
    Restore(RestoreTransferPacket),
}
```

### Rollback packet grain

Rollback packets are:

- typed minimal inverse authority effects
- subsystem-local
- effect-derived
- ordered only where subsystem causality demands ordering

Rollback packets are not:

- imperative reverse commands
- generic serialized undo scripts
- baseline state bundles with a new name

### Lifecycle transfer semantics

`Move`

- transfer active authority to/from stored branch state
- no duplication implied

`Duplicate`

- explicit branch fork with real second branch-owned truth surface
- only this path duplicates authority

`Restore`

- reconstructive restore from `ReconstructabilityProof`
- not a copy of an arbitrary branch bundle

### Heavy capture policy

Witness-gating alone is not enough unless witness construction is tightly owned.

Rules:

- `HeavyCaptureWitness` construction must be owned by one sealed internal lifecycle module only
- routine branch switch/create/restore paths cannot construct it
- tests may access it only through explicitly test-scoped helper entrypoints
- every heavy capture increments a dedicated runtime counter

### Proof semantics

`TransactionRollbackPacket`

- producer: mutation/effect application
- certifies: minimal inverse authority effect for one subsystem mutation
- invalidated by: application against the wrong authority baseline or wrong transaction order
- replay-verifiable: yes, against effect log and pre/post authority truth
- reuse model: single-use per failed transaction

`AuthorityTransferPacket`

- producer: lifecycle move path
- certifies: single-consumer transfer of authority and required derived truth
- invalidated by: mutation after transfer creation
- reuse model: single-use move-only

`ExplicitBranchForkPacket`

- producer: branch fork path
- certifies: explicit duplication for a second branch-owned truth surface
- invalidated by: mutation after fork packet creation
- reuse model: single-use

### Compile-time enforcement

- rollback/finalize consumes `TransactionRollbackPacket`s only
- branch switch and active restore apply lifecycle state through `BranchLifecycleTransfer`
- branch restore APIs accept `ReconstructabilityProof` only at the public proof boundary
- branch fork APIs accept `ExplicitBranchForkPacket` only
- heavy capture APIs require runtime-owned helper construction of `HeavyCaptureWitness`
- rollback and transfer packets remain move-only unless a real second consumer exists

### Measurement boundaries

Transaction/lifecycle surfaces must expose:

- `rollback_packet_breadth`
- `rollback_packet_count_by_subsystem`
- `move_transfer_count`
- `explicit_fork_count`
- `restore_transfer_count`
- `heavy_capture_count`

### Acceptance tests

- `read_only_and_no_op_transactions_emit_zero_rollback_packets`
- `rollback_packets_restore_pre_transaction_truth_without_baseline_bundles`
- `branch_switch_is_move_only`
- `branch_fork_duplicates_only_explicit_branch_owned_truth`
- `branch_restore_requires_reconstructability_proof`
- `heavy_capture_requires_internal_witness_and_increments_counter`
- `branch_lifecycle_telemetry_distinguishes_fork_move_and_restore_packets`

### Closeout criteria

- rollback is effect-derived inverse authority restoration, not generic undo
- branch lifecycle operations are type-separated and cost-honest
- heavy capture is narrow, counted, and non-routine
- no supported path can construct `HeavyCaptureWitness` outside runtime-owned helpers
- no supported active lifecycle apply path can bypass `BranchLifecycleTransfer`

### Certification staging

Compile-time barrier:

- rollback/finalize consumes `TransactionRollbackPacket`s only
- branch switch/fork/restore each accept their own transfer packet family only
- heavy capture requires sealed witness construction

Local proof tests:

- `read_only_and_no_op_transactions_emit_zero_rollback_packets`
- `branch_switch_is_move_only`
- `branch_restore_requires_reconstructability_proof`

Adversarial/property tests:

- failure injection at multiple commit phases with repeated branch switch/fork/restore churn
- allocator/catalog churn under rollback-heavy workloads

Cross-phase equivalence tests:

- rollback restores pre-transaction authority and semantically required derived truth exactly
- deterministic diagnostic rebuild after rollback does not alter semantic truth

Allowed harness debt:

- long-duration branch churn harness may remain open if packet counters and exact rollback equivalence tests already land

## Cross-Phase Measurement and Certification

### Required measurement boundaries

These are mandatory and must be surfaced at subsystem facades and phase boundaries:

Reconstructability:

- `journal_replay_span`
- `journal_suffix_breadth`
- `restore_authority_breadth`
- `restore_required_derived_breadth`
- `restore_diagnostic_richness_breadth`

Merge:

- `boundary_witness_kind`
- `source_slice_breadth`
- `proof_minimal_overlap_breadth`
- `conservative_overlap_expansion_breadth`
- `final_candidate_breadth`
- `reconciliation_breadth`

Parallel apply:

- `lowered_apply_kind`
- `group_local_authority_breadth`
- `group_local_packet_breadth`
- `reduction_packet_breadth`
- `shared_surface_publication_breadth`
- `parallel_admission_rejection_reason`

Rollback/lifecycle:

- `rollback_packet_breadth`
- `rollback_packet_count_by_subsystem`
- `move_transfer_count`
- `explicit_fork_count`
- `restore_transfer_count`
- `heavy_capture_count`

### Required adversarial certification scenarios

- checkpoint + bounded journal reconstruction equivalence under long branch/snapshot churn
- merge-after-restore boundedness under overlapping mutation journals
- serial vs concurrent equivalence on disjoint geometry-style subgraphs
- repeated branch switch/fork/restore under allocator/catalog churn
- rollback exactness under failures injected at multiple commit phases
- negative-space certification:
  - unsupported merge without boundary witness fails before candidate construction
  - concurrent apply without `DisjointApplyProof` cannot compile/lower
  - restore without `ReconstructabilityProof` cannot compile/lower
  - routine lifecycle paths cannot call heavy capture helpers

## Document and Closeout Updates

When implemented, the following docs must be updated together:

- [signal_architecture2.md](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/signal_architecture2.md)
- [test-requirements.md](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/test-requirements.md)
- [worth_signal_adversarial_testing_matrix.md](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/engineering/worth_signal_adversarial_testing_matrix.md)
- [worth_signal_fintech_certification_matrix.md](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/engineering/worth_signal_fintech_certification_matrix.md)

Required closeout statement:

- `S9.12`, `S9.15`, `S9.9`, and `S9.10` are complete only when supported execution is proof-driven, no forbidden fallback path remains representable, and the certification suite proves breadth, equivalence, and exactness through named counters and proof-bearing runtime surfaces.

## Module Ownership and Facade Rule

To prevent proof construction from leaking across unrelated modules:

- each workstream must have one owning module subtree
- proof constructors must be private to the owning subtree unless the proof is itself a public facade artifact
- sealed witness constructors must live in exactly one internal module
- facade surfaces may expose proof consumption, counters, and typed failure, but must not expose internal assembly shortcuts

Recommended ownership split:

- `S9.12`: reconstructability/checkpoint subtree
- `S9.15`: merge planning and merge execution subtree
- `S9.9`: planner/apply lowering plus concurrent apply execution subtree
- `S9.10`: transaction rollback and lifecycle transfer subtree

If a proof is created opportunistically in a caller because the owning module did
not provide a constructor, the architecture is incomplete and the caller code is
wrong.

## Assumptions and Defaults

- explicit failure remains preferable to broad fallback whenever proof is missing
- proof-authorized indexes are allowed only when maintained from the same canonical semantics they serve
- diagnostic policy remains strictly post-semantic and may not affect authority, required derived truth, or admissibility
- `S10` is out of scope for this document; no merge-forward feature expansion belongs in this batch
