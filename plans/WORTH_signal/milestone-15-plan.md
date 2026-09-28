# Milestone 15 Engineering Spec: Proof-Carrying Graph Parallelism

> **Status:** Planned
>
> **Prerequisite:** [milestone-14-plan.md](./milestone-14-plan.md)
>
> **Architecture parent:** [signal_architecture2.md](./signal_architecture2.md), `S9.17.2`
>
> **Successor:** [milestone-16-plan.md](./milestone-16-plan.md)

## 1. Goal And Roadmap Placement

Milestone 15 parallelizes every graph-level operation for which WORTH can carry
dependency, control-order, and mutation-disjointness proof, while preserving a
canonical serial meaning for the graph.

Milestone 14 supplies bounded execution resources, immutable worker inputs,
managed cancellation, deterministic publication, hierarchical locality, and
non-authoritative shard placement. Milestone 15 uses those foundations across
invalidation admission, ready-work scheduling, planning, eligibility,
evaluation, apply preparation, and proof-safe commit.

The central claim is not that every graph is parallel. It is:

> Every causally independent graph operation may execute concurrently, and
> every causal or conflicting operation remains explicitly ordered.

## 2. Current Boundary

WORTH currently forms topological stages and can precompute same-stage nodes in
parallel. It can also build suppression-free apply packets concurrently for
groups whose `ApplyFootprint`s do not conflict, then publish them in canonical
stage order.

The present parallel lane narrows sharply when a task has:

- dependency updates or rewiring
- output-identity comparison requiring shared-surface suppression
- an overlapping node/source footprint
- validation or planning work that remains serialized before precompute

Milestones 12 and 13 also establish a direct-hop semantic invalidation stream.
That stream may contain independent ready items, but scheduling mechanics must
not outrun unresolved predecessors or reinterpret causal evidence.

## 3. Adversarial Courtroom

Construct one graph world containing:

- at least 100,000 nodes with wide antichains and a long critical chain
- aspect-translating and partition-narrowing edges from Milestones 12-13
- deep hierarchical scope paths with exact-leaf, ancestor-subtree,
  sibling-disjoint, and unscoped subscriptions
- reconvergent diamonds and nodes with multiple causal justifications
- independent disjoint subgraphs and intentionally overlapping mutation sets
- dynamic dependency rewiring that reveals an edge absent from the prior
  execution
- output suppression, output-identity comparison, conditions, temporal gates,
  previous-value reads, on-demand nodes, and async-capable nodes
- a cycle attempt that must fail before graph mutation
- cancellation and failure at every graph phase
- branch capture, rollback, restore, replay, and repeated schedule perturbation
- hot-subtree skew, cross-shard diamonds, shard-boundary rewiring, and
  epoch-bounded shard rebalancing

Compare serial execution with graph-parallel execution across worker budgets
`1`, `2`, `P/2`, and `P`, and across forced stealing/interleaving schedules.

Required result:

- only ready topological antichains execute concurrently
- no task reads a predecessor version that can still change in its epoch
- disjoint invalidation edges and planning classifications can be sharded and
  merged canonically
- cross-shard dependencies remain explicit, readiness-ordered boundaries
- rewiring is evaluated from worker-local proposals and reconciled before any
  later task that could depend on the new edge is admitted
- overlapping mutation footprints are split into ordered conflict groups
- graph-parallel and serial execution commit identical canonical results
- realized speedup is reported against work and span, not core count alone

The courtroom must convict:

- parallelizing by stage index while ignoring dynamic control dependence
- using the previous dependency graph as complete proof after rewiring
- marking or enqueuing subscribers before semantic edge admission
- concurrent shared graph mutation guarded only by locks
- serial semantic work hidden inside a `FullParallel` label
- a conflict detector that ignores source, partition, snapshot, subscription,
  or diagnostic publication surfaces that participate synchronously
- a scheduler whose queues are unbounded under a wide frontier
- shard-local candidate discovery that skips an ancestor, loses an unscoped
  subscriber, or promotes physical placement into semantic authority

## 4. Product Decision Lock

### 4.1 Three Independent Proofs Are Required

A graph batch is parallel-admissible only when it carries:

1. dependency readiness: all producer versions read by the batch are settled
2. control-order safety: no earlier operation can reveal a new dependency that
   changes the batch's legal inputs
3. mutation disjointness: worker-local effects either do not overlap or have a
   declared deterministic reduction/reconciliation rule

Topological level alone proves none of these completely in a dynamically
rewiring graph.

### 4.2 Parallel Invalidation Does Not Own Semantics

Milestone 13's causality owner admits each direct dependency edge. Parallel
frontier execution may shard admitted edges, canonicalize duplicate causes,
and schedule ready items. It may not perform its own aspect, partition, or
condition interpretation.

### 4.3 Dynamic Rewiring Creates An Epoch Boundary

A worker may propose dependency changes from its immutable snapshot. The
proposal is non-authoritative until reconciliation validates graph legality,
cycle safety, subscription changes, and affected readiness.

Tasks whose legality could change because of a proposed rewire are not admitted
in the same speculative epoch. WORTH may prove narrower independence, but it
may not assume it.

### 4.4 Conflict Partitioning Is Lowered Truth

The planner lowers ready work into ordered conflict partitions. Tasks inside a
partition have a sealed disjointness proof; partitions remain ordered where
their effects overlap.

The executor consumes the partitions. It does not recalculate footprints or
select a more aggressive schedule at runtime.

### 4.5 Publication Remains One Semantic Commit

Worker-local evaluation, comparison, patch construction, subscription delta
construction, and snapshot construction may run concurrently when admitted.
Canonical publication and observable commit remain one transactionally
coherent result.

Physically parallel writes are legal only when storage ownership is disjoint
and a single commit manifest makes their visibility atomic. Otherwise WORTH
publishes serially without changing the parallel-compute claim.

### 4.6 Backpressure Is Part Of Graph Parallelism

Ready queues, prepared packets, rewiring proposals, and unpublished results are
bounded by the Milestone 14 lease. Exhaustion either reduces admitted
concurrency or rejects before new work is spawned. It never creates an
unbounded background lane.

### 4.7 Hierarchical Candidate Lookup May Be Sharded, Not Causality

The reverse-subscription hierarchy may partition candidate lookup by
`ProducerAspectKey` and the scope carried by a committed producer change. Each
worker returns non-authoritative candidates. The existing causal owner then
validates every immediate edge and mints work; deterministic merge is by
semantic cause/work identity, never by shard or completion order.

An exact-leaf change queries its exact and ancestor-covering subscribers. A
subtree change queries lawful descendants and covering ancestors. An unscoped
subscriber is always included. Sibling-disjoint subtrees contribute no
candidate or ready work.

### 4.8 Cross-Shard Boundaries And Rebalancing Are Explicit

Graph shard assignment carries explicit incoming/outgoing dependency
boundaries. A cross-shard edge has the same readiness and control-order law as
an in-shard edge; message or queue arrival cannot establish readiness.

Hot-subtree splitting or shard rebalancing may occur only between graph epochs,
after current publication settles. A `ShardRebalanceProposal` is mechanical
planner input, not topology mutation. Rebalancing cannot change semantic work
identity, dependency revision, cause identity, or canonical publication.

## 5. Required Proof-Bearing Forms

The implementation must establish canonical equivalents of:

```rust
pub struct SettledDependencySet { /* versions and readiness */ }
pub struct ControlOrderProof { /* earlier dynamic effects excluded */ }
pub struct GraphMutationFootprint { /* every synchronous write surface */ }
pub struct DisjointGraphBatch { /* sealed parallel authority */ }
pub struct OrderedConflictPartition { /* one parallel partition */ }
pub struct DependencyRewriteProposal { /* worker-local, non-authoritative */ }
pub struct GraphEpochPublication { /* canonical atomic visibility */ }
pub struct GraphParallelExecutionReport { /* work, span, conflicts, fallback */ }
pub struct GraphShardAssignment { /* non-authoritative admitted-work layout */ }
pub struct CrossShardDependencyBoundary { /* explicit readiness boundary */ }
pub struct ShardRebalanceProposal { /* epoch-bounded physical change */ }
```

The final three forms are planner outputs, not safety authority. Only the
settled-dependency, control-order, and mutation-disjointness progression may
authorize concurrent graph execution.

Authority direction is fixed:

```text
causal invalidation / requested targets
  -> ready semantic work
  -> settled-dependency and control-order proof
  -> mutation-footprint lowering
  -> ordered conflict partitions
  -> worker-local compute and effect proposals
  -> reconciliation and canonical epoch publication
```

Diagnostics derive from this chain and cannot admit work or mint
disjointness.

## 6. Architectural Destination

Milestone 15 populates the committed `execution/graph` topology from Milestone
15:

```text
data/proof/execution/
  graph.rs                         [graph readiness and disjoint batch proof]
  batch.rs                         [prepared batch forms]
  publication.rs                   [epoch publication proof]

logic/planner/execution/
  graph/
    mod.rs                         [stable graph-parallel orchestration]
    readiness.rs                   [dependency readiness]
    antichain.rs                   [causally independent work formation]
    control_order.rs               [dynamic control-dependency proof]
    footprint.rs                   [complete synchronous mutation surface]
    conflict_partition.rs          [ordered disjoint grouping]
    rewiring.rs                    [proposal and reconciliation]
    epoch.rs                       [publication boundary]
    locality.rs                    [hierarchical candidate batch formation]
    shard.rs                       [graph shard assignment]
    cross_shard.rs                 [dependency boundary readiness]
    rebalancing.rs                 [epoch-bounded physical placement]

tests/parallel_execution/
  graph_parallelism.rs
  fixtures/execution_world.rs
  fixtures/schedule_control.rs
  oracle/serial_execution.rs
```

`logic/planner/execution/graph` owns graph-parallel orchestration, not graph
semantic meaning. Invalidation semantics remain under `logic/invalidation`,
condition meaning remains with condition ownership, and backend mechanisms
remain under the execution backend boundary.

Forbidden placements include graph safety rules in Rayon closures, dependency
rewiring inside generic packet merge, effect footprints inferred in apply,
locks used as evidence of semantic independence, or graph-parallel public APIs
that expose internal stage indices.

## 7. Ordered Implementation Phases

### M15.0 - Complete Synchronous Surface Inventory

- enumerate every graph, subscription, snapshot, lineage, observation, and
  canonical diagnostic surface changed synchronously by execution
- make missing footprint membership fail admission rather than default narrow
- add hostile overlap cases for each surface

### M15.1 - Parallel Causal Admission And Planning

- shard already-admitted direct-hop invalidation work
- parallelize pure plan validation, condition preview, and eligibility where
  immutable inputs make them independent
- merge all results by canonical semantic identity
- shard hierarchical candidate lookup without moving causal admission into the
  index or worker

### M15.2 - Antichain And Control-Order Proof

- form ready antichains from settled dependency versions
- establish the dynamic-rewiring control-order boundary
- deny or split batches whose safety depends on unresolved proposals
- make every cross-shard dependency boundary participate in the same readiness
  proof

### M15.3 - Conflict Partitions And Worker-Local Effects

- lower complete mutation footprints
- build ordered disjoint conflict partitions
- produce evaluation, comparison, dependency, snapshot, and apply proposals
  without shared authoritative mutation

### M15.4 - Rewiring Reconciliation And Epoch Publication

- validate proposed dependencies and cycle safety
- re-admit affected later work after the topology epoch changes
- publish one canonical epoch or no epoch on precommit failure
- apply shard rebalancing only after publication settles and prove the next
  assignment is semantically invisible

### M15.5 - Graph-Parallel Certification

- run serial-oracle differential histories under forced schedules
- certify bounded queues, work/span counters, conflict fallback, cancellation,
  restore, and replay
- seal the graph-parallel certification run

## 8. Documentation Deliverables

Milestone 15 must revise:

- `signal_architecture2.md`: graph readiness, control order, conflict
  partitions, hierarchical graph sharding, cross-shard boundaries, and epoch
  publication
- `test-requirements.md`: hostile rewiring, overlap, antichain, and queue-bound
  requirements
- public execution documentation: how graph parallelism is selected and how
  fallback is explained without exposing mechanisms
- performance documentation: named work/span, critical-path, coordination,
  memory, and saturation envelopes

## 9. Must Ship And Must Preserve

Must ship:

- parallel direct-hop invalidation application and pure planning work
- settled-dependency antichains
- explicit dynamic control-order proof
- complete graph mutation footprints and conflict partitions
- worker-local dependency/snapshot/apply proposals
- rewiring reconciliation and atomic epoch publication
- serial/parallel differential certification under hostile schedules
- hierarchical candidate sharding, explicit cross-shard readiness, and
  epoch-bounded hot-subtree rebalancing

Must preserve:

- Milestones 12-13 semantic admission and locality
- Milestone 14 resource, cancellation, and determinism contracts
- transactional rollback and commit-bounded observation
- cycle denial before false graph mutation
- branch, replay, temporal, condition, previous-value, and async semantics

## 10. Explicit Exclusions

Milestone 15 does not:

- expose partitioned computation inside one node
- introduce domain-specific kernels or partition types
- infer geometry adjacency, spatial distance, or host topology from scope paths
- implement accelerator or distributed execution
- allow arbitrary cyclic asynchronous computation
- promise speedup for a graph whose work is dominated by its critical path,
  shared publication, or memory bandwidth
- remove serial execution or explicit serial conflict partitions

## 11. Acceptance Evidence

Milestone 15 closes only when:

- serial and graph-parallel mutation histories agree through the independent
  serial oracle
- forced schedule permutations preserve canonical output, graph, snapshot,
  replay, and explanation artifacts
- no task observes an unsettled predecessor version
- exact/subtree/unscoped hierarchical candidate batches equal the independent
  serial candidate oracle, with zero sibling-disjoint work
- dynamic rewiring cannot reveal a dependency behind already-executed later
  work
- every synchronous mutation surface participates in conflict proof
- overlapping tasks are ordered and disjoint tasks can execute concurrently
- cancellation/failure before epoch publication leaves no partial epoch
- graph placement changes preserve admitted work, dependency revisions, cause
  identities, and canonical history
- ready queues and unpublished packets stay within named bounds under wide
  frontier pressure
- reports expose total work, span, critical path, conflict width, active
  concurrency, queue width, fallbacks, publication breadth, per-shard work,
  cross-shard boundaries/messages, migrations, and imbalance
- removing control-order proof, one footprint axis, or canonical epoch
  publication turns evidence red
- focused tests, complete affected suites, boundary checks, context checks,
  formatting, and dirty Rust line-cap checks pass

## 12. Successor Handoff

Milestone 16 may compose nested partition work inside graph tasks, but every
child task must subdivide the same resource lease and return one node-local
result compatible with Milestone 15's immutable-input and epoch-publication
contracts.
