# worth-signal Performance Architecture â€” Multi-Domain Performance Viability Plan

> **Status:** Pre-production. Breaking changes are expected and encouraged.
>
> **Scope:** Performance architecture for `worth-signal` across multi-domain workloads:
>
> - web systems
> - game/runtime workloads
> - CAD / geometry kernels
> - chip simulation
>
> **Core constraint:** Preserve semantic traceability and deterministic replay while making hot execution paths performant enough for domain-specific production use.
>
> **Severity:** This document is not a tuning backlog. It is a viability document. For kernel-class CAD and aircraft-scale geometry workloads, the current `worth-signal` hot mutation path is not yet credible. Incremental wins matter, but they do not close the architecture gap.
>
> **Relationship to older docs:** This is now the canonical performance plan. It supersedes the older performance-architecture drafts by grounding the program in the current codebase and the benchmark suite we actually run.

---

## Table of Contents

1. [Doctrine](#doctrine)
2. [Reality Check](#reality-check)
3. [Viability Gap](#viability-gap)
4. [Current Code Map](#current-code-map)
5. [Performance Profiles](#performance-profiles)
6. [Phase P0 â€” Measurement Discipline](#phase-p0--measurement-discipline)
7. [Phase P1 â€” Trace Cost Separation](#phase-p1--trace-cost-separation)
8. [Phase P2 â€” Mutation Backend Redesign](#phase-p2--mutation-backend-redesign)
9. [Phase P3 â€” Data Layout and Locality](#phase-p3--data-layout-and-locality)
10. [Phase P4 â€” Allocation Discipline](#phase-p4--allocation-discipline)
11. [Phase P5 â€” Parallel Scaling](#phase-p5--parallel-scaling)
12. [Phase P6 â€” Maintenance and GC Policy](#phase-p6--maintenance-and-gc-policy)
13. [Phase P7 â€” Domain Profile Packaging](#phase-p7--domain-profile-packaging)
14. [Numeric Targets](#numeric-targets)
15. [Sequencing](#sequencing)
16. [What Must Never Be Sacrificed](#what-must-never-be-sacrificed)

---

## Doctrine

### The rule

WORTH must be:

- **traceable enough for aerospace-grade audit**
- **debuggable enough for agent-native kernel development**
- **fast enough for production CAD, chip simulation, and runtime workloads**

For aircraft-class geometry systems, these are not tradeable aspirations. They are simultaneous requirements.

These requirements are compatible only if we separate:

1. **semantic traceability**
2. **trace materialization cost**

### M10 policy vocabulary

The runtime now names two independent policy axes through
`worth-foundational`: `ExecutionObjectiveProfile` (`LatencyBounded`, `Balanced`,
or `Throughput`) and `ObservationActivationProfile` (`OnDemand` or
`Continuous`).  A diagnostics tier remains a descriptive preset input; it is
not the planner's runtime authority.  Signal policy must progress through
`request -> admitted -> resolved -> installed`, and planners consume the
installed policy rather than reading a caller-supplied tier directly.

`Throughput` is therefore a hot-path objective, not permission to weaken graph
truth, deterministic ordering, durability, or stable lineage.  Optional
diagnostic, provenance, replay, and structural work is either explicitly
activated and disclosed or represented as a typed absence.  No timing claim is
authoritative by itself.

The public constructor `SignalRuntimePolicy::operational()` is the
Throughput + OnDemand production posture.  The idle-versus-introspective
packet records that this posture completes more work than Forensic
Continuous; it does not introduce a second constructor.  Parallel
region-disjoint execution remains the next Throughput scaling investment.

### What this document is not

This is not a list of performance tips.

It is not a promise that a sequence of 10-20% wins will make the system kernel-ready.

It is not permission to confuse useful bridge optimizations with closure of the core performance gap.

### Mandate

`worth-signal` must become credible for geometry-kernel-class mutation and recomputation workloads.

That requires:

- measurement strong enough to expose structural waste
- explicit rejection of hot-path cost models that do not scale
- redesign of the mutation backend rather than indefinite clone/rewrite cleanup
- profile-gated observability so hot execution does not pay forensic cost by default

### What is universal

These are not optional in any mode:

- the spec graph is the source of truth
- the reactive graph is the computation backbone
- every meaningful decision has stable lineage identity
- tolerance and escalation decisions are explicit, bounded, and inspectable
- replay and reconstruction remain deterministic

### What is configurable

These must vary by domain and mode:

- how much trace detail is retained eagerly
- how mutation is physically executed
- how storage is laid out and compacted
- how maintenance is scheduled
- how parallel work is admitted
- how much observability is materialized inline

### Hard line

WORTH must never become a black box, but it also must not force every hot path to pay the cost of maximum introspection.

The correct model is:

- **lineage identity always**
- **rich materialization by profile**
- **compact hot-path semantic facts in prod**
- **reconstructable deep provenance in debug / forensic**

---

## Reality Check

The current system is materially better than it was, but still far from geometry-kernel-grade performance.

This is not a tuning problem anymore. It is a performance viability problem.

The main gaps are:

- mutation still pays too much immutable rewrite cost
- hot-path data layout is not yet cache-first enough
- effect and trace assembly still allocate too much
- parallel scaling still needs deeper audit
- maintenance policy is not yet domain-specialized enough

Incremental cleanup is still worth doing, but it is bridge work.

Bridge work is not the answer to the main viability gap.

The current churn-heavy mutation/storage model is still too expensive for the class of local topology editing expected in a serious geometry kernel.

### Current interpretation

Recent improvements that reduce churn-path medians by tens of percentage points are useful and should continue.

They do **not** justify a conclusion that the architecture is close to sufficient.

The correct interpretation is:

- local optimizations can remove obvious waste
- local optimizations can sharpen the benchmark signal
- local optimizations can buy time and reduce incidental pain
- backend redesign is still required

---

## Viability Gap

### What is not credible yet

For aircraft-scale geometry work, the following behaviors are not acceptable as the steady-state hot-path model:

- whole-slice dependency rewrites for local edge changes
- whole-slice subscriber rewrites for local edge changes
- repeated clone/edit/reintern cycles during churn-heavy reconciliation
- hot mutation paths that scale with broad container rewrite cost instead of local edit radius
- observability richness that taxes operational mutation paths by default

### What must change

The main required redesign is not optional:

- a batched mutable topology-edit backend for dependency and subscriber updates
- storage representations that tolerate churn without constant whole-set rewrite cost
- stronger separation between operational execution and forensic materialization
- geometry-shaped certification workloads that prove local edits stay local in cost

### Bridge Work Versus Viability Work

#### Bridge work

Examples:

- remove unnecessary sorting
- replace full scans with range-based edits
- reduce per-effect allocations
- remove hot-path set/tree allocations
- tighten suppression traversal scratch behavior

These changes are good and should continue.

They are not sufficient.

#### Viability work

Examples:

- redesign mutation/storage to support batched local rewiring
- introduce transient mutable builders and single-commit reconciliation
- specialize storage/layout for churn-heavy geometry-style neighborhoods
- define kernel-operational profiles that preserve lineage identity without paying full forensic cost

These changes determine whether `worth-signal` becomes suitable for the target domain at all.

---

## Current Code Map

This section exists to keep the plan honest. These are the concrete code surfaces the performance program is currently about.

### Measurement and current evidence

- benchmark suite:
  - [performance_profiles.rs](/Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/tests/performance_profiles.rs)
- recorded baseline and measured deltas:
  - [signal_performance_baseline.md](/Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/plans/worth_signal/signal_performance_baseline.md)

### Trace / effect hot path

- effect application:
  - [effect.rs](/Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/graph/runtime/effect.rs)
- graph-owned hot traversal state:
  - [graph.rs](/Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/graph/runtime/graph.rs)

Confirmed current concerns:
- `EvaluationEffect` size and allocation shape
- trace/provenance construction cost on hot execution paths
- suppression propagation overhead

### Mutation backend and churn pressure

- topology mutation and reconciliation:
  - [mutation.rs](/Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/graph/topology/mutation.rs)
- retirement cleanup:
  - [retirement.rs](/Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/graph/lifecycle/retirement.rs)
- rollback repair:
  - [patch_buffer.rs](/Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/transaction/patch_buffer.rs)
  - [transaction_commit.rs](/Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/transaction/transaction_commit.rs)

Confirmed current concerns:
- adjacency sets still rewrite too much under churn
- transient mutable builders do not exist yet
- raw point-update mutation remains more expensive than it should be

### Data layout / locality

- node state:
  - [entry.rs](/Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/node/entry.rs)
- graph stores:
  - [segmented.rs](/Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/graph/storage/segmented.rs)
  - [handles.rs](/Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/graph/storage/handles.rs)
- topology runtime access:
  - [runtime.rs](/Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/graph/topology/runtime.rs)

Confirmed current concerns:
- `NodeEntry` hot/cold split can still go further
- edge/snapshot access patterns still need locality work
- scratch retention policy is not yet profile-specialized

### Parallel scaling and execution path collapse

- shared runtime/transaction execution path:
  - [shared.rs](/Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/transaction/runtime/execution/shared.rs)
- planner execution:
  - [mod.rs](/Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/planner/execution/mod.rs)
- precompute dispatch:
  - [dispatch.rs](/Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/planner/precompute/dispatch.rs)

Confirmed current concerns:
- parallel staging and post-apply bottlenecks still need deeper profiling
- suppression work may still erase some executor gains

### Observability and policy gating

- diagnostics profile:
  - [profile.rs](/Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/diagnostics/policy/profile.rs)
- runtime policy:
  - [mod.rs](/Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/diagnostics/policy/mod.rs)
- deployment presets:
  - [deployment.rs](/Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/presentation/outputs/deployment.rs)

Confirmed current concerns:
- observability is profile-gated
- hot mutation/storage/effect backends are **not yet** profile-gated enough

---

## Performance Profiles

WORTH needs a two-axis profile system:

1. **Domain**
2. **Mode**

### Domains

- `Web`
- `Game`
- `CAD`
- `ChipSim`

### Modes

- `Debug`
- `Dev`
- `Prod`

### Resulting profile matrix

- `WebDebug`, `WebDev`, `WebProd`
- `GameDebug`, `GameDev`, `GameProd`
- `CADDebug`, `CADDev`, `CADProd`
- `ChipSimDebug`, `ChipSimDev`, `ChipSimProd`

### What profiles should control

- diagnostics retention
- explanation/provenance materialization
- replay detail
- mutation backend
- effect allocation strategy
- maintenance cadence
- scratch retention strategy
- executor admission policy
- compaction aggressiveness

### Important constraint

Profiles should choose **policy and backend**, not semantic meaning.

The same computation must mean the same thing across profiles. Only cost model and retention model may vary.

---

## Phase P0 â€” Measurement Discipline

### Goal

Stop making performance decisions from one-off noisy runs.

This phase exists to prevent false comfort from isolated wins or losses. Kernel-grade redesign cannot be guided by anecdotes.

### Work

- expand the benchmark matrix to cover:
  - raw mutation stress
  - production reconciliation stress
  - suppression/propagation stress
  - mixed-fanout realistic workloads
  - observability profile deltas
  - future CAD/kernel analog workloads
- record:
  - repeated runs
  - medians
  - p95 / p99 where possible
  - workload shape
  - active profile
  - hot-path counters
- maintain:
  - baseline
  - current
  - target
  - delta

### Deliverables

- stable performance suite
- updated [signal_performance_baseline.md](./signal_performance_baseline.md)
- per-phase measured deltas

### Acceptance

- no optimization claim without repeated measurements
- no single-run screenshot math
- all major performance phases have before/after data

---

## Phase P1 â€” Trace Cost Separation

### Goal

Keep semantic traceability universal while removing rich trace construction from hot production paths.

This is required for viability, not polish. A geometry kernel cannot afford to pay forensic-materialization cost on every serious operational edit.

### Design

Introduce a compact hot-path trace model containing only the information needed to preserve semantic identity and deterministic replay:

- decision kind
- node / operation id
- stable lineage id
- upstream lineage ids
- tolerance class
- margin / threshold metrics
- escalation markers
- replay linkage ids

Rich explanation / provenance objects become:

- eagerly retained in `Debug`
- partially retained or reconstructable in `Dev`
- compact-fact-backed in `Prod`

### Work

- identify every hot path that currently constructs rich trace/provenance artifacts inline
- add compact semantic event records
- move rich artifact construction behind profile gates
- ensure deterministic reconstruction from compact facts + retained state
- keep replay/event ordering stable

### Acceptance

- hot paths in `Prod` no longer allocate rich explanation trees
- lineage identity is never dropped
- debug/forensic semantics remain reconstructable

---

## Phase P2 â€” Mutation Backend Redesign

### Goal

Stop using clone/mutate/reintern as the default physical mutation model.

### Problem

Current churn-heavy mutation still overpays for:

- repeated vector cloning
- point insert/remove against immutable adjacency sets
- whole-set reinterning
- repeated per-source rewrites during reconciliation

For aircraft-class geometry work, these costs are not â€œsuboptimal.â€ They are disqualifying if left as the default hot-path model.

### Design

Keep one semantic reconciliation model, but allow different backend implementations.

The redesign target is explicit:

- local topology edits must cost closer to the changed neighborhood
- repeated rewiring must batch into mutable working sets
- commit must happen once per affected set or batch, not once per tiny edge mutation
- operational profiles must not pay avoidable reconstruction or rewrite tax

### Subphases

#### P2.1 â€” Bulk reconciliation as the default semantic shape

- dependency reconciliation
- subscriber reconciliation
- rollback repair
- retirement cleanup

All should express work as **source-keyed final-state rewrites**, not repeated tiny edge edits.

#### P2.2 â€” Transient mutable builders

Add mutable transient builders for:

- dependency sets
- subscriber sets

These builders should:

- accept sorted inserts/removes efficiently
- commit back to segmented storage once per affected set

This is the first major redesign gate. If this class of builder does not materially reduce churn-path cost, the storage backend needs a deeper replacement rather than more local cleanup.

#### P2.3 â€” Backend selection by profile

Examples:

- `Balanced`
  - current segmented immutable-ish path with bulk rewrites
- `Throughput`
  - transient builder path
- `FrameBound`
  - bounded-budget mutation path
- `Kernel`
  - aggressive bulk builder path with minimal hot-path observability

#### P2.4 â€” Geometry-topology readiness

The mutation backend must explicitly support workloads like:

- many leaves
- many shared sources
- rotating dependency windows
- repeated source replacement
- localized topology healing analogs

These are not â€œnice to haveâ€ benchmark shapes. They are minimum viability evidence for geometry-kernel credibility.

### Acceptance

- production reconciliation workloads improve materially again
- rollback and retirement use the same backend model
- common local rewiring avoids repeated clone/reintern cycles
- the hot mutation path no longer relies on whole-slice rewrite as its default physical model

---

## Phase P3 â€” Data Layout and Locality

### Goal

Make hot reads and writes cache-friendly enough for CAD/kernel and chip-sim scale.

### Work

#### P3.1 â€” Hot/cold separation

Push `NodeEntry` further toward hot/cold separation:

- hot:
  - state
  - versions
  - dependency/subscriber ids
  - dirty masks
- cold:
  - trace summaries
  - causality
  - large optional metadata

#### P3.2 â€” Edge/snapshot locality

Audit:

- dependency edge storage
- subscriber edge storage
- snapshot storage

Decide where SoA-like or more compact layouts are warranted.

#### P3.3 â€” Token comparison fast paths

Support hash-first comparison with collision-safe fallback for hot partition matching.

#### P3.4 â€” Scratch retention policy

`TraversalScratch` must support profile-aware retention:

- `Debug` may keep larger scratch state
- `Prod` should cap memory waste
- `FrameBound` / `Kernel` should avoid surprise retained bloat after spikes

### Acceptance

- smaller hot node footprint
- fewer cache-hostile reads in common execution paths
- measurable gains in locality-sensitive workloads

---

## Phase P4 â€” Allocation Discipline

### Goal

Eliminate avoidable heap churn from the effect path and related hot loops.

### Work

#### P4.1 â€” `EvaluationEffect` discipline

`EvaluationEffect` is now central architecture and must become allocation-aware:

- reusable effect builders
- move-based commit everywhere possible
- avoid cloning snapshots and causality
- keep compact region/label structures small-buffered where feasible

#### P4.2 â€” Hot-path allocation audit

Audit and remove:

- `BTreeSet` on hot paths
- avoidable `String` construction
- per-node temporary vectors that can be reused
- canonicalization helpers doing avoidable clone/sort work

#### P4.3 â€” Builder / arena experiments

If needed, add:

- transaction-local effect arenas
- scratch-owned reusable effect buffers
- iterator-to-store rewrite paths

### Acceptance

- effect application becomes near-zero-alloc in common cases
- allocation-sensitive workloads show measurable gains
- hot-path memory churn is visibly reduced

---

## Phase P5 â€” Parallel Scaling

### Goal

Make multicore execution actually scale under realistic workloads.

### Work

#### P5.1 â€” Shared staging audit

Identify and remove:

- central lock contention
- shared mutable staging bottlenecks
- hidden serialization points

#### P5.2 â€” Suppression propagation scaling

Audit and redesign the sequential post-apply suppression walk where needed:

- reduce redundant traversal state
- bound per-effect propagation overhead
- avoid erasing parallel gains with serial cleanup

#### P5.3 â€” Profile-aware executor strategy

Parallel behavior should vary by domain:

- `Web`: conservative
- `Game`: bounded and latency-aware
- `CAD`: balanced with strong determinism
- `ChipSim`: throughput-heavy

### Acceptance

- parallel workloads scale without obvious central contention
- suppression/maintenance no longer erase gains
- executor choice is domain/profile-aware

---

## Phase P6 â€” Maintenance and GC Policy

### Goal

Ensure maintenance never creates unacceptable latency cliffs.

### Work

#### P6.1 â€” Explicit maintenance policy by profile

- `Debug`
  - richer checks, freer maintenance
- `Dev`
  - bounded but useful maintenance
- `Prod`
  - strongly bounded
- `FrameBound`
  - explicit maintenance windows only
- `Kernel`
  - explicit maintenance phases only

#### P6.2 â€” Compaction policy hardening

- profile-aware compaction cadence
- bounded maintenance budgets
- no surprise compaction in hot windows

#### P6.3 â€” Scratch / retained state discipline

- bounded scratch reuse
- controlled shrink/reset policy after spikes

### Acceptance

- no surprise maintenance spikes in latency-sensitive modes
- compaction and scratch retention are explicit policy

---

## Phase P7 â€” Domain Profile Packaging

### Goal

Turn the profile system into real product-ready policy bundles, not labels.

### Work

Each domain/mode combination should define:

- diagnostics retention
- mutation backend
- effect allocation mode
- maintenance cadence
- executor strategy
- artifact materialization mode

### Example expectations

#### `WebProd`

- low-overhead
- conservative parallelism
- minimal eager diagnostics
- bounded maintenance

#### `GameProd`

- strict frame sensitivity
- bounded mutation and maintenance
- minimal hot-path diagnostics
- no surprise compaction

#### `CADDev`

- rich reconstruction and lineage
- heavier validation
- still reasonably performant mutation backend

#### `CADProd`

- high throughput with compact hot-path tracing
- deterministic reconstruction path
- limited eager artifact retention

#### `ChipSimProd`

- throughput-first
- large-graph-friendly storage behavior
- very strict allocation discipline
- compact facts, reconstructable deep audit

### Acceptance

- profiles are real and coherent
- no domain pays permanently for another domainâ€™s debug burden

---

## Numeric Targets

These numbers should be maintained in [signal_performance_baseline.md](./signal_performance_baseline.md).

### Near-term

- production reconciliation rotating-window workload
  - current: ~`1.46 ms`
  - target 1: `<500 us`
  - target 2: `<100 us` for common local edits

### Broader targets

- suppression workloads: lower mean and lower variance
- fintech mixed-fanout: materially reduced runtime without losing trace identity
- p95/p99 matter more than headline means

### Geometry-kernel interpretation

For aerospace/CAD-grade localized topology work:

- common local edits should trend toward **double-digit to low-hundreds of microseconds**
- large operation orchestration may still be millisecond-scale, but with bounded and explainable phase costs

---

## Sequencing

Recommended order:

1. `P0` measurement discipline
2. `P1` trace cost separation
3. `P2` mutation backend redesign
4. `P4` allocation discipline
5. `P3` data layout and locality
6. `P5` parallel scaling
7. `P6` maintenance and GC policy
8. `P7` domain profile packaging

### Why this order

- first separate traceability from trace cost
- then fix the largest mutation bottlenecks
- then remove allocation waste
- then attack locality
- then harden parallelism
- then formalize maintenance and profile packaging

---

## What Must Never Be Sacrificed

Even under the strongest performance specialization:

- no silent tolerance widening
- no black-box geometric decisions
- no loss of stable lineage identity
- no nondeterministic replay semantics
- no domain profile that changes what the engine *means*, only what it *costs*

This is the core doctrine:

> **WORTH may specialize execution policy aggressively, but it must never specialize truth.**
