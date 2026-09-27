# Milestone 5 Engineering Spec: Live Query Promotion, Query-Shaped Patches, And Convergence Proof

> **Status:** Draft engineering spec
>
> **Roadmap parent:** [worth_query_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/worth_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/worth_query_vision.md)
>
> **Prior milestone:** [milestone-4.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/milestone-4.md)
>
> **Prior closeout:** [milestone-4-closeout.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/milestone-4-closeout.md)
>
> **Test requirements:** [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/test-requirements.md)
>
> **Primary architectural driver:** make live promotion a proof-bearing execution mode over the Milestone 3 and Milestone 4 plan substrate so one-shot query meaning survives invalidation, patch construction, suppression, and replay without degrading into raw CDC or host-local recomputation
>
> **Companion docs:**
> - [MENTALITY.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/docs/coding-guidelines/MENTALITY.md)
> - [arch_laws.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/docs/coding-guidelines/arch_laws.md)
> - [perf_laws.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/docs/coding-guidelines/perf_laws.md)
> - [domain_laws.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/docs/coding-guidelines/domain_laws.md)
> - [worth_query_vision.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/worth_query_vision.md)
> - [worth_query_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/worth_query_roadmap.md)
> - [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/test-requirements.md)
> - [milestone-3.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/milestone-3.md)
> - [milestone-3-closeout.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/milestone-3-closeout.md)
> - [milestone-4.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/milestone-4.md)
> - [milestone-4-closeout.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/milestone-4-closeout.md)

## Goal

Make live promotion a first-class proof-bearing execution mode so admitted
detail, ordered collection, and bounded materialization queries can execute as
one-shot reads or live-maintained subscriptions without changing canonical
query meaning, while query consumers receive query-shaped patches rather than
raw CDC or host-recomputed deltas.

## Why This Milestone Exists

Milestone 3 made one-shot planning and snapshot-bound execution honest.
Milestone 4 made ordered collection semantics, bounded traversal/materialized
breadth, derived-field shaping, rollups, and CDC-shaped result families
planner-owned and digest-bearing.

Milestone 5 is where those guarantees either survive time or collapse into a
second product.

If live query support is bolted on as:

- raw runtime bridge events plus host-side filtering
- subscription-specific builders or execution APIs
- collection re-sorting and row membership repair in clients
- best-effort suppression without query-shaped semantics
- background full re-execution disguised as incremental maintenance

then the vision breaks. `worth-query` would have one truth for one-shot reads
and a different truth for long-lived consumers.

This milestone therefore exists to freeze:

- that live promotion is an execution-mode change, not a query-language change
- that query relevance is derived from planner-owned metadata, not subscriber
  heuristics
- that patch construction is a query artifact family, not a raw event stream
- that suppression is planner-owned and explicit, not a transport optimization
- that live maintenance converges to the same truth as fresh execution for the
  same basis and change stream

## Governing Summaries

- `MENTALITY.md`: the hard problem is not "deliver updates." It is making the
  same query remain semantically identical across one-shot execution, long-lived
  maintenance, replay, and suppression pressure. The milestone must solve that
  adversarial integrity problem first.
- `arch_laws.md`: Laws 2, 6, 7, 8, 17, 18, 21, 22, 26, 27, 30, 33, 35, 40, and
  especially 41 dominate this milestone. Relevance contracts must be declared,
  domain effects and observability must stay separate, batch summaries should
  carry cross-phase facts once, and live proof types must encode exactly what
  has been proven.
- `perf_laws.md`: live maintenance must degrade by explicit policy, not hidden
  breadth or coordination. Invalidation breadth, patch width, suppression, and
  full re-execution fallback must be named, counted, and proven at the live
  boundary.
- `domain_laws.md`: promotion, relevance analysis, bridge-facing invalidation
  metadata, patch families, suppression policy, replay harnesses, and live
  diagnostics are different responsibilities and must not collapse into one
  "subscriptions" module.
- `worth_query_vision.md`: read-to-subscribe promotion, incremental result
  maintenance, query-to-signal bridging, ordered live collections, and
  tolerance-aware aggregation are product theses. They must be query-shaped,
  basis-honest, and typed.
- `worth_query_roadmap.md`: Milestone 5 is the first proof that query meaning
  survives time instead of only one-shot reads. It belongs after Milestone 4
  because live maintenance must inherit already-frozen ordering, pagination,
  traversal, and CDC-family semantics.
- `test-requirements.md`: the `Live Promotion Convergence And Suppression Test`
  is the closeout proof. It requires convergence between live-maintained and
  fresh execution, explicit suppression of irrelevant updates, query-shaped
  patches, and canonical machine-checkable replay evidence.
- `milestone-3.md`: planning, basis resolution, and one-shot execution are
  already explicit proof-bearing phases. Milestone 5 must extend those proofs
  into live mode rather than inventing a parallel live runtime.
- `milestone-3-closeout.md`: runtime-backed one-shot execution is already the
  canonical parity lane. Live mode must preserve that one-shot meaning and use
  it as the freshness oracle during certification.
- `milestone-4.md`: stable ordering, bounded traversal/materialization,
  aggregate and rollup families, derived-field shaping, and CDC-shaped result
  families are already planner-owned collection semantics. Milestone 5 must
  maintain those exact semantics under churn rather than re-deciding them from
  raw change events.
- `milestone-4-closeout.md`: the current crate already has a runtime-backed
  collection certification harness, planner-owned cursor semantics, and zero
  executor rediscovery on one-shot paths. Milestone 5 should reuse that harness
  style and those plan artifacts rather than loosening them.

## Adversarial Constraint

Milestone 5 must survive the following hostile condition:

> An admitted query begins from one explicit stable basis, is promoted into
> live mode, receives a long stream of relevant and irrelevant truth changes in
> varying order, applies query-shaped incremental maintenance with optional
> suppression, and must converge to exactly the same typed result meaning and
> delivery-family meaning as repeated fresh re-execution of the canonical query
> over the same evolving truth history.

Concretely, the design must remain correct when all of the following are true:

- the same admitted query is executed once as a one-shot read and once as a
  live subscription from the same starting basis
- detail queries, ordered collections, and bounded materialization families
  experience updates that change projection values, collection membership,
  ordering keys, traversal-visible descendants, and derived-field outputs
- many updates are irrelevant to the query and should be suppressed before
  patch construction or delivery
- some updates are relevant but below a declared admitted suppression threshold
  and should not produce visible consumer change
- signal invalidation, bridge routing, and query patch construction happen in
  distinct subsystems with distinct ownership boundaries
- subscribers live long enough that naive systems would drift, leak breadth, or
  depend on hidden full refreshes

If any supported path:

- changes query meaning because live promotion used a different query artifact
  than one-shot execution
- exposes raw CDC as the consumer contract and makes the consumer reconstruct
  query semantics
- re-sorts, re-filters, or repairs membership outside the planned query
  substrate
- treats relevance as host heuristic instead of planner-owned metadata
- suppresses updates in a way that changes the final query truth instead of
  only delivery cadence
- cannot replay the same truth changes into the same patch evolution

then Milestone 5 has failed.

## Product Decision Lock

- live promotion is an execution-mode transformation over canonical admitted
  query artifacts, not a second query language and not a subscription-only
  builder family
- live-maintained results are query-shaped and result-family-shaped; consumers
  do not reinterpret raw CDC into detail rows, collection splices, or material
  child updates
- `worth-signal` owns scheduling and dependency invalidation mechanics;
  `worth-query` owns how a query lowers into signal-facing relevance contracts
  and query-shaped patches
- the runtime bridge owns patch-to-invalidation routing and change summaries;
  `worth-query` must consume bridge-owned change descriptors rather than reach
  into bridge internals or duplicate routing logic
- live mode must preserve Milestone 3 basis identity and Milestone 4 ordering,
  traversal, rollup, derived-field, and CDC-family meaning
- suppression is an explicit query-plan/live-plan policy with counters and
  diagnostics, not an accidental side effect of batching or transport
- admitted full re-execution fallback, when necessary, must be typed,
  diagnostically visible, and semantically equivalent to incremental
  maintenance
- durable subscription resume, persisted checkpoints, and restart-stable live
  continuation are out of scope for this milestone and must not be implied

Normative consequence:

- any implementation path that requires a separate "live query AST" is out of
  spec
- any implementation path that delivers runtime CDC as though it were the
  query's live patch family is out of spec
- any implementation path that lets host adapters decide whether a change is
  relevant is out of spec
- any implementation path that silently falls back to broad full re-execution
  without typed evidence is out of spec

## Compile-Time Enforcement Policy

Milestone 5 must classify which live-promotion guarantees become
unrepresentable, uncompilable, or construction-time rejection.

`Unrepresentable` in public types:

- publicly constructible live plan artifacts that do not carry the source
  one-shot plan identity, basis identity, relevance contract, and patch-family
  classification
- publicly constructible live patch envelopes that do not carry result-family
  identity, patch digest, and counter evidence
- publicly constructible suppression policies or fallback policies that are
  plain bags instead of closed/sealed query-owned families

`Uncompilable` through visibility and compile-fail enforcement:

- external construction of `LiveQueryPlan`, `LivePatchEnvelope`,
  `LivePromotionBundle`, or materially equivalent proof-bearing types without
  the crate-owned lowering path
- public APIs that accept raw bridge events, raw CDC records, or host-built
  patch bags as live query maintenance input
- public execution surfaces that let consumers fabricate "relevance matched"
  or "suppressed" outcomes without going through query-owned proving code
- public conversion paths that bypass one-shot planning and directly mint live
  artifacts from raw authored or merely canonical query forms

`Construction-time rejection`:

- non-admitted query families requested for live promotion
- unsupported result-family/live combinations
- unsupported suppression classes
- unsupported patch families for a plan's declared ordering or traversal mode
- invalid basis promotion requests
- unsupported fallback strategies

Rules:

- the strongest available boundary must be used
- sealed constructors and private fields are mandatory for live proof types
- compile-fail coverage is required for live artifact privacy and for
  "no raw CDC as live patch" boundaries
- runtime rejection is allowed only for information genuinely unavailable until
  bridge change summaries or basis-compatible invalidation arrive

## Scope

### In Scope

- promotion of admitted Milestone 3 and Milestone 4 query families into live
  execution mode
- query-to-signal lowering metadata sufficient for incremental maintenance
- bridge-aware invalidation metadata and relevance contracts
- query-shaped incremental patch families for:
  - detail reads
  - ordered collections
  - bounded materialized relations
- explicit suppression of irrelevant or admitted suppressible updates
- typed live patch/result envelopes, replay bundles, diagnostics, and counters
- typed full re-execution fallback where incremental maintenance is not the
  admitted path for a specific relevant change class
- monotonic change-sequence identity and replay-safe progression artifacts
- closed refresh-admission matrices per admitted live family
- milestone-native certification proving convergence, suppression, and replay

### Explicitly Out Of Scope

- historical, diff, lineage, or correspondence semantics
- policy masking semantics as a source of legality; policy authority remains a
  later milestone even though live artifacts must preserve future masked-plan
  boundaries cleanly
- durable live subscription resume, persisted checkpoints, or restart-stable
  continuation
- store-backed live replay and store-backed subscription parity
- grouped or temporal view-shape semantics beyond the already admitted ordinary
  and CDC-shaped collection/result families
- transport-level delivery lifecycle, network protocol framing, or client cache
  ownership

## Live Query Architecture

### One Promotion Boundary

Milestone 5 extends the existing proof chain. It must not create a second live
runtime beside the one-shot planner/executor substrate.

The authoritative flow should become:

`ValidatedQueryBundle`
-> `ExecutionPlanBundle`
-> `CollectionPlanBundle` where applicable
-> `LivePromotionRequest`
-> `LiveQueryPlan`
-> `LiveSubscriptionState`
-> `LivePatchEnvelope`
-> `LiveReplayBundle`

Live promotion therefore consumes already-proven query meaning. It does not
re-author:

- predicates
- projection meaning
- ordering meaning
- traversal/materialization meaning
- result-family meaning
- basis identity

### Authority Boundaries

`worth-query` owns:

- lowering one-shot plans into live-query plans
- query relevance contracts derived from canonical query semantics
- query-shaped patch-family definitions
- suppression policy vocabulary for admitted query families
- replay/convergence certification artifacts

`worth-signal` owns:

- dependency tracking
- invalidation scheduling
- coalescing and execution timing mechanics

The runtime bridge owns:

- change-summary and patch-routing infrastructure
- canonical bridge-facing invalidation descriptors
- patch-to-query routing seams

Execution owns:

- applying the already-lowered live plan to admitted change summaries
- producing query-shaped patches or typed re-execution fallback envelopes

Hosts and delivery glue may own:

- transport of live patch envelopes
- presentation of already-lowered patches
- lifecycle of consumer subscription handles

Hosts and delivery glue may not own:

- deciding query relevance
- synthesizing collection membership changes
- inventing suppression outcomes
- re-sorting live rows
- projecting raw CDC into consumer-shaped patches

### Live Basis And Subscription Identity

Milestone 5 must preserve the Milestone 3 rule that execution is basis-honest.
Live mode begins from an explicit one-shot basis and evolves through an
explicit admitted change history.

Representative artifact families:

- `LivePromotionRequest`
- `LiveQueryPlan`
- `LiveSubscriptionIdentity`
- `LiveStartBasis`
- `LiveProgressBasis`
- `LiveChangeSequenceId`
- `LiveChangeOrdinal`
- `LiveReplayDigest`
- `LivePatchDigest`
- `LiveSubscriptionCounters`

Rules:

- live promotion begins from one `ResolvedSnapshotBasis` or materially
  equivalent explicit start basis
- `LiveSubscriptionIdentity` includes the source query/plan identity and start
  basis identity
- progression through live maintenance must update basis progress explicitly
  rather than mutating hidden runtime state
- every admitted live update must carry one explicit monotonically advancing
  `LiveChangeOrdinal` inside one explicit `LiveChangeSequenceId`; replay and
  convergence proofs compare semantic sequence identity rather than transport
  arrival timing
- equality for live subscriptions must be typed and digest-bearing, not handle
  equality on runtime-owned subscriber objects
- fresh re-execution parity for certification must compare against the live
  subscription's declared progress basis, not an ambient "latest" read
- coalescing is allowed only when the coalesced bundle preserves the same
  effective `LiveProgressBasis` and patch outcome the uncoalesced admitted
  sequence would have produced

### Relevance, Invalidation, And Suppression

Milestone 5 must introduce one closed vocabulary for query relevance.

Representative artifact families:

- `QueryRelevanceContract`
- `RelevantChangeClass`
- `IrrelevantChangeClass`
- `SuppressionDecision`
- `SuppressionReason`
- `IncrementalMaintenanceAdmission`
- `FullRefreshFallbackClass`
- `RefreshAdmissionMatrix`
- `BridgeInvalidationDescriptor`
- `BridgeChangeSummary`

Rules:

- relevance is computed from planner-owned query semantics plus bridge-owned
  change summaries
- relevance may depend on projection, ordering keys, traversal/materialization
  boundaries, and admitted derived-field dependencies
- bridge-owned change summaries for admitted incremental families must carry
  enough canonical before/after evidence to decide:
  - whether membership changed
  - whether ordering position changed
  - whether bounded materialization scope changed
  - whether a derived-field-visible value changed
- irrelevant changes must be suppressible before patch construction
- suppressible changes must not alter final query truth; they may only alter
  whether a visible patch is delivered now
- any change requiring full re-execution must be typed as such, counted, and
  admitted by a closed `RefreshAdmissionMatrix`
- query relevance must not require host-local graph walking or ad hoc
  subscription-specific callbacks

Refresh admission rule:

- full refresh may not be an open-ended escape hatch
- Milestone 5 must classify refresh-admitted change classes explicitly
- if a change class is incremental for an admitted live family, the executor
  may not silently choose refresh because incremental code was inconvenient
- refresh-admitted classes must be listed in certification artifacts by family
  and reason

### Query-Shaped Patch Families

Milestone 5 must define patch families as query result artifacts, not as raw
transport payloads.

Representative patch families:

- `DetailPatch`
- `OrderedCollectionPatch`
- `BoundedMaterializationPatch`
- `CollectionMembershipChange`
- `CollectionOrderingChange`
- `ProjectionFieldDelta`
- `DerivedFieldDelta`
- `CdcResultFamilyPatch`
- `PatchEnvelopeFamily`
- `PatchFamilyMarker<TPlanFamily>`

Rules:

- detail live patches must identify projection-visible field deltas without
  exposing non-query state
- ordered collection live patches must express membership insertion, removal,
  replacement, and position change in planner-owned ordering terms
- bounded materialization live patches must preserve declared traversal scope
  and relation-edge semantics
- patch-family construction must be type-coupled to the promoted plan family;
  a detail live plan must not be able to mint ordered-collection or bounded-
  materialization patch variants through one generic runtime bag
- CDC-shaped live patches remain query-family patches first and integration
  delivery shapes second
- if a relevant change cannot be represented honestly in the admitted patch
  family, the system must emit a typed refresh fallback rather than a misleading
  pseudo-patch

Required concrete patch semantics:

- detail patch:
  - changed projected fields
  - removed projected fields where nullability/presence changed
  - explicit "refresh required" only for closed admitted refresh classes
- ordered collection patch:
  - insert at stable ordering position
  - remove from stable ordering position
  - replace row payload without membership change
  - move row from old position to new position when ordering keys changed
- bounded materialization patch:
  - add newly in-scope related material
  - remove no-longer-in-scope related material
  - mutate already-in-scope related material

### Convergence And Replay Evidence

Live support is not honest until it can be replayed and compared to fresh
execution.

Milestone 5 must therefore introduce replay-oriented bundle families such as:

- `LiveReplayBundle`
- `LiveConvergenceReport`
- `LivePatchTrace`
- `LiveSuppressionReport`
- `LiveFallbackReport`

The replay contract must prove:

- the same starting basis and same admitted change stream produce the same live
  patch evolution
- the same live end state matches fresh re-execution at the same progress basis
- suppression decisions are explainable and deterministic for admitted change
  classes
- fallback to full refresh, where admitted, remains query-equivalent and
  explicitly visible in artifacts

## Performance Architecture

### Performance Must Be A Plan Artifact, Not An Implementation Hope

Milestone 5 must encode performance at the same boundary where it encodes live
semantics.

The live system is not allowed to claim:

- "incremental"
- "suppressed"
- "coalesced"
- "bounded"
- "cheap enough"

unless those claims exist as named query-owned artifacts, named counters, and
named certification rows.

Representative performance artifact families:

- `LiveMaintenanceComplexityContract`
- `LiveMaintenanceCostClass`
- `IncrementalPatchEligibility`
- `IncrementalMaintenanceClass`
- `PatchWidthPolicy`
- `PatchWidthBudget`
- `CoalescingAdmissionClass`
- `RefreshCostClass`
- `RefreshAdmissionStatus`
- `LivePerformanceReport`

The contract is not complete unless it names the units of work explicitly.

Representative contract dimensions by family:

- detail family:
  - projected field delta count
  - derived-field recomputation count
  - refresh fallback class
- ordered collection family:
  - membership delta count
  - ordering reposition count
  - page-local versus cross-page move class
  - refresh fallback class
- bounded materialization family:
  - in-scope node delta count
  - in-scope edge delta count
  - scope expansion/contraction count
  - refresh fallback class

These are not suggestions. A family without named units of work does not yet
have an honest complexity contract.

### Cost Classes Must Be Structural

Milestone 5 must not hide materially different live cost shapes behind one
generic maintenance abstraction.

At minimum, the architecture must distinguish:

- detail-family maintenance cost
- ordered-collection maintenance cost
- bounded-materialization maintenance cost
- refresh-fallback cost

Rules:

- a detail-family live plan must not silently inherit ordered-collection
  repositioning cost semantics
- bounded-materialization cost must remain distinct from simple row-patch cost
- refresh-fallback cost must remain distinct from incremental patch cost even
  when both are semantically correct
- live family admission must therefore be both semantic and cost-class aware
- the cost class must be carried by the promoted live plan itself, not inferred
  later from patch shape or executor branch choice

### Incremental Eligibility Must Be Proven Upstream

Incremental maintenance may not be a speculative runtime guess.

Milestone 5 must introduce an explicit eligibility proof surface such as:

- `IncrementalPatchEligibility`
- `IncrementalMaintenanceClass`
- `RefreshAdmissionMatrix`

The planner/live-promoter must decide:

- which live families are incrementally maintainable
- which change classes are incrementally maintainable
- which change classes are refresh-admitted
- which change classes are forbidden entirely

The executor may consume that proof. It may not improvise it.

### Patch Width And Coalescing Must Be Policy, Not Accident

Patch size and coalescing behavior are architecture, not transport trivia.

Milestone 5 must encode:

- one explicit `PatchWidthPolicy` per admitted live family
- one explicit `PatchWidthBudget`
- one explicit `CoalescingAdmissionClass`

Rules:

- if an incremental patch exceeds the admitted width budget, the next action
  must be determined by explicit policy rather than ad hoc branching
- coalescing is admitted only where semantic replay parity can still be proven
- coalescing must preserve the same effective progress basis and patch outcome
  as the uncoalesced admitted sequence
- "just batch it more" is not an architectural answer unless the coalescing
  class is explicitly admitted and certified
- width budget must be expressed in family-owned units, not one generic "patch
  size" scalar that hides cost differences between field deltas, row moves, and
  materialization breadth

### Bridge Summaries Must Carry Performance-Critical Delta Evidence

Milestone 5 must make explicit that bridge summaries are part of the
performance architecture, not only the correctness architecture.

For admitted incremental families, bridge-facing summaries must expose enough
delta structure to avoid broad rereads for ordinary live changes.

Representative required before/after evidence includes:

- old/new ordering keys
- old/new membership truth
- old/new materialization scope membership
- old/new derived-field inputs when those affect visible output

If that evidence is missing, the family is either:

- not incrementally maintainable yet
- refresh-admitted only for that change class
- out of scope for this milestone

### Performance Status Must Be Explicit

Every admitted live family should carry an explicit performance status such as:

- `Verified`
- `Debt`
- `Forbidden`

Rules:

- a family may claim `Verified` only when its live complexity contract,
  counters, and certification rows all exist
- a family marked `Debt` must name exactly what is still unproven
- a family marked `Forbidden` must fail typed and early rather than degrading
  into a broad path
- performance status must be attached to the admitted live family or live plan
  artifact, not merely recorded in diagnostics prose

### Performance Status And Eligibility Should Be Type-Carried

The spec should make it hard for implementation to treat performance posture as
optional metadata.

Representative shape:

- `LiveQueryPlan<VerifiedPerformance>`
- `LiveQueryPlan<DebtPerformance>`
- `LiveQueryPlan<ForbiddenPerformance>` only as an internal planning rejection
  state, never as an executable public type

The exact names may change, but the architectural rule should not:

- executable live plans must carry explicit performance posture
- non-executable or non-incremental families should fail before they can become
  ordinary executable live plan values
- performance posture should participate in family admission and certification,
  not only in logging

## Required Internal Subsystems

- `live_promotion/`
  promotion request handling, live-plan construction, and subscription identity
- `live_relevance/`
  query relevance contracts, bridge-summary matching, and suppression
  classification
- `live_patches/`
  detail, collection, materialization, and CDC-family patch construction
- `live_refresh/`
  typed full re-execution fallback and refresh-equivalence reporting
- `live_replay/`
  replay bundles, convergence harnesses, and deterministic patch traces
- `live_performance/`
  complexity contracts, cost classes, width budgets, coalescing classes, and
  performance-status proof surfaces
- `diagnostics/live/`
  counters, reports, suppression logs, and failure digests
- `harness/live_certification/`
  milestone-native acceptance rows and bundle completeness reporting

Boundary rules:

- `live_promotion/` must not revalidate schema legality or replanning semantics
- `live_relevance/` must not become a second bridge implementation
- `live_patches/` must not expose raw CDC or host-specific delivery bags
- `live_refresh/` must not silently replace incremental maintenance without
  typed visibility
- `live_performance/` must not become a bag of counters detached from plan
  semantics; it owns performance contracts, not generic telemetry
- `diagnostics/live/` must not become semantic authority
- `harness/live_certification/` must exercise production live lowering and
  patch logic rather than alternate shadow implementations

## Phases

### Phase 1: Freeze Live Promotion Authority And Artifact Families

Phase 1 exists to define what counts as one live subscription and what proof it
must carry.

Milestone 5 must first define:

- `LivePromotionRequest`
- `LiveQueryPlan`
- `LiveSubscriptionIdentity`
- `LiveStartBasis`
- `LiveProgressBasis`
- `QueryRelevanceContract`
- `SuppressionDecision`
- `PatchEnvelopeFamily`
- `LiveReplayBundle`
- `LiveSubscriptionCounters`
- `LiveChangeSequenceId`
- `LiveChangeOrdinal`
- `RefreshAdmissionMatrix`
- `PatchFamilyMarker<TPlanFamily>`
- `LiveMaintenanceComplexityContract`
- `LiveMaintenanceCostClass`
- `IncrementalPatchEligibility`
- `PatchWidthPolicy`
- `PatchWidthBudget`
- `CoalescingAdmissionClass`
- `RefreshCostClass`
- `RefreshAdmissionStatus`

This phase leaves the system in a coherent state where:

- live promotion is a query-owned proof boundary instead of a runtime helper
- one-shot plans are the only admitted promotion input
- basis identity and progress identity are explicit
- change-sequence identity is explicit
- performance policy is explicit
- raw CDC and host patch bags are structurally out of spec

### Phase 2: Lower One-Shot Plans Into Live Plans And Relevance Contracts

Phase 2 exists to keep live mode from becoming a parallel planner.

Milestone 5 must then implement:

- lowering from `ExecutionPlanBundle` and `CollectionPlanBundle` into
  `LiveQueryPlan`
- derivation of `QueryRelevanceContract` from projection, ordering, traversal,
  derived-field, and result-family semantics
- explicit classification of which admitted query families are live-promotable
- explicit classification of which suppressible change classes are admitted per
  live family
- explicit classification of which refresh fallback classes are admitted per
  live family
- explicit complexity contracts and cost classes per admitted live family
- explicit incremental eligibility proofs per admitted change class
- explicit performance status attachment per admitted live family
- typed rejection for non-live-promotable families or unsupported
  result-family/live combinations

This phase leaves the system in a coherent state where:

- the same one-shot plan yields the same live plan every time
- live relevance is derived from query semantics instead of subscriber glue
- admitted and non-admitted live families are explicit and typed

### Phase 3: Bridge-Aware Invalidation And Suppression Routing

Phase 3 exists to make invalidation routing explicit without stealing bridge or
signal ownership.

Milestone 5 must then implement:

- bridge-facing invalidation descriptors or materially equivalent query-facing
  adapters
- classification of relevant, irrelevant, suppressible, and refresh-required
  change classes
- typed suppression rules for irrelevant and admitted low-signal changes
- deterministic ordering-change and membership-change classification from
  bridge summaries rather than host repair
- exact admission of coalescing classes and patch-width policies
- exact counters for invalidation breadth, suppressed updates, and refresh
  fallback pressure
- exact family-owned width-budget units for detail, collection, and
  materialization live families

This phase leaves the system in a coherent state where:

- changes can be matched to query semantics deterministically
- irrelevance is explicit and counted
- suppression is no longer transport folklore

### Phase 4: Query-Shaped Patch Construction And Fallback Discipline

Phase 4 exists to make incremental maintenance honest at the consumer boundary.

Milestone 5 must then implement:

- query-shaped patch envelopes for admitted detail, ordered collection, and
  bounded materialization families
- patch-digest and delivery-digest rules
- type-coupled patch-family constructors tied to promoted plan family
- width-budget enforcement and typed overflow handling per admitted live family
- typed fallback refresh envelopes where incremental maintenance is not the
  admitted path for a relevant change class
- zero raw-CDC leakage across the consumer-facing live contract

This phase leaves the system in a coherent state where:

- incremental maintenance has a closed patch vocabulary
- fallback is explicit and equivalent rather than silent and broad
- clients can consume live query meaning directly without reconstructing it

### Phase 5: Live Replay, Convergence, And Certification Bundles

Phase 5 exists to prove that live mode survives time and replay instead of
looking correct only in one process run.

Milestone 5 must then implement:

- replay of admitted truth-change sequences into live subscriptions
- convergence comparison against fresh query re-execution at the same progress
  basis
- canonical replay bundles carrying `query_digest`, `result_digest`,
  `delivery_digest`, `replay_digest`, and `counter_snapshot`
- deterministic suppression and fallback reporting
- replay over admitted coalesced and uncoalesced change sequences where both
  claim the same semantic sequence identity
- replay proof that cost-honest coalescing preserves semantic patch outcome

This phase leaves the system in a coherent state where:

- live end state is mechanically comparable to fresh execution
- patch evolution is replay-safe and testable
- certification can operate from bundle artifacts instead of narrative logs

### Phase 6: Certification, Counter Proof, And Boundary Hardening

Phase 6 exists to close the milestone through named proof rather than feature
demo confidence.

Milestone 5 must finally ship:

- the `Live Promotion Convergence And Suppression Test`
- canonical rows for admitted detail, ordered collection, and bounded
  materialization live lanes
- rejection rows for unsupported live families, unsupported patch families, and
  forbidden raw-CDC leakage
- rejection rows for forbidden refresh escape hatches and non-monotonic change
  sequence identity
- rejection rows for forbidden width-budget overflow behavior and forbidden
  coalescing classes
- compile-fail or privacy hardening proving that live artifacts cannot be
  WORTHd or constructed from raw change records externally

This phase leaves the system in a coherent state where:

- live promotion is certifiable rather than plausible
- Milestone 6 can extend basis classes without redefining live semantics
- Milestone 8 view-shape work can inherit a real live substrate instead of
  cosmetic refresh behavior

## Must Ship

- proof-bearing `LiveQueryPlan`, `LiveSubscriptionIdentity`,
  `LivePatchEnvelope`, and `LiveReplayBundle` families or materially equivalent
  types
- live-promotion lowering from admitted one-shot plans into live plans
- query-to-signal relevance metadata sufficient for admitted incremental
  maintenance
- bridge-aware change summary matching for admitted live families
- explicit live complexity contracts, cost classes, and performance statuses for
  admitted live families
- explicit patch-width policies, budgets, and coalescing-admission classes for
  admitted live families
- one dedicated performance subdomain owning cost contracts rather than generic
  telemetry-only reporting
- query-shaped patch families for:
  - detail reads
  - ordered collections
  - bounded materialized relations
- explicit suppression and typed refresh fallback policies
- typed live diagnostics, replay bundles, and exact counters
- milestone-native certification proving convergence, suppression, parity, and
  rejection behavior

## Must Preserve

- canonical query meaning from Milestone 1 remains authoritative
- validation legality from Milestone 2 remains authoritative
- one-shot planning and basis identity from Milestone 3 remain authoritative
- collection semantics from Milestone 4 remain authoritative
- signal scheduling remains owned by `worth-signal`
- patch routing remains owned by the runtime bridge
- consumers receive query-shaped deltas rather than raw CDC
- suppression never changes final query truth; it only changes visible update
  cadence
- any admitted full refresh fallback remains explicit, typed, and
  query-equivalent
- patch-family identity remains type-coupled to the promoted plan family
- bridge coalescing cannot change semantic patch outcome for admitted sequence
  identities
- performance claims are attached to named contracts rather than implementation
  folklore

## Complexity / Proof Obligations

Milestone 5 must name costs and proofs in terms of:

- invalidation breadth
- relevance-match width
- patch width
- suppressed update count
- refresh fallback count
- replayed change count
- delivery-family width
- width-budget overflow pressure
- coalescing pressure
- work avoided through suppression and stable ordering/materialization proofs

Minimum required counters:

- `live_invalidation_event_count`
- `live_relevance_match_count`
- `live_irrelevant_suppression_count`
- `live_threshold_suppression_count`
- `live_patch_count`
- `live_patch_field_delta_count`
- `live_collection_membership_change_count`
- `live_collection_reorder_count`
- `live_materialization_patch_count`
- `live_refresh_fallback_count`
- `live_refresh_denial_count`
- `live_replay_change_count`
- `live_change_sequence_gap_count`
- `live_coalesced_change_bundle_count`
- `live_delivery_width`
- `live_patch_width_overflow_count`
- `live_coalescing_denial_count`
- `live_refresh_cost_class_count`
- `live_work_avoided_by_irrelevance_count`
- `live_work_avoided_by_stable_ordering_count`
- `live_work_avoided_by_scope_proof_count`
- `live_executor_rediscovery_count`

Rules:

- counters belong to live patch envelopes and replay bundles
- representative certification scenarios must assert exact counts
- `live_executor_rediscovery_count` must be exactly zero on every admitted path
- any admitted refresh fallback must be counted separately from successful
  incremental patch construction
- any denied refresh escape hatch must increment `live_refresh_denial_count`
- any non-monotonic or gapful admitted sequence must be rejected and counted via
  `live_change_sequence_gap_count`
- any denied coalescing request must increment `live_coalescing_denial_count`
- width-budget overflow must resolve only through explicit `PatchWidthPolicy`
  outcomes and increment `live_patch_width_overflow_count`
- no supported path may hide broad re-execution inside generic live patch
  counts
- "work avoided" counters must exist so suppression and narrowing remain
  mechanically visible rather than anecdotal

Minimum certification rows should include:

- `detail-live-convergence`
- `ordered-collection-live-convergence`
- `bounded-materialization-live-convergence`
- `irrelevant-update-suppression`
- `threshold-suppression-parity` where admitted
- `refresh-fallback-equivalence`
- `coalesced-sequence-replay-parity`
- `patch-width-budget-overflow-policy`
- `work-avoided-counter-parity`

Minimum rejection rows should include:

- `unsupported-live-family`
- `unsupported-patch-family`
- `raw-cdc-leakage-forbidden`
- `invalid-live-basis-promotion`
- `forbidden-refresh-escape-hatch`
- `non-monotonic-change-sequence`
- `forbidden-coalescing-class`
- `forbidden-width-budget-overflow-behavior`

## Allowed Debt

- some query families may remain non-live-promotable as explicit `Debt` while
  admitted families are fully parity-proven
- richer suppression policies may remain `Debt` if admitted suppression
  semantics are explicit, deterministic, and certified
- broader refresh-admitted classes may remain `Debt` if the shipped refresh
  matrix is closed, narrow, and certified
- broader cost-class coverage may remain `Debt` if admitted live families
  already carry named complexity contracts and explicit performance status
- durable restart-stable live continuation may remain blocked on
  `worth-store`
- raw CDC delivery disguised as query-shaped maintenance may not exist as debt
- host-side collection repair, hidden re-sorts, or hidden refresh fallback may
  not exist as debt

## Acceptance Evidence

Milestone 5 is complete only when `worth-query` can prove:

- the `Live Promotion Convergence And Suppression Test` in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/test-requirements.md)
  passes with canonical machine-checkable artifacts
- the same query expression can execute as one-shot or live without semantic
  drift
- irrelevant truth changes are suppressed before query-shaped patch delivery
- query-shaped live patches preserve ordering, membership, projection, and
  bounded materialization semantics
- replaying the same canonical truth changes yields the same live query result
  evolution
- any admitted full refresh fallback remains explicitly visible and
  query-equivalent to fresh execution
- non-monotonic or semantically incomplete change sequences fail explicitly
  rather than drifting or guessing
- admitted performance claims are backed by named counters, overflow policy, and
  coalescing policy rather than implementation convention

Required verification output must include:

- `query_digest`
- `result_digest`
- `delivery_digest`
- `replay_digest`
- `counter_snapshot`

## Architectural Notes

### Law 41 Is Still The Load-Bearing Rule

The most important hardening rule remains that the type must encode what has
been proven.

That means:

- one-shot plans are not live plans
- live plans are not live patch envelopes
- live patch envelopes are not replay proofs
- bridge change summaries are not consumer patch artifacts
- suppression decisions are not transport hints

If a public constructor can mint those stronger artifacts without the proving
path, the milestone is structurally wrong even if demos work.

### Live Must Stay Query-Shaped, Not Event-Shaped

Milestone 5 should be smart only in ways that preserve semantic honesty:

- reuse the one-shot planner's meaning
- derive relevance from explicit query dependencies
- emit refresh fallback when incremental maintenance cannot stay honest

It must not be smart in these ways:

- infer membership or ordering changes from consumer-local caches
- reinterpret raw CDC as if that were already the query patch
- hide broad recomputation behind "incremental" naming
- use refresh as the default answer whenever patch construction feels hard

### Suppression Is A Semantic Boundary

Suppression is not merely a bandwidth optimization. It changes when updates are
visible, so it must be modeled as explicit query-owned policy.

The required rule is:

- suppression may defer or collapse visible patch delivery
- suppression may not change the final query truth at a declared progress basis

Any system that cannot prove that distinction does not have honest suppression.

### Refresh Fallback Must Stay Narrow

Refresh fallback is necessary, but it is also the easiest place to smuggle in a
fake incremental system.

Milestone 5 must therefore make these rules explicit:

- refresh is admitted only for closed named change classes
- refresh denial is a typed outcome, not a comment or TODO
- admitted detail and ordered-collection families should default to incremental
  patch construction for ordinary projection, membership, and ordering changes
- if a family depends on refresh for its ordinary update path, that family is
  not honestly live-promotable yet

### Bridge Summaries Must Be Sufficient For Incremental Truth

The live design fails if the bridge only provides "something changed nearby"
and forces `worth-query` to rediscover semantics through broad rereads.

For admitted incremental families, bridge-facing summaries must be rich enough
to let the query layer classify:

- relevance versus irrelevance
- membership change versus payload-only change
- ordering move versus stable replacement
- in-scope versus out-of-scope materialization changes

If the bridge cannot supply that summary honestly for a family, the family must
either stay non-live-promotable or route through an explicit refresh-admitted
class.

## Sequencing Notes

Milestone 5 belongs immediately after Milestone 4 because live maintenance
must inherit already-frozen ordering, cursor, traversal, rollup, derived-field,
and CDC-family semantics rather than inventing them from change events.

It must land before Milestone 6 because historical and diff work need a real
proof that query meaning survives time, not just one-shot execution.

It must land before Milestone 8 because view-shape-specific live behavior
depends on an existing live patch substrate instead of cosmetic refresh.

## Parallelization Notes

Once the live-plan and relevance-contract boundary is frozen:

- early Milestone 6 basis expansion can proceed in parallel without redefining
  live patch meaning
- bridge-side change-summary refinement can proceed in parallel so long as it
  continues to feed the same query-facing invalidation contract
- broader live-family experiments can proceed behind explicit debt markers
  without weakening admitted families
- compile-time tightening of patch-family markers and live sequence identity can
  proceed in parallel without changing milestone semantics

## Explicit Failure Taxonomy For Milestone 5

- unsupported live query family
- unsupported result-family/live combination
- invalid live basis promotion
- unsupported suppression policy
- unsupported patch family
- bridge change-summary incompatibility
- relevance contract invariant break
- forbidden raw-CDC leakage
- refresh fallback denied
- forbidden refresh escape hatch
- non-monotonic live change sequence
- live replay divergence
- live convergence failure
- live artifact invariant break

## Anti-Patterns Explicitly Rejected

- separate live-query builders or ASTs
- raw CDC as the consumer-facing live contract
- host-side row re-sorting or membership repair
- hidden full refreshes labeled as incremental maintenance
- broad "change happened, just refresh" bridge summaries presented as
  incremental support
- subscriber-local relevance heuristics
- one subscriptions mega-module mixing promotion, signal glue, bridge routing,
  patch construction, replay, and diagnostics
- public construction of live proof types without the proving path
- suppression implemented as an untracked side effect of batching

## Self-Check

This milestone solves a real structural problem rather than packaging work
cosmetically because it introduces the first proof-bearing live-execution
boundary between one-shot query semantics and long-lived maintained semantics.

The adversarial constraint is load-bearing because it forbids the naive failure
mode where long-lived subscriptions drift from fresh truth or depend on raw
event interpretation by consumers.

The milestone preserves authority boundaries because `worth-query` owns live
promotion and query-shaped patch meaning, `worth-signal` still owns scheduling,
the runtime bridge still owns routing, and host delivery code remains a
consumer rather than a semantic co-author.

The milestone defines proof obligations rather than implementation chores
because convergence, suppression, replay parity, refresh fallback honesty, and
exact invalidation/patch counters are required for closeout.

A competent engineer should be able to map this spec into honest live-plan,
relevance, patch, replay, and certification modules without inventing the
architecture during implementation.

This milestone belongs fifth in the roadmap because it is the first point
where query meaning must survive time under churn, after one-shot and
collection semantics already exist and before historical or view-shape work
tries to build on top.

## Closeout Standard

Milestone 5 is complete only when all of the following are true:

- admitted one-shot query families can be promoted to live mode without a
  second query language
- live plans are derived from existing planned query meaning rather than
  subscriber-local semantics
- relevant and irrelevant changes are classified explicitly and counted
- admitted live patches are query-shaped, result-family-shaped, and
  consumer-ready
- suppression is explicit, deterministic, and does not change final query
  truth
- any admitted refresh fallback remains typed and query-equivalent
- replay proves convergence against fresh execution with canonical
  machine-checkable artifacts

If code lands but live maintenance still depends on raw CDC, host repair,
hidden refresh, or non-sealed live proof types, Milestone 5 is not complete.
