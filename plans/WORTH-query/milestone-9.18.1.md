# Milestone 9.18.1: Structural Graph Laws, Walks And Persistent Aggregate Identity

> **Status:** Phase 1 in progress. Written on 2026-10-24 against master `b56db81af6`.
> **Dependencies:** 9.17.6.3 (partitioners and exact invalidation) is complete.
> 9.18 does not block this milestone. Every phase lists its own dependencies, and
> phases with no dependency on each other may run in parallel.

## Why This Milestone Exists

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
8. **Performance is tested, not contracted.**
   - Equivalent-read comparisons, growth tests and timing are welcome.
   - Exact read counts are observations, not fixed performance expectations
     (MENTALITY section 7).
   - No signature carries cost.
   - No phase adds a work budget, a ceiling or an exhaustion matrix.
   - A walk ends when its frontier is exhausted, or on cancellation.
9. **Consumers delete their workarounds in the same phase.** A phase is not
   done while a public fixture or the Proprietary CAD entry still carries the
   hand-written version of what that phase supplies.

Code and test files stay within 400 lines. Planned syntax is marked
`rust,ignore` until it compiles; each phase turns its DX into a compiled
example.

---

## Phase 1: One Read-Count And Receipt Surface

**Depends on:** nothing.

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
- Declared queries gain a closure step on `ApplicationQueryRootPath`
  (`forward_closure`, `reverse_closure`). The receipt records the step and its
  reads.
- All of them lower onto Relational's `walk_outgoing_from` and
  `walk_incoming_from`, the traversal Query already uses to widen invariant scope.
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

### Tests That Prove It

- **Seeded differential against an independent breadth-first model** on random
  graphs with cycles, self-edges, diamonds and disconnected parts. It compares
  order, distances and membership.
- **Footprint court.**
  - An invariant that walks three hops must refuse a commit that changes only
    the third-hop edge into a violating state.
  - Mutant: register only the first hop, so the court accepts a stale verdict.
    It must die.
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
let history = reader.walk(PreviousFeature::reference(), Direction::Reverse, &tip)?;
for step in &history {
    let feature = step.entity();
    let distance = step.distance();
}
let depends = reader.reaches(PreviousFeature::reference(), &later, &earlier)?;

// Declared query: today's typed path builder, with one new step kind.
let ancestry = ApplicationQueryRootPath::from(Feature::reference())
    .reverse_closure(PreviousFeature::reference());
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

match request.lineage(&loop_identity)?.latest() {
    Lineage::Current(identity) => { /* still whole */ }
    Lineage::SplitInto(parts) | Lineage::MergedInto(parts) => { /* follow */ }
}
```

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

## Closing The Milestone

- 9.18.1 closes when every phase's "How We Know It's Done" list holds on master.
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
