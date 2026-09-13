# C.7 Phase 7 Root Publication Implementation Plan

Phase 7 replaces every current-root execution path with one Store-owned
progression. The progression consumes the exact durable mutation group, root
candidate, checkpoint/reopen basis, admitted durability policy, and C.4 effect
receipts. It advances the current-root projection only after parent-namespace
durability and retains the displaced root without granting deletion authority.

## Adversarial constraint

No value may represent a namespace-durable current root unless it consumes and
matches all of the following:

- the old durable root and exact successor generation;
- the Store identity and runtime generation;
- one sealed durability-group identity and membership digest;
- every member's WAL-durable basis and completed data settlement;
- the applicable namespace-durable checkpoint or typed bootstrap-reopen basis;
- the admitted durability-policy identity and backend profile;
- every required candidate-artifact synchronization receipt;
- the exact candidate-to-bootstrap-catalog replacement receipt;
- the exact parent-directory synchronization receipt; and
- the `RootPublication` work basis, Signal route, scheduler capability, and
  Foundational policy-admission receipt for each effect.

A candidate sync is not replacement. Replacement is not namespace durability.
After a possible replacement without namespace durability, the runtime becomes
inspection-required and cannot acknowledge or advance its current-root
projection. A foreign, copied, reordered, incomplete, stale, or digest-only
basis grants no authority.

## Destination skeleton

```text
worth-store/src/physical_runtime/durability/
|-- publication/
|   |-- mod.rs
|   |-- identity.rs
|   |-- root_projection.rs
|   |-- preparation.rs
|   |-- artifact_manifest.rs
|   |-- candidate_durability.rs
|   |-- replacement.rs
|   |-- namespace_durability.rs
|   |-- retained_root.rs
|   |-- current_root_owner.rs
|   |-- work_port.rs
|   `-- failure.rs
`-- mutation/progression/
    |-- root_prepared.rs
    |-- root_replaced.rs
    `-- root_namespace_durable.rs

worth-store/src/physical_runtime/work/
|-- declaration/root_publication_scope.rs
|-- execution/command/root_publication.rs
|-- execution/outcome/root_publication.rs
`-- execution/settlement/classification/root_publication.rs

worth-store/src/physical_runtime/instance/
|-- scheduler_admission/root_publication.rs
`-- signal_owner/graph/root_publication.rs

worth-store/src/physical_runtime/record_serving/work_semantics/durability/
`-- root_publication_basis.rs
```

The directories are semantic growth boundaries. A directory may initially
contain one file when the responsibility is expected to gain siblings.

## Slice 7A: preserve the root projection

Split prepared payload material into consumed data material and retained root
projection material. Preserve payload manifests, placements, segment updates,
inline-allocation state, observation, and the source-root basis after streaming
sources are consumed. Carry the retained projection linearly through WAL and
data progression.

Join settled members in sealed group order. One-member groups retain individual
identity; multi-member groups merge into one successor-root projection bound to
the exact group and membership digest. Missing, duplicate, reordered, foreign,
or incompletely settled membership is denied before root effects.

Proof: focused split/carriage tests, group-order and identity mutants, and
compile failures for root preparation without `DataSettledPhysicalMutation`.

## Slice 7B: exact root work authority

Install `store.physical.durability.root-publication-basis` as a mutation
`DependencyAndOutput` contract serving only `RootPublication`, partitioned by
candidate publication identity and Store/runtime generation.

Add a dedicated root-publication work operation, scope, durability action,
Signal route, command, outcome, settlement, and recovery posture. Candidate
file sync, durable rename, and directory sync receive distinct scheduler
admission types backed by filesystem-admitted C.4 capability claims. Each
scheduled action carries the exact Foundational policy-admission receipt.

Proof: family, partition, capability, policy, action, artifact, Store, runtime,
and generation substitution tests plus direct-construction compile failures.

## Slice 7C: compiler-visible root progression

Implement the linear group progression:

```text
DataSettledPhysicalMutationMembers
  -> RootPublicationPreparedPhysicalMutationMembers
  -> RootReplacedPhysicalMutationMembers
  -> RootNamespaceDurablePhysicalMutationMembers
```

Root preparation materializes the exact candidate artifact manifest. Candidate
durability consumes all required file-sync completions. Replacement consumes
only the candidate-durable value. Namespace synchronization consumes only the
replaced value. No public constructor or generic phase object exists.

Proof: exact artifact-manifest inspection, fail-before and
indeterminate-after-effect seams, missing-directory-sync mutant, and compile
failures for skipped or reordered stages.

## Slice 7D: one current-root owner and retained old-root truth

The bootstrap catalog remains the durable truth. One Store runtime owner holds
the serving projection and its cutover lock. It alone may consume
`RootNamespaceDurablePhysicalMutationMembers`, advance the projection exactly
once, and emit the retained old-root manifest plus its required artifact
inventory. Retention grants no deletion authority.

Move current-root and free-space mutation authority out of general publication
director state. Read-side snapshots remain observations and cannot advance the
root.

Proof: double-advance, stale-root, foreign-generation, premature-retention, and
early-deletion attacks; fresh-process offline inspection must identify exactly
the namespace-durable catalog root while the old root remains present.

## Slice 7E: cut competing publication paths

Delete `PhysicalRootPublicationStore`, `PhysicalRootPublicationRuntime`, the
`root-publications.log` writer/parser, recovery execution through that runtime,
and its test fixture. Narrow physical-isolation code to pure stable-reader or
root-validation meaning only where a real C.10 consumer remains.

Delete or narrow layout migration, rollback, and maintenance execution APIs
that directly publish through the parallel runtime. Remove generic catalog
replacement work and any path capable of replacing the bootstrap catalog
without the Phase 7 progression. The product is unreleased: no alias,
compatibility adapter, legacy feature, or fallback executor is preserved.

Proof: source, public-export, Cargo graph, fixture, and filesystem artifact
absence gates.

## Slice 7F: closure evidence

Run the focused Store, physical-backend, scheduler, physical-isolation,
layout-indexes, test-support, operations, and certification tests affected by
the cutover. Run formatting, clippy, Rust line caps, boundary check, and
agent-context check. Resolve every material review finding before Phase 8.

## Out of scope

Phase 7 does not create the final caller handle, physical acknowledgment,
automatic retry policy, recovery replay, semantic commit, branch authority, or
stable-reader reclamation. It may complete the narrow Phase 8 facade cutover
needed to remove a competing current-root writer, but that does not close
Phase 8.
