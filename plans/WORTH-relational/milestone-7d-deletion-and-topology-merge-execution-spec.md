# Milestone 7D Deletion And Topology Merge Execution Spec

## Purpose

This document is the build specification for Milestone 7D.

Milestone 7C established authoritative merge execution for the admitted
non-deletion merge subset. Milestone 7D extends the merge ontology without
weakening the proof chain, serialized authority model, replay parity, or
diagnostics rigor established in 7C.

The governing rule remains:

`parallelize disposable work, serialize authority`

Milestone 7D does not treat deletion or topology semantics as â€œspecial merge
exceptions.â€ It treats them as truth-bearing merge classes that must either:

- compile into an authoritative executable class, or
- remain explicitly non-executable with typed denial and proof-bearing
  diagnostics

There is no silent middle ground.

## Problem Statement

After 7C, the runtime can already:

- classify deletion conflicts explicitly
- preserve branch-local visibility evidence
- produce typed blocked/rejected lowered plans
- execute admitted merge classes through the authoritative commit pipeline

What it still cannot do is express the deletion/topology universe honestly
enough to evolve execution incrementally.

The current failure mode is not correctness of 7C execution. The failure mode
is ontology collapse:

- all deletion cases look like one generic â€œblocked deletionâ€
- relation rewiring remains visible as conflict but not as execution-class
  ontology
- operators cannot distinguish â€œsafe but not yet admittedâ€ from â€œsemantically
  impossible under current truth rulesâ€
- future executable promotion risks leaking semantics into lowering or commit
  apply instead of passing through a clear proof boundary

Milestone 7D exists to fix that before we admit any new executable merge
classes.

## Scope

Milestone 7D covers:

- exhaustive domain-agnostic merge policy coverage for generic relational truth
- explicit deletion execution ontology
- explicit topology execution ontology for relation rewiring surfaces
- typed mapping from conflict class to execution class or denial class
- promotion of the safest deletion executable subset
- preservation of fail-closed behavior for all not-yet-admitted classes
- certification of branch-local deletion/topology merge behavior
- preservation of identity, lineage, and provenance through every merge class

Milestone 7D does not yet attempt:

- arbitrary manual-resolution execution
- broad topology-region reconciliation
- generic rewrite-based merge intent repair
- heuristic relation endpoint rehoming

## Governing Constraints

1. Execution must continue to consume proof-carrying lowered or compiled merge
   artifacts only.
2. Commit application must not rediscover deletion semantics, topology
   semantics, or relation continuity.
3. Cost must remain proportional to admitted merge surface, not full branch
   breadth or full topology neighborhood breadth.
4. Diagnostics must distinguish:
   - exact shared truth
   - source-only addition
   - schema-declared correspondence reconciliation
   - executable deletion classes
   - explicitly non-executable deletion classes
   - topology-local blocked classes
   - topology-region escalation classes
   - divergent visible-state/manual-resolution classes
5. Identity, lineage, and provenance must remain explicit across every merge
   class, whether executed or denied.
6. Any merge class not admitted in 7D must remain fail-closed and machine-
   localizable.

## Current Baseline To Preserve

7D builds on these already-certified 7C guarantees:

- sealed merge preparation
- freshness verification and authority binding
- bound executable plan compilation
- merge-to-mutation derivation for admitted non-deletion classes
- shared authoritative commit pipeline
- durable merge diagnostics and replay parity
- explicit visibility evidence and branch-local historical visibility repair
- explicit lowered deletion blocked-reason taxonomy

7D must not regress any of them.

## Architectural Decision

Deletion and topology semantics will be introduced in two layers:

1. `MergeResolutionClass`
   This is the full ontology: what kind of merge situation exists in truth.
2. `MergeExecutableClass`
   This is the admitted executable subset for the current milestone.

The architecture must never equate â€œrepresentedâ€ with â€œexecutable.â€

The execution boundary may consume only `MergeExecutableClass`.
All other classes remain typed denial surfaces.

7D also fixes the policy ownership boundary explicitly:

- the runtime must own every domain-agnostic merge policy needed to reconcile
  generic relational truth
- custom merge policy hooks exist for domain-authored aspect semantics and
  domain-specific structural meaning, but they must not be used to patch holes
  in the generic merge ontology

The architecture must never offload core merge truth to custom policy merely
because the semantics are inconvenient.

## Domain-Agnostic Merge Policy Coverage

7D is complete only if the runtime covers the full generic merge universe
honestly. That does not mean every class becomes executable in 7D. It means
every class is owned by the runtime as either:

- an admitted executable class, or
- an explicitly certified non-executable class with typed denial and proof-
  bearing diagnostics

There must be no generic merge class whose only answer is â€œleave it to custom
policy.â€

### Record-Level Structural Merge Classes The Runtime Must Own

The runtime itself must classify, lower, diagnose, and preserve truth for all
of these generic record-level merge situations:

- `ExactSharedTruth`
- `SourceOnlyAddition`
- `SchemaDeclaredCorrespondence`
- `Deletion(SourceDeletedTargetLive)`
- `Deletion(SourceLiveTargetDeleted)`
- `Deletion(DeletedOnBothSides)`
- `Deletion(DeletedVsModified)`
- `Deletion(DeletedVsRewired)`
- `Topology(RelationEndpointStable)`
- `Topology(RelationEndpointRewiredLocal)`
- `Topology(RelationEndpointRewiredEscalated)`
- `Topology(TopologyRegionConflict)`
- `DivergentVisibleState`

This list is exhaustive for the domain-agnostic structural merge universe
recognized by the runtime in 7D.

### Built-In Aspect Merge Policies The Runtime Must Own

The runtime must also continue to own the generic aspect-policy universe. At a
minimum, these built-in policy kinds are part of the core merge surface and may
not be delegated to custom policy:

- `FailOnConflict`
- `LastWriterWins`
- `MonotonicCounter`
- `AdditiveSet`
- `PreferRicher`

These are generic data-shape policies, not domain-specific semantics. They are
part of the runtimeâ€™s standard merge contract and must preserve:

- explicit applied-policy provenance
- explicit per-aspect resolution records
- explicit authorized value surfaces
- replay-stable policy lowering

### Merge Policy Resolution Outcomes The Runtime Must Own

For every generic record-level and aspect-level merge class above, the runtime
must own the policy-resolution boundary explicitly:

- `AutoResolved`
- `RequiresManualResolution`
- `Reject`

No generic merge class may bypass this resolution surface through helper logic,
silent policy defaults, or caller-authored repair.

This boundary must become more proof-bearing as 7D closes out. A generic class
that has merely been observed is not equivalent to one that has been policy-
resolved. Later phases must not infer that distinction procedurally.

### Identity, Lineage, And Provenance Preservation Requirements

Every generic merge class, whether executed or denied, must preserve:

- identity basis provenance
- target/source/base visibility provenance
- causal disposition provenance
- applied policy provenance
- record-level classification provenance
- lineage continuity semantics, including explicit â€œunchangedâ€ when lineage does
  not move

The runtime must not treat identity, lineage, or provenance preservation as
optional only because a class remains non-executable in 7D.

## Proof-Carrying Type Boundary Requirements

The remaining 7D work must move more assumptions out of comments and helper
behavior and into proof-bearing types or named policy surfaces.

At a minimum, the implementation must distinguish these states structurally:

- generic runtime-owned merge class vs domain-specific custom-policy surface
- classified generic merge class vs policy-resolved generic merge class
- policy-resolved generic merge class vs lowered execution-or-denial class
- denial class with complete provenance vs denial class with incomplete
  provenance
- lineage-preserved-unchanged vs lineage-transformation-authorized
- topology-local denial vs topology-region denial with bounded neighborhood
  proof

The goal is not wrapper proliferation. The goal is to eliminate convention-
based ambiguity where downstream phases can reinterpret what an upstream phase
already proved.

### Newtype And Wrapper Targets

The following semantic surfaces are strong candidates for stricter wrappers or
sealed proof types before 7D is considered complete:

- generic merge class ownership
- manual-resolution eligibility vs hard reject
- lineage continuity verdict
- provenance completeness for denied classes
- topology escalation proof
- policy provenance bundle
- visibility provenance bundle

Introduce a wrapper only when it either:

- makes an important illegal state unrepresentable, or
- prevents a later phase from having to re-infer a proof already established
  upstream

## Custom Merge Policy Boundary

Custom merge policy hooks are allowed only for semantics that are not generic
to relational truth.

Custom policy is appropriate for:

- domain-authored aspect reconciliation beyond the built-in policy set
- domain-specific replacement or supersession semantics
- domain-specific topology repair or endpoint rehoming semantics
- domain-specific equivalence rules
- domain-specific lineage promotion rules

Custom policy is not appropriate for:

- one-sided deletion classification
- deletion-vs-modification classification
- deletion-vs-rewiring classification
- relation-local rewiring classification
- topology-region conflict detection
- generic manual-resolution vs reject boundaries
- freshness, replay, lineage, or provenance preservation

If a merge behavior is expected to arise in any serious relational domain
without requiring domain-specific business meaning, it belongs in the runtime,
not in a custom merge policy.

## Execution Ontology

### Deletion Resolution Classes

The runtime must explicitly preserve at least these deletion classes:

- `SourceDeletedTargetLive`
- `SourceLiveTargetDeleted`
- `DeletedOnBothSides`
- `DeletedVsModified`
- `DeletedVsRewired`

These classes already exist at conflict time. 7D elevates them into the
execution/denial ontology so that downstream phases do not collapse them back
into generic blocked deletion.

### Topology Resolution Classes

The runtime must explicitly preserve at least these topology classes:

- `RelationEndpointStable`
- `RelationEndpointRewiredLocal`
- `RelationEndpointRewiredEscalated`
- `TopologyRegionConflict`

7D may refine the exact names, but the ontology must preserve the distinction
between relation-local rewiring and topology-region escalation.

## 7D Admitted Executable Subset

The default 7D promotion target is:

- `DeletedOnBothSides`

Why this class first:

- it has the smallest semantic surface
- it does not require one branch to â€œwinâ€ a visible record over the other
- it does not imply topology rewiring
- it is the easiest class to replay/certify without hidden identity mutation

All other deletion classes remain non-executable in the initial 7D slice unless
their authoritative semantics are separately specified and certified.

## Type Surfaces

### New Or Extended Planning Types

Primary module targets:

```text
crates/worth-relational/src/merge/data/conflicts.rs
crates/worth-relational/src/merge/data/policy.rs
crates/worth-relational/src/merge/data/execution.rs
crates/worth-relational/src/merge/data/artifacts.rs
```

Types to add or extend:

- `DeletionExecutionClass`
- `TopologyExecutionClass`
- `MergeResolutionClass`
- `MergeExecutableClass`
- explicit denial taxonomy for non-executable deletion classes
- explicit denial taxonomy for topology-local vs topology-region classes

Required invariants:

- every deletion conflict must map to exactly one deletion execution or denial
  class
- every topology conflict must map to exactly one topology execution or denial
  class
- it must be impossible for execution compilation to receive a generic
  â€œblocked deletionâ€ bucket

Additional required invariants:

- generic runtime-owned merge classes must not silently fall through to custom
  policy
- denied generic classes must retain provenance required for replay/recovery
  certification

### Process Types And Phase Outputs

The remaining 7D work should be modeled as a repeatable proof-widening process
for each generic merge class:

1. classified
2. policy-resolved
3. readiness-lowered
4. execution-or-denial shaped
5. provenance-complete
6. replay/recovery certified

If the implementation carries these steps as fields on broad records, that is
acceptable only if the fields are interpreted unambiguously and construction
remains sealed. If downstream code still needs to infer which proof stage a
record has reached, the type boundary is too weak.

### Lowering Types

Primary module target:

```text
crates/worth-relational/src/merge/logic/lowering.rs
```

Lowering must:

- preserve the specific deletion/topology class
- derive typed denial bundles for non-executable classes
- admit only the current executable subset
- keep the current `Blocked` / `Rejected` / `Admitted` contracts intact
- preserve manual-resolution vs reject meaning as a named policy boundary, not
  an incidental blocked/rejected side effect
- preserve provenance completeness needed by later replay/recovery

### Execution Types

Primary module targets:

```text
crates/worth-relational/src/merge/data/execution.rs
crates/worth-relational/src/merge/logic/execution.rs
crates/worth-relational/src/merge/logic/execution_mutation_plan.rs
```

If `DeletedOnBothSides` is promoted, execution must add a concrete executable
record-plan variant for authoritative no-op convergence or authoritative tombstone
convergence, whichever semantics the runtime formally adopts.

The execution type must make the choice explicit. It must not rely on â€œno
mutation happened so it must have been both deleted.â€

### Assumptions That Must Become Explicit

The following assumptions may no longer remain implicit in helper behavior,
tests, or broad record structs:

- whether a generic class is runtime-owned or custom-policy-owned
- whether a non-executable class is manual-resolution eligible or hard reject
- whether lineage is unchanged, preserved, or explicitly transformed
- whether topology escalation is policy-forced or structurally forced
- whether a built-in aspect policy is sufficient to auto-resolve a class
- whether fail-closed status is milestone-local or reflects the absence of any
  generic truth rule

Each of these must be encoded as:

- a named policy surface, or
- a sealed proof-bearing type, or
- both

## Semantic Requirements For The First Executable Deletion Class

If 7D promotes `DeletedOnBothSides`, the runtime must specify:

- whether authoritative execution publishes a merge commit with zero record
  mutations but positive merge truth
- whether diagnostics describe it as preserved mutual deletion truth or
  converged deletion truth
- whether lineage surfaces change at all
- whether merge execution artifact rows for this class are execution rows, not
  denial rows

This class must not be admitted until these semantics are explicit.

## Non-Executable Classes That Must Remain Fail-Closed

Unless explicitly promoted later in 7D, these remain non-executable:

- `SourceDeletedTargetLive`
- `SourceLiveTargetDeleted`
- `DeletedVsModified`
- `DeletedVsRewired`
- topology-region escalation classes

The runtime must localize each with:

- class-specific denial kind
- branch-local visibility evidence
- base evidence
- if relation-local, continuity/propagation evidence

This fail-closed status does not make these classes â€œcustom-policy territory.â€
It means the runtime owns them as generic truth classes but has not yet admitted
them for authoritative execution.

## Implementation Phases

Implementation must proceed in this order.

### Phase A: Ontology Lift

Goal:
Lift deletion and topology classes into explicit execution/denial ontology
without changing executable behavior.

Code ownership:

```text
crates/worth-relational/src/merge/data/conflicts.rs
crates/worth-relational/src/merge/data/policy.rs
crates/worth-relational/src/merge/logic/lowering.rs
crates/worth-relational/src/merge/data/artifacts.rs
```

Required outcomes:

- generic blocked deletion is eliminated from authoritative planning surfaces
- every lowered blocked deletion carries a specific class
- artifact digest basis includes the class-bearing denial surface
- generic runtime-owned classes are distinguishable from custom-policy surfaces
  before lowering

Exit criteria:

- no authoritative artifact uses a generic deletion blocked reason

### Phase B: Executable-Class Boundary

Goal:
Define which deletion/topology classes are executable vs non-executable.

Required outcomes:

- explicit mapping from `MergeResolutionClass` to `MergeExecutableClass` or
  denial
- compilation rejects any non-admitted class structurally
- execution types cannot be constructed from non-admitted classes
- runtime-vs-custom policy ownership boundary is explicit and exhaustive for
  the generic merge universe
- policy-resolved generic classes carry enough proof that later phases do not
  re-decide manual-resolution vs reject eligibility

Exit criteria:

- illegal state â€œnon-admitted deletion/topology class reaches executable planâ€
  is unrepresentable

### Phase C: First Deletion Promotion

Goal:
Promote the first deletion executable class, expected to be
`DeletedOnBothSides`.

Required outcomes:

- compiled executable record-plan variant exists
- mutation derivation handles the class without rediscovering semantics
- commit pipeline remains shared
- success/failure diagnostics distinguish executable deletion truth from denied
  deletion truth
- lineage semantics for the admitted class are explicit and proof-bearing, not
  inferred from â€œno mutation emittedâ€

Exit criteria:

- one deletion class is executable end to end with replay parity

### Phase D: Certification And Hostile-Path Hardening

Goal:
Certify the expanded ontology and first executable deletion class.

Required tests:

- branch-local deletion classification parity
- replay parity for deletion-bearing merge commits
- recovery parity for deletion-bearing merge commits
- denial artifact parity for non-executable deletion classes
- topology-local denial parity
- topology-region denial parity
- generic manual-resolution vs reject parity
- per-class identity, lineage, and provenance parity across replay/recovery
- policy-provenance parity for built-in generic aspect policies
- proof-stage parity: no phase may require rediscovery of a proof already
  claimed by an upstream phase
- hostile stale-head and schema-drift rejection on deletion-bearing prepared
  merges

Exit criteria:

- machine-checkable artifacts distinguish executable deletion truth,
  non-executable deletion denial, topology denial, and generic manual-
  resolution truth without ambiguity

## Performance Contracts

7D must obey these explicit cost rules:

- deletion/topology classification may scale with candidate merge record breadth
  and admitted relation-local evidence breadth
- execution compilation may scale only with admitted executable record breadth
- commit apply must not scan full visible views or topology regions
- topology-region escalation detection must be counted explicitly when it occurs
- no phase may hide broad view indexing or neighborhood rediscovery in helper
  code
- proof-stage transitions must not duplicate large structural packets merely to
  rename their phase; every wrapper must either remove an illegal state or
  materially narrow what a later phase may do

## Diagnostics Requirements

The artifact family must teach the architecture:

- planning artifact: full deletion/topology classification and denial evidence
- execution artifact: executed rows only
- failure artifact: typed denial or freshness/authority failure only

Diagnostics for denied generic classes must also teach:

- whether the class is runtime-owned generic truth or custom-policy territory
- whether the outcome is manual-resolution eligible or rejected
- whether lineage is unchanged or explicitly frozen
- whether topology escalation was policy-forced or structurally forced

Success-path execution artifacts must never contain blocked rows.

## Open Decisions To Make Explicit Before Promotion

Before promoting any deletion class, the implementation must answer:

1. Is mutual deletion execution a no-op truth convergence or a distinct
   authoritative deletion-preservation act?
2. Does deletion execution affect lineage surfaces at all?
3. Can any one-sided deletion class ever be safely executable without explicit
   user intent?
4. Is relation endpoint rewiring ever relation-local, or does it always escalate
   to topology-region denial in this milestone?

These decisions must be encoded in types or named policy surfaces, not left as
code comments or implicit behavior.

## Generic Merge Coverage Checklist

7D is not complete unless the document and implementation can account for every
generic merge policy surface below:

- `ExactSharedTruth`
- `SourceOnlyAddition`
- `SchemaDeclaredCorrespondence`
- `Deletion(SourceDeletedTargetLive)`
- `Deletion(SourceLiveTargetDeleted)`
- `Deletion(DeletedOnBothSides)`
- `Deletion(DeletedVsModified)`
- `Deletion(DeletedVsRewired)`
- `Topology(RelationEndpointStable)`
- `Topology(RelationEndpointRewiredLocal)`
- `Topology(RelationEndpointRewiredEscalated)`
- `Topology(TopologyRegionConflict)`
- `DivergentVisibleState`
- `FailOnConflict`
- `LastWriterWins`
- `MonotonicCounter`
- `AdditiveSet`
- `PreferRicher`
- `AutoResolved`
- `RequiresManualResolution`
- `Reject`

For each item above, the runtime must define all of:

- classification surface
- policy-resolution surface
- execution or denial surface
- identity/lineage/provenance surface
- replay/recovery certification surface
- proof-stage ownership surface
- explicit runtime-vs-custom policy ownership surface

If any item lacks one of those surfaces, 7D is not closed.

## Recommended Immediate Build Order

The next coding work should follow this exact sequence:

1. eliminate generic deletion blocked-denial surfaces everywhere
2. add `MergeResolutionClass` / `MergeExecutableClass` split
3. keep all deletion classes fail-closed under the new ontology
4. specify `DeletedOnBothSides` execution semantics explicitly
5. promote `DeletedOnBothSides` only if the semantics stay replay-stable and
   mutation-derivation clean
6. certify before touching one-sided deletion or topology-region execution

## Definition Of Done

7D is complete only when:

- every domain-agnostic merge policy surface is explicitly owned by the runtime
- deletion/topology ontology is explicit and proof-bearing
- at least one deletion class is either explicitly certified executable or
  explicitly certified non-executable with typed denial
- one-sided deletion and delete-vs-modify/delete-vs-rewire remain explicitly
  owned generic classes even when fail-closed
- topology-local rewiring and topology-region conflict remain distinct in
  ontology, denial, diagnostics, and certification
- the remaining generic classes carry proof-bearing policy-resolution and
  provenance surfaces rather than relying on helper-path interpretation
- the remaining implicit assumptions named in this spec have been converted into
  named policy surfaces or sealed proof-bearing types
- replay/recovery parity holds for the newly admitted executable set
- operators can distinguish every blocked deletion/topology class without
  reverse-engineering generic denial buckets
- identity, lineage, and provenance remain explicit across executed and denied
  merge classes
- custom policy is used only for domain-specific semantics, never for missing
  generic merge truth
- the commit pipeline remains shared and merge semantics remain absent from
  generic commit apply
