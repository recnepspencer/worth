# WORTH Signal Vision

## Thesis

WORTH uses two distinct runtimes for two distinct jobs:

- `worth-relational` is the source-of-truth runtime. It owns identity, mutation, history, diffs, and traversal over the model graph.
- `worth-signal` is the derived-computation runtime. It owns invalidation, recomputation, scheduling, conditions, and transactional refresh of computed state.
- The bridge layer connects them. Relational changes become signal invalidations, and signals evaluate against stable host snapshots without becoming the owner of truth.

This split is not optional. `worth-signal` is scheduling and execution infrastructure, never truth. It exists to make derived computation deterministic, transactional, aspect-aware, and auditable while remaining fully decoupled from domain-specific storage and semantics.

`worth-signal` should be designed as a standalone generic library. CAD is a major target, but not the definition of the runtime. The same core should make sense for game engines, financial platforms, chip simulation, AI systems, and other domains where dependency-aware incremental computation matters.

## Mission

`worth-signal` is the universal derived-computation substrate for WORTH. More generally, it is a generic incremental computation runtime for systems that need dependency-aware recomputation over host-managed state. Every expensive or dependency-sensitive derived value can be modeled as a signal node: derived artifacts, validation summaries, simulation results, risk calculations, pricing outputs, query results, or analysis pipelines. The runtime exists to answer one question reliably:

> Given a set of upstream changes, what must recompute, in what order, under what conditions, and with what causal trace?

The runtime must answer that question with five non-negotiable properties:

1. Deterministic execution
2. Transactional rollback semantics
3. Aspect-aware invalidation granularity
4. Explicit separation from truth-state storage
5. First-class runtime self-inspection into why recomputation happened

Diagnostics are not optional polish. `worth-signal` must assume there will be runtime bugs, host bugs, policy mistakes, and hard-to-reproduce invalidation pathologies. Provenance, inspection, and metrics are therefore part of the product contract, not just developer support tooling.

The same is true for the test harness. `worth-signal` is now complex enough that scenario builders, regression seeders, parity drivers, and lifecycle-aware verification are no longer “nice test ergonomics.” They are part of how the runtime defends itself against future regressions.

## Architectural Model

### Runtime stack

| Layer | Responsibility | Owns |
| --- | --- | --- |
| `worth-relational` | Truth-state graph runtime | identity, transactions, history, diffs, traversal, integrity |
| `worth-signal` | Derived-computation runtime | dependency DAG, invalidation, recomputation, scheduling, conditions, runtime self-inspection |
| Bridge / integration | Decoupled coordination | patch-to-invalidation, aspect mapping, snapshot evaluation, node-key mapping |

### What `worth-signal` owns

- Evaluation dependency graph scheduling and invalidation
- Deterministic ordering
- Conditional and policy-aware evaluation gates
- Transactional invalidation and hard rewind on failure
- Node-scoped execution metadata such as aspects, conditions, comparator policy, and telemetry
- Query-style incremental execution semantics
- Future execution planning, staged execution, and parallel dispatch

### What `worth-signal` does not own

- Truth-state graphs, domain entities, or structural storage
- Domain numerics, geometry kernels, topology mutation, or schema rules
- Semantic meaning of aspects beyond host-defined slots
- Host identity models, diffs, or lineage semantics
- Permanent fusion with relational storage
- Mandatory use of the bridge for standalone signal use

### Structural rule

Signals consume host snapshots and emit derived-state refresh. They do not become a second source of truth, and they do not own the structural graph they observe.

## Provenance Model

`worth-signal` must make it possible to explain why a node is in its current state, not merely that it was evaluated.

The provenance ladder should stay explicit:

- invalidation provenance: which upstream changes dirtied or maybe-staled the node
- dependency provenance: which dependencies were considered and how their versions compared
- condition provenance: which evaluation conditions deferred or allowed work
- comparator provenance: which changes were suppressed as not meaningful
- recomputation provenance: whether work actually ran and what trace summary it produced
- host causality metadata: optional upstream provenance attached by host runtimes or future bridge integration

This is the baseline for trust, debugging, compliance, and hard-software diagnosis. The explanation surface, metrics surface, and inspection APIs should all reinforce this same causal model rather than invent separate diagnostic stories.

## API Strategy

`worth-signal` should expose two public faces built on one runtime.

The product requirement is not only "easy mode for simple cases." The full-power surface must also be beautiful for expert users who need explicit control. In WORTH, "beautiful" means explicit without boilerplate, powerful without generic noise, and predictable without ambient magic.

Both surfaces must remain directly usable without `worth-relational` or the bridge. The bridge is an integration layer, not the required entrypoint to the signal runtime.

### `worth-signal-core`

This is the low-level runtime surface. It keeps all control explicit:

- `SignalGraph`
- `SignalRuntime`
- transactions
- aspects
- evaluation conditions
- comparator policies
- explicit invalidation and evaluation entrypoints
- future planners and schedulers

This surface exists for kernel internals, integration layers, performance-sensitive paths, and any host that needs precise runtime control.

The design goal for this surface is not raw capability alone. The core API should read as a clean execution pipeline:

- configure runtime and policy once
- open a transaction or execution scope explicitly
- mark dirty inputs and emit staged changes
- evaluate targets under named policies
- commit or rollback deterministically

The hard-mode surface should therefore prefer:

- builder-driven construction over generic-heavy constructors
- fluent node and policy configuration over bulky config structs
- defaulted generics and obvious defaults for common advanced cases
- explicit extension points for comparators, conditions, planners, and bridge hooks
- local reasoning at the call site about what mutates, what evaluates, and what commits
- stable query-style execution surfaces for keyed computations and cache validation
- accessible, intention-revealing names such as `depends_on_aspects(...)` and `condition(...)` over compressed or insider-oriented terminology

### `worth-signal-easy`

This is the ergonomic signal surface. It is a core product pillar because the runtime should feel simple for common use, not only powerful for specialists.

The easy surface should provide:

- input signals
- computed signals
- effects/watchers
- batched updates
- automatic dependency capture
- implicit lazy pull on read for common workflows

The easy surface must compile down to the same runtime primitives as the core layer. It is a UX layer, not a separate execution engine.

### Design rule

Easy syntax must not imply easy internals. The low-level contract remains explicit, deterministic, transactional, and auditable even when the user-facing syntax feels Angular/Solid-style simple.

### Core API quality rule

The hard version must not feel like internal scaffolding. Advanced users should be able to express transactions, policy wiring, evaluation, and inspection in short linear flows without repeatedly spelling type noise or assembly-like setup.

### Naming rule

API language should optimize for first-read comprehension. Prefer accessible, slightly longer, intention-revealing names over compressed, overly technical, or clever names. If a longer name teaches the runtime model more clearly, prefer the longer name.

## Capability Matrix

Status meanings:

- `Implemented`: real in the current crate and covered by code/tests
- `Next`: intended near-term productization or runtime work
- `Later`: important, but deferred until the foundation above it exists

### WORTH Signal

#### Foundation runtime

| Capability | Status | Notes |
| --- | --- | --- |
| Dependency DAG | Implemented | `SignalGraph` owns the evaluation graph and rejects cycles at runtime |
| Aspect / granularity system | Implemented | Aspects, masks, and per-aspect versions are present |
| Transactional invalidation | Implemented | Runtime-backed transactions hard-rewind via sparse patching |
| Maybe-stale states | Implemented | `Clean` / `MaybeStale` / `Dirty` are part of the core contract |
| Lazy recomputation | Implemented | Pull-based evaluation recomputes only when requested |
| Aspect-based invalidation | Implemented | [Milestone 12](./milestone-12-plan.md) certifies producer-local transitive causality; [Milestone 13](./milestone-13-plan.md) certifies direct-hop semantic-frontier locality, typed work progression, and realized structural cost slopes; [Milestone 13.1](./milestone-13.1-plan.md) carries that precision through Runtime Bridge into Query-owned maintenance |
| Dynamic dependency discovery | Implemented | `EvaluationContext` records upstream reads explicitly |
| Conditional nodes | Implemented | `OnDemand`, `Debounce`, `AspectFilter`, `DeltaThreshold`, `Custom` |
| Tolerance / epsilon gates | Implemented | Comparator policies include exact and tolerance-based suppression |
| Deterministic execution behavior | Implemented | Current traversal/order semantics are intentionally deterministic |
| Telemetry baseline | Implemented | Runtime counters exist for evaluation, invalidation, rollback, GC, and gating |
| Execution objective and observation activation policy | Implemented | Foundational objective/activation vocabulary lowers through admitted, resolved, and installed Signal policy; throughput remains an objective, not a truth or durability waiver |
| Serialization-friendly graph state | Implemented | Core graph and node data are serializable, though snapshot API is not yet first-class |

#### Incremental computation semantics

| Capability | Status | Notes |
| --- | --- | --- |
| Query-style incremental execution | Implemented | Keyed computation families and family-scoped lookup now exist; broader query ergonomics can deepen later |
| Reactive diff propagation / result diffing | Implemented | Output identity, output change reporting, and downstream suppression are now first-class |
| Partial recomputation boundaries | Implemented | Partition-aware outputs, changed-region metadata, and partition-scoped subscriptions now exist |
| Structural memoization | Implemented | First-pass explicit memoization exists through host-supplied structural keys and family-scoped caches |
| Speculative evaluation / branching | Later | Depends on branchable execution state and discard semantics |
| Fixed-point / convergence nodes | Later | Valuable for solver-style workloads, but not foundation-critical |

#### Introspection and debugging

| Capability | Status | Notes |
| --- | --- | --- |
| Execution trace / provenance baseline | Implemented | Trace summary, structured explanations, and optional causality metadata now exist |
| Explain / provenance API | Implemented | `explain(node)` is now a first-class structured surface with human-readable display |
| Diagnostics-first inspection model | Implemented | Explanation, inspection, DOT export, and metrics now reinforce one causal debugging story |
| Diagnostics contract | Implemented | Diagnostics are now a first-class public subsystem with profiles, summaries, diffs, flow/failure artifacts, and one public entrypoint |
| End-to-end causality integration | Next | Signal explanation should be able to connect cleanly back through bridge-carried truth provenance |
| Graph inspection tools | Implemented | Direct graph export and dependency-chain inspection exist; hot-path analysis can deepen later |
| Dependency inspection | Implemented | Direct APIs now answer “who depends on what” explicitly |
| Execution metrics | Implemented | Telemetry is now surfaced intentionally through graph/runtime metrics snapshots |

#### Scheduling and execution

| Capability | Status | Notes |
| --- | --- | --- |
| Deterministic execution mode | Implemented | Current runtime behavior is deterministic by design |
| Core API ergonomics | Implemented | Full-power API now uses builders, explicit transactions, and accessible naming |
| Builder-based runtime ergonomics | Implemented | Runtime builder, transaction helpers, and node builders are now first-class |
| Explicit execution planner | Implemented | Reusable staged planning now exists, but hot-path hardening and cached topology remain future work |
| Parallel prepared execution | Partially implemented | Same-stage precompute and proof-safe grouped apply packet construction can run concurrently; canonical publication remains ordered and unsupported mutable stages fall back explicitly |
| Deterministic parallel foundation | Planned | [Milestone 14](./milestone-14-plan.md) establishes one bounded resource authority, determinism contracts, hierarchical semantic locality, non-authoritative execution placement, cancellation, and canonical publication |
| Proof-carrying graph parallelism | Planned | [Milestone 15](./milestone-15-plan.md) extends concurrency across hierarchical candidate batches and graph shards only where dependency readiness, control order, cross-shard boundaries, and mutation footprints prove safety |
| Structured partitioned parallelism | Planned | [Milestone 16](./milestone-16-plan.md) adds domain-agnostic map/reduce/scan/fork-join/round infrastructure with explicit locality binding and boundary reads for work inside a node |
| Portable execution backends | Planned | [Milestone 17](./milestone-17-plan.md) carries semantic locality and replaceable physical placement separately across versioned native, WASM-worker, remote, and accelerator-conformance boundaries |
| Cost-aware scheduling | Later | Requires per-node cost metadata and planner integration |
| Priority propagation | Later | Requires explicit scheduling model and prioritization semantics |

#### State / replay / evolution

| Capability | Status | Notes |
| --- | --- | --- |
| First-class signal graph snapshots | Next | Data is serializable, but snapshot/restore is not yet a first-class API |
| Replay-oriented evaluation state capture | Next | Needs explicit runtime snapshot surface and metadata framing |
| Branchable evaluation paths | Later | Depends on snapshot/branch semantics rather than current in-place flow |
| Signal lineage | Next | Track how computed artifacts evolve across evaluations, cache refresh, replacement, snapshot restore, and branch switches |

#### Runtime trust infrastructure

| Capability | Status | Notes |
| --- | --- | --- |
| Production diagnostics subsystem | Implemented | Diagnostics are a public runtime contract with recorder/store/policy separation and one diagnostics entrypoint |
| Signal-runtime test harness | Next | Harness must sit on top of production diagnostics and one truthful execution model before Phase 5 begins |

#### Easy API / developer experience

| Capability | Status | Notes |
| --- | --- | --- |
| Easy-mode signal API | Implemented | `worth_signal::easy::*` now exists as a separate surface over the same runtime |
| Angular-style computed ergonomics | Implemented | Input/computed/get/set/batch are now available without changing the core contract |
| Automatic dependency capture in easy API | Implemented | Easy-mode computed closures now discover dependencies automatically |
| Effects/watchers | Next | Ergonomic layer should expose subscription/effect patterns |
| Batch ergonomics | Implemented | Easy API now provides explicit batching over the same runtime semantics |

#### Bridge / dual-runtime integration

| Capability | Status | Notes |
| --- | --- | --- |
| Dual-graph architecture | Next | Architectural direction is clear, but dedicated bridge surface is not yet formalized |
| Patch-to-invalidation bridge | Implemented | [Milestone 13.1](./milestone-13.1-plan.md) carries authoritative Relational aspect/locality changes through installed Bridge correspondence into scoped Signal invalidation and Query maintenance |
| Aspect mapping layer | Implemented | Runtime Bridge owns exact installed correspondence and declared widening under [Milestone 13.1](./milestone-13.1-plan.md); Signal slots remain runtime-local |
| Snapshot evaluation | Implemented | Granular Query source reads are bound to the admitted immutable snapshot basis and fail closed when that basis drifts |
| Bulk change propagation | Implemented | [Milestone 13.1](./milestone-13.1-plan.md) carries semantic batches with owner-separated performed counters; physical parallel dispatch remains Milestone 14 work |
| Change stream protocol | Later | Generic protocol should exist before tighter integration scales up |
| Reactive source protocol | Later | Generic read contract for signal consumers without fusion |
| Relational-key to signal-node mapping | Later | Needed to keep truth IDs and signal IDs decoupled |
| Field / lens subscriptions | Later | Fine-grained subscriptions belong in the bridge, not core graph ownership |

### WORTH Relational Context

`worth-relational` is not the main subject of this document, but the signal vision depends on the truth-side runtime being explicit.

| Capability family | Role in the stack | Status framing |
| --- | --- | --- |
| Identity / storage | Truth-side handles, arena layout, structural identity hooks | Core truth-runtime concern |
| Transactions / savepoints | Authoritative mutation boundary | Core truth-runtime concern |
| History / MVCC / replay | Stable snapshots and deterministic history | Required for long-term bridge maturity |
| Diffs / CDC / patch feeds | Drives signal invalidation inputs | Direct bridge dependency |
| Traversal / introspection | Read-side access for host snapshots | Required integration surface |

## Roadmap

This roadmap is product-oriented. The existing foundation execution plan remains the detailed crate-level implementation plan for the current base runtime, and the dedicated Phase 1 plan captures the concrete productization pass. This document defines what the product becomes after that foundation exists.

### Phase 1: Productize the current runtime (Completed)

**Outcome:** `worth-signal` is both powerful and approachable for direct use.

Major additions:

- hard-mode API cleanup for transactions, builders, and common execution flows
- accessible naming cleanup for core APIs, builders, and examples
- `worth-signal-easy` ergonomic API
- runtime builder for `SignalRuntime`
- transaction closure helpers
- node builder ergonomics
- better crate-level documentation of core vs easy usage

Phase 1 established the public runtime surface. The current runtime is now legible and humane enough to build on without carrying forward the old public API clutter.

See `plans/engineering/worth_signal_phase1_plan.md` for the concrete execution plan.

### Phase 2: Observability and dependency inspection (Completed)

**Outcome:** users can see why work ran, what depends on what, and where recomputation cost is going.

Major additions:

- first-class execution provenance surface through `explain(node)`
- dependency inspection APIs
- graph inspection and debug export tools
- richer surfaced metrics on top of existing telemetry
- bridge-ready causality metadata hooks

Phase 2 is the baseline provenance phase for the runtime. It should be treated as core product infrastructure, not optional debugging sugar. The goal is to make current-state causality explicit: why a node is dirty, maybe-stale, deferred, recomputed, or unchanged in the moment.

### Phase 3: Smarter propagation and diff-aware execution (Completed)

**Outcome:** the runtime suppresses more unnecessary work and becomes more semantically aware of unchanged outputs.

Major additions:

- query-style keyed computation surfaces
- output identity / result diffing
- downstream suppression when outputs are unchanged
- partition-aware output reporting and partition-scoped subscriptions
- first structural memoization layer with explicit host keys

Phase 3 established the smarter propagation substrate while preserving future snapshot and signal-lineage semantics. Output diffing, partition-aware subscriptions, and memoization now exist in forms that keep artifact continuity describable later.

### Phase 4: Execution planning and prepared parallel precompute (Completed)

**Outcome:** evaluation is planned explicitly, then dispatched through one truthful prepared execution backbone with optional same-stage parallel precompute.

Major additions:

- reusable execution planner
- staged execution model
- executor abstraction
- deterministic staged scheduling
- real stage-local parallel precompute on the prepared execution contract

Phase 4 is now materially complete for the planner/prepared backbone. The runtime has one real planner/executor backbone, prepared evaluation, execution records, and honest same-stage parallel precompute.

What Phase 4 did **not** complete:

- fully mature parallel execution
- strict hierarchical resource budgeting
- graph-wide control-order and disjoint-mutation proof
- structured nested partition work
- portable WASM-worker, accelerator, and remote backend execution
- data-oriented planner/storage hardening for very large graphs

Later work added proof-safe grouped concurrent apply packet construction with
deterministic publication, but it does not close those broader claims. The
numbered Milestones 14-17 own their completion.

### Phase 4.5: Runtime trust layer and diagnostics contract (In progress)

**Outcome:** diagnostics become a first-class runtime contract and the crate finishes collapsing toward one execution story.

Major additions:

- public diagnostics subsystem with one entrypoint
- lifecycle-native flow diagnostics
- failure and rollback diagnostics
- diagnostics profiles and bounded retention policy
- repeated boundedness and serial/parallel parity hardening
- final callback-era execution cleanup so the harness rests on one truthful engine
- scale-hardening plan so planner, diagnostics, storage locality, and executor work are tracked explicitly before aerospace-grade claims are made

See `plans/engineering/worth_signal_scale_hardening_plan.md` for the strict follow-on plan covering planner hot-path cost, diagnostics overhead, node/snapshot locality, and executor maturation.

This is the phase that turns runtime self-inspection from “good debugging support” into real trust infrastructure for hard software.

### Harness Foundation: Scenario Infrastructure Before Phase 5

**Outcome:** future snapshot/replay/lineage work lands on top of a reusable runtime harness instead of ad hoc tests.

Major additions:

- builders for common signal topologies and runtime scenarios
- named seeders and mandatory regression seeders
- lifecycle-aware drivers for serial, parallel, transaction, and rollback flows
- selectors and fluent verification built on production diagnostics
- parity and determinism helpers that become the default validation path for later phases

The harness is not optional test polish. It is infrastructure for proving determinism, lifecycle correctness, provenance, and historical behavior as the runtime grows.

### Phase 5: Snapshots, replay, and branchable evaluation

**Outcome:** signal state becomes explicitly captureable, inspectable, and replay-friendly.

Major additions:

- first-class snapshot/restore API
- evaluation-state persistence model
- signal lineage foundations so computed artifacts can be tracked across refresh, replacement, restore, memoized reuse, and branch switches
- replay-oriented inspection tooling
- branchable evaluation-path foundations

See `plans/engineering/worth_signal_state_lineage_design.md` for the concept lock on snapshots, replay, provenance, and signal lineage.

Phase 5 is the historical-provenance phase. It should extend provenance from current-state explanation into replayable, restorable, and time-aware understanding of how evaluation state and computed artifacts evolved.

### Phase 6: Dual-runtime bridge completion

**Outcome:** relational truth and signal computation compose cleanly through stable protocols.

Major additions:

- patch-to-invalidation bridge
- aspect mapping layer
- snapshot-backed evaluation surface
- bulk diff propagation
- integration contracts for node-key mapping and source reads

Phase 6 is the end-to-end provenance phase. It should allow truth-side commit and patch causality to flow cleanly through the bridge into signal-side explanation and diagnostics without collapsing the runtimes into one system.

### Phase 7: Advanced semantics

**Outcome:** the runtime supports more demanding generic execution patterns without becoming domain-specific.

Major additions:

- fixed-point / convergence nodes
- speculative evaluation / branching
- mature structural memoization
- cost-aware and priority-aware scheduling policy surfaces

## Principles

1. `worth-signal` is never truth.
2. Easy syntax is a product requirement, not a runtime simplification.
3. Hard-mode syntax quality is a product requirement, not a cosmetic afterthought.
4. Determinism is a product feature, not just a test harness property.
5. Transactions and rollback are mandatory semantics, not optional safety sugar.
6. Bridge protocols must preserve decoupling between truth and compute.
7. Structural and domain semantics remain outside the runtime unless exposed as generic hooks.
8. `worth-signal` must stand on its own as a reusable library with standalone APIs, not only as one half of the WORTH stack.
9. Accessible naming is a product feature; prefer intention-revealing names over insider shorthand.
10. Diagnostics are first-class runtime architecture; explanation, inspection, provenance, and metrics must ship as core capabilities.
11. Signal lineage is a real runtime concern distinct from host truth lineage and should be modeled explicitly when replay and branching mature.
12. The runtime harness is first-class infrastructure; regression seeders, parity drivers, and lifecycle-aware verification must evolve with the runtime instead of trailing behind it.

## Non-goals

- Embedding geometry- or topology-specific semantics directly into `worth-signal`
- Collapsing relational truth storage and signal execution into one fused runtime
- Replacing explicit core APIs with only an ergonomic wrapper
- Treating the easy API as permission to weaken transactional or deterministic guarantees
- Shipping parallel execution before an explicit planner/stage model exists

## Public Surface Vocabulary

These names are conceptual API categories, not necessarily immediate crate splits:

- `worth-signal-core`: low-level runtime surface
- `worth-signal-easy`: ergonomic signal surface
- bridge / integration layer: relational-to-signal coordination surface

`worth-signal-core` should be optimized for expert readability, not only raw power.
Both `worth-signal-core` and `worth-signal-easy` should be directly usable without going through the bridge layer.

The current concrete vocabulary remains the anchor for the runtime contract:

- `SignalGraph`
- `SignalRuntime`
- transactions
- aspects
- evaluation conditions
- comparator policies

## Current-State Notes

- The current foundation is already strong enough to justify this vision: DAG scheduling, aspects, conditional evaluation, comparator policies, deterministic behavior, telemetry, and transactional rewind are real.
- The current runtime self-inspection baseline is now real too: structured explanations, dependency inspection, DOT export, surfaced metrics, richer trace summaries, and generic causality hooks are part of the runtime trust substrate.
- The diagnostics contract is now real too: public diagnostics entrypoints, profiles, lifecycle flow artifacts, structured diffs, and bounded retained history are part of the runtime product.
- The current foundation execution plan should be treated as the implementation hardening path for the base runtime, not as the final product vision.
- The remaining blocker before the harness is the last internal callback-era execution debt in the test substrate. Public execution is already on the prepared path; the final cleanup is about making the harness rest on one truthful engine model.
- The next major leap is not inventing a new core model. It is finishing that cleanup, building the harness, and then pushing into snapshots, lineage, replay, and bridge-grade causality.
- The state/replay/lineage concepts are now locked separately in `plans/engineering/worth_signal_state_lineage_design.md` so later phases do not drift into ad hoc semantics.
