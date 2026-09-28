10 generic ultimate tests
1. Hostile commit/replay equivalence test
Purpose

Prove that canonical commit artifacts are sufficient to reconstruct observable truth exactly, across success, rollback, savepoints, branches, and replays.

Scenario

Run a long deterministic workload containing:

creates

same-commit graph creation where relations target entities created in the same authoritative commit

updates

deletes

relation edits

nested savepoints

rollback injections

branch creation

branch switches

snapshot capture

restore

lineage-affecting operations

index rebuild publication

mixed entity and relation aspect changes

Then compare four executions:

original authoritative run

replay from canonical commit envelopes

replay from snapshot + suffix commit envelopes

fresh runtime reconstructed from durable canonical artifacts

Must verify

All compared runs produce identical canonicalized:

visible snapshot truth

branch head map

entity and relation iteration order

patch artifacts

diagnostics summaries

lineage graph

historical resolution outputs

replay stream summaries

storage-visible query results

Required verification output

truth_digest

patch_digest

lineage_digest

replay_digest

diagnostics_digest

branch_heads_digest

query_surface_digest

Pass condition

All digests match for all equivalent histories.

This is the single most important relational test.

2. Savepoint rollback fracture test
Purpose

Prove that nested savepoints rewind touched truth precisely, without corrupting untouched truth, lineage, patch preparation, or snapshot visibility.

Scenario

Within one outer transaction:

mutate entities and relations

create nested savepoint A

mutate more

create nested savepoint B

mutate more

rollback to B

mutate alternate path

rollback to A

mutate another alternate path

commit

Also inject failures at:

before final apply

after merged plan generation

after patch fragment preparation

before publication

Must verify

touched-state rewind exactness

no leaked side effects from abandoned savepoint paths

no patch residue from rolled-back paths

no lineage residue from rolled-back paths

final commit reflects only surviving path

Required verification output

per-savepoint touched-entity/relation set

pre/post rollback truth digests

abandoned mutation residue report

patch fragment inclusion report

lineage residue report

final canonical commit artifact set

Pass condition

Rolled-back work leaves zero authoritative residue.

3. Snapshot pinning and reclaim correctness test
Purpose

Prove MVCC retention and reclaim correctness under pinned snapshots, active mutation, and branch retention.

Scenario

create many snapshots at staggered points

pin some by active readers

pin some by branches

pin some by replay retention

mutate hot records repeatedly

release pins gradually

run retention/reclaim cycles

Must verify

historical reads from pinned snapshots remain correct

reclaim never removes data still needed by any pin class

released history becomes reclaimable exactly when expected

reclaim does not alter visible truth or replay for retained windows

Required verification output

lifecycle transition log by record/version:

Live

DeletedRetained

PinnedBySnapshot

PinnedByBranch

PinnedByReplayRetention

Reclaimable

Reusable

pinned snapshot truth digests

reclaim decision summary

post-reclaim read parity report

Pass condition

Every pinned reader sees exact historical truth; every reclaimed payload was truly safe to reclaim.

4. Deterministic observability under hostile scheduling test
Purpose

Prove that internal worker scheduling variability cannot change any observable output.

Scenario

Use the same workload repeatedly while varying:

worker count

fragment preparation ordering

validation worker order

index fragment computation order

diagnostics fragment order

artificial stalls/yields

randomized legal scheduling with fixed seeds

Must verify identical canonical outputs for:

snapshots

public iteration order

patch order

diagnostics order

replay order

lineage order

query results

branch metadata

Required verification output

per-run canonical artifact bundle

scheduling seed / worker topology metadata

mismatch matrix by observable surface

Pass condition

Internal schedules differ, but all observable surfaces are identical.

5. Index non-authority corruption test
Purpose

Prove that derived/secondary indexes never become authority and that storage-visible fallback remains correct under index lag, mismatch, and corruption.

Scenario

Run workloads where you deliberately create:

stale index generation

missing index generation

partially rebuilt index generation

corrupted index payload

index publication failure

index built from older snapshot

Then issue bulk and targeted queries through both:

index-assisted path

authoritative storage fallback path

Must verify

fallback path always returns correct truth

index mismatch is detected diagnostically

missing/corrupt index never changes authoritative read semantics

index publication is version-bound and explicit

index failures do not corrupt commit visibility

Required verification output

query parity matrix

index generation/version map

fallback invocation report

mismatch detection diagnostics

publication acceptance/rejection report

Pass condition

Truth semantics are identical with or without usable indexes.

6. Diff/CDC truth parity test
Purpose

Prove that commit-native diffs faithfully describe the actual committed truth change, including aspects, relation changes, lineage-affecting transitions, and stream resume semantics.

Scenario

Run workloads with:

entity updates

relation updates

same-commit graph creation where CDC must describe one coherent publish boundary rather than an orphan-entity intermediate state

relation-kind changes

replacements

deletes

branch-local commits

savepoint-abandoned work

resume/checkpoint boundaries

stream consumers resuming from checkpoints

Must verify

patch contents match actual truth delta

aspect tagging is exact

relation aspect tagging is exact

abandoned/rolled-back work never appears in CDC

resume from checkpoint produces exactly-once visible stream semantics under declared contract

replay derived from commit envelopes matches CDC-derived expectations

Required verification output

patch_vs_truth_delta_report

aspect_tag_accuracy_report

resume_checkpoint_matrix

subscriber_recovery_report

cdc_order_digest

Pass condition

CDC is complete, precise, canonical, and replay-consistent.

7. Lineage/correspondence hardening test
Purpose

Prove that lineage is authoritative identity evolution, not event-logging theater, and that correspondence stays advisory until explicit promotion.

Scenario

Run identity-evolution workloads containing:

replace

split

merge-like correspondence candidates

branch divergence

branch-local independent replacements

advisory correspondence suggestions

explicit authoritative promotion of selected correspondences

invalid correspondence attempts

ambiguous parentage attempts

Must verify

lineage graph invariants reject ambiguity

storage identity and lineage identity remain distinct

advisory correspondence does not silently become authority

historical ID resolution works through legitimate lineage

branch-local identity evolution remains branch-local until explicit merge/promotion semantics say otherwise

Required verification output

lineage graph export

correspondence candidate set

authoritative promotion log

rejected-invariant report

historical resolution matrix

Pass condition

Only explicitly authoritative lineage affects identity-evolution queries.

8. Merge-ready history shape test
Purpose

Prove that the runtime truly remains merge-ready and does not silently collapse back into linear-history assumptions.

Scenario

Even before actual merge execution is fully enabled, construct histories with:

zero-parent root commits

one-parent normal commits

ordered multi-parent commit-envelope fixtures

replay and diagnostics processing over ordered parent lists

branch comparisons and ancestry queries on multi-parent envelopes

Must verify

ordered parent lists persist through durability, replay, diagnostics, and branch reasoning

APIs do not assume “single parent or none”

parent order is canonical and stable

branch ancestry queries remain correct

Required verification output

parent-list serialization artifacts

ancestry query matrix

replay acceptance/rejection report for merge-ready envelopes

diagnostics summary for parent ordering

Pass condition

The system remains operationally consistent with ordered parent-list history from day one.

9. Bulk query and traversal stress truth test
Purpose

Prove that bulk traversal/query results remain correct, canonical, and scalable under large cyclic graphs, heavy relation counts, and mixed aspect filters.

Scenario

Build a very large cyclic graph with:

multiple entity kinds

multiple relation kinds

relation aspects

branch-local deltas

historical versions

mixed hot/cold regions

Run:

relation-type scans

bulk neighborhood traversals

aspect-filtered bulk reads

snapshot-scoped bulk queries

storage fallback and index-assisted variants

Must verify

exact result parity across query paths

canonical result order

no ping-pong API dependency to answer bulk workloads

snapshot-visible reads are isolated from later mutations

performance stays proportional to requested surface, not whole-graph scans, where the design promises it

Required verification output

query result digests

canonical order reports

path parity matrix

snapshot isolation matrix

touched-state/work-packet metrics

Pass condition

Bulk query surfaces behave as primary, first-class APIs, not stitched-together single-record loops.

10. Durable recovery and schema mismatch test
Purpose

Prove that durable recovery rebuilds authoritative truth from canonical artifacts, and that schema/version mismatches fail explicitly rather than drifting silently.

Scenario

Persist committed history using canonical durable artifacts, then test:

clean recovery

interrupted publication

partial artifact presence

patch present but replay envelope missing

schema mismatch

kind registry mismatch

parent-list mismatch

diagnostics profile mismatch where allowed/not allowed

Must verify

partial durable publication never becomes visible truth

clean recovery reconstructs exact authoritative state

mismatches fail explicitly with structured failure classes

no dependence on transient arena layout

snapshots recover as views, not as primary authority

Required verification output

durable artifact completeness report

recovery truth digest

partial-publication rejection report

schema/kind mismatch report

recovery failure taxonomy summary

Pass condition

Recovery is exact when valid, and loudly rejected when invalid.

13. Invariant extensibility and structural legality certification test
Purpose

Prove that the completed invariant subsystem enforces structural legality as a
truth-runtime authority surface, and that custom structural invariants
participate in the same planning, execution, artifact, and replay contract as
native invariants without opening a semantic type-erasure escape hatch.

Scenario

Run deterministic workloads containing:

native invariant registration and execution

custom invariant registration with stable semantic identity

custom invariant preparation and execution over structural scope

hostile cycle-inducing relation creation

payload schema violations

cross-partition relation attempts

publication-boundary connectivity and minimum-cardinality failures

savepoint rollback of invariant-affecting work

custom invariant panic injection during scope preparation and execution

replay and durable recovery over invariant-bearing histories

Must verify

native and custom invariants share one authority pipeline for registration,
selection, lowering, execution, and artifact shaping

custom invariants cannot access signal or other derived state

custom invariant scope and executable pairing is packet-owned and exact; no
framework-level route-and-downcast mismatch contract is representable

acyclicity rejection is exact and cost-visible under hostile cycle formation

payload schema rejection diagnostics localize exact field/type/constraint
failures

partition isolation rejects forbidden cross-partition relations exactly

publication-boundary failures produce explicit committed-but-unpublished
semantics and do not leak to published CDC surfaces

savepoint rollback leaves zero native or custom invariant residue

custom invariant panics are captured as typed failures and never crash the
runtime

replay and durable recovery preserve invariant artifacts and publication-blocked
outcomes exactly

Required verification output

invariant_artifact_digest

custom_invariant_registry_digest

invariant_decision_log_digest

structural_legality_counter_snapshot

custom_panic_capture_report

publication_boundary_rejection_matrix

Pass condition

Structural legality is enforced canonically, custom invariants remain inside
the same authority contract as native invariants, and invariant-bearing
histories replay and recover without semantic drift.

11. Schema evolution CDC contract test
Purpose

Prove that schema transition boundaries are first-class canonical artifacts and
that CDC/subscriber continuation consumes those boundary artifacts rather than
rediscovering compatibility from raw schema state.

Scenario

Run deterministic workloads containing:

explicit schema transition commits

harmless additive surface growth

subscriber-visible but still bridgeable boundaries

contract-upgrade boundaries

renegotiation-required boundaries

rejected incompatible boundaries

checkpoint/resume before and after schema boundaries

replay and durable recovery over schema-transition-bearing histories

Must verify

schema boundary truth is preserved in canonical commit artifacts

subscriber continuation outcome is determined by persisted continuation descriptors

harmless boundaries continue without host choreography

visible bridges remain semantically correct even if boundary metadata is ignored

upgrade-only boundaries require declared subscriber support

renegotiation-required boundaries fail explicitly instead of drifting silently

replay and recovery reproduce identical schema boundary and continuation outcomes

Required verification output

schema_transition_digest

schema_boundary_cdc_digest

subscriber_contract_matrix

transition_decision_digest

descriptor_version_digest

Pass condition

Schema-bearing CDC continuity is exact, explicit, and replay/recovery-stable.

12. Schema reconciliation classification test
Purpose

Prove that schema divergence is classified deterministically, reconciled by
explicit preservation policy, and emitted as lineage-bearing canonical truth.

Scenario

Run reconciliation workloads containing:

additive divergence

narrowing divergence with no policy

narrowing divergence with preservation policies

type-incompatible conflicts

structural-incompatible conflicts

canonicalized ordering of schema pairs

direction-sensitive cases where direction is explicit input

replay and durable recovery over reconciliation-bearing histories

Must verify

reconciliation classification is deterministic

generic compatibility summaries never govern runtime behavior

resulting schema identity and lineage are explicit

lossy reconciliation is annotated explicitly when permitted

incompatible transitions fail closed with structured diagnostics

replay and recovery preserve reconciliation artifacts exactly

Required verification output

schema_reconciliation_digest

schema_lineage_digest

reconciliation_policy_matrix

schema_conflict_localization_report

descriptor_version_digest

Pass condition

Schema reconciliation is deterministic, policy-driven, lineage-aware, and
truth-grade.

2 CAD-specific ultimate tests
CAD 1. Topology identity survival test
Purpose

Prove that topological entities survive replacement/split/rebuild workflows with truthful lineage, exact relation updates, and stable historical resolution.

Scenario

Execute a hostile CAD editing workload with:

edge split

face split

shell-local rebuild

boolean-like replacement

deletion and recreation of adjacent topology

branch divergence on the same region

snapshot restore before and after structural edits

The relational runtime must track:

entity identity

relation identity

topology adjacency

lineage transitions

branch-local history

Must verify

topological adjacency is correct in visible truth

replaced entities resolve historically to successors when appropriate

relation identities change canonically and queryably

branch-local topology histories remain isolated

restore does not fabricate derivational lineage

deleted-retained topology remains historically inspectable while pinned

Required verification output

topology truth snapshot bundle:

entities

adjacency relations

face/edge/vertex incidence

lineage ancestry graphs for selected topological entities

relation-history report for selected edge/face relations

branch-local topology parity matrix

restore-vs-recompute semantic distinction report

Pass condition

Topology evolution is historically queryable, branch-aware, and structurally correct.

CAD 2. Missing-twin / nonmanifold corruption localization test
Purpose

Prove that when topology corruption occurs, the relational runtime localizes the exact commit, identity evolution chain, and relation history that caused it.

Scenario

Inject hostile topology mutations that produce:

missing twin

broken face loop

nonmanifold edge

invalid shell promotion state

branch-local corruption that must not leak to main

Run with commit-boundary and snapshot-audit invariants enabled.

Must verify

corruption is caught at the declared invariant boundary

failed commit publishes nothing authoritative

diagnostics identify:

offending entity/relation IDs

invariant class

candidate causal commit

relevant lineage chain

relevant relation history

branch-local corruption does not leak across branches

replay reproduces the failure exactly

Required verification output

invariant violation report with entity/relation focus set

candidate-causal-commit artifact bundle

lineage chain for corrupted topology

relation-history chain for twin/loop adjacency

branch leakage report

failure replay equivalence digest

Pass condition

Topology corruption becomes a precisely localized truth-runtime event, not a vague downstream symptom.

2 chip-simulator-specific ultimate tests
Chip 1. Netlist rewiring identity and history test
Purpose

Prove that rewiring, cell replacement, bus decomposition, and hierarchy edits preserve authoritative identity/history semantics across branches and snapshots.

Scenario

Run a hostile chip-design workload with:

gate replacement

net split

bus expansion/contraction

module hierarchy rewiring

branch-local alternate implementations

snapshot restore mid-edit

correspondence candidates across branch-local rewrites

Must verify

net and relation identities remain distinct and queryable

historical resolution answers what a signal/net became

hierarchy relations update canonically

branch-local rewiring histories remain isolated

correspondence stays advisory until explicit promotion

replay and CDC faithfully describe connectivity changes

Required verification output

connectivity truth snapshot bundle

hierarchical relation graph digest

selected net/cell lineage ancestry graphs

correspondence candidate/promotion report

CDC/connectivity parity report

branch-local connectivity isolation matrix

Pass condition

Connectivity evolution remains historically trustworthy and branch-aware under aggressive rewiring.

Chip 2. Snapshot-stable concurrent read vs hot rewrite test
Purpose

Prove that long-running analyses over snapshot-pinned chip truth remain exact while hot rewrites continue, without index authority leaks or visibility drift.

Scenario

Simulate:

active branch doing rapid rewrites of hot nets/modules

concurrent snapshot-pinned readers doing:

fanout analysis

relation-type scans

hierarchy traversal

timing-like bulk reads

index lag and rebuilds during active mutation

checkpoint/resume over CDC consumers

Must verify

pinned readers see perfectly stable connectivity truth

active branch rewrites do not leak into pinned snapshot reads

index lag does not change truth semantics

CDC consumers can resume deterministically

canonical observability preserved despite high churn

Required verification output

snapshot-reader truth digests at multiple times

active-branch evolving truth digests

index-assisted vs fallback query parity matrix

CDC resume/checkpoint matrix

observable-order digest across churn

retention/reclaim report for hot records

Pass condition

Hot rewrite pressure cannot break snapshot stability, fallback authority, or deterministic stream semantics.

What these tests collectively prove
Generic 10 prove

truth authority is serialized and deterministic

snapshots are real

replay is real

rollback is exact

CDC is canonical

lineage is semantic

merge-readiness is real

indexes are non-authoritative

bulk queries are first-class

durability preserves truth, not memory layout

CAD-specific 2 prove

topology evolution is historically correct

topology corruption is localizable and non-ambiguous

Chip-specific 2 prove

connectivity evolution is historically correct

concurrent analysis over hot rewrites remains snapshot-safe and authority-safe

The meta-rule for all 14 tests

Every one of these should emit canonical machine-checkable artifacts, not “logs looked good.”

At minimum, each certification run should produce canonical digests for:

visible truth

branch heads

patch artifacts

diagnostics summaries

lineage graph

replay stream

query surfaces

retained-vs-reconstructed historical surfaces where applicable

Without that, the suite can still be good — but it will not be truth-grade.
Milestone 5 named certification suites

- Schema evolution CDC contract test
  Required machine-checkable outputs:
  - schema_transition_digest
  - schema_boundary_cdc_digest
  - subscriber_contract_matrix
  - transition_decision_digest
- Schema reconciliation classification test
  Required machine-checkable outputs:
  - schema_reconciliation_digest
  - reconciliation_policy_matrix
  - schema_conflict_localization_report
  - reconciliation_replay_digest
- Diff/CDC truth parity test
  Required machine-checkable outputs:
  - diff_digest
  - cdc_digest
  - cdc_diagnostics_digest
  - continuation_counter_snapshot
- Hostile commit/replay equivalence test
  Required machine-checkable outputs:
  - truth_digest
  - patch_digest
  - lineage_digest
  - replay_digest
  - diagnostics_digest
  - branch_heads_digest
  - query_surface_digest
- Durable recovery and schema mismatch test
  Required machine-checkable outputs:
  - recovery_schema_bundle_digest
  - recovery_compatibility_diagnostic_digest
  - mismatch_failure_digest
- Invariant extensibility and structural legality certification test
  Required machine-checkable outputs:
  - invariant_artifact_digest
  - custom_invariant_registry_digest
  - invariant_decision_log_digest
  - structural_legality_counter_snapshot
  - custom_panic_capture_report
  - publication_boundary_rejection_matrix

## Milestone 9.17.1: Supply Chain Branch-Local MVCC Certification

The governing architecture is
[Query Milestone 9.17.1](../WORTH-query/milestone-9.17.1.md), and the
authoritative named suite is
[Owner Component Basis And Relational Branch-Local MVCC Certification](../WORTH-query/test-requirements.md#milestone-9171-required-suite).
This section records the Relational-owned test obligation. It does not create a
second specification or permit a Relational-only authority dialect.

### World-first prerequisite

Before branch-local MVCC implementation can claim phase closure, Relational
must ship the deterministic Supply Chain certification world with:

- immutable semantic definitions for ports, terminals, berths, vessels,
  voyages, calls, cargo, inspections, and their relation contracts;
- Court, Standard, and Scale profiles with identical meaning and increasing
  density;
- empty, operating, contested-planning, retention-pressure, and schema-version
  baselines;
- named Storm Reroute, Atlas Maintenance, Medical Hold, Southpoint Expansion,
  Competing Arrival, Atlas Retirement, Port-Call Rewire, and Hazard V2 deltas;
- a compiler that installs a fresh runtime only through public schema and
  transaction facades and binds semantic names only to owner-issued handles;
- a baseline audit that distinguishes fixture failure from runtime failure;
- a pure semantic oracle that does not use production queries, MVCC roots,
  indexes, encoders, digests, branch-head lookup, visibility, or history
  classifiers; and
- separate public observation and comparison paths plus replayable profile,
  seed, delta trace, and pause schedule for every failure.

The existing Fintech and generic worlds continue as preservation evidence, not
9.17.1's independent semantic oracle.

### Mandatory certification groups

The `relational_certification` integration target owns these cohesive groups:

1. **World causality** — public installation, owner-issued handle completeness,
   baseline/oracle agreement, profile parity, and typed fixture failures.
2. **Reference and ancestry** — immutable commit versus mutable reference,
   exact fork basis, local version zero, metadata generation, foreign/equal-
   ordinal substitution, and one canonical shared ancestor.
3. **Semantic isolation** — repeatable admitted-basis reads, read-your-writes,
   no sibling crossover, three-way fan-out, and per-step pure-oracle comparison.
4. **Independent progress and publication** — paused Storm versus progressing
   Maintenance, one-winner Competing Arrival, atomic complete-root visibility,
   exact stale outcome, and zero losing residue.
5. **Structural sharing** — zero-copy fork, touched-region copy-on-write,
   unchanged-region reuse, logical-versus-physical byte accounting, no shared
   mutable fate, and unique-only reclamation.
6. **Retention and lifecycle** — independent head/observation/transaction/
   candidate/external obligations, archive/delete posture, sibling-safe
   reclamation, readmission after boundary weakening, and exact terminal
   release.
7. **Cancellation and budgets** — every named pre-effect seam, deferred
   cancellation inside the bounded critical section, performed outcome after
   linearization, and typed exhaustion before effects.
8. **Model sequences** — seeded and shrinkable fork/delta/observe/retain/
   archive/delete traces checked after every step against semantic branch state
   and ancestry.
9. **Cost slopes** — Court/Standard/Scale fixture separation; 1/64/4,096
   branches; retained-history, footprint, and immutable-holder axes; zero
   unrelated-branch wait/contact; and no total-world copy slope.

The later Relational merge program must reuse this world, compiler, baselines,
deltas, oracle, observation adapter, comparator, and sharing evidence for
disjoint adoption, same-field conflict, delete-versus-update, endpoint rewiring,
schema reconciliation, and common-ancestor selection. 9.17.1 prepares those
honest inputs but does not claim merge behavior.
