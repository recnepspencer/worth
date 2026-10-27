# Milestone 9.18.1: Structural Graphs And Application Runtime Integration

> **Status:** Phases 1–3 accepted on 2026-10-26; Phases 4–6 remain unfinished.
> Phase 7 accepted on 2026-10-26. Phases 8–11 remain unfinished.
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
| Managed asynchronous resources stop short of the application SDK | Signal/Bridge own the lifecycle; older `worth-query/src/runtime/owned_async_{source,lifecycle}.rs` orchestrate it | Phase 8 supplies one declared, installed, governed lifecycle through Decl/Host |
| Causal inspection stops short of ordinary application receipts | Older `worth-query/src/runtime/inspection/causal/` already anchors observations, admits inspection and consumes Bridge evidence | Phase 9 connects actual application receipts and fresh disclosure to that machinery |
| Native record/aspect history has no bounded application endpoint | `worth-relational/src/history/access/aspect_history_queries.rs` selects canonical changes but collects a branch-wide scan into `Vec` | Phase 10 supplies native continuation and a disclosed record/aspect history query on an exact retained product observation |
| Stale Relational identity planning catalogs remain exported | `identity_authority/phase_one_{family_map,compile_fail_targets,root_break_targets}.rs` have no discovered callers; listed UI paths do not exist | Phase 11 deletes those three catalogs and their exports, preserving actual identity authority and tests |
| Bridge has unused duplicate facade names | `facade::{everyday,advanced,specialist}` each wildcard-re-exports the same parent | Phase 11 removes the three aliases; the standalone `worth_runtime_bridge::facade` remains supported |

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
   orbit, component, walk, lineage. No geometry, CAD or banking type enters the
   platform.
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
9. **Consumers delete their workarounds in the same phase.** A phase is not
   done while a public fixture or the Proprietary CAD entry still carries the
   hand-written version of what that phase supplies.
   For the added capabilities, a real two-facade application fixture is required;
   a domain feature need not be invented in Proprietary merely to exercise it.
   Migrated orchestration and its displaced executable path are cut over in the
   same integration phase, not deferred to Phase 11's unrelated small cleanup.

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
  Until Phase 6 lands, orbits are declared on stored relations.

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

**Depends on:** Phase 4 (aggregate identity and lineage).

### Requirements

- An application declares a component set over one or more relations, read as
  undirected coupling, for example components of a declared adjacency relation.
- Query gains the plan shape that `plans/deferred-work.md` defers under
  "Components (islands) plan shape in Query":
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

```rust,ignore
schema.components::<Shells>(&[FaceAdjacency::reference()]);

let shell = reader.component_of(Shells::reference(), &face)?;
let shell_identity = shell.identity();
let faces = shell.members();
let count = reader.component_count(Shells::reference())?;
```

---

## Phase 6: Derived Relations Readable By Rules

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
worth-query-declaration/src/application_query/
  async_resource/                                           M typed resource meaning
  history/                                                  N target/aspect selection
worth-query-installation/src/application_query/
  async_resource/                                           M installed declarations
  history/                                                  N schema/disclosure binding
worth-query-admission/src/domain_computation/
  basis_lifecycle/                                          E original scoped inspection
worth-query-execution/src/domain_computation/primary_graph/
  async_resource/                                           M live resource orchestration
  causal_inspection/                                        M application evidence binding
  application_query/history/                                N native page adapter
worth-query-publication/src/application_entry/
  resource/                                                 N governed retained requests
  inspection/                                               N receipt-rooted inspection
  query/                                                    E ordinary history query entry
worth-query/src/
  runtime/owned_async_{source,lifecycle}.rs                  M displaced orchestration
  runtime/inspection/causal/                                 M existing inspection owner
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

**Depends on:** Phase 7 and the completed application entry, provider, conditional
runtime and 9.17.7 external-effect contracts. This carries the implemented 9.4
resource machinery into the current application SDK; it does not reopen 9.4's
entire lower-runtime portfolio.

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

```rust,ignore
// Planned syntax; the retained owner delegates the existing native lifecycle.
let mut resource = request.resources()
    .start(Assessment::reference(), input)?;
let state = resource.inspect(&fresh_request)?;
let progress = resource.advance(&fresh_request)?;
// A close refusal keeps resource custody available for a later funded retry.
let closed = resource.close(&fresh_request)?;
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
    .for_observation(result.receipt())?
    .why_changed()
    .reference_only()
    .execute()?;
```

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
// Planned installed query shape; bounds are pagination limits, not fixed costs.
let page = request.at(&retained_observation)
    .query(RecordChanges::for_entity(&entity).aspect(Geometry::reference()))
    .page_size(32)
    .maximum_source_visits(256)
    .execute()?;
let next = page.continuation(); // None only when the selected history is complete.
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

QA must review real receipt acquisition and request authority, exact source/product
binding, disclosure and metadata leakage, refused-progress/close custody, retained
history availability, bounded source traversal and the single-owner migration.
Compilation of an old facade, a mock provider result or a complete-looking truncated
page cannot stand in for those contracts. Batch affected grouped builds; a pure
specification change checks links, scope and path claims without compiling runtimes.

## Closing The Milestone

- 9.18.1 closes when every phase's "How We Know It's Done" list holds on master.
- All eleven phases are required. Phases 1–6 retain their original acceptance
  and Proprietary consumer obligations; the additional application-runtime work
  is proved through the current two-facade public composition and grouped courts.
  No added finding is satisfied by a raw re-export, a future-work note or an old
  lower-runtime example alone. Phase 11 closes both cleanup findings in one batch.
- WORTH Proprietary builds against the candidate revision through a reviewed pin
  update, with its workarounds deleted.
- Platform certification alone is platform evidence. The Proprietary adoption
  gate stays open until the private build passes.
- Each implementation commit runs:
  - `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .`;
  - `cargo run --manifest-path tools/agent-context/Cargo.toml -- check`;
  - affected formatting;
  - `scripts/ci/check_workspace_rust_line_caps.sh dirty`;
  - the affected grouped certification targets, confirmed by discovery.

  A document-only revision checks links and path claims.

Not supplied here:
- typed correlated path programs (9.20);
- managed search and access products (9.19);
- decision summaries (9.21);
- merge, rebase and multi-parent lineage (cross-runtime).
