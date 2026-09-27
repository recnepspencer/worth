# Milestone 5.1 Engineering Spec: Region-Scoped Live Narrowing And Stream-Contract Delivery

> **Status:** Draft engineering spec
>
> **Roadmap parent:** [worth_query_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/worth_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/worth_query_vision.md)
>
> **Prior milestone:** [milestone-5.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/milestone-5.md)
>
> **Prior closeout:** [milestone-5-closeout.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/milestone-5-closeout.md)
>
> **Test requirements:** [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/test-requirements.md)
>
> **Primary architectural driver:** make region-aware invalidation and stream-contract lowering planner-owned live artifacts so locality-bearing truth changes can narrow below broad aspect scope and still arrive as query-shaped delivery contracts rather than raw partition events or transport-local glue
>
> **Companion docs:**
> - [MENTALITY.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/docs/coding-guidelines/MENTALITY.md)
> - [arch_laws.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/docs/coding-guidelines/arch_laws.md)
> - [perf_laws.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/docs/coding-guidelines/perf_laws.md)
> - [domain_laws.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/docs/coding-guidelines/domain_laws.md)
> - [worth_query_vision.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/worth_query_vision.md)
> - [worth_query_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/worth_query_roadmap.md)
> - [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/test-requirements.md)
> - [milestone-4.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/milestone-4.md)
> - [milestone-4-closeout.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/milestone-4-closeout.md)
> - [milestone-5.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/milestone-5.md)
> - [milestone-5-closeout.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/milestone-5-closeout.md)
> - [worth-runtime-bridge milestone-2.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-2.md)
> - [worth-runtime-bridge milestone-6.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-6.md)

## Goal

Make live query narrowing locality-aware and stream-contract-honest so admitted
live queries can classify region- and partition-scoped truth changes below
broad aspect invalidation, suppress off-region churn before delivery, and lower
query-shaped live output into formal bridge stream contracts without changing
canonical query meaning.

## Why This Milestone Exists

Milestone 5 closed the first honest live substrate for `worth-query`. It
proved that one-shot query meaning can promote into live mode, stay
query-shaped under churn, and converge against fresh re-execution.

That is necessary, but not yet production-grade for the workloads `worth-query`
claims to serve.

Geometry, CAD, chip, and integration-facing workloads care about more than
"this aspect changed somewhere nearby." They care that:

- a topology-region query only reacts to the touched region, not the whole
  assembly
- a partition-local collection does not widen into cross-partition refresh
  because the change stream lacked locality vocabulary
- a query-shaped live delivery can be lowered into a formal stream contract
  instead of being wrapped in transport-local glue
- unsupported locality or stream combinations fail explicitly instead of
  degrading into broad invalidation dressed up as fine-grained live support

Milestone 5.1 therefore exists to harden live mode at the next structural seam:

- locality must become a planner-owned live contract, not an incidental bridge
  detail
- region-aware suppression must be explicit and counter-proven, not a side
  effect of broad relevance filtering
- query delivery must be able to lower into bridge change-stream contracts
  without collapsing query meaning into raw partition events
- widening beyond the declared locality contract must be typed, counted, and
  denied unless the plan explicitly admits it

If this milestone is skipped, `worth-query` would still have an honest
live/subscription story at the broad aspect level, but it would not yet have
the locality-honest live substrate needed for region-bearing product surfaces
or stream-backed integration delivery.

## Governing Summaries

- `MENTALITY.md`: the hard problem is not adding "finer filters" to live mode.
  It is surviving locality-bearing churn without broadening recomputation or
  drifting into transport-defined semantics. The milestone must solve that
  structural narrowing problem first.
- `arch_laws.md`: Laws 2, 7, 8, 17, 21, 22, 26, 27, 30, 32, 33, 35, 40, and 41
  dominate this milestone. Locality contracts must be declared before
  execution, query delivery and stream delivery must remain separate but
  aligned, and proof-bearing live types must encode exactly which narrowing and
  stream-admission facts have been proven.
- `perf_laws.md`: locality only matters if it is mechanically visible. Region
  matches, off-region suppressions, widening denials, and stream-lowering
  breadth must be named counters with exact proof rows rather than optimistic
  performance claims.
- `domain_laws.md`: locality planning, invalidation classification, stream
  contract lowering, replay bundles, and diagnostics are separate
  responsibilities and must not collapse into one "live delivery" module.
- `worth_query_vision.md`: live promotion, query-to-signal bridging,
  CDC-shaped output, subgraph-scoped reads, and policy-aware/delivery-aware
  shaping all depend on the query layer owning structured narrowing and result
  contracts rather than raw runtime deltas.
- `worth_query_roadmap.md`: Milestone 5.1 belongs immediately after Milestone 5
  because it is live-maintenance hardening. It must prove region-scoped
  invalidation narrowing and change-stream-backed delivery contracts without
  reopening historical, preview, policy, or durability semantics.
- `test-requirements.md`: the `Region-Scoped Live Narrowing And Stream Contract
  Test` is the closeout proof. It requires narrower-than-broad invalidation
  where lower runtimes admit locality, early typed rejection for unsupported
  region/stream combinations, and parity-safe stream-backed delivery for the
  same canonical live query meaning.
- `milestone-4.md`: bounded traversal/materialization and CDC-shaped result
  families already froze breadth and delivery-shape semantics on one-shot
  paths. Milestone 5.1 must inherit those planner-owned boundaries instead of
  inventing a second locality story in live mode.
- `milestone-4-closeout.md`: runtime-backed collection semantics, CDC-shaped
  result-family lowering, and collection certification rows already exist.
  Milestone 5.1 should reuse that digest-bearing collection/result-family
  substrate when locality-sensitive live queries operate over ordered
  collections or bounded materialization.
- `milestone-5.md`: live promotion, query-shaped patches, suppression, replay,
  and refresh/coalescing policies are already query-owned. Milestone 5.1 must
  extend those live proofs with locality metadata rather than bypass them with
  bridge-local routing shortcuts.
- `milestone-5-closeout.md`: the runtime-backed live substrate is already
  closed narrowly and honestly for admitted families. Region-aware narrowing
  and stream-contract delivery must now build on that substrate without
  weakening Milestone 5's no-raw-CDC and no-hidden-refresh guarantees.
- `worth-runtime-bridge/milestone-2.md`: the bridge already treats field,
  lens, region, partition, and facet slices as canonical routing categories.
  Query locality must consume that admitted slice vocabulary rather than invent
  host-local region semantics.
- `worth-runtime-bridge/milestone-6.md`: change-stream declaration, stream
  member identity, stream window identity, consumer-contract admission, and
  replay-safe stream vocabulary are bridge-owned protocol responsibilities.
  Query delivery may lower into those contracts, but it must not redefine them
  or pretend transport offsets are canonical stream truth.

## Adversarial Constraint

Milestone 5.1 must survive the following hostile condition:

> An admitted live query with explicit locality-bearing scope is promoted from
> one explicit start basis, receives a long stream of truth changes where some
> changes hit the query's declared region or partition and many others do not,
> and must maintain the same query-shaped live meaning while lowering admitted
> delivery output into formal stream contracts without widening to broad aspect
> invalidation, raw partition events, or transport-local consumer glue.

Concretely, the design must remain correct when all of the following are true:

- the same live-promotable query is evaluated once with broad aspect-level
  control surfaces and once with admitted region- or partition-scoped locality
  surfaces
- some truth changes affect the same aspects but different regions/partitions
  than the query declared
- some changes require stream-contract admission for delivery, while others are
  locality-valid for live maintenance but not stream-contract-admissible
- ordered collection and bounded materialization families rely on locality
  evidence to avoid full-scope membership or traversal recomputation
- bridge slice identity and stream window identity are explicit lower-runtime
  contracts that `worth-query` must consume rather than reinterpret
- multiple delivery shapes observe the same live query meaning and must remain
  query-shaped even when their stream-consumer contracts differ

If any supported path:

- widens a locality-bearing change into broad aspect invalidation when the
  lower runtimes admitted narrower routing
- treats raw partition or stream members as though they were already the query
  delivery contract
- lets host transport or consumer adapters decide whether a change stayed
  in-region
- hides widening or stream-admission failure behind generic refresh behavior
- changes query meaning depending on whether the consumer asked for direct live
  patches or stream-contract-backed delivery
- cannot replay the same locality-bearing change history into the same query
  result and delivery evolution

then Milestone 5.1 has failed.

## Product Decision Lock

- region- and partition-scoped narrowing are extensions of the Milestone 5 live
  plan, not alternate subscription builders, runtime-only heuristics, or host
  filter callbacks
- locality-bearing query semantics are declared by query/collection/live plans
  and matched against lower-runtime locality contracts; they are not inferred
  from transport topics, host paths, or consumer names
- the runtime bridge remains the authority for canonical slice identity and
  change-stream contract vocabulary; `worth-query` may lower into bridge
  contracts but may not redefine stream member or stream window truth
- query delivery remains query-shaped even when it is emitted through a formal
  stream contract; raw partition events and raw CDC records are never the
  consumer-facing query contract
- region widening is not a convenience fallback. It is either:
  - an explicitly admitted planner-owned widening class with typed evidence, or
  - a typed denial with exact counters and diagnostics
- stream-contract lowering is a delivery-contract transformation over canonical
  live query meaning, not a second execution semantics path
- durable stream continuation, persisted checkpoints, restart-stable stream
  resume, and portable stream artifacts remain out of scope for this milestone
  even if the bridge already names those concepts

Normative consequence:

- any implementation path that routes locality-sensitive live queries through
  broad aspect invalidation by default is out of spec
- any implementation path that exposes bridge stream members as though they
  were already query-shaped delivery payloads is out of spec
- any implementation path that lets host adapters decide locality or stream
  contract admission is out of spec
- any implementation path that silently converts locality mismatch into broad
  refresh or broad delivery is out of spec
- any implementation path that implies durable continuation because it uses
  stream vocabulary is out of spec

## Compile-Time Enforcement Policy

Milestone 5.1 must classify which locality and stream guarantees become
unrepresentable, uncompilable, or construction-time rejection.

`Unrepresentable` in public types:

- publicly constructible locality-bearing live plans that do not carry source
  plan identity, locality contract identity, and widening policy identity
- publicly constructible region-aware live patch or delivery envelopes that do
  not carry query result-family identity plus locality/progress evidence
- publicly constructible stream-lowered delivery artifacts that do not carry
  both query delivery identity and admitted bridge contract identity
- publicly constructible locality predicates or stream-consumer declarations as
  open-ended bags instead of closed query-owned or bridge-owned families

`Uncompilable` through visibility and compile-fail enforcement:

- external construction of `RegionScopedLivePlan`,
  `LocalityNarrowingDecision`, `StreamLoweredDeliveryContract`,
  `RegionScopedReplayBundle`, or materially equivalent proof-bearing types
  without crate-owned lowering
- public APIs that accept raw bridge slice events, raw partition events, raw
  stream members, or host-authored locality bags as though they were admitted
  query maintenance input
- public conversion paths that bypass Milestone 5 live plans and mint
  locality-bearing or stream-bearing artifacts directly from raw query inputs
- public APIs that let consumers claim "in-region," "widened," or
  "stream-admitted" outcomes without the proving path

`Construction-time rejection`:

- non-locality-admitted live families requested for region-sensitive
  maintenance
- unsupported locality predicates or unsupported region/partition scope mixes
- unsupported query-result-family/stream-contract combinations
- stream consumer shapes that cannot preserve the canonical query delivery
  contract for the admitted live family
- invalid widening requests or invalid locality fallback requests
- invalid lower-runtime locality/stream capability pairings

Rules:

- the strongest available boundary must be used
- locality-bearing live plan and stream-lowered delivery types must use sealed
  constructors and private fields
- compile-fail coverage is required for:
  - no raw stream member as query delivery input
  - no external construction of locality-bearing live artifacts
  - no direct external construction of widening/admission outcomes
- runtime rejection is allowed only for facts genuinely unavailable until the
  lower runtime reports locality slice or stream-consumer compatibility

## Scope

### In Scope

- region- or partition-aware extensions to admitted Milestone 5 live query
  families
- planner-owned locality predicates, locality contracts, and locality-bearing
  live metadata
- bridge-facing locality/slice matching sufficient for region-aware relevance
  classification
- typed narrowing decisions for:
  - in-region relevance
  - off-region suppression
  - explicit widening denial
  - explicit admitted widening where the plan allows it
- stream-contract lowering for admitted query-shaped live delivery and admitted
  CDC/live output families
- typed stream-lowered delivery envelopes, diagnostics, counters, and replay
  bundles
- replay and parity proof comparing:
  - region-aware live maintenance
  - broad aspect control execution
  - fresh query re-execution at the same progress basis
- milestone-native certification for region narrowing, stream admission, and
  rejection behavior

### Explicitly Out Of Scope

- preview-session basis, speculative branch workflow, or promotion semantics
- historical, diff, lineage, or correspondence basis variation
- policy masking as a legality source, though the locality and delivery
  artifacts must preserve clean seams for later policy-aware live work
- durable stream resume, persisted checkpoints, replay after process restart,
  or artifact portability
- store-backed live parity or store-backed stream continuation
- arbitrary user-authored region languages or transport-authored topic routing
- network framing, broker configuration, or host transport lifecycle

### Initial Admission Matrix

Milestone 5.1 must not leave "admitted family" decisions ambient.

Initial locality-admitted live query families:

- detail live queries whose locality predicate lowers to one bridge-admitted
  `entity region surface` or `entity partition surface`
- ordered collection live queries whose root collection scope is already
  planner-bounded and whose locality predicate lowers to one admitted region or
  partition scope on the root entity set
- bounded materialization live queries whose locality-sensitive descendants are
  already bounded by Milestone 4 traversal/materialization contracts and whose
  locality predicate is rooted in the same bounded scope

Initial locality-denied live query families:

- aggregate/rollup families that cannot yet prove locality-preserving input
  maintenance
- unbounded traversal/materialization families
- any family whose locality meaning would require host-authored graph walking,
  callback filtering, or transport-topic interpretation

Initial bridge slice categories admitted at the query seam:

- `entity region surface`
- `entity partition surface`
- explicitly registered coarse fallback slice only as a typed denial or
  non-locality control lane, never as successful region-scoped admission

Slice categories not admitted as successful Milestone 5.1 locality proof:

- field-only slices pretending to prove region scope
- host-defined slice categories
- transport-defined topic or channel categories

Initial stream-lowered delivery families:

- CDC-shaped collection delivery whose collection/result-family meaning is
  already planner-owned from Milestone 4 and live-owned from Milestone 5
- locality-sensitive detail delivery only if the emitted stream member still
  preserves one entity/detail result-family identity without host re-shaping

Initial stream-lowering denials:

- ordered collection splice families that would require transport-local member
  reconstruction
- locality-sensitive delivery families whose stream member would have to carry
  raw partition events instead of query-owned payload identity
- any consumer contract that cannot preserve canonical query delivery ordering
  and locality-suppression truth

Any family not named above is out of scope for Milestone 5.1 and must fail
typed and early rather than entering implicit beta support.

### Initial Performance Posture Matrix

Milestone 5.1 must also freeze the first admitted cost postures rather than
letting execution choose between "narrow" and "broad" dynamically without plan
identity.

Initial admitted cost postures:

- detail + region scope:
  locality match cost is bounded by one admitted region-slice match plus one
  detail payload update or typed suppression
- detail + partition scope:
  locality match cost is bounded by one admitted partition-slice match plus one
  detail payload update or typed suppression
- ordered collection + partition scope:
  locality match cost is bounded by one partition-slice match plus one
  membership/order maintenance decision over the already admitted root
  collection plan
- bounded materialization + region scope:
  locality match cost is bounded by one region-slice match plus one bounded
  descendant maintenance path inside the already admitted traversal breadth

Initial forbidden cost postures:

- locality-sensitive maintenance whose cost depends on broad collection re-scan
  as the ordinary successful lane
- locality-sensitive maintenance whose cost depends on unbounded descendant
  rediscovery
- stream lowering whose ordinary successful lane depends on reconstructing
  stream members from host transport state
- any admitted path whose broad control fallback is cheaper to implement and
  therefore silently chosen despite a locality-admitted plan

If the architecture cannot state the successful-path cost posture for an
admitted family, that family is not ready to ship.

## Region-Scoped Live Architecture

### One Locality-Honest Extension Of Milestone 5

Milestone 5.1 extends the existing live proof chain. It must not create a
second live substrate for locality-sensitive queries and it must not create a
parallel delivery semantics path for stream-backed consumers.

The authoritative flow becomes:

`ValidatedQueryBundle`
-> `ExecutionPlanBundle`
-> `CollectionPlanBundle` where applicable
-> `LiveQueryPlan`
-> `RegionScopedLivePlan`
-> `RegionScopedLiveSubscriptionState`
-> `RegionScopedLivePatchEnvelope`
-> `StreamLoweredDeliveryContract` where admitted
-> `RegionScopedReplayBundle`

Region narrowing therefore consumes already-proven query meaning and already-
proven live semantics. It does not re-author:

- projection meaning
- collection membership meaning
- ordering meaning
- traversal/materialization meaning
- suppression truth
- result-family meaning
- basis identity

It adds only the additional proofs that:

- locality scope is explicit
- the lower runtime admitted a narrower routing surface
- the observed change was in-region, off-region, or widening-sensitive
- the emitted delivery contract remained query-shaped while lowering into a
  bridge-owned stream declaration

### Proof Chain

Milestone 5.1 must make the locality proof chain as explicit as Milestone 5
made the live proof chain.

The required chain is:

`LiveQueryPlan`
-> `LocalityAdmittedLivePlan`
-> `BridgeSliceCompatibleChange`
-> `LocalityMatchedChange`
-> `RegionScopedLivePatchEnvelope`
-> `StreamLoweringAdmittedDelivery` where admitted
-> `RegionScopedReplayBundle`

Required invariants:

- only `LocalityAdmittedLivePlan` may enter locality-sensitive execution
- only `BridgeSliceCompatibleChange` may be evaluated for locality match
- `LocalityMatchedChange` must distinguish:
  - `InRegion`
  - `OffRegionSuppressed`
  - `WideningDenied`
  - `WideningAdmitted`
- only `RegionScopedLivePatchEnvelope` may lower into stream delivery
- only stream-lowering-admitted delivery families may produce
  `StreamLoweredDeliveryContract`
- performance posture must be fixed no later than `LocalityAdmittedLivePlan`;
  execution may consume cost posture, but may not invent it

Encoding rule:

- do not encode these states as one mutable bag plus booleans
- use sealed constructors, typestates, or sealed witness-bearing enums so
  later phases cannot "skip ahead" to a stronger proof state

### Authority Boundaries

`worth-query` owns:

- locality-bearing live plan lowering from Milestone 5 live plans
- query-declared locality predicates or materially equivalent plan-owned
  narrowing surfaces
- region-aware relevance, suppression, widening denial, and admitted widening
  classification
- query-shaped delivery-contract lowering for admitted stream-backed output
- replay/convergence artifacts proving locality and stream parity

The runtime bridge owns:

- canonical slice identity and fine-grained routing surfaces
- canonical change-stream declaration, consumer-contract, stream-member, and
  stream-window vocabulary
- routing and protocol interpretation between truth publications and admitted
  downstream consumer contracts

`worth-signal` owns:

- dependency tracking
- scheduling and invalidation timing mechanics
- downstream execution timing after admitted routing

Execution owns:

- consuming region-scoped live plans against admitted locality-bearing bridge
  summaries
- producing query-shaped region-scoped patch envelopes
- lowering admitted delivery output into stream contracts
- emitting typed denial or widening outcomes when locality or stream contracts
  cannot stay honest

Hosts and transport glue may own:

- transport of already-lowered stream contracts or live patch envelopes
- consumer lifecycle around admitted subscription handles
- presentation formatting that does not alter query meaning

Hosts and transport glue may not own:

- region/partition match classification
- widening decisions
- stream contract admission
- reinterpretation of raw bridge stream members into query payloads
- repair of off-region suppression or locality-based ordering behavior

### Locality And Stream Identity

Milestone 5.1 must preserve Milestone 5 basis/progress identity while adding
explicit locality and stream-contract identities.

Representative artifact families:

- `LocalityPredicateContract`
- `LocalityScopeDigest`
- `RegionScopedLivePlan`
- `RegionScopedSubscriptionIdentity`
- `RegionSliceMatch`
- `PartitionSliceMatch`
- `LocalityMatchClass`
- `LocalityWideningPolicy`
- `LocalityWideningDecision`
- `StreamContractRequest`
- `AdmittedStreamConsumerContract`
- `StreamLoweredDeliveryContract`
- `StreamContractDigest`
- `RegionScopedReplayDigest`
- `RegionScopedLiveCounters`

Rules:

- locality-bearing live identity includes the source query digest, plan digest,
  live subscription identity, and locality-scope digest
- locality equality is digest-bearing rather than "same callback/topic" host
  identity
- stream-lowered delivery identity must include both the query delivery digest
  and the admitted bridge consumer-contract digest
- the same canonical query meaning with a different locality predicate must
  produce distinct locality-scope identity
- the same locality-bearing query meaning with two different admitted stream
  consumer contracts may produce different delivery-contract digests while
  preserving the same underlying query result meaning
- no transport offset, topic name, broker cursor, or host channel id may serve
  as canonical locality or stream identity in this milestone

### Performance Architecture

Milestone 5.1 must encode performance as plan-owned architecture, not as
post-hoc observability.

Representative artifact families:

- `LocalityCostPosture`
- `LocalityMaintenanceClass`
- `LocalityBreadthBudget`
- `LocalityWideningBudget`
- `StreamLoweringCostPosture`
- `StreamMemberWidthBudget`
- `LocalityWorkClass`
- `LocalityPerformanceStatus`

Rules:

- every admitted locality-bearing live plan must carry one explicit
  `LocalityCostPosture`
- every admitted stream-lowered delivery family must carry one explicit
  `StreamLoweringCostPosture`
- locality-sensitive execution may not choose between narrow and broad paths by
  convenience once the plan is admitted; the allowed widening/fallback posture
  must already be encoded on the plan
- budgets must be structural and query-shaped:
  - region/partition match breadth
  - descendant maintenance breadth
  - delivery member width
  - admitted widening width
- if execution crosses a budget boundary, the outcome must be one of:
  - a typed widening-admitted outcome already declared by the plan
  - a typed denial
  - a typed refresh/fallback class already declared by the plan
- no admitted path may hide broad work inside generic "success" counters

### Locality Vocabulary Freeze

Milestone 5.1 must freeze what query-authored locality is allowed to mean.

Allowed query-side locality sources:

- a planner-owned bounded region scope rooted in an entity or collection plan
- a planner-owned partition scope rooted in the same admitted plan
- a bounded materialization scope whose locality contract is derived from the
  same root scope and whose descendant breadth is already explicit

Forbidden locality sources:

- host closures
- callback predicates
- ad hoc path grammars
- free-form strings
- transport topics or broker partitions
- field-only predicates used as a proxy for region truth

Normative rule:

- every locality predicate in this milestone must lower into one bridge-admitted
  canonical slice category
- if the predicate cannot lower into a bridge-admitted slice category, it is
  not an admitted Milestone 5.1 locality predicate

### Region Relevance, Narrowing, And Widening Denial

Milestone 5.1 must introduce one closed vocabulary for locality-aware live
relevance.

Representative artifact families:

- `LocalityAwareRelevanceContract`
- `LocalityRelevantChangeClass`
- `OffRegionSuppressionReason`
- `RegionScopedSuppressionDecision`
- `LocalityWideningDenial`
- `LocalityWideningAdmission`
- `BridgeSliceSummary`
- `BridgeRegionDescriptor`
- `BridgePartitionDescriptor`

Rules:

- locality-aware relevance is computed from:
  - planner-owned query semantics
  - planner-owned locality predicates
  - bridge-owned canonical slice summaries
- a change that is aspect-relevant but locality-irrelevant must be suppressible
  before visible patch construction
- a change that crosses the locality boundary in a way the admitted plan does
  not support must produce a typed widening denial or admitted widening
  outcome; it must not silently refresh broadly
- collection membership, ordering, traversal breadth, and derived-field
  visibility may depend on locality proofs for admitted families
- region-aware suppression may defer visible delivery, but it may not change
  the final query truth at the declared progress basis
- if the bridge cannot provide the locality evidence needed for an admitted
  family, the family must:
  - stay out of scope for region narrowing, or
  - route through an explicit typed denial path
- if locality-sensitive maintenance would exceed the admitted
  `LocalityBreadthBudget`, the outcome must be explicit and typed rather than
  silently broadening

### Query-Shaped Stream Contracts

Milestone 5.1 must define exactly how query-shaped live delivery lowers into
formal bridge stream contracts without changing semantics.

Representative artifact families:

- `QueryDeliveryContract`
- `DeliveryContractLowering`
- `StreamMemberProjection`
- `StreamWindowCompatibility`
- `StreamAdmissionFailure`
- `DeliveryContractReplayRecord`

Rules:

- query delivery is authoritative for result-family meaning
- bridge stream contracts are authoritative for stream declaration, stream
  member identity, stream window identity, and consumer protocol vocabulary
- stream lowering must happen after query delivery semantics are already fixed
- stream-lowered delivery must preserve:
  - query result-family identity
  - locality-suppression truth
  - ordering/membership/materialization truth
  - delivery digest parity for semantically equivalent lanes
- unsupported stream consumer shapes must fail before delivery-time drift
- stream lowering may change protocol-facing metadata, but it may not change
  the query result meaning or the explanation of why the query changed
- stream lowering must carry one explicit member-width and window-width budget;
  if the query-shaped payload cannot fit the admitted stream contract honestly,
  lowering must fail typed and early

## Phases

### Phase 1: Freeze Region And Stream Authority Surfaces

Phase 1 exists to make locality and stream-contract semantics explicit instead
of leaving them distributed across live code, bridge glue, and delivery
helpers.

Milestone 5.1 must introduce:

- `LocalityPredicateContract`
- `LocalityScopeDigest`
- `RegionScopedLivePlan`
- `RegionScopedSubscriptionIdentity`
- `LocalityAwareRelevanceContract`
- `RegionSliceMatch`
- `PartitionSliceMatch`
- `LocalityMatchClass`
- `LocalityWideningPolicy`
- `LocalityWideningDecision`
- `StreamContractRequest`
- `AdmittedStreamConsumerContract`
- `StreamLoweredDeliveryContract`
- `StreamContractDigest`
- `RegionScopedLiveCounters`
- `RegionScopedPlanningReport`
- `LocalityCostPosture`
- `LocalityMaintenanceClass`
- `LocalityBreadthBudget`
- `LocalityWideningBudget`
- `StreamLoweringCostPosture`
- `StreamMemberWidthBudget`
- `LocalityPerformanceStatus`

This phase leaves the system in a coherent state where:

- locality-bearing authority is no longer hidden inside generic live metadata
- query delivery identity and stream-contract identity are separate but linked
- widening is a named semantic outcome instead of implementation folklore

### Phase 2: Lower Live Plans Into Locality Contracts

Phase 2 exists to keep locality-sensitive live execution from becoming a
parallel planner.

Milestone 5.1 must then implement:

- lowering from `LiveQueryPlan` into `RegionScopedLivePlan`
- derivation of locality-aware relevance from projection, ordering,
  traversal/materialization, and admitted collection/result-family semantics
- explicit classification of which Milestone 5 live families are locality-
  admitted in Milestone 5.1
- explicit classification of admitted locality predicates and scope classes
- explicit classification of which delivery/result families are stream-
  lowerable in this milestone
- explicit attachment of cost posture, breadth budgets, and performance status
  for every admitted locality-bearing family
- typed rejection for unsupported locality family, unsupported locality scope,
  and unsupported stream-lowering combination
- typed rejection for any locality predicate that cannot lower into one bridge-
  admitted region/partition slice category

This phase leaves the system in a coherent state where:

- the same Milestone 5 live plan yields the same locality-bearing live plan
- locality is derived from query semantics instead of bridge/host guesswork
- admitted and denied locality/stream combinations are explicit and typed

### Phase 3: Admit Region-Scoped Invalidation And Widening Boundaries

Phase 3 exists to turn locality intent into explicit runtime match behavior.

Milestone 5.1 must then implement:

- matching between planner-owned locality contracts and bridge-owned slice
  summaries
- explicit classification of:
  - in-region relevant changes
  - off-region suppressible changes
  - locality-mismatch widening denials
  - admitted widening outcomes where the plan says they are legal
- exact counters for region matches, partition matches, off-region
  suppressions, widening admissions, and widening denials
- exact denial behavior when bridge summaries are too weak for the admitted
  locality-sensitive family
- explicit budget accounting for locality breadth and widening width so broad
  work cannot masquerade as successful narrow maintenance

This phase leaves the system in a coherent state where:

- locality-sensitive live relevance is deterministic and bridge-compatible
- off-region churn suppresses before visible delivery
- broadening across locality boundaries is explicit, typed, and counted

### Phase 4: Lower Query Delivery Into Stream Contracts

Phase 4 exists to make stream-backed delivery a delivery-contract transformation
rather than a second query semantics path.

Milestone 5.1 must then implement:

- query-owned delivery contract artifacts for admitted live output families
- lowering from query delivery contracts into admitted bridge stream-consumer
  contracts
- stream compatibility checks over locality-sensitive delivery families
- typed stream admission failures for unsupported consumer-shape or stream-
  contract combinations
- exact counters for stream admissions, stream denials, and stream-lowered
  delivery width
- explicit member-width and window-width budget enforcement on every admitted
  stream-lowered delivery family

This phase leaves the system in a coherent state where:

- query-shaped delivery can be emitted directly or via stream lowering without
  semantic drift
- bridge stream vocabulary is consumed rather than reinvented
- unsupported stream paths fail before protocol-local drift begins

### Phase 5: Execute, Replay, And Compare Region-Scoped Live Evolution

Phase 5 exists to prove locality-aware live maintenance survives time instead
of only looking plausible in direct execution.

Milestone 5.1 must then implement:

- execution of region-scoped live plans against admitted bridge slice summaries
- replay of locality-bearing change sequences into live subscriptions
- parity comparison against:
  - fresh re-execution at the same progress basis
  - the broader aspect-level control surface for the same query/basis
- canonical replay bundles carrying `query_digest`, `delivery_digest`,
  `replay_digest`, and `counter_snapshot`
- deterministic reporting of region suppression, widening denial, and stream
  contract admission/denial

This phase leaves the system in a coherent state where:

- region-aware live evolution is replay-safe
- narrowing claims are mechanically comparable against broad control lanes
- stream-lowered delivery can be certified from bundle artifacts

### Phase 6: Certification, Counter Proof, And Boundary Hardening

Phase 6 exists to close the milestone through named proof rather than
"finer-grained live updates" demos.

Milestone 5.1 must finally ship:

- the `Region-Scoped Live Narrowing And Stream Contract Test`
- canonical rows proving:
  - relevant-region maintenance
  - off-region suppression
  - broad-versus-region parity safety
  - stream-contract-backed delivery parity
- rejection rows proving:
  - unsupported locality family
  - unsupported locality predicate
  - unsupported stream consumer contract
  - forbidden raw partition/raw stream leakage
  - forbidden widening without plan admission
- compile-fail or privacy hardening proving locality-bearing and stream-lowered
  artifacts cannot be WORTHd externally

This phase leaves the system in a coherent state where:

- region-scoped live narrowing is certifiable rather than aspirational
- Milestone 5.2 can add preview contexts without redefining locality semantics
- Milestone 9 can later compose policy masking on top of a real locality- and
  stream-honest live substrate

### Representative Scenario Matrix

Milestone 5.1 certification should not stay at the level of abstract row names.
At minimum it must exercise these concrete lanes:

- `detail-region-hit`:
  one entity detail query with one admitted region predicate, one in-region
  change, and parity against fresh re-execution
- `detail-off-region-suppressed`:
  same detail query, same aspect family, but change lands in a different
  region and suppresses before visible delivery
- `collection-partition-hit`:
  one ordered collection query rooted in one admitted partition scope with a
  membership-preserving in-partition update
- `collection-cross-partition-denied`:
  same ordered collection query receives a cross-partition change that would
  require broadening and must produce typed widening denial
- `bounded-materialization-region-hit`:
  one bounded materialization query where a descendant inside the admitted
  region updates and the patch remains traversal-bounded
- `cdc-stream-lowered-parity`:
  one CDC-shaped collection delivery lane emitted directly and through one
  admitted stream contract with identical query delivery meaning
- `raw-stream-member-forbidden`:
  a hostile lane attempting to lower raw bridge stream members as query
  delivery payload

If the harness cannot name concrete lanes at this granularity, the milestone is
still too abstract to close honestly.

## Must Ship

- proof-bearing `RegionScopedLivePlan`, `LocalityAwareRelevanceContract`,
  `StreamLoweredDeliveryContract`, and `RegionScopedReplayBundle` families or
  materially equivalent types
- locality-bearing live-plan lowering from admitted Milestone 5 live plans
- query-declared locality predicates or materially equivalent plan-owned
  narrowing surfaces for admitted live families
- bridge-compatible region/partition slice matching for admitted locality-aware
  live maintenance
- explicit widening policy, widening admission, and widening denial artifacts
- plan-owned locality cost posture, breadth budgets, widening budgets, and
  performance status for every admitted family
- query-shaped delivery-contract lowering into admitted bridge stream consumer
  contracts
- plan-owned stream-lowering cost posture and member-width budgets for every
  admitted stream-lowered family
- one dedicated locality/stream performance subdomain owning counters and
  contract status rather than generic telemetry-only logging
- typed locality and stream diagnostics, replay bundles, and exact counters
- milestone-native certification proving locality narrowing, stream-contract
  parity, and rejection behavior

## Must Preserve

- canonical query meaning from Milestone 1 remains authoritative
- validation legality from Milestone 2 remains authoritative
- one-shot planning and basis identity from Milestone 3 remain authoritative
- collection/result-family semantics from Milestone 4 remain authoritative
- live promotion, replay, and no-raw-CDC boundaries from Milestone 5 remain
  authoritative
- `worth-signal` remains the owner of dependency scheduling and execution timing
- the runtime bridge remains the owner of slice identity and stream protocol
  vocabulary
- consumers continue to receive query-shaped delivery contracts rather than raw
  partition or stream events
- off-region suppression never changes final query truth; it only changes
  visible delivery cadence
- widening cannot occur silently; every widening class is typed, denied, or
  explicitly admitted
- stream lowering cannot change query result meaning or result-family meaning
- durable stream continuation and persisted stream checkpoints remain out of
  scope and explicitly deferred

## Complexity / Proof Obligations

Milestone 5.1 must name costs and proofs in terms of:

- locality match width
- region match count
- partition match count
- off-region suppression count
- widening admission count
- widening denial count
- stream admission count
- stream denial count
- stream delivery width
- locality breadth budget crossings
- stream member-width budget crossings
- locality-sensitive work avoided versus broad control execution
- executor rediscovery avoidance across locality-bearing paths

Minimum required counters:

- `locality_region_match_count`
- `locality_partition_match_count`
- `locality_off_region_suppression_count`
- `locality_irrelevant_broad_control_count`
- `locality_widening_admission_count`
- `locality_widening_denial_count`
- `locality_bridge_slice_incompatibility_count`
- `stream_contract_admission_count`
- `stream_contract_denial_count`
- `stream_lowered_delivery_count`
- `stream_lowered_delivery_member_count`
- `stream_lowered_delivery_width`
- `locality_breadth_budget_cross_count`
- `locality_widening_budget_cross_count`
- `stream_member_width_budget_cross_count`
- `locality_replay_change_count`
- `locality_replay_divergence_count`
- `locality_work_avoided_by_region_narrowing_count`
- `locality_work_avoided_vs_broad_control_count`
- `locality_executor_rediscovery_count`

Rules:

- counters belong to region-scoped patch envelopes, stream-lowered delivery
  envelopes, and replay bundles
- representative certification scenarios must assert exact counts
- `locality_executor_rediscovery_count` must be exactly zero on every admitted
  path
- every widening denial must increment `locality_widening_denial_count`
- every bridge slice incompatibility that blocks admitted locality behavior
  must increment `locality_bridge_slice_incompatibility_count`
- every denied stream-lowering request must increment
  `stream_contract_denial_count`
- every locality breadth budget crossing must increment
  `locality_breadth_budget_cross_count`
- every widening-budget crossing must increment
  `locality_widening_budget_cross_count`
- every stream member-width budget crossing must increment
  `stream_member_width_budget_cross_count`
- no supported path may hide broad control execution inside locality-bearing
  maintenance counts
- "work avoided" counters must compare the locality-bearing lane to the broad
  aspect control lane where both are admitted

Minimum certification rows should include:

- `region-live-convergence`
- `off-region-suppression-parity`
- `broad-vs-region-narrowing-control`
- `stream-contract-delivery-parity`
- `locality-breadth-budget-enforcement`
- `stream-member-width-budget-enforcement`
- `locality-work-avoided-counter-parity`

Minimum rejection rows should include:

- `unsupported-locality-family`
- `unsupported-locality-predicate`
- `unsupported-stream-consumer-contract`
- `raw-partition-event-leakage-forbidden`
- `raw-stream-member-leakage-forbidden`
- `forbidden-locality-widening`
- `forbidden-broad-success-lane`
- `forbidden-stream-width-overflow-success`
- `bridge-slice-incompatibility-denied`

## Allowed Debt

- some live families may remain non-locality-admitted as explicit `Debt` while
  admitted families are fully parity-proven
- richer locality predicate families may remain `Debt` if the admitted
  locality-scope vocabulary is closed, explicit, and certified
- broader stream consumer shapes may remain `Debt` if the shipped stream
  contract matrix is closed, explicit, and certified
- durable stream continuation and persisted checkpoints may remain blocked on
  `Milestone 11`
- raw partition events, raw stream members, or broad aspect invalidation
  disguised as region-aware query delivery may not exist as debt
- host-side locality classification or host-side stream lowering may not exist
  as debt

## Acceptance Evidence

Milestone 5.1 is complete only when `worth-query` can prove:

- the `Region-Scoped Live Narrowing And Stream Contract Test` in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/test-requirements.md)
  passes with canonical machine-checkable artifacts
- region-scoped live invalidation stays narrower than broad aspect invalidation
  when lower-runtime locality contracts admit that narrowing
- off-region changes suppress before visible query delivery
- query-shaped delivery contracts can lower into formal stream contracts
  without changing query meaning
- unsupported locality or stream-contract combinations fail typed and early
- replaying the same locality-bearing change history yields the same
  locality-aware live result and delivery evolution
- widening beyond the admitted locality contract is explicit, counted, and
  never silent

Required verification output must include:

- `query_digest`
- `delivery_digest`
- `replay_digest`
- `counter_snapshot`

## Architectural Notes

### Region Narrowing Must Stay Query-Shaped

Milestone 5.1 should be precise only in ways that preserve the query's own
meaning:

- reuse Milestone 5 live planning and patch identity
- derive locality relevance from declared query scope and bridge slice facts
- compare locality-bearing lanes against broad control lanes explicitly

It must not be precise in these dishonest ways:

- infer region identity from host topic names or transport partition keys
- reinterpret raw bridge slice artifacts as consumer-ready query patches
- silently broaden to full aspect scope and still claim region-aware delivery

### Stream Contracts Must Stay Delivery-Contract Honest

The stream contract is not the query. The stream contract is the admitted
bridge protocol shape for delivering query-owned meaning.

The required rule is:

- query delivery defines what changed for the query
- stream lowering defines how that query-owned delivery is consumed through the
  bridge protocol

If stream lowering can change membership, ordering, locality suppression, or
result-family meaning, the milestone is structurally wrong.

### Widening Denial Is A First-Class Outcome

The easiest way to fake this milestone is to claim region support but route
every ambiguous change through broad invalidation or full refresh.

Milestone 5.1 must instead make widening explicit:

- admitted widening classes are named and narrow
- denied widening classes are typed and counted
- unsupported bridge locality evidence is surfaced as incompatibility, not
  hidden recovery behavior

If locality-sensitive support depends on "just widen when unsure," the family
is not honestly admitted.

### Performance Must Be Admission-Owned

The main performance trap here is letting locality be semantically explicit but
cost posture remain execution-local.

That would let an implementation say:

- "yes, this is a region-scoped plan"
- while still broad-scanning the collection on every accepted update
- or broadening stream member construction whenever payload shaping gets hard

Milestone 5.1 must instead require:

- cost posture attached to the admitted plan
- breadth budgets attached to the admitted plan
- typed outcomes when those budgets are crossed
- exact proof rows showing narrow successful lanes really stay narrow

If performance posture is not encoded before execution begins, the architecture
is still soft even if the semantics sound precise.

### Milestone 5.1 Must Not Smuggle In Durability

Bridge stream docs already carry checkpoint, resume, and replay vocabulary.
That does not mean Milestone 5.1 closes durable stream continuation.

This milestone is about:

- runtime-backed locality-aware live narrowing
- formal stream-contract lowering
- parity-safe query-shaped delivery

It is not about:

- persisted stream offsets
- restart-stable continuation
- durable checkpoints
- portable stream artifacts

Those remain later durable work even if the type vocabulary deliberately leaves
room for them.

## Sequencing Notes

Milestone 5.1 belongs immediately after Milestone 5 because it hardens the
live substrate already shipped there instead of introducing a new capability
family such as preview basis, history, or policy masking.

It must land before Milestone 5.2 because preview-session contexts will need to
compose with whatever live/locality semantics already exist.

It must land before Milestone 9 because policy-aware live masking should
compose on top of a real locality- and delivery-contract substrate rather than
redefining live narrowing itself.

## Parallelization Notes

Once the locality-bearing live-plan boundary is frozen:

- early Milestone 5.2 preview-context work can proceed in parallel without
  redefining locality semantics
- bridge-side stream-consumer hardening can proceed in parallel so long as it
  continues to expose the same bridge-owned stream contract vocabulary
- broader locality predicate experiments can proceed behind explicit debt
  markers without weakening admitted families
- counter hardening and compile-time tightening can proceed in parallel without
  changing milestone semantics

## Explicit Failure Taxonomy For Milestone 5.1

- unsupported locality-bearing live family
- unsupported locality predicate or locality scope
- invalid locality-bearing promotion input
- bridge slice incompatibility
- off-region suppression invariant break
- forbidden locality widening
- stream consumer-contract incompatibility
- forbidden raw partition event leakage
- forbidden raw stream member leakage
- stream lowering semantic drift
- locality replay divergence
- locality artifact invariant break

## Anti-Patterns Explicitly Rejected

- region-aware support implemented as broad aspect invalidation with a nicer
  name
- host-side locality classification based on topics, paths, or callback shape
- raw partition events or raw stream members as consumer-facing query delivery
- stream-contract lowering that redefines query membership or ordering meaning
- hidden widening or hidden broad refresh as the default answer to locality
  ambiguity
- one mega-module mixing locality planning, bridge matching, stream lowering,
  replay, and diagnostics
- public construction of locality-bearing or stream-lowered proof types without
  the proving path
- durable-stream marketing language for a milestone that explicitly does not
  close restart-stable continuation

## Self-Check

This milestone solves a real structural problem rather than packaging work
cosmetically because it introduces the first locality-bearing live proof
boundary between broad live relevance and region/partition-aware live meaning.

The adversarial constraint is load-bearing because it forbids the naive failure
mode where every locality-sensitive update path silently broadens to aspect-
level invalidation or transport-defined semantics.

The milestone preserves authority boundaries because `worth-query` owns
locality-bearing live semantics and query-shaped delivery, the runtime bridge
still owns slice and stream protocol vocabulary, and `worth-signal` still owns
scheduling.

The milestone defines proof obligations rather than implementation chores
because narrower-than-broad invalidation, stream-contract parity, widening
denial, replay parity, and exact locality/stream counters are required for
closeout.

A competent engineer should be able to map this spec into honest locality,
delivery-contract, replay, certification, and compile-fail modules without
inventing the architecture during implementation.

This milestone belongs at 5.1 because it is a decimal hardening pass on live
semantics before preview, frontier/planning posture, and workflow surfaces
build on top of the live substrate.

## Closeout Standard

Milestone 5.1 is complete only when all of the following are true:

- admitted Milestone 5 live families can attach locality-bearing narrowing
  contracts without a second live runtime
- locality-sensitive changes are classified explicitly as in-region,
  off-region, widening-admitted, or widening-denied
- off-region churn suppresses before visible delivery
- admitted query delivery can lower into formal bridge stream contracts without
  semantic drift
- widening beyond the declared locality contract is explicit, typed, and
  counted
- replay proves region-aware convergence against fresh execution and against
  the broader aspect control lane where both are admitted
- forbidden raw partition/raw stream leakage and unsupported locality/stream
  combinations fail typed and early

If code lands but locality support still depends on broad invalidation, host
topic heuristics, raw bridge events, hidden widening, or implied durable
continuation, Milestone 5.1 is not complete.
