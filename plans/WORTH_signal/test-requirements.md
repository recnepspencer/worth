1. The hostile replay equivalence test
Purpose

S9.9 closeout note

## M10 profile-policy boundary

The M10 policy vocabulary is a contract boundary, not a benchmark label.  The
Signal tests must prove that:

- objective and observation activation are independently represented and
  canonically carried from Foundational profile input;
- requests progress through admitted, resolved, and installed Signal policy
  phases before planner use;
- a `Throughput + Continuous` request is admitted as an independent objective
  and activation combination, while a valid throughput/on-demand preset changes
  strategy through the installed policy;
- planner behavior does not read a caller-supplied diagnostics tier as a
  second authority;
- optional observation work names its active disposition and included/excluded
  work, with inactive observation represented by typed absence;
- these policy choices do not alter semantic output, invalidation truth,
  deterministic ordering, durability, or stable lineage.

These tests are focused boundary and adversarial tests.  They do not claim
zero-work or completed throughput evidence until a later runtime measurement
phase supplies performed counters and independent workload slopes.

Proof-safe grouped concurrent apply must now be certified at crate scope
alongside honest serial fallback for ineligible full-parallel stages. The
required owning lanes are `tests::adversarial_parallel` and
`tests::telemetry_contract`, and success means semantic equivalence plus bounded
packet/reduction counters rather than a generic "parallel mode ran" assertion.

Prove that the runtime is truly:

deterministic

transactional

replayable

provenance-stable

lineage-correct

planning-independent in semantics

serial/parallel equivalent

This is the single hardest â€œsame world, same truth, same answerâ€ test.

Scenario

Build a medium-large graph with all of the following present in one topology:

aspect-scoped dependencies

dynamic dependency discovery

conditional gates

comparator suppression

partition-aware outputs

structural memoization

keyed query families

multiple branches of dependent subgraphs

at least 3 execution stages in the planner

same-stage parallel precompute opportunities

Then run this exact script:

Start from a canonical baseline snapshot S0

Apply a deterministic patch sequence P1..P20

After selected patches, do:

explicit evaluation of target set A

lazy pull of target set B

snapshot capture

rollback-triggering fault injection during one transaction

retry after rollback

At patch 10, fork branch B1

On B1, apply P11b..P15b

On main branch, apply P11a..P15a

Restore an earlier snapshot on each branch

Resume forward execution from both restores

Record lineage across:

replacement

refresh

memoized reuse

invalidation without replacement

snapshot restore

branch switch / fork / merge markers

Execute the full scenario in four modes:

serial planner + serial execution

staged planner + parallel precompute

fresh process replay from captured snapshots/logs

event-by-event deterministic replay from recorded execution history

What this probes

It simultaneously probes:

deterministic evaluation order

deterministic artifact evolution

rollback hard-rewind correctness

snapshot capture fidelity

restore correctness

branch semantics

planner/executor parity

memoization correctness under replay

explain/provenance stability

lineage event stability

separation of truth from compute, because replay must work from host snapshots rather than hidden signal-owned truth

Required verification output

The output should be a verification package with these exact categories.

A. Final state equivalence report

For every tested mode, emit:

final node states

final artifact identities

final output values

final output change classifications

final cleanliness state (Clean / MaybeStale / Dirty)

final dependency graph snapshot

final per-aspect versions

Pass condition

All four modes must produce bit-for-bit identical canonicalized final-state artifacts, except for fields explicitly declared non-semantic, like wall-clock timestamps.

B. Snapshot equivalence matrix

For every captured snapshot:

snapshot ID

canonical node-state digest

canonical artifact-state digest

canonical dependency-state digest

lineage digest through that point

Pass condition

Restoring snapshot Sn and re-running the exact suffix workload must produce the same canonical digest as the original historical execution suffix.

That proves snapshot correctness plus replay correctness.

C. Rollback integrity report

For each injected failure:

transaction ID

fault point

pre-transaction digest

failure-time partial internal digest

post-rollback digest

retry digest after successful retry

Pass condition

post-rollback digest must equal pre-transaction digest exactly

retry digest must equal the digest from the equivalent no-failure control run

That proves hard-rewind, not soft best-effort cleanup.

D. Provenance equivalence report

For a selected set of nodes, emit canonicalized explain(node) outputs including:

invalidation provenance

dependency provenance

condition provenance

comparator provenance

recomputation provenance

host causality metadata if present

Pass condition

Across all execution modes and replay modes, the explanation must be semantically identical after canonicalization.

Not merely same final value â€” same causal explanation.

E. Lineage equivalence report

Emit the full lineage stream with canonical ordering, including:

ArtifactTransition

Invalidation

SnapshotRestore

BranchFork

BranchSwitch

BranchMerge

Pass condition

For equivalent historical executions, lineage records must be semantically identical and sequence-consistent. If replay produces different artifact ancestry or different restore/memoized-reuse history, the runtime has failed the Phase 5 promise.

Why this is ultimate

Because it proves the deepest claim in the vision:

the runtime is not just usually correct; it is historically reproducible, causally inspectable, and planner-independent in semantics.

If this fails, the â€œdeterministic transactional auditable derived runtimeâ€ claim is not yet real.

S9.16.5 alignment note

Any certification bundle that exercises replay, lineage, explanation, provenance,
or cold artifact access must also prove the diagnostics-tier contract:

- DiagnosticsTier changes retained richness, not canonical runtime/replay/lineage truth
- RetentionBudget bounds retained history/detail/replay envelopes
- ReconstructionBudget gates explicit cold materialization only
- DiagnosticsAvailability distinguishes retained, reconstructed, omitted, denied, and unavailable outcomes explicitly
- ordinary summary/history/replay reads must perform zero cold reconstruction
- retained/reconstructed/denied cold-work counters must be attributable by access lane and API family
- retained-envelope shaping must follow the active runtime policy budget rather than tier defaults alone
- long-session branch/snapshot churn must remain bounded by retained history/detail/replay envelopes

1A. The adversarial observation and delivery equivalence test
Purpose

Prove that runtime-local observation is truly:

commit-bounded

rollback-safe

deterministic

classification-stable

replay-honest

branch/restore coherent

bounded by relevant change rather than graph size

This is the substrate test that future watchers, effects, forms, resources, and
UI adapters must inherit instead of redefining.

Scenario

Build a medium-large graph with all of the following present in one topology:

aspect-scoped dependencies

conditional gates

comparator suppression

partition-aware outputs

structural memoization

keyed query families

multiple overlapping observer registrations

observers interested in:

touched change

recomputed change

meaningful change

Then run this exact script:

Start from a canonical baseline snapshot S0

Register a deterministic observer set O1..On over overlapping node sets

Apply a deterministic patch sequence P1..P20

Include in that sequence:

multiple writes to the same source before one commit

recompute-without-meaningful-change cases

rollback-triggering fault injection after observation staging but before final
commit success

unsubscribe during heavy churn

branch fork

branch-local edits

snapshot restore

branch restore

merge-driven rewrites if the runtime surface admits them in the current phase

Execute the full scenario in these modes:

serial baseline

parallel-capable runtime mode where admitted

fresh-process replay from captured snapshots/logs

event-by-event deterministic replay from recorded execution history

What this probes

It simultaneously probes:

observer delivery boundary correctness

rollback suppression

per-observer transaction coalescing

touched vs recomputed vs meaningful-change classification stability

observer ordering determinism

unsubscribe lifecycle correctness

branch/restore delivery honesty

replay parity of observer-visible semantic change

bounded observer matching and delivery breadth

Required verification output

A. Observer delivery equivalence report

For every tested mode, emit:

observer ID

transaction ordinal

delivery count

delivery classification set

affected observed scope digest

branch ID

Pass condition

Equivalent executions must produce semantically identical per-observer delivery
streams after canonicalization. The runtime may not change observer-visible
truth across execution mode, replay mode, or restart mode.

B. Rollback suppression report

For each injected failure:

transaction ID

observer packets staged before failure

observer packets actually delivered

post-rollback digest

retry delivery digest

Pass condition

No normal observer delivery may escape from the failed transaction.
The retry lane must match the no-failure control lane exactly.

C. Coalescing and ordering report

For each observer and transaction:

matched node count

coalesced packet count

delivery ordinal

Pass condition

Each observer receives at most one normal delivery packet per committed
transaction boundary, and ordering remains deterministic across equivalent
executions.

D. Boundedness report

Emit:

staged observation candidate count

matching observer-set width

delivered observation count

coalesced observation count

rollback-suppressed delivery count

observation classification breadth

Pass condition

Delivery and matching breadth must scale with changed derived surface plus
matching observers, not with total graph size or total active observer count.

Why this is ultimate

Because it proves the runtime owns committed derived-state observation as a real
semantic substrate instead of leaving that job to adapter heuristics.

If this fails, future watchers, effects, forms, and resources will be forced to
invent their own truth model on top of the runtime.

2. The adversarial granularity suppression test
Purpose

Prove that aspect-aware invalidation, comparator suppression, partition-scoped propagation, structural memoization, and conditional gates all work together without semantic leaks.

This is the test that proves WORTH Signal is not just a reactive engine, but a precision recomputation engine.

Scenario

Construct a graph with deliberate granularity traps:

upstream nodes exposing many independent aspects

downstream nodes subscribing to:

single aspects

aspect groups

partition subsets

dynamic subsets discovered at evaluation time

comparator policies including:

exact equality

epsilon/tolerance

structural identity

partition-local diff suppression

conditions including:

debounce

on-demand

delta threshold

custom host gate

query families keyed by host-supplied structural keys

memoization caches shared across families where legal

Then generate a long randomized-but-seeded workload of host changes with these properties:

some changes alter irrelevant aspects

some alter relevant aspects but produce comparator-suppressed outputs

some alter one partition while leaving others identical

some flip conditions from blocked to allowed

some alter dynamic dependency shape

some cause same-value recomputes that should not propagate

some force true downstream recompute

some invalidate huge transitive regions where only a tiny frontier should actually run

Run thousands of mutations.

Also run a reference oracle beside it:

either a brute-force full recompute engine

or a â€œrecompute everything every timeâ€ correctness harness

What this probes

It simultaneously probes:

aspect invalidation correctness

absence of over-invalidation

absence of under-invalidation

comparator correctness

result diff correctness

partition-aware suppression

dynamic dependency capture correctness

condition gating correctness

memoization validity boundaries

query family key correctness

runtime trust metrics honesty

Required verification output
A. Precision matrix

For every mutation step, emit:

changed host aspects

nodes marked dirty / maybe-stale

nodes actually evaluated

nodes skipped due to condition

nodes suppressed due to comparator / unchanged output

nodes reused via memoization

partitions reported changed

downstream nodes propagated to

Pass condition

This matrix must exactly match the oracleâ€™s semantic necessity set:

every node that must recompute eventually does

no node that need not recompute is actually executed unless explicitly allowed by policy

no partition outside the true changed set is reported changed

This is the most important precision proof.

B. Waste ratio report

For the full workload, emit:

total dirty marks

total evaluations

total suppressed propagations

total memoized reuses

total partition-local suppressions

total condition deferrals

total no-op invalidations

full-recompute reference evaluation count

Pass condition

Not just â€œsmaller than brute force.â€
The runtime must show a stable and explainable suppression profile under adversarial conditions, and every suppression class must be attributable by provenance.

In other words: optimization must stay explainable, not become mysterious magic.

C. Correctness parity report

At each step, compare runtime outputs to full oracle outputs:

final values

per-partition values

per-query-family values

exposed downstream artifacts

Pass condition

Exact parity on all semantically visible outputs for all steps.

If suppression ever hides a necessary recompute, this catches it.

D. Granularity attribution report

For selected nodes, emit structured explanations showing:

why they were dirtied

which aspect triggered dirtiness

which dependency path mattered

whether comparator suppressed downstream effect

whether partition diff narrowed propagation

whether memoized reuse occurred

whether conditions deferred execution

Pass condition

Explanation must align with the precision matrix and with actual runtime behavior. The runtime cannot merely do the right thing; it must explain the right reason.

Why this is ultimate

Because most incremental runtimes can be correct by being noisy.
The hard problem is being correct and surgically precise at the same time.

This test proves the claim:

WORTH Signal answers not only what must recompute, but what must not.

That is one of the core promises of the whole system.

### Milestones 12-13 financial invalidation binding addendum

The adversarial granularity suppression test is not satisfied by direct-edge
same-aspect examples or by proving only that transitive nodes are graph-
reachable.

Milestones 12 and 13 must expand the existing causally valid fintech financial
world and certify their guarantees during implementation. A generic graph
fixture may support a focused local proof, but cannot close either milestone.
There is no later invalidation-certification milestone.

Every adversarial test is a named scenario with:

- a production-valid financial baseline and semantic handles issued by the
  existing world compiler
- one named market, portfolio, partition, condition, topology, or branch delta
- a falsifiable production claim and the plausible implementation defect it
  must expose
- independently declared expected financial truth and semantic necessity set
- consequential observations through the real runtime composition root
- a positive/negative twin or mutation probe proving intended sensitivity
- scenario identity, seed, financial scale tuple, policy tuple, diagnostic
  tier, and exact failing mutation step for reproduction

#### Financial world authenticity floor

The current financial-shaped lifecycle fixture is inherited setup, not by
itself Milestone 12 certification. The Milestone 12 world compiler must issue a
sealed `CausallyCompleteFinancialBaseline` from an immutable
`FinancialWorldDefinition` with:

- deterministic fixed-point market values for quotes, FX pairs, curve buckets,
  and volatility buckets
- typed positions with instrument identity, quantity, currency, maturity or
  bucket ownership, pricing/risk model assignment, factor subscriptions, and
  book/desk ownership
- at least one rates-sensitive, one FX-sensitive, and one volatility-sensitive
  position whose required dependency changes follow from those economic facts
- exact declared producer output aspects and consumer dependencies for every
  node used by a decisive scenario
- typed semantic handles issued by construction rather than scenario-authored
  raw `NodeId`, aspect-slot, partition-token, or revision literals
- initialized source truth, committed output truth, dependency snapshots, and
  branch/checkpoint posture required by the named baseline

Financial values and Signal revisions are different truth domains. Pricing,
risk, aggregation, and oracle formulas must consume financial values, never
perform economic arithmetic on `AspectVersion` counters. A deterministic
projection may convert an economic result into a dependency-visible semantic
version only after the economic result exists. The projection must retain the
prior producer-local revision for an unchanged canonical result and advance a
monotonic revision for a changed result; it cannot decide consumer routing,
condition eligibility, or the necessity manifest.

The reference financial formulas require focused closed-form or table-driven
tests independent of Signal invalidation. This is an authenticity floor, not a
request for a production quant library: day-count, calibration, exchange, or
regulatory detail is required only when a named scenario depends on it.

`FreshFinancialRecompute` must reconstruct expected outputs from authoritative
market and portfolio inputs without consuming the incremental frontier, dirty
masks, incremental condition classification, suppression routing, or ready
queues. `FinancialNecessityManifest` must enumerate economically affected work
from scenario-owned positions, factor subscriptions, aspect translations, and
partition ownership without calling production routing or scheduling logic.

Milestone 12 must own these scenario families:

- `quote_to_risk_aspect_translation`: `PRICE -> RISK` positive/negative filter
  twins expose root-aspect copying
- `heterogeneous_consumer_comparators`: exact, tolerance, and installed
  comparator consumers observe one producer delta and admit it independently
- `tolerance_suppressed_repricing`: small and large quote moves distinguish
  lawful suppression from real downstream change
- `producer_local_factor_slot_collision`: equal numeric slots on distinct FX,
  curve, or market dependencies retain producer identity
- `partitioned_curve_bucket_bump`: a rates detail change remains separate from
  unrelated regions; at least two detail changes commit while the valuation
  and its threshold-gated risk consumer remain unevaluated, so lost or
  cross-product scopes are visible before the settled financial gate admits
- `gated_repricing_release`: authentic small/large financial delta-threshold
  twins prove blocked then admitted work cannot strand a necessary risk
  consumer; named domain-neutral production scenarios prove pending precedence
  for on-demand, temporal, custom, installed, and async orchestration axes
- `instrument_dependency_rewire`: committed dependency changes and cycle
  rejection cannot reuse stale causes, including removal and recreation of an
  identical edge under a new dependency revision
- `branch_shock_restore_replay`: branch, checkpoint, replay, diagnostic tier,
  and async-capability composition preserve financial truth

Phase 1 must demonstrate the inherited defect with the named red control before
production invalidation repair begins. Each later Milestone 12 phase must pass the focused or
financial evidence assigned to the authority that phase establishes; once a
scenario's production path cuts over, it may not be left red for a successor
phase. Final construction of
`FinancialAspectCausalityCertificationRun` must reject missing, duplicate,
stale, wrong-scenario, wrong-policy, wrong-revision, or mismatched-oracle
evidence.

Focused Milestone 12 transition evidence must additionally prove:

- every dependency-cause binding axis has a matching positive case and a
  one-axis-drift denial twin
- producer output-equivalence decisions are invariant under exact, tolerance,
  custom, and installed consumer dependency comparators; consumer admission is
  invariant under the producer's output-equivalence policy once the same
  committed delta exists
- installed conditional dependency and output comparators retain distinct
  role-bound identities and reach their respective admission/commit boundaries
- a legacy output-identity configuration restores to the same canonical split
  producer/consumer policies and behavior as its explicit replacement
- a root recompute seed cannot construct a resolved descendant aspect cause
- a prepared or compared apply packet cannot construct performed output-commit
  evidence
- contradictory `OutputChange::Unchanged` plus changed semantic aspect versions
  is rejected before publication
- failure at each pre-publication effect seam exposes no committed delta, cause,
  observation, or replay truth
- clearing and rebuilding dirty aspect/scope caches from canonical cause state
  is exact
- conservative legacy scope unions remain correctness-compatible but cannot
  satisfy exact locality evidence
- eligible grouped-parallel cases prove that the parallel preparation/apply
  path actually executed, while honest serial fallback is separately reported;
  equality between two silently serial runs is not parallel-path evidence

#### Milestone 12 phase evidence map

| Phase | Evidence required before the phase closes |
|---|---|
| Phase 1 - Authentic Financial Courtroom And Red Control | authentic financial definition and sealed baseline; independent reference-formula tests; runnable fresh-recompute and necessity-manifest cores; failing `quote_to_risk_aspect_translation` matched/unmatched red control against the inherited runtime |
| Phase 2 - Output Contract And Owner-Specific Proof Forms | producer/consumer policy separation; installed-role preservation; legacy upgrade equivalence; contradictory-output denial; every binding-axis drift twin; prepared packets denied performed authority |
| Phase 3 - Canonical Cause Storage And Recovery Basis | exact cause-set/cache rebuild; repeated-scope accumulation; rollback removal; checkpoint serialization and fresh-runtime readmission; same-shaped rewire rejects the prior revision |
| Phase 4 - Atomic Output Commit And Direct Cause Admission | `quote_to_risk_aspect_translation`, `heterogeneous_consumer_comparators`, and `tolerance_suppressed_repricing`; prepublication failure seams expose no partial state; serial and exercised parallel-feature commits agree |
| Phase 5 - Planner, Condition, Scope, And Rewire Cutover | `partitioned_curve_bucket_bump`, `gated_repricing_release`, and `instrument_dependency_rewire`; conditions consume settled immediate causes; copied-aspect, cross-product-scope, last-scope-only, and stale-revision mutations fail |
| Phase 6 - Branch Composition, Financial Certification, And Documentation | `branch_shock_restore_replay`; unresolved causes cross branch/checkpoint/replay/diagnostic/async lanes without being drained before capture; producer-local slot collision; aspect-free/scope-free structural transitive summaries; every earlier scenario and mutation probe remains green/red as appropriate; sealed `FinancialAspectCausalityCertificationRun` rejects missing, duplicate, mixed-seed, stale, wrong-scenario, wrong-policy, wrong-revision, wrong-diagnostics-tier, mismatched-oracle/work, or incomplete-lifecycle evidence |

Milestone 13 must own these scale-sensitive scenario families:

- `sparse_book_fanout`: one relevant depth-16 pricing/risk chain under
  increasing disjoint instrument/audit fanout
- `partitioned_curve_universe`: one rates bucket among 16, 256, and 1,024
  independently owned regions
- `convergent_factor_batch`: price, FX, curve, and volatility shocks converge
  with duplicate causes and insertion-order permutations
- `dense_market_close`: sparse, medium, and dense legitimate frontiers at
  `10^3`, `10^4`, and `10^5` scheduled-lane node scales
- `portfolio_dependency_churn`: instrument/desk/factor/model rewiring and cycle
  rejection under repeated mutations
- `branch_restore_locality_replay`: the narrow and dense traces repeat after
  capture, restore, replay, and deterministic rerun

Every Milestone 13 runner is compiled from a production-valid
`FinancialWorldDefinition`. Scale is expressed as books, desks, positions,
factors, owned regions, models, and subscriptions. Appending raw graph nodes,
edges, aspects, scopes, revisions, dirty state, queue entries, or counters in a
scenario is forbidden. Disconnected node padding cannot satisfy a fanout or
scale axis. Baseline sealing must prove that compiler-generated dependencies
and snapshots still match the economic declarations before the mutation runs.

Before invoking production invalidation, the independent locality oracle must
derive and freeze:

- `Q`: exact reverse-index bucket keys implied by the performed deltas
- `C`: authoritative direct dependency declarations in those buckets
- `K`: canonical distinct M12 dependency causes expected after admission
- `U`: canonical target/revision/epoch/stage work identities after merge
- `E`: exact economically necessary evaluations
- `S`: exact unchanged-output propagation stops
- `P`: exact peak ready width under the declared hostile release schedule

The oracle may consume only the immutable financial definition, named
mutation, and declared execution posture. It may not consume the production
reverse index, router, scheduler, dirty state, queue, trace, counters, or
receipt. Small decisive cases compare the exact identities in `Q/C/K/U/E/S`,
not only their counts. Scheduled cases may compare canonical digests and
cardinalities only when an ordinary-size twin proves the identical generator
and expectation rules with full identities.

For each complete trace, report and assert realized:

- reverse-index bucket probes and candidates returned
- dependency edges examined
- candidates rejected by aspect, scope, or comparator before mutation/enqueue
- canonical causes admitted and work identities merged
- work items enqueued and popped
- nodes visited and evaluated
- produced deltas and unchanged-output stops
- non-semantic traversal visits
- maximum ready-frontier width
- stale/rebind denials, topology revalidation work, batch allocations, and peak
  batch memory where measurable

The receipt and declared phase checkpoints must satisfy exact manifest
equations: bucket probes equal `|Q|`; candidates returned and edges examined
equal `|C|`; rejection rows partition `C - K`; performed admission evidence
and the pre-settlement storage checkpoint contain `K`; the
post-lowering/pre-execution checkpoint and later performed execution identity
contain `U`; evaluation identity equals `E`; stop identity equals `S`; and peak
ready width equals `P`. Other performed rows must equal the scenario-owned
expectation, not a tolerance selected from the observed run. `K` may be
lawfully released after settlement and `U` consumed by execution, so observing
either at an undeclared earlier/later point is not evidence. Planned/predicted
counters cannot satisfy any equation.

Three slopes are separate verdicts:

- adding `x` valid dependencies outside `Q` while holding `E` fixed produces
  exact zero delta in candidate, visit, dirty, queue, and evaluation rows
- adding `x` valid rejecting dependencies inside a queried bucket produces
  exactly `+x` candidates, examinations, and the applicable rejection row, with
  zero admitted-cause, ready-work, and evaluation delta
- adding economically affected declarations expands causes, work, and
  evaluations by the exact independent `K/U/E` delta; dense work may not be
  dropped to preserve a cheap slope

Wall-clock measurements are supplemental and an aggregate "fanout" slope
cannot replace these three structural verdicts.

The scenario-specific red controls are normative:

- `sparse_book_fanout` must fail for closure walking, producer-wide scans,
  late filtering, omitted rejected-candidate rows, or disconnected padding
- `partitioned_curve_universe` must fail for all-region scans, aspect/scope
  cross-products, unrelated detail enumeration, last-detail loss, late scope
  rejection, or trusting a drifted derived index
- `convergent_factor_batch` must fail for node/slot-only deduplication,
  last-cause-wins, duplicate evaluation, reconstructed provenance, or
  seed-order-dependent canonicalization
- `dense_market_close` must fail for sparse-only shortcuts, fixed work caps,
  dropped necessary work, reachability relabeled as evaluation, omitted
  width/allocation rows, or strategy comparison over different work streams
- `portfolio_dependency_churn` must fail for stale work execution, edge-shape
  identity without revision, revision change on rejected cycles, topology-order
  reuse, uncounted revalidation, or work emitted by a rejected mutation
- `branch_restore_locality_replay` must fail for restored ready authority,
  stale binding reuse, same-count cause substitution, recovery/replay charged
  as hot work, or parity that compares only final values/event kinds/counts

The full construction, independent observations, and exact slope laws for each
family are normative in `milestone-13-plan.md` section 3.1. A scenario must fail
when its baseline provenance is false; setup failure cannot be counted as a
successful red mutation. Section 3.1.4's ordinary and scheduled scale tuples
are mandatory case-identity data: runners may add smaller debugging cases but
may not resize, round, substitute, or silently skip a required tuple. A typed
incomplete/resource-denied scheduled outcome remains in the report and does
not count as a passing scale.

Every Milestone 13 implementation phase must name and pass its assigned
financial scenarios before it closes. Correctness and locality are separate
verdicts, strategy comparisons consume exactly the same admitted work stream,
and `FinancialFrontierLocalityCertificationRun` rejects missing, duplicate,
stale, or mismatched evidence. The ordinary gate includes a small instance of
every scenario plus sparse/dense slopes; scheduled lanes own broad seeds and
the largest scales, but no correctness claim may exist only there.

Milestone 13 phase/progression evidence must additionally prove:

- Signal owner wrappers retain admitted source, prepared/committed dependency,
  and admitted structural origins, converge them into resolved work, then
  progress through lowered, ready, and executed forms backed by `worth-proof`;
  the executor accepts only ready work and evidence packaging accepts only
  performed execution
- root source seeds/raw direct bases, topology intents/raw structural bases, raw
  producer deltas, prepared output packets, raw cause sets, stale/boundary-
  bridged work, Foundational receipts, and canonical digests cannot substitute
  for a required stronger Signal phase
- an internal lawful progression fixture type-checks without exposing a
  test-only facade; construction/export inventories prove stronger forms and
  witnesses are not publicly constructible; QA compiler-failure mutations
  cover internal phase skips
- committed UI compile-fail cases are limited to genuine caller-visible
  impossibilities with positive twins; private import failure is not accepted
  as proof of internal phase topology
- executable-work binding axes cover graph instance, target node, dependency
  revision, origin generation or dependency commit ordinals, readiness epoch, and
  stage/order, with a positive case and one-axis drift denial for each
- output/cause publication retains Milestone 12 atomicity; scheduling never
  republishes or reconstructs causes
- ready queues are absent from checkpoint authority and post-restore work is
  rebuilt from readmitted source bases, canonical causes, node state, and
  current topology
- direct adjacency examinations are reported separately from admitted semantic
  work; exact changes use producer-local aspect and partition/detail reverse
  buckets rather than scanning all producer subscribers, and rejected
  transitive fanout beyond returned candidates creates no descendant visit
- reverse-index destruction/rebuild proves dependency topology is authoritative;
  index drift is detected or repaired before ordinary admission, and index hits
  still pass authoritative edge/snapshot/revision/cause validation
- `FinancialLocalityExpectationManifest` derives expected counter contracts
  from the financial definition/topology without importing production routing,
  queue, counter, or scheduling code
- Signal performed receipts supply observed rows; Foundational performance
  bundles supply portable work disclosure and counter-backed attachment only;
  neither expected rows nor Foundational evidence can mint operational work
- public facade tests prove `InvalidationPlanningEstimate` cannot substitute
  for `SignalInvalidationExecutionReceipt`; removed reachability/transitive-wave
  constructors have no operational compatibility alias
- ordinary hot-path receipts explicitly exclude replay/reconstruction,
  support-report assembly, and forensic parity work
- each locality scenario owns an opaque completion created only by its real
  runner after semantic, necessity, receipt, slope, lifecycle, and mutation
  checks; a generic scenario-plus-boolean completion is forbidden
- the sealed run derives case/report identities and rejects paired tampering of
  expected/actual fields, all-same wrong seeds, work-stream relabeling, and
  missing/duplicate scenario completions

#### Milestone 13 phase evidence map

| Phase | Evidence required before the phase closes |
|---|---|
| Phase 1 - Boundary Inventory, Red Slopes, And Proof Contract Freeze | authentic sparse/partition red slopes; current-boundary inventory; frozen typed scale tuples and `Q/C/K/U/E/S/P` contracts for all six scenarios; separate index-disjoint, queried-candidate, and semantic-frontier slope verdicts; independent locality expectation core; Proof compile-pass progression prototype and compiler-failure prepared-to-executor mutation without widened visibility |
| Phase 2 - Owner-Specific Proof Progression And Atomic Promotion | construction/export inventory plus phase-skip compiler mutations; private authorities/capabilities; performed output promotion only; every work-binding drift twin; prepublication failure exposes no committed work |
| Phase 3 - Direct-Hop Routing Cutover | transitive pre-mark/walk removed; sparse fanout and partition universe green; pre-mutation/pre-enqueue rejection; restored closure-walk and late-filter mutations red |
| Phase 4 - Topology Lowering, Readiness, And Causal Deduplication | convergent factor permutations and sparse/medium/dense small worlds; exact cause-preserving merge; performed enqueue/pop/dedup rows; pending-first condition and async families green |
| Phase 5 - Rewire, Restore, And Trust-Boundary Reconstitution | dependency churn and branch/restore/replay; stale work denial; no serialized ready authority; rebuilt current binding; cycle rejection creates no work |
| Phase 6 - Realized Counters And Foundational Evidence Cutover | predicted/realized type and public-facade separation; reachability-plan/wave constructor removal; performed Signal rows; Foundational canonical identity and counter-backed receipt; wrong/missing/duplicate/unexpected row denials; diagnostic-tier equality with cold-work separation |
| Phase 7 - Scale Courtroom, Strategy Decision, And Closeout | all six scenario families at ordinary/scheduled lanes; sparse/dense slopes; identical admitted work for strategy comparisons; complete sealed locality run; historical mutations red; M14 handoff frozen |

### Milestone 13.1 cross-runtime granular invalidation binding addendum

Milestone 13.1 must prove that the locality and performed-work contract sealed
by Milestone 13 survives the real Relational -> Runtime Bridge -> Signal ->
Runtime Bridge -> Query -> Query Host path. A direct `SignalGraph` test, a
Query-local `BridgeChangeSummary`, a count-only invalidation sink, or a test-only
region executor cannot close this milestone.

The real cross-runtime courtroom must own these scenario families:

- `curve_detail_to_live_risk`: one exact curve partition/detail reaches the
  matching detail, whole-partition, and unscoped consumers while sibling detail,
  unrelated aspect, audit, and unrelated Query consumers perform zero work
- `suppressed_quote_no_query_patch`: below- and above-tolerance quote twins
  separate admitted direct truth from performed Signal-derived consequence and
  prove comparator suppression emits no false derived Query patch
- `ordered_portfolio_membership`: projection-only, predicate-membership,
  ordering, and window-boundary changes select distinct Query maintenance
  consequences without whole-query re-execution
- `shared_lease_disclosure_noninterference`: equivalent maintenance is shared,
  consumer delivery remains separately authorized, and revocation after impact
  selection prevents protected publication without leaking scope or timing
- `correspondence_rebind_restore`: destroyed derived indexes, fresh runtime
  identities, delayed old delivery, duplicate/reordered delivery, and current
  delivery prove owner-specific readmission and convergence
- `opaque_region_platform_twin`: non-financial opaque partition/detail tokens
  and a WASM-supported lane prove that domain meaning and host-only assumptions
  did not enter Signal, Proof, or Foundational authority

Every scenario is compiled through production-valid Relational, Bridge, Signal,
Query installation, managed-live, and Query-host composition. The world issues
semantic handles and authority-bearing products. Scenarios may not author raw
Signal nodes/aspects/dirty state, Bridge receipt bindings, Query admitted
impacts, ready queues, performed counters, or publication authority.

Before the committed mutation enters production routing, an independent
`CrossRuntimeInvalidationNecessityManifest` must freeze:

- `R`: exact authoritative Relational changes and locality
- `B`: exact installed Bridge correspondence and widening outcomes
- `S`: exact installed Signal scoped seeds and performed work
- `I`: exact Query dependency impacts by semantic role
- `M`: exact distinct Query maintenance operations after lawful sharing
- `D`: exact per-consumer query-shaped deliveries after current disclosure
- `X`: exact suppressions, denials, stale/rebind outcomes, and terminations

The manifest consumes only immutable world declarations, installed semantic
contracts, and the named mutation. It may not consume Bridge matching, Signal
routing, Query impact indexes, queues, receipts, diagnostics, or observed
counters. Ordinary cases compare full identities. Scheduled cases may compare
canonical digests and cardinalities only when an ordinary-size twin proves the
same generator and expectation rules with full identities.

Direct truth and Signal-derived consequence are separate proof obligations:

- a Bridge-admitted authoritative truth change may drive a Query dependency
  that directly consumes that truth without fabricating Signal execution
- a Query dependency that declares a Signal-derived or conditional consequence
  requires a performed `SignalInvalidationExecutionReceipt`
- a Signal receipt proves performed Signal work only; it cannot authorize Query
  impact, maintenance, disclosure, or publication
- zero-work and count-only Signal receipts are forbidden
- comparator suppression must remain distinguishable from no Signal
  participation and from a performed meaningful output change

Focused transition evidence must prove:

- installed Signal scoped change admission preserves exact aspect-correlated
  partition/detail pairs; a bare detail, aspect/scope swap, foreign capability,
  duplicate capability, or stale graph is denied
- unknown or newly introduced partitions still reach an explicitly unscoped
  consumer while scoped siblings remain disjoint
- Runtime Bridge exact whole-aspect, record, whole-partition, and
  partition/detail lowering, declared widening, suppression, stale, defer,
  denial, failure, and rebind outcomes are typed
- source runtime, source installation generation, source authority binding,
  commit, snapshot/branch, aspect contract revision, correspondence revision,
  Signal graph instance, and target generation each have one-axis drift denials
- Query candidate selection and current admission are separate; index hits do
  not bypass purpose, tenant, disclosure, branch, basis, installation, lease,
  or lifecycle revalidation
- projection, membership, ordering, grouping, window, relation, conditional or
  workflow, authorization/disclosure, and structural dependency roles choose
  their declared maintenance consequence
- publication consumes Query-performed maintenance or a typed lawful
  suppression/terminal outcome, never Bridge or Signal evidence directly
- Query public callers can use the granular path without importing lower-runtime
  internals; predicted Signal estimates, descriptive bridge summaries, raw
  Signal receipts, Foundational reports, and stale/foreign owner products cannot
  substitute for the required next phase
- destroyed Bridge, Signal, and Query indexes rebuild from authority; ready and
  candidate queues do not survive as checkpoint authority
- duplicate, delayed, and reordered delivery converges without duplicate
  maintenance or publication
- ordinary receipts exclude replay, reconstruction, diagnostic, support, and
  forensic work

The structural scale verdicts are independent:

- increasing Bridge correspondences outside queried semantic keys changes no
  downstream candidate, seed, impact, maintenance, or publication row
- increasing returned-but-rejected Bridge candidates changes only Bridge
  candidate, examination, and rejection rows
- increasing unrelated Signal subscribers changes no Signal or Query work row
- increasing unrelated installed queries changes no admitted impact,
  maintenance, or publication row
- increasing equivalent shared consumers changes authorized delivery count but
  not maintenance count
- increasing result rows outside the affected region changes no maintenance row
- increasing the actual semantic frontier expands every owner row by the exact
  independent `B/S/I/M/D` delta; dense necessary work is never dropped

Each owner reports its own performed counters. A Bridge target count cannot
stand in for Signal work, and an end-to-end total cannot hide a broad scan in
one runtime. Wall-clock evidence is supplemental to exact structural counters.

#### Milestone 13.1 phase evidence map

| Phase | Evidence required before the phase closes |
|---|---|
| Phase 1 - Boundary Freeze And End-To-End Red Courtroom | real composed financial and opaque-region worlds; current coarse/count-only and test-only boundary inventory; independent `R/B/S/I/M/D/X` core; inherited 5y/10y locality-loss and direct-vs-derived red controls |
| Phase 2 - Signal Installed Scoped Invalidation Contract | installed capability plus exact region admission; partition+detail correlation; unknown-partition unscoped twin; performed receipt from actual execution; predicted/performed and foreign/stale denials; default/parallel/WASM parity |
| Phase 3 - Runtime Bridge Exact Lowering And Performed Delivery | every exact/widened/suppressed locality outcome; direct truth versus performed Signal product; every Bridge binding drift twin; failure seams expose no false performed delivery |
| Phase 4 - Query Installed Invalidation Manifest And Admission | complete dependency-role manifest; exact indexed selection; current Query admission; index destruction/rebuild; authority drift denials; public phase-skip compile evidence |
| Phase 5 - Production Maintenance, Sharing, And Facade Cutover | curve, suppression, ordered membership, and shared-disclosure scenarios through real host observation; production region execution; query-shaped outcomes; coarse/count-only path and descriptive authority lane removed |
| Phase 6 - Rebind, Restore, Branch, And Lifecycle Closure | destroyed derived state, fresh-runtime rebind, stale/delayed/duplicate/reordered delivery, revocation before publication, no serialized ready/candidate authority, cert-only replay separation |
| Phase 7 - Structural Slopes, Certification, Documentation, And Handoff | all six scenarios, seven independent slopes, sealed cross-runtime run, owner-separated performed rows, default/parallel/WASM/constitutional gates, executable documentation, frozen M14 handoff |

The sealed cross-runtime run rejects missing, duplicate, stale, wrong-scenario,
wrong-seed, wrong-policy, wrong-diagnostics-tier, mixed-runtime,
wrong-direct-versus-derived, and mismatched-manifest evidence. Coordinated
caller-supplied expected/actual values cannot construct completion evidence.

### Milestones 14-17 binding addendum

Parallel execution evidence must use an independent serial oracle and hostile
schedule control. Running the same happy-path graph once with `parallel`
enabled does not prove any milestone in this sequence.

Milestone 14 must prove:

- configured worker and memory budgets are strict hierarchical leases rather
  than dispatch hints
- nested execution cannot create independent capacity or oversubscribe the
  parent lease
- worker-local compute has no authoritative graph mutation capability
- schedule and completion-order perturbation cannot change canonical
  publication, replay, history, or explanation under bitwise determinism
- cancellation, deadline, and failure at each precompute/apply/publication
  boundary expose exact typed progress and preserve precommit atomicity
- a serial-only target consumes the same lowered plan without semantic drift
- M13 partition/detail behavior is preserved exactly as the two-segment case of
  a bounded opaque scope hierarchy
- exact-leaf, ancestor-subtree, unscoped, and sibling-disjoint mutations produce
  the exact independent candidate/work sets at path depths `1`, `2`, `4`, and
  `8`
- `ProducerAspectKey`, scope paths, subscriber buckets, and physical shard keys
  cannot substitute for current edge, snapshot, cause, readiness, or commit
  authority
- worker-count and epoch-bounded shard-placement changes preserve admitted
  semantic work and canonical outputs while exposing occupancy, imbalance,
  migration, and resident-byte evidence

Milestone 15 must combine wide antichains, long critical chains, reconvergence,
dynamic rewiring, overlapping mutation footprints, and bounded ready queues.
The evidence must fail if readiness, control-order proof, any synchronous
mutation-footprint axis, or atomic epoch publication is removed. Serial and
graph-parallel mutation histories must agree step by step, not only at one
final output.

The same graph courtroom must include deep hierarchical subscriptions,
cross-shard diamonds, hot-subtree skew, shard-boundary rewiring, and
epoch-bounded rebalancing. A sibling-disjoint subtree must contribute zero
candidates and work. Every cross-shard dependency must remain readiness-ordered,
and placement changes must leave causes, dependency revisions, work identity,
history, replay, and explanation unchanged.

Milestone 16 must certify map, reduce, scan, fork/join, and synchronous rounds
using stable partition identity and explicit read/write sets. The test matrix
must vary worker count, partition count, skew, grain size, numerical reducer,
nesting depth, cancellation point, and serial-only capability. Floating-point
bitwise claims require fixed partition and join order; approximate equality
cannot certify a bitwise contract.

The partition matrix must distinguish semantic scope path, stable work
partition, and physical shard. It must cover hierarchy-aligned and deliberately
misaligned placements, explicit boundary/halo reads, hot-subtree splitting, and
rebalancing. Each placement must preserve partition/access/result identity and
must report boundary reads, cross-shard bytes, residency, and migrations.

Milestone 17 must cross real external boundaries:

- a real WASM worker/host boundary for worker claims
- a real child-process or network loopback boundary for distributed claims
- serialized versioned submissions and results rather than in-memory artifact
  reuse
- duplicate, delayed, reordered, lost, corrupt, incompatible, and stale-epoch
  result delivery
- crash before execution, during execution, after retained result, and during
  response delivery
- stable idempotency identity, typed indeterminate outcomes, and recovery
  handles
- explicit worker-unavailable and product-declared WASM fallback behavior
- separate wire identities for admitted semantic locality and replaceable
  physical shard placement
- backend-specific shard layouts and reordered physical arrival that preserve
  semantic work and canonical publication
- missing, stale, or mismatched locality/partition/shard-plan/boundary-read/epoch
  result packets rejected before publication

An accelerator backend may be claimed only from a real adapter on named
hardware running the common conformance suite. A trait, mock device, or CPU
implementation labeled as an accelerator is negative evidence, not support.

Every performance report in Milestones 14-17 must name total work, span,
critical path, active versus leased workers, queues, steals, barriers, conflict
partitions, publication breadth, nested leases, partitions, reductions,
transient and peak memory, transferred bytes and round trips where applicable,
fallbacks, workload distribution, hardware/runtime versions, cold/warm
posture, repetitions, variance, and percentiles. Elapsed time cannot compensate
for a structural-bound failure.

3. The dual-history branch and restoration torture test
Purpose

Prove that Phase 5 is real: snapshots, lineage, replay, branchable evaluation, restore semantics, memoized reuse history, and future bridge-grade causality all behave coherently through time.

This is the â€œtime and history are first-classâ€ test.

Scenario

Build a scenario with:

a stable baseline graph

several expensive derived artifacts

keyed families

partial outputs / partitions

branchable evaluation state

snapshot capture before and after meaningful transitions

Then execute a deliberately twisted history:

Baseline evaluation on branch main

Capture snapshot S1

Apply upstream change A

Evaluate a subset of targets only

Capture snapshot S2

Apply upstream change B

Force a transaction failure during recompute of one expensive node

Roll back

Re-run successfully

Fork branch explore

On explore, apply C, D, E

Restore S1 on explore

Re-evaluate under a different target request pattern

Memoized-reuse some artifacts

On main, continue with F, G

Restore S2 on main

Continue forward again

Merge selected branch results logically at the host layer or through future branch semantics

Replay both histories from snapshots and from recorded events

Compare all artifact ancestry and explanations

The key is that the same artifact families must experience:

replacement

refresh with meaningful diff

refresh with no downstream change

invalidation without immediate recompute

restore

memoized reuse after restore

branch divergence

branch convergence markers

What this probes

It simultaneously probes:

snapshot fidelity

restore semantics

branch-local state isolation

lineage continuity

artifact identity evolution

replayability of historical paths

divergence and reconvergence semantics

current-state explanation vs historical explanation

readiness for bridge-carried causality

distinction between signal lineage and truth lineage

Required verification output
A. Branch-state divergence map

For each branch and checkpoint, emit:

active snapshot base

node-state digest

artifact-state digest

dirty/maybe-stale/clean map

dependency graph digest

planned targets

realized evaluations

Pass condition

Branches must remain isolated after fork. Restoring or recomputing on one branch must not mutate the other. Equivalent historical suffixes on equivalent bases must converge to identical digests.

B. Artifact ancestry graph

For selected artifacts across time, emit a graph of:

artifact ID

node

parent artifact ID

transition kind

execution record ID

semantic segment ID

branch ID

snapshot restore associations

invalidation events

memoized reuse references

Pass condition

The ancestry graph must be acyclic where derivational parentage is claimed, branch-aware, and historically coherent.

Specific required checks:

restore must not masquerade as derivational parentage

memoized reuse must not masquerade as fresh recomputation

invalidation must not create fake artifact replacement

branch-local artifacts must remain branch-local unless merge semantics explicitly say otherwise

C. Historical explainability report

For selected checkpoints, emit two views:

current-state explanation
Why is this node in its current state right now?

historical evolution explanation
How did this artifact or node state evolve from S1 through the current checkpoint?

Pass condition

The current-state explanation and historical explanation must agree, but not collapse into each other.

That proves the runtime can answer both:

â€œwhy is it this way now?â€

â€œhow did it get here?â€

Without Phase 5, this usually breaks.

D. Replay branch parity report

Replay each branch history by:

snapshot restore + suffix events

raw event log replay from origin

fresh recomputation from equivalent truth snapshots

Pass condition

For each branch checkpoint:

final node-state digest identical

final artifact-state digest identical

lineage digest identical

explanation digest identical

That proves branchable replay is not just visually plausible but semantically closed.

E. Causality threading report

Attach mock host causality metadata at change boundaries, such as:

truth transaction ID

patch ID

changed entity/aspect references

Then verify it appears consistently through:

invalidation provenance

explain outputs

lineage records

replay artifacts

Pass condition

The same host causality token that originated the change must remain traceable through all affected signal-side historical surfaces.

This is not full Phase 6 bridge completion, but it proves Phase 5 is bridge-ready instead of being a dead-end local history model.

Why this is ultimate

Because this is the test that proves WORTH Signal is not only an incremental runtime, but a time-aware execution substrate with honest historical semantics.

If this fails, snapshots and lineage are just decorative features.
If it passes, you have something much rarer.

What these three tests collectively prove

Together they prove the whole promise stack:

Test 1 proves

deterministic semantics

transactional semantics

replay semantics

planner/executor parity

provenance stability

Test 2 proves

precision invalidation

suppression correctness

granular recomputation boundaries

memoization validity

explainable efficiency

Test 3 proves

snapshot/restore truthfulness

branch-local historical correctness

lineage honesty

replay across time

bridge-ready causality threading

The most important meta-rule

For all three tests, the verification output must be based on canonicalized machine-checkable artifacts, not â€œdeveloper looked at logs and it seemed right.â€

That means every test should emit canonical digests for:

node states

artifacts

dependency shape

explanations

lineage streams

branch histories

replay outcomes

If you do not make the outputs canonical and comparable, you will accidentally turn a truth-runtime claim into a vibes-runtime claim.

S9.15 bounded merge closeout note

The bounded merge substrate is considered closeable only if the crate-level
test bundle proves all of the following on supported paths:

- merge planning lowers from `MergeBoundaryWitness` through
  `StructuralMergeJournalSlice`, `ProofMinimalOverlapBasis`,
  `ConservativeOverlapExpansion`, `PlannedMergeCandidateSet`, and
  `LoweredMergePlan`
- supported merge candidate construction never depends on whole-live branch
  scans or ambient branch-state discovery
- proof-minimal overlap and conservative expansion remain distinct bounded
  phases rather than two names for the same candidate set
- repeated merge and restore preserve future bounded-merge boundary validity
- convenience performance-only index churn does not change lowered merge
  candidates

Minimum named crate-level evidence:

- `tests::merge_adoption::merge_branch_without_established_journal_boundary_fails_explicitly`
- `tests::merge_adoption::merge_branch_uses_branch_local_mutation_scope_instead_of_whole_live_scan`
- `tests::merge_adoption::proof_minimal_overlap_and_conservative_expansion_remain_distinct_and_bounded`
- `tests::merge_adoption::merge_candidate_construction_is_identical_with_and_without_convenience_branch_indexes`
- `tests::merge_adoption::active_restore_reinstates_branch_merge_ledger_boundary_for_later_fast_forward_merge`
- `tests::merge_adoption::repeated_merge_after_target_restore_stays_bounded_and_history_honest`

Negative-space rule:

- any future supported merge path that reintroduces `MergeCandidateScope`,
  whole-live supported candidate scope, executor-side candidate discovery, or
  convenience-index-dependent candidate shaping is a certification regression

1. The topology churn test

This targets graphs whose dependency shape changes constantly during evaluation.

Why it matters

Geometry kernels and game systems both have workloads where the set of dependencies is not stable:

a boolean operation touches different faces/edges after topology changes

a solver watches different constraints after a branch of logic flips

an AI or gameplay system changes which entities it observes based on world state

a query family suddenly fans out across a different subset

A runtime can look fine on stable DAGs and still fail once dependency capture changes every frame or every operation.

What to stress

dynamic dependency discovery that changes shape every evaluation

repeated add/remove of dependencies on hot nodes

oscillation between dependency sets A and B

conditional nodes whose conditions themselves depend on dynamic reads

partition subscriptions that grow and shrink aggressively

repeated invalidation during topology shape churn

What to verify

no stale edges remain after dependency reshaping

no phantom dependencies accumulate

no under-invalidation after dropping and re-adding dependencies

dependency inspection always reflects the current graph truthfully

replay reproduces the same dependency shape history

planner cost does not explode under churn

This flushes out â€œdependency leakâ€ bugs fast.

2. The fanout shockwave test

This targets huge invalidation blast radii.

Why it matters

In geometry and game workloads, some upstream nodes are effectively â€œstructural hubsâ€:

transform hierarchies

world-space caches

canonical mesh state

material/visibility sets

simulation clocks

global constraint summaries

A single small change can cause giant maybe-stale cones. The runtime has to stay correct and bounded.

What to stress

one hot node with enormous downstream fanout

mixed aspect subscriptions so only part of fanout should truly propagate

chained high-fanout layers

selective target evaluation after broad invalidation

repeated updates to the same hub with small semantic deltas

What to verify

invalidation breadth matches actual semantic reach

downstream suppression prevents recompute floods when outputs are unchanged

telemetry and explanation remain bounded and usable

planner/runtime overhead grows acceptably with fanout

no queue duplication, repeated re-dirtying, or pathological re-enqueue behavior

This finds â€œworks in principle, collapses under breadthâ€ problems.

3. The deep chain numerical stability test

This targets long dependency chains where small differences cascade.

Why it matters

Geometry kernels and simulation/game systems often have:

long transform chains

derived bounds from derived bounds

repeated tolerance-based suppression

incremental updates where tiny numeric differences matter sometimes and not others

A runtime can become semantically inconsistent when comparator policies interact with deep chains.

What to stress

long linear chains and layered diamonds

exact comparators mixed with tolerance comparators

repeated tiny upstream deltas near epsilon thresholds

alternating significant/insignificant updates

restore/replay after threshold-boundary behavior

What to verify

same seeded run always crosses thresholds at the same points

comparator suppression never hides semantically required downstream work

replay reproduces the same suppression history

explanations show why a threshold suppressed or allowed propagation

branch/replay does not diverge due to threshold instability

This is especially important before geometry kernels, because tolerance semantics are where trust dies quietly.

4. The hot/cold locality skew test

This targets real workload skew instead of balanced synthetic graphs.

Why it matters

Real systems are not uniform.

You usually get:

a tiny set of super-hot nodes

a huge cold tail

occasional cold-to-hot promotions

repeated reads of the same keyed families

sparse mutations against a giant graph

A runtime can benchmark well on smooth graphs and still perform terribly under skew.

What to stress

a giant graph with 1â€“5% very hot nodes

repeated updates concentrated on hot regions

occasional structural changes in cold regions

heavy snapshotting and diagnostic collection while hot set churns

repeated same-target pulls

What to verify

planner cost is not dominated by cold graph scan

snapshot and diagnostics do not scale with untouched cold regions

memoization and query caches remain effective under skew

cold regions do not experience accidental dirtiness

locality-sensitive storage assumptions hold

This is one of the best pre-production realism tests.

5. The orphaned-history test

This targets lineage and snapshot correctness under deletion, replacement, and branch discard.

Why it matters

Geometry and game systems both produce lots of â€œthis thing used to matter, now it does notâ€ history:

geometry entities disappear after a boolean

a procedural generation branch is abandoned

cached artifacts get invalidated and replaced repeatedly

branch-local experiments are discarded

History systems often get confused when the current world no longer contains the old objects.

What to stress

artifact replacement chains

invalidation without replacement

snapshot restore of artifacts no longer current

branch discard after many lineage events

repeated remove/recreate of logically similar artifacts

branch merge where one side references dead history

What to verify

lineage remains queryable for non-current artifacts

deletion/discard does not corrupt ancestry

restore does not fabricate derivation where none exists

history for dead branches remains coherent if retained

GC/compaction policies do not break historical explanation

This is very important for â€œtruthful past, changing presentâ€ systems.

6. The determinism under hostile scheduling test

This is a stronger version of serial/parallel parity.

Why it matters

Game engines especially will pressure you toward aggressive scheduling. Geometry kernels may too once expensive derived work gets parallelized.

It is one thing to match serial and parallel once. It is another to remain deterministic under ugly executor behavior.

What to stress

randomized worker wake order

randomized task steal order

artificial stalls

artificial preemption points

repeated runs with same logical inputs but different execution timing

stage-local parallel work with different chunking strategies

What to verify

final state digest identical across all runs

lineage identical

explain output identical after canonicalization

no hidden race-dependent dependency capture

no ordering-sensitive comparator or memoization behavior

If this fails, the runtime is not really deterministic â€” it is just lucky.

7. The corruption resistance test

This targets bad host integrations and half-broken runtime state.

Why it matters

Before geometry or game use, you want to know how the runtime behaves when the host lies, WORTHts, or violates a contract.

Because real integrations will.

What to stress

Inject deliberate bad behavior:

invalid host snapshot version metadata

wrong aspect mapping

reused structural key for semantically different artifact

missing causality metadata

duplicate node registration attempts

stale snapshot restore request

branch switch to unknown lineage state

illegal cycle introduction attempt during dynamic rewire

What to verify

runtime rejects impossible states loudly

diagnostics identify contract violation class clearly

rollback restores consistency after failed operations

bad host metadata cannot silently poison causal story

corruption is contained rather than spreading

This is less about elegance and more about survivability.

8. The memory pressure and retention test

This targets long-lived systems.

Why it matters

Game editors, CAD tools, simulations, and runtime authoring tools may stay alive for hours. History, diagnostics, memoization, snapshots, and lineage can quietly become a memory disaster.

What to stress

long run with thousands of snapshots

bounded diagnostics retention

lineage retention with compaction policy

repeated branch creation/discard

query-family cache churn

repeated hot/cold region transitions

What to verify

memory growth follows policy instead of drifting upward

retained history stays queryable within declared bounds

compaction does not corrupt replay or explanation for retained windows

dropped history fails explicitly, not ambiguously

performance does not degrade sharply after long uptime

A lot of runtimes fail here before they fail anywhere else.

8A. The observation and managed-resource long-session extension

This extends the long-session requirement to runtime-managed observation
resources and any future higher-level abstraction that lowers into them.

Why it matters

Observer populations, watchers, effects, and future category adapters like
forms or resources can quietly become a second memory and breadth disaster even
when snapshots, lineage, and diagnostics stay bounded.

What to stress

long run with thousands of observer registrations and teardowns

mixed persistent and short-lived observer populations

branch-local observer or watcher churn

restore after large observer churn history

observer index maintenance pressure

What to verify

disposed resources stop exerting semantic influence on future transaction
boundaries

matching breadth does not silently drift upward with historical churn

delivery breadth stays bounded by active relevant observers, not by dead
registrations that were never cleaned up honestly

long-session observer churn remains attributable through named counters rather
than inferred from latency alone

9. The oscillation test

This targets systems that flip back and forth between nearby states.

Why it matters

Games and simulations do this constantly. Geometry tools do it whenever a user drags a handle or scrubs a parameter.

What to stress

repeated A â†” B â†” A â†” B updates

threshold-boundary oscillation

dependency-shape oscillation

branch restore to an old state followed by return to newer state

memoized reuse opportunities across oscillation

What to verify

no cumulative dependency garbage

no lineage corruption from repeated revisit of similar states

memoized reuse is correct, not stale reuse

output suppression stays stable and does not â€œlearn the wrong thingâ€

replay reproduces oscillation exactly

This flushes out state contamination bugs.

9A. The future abstraction lifting rule

Every future higher-level abstraction built on `worth-signal` must certify its
truth by reduction to substrate invariants rather than by abstraction-local
happy-path behavior alone.

This includes, but is not limited to:

forms

resources

outputs

workflow projections

UI adapters

Normative consequence

- a form layer is not considered honest because "dirty fields update correctly"
- a resource layer is not considered honest because "loading and ready states
  render correctly"
- an effect layer is not considered honest because "callbacks fired in the
  expected demo"

Instead, each abstraction must prove which core runtime invariants it depends
on and certify reduction to those invariants:

commit-bounded delivery

rollback suppression

replay parity

branch/restore parity

suppression and classification truth

resource lifecycle and teardown honesty

boundedness counters

If a future abstraction cannot describe itself in those substrate terms, the
abstraction is trying to invent a second runtime truth model.

9B. The future abstraction workload grammar

Any future category-specific certification suite must instantiate the same
hostile workload grammar before adding category-specific expectations.

The grammar includes:

multiple writes before commit

recompute without meaningful output change

rollback after staging work

branch fork

branch-local divergence

snapshot restore

branch restore

merge or convergence where admitted

subscribe or acquire

unsubscribe or dispose

long-session churn

diagnostics-tier variation

skewed hot-set pressure

A future form or resource suite may add domain-specific assertions, but it must
still prove itself against this substrate grammar first.

10. The hostile domain adapter test

This is the most practical one before geometry kernels or game engines.

Why it matters

The runtime itself may be solid, but the integration seam is where domain bugs enter.

So build fake-but-brutal adapters that mimic domain patterns without needing the real kernel or engine yet.

Two adapters I would build
A. Fake geometry adapter

Model:

entities with topology-like IDs

face/edge/region-style partitions

tolerance-sensitive derived summaries

structural edits that remap dependency neighborhoods

boolean-like operations that replace many artifacts at once

Verify:

topology churn

partition-aware diffing

lineage through replacement and restore

tolerance gating behavior

replay after structural edits

B. Fake game-world adapter

Model:

entity/component graph

transforms

visibility/culling summaries

AI perception queries

frame-like repeated updates

hot moving subset inside large cold world

Verify:

hot/cold skew

fanout from transforms/global state

dynamic dependency capture

per-frame determinism

bounded diagnostics across long sessions

This gives you domain-shaped pressure before the actual domains add their own complexity.

10A. The substrate boundedness and lifting test

This proves that future abstractions can only become honest product surfaces by
inheriting the runtime's semantic and boundedness contracts, not by hiding
broader scans, extra coordination, or new truth semantics behind convenience
APIs.

Scenario

Create at least one minimal representative harness for each admitted future
category once it exists, such as:

form

resource

output or view-model

effect or watcher adapter

Run each harness through the hostile workload grammar and emit:

its lowered runtime footprint

its observer or managed-resource count

its matching breadth

its delivery breadth

its rollback and replay parity result

Pass condition

Every higher-level category must be demonstrably reducible to substrate
contracts without introducing a second semantic engine or hiding a broader cost
surface than the runtime counters admit.

If I had to pick the highest-value additional set

If you do not want ten more tests, I would prioritize these five:

Topology churn

Fanout shockwave

Deep chain numerical stability

Determinism under hostile scheduling

Memory pressure and retention

Those five, combined with the original three, would expose most of the serious hidden risks before geometry kernel or game engine adoption.

The meta-principle

Before using this on geometry kernels or game engines, you want proof across five axes:

semantic correctness

historical correctness

scheduler correctness

precision under hostile graph shape

boundedness over long-lived runs

The original three hit the first two hardest.
These extra tests hit the last three.

11. The temporal eligibility replay parity test

This is the first non-optional certification gate for making time a real
runtime primitive instead of a host convenience.

Why it matters

If temporal eligibility changes between ordinary execution, replay, restore, or
branch re-entry, then "debounce", "delay", "stale-after", "timeout", and any
future time-shaped policy are all lies wearing nicer names.

For a high-trust runtime substrate, "the same clock basis and same inputs
produced different admission decisions" is not a performance bug. It is a
truth failure.

What to stress

seeded temporal workloads containing:

time-gated nodes

previous-value-sensitive nodes

multiple temporal policies in one graph

at minimum mixes of:

after

at-or-after

debounce

throttle

stale-after

interval

branch fork before a scheduled wake becomes ready

snapshot restore to a point before and after temporal admission

replay from checkpoint plus bounded history

long gaps with no host writes where only time advance can make work eligible

oscillation around eligibility boundaries

What to verify

for the same authoritative input history and same clock basis:

temporal eligibility decisions are identical

wake ordering is identical after canonicalization

committed outputs are identical

explanations of why work became eligible are identical

branch restore and replay do not invent or erase temporal wake history

Pass condition

The runtime must emit canonical digests for:

clock checkpoints

scheduled wake sets

ready ordering

eligibility decisions

committed outputs

temporal explanations

All equivalent runs must match exactly except for fields explicitly declared
non-semantic.

12. The temporal branch restore equivalence test

This proves that time is branch-honest instead of being ambient process state
that leaks across histories.

Why it matters

If one branch can advance temporal eligibility for another branch, or if a
restore replays a different timer story than the original history, then the
runtime does not really support branchable time semantics.

What to stress

branch fork while multiple temporal wakes are pending

different time-advance scripts on each branch

restore to checkpoints on each branch repeatedly

re-admission of work after restore

mixed host invalidation plus time-driven invalidation

What to verify

branch-local temporal state stays isolated

restoring a branch reproduces the same ready set the original history had at
that checkpoint

equivalent suffixes from equivalent restored checkpoints converge to identical
digests

temporal explanations stay branch-local and checkpoint-honest

Pass condition

For every branch/checkpoint pair, the runtime must emit:

clock basis digest

scheduled wake digest

ready queue digest

node-state digest

explanation digest

Equivalent restored suffixes must produce identical canonical digests.

13. The temporal wake boundedness test

This is the anti-fake-performance test for time.

Why it matters

A runtime can claim temporal support while secretly doing broad timer scans,
per-node polling, or repeated heap churn that only looks acceptable in toy
graphs. That is not good enough for long-lived enterprise state management.

What to stress

very large graphs with a small temporal frontier

bursty timer readiness where only a few nodes become eligible per advance

long idle periods

branch-local pending wakes under heavy churn

thousands of repeated clock advances with sparse eligibility changes

large elapsed jumps across many interval periods with different missed-tick
policies

node replacement or policy rewrite while many unrelated wakes remain pending

What to verify

ready-node discovery scales with the temporal frontier, not total graph size

long idle periods do not trigger broad scans

dead or restored wakes do not stay resident forever

named counters can prove where temporal work went

interval catch-up behavior scales with the admitted missed-tick policy outcome,
not with raw elapsed-period count when the policy does not require that work

branch restore and policy rewrite do not rebuild temporal readiness by whole
registry scan

Pass condition

The runtime must expose and certify counters for at least:

temporal wake count

deferred-by-time count

ready queue width

broad temporal scan denial count

branch-local temporal restore count

Any implementation that cannot make those costs attributable is not honest
enough to ship.

14. The previous-value and time-gated node equivalence test

This proves that previous-value access is a semantic feature, not a hidden
side-channel.

Why it matters

Many of the most valuable temporal behaviors depend on comparing the current
candidate world to the last committed world. If previous-value reads are not
transactional, branch-honest, and replayable, every debounce/windowing/fresh
state rule built on top will drift.

What to stress

nodes that compare current candidate values to last committed values

nodes whose eligibility depends on both previous-value and time windows

rollback after previous-value-sensitive work was staged

restore to checkpoints before and after previous committed transitions

replay of threshold-boundary transitions

What to verify

previous-value reads always refer to the same committed history across normal
execution, replay, restore, and branch re-entry

rollback does not leak staged-but-uncommitted previous values

explanations distinguish:

current candidate value

last committed value

temporal basis

Pass condition

Equivalent histories must produce identical canonical digests for:

current outputs

previous-value references

eligibility decisions

rollback suppression behavior

explanation artifacts

15. The async resource lifecycle parity test

This is the foundational async honesty test.

Why it matters

If pending, fulfilled, rejected, cancelled, stale, and superseded states are
not canonical runtime truths, every future resource, route loader, form submit
flow, cache layer, and query replacement will invent its own state machine.

That is exactly how global systems become impossible to audit.

What to stress

async/resource nodes with:

overlapping admissions

revalidation while work is still inflight

success, failure, cancellation, and retry paths

branch fork with inflight work

snapshot restore with inflight work

replay from checkpoint plus completion history

What to verify

resource lifecycle transitions are identical across equivalent executions

completion handling re-enters the runtime transactionally

rollback suppresses failed completion commits fully

observation remains commit-bounded even when driven by async completion

Pass condition

The runtime must emit canonical digests for:

request identity

lifecycle transitions

committed resource state

observation boundaries

diagnostics and explanations

Equivalent runs must match exactly after canonicalization.

16. The out-of-order completion supersession test

This is the first true async nightmare test.

Why it matters

Real systems do not complete in order.

Requests race each other.

Networks stall.

Retries come back after later revalidations already succeeded.

User intent moves on while old work is still alive.

If old completions can overwrite newer admitted intent, the runtime is not safe
for serious state management.

What to stress

issue request R1

before R1 completes, invalidate and admit R2

before R2 completes, admit retry or replacement R3

complete R2, then R1, then R3 in hostile order

mix success, failure, timeout, cancellation, and explicit supersession

repeat under branch fork, restore, and replay

What to verify

only the newest admitted generation allowed by policy may commit

older completions are denied explicitly as stale or superseded

denial classification is stable across replay

observers never see impossible intermediate truths

Pass condition

The runtime must produce a machine-checkable denial report containing:

request identity

generation or epoch identity

completion order

admission outcome

denial classification

committed winner

If any stale completion commits in any equivalent run, the phase is not
certifiable.

17. The async rollback and observation equivalence test

This proves that async completion cannot punch holes through transaction
boundaries.

Why it matters

One of the ugliest async failure modes is "the state rolled back, but some
observer, effect, or cache already saw the half-committed result."

That bug destroys trust quietly because the final state can still look right.

What to stress

completion handling that fails after staging resource updates but before commit

multiple observers with overlapping scopes

unsubscribe during completion churn

retry after rollback

mixed sync and async invalidations inside the same hostile workload

What to verify

no observer delivery escapes from the failed completion transaction

post-rollback state equals the pre-transaction digest exactly

retry delivery equals the no-failure control path

completion-driven observation remains one packet per committed boundary per
observer

Pass condition

The runtime must emit:

pre-transaction digest

staged-but-suppressed observer packet digest

post-rollback digest

retry digest

per-observer delivery digest

The rollback lane must be indistinguishable from "the failed completion never
committed."

18. The async branch restore and replay equivalence test

This proves that inflight work and completed resource truth survive historical
machinery honestly.

Why it matters

Async state is where many runtimes quietly give up on replay honesty. They
replay values but not lifecycles, or they restore outputs but lose the story of
which work was still inflight and which completion was denied.

That is not acceptable for a system that may need to explain why a decision was
made later.

What to stress

branch fork with inflight requests

different completion orders on each branch

restore to checkpoints before and after completion

replay from checkpoint plus completion stream

long-lived inflight work spanning multiple unrelated sync transactions

What to verify

branch-local inflight state stays isolated

restore reconstructs the same inflight and committed resource story the branch
originally had

equivalent replay produces identical lifecycle and denial digests

explanations can answer both:

why is the resource in this state now

how did this request lifecycle evolve

Pass condition

For each branch checkpoint, the runtime must emit canonical digests for:

inflight set

committed resource states

lifecycle history

denial history

explanation artifacts

Equivalent histories must converge exactly.

19. The async inflight boundedness test

This is the anti-meltdown test for real async pressure.

Why it matters

The runtime is going to be judged under unreliable connections, partial
connections, duplicate delivery, cross-region latency, retries, operator
churn, and long-lived sessions.

If inflight tracking, cancellation, retry bookkeeping, or completion matching
widens into broad scans, hidden maps, or retention leaks, the system will fail
operationally long before it fails semantically.

What to stress

large graphs with a small active inflight surface

large inflight populations with sparse completion activity

heavy cancellation and retry churn

long sessions with repeated acquire, supersede, cancel, and dispose cycles

branch churn while inflight work remains live

What to verify

completion matching scales with inflight-local structures, not total graph size

dead, cancelled, or superseded inflight records do not accumulate forever

named counters explain inflight cost posture

retention policies fail explicitly when history is outside the retained window

Pass condition

The runtime must expose and certify counters for at least:

inflight request count

fulfilled count

rejected count

cancelled count

superseded completion denial count

retry admission count

inflight broad-scan denial count

async branch restore count

If the runtime cannot show boundedness with counters and canonical digests, it
is not ready for hostile production workloads.

19A. The worst async nightmare grammar

Every future async/resource certification suite must instantiate this workload
grammar before it adds any higher-level expectations.

The canonical async failure families are:

completion ordering failures

completion integrity failures

request identity failures

liveness failures

async pressure failures

The grammar must instantiate at least one hostile case from each family.

Completion ordering failures include:

multiple admissions before any completion

out-of-order completion

duplicate completion delivery for the same request

success after timeout

failure after supersession

cancellation racing completion

retry racing fresh admission

Completion integrity failures include:

broken connection before completion delivery

partial connection with delayed completion delivery

contradictory completion reports for the same request

partial payload delivery

completion with impossible status or timing claims

host metadata omission or corruption

Request identity failures include:

completion with missing or corrupted request identity

completion for an unknown request

completion for a retired, cancelled, or superseded request

completion that lies about generation or attempt identity

Liveness failures include:

lost completion

ghost inflight state

zombie completion after the runtime moved on

timeout truth split between runtime and host boundary

Async pressure failures include:

long-session churn with acquire and dispose

retry storms

completion floods

inflight buildup with sparse resolution

starvation under repeated supersession or retry pressure

This is the minimum hostile async shape.

If a future abstraction cannot survive this grammar, it does not deserve to be
called first-class.

19B. The regulated-system adversarial rule

If `worth-signal` is going to become a serious global runtime substrate, then
test requirements must assume hostile operators, hostile schedulers, hostile
clocks, broken or partial connections, dishonest completions, duplicate
delivery, identity mismatch, liveness failure, and long-lived sessions instead
of "well-behaved app code."

Normative consequence

No temporal or async feature is considered complete because:

it works in a demo

it survives one serial path

it survives one replay path

it can usually recover after failure

Instead it must prove:

deterministic committed truth

explicit denial of impossible or stale work

rollback integrity

branch and restore honesty

replayable causal explanation

bounded hot-path cost

bounded long-session retention

For high-trust workloads, the runtime must be able to answer:

what committed

what was denied

why it was denied

what would replay identically

which completions were rejected as malformed, partial, contradictory, or
dishonest

what history was intentionally dropped by policy

If it cannot answer those questions with machine-checkable artifacts, it is not
yet honest enough.

20. The async resource policy family certification test

This is the test family that proves async policy is a runtime substrate, not
adapter folklore.

Why it matters

Async behavior is where product assumptions usually leak into infrastructure:
retry delay, retry budget, timeout scope, output visibility, cancellation
propagation, revalidation, retained history, and diagnostic richness all look
like local choices until branch restore, replay, or a regulated audit asks why
the runtime did what it did.

If these choices are not deterministic policy descriptors, every higher-level
resource surface will create a second state machine.

What to stress

Run equivalent async/resource workloads across declared policy families for:

retry and backoff:

disabled retry

fixed delay

exponential backoff

capped exponential backoff

deterministic jitter

max attempts

max elapsed retry window

failure-class-based retry

node, family, runtime, and declared-scope retry budgets

duplicate pending retry coalescing

timeout and deadline:

disabled timeout

fixed timeout

transaction/runtime inherited deadline

per-attempt timeout

total request-lifetime timeout

progress-heartbeat extension

terminal timeout

timeout as revalidation-eligible

cancellation and supersession:

runtime-hard cancellation

best-effort host cancellation signal

cancellation grace period

cancellation after supersession

dependent-resource cancellation propagation

newest-generation-wins supersession

overlapping-generation policy

intent-equivalence coalescing

leave old host work running while denying completion

cancel old host work on supersession

revalidation and freshness:

explicit revalidation

stale-after revalidation

dependency-change revalidation

observer-demand revalidation

terminal-state revalidation

fulfilled-only revalidation

forced revalidation with active-handle proof

deduped and coalesced revalidation

observation and output continuity:

lifecycle-only observation

output-continuity observation

denied-completion observation

retry-schedule observation

per-transaction coalesced observation

preserve previous output while pending

hide previous output while pending

preserve or hide output after rejection, timeout, cancellation, or supersession

retention, diagnostics, and replay compatibility:

retain all lifecycle transitions

retain terminal summaries only

retain denied completions by budget

retain retry lineage by budget

compact superseded, cancelled, and timed-out records

retained-history unavailable classifications

diagnostics expansion budgets

policy version compatibility and incompatibility

What to verify

Policy identity:

every decision carries policy id, semantic name, version, digest, and selection
basis

changing policy parameters changes descriptor digest

unknown, duplicate, or incompatible policies deny before execution work is
constructed

Lifecycle law preservation:

policy may alter eligibility, timing, visibility, retention, or diagnostics

policy may not alter request identity, generation, attempt lineage, branch
epoch matching, stale completion denial, or denied-completion non-apply

Replay and branch parity:

deterministic jitter replays identically

budget exhaustion replays identically

policy-compatible restore emits compatibility proof

policy-incompatible restore emits typed denial

Performance honesty:

retry cost reports decision width, temporal wake footprint, and budget-scope
touches

timeout cost reports temporal frontier width and affected request count

cancellation and supersession cost report affected request footprint and
host-signal advisory width separately

revalidation cost reports active-handle proof checks and coalescing width

observation cost reports candidate width, coalesced width, and delivery width

retention and diagnostics cost report retained-summary reads, cold
reconstruction, pruned records, and diagnostics budget consumption separately

Pass condition

The runtime must emit canonical policy certification artifacts containing:

policy registry digest

policy descriptor digest

policy selection basis

policy decision trace

resource lifecycle digest

output-continuity digest

retry lineage and budget digest

timeout/deadline digest

cancellation/supersession digest

revalidation/freshness digest

observation delivery digest

retention and diagnostics digest

replay compatibility or incompatibility artifact

boundary performance envelope

Equivalent runs must match exactly when policies are compatible. Incompatible
policy history must deny explicitly; it must never silently reinterpret old
async truth.

20A. The async policy registry boundary test

Purpose

Prove that async/resource policy extensibility behaves like the existing
strategy registry surfaces rather than like loose application callbacks.

What to stress

register duplicate policy ids

register duplicate semantic names

reference unknown policy names from resource declarations

restore from a checkpoint with missing policy descriptors

restore from a checkpoint with incompatible policy versions

try to construct policy descriptors or force tokens outside the proving module

What to verify

duplicate registrations fail before runtime construction

unknown policy declarations fail before descriptor lowering completes

missing or incompatible restore policy descriptors produce typed compatibility
denials

private constructors prevent WORTHd policy descriptor proofs

Pass condition

No policy decision can enter request admission, temporal wake allocation,
completion admission, transaction apply, observation, or replay unless it has
lowered through a frozen deterministic descriptor.

21. The async capability attachment equivalence test

Purpose

Prove that async capability attached to ordinary nodes is the same substrate as
the legacy resource-shaped path rather than a second implementation wearing the
same words.

Why it matters

If capability-first declarations and legacy resource-shaped declarations lower
through different descriptor, lifecycle, replay, or denial paths, the runtime
will quietly fork into two async systems.

That would erase the whole point of Milestone D.

What to stress

declare semantically identical async nodes through both:

- capability-first ordinary-node attachment
- legacy resource-shaped compatibility vocabulary

Include at minimum:

- plain leaf async nodes
- keyed/query/computed-family async nodes
- condition-gated async nodes
- nodes with non-default output continuity and observation policies

Run equivalent workloads across both declaration paths including:

- fresh admission
- retry
- timeout
- cancellation
- revalidation
- restore and replay

What to verify

both declaration forms lower to identical canonical descriptor truth

request identity law is identical

lifecycle transitions are identical

denial classifications are identical

observation and output continuity digests are identical

replay and restore digests are identical

Pass condition

The runtime must emit canonical equivalence artifacts containing:

- capability declaration digest
- lowered descriptor digest
- lifecycle digest
- denial digest
- observation/output-continuity digest
- replay/restore digest

Equivalent capability-first and compatibility-first declarations must match
exactly when they mean the same thing.

22. The interior async node gate equivalence test

Purpose

Prove that an async-capable node can live in the middle of the graph and gate
downstream work without inventing a second dependency model.

Why it matters

If interior async gates are implemented as pseudo-nodes, side channels, or
special adapter-managed boundaries, then async capability is not really part of
the graph. It is just a leaf-shaped convenience with extra tricks.

What to stress

build a graph where:

- upstream sync nodes produce request/admission inputs
- an interior async-capable node depends on them
- multiple downstream nodes depend on:
  - lifecycle truth
  - committed output truth
  - output continuity posture

Include:

- one path where the async node remains pending
- one path where it fulfills
- one path where it times out or rejects
- one path where it preserves prior output while pending
- one path where it hides prior output while pending
- one path where upstream invalidation changes admission legality before the
  current async lineage resolves

Run this under:

- ordinary execution
- branch fork before completion
- snapshot restore before and after completion
- replay from checkpoint plus completion history

What to verify

downstream nodes observe only graph-legal lifecycle/output facts

interior async gating does not introduce hidden graph edges or bypass ordinary
dependency semantics

pending lifecycle does not masquerade as node dirtiness

failed or timed-out async gate state preserves correct downstream semantics

restore and replay reconstruct the same gate story and downstream consequences

Pass condition

The runtime must emit canonical artifacts containing:

- interior gate descriptor digest
- lifecycle digest
- downstream dependency-state digest
- output continuity digest
- replay/restore digest
- explanation digest

Equivalent executions must converge exactly.

23. The hierarchical async capability replay and cancellation test

Purpose

Prove that async-capable nodes can depend on other async-capable nodes while
preserving one coherent runtime story for cancellation, retry, replay, and
restore.

Why it matters

This is where naive implementations split apart:

- parent/child cancellation becomes host folklore
- replay restores outputs but not inflight hierarchy truth
- child completion races produce illegal parent admission
- branch restore WORTHts which async layer was still authoritative

What to stress

build at least a three-level async-capable dependency chain:

- parent async-capable node
- child async-capable node
- grandchild async-capable node

Include hostile cases for:

- parent cancellation propagating downward
- child failure causing parent revalidation or denial
- grandchild completion arriving after parent lineage moved on
- retry at one layer while another layer is still pending
- branch fork while multiple layers are inflight
- restore to checkpoints before and after mixed-layer completion

What to verify

hierarchical cancellation footprint is exact and branch-local

retry, timeout, revalidation, and denial history remain layer-honest

replay reconstructs the same multi-level inflight and committed story

no layer can commit stale work once an upstream or downstream dependency made
that lineage non-authoritative

Pass condition

The runtime must emit canonical artifacts containing:

- hierarchical inflight-set digest
- hierarchical cancellation-footprint digest
- lifecycle-history digest
- denial-history digest
- replay/restore digest
- explanation digest

Equivalent hierarchical histories must converge exactly.

24. The condition-gated async admission parity test

Purpose

Prove that conditions and previous-value / temporal gates shape async
admission truth without mutating lifecycle classification truth.

Why it matters

This is the most likely naive trap once async capability moves onto ordinary
nodes. A rushed implementation will turn "blocked by condition" into a fake
pending state, or will treat previous-value/temporal gating as hidden lifecycle
machinery.

What to stress

async-capable nodes that are also:

- on-demand
- debounce-gated
- throttle-gated
- stale-after gated
- previous-value-sensitive
- delta-threshold sensitive

Include workloads where:

- conditions block new async admission while ordinary dirtiness still changes
- revalidation is legal but fresh lineage admission is not
- previous-value and temporal windows together decide admission
- rollback happens after gating state changed but before commit
- restore happens before and after gating flips

What to verify

condition outcomes remain admission truth, not lifecycle truth

blocked async admission does not mint fake inflight lifecycle

previous-value and temporal gates replay identically

restore reconstructs the same admission legality story the original branch had

Pass condition

The runtime must emit canonical artifacts containing:

- condition/admission classification digest
- lifecycle digest
- previous-value reference digest
- temporal basis digest
- replay/restore digest
- explanation digest

Equivalent histories must converge exactly.

25. The async capability compile-time boundary test

Purpose

Prove that arbitrary nodes do not become async-capable by accident, and that
legacy compatibility vocabulary cannot bypass capability-first lowering.

Why it matters

Milestone D will fail quietly if callers can:

- reach async-only surfaces from ordinary node declarations
- construct lowered capability descriptors directly
- use compatibility aliases to skip the capability-first declaration path

That would make the architecture depend on convention instead of enforcement.

What to stress

attempt to:

- use async-only builder methods on ordinary node-only declarations
- construct lowered capability descriptors outside the proving module
- call lifecycle/replay/inspection surfaces that require async capability on
  plain nodes without capability proof
- use legacy resource-shaped constructors to enter async runtime paths without
  capability-first lowering

What to verify

the compiler rejects forbidden construction and access where possible

typed lowering or admission rejects the remainder before runtime execution

compile-fail fixtures stay synchronized with the intended boundary shape

Pass condition

No async-only declaration, lowering, lifecycle, observation, or replay surface
may be reachable without a capability-bearing proof path.
