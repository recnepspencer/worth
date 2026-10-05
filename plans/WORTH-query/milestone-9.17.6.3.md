# Milestone 9.17.6.3: Exact Invalidation And Parallel Computation

> **Status:** In progress. Phases 1 to 4 are implemented and reviewed. Phase 5
> is implemented. Phases 6 to 8 remain. Successor to
> [9.17.6](./milestone-9.17.6.md). This is the
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
  conditional operations consume those deliveries. Application
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

- A neutral model repeated at 1 and 100 independent copies, with a fixed
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
- A workflow definition or capacity change under a waiting instance, which
  keeps its pinned revision while a new start reads the new one.

Required:

- For the fixed edit and the advances after it, producer contacts,
  source-query runs, advances and chain decisions are equal at 1 and 100
  copies, and the edit is delivered exactly. The counts come from the
  execution observer; no meter is added for them.
- Demanding a clean output costs zero source-query runs and zero producer
  contacts, and decides nothing again.
- One `advance` with an adequate budget settles every marked output, and
  exhaustion returns a typed outcome naming the remaining work; the application
  never polls.
- A downstream output demanded before its upstream recomputes reports Pending,
  and an equal upstream republication clears the marks below it with zero
  producer contacts. A decision reads its upstream's current output.
- The racing commit is caught by insertion replay, and the older-snapshot demand
  ignores later marks.
- A demand that never settled stops `Superseded` when a commit replaces its
  source, and its caller demands again. A settled demand that stays open keeps
  following its output through that commit.
- Every listed discontinuity triggers the full-verification fallback once per
  affected output: its demand runs its source query once, reaches no producer
  and decides nothing. An output demanded at least once per retained window
  never leaves it.
- A randomized differential test over all of the above finds no difference
  between marking and full verification: it runs seeded with the equivalence
  check on, and after every step each settled output equals what a plain model
  computes from scratch.

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
- Every fallback to full recomputation is reported with its typed cause.

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
  Relational seals this description into the canonical commit envelope before
  publication. Native segment and checkpoint entries write explicit touch-wire
  version 3; version 2 entries readmit with `Unavailable` precision and require
  full verification. A version 2 entry claiming exact touches is rejected.
- **Delivery.** Marking consumes every committed patch envelope on the branch
  through one Query-owned Bridge subscription with no Signal targets.
  - It carries the full published aspect change with its field path, the
    index-membership records, and, from Phase 4, the scope path at the depth
    Signal sealed it.
  - Delivery runs in commit order and synchronously with commit visibility.
  - Conditional operations consume the same subscription, and their
    correspondence-based matching is replaced by marking. Signal
    correspondences keep delivering to Signal-hosted nodes and never feed
    Query marking. No second path feeds Query invalidation.
  - Live queries are caused by committed application emissions, delivered in
    Product commit order by Query's commit-causality source. They invalidate
    nothing, so they are not a second invalidation path. Each delivery
    re-admits the principal and re-reads at the cause's Product observation. A
    future live query whose result depends on data registers in the reverse
    index like any other consumer.

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

  It runs once per affected output: the verified row is marked clean again,
  so its next clean demand runs no source query. A restored dependent carries
  no upstream edges and executes once instead. It is observed as source-query
  runs that reach no producer; it has no counter of its own.
- **Commit-time checks.** A commit still recompares its attempt's read facts
  (optimistic concurrency control). Marking replaces only demand-time
  re-verification.
- **Equivalence check.** A debug and certification mode runs full verification
  beside marking and fails on any difference.
- **One mechanism.** One invalidation mechanism serves outputs and conditional
  operations, and no parallel lane remains. Workflow coverage needs none: a
  waiting instance pins its definition revision and history basis, which are
  never marked, and start and migration admission read definition and lineage
  capacity fresh.
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

#### Phase 5 decision-context and publication boundary

The three-part reuse key identifies prepared input; it is not, by itself, proof
that a completed handler has the same decision context. `DecisionReader` also
exposes the request key, principal and operation scope. The installed producer
declaration must explicitly state its deterministic reuse contract and these
context dependencies before execution. An existing handler without that contract
remains executable and is ineligible for input cutoff. Use the existing
`worth-foundational::execution::DeterminismContract` vocabulary; changing this
contract or its dependencies changes the installed producer edition.

The handler boundary records actual context consumption only to enforce the
installed declaration. It does not discover the dependency contract. A read of
undeclared context, or a managed computation without retained currentness
dependencies, prevents creation of the completed reuse proof. It must never
silently omit a dependency. Declared key dependencies compare the original
canonical key identity; principal dependencies compare owner-issued native
identity and freshness evidence; scope dependencies compare exact operation
affinity. These comparisons, their retained backing and canonical encoding draw
from the same admitted request meter. Authentication and operation authorization
precede reuse even when the handler declares independence from that context.
The same completeness check covers the inner operation projection reader:
public raw `reader()` access prevents input reuse in this phase. Its native
version is one observation outside declared decision facts. Tracked outer
projection methods borrow the private reader directly and remain eligible.

The completed proof carries the handler fact prefix separately from the freshly
read source suffix. Input sameness, the declared decision context, the dirty
handler facts, consumed-output closure and the native output witness must all
close before a sealed stable-output publication proof exists. Missing proof,
including a restored row without live proof, selects ordinary execution or the
specified full-verification fallback. No contact counter or matching digest may
construct the publication proof. `ContractEquivalent` needs its installed
identity-bearing predicate; its identifier alone cannot enable reuse.

Stable publication belongs to the existing Query lineage and demand owners. It
creates a new settlement identity at the actual current product observation,
retains the original output correspondence and performed-receipt provenance,
and installs fresh source facts and postings. It does not perform a new World
commit, invent a successor generation or manufacture a commit receipt. The
public settlement distinguishes stable reuse from a newly performed commit.
Its readiness evidence reports zero producer and delivery contacts.

The executor carries a typed progression: original prepared input custody goes
to fresh execution, or owner-published stable lineage goes directly to Ready.
Stable consumers preserve that authority rather than synthesizing a receipt or
re-running delivery. The performed native output witness contributes exact
entity-kind and aspect-revision facts to the alias's canonical fact set, so
marking, consumed-output checks and downstream optimistic commits use the same
authority.

Settlement registration first projects source and output facts into one owned
map of keys and composite fact ordinals. A consuming prepared-posting value
then installs each unique key into the reverse index once. The immutable facts
remain shared; grouping must preserve every matched ordinal and all admission
and retained-capacity charges.

Ready storage is prepared before effects and travels in the required-output
execution envelope, then through published and delivered checkpoints to its
terminal fill. The final shared completion owns its refundable capacity until
the last reader releases it. Clean reuse admits the actual Query read and
disclosure authorities without executing the source query. Its exact accepted
candidate pins that completion; native currentness certification and Product
basis binding produce sealed proof consumed by the settlement constructor while
the admitted Query plan remains live. The constructor shares the accepted
authority rather than copying its historical receipt or re-verifying facts.
Partition reuse and required-wave scheduling consume this same handoff.

Branch coordination has a finite installed aggregate retained-byte profile.
One refundable ticket funds a lane and its Weak map entry until both observers
release custody. All lane acquisition is fallible before owner effects. Stable
publication also samples the admitted request's cancellation and expiry after
blocking Product guard acquisition, immediately before source CAS. Its typed
stop and full prepared custody leave both owner guards before denial allocation
or cleanup.

Prepare all lineage payload, retained capacity, prerequisite claims and registry
handoff storage before cutover. The existing occurrence commit lane holds a
prospective lineage address while the registry prepares its prerequisite
vacancy. This address is preparation custody, not accepted output authority.
The final lineage guard covers actual path admission, geometric backing growth
preparation, and the product-currentness callback; any new empty path remains
invisible under that same guard and has a prepaid inverse on failure. Existing
record backings remain unchanged until CAS succeeds. Under that guard, validate the
original partition locator and settlement, perform the source-owner CAS, then
infallibly append the immutable stable row and replace its exact derived
locator. Family reads and partition reads must select that same new row. A
denial before CAS leaves the prior row, locator and postings usable. Historical
rows and receipts remain immutable. Checkpoint capture preserves the stable
reuse distinction and original provenance; restoration does not promote it to
new performed authority and requires the existing reconstruction verification.
The existing occurrence commit lane and owner-issued product observation pin
remain held and currentness-checked through cutover; native actor CAS alone does
not prove that the independently owned product reference stayed current. World's
observation port holds its existing operation
reservation and upgraded service for a short callback over the branch read
guard. Only that callback issues the non-escaping `CurrentProductHead` witness
required by Query's stable cutover. A retained observation alone cannot replace
this witness. Native companion image allocation must also be prepared before
that callback; its consuming CAS returns retired custody for later cleanup.
Stable publication accepts only a current-source registration bound to the
cutoff's exact native root, commit and position. Historical insertion replay
remains available for ordinary post-effect registration; it cannot advance a
stable cutoff across source movement. The performed receipt retains the exact
owner-issued settlement address, so replacing a same-generation partition
locator cannot hide its immutable original row.
Lock order is lineage then source edit, with no reverse lineage acquisition from
source preflight. No registry lock is acquired under that pair. The prepared
row, locator and prerequisite handoff use only moves after CAS; displaced
payload destruction is deferred until outside the critical window.

The decisive tests include equal prepared input under changed request keys and
principal identity revisions; handlers both dependent on and independent of
each declared context field; an undeclared getter; an unavailable equivalence
predicate; and managed computation without currentness evidence. Dependent or
ineligible handlers execute, while lawfully independent handlers reuse with zero
contacts. A forced actor-CAS conflict and a one-unit work or capacity shortfall
must preserve the old settlement. Independent family, partition, retained-read
and checkpoint observations must agree after successful stable publication.
Bypassing context completeness, native output verification or the locator CAS
must make the evidence fail.

The existing certification topology supplies the first concrete producer cases:
`anchor-a` uses tracked projection reads; `anchor-isolated` additionally calls
the raw inner reader's native `version()`; `anchor-island` reads the undeclared
request key. Editing each scope's successor position changes its source epoch
without changing its canonical operation input. Re-demand must reuse the first
case and enter the actual installed handler in both negative cases. Use public
producer-contact work and current output observations as the oracles; fixture
provider callbacks alone do not establish handler entry.

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

A caller that owns a declared work ceiling states it with
`ExecutionWorkCeiling`. The patterns it runs inside meter against the narrower
of that ceiling and the lease's, so a run without a lease exhausts at the same
canonical boundary as a leased one. The scope hands its closure no kernel
context: patterns stay the only way to charge work.

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
does no re-partitioning. Routing reports its work units, so a caller that
partitions before any pattern is admitted charges them against its own ceiling.

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
- **Checked result declaration.** A node may declare
  `max_checked_result_heap_bytes`, the maximum additional heap of its checked
  result, trace and keyed output; dependency capture has a separate bound.
  `None` keeps adaptive admission and `Some(0)` explicitly declares a heap-free
  result. Before callbacks, the epoch admits a common result grant at least as
  large as every selected declaration or rejects the candidate. The checked
  callback enforces each node's own limit. Fresh Signal snapshots write schema
  3; schema 2 remains readable with an absent or null declaration, while a
  schema 2 payload carrying a non-null declaration is rejected before restore.
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

- `ApplicationComputationPartition` becomes the partition key contract: a
  canonical encoding, `Send + Sync`, and its identity. The canonical encoding is
  the declaration's prefix-free encoding of the key's `Serialize` form, the one
  that identifies structured operation inputs. Two keys are the same key exactly
  when their encodings are equal. A key carries no `Ord` or `Eq` of its own: a
  second notion of sameness could disagree with the encoding.
- The declaration crate owns the one derivation: the SHA-256 of a key's
  canonical encoding under its declared identity, through the encoder's own sink
  and admission. The 32-byte digest is the keyed partitioner's key, and the
  `PartitionIdentity` is its first eight bytes, big-endian. The encoding is
  one-way, so Query keeps the typed key of each partition and hands it to the
  kernel in the partition view.
- Two different digests that share a `PartitionIdentity` are the partitioner's
  identity collision, surfaced as a typed denial that names the partition.
  *Limitation:* whoever controls key values can craft such a pair, and it denies
  every run while both keys are in the input.
- Order is partition identity order, which is digest order: deterministic and
  not chosen by the author. Under `CanonicalBitwise` it is the reduction order,
  so the encoder and the truncation are frozen parts of a result's meaning.
  `ORDERING` is deleted from
  `ApplicationManagedComputation`, and a `Deterministic` computation declares the
  platform's single-partition key, `ApplicationSingleComputationPartition`.
  Program validation denies a `Deterministic` computation that declares any
  other key and a `DeterministicPartitioned` one that declares the single key.
- Lineage already uses "partition" for its source-selection digest, so every
  type of this concept spells it "computation partition".
- `DeterministicPartitioned` implies per-partition reuse. `Reuse` keeps its
  meaning: disposable warm-start evidence.
- The declaration names its `DeterminismContract`, defaulting to
  `CanonicalBitwise`. It is part of the manifest record and of the program
  revision. *Limitation:* installation refuses every `ContractEquivalent`
  declaration. Predicates are installed only when the execution authority is
  constructed and are observed only by requesting a lease; Query holds no
  authority at installation, and a declaration names only the contract id, so a
  predicate's 32-byte identity cannot enter the revision digest.
- These changes alter program revision identity, and existing digests are
  re-baselined in Phase 6.

An output keeps one settlement row, and the settlement identity is not widened.
Marks stay per row and per fact ordinal. A routing table on the row maps each
fact ordinal to the computation partitions that read it, so a touched fact marks
exactly those partitions. Retained partition results are keyed under that row by
computation identity, implementation edition and partition identity.

Retention belongs to Query. `worth-execution`'s persistent `ReductionTree` is
the retained store: its nodes are shared, so a clone costs the same at any
partition count. Query owns the retained tree value, its retained-byte charge
and its eviction. A branch fork clones the tree, so the two branches share every
node neither has replaced. No node store keyed by child identities is built.
Exceeding the computation's `maximum_retained_bytes` evicts to full
recomputation for that computation.

A partition whose recomputed encoding equals its previous encoding stops
propagation, so nothing above it recombines.

Full recomputation is the fallback in these cases. Each is reported as a typed
cause, and none is counted:

- a basis that is not continuous (see Currentness, demand and advancement);
- the partitioner reports a rebuild;
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
| Commit-time marking | O(touched records + matched facts + marked downstream closure) logical routing operations, plus separately bounded physical index navigation and selected copy-on-write paths; no scan of settlements or waiting work |
| Clean demand | Zero fact checks and zero source-query re-runs |
| Fixed edit at 1, 10 and 100 model copies | Equal logical marking operations, verification work, producer contacts and reverse-index capacity bytes per consumed fact; separately bounded and reported physical index navigation and selected copy-on-write paths |
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
    type Partition = RegionKey; // Serialize (canonical encoding), Send + Sync
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
    // Clone + Send + Sync + ChargedBytes + CanonicalBits
    type PartitionResult = RegionTotals;
    type Output = RegionBalanceSheet;
    type Stopped = BalanceDenial; // Send + ChargedBytes

    fn partitions(
        &self,
        entries: &PostedEntries,
    ) -> WorthQueryComputationPartitionPlan<RegionKey> {
        WorthQueryComputationPartitionPlan::keyed(
            entries.items(),
            |entry| PartitionItemId(entry.id()),
            |entry| entry.region(),
        )
    }

    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, RegionKey, PostedEntries>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<RegionTotals, WorthQueryManagedComputationDenial<BalanceDenial>> {
        // Each item is its identity and its position in the planned input.
        RegionTotals::sum(partition.input(), partition.items(), checkpoint)
    }

    fn reducer(&self) -> WorthQueryDeterministicReducer<RegionTotals> {
        WorthQueryDeterministicReducer::canonical(RegionTotals::zero, RegionTotals::combine)
    }

    fn complete(&self, totals: RegionTotals) -> Result<RegionBalanceSheet, BalanceDenial> {
        RegionBalanceSheet::validated(totals)
    }
}
```

The framework runs `partitions` and gathering in `prepare` on the owner thread,
`compute_partition` on the lease, the reducer over the canonical tree, and
`complete` on the owner thread.

The plan names each item by a stable identity of the item itself, never its
position, so a reordered input plans the same partitions with the same members.
A partition's view lists its items in ascending item identity order, and
`combine` sees partitions in partition identity order; the canonical tree fixes
how they associate. Naming two items alike is denied. The owner is installed
with `partitioned_computation`: installation refuses a `DeterministicPartitioned`
computation bound to the single-partition owner and a `Deterministic` one bound
to a partitioned owner.

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
  src/reduction/{plan,tree}.rs                           N  persistent treap; a clone shares every node
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

Phase 5 refines the existing Query owners along their authority boundaries:

```text
worth-query-execution/src/domain_computation/primary_graph/
  application_contribution/producer/
    input_reuse_contract.rs                         N  static declaration, portable meaning
    registry/{declaration,semantic_edition}.rs       R  installed contract and handler edition
    execution/input_cutoff/                         N  admitted input -> verified reuse or execution
      preparation.rs                               N  selection, declared context and dependency proof
      publication.rs                               N  consumes proof through lineage/demand owners
  handler/
    invariant.rs                                   R  DecisionReader records context consumption
    decision_context.rs                            N  finite consumption and completeness evidence
  application_attempt/read_set/
    handler_fact_boundary.rs                       R  complete prefix, independent of reuse eligibility
    completed_input_reuse.rs                       N  sealed proof of the exact completed invocation
  output_lineage/
    input_cutoff.rs                                R  pins prior immutable owner record
    stable_publication/                            N  same-observation publication lifecycle
      preparation.rs                               N  payload, capacity and exact locator claim
      cutover.rs                                   N  owner CAS and infallible row/locator install
    partition_index/                               R  exact current locator replacement
    invalidation/                                  R  canonical settlement posting and edge owner
      settlement/postings.rs                       N  prepared unique keys and composite ordinals
    required_settlement/current_accepted.rs         N  sealed exact accepted currentness proof
  application_output_demand/
    registry/record_capacity.rs                    N  ordered record storage and escaped wake custody
    registry/required_custody.rs                    N  existing aggregate lifetime capacity
    registry/ready_backing.rs                      N  prepared storage -> shared Ready completion
    registry/stable_reuse.rs                        N  accepted stable authority and ready handoff
    settlement/stable_reuse.rs                      N  current observation and original provenance
    registry/accepted_checkpoint.rs                R  capture preserves publication posture
```

These are semantic destinations, not empty placeholders. The producer contract
owns declared sameness, the handler/read-set boundary owns completion proof,
lineage owns immutable output history, and the demand registry owns accepted
readiness and prerequisite custody. Publication orchestration consumes these
owners' proofs; it cannot mint their authority. Phase 6 adds partition preparation
and reuse beside the producer cutoff boundary, and Phase 7 adds required-wave
scheduling beside demand progression without moving these facades. Proof
constructors remain visible only to their producing owner, and exhaustive
accepted-authority matches enforce propagation through reads and recovery.

The registry keeps one ordered record owner. Exact-key lookup charges the
selected tree descent; insertion prepares key storage, possible tree splits and
the wake's independent lifetime before visibility. Cached Ready values remain
in that owner. Removing its last record preserves the empty tree's storage
credit until the tree is destroyed, and an escaped notification retains its own
credit until its final owner drops. This avoids a second readiness cache and
keeps unrelated records out of current-demand lookup cost.

Refreshing a demand carries its original request admission through retained
producer selection and registry replacement. The existing reconstruction
selector preserves its bounded remainder on both success and failure; its
legacy Work is reconciled into that same admission. This bridge does not claim
that inherited reconstruction copies have entered the new ordinary cost class.
Replacement prepares matching obligation slots, exact missing commit
provenance, retained record growth and required membership before either record
changes. Failure leaves both records and their reservations usable. Retired
backings keep their aggregate credits through destruction outside the registry
guard; a prepaid owner cleanup refunds those exact credits afterward.

Query permission preparation precedes graph-work construction. Its owner selects
the Product basis, holds the security lease, validates the current principal and
scope, and issues the installed policy authority with a reserved Query session
identity. The sealed permission carries that exact basis and identity forward:
Clean reuse consumes it with the exact accepted Ready completion; disclosure
consumes it to construct graph work in the reserved session. Neither branch
selects a replacement Product or repeats permission admission. Eligibility and
resource refusal remain typed outcomes, distinct from policy denial.

Producer mutation installation selects its exact operation obligations and
validates their semantic owners once, against the final installed provider
support. Admission retains that immutable selection and lowered resource basis
in an opaque installation template; the template reserves no live capacity and
grants no permission. Invocation consumes the current operation owner's proof,
joins its full schema binding and obligation identity with that template, and
checks the same installed support authority before a funded shallow retention.
It derives only the unique invocation resource identity and reserves the actual
participating capacity ports on the original request meter. Immutable obligation
copies, selection walks, owner validation and resource strategy rediscovery do
not belong to that hot invocation; its counters report those checks as zero.
Selected session construction still consumes fresh permission and the exact
retained Product basis. Required-wave and partition successors use these same
installation and invocation boundaries rather than adding another authority or
allowance.

Installation also owns one immutable compiled operation contract value.
Admitted invocations retain a funded shallow share of it; they do not copy its
declaration vectors and strings. Borrowed contract inspection and content-based
reinstallation comparison preserve their meaning. Sharing the representation
does not grant current issuer authority or replace its schema and operation
identity checks.

Native principal probes, selected-index lookup, authorization traversal and
framework-owned preparation use the carried request meter before reads, copies
or allocation. The installed application principal decoder runs freshly, with
its ordinary acceptance semantics. Application-authored decoder internals retain
their existing application cost contract; native admission does not claim to
meter arbitrary application instructions or allocations. Clean may not substitute
prior decoder success for fresh principal acceptance.

Permission prepares only its selected principal/scope field indexes. Missing
currency reconstructs those exact fields through the native index owner on the
same request meter; it does not materialize unrelated record aspects or rebuild
the installed index catalog. Disclosure must additionally prepare the indexes
consumed by its actual graph contract. Native generation retention keeps its
existing native cost class; request scratch prepayment is not an aggregate
native retained-byte ledger.

Cursor teardown retains creation-time credit for present-row removal and inner
entry destruction, and destroys extracted custody after the registry unlocks.
An absent-occurrence lookup belongs to the existing Query Product-retirement
cleanup lane: it allocates nothing and is bounded by the installed required
capacity divided by the mandatory positive cursor-row claim. It can repeat per
cleanup retry and is not charged as advance/read request Work.

The retained read owns its final storage credit. Query permission, actor
certification and any disclosure fallback continue on the same request meter.
Private currentness proof is consumed while its exact selected basis remains
pinned. These boundaries also serve partition reuse and required-wave scheduling
without adding another authority or allowance. Phase 6 can add partition
selection after permission, and Phase 7 can schedule prepared required work
without changing either policy admission or publication authority.

Serial required advancement holds one selected Product operation through each
dependency wave. An actual performed publication requires selection of the
authentic new World head before a downstream producer continues; that next wave
starts inside the same caller advance. Each producer borrows the exact
Product/native basis for its wave, validates its
own installed source, principal, scope and policy, and uses the same request
admission. Before the wave enters per-producer progression, a sealed shared
selection phase funds one final-owner Arc around the already issued Query
snapshot, Native retention obligation and selected-program interpretation. The
ordinary exclusive lease remains inline. A producer retains a metered local
identity and Product share of this custody; it does not register another Native
snapshot or reinspect program support. A terminal receipt distinguishes local
share disposal from physical Native release. Only final-owner release or drop
closes the snapshot, Native obligation and program guard. Preparation refusal
returns the still-exclusive selected operation intact.

Disclosure may retain an admitted copy of that pinned selection; it
cannot replace it with a fresh head selection. The registry pins the exact
required row's Ready completion and retained source together under its token,
version and Idle checks. This pin is storage custody, not currentness proof.

When currentness reaches an unresolved consumed output, the lineage/actor owner
returns the exact pending settlement with its funded evidence custody. The
coordinator joins that identity through the existing exact settlement index and
certifies that prerequisite's own source before retrying its dependent. It must
not infer the prerequisite from a latest producer result, scan Clean rows, or
acknowledge an unresolved edge. Missing evidence, unavailable Ready storage and
resource refusal preserve pending custody. Only the actual accepted consequence
permits the exact-version acknowledgement. The public A/B/C fixture must prove this
handoff through real handler `current_output` reads, one caller advance, original
performed provenance, no new World commit and zero downstream producer contacts.

The native delivery owner counts its actual posting-key lookups, matched
postings, newly marked fact ordinals, visited vertices and consumed downstream
edges. A fixed diagnostic report travels in the existing branch image with the
native commit identity; derived-only image edits preserve that report. Reading
it requires the exact native root, commit and position, so a later image cannot
be reported as another native delivery. An unavailable change retains an
explicit discontinuity posture rather than fabricated per-row counts. Physical
tree navigation and retained-capacity bounds continue through their existing
admission owner. An independent native revision oracle over actual created rows
at 1, 10 and 100 copies checks selected-mark locality; this owner proof does not
substitute for public producer-contact or the full differential courtroom.

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

The phase boundary is a consuming progression: planned work becomes an opaque
checked epoch selection, preparation consumes that selection and its resource
grant, and apply consumes prepared proposals with their disjointness proof,
caller lease and immutable resource summary. Legacy serial work has a separate
variant. Publication consumes a fully prepared epoch; it does not repeat
admission or discover a new memory policy after evaluation. Capacity bounds
come from the storage and packet owners that allocate those structures. Phase 7
passes its lease into this same boundary and consumes the resulting progress;
it does not rebuild Signal admission or create a second execution lane.

Each logical stage mints one readiness epoch and carries it in both admitted
variants. Resource backpressure can split that stage into several publication
epochs without changing its semantic readiness identity or canonical task
order. Physical progress still counts the epochs that actually published.

Before evaluator dispatch, Execution reserves the admitted apply and candidate
maps' memory under their selected leases, including any Serial child and
checkpoint contexts. Selection accounts for their simultaneous reservations
inside the request. Candidate slots come from the selected producers' declared
aspects; their storage bounds come from the reverse index and admitted output
heap. Canonical output deltas later activate those slots, preserving scoped
lookup. Consuming prepared-map values retain the reservations through
settlement. They bind the preparation parent and physical accounting ledger;
dispatch under a different parent is denied. Dispatch resolves current work,
cancellation and deadlines and acquires workers then. It does not readmit the
same memory or retain a stale work allowance. Host contention therefore cannot
turn a successfully admitted epoch into a later apply or candidate memory denial.

Prospective subscriber settlement distinguishes producer output edits from
consumer operational edits before callback dispatch. Producer edits may replace
artifacts and semantic diagnostics; consumer edits may change only topology,
invalidation state and pending revalidation. Consumer preparation preserves cold
artifacts and shares immutable warm companions through the node storage owner.
The prepared mutation surface makes artifact writes unavailable to that role.
A previously selected consumer becoming a producer requires the incremental
producer capacity before dispatch; selection alone is insufficient proof.

Node draft capacity comes from the unique selected-node union. Cause preparation
accounts for selected pages, replacement cells, index paths and the actual peak
of sequential staging. Request allocation and retained logical custody remain
separate: shared historical roots do not become new request allocations, while
their complete custody remains reserved by the retained storage owner. Moving
between a cause map and its successor vectors releases predecessor scratch only
when those allocations are actually gone. These changes preserve canonical
cause handles, atomic publication and the original request limits.

Canonical scope sets and prepared invalidation caches own only their populated
scope vectors. Empty sets allocate no scope backing. Their public values and
serialized sequence shape remain unchanged; every nonempty backing and copy is
admitted at the existing scope or cache owner. Prepared caches move into the
installed node's scope storage without duplicating strings. Persistent index
forecasts account for the final shared index structure and one in-flight edit,
rather than summing a complete retained path for each final page. Node root
copy-on-write storage is admitted separately from node draft backing and from
retained logical custody.

QA for this refinement includes the real checked Bank rejected-descendant case
at 4,096 outputs under its existing 16 MiB preparation limit, independent
allocation evidence for shared-history cause edits, consumer-to-producer
capacity upgrades, and retained preparation denial preserving both roots and
observable artifacts. Bridge, World, WASM and Query consume the same prepared
graph boundary; this refinement creates no additional execution authority.

### Phase 5: Exact invalidation

The touched graph becomes the invalidation cause behind the public facade.
Proofs use real A/B/C producer handlers and their `current_output`
dependencies. Each open item closes one journey through the public facade
before machinery for another lands.

- Emit observable-revision touched records from Relational, with old and new
  index keys, and bump an aspect version only when one of its fields changed.
  *Completed.*
- Deliver every committed patch envelope to one Query-owned Bridge
  subscription. *Completed.*
  - Conditional operations read the subscription, so any writer's commit on
    the branch reaches them. Invalidations an observation consumed are never
    dropped: after a stale, reordered or failed clock reading they stay on the
    operation for the next accepted or duplicate observation's batch. Owed
    invalidations stay bounded: ones Query converges as one keep only the
    newest commit, a carried dependency that changes differently again
    escalates to `RefreshAll`, and `RefreshAll` absorbs everything owed. A
    healed lagging cursor's batch carries `RefreshAll`. *Completed.*
  - Workflow coverage needs no marking. Running instances pin their definition
    revision, and a new revision supersedes it only for new starts. Capacity
    bounds live instances per definition lineage, so it limits new starts. A
    definition or capacity change therefore leaves nothing to reconsider for a
    waiting instance, whose pinned basis is never marked; start and migration
    admission read definition and capacity fresh. The pin covers only facts
    read from the pinned definition, and lineage capacity is a live fact. No
    waiting step reads it: the `WorkflowInstanceCapacity` fact is built only at
    start (`application_attempt/workflow_instance_program.rs:148-155`).
    *Completed:*
    - A certification proof holds a waiting instance on its pinned revision
      across a definition change and a capacity change, while a new start is
      sent to the new revision and reads the new capacity.
    - The pinned instance completes at its own revision's node while its
      lineage is full, so no waiting step reads lineage capacity.
    - The workflow projections in `invalidation/fact_keys.rs` are deleted.
      Workflow attempts publish without an output binding, so no output's
      decision reads definition or capacity truth; one that did has no posting
      to mark it and verifies in full.
- Land the settle-time reverse index with insertion replay, commit-time
  marking, upstream propagation, selection and lineage marking. *Completed.*
- Close multi-root (diamond) dependencies: one advance follows every consumed
  output edge into a shared dependent. *Completed:* after both roots change,
  one caller advance settles the shared dependent reading both new values,
  each producer contacted once; a dependent whose upstream an earlier advance
  already refreshed resolves against that Current row instead of waiting on it.
  - Every chain node's decision reads its upstream values. A root that returns
    to an earlier value settles the shared dependent in one advance again,
    reading the value it returned to.
- Land clean reuse, the input-value reuse key and cutoff, stable
  republication, the required set and one-call advancement. *Completed:*
  - One C advance discharges an unchanged-input upstream cutoff without
    another World commit or C producer contact.
  - When A publishes a replacement, one C advance follows it to the new World
    head and keeps C's output entity.
  - Reusing a retained Ready skips the source-query kernel, and changed input
    reenters it.
- Public reuse holds under explicit, independent source and producer resource
  policies. *Completed.*
- Retry an interrupted successor on the same caller: the caller's next advance
  resumes B. *Completed:* a request that loses its authority mid-refresh stops
  as that request's own Cancelled or TimedOut, and a refresh that stops without
  publishing returns its row to the Ready it reopened, so the next advance
  claims B again without contacting the settled root.
- Consume the required-work queue in production advance, and keep retained
  custody steady across cycles. *Completed.*
  - Under alternating input at small retention, a middle consumer that reads
    its upstream values settles in one advance every cycle. A caller successor
    whose settlement cannot be retained is refreshed again on the same wave:
    promotion follows the caller's own refresh line, and the newer refresh
    takes the older one's custody slot. Custody stays steady and the chain
    stays live.
  - At that retention the index keeps every row of the chain. No
    registration is refused, and every cycle refreshes each row once, from
    the first:
    `checkpoint_recovery/required_chain/required_queue/steady_retention.rs`,
    `the_required_chain_stays_live_for_a_hundred_cycles_at_small_retention`.
  - While a dependent's demand is open, the upstreams it consumes are
    required, transitively. A dependent opened after every chain demand
    closed refreshes its stale, undemanded upstream and settles over it in
    one advance. A refresh the World supersedes before it publishes gives its
    occurrence back to the newest Ready it replaced, so the wave refreshes
    that row again instead of failing the dependent.
  - A caller whose advance is refused required custody ends the refreshes
    it carried, so it never waits holding custody. The stop is retryable
    when one of those refreshes had published, or when another demand holds
    custody its advance or close moves: a demand open outside the caller's
    chain, or one open on a chain row that is stale or still refreshing.
    Otherwise a retry would meet the same custody, so the stop is terminal
    for that caller and leaves the rows claimable: retention is the caller's
    budget. A demand refused at its start holds no row yet, so every open
    demand counts as another one. Every refusal of the shared budget takes
    this one posture.
  - Required work first retires closed cached rows nothing holds, and a
    superseded row retires once its own demands close and the newest row of
    its occurrence has settled Ready, handing its settlements and
    dependents' claims to that row. However a refresh stops, a continuation
    whose row returned to the Ready it reopened ends before the next claim
    reserves custody, so a retry holds no more custody. A refresh under a
    new key that its last owner lets go before it publishes gives the
    occurrence back to the Ready it replaced. At three Ready rows per
    settled demand, a reopened dependent's first advance stops retryable,
    and its retry settles beside the unrelated caller's open Ready.
  - Eviction degrades to Fresh, never to reuse and never to a standing
    deferral. *Completed:* a row retired for custody drops its settlement
    postings and keeps its lineage. Only a row holds the claims on what its
    record consumed and answers its pending edges, so a lineage head that
    consumed upstream outputs and that no row posts is an evicted one, and
    the next start of its producer succeeds it. Input cutoff neither reuses
    nor awaits it, the commit is its successor instead of a replay, and the
    successor's publication retires its settlement. This is the single rule
    because every Ready row of a dependent then holds claims on what it
    consumed: no upstream row is evicted beneath a live dependent, so no
    pending edge is left without a row to answer it. Evicting the lineage
    alone would not hold, since the World would replay the old commit into a
    Ready with no claims. A head that consumed nothing needs no row, and
    reuses or replays as before. The newest settlement of an occurrence also
    supersedes those of the rows it replaced, so postings handed to the
    newest row retire and repeated refreshes hold steady custody. At eight
    to ten rows, a dependent restarted after its upstream refreshed decides
    again and settles within eight advances.
  - Conservative posture: before a dependent's first claim, its upstream
    owner reads as a demand open outside its chain, so a refusal there
    offers a retry. This ends once that demand closes.
  - Retained state follows what live demands hold, not how many generations
    were published. An unrelated caller demanded every cycle drives the
    required wave every cycle, and retained invalidation bytes and required
    custody are equal every cycle at three and at five Ready rows per
    settled demand. Lineage retirement frees the unpinned generations behind
    a pinned one, and a row replaced under a refresh leaves the required set
    when that refresh publishes.
  - A refresh refused custody while its caller still holds its own reopened
    stale Ready releases that Ready and goes Fresh; the stop stays retryable
    for a demand that has not settled. An overwritten middle output settles
    at seven to ten rows of required custody.
- Land the full-verification fallback and the equivalence-check mode.
  *Completed.* CI runs the checkpoint courtroom with the equivalence check,
  the execution observer and World operation control.
- Close the exact-invalidation courtroom through the public facade, over a
  neutral model of independent rings: a root and two chained consumers whose
  decisions read their upstream values. *Completed:*
  - Locality. One root-input edit costs the same producer contacts,
    source-query runs, advances and decisions at 1 and 100 rings with every
    chain demanded, and is delivered exactly. Afterwards every open demand
    of every ring costs no source query, no producer contact and no decision.
  - Randomized differential. Fixed seeds drive a root-input edit, a write of
    a field to its current value, a fetched field the input omits on the
    next body and on a farther one, an overwrite of a root and of a chain
    output by a writer that is not its producer, a delete and create under a
    new or a retired index key with its relations removed and added, a
    root-input commit racing a started demand, and a checkpoint restore that
    ends the world and opens the next from its capture. Each commit is
    followed by no demand, the open chain in a drawn order of its three
    demands, a fresh ring, or every output twice. The equivalence check runs
    throughout, every settled output equals the model, the output read
    through each settlement's own observation equals a fresh read, the
    second demand of every output reaches no producer, and a failure prints
    its seed and steps.
  - One advance settles every demand, whether it starts its row or has
    settled before, and the courtroom fails on a second one. `Pending` is
    answered only for work outside the call: another caller's running work,
    a readiness delivery that has not arrived, or a source the application
    must disclose again. The outputs of a performed write settle the same
    way: the root and every level of its dependents in the call that settles
    the root. A dependent refused its work fails that call with the root
    published; the next call is refused again and publishes nothing. The
    facade's `settlement_attempts` and the bank's rounds policy repeat an
    advance across such waits and nothing else.
  - Observation capacity. The same sequences run in a branch that admits 16
    product observations, and no caller is refused one or asked to retry. A
    demand, a read or a mutation commit the branch has no observation for
    retires closed cached rows, then releases the sources of performed
    writes nobody holds, until it is admitted; admission is the one place
    that reclaims. A released write keeps its obligation, and its output is
    produced from the source its next demand discloses. A write-only caller
    cannot fill a branch for good: more writes than the branch admits
    observations are all performed, and a demand and another write follow.
    A retired row costs its next demand one source query: a root then reuses
    its input and reaches no producer, and a chain node is decided again by
    one producer contact. A released write is no longer re-entered through
    its receipt: recovery answers `RetainedBasisUnavailable`, and the same
    request is answered as already committed, not performed again.
  - Discontinuities. A restored root is readmitted from its checkpoint facts
    with no source query and no producer contact. A restored consumer carries
    no upstream edges, so it goes Fresh: it executes once, decides over its
    upstream again, and never settles over what the checkpoint held. A
    checkpoint carries producer facts only for an output that consumed no
    other, under a new fact wire version; facts at an older version are
    never read, so their row starts Fresh. After more commits than the
    retained window holds, an unedited chain is verified in full once, by
    one source query, and reaches no producer. The verified row is marked
    clean again, and a clean demand moves its row's verified-through
    position to the position it was checked against: an output demanded
    inside every window never leaves it and runs no source query of its own
    over more cycles than the window holds. An edited output executes again.
    A forked branch settles on its own edit and leaves its parent's outputs
    untouched.
  - A restored output takes its producer mode from the first advance whose
    chain reaches it: its own demand's or a dependent's. Demanding only the
    last consumer of a restored chain settles the chain.
  - Checkpoint format 8 has one decoder. Outputs of a format 5 to 7
    checkpoint restore as descriptive rows: their facts are never read, no
    tracked read precedes a demand, and the first demand executes once under
    Preserve.
  - Republication. A checkpoint restore stays Fresh until verified. A
    restoration on the runtime that performed the output continues the
    suspended performed record and seals a new witness; its first reader
    verifies in full, and a row left without a sealed witness requires full
    verification. A head without facts yields no candidate. A stable alias
    is compared by its origin's witness.
  - An exact selection is over a live output whose row carries a sealed
    witness and a settlement the input cutoff verifies. The cutoff declines
    a row still in its checkpoint posture and a settlement the owner cannot
    place under the selected source: no row, another runtime or branch, or
    a read before its basis. Selection asks the cutoff's own predicate,
    `cutoff_declines` in `output_lineage/input_cutoff/verification.rs`, so
    the two hold one list. A row of this runtime the cutoff declines, or
    one without a sealed witness, is not exact: its demand selects the
    Preserve posture, which runs once over the live output and is then
    reused. A settlement that requires full verification for any other
    reason is compared in full and reused with no producer contact: a
    restoration on the runtime that performed the output,
    `an_output_restored_on_the_runtime_that_performed_it_is_reused`. An exact
    producer that declares no Preserve posture reuses its live output or is
    refused `MissingApplicableProducer` before any effect: it never
    executes over a live output. A refresh in a family whose other producer
    declares Preserve switches to that producer, which runs once over the
    live output: `checkpoint_recovery/generated_restoration.rs`.
  - A settlement retains only facts a verifier can compare. A rebase is
    charged what it examines, as the decision that read the same facts was.
    It fails as a whole, and a commit whose rebase failed retains none of
    its facts: one left as its handler read it has no native revision. That
    commit carries one answer out: whether its own effect moved a fact its
    source query read. The comparison a successful rebase makes of those
    facts decides it, and a read the walk could not decide counts as moved.
    A commit that moved none is current while its own publication is the one
    selected. A commit that moved one, and any commit without facts at a
    later publication, is superseded: it refreshes on the required wave, as
    a row whose facts read stale does. The row is one without facts, never
    exact: a later demand selects the Preserve posture. Where the family
    installs a Preserve producer that demand settles on the commit while it
    is current and runs the producer once it is superseded; where it
    installs none the demand is refused `MissingApplicableProducer` before
    any effect: `checkpoint_recovery/input_cutoff/unrebased_settlement.rs`.
  - An equal republication of a root clears the marks below it. Demanding
    the last consumer first settles the chain in one advance each, with one
    source query, no producer contact and no decision.
  - A consumer of an equally republished root decides again when the root
    then changes, instead of waiting forever on the older equal row.
  - A settled demand that stays open keeps following its output: through a
    refresh of its equal republication under the same source, through a
    refresh whose delivery has not arrived when the next commit lands, and
    through a demand of a newer source that closes or is superseded before
    it publishes. That demand's row gives the occurrence back to the newest
    Ready it replaced, for a fresh demand as for a refresh. Rows of one
    source order by their refresh, so the refreshed alias is the newest, in
    whichever order the held demands advance. A never-settled demand that
    joined the undelivered refresh ends that row when it stops, and the
    settled holder moves on from the ended row with no predecessor,
    whichever of the two advances first:
    `exact_invalidation/undelivered_refresh.rs`,
    `a_never_settled_stop_leaves_an_undelivered_refresh_to_its_settled_holder`.
  - A held chain left unadvanced while its root is edited, until the window
    no longer retains that edit's mark, follows the edit in one advance per
    demand, in any order. A settled demand follows its output to the newly
    selected source inside the same advance, which reads the retained source
    once more.
  - A demand that never settled stops `Superseded` when a commit replaces
    its source; its caller demands again. The stop ends only that demand's
    interest: a committed Ready stays for its settled holders, which follow
    their output whether they advance before or after the stop.
  - A native Relational writer that is not Query is delivered exactly and
    marks what the same write through a declared operation marks, for a root
    input and for a chain output. CI's featured run drives it.
  - A demand at a retained observation older than the head ignores later
    commits. It settles free on the output current at its own snapshot,
    before and after the head settles on a later edit, holds no registry row
    and decides nothing; the head demand settles on the new output. It stops
    `Superseded` only when no retained output verifies current at its
    snapshot. Output history below the retained window hands a reader the
    older pinned generation in place of a freed one; that reader verifies it
    in full at its own snapshot and never settles on it.
  - Rules the neutral fixture cannot reach are proven at their owner. Paths
    are under `worth-query-execution/src/domain_computation/primary_graph`
    unless they name a crate.
    - Delivery overflow. A commit whose marking exceeds the ceiling
      publishes under a new delivery epoch; every reader then verifies in
      full, and earlier snapshots stay clean:
      `output_lineage/invalidation/native_journey_tests/marking_ceiling.rs`,
      `matched_fan_out_above_the_ceiling_publishes_and_readers_fully_verify`.
      A fixture fact has at most three readers, and a ceiling that low also
      refuses settlement registration.
    - Undeclared change. A commit delivered without touch keys starts the
      same epoch: readers registered before it verify in full, an earlier
      snapshot stays clean, and a reader registered after it is exact:
      `output_lineage/invalidation/native_journey_tests/undeclared_change.rs`,
      `a_commit_delivered_without_touch_keys_starts_a_fully_verified_epoch`.
      The proof loses the keys of a commit whose selectors exceed
      preparation memory. A Relational schema transition, which changes an
      aspect contract revision, commits without touches and reaches delivery
      the same way; the installed schema fixes the revision, and Query
      issues no schema transition.
    - Program adoption preserves settled outputs. A row is keyed by producer
      identity and source epoch, and a runtime installs one producer under
      an identity, so two programs cannot supply different producer code
      under one row key. A clean output settled before the adoption answers
      a demand under the adopted program with its original commit and no
      producer contact, and input cutoff keys reuse on the installed
      producer edition, with no program:
      `worth-query-certification/tests/application_graph/adoption/live_outputs.rs`,
      `a_clean_output_settled_before_adoption_answers_under_the_adopted_program`.
      A changed source is produced by the adopted program under its own
      commit, and the carried instance refuses the assessment settled before
      the change: `a_changed_source_is_produced_by_the_adopted_program`.
      Adoption commits as an ordinary transaction with exact touches, and a
      demand typed on a program its branch no longer runs stops
      `PublicationStale`. The courtroom fixture installs one program.
    - Retained capacity. The versions of a branch's marks are persistent
      and share every node their own edits did not copy, so a reservation
      follows the allocation that owns the bytes. A version reserves what
      its edits copied, never more than its whole index, and the root
      reserves what its oldest version shares with versions that have left.
      A window of deliveries over one row holds that row once, released
      with the last version that held it:
      `output_lineage/invalidation/native_journey_tests/shared_versions.rs`,
      `versions_sharing_a_row_reserve_it_once_and_release_it_with_the_last`.
      No history reserves more than one whole index per retained version.
      An index with no room for a registration, or for the delivery of a
      producer's own commit, stops that advance `RetentionBudgetExceeded`.
      Retention is the budget of the advance that met it, never the
      producer's failure, so the row stays claimable. The stop is terminal
      for its caller, as retention is wherever no advance or close of a
      demand frees the room: index room returns as later commits move the
      window, and a later claim of the row then settles:
      `checkpoint_recovery/required_chain/required_queue/exhausted_index.rs`,
      `an_index_too_small_for_a_commit_stops_the_advance_for_retention`.
    - Row lifetime. The lineage owns it. Every publication retires the row
      of the generation its record displaces, whether or not a demand
      manages it, and the lineage never names that generation again, so the
      invalidation owner keeps a released row it could not retire and
      retries it at the next release:
      `output_lineage/invalidation/native_journey_tests/displaced_generation.rs`,
      `a_publication_without_prerequisites_retires_the_row_its_generation_displaces`
      and `a_release_the_owner_could_not_admit_retires_at_the_next_release`.
      Every courtroom publication belongs to a demand. The public consumer
      journey commits program actions that none manages, and CI runs it
      beside the courtroom.
    - Branch merge. Product history is single-rooted and never merges, so
      no output demand meets one. A native Relational merge emits exact
      touches: `worth-relational/src/tests/history/milestone_7d_phase_d/merge_descriptive_touches.rs`,
      `native_merge_touches_only_changed_revision_and_both_index_membership_keys`.
      The facade offers no merge.
    - Un-indexed predicate. It is a root-path guard, witnessed by field and
      adjacency facts: `tests/application_query/root_guard_basis.rs`,
      `root_path_guard_reads_its_pinned_truth_version`, and
      `tests/application_query/root_selection/result_set.rs`,
      `empty_path_union_stales_when_a_matching_edge_is_inserted`. An output
      demand's source is one owner-paired row selected by an indexed
      equality, so the fixture declares no guard.
    - Stored absence and selection facts. Each posts under the old and the
      new index key and the absent field's address:
      `output_lineage/invalidation/fact_keys/tests.rs`,
      `indexed_selection_matches_old_key_and_definition_touches` and
      `absent_field_and_native_revision_share_exact_addresses_after_admission`;
      `tests/application_query/root_guard_basis.rs`,
      `empty_indexed_root_set_stales_when_its_scoped_guard_becomes_a_match`.
      An output demand stores single-row source witnesses only; selection
      facts come from workflow discovery, and the fixture declares none.
  - Accepted postures:
    - A fresh demand on a forked branch executes its producers again. A row
      is keyed by its branch incarnation, so the fork has no row and starts
      Fresh; values agree.
    - A chain node's written value does not depend on what it consumed: its
      handler writes back the field its own source query reads. The
      courtroom varies what a consumer decides over by overwriting its
      upstream output.
    - A consumer that executes and writes an equal value does not cut off
      its dependents. Equality is certified by input cutoff only.
    - No public observation counts fact checks or fallback events. The
      courtroom reads source-query runs and producer contacts.
  - Stated limitations:
    - A version pinned without its root past the window is covered only for
      its own copies; its holders are call-scoped or certification-only.
    - The registry index can lag the owner after an unmanaged retirement
      until the next release: a late free, never a wrong answer, since the
      owner answers `MissingSettlement`, which forces full verification.
    - Replacing a root's reservation for what its oldest version shares
      reserves the new amount before the old one frees, so a delivery
      transiently needs up to one index of free room. A refusal there
      defers the writer.
    - `RetiredOutputEntity` is not an exact fact key: a row that decided on
      one verifies in full under exact invalidation.
    - Republication does not retire the displaced generation's mark row.
    - A same-runtime restoration whose settlement could not be registered
      is not verified in full and reused: its demand runs the family's
      Preserve producer once. In a family that installs no Preserve producer
      that demand is refused `MissingApplicableProducer`, and the refusal
      persists while the output lives.
    - A commit whose own effect grows a selection it read past the declared
      width of that selection cannot observe it again complete, so its
      rebase fails. A handler inside its declared budgets can therefore
      commit facts that do not rebase, and a producer whose every commit
      does so never leaves a row with facts: it runs again after each later
      publication. No fixture has such a producer.
    - A rebase observes a selection before it charges for what the
      observation examined, so one observation past the budget runs before
      the rebase is refused. Each observation stays capped by its own
      declared candidate limit.
    - Any later publication on the branch, even an unrelated one, supersedes
      a settlement without facts: nothing is left to compare with it. A
      publication between the commit and its certification therefore makes
      the committing demand refresh: the Preserve producer runs, or the
      demand is refused `MissingApplicableProducer` in a family that
      installs none.
    - A later demand that settles on a commit without facts is admitted
      under the Preserve producer first: the provider is asked its demand
      resources, and nothing runs.
    - A workflow assessment binds the facts of the output it assesses and
      compares them again at its transition. A settlement without facts
      gives it none, so the assessment answers
      `WorkflowAssessmentEvidenceMismatch` until a commit that rebases
      refreshes the row.
    - Query's own delegation activation reads nothing about the child
      grant, so a second activation for an existing child id commits. The
      bank handler reads the child's absence. The platform fix changes a
      declared contract and is tracked as its own task.
    - An exact selection reports its producer's first declared posture, also
      for a producer that declares Preserve and then runs over the live
      output. The reported posture selects no behavior.
    - A checkpoint-restored output that is suspended and restored executes
      once under Preserve, where a full verification would reuse it.
- Checkpoint readmission rebuilds the original complete Native output-aspect
  witness from captured facts. *Completed:*
  - It verifies those original revisions and supported producer facts against
    the selected World before creating restored Ready authority.
  - Missing, unsupported or changed evidence follows Fresh, and admission
    exhaustion keeps its resource denial.
  - Public Current may consume the verified restored witness, but fact-only
    restoration is not enough.
  - The owner proof changes output content while a producer field stays
    current, and the combined verifier must reject the old witness.
- An admitted demand keeps the immutable producer entry chosen by the existing
  selector. *Completed:*
  - Advancement consumes that entry after the exact Interest-to-Ready join, with
    no second search by its copied name.
  - Successor transfer is admitted only after the actual successor passes its
    joins, and a Clean result pays its scalar contact reset.
  - The displaced certification methods and the conversion-only cue adapter are
    removed.
  - Fresh request, principal and policy admission stay on the production path.
- Source fact preparation resolves each observed field's installed contract
  once. *Completed:*
  - A private prepared materialization carries the validated field
    observations and admitted storage into conversion, and conversion consumes
    that proof without another layout lookup.
  - Typed native locators keep first-occurrence fact order and
    duplicate-conflict semantics, and Work follows each locator's own
    initialized comparison bytes.
  - Publication witness preparation stays under the same request admission.
- Rename the Signal `Touched` observation tier to `Visited`, with one
  spelling: no serde alias and no decoding of the pre-rename policy schema.
  *Completed.*
- Restore the public definition, rules and vocabulary in the same change, and
  point the 9.17.6 plan here. *Completed:* the rules are in
  [How WORTH Works §10.5](../../docs/how-it-works.md#105-marking-and-currentness)
  and the glossary.
- A neutral application proves the exact-invalidation courtroom with operation
  counts, locality evidence and the randomized differential test.
  *Completed:* the courtroom above. Its oracle and seeded sequences run
  through one neutral fixture, the topology entry.

Work admission and capacity follow these rules. *Completed.*

- Work admission counts named operations and bounded comparison or
  initialized-copy payloads. It does not claim to count machine instructions.
  Constant state transitions use their owner's fixed operation granule.
- A borrowed carrier does not pay for hypothetical moves. Destruction visits
  initialized elements, not unused Vec capacity.
- Conditional continuation installation reserves its maximum on the original
  request meter before Fresh effects and settles only the installation it
  reached. The other branches refund that reservation without refunding
  nested work.
- Prepared and retained memory stay separate capacity claims, each with its
  actual lifetime.
- Static diagnostic subjects are borrowed and need no speculative String
  allocation.
- The contact fixture's 4,096 producer allowance stays artifact policy. Source
  currentness has its own host-bounded allowance. Exact exhaustion and locality
  evidence establish the accounting and scaling contract, not a constant fitted
  to a fixture.

The next phase may trust that the touched graph alone decides what recomputes.

### Phase 6: Partitioned managed computations

- Change the partitioned declaration as described, and re-baseline revision
  digests. *Completed:* the key contract, the single computation partition,
  the determinism contract and the validation denial are declared, and the
  topology entry proves them through the public facade. No stored digest
  existed to re-baseline.
- Make `DeterministicPartitioned` execute through the partitioned owner binding.
  *Partly completed:* a partitioned owner installs and runs: key derivation,
  routing and gathering in `prepare`, every partition's kernel through the
  execution map, the reducer over the canonical tree, and `complete`. Every run
  recomputes every partition, inside the computation's declared work.
  Installation refuses a mismatched owner binding and every `ContractEquivalent`
  declaration. The topology entry proves through the public facade the same
  bits and charged work on every run and in every input order, the least
  failing partition, the canonical work boundary, and a typed denial for a
  kernel panic, a reducer panic, a result over the declared bytes and a key
  that does not encode.
  *Not completed:* no lease reaches a managed computation. Query requests none,
  and Phase 7 carries the request lease to `compute`. Every run is the serial
  backend with fallback `NoLease`, so the proof against the serial oracle at
  two workers, machine width and wider than the machine is not made.
  *Limitations:* the reducer and `prepare` are not interruptible. The declared
  bytes bound each partition's result, not their total. `prepare`'s routing
  memory is uncharged without a lease. The floating-point sum proof compares
  runs with each other; the comparison against the serial oracle
  (`certify_reduce`) needs a lease.
- Route marks to computation partitions through the settlement row's routing
  table, with item routing so `prepare` re-gathers only marked partitions.
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
