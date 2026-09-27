# WORTH Relational Vision

## Thesis

WORTH needs a first-class truth runtime, not an ad hoc storage layer.

`worth-relational` is a standalone truth-state runtime library: the system that
owns identity, mutation, history, diffs, lineage, and traversal over a
host-managed graph or relational state model. It is not merely persistence, and
it is not a thin convenience wrapper around in-memory collections. It is the
runtime responsible for authoritative graph truth.

The runtime is intended for systems where authoritative state must outlive any
single derived read model, evaluation layer, export format, or application
session. That includes geometry kernels, chip-design systems, AI-native editing
systems, and web/data platforms that need auditable, branchable, replayable
truth instead of loose operational state.

## What This Runtime Is For

`worth-relational` exists for product surfaces where correctness and historical
meaning matter as much as current value.

It is meant to support:

- AI systems that need world state they can branch, inspect, rewind, compare,
  and audit exactly instead of heuristically reconstructing what changed
- AI world-model and agent-environment systems that need speculative futures,
  structural correspondence, historical grounding, and exact committed truth as
  a first-class substrate for planning
- chip-design systems that need identity-safe rewiring, exact connectivity
  history, snapshot-stable concurrent analysis, and replayable certification
  rather than fragile tool-local state
- geometry and CAD kernels that need topology identity to survive rebuilds,
  branch-local edits to stay intelligible, corruption to localize precisely,
  and large relational traversals to remain industrially fast
- web and data platforms that need durable change feeds, exact recovery,
  consistent historical reads, and truthful read acceleration without letting
  indexes become accidental authority
- collaborative and branch-divergent editing systems that need real
  branch-native truth instead of shallow undo stacks and ad hoc merge metadata
- incremental compiler and IR systems that need durable graph truth,
  historical resolution, and replayable state transitions instead of bespoke
  invalidation folklore
- workflow, node-editor, and visual-editor platforms that need transactional
  graph state, stable identity, scoped diffs, and graph introspection so large
  interactive systems stop devolving into fragile editor glue

The technical thesis is the same across all of them:

- truth must be authoritative
- history must be explicit
- diffs must be first-class
- identity must survive change
- reads must remain stable under mutation
- replay must be real

## Why This Runtime Is Different

These are not optional add-ons. They are the capabilities that make
`worth-relational` strategically different from ordinary graph storage:

- generational IDs and multi-layer identity
- sparse transactional mutation
- nested savepoints
- MVCC snapshots
- snapshot reads during active mutation
- branchable version graph
- deterministic replay
- graph time travel and retention-aware history
- patch streams / CDC
- protocol-facing streaming patch feeds
- first-class relational aspect semantics
- lineage-aware identity evolution
- branch-aware correspondence hooks
- bulk relational queries
- graph introspection and transaction-surface introspection
- schema-defined relation integrity and typed relation contracts
- structural fingerprint surfaces

If these are treated as â€œnice to have later,â€ the runtime collapses back into
ordinary mutable graph storage and loses the leverage needed for high-assurance
systems.

## Mission

`worth-relational` exists to make truth-state graph operations safe,
replayable, branchable, and inspectable at industrial scale.

It must answer these questions as native runtime responsibilities:

- What is the authoritative identity of this entity or relation?
- How do writes happen transactionally and rewind cheaply?
- How do multiple readers observe stable graph state while mutation continues
  elsewhere?
- How does committed truth emit structured diffs rather than opaque mutation
  fallout?
- How does an entity survive replacements, splits, merges, and branch
  divergence?
- How do downstream systems query the graph efficiently in bulk rather than one
  edge at a time?
- How do operators and tools inspect recent mutation, retained history, and
  graph structure directly instead of reverse-engineering those answers from
  side effects?
- How do schema contracts and relation invariants become authoritative runtime
  behavior instead of scattered application code?

This runtime is the substrate that makes mergeable history, AI-native editing,
deterministic replay, topology-aware tooling, connectivity analysis, and
derived computation possible across many domains. For geometry kernels
specifically, it is the difference between a modeler that keeps losing the
meaning of topology under change and one that can preserve, inspect, replay,
and certify structural truth as the model evolves.

## Architectural Model

### Runtime stack

| Layer | Responsibility | Owns |
| --- | --- | --- |
| `worth-relational` | Truth-state graph runtime | identity, mutation, history, diffs, lineage, traversal, integrity |
| `worth-signal` | Derived-computation runtime | invalidation, recomputation, conditions, scheduling, runtime self-inspection |
| Bridge / integration | Decoupled coordination | patch-to-invalidation, aspect mapping, snapshot evaluation, key mapping |

### Ownership boundary

`worth-relational` owns:

- stable identity and safe handle reuse
- transactional mutation boundaries
- snapshot/history/version graph behavior
- structured diffs and change streams
- lineage and correspondence foundations
- structural identity and structural fingerprint surfaces
- query and traversal primitives
- graph introspection and historical inspection surfaces
- transaction-surface introspection
- secondary index and derived-index hooks
- integrity and schema enforcement hooks
- retention and reclaim visibility for historical truth

`worth-relational` does not own:

- reactive invalidation scheduling
- derived-computation policies or planners
- domain-specific numeric solvers
- projection execution policy
- permanent fusion with signal execution
- host-specific kernel feature execution

Structural rule:

Truth mutation, truth history, and truth identity live here. Downstream systems
may observe, project, accelerate, or react to truth, but they do not redefine
it.

## Principles

1. Truth is graph-native, not blob-native.
2. Identity must survive mutation, branching, and structural change.
3. Writes happen only through transactions.
4. Undo, branching, and replay are foundational, not add-ons.
5. Diffs are first-class outputs of commit, not side effects reconstructed later.
6. Lineage is a graph, not loose metadata.
7. Bulk traversal and bulk mutation matter as much as single-entity convenience APIs.
8. Deterministic ordering is a product feature, not a debugging aid.
9. The runtime remains generic; domain meaning arrives through schema, kinds, and host hooks.
10. The runtime must stand on its own as a reusable library.
11. Integrity and schema validation are first-class runtime architecture.
12. Diagnostics are a first-class runtime contract.
13. Parallelism happens around immutable reads and immutable outputs, not through authoritative mutation.

## Foundational Decisions

These are locked architectural decisions:

- single logical writer for authoritative truth commit
- generational identity as the only authoritative ID base
- separate entity and relation identity systems
- immutable committed snapshots
- canonical ordering for every observable output
- commit-native patch and CDC emission
- structured diagnostics as a production contract
- arena plus sidecar storage as the foundational memory model
- parallelize preparation and consumption, serialize authority
- derived indexes remain non-authoritative and publish immutably
- replay is executed from canonical commit envelopes, not patch-only reconstruction
- history representation is merge-ready now: ordered parent commit lists
- authoritative storage-visible reads always retain a non-index fallback path
- lineage is a constrained graph with explicit invariants, not event logging theater
- durability persists canonical truth artifacts rather than transient arena layout
- retention and reclaim are product-visible semantics, not hidden garbage collection folklore
- no hidden mutation during reads
- no scheduler-dependent semantics
- low-level bulk query and bulk mutation mechanics remain in the runtime; domain semantics layer above them

## How This Vision Drives Engineering

This document is intentionally written so a roadmap can be derived from it.

The derivation rule is:

- each capability pillar below implies concrete runtime surfaces that must exist
- each technical role implies constraints that implementation must preserve
- each â€œwhat this enablesâ€ section implies real product use cases the runtime
  must serve, not marketing examples
- if a capability is named here but not yet fully present in code, it belongs on
  the roadmap as remaining engineering work
- if a capability is present in code but not yet proven under the hostile
  scenarios in
  [test-requirements.md](/Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/plans/worth-relational/test-requirements.md),
  it belongs on the roadmap as certification work

In other words:

- the vision says what the runtime must be able to do
- the roadmap says what still must be engineered
- the test requirements say what must be proven before the capability is trusted

## Capability Pillars

Each pillar describes both the technical role of the feature and the kinds of
systems that need it.

### Identity Architecture

#### Generational IDs

Technical role:
Handles remain safe under deletion and slot reuse. All persistent references
must pass through typed, generation-aware identity rather than raw slot
indices.

What this enables:

- AI editing systems can hold references across speculative rewrites without
  stale-handle corruption
- chip tools can safely reuse storage while preserving identity correctness for
  long-running analysis
- geometry kernels can keep stable external references even as topology is
  deleted and recreated
- web/data systems can expose durable handles without ABA-style bugs

#### Structural identity hooks

Technical role:
Storage identity alone is not enough when systems need to recognize â€œthe same
structureâ€ across rebuilds, branches, or host-driven reinterpretation.

What this enables:

- AI systems can compare candidate rewrites structurally instead of only by raw
  storage identity
- chip flows can match corresponding nets/modules across branch-local rewrites
- geometry/CAD systems can reason about rebuilt topological regions without
  pretending slot identity is semantic identity
- web/data systems can support durable correspondence across schema-driven
  transformations

#### Structural fingerprint surfaces

Technical role:
Hosts need canonical runtime-supported fingerprint surfaces when they want to
compare structures, detect equivalence classes, or persist structural
comparisons without collapsing those policies into raw storage identity.

What this enables:

- AI systems can compare candidate worlds or edits by structure rather than
  only by storage slot reuse
- chip tools can fingerprint connectivity regions for branch comparison and
  reuse analysis
- geometry/CAD systems can compare rebuilt topological neighborhoods
- web/data systems can detect equivalent transformed structures across
  migrations or reconciliation flows

#### Multi-layer identity model

Technical role:
The runtime must distinguish storage identity, lineage identity, structural
identity, and optional semantic identity.

What this enables:

- AI branch exploration where â€œsame conceptâ€ and â€œsame stored recordâ€ are not
  the same question
- chip and geometry workflows where historical continuity matters more than raw
  storage slot reuse
- web/data platforms where durable external identity, storage identity, and
  migration-time correspondence must stay separate

### Transaction Architecture

#### Transactional mutation

Technical role:
Truth mutation flows only through explicit transaction boundaries. This is what
makes history, rollback, diff emission, and replay reliable.

What this enables:

- AI-assisted editing with controlled speculative changes
- chip editing flows where rewires and replacements can be validated before
  publication
- geometry operations where structural edits either publish coherently or leave
  no residue
- web/data applications where business truth changes are auditable and replayable

#### Sparse undo log

Technical role:
Rollback scales with touched state, not whole-graph size.

What this enables:

- speculative AI transforms that rewind cheaply
- chip and geometry editing sessions that can abandon local experimental paths
- large web/data workflows that need correction without whole-state copying

#### Nested savepoints

Technical role:
Complex operations need partial rollback without discarding an entire outer
transaction.

What this enables:

- AI systems exploring alternate repair or synthesis paths inside one larger
  operation
- geometry kernels evaluating multiple local topological repair attempts
- chip tools trying alternate rewiring plans before publishing one
- multi-step application workflows that need scoped rewind points instead of
  all-or-nothing failure

#### Transaction-surface introspection

Technical role:
The runtime must expose recent mutation and transaction-boundary inspection as
product surfaces, not force operators to derive them indirectly from logs,
patches, or debug-only tools.

What this enables:

- AI systems can explain what a speculative branch changed before publication
- chip and geometry tools can inspect exact touched truth before accepting an
  operator result
- workflow and visual-editor platforms can show transaction-scoped previews and
  audit surfaces
- operational tooling can reason about transaction shape without replaying full
  history

#### Bulk mutation APIs

Technical role:
Large graph edits must be expressible as one structured operation, not a loop
of tiny writes.

What this enables:

- model importers
- geometry rebuild passes
- chip netlist transforms
- web/data migrations and large operator batches

### History Architecture

#### MVCC snapshots

Technical role:
Readers need stable views while mutation continues elsewhere.

What this enables:

- AI agents inspecting stable truth while other work continues
- chip analysis over pinned snapshots during hot rewrites
- CAD/geometry tools comparing historical surfaces without blocking edits
- web/data applications serving consistent reads during active updates

#### Snapshot reads during active mutation

Technical role:
Long-running reads and derived computation must be able to pin stable truth
instead of racing active writes.

What this enables:

- concurrent analysis in chip design
- stable geometry inspection during editing
- branch comparison and audit tooling
- high-value web/data reads that must not drift mid-request

#### Branchable version graph

Technical role:
Version history must be graph-shaped, not stack-shaped.

What this enables:

- AI-native editing and alternate solution branches
- branch-local chip or geometry experiments
- collaborative design flows
- web/data review and â€œwhat-ifâ€ workflows that need more than linear undo

#### Deterministic replay

Technical role:
Canonical commit artifacts must be sufficient to reconstruct observable truth
exactly.

What this enables:

- AI debugging and regression minimization
- chip and geometry certification workflows
- audit and incident analysis
- trustworthy recovery in web/data systems

#### Graph time travel and retention

Technical role:
The runtime must intentionally retain or reconstruct historical graph states and
make reclamation explicit.

What this enables:

- historical geometry or topology inspection
- hot-net history analysis in chip design
- audit windows in web/data platforms
- AI systems comparing current truth against prior committed states

Technical consequence:
Retention, pinning, and reclaim must be explicit runtime surfaces. Historical
truth availability cannot be treated as an accidental side effect of whether
old data has not yet been collected.

### Diff and CDC Architecture

#### CDC / patch streams

Technical role:
Every commit emits structured change data as native runtime output.

What this enables:

- bridge routing and downstream invalidation
- subscriber systems consuming durable truth changes
- audit exports
- application/web platforms reacting to canonical committed state changes

#### Protocol-facing streaming patch feeds

Technical role:
Patch streams must be consumable as durable external protocol surfaces, not
only as internal bridge artifacts.

What this enables:

- resumable downstream subscribers
- replication and audit pipelines
- chip and geometry downstream consumers that must track canonical truth
  without inference
- platform integrations that need exact, machine-checkable committed change
  feeds

#### Stream correctness semantics

Technical role:
Ordering, resume, checkpoint, idempotence, and failure behavior must be
explicit runtime contract, not accidental behavior.

What this enables:

- durable subscribers
- exact CDC recovery after interruption
- chip and geometry downstream consumers that cannot tolerate dropped or drifted changes
- AI and web platforms that consume truth changes incrementally

#### Relational aspect semantics

Technical role:
The truth runtime must have first-class aspect semantics for entities and
relations. Aspects are not only payload-derived labels; they are stable,
queryable change surfaces that let committed truth describe which semantic
facets changed.

The runtime must be able to express:

- entity aspects
- relation aspects
- aspect-aware committed diffs
- aspect-aware historical reads
- aspect-aware and lineage-aware historical reads where both change surface and
  identity evolution matter
- aspect-aware projections and bulk queries
- canonical aspect identity and ordering in observable artifacts

What this enables:

- AI systems can distinguish structural edits, metadata edits, and semantic
  edits when inspecting committed truth history
- chip-design systems can express connectivity, hierarchy, timing-related, and
  metadata change surfaces explicitly rather than inferring everything from raw
  payload keys
- geometry and CAD systems can express topology, adjacency, geometry, and
  annotation-style change surfaces explicitly
- web and data platforms can ship durable CDC with domain-aware change
  semantics instead of only generic field-name deltas

#### Aspect-tagged diffs

Technical role:
Committed diffs must say what changed, not merely that something changed, and
that precision must apply to both entities and relations.

What this enables:

- exact committed change description across history
- aspect-aware CDC consumers
- precise topology and connectivity change reporting
- incremental tools that need field or semantic-surface level change artifacts

#### Aspect-aware query and projection surfaces

Technical role:
Aspect semantics must not stop at diff output. Queries, projections, and
historical reads need to be able to ask for specific semantic surfaces of truth
instead of treating every read as a full-record fetch.

What this enables:

- aspect-scoped bulk reads in large AI and application workloads
- topology- or geometry-focused reads in CAD without paying for unrelated
  payload surfaces
- connectivity- or hierarchy-focused reads in chip design
- better proportionality for large read workloads where only specific truth
  surfaces matter

#### Lineage-aware diffs

Technical role:
Diffs must be able to express identity evolution, not only add/remove/update.

What this enables:

- topology replacement and split semantics in CAD
- connectivity-preserving rewiring semantics in chip design
- identity-sensitive downstream systems
- AI tooling that needs to know what old truth became, not just what disappeared

### Lineage Architecture

#### Lineage events

Technical role:
Replace, split, and merge-like structural changes are explicit first-class
events.

What this enables:

- topology identity survival in geometry/CAD
- rewiring and replacement history in chip design
- identity-aware model editing in AI systems
- durable evolution history in web/data domains where records transform over time

#### Historical ID resolution

Technical role:
Consumers need to ask what an older entity became.

What this enables:

- historical topology tracing
- signal/net ancestry queries
- audit and replay consumers tracking evolving entities
- AI systems grounding edits against prior committed identity

Technical consequence:
Historical resolution must be ergonomic enough to serve as a product surface
for tools and downstream consumers, not merely an internal correctness proof.

#### Lineage graph

Technical role:
Identity evolution must be queryable as graph truth, not loose metadata.

What this enables:

- branch comparison
- merge and correspondence tooling
- topology and connectivity debugging
- historical identity reasoning for applications that need traceability

Technical consequence:
The lineage graph must be queryable directly. Identity evolution cannot remain
opaque behind one-off helper paths.

#### Branch-aware correspondence hooks

Technical role:
Branches need a way to express â€œthese are probably the same thingâ€ without
silently making that authoritative.

What this enables:

- AI-assisted matching between alternate edits
- chip and geometry branch comparison
- future merge tooling
- application-level reconciliation policies over divergent histories

### Integrity Architecture

#### Schema-defined relation invariants

Technical role:
The runtime must enforce generic relation integrity contracts declared by the
schema instead of relying on convention or scattered application code.

What this enables:

- topology and connectivity systems can declare legal relation structure
  without embedding integrity policy in every operator
- workflow and application graphs can express cardinality, uniqueness, and
  ownership rules centrally
- AI systems can validate speculative edits against real truth contracts before
  publication

#### Typed relation contracts

Technical role:
Relation and kind contracts must be strong enough to express industrial graph
schemas clearly while keeping the runtime generic.

What this enables:

- chip, geometry, IR, and workflow systems can define rich graph schemas
  without runtime ambiguity
- schema drift and illegal relation formation are rejected earlier
- host code can reason about graph legality without reinventing basic contract
  enforcement

#### Integrity validation hooks

Technical role:
Validation and commit-time integrity checking must be part of the runtime
contract, not optional application aftercare.

What this enables:

- authoritative rejection of corrupted truth
- localized diagnostics for bad graph edits
- durable confidence that replay and recovery preserve accepted truth only

### Query and Scale Architecture

#### Single-entity traversal primitives

Technical role:
The graph still needs sharp low-level operations like `targets_of` and
`sources_of`.

What this enables:

- precise inspection tooling
- mutation operators
- small interactive product surfaces
- adapter layers that need dependable graph primitives

#### Graph introspection APIs

Technical role:
The runtime must expose graph structure, relation shape, counts, touched scope,
and inspection surfaces directly rather than forcing every host to build its
own partial graph debugger.

What this enables:

- AI systems can inspect world state and mutation scope directly
- workflow and node-editor platforms can power graph tooling and admin surfaces
- compiler and IR systems can inspect graph health and connectivity
- performance and correctness tooling can query truth structure without custom
  invasive instrumentation

#### Bulk relational queries

Technical role:
Large graph workloads need vectorized queries, not per-entity loops disguised
as APIs.

What this enables:

- chip fanout and hierarchy analysis
- geometry adjacency and neighborhood queries
- AI planning and validation over large truth surfaces
- web/data platforms serving large filtered truth slices efficiently

Technical consequence:
The runtime must provide low-level mechanical bulk primitives such as
source/target scans, relation-kind scans, and ordered bulk traversal building
blocks. Semantic domain queries belong above them.

#### Relation-type scans

Technical role:
Systems frequently need to scan all edges of a kind efficiently.

What this enables:

- topology relation scans
- connectivity-class queries
- schema-driven analytics
- broad platform queries without full graph walks

#### Secondary index and derived-index hooks

Technical role:
The runtime must support read-side acceleration without allowing derived state
to become authority.

What this enables:

- high-volume product queries
- chip and geometry indexing for common access paths
- host-tunable read acceleration
- web/data performance surfaces that still preserve truth fallback

#### Parallel read access and partitioning hints

Technical role:
Stable snapshot reads and future scale-out need deterministic packetized read
surfaces, not only serial traversal.

What this enables:

- snapshot-safe parallel analysis
- partition-aware bulk planning for chip and geometry workloads
- AI systems distributing read-side inspection over immutable truth
- high-scale application platforms that need concurrency without semantic drift

Technical consequence:
Partition-aware query surfaces and partitioning hints matter at runtime level
for scale. They cannot be left entirely to higher layers if the runtime wants
to make honest locality and proportionality claims.

#### Memory stability guarantees

Technical role:
Very large graph systems fail when allocation behavior is unpredictable or
locality collapses.

What this enables:

- long-lived geometry and chip workloads
- high-churn AI editing systems
- sustained web/data platform operation under heavy history and query pressure

Technical consequence:
Predictable allocation, bounded churn, and locality-preserving layout are part
of the product requirement, not implementation polish.

## Domain Fit

The runtime remains generic, but the intended fit is explicit.

### AI Systems

`worth-relational` should support:

- speculative editing with savepoints and rollback
- branch-local alternate solutions
- deterministic replay of model-assisted changes
- correspondence and lineage over rewritten structures
- stable snapshots for evaluation and audit
- structural comparison and recent-mutation inspection for AI world-modeling
- exact historical inspection for agent reasoning over prior committed truth

Revolutionary use:
an AI system can treat world state as a branchable, replayable, auditable truth
graph instead of a pile of mutable tool state and post-hoc logs.

### Collaborative and Branch-Divergent Systems

`worth-relational` should support:

- branch-local edits with explicit lineage and correspondence
- replayable historical truth for review and audit
- deterministic diff and CDC surfaces for collaboration tooling
- retained snapshots and historical resolution for â€œwhat changed?â€ workflows

Revolutionary use:
collaboration can move from â€œbest effort merge and undoâ€ to true
branch-native editing where identity, history, and change semantics survive
divergence cleanly.

### Incremental Compiler and IR Systems

`worth-relational` should support:

- durable graph truth for IR/state representation
- deterministic replay and historical inspection
- fast bulk traversal over typed relation contracts
- structural identity and correspondence across rewrites and branch-local edits

Revolutionary use:
compiler and IR systems can get a real truth substrate with durable history and
identity survival instead of rebuilding these guarantees piecemeal around a
query engine.

### Chip Design

`worth-relational` should support:

- rewiring and replacement with trustworthy identity/history semantics
- snapshot-safe concurrent analysis
- exact connectivity diffs and CDC
- branch-local alternate implementations
- durable replay and recovery for certification-grade flows

Revolutionary use:
chip flows can treat connectivity truth as replayable, branchable, and
historically queryable at industrial scale, which is much closer to a
certifiable design state engine than a traditional pile of tool-local graphs.

### Geometry and CAD

`worth-relational` should support:

- topology identity survival across split, replace, and rebuild operations
- adjacency and incidence relations as first-class truth
- branch-local structural edits
- corruption localization through lineage, relation history, and diagnostics
- historical topology inspection under retained snapshots

Revolutionary use:
geometry kernels can stop treating persistent topology identity, rebuild-safe
history, and corruption localization as brittle afterthoughts. The runtime
makes it possible to build kernels where topological truth survives aggressive
editing, can be queried in bulk, and can be replayed or certified after the
fact.

### Workflow, Node-Editor, and Visual-Editor Platforms

`worth-relational` should support:

- transactional graph editing with rollback and savepoints
- typed relation contracts for workflow legality
- graph introspection for editor/debug tooling
- durable CDC and historical inspection for collaborative and operational flows

Revolutionary use:
large editor and workflow products can get industrial graph-state mechanics
instead of spending years reinventing undo, identity, diff, and audit systems
around ad hoc node stores.

### Web and Data Platforms

`worth-relational` should support:

- transactional truth mutation with exact rollback
- durable CDC and subscriber recovery
- authoritative read fallback even when indexes lag or fail
- historical reads and auditability
- large-scale bulk query surfaces over canonical truth

Revolutionary use:
web and data systems can promote their source of truth from â€œapplication state
plus infrastructure accidentsâ€ to a branchable, replayable, audit-grade truth
runtime with first-class change feeds.

## Non-Goals

- turning the truth runtime into a reactive scheduler
- fusing truth storage and signal evaluation into one system
- baking domain-specific semantic meaning into the generic runtime core
- reducing identity to a single storage handle model
- treating MVCC, CDC, lineage, or bulk queries as optional polish

## Companion Documents

- [worth_relational_roadmap.md](/Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/plans/worth-relational/worth_relational_roadmap.md)
- [test-requirements.md](/Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/plans/worth-relational/test-requirements.md)
- [plans/engineering/worth_signal_vision.md](/Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/plans/engineering/worth_signal_vision.md)
- [plans/engineering/worth_runtime_bridge_vision.md](/Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/plans/engineering/worth_runtime_bridge_vision.md)

The truth runtime's MVCC, CDC, lineage, replay, and bulk query architecture are
what make the rest of the stack viable. If these are weak, every projection,
bridge, and reactive layer built on top of them becomes weaker too.
