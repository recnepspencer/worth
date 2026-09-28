# Milestone 16 Engineering Spec: Structured Partitioned Parallelism

> **Status:** Planned
>
> **Prerequisite:** [milestone-15-plan.md](./milestone-15-plan.md)
>
> **Architecture parent:** [signal_architecture2.md](./signal_architecture2.md), `S9.17.3`
>
> **Successor:** [milestone-17-plan.md](./milestone-17-plan.md)

## 1. Goal And Roadmap Placement

Milestone 16 gives computation authors a domain-agnostic structured parallelism
surface for work inside one WORTH node.

Milestone 15 exploits independence between graph nodes. Expensive computations
can still dominate the critical path when one node contains a large collection,
spatial structure, numerical workload, document, image, simulation, or other
partitionable value. Milestone 16 allows that node to declare stable partitions
and lawful composition without spawning threads or naming an execution backend.

The milestone is infrastructure. Geometry is one future consumer and one
hostile certification workload; it is not part of the crate's ontology, API
names, proof vocabulary, or module topology.

Structured partitions may align with Milestone 14's hierarchical semantic
locality and Milestone 15's physical graph shards, but those are independent
axes. A scope path narrows meaning, a work partition defines lawful inner
computation, and a shard assignment selects physical placement.

## 2. Current Boundary

Node evaluators currently receive a read view and return one prepared
evaluation. They can use arbitrary host code internally, but WORTH cannot:

- inspect or bound nested concurrency
- prove that inner writes are disjoint
- prevent graph-level and inner parallel work from oversubscribing resources
- prescribe deterministic partition and reduction order
- report inner work, span, memory, or backend eligibility
- replay or explain the execution shape that affected an order-sensitive result

A consumer can call Rayon or another library inside its evaluator, but that
work is invisible to WORTH's resource, cancellation, determinism, and
certification contracts. Milestone 16 replaces that invisible lane with
structured declaration and runtime-carried execution context.

## 3. Adversarial Courtroom

Define domain-neutral certification computations that exercise all supported
patterns:

- a map over at least `10^7` elements with highly skewed per-partition cost
- a two- and three-dimensional tiled computation with boundary reads
- a depth-eight opaque scope hierarchy with partitions both aligned and
  deliberately misaligned to physical shards
- fork/join recursion with unbalanced branches
- deterministic integer, exact-value, and floating-point reductions
- prefix scan with partition boundaries at hostile positions
- a synchronous iterative computation whose convergence takes many rounds
- nested partition work inside a wide graph antichain
- empty, singleton, tiny, dense, and memory-saturating inputs
- cancellation and deadline expiry inside map, reduction, scan, and round
  boundaries
- worker budgets from one to available parallelism and a platform exposing only
  serial execution
- branch, restore, replay, and repeated schedule perturbation

Required result:

- the same declaration executes correctly with a one-worker or multi-worker
  lease
- nested work never exceeds the parent lease
- each partition has stable identity and explicit read/write footprint
- deterministic modes produce the declared canonical result independently of
  worker schedule and count
- reduction law requirements are checked before execution
- round `n + 1` cannot observe a partially published round `n`
- cancellation reports the exact partition/round progress boundary
- no concrete thread, worker, device, or transport type enters the declaration
- explicit boundary/halo reads remain lawful across shard placement and are
  accounted rather than hidden as local access

The courtroom must convict:

- user-authored `.parallel_safe(true)` assertions
- partition identity derived from worker index or completion order
- inner thread-pool creation
- floating-point `reduce` whose result silently changes with worker count
- a scan implemented as an unordered reduction
- iterative work with no declared convergence, round, or exhaustion contract
- partition-local mutation that aliases another partition's write set
- a serial fallback that changes partition or reduction semantics

## 4. Product Decision Lock

### 4.1 Structured Patterns Are The Public Surface

The supported foundational patterns are:

- `map`: independent application over stable work items
- `reduce`: associative composition with an explicit identity and determinism
  contract
- `scan`: ordered prefix/suffix composition
- `fork_join`: recursively decomposable independent subcomputations
- `rounds`: bulk-synchronous iterative computation with explicit convergence

Raw spawn, raw threads, worker indices, backend queues, and shared mutable
callbacks are not public computation-authoring APIs.

### 4.2 Partitions Carry Read And Write Meaning

A partition declaration identifies stable logical work and its read/write
footprints. `PartitionWriteSet` must be disjoint across concurrently admitted
partitions. `PartitionReadSet` may overlap writes only where the declared
pattern provides a synchronization boundary, such as reading the prior
iteration or prior round.

These names remain domain-neutral. A consumer may interpret a partition as a
mesh region, image tile, ledger shard, matrix block, document section, or any
other domain concept outside `worth-signal`.

### 4.3 Safety Is Framework-Minted

Computation authors declare semantic partitioning, access, reduction, and
convergence contracts. WORTH validates and lowers them into sealed execution
proof. A declaration cannot directly construct `DisjointPartitionBatch`,
`DeterministicReductionPlan`, or other execution authority.

### 4.4 Deterministic Reduction Is Explicit

Reduction contracts distinguish:

- canonical fixed-tree reduction for bitwise stability
- contract-equivalent reduction for declared associative/equivalent values
- relaxed reduction only where the declaration and request permit it

Floating-point associativity is never inferred. A reducer that cannot satisfy
its requested determinism contract is denied before dispatch or resolved to a
lawful certified implementation.

### 4.5 Nested Parallelism Subdivides One Lease

Partition work receives a child execution context derived from its graph task's
Milestone 14 lease. It may subdivide that lease recursively. It cannot allocate
independent worker capacity.

The scheduler may choose graph breadth, partition breadth, or a mixture based
on work, span, locality, and memory pressure while preserving the lowered plan.

### 4.6 Iteration Is Bulk-Synchronous By Default

Each round reads one committed round image, produces worker-local partition
results, reduces them canonically, and publishes the next image atomically.
Convergence, maximum rounds, cancellation points, and non-convergence outcomes
are declared.

Asynchronous fixed-point execution is excluded unless a later specification
introduces a narrower monotone convergence authority. It is not an optional
flag on ordinary rounds.

### 4.7 Execution Placement Is Not Domain Meaning

A partition declaration may state portable requirements such as memory class,
data layout, numerical capabilities, and allowed determinism. It does not name
CPU, Rayon, Web Worker, GPU, or remote transport. Milestone 17 maps lowered
requirements to certified backend capabilities.

### 4.8 Scope Hierarchy, Work Partition, And Shard Are Separate Axes

A computation may bind a stable work partition to one or more admitted
`ScopePath` subtrees to improve data locality. That binding does not make the
partition an invalidation cause and does not make the scope path a write-set
proof. Read/write validation remains authoritative for structured execution.

Boundary or halo reads crossing a partition or shard are declared explicitly
in the read set and included in memory/coordination estimates. The planner may
co-locate related partitions or split a hot subtree, but it cannot change
stable partition identity, access legality, reduction order, or semantic work.

## 5. Required Proof-Bearing Forms And Caller DX

The implementation must establish canonical equivalents of:

```rust
pub struct StableWorkPartition { /* semantic identity and bounds */ }
pub struct PartitionReadSet { /* immutable inputs and prior-round reads */ }
pub struct PartitionWriteSet { /* exclusive result ownership */ }
pub struct PartitionComputation<I, O> { /* declarative structured work */ }
pub struct DisjointPartitionBatch { /* framework-minted execution proof */ }
pub struct DeterministicReductionPlan { /* identity, tree, join order */ }
pub struct ScanPlan { /* ordered partition and carry contract */ }
pub struct SynchronousRoundPlan { /* state, convergence, exhaustion */ }
pub struct PartitionExecutionReport { /* work, span, memory, reductions */ }
pub struct PartitionLocalityBinding { /* admitted scope-to-work association */ }
pub struct PartitionBoundaryReadSet { /* explicit cross-partition inputs */ }
```

Illustrative authoring shape:

```rust
let aggregate = PartitionComputation::over(input)
    .partition_by(stable_partitioner)
    .map_partition(map_partition)
    .reduce(DeterministicReducer::canonical(identity, combine));
```

The final facade may use builders, definition functions, or traits, but it must
preserve ordered proof progression and keep execution mechanisms absent.

## 6. Architectural Destination

Milestone 16 populates the committed partition topology:

```text
data/proof/execution/
  partition.rs                    [stable partition and access proof]
  partition_locality.rs           [scope binding without invalidation authority]
  reduction.rs                    [reduction/scan/round proof]

logic/planner/execution/
  partition/
    mod.rs                        [stable structured-parallel facade]
    declaration.rs                [pattern and semantic contract]
    access.rs                     [read/write-set validation]
    boundary.rs                   [halo and cross-shard read declaration]
    placement.rs                  [partition-to-shard lowering]
    lowering.rs                   [proof-bearing execution form]
    map.rs                        [independent application]
    reduction.rs                  [canonical reduction planning]
    scan.rs                       [ordered scan planning]
    fork_join.rs                  [recursive decomposition]
    rounds.rs                     [bulk-synchronous iteration]
    reporting.rs                  [structural work/span evidence]

tests/parallel_execution/
  partitioned_parallelism.rs
  fixtures/partition_workloads.rs
  oracle/serial_execution.rs
```

The structural axis is execution pattern, not consumer domain or backend.
Pattern declarations own semantic requirements; lowering owns proof; backends
only execute the proof-bearing result.

Forbidden placements include geometry-named modules or types, consumer-owned
thread pools, backend branches in reducers, partition safety in diagnostics,
generic `parallel_helpers`, or one catch-all kernel module.

## 7. Ordered Implementation Phases

### M16.0 - Pattern And Access Contract Freeze

- freeze map, reduce, scan, fork/join, and synchronous-round semantics
- define stable partition identity and read/write sets
- add compile-fail proof that declarations cannot forge execution authority
- freeze the distinction among hierarchical scope, stable work partition, and
  physical shard placement

### M16.1 - Partition Validation And Lowering

- validate coverage, overlap, identity stability, access legality, and resource
  fit before dispatch
- mint disjoint partition batches and explicit serial fallback plans
- expose inspectable work/span and memory estimates
- validate locality bindings and make every boundary/halo read explicit before
  shard-aware lowering

### M16.2 - Map And Fork/Join Execution

- execute independent and recursively decomposed work through child leases
- preserve cancellation, deadline, worker-local result, and canonical
  publication contracts
- certify skewed-work load balancing without worker-index semantics
- compare hierarchy-aligned, deliberately misaligned, and rebalanced placement
  without changing stable work identity

### M16.3 - Deterministic Reduce And Scan

- establish fixed-tree and contract-equivalent reduction paths
- implement ordered carry propagation for scan
- deny illegal or unsupported determinism before execution

### M16.4 - Synchronous Iterative Rounds

- establish round images, canonical reduction, atomic round publication,
  convergence, exhaustion, and cancellation outcomes
- preserve replay and branch reconstruction of round conclusions

### M16.5 - Nested And Platform-Neutral Certification

- combine wide graph antichains with deep partition work under one lease
- certify serial-only and multi-worker execution from the same declaration
- seal the structured-partition certification run

## 8. Documentation Deliverables

Milestone 16 must create or revise developer-facing documentation for
computation authors covering:

- choosing map, reduce, scan, fork/join, or rounds
- declaring stable partition identity and access sets
- determinism and numerical reduction consequences
- nested resource budgeting and cancellation
- serial-only platform behavior
- inspecting resolved plans and execution reports
- binding work to opaque hierarchical locality and declaring boundary reads

Examples must be domain-neutral and executable. Domain-specific crates may add
their own guides later without becoming authority over this contract.

## 9. Must Ship And Must Preserve

Must ship:

- domain-neutral partition declarations and proof lowering
- map, reduce, scan, fork/join, and synchronous-round patterns
- explicit read/write-set disjointness
- deterministic reduction plans
- hierarchical lease composition
- serial and parallel execution parity
- structural work, span, memory, and coordination reports
- shard-aware placement with explicit boundary/halo traffic

Must preserve:

- Milestone 14 resource/cancellation/determinism truth
- Milestone 15 graph control-order and epoch publication
- serial execution as canonical oracle and complete platform posture
- branch, replay, rollback, observation, temporal, and async semantics
- backend and consumer-domain independence

## 10. Explicit Exclusions

Milestone 16 does not:

- implement or name a geometry kernel
- treat scope paths as a spatial index or infer domain adjacency
- expose GPU, Web Worker, Rayon, SIMD, or network APIs
- provide arbitrary shared-memory mutation between partitions
- infer floating-point associativity
- support ungoverned asynchronous cyclic convergence
- promise speedup below measured grain-size, locality, or memory-bandwidth
  thresholds

## 11. Acceptance Evidence

Milestone 16 closes only when:

- all structured patterns equal the independent serial oracle across worker
  counts and forced schedules
- nested graph and partition work never exceed one hierarchical lease
- overlapping write sets are rejected or serialized before dispatch
- stable partition identity is independent of worker count and schedule
- aligned, misaligned, and rebalanced shard placements preserve partition,
  access, and result identity
- canonical reductions remain bitwise stable where claimed
- scan preserves exact declared order
- iterative rounds expose convergence, exhaustion, cancellation, and round
  publication truth
- serial-only capability executes the same declaration without semantic drift
- reports expose partitions, logical items, work, span, steals, reductions,
  barriers, boundary reads, cross-shard bytes, residency, peak memory, and
  fallback reasons
- mutation probes against access validation, fixed reduction order, and lease
  subdivision turn evidence red
- focused tests, complete affected suites, boundary checks, context checks,
  formatting, and dirty Rust line-cap checks pass

## 12. Successor Handoff

Milestone 17 may execute prepared graph and partition batches on additional
platforms. It may not reinterpret partition meaning, weaken determinism,
reconstruct safety, or make transport/device placement part of computation
semantics. Semantic scope paths and physical shard assignments must remain
separate fields across every backend boundary.
