# Milestone 7D Closeout: Deletion and Topology Merge Authority Completion

## Status

Milestone 7D is closed as of 2026-03-29.

The runtime now treats deletion and topology merge semantics as first-class
authority surfaces instead of as generic blocked edge cases or future-policy
placeholders.

The semantic center shipped in this milestone is:

merge now owns an explicit domain-agnostic ontology for deletion, topology,
built-in merge policy resolution, and proof-bearing denial or execution
surfaces, while preserving identity, lineage, provenance, durability parity,
replay parity, and bounded cost accounting through the same authoritative commit
pipeline established in 7C.

This is not "more merge cases were added."

The runtime now owns:

- explicit deletion-class ontology
- explicit topology-class ontology
- an explicit split between full merge truth and admitted executable subset
- proof-bearing policy ownership and policy-boundary surfaces
- the first executable deletion class:
  - `DeletedOnBothSides`
- typed non-executable denial surfaces for all remaining generic deletion and
  topology classes
- bounded topology-region conflict detection with neighborhood evidence
- built-in generic merge policy support for:
  - `FailOnConflict`
  - `LastWriterWins`
  - `MonotonicCounter`
  - `AdditiveSet`
  - `PreferRicher`
- persistent-name and ancestor-basis policy resolution for built-in merge
  policies
- durable merge execution diagnostics and denial artifacts with replay and
  recovery parity
- execution and planning complexity contracts for the merge pipeline

## Shipped Scope

Milestone 7D delivered:

- explicit generic merge ontology through:
  - `MergeResolutionClass`
  - `MergeExecutableClass`
  - `DeletionExecutionClass`
  - `TopologyExecutionClass`
- explicit deletion denial taxonomy for:
  - `SourceDeletedTargetLive`
  - `SourceLiveTargetDeleted`
  - `DeletedOnBothSides`
  - `DeletedVsModified`
  - `DeletedVsRewired`
- explicit lowering preservation of deletion semantics through:
  - `LoweredMergeBlockedReason`
  - `LoweredRecordDenialKind`
  - `LoweredAspectDenialIntent`
- explicit topology ontology for:
  - `RelationEndpointStable`
  - `RelationEndpointRewiredLocal`
  - `RelationEndpointRewiredEscalated`
  - `TopologyRegionConflict`
- bounded topology escalation with:
  - endpoint-incidence scoping
  - connected-neighborhood BFS detection
  - neighborhood record evidence
  - rewired-record subset evidence
  - `TopologyRegionConflictReason`
- explicit policy ownership and proof boundaries through:
  - `MergePolicyOwnershipClass`
  - `MergePolicyOwnershipSurface`
  - `MergePolicyProofBoundary`
  - specific `MergeManualResolutionClass`
  - specific `MergePolicyRejectClass`
- executable promotion of `DeletedOnBothSides` through:
  - lowering
  - execution compilation
  - merge-to-mutation derivation
  - authoritative commit publication
  - replay/recovery certification
- explicit lineage and deletion semantics on executed deletion convergence
- built-in generic merge policy support for payload-field aspects with:
  - schema admission
  - proof-bearing resolved-value strategy surfaces
  - inline canonical value materialization when required
  - causal-gated `LastWriterWins`
  - three-way `MonotonicCounter`
  - observed-remove `AdditiveSet`
- explicit ancestor/persistent-name resolution for built-in policy execution
  without heuristic field-name fallback
- unified interned symbol resolution across:
  - policy resolution
  - identity discovery
  - conflict classification
  - execution compile
  - execution mutation derivation
- direct execution complexity proof coverage in addition to planning coverage

## Phase Completion Map

Milestone 7D is considered closed against the implementation work carried out
in this phase set.

### Phase A: Generic Policy Ownership and Coverage Foundation

Closed by:

- `crates/worth-relational/src/merge/data/policy.rs`
- `crates/worth-relational/src/merge/logic/policy.rs`
- `crates/worth-relational/src/merge/data/artifacts.rs`
- `crates/worth-relational/src/merge/logic/planning_artifact.rs`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_a.rs`

What is proven:

- runtime-owned policy participation is explicit before lowering
- custom-policy participation is mechanically distinguishable from runtime-only
  merge policy ownership
- planning artifacts and digest bases preserve policy ownership truth
- generic merge policy coverage is treated as a runtime responsibility rather
  than as an implied custom-policy extension seam

### Phase B: Policy Boundary Hardening

Closed by:

- `crates/worth-relational/src/merge/data/policy.rs`
- `crates/worth-relational/src/merge/logic/policy.rs`
- `crates/worth-relational/src/merge/logic/lowering.rs`
- `crates/worth-relational/src/merge/logic/execution.rs`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_b.rs`

What is proven:

- manual-resolution and reject classes are named proof boundaries, not helper
  folklore
- record-level proof aggregation preserves specific upstream policy classes
- lowering preserves those classes into denial and reject surfaces instead of
  flattening them into generic buckets
- execution diagnostics preserve policy proof boundaries structurally

### Phase C: Executable Deletion Class Promotion

Closed by:

- `crates/worth-relational/src/merge/data/execution.rs`
- `crates/worth-relational/src/merge/logic/execution.rs`
- `crates/worth-relational/src/merge/logic/execution_mutation_plan.rs`
- `crates/worth-relational/src/merge/data/execution_artifacts.rs`
- `crates/worth-relational/src/merge/logic/execution_diagnostics.rs`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_c.rs`

What is proven:

- `DeletedOnBothSides` is the first promoted executable deletion class
- execution consumes an explicit executable class, not a generic deletion
  shortcut
- deletion convergence carries explicit equality witness, deletion semantics,
  and lineage continuity
- merge publication, replay, and recovery preserve executed deletion truth
  without requiring emitted mutation intents

### Phase D: Generic Deletion and Topology Closeout Certification

Closed by:

- `crates/worth-relational/src/merge/logic/conflicts.rs`
- `crates/worth-relational/src/merge/logic/policy.rs`
- `crates/worth-relational/src/merge/logic/identity.rs`
- `crates/worth-relational/src/merge/logic/execution.rs`
- `crates/worth-relational/src/merge/logic/execution_mutation_plan.rs`
- `crates/worth-relational/src/merge/logic/naming.rs`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_d.rs`
- `crates/worth-relational/src/tests/complexity/contracts/merge_budgets.rs`

What is proven:

- topology-region conflict is a real bounded runtime detector, not a future
  bucket
- rewiring escalation remains fail-closed by explicit policy rather than by
  missing implementation
- disjoint rewires do not falsely escalate into topology-region conflict
- unrelated relation additions do not inflate topology-region counters
- built-in generic merge policies are resolved through explicit ancestor and
  persistent-name proof surfaces
- `AutoResolved` built-in policy rows cannot silently degrade into missing
  authoritative values
- symbolized aspect and field names are resolved consistently across the merge
  pipeline
- merge execution has an explicit complexity contract in addition to planning
  and verification contracts

## What 7D Now Guarantees

The runtime now guarantees all of the following simultaneously:

- every domain-agnostic deletion class is explicit in the merge ontology
- every domain-agnostic topology class in current scope is explicit in the merge
  ontology
- the full merge-truth surface is separated from the admitted executable subset
- denial classes are typed, replayable, and durable
- `DeletedOnBothSides` executes through the same serialized authority path as
  ordinary commits
- one-sided deletion and topology rewiring remain fail-closed instead of
  degenerating into implicit policy
- topology-region conflict detection is bounded to rewired-neighborhood scope
  and emits proof-bearing evidence
- built-in generic merge policy resolution preserves:
  - identity basis
  - ancestor basis
  - causal gating
  - resolved-value provenance
  - recovery parity
- merge diagnostics remain phase-honest:
  - success artifacts for executed merge truth
  - failure artifacts for denied, stale, or drifted merge attempts
- merge performance accounting now covers:
  - planning
  - topology-region detection
  - execution verification
  - execution commit

## Domain-Agnostic Merge Ownership Boundary

The runtime now fully owns the generic relational merge universe for:

- `ExactSharedTruth`
- `SourceOnlyAddition`
- `SchemaDeclaredCorrespondence`
- `DivergentVisibleState`
- all current deletion classes
- all current topology classes
- built-in aspect merge policies:
  - `FailOnConflict`
  - `LastWriterWins`
  - `MonotonicCounter`
  - `AdditiveSet`
  - `PreferRicher`
- generic resolution outcomes:
  - `AutoResolved`
  - `RequiresManualResolution`
  - `Reject`

The runtime does not delegate missing generic semantics to custom policy.

Custom policy remains the extension seam only for domain-authored meaning beyond
the generic relational truth model.

## Identity, Lineage, and Provenance Guarantees

Milestone 7D closes only because the runtime preserves identity, lineage, and
provenance across both executed and denied merge classes.

That now includes:

- storage identity where exact parity exists
- lineage identity where branch-local divergence preserves logical continuity
- declared-key and schema-declared correspondence identity when admitted
- visibility provenance for source, target, and base evidence
- policy provenance for built-in policy resolution
- denial provenance for blocked and rejected generic classes
- lineage continuity semantics on executed deletion convergence
- equality witness digests on exact shared and mutual-deletion convergence

The runtime may still deny a merge class.

It no longer loses the explanation for why that class existed or how it was
classified.

## Performance Closeout

Milestone 7D closes with the following merge performance boundaries explicitly
measured and certified:

- `runtime.merge.planning`
- `runtime.merge.topology_region_detection`
- `runtime.merge.execution_verification`
- `runtime.merge.execution_commit`

The important closeout change is this:

- built-in generic policy resolution no longer relies on repeated broad scans or
  heuristic fallback

Instead it uses:

- cached read-view indexes for source, target, and ancestor views
- cached ancestor payload reconstruction per ancestor commit
- unified interned-string resolution across the merge stack

This keeps the policy phase request-shaped and prevents hidden O(view breadth)
fallback loops from leaking into per-aspect merge resolution.

## Certification Summary

Milestone 7D now has explicit certification coverage for:

- `DeletedOnBothSides` end-to-end execution
- `DeletedOnBothSides` replay parity
- `DeletedOnBothSides` recovery parity
- stale-head rejection for prepared deletion merges
- schema-drift rejection for prepared deletion merges
- one-sided and mixed deletion denial parity across recovery
- rewiring escalation denial parity across recovery
- topology-region conflict detection and counter boundedness
- topology-region recovery parity
- disjoint-rewire non-escalation
- unrelated-addition non-inflation of topology counters
- built-in policy reject fallback parity
- built-in policy auto-resolution parity
- built-in policy recovery parity
- merge execution complexity-budget proof coverage

Library closeout verification at milestone completion:

- `cargo check -p worth-relational`
- `cargo test -p worth-relational milestone_7d_phase_c -- --nocapture`
- `cargo test -p worth-relational milestone_7d_phase_d -- --nocapture`
- `cargo test -p worth-relational --lib -- --nocapture`

Final result at closeout:

- `547 passed; 0 failed`

## What Remains Intentionally Deferred

Milestone 7D does not close by pretending every merge class is executable.

The following remain intentionally deferred beyond 7D:

- one-sided deletion execution semantics
- generic delete-vs-modify execution semantics
- generic delete-vs-rewire execution semantics
- executable relation rewiring
- executable topology-region reconciliation
- broad manual-resolution execution
- domain-authored semantic repair for topology or deletion meaning

Those are not missing generic ontology anymore.

They are intentionally deferred execution/policy expansions beyond the
domain-agnostic runtime-owned base that 7D was responsible for completing.

## Handoff Constraint For The Next Milestone

Any future milestone that widens merge execution must preserve the following
7D invariants:

- do not collapse represented merge truth back into generic blocked buckets
- do not bypass the `MergeResolutionClass` / `MergeExecutableClass` split
- do not reintroduce helper-level symbol resolution asymmetry
- do not rediscover ancestor basis inside execution
- do not widen executable topology semantics without bounded neighborhood proof
- do not delegate missing domain-agnostic semantics to custom policy
- do not allow execution-time semantic reinterpretation of planning proof

The next milestone may widen execution.

It must consume the 7D ontology and proof boundaries as authoritative inputs,
not as advisory structure to reinterpret.
