# Milestone 5.3 Engineering Spec: Frontier-Aware Planning And Deterministic Parallel Admission

> **Status:** Completed. Closeout: [milestone-5.3-closeout.md](./milestone-5.3-closeout.md).
>
> **Roadmap parent:** [worth_query_roadmap.md](./WORTH_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./WORTH_query_vision.md)
>
> **Prior milestone:** [milestone-5.md](./milestone-5.md)
>
> **Adjacent milestone:** `milestone-5.2.md` is intentionally concurrent work and must remain authority-distinct from this milestone's planning/posture boundary.
>
> **Adjacent hardening milestone:** [milestone-5.1.md](./milestone-5.1.md)
>
> **Prior closeout:** [milestone-5-closeout.md](./milestone-5-closeout.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** make query planning consume lower-runtime frontier posture and deterministic parallel-admission proofs so serial versus parallel execution remains a plan-owned cost choice with identical canonical query meaning instead of an executor-side heuristic
>
> **Companion docs:**
> - [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)
> - [arch_laws.md](../../docs/coding-guidelines/arch_laws.md)
> - [perf_laws.md](../../docs/coding-guidelines/perf_laws.md)
> - [domain_laws.md](../../docs/coding-guidelines/domain_structure_laws.md)
> - [worth_query_vision.md](./WORTH_query_vision.md)
> - [worth_query_roadmap.md](./WORTH_query_roadmap.md)
> - [test-requirements.md](./test-requirements.md)
> - [milestone-4.md](./milestone-4.md)
> - [milestone-4-closeout.md](./milestone-4-closeout.md)
> - [milestone-5.md](./milestone-5.md)
> - [milestone-5-closeout.md](./milestone-5-closeout.md)
> - [milestone-5.1.md](./milestone-5.1.md)

## Goal

Make planning frontier-aware and parallel-admission-aware so admitted bulk,
collection, and live-maintenance query families can preserve one canonical
query/result meaning while the plan declares whether the route is serial,
parallel-admitted, or serial-fallback, with mechanically visible breadth and
admission evidence.

## Why This Milestone Exists

Milestone 3 proved that `worth-query` can lower canonical query meaning into a
plan without executor rediscovery. Milestone 4 proved that large-surface
collection/result families remain planner-owned. Milestone 5 and Milestone 5.1
proved that live and locality-bearing execution stay query-shaped and replay-
safe.

That still leaves one dangerous gap:

- the planner can know the query shape and basis but still plan as if serial
  execution is the only honest route
- the executor can then "help" by speculating about breadth, frontier shape,
  and parallel safety at runtime
- parallel work can become a hidden optimization path whose failure mode is
  semantic drift, hidden fallback, or impossible-to-certify cost posture

`worth-query` should not act surprised that `worth-signal` already has
frontier and deterministic parallel-admission knowledge. If the lower runtime
can prove that a set of query work packets is frontier-disjoint and safe to run
in parallel, the planner must own that admission decision. If the lower runtime
cannot prove it, the serial fallback must be explicit before execution starts.

Milestone 5.3 therefore exists to freeze:

- frontier posture as planner input, not executor folklore
- deterministic parallel admission as a lowered route property, not speculative
  runtime branching
- serial fallback as a typed plan outcome, not a hidden "couldn't parallelize"
  anecdote
- breadth prediction, realized breadth, and admission counters as first-class
  proof artifacts
- parity between serial and parallel admitted lanes for the same canonical
  query meaning

If this milestone is skipped, `worth-query` would still have proof-bearing
plans, but its cost posture would remain soft at exactly the point where bulk
reads, live maintenance, and multi-query orchestration start to matter.

## Governing Summaries

- `MENTALITY.md`: the hard problem is not "make it faster with parallelism."
  It is preserving canonical query meaning while exposing cost posture
  honestly under broad/bursty workloads. The milestone must solve
  meaning-preserving planning posture first.
- `arch_laws.md`: Laws 2, 7, 8, 17, 22, 26, 27, 30, 31, 32, 33, and 41
  dominate this milestone. Admission decisions belong before execution,
  lower-runtime authority must stay separate from query lowering, proof types
  must encode what was proven, and measurement boundaries must exist at the
  planner/executor seam.
- `perf_laws.md`: semantic intent should compile into execution strategy, no
  reuse or narrowing claim is honest without explicit equivalence and counters,
  and throughput scales with structural independence rather than hopeful core
  usage. Parallelism without disjointness proof is fake throughput.
- `domain_laws.md`: frontier posture derivation, admission lowering, fallback
  classification, execution-route counters, replay parity, and diagnostics are
  separate responsibilities and must not ship as one "parallel planner" blob.
- `worth_query_vision.md`: `worth-query` owns query expression and planning,
  not reactive scheduling. It must give the runtime enough structure to
  optimize, narrow, and incrementally maintain results without taking over
  lower-runtime authority.
- `worth_query_roadmap.md`: Milestone 5.3 is specifically about frontier-aware
  planning and deterministic parallel admission. It belongs after the live
  substrate exists and before historical/correspondence/workflow milestones
  compound cost posture on top of soft planning.
- `test-requirements.md`: the `Frontier Planning And Parallel Admission Parity
  Test` is the closeout proof. It requires identical canonical meaning across
  serial, parallel-admitted, and typed serial-fallback lanes, plus exact
  counters for predicted and realized breadth.
- `milestone-4.md`: broad collection/result families already have planner-owned
  breadth, ordering, and result-shape semantics. Milestone 5.3 must consume
  those plan families rather than creating a second bulk-execution planner.
- `milestone-4-closeout.md`: runtime-backed collection certification, breadth
  counters, and no-rediscovery proof already exist. Milestone 5.3 should reuse
  that counter and certification posture for bulk/collection lanes.
- `milestone-5.md`: live promotion already froze query-shaped patch meaning,
  replay identity, refresh policy, and no-raw-CDC boundaries. Parallel
  admission must operate on those frozen live artifacts rather than reopening
  live semantics.
- `milestone-5-closeout.md`: the runtime-backed live substrate is already
  closed for admitted families. Milestone 5.3 should treat live execution as
  one consumer of planned route posture, not as a separate optimization system.
- `milestone-5.1.md`: locality-aware narrowing and stream-contract lowering are
  already planner-owned live extensions. Frontier and parallel-admission
  posture must compose with those live/locality artifacts without redefining
  locality truth.

## Adversarial Constraint

Milestone 5.3 must survive the following hostile condition:

> The same admitted canonical query or query bundle is planned once against a
> lower-runtime frontier surface that may or may not prove structural
> disjointness, then executed through serial, parallel-admitted, or typed
> serial-fallback routes; every admitted route must preserve identical query
> meaning, result meaning, and delivery-family meaning while making breadth and
> admission posture explicit rather than executor-discovered.

Concretely, the design must remain correct when all of the following are true:

- bulk collection queries, bounded materialization queries, and admitted live
  maintenance families each produce multiple planner-owned work packets whose
  packet identity, packet digest, packet-local output boundary, and merge path
  are all derived from canonical query meaning rather than executor chunking
  convenience
- the lower runtime can sometimes prove structural disjointness and sometimes
  deny it for nearly identical query shapes because the active frontier or
  basis changed
- multi-query bundles carry one exact resolved bundle basis digest, and any
  route whose resolved basis differs must be rejected rather than admitted as a
  "same basis class" convenience
- predicted breadth and realized breadth can differ, and that difference must
  become a typed route outcome rather than a silent successful lane plus one
  counter increment
- a naive executor could decide to speculate, lock, widen, or serialize late
  instead of honoring the planner-owned admission result
- preview-session work in Milestone 5.2 is landing concurrently and must not be
  required for frontier posture or parallel parity to remain meaningful

If any supported path:

- changes result meaning because parallel admission used a different semantic
  route than the serial control lane
- lets the executor rediscover or override planner-owned admission decisions
- treats "parallel" as a best-effort runtime convenience instead of a proved
  route property
- silently falls back to serial execution without typed diagnostics and exact
  counters
- hides broad frontier scans or coordination cost behind one generic execution
  counter
- requires preview-session context, host orchestration, or consumer-local
  batching to explain why work was or was not parallelized

then Milestone 5.3 has failed.

## Product Decision Lock

- frontier posture is lower-runtime authority and must enter `worth-query`
  through explicit planner inputs or query-facing adapters, not through
  executor-local probing
- deterministic parallel admission is a planned execution-route property, not a
  speculative runtime optimization
- serial fallback is a successful but explicit route class only when the plan
  lowered it as such; it is never an invisible recovery path
- serial and parallel-admitted routes must preserve identical canonical query,
  result, basis, and delivery-family meaning
- `worth-signal` remains authoritative for frontier shape and parallel-safety
  semantics; `worth-query` owns how that knowledge becomes plan metadata,
  diagnostics, and certification artifacts
- executor/runtime code may consume planned route posture and realized
  execution counters, but may not re-decide parallel safety
- multi-query bundle parallelism is admitted only through explicit disjointness
  proofs carried by the lowered plan bundle, never by opportunistic task
  spawning
- Milestone 5.2 preview-session basis work is a separate authority surface;
  Milestone 5.3 may compose with preview bases later but must not depend on
  preview lifecycle artifacts to define frontier or admission semantics

Normative consequence:

- any implementation path that asks the executor to "try parallel, fall back if
  it seems unsafe" is out of spec
- any implementation path that hides serial fallback inside generic runtime
  scheduling is out of spec
- any implementation path that claims planner-owned breadth posture without
  exact predicted-versus-realized counters is out of spec
- any implementation path that derives parallel safety from host thread pools,
  transport fan-out, or consumer affinity instead of structural frontier proof
  is out of spec

## Compile-Time Enforcement Policy

Milestone 5.3 must classify which frontier/admission guarantees become
unrepresentable, uncompilable, or construction-time rejection.

`Unrepresentable` in public types:

- publicly constructible frontier-aware plan artifacts that do not carry source
  query-plan identity, frontier posture identity, and route-admission posture
- publicly constructible parallel-admitted execution routes that do not carry
  the disjointness proof or admitted serial-fallback class they were lowered
  from
- publicly constructible route-counter bundles that omit predicted breadth,
  realized breadth, and admission outcome identity
- publicly constructible "parallel options" bags that flatten semantically
  distinct route classes into booleans or free-form config

`Uncompilable` through visibility and compile-fail enforcement:

- external construction of `FrontierAwarePlan`, `ParallelAdmissionRoute`,
  `SerialFallbackRoute`, `FrontierParityBundle`, or materially equivalent
  proof-bearing artifacts without crate-owned lowering
- public APIs that accept raw executor hints, raw thread-pool options, or
  host-authored "safe to parallelize" claims as though they were admitted
  planning inputs
- public execution surfaces that let consumers override route posture after the
  plan is lowered
- public conversion paths that bypass one-shot/live/locality plan families and
  mint frontier-aware routes directly from raw query authorship
- internal execution surfaces that accept both `ParallelAdmissionRoute` and
  `SerialFallbackRoute` through one untyped executor entry point

`Construction-time rejection`:

- unsupported query families requested for frontier-aware or parallel-admitted
  planning
- unsupported multi-query bundle compositions whose shared basis or locality
  proofs are insufficient for deterministic admission
- unsupported lower-runtime frontier postures
- invalid or contradictory route policy requests
- unsupported serial-fallback classes

Rules:

- the strongest available boundary must be used
- frontier/admission proof types must use sealed constructors and private
  fields
- compile-fail coverage is required for no external route-forging and no
  executor-override boundaries
- route-typed executor entry points are mandatory so a serial or fallback route
  cannot be handed to the parallel executor path by mistake
- runtime rejection is allowed only for facts genuinely unavailable until the
  lower runtime exposes the active frontier posture for the declared basis

## Scope

### In Scope

- frontier-aware planning metadata for admitted collection, bulk, and live
  maintenance families
- deterministic parallel-admission lowering for admitted query routes and
  multi-query bundles
- typed serial-fallback lowering where deterministic parallel admission is
  denied
- exact predicted-breadth and realized-breadth counters on planned routes and
  execution reports
- parity/replay comparison of serial, parallel-admitted, and serial-fallback
  lanes for the same canonical query meaning
- milestone-native certification for frontier planning and deterministic
  parallel admission
- compile-time and privacy hardening around route posture artifacts

### Explicitly Out Of Scope

- preview-session basis identity, preview lifecycle, or preview-versus-promoted
  comparison semantics, which remain Milestone 5.2 authority
- historical, diff, lineage, or correspondence basis variation
- store-backed frontier parity or store-backed execution parity
- dynamic work-stealing schedulers, transport-level fan-out, or thread-pool
  tuning
- executor-side speculative locking or conflict detection as an alternative to
  planner-owned admission
- durable continuation, restart-stable replay, or persisted route posture
  artifacts

### Initial Admission Matrix

Initial frontier-aware query families admitted in Milestone 5.3:

- ordered collection families whose Milestone 4 collection plans already expose
  bounded breadth and explicit result-family identity
- bounded materialization families whose traversal breadth is already planner-
  owned and locality-safe where applicable
- admitted Milestone 5 live families whose relevance/patch families are
  already frozen and whose lower-runtime frontier posture can be surfaced
  without redefining live meaning
- admitted Milestone 5.1 locality-bearing live families only when the
  lower-runtime frontier posture composes with the already-admitted locality
  contract instead of widening it
- multi-query bundles composed only of the admitted families above and one
  exact resolved bundle basis digest/proof

Initial frontier-denied families:

- unbounded traversal or materialization families
- unsupported aggregate or rollup families whose breadth posture is not yet
  lowered honestly
- query families requiring preview-session basis semantics to define their
  correctness
- any family whose parallel safety would require executor-local discovery,
  speculative locking, or host-authored batching
- any bundle whose member routes resolve to different basis digests even if
  they share one broader basis class

Initial route posture classes:

- `frontier-serial`
- `frontier-parallel-admitted`
- `frontier-serial-fallback`
- typed denial for unsupported frontier/admission combinations

Any route posture not named above is out of scope for Milestone 5.3 and must
fail typed and early rather than becoming implicit beta support.

### Initial Performance Posture Matrix

- ordered collection + frontier serial:
  predicted breadth is bounded by the planned collection breadth class and one
  serial route over the admitted packet family
- ordered collection + frontier parallel-admitted:
  predicted breadth is bounded by the same collection breadth class, but work
  packets may execute concurrently only when the lower runtime proves
  disjointness
- bounded materialization + frontier serial:
  predicted breadth is bounded by traversal/materialization breadth already
  frozen in Milestone 4
- bounded materialization + frontier parallel-admitted:
  packet-level concurrency is admitted only when descendant scopes are proven
  disjoint and no route needs executor-local merge decisions
- live/locality-bearing maintenance + frontier serial or serial fallback:
  route posture may stay serial even when the live family itself is admitted;
  that serial posture must remain explicit and counted, not hidden as "just how
  this run happened"

Rules:

- predicted breadth, route posture, and fallback class belong to the lowered
  plan, not to executor logs
- realized breadth and admitted parallel batch counts belong to execution
  reports and replay bundles
- any admitted route whose realized breadth violates its declared posture must
  emit typed diagnostics and exact counters

## Frontier-Aware Planning Architecture

### One Planning Boundary

Milestone 5.3 extends the existing proof chain. It must not create a second
"parallel planner" beside ordinary query planning.

The authoritative flow becomes:

`ValidatedQueryBundle`
-> `ExecutionPlanBundle` or `LiveQueryPlan` where already admitted
-> `FrontierPlanningInput`
-> `PlannedWorkPacketSet`
-> `FrontierAwarePlan`
-> `ParallelAdmissionRouteSet`
-> `ExecutionRouteReport`
-> `FrontierParityBundle`

Frontier posture therefore consumes already-proven query meaning. It does not
re-author:

- query legality
- basis identity
- collection/result-family meaning
- live patch semantics
- locality meaning

### Authority Boundaries

`worth-query` owns:

- lowering admitted query plans into frontier-aware route posture
- lowering admitted route families into planner-owned work packets with exact
  packet identity and merge boundaries
- route-family identity for serial, parallel-admitted, and serial-fallback
  lanes
- predicted breadth posture and route-level diagnostics
- parity/replay certification artifacts for serial versus parallel admitted
  execution

`worth-signal` owns:

- frontier topology and disjointness semantics
- deterministic parallel-admission criteria
- lower-runtime proof surfaces indicating whether routes are safe to parallelize

Execution owns:

- consuming the already-lowered route posture
- reporting realized breadth, admitted batch counts, and fallback realization
- refusing executor-side speculative admission

Hosts and orchestration glue may own:

- transport and lifecycle of already-lowered plan bundles
- presentation of already-emitted route diagnostics

Hosts and orchestration glue may not own:

- deciding parallel safety
- downgrading or upgrading route posture
- synthesizing serial-fallback explanations
- grouping arbitrary queries into "parallel bundles" without planner-owned
  admission

### Frontier And Route Vocabulary

Representative artifact families:

- `FrontierPlanningInput`
- `PlannedWorkPacket`
- `PlannedWorkPacketDigest`
- `PacketEquivalenceContract`
- `PacketMergeContract`
- `PacketMergeBoundary`
- `BundleResolvedBasisDigest`
- `FrontierPostureDigest`
- `FrontierBreadthPrediction`
- `FrontierPredictionDriftOutcome`
- `FrontierDisjointnessClass`
- `FrontierAwarePlan`
- `PlannedRouteFamily`
- `ParallelAdmissionRoute`
- `SerialFallbackRoute`
- `ParallelAdmissionDecision`
- `SerialFallbackReason`
- `ParallelAdmissionCounterBundle`
- `FrontierParityBundle`

Rules:

- frontier posture is digest-bearing and basis-explicit
- every planner-owned packet must carry one packet digest derived from
  canonical query meaning, not executor chunk size
- packet families must declare one closed merge contract that explains how
  packet outputs reduce back into one canonical result or delivery-family
  artifact
- only packet families carrying proven disjointness may enter the
  parallel-admitted lane
- route posture is closed vocabulary, not free-form runtime text
- every parallel-admitted route must carry the disjointness proof class it was
  lowered from
- every serial-fallback route must carry the exact denial or fallback reason it
  was lowered from
- multi-query bundles must carry one exact `BundleResolvedBasisDigest` plus
  per-route posture identity; mixed-basis bundles are typed denial, not soft
  unsupported composition
- executor reports must round-trip to the planned route family without
  semantic reinterpretation

## Phases

### Phase 1: Freeze Frontier And Admission Authority Surfaces

Phase 1 exists to prevent frontier posture from leaking into executor folklore.

Milestone 5.3 must introduce:

- `FrontierPlanningInput`
- `PlannedWorkPacket`
- `PlannedWorkPacketDigest`
- `PacketEquivalenceContract`
- `PacketMergeContract`
- `PacketMergeBoundary`
- `BundleResolvedBasisDigest`
- `FrontierPostureDigest`
- `FrontierBreadthPrediction`
- `FrontierPredictionDriftOutcome`
- `FrontierDisjointnessClass`
- `FrontierAwarePlan`
- `PlannedRouteFamily`
- `ParallelAdmissionRoute`
- `SerialFallbackRoute`
- `ParallelAdmissionDecision`
- `SerialFallbackReason`
- `ParallelAdmissionCounterBundle`
- `FrontierParityBundle`
- `FrontierPlanningReport`
- `FrontierComplexityContract`
- `FrontierPerformanceStatus`

This phase leaves the system in a coherent state where:

- frontier posture is a query-owned planning boundary
- packetization is planner-owned rather than executor-owned
- route posture is explicit and typed
- serial fallback is no longer an implicit runtime story
- performance claims have named contract surfaces before execution begins

### Phase 2: Lower Frontier Posture Into Query Plans

Phase 2 exists to make lower-runtime frontier knowledge visible at planning
time instead of during execution surprises.

Milestone 5.3 must then implement:

- lowering from admitted `ExecutionPlanBundle`, `CollectionPlanBundle`,
  `LiveQueryPlan`, and admitted locality-bearing live plans into
  `FrontierAwarePlan`
- lowering those admitted plans into one `PlannedWorkPacketSet` whose packet
  boundaries and packet digests are canonical for that route family
- derivation of predicted breadth from already-frozen collection/live/locality
  semantics plus lower-runtime frontier posture
- exact bundle-basis proof derivation through `BundleResolvedBasisDigest`
- explicit classification of which admitted query families can request frontier
  posture
- typed denial for unsupported frontier-aware requests
- typed denial for mixed-basis bundles even when basis classes match
- explicit route-level parity identity so serial and parallel lanes stay tied
  to the same canonical query meaning

This phase leaves the system in a coherent state where:

- frontier-aware planning is derived from canonical query meaning
- unsupported families fail before execution
- route posture no longer depends on executor-local discovery

### Phase 3: Lower Deterministic Parallel Admission And Serial Fallback

Phase 3 exists to keep parallelism honest by making admission a proof-bearing
plan result instead of speculative concurrency.

Milestone 5.3 must then implement:

- deterministic lowering into `ParallelAdmissionRoute` when lower-runtime
  disjointness is proven
- deterministic lowering into `SerialFallbackRoute` when lower-runtime
  disjointness is denied for an otherwise admitted frontier-aware family
- explicit fallback reasons such as:
  - overlapping frontier slices
  - insufficient disjointness proof
  - unsupported bundle composition
  - locality/route interaction denial where applicable
- explicit route posture for multi-query bundles with mixed admitted and denied
  routes
- exact counters for planned parallel route count, planned serial fallback
  count, and predicted breadth per route
- a typed `FrontierPredictionDriftOutcome` family with at least:
  - `WithinBudget`
  - `SerialFallbackRequired`
  - `DeniedByDrift`

This phase leaves the system in a coherent state where:

- every admitted route already knows whether it is serial or parallel-admitted
- fallback is explicit and digest-bearing
- the executor no longer has permission to invent admission posture

### Phase 4: Execute Planned Routes Without Executor Rediscovery

Phase 4 exists to make route posture executable without semantic mutation.

Milestone 5.3 must then implement:

- execution of `ParallelAdmissionRoute` through a parallel-route-typed entry
  point without executor-side reclassification
- execution of `SerialFallbackRoute` with the same canonical result meaning as
  the serial control lane through a serial-route-typed entry point
- exact reporting of realized breadth, admitted parallel batch counts, serial
  fallback realizations, and executor rediscovery attempts
- explicit failure if execution would require speculative route mutation
- explicit failure if prediction drift resolves to `DeniedByDrift`; explicit
  typed serial route if it resolves to `SerialFallbackRequired`
- route reports that preserve query/result/delivery-family identity while
  adding only cost posture evidence

This phase leaves the system in a coherent state where:

- serial and parallel-admitted routes are executable without semantic drift
- realized breadth is mechanically comparable to predicted breadth
- executor rediscovery is visible and rejectable instead of silently tolerated

### Phase 5: Replay, Compare, And Certify Serial Versus Parallel Parity

Phase 5 exists to prove that parallel admission changes cost posture only.

Milestone 5.3 must then implement:

- parity comparison across:
  - frontier-aware serial control routes
  - frontier-aware parallel-admitted routes
  - typed serial-fallback routes
- replay or repeated execution for admitted route families on the same basis
- canonical bundles carrying `query_digest`, `plan_digest`, `result_digest`,
  and `counter_snapshot`
- explicit comparison of predicted breadth to realized breadth
- explicit reporting when unsupported families were denied rather than admitted

This phase leaves the system in a coherent state where:

- serial versus parallel parity is machine-checkable
- fallback honesty is certifiable
- breadth claims can be audited from bundle artifacts alone

### Phase 6: Counter Hardening, Boundary Hardening, And Closeout

Phase 6 exists to close the milestone through proof instead of "parallel seems
faster" demos.

Milestone 5.3 must finally ship:

- the `Frontier Planning And Parallel Admission Parity Test`
- canonical rows proving:
  - serial control parity
  - parallel-admitted parity
  - typed serial-fallback parity
  - predicted-versus-realized breadth visibility
  - exact-basis bundle parity
- rejection rows proving:
  - unsupported frontier family
  - unsupported bundle composition
  - mixed-basis-bundle-denied
  - forbidden executor-side speculative admission
  - forbidden hidden serial fallback
  - forbidden-serial-route-on-parallel-entrypoint
- compile-fail or privacy hardening proving route posture artifacts cannot be
  WORTHd externally

This phase leaves the system in a coherent state where:

- frontier-aware planning is certifiable rather than aspirational
- Milestone 5.4 can add correspondence/historical materialization posture
  without reopening route-admission honesty
- Milestone 5.6 can surface capability metadata against a real route-posture
  substrate

## Must Ship

- proof-bearing `FrontierAwarePlan`, `ParallelAdmissionRoute`,
  `SerialFallbackRoute`, and `FrontierParityBundle` families or materially
  equivalent types
- frontier-aware lowering from admitted query plans into route-posture-aware
  plans
- deterministic parallel-admission metadata on admitted planned routes
- typed serial-fallback metadata and diagnostics on denied or non-admitted
  routes
- one dedicated frontier/admission performance subdomain owning cost
  contracts, posture counters, and performance status
- exact predicted-breadth and realized-breadth counters on every admitted
  frontier-aware route family
- planner-owned packet identity and merge contracts for every admitted
  frontier-aware route family
- milestone-native certification proving serial/parallel parity, fallback
  honesty, and rejection behavior
- compile-time/privacy hardening preventing executor or host code from
  fabricating route posture

## Must Preserve

- canonical query meaning from Milestone 1 remains authoritative
- validation legality from Milestone 2 remains authoritative
- proof-bearing planning and basis identity from Milestone 3 remain
  authoritative
- collection/result-family semantics from Milestone 4 remain authoritative
- live promotion, replay, and no-raw-CDC boundaries from Milestone 5 remain
  authoritative
- locality-bearing live semantics and stream-contract boundaries from
  Milestone 5.1 remain authoritative where those families compose with this
  milestone
- `worth-signal` remains the owner of frontier and deterministic
  parallel-admission semantics
- execution consumes lowered route posture rather than speculating about
  parallel safety
- serial and parallel-admitted lanes remain semantically identical for the same
  canonical query/basis pair
- unsupported families fail closed or remain explicit debt rather than
  opportunistically parallelizing

## Complexity / Proof Obligations

Milestone 5.3 must name costs and proofs in terms of:

- frontier lookup count
- predicted breadth
- realized breadth
- admitted parallel route count
- admitted parallel batch count
- serial fallback plan count
- serial fallback execution count
- disjointness denial count
- bundle route count
- executor rediscovery count
- work avoided versus serial control where the route was parallel-admitted
- mixed-basis bundle denial count
- packet merge width
- packet merge reduction count

Minimum required counters:

- `frontier_lookup_count`
- `frontier_prediction_count`
- `frontier_predicted_breadth`
- `frontier_realized_breadth`
- `parallel_admission_route_count`
- `parallel_admission_batch_count`
- `parallel_admission_denial_count`
- `serial_fallback_plan_count`
- `serial_fallback_execution_count`
- `bundle_parallel_route_count`
- `bundle_serial_route_count`
- `mixed_basis_bundle_denial_count`
- `packet_merge_width`
- `packet_merge_reduction_count`
- `frontier_prediction_drift_count`
- `executor_parallel_rediscovery_count`
- `work_avoided_by_parallel_admission_count`
- `work_preserved_by_serial_fallback_count`

Rules:

- counters belong to route reports and parity bundles
- representative certification scenarios must assert exact counts
- `executor_parallel_rediscovery_count` must be exactly zero on every admitted
  path
- every denied deterministic-admission attempt must increment
  `parallel_admission_denial_count`
- every realized serial-fallback execution must increment
  `serial_fallback_execution_count`
- any predicted-versus-realized breadth mismatch outside the admitted posture
  must increment `frontier_prediction_drift_count`
- every mixed-basis bundle denial must increment
  `mixed_basis_bundle_denial_count`
- every admitted packet merge must increment `packet_merge_reduction_count`
- no successful parallel lane may continue after a `DeniedByDrift` outcome
- no supported path may hide route mutation inside generic execution counters
- "work avoided" and "work preserved" counters must make the cost difference
  between parallel-admitted and serial-fallback lanes mechanically visible

Minimum certification rows should include:

- `frontier-serial-control`
- `parallel-admitted-parity`
- `serial-fallback-parity`
- `predicted-vs-realized-breadth`
- `bundle-route-posture-parity`
- `exact-basis-bundle-parity`
- `work-avoided-counter-parity`

Minimum rejection rows should include:

- `unsupported-frontier-family`
- `unsupported-bundle-composition`
- `mixed-basis-bundle-denied`
- `forbidden-executor-speculative-admission`
- `forbidden-hidden-serial-fallback`
- `invalid-route-posture-override`
- `forbidden-serial-route-on-parallel-entrypoint`

## Allowed Debt

- unsupported query families may remain serial-only as explicit `Debt`
- broader bundle composition classes may remain `Debt` if admitted bundle
  families are closed, explicit, and certified
- richer frontier cost models may remain `Debt` if shipped routes already carry
  explicit predicted and realized breadth counters
- preview-session composition with frontier posture may remain deferred to
  Milestone 5.2 closeout or follow-on integration work
- executor-side speculative parallel admission may not exist as debt
- hidden serial fallback may not exist as debt
- host-authored parallel safety claims may not exist as debt

## Acceptance Evidence

Milestone 5.3 is complete only when `worth-query` can prove:

- the `Frontier Planning And Parallel Admission Parity Test` in
  [test-requirements.md](./test-requirements.md)
  passes with canonical machine-checkable artifacts
- frontier-aware planning decisions are explicit and digest-bearing
- admitted serial and parallel-admitted routes remain semantically identical
- typed serial-fallback routes remain explicit rather than hidden executor
  behavior
- unsupported parallel families fail closed or remain explicit debt
- predicted breadth, realized breadth, admitted parallel routes, and fallback
  outcomes are mechanically visible in exact counters
- mixed-basis bundles fail typed and early
- prediction drift resolves into one explicit outcome contract rather than one
  passive counter

Required verification output must include:

- `query_digest`
- `plan_digest`
- `result_digest`
- `counter_snapshot`

## Architectural Notes

### Parallelism Must Be A Proof, Not A Hope

The key rule in this milestone is simple:

- the planner may admit parallel execution only when the lower runtime proved
  structural disjointness
- the executor may never "discover" that proof on its own
- the executor may never invent packet boundaries or merge contracts on its own

Anything weaker creates a system that might be fast, but cannot be certified.

### Packet Identity Must Be Canonical

If the planner says a route is parallel-admissible, it must also say what the
parallel units actually are.

The required rule is:

- packet boundaries are planner-owned
- packet identity is digest-bearing
- merge shape is planner-owned
- executor chunk size is an implementation detail inside one packet, never the
  definition of the packet itself

Otherwise "parallel route" is just hidden executor sharding with nicer
language.

### Frontier Posture Must Change Cost, Not Meaning

Parallel admission is allowed to change:

- when work is scheduled
- how many disjoint packets run together
- how much coordination cost is paid

It is not allowed to change:

- which query semantics were applied
- which rows or patches exist
- which basis was read
- which delivery-family meaning was produced

If route posture changes result meaning, the planner and executor are no
longer talking about the same query.

### Serial Fallback Is Not Failure, But It Must Be Honest

Some admitted families will remain serial for honest reasons. That is fine.
What is not fine is pretending the executor "just happened" to serialize.

The required rule is:

- serial fallback is a named planned route
- serial fallback carries one explicit reason
- serial fallback stays parity-safe with the serial control lane
- serial fallback cannot be accidentally executed through the parallel entry
  point because the executor API surface must be route-typed

That makes fallback a real architectural boundary instead of a debugging story.

### This Milestone Must Not Steal Preview Authority

Milestone 5.2 is about preview-session basis identity and branch-workflow
foundations. Milestone 5.3 is about planning cost posture.

The boundary must stay clean:

- 5.2 owns whether a preview basis is a valid basis class
- 5.3 owns how an already-admitted basis class lowers into serial or parallel
  route posture

If 5.3 needs preview-session lifecycle semantics in order to define route
posture, the split is wrong.

## Sequencing Notes

Milestone 5.3 belongs after Milestone 5 because live and collection semantics
must already be frozen before route posture can harden them.

It belongs after Milestone 5.1 because locality-bearing live families need
their narrowing contracts frozen before frontier posture can compose with them
honestly.

It can progress concurrently with Milestone 5.2 because preview-session basis
identity and frontier/parallel posture are separate authority surfaces. The
shared rule is that 5.3 may consume already-admitted basis classes, but may not
define them.

## Parallelization Notes

Once the frontier-aware planning boundary is frozen:

- Milestone 5.2 preview-session work can continue in parallel so long as it
  exposes basis identity through the existing planning seam rather than a new
  executor-local one
- Milestone 5.4 correspondence/historical work can design materialization-path
  posture without changing serial/parallel parity semantics
- counter hardening and compile-time tightening can proceed in parallel without
  changing milestone semantics

## Explicit Failure Taxonomy For Milestone 5.3

- unsupported frontier-aware family
- unsupported multi-query bundle composition
- invalid frontier posture input
- deterministic parallel-admission denial
- mixed-basis bundle denial
- prediction drift denial
- invalid route-posture override
- forbidden executor speculative admission
- forbidden hidden serial fallback
- forbidden serial-route-on-parallel-entrypoint
- frontier prediction drift
- serial/parallel parity divergence
- frontier artifact invariant break

## Anti-Patterns Explicitly Rejected

- "try parallel and see what happens"
- executor-side lock probing as a substitute for planner-owned disjointness
  proof
- booleans like `can_parallelize=true` with no proof-bearing route identity
- executor-defined packetization or executor-defined merge reduction
- hidden serial fallback inside one generic execution lane
- host-side grouping of arbitrary queries into parallel batches
- one mega-module mixing frontier posture derivation, route lowering,
  execution, diagnostics, and certification
- any implementation that requires preview-session lifecycle semantics to
  explain ordinary frontier posture

## Self-Check

This milestone solves a real structural problem rather than packaging work
cosmetically because it freezes how `worth-query` consumes lower-runtime
frontier knowledge before historical, workflow, and facade work multiply the
number of broad execution surfaces.

The adversarial constraint is load-bearing because it forbids the easy failure
mode where parallelism appears as an executor-local optimization that silently
changes route semantics or hides fallback.

The milestone preserves authority boundaries because `worth-signal` still owns
frontier and disjointness truth, `worth-query` owns plan lowering and route
posture artifacts, and the executor remains a consumer rather than a second
planner.

The milestone defines proof obligations rather than implementation chores
because serial/parallel parity, typed serial fallback, predicted-versus-
realized breadth counters, and zero executor rediscovery are required for
closeout.

A competent engineer should be able to map this spec into honest frontier,
route-posture, execution-report, and certification modules without inventing
the architecture during implementation.

This milestone belongs at 5.3 because it hardens planning posture after live
and locality semantics exist, while still staying ahead of correspondence,
workflow, facade, and later historical/store-backed work.

## Closeout Standard

Milestone 5.3 is complete only when all of the following are true:

- admitted query families can lower into frontier-aware route posture without a
  second planner
- deterministic parallel admission is explicit, typed, and proof-bearing
- serial fallback is explicit, typed, and parity-safe
- serial and parallel-admitted lanes remain semantically identical
- predicted breadth and realized breadth are mechanically visible and exact
- executor-side speculative admission and hidden fallback fail closed

If code lands but route posture still depends on executor heuristics, hidden
fallback, host-authored batching, or non-sealed admission artifacts, Milestone
5.3 is not complete.
