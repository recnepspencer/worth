# Milestone 11 Engineering Spec: Background Maintenance Isolation And Scheduling Contracts

> **Status:** Completed on 2026-04-21
>
> **Closeout:** [milestone-11-closeout.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-11-closeout.md)
>
> **Roadmap parent:** [worth_store_roadmap.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/worth_store_roadmap.md)
>
> **Vision parent:** [worth_store_vision.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/worth_store_vision.md)
>
> **Test requirements:** [test-requirements.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/test-requirements.md)
>
> **Prerequisite milestones:**
> - [milestone-10.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-10.md)
> - [milestone-8.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-8.md)
> - [milestone-9.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-9.md)
>
> **Concurrent milestone:**
> - `Milestone 13` (`Tiering And Durable Working-Set Intelligence`)
>
> **Impacted later milestones:**
> - `Milestone 12` (`Artifact Format Evolution And Rolling Compatibility`)
> - `Milestone 14` (`Replication, Capsules, And Integrity Verification`)
> - `Milestone 21` (`Admission Control And Budget Contracts`)
> - `Milestone 22` (`Operator Repair, Audit, And Forensic Recovery Tooling`)
>
> **Primary architectural driver:** make background maintenance a typed,
> proof-bearing store subsystem that can pace compaction, rebuild, snapshot,
> and publication-adjacent work without turning queue state, worker heuristics,
> or tier-placement opportunism into shadow authority

## Goal

Make background maintenance operationally explicit and mechanically bounded so
foreground writes, foreground reads, and policy-promised visibility do not
quietly become side effects of whichever maintenance worker happened to run
first.

## Why This Milestone Exists

Milestone 11 is not "add a task runner."

It is the milestone that decides whether `worth-store` can execute derived
maintenance honestly once Milestone 10 has made retention, compaction, reclaim,
and rebuild debt semantically exact.

Milestone 10 defines:

- what retained truth must survive
- what compaction may rewrite
- what reclaim may delete
- what rebuild debt must be emitted when derived families are removed

That is necessary, but not sufficient. Once those rules exist, the store still
has a second hard problem:

- how compaction, rebuild, snapshot refresh, bulk residue cleanup, and
  replication-preparation work are admitted under foreground load
- how the store prevents background debt from silently capturing I/O, memory,
  and branch-local locality needed by foreground work
- how a foreground path can prove it remained on its admitted lane rather than
  paying hidden maintenance tax
- how later Milestone 13 tier movement and working-set adaptation can run in
  parallel without becoming the de facto scheduler for retention and rebuild
  work

If this milestone is weak, the store inherits a fake architecture:

- Milestone 10 will be semantically correct, but operator-visible behavior will
  still depend on ad hoc background worker timing
- Milestone 13 tiering will compete with compaction and rebuild in whatever
  order seems profitable locally, even when that broadens foreground work
- replication, compatibility migration, and repair tooling will each invent
  their own maintenance loops because no shared scheduling boundary exists
- budget and admission milestones later will observe debt after it already
  harmed foreground latency instead of before

This milestone exists to lock one typed maintenance runtime boundary before
late-store platform programs start layering their own workers on top of
retention semantics.

## Hard Part

The hard part is not creating queues.

The hard part is keeping six things separate that naive storage systems blur
together:

- canonical authoritative truth and foreground truth-serving work
- Milestone 10 policy decisions about what maintenance is legal
- background execution of derived maintenance work
- maintenance debt accounting and escalation policy
- tenant, branch, or locality-aware placement work from Milestone 13
- operator-facing pacing and degraded-state reporting

The design fails if:

- foreground reads become fast only when a background materializer happened to
  precompute the right family, so maintenance timing becomes shadow authority
- background work steals enough read or write budget that foreground latency is
  determined by queue depth instead of explicit policy
- maintenance scheduling depends on backend-local queue shape, file ordering, or
  thread availability rather than typed work classes and explicit budgets
- rebuild debt and compaction debt exist only as counters after the fact rather
  than inputs to admission and escalation
- Milestone 13 tier moves are allowed to outrank retained-truth rebuild or
  compaction cutover work simply because placement heuristics called them hot
- one broad worker loop executes compaction, rebuild, snapshot, and
  replication-prep under one opaque "background tasks" umbrella, erasing
  failure topology and cost boundaries

Milestone 11 therefore has to define one maintenance admission model, one
foreground-isolation model, and one debt-escalation model that later platform
work inherits instead of bypassing.

## Explicit Assumptions

- Milestone 10 remains the authority on retention legality, compaction
  cutover, reclaim eligibility, basis-survival publication, and rebuild debt.
- Milestone 11 schedules maintenance work; it does not decide whether that work
  is semantically legal.
- Milestone 8 continues to own stable-basis reads and durable continuation
  semantics; Milestone 11 may protect their latency and resource posture, but
  it may not redefine basis or cursor meaning.
- Milestone 9 bulk programs remain their own execution family with their own
  checkpoint and witness model; Milestone 11 may schedule cleanup or deferred
  rebuild around bulk results, but it does not absorb bulk orchestration.
- Milestone 13 is developed concurrently and owns placement policy, tier
  movement, and working-set adaptation; Milestone 11 must provide the scheduling
  containment those programs run inside, not absorb their placement semantics.
- `worth-relational` and the runtime stack continue to own truth semantics; the
  maintenance scheduler owns only derived work admission, pacing, and reporting.
- background maintenance results remain derived or support artifacts; queue
  state, worker reservations, and scheduler hints are never authority.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects is solving the hostile operational
  failure before convenience workers spread everywhere. Milestone 11 therefore
  starts from hidden foreground interference and unbounded debt growth, not from
  "we should process maintenance asynchronously."
- `arch_laws.md`
  The most important thing it protects here is plan/execute separation with
  proof-bearing boundaries. Maintenance legality, scheduling admission,
  execution pacing, debt escalation, and operator reporting must remain
  separate types and phases instead of one mutable worker state machine.
- `perf_laws.md`
  The most important thing it protects is cost honesty. Milestone 11 must name
  queue breadth, worker reservation, foreground interference, and escalation
  triggers explicitly instead of hiding them behind elapsed-time metrics.
- `domain_laws.md`
  The most important thing it protects is decomposition by reason-to-change.
  Work-class definition, admission, pacing, reservation, escalation, and
  evidence publication must be separate subdomains rather than one background
  module.
- `worth_store_vision.md`
  The most important thing it protects is that derived durable work remains
  subordinate to canonical truth and explicit policy. Milestone 11 must
  therefore schedule maintenance without making maintenance timing part of truth
  meaning.
- `worth_store_roadmap.md`
  The most important thing it protects is sequencing. Milestone 11 belongs
  after Milestone 10 because the scheduler cannot honestly pace work whose
  legality and rebuild basis are still ambiguous, and it must leave room for
  Milestone 13 to progress concurrently.
- `test-requirements.md`
  The most important thing it protects is certification-grade proof. Milestone
  11 needs a named suite proving foreground isolation, bounded pacing, and
  debt-escalation honesty rather than a benchmark narrative.
- `worth_store_dependency_map.md`
  The most important thing it protects is unlock shape. Milestone 10 unlocks
  Milestone 11, Milestone 12, and Milestone 13 together; this spec therefore
  has to make concurrency with Milestone 13 explicit rather than accidental.
- `milestone-8.md`
  The most important thing it protects is that stable-basis reads and durable
  continuation remain exact, cost-honest, and typed. Milestone 11 must preserve
  those foreground contracts under background load.
- `milestone-9.md`
  The most important thing it protects is that bulk work remains canonical and
  resumable with explicit support artifacts. Milestone 11 should schedule any
  follow-on cleanup or debt around bulk work without collapsing bulk execution
  into the generic maintenance runtime.
- `milestone-10.md`
  The most important thing it protects is one retained-range authority model
  plus one non-authoritative compaction and reclaim pipeline. Milestone 11 must
  inherit those work classes and debt surfaces rather than redefining them as
  scheduler-local concepts.

## Adversarial Constraint

Milestone 11 must survive this hostile condition:

> A store under sustained foreground reads and writes, active Milestone 10
> compaction/reclaim/rebuild debt, restart-recovered maintenance backlog,
> varying tenant or branch-local hot spots, and concurrently developed
> Milestone 13 tier-move proposals must preserve foreground latency and
> foreground truth visibility within declared budgets while executing only
> policy-admitted background work, escalating debt explicitly when budgets are
> exceeded, and never allowing queue order, worker contention, or placement
> heuristics to become the hidden scheduler of retained truth.

## Product Decision Lock

- background maintenance may execute only over typed work units whose legality
  has already been established by earlier milestone contracts
- foreground reads and writes always have an explicit resource reservation that
  background work may not consume implicitly
- queue position, backlog age, and worker utilization are scheduling facts, not
  authority facts
- maintenance debt is an architectural surface, not a private scheduler metric
- restart-recovered backlog must re-enter the scheduler through the same typed
  admission path as freshly planned work
- Milestone 13 tiering work may share the maintenance runtime, but it must do
  so through its own work classes and priorities rather than by mutating
  compaction or rebuild queues
- no maintenance family may claim hidden priority because it is "usually cheap"
  or "just a cache refresh"; priority must be declared and observable
- foreground operations must surface when they benefited from, waited on, or
  were broadened by maintenance state
- the first ship may be conservative about concurrency and pacing, but it may
  not be ambiguous about which work classes are admitted, which are deferred,
  and which policy trigger caused escalation

Normative consequence:

- any implementation that lets compaction or rebuild run opportunistically
  whenever a thread is idle, without foreground reservations and explicit
  pacing, is out of spec
- any implementation that allows tier movement or working-set refresh to bypass
  the shared maintenance admission model is out of spec
- any implementation that reports clean foreground success while hidden
  maintenance interference broadened or delayed the path is out of spec
- any implementation that reconstructs backlog priority from backend-local file
  order, queue insertion order alone, or thread wake timing is out of spec
- any implementation that cannot restate why work was deferred, escalated,
  paused, or admitted from typed scheduler evidence is out of spec

## Scope

### In Scope

- explicit maintenance work-class taxonomy for compaction, reclaim-adjacent
  cleanup, rebuild, snapshot publication or refresh, replication-preparation,
  and maintenance-local audit or validation tasks admitted at this stage
- a concrete first-ship scheduler topology rather than abstract worker
  vocabulary alone
- scheduler admission and pacing contracts over typed maintenance work units
- foreground reservation and interference boundaries for reads, writes, and
  basis-pinned continuation paths
- restart-visible backlog reconstruction and re-admission
- debt publication, starvation detection, and escalation policy surfaces
- operator-visible scheduling state, rejection causes, and degraded posture
- concurrency containment between Milestone 11 maintenance scheduling and
  Milestone 13 tiering work classes
- machine-checkable evidence and exact counters for isolation and pacing claims

### Explicitly Out Of Scope

- deciding retention legality, reclaim legality, or basis-survival conclusions,
  which remain Milestone 10 work
- deciding placement policy, hotness classification, or tier semantics, which
  remain Milestone 13 work
- compatibility/version policy, which remains Milestone 12 work
- replication capsule meaning or cross-machine transfer semantics, which remain
  Milestone 14 work
- global admission budgets across all artifact families, which remain Milestone
  21 work
- operator repair-plan semantics, which remain Milestone 22 work

## Maintenance Authority Model

### Background Non-Authority Rule

Background maintenance is never a truth authority surface.

Maintenance may:

- publish or refresh derived durable families
- execute Milestone 10 compaction or rebuild plans already proven legal
- prepare later replication or export inputs once those products exist
- refresh snapshots or other derived acceleration families already admitted by
  prior milestone contracts

Maintenance may not:

- redefine retained truth
- invent basis-survival conclusions
- promote queue state into policy truth
- change branch, cursor, schema, or lineage meaning
- turn a derived artifact into the only readable path for foreground truth

Required classification rule:

- canonical truth-serving reads and writes remain foreground authority-path work
- maintenance work units remain derived execution intents
- scheduler state remains ephemeral control-plane state
- debt summaries and escalation records are support artifacts for operation and
  certification, not semantic authority

### First-Ship Scheduler Shape Rule

Milestone 11 must freeze one concrete first-ship scheduler topology so the
implementation cannot hide a naive queue behind abstract nouns.

Required first-ship topology:

- one store-owned maintenance facade
- one scheduler authority that admits work from descriptors only
- one queue family per work class
- one locality bucket inside each queue family
- one explicit foreground reservation pool
- one explicit background reservation pool
- one restart-recovered intake lane

Required first-ship posture:

- work is admitted by `WorkClass x LocalityScope x ReservationClass`
- FIFO is allowed only inside one `(work class, locality bucket, priority band)`
  lane
- cross-work-class ordering must be resolved by typed priority policy, not by
  insertion order alone
- cross-locality borrowing is forbidden by default and requires explicit
  escalation

Explicit first-ship debt:

- adaptive multi-pool work stealing across locality buckets
- dynamic topology reshaping based on observed density regime
- speculative reservation borrowing across foreground and background pools

Rules:

- alternative scheduler shapes may appear later only as explicit roadmap work
- first ship may be conservative, but it may not be topology-ambiguous

### Multi-Dimensional Resource Budget Rule

Reservation must be expressed in the resource dimensions that actually create
foreground pain.

Required budget objects:

- `IoBudgetUnits`
- `CpuBudgetUnits`
- `MemoryBudgetUnits`
- `PublicationSlotBudget`
- `ForegroundLatencyGuard`

Required descriptor surfaces:

- `PredictedIoDemand`
- `PredictedCpuDemand`
- `PredictedMemoryDemand`
- `PredictedPublicationDemand`

Rules:

- every admitted `MaintenanceWorkDescriptor` must declare predicted demand in
  these dimensions before admission
- no work may be admitted on the strength of one vague "background slot"
- background admission must fail, defer, or down-quantize if any one resource
  dimension exceeds the currently available background budget
- borrowing from foreground budgets must remain explicit per resource
  dimension, not as one rolled-up escalation flag

### Work-Class Identity Rule

Milestone 11 must freeze one exact work-class vocabulary before building any
worker runtime.

Minimum required work classes:

- `CompactionMaintenanceWork`
- `RetainedRangeRebuildWork`
- `SnapshotRefreshWork`
- `DerivedFamilyRebuildWork`
- `ReplicationPreparationWork`
- `MaintenanceAuditWork`
- `TierPlacementProposalWork`
- `TierMoveExecutionWork`

Rules:

- each admitted work item belongs to exactly one work class
- each admitted work item belongs to exactly one `LocalityScopeToken`
- each work class declares whether it is foreground-blocking, foreground-aware,
  or fully deferrable
- each work class declares the proofs it must consume before execution
- Milestone 13 work classes are present here only as scheduling containers; the
  semantic meaning of tiering still belongs to Milestone 13

Required locality vocabulary:

- `BranchLocalityScope`
- `ArtifactFamilyLocalityScope`
- `TenantLocalityScope`
- `StoreGlobalLocalityScope`

Rules:

- `StoreGlobalLocalityScope` is explicit debt-bearing breadth, not the default
- work that cannot name its locality scope is not admissible

### Maintenance Plan Rule

Execution must consume lowered maintenance plans, not raw artifact ids or raw
queue callbacks.

Minimum plan families:

- `ForegroundReservedMaintenancePlan`
- `BackgroundPacedMaintenancePlan`
- `EscalatedMaintenancePlan`
- `DeferredMaintenancePlan`

Required execution-lifecycle families:

- `DiscoveredMaintenanceWork`
- `AdmittedMaintenanceWork`
- `ReservedMaintenanceWork`
- `ExecutingMaintenanceWork`
- `CompletedMaintenanceReceipt`
- `CancelledMaintenanceWork`

Required pacing surfaces:

- `MaintenanceQuantum`
- `PacingWindow`
- `QuantumBudgetReceipt`

Required rule:

- a plan is produced after legality, work class, locality scope, and resource
  posture are resolved
- execution consumes the plan without re-deciding whether the work is legal,
  urgent, or allowed to borrow foreground budget
- changing priority, locality, or pacing posture requires a new plan or a typed
  escalation transition, not mutation of a live worker object
- only `ReservedMaintenanceWork` may transition into
  `ExecutingMaintenanceWork`
- only `ExecutingMaintenanceWork` may produce
  `CompletedMaintenanceReceipt`
- obsolete or superseded work must transition to `CancelledMaintenanceWork`
  rather than being silently left in queue state
- execution may consume only one admitted `MaintenanceQuantum` at a time
- continued execution after a quantum expires requires a fresh
  `QuantumBudgetReceipt`, not an ambient loop that runs until done

### Foreground Reservation Rule

Foreground truth-serving work must have an explicit reservation boundary the
maintenance runtime cannot cross silently.

Minimum reservation families:

- `ForegroundWriteReservation`
- `ForegroundReadReservation`
- `ForegroundContinuationReservation`
- `BackgroundMaintenanceReservation`

Rules:

- background work may consume only capacity explicitly assigned to background
  reservations
- borrowing from foreground reservations must be an explicit escalated outcome
  recorded in evidence, never an ambient optimization
- if a foreground path waits on maintenance, the result envelope must say which
  maintenance class caused the wait and whether the wait was policy-admitted
- foreground waits on background cutover, publish, or reservation release must
  be explicit wait families rather than hidden lock contention

### Debt And Escalation Rule

Debt must become an input to scheduling before it becomes an operational
surprise.

Minimum debt families:

- `CompactionDebt`
- `RebuildDebt`
- `SnapshotDebt`
- `ReplicationPreparationDebt`
- `TierPlacementDebt`

Required freshness surfaces:

- `PlanGeneration`
- `SupersessionEpoch`
- `FreshnessWindow`

Minimum escalation outcomes:

- `StayBackground`
- `PaceUpWithinBackgroundBudget`
- `EscalateWithForegroundImpact`
- `DeferWithOperatorSignal`
- `RejectNewDerivedWork`

Rules:

- debt is recorded per work class and locality scope
- escalation decisions must be typed and machine-checkable
- escalation cannot change semantic legality; it changes only execution posture
- first ship may reject some escalations rather than performing them, but the
  rejection must be explicit and observable
- stale work must be freshness-checked before expensive construction begins
- freshness failure must lead to cancellation, coalescing, or replanning, not
  to "finish the old work since we already started"

### Naive Trap Rejection Rule

Milestone 11 must explicitly block the failure modes a naive scheduler will
otherwise ship.

Minimum traps and mandatory posture:

- `ForegroundWaitsOnBackgroundWhileBackgroundHoldsPublicationSlot`
  must be represented as an explicit typed violation or an explicit foreground
  wait class; it may not appear as ordinary latency
- `GlobalFifoStarvesHotLocalityRepair`
  must be prevented by work-class and locality-bucket separation; one cold queue
  may not bury unrelated hot-locality repair forever
- `TierMoveFloodBlocksCompactionCutover`
  must be prevented by separate work classes plus priority policy; tier moves
  may not consume the compaction cutover lane implicitly
- `RestartRequeuesSupersededWork`
  must be prevented by equivalence and cancellation rules; recovered backlog
  must collapse obsolete work before execution
- `DuplicateDerivedRefreshStorm`
  must be prevented by coalescing or typed duplicate rejection for equivalent
  work descriptors
- `ForegroundReadNeedsMaintenanceWarmupToLookFast`
  must be reported as explicit wait, explicit fallback, or explicit debt, never
  as ordinary isolated success

Rules:

- each trap must map to a typed rejection, cancellation, coalescing, or
  explicit wait surface
- "operators will tune it later" is not an admissible mitigation
- stale-work detection must happen before a plan consumes a fresh quantum or
  expensive materialization budget

### Restart Re-Admission Rule

Recovered backlog must re-enter the scheduler through proof-bearing work
descriptors rather than raw queue replay.

Required restart surfaces:

- `RecoveredMaintenanceBacklog`
- `RecoveredMaintenanceDescriptor`
- `RestartMaintenanceAdmission`

Rules:

- restart may reconstruct pending work from persisted maintenance support
  artifacts and retained truth basis
- restart may not trust thread-local progress, host-local temp queues, or
  partial worker memory
- recovered work must re-pass admission and pacing under current reservations
  before execution resumes
- recovered work must be equivalence-checked against already completed or
  superseded work before it re-enters execution

### Equivalence And Coalescing Rule

Maintenance reuse and cancellation require an explicit sameness contract.

Required equivalence surfaces:

- `MaintenanceWorkIdentity`
- `MaintenanceEquivalenceKey`
- `SupersededMaintenanceWitness`

Rules:

- equivalent refresh or rebuild work must canonicalize to the same
  `MaintenanceEquivalenceKey`
- coalescing is legal only for work classes that declare equivalence semantics
- restart recovery and live scheduling must use the same equivalence basis
- cancelling superseded work requires a typed witness, not queue mutation by
  convention
- equivalence checks must consume freshness and generation state rather than
  raw queue position alone

### Milestone 13 Concurrency Boundary Rule

Milestone 11 and Milestone 13 must share one maintenance runtime boundary
without collapsing into one milestone.

Required boundary:

- Milestone 11 owns maintenance work classes, reservation boundaries, pacing,
  starvation detection, and escalation mechanics
- Milestone 13 owns hotness classification, placement policy, tier selection,
  and tier-move semantics
- Milestone 13 may publish `TierPlacementProposalWork` and
  `TierMoveExecutionWork`, but it may not redefine scheduler priorities for
  compaction, rebuild, or snapshot families
- Milestone 11 may schedule tier work, but it may not redefine what "hot,"
  "warm," or "cold" mean

## Compile-Time Boundary Rule

The highest-risk scheduling boundaries must be mechanically enforced.

Required proof-bearing surfaces:

- `MaintenanceWorkDescriptor`
- `BackgroundPacedMaintenancePlan`
- `EscalatedMaintenancePlan`
- `ForegroundReservationWitness`
- `MaintenanceEscalationDecision`
- `RecoveredMaintenanceDescriptor`

Required compile-time posture:

- execution may not accept raw closures or raw artifact identifiers as
  maintenance work
- `DiscoveredMaintenanceWork` may not execute before admission
- `AdmittedMaintenanceWork` may not execute before reservation
- `ReservedMaintenanceWork` may not be reconstructed from raw queue state
- escalated execution may not be caller-synthesized from booleans such as
  `force = true`; it must consume a typed escalation decision
- tier work may not be enqueued through compaction or rebuild plan constructors
- restart recovery may not inject raw queue entries without reconstructing a
  typed recovered descriptor
- cross-locality execution may not borrow another locality bucket without an
  explicit `CrossLocalityEscalationWitness`
- foreground result envelopes may not omit the resolved interference posture
  when maintenance interaction occurred

Required proof surface:

- compile-fail tests for raw-id maintenance execution attempts
- compile-fail tests for executing discovered work before admission
- compile-fail tests for executing admitted work before reservation
- compile-fail tests for synthetic escalation construction
- compile-fail tests for tier-work submission through non-tier work families
- compile-fail tests for restart queue injection without recovered descriptors
- compile-fail tests for cross-locality execution without escalation witness

## Performance-Shaping Types Rule

Milestone 11 must encode the dominant pacing and interference decisions into
lowered types so execution consumes a resolved strategy rather than
rediscovering it in worker threads.

Required performance-shaping surfaces:

- `MaintenanceQueueSummary`
- `MaintenanceLocalitySummary`
- `MaintenanceReservationSummary`
- `MaintenancePacingPlan`
- `MaintenanceResourceBudgetSummary`
- `ForegroundIsolationVerdict`
- `MaintenanceInterferenceReport`
- `MaintenanceCoalescingDecision`
- `SchedulerTopologyDescriptor`
- `ColdStartSchedulerSummary`

Required posture:

- queue breadth and locality are summarized once before execution
- foreground isolation is decided before worker admission begins
- pacing plans carry exact work-class, locality, and reservation posture
- pacing plans carry exact multi-dimensional resource posture
- execution does not re-scan global backlog to rediscover work class or
  priority once a plan exists

Rules:

- execution may not promote deferred work to escalated work without an explicit
  transition record
- foreground reads and writes may not have to inspect scheduler internals to
  learn whether maintenance interfered; that posture must already be present in
  their result surface
- execution may not rediscover equivalence or locality from raw artifacts once
  a trusted summary exists
- cold-start scheduling may not require warm in-memory debt or queue summaries
  to remain bounded and honest

## Lowered Work Unit Families Rule

Maintenance breadth must be partitioned by semantic family and locality before
execution.

Required lowered work units:

- `CompactionMaintenanceUnit`
- `RebuildMaintenanceUnit`
- `SnapshotMaintenanceUnit`
- `ReplicationPreparationUnit`
- `MaintenanceAuditUnit`
- `TierPlacementMaintenanceUnit`
- `TierMoveMaintenanceUnit`
- `ForegroundWaitDependencyUnit`

Required posture:

- every unit ties to one work class and one locality scope
- foreground-aware work remains separable from fully deferrable work
- unit families remain distinct even if one worker pool later executes multiple
  families

Rules:

- compaction, rebuild, snapshot, and tier movement may not collapse into one
  undifferentiated queue item type
- maintenance audit work remains distinct from mutation-adjacent maintenance
  work
- foreground-wait dependencies must be explicit units rather than hidden lock
  relationships

## Read And Result Cost Surface Rule

Foreground and maintenance results must expose their scheduling posture.

Required result surfaces:

- resolved reservation class
- resolved maintenance plan family
- resolved quantum width
- resolved resource-budget posture
- wait-or-interference classification
- debt delta caused, reduced, or observed
- broadened foreground work count attributable to maintenance interference
- coalesced-work count
- cancelled-superseded-work count
- locality-borrow posture
- locality-touch count
- global-scope fallback count

Rules:

- a caller must be able to tell whether foreground work stayed isolated,
  observed explicit interference, or triggered an escalation path
- zero-work idle scheduling and real maintenance execution must be
  distinguishable

## Complexity-Status Surface Rule

Milestone 11 evidence must publish path-local complexity status rather than one
rolled-up scheduler verdict.

Minimum named paths:

- `maintenance_admission`
- `maintenance_pacing`
- `foreground_isolation`
- `debt_escalation`
- `restart_readmission`
- `freshness_rejection`
- `cold_start_scheduler_boot`

Rules:

- each path declares at least `Verified` or `Debt`
- any `Debt` path names the unresolved breadth, tuning gap, or unsupported
  policy shape explicitly

Minimum contracts:

- maintenance admission cost is proportional to:
  - work descriptors inspected
  - locality scopes summarized
  - reservation classes evaluated
  - resource dimensions evaluated
  - not total store history or total derived artifact count
- maintenance pacing cost is proportional to:
  - admitted maintenance units scheduled
  - reservation buckets consulted
  - admitted quanta granted
  - escalation thresholds evaluated
  - not total worker wakeups or backend thread count
- foreground isolation cost is proportional to:
  - foreground operations observed
  - maintenance interactions actually encountered
  - declared reservation checks performed
  - not full backlog scans per foreground request
- debt escalation cost is proportional to:
  - debt families evaluated
  - policy thresholds crossed
  - explicit escalation transitions emitted
- restart readmission cost is proportional to:
  - recovered maintenance descriptors reconstructed
  - admissibility checks re-run
  - not blind replay of every historical maintenance event
- freshness rejection cost is proportional to:
  - candidate work items compared against current generation or supersession
    state
  - not full queue replay or full artifact-family re-enumeration
- cold-start scheduler boot cost is proportional to:
  - persisted queue and debt summaries loaded
  - recovered descriptors reconstructed
  - not full historical maintenance artifact replay from genesis

Forbidden hidden work:

- scanning the full maintenance backlog on every foreground read or write
- inferring work class or priority from file placement, queue age alone, or
  worker thread identity
- promoting tiering work by hidden heuristic when foreground reservations are
  already exhausted
- treating debt counters as observability-only while the scheduler makes
  implicit escalation choices elsewhere
- constructing large maintenance units before freshness, quantum, or
  publication-slot admission has succeeded
- requiring warm cache state to avoid store-global queue scans at scheduler
  startup

Minimum counters:

- `maintenance_work_descriptor_count`
- `maintenance_admitted_plan_count`
- `maintenance_deferred_plan_count`
- `maintenance_escalated_plan_count`
- `maintenance_rejected_plan_count`
- `maintenance_quantum_grant_count`
- `maintenance_quantum_exhaustion_count`
- `maintenance_io_budget_units_reserved`
- `maintenance_cpu_budget_units_reserved`
- `maintenance_memory_budget_units_reserved`
- `maintenance_publication_slot_budget_reserved`
- `maintenance_queue_depth`
- `maintenance_queue_locality_scope_count`
- `maintenance_store_global_scope_count`
- `maintenance_background_unit_execute_count`
- `maintenance_foreground_wait_count`
- `maintenance_foreground_wait_on_cutover_count`
- `maintenance_foreground_broadened_count`
- `maintenance_foreground_interference_count`
- `maintenance_reservation_violation_count`
- `maintenance_cross_locality_escalation_count`
- `maintenance_starvation_trigger_count`
- `maintenance_debt_escalation_count`
- `maintenance_compaction_debt_units`
- `maintenance_rebuild_debt_units`
- `maintenance_snapshot_debt_units`
- `maintenance_replication_prep_debt_units`
- `maintenance_tiering_debt_units`
- `maintenance_restart_recovered_count`
- `maintenance_restart_readmission_count`
- `maintenance_restart_rejection_count`
- `maintenance_coalesced_work_count`
- `maintenance_cancelled_superseded_work_count`
- `maintenance_tier_work_execute_count`
- `maintenance_freshness_rejection_count`
- `maintenance_locality_touch_count`
- `maintenance_global_scope_fallback_count`
- `maintenance_cold_start_boot_count`
- `maintenance_cold_start_global_scan_count`
- `maintenance_plan_execute_without_descriptor_count`
- `maintenance_illegal_escalation_count`
- `maintenance_truth_visibility_violation_count`

Required counter assertions:

- `maintenance_reservation_violation_count` remains zero in admitted lanes
- `maintenance_plan_execute_without_descriptor_count` remains zero in all lanes
- `maintenance_illegal_escalation_count` remains zero outside explicit hostile
  proof lanes
- `maintenance_truth_visibility_violation_count` remains zero in all admitted
  certification lanes
- `maintenance_store_global_scope_count` remains zero in representative
  locality-bounded lanes and increments only in explicit debt lanes
- `maintenance_global_scope_fallback_count` remains zero in representative
  admitted locality-bounded lanes
- `maintenance_cross_locality_escalation_count` remains zero outside explicit
  cross-locality hostile or debt lanes
- `maintenance_quantum_exhaustion_count` increments only when work genuinely
  consumes its admitted quantum and must yield or re-admit
- `maintenance_freshness_rejection_count` increments in representative stale
  duplicate and superseded lanes before expensive execution begins
- `maintenance_cold_start_global_scan_count` remains zero in representative
  admitted cold-start lanes
- `maintenance_foreground_broadened_count` remains zero for representative
  isolated foreground lanes and increments only where interference is explicit
- `maintenance_coalesced_work_count` and
  `maintenance_cancelled_superseded_work_count` distinguish real coalescing
  from ordinary execution
- `maintenance_restart_readmission_count` exactly matches the recovered
  descriptors re-admitted in representative restart lanes

## Phases

### Phase 1: Lock Maintenance Work Classes, Reservations, And Concurrency Boundaries

Phase 1 defines what the maintenance runtime is allowed to schedule before any
worker loop exists.

Required work:

- define maintenance work-class taxonomy and work-descriptor identity
- define first-ship scheduler topology and locality bucket model
- define multi-dimensional resource budget objects and descriptor demand fields
- define foreground and background reservation families
- define lowered plan families for admitted, deferred, and escalated work
- define lifecycle types from discovered work to completed receipt
- define quantum, pacing-window, and freshness-generation vocabulary
- define debt families and escalation outcome vocabulary
- define equivalence, coalescing, and supersession witnesses
- define restart-recovered descriptor vocabulary
- define Milestone 13 concurrency boundary and tier-work container classes
- define operator-visible scheduler state and failure taxonomy
- define counters and certification bundle shape for isolation and pacing claims

Exit condition:

- every admitted maintenance unit belongs to one exact work class
- foreground reservations are explicit and machine-checkable
- Milestone 13 can proceed concurrently without mutating Milestone 11
  scheduling semantics

### Phase 2: Implement Admission, Pacing, And Debt Publication

Phase 2 turns scheduler legality and pacing into machine-checkable runtime
surfaces.

Required work:

- implement descriptor-to-plan admission
- implement queue summaries and reservation summaries
- implement multi-dimensional budget admission and quantum sizing
- implement locality-bucket admission and explicit store-global debt lanes
- implement paced background execution and explicit deferral
- implement debt publication per work class and locality scope
- implement duplicate-work coalescing and superseded-work cancellation
- implement freshness rejection before expensive construction
- implement starvation detection and typed escalation decisions
- expose typed admission, pacing, and escalation failures
- emit exact queue, admission, debt, and escalation counters

Exit condition:

- work is no longer admitted through ad hoc callbacks or raw queue entries
- debt and starvation become typed scheduler inputs rather than after-the-fact
  metrics
- the scheduler can explain why work ran, waited, deferred, or escalated

### Phase 3: Enforce Foreground Isolation And Restart Readmission

Phase 3 makes the runtime operationally safe under restart and active load.

Required work:

- implement foreground reservation enforcement for reads, writes, and
  continuation paths
- implement explicit interference reporting in foreground result envelopes
- implement explicit foreground wait dependency tracking for cutover and publish
  blockers
- implement recovered backlog reconstruction and typed restart readmission
- implement cold-start scheduler boot from persisted summaries without
  full-history replay
- collapse or cancel superseded recovered work before execution
- reject execution without descriptors, reservations, or escalation witnesses
- implement typed reservation violations, illegal escalation, and starvation
  failure surfaces
- emit exact foreground-interference and restart counters

Exit condition:

- foreground work can prove whether it stayed isolated
- restart backlog no longer depends on ambient queue state
- illegal maintenance execution paths become mechanically rejectable

### Phase 4: Integrate Tier-Work Containers And Late Maintenance Families

Phase 4 makes the scheduler a real shared runtime boundary for later work
without surrendering Milestone 11 authority boundaries.

Required work:

- admit tier-placement proposal and tier-move execution work as scheduling
  families
- keep tier-work semantics separate from compaction, rebuild, and snapshot work
- admit snapshot refresh and replication-preparation work into the same plan
  family structure
- publish family-local debt and interference evidence across all admitted work
  families
- verify cross-locality borrowing remains explicit and typed for tier work
- verify that concurrent Milestone 13 work uses shared scheduling containment
  instead of its own hidden worker loop

Exit condition:

- late-store maintenance families share one runtime boundary
- Milestone 13 work is schedulable without redefining scheduler semantics
- compaction and rebuild priority are no longer vulnerable to hidden tiering
  worker competition

### Phase 5: Prove Foreground Isolation, Pacing, And Debt-Escalation Honesty

Phase 5 turns maintenance scheduling into a certifiable platform surface.

Required work:

- run the Milestone 11 named suite:
  `Background Maintenance Isolation And Scheduling Test`
- compare isolated foreground lanes against hostile backlog lanes
- compare deferred, escalated, and restart-recovered maintenance lanes
- compare coalesced-versus-duplicate-submission lanes
- compare locality-bounded lanes against explicit cross-locality escalation
  lanes
- compare cold-start lanes against warm-start lanes for equivalent admitted
  backlog shape
- compare freshness-rejected lanes against naive duplicate-execution lanes
- include a tier-move flood lane versus compaction-cutover lane
- include a restart-supersession lane where recovered work should cancel rather
  than execute
- prove queue order and worker timing do not change foreground truth-visible
  conclusions
- emit machine-checkable truth, diagnostics, failure, and counter bundles

Exit condition:

- foreground truth visibility remains exact under admitted maintenance pressure
- background maintenance pacing is explicit and bounded
- debt escalation is typed, observable, and policy-driven
- Milestone 11 closeout evidence exists in machine-checkable form

## Must Ship

- explicit maintenance work-class taxonomy
- explicit first-ship scheduler topology and locality bucket model
- explicit multi-dimensional budget and quantum vocabulary
- proof-bearing maintenance descriptors, reservation witnesses, and lowered plan
  families
- lifecycle-typed work progression from discovered to completed or cancelled
- explicit debt-family and escalation-decision surfaces
- equivalence, coalescing, and supersession surfaces
- freshness and generation-based stale-work rejection surfaces
- paced background scheduling with foreground-reservation enforcement
- restart backlog reconstruction and typed readmission
- family-local scheduling containers for concurrent Milestone 13 work
- operator-visible maintenance state, starvation state, and debt state
- typed maintenance admission, pacing, reservation, restart, and escalation
  failures
- exact scheduling and interference counters
- machine-checkable Milestone 11 certification output

## Must Preserve

- retained truth legality remains owned by Milestone 10, not by the scheduler
- foreground reads, writes, and continuation stay semantically exact regardless
  of maintenance queue state
- maintenance remains derived and policy-driven
- queue order, worker timing, and placement heuristics never become shadow
  authority
- Milestone 13 remains free to define tiering semantics inside the scheduling
  boundary frozen here
- later compatibility, replication, budget, and repair work inherit one shared
  maintenance runtime instead of inventing their own

## Acceptance Evidence

Milestone 11 is complete only when the store satisfies the named Milestone 11
suite:

- `Background Maintenance Isolation And Scheduling Test`

Required machine-checkable outputs:

- `truth_digest`
- `diagnostics_digest`
- `failure_digest`
- `counter_snapshot`
- `scheduler_topology_report`
- `resource_budget_report`
- `maintenance_interference_matrix`
- `debt_escalation_report`

Milestone-specific proof obligations:

- foreground truth-visible results remain equal between isolated and hostile
  backlog lanes for equivalent admitted work
- background maintenance does not silently broaden or delay foreground work
  without explicit evidence
- debt escalation and deferral outcomes are typed and machine-checkable
- restart-recovered backlog re-enters through the same admission model as fresh
  work
- duplicate or superseded work is coalesced or cancelled through explicit
  equivalence rules rather than queue folklore
- stale work is rejected before expensive construction or publication-slot
  consumption
- locality-bounded work remains locality-bounded unless an explicit escalation
  witness says otherwise
- cold-start scheduler admission remains bounded and semantically equivalent to
  warm-start admission for the same recovered backlog
- multi-dimensional background budgets remain explicit enough that one resource
  class cannot be silently exhausted behind another
- tier-move pressure does not silently outrank compaction cutover or retained
  rebuild work
- concurrent tier-work containers do not bypass foreground reservations or
  scheduler visibility
- queue timing changes cost only, not semantic foreground conclusions

Milestone 11 is not closed by "maintenance throughput improved" or "background
tasks eventually drained" claims.

## Architectural Notes

- The smart abstraction is not "background jobs." The smart abstraction is one
  typed maintenance runtime with explicit reservations, debt, and escalation.
- Milestone 10 defines which maintenance work is legal; Milestone 11 defines
  how that work competes with foreground load.
- The concurrency note with Milestone 13 is structural, not cosmetic. Milestone
  13 should inherit the scheduler boundary here rather than introducing another
  hidden worker system.
- Conservative first ship is fine. Hidden interference is not.

## Sequencing Notes

This milestone belongs immediately after Milestone 10 because maintenance
isolation is only honest once retention, compaction, reclaim, and rebuild debt
already have explicit meaning.

- `Milestone 13` should proceed concurrently inside the scheduler boundary
  frozen here because tiering needs typed maintenance containment, but it should
  not block Milestone 11 from locking work classes and reservation rules.
- `Milestone 12` should inherit this maintenance runtime for compatibility
  rebuilds and rolling-format maintenance instead of inventing another
  background execution boundary.
- `Milestone 14`, `Milestone 21`, and `Milestone 22` should reuse the same
  scheduler evidence, debt surfaces, and restart readmission model for
  replication preparation, budget policy, and operator repair workflows.
