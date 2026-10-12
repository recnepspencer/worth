# Performance Architecture Laws

## I. Core Laws

Execution breadth must be bounded by semantic delta plus the smallest honest physical or algorithmic granule. Every resulting read, write, invalidation, recomputation, retention, or flush amplification must be named, measured, and policy-bounded.

The touched graph is the commit's sealed semantic delta and the cause of invalidation. Consumed dependency facts intersect that delta through the reverse index: matched settlements become dirty, and their downstream consumers become pending-upstream. This affected closure bounds verification and recomputation on a continuous basis. Scope paths do not refine the touched graph: a sealed touch carries no scope path. They refine the changed regions the Bridge lowers from committed changes and delivers to Signal; declared widening and bounded fallback to full verification remain counted. A change to an unrelated record must not induce a scan of every retained result.

Authoritative commit work includes declared synchronous invariants and bounded invalidation delivery. Invalidation follows the touched graph through reverse-index postings and downstream edges; its installed ceiling bounds fan-out, and a delivery discontinuity requires later full verification without refusing the writer. Arbitrary projections, diagnostics and explanations must not become synchronous correctness work. Every derived structure must declare whether it participates synchronously in correctness or updates asynchronously, together with its write amplification and staleness contract.

Semantic intent, policy, topology, normalization, and dispatch narrowing must resolve before the hot path into the narrowest honest execution strategy. Runtime resolution is permitted only for facts unavailable upstream or themselves constituting the operation.

API shape must reveal traversal, reconstruction, synchronization, allocation, amplification, and coordination cost. Cheap-looking surfaces must not conceal broad scans, graph walks, reconstructive work, remote effects, or rich-path behavior.

Execution topology must represent dominant locality and traversal direction explicitly. Storage and access structures are selected from traversal frequency, update cost, density, and consistency requirements; every required traversal direction must have an explicit strategy and visible cost, but no particular index topology is universally mandatory.

Every reuse surface—caching, suppression, memoization, diffing, or incremental recomputation—requires stable identity, canonical ordering, comparator semantics, and a precise invalidation basis sufficient to justify sameness as a semantic claim.

Boundary cardinality must match domain cardinality. Bulk domains must expose bulk execution rather than externalizing scalar loops that fragment work and destroy amortization.

Preserve expensive proof within a trust boundary when carriage costs less than rediscovery; revalidate only when authority, ownership, version, or trust changes, or when proof maintenance would cost more than reconstruction.

Breadth, richness, coordination, and quality may degrade only by explicit policy. Degradation must identify the triggering budget, retained guarantees, weakened guarantees, and recovery posture.

Structural waste dominates constant waste. Remove scope leakage, amplification, repeated maintenance, path conflation, projection inflation, repeated rediscovery, and avoidable coordination before tuning capacities, layouts, or instruction-level constants.

A performance claim is valid only at its named boundary and must carry counters explaining the work performed. End-to-end claims require end-to-end measurement; slope claims require scale-sensitive evidence across every independent input axis.

A performance claim must name its workload distribution, scale axes, environment, hardware and runtime configuration, cold or warm posture, repetitions, variance, structural counters, and reported percentiles. Evidence may not transfer across an unmeasured workload, scale regime, execution lane, or environment.

Mechanical layout may co-locate data evaluated together without collapsing semantic ownership. Hot local handles should lower to flat, generational, or arena-indexed addressing when admission permits it; durable, external, or authority-bearing identity remains distinct. Pointer chasing, hashing, translation, and indirection are counted costs, not prohibited representations.

Allocation must have a named lifecycle and bounded hot-path budget. General-purpose allocation, fragmentation, reclamation, initialization, and cross-thread ownership are coordination costs; operational paths use preallocated, pooled, arena-scoped, or explicitly budgeted storage matched to the dominant lifetime.

Parallel throughput is bounded by structural independence and coordination depth, not core count alone. At the execution layer, work sums charged operations; span adds dependent stages and takes the maximum of independent branches. Nested computation contributes both measures to its calling branch. With sufficient memory and no in-flight interruption, placement must preserve results, charged work and structural span. The execution oracle runs one admitted batch through serial and selected placement and compares values, the stop, charged work and charged span. Structural span describes the computation even when placement is serial; it is not elapsed time. Declare independent membership before placement and keep owner application in canonical order. Shared mutation, cross-thread reference counting, atomics, locks, barriers and queue handoffs are explicit costs; scalable plans carry disjointness, partitioning or a measured synchronization budget.

A producer wave carries member denials as data. When canonical owner completion or application keeps only a prefix, every member of that wave that computed remains charged. A request or execution stop instead retains canonical-prefix accounting and reports discarded work as a physical metric. Owner work is charged where it runs; waves never dispatched cost nothing. A managed computation's carried partition replays its accepted kernel charges and structural span, so reuse preserves the full-run bill and the work-ceiling boundary. Use one accounting owner for carried and recomputed work. Memory is physical: a carried partition reserves what it holds, so a memory denial may depend on reuse and placement. Diagnostics must not change the request's bill.

Threads must be created by the physical execution authority or at a production site listed by category in the boundary-check configuration. The check rejects unlisted production thread creation, pool construction and Rayon use outside Execution. Each listed site declares its category, reason, source item and exact count; growth or a stale declaration fails the check. A target without OS threads uses bounded serial execution with the same work and span accounting, without constructing a worker pool.

Rejection must precede expensive construction. Evaluate the cheapest, most restrictive disqualifying constraints before allocation, topology traversal, coordination, or rich intermediate construction.

Every cost belongs to a named lane: ordinary, cold-start, recovery or reconstruction, migration or compaction, diagnostic, or background maintenance. Work may move between lanes only by explicit policy; amortization and background execution do not erase cost, and deferred queues must remain bounded.

Latency and throughput contracts must declare arrival rate, burst model, service distribution, utilization envelope, headroom, queue bounds, and percentile or worst-case posture. Mean latency under unsaturated load cannot support a tail-latency or capacity claim; overload must reject, backpressure, or degrade before queues become unbounded.

Logical work and physical amplification are separate accounting domains. Boundaries must expose relevant bytes read and written, syscalls, durability barriers, messages, retries, compaction work, cache fills, and device or network round trips. Durability ordering and recovery work may not be optimized out of the claimed cost.

Memory contracts must distinguish transient allocation, peak footprint, retained state, resident working set, and reclaimable projection. Page faults, eviction, cache misses, garbage collection, and reclamation are counted work. An operation with bounded immediate memory but unbounded retained or queued state is not memory-bounded.

## II. Tradeoff Laws

Choose batch, incremental, rebuild, or desired-state-diff execution by total invariant cost across observability, density, memory, latency, and update breadth. Strategies must expose their profitable regime and a governed transition when density or maintenance cost crosses it. Under desired-state diffing, producers declare truth while the framework owns comparison, patching, and suppression.

Ownership multiplicity must equal observer multiplicity. Move by default; clone only to establish a named concurrent-observer, temporal, isolation, or retry boundary.

Partition boundaries are profitable only when gains in locality, ownership, or isolation exceed communication and coordination cost.

Speculative reflection of effects is a latency strategy, not a throughput law. Speculate only when divergence, rollback, reconciliation, and authority-lag costs remain below the latency cost of waiting for authoritative truth.

## III. Named Failure Modes

Scope leakage: Invalidation, recomputation, retention, or flushing exceeds semantic delta and its declared operational granule.

Projection inflation: Derived, cached, indexed, explanatory, or diagnostic structures are maintained as authoritative truth without a synchronous correctness contract.

Plan/execute conflation: Execution reconstructs decisions that should have been resolved during planning, lowering, or normalization.

Boundary cardinality mismatch: Scalar orchestration is imposed over bulk semantics.

Per-edit structural maintenance: Topology, indexing, subscriber state, bookkeeping, or diagnostics are maintained per sub-edit inside a semantically atomic operation.

Equivalence drift: Reuse exists without stable sameness semantics, canonical order, comparator discipline, or precise invalidation.

Path conflation: Ordinary paths inherit diagnostic, forensic, explanatory, reconstructive, migration, or maintenance cost.

Breadth-coupled coordination: Local intent induces deep synchronous propagation or broad coordination inside one operational boundary.

Repeated rediscovery: Later phases re-filter, re-prove, or re-derive facts already established inside the same trust boundary.

Mechanical/semantic collapse: Mechanical optimization corrupts semantic ownership, or semantic layering conceals mechanical cost.

Amplification blindness: A small logical delta induces undeclared physical reads, writes, messages, retries, or compaction.

Saturation collapse: Average throughput hides nonlinear queue growth and tail latency near capacity.

Working-set escape: Retained state exceeds its residency budget and exports cost into faults, eviction, reclamation, or garbage collection.

Background-cost laundering: A foreground path appears cheap by exporting unbounded work to maintenance queues.

Benchmark scope laundering: A measured claim is applied to an unmeasured workload, scale, environment, percentile, or cold or warm regime.

## IV. Operational Review Vector

Judge disputed designs by semantic scope, execution and amplification breadth, synchronous maintenance surface, equivalence basis, locality, boundary honesty, coordination depth, proof strategy, lifecycle fit, density regime, authority model, and measurement boundary.
