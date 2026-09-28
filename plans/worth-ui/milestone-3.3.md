# Milestone 3.3 Engineering Spec: UI Authority Graph, Identity, Participation, And Core Indexes

> **Status:** Planned
>
> **Roadmap parent:** [worth_ui_roadmap.md](./worth_ui_roadmap.md)
>
> **Primary prerequisite:** `Milestone 3.2 Canonical Declaration Artifacts And Aspect Contracts`
>
> **Follow-on sequence:** `Milestone 3.4 Admission, Support, And Graph Touch Obligations`
>
> **Primary architectural driver:** make runtime UI existence, topology,
> participation, and bounded lookup derive from one runtime-owned graph
> authority rather than declaration tree position, renderer-local structure, or
> ad hoc traversal folklore.

## Goal

Make Worth UI runtime graph truth explicit, stable, and bounded.

Milestone 3.3 is complete when Worth UI can admit runtime graph nodes from
sealed declaration handoff artifacts, assign stable runtime-owned identities,
represent topology and participation as explicit graph authority, maintain the
required index set transactionally with graph mutation, and answer ordinary
runtime lookups without recursive tree walks, source reopening, or renderer
re-interpretation.

This milestone closes one authority boundary:

- what runtime nodes exist
- which declaration artifact each node instantiates
- which parent, slot, page, region, and mosaic relationships apply
- which participation axes are admitted for each node
- which graph-owned attachments already belong to the node even when their
  richer runtime lanes land later
- which mounted-receipt authority seeds exist
- which core lookup indexes are authoritative for ordinary graph access

It does not close broader runtime execution, host observation, or touched-work
selection.

## Non-Goals

Milestone 3.3 does not implement:

- obligation selection or admission planning
- Query execution
- service execution
- measurement execution
- host observation collection
- hit-test algorithms
- focus algorithms
- accessibility adapter output
- visual layout solving
- inspector panel UI
- replay artifacts
- cross-world reconciliation policy beyond graph instantiation boundaries
- broad diagnostic storytelling beyond graph authority evidence

3.3 closes graph truth and bounded lookup only:

- graph node identity
- runtime node existence
- declaration-instance correspondence
- parent-child-slot topology
- page-region-mosaic membership
- explicit participation posture
- stable repeated-instance identity
- graph-owned attachment posture for future Query, service, and diagnostic
  lanes
- mounted receipt authority seed storage
- transactionally aligned core indexes

## Why This Sequence Exists

Milestone 3.1 created the public/runtime boundary and certification topology.
Milestone 3.2 created canonical declaration authority and the sealed graph
handoff surface. Milestone 3.3 is the next load-bearing slice: once authored
meaning is canonical, the runtime must decide what actually exists at runtime
and how later phases find it without rediscovering topology by walking
declaration trees or renderer-owned structures.

This is not “a tree representation of declarations.” It is runtime authority
for node existence and bounded lookup:

- 3.4 needs graph-owned node identity, participation posture, and declaration
  correspondence before it can select touched obligations honestly.
- 3.5 needs graph-owned evidence lanes before inspection can explain what the
  runtime instantiated and why.
- later host, layout, input, and accessibility work need explicit participation
  axes instead of inferring them from containment or visibility folklore.

If 3.3 stays thin, later milestones will recover runtime structure through tree
position, recursive walks, renderer caches, or declaration-family-local
conventions. That would create folklore instead of architecture.

## Governing Summaries

- `MENTALITY.md`
  protects adversarial-constraint-first design. 3.3 must start from graph
  churn, repeated instances, partial participation, and bounded lookup under
  growth, not from a friendly one-page tree demo.
- `arch_laws.md`
  protects one canonical authority per boundary, proof-bearing progression, and
  explicit source precedence. Declaration authority remains upstream; graph
  truth becomes the one downstream runtime authority for node existence and
  participation.
- `composition_laws.md`
  protects named semantic steps. 3.3 must not collapse graph instantiation,
  identity assignment, topology mutation, participation mutation, index repair,
  mounted receipt handling, and closeout proof into one facade or one graph
  manager.
- `domain_structure_laws.md`
  protects separable topology. Identity, graph mutation, participation, mounted
  receipts, indexes, handoff, and certification must live in distinct homes
  because they evolve, fail, and scale differently.
- `perf_laws.md`
  protects bounded breadth and honest cost. Ordinary graph access must use
  runtime-owned indexes, not recursive graph walks or repeated declaration
  recovery.
- `worth_ui_roadmap.md`
  requires `UiGraphNodeIdentity`, `UiGraphSnapshot`, parent/child/slot
  topology, page/region/mosaic membership, explicit participation posture,
  declaration-instance correspondence, stable repeated-instance identity, and
  the initial core index set with transactional alignment.
- `WORTH_UI_README.md`
  requires the graph to remain the authority for runtime structure,
  participation, graph-owned attachments, and future bounded rebind rather
  than letting Query bindings, services, diagnostics, or host adapters create
  side topologies later.
- `worth-ui-dsl-vision.md`
  requires DSL-authored lanes such as participation, query-binding, intent,
  services, diagnostics, motion, and operability to lower into separable graph-
  consumable runtime facts rather than one component-local blob.
- `ai-diagnostics.md`
  requires every serious runtime family to ship typed evidence, targeted
  inspection entry points, relevance hooks, and replay-safe stop surfaces
  alongside the runtime authority it introduces.
- `worth-query/docs/AI_README.md`
  requires Worth UI to consume Query through Query-owned public lanes rather
  than a UI-local pseudo runtime. For 3.3 that especially means:
  - Query basis and world posture come from Query-owned basis artifacts such as
    `ResolvedSnapshotBasis` and `SnapshotResolutionReport`
  - projected runtime data comes from Query projection-consumption APIs such as
    `WORTHQueryReadResult::consume_projection_facts(...)`,
    `WORTHQueryWriteReceipt::consume_projection_facts(...)`, and
    `QueryContextExecutionArtifact::consume_projection_facts(...)`
  - retained Query evidence inspects through `workspace.inspect(...)`
  - cross-runtime causal explanation stays on Query's
    `admit_causal_inspection` / `request_causal_inspection` lane
  - preview and branch world identity stay on typed Query artifacts such as
    `WORTHQuerySessionLabel` and `BridgePreviewSessionIdentity`, never raw
    strings

## Adversarial Constraint

3.3 must survive this hostile condition:

> Worth UI instantiates large page graphs with repeated declaration families,
> nested regions, mosaic composition, multiple participation axes, mounted
> receipts, and aspect publication or consumption edges. Edits, admissions,
> removals, repeated-instance reshaping, and host-facing participation changes
> occur repeatedly. Across that churn, the runtime must derive one deterministic
> graph authority from sealed declaration handoff plus runtime instantiation
> facts,
> assign stable identities that do not depend on tree position, keep graph and
> index mutation transactionally aligned, and answer ordinary lookups through
> bounded indexes rather than recursive walks, declaration reopening, or
> renderer-owned caches.

If a node can only be found by walking the tree, if repeated-instance identity
depends on sibling position noise, if participation is implied instead of
stored, if index mutation can lag committed graph mutation at all, if mounted
receipts exist outside graph authority, or if aspect
publishers and consumers must be rediscovered by reinterpreting declarations,
3.3 is not closed.

## Product Decision Lock

- `UiGraphSnapshot` is runtime graph authority, not a convenience projection.
- Declaration artifacts remain upstream authority for authored meaning; they do
  not become runtime node truth by themselves.
- `UiGraphNodeIdentity` is runtime instance identity, not declaration identity,
  tree position, or renderer path.
- Repeated-instance identity must be stable on an admitted equivalence basis;
  sibling index alone is illegal as identity.
- Participation is multi-axis and explicit. It may not be inferred from one
  coarse mounted or visible flag.
- Core indexes are authoritative runtime structures, not best-effort caches.
- Graph mutation and core-index mutation must share one transaction boundary.
- Graph-owned attachments such as Query binding attachment, service
  attachment, and diagnostic attachment must have one authoritative home at the
  graph boundary before richer runtime behavior broadens.
- Query-owned support, basis, projection-consumption, preview, and inspection
  APIs remain Query-owned. Worth UI may attach graph truth to those artifacts
  through `worth-ui-query-binding`; it may not redefine them.
- Mounted receipts are graph-owned runtime evidence, not host-owned side tables.
- 3.3 may expose graph evidence and closeout reports, but it may not solve 3.4
  obligation selection or 3.5 inspection breadth by hiding them in graph blobs.

## Runtime Authority Progression

3.3 must preserve explicit phase progression:

`DSL source -> parsed source AST -> UiDslSemanticArtifact -> UiDeclarationArtifact -> UiDeclarationGraphHandoff -> UiGraphInstantiationPlan -> UiGraphMutation -> UiGraphSnapshot`

The exact type names may vary, but the phase law may not:

- source text is not graph authority
- parser output is not graph authority
- declaration artifacts are not graph authority
- graph handoff is the sealed declaration input to graph instantiation
- instantiation planning resolves graph-instantiation facts before mutation
- mutation produces one coherent graph snapshot and index state

If production code can skip from declarations directly into ad hoc graph edits
without an admitted graph instantiation planning or mutation boundary, the
phase progression has collapsed.

`UiGraphSnapshot` is the committed graph authority artifact for this milestone.
Its lifecycle must be explicit:

- `UiGraphSnapshot` is immutable once committed
- every committed graph snapshot has a `UiGraphGeneration`
- every graph mutation consumes exactly one base generation and either denies
  or publishes one new coherent generation
- inspection may retain old generations by evidence handle
- ordinary graph authority points to one current generation per world profile

This allows replay, hot reload, and inspection to reason about graph authority
without mutating snapshots in place.

## Graph Identity Contract

3.3 must make runtime identity precise.

The graph identity model must distinguish at least:

- `UiDeclarationIdentity`
  authored-semantic identity from 3.2
- `UiGraphNodeIdentity`
  runtime identity of one admitted graph node
- repeated-instance identity basis
  the stable basis that distinguishes multiple runtime instances admitted from
  one declaration authority surface
- `UiMountedReceiptIdentity`
  runtime identity of a mounted receipt owned by the graph

Identity rules:

- `UiGraphNodeIdentity` must never be derived from current tree position.
- a graph node identity must survive unrelated sibling insertion or removal
  unless the admitted repeated-instance equivalence basis itself changed
- declaration identity and graph node identity must remain separate even when a
  declaration currently admits exactly one node
- repeated instances must admit a stable instance basis before mutation; if the
  basis is missing or contradictory, graph instantiation must deny instead of
  minting position-based identity
- mounted receipt identity must not be synthesized from opaque host handles
  alone

3.3 must also force the repeated-instance basis itself to be typed and
classifiable. The runtime may choose exact names, but it must admit one sealed
repeated-instance basis contract surface that can distinguish at least:

- declaration-keyed repeated-instance basis
- runtime-data-keyed repeated-instance basis
- denied or unavailable repeated-instance basis

If a repeated-instance basis comes from runtime data rather than declaration
authority, that basis must arrive through an admitted runtime boundary,
canonicalize before graph mutation, and become part of the graph instantiation
proof. The graph may not accept an unclassified "some runtime key" identity
basis.

If that runtime data comes from Query, the ordinary path is typed
projection-consumption or retained Query evidence, not raw materialization rows,
lower-runtime internals, or caller-owned string digests.

## Declaration To Runtime Correspondence Law

3.3 must make declaration-instance correspondence explicit and queryable.

The graph authority must answer these questions without tree walks:

- which graph node or nodes currently instantiate a given declaration identity
- which declaration identity owns a given graph node
- whether a declaration currently admits zero, one, or multiple graph nodes
- which repeated-instance basis distinguishes the multiple nodes when they
  exist

This correspondence is not incidental metadata. It is a core graph law because
3.4 and 3.5 need runtime authority to explain what declaration meaning became
instantiated, suppressed, repeated, or mounted.

## Topology Contract

3.3 must represent topology as graph authority, not declaration leftovers.

The graph topology model must encode:

- parent identity
- child set
- slot identity
- slot occupant set
- page membership
- region membership
- mosaic membership
- any admitted authored ordering guarantee from 3.2 where ordering is
  semantically meaningful
- graph-owned attachment posture for any declaration-consumable lane that must
  later bind through graph truth rather than a side registry

Topology rules:

- parent-child topology is runtime graph truth, not a re-read declaration tree
- slot occupancy must be explicit when slot semantics exist
- page, region, and mosaic membership must be graph-owned facts rather than
  family-local tags
- graph topology may consume declaration structural intent from 3.2, but must
  not reopen source or reinterpret family syntax
- ordinary topology queries must use authoritative indexes, not recursive
  descent

## Participation Contract

3.3 must make runtime participation explicit on separate axes.

The required participation axes are:

- exists
- mounted
- visible
- layout
- hit-test
- focus
- accessibility
- paint
- input
- query-bound
- service-bound
- diagnostic

Participation rules:

- each axis must be representable independently
- one axis may depend on another by law, but dependency must be encoded as
  explicit admission or denial logic rather than assumed by readers
- participation posture must live on graph authority, not be recomputed from
  host residue or renderer-local heuristics
- a node may exist in the graph while being non-participating on some or all
  axes
- diagnostic participation must stay explicit so inspection and certification
  do not have to infer graph visibility from unrelated runtime state
- each participation posture must carry a reason or source classification
  sufficient to distinguish declaration-driven, graph-instantiation-driven,
  world-driven, host-capability-driven, service-deferred, diagnostic, and
  architecturally-owned-but-not-yet-admitted posture

3.3 does not need to solve all future participation policy, but it must create
the storage and mutation boundary that prevents those policies from becoming
implicit folklore later.

The exact type names may vary, but the participation posture contract must be
strong enough to carry at least:

- axis
- status
- source or reason classification
- evidence ref or equivalent evidence handle

When a participation reason comes from Query-backed world or basis posture, the
evidence ref must point back to Query-owned evidence rather than a UI-local
reinterpretation. Use Query basis and inspection artifacts, not copied labels
or booleans.

The roadmap's `page identity -> participating node set` index must not collapse
these axes back into one ambiguous membership answer. 3.3 therefore requires:

- one authoritative page-participation lookup surface
- node-level participation posture available from that lookup path without
  recursive recovery
- enough returned structure to distinguish which participation axes caused a
  node to appear or not appear

The exact API shape may vary, but a caller must not need to infer axis-specific
participation by separately walking descendants after page lookup.

## Graph Attachment Contract

3.3 must reserve explicit graph-owned attachment posture for lanes that the
broader runtime already treats as graph-scoped, even when their richer
execution families land in later milestones.

At minimum, the graph must have one explicit authoritative home for:

- query binding attachment posture
- service attachment posture
- diagnostic attachment posture

This milestone does not need to implement full Query binding runtime, portal or
focus or motion services, or mounted diagnostic surfaces. It does need to make
sure those lanes will attach to graph truth rather than inventing side tables
or host-local topology later.

For the Query lane specifically, 3.3 must not invent UI-local substitutes for
Query-owned public APIs. The graph may record Query-binding attachment posture,
but Query-owned meaning still comes from Query surfaces such as:

- `ResolvedSnapshotBasis`
- `SnapshotResolutionReport`
- `WORTHQueryReadResult::consume_projection_facts(...)`
- `WORTHQueryWriteReceipt::consume_projection_facts(...)`
- `QueryContextExecutionArtifact::consume_projection_facts(...)`
- `workspace.inspect(...)`
- `admit_causal_inspection`
- `request_causal_inspection`

The Worth UI side may attach graph nodes to those Query-owned artifacts through
`worth-ui-query-binding`; it may not replace them with UI-local basis labels,
support flags, projection bags, or explanation helpers.

The exact types may vary, but 3.3 must make these distinctions mechanically
possible:

- attached
- unattached
- not_applicable
- architecturally_owned_but_not_yet_admitted

If a later lane would need to answer "which graph node owns this binding or
service or diagnostic surface?" and 3.3 has no explicit graph-owned home for
that answer, the graph boundary is too thin.

The attachment posture must also carry a typed lane identity. The exact names
may vary, but the contract must be strong enough to admit:

- `UiGraphAttachmentLane`
  - `query_binding`
  - `service`
  - `diagnostic`
- `UiGraphAttachmentPosture`
  - lane
  - owner node
  - declaration identity
  - status
  - evidence ref

This prevents "service attachment" from becoming too broad once portal, focus,
motion, command-routing, and scroll lanes arrive.

For `query_binding`, the attachment posture records graph ownership and
attachment status only. It does not replace Query binding request, basis,
projection-consumption, support, or inspection artifacts.

## Mounted Receipt Scope Guard

Mounted receipt authority in 3.3 is a graph-owned seed or placeholder
authority, not the full later mounting layer.

This milestone must admit:

- one graph-owned mounted receipt identity surface
- mounted receipt authority seed storage under graph authority
- `mounted receipt identity -> mounted receipt` lookup
- explicit correspondence between mounted receipts and graph nodes

Mounted receipt rules:

- mounted receipt existence must be queryable without host-specific traversal
- mounted receipt mutation must align transactionally with graph mutation
  whenever graph truth changes the mounted posture
- mounted receipts may carry host-linked evidence, but host-linked evidence may
  not replace graph ownership of receipt identity or correspondence
- 3.3 owns mounted receipt identity, graph correspondence, mounted posture
  relationship, and graph-owned evidence slots only
- 3.3 does not produce mounted frame receipts, paint intent, allocation boxes,
  visual geometry, host-consumable output, or layout-derived mounting

This keeps the graph boundary correct without pulling the later mounting and
host-output runtime into 3.3.

## Graph Instantiation Admission Scope

Graph instantiation in 3.3 means admission to instantiate graph truth from the
sealed 3.2 declaration handoff.

3.3 graph instantiation may decide whether the handoff can instantiate:

- graph nodes
- topology
- participation posture
- graph-owned attachments
- mounted receipt authority seeds
- core index contributions

3.3 graph instantiation does not select:

- graph touch obligations
- runtime support admission
- Query binding admission
- service admission
- measurement admission
- intent operability

Those remain outside 3.3 and must not be partially smuggled into graph
instantiation naming or helper logic.

For Query-backed lanes, that means 3.3 may admit graph-side instantiation only.
It may not pretend to own Query support or admission posture that belongs to
Query's public support and inspection surfaces such as `support_report()` and
its support snapshot family.

## Graph Mutation Vocabulary

3.3 must make graph mutation concrete enough that "transactionally aligned"
becomes an operational rule rather than an aspiration.

The initial admitted mutation vocabulary must include only graph-owned verbs
such as:

- `instantiate_node`
- `remove_node`
- `replace_node_identity_preserving`
- `move_node`
- `attach_to_slot`
- `detach_from_slot`
- `update_membership`
- `update_participation`
- `attach_query_binding_posture`
- `attach_service_posture`
- `attach_diagnostic_posture`
- `create_mounted_receipt_slot`
- `remove_mounted_receipt_slot`

All graph mutation must occur through admitted graph mutation plans.

Forbidden ordinary paths:

- direct node-map update
- direct index update
- direct membership update
- direct mounted-receipt authority-seed update

The exact internal implementation may vary, but public and certifiable graph
truth must only move through these admitted mutation shapes or narrower
equivalents.

## Graph And Index Transaction Law

3.3 must turn “the indexes match the graph” into architecture.

The implementation may choose internal transaction mechanics, but it must
preserve these laws:

1. graph mutation and core-index mutation share one admitted mutation boundary
2. either a mutation commits both graph and index effects coherently, or the
   mutation denies and publishes no new graph snapshot
3. no ordinary caller may observe a committed graph snapshot paired with stale
   core indexes
4. if the runtime uses staging or repair structures internally, those
   structures must stay behind the graph authority boundary and must not become
   a second public truth model

This milestone may admit internal mutation planning artifacts, but it may not
admit public best-effort indexes, a public "mostly committed" graph state, or a
public repair-required graph snapshot.

## Core Index Set Lock

The roadmap index set is mandatory. 3.3 must ship authoritative structures for:

- `graph node identity -> node`
- `declaration identity -> graph node(s)`
- `parent identity -> child set`
- `slot identity -> occupant set`
- `page identity -> participating node set`
- `published aspect -> publishing node/receipt set`
- `consumed aspect -> dependent node/receipt set`
- `mounted receipt identity -> mounted receipt`

Index rules:

- these indexes are part of graph authority, not optional caches
- lookup cost and maintenance cost must be explicit at the API boundary
- aspect indexes must consume 3.2 declaration aspect contracts plus runtime
  correspondence rather than rediscovering publishers and consumers from source
- page identity participation lookup must respect explicit participation posture
  rather than returning all descendants by containment, and it must preserve
  enough axis-distinguishing posture that later phases do not collapse
  participation back into one undifferentiated set
- declaration identity to graph nodes must support zero, one, or many nodes
  honestly

## Deferred Index Families

3.3 ships the mandatory core graph indexes above. It must also name the next
deferred index families so their temporary absence cannot justify side
topologies.

Deferred beyond 3.3:

- portal owner -> portal attachment set
- focus scope -> participant set
- query binding identity -> bound node set
- runtime service attachment -> attached node set
- consumed fact identity -> dependent graph node or receipt set
- repeated owner + logical member identity -> runtime instance node
- host observation target -> affected graph node or receipt set

3.3 does not need to implement these indexes yet. It does need to preserve the
graph-owned attachment posture and identity surfaces they will later depend on.

## Aspect Index Correspondence Law

3.3 must lock how aspect indexes relate to 3.2 contracts.

The graph may not decide aspect semantics independently. Instead:

- declaration authority from 3.2 defines what a declaration may publish or
  consume
- graph authority from 3.3 defines which runtime nodes currently instantiate
  those declaration contracts
- aspect indexes combine those two truths into bounded runtime lookup surfaces

This prevents three failure modes:

- declaration truth being reinterpreted per renderer or per host
- graph truth being recovered through declaration scans
- aspect lookup turning into broad tree traversal

3.3 must also preserve enough graph-side attachment posture that later Query
binding, service, and diagnostic evidence can attach to aspect-bearing graph
nodes without rebuilding a second topology keyed by convenience ids.

For Query-backed aspects, the graph must attach to Query-produced fact and
inspection lanes honestly. It should consume typed Query fact receipts and
retained evidence, not rebuild memberships or basis posture from raw rows,
source strings, or lower-runtime details.

Aspect indexes are many-to-many. The contract must allow:

- one node publishing multiple aspects
- one aspect published by multiple nodes
- one node consuming multiple aspects
- one aspect consumed by multiple nodes
- one node consuming an aspect it also publishes

Index entries must also classify publisher and consumer kind strongly enough to
distinguish:

- publisher kind
  - `graph_node`
  - `mounted_receipt_slot`
  - `future_receipt`
- consumer kind
  - `graph_node`
  - `attachment_posture`
  - `future_obligation`
  - `future_receipt`

Unsupported or not-yet-admitted aspect lanes must remain explicit posture,
never absent folklore that downstream lanes reinterpret inconsistently.

## Index Cost Honesty Law

Every graph access API must reveal whether it is:

- identity lookup
- declaration correspondence lookup
- topology lookup
- page participation lookup
- aspect publisher lookup
- aspect consumer lookup
- mounted receipt lookup

Cheap-looking APIs may not conceal:

- recursive tree walks
- broad scans of all nodes
- source reopening
- declaration contract reconstruction
- host callback enumeration

If an API looks scalar but internally performs breadth-coupled traversal, the
surface is dishonest and violates 3.3.

The lookup contract should be proof-bearing enough that tests and inspection
can assert no silent traversal occurred. The exact names may vary, but ordinary
lookup should be able to expose a receipt shape equivalent to:

- `UiGraphLookupReceipt`
  - `lookup_family`
  - `index_identity`
  - `base_generation`
  - `result_count`
  - `cost_class`
  - `evidence_ref`

with cost classes equivalent to:

- `scalar_index`
- `small_index_set`
- `bounded_index_range`
- `denied_unindexed`
- `unsupported`

When a lookup result depends on Query-owned context, the graph lookup receipt
must still distinguish "graph lookup was bounded" from "Query meaning was
inspected elsewhere." It must not blur graph lookup proof with Query support or
projection-consumption proof.

## Graph Evidence And Inspection Contract

The broader Worth UI architecture does not allow runtime milestones to postpone
explainability until the end. 3.3 therefore must ship the minimum graph-facing
inspection and evidence surfaces needed to explain graph truth honestly.

At minimum, 3.3 must provide:

- graph inspection target support for graph node identity and graph topology
- graph evidence refs or equivalent graph evidence handles
- graph-scoped relevance routing for declaration identity, graph node identity,
  and aspect-local graph questions
- replay-safe graph stop-point readiness for:
  - after graph instantiation
  - after graph mutation
  - after index alignment

Minimum graph questions 3.3 must support through the formal inspection
architecture:

- `inspect_graph_node(node_id)`
- `inspect_declaration_instances(declaration_id)`
- `inspect_parent_child(parent_id)`
- `inspect_slot_occupants(slot_id)`
- `inspect_page_participation(page_id, axes)`
- `inspect_aspect_publishers(aspect_id)`
- `inspect_aspect_consumers(aspect_id)`
- `inspect_mounted_receipt_slot(receipt_id)`

Each answer must be able to return:

- target
- base graph generation
- lookup receipt
- evidence refs
- unsupported or denied posture when the requested lane is not admitted

If the answer crosses into Query-owned meaning, the graph inspection surface
must link out to Query-owned evidence lanes instead of replaying Query support,
basis, or projection semantics locally. In particular:

- per-target retained Query evidence remains on `workspace.inspect(...)`
- cross-runtime "why" remains on `admit_causal_inspection` /
  `request_causal_inspection`
- projection-backed runtime facts remain on `consume_projection_facts(...)`

This does not require the full later inspector UI or full replay subsystem. It
does require enough formal support that graph truth is inspectable through the
same public inspection architecture rather than logs or renderer-local helpers.

3.3 graph inspection must stay thin. It must prove:

- graph truth can be targeted through the formal inspection architecture
- scoped evidence refs can be returned
- broad dumps and log-scraping are unnecessary
- future replay can stop at graph-relevant boundaries

It does not need to explain every later UI bug or provide full runtime
storytelling.

## Repeated-Instance Stability Law

3.3 must make repeated-instance identity a first-class architectural question.

Allowed stability bases include only admitted, proof-bearing runtime or
declaration-derived facts such as:

- explicit authored repetition keys admitted by declaration contracts
- stable data identity supplied through an admitted runtime boundary
- sealed instance-basis artifacts produced during graph instantiation planning

Disallowed bases include:

- raw sibling index
- current tree walk order when that order is not semantically admitted
- opaque renderer object address
- host pointer identity
- incidental allocation order

If a repeated-instance family needs richer basis support than 3.3 can admit
honestly, the runtime must deny or classify it explicitly rather than smuggling
position identity through the graph.

Equivalent repeated-instance basis inputs must converge to equivalent graph
identity outcomes. A basis representation change that preserves the same
semantic instance set must not churn runtime identities. If convergence depends
on caller-specific ordering or host residue, the basis contract is too weak for
admission.

Operational examples:

Allowed:

- authored key such as `field("title")`
- runtime data key such as `user.id` from an admitted runtime boundary
- declaration-keyed singleton basis for one-instance declarations

Denied:

- repeat without key
- repeat keyed by row index
- repeat keyed by display label
- repeat keyed by source span
- repeat keyed by host handle
- repeat keyed by allocation order

Required stability proof shape:

- given rows `[A, B, C]`, insert `X` before `B`
- `A`, `B`, and `C` keep graph identity
- `X` receives new identity
- no identity is derived from sibling index

## World Profile Contract

3.3 must not let world-sensitive graph instantiation become hidden topology drift.

The runtime may choose exact type names, but it must make the current graph
world basis explicit enough to answer:

- which runtime world or host profile this graph snapshot belongs to
- whether a graph instantiation or participation difference came from declaration
  meaning, world facts, or host evidence
- whether two snapshots are same-world comparable, different-world comparable,
  or not comparable
- whether two snapshots are comparable as the same world authority or as
  different world authorities

World-sensitive graph differences must therefore enter through explicit
admission inputs or participation inputs, not through implicit global state,
thread-local host conditions, or renderer residue.

When the world/profile distinction is Query-backed, Worth UI must consume the
typed Query identity and basis lane rather than inventing UI-local world
strings. In particular:

- branch or preview session identity belongs to `WORTHQuerySessionLabel` on the
  ordinary runtime-facing path
- declaration-bound preview identity belongs to `BridgePreviewSessionIdentity`
- snapshot basis posture belongs to `ResolvedSnapshotBasis` and
  `SnapshotResolutionReport`

3.3 does not need to solve cross-world reconciliation, but it must prevent
cross-world identity drift from being mistaken for ordinary same-world graph
mutation.

The exact names may vary, but the graph world contract must be strong enough to
admit a minimal compare surface such as:

- `UiGraphWorldProfile`
- `UiGraphSnapshotComparable`
- `UiGraphWorldDifferenceKind`

with difference kinds equivalent to:

- `same_world_mutation`
- `same_declaration_different_world`
- `different_declaration`
- `not_comparable`

This same contract should be strong enough to support a later comparison shape
such as:

- `same_world_successor`
- `same_world_unrelated_generation`
- `same_declaration_different_world`
- `different_declaration_authority`
- `not_comparable`

## Planned Directory Skeleton

3.3 should force the runtime toward graph-owned responsibility shape rather than
one facade-owned subsystem. The exact names may evolve, but the topology should
look like this:

```text
workspaces/worth-ui/crates/worth-ui-runtime/src/
  graph/
    identity/
      graph_node_identity.rs
      repeated_instance_basis.rs
      mounted_receipt_identity.rs
      mod.rs
    admission/
      graph_instantiation_plan.rs
      graph_instantiation_denial.rs
      declaration_graph_handoff_consumer.rs
      mod.rs
    mutation/
      graph_mutation.rs
      graph_mutation_plan.rs
      graph_mutation_commit.rs
      mod.rs
    snapshot/
      graph_snapshot.rs
      graph_node.rs
      declaration_correspondence.rs
      mod.rs
    topology/
      parent_child_topology.rs
      slot_topology.rs
      page_membership.rs
      region_membership.rs
      mosaic_membership.rs
      mod.rs
    participation/
      participation_axis.rs
      participation_posture.rs
      participation_instantiation.rs
      mod.rs
    mounted_receipt/
      mounted_receipt_authority_seed.rs
      mounted_receipt_authority_seed_store.rs
      mod.rs
    indexes/
      graph_node_index.rs
      declaration_correspondence_index.rs
      parent_child_index.rs
      slot_occupancy_index.rs
      page_participation_index.rs
      published_aspect_index.rs
      consumed_aspect_index.rs
      mounted_receipt_index.rs
      mod.rs
    closeout/
      graph_closeout_report.rs
      milestone33_closeout_profile.rs
      mod.rs
    inspection/
      graph_inspection_support.rs
      graph_evidence_refs.rs
      graph_relevance.rs
      mod.rs
    mod.rs
```

This skeleton is normative about responsibility boundaries, not exact filenames.
Identity, admission, mutation, topology, participation, mounted receipts, and
indexes must stay structurally distinct.

## Graph Certification Matrix

Every major 3.3 contract surface must map to a named proof family:

- `graph_identity_suite`
  proves stable `UiGraphNodeIdentity`, repeated-instance anti-position law, and
  declaration/runtime identity separation.
  owning crate: `worth-ui-certification`
  enforcement: compile-fail plus runtime certification
- `graph_instantiation_suite`
  proves only sealed 3.2 handoff artifacts can feed graph instantiation and that
  contradictory or basis-free runtime instantiation denies locally.
  owning crate: `worth-ui-certification`
  enforcement: compile-fail plus hostile runtime proof
- `graph_topology_suite`
  proves parent/child/slot/page/region/mosaic correspondence is explicit and
  bounded.
  owning crate: `worth-ui-certification`
  enforcement: runtime certification plus topology audit
- `graph_participation_suite`
  proves each participation axis remains explicit and no axis is silently
  inferred from containment alone.
  owning crate: `worth-ui-certification`
  enforcement: runtime certification
- `graph_attachment_suite`
  proves graph-owned Query/service/diagnostic attachment posture exists and no
  later lane needs a side topology to answer node ownership.
  owning crate: `worth-ui-certification`
  enforcement: runtime certification
- `graph_world_profile_suite`
  proves world-sensitive instantiation and participation differences are explicit,
  classified, and cannot drift in through hidden global state.
  owning crate: `worth-ui-certification`
  enforcement: runtime certification plus hostile world-change proof
- `graph_index_alignment_suite`
  proves graph mutation and index mutation commit coherently and ordinary
  lookups never require recursive graph walks.
  owning crate: `worth-ui-certification`
  enforcement: runtime certification plus residue scan
- `aspect_index_correspondence_suite`
  proves publisher and consumer indexes derive from 3.2 aspect contracts plus
  graph correspondence, never renderer-local rediscovery.
  owning crate: `worth-ui-certification`
  enforcement: runtime certification plus dependency audit
- `mounted_receipt_suite`
  proves mounted receipt authority seeds are graph-owned, correspond to graph
  nodes, and are
  available through their authoritative index.
  owning crate: `worth-ui-certification`
  enforcement: runtime certification
- `graph_inspection_support_suite`
  proves graph node identity and topology are inspectable through the formal
  inspection architecture rather than logs, dumps, or renderer helpers.
  owning crate: `worth-ui-certification`
  enforcement: runtime certification plus inspection query proof
- `graph_public_boundary_suite`
  proves public callers cannot bypass the facade into internal graph topology or
  mutate indexes independently from graph authority.
  owning crate: `worth-ui-certification`
  enforcement: compile-fail plus topology audit
- `milestone_3_4_handoff_suite`
  proves 3.4 can consume graph node identity, participation posture,
  declaration correspondence, and aspect indexes through proof-bearing graph
  authority rather than raw graph internals.
  owning crate: `worth-ui-certification`
  enforcement: compile-fail plus hostile handoff proof

## Test Topology Requirements

3.3 tests must obey the same structure law as production:

- identity fixtures belong with graph identity proof, not generic graph helpers
- participation fixtures belong with participation proof, not broad runtime
  world setup
- aspect-index proof belongs with aspect correspondence, not Query execution
  tests
- mounted receipt authority-seed proof belongs with mounted receipt authority,
  not host-adapter tests
- graph inspection support proof belongs with graph evidence and inspection
  support, not ad hoc debug helpers
- graph/index transactional proof belongs with mutation and index alignment, not
  inspector-facing broad suites

Required hostile topology:

- compile-fail fixtures for raw graph-node identity construction, raw index
  mutation, and deep import bypass
- residue scans proving production code does not perform ordinary lookup by
  recursive tree walk
- stability tests showing sibling churn does not mutate repeated-instance
  identity when the admitted basis is unchanged
- world-change tests proving same-declaration graph differences classify as
  world-sensitive instantiation or participation changes rather than silent
  identity churn
- alignment tests showing committed graph mutations and committed index
  mutations remain coherent
- denial tests proving basis-free repeated instances, contradictory topology,
  or contradictory participation posture deny through graph instantiation
- correspondence tests proving declaration identity to graph nodes remains
  explicit across zero/one/many instance cases
- attachment tests proving Query/service/diagnostic attachment posture belongs
  to graph authority even before the richer runtime lanes land
- inspection tests proving graph targets can be queried through the formal
  inspection architecture without logs or broad graph dumps

## World-Sensitivity Law

3.3 must keep world-sensitive graph truth explicit.

- declaration authority from 3.2 may be mostly world-neutral
- graph instantiation may be world-sensitive
- if a world changes whether a declaration instantiates, repeats, mounts, or
  participates, that difference must appear as explicit graph instantiation or
  participation posture rather than hidden host-local drift
- same-world graph mutation and cross-world graph difference must remain
  distinguishable at the graph authority boundary

This prevents accidental conflation between:

- one declaration artifact admitted differently in different runtime worlds
- one graph node that exists but does not participate on a given axis
- one graph node that never instantiated in the current world

## Phases

### Phase 1: Freeze Graph Authority Envelope And Runtime Identity Law

Phase 1 defines the one authoritative graph snapshot boundary and the identity
distinctions every later graph lane depends on.

**Relevant subsystems**

- graph snapshot lane
- graph identity lane
- declaration/runtime correspondence lane
- certification identity suites

**Relevant APIs**

- `UiGraphSnapshot`
- `UiGraphNodeIdentity`
- declaration identity to graph node correspondence surface
- repeated-instance identity basis surface
- world profile or world basis surface for graph authority
- graph attachment posture surface

**Warnings**

- Do not let the first graph snapshot be “just a tree.”
- Do not collapse declaration identity and runtime node identity.
- Do not leave repeated-instance identity as a later cleanup item.

**Test requirements**

- Boundary test: public code cannot mint graph identities from raw scalars.
- Separation test: declaration identity and graph node identity remain distinct
  even for single-instance declarations.
- Stability test: unrelated sibling churn does not rewrite runtime identity when
  the admitted instance basis is unchanged.
- World-basis test: graph authority carries enough world classification to
  distinguish same-world mutation from cross-world re-admission.

**Engineering decisions**

- `UiGraphSnapshot` is the one graph authority artifact.
- runtime identity is not tree position
- repeated-instance basis must be explicit before graph mutation
- correspondence between declaration and runtime identity is first-class
- world classification is explicit graph authority, not ambient runtime state
- graph-owned attachment posture exists before richer binding/service/runtime
  lanes broaden
- identity and correspondence indexes begin with the first graph authority
  envelope rather than waiting for the final index closeout phase

**Open questions**

- None.

### Phase 2: Admit Graph Nodes Only Through The Sealed 3.2 Handoff Boundary

Phase 2 closes the declaration-to-graph instantiation seam.

**Relevant subsystems**

- graph instantiation lane
- declaration handoff consumer lane
- graph instantiation denial lane
- certification handoff suites

**Relevant APIs**

- `UiDeclarationGraphHandoff`
- graph instantiation plan
- graph instantiation denial surface
- graph mutation entry surface
- repeated-instance basis admission surface

**Warnings**

- Do not let graph construction read raw declarations or source text directly.
- Do not allow renderer-local helpers to classify graph families.
- Do not mint runtime nodes when repeated-instance basis is missing or
  contradictory.

**Test requirements**

- Handoff test: graph instantiation accepts only sealed declaration handoff
  artifacts, not parser-local or support-only products.
- Denial test: contradictory structural or repeated-instance basis input fails
  before graph mutation.
- Boundary test: runtime-data-keyed repeated-instance basis must arrive through
  an admitted typed boundary rather than arbitrary caller-local keys.
- Boundary test: production code cannot bypass the handoff consumer with raw
  graph-node construction.

**Engineering decisions**

- 3.2 remains the only authored semantic source
- graph instantiation resolves runtime instantiation facts before mutation
- denial happens at the instantiation boundary, not deep inside index maintenance
- runtime-data-driven instance identity is admitted explicitly or denied
- declaration-to-runtime correspondence index lands with graph instantiation,
  not as a late add-on

**Open questions**

- None.

### Phase 3: Materialize Parent Child Slot And Membership Topology As Graph Truth

Phase 3 turns structural intent into admitted runtime topology.

**Relevant subsystems**

- graph topology lane
- parent-child topology
- slot topology
- page-region-mosaic membership
- certification topology suites

**Relevant APIs**

- parent identity to child set lookup
- slot identity to occupant set lookup
- page membership surface
- region membership surface
- mosaic membership surface
- graph attachment posture lookup

**Warnings**

- Do not recover topology by recursively rereading declarations.
- Do not treat slot occupancy as implicit sibling order.
- Do not collapse page, region, and mosaic membership into one generic tag.

**Test requirements**

- Lookup test: ordinary topology lookup uses authoritative index access rather
  than recursive descent.
- Membership test: page, region, and mosaic participation is explicit and
  queryable.
- Denial test: contradictory topology handoff input denies locally.

**Engineering decisions**

- graph topology is runtime-owned truth
- membership categories are first-class, not comments on nodes
- authored ordering matters only when 3.2 admitted it as semantic meaning
- graph-owned attachments remain part of graph truth rather than future side
  registries
- topology indexes land with topology truth rather than after a temporary
  traversal-based period

**Open questions**

- None.

### Phase 4: Encode Participation As Explicit Multi Axis Runtime Posture

Phase 4 closes the participation storage and mutation model.

**Relevant subsystems**

- participation lane
- participation posture storage
- participation instantiation logic
- certification participation suites

**Relevant APIs**

- node participation posture lookup
- page identity to participating node set lookup
- participation mutation surface

**Warnings**

- Do not reduce participation to one boolean.
- Do not infer focus, input, or accessibility participation from visibility
  alone.
- Do not make diagnostic participation implicit.

**Test requirements**

- Axis test: all required axes are stored explicitly.
- Independence test: nodes may differ across axes without graph corruption.
- Lookup test: page participation index respects explicit posture rather than
  simple containment.
- Axis-evidence test: page participation lookup returns enough information to
  explain axis-specific inclusion without descendant walks.

**Engineering decisions**

- participation lives on graph authority
- axis dependency rules must be explicit if they exist
- non-participating nodes may still exist in the graph
- page participation lookup is an authority surface, not a lossy convenience set
- existence, paint, query-bound, and service-bound posture are explicit rather
  than left to later folklore
- participation index surfaces land with participation truth even if Phase 6
  later closes the full cost-honesty proof

**Open questions**

- None.

### Phase 5: Seed Mounted Receipt Authority At The Graph Boundary

Phase 5 binds mounted receipt authority seeds to graph truth without pulling in
the later full mounting runtime.

**Relevant subsystems**

- mounted receipt authority seed lane
- mounted receipt identity lane
- mounted receipt authority seed storage
- certification mounted receipt suites

**Relevant APIs**

- `UiMountedReceiptIdentity`
- mounted receipt lookup
- graph node to mounted receipt correspondence surface

**Warnings**

- Do not let mounted receipt authority seeds live only in host-local side tables.
- Do not key mounted receipt identity only by host handle.
- Do not let mounted posture and mounted receipt authority seed storage drift apart.
- Do not treat mounted receipt authority seeds as mounted frames or host output.

**Test requirements**

- Correspondence test: mounted receipt authority seeds map explicitly to graph nodes.
- Lookup test: mounted receipt identity lookup is bounded and authoritative.
- Alignment test: mounted posture changes and mounted receipt mutation stay
  transactionally aligned.

**Engineering decisions**

- mounted receipt authority seeds are graph-owned runtime evidence placeholders
- host-linked evidence is attached, not authoritative by itself
- mounted receipt identity is separate from graph node identity
- mounted receipt slot index surfaces land with mounted receipt authority seeds
  even though later mounting behavior is deferred

**Open questions**

- None.

### Phase 6: Build The Core Index Set And Make Ordinary Lookup Bounded

Phase 6 implements the roadmap index set as graph authority.

**Relevant subsystems**

- core index lane
- declaration correspondence index
- topology indexes
- aspect indexes
- mounted receipt index
- graph inspection support lane
- certification lookup and residue suites

**Relevant APIs**

- graph node identity lookup
- declaration identity to graph nodes lookup
- parent identity to child set lookup
- slot identity to occupant set lookup
- page identity to participating node set lookup
- published aspect lookup
- consumed aspect lookup
- mounted receipt identity lookup
- graph evidence ref lookup or equivalent graph inspection handle lookup

**Warnings**

- Do not call these caches if ordinary runtime lookup depends on them.
- Do not perform hidden recursive walks under scalar lookup APIs.
- Do not rediscover aspect publishers or consumers from declarations or
  renderer-local code.

**Test requirements**

- Cost-honesty test: ordinary lookup does not recurse through the graph.
- Aspect test: aspect indexes derive from 3.2 contracts plus graph
  correspondence.
- Multiplicity test: declaration-to-graph lookup handles zero/one/many nodes
  honestly.

**Engineering decisions**

- ordinary graph access is index-first
- aspect indexes are runtime-owned projections from declaration truth plus graph
  truth
- lookup surfaces must reveal which index family they traverse
- graph inspection uses formal graph evidence handles rather than broad dumps
- earlier phases introduce the indexes required by their graph truth lanes;
  Phase 6 closes the full index set, cost-honesty contract, and residue proof

**Open questions**

- None.

### Phase 7: Transactionally Align Graph Mutation And Index Mutation

Phase 7 turns coherence into a mechanical boundary.

**Relevant subsystems**

- graph mutation lane
- graph mutation commit lane
- index alignment lane
- certification coherence suites

**Relevant APIs**

- graph mutation plan
- graph mutation commit result
- graph/index closeout evidence

**Warnings**

- Do not let indexes repair lazily behind a committed graph snapshot.
- Do not expose a repair-required graph snapshot at the public graph authority
  boundary.
- Do not permit direct index mutation outside the graph mutation boundary.
- Do not certify success from shape alone; prove coherence.

**Test requirements**

- Atomicity test: committed graph snapshots always pair with committed coherent
  indexes.
- Denial test: failed mutation cannot publish partial graph authority.
- No-half-state test: public graph authority exposes either the prior coherent
  snapshot or the new coherent snapshot, never an intermediate repair state.
- Boundary test: production code cannot mutate core indexes independently from
  graph mutation.

**Engineering decisions**

- graph and index mutation share one commit law
- temporary staging is internal only
- coherence is part of ordinary completion, not a future hardening pass

**Open questions**

- None.

### Phase 8: Close Graph Authority And Hand 3.4 A Proof Bearing Runtime Surface

Phase 8 publishes the exact graph authority 3.4 may consume and the exact work
that still remains outside 3.3.

**Relevant subsystems**

- graph closeout lane
- 3.4 handoff lane
- certification closeout audit

**Relevant APIs**

- `UiGraphCloseoutReport`
- 3.4 graph handoff surface
- graph closeout profile
- graph inspection support report or equivalent closeout evidence

**Warnings**

- Do not hand 3.4 raw graph internals or declaration artifacts and call that a
  handoff.
- Do not claim obligation selection, Query execution, or host observation as
  solved by graph truth alone.
- Do not certify milestone completion without proving bounded lookup and index
  alignment.

**Test requirements**

- Handoff test: 3.4-facing graph inputs are fully derivable from graph
  authority without tree walks or declaration reopening.
- Coverage test: closeout proof enumerates the shipped identity, topology,
  participation, attachment, mounted receipt, index, and graph-inspection
  support lanes.
- Rejection test: raw declaration artifacts, recursive walks, and index-free
  lookup paths are denied as ordinary graph-access surfaces.

**Engineering decisions**

- 3.3 closes at graph authority and bounded lookup
- 3.4 consumes graph-owned identity, participation, and correspondence
  evidence, not tree structure folklore
- closeout must make remaining admission and obligation work explicit

**Open questions**

- None.

## Must Ship

- `UiGraphNodeIdentity`
- `UiGraphSnapshot`
- explicit declaration identity to graph node correspondence
- explicit repeated-instance identity basis
- explicit graph world profile or world basis
- explicit graph-owned attachment posture for Query, service, and diagnostic
  lanes
- parent/child topology
- slot occupancy topology
- page membership
- region membership
- mosaic membership
- explicit participation posture covering:
  - exists
  - mounted
  - visible
  - layout
  - hit-test
  - focus
  - accessibility
  - paint
  - input
  - query-bound
  - service-bound
  - diagnostic
- graph-owned mounted receipt identity and authority seed storage
- authoritative core indexes for:
  - `graph node identity -> node`
  - `declaration identity -> graph node(s)`
  - `parent identity -> child set`
  - `slot identity -> occupant set`
  - `page identity -> participating node set`
  - `published aspect -> publishing node/receipt set`
  - `consumed aspect -> dependent node/receipt set`
  - `mounted receipt identity -> mounted receipt`
- graph mutation and index mutation transactional alignment
- certification proof that ordinary graph lookup avoids recursive tree walk and
  tree-position identity

## Must Preserve

- Milestone 3.1’s single public facade discipline
- Milestone 3.2’s declaration authority and sealed graph handoff boundary
- strict separation between declaration truth and graph truth
- strict separation between graph truth and later obligation or observation
  truth
- host neutrality through `worth-ui-host-contract`
- explicit support for repeated instances without position identity folklore
- explicit participation axes instead of single-flag collapse

## Acceptance Evidence

3.3 is complete only when all of these are true:

- runtime nodes are admitted from sealed 3.2 handoff artifacts rather than
  source text, parser output, or renderer-local interpretation
- every runtime node has a stable `UiGraphNodeIdentity` that does not depend on
  tree position
- repeated instances use an admitted stable identity basis or deny locally
- world-sensitive graph differences are explicit at the graph authority
  boundary rather than ambient runtime drift
- declaration identity to graph node correspondence is explicit and bounded
- parent/child/slot/page/region/mosaic topology is graph-owned and queryable
- participation axes remain explicit: exists, mounted, visible, layout,
  hit-test, focus, accessibility, paint, input, query-bound, service-bound,
  and diagnostic
- ordinary graph lookups do not require recursive tree walks
- graph mutation and index mutation are transactionally aligned
- aspect publisher and consumer lookup derives from 3.2 contracts plus graph
  correspondence instead of declaration rescans
- mounted receipt authority seed identity and lookup are graph-owned and bounded
- graph-owned attachment posture exists before richer binding/service/diagnostic
  execution lanes land
- graph truth is inspectable through the formal inspection architecture instead
  of logs or broad dumps
- compile-fail, runtime, topology, and residue suites prove public callers
  cannot bypass graph authority or mutate indexes independently

## Allowed Debt

3.3 may defer richer participation policy, broader host integration, later
inspection storytelling, or more advanced index families when the ordinary
graph-authority path already exists and the deferred work is mechanically
contained.

Any allowed debt must satisfy `MENTALITY.md`: it must be named, important
enough to justify deferral, bounded so it cannot be mistaken for the ordinary
lane, and attached to an explicit follow-on milestone.

3.3 may not mark these as debt:

- `UiGraphNodeIdentity`
- explicit declaration-to-runtime correspondence
- stable repeated-instance identity basis law
- explicit graph world profile or world basis
- explicit graph-owned attachment posture for Query, service, and diagnostic
  lanes
- explicit parent/child/slot/page/region/mosaic graph topology
- explicit participation posture axes
- graph-owned mounted receipt identity and authority seed lookup
- the full roadmap core index set
- transactional alignment between graph mutation and index mutation
- residue rejection for recursive-walk ordinary lookup
- sealed 3.4 handoff from graph authority

## Sequencing Notes

3.3 belongs immediately after 3.2 because canonical declaration authority is not
enough. The runtime still needs one authority for what exists, where it lives,
how it participates, and how later phases find it cheaply.

3.3 belongs before:

- 3.4, because touched-obligation selection needs graph identity,
  correspondence, and participation truth before it can select checks honestly
- 3.5, because inspection needs graph-owned runtime evidence before it can
  explain existence, participation, or mounted state
- broader host, layout, and interaction work, because those lanes need explicit
  participation and mounted receipt authority instead of tree folklore

## Required Self Check

Before closeout, answer these with evidence:

- Does 3.3 make graph truth runtime-owned authority rather than a declaration
  tree with better names?
- Can ordinary runtime lookup avoid recursive graph walk for the roadmap index
  families?
- Can repeated instances keep stable identity without relying on sibling
  position noise?
- Can later phases name the graph authority surface they consume rather than
  “the current runtime tree”?
- Do mounted receipts and participation posture belong to graph authority
  rather than host or renderer side tables?

Reopen 3.3 if any of these become true:

- ordinary lookup requires recursive tree walk
- graph node identity depends on tree position
- repeated-instance identity depends on incidental order or allocation
- world-sensitive graph differences drift in through hidden global or host state
- participation is inferred rather than stored explicitly
- Query/service/diagnostic ownership requires side topologies outside graph
  authority
- indexes can drift from committed graph truth
- aspect lookup reopens declarations or consults renderer-local logic
- mounted receipts live outside graph authority
- graph inspection requires logs, host fields, or broad runtime dumps
- public exports mirror internal graph topology deeply enough that internal
  refactors would become breaking changes
