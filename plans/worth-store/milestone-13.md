# Milestone 13 Engineering Spec: Tiering And Durable Working-Set Intelligence

> **Status:** Closed via [milestone-13-closeout.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-13-closeout.md)
>
> **Closeout:** [milestone-13-closeout.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-13-closeout.md)
>
> **Roadmap parent:** [worth_store_roadmap.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/worth_store_roadmap.md)
>
> **Vision parent:** [worth_store_vision.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/worth_store_vision.md)
>
> **Test requirements:** [test-requirements.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/test-requirements.md)
>
> **Prerequisite milestones:**
> - [milestone-10.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-10.md)
>
> **Concurrent milestones:**
> - `Milestone 11` (`Background Maintenance Isolation And Scheduling Contracts`)
>
> **Impacted later milestones:**
> - `Milestone 14` (`Replication, Capsules, And Integrity Verification`)
> - `Milestone 20` (`Native Blob And Object Storage`)
> - `Milestone 21` (`Admission Control And Budget Contracts`)
>
> **Primary architectural driver:** make placement, recall, and hotness
> classification physically explicit across authoritative and derived artifact
> families without letting tier residency, working-set heuristics, or scheduler
> behavior become semantic truth

## Goal

Add hot/warm/cold placement and durable working-set intelligence as explicit,
proof-bearing store programs that change where artifacts live and how costly
they are to access without changing replayable truth, restore truth, or branch
meaning.

## Why This Milestone Exists

Milestone 13 is not "put cold things on slower storage."

It is the milestone that decides whether `worth-store` can express placement as
a real product surface instead of a backend-local optimization superstition.

Milestone 10 already made retention, compaction, reclaim, and rebuild debt
honest over authoritative ranges and derived families.

Milestone 13 now has to answer the next hard storage question:

- what tier residency is allowed to mean for canonical authority versus derived
  accelerators
- which artifact families may move, cool, or require recall without changing
  visible truth
- how hotness and working-set classification become typed evidence instead of
  cache folklore
- how cold recall remains an explicit cost lane rather than a hidden fallback
- how placement decisions stay compatible with later replication, blob tiering,
  and budget controls
- how concurrent Milestone 11 scheduling work can pace tier activity without
  redefining what tier placement means

If this milestone is weak, later milestones inherit a false foundation:

- background maintenance will schedule tier work whose semantic boundaries are
  still ambiguous
- replication and capsules will ship artifact sets without honest placement
  posture
- blob storage will invent a second tier model instead of inheriting one
  store-wide placement vocabulary
- budget controls will reason about pressure and locality without a real
  placement authority model
- cold recall will become a backend accident that silently broadens foreground
  work

This milestone exists to make placement, locality, and recall explicit before
later platform programs start assuming "of course we can move this elsewhere"
means anything mechanically precise.

## Hard Part

The hard part is not assigning labels like `hot`, `warm`, or `cold`.

The hard part is keeping one honest separation among six things naive storage
systems routinely collapse:

- authoritative truth that must remain semantically exact regardless of where it
  is stored
- derived durable artifacts whose residency can change aggressively because they
  remain rebuildable from authority
- placement evidence that explains why an artifact is resident in one tier now
- working-set predictions that may guide placement but never redefine truth
- recall paths that make colder placement visible as cost, not as missing truth
- scheduler-owned pacing and debt management that Milestone 11 will own without
  inheriting semantic placement authority

The design fails if:

- a tier move changes which artifact is treated as authoritative for replay,
  restore, or branch reads
- hotness classification is inferred from backend-local cache residue instead of
  explicit typed observation windows
- reads silently broaden from retained authority into opportunistic fallback
  scans because a tier miss was hidden
- the store cannot explain which artifact families are resident, recallable, or
  rebuildable after a restart
- Milestone 11 has to invent its own interpretation of placement classes,
  locality, or recall cost just to schedule work

Milestone 13 therefore has to define one placement authority model, one
working-set evidence model, and one typed recall story that later scheduling,
replication, blob, and budget systems inherit.

## Explicit Assumptions

- Milestone 10 remains the owner of retention, compaction, reclaim,
  retained-range truth, and rebuild-debt legality.
- Canonical authoritative artifacts remain the only semantic durable truth
  authority even when moved across tiers.
- Snapshots, branch-delta layers, Milestone 6 layout families, and later durable
  accelerators remain derived artifacts with explicit rebuild posture.
- Tier residency is a cost and locality property, not a semantic state
  transition.
- Working-set intelligence may influence placement, recall prefetch, and
  operator advice, but it may not change logical branch meaning, replay meaning,
  or restore meaning.
- Milestone 11 is being built concurrently and owns pacing, foreground versus
  background isolation, starvation protection, and debt-escalation policy for
  maintenance work; Milestone 13 must publish placement work classes and cost
  posture without redefining scheduler behavior.
- `worth-relational` still owns commit, branch, lineage, and replay semantics;
  tiering may move storage, but it may not reinterpret surviving authority.
- Later blob placement should inherit this milestone's placement vocabulary
  rather than inventing a second hot/warm/cold model.

If implementation pressure tries to make tier membership a hidden semantic mode
or to make the scheduler the owner of placement meaning, the milestone must be
revised before code lands.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects is solving the hostile placement problem
  before heuristic caching becomes the real architecture. Milestone 13
  therefore starts from shadow-authority and hidden-recall failure modes, not
  from storage-cost convenience.
- `arch_laws.md`
  The most important thing it protects here is proof-bearing authority
  separation and lowered execution plans. Tier placement, recall eligibility,
  and working-set classification must remain structurally separate from
  canonical truth and from scheduler execution.
- `perf_laws.md`
  The most important thing it protects is cost visibility at the boundary that
  claims the win. Milestone 13 must expose tier misses, recall breadth,
  reclassification breadth, and move breadth as exact counters instead of
  hiding them behind vague cache-hit narratives.
- `domain_laws.md`
  The most important thing it protects is decomposition by reason-to-change.
  Placement policy, working-set observation, tier-move planning, recall
  execution, and placement evidence must remain separate subdomains rather than
  one giant storage-heuristics module.
- `worth_store_vision.md`
  The most important thing it protects is that hot/cold branch lifecycle
  tiering and durable working-set intelligence are product-visible accelerators
  layered over durable truth, not alternate truth models. Milestone 13 must
  therefore keep placement advisory, restart-visible, and basis-explicit.
- `worth_store_roadmap.md`
  The most important thing it protects is sequencing. Milestone 13 belongs
  after retention and rebuild rules are stable, should overlap carefully with
  Milestone 11, and must prepare later replication, blob, and budget work
  without inventing new lifecycle semantics.
- `test-requirements.md`
  The most important thing it protects is certification-grade non-authority.
  Milestone 13 is not closed until `Tiering And Working-Set Non-Authority Test`
  proves that movement and adaptation affect cost only.
- `milestone-10.md`
  The most important thing it protects is retained-range truth and rebuild
  honesty. Milestone 13 must consume Milestone 10's authority, compaction, and
  reclaim rules rather than creating tier-local exceptions to them.
- `worth_store_dependency_map.md`
  The most important thing it protects is the actual unlock shape: Milestone 10
  unlocks Milestone 13, Milestone 11 can progress concurrently without owning
  placement semantics, and Milestone 13 is what later budget controls depend on
  for a real tier model.

## Adversarial Constraint

Milestone 13 must survive this hostile condition:

> A store under mixed branch heat, repeated snapshot and layout reads,
> authoritative and derived artifacts moving across hot, warm, and cold tiers,
> restart after partially completed tier moves, concurrent Milestone 11 pacing
> and debt scheduling, and later-read recall from colder placement must preserve
> the same replay truth, restore truth, and visible branch meaning while making
> placement, recall, and hotness decisions explicit as cost-only artifacts.

## Product Decision Lock

- tier residency is always a derived placement fact, never a source of semantic
  truth
- authoritative artifacts may move across tiers, but moving them does not
  change their authority classification
- derived artifacts may be placed more aggressively than authoritative
  artifacts, but only within declared rebuild and retention rules
- working-set intelligence is advisory and evidence-bearing, not ambient cache
  magic
- cold recall is an explicit read or maintenance lane with observable counters
  and typed posture; it is not a silent fallback
- tier moves consume lowered placement plans, not raw file paths, raw hotness
  scores, or ambient backend heuristics
- Milestone 13 owns placement meaning, tier classes, locality evidence, and
  recall posture
- Milestone 11 owns when and how placement work runs in the background, how it
  is paced, and how debt escalates
- later replication, blob tiering, and budget control must inherit this
  milestone's placement vocabulary rather than defining independent residency
  semantics

## Required Contracts And Counters

### First-Ship Placement Policy Rule

Milestone 13 must define a concrete conservative first-ship placement surface so
implementation cannot hide naive traps behind "smarter adaptation later."

Required first-ship supported policies:

- active branch-head authority and stable-basis support may be classified `Hot`
  or `Warm`, but not silently cooled behind implicit authority recall
- retained but not currently hot authoritative ranges may be explicitly placed
  `Warm` when replay and restore remain directly legal from declared authority
- snapshots, branch-delta derived lanes, and Milestone 6 derived families may be
  cooled to `Cold` only when recall or rebuild legality is already declared
- working-set promotion may use explicit observed access windows over admitted
  branch, retained-basis, and artifact-family scopes
- working-set cooling may be driven only by explicit observed inactivity windows,
  not by ambient cache disappearance

Explicit out-of-scope debt for first ship:

- cross-branch global heat balancing that requires whole-store optimization
- predictive prefetch or speculative promotion based on unproven future demand
- scheduler-driven tier mutation that bypasses placement planning
- aggressive cold-authority policies that require authoritative recall before the
  read surface can even name the retained truth lane

Rules:

- unsupported adaptive or predictive policies must remain explicitly marked
  `Debt`
- first-ship conservative placement must be certifiable end to end before more
  aggressive modes are admitted

### Placement Authority Rule

Tiering must classify residency against already-honest authority and derivation
categories before any move or recall is admitted.

Required placement vocabulary:

- `AuthoritativeTierResidency`
- `DerivedTierResidency`
- `TierResidenceClass`
- `TierPlacementEvidence`
- `PlacementBudgetClass`
- `RecallEligibilityWitness`
- `PlacementNonAuthorityWitness`

Rules:

- one placement decision must be explainable back to artifact family, retained
  basis, and authority classification
- tier state may not redefine whether an artifact is authoritative or derived
- residency metadata is proof and diagnostics, not authority
- a missing hot copy is not semantic failure if a legal colder resident or
  rebuildable basis still exists

### Tier Move Protocol Rule

Tier movement must be a typed publish-and-cutover protocol rather than an
ambient copy followed by backend-local cleanup.

Required move protocol surfaces:

- `TierTransferIntent`
- `TransferredTierReplica`
- `VerifiedTierReplica`
- `TierCutoverWitness`
- `RetiredTierReplica`
- `CanonicalResidencyManifest`

Rules:

- movement must proceed through explicit prepare, transfer, verify, cutover, and
  retire phases
- transferred bytes are not admitted as the new resident copy until verify
  succeeds and a `TierCutoverWitness` exists
- cutover updates one canonical residency manifest instead of relying on
  backend-local file presence as truth
- retiring the old resident copy is a later phase than cutover, not part of the
  same untyped side effect
- crash between phases may leave duplicate physical replicas, but never
  ambiguous residency truth

### Working-Set Evidence Rule

Working-set adaptation must be driven by explicit observed demand rather than
ambient backend cache behavior.

Required evidence surfaces:

- `WorkingSetObservationWindow`
- `PlacementDemandSummary`
- `HotnessClassificationVerdict`
- `TierPromotionCandidate`
- `TierCoolingCandidate`

Rules:

- hotness must be explainable in terms of observed access families, branch or
  retained-basis scope, and artifact-family posture
- repeated access may justify promotion or residency retention, but it may not
  change truth semantics
- a heuristic score alone is not enough to move durable authority; the move must
  lower to a typed placement plan
- restart-visible placement state must distinguish fresh observation from stale
  prior classification

### Placement Plan Lowering Rule

Execution must consume one lowered placement plan per move family rather than
re-deciding residency during the move.

Required plan families:

- `AuthoritativeTierMovePlan`
- `DerivedTierMovePlan`
- `RecallPreparationPlan`
- `PlacementStabilityPlan`
- `TierMoveRejection`

Rules:

- planners resolve legality, source residency, target residency, retained-basis
  requirements, and rebuild posture before execution
- executors may not re-run hotness classification or placement policy resolution
  once a lowered plan exists
- authoritative and derived moves must remain distinct plan families even when
  the physical transport mechanism is shared
- rejected plans must carry typed reasons instead of degenerating into backend
  booleans or skipped work

### Recall And Tier-Miss Rule

Cold access must be visible as an explicit lane instead of a hidden change in
read meaning.

Required surfaces:

- `TierMissOutcome`
- `ColdRecallPlan`
- `RecallCompletionWitness`
- `RecallCostClass`
- `RecallDebtSummary`
- `RetainedReadPlacementPath`

Rules:

- retained reads must expose whether the answer came from hot resident data, warm
  resident data, cold recalled data, or rebuild-assisted derived data
- a tier miss may increase cost, but it may not broaden semantic authority or
  silently substitute a different truth source
- recall legality for derived artifacts must respect rebuild and retention rules
  already established by Milestone 10
- recall posture must declare whether the lane is `Inline`, `Bounded`, or
  `Deferred` before execution begins
- recall debt must remain separate from scheduler debt; Milestone 13 names the
  semantic reason while Milestone 11 decides pacing

### Recall Amplification Rule

One cold miss must not silently explode into arbitrary cross-tier work.

Required surfaces:

- `RecallAmplificationBudget`
- `FamilyLocalRecallUnit`
- `BroadenedRecallPlan`
- `RecallCoalescingKey`

Rules:

- one admitted cold miss may touch at most one family-local recall unit unless a
  typed `BroadenedRecallPlan` explicitly widens the lane
- foreground reads may not trigger whole-branch or whole-tier recall through
  ambient fallback
- if widening is required, the widened breadth must be visible in the plan and
  result counters before execution begins
- identical recall requests over the same admitted unit should coalesce through a
  shared `RecallCoalescingKey` instead of duplicating work

### Read-Handle Boundary Rule

Read-path APIs must make placement posture explicit in the type they consume, not
only in the diagnostics they emit afterward.

Required read-boundary surfaces:

- `PlacementBoundArtifactRef`
- `ResidentReadLease`
- `ColdRecallLease`
- `PlacementResolvedReadHandle`
- `SchedulerPlacementWorkToken`
- `PlacementExecutionOrigin`

Rules:

- retained-read entrypoints may not accept raw artifact digests, raw tier ids, or
  raw storage locators as enough proof to read
- resident reads consume a `ResidentReadLease`; colder reads consume a
  `ColdRecallLease` and yield a `PlacementResolvedReadHandle`
- placement-bound handles must carry their allowed execution origin so
  foreground, background, and restart lanes cannot silently share one cost model
- scheduler-owned code may queue or pace placement work only through a
  `SchedulerPlacementWorkToken`; it may not mutate residency directly
- placement reporting must reflect the handle family actually consumed, not a
  best-effort reconstruction after the fact

### Compile-Time Boundary Rule

The highest-risk tiering boundaries must be compiler-enforced rather than left
as doc-only rules.

Required proof-bearing surfaces:

- `TierPlacementPlan`
- `RecallEligibilityWitness`
- `HotnessClassificationVerdict`
- `RetainedReadPlacementPath`
- `PlacementNonAuthorityWitness`

Required compile-time posture:

- tier moves may not consume raw storage paths, raw backend tier ids, or raw
  artifact digests directly; they must consume lowered placement plans
- recall execution may not consume a nullable "maybe present" storage hint as
  proof that colder access is legal; it must consume a `RecallEligibilityWitness`
- working-set reclassification may not directly mutate residency state without
  producing a `HotnessClassificationVerdict`
- read-path reporting may not erase whether data was resident or recalled; it
  must surface a typed `RetainedReadPlacementPath`
- placement metadata may not be accepted as authority proof anywhere truth is
  decided
- retained-read entrypoints may not admit raw locators and then "decide later"
  whether the read was resident or recalled; the lease type must force that
  distinction up front

Required proof surface:

- compile-fail tests for raw-id tier move admission
- compile-fail tests for synthetic recall witness construction
- compile-fail tests for treating placement metadata as retained-truth authority
- compile-fail tests for direct scheduler mutation of placement state without a
  lowered plan
- compile-fail tests for raw-locator retained reads that bypass placement-bound
  leases

### Performance-Shaping Types Rule

Milestone 13 must encode the dominant locality and recall cost decisions into
lowered types so downstream phases consume pre-resolved placement posture.

Required performance-shaping surfaces:

- `PlacementDemandSummary`
- `TierLocalityFootprint`
- `FamilyLocalPlacementPlan`
- `RetainedRangePlacementPlan`
- `TierMoveBreadthSummary`
- `RecallBreadthSummary`
- `WorkingSetDebtSummary`

Required posture:

- access heat and locality are summarized once per admitted observation window
- placement planning lowers to one concrete move family before physical movement
  starts
- locality planning lowers to branch-local, retained-range-local, or
  family-local units before execution
- read results expose whether they remained resident or required recall
- family-local breadth is carried forward instead of rediscovered during move or
  recall execution

Rules:

- execution may not rescan whole history or whole artifact inventories once an
  equivalent trusted breadth summary already exists
- cheap-looking reads may not hide that they triggered cold recall
- planners may not collapse locality scopes into one ambient global placement
  sweep unless that broadened scope is explicit and countered
- placement counters must describe moved and recalled breadth, not just elapsed
  time

### Lowered Work Unit Families Rule

Tier work must be partitioned by semantic family and locality before execution.

Required lowered work unit families:

- `AuthoritativeTierMoveUnit`
- `DerivedTierMoveUnit`
- `SnapshotRecallUnit`
- `DeltaRecallUnit`
- `LayoutFamilyRecallUnit`
- `PlacementObservationUnit`

Rules:

- every work unit is tied to one artifact family and one source or target tier
- authoritative and derived movement must remain distinct even if later
  orchestration batches them together
- observation work units remain separate from move and recall work units
- Milestone 11 may schedule these units, but Milestone 13 defines what they
  mean

### Read And Result Cost Surface Rule

Every tiering decision, recall result, and retained read must expose its
placement posture and cost lane in the result envelope.

Required result surfaces:

- read-path placement classification
- execution-origin classification
- tier move breadth summary
- hotness reclassification count
- tier miss and recall count
- reuse-versus-movement classification
- placement debt delta

Rules:

- a consumer must be able to tell whether the operation stayed hot, stayed warm,
  moved colder, promoted hotter, or required recall
- a consumer must be able to tell whether the work was foreground, background, or
  restart-originated
- end-to-end placement claims are invalid unless the result envelope carries the
  structural counters that justify them
- result surfaces must distinguish `ResidentHit`, `WarmHit`, `ColdRecallHit`,
  `PromotionScheduled`, `PromotionCompleted`, and real move execution

### Complexity-Status Surface Rule

Milestone 13 evidence must publish path-local complexity status rather than one
rolled-up placement verdict.

Minimum named paths:

- `placement_state_reconstruction`
- `working_set_classification`
- `tier_move_planning`
- `tier_move_cutover`
- `tier_move_execution`
- `cold_recall_execution`
- `recall_coalescing`

Rules:

- each path declares at least `Verified` or `Debt`
- any `Debt` path names the unresolved breadth, heuristic exposure, or missing
  proof

Minimum contracts:

- placement state reconstruction cost is proportional to:
  - residency manifest entries loaded
  - unresolved transfer intents recovered
  - not total tier inventory walked from storage
- working-set classification cost is proportional to:
  - observation windows evaluated
  - artifact families summarized
  - locality groups classified
  - not total historical bytes
- tier move planning cost is proportional to:
  - admissible placement candidates evaluated
  - retained-basis or rebuild checks needed
  - family-local locality footprints considered
- tier move execution cost is proportional to:
  - artifacts or placement units moved
  - residency metadata updates emitted
  - not unrelated store breadth
- tier move cutover cost is proportional to:
  - residency manifest entries updated
  - verified replicas promoted
  - retired replicas recorded
  - not total tier inventory
- cold recall execution cost is proportional to:
  - recall units admitted
  - artifacts or families rehydrated
  - retained-basis validations required
  - not total tier inventory
- recall coalescing cost is proportional to:
  - unique coalescing keys admitted
  - duplicate requests suppressed
  - not total waiting readers

Forbidden hidden work:

- full-tier or full-inventory walks during startup just to discover canonical
  residency state
- full-store scans to explain one narrow hot branch reclassification
- silent foreground broadening because a cold tier read fell back to an ambient
  backend search
- one miss widening into multi-family recall without a typed broadened plan
- scheduler-side reinterpretation of tier meaning during execution
- placement metadata substituting for retained authority in read or restore
  lanes

Minimum counters:

- `placement_state_manifest_load_count`
- `placement_state_recovery_count`
- `working_set_observation_window_count`
- `working_set_reclassification_count`
- `hot_tier_resident_read_count`
- `warm_tier_resident_read_count`
- `cold_tier_recall_count`
- `foreground_cold_recall_count`
- `background_tier_move_count`
- `restart_recall_count`
- `tier_move_plan_count`
- `tier_move_cutover_count`
- `tier_move_cutover_rejection_count`
- `authoritative_tier_move_count`
- `derived_tier_move_count`
- `tier_move_rejection_count`
- `tier_miss_count`
- `broadened_recall_plan_count`
- `recall_coalesced_request_count`
- `recall_duplicate_suppression_count`
- `placement_debt_count`
- `working_set_debt_count`
- `tier_truth_parity_failure_count`
- `tier_restore_parity_failure_count`
- `tier_recall_failure_count`

Required counter assertions:

- `placement_state_manifest_load_count` and `placement_state_recovery_count`
  remain proportional to admitted manifest and in-flight transfer state, not raw
  backend inventory breadth
- `tier_truth_parity_failure_count` remains zero for admitted tier-move lanes
- `tier_restore_parity_failure_count` remains zero for admitted recall and
  restart lanes
- `tier_move_cutover_rejection_count` increments when verify correctly refuses to
  promote a transferred replica into canonical residency
- `foreground_cold_recall_count` increments only for admitted foreground recall
  lanes and must stay zero on purely resident-read workloads
- `recall_duplicate_suppression_count` increments when equivalent recall demand is
  coalesced instead of redundantly executed
- `tier_miss_count` increments only when colder-resident access actually broadens
  into an explicit recall path
- `placement_debt_count` and `working_set_debt_count` distinguish incomplete
  heuristic ambition from completed safe placement work

## Phases

### Phase 1: Lock Placement Semantics, Tier Vocabulary, And Concurrency Boundaries

Phase 1 defines what tiering and working-set adaptation are allowed to mean
before any movement or heuristic lane lands.

Required work:

- define hot, warm, and cold residency vocabulary over authoritative and derived
  artifact families
- define first-ship conservative placement classes and explicit adaptive-policy
  debt markers
- define placement evidence, non-authority witnesses, and retained read
  placement paths
- define placement budget classes and recall cost classes for foreground,
  background, and restart-origin lanes
- define working-set observation windows and classification verdict types
- define authoritative versus derived move-plan families
- define typed prepare, transfer, verify, cutover, and retire phases for tier
  movement
- define typed recall eligibility and recall completion witnesses
- define recall amplification budgets and broadened-recall plan families
- define the Milestone 13 versus Milestone 11 boundary for semantics versus
  scheduling
- define explicit first-ship debt markers for advanced adaptive heuristics

Exit condition:

- every tier decision has one exact authority-versus-derivation explanation
- tier residency is mechanically non-authoritative
- cold recall has one typed meaning
- Milestone 11 can schedule tier work without inventing placement semantics

### Phase 2: Observe Demand And Lower Placement Plans

Phase 2 turns locality and access pressure into machine-checkable placement
decisions.

Required work:

- implement observation windows over branch, retained-basis, and artifact-family
  access
- implement working-set demand summaries and hotness classification verdicts
- implement placement candidate planning for authoritative and derived families
- implement typed rejection reasons for illegal or unjustified moves
- implement locality breadth summaries and explicit debt markers for unsupported
  adaptive policies
- implement execution-origin attribution for foreground, background, and restart
  placement work
- implement placement-bound read and recall lease planning rather than
  post-hoc placement inference
- implement recall coalescing and typed broadened-recall planning for any lane
  that must widen past one family-local unit
- emit exact observation, classification, and planning counters

Exit condition:

- working-set classification can explain every move candidate back to observed
  demand and family posture
- raw heuristics are not sufficient to move artifacts directly
- authoritative and derived placement work lower to distinct plan families

### Phase 3: Execute Tier Moves And Explicit Recall Lanes

Phase 3 makes placement physically real without changing truth meaning.

Required work:

- implement authoritative artifact movement across admitted tiers
- implement derived artifact movement and residency refresh across admitted tiers
- implement typed move prepare, transfer, verify, cutover, and retire phases
- maintain one canonical residency manifest through cutover
- implement explicit cold recall for snapshots, deltas, and layout families
- enforce miss-amplification budgets and coalesced recall execution
- persist placement evidence and residency metadata with restart-safe posture
- expose typed move rejection, recall failure, and locality mismatch failures
- emit exact move, miss, and recall counters

Exit condition:

- artifacts can move or be recalled without altering replay, restore, or branch
  truth
- colder placement remains observable as cost, not semantic drift
- restart after partial movement never leaves ambiguous residency truth or
  requires backend-local file inspection to decide the live copy

### Phase 4: Publish Placement-Aware Read Surfaces And Scheduler Handoff

Phase 4 turns tiering into an honest store surface rather than an internal
optimization.

Required work:

- expose placement-aware read-path reporting on retained reads and restore lanes
- require placement-bound read handles and explicit recall leases on public
  read-path entrypoints
- expose placement debt and working-set debt as operator-visible evidence
- expose execution-origin and reuse-versus-movement result classification
- define family-local placement work units suitable for Milestone 11 scheduling
- preserve scheduler-independent semantic meaning for tier moves and recalls
- implement typed handoff surfaces so pacing policy consumes placement work
  without redefining it
- certify that advisory placement remains distinct from scheduler policy

Exit condition:

- callers can tell whether a read stayed resident or required recall
- operators can observe placement debt without inferring semantic safety from
  scheduler state
- Milestone 11 has a typed work boundary to consume concurrently

### Phase 5: Prove Tiering And Working-Set Non-Authority

Phase 5 turns tiering into a certifiable store surface rather than an
implementation detail.

Required work:

- run the Milestone 13 named suite:
  `Tiering And Working-Set Non-Authority Test`
- compare pre-move and post-move replay, restore, and branch-visible truth
- compare resident-read and recalled-read lanes for the same retained basis
- include crash points between move prepare, transfer, verify, cutover, and
  retire
- include interleavings where live-query continuation or foreground reads race
  tier movement without changing truth meaning
- include duplicate-demand lanes that prove coalesced recall stays exact
- include restart lanes that reconstruct residency from manifests rather than
  walking tier contents
- emit machine-checkable truth, artifact, diagnostics, and counter bundles

Exit condition:

- tier movement changes placement only
- working-set adaptation remains advisory
- no hidden eviction of authoritative truth occurs
- Milestone 13 closeout evidence exists in machine-checkable form

## Must Ship

- explicit hot, warm, and cold residency vocabulary over authoritative and
  derived artifact families
- concrete first-ship conservative placement policies with explicit adaptive
  `Debt` boundaries
- typed working-set observation windows and hotness classification verdicts
- lowered authoritative and derived tier-move plans
- crash-safe move prepare/transfer/verify/cutover/retire surfaces with one
  canonical residency manifest
- explicit cold-recall planning and completion witnesses
- typed placement budget and recall cost classes
- placement-bound read handles and explicit recall leases
- recall amplification budgets, broadened-recall plans, and coalescing surfaces
- placement-aware retained read path reporting
- execution-origin and reuse-versus-movement result classification
- family-local placement work units suitable for scheduler handoff
- compile-fail boundary coverage for plan-only movement, typed recall, and
  non-authority enforcement
- typed placement, recall, and reclassification failures
- exact placement, miss, recall, and debt counters
- machine-checkable Milestone 13 certification output

## Must Preserve

- canonical authoritative truth remains authoritative across all tiers
- tiering and working-set decisions remain advisory and derived
- colder placement changes cost only, not replay, restore, or branch meaning
- recall never broadens semantic authority beyond the retained basis already
  promised
- scheduler pacing does not redefine tier semantics
- later replication, blob, and budget systems inherit one placement vocabulary
  instead of inventing their own

## Acceptance Evidence

Milestone 13 is complete only when the store satisfies the named Milestone 13
suite:

- `Tiering And Working-Set Non-Authority Test`

Required machine-checkable outputs:

- `truth_digest`
- `artifact_digest`
- `diagnostics_digest`
- `counter_snapshot`

Milestone-specific proof obligations:

- tier movement does not change replay, restore, or branch truth
- working-set adaptation remains advisory rather than semantic
- authoritative truth is never hiddenly evicted or replaced by placement
  metadata
- restart and partial movement preserve one exact placement meaning
- recalled lanes converge to the same truth-visible result as resident lanes
- placement evidence and counters are sufficient to explain cost posture
- move cutover remains crash-safe and never requires backend-local presence
  guessing to determine the canonical resident copy
- raw locators and ambient fallback cannot bypass the placement-bound read model
- startup reconstruction of placement state remains manifest-bounded rather than
  inventory-scan-based
- one cold miss cannot silently widen into multi-family recall without an
  explicit broadened plan

Milestone 13 is not closed by "latency went down" or "cold data was fetched
successfully" tests.

## Architectural Notes

- The smart abstraction is not "cache management." The smart abstraction is one
  placement authority model plus one explicit recall story layered over already
  honest retained truth.
- First-ship value comes from explicit residency posture and typed recall, not
  from ambitious adaptive heuristics.
- Milestone 13 should publish placement work classes and debt surfaces that
  Milestone 11 can schedule, but it must not make scheduler policy part of
  placement semantics.
- Blob tiering should later inherit the same residency and recall vocabulary
  rather than creating a second store-local lifecycle.

## Sequencing Notes

This milestone belongs after Milestone 10 because placement cannot be made
honest until retention, compaction, reclaim, and rebuild rules are already
stable.

- `Milestone 11` should proceed concurrently because operational pacing and
  isolation can be developed in parallel once Milestone 13 freezes placement
  work classes, residency meaning, and recall posture; Milestone 13 must not
  wait for full scheduler policy to make placement semantics exact.
- `Milestone 14` and `Milestone 20` should inherit this milestone's tier and
  recall vocabulary rather than inventing replication-local or blob-local
  placement semantics.
- `Milestone 21` depends on this milestone because budget controls need a real
  placement and working-set model to govern.
