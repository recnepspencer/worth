# Milestone 9.17: Composite Runtime Branching And Branch-Local MVCC

## Governing Role

Milestone 9.17 is the governing umbrella for the ordinary composite product-
branch foundation. It is implemented through five authority-aligned
submilestones:

1. [Milestone 9.17.1](./milestone-9.17.1.md) first establishes the causal
   Supply Chain certification world and independent semantic oracle, then exact
   owner-issued component bases and real Relational branch-local MVCC with
   branch isolation and structural sharing.
2. [Milestone 9.17.1.1](./milestone-9.17.1.1.md) corrects the owner-port
   concurrency, performed-settlement recovery, exact Signal retention,
   facade, evidence-lane, and documentation defects discovered after 9.17.1
   closure without reopening its historical phase record. Closed on
   2026-08-29.
3. [Milestone 9.17.1.2](./milestone-9.17.1.2.md) completes the Relational bundle
   and Signal operation services, including independent branch progress. It is
   the required entry gate to 9.17.2.
4. [Milestone 9.17.2](./milestone-9.17.2.md) establishes in-memory composite
   runtime-world history and coordinated publication in the dedicated
   `worth-runtime-world` composition owner.
5. [Milestone 9.17.3](./milestone-9.17.3.md) carries that authority through
   Query, gates existing-outbox eligibility on performed composite publication,
   cuts the public facade over, and certifies the product boundary.

The dependency order is strict:

```text
9.17.1 owner component bases and Relational MVCC
    -> 9.17.1.1 owner-port concurrency and lifecycle closure
        -> 9.17.1.2 final owner services and Signal independent progress
            -> 9.17.2 composite history and coordinated publication
                -> 9.17.3 Query carriage, facade, and certification
                    -> 9.17.4 application API hardening (post-closure)
                        -> 9.18 tree-based semantic undo and redo
```

This partition is not five interpretations of one feature. The product steps
own distinct authority boundaries; corrective 9.17.1.1 and 9.17.1.2 close the
owner-facing service, concurrency, and lifecycle contracts without adding a
new semantic owner. Each reaches a useful, independently reviewable completion
state. Milestone 9.17 closes only when all five close and the cumulative courtroom passes
through the real public Query composition root.

## Goal

Establish one ordinary product branch as a Runtime World owner-issued reference
to an exact composite Relational-plus-Signal world commit while preserving distinct
branch, basis, version, history, and publication authority inside Relational
and Signal.

A product branch may select one Relational branch basis and a different Signal
branch basis. Two product branches may share one immutable Signal basis while
their Relational histories diverge, or may later fork and advance Signal
independently. Writers whose selected Relational branches and product heads are
independent progress independently. Writers that compete for the same owner or
composite head receive one typed committed, stale, conflict, cancelled, or
indeterminate outcome.

The resulting single-parent composite history is the sole ordinary foundation
consumed by [Milestone 9.18](./milestone-9.18.md). Semantic merge, rebase,
multi-parent publication, offline synchronization, Store-native replication,
and distributed cross-region recovery remain outside this milestone.

## Roadmap Placement

Milestone 9.16 establishes the typed, authenticated, authorized ordinary Query
front door. Milestone 9.16.1 makes branch affinity mandatory through planning,
provider sessions, read sets, proposals, invariants, commits, receipts, and
publication, while intentionally retaining conservative single-product-world
and globally coordinated Relational mechanics. Milestone 9.16.2 establishes
stable package identity, reconstruction, and release carriage. It does not
persist application state or create a physical runtime boundary.

Milestone 9.17 replaces those implementation limits without weakening the
front door:

```text
owner-issued Relational branch basis ----\
                                         +--> admitted composite runtime basis
owner-issued Signal branch basis --------/             |
                                                       v
                                          composite world commit
                                                       |
                                                       v
                                          product branch reference
                                                       |
                       +-------------------------------+------------------+
                       |                                                  |
                       v                                                  v
          Relational branch-local work                       Signal-local work
                       |                                                  |
                       +------------ owner-issued results ----------------+
                                                       |
                                                       v
                              Runtime World coordinated compare-and-publish
                                                       |
                                                       v
                                     new composite commit and branch head
                                                       |
                                                       v
                               Query-carried public product-branch workflow
```

Multi-parent history, merge, rebase, Store-native replication, and offline
generalizations remain in the cross-runtime roadmap. Application state remains
memory-resident throughout 9.17; Worth Store integration later adds durability,
recovery, and physical residency under the completed semantic owners.

## Current Boundary

- Relational owns authoritative graph truth, branch and commit identity,
  snapshots, transaction conflict meaning, history, and retention, but its
  current transaction entry and commit coordination remain globally mutable.
- Signal owns branch-local derived execution state, exact snapshot and restore
  posture, definition-bound evaluation, and branch lifecycle. Its existing
  branch basis is materially stronger than a raw branch id or numeric version.
- Runtime Bridge owns cross-runtime correspondence and protocol admission. It
  does not and, because Relational already depends on it, cannot legally absorb
  the concrete cross-owner composition graph. The dedicated Runtime World owner
  does not yet exist.
- Query carries branch affinity, but some current surfaces derive product truth
  identity from Relational branch identity and therefore cannot represent an
  independently selected Signal basis honestly.
- `worth-foundational` already owns portable branch, commit, parentage,
  correspondence, canonical-basis, locator, and boundary vocabulary. 9.17.1
  extends that descriptive language with the shared immutable-target/mutable-
  reference observation grammar required by both owners. Those values describe
  cross-boundary meaning but grant no runtime authority.
- `worth-proof` already owns phase progression, checked outcomes, assumption
  basis, freshness downgrade, readmission, performed-effect law, and fixed-
  shape composition. It is the substrate for compiler-visible progression,
  not a replacement runtime or history engine.

The missing product fact is:

> Which exact owner-issued Relational basis and exact owner-issued Signal basis
> constitute the current product world selected by this product branch?

Neither a Relational branch head nor a floating Signal branch identifier can
answer that question alone.

## Cumulative Adversarial Courtroom

Two product branches begin from one composite commit. They share one immutable
Signal basis and fork distinct Relational branches. A Relational writer on the
first branch blocks while holding every resource it may lawfully hold. The
second branch commits. A third operation advances Signal while retaining
Relational. A fourth changes both components.

At the same time:

- different component branches carry equal local version ordinals;
- two writers race one Relational branch head;
- two writers race one product branch head;
- one component preparation succeeds before another component rejects;
- a product head advances after preparation but before publication;
- a hostile caller swaps a component basis, correspondence, expected head,
  retention pin, receipt, or equal ordinal from a neighboring runtime or
  branch;
- a trust-boundary round trip weakens basis freshness and demands owner
  readmission;
- cancellation occurs before preparation, between component preparations,
  before composite publication, and after owner-local immutable work exists;
- failure occurs after owner-local outbox creation and before and after
  product-head comparison/publication; and
- Query history, inspection, and live paths attempt to fall back to
  Relational-only or ambient Signal selection.

The independent oracle must observe:

- unrelated Relational and product branches progress despite the blocked
  writer;
- same-branch races retain exact typed stale/conflict posture;
- immutable Signal basis reuse performs no graph copy, evaluation, or ambient
  `latest` lookup;
- only operation-named components advance;
- every successful ordinary composite commit has exactly one canonical parent;
- exactly one winning product-head movement occurs;
- no public or historical observation sees a half-current product world;
- no copied, stale, foreign, equal-ordinal, or representation-equal artifact
  substitutes for owner authority;
- owner work without product movement leaves either complete cleanup or typed,
  bounded `ProductUnpublishedOwnerEffects`; and
- the same exact product-world basis reaches Query planning, execution,
  publication, receipts, history, and live observation; and
- an existing Relational outbox fact is dispatchable only when its exact
  composite publication is performed.

A Relational-only product branch, one-to-one component branch assumption,
floating head lookup, global commit mutex, best-effort multi-runtime
publication, or Query-restamped authority must fail this courtroom.

## Product Decision Lock

1. Relational owns Relational branch identity, versions, snapshots,
   transactions, conflicts, authoritative commits, retention, and owner-local
   publication.
2. Signal owns Signal branch identity, exact bases, snapshots, restore,
   definition-bound execution state, derived lifecycle, and owner-local
   advancement.
3. Runtime Bridge owns installed semantic correspondence. The dedicated
   Runtime World owner admits exact component composition, owns immutable
   composite commits, product references and generations, and coordinates
   publication. It owns composition currentness, not component truth.
4. Query owns branch workflow declaration, fresh admission, public progression,
   DX, and outcome projection. It cannot mint a component basis,
   correspondence, composite commit, or product-head movement.
5. Product, Relational, and Signal branch identities are distinct meanings and
   types. Equal text, digests, or version ordinals grant no correspondence.
6. A composite basis names exact owner-issued component bases. No component is
   selected by ambient branch state or a `latest` lookup. The first root and
   product reference require explicit exact-basis bootstrap; construction does
   not infer an initial head.
7. A composite world commit is immutable composition truth with one ordinary
   parent, exact component bases, component-change posture, correspondence,
   and publication-attempt provenance. The performed-publication envelope
   separately binds the commit to the reference movement that selected it.
8. A product branch is a mutable compare-and-publish reference to one composite
   commit. Component heads do not independently define product currentness.
9. Component reuse is exact immutable-basis reuse. Component mutation requires
   owner-issued advancement or fork authority.
10. Unchanged components remain at the exact carried basis and are never
    opportunistically refreshed.
11. Owner-local results remain component evidence until Runtime World
    compatibility admission and product-head compare-and-publish succeed.
12. Failed or losing work moves no product head. If any owner effect occurred,
    bounded `ProductUnpublishedOwnerEffects` records the exact recovery
    obligation; pre-effect denial is exact no-effect.
13. Independent branches are never serialized by an ordinary global commit
    lock. Shared mechanical storage cannot erase logical ownership.
14. Relational forks reuse exact immutable ancestry without duplicating truth;
    branch-local writes copy only touched persistent regions, and stable
    inspection distinguishes logical branch bytes from unique physical bytes.
15. Retention pins exact composite commits and every exact component basis
    needed by live branches, snapshots, transactions, corrections,
    publication, or recovery.
16. `worth-foundational` vocabulary remains descriptive and canonical. A
    canonical digest, locator, branch descriptor, or correspondence value
    cannot become operational admission authority.
17. `worth-proof` supplies the concrete authority-witness carrier plus
    progression, basis, freshness, readmission, structural-fact, and performed-
    effect law. Each owner declares a sealed marker and privately issues the
    specialized Proof carrier after live checks. Governed facades require that
    exact specialization or a stronger owner artifact; they never accept a
    caller-selected generic authority marker.
18. Every operational phase transition consumes the exact private-minted
    predecessor. Illegal ordering, raw candidate promotion, stale proof reuse,
    and weaker-type substitution are compiler-rejected where public API shape
    permits and otherwise fail typed before effects.
19. Derived Signal caches, Query projections, diagnostics, and certification
    artifacts have zero component or composite currentness authority.
20. Merge, rebase, multi-parent publication, tags, offline synchronization,
    Store-native replication, durability, and distributed recovery remain
    governed elsewhere.
21. Relational and Signal own component state; the Runtime World composition
    owner owns composite history/currentness; the base Runtime Bridge owns
    installed semantic correspondence; Query owns fresh dispatch admission.
    No physical adapter participates in this milestone.
22. Query's existing outbox payload remains in the Relational component commit,
    but only performed Bridge composite publication can make it product-
    dispatchable after 9.17.3.

## Authority-Aligned Milestone Partition

| Submilestone | Sole governing outcome | Primary owner | Explicit exclusion |
| --- | --- | --- | --- |
| [9.17.1](./milestone-9.17.1.md) | Causal merge-ready Relational certification world, exact component basis contracts, and structurally shared concurrent Relational branch-local MVCC | Relational and Signal, each for its own basis; Relational certification for the world/oracle | No merge behavior, composite product branch, persistence, or public Query workflow |
| [9.17.1.1](./milestone-9.17.1.1.md) | Independently borrowable Relational owner services, pre-effect recoverable Relational settlement, exact non-current Signal retention, terminal lease lifecycle, and executable owner evidence | Relational and Signal, each for its own component contract and lifecycle | No composite history, Query carriage, persistence, or reinterpretation of historical 9.17.1 phases |
| [9.17.1.2](./milestone-9.17.1.2.md) | Complete Relational service bundle plus concrete Signal services and independent branch execution | Relational and Signal | No composite history, product currentness, Query carriage, or changed owner meaning |
| [9.17.2](./milestone-9.17.2.md) | Memory-resident immutable composite history, product branch references, and coordinated compare-and-publish | `worth-runtime-world` composition owner in the Runtime Bridge authority band | No Query-owned history, persistence, or public facade cutover |
| [9.17.3](./milestone-9.17.3.md) | End-to-end Query carriage, live-runtime existing-outbox composite gating, public branch facade, lifecycle docs, and hostile certification | Query as audience/admission facade and certification owner | No new component, composition, or physical-storage authority |

No submilestone may claim the next submilestone's product. In particular,
9.17.1, 9.17.1.1, and 9.17.1.2 do not ship a product branch; 9.17.2 does not
claim public Query completion; and 9.17.3 does not repair missing owner-local
MVCC, owner services, or composition history with facade glue.

## Governing Destination Topology

```text
worth-relational/
    branch/
    history/
    mvcc/

worth-signal/
    branch/
        owner_services/

worth-runtime-bridge/
    correspondence/

worth-runtime-world/
    basis/
    identity/
    history/
    branch/
    publication/
    retention/
    recovery/

worth-query-execution/
    basis/
        runtime_world.rs
        product_branch.rs

worth-query-decl/ and worth-query-host/
    branch workflow facade over admitted product branches

worth-query-certification/
    runtime_world_branching/
```

The submilestone specs define the populated files and ownership within these
stable axes. Forbidden destinations include a Query-local branch registry, a
Relational field containing an ambient Signal head, a generic
`branch_manager`, a Proof-owned runtime workflow engine, a Foundational-owned
operational branch handle, or a second composite-history implementation in the
later merge roadmap. A temporary physical-runtime composition, owner-local
persistence hook, and adapter-minted performed publication are also forbidden.

## Cumulative Performance Contract

- Relational commit coordination scales with contention and state touched on
  the selected Relational branch, not total branches or unrelated writers.
- Composite publication scales with the fixed component count plus the
  components changed by the operation; it never scans all runtimes, branches,
  or history.
- Product branch creation is O(component bindings), O(1) for the ordinary
  Relational-plus-Signal composition, plus declared retention work.
- Exact Signal basis reuse performs zero graph copy, evaluation, cache
  duplication, or hidden owner advancement.
- Single-component operations perform exact-zero preparation in unchanged
  components beyond carried-basis validation required for publication.
- Ancestry traversal, historical comparison, retention reclamation, and orphan
  cleanup remain explicit history or maintenance lanes.
- Component publication and composite publication expose exact owner contacts,
  comparison attempts, owner-local candidates, product-unpublished owner
  effects, and cleanup work; unrelated branches contribute zero synchronous
  semantic coordination.
- Pending dispatch lookup is indexed and joins one outbox locator to one exact
  performed composite publication; it does not scan histories.
- Receipts expose exact owner preparation, component basis validation,
  composite comparison, same-branch retry, unrelated-branch wait,
  product-unpublished-owner-effects, cleanup, and Query carriage counters.

## Must Preserve

- Milestone 9.16.1's single graph-obligation and provider-session progression;
- Milestone 9.16's authentication, authorization, disclosure, invariant,
  compare-and-commit, recovery, aftermath, and publication contracts;
- Milestone 9.16.2's package identity, reconstruction, archive, and host release
  boundary, while keeping branch authority outside archive bytes and release
  metadata;
- Relational and Signal component authority;
- base Bridge correspondence authority and Runtime World composition authority
  without component-truth absorption;
- Query's inability to mint lower authority;
- Foundational canonical and boundary vocabulary without authority promotion;
- Proof phase, freshness, readmission, performed-effect, and structural-fact
  law beneath owner-specific types; and
- certification-only replay.

## Explicit Non-Goals

- application undo, redo, inverse, or compensation semantics;
- semantic diff, merge, or rebase policy;
- multi-parent commits, tags, or best-common-ancestor logic;
- offline replicas and synchronization;
- persistence, restart recovery, and distributed cross-region publication; and
- treating derived Signal output or Query materialization as component truth.

## Allowed Debt

- Store-native graph persistence, recovery, replication, distributed atomic
  publication, and cross-region recovery remain explicit Store/cross-runtime
  handoffs.
- Semantic merge, rebase, multi-parent history, and offline synchronization
  remain explicit cross-runtime roadmap work.
- No runtime-backed global coordinator, floating component head, Relational-
  only product branch, ambient Signal selection, half-publication lane, raw
  authority constructor, or callable compatibility path may remain debt.

## Parallelization And Store Dependency

The three submilestones are sequential because each downstream owner consumes
the upstream authority contract. Within a submilestone, inventories,
documentation scaffolds, independent oracle construction, and non-overlapping
owner-local work may proceed in parallel only after their shared binding and
phase contracts freeze. Facade cutover and cumulative certification close last.

The 9.17 sequence is not blocked on `worth-store` because it makes no persistence
claim. Store-native graph execution, durability, recovery, replication,
distributed atomic publication, and joined Store-provider certification remain
explicit Store or cross-runtime work.

## Documentation Deliverables

- this umbrella remains the single governing decision record and links every
  submilestone;
- each submilestone owns its current boundary, courtroom, decision lock,
  destination topology, phase plan, proof obligations, and handoff;
- 9.17.1.1 corrects the owner-facing Relational and Signal guides and evidence
  commands;
- 9.17.1.2 documents the complete Relational bundle and Signal owner-service
  lifecycle, per-branch boundary, and executable independent progress;
- public Query branch documentation lands only in 9.17.3 after the real facade
  exists;
- `worth-foundational` and `worth-proof` docs are linked where their shared
  vocabulary or progression law is used, without teaching them as operational
  owners; and
- successor documents identify 9.17.3 as the final branching-umbrella
  prerequisite and 9.17.4 as subsequent application API hardening, while
  retaining 9.17 as the branching semantic umbrella.

## Umbrella Acceptance

Milestone 9.17 closes only when:

- 9.17.1 proves a causal production-backed Supply Chain world against an
  independent oracle, exact component bases, no branch crossover, shared immutable ancestry
  without copied truth, and independent Relational branch progress without a
  global ordinary commit coordinator;
- 9.17.1.1 proves that Relational preparation, fork, publication, and settlement
  are independently borrowable; performed settlement survives capability loss;
  non-current exact Signal bases remain retainable; lease drop is terminal;
  and the facade, docs, feature lane, scheduled proofs, and contention court
  execute the same contract;
- 9.17.1.2 proves complete concrete owner bundles, exactly one same-basis Signal
  movement, unrelated progress without a whole-runtime mutex, and lifecycle-
  total closure, cancellation, panic, and capacity denial;
- 9.17.2 proves exact composite correspondence, immutable single-parent
  history, product branch references, retention, product-head comparison, and
  no-half-publication compare-and-publish;
- 9.17.3 proves the complete authority reaches every Query boundary through
  the public facade, gates existing-outbox dispatch on performed composite
  publication, and deletes the Relational-only/ambient-Signal lane;
- the cumulative courtroom runs through real owner facades and the real Query
  composition root with an independent component/composite-history oracle;
- public invalid phase order and raw/weak authority substitution are
  mechanically denied using compiler-visible owner-specific types;
- default and admitted parallel lanes converge on identical composite truth;
- documentation, dependency enforcement, facade inventory, residue scans,
  structural counters, and runtime evidence agree; and
- no submilestone closeout claims the umbrella before all downstream
  integration evidence exists.

The decisive product observation is the product branch head and exact
owner-observed component bases. A green Relational commit, Bridge unit test,
Runtime World unit test, or Query facade compile alone is not umbrella closure.

## Handoff

[Milestone 9.17.4](./milestone-9.17.4.md) follows the certified 9.17.3 foundation
as application API hardening; it does not reopen this umbrella's historical
certification. [Milestone 9.18](./milestone-9.18.md) begins after that consumer
cutover and the completed 9.17 umbrella. It consumes exact product branches, immutable
single-parent composite commits, component bases, ancestry, retention, and
coordinated compare-and-publish authority and performed-publication-gated
aftermath to define tree-based semantic undo and
redo as freshly admitted composite history. It may not create another
composition owner, treat Relational history as the whole product world, or
weaken independent component and branch progress.
