# Milestone 3 Engineering Spec: Query Planning, Snapshot-Bound Execution, And Binding Parity

> **Status:** Completed. Store-backed execution parity remains open for later milestones.
>
> **Roadmap parent:** [worth_query_roadmap.md](./WORTH_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./WORTH_query_vision.md)
>
> **Prior milestone:** [milestone-2.md](./milestone-2.md)
>
> **Prior closeout:** [milestone-2-closeout.md](./milestone-2-closeout.md)
>
> **Closeout:** [milestone-3-closeout.md](./milestone-3-closeout.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** make planning and one-shot execution a proof-bearing phase so validated query meaning lowers once, binds once, executes against stable truth once, and never gets reinterpreted by host glue or executor convenience paths
>
> **Companion docs:**
> - [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)
> - [arch_laws.md](../../docs/coding-guidelines/arch_laws.md)
> - [perf_laws.md](../../docs/coding-guidelines/perf_laws.md)
> - [domain_laws.md](../../docs/coding-guidelines/domain_structure_laws.md)
> - [worth_query_vision.md](./WORTH_query_vision.md)
> - [worth_query_roadmap.md](./WORTH_query_roadmap.md)
> - [test-requirements.md](./test-requirements.md)
> - [milestone-1.md](./milestone-1.md)
> - [milestone-1-closeout.md](./milestone-1-closeout.md)
> - [milestone-2.md](./milestone-2.md)
> - [milestone-2-closeout.md](./milestone-2-closeout.md)

## Goal

Make planning and one-shot execution first-class proof phases so validated
query meaning lowers deterministically into executable plans, binds through one
query-owned descriptor vocabulary, reads canonical truth through stable
snapshots, and returns typed results plus execution evidence without host-local
semantic reinterpretation.

## Why This Milestone Exists

Milestone 1 froze what a query is. Milestone 2 froze whether that query is
legal against a schema basis. Milestone 3 freezes how a legal query becomes an
executable read without re-opening either of those solved questions.

`worth-query` only becomes a real subsystem once planning and execution are
explicit, typed, and parity-safe:

- collection semantics in Milestone 4 need a planner-owned definition of read
  breadth, narrowing, ordering basis, and fallback admission
- live promotion in Milestone 5 needs plan artifacts that can survive one-shot
  execution and later incremental maintenance without changing query meaning
- historical and diff contexts in Milestone 6 need execution to be basis-honest
  now rather than retrofitted later
- type-bound execution only works if host-facing descriptors bind onto
  planner-owned semantics instead of inventing route-local query behavior
- any future runtime-backed versus store-backed parity depends on one planned
  artifact family and one execution envelope, not one ad hoc path per consumer

Milestone 3 is therefore not "run the query." It is "make planning,
snapshot-basis selection, fallback admission, execution, and binding parity
explicit enough that later surfaces inherit one execution truth instead of
forking the subsystem."

## Governing Summaries

- `MENTALITY.md`: the hard problem is not getting rows back. It is surviving
  execution-path variation, binding-path variation, and future store/runtime
  split without semantic drift. The planner and executor must therefore be
  designed as proof boundaries first.
- `arch_laws.md`: Laws 7, 8, 12, 17, 19, 26, 30, 40, and especially 41 shape
  this milestone. Plans and execution results must be self-describing, typed,
  rollback-free derived artifacts; errors must be structured; authoritative and
  derived state must stay separate; and validated meaning must transition into
  planned meaning through explicit proof-bearing types.
- `perf_laws.md`: plan cost and execution breadth have to be made visible at
  the plan and result boundary. Narrowing, snapshot reuse, fallback admission,
  and breadth claims need exact counters rather than elapsed-time vibes.
- `domain_laws.md`: planning, snapshot basis resolution, execution, binding
  translation, result shaping, and diagnostics are distinct subdomains. This
  milestone must not ship as one "query runtime" blob.
- `worth_query_vision.md`: `worth-query` explicitly owns planning and execution
  while still not owning truth semantics. It also promises type-bound execution
  and typed result shapes, which means host binding and execution cannot become
  hidden alternate authority paths.
- `worth_query_roadmap.md`: Milestone 3 exists to lower validated query meaning
  into proof-carrying plans, stable snapshot reads, and parity-safe binding
  descriptors before collection scale, live promotion, or historical reads are
  allowed to build on top.
- `test-requirements.md`: the `Planner / Executor / Binding Parity Test` is the
  closeout proof. Equivalent planning and execution paths must emit identical
  `plan_digest`, `result_digest`, `basis_digest`, and exact counter evidence.
- `milestone-1.md`: canonical query and result-shape authority are already
  frozen. Milestone 3 must consume those identities indirectly through
  validated proof, not re-accept weaker authored artifacts or builder residue.
- `milestone-1-closeout.md`: the crate already proved deterministic query
  identity and compile-time boundary protection. Milestone 3 must preserve that
  authority separation when adding executable planning surfaces.
- `milestone-2.md`: legality belongs to validation, not to planning or
  execution. Milestone 3 must treat validated artifacts as the only admission
  surface and must not re-decide schema legality, widening, or predicate
  semantics.
- `milestone-2-closeout.md`: the current crate already has schema-owned
  validation artifacts, rejection matrices, and sealed proof boundaries.
  Milestone 3 must extend that proof chain into plans and execution results
  without downgrading to ambient runtime discovery.

## Adversarial Constraint

Milestone 3 must survive the following hostile condition:

> The same validated query meaning is executed through direct runtime-backed
> planning, independently repeated planning, admitted type-bound binding
> descriptors, and later store-backed execution where admitted. Every admitted
> path must converge to the same plan semantics, the same snapshot basis
> semantics, and the same typed result meaning, without the executor
> rediscovering legality, projection, narrowing, ordering, or scope that the
> planner should already have frozen.

Concretely, the design must remain correct when all of the following are true:

- the same validated query bundle is planned multiple times with different
  caller-local labels, diagnostics richness, or binding metadata ordering
- the runtime can offer more than one physical read path for the same legal
  query shape, but only some are admitted and parity-safe
- a host binding path supplies entity identity, collection target, or context
  slots through query-owned binding descriptors instead of direct invocation
- snapshots can be resolved from current runtime truth now and from durable
  store state later, but both must describe the basis identically where both
  are admitted
- result shaping can happen partly before and partly after truth reads, but the
  planner must own which work belongs in which phase
- callers request unsupported plan shapes or unsupported execution backends
  that a naive system would satisfy by broadening reads or silently dropping to
  a semantically different path

If any supported path:

- lets the executor rediscover validation-owned legality or planner-owned
  narrowing decisions
- changes plan identity because the host reached the same query through a
  different binding or invocation route
- executes against ambient mutable truth instead of an explicit stable basis
- silently widens to a broader read or fallback path than the plan admitted
- changes result-shape semantics depending on whether a route binding, direct
  call, or future store path was used

then Milestone 3 has failed.

## Product Decision Lock

The following decisions are explicit and not open questions for this
milestone:

- Milestone 3 introduces a distinct planned-query proof phase; validated query
  bundles from Milestone 2 are necessary but not sufficient for execution
- planning owns read-path admission, narrowing freeze, execution-stage
  partitioning, and basis requirements; execution owns only running the plan
  against an admitted basis and producing typed derived results
- one-shot execution must run against stable snapshot authority rather than
  ambient mutable runtime state
- type-bound execution is query-owned binding onto canonical query plan inputs,
  not host-owned query synthesis and not route/controller-local fallback logic
- runtime-backed execution is the canonical completion target for this
  milestone; store-backed parity is admitted only where the store path can
  honestly consume the same planned semantics
- unsupported plan or backend shapes must fail typed and early; they may not
  degrade into hidden re-planning, hidden widening, or consumer-local repair

Normative consequence:

- any implementation path that accepts canonical or authored query artifacts
  directly into execution, bypassing validation and planning, is out of spec
- any implementation path that lets route bindings or UI descriptors invent
  hidden filters, hidden basis selectors, or hidden result-shape mutations is
  out of spec
- any implementation path that resolves truth reads from "latest runtime
  state" without an explicit snapshot basis artifact is out of spec
- any implementation path that makes plan identity depend on the chosen
  backend, binding path, or diagnostics mode is out of spec

## Compile-Time Enforcement Policy

Milestone 3 must explicitly classify which planning and execution guarantees
become unrepresentable, uncompilable, or construction-time rejection.

`Unrepresentable` in public types:

- publicly constructible planned artifacts missing validated query identity,
  validated result-shape identity, or basis requirements
- publicly constructible execution results that do not carry basis identity,
  plan identity, and counter evidence
- public plan variants that encode unsupported fallback or unsupported basis
  modes as ordinary user-buildable states

`Uncompilable` through visibility and compile-fail enforcement:

- external construction of `PlannedQueryArtifact`, `PlannedResultShapeArtifact`,
  `ExecutionPlanBundle`, or materially equivalent proof-bearing types without
  the planner path
- public execution entry points that accept weaker than validated or planned
  artifacts once the proof path exists
- external access to internal snapshot-resolution or planner-lowering helpers
  that would let callers synthesize plan proof or bypass binding policy
- public host-binding adapters that can alter query meaning outside the planner

`Construction-time rejection`:

- unsupported plan families or backend strategies
- unsupported snapshot basis classes
- invalid or incomplete host binding descriptor fulfillment
- plan/result-shape incompatibility discovered only when lowering validated
  semantics into execution phases
- unsupported widening, unsupported fallback, or unsupported binding-origin
  combinations

Rules:

- the strongest boundary available must be used
- sealed constructors and `pub(crate)` boundaries are mandatory for planned and
  executed proof types
- compile-fail coverage is required for public proof-boundary rules
- runtime rejection is allowed only for information genuinely unavailable until
  basis resolution or backend capability matching

## Scope

### In Scope

- planning of Milestone 2 validated query bundles into proof-bearing execution
  plans
- explicit snapshot-basis requirements and snapshot-bound execution contracts
  for one-shot reads
- runtime-backed execution against admitted `worth-relational` truth surfaces
- type-bound binding descriptors that resolve onto planner-owned inputs without
  changing canonical query meaning
- explicit separation between planner lowering, snapshot resolution, execution,
  and result shaping
- execution envelopes carrying plan identity, basis identity, typed results,
  diagnostics, and exact counters
- typed execution failures for unsupported plan shapes, unsupported backends,
  binding failures, basis failures, fallback denials, and plan/result
  invariant breaks
- milestone-native certification proving planner, executor, and binding parity

### Explicitly Out Of Scope

- collection-scale pagination, cursor progression, aggregation, or CDC-shaped
  output beyond whatever metadata the planner must reserve for Milestone 4
- live promotion, invalidation, or `worth-signal` integration
- historical, diff, lineage, or correspondence execution semantics
- policy masking and tenant-schema semantics beyond future-safe basis slots
- durable saved-query persistence and import/export
- store-backed pushdown features that cannot yet prove plan/result parity
- host framework lifecycle management, network delivery, or route/controller
  orchestration

## Planning And Execution Architecture

### One Planner-Owned Meaning Transition

Milestone 3 must introduce exactly one meaning transition between validated
query semantics and executable read semantics.

That transition is planning.

Planning consumes:

- one `ValidatedQueryBundle`
- one explicit `PlanningSemanticInputs`
- one explicit `PlanningAmbientContext`
- one explicit binding fulfillment set where type-bound execution is used
- one explicit basis intent

Planning produces:

- one `PlannedQueryArtifact`
- one `PlannedResultShapeArtifact`
- one `ExecutionPlanBundle`
- one planning report with typed diagnostics and counters

`PlanningSemanticInputs` is the only planner input family allowed to affect
`plan_digest`.

`PlanningAmbientContext` may carry host labels, tracing richness, UI route
names, or other explanatory metadata, but is excluded from all canonical
digests and may not change route selection, fallback policy, basis
requirements, or result-shaping semantics.

Forbidden planner input classes:

- hidden policy selectors
- hidden tenant selectors
- hidden execution backend overrides
- hidden fallback preferences
- hidden projection, predicate, ordering, or traversal mutations
- host-local closures or opaque callback handles

Execution may derive results from the plan. It may not derive new meaning that
the planner failed to express.

Planner-owned decisions include:

- which validated projection and traversal surfaces are actually read
- which narrowing decisions are locked into the read path
- what execution path is legal for the validated query family
- which result-shaping work belongs before or after runtime reads
- what snapshot basis requirements the executor must satisfy
- whether authoritative fallback exists and under what exact contract

Executor-owned work is deliberately narrower:

- obtain or receive an admitted snapshot matching the plan's basis contract
- perform the reads the plan already declared
- materialize the declared typed result shape
- emit counters, diagnostics, and basis identity
- fail if plan requirements cannot be honored

The executor may not reinterpret predicates, re-check schema legality except as
invariant assertions, widen projection or traversal breadth, or choose a
semantically different fallback path than the plan admitted.

### Snapshot-Bound Basis Contract

Milestone 3 must make basis identity explicit before execution begins.

The planner does not own truth semantics, but it does own the contract for what
kind of truth basis the executor must read.

Minimum milestone basis artifacts:

- `ExecutionBasisIntent`
- `ResolvedSnapshotBasis`
- `BasisAuthorityFamily`
- `ResolvedSnapshotIdentity`
- `SnapshotLineageClass`
- `BasisResolutionMode`
- `ResolvedBasisProof`
- `BasisDigest`
- `SnapshotResolutionReport`

The basis contract must state at least:

- authority family: runtime-backed now, store-backed only where admitted
- basis kind: current stable snapshot in this milestone
- resolved snapshot identity: one canonical identity for the exact basis read
- lineage class: whether the basis is current-head, equivalent replay, or a
  future extension class
- resolution mode: direct runtime resolution, admitted store resolution, or
  another explicitly admitted mode
- replay class: whether the basis is replayable, equivalent-only, or future
  extensible
- fallback allowance: whether any alternate basis resolution path exists

Rules:

- execution begins only after a `ResolvedSnapshotBasis` is present
- basis identity participates in execution outputs and certification bundles
- changing basis identity must change `basis_digest`
- unsupported basis kinds fail before reads begin
- "read latest truth" is not a valid basis contract
- `ResolvedSnapshotBasis` equality must be defined by typed basis fields rather
  than runtime handle equality
- runtime handles, store cursors, and adapter-local references are
  explanatory-only and excluded from `basis_digest`

`ResolvedSnapshotIdentity` must be composed from explicit identity-bearing
fields rather than "whatever the runtime gives back."

Minimum identity-bearing basis fields:

- `BasisAuthorityFamily`
- canonical branch/workspace authority if present
- canonical snapshot or commit identity
- canonical schema-basis identity where execution legality depends on it
- canonical replay/equivalence class

Two bases are equivalent only if all identity-bearing basis fields compare
equal.

Runtime-backed basis semantics in Milestone 3:

- the executor must read through stable snapshot or materially equivalent
  immutable truth surfaces from `worth-relational`
- the snapshot must be explicit enough that repeated execution can prove it
  read the same truth basis
- basis resolution must be separated from query planning even if one facade
  call performs both operations

Store-backed basis semantics in Milestone 3:

- store-backed execution is optional and admitted only where the resolved store
  basis can compare honestly to the runtime basis
- if store parity is not yet honest, store support must fail typed rather than
  silently diverging

### Plan Artifact Families

Milestone 3 must introduce proof-bearing plan artifacts rather than treating
`planned` as a Boolean on the validated bundle.

Minimum artifact families:

1. `PlanningRequestContext`
2. `PlannedQueryArtifact`
3. `PlannedResultShapeArtifact`
4. `PlannedExecutionRoute`
5. `ExecutionMechanics`
6. `FallbackDisposition`
7. `BindingRequirements`
8. `BoundBindings`
9. `BindingResolution`
10. `ExecutionPlanBundle`
11. `ExecutionResultEnvelope`

The `PlannedQueryArtifact` owns:

- validated query identity
- plan family identity
- admitted planned execution route
- frozen narrowing decisions
- read breadth contract
- snapshot requirements
- fallback disposition
- planner-owned digests

The `PlannedResultShapeArtifact` owns:

- validated result-shape identity
- execution-stage partitioning for shaping work
- delivery-relevant result semantics for one-shot execution
- reserved non-semantic extension slots only where they are:
  - non-authoritative
  - excluded from all canonical digests
  - unable to alter planning, execution, or result meaning in Milestone 3
  - typed so they cannot be mistaken for admitted semantics

The `PlannedExecutionRoute` owns:

- canonical backend family
- route identity that affects plan semantics
- planner-visible route invariants

The `ExecutionMechanics` owns:

- route-local cost markers
- non-identity-bearing optimization metadata
- backend-specific handles excluded from canonical digests

The `FallbackDisposition` owns:

- whether fallback is forbidden, admitted-but-unused, or admitted-and-selected
- the exact semantic class of fallback the planner is allowed to authorize

The `BindingRequirements` owns:

- every binding slot that must be fulfilled before planning can close
- the required subject kind for each slot
- whether a slot is identity-bearing for canonical query meaning

The `BoundBindings` owns:

- one canonical fulfillment per admitted slot
- no duplicate, ambiguous, or extra slot fulfillment
- only typed subjects, never opaque host values

The `BindingResolution` owns:

- `BindingFulfillmentDigest`
- completeness proof over the `BindingRequirements`
- conflict-free proof over the `BoundBindings`
- diagnostic-only residue for excluded binding-origin metadata

The `ExecutionPlanBundle` owns:

- `query_digest`
- `plan_digest`
- `basis_requirements_digest`
- compatibility relation between planned query, planned result shape,
  semantic route, and binding proof
- planning report
- planning counters

Derived-only artifacts include materialized row buffers, runtime adapter
structs, host framework request context, UI route metadata, and runtime/store
handles. Those may attach to diagnostics, but they never become plan
authority.

### Type-Bound Binding Parity Contract

Type-bound execution is admitted in Milestone 3 only under a strict parity
rule: binding descriptors are allowed to supply planner-owned inputs, but they
are never allowed to rewrite planner-owned semantics.

Binding descriptors may supply:

- root subject identity
- admitted collection anchor identity
- declared parameter slot fulfillment once those slots are part of the query's
  canonical meaning

Binding fulfillment may not be represented as string-keyed maps, untyped JSON
objects, or host-owned callback resolution state.

Milestone 3 must freeze explicit wrappers materially equivalent to:

- `BindingSlotId`
- `BindingRequirements`
- `BoundBindings`
- `BindingResolution`
- `BindingFulfillmentDigest`

Binding descriptors may not supply:

- hidden predicates
- hidden projection expansion
- hidden traversal depth
- hidden policy context
- hidden fallback preferences
- hidden backend selection

Milestone 3 binding parity rule:

- direct invocation and descriptor-bound invocation for the same canonical
  meaning must produce the same `plan_digest`
- semantically different binding fulfillment must produce a different
  `BindingFulfillmentDigest`, and must change `plan_digest` only when the
  fulfilled slot is identity-bearing
- binding-origin metadata excluded from canonical identity may survive only in
  diagnostics
- incomplete or conflicting binding fulfillment must fail before planning
  yields a finished `ExecutionPlanBundle`
- extra undeclared binding fulfillment must fail typed rather than being
  ignored as convenience residue

Representative binding flow:

1. canonical and validated query meaning already exists
2. `BindingRequirements` is derived from canonical and validated meaning
3. binding fulfillment produces one `BindingResolution`
4. planner lowers fulfilled meaning into one `ExecutionPlanBundle`
5. execution consumes that bundle exactly as if the query were invoked directly

### Execution Envelope And Result Contract

Milestone 3 must make the result boundary self-describing enough that consumers
and certification harnesses can explain what happened without ambient runtime
access.

Minimum execution envelope categories:

- typed primary result
- `query_digest`
- `plan_digest`
- `basis_digest`
- execution diagnostics
- exact counter snapshot
- route/fallback residue report
- narrowing report

The result envelope must be sufficient to answer:

- what query meaning executed
- what plan meaning executed
- what truth basis was read
- whether an authoritative route or fallback route was used
- what breadth the read actually touched
- whether result shaping stayed within the declared plan

Rules:

- the result envelope is derived from canonical truth and the execution plan
- diagnostics tier may change verbosity, but not the canonical result fields
- typed result-shape meaning remains query-owned, not serializer-owned
- execution counters belong in the envelope, not in side-channel logs only

## Plan Digest Basis Rules

Milestone 3 must define plan identity precisely enough that two independent
planning runs can produce the same `plan_digest` for the same validated query
meaning, fulfilled bindings, and basis intent.

Included in `PlanDigest`:

- validated query identity
- validated result-shape identity
- admitted execution semantic route
- frozen read breadth class
- frozen narrowing decisions
- snapshot basis requirements
- fallback admission class
- identity-bearing binding fulfillment digest
- execution-stage partitioning decisions that affect result meaning

Excluded from `PlanDigest`:

- planner implementation details
- diagnostics richness
- host labels and route names
- non-identity-bearing binding-origin metadata
- execution mechanics
- runtime-specific handle values
- actual resolved snapshot instance identity
- elapsed time or profiling data

Normalized ordering requirements:

- projection and traversal order inherit validated canonical ordering
- route families sort by canonical route identity
- fallback clauses sort by canonical fallback role
- binding slots sort by canonical slot identity
- execution-stage shaping steps sort by canonical stage then field identity

Equivalence relation:

- two plans are equivalent if they declare the same validated identity, route
  family, narrowing decisions, basis requirements, fallback admission class,
  identity-bearing binding fulfillment, and execution-stage shaping semantics

Conflict behavior:

- conflicting route selection for the same validated meaning is a typed
  planning failure
- conflicting fallback disposition is a typed planning failure
- conflicting stage partitioning that would change result semantics is a typed
  planning failure
- conflicting identity-bearing binding fulfillment is a typed planning failure

`BasisDigest` rules:

- `basis_digest` identifies the resolved snapshot basis, not merely the basis
  intent
- changing runtime/store authority, snapshot identity, or future historical
  basis identity must change `basis_digest`
- two equivalent executions over the same resolved basis must emit identical
  `basis_digest`

Parity rule:

- runtime-backed and store-backed execution do not need identical
  `ExecutionMechanics`
- they do need identical `plan_digest` whenever they claim to execute the same
  planned semantics
- if backend choice changes semantics, the route must be modeled as a distinct
  `PlannedExecutionRoute` rather than hidden inside mechanics

## Query Surface And Abstraction Rules

Milestone 3 must preserve one public facade while still letting planning and
execution decompose internally by responsibility.

Correct abstraction:

- public planning and execution entry through the `worth-query` facade
- internal decomposition by planning, basis resolution, execution, binding,
  result shaping, diagnostics, and certification
- one proof-bearing bundle crossing each phase boundary

Incorrect abstraction:

- one `query_runtime` module that owns planning, execution, bindings,
  snapshots, counters, and result shaping together
- host-specific execution helpers that bypass the planner and call runtime
  reads directly
- planner APIs that expose raw `worth-relational` adapter internals as public
  query contracts

Rules:

- external consumers depend on the facade and proof-bearing bundles only
- internal planner and executor helpers stay `pub(crate)` unless the facade
  explicitly promotes them
- harness and certification code must reuse production planners and executors
  rather than carrying alternate semantics
- `PlanningRequestContext` must be mechanically split so semantic inputs,
  ambient diagnostics, and forbidden host influence cannot share one loose bag

## Required Internal Subsystems

Milestone 3 should decompose into explicit subdomains materially equivalent to:

- `planning/`
- `planning/artifacts/`
- `execution/`
- `execution/artifacts/`
- `basis/`
- `binding/runtime/`
- `result_projection/`
- `diagnostics/planning/`
- `diagnostics/execution/`
- `harness/planning_certification/`
- `harness/planning_matrix/`

Responsibility map:

- `planning/` lowers validated meaning into route-safe execution artifacts
- `basis/` resolves and reports snapshot identity
- `execution/` consumes plan plus basis and materializes typed results
- `binding/runtime/` fulfills query-owned binding descriptors without changing
  canonical meaning
- `result_projection/` applies planner-declared shaping phases
- `diagnostics/*` own reports and counters, not semantic truth
- `harness/*` proves parity using production code paths

Boundary rules:

- `planning/` must not own schema legality
- `execution/` must not invent new route-selection policy
- `basis/` must not invent query semantics
- `binding/runtime/` must not become a host-specific alternate authoring layer
- `harness/` must not fork planner or executor semantics
- `result_projection/` must not smuggle future view-shape or delivery semantics
  into Milestone 3 through reserved metadata

## Phases

Milestone 3 is intentionally split into six phases because planning, binding
fulfillment, basis resolution, execution, and certification are separate proof
boundaries. Collapsing them into three broad buckets would hide real authority
transitions and make later implementation drift much more likely.

### Phase 1: Freeze Planned Artifact, Binding, And Basis Authority

Phase 1 exists to define exactly what planning is allowed to know and what it
is allowed to prove.

Milestone 3 must first define:

- planned artifact families and sealed constructors
- execution route vocabulary and fallback policy vocabulary
- execution basis intent vocabulary and resolved snapshot basis identity
- binding fulfillment vocabulary for type-bound execution
- plan digest and basis digest rules

This phase leaves the system in a coherent state where:

- planning has one authoritative input surface: `ValidatedQueryBundle`
- execution basis is explicit instead of ambient
- host binding is classified as fulfillment, not semantic authoring
- unsupported plan families are named and fail closed instead of drifting
  through executor convenience logic

### Phase 2: Build Binding Fulfillment And Plan Inputs

Phase 2 exists to prevent host invocation paths from becoming a hidden second
query language.

Milestone 3 must then implement:

- derivation of `BindingRequirements` from canonical and validated query meaning
- typed binding fulfillment into `BoundBindings`
- `BindingResolution` assembly and `BindingFulfillmentDigest`
- typed rejection for missing, extra, ambiguous, or conflicting binding
  fulfillment
- mechanical separation between `PlanningSemanticInputs` and
  `PlanningAmbientContext`

This phase leaves the system in a coherent state where:

- direct and bound invocation feed the planner through the same semantic input
  model
- binding fulfillment is proof-bearing instead of stringly host glue
- the planner can rely on one closed set of semantic inputs before route or
  basis lowering begins

### Phase 3: Lower Validated Query Bundles Into Proof-Bearing Execution Plans

Phase 3 exists to make the planning vocabulary operational.

Milestone 3 must then implement:

- planning of validated projection, traversal, predicate, and ordering meaning
  into admitted runtime read paths
- deterministic route selection and fallback admission
- planner-owned result-shaping stage partitioning
- planned query and planned result-shape artifact construction
- typed planning reports and exact planning counters

This phase leaves the system in a coherent state where:

- the same validated meaning and basis intent lower into the same plan
- route selection is explicit rather than hidden in execution code
- the executor can treat the plan as authority instead of rediscovering query
  semantics

### Phase 4: Resolve Stable Snapshot Basis And Lock Execution Preconditions

Phase 4 exists to make basis selection explicit and certifiable before any read
occurs.

Milestone 3 must then ship:

- resolution of `ExecutionBasisIntent` into `ResolvedSnapshotBasis`
- `ResolvedSnapshotIdentity`, `BasisResolutionMode`, and `ResolvedBasisProof`
- typed rejection for unsupported basis kinds and failed basis resolution
- explicit preflight checks that plan route, fallback policy, and basis class
  are compatible before execution starts
- canonical `basis_digest` emission rules

This phase leaves the system in a coherent state where:

- execution never starts against ambient latest truth
- basis identity is frozen before read breadth is measured
- runtime-backed and admitted store-backed lanes can compare basis semantics
  honestly

### Phase 5: Execute Plans And Materialize Typed Result Envelopes

Phase 5 exists to make execution a faithful derivation of the plan rather than
an alternate source of truth.

Milestone 3 must then ship:

- execution of planned reads against stable truth surfaces
- planner-declared result shaping over execution outputs
- execution result envelopes with basis identity, counters, and diagnostics
- typed execution failures for basis mismatch, unsupported route, and forbidden
  fallback

This phase leaves the system in a coherent state where:

- one-shot execution is snapshot-honest
- typed result meaning is derived from the plan instead of host-local shaping
- runtime-backed execution can serve as the canonical reference lane for later
  parity work

### Phase 6: Certification, Counter Proof, And Boundary Hardening

Phase 6 exists to prove that planning and execution are parity-safe and
boundary-safe.

Milestone 3 must finally ship:

- milestone-native certification through the `Planner / Executor / Binding
  Parity Test`
- hostile parity fixtures covering direct invocation, repeated planning,
  bound invocation, and rejected backend/fallback scenarios
- exact counter assertions for representative admitted and rejected cases
- compile-time or harness hardening proving that weaker artifacts cannot bypass
  into execution once planned artifacts exist

This phase leaves the system in a coherent state where:

- planning and execution are certifiable rather than plausible
- later milestones can build collection, live, and historical semantics on top
  of a frozen one-shot execution substrate
- host-bound execution paths no longer threaten semantic drift

## Must Ship

- proof-bearing `PlannedQueryArtifact`, `PlannedResultShapeArtifact`, and
  `ExecutionPlanBundle` families or materially equivalent types
- one query-owned execution basis contract and resolved snapshot basis artifact
- deterministic runtime-backed planning over Milestone 2 validated query
  bundles
- one-shot snapshot-bound execution against admitted `worth-relational` truth
  surfaces
- type-bound binding fulfillment and parity-safe plan lowering
- typed planning reports, execution reports, digests, and exact counters
- typed execution/result envelopes carrying `query_digest`, `plan_digest`, and
  `basis_digest`
- milestone-native certification proving planner, executor, and binding parity
- sealed planned/executed construction so weaker artifacts cannot leak into
  execution by accident

## Must Preserve

- canonical query meaning from Milestone 1 remains authoritative
- schema legality from Milestone 2 remains authoritative and is not rediscovered
- execution reads through explicit stable basis identity rather than ambient
  mutable truth
- results remain derived from canonical truth surfaces
- binding descriptors remain query-owned metadata rather than host-owned query
  synthesis
- unsupported plan/backend/fallback combinations fail explicitly rather than
  widening or degrading silently
- no alternate public planning or execution path exists outside the query
  facade

## Complexity / Proof Obligations

Milestone 3 must name costs and proofs in terms of:

- validated projection width
- validated traversal clause count
- planned read breadth
- snapshot basis resolution work
- fallback admission count
- actual breadth touched during execution
- result-shaping field count

Minimum required counters:

- `planned_projection_entry_count`
- `planned_traversal_clause_count`
- `route_candidate_count`
- `planned_read_surface_count`
- `planned_fallback_option_count`
- `fallback_denial_count`
- `snapshot_basis_resolution_count`
- `execution_read_operation_count`
- `execution_records_examined_count`
- `execution_records_emitted_count`
- `execution_fallback_taken_count`
- `execution_result_shape_binding_count`
- `post_read_shape_field_count`
- `executor_semantic_rediscovery_count`

Rules:

- planning counters belong to the plan bundle
- execution counters belong to the execution result envelope
- representative certification scenarios must assert exact counts
- any non-zero forbidden widening residue on a supported path is forbidden in
  Milestone 3
- any executor rediscovery of planner-owned decisions is forbidden and must be
  detectable through counters or typed invariant failure
- `executor_semantic_rediscovery_count` must be exactly zero on every admitted
  path
- fallback denial must be counted separately from fallback taken so the system
  cannot hide forbidden route pressure inside generic failure totals

Normative representative scenarios:

- legal detail query through direct invocation
- equivalent direct query through independently repeated planning
- equivalent descriptor-bound execution
- changed identity-bearing binding fulfillment yields different plan identity
- changed basis intent yields different plan identity or typed rejection
- unsupported backend route request
- forbidden fallback widening case
- snapshot basis resolution failure
- result-shape stage partitioning parity case

### Milestone 3 Planner / Executor / Binding Certification Matrix

Milestone 3 closeout must include a named certification matrix rather than only
loosely related tests.

The matrix is the closeout surface for Milestone 3 parity claims. "Covered
elsewhere in tests" does not satisfy this requirement.

Milestone 3 must ship one named certification artifact materially equivalent
to:

- `planner_executor_binding_parity_certification_artifact`
- `planning_execution_certification_matrix`
- `bundle_completeness_report`

Minimum required canonical rows:

- `direct-runtime-plan-parity`
- `replanned-runtime-parity`
- `type-bound-runtime-parity`
- `runtime-basis-repeatability`

Minimum required intentional-difference rows:

- `identity-bearing-binding-difference`
- `basis-difference`
- `route-semantic-difference`

Minimum required rejection rows:

- `unsupported-backend-route`
- `unsupported-fallback-shape`
- `binding-fulfillment-conflict`
- `snapshot-basis-resolution-failure`

Each row must emit or reference:

- `query_digest`
- `plan_digest`
- `result_digest` where executed
- `basis_digest` where basis resolution succeeded
- `failure_digest` where rejected
- `counter_snapshot`

And each row must prove one of:

- equivalent planned identity
- equivalent execution result identity
- intentionally distinct planned or result identity
- typed early rejection before unsupported execution begins

Rules:

- required rows may not be satisfied by nearby or semantically similar rows
- if a row depends on a still-blocked admitted store path, the artifact must
  mark it unmet rather than silently omit it
- Milestone 3 may not be declared closed while any required runtime-backed row
  remains unmet
- at least one intentional-difference row must prove that over-canonicalization
  does not collapse materially different semantics into one digest

## Allowed Debt

- store-backed parity may remain `Debt` while runtime-backed planning and
  execution are canonical and honest
- richer route optimization may remain `Debt` if admitted routes already expose
  exact counters and preserve semantics
- compile-time proof for some binding-fulfillment shapes may remain `Debt` if
  construction-time proof is sealed and parity-safe
- executor rediscovery of planner-owned legality, hidden widening, or ambient
  mutable reads may not exist as debt

## Acceptance Evidence

Milestone 3 is complete only when `worth-query` can prove:

- the `Planner / Executor / Binding Parity Test` in
  [test-requirements.md](./test-requirements.md)
  passes with canonical machine-checkable artifacts
- identical validated query meaning and basis intent lower into identical
  execution plans
- one-shot execution reads through stable snapshots and returns the declared
  typed result shape
- direct invocation and admitted type-bound invocation round-trip to the same
  canonical plan and result meaning
- intentionally different identity-bearing binding or basis inputs produce
  distinct digests or typed rejection rather than accidental equivalence
- `counter_snapshot` proves exact planning/execution breadth and shows zero
  forbidden widening residue on supported paths
- runtime-backed execution serves as the canonical parity reference lane, and
  any admitted store-backed lane compares equal for the same basis where store
  support exists

## Architectural Notes

### Law 41 Remains The Load-Bearing Rule

The most important hardening rule in this milestone is still that the type must
encode what has been proven.

That means:

- validated query types are not planned query types
- planned query types are not executed result envelopes
- basis intent is not the same thing as resolved snapshot basis
- bound invocation metadata is not the same thing as canonical query meaning

If an implementation exposes a public constructor for planned or executed proof
artifacts that does not pass through the proving path, the milestone has been
violated even if happy-path execution still works.

### Planning Must Freeze Meaning, Not Rediscover It

Milestone 3 may use smart abstractions only where they preserve semantic,
correctness, and cost honesty.

The planner may be smart in these ways:

- choose among semantically equivalent admitted runtime routes
- normalize equivalent basis intents
- reserve execution-stage shaping steps for later reuse

If Milestone 3 keeps any reserved extension slots, they must be typed as
non-semantic placeholders and must be:

- excluded from `query_digest`, `plan_digest`, `result_digest`, and
  `basis_digest`
- incapable of affecting route selection, basis resolution, fallback
  admission, or result meaning
- rejected if any caller attempts to author them as active semantics

It must not be smart in these ways:

- infer hidden predicates from host context
- broaden projection or traversal because a narrower route is inconvenient
- re-check legality that validation already owned
- change result-shape meaning depending on the chosen execution backend

### Snapshot-Honest Execution Is Mandatory

Milestone 3 is the point where `worth-query` must stop pretending that "run it
now" is a meaningful truth contract.

Execution must instead be explicit about:

- what basis was read
- whether that basis was runtime-backed or store-backed
- whether fallback occurred
- how much breadth was touched

Without that, later live, diff, and historical milestones would be building on
ambient truth reads rather than a certifiable execution substrate.

## Sequencing Notes

This belongs third because collection semantics, live promotion, and historical
execution all depend on proof-bearing one-shot plans and snapshot-honest
execution.

Milestone 3 must land before:

- Milestone 4 collection semantics, because collection breadth and ordering
  need the planner/executor substrate first
- Milestone 5 live promotion, because live maintenance must preserve plan
  meaning already proven in one-shot execution
- Milestone 6 historical and diff semantics, because basis identity has to be
  explicit before multiple basis kinds can be compared honestly

## Parallelization Notes

Once the planning proof boundary is frozen:

- Milestone 4 collection planning can begin against the same planned artifact
  model
- early store-parity experimentation can proceed in parallel as long as it does
  not weaken runtime-backed canonicality
- host-framework binding adapters can proceed in parallel as long as they lower
  exclusively into the same binding-fulfillment and planning paths
- result-shape and delivery work for later milestones can extend the execution
  envelope without redefining plan semantics

## Explicit Failure Taxonomy For Milestone 3

Milestone 3 must ship typed failures for at least:

- unsupported execution route family
- unsupported backend parity request
- unsupported fallback shape
- binding fulfillment missing required slot
- binding fulfillment conflict
- unsupported basis kind
- snapshot basis resolution failure
- plan/result-shape compatibility failure
- planner invariant break
- executor invariant break
- forbidden widening denied
- forbidden executor rediscovery detected

These are query planning and execution failures, not raw strings and not
borrowed runtime errors.

## Anti-Patterns Explicitly Rejected

- accepting canonical or validated artifacts directly into runtime reads without
  a planned proof phase
- resolving against ambient mutable truth instead of explicit snapshot basis
- host route/controller glue that rewrites query meaning during binding
- string-keyed or JSON-shaped binding fulfillment bags
- executor-owned legality discovery
- hidden widening to a broader authoritative route because a narrow route failed
- semantic route and mechanical route detail collapsed into one type
- one mega-runtime module that mixes planning, basis resolution, binding,
  execution, and result shaping
- certification harnesses that fork planner or executor semantics
- any plan identity that changes because diagnostics richness changed

## Self-Check

This milestone solves a real structural problem rather than packaging work
cosmetically because it introduces the first proof-bearing execution boundary
between validated query meaning and actual truth reads.

The adversarial constraint is load-bearing because it forbids the naive failure
mode of letting execution path, binding path, or basis resolution path change
the semantic meaning of the same validated query.

The milestone preserves authority boundaries because `worth-relational` still
owns truth semantics, validation still owns legality, `worth-query` owns
planning and result shaping, and host frameworks remain consumers rather than
semantic co-authors.

The milestone defines proof obligations rather than implementation chores
because deterministic plan identity, basis identity, counter evidence, typed
fallback denials, and parity certification are required for closeout.

A competent engineer should be able to map this spec into honest planning,
basis, execution, binding, and certification modules without inventing the
architecture during implementation.

This milestone belongs third in the roadmap because one-shot snapshot-honest
execution has to exist before collection scale, live promotion, and historical
basis expansion can be honest.

## Closeout Standard

Milestone 3 is complete only when all of the following are true:

- a proof-bearing planned-query boundary exists
- runtime-backed one-shot execution runs only through explicit snapshot basis
- direct invocation and admitted type-bound invocation lower into the same plan
  semantics
- unsupported route, backend, basis, and fallback shapes fail typed and early
- legal executions emit deterministic `plan_digest`, `result_digest`, and
  `basis_digest` for the same inputs
- no supported path widens or silently degrades
- certification proves planner parity, executor parity, binding parity, and
  exact planning/execution counter behavior with canonical machine-checkable
  artifacts

If code lands but execution still depends on ambient mutable reads, executor
rediscovery, route-local query synthesis, or non-sealed planned artifacts,
Milestone 3 is not complete.
