# Milestone A Engineering Spec: Temporal Runtime Substrate

> **Status:** Closeout candidate
>
> **Closeout:** [milestone-a-closeout.md](./milestone-a-closeout.md)
>
> **Roadmap parent:** [worth_signal_temporal_async_roadmap.md](./worth_signal_temporal_async_roadmap.md)
>
> **Vision parents:**
> - [worth_signals2.md](./worth_signals2.md)
> - [worth_signal_vision.md](./worth_signal_vision.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Prerequisite milestone:**
> - [milestone-11-closeout.md](./milestone-11-closeout.md)
>
> **Primary architectural driver:** make time a runtime-owned semantic axis so
> temporal eligibility, scheduled wakes, and previous-value-sensitive
> computation stop depending on ambient host condition callbacks

## Summary

Milestone A makes time a first-class runtime primitive in `worth-signal`.

This milestone is not "add a few more condition variants."

It is:

- runtime-owned temporal eligibility
- explicit clock and clock-basis semantics
- first-class temporal policy families instead of callback folklore, including
  `After`, `AtOrAfter`, `Debounce`, `Throttle`, `StaleAfter`, and `Interval`
- scheduled wake ownership and deterministic ready ordering
- temporal invalidation as first-class runtime truth
- previous-value-sensitive evaluation on transactional footing
- branch-, restore-, and replay-honest temporal state
- diagnostics-visible temporal provenance
- bounded temporal execution with named counters and proof obligations

The governing rule is:

`admit temporal meaning once, lower it once, wake it once, replay it honestly`

If temporal readiness is still fundamentally decided by host callback folklore
after this milestone, the milestone is incomplete.

## 1. Goal

Make time a first-class, runtime-owned capability of `worth-signal` so that:

- temporal gating is canonical runtime truth
- hosts supply clock input without defining temporal meaning
- previous-value-sensitive nodes can evaluate against transactional history
- branch, restore, replay, and diagnostics preserve one temporal story

## 2. Why This Milestone Exists

`worth-signal` already owns conditional execution, but it does not yet own time
as a real semantic axis.

Today the crate has:

- `OnDemand`
- `Debounce(...)`
- custom conditions
- low-level runtime timing helpers

But it still lacks:

- runtime-owned clock domains
- a frozen temporal policy vocabulary with the core time primitives the vision
  will actually need
- runtime-owned scheduled wake state
- deterministic temporal admission artifacts
- previous-value access as an explicit runtime capability
- branch/restore/replay semantics for temporal eligibility

Without this milestone:

- debounce remains partially host-defined
- future timeout, stale-after, and retry windows have no honest substrate
- previous-value logic drifts into ad hoc node-local tricks
- later async/resource work will inherit a weak temporal model

## 3. Hard Part

The hard part is not reading a clock.

The hard part is freezing one exact truth-preserving relationship among:

- host-supplied time input
- runtime-owned clock basis
- clock-domain selection and admissibility
- temporal eligibility rules
- scheduled wake storage and ready ordering
- wake retirement and rescheduling
- previous committed value access
- branch, restore, and replay reconstruction
- diagnostics and explanations of why work became eligible

The design fails if:

- hosts can redefine temporal meaning after node admission
- two equivalent runs with the same clock basis produce different wake order
- previous-value reads can observe staged-but-uncommitted state
- branch restore leaks ambient process time into restored temporal truth
- timer handling broadens into whole-graph polling
- large clock jumps cause interval catch-up explosions or unbounded replay work
- reschedule and retirement paths hide broad scans behind tiny time advances
- wake handling relies on per-wake heap allocation churn in hot paths

## 4. Explicit Assumptions

- `worth-relational` remains the owner of truth identity, mutation, history,
  diffs, and traversal.
- `worth-signal` remains the owner of derived execution truth, not authority.
- hosts may provide clock inputs or advance requests, but the runtime owns how
  those inputs affect eligibility.
- the runtime must distinguish monotonic execution time from any host wall-clock
  or presentation time.
- Milestone A is core-only; wasm, React, route-resource, and form ergonomics
  remain out of scope.
- Milestone 11 observation guarantees remain product contracts and may not be
  weakened by temporal work.

## 5. Governing Summaries

- `MENTALITY.md`
  The most important thing it protects here is solving the hostile constraint
  first. Temporal support must start from replay-honest eligibility and bounded
  wake handling, not from "debounce is easy to expose."
- `arch_laws.md`
  The most important laws here are 17, 18, 22, 27, 30, 33, 36, 37, and 41.
  Temporal policy resolution must be isolated from execution, time-aware
  observation must remain phase-correct, shared temporal facts must lower once,
  the executor must consume lowered temporal plans, checkpoint-plus-bounded-
  journal reconstruction must remain possible, invalid lifecycle states must be
  unrepresentable, and each phase must carry proof-bearing temporal types.
- `perf_laws.md`
  The most important thing it protects is breadth honesty. Time cannot become a
  cheap-looking API that secretly polls the whole graph or rescans all pending
  nodes on every advance.
- `domain_laws.md`
  The most important thing it protects is subsystem shape. Temporal state,
  scheduling, replay support, and diagnostics need their own clean subsystem
  boundaries rather than being smeared across helpers or convenience modules.
- `worth_signals2.md`
  The most important thing it protects is the product thesis that temporal and
  previous-value signal support are first-class runtime capabilities, not
  adapter-local conveniences.
- `worth_signal_vision.md`
  The most important thing it protects is the authority boundary:
  `worth-signal` owns derived execution semantics and must remain standalone,
  deterministic, transactional, and auditable.
- `worth_signal_temporal_async_roadmap.md`
  The most important thing it protects is sequencing. Time lands before async
  because async lifecycle truth needs a runtime-owned temporal substrate for
  deadlines, staleness windows, retries, and replayable ordering.
- `test-requirements.md`
  The most important thing it protects is certification. Temporal support is
  not closed until replay parity, branch restore parity, bounded wake handling,
  and previous-value honesty are all machine-checked.
- `milestone-11-closeout.md`
  The most important thing it protects is continuity. Temporal work must extend
  the same transaction, observation, diagnostics, and lifecycle rigor that the
  runtime-local observation subsystem now already enforces.

## 6. Adversarial Constraint

Milestone A must survive the following hostile condition:

> A branchable, replayable runtime with deterministic execution, rollback-safe
> observation, time-gated nodes, and previous-value-sensitive nodes must
> produce the same temporal eligibility decisions, the same wake ordering, the
> same committed outputs, and the same explanations regardless of whether work
> was driven by direct invalidation, logical time advance, branch restore, or
> replay from checkpoint plus bounded temporal history.

Concretely, the design must remain correct when all of the following are true:

- multiple temporal wakes are pending at once
- branch fork happens before scheduled wakes become ready
- restore returns to a point before and after temporal admission
- long gaps occur with no host writes and only time advance can make work ready
- previous-value-sensitive nodes sit near temporal threshold boundaries
- diagnostics tier changes between equivalent runs

If any supported path falls back to host-decided temporal truth, ambient
process time leakage, broad timer scans, or staged previous-value leakage under
those conditions, the milestone has failed.

## 7. Product Decision Lock

- time is a first-class core runtime capability, not a host-side convention
- hosts may provide time input, but may not define temporal eligibility
  semantics ad hoc after registration
- temporal policy families are sealed runtime vocabulary in this milestone, not
  open-ended strategy plugins
- the minimum first sealed family for this milestone explicitly includes
  `After`, `AtOrAfter`, `Debounce`, `Throttle`, `StaleAfter`, and `Interval`
- the runtime owns at least one canonical monotonic clock basis; wall-clock
  correlation is metadata, not eligibility authority
- temporal state is derived runtime truth, not authority
- previous-value access is a runtime-owned transactional capability, not a
  node-local cache trick
- temporal eligibility, scheduled wakes, and temporal provenance must all be
  branch- and replay-honest
- recurring wake generation belongs to the temporal substrate in this
  milestone and may not be postponed to the async phase
- temporal support must preserve commit-bounded observation and rollback
  semantics

Normative consequence:

- any implementation that still treats `Debounce(...)` primarily as "ask the
  host whether it is ready" is out of spec
- any implementation that lets wall-clock time directly decide replay-critical
  eligibility without a recorded canonical basis is out of spec
- any implementation that lets restored or replayed branches consult ambient
  process time directly is out of spec
- any implementation that lets previous-value-sensitive logic observe
  uncommitted intermediate state is out of spec
- any implementation that can leave stale scheduled wakes alive after node
  replacement, policy rewrite, branch restore, or graph disposal is out of spec
- any implementation that widens temporal handling into graph-wide polling is
  out of spec
- any implementation that turns one large clock jump into unbounded interval
  catch-up work without an explicit missed-tick policy is out of spec
- any implementation that rebuilds all branch-local wake state by whole-registry
  scan on restore is out of spec

## 8. Scope

### 8.1 In Scope

- runtime-owned clock abstraction and clock basis vocabulary
- sealed temporal policy families and migration of legacy debounce semantics,
  with explicit support for `After`, `AtOrAfter`, `Debounce`, `Throttle`, and
  `StaleAfter`, plus recurring `Interval`
- temporal eligibility artifacts
- scheduled wake storage and ready selection
- wake retirement, cancellation, and rescheduling semantics
- temporal invalidation causes
- previous-value access semantics sufficient for time-sensitive nodes
- branch/snapshot/replay temporal reconstruction
- temporal diagnostics and explanation surfaces
- public core APIs for honest time advance or time input submission
- named counters and complexity contracts for temporal work

### 8.2 Explicitly Out Of Scope

- wasm bindings
- React or Angular surfaces
- route-resource APIs
- async/resource node lifecycle
- network transport semantics
- domain-specific time policy families beyond what the substrate needs to be
  honest

## 9. Current-State Assessment

The runtime is already structurally ready for this milestone in several ways:

- the crate already owns conditional execution and condition evaluation
- the transaction/runtime architecture already owns rollback, observation,
  diagnostics, replay-facing state, and branch-aware machinery
- the crate already has deterministic execution as a product contract
- the runtime already separates authority from derivation correctly

However, the missing temporal category is still real:

- `EvaluationCondition` exposes `Debounce(...)`, but temporal readiness is not
  runtime-owned truth yet
- `ConditionResolver` still acts as the real temporal admission authority for
  debounce-like behavior
- the public condition vocabulary still lets time-shaped meaning hide inside
  `Custom(...)`
- `clock.rs` is not yet a semantic runtime clock subsystem
- there is no first-class scheduled temporal wake state
- previous-value-sensitive execution is not yet frozen as a runtime contract
- temporal replay and branch-restore parity are not yet explicit product
  surfaces

This means the runtime has timing-adjacent pieces, but not yet an honest time
architecture.

## 10. Architecture Rules For This Milestone

### 10.1 Time Is A Runtime Subsystem, Not A Condition Helper

Temporal state must be modeled as a first-class runtime subsystem with owned
state, lifecycle, and facade access. It must not be implemented as:

- a thin extension of `ConditionResolver`
- ad hoc timestamps stored directly on random node records
- callback-time polling over node conditions

Required consequence:

- `SignalRuntime` gains an owned temporal subsystem
- builder and facade surfaces expose temporal policy configuration explicitly

### 10.2 Clock Input And Temporal Meaning Are Separate

Hosts may provide clock input. The runtime owns what that input means.

Acceptable:

- host advances a monotonic clock basis
- host supplies a checkpointed time input envelope
- runtime lowers registered temporal policies against that basis

Not acceptable:

- host callback says whether a node is "ready" without runtime-owned evidence
- node-local closures reinterpret time independently
- replay and restore depend on ambient wall clock instead of recorded basis

### 10.2.1 Clock Domains Must Be Explicit And Ranked

This milestone must make clock-domain meaning concrete enough that engineers
cannot accidentally mix replay-safe time with presentation time.

Required consequence:

- a canonical monotonic execution clock is required
- any wall-clock or presentation clock must be modeled as secondary metadata or
  explicitly non-authoritative input
- a temporal policy must declare which admissible clock domain it consumes
- replay-critical eligibility may not depend on an undeclared or ambient clock

### 10.3 Temporal Resolution Must Precede Wake Execution

Temporal policy selection and eligibility derivation must happen before the
execution path evaluates nodes.

Required consequence:

- one lowering pass derives temporal eligibility from runtime facts
- the executor consumes lowered temporal readiness artifacts
- diagnostics consume committed temporal artifacts or retained summaries rather
  than re-deciding eligibility later

### 10.3.1 Wake Lifecycle Must Be Framework-Owned

Scheduled wakes are managed runtime resources and must obey framework-owned
lifecycle semantics.

Required consequence:

- a wake cannot outlive the node registration or branch lineage that created it
- node replacement, condition rewrite, restore, and disposal paths must retire
  or supersede prior wakes structurally
- a completed or retired wake may not be re-admitted by stale references later
- recurring wakes must regenerate only through explicit runtime-owned interval
  policy, not by callback-local rearming

### 10.4 Previous-Value Access Must Be Transactional

Previous-value-sensitive evaluation must mean "last committed value on this
branch under this checkpoint lineage," not "whatever happened to be seen last."

Required consequence:

- previous-value access is phase-typed
- rollback suppresses staged previous-value mutation
- restore and replay reconstruct the same previous-value references the
  original history had

### 10.4.1 Previous-Value Readability Must Be Capability-Shaped

Previous-value access should be impossible from contexts that have not earned
it.

Required consequence:

- only temporal-evaluation contexts with the correct proof type can access
  previous committed value references
- ordinary condition registration or arbitrary host callback surfaces cannot
  reach previous-value state directly

### 10.5 Temporal State Must Be Checkpoint-Plus-Journal Reconstructable

Temporal truth must remain reconstructable from a checkpoint plus bounded
temporal history.

Required consequence:

- scheduled wake state is serializable or reconstructable from retained
  artifacts
- branch restore does not require ambient process state to recover timer truth
- replay can canonicalize the same eligibility and ready ordering story

### 10.5.1 Temporal Truth Must Flow Through One Canonical Artifact

Milestone A must preserve the runtime rule that one committed boundary produces
one canonical truth artifact from which all temporal views are derived.

Required consequence:

- time-driven admissions and deferrals are recorded in canonical transaction or
  checkpoint-visible artifacts
- replay, diagnostics, and observation derive temporal truth from the same
  canonical record rather than recomputing separate stories

### 10.6 Temporal Work Must Stay Breadth-Bounded

Ready-node discovery must consume an explicit temporal ownership and lookup
structure. It must not depend on broad rescans.

Required consequence:

- temporal registration lowers into runtime-owned wake/index structures
- clock advance and ready selection operate over the temporal frontier
- long idle periods do not induce whole-graph temporal polling

### 10.6.1 Large Time Jumps Must Be Policy-Bounded

The runtime must remain honest when time advances by much more than one
eligibility quantum.

Required consequence:

- interval semantics must define whether missed periods collapse, catch up, or
  keep only the latest eligible wake
- the runtime may not silently iterate one period at a time across arbitrarily
  large elapsed spans unless that exact cost model is explicitly admitted and
  proven bounded for the chosen policy
- replay and restore must preserve the same missed-tick decision, not merely
  the same final output

### 10.6.2 Allocation Churn Must Be A Named Design Concern

Temporal support will look cheap in small tests even if it allocates on every
wake, reschedule, or restore path. The spec must forbid that drift early.

Required consequence:

- wake scheduling, retirement, and ready promotion should prefer pre-allocated,
  arena-backed, or otherwise lifecycle-bounded structures
- hot-path temporal work may not rely on repeated per-wake heap allocation as
  its ordinary execution model
- counters must make wake allocation or reuse posture attributable

## 11. Required Architecture Changes

### 11.1 Add A Dedicated Temporal Subsystem

Add a runtime-owned temporal subsystem under
`logic/transaction/runtime/state`.

It should own:

- clock basis and clock-domain state
- temporal policy registry for sealed built-ins
- scheduled wake registry
- deterministic wake ordering metadata
- temporal counters and diagnostics support

It should also make one structural storage decision explicit:

- wake lookup, ready ordering, and branch-local temporal state may not be one
  undifferentiated map if that would force broad restore or broad readiness
  scans later

It must not own:

- truth authority
- async/resource lifecycle
- frontend or transport policy

### 11.2 Introduce Temporal Policy And Proof Types

Add explicit types for the temporal pipeline, with exact names allowed to
evolve.

At minimum the architecture should preserve distinct forms for:

- clock input intent
- validated clock advance request
- runtime-owned clock basis
- temporal policy declaration
- temporal condition intent
- frozen temporal descriptor
- scheduled temporal wake
- ready temporal wake
- retired or superseded temporal wake
- recurring wake policy metadata
- temporal evaluation cause
- previous-value reference
- committed temporal explanation artifact

The milestone should also prefer semantic newtypes wherever raw primitives
would blur meaning. This includes, at minimum:

- `ClockDomainId`
- `ClockTick`
- `ClockAdvanceOrdinal`
- `IntervalPeriod`
- `IntervalAnchor`
- `MissedTickPolicyId`
- `TemporalWakeId`
- `WakeOrdinal`
- `TemporalCauseId`
- `ClockCheckpointId`
- `PreviousValueRevision`

### 11.3 Lower Conditions Through Temporal Runtime Forms

Temporal conditions must stop flowing through open-ended host readiness paths.

Required consequence:

- registration intent lowers into runtime-owned temporal descriptors
- legacy `Debounce(...)` and time-shaped condition cases migrate through the
  same lowering path as any broader first-class temporal policy family
- the sealed family in this milestone must concretely cover:
  - relative delay / `After`
  - absolute threshold / `AtOrAfter`
  - quiet-period coalescing / `Debounce`
  - rate limiting / `Throttle`
  - freshness expiry / `StaleAfter`
  - recurring wake generation / `Interval`
- scheduled wake bookkeeping consumes those lowered descriptors
- commit/evaluation paths consume ready or deferred temporal proofs rather than
  rediscovering time semantics from raw condition declarations

### 11.4 Stage Temporal Facts In Transactions

`TransactionScratch` needs a temporal lane rather than forcing later phases to
rediscover what time did.

Expected additions include:

- staged temporal eligibility facts
- staged ready-wake sets
- staged wake retirements and supersessions
- staged recurring wake regeneration decisions
- staged previous-value references
- counters for temporal classification and wake handling work

The staged forms must be phase-distinct and proof-bearing.

The compile-time bias here should be explicit:

- raw wake ids do not cross phase boundaries without proof wrappers
- only the lowering phase can construct ready-wake proofs
- only the commit/replay integration phase can construct committed temporal
  explanation artifacts

The performance bias here should also be explicit:

- transaction staging should carry enough temporal summary that later phases do
  not rediscover due wakes, missed-tick decisions, or prior readiness facts
- branch restore should consume retained temporal summaries or indexed state,
  not rebuild timing truth from raw node conditions

### 11.5 Add Branch / Replay / Diagnostics Temporal Artifacts

Temporal work must flow into the same trust surfaces as the rest of the
runtime.

Required consequence:

- diagnostics can explain why a node was deferred or admitted by time
- replay can compare canonical temporal digests
- branch and restore paths preserve temporal state honestly
- the public facade does not expose internal temporal subsystem types directly;
  it exposes capability-shaped requests and summaries only

## 12. Milestone Phases

### Phase 1: Temporal Contract Freeze

Deliver:

- the core type vocabulary for temporal runtime state
- the subsystem boundary
- the product-decision lock encoded in docs and public naming
- the high-level phase model for temporal runtime progression

Must prove:

- host clock input and runtime temporal meaning are non-overlapping categories
- temporal semantics are no longer spec-shaped as "custom condition folklore"
- the milestone sequence is explicit enough that later implementation cannot
  collapse structural steps back into one convenience pass

### Phase 2: Clock Basis And Domain Semantics

Deliver:

- canonical monotonic execution clock semantics
- explicit clock-domain vocabulary
- admissibility rules for authoritative versus non-authoritative clock domains
- validated clock advance request forms

Must prove:

- replay-critical eligibility cannot depend on undeclared ambient clocks
- wall-clock or presentation-time inputs cannot silently become authority
- clock-basis identity survives branch, restore, and replay boundaries

### Phase 3: Sealed Temporal Policy Family

Deliver:

- the initial sealed temporal policy family for this milestone
- explicit canonical semantics for:
  - `After`
  - `AtOrAfter`
  - `Debounce`
  - `Throttle`
  - `StaleAfter`
  - `Interval`
- interval anchor and missed-tick policy semantics

Must prove:

- the first sealed family is explicit and product-committed rather than left as
  an implementation guess
- interval semantics are explicit enough that engineers cannot silently invent
  their own missed-tick behavior later
- legacy debounce semantics have a defined migration target instead of an
  ambiguous compatibility story

### Phase 4: Temporal Proof Types And Capability Surfaces

Deliver:

- compile-time-visible proof-bearing temporal forms
- capability-shaped access for previous-value-sensitive evaluation
- phase-distinct wake and clock request types

Must prove:

- compile-time-visible phase types exist for at least clock advance request,
  scheduled wake, ready wake, retired wake, and previous-value access
- illegal previous-value access is structurally unrepresentable outside the
  temporal evaluation lane

### Phase 5: Wake Storage And Frontier Indexing

Deliver:

- wake retirement and supersession substrate
- scheduled wake registry
- deterministic ready ordering
- runtime-owned lookup structures for the temporal frontier

Must prove:

- clock advance and ready selection do not require graph-wide scans
- equivalent ready sets canonicalize identically
- the temporal frontier is represented architecturally rather than recovered by
  broad polling

### Phase 6: Interval Regeneration And Wake Lifecycle

Deliver:

- interval regeneration substrate
- explicit retirement, supersession, and reschedule ownership
- recurring wake regeneration policy application
- lifecycle rules for node replacement, condition rewrite, restore, and
  disposal

Must prove:

- wake ownership is framework-owned and branch-aware
- stale wakes cannot survive structural lifecycle changes
- interval wake regeneration is replay-honest under large time jumps
- large time jumps do not degrade into hidden per-period replay loops unless the
  selected missed-tick policy explicitly requires and bounds that behavior

### Phase 7: Temporal Eligibility Lowering And Execution Admission

Deliver:

- temporal descriptor lowering from node conditions
- deferred-by-time and ready-by-time proof types
- temporal invalidation causes
- execution-path consumption of lowered temporal artifacts
- migration of legacy debounce semantics into the sealed temporal policy family
- interval lowering with explicit period, anchor, and missed-tick policy

Must prove:

- the executor consumes lowered temporal facts instead of rediscovering them
- temporal readiness is runtime-owned rather than callback-owned
- time-shaped meaning can no longer hide inside open-ended custom readiness
  paths

### Phase 8: Transaction Staging And Previous-Value Semantics

Deliver:

- `TransactionScratch` temporal lanes
- staged temporal eligibility facts
- staged ready-wake sets
- staged wake retirements and supersessions
- staged recurring wake regeneration decisions
- previous-value reference types
- branch- and rollback-safe previous-value access
- time-gated evaluation support that can depend on committed prior state
- capability-shaped evaluation context that makes illegal previous-value access
  unrepresentable outside the temporal lane

Must prove:

- later phases do not rediscover due wakes, missed-tick decisions, or prior
  readiness facts
- previous-value reads cannot see uncommitted state
- rollback preserves previous-value honesty
- threshold-boundary behavior replays identically

### Phase 9: Branch Restore And Replay Integration

Deliver:

- branch-local temporal restore substrate
- temporal state integration with branch/restore/replay
- canonical temporal commit or checkpoint artifact shape
- interval replay and restore evidence

Must prove:

- branch restore does not rebuild temporal readiness by whole-registry scan
- replay and restore preserve one temporal story
- branch-local temporal state stays isolated and reconstructable

### Phase 10: Diagnostics, Facade, And Certification Surface

Deliver:

- diagnostics-visible temporal provenance
- public core facade for honest time advance or time input submission
- counters and complexity-contract documentation for temporal work
- cost-honest public temporal summaries

Must prove:

- the public temporal surface is cost-honest and does not disguise broad
  reschedule, restore, or catch-up work behind a cheap-looking call
- diagnostics richness does not alter temporal truth
- hosts can supply time without redefining time semantics

### 12.1 Phase Ordering Rationale

The ordering is intentionally strict.

- `Phase 1` freezes the boundary so implementation cannot slide back into a
  vague "better debounce" effort.
- `Phase 2` comes before policy work because temporal policy meaning is
  dishonest until the runtime knows which clock basis is authoritative.
- `Phase 3` freezes the sealed temporal family before lowering or storage work
  so later phases do not quietly invent policy semantics during implementation.
- `Phase 4` defines the proof-bearing and capability-shaped forms before hot
  storage and execution work, so illegal state transitions can be made
  unrepresentable instead of merely discouraged.
- `Phase 5` builds the frontier-indexed wake substrate only after clock and
  policy semantics are frozen, so the indexing model reflects real temporal
  meaning rather than placeholder assumptions.
- `Phase 6` lands recurring wake lifecycle only after one-shot wake storage and
  frontier ownership exist, because `Interval` is a lifecycle extension over
  the same substrate rather than a separate mini-system.
- `Phase 7` lowers node declarations into temporal execution truth only after
  clock semantics, policy vocabulary, proof types, and wake ownership all
  exist.
- `Phase 8` stages temporal facts and previous-value-sensitive evaluation only
  after the runtime can already express admitted temporal eligibility honestly.
- `Phase 9` integrates branch restore and replay after the runtime has real
  temporal artifacts worth restoring and replaying.
- `Phase 10` comes last because diagnostics, facade exposure, and certification
  must rest on the already-frozen runtime truth rather than defining it.

If any future edit tries to merge non-adjacent phases, it must prove that no
real structural dependency is being hidden by that compression.

## 13. Must Ship

Milestone A is not done because `Debounce(...)` got prettier.

It is done only when `worth-signal` ships:

- a runtime-owned temporal subsystem
- explicit clock basis and clock-domain vocabulary
- a sealed first-class temporal policy family, not only a prettier debounce
- explicit first-class support for `After`, `AtOrAfter`, `Debounce`,
  `Throttle`, `StaleAfter`, and `Interval`
- first-class scheduled wake ownership and deterministic ready ordering
- wake retirement and supersession semantics
- lowered temporal eligibility artifacts
- transactional previous-value access for time-sensitive nodes
- branch/restore/replay-aware temporal state
- diagnostics-visible temporal provenance
- public core APIs for honest time advance or clock input submission
- named counters and complexity contracts for temporal work

### 13.1 Required Named Test Families

- `temporal_eligibility_replay_parity`
- `temporal_branch_restore_equivalence`
- `temporal_wake_boundedness`
- `previous_value_time_gated_equivalence`

These families are the owning implementation lanes for the corresponding
temporal substrate requirements now declared in
[`test-requirements.md`](./test-requirements.md), especially:

- `11. The temporal eligibility replay parity test`
- `12. The temporal branch restore equivalence test`
- `13. The temporal wake boundedness test`
- `14. The previous-value and time-gated node equivalence test`

### 13.2 Hostile Conditions Required In Certification

- multiple pending wakes in one runtime
- branch fork before readiness
- node replacement or condition rewrite while wakes are pending
- restore to checkpoints before and after temporal admission
- long idle periods with only time advance
- threshold-boundary oscillation for previous-value-sensitive nodes
- throttle and debounce oscillation under bursty invalidation
- stale-after expiry without upstream writes
- large time jumps across many interval periods
- interval replay after restore with mixed missed-tick policies
- diagnostics-tier variation across equivalent runs

## 14. Must Preserve

- deterministic execution remains a product contract
- commit-bounded observation remains unchanged
- rollback remains hard rewind rather than best-effort cleanup
- authority stays outside `worth-signal`
- one canonical temporal truth artifact remains the source for diagnostics,
  replay, and observation-derived views
- hosts may supply clock input but may not define temporal meaning after
  admission
- temporal richness in diagnostics may vary by policy, but temporal truth may
  not

## 15. Performance Contracts

The milestone must expose named counters for at least:

- temporal wake count
- deferred-by-time count
- ready queue width
- retired wake count
- rescheduled wake count
- interval wake regeneration count
- missed interval count
- temporal eligibility lowering count
- previous-value reference count
- branch-local temporal restore count
- temporal replay parity check count
- temporal broad-scan denial count
- wake allocation count
- wake reuse count
- branch-restore temporal rebuild denial count

The milestone must also declare named complexity contracts for:

- temporal registration lowering
- clock advance
- ready-node selection
- interval regeneration
- wake retirement and reschedule
- previous-value lookup
- branch restore of temporal state
- diagnostics-time temporal explanation expansion

Each contract must name its real cost bases explicitly. At minimum:

- clock advance cost must be stated in terms of temporal frontier width, not
  total graph size
- ready selection cost must be stated in terms of ready wake width and wake
  structure maintenance
- wake retirement cost must be stated in terms of retired or superseded wake
  footprint, not ambient graph breadth
- interval regeneration cost must be stated in terms of due recurring wakes and
  missed-tick handling, not total historical periods elapsed since origin
- wake retirement and reschedule cost must be stated in terms of affected wake
  footprint, not total registered temporal nodes
- branch restore cost must be stated in terms of restored branch-local temporal
  state and retained summaries, not total live temporal registry breadth
- previous-value lookup cost must be stated in terms of committed lineage
  access, not ambient graph traversal
- diagnostics expansion cost must be explicitly separated from operational
  readiness cost

### 15.1 Named Temporal Performance Failure Modes

Milestone A should name the failure modes it intends to prohibit so later work
cannot accidentally reintroduce them under nicer names.

At minimum:

- `TemporalBroadScan`
  A clock advance, restore, or reschedule path scans the whole temporal
  registry when the ready or affected frontier is smaller.
- `IntervalCatchUpExplosion`
  A large elapsed time jump creates work proportional to missed periods rather
  than to the semantically admitted missed-tick policy outcome.
- `WakeAllocationChurn`
  Ordinary wake scheduling, retirement, or regeneration relies on repeated
  heap allocation instead of lifecycle-bounded structures.
- `BranchRestoreTemporalRebuild`
  Restoring a branch reconstructs temporal readiness by broad rediscovery over
  raw node conditions rather than by retained temporal state.
- `RescheduleBreadthLeak`
  Changing one node's temporal policy or replacing one node induces broad
  reschedule work unrelated to that node's owned wake footprint.

## 16. Acceptance Evidence

Milestone A is complete only when `worth-signal` can certify all of the
following with canonical machine-checkable artifacts:

- the `Temporal Eligibility Replay Parity Test`
- the `Temporal Branch Restore Equivalence Test`
- the `Temporal Wake Boundedness Test`
- the `Previous-Value And Time-Gated Node Equivalence Test`

The certification bundle must include canonical digests for:

- clock checkpoints
- clock-domain declarations
- scheduled wake sets
- ready ordering
- retired and superseded wake sets
- interval regeneration decisions
- temporal eligibility decisions
- previous-value references
- committed outputs
- branch/restore temporal state
- diagnostics/explanation artifacts

## 17. Architectural Notes

- Milestone A should prefer sealed built-in temporal policy families first.
  In this milestone, that family should be broad enough to freeze the core
  architectural shape rather than only to rescue `Debounce(...)`.
- The first sealed family is an explicit milestone commitment:
  `After`, `AtOrAfter`, `Debounce`, `Throttle`, `StaleAfter`, and `Interval`
  or direct canonical equivalents. This is part of the substrate, not optional
  polish.
- This milestone should treat `Debounce(...)` as a migration case, not as the
  final shape of temporal policy vocabulary.
- `After` and `AtOrAfter` are the core relative and absolute scheduling
  primitives.
- `Throttle` is the core bounded-rate primitive.
- `StaleAfter` is the core freshness-expiry primitive that later async and
  resource work will depend on.
- `Interval` is the core recurring-wake primitive and belongs in the complete
  timing system before async work begins.
- `Interval` must not be specified as a naive "run every N" surface. Its
  substrate contract must include period, anchor, and missed-tick policy.
- Previous-value support should be introduced only to the degree necessary to
  make temporal computation honest. Broader historical value APIs belong to
  later work unless they are required to close replay parity here.

## 18. Explicit Deferrals

Milestone A intentionally does not include:

- async/resource lifecycle
- retry or timeout product surfaces beyond what temporal substrate vocabulary
  must make possible later
- frontend timer ergonomics
- domain-specific temporal policy libraries
- route-resource or form abstractions

Those remain later roadmap work.

## 19. Sequencing Notes

This milestone belongs immediately before async/resource substrate work because
async lifecycle truth depends on runtime-owned time.

Milestone A must close before the async milestone can honestly define:

- stale-after windows
- timeout semantics
- retry/backoff scheduling
- deterministic completion ordering against time

If async lands first, the runtime will smuggle temporal meaning back into host
glue and poison the architecture.

## 20. Milestone Done When

Milestone A is done only when `worth-signal` can support temporal and
previous-value-sensitive execution through a frozen, typed, replay-honest
substrate that:

- preserves authority boundaries
- makes temporal eligibility runtime-owned truth
- keeps previous-value access transactional and branch-honest
- exposes bounded, measurable temporal work
- integrates with rollback, observation, diagnostics, branch, and replay
  without inventing a second semantic story

At that point, `worth-signal` will finally own time as a real execution axis
rather than as a suggestion passed through host conditions.
