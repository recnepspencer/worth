# worth-signal Milestone 3

> **Status:** Proposed engineering spec
>
> **Roadmap parent:** [performance.md](./performance.md)
>
> **Related implementation surfaces:**
> - [apply.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/evaluation/engine/apply.rs)
> - [serial_batch.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/planner/apply/serial_batch.rs)
> - [entries.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/storage/entries.rs)
> - [dependency.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/dependency.rs)
> - [proof.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/proof.rs)

## Goal

Milestone 3 converts dependency snapshot handling from a replacement-centric
optimization branch into a proof-carrying architecture with compile-time
separation between:

- canonical snapshot shape authority
- stable-shape version delta execution
- structural replacement execution

The implementation goal is not "more uses of `VersionOnly`." The goal is to
make the hot path structurally unable to take the wrong snapshot-commit path
once shape stability has been proven.

The runtime must treat stable-shape churn as its own first-class semantic case
with explicit types, sealed constructors, batch-safe commit forms, and counters
that prove the version-only path is actually dominating the intended workloads.

## Why This Milestone Exists

The current repo already contains `DependencySnapshotUpdate::VersionOnly`, but
it is still too weak architecturally:

- shape stability is discovered opportunistically in `apply.rs`
- version-only updates carry raw version vectors without a proof of which shape
  they belong to
- commit code still reconstructs full snapshots from generic update enums
- batch forms do not encode whether entries are shape-stable or shape-replacing
- restore/replay integrity relies on convention rather than a stronger proof
  chain

The existing scan in `apply.rs` is already doing the right conceptual work. It
compares current runtime dependencies against previous committed snapshot order
and records version-only differences when sort keys match. The problem is not
that the runtime cannot discover stable shape. The problem is that the runtime
discovers it and then immediately collapses that discovery into an unbound
`Vec<u64>`.

This milestone closes that gap.

## Concrete Rust Surface

Milestone 3 should be implemented as a small set of explicit Rust-facing
surfaces rather than as an informal set of helper branches.

Primary dependency-module types:

```rust
pub struct DependencySnapshotShape { /* private fields */ }
pub struct SnapshotShapeHandle(/* private */);
pub struct DependencySnapshotShapeStore { /* graph-owned derived state */ }

pub struct DependencyInputScan { /* single-pass scan facts */ }
pub struct StableShapeSnapshotBasis { /* private fields */ }
pub struct VersionVector { /* private fields */ }

pub struct VersionOnlySnapshotUpdate { /* private fields */ }
pub struct ReplacementSnapshotUpdate { /* private fields */ }

pub enum CommittedSnapshotUpdate {
    VersionOnly(VersionOnlySnapshotUpdate),
    Replace(ReplacementSnapshotUpdate),
}

pub enum SnapshotChangeKind {
    Unchanged,
    StableShapeVersionOnly,
    StructuralReplace,
}
```

Primary constructors and lowering APIs:

```rust
impl DependencySnapshot {
    pub fn shape(&self) -> DependencySnapshotShape;
}

impl DependencySnapshotShape {
    pub(crate) fn structural_hash(&self) -> u64;
    pub(crate) fn intern(
        &self,
        store: &mut DependencySnapshotShapeStore,
    ) -> SnapshotShapeHandle;
}

impl DependencyInputScan {
    pub(crate) fn from_runtime_dependencies(/* graph + node context */) -> Result<Self, SignalError>;
    pub(crate) fn into_resolved_input(
        self,
        shape_store: &mut DependencySnapshotShapeStore,
    ) -> ResolvedDependencyInput;
}

impl StableShapeSnapshotBasis {
    pub(crate) fn prove(
        scan: &DependencyInputScan,
        shape_store: &mut DependencySnapshotShapeStore,
    ) -> Option<Self>;
}

impl VersionOnlySnapshotUpdate {
    pub(crate) fn from_basis_and_versions(
        basis: StableShapeSnapshotBasis,
        versions: VersionVector,
    ) -> Self;
}

impl ReplacementSnapshotUpdate {
    pub(crate) fn from_snapshot(
        snapshot: DependencySnapshot,
        shape_store: &mut DependencySnapshotShapeStore,
    ) -> Self;
}
```

Primary graph/storage entrypoints:

```rust
impl SignalGraph {
    pub(crate) fn commit_snapshot_update(
        &mut self,
        node: NodeId,
        update: CommittedSnapshotUpdate,
    ) -> Result<SnapshotDeltaRecord, SignalError>;

    pub(crate) fn apply_stable_shape_snapshot_batch_commit(
        &mut self,
        commit: StableShapeSnapshotBatchCommit,
    ) -> Result<(), SignalError>;

    pub(crate) fn apply_mixed_snapshot_batch_commit(
        &mut self,
        commit: MixedSnapshotBatchCommit,
    ) -> Result<(), SignalError>;
}
```

Planner/evaluation boundary:

```rust
pub(crate) struct ResolvedDependencyInput {
    pub context: DependencyInputContext,
    pub update: CommittedSnapshotUpdate,
    pub delta: SnapshotDeltaRecord,
    pub meaningful_input_changes: u32,
}
```

This is the intended ownership split:

- `dependency.rs` owns proof-bearing snapshot update construction
- `apply.rs` owns runtime dependency scan and resolution into proof-bearing forms
- `serial_batch.rs` owns stage-level homogeneous stable-shape batch lowering
- `entries.rs` owns execution of commit variants, not proof discovery

## Adversarial Constraint

Stable-shape churn must not keep paying full snapshot rebuild cost, and any
optimization must preserve:

- replay truth
- restore truth
- merge truth
- subscriber integrity
- deterministic canonical ordering
- branch structural conflict correctness

The hot path must not silently diverge from the cold path. Any optimized path
must be the same semantic path with stronger proof-carrying structure, not an
alternate interpretation.

## Core Architecture Rule

The proof chain must be:

`DependencySnapshot`
-> `DependencyInputScan`
-> `StableShapeSnapshotBasis`
-> `VersionVector`
-> `VersionOnlySnapshotUpdate`
-> `StableShapeSnapshotBatchCommit`

and separately:

`DependencySnapshot`
-> `DependencyInputScan`
-> `ReplacementSnapshotUpdate`
-> `MixedSnapshotBatchCommit`

This chain must be the only legal route for stable-shape version-only commit.

Once a stable-shape proof exists, execution must consume that proof-bearing form
directly. No later phase should re-decide whether the update is version-only or
replacement.

## Implementation Changes

### 1. Introduce explicit snapshot authority and proof types

Add new domain types under the dependency snapshot subsystem to separate
structural truth from version churn truth.

Required new types:

- `SnapshotShapeHandle`
  A stable identifier for canonical dependency membership/order. This is not a
  storage alias and not a `DependencySnapshotId`. It identifies the structural
  shape only.
- `DependencySnapshotShape`
  Canonical ordered shape authority: source/aspect/scope membership without
  cached versions.
- `StableShapeSnapshotBasis`
  Proof object tying:
  - node
  - current `DependencySnapshotId`
  - `SnapshotShapeHandle`
  - ordered entry count
  This type means "the current committed snapshot for this node has this exact
  canonical shape."
- `VersionVector`
  Compact ordered version payload aligned to one proven shape. Constructor must
  be sealed so only shape-checked builders can create it.
- `VersionOnlySnapshotUpdate`
  A proof-carrying version-only update containing:
  - `basis: StableShapeSnapshotBasis`
  - `versions: VersionVector`
- `ReplacementSnapshotUpdate`
  Explicit replacement path containing:
  - canonical next snapshot
  - optionally the derived next shape handle
- `CommittedSnapshotUpdate`
  Sum type for the commit boundary only:
  - `VersionOnly(VersionOnlySnapshotUpdate)`
  - `Replace(ReplacementSnapshotUpdate)`

Semantics safeguards:

- `VersionVector` must not be constructible from arbitrary `Vec<u64>` outside
  the dependency module.
- `StableShapeSnapshotBasis` must be constructible only by a function that
  compares the current runtime dependency view against the currently committed
  snapshot shape.
- `VersionOnlySnapshotUpdate` must require a `StableShapeSnapshotBasis`;
  callers cannot "declare" stable shape by hand.
- `ReplacementSnapshotUpdate` is the only legal route when membership/order
  changes.

This change makes the invalid state "version-only update against the wrong
shape" unrepresentable outside the module.

### 2. Split shape authority from full snapshot authority

Refactor the snapshot subsystem so structural shape is a first-class authority
object rather than an implicit property of `DependencySnapshot.entries()`.

Required storage model changes:

- `DependencySnapshot` remains the replay/restore authority for full snapshot
  truth.
- `DependencySnapshotShapeStore` is added to intern canonical ordered shapes and
  return `SnapshotShapeHandle`.
- committed node state continues to hold `DependencySnapshotId`; do not add a
  second node field yet.
- shape handle is recovered from the snapshot store at the proof boundary, not
  stored ad hoc in unrelated subsystems during this milestone.

Required API additions in the dependency module:

- `DependencySnapshot::shape() -> DependencySnapshotShape`
- `DependencySnapshotShape::handle(&mut DependencySnapshotShapeStore) -> SnapshotShapeHandle`
- `StableShapeSnapshotBasis::prove(node, previous_snapshot, scan) -> Option<Self>`
- `VersionOnlySnapshotUpdate::from_proven_basis(basis, versions) -> Self`

Required API removals/restrictions:

- stop exposing generic "raw version-only delta from any `Vec<u64>`" as the
  primary builder
- `DependencySnapshotUpdate::between(...)` must no longer be the dominant
  hot-path constructor for stable-shape churn
- keep a compatibility adapter during migration, but mark it as
  replacement-oriented/fallback-only inside the dependency module

The architecture rule is:

- `between(...)` may remain as a slow fallback compatibility constructor
- hot apply/reconcile paths must instead choose between:
  - `prove stable shape -> build version-only update`
  - `build replacement update`

### 3. Make proof construction single-pass and consume scan output

`StableShapeSnapshotBasis::prove()` must not rescan dependencies and must not
re-discover facts that the dependency scan already established.

The current scan in `apply.rs` already computes:

- dependency count agreement
- ordered sort-key agreement
- liveness eligibility
- changed-version count
- ordered stable-shape version list

Milestone 3 must formalize that existing computation rather than duplicate it.

Add a new intermediate type:

- `DependencyInputScan`
  a single-pass scan result produced during dependency traversal containing:
  - node
  - previous snapshot id
  - previous entry count
  - ordered dependency count
  - ordered sort-key match status
  - live-source eligibility status
  - changed-version count
  - ordered next versions
  - optional next canonical replacement snapshot material when shape is not
    stable
  - optional structural hash accumulator for the shape

Rules:

- `DependencyInputScan` is produced once per dependency-input build.
- `StableShapeSnapshotBasis::prove(...)` consumes `DependencyInputScan`.
- `prove(...)` may consult the shape store, but it must do so from scan-derived
  structural facts rather than by re-traversing dependencies.
- no second dependency walk is permitted on the hot path to recover proof.

This must be explicit because the current opportunistic check is effectively
free as a byproduct of the scan. The proof-carrying design must preserve that
property.

### 4. Specify `DependencySnapshotShapeStore` ownership, interning, and lifecycle

The shape store is new derived state. Its ownership and lifecycle must be
defined so it does not become an unbounded cache with ambiguous authority.

Milestone 3 defaults:

- `DependencySnapshotShapeStore` lives inside `SignalGraph` next to dependency
  snapshot storage.
- it is graph-owned derived state, not transactional state and not authority
  state.
- it interns canonical ordered structural shapes, not full versioned snapshots.
- `SnapshotShapeHandle` is obtained by:
  - computing a structural hash during the dependency scan or from canonical
    snapshot shape extraction
  - consulting the shape store using the hash plus collision-safe canonical
    shape equality
- interning must not require reconstructing full `DependencySnapshot` values
  when the shape already exists in scan output.

Lifecycle rules:

- the shape store is rebuildable from committed snapshot authority alone
- shape-store contents are not part of replay authority, branch authority, or
  merge authority
- no other subsystem may treat `SnapshotShapeHandle` as canonical truth in the
  absence of the committed snapshot
- compaction or future maintenance may rebuild or prune the shape store without
  altering graph meaning

Milestone 3 does not require eviction policy implementation, but it does require
an explicit classification:

- the shape store is derived state
- unbounded growth is acceptable only temporarily if rebuildability is preserved
- if shape count growth proves significant, future maintenance work can prune or
  rebuild it from snapshot authority

### 5. Replace the current dependency-input pipeline, do not layer on top of it

Milestone 3 must not introduce new proof-bearing forms while keeping the old
generic `EffectDependencyInputs` path as the real runtime path.

Current state:

- `serial_batch.rs` pre-builds dependency inputs with
  `collect_effect_dependency_inputs_iter`
- `apply.rs` builds `EffectDependencyInputs`
- those inputs currently contain generic `DependencySnapshotUpdate`

Required redesign:

- the serial batch pre-build phase must produce proof-bearing resolved snapshot
  input forms instead of the old generic shape
- `EffectDependencyInputs` must be replaced or narrowed so it carries
  `CommittedSnapshotUpdate`, not the old generic `DependencySnapshotUpdate`
- the stage-level dependency-input prebuild is the canonical place where
  stable-shape proof is established for batch execution

New staged forms:

- `StableShapeDependencyInput`
  contains:
  - `StableShapeSnapshotBasis`
  - `VersionVector`
  - exact changed-version count
- `ReplacementDependencyInput`
  contains:
  - canonical next snapshot
  - exact changed-entry count
- `ResolvedDependencyInput`
  enum used only after proof resolution:
  - `StableShape(StableShapeDependencyInput)`
  - `Replace(ReplacementDependencyInput)`

Required hot-path rules:

- dependency scan must compare current dependencies against previous committed
  snapshot shape once
- if all sort keys match and live-source eligibility holds, construct
  `StableShapeDependencyInput`
- if not, construct `ReplacementDependencyInput`
- only the stable-shape branch may build `VersionVector`
- only the replacement branch may build a new canonical `DependencySnapshot`

The stage then decides:

- if every resolved input in the stage is `StableShape`, lower to
  `StableShapeSnapshotBatchCommit`
- otherwise lower to `MixedSnapshotBatchCommit`

This makes stable-shape batch homogeneity a stage-level proof, not an ad hoc
per-entry observation.

### 6. Redesign pending snapshot and batch commit types around semantic cases

Current batch structures carry `DependencySnapshotUpdate` too early and too
generically. Replace that with proof-bearing batch forms.

Required new batch forms in the proof layer:

- `PendingStableShapeSnapshotCommit`
  contains `node`, `VersionOnlySnapshotUpdate`, `SnapshotDeltaRecord`
- `PendingReplacementSnapshotCommit`
  contains `node`, `ReplacementSnapshotUpdate`, `SnapshotDeltaRecord`
- `PendingSnapshotCommit`
  enum over the two
- `StableShapeSnapshotBatchCommit`
  stage batch containing only stable-shape entries
- `MixedSnapshotBatchCommit`
  fallback/general batch for mixed stage contents

Commit policy:

- serial staged path must produce `StableShapeSnapshotBatchCommit` whenever
  every pending snapshot in the stage is proven stable-shape
- mixed stages may use `MixedSnapshotBatchCommit`
- batch commit code must not collapse these forms back into one generic path
  before the storage boundary

This is the key compile-time enforcement improvement:

- the stable-shape batch commit function accepts only
  `StableShapeSnapshotBatchCommit`
- therefore it cannot receive replacement entries by accident
- mixed fallback is explicit in the type system rather than hidden in per-entry
  matching

### 7. Derive semantic change kind from the commit variant

`SnapshotChangeKind` must not be independently writable.

Add:

- `SnapshotChangeKind`
  - `Unchanged`
  - `StableShapeVersionOnly`
  - `StructuralReplace`

Rules:

- `StableShapeVersionOnly` is derived from
  `PendingSnapshotCommit::StableShape(...)`
- `StructuralReplace` is derived from
  `PendingSnapshotCommit::Replace(...)`
- `Unchanged` is derived from explicit no-op construction
- `SnapshotDeltaRecord` may carry `change_kind`, but it must only be created by
  variant-specific constructors so semantic kind cannot desynchronize from the
  actual update variant

This avoids the exact class of bug where diagnostics and execution silently
disagree about what happened.

### 8. Add a storage commit path optimized for proven stable-shape updates

Refactor `entries.rs` and the dependency storage subsystem to commit version-only
updates without generic snapshot reconstruction as the primary path.

Required new storage APIs:

- `apply_stable_shape_snapshot_batch_commit(commit: StableShapeSnapshotBatchCommit)`
- `apply_mixed_snapshot_batch_commit(commit: MixedSnapshotBatchCommit)`
- `commit_version_only_update(update: VersionOnlySnapshotUpdate, previous: &DependencySnapshot) -> DependencySnapshotId`
- `commit_replacement_update(update: ReplacementSnapshotUpdate) -> DependencySnapshotId`

Implementation rules:

- stable-shape commit path must assume proof validity and operate directly on
  previous snapshot + ordered version payload
- replacement path remains canonicalization/interner-based
- storage counters must increment per path before any generic fallback logic
- if a stable-shape batch commit needs per-entry structural checking, the design
  is wrong; the checking belongs in proof construction, not execution

Expected physical behavior:

- the stable-shape commit path still materializes an authoritative immutable
  `DependencySnapshot` for replay/restore
- but it does so through the shape-aligned path, not through generic
  replace-oriented branching
- interning should deduplicate resulting snapshots as before
- the shape store may be used to avoid re-deriving structural identity from the
  full snapshot each time

### 9. Mechanically demote `between()` to compatibility/fallback status

`DependencySnapshotUpdate::between(...)` may remain temporarily, but it must no
longer be an invisible architectural escape hatch.

Required enforcement:

- all hot-path evaluation and staged batch paths must migrate off `between()`
- `between()` must increment a dedicated counter every time it is called
- perf tests must assert that representative stable-shape churn workloads do not
  route through `between()` except for genuine structural replace cases or
  compatibility-only cold paths
- where practical, reduce visibility of `between()` so only dependency-module
  fallback surfaces can call it

This gives the codebase a mechanical signal if the old generic constructor
starts creeping back into the hot path.

### 10. Strengthen delta/accounting semantics to match the new proof chain

`SnapshotDeltaRecord` currently tracks counts only. Keep that record for
observability compatibility, but extend it through variant-safe constructors.

Required semantics:

- `StableShapeVersionOnly` means shape is proven unchanged and only versions
  differ
- `StructuralReplace` means membership/order changed or proof was unavailable
- `Unchanged` means exact no-op
- the commit path must not reclassify these cases

This preserves honest decision logs and makes diagnostics/replay aware of the
semantic path taken.

### 11. Preserve replay, restore, merge, and subscriber semantics explicitly

Milestone 3 must not only optimize evaluation apply. It must keep every
downstream consumer semantically aligned.

Required invariants:

- restore batches derived from two graphs must classify stable-shape vs
  replacement exactly the same way as live apply would
- branch merge conflict detection must continue to treat structural snapshot
  mismatches as structural, not version-only
- subscriber integrity logic must not consume shape handles as authority;
  subscriber truth remains derived from actual topology/snapshot authority
- reuse basis and structural dependency basis must remain aligned with committed
  snapshot identity semantics

Required code updates:

- restore derivation paths that currently call
  `DependencySnapshotUpdate::between(...)` should migrate to a new proof-aware
  constructor that emits `CommittedSnapshotUpdate`
- merge/adoption tests that already mention "narrow snapshot delta path" must be
  updated to assert the new semantic classification instead of just the old enum
  shape
- any code that treats `DependencySnapshotId` as a structural-shape proxy must
  remain unchanged unless it is explicitly verified safe; do not conflate
  snapshot instance identity with shape identity in this milestone

### 12. Add compile-time sealing and visibility constraints

To enforce the architecture mechanically:

- constructors for `StableShapeSnapshotBasis`, `VersionVector`, and
  `VersionOnlySnapshotUpdate` must be `pub(crate)` or sealed to the dependency
  module
- fields for those types must remain private
- external code may read counters and summaries, but may not synthesize
  proof-bearing forms
- the stable-shape batch commit function must not accept the generic enum
- planner/evaluation code should depend on facade-level constructors, not
  internal shape-matching helpers
- keep exactly one public facade surface for snapshot proof construction and
  commit lowering

This is the milestone's compile-time guarantee:

wrong snapshot-update semantics should fail to compile, not merely fail a
runtime assertion.

## Integration Plan

### Phase 1: Dependency subsystem proof model

Build the dependency-module types and storage abstractions first:

- `DependencySnapshotShape`
- `SnapshotShapeHandle`
- `DependencySnapshotShapeStore`
- `StableShapeSnapshotBasis`
- `VersionVector`
- `VersionOnlySnapshotUpdate`
- `ReplacementSnapshotUpdate`
- `CommittedSnapshotUpdate`
- variant-safe `SnapshotDeltaRecord` constructors

Exit condition:

- the dependency module can express stable-shape and replacement updates as
  separate proof-bearing forms
- raw generic version-only construction is sealed away from external callers

### Phase 2: Single-pass dependency scan and proof construction

Refactor dependency-input building so scan output is the proof source:

- introduce `DependencyInputScan`
- convert current opportunistic shape-stable loop into proof construction logic
- ensure `prove()` consumes scan output without a second dependency traversal

Exit condition:

- dependency input building is single-pass
- stable-shape proof is explicit and recoverable from scan output

### Phase 3: Replace the current dependency-input pipeline

Refactor apply/planner batch code:

- replace generic `EffectDependencyInputs` payloads with proof-bearing resolved
  forms
- update serial stage pre-build in `serial_batch.rs`
- add stage-level homogeneous stable-shape batch lowering

Exit condition:

- stage execution uses `ResolvedDependencyInput`
- batch shape homogeneity is represented in types

### Phase 4: Storage commit split

Refactor storage commit paths:

- add stable-shape-specific batch commit
- add mixed/fallback batch commit
- remove generic reclassification inside commit execution

Exit condition:

- proven stable-shape batches never go through generic replacement-oriented
  commit logic

### Phase 5: Cold-path alignment

Update:

- restore derivation
- merge/adoption classification
- replay-sensitive surfaces
- telemetry and diagnostics summaries

Exit condition:

- hot and cold paths use the same semantic constructors and classifications

### Phase 6: Mechanical enforcement and perf proof

Finalize:

- visibility restrictions
- fallback counters
- perf assertions
- certification tests

Exit condition:

- the architecture is mechanically enforced
- representative churn profiles prove the stable-shape path dominates

## Test Plan

### Semantic certification

Add tests that prove the proof chain, not just behavior:

- stable-shape dependency churn produces `StableShapeVersionOnly`
  classification and never enters replacement commit
- any membership addition/removal/reordering forces `StructuralReplace`
- dead-source or invalid-liveness conditions break stable-shape proof and force
  replacement
- version-only path reconstructs the same authoritative snapshot as full
  replacement would
- replay from committed artifacts is identical regardless of whether live
  execution used stable-shape or replacement path
- restore batch derivation classifies updates identically to live apply for the
  same before/after snapshots

### Compile-time / architectural tests

Add tests or module-visibility assertions that enforce:

- `VersionOnlySnapshotUpdate` cannot be built from raw versions outside the
  dependency module
- stable-shape batch commit cannot accept replacement entries
- mixed batch commit is required when stage entries are heterogeneous
- batch types preserve node uniqueness and stage order guarantees

### Counter and performance proof tests

Add exact counter assertions for:

- `version_only_snapshot_update_count`
- `shared_snapshot_replacement_count`
- new counters:
  - `stable_shape_snapshot_proof_count`
  - `stable_shape_snapshot_proof_failure_count`
  - `stable_shape_batch_commit_count`
  - `structural_replace_batch_commit_count`
  - `snapshot_shape_reuse_count`
  - `snapshot_between_fallback_count`

Perf acceptance:

- staged rotating-window profiles show lower `dependency_input_build_nanos`
- lower `dependency_reconcile_nanos` where snapshot-adjacent work is involved
- lower `snapshot_batch_commit_nanos`
- representative stable-shape churn workloads show low or zero
  `snapshot_between_fallback_count`
- no regression in restore, merge adoption, branch replay, or subscriber
  integrity representative lanes

### Required regression lanes

Run and keep green:

- full serial library sweep
- full parallel library sweep
- ignored perf suite with `--test-threads=1`
- snapshot restore and dependency restore batch tests
- merge adoption tests that exercise stable-shape reconciliation
- parity tests for retained vs reconstructed artifacts

## Assumptions and Defaults

- `DependencySnapshot` remains the authoritative replay/restore artifact for
  this milestone; we are not replacing it with a pure shape-plus-delta store
  yet.
- `SnapshotShapeHandle` is a structural-shape identifier only; it must never be
  treated as equivalent to `DependencySnapshotId`.
- The stable-shape path is only valid when ordered dependency membership matches
  exactly; this milestone will not introduce tolerance for reorder-equivalent
  but non-canonical inputs.
- Stage batching will prefer homogeneous stable-shape commit when possible and
  fall back to explicit mixed commit when not.
- Compatibility adapters may exist temporarily, but all hot-path code in
  evaluation/planner/storage must migrate to the proof-bearing constructors
  before the milestone is considered complete.
- `DependencySnapshotShapeStore` is derived state and must remain rebuildable
  from authoritative snapshot state.
- `prove()` must remain single-pass relative to dependency traversal; no second
  dependency walk is acceptable on the hot path.
