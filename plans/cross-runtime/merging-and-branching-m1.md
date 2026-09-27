# Milestone 1 Engineering Spec: Semantic History Laws And Ownership

> **Status:** Planned
>
> **Roadmap parent:** [merging-and-branching-roadmap.md](merging-and-branching-roadmap.md)
>
> **Primary architectural driver:** extend the mechanically enforced composite
> branch/history authority model established by Query Milestones 9.17 and 9.18
> before complete semantic-world basis, multi-parent history, and canonical
> semantic change expand independently

## Goal

Extend the smallest mechanically enforced semantic-history foundation that lets
Milestones 2 through 4 generalize Milestone 9.17's composite runtime basis and
single-parent product branch into one complete semantic-world basis, one
canonical multi-parent commit/reference graph, and one canonical semantic-
change model without creating competing meanings or authority lanes.

Milestone 1 closes laws, ownership, typed authority roles, and migration
classification. It does not close the runtime algorithms owned by Milestones 2
through 4.

## Why This Milestone Exists

The current platform already has real and useful surfaces that use words such
as basis, branch, version, commit, change, and publication:

- Relational exposes `CommitId`, `CommitReference`, `OrderedParentList`,
  `BranchId`, `BranchHead`, `VersionNode`, `VersionGraphSnapshot`,
  `HistoryAccess`, and `HistoryAuthority`.
- Query has runtime basis resolution, `WorthQueryCommitIdentity`, preview basis
  admission, and declarative branch-comparison projections.
- Signal has branch-local execution identities, branch-basis artifacts,
  revisions, and change inputs for derived work.
- Runtime Bridge carries projected truth identities, admitted bridge
  identities, preview bases, and committed-patch envelopes across a protocol
  boundary.
- Foundational and Proof already supply shared identity categories, freshness
  postures, sealed progression machinery, and evidence vocabulary.

Those surfaces were created for their own runtime responsibilities. Similar
names therefore do not imply one semantic-history contract. For example, a
Signal branch identifies a Signal-owned definition/execution basis; it is not
automatically a Relational branch reference or product branch. Milestone 9.17's
Bridge composite commit and product reference prove exact correspondence but do
not grant either component owner's internal publication authority. A Query
branch comparison artifact is a product projection; it is not canonical
semantic diff.

Milestones 2 through 4 will make these distinctions load-bearing. If they
extend the current surfaces independently, the platform can end up with
several plausible commit identities, several meanings of basis, reference
movement without one publication authority, or Query-shaped change promoted
into truth. Retrofitting one model after those surfaces expand would be both
more expensive and less mechanically trustworthy.

Milestone 1 belongs first because it freezes and mechanically extends the
semantic border that all three following milestones cross:

```text
Milestone 1: inherited vocabulary + ownership + authority roles + enforcement
       |
       +--> Milestone 2: build and admit complete semantic-world bases
       +--> Milestone 3: build commit DAGs and governed references
       +--> Milestone 4: build canonical semantic change and diff
```

The output is useful only when it changes what code is permitted to compile and
what new surfaces are permitted to enter the architecture. A prose glossary by
itself is not the milestone.

## Governing Summaries

- `MENTALITY.md`
  protects foundation-first construction under adversarial pressure. This
  milestone therefore installs the minimum authority and enforcement substrate
  before adding history behavior, and treats bypass attempts as primary design
  evidence.
- `arch_laws.md`
  protects compiler-visible authority and typed phase progression. Semantic
  history transitions must consume concrete sealed platform authority types;
  identifiers, labels, digests, receipts, serialized records, and generic
  marker bounds cannot open governed doors.
- `composition_laws.md`
  protects one named semantic responsibility per module. World-basis
  contracts, immutable history vocabulary, reference-movement authority,
  canonical-change contracts, Query projection, and boundary enforcement
  receive distinct homes.
- `domain_structure_laws.md`
  protects truth-source and lifecycle boundaries in the filesystem. Milestone
  1 installs only the responsibility topology needed by
  `semantic_world_basis/`, `branch_graph/`, and `semantic_change/`; it does not
  create empty homes for later roadmap responsibilities.
- `perf_laws.md`
  protects cost honesty and the distinction between ordinary and
  reconstructive work. Basis admission, reference comparison/publication, and
  ancestry acceleration receive named cost contracts, while authoritative
  history and rebuildable acceleration remain visibly different artifact
  classes.
- `merging-and-branching-roadmap.md`
  protects the semantic-Git model and its build order. Milestone 1 is the
  enforceable language and ownership foundation for Milestones 2 through 4,
  rather than a miniature implementation of the rest of the roadmap.
- `physical-database-roadmap.md`
  protects the physical Store as a branch-agnostic database substrate. Its
  records may preserve semantic artifacts supplied through integration, but
  physical keys, rows, pages, and receipts do not decide semantic history.
- `physical-foundation-reconstruction-roadmap.md`
  protects the separation between semantic runtime authority and physical
  reconstruction. Destroying and rebuilding physical or derived structures
  must not change semantic identity or promote reconstructed representation
  into authority.
- `runtime-integration-roadmap.md`
  protects the Store-backed composition root: it owns an existing Query
  runtime, a sibling physical Store, and a narrow adapter between them.
  Milestone 1 therefore freezes a provider/adapter handoff law, not a
  Store-runtime prerequisite.

## Adversarial Constraint

The platform begins this milestone with adjacent, partially overlapping
history language in four runtimes. The milestone must make it impossible for a
new or retained surface to become a second semantic-history authority merely
because it carries a convincing id, digest, branch name, version, projected
change, persisted representation, or generic proof marker.

The hostile test is:

> Given two public paths that name the same apparent branch, commit, basis, or
> change, can an independent reviewer determine from types and enforced
> dependency direction which path owns truth, which path only projects or
> transports it, and which concrete authority is required to change it?

Milestone 1 fails that test if any of the following can occur:

- a caller constructs or deserializes an id-like value and thereby admits a
  semantic world, publishes a commit, or moves a reference;
- Query, Signal, Runtime Bridge, or a persisted Store representation can become
  commit/reference authority through a convenience conversion;
- parent order, reference generation, semantic-change provenance, or
  authoritative-versus-derived classification is left as commentary that
  public code can bypass;
- an existing public surface needed by Milestones 2 through 4 remains
  unclassified, so a later milestone must guess whether it is canonical,
  derived, compatibility-only, or being replaced;
- an ancestry accelerator or Query projection cannot be deleted and rebuilt
  from canonical inputs without changing authoritative identity;
- ordinary runtime code gains access to certification replay while attempting
  to prove history equivalence.

The quantitative closure rule is:

- 100 percent of the in-scope existing public surfaces are present in the
  checked inventory;
- 100 percent of governed state-changing entrypoints require a concrete sealed
  authority role;
- every derived artifact family named by this milestone has at least one
  destroy-and-rebuild parity test;
- adding another in-scope public surface makes the inventory/enforcement suite
  fail until that surface receives an explicit classification.

## Product Decision Lock

- A semantic world is the authoritative meaning observed at one admitted
  commit basis. It is not a copied map owned by a branch.
- A basis is a proof-bearing coordinate for semantic interpretation. A raw
  locator, branch label, version number, digest, or receipt may contribute
  evidence, but is not itself an admitted basis.
- A commit is an immutable semantic-history node. Its identity binds canonical
  ordered parents, the semantic-world basis, canonical authoritative change,
  and publication provenance.
- Parent order is authoritative. First parent records target-lineage meaning;
  any additional parents preserve admitted source histories in canonical
  order.
- A reference is a typed named pointer to a commit plus its mutation policy and
  generation. Reference movement is a governed compare-and-publish transition
  separate from commit creation.
- A branch is a mutable reference kind. A tag is a distinct named-reference
  kind whose mutation and retention posture is explicit rather than inferred
  from its name.
- A speculative workspace is based on an exact commit but is neither a commit
  nor a reference. Observing or preparing work against a basis grants no
  publication authority.
- A semantic change is the canonical authoritative effect associated with a
  commit. A diff is a basis-bound comparison artifact that may equal,
  aggregate, or project canonical changes only after Milestone 4 proves the
  relevant law.
- A merge base is the exact best common-ancestor basis or basis set selected
  under canonical ancestry semantics. A convenient common ancestor is not an
  equivalent substitute.
- Relational owns Relational component commits, branches, references,
  authoritative-change truth, and the truth-version component of the composite
  world basis. Definition authorities remain owners of their meaning-bearing
  components.
- Query owns declarations, audience-facing context, and projections. It
  carries admitted semantic identity without becoming its source.
- Signal owns Signal component branches and bases, definition execution,
  invalidation, and derived results. Similar branch or change terminology
  remains Signal-local unless an explicit cross-runtime binding admits it.
- Runtime Bridge owns exact component correspondence, composite runtime-world
  commits, product branch references, composite currentness, and protocol
  continuity. A Bridge projection or receipt does not grant Relational or
  Signal internal publication authority, and lower-runtime currentness cannot
  be inferred from composite identity alone.
- The physical Store remains branch-agnostic. Later Store-backed composition
  consumes semantic contracts through Query provider and adapter boundaries;
  no Store implementation milestone is a prerequisite for `SemanticClose`.
- Current surfaces may remain as compatibility projections while Milestones 2
  through 4 migrate them. Compatibility must be typed and one-way toward the
  canonical owner; it cannot become a second write path.

## Existing Surface Classification Contract

Milestone 1 produces one machine-checked inventory of existing public surfaces
that Milestones 2 through 4 will retain, wrap, or replace. The inventory is
selected by causal relevance, not by searching every occurrence of words such
as `version` or `change`.

A surface is in scope when it is public and at least one of these is true:

1. it identifies or admits truth-version basis, commit history, a named
   reference, or canonical authoritative change;
2. it moves that meaning across a crate or audience boundary used by
   Milestones 2 through 4;
3. it can observe, compare, prepare, publish, or project that meaning;
4. a later M2–M4 implementation could plausibly mistake it for the canonical
   surface.

Purely local counters, package versions, UI labels, temporal revisions, and
generic change containers enter only when a real M2–M4 boundary trace reaches
them.

### Required Inventory Fields

Each row records:

- fully qualified public path and owning crate;
- the semantic-history term it appears to represent;
- actual domain responsibility;
- authority class:
  `canonical-authority`, `derived`, `projection`, `transport`, `persisted-
  representation`, or `diagnostic`;
- migration disposition:
  `retained`, `compatibility-only`, or `scheduled-for-replacement`;
- allowed operations:
  observe, compare, prepare, admit, publish commit, or move reference;
- concrete authority or proof required for every governed operation;
- ordinary, reconstructive, or certification lane;
- destination owner and owning milestone: M2, M3, or M4;
- tests that prove its classification and prevent promotion.

`canonical-authority` means the surface is on the owning runtime's current
truth path. It does not imply that its present public shape satisfies the new
contract. For example, the existing Relational history subsystem owns current
branch-head truth, while a public data type with constructible fields may still
be scheduled for replacement before M3 can treat it as canonical history
authority.

### Required Seed Audit

The implementation audit starts from these known surfaces and follows their
public re-exports and callers:

| Subsystem | Current surfaces to trace | Initial authority reading |
|---|---|---|
| Relational | `CommitId`, `CommitReference`, `OrderedParentList`, `BranchId`, `BranchHead`, `VersionNode`, `VersionGraphSnapshot`, `HistoryAccess`, `HistoryAuthority`, commit publication, branch creation, ancestry access, canonical commit envelopes, published authoritative patch envelopes | Relational runtime state is the canonical component owner. Public ids and DTOs are not authority merely because they describe that state; each mutation or admission surface needs a migration disposition. |
| Query | `ExecutionBasisIntent`, `ResolvedSnapshotBasis`, `LowerRuntimeBasisEvidence`, `WorthQueryCommitIdentity`, `WorthQueryPreviewBasisAdmission`, `DeclarativeBranchCompareArtifact`, and the `worth-query-decl` / `worth-query-host` / `worth-query-replay` facade split | Declaration, admission orchestration, and product projection. Query may carry admitted identity; it is not commit, reference, or canonical-change truth. |
| Signal | `SignalBranchId`, `SignalSnapshotId`, `SignalBranchBasisArtifact`, `PreviousValueRevision`, and `BatchChange` where they enter a cross-runtime trace | Signal-local definition/execution basis or change input. Cross-runtime equality requires an admitted binding; name similarity carries no authority. |
| Runtime Bridge | Milestone 9.17 composite runtime-world basis, commit, product branch, reference-generation, and publication artifacts plus `TruthCommitIdentity`, `TruthBranchIdentity`, `BridgeAdmittedTruthCommitIdentity`, `BridgePreviewSessionBasis`, `BridgeSpeculativeBranchBinding`, `BridgeAsyncRequestTruthViewBasis`, and `BridgeCommittedPatchEnvelope` | Canonical composition correspondence/currentness and protocol admission over owner-issued components. Bridge admission never substitutes for Relational or Signal internal publication authority. |
| Foundational / Proof | shared identity projections, basis posture, transition outcomes, authority witnesses, and certification lane selectors used by the rows above | Vocabulary and proof substrate only. Public governed doors require concrete semantic-history roles, not generic marker acceptance. |
| Store handoff | Query provider outputs and adapter inputs that can later carry semantic basis, commits, references, or changes into physical persistence | Downstream consumption. Physical records and receipts remain representations and evidence, never semantic admission or publication authority. |

The seed table is the bounded migration map for the three immediately
following milestones.

### Checked-Inventory Rule

The inventory must be represented in a format consumed by repository tooling.
The implementation may extend the existing boundary-check configuration model
or add a narrowly owned semantic-history inventory beside it. In either case:

- CI extracts the declared in-scope public paths and verifies that they still
  exist;
- compile and boundary fixtures reference inventory row identifiers;
- a newly exported surface on an in-scope boundary fails the check until it is
  classified;
- a row cannot claim `canonical-authority` without naming its concrete
  authority gate and owner;
- a scheduled replacement cannot be used by new M2–M4 code unless the
  compatibility edge is declared explicitly.

## Destination Responsibility Topology

Milestone 1 establishes responsibility destinations, not empty subsystem
scaffolding:

```text
worth-relational
  semantic_world_basis/   truth-version component evidence
  branch_graph/           Relational component history and reference authority
  semantic_change/        canonical authoritative-change contract

worth-query
  existing legal facades  declaration, host projection, and cert-only replay routing

worth-signal
  existing branch/basis surfaces classified as Signal component authority

worth-runtime-bridge
  runtime_world/          composite-basis admission and product history/reference authority
  existing identity and envelope surfaces classified as protocol transport

worth-foundational
  only cross-crate vocabulary proven genuinely shared

worth-proof
  reusable sealed progression substrate and concrete semantic-history proof roles

tools/boundary-check
  dependency rules, public-surface inventory checks, and representative inversion fixtures
```

The Relational component destinations and Runtime Bridge composition
destination are semantic responsibilities that M2, M3, and M4 will populate.
Milestone 1 installs a destination only when it contains a real contract or
enforcement surface consumed immediately by one of those milestones. Existing
`history/`, `publication/`, `basis/`, and facade modules remain in place until
their inventory disposition authorizes a deliberate migration.

The topology preserves these directions:

```text
definition owner evidence -------\
Relational truth version --------+
Signal branch/definition basis --+--> Bridge-admitted semantic-world basis --> Query projection
policy / tenant evidence --------/

Relational commit + change --> Bridge transport --> Signal invalidation/reconciliation

Query provider --> Store adapter --> branch-agnostic physical Store
                                      |
                                      +--> persisted representation/effect evidence
```

The Store-backed path is a later consumer of the same semantic contracts. It
does not sit between Query and the semantic runtimes for ordinary execution,
and it does not participate in M1 closeout.

## Shared Crate Adoption Plan

Milestone 1 uses shared crates only where the responsibility is already shared.

### `worth-foundational`

Foundational may receive a term or identity category only when at least two
independent audience boundaries need the same meaning and neither consumer
owns semantic currentness. Suitable candidates include:

- a shared semantic-history artifact classification used by boundary evidence;
- opaque projected identity categories whose constructors remain owner-
  controlled;
- boundary evidence describing projection, transport, or compatibility
  posture.

Relational commit/reference state, basis admission decisions, and canonical
change bodies remain outside Foundational.

### `worth-proof`

Proof supplies the reusable mechanics for:

- unresolved, resolved, admitted, prepared, and publication-ready progression;
- freshness/currentness posture;
- sealed authority witnesses;
- typed success, stale, denied, rebind-required, indeterminate, and failed
  outcomes;
- joining independent component evidence into a stronger proof.

The governed public surfaces created by this milestone expose concrete roles
such as semantic-world admission authority, commit-publication authority, and
reference-movement authority. `AuthorityMarker` and `AuthorityWitness<T>` may
remain internal construction machinery; public APIs may not accept an
arbitrary caller-defined marker as equivalent authority.

### Audience Facades

- ordinary product code consumes Query through `worth-query-decl` and
  `worth-query-host`;
- certification reconstruction consumes `worth-query-replay`;
- pure schema/meaning crates remain Query-agnostic;
- Relational, Signal, and Runtime Bridge expose only their existing legal
  audience facades while M1 classifications are installed.

Any boundary configuration changes are made in
`tools/boundary-check/config/road1.toml`. Generated `AGENT_CONTEXT.md` files are
regenerated through `tools/agent-context`; they are never hand-edited.

## Phase Plan

### Phase 1: Causal Surface Inventory And Vocabulary Lock

Build the bounded migration map and freeze the core semantic-history language
before creating new public types.

**Relevant subsystems**

- Relational history, publication, ancestry, mutation, and facade surfaces
- Query basis, memory-workspace, declarative comparison, host, declaration,
  and replay facades
- Signal branch, snapshot, basis, temporal, and change-input surfaces reached
  by the M2–M4 trace
- Runtime Bridge truth identity, speculation, async basis, committed patch, and
  facade surfaces
- Foundational glossary and boundary evidence
- boundary checker canonical configuration

**Relevant APIs and source surfaces**

- `worth_relational::facade::history`
- Relational `history/`, `authority/commit/`, `publication/patch/`, and
  ancestry access implementations
- Query `basis/`, `basis_lifecycle/`, `memory_workspace/`, and
  `declarative_live` projections
- Signal `SignalBranchId`, `SignalSnapshotId`,
  `SignalBranchBasisArtifact`, and traced change inputs
- Runtime Bridge truth identities, admitted truth identities, preview bases,
  speculative bindings, and committed patch envelopes
- `cad/docs/worthy-foundations/GLOSSARY.md`
- `tools/boundary-check/config/road1.toml`

**Required work**

1. Trace each M2, M3, and M4 input backward to every current public producer,
   adapter, re-export, and consumer.
2. Add one checked inventory row per in-scope surface using the fields defined
   above.
3. Assign both authority class and migration disposition. Record uncertainty
   as an implementation blocker rather than choosing a permissive
   classification.
4. Reconcile the roadmap vocabulary with the platform glossary and add the
   locked definitions for world, basis, commit, ordered parent, reference,
   branch, tag, speculative workspace, semantic change, diff, and merge base.
5. Record same-word/different-meaning cases explicitly, especially Signal
   branches, Bridge truth identities, Query comparison artifacts, and physical
   Store representations.
6. Make the inventory part of the canonical `road1.toml` machine contract,
   either as first-class entries or as generated data whose source is declared
   there.

**Warnings**

- The inventory boundary is causal relevance to M2–M4. A lexical scan is a
  discovery aid, not the scope definition.
- Existing authority ownership and public type fitness are separate judgments.
  Preserve current truth while still marking an unsafe public shape for
  replacement.
- A compatibility row needs a named direction and expiry milestone; otherwise
  it quietly becomes a permanent second model.

**Test requirements**

- Add an inventory-completeness test that exports a new representative
  history-bearing facade item in a fixture and proves the checker fails until
  the item is classified.
- Add a semantic-collision test covering at least Relational branch identity,
  Signal branch identity, Query commit projection, and Bridge truth-commit
  projection. The test proves that equal display values do not satisfy one
  another's typed roles.
- Add a stale-row test proving renamed or removed public paths fail the
  inventory check rather than leaving dead architecture assertions.
- Add a certification-lane classification test proving Query replay surfaces
  are recognized as cert-only and ordinary facade surfaces remain distinct.

**Engineering decisions**

- `road1.toml` remains the canonical machine authority. The implementation
  extends its model rather than creating an ungoverned parallel checklist.
- The platform glossary owns the prose meaning of shared terms; domain crates
  own their concrete truth and behavior.
- Classification uses two axes because authority and migration posture answer
  different questions.
- The first inventory is intentionally limited to surfaces consumed or
  threatened by M2–M4.

**Open questions**

- None. Discovery may reveal more in-scope rows, but it does not reopen the
  selection rule or the classification vocabulary.

### Phase 2: Authority Matrix And Enforced Destination Boundaries

Turn the vocabulary into dependency direction and responsibility-shaped
destinations.

**Relevant subsystems**

- Relational semantic authority
- Query declaration, host, engine, and replay audience facades
- Signal derived runtime
- Runtime Bridge protocol transport
- Foundational and Proof law substrates
- Store-backed composition boundary
- boundary checker and generated agent context

**Relevant APIs and source surfaces**

- `HistoryAuthority` and current Relational commit/branch mutation paths
- Query facade packages declared in `road1.toml`
- Runtime Bridge facade identity and envelope constructors
- Signal facade branch/basis exports
- `worth-proof::facade`
- `worth-foundational::facade`
- `tools/boundary-check/config/road1.toml`

**Required work**

1. Encode this authority matrix in boundary configuration and facade tests:

   | Responsibility | Canonical owner | Legal consumers | Mutation/admission direction |
   |---|---|---|---|
   | truth-version component of semantic-world basis | Relational | Runtime Bridge composition; Query and Signal through admitted projections | only Relational authority admits current truth-version evidence |
   | Signal branch/definition components of semantic-world basis | Signal | Runtime Bridge composition and Query projection | only Signal authority emits current Signal basis evidence |
   | other definition/policy components of semantic-world basis | each definition or policy owner | Runtime Bridge composite-basis admission and Query projection | component owner emits evidence; Bridge admission joins it |
   | immutable composite commit and ordered parents | Runtime Bridge | Query projection, component owners through protocol, certification | only Bridge composite publication creates product semantic history; component commits remain owner-local |
   | mutable product references and update records | Runtime Bridge | Query workflow and inspection, component owners through protocol | only Bridge composition authority compares and publishes product-reference movement |
   | immutable component commits and component references | Relational or Signal owner | Runtime Bridge composition and legal projections | only the component owner creates or advances its internal history |
   | canonical authoritative change | owning runtime at commit, with Relational history binding | Query projection, Bridge transport, Signal invalidation | owning authority emits; history publication binds |
   | ancestry acceleration | Relational-derived | Relational access, Query projection | rebuilds from canonical commit graph |
   | Query comparison/basis result | Query projection | product and host audiences | never readmitted as lower-runtime authority |
   | Store-backed representation | physical Store through the composition adapter | Query provider, recovery/readmission owner | preserved/readmitted as representation and evidence |

2. Install the minimum real destination module for each immediately consumed
   component or composition responsibility. Each module exposes a narrow
   contract and a destination for its owning milestone, not a placeholder
   namespace.
3. Route all new cross-runtime use through legal facades. Pure meaning crates
   remain Query-agnostic, and certification replay remains confined to the
   cert band.
4. Add source-identifier or dependency rules for representative authority
   inversions that the existing crate graph alone cannot detect.
5. Regenerate agent context after changing the canonical machine contract.

**Warnings**

- Relational ownership of the truth-version component does not absorb
  definition authority. Composite-basis admission must preserve every
  component owner.
- Runtime Bridge transport admission alone cannot seal component currentness or
  owner-local publication. Only the distinct Milestone 9.17 composition
  authority may admit a composite basis or move a product reference.
- The Store rule constrains direction only. M1 closeout stays independent of a
  Store runtime integration milestone.
- Destination modules need one semantic responsibility each; `history_common`,
  `branch_helpers`, or a collaboration type bag would erase the boundary being
  installed.

**Test requirements**

- Add boundary-check fixtures that reject Query-to-Relational authority
  inversion, Signal publication authority, Bridge-to-Relational truth
  promotion, and semantic-runtime imports of Store physical mechanisms.
- Add facade-routing tests proving ordinary Query consumers can reach declared
  projection/context surfaces through `worth-query-decl` or
  `worth-query-host`, while ordinary crates cannot import
  `worth-query-replay`.
- Add a component-ownership test proving a composite-basis contract cannot
  treat the Relational truth-version component as evidence for definition or
  policy components.
- Add a Store-handoff dependency test proving the composition adapter may
  consume provider contracts while physical Store owner crates remain unable
  to import or define semantic branch authority.

**Engineering decisions**

- Relational is the single semantic commit/reference authority.
- Composite semantic-world authority is joined evidence, not a central crate
  claiming all component truth.
- Query is the single ordinary product-facing projection route.
- Runtime Bridge and Store adapter boundaries are typed consumers, never
  alternate semantic owners.
- Boundary enforcement changes land in the same phase as the topology they
  protect.

**Open questions**

- None.

### Phase 3: Concrete Authority Roles And Minimal Typed Progression

Create the sealed roles and stage boundaries that M2–M4 will fill with
domain-complete artifacts.

**Relevant subsystems**

- `worth-proof` transition and witness substrate
- Relational basis, commit publication, branch/reference, and mutation
  authority
- Query basis/context projection
- Runtime Bridge admitted identity projection
- compile-fail certification harnesses

**Relevant APIs and source surfaces**

- `TransitionOutcome`, `TransitionReadiness`, freshness-scoped bases, and
  checked recipe progression in `worth-proof`
- Relational `HistoryAuthority`, commit preparation/publication phases, and
  mutation workspaces
- Query `ExecutionBasisIntent`, `ResolvedSnapshotBasis`, and
  `WorthQueryPreviewBasisAdmission`
- Runtime Bridge Milestone 9.17 component correspondence, composite commit,
  product branch, and `BridgeAdmittedTruthCommitIdentity` surfaces

**Required work**

1. Define concrete sealed authority roles for:

   - semantic-world component resolution;
   - semantic-world basis admission;
   - owner-local component publication;
   - composite commit publication;
   - canonical-change preparation;
   - component-reference and product-reference comparison and movement.

2. Freeze the minimum stage grammar:

   ```text
   raw world locator
     -> owner-resolved component evidence
     -> joined compatible component evidence
     -> admitted semantic-world basis

   projected commit identities
     -> owner-observed commits
     -> basis-compatible comparable commits

   raw mutation effect
     -> owner-canonicalized change candidate
     -> prepared canonical change
     -> commit-publication input

   reference update intent + expected generation + target commit
     -> authority/policy/freshness checked update
     -> publication-ready reference movement
     -> typed publication outcome
   ```

3. Make each strengthening transition constructor-private or sealed to its
   owner. Public callers submit requests and receive typed outcomes; they do
   not receive a minting hook.
4. Reuse Proof transition machinery internally while exposing concrete
   semantic-history roles at governed public doors.
5. Define the minimum outcome vocabulary required by these skeletons:
   success, stale basis/head, denied, rebind required, indeterminate
   publication, and failed. Each outcome preserves the last trustworthy stage.
6. Bind observation and comparison separately from mutation. A caller able to
   observe a commit does not thereby gain commit-publication or
   reference-movement authority.
7. Leave component payload completeness, commit canonicalization, reference
   algorithms, and semantic-change normalization to M2, M3, and M4
   respectively.

**Warnings**

- A generic `AuthorityMarker` bound on a public governed function is still a
  caller-mintable door.
- Sealed stage names must correspond to a real invariant. Empty witness
  wrappers around unchecked payloads do not strengthen authority.
- An admitted Bridge identity remains a boundary-admitted projection. The
  Relational owner must still observe or readmit the referenced commit.
- The typed skeleton should be payload-parametric only inside Proof machinery;
  public semantic-history gates stay concrete.

**Test requirements**

- Add compile-fail cases proving external callers cannot construct semantic-
  world admission authority, commit-publication authority, prepared canonical
  change, or reference-movement authority.
- Add a forged-marker test proving a caller-defined `AuthorityMarker`
  implementation cannot satisfy any semantic-history governed API.
- Add a phase-skipping test proving raw locators, projected commit ids, raw
  changes, and reference labels cannot enter their respective publication
  stages.
- Add an observation-versus-mutation test proving commit observation and
  comparison evidence cannot move a reference or publish a commit.
- Add an outcome-preservation test proving stale, rebind-required, and
  indeterminate paths retain the last admitted basis and expected reference
  generation needed for safe retry or inspection.

**Engineering decisions**

- M1 freezes roles and progression; the owning milestone supplies the full
  semantic payload and algorithm.
- Governed public functions consume concrete sealed roles even when their
  implementation composes generic Proof primitives internally.
- Publication authority is split between commit publication and reference
  movement because immutable-node creation and mutable-pointer update are
  different acts.
- Canonical-change preparation is distinct from commit publication so M4 can
  prove change semantics independently while M3 owns history publication.

**Open questions**

- None.

### Phase 4: Canonical, Derived, Projection, And Cost Laws

Make artifact authority and cost posture explicit before M2–M4 introduce new
artifact families.

**Relevant subsystems**

- Relational commit graph, branch heads, canonical commit envelopes, published
  authoritative patches, ancestry access, and derived indexes
- Query resolved bases and branch-comparison projections
- Runtime Bridge identities and committed-patch envelopes
- Signal branch-basis and change-input surfaces
- Store adapter handoff declarations
- diagnostics and performance evidence

**Relevant APIs and source surfaces**

- `CommitReference`, `OrderedParentList`, `BranchHead`,
  `VersionGraphSnapshot`
- `CanonicalCommitEnvelope` and published authoritative patch envelopes
- Relational ancestry access and `DerivedIndexArtifacts`
- `ResolvedSnapshotBasis` and `DeclarativeBranchCompareArtifact`
- `BridgeCommittedPatchEnvelope`
- Foundational counter-backed performance receipts where a cross-boundary
  receipt is actually emitted

**Required work**

1. Classify the retained current history artifacts:

   - Relational commit graph, branch-head state, canonical commit envelope, and
     authoritative patch/effect source as canonical owner state under their
     current runtime contracts;
   - ancestry/reachability acceleration and cached summaries as derived;
   - Query basis/comparison artifacts as projections;
   - Runtime Bridge identities/envelopes as transported projections and
     protocol evidence;
   - Signal branch/basis/change artifacts as Signal-local authoritative or
     derived artifacts according to their own responsibility, with no implicit
     equivalence to semantic history;
   - Store records as persisted representations/effect evidence readmitted by
     semantic owners.

2. Require every M2–M4 artifact declaration to state:

   - canonical owner and identity basis;
   - admitted semantic-world basis;
   - source authority and producing transition;
   - mutability and lifecycle;
   - rebuildability and canonical rebuild inputs;
   - ordinary versus reconstructive access;
   - compatibility/projection direction;
   - exact counters required at its public cost boundary.

3. Establish deletion/rebuild parity for every currently retained derived
   history artifact found by Phase 1. Rebuild uses only canonical owner state
   and produces the same semantic answers and canonical digest/equivalence
   basis.
4. Establish projection parity for retained Query and Bridge surfaces:
   projecting the same admitted owner artifact twice yields equivalent
   projected identity and meaning; projections cannot be read back as owner
   authority without an owner-controlled readmission.
5. Record the required counter families for the next milestones:

   | Handoff | Exact counter families |
   |---|---|
   | M2 basis admission/comparison | components resolved, compatibility checks, identity comparisons, unavailable components |
   | M3 ancestry/reference work | commits visited, parent edges visited, derived-index hits/misses, reference compare attempts, publication coordination waits |
   | M4 semantic change/diff | candidate semantic items, normalized changes, dependency-closure items, materialization requests, projection rows |

   M1 installs the contract and evidence shape. Each owning milestone binds the
   counters to its actual algorithm and denominator.
6. Keep ordinary and reconstructive cost reports distinct. Destroy-and-rebuild
   proof may traverse all canonical history; an ordinary branch-head read or
   admitted indexed ancestry lookup cannot inherit that cost class silently.

**Warnings**

- Serialization stability and semantic identity are separate. Diagnostic or
  formatting changes cannot alter canonical meaning accidentally.
- A cached artifact that is required to recover authority is canonical state,
  regardless of a `derived` label. Classification follows rebuild law.
- Projection parity proves faithful observation, not promotion back into
  authority.
- Counter names without denominators or a decision they protect are not a cost
  contract.

**Test requirements**

- Add destroy-and-rebuild tests for every retained derived history artifact
  discovered in Phase 1. Compare authoritative answers and canonical
  equivalence, not backend bytes or self-produced diagnostics.
- Add projection parity tests for one Relational-to-Query and one
  Relational-to-Bridge path, followed by compile-fail or runtime rejection
  showing the projection cannot mint owner authority.
- Add canonical/derived drift tests that tamper with a derived artifact and
  prove owner truth remains unchanged and rebuild repairs the projection.
- Add a cost-contract fixture that rejects a new M2–M4 ordinary operation
  declaration lacking its denominator and exact structural counters.

**Engineering decisions**

- Artifact classification is determined by authority and rebuildability, not
  by durability or type name.
- Canonical identity excludes explanatory-only material unless an owning
  milestone explicitly proves that material meaning-bearing.
- Cost contract fields are part of the handoff from M1; counter
  instrumentation lands with the algorithm that creates the work.
- Store may preserve canonical semantic payloads, but its physical layout and
  receipts stay outside their semantic identity basis.

**Open questions**

- None.

### Phase 5: Hostile Enforcement Courtroom And M2–M4 Handoff

Close the milestone with mechanical evidence that each next milestone has one
legal construction path and that adjacent runtimes cannot create alternatives.

**Relevant subsystems**

- boundary checker and agent-context generator
- compile-fail certification crates
- Relational, Query, Signal, and Runtime Bridge public facades
- Store composition/provider boundary
- platform glossary and target roadmap

**Relevant APIs and source surfaces**

- all checked inventory rows
- concrete authority roles from Phase 3
- legal Query audience facades
- canonical `road1.toml`
- `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .`
- `cargo run --manifest-path tools/agent-context/Cargo.toml -- check`

**Required work**

1. Add three positive compile probes:

   - an M2-shaped owner module can resolve components and produce an admitted
     basis only through the semantic-world admission role;
   - an M3-shaped owner module can prepare commit publication and reference
     movement as separate transitions;
   - an M4-shaped owner module can prepare canonical change while remaining
     unable to publish history without the M3 authority seam.

2. Add negative probes for each adjacent runtime:

   - Query projection cannot publish component or composite commits or move
     references;
   - Signal branch/basis authority cannot satisfy Relational history roles;
   - Runtime Bridge composition authority cannot satisfy Relational or Signal
     owner-local history roles;
   - a Bridge transport receipt without composition authority cannot admit
     semantic currentness or publication;
   - deserialized or persisted representations cannot satisfy admission or
     publication roles.

3. Run representative mutation tests against the machine contract. Removing
   an owner edge, relaxing an audience rule, adding an unclassified surface,
   or exposing a sealed constructor must make the suite fail for the intended
   reason.
4. Produce a closeout inventory report containing:

   - row counts by owner, authority class, and disposition;
   - every compatibility edge and expiry milestone;
   - every scheduled replacement and destination milestone;
   - every boundary rule and its hostile fixture;
   - every compile-pass and compile-fail progression fixture;
   - rebuild and projection parity evidence;
   - generated-context parity with `road1.toml`.

5. Produce one handoff table for M2, M3, and M4:

   | Milestone | Receives from M1 | Must add before its own close |
   |---|---|---|
   | M2 | Milestone 9.17 ordinary composite runtime basis, world/basis vocabulary, component ownership, admission role, Query projection direction, cost declaration | complete semantic-world basis payload beyond the ordinary Relational-plus-Signal slice, admission/readmission algorithms, compatibility and drift semantics, basis-use evidence, exact counters |
   | M3 | Milestone 9.17 single-parent composite commit/reference substrate, ordered-parent law, separate component/composite publication roles, Bridge composition ownership, component ownership, ancestry classification | canonical multi-parent DAG, arbitrary-basis fork, compare-and-publish references, named-reference semantics, ancestry algorithms/indexes, concurrency and exact counters |
   | M4 | semantic-change/diff vocabulary, preparation role, owner/change binding, projection direction, cost declaration | canonical change families, normalization, applicability, composition, inversion where lawful, diff planning/execution, exact counters |

6. Update the roadmap only where implementation discovery changed an
   architectural fact. Preserve M1's closed boundary rather than accumulating
   later milestone planning in its closeout.

**Warnings**

- Compile-pass probes demonstrate legal construction direction; they do not
  claim that M2–M4 runtime behavior already exists.
- A boundary rule without a fixture is easy to weaken accidentally.
- A fixture that imports private implementation modules proves little about
  the actual public boundary.
- Closeout counts classify only the causal M2–M4 surface, and the report must
  state that denominator.

**Test requirements**

- Run the full boundary checker and agent-context consistency check from the
  workspace root.
- Run all new compile-pass, compile-fail, inventory, rebuild, projection, and
  mutation suites in the crates that own them.
- Add a cross-crate facade test proving the one legal observation route and
  the absence of an authority-return path from Query/Bridge projections.
- Add a clean-room handoff test in which minimal M2-, M3-, and M4-shaped
  fixtures compile against public contracts without using internal modules.

**Engineering decisions**

- `SemanticClose` is awarded on enforceable construction direction, not on
  implementation of M2–M4 behavior.
- The closeout report is evidence generated from checked inventory and tests,
  not a manually maintained second source of truth.
- Store-backed restart or persistence evidence is outside the M1 close
  denominator; the handoff direction is nevertheless enforced now.
- Any unresolved authority-classification row blocks closeout.

**Open questions**

- None.

## Must Ship

- locked semantic-history definitions for world, basis, commit, ordered
  parent, reference, branch, tag, speculative workspace, semantic change,
  diff, and merge base;
- the bounded, machine-checked M2–M4 surface inventory with authority class,
  migration disposition, owner, legal operations, proof gate, lane, and
  destination milestone;
- an enforced authority matrix covering semantic-world components, immutable
  commits, mutable references, canonical authoritative change, derived
  ancestry, Query projection, Bridge transport, Signal-local branch semantics,
  and Store-backed representation;
- the minimal real responsibility topology for `semantic_world_basis/`,
  `branch_graph/`, and `semantic_change/`;
- concrete sealed roles for semantic-world admission, commit publication,
  canonical-change preparation, and reference comparison/movement;
- typed stage and outcome grammar sufficient for M2–M4 to strengthen artifacts
  without reopening authority direction;
- boundary-check rules and hostile fixtures for representative inversions;
- compile-fail proof that ids, labels, digests, generic markers, projections,
  and persisted representations cannot mint semantic authority;
- canonical/derived rebuild and projection parity for current retained
  artifacts in the milestone denominator;
- cost-contract handoffs naming the counters and denominators M2–M4 must bind
  to their algorithms;
- positive public-contract probes and a generated closeout report handing one
  legal construction path to each of M2, M3, and M4.

## Must Preserve

- existing runtime behavior while surfaces are classified and migration seams
  are installed;
- Relational ownership of current truth and serialized publication;
- each definition authority's ownership of its meaning-bearing component;
- Signal's independent authority over derived execution state;
- Runtime Bridge's independent authority over protocol admission and
  continuity;
- Query's legal declaration, host, and certification audience split;
- Store's branch-agnostic physical model and the Store-backed composition
  root's Query-provider/adapter shape;
- canonical versus derived, ordinary versus reconstructive, and observation
  versus mutation distinctions;
- exact ordered-parent meaning already carried by Relational history;
- one-way tier direction and Query-agnostic pure meaning crates;
- compatibility for existing callers through explicit, typed, expiring
  projection paths.

## Acceptance Evidence

Milestone 1 is accepted only when the closeout bundle contains all of the
following:

1. **Vocabulary evidence**
   - platform glossary diff containing every locked term;
   - collision review showing how same-word/different-meaning surfaces remain
     distinguishable.
2. **Inventory evidence**
   - machine-readable rows for every causally in-scope public surface;
   - zero unclassified rows;
   - zero stale paths;
   - explicit denominator and trace from M2–M4 inputs.
3. **Authority evidence**
   - owner matrix encoded in boundary rules;
   - one positive public route per governed role;
   - negative fixtures for Query, Signal, Bridge, persisted-representation,
     replay-lane, and generic-marker promotion.
4. **Progression evidence**
   - compile-pass fixtures for M2-, M3-, and M4-shaped consumers;
   - compile-fail fixtures for caller construction, phase skipping,
     observation-to-mutation promotion, and commit/reference authority
     conflation.
5. **Artifact evidence**
   - destroy-and-rebuild parity for each retained derived history artifact in
     the inventory;
   - projection parity plus failed reverse promotion for Query and Bridge.
6. **Cost evidence**
   - M2–M4 handoff declarations naming exact structural counters,
     denominators, and ordinary/reconstructive class;
   - no new M1-owned uninstrumented hot path.
7. **Machine-contract evidence**
   - passing boundary checker;
   - passing agent-context consistency check;
   - passing relevant crate tests and certification fixtures;
   - mutation tests proving representative rule removal is detected.

Test output or generated evidence must identify the inventory row, authority
role, or boundary rule it proves. A passing workspace test count without that
traceability is insufficient.

## Sequencing Notes

1. Phase 1 lands first because every later edit depends on the current-surface
   denominator and locked vocabulary.
2. Phase 2 follows with dependency direction and destination ownership. New
   semantic-history public types begin only after these checks are active.
3. Phase 3 installs concrete authority roles and typed progression against the
   enforced owner map.
4. Phase 4 classifies retained artifacts, proves rebuild/projection law, and
   records the cost contract consumed by the next milestones.
5. Phase 5 runs the hostile courtroom and emits the M2–M4 handoff.
6. M2 may begin after the semantic-world ownership, admission role, and
   inventory rows it consumes are closed.
7. M3 may begin after commit/reference ownership, separate publication roles,
   and ordered-parent law are closed.
8. M4 may begin after canonical-change preparation, owner binding, and
   projection law are closed.
9. Parallel M2–M4 implementation is safe only across already-closed M1 seams;
   any required authority change returns to M1 as an explicit contract
   amendment.
10. Store-backed integration consumes the contracts after their semantic owner
    ships them. Its schedule does not gate M1 `SemanticClose`.

## Closeout Gate

Earn `SemanticClose` when:

- M2–M4 have one vocabulary and one legal owner path for every foundation they
  consume;
- the complete causal surface denominator is classified and checked;
- concrete sealed roles prevent caller-minted admission, commit publication,
  canonical-change preparation, and reference movement;
- representative authority inversions fail mechanically;
- retained derived and projection artifacts satisfy rebuild/parity law;
- the canonical machine contract and generated agent context agree; and
- all acceptance evidence is traceable to a checked inventory row, authority
  role, or boundary rule.

Documentation agreement, empty destination modules, or compile-only marker
types do not earn closeout. Store-backed restart and physical durability earn
their own later integration closure without changing the meaning of this
milestone.
