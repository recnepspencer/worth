# Milestone 13 Engineering Spec: Locality-First Frontier Execution

> **Status:** Completed
>
> **Closeout:** [milestone-13-closeout.md](./milestone-13-closeout.md)
>
> **Prerequisite:** [milestone-12-plan.md](./milestone-12-plan.md) and its
> [closeout](./milestone-12-closeout.md)
>
> **Architecture parent:** [signal_architecture2.md](./signal_architecture2.md),
> `S9.16.3` and `S9.16.6`
>
> **Successor:** [milestone-13.1-plan.md](./milestone-13.1-plan.md)

## 1. Goal And Roadmap Placement

Milestone 13 makes invalidation breadth scale with the realized semantic
frontier plus the smallest declared indexed candidate/order granule, rather
than the complete reachable subscriber closure or all direct subscribers.
Index probes and candidate-edge examination are counted work and may not be
relabeled semantic work.

Milestone 12 established the semantic authority that Milestone 13 must carry:

- root mutations create unresolved source-recompute obligations
- only an immediate producer's performed output commit creates a downstream
  aspect/scoped dependency cause
- causes remain bound to graph instance, consumer, dependency revision,
  producer, aspect, edge scope, cached version, output ordinal, and committed
  version
- direct source bases and dependency causes remain distinct authoritative
  state
- financial truth and necessity have independent oracles

Milestone 13 turns that authority into a compiler-enforced direct-hop work
progression. It removes structural transitive pre-marking, establishes one
canonical ready-work boundary, records realized work at the boundary where it
happens, and certifies sparse and dense cost slopes in the authentic financial
world.

Together, Milestones 12 and 13 close `S9.16.3`. There is no later
invalidation-certification milestone. Milestone 14 may parallelize only the
work stream sealed here.

## 2. Current Boundary

The completed Milestone 12 runtime is semantically correct but deliberately
does not claim transitive locality. The current root invalidation lane still:

1. plans matching direct subscribers
2. marks each direct subscriber pending revalidation
3. walks every transitively reachable subscriber
4. marks every reached descendant pending revalidation
5. reports transitive reachability and visits as execution work

The structural walk carries no aspect or scope authority, so it can no longer
invent incorrect descendant meaning. It can still perform work proportional
to a broad reachable graph when only one narrow chain can produce a meaningful
change.

The current `FrontierPlan` and `FrontierExecutionSummary` are public summary
forms with public constructors. `FrontierPredictedCounters` and several
execution counters still describe the inherited reachability algorithm.
`PreparedDirectCauseAdmission` is owner-local and correctly computes immediate
consumer causes, but it does not yet lower through a proof-carrying ready-work
phase family.

The decisive gap is therefore not another aspect fix. It is the absence of a
single enforced progression from performed immediate-dependency truth to
current, ordered, unique, executable work.

## 3. Adversarial Financial Courtroom

Milestone 13 expands the same `FinancialWorldDefinition` and
`CompiledFinancialWorld` authority established by Milestone 12. A generated
generic graph, a raw `SignalGraph`, or a fixture that derives expected work by
calling production routing is supplemental evidence only.

### 3.1 Required Scenario Families

| Scenario | Production-valid world and independent scale axes | Defect it must convict |
|---|---|---|
| `sparse_book_fanout` | one quote-to-valuation-to-risk chain of depth 16 remains economically relevant while direct adjacency and rejected transitive fanout vary independently across `PRICE`, `FX`, `CURVE`, `VOLATILITY`, audit, and reporting subscribers at `10^3`, `10^4`, and `10^5` total nodes | complete subscriber-closure walking, late aspect filtering, edge scans hidden as semantic work, or visit counters that omit rejected branches |
| `partitioned_curve_universe` | one rates bucket changes among 16, 256, and 1,024 independently declared curve/credit regions while overlap density and instruments per region vary separately | partition widening, global region scans, scope cross-products, or rejection after dirty mutation/enqueue |
| `convergent_factor_batch` | price, FX, curve, and volatility commits converge on one portfolio aggregate through distinct dependencies, duplicate causes, seed-order permutations, and repeated same-epoch admissions | lossy node-only deduplication, duplicate evaluation, or provenance reconstructed after merge |
| `dense_market_close` | lawful sparse, medium, and dense market-close frontiers at `10^3`, `10^4`, and `10^5` nodes, with the semantic density axis varied independently from graph size | a strategy or proof that works only for sparse graphs, drops necessary dense work, or hides queue/memory amplification |
| `portfolio_dependency_churn` | instruments move between desks, factors, curves, and pricing models between commits; same-shaped edges are removed/recreated; rejected cycles and accepted rewires are interleaved with pending work | stale ready work, wrong dependency revision, topology-order reuse, or uncounted revalidation/churn work |
| `branch_restore_locality_replay` | the same narrow, convergent, and dense traces run after branch capture, checkpoint restore, supported readmission, replay, and deterministic rerun | persisted derived queues, current-proof reuse after a trust boundary, nondeterministic order, or work laundered into reconstruction |

### 3.1.1 Normative World And Counter Model

The six names above are not permission for scenario-authored graph fixtures.
Every case begins with a `FinancialWorldDefinition` whose books, desks,
positions, factors, region ownership, pricing/risk model assignment,
subscriptions, conditions, and output policies determine the compiled graph.
Scale generators may repeat economic structures, but they may not append raw
`NodeId`, aspect-slot, scope-token, edge, dirty-state, ready-work, or counter
facts after compilation. Compiler-issued semantic handles are the only handles
the runner may retain.

Before production invalidation runs, the independent locality owner derives
the following values from the immutable definition, named mutation, and
declared execution posture without consulting the production reverse index,
routing, scheduler, dirty state, ready queue, trace, or performed receipt:

| Symbol | Independent meaning |
|---|---|
| `Q` | the exact set of reverse-index bucket keys implied by every performed producer delta in the trace |
| `C` | the ordered multiset of authoritative direct dependency declarations belonging to `Q` |
| `K` | the canonical distinct dependency causes that should survive M12 admission |
| `U` | the canonical distinct `(graph, target, dependency revision, readiness epoch, stage)` work identities after lawful merge |
| `E` | the exact semantic outputs that must evaluate, from `FinancialNecessityManifest` |
| `S` | the exact propagation stops caused by unchanged producer output |
| `P` | the expected maximum ready width derived from the declared stage/order graph and hostile release schedule |

For an exact detail delta, `Q` contains exactly the producer/aspect unscoped
bucket, the matching whole-partition bucket, and the matching detail bucket.
For a whole-partition delta, `Q` contains the unscoped bucket, the matching
whole-partition bucket, and every declared detail bucket in that partition.
For a whole-aspect delta, `Q` contains the producer/aspect broad bucket family.
The manifest canonicalizes repeated keys before counting probes.

The decisive receipts must satisfy, for the complete named trace:

- reverse-index bucket probes equal `|Q|`
- reverse-index candidates returned and direct subscriber edges examined equal
  `|C|`
- all candidate rejection rows partition `C - K`; no candidate disappears
  between discovery and a named admission or rejection outcome
- performed admission evidence and the named pre-settlement storage checkpoint
  contain exactly `K`, including every
  producer, aspect, scope, snapshot version, output ordinal, committed version,
  consumer, and dependency revision
- the named post-lowering/pre-execution checkpoint contains exactly `U`; later
  execution may lawfully consume it but must retain its identity in the
  performed receipt
- evaluated semantic output identities equal `E`
- propagation-stop identities equal `S`
- maximum ready-frontier width equals `P` for the declared hostile release
  schedule
- ready enqueue/pop, merge, stale/rebind denial, topology revalidation, and
  allocation rows equal the scenario-specific expectation manifest rather than
  a tolerance chosen from observed values

The expectation manifest declares its observation points. `K` is compared
after all named producer commits and before the affected target settles; `U`
is compared after lowering/readiness admission and before execution; `E`, `S`,
committed financial truth, and total counter rows are compared after the trace
settles. Reading a cause before all commits arrive or after lawful release, or
reading a ready batch after lawful consumption, cannot satisfy the contract.

Small decisive cases retain a Development-tier identity sidecar for `Q`, `C`,
`K`, `U`, `E`, and `S`; counters alone cannot prove that the right nodes were
visited. The sidecar is derived from performed events, never accepted as
operational authority. Scheduled scale cases may retain canonical digests and
cardinalities instead of every identity, provided an ordinary-size twin proves
the same generator and expectation rules with full identities. Operational
counters and committed truth must be identical across diagnostics tiers.

### 3.1.2 Three Different Slopes

The courtroom reports three slopes separately. Combining them into one
"fanout" number is forbidden:

1. **Index-disjoint slope.** Add `x` valid direct dependencies outside `Q`
   while holding the mutation, queried-bucket membership, and `E` fixed.
   Bucket probes, candidates returned, edges examined, non-semantic visits,
   dirty mutations, ready work, and evaluations must have exact delta zero.
2. **Queried-candidate slope.** Add `x` valid but rejecting dependencies inside
   a queried bucket while holding `K`, `U`, and `E` fixed. Candidates returned
   and edges examined must increase by exactly `x`, the applicable rejection
   row must increase by exactly `x`, and admitted causes, ready work, and
   evaluations must have exact delta zero. This is the smallest lawful indexed
   candidate granule and must not be misreported as semantic work.
3. **Semantic-frontier slope.** Add `x` economically affected declarations so
   that the independent manifest expands `K`, `U`, or `E`. Performed work must
   grow by the corresponding exact manifest delta. A locality optimization may
   not keep the graph cheap by dropping necessary dense work.

Wall-clock time is supplemental. These structural deltas, not a fitted timing
threshold, decide pass or fail.

### 3.1.3 Scenario-Specific Contracts

| Scenario | Frozen construction and hostile action | Exact independent observations | Assigned red mutations |
|---|---|---|---|
| `sparse_book_fanout` | Compile one economically necessary depth-16 quote/valuation/risk chain. Vary separately: dependencies on disjoint producer aspects, dependencies in the queried bucket that reject by current contract/comparator, and descendants below those rejected direct consumers. Keep the relevant chain and mutation identical at every scale. Every added node must have valid financial/audit/reporting ownership and a compiler-declared dependency; disconnected padding is forbidden. | `E`, `K`, and `U` for the depth-16 chain are identical across irrelevant scales. Index-disjoint additions and rejected descendants have zero delta for every hot-path row. Queried-bucket rejecting additions produce the exact `+x` candidate/examination/rejection delta and zero semantic-work delta. Identity evidence proves no transitive-only descendant was visited. | restore subscriber-closure walking; query the producer-wide subscriber list; filter aspect after mutation; omit rejected candidates from counters; replace connected fanout with disconnected padding |
| `partitioned_curve_universe` | Compile 16, 256, and 1,024 independently owned curve/credit regions. Mutate one declared curve detail. Vary disjoint-region count, matching-partition overlap density, and instruments per matching region independently. Include an `A@x`/`B@y` correlated-scope twin and whole-partition/detail twins. | An exact detail change probes exactly unscoped + matching partition + matching detail keys. Disjoint regions have zero hot-row delta. Adding `x` rejecting members to a queried scope bucket gives the exact candidate slope. `K` retains exact aspect/scope pairs; no mask-by-flat-scope cross-product appears. No dirty mutation or enqueue occurs for a rejected scope. | scan all regions; flatten aspect/scope correlation; enumerate unrelated detail buckets; retain only the last detail; reject scope after dirty mutation or enqueue; trust a drifted derived index over topology |
| `convergent_factor_batch` | Commit price, FX, curve, and volatility changes through four distinct immediate producers into one portfolio target in every seed-order permutation. Repeat identical same-epoch admissions and include two causes sharing a numeric aspect slot but not producer identity. Hold the economic mutation set fixed. | The target's canonical cause set has exactly the four distinct bindings. Repeated identical causes add no cause identity. All successful lowerings for the same target/epoch/stage produce one `U` entry; merge count equals successful same-identity lowerings minus one; the target enqueues, pops, and evaluates exactly once. Every permutation yields the same cause/work digest and committed truth. | node-only or aspect-slot-only deduplication; last-cause-wins replacement; duplicate evaluation; provenance reconstructed from graph edges after merge; seed-order-dependent canonicalization |
| `dense_market_close` | Using the same economic generator, hold total compiled graph size fixed while selecting sparse, medium, and dense lawful mutation sets, then repeat each density at `10^3`, `10^4`, and `10^5` total nodes. Stage widths and expected affected positions are declared before execution. | `E` exactly equals the independently derived dense necessity set; no necessary work is dropped. Candidate, cause, work, evaluation, stop, and peak-width rows equal their manifests. At fixed graph size, growth follows the semantic-frontier delta rather than total reachability. At fixed density, scale growth is reported structurally; memory/item envelopes use ready-item and allocation rows, never wall-clock alone. Both mechanical strategies consume the same `U`. | sparse-only shortcut; fixed work cap; silently dropped dense work; full-graph walk relabeled as evaluation; queue-width or allocation rows omitted; strategies compared on different admitted streams |
| `portfolio_dependency_churn` | Establish pending work, then move instruments among declared desks, factors, curves, and pricing models through the production mutation authority. Remove and recreate a same-shaped edge, interleave an atomic rejected cycle, and admit current work after the accepted rewire. The trace records every performed topology receipt and exact affected owner. | Expected dependency-revision and topology-epoch deltas are derived from accepted mutation receipts, not guessed constants. Pre-rewire ready work is stale/rebind-required and cannot execute. The rejected cycle changes no topology, revision, cause, queue, or counter authority. Current `K`, `U`, `E`, revalidation rows, and final truth match the post-rewire definition; old causes and snapshots are absent by identity. | keep stale ready work current; compare edge shape without revision; advance revision on rejected cycle; reuse old topology order; omit revalidation/churn rows; let rejected topology mutation enqueue work |
| `branch_restore_locality_replay` | Run ordinary-size narrow, convergent, and dense traces with unresolved authoritative source/cause state and derived ready work present immediately before capture. Cross branch capture, checkpoint image, supported fresh-runtime readmission, replay, and deterministic rerun. Destroy all derived indexes, queues, summaries, and receipts before reconstruction. | Exact authoritative source/cause fingerprints survive where the lifecycle promises; pre-boundary ready identities never execute as current. Reconstructed `Q`, `K`, `U`, `E`, committed truth, and canonical order equal the corresponding cold run. Recovery/index-rebuild/replay rows appear only in their named receipts; ordinary hot rows match the cold operational twin. Replay equivalence compares canonical event association/detail, not event kinds or counts alone. | serialize/restore ready authority; reuse old graph/revision/epoch binding; lose or substitute one pending cause while preserving cardinality; count rebuild/replay as hot work; compare only final values, event kinds, or cause counts |

Each runner owns a typed scale tuple and mutation trace for the construction
above. A runner must fail during baseline sealing if the compiler-generated
topology, dependency snapshots, semantic handles, or economic ownership do not
match the declared contract; removing subscriber edges during automatic
dependency capture is therefore a setup failure, not a passing locality result.

### 3.1.4 Frozen Scale Tuples And Lane Budget

Scale values are data in the Foundational case identity. A runner may add a
smaller debugging case, but it may not replace or silently resize these cases:

| Scenario | Ordinary change-gate tuple | Scheduled tuple |
|---|---|---|
| `sparse_book_fanout` | exact compiled totals `N = 64, 512, 4,096`; run separate index-disjoint, queried-rejecting, and rejected-descendant variants with the depth-16 relevant chain unchanged and all `N - 16` remaining nodes assigned to the named axis | `N = 1,000, 10,000, 100,000` for each axis variant |
| `partitioned_curve_universe` | region count `R = 16, 256`; matching queried-bucket memberships `M = 1, R/16, R/4`; one instrument per disjoint region plus a separate instruments-per-matching-region twin `I = 1, 8` | `R = 1,024`, `M = 1, 64, 256`, and `I = 1, 8, 32` |
| `convergent_factor_batch` | all `4! = 24` producer-commit permutations with identical-admission retry count `D = 0, 1` | all 24 permutations for `D = 8` across at least 16 canonical seeds |
| `dense_market_close` | exact compiled totals `N = 1,000` at affected semantic-output ratios `1/100`, `1/4`, and `4/5`; each ratio must divide the generated addressable set without rounding | `N = 10,000` and `100,000` at the same three exact ratios |
| `portfolio_dependency_churn` | `R = 8` full churn rounds; every round contains one accepted owner/model move, one accepted same-shaped remove/recreate, one atomic rejected cycle, and one current post-rewire mutation | `R = 256` rounds across at least 16 canonical seeds |
| `branch_restore_locality_replay` | narrow, convergent, and dense (`N = 1,000`, ratio `4/5`) posture twins, each run cold and through branch/checkpoint/readmission/replay | the narrow and convergent scheduled tuples plus dense `N = 10,000`, ratio `4/5`, across at least 8 canonical seeds; the `100,000` restore twin is a retained benchmark artifact, not an ordinary merge gate |

`N` means the exact compiled semantic-output node count recorded by the world
compiler, not requested padding. If the economic generator cannot construct a
declared tuple without violating ownership or subscription invariants, the
case fails construction; it is not rounded to a convenient graph size.

The scheduled lane records seed, elapsed time, peak memory, and retained
artifacts for diagnosis, but only structural manifests decide correctness and
locality. Retry or resource failure produces a typed incomplete scheduled case
and cannot be omitted from the report as though the scale passed.

The ordinary change gate contains causally complete small instances of every
scenario, one sparse fanout slope, one partition slope, one convergent
permutation family, and one sparse-versus-dense twin. Scheduled lanes own the
largest scales and longer seed permutations. Correctness and the small-scale
structural law may not depend only on a scheduled lane.

Every case records a Foundational-canonical case identity over:

- scenario identity and schema version
- financial-world seed and scale tuple
- exact mutation trace and mutation step
- consumer comparator and producer output-equivalence policies
- diagnostics tier
- cold, warm, restored, or replay-derived posture
- expected semantic work identity
- expected counter-contract identity

Debug text, enum discriminants without a rule version, insertion order, and
elapsed timestamps are not identity sources.

### 3.2 Hostile Sequence

For each applicable scenario:

1. compile the immutable financial definition through the production world
   compiler
2. establish a causally complete evaluated baseline and dependency snapshots
3. capture the independent fresh-recompute result, necessity manifest, and
   locality expectation before invoking production invalidation
4. apply the named economic or topology mutation through the production world
   surface
5. commit producer outputs one hop at a time while recording the performed
   direct admissions, ready batches, evaluations, and output stops
6. vary irrelevant fanout, region count, semantic density, seed order, and
   topology churn independently
7. repeat the applicable trace after branch/checkpoint/restore/replay
8. compare operational truth, exact causal work, exact counter rows, and cost
   slopes through independent observations

### 3.3 Required Outcomes

- final incremental financial truth equals `FreshFinancialRecompute`
- evaluated semantic work equals `FinancialNecessityManifest`
- every downstream hop begins with a performed immediate-producer commit
- aspect- and scope-disjoint direct edges are rejected before dirty mutation
  and before ready enqueue
- no transitive descendant is visited merely because it is reachable
- exact-aspect and exact-scope changes probe only the corresponding reverse
  subscription buckets plus the producer's unscoped bucket; index probes and
  candidate-edge examinations are reported separately from admitted semantic
  work
- rejected transitive fanout beyond examined direct candidates adds no visits
- multiple same-target/same-epoch admissions produce one work item whose
  canonical cause set retains every distinct dependency cause
- a rewire, restore, or graph-instance change makes earlier ready work stale or
  rebind-required and unexecutable
- deterministic and optimized scheduling consume the same admitted work
  identity and commit equivalent truth
- ordinary execution excludes replay, report assembly, and forensic work
- recovery/replay reports their own work rather than adding it to the hot-path
  receipt

### 3.4 Mutation Sensitivity

The courtroom must turn red when any of these mutations is applied:

- restore the complete transitive subscriber walk
- move aspect or scope rejection after node-state mutation or enqueue
- allow a root source seed to construct a committed work item
- allow a prepared, unperformed output packet to construct a committed work
  item
- bypass the performed output-commit transition
- make scheduling re-read graph edges to invent or widen causes
- replace the canonical cause set with a node-only bitset
- omit graph instance, dependency revision, readiness epoch, or stage/order
  binding from executable work
- persist and directly execute a restored ready queue
- count estimates as realized observations
- build Foundational counter rows from expected values rather than Signal's
  performed execution receipt
- derive the locality oracle from production routing/scheduling
- let a Foundational receipt or canonical digest enter an operational
  transition
- make two strategies compare different admitted work streams
- certify only wall-clock improvement

## 4. Product Decision Lock

### 4.1 One Causal Work Progression

The ordinary lane has three lawful semantic origins and one shared progression:

```text
readmitted source-recompute basis
  -> source-obligation admission ---------------------------\
                                                               \
candidate producer output                                      \
  -> prepared direct dependency admission                       \
  -> atomic performed output commit                               \
  -> committed direct invalidation settlements ------------------> resolved current work
                                                                /
performed topology mutation                                    /
  -> structural-recompute admission --------------------------/
  -> topology-lowered work batch
  -> readiness-admitted work batch
  -> performed batch execution
  -> derived operational summary and Foundational receipt
```

No other lane may construct executable invalidation work. In particular:

- a raw root seed cannot skip source-basis admission
- graph reachability cannot skip direct dependency admission
- a prepared output packet cannot skip performed publication
- a topology edit intent cannot skip performed topology mutation and structural
  admission
- a canonical source basis, cause set, or structural marker cannot skip the
  owner-specific resolved-work transition, topology lowering, and readiness
  admission
- a summary, trace, counter bundle, digest, or receipt cannot move backward
  into operational authority

### 4.2 Direct-Hop Realization Is The Only Ordinary Propagation

One performed `ProducedAspectDelta` queries only the producer's current
producer-local aspect/scope reverse-subscription buckets. Each returned direct
edge candidate is admitted or rejected by the Milestone 12 causality owner
using the authoritative edge, dependency snapshot, consumer comparator, and
current dependency revision.

Further propagation does not exist until an admitted consumer evaluates and
performs its own output commit. There is no precomputed semantic transitive
cone and no structural descendant pre-marking in the ordinary invalidation
lane.

Milestone 13 requires a derived reverse subscription index keyed by:

```text
(producer NodeId, producer-local Aspect)
  -> unscoped subscribers
  -> whole-partition subscribers by interned partition token
  -> detail subscribers by interned (partition, detail) token
```

An exact detail change queries unscoped, matching whole-partition, and matching
detail buckets. A whole-partition change may lawfully enumerate that
partition's detail buckets. A whole-aspect or `ConservativeLegacyUnion` change
may use the broader aspect bucket, but the latter cannot satisfy exact-scope
locality certification.

The index is derived from authoritative dependency edges. Topology mutation
updates it transactionally with dependency/subscriber membership; restore may
rebuild and validate it from authoritative topology. An index hit is candidate
discovery only. Causality must still validate the current edge, snapshot,
revision, comparator, and scope, and only that validation can create a cause.

The ordinary exact-aspect/exact-scope lane may not silently fall back to a full
producer subscriber scan. Rebuild, validation, or explicitly conservative
scope lanes are separately named and counted.

Cycle and topology legality checks remain separately named and costed. They
may inspect broader topology before a topology mutation commits; they may not
mint causes, mutate ordinary dirty state, or satisfy locality receipts.

### 4.3 Authority Placement

The strongest-owner rule is normative:

| Owner | Owns | Does not own |
|---|---|---|
| `worth-signal` | graph-local producer/consumer identity, source obligations, dependency causes, node state, topology epochs, ready-work binding, scheduling effects, realized counters, operational denials | reusable generic proof law or cross-runtime descriptive vocabulary |
| `worth-proof` | phase/stage carriers, sealed proof and witness progression, binding-axis comparison, freshness downgrade/readmission topology, `Performed`, typed transition outcomes | Signal graph execution, Signal identities, queue mechanics, counter meaning, financial certification |
| `worth-foundational` | portable canonicalization, case/report identity, performance claim vocabulary, included/excluded work disclosure, counter-backed receipts, support/report packaging | Signal operational authority, `NodeId`, numeric Signal aspect slots, graph instance, dependency revision, ready queue, financial meaning |
| fintech test domain | financial definition, reference arithmetic, necessity/locality expectation manifests, scenario actions, sealed certification verdicts | production invalidation authority or reusable substrate APIs |

`worth-signal` already depends directly on both substrate crates. The
implementation must use those manifests' real public facades; it may not
hand-roll local `Artifact`, `Recipe`, `TransitionOutcome`, `Binding`, canonical
digest, or performance-receipt substitutes.

### 4.4 Compiler-Visible Runtime Phases

Signal must expose owner-specific private wrappers whose internal progression
uses `worth-proof`. Canonical equivalents are:

```rust
pub(crate) struct AdmittedSourceRecompute(
    /* current persisted/readmitted source obligation */
);

pub(crate) struct PreparedDirectInvalidation(
    worth_proof::Recipe<worth_proof::Unresolved, PreparedDirectInvalidationPayload>,
);

pub(crate) struct CommittedDirectInvalidation(
    /* owner wrapper over the resolved/current-basis Proof form */
);

pub(crate) struct AdmittedStructuralRecompute(
    /* performed topology mutation plus current structural obligation */
);

pub(crate) struct ResolvedInvalidationWork(
    /* sealed convergence of the three proven origins */
);

pub(crate) struct LoweredInvalidationBatch(
    /* owner wrapper over the lowered Proof form */
);

pub(crate) struct ReadyInvalidationBatch(
    /* owner wrapper over ExecutionReadyRecipe */
);

pub(crate) struct ExecutedInvalidationBatch(
    /* owner wrapper over ExecutedRecipe plus performed Signal outcome */
);
```

These are semantic requirements, not exact public spellings. The following are
exact requirements:

- wrappers are at most crate-visible and fields/constructors are private to
  their semantic owner; execution transitions are visible only to the
  scheduling orchestrator that owns them
- no `new`, `from_parts`, `Default`, serde constructor, or public generic
  constructor can mint a stronger phase
- each transition consumes the predecessor by value
- later functions accept only the strongest predecessor they need
- the executor accepts only `ReadyInvalidationBatch`
- summaries and receipts accept only `ExecutedInvalidationBatch` or its sealed
  performed receipt
- phases do not coexist behind a runtime enum used as authority
- adding a new required phase must break every construction/progression site
  until it is handled

The Proof pleasant lane may be used where it remains semantically obvious. The
raw Proof lane is permitted when explicit transition nouns make review
stronger. Neither lane creates a second semantic system.

### 4.5 Required Transition Contracts

| Transition | Consumes | Produces | Exclusive authority/capability | Typed non-success |
|---|---|---|---|---|
| admit source obligation | current persisted/readmitted direct source basis plus current node/revision facts | `AdmittedSourceRecompute` | private Signal source-recompute authority | stale basis, rebind required, retired node, internal failure |
| prepare direct admission | candidate output decision plus current direct-edge/snapshot facts | `PreparedDirectInvalidation` | Signal causality preparation owner | denied contract, stale snapshot, rebind required, internal failure |
| publish and resolve | prepared direct admission plus the exact atomic output-commit packet | `CommittedDirectInvalidation` | private `OutputCommitAuthority` and performed publication outcome | publication failure; no committed work exists |
| admit structural obligation | performed topology mutation plus current structural-recompute basis/revision | `AdmittedStructuralRecompute` | private topology-mutation authority | rejected mutation, stale revision, rebind required |
| converge resolved origin | exactly one of the three admitted owner forms | `ResolvedInvalidationWork` | private invalidation-origin authority | stale/rebind-required origin; no raw basis accepted |
| lower topology | `ResolvedInvalidationWork` plus current topology/stage basis | `LoweredInvalidationBatch` | private Signal topology-lowering capability | stale revision, rebind required, cycle/topology denial |
| admit readiness | lowered batch plus current planner epoch and pending-predecessor truth | `ReadyInvalidationBatch` | private Signal readiness authority | deferred dependency, stale epoch, condition/async denial without cause loss |
| execute | ready batch plus transaction-local evaluation/apply capability | `ExecutedInvalidationBatch` | private Signal execution authority; `Performed` records the outcome | failure/rollback before commit, typed partial posture only after a lawful commit boundary |
| package evidence | executed receipt plus Signal-owned realized counters | Foundational counter-backed receipt and cold reports | evidence materializer only | missing/duplicate/mismatched counter row, wrong work disclosure, stale case identity |

An authority witness authorizes its one transition; it is not a proof that the
transition happened. `Performed` is created only where Signal observes the
actual effect outcome.

The owner denial topology must preserve at least:

```rust
pub(crate) enum InvalidationProgressionDenial {
    StaleGraphInstance,
    StaleDependencyRevision,
    StaleOriginGeneration,
    StaleReadinessEpoch,
    StaleStageOrder,
    RebindRequired,
    DependencyPending,
    ConditionDeferred,
    AsyncCapabilityUnavailable,
    TopologyCycle,
    SubscriptionIndexRebuildRequired,
    ContractRejected,
}
```

Internal defects remain a distinct failed outcome rather than a denial.
Publication rollback, readiness deferral, stale input, rebind requirement, and
hard failure may not collapse into `Result<(), SignalError>` or a boolean at
the phase boundary. The exact enum may be split by phase when that better
preserves ownership, but the distinctions and remediation posture are fixed.

The publish-and-resolve transition preserves Milestone 12 atomicity: output
identity/version, dependency snapshots, performed producer delta, direct cause
set replacements, and predecessor-settlement state commit or roll back as one
packet. Later scheduling consumes that committed state. It does not republish
causes. Batch execution owns readiness queue mechanics and consumer evaluation,
not the semantic cause decision.

The source and structural origins do not pretend that a producer output commit
occurred. They retain distinct origin evidence through `ResolvedInvalidationWork`
and execution receipts. Convergence shares only topology lowering, readiness,
ordering, and execution law.

### 4.6 Binding And Freshness

Executable work must carry a declarative `worth_proof::Binding` with at least:

- graph instance
- target `NodeId` (the consumer for dependency-origin work)
- target node dependency revision
- sealed origin binding: source-admission generation, canonical dependency
  cause-set generation plus producer commit ordinals, or performed topology-
  mutation ordinal
- planner/readiness epoch
- lowered stage/order identity

The existing dependency-cause binding remains the semantic cause authority and
is not flattened into the work binding. The work binding says the scheduled
form and its source/dependency/structural origin are still current for
execution; a dependency-cause binding says why a consumer is semantically
invalidated.

Every binding axis requires a positive case and a one-axis drift twin through
the Proof binding certification surface. A mismatch produces typed stale or
rebind-required progression before execution.

Ready work is derived, process-local state:

- it is not serialized into graph checkpoints
- it is not restored as current authority
- branch, checkpoint, graph-instance, dependency-revision, or topology-epoch
  boundaries discard it or bridge it to a non-executable Proof freshness form
- current work after restore is rebuilt from readmitted direct source bases,
  canonical dependency causes, node state, and current topology
- a Foundational canonical identity may compare pre/post cases but cannot
  re-admit the work

### 4.7 Canonical Work Item And Batch

The Signal-owned forms must be equivalent to:

```rust
pub(crate) struct InvalidationWorkItem {
    node: NodeId,
    dependency_revision: DependencyRevision,
    input: CanonicalResolvedInvalidationInput,
    readiness_epoch: InvalidationReadinessEpoch,
}

pub(crate) struct ReadyInvalidationBatch {
    stage: StageId,
    entries: CanonicalNonEmptyWorkItems,
    binding: worth_proof::Binding<InvalidationWorkBindingAxes>,
    /* Proof execution-ready carrier retained privately */
}
```

`CanonicalResolvedInvalidationInput` is a sealed owner form retaining exactly
one lawful origin: source-recompute basis, dependency-cause set, or structural-
recompute basis. Dependency causes remain aspect-correlated and dependency-
specific. There is no separate `narrowed_scopes` field whose value can drift
from the origin. Derived masks and locality summaries are projections only.

The canonical batch guarantees:

- non-empty membership
- one entry per target node and readiness epoch
- canonical target order within one lowered stage
- exact retained origin; dependency-origin entries retain the exact cause union
- current dependency revision and graph instance
- no cross-stage entry

### 4.8 Scheduling Is Mechanical

Scheduling may:

- combine already-admitted items for the same target/epoch
- lower items into existing WORTH stage/topology order
- defer items whose predecessors are unresolved
- enqueue, pop, and batch current ready work
- choose an internal measured mechanical strategy after lowering

Scheduling may not:

- inspect a root aspect to infer descendant meaning
- discover a new changed aspect or scope
- call a consumer comparator to mint a cause
- reconstruct causes from current graph scans
- widen an edge scope
- erase dependency identity during deduplication
- execute stale/rebind-required work
- make diagnostics policy affect admission or order

The public invalidation facade remains stable. Queue and traversal strategy are
not caller-selectable policy in this milestone.

### 4.9 Deduplication Preserves Causality

Deduplication identity is `(graph instance, target node, dependency revision,
readiness epoch, stage)`. A merge unions canonical cause keys and rejects any
cause whose binding is stale for that identity.

Two causes for the same consumer are not duplicates merely because they share
an aspect slot. Distinct producer, edge scope, cached version, output ordinal,
or committed version remains visible in the canonical cause set.

The deduplicator returns a new lowered/ready proof form. It does not mutate a
batch in place while using the old proof.

### 4.10 Realized Cost Truth And Foundational Packaging

Signal's performed execution receipt owns exact realized counters for:

- source output deltas consumed
- direct subscriber edges examined
- reverse-index bucket probes
- reverse-index candidates returned
- candidates rejected by aspect contract
- candidates rejected by partition/detail scope
- candidates rejected by comparator
- direct settlements produced
- work items admitted
- work items merged by deduplication
- ready items enqueued
- ready items popped
- stale/rebind-required work rejected
- nodes evaluated
- produced deltas emitted
- propagation stops from unchanged output
- non-semantic routing/scheduling node visits
- maximum and retained ready-frontier width
- topology/revision revalidation work
- batch-local allocations and peak batch memory where measurable

Predicted counts remain a separate planning artifact with names that cannot be
mistaken for performed execution. A predicted value never enters a
counter-backed execution receipt row.

After performed Signal execution, evidence packaging uses the existing
`worth_foundational::performance()` and counter-backed receipt surfaces. The
ordinary claim is:

- boundary: `AuthoritativeExecution`
- evidence strength: `CounterBackedExecutionReceipt`
- breadth/locality: `DeltaBound`
- access posture: `TraversalLocal` for the certified direct-hop strategy, or
  `DensityAdaptive` only when the same-work-stream strategy evidence exists
- temperature: `HotPath`
- freshness: `ExactBasisCurrent`
- fallback/debt: `Verified`
- included work: the exact authoritative read, validation/planning, and
  mutation work performed inside the named boundary
- excluded work: replay/reconstruction, support report assembly, and forensic
  parity

Each scenario creates its counter specs from an independent
`FinancialLocalityExpectationManifest`, then attaches the observed rows copied
from the performed Signal receipt. The Foundational builder must reject
missing, duplicate, unexpected, or mismatched rows. The cost-slope report is a
separate derived verdict across canonical receipts; it may not rewrite a row
to satisfy a slope.

Foundational canonicalization establishes case and report identity. Direct
hashing, debug formatting, or a local string-join digest is forbidden for
cross-boundary certification identity.

The M13 case/report basis extends the existing Milestone 12 financial
certification identity family and rule-version posture. It does not create a
parallel locality-only identity authority; the additional scale, work-stream,
counter-contract, and execution-posture axes are appended through the
Foundational canonical basis API.

### 4.11 Authoritative And Derived State

Authoritative operational state is limited to:

- direct source invalidation bases
- canonical dependency causes
- dependency revisions and topology
- dependency snapshots
- node state and committed output identity/version

Ready queues, work batches, masks, counter bundles, summaries, traces,
reverse subscription indexes, Foundational receipts, slope reports, and
certification runs are derived.
Destroying all of them must not prevent complete reconstruction of lawful
future work from authoritative state.

### 4.12 Diagnostics And Evidence Lanes

Operational counters needed to interpret the locality contract are emitted
independently of diagnostic tier. Rich per-entry traces remain policy-controlled
sidecars.

- `Operational` retains canonical counters and bounded summaries
- `Development` may retain entry-level decisions
- `Forensic` may retain richer cause/ordering material

All tiers execute the same proof progression and commit the same truth. Cold
report/canonicalization work is counted outside the ordinary execution receipt.

### 4.13 Strategy Decision

Mechanical strategies may be compared only after semantic admission and with
the same canonical lowered work identity. The test-domain conclusion is:

```rust
pub enum TraversalStrategyDecision {
    CurrentStrategyCertified,
    OrderedReadyWorkCandidate(OptimizationEvidence),
    InsufficientEvidence(MeasurementGap),
}
```

This is certification evidence, not runtime policy or execution authority. It
does not pre-authorize a priority queue, tree algorithm, or caller-selected
backend. `InsufficientEvidence` blocks the corresponding optimization claim;
it does not weaken correctness or permit the old reachability walk.

### 4.14 Public Facade Cutover

The currently re-exported `FrontierPlan`, `FrontierWave*`,
`TransitiveFrontier*`, and publicly constructible execution summaries encode
the inherited reachability algorithm and cannot remain the public contract.

The migration is fixed:

- operational prepared/lowered/ready forms remain internal and are not
  re-exported
- caller-visible planning data becomes a read-only
  `InvalidationPlanningEstimate` whose name and type state that its counters
  are predicted
- performed operational observation becomes
  `SignalInvalidationExecutionReceipt`, with private construction and realized
  counters
- optional `InvalidationExecutionSummary` and trace views derive from the
  performed receipt and remain non-authoritative
- direct/transitive wave constructors and `TransitiveReachability` locality
  claims are removed from integration/adapters facades
- no deprecated alias may preserve public construction of an operational plan
  or allow a predicted form where a performed receipt is required

Migration documentation names replacements and the semantic difference. A
source-compatible adapter is allowed only if it returns a descriptive derived
view and cannot satisfy any operational or performed-evidence bound.

## 5. Required Proof And Evidence Forms

### 5.1 Signal-Owned Operational Forms

Implementation must establish and keep private canonical equivalents of:

- `PreparedDirectInvalidation`
- `CommittedDirectInvalidation`
- `AdmittedSourceRecompute`
- `AdmittedStructuralRecompute`
- `ResolvedInvalidationWork`
- `InvalidationWorkBindingAxes`
- `LoweredInvalidationBatch`
- `ReadyInvalidationBatch`
- `ExecutedInvalidationBatch`
- `SignalInvalidationExecutionReceipt`
- `SignalInvalidationRealizedCounters`

Each type must document:

- what it proves
- its private constructor/transition owner
- what it authorizes
- what it cannot authorize
- its trust/freshness posture
- its only lawful consumers

### 5.2 Proof Substrate Use

The operational family must use:

- `Recipe` or `Artifact` for compiler-visible phase
- private owner-specific `AuthorityWitness` and `CapabilityWitness` values for
  progression
- `TransitionOutcome` for success, denial, deferred, stale,
  rebind-required, and failed outcomes
- `Binding` plus binding-axis drift certification for current work identity
- `Performed` only at the actual Signal effect boundary
- Proof freshness/boundary forms when work crosses a trust boundary
- `NonEmpty`, canonical/unique structural proof forms where their generic law
  matches the batch invariant

Signal-specific wrappers remain the operational API. A generic Proof carrier
does not authorize Signal execution by itself.

### 5.3 Foundational Evidence Forms

The financial certification owner must establish:

- `FinancialLocalityExpectationManifest`
- opaque scenario-family completion evidence for each of the six courtrooms
- `InvalidationCostSlopeReport`
- `InvalidationStrategyReport`
- `FinancialFrontierLocalityCertificationRun`
- Foundational-canonical case and report artifacts
- attached `FoundationalCounterBackedPerformanceReceipt` values

These remain test-domain evidence. Production exports only domain-neutral
Signal execution receipts/counters and selectively exposed summaries. A second
domain courtroom may justify extracting portable certification vocabulary
later; Milestone 13 does not place financial terms in either substrate crate.

Each scenario-family completion is constructed only by that scenario runner
after it verifies fresh financial truth, necessity, current work identity,
counter-backed receipts, slope obligations, lifecycle posture, and its named
mutation probes. There is no generic `completion(scenario, bool)`, public
`from_verified`, or caller-supplied duplicate expected/actual field pair.

`FinancialFrontierLocalityCertificationRun` consumes exactly one opaque
completion for every required scenario, derives the sorted Foundational case
identities itself, and constructs one Foundational-canonical report identity.
Cost-slope and strategy reports retain the exact case/work/receipt identities
they compare. Changing both sides of a caller-authored equality, changing all
seeds together, or relabeling one strategy's work stream must not seal.

### 5.4 Compiler-Enforcement Evidence

Internal phase safety is proved at its real boundary rather than by exposing a
test-only facade:

- private fields and constructors prevent sibling modules from fabricating a
  stronger owner form
- transition functions consume exact predecessor types and return exact
  successor types
- the executor is module-visible only to scheduling and accepts only the ready
  form
- the evidence materializer accepts only the executed/performed receipt
- an internal compile-pass progression fixture type-checks the complete lawful
  orchestration without alternate constructors
- construction/export inventories mechanically assert that no stronger form,
  authority witness, or execution function is publicly constructible or
  re-exported
- QA mutation probes attempt each skip or substitution and record the expected
  compiler failure against the frozen source

The mutation matrix covers prepared -> execute, committed -> execute, lowered
-> execute, raw root seed/direct basis -> ready work, topology edit intent/raw
structural basis -> ready work, raw delta -> performed work, raw cause set ->
ready work, stale/bridged work -> execute, and Foundational evidence -> Signal
transition.

Committed compile-pass/compile-fail UI cases are required only for a genuine
caller-visible impossibility. A private type failing to import is not evidence
of correct phase topology, and Milestone 13 must not widen visibility or add a
test-only production branch merely to create such a test.

Runtime drift, stale revision, queue behavior, cost, and financial truth remain
runtime/property/certification evidence.

## 6. Architectural Destination

The destination tree is normative. Status annotations describe Milestone 13's
required cutover.

```text
crates/worth-signal/src/
  data/proof/invalidation/
    mod.rs                                      [existing; stable internal facade]
    binding.rs                                  [existing; cause axes retained]
    source_seed.rs                              [existing M12 root obligation]
    output_commit.rs                            [existing M12 performed delta]
    revalidation.rs                             [existing M12 cause/input truth]
    progression_contract_tests.rs               [Phase 1 internal lawful progression prototype]
    progression_contract_mutation.txt           [Phase 1 actual-source E0308 mutation record]
    progression/                                [created; dominant axis = phase truth]
      mod.rs                                    [stable owner-specific proof facade]
      source.rs                                 [created; admitted source origin]
      prepared.rs                               [created; unperformed direct admission]
      committed.rs                              [created; performed commit-derived truth]
      structural.rs                             [created; admitted topology origin]
      resolved.rs                               [created; sealed origin convergence]
      lowered.rs                                [created; stage/order-bound work]
      ready.rs                                  [created; execution-admitted work]
      executed.rs                               [created; performed execution receipt]
      binding.rs                                [created; executable work axes/freshness]
    observation/                                [created; descriptive public views]
      mod.rs                                    [stable observation facade]
      prediction.rs                             [created; planning estimate]
      receipt.rs                                [created; performed realized receipt]
      summary.rs                                [created; optional derived summary]
    plan.rs                                     [removed inherited reachability shape]
    execution.rs                                [removed inherited wave shape]

  logic/invalidation/
    causality/
      dependency_admission.rs                   [existing M12 meaning owner; revised]
      revalidation.rs                           [existing M12 pending resolution]
      cause_aggregation.rs                      [existing M12 cause union]
    routing/
      mod.rs                                    [created from routing.rs facade]
      direct_admission.rs                       [created from planning.rs]
      direct_application.rs                     [created; direct settlements only]
      predicted_cost.rs                         [created/narrowed planning estimates]
      realized_cost.rs                          [created; performed counter capture]
      evidence.rs                               [existing; derived sidecars only]
    scheduling/                                 [created; dominant axis = ready lifecycle]
      mod.rs                                    [stable internal facade]
      lowering.rs                               [created; stage/order transition]
      readiness.rs                              [created; pending/condition admission]
      deduplication.rs                          [created; cause-preserving merge]
      queue.rs                                  [created; transaction/batch-local mechanics]
      execution.rs                              [created; consumes ready form only]

  data/graph/
    topology/subscriber_index/                  [created; derived narrowing]
      mod.rs                                    [stable derived-index facade]
      membership.rs                             [created; topology delta apply]
      buckets.rs                                [created; aspect/partition/detail lookup]
      rebuild.rs                                [created; authority-only reconstruction]
    storage/invalidation_causes/                [existing M12 authority]
    runtime/effect/output_commit.rs             [existing M12 atomic owner; revised]

  tests/domains/fintech/
    world/
      locality_scale.rs                         [created; frozen ordinary/scheduled tuples]
      locality_definition.rs                    [created; immutable locality-world meaning]
      locality_definition/actions.rs            [created; ordered economic/lifecycle traces]
      locality_definition/policy.rs             [created; economic admission/comparison/output policy]
      locality_definition/validation.rs         [created; generator invariant enforcement]
      locality_definition/generation/           [created; dominant axis = scenario family]
        sparse.rs                               [created; depth/fanout financial generator]
        partitioned.rs                          [created; independent R/M/I curve generator]
        convergent.rs                           [created; four-producer convergence contract]
        dense.rs                                [created; exact affected-density contract]
        churn.rs                                [created; topology receipt/revision contract]
        restore.rs                              [created; narrow/convergent/dense lifecycle contract]
      compiler/
        compiled_authority.rs                   [created; shared portfolio/locality authority]
        locality_topology.rs                    [created; economic subscription and policy-to-Signal lowering]
        locality_evaluation.rs                  [created; scoped financial evaluators]
        locality_execution.rs                   [created; scenario compile, action, and settlement]
        locality_execution/
          baseline.rs                           [created; baseline topology and artifact seal]
    invalidation/
      boundary_inventory_m13.rs                 [created; Phase 1 current-boundary record]
      boundary_inventory_m13/counters.rs        [created; exact counter/topology/export census]
      boundary_inventory_m13/constructors.rs    [created; exact public constructor signature census]
      boundary_inventory_m13/owners.rs          [created; complete owner-function census]
      boundary_inventory_m13/planner.rs         [created; planner/readiness/executor census]
      boundary_inventory_m13/source_edges.rs    [created; parsed owner-census mutation proofs]
      boundary_inventory_m13/source_edges/
        owner_bodies.rs                         [created; 50 routing/planner/writer/publication owners]
        parsed_owner.rs                         [created; AST-scoped owner-body identity enforcement]
        source_files.rs                         [created; 33 critical runtime/planner file AST census]
      locality_red_controls.rs                  [created; inherited-breadth slope baseline]
      sparse_book_fanout.rs                     [created]
      partitioned_curve_universe.rs              [created]
      convergent_factor_batch.rs                 [created]
      dense_market_close.rs                      [created]
      portfolio_dependency_churn.rs              [created]
      branch_restore_locality_replay.rs           [created]
    certification/invalidation/
      fresh_recompute.rs                        [existing M12 oracle]
      locality_fresh_recompute.rs               [created; locality-world financial oracle]
      necessity_manifest.rs                     [existing M12 oracle; extended]
      locality_contract.rs                      [created; generator/mutation/identity contract]
      locality_contract/mutations.rs            [created; exhaustive scenario mutation sets]
      locality_expectation.rs                   [created independent counter oracle]
      locality_expectation/counter_contract.rs  [created exact scenario counter rows]
      locality_expectation/
        independence.rs                         [created; complete oracle dependency-graph enforcement]
        independence/module_graph.rs            [created; fail-closed module/reexport provenance]
        independence/module_graph/syntax.rs     [created; crate/self/super/glob/macro path resolution]
        trace.rs                                [created; complete economic delta/work trace]
        trace/actions.rs                        [created; ordered economic/lifecycle interpreter]
        trace/actions/topology.rs               [created; accepted/rejected topology receipt interpreter]
        trace/checkpoints.rs                    [created; durable cause/source-basis reconstruction checkpoints]
        trace/derivation.rs                     [created; semantic delta and work derivation]
        candidates.rs                           [created; ordered Q/C and fully bound K]
        recovery_assertions.rs                  [created; exact restore provenance courtroom]
        tests.rs                                [created; fault-sensitive manifest proofs]
        scenario_tests.rs                       [created; scenario courtroom contracts]
      locality_case_identity.rs                 [created Foundational identity]
      locality_receipt.rs                       [created receipt attachment]
      cost_slope.rs                             [created cross-scale verdict]
      strategy_decision.rs                      [created same-work-stream verdict]
      locality_run.rs                           [created sealed run]

crates/worth-signal/tests/
  milestone_13_compile_time.rs                  [legacy caller-visible reachability inventory]
  ui/milestone_13/                              [positive caller compatibility twin]
```

Structural ownership:

- `data/proof/invalidation/progression` owns phase truth, not algorithms
- `data/proof/invalidation/observation` owns immutable descriptive views and
  cannot be imported by operational transitions
- `causality` owns semantic admission and cause meaning
- `routing` owns immediate-edge realization and its measured cost
- `scheduling` owns already-admitted work lifecycle and cannot import financial
  certification
- `output_commit` is the only promotion boundary from prepared to committed
- source admission and performed topology mutation are separate origin owners;
  neither can mint dependency-commit truth
- `topology/subscriber_index` is a rebuildable candidate index beneath the
  causality owner; it never stores or constructs causes
- fintech certification consumes receipts and cannot be imported by production
- Foundational and Proof remain dependencies beneath Signal's owner-specific
  facade; dependency direction never reverses

Forbidden placements:

- a generic `helpers`, `utils`, `common`, `manager`, or catch-all `logic` file
- phase constructors in a public facade
- financial scenario types in production modules
- Signal `NodeId`, graph instance, aspect slot, or queue vocabulary in
  `worth-foundational`
- Signal scheduling policy or runtime graph mechanics in `worth-proof`
- diagnostics, reports, or serde restore paths that mint ready authority
- a replacement queue hidden inside `routing/application.rs`
- a second cause store inside scheduling
- deprecated aliases that preserve public reachability-plan construction or
  transitive-wave locality claims

The existing broad `routing/application.rs`, reachability summaries, and
transitive counters are replaced or deleted during cutover; they do not survive
as a compatibility lane.

## 7. Ordered Implementation Phases

Every phase advances runtime authority or proof strength. A phase may not close
with its assigned financial scenario red, absent, or deferred to Phase 7.

### Phase 1 - Boundary Inventory, Red Slopes, And Proof Contract Freeze

What becomes true:

- the exact current transitive walk, constructor surfaces, counter sources,
  queue/planner seams, and restore paths are inventoried
- small and medium sparse/partition cases demonstrate the inherited breadth
  failure with realized counters
- every scenario's typed scale tuple, generator invariants, `Q/C/K/U/E/S/P`
  expectation manifest, identity-sidecar contract, and assigned mutation set
  are frozen before production routing changes
- the full Signal/Proof/Foundational ownership matrix, three lawful origin
  families, and shared phase type family are frozen before production cutover

Required evidence:

- `sparse_book_fanout` and `partitioned_curve_universe` red controls fail
  because non-semantic visits grow, not because financial setup is broken
- the three slope classes are independently red/green: index-disjoint work is
  exactly flat, queried-candidate work grows by exact bucket membership, and
  semantic work grows by the independent necessity delta
- internal compile-pass prototype proves the intended Proof progression is
  supported by existing dependencies
- a QA source mutation that passes prepared work to the ready-only executor
  fails compilation without widening production visibility
- independent locality expectation manifests do not import routing/scheduling

Phase 2 may trust the authority placement and falsifiable baseline.

### Phase 2 - Owner-Specific Proof Progression And Atomic Promotion

What becomes true:

- admitted-source, prepared-direct, committed-direct, admitted-structural,
  resolved, lowered, ready, and executed Signal forms exist behind private
  constructors
- atomic output publication is the only transition that promotes prepared
  admission into committed invalidation truth
- binding axes and freshness outcomes are compiler/runtime visible

Required evidence:

- construction/export inventory, lawful compile-pass progression, and recorded
  compiler-failure mutation matrix for skipped/forged phases
- failure at every prepublication seam produces no committed work
- every executable-work binding axis has a one-axis drift denial twin
- valid M12 output commit/cause behavior remains green

Phase 3 may trust that unperformed or stale facts cannot enter execution.

### Phase 3 - Direct-Hop Routing Cutover

What becomes true:

- the ordinary transitive closure walk and transitive pre-marking are removed
- performed output deltas query only current producer-local aspect/scope index
  buckets and validate returned authoritative direct edges
- disjoint direct edges are rejected before dirty mutation and ready enqueue
- unchanged outputs stop propagation without descendant visits

Required evidence:

- `sparse_book_fanout` turns green at small/medium scales
- `partitioned_curve_universe` turns green across region-count/overlap twins
- index-destroy/rebuild and index-drift denial twins prove topology remains the
  authority
- a same-depth but increasing disjoint fanout slope remains flat with respect to
  disjoint-aspect/disjoint-region candidate edges and non-semantic visits
- mutations restoring closure walking or late filtering turn both scenarios red

Phase 4 may trust direct committed settlements as the only work source.

### Phase 4 - Topology Lowering, Readiness, And Causal Deduplication

What becomes true:

- direct settlements lower into canonical stage-bound batches
- readiness admission consumes current pending-predecessor, condition, and
  async truth
- same-target/same-epoch work deduplicates without losing causes
- only ready forms can execute

Required evidence:

- `convergent_factor_batch` passes seed-order and duplicate-cause permutations
- `dense_market_close` passes sparse, medium, and dense small-scale twins
- queue insertion/pop and dedup counters come from performed mechanics
- ordinary, temporal, custom, installed, on-demand, and async pending-first
  guarantees remain green

Phase 5 may trust one current, canonical, causally complete ready stream.

### Phase 5 - Rewire, Restore, And Trust-Boundary Reconstitution

What becomes true:

- dependency revision/topology epoch drift invalidates earlier work
- ready queues are absent from durable authority and cannot be restored current
- supported restore/readmission rebuilds lawful work from M12 authority
- branch and replay preserve truth/order without importing recovery cost into
  the hot lane

Required evidence:

- `portfolio_dependency_churn` rejects stale same-shaped rewire work
- `branch_restore_locality_replay` proves pre-restore ready forms cannot execute
  and reconstructed work has the current binding
- exact source bases and dependency causes survive; derived queue identity does
  not
- cycle rejection remains atomic and produces no work

Phase 6 may trust current-basis performed execution receipts.

### Phase 6 - Realized Counters And Foundational Evidence Cutover

What becomes true:

- predicted and realized counter types, names, storage, and consumers are
  mechanically separate
- Signal execution receipts contain the exact hot-path rows
- public integration/adapters facades expose the predicted/performed
  distinction and remove reachability-shaped constructors
- Foundational counter-backed receipts attach only after performed execution
- case/report identities use Foundational canonicalization
- diagnostics tiers cannot change operational rows

Required evidence:

- missing, duplicate, unexpected, predicted-as-realized, or mismatched rows are
  denied
- included/excluded work disclosure rejects replay/support laundering
- Operational/Development/Forensic twins produce identical operational
  receipts and different lawful sidecars only
- a mutation copying expectation values into observed rows fails independent
  state/counter observations

Phase 7 may trust canonical receipts for slope and strategy decisions.

### Phase 7 - Scale Courtroom, Strategy Decision, And Closeout

What becomes true:

- all six scenario families pass ordinary and scheduled lanes
- sparse, partitioned, convergent, churn, and dense cost slopes are sealed
- strategy comparisons consume identical admitted work identities
- one `FinancialFrontierLocalityCertificationRun` and one typed strategy
  decision are produced from complete, current evidence
- `S9.16.3` and the invalidation portion of `S9.16.6` are closed

Required evidence:

- scheduled `10^3`, `10^4`, and `10^5` financial scale sweeps with independent
  irrelevant-fanout and semantic-density axes
- missing, duplicate, stale, wrong-policy, wrong-scale, wrong-posture,
  wrong-counter-contract, mixed-work-stream, or mismatched-oracle reports are
  rejected
- deterministic and optimized modes commit equivalent canonical truth
- historical red mutations remain red
- all documentation and successor contracts match final source

Milestone 13.1 may trust only the sealed canonical work stream and measured
envelopes, not an implementation-specific queue. Milestone 14 inherits that
same restriction after the platform carriage cutover.

## 8. Complexity And Resource Contracts

The ordinary direct-hop invalidation lane targets:

```text
O(committed_source_deltas
  + reverse_index_bucket_probes
  + examined_indexed_direct_edge_candidates
  + admitted_direct_settlements
  + ready_order_cost
  + evaluated_nodes)
```

`ready_order_cost` must be declared for the selected mechanical strategy and
must be `log-or-better` per admitted item or a measured batch-local alternative
whose density regime is explicit.

The bound must not contain:

- total graph nodes
- total reachable descendants
- all producer subscribers for an exact-aspect/exact-scope change
- unrelated partitions/regions
- diagnostic trace count
- replay suffix length
- report/canonicalization work

unless the named semantic delta or separately named lane actually includes
that breadth.

Additional contracts:

- memory is bounded by current direct settlements plus current ready frontier,
  not historical admissions or total graph size
- retained queue memory returns to the current-live bound after a batch
- topology/cycle preflight has its own counter contract and cannot satisfy the
  ordinary locality receipt
- reverse-index maintenance/rebuild has named topology-mutation/reconstruction
  contracts and cannot be laundered into the direct execution receipt
- branch restore/replay has a separate reconstruction receipt
- no background queue may grow without an explicit bound and ownership
- wall-clock measurements name hardware, runtime configuration, cold/warm
  posture, repetitions, variance, and percentiles, but structural counters are
  the correctness boundary for locality

## 9. Documentation Deliverables

Milestone 13 must revise these durable audience documents:

| Audience | Authoritative document | Required content and implementation check |
|---|---|---|
| runtime implementers | `signal_architecture2.md` | direct-hop work progression, Proof phase topology, owner authorities, current-basis restore law, and M14 handoff; type/module names checked against source |
| acceptance/QA reviewers | `s9_16_acceptance_map.md` | exact semantic versus locality closure, phase denial matrix, counter ownership, and sealed evidence |
| test authors | `test-requirements.md` | authentic financial worlds, independent locality manifest, scale axes, mutation probes, compile/runtime proof boundaries, and lane budgets |
| Signal callers | `crates/worth-signal/README.md` and relevant reference page | observable locality/counter contract, predicted versus realized meaning, unchanged public invalidation semantics, and typed failures if any facade changes |
| financial courtroom maintainers | fintech `README.md` | six scenario families, reproduction tuple, oracle separation, scheduled lanes, Foundational receipt construction, and mutation catalog |
| substrate users | existing Proof/Foundational docs only if adoption exposes a genuine missing generic contract | Signal adoption examples must use substrate vocabulary without adding Signal or financial meaning to substrate docs |

Examples that describe public surfaces must compile or run against the real
facade. Obsolete reachability-locality claims, transitive-scope summaries, and
counter names must be corrected or removed rather than left beside the new
contract.

## 10. Must Ship And Must Preserve

Must ship:

- compiler-visible prepared -> committed -> lowered -> ready -> executed
  progression using `worth-proof`
- owner-specific private Signal wrappers and authorities
- direct-hop-only ordinary routing
- rebuildable producer/aspect/partition/detail reverse subscription indexing
- pre-mutation/pre-enqueue aspect and scope rejection
- current-basis work binding and drift denial
- canonical stage-bound ready batches
- cause-preserving deterministic deduplication
- realized Signal counters and performed execution receipts
- public predicted-estimate/performed-receipt migration with no operational
  compatibility lane
- Foundational canonical case/report identities and counter-backed receipts
- independent financial locality expectation manifests
- scale-sensitive sparse, partitioned, convergent, churn, restore, and dense
  certification
- one sealed locality run and typed strategy decision

Must preserve:

- all Milestone 12 cause binding axes and exact versus conservative scope truth
- producer output-equivalence versus consumer comparator separation
- transaction rollback and commit-bounded observation
- cycle rejection before unlawful topology state commits
- deterministic stage/publication semantics
- branch, checkpoint, replay, temporal, condition, installed, on-demand, and
  async lifecycle guarantees
- WASM-capable serial semantics
- diagnostics policy independence
- domain-agnostic production vocabulary

## 11. Explicit Exclusions

Milestone 13 does not:

- implement graph-wide parallel execution or resource leases
- implement an order-maintenance tree or prescribe a priority queue
- expose a caller-selectable traversal/queue strategy
- persist ready queues as authority
- move Signal graph identity or aspect slots into Foundational
- move Signal runtime progression into Proof as a generic graph engine
- add financial or geometry vocabulary to production invalidation
- claim geometry, imaging, simulation, or another domain ready from financial
  evidence
- accept conservative legacy scope unions as exact locality evidence
- replace structural counters with elapsed-time thresholds
- preserve the inherited transitive walk as fallback or compatibility behavior

## 12. Acceptance Evidence

Milestone 13 closes only when:

- the complete transitive subscriber-closure walk is absent from the ordinary
  invalidation lane
- performed immediate-producer commit is the only source of committed direct
  dependency work; current source and structural obligations enter only through
  their separate owner admissions
- admitted-source, prepared-direct, committed-direct, admitted-structural,
  resolved, lowered, ready, and executed phases/origins are compiler-visible
  forms with private owner transitions
- invalid/skipped/out-of-order progression is uncallable through private
  constructors, consuming signatures, executor visibility, construction/export
  inventories, lawful compile-pass evidence, and compiler-failure mutations
- work binding drift on every declared axis is denied before execution
- restored/boundary-bridged ready work cannot execute
- disjoint subscribers are rejected before dirty mutation and enqueue
- increasing irrelevant fanout/partition count does not create proportional
  exact-lane index candidates, non-semantic visits, dirty mutations, or ready
  work
- exact-aspect/exact-scope routing never falls back to scanning every producer
  subscriber; conservative/rebuild lanes are separately typed and counted
- all admitted work retains exact M12 immediate-dependency causes
- multi-source deduplication is deterministic and causally complete
- predicted and realized counters are different types/surfaces and no predicted
  row can enter a performed receipt
- old public reachability plans/transitive-wave constructors are removed; any
  retained compatibility view is descriptive-only and cannot satisfy an
  operational or performed-evidence bound
- Signal owns observed counters; the independent financial manifest owns
  expectations; Foundational attaches and canonicalizes without gaining
  operational authority
- Foundational receipts disclose exact included/excluded work and reject
  missing, duplicate, unexpected, or mismatched rows
- deterministic and optimized strategies consume the same canonical work
  identity and commit equivalent results
- full fresh recompute and financial necessity oracles remain green
- every implementation phase closes only after its assigned financial
  scenarios and mutation probes are green/red as required
- `FinancialFrontierLocalityCertificationRun` rejects missing, duplicate,
  stale, wrong-seed, wrong-scale, wrong-policy, wrong-posture,
  wrong-counter-contract, mixed-strategy-stream, or mismatched evidence
- ordinary and scheduled scale lanes meet their declared structural envelopes
- every scenario satisfies its frozen `Q/C/K/U/E/S/P` manifest and its assigned
  mutations fail for the named causal observation rather than setup failure
- index-disjoint, queried-candidate, and semantic-frontier slopes are reported
  and decided separately; no aggregate fanout or wall-clock threshold can
  substitute for them
- focused tests, complete `worth-signal` default and parallel-feature suites,
  any justified public compile-time UI lane, doctests, WASM check, formatting, boundary check,
  agent-context check, dirty line-cap check, and diff check pass
- a fresh final critic reviews the frozen final-source fingerprint rather than
  an earlier repair state

## 13. Successor Handoff

[Milestone 13.1 - Cross-Runtime Granular Invalidation](./milestone-13.1-plan.md)
inherits:

- a Signal-owned, Proof-progressed canonical ready-work stream
- exact current-basis work bindings
- cause-preserving work identity
- direct-hop locality and sparse/dense measured envelopes
- exact unscoped/partition/detail locality as the certified two-segment base
  case for a future bounded `ScopePath`
- performed Signal execution receipts
- Foundational counter-backed locality evidence
- one typed mechanical strategy conclusion

Milestone 13.1 must carry this sealed Signal-local contract through Runtime
Bridge installed correspondence and Query-owned impact, maintenance, and
publication. It may not turn any Signal index key, cause, readiness product,
or performed receipt into Bridge correspondence or Query permission.

Milestone 14 may prepare and execute independent ready batches under resource
leases. It may not weaken direct-hop admission, current-basis binding,
cause-preserving deduplication, deterministic order, either independent
financial oracle, the Foundational work-disclosure boundary, or the distinction
between admission authority and performed execution.

Milestone 14 may generalize the physical shape of the derived subscriber index
from partition/detail to a bounded opaque hierarchy and may derive physical
execution shards from already-admitted work. It must preserve M13's exact
two-segment behavior, immediate-producer-local aspect meaning, and causal
revalidation. Neither `ScopePath`, `ProducerAspectKey`, nor an execution shard
may become edge, cause, readiness, or commit authority. Geometry and other
domains assign path-segment meaning outside `worth-signal`.

If and only if the final decision is `OrderedReadyWorkCandidate`, a separate
WORTH-native traversal optimization specification may consume that evidence.
It cannot replace the Milestone 13.1 platform cutover or enter Milestone 14 as
an unmeasured compatibility lane.
