# Milestone 9.18.1: Structural Graphs And Application Runtime Integration

> **Status:** Phases 1–4 accepted on 2026-10-26; Phases 5–6 accepted on 2026-10-27.
> Phases 7 and 11 accepted on 2026-10-26; Phases 8 and 10 accepted on 2026-10-27.
> Phase 9 remains unfinished.
> The 2026-10-27 Proprietary findings add Phases 12–18. Phases 12 and 15 are
> transferred, not completed, to 9.19's managed-index and final CAD qualification.
> Phases 13–14 and 19 are complete. Phases 16–18 remain unfinished.
> Addendum 1 defines correction-aware consecutive source undo.
> Written on 2026-10-24 against master `b56db81af6`.
> **Dependencies:** 9.17.6.3 (partitioners and exact invalidation) is complete.
> 9.18 does not block this milestone. Every phase lists its own dependencies, and
> phases with no dependency on each other may run in parallel.
> The added application-runtime phases consume the completed composite-world,
> request, inbound-completion and history contracts; they do not block the
> original graph phases or reopen their accepted evidence.

## Why This Milestone Exists

The following graph boundary records the original discovery against
`b56db81af6`. Phases 1–6 retain their existing meaning and numbering.

Relational already evaluates acyclicity, symmetry, uniqueness, minimum
connectivity and partition isolation at commit. It already walks the graph with
`BoundedStructuralTraversal`. `worth-execution` already maintains connected
components incrementally. Applications reach none of it:

- `lower_relation_integrity`
  (`worth-query-execution/src/domain_computation/primary_graph/schema_layout/registry_lowering.rs`)
  lowers only endpoint kinds, cardinality and deletion.
- The invariant reader
  (`primary_graph/invariant_projection/traversal.rs`) reads one hop:
  `relations_from` and `relations_to`. Query does walk the graph internally, but
  only to widen an invariant's scope
  (`installation_state/substrate_compilation/invariant/execution.rs`). It never
  offers that walk as a read.
- The Components partitioner (`crates/worth-execution/src/partition/components.rs`)
  has no Query plan shape. `plans/deferred-work.md`, under "Components (islands)
  plan shape in Query", records that gap.
- Managed derived views can be read by hosts but not by the rules that decide a
  commit.
- An application has no stable identity for a cycle or an island. The
  Components partitioner names an island by its least member, so the name
  changes when that member is removed.

Consumers therefore hand-write what the platform owns:
- the public `topology_entry` fixture walks rings by chaining exactly-one
  successors (`planar_invariant.rs`, `planar_topology.rs`);
- WORTH Proprietary's CAD entry (`crates/worth-cad-entry/src/basis/thread.rs`)
  detects feature-history cycles by hand and keeps its own read counter
  (`LiveQueryCost`).

Each of these is a second owner of a meaning the platform already has. Persistent
naming of a loop, a shell or any other derived aggregate needs a platform owner
before a consumer can name anything.

This milestone gives applications that machinery through Query's declaration and
read surfaces. It adds no domain types. A face, a loop and a shell stay
application meaning. The platform supplies laws, walks, orbits, components,
derived relations and the identity of aggregates.

### Additional Runtime Discovery — 2026-10-26

A read-only audit against `100dcd1eee` found two application integrations that
remain on the older Query surface, a native history capability worth carrying
through, one incorrect identity conversion, and two small cleanup opportunities.
All six findings are now required scope. The two cleanup findings share one
batch in Phase 11; this does not authorize a general legacy-code purge.

| Finding | Existing boundary | Required destination |
| --- | --- | --- |
| Exact async source binding | World derives `TruthCommitIdentity` from the admitted immutable native reference; populated forks preserve their actual commit despite a reset branch version. Empty targets refuse before Signal admission | Phase 7 courts cover native equality, sibling publication, reserved occurrence gaps and cleanup; the Decl/Host resource endpoint remains Phase 8 |
| Managed asynchronous resources stop short of the application SDK | Signal/Bridge own the lifecycle; Execution owns the legacy occurrence-custody integration, with directional older engine/UI adapters | Phase 8 supplies one declared, installed, governed lifecycle through Decl/Host |
| Causal inspection stops short of ordinary application receipts | Older `worth-query/src/runtime/inspection/causal/` already anchors observations, admits inspection and consumes Bridge evidence | Phase 9 connects actual application receipts and fresh disclosure to that machinery |
| Bounded record/aspect history reaches the application SDK | Relational retains finite canonical pages; World follows exact raw product ancestry and component evidence | Phase 10 captures disclosed exact generations through installed History meaning and continues at pinned truth with fresh SubjectRead and independent metadata decisions |
| Relational identity planning inventories retired | The three string-only Phase 1 catalogs and their module/export declarations have been removed | Phase 11 preserves actual identity authority, selection/receipt issuance and real compile-boundary tests |
| Bridge uses one facade import | The three hidden wildcard alias modules have been removed | Phase 11 retains the standalone `worth_runtime_bridge::facade` and its substantive control, replay and certification capabilities |

This extension completes existing runtime capabilities at their application
boundary. It does not add a second async engine, causal evidence store, history
authority or application-owned retry machine. Native aspect history is an audit
projection, not an inverse/pre-image source or correction permission. Phase 10
does not replace 9.18's composite operation/correction history.

## Placement

- **Consumes.**
  - 9.17.6.3: partitioners, incremental maintenance, invalidation and canonical
    reduction.
  - 9.17.4: ordinary application entry and declaration.
  - Relational's native contract evaluators and traversal.
- **Before 9.19.**
  - 9.19 and 9.20 consume this milestone's walk step, orbit reads and component
    reads. 9.20's typed path programs gain a closure step kind from Phase 3; they
    do not build a second traversal.
  - 9.21's decision summaries stay separate from Phase 6's derived relations.
    Summaries are attached evidence; derived relations are graph projections.
- **With 9.18.** Correction publishes ordinary new commits, so aggregate
  identity and lineage follow correction with no special path. Reversing an edit
  that split an orbit is a merge with ordinary lineage. Restoring the earlier
  identity is a correspondence binding that the application installs, through
  9.18's correspondence contract.
- **Runtime integration follow-through.** Phase 8 carries the implemented 9.4
  async/resource lifecycle into the current two-facade application SDK. Phase 9
  carries existing causal inspection, not 9.21's new decision attachments or
  incremental summaries. Phase 10 adds retained record/aspect audit queries,
  preserving 9.18's correction ownership and fresh disclosure. Durable archives,
  restart-stable async custody and Store-backed historical reconstruction remain
  successor contracts; unavailable evidence must be reported explicitly.
- **Proprietary.** The CAD kernel's Extrude needs Phases 1 to 4 first, then
  Phase 5 for shells. Phase 6 is needed by tessellation and by derived adjacency.
  Proprietary's `cad/docs/plans/platform-gaps.md` lists these gaps and is updated
  as each phase lands.

## Decisions That Hold In Every Phase

1. **One seam per meaning.**
   - Laws lower through `lower_relation_integrity` only.
   - Walks lower onto Relational's traversal only.
   - Components lower onto `worth-execution`'s partitioner only.
   - Derived relations extend the existing `application_query/derived_view/`
     owner only.

   No phase adds a second evaluator, traversal, partitioner or view engine.
2. **Domain-neutral vocabulary.** Names describe graphs: acyclic, symmetric,
   orbit, component, walk, lineage. The general managed-index extension belongs
   to 9.19; spatial algorithms remain provider-owned. CAD topology, exact-contact
   policy and other application domain types stay outside Query and Relational.
3. **No raw handles.** Applications never receive Relational, Bridge or Signal
   handles, as the capability index already requires. Each new capability is a
   typed Query surface.
4. **Commit-boundary truth.**
   - Laws and rules judge the proposed commit as a whole, never an intermediate
     state.
   - A read inside a rule sees the proposed state, including derived state.
5. **Loud, typed refusals.**
   - A violation names the relation, the law and the entities involved, as typed
     identities and not as strings.
   - An interrupted or incomplete evaluation refuses. It never passes, and it
     never returns a truncated answer as if it were complete.
6. **Footprints are complete.** Every entity and edge a rule reads through a
   walk, orbit, component or derived read enters that rule's read footprint, so
   the rule is re-evaluated whenever any of them changes. A rule never goes
   stale.
7. **Deterministic and replayable.**
   - Orders are canonical, by entity identity.
   - Identities and lineage depend only on committed history, never on worker
     count, evaluation order or cache state.
   - Cert-only replay reproduces them exactly.
8. **Performance and bounded observation.**
   The original graph-read policy remains unchanged for Phases 1–6:
   - Equivalent-read comparisons, growth tests and timing are welcome.
   - Exact read counts are observations, not fixed performance expectations
     (MENTALITY section 7).
   - No signature carries cost.
   - No phase adds a work budget, a ceiling or an exhaustion matrix.
   - A walk ends when its frontier is exhausted, or on cancellation.
   Phases 7–11 preserve original request controls and existing resource owners.
   Async queues and retained inspection/history pages have explicit finite
   admission and pagination limits. A history page bounds source visits as well
   as returned entries, so an empty-match query cannot scan an entire branch in
   one call. These limits do not introduce fixed read-cost promises, a second
   work-accounting system or extra meters for the original graph reads.
   Phases 12–18 likewise reuse existing request/resource owners. Their growth
   courts qualify actual locality and storage behavior without adding fixed
   read-count promises or production cost signatures.
9. **Consumers delete their workarounds in the same phase.** A phase is not
   done while a public fixture or the Proprietary CAD entry still carries the
   hand-written version of what that phase supplies.
   For the added capabilities, a real two-facade application fixture is required;
   a domain feature need not be invented in Proprietary merely to exercise it.
   Migrated orchestration and its displaced executable path are cut over in the
   same integration phase, not deferred to Phase 11's unrelated small cleanup.
   Phases 16–17 explicitly require the real Proprietary consumer evidence named
   below. The transferred index/locality finding is adopted once at the end of
   9.19; a public fixture does not replace its final private gate.

Code and test files stay within 400 lines. Planned syntax is marked
`rust,ignore` until it compiles; each phase turns its DX into a compiled
example.

---

## Phase 1: One Read-Count And Receipt Surface

**Depends on:** nothing.

**Accepted.** The common actual-read surface, compiled planar differential,
public facade and guide, and Proprietary SDK adoption are verified.

### Requirements

- The invariant reader, the operation (decision) reader and declared-query
  receipts report their reads through one documented public type. The invariant
  side already counts its reads in `WorthQueryInvariantProjectionWork`
  (`invariant_projection/work.rs`). It becomes reachable and documented through
  `worth-query-host`'s facade, and is listed in
  `tools/boundary-check/snapshots/facades.toml`.
- `WorthQueryApplicationInvariantProjectionReader` is documented as a public
  read surface:
  - its methods;
  - what a rule may read;
  - proposed versus committed reads;
  - its denials.
- Declared-query receipts and invariant read counts use the same names for the
  same kind of read: adjacency lists, edges inspected, field reads, index
  candidates. One read kind has one counter name.
- Every read added by Phases 3 to 6 reports into this surface. No phase adds its
  own counter type.
- Counts are observations for tests and diagnostics. Production code never
  asserts them and never branches on them.

### Tests That Prove It

- A compiled example: an invariant reads relations and fields, and the test reads
  the counts the platform recorded for that evaluation.
- An equivalent-read comparison on `topology_entry`'s planar invariant. It
  observes the current one-hop ring walk, so later phases can compare their
  reads without fixing an incidental exact count.
- A facade snapshot check fails if the reader or the count type drops out of the
  host facade.
- A differential: a declared query and an invariant that read the same edges
  report the same adjacency and edge counts.

### How We Know It's Done

- The reader and the count type are in the host facade, the snapshot and the
  feature guide `foundations/` (or `modeling/schema-validation.md`), with the
  compiled example.
- WORTH Proprietary's `worth-cad-entry` deletes `LiveQueryCost` and its count
  tests read the platform's counts.

### Gaps To Watch

- **Counts turning into contracts.** A doc or API that says "this read costs
  exactly N" and a runtime check that enforces it. Tests compare equivalent reads;
  production may not.
- **Counting a model instead of the reads.** A counter must be incremented where
  the read happens, not computed from what the code expects to read.
- **Two names for one read kind.** Two counters for the same read kind recreate
  the duplicate the phase deletes.
- **The existing projection work budget.** Leave it as it is unless a court in
  this milestone hits it. Never extend it to the new reads.

### Target DX

```rust,ignore
fn check(reader: &mut WorthQueryApplicationInvariantProjectionReader<'_, Model>)
    -> Result<(), Denial> { /* ordinary reads */ }

// In a test:
let evaluation = harness.evaluate_invariant(&branch, Rule::reference())?;
assert_eq!(evaluation.reads().adjacency_edges_inspected(), 6);
```

---

## Phase 2: Relation Laws As Declarations

**Depends on:** nothing. Runs in parallel with Phase 1.

### Requirements

- `ApplicationRelationIntegrity`
  (`worth-query-declaration/src/application_schema/relation_integrity/`) gains
  declared laws, each lowered by `lower_relation_integrity` to the Relational
  contract that already evaluates it:

  | Law | Relational contract |
  | --- | --- |
  | `acyclic` (source to target) | `AcyclicityContractDeclaration` |
  | `symmetric` | `SymmetryContractDeclaration` |
  | `unique` within a declared scope | `UniquenessContractDeclaration` |
  | `reaches_at_least(n, kinds)` | `ConnectivityMinimumContractDeclaration` |
  | `same_partition_endpoints` | `PartitionIsolationContractDeclaration` |

- Laws are part of the relation's canonical meaning. Adding, removing or changing
  one changes schema and program identity, as endpoint policy already does
  (`canonical_identity_version_tests.rs`).
- Laws apply at the commit boundary on every application branch, whoever wrote
  the commit: a handler, a workflow, a correction or an MCP-driven agent.
- A violation returns a typed application denial carrying:
  - the relation;
  - the law;
  - the entity identities from Relational's violation fields (for acyclicity,
    the edge whose endpoints close the cycle).
- Evaluation stays scoped to the edges the commit touched, as the acyclicity
  evaluator already scopes to planned edges.

### Tests That Prove It

- **Per law, a hostile commit and its lawful twin.**
  - The hostile commit violates the law and refuses with the typed denial naming
    the entities.
  - The twin differs by one edge, reaches the same check, and commits.
- **Acyclicity:**
  - a cycle closed across two commits;
  - a cycle closed inside one commit by two new edges;
  - a self-edge;
  - a cycle 1,000 edges long. Detection must not depend on any depth limit.
- **Symmetry:** deleting one side of a pair refuses. Deleting both in one commit
  commits.
- **Seeded differential.** Random edit sequences over a small graph are checked
  against an independent in-test graph model that computes each law's verdict. It
  must not call production evaluators.
- **Identity.** Changing a law changes schema identity, and an installed program
  that depended on the old meaning is refused as incompatible.
- **Mutants that must die:**
  - drop the lowering of each law;
  - make an interrupted evaluation return no violation;
  - evaluate against the committed state instead of the proposed one.

### How We Know It's Done

- All five laws are declarable, lowered and certified in the public
  `application_graph` certification target.
- `topology_entry` declares the laws it currently checks by hand, and its hand
  checks are deleted.
- WORTH Proprietary declares `PreviousFeature` acyclic and deletes the hand cycle
  detection in `basis/thread.rs`.
- `modeling/schema-validation.md` documents every law with a compiled example.

### Gaps To Watch

- **Silent pass on interruption.** `evaluate_acyclicity_contract` returns `None`
  when `context.checkpoint(1)` fails. The lowered path must turn an interrupted
  evaluation into a refusal. Prove it with the interruption mutant above.
- **Depth caps.** Custom-rule traversal caps depth at 32
  (`validation/custom_rule/traversal.rs`). Acyclicity must use its own
  reachability search and stay complete on long histories.
- **String-only violations.** Relational's `detail` string is not the denial. The
  denial is typed and names identities.
- **Minimum connectivity's boundary.** `ConnectivityMinimumEnforcement` names
  only `SnapshotPublication`. Confirm that this boundary is the application
  commit, or add the commit boundary in Relational. It must never be checked
  later than publication.
- **Symmetry's pairing.** Declare the pairing on the relation. Do not add a
  registry of inverse relations next to the schema.
- **A second validator.** No application-side law checker. The CAD side deletes
  its own.

### Target DX

```rust,ignore
const PREVIOUS_FEATURE: ApplicationRelationIntegrity =
    ApplicationRelationIntegrity::same_context_no_self_edges_unbounded_retain_dangling()
        .acyclic();

const FIN_RADIAL: ApplicationRelationIntegrity =
    ApplicationRelationIntegrity::same_context_no_self_edges_unbounded_retain_dangling()
        .symmetric();

match outcome {
    MutationOutcome::Denied(denial) => match denial.relation_law() {
        Some(RelationLawDenial::Cycle { relation, closing_edge }) => { /* typed */ }
        _ => {}
    },
    _ => {}
}
```

---

## Phase 3: Walks And Reachability

**Depends on:** Phase 1 (reads report through its surface).

### Requirements

- The invariant reader and the operation reader gain:
  - `walk(relation, direction, from)`: every entity reachable from `from`, each
    with its distance, in canonical breadth-first order;
  - `reaches(relation, from, to)`;
  - `closure(relation, direction, roots)` over a root set.
- These reads repeat one endorelation (the same entity kind at both ends).
  Roots are included once at distance zero; other visits use minimum distance
  from any root. Canonical order is `(distance, native entity identity)`.
  Forward follows stored source to target, reverse follows target to source,
  and reachability is reflexive for a valid local entity. `RelationDirection`
  is shared portable vocabulary, without read or mutation authority.
- Declared queries gain a closure step on `ApplicationQueryRootPath`
  (`forward_closure`, `reverse_closure`). The receipt records the step and its
  reads.
- Invariant views lower to complete, kind-filtered native
  `walk_outgoing_from` / `walk_incoming_from` reads. Their native scope-widening
  relative retains its own bounded policy. Snapshot/operation and declared
  reads lower into the native pinned-query executor, preserving its visibility,
  request, packet-memory and reduction authority. Query never implements a
  competing loop of one-hop reads.
- **No constant depth cap.**
  - A walk ends when its frontier is exhausted, or on cancellation.
  - Cancellation refuses with a typed denial; it never returns a partial walk.
  - Relational's 32-step cap serves scope widening. It is not a product limit,
    and it does not apply to these reads.
- Cycles are safe. Every entity appears once.
- Walks honor branch visibility and disclosure exactly as one-hop reads do.
- **The footprint is complete.** Every edge and entity a walk visits enters the
  rule's read footprint, so a change anywhere along the walked path re-evaluates
  the rule.
  - Include physical self, back and diamond edges that discover no new entity.
  - Current native applicability uses declared-kind closure; Query's frozen
    entity/relation sets enforce prepared read authority. This phase does not
    introduce a retained per-rule verdict cache or a domain root-selection
    policy. Preparation must identify affected roots honestly.

### Tests That Prove It

- **Seeded differential against an independent breadth-first model** on random
  graphs with cycles, self-edges, diamonds and disconnected parts. It compares
  order, distances and membership.
- **Footprint court.**
  - An invariant that walks three hops must refuse a commit that changes only
    the third-hop edge into a violating state.
    Require actual fresh preparation/evaluation, the exact typed custom
    violation and unchanged committed truth, with a completing lawful twin.
  - Mandatory mutants register only first-hop entities or first-hop relations
    at the real prepared-admission handoff. Each must fail that exact violation
    expectation; a generic access/execution refusal is not a passing court.
    These expose incomplete prepared authority, not a fictitious cached verdict.
    Separate root-only activation from deep-edge changes when touched endpoints
    or root discovery would otherwise mask missing entity admission.
- **Long walks.** A walk over 1,000 steps of history succeeds and matches the
  model.
- **Growth.** With the walked subgraph held fixed, 10 and 10,000 unrelated
  entities give the same walk reads, using Phase 1's counts.
- **Cancellation.** A walk cancelled at a safe point refuses with a typed
  denial, and its twin completes.
- **Foreign identities.** An identity from another branch is refused, as
  `relations_from` refuses it today.

### How We Know It's Done

- Walks, reachability and closure exist on both readers and as a declared-query
  step, with compiled examples in the feature guide.
- `topology_entry`'s rules that chain hops by hand use walks. Its rings move to
  orbits in Phase 4.
- WORTH Proprietary's `live()` in `basis/thread.rs` reads history through
  `walk`.

### Gaps To Watch

- **Stale rules.** An incomplete footprint is the most dangerous failure in this
  milestone: the rule passes on old data and nothing reports it. The footprint
  mutant is mandatory.
- **Truncation reported as complete.** No walk may return early without a typed
  refusal.
- **Order drift.** Order must not depend on hash, storage or worker order.
- **Host N+1 loops.** One walk is one traversal, not a loop of one-hop reads in
  Query.
- **Growing into a path language.** Typed correlated paths belong to 9.20, which
  consumes this phase's closure step as one step kind.

### Target DX

```rust,ignore
// Installed invariant views use the resolver-issued endorelation binding.
// PreviousFeature points from each feature to its previous feature.
let history = view.walk(&previous, RelationDirection::Forward, &tip)?;
for step in &history {
    let feature = step.entity();
    let distance = step.distance();
}
let depends = view.reaches(&previous, &later, &earlier)?;

// Declared query: today's typed path builder, with one new step kind.
let ancestry = ApplicationQueryRootPath::from(Feature::reference())
    .forward_closure(PreviousFeature::reference());
```

---

## Phase 4: Orbit Relations And Persistent Aggregate Identity

**Depends on:**
- Phase 2, because the orbit law lowers through the same seam;
- Phase 3, because orbit reads use the same traversal.

**Accepted, 2026-10-26.** Stored orbits expose complete ordered reads, tracked
decisions and declared queries through one Native aggregate identity/lineage
owner. The independent permutation, anchor-removal, split/merge, replay/fork,
write-guard and currentness courts qualify that owner. `topology_entry` consumes
orbit windows instead of its former hand ring walk. The
[feature guide](../../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md)
links compiled projection, handler, query and privileged lineage examples.
Proprietary's [CAD adoption contract](https://github.com/recnepspencer/worth-proprietary/blob/main/cad/docs/plans/query-9.18.1-adoption.md)
records genuine sheet edits, holes, split/merge lineage and current-main
qualification at `33278669079835d5f0f5c96556697784fa449eb9`, retaining SDK `b9cdf607`.
The derived-orbit substrate and its private consumer gates are qualified under
accepted Phase 6. Governed and tracked lineage reads remain separate unavailable
capabilities.

### Requirements

- **The orbit law.** A relation declared `orbit` is a permutation of its kind:
  every member has exactly one successor and exactly one predecessor, and every
  successor is of the same kind.
  - It lowers to exactly-one cardinality on both sides, plus same-kind
    endpoints.
  - A commit that leaves a member without a successor, or with two, refuses and
    names the member.
- **Orbit reads**, on both readers and as a declared-query shape:
  - `orbit(relation, member)`: the members in successor order, starting from the
    orbit's anchor;
  - `orbit_of(relation, member)`: the orbit's identity;
  - the orbit's length.
- **Aggregate identity.** This phase introduces one owner for the identity of a
  derived aggregate. Phase 5 reuses it.
  - An identity is minted when the aggregate first appears, deterministically
    from the commit occurrence and the aggregate's least member at that moment.
  - It persists through every edit that keeps the aggregate whole, including
    removal of the member it was minted from.
  - **Split:** the part holding the least surviving member keeps the identity.
    Every other part gets a new identity with lineage `split_from(old)`.
  - **Merge:** the result keeps the identity of the participant with the least
    current anchor. The others get lineage `merged_into(survivor)`.
  - **Storage:** identities and lineage are platform-owned committed records,
    written by Query in the same commit as the edit that caused them. Handlers
    can read them and can never write them.
- **Lineage reads:**
  - the lineage of an identity;
  - the current identity that a retired identity's lineage leads to.
- **Composing orbits.** An orbit relation may itself be a derived relation (Phase
  6), so a composed permutation becomes an orbit without new platform vocabulary.
  The accepted Phase 6 substrate binds derived permutation sources and is
  qualified through the private consumer gates.

### Tests That Prove It

- **Seeded differential against an independent permutation model.** It does
  cycle decomposition plus the identity rules above. Random successor rewiring
  sequences are compared for orbit membership, order, identities and lineage.
- **Anchor removal.** Removing the member an identity was minted from keeps the
  identity. The 9.17.6.3 least-member rule would rename it, and this test must
  catch that.
- **Split and merge.** Each yields exactly the lineage the rules predict. Edits
  to unrelated orbits leave their identities untouched.
- **Law.** An open chain, a member with two successors, and a successor of
  another kind each refuse and name the member. Their lawful twins commit.
- **Replay and branches.**
  - Cert-only replay of the same commits reproduces every identity and lineage
    record.
  - Two branches forked from one basis keep shared identities. They diverge only
    where their edits split or merge.
- **Footprint.** A rule that reads an orbit is re-evaluated when any successor
  edge in that orbit changes.
- **Mutants that must die:**
  - name orbits by least member;
  - let a handler write an identity record;
  - mint from evaluation order instead of the commit occurrence.

### How We Know It's Done

- `topology_entry`'s rings use orbit reads, and its hand ring walk in
  `planar_invariant.rs` is deleted.
- WORTH Proprietary's cell complex reads face loops as orbits, and names loops
  by orbit identity and lineage.
- The feature guide documents the orbit law, orbit reads, identity and lineage
  rules, with compiled examples.

### Gaps To Watch

- **Inheriting least-member naming.** That rule is deterministic but not
  persistent. The aggregate identity owner must not reduce to it.
- **Order-dependent minting.** If identity depends on which worker or which edge
  is processed first, replay and branches diverge.
- **Application-written identities.** If handlers can write them, a second owner
  of naming appears.
- **Correction semantics.** Reversal under 9.18 publishes new commits. Lineage
  must follow ordinary rules, and no correction path may rewrite old lineage.
- **Rings stored as ordered lists.** Order is the successor relation. No
  sequence field or position index sits beside it.

### Target DX

```rust,ignore
const NEXT_FIN: ApplicationRelationIntegrity =
    ApplicationRelationIntegrity::same_context_no_self_edges_unbounded_retain_dangling()
        .orbit();

let ring = reader.orbit(NextFin::reference(), &fin)?;
let loop_identity = ring.identity();          // persistent across edits
for member in ring.members() { /* successor order from the anchor */ }

// In an admitted handler, the operation inspection reader additionally
// requires NextFin's OperationReads contract. This is privileged inspection:
// it records no lineage decision dependency or governed history disclosure.
let history = reader.reader().lineage(NextFin::reference(), loop_identity)?;
let current = history.latest_identity(); // merge continuation, or None
for event in history.events() {
    let _incident = (event.occurrence(), event.kind(), event.from(), event.to());
}
```

The compiled handler and its candidate/currentness proof live in
[`invariant_orbits/decision/handler.rs`](../../workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/tests/application_attempt/invariant_orbits/decision/handler.rs).
`DecisionReader::reader()` marks opaque context, so this inspection cannot
produce a completed decision reuse proof. Scoped/governed history disclosure
and tracked lineage decisions remain separate unfinished capabilities.

---

## Phase 5: Components With Persistent Identity

**Accepted, 2026-10-27.** The component identity, lineage and maintenance courts
and the complete current-main CAD qualification recorded under Phase 6 accept
this phase. The SDK remains `5b01312ed0048ad4591e6b4a50e08685ea6d77ba`.

**Depends on:** Phase 4 (aggregate identity and lineage).

### Requirements

- An application declares a component set over one or more relations, read as
  undirected coupling, for example components of a declared adjacency relation.
- Query installs the component-set root and component-read plan shape:
  - it lowers onto `ComponentPartitioner` and its incremental maintenance
    (9.17.6.3 Slice 6.8);
  - it gives each component the Phase 4 aggregate identity, not the
    partitioner's least-member name.
- **Reads**, on both readers and as a declared-query shape:
  - `component_of(set, member)`;
  - the members of a component;
  - the components of a set;
  - the component count.
- Merges, splits and anchor removal publish lineage through the Phase 4 owner.
  Unaffected components are never re-read.
- Rules may state component counts, for example "this set has exactly one
  component". That goes through ordinary invariants; no new law is added.

### Tests That Prove It

- **Seeded differential against an independent union-find model**, comparing
  membership, counts, identities and lineage across random edit sequences.
- **The deferred item's court.** `topology_entry`'s differential is extended with
  island merges, splits and least-member removal, with identity persisting where
  Phase 4's rules say it does.
- **Locality.** A split re-reads only the affected component. Phase 1's counts
  are unchanged for unrelated components at 10 and 10,000 components.
- **Replay and parallel runs.** Cert-only replay and runs at different worker
  counts produce identical identities and lineage.
- **Footprint.** A rule that reads a component is re-evaluated when any coupling
  edge in it changes.

### How We Know It's Done

- The deferred-work entry is removed because it has shipped.
- `topology_entry` uses component reads.
- WORTH Proprietary reads shells as components of face adjacency, with persistent
  shell identity.
- The feature guide documents component sets with compiled examples.

The public [topology island consumer](../../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md#public-topology-island-consumer)
now uses complete component reads plus genuine ordered orbits. Its existing
grouped topology target includes independent seeded union-find comparison across
ordinary coupling edits and least-member replacement, and original business-read
locality beside 10 and 10,000 unrelated rings. The Native component courts also
audit actual cert-only replay after split/merge and compare complete published
membership, identities, incident lineage and historical-name resolution across
one, two and eight workers. Parallel runs require actual overlapping invariant
readers and staged-parallel metadata; they do not claim parallel partition
maintenance. See [component replay and worker determinism](../../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md#component-replay-and-worker-determinism).
**Accepted, 2026-10-27.** Installed CAD reads retain genuine Native shell
components over derived face-use adjacency. The eleven focused Entry courts
cover independent topology oracles, actual split/merge and least-member removal,
whole-cohort context transfer, versioned ingress and tessellation consumption,
typed side-anchor refusal, retained decisions and cache invalidation. Production
reclamation reuses the original face slot with a higher generation while the
original historical fields and retained component remain valid; a stale current
seed refuses. The Components deferred-work entry is removed.

### Gaps To Watch

- **The partitioner's name leaking.** Least-member names may serve internally as
  routing keys. They never surface as identity.
- **Splits on large components.** Recomputation cost grows with the component.
  That is acceptable and tested for growth; do not add a budget.
- **Parallel nondeterminism.** Partitions processed in parallel must reduce
  canonically before identities are assigned.
- **Visibility.** A component never spans entities the reader cannot see. A
  hidden member must not change an identity the reader observes, or the identity
  leaks disclosure.

### Target DX

This is a component set of stored faces. CAD shells require the projected
face-use member domain specified below; these face islands are not shells.

```rust,ignore
schema.components::<FaceIslands>(&[FaceAdjacency::reference()]);

let island = reader.component_of(FaceIslands::reference(), &face)?;
let island_identity = island.identity();
let faces = island.members();
let count = reader.component_count(FaceIslands::reference())?;
```

---

## Phase 6: Derived Relations Readable By Rules

**Accepted, 2026-10-27.** The public logical-source substrate is qualified by 35
runtime courts and a Host compile-pass/compile-fail pair. Genuine Native sources
feed proposed/committed rule reads, declared decisions, ordinary committed
projection and framework-owned managed counts. The evidence covers derived
laws, walks, orbits, complete component/census reads and Native identity/lineage;
exact audience/posture/currentness, original-request refusal and retained
material custody; cache invalidation/discard/rebuild; and the real grant-authority
adversary. Legacy bytes, durable recovery, independent seeded membership/heir
oracles, cert-only replay and actual one/two/eight-worker execution are qualified.
The 10/10,000 courts compare actual maintenance and business-read acquisition
category totals. Equal totals do not claim fixed work or an acquisition-identity
roster. The public production-maintenance prerequisite is separately qualified
at `5b01312ed004` by ten Native courts, one Query owner court and a Host example.
The same SDK pin is adopted by the eleven focused CAD Entry courts. Actual
region/exterior incidence supplies derived shell adjacency and retained decisions;
malformed both/neither anchors refuse without publication. A retained guarded
decision becomes stale before an unchanged physical-only twin commits in the
same world, and a fresh guarded decision then commits. Cold/warm, unrelated-body
and source-transition cache controls preserve their distinct causes. Combined
public and private evidence accepts Phases 5–6; no derived value grants authority.

The complete private qualification integrates incoming main `52b0fd370ee428f1`
in merge `70aa8334bd8eb558bcbe589507152049b9805728`, retaining SDK
`5b01312ed0048ad4591e6b4a50e08685ea6d77ba`. Its
[CAD adoption contract](https://github.com/recnepspencer/worth-proprietary/blob/70aa8334bd8eb558bcbe589507152049b9805728/cad/docs/plans/query-9.18.1-adoption.md)
records all 356 Entry tests passing with one existing ignored test, all 3,480
workspace CI tests passing with 78 skipped, 62 cost tests passing, 213 doc tests
passing, and an all-target check. The CI and cost lanes exclude only Entry, whose
complete required lane ran separately. Oracle, determinism, repository Python,
policy, dependency and line-cap gates also pass. The earlier Entry and CI failures
remain recorded alongside their reviewed corrections and successful full retries.
This qualification used Rust 1.98.0 (`88d9e12ae178fab0fb5cc050a94da85685d449ea`);
it does not close Phase 18’s required Rust 1.99.x qualification.

**Depends on:** Phase 3 (walk reads), and the existing derived-view owner.

### Requirements

- A managed derived relation or collection (`ApplicationDerivedCollection` and
  `application_query/derived_view/`) may be declared readable by invariants and
  decisions.
- **Proposed state.** Inside a rule, a derived read sees the derived state of the
  proposed commit. It never sees the last committed derived state.
- Derived state is maintained incrementally from its declared dependencies, using
  9.17.6.3 invalidation. It can always be rebuilt from source.
- **Derived values are evidence, never authority.** A derived read cannot satisfy
  a capability, approval or authority check.
- **Composition.** Derived relations can be the target of Phase 2 laws, Phase 3
  walks, Phase 4 orbits and Phase 5 component sets. A composed permutation is
  then a derived relation declared `orbit`.
- No second view engine. Phase 6 extends the existing derived-view owner.

### Tests That Prove It

- **Proposed-state court.**
  - A rule over a derived relation must refuse a commit whose violation exists
    only in that commit's derived state.
  - Mutant: read the committed view. It must die.
- **Seeded differential.** Incremental maintenance matches a full rebuild after
  random edit sequences, eviction and rebuild.
- **Authority.** A hostile twin presents a derived value as authority and is
  denied. Its lawful twin uses the source fact.
- **Laws on derived relations.** A derived relation declared `orbit` and one
  declared `acyclic` each refuse a violating source edit, naming source entities.
- **Growth.** Derived maintenance reads scale with the source delta, using Phase
  1's counts.

### How We Know It's Done

- Rules read derived relations with compiled examples in the feature guide.
- One public fixture composes a derived orbit.
- WORTH Proprietary reads face adjacency as a derived relation, not a
  handler-maintained stored one, and its tessellation plan consumes this phase.

### Gaps To Watch

- **Stale derived reads.** Reading the committed view inside a rule is a silent
  stale check. The proposed-state mutant is mandatory.
- **Derived becoming authority.** If a derived value can unlock an action, a
  cache becomes a permit.
- **Cycles between derived relations.** A derived relation that depends on itself
  through another is refused at installation, never at run time.
- **Overlap with 9.21.** Decision summaries are separate and stay separate.

### Target DX

```rust,ignore
schema.derived_relation::<FaceAdjacency>(
    ApplicationDerivedRelation::compose(FaceLoop::reference(), LoopFin::reference())
        .then(FinRadial::reference()),
);

let neighbours = reader.derived_relations_from(FaceAdjacency::reference(), &face)?;
```

---

## CAD Shell Member Domain

This contract governs the CAD shell adoption accepted with Phases 5–6. The
qualified public substrate supports finite carrier/case members through the same
Native evaluator and aggregate coordinator, including actual logical component,
lineage and committed-projection reads. Generic projected membership and public
fixtures are insufficient evidence of CAD shell adoption.

The finite projected-domain meaning prerequisite provides typed carrier/case
declaration, canonical portable readmission and Archive tag44/v1 transport.
Existing entity-domain member bytes and absent-domain schema identities remain
unchanged; older Frame18 readers reject tag44 as an unsupported variant. Actual
schema binding now resolves the registered Native domain/case owner; only a
genuine selected view can issue member observations. See [declaration status and focused courts](../../workspaces/worth-query/crates/worth-query-decl/README.md#projected-member-domain-meaning).
The installed CAD courts supply the consumer evidence below; public meaning and
logical-source qualification alone do not establish CAD adoption.

### Actual Consumer And Decisive Proof

CAD's accepted cell-complex contract derives exactly two face uses per face:
`(face, Negative)` and `(face, Positive)`, even when both sides have the same
region. A shell is a maximal component of one region's boundary face uses joined
through shared boundary vertices; shared-edge sectors are included in that
connectivity. The exterior is selected by the actual side-to-body incidence,
not a fabricated Region entity. Region-side incidence is source truth.

A face-only or shared-edge-only implementation must fail these actual installed
CAD journeys before acceptance:

- A two-sided sheet has one exterior shell containing both uses of each face.
  Self-edge exclusion compares complete face-use keys, never carrier faces.
- Two boundary patches touching at a shared vertex join the appropriate shell
  without inventing a shared edge. Disjoint patches remain separate.
- A shared face between distinct regions contributes its two uses to the
  distinct region boundaries. A route through another region cannot join them.
  Interior sides in the same body that touch a shared vertex remain separate
  when their regions differ; an unconditional face-to-body fallback must fail.
- Actual source edits cut and join components, remove the least carrier, and
  delete/recreate a carrier in a reused physical slot. Membership, persistent
  names, retirement and split/merge continuations agree with independent
  topology observations; a stale generation cannot select the new use.
- A whole component transfers to another region with its use keys unchanged.
  Its opaque Native identity survives without fabricated split/merge events;
  current region changes, its old region-qualified address refuses current
  resolution, and an authorized pinned historical read still observes the old
  region. Reuse must observe the source edit even if adjacency endpoints match.
- An absent stored-region side changes to a real region and back to its actual
  exterior body. Proposed/committed rules, retained decisions and cached managed
  results consume the changed positive and negative source facts. Endpoint
  deduplication or Boolean recipe composition must not lose those dependencies.
  Invalid both-anchor and neither-anchor edits retain the actual `SideAnchors`
  refusal and publish no partial shell state.

The public Native/Query courts additionally cover eviction and rebuild, exact
fork/replay identity and lineage, and real differing-worker executions for this
member domain. Read-only logical members cannot enter an entity writer, supply
a mutation target or satisfy an authority/capability check. Negative cases need
lawful actual-source counterparts. Reuse existing grouped targets; no new
one-case Cargo targets, budget knobs or proof journals are required.

### Product And Authority Decisions

1. A declared projected member domain is a finite product of one stored carrier
   entity kind and named cases. Its logical key contains the complete carrier
   identity, including generation, and the canonical semantic case identity.
   Case identity is independent of declaration/enumeration order. The domain
   has no independently writable entity kind, allocated virtual EntityId or
   RelationId. A declaration, copied key, digest or current least member grants
   no observation or mutation authority.
2. Native issues read-only logical member handles against the actual selected
   schema/domain, carrier generation, immutable root and observation posture.
   Query resolves the corresponding exact application binding. Preparation
   admits the complete physical leaf closure and retains actual positive and
   absence facts; evaluation consumes that frozen proof. Foreign domains,
   roots, generations or narrower access audiences refuse before use. A
   serialized selector needs fresh admission and cannot restore an old proof.
3. Extend the existing derived-relation recipe/evaluator with carrier projection,
   case-restricted carrier projection, homogeneous union and intersection.
   These are closed typed meaning, not callbacks into application code. For
   shells, derive face-use-to-vertex incidence, stored-region incidence and
   explicit exterior-body incidence. CAD already stores `FaceNegative`,
   `FacePositive`, `FaceNegativeExterior` and `FacePositiveExterior`; exactly
   one stored-region or exterior anchor per side is enforced by its existing
   `SideAnchors` invariant. Region incidence unions the Negative case/carrier
   leg followed by `FaceNegative` with the Positive leg followed by
   `FacePositive`. Exterior incidence instead follows the corresponding
   explicit `FaceNegativeExterior`/`FacePositiveExterior` legs. It never falls
   back to a face's owning body merely because no Region was observed. Shell
   adjacency is shared-vertex incidence
   intersected with the union of shared-region and shared-exterior incidence,
   excluding equal complete logical keys. All branches retain their real
   supporting and empty reads, including both explicit side-anchor rosters.
   Replacing a stored-region side with an exterior side edits those actual
   incidences; it is not a complement or application-defined fallback operator.
   Native contains no CAD region, orientation or shell policy.
4. The existing Native aggregate coordinator, partition maintenance and naming
   policy own projected components. Complete logical member keys determine
   heirs and name minting; routing keys and hashes never become identity.
   Projected membership uses an explicit reserved Native metadata grammar for
   logical coordinates, not fake business entities or physical relations.
   Entity-domain records, meanings, canonical bytes and identities retain their
   existing encoding when the projected-domain extension is absent. New domain
   and recipe meaning participates in schema snapshots, digest, portability,
   recovery and exact readmission before any runtime rebinding.
5. Face-use identity does not contain region. A committed shell retains its
   Native component identity across a whole-cohort region transfer under the
   existing member-heir policy; region is separately observed current context.
   A pure candidate's `(region, least face use)` is only a `ShellLocator`.
   Committed CAD reads retain the real typed aggregate identity and current
   region. Old representative `ShellName` bytes are never interpreted as Native
   identities: current identity-bearing readmission refuses them as legacy
   locators and directs a fresh source read. No stored carrier reference is
   migrated through a guessed member or region.
6. Domain projection, source evaluation, maintenance and retained proof use the
   original request's work, memory and cancellation owners. Single-component
   work is bounded by affected incidence and component membership, including
   actual vertex-star fan-out and finite case multiplicity. Do not construct a
   body-wide face Cartesian product or scan unrelated components. Full domain
   census and reconstructive rebuild retain their explicit broader cost.

### Target Contracts And Destination

The declaration facade adds `ApplicationProjectedMemberDomainRef`,
`ApplicationProjectedMemberDomainDefinition` and typed
`ApplicationProjectedMemberCaseRef`. A domain reference cannot enter
`schema.entity` or a stored relation endpoint. The existing component-set meaning
gains an explicit projected-domain form; its typed `projected` constructor is
distinct from the physical-entity constructor. Native and Query read-only
projected identities remain distinct from physical entity identities. The common
component read family accepts the corresponding typed binding/member pair;
the actual Native aggregate owner remains singular.

Target usage, after the declared domain/cases and set have been installed:

```rust,ignore
let use_ = reader.projected_member(FaceUses::reference(), &face, NegativeSide::reference())?;
let shell = reader.component_of(Shells::reference(), &use_)?;
let identity = shell.identity();
let face_uses = shell.members();
```

Extend these existing authority axes; create populated responsibilities only:

```text
worth-query-declaration/src/application_schema/
  projected_member_domain/{reference,definition,validation}.rs       create
  component_set.rs; derived_relation/{expression,recipe,validation}.rs extend
  canonical_identity/member/projected_member_domain.rs              create
worth-query-package-archive/src/record/application_schema/member/
  projected_member_domain.rs                                       create
worth-relational/src/
  projected_member_domain/{definition,key,admission,projection}.rs   create
  derived_relation/{catalog,evaluation,proof,pinned}.rs              extend
  aggregate_identity/{transition,projection,materialization,pinned}.rs extend
worth-query-execution/src/domain_computation/primary_graph/
  schema_layout/projected_member_domain.rs                          create
  application_invariant/{typed_binding,prepared_scope,typed_view}/  extend
  invariant_projection/component/; application_mutation/            extend
worth-cad-entry/src/
  schema/shells.rs; basis/live/shells.rs                             create
worth-topology/src/
  names/uses.rs; cells/{shell,uses}.rs                               change
```

Pure declaration/archive own meaning only; Native owns live admission, actual
source evaluation and persistent metadata; Query carries their exact proofs
through installed rules, decisions and result/disclosure owners. CAD owns face
use, region and shell interpretation. The stable public facades remain Query
Decl/Host and Native's runtime facade; do not add application deep imports,
application callbacks, a second graph/view engine, writable FaceUse/Shell rows,
or a private CAD partitioner for committed shell reads. Pure topology validation
may retain candidate-local connectivity/locators and serves as an independent
oracle; it does not mint committed identity.

Implement in authority order within the existing phases: finite portable domain
meaning and exact compatibility; Native source/domain admission and recipe
evaluation; shared aggregate persistence and read handoffs; ordinary Query
rule/decision/disclosure consumption; actual CAD adoption and removal of its
representative-name publication path. Each reviewed slice is committed, but
a slice alone never accepts a phase: its complete original obligations govern
acceptance.

The continuing CAD contract is `cad/docs/plans/query-9.18.1-adoption.md` with the
cell-complex naming rules and general-modeling use-name contract updated in the
same adoption. The public capability index must distinguish currently shipped
entity-domain components from projected-member requirements. The existing
relation-law/invariant guides gain real compiled domain/case examples, failure
and source-affinity semantics, region transfer, compatibility and lifecycle
before phase acceptance; target snippets above are not substitutes for them.

---

## Runtime Integration Courtroom (Phases 7–11)

The plausible defective implementation re-exports older Query/Bridge types,
invents an observation receipt from display fields, calls native history and
truncates its returned vector, or records an async source using a convenient
counter. It passes a single-branch demonstration while losing provenance,
disclosure, bounded work and cleanup. The following real journeys must expose it
before the added phases can close.

- **Async source and lifecycle.** Start from an actual committed product, fork
  it, publish on a sibling and burn a prepared occurrence before starting an
  installed asynchronous resource. Its source commit must equal the native
  selected target, independently of branch-local version and allocator counters.
  Complete one request; timeout and retry another; deliver the older completion
  after supersession. Observe actual result state, provider contacts and native
  active requests. A duplicate cannot publish again, a stale completion cannot
  fulfill its successor, and refusal/cancellation retains the exact cleanup
  owner. Inspect and close through fresh authorized application requests.
- **Causal explanation.** An ordinary application source operation changes an
  input used by a real managed producer and declared query. Retain its actual
  application receipt, then explain the observation through Host. References
  must identify the actual source change, invalidation/evaluation and observation;
  an independent equal-output case must explain suppression without inventing a
  changed output. Substitute a sibling receipt, change the principal's disclosure,
  retire optional evidence and cancel materialization. Require original typed
  mismatch, denial, unavailable/omission or cancellation outcomes as applicable.
  No inspection may execute a business effect, re-run the source or manufacture
  missing evidence. Compare the source operation's own outcome and charged work
  with optional explanation off and on. Later inspection runs on a separately
  admitted fresh request and pays for its actual reads/materialization there.
- **Record/aspect history.** Through ordinary handlers, change one selected
  aspect, change another aspect, make a retained equal-value edit and delete the
  record; fork and edit a sibling independently. Query the exact selected product
  history through only Decl/Host. Compare emitted occurrences and aspect/structural
  facts with the independently recorded authored sequence. Page across both
  matching and nonmatching changes, including an empty result page with remaining
  ancestry, and one commit with more matching records than a page can hold.
  Revocation, foreign continuation, cancellation and retention expiry cannot
  disclose protected entries or turn an incomplete traversal into complete history.
  Ten and 10,000 unrelated retained changes exercise the same finite page bounds;
  total traversal may grow with ancestry, but one page must not eagerly scan or
  clone it all. Drop pages and continuations and observe release of their actual
  retention and allocation custody.
- **Cleanup.** Use source/export/caller inventory and surviving owner checks,
  rather than a fabricated runtime journey, to prove deletion of only the three
  Relational planning catalogs and three Bridge alias modules. Existing real
  identity issuance, foreign-selection refusal and standalone facade consumers
  must still compile and work.

Use existing grouped certification targets and shared honest fixtures. Old-facade
compile examples or lower-runtime unit tests alone cannot qualify a two-facade
application claim. No fixed test count, duplicate compiler portfolio or new proof
ledger is required. These are process-local runtime claims, not restart/durable
transport claims.

## Runtime Integration Destination

The tree below fixes responsibility and dependency direction, not empty modules
to create in advance. `E` means extend an existing owner, `N` a new populated
responsibility, `M` migrate the named existing responsibility, and `D` delete.
Query crate paths below are under `workspaces/worth-query/crates/`.

```text
crates/worth-relational/src/
  branch/{basis,target}.rs                                  E selected commit truth
  history/access/aspect_history_queries.rs                  E existing selectors
  history/access/aspect_history/                            N retained native paging
    {cursor,page,selection}.rs
  identity_authority/
    phase_one_family_map.rs                                 D stale catalog
    phase_one_compile_fail_targets.rs                       D stale catalog
    phase_one_root_break_targets.rs                         D stale catalog
    mod.rs                                                  E remove catalog exports
crates/worth-runtime-world/src/lifecycle/owner/
  owned_async_product.rs                                    E exact async source
crates/worth-runtime-bridge/src/
  conditional_execution/owned_async/                         E lifecycle authority
  diagnostics/causal_envelope/                               E evidence assembly
  facade.rs                                                 E remove alias modules
worth-query-declaration/src/
  application_resource/                                     N typed resource meaning, action/disclosure profiles
  application_schema/value_binding/canonical/               E complete descriptive codec reference
  application_query/
    history/                                                N target/aspect selection
    inspection/{definition,metadata,validation}.rs           N exact-binding inspection metadata meaning
    canonical_basis/inspection.rs                           N portable metadata identity
    disclosure_contract/inspection.rs                       N typed metadata rules
worth-query-installation/src/
  application_resource/                                     N actual typed resource/provider correspondence
  application_query/
    history/                                                N schema/disclosure binding
    inspection.rs                                           N validate exact compiled binding before issuer admission
worth-query-package-archive/src/record/
  application_schema/member/inspection.rs                   N additive member45/v1
  application_query/disclosure/inspection.rs                N additive selector8/v1

worth-query-admission/src/domain_computation/
  basis_lifecycle/                                          E original scoped inspection
worth-query-execution/src/domain_computation/
  evidence_identity/                                        M shared diagnostic identity protocol
  primary_graph/
    application_resource/                                   M live resource orchestration
    causal_inspection/                                      M operational inspection and application evidence binding
    application_query/history/                              N native page adapter
worth-query-publication/src/application_entry/
  resource/                                                 N governed retained requests
  inspection/                                               N receipt-rooted inspection
  query/                                                    E ordinary history query entry
worth-query/src/
  runtime/owned_async_{source,lifecycle}.rs                  M displaced orchestration
  runtime/inspection/causal/                                 E old-receipt translation and historical certification only
worth-query-{decl,host}/src/facade.rs                        E audience exports only
worth-query-certification/tests/application_graph/           E grouped public journeys
```

Declaration owns portable resource and history meaning; Installation owns their
schema/provider bindings; Admission owns fresh scope/disclosure; Execution owns
live retained resources and observation binding; Publication owns ordinary entry
and refusal-preserving progression. Relational owns canonical change selection;
World owns exact composite correspondence; Bridge owns causal translation; Signal
owns asynchronous scheduling and settlement. There is no peer-runtime dependency
on Query. Required portable descriptions and legality proofs stay in the existing
Foundational/Proof owners under the repository's type-placement rules.

The older `worth-query` crate depends on the split owners. Execution therefore
cannot import its inspector or async runtime. Migrate their actual reusable meaning
and orchestration down to the owners above; update existing internal callers and
remove the displaced implementation in the same phase. Keep a facade-only export
only where the existing supported audience requires the same contract. Do not
duplicate the implementation or introduce a reverse dependency to avoid this move.
The shared Query diagnostic identity protocol is a peer of the execution owners;
its scopes span workflows, mutations, inspection and consumer evidence. Diagnostic
encoding preserves scope and scheme but grants no inspection or execution authority.
Runtime observation/cursor/resource families enter as children of their named
owner; no flat integration bag or shared cross-runtime state machine is allowed.

---

## Phase 7: Exact Async Source Commit Identity

**Depends on:** the completed admitted Relational/Signal bases and Runtime World
publication owners. No dependency on unfinished graph phases. Must precede Phase 8.

**Accepted, 2026-10-26.** The existing World route derives its exact commit from
the admitted immutable Relational observation. The
[World/Bridge source court](../../crates/worth-runtime-world/tests/runtime_world_certification/owned_async.rs)
compares the admitted binding with the native populated-fork target after sibling
publication and a discarded reserved occurrence; empty sources refuse before
Signal admission and refused revalidation retains original cleanup custody.
The existing installed Query request-response courts preserve completion,
timeout/retry, replacement and exact-request retirement. This qualifies the
native source repair; the application resource endpoint remains Phase 8.

### Requirements

- Repair `validate_owned_async_product` to derive the actual selected commit
  from the already-admitted immutable Relational observation. A branch truth
  version, root identity, global allocator preview or display digest is never a
  commit ID. Preserve the existing World-owner and source-observation checks.
- An empty selected target has no authoritative source commit. Return a typed
  no-committed-source denial before Signal request admission; never synthesize
  commit zero or silently change authoritative admission to branch-head meaning.
- Admission, revalidation, completion and writeback consume that same exact
  source binding. This repair does not create a new identity allocator or grant
  authority to a descriptor supplied by a caller.

### Tests That Prove It

- At the actual World/Bridge owned-async entry, start a request on a populated
  fork whose branch version is zero. Compare its admitted commit with the native
  reference target; exercise sibling publication and reserved occurrence gaps.
- An empty target refuses before an active Signal request exists; its lawful
  committed twin starts. Preserve foreign owner/source mismatch and exact-request
  retirement courts. Observe writeback causality where that declared family uses it.
- The mutant that substitutes `truth_version().as_u64()` must fail the native
  commit-equality assertion, not merely fail earlier during setup.

### How We Know It's Done

The existing owned-async production route retains the exact source commit through
its lifecycle and cleanup, with no branch-counter conversion remaining. Phase 8
may rely on this binding; its application endpoint is not qualified by this local
repair alone.

---

## Phase 8: Managed Async Resources Through Decl And Host

**Accepted, 2026-10-27.** Typed resource declarations and installed-profile lookup preserve
the complete input/result/failure codec description, exact Access recipe,
provider role, finite capacities and independent Start/Inspect/Advance/Close
requirements. Member 39 and disclosure selector 7 extend the current protocol 18.
Portable value-shape text is descriptive; only actual typed inventory can resolve
its decoder. Boolean availability and lifecycle state do not authorize counts;
cleanup detail requires both Count and Explanation influence. The Host provider
and progression surface is implemented with its actual provider and cleanup journeys.

Qualification uses the actual World/Bridge custody courts, one closed compile-fail
case, five [Resource journeys](../../workspaces/worth-query/crates/worth-query-certification/tests/application_graph/source_only/capability/resources/journeys.rs),
the existing ordinary-query revocation control and the
[combined workflow](../../workspaces/worth-query/crates/worth-query-certification/examples/observable_workflow.rs).
The integrated tree repeats those six selected application cases and the combined
example; its populated fork, genuine prepare/drop, sibling publication and owned
recovery remain in that one real journey. The shared example declares the same
history meaning required by its actual handlers and program feature. These are
17 distinct focused qualifications, not a rerun of the lower async portfolio.

The installed-access prerequisite compiles explicit Resource/action/facet
obligations rather than reusing the Access query's payload plan or permission.
Close and detailed Close retain their own action; only the latter selects cleanup
disclosure. ResourceRead admission checks exact installed purpose/subject and
actual provider-support affinity before reserving capacity. The reservation
remains with its admitted plan until terminal/drop. The endpoint uses a distinct
Resource ProductRead intent, actual installed contract leaf, original basis and
independent current authorization Product, with its own proof/terminal. Query
completion remains Query-only; compiled contracts grant no permission.

The native prerequisite adds World-qualified managed request
admission, exact attempt-owned queue custody and genuine Signal completion commit.
Completion admission remains distinct from commit; cancellation leaves queue drain
custody with its owner. A committed result retains any refused timeout cleanup.
The Decl/Host endpoint carries these lower-owner operations through its runtime row.
Signal completion admission, queue and timeout-cleanup owners retain their actual
originating service/source binding. Public report DTOs remain diagnostic
projections and cannot construct admission or lend authority across runtimes.
Managed declarations refuse legacy admission, retry and revalidation entrances
before effects. Managed retry and revalidation retain one native custodian. One
cell lock validates the exact prior occurrence, installed definition and drained
queue, performs the native transition, and binds only a genuinely new occurrence.
Coalescing preserves the winner's queue and original business-source claim while
the custodian moves to the current service origin. Retry schedules retain their
issuer; foreign schedules cannot advance another runtime's clock. Actual partial
clock progress and acquired or retired wakes remain in a bounded cleanup handoff,
which blocks overlapping progression and cannot clear a successor's pending retry.
Timeout admission precedes fallible wake retirement, so cleanup cannot discard a
timeout transition. Bridge installs the matching port and invalidates its stale
receipt before fallible descriptive assembly. The internal Query/UI route now
retains the actual unqueued custodian through entry, progression and refused
opening cleanup. Its immutable witnesses preserve their original occurrences.
The read-only UI explicitly selects the genuine observed `BranchHead` snapshot,
without a committed-source or fresh-currentness claim. The separate authoritative
source entrance still refuses an empty target before native admission.
Initial default supersession retires the prior timeout; the separate Native
coalescing policy retires only its new unused timeout. The application cutover
removes public report-stripping initial, timeout, schedule, retry and revalidation
Native service/Bridge/World alternatives and migrates actual callers to genuine custody. Lower Native kernels remain their
single owners. Focused World, Bridge, compile-fail and public application journeys
qualify the application control handoff, provider verification, fresh
authorization/disclosure and compatibility cutover.

The qualified lower-owner retention prerequisite remains separate from Host
integration. An explicitly installed queued process-local posture uses exact
owner-selected terminal, denial, retry-lineage and retired-wake release. It
retains one fixed predecessor obligation before the next report can overwrite
its genuine selection; existing omission summaries absorb expired diagnostics.
The genuine pending retry schedule survives deferred report changes; an explicit
process-local abort retires only that exact backoff without advancing time and
retains its actual released schedule/retirement evidence.
Stored ProcessLocal occurrences reject raw Native timeout/retry/revalidation
before effects; those entrances cannot bypass the managed cleanup custodian.
Exact predecessor release carries the genuine current occurrence, preserving
its generation charge independently of newer same-node index entries.
Refusal preserves the real custodian and any completion owner. Success consumes
them into released evidence without live progression capability. Fulfilled
process-local requests require their actual completion cleanup owner before
release and fresh Start; they cannot revalidate around that handoff. Legacy
retention remains unchanged. Query installs released row state before its
original-request terminal handoff. The public journeys also qualify the finite
provider/row path; Native release alone is not the Host lifecycle proof.

The Host endpoint admits finite Native slots, allocated rows, retained result
slots and fixed ingress before effects. A genuine issued attempt/verifier binds
the producer projection, original-request scratch and actual installed decoder
before Native completion stage/commit. Verified candidates survive result-slot
refusal without a second provider call. Accepted clock-only progress is an honest
owned Reserved outcome. Provider/Native/recovery custody lives in the runtime row;
move-only handles and weak exact receipts cannot reconstruct it. Fresh exact
receipt reclaim requires Close plus Availability; there is no browse/index or
implicit abandoned-row recovery. Existing explicit runtime close refuses any
allocated row and successful close clears its Resource registry. Rust Drop and
process expiry make no Native/provider cleanup claim.

The populated-fork/sibling source comparison uses a closed receiver-owned
certification probe after ordinary State admission. It compares the actual
retained Bridge source with an independently reissued Native read, without
adding consumer metadata meaning or deriving authority from transport correlation.
The combined example drops a genuine prepared program adoption before its
sibling publication, proving the source remains distinct from a burned Native
reservation as well as a branch version. All lifecycle actions use the ordinary
Decl/Host API.

The separate public recoverable-effect journey derives a move-only handle
from an actual committed outbox/aftermath operation and admits genuine inspection.
The current terminal/outcome owner precedes the historical outbox provenance read
and is rechecked after arbitrary verification. Authenticated indeterminate
admission cannot be ordinary result commit or automatic redispatch. Refused
cleanup retains the exact row/handle; successful authorized resource cleanup
returns the original recovery handle. Declared preimage grammar admits recovery
ownership/inspection here; it is not evidence of an executable corrector.

**Depends on:** Phase 7 and the completed application entry, provider, conditional
runtime and 9.17.7 external-effect contracts. This carries the implemented 9.4
resource machinery into the current application SDK; it does not reopen 9.4's
entire lower-runtime portfolio.

Native cancellation and the immutable-witness retirement adapter remain Native-only
actions: they create no successor or new wake custody, and do not attest to
provider, queue or full Resource cleanup. The genuine custodian retains those
existing obligations.

### Requirements

- Applications declare a resource's typed input/result bindings, semantic
  identity, provider role, supported lifecycle and installed finite queue/retention
  limits through Decl. Host installs the real provider on the existing runtime;
  validation rejects incompatible declarations before resource effects.
- One retained application owner binds the original input, request occurrence,
  installed declaration and exact selected product. Fresh authenticated requests
  govern start, inspection, advancement, retry/revalidation and close. Query
  delegates transitions to Signal/Bridge/World, preserving their original request
  controls, timeout/retry policy and typed result-state distinctions.
- Preserve Pending, fulfilled/domain failure, cancellation, timeout, stale and
  superseded meaning where the installed native family supports them. Admission,
  cancellation and completion are different evidence. Refused progression retains
  the original owner; close reports outstanding cleanup rather than abandoning it.
- Provider completion enters through an installed verifier/adapter bound to that
  exact request. A payload, transport acknowledgement or copied observation cannot
  complete a successor or grant capability. Effects-indeterminate remains distinct;
  timeout/cancellation cannot imply rollback or authorize redispatch. Reuse 9.17.7
  when the request participates in an owned external effect.
- Admission, optional derived output publication and external effects retain their
  actual separate postures. Scope is process-local custody, with typed unavailable
  recovery where restart support is absent; no durable-job claim is implied.

### Tests That Prove It

The async courtroom above runs through only Decl/Host with a real installed
provider and owner-controlled completion ingress. Include one pending-to-result
journey, timeout/retry with delayed obsolete completion, disclosure revocation and
refused close followed by successful cleanup. Observe actual request identity,
result and native active-request release. Existing inbound completion and temporal
operations retain their separate semantics. Forged completion and stale request
twins must reach the disputed admission and refuse before result publication.

### Target DX

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let resources = request.resources();
let mut resource = resources.start::<Assessment>(input)?;
let original = resource.receipt();
resources.advance(&mut resource)?;
let result = resources.with_result_governed(&original, fresh_metadata, |value| inspect(value))?;
let closed = resources.close(&mut resource)?;
```

### How We Know It's Done

One compiled facade-only resource example starts, observes, completes, retries and
closes through the existing installed runtime. Its old Query orchestration has
been migrated without a duplicate lifecycle, native handle exposure or extra
Signal graph. The capability index and application guide explain its distinction
from external-input capture, temporal operations and authenticated inbound effects.

---

## Phase 9: Causal Inspection From Ordinary Application Receipts

**Depends on:** existing application receipts, scoped inspection and the retained
Bridge/Signal/Relational evidence owners. Use Phase 8 evidence when inspecting an
async resource; ordinary query/source inspection does not depend on Phase 8 or
unfinished graph phases.

The Native observation handoff prerequisite is implemented: output-readiness
delivery privately retains the actual immutable Bridge/Signal observation rather
than counters alone. Read-only custody preserves the actual invalidation backing
and its original reservation while the unique performed receipt transfers once;
it does not retain a snapshot reader or operational reentry core. See
[Native custody](../../crates/worth-runtime-bridge/README.md#read-only-native-observation-custody).
Fresh Host admission, connected producer-chain references and the application
inspection journey remain required below; this prerequisite does not close Phase 9.

Optional inspection meaning belongs to the existing ordinary query binding:
`ApplicationQueryInspectionDefinition<Schema, Binding>` takes an explicit metadata
disclosure contract, and `application_query_inspection` registers that binding's
actual descriptor. There is no separate inspection marker or string-only binding
registry. Both the definition and every metadata selector retain the complete
fourteen-field `ApplicationQueryBindingReference`, including input, result, scope
and principal meaning. The six closed facets are observation identity, evidence
links, evidence counts, evidence availability, explanation and diagnostic fields.
Governed contracts cover every facet exactly once: evidence counts require `Count`
influence; the other five require `Explanation`. Mixed, duplicate, partial and
foreign-binding selectors refuse. Public metadata requires an explicit public
contract without rules; an installed-policy placeholder is insufficient.

Metadata admission is independent of the original payload contract and never
grants diagnostic field values. The eventual fresh Host issuer must separately
check the current subject's `DiagnosticMask` before reading those fields, and
must admit requested metadata before looking up family availability, links or
counts. Installation validates every definition against the actual compiled
query-binding catalog, after ordinary query/result, scope, principal and limits
checks. Until the fresh issuer exists, valid inspection meaning returns typed
`InspectionUnavailable`; malformed or foreign definitions refuse with their own
cause before that unavailability outcome. Portable member45/v1 and selector8/v1
are additive subgrammars and preserve unextended v18 schema identity and bytes;
unknown versions refuse before payload fields or allocations.

### Requirements

- A fresh application request can select an actual declared-query observation or
  performed source-operation receipt and request a causal explanation through Host.
  Bind exact runtime, product occurrence, installed query/operation, result shape
  and retained evidence. Extend actual producers to carry genuine references when
  needed; never reconstruct authority or reference identity from display fields.
- Migrate/reuse `CausalInspection`'s anchoring, reference resolution, typed
  admission, Bridge assembly, materialization and omission contracts. A receipt
  supplies observation evidence only. Current principal, purpose, scope and field
  disclosure must freshly admit inspection and any continuation/materialization.
- Planning, admission and optional rich materialization remain distinct. Expose
  admitted, advisory, denied, unavailable/expired and cancelled postures with the
  original typed cause. Missing evidence is explicit; no invented causal narrative,
  re-executed source or success inferred from an empty evidence set.
- Reference-only and richer views use the existing evidence owners and policies.
  Protected contents cannot leak through links, counts, family availability or
  explanations. References and retained pages have bounded, explicit release/close
  custody. Optional materialization is demand-driven and original-request funded;
  it cannot become synchronous ordinary commit work or alter its charged bill.
- Source changes, invalidation, producer evaluation, equivalent-output suppression
  and query observation keep distinct meanings. The result is structured evidence
  for an application explanation, not automatic author attribution or domain prose.
- No new causal archive, graph truth store or correction permission. 9.21 retains
  ownership of new governed decision attachments and incremental summaries; this
  phase connects evidence that already exists to its real application audience.

### Tests That Prove It

Use the causal courtroom above through current application receipts and two
facades, with independent source/evaluation observations. A sibling/foreign receipt
or matching-looking display digest cannot substitute. A revoked inspection cannot
expose retained fields; expired evidence returns its actual availability posture.
Cancelling optional materialization leaves the business result and its cleanup
unchanged. Substituting a synthesized receipt or bypassing fresh disclosure must
turn these same assertions red. Do not count the older facade signature test as
proof that the new receipt acquisition and authorization path is complete.

### Target DX

```rust,ignore
// Planned syntax; receipt is returned by an actual ordinary application query.
let explanation = request.inspections()
    .for_observation::<InstalledQueryBinding>(result.receipt())?
    .why_changed()
    .reference_only()
    .execute()?;
```

The explicit installed binding preserves the original source's query, parameter,
scope and result interpretation during fresh admission. This remains planned DX
until the ordinary and governed Host entrances compile. Moving the operational
core and retaining an opaque native observation in Publication receipts do not
complete this phase: genuine producer-chain references, fresh disclosure,
availability, materialization controls and release evidence remain required.

### How We Know It's Done

A compiled ordinary query/source journey obtains and explains its genuine receipt
using only Decl/Host. The old implementation is moved into the split owners rather
than copied, and optional materialization, disclosure, availability and cleanup
are documented with the ordinary receipt path. Raw Bridge or Signal access is not
required from an application.

---

## Phase 10: Bounded Record And Aspect Change History

**Depends on:** retained native canonical changes, exact World product ancestry,
ordinary installed query/disclosure and request controls. Independent of Phases
7–9 and unfinished graph phases. Previously optional in the audit, this capability
is now required milestone scope by the 2026-10-26 extension.

**Accepted, 2026-10-27.** The installed Decl/Host route captures exact disclosed
entity and physical-relation generations from complete one-shot reads, then pages
canonical changes at a pinned product with fresh SubjectRead and independently
governed history metadata. The [facade audit journeys](../../workspaces/worth-query/crates/worth-query-certification/tests/application_graph/document_retention_model/record_history/court.rs)
and [physical occurrence journey](../../workspaces/worth-query/crates/worth-query-certification/tests/application_graph/relation_laws/history/court.rs)
qualify equal-value occurrences, fork/sibling exclusion, ordered/filtered physical
IDs, deletion/replacement, current-security revoke/regrant, typed retired gaps,
cancellation and final Native/World release. Native and World owner courts qualify
finite source visits, page backing and immutable request-bound successors.
Partial ordered/live capture and restart-persistent continuations remain outside
this contract. The [capability guide](../../workspaces/worth-query/crates/worth-query/docs/capabilities/record-and-aspect-history.md)
describes the supported route and its explicit work/storage limits. The
[Native growth court](../../crates/worth-runtime-world/tests/runtime_world_certification/court/record_history/growth.rs)
compares ten and 10,000 real unrelated retained record changes with identical
one-entry/one-visit/page-backing limits. Individual request work maxima remain
equal, and the exact full traversal reaches the independently selected last
record after every empty incomplete prefix.

### Requirements

- Declare a typed entity/relation history query with selected aspect references
  through the ordinary query declaration/installation owners. Execute at an exact
  retained product observation through Host. Relational selects canonical native
  changes; World supplies ancestry/component correspondence; Query controls schema
  interpretation and fresh disclosure. This is not a second correction history.
- Return genuine commit/version/branch and record identity, changed aspects and
  structural/opacity facts. Preserve retained equal-value write events; absence
  from a value diff is not absence from authored history. An unchanged component
  retained by several product commits must not duplicate one native change event.
  Native events outside the selected product ancestry must not appear.
- Direct record identity includes generation. Automatic entity-successor or
  aggregate-lineage traversal is not implied; an explicit future lineage selection
  must use its existing owner. No before-images, actor attribution or inverse are
  fabricated from aspect-change metadata.
  Resolve the typed subject in an owner-retained disclosed observation or retained
  canonical tombstone/history target. A deleted record remains inspectable only
  through such fresh admission; a raw identifier cannot recreate its provenance
  or borrow disclosure from a later entity occupying the same slot.
- Add a native retained cursor/page over canonical changes before application
  carriage. Bound returned entries, scanned canonical source items and owned page
  memory using explicit page limits and original request controls. A cursor can
  stop inside a large commit. Never materialize the full branch vector and slice
  it, or restart the ancestry scan for every page.
- Follow actual retained ancestry/publication order, not numeric commit-ID order.
  A continuation retains exact runtime, product observation, query/target/filter
  and cursor position; it grants no new read authority. Fresh request admission
  applies on every page. New commits do not silently move a pinned history basis.
- Incomplete, complete, cancellation, expiry/unavailability, foreign cursor and
  disclosure refusal remain typed. An empty page may carry remaining work; it is
  not evidence of no change. No hidden entries, protected counts or cursor metadata
  leak. Cancellation preserves the last accepted continuation, and Drop/close
  releases the actual native/World pins and page storage.
  Installed history disclosure must explicitly cover occurrence existence,
  selected ancestry and continuation/completion metadata, independently of payload
  visibility. Refuse before traversal when that metadata cannot be disclosed;
  do not claim a complete history by silently dropping protected source events.
- Initial total work may grow with retained ancestry and examined change items;
  say so in the API/guide. No record-local cost or constant-time claim is made.
  Optional indexing must remain derived and governed by the existing native owner,
  not a Query-maintained canonical event log or an unrequested search product.

### Tests That Prove It

Use the record/aspect courtroom above, with a full unpaginated reference result
from the independently recorded authored sequence. Concatenating lawful pages
matches that sequence with no duplication, omission or sibling event. Check
reverse occurrence/publication order, unchanged composite components, deletion
and generational replacement. A scan-all-then-truncate mutant must fail the finite
page-work condition; an empty-page-is-complete mutant must lose an expected later
event. Wrong basis/filter and revoked disclosure must refuse with no protected
output. Check cancellation, expiry and final retention release at their real owners.

### Target DX

```rust,ignore
// Illustrative: not compiled.
// Installed history, captured subjects and selections come from actual Host admission.
let page = runtime.read_application_history(
    &phase, &history, &external, &subjects, &mut cursor,
    current_security, &selected_truth, &request, limits,
)?;
for event in page.entries() {
    inspect(event.commit_id(), event.record(), event.changed_aspects());
}
let complete = page.is_complete();
```

### How We Know It's Done

One compiled facade-only audit-panel fixture selects a real record, reads and
continues its aspect history, handles denial/expiry and releases all retained
custody. Native bounded paging replaces the eager collection on this application
route. Existing historical state reads, product ancestry and 9.18 corrections
remain supported by their original owners.

---

## Phase 11: Batched Obsolete Catalog And Facade Cleanup

**Depends on:** the surviving identity issuance and standalone Bridge facade.
Independent of the new runtime integrations; implement both cleanup findings in
one reviewed batch. Cleanup of orchestration displaced by Phases 8–9 belongs to
those integration phases, not this batch.

**Accepted, 2026-10-26.** The three Relational planning catalogs and the three
hidden Bridge facade aliases are removed together. Source/caller inventory finds
no executable references to the retired symbols. The existing grouped World
delivery and owned-async courts preserve native source selection/receipt delivery,
supported root-facade consumption and World/source-owner refusal. Actual identity
authority, kinds, categories, owner implementations and compile-boundary tests
remain unchanged. The current
[Relational source-identity contract](../../crates/worth-relational/API_OVERVIEW.md#exact-source-identity-and-receipt-authority)
and [Bridge import guidance](../../crates/worth-runtime-bridge/API_OVERVIEW.md#public-surface-map)
replace the stale planning/import guidance.

### Requirements

- Remove Relational's `phase_one_family_map.rs`,
  `phase_one_compile_fail_targets.rs` and `phase_one_root_break_targets.rs`, with
  their module/export declarations. These are stale textual planning inventories,
  not identity authority or mechanically executed compile-fail tests. Preserve
  `authority.rs`, `categories.rs`, `kinds.rs`, actual source selection/receipt
  issuance and real compile-boundary tests.
- Remove only Bridge's hidden `everyday`, `advanced` and `specialist` wildcard
  alias modules. Preserve the supported standalone root facade and substantive
  specialist capabilities. Migrate any newly discovered real callers to the
  surviving root import in the same batch; retain no compatibility forwarding bag.
- Reconfirm callers, exports, documentation and supported compatibility before
  deletion. Remove stale catalog/alias references and refresh affected facade
  snapshots. This batch does not retire Signal's expressly supported deprecated
  admission aliases, low-level resource owner, isolated tests or other engines.

### Tests That Prove It / How We Know It's Done

The deleted symbols have no implementation or executable callers left; the old
catalog's nonexistent test paths are no longer advertised. Existing native
selection/receipt and foreign-owner tests plus the supported standalone facade
consumers preserve the real boundary. Use focused existing grouped compile/runtime
checks and a residue search, not a new catalogue-validation or getter test suite.
Both cleanup findings ship together with corrected public import guidance.

## Proprietary Platform Follow-through — 2026-10-27

The CAD report against public SDK `ba86433627` adds the following required
phases. This is the report's basis, not an instruction to change an SDK pin.
Existing accepted phases retain their historical acceptance. These findings
extend the milestone's ordinary preparation, candidate validation, failure and
consumer qualification boundaries; none is an optional closeout item.
The index implementation and its spatial-dependent locality acceptance are now
required scope in [9.19](milestone-9.19.md), whose final phase performs Proprietary
cutover after public qualification. Numbers 12 and 15 are retained as transfer
references, not local implementation phases or claims of completion.

**Addendum 1.** Partitioned computation with per-partition reuse is withdrawn as
a gap: the existing managed-computation contract in
[`docs/build-an-application.md` section 3.6](../../docs/build-an-application.md#36-declare-and-install-a-partitioned-managed-computation)
already supplies declared partitions/keys, tracked gathering, pure partition
computation, owner completion and reuse retained on the producer record at the
reported SDK. The one-retained-computation-per-handler rule remains a consumer
design constraint; this finding does not require another retained stage or a
relaxed handler limit. This withdrawal names the reuse finding, not Phase 13's
separate compiler-qualification refusal. The spatial finding remains required in
9.19 on both committed preparation and candidate invariant readers. Consecutive
same-field undo is a new P1 correction-derivation requirement in Phase 19 below;
all other findings retain their scope and priority.

| Priority | Finding | Required phase and owner |
| --- | --- | --- |
| P1 | Retained spatial range reads in preparation and candidate validation | 12 transferred to 9.19 — general managed-index extension; provider owns spatial mechanics |
| P1 | Unqualified compiler reported as byte overflow | 13 — worth-foundational + Query Admission/Execution |
| P1 | Consecutive undo refuses an earlier source after later overlapping sources are reversed | 19 — Query correction derivation and canonical correction lineage |
| P2 | Unobserved field write loses its cause | 14 — worth-query-execution |
| P2 | Scoped invariants must stay local on real large bodies | 15 transferred to 9.19 — public qualification followed by final Proprietary adoption |
| P3, required | Cell-version storage and live-at-k cost | 16 — Relational + Query, with Proprietary CAD acceptance |
| P3, required | Proposal acceptance reissues ordinary requests | 17 — Query entry/admission + public application authoring |
| P3, required | Qualify Rust 1.99.x and keep the supported stable toolchain current | 18 — worth-foundational + workspace toolchain/CI owners |

### Decisive Proof For The Added Phases

The following spatial journey is the consumer requirement transferred to 9.19;
its self-contained phases govern implementation and final private adoption.
Use the real Decl/Host installation and the installed CAD entry. In a committed
1,000-vertex chain, move a middle vertex so its incident cells encounter an
otherwise untouched, non-incident cell. Operation preparation discovers pairs
from the retained committed spatial root; the installed invariant searches the
proposed root, detects the new contact and refuses publication. An otherwise
equal non-contacting move commits. Repeat at 2,000 vertices with the same local
edit and spatial distribution: work differs with index height, affected incidence
and actual hits, rather than a full sweep or rebuild. In the METABOSS
50,000-face body, a boolean touching one face discovers the touched candidate
pairs without visiting all unrelated faces. Pair discovery is observed separately
from the already partitioned pair-certification computation.

Compare range results against an independent exhaustive geometric reference in
the test, including empty ranges, retired/made/moved cells and incident edges.
After refusal or interruption, independently read committed graph truth and the
committed range result; both must retain their original basis and contents.
Stale committed-root substitution, missing incident-edge maintenance, omitted
empty-range dependencies and scan-all-then-filter implementations must fail this
evidence. Dense overlap may legitimately increase the number of candidates;
these locality claims do not promise constant work for arbitrary distributions.

The failure courts remove the decide-time `DefinitionRecordField` observation
from the real installed record probe and exercise a genuinely unqualified sort
admission. Both must retain their own typed cause through the public caller,
before publication. A generic provider rejection or charged-byte overflow cannot
satisfy either case. The proposal court accepts retained authored intent under
fresh authority, then repeats with revoked authority or a stale basis; recording
the proposal supplies neither acceptance permission nor replay authority.

### Destination Ownership And Module Boundaries

The following tree names owners for Phases 13–14 and 16–18.
The generic index topology is governed by 9.19, not a spatial subsystem here.
Created destinations are populated by their implementing slice, not by
empty placeholders in this specification change. Implementation planning may
refine private children without moving the public facade or changing ownership.

```text
crates/worth-foundational/
  build.rs                                      [existing: exact compiler qualification]
  src/canonicalization/basis/construction/
    admitted.rs                                 [existing: typed preparation outcome]
    sort_admission.rs                           [existing: reviewed sort work/storage bound]
    compiler_qualification/                     [created: compiler identity and eligibility meaning]
workspaces/worth-query/crates/
  worth-query-admission/src/
    application_query/parameter_canonical_basis/  [existing: preserve sort qualification refusal]
    canonical_identity_derivation/               [existing: preserve canonical cause]
  worth-query-execution/src/domain_computation/primary_graph/
    application_attempt/provider_binding/effect_lowering/
      entity_field_patch.rs                     [existing: exact unobserved-write refusal]
    application_attempt/compare_and_commit/commit_outcome/denial/
      unique_value.rs                           [existing: preserve lowering cause and stage]
  worth-query-certification/tests/application_graph/
    proposal_acceptance/                        [created: fresh ordinary acceptance journey]
```

9.19 extends Query's existing managed-view owner for generic provider roots,
candidate staging and dependencies. Relational supplies source/candidate custody
and commit integration; numerical algorithms remain outside the generic runtime.
This milestone creates no parallel index registry or specialized spatial facade.

Stable public entry remains `worth-query-decl` and `worth-query-host`.
Preparation and candidate adapters consume their respective original read
authority; neither can mint publication or mutation capability. Private module
visibility, facade/dependency checks and the existing archive-version discipline
enforce this direction. No unmanaged application-owned index cache, generic callback
validator, new executor or production replay entrance is permitted. Cost courts
extend the existing grouped Native/Query targets and CAD installed tests; they
do not introduce a cost registry or a persistent implementation journal.

---

## Phase 12: Managed Index Finding — Transferred To 9.19

**Status: transferred; not implemented or accepted by this document. Priority: P1.**
The general managed derived-index extension, committed preparation reads,
candidate invariant reads and spatial/nonspatial qualification are required in
[Milestone 9.19](milestone-9.19.md). Its final phase performs the single integrated
Proprietary cutover. Phase 6 supplies shared prerequisites; this transfer does not
add spatial meaning to its graph contracts. No independent Phase 12 implementation
or private adoption is required to close 9.18.1.

---

## Phase 13: Honest Compiler Qualification Refusals

**Status: complete. Priority: P1.**
**Owner:** worth-foundational + Query Admission/Execution.
**Depends on:** existing canonical sort admission; independent of Phases 6, 8–9
and 12. Rust 1.99 qualification does not substitute for this repair.

### Requirements And Acceptance

`canonical_verified_std_sort` requires an exact release/full-commit identity;
the [contributor toolchain contract](../../CONTRIBUTING.md#rust-toolchain-and-canonical-sort-qualification)
lists the supported identities. Preserve an unqualified implementation as a typed
compiler-qualification refusal containing the actual compiler release and full
commit, rather than `AccountingOverflow` or
`ExecutionResource { denial: ChargedBytesOverflow }`. Carry that cause through
foundational preparation, parameter/canonical-identity admission and original
source disclosure to the ordinary caller without relabeling its failure stage.
Real arithmetic overflow and request exhaustion keep their own distinct causes.
No successful canonical artifact or downstream publication follows refusal.

Use the Proprietary `tools/correction-sdk-probe/` reproducer identified on
`codex/undo-redo-20261027` at `0eea66558`, plus a focused owner court and a qualified
compiler twin. Exercise the refused qualification boundary even after the default
compiler is upgraded; a forced byte-overflow surrogate is not evidence. Audit
all mappings of `UnsupportedSortImplementation`, including graph-index inventory
and canonical identity paths. Public machine-readable release/commit context and
the qualified twin must reach the caller before this phase closes.

---

## Phase 14: Preserve Unobserved Field-Write Causes

**Status: complete. Priority: P2.**
**Owner:** worth-query-execution.
**Depends on:** existing decision observations and candidate effect lowering;
independent of the new spatial and async work.

### Requirements And Acceptance

An existing-entity field write without its required decision observation refuses
as typed `UnobservedWrite`, naming the exact entity generation and field locator.
Enforce at the earliest boundary with that information and retain the cause
through `EffectLowering`; do not translate it into an unexplained
`ProviderRejected(ProposalBinding)`. If detected during writing, preserve that
earlier stage rather than pretending lowering occurred. Observation of absence,
entity creation and actual stale/foreign observations retain their distinct
existing semantics. A locator identifies the attempted write, not disclosed
field contents or new authority.

Remove the decide-time `DefinitionRecordField` read from the installed CAD
`installed/record.rs` journey. It must reach the disputed write, refuse with the
typed locator and cause, and leave truth unchanged. Restore only that read for
the lawful committing twin. Both public effect-lowering and final caller
assertions must expose cause-erasing translations; a direct private constructor
test alone cannot close this phase.

---

## Phase 15: Spatial-Dependent Invariant Locality — Transferred To 9.19

**Status: transferred; not qualified by this document. Priority: P2.**
[Milestone 9.19](milestone-9.19.md) owns the 1,000/2,000-vertex and 50,000-face
locality requirements, including preparation, maintenance, scope acquisition,
non-incident contact, ordinary validation and refusal atomicity. Its public
phases expose defects before one final real CAD cutover. The requirement remains
mandatory there; this transfer neither proves locality nor weakens the affected
graph phases' existing tests. It is not a 9.18.1 closure dependency.

---

## Phase 16: Cell-Version Storage And Live-At-k Qualification

**Status: unfinished; required. Priority: P3.**
**Owner:** Relational + Query, with Proprietary CAD Acceptance 8.
**Depends on:** real cell-bearing publication and the accepted history reader.
This phase qualifies cell-version storage and history reads independently of the
transferred index work. Spatial-dependent integrated locality is rechecked in
9.19's final Proprietary phase and is not a prerequisite for this phase's closure.

### Requirements And Acceptance

Measure entities, relations and retained bytes per cell version, versions made
per local edit, and real `live_at(k)` query work at 1, 100 and 10,000 features.
Separate authoritative version records, retained history, derived/index storage,
transient/peak allocation and returned live cells. Vary history length and live
body size independently; compare warm local edits, cold reconstruction and
historical reads without charging one lane to another or omitting setup work.

Use installed cell-bearing publications and actual platform observations, with
an independently authored live-cell reference. State the expected growth class
before measuring and report observed results in the existing CAD acceptance
document, with the named workload and environment. Repeated whole-body copies
for local edits, hidden full-history rebuilds in the ordinary edit lane, or a
failure of CAD Acceptance 8 must be fixed at the responsible platform owner.
Output-sized historical materialization is reported honestly; no arbitrary
constant-time live-history claim or fitted exact-count golden is required.
No private packed store, second version table or replacement cost ledger may
stand in for this acceptance.

---

## Phase 17: Proposal Acceptance Through Fresh Ordinary Requests

**Status: unfinished; required. Priority: P3.**

The public expected-source fork prerequisite is qualified. It consumes an
admitted application selection, retains that exact source
with the created candidate, and preserves candidate/source custody on refused
close. Guarded model-head adoption is also a qualified public prerequisite:
the candidate checks Native bases and holds World's exact source/candidate
currentness guards across replacement of the application's actual branch slot,
checking the same request's liveness inside the guarded section. Stored
expected-basis readmission is also a qualified public prerequisite:
versioned comparison data re-enters the actual World/Native owners, including
Native's separate exact-currentness check before application selection. The
owner/application cases and current repository gates pass.
The complete public stored-proposal acceptance journey remains unfinished.
**Owner:** Query entry/basis admission and Native/World source-basis receivers;
the public application fixture owns its domain intent and model selection.
**Depends on:** ordinary governed application requests and existing source
operation/correction contracts. Independent of spatial indexing and Phases 8–9.
The ordinary expected-source handoff belongs to this phase. Applications consume
the platform's selected-source custody instead of implementing currentness.

**Certification boundary:** public code and public tests are sufficient to close
this phase. The real application composition uses only Decl/Host audience facades;
focused owner tests supplement it where they expose private synchronization or
retention semantics. No private repository, CAD integration, SDK pin change or
private test run is a Phase 17 gate. Public closure certifies the public contracts
below; it makes no claim that an untested private consumer has adopted them.
Other phases' explicit private adoption gates remain unchanged.

### Requirements And Acceptance

A proposal stores authored intents and typed domain references as ordinary
versioned domain data. It stores no executable platform receipt, admitted writer,
replay proof or transferable permission. Acceptance interprets the selected
intents and reissues ordinary requests under the current principal, scope,
disclosure, basis and installed operation contracts. Each attempt receives its
actual ordinary outcome. Stale basis, revoked authority and unavailable domain
references refuse through their existing owners; acceptance never silently
rebases or substitutes identities. Multi-intent partial acceptance follows the
existing operation/workflow effect posture and cannot imply rollback.

Compile and exercise a real public Decl/Host application acceptance journey:
record, retain, accept under fresh authority, then exercise revoked authority and
stale-basis twins. Ordinary production import/dependency checks must reject
`worth-query-replay` on this route; cert-only replay remains available only to its
existing audience. Document how the existing request mechanism supplies this
path, correcting any missing facade integration without introducing a proposal
executor or using certification replay to simulate ordinary acceptance.

### Current Public Boundary And Decisive Court

`branches().select_expected` freshly admits stored comparison data through World
and Native. Native readmission can admit retained history, so its separate exact
currentness comparison and equality with World's component tuple remain mandatory.
`fork_expected` consumes the resulting application selection. Native rechecks
that source during creation; World carries the expected composite observation.
The resulting candidate retains source custody and offers guarded replacement
of the application's actual model-head slot. Ordinary `fork` still resolves a
source during creation and cannot substitute for this contract.

The decisive public journey records a real edit against model A, releases the
recording request and later accepts under the current caller. Recording must not
move A's authored-model head. Move A after expected-source comparison but before
fork creation, and separately while the candidate is being evaluated before
adoption. Acceptance must refuse the stale selection at the receiving owner, or
the latter movement must be structurally impossible under an actual exclusive
model transition. A standalone precheck, an unguarded head-map update or a final
check after forking a different source cannot pass this court.

Retire and recreate a referenced feature under the same display name, and reopen
another session whose local handle spelling is again `model-1`. Neither may bind
the stored proposal to a replacement occurrence. Revoke proposal/result READ and
mutation authority separately, including on an idempotent acceptance retry. The
ordinary denial must occur before the protected lookup or effect it governs.
Include a two-intent refusal and observe the real staged-fork cleanup and published
model head; report the existing partial, committed, indeterminate or cleanup
posture without inventing rollback. Observe typed authored parameters and committed
application state independently of the proposal's returned result. Geometry is
not a claim of this public phase and requires no private CAD fixture.

### Basis, Storage And Admission Contract

- Proposal data retains the genuine model incarnation, typed target occurrences,
  authored intents and descriptive expected model/product-basis coordinates.
  A process-local handle, feature name or counter alone is insufficient.
  Descriptors are comparison data, not read admission, fork custody or permission.
  No admitted basis, snapshot lease, writer or executable receipt is serialized.
- A fresh ordinary request admits proposal/model visibility and compares the
  stored expected selection through the actual Native/World owners. The exact
  admitted selection then reaches Query's fork receiver and World's existing
  expected-source creation plan. The receiver must not resample a newer source
  after comparison. Missing, foreign, retired and stale selections retain typed
  refusals; equal payloads do not establish the original basis.
- Expected-source fork construction requires the freshly admitted selection as
  a nonoptional input. Ordinary unconditional fork-from-current remains a
  different product operation; it cannot satisfy proposal acceptance. Preserve
  Native's source-generation/root/version recheck and World component affinity.
  Reuse their existing evidence and custody rather than adding a second proof
  registry or a proposal-specific platform executor.
- Protect adoption of the candidate as the selected model's head with the same
  expected source. Use the existing owning transition to compare and adopt, or
  carry a structurally enforced exclusive interval covering selection, fork and
  adoption. An application convention about exclusive access is not proof.
  Fork construction consumes the admitted selection into live transition custody;
  its result retains the candidate and expected-source continuation needed for
  adoption. A bare branch handle cannot certify that transition. Refusal after a
  fork exists preserves its actual cleanup or recovery custody and cannot claim
  that no effect occurred anywhere.
- Store proposals as ordinary versioned domain entities on a separately owned
  storage branch, so saving a proposal does not advance the checked authored
  model head. Its live branch custody belongs to the application runtime, outside
  the stored record. Specify close, cancellation, unavailable storage and pending
  cleanup through existing lifecycle owners; no immortal side map or retention
  ledger substitutes for versioned data.
- Authored parameters retain their declared operation binding and typed codec
  through storage and loading. The public fixture uses registered structured
  operation inputs; it cannot store a generic JSON value and recover meaning
  from an operation name alone. Applications interpret those intents through
  ordinary source operations, never through receipt replay or a raw field writer.
- Fresh proposal/model/result disclosure precedes cached-result disclosure at
  the proposal ingress. Create a new acceptance-attempt idempotency key and reuse
  it only for retries of that attempt. Do not reuse the recording token or an
  old continuation. Preserve ordinary operation idempotency; do not move its
  established semantics merely to conceal an unauthenticated proposal ingress.
  The accepting host/caller identity governs both loading and execution; stored
  authorship supplies neither identity nor authority.

### Operational Sequence And Authority

These are semantic boundaries in the existing application/owner path, not a new
platform proposal executor. [Architecture Laws](../../docs/coding-guidelines/arch_laws.md)
1, 4, 5, 9, 10, 14, 16, 17, 19–22
govern the transitions; the [testing laws](../../docs/coding-guidelines/testing_laws.md)
govern their evidence.

1. **Initialize the application model.** Its owner holds a genuine model
   incarnation and the actual mutable head slot. Model identity must survive
   head movement and differ for another model or a recreated session, even when
   display names match. Issue identity through ordinary installed domain
   creation; never fabricate a generation or hash a display name. A model's
   owned head and a shared borrowed seed have different close obligations.
2. **Record data without authoring the model.** Through an ordinary authorized
   operation, store the versioned proposal on its separately owned storage
   branch: model incarnation, full-generation typed target references, ordered
   typed inputs and expected-basis description. Storage custody stays in the
   application runtime. Release the recording request and selection before
   acceptance; only domain data survives. Unsupported record versions refuse
   before candidate creation or authored effects. Coordinates address the
   original live installation; durable process-restart restoration is outside
   this phase, as it is for the milestone's other retained artifacts.
   An attempt binds the selected proposal occurrence/version and ordered input
   meaning. A later proposal revision cannot silently replace the loaded intent;
   protocol format version and domain record version are distinct concepts.
   Fork the component that proposal storage writes: a different product name
   sharing a mutable Relational branch is not write isolation. Recording must
   leave the original model basis admissible for subsequent expected selection,
   not merely leave its displayed values and application slot unchanged.
3. **Enter a fresh attempt.** The current caller supplies the principal, scope,
   request lifetime and a fresh acceptance-attempt key. Fresh ordinary READ
   admission covers proposal, model and any existing result before protected
   result lookup or disclosure. Mutation authority is independently admitted
   through the ordinary granting owner, including on a retry. A stored author,
   cached result, boolean permission flag or descriptive basis grants none of
   these permissions. Do not add a second permission registry.
   Declare and invoke real governed reads for the independent READ cases. The
   existing retention example supplies value/rule semantics; its unrestricted
   public-read helpers alone cannot demonstrate revocation. Change genuine
   admitted grant state through its ordinary owner, never a fixture-only flag.
4. **Resolve retry before new execution.** An authorized retry of the same
   attempt resolves its original ordinary outcome/continuation without another
   adoption. A successful attempt has itself moved the model head: its retry
   must not be rejected merely because the proposal's original basis is now
   stale. A new attempt must compare the recorded basis again. Keep ordinary
   operation idempotency ordering intact; changed intent under an existing key
   receives the ordinary intent-drift refusal. Never reuse the recording key.
5. **Readmit, then create from that selection.** Compare the loaded genuine model
   and target occurrences with this application's model using declared ordinary
   domain reads/affinity contracts. A valid target in B is not a target in A.
   Ask
   `branches().select_expected(&description, &phase)` for fresh World/Native
   admission and pass its owned result to `fork_expected`. Do not drop it and
   call unconditional `fork`, resample latest state or normalize equal values
   into identity. Reject ineligible input before expensive construction.
6. **Execute on the candidate.** Reissue the ordered authored inputs through
   their declared ordinary operations under current authority. Preserve actual
   source publication, invariant, required-output and request outcomes. Prepare
   adoption metadata before the irreversible selection boundary. Partial work
   on a private candidate is real staged work, even when the model stays on its
   source. No authored handler runs inside World's guarded replacement callback.
7. **Adopt the actual slot.** Consume the retained candidate with
   `candidate.adopt(&mut model.head, &phase)`. Source/candidate currentness and
   original-request liveness must cover replacement. A temporary local copy
   followed by an unguarded map insertion is forbidden. Hold exclusive ownership
   of the application's model state through its infallible metadata update;
   `&mut` on an unrelated wrapper does not exclude aliased platform writes.
8. **Finish or retain custody.** On refusal before adoption, the model's actual
   selection remains unchanged; retain the same candidate/recovery owner or
   close it through existing lifecycle. On success, update model metadata and
   retain the actual performed outcome before retiring the previous owned
   branch. A later old-head close, result delivery or result-storage failure is
   committed-with-cleanup/delivery-failure, never an unchanged-model refusal.
   Retain the attempt continuation across this gap so retry cannot apply twice.
   Shutdown, cancellation and unavailable storage must leave each obligation
   with an explicit owner; no permanent side map substitutes for managed custody.

The stored record is non-authoritative data. The selected operation is freshly
owner-admitted custody; the expected-source candidate carries it into creation
and adoption. The actual replacement is the application selection commit point.
Existing owner results decide whether component effects, unpublished recovery or
deferred cleanup also exist. Translation may add application context but must
preserve the owning typed cause and posture (Architecture Laws 5 and 14).

### Public Failure Matrix

Use one small installed public application with independently created models A
and B, genuine target occurrences, a separately owned proposal-storage branch
and real ordinary operations. Reuse the document-retention model's semantics
where suitable: A starts at 7 days, B at 5 days, and the proposal sets A to 3
days. Use the first program's installed rule (maximum 10 days), so the positive
edit is lawful and a later 11-day edit reaches a genuine invariant refusal.
The fixture's declared application type and registered schema/contributions
must agree; no ad-hoc declaration may bypass installation or publication checks.
A fresh declared query must read 7 before recording, still 7
after recording, and 3 only after adoption; B must remain unchanged. Read the
stored typed input independently of the acceptance return value. Registered
structured inputs include their target identity and parameter value, so dropping
either changes the intended operation. No producer-output feature from unfinished
Phase 9 or spatial work from 9.19 is needed to prove this path.

The rows are distinguishing scenarios, not a Cartesian product or a fixed test
count. Parameterize cases sharing one cause. Every negative case first proves
that its setup and unmodified positive counterpart reach the intended boundary.
Assert the typed cause, actual model selection, committed reads and surviving
custody; a generic `is_err`, string match or receipt echo is insufficient.

| Failure pressure and lawful sequence | Required outcome and independent observation | Defect the public evidence must expose |
| --- | --- | --- |
| Record, release all original request/selection custody, load under a different authorized caller and accept. | Ordinary performed outcome; loaded typed input is 3; recording changes only storage, adoption changes A to 3, B stays unchanged. | Recording secretly edits A, stored authorship grants authority, or returned success hides an unchanged/wrong head. |
| Store/load a structured input through its declared codec; splice a different registered binding with a similar representation, or supply an unsupported proposal/description version. | Intended decoding/binding/version refusal before candidate effects; authorized original roundtrip succeeds. Retain `UnsupportedDescriptionVersion`/admission cause where applicable. | Type erasure, name-based redispatch or silently interpreting unsupported bytes as current input. |
| Record A's basis, move A, then try a new attempt; repeat after restoring equal payload values. | Typed stale World/Native selection refusal; no adoption and no silent rebase. Freshly recording the moved basis succeeds. | Revision counters or payload equality substituting for exact owner currentness. |
| After successful expected selection, commit a real source move before fork creation. | Expected-source creation refusal at its receiving owner; actual head remains the moved source. A fresh selection creates successfully. | Compare-then-unconditional-fork race. |
| Run a real candidate operation, then move the source before adoption. Separately replace the model's actual head slot. | Source-basis/head-currentness denial in the first case, `HeadReplaced` in the second; neither overwrites the winner. Candidate effects remain inspectable until ordinary close. | A standalone precheck or adoption of a temporary slot instead of the actual application slot. |
| Present foreign installation/phase data, current World data with retained old Native history, and mismatched component data. | Preserve `Admission`, `NativeReadmission`, `NativeCurrentness`, `ComponentMismatch` or `ForeignPhase` at the deliberately reached boundary. No candidate/adoption from foreign custody. Carry existing qualified cases where unchanged. | Historical admission mistaken for currentness, cross-owner recombination or scope laundering. |
| Retire and recreate the target under the same display name; create another session whose local model spelling is again `model-1`. | Genuine occurrence/incarnation refusal before target mutation. The replacement's committed state is unchanged; a proposal genuinely recorded for it succeeds. | Looking up the latest name or local handle and substituting its occurrence. |
| Combine A's model/basis with B's genuinely issued target, or a result belonging to another proposal/attempt; revise a proposal after its selected input was loaded. | Typed domain/attempt-affinity or intent-drift refusal for cross-binding; both models remain unchanged. A loaded attempt retains its explicitly selected proposal version, never silently accepts the revision's different input. Valid matching bindings succeed. | Treating full-generation identity as proof of model membership, or a cache key/name as proof of proposal/version/attempt affinity. |
| Revoke proposal READ, model READ and result READ independently, with a real cached successful result present. | Owning disclosure refusal before protected lookup/disclosure; no protected payload, result, identifier or debug detail escapes. Restoring the actual grant admits the corresponding read. | A cache hit or basis descriptor bypassing fresh disclosure, including on an idempotent retry. |
| Leave all READ grants valid but revoke mutation authority; repeat with the same completed attempt key. | Owning mutation-authorization refusal before governed lookup/effect; the cached result cannot supply authority. Model state stays unchanged. | READ being treated as permission to author, or ordinary retry weakening proposal-ingress authority. |
| Authorized same-attempt retry after success; changed input under that key; two contenders using the same attempt. | Existing idempotent prior outcome/continuation or intent-drift refusal as appropriate. At most one adoption for the attempt; losing candidate is retained or closed. Observe actual selected branch and performed occurrences, not a handler invocation counter. | Duplicate authoring, replaying after result-delivery loss, or stale-rejecting every successful retry because it moved its own source. |
| Two-intent proposal: the first ordinarily commits 3 on the candidate; the second requests 11 and reaches the installed maximum-10 invariant. | The intended second-operation typed denial; actual model still selects its 7-day source. Candidate first-step state is independently readable as 3, and any unpublished recovery remains honestly owned until closed. | A preflight/setup failure masquerading as partial-work evidence, or claiming nothing happened anywhere. |
| Cancel/expire the request before effects, after candidate work and while waiting to adopt. | Typed stop with the same request identity. No adoption before the commit point; preserve candidate/recovery custody after staged work. A fresh authorized request can close or retry the retained owner. | Treating cancellation as rollback, losing candidate effects, or ignoring a stop after waiting for guards. |
| Separately retire the candidate before adoption, refuse candidate close, and retain a real reader that defers cleanup. | Candidate-admission/close denial preserves the identical candidate/source continuation and actual retired/live posture. The original model stays selected. Release the reader and settle deferred cleanup through its actual retry owner. | Replacing custody with a bare branch number, leaking an invisible branch, or interpreting pending cleanup as no effect. |
| Force an existing owner publication gap during candidate creation or execution. | Real `ProductUnpublished`/ordinary publication recovery, unchanged application selection, retained original recovery custody. Resolve that recovery before issuing a replacement attempt. | Mocking a failed commit, losing component effects, or retrying as though publication was definitely absent. |
| Adopt successfully, then refuse old-owned-head close or result delivery/storage. | The new head remains selected and reads 3. Return the committed posture with actual cleanup/delivery continuation; an authorized retry does not adopt again. | Closing the source before adoption, false rollback after the commit point, or dropping the only retry owner. |
| Close or make proposal storage unavailable; cancel recording/acceptance; shut down with pending cleanup. | Owning unavailable/stop/cleanup outcome, no fallback to an ungoverned map and no mutation of A from a failed recording. Terminal disposal releases storage and candidate obligations after retained readers are released. | Immortal storage, hidden resources or unavailable storage silently becoming latest-model execution. |
| Try to use description data as selected custody or import replay/internal runtimes from the ordinary application route. | Existing compiler/dependency enforcement refuses the invalid route; valid Decl/Host application still compiles. | A fixture-only bypass or a proposal-specific platform executor disguised as ordinary execution. |

For races, use deterministic owner-local barriers or existing scoped controls at
the comparison, creation and adoption boundaries; sleeping is not a schedule.
Drive a genuine competing publication. If a model owner's typed exclusive
interval makes one mutation schedule impossible, prove that exclusion and run
the strongest reachable competing schedule; a comment or an unused mutex is
not exclusion. Owner tests may expose an otherwise inaccessible interleaving,
but the public application must also consume the real guarded receiver.

### Evidence Boundaries And Certification Run

The exact consumer court `proposal_acceptance/courts/attempts/handoff.rs` may arm
Publication's receiver-local source-completion handoff pause. It parks only after
the original operation has genuinely completed and before the initiating request
returns. Cancelling that real scope must retain actual Fresh evidence for a new
authorized request. The control may not inject an outcome, modify command meaning,
mint authority or substitute a second runtime. Keep this caller closed in
`control-callers.txt`; ordinary application sources cannot arm it.

The exact `application/acceptance/publication_tests.rs` consumer court may obtain
the installed runtime's World compare pause through Execution and arm that same
World receiver. It cancels the real request after component effects, releases and
joins the worker before assertions, and observes genuine unpublished custody.
Its private application-state inspection is read-only; all business execution,
admission and recovery still use the ordinary Decl/Host application paths. Keep
both control-owner caller entries exact and closed.

The exact `application/candidate/publication_tests.rs` consumer court may use the
same installed World compare pause during a genuine candidate edit. It retains
the actual ordinary operation across cancellation and native recovery, then
observes its Recovered completion and independently reads candidate state before
adopting the application's unchanged source slot. Its private staging inspection
is read-only and occurs through the freshly admitted attempt view; keep both
control-owner caller entries exact and closed.

The [existing expected-source cases](../../workspaces/worth-query/crates/worth-query-certification/tests/application_graph/proposal_acceptance.rs),
[adoption cases](../../workspaces/worth-query/crates/worth-query-certification/tests/application_graph/proposal_acceptance/adoption.rs)
and [stored-basis cases](../../workspaces/worth-query/crates/worth-query-certification/tests/application_graph/proposal_acceptance/stored_basis.rs)
already qualify their platform prerequisites. Carry that evidence unless a
change invalidates it; they do not yet prove stored proposal I/O or ingress.
Place the complete public application journey in the existing consumer-root
target, with scenarios split by responsibility inside its proposal module.
Owner tests stay with the relevant owner. Add no binary per matrix row.

Before acquisition, derive affected owners/courts through the repository's
iteration tooling and confirm actual target/selector discovery. Acquire each
changed grouped artifact once, run the first positive storage-to-adoption case,
then the affected failure families from that artifact. A setup refusal, zero
matches, ignored case or compiler failure is not the intended negative result.
Record ordinary CI/test output against the source revision; add no evidence
registry, implementation journal or tests whose only job is to certify tests.
Keep unit compilation, runtime, setup and observation failures distinguishable.

Use the pinned build configuration and existing warm targets. Repeat only
invalidated evidence after a correction; finish with required public repository
gates and the persistent qa-loop, qa-tests and code-quality-qa review. The test
reviewer must check fixture issuance, independent expected reads, intended denial
causes, ownership after each effect boundary and meaningful sensitivity to a
bypass of the disputed guard. Review can establish obvious sensitivity directly;
do not demand an exhaustive mutation campaign or resource-budget portfolio.
This phase adds no mandatory resource estimates or arbitrary work ceilings to
ordinary authoring. Private test targets and private SDK qualification are not
part of this certification run.

### QA Considerations

Authority review must trace actual grant issuance and the read/mutation/cache
ordering, rather than accept a fixture flag as revocation. Currentness review
must distinguish genuine Native/World comparisons from application counters and
must inspect replacement of the actual slot. Lifecycle review must follow staged
work, unpublished component effects, post-adoption delivery and cleanup to their
remaining owners. Test review must distinguish the intended refusal from invalid
setup and compare freshly read committed state with independently authored
expectations. Review only causally affected evidence after a correction; no
private adoption or recursive evidence machinery is needed for these claims.

### How We Know Phase 17 Is Done

- The public application records and loads a real versioned proposal, releases
  the original request and accepts through fresh ordinary authority, exact owner
  selection, expected-source creation and guarded actual-slot adoption.
- The relevant matrix families have adequate passing public evidence at their
  owning boundaries. Required race, identity, independent READ/mutation, retry,
  staged-refusal and cleanup postures are proved; no known material defect is
  hidden by a generic denial or a substitute fixture.
- Stored records contain typed intent/comparison data only. Ordinary production
  dependency checks exclude replay and internal owner access. No second
  currentness engine, proposal executor, truth map or permission registry remains.
- Current public docs show the compiled recipe, typed refusals, retry ordering
  and cleanup. Public gates and all three QA reviews pass on the final change;
  the complete phase is merged to public master. A private consumer run is not
  required and private adoption is not claimed.

### Destination And Delivery Order

`E` marks an existing owner and `C` a responsibility created by this phase.
Populate destinations only when implementing their responsibility; the tree does
not authorize empty placeholders or duplicate services.

```text
worth-query-execution/src/
  basis/product_branch/
    creation.rs                         E ordinary fork construction
    creation/expected_source.rs         C required admitted-selection handoff
    expected_description.rs             C versioned non-authoritative basis data
    expected_description/world_encoding.rs C serde encoding without World dependency
  domain_computation/primary_graph/
    product_operation/branches.rs        E application lifecycle composition
    product_operation/branches/
      expected_basis.rs                 C fresh World/Native description admission
      expected_source.rs                C candidate and selected-source custody
      expected_source/adoption.rs       C guarded application head replacement
worth-runtime-world/src/
  branch/reference_description.rs       C descriptive reference coordinates
  branch/creation/lowered.rs             E expected composite source
  branch/currentness_scope.rs           E exact source/candidate reference guards
  lifecycle/service_ports/observation.rs E admission before guarded installation
  lifecycle/owner/branch_service/
    observation.rs                      E exact stored-reference readmission
    reference_description_tests.rs      C retirement and same-name replacement
    creation/execution/forks.rs          E component-owner fork and source recheck
worth-relational/src/branch/
  owner_services/basis_port.rs           E owner basis readmission/comparison
  fork_source_validation.rs              E source generation/root/version recheck
worth-query-certification/tests/application_graph/proposal_acceptance/
  adoption.rs                           C ordinary requests and guarded adoption
  stored_basis.rs                       C serialized basis owner readmission
worth-query-certification/fixtures/consumer_entry/consumer_root/src/
  proposal_acceptance.rs                 C register the public journey modules
  proposal_acceptance/
    schema.rs                           C model, proposal and typed intent meaning
    application/
      model.rs                          C genuine incarnation and actual head slot
      storage.rs                        C ordinary proposal I/O and branch custody
      acceptance.rs                     C fresh ingress and expected-source workflow
      attempt.rs                        C retry/result-delivery continuation
    courts/
      acceptance.rs                     C record/load/accept and independent reads
      currentness.rs                    C stale source and guarded adoption races
      disclosure.rs                     C separate READ/mutation and cached retries
      identity.rs                       C retirement, recreation and foreign owners
      lifecycle.rs                      C staged effects, stops and cleanup
```

Existing owner types construct all admitted selections and transition custody.
Only audience facades export the ordinary Query workflow; consumer code does not
import World, Native or cert-only replay to perform it. The public application's
domain owns authored intent meaning and model-head selection. Cross-domain fork composition remains in Query
and World, and Native remains authoritative for its source axes. No new Native
algorithm is required where the existing receiver already proves the contract.

The application's meaning axis is its declared schema; its application children
separate model selection, storage lifecycle, acceptance control flow and attempt
delivery. Courts group by the invariant they can independently break. Application
code imports only the public facades; no owner test hook becomes a production
capability. Facades aggregate exports and contain no workflow. New parameter
families extend typed schema/codec declarations; they must not grow an untyped
dispatcher in acceptance. These destinations are populated only as their
responsibility is implemented and remain subject to the ordinary file limits.

Implement the remaining work in this dependency order, committing reviewed slices:

1. Declare the public application's model/proposal meaning and typed input
   codecs. Establish genuine model creation, actual-slot ownership and separately
   disposable storage. Prove recording does not change the authored model.
2. Complete the public record/load/fresh-accept journey, consuming the already
   qualified selection, fork and adoption path. Join fresh disclosure and
   mutation admission before retry lookup; retain ordinary attempt outcomes and
   delivery/cleanup custody before adding broad hostile coverage.
3. Add the failure families at their actual boundaries. Remove unconditional
   fork/resampling, name-only target binding, unguarded slot insertion, premature
   old-head retirement and cached-disclosure shortcuts from this application
   path. Repair shared public owners if the journey exposes a missing contract.
4. Update the compiled consumer recipe, qualify affected public courts and
   required public gates, and obtain final review of source, tests and docs.
   Merge the complete public phase. No private lane is needed to establish any
   of these claims.

The compiled public basis handoff makes the freshly admitted selection explicit.
The application must first load its proposal/model through ordinary READ admission:

```rust,ignore
let selected = application.branches().select_expected(&proposal.basis, &phase)?;
let candidate = application.branches()
    .fork_expected(selected)?
    .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
    .create(&phase)?;
// Run ordinary authored requests on candidate before guarded head replacement.
let previous = candidate.adopt(&mut model.head, &phase)?;
application.on_branch(previous).close(&phase)?;
```

### Continuing Documentation

For application authors, extend `docs/build-an-application.md` and
`docs/capability-index.md` with the compiled expected-basis workflow, independent
READ/mutation refusal, attempt retries and cleanup. Keep the ordinary request
guide authoritative for admission. Extend the branch guide's stored-basis recipe
with the real compiled public proposal example. Explain the distinct proposal,
model and result READs, independent mutation admission, new-attempt versus
same-attempt retry, actual-head adoption and post-adoption cleanup/delivery
failure. Remove guidance that equates acceptance with replay onto the latest
basis. Consumer guides teach implemented behavior only; they do not advertise a
target API before it works or contain handoff logistics, qualification counts,
milestone history or private adoption claims.

---

## Phase 18: Mandatory Current Rust Toolchain Qualification

**Status: unfinished; required. Priority: P3. Rust 1.99.x is not optional.**
**Owner:** worth-foundational + workspace toolchain/CI owners, with Proprietary
SDK qualification.
**Depends on:** the canonical sort bound and its existing compatibility contract.
Source review can run alongside Phase 13; closure includes Phase 13's honest
refusal on an unqualified compiler.

The qualified workspace and CI toolchain selects official stable Rust 1.99.1,
full compiler commit `58cf8f9c35b355199727627d07ae34a4e2846d67`, with the
existing 32/64-bit target boundary. Exact core sort, alloc buffer and compiler
specialization-owner review preserves the existing bounds and canonical version.
The [contributor toolchain contract](../../CONTRIBUTING.md#rust-toolchain-and-canonical-sort-qualification)
owns the exact compiler identities and upgrade procedure. Proprietary default
toolchain adoption and the remaining SDK qualification are unfinished; public
qualification alone does not close this phase.

### Requirements And Acceptance

Qualify an official stable Rust 1.99.x release for `canonical_verified_std_sort`.
Verify the exact published release and full compiler commit before adding its
qualification; a wildcard, version comparison, beta/nightly identity or disabled
guard cannot establish qualification. Review the corresponding standard-library
stable sort implementations, comparison/move/scratch bounds and supported target
widths against `sort_admission.rs`. Re-run canonical ordering/digest and admission
evidence: comparison counts already participate in canonical material, so any
change must follow the existing canonical version/compatibility owner rather
than silently changing identities or weakening bounds.

Update the supported workspace, CI and Proprietary development toolchain pins
to the qualified stable release, and pass the affected grouped public tests,
the correction probe and private SDK qualification. Keep supported older
qualification entries only where their compatibility commitment remains valid.
If a newer official stable Rust release exists when this phase is implemented,
qualify and adopt that release as well; 1.99 support is the required floor, not
permission to freeze the project on an already superseded default. If the
required official release is unavailable, the phase remains unfinished; remaining
on 1.98 or treating qualification as a nice-to-have cannot close the milestone.
Record the exact supported release/commit and upgrade procedure in existing
toolchain/contributor documentation, including the typed unqualified outcome.

---

## Phase 19: Consecutive Undo After Verified Source Restoration

**Status: accepted. Priority: P1.**
**Owner:** Query correction derivation, consuming existing canonical occurrence,
footprint and correction-lineage owners. **Depends on:** 9.18's recorded source
capture, ordinary correction binding, exact target ancestry and fresh admission.
Independent of managed indexes, async resources and causal inspection. Schedule
this correction fix ahead of the remaining P2/P3 findings; numbering does not
make it a last-priority task.

### Problem And Required Outcome

For one authored parameter initially 3, let A write 7 and B write 8. After a
performed reversal of B, the parameter is 7. An ordinary reversal of A must then
be applicable and restore A's captured pre-image, 3. The reported implementation
instead returns `Derivation(SourceDiverged)` because the source facts moved after
A, without recognizing the later occurrence's completed reversal. Three successive
authored writes followed by three selected reversals must similarly walk back
to the original value. This is required multi-step undo through existing ordinary
correction requests, not replay, rebasing or restoration of an old world head.

The consumer-provided reproducer is
`crates/worth-cad-entry/tests/operation_family/installed/correction/source_only_probe.rs`
on Proprietary branch `codex/undo-redo-20261027`, commit `46304f85e`. It uses one
bare setter through `ApplicationCorrectionBinding` and `request.corrections()`.
The reference is supplied evidence to obtain and qualify, not a claim that this
specification edit executed that private probe.

### Requirements And Applicability Contract

- Derive eligibility from the exact selected target's retained canonical source
  occurrences, complete semantic dependency footprints and performed correction
  lineage. An earlier source becomes applicable when every later overlapping
  occurrence that blocks its recorded support has been lawfully reversed on that
  target and the original support has been verified as restored. Include the
  restorative writes of those reversals in this lineage reasoning; they cannot
  themselves remain unexplained blockers. Append history normally and retain all
  original occurrences, reversals and identities.
- Value equality alone is insufficient. An ordinary write of the same bytes,
  a live overlapping occurrence, or a performed reapplication remains divergence
  until its own exact occurrence is reversed. Reapplication is a new occurrence;
  reversing its predecessor does not cancel it. A still-live change to another
  semantic dependency of A also retains `SourceDiverged`, even when A's written
  field happens to match its after-image. Disjoint changes remain preserved.
- Inspect/select the original B receipt, prepare/publish its ordinary reversal,
  then freshly select the original A receipt on the resulting target and
  prepare/publish A's reversal. This is the required user-visible call sequence.
  Selecting the latest correction as redo is a distinct action and cannot be
  presented as the only route for a second undo. Do not require a new application
  inverse, custom setter, private undo stack or cert-only replay entrance.
- Preserve exact entity generation, source/program compatibility, original
  pre-image/input custody, current disclosure/principal/purpose and inverse
  admission. A copied receipt, matching locator or caller assertion of restoration
  cannot supply eligibility. A sibling reversal cannot cancel a blocker in the
  selected ancestry; inherited ancestor corrections retain their real provenance.
- The eligibility decision and prepared correction remain bound to the exact
  current target. A subsequent conflicting publication must fail the existing
  stale-target check. Equivalent retries retain the original performed outcome;
  repeated reversal cannot publish a second inverse. Missing/expired evidence,
  cancelled comparison and unavailable program support keep their typed refusal
  before effects rather than being treated as successful restoration.
- Scope is reversible recorded-source applicability. Compensation, reconciliation
  and irreversible external effects keep their existing owners and effect posture;
  none is implicitly cancelled by matching local field values. Required derived
  outputs settle through ordinary demand after source publication.

### Destination And Cost

Extend the existing owner, with Query crate paths below rooted at
`workspaces/worth-query/crates/`:

```text
worth-query-execution/src/domain_computation/primary_graph/product_operation/
  correction_source/
    compatibility.rs                  E consume correction-aware source eligibility
    lineage.rs                        E exact reversal/reapplication association
    restored_support/                 N interpret eligible restored support
      overlap.rs                      N compare canonical dependency footprints
      cancellation.rs                 N resolve performed correction relationships
      eligibility.rs                  N produce the checked restoration result
worth-query-certification/tests/application_graph/
  correction/                         N consecutive-source regression scenarios
```

`E` extends existing production owners; `N` is populated by implementation, not
created as an empty placeholder. Reuse the existing correction history/index and
source-fact comparison owners; an auxiliary lookup is derived from canonical
history and must rebuild without changing eligibility. It cannot become a second
undo history or a process-local stack. Dependency direction remains from native
history/source authority into Query's correction interpretation, never the reverse.
The public facade remains `request.corrections()`.

Ordinary edits incur no new correction-history scan or eager inverse work.
Correction work follows relevant overlap and correction ancestry under existing
request/history controls; unrelated fields, bodies and branches must not cause
a whole-world scan. Exhaustion refuses honestly. Preserve original request and
storage custody while walking or rebuilding relevant correction evidence.

### Verification And Completion

Use an actual installed bare setter and genuine performed receipts through only
Decl/Host, with independently authored expected values and lineage observations:

- `3 -> A:7 -> B:8 -> reverse(B):7 -> reverse(A):3` must perform both reversals.
- `3 -> A:7 -> B:8 -> C:9`, reversed C, B, A, must produce 8, 7, 3 respectively.
- Live B refuses A. Reversing B then reapplying its retained meaning also refuses
  A; reverse that new occurrence before A can qualify. A later live equal-value
  write and a changed independent handler dependency must not be laundered into
  restoration by an after-image comparison.
- Preserve a disjoint field edit, reject sibling/counterfeit restoration, and
  preserve generation/current-authority checks. Exercise stale preparation,
  duplicate request/reversal, missing lineage and request interruption with no
  extra publication. Release/rebuild derived correction indexes and require the
  same verdicts from retained canonical history. Compare a fixed overlap chain
  beside increasing unrelated authored history without ordinary-edit tax.

Extend the existing grouped correction certification target and real private
`source_only_probe.rs`; do not add a parallel test binary or replace the setter
with a test-only bypass. Review any prior court expecting `SourceDiverged` after
verified reversal/restoration and replace only that obsolete expectation. Retain
genuine live-write, independent-dependency and reapplication divergence controls.

At implementation, revise
`workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md`,
`docs/build-an-application.md` and `docs/capability-index.md` with a compiled,
pragmatic consecutive-undo sequence, explicit source selection versus redo,
current authority and typed refusal/retry behavior. Consumer guides describe the
available implementation when it ships, not this phase's progress or future API.
This phase closes only after public qualification and the candidate-SDK private
probe pass, reviewed corrections to the superseded expectation, and required
repository checks. A plan change or a value-equality shortcut cannot close it.

### Reverse Consecutive Original Sources

Keep each original performed receipt. After A changes a source from `3` to `7`
and B changes it to `8`, reverse original B first. Once its reversal is published,
use a fresh correction request to select original A and reverse it to `3`.
Select the original receipt for each operation; a reapplication has its own new
performed receipt.

Call `request.retain_current_read()` immediately after an endpoint whose Product
history must remain readable, and keep the returned observation alive through the
later correction. Receipts alone do not retain that history. Prepare and publish
each correction through the ordinary request API, retaining the returned owner
while publication or recovery is unsettled.

Handle `SourceDiverged` when a live or reapplied overlapping write or changed
independent dependency blocks the selected source. Equal values alone do not
establish restoration. Canonical performed cancellation, complete compatible
field/aspect support, current disclosure and inverse admission still apply.
Unavailable retained evidence and unsupported shapes return typed refusals. If
publication finds a stale target, prepare again against the current target while
keeping the same client key for the same command.

Follow the [recorded-source correction guide](../../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md#reverse-consecutive-original-sources)
for the request, preparation, publication and retry sequence. The
[compiled correction example](../../workspaces/worth-query/crates/worth-query-certification/tests/application_graph/correction/consecutive_undo.rs)
shows the same calls with original receipts and retained endpoint observations.

## Documentation And Successor Handoff For The Added Phases

- Revise `docs/capability-index.md` and `docs/build-an-application.md` for
  application authors: declared async resources, actual receipt-rooted causal
  inspection, record/aspect query declarations, finite pages, fresh disclosure,
  original cleanup and the distinctions from inbound completion and corrections.
  Link compiled two-facade examples in the existing certification/example owners.
- Revise `workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md`
  with the resource, inspection and history entry/cleanup sequence. Keep
  `docs/execution/recorded-source-correction.md` within that same Query docs tree
  authoritative for inverse, pre-image and correction history; link audit
  history without conflating it.
- Revise `plans/WORTH-query/causal-inspection-dx.md` to point at the current
  application path and clearly identify any remaining platform-only audience.
  Update the Bridge API overview/README and Relational identity documentation
  where cleanup or migrated caller paths make their statements false. Do not
  create a competing status ledger or speculative feature guide.
- Retained artifacts continue to obey their existing protocol identity/version,
  classification, retention and compatibility owners. Store-backed archive,
  durable restart and multi-parent history remain successor contracts. 9.19–9.20
  consume the same query/provider admission; 9.21 can consume the exposed causal
  references without acquiring authority over their source records.
- For the retained Phases 13–14 and 16–18, revise `docs/capability-index.md` and
  `docs/build-an-application.md` for typed compiler/write refusals, storage/history
  qualification and ordinary proposal acceptance. Managed-index and candidate
  query guidance belongs to 9.19. Extend the
  managed-derived-view explanation in the application guide and
  `workspaces/worth-query/crates/worth-query/docs/modeling/schema-validation.md`
  with compiled two-facade examples. Update `CONTRIBUTING.md` with exact qualified
  Rust identities and the upgrade procedure. For phases retaining explicit
  private adoption obligations, update
  `cad/docs/plans/platform-gaps.md`, `query-9.18.1-adoption.md` beside it, and the
  existing general-modeling Acceptance 8 sections with actual adoption
  and measurements; a public platform court alone does not assert private closure.
  Phase 17's public-only certification boundary is explicit: its proposal recipe
  and failure matrix close through public Decl/Host tests and public gates.
  It requires no private documentation edit or private adoption certificate.

QA must review real receipt acquisition and request authority, exact source/product
binding, disclosure and metadata leakage, refused-progress/close custody, retained
history availability, bounded source traversal and the single-owner migration.
Compilation of an old facade, a mock provider result or a complete-looking truncated
page cannot stand in for those contracts. Batch affected grouped builds; a pure
specification change checks links, scope and path claims without compiling runtimes.

## Closing The Milestone

- 9.18.1 closes when every phase's "How We Know It's Done" list holds on master.
- Phases 1–11, 13–14 and 16–19 are required here: seventeen implementation phases,
  with numbers 12 and 15 retained solely as transfer references to required 9.19
  scope. No transferred finding is marked completed. Phases 1–6 retain their
  original acceptance
  and Proprietary consumer obligations; the additional application-runtime work
  is proved through the current two-facade public composition and grouped courts.
  No added finding is satisfied by a raw re-export, a future-work note or an old
  lower-runtime example alone. Phase 11 closes both cleanup findings in one batch.
  The retained Proprietary findings include storage qualification and
  mandatory current Rust qualification, plus P1 consecutive source undo after
  verified reversal/restoration. 9.19 owns the general index extension,
  both index readers and their final private locality cutover. Phase numbering
  does not impose serial execution:
  use each phase's actual dependencies.
- WORTH Proprietary builds against the candidate revision through a reviewed pin
  update, with its workarounds deleted.
- Platform certification alone is platform evidence. The Proprietary adoption
  gate stays open until the private build passes.
- Phase 17 is certified entirely by its public application/owner courts, public
  documentation and public gates. Its completion requires no private repository
  edit, SDK pin or private build. This exception does not waive another phase's
  expressly retained Proprietary requirement or assert private integration.
- Each implementation commit runs:
  - `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .`;
  - `cargo run --manifest-path tools/agent-context/Cargo.toml -- check`;
  - affected formatting;
  - `scripts/ci/check_workspace_rust_line_caps.sh dirty`;
  - the affected grouped certification targets, confirmed by discovery.

  A document-only revision checks links and path claims.

Not supplied here:
- typed correlated path programs (9.20);
- managed indexes, search/access products and spatial-dependent CAD locality
  (9.19), including the transferred Phase 12 and 15 findings;
- decision summaries (9.21);
- merge, rebase and multi-parent lineage (cross-runtime).
