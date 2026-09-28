# Milestone 4 Engineering Spec: Collection Semantics, Ordering, Pagination, And Bounded Traversal

> **Status:** Completed. Durable cursor resume, store-backed collection parity, and broader family coverage remain open for later milestones. Closeout: [milestone-4-closeout.md](./milestone-4-closeout.md).
>
> **Roadmap parent:** [worth_query_roadmap.md](./WORTH_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./WORTH_query_vision.md)
>
> **Prior milestone:** [milestone-3.md](./milestone-3.md)
>
> **Prior closeout:** [milestone-3-closeout.md](./milestone-3-closeout.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** make collection reads, page advancement, bounded traversal, aggregate shaping, and CDC-shaped result families planner-owned proof surfaces so large-surface reads stay basis-honest, breadth-explicit, and parity-safe under repeated execution
>
> **Companion docs:**
> - [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)
> - [arch_laws.md](../../docs/coding-guidelines/arch_laws.md)
> - [perf_laws.md](../../docs/coding-guidelines/perf_laws.md)
> - [domain_laws.md](../../docs/coding-guidelines/domain_structure_laws.md)
> - [worth_query_vision.md](./WORTH_query_vision.md)
> - [worth_query_roadmap.md](./WORTH_query_roadmap.md)
> - [test-requirements.md](./test-requirements.md)
> - [milestone-2.md](./milestone-2.md)
> - [milestone-2-closeout.md](./milestone-2-closeout.md)
> - [milestone-3.md](./milestone-3.md)
> - [milestone-3-closeout.md](./milestone-3-closeout.md)

## Goal

Make collection-scale query behavior a first-class proof-bearing surface so
ordered pages, bounded traversal/materialization, aggregate/rollup families,
query-time derived fields, and CDC-shaped result families preserve one
canonical query meaning for one declared basis rather than fragmenting into
host-local loops, offset/limit instability, or post-processed delivery hacks.

## Why This Milestone Exists

Milestone 3 made one-shot planning and snapshot-bound execution honest for a
single declared query meaning. Milestone 4 is where `worth-query` either
becomes a real product surface or collapses into "detail reads plus host-side
collection glue."

Collection semantics are the first place where scale and product ergonomics
push directly against architectural honesty:

- ordered collections tempt callers to smuggle offset/limit in as fake stable
  pagination
- eager loading and traversal breadth tempt the runtime to over-read and rely
  on post-filtering
- aggregates, rollups, and derived fields tempt consumers to recompute outside
  the planner
- CDC-shaped output tempts integration paths to bypass query-shaped results and
  expose raw runtime deltas instead
- large result surfaces tempt APIs to hide cardinality, breadth, and truncation
  cost behind "simple list" convenience calls

Milestone 4 exists to freeze these semantics before live promotion, historical
basis variation, and view-shape composition build on top of them. If
collection-scale execution is not planner-owned now, later milestones inherit a
fractured substrate and certification becomes performative.

## Governing Summaries

- `MENTALITY.md`: the hard problem is not returning vectors of rows. It is
  surviving large-surface execution without breadth drift, unstable page
  advancement, or host-side semantic repair. The milestone must therefore solve
  cursor and breadth honesty first.
- `arch_laws.md`: Laws 7, 14, 17, 24, 26, 27, 30, 32, 33, 40, and 41 dominate
  this milestone. Collection planning must lower once; executor/runtime paths
  must consume proof-bearing collection plans rather than rediscover breadth,
  ordering, or grouping semantics.
- `perf_laws.md`: collection breadth, traversal breadth, aggregate input width,
  and delivery width must be explicit counters and bounded contracts. No API
  may look scalar while hiding broad scans or host recomputation.
- `domain_laws.md`: ordering, pagination, traversal/materialization,
  aggregation/rollups, derived-field shaping, and CDC rendering are separate
  subdomains and must not ship as one "collection helper" blob.
- `worth_query_vision.md`: pagination, bounded results, aggregation,
  rollups, CDC-shaped output, and eager relation loading are first-class query
  capabilities. They must remain query-shaped and aspect-aware rather than
  degrading into alternate APIs.
- `worth_query_roadmap.md`: Milestone 4 is the runtime-backed collection
  milestone. It must close before live promotion because live maintenance must
  inherit stable collection semantics rather than invent them.
- `test-requirements.md`: the `Collection, Cursor, Rollup, And CDC Shape Parity
  Test` is the closeout proof. Cursor progression, breadth counters, derived
  result families, and CDC-shaped rendering must be machine-checkable and
  parity-safe for one basis.
- `milestone-2.md`: legality still belongs to validation, not collection
  execution. Aggregation families, derived fields, traversal declarations, and
  ordering semantics must execute only through validated proof surfaces.
- `milestone-2-closeout.md`: validation rejection already has a named
  certification architecture. Milestone 4 must extend that proof chain rather
  than accepting runtime-local collection repairs.
- `milestone-3.md`: planned/basis/execution artifacts are already frozen for
  runtime-backed one-shot execution. Milestone 4 must extend those artifacts
  with collection proof, not replace them with a parallel collection runtime.
- `milestone-3-closeout.md`: Milestone 4 may assume a stable planned artifact
  boundary, explicit basis identity, and shared certification harness, but it
  may not assume live, historical, or durable store semantics.

## Adversarial Constraint

Milestone 4 must survive the following hostile condition:

> The same validated query meaning is executed repeatedly as an ordered
> collection, a cursor-advancing page stream, a bounded traversal/materialized
> collection, an aggregate/rollup family, a query-time derived-field result,
> and a CDC-shaped delivery view for the same snapshot basis. Every admitted
> surface must preserve the same canonical scope, ordering basis, truncation
> semantics, and result meaning without widening reads, destabilizing cursors,
> or offloading semantics into host-side post-processing.

Concretely, the design must remain correct when all of the following are true:

- the same collection query is re-executed with different page sizes, cursor
  positions, or delivery/result-family selections against the same basis
- ordering involves non-trivial declared keys, tie-breakers, and relation- or
  aggregate-derived sort inputs that a naive executor would only partially
  encode
- traversal/materialization breadth would naturally fan out beyond the caller's
  intended scope unless depth, edge-class, and shape limits are planner-owned
- aggregate and rollup inputs are much wider than the projected output, making
  host-local recomputation or hidden fallback tempting
- query-time derived fields and CDC-shaped outputs describe the same truth
  through different result families and could drift if they are not lowered
  from one common plan
- active mutation continues elsewhere in the runtime while the query walks
  pages or materializes eager relation surfaces

If any supported path:

- treats offset/limit as equivalent to stable cursor advancement
- widens traversal or eager materialization beyond declared planner-owned
  bounds
- recomputes aggregates, rollups, or derived fields outside the planned query
  substrate
- changes result meaning depending on whether the caller requested ordinary
  shaped results or CDC-shaped delivery output
- hides collection breadth, page truncation, aggregate input width, or
  traversal depth from the execution/reporting surface

then Milestone 4 has failed.

## Product Decision Lock

The following decisions are explicit and not open questions for this
milestone:

- collection semantics are planner-owned query semantics, not host-local loops
  over repeated detail reads
- stable page advancement is cursor-based and basis-bound; offset/limit may be
  admitted only as an explicitly unstable convenience surface if it does not
  claim cursor semantics
- ordering semantics, tie-break semantics, bounded traversal/materialization,
  aggregation, rollups, derived fields, and CDC-shaped result families are
  lowered from one common collection planning path
- CDC-shaped output remains a result family over canonical query meaning, not a
  direct exposure of runtime CDC internals
- runtime-backed collection execution is the canonical completion target for
  this milestone; restart-stable cursor durability and store-backed parity stay
  deferred to later milestones
- unsupported collection families, unsupported ordering/aggregation shapes, and
  unsupported traversal breadth requests fail typed and early

Normative consequence:

- any implementation path that computes cursor advancement from ambient list
  offsets instead of planner-owned ordering identity is out of spec
- any implementation path that executes broad relation loads and trims them in
  host code while claiming bounded traversal is out of spec
- any implementation path that treats derived fields or rollups as view-layer
  recomputation instead of planned derived result semantics is out of spec
- any implementation path that returns raw CDC deltas while claiming canonical
  CDC-shaped query output is out of spec

## Compile-Time Enforcement Policy

Milestone 4 must classify which collection guarantees become
unrepresentable, uncompilable, or construction-time rejection.

`Unrepresentable` in public types:

- publicly constructible collection plan artifacts missing ordering basis,
  traversal/materialization bounds, or result-family classification
- publicly constructible cursor or page tokens that do not carry a canonical
  page-advance basis
- publicly constructible aggregate/rollup/derived result artifacts that do not
  carry basis identity and width accounting
- publicly constructible "generic sort descriptors", "generic page requests",
  or "generic aggregate specs" that collapse semantically distinct collection
  families into stringly bags

`Uncompilable` through visibility and compile-fail enforcement:

- external construction of proof-bearing collection artifacts, page-advance
  artifacts, or CDC-shaped result artifacts without crate-owned lowering
- public execution entry points that accept raw host pagination state, raw
  aggregate descriptions, or host-built CDC outputs as though they were planned
  query semantics
- external access to planner internals that would let callers synthesize stable
  cursor proofs or traversal-bound proofs without validation/planning
- public helper APIs that let consumers "advance" cursors, "complete"
  tie-breakers, or "finish" CDC rows by patching planner-owned meaning in
  host/runtime glue

`Construction-time rejection`:

- unsupported ordering families or missing stable tie-break contracts
- unsupported cursor advancement requests
- unsupported traversal/materialization breadth combinations
- unsupported aggregation/rollup/derived-field families
- unsupported CDC-shaped output families
- page/result-family requests incompatible with the declared collection plan

Rules:

- the strongest boundary available must be used
- proof-bearing collection plan and cursor artifacts must use sealed
  constructors and private fields
- planner-owned family enums must be closed/sealed rather than open-ended
  consumer extension points in this milestone
- compile-fail coverage is required for public collection-proof boundaries
- runtime rejection is allowed only for facts genuinely unavailable until basis
  resolution, page-advance state, or admitted backend capability matching

## Scope

### In Scope

- collection planning as a first-class extension of the Milestone 3 planned
  execution substrate
- typed ordering semantics and stable tie-break contracts
- opaque cursor-based page advancement for one declared basis
- bounded result-set declarations
- bounded traversal/materialization and eager relation loading contracts
- aggregate query families with explicit grouping semantics
- relational rollup families over declared relation edges
- query-time derived-field declarations that lower into planned semantics
- CDC-shaped result families for integration-facing consumers
- collection execution/report envelopes carrying exact breadth and truncation
  counters
- milestone-native certification proving collection, cursor, rollup, and
  CDC-shaped parity

### Explicitly Out Of Scope

- live subscription promotion, incremental maintenance, or `worth-signal`
  invalidation work
- historical, diff, lineage, or correspondence semantics
- durable cursor persistence, restart-stable page resume, or saved-query
  artifact durability
- policy masking and tenant-aware narrowing beyond preserving the right
  planner-owned seams for Milestone 9
- store-backed parity claims that cannot yet honestly consume the same
  collection plan semantics
- server/network subscription delivery mechanics

## Collection Architecture

### One Planner-Owned Collection Transition

Milestone 4 extends the Milestone 3 proof chain. It does not introduce a
second "collection runtime."

The authoritative flow becomes:

`ValidatedQueryBundle`
-> `PlanningRequestContext`
-> `CollectionPlanningContext`
-> `CollectionPlanBundle`
-> `CollectionExecutionPreflight`
-> `CollectionResultEnvelope`

Collection semantics are therefore derived from validated and planned query
meaning; they are never re-authored by route glue, pagination helpers, or
delivery adapters.

### Authority Boundaries

`Validation` remains authoritative for:

- legality of ordering expressions, traversal declarations, aggregate/rollup
  declarations, and derived-field declarations
- schema legality of eager relation loading and result-shape declarations

`Collection planning` becomes authoritative for:

- ordering basis and stable tie-break basis
- admitted cursor advancement semantics
- page/window/truncation semantics
- traversal/materialization breadth and eager load boundaries
- aggregate/rollup execution shape
- query-time derived-field lowering
- CDC-shaped result-family selection and parity relation to ordinary results
- stage partitioning between read-time work and post-read shaping work

`Basis resolution / preflight` remain authoritative for:

- basis identity
- compatibility between collection plan requirements and the admitted runtime
  basis

`Execution` becomes authoritative for:

- consuming the lowered collection plan against the admitted basis
- producing typed ordinary or CDC-shaped collection result envelopes
- exact breadth, truncation, and width counters

`Host / delivery glue` may own:

- presentation of already-planned page tokens
- transport of already-lowered CDC-shaped results
- consumer-local formatting that does not mutate query meaning

`Host / delivery glue` may not own:

- stable cursor derivation
- offset-to-cursor repair
- aggregate/rollup recomputation
- derived-field recomputation
- traversal widening
- CDC semantics synthesis
- ordering tie-break completion
- result-family-specific semantic narrowing

### Planner Split Requirements

Milestone 4 should not ship as one large `collection_planner.rs`.

At minimum, the planner boundary should stay decomposable into distinct
subdomains such as:

- `ordering_planner`
- `pagination_planner`
- `traversal_planner`
- `aggregate_planner`
- `rollup_planner`
- `derived_field_planner`
- `cdc_family_planner`

These may share a façade, but they should remain independently testable and
proof-bearing because they fail for different reasons and will evolve under
different milestone pressure later.

## Phases

### Phase 1: Freeze Collection Authority Surfaces

Define the new proof-bearing collection artifact family and the authority split
that later phases must obey.

This phase must introduce:

- `CollectionPlanningContext`
- `CollectionPlanBundle`
- `CollectionOrderingBasis`
- `OrderingKeyPath`
- `OrderingDirection`
- `OrderingTieBreakContract`
- `StableOrderingContract`
- `CollectionWindowPolicy`
- `CursorAdvanceContract`
- `OpaquePageCursor`
- `CursorBoundaryDigest`
- `TraversalBoundContract`
- `TraversalDepthLimit`
- `TraversalEdgeClass`
- `MaterializationBreadthClass`
- `AggregateShapeArtifact`
- `AggregateFunctionFamily`
- `AggregateGroupingShape`
- `AggregateInputBreadth`
- `RollupShapeArtifact`
- `RollupEdgeClass`
- `DerivedFieldPlanArtifact`
- `DerivedFieldComputationClass`
- `CollectionResultFamily`
- `PostReadShapingPlan`
- `CollectionCounters` and `CollectionPlanningReport`

Phase 1 is complete only when collection-specific authority is no longer hidden
inside generic planning/execution residue and compile-time boundaries can be
enforced around the new artifact family.

### Phase 2: Lower Ordering, Page, And Bound Semantics

Teach the planner to lower validated collection declarations into one stable
ordering/page/bound proof surface.

This phase must:

- freeze ordering keys plus stable tie-break semantics
- define opaque page-advance state that is basis-bound and ordering-bound
- lower bounded result semantics and truncation semantics into the plan
- reject unsupported or unstable ordering/page combinations before execution
- make "stable ordering" impossible to claim without a closed tie-break proof
- make cursor/token equality and non-equality digest-bearing rather than
  host-string-comparison behavior

Phase 2 is complete only when page advancement and truncation are planner-owned
semantics rather than execution-local iteration behavior.

### Phase 3: Lower Traversal, Materialization, And Aggregate Families

Teach collection planning to own breadth-bearing result-shape families.

This phase must:

- lower bounded eager materialization and traversal breadth into explicit plan
  artifacts
- lower aggregate and grouping semantics into explicit planned shape
- lower relational rollup semantics over admitted relation edges
- make aggregate-input breadth and traversal breadth explicit reportable values
- distinguish read-breadth from post-read shaping breadth explicitly
- reject any aggregate/rollup family that would require host-owned semantic
  completion to appear correct

Phase 3 is complete only when broad collection families stop looking like one
"list query" with hidden executor branching.

### Phase 4: Lower Derived Fields And CDC-Shaped Result Families

Freeze result-family parity so ordinary collection output and CDC-shaped output
share one planned semantic substrate.

This phase must:

- lower query-time derived fields into explicit planner-owned result semantics
- define CDC-shaped result families as planned output families over the same
  canonical query meaning
- name equality and inequality boundaries between ordinary and CDC-shaped
  result families
- reject unsupported delivery/result-family combinations typed and early
- define whether each derived field is read-time derivable, post-read derivable,
  or unsupported in this milestone
- define which CDC-shaped fields are semantic output and which are transport
  metadata excluded from canonical meaning

Phase 4 is complete only when derived fields and CDC-shaped output stop being
plausible host-postprocessing escape hatches.

### Phase 5: Execute Collection Plans Against Stable Bases

Wire the lowered collection plan into runtime-backed execution and exact
counter/report surfaces.

This phase must:

- execute ordered pages against stable snapshot bases
- materialize bounded traversal/aggregate/rollup families through one admitted
  runtime-backed execution path
- produce `CollectionResultEnvelope` variants for ordinary and CDC-shaped
  outputs
- expose exact counters for page width, result width, traversal depth,
  materialization breadth, aggregate input breadth, rollup breadth, derived
  field count, and CDC output width
- enforce `executor_collection_rediscovery_count == 0`
- enforce that unsupported families fail before broad reads begin whenever the
  missing fact is already knowable at planning time

Phase 5 is complete only when collection-scale execution is basis-honest and
counter-explained for admitted runtime-backed paths.

### Phase 6: Certify Collection, Cursor, Rollup, And CDC Parity

Close the milestone with named proof instead of feature demos.

This phase must:

- add the `Collection, Cursor, Rollup, And CDC Shape Parity Test`
- prove ordered-page stability for one basis
- prove bounded traversal/materialization honesty
- prove aggregate/rollup/derived-field parity for repeated execution
- prove CDC-shaped parity against ordinary canonical collection meaning
- prove required rejection rows for unsupported ordering, cursor, traversal,
  and result-family shapes

Phase 6 is complete only when the milestone closes through the shared
certification harness with deterministic machine-checkable artifacts.

## Must Ship

- proof-bearing collection planning artifacts layered on top of Milestone 3
  planning/basis/execution artifacts
- typed ordering and stable tie-break semantics
- opaque cursor-based page advancement for one declared basis
- bounded result-set and truncation semantics
- bounded traversal/materialization and eager relation loading contracts
- aggregate query families with explicit grouping semantics
- relational rollup query families over admitted relation edges
- query-time derived-field declarations lowered into planned result semantics
- CDC-shaped result families lowered from canonical query meaning
- collection execution envelopes and reports with exact breadth/truncation
  counters
- milestone-native certification rows and completeness reporting

## Must Preserve

- validation remains the only authority over legality
- planning remains the only authority over ordering, bounds, grouping, and
  result-family lowering
- execution consumes lowered plans and does not rediscover collection
  semantics
- collection execution remains snapshot-stable for one basis under active
  mutation
- ordinary results and CDC-shaped results remain query-shaped views over the
  same canonical truth rather than competing semantic paths
- derived fields, rollups, and aggregate results remain derived result
  semantics, never stored authority
- later milestones retain room to add live promotion, historical basis
  variation, view shapes, and durability without reinterpreting Milestone 4
  plans

## Acceptance Evidence

Milestone 4 is complete only when `worth-query` can prove:

- the `Collection, Cursor, Rollup, And CDC Shape Parity Test` in
  [test-requirements.md](./test-requirements.md)
  passes with canonical machine-checkable artifacts
- ordered collection queries return stable cursor-advancable pages for one
  snapshot basis
- page advancement for the same basis preserves ordering identity and
  truncation honesty
- bounded traversal and eager materialization stay within declared scope,
  depth, and relation-edge contracts
- aggregate, rollup, and query-time derived-field results remain tied to the
  declared truth basis and declared projection
- CDC-shaped output matches the same canonical query meaning as ordinary
  collection execution for the same query and basis
- collection counters and diagnostics explain why a query touched the breadth
  it touched

Required verification output must include:

- `query_digest`
- `plan_digest`
- `basis_digest`
- `result_digest`
- `delivery_digest`
- `cursor_progress_report`
- `counter_snapshot`

Required canonical rows should include at minimum:

- `ordered-collection-parity`
- `cursor-advance-repeatability`
- `bounded-traversal-parity`
- `aggregate-rollup-parity`
- `derived-field-parity`
- `cdc-shaped-result-parity`

Required rejection rows should include at minimum:

- `unsupported-ordering-family`
- `unstable-cursor-shape`
- `unsupported-traversal-bound`
- `unsupported-aggregate-family`
- `unsupported-cdc-result-family`

## Architectural Notes

### Newtype And Artifact Guidance

Milestone 4 should prefer explicit wrappers over generic maps or ad hoc tuples.
The likely proof-bearing vocabulary includes types such as:

- `CollectionOrderingBasis`
- `StableOrderingContract`
- `OrderingKeyPath`
- `OrderingDirection`
- `OrderingTieBreakContract`
- `CollectionWindowPolicy`
- `PageWindowSpec`
- `CursorAdvanceContract`
- `OpaquePageCursor`
- `CursorBoundaryDigest`
- `TraversalBoundContract`
- `TraversalDepthLimit`
- `TraversalEdgeClass`
- `MaterializationBreadthClass`
- `PostReadShapingPlan`
- `AggregateGroupingShape`
- `AggregateFunctionFamily`
- `AggregateInputBreadth`
- `RollupShapeArtifact`
- `RollupEdgeClass`
- `DerivedFieldShape`
- `DerivedFieldComputationClass`
- `CollectionResultFamily`
- `CollectionPlanDigest`
- `DeliveryDigest`
- `CursorProgressReport`

The exact names may change, but the semantic splits should not collapse:

- ordering basis is not the same thing as page window policy
- stable ordering proof is not the same thing as a list of sort keys
- traversal bounds are not the same thing as materialization breadth reports
- aggregate grouping shape is not the same thing as rollup edge class
- aggregate input breadth is not the same thing as collection result width
- derived-field computation class is not the same thing as result-family choice
- CDC-shaped family choice is not the same thing as transport formatting

### Naive Trap Avoidance

The spec should explicitly steer implementation away from these failure modes:

- "cursor as serialized offset": forbidden for any certifying stable page path
- "sorting plus implicit entity id fallback": forbidden unless the fallback is
  part of `StableOrderingContract`
- "bounded traversal by best effort": forbidden; bounds must be plan fields,
  counters, and rejection criteria
- "aggregate as ordinary collection post-pass": forbidden unless the planner
  lowered that exact shaping stage as part of `PostReadShapingPlan`
- "derived field as UI helper": forbidden for any field that affects certified
  result meaning
- "CDC output as formatter over raw runtime deltas": forbidden; CDC families
  must be query-result families first and transport shapes second

### Compiler-Level Quality Gates

Milestone 4 should explicitly require:

- compile-fail tests for private fields on `CollectionPlanBundle`,
  `OpaquePageCursor`, and `CollectionResultEnvelope`
- compile-fail tests proving consumers cannot construct "stable ordering" or
  "stable cursor" proofs directly
- sealed constructors for aggregate, rollup, derived-field, and CDC-family
  artifacts
- `pub(crate)` planner lowering entry points for family-specific planners, with
  only façade-level crate APIs exposed
- type-level separation between ordinary collection result envelopes and
  CDC-shaped collection result envelopes even if they share storage internals

### Counter Expectations

Milestone 4 should add exact counters for collection-specific breadth rather
than hiding them inside generic execution counts. At minimum the architecture
should leave room for exact or exact-equivalent counters such as:

- `collection_result_count`
- `page_width`
- `page_truncation_count`
- `cursor_advance_count`
- `traversal_depth`
- `materialized_relation_count`
- `aggregate_input_count`
- `rollup_input_count`
- `derived_field_evaluation_count`
- `cdc_output_count`
- `executor_collection_rediscovery_count == 0`

### Honest Non-Goals

Milestone 4 should not pretend to close:

- restart-stable cursor durability
- live incremental maintenance of collections
- historical or diff-aware page replay
- durable store-backed aggregation parity
- consumer-specific rendering concerns above the query result family boundary

## Sequencing Notes

Milestone 4 belongs immediately after Milestone 3 because collection semantics
are the first broad-surface place where plan/execute dishonesty, breadth drift,
and host-local recomputation would fracture the subsystem if left implicit.

It must land before Milestone 5 because live promotion cannot honestly converge
collection results unless ordering, page advancement, traversal/materialization
bounds, aggregate families, and CDC-shaped result meaning are already frozen.

It must land before Milestone 8 because view shapes, grouped timelines, and
inspector/detail surfaces depend on real collection/result-family semantics,
not placeholder list behavior.

## Unresolved Judgment

The main remaining judgment call is whether Milestone 4 should admit any
explicitly unstable offset/limit convenience surface at all.

My recommendation is:

- keep stable pagination strictly cursor-based in the proof-bearing surface
- if offset/limit exists, classify it as convenience-only, non-certifying, and
  non-equivalent to stable cursor advancement

That keeps the milestone semantically sharp and avoids poisoning later live or
historical pagination work with fake stability claims.
