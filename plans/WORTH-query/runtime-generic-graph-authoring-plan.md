# WORTH Query Runtime Generic Graph Authoring, Mixed-Shape Composition, And Identity-Preserving Existing-Truth Plan

> **Status:** Implemented and closed upstream hardening gate
>
> **Shipped closeout:** [runtime-generic-graph-authoring-closeout.md](./runtime-generic-graph-authoring-closeout.md)
>
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./worth_query_vision.md)
>
> **Test requirements:** [test-requirements-milestone-9_3-and-runtime-gates.md](./test-requirements-milestone-9_3-and-runtime-gates.md)
>
> **Primary predecessor:** [runtime-authoritative-mutation-evidence-plan.md](./runtime-authoritative-mutation-evidence-plan.md)
>
> **Primary downstream pressure:** [../worth/worth-query-runtime-kernel-hard-break.md](../worth/worth-query-runtime-kernel-hard-break.md)
>
> **Primary owners:** `worth-query`, `worth-runtime-bridge`, and production runtime adapters in downstream domains
>
> **Prerequisite milestones and gates:**
> - [runtime-api-public-stabilization-plan.md](./runtime-api-public-stabilization-plan.md)
> - [runtime-authoritative-mutation-evidence-plan.md](./runtime-authoritative-mutation-evidence-plan.md)
>
> **Concurrent downstream programs:**
> - [../worth/worth-query-runtime-kernel-hard-break.md](../worth/worth-query-runtime-kernel-hard-break.md)
>
> **Impacted later roadmap work:**
> - `Milestone 9.4` (`Temporal Query Basis And Time-Aware Subscription Contracts`)
> - `Milestone 9.5` (`Async And Resource-Backed Query Families`)
> - `Milestone 10` (`Store-Backed Execution And Historical Parity`)
> - `Milestone 11` (`Durable Saved-Query, Cursor, And Artifact Reload Semantics`)
>
> **Primary architectural driver:** make graph-shaped mutation authoring
> physically executable through one ordinary Query runtime contract without
> allowing downstream domains to reintroduce shadow runtime semantics for
> relation rewrites, subgraph composition, or backend verification

## Goal

Harden WORTH Query's public mutation/runtime surface so serious downstream
domains can use Query as the ordinary graph-authoring runtime without keeping
domain-local substitutes for:

- identity-preserving existing-target relation rewrites
- invariant-complete same-batch graph composition
- generic mixed-shape graph composition that can combine create, update,
  retarget, and retire semantics in one authoring program
- backend-verified existing-truth checks on real bridge-backed runtimes

The result must be a domain-agnostic authoring contract that lets downstream
domains express graph-shaped workflows through Query-native mutation authoring,
receipts, inspection, support metadata, and typed denial rather than through
private authority glue or mirror runtimes.

## Why This Plan Exists

The runtime authoritative mutation evidence gate solved a major part of the
problem: public receipts now preserve target evidence, batch/session authority
meaning, causality/provenance, existing-truth bindings, and admitted verified
mutation families strongly enough that downstream domains can stop rebuilding
explanation layers locally.

That was necessary. It is not yet sufficient for Query to feel like the real
runtime.

Downstream kernel pressure now exposes a broader remaining substrate problem
that is still too generic to leave inside domain-local adapters:

1. existing-target relation rewires need a true identity-preserving update lane
   rather than delete-plus-recreate disguise
2. same-batch graph authoring needs a first-class public composition surface
   rather than a fragile pile of scalar symbolic writes
3. same-batch graph authoring must widen from a pile of named lucky workflows
   into a generic mixed-shape composition engine that can preserve honest
   semantics across created and existing targets, preserved identity, and
   retirement within one canonical program
4. backend-verified existing-truth checks need a stable ordinary story on real
   bridge-backed runtimes, not just on memory or compatibility slices

If these gaps are not solved in WORTH Query itself:

- downstream domains will author relation rewires through local shadow
  semantics that violate existing-target meaning
- graph-shaped create-plus-attach workflows will drift into one-off builder
  tricks rather than one generic runtime story
- downstream pressure will keep widening named workflow lanes one by one,
  turning the roadmap into a costly pile of narrow admissions instead of one
  capability-generic composition contract
- backend-verified checks will remain technically available in principle but
  operationally incomplete on the production runtime paths that matter
- Query will continue to look like the runtime while domains quietly keep the
  real hard mutation semantics above or beside it

This plan exists to close those generic substrate gaps once, upstream, so the
same public mutation/runtime facade can honestly serve as the daily-driver
runtime for graph-shaped domains.

## Hard Part

The hard part is not adding three more facade methods.

The hard part is keeping six things separate that a weaker runtime will blur
together the moment downstream pressure increases:

- existing-target mutation semantics that preserve target identity
- replacement workflows that only look update-shaped at the API boundary
- same-batch graph composition semantics that preserve symbolic intent,
  ordering, and resolved-target meaning
- generic mixed-shape composition semantics that preserve lifecycle meaning
  across create, update, retarget, and retire steps without collapsing back
  into scalar batch folklore
- backend-verified existing-truth semantics that depend on lower-runtime truth
  authority rather than Query-local assertions
- support metadata and certification evidence that must report the same truth
  as the runtime without requiring internal-code archaeology

The design fails if:

- a relation "update" is implemented as create-plus-delete under a nicer name
- graph composition is really just scalar batch mutation plus raw symbolic
  string folklore
- graph composition remains a growing list of named downstream workflows rather
  than becoming one capability-generic composition contract
- bridge-backed verification is documented as ordinary while production
  runtimes still deny or degrade it silently
- support metadata says "supported" for substrate that only works on
  compatibility or memory runtimes
- downstream domains still need private authoring builders to express one
  invariant-complete graph workflow honestly

This plan therefore has to define one exact public authoring contract, one
runtime-support contract, and one proof contract that all three missing
capabilities inherit.

## Explicit Assumptions

- the Runtime API Public Stabilization Gate remains the governing public facade
  shape for workspaces, handles, inspection, and support metadata
- the Runtime Authoritative Mutation Evidence Gate remains the governing
  evidence story for target binding, causality, provenance, batch/session
  aggregation, and typed denial
- lower runtimes and bridges remain authoritative for relation identity,
  replay, merge, naming, and verification semantics
- Query may orchestrate mutation authoring and evidence, but it may not invent
  truth identity semantics that the lower runtime cannot preserve
- same-batch symbolic references already exist as substrate, but they are not
  yet sufficient on their own to count as a first-class graph composition
  surface
- downstream domains such as Worth are valid pressure tests for generic
  runtime capability, but they do not get to define the public Query contract
  or the roadmap shape of generic graph composition
- production bridge-backed runtime support is the governing support bar; memory
  and scaffold runtimes may lead implementation, but they do not define
  completion

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects is solving the real substrate gap rather
  than widening pretty wrappers. This plan therefore starts from the hostile
  downstream cases that would force shadow runtimes back into existence.
- `arch_laws.md`
  The most important thing it protects here is explicit proof-bearing public
  surfaces. Existing-target update, graph composition, and verified
  existing-truth support must become typed, inspectable, and support-reportable
  runtime contracts rather than metadata conventions.
- `perf_laws.md`
  The most important thing it protects is breadth and coordination honesty.
  Graph composition and verified existing-truth support must lower once and
  execute once; they may not broaden into repeated rediscovery, implicit scans,
  or domain-local patchup passes.
- `domain_laws.md`
  The most important thing it protects is responsibility separation. Generic
  graph authoring and existing-target mutation semantics belong in Query; the
  downstream domain should still own vocabulary, invariants, and interpretation.
- `worth_query_vision.md`
  The most important thing it protects is the product promise that developers
  declare query intent once and reuse one runtime surface for reads, writes,
  subscriptions, branches, history, and explanation. This plan extends that
  promise into graph-shaped authoritative mutation.
- `worth_query_roadmap.md`
  The most important thing it protects is roadmap sequencing. This plan belongs
  under the Runtime Authoritative Mutation Evidence Gate and before downstream
  domains widen more workflow families on top of incomplete generic substrate.
- `test-requirements-milestone-9_3-and-runtime-gates.md`
  The most important thing it protects is certification-grade proof. This plan
  must end in named runtime-gate suites with hostile rows, compile-fail
  boundaries, and exact digest/counter output rather than feature demos.
- `runtime-authoritative-mutation-evidence-plan.md`
  The most important thing it protects is the existing public evidence model.
  This plan must extend that model without inventing a second mutation contract
  or weakening the already-shipped target/causality/provenance story.
- `runtime-api-public-stabilization-plan.md`
  The most important thing it protects is beautiful ordinary DX. The new
  surfaces must feel like stable public runtime authoring, not expert-only
  lower-level seams.
- `../worth/worth-query-runtime-kernel-hard-break.md`
  The most important thing it protects is the downstream kernel bar. Worth
  should not solve generic graph authoring, verified mutation, or identity-
  preserving rewires locally; if those are needed, the fix belongs here.

## Adversarial Constraint

For the same graph-shaped workflow expressed through the public Query facade,
the runtime must preserve the same target identity meaning, graph-composition
meaning, verification meaning, and receipt/inspection meaning regardless of
whether the workflow:

- mutates an existing relation in place
- creates several entities and relations in one same-batch subgraph
- mixes created targets, existing targets, preserved identity, and retirement
  inside one same-batch composition program
- verifies existing truth before mutate/delete continuation
- executes live on a production bridge-backed runtime
- is denied because the runtime cannot yet preserve the required meaning

If any admitted path:

- rewrites an existing-target relation by deleting one identity and creating a
  new one under an "update" label
- makes graph-shaped authoring depend on caller-owned ordering folklore or raw
  symbolic identity strings without one composition contract
- widens support by admitting another named downstream workflow while the
  generic composition engine still cannot explain or certify the same lifecycle
  shape upstream
- exposes a verified existing-truth surface in the facade but leaves production
  bridge-backed runtimes unable to admit it honestly
- forces downstream domains to distinguish "real runtime support" from
  "technically present but not production-grade" by reading internal code

then this plan has failed.

The public authoring surface must make the same canonical meanings available on
real runtime paths or deny typed and early before domains are tempted to fill
the gap themselves.

## Geometry Kernel Pressure Cases

This plan remains domain-agnostic, but its runtime bar is intentionally shaped
by hostile graph programs that geometry kernels routinely need.

These are not topology-specific implementation requirements. They are generic
pressure cases that the substrate must survive honestly.

Required hostile graph programs:

- `EdgeSplit`
  - an existing edge relation is retired
  - two new edges are created
  - one new vertex is created
  - adjacency relations are rewired
  - old edge identity is either preserved as lineage parent or explicitly
    retired
  - receipt must prove lineage and identity outcomes rather than forcing the
    kernel to reconstruct them from raw deltas

- `LoopSuccessorRewire`
  - an existing successor relation retargets from old successor to new
    successor
  - relation identity must remain preserved
  - verification must assert old source and target before mutation
  - denial must distinguish identity-preservation failure from target-family
    unsupportability

- `FaceInnerLoopInsertion`
  - a loop entity is created
  - a relation from an existing face to the symbolic loop is created
  - symbolic edges and vertices may also be created in the same program
  - the resolution map must expose all symbolic-to-resolved identities

- `FailedNonManifoldAdmission`
  - generic graph composition substrate can express the shape
  - a domain invariant hook denies the resulting topology
  - denial must not collapse into runtime support denial
  - support posture and domain-invalidity posture must remain explicitly
    distinguishable

These cases exist to keep the plan honest about the difference between:

- Query cannot express the graph composition
- the runtime cannot preserve the required identity semantics
- bridge-backed verification is unavailable
- the domain rejected the resulting graph as invalid

Those outcomes must not collapse into one generic graph-composition denial.

## Product Decision Lock

- Query remains a domain-agnostic mutation/runtime facade; it does not become a
  topology engine, CAD kernel, workflow author, or naming semantic authority.
- Existing-target update means identity-preserving update. Delete-plus-recreate
  may not masquerade as that surface.
- Same-batch graph authoring must be a first-class public composition story,
  not a downstream convention over scalar batch operations.
- Mixed-shape graph composition is a capability-generic runtime target, not a
  promise to admit every domain-specific workflow. Query owns generic
  composition semantics; domains still own their invariants.
- Backend-verified existing-truth support counts as "supported" only when real
  bridge-backed runtimes can admit it honestly through the ordinary facade.
- Receipts, inspection, support metadata, and typed denials must all agree on
  the new authoring families.
- If a required capability is missing, the fix belongs in WORTH Query and its
  bridge/runtime contracts, not in a domain-local wrapper that restores the old
  dual-runtime shape.
- Extension hooks are allowed only when they preserve the canonical runtime
  semantics. Domains may extend lowering, invariant validation, capability
  declaration, and artifact interpretation; they may not override target
  identity semantics, receipt truth, support posture, or denial taxonomy.

Normative consequence:

- any implementation that exposes relation update while changing authoritative
  relation identity is out of spec
- any implementation that teaches graph composition as "just use batch plus
  strings carefully" is out of spec
- any implementation that documents verification as ordinary while support
  metadata or bridge-backed runtimes still deny the same family is out of spec
- any implementation that makes downstream domains compute support posture from
  internal bridge code instead of public support artifacts is out of spec

## Scope

### In Scope

- one exact public authoring family for identity-preserving existing-target
  relation update
- one exact public authoring family for same-batch graph composition
- one exact public bridge-backed support/admission family for backend-verified
  existing-truth operations on admitted entity and relation families
- receipt, inspection, support-profile, and support-matrix widening for all of
  the above
- compile-fail, support-closeout, and hostile certification proof surfaces
- public feature docs and roadmap/closeout updates that teach the new runtime
  literally

### Explicitly Out Of Scope

- topology-specific, workflow-specific, CAD-specific, or naming-specific
  semantic helpers
- downstream invariant packs for any one domain's complete graph workflow
- lower-runtime implementations of relation identity mutation that are not
  actually supported by the underlying truth engine
- store-backed historical parity, temporal basis semantics, or async resource
  scheduling beyond the support/reporting obligations needed to keep this work
  honest

## Extensibility And Hook Boundaries

This plan must leave room for native extension on top of the mixed-shape
composition engine without reopening the shadow-runtime problem it is trying to
close.

Allowed extension hooks:

- domain-side intent lowering into the generic graph-composition program
- domain-owned invariant packs that validate whether an otherwise-admitted
  generic composition is valid for that domain
- domain capability declarations built on top of admitted generic substrate
- domain interpretation layers that consume canonical Query receipts and
  inspection artifacts to produce domain diagnostics, certification output, or
  UX-facing summaries

Not allowed as extension hooks:

- alternate target identity semantics
- alternate relation update semantics that disguise replacement as update
- alternate receipt or inspection truth that competes with the canonical Query
  artifact family
- domain-local support posture that broadens or narrows the public support
  contract privately
- ad hoc symbolic resolution rules outside the Query-owned composition builder
  and lowering path
- arbitrary execution-mode injection that bypasses the program/lowering model

Rules:

- hooks may add domain meaning; they may not weaken generic runtime truth
- hooks must consume proof-bearing runtime artifacts rather than reconstructing
  parallel truths from raw deltas
- unsupported domain extensions must fail typed and early rather than falling
  back to hidden local execution semantics
- if a domain extension needs new generic lifecycle or target semantics, the
  fix belongs in WORTH Query first and the hook may only consume that widened
  substrate afterward

## Required Runtime Contracts And Counters

The runtime must emit machine-checkable counter bundles for this gate instead
of only human-readable summaries.

Required output bundle:

- `counter_snapshot`

Rules:

- `counter_snapshot` must be emitted by named certification suites for this
  gate
- counters attach to authoring-family work units rather than one rolled-up
  "graph mutation" total
- support-closeout docs must quote the same counter families the tests certify

### Lifecycle Outcome Taxonomy

Mixed-shape graph composition must report lifecycle outcomes with enough
precision for lineage-heavy kernels to distinguish current-truth removal from
hard deletion or pre-execution denial.

Required lifecycle outcome taxonomy:

- `Created`
- `UpdatedIdentityPreserved`
- `RetargetedIdentityPreserved`
- `RetiredCurrentTruth`
- `SupersededWithLineage`
- `DeletedIfUncommitted`
- `DeniedBeforeExecution`

Rules:

- `retire` may not remain a vague umbrella for delete, supersede, or hide
- lifecycle outcome surfaces must distinguish current-topology removal from
  hard deletion whenever lineage or replay semantics are preserved
- denied work must report `DeniedBeforeExecution` rather than pretending to be
  a partial lifecycle receipt
- first-ship mixed-shape certification must freeze lifecycle outcome meaning in
  receipt and inspection output, not just final row shape

### Identity-Preserving Update Contract

Identity-preserving existing-target update must preserve one authoritative
target identity before and after mutation.

Required contract surfaces:

- `DeclaredExistingTarget`
- `ResolvedExistingTarget`
- `ExistingTargetIdentityDigest`
- `IdentityPreservingUpdateSupportVerdict`
- typed denial taxonomy for unsupported identity-preserving update families

Required counters and outputs:

- existing-target update component count
- target-identity-preserved count
- target-identity-denied count
- touched-aspect breadth
- affected live/computed breadth

Rules:

- no admitted update path may change authoritative relation identity
- identity-preservation success or denial must be visible in receipt,
  inspection, and support metadata
- batch/session summaries must not collapse identity-preserving update into the
  same family meaning as create-plus-delete
- lowerings that require delete-plus-create replacement must deny before
  mutation execution begins rather than after a provisional plan is built

### Graph Composition Contract

Graph composition must preserve symbolic authoring intent as one explicit
runtime program rather than one caller-owned ordering convention.

Required contract surfaces:

- `GraphCompositionProgram`
- `GraphCompositionSymbol`
- `GraphCompositionResolutionMap`
- `GraphCompositionBreadth`
- `GraphCompositionEvidence`
- `GraphCompositionSupportVerdict`
- typed denial taxonomy for unresolved, illegal, or unsupported composition
  edges

Required counters and outputs:

- symbolic entity count
- symbolic relation count
- existing-target edge count
- symbolic entity follow-up mutation count
- symbolic relation follow-up mutation count
- symbolic relation retirement count
- symbolic-resolution count
- graph breadth and component-order count
- lifecycle outcome breadth by taxonomy family

Rules:

- symbolic references must be typed handles, not raw public strings
- composition must lower once into canonical mutation commands
- receipts and inspection must expose symbolic-to-resolved mapping explicitly
- receipts and inspection must expose canonical lowered program meaning
- mixed existing-target and symbolic edges are part of first-ship completion;
  "symbolic entity creation only" does not count as generic graph composition
- mixed create/update/delete/retarget closures are the target completion shape;
  endlessly adding named downstream workflow admissions is not
- composition lowering may not rediscover graph ordering or target shape by
  rereading workspace state during execution
- graph-composition denial must stay distinguishable from:
  - lower-runtime identity-preservation denial
  - bridge verification unavailability
  - domain invariant denial
- domain-invalid graph programs must not be reported as runtime-support denial
  when the generic composition substrate could otherwise express them honestly

### Bridge-Backed Verification Contract

Bridge-backed verification support must remain ordinary only when the real
runtime can execute it with the same public meaning as the facade advertises.

Required contract surfaces:

- `BridgeBackedVerificationSupportVerdict`
- `VerifiedAssumptionSet`
- `AssumptionSnapshotDigest`
- `VerifiedPreconditionDigest`
- per-family verification support rows
- typed denial taxonomy for unsupported verification substrate

Required counters and outputs:

- verified-assertion family count
- verified-update family count
- verified-delete family count
- verification-read-set breadth
- verification-denial count by family

Rules:

- support must be reported per family, not as one vague verification bool
- production bridge-backed runtimes are the completion bar
- compatibility or memory support may not be reported as ordinary production
  support
- verification denials must distinguish unsupported bridge substrate from
  target-shape or collection mismatch so downstream callers can react honestly
- verification evidence must distinguish:
  - target binding evidence
  - verification assertion evidence
  - verified assumption/read-set evidence
  - mutation result evidence

### Verified Assumption And Snapshot Contract

Bridge-backed verification must preserve the exact preconditions that a mixed-
shape operation depended on, not just the fact that some check happened.

Required contract surfaces:

- `VerifiedAssumptionSet`
- `AssumptionSnapshotDigest`
- `VerifiedPreconditionDigest`
- `VerificationReadSetBreadth`

Rules:

- a verified mutation must be able to say "this operation was admitted only if
  these old truths still held at this snapshot basis"
- assumption/read-set evidence must stay distinct from target binding evidence
  and from mutation result evidence
- collaborative editing, replay, branching, and live UI consumers must be able
  to distinguish:
  - binding target resolution
  - verified old-truth assumptions
  - the snapshot basis those assumptions were read from
  - the mutation results produced after admission
- first-ship certification must freeze assumption/read-set breadth and digest
  outputs for at least one preserved-identity retarget case

### Admission Trace Contract

Denied graph work must expose an admission-classification path that is richer
than a final denial code and explicitly separate from execution receipts.

Required contract surfaces:

- `GraphCompositionAdmissionTrace`
- `AdmissionTraceStage`
- `AdmissionTraceDigest`

Required stages:

- `ProgramParsed`
- `SymbolsValidated`
- `LoweringValidated`
- `CapabilityFamilyClassified`
- `SupportPostureResolved`
- `IdentityPreservationEvaluated`
- `VerificationSubstrateEvaluated`
- `DomainInvariantEvaluated`
- `DeniedBeforeExecution`

Rules:

- admission traces are for denied or fail-closed work; they are not receipts
- admitted work may summarize admission stages, but denied work must expose the
  exact stage boundary that prevented execution
- domain invariant denial must remain distinguishable from runtime support or
  identity-preservation denial
- first-ship certification must include at least one denied trace that reaches
  domain-invariant evaluation and one that fails earlier in generic runtime
  admission

## Compile-Time Boundary Rule

The public API must make the proof-bearing runtime boundary mechanically
enforceable instead of convention-based.

Required compile-fail boundaries:

- external callers cannot mint graph-composition symbols directly
- external callers cannot mint graph-composition resolution maps directly
- external callers cannot mint identity-preserving update support verdicts
  directly
- external callers cannot mint bridge-backed verification support verdicts
  directly
- external callers cannot submit raw symbolic string identities where typed
  graph-composition handles are required
- external callers cannot fabricate proof-bearing inspection/evidence handles
  for identity-preserving update or graph-composition families

Required implementation consequence:

- proof-bearing authoring artifacts are constructed only by Query-owned
  builders, lowering, execution, and inspection paths
- facade-visible types should make the illegal states above unrepresentable or
  uncompilable rather than merely documented as forbidden

## Required Public Authoring Surfaces

The resulting runtime surface must make these authoring shapes ordinary,
typed, and inspectable.

### Identity-Preserving Existing-Target Relation Update

```rust
let binding = workspace.bind_existing_relation(
    WORTHQueryExistingRelationTarget::new(
        format!("{relation_id:?}"),
        live_query_identity.clone(),
    )?
    .in_target_collection("WorthTopologyRelation")?,
)?;

let receipt = workspace.update_existing_verified(
    binding,
    |assertion| {
        assertion
            .aspect("topology.kind", "worth.half_edge_next")
            .aspect("topology.source_identity", old_source_identity.clone())
            .aspect("topology.target_identity", old_target_identity.clone())
    },
    |update| {
        update
            .aspect("topology.target_identity", new_target_identity.clone())
            .touches(["topology.boundary", "diagnostics.decisions"])
            .metadata("domain.family", "RewireLoopSuccessor")
    },
)?;
```

### First-Class Same-Batch Graph Composition

```rust
let receipt = workspace.compose_graph(|graph| {
    let inner_loop = graph.insert_entity("WorthTopologyEntity", |insert| {
        insert
            .aspect("topology.kind", "worth.loop")
            .aspect("topology.structure", "cube.face.inner_loop")
            .aspect("naming.persistent_name", "cube.face.inner_loop")
    })?;

    graph.insert_relation("WorthTopologyRelation", |insert| {
        insert
            .aspect("topology.kind", "worth.face_inner_loop")
            .source_existing(face_identity)
            .target_symbolic(inner_loop.identity())
            .touches(["topology.boundary", "diagnostics.decisions"])
    })?;

    Ok(())
})?;
```

### Bridge-Backed Backend-Verified Existing-Truth Check

```rust
let binding = workspace.bind_existing_entity(
    WORTHQueryExistingEntityTarget::new(
        format!("{entity_id:?}"),
        query_identity.clone(),
    )?
    .in_target_collection("WorthTopologyEntity")?,
)?;

let receipt = workspace.verify_existing(binding, |assertion| {
    assertion
        .aspect("topology.kind", "worth.vertex")
        .aspect("naming.persistent_name", "cube.vertex.1")
})?;
```

These are required output standards, not suggestive examples. Equivalent names
are acceptable only if the semantic contract remains equally explicit.

## First-Ship Scope Rule

The first ship may be conservative. It may not be vague.

Required first-ship topology:

- one public workspace-owned graph composition entry surface
- one typed graph-composition builder context
- one typed symbolic-handle family returned from that builder context
- one canonical lowering artifact for composed graph programs
- one support verdict family for identity-preserving update support
- one support verdict family for bridge-backed verification support
- one receipt/inspection evidence path that reuses the same public mutation
  evidence story as ordinary scalar mutation

Required first-ship posture:

- identity-preserving relation update support must be declared per admitted
  target family
- graph composition support must be declared per admitted edge/target family
- bridge-backed verification support must be declared per admitted verification
  family
- denial posture must be emitted through the same support artifacts and
  inspection vocabulary as success posture

Explicit first-ship debt:

- relation families that still require lower-runtime identity support before
  admission
- graph workflows that still require downstream invariant-complete subgraph
  widening
- verification families that remain unavailable on production bridge-backed
  runtimes even if memory or scaffold runtimes can prove them earlier

Allowed first-ship conservatism:

- entity-family support may land before relation-family support where the lower
  runtime genuinely differs
- one graph-composition facade may ship with a narrower admitted workflow set
  than scalar batch mutation
- bridge-backed verification may ship per family instead of universally

Required first-ship honesty:

- every admitted family must be named explicitly in support metadata
- every denied neighbor must be named explicitly in support metadata
- docs and examples must use only admitted first-ship families unless a denial
  example is the point
- first-ship debt must be recorded as explicit blocked families, not implied by
  silence

Not allowed as first-ship debt:

- relation replacement disguised as identity-preserving update
- raw string symbolic references as the public composition contract
- bridge-backed verification documented as ordinary when only compatibility or
  memory runtimes support it
- support artifacts that say "generic graph authoring supported" while only one
  narrow builder path actually works

## Phases

### Phase 1: Freeze The Public Authoring Vocabulary And Capability Taxonomy

Lock one coherent public vocabulary for the three missing substrate families
before implementation spreads the wrong names.

Must ship:

- one public authoring family for identity-preserving existing-target relation
  update
- one public authoring family for same-batch graph composition
- one public capability taxonomy for graph composition breadth so support can
  distinguish:
  - admitted composition lifecycle steps
  - admitted mixed-shape target combinations
  - denied-but-planned neighbors
- one public support/admission family for bridge-backed backend-verified
  existing-truth checks
- typed receipt and inspection accessors that preserve the same evidence story
  already required by the authoritative mutation evidence gate
- one explicit extension-hook taxonomy that distinguishes:
  - allowed lowering hooks
  - allowed invariant-pack hooks
  - allowed interpretation hooks
  - forbidden semantic-bypass hooks
- support-matrix rows that distinguish:
  - stable public runtime surfaces
  - admitted but bridge-backed-runtime-incomplete surfaces
  - unsupported neighbors
- compile-fail boundaries for proof-bearing support verdicts and graph
  composition evidence artifacts

Must preserve:

- aspect-native insert/update/delete/batch remain the ordinary vocabulary
- new graph-oriented authoring surfaces compose with existing target evidence
  rather than replacing it
- public names remain domain-neutral and cost-honest
- extension taxonomy remains capability-oriented and does not become a hidden
  second runtime contract

This phase is complete only when a downstream engineer can tell, from public
types and support metadata alone, which graph-authoring/runtime surfaces are
stable, denied, or still deferred.

### Phase 2: Identity-Preserving Existing-Target Update And Retarget Substrate

Add a real existing-target relation update lane whose semantics preserve target
identity rather than hiding a replacement workflow behind update-shaped names.

Must ship:

- a public relation binding path for existing-target updates on ordinary
  bridge-backed runtimes
- `update_existing(...)` and `update_existing_verified(...)` support for
  relation targets where the authoritative relation identity remains the same
  before and after mutation
- explicit typed denial for:
  - unsupported identity-preserving relation update families
  - collection mismatch
  - target-shape mismatch
  - backend verification unsupported
- support for retargeting an identity-preserved relation toward admitted
  created-or-existing target references where lower truth can still preserve
  authoritative relation identity honestly
- receipt and inspection evidence that preserve:
  - binding family
  - declared target
  - resolved target
  - target identity digest
  - causality/provenance digest
  - touched-aspect fallout
- exact denial taxonomy rows for:
  - identity-not-preservable on this runtime
  - unsupported relation family
  - target mismatch
  - verification unavailable on this runtime

Must preserve:

- Query does not invent relation identity semantics; lower truth runtimes still
  decide whether a relation can be updated in place
- if the lower runtime cannot preserve identity, Query denies typed and early
- batch/session aggregate evidence remains honest about this mutation family
- created-target retarget support may not be reported as admitted unless the
  final receipt and inspection surfaces can preserve one truthful target story

This phase is complete only when an existing-target relation rewrite can be
expressed through the ordinary public facade without delete-plus-recreate
disguise and without domain-local target recovery.

### Phase 3: Generic Mixed-Shape Graph Composition Program

Make same-batch graph construction an explicit public runtime capability whose
program model can express mixed create/update/retarget/retire lifecycles rather
than a downstream convention over scalar batch operations.

Must ship:

- one public composition surface for same-batch graph authoring, such as
  `compose_graph(...)` or an equally explicit family
- explicit symbolic entity/relation handles produced within the composition
  block and reused through typed identity references rather than raw strings
- explicit composition lifecycle steps for:
  - symbolic entity declaration
  - symbolic relation declaration
  - symbolic entity follow-up mutation
  - symbolic relation follow-up mutation
  - symbolic relation retirement
- composition-level receipts and inspection that preserve:
  - component ordering
  - symbolic-to-resolved target mapping
  - graph breadth counters
  - canonical lowered program meaning
  - affected live/computed breadth
  - typed denial for unresolved or illegal composition edges
- support for mixed existing-target and same-batch symbolic references inside
  one composition block
- support for mixed create/update/delete/retarget closures where the runtime
  can preserve one coherent target-identity and lifecycle story
- compile-fail boundaries that prevent public fallback to raw symbolic strings

Must preserve:

- composition lowers once into canonical mutation plans; execution does not
  rediscover the graph shape at the hot path
- composition does not hide domain invariants; unsupported or incomplete graph
  workflows still deny typed and early
- scalar batch APIs remain available for non-graph workflows
- composition widening is capability-generic rather than a permanent pile of
  named downstream workflow exceptions

This phase is complete only when downstream domains can author one admitted
mixed-shape multi-entity, multi-relation program through one public composition
surface without stitching together raw symbolic identity folklore or
workflow-local lifecycle tricks themselves.

### Phase 4: Capability-Generic Admission And Denial For Mixed-Shape Composition

Replace workflow-by-workflow admission folklore with one explicit support and
denial story for mixed-shape graph composition capabilities.

Must ship:

- support metadata that reports graph-composition support by admitted
  capability family, not by one vague "graph authoring supported" bit
- typed denial that distinguishes:
  - unresolved symbolic references
  - illegal program ordering
  - unsupported lifecycle-step combinations
  - unsupported target-shape combinations
  - lower-runtime identity-preservation gaps
  - bridge verification substrate unavailable
  - domain invariant denial after generic composition admission
  - incomplete invariant-bearing subgraphs that generic Query is not allowed to
    admit yet
- canonical admission rules for composition receipts so:
  - real composition receipts expose composition program, breadth, resolution,
    and evidence
  - reconstructed or generic scalar batches fail closed and cannot impersonate
    those artifacts
- support and denial participation rules for domain extension hooks so
  downstream code can add domain validation or interpretation without privately
  mutating the generic support contract
- admission traces that show where denied work failed without forcing callers
  to reverse-engineer denial from logs or internal branching
- exact counters for admitted versus denied composition capability families

Must preserve:

- support posture remains explicit and machine-checkable
- denied neighbors fail typed and early before domains are tempted to widen
  them locally
- generic capability admission does not erase the distinction between substrate
  support and domain-specific invariant closure
- denied operations must preserve the distinction among:
  - Query cannot express the composition
  - the runtime cannot preserve identity
  - verification substrate is unavailable
  - the domain rejected the result as invalid
- extension hooks remain consumers of runtime truth rather than alternate
  producers of it

This phase is complete only when engineers can widen composition capability by
reading one capability/support contract rather than by hunting for named
workflow exceptions in downstream code.

### Phase 5: Bridge-Backed Backend-Verified Existing-Truth Execution

Turn backend-verified existing-truth checks into a real ordinary runtime
capability on production bridge-backed runtimes.

Must ship:

- bridge-backed runtime support for:
  - `verify_existing(...)`
  - `probe_existing(...)`
  - `update_existing_verified(...)`
  - `delete_existing_verified(...)`
  on admitted entity and relation families
- public support metadata that reports bridge-backed verification posture
  honestly by family
- typed denial for bridge-backed runtimes that lack required verification
  substrate
- verification receipts and inspection bundles that preserve the same target
  evidence, causality, provenance, and mode distinction already frozen by the
  mutation-evidence gate
- verified assumption/read-set surfaces that preserve precondition snapshots
  and read-set breadth for admitted verified mutations
- explicit support rows that distinguish:
  - admitted on production bridge-backed runtimes
  - admitted only on non-production runtimes
  - denied everywhere

Must preserve:

- verification support remains lower-runtime-authority-dependent rather than
  fabricated by Query
- unsupported families fail closed rather than degrading into retained local
  assertions with the same public shape
- production runtimes and memory/scaffold runtimes remain phase-typed in
  support posture
- assumption/read-set evidence stays distinct from binding and result evidence
  in receipt, inspection, and certification output

This phase is complete only when backend-verified existing-truth support is
ordinary and support-reportable on real bridge-backed runtimes, or denied typed
and early with no ambiguity.

### Phase 6: Support, Documentation, And Certification Closeout

Close the gate with machine-checkable proof, frozen support metadata, and
developer-facing documentation that teaches the new runtime honestly.

Must ship:

- support-matrix rows and support-profile tests for all newly admitted or
  denied authoring families
- compile-fail boundaries preventing external minting of proof-bearing support,
  closeout, or graph-composition evidence artifacts where appropriate
- documentation that teaches the allowed extension-hook boundaries explicitly,
  including examples of domain lowering and invariant validation that do not
  fork the runtime truth contract
- feature docs that show:
  - identity-preserving relation update authoring
  - graph composition authoring
  - bridge-backed verification authoring
  - typed denial and support-report reading
  - assumption snapshot / read-set evidence reading
  - admission trace reading for denied work
- roadmap and closeout documents updated so downstream domains can cite one
  stable upstream contract instead of oral tradition

Must preserve:

- the public docs teach only admitted stable or explicitly denied/deferred
  surfaces
- deleted builder-shaped seams and any surviving lower-level mutation seams
  remain named honestly and are not mixed into the new authoring story

This phase is complete only when public docs, roadmap placement, support
metadata, and certification suites all tell the same story.

## Practicality QA Findings

This plan was revised after hostile review for practical implementation shape.

Findings that needed correction:

1. The earlier version was strong on semantic direction but too weak on file
   ownership boundaries. That would have encouraged continued growth of
   catch-all files such as `graph_composition.rs`, `runtime_batch_writes.rs`,
   and `workspace_graph.rs`.
2. The earlier version widened composition semantics without naming where
   extension hooks, admission classification, and lowering ownership should
   live. That would have made the hook story easy to implement as ad hoc
   branching inside unrelated modules.
3. The earlier version named certification obligations but did not give a
   decomposed test-module topology. That would have increased the chance that
   graph-composition certification turns back into a few oversized test files.

Correction rule:

- every new concern in this plan must have an explicit module home
- workspace entrypoints stay thin and delegate immediately
- runtime surface artifacts stay separate from mutation lowering
- certification files split by concern rather than by chronology
- decomposition is the default; merging later is cheaper than surgery later

## Implementation Topology And File Skeleton

The plan is not considered practical unless engineers can see, before
implementation, where the code is expected to live.

The following skeleton is the intended default topology. Equivalent names are
acceptable only if ownership boundaries remain equally explicit.

### Runtime Entry And Thin Facade

These files should stay thin:

- `crates/worth-query/src/runtime/workspace_graph.rs`
  - owns only the public `compose_graph(...)` entrypoint and immediate
    delegation into the composition subsystem
- `crates/worth-query/src/runtime/runtime_batch_writes.rs`
  - owns batch execution orchestration, not graph-program planning semantics
- `crates/worth-query/src/runtime/error.rs`
  - owns typed runtime error integration, not denial construction logic

Rule:

- if a change is primarily about graph-program semantics, it should not be
  implemented first in `workspace_graph.rs` or `runtime_batch_writes.rs`

### Graph Composition Mutation Subsystem

Expected home:

```text
crates/worth-query/src/runtime/mutation/graph_composition/
  mod.rs
  builder.rs
  symbols.rs
  declarations.rs
  lifecycle.rs
  lowering.rs
  admission.rs
  admission_trace.rs
  denial.rs
  hooks.rs
  capability_families.rs
```

Ownership:

- `builder.rs`
  - user-facing composition builder state
  - sequencing of builder calls
  - no receipt/evidence shaping
- `symbols.rs`
  - typed composition-local symbol handles and symbol bookkeeping
  - no lowering logic
- `declarations.rs`
  - entity and relation declaration operations
  - declaration-time validation only
- `lifecycle.rs`
  - follow-up mutation and retirement steps
  - lifecycle-step classification
- `lowering.rs`
  - canonical lowering from composition program into runtime write commands
  - no public support wording
- `admission.rs`
  - capability-family classification for mixed-shape composition combinations
  - no typed denial formatting text
- `admission_trace.rs`
  - stage-by-stage denied-path trace assembly
  - no receipt shaping
- `denial.rs`
  - graph-composition-specific denial artifact construction
  - denial kind mapping from lower/runtime outcomes
- `hooks.rs`
  - extension hook contracts for lowering, invariant packs, and interpretation
  - must not own alternate execution semantics
- `capability_families.rs`
  - capability-family names and helper classification used by support artifacts

Practical rule:

- if `graph_composition.rs` survives as a single file, that is a temporary
  transition state, not the intended end-state topology

### Identity-Preserving Update And Retarget Subsystem

Expected home:

```text
crates/worth-query/src/runtime/mutation/existing_update/
  mod.rs
  binding.rs
  identity_preserving.rs
  retarget.rs
  verified.rs
  assumptions.rs
  denial.rs
```

Ownership:

- `binding.rs`
  - existing-target binding assembly and binding-family classification
- `identity_preserving.rs`
  - ordinary identity-preserving update semantics
- `retarget.rs`
  - created-or-existing target retarget rules under preserved identity
- `verified.rs`
  - verified update entry shaping, not bridge support reporting
- `assumptions.rs`
  - verified assumption set and snapshot/read-set digest shaping
- `denial.rs`
  - typed denial construction for mismatch, unsupported family, and
    identity-not-preservable outcomes

Rule:

- relation retarget logic and entity retarget logic may share helpers, but
  family-specific rules should not collapse into one giant switch file

### Support And Capability Classification

Expected home:

```text
crates/worth-query/src/runtime/support/graph_composition/
  mod.rs
  capability_rows.rs
  denial_classes.rs
  hook_rows.rs
  admission_trace_rows.rs
```

Ownership:

- `capability_rows.rs`
  - public support-family rows for mixed-shape composition capabilities
- `denial_classes.rs`
  - public fail-closed denial class naming for composition capability posture
- `hook_rows.rs`
  - public support posture for allowed extension-hook classes where needed
- `admission_trace_rows.rs`
  - public naming for admission-trace stages and denial boundaries

Rule:

- support wording must not be assembled ad hoc inside mutation lowering files

### Surface Artifact Topology

Public proof-bearing surface types should stay structurally separate and small.

Preferred home:

```text
crates/worth-query/src/runtime/surface/graph_composition/
  mod.rs
  breadth.rs
  evidence.rs
  program.rs
  resolution_map.rs
  lifecycle_outcome.rs
  admission_trace.rs
  denial.rs
  support_verdict.rs
```

If migration cost makes a subdirectory unnecessary immediately, the existing
separate files in `surface/` may remain, but the ownership split above must
still hold.

### Inspection Topology

Expected home:

```text
crates/worth-query/src/runtime/inspection/unified/
  batch_write.rs
  component.rs
  graph_composition.rs
  admission_trace.rs
```

Ownership:

- `graph_composition.rs`
  - inspection-facing composition projection helpers
  - composition-specific digest/counter rendering
- `admission_trace.rs`
  - denied-path inspection projection helpers
  - stage/digest rendering for admission traces

Rule:

- `batch_write.rs` should aggregate, not become the permanent home for every
  composition-specific inspection rule

### Test And Certification Topology

Expected home:

```text
crates/worth-query/src/runtime/tests/mutation/graph_composition/
  mod.rs
  declarations.rs
  lifecycle.rs
  mixed_shape.rs
  denial.rs
  admission_trace.rs
  boundary.rs
  support.rs
  geometry_pressure.rs

crates/worth-query/src/runtime/tests/assembly/support_profile/
  authority_evidence_closeout.rs
  graph_composition_capabilities.rs

crates/worth-query/tests/ui/
  runtime_graph_composition_symbol_constructor_private.rs
  runtime_graph_composition_program_constructor_private.rs
  runtime_graph_composition_evidence_constructor_private.rs
  runtime_graph_composition_resolution_map_constructor_private.rs
  runtime_graph_composition_denial_constructor_private.rs
```

Test ownership:

- `declarations.rs`
  - symbolic declaration happy paths
- `lifecycle.rs`
  - follow-up mutation and retirement lifecycle cases
- `mixed_shape.rs`
  - mixed created/existing/update/delete/retarget programs
- `denial.rs`
  - typed composition denials
- `admission_trace.rs`
  - denied-path stage and digest certification
- `boundary.rs`
  - reconstructed/generic batch fail-closed behavior
- `geometry_pressure.rs`
  - hostile geometry-inspired generic graph programs
- `support.rs`
  - support/admission and capability-family freezes for composition

Rule:

- one test file per concern, not one test file per milestone turn

### Hook Integration Topology

Allowed extension hooks should have explicit contract homes:

```text
crates/worth-query/src/runtime/mutation/graph_composition/hooks.rs
crates/worth-query/src/runtime/surface/graph_composition/support_verdict.rs
```

The intended hook classes are:

- lowering hooks
- invariant-pack hooks
- interpretation hooks
- admission-trace interpretation hooks

The following are intentionally not hook homes and should not grow alternate
semantics:

- `workspace_graph.rs`
- `runtime_batch_writes.rs`
- downstream domain facades

### Phase-To-Module Mapping

Minimum practical mapping for implementation planning:

- Phase 1
  - `mutation/graph_composition/capability_families.rs`
  - `support/graph_composition/*`
  - `surface/graph_composition/support_verdict.rs`
- Phase 2
  - `mutation/existing_update/*`
  - `surface/graph_composition/lifecycle_outcome.rs`
- Phase 3
  - `mutation/graph_composition/{builder,declarations,lifecycle,lowering}.rs`
  - `surface/graph_composition/{program,breadth,evidence,resolution_map}.rs`
- Phase 4
  - `mutation/graph_composition/{admission,admission_trace,denial,hooks}.rs`
  - `inspection/unified/{graph_composition,admission_trace}.rs`
- Phase 5
  - bridge-backed support rows plus verified update/delete execution glue
  - `mutation/existing_update/assumptions.rs`
- Phase 6
  - feature docs, closeout docs, support freezes, compile-fail fixtures

### File Size Discipline For This Plan

This plan inherits the workspace rule that code and test files stay at 400
lines or fewer by default.

Implementation consequence:

- if a graph-composition or mixed-shape concern threatens to push a file above
  the cap, split by responsibility immediately
- do not defer decomposition into a later cleanup batch
- if a current file is already near the cap, treat extraction as part of the
  feature, not as optional polish

## Must Ship

- one public identity-preserving existing-target relation update family
- one public composed graph authoring family
- one public mixed-shape graph composition capability contract
- one honest bridge-backed backend-verified existing-truth support story
- support metadata, receipts, and inspection for all of the above
- hostile certification suites and compile-fail boundaries for the new
  families
- public documentation updates that teach the new runtime authoring surfaces
  directly

## Must Preserve

- lower runtimes remain authoritative for truth identity, replay, naming, and
  verification semantics
- Query remains domain-agnostic and does not absorb topology-specific concepts
- unsupported or incomplete graph workflows fail typed and early
- existing target evidence, causality, provenance, and batch/session authority
  evidence remain one coherent public story
- public API stabilization vocabulary remains the governing facade shape

## Required Documentation Updates

This plan is not closed until documentation reflects the new support literally.

Must update:

- [worth_query_roadmap.md](./worth_query_roadmap.md)
  - place this gate intentionally under the runtime authoritative mutation
    evidence family
- [runtime-authoritative-mutation-evidence-plan.md](./runtime-authoritative-mutation-evidence-plan.md)
  - widen the gate narrative so the new authoring families are part of the
    same public evidence contract rather than a side appendix
- [test-requirements-milestone-9_3-and-runtime-gates.md](./test-requirements-milestone-9_3-and-runtime-gates.md)
  - add named certification suite rows for the new authoring families
- public feature docs for:
  - runtime mutation authoring
  - support matrix / admission reading
  - graph composition authoring
  - existing-truth verification authoring
- closeout docs for the runtime authoritative mutation evidence gate once this
  plan is implemented

Documentation must explicitly distinguish:

- stable public runtime support
- denied-but-planned neighbors
- deleted builder-shaped mutation seams and any surviving lower-level mutation
  seams
- first-ship debt that remains explicit after implementation
- any fallback or workaround examples must be labeled non-ordinary and may not
  appear before the canonical authoring examples in feature docs

## Acceptance Evidence

- one named certification suite:
  `Runtime Identity-Preserving Existing-Target Relation Update Test`
  proving identity-preserving relation updates remain identity-preserving in
  receipts, inspection, aggregate evidence, and support metadata
- one named certification suite:
  `Runtime Mixed-Shape Graph Composition Test`
  proving graph composition preserves symbolic references, resolved target
  mapping, typed ordering meaning, graph breadth counters, lifecycle counters,
  lifecycle outcome taxonomy, admission traces, and typed denial under hostile
  incomplete-subgraph and mixed-shape conditions
- one named certification suite:
  `Runtime Bridge-Backed Verified Existing-Truth Support Test`
  proving verified existing-truth surfaces are admitted or denied honestly on
  bridge-backed runtimes by family, not by one vague support bit
- support-matrix, support-profile, and public closeout tests updated for the
  new families
- compile-fail tests proving external callers cannot mint proof-bearing support
  or composition evidence artifacts directly
- compile-fail tests proving external callers cannot fake graph-composition
  symbols, support verdict artifacts, or identity-preserving-update evidence
  handles directly
- feature docs and examples updated to teach the new surfaces and no longer
  imply the old narrower substrate story
- exact counter assertions for:
  - identity-preserved versus denied update families
  - symbolic entity/relation breadth
  - symbolic lifecycle-step breadth
  - verification read-set breadth
  - lifecycle outcome breadth by taxonomy family
  - symbolic-resolution count
  - mixed-shape composition admitted versus denied capability count
  - bridge-backed verification admitted versus denied count by family
- machine-checkable output bundles from each named certification suite
  including:
  - `truth_snapshot`
  - `inspection_snapshot`
  - `support_snapshot`
  - `assumption_snapshot`
  - `admission_trace_snapshot`
  - `counter_snapshot`

## Sequencing Notes

- This plan belongs under the Runtime Authoritative Mutation Evidence Gate,
  not as a separate parallel milestone, because it extends the same public
  mutation/runtime evidence story rather than defining a new runtime family.
- It should land before downstream domains widen more ordinary graph workflows
  on top of Query, because otherwise those domains will be forced to invent
  local substitutes for one of the remaining generic composition surfaces.
- Further Worth kernel widening should be treated as hostile certification
  pressure, not as the roadmap driver for generic composition semantics.
- The roadmap priority after the current admitted surfaces is capability-generic
  mixed-shape composition closure, not indefinite accumulation of tiny named
  workflow lanes.

## Architectural Notes

- The "graph authoring" surface is generic runtime vocabulary, not a topology
  vocabulary. It should be usable by any domain that authors related entities
  and relations in one batch.
- The "identity-preserving update" surface is the semantic opposite of
  delete-plus-recreate disguise. If the lower runtime cannot do it honestly,
  denial is the correct behavior.
- The bridge-backed verification requirement is partly a runtime-adapter
  hardening program, but it is still a Query plan because the public facade
  and support contract must expose the result honestly.
- This plan should produce one stable upstream dependency contract that the
  Worth hard-break program can cite directly when deleting more domain-local
  mutation glue.
