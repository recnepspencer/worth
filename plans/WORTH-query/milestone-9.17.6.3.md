# Milestone 9.17.6.3: Exact Invalidation And Parallel Computation

> **Status:** Not started. Successor to [9.17.6](./milestone-9.17.6.md). This is the
> one plan for exact invalidation and parallel execution in WORTH. It replaces the
> canceled Signal Milestones 14 to 17 and the 9.17.6 rules that kept touched
> records as evidence only. Every phase depends only on completed work. Portable
> and distributed backends are deferred; see [deferred work](../deferred-work.md).

## Goal And Placement

This milestone has two measures, and they share one key.

1. **Exact invalidation.** The touched graph drives recompute. A commit's sealed
   touched records mark exactly the outputs whose consumed facts they intersect,
   at the smallest granule that holds them. Only those outputs recompute, and
   nothing else is re-checked. The cost of keeping outputs current grows with the
   change, not with the model.
2. **Parallel computation.** What does recompute runs in parallel wherever the
   declarations make it independent, with the same answer at every worker count.

Any application on WORTH gets parallel computation by declaring what is
independent. It never spawns a thread, sizes a pool, picks a backend or asserts
that its own work is safe to run concurrently. The platform finds the parallelism
the declarations permit, at every layer, runs it inside one bounded resource
authority, and proves the answer is identical to serial execution.

This milestone carries both measures through every layer that needs them:

| Layer | What changes |
| --- | --- |
| Execution runtime (new `worth-execution`) | One authority: pool, leases, capability, cancellation, patterns, partitioners, serial oracle, work and span accounting |
| Relational | Touched records for every observable revision, including index membership; aspect versions bumped only by a changed field; query fragments, index builds, validation, bulk create and commit preparation on the caller's lease with stable partitions |
| Signal | Independent ready graph work under three proofs, candidate lookup over a scope-path hierarchy, precompute and apply, all with canonical publication; the loosest observation tier is renamed `Visited` |
| Runtime Bridge | Delivers every committed patch envelope to one Query-owned subscription at full precision, including scope paths at the depth Signal seals them; carries the caller's lease |
| Query | Settle-time reverse index, commit-time marking, clean reuse, input and output cutoff and one-call advancement; partitioned managed computations with per-partition reuse; independent computations inside one `advance`; workflow frontier stages; derived-view reconstruction |
| Server | Shared-read batches on the request lease |
| World | Composes one authority per process for the runtimes it builds |
| WASM | Resolves the serial posture and runs every declaration unchanged |

The platform owns:

- touched-driven marking, reuse and cutoff;
- the execution authority and the lease hierarchy;
- structured patterns (map, reduce, scan, fork/join, bulk-synchronous rounds, and
  decomposition with an interface stage);
- partitioners (by key, into connected components, by recursive bisection);
- the canonical reduction tree and per-partition reuse;
- scheduling across independent work;
- determinism certification and work and span accounting.

Domains own what a partition means (a ledger region, a document section, an image
tile, a mesh region, a load path, an assembly) and the kernels that compute it.
Numerical kernels such as a sparse factorization belong in domain or numerical
libraries, not in Query and not in the execution runtime.

**The same key serves parallelism and reuse.** A partition is both the unit of
parallel work and the unit of reuse. Its settlement records the facts it consumed,
so when a commit's touched graph intersects those facts, marking lands on that
partition and nothing else. The touched graph is the only cause of invalidation.
A partition key, a scope path and a physical shard are never an invalidation
cause and never a currentness proof.

**This milestone sets the precedent.** From here on, no layer re-checks clean
state to decide currentness, and every parallel computation in WORTH runs through
the one execution authority and the structured patterns.
Direct `rayon`, thread spawning and ad hoc pools outside `worth-execution` are
rejected by the boundary check across the repository. A thread that is not
parallel computation (an owner thread, an input/output or durability worker, an
event loop, an async runtime) is declared in the boundary configuration with its
category and reason. Every existing parallel lane migrates onto the authority or
is deleted in this milestone.

Progress stays caller-pumped. There is no background scheduler and no execution
handle. All parallel work runs inside a caller's `advance` or an explicit
execution request, within that request's lease, and every task is joined before
the call returns.

## Current Boundary And Required Change

Parallelism exists today as separate lanes with separate pools and no shared
budget or determinism contract.

- **Signal.** Signal always compiles Rayon; the opt-in `parallel` feature gates
  its use.
  - `logic/planner/precompute/executor_pool.rs` builds one process-global Rayon
    pool sized by `available_parallelism()` and ignores the requested
    `worker_count`.
  - `read_preparation.rs` merges `LocallyOrderedShard`s in order through
    `MergeableOrderedStream`.
  - `apply/stage/concurrent.rs` builds grouped apply packets in parallel on that
    pool and publishes their commits serially in stage task-index order.
  - Precompute admission rejects `FullParallel`. Apply under a `FullParallel`
    executor may still run grouped concurrent packet construction.
  - Inputs are `StageExecutor` and `ParallelExecutionPolicy` (feature-gated),
    plus the graph-derived `ParallelismHint`, which transactions lower to
    `StageExecutor`. `balanced_parallel` and `aggressive_parallel` read
    `available_parallelism()`. The resolved runtime policy carries
    `parallel_min_tasks` and `full_parallel_min_tasks`, and Foundational's
    `ExecutionObjectiveProfile` feeds parallel admission.
  - No manifest enables `parallel`, so none of this runs in application
    execution.
  - The subscriber index has exact `Unscoped`, `WholePartition` and `Detail`
    lanes (Milestone 13) and no deeper hierarchy. `ProducerAspectKey` is private
    and built only from authoritative edges or committed output.
- **Relational.** Rayon is always compiled, gated at runtime by
  `RelationalExecutionModel` (default `SingleLaneExecution`; only
  `ParallelPreparation` enables Rayon). Query fragment and plan execution in
  `crates/worth-relational/src/visibility/materialization/` bucket packets as
  `packets.len().min(rayon::current_num_threads())`, so bucket identity depends
  on the machine. Merges are deterministic: query fragments through
  `reduce_query_fragments`, commit preparation and bulk create through
  `OrderedReductionStream`. `PlanningContract` carries safety booleans such as
  `deterministic_merge_required`.
- **Query derived views.** `reconstruct_parallel/worker_pool.rs` spawns up to
  eight scoped threads on every call and reorders results through a `BTreeMap`.
- **Query frontier planning.** `ParallelAdmissionRoute` is publicly exported from
  a testing module. Its only executor,
  `execution/preflight.rs::execute_parallel_admission_route`, is `#[cfg(test)]`
  and runs the bundle serially.
- **Server.** `product_adapter/execution_pipeline/read_batch_execution.rs` spawns
  one scoped thread per shared-read slot. Semantic routes run on Tokio's blocking
  pool through `spawn_blocking`.
- **Query managed computations.**
  `ApplicationComputationExecution::DeterministicPartitioned` and the
  `Partition` marker are declared and digested into the program revision
  identity, but nothing dispatches on them. The application's handler drives
  `WorthQueryManagedComputationOwner::{prepare, compute, complete}` itself on
  its own thread, outside any `advance`. `compute` is the natural parallel seam:
  the owner is `Send + Sync`, and `Prepared` and `Computed` are `Send`.
- **Query workflows.** `parallel_progression.rs` obtains frontier evidence from a
  host-installed parallel-admission provider, then `execute_parallel_stages`
  runs the stages in a serial loop over `&mut WorthQueryWorkspace`.
- **Application outputs.** `advance_output_demand` progresses one demand per call,
  one producer at a time, so an application calls `advance` repeatedly until its
  outputs settle. Producer `demand_resources` is a self-declared allowance. The
  only measured application-authored work is charged through
  `WorthQueryManagedComputationCheckpoint::advance`.
- **Invalidation of application outputs.** Relational seals field-exact changes
  with every commit. Bridge's direct correspondence path delivers them only for
  an installed correspondence with Signal targets, matched against that
  correspondence's dependency, and carries no index-membership records. Query
  live queries and conditional operations consume those deliveries. Application
  outputs never read them. Instead:
  - reuse re-runs the source query and hashes its whole footprint
    (`identity_current`), so any revision change in the footprint forces a fresh
    computation, even when the value the producer receives is unchanged;
  - demand re-verifies every stored fact of every settlement in the demanded
    closure (`application_output_demand/currentness.rs`), in demand progression,
    in reuse selection and in `require_current_program_output`, so a ready
    output is re-checked on every advancement;
  - the cost of confirming currentness therefore grows with the model, not with
    the edit.
- **Stamping.** Field revisions are value-compared on entity and relation
  update. Authoritative relation patches, however, mark every patched aspect
  changed (`patch_authority.rs`), so an equal-value patch still bumps the aspect
  version, and a reader of aspect-revision facts sees a change that did not
  happen.
- **Published rules.** Query 9.17.6 kept touched sets as a delivery filter and
  made demand-time version comparison authoritative. Its plan states that
  "evidence staleness is never an eagerly stored flag", that "an ordinary source
  commit performs zero workflow invalidation work", and that a touched set "may
  narrow delivery" but "cannot replace admission's authoritative currentness
  proof". Its Phase 2.4 retains outputs by comparing native versions at demand
  time, and its warm-progression cost row allows every checked dependency on
  every warm step. `docs/how-it-works.md` §10 and `docs/glossary.md` define
  touched records only as commit evidence, and §9.8 documents
  `advance(&fresh_request)` only as returning Pending or Settled for one demand.
  Signal
  Milestone 11 named its loosest observation tier `Touched`, and it fires for
  every node invalidation visits, even one that neither recomputes nor
  changes.
- **Thread affinity.** Native artifacts are thread-bound
  (`WorthQueryArtifactThreadBound`, denial `ForeignThread`), checked at run time.
- **WASM.** By default the runtime runs in one dedicated Web Worker, with an
  explicit main-thread compatibility mode. It has no helper workers.
  `worth-signal-wasm` owns worker placement and fallback.
- **Other threads.** Store, Bank and Worth UI create owner,
  durability, watcher, event-loop, stream-reader and async-runtime threads. None
  of them is parallel computation, and none is declared anywhere.
- **Plans.** Signal Milestones 14 to 17 planned an authority, determinism
  contracts, graph parallelism, structured patterns and backends inside
  `worth-signal`. None of their types exist. They are canceled and replaced by
  this plan, because Relational, Query and Server need the same authority, and
  Relational cannot depend on Signal. The existing test
  `crates/worth-signal/tests/milestone_17_compile_time.rs` belongs to an older
  numbering and is unrelated.

## Adversarial Courtroom

### Exact invalidation

- A neutral model repeated at 1, 10 and 100 independent copies, with a fixed
  one-field edit in one copy.
- A commit that races an output's settlement.
- A demand at an older snapshot than the latest marks.
- A commit by a writer other than Query, and a branch merge.
- A created entity, and a changed index key, that match a stored selection,
  absence or set-completeness fact, both through an index and through an
  un-indexed predicate.
- Deletes and slot reuse with a new generation; relation add and remove;
  aspect-version bumps without field changes; declared widening.
- An upstream that republishes an equal value, and a write that sets a field to
  its current value.
- A change to a fetched field that the producer's input value omits.
- A program installation, an aspect contract-revision change, a delivery
  overflow, a checkpoint restore, a reopen from outside the retained commit
  window, and a branch switch.
- Workflow definition and capacity facts changing under waiting work.

Required:

- For the fixed edit, marking work, verification work, producer contacts and
  reverse-index bytes per consumed fact are equal at 1, 10 and 100 copies.
- Demanding a clean output costs zero fact checks and zero source-query re-runs.
- One `advance` with an adequate budget settles every marked output, and
  exhaustion returns a typed outcome naming the remaining work; the application
  never polls.
- A downstream output demanded before its upstream recomputes reports Pending,
  and an equal upstream republication clears the marks below it with zero
  producer contacts.
- The racing commit is caught by insertion replay, and the older-snapshot demand
  ignores later marks.
- Every listed discontinuity triggers the full-verification fallback once per
  affected output, counted and reported.
- A randomized differential test over all of the above finds no difference
  between marking and full verification.

### Same answer at every worker count

- A map over 10^7 items with heavily skewed per-item cost.
- Floating-point sums and dot products with adversarial magnitudes, where any
  change of association changes the bits, including signed zeros and NaN
  payloads.
- A prefix scan with partition boundaries at hostile positions.
- A bulk-synchronous iteration that takes many rounds to converge.
- A wide Signal antichain with long critical chains, reconvergence, and nodes
  that propose dependency rewires while the batch runs.
- Async-capable, temporal, previous-value and on-demand nodes inside a wide
  batch.
- Kernel failures in several partitions at once, and a kernel panic.
- The same declarations with one worker, two, the machine's width and a width
  larger than the machine, under forced random schedules, each beside a serial
  oracle.

Required:

- Every completed result equals the serial oracle bit for bit under
  `CanonicalBitwise`.
- Charged work is identical at every worker count for every completed outcome.
- Partition identity and reduction shape never depend on worker count, worker
  index or completion order.
- Graph publication, replay, history and explanation are identical to serial.
- A failure reports the same partition and the same outcome as the serial run.

### Isolation and reuse

- A 10,000-partition computation where one partition's consumed fact changes.
- An edit that inserts a partition, one that deletes a partition, and one that
  moves an item between partitions.
- An edit that merges two islands, one that splits an island, and one that
  removes an island's least member.
- A result that changes from `0.0` to `-0.0`.
- A commit racing a partition's settlement.
- A leaf change deep in a scope hierarchy with 10^5 sibling subtrees.
- Branch switch, merge, checkpoint restore and undo across partitioned outputs.

Required:

- Exactly the affected partitions recompute, and `prepare` re-gathers only
  their items.
- The combine recomputes only the reduction-tree nodes on the affected root
  paths.
- Unaffected islands keep their identity and results.
- A changed encoding always propagates; an identical encoding always stops.
- Sibling-disjoint scope subtrees contribute zero candidate and zero ready work.
- A branch switch reuses the retained nodes both branches share.
- Every fallback to full recomputation is counted and reported with its cause.

### Decomposition with an interface

- A neutral sparse linear system split into subdomains. Interior work runs per
  subdomain, the interface system is assembled by canonical reduction and solved
  once, and back-substitution runs per subdomain.
- A subdomain edit that changes its condensed interface contribution, and one
  that does not.

Required:

- The decomposed result equals the serial oracle of the same decomposition bit
  for bit at every worker count.
- It is `ContractEquivalent`, under a test-local tolerance contract, to the
  undecomposed solve.
- When a subdomain's condensed contribution is unchanged in encoding, the
  interface solve is reused.
- A changed interface solution reruns back-substitution for every subdomain
  whose interface slice changed, charged and counted.

### Scale and exhaustion

- Wide independent computations in one `advance`, each with nested partition
  work, while Signal runs a wide antichain and Relational runs parallel query
  fragments under the same request lease.
- Empty, singleton, tiny and memory-saturating inputs.
- Cancellation and deadline expiry inside map, reduction, scan, round, interface
  and graph-epoch stages.
- A runtime called without a lease.
- A platform exposing only serial execution (wasm32).

Required:

- Active workers across every layer never exceed the process authority, and
  nested work never exceeds its parent lease.
- Cancellation and deadline report a canonical-prefix boundary, before which
  every partition completed, and the disposition. The boundary may differ by
  worker count; every completed partition's result equals the serial oracle.
- Ready queues, prepared packets and unpublished results stay bounded by the
  lease.
- A runtime called without a lease creates no thread.
- The serial platform executes the same declarations with the same results and
  reports its serial posture.

### The courtroom must convict

- a clean output whose facts are re-checked or whose source query is re-run on
  demand;
- a commit that scans settlements or waiting work;
- a second delivery path feeding Query invalidation;
- a mark honored beyond the demand's snapshot, or a mark missed by a racing
  settlement;
- a write that bumps the revision of an equal value;
- reuse keyed on the source query's footprint instead of the producer's input
  value;
- an application that polls `advance` to settle its outputs;
- a user-authored "parallel safe" assertion or a forged disjointness proof;
- partition identity or reduction shape derived from a worker index, bucket
  count, rank position or completion order;
- a thread pool, `rayon` call or undeclared thread outside `worth-execution` in
  production code;
- a runtime that dispatches work without the caller's lease;
- a floating-point reduction whose bits change with worker count;
- a scan implemented as an unordered reduction;
- iteration with no declared convergence, round limit or exhaustion outcome;
- a partition write that aliases another partition's write set, or a read that
  crosses a concurrent write outside the pattern's synchronization boundary;
- a serial fallback that changes partition or reduction semantics;
- serial work reported under a parallel posture;
- a partition key, scope path or shard used as an invalidation cause or a
  currentness proof;
- transitive copying of aspects or scope paths to dependents;
- locks or atomics offered as independence evidence;
- publication in completion order;
- graph work admitted concurrently on topological level alone;
- a rewire proposal that changes another admitted node's legal inputs within the
  same epoch;
- candidate lookup that skips a covering ancestor or drops an unscoped
  subscriber;
- output cutoff decided by `PartialEq` instead of canonical encoding;
- a `compute` step whose result changes with schedule through interior
  mutability;
- a native artifact reached from a partition kernel or a wave `compute`.

## Product Decision Lock

### The touched graph

The touched graph is the exact set of changes a commit made. Relational seals
field-exact changes in the published patch envelope: changed records, aspect
field paths, adjacency changes (relation kind and endpoints) and aspect-version
bumps. Other envelope producers may declare a coarser change. Every commit
contributes a touched graph, whichever writer made it. It has two roles, and
neither may be dropped:

- **Evidence.** It is proof of what the commit changed, used for receipts, undo
  and audit.
- **Cause.** It is the only thing that invalidates. Declared dependencies say
  what may affect an output. The touched graph says what did. The runtime marks
  exactly the consumers whose consumed facts intersect the touched graph, at the
  smallest granule that holds those facts. A consumer it does not reach, and
  whose upstream outputs are current, is current without re-checking.

Every layer carries the full precision the layer below sealed. Query consumes
Relational changes at field precision for application outputs, not only for live
queries. A coarser change is lawful only when its producer declares the widening
(`DeclaredWidening`, a whole-aspect write, or an empty scope). Such a change marks
at its declared breadth and is counted and reported.

### Touched records Query consumes

- **Observable revisions.** Relational emits a touched record for every revision
  bump a stored fact can observe:
  - a field value;
  - an aspect version;
  - an adjacency structural revision;
  - index membership.

  Index membership changes carry the old and new index keys, derived by
  Relational from the field changes and the installed index definitions, so a
  phantom match is caught.
- **Delivery.** Marking consumes every committed patch envelope on the branch
  through one Query-owned Bridge subscription with no Signal targets.
  - It carries the full published aspect change with its field path, the
    index-membership records, and, from Phase 4, the scope path at the depth
    Signal sealed it.
  - Delivery runs in commit order and synchronously with commit visibility.
  - Live queries, conditional operations and workflow coverage consume the same
    subscription, and their correspondence-based matching is replaced by
    marking. Signal correspondences keep delivering to Signal-hosted nodes and
    never feed Query marking. No second path feeds Query invalidation.

### Marking

- **Settle-time index.** When an output settles, each consumed fact enters a
  reverse index:
  - entity presence and generation;
  - field path;
  - aspect-revision facts, keyed by entity and aspect, which match any change
    in that aspect;
  - adjacency anchor, relation kind and direction;
  - index key, for selection, absence and set-completeness facts answered
    through an index;
  - entity kind and predicate field paths, for those facts answered without an
    index.

  Before the output is marked current, insertion replays the touched records of
  commits after its read basis, so a commit that races the settlement is never
  missed.
- **Commit-time marking.** Each commit's touched records are matched against the
  index. Exactly the matching settlements are marked dirty, with the matched
  facts and the commit version.
  - The cost is proportional to the touched records, their matches and the
    marked downstream closure.
  - No commit scans settlements or waiting work.
- **Upstream propagation.** Marking a settlement also marks, transitively, every
  settlement that consumed its output as pending-upstream.
  - A pending-upstream output is not current, and demand for it reports
    Pending.
  - When an upstream republishes an equal value, the pending marks below it
    clear without any producer contact.
- **Selection facts.** A created entity, or a changed index key, that matches a
  stored selection, absence or set-completeness fact marks that fact. For a fact
  answered without an index, a created entity of that kind, or a change to any
  predicate field path on that kind, marks it.
- **Lineage.** Deletes and slot reuse with a new generation mark every fact about
  the old entity. Marks are per branch lineage, so an output current on one
  branch is not thereby current on another. A merge commit's touched graph is its
  difference from the target branch's parent, and the target lineage stays
  continuous.
- **Index cost.**
  - Index entries reference the settlement and a fact ordinal, so fact storage
    is shared rather than copied.
  - The index is charged as derived retained bytes.
  - Entries are removed when their settlement is superseded or reclaimed.

### Currentness, demand and advancement

- **Continuous basis.** A continuous basis requires all of the following:
  - the same branch lineage;
  - the settlement's read basis still inside Relational's retained commit
    window;
  - the demand's snapshot at or after that basis.

  A demand at snapshot `v` honors marks up to `v`.
- **Clean reuse.** On a continuous basis, a settlement that is neither dirty nor
  pending-upstream is current. It is reused without re-running its source query,
  hashing a footprint or scanning facts.
- **Dirty recompute.** A dirty settlement re-verifies only its marked facts. If
  they changed, it recomputes its output key, subject to input cutoff. A
  partitioned computation recomputes only its marked partitions.
- **Required set.** The outputs still required are those with an open demand, or
  those required by a performed operation (`start_required_outputs`). The set
  lives in Query's demand registry, and closing a demand removes its outputs.
- **Advancement.** One host `advance(&fresh_request)` progresses exactly the
  dirty and pending-upstream members of the required set, in dependency-ready
  waves, to completion within its budget. It then reports readiness. When the
  budget is exhausted first, it returns a typed outcome naming the remaining
  work.
  - The authenticated request stays the principal.
  - No background sweeper is added.
  - The application neither re-demands its whole output tree nor polls.
- **Full-verification fallback.** Full verification by revision comparison
  remains only when the basis is not continuous, or when marking may be
  incomplete:
  - checkpoint restore and reopen;
  - a read basis outside the retained commit window;
  - a switch to another branch lineage;
  - a foreign basis;
  - program or schema installation;
  - an aspect contract-revision change, or an index-definition change;
  - a delivery gap: an overflowed or missed batch, or marks that lag the
    demand's snapshot.

  It runs once per affected output and is counted and reported.
- **Commit-time checks.** A commit still recompares its attempt's read facts
  (optimistic concurrency control). Marking replaces only demand-time
  re-verification.
- **Equivalence check.** A debug and certification mode runs full verification
  beside marking and fails on any difference.
- **One mechanism.** Workflow definition and capacity facts mark through the same
  path. A workflow history basis is pinned to an immutable snapshot and is never
  marked. One invalidation mechanism serves outputs, live queries and workflow
  coverage, and no parallel lane remains.
- **Charging.** Work budgets charge marking and dirty verification, never a scan
  of clean state.

### Cutoff on both sides

Input cutoff and output cutoff are one mechanism.

- **Reuse key.** An output's reuse key has three parts:
  - the source query's selection facts, meaning which entities it chose;
  - the canonical digest of the input value the producer receives;
  - the computation's implementation edition. Today that is its declared
    identity revision; an edition derived from code fills the same field without
    changing the key.
- **Input cutoff.** Rebuilding the input value is cheap. If its digest is
  unchanged, the output is reused without contacting the producer. A field that
  enters the input but cannot affect the output is a defect, and the courtroom
  exposes it as an unexpected contact.
- **Stamping.** Every Relational write path compares values before stamping,
  and an aspect version bumps only when one of its fields changed. An equal value
  keeps its revisions, whichever patch kind wrote it.
- **Republication.** A republished output keeps the entity identity of its output
  key, whatever its value. An equal canonical encoding keeps its revision.
  Equality is byte equality of the canonical encoding, never `PartialEq`, so
  `-0.0` and `0.0` differ. The same rule decides partition and reduction-tree
  cutoff.
- **Determinism.** Cutoff and reuse depend on deterministic producers. The
  serial oracle and schedule perturbation certify them.

### The public definition is restored

- **Definition.** `docs/glossary.md` and `docs/how-it-works.md` state the
  definition in [The touched graph](#the-touched-graph). How-it-works §9.8 and
  §10 describe marking and upstream propagation, clean reuse and dirty
  recompute, one-call advancement of the required set and the full-verification
  fallback. `docs/coding-guidelines/perf_laws.md` states that the touched graph
  is the semantic delta that bounds recompute.
- **9.17.6 rules replaced.** The 9.17.6 rules quoted in Current Boundary are
  replaced by this milestone's marking and currentness rules. The 9.17.6 plan
  points here.
- **9.17.6 rules kept:**
  - no commit scans waiting work (the reverse index serves instead);
  - negative and set-completeness dependencies;
  - ABA denial;
  - reuse as a checked equivalence, never an identity or value match.
- **Progression.** Caller-pumped progression is kept: the host's authenticated
  `advance` stays the principal. One call now completes the dirty required set.
- **Signal observation tier.** The `Touched` tier and every identifier bound to
  it are renamed to `Visited`: its policy constructors, the `touched` field on
  committed and classified observation summaries,
  `ObservationScratchSummary::touched_event_count`, and the tier's `touched()`
  accessors. "Touched" then means only what a commit changed.
- **Granules.** The glossary states that a shard is a placement unit, and that a
  scope path is a precision granule of touched records, never a cause on its
  own.

### The execution authority and leases

`worth-execution` is a runtime crate. It depends only on `worth-foundational` and
`worth-proof`, so Relational, Signal, Query and Server can all depend on it. It
owns:

- the physical worker pool, created once when the authority is constructed;
- `ExecutionAuthority`, which grants `ExecutionResourceLease`s;
- platform capability resolution;
- the backend port with its backends;
- the pattern executors, the partitioners, the canonical reduction tree and the
  serial oracle.

`ExecutionAuthority` is constructed only by a composition root: a Server host,
or a host that composes a World or runtimes directly. A process constructs at
most one authority; a Server host injects its authority into the World it
builds, and a second construction in one process is denied. Leases, sealed batches
and reduction plans are never publicly constructible.

Callers state one execution request policy: posture, determinism, `max_workers`,
memory, deadline and cancellation. Query requests, Relational requests, Signal
evaluation and Server requests carry it. No other spelling of parallel intent
exists.

Leases:

- A lease caps concurrently active workers and charged memory, and carries
  deadline and cancellation.
- Children draw worker slots dynamically from the parent's cap; there is no
  static split. A child that finds no free slot runs inline on the requesting
  worker, so waiting never idles a leased slot and nesting cannot deadlock.
- Lease memory is charged bytes (declared sizes, the same accounting as retained
  bytes), not allocator measurement.
- Every entry point that may dispatch work takes the caller's lease: Relational
  read, commit and index requests; Signal evaluation; Bridge correspondence;
  Server request execution. Bridge carries the lease through and never acquires
  one.
- A runtime called without a lease runs on the serial backend and creates no
  thread. Only the process's one authority owns a native pool.

Placement follows the repository placement rule:

- **`worth-proof`** gains generic checked doors over opaque ordered keys:
  disjoint key-set families and canonical order over identities, extending
  `CanonicalVec` and the existing disjointness doors. It also gains an execution
  authority marker. It learns nothing about graphs or determinism.
- **`worth-foundational::execution`** holds portable vocabulary that means the
  same across runtimes: `DeterminismContract`, `ExecutionPosture`,
  `ExecutionBudget`, stable partition identity and the report values. Reports
  describe work and authorize nothing.
- **`worth-execution`** owns `DisjointPartitionBatch` and
  `DeterministicReductionPlan`, each carrying a `Proof` from those doors, and
  everything with a pool, a counter, a live table or `Drop`.
- **`worth-signal`** owns `DisjointGraphBatch` and fills the door from its own
  footprints. Footprint completeness is Signal's obligation.

### Independence is declared, safety is minted

Callers declare posture, budget, deadline, cancellation and determinism.
Computations declare partitioning, read and write sets, reducers and convergence.
Before dispatch the framework validates coverage, uniqueness and write-set
disjointness, and that the declared determinism contract is satisfiable by the
declared reducer (an identity element, and an equivalence contract where one is
required). No algebraic law is inferred or checked; the oracle evaluates the
canonical tree. Overlapping write sets are denied before dispatch, naming the
pair. A read set may intersect a concurrently admitted write set only through the
pattern's synchronization boundary: the prior round image, the scan carry, or the
interface stage.

### Determinism is a contract family

```rust
pub enum DeterminismContract {
    CanonicalBitwise,
    ContractEquivalent(EquivalenceContractId),
}
```

- `CanonicalBitwise` is the default. It fixes partition order, reduction-tree
  shape and publication order wherever the result depends on order.
- `ContractEquivalent` names an installed, identity-bearing equivalence
  predicate, digested into revision identity. Certification uses it in place of
  bitwise comparison.

Throughput-relaxed determinism is deleted rather than deferred: it breaks replay
and reconstruction identity, and nothing needs it. Floating-point associativity is
never inferred. No backend weakens a resolved contract.

### Compute is local, publication is canonical

Workers read immutable inputs and return worker-local results or effect
proposals. Authoritative state changes only through the existing transaction and
apply authorities, in canonical order. Physically parallel writes are legal only
when storage ownership is disjoint and one commit manifest makes their visibility
atomic. Otherwise publication is serial, and the parallel compute claim is
unchanged.

### Cancellation, failure and panics

- Cancellation and deadline are observed only at deterministic safe points:
  kernel checkpoints and wave, round and epoch boundaries.
- The call joins every in-flight task before returning.
- The outcome reports a canonical-prefix boundary, before which every partition
  completed, and the disposition: no work, worker-local work only, prepared
  publication, or committed. The boundary may differ by worker count, because
  more workers finish more work before a deadline fires; every completed result
  equals the serial oracle. Discarded
  in-flight work is reported separately as physical, uncharged work.
- When several partitions fail, the reported failure is the one with the least
  partition identity, and every partition ordered before it completes. Serial and
  parallel runs therefore report the same failure.
- The work ceiling follows the same rule. Each partition charges a checkpoint
  local to it, capped at the computation's remaining ceiling. After the join,
  exhaustion is decided by canonical prefix sums of partition work, and work
  past the first exceeding partition is discarded as physical, uncharged work.
- A kernel panic becomes a typed failure. It never poisons the pool or leaks a
  lease.

### Structured patterns are the only public surface

- `map`: independent application over stable partitions.
- `reduce`: a canonical reduction over the canonical tree, with an explicit
  identity element.
- `scan`: an ordered prefix composition with carry. A changed partition
  recomputes every later carry; scan makes no sublinear reuse claim.
- `fork_join`: recursive independent subcomputations.
- `rounds`: bulk-synchronous iteration inside one computation. Each round reads
  one committed round image. Convergence, maximum rounds and the non-convergence
  outcome are declared. Iteration across commits stays with Query convergence
  epochs.
- `decompose`: the interface pattern, in four stages:
  1. a map over partitions produces interior results and condensed interface
     contributions;
  2. a canonical reduction assembles the interface problem;
  3. one interface computation solves it, and may itself decompose;
  4. a map over the partitions back-substitutes.

  The platform owns the stage order, determinism and per-stage reuse. The domain
  owns the matrices and solvers.

Raw spawn, raw threads, worker indices, backend queues and shared mutable
callbacks are not public computation APIs. Asynchronous fixed-point iteration is
deleted rather than deferred.

### Partitions and partitioners

A partition has a stable identity derived from data, with explicit read and write
sets and declared boundary (halo) reads.

A partitioned computation's input items carry their source fact identity. The
partitioner records which item goes to which partition. A partition's consumed
facts are the facts of its routed items plus its declared boundary reads. On a
commit, `prepare` re-gathers only the items of marked partitions, and the
partitioner re-routes only touched items.

The platform provides three domain-neutral partitioners, and a domain may supply
its own:

- **Keyed.** Groups items by a declared key.
- **Components.** Finds the connected components of a declared coupling relation
  with a deterministic union-find. An island's identity is its least member
  identity, so an edit elsewhere never renames it. A merge is incremental. A
  split, or removal of an island's least member, recomputes the affected
  island's components at a cost proportional to its members and edges. Only the
  islands involved change identity.
- **Bisection.** Recursively bisects a declared weighted graph with deterministic
  tie-breaking. A partition's identity is its cut path. Each cut yields two
  partitions and an interface set for `decompose`. Only a subtree whose imbalance
  exceeds the declared tolerance is re-cut, and re-cuts are counted. Cut quality
  is reported, not promised.

A partitioner is itself a charged computation. Its output is retained and
maintained incrementally, so an edit that leaves the coupling relation unchanged
does no re-partitioning.

A partition may bind to a scope-path subtree for locality. The binding improves
placement and nothing else. Read and write validation stays authoritative.

### The canonical reduction tree

Every `reduce`, every retained tree and the serial oracle evaluate one shape: a
Cartesian tree (treap) in-order by partition identity, with priority equal to a
fixed digest of the identity and ties broken by identity order. The shape is a
pure function of the set of identities, independent of edit history and worker
count.

- Changing, inserting or deleting one partition recombines the nodes on the
  affected root paths: expected O(log P) for P partitions, with the actual count
  charged and reported.
- The incremental result is bit-identical to a full recomputation, because a
  full recomputation builds the same tree.
- A partition identity change (an island merge or split, or removal of its least
  member) is a deletion plus an insertion.

Output cutoff compares the canonical encoding of a result byte for byte, never
`PartialEq`. The same rule applies to interior tree nodes and to `decompose`
interface contributions.

### Relational

Every Rayon use in Relational runs on the caller's lease:

- query fragment and plan execution;
- index builds;
- the validation engine;
- bulk create;
- commit preparation.

Buckets derive from stable partition identity, never `current_num_threads()`.
Merges keep `reduce_query_fragments` and `OrderedReductionStream`, now over the
canonical order. `RelationalExecutionModel` and the safety booleans of
`PlanningContract` are deleted; Relational takes `ExecutionPosture` from the
request policy, and the affected digests are re-baselined.

### Signal: graph parallelism

The authority chain runs in one direction: causal admission, ready work,
readiness and control-order proof, footprint, conflict groups, worker-local
proposals, reconciliation, epoch publication. Reports and diagnostics derive from
this chain and admit nothing.

Ready graph work is admitted concurrently only with three proofs:

1. **Dependency readiness.** Every producer version the batch reads is settled.
2. **Control-order safety.** No earlier operation can reveal a dependency that
   changes the batch's legal inputs.
3. **Mutation disjointness.** Worker-local effects do not overlap.

Topological level alone proves none of these in a rewiring graph.

- **Footprints fail closed.** Every graph, subscription, snapshot, lineage,
  observation and canonical diagnostic surface belongs to the footprint. A
  surface missing from a footprint fails admission; it never defaults to narrow.
- **Conflict groups.** The planner lowers ready work into ordered conflict groups
  carrying a sealed `DisjointGraphBatch`. The executor consumes them and never
  recomputes footprints.
- **Rewiring epochs.** A worker may propose dependency changes from its snapshot.
  A proposal is non-authoritative until reconciliation validates legality, cycle
  safety, subscription changes and affected readiness. Work whose legality a
  proposed rewire could change is not admitted in the same epoch.
- **Publication.** Worker-local evaluation, comparison, patch construction and
  snapshot construction run concurrently. Publication is one canonical epoch
  commit.
- **Backpressure.** Ready queues, prepared packets, rewiring proposals and
  unpublished results are bounded by the lease. Exhaustion reduces admitted
  concurrency or rejects before dispatch.
- **Node kinds.** Async-capable, temporal, previous-value and on-demand nodes run
  as singleton conflict groups, ordered canonically.
- **Stays serial.** Parallel plan validation, condition preview and eligibility
  stay serial. This milestone makes no parallel claim for them.
- **Causality stays put.** Milestone 13's causality owner still admits each
  dependency edge, and the Proof-enforced invalidation progression and
  current-basis binding are unchanged. Parallel execution may split admitted
  edges, canonicalize duplicate causes and schedule ready items. It never
  interprets aspects, partitions or conditions itself.
- **Precompute.** It admits `FullParallel` when the lowered batch carries the
  three proofs.

The existing inputs are replaced, not kept beside the new model:

- `ParallelismHint::Serial | Preferred` becomes `ExecutionPosture::Serial |
  Automatic`.
- The worker count becomes the lease's `max_workers`.
- `parallel_min_tasks`, `full_parallel_min_tasks`, chunk size and group width
  become resolved planner policy, shown in the plan. `ExecutionObjectiveProfile`
  lowers into the same policy.
- Signal's `StageExecutor` and `ParallelExecutionPolicy` are deleted, together
  with the `*_with_executor` entry points. Diagnostic `executor` fields are
  replaced by the execution report.

### Signal: hierarchical locality

Milestone 13's `WholePartition` and `Detail` lanes become the two-segment case of
a canonical `ScopePath` bounded at eight segments. Segments are opaque
host-defined identity. The runtime understands only four relationships: exact,
ancestor, descendant subtree and disjoint.

The reverse-subscription owner builds its private `ProducerAspectKey` only from
authoritative dependency edges or committed producer output identity, and indexes
unscoped subscribers and hierarchical scope buckets. Lookup runs as a `map` over
index partitions. Each worker returns non-authoritative candidates, and the
causal owner validates every immediate edge. The merge orders by cause and work
identity.

- An exact-leaf change queries exact and ancestor-covering subscribers.
- A subtree change queries lawful descendants and covering ancestors.
- Unscoped subscribers are always included.
- Sibling-disjoint subtrees contribute nothing.
- The Milestone 13 partition and detail certification passes unchanged as the
  two-segment case, and hierarchical candidates at depths 1, 2, 4 and 8 equal an
  independent serial candidate oracle.

`ScopePath` and `ProducerAspectKey` stay Signal graph-runtime vocabulary.

### Runtime Bridge

Bridge carries the caller's lease into Relational and Signal and never acquires
one. It delivers the scope path at the depth Signal sealed it, alongside the field
path it already carries on the direct correspondence path. Query's marking
consumes whatever depth arrives, so deeper Signal scopes open no precision gap
above Signal. Bridge adds no scheduling.

### Query: partitioned computations and per-partition reuse

`DeterministicPartitioned` executes through a partitioned owner binding (see
Public Developer Experience). The declaration changes with it:

- `ApplicationComputationPartition` becomes the partition key contract:
  `Ord`, a canonical encoding, `Send + Sync`, and its identity.
- Order is partition identity order. `ORDERING` is deleted from
  `ApplicationManagedComputation`, and a `Deterministic` computation declares the
  platform's single-partition key.
- `DeterministicPartitioned` implies per-partition reuse. `Reuse` keeps its
  meaning: disposable warm-start evidence.
- The declaration names its `DeterminismContract`, defaulting to
  `CanonicalBitwise`.
- These changes alter program revision identity, and existing digests are
  re-baselined in Phase 6.

Each partition's result settles with its own consumed facts, keyed by
computation identity, implementation edition, partition identity and read basis.
The reverse index holds those facts at partition granularity, so a touched fact
marks exactly the partitions that read it.

Retention belongs to Query. `worth-execution` provides the canonical tree
algorithm over a caller-owned node store. Query owns retention, eviction, branch
sharing and retained-byte charging. Nodes are keyed by their children's
identities, so a branch switch reuses the nodes both branches share. Exceeding
the computation's `maximum_retained_bytes` evicts to full recomputation for that
computation.

A partition whose recomputed encoding equals its previous encoding stops
propagation, so nothing above it recombines.

Full recomputation is the fallback in these cases, each counted and reported with
its cause:

- a basis that is not continuous (see Currentness, demand and advancement);
- a partitioner output change beyond the declared bound;
- eviction under the retained-byte ceiling.

An edition change is a reuse-key miss, not a fallback.

### Query: parallel advancement

The one-call advancement above runs its waves in parallel.

- A wave contains the required members whose declared upstream outputs are all
  committed in the current basis. Wave membership comes from dependency
  readiness in the required-set owner, never from a level number.
- `compute` is pure over `Prepared`. All reads happen in `prepare` on the owner
  thread, so no rewire can occur inside a wave.
- Within a wave, the `compute` steps of independent computations run
  concurrently under the request's lease, alongside their nested partition work.
  `prepare` and `complete` stay on the owner thread. Commits and publications
  apply in canonical order.

The authenticated request stays the principal, and no work outlives the call.
Charged work is identical at every worker count.

The workflow parallel frontier runs admitted stage preparation as worker-local
computation on the lease and applies stage effects in canonical order.
Derived-view reconstruction is a `map` with canonical publication.

### Server and World

The World composes one authority per process and hands it to the Relational and
Signal runtimes it builds. Query, which is built on the World, takes the authority
from its World. Server receives the authority from its host composition and draws
each request's lease from it. Server shared-read batches are a `map` over read
slots on the request lease. Semantic routes may still enter through Tokio's
blocking pool, a declared async-runtime thread; any parallel work inside them
runs on the request lease.

### WASM

On wasm32 the authority resolves the serial posture and reports it.
`worth-signal-wasm` keeps its worker placement and fallback authority, which is
independent of `ExecutionPosture`. Its `parallel_executor_usage_count` is
replaced by the execution report.

### Accounting is work and span

Every pattern and every graph epoch reports gated counters, all deterministic:

- **work:** the sum of charged operation units;
- **span:** the charged critical path;
- per-partition work, reductions performed and reused, rounds, epochs, conflict
  groups, boundary reads, retained bytes and fallbacks.

Physical counters are reported and never gated, except that the lease bounds are
asserted: active-worker high-water mark, steals, queue width and discarded
in-flight work.

Cost gates assert work and span on any machine without wall-clock. Span proves
that parallelism is available even on a one-core machine. A planner grain
threshold, derived from declared per-item work estimates, runs small inputs
serially. That changes placement, never meaning or charged work.

### Platform capability and backends

Parallel support is a resolved capability. The authority executes through a
backend port that carries in-process tasks. This milestone delivers three
backends:

- **Serial:** a complete implementation of every pattern and the oracle.
- **Native:** a work-stealing pool.
- **Schedule perturbation:** forced random schedules for certification, which
  also proves that the port is neutral.

Feature flags and targets change which backends are available, never meaning,
charged work or proof topology.

Deferred backends plug into the port: WASM helper workers, remote and distributed
execution, accelerators, and physical shard placement with rebalancing. Portable
execution requires a registered computation identity and edition, which managed
computations already have. Raw `PartitionComputation` closures stay in-process.

### Thread affinity

Partition kernels receive `Send` immutable inputs and return `Send` results.
Native artifacts stay on their owner thread, and reaching one from a partition
kernel or a wave `compute` is denied in every posture, including serial and one
worker. A kernel that needs native artifact data reads it in `prepare` on the
owner thread.

### Existing lanes converge

Each existing lane migrates in this milestone or is deleted. No compatibility
executor and no second pool survives.

| Lane | Change |
| --- | --- |
| Signal `ParallelismHint`, `StageExecutor`, `ParallelExecutionPolicy`, `*_with_executor`, diagnostic `executor` fields | Lowered or deleted as described in Signal: graph parallelism |
| Signal `parallel_min_tasks`, `full_parallel_min_tasks`; Foundational `ExecutionObjectiveProfile` | Resolved planner policy |
| Signal `precompute/executor_pool.rs` | Deleted; precompute and apply run on leases |
| Signal `parallel` feature and machine-width reads | Deleted; capability is resolved at run time |
| Relational Rayon uses | Run on the caller's lease; buckets from stable partition identity |
| Relational `RelationalExecutionModel`, `PlanningContract` safety booleans | Deleted; posture comes from the request policy |
| Query `reconstruct_parallel` worker pool | Deleted; reconstruction is a `map` |
| Query frontier-planning `ParallelAdmissionRoute` and its test-only executor | Deleted rather than migrated; the workflow parallel frontier is the production route |
| Query workflow parallel frontier | Worker-local stage preparation; canonical effect application |
| `DeterministicPartitioned` | Executes through the partitioned owner binding |
| Server per-slot read threads | A `map` over read slots |
| `worth-signal-wasm` `parallel_executor_usage_count` | Replaced by the execution report |

### Neighboring milestones

- **Signal Milestones 14 to 17** are canceled, and their plan files are deleted.
  Signal's roadmap, vision, architecture (S9.17), acceptance map and test
  requirements point here.
- **Query 5.3.** Signal stays authoritative for graph frontier and parallel
  admission. The execution resource authority is `worth-execution`.
- **Query 9.17.6.** Its rules that kept touched records as evidence only are
  replaced here, and its plan points here. Its caller-pumped progression, ABA
  denial, negative dependencies and checked-equivalence reuse are kept.
- **Query 9.20.** Set execution consumes partitions, disjointness and canonical
  reduction from `worth-execution`, and adds domain conflict meaning without a
  partitioning or reduction module of its own.
- **Query 9.22.** Occurrence-safe reuse consumes `DeterminismContract` and
  per-partition reuse, and adds occurrence identity on top.

## Budgets, Outcomes, And Cost Contracts

| Contract | Bound |
| --- | --- |
| Commit-time marking | O(touched records + matched facts + marked downstream closure); no scan of settlements or waiting work |
| Clean demand | Zero fact checks and zero source-query re-runs |
| Fixed edit at 1, 10 and 100 model copies | Equal marking work, verification work, producer contacts and reverse-index bytes per consumed fact |
| Reverse index | Charged as derived retained bytes; entries reclaimed with their settlement |
| Unchanged producer input or output encoding | Zero producer contacts downstream of it |
| Charged work | Identical at every worker count and schedule for completed outcomes |
| Span | Reported per pattern and epoch; asserted against the declared structure |
| Process authority | Active workers across all layers never exceed the authority's width |
| Lease | Nested work draws from its parent's cap and never exceeds it |
| Threads | No worker thread is created after authority construction; none without a lease |
| One inserted, deleted or changed partition | Expected O(log P) reduction nodes; the actual count is charged and reported |
| Unchanged recomputed encoding | Zero reduction nodes recombined above it |
| Island merge or split | Only the islands involved recompute |
| Unchanged coupling relation | Zero re-partitioning work |
| Deep leaf change among disjoint subtrees | Candidate work proportional to the path depth plus matching subscribers |
| Scheduling overhead | O(partitions + reductions + conflict groups), excluding charged partitioner work |
| Retained partition results and tree nodes | Charged as derived retained bytes, owned by Query |
| Fallback to full verification, full recomputation or serial publication | Counted and reported with its cause |
| Serial platform | Same result, same charged work, reported serial posture |

Outcomes are typed:

- completed;
- canceled at a named boundary;
- deadline exceeded at a named boundary;
- resource exhausted;
- not converged;
- failed, with a typed kernel denial or a panic, naming the least failing
  partition;
- denied before dispatch, naming the violated rule.

## Public Developer Experience

A domain partitions a managed computation by declaring a partition key, a
partitioner, a per-partition kernel and a reducer. It writes no threading code.

```rust
impl ApplicationManagedComputation<Ledger, Balances> for RegionBalances {
    type Input = PostedEntriesInput; // Value = PostedEntries
    type Output = RegionBalanceSheet;
    type Partition = RegionKey; // Ord, canonical encoding, Send + Sync
    type Reuse = NoWarmStart;
    type Stopped = BalanceStopped;
    const IDENTITY: &'static str = "ledger.region-balances";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
    const DETERMINISM: DeterminismContract = DeterminismContract::CanonicalBitwise;
    const RESOURCES: ApplicationComputationResourceCeiling =
        ApplicationComputationResourceCeiling::new(1 << 20, 1 << 24);
}

impl WorthQueryPartitionedComputationOwner<Ledger, Balances, RegionBalances>
    for RegionBalancesOwner
{
    type PartitionResult = RegionTotals;
    type Output = RegionBalanceSheet;
    type Stopped = BalanceDenial;

    fn partitions(&self, entries: &PostedEntries) -> PartitionPlan<RegionKey> {
        PartitionPlan::keyed(entries.items(), |entry| entry.region())
    }

    fn compute_partition(
        &self,
        partition: PartitionView<'_, RegionKey, PostedEntries>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<RegionTotals, WorthQueryManagedComputationDenial<BalanceDenial>> {
        RegionTotals::sum(partition.items(), checkpoint)
    }

    fn reducer(&self) -> DeterministicReducer<RegionTotals> {
        DeterministicReducer::canonical(RegionTotals::zero, RegionTotals::combine)
    }

    fn complete(&self, totals: RegionTotals) -> Result<RegionBalanceSheet, BalanceDenial> {
        RegionBalanceSheet::validated(totals)
    }
}
```

The framework runs `partitions` and gathering in `prepare` on the owner thread,
`compute_partition` on the lease, the reducer over the canonical tree, and
`complete` on the owner thread.

Code outside Query uses the same patterns directly:

```rust
let totals = PartitionComputation::over(&records)
    .partition_by(Partitioner::components(&coupling))
    .map_partition(|partition, cx| kernel(partition, cx))
    .reduce(DeterministicReducer::canonical(Totals::zero, Totals::combine))
    .run(&lease)?;
```

Rules for authors and agents:

- Declare independence. Never spawn threads or import Rayon.
- Match the partitioner to the data: keyed for grouping, components for coupled
  systems, bisection for one large connected system.
- Use `decompose` when partitions interact only through an interface.
- Read work and span from the report. Never gate on elapsed time.

The final names may differ, but each concept must exist, and no public API may
name a thread, pool or backend.

## Destination Topology And Enforcement

Legend: E = existing owner; N = new; R = extended or replaced; D = deleted.

```text
crates/worth-execution/                                  N  execution runtime
  src/authority/                                         N  pool, leases, capability, cancellation
  src/backend/{port,serial,native,perturbation}.rs       N
  src/pattern/{map,reduce,scan,fork_join,rounds,decompose}.rs  N
  src/partition/{identity,access,keyed,components,bisection}.rs N
  src/reduction/canonical_tree.rs                        N  treap shape over a caller-owned node store
  src/report/                                            N  work, span, partition and epoch reports
  src/oracle/                                            N  serial oracle
crates/worth-foundational/src/execution/                 N  portable vocabulary and reports
crates/worth-proof/src/                                  R  generic disjoint-family and canonical-order doors
crates/worth-relational/src/                             R  observable-revision touched records, value-compared aspect versions,
                                                            leased execution, stable buckets, posture from policy
crates/worth-signal/src/logic/planner/                   R  graph parallelism, precompute and apply on leases
crates/worth-signal/src/logic/planner/precompute/executor_pool.rs  D
crates/worth-signal/src/data/graph/topology/subscriber_index/      R  ScopePath hierarchy
crates/worth-signal/src/                                 R  observation tier renamed Visited
crates/worth-signal-wasm/                                R  execution report
crates/worth-runtime-bridge/src/                         R  Query-owned envelope subscription, lease carriage,
                                                            scope-path delivery
crates/worth-runtime-world/                              R  composes the one authority
crates/worth-server/src/product_adapter/execution_pipeline/  R  leased read batches
workspaces/worth-query/crates/worth-query-declaration/   R  partition key contract, determinism
workspaces/worth-query/crates/worth-query-execution/     R  reverse index, marking, required set, input and output
                                                            cutoff, one-call advancement; partitioned owner,
                                                            per-partition reuse and retention, parallel waves,
                                                            derived-view reconstruction
workspaces/worth-query/crates/worth-query/               R  workflow frontier on leases; frontier-planning route deleted
workspaces/worth-query/crates/worth-query-host/          R  facade
tools/boundary-check/config/road1.toml                   R  crate edges, threading rule, declared threads
plans/WORTH_signal/milestone-{14,15,16,17}-plan.md       D  canceled
```

Enforcement:

- The boundary check registers `worth-execution` below Relational and Signal.
- The threading rule covers every production source in `crates/` and
  `workspaces/`. It rejects `rayon`, thread spawning, scoped threads, thread
  builders and pool construction outside `worth-execution`.
- Every other thread is declared in the boundary configuration with a category
  and reason. The categories are owner thread, input/output or durability
  worker, event loop or watcher, stream reader, async runtime, and certification
  harness. The inventory taken for this plan covers Store's Signal owner,
  mutation and checkpoint threads; Worth UI's event-loop, watcher and readiness
  threads; Bank's Tokio runtimes and process threads; Server's blocking pool;
  and the certification shard and race harnesses.
- The exemption is mechanical: `#[cfg(test)]` code, `tests/`, `benches/` and
  `examples/`. A test thread inside a production file moves into test code.
- Phase 1 lands the rule with a ratchet list naming each remaining parallel lane
  and the phase that deletes it. The list only shrinks, the boundary check is
  green at every phase, and the list is empty at closure.
- A compile-fail test proves that public code cannot construct a lease, a sealed
  batch or a reduction plan.
- Geometry-, CAD- or other domain-named types and modules are forbidden in
  `worth-execution`.

## Ordered Phases

Phases 1 to 4 run in order. Phase 5 needs no parallel machinery and may proceed
beside them. Phase 6 needs Phases 2 and 5, and Phase 7 needs Phases 4 and 6.

### Phase 1: Execution authority

- Create `worth-execution` with:
  - the authority, leases and the request policy;
  - capability resolution;
  - the backend port with the serial, native and schedule-perturbation backends;
  - safe-point cancellation, deadlines, failure ordering and panic containment;
  - work and span accounting.
- Land the portable vocabulary in `worth-foundational` and the generic doors in
  `worth-proof`.
- Build the serial oracle.
- Land the threading rule, the declared-thread inventory and the ratchet list.

The next phase may trust that any work it dispatches is bounded, cancelable,
accounted and comparable against a serial oracle, and that no new lane can appear.

### Phase 2: Patterns, partitioners and the canonical tree

- Land map, reduce, scan, fork/join, rounds and decompose.
- Land the keyed, components and bisection partitioners and the canonical
  reduction tree.
- Neutral proofs cover every courtroom case that needs no other layer, including
  insertion, deletion, signed zero and the decomposed sparse system with a
  test-local solver.

The next phase may trust the patterns as the only lawful way to express parallel
work.

### Phase 3: Relational, Server and World on the authority

- Thread the lease through Relational requests and Bridge correspondence, and
  move every Relational Rayon use onto it with stable buckets.
- Delete `RelationalExecutionModel` and the `PlanningContract` safety booleans,
  and re-baseline the affected digests.
- Move the Server read batches onto the request lease.
- Compose one authority in the World and in Server hosts, and resolve the serial
  posture on wasm32.
- Shrink the ratchet list by the Relational and Server lanes.

The next phase may trust that Relational and Server run only on the caller's
lease. The Signal and Query lanes remain on the ratchet list.

### Phase 4: Signal graph parallelism and locality

- Replace the Signal executor inputs, delete the Signal pool and the `parallel`
  feature, and move the Signal WASM counter to the execution report.
- Land three-proof graph admission with fail-closed footprints, conflict groups,
  rewiring epochs, backpressure and canonical epoch publication, and admit
  `FullParallel` precompute.
- Generalize the subscriber index to `ScopePath` with parallel candidate lookup.
- Have Bridge deliver scope paths at full depth.
- The existing parallel-versus-serial Signal certification suites pass through
  the authority in the default build.
- Signal called from Query runs the serial posture until Phase 7 passes the
  request lease.

Phase 7 may trust exact, parallel graph progression below Query.

### Phase 5: Exact invalidation

- Emit observable-revision touched records from Relational, with old and new
  index keys, and bump an aspect version only when one of its fields changed.
- Deliver every committed patch envelope to one Query-owned Bridge subscription,
  and move live queries, conditional operations and workflow coverage onto it.
- Land the settle-time reverse index with insertion replay, commit-time marking,
  upstream propagation, selection and lineage marking.
- Land clean reuse, the input-value reuse key and cutoff, stable republication,
  the required set and one-call advancement.
- Land the full-verification fallback and the equivalence-check mode.
- Rename the Signal `Touched` observation tier to `Visited`.
- Restore the public definition, rules and vocabulary in the same change, and
  point the 9.17.6 plan here.
- A neutral application proves the exact-invalidation courtroom with operation
  counts, including the randomized differential test.

The next phase may trust that the touched graph alone decides what recomputes.

### Phase 6: Partitioned managed computations

- Change the partitioned declaration as described, and re-baseline revision
  digests.
- Make `DeterministicPartitioned` execute through the partitioned owner binding.
- Settle per-partition results with their consumed facts in the reverse index,
  with item routing so `prepare` re-gathers only marked partitions.
- Retain the canonical tree in Query with eviction and branch sharing, and apply
  encoding cutoff per partition.
- Maintain partitioner output incrementally and keep island identity stable.
- A neutral application proves the isolation and reuse courtroom with operation
  counts.

The next phase may trust that partition-granular reuse is exact.

### Phase 7: Parallel advancement and remaining Query lanes

- Pass the request lease from `advance` into Bridge, Relational and Signal.
- Run the `compute` steps of each dependency-ready wave concurrently under the
  request lease, with nested partition work, and apply commits in canonical
  order.
- Move the workflow frontier and derived-view reconstruction onto the
  authority, and delete `ParallelAdmissionRoute` with its test-only executor.
- Empty the ratchet list.
- A neutral application and the Bank reference prove identical results, charged
  work and commit order at one worker and many, and span strictly less than work
  wherever independence exists.

The next phase may trust that advancement exploits every declared independence.

### Phase 8: Documentation and closure

- Update the public documentation listed below.
- Run the complete certification and the mutation probes on x86-64 and wasm32.

## Verification, Review, And Documentation

Each phase runs, for its touched crates:

```text
cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
cargo run --manifest-path tools/agent-context/Cargo.toml -- check
scripts/ci/check_workspace_rust_line_caps.sh dirty
```

It also runs formatting, clippy with warnings denied, the focused owner tests and
the affected certification suites.

Mutation probes must turn evidence red:

- skip insertion replay at settlement;
- drop the old index key from an index-membership touched record;
- honor a mark beyond the demand's snapshot;
- stop upstream propagation after one level;
- drop the predicate-field key from an un-indexed selection fact;
- stamp an equal value with a new revision;
- key reuse on the source query's footprint instead of the input value;
- re-check a clean settlement's facts on demand;
- reverse one reduction's combine order;
- build the reduction tree by rank position instead of identity digest;
- derive a bucket from the worker count;
- let a nested computation exceed its lease;
- dispatch Relational work without the caller's lease;
- publish in completion order;
- report the first failure to finish instead of the least failing partition;
- admit two graph nodes on topological level alone;
- admit a node whose inputs a same-epoch rewire proposal changes;
- default a missing footprint surface to narrow;
- decide cutoff with `PartialEq`;
- drop a partition's consumed fact from the reverse index;
- skip a reduction-tree node on recombination;
- truncate a scope path in Bridge delivery.

An independent review checks every public statement about touched records,
invalidation, currentness, progression, execution, parallelism, determinism,
locality and reuse against the code. A search of public docs and plans finds no
remaining statement that touched records are evidence only, or that a commit
performs no invalidation work, and no public identifier names the visited tier
`touched`.

Documentation must be domain-neutral, with executable examples from several
domains:

- `docs/how-it-works.md`: the touched graph as cause, marking, clean reuse,
  cutoff and one-call advancement (§9.8 and §10); the execution authority and
  lease flow, parallelism at each layer, parallel advancement inside a
  caller-pumped `advance`, and per-partition reuse.
- `docs/glossary.md`: touched graph, visited, marking, reverse index, required
  set, execution authority, lease, determinism contract,
  partition, partitioner, island, conflict group, decomposition, scope path,
  work, span. The glossary separates an execution partition from a Signal
  conflict group and from the Milestone 13 `WholePartition` scope lane.
- `docs/coding-guidelines/perf_laws.md`: the touched graph as the semantic delta
  that bounds recompute, work and span as the parallel cost law, and the
  threading rule.
- `docs/build-an-application.md`: declaring a partitioned managed computation.

## Completion And Successor Handoff

The milestone closes when:

- every courtroom case passes;
- every lane in the convergence table runs on the authority or is deleted, and
  the ratchet list is empty;
- the threading rule rejects a hand-written thread and accepts every declared
  one;
- every mutation probe turns evidence red;
- the documentation matches the code.

Successors:

- **Query 9.20 and 9.22** consume the patterns and reuse.
- **Application adopters** delete their settle-polling loops and partition their
  computations. Proprietary adopters
  consume the pinned public revision.
- **Deferred backends** (WASM helper workers, remote and distributed execution,
  accelerators, physical shard placement and rebalancing) plug into the backend
  port. Each returns when a real workload needs it.
