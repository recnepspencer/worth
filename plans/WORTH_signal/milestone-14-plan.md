# Milestone 14 Engineering Spec: Deterministic Parallel Execution Foundation

> **Status:** Planned
>
> **Prerequisites:**
> - [milestone-12-plan.md](./milestone-12-plan.md)
> - [milestone-13-plan.md](./milestone-13-plan.md)
> - [milestone-13.1-plan.md](./milestone-13.1-plan.md)
>
> **Architecture parent:** [signal_architecture2.md](./signal_architecture2.md), `S9.17`
>
> **Successor:** [milestone-15-plan.md](./milestone-15-plan.md)

## 1. Goal And Roadmap Placement

Milestone 14 turns WORTH's existing stage-local parallel mechanisms into one
truthful, deterministic execution foundation with runtime-owned resource
authority.

The current runtime already has valuable pieces: immutable execution snapshots,
parallel precompute, worker-local apply packets, disjoint apply footprints, and
canonical stage-order publication. The missing foundation is one authority that
decides how much parallel work may exist, which execution capabilities are
available, how nested work consumes the same budget, how cancellation is
observed, and what determinism contract the resulting execution must satisfy.

Milestone 14 establishes that foundation before Milestones 15 and 16 widen the
amount and shape of parallel work. It does not yet claim that the complete graph
or the inside of arbitrary computations runs in parallel.

It also establishes the locality substrate those wider forms need at massive
scale: M13's exact partition/detail subscription is generalized into a bounded,
domain-agnostic hierarchy, and the execution planner may derive a separate
physical shard placement from that hierarchy. Semantic locality says which work
may matter. Physical sharding says where admitted work may run. A shard key is
never dependency, invalidation, readiness, or commit authority.

Its input is Milestone 13's Signal-owned `ReadyInvalidationBatch` progression,
as carried into real Runtime Bridge and Query platform composition by Milestone
13.1, not a raw node list or a Foundational report. Parallel preparation must
consume the current-basis ready form and preserve its cause/work identity. A
resource lease authorizes dispatch capacity; it does not replace invalidation
readiness, Bridge correspondence, or Query impact admission.

## 2. Current Boundary

Present execution has four important limitations:

- `ParallelExecutionPolicy.worker_count` influences chunking and admission, but
  the shared Rayon pool is created from machine availability and does not make
  the requested worker count a strict resource lease
- graph-level and future nested computation parallelism have no shared budget,
  so composing them could oversubscribe the same machine
- `StageExecutor` mixes caller posture with a concrete execution shape and does
  not describe platform capability, deadline, cancellation, or determinism
- the native parallel implementation is mechanically close to the planner,
  while serial, native-threaded, WASM-worker, accelerator, and remote execution
  need one semantic contract without becoming one mechanism
- M13's certified locality index has exact unscoped, partition, and detail
  lanes, but does not yet represent a deeper host-defined hierarchy or a
  deterministic physical placement plan over admitted work

The existing grouped-concurrent apply restriction remains honest: dependency
updates, rewiring, output-identity comparison, or overlapping mutation
footprints currently force serial apply. Milestone 14 must not remove those
fallbacks until a later milestone carries stronger proof.

## 3. Adversarial Courtroom

Run one production-valid workload through the public runtime composition root
under all of these conditions:

- serial execution, a one-worker parallel budget, and a multi-worker budget
- repeated runs with different physical task-stealing schedules
- nested runtime evaluation requests competing for one process-wide budget
- cancellation before dispatch, during worker-local compute, after local packet
  construction, and immediately before canonical publication
- a deadline expiring at each of the same boundaries
- worker panic or typed compute failure in the earliest, middle, and latest
  task of a stage
- graph stages containing disjoint work, overlapping apply footprints,
  dependency rewiring, output-identity comparison, async-capable nodes, and
  comparator suppression
- scope paths at depths `1`, `2`, `4`, and `8`, including exact-leaf,
  ancestor-subtree, sibling-disjoint, and unscoped subscriptions
- a heavily skewed hot subtree whose physical shard assignment is split and
  rebalanced between execution epochs
- branch capture, rollback, restore, replay, and deterministic rerun
- a target configuration with no parallel execution capability

Required outcome:

- the configured lease is never exceeded, including nested work
- serial and parallel executions commit the same canonical semantic artifacts
- task-stealing order never becomes publication order
- cancellation and timeout report typed progress and never masquerade as
  rollback
- a pre-publication failure commits no partial stage truth
- an unsupported or unprofitable parallel request resolves before dispatch to
  an explicit serial plan or typed denial according to declared policy
- lack of parallel capability changes throughput, never meaning
- the M13 partition/detail courtroom remains an exact two-segment
  specialization of the hierarchical contract
- changing worker count or physical shard placement changes no admitted cause,
  canonical artifact, replay identity, or explanation

The courtroom must convict:

- a worker-count hint that permits more active workers than its lease
- a new thread pool per request or per nested computation
- hidden parallel fallback that is absent from the resolved plan and report
- a worker mutating authoritative graph state directly
- completion-order publication
- cancellation that drops a handle while untracked work continues
- bitwise-deterministic claims backed only by semantic equality
- treating a scope path, `ProducerAspectKey`, or physical shard key as proof
  that an edge, snapshot, cause, or work item is current
- copying one producer's aspect or scope path transitively onto a descendant

## 4. Product Decision Lock

### 4.1 Parallelism Is Resolved Policy, Not Caller-Asserted Safety

Callers may declare execution posture, maximum resource budget, deadline,
cancellation, and determinism requirements. They may not declare a task
`parallel_safe` or forge a disjointness proof.

The framework derives safety from lowered dependencies, control ordering,
mutation footprints, and backend capability.

### 4.2 One Runtime-Owned Resource Authority

Every graph-level and nested parallel task consumes a lease from one bounded
runtime resource authority. A lease proves the maximum active concurrency,
memory allowance, and applicable cancellation/deadline context.

A child operation may subdivide its parent's lease. It may not create new
capacity. Physical executor threads may be shared across runtimes, but logical
leases, fairness, and accounting remain runtime-owned and inspectable.

### 4.3 Determinism Is A Contract Family

The public contract must distinguish at least:

```rust
pub enum DeterminismContract {
    CanonicalBitwise,
    ContractEquivalent(EquivalenceContractId),
    RelaxedThroughput,
}
```

`CanonicalBitwise` fixes canonical partition, reduction, and publication order
where numerical results depend on ordering. `ContractEquivalent` requires an
explicit equivalence contract. `RelaxedThroughput` is unavailable unless the
computation declaration and caller both permit the weaker guarantee.

No backend may silently weaken the resolved contract.

### 4.4 Compute Is Local; Publication Is Canonical

Workers consume immutable read snapshots and return worker-local result or
effect packets. Authoritative graph state is changed only through the existing
transaction/apply authority after validation and canonical ordering.

Disjoint physical publication may be introduced later, but observable commit,
replay, history, and explanation order must remain canonical.

### 4.5 Cancellation Is Managed Lifecycle

An execution request returns a framework-owned handle whenever work may
outlive the initiating stack. The handle exposes status, cancellation,
deadline, progress, final outcome, and disposal.

Cancellation stops admission at named safe points. It does not undo committed
work and must report whether no work, worker-local work, prepared publication,
or committed work existed when cancellation was observed.

### 4.6 Platform Capability Is Explicit

Parallel support is a resolved capability, not a build-target assumption.
Serial execution remains a complete implementation of the same lowered
contract. Feature flags and target configuration may change available backend
membership but may not change semantic meaning, authority, or proof topology.

### 4.7 Existing Executor Inputs Must Converge

`ParallelismHint`, `StageExecutor`, and `ParallelExecutionPolicy` are present
public/specialist inputs, but they may not survive as a parallel authority lane
beside the new request and lease model.

The migration is fixed:

- `ParallelismHint::Serial | Preferred` lowers to
  `ExecutionPosture::Serial | Automatic`
- public worker count becomes a strict `max_workers` budget, never a pool-size
  promise or chunking-only hint
- chunk size, apply group width, and similar executor mechanics become resolved
  planner policy and remain inspectable in the plan rather than caller-owned
  safety controls
- `StageExecutor` becomes an internal resolved executor form or is removed;
  ordinary `*_with_executor` entry points migrate to request policy
- any temporary deprecated surface must lower immediately through the one new
  authority, state its semantic difference, and have a named removal point

A permanent compatibility executor path or second pool registry is forbidden.

### 4.8 Hierarchical Locality Is Domain-Agnostic Semantic Narrowing

M13's partition/detail form becomes the two-segment base case of a bounded
canonical `ScopePath`. Path segments are opaque host-defined identity. The
runtime understands only exact, ancestor, descendant-subtree, and disjoint
relationships; it does not understand assembly, component, body, region,
element, geometry, spatial distance, or topology.

The reverse-subscription owner constructs a private `ProducerAspectKey` only
from authoritative dependency edges or committed producer output identity. It
indexes unscoped subscribers and hierarchical scope buckets. The key and index
are non-authoritative candidate-selection structures: causal admission must
still validate the immediate edge, snapshot, dependency revision, comparator,
and exact scope relationship. Producer-local aspect identity is never copied
across a transitive hop.

`ProducerAspectKey`, `ScopePath`, and the subscription hierarchy are
`worth-signal` graph-runtime vocabulary. They do not move to `worth-proof` or
`worth-foundational`. Proof owns operation legality; Foundational may
canonicalize cross-boundary evidence identity but does not acquire graph
topology or locality authority.

### 4.9 Physical Sharding Is Derived Placement, Not Semantic Authority

After semantic work is admitted, the planner may derive an opaque
`ExecutionShardKey` and deterministic `ShardAssignmentPlan` from current work,
scope hierarchy, cost evidence, memory constraints, and the active resource
lease. A shard selects co-location and dispatch; it cannot admit work, suppress
work, change dependencies, establish currentness, or publish graph truth.

Rebalancing occurs only between named execution epochs. The planner records the
old and new assignment, affected resident bytes, migrations, and cross-shard
boundaries. Rebalancing the same admitted batch must preserve semantic work
identity and canonical publication.

## 5. Required Proof-Bearing Forms And Caller DX

The implementation must establish canonical equivalents of:

```rust
pub struct ExecutionRequestPolicy {
    posture: ExecutionPosture,
    determinism: DeterminismContract,
    budget: ExecutionBudget,
    deadline: Option<ExecutionDeadline>,
}

pub struct ResolvedExecutionCapabilities { /* platform and backend facts */ }
pub struct ExecutionResourceLease { /* sealed bounded authority */ }
pub struct PreparedExecutionBatch { /* immutable work and ordering proof */ }
pub struct WorkerLocalExecutionPacket { /* non-authoritative result */ }
pub struct CanonicalPublicationPlan { /* validation and order */ }
pub struct ExecutionOutcomeEnvelope { /* typed progress and cost */ }
```

The runtime-locality and placement owners separately establish these
non-authoritative forms:

```rust
pub struct ScopePath { /* bounded canonical host-defined segments */ }
pub(super) struct ProducerAspectKey { /* private reverse-index lookup */ }
pub(super) struct ExecutionShardKey { /* private physical placement */ }
pub struct ShardAssignmentPlan { /* deterministic non-authoritative layout */ }
```

Ordinary caller intent should read like:

```rust
let outcome = runtime
    .evaluate_many(targets)
    .with_execution_policy(ExecutionRequestPolicy::automatic())
    .with_determinism(DeterminismContract::CanonicalBitwise)
    .run()?;
```

Advanced callers may bound workers, memory, deadline, and cancellation. They
must not select Rayon, Web Workers, or any other backend mechanism through the
core semantic facade.

## 6. Architectural Destination

Milestones 14-17 commit to this destination topology:

```text
crates/worth-signal/src/
  data/proof/execution/                         [created responsibility family]
    mod.rs                                      [stable proof facade]
    capability.rs                               [Milestone 14]
    budget.rs                                   [Milestone 14]
    determinism.rs                              [Milestone 14]
    batch.rs                                    [Milestones 14-15]
    publication.rs                              [Milestones 14-15]
    placement.rs                                [Milestone 14; non-authoritative layout]
    graph.rs                                    [committed Milestone 15]
    partition.rs                                [committed Milestone 16]
    reduction.rs                                [committed Milestone 16]
    backend.rs                                  [committed Milestone 17]
    certification/
      mod.rs
      case.rs
      equivalence.rs
      cost.rs
      run.rs
  logic/planner/
    execution/                                  [existing orchestration owner]
      mod.rs
      admission.rs                              [created/replaced]
      resource_authority.rs                     [created Milestone 14]
      cancellation.rs                           [created Milestone 14]
      dispatch.rs                               [created/replaced]
      publication.rs                            [created from current reduction]
      placement/                                [created Milestone 14 child]
        mod.rs                                  [stable internal placement facade]
        assignment.rs                           [deterministic shard assignment]
        rebalancing.rs                          [epoch-bounded placement changes]
      graph/                                    [committed Milestone 15 child]
        mod.rs
        antichain.rs
        control_order.rs
        conflict_partition.rs
      partition/                                [committed Milestone 16 child]
        mod.rs
        declaration.rs
        lowering.rs
        reduction.rs
        rounds.rs
      backend/                                  [committed Milestone 17 child]
        mod.rs                                  [stable backend port]
        serial.rs
        native.rs
  data/graph/topology/subscriber_index/         [extended Milestone 14]
    producer_aspect_key.rs                      [private immediate-producer key]
    scope_path.rs                               [bounded canonical hierarchy]
    hierarchy.rs                                [derived subscriber candidate index]
  tests/parallel_execution/                     [created responsibility family]
    mod.rs
    resource_authority.rs                       [Milestone 14]
    graph_parallelism.rs                        [committed Milestone 15]
    partitioned_parallelism.rs                  [committed Milestone 16]
    backend_conformance.rs                      [committed Milestone 17]
    fixtures/
      execution_world.rs
      schedule_control.rs
    oracle/
      serial_execution.rs
```

The stable facade remains `facade.rs` and `facade/specialist.rs`. Proof forms
are re-exported selectively; executor mechanisms remain internal.

The dominant structural axes are proof truth under `data/proof/execution`,
execution lifecycle under `logic/planner/execution`, and replaceable external
mechanism under `execution/backend`.

Forbidden placements include Rayon-named public types, WASM or geometry types
inside `worth-signal`, backend selection inside node semantic definitions,
parallel safety booleans, generic executor helpers, and diagnostics that mint
execution authority.

## 7. Ordered Implementation Phases

### M14.0 - Contract And Capability Freeze

- inventory every serial/parallel plan, policy, report, pool, and fallback
- freeze execution posture, determinism, budget, cancellation, and capability
  distinctions
- add failing proof for the current worker-count non-enforcement
- freeze `ScopePath` depth/canonicalization and the exact M13 two-segment
  compatibility law
- freeze the authority separation between semantic locality and physical shard
  placement

### M14.1 - Runtime Resource Authority

- establish sealed resource leases and hierarchical subdivision
- replace per-call worker interpretation with bounded shared scheduling
- account active tasks, queue width, memory, steals, and nested lease use
- make shard occupancy, resident bytes, and placement fairness consume that
  same bounded authority

### M14.2 - Public Policy Migration

- migrate `ParallelismHint`, `StageExecutor`, `ParallelExecutionPolicy`, and
  `*_with_executor` callers to the request/lease authority
- move mechanical chunking/group controls into resolved planner policy
- delete the old ordinary execution path after all callers and examples move

### M14.3 - Prepared Batch And Publication Boundary

- make every worker consume immutable prepared work
- consolidate worker-local packet and canonical publication forms
- preserve serial and grouped-concurrent apply through the same lowered form
- attach deterministic non-authoritative shard assignments only after semantic
  work admission and exclude placement from canonical result identity

### M14.4 - Managed Cancellation And Failure Atomicity

- introduce the execution handle and named safe points
- prove pre-publication failure/cancellation atomicity
- preserve typed partial or committed outcomes after the commit boundary

### M14.5 - Determinism And Capability Certification

- compare serial and varied physical schedules through an independent serial
  oracle
- certify no-capability target behavior
- certify hierarchical exact/subtree/disjoint lookup and placement/rebalance
  invariance across worker budgets
- seal the Milestone 14 certification run

## 8. Documentation Deliverables

Milestone 14 must revise:

- `signal_architecture2.md`: execution resource authority, determinism,
  publication boundary, hierarchical locality, and non-authoritative placement
- `WORTH_signal_vision.md`: replace stale precompute-only status
- `test-requirements.md`: schedule perturbation, budget, cancellation, and
  publication atomicity requirements
- the public `worth-signal` execution documentation: ordinary and advanced
  policy examples, typed outcomes, and platform-capability behavior
- deprecation/migration guidance for `ParallelismHint`, `StageExecutor`,
  `ParallelExecutionPolicy`, and `*_with_executor` entry points

Examples must compile against the real facade and must not mention a concrete
backend in ordinary usage.

## 9. Must Ship And Must Preserve

Must ship:

- one bounded resource authority and hierarchical leases
- explicit capability resolution
- determinism contract family
- managed cancellation/deadline lifecycle
- immutable prepared batches and canonical publication
- serial/parallel/capability certification with structural counters
- bounded hierarchical locality plus deterministic shard placement and
  epoch-bounded rebalancing

Must preserve:

- Milestones 12-13 invalidation correctness and locality certification
- M13 partition/detail behavior as an exact two-segment hierarchy
- the Proof-enforced prepared -> committed -> lowered -> ready -> executed
  invalidation progression and current-basis work binding
- the rule that Foundational receipts describe performed work but authorize no
  scheduling or execution
- one lowered semantic plan for serial and parallel execution
- rollback-safe, commit-bounded observation
- branch, replay, temporal, condition, and async-capability truth
- explicit serial fallback for work not yet proved safe in parallel

## 10. Explicit Exclusions

Milestone 14 does not:

- parallelize every graph phase
- add public partitioned computation APIs
- introduce geometry, image, simulation, or other domain vocabulary
- interpret hierarchy segments as spatial/topological meaning or build a
  geometry search index
- implement GPU or remote execution
- weaken deterministic publication for throughput
- treat serial resolution on a nonparallel platform as an error when policy
  permits a correct serial plan

## 11. Acceptance Evidence

Milestone 14 closes only when:

- active work never exceeds its resource lease under nested pressure
- serial, one-worker, and multi-worker runs produce identical canonical
  artifacts under `CanonicalBitwise`
- schedule perturbation cannot change publication, replay, or explanation order
- cancellation and deadline outcomes identify the exact progress boundary
- worker failure before publication commits no partial stage truth
- unsupported and unprofitable parallel requests resolve explicitly before
  dispatch
- worker-local code has no authoritative graph mutation capability
- exact-leaf and ancestor-subtree changes query only the lawful hierarchy
  buckets, while sibling-disjoint changes contribute zero candidates and work
- path depth, worker count, shard assignment, and epoch-bounded rebalancing do
  not change admitted work or canonical outputs
- counters expose active workers, queue breadth, steals, nested lease breadth,
  local packets, publication breadth, cancellation points, fallback reason,
  hierarchy probes, shard occupancy, migrations, resident bytes, and imbalance
- mutation probes against lease enforcement, canonical ordering, and
  worker-local isolation turn evidence red
- focused tests, complete affected suites, boundary checks, context checks,
  formatting, and dirty Rust line-cap checks pass

## 12. Successor Handoff

Milestone 15 may trust resource leases, immutable worker input, canonical
publication, and determinism contracts. It must derive graph parallelism from
dependency, control-order, and mutation-footprint proof rather than caller
assertion.
