# M10.0 Checklist: Merge Inventory And Replacement Cut Line

> **Status:** Active
>
> **Parent spec:** [milestone-10-plan.md](./milestone-10-plan.md)

## Purpose

This document is the concrete execution artifact for `M10.0`.

It exists to prevent the milestone from starting â€œin the middleâ€ by making the
current merge substrate explicit:

- what exists now
- what is preserved
- what is extended
- what is replaced
- what is deleted
- where current hardcoded semantics live

This is the cut line for S10. Pre-S10 merge semantics are prototype semantics.
They may be replaced directly rather than compatibility-layered.

## 1. Current Merge Surface Inventory

### 1.1 Merge module files

Current file set under
`crates/worth-signal/src/logic/transaction/runtime/state/merge/`:

- `adoption.rs`
- `conflict.rs`
- `core.rs`
- `execute.rs`
- `journal.rs`
- `mod.rs`
- `plan.rs`
- `policy.rs`
- `result.rs`

### 1.2 Runtime orchestration file

Current branch-merge orchestration is centered in:

- [merge_runtime.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/branching/merge_runtime.rs)

This file currently contains both:

- bounded merge planning substrate logic
- semantic policy selection and conflict auto-resolution logic

S10 must preserve the first and replace the second.

### 1.3 Public and semi-public exports

Current merge-related exports appear in:

- [merge/mod.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/merge/mod.rs)
- [logic/transaction/mod.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/mod.rs)
- [facade.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/facade.rs)

Current exported merge family includes:

- request and coarse merge-mode enums
- conflict evidence and conflict-resolution records
- mutation journal types
- lowered merge plan
- result and execution summary envelopes
- coarse reconciliation policy enums

## 2. Preserve / Extend / Replace / Delete

### 2.1 Preserve unchanged

These are part of the S9.15 bounded merge floor and should remain conceptually
intact unless implementation details force small refactors:

- `MergeBoundaryWitness`
- `MergeBoundaryWitnessKind`
- `StructuralMergeJournalSlice`
- `BranchMutationJournalSlice`
- `ProofMinimalOverlapBasis`
- `ConservativeOverlapExpansion`
- `PlannedMergeCandidateSet`
- `BranchMergeCounters`

Why preserved:

- they encode bounded candidate discovery
- they already align with test-requirements closeout language
- they are the right proof-bearing backbone for S10

### 2.2 Preserve but extend

These should remain, but will need new fields or companion artifacts:

- `LoweredMergePlan`
- `BranchMergeExecutionSummary`
- `BranchMergeResult`
- `BranchConflictResolutionPlan`
- `BranchMergeConflictEvidence`
- `MergeNodeMap`
- `NodeMergePlan`

Expected extension direction:

- schema-registry provenance
- frozen strategy-registry provenance
- lowered strategy bundle
- per-family decision summaries
- replay-visible semantic versioning

### 2.3 Replace

These are structurally too coarse or too hardcoded to remain the primary
semantic control plane:

- `BranchMergeReconciliationPolicy`
- `ExistingTargetMergePolicy`
- `SourceOnlyMergePolicy`
- `ConflictMergePolicy`
- `BranchMergeStrategy`

Replacement direction:

- family-specific strategy registrations
- frozen strategy registry
- schema-owned merge semantics
- lowered strategy bundle attached to `LoweredMergePlan`

Notes:

- some names may survive as compatibility-free aliases during the refactor, but
  they are not the target architecture

### 2.4 Delete or demote from authoritative role

The following patterns must disappear from the supported authority path:

- planner-local inline construction of reconciliation policy
- executor-visible semantic branching based on coarse enums
- implicit storage-identity-first merge semantics as the only path
- merge-base semantics inferred only from ancestry without named lowered policy

Concrete deletion target:

- the current â€œpolicy assembled inline in `merge_runtime.rs`â€ approach

## 3. Hardcoded Semantic Decisions To Remove

### 3.1 Inline reconciliation policy construction

Current hardcoding in
[merge_runtime.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/state/branching/merge_runtime.rs):

- `existing_target: PreserveEquivalentOtherwiseAdoptSource`
- `source_only: IntroduceAdoptableSkipNonAdoptable`
- `conflict: ResolveSourceStateWhenStructureMatches`

Why it must go:

- this is planner-owned semantics, not schema-owned semantics
- it cannot vary by host domain or schema scope
- it is not frozen at runtime construction

### 3.2 Merge-kind to merge-strategy mapping

Current hardcoding:

- `FastForward -> AdoptSourceHead`
- `Applied -> AdoptSourceSubset`
- `ConflictResolved -> RebaseSourceOntoTarget`

Why it must go:

- this conflates runtime outcome classification with semantic execution policy
- strategy should be lowered from declarations, not inferred from outcome enum

### 3.3 Identity matching by storage-node overlap

Current hardcoding:

- planning effectively treats `target_graph.is_alive(source_node)` and direct
  node-id overlap as the only identity basis on the supported path

Why it must go:

- S10 requires multiple identity bases
- identity evidence must be bounded and declared, not implicit

### 3.4 Conflict auto-resolution admissibility

Current hardcoding:

- `can_auto_resolve_conflicts(...)` encodes which conflict families are
  admissible for automatic resolution

Why it must go:

- admissibility belongs to lowered strategy identity, not a fixed function over
  coarse policy enums

### 3.5 Conflict-kind to supported-strategy mapping

Current hardcoding:

- `conflict_resolution_requirements_for_kind(...)`
- `conflict_resolution_strategies_for_kind(...)`

Why it must go:

- these mappings must become family registrations and lowered strategy outputs,
  not fixed runtime tables

## 4. Current File Ownership Assessment

### 4.1 Files that should remain responsibility owners

- `journal.rs`
  owner of branch-carried merge proof and journal-derived candidate truth
- `plan.rs`
  owner of lowered merge packet families
- `result.rs`
  owner of canonical merge result and execution summary envelopes
- `execute.rs`
  owner of node adoption/materialization mechanics only

### 4.2 Files whose responsibility must shrink

- `merge_runtime.rs`
  should stop owning semantic resolution and become orchestrator over lowered
  proof-bearing forms
- `policy.rs`
  should stop being the primary semantic declaration surface and either become:
  - a transitional compatibility file during refactor
  - or be replaced by strategy-family modules entirely

### 4.3 New ownership zones required by S10

New subsystem expected:

- `merge/strategies/`
  owner of strategy descriptor, registration, frozen registry, and
  family-specific lowering

New subsystem expected:

- `schema/`
  owner of first-class merge schema registry and scope authority

## 5. Public API Cut Line

### 5.1 Merge exports that should remain public in some form

- merge request/result envelopes
- merge planning/report artifacts
- branch merge counters
- conflict evidence and execution summaries

### 5.2 Merge exports that should become internal or be replaced

- coarse reconciliation policy enums as the primary public semantic surface
- any open construction path for lowered merge packets

### 5.3 Compile-time protection needed

S10 must add compile-time coverage proving:

- lowered merge strategy bundles are not publicly WORTHable
- merge-capable runtimes cannot build without schema + frozen strategy registry
- external callers cannot bypass lowered merge planning by constructing late
  phase packets manually

## 6. M10.0 Deliverables Checklist

- [ ] Inventory each current merge file and mark preserve / extend / replace / delete
- [ ] Inventory each current merge-related public export and mark preserve / replace / hide
- [ ] Mark every hardcoded merge-semantic decision in `merge_runtime.rs`
- [ ] Mark every current coarse policy type that will be replaced by family strategies
- [ ] Confirm S10 will replace pre-S10 merge/replay artifacts instead of compatibility-layering them
- [ ] Confirm new ownership zones: `schema/` and `merge/strategies/`
- [ ] Confirm bounded merge proof backbone remains authoritative

## 7. Ready-To-Start Criteria For M10.1

M10.1 may begin once all boxes above are checked and we agree on:

- which current artifacts are being preserved as the bounded substrate
- which current semantic surfaces are being replaced
- which new modules own schema authority and strategy authority
