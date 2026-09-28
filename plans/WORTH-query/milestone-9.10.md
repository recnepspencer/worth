# Milestone 9.10 Engineering Spec: Graph Read Access Planning And Declarative Index Admission

> **Status:** Completed.
>
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Primary predecessor:** [milestone-9.9.md](./milestone-9.9.md)
>
> **Purpose:** make declared graph reads compile into proof-bearing,
> budget-admitted access plans so covered read surfaces cannot perform hidden
> N+1 traversal, cannot build unbounded in-memory indexes, and cannot teach AI
> agents a stronger graph/index mental model than the runtime can enforce.

## Goal

WORTH Query must derive graph read access needs from the declared read shape,
plan the required adjacency/index posture before execution, and either execute
through an admitted bounded plan or fail with a typed posture that explains the
missing support, budget excess, or required materialization lane. The common
authoring path must stay declarative, but every hidden-looking convenience must
lower into proof-bearing artifacts that can be inspected, tested, replayed, and
refused before execution.

The product target is not "cache more graph data." The product target is:

```text
read declaration
-> admitted schema/query references
-> admitted basis, policy, tenant, and relationship-proof bindings
-> graph read operation resolution
-> canonical graph read access shape
-> selectivity and cardinality estimate
-> required access structures, worksets, proof buffers, and lifecycle support
-> admitted inline, paged streaming, persistent-index-required,
   store-backed-index-required, async-materialized,
   access-capability-registration-required, or denied
-> receipt proving no hidden N+1 and no unbounded background indexing
```

## Why This Milestone Exists

The current docs correctly point agents away from caller-owned graph loops and
toward Query-owned graph/index views, but the runtime is not yet robust enough
to honor that promise across broad boolean predicates, multi-relation edge
walks, dense graph neighborhoods, policy/tenant narrowing, and wild traversal
shapes. Milestone `9.9` gives read/touch descriptors and obligation dispatch
authority. Milestone `9.10` uses that foundation to make graph access planning
itself a first-class Query authority boundary before store-backed execution
inherits the runtime-backed read model.

## Governing Summaries

- `MENTALITY.md`: protect the adversarial constraint first; the spec must build
  the foundation that survives worst-case graph reads instead of shipping a
  convenience cache and hoping to optimize later.
- `arch_laws.md`: semantic intent must compile into execution strategy,
  applicability must be pre-solved at entry, lowered plans must be the only
  executor input, cost boundaries must be explicit in API shape, and each phase
  output must carry the proof established by that phase.
- `dx_laws.md`: good DX is organized truth, not cute syntax; friendly
  declarations must lower into inspectable plans, stringly names are acceptable
  only at declaration boundaries, and proof-carrying progression must appear in
  the type system without making the common path ceremonial.
- `composition_laws.md`: planning, estimation, admission, execution,
  diagnostics, and certification must live in named files and functions rather
  than collapsing into broad read helpers.
- `domain_structure_laws.md`: authoritative truth, derived indexes, cached
  access structures, and diagnostic artifacts need separate structural homes;
  graph/index projections must be visibly derived and rebuildable.
- `perf_laws.md`: breadth must be bounded by semantic delta, expensive work
  must reject before construction, access patterns must reflect traversal
  locality, and every performance claim needs counters.
- `worth_query_roadmap.md`: Query owns typed expression, lowering, result
  shaping, and planning while lower runtimes keep truth authority; this
  milestone belongs before store-backed execution so store pushdown inherits a
  complete access-plan contract rather than discovering it later.

## Adversarial Constraint

For every covered graph read declaration, including broad boolean predicates,
multi-relation edge walks, dense frontier expansion, policy/tenant narrowed
reads, relationship-proof reads, reusable read families, live-promoted reads,
and preview/branch reads, Query must decide the access posture before expensive
execution begins. It must be impossible for covered surfaces to:

- perform caller-owned N+1 edge lookups
- materialize unbounded adjacency, frontier, visited, proof, or result buffers
- hide broad graph traversal behind cheap-looking APIs
- build background indexes without a lifecycle, budget, and receipt
- confuse persistent, runtime, ephemeral, streaming, async, and denied postures
- execute with a graph access plan that cannot be replayed, inspected, and
  compared through structural counters

## Product Decision Lock

- Declarative index admission means **automatic plan derivation**, not
  automatic eager materialization and not a magic "index everything" switch.
- Inline graph reads may use existing indexes, admitted runtime indexes, or
  bounded ephemeral indexes only within certified memory and breadth budgets.
- Dangerous broad reads must return typed outcomes such as
  `RequiresPersistentIndex`, `RequiresAsyncMaterialization`,
  `AdmittedPagedStreaming`, or `DeniedBudgetExceeded` before the hot path burns
  time or RAM.
- No graph read edge case may be recorded as an unnamed gap or catch-all
  bucket. Every case must have a named admission posture, required-capability
  posture, async/materialized posture, store-owned capability posture, or typed
  denial with receipt evidence.
- Store-owned durable index work must appear as a typed required-capability
  posture with an owning milestone, not as a missing runtime edge case.
- AI-facing docs must teach the exact boundary: Query learns access needs from
  declarations, but Query does not allocate the universe to satisfy an unsafe
  declaration.
- Public authoring APIs may remain concise, but the accountability surface must
  expose the exact access shape, access requirement rows, cost estimate, support
  inventory match, selected posture, and denial/materialization escape hatch
  before execution.
- Raw strings, raw relation names, and loose selectors may appear only at
  declaration/admission boundaries. After admission they must become typed
  proof-carrying references such as admitted projection fields, predicate
  fields, ordering fields, relations, traversal operators, and access-plan
  index rows.
- An executor must never accept raw read graphs, raw names, or merely validated
  selectors when an admitted graph access plan is required. The only executable
  graph-read input is the lowered proof-bearing access plan for the selected
  posture.
- WORTH Query must not hardcode every domain graph operation. Domains may
  register graph-read operations, but only if those operations lower into
  Query-readable access shapes, typed access requirements, conservative cost
  contracts, support/inventory matches, admission postures, admitted access
  plans, and standard receipts.
- Domains may extend graph-read receipt evidence with domain-specific counters
  or explanations. They may not replace Query's standard access counters,
  self-certify unsafe execution, satisfy support with strings or local caches,
  or execute callbacks that bypass Query-owned planning.

## DX And Proof-Bearing Contract

The milestone must preserve two truths at once:

- Common graph-read authoring stays declarative. A caller should describe the
  read family, root, schema view, traversal operator, predicates, ordering, and
  result shape as one semantic unit instead of manually registering unrelated
  planner, index, budget, and execution fragments.
- Phase progression stays structural. The runtime must lower that declaration
  through typed artifacts whose names and fields encode what has been proven:
  raw declaration, admitted schema references, canonical read graph, operation
  resolution, access shape, selectivity shape, access requirement set, cost estimate, support
  inventory match, access admission, lowered execution plan, and receipt.

The desired DX shape is:

```rust
let family = workspace.define_read_family("tenant-face-neighborhood", |read| {
    read.anchored_frontier_collection(/* declaration-owned read meaning */)
})?;

let plan = workspace.plan_graph_read_access(&family)?;

plan.required_indexes();
plan.cost_estimate();
plan.admission();
plan.explain();

let result = workspace.execute_read_family_with_access_plan(plan)?;
```

This is intentionally different from:

```rust
workspace.auto_index_every_graph_read();
```

The former keeps the common path readable while making the lowered access
contract inspectable. The latter hides which indexes, postures, costs, and
denials exist, and is prohibited.

The proof chain must make invalid shortcuts unrepresentable:

```text
Raw graph read declaration
-> AdmittedQuerySchemaReferences
-> WORTHQueryGraphReadBasisBinding
-> WORTHQueryGraphReadPolicyTenantProofBinding
-> WORTHQueryReadGraph
-> WORTHQueryGraphReadOperationResolution
-> WORTHQueryGraphReadAccessShape
-> WORTHQueryGraphReadAccessRequirementSet
-> WORTHQueryGraphReadIntrinsicCostEstimate
-> WORTHQueryGraphIndexInventoryMatch
-> WORTHQueryGraphReadSupportedCostEstimate
-> WORTHQueryGraphReadAccessAdmission
-> WORTHQueryAdmittedGraphReadAccessPlan
-> WORTHQueryGraphReadExecutionReceipt
```

Each arrow is an authority boundary. Each output type must expose only read-only
views of the proof it carries, and construction must be sealed to the function
that actually establishes the proof.

## Law 41 Proof-Type Requirements

This milestone is a law-41 milestone. It is not enough for APIs to return
`Result<T, E>` or for structs to contain the right fields. Every major artifact
must encode what has been proven about the value, who proved it, and which
phase may consume it next.

Required proof-type rules:

- Constructors for proof-bearing artifacts are private, crate-private, or
  otherwise sealed to the proving module. Public `new(...)` constructors are
  prohibited for artifacts whose names imply admission, planning, support,
  inventory matching, or execution eligibility.
- Fields on proof-bearing artifacts are private. Accessors expose read-only
  views, stable digests, explanatory rows, or typed references; they must not
  let callers synthesize or mutate the proof state.
- Phase transitions consume the prior proof type and produce the next proof
  type. A transition may not accept raw declarations, raw names, or weaker
  artifacts when a stronger proof has already been established.
- Distinct proof stages use distinct types even when their data layout is
  identical. A validated relation name, an admitted schema relation, a planned
  traversal relation, and an inventory-supported adjacency requirement are not
  interchangeable.
- Runtime assertions are not substitutes for type-level phase progression.
  Runtime checks may defend untrusted boundaries, but inside Query-owned
  lowering the compiler should make skipped phases uncallable.
- Any bypass needed for tests must live in a named test-only fixture or sealed
  harness module and must not be exported through the public facade.

The minimum proof-type transition table is:

| Phase | Input type | Output type | Proof established |
| --- | --- | --- | --- |
| Schema reference admission | raw declaration references | `WORTHQueryAdmittedQuerySchemaReferences` | referenced fields, relations, directions, and result-shape aliases exist in the admitted schema view |
| Basis and narrowing admission | raw basis, policy, tenant, and proof context | `WORTHQueryGraphReadBasisBinding` + `WORTHQueryGraphReadPolicyTenantProofBinding` | root universe and visibility/proof constraints are admitted before planning |
| Read graph canonicalization | admitted schema references + admitted basis/narrowing bindings | `WORTHQueryReadGraph` | read meaning is canonical, ordered, digestible, schema-admitted, and basis/narrowing-aware |
| Operation resolution | `WORTHQueryReadGraph` + operation registry | `WORTHQueryGraphReadOperationResolution` | built-in or domain-registered operation semantics are resolved, or a typed required-capability/unsupported denial is produced |
| Access-shape derivation | `WORTHQueryGraphReadOperationResolution` | `WORTHQueryGraphReadAccessShape` | traversal, root, predicate, ordering, basis, policy, tenant, and result-pressure semantics are classified |
| Selectivity normalization | `WORTHQueryReadGraph` + admitted predicate references | `WORTHQueryBooleanSelectivityShape` | boolean broadness, anchoring, intersection, and risky OR posture are normalized |
| Requirement derivation | access shape + selectivity shape | `WORTHQueryGraphReadAccessRequirementSet` | required adjacency, predicate, ordering, lifecycle, workset, proof, visited, dedup, and result-buffer structures are named as typed requirement rows |
| Intrinsic cost estimation | requirement set + runtime statistics proof | `WORTHQueryGraphReadIntrinsicCostEstimate` | abstract read pressure is computed conservatively before concrete support is considered |
| Inventory matching | requirement set + support inventory | `WORTHQueryGraphIndexInventoryMatch` | each requirement row is matched to admitted support or localized missing capability evidence |
| Supported cost estimation | requirement set + runtime statistics proof + inventory match | `WORTHQueryGraphReadSupportedCostEstimate` | concrete execution pressure under the matched support posture is computed conservatively |
| Admission | supported cost estimate + inventory match + budget | `WORTHQueryGraphReadAccessAdmission` | the read is classified as admitted, required-capability, async/materialized, store-owned, streaming, or denied |
| Lowering | access admission | `WORTHQueryAdmittedGraphReadAccessPlan` | selected posture, support evidence, budget, counters, and execution strategy are complete enough for the executor |
| Execution | admitted access plan | `WORTHQueryGraphReadExecutionReceipt` | execution consumed the admitted plan and emitted counters proving the planned posture actually ran |

Every row in this table needs positive tests proving the valid transition and
compile-fail or facade-boundary tests proving downstream code cannot construct
the output type directly, skip the input type, or pass a weaker artifact to the
next phase.

## Terminology Lock

This milestone must not use "index", "plan", "support", or "automatic" as
loose words. Each term names a different authority boundary.

- **Read declaration**: the authoring surface that describes graph-read meaning:
  root, scope class, traversal operator, predicates, ordering, projection,
  result shape, basis, policy, tenant, and relationship-proof posture.
- **Admitted schema/query reference**: a typed proof that a declaration-bound
  reference names a real schema field, relation, direction, result alias, or
  traversal operator in the admitted schema view.
- **Access shape**: the canonical graph-read cost/strategy shape derived from
  an admitted read graph before execution. It is not an executor observation.
- **Domain graph-read operation**: a domain-authored operation such as
  `face_neighborhood`, `construction_dependency_closure`,
  `workflow_invalidation_closure`, `where_used_expansion`, or
  `training_effectivity_chain`. A domain operation may teach Query new graph
  semantics only by registering a lowering contract into Query-owned planning
  artifacts.
- **Operation resolution**: the phase that classifies a graph-read operation as
  built-in, domain-registered, requiring access capability registration, or
  denied as unsupported before access shape derivation.
- **Access requirement set**: typed rows describing the access structures,
  execution worksets, proof obligations, buffers, lifecycle capabilities, and
  actual indexes a read would need. It is not evidence that those structures
  exist.
- **Index inventory**: the runtime/lower-runtime report of existing or missing
  graph access support. Inventory is derived support posture, not authority
  over truth.
- **Inventory match**: a proof artifact that compares requirement rows against
  inventory rows and localizes support, missing capability, lifecycle mismatch,
  complexity mismatch, or store-owned debt.
- **Declarative index admission**: the decision that classifies a read as
  inline-indexed, streaming, persistent-required, async/materialized,
  store-owned-required, capability-registration-required, live-maintenance-
  required, or denied. It is not eager construction of indexes.
- **Provisioning**: actual allocation or lifecycle creation of a bounded
  ephemeral, streaming, persistent, or materialized access resource. It may
  happen only after admission selects that posture.
- **Persistent index requirement**: a typed requirement that a reusable family
  needs persistent support. It is not a promise that durable store-backed
  persistence exists before Milestone `10`/`11`.
- **Store-owned index capability**: a requirement row whose realization belongs
  to `worth-store` or another lower runtime. Query records the required
  capability and owner; Query does not fake the support.
- **Graph read access plan**: the lowered proof-bearing execution input. It
  carries selected posture, requirement rows, inventory match, budget, support
  evidence, counters to collect, and denial/materialization alternatives.
- **Access receipt**: the post-execution or denied-path evidence that the
  selected access plan was consumed or refused exactly as admitted.

Law for all later sections:

```text
Automatic means derived and inspectable. It never means hidden, unbounded,
untyped, unbudgeted, or unexplainable.
```

## Covered Graph Read Definition

A covered graph read is any public Query read declaration, read family, read
intent, live read, preview/branch read, policy/tenant read, relationship-proof
read, or reference-consumer read surface that uses admitted graph relations,
traversal operators, or a registered domain graph-read operation and falls
within the graph-read access vocabulary.

Covered reads must enter graph-read access planning. They may not execute
through raw lower-runtime traversal, local graph caches, local adjacency maps,
manual relation loops, or helper wrappers that simulate a single graph read by
issuing repeated Query reads.

Excluded graph work:

- arbitrary graph algorithms outside the admitted access vocabulary
- offline research/debug scripts that are not product/runtime proof
- lower-runtime internals before they cross a Query public facade
- store-owned pushdown internals before the owning milestone admits them
- domain operations that cannot lower into Query access planning

Excluded work must not be documented as ordinary Query graph-read support.

## Domain-Pluggable Graph Operation Registry

WORTH Query is the platform-level access authority, not the owner of every
domain graph operation. Domains may register operation semantics, but Query owns
admission, planning, support, execution posture, and receipts.

The platform contract is:

```text
domain operation declaration
-> admitted domain references
-> Query-readable operation resolution
-> access shape
-> access requirement set
-> intrinsic and supported cost contracts
-> support/inventory match
-> admission posture
-> admitted access plan
-> standard receipt counters + optional domain receipt extension
```

Operation resolution outcomes:

- `BuiltInGraphReadOperation`: Query owns the operation semantics directly.
- `DomainRegisteredGraphReadOperation`: a domain registered a lowering contract
  that produces Query-readable access shape and requirement evidence.
- `RequiresAccessCapabilityRegistration`: the operation may be legitimate, but
  no admitted lowering/support capability exists yet.
- `DeniedUnsupportedShape`: the operation cannot honestly lower into Query
  access planning.

Domain operation registration must provide:

- operation identity, version, and domain owner
- admitted domain reference types
- lowering into Query-readable access shape dimensions
- access requirement derivation contract
- conservative intrinsic cost hints, when domain semantics affect cost
- support capability requirements
- standard counter mapping
- optional domain receipt extension schema
- docs/support row identity
- compile-fail or facade-boundary proof that domain code cannot execute the
  operation callback directly, return rows directly, or satisfy support locally

Domains may:

- register graph-read operation semantics
- provide conservative cost hints
- register support capabilities
- add domain receipt extensions
- define domain-specific denial explanations

Domains may not:

- scan graph edges locally
- allocate hidden adjacency maps or local caches
- return graph read rows directly from a callback
- self-certify unsafe execution
- satisfy support with strings, labels, local support rows, or caller-owned
  indexes
- replace Query's standard graph access counters

## Artifact Ladder

`9.10` must be implemented as an explicit graph-read access artifact ladder.
A later artifact may consume earlier artifacts; it may not re-open raw read
declarations, raw relation labels, schema strings, runtime observations, or
consumer-local graph walks when a proof-bearing artifact already exists.

Authoring and schema admission:

- `WORTHQueryGraphReadDeclaration`
- `WORTHQueryReadFamily`
- `WORTHQueryAdmittedQuerySchemaReferences`
- `WORTHQueryAdmittedGraphReadRelation`
- `WORTHQueryAdmittedGraphReadProjectionField`
- `WORTHQueryAdmittedGraphReadPredicateField`
- `WORTHQueryAdmittedGraphReadOrderingField`
- `WORTHQueryAdmittedGraphReadResultField`
- `WORTHQueryAdmittedGraphTraversalOperator`

Canonical read meaning:

- `WORTHQueryReadGraph`
- `WORTHQueryReadGraphDigest`
- `WORTHQueryGraphReadBasisBinding`
- `WORTHQueryGraphReadPolicyTenantProofBinding`

Operation resolution:

- `WORTHQueryGraphReadOperationRegistry`
- `WORTHQueryGraphReadOperationRegistration`
- `WORTHQueryGraphReadOperationResolution`
- `WORTHQueryBuiltInGraphReadOperation`
- `WORTHQueryDomainRegisteredGraphReadOperation`
- `WORTHQueryGraphReadOperationCapabilityRequirement`
- `WORTHQueryGraphReadOperationUnsupportedDenial`
- `WORTHQueryGraphReadDomainReceiptExtension`

Access shape and selectivity:

- `WORTHQueryGraphReadAccessShape`
- `WORTHQueryGraphReadAccessShapeDigest`
- `WORTHQueryBooleanSelectivityShape`
- `WORTHQueryPredicateSelectivityClass`
- `WORTHQueryGraphReadScopePressure`
- `WORTHQueryGraphReadResultPressure`

Requirement and support:

- `WORTHQueryGraphReadAccessRequirementSet`
- `WORTHQueryGraphReadAccessRequirementRow`
- `WORTHQueryGraphReadAccessRequirementKind`
- `WORTHQueryGraphReadAccessRequirementDigest`
- `WORTHQueryGraphIndexInventory`
- `WORTHQueryGraphIndexSupportRow`
- `WORTHQueryGraphIndexInventoryMatch`
- `WORTHQueryGraphIndexInventoryMatchRow`
- `WORTHQueryGraphIndexSupportPosture`

Cost, budget, and complexity:

- `WORTHQueryGraphReadIntrinsicCostEstimate`
- `WORTHQueryGraphReadSupportedCostEstimate`
- `WORTHQueryGraphReadBudget`
- `WORTHQueryGraphReadBudgetClass`
- `WORTHQueryGraphReadComplexityContract`
- `WORTHQueryGraphReadRuntimeStatisticsProof`
- `WORTHQueryGraphReadAccessCounterPlan`

Admission and execution plans:

- `WORTHQueryGraphReadAccessAdmission`
- `WORTHQueryGraphReadAccessAdmissionPosture`
- `WORTHQueryGraphReadAccessDenial`
- `WORTHQueryGraphReadRequiredCapabilityOwner`
- `WORTHQueryGraphReadAccessCase`
- `WORTHQueryGraphReadAccessCaseRegistry`
- `WORTHQueryAdmittedGraphReadAccessPlan`
- `WORTHQueryGraphReadAccessPlanExplanation`

Execution postures and lifecycle:

- `WORTHQueryInlineIndexedGraphReadExecution`
- `WORTHQueryEphemeralGraphIndexPlan`
- `WORTHQueryEphemeralGraphIndexScope`
- `WORTHQueryEphemeralGraphIndexReceipt`
- `WORTHQueryGraphReadStreamingPlan`
- `WORTHQueryGraphReadFrontierCursor`
- `WORTHQueryGraphReadStreamingReceipt`
- `WORTHQueryPersistentGraphIndexRequirement`
- `WORTHQueryGraphReadFamilyIndexContract`
- `WORTHQueryGraphReadMaterializationRequest`
- `WORTHQueryGraphReadMaterializationJob`
- `WORTHQueryGraphReadMaterializationProgress`
- `WORTHQueryGraphReadMaterializationReceipt`
- `WORTHQueryLiveGraphReadAccessPlan`

Receipt, diagnostics, and adoption:

- `WORTHQueryGraphReadExecutionReceipt`
- `WORTHQueryGraphReadAccessReceiptSummary`
- `WORTHQueryGraphReadAccessReplayRecord`
- `WORTHQueryGraphReadAccessCertificationMatrix`
- `WORTHQueryGraphReadBypassAuditRow`
- `WORTHQueryGraphReadAdoptionManifest`
- `WORTHQueryGraphReadResidueManifest`
- `WORTHQueryGraphReadAccessDocsAgreementProof`

## Existing Surface Inventory

The milestone should widen live Query surfaces before inventing new ones.

Current surfaces to preserve and extend:

- `workspace.compose_read(...)`
- `workspace.compose_read_with_invariant_pack(...)`
- `workspace.define_read_family(...)`
- `workspace.define_read_family_with_invariant_pack(...)`
- `workspace.execute_read_family(...)`
- `workspace.execute_read_family_in_basis_context(...)`
- `workspace.read_family_intent(...)`
- live read and subscription-adjacent read surfaces
- preview, branch, policy/tenant, and relationship-proof read surfaces
- `workspace.public_read_composition_support_report(...)`
- support matrix and admission reporting surfaces
- Consumer Kit hard-prohibition, boundary audit, support snapshot, support
  pinning, in-memory workspace, adoption, and residue machinery from `9.8`
- graph touch obligation descriptors and envelope vocabulary from `9.9`

New surfaces are allowed only where existing surfaces cannot honestly express:

- explicit graph-read access planning before execution
- proof-bearing schema/query reference admission for graph-read access planning
- required graph index rows and inventory-match rows
- typed cost estimate, budget, complexity, and counter-plan artifacts
- admitted access plans as executable graph-read input
- streaming, ephemeral-index, persistent-requirement, async-materialization, and
  live-maintenance access postures
- graph-read bypass audit rows and reference-consumer adoption manifests

## Planning Purity Boundary

The planning side of `9.10` has a pure, replay-stable core:

```text
admitted read graph
+ admitted basis/policy/tenant/relationship-proof bindings
+ operation registry
+ runtime statistics proof
+ index inventory
+ graph read budget
-> operation resolution
-> access shape
-> selectivity shape
-> access requirement set
-> intrinsic cost estimate
-> inventory match
-> supported cost estimate
-> access admission
-> admitted access plan or typed denial
```

This boundary may inspect support rows, runtime statistics proofs, and admitted
basis/narrowing artifacts. It may not inspect elapsed execution behavior,
mutate indexes, warm caches, scan graph edges to discover strategy, or run
caller-owned graph loops.

The execution side is deterministic only when the admitted access plan and its
declared state/basis inputs are part of the execution evidence:

```text
admitted access plan
-> selected executor posture
-> bounded resource lifecycle
-> structural counters
-> receipt / denial / progress / cursor evidence
```

This milestone must not describe final execution success as a pure product of
the read declaration alone. The read declaration derives requirements. Support,
budget, basis, policy, tenant, relationship proof, and runtime statistics decide
admission. Execution proves the admitted posture actually ran or stopped.

## Access Posture Matrix And Fallback Law

Every covered graph read access case must expose its legitimate access posture
alternatives before execution. A reusable low-latency family may require a
persistent index, the same semantic read may stream for first-page access, a
full export may require async materialization, and the current budget may deny
inline execution.

The rule is:

```text
Planning may expose multiple valid admitted or required postures.
Each execution attempt must select exactly one posture before work begins.
The executor may not switch postures without a new admission receipt.
```

Allowed postures:

- `AdmittedInlineIndexed`: existing or runtime-maintained support satisfies all
  requirements within inline budget.
- `AdmittedBoundedEphemeralIndex`: Query may build bounded read-scope or
  family-execution-scope derived structures within explicit lifecycle and
  memory budgets.
- `AdmittedPagedStreaming`: the read is too broad for inline materialization
  but can stream through bounded frontier pages with opaque basis-bound cursors.
- `RequiresPersistentIndex`: runtime-backed execution requires prebuilt
  persistent support, but durable store-backed realization is not implied.
- `RequiresStoreBackedPersistentIndex`: a durable index capability belongs to
  `worth-store` or another lower-runtime owner and is not admitted here.
- `RequiresAccessCapabilityRegistration`: the read shape cannot be classified
  until a domain/lower-runtime capability registers support posture.
- `RequiresAsyncMaterialization`: legitimate large reads must enter an admitted
  materialization request/job lane before work begins.
- `RequiresStoreBackedLiveAccessMaintenance`: one-shot execution may be
  possible, but live maintenance requires lower-runtime durable support.
- `DeniedBudgetExceeded`: the read exceeds declared budget and has no admitted
  streaming/materialization/capability posture for this request.
- `DeniedUnsupportedShape`: the read shape is outside the admitted graph-read
  access vocabulary.
- `DeniedPolicyTenantOrRelationshipProof`: narrowing/proof admission denies
  before access structures are built.

Forbidden fallback:

- inline plan silently degrades to streaming
- missing support silently uses local traversal
- broad read silently scans graph edges
- unsupported shape silently uses generic execution
- ephemeral allocation silently exceeds its admitted lifecycle or byte budget

Allowed stop-and-readmit:

- admitted plan stops with `ActualFrontierExceeded`
- admitted plan stops with `ActualResidentBytesExceeded`
- receipt suggests streaming, materialization, persistent support, or access
  capability registration
- caller explicitly requests a new posture
- new admission produces a new access plan and a new receipt identity

Matrix axes:

- scope class: local, anchored, frontier, broad search
- graph operator: direct edge, successor walk, bounded ancestor, bounded
  descendant, shared endpoint, shared attachment, anchored frontier,
  broad boolean frontier
- relation direction: outgoing, incoming, bidirectional, mixed
- predicate posture: anchored, selective, broad, unknown-conservative,
  OR-heavy, traversal-bearing
- result pressure: detail, ordered collection, unordered collection,
  aggregation-adjacent, proof-heavy
- basis posture: current, branch, preview, historical, reusable family,
  live-promoted
- narrowing posture: none, policy, tenant, relationship proof, combined
- support posture: existing runtime, bounded ephemeral, persistent required,
  store-owned required, capability-registration required, unsupported

The matrix must prove:

- every public graph read operator maps to a row
- no row uses a generic catch-all posture
- unsupported rows fail closed with typed evidence
- every admitted row names the access requirements, complexity contract, budget,
  and receipt counters
- every denied or required-capability row names the missing capability and the
  owning follow-on milestone or owner

## Access Requirement And Inventory Closure

Access requirement rows must be more precise than "needs an index." Actual
indexes are only one subset of the broader access requirement model.

Required row dimensions:

- requirement kind:
  `DirectionalAdjacency`, `ReverseAdjacency`, `PredicateSupport`,
  `OrderingSupport`, `TraversalWorkset`, `VisitedSet`, `DedupSet`,
  `ProofSupport`, `ResultBuffer`, `MaterializationLifecycle`,
  `LiveMaintenanceSupport`
- relation identity and relation authority
- direction
- traversal operator
- depth / frontier posture
- predicate family and predicate field authority
- ordering support
- lifecycle: existing runtime, bounded ephemeral, reusable-family persistent,
  store-backed durable, live-maintenance, materialized
- rebuild basis from authoritative truth
- invalidation basis
- complexity contract
- memory estimate basis
- proof/visited/dedup/result-buffer pressure

Inventory rows must name:

- lifecycle owner
- support owner crate/runtime
- support posture
- support state:
  `Declared`, `Measured`, `Certified`, `Available`,
  `TemporarilyUnavailable`, `StoreOwnedUnavailable`, or `Unsupported`
- rebuild basis
- invalidation basis
- supported relation direction and predicate/order families
- admitted complexity contract
- basis compatibility
- current availability
- store-gated owner milestone when not admitted

Inventory matching must produce localized outcomes:

- exact match
- direction mismatch
- predicate mismatch
- ordering mismatch
- lifecycle mismatch
- complexity mismatch
- budget mismatch
- missing rebuild basis
- missing invalidation basis
- store-owned capability required
- capability registration required
- unsupported shape

No inventory match may be satisfied by a caller-provided display label, string
name, or manually assembled support row.

## Read Lifecycle Semantics

One-shot reads, reusable read families, and live-promoted reads are not just
shape flags. They have different lifecycle semantics and different admission
rules.

- `OneShotReadDeclaration`: may admit inline indexed, bounded ephemeral,
  streaming, async materialization, required capability, store-owned
  requirement, or denial. It must not imply reusable low-latency support.
- `ReusableReadFamily`: may derive persistent index requirements, reusable
  support contracts, stable family index digests, and family-level support
  pins. It must not build hidden persistent support merely because execution
  repeats.
- `LivePromotedReadFamily`: may require live maintenance, invalidation
  support, durable lower-runtime support, and separate maintenance breadth
  budgets. A read that is safe one-shot may still deny live promotion.

Admission and receipts must preserve which lifecycle is being evaluated.

## Replay, Diagnostics, And Counter Closure

Access-plan replay must preserve:

- read-family digest
- admitted schema-reference digest
- read-graph digest
- access-shape digest
- selectivity-shape digest
- required-index-set digest
- inventory-match digest
- cost-estimate digest
- budget digest
- admission digest
- admitted-access-plan digest
- selected execution posture
- denial/materialization/streaming cursor identity where applicable
- structural counters

Diagnostics must localize failure to one phase:

- schema/reference admission
- read graph canonicalization
- access-shape derivation
- selectivity normalization
- access requirement derivation
- inventory matching
- cost estimation
- budget admission
- support/capability admission
- policy/tenant/relationship-proof admission
- access-plan lowering
- ephemeral-index provisioning
- streaming cursor admission
- async materialization request admission
- live-maintenance admission
- execution counter mismatch
- consumer bypass/adoption failure
- docs/support/certification disagreement

Counters must include at least:

- candidate roots estimated / touched
- relation requirements emitted
- inventory rows inspected
- support rows matched / missing
- edge touches estimated / actual
- frontier width estimated / actual
- frontier pages emitted
- visited entries estimated / actual
- dedup entries estimated / actual
- proof rows estimated / actual
- result rows estimated / actual
- resident bytes estimated / actual
- ephemeral bytes allocated / released
- cursor advances
- materialization checkpoints
- denied-before-edge-scan count
- fallback count, which must remain zero on admitted no-fallback lanes

Elapsed time may be recorded as diagnostic metadata, but never as the
complexity contract.

Receipts must be layered so ordinary reads do not carry enormous diagnostic
payloads by default:

- `WORTHQueryGraphReadAccessReceiptSummary`: compact default result attachment
  with stable digests, selected posture, support posture, and key counters.
- `WORTHQueryGraphReadExecutionCounterReceipt`: expanded structural counters
  for performance and certification.
- `WORTHQueryGraphReadReplayRecord`: replay inputs, digests, and posture
  evidence sufficient to reproduce the access decision.
- `WORTHQueryGraphReadDiagnosticEnvelope`: rich explanation materialized only
  when artifact policy asks for it.
- `WORTHQueryGraphReadDomainReceiptExtension`: optional domain-owned extension
  that may add counters/explanations but cannot replace standard access
  counters.

## Consumer Adoption And Anti-Folklore Contract

`9.10` is not closed while reference consumers still teach or depend on local
graph-read access folklore for covered surfaces.

Forbidden covered patterns:

- per-result neighbor lookups
- manual loops over relation rows
- ad hoc adjacency maps
- surface-local graph caches
- broad boolean scans hidden behind helper names
- local index names or local support rows that pretend to satisfy Query access
  requirements
- helper wrappers that call `compose_read(...)` repeatedly to simulate one
  graph read
- docs that teach raw lower-runtime graph traversal as ordinary read access

Consumer adoption requires:

- AST-level detection for loops over relation rows, repeated relation lookups
  inside result iteration, and local `HashMap`/adjacency construction from
  relation rows
- API-boundary detection for direct lower-runtime traversal calls from
  forbidden crates and helper fronts that call `compose_read(...)` repeatedly
  to simulate one graph read
- runtime counters for per-result neighbor lookup count, edge scans without
  admitted access-plan digest, and fallback count
- manifest evidence that every local graph access path is deleted, migrated, or
  classified as typed residue
- source-set audit rows keyed by authority violation, not fragile text alone
- false-positive proof for comments, docs, literals, and unrelated collection
  iteration
- adoption manifest rows for every migrated surface
- residue manifest rows for every explicitly deferred local access path
- support pins for required access-planning rows
- in-memory proof that consumers can test access planning without fabricated
  receipts

## Admitted Surface

The admitted product surface after this milestone is Query-owned graph-read
access planning and admission:

- graph read declarations lower into admitted schema/query references and
  canonical read graphs
- access shape, selectivity, access requirement set, inventory match, intrinsic
  and supported cost estimates,
  admission, and admitted access plan are sealed Query artifacts
- covered graph reads execute only through proof-bearing access plans
- receipts expose access-plan summaries and counters without requiring callers
  to re-run planning
- denied reads still publish admission envelopes
- inline, ephemeral, streaming, persistent-required, store-owned-required,
  capability-registration-required, async-materialized, live-maintenance-
  required, and budget-denied postures are typed and inspectable
- `worth-topo` and `worth-kernel` covered graph reads consume Query access
  receipts instead of deriving access adequacy locally

## Excluded Surface

This milestone does not claim:

- durable store-backed index realization unless supplied by the owning store
  milestone
- restart-stable saved access artifacts or durable cursor reload
- arbitrary graph algorithms outside the admitted read vocabulary
- final store-backed pushdown parity
- lower-runtime truth ownership inside Query
- live maintenance of every one-shot-safe read shape
- unbounded background index construction
- hidden query warmup side effects on read observation
- consumer-local indexes as admitted Query support
- raw relation traversal as an ordinary public graph-read path

## Workflow Surface

The ordinary graph-read access workflow after closure is:

```text
author graph read declaration
-> admit schema/query references
-> admit basis, policy, tenant, and relationship-proof bindings
-> canonicalize read graph
-> resolve built-in or domain-registered graph operation
-> derive access shape and selectivity shape
-> derive access requirement set
-> match against inventory/support rows
-> estimate intrinsic cost
-> estimate supported cost under matched inventory
-> apply budget
-> admit posture or return typed denial/required capability/materialization
-> execute admitted access plan or stream/materialize through admitted handle
-> attach access receipt to result / denial / progress / cursor
```

The ordinary reference-consumer migration workflow is:

```text
identify local graph-read access folklore
-> express the read as a Query read family or graph-read declaration
-> plan access and inspect access requirement rows
-> pin support posture and budget
-> execute through admitted access plan
-> delete local loop/cache/index helper
-> record zero-residue or explicit residue with owner and removal trigger
```

## Deletion Targets

The milestone is not closed until covered duplicates are deleted or named as
audited residue. Expected deletion target classes:

- topology read helpers that reconstruct adjacency from raw rows
- kernel construction read/check helpers that scan topology locally where Query
  can express the access shape
- test harness helpers that fabricate read receipts or index support rows
- local "required access requirement" lists outside Query support rows
- local source greps or audits that duplicate Consumer Kit graph-read bypass
  audits
- broad boolean read helpers that return ordinary inline payloads without
  access-plan evidence
- docs and examples that teach caller-owned graph traversal on covered lanes

## Allowed Residue

Residue is allowed only when all of the following are true:

- the covered read declaration or admitted schema reference cannot yet be
  produced from the real surface
- the missing access shape, support row, capability registration, or lower-
  runtime owner is named
- the owning crate and follow-on phase are named
- `introduced_in` names the phase that created or discovered the residue
- `must_not_exceed_count` caps the allowed residue rows for that class
- `removal_trigger` names the concrete access-plan, support-row, or lower-
  runtime capability that requires deletion
- the local path cannot be mistaken for ordinary supported Query authority
- a bypass audit row or adoption manifest row records the residue
- certification proves the residue does not apply to a covered lane

Residue is not allowed for:

- convenience wrappers around covered Query reads
- local per-node neighbor loops where a Query access plan exists
- local support rows that mirror Query support posture
- docs drift
- unbounded graph scans retained for "debugging"
- residue classes whose count grows after introduction

## Complexity / Proof Closure

The complexity proof surface must include:

- access-shape derivation complexity counters
- access-requirement derivation row counts
- inventory-match lookup counters
- no full-graph edge scan during planning
- denied-before-edge-scan proof for broad/budget-exceeded reads
- exact resident-byte estimates and actual resident-byte counters
- bounded ephemeral allocation/release counters
- streaming max-resident-frontier counters
- async materialization checkpoint and cancellation counters
- live maintenance breadth counters separate from one-shot execution breadth
- docs/support/certification agreement proof for every access posture
- consumer-kit certification proving adoption can be performed through
  Query-shipped planning, support-pin, manifest, audit, and in-memory workspace
  surfaces

## Closure Tiers

The milestone remains one closure target, but implementation planning should
distinguish four kinds of mandatory work:

Closure-critical:

- admitted schema/query references
- admitted basis, policy, tenant, and relationship-proof bindings
- canonical read graph
- graph-read operation resolution
- access shape
- selectivity shape
- access requirement set
- inventory/support model
- intrinsic and supported cost estimates
- budget and admission
- admitted access plan
- receipt summary

Posture-critical:

- inline indexed
- bounded ephemeral
- paged streaming
- persistent-required
- store-owned-required
- access-capability-registration-required
- async materialization
- live-maintenance-required
- typed denial

Adoption-critical:

- `compose_read`
- read families
- read intents
- live reads
- preview/branch reads
- policy/tenant/relationship-proof reads
- worth-topo migration
- worth-kernel migration
- consumer bypass audit
- residue manifests

Certification-critical:

- access posture matrix
- no-N+1 proof
- memory/breadth proof
- proof-type construction boundaries
- support/inventory agreement
- docs/support/AI_README agreement

No tier may be omitted from closeout. The tier split exists to make the work
reviewable, not to create optional deliverables.

## Phase Ordering Lock

Implementation order must follow the proof chain, regardless of how individual
workstreams are batched:

```text
1. admitted schema/query references
2. admitted basis, policy, tenant, and relationship-proof bindings
3. canonical read graph
4. graph-read operation resolution
5. access shape
6. selectivity algebra
7. access requirement set
8. inventory/support row vocabulary
9. inventory matching
10. intrinsic cost estimate
11. supported cost estimate
12. budget admission
13. selected posture / alternatives
14. admitted access plan
15. execution posture
16. receipt summary / counters / replay / diagnostics
17. public surface integration
18. reference adoption
19. hostile certification closeout
```

No phase may consume an artifact that has not been frozen by an earlier proof
transition. In particular:

- access shape consumes operation resolution, not raw read graph operation
  strings
- selectivity consumes admitted predicate references and access shape, not
  executor-side filter observations
- access requirements consume access shape and selectivity, not inventory
- inventory matching consumes access requirements, not cost estimates
- intrinsic cost is computed before support posture can make the read look
  cheap
- supported cost is computed after inventory matching
- admission consumes supported cost, inventory match, and budget
- execution consumes the admitted access plan

## Selectivity Algebra Lock

Phase 2 must define an explicit selectivity algebra, not only a list of desired
planner intuitions.

Required selectivity classes:

- `ExactAnchor`
- `TenantAnchor`
- `PolicyAnchor`
- `SelectivePredicate`
- `RangePredicate`
- `BroadPredicate`
- `UnknownPredicate`
- `TraversalPredicate`
- `DisjunctionBarrier`
- `IntersectionEligible`
- `PostTraversalOnly`

Required algebra rules:

- `AND` preserves the strongest available pre-traversal anchor unless a branch
  is `PostTraversalOnly`.
- `OR` loses anchor posture unless every branch shares the same admitted
  pre-traversal anchor.
- Traversal-bearing predicates cannot contribute to root selectivity unless
  supported by a registered capability.
- Unknown predicates are broad unless narrowed by a prior admitted anchor.
- Policy, tenant, and relationship-proof anchors participate before traversal,
  not as post-read filters.
- A broad OR over traversal-bearing branches cannot lower to inline execution
  unless a registered capability and budgeted posture admit it.
- Selectivity shapes retain branch identity for diagnostics and receipt
  evidence, even when canonical equivalence collapses ordering.

## Milestone Done When

This milestone is done only when:

- the artifact ladder is implemented through receipt attachment
- every Law 41 transition row has positive and construction-WORTHry proof
- every access posture matrix row has tests and evidence
- every public graph read operator maps to an access-case registry row
- every admitted posture carries access requirements, budget, complexity contract,
  and receipt counters
- every denied or required-capability posture localizes missing support and
  names the owner/follow-on milestone
- broad/dense reads deny, stream, require persistent support, or require async
  materialization before edge scans or unbounded allocation
- `compose_read`, read families, intents, live, preview, branch, policy/tenant,
  and relationship-proof reads preserve the same access-plan contract where
  covered
- `worth-topo` and `worth-kernel` covered duplicates are deleted or explicitly
  audited as residue
- docs, support rows, AI_README, and certification agree on the admitted
  posture vocabulary
- the closeout includes support rows, test-requirements matrix, bypass audit
  result, adoption manifests, residue manifests, and proof-type construction
  boundary evidence

## Phase Plan

### Phase 1: Graph Read Access Shape Vocabulary

Freeze a sealed `WORTHQueryGraphReadAccessShape` derived from canonical read
graphs, not from executor observations. The shape must encode root posture,
scope class, relation directions, traversal operators, maximum depth, frontier
fanout posture, predicate families, ordering, result-shape pressure, policy/
tenant/relationship-proof narrowing, basis posture, and whether the read is
one-shot, reusable, live-promoted, preview, or branch-scoped.

**Relevant subsystems**
- `crates/worth-query/src/runtime/workspace_queries.rs`
- `crates/worth-query/src/runtime/runtime_read_intents.rs`
- `crates/worth-query/src/runtime/surface/read_composition.rs`
- `crates/worth-query/src/policy_narrowing/`
- `crates/worth-query/src/relationship_proof/`

**Relevant APIs**
- `WORTHQueryReadGraph`
- `WORTHQueryReadFamily`
- `WORTHQueryReadReceipt`
- new: `WORTHQueryAdmittedQuerySchemaReferences`
- new: `WORTHQueryAdmittedGraphReadRelation`
- new: `WORTHQueryAdmittedGraphReadProjectionField`
- new: `WORTHQueryAdmittedGraphReadPredicateField`
- new: `WORTHQueryAdmittedGraphReadOrderingField`
- new: `WORTHQueryGraphReadAccessShape`
- new: `WORTHQueryGraphReadAccessShapeDigest`

**Warnings**
- Do not infer access shape from elapsed execution behavior; that turns a plan
  into a profiler artifact.
- Do not collapse relation direction into relation kind. Direction is a cost
  boundary and must survive canonicalization.
- Do not pass raw strings, merely validated names, or schema-unproven selectors
  into access-shape derivation. Access-shape derivation consumes admitted
  schema/query references produced by Query-owned validation.
- Do not make callers manually thread every proof artifact through ordinary
  authoring. The declaration surface may be concise, but the lowered artifacts
  must exist and be inspectable.

**Test requirements**
- Adversarial equivalence: semantically equivalent declarations with different
  builder ordering produce the same access-shape digest.
- Adversarial rejection: changing relation direction, maximum depth, predicate
  family, policy basis, or result-shape breadth changes the digest or produces
  a typed incompatibility.
- Adversarial locality: a one-hop direct edge read and a bounded successor walk
  over the same relation do not collapse into one access shape.
- Adversarial construction: downstream code cannot construct
  `WORTHQueryAdmittedQuerySchemaReferences`,
  `WORTHQueryAdmittedGraphReadRelation`, or
  `WORTHQueryGraphReadAccessShape` directly, and cannot derive an access shape
  from raw strings or merely validated names.

**Engineering decisions**
- Access shape is derived after read graph canonicalization and before
  execution admission.
- Shape fields are private; only Query-owned lowering can construct the sealed
  artifact.
- The public declaration may accept declaration-bound names where that keeps the
  authoring surface readable, but the canonical read graph stores admitted
  typed references, not raw strings.
- The access-shape artifact is the first graph-read phase output that encodes
  the proof "schema references and traversal operators have been admitted for
  this read graph."

**Open questions**
- None.

### Phase 2: Boolean Predicate Normalization And Selectivity Inputs

Normalize boolean predicate expressions into proof-bearing selectivity input
trees before graph expansion. The planner must know which branches are anchored,
which are broad, which can intersect before traversal, and which OR-heavy
branches require streaming or materialization.

**Relevant subsystems**
- `crates/worth-query/src/query_ast/`
- `crates/worth-query/src/planning/`
- `crates/worth-query/src/runtime/surface/read_composition.rs`

**Relevant APIs**
- predicate expression nodes
- `WORTHQueryReadGraph`
- new: `WORTHQueryBooleanSelectivityShape`
- new: `WORTHQueryPredicateSelectivityClass`

**Warnings**
- Do not evaluate predicates to discover selectivity. This phase produces
  planner input, not data-plane filtering.
- Unknown selectivity is not optimistic. Unknown broadness must classify as
  risky until a later phase proves otherwise.

**Test requirements**
- Adversarial equivalence: `A AND (B AND C)` and `(C AND A) AND B` normalize to
  identical selectivity shape when the predicates are semantically equivalent.
- Adversarial rejection: broad OR over traversal-bearing branches does not
  lower to inline execution unless it has a budgeted posture; otherwise it
  returns a typed streaming, materialization, required-capability, or denial
  posture.
- Adversarial ordering: selective anchored predicates are visible as eligible
  pre-traversal constraints, while broad predicates stay explicitly broad.
- Adversarial ladder: selectivity normalization consumes admitted predicate
  references and read-graph proof from the artifact ladder, not raw predicate
  strings or executor-side filter observations.

**Engineering decisions**
- Normalization preserves enough branch identity for diagnostics and receipts.
- The selectivity tree is an input to planning only; it is not an executor-side
  predicate interpreter.

**Open questions**
- None.

### Phase 3: Graph Read Operation Resolution

Resolve every graph-read operation before access-shape derivation. Query-owned
built-ins and domain-registered operations must both lower into Query-readable
operation resolution artifacts; missing domain capabilities must return typed
required-capability posture instead of falling back to generic traversal.

**Relevant subsystems**
- `crates/worth-query/src/runtime/surface/read_composition.rs`
- `crates/worth-query/src/runtime/support/`
- `crates/worth-query/src/planning/`
- domain capability contribution and support surfaces

**Relevant APIs**
- new: `WORTHQueryGraphReadOperationRegistry`
- new: `WORTHQueryGraphReadOperationRegistration`
- new: `WORTHQueryGraphReadOperationResolution`
- new: `WORTHQueryBuiltInGraphReadOperation`
- new: `WORTHQueryDomainRegisteredGraphReadOperation`
- new: `WORTHQueryGraphReadOperationCapabilityRequirement`
- new: `WORTHQueryGraphReadOperationUnsupportedDenial`
- `WORTHQueryReadGraph`

**Warnings**
- Do not hardcode domain-specific operations into Query built-ins merely
  because they are important to a reference consumer.
- Do not let domain operation registration execute callbacks, scan graph edges,
  allocate adjacency maps, or return rows directly.
- Do not classify a missing domain operation as generic traversal. It must be
  `RequiresAccessCapabilityRegistration` or `DeniedUnsupportedShape`.

**Test requirements**
- Adversarial built-in parity: built-in operation resolution is stable across
  equivalent read declarations.
- Adversarial domain registration: a domain-registered operation lowers to
  Query-readable access shape and requirement evidence without exposing a
  domain callback execution path.
- Adversarial missing capability: an unregistered but recognizable domain
  operation returns `RequiresAccessCapabilityRegistration` with owner/support
  evidence.
- Adversarial unsupported shape: arbitrary graph work that cannot lower into
  Query access planning returns `DeniedUnsupportedShape`.
- Adversarial construction: downstream code cannot construct operation
  resolution artifacts directly or promote a domain operation from strings.

**Engineering decisions**
- Operation resolution sits between canonical read graph and access-shape
  derivation.
- Domain operation registration is a semantic extension point, not an execution
  callback.

**Open questions**
- None.

### Phase 4: Access Requirement Set Derivation

Derive a `WORTHQueryGraphReadAccessRequirementSet` from operation resolution,
access shape, and selectivity inputs. It must name directional adjacency,
reverse adjacency, predicate support, ordering support, traversal worksets,
visited sets, dedup sets, proof support, result buffers, materialization
lifecycle, and live-maintenance support without deciding whether those
structures already exist.

**Relevant subsystems**
- `crates/worth-query/src/runtime/support/`
- `crates/worth-query/src/planning/`
- `crates/worth-relational/src/validation/`

**Relevant APIs**
- new: `WORTHQueryGraphReadAccessRequirementSet`
- new: `WORTHQueryGraphReadAccessRequirementRow`
- new: `WORTHQueryGraphReadAccessRequirementKind`
- `WORTHQueryGraphReadOperationResolution`
- `WORTHQueryGraphReadAccessShape`

**Warnings**
- Requirement derivation is not index construction. Mixing these steps hides
  missing support until allocation time.
- Do not model access requirements as strings. Requirement rows need typed
  relation, direction, predicate, lifecycle, workset, proof, and buffer fields.
- Do not allow a caller-provided index label to satisfy a requirement row.

**Test requirements**
- Adversarial parity: equivalent access shapes derive byte-identical access
  requirement sets with stable row ordering.
- Adversarial localization: changing one traversal relation changes only the
  relevant requirement row and receipt digest.
- Adversarial completeness: every traversal-bearing covered read derives at
  least one directional adjacency or traversal-workset requirement or returns a
  typed required-capability or denial posture.
- Adversarial construction: callers cannot WORTH requirement rows from display
  names, strings, or public struct expressions.

**Engineering decisions**
- Requirement rows include a rebuild basis proving the structure is derived
  from authoritative truth where applicable.
- Requirement set digest participates in read receipt identity.
- Requirement derivation consumes operation resolution, access shape, and
  selectivity shape; it does not re-open the raw authored declaration.

**Open questions**
- None.

### Phase 5: Access Cost Model And Budget Contract

Build a runtime-backed cost model that estimates frontier breadth, edge touches,
candidate roots, intermediate set size, index bytes, result bytes, proof bytes,
and allocation lifecycle before execution begins.

**Relevant subsystems**
- `crates/worth-query/src/planning/`
- `crates/worth-query/src/runtime/support/`
- `crates/worth-query/src/runtime/state_snapshot.rs`

**Relevant APIs**
- new: `WORTHQueryGraphReadIntrinsicCostEstimate`
- new: `WORTHQueryGraphReadSupportedCostEstimate`
- new: `WORTHQueryGraphReadBudget`
- new: `WORTHQueryGraphReadBudgetClass`
- new: `WORTHQueryGraphReadComplexityContract`

**Warnings**
- Elapsed time is not the contract. Structural counters are the contract.
- Estimates may be conservative; they may not under-report unknown broadness to
  admit unsafe execution.

**Test requirements**
- Adversarial budget: a dense boolean traversal with low-selectivity predicates
  predicts a broad posture and cannot enter inline indexed execution.
- Adversarial equivalence: two equivalent access shapes over the same runtime
  statistics produce identical cost-estimate digests.
- Adversarial memory: estimated index bytes include adjacency, reverse
  adjacency, frontier, visited, dedup, proof, and result buffers when relevant.
- Adversarial construction: budget classes, complexity contracts, and cost
  estimate statuses that imply admission cannot be publicly fabricated or
  upgraded from unmeasured/unknown-conservative evidence.
- Adversarial planning purity: cost estimation does not scan graph edges,
  allocate adjacency/frontier buffers, or observe elapsed execution behavior to
  choose a strategy.

**Engineering decisions**
- Default inline ephemeral index memory budget is small and explicit; raising it
  requires a typed budget admission in a later phase.
- Cost model status distinguishes `Measured`, `Estimated`, `UnknownConservative`,
  and `RequiresCapabilityRegistration`.

**Open questions**
- None.

### Phase 6: Access Admission Outcomes

Freeze typed access admission outcomes and an exhaustive graph read access-case
registry so budget, capability, store-ownership, and support decisions are
visible before execution. The system must be able to admit inline indexed,
paged streaming, prebuilt/persistent-index-backed, async materialized, require
store-backed capability, require access capability registration, or deny with a
structured reason. Admission must produce a proof-bearing artifact that is the
only valid input to graph-read execution for the selected posture.

**Relevant subsystems**
- `crates/worth-query/src/intent_admission/`
- `crates/worth-query/src/runtime/surface/read_composition.rs`
- `crates/worth-query/src/runtime/support/`

**Relevant APIs**
- new: `WORTHQueryGraphReadAccessAdmission`
- new: `WORTHQueryGraphReadAccessAdmissionPosture`
- new: `WORTHQueryGraphReadAccessDenial`
- new: `WORTHQueryGraphReadBudgetExceededDenial`
- new: `WORTHQueryGraphReadAccessCase`
- new: `WORTHQueryGraphReadAccessCaseRegistry`
- new: `WORTHQueryGraphReadRequiredCapabilityOwner`
- new: `WORTHQueryAdmittedGraphReadAccessPlan`
- new: `WORTHQueryGraphReadAccessPlanExplanation`

**Warnings**
- `DeniedBudgetExceeded` is not a soft warning. Covered execution must not run
  unless the caller re-declares the work through an admitted posture.
- `RequiresPersistentIndex` and `RequiresAsyncMaterialization` must be product
  postures, not string messages.
- Generic catch-all admission variants are prohibited. Unknown or
  not-yet-admitted shapes must classify as a named required capability, a
  store-owned requirement, or a typed denial.
- Do not make `compose_read(...)` or `execute_read_family(...)` silently perform
  graph access admission on a path that cannot be inspected. Friendly one-shot
  helpers may internally plan and execute, but their receipt must expose the
  same admission artifact a caller could have inspected explicitly.
- Do not allow execution APIs to accept weaker inputs than the proof chain
  already established. If an admitted access plan exists, executor entry takes
  that admitted plan, not the raw family plus ambient strategy flags.

**Test requirements**
- Adversarial rejection: a read whose estimated frontier exceeds inline budget
  returns `DeniedBudgetExceeded` or `RequiresAsyncMaterialization` before any
  edge scan counter increments.
- Adversarial parity: the same admitted read through one-shot and reusable
  family fronts produces the same access admission digest.
- Adversarial DX: denial carries access requirement rows, exceeded budget fields,
  and suggested posture without exposing internal executor topology.
- Adversarial inspectability: a caller can plan a graph read, inspect the
  access requirement rows, selected posture, cost estimates, support inventory match,
  and denial/materialization escape hatch, and then execute the exact admitted
  plan without the executor recomputing strategy.
- Adversarial construction: downstream code cannot construct
  `WORTHQueryGraphReadAccessAdmission` or
  `WORTHQueryAdmittedGraphReadAccessPlan` directly, cannot turn a denial into
  an admitted plan, and cannot call executor entrypoints with raw read
  families, raw read graphs, or caller-owned strategy flags where an admitted
  access plan is required.
- Adversarial edge cases: every public read operator, relation direction,
  predicate family, basis posture, live/preview/branch posture, and
  relationship-proof combination maps to an access-case registry row before any
  executor can run.

**Engineering decisions**
- Admission outcome attaches to the read receipt even on denied paths.
- Denials lower through typed stop classes rather than message matching.
- `WORTHQueryAdmittedGraphReadAccessPlan` consumes access admission plus the
  selected support/inventory evidence. It cannot be constructed from raw read
  graphs, raw index names, or caller-owned strategy flags.
- The access-case registry is the implementation checklist for this milestone:
  adding a new graph read capability requires adding the case row, tests,
  receipt posture, support-row behavior, and docs row in the same phase.

**Open questions**
- None.

### Phase 7: Existing Access Inventory And Support Rows

Expose runtime and lower-runtime graph access inventory through Query support
rows. Query must be able to answer whether a required access structure,
workset, proof support, lifecycle capability, or actual index already exists,
whether it is runtime-maintained, lower-runtime-owned, persistent, ephemeral,
store-owned-required, access-capability-registration-required, temporarily
unavailable, or denied by a typed support posture.

**Relevant subsystems**
- `crates/worth-query/src/runtime/support/`
- `crates/worth-query/src/facade/`
- `crates/worth-relational/src/`

**Relevant APIs**
- new: `WORTHQueryGraphIndexInventory`
- new: `WORTHQueryGraphIndexSupportRow`
- new: `WORTHQueryGraphIndexPosture`
- `WORTHQueryGraphReadAccessRequirementSet`

**Warnings**
- Inventory is not authority over truth. Indexes remain derived structures.
- Do not report `Verified` for a row unless rebuild basis, invalidation basis,
  and access complexity are certified.

**Test requirements**
- Adversarial parity: the same runtime assembly publishes identical inventory
  digests across repeated support-report calls.
- Adversarial drift: a missing access requirement localizes to the exact
  requirement row and cannot masquerade as broad support.
- Adversarial authority: inventory rows cannot be constructed by consumers or
  patched through facade escape hatches.
- Adversarial inventory closure: every inventory match outcome is one of the
  localized outcomes in the Requirement And Inventory Closure section; a
  generic "missing index" outcome fails certification.

**Engineering decisions**
- Inventory rows record lifecycle owner, derivation basis, invalidation basis,
  supported relation direction, and complexity contract.
- Store-backed persistent indexes must be visible as
  `RequiresStoreBackedPersistentIndex` rows with owning milestones; runtime-
  backed admission must not pretend they exist.

**Open questions**
- None.

### Phase 8: Bounded Ephemeral Index Provisioning

Build bounded ephemeral index provisioning for small admitted graph reads. The
runtime may build transaction/read-scope adjacency or predicate-support
structures only when the budget admits the allocation and lifecycle scope is
explicit.

**Relevant subsystems**
- `crates/worth-query/src/runtime/`
- `crates/worth-query/src/planning/`
- `crates/worth-query/src/runtime/surface/read_composition.rs`

**Relevant APIs**
- new: `WORTHQueryEphemeralGraphIndexPlan`
- new: `WORTHQueryEphemeralGraphIndexScope`
- new: `WORTHQueryEphemeralGraphIndexReceipt`

**Warnings**
- Ephemeral does not mean untracked. Allocation lifecycle, byte budget, and
  rebuild basis must be visible.
- Do not keep ephemeral indexes alive past their admitted scope because a later
  read "might use them."

**Test requirements**
- Adversarial budget: ephemeral provisioning rejects before allocation when the
  estimated index bytes exceed the inline ephemeral budget.
- Adversarial cleanup: after the read scope ends, ephemeral index lifecycle
  counters prove no orphan resource remains.
- Adversarial equivalence: rebuilding the same bounded ephemeral index from the
  same snapshot produces the same index digest and access receipt.
- Adversarial construction: callers cannot create an ephemeral index receipt or
  extend an ephemeral index lifecycle without presenting the admitted access
  plan and lifecycle scope proof that authorized the allocation.
- Adversarial lifecycle: provisioning cannot outlive the `WORTHQueryEphemeralGraphIndexScope`
  named in the admitted plan, and cleanup counters participate in the access
  receipt digest.

**Engineering decisions**
- Ephemeral indexes are read-scope or family-execution-scope resources, not
  authoritative state and not hidden caches.
- The receipt records actual allocated bytes, touched nodes, touched edges, and
  lifecycle scope.

**Open questions**
- None.

### Phase 9: Paged Streaming Frontier Execution

Add an admitted paged streaming posture for reads that are too broad for inline
materialization but can be executed safely as bounded frontier slices with
opaque cursors and checkpointed progress.

**Relevant subsystems**
- `crates/worth-query/src/runtime/workspace_queries.rs`
- `crates/worth-query/src/runtime/surface/`
- `crates/worth-query/src/runtime/state_snapshot.rs`

**Relevant APIs**
- new: `WORTHQueryGraphReadStreamingPlan`
- new: `WORTHQueryGraphReadFrontierCursor`
- new: `WORTHQueryGraphReadStreamingReceipt`
- `WORTHQueryGraphReadAccessAdmissionPosture::AdmittedPagedStreaming`

**Warnings**
- Streaming must not silently change result semantics. Page boundaries are
  delivery mechanics, not query meaning.
- Cursor identity must be basis-bound and opaque; callers cannot WORTH frontier
  continuation state.

**Test requirements**
- Adversarial convergence: executing all pages of a streaming plan produces the
  same canonical result set as an admitted full execution on a small reference
  graph.
- Adversarial memory: max resident frontier and visited buffers stay within the
  page budget across a dense graph scenario.
- Adversarial cursor: replaying, skipping, or forging a cursor fails with typed
  basis/continuation denial.
- Adversarial posture matrix: streaming rows appear in the access posture
  matrix with page budget, cursor identity, max resident frontier, and replay
  evidence; streaming cannot be a hidden fallback from inline execution.

**Engineering decisions**
- Streaming is a distinct execution posture with receipts; it is not a fallback
  hidden behind ordinary inline reads.
- Page receipts aggregate into one access-plan digest for replay and inspection.

**Open questions**
- None.

### Phase 10: Persistent Index Requirement Declaration

Ship declared persistent-index requirements for read families whose safe
execution depends on prebuilt adjacency, predicate, ordering, or relation
support. Before `worth-store` owns durable index persistence, runtime-backed
persistent requirements must remain support-row honest.

**Relevant subsystems**
- `crates/worth-query/src/runtime/builder.rs`
- `crates/worth-query/src/runtime/support/`
- `crates/worth-query/docs/authoring/read-composition.md`

**Relevant APIs**
- new: `WORTHQueryPersistentGraphIndexRequirement`
- new: `WORTHQueryGraphReadFamilyIndexContract`
- `WORTHQueryReadFamily`

**Warnings**
- Persistent-index requirement is not a promise that durable storage exists
  before store-backed milestones close.
- Do not let callers attach arbitrary strings as index names; index identity is
  derived from typed requirement rows.

**Test requirements**
- Adversarial rejection: a read family requiring a persistent index cannot
  execute inline when the inventory lacks the matching admitted row.
- Adversarial equivalence: identical read-family declarations derive identical
  persistent index requirement digests.
- Adversarial support honesty: store-backed persistent index rows return
  `RequiresStoreBackedPersistentIndex` with owner, basis, and missing-capability
  evidence until the owning store milestone admits them.
- Adversarial requirement identity: persistent requirements are derived from
  typed requirement rows and read-family identity, not caller-provided index
  names or display labels.

**Engineering decisions**
- Runtime-backed persistent requirements can resolve to existing runtime indexes
  or produce `RequiresPersistentIndex`.
- Durable persistence of indexes remains Milestone `10`/`11` scope unless
  `worth-store` supplies the admitted row.

**Open questions**
- None.

### Phase 11: Async Materialized Graph Read Jobs

Add an explicit async materialization posture for legitimate large graph reads.
The inline read path must be able to return `RequiresAsyncMaterialization`, and
the async path must run under checkpoints, cancellation, memory caps, and
receipt-backed progress.

**Relevant subsystems**
- `crates/worth-query/src/application/declaration_entry_orchestration/`
- `crates/worth-query/src/runtime/runtime_read_intents.rs`
- `crates/worth-query/src/runtime/surface/`

**Relevant APIs**
- new: `WORTHQueryGraphReadMaterializationRequest`
- new: `WORTHQueryGraphReadMaterializationJob`
- new: `WORTHQueryGraphReadMaterializationProgress`
- new: `WORTHQueryGraphReadMaterializationReceipt`

**Warnings**
- Async materialization is not an escape hatch to bypass budgets; it has its own
  stricter lifecycle and cancellation contract.
- Materialized output is derived state and must remain rebuildable from
  authoritative truth plus the admitted plan.

**Test requirements**
- Adversarial cancellation: cancelling a large graph materialization releases
  allocated frontier/index resources and preserves a typed cancellation receipt.
- Adversarial replay: re-running a completed materialization from the same
  snapshot and access plan produces the same materialization digest.
- Adversarial denial: a large read cannot smuggle into async execution without
  an admitted materialization request and budget envelope.
- Adversarial recovery: indeterminate materialization outcomes expose recovery
  handles/progress evidence rather than flattening into executor failure or
  ordinary denied read results.

**Engineering decisions**
- Async jobs are declaration/admission-owned Query artifacts, not host task
  handles.
- Job progress counters include touched edges, frontier pages, allocated bytes,
  emitted rows, and checkpoint count.

**Open questions**
- None.

### Phase 12: Ordinary Read Surface Integration

Integrate access planning into `compose_read`, reusable read families,
read-family intent execution, basis-context execution, and helper fronts. The
executor must consume an admitted access plan and must not rediscover strategy
from raw read graph shape.

**Relevant subsystems**
- `crates/worth-query/src/runtime/workspace_queries.rs`
- `crates/worth-query/src/runtime/runtime_read_intents.rs`
- `crates/worth-query/src/runtime/surface/read_composition.rs`

**Relevant APIs**
- `compose_read`
- `define_read_family`
- `execute_read_family`
- `execute_read_family_in_basis_context`
- `read_family_intent`
- `WORTHQueryGraphReadAccessAdmission`

**Warnings**
- Do not integrate only the shortest happy path. Reusable families and intent
  fronts are covered surfaces.
- The executor must not branch over access strategy beyond consuming the lowered
  plan variant.

**Test requirements**
- Adversarial parity: `compose_read`, `execute_read_family`, and
  `read_family_intent(...).execute()` produce equivalent access-plan evidence
  for the same declaration and basis.
- Adversarial rejection: unregistered, missing-capability, or over-budget access
  shapes stop at admission and never increment edge-scan execution counters.
- Adversarial no-N+1: covered helper fronts prove exact-zero per-result
  neighbor lookup loops through counters and bypass audit.
- Adversarial helper honesty: friendly helper fronts may plan-and-execute in
  one call only if the receipt exposes the same admitted access-plan artifact
  available through explicit planning.

**Engineering decisions**
- `WORTHQueryReadReceipt` gains an access-plan summary and complexity counters.
- Denied read surfaces still publish an access-admission envelope.

**Open questions**
- None.

### Phase 13: Live Read And Subscription-Adjacent Access Planning

Carry graph read access planning into live-promoted reads and subscription-
adjacent read declarations where the same canonical read meaning is maintained
over time. The runtime must distinguish one-shot access cost from maintenance
cost.

**Relevant subsystems**
- `crates/worth-query/src/runtime/tests/live_state/`
- `crates/worth-query/src/runtime/surface/`
- `crates/worth-query/src/subscription_*` surfaces

**Relevant APIs**
- live read intent surfaces
- subscription family selection diagnostics
- new: `WORTHQueryLiveGraphReadAccessPlan`

**Warnings**
- Do not claim live maintenance parity from one-shot planning alone.
- Live access indexes must not couple mutation cost to every derived view unless
  admitted by support rows.

**Test requirements**
- Adversarial parity: one-shot and live-promoted reads with the same declaration
  share compatible access-shape and required-index digests.
- Adversarial maintenance: mutation updates touch only semantically affected
  access structures, with exact counters proving bounded breadth.
- Adversarial denial: a read that is safe one-shot but unsafe to maintain live
  returns a live-specific support denial rather than silently degrading.
- Adversarial cost separation: live maintenance counters and one-shot execution
  counters are separate receipt surfaces and cannot be collapsed into one
  generic graph-read cost claim.

**Engineering decisions**
- Live planning records maintenance breadth separately from initial execution
  breadth.
- Store-backed durable live access maintenance returns
  `RequiresStoreBackedLiveAccessMaintenance` with owner, support-row evidence,
  and receipt posture unless admitted by store support.

**Open questions**
- None.

### Phase 14: Preview, Branch, Policy, Tenant, And Relationship-Proof Parity

Ensure preview sessions, branch-scoped reads, policy/tenant narrowed reads, and
relationship-proof reads use the same access-planning pipeline without
promoting lower-authority labels, snapshots, or proof descriptors into raw
planner strings.

**Relevant subsystems**
- `crates/worth-query/src/runtime/preview/`
- `crates/worth-query/src/runtime/branch.rs`
- `crates/worth-query/src/policy_narrowing/`
- `crates/worth-query/src/relationship_proof/`

**Relevant APIs**
- preview read surfaces
- branch read surfaces
- `admit_policy_tenant_context`
- `admit_relationship_proofs`
- `WORTHQueryGraphReadAccessShape`

**Warnings**
- Policy and relationship proof narrowing affect both access eligibility and
  cost; they cannot be applied after edge expansion.
- Branch and preview identity must remain typed basis artifacts, not strings
  embedded in access-plan digests.

**Test requirements**
- Adversarial parity: current-head and preview/branch reads with equivalent
  admitted basis derive compatible access requirements and distinct basis-bound
  digests.
- Adversarial leakage: policy-denied or relationship-proof-denied reads do not
  build adjacency, frontier, visited, or result buffers before denial.
- Adversarial tenant: tenant narrowing changes access shape and cost estimate
  before execution, not after result filtering.
- Adversarial authority preservation: branch, preview, policy, tenant, and
  relationship-proof identity enter access planning as admitted artifacts, not
  raw labels embedded in digests or planner strings.

**Engineering decisions**
- Access planning consumes admitted basis/narrowing artifacts, not raw caller
  context.
- Denial receipts distinguish access budget failure from policy/tenant/
  relationship-proof denial.

**Open questions**
- None.

### Phase 15: Consumer Bypass Audit For Graph Reads

Extend consumer-kit audit machinery to detect caller-owned graph read folklore:
manual loops over relation rows, per-node neighbor lookups, ad hoc adjacency
maps, surface-local graph caches, manual broad boolean scans, and hidden
fallbacks that bypass Query access planning.

**Relevant subsystems**
- `crates/worth-query/src/consumer_kit/`
- `crates/worth-topo/src/`
- `crates/worth-kernel/src/`

**Relevant APIs**
- hard-prohibition registry and boundary audit surfaces
- new: graph-read access bypass registry rows
- new: reference-consumer residue reports

**Warnings**
- False positives against docs/comments/literals are not acceptable; the audit
  must be precise enough for normal development.
- Do not treat existing helper loops as harmless because they are test support
  if they claim production proof.

**Test requirements**
- Adversarial detection: seeded N+1 relation loops, ad hoc adjacency maps, and
  manual frontier scans are detected in covered source sets.
- Adversarial false-positive: comments, docs, string literals, and unrelated
  collection iteration do not trigger graph-read bypass findings.
- Adversarial adoption: reference consumers delete or explicitly classify all
  covered graph-read folklore.
- Adversarial residue cap: every graph-read residue row carries owner,
  introduced-in phase, blocker, must-not-exceed count, and removal trigger; a
  growing residue class fails certification.

**Engineering decisions**
- Bypass rows are keyed by authority violation, not by fragile text pattern
  alone.
- Reference consumers may keep explicit residue only with typed owner, reason,
  and follow-on closure path.

**Open questions**
- None.

### Phase 16: Reference Adoption In worth-topo Graph Reads

Migrate worth-topo graph read surfaces that currently rely on local topology
walks, runtime-boundary read lowering, or test harness convenience into the
Query access-planning lane where covered. The goal is not to delete domain
meaning; it is to delete access-path folklore.

**Relevant subsystems**
- `crates/worth-topo/src/projection/runtime_boundary/`
- `crates/worth-topo/src/projection/read_views/`
- `crates/worth-topo/src/certification/`

**Relevant APIs**
- topology read handles
- Query read composition helpers
- `WORTHQueryGraphReadAccessAdmission`

**Warnings**
- Topology remains the domain authority for topology meaning; Query owns access
  planning and receipts.
- Do not migrate by wrapping old loops behind new names. The access receipt must
  prove the planned path ran.

**Test requirements**
- Adversarial parity: migrated topology reads produce the same domain result as
  the old certified reference scenarios on small graphs.
- Adversarial no-N+1: hostile dense topology scenarios prove exact-zero
  caller-owned per-edge/per-node lookup loops on covered paths.
- Adversarial broadness: broad boolean/topology search cases route to streaming,
  persistent-index-required, async, or denial posture instead of inline RAM
  expansion.

**Engineering decisions**
- Adoption starts with the read surfaces that AI_README currently teaches as
  graph/index-shaped.
- Existing topo tests may retain ergonomic helpers only if helpers cross into
  typed Query access plans before ledger/read execution.

**Open questions**
- None.

### Phase 17: Reference Adoption In worth-kernel Construction Reads

Migrate worth-kernel construction and phase-chain read/check surfaces that
currently reconstruct topology or access paths locally into declared Query read
access plans where covered.

**Relevant subsystems**
- `crates/worth-kernel/src/construction/`
- `crates/worth-kernel/src/construction/phase_chain/`
- `crates/worth-kernel/src/query_adoption/`

**Relevant APIs**
- construction read/check surfaces
- Query read access admission APIs
- consumer-kit bypass audit APIs

**Warnings**
- Construction legality remains obligation-authority work from Milestone `9.9`;
  this phase targets graph read access path planning.
- Do not use kernel construction as a hidden broad scan escape hatch.

**Test requirements**
- Adversarial parity: covered construction reads/checks produce the same
  certified construction decisions when lowered through Query access plans.
- Adversarial rejection: over-budget construction graph scans fail before local
  graph materialization or return an async/materialized posture.
- Adversarial residue: covered kernel files have exact-zero unclassified manual
  graph-read access folklore.

**Engineering decisions**
- Kernel consumes Query access receipts; it does not derive index adequacy from
  local graph shape.
- Any offline-only residue must be typed and excluded from ordinary runtime
  support posture.

**Open questions**
- None.

### Phase 18: Public Docs, AI_README, And DX Target

Update public docs and AI orientation so agents learn the exact contract:
Query derives access plans from declarations, admits only typed bounded index
postures, and returns typed required-capability, materialization, or denial
postures instead of hiding N+1 or RAM-expansive work. The docs must teach that
the high-level declaration is the authoring surface and the access plan is the
accountability surface.

**Relevant subsystems**
- `crates/worth-query/docs/AI_README.md`
- `crates/worth-query/docs/authoring/read-composition.md`
- `crates/worth-query/docs/foundations/support-matrix-and-admission.md`
- new: `crates/worth-query/docs/authoring/graph-read-access-planning.md`

**Relevant APIs**
- public read composition facade
- explicit graph-read access planning facade
- access-plan support rows
- graph read receipt access-plan summaries

**Warnings**
- Do not document the aspirational index story as if every graph shape is
  admitted. Support rows must name what is real.
- Do not teach "automatic indexing" without teaching the inspectable access
  plan, budget denial, and async/persistent/streaming postures.
- Do not teach examples that depend on `unwrap()` or raw strings flowing past
  the declaration boundary. Examples must either use typed/generated schema
  references or show the admission step that converts declaration names into
  proof-carrying references.

**Test requirements**
- Adversarial agreement: AI_README, feature docs, support rows, and public
  certification name the same access postures, required-capability cases, and
  typed denials.
- Adversarial DX: the documented happy path shows a graph read declaration that
  produces an inspectable access plan and access receipt without caller-owned
  loops.
- Adversarial denial docs: broad boolean examples show typed budget denial and
  legitimate escape hatches rather than "increase limit and retry."
- Adversarial no-magic docs: docs list the exact access requirement rows for a
  representative complex graph read so readers can see which access structures
  the runtime derived before execution.

**Engineering decisions**
- AI_README gets a distinct category for graph read access planning, separate
  from graph touch obligation authority.
- Docs include an illustrative canonical DX target. The exact function names may
  follow the final read-composition facade, but the proof-bearing shape is
  locked:

```rust
let family = workspace.define_read_family("tenant-face-neighborhood", |read| {
    read.anchored_frontier_collection(
        "HalfEdge",
        topology_schema,
        topology_schema.relations([
            "HalfEdgeNext",
            "HalfEdgeTwin",
            "HalfEdgeFace",
        ])?,
        GraphTraversalDepth::bounded(4)?,
        |query| {
            query
                .where_equal(topology_schema.predicate_field("tenant", "tenant_id")?, current_tenant)
                .project(topology_schema.projection_field("identity", "half_edge_id")?)
                .project(topology_schema.projection_field("topology", "face_id")?)
                .project(topology_schema.projection_field("topology", "next_id")?)
        },
        |shape| {
            shape
                .field("half_edge_id")
                .field("face_id")
                .field("next_id")
        },
    )
})?;

let access_plan = workspace.plan_graph_read_access(&family)?;

let indexes = access_plan.required_indexes();
assert!(indexes.contains_directional_adjacency(topology_schema.relation("HalfEdgeNext")?, Outgoing));
assert!(indexes.contains_predicate(topology_schema.predicate_field("tenant", "tenant_id")?));

let result = workspace.execute_read_family_with_access_plan(access_plan)?;

let receipt = result.receipt().graph_access_plan();
assert!(receipt.proves_no_caller_owned_n_plus_one());
assert!(receipt.max_resident_bytes() <= receipt.admitted_budget().max_resident_bytes());
```

**Open questions**
- None.

### Phase 19: Architectural Certification Closeout

Close the milestone with hostile certification across access shape, budgeting,
index requirements, admission outcomes, execution postures, no-N+1 audits,
memory counters, reference adoption, proof-type construction boundaries, and
docs/support agreement.

**Relevant subsystems**
- `crates/worth-query/tests/`
- `plans/worth-query/test-requirements.md`
- `crates/worth-topo`
- `crates/worth-kernel`

**Relevant APIs**
- graph access plan APIs from this milestone
- support matrix rows
- consumer-kit bypass audit APIs

**Warnings**
- A passing small graph suite does not prove this milestone. Dense, broad, and
  high-branching hostile workloads are mandatory.
- Do not close while any covered surface can execute graph reads without an
  access-plan receipt.

**Test requirements**
- Adversarial matrix: every access posture is certified across representative
  local, anchored, frontier, broad boolean, policy/tenant, relationship-proof,
  reusable family, preview/branch, and live-promoted reads.
- Adversarial memory: exact counters prove dense broad reads deny, stream, or
  materialize under budget without exceeding declared resident-byte limits.
- Adversarial no-N+1: bypass audit and runtime counters prove exact-zero
  covered caller-owned N+1 graph read paths.
- Adversarial replay: access-plan receipts replay to the same digests and
  counter envelopes under identical runtime truth.
- Adversarial proof typing: compile-fail or facade-boundary suites prove every
  proof-bearing artifact in the Law 41 transition table cannot be directly
  constructed, phase-skipped, upgraded, mutated, or passed to the wrong next
  phase by downstream code.

**Engineering decisions**
- Certification matrix names runtime-backed support separately from store-owned
  durability capability requirements.
- Closure requires reference-consumer adoption, not just Query-local APIs.

**Open questions**
- None.

## Must Ship

- an explicit graph-read access planning facade that lowers a read family into
  an inspectable access plan before execution
- proof-bearing schema/query reference admission for graph-read relations,
  projection fields, predicate fields, ordering fields, traversal operators,
  and result-shape fields
- sealed constructors, private fields, read-only accessors, and proof-consuming
  transition functions for every artifact in the Law 41 transition table
- sealed graph read access shape vocabulary derived from canonical read graphs
- boolean predicate normalization and selectivity input artifacts
- graph-read access requirement set derivation with typed access/lifecycle rows
- access inventory matching as a distinct proof artifact from requirement
  derivation
- intrinsic and supported cost estimates, memory/breadth budgets, and
  complexity contracts
- typed access admission outcomes:
  `AdmittedInlineIndexed`, `AdmittedBoundedEphemeralIndex`,
  `AdmittedPagedStreaming`,
  `RequiresPersistentIndex`, `RequiresAsyncMaterialization`,
  `RequiresStoreBackedPersistentIndex`,
  `RequiresAccessCapabilityRegistration`,
  `RequiresStoreBackedLiveAccessMaintenance`, and `DeniedBudgetExceeded`
- exhaustive access-case registry covering admitted, required-capability,
  materialized, streaming, store-owned, and typed-denial graph read cases
- existing access inventory and support rows
- `WORTHQueryAdmittedGraphReadAccessPlan` as the only executable input for
  graph-read execution postures that require access planning
- bounded ephemeral index provisioning with lifecycle receipts
- paged streaming frontier execution
- persistent index requirement declarations for read families
- async graph read materialization jobs
- ordinary read, reusable family, intent, live, preview, branch, policy/tenant,
  and relationship-proof integration
- consumer bypass audit for graph read N+1 and manual access folklore
- worth-topo and worth-kernel reference adoption
- public docs and AI_README category for graph read access planning
- architectural certification matrix with no-N+1 and memory-budget proof
- compile-fail or facade-boundary certification that law-41 proof artifacts
  cannot be WORTHd, skipped, or upgraded by downstream code

## Must Preserve

- `worth-relational` remains the authority for truth semantics and relational
  state mechanics.
- Query owns typed read declaration, lowering, access planning, admission,
  receipts, and support posture.
- Indexes, access plans, materializations, and receipts are derived from
  authoritative truth and must be rebuildable.
- Store-backed durable indexes and restart-stable access artifacts are explicit
  Milestone `10`/`11` capability requirements until the owning milestones admit
  them.
- Graph touch obligation authority from Milestone `9.9` remains separate from
  graph read access planning; the former decides which obligations fire, while
  this milestone decides how reads access graph truth safely.

## Acceptance Evidence

- a representative complex graph read can be declared concisely, planned before
  execution, inspected for exact access requirement rows and cost posture, and then
  executed through the same admitted plan
- covered graph reads execute only through proof-bearing access plans
- hidden N+1 graph traversal is mechanically impossible on covered public lanes
- broad boolean and dense frontier reads deny, stream, require persistent index,
  or require async materialization before expensive work begins
- access-plan receipts expose access requirements, selected posture, estimated and
  actual counters, resident-byte budgets, touched-edge counts, frontier counts,
  and fallback/denial posture
- memory and breadth counters have exact hostile tests rather than elapsed-time
  claims
- docs, support rows, AI_README, and certification agree about admitted,
  required-capability, materialization, store-owned, and typed-denial graph read
  access postures
- docs and tests prove that declaration-bound strings do not survive as
  executable authority; execution receives admitted references and admitted
  access plans
- compile-fail or facade-boundary tests prove downstream code cannot construct
  admitted schema references, access shapes, access requirement rows, inventory
  matches, admissions, admitted access plans, ephemeral index receipts, or
  execution receipts without passing through the proving phase
- every edge case named in docs, tests, support rows, or the access-case
  registry maps to a concrete admission result and receipt field
- worth-topo and worth-kernel covered graph read surfaces delete or classify
  local access-path folklore

## Sequencing Notes

- This milestone belongs after Milestone `9.9` because it consumes graph read
  shape and obligation-envelope vocabulary without overloading obligation
  dispatch with access-path planning.
- This milestone belongs before Milestone `10` because store-backed execution
  and pushdown should inherit a complete runtime-backed access-plan contract,
  not discover N+1 prevention, memory budgets, and index posture later.
- Milestone `10` may later add store-backed persistent index realization and
  pushdown parity, but it should not redefine the access-shape, budget, denial,
  and receipt model established here.
- Milestone `11` may later make saved access artifacts durable, but restart-
  stable persistence is not required for runtime-backed closure here.
