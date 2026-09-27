# Milestone 12 Engineering Spec: Aspect-Causal Invalidation

> **Status:** Completed
>
> **Closeout:** [milestone-12-closeout.md](./milestone-12-closeout.md)
>
> **Architecture parents:**
> - [signal_architecture2.md](./signal_architecture2.md), especially `S2.2`,
>   `S9.16.3`, and `S9.16.6`
> - [s9_16_acceptance_map.md](./s9_16_acceptance_map.md)
> - [`worth-foundational` ownership contract](../../crates/worth-foundational/README.md)
> - [`worth-proof` ownership contract](../../crates/worth-proof/README.md)
>
> **Inherited closeouts:**
> - [milestone-d-closeout.md](./milestone-d-closeout.md)
> - [milestone-11-closeout.md](./milestone-11-closeout.md)
>
> **Successor:** [milestone-13-plan.md](./milestone-13-plan.md)

## 1. Goal And Roadmap Placement

Milestone 12 makes aspect invalidation causally correct across every dependency
hop in `worth-signal`.

The milestone reopens the semantic portion of `S9.16.3`. The current runtime
can narrow many direct subscribers, but its transitive frontier copies the
original seed aspect through every reachable descendant. That is not a lawful
interpretation of aspect truth: a Signal aspect is a producer-local output
slot. The same numeric slot on another producer has no inherited meaning.

Milestone 12 establishes one governing rule:

> A root mutation may create unresolved recompute work. Every resolved
> downstream aspect fact comes only from the immediate dependency's committed
> semantic output delta.

This milestone owns semantic truth, cause freshness, condition safety, and
recovery of pending invalidation authority. It may retain broad structural
candidate discovery temporarily, but structural reachability may carry only
unresolved revalidation. Milestone 13 replaces broad discovery with
locality-first direct-hop scheduling and certifies realized cost.

Milestone 12 expands and certifies the existing fintech financial world while
the semantic authority is implemented. There is no later invalidation-
certification milestone.

## 2. Current Boundary And Inherited Guarantees

The current invalidation path conflates four facts:

1. a caller declares that a source aspect may have changed
2. a source evaluation or installed change commits dependency-visible output
3. a direct dependency edge matches a committed producer change
4. a descendant is structurally reachable from an earlier node

Only the third fact proves a resolved consumer-local invalidation cause. The
first fact schedules work, the second fact mints producer authority, and the
fourth fact can establish only an unresolved ordering obligation.

Today, `FrontierWavePlan.aspect` and `TransitiveFrontierRoot.aspect` assign one
root slot to a whole wave. `execute_transitive_wave` stores that slot in every
reachable descendant's `dirty_aspects`, and ordinary, installed, and async
condition admission consume the aggregate as though it were resolved local
truth.

The smallest hostile graph is:

```text
source produces PRICE
  -> middle consumes PRICE and produces RISK
     -> matched leaf consumes RISK and uses AspectFilter(RISK)
     -> unmatched leaf consumes ALERT and uses AspectFilter(ALERT)
```

The inherited path can place `PRICE` on both leaves. The matched leaf may then
defer even after the middle committed `RISK`, while the unmatched leaf's result
depends on accidental numeric slot reuse.

The repair must preserve these accepted guarantees:

- `Clean | MaybeStale | Dirty` remains the public graph lifecycle vocabulary
- direct dependency edges retain producer, aspect, and optional partition or
  detail scope
- dependency snapshots remain the last-evaluated consumer basis
- cycle preflight rejects unlawful topology before commit
- transactions suppress observation and durable change on rollback
- Milestone 11 observation remains commit-bounded and derives from runtime
  truth rather than becoming invalidation authority
- deterministic ordering, diagnostics-tier truth, async lifecycle truth,
  branch isolation, checkpoint restore, and replay remain intact
- installed Query/Bridge lowering remains the only portable-semantics-to-
  Signal-slot boundary

Code is evidence of present reality, not authority over the destination.

## 3. Adversarial Constraint And Decisive Financial Courtroom

### 3.1 Adversarial Constraint

The runtime must remain correct when an intermediate producer translates
aspects, multiple producer commits accumulate while a consumer is blocked,
different consumers apply different comparators to the same delta, dependency
topology is removed and recreated with the same shape, and pending work crosses
transaction, branch, checkpoint, replay, diagnostic-tier, async, serial,
parallel, and WASM-capable execution boundaries.

The plausible dishonest implementation is not merely root-aspect copying. It
is any implementation that:

- renames reachability as a cause
- stores only one node-level aspect mask
- crosses changed aspects with one undifferentiated scope union
- applies a consumer comparator while minting the producer delta
- keeps only the latest producer's changed scopes while an older cause remains
  pending
- treats a same-shaped rewire as the same dependency epoch
- publishes a delta before node state and dependency snapshot commit are both
  owned
- reconstructs cause authority from diagnostics, replay presentation, or a
  fresh scan of current graph state

### 3.2 Production Entry And Composition Root

The decisive proof expands the existing production-shaped fintech world under
`crates/worth-signal/src/tests/domains/fintech`. Scenarios enter through the
real runtime transaction, mutation, target-read, evaluation/apply, branch,
checkpoint, and replay composition roots. A test-only invalidation API,
alternate graph owner, or generic graph-only closeout world is forbidden.

Each scenario owns:

- a causally complete financial baseline issued by the existing world compiler
- one named market, portfolio, partition, topology, condition, or branch delta
- the exact financial outputs and dependency snapshots that must commit
- an independently authored semantic necessity set
- the plausible defect it convicts
- a positive/negative twin or mutation probe
- scenario identity, seed, scale tuple, policy tuple, diagnostic tier, cold or
  warm posture, and exact failing mutation step

### 3.3 Required Scenarios

| Scenario | Hostile sequence and required truth | Defect convicted |
|---|---|---|
| `quote_to_risk_aspect_translation` | A primary-market `PRICE` change is normalized and repriced into `RISK`; `AspectFilter(RISK)` runs and an `AspectFilter(ALERT)` twin does not | root-aspect copying, reachability-as-change, or filter evaluation before local cause resolution |
| `heterogeneous_consumer_comparators` | One committed price delta reaches exact, tolerance-5, and installed-comparator consumers; a delta of 2 reaches only the exact consumer and a delta of 6 reaches all lawfully affected consumers | applying one consumer's comparator while minting the producer delta or globally suppressing a multi-consumer change |
| `tolerance_suppressed_repricing` | A within-tolerance input move leaves the producer's committed semantic price and downstream risk unchanged; a larger twin commits changed price and risk | publishing candidate versions after semantic suppression, broad cleanup as correctness, or proving unchanged output from missing execution |
| `producer_local_factor_slot_collision` | FX and curve producers reuse the same numeric slot while retaining distinct producer and dependency identity; only the economically affected path changes | flattening causes into a node-level aspect mask or treating equal slots as shared meaning |
| `partitioned_curve_bucket_bump` | While the downstream valuation and threshold-gated risk consumer remain unevaluated, two committed curve changes affect distinct rates details; after immediate dependencies settle, the exact accumulated detail union reaches the valuation and the financial gate is admitted while unrelated details remain unchanged | crossing every changed aspect with every region, retaining only the latest scope, or widening detail to partition scope |
| `gated_repricing_release` | An authentic financial delta-threshold consumer stays suppressed for the small twin and is admitted for the large twin; domain-neutral scenario controls separately prove that pending dependency truth precedes on-demand, temporal, custom, installed, and async policies | evaluating conditions against ancestor evidence, treating empty or mismatched masks as resolved, or stranding pending work |
| `instrument_dependency_rewire` | An instrument changes factor/model dependency, removes and recreates a same-shaped edge, rejects a cycle, and then evaluates under the new dependency revision | accepting stale causes, using storage-handle coincidence as semantic freshness, or publishing causes from rejected topology |
| `branch_shock_restore_replay` | Main and analysis branches accumulate different pending shocks, capture and restore checkpoints, replay the same trace, and preserve async audit lifecycle independently | omitting causes from rollback/checkpoint authority, leaking branch-local causes, or reconstructing cause from diagnostics/replay presentation |

The partition scenario must include at least two producer commits before the
consumer advances its dependency snapshot. The rewiring scenario must remove
and re-add an identical `(producer, aspect, scope)` edge under a new dependency
revision. Comparator scenarios must use one producer with concurrent consumers;
separate producers do not expose global suppression.

The financial world uses a delta threshold because it has real economic
meaning for accumulated repricing. On-demand request mode, temporal wake
policy, custom host policy, installed portable policy, and async lifecycle are
runtime orchestration axes rather than financial formulas. Their required
pending-before-policy twins therefore use named production-valid Signal
scenarios against the same `NodeInvalidationInput` owner; the certification
must not invent fake financial semantics merely to place those axes in the
financial definition.

### 3.4 Independent Oracles

Two independent oracles are mandatory:

1. `FreshFinancialRecompute` reconstructs the declared financial world from
   authoritative market and portfolio inputs and evaluates without dirty
   masks, pending cause sets, incremental frontier planning, incremental
   condition classification, suppression routing, or ready queues.
2. `FinancialNecessityManifest` enumerates economically required evaluations
   and committed outputs from scenario-owned positions, factor subscriptions,
   aspect translations, comparator policies, and partition ownership. It may
   not call production routing, cause admission, or scheduling logic.

For Milestone 12, "semantic work" means nodes that evaluation admits and the
output deltas they commit. Broad candidate visits and edge checks remain a
separate structural-cost verdict owned by Milestone 13. This distinction
prevents Milestone 12 from claiming locality it does not yet establish.

Every scenario must agree with `FreshFinancialRecompute` on committed financial
outputs and dependency snapshots, and with `FinancialNecessityManifest` on
semantic work and necessary dependency causes. Scenario-owned independent
expectations judge lifecycle and pending-cause posture. An oracle may overlap
another verdict, but the production runtime is never the source of its expected
answer. Deleting, bypassing, weakening, stale-reusing, or globally flattening
the disputed authority must turn at least one named scenario red.

### 3.5 Focused Boundary Proofs

The financial courtroom is supplemented, not replaced, by focused proofs for
local contracts:

- every dependency-cause binding axis has an identical case and one-axis-drift
  rejection twin
- producer output-equivalence and consumer dependency-comparator configurations
  cannot affect one another's decision boundary
- legacy output-identity configuration canonicalizes to the explicit split
  policy without semantic drift
- `OutputChange::Unchanged` with a non-empty semantic aspect change is rejected
  before publication
- a prepared apply packet cannot mint performed output-commit evidence
- failure after comparison, after cause admission, after state preparation, or
  before canonical publication exposes no producer version, dependency-
  snapshot update, committed delta, consumer cause, or observation
- destroying the derived dirty mask and rebuilding it from the canonical cause
  set produces the same mask and scoped aggregate
- a serialized cause basis is readmitted against the restored graph; raw
  serialized bytes cannot reopen runtime authority

## 4. Product Decision Lock

### 4.1 Signal Aspects Are Producer-Local Runtime Meaning

`worth-signal::Aspect`, `AspectMask`, and `AspectVersion` remain compact
runtime-local storage and comparison types. An aspect is interpreted against
the producer whose output contract declares it. Equal numeric slots on distinct
producers do not imply shared semantic meaning.

`worth-foundational::AspectContract`, `AspectBinding`, portable masks, and
authoritative change kinds remain bridge-facing portable meaning. They do not
enter internal Signal routing and cannot construct Signal invalidation
authority. Existing installed lowering capabilities remain the lawful bridge.

### 4.2 Root Seeds And Committed Deltas Are Different

The public `mark_changed`, `mark_changed_with_regions`, and batch equivalents
declare source-local recompute intent. They create a `SourceRecomputeSeed`,
transition the named source into required work, and may mark descendants only
as unresolved revalidation candidates. They do not manufacture a committed
producer version or resolved dependency cause.

An entry point that has already advanced dependency-visible Signal versions,
including installed aspect changes, emits a
`CommittedProducedAspectDelta` carrying a source-entry origin through the same
output-commit authority used by evaluated producers. It may admit direct
dependency causes immediately.

The root exception ends at the source. No root seed may be copied into a
descendant cause, delta, dirty mask, condition input, replay artifact, or
certification report.

The current internal `SemanticBatchCommit` name overclaims this boundary: the
returned value proves admission of root change declarations, not output or
transaction commit. It is replaced by `SourceRecomputeAdmission`. The public
facade exposes the honest `ChangeBatchAdmission` name. Existing
`ChangeBatchCommit` / `SemanticBatchCommit` aliases may remain only as
deprecated compile-compatible names carrying the same weaker admission
semantics, with a documented replacement and no independent constructor or
authority lane.

### 4.3 Output Commit Is The Semantic Authority Boundary

Evaluation produces candidate output. Comparison classifies semantic output.
Apply prepares node state and dependency snapshot changes. Only successful
publication of the complete commit mints a committed produced delta.

The canonical decision is:

```rust
pub enum SemanticOutputCommitDecision {
    Unchanged {
        retained_version: AspectVersion,
        reason: SemanticOutputEquivalence,
    },
    Changed {
        committed_version: AspectVersion,
        changes: NonEmptyCanonicalAspectChangeSet,
    },
}
```

When output is semantically unchanged, dependency-visible semantic versions do
not advance and no produced delta exists. Candidate versions may appear only in
policy-governed diagnostics. The old behavior of publishing a new semantic
version and then walking the subscriber closure to clean it is not an ordinary
correctness path.

When output is changed, the runtime compares the previous and candidate
versions only across the node contract's declared produced aspects. Each
differing aspect becomes one change entry. `OutputChange` describes artifact
continuity; it cannot create, erase, or relabel an aspect change. An explicit
`OutputChange::Unchanged` with a non-empty semantic change set is a typed
`OutputCommitContractViolation` and fails before publication.

Producer equivalence is configured independently from dependency comparison:

```rust
pub enum OutputEquivalencePolicy {
    ExactAspectVersion,
    AspectVersionTolerance { epsilon: u64 },
    OutputIdentity,
    Custom { key: OutputEquivalencePolicyKey },
    Installed { identity: InstalledOutputEquivalenceIdentity },
}
```

`NodeEvaluationConfig.output_equivalence` is the sole semantic policy input to
`SemanticOutputCommitDecision`. `NodeEvaluationConfig.comparator` remains the
consumer-facing dependency comparator described by its existing contract; it
cannot suppress the node's own produced delta. The performance
`EquivalenceContract` becomes a derived validation/report projection of the
two explicit policies, not a second runtime truth source. The existing
`.equivalence(...)` builder surface may state a required performance contract,
but build and restore reject disagreement with the semantic fields; it never
selects runtime suppression behavior.

The caller API makes the split visible:

```rust
graph
    .node()
    .dependency_comparator(VersionComparatorPolicy::Tolerance { epsilon: 5 })
    .output_equivalence(OutputEquivalencePolicy::OutputIdentity);
```

For source compatibility, `.comparator(...)` remains a deprecated alias of
`.dependency_comparator(...)`, `.tolerance(...)` remains dependency-only, and
`.output_identity()` becomes an alias of
`.output_equivalence(OutputEquivalencePolicy::OutputIdentity)`. New
configuration never sets both policies through one call.

Checkpoint/config upgrade is deterministic. A legacy image with no
`output_equivalence` field defaults to `ExactAspectVersion`, except that the exact
legacy combination of `comparator = OutputIdentity` and an
`EquivalenceContract` triplet
`identity_basis = OutputIdentity`,
`suppression_basis = OutputIdentityAndComparator`, and
`comparator_basis = OutputIdentity` is
upgraded to two explicit fields: dependency comparator `OutputIdentity` and
output equivalence `OutputIdentity`. The upgrade is recorded in restore
diagnostics and produces the same canonical current image as an explicitly
authored new configuration. No runtime heuristic may infer producer
equivalence from a consumer comparator after decode.

Installed conditional lowering already declares `dependency_comparator` and
`output_comparator` independently. Milestone 12 preserves that good contract:
installation lowers the first into consumer dependency comparison and the
second into `OutputEquivalencePolicy`, validates the installed identity's role,
and never assigns the output comparator to the dependency-comparator field.
Exact, tolerance, output-identity, custom, and runtime-resolved installed
output policies must all reach the producer commit boundary with their meaning
intact.

Prepared, compared, or suppression-classified packets are non-authorities.
Serial apply, grouped parallel apply, and serial/WASM-capable execution must
reach one canonical publication function. Before that function, the runtime
resolves every fallible output-equivalence and direct consumer-comparator
decision, validates current dependency revisions, and reserves cause-store
capacity. Its private commit packet owns the producer state transition, the
producer's evaluated dependency-snapshot update, the committed delta record,
all admitted current-edge cause-set mutations, derived-cache rewrites, and the
commit ordinal. Publication applies that packet as one graph-owned, non-
fallible state transition. No observer or subsequent work sees a producer
version without its admitted causes, or a cause without its producer version.

A private Signal-owned wrapper over `worth_proof::Performed` is minted only
after that full state transition, never during preparation. Observation,
diagnostics, replay presentation, and successor scheduling consume the
performed result; they are not members of the atomic truth mutation and cannot
roll it back or widen it.

### 4.4 Output Changes Preserve Aspect-Scope Correlation

A produced delta is a canonical non-empty set of per-aspect changes:

```rust
pub struct ProducedAspectChange {
    aspect: Aspect,
    previous_version: u64,
    committed_version: u64,
    changed_scopes: PartitionScopeSet,
}

pub struct ProducedAspectDelta {
    producer: NodeId,
    output_commit_ordinal: OutputCommitOrdinal,
    committed_output_version: AspectVersion,
    changes: NonEmptyCanonicalAspectChangeSet,
    scope_precision: ScopePrecision,
}

pub enum ScopePrecision {
    ExactAspectScopes,
    ConservativeLegacyUnion,
}
```

`NodeEvaluationResult` gains an exact aspect-scoped changed-region authoring
surface. Existing unqualified `changed_regions` remains a compatibility input:
when exactly one produced aspect changed, the correlation is unambiguous and
may lower to `ExactAspectScopes`; when multiple aspects changed, its region union
is conservatively attached to every changed aspect and the delta is marked
`ConservativeLegacyUnion`. That multi-aspect lane is correctness-preserving but
cannot satisfy Milestone 13's exact locality certification.

The Milestone 12 partition courtroom must use `ExactAspectScopes`. Empty scopes
mean whole-aspect change, not unknown detail. Scope precision is retained in
summaries so a compatibility fallback cannot masquerade as exact narrowing.

The intended caller-facing authoring path is explicit at the aspect boundary:

```rust
let result = NodeEvaluationResult::from_version(
    AspectVersion::from_updates([(RISK, next_risk_version)]),
)
.with_changed_aspect_region(
    RISK,
    ChangedRegion {
        partition: PartitionToken::new("rates"),
        detail: Some("bucket-0".to_owned()),
    },
);
```

The builder records exact aspect/scope correlation. The existing
`.with_changed_region(region)` method remains the visibly conservative
compatibility surface and produces `ConservativeLegacyUnion` whenever more
than one aspect changed.

### 4.5 Consumer Comparison Happens At Dependency Admission

The producer delta records producer truth before consumer policy. For each
current direct dependency edge, Signal compares the consumer snapshot's cached
version with the committed producer version using that consumer's dependency
comparator.

Therefore:

- one producer delta may create a cause for one consumer and no cause for
  another
- tolerance, custom, and installed dependency comparators do not suppress the
  producer delta globally
- producer output-equivalence policy and consumer dependency-comparison policy
  remain separate typed decisions
- `VersionComparatorPolicy`, including its installed variants, is resolved at
  the consumer edge; `OutputEquivalencePolicy` is resolved at the producer
  commit boundary
- `NodeBuilder::comparator` and the legacy serialized `comparator` field cannot
  silently regain producer-suppression authority
- the direct cause retains the exact immediate dependency edge and narrowed
  aspect-correlated scopes that justified admission

### 4.6 Resolved Causes Are Bound To Dependency Freshness

A resolved cause retains both consumer and producer identity. Its mandatory
stable key and binding axes are:

```rust
pub struct DependencyCauseKey {
    graph_instance: GraphInstanceId,
    consumer: NodeId,
    dependency_revision: DependencyRevision,
    producer: NodeId,
    aspect: Aspect,
    edge_scope: Option<PartitionSubscription>,
}

worth_proof::binding_axes! {
    pub struct DependencyCauseBindingAxes {
        pub graph_instance: GraphInstanceId => GraphInstance,
        pub consumer: NodeId => Consumer,
        pub dependency_revision: DependencyRevision => DependencyRevision,
        pub producer: NodeId => Producer,
        pub aspect: Aspect => Aspect,
        pub edge_scope: Option<PartitionSubscription> => EdgeScope,
        pub cached_version: u64 => CachedVersion,
        pub output_commit_ordinal: OutputCommitOrdinal => OutputCommitOrdinal,
        pub committed_version: u64 => CommittedVersion,
    }
    drift pub enum DependencyCauseBindingDrift;
}
```

The Signal-owned resolved cause stores serializable axes and uses
`worth_proof::Binding<DependencyCauseBindingAxes>` at live admission. Raw Proof
carriers are not serialized and `worth-proof` does not gain a serde dependency.
Restore reconstructs and revalidates the live binding from Signal-owned axes.

Every dependency-set mutation advances a logical `DependencyRevision`, even if
the resulting edge set is byte-for-byte equal to an earlier shape. Storage
handles may accelerate the live check but cannot replace the logical revision,
because compaction and restore may rewrite storage representation.

### 4.7 Pending Revalidation Is Not Changed Output

The runtime represents unresolved structural obligation separately:

```rust
pub struct PendingDependencyRevalidation {
    consumer: NodeId,
    dependency_revision: DependencyRevision,
    unresolved_edges: CanonicalDependencyEdgeKeySet,
}

pub enum NodeInvalidationInput {
    Pending(PendingDependencyRevalidation),
    Resolved(CanonicalDependencyCauseSet),
    ResolvedNoChange(ResolvedDependencyBasis),
}
```

Pending revalidation authorizes ordering and upstream evaluation only. It does
not authorize a dirty aspect, condition decision, subscriber cause, output
delta, or diagnostic statement that output changed.

When an upstream producer commits no semantic delta, the corresponding pending
edge resolves to no-change. Once all unresolved edges settle and no resolved
cause remains, the consumer becomes `ResolvedNoChange`; that conclusion may
continue through structurally pending descendants without evaluating an aspect
condition or manufacturing a zero-width cause.

Milestone 12 may use broad reachability to create pending candidates so a clean
requested descendant can discover a dirty ancestor. Milestone 13 must replace
that breadth. The pending representation is deliberately independent of the
scheduling mechanism so that replacement is additive.

### 4.8 Conditions Consume One Resolved Projection

Ordinary conditions, installed conditional execution, and async-capability
admission consume the same `NodeInvalidationInput` projection.

- `Pending` evaluates no aspect, threshold, custom, or installed condition; it
  first schedules or waits for the named upstream revalidation
- `Resolved` derives the node-local dirty mask and scope aggregate from the
  canonical immediate-dependency cause set, then applies condition policy
- `ResolvedNoChange` may validate the node clean without compute
- temporal, on-demand, async lifecycle, previous-value, and custom policies
  remain orthogonal typed axes and cannot reinterpret cause truth

The current rule that an empty dirty mask may be treated as eligible remains
valid only inside `Resolved` or `ResolvedNoChange`. It is never a shortcut for
`Pending`.

### 4.9 Canonical Cause Storage Carries Pending Operational Proof

The committed output-delta record, current dependency revision, and consumer
dependency-snapshot basis are the canonical authority chain. The cause store is
the sole proof-carrying operational projection admitted from that chain until
the consumer commits a new dependency snapshot or the cause is lawfully
invalidated by rewire, rollback, branch replacement, or node retirement. It is
rebuildable from canonical mutation, topology, and output-commit records, but
not from only the latest producer diagnostic artifact or a scan of current
versions when multiple commits or detail scopes have accumulated.

Signal therefore owns a canonical graph-level cause-set store. Node hot state
contains a compact `PendingCauseSetId`, not an inline cause vector. The store:

- canonicalizes by `DependencyCauseKey`
- keys at most one pending entry by `DependencyCauseKey` for each current
  canonical dependency edge
- coalesces repeated commits by rebuilding the live binding with the newest
  commit ordinal/version while retaining the consumer snapshot's cached version
  and the exact union of still-pending aspect-correlated scopes
- preserves distinct immediate dependencies even when aspects match
- supports empty, replace, and narrow update forms
- participates in graph compaction and generational stale-handle rejection
- serializes Signal-owned axes and scope data through checkpoint images

Cause-store cardinality is bounded by the consumer's current canonical
dependency-edge count, not producer commit count. Scope accumulation is
normalized to the dependency edge: unscoped edges become whole-aspect causes,
whole-partition edges retain one partition scope, and detail edges retain their
declared detail. Per-commit seed and lineage history stays out of hot cause
state.

Canonical operational summaries must report cause entries inserted, updated,
resolved, removed by rewire, rejected as stale, restored, and the maximum
retained pending entries. These counters establish storage/lifecycle honesty;
Milestone 13 owns traversal slopes and Foundational counter-backed locality
receipts.

`dirty_aspects` and dirty partition/detail aggregates remain compact derived
caches. They authorize nothing independently. Clearing and rebuilding them
from the current cause set must be exact. Diagnostics, traces, lineage,
frontier summaries, and seed references derive from the operational artifacts
and remain cold or policy-bounded non-authorities.

### 4.10 Transaction, Branch, Checkpoint, Replay, And Observation Law

Cause-state mutation is part of the same reversible transaction patch as node
state, dependency snapshots, dependency rewiring, subscriber topology, and
runtime artifacts.

- rollback restores the previous cause-set handle and store ownership and
  publishes no observation
- branch fork receives isolated cause state through the existing governed
  branch lifecycle
- checkpoint captures canonical cause axes, dependency revisions, commit
  ordinals, and derived-cache inputs
- restore readmits serialized cause bases against restored graph identity and
  topology before any condition or evaluation may consume them
- replay reconstructs cause authority from canonical mutation and output-commit
  records, never from diagnostic presentation
- Milestone 11 observation derives `Touched`, `Recomputed`, and
  `MeaningfulChange` from the committed transaction result; pending or rolled-
  back cause mutation is not observable delivery truth

Restored or replayed diagnostics may explain a cause but cannot mint or repair
one. A missing authoritative cause record is a typed reconstruction failure,
not permission to scan current graph state and guess.

### 4.11 Foundational And Proof Adoption

Milestone 12 uses existing lower-level contracts without changing their
ownership:

- `worth-proof::Binding` enforces complete dependency-cause comparison axes
- a private Signal wrapper over `worth-proof::Performed` distinguishes an
  applied and published output commit from permission or preparation to commit
- owner-specific Signal types and private constructors remain the governed
  surface; a caller-selected `AuthorityMarker` opens no door
- `worth-foundational` canonicalization establishes certification case and
  report identity; debug strings and ad hoc hashes are forbidden
- Foundational provenance and lineage vocabulary may package cold exported
  explanation, never operational admission
- Foundational performance claims and counter-backed receipts are reserved for
  Milestone 13's measured locality boundary

No new `worth-proof` or `worth-foundational` API is required. Milestone 12 must
not add a generic invalidation graph, runtime cause store, Signal slot type, or
Signal-specific performance work class to either crate.

### 4.12 Domain And Platform Neutrality

Production types use only graph, node, dependency, aspect, scope, version,
commit, and readiness vocabulary. Financial names live only in the fintech
test domain and its durable documentation.

Cause truth is independent of worker identity, thread count, wall clock, or
native-only primitives. Commit and dependency ordinals are Signal-owned logical
values. Serial, existing parallel-feature, and WASM-capable builds obey the same
authority and publication sequence.

## 5. Required Proof-Bearing Forms And Authority Direction

The implementation must establish canonical equivalents of:

```rust
pub struct SourceRecomputeSeed { /* unresolved root intent */ }
pub struct SourceRecomputeAdmission { /* admitted root seed batch */ }
pub struct ProducedAspectChange { /* one producer-local aspect and scopes */ }
pub struct ProducedAspectDelta { /* canonical committed producer artifact */ }
pub struct CommittedProducedAspectDelta { /* private performed-effect wrapper */ }
pub struct DependencyCauseKey { /* stable current-edge identity */ }
pub struct DependencyCauseBindingAxes { /* every freshness and identity axis */ }
pub struct ResolvedDependencyInvalidationCause { /* one immediate edge */ }
pub struct CanonicalDependencyCauseSet { /* retained consumer causes */ }
pub struct PendingDependencyRevalidation { /* unresolved immediate edges */ }
pub enum NodeInvalidationInput { Pending, Resolved, ResolvedNoChange }
pub enum OutputCommitContractViolation { /* typed contradictory output */ }
```

Authority is fixed:

```text
host mutation declaration
  -> SourceRecomputeSeed
  -> source required work + unresolved descendant revalidation
  -> evaluation candidate
  -> producer semantic comparison
  -> atomic output and dependency-snapshot publication
  -> CommittedProducedAspectDelta
  -> current direct dependency edge + consumer snapshot/comparator admission
  -> ResolvedDependencyInvalidationCause
  -> canonical consumer cause set
  -> resolved condition/planner input
```

An installed source change that already advanced Signal's dependency-visible
version enters at `CommittedProducedAspectDelta`. It does not bypass commit
authority.

| Form | Constructor | Authorizes | Cannot authorize | Consumer |
|---|---|---|---|---|
| `SourceRecomputeSeed` | transaction mutation owner | source work and unresolved candidate discovery | changed descendant aspect or condition verdict | planner/revalidation |
| `SourceRecomputeAdmission` | root mutation admission owner | confirms canonical root seed admission | output commit, transaction commit, or descendant cause | caller summary/transaction staging |
| `CommittedProducedAspectDelta` | canonical output publication only | direct edge admission | transitive descendant meaning without another commit | causality router |
| `ResolvedDependencyInvalidationCause` | current edge + snapshot + comparator admission | one consumer's resolved local cause | another consumer, revision, edge, or commit | cause store and conditions |
| `PendingDependencyRevalidation` | structural discovery against current revision | upstream ordering/revalidation | dirty aspect, compute suppression, or clean state | planner |
| `CanonicalDependencyCauseSet` | graph cause store | derived dirty aggregate and semantic admission | portable meaning or cross-branch use | planner/evaluator |
| certification reports | fintech certification owner | milestone evidence only | runtime mutation, cause repair, or production admission | QA/closeout |

## 6. Architectural Destination

Milestones 12 and 13 commit to this destination topology. Bracketed labels are
part of the specification: implementation planning must preserve the stated
movement and ownership rather than creating parallel lanes.

```text
crates/worth-signal/src/
  data/
    proof.rs                                      [moved to proof/mod.rs]
    proof/
      mod.rs                                      [moved stable internal facade]
      invalidation/                               [created operational proof family]
        mod.rs                                    [created facade]
        source_seed.rs                            [created/replaces seed semantics]
        output_delta.rs                           [created committed delta vocabulary]
        dependency_cause.rs                       [created cause and binding axes]
        revalidation.rs                           [created pending/resolved input]
        plan.rs                                   [moved/replaces invalidation_plan.rs]
        execution.rs                              [moved/replaces invalidation_execution.rs]
      invalidation_admission.rs                   [removed after migration]
      invalidation_frontier.rs                    [removed after migration]
      invalidation_plan.rs                        [removed after migration]
      invalidation_execution.rs                   [removed after migration]
      dirty_batch.rs                              [modified; commit summary replaced by admission]
    output/
      equivalence.rs                              [created producer-only semantic policy]
      aspect_changes.rs                           [created exact aspect/scope authoring]
      evaluation.rs                               [modified compatibility entry]
    conditional_execution/
      condition_resolution.rs                     [modified shared input consumer]
      contract.rs                                 [modified preserve split comparator roles]
      dependency_versions.rs                      [modified resolved-cause admission]
    graph/
      construction/
        node_builder.rs                           [modified explicit split-policy DX]
      storage/
        invalidation_causes/                      [created live authority storage]
          mod.rs                                  [created storage facade]
          handles.rs                              [created generational handles]
          cause_sets.rs                           [created canonical set store]
          checkpoint.rs                           [created serialization/readmission basis]
          compaction.rs                           [created handle rewrite ownership]
      runtime/
        effect/
          output_commit.rs                        [created canonical commit authority]
          suppression.rs                          [ordinary broad cleanup removed/reduced]
    node/
      condition.rs                                [modified split comparator/equivalence config]
      entry/
        layout.rs                                 [modified compact cause handle/cache]
        checkpoint.rs                             [modified cause checkpoint image]
      checkpoint_image.rs                         [modified serialized cause basis]
  logic/
    invalidation/
      mod.rs                                      [existing stable internal facade]
      subscription.rs                             [existing edge-match mechanics]
      causality/                                  [created semantic authority]
        mod.rs                                    [created causality facade]
        source_seed.rs                            [created root intent lowering]
        dependency_admission.rs                   [created consumer-policy admission]
        revalidation.rs                           [created pending resolution]
        cause_aggregation.rs                      [created canonical coalescing]
      routing/
        mod.rs                                    [existing mechanism boundary]
        planning.rs                               [modified pending candidates only]
        application.rs                            [modified; no copied aspect authority]
        counters.rs                               [existing/M13 realized expansion]
        evidence.rs                               [derived summaries only]
        seeds.rs                                  [modified root seed preparation]
      scheduling/                                 [Milestone 13 committed successor]
        mod.rs                                    [M13 stable internal facade]
        ready_work.rs                             [M13]
        topological_order.rs                      [M13]
    evaluation/
      condition/
        invalidation_input.rs                     [created shared resolved projection]
        resolver.rs                               [modified ordinary conditions]
      engine/
        apply.rs                                  [modified serial commit entry]
        prepared_apply/
          parallel.rs                             [modified parallel commit entry]
    transaction/
      runtime/
        state/
          async_capability/
            admission.rs                          [modified shared input consumer]
        transaction/
          transaction_mutation.rs                 [modified seed/cause rollback capture]
  tests/
    domains/
      fintech/
        world/                                    [created authoritative test-domain meaning]
          mod.rs                                  [created stable financial-world facade]
          definition.rs                           [created immutable scenario input authority]
          market_inputs.rs                        [created fixed-point market/curve/vol/FX ledger]
          positions.rs                            [created instruments, quantities, ownership, subscriptions]
          reference_finance.rs                    [created locally certified financial formulas]
          semantic_projection.rs                  [created economic-result to Signal revision mapping]
          baseline.rs                             [created sealed causally complete baseline]
          compiler.rs                             [created production-runtime world projection]
        model.rs                                  [removed after meaning/generation split]
        world_assembly.rs                         [removed after compiler cutover]
        fixture.rs                                [modified runtime-world facade consumes definition]
        fixture/
          market_world.rs                         [modified value/revision-separated source projection]
          portfolio_world.rs                      [modified typed positions and exact contracts]
          risk_world.rs                           [modified book/desk/scenario projection]
        market_state.rs                           [modified ledger-to-runtime source admission only]
        invalidation/                             [created financial scenario family]
          mod.rs
          quote_to_risk_aspect_translation.rs
          heterogeneous_consumer_comparators.rs
          tolerance_suppressed_repricing.rs
          producer_local_factor_slot_collision.rs
          partitioned_curve_bucket_bump.rs
          gated_repricing_release.rs
          instrument_dependency_rewire.rs
          branch_shock_restore_replay.rs
          sparse_book_fanout.rs                    [M13 committed successor]
          partitioned_curve_universe.rs            [M13]
          convergent_factor_batch.rs               [M13]
          dense_market_close.rs                    [M13]
          portfolio_dependency_churn.rs            [M13]
          branch_restore_locality_replay.rs         [M13]
        certification.rs                          [existing stable domain facade]
        certification/
          workflow/                               [existing family migrated without semantic change]
            adapter.rs
            artifact_matrix.rs
            independent_oracle.rs
            scenario.rs
            session.rs
          invalidation/                           [created financial certification family]
            mod.rs
            financial_scenario.rs
            fresh_recompute.rs
            necessity_manifest.rs
            equivalence.rs
            causality_run.rs
            locality_run.rs                       [M13]
            cost_slope.rs                         [M13]
            strategy_decision.rs                  [M13]
```

### 6.1 Structural Axes And Dependency Direction

- `data/proof/invalidation` owns immutable operational phase artifacts. It does
  not own live storage, traversal, financial scenarios, or certification runs.
- `data/graph/storage/invalidation_causes` owns the runtime lifecycle of pending
  cause sets. It depends on proof vocabulary and exposes storage operations only
  through the graph owner.
- `data/graph/runtime/effect/output_commit` is the spatially locatable effect
  boundary that mints performed output commit authority. It does not route
  subscribers.
- `data/output/equivalence`, node configuration, and node construction expose
  producer equivalence separately from consumer comparison. Performance
  equivalence metadata validates that selection but cannot override it.
- `logic/invalidation/causality` consumes committed deltas and current
  dependency truth to derive causes. Routing applies those decisions but cannot
  mint or widen them.
- `logic/evaluation/condition/invalidation_input` is a projection adapter from
  causality into condition policy. It does not duplicate cause classification.
- `logic/invalidation/scheduling` is the committed Milestone 13 sibling. It
  consumes already-decided work and cannot acquire semantic authority.
- `tests/domains/fintech/invalidation` owns financial scenarios;
  `certification/invalidation` owns the independent oracles and sealed financial
  run. Production code remains domain-neutral.
- `tests/domains/fintech/world` owns authoritative synthetic financial meaning.
  Its compiler projects that meaning into the real Signal runtime, while
  certification recomputes from the immutable definition without importing the
  runtime fixture, routing, dirty state, or scheduler. Existing `model.rs` and
  `world_assembly.rs` do not survive as parallel financial truth owners.

The stable operational facade remains `logic/invalidation/mod.rs`; public
callers continue through `easy` and `facade`. Internal modules default to
`pub(crate)` or narrower. Cause constructors and performed commit witnesses are
private to their owners.

### 6.2 Forbidden Placement And Parallel Authority

Forbidden placements include:

- new invalidation semantics in `helpers`, `common`, generic planner files,
  diagnostics, replay presentation, async lifecycle, or public facades
- certification cases or sealed financial runs under production `data/proof`
- a second generic graph courtroom beside fintech
- financial vocabulary in production invalidation, storage, proof, scheduling,
  or public APIs
- a second cause store, dirty mask authority, compatibility router, or legacy
  transitive aspect lane after cutover
- raw `worth-proof::Artifact`, `Binding`, `Performed`, or generic authority
  markers exposed as the Signal facade
- Foundational portable aspects used as internal Signal slot identity

Focused graph fixtures may support one financial scenario or one genuinely
local transition proof. They cannot become an alternative closeout authority.

## 7. Ordered Implementation Phases

Milestone 12 is implemented as six numbered phases. A phase closes only when
its named authority and evidence are green; completing tasks or compiling an
intermediate type surface is not a phase gate.

| Phase | Name | Decisive gate |
|---|---|---|
| Phase 1 | Authentic Financial Courtroom And Red Control | a causally complete financial baseline and independent oracles expose the inherited `PRICE -> RISK` defect |
| Phase 2 | Output Contract And Owner-Specific Proof Forms | producer/consumer policies and proof-bearing forms are type-separated and drift-tested |
| Phase 3 | Canonical Cause Storage And Recovery Basis | pending causes have one recoverable graph-owned lifecycle |
| Phase 4 | Atomic Output Commit And Direct Cause Admission | output state, snapshots, deltas, and admitted causes publish atomically |
| Phase 5 | Planner, Condition, Scope, And Rewire Cutover | every condition and topology path consumes resolved immediate-dependency causality |
| Phase 6 | Branch Composition, Financial Certification, And Documentation | every lifecycle lane passes the sealed financial causality run |

### Phase 1 - Authentic Financial Courtroom And Red Control

What becomes true:

- `S9.16.3` is explicitly reopened and the current transitive claim is denied
- the financial world has authoritative economic inputs and positions rather
  than only financial names attached to version arithmetic
- the financial baseline and both independent oracle contracts are real,
  runnable, and independent of incremental invalidation
- every consumer, persistence path, and derived view of `dirty_aspects` and
  dirty scopes is inventoried

Architecture and proof:

- establish a test-domain `FinancialWorldDefinition` containing deterministic
  fixed-point market values, FX pairs, curve and volatility buckets, positions,
  quantities, currencies, maturity or bucket ownership, model/factor
  subscriptions, and book/desk ownership
- keep financial values distinct from Signal `AspectVersion` revisions;
  runtime outputs may project economic results into semantic versions, but no
  pricing or risk formula may treat a version counter as a financial amount
- establish one deterministic `FinancialSemanticProjection` that maps changed
  canonical economic results to producer-local Signal aspects and monotonic
  semantic revisions; unchanged economic results retain their prior revision,
  and the projection never decides routing or consumer necessity
- include at least one rates-sensitive, one FX-sensitive, and one volatility-
  sensitive position so the required aspect, partition, and producer-collision
  scenarios arise from real economic dependencies
- make the world compiler issue typed semantic handles and seal a
  `CausallyCompleteFinancialBaseline` only after required sources, output
  contracts, dependency snapshots, and initial financial truth are established
- implement the reusable core of `FreshFinancialRecompute` from authoritative
  market/portfolio inputs without Signal incremental state
- implement the reusable core of `FinancialNecessityManifest` from positions,
  subscriptions, aspect translations, comparator policies, and partition
  ownership without production routing or scheduling
- add the failing `quote_to_risk_aspect_translation` matched/unmatched twins
- add focused proof that the inherited root-aspect path fails for the intended
  condition mismatch
- freeze semantic-work versus structural-visit accounting

Phase gate:

- financial formula fixtures prove the reference world independently of Signal
  invalidation, and value/revision conflation is mechanically absent
- the sealed baseline is causally complete and reproducible by scenario, seed,
  financial scale, policy, diagnostic tier, and exact mutation step
- fresh recompute and necessity manifests disagree with the inherited runtime
  for the named defect
- no generic graph fixture is accepted as the phase authority

The next phase may trust the courtroom and the complete boundary inventory.

### Phase 2 - Output Contract And Owner-Specific Proof Forms

What becomes true:

- source recompute seeds, exact aspect changes, committed deltas, resolved
  causes, pending revalidation, and output contract violations are distinct
  types
- producer comparison and consumer dependency comparison are separate
  decisions

Architecture and enforcement:

- establish the `data/proof/invalidation` topology
- split producer output equivalence from consumer dependency comparison in
  node configuration, builder DX, runtime resolution, and checkpoint upgrade
- add exact aspect-scoped output authoring plus conservative legacy precision
- implement every `DependencyCauseBindingAxes` drift twin
- establish the private Signal wrapper over `Performed`
- keep constructors private and prevent summaries/diagnostics from constructing
  operational artifacts

Phase gate:

- contradictory output evidence is rejected before effects
- exact, tolerance, and installed dependency comparators cannot alter the
  producer's semantic commit decision, while output equivalence cannot alter a
  consumer's dependency admission
- legacy output-identity configuration restores into the same canonical split
  configuration and behavior as its explicitly authored replacement
- a prepared packet cannot mint committed delta authority
- `producer_local_factor_slot_collision` fails if producer or consumer identity
  is removed from the binding

The next phase may trust the immutable forms and their construction doors.

### Phase 3 - Canonical Cause Storage And Recovery Basis

What becomes true:

- pending operational causes have one graph-owned lifecycle and dirty masks are
  derived caches
- dependency revisions distinguish same-shaped rewires

Architecture and enforcement:

- establish the canonical cause-set store and generational handle
- integrate cause state with node images, transaction patches, branch state,
  checkpoint serialization, restore readmission, and compaction
- canonicalize repeated causes without flattening dependencies
- prove exact dirty mask/scope rebuild from the cause set

Phase gate:

- two pending detail changes survive coalescing and cache destruction
- rollback and checkpoint round trips preserve or remove causes exactly as the
  transaction outcome requires
- a stale cause or store handle is rejected after same-shaped dependency
  revision change

The next phase may trust persisted cause ownership and freshness axes.

### Phase 4 - Atomic Output Commit And Direct Cause Admission

What becomes true:

- serial, grouped-parallel, and serial/WASM-capable apply paths publish through
  one output-commit authority
- consumer comparators admit causes independently
- semantic suppression emits neither changed dependency-visible version nor
  downstream cause

Architecture and enforcement:

- derive per-aspect changes from previous versus committed produced versions
- prevalidate and reserve the complete producer-state, dependency-snapshot,
  delta-record, and direct-cause mutation packet before non-fallible publication
- publish the performed commit only after the complete packet is visible
- admit causes only through current direct edges and current consumer snapshots
- remove broad suppression cleanup as an ordinary semantic authority
- keep root recompute seeds unresolved until the source commits

Phase gate:

- `quote_to_risk_aspect_translation`,
  `heterogeneous_consumer_comparators`, and
  `tolerance_suppressed_repricing` agree with fresh recompute and necessity
  manifests
- failure at each pre-publication seam exposes no partial producer state,
  dependency snapshot, committed delta, cause, or observation
- serial and parallel-feature modes produce identical committed delta order

The next phase may trust committed deltas and direct resolved causes.

### Phase 5 - Planner, Condition, Scope, And Rewire Cutover

What becomes true:

- reachability carries only pending revalidation
- every condition family consumes the shared resolved invalidation input
- old copied-aspect and dirty-mask authority paths are removed

Architecture and enforcement:

- convert broad transitive application from aspect marking to unresolved edge
  revalidation
- resolve pending predecessors before ordinary, installed, temporal, on-demand,
  custom, or async condition policy
- preserve exact accumulated aspect/detail scopes
- reconcile dynamic dependency updates atomically with cause invalidation and
  cycle rejection

Phase gate:

- `partitioned_curve_bucket_bump`, `gated_repricing_release`, and
  `instrument_dependency_rewire` agree with both independent oracles
- deleting pending-state distinction or reintroducing aggregate-mask condition
  admission makes a named scenario fail
- no operational consumer reads `dirty_aspects` without a resolved-cause basis

The next phase may trust complete ordinary semantic cutover.

### Phase 6 - Branch Composition, Financial Certification, And Documentation

What becomes true:

- causal authority survives every inherited lifecycle and the milestone has one
  sealed financial verdict

Architecture and proof:

- extend `FreshFinancialRecompute` and `FinancialNecessityManifest` across the
  complete scenario/lifecycle matrix established by the earlier phases
- establish workflow and invalidation as sibling fintech certification families
- run deterministic scenario traces across transaction rollback, branch,
  checkpoint restore, replay, diagnostic tiers, async capability, serial,
  parallel-feature, and WASM-capable compilation lanes
- canonicalize scenario/report identity with Foundational artifacts
- seal `FinancialAspectCausalityCertificationRun`; reject missing, duplicate,
  stale, wrong-scenario, wrong-policy, wrong-revision, or mismatched-oracle
  evidence
- remove the legacy transitive aspect authority and obsolete proof files rather
  than retaining a compatibility lane

Phase gate:

- `branch_shock_restore_replay` and all prior scenarios remain green
- every assigned mutation probe turns its scenario red
- public examples and architecture documents describe the real committed path
- Milestone 13 can consume the committed delta/cause stream without changing
  semantic ownership

## 8. Documentation Deliverables

Milestone 12 must revise named durable documents; "update docs" is not an
accepted task.

| Audience | Authoritative document | Required content and executable check |
|---|---|---|
| runtime implementers | `signal_architecture2.md` | root seed versus committed delta, producer-local aspects, cause storage, condition ordering, persistence, and M12/M13 ownership; type/module names checked against production |
| acceptance and QA owners | `s9_16_acceptance_map.md` | exact M12 proof surfaces, negative space, financial sealed run, and semantic-versus-structural verdict split |
| test authors | `test-requirements.md` | financial-world authenticity floor, all named M12 scenarios, phase evidence map, independent oracle exclusions, binding-axis twins, multi-commit scope accumulation, and mutation sensitivity |
| standalone Signal callers | `crates/worth-signal/README.md` and condition guide | `mark_changed` as recompute intent, `ChangeBatchAdmission` versus deprecated commit aliases, producer output equivalence versus consumer dependency comparison, output version/aspect responsibilities, exact aspect-scoped regions, and an `A -> B` condition example compiled as a doctest or real example |
| portable integration authors | existing installed conditional operations documentation | portable Foundational aspect meaning lowers to Signal slots once; invalidation does not reinterpret portable identities internally |
| fintech certification maintainers | `crates/worth-signal/src/tests/domains/fintech/README.md` | authoritative market/position definition, financial-value versus Signal-revision boundary, baseline provenance, scenario catalog, oracle ownership, reproduction tuple, ordinary/scheduled lanes, and mutation probes |

The Foundational and Proof READMEs already state the required ownership and need
no prerequisite API expansion. After implementation, they may receive a narrow
cross-link to Signal as an adoption example only if the final code actually
uses the named contracts; they must not become milestone residue.

## 9. Must Ship, Must Preserve, And Mechanical Enforcement

Must ship:

- immutable authentic financial-world definition, deterministic semantic
  projection, and sealed causally complete baseline
- root recompute intent distinct from committed output delta
- one atomic commit door for dependency-visible semantic output
- separately configured producer output equivalence and consumer dependency
  comparison, including deterministic legacy upgrade
- aspect-correlated scope deltas with explicit legacy precision
- consumer-specific dependency comparator admission
- complete cause binding axes and logical dependency revision
- canonical pending cause storage and exact derived dirty caches
- pending revalidation distinct from resolved cause and resolved no-change
- shared ordinary/installed/async condition projection
- rollback, branch, checkpoint, restore, replay, observation, serial, parallel-
  feature, and WASM-capable composition
- independent financial recompute and necessity oracles
- phase-assigned mutation-sensitive scenarios and sealed financial certification

Must preserve:

- public `easy` and `facade` entry compatibility unless a documented semantic
  contradiction requires a typed denial
- graph lifecycle vocabulary and deterministic ordering
- cycle preflight before unlawful topology commit
- dependency snapshot storage and restore truth from inherited milestones
- async lifecycle as an orthogonal authority
- Milestone 11 commit-bounded observation
- hot/cold diagnostic separation and policy-controlled richness
- domain-neutral production vocabulary

Mechanical enforcement must include:

- module visibility preventing non-owner construction of deltas and causes
- `binding_axis_drift_certification!` coverage for every cause axis
- deletion inventory proving old transitive aspect fields and constructors are
  gone
- focused transition tests for output commit and cause-set lifecycle
- named financial mutation probes
- dirty Rust line-cap enforcement for all touched code and tests
- boundary and agent-context checks

## 10. Explicit Exclusions

Milestone 12 does not:

- make traversal breadth scale-local; Milestone 13 owns that claim
- add a priority queue, order-maintenance timestamps, or tree-only traversal
- expose caller-selectable scheduling strategy
- add a generic invalidation runtime to `worth-proof`
- replace Signal slots with Foundational portable aspects
- add Signal-specific vocabulary to `worth-foundational`
- put financial certification types on the production facade
- claim geometry, imaging, simulation, or another domain ready from financial
  evidence
- let diagnostics, replay presentation, or a test-only API repair missing cause
  authority
- preserve the old copied-aspect lane as a compatibility fallback

## 11. Acceptance Evidence

Milestone 12 closes only when:

- financial formulas consume authoritative fixed-point values rather than
  `AspectVersion` counters, every decisive node declares exact produced aspects,
  and the baseline owns established dependency snapshots before mutation
- `PRICE -> RISK` admits the matched filter and rejects the unmatched twin
- root recompute intent creates no resolved descendant aspect before producer
  commit
- `ChangeBatchAdmission` and deprecated commit aliases cannot construct or
  imply output-commit authority
- one producer delta is admitted differently by consumers with different
  lawful comparator policies
- producer equivalence decisions are invariant under consumer comparator
  changes, and consumer admission is invariant under producer equivalence once
  the same committed delta exists
- semantic output suppression retains the prior dependency-visible version and
  emits no cause
- producer-local slot collisions preserve distinct dependency identity
- two gated producer commits preserve their exact accumulated aspect/detail
  scopes
- pending predecessors cannot be treated as empty, mismatched, clean, or
  condition-suppressed
- a same-shaped dependency remove/re-add rejects the earlier revision's cause
- failed or rolled-back publication exposes no committed delta, cause,
  observation, or replay truth
- dirty masks and scoped aggregates rebuild exactly from canonical cause state
- incremental committed outputs and dependency snapshots equal
  `FreshFinancialRecompute` in every required scenario
- evaluated semantic work equals `FinancialNecessityManifest`; structural visit
  breadth remains explicitly unclaimed until Milestone 13
- branch, checkpoint restore, replay, diagnostics tiers, async capability,
  serial, parallel-feature, and WASM-capable lanes preserve the same causal
  conclusions
- Phase 1 proves the inherited named red control; every later phase closes only
  after the focused or financial evidence assigned to its new authority is
  green, and a cut-over scenario cannot be deferred red to a successor phase
- `FinancialAspectCausalityCertificationRun` rejects missing, duplicate, stale,
  wrong-scenario, wrong-policy, wrong-revision, or mismatched-oracle evidence
- removing immediate-dependency identity, dependency revision, commit ordering,
  exact scope correlation, pending-state distinction, or owner-only commit
  authority makes the courtroom fail
- focused tests, the complete `worth-signal` suite, boundary checks, context
  checks, formatting, and dirty Rust line-cap checks pass

## 12. Successor Handoff

Milestone 13 inherits:

- committed per-aspect/per-scope producer deltas
- consumer-specific resolved dependency causes
- canonical pending cause sets and dependency revisions
- the pending/resolved/no-change condition input
- independent financial truth and necessity oracles
- exact versus conservative scope precision

Milestone 13 may replace broad pending-candidate discovery with direct-hop
semantic admission, ready-work scheduling, and measured strategies. It may not
reinterpret aspects, move consumer comparison into producer commit, weaken any
binding axis, reconstruct causes from current graph scans, accept conservative
legacy scopes as exact locality evidence, move certification into production
authority, or alter the M12 financial semantic verdict.
