# worth-signal Performance Architecture

> **Status:** Pre-production. Breaking changes are expected.
>
> **Parent:** [signal_architecture2.md](./signal_architecture2.md)
>
> **Goal:** Define the performance work needed to make `worth-signal` viable across very different deployment niches without hard-coding one niche's assumptions into the engine forever.

---

## Table of Contents

1. [Problem Statement](#problem-statement)
2. [Performance Model](#performance-model)
3. [Performance Profiles](#performance-profiles)
4. [Phase P1 â€” Hot Path Isolation](#phase-p1--hot-path-isolation)
5. [Phase P2 â€” Data Layout and Locality](#phase-p2--data-layout-and-locality)
6. [Phase P3 â€” Topology Mutation Discipline](#phase-p3--topology-mutation-discipline)
7. [Phase P4 â€” Parallel Scaling](#phase-p4--parallel-scaling)
8. [Phase P5 â€” Profile-Gated Observability](#phase-p5--profile-gated-observability)
9. [Phase P6 â€” Domain-Specific Optimization Hooks](#phase-p6--domain-specific-optimization-hooks)
10. [Current Confirmed Hot-Path Costs](#current-confirmed-hot-path-costs)
11. [What Must Stay Configurable](#what-must-stay-configurable)
12. [Sequencing](#sequencing)

---

## Problem Statement

`worth-signal` is no longer just a correctness-first reactive graph. It is intended to support very different workloads:

- web and application backends
- geometry kernels and CAD-like topology graphs
- chip simulators and hardware-style propagation graphs
- possibly game-engine or frame-budget execution contexts

Those workloads do **not** want the same performance law.

Examples:

- a web-oriented runtime may value rich diagnostics and flexible dynamic tokens
- a geometry kernel may care most about adjacency mutation cost and cache locality
- a chip simulator may care most about deterministic propagation throughput and sparse fanout scaling
- a frame-budget runtime may care most about bounded maintenance pauses and allocation predictability

So this document does **not** say â€œone perfect fast architecture for every deployment.â€ It says:

1. isolate the universal hot-path costs
2. make those costs configurable or profile-driven where the product needs differ
3. avoid letting convenience-oriented defaults permanently dictate the engine core

---

## Performance Model

We should treat performance work in three layers:

### Universal engine concerns

These are almost always worth improving:

- cache locality on hot graph traversal paths
- avoiding unnecessary heap allocation on evaluation hot paths
- avoiding lock contention in parallel execution
- avoiding quadratic adjacency mutation behavior
- separating hot-path execution from heavyweight observability

### Profile-sensitive concerns

These depend on the deployment:

- how much diagnostics is acceptable on the hot path
- whether partition/detail tokens can remain string-friendly at the public boundary
- how aggressively compaction/maintenance should run automatically
- whether deterministic ordering is worth extra cost in every operational mode

### Domain-specific concerns

These should not become universal engine law:

- geometry-specific quotient/equality policies
- chip-simulation-specific event batching
- web-specific diagnostics retention defaults
- frame-budget scheduling and maintenance windows

---

## Performance Profiles

The engine should support a small number of explicit performance profiles rather than hiding policy in ad hoc flags.

### Profile A â€” `Balanced`

Default product profile.

Characteristics:

- solid runtime performance
- production-safe diagnostics
- bounded maintenance
- deterministic behavior
- developer-friendly defaults

Good for:

- general application runtimes
- most integration and pre-production use

These named profiles lower to the Foundational objective/activation axes.  The
lowering is compiler-visible: a profile request is admitted, resolved, and
installed before planning, and the planner reads only the installed policy.
Tier labels remain descriptive compatibility presets rather than a second
execution authority.

### Profile B â€” `Introspective`

Diagnostics-heavy profile.

Characteristics:

- richer replay, explain, flow capture, and semantic history
- more allocation and retention allowed
- used for harnesses, certification, debugging, and investigation

Good for:

- harness execution
- replay and forensic analysis
- certification-style workflows

### Profile C â€” `Throughput`

High-scale compute profile.

Characteristics:

- minimal hot-path observability cost
- low-allocation execution
- aggressive use of pre-sized buffers and compact representations
- stricter limits on dynamic formatting and retention

Good for:

- geometry kernels
- chip simulators
- large batched evaluation graphs

### Profile D â€” `FrameBound`

Latency-budget profile.

Characteristics:

- bounded maintenance work
- no surprise compaction spikes
- hot-path allocation discipline
- explicit end-of-cycle cleanup windows

Good for:

- game-engine style loops
- any runtime with strict latency ceilings

> [!IMPORTANT]
> These profiles are policy overlays on one engine architecture. They are not forks of the runtime.

---

## Phase P1 â€” Hot Path Isolation

### Problem

The first performance mistake is treating every code path as equally important. `worth-signal` already has the architecture to separate concerns; now it needs to use that separation to isolate its true hot paths.

### Design

Define the canonical hot paths:

- invalidation traversal
- planner/session construction
- evaluation effect application
- transaction-local execution
- topology mutation and reconciliation

Define the canonical cold paths:

- explain
- replay browsing
- rich diagnostics/history capture
- certification/harness shaping
- heavy metrics export

### Implementation Plan

1. Audit every evaluation/invalidation/transaction path for:
   - heap allocations
   - string formatting
   - map/tree construction
   - diagnostic object construction
   - lock acquisition
2. Move cold-path work behind:
   - explicit observer/read boundaries
   - explicit diagnostics profiles
   - deferred or post-execution materialization
3. Add internal profiling labels or counters so future work can tell:
   - hot-path execution cost
   - maintenance cost
   - observability cost

### Acceptance Criteria

- hot paths do not construct rich diagnostic artifacts by default
- production runtime execution can be reasoned about independently from explain/replay machinery
- cold-path shaping stays available without polluting the engine core

---

## Phase P2 â€” Data Layout and Locality

### Problem

The graph still risks carrying too much cold data through hot access patterns. This is especially dangerous for:

- `NodeEntry`
- adjacency storage
- traversal scratch
- version/snapshot access

### Design

Treat layout as a first-class architectural concern, not a later micro-optimization.

#### P2.1 â€” Hot/cold split on node state

Hot fields should be cheap to touch during invalidation and planning:

- state
- dirty masks
- version summaries
- compact handle references

Cold fields should be moved away from the common cache line when possible:

- detailed diagnostics references
- rarely used metadata
- heavyweight partition detail state that is only relevant in some domains

#### P2.2 â€” Storage-family locality review

Audit:

- edge stores
- snapshot stores
- compaction state
- traversal scratch buffers

for:

- pointer chasing
- avoidable indirection
- opportunities for SoA or tighter packed storage

#### P2.3 â€” Public-boundary allocation vs internal representation

Public APIs may still accept ergonomic types such as strings or richer tokens, but the engine core should normalize them quickly into compact internal representations.

This means:

- do not force string-based identities through the hot core forever
- keep profile/domain flexibility at the boundary
- keep core traversal and mutation representations compact

#### P2.4 â€” Effect Allocation Discipline

`EvaluationEffect` is now one of the engine's central hot-path artifacts. That means its allocation behavior is not an implementation detail; it is part of the architecture.

Current concerns include:

- cloning dependency snapshots during effect commit
- rebuilding changed-region vectors on common evaluation paths
- carrying heavyweight labels or metadata in profiles that do not need them

The engine should support one clean low-allocation effect path:

- move-based effect commit where possible
- reusable builders that clear and retain capacity
- or session/transaction-scoped arenas for transient effect-side data

The goal is to prevent per-node effect creation from becoming a hidden heap-allocation tax.

#### P2.5 â€” Hash-First Token Comparison

Token ergonomics at the public boundary do not require the engine core to ignore precomputed hashes.

For token-heavy invalidation and partition-matching paths, comparison should be structured as:

1. compare stable hash
2. if hashes differ, fail fast
3. if hashes match, fall back to full value comparison for collision safety

This preserves correctness while reducing repeated string-heavy comparison cost.

#### P2.6 â€” Scratch Retention Policy

`TraversalScratch` currently benefits throughput by retaining grown capacity, but that should be a policy choice rather than an implicit permanent law.

Different profiles may want different retention behavior:

- `Throughput`: retain aggressively
- `Balanced`: retain but trim opportunistically
- `FrameBound`: decay or bound retained capacity after bursts

### Implementation Plan

1. Measure/inspect `NodeEntry` and related hot structs for hot/cold separation opportunities.
2. Audit partition/detail token normalization paths.
3. Audit `EvaluationEffect` construction and commit for clone-heavy or allocation-heavy behavior.
4. Move profile/domain-friendly representations to the boundary if they are polluting hot storage.
5. Add explicit scratch-retention policy instead of one implicit retention law.
6. Keep the resulting layout configurable where the domain genuinely differs.

### Acceptance Criteria

- hot graph traversal does not pull obviously cold data by default
- internal storage is more compact than the public ergonomic boundary
- web-friendly/public-friendly API choices do not permanently dictate engine memory layout

---

## Phase P3 â€” Topology Mutation Discipline

### Problem

For dynamic workloads, edge mutation cost can dominate:

- repeated `Vec::insert`
- repeated sort/dedup
- cloning adjacency sets
- repeated source-set rebuilding

That is acceptable for some low-churn workloads and fatal for others.

### Design

Make adjacency mutation strategies explicit and profile-sensitive.

#### P3.1 â€” Mutation-path complexity audit

Audit all topology mutation and reconciliation flows for:

- repeated cloning
- O(N) insertion shifting
- repeated full-set sorting
- repeated source-union rebuilding

#### P3.2 â€” Profile-sensitive adjacency representation

Different domains may want different tradeoffs:

- compact sorted segments for deterministic low-mutation workloads
- chunked append-oriented mutation buffers for higher churn
- periodic canonicalization rather than immediate perfect ordering

The engine should not hard-code one strategy if the workload classes differ materially.

#### P3.3 â€” Canonicalization boundaries

Not every mutation must immediately pay the full canonicalization cost.

Possible model:

- mutate in a cheaper working representation
- canonicalize at explicit boundaries:
  - commit
  - maintenance epoch
  - plan materialization
  - observation preparation

#### P3.4 â€” Batched Subscriber Reconciliation

Subscriber reconciliation should not pay one clone-mutate-reinsert cycle per individual source-edge change when the engine already knows the complete before/after dependency set.

The desired shape is:

- collect all subscriber mutations for the current reconciliation
- group them by affected source node
- apply one consolidated subscriber-set mutation per affected source

This is especially important for rewiring-heavy or keyed dynamic workloads.

### Implementation Plan

1. Identify the hottest topology mutation flows.
2. Classify workloads:
   - low churn / deterministic
   - high churn / runtime mutating
3. Add batched reconciliation for subscriber-set updates before changing broader storage strategy.
4. Introduce a strategy boundary in `EdgeTopology` if the current one-size-fits-all store is too constraining.
5. Preserve correctness and bidirectional invariants regardless of chosen mutation strategy.

### Acceptance Criteria

- adjacency mutation cost is no longer accidentally quadratic in common dynamic cases
- canonicalization work happens at explicit boundaries where possible
- deterministic behavior is preserved where required, but not at any cost in every profile

---

## Phase P4 â€” Parallel Scaling

### Problem

Parallel execution only helps if shared-state contention stays low. Any shared mutex-protected staging or central accumulation point can flatten scaling.

### Design

Parallel work should produce local data first and merge centrally only at clearly bounded seams.

#### P4.1 â€” Local-first staging

Parallel tasks should prefer:

- local effect/discovery output
- local report fragments
- local semantic summaries

Then merge through:

- effect application
- explicit reduction steps
- post-stage aggregation

#### P4.2 â€” Contention audit

Audit parallel paths for:

- shared `Mutex<HashMap<...>>`
- shared append vectors
- shared diagnostics staging
- shared string formatting and retention

#### P4.3 â€” Profile-sensitive parallel policy

Not every profile wants the same strategy:

- `Throughput` may prefer wider parallel staging and thinner reporting
- `Introspective` may accept lower throughput for richer traceability
- `FrameBound` may prefer bounded work over maximum thread saturation

#### P4.4 â€” Suppression Propagation Scaling

Even when precompute scales well, the engine can still bottleneck on sequential post-apply suppression propagation.

That means suppression cost must be treated as part of the scaling model:

- keep visited tracking lightweight for common small cases
- avoid allocation-heavy traversal structures where possible
- ensure suppression-side diagnostics do not dominate propagation work

### Implementation Plan

1. Identify remaining shared-state parallel bottlenecks.
2. Replace central shared staging with per-task or per-stage local accumulation where possible.
3. Audit sequential post-apply propagation for scaling bottlenecks, not just parallel precompute.
4. Keep reduction points explicit and measurable.
5. Let strategy selection remain profile-derived, not hard-coded globally.

### Acceptance Criteria

- parallel execution no longer depends on avoidable hot mutex contention
- merge/reduction points are explicit and auditable
- the chosen parallel behavior can vary by profile without forking the engine

---

## Phase P5 â€” Profile-Gated Observability

### Problem

Observability is valuable, but rich telemetry, flow summaries, replay events, and explanation artifacts should not silently dominate the hot path in every deployment.

### Design

Observability must be explicitly profile-gated and phase-bounded.

#### P5.1 â€” Minimal operational reporting

The default operational path should prefer:

- counters
- compact summaries
- lightweight record IDs

over:

- full flow summaries
- eagerly materialized cause chains
- heavy string formatting
- rich replay payloads on every operation

#### P5.2 â€” Deferred materialization

Where possible:

- store enough to reconstruct later
- materialize rich artifacts only when the profile or caller requests them

#### P5.3 â€” Harness and diagnostics honesty

Harness and certification flows can be rich, but they must remain explicit `Introspective`-style surfaces, not hidden cost inside normal execution.

#### P5.4 â€” Confirmed Hot-Path Allocation Audit

The engine should keep an explicit watchlist of concrete allocation-sensitive hot-path patterns instead of relying on vague performance intent.

Current confirmed examples include:

- changed-partition counting that allocates set structures during effect application
- dependency snapshot cloning during effect commit
- canonicalization helpers that allocate or clone on common paths
- label/detail formatting that leaks into execution-sensitive surfaces

### Implementation Plan

1. Audit where rich diagnostics are constructed eagerly.
2. Separate â€œrecord enough to explain laterâ€ from â€œfully explain now.â€
3. Keep a concrete hot-path allocation watchlist and burn it down intentionally.
4. Make diagnostic depth a deliberate profile/policy choice.
5. Keep harness on real production code, but allow it to opt into richer observation profiles.

### Acceptance Criteria

- throughput-oriented execution does not pay introspective-cost by default
- rich explain/replay remains available
- harness remains a real consumer of production code, not a special engine variant

---

## Phase P6 â€” Domain-Specific Optimization Hooks

### Problem

Some performance needs are domain-specific and should not become universal engine rules.

### Design

Provide explicit hooks where domains can specialize without forking the engine core.

#### P6.1 â€” Boundary normalization hooks

Examples:

- geometry kernels normalize topology/partition identities
- chip simulators normalize wire/net identifiers
- web systems normalize request/session/detail keys

#### P6.2 â€” Equality/comparison hooks

Examples:

- geometry outputs may need tolerance-aware identity
- chip outputs may need logical equivalence rather than raw bit-pattern equivalence

#### P6.3 â€” Maintenance policy hooks

Examples:

- frame-bound systems may only compact in explicit frame windows
- throughput systems may allow more aggressive amortized maintenance
- introspective systems may retain more history and delay cleanup

### Implementation Plan

1. Keep core engine hooks generic:
   - comparator policies
   - token normalization boundaries
   - maintenance policy inputs
2. Avoid baking geometry/game/chip assumptions directly into core storage and execution.
3. Let profiles and domain integrations supply policy, not structural forks.

### Acceptance Criteria

- the engine can optimize for multiple niches without splitting into incompatible runtime architectures
- domain-specific optimization stays at policy and boundary hooks where possible

---

## Current Confirmed Hot-Path Costs

These are not hypothetical concerns. They are concrete patterns that exist in the current codebase and should influence implementation order.

### 1. Subscriber reconciliation clone/reinsert cycles

The current reconciliation path can pay repeated:

- clone adjacency set
- mutate working vector
- reinsert and canonicalize

for multiple affected source nodes independently. This is why `P3.4` is explicit in this plan.

### 2. Dependency snapshot cloning on the effect path

Effect application still risks cloning full dependency snapshots on the recompute path. This is a direct `P2.4` target.

### 3. Allocation-heavy changed-partition counting

Changed-region accounting can allocate set structures where the common cardinality is small. This is part of `P5.4`.

### 4. String-heavy partition matching

Token ergonomics are good at the boundary, but hot matching should not ignore stable hashes. This is why `P2.5` is a named item rather than an implied optimization.

### 5. Sequential suppression propagation cost

Even a well-parallelized planner can still bottleneck on sequential suppression walks if they allocate or use heavyweight visited structures. This is why `P4.4` exists.

### 6. Scratch retention after bursty growth

Monotonic scratch growth is not always an acceptable permanent cost. This is why `P2.6` is profile-aware rather than a single default retention law.

---

## What Must Stay Configurable

These choices should not be turned into permanent universal law without strong evidence:

- diagnostics depth and retention
- token representation at the public boundary
- compaction cadence and maintenance windows
- deterministic canonicalization eagerness
- parallelism width vs bounded latency tradeoffs
- adjacency mutation strategy if workload classes diverge materially

If we ever choose a single default, that default must remain a policy choice, not an irreversible structural assumption.

---

## Sequencing

Recommended order:

1. `P1` hot-path isolation
2. `P5` observability gating
3. `P2` data layout and locality
4. `P4` parallel scaling
5. `P3` topology mutation discipline
6. `P6` domain-specific optimization hooks

Reasoning:

- first identify what is actually hot and what should be cold
- then stop paying introspective cost in hot paths
- then fix storage/layout and parallel contention
- then redesign mutation strategy with actual workload profiles in mind
- then expose domain-appropriate hooks without hard-coding one nicheâ€™s performance law into the engine
