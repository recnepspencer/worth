# WORTH Relational Future Roadmap

## Purpose

This document replaces the earlier buildout roadmap.

It tracks only the work that still remains for `worth-relational`.
Foundational architecture and Phase 8 are now shipped baseline. This roadmap
exists to define the remaining product milestones and the acceptance path that
must be satisfied before the runtime can be considered complete against
[test-requirements.md](./test-requirements.md).

The operating rule remains:

parallelize disposable work, serialize authority.

That rule still governs every remaining milestone:

1. immutable read, planning, validation, and preparation may scale out
2. authoritative truth mutation and publication remain serialized per selected
   authority scope and canonical
3. downstream consumption may scale out only over immutable published artifacts

### Query Milestone 9.17.1 Authority-Scope Clarification

Status: Closed on 2026-08-28. The executable owner flow is frozen in
[`BRANCH_LOCAL_MVCC.md`](../../crates/worth-relational/BRANCH_LOCAL_MVCC.md),
and the only component contract available to Milestone 9.17.2 is
[`OWNER_COMPONENT_PORT.md`](../../crates/worth-relational/OWNER_COMPONENT_PORT.md).
Runtime Bridge receives no Relational component mutation, currentness,
retention, or publication authority.

[Query Milestone 9.17.1](../WORTH-query/milestone-9.17.1.md) is the governing
cutover for Relational branch-local MVCC. From that cutover onward, “serialized
authority” in this roadmap means one exact selected branch reference has one
linear publication order. It does not authorize a runtime-global commit mutex,
broad `&mut RelationalRuntime` transaction lifetime, global publication actor,
or queue that prevents unrelated branches from progressing.

9.17.1 also separates immutable commits from mutable branch references and
makes one reference-selected immutable branch root the visibility
linearization boundary. Existing roadmap requirements for canonical commits,
history, replay, patch publication, storage-visible fallback, merge, and
inspection must migrate to that root and performed commit; they do not remain
parallel global authority paths.

The cutover begins by adding the Supply Chain certification world
through production schema/transaction facades, with an independent semantic
oracle and Court/Standard/Scale profiles. Forks must share the exact immutable
baseline without copying truth or commit envelopes; branch writes copy only
touched persistent regions; semantic state never crosses between siblings;
and stable inspection evidence distinguishes logical branch bytes from unique
physical authoritative bytes. The same world, named deltas, and oracle are
retained for later merge certification rather than replaced by a merge-only
fixture.

## Shipped Baseline

The roadmap no longer tracks the already-shipped foundation as future work.

The current shipped baseline includes:

- identity and storage foundations
- transactional commit authority
- savepoint and rollback substrate
- MVCC snapshots and version/history substrate
- deterministic patch, diagnostics, and replay publication
- same-commit graph creation with symbolic endpoint resolution for entity and
  relation creation inside one authoritative commit
- lineage, query, and index foundations
- proof-driven parallel preparation and post-commit scaling
- `worth-harness` parity and certification substrate

The shipped closeout reference for the latest major runtime milestone is
[milestone-6-closeout.md](./milestone-6-closeout.md).

## Roadmap Rules

The remaining roadmap is organized around product-complete milestones first and
certification programs second.

Rules for every remaining item:

- each milestone must describe the missing runtime/product capability, not only a test label
- each milestone must preserve authority-scope serialization, canonical
  observability, replay from canonical commit artifacts, and storage-visible
  fallback semantics
- each milestone must name the exact acceptance requirements in
  [test-requirements.md](./test-requirements.md)
- the roadmap uses `test-requirements.md` as the authoritative source for what each named acceptance test demands
- no milestone is complete until both implementation and acceptance requirements are closed

## Milestone 1: CDC and Subscriber Recovery

Status: Closed on 2026-03-13. See
[milestone-1-closeout.md](./milestone-1-closeout.md).

### Goal

Finish the stream product surface so CDC is not just emitted patch data, but a
durable downstream consumption contract.

### Must Ship

- subscriber-facing recovery model
- explicit resume/checkpoint semantics
- deterministic resumed publication order
- explicit subscriber-visible failure classes
- replay-to-CDC consistency as a runtime contract
- protocol-facing streaming patch feeds rather than internal-only patch emission
- diagnostics for resume tokens, checkpoints, recovery decisions, and stream rejection/failure

### Must Preserve

- commit-native canonical patch order
- coherent publication
- replay from canonical commit artifacts
- no abandoned savepoint or rollback work appearing in published CDC
- no scheduler-shaped stream semantics

### Explicit Boundary

Milestone 1 includes schema/version compatibility checking and explicit
subscriber-visible failure for incompatible checkpoints, recovery plans, or
runtime/schema combinations.

Milestone 1 does not include live schema-version transition inside an already
running subscriber contract. In other words, this milestone must fail
structurally on incompatible schema boundaries; it does not yet promise
mid-stream schema renegotiation, dual-schema CDC emission, or seamless
subscriber continuation across runtime schema changes.

### Acceptance Requirements

This milestone is complete only when the implementation satisfies the following
named requirements from
[test-requirements.md](./test-requirements.md):

- `Diff/CDC truth parity test`
- `Hostile commit/replay equivalence test`
- `Durable recovery and schema mismatch test`
- `Snapshot-stable concurrent read vs hot rewrite test`

The roadmap intentionally references the requirements doc directly here. The
requirements doc remains the source of truth for the exact scenario,
verification output, and pass condition those tests impose.

## Milestone 2: Relational Aspect Semantics

Status: Closed on 2026-03-19. See
[milestone-2-closeout.md](./milestone-2-closeout.md).

### Goal

Finish the truth-layer aspect system so aspects are first-class relational
semantics for entities, relations, committed diffs, projections, historical
reads, and bulk queries, rather than only payload-derived change labels.

### Must Ship

- stable aspect identity at the truth layer
- explicit entity aspect semantics
- explicit relation aspect semantics
- aspect participation in committed diff and CDC output
- aspect-aware projection and bulk-query surfaces
- aspect-aware historical read surfaces
- aspect-aware and lineage-aware historical read surfaces where identity evolution and change surface both matter
- schema and kind driven aspect declarations where the runtime needs more than raw payload-key extraction
- diagnostics and artifacts that expose aspect-level committed change semantics canonically

### Must Preserve

- canonical patch ordering
- coherent publication
- deterministic replay from canonical commit artifacts
- no hidden mutation during reads
- authoritative storage-visible fallback semantics
- no scheduler-shaped observability

### Acceptance Requirements

This milestone is complete only when the implementation satisfies the following
named requirements from
[test-requirements.md](./test-requirements.md):

- `Diff/CDC truth parity test`
- `Bulk query and traversal stress truth test`
- `Hostile commit/replay equivalence test`
- `Topology identity survival test`
- `Netlist rewiring identity and history test`

## Milestone 3: Structural Identity, Introspection, and Historical Inspection

### Goal

Finish the runtime surfaces that let consumers inspect structure, recent
mutation, and retained history as first-class truth capabilities rather than
ad hoc debug utilities.

### Must Ship

- explicit structural identity surfaces
- structural hashing or fingerprint surfaces that hosts can use without collapsing structural policy into storage identity
- graph introspection APIs over entities, relations, kinds, counts, connectivity classes, and change boundaries
- recent-mutation and transaction-surface introspection
- graph time-travel surfaces for retained or reconstructible historical truth
- explicit retention and reclaim product surfaces rather than only internal retention machinery
- historical inspection surfaces that compose structural identity, lineage, and aspects coherently

### Must Preserve

- permanent separation between storage identity, lineage identity, and structural identity
- canonical observability for historical reads
- explicit reclaim behavior under pinned readers and retained history
- no hidden mutation during inspection or time travel
- replay from canonical commit artifacts

### Acceptance Requirements

This milestone is complete only when the implementation satisfies the following
named requirements from
[test-requirements.md](./test-requirements.md):

- `Hostile commit/replay equivalence test`
- `Snapshot pinning and reclaim correctness test`
- `Lineage/correspondence hardening test`
- `Bulk query and traversal stress truth test`
- `Topology identity survival test`
- `Netlist rewiring identity and history test`

## Milestone 4: Relation Integrity and Schema Contracts

Status: Closed on 2026-03-21. See
[milestone-4-closeout.md](./milestone-4-closeout.md).

### Goal

Finish the runtime contract layer for relation legality, schema-defined
invariants, and typed relation ergonomics so the runtime is enforcing graph
truth rules instead of merely storing graph-shaped data.

### Must Ship

- relation integrity as an explicit runtime capability
- schema-defined invariants for relation legality, cardinality, symmetry, uniqueness, and deletion effects where the schema declares them
- typed node and relation contract richness sufficient for industrial schema design
- integrity validation hooks that participate in authoritative commit
- failure diagnostics for relation-contract and invariant violations
- schema surfaces that remain generic while still being strong enough for topology, connectivity, workflow, and IR-style truth graphs

### Must Preserve

- serialized authority
- canonical failure reporting
- no partial publication after integrity failure
- schema-defined semantics instead of hardcoded domain semantics
- replay and recovery consistency for accepted and rejected commits

### Acceptance Requirements

This milestone is complete only when the implementation satisfies the following
named requirements from
[test-requirements.md](./test-requirements.md):

- `Savepoint rollback fracture test`
- `Hostile commit/replay equivalence test`
- `Durable recovery and schema mismatch test`
- `Missing-twin / nonmanifold corruption localization test`

## Milestone 5: Schema Evolution, CDC Contract Evolution, and Schema Reconciliation

### Goal

Finish live schema evolution as an explicit runtime and CDC capability so
schema changes can participate in authoritative truth, replay, recovery, and
subscriber contracts without collapsing into ad hoc host coordination.

Additionally, finish classified schema reconciliation so that schema
divergence across branches, tenants, or evolution boundaries is handled by
explicit policy rather than blanket rejection or silent corruption.

### Must Ship

- explicit schema evolution model for authoritative runtimes
- schema strata classification so structural, value-domain, identity,
  behavioral, publication, and subscriber-contract change surfaces are not
  collapsed into one vague schema-diff bucket
- canonical publication of schema transition boundaries
- subscriber-facing schema transition semantics for checkpoint/resume and
  active stream continuation
- typed transition policies such as fail, require renegotiation, or continue
  under an explicitly compatible schema bridge where supported
- recovery and replay semantics that preserve schema transition boundaries as
  canonical artifacts
- diagnostics for schema transition decisions, rejection, and subscriber
  contract renegotiation
- compatibility rules that distinguish:
  - incompatible schema mismatch
  - resumable compatible evolution
  - transitions that require a new subscriber contract
- descriptor semantics versioning, canonicalization versioning, and explicit
  invalidation rules for continuation/reconciliation descriptors
- explicit verification modes that separate normal recovery verification from
  audit-only recomputation and corruption diagnosis

#### Schema Reconciliation

Schema reconciliation provides classified handling of schema divergence during
merge, deployment, and tenant migration. The runtime must classify every schema
difference into a reconciliation category and apply the appropriate policy
rather than treating all incompatibilities as fatal.

Required reconciliation categories:

- **additive reconciliation**: new aspects, fields, entity types, or relation
  types introduced on one side but absent on the other must be
  auto-reconcilable with explicit default or null semantics; this covers the
  common case of one branch or tenant evolving the schema while another has not
  yet adopted the change
- **narrowing reconciliation**: aspects or fields removed on one side but still
  present on the other must be handled by caller-supplied policy (prefer
  richer, prefer target, or reject); the runtime must not silently drop or
  silently preserve without an explicit policy decision
- **type-incompatible rejection**: same-named aspect or field with incompatible
  type on each side must be classified as a genuine conflict and fail-explicit
  with structured diagnostics; the runtime must not attempt implicit coercion
- **structural-incompatible rejection**: fundamentally different relation
  topology or entity modeling between sides must be classified as structural
  conflict and fail-explicit with structured diagnostics

Required reconciliation behaviors:

- schema reconciliation must participate in the merge commit pipeline when
  branches carry divergent declared schemas
- reconciliation decisions must be emitted as canonical artifacts for replay,
  diagnostics, and audit
- reconciliation must compose with Milestone 7C merge execution so that
  data-level merge and schema-level reconciliation are resolved in the same
  transactional commit
- reconciliation must support caller-supplied policies rather than hardcoded
  resolution strategies
- reconciliation must never silently drop schema structure, silently coerce
  types, or silently adopt incompatible topology

#### Motivation

Schema reconciliation is especially critical for web development with custom
workflows, per-tenant customization, plugin/extension systems, and any domain
where schema divergence is the norm rather than an exceptional event. Without
classified reconciliation, fail-explicit rejection on schema mismatch creates
unacceptable friction for deployments, tenant migrations, and feature rollouts
in these domains.

### Must Preserve

- serialized authority for schema-affecting truth mutation
- canonical CDC and patch ordering across schema boundaries
- replay from canonical commit artifacts
- no hidden host-side schema repair during resume or recovery
- explicit failure instead of silent drift when transition semantics are not
  supported
- reconciliation decisions as canonical artifacts, not silent internal behavior

### Acceptance Requirements

This milestone is complete only when the implementation satisfies the
following named requirements from
[test-requirements.md](./test-requirements.md),
and any additional schema-evolution-specific certification requirements added
there:

- `Diff/CDC truth parity test`
- `Hostile commit/replay equivalence test`
- `Durable recovery and schema mismatch test`

Additionally, this milestone must add and satisfy an explicit schema-evolution
CDC certification requirement if
[test-requirements.md](./test-requirements.md)
does not yet contain one.

Additionally, this milestone must add and satisfy an explicit schema
reconciliation certification requirement covering additive, narrowing,
type-incompatible, and structural-incompatible classification with
policy-driven resolution.

## Milestone 6: Lineage and Correspondence Completion

Status: Closed on 2026-03-23. See
[milestone-6-closeout.md](./milestone-6-closeout.md).

### Goal

Finish identity-evolution semantics so lineage is authoritative truth, not just
a base graph plus advisory metadata.

### Must Ship

- explicit replace/split/merge-like lineage event coverage
- authoritative promotion flow from advisory correspondence
- invalid and ambiguous correspondence rejection
- branch-local identity-evolution isolation
- historical ID resolution across legitimate lineage chains
- lineage graph as a named query surface, not only internal bookkeeping
- historical resolution ergonomics that downstream systems can consume directly
- lineage-aware committed change surfaces where identity evolution must appear

### Must Preserve

- permanent separation between storage identity and lineage identity
- advisory correspondence stays non-authoritative until explicit promotion
- canonical serialized lineage finalization
- branch-local lineage does not leak across branches
- deterministic replay and diagnostics of lineage behavior

### Acceptance Requirements

This milestone is complete only when the implementation satisfies the following
named requirements from
[test-requirements.md](./test-requirements.md):

- `Lineage/correspondence hardening test`
- `Topology identity survival test`
- `Netlist rewiring identity and history test`
- `Hostile commit/replay equivalence test`

## Milestone 6.5: Invariant Completion and Custom Invariant Support

### Goal

Complete the native invariant suite so the runtime enforces the full set of
domain-agnostic structural truth rules, and ship the custom invariant
extensibility surface so domain-specific structural invariants can participate
in the same commit-time and publication-boundary enforcement pipeline as native
rules.

This milestone exists because the geometry kernel — the primary consumer of
this runtime — requires both domain-agnostic topological invariants (acyclicity,
cardinality minimum) and domain-specific structural invariants (manifold edge,
orientation consistency, face loop closure) that cannot be expressed as signal
computations. These must be enforced at the relational layer, not deferred to
downstream consumers.

### Must Ship

#### New Native Invariants

- **cardinality minimum**: schema-declared minimum relation count per endpoint,
  enforced at publication boundary; entity creation with zero relations is
  permitted during construction, but publication requires satisfaction of
  declared minimums
- **acyclicity**: schema-declared cycle prohibition for directed relation kinds,
  enforced at commit boundary; Milestone 6.5 uses commit-time reachability
  search from the newly introduced relation target back toward the source over
  the candidate relation-kind graph, with explicit counters and complexity
  contracts. The roadmap must remain honest that worst-case breadth can equal
  the full candidate relation-kind graph unless a future milestone introduces a
  persistent incremental cycle-detection structure
- **payload schema validation**: schema-declared JSON structure contracts per
  entity and relation kind, enforced at commit boundary; the runtime validates
  that committed payloads conform to declared field presence, type, and
  constraint rules
- **partition isolation**: schema-declared cross-partition prohibition for
  specific relation kinds, enforced at commit boundary; prevents relations of
  declared kinds from connecting entities in different partitions
- **connectivity minimum**: schema-declared reachability requirement from
  entities of one kind to entities of another kind, enforced at publication
  boundary; prevents orphaned subgraphs that violate declared dependency
  contracts

#### Custom Invariant Extensibility

- a public `CustomInvariantRule` trait that domain-specific invariants implement
  to participate in the existing invariant execution pipeline
- custom invariants must declare their execution point (commit boundary or
  publication boundary), cost class, and invariant group membership
- custom invariants receive the same structural authority surfaces as native
  invariants, but through a dedicated custom execution context and scope planner
  that expose touched records, relation endpoints, kind counts, payload fields,
  and session-budgeted bounded graph traversal
- custom invariants must not receive access to the signal graph or computed
  values; the boundary between structural invariants and derived invariants is
  enforced by the API surface
- custom invariant failures participate in the same typed
  `InvariantViolation` and diagnostics pipeline as native failures
- custom invariants must be registered through a frozen runtime/schema registry
  that separates semantic rule identity from operational metadata so historical
  artifacts remain interpretable across binary versions
- custom invariants must not become a type-erasure escape hatch; the framework
  may erase executable storage at registration boundaries, but each lowered
  packet must own the exact executable/scope pairing produced during planning so
  no runtime route-and-downcast contract remains representable
- the planner and evaluator must handle custom invariants without semantic
  special-casing; they flow through the same planning, lowering, and packet
  execution pipeline as native rules

### Must Preserve

- serialized authority for invariant-affecting truth mutation
- canonical failure reporting for all invariant types (native and custom)
- no partial publication after invariant failure
- explicit separation between committed authority, publication eligibility, and
  published observability for publication-boundary invariant failures
- replay and recovery consistency for commits validated against custom invariants
- the existing native invariant performance characteristics must not degrade
  when custom invariants are registered
- custom invariant panics must not crash the runtime; custom rule evaluation
  must be isolated via `catch_unwind` or equivalent boundary

### Acceptance Requirements

This milestone is complete only when the implementation satisfies the following
named requirements from
[test-requirements.md](./test-requirements.md):

- `Hostile commit/replay equivalence test`
- `Missing-twin / nonmanifold corruption localization test`
- `Savepoint rollback fracture test`

Additionally, this milestone must add and satisfy an explicit invariant
extensibility certification requirement covering:

- custom invariant registration, evaluation, and failure reporting parity with
  native invariants
- packet-owned custom scope/executable pairing with no `Any`-style semantic
  escape hatch at the framework boundary
- acyclicity enforcement under hostile cycle-inducing commit sequences
- cardinality minimum enforcement at publication boundary with deferred
  construction semantics
- payload schema validation with structured rejection diagnostics
- partition isolation enforcement with cross-partition relation rejection
- connectivity minimum enforcement at publication boundary
- explicit committed-but-unpublished semantics for publication-boundary failures

## Milestone 7: Merge-Ready History, Merge Ontology, and Merge Execution

This milestone is intentionally split so the roadmap stays honest about what is
already structurally supported versus what may still be missing as product
behavior.

### Milestone 7A: Merge-Ready History Certification

#### Goal

Prove that merge-ready history shape is operationally real across replay,
durability, diagnostics, and ancestry reasoning.

#### Must Ship

- ordered multi-parent commit-envelope fixtures as accepted certification inputs
- ordered parent persistence through durability
- replay handling for ordered parent lists
- diagnostics and ancestry reasoning that remain correct on ordered parents
- explicit evidence that observable surfaces do not quietly assume "single parent or none"

#### Acceptance Requirements

This sub-milestone is complete only when the implementation satisfies the
following named requirement from
[test-requirements.md](./test-requirements.md):

- `Merge-ready history shape test`

### Milestone 7B: Merge Artifact Ontology and Reconciliation Semantics

#### Goal

Define the canonical merge authority model before execution exists so merge
semantics are explicit, replayable, durable, policy-governed, and diagnostics-
rich rather than being rediscovered procedurally inside execution code.

#### Must Ship

- merge conflict classification and structured diagnostics
- canonical merge artifact ontology for:
  - identity matching and reconciliation candidates
  - typed conflict classification
  - typed reconciliation decisions
  - causal frontier / causal dependency evidence
  - schema-policy resolution evidence
  - lowered merge plans
  - merge explanation and diagnostics surfaces
- causal commit metadata as a real authority/path artifact, not helper metadata
- schema-declared merge policy declarations and canonical policy-resolution
  artifacts
- replay/durability/publication-facing merge artifact shapes sufficient for
  later execution and certification
- fixture-driven and harness-driven certification scaffolding that proves the
  runtime can carry merge semantics canonically before authoritative execution
  is enabled

#### Explicit Merge/Reconciliation Requirements

Milestone 7 must be treated as more than "make merge commits exist."

The roadmap needs to be explicit that several merge/reconciliation behaviors are
not fully supported yet but are architecturally required for a production-grade
relational runtime. These are not optional refinements to append later if
someone notices them. They are part of the real merge product surface.

Required future merge behaviors include:

- persistent identity matching across branches when corresponding records are
  not the same raw record id
- aspect-aware merge into an existing target record when one branch carries
  richer declared aspect structure than another
- deletion/removal semantics as a first-class merge result
- relation endpoint rewiring merge semantics when one branch changes topology
  and another changes payload/aspects
- typed three-way merge semantics over base/source/target for record state,
  aspect deltas, and relation structure
- policy-driven conflict resolution beyond honest conflict detection and
  fail-closed rejection
- partial mergeability where non-conflicting regions can converge while
  conflicting regions remain isolated
- rich merge explanation surfaces that can answer why a target branch ended up
  with a particular adopted, rejected, or reconciled aspect shape

One especially important required behavior is this:

- merging richer aspect structure into an existing poorer target structure must
  eventually be modeled as explicit reconciliation semantics, not silently
  collapsed into ordinary introduction, ordinary replacement, or lineage
  continuity alone

The roadmap is intentionally explicit here because the likely failure mode would
otherwise be one of:

- duplicating logically corresponding records instead of reconciling them
- treating richer aspect structure as unrelated introduction
- overloading lifecycle or lineage continuity with merge identity meaning
- bolting on later heuristic matching without a first-class identity contract

Milestone 2 is only a prerequisite for this work. Milestone 7B is where the
runtime must become explicit about these merge/reconciliation behaviors as real
product requirements, with canonical artifacts and lowered plans that later
execution must consume rather than reinterpret.

#### Causal Commit Metadata

Branches are structurally equivalent to partitioned nodes in a distributed
system. To make merge semantically precise, the runtime must carry causal
metadata on commits so the merge pipeline can formally distinguish between
causally independent operations (safe to auto-merge under declared policy) and
causally dependent operations (require ordered resolution).

Required causal metadata capabilities:

- each commit must carry a logical causal timestamp per branch, not wall-clock
  time, so the runtime can determine happens-before relationships across
  branches without relying on synchronized clocks
- merge must be able to prove whether two branches' commits are causally
  independent (concurrent, safe to reconcile under declared policy) or causally
  dependent (one must be ordered after the other)
- CDC subscribers must be able to request causally consistent snapshots, meaning
  every commit they observe has also had all of that commit's causal
  predecessors observed
- merge commits must record the causal frontier of both source branches at merge
  time as canonical artifacts for replay and diagnostics

Causal metadata does not replace structural conflict detection. It adds a
second axis: two commits may be structurally non-conflicting but causally
dependent, or structurally conflicting but causally independent. Milestone 7B
must define and persist this causal reasoning canonically so Milestone 7C can
consume it rather than recompute it ad hoc during execution.

#### Schema-Declared Merge Policies (CRDT-Style)

Policy-driven conflict resolution should not be implemented as arbitrary
caller-supplied closures. Instead, the schema must declare per-aspect merge
semantics from a classified set of conflict-free resolution strategies. These
are the relational runtime's equivalent of CRDTs (Conflict-free Replicated Data
Types), adapted for the branch-and-merge model rather than the eventual
consistency model.

Required schema-declared merge semantics:

- **FailOnConflict**: default behavior; concurrent modifications to the same
  aspect on different branches produce a structural conflict that must be
  resolved manually or rejected
- **LastWriterWins**: concurrent modifications resolve by causal timestamp;
  requires causal metadata to be meaningful, otherwise falls back to
  fail-on-conflict
- **MonotonicCounter**: numeric aspect values merge by summation of per-branch
  deltas from the common ancestor; each branch's contribution is independently
  valid and merge produces the combined total
- **AdditiveSet**: set-valued aspects merge by observed-remove semantics; adds
  win over concurrent removes and the merged result contains any element added
  by either branch unless both branches independently removed it
- **PreferRicher**: when one branch carries richer declared aspect structure
  than the other, the richer structure wins; this is the explicit formal
  resolution for the "richer aspect into poorer target" behavior described
  above
- **Custom**: caller-supplied merge policy registered through the schema
  registry with the same freeze-at-construction and deterministic-descriptor
  rules as custom invariants

Merge policy enforcement rules:

- merge policies must be declared per-aspect at schema level, not supplied ad
  hoc at merge time; the schema is the authority for how conflicts resolve
- merge policies must compose with causal metadata; LastWriterWins is only valid
  when causal timestamps are present, and the runtime must reject the
  declaration if causal metadata is not enabled
- merge policies must be recorded in canonical merge artifacts so replay can
  verify that the same policy was applied
- merge policies that are conflict-free by construction (MonotonicCounter,
  AdditiveSet) must be auto-resolved without caller intervention; the merge
  pipeline must not present these as conflicts
- merge policies must not bypass the invariant pipeline; after auto-resolution,
  the merged state must still satisfy all declared invariants at the merge
  commit boundary

#### Must Preserve

- single serialized authority for final truth commit
- canonical observability and replay
- coherent publication semantics for merge-bearing artifacts
- explicit separation between merge ontology/planning and merge execution
- no host-side heuristic merge identity or policy logic becoming accidental
  authority
- explicit failure rather than silent semantic drift

#### Acceptance Requirements

This sub-milestone is complete only when:

- the roadmap is paired with an explicit merge-ontology certification test or suite if
  [test-requirements.md](./test-requirements.md)
  does not already contain one
- merge-bearing histories can persist, replay, and recover the canonical merge
  artifact surfaces without authoritative merge execution enabled
- `Hostile commit/replay equivalence test` remains satisfied for histories
  carrying merge ontology artifacts and causal metadata
- `Durable recovery and schema mismatch test` remains satisfied for histories
  carrying merge ontology artifacts and causal metadata
- `Merge-ready history shape test` remains satisfied with the new merge
  ontology artifacts present

### Milestone 7C: Authoritative Merge Execution

#### Goal

Finish authoritative multi-parent merge commit creation as a first-class
runtime feature by executing only against the canonical merge ontology,
conflict taxonomy, causal metadata, and lowered merge plans established in
Milestone 7B.

#### Must Ship

- authoritative multi-parent merge commit execution
- deterministic parent ordering at merge commit creation time
- execution that consumes lowered merge plans rather than rediscovering merge
  semantics during the hot path
- replay semantics for merge commits
- durable publication semantics for merge commits
- branch-head advancement semantics for successful merges
- invariant-boundary enforcement over merged results
- explicit typed failure for unmergeable or policy-rejected merge requests

#### Must Preserve

- single serialized authority for final truth commit
- canonical observability and replay
- coherent publication
- explicit failure rather than partial merge truth

#### Acceptance Requirements

This sub-milestone is complete only when:

- the roadmap is paired with an explicit merge-execution certification test or suite if
  [test-requirements.md](./test-requirements.md)
  does not already contain one
- `Hostile commit/replay equivalence test` is satisfied for merge-bearing histories
- `Durable recovery and schema mismatch test` is satisfied for merge-bearing histories
- `Merge-ready history shape test` remains satisfied on real merge-produced histories, not only fixtures

### Milestone 7D: Deletion And Topology Merge Execution

#### Goal

Extend merge from the admitted non-deletion 7C subset into explicit deletion
and topology merge semantics without weakening the proof chain, serialized
authority model, replay parity, or diagnostics rigor established in Milestone
7C.

#### Must Ship

- explicit deletion execution ontology rather than a generic blocked-deletion
  bucket
- explicit topology execution ontology for relation rewiring and topology
  escalation surfaces
- typed mapping from conflict classification into:
  - executable merge classes
  - explicitly non-executable denial classes
- promotion of the safest first deletion execution class
- durable and replay-stable diagnostics that distinguish:
  - executable deletion truth
  - blocked deletion truth
  - topology-local rewire escalation
  - topology-region denial
- branch-local deletion and topology certification over real authored histories,
  not synthetic patch-only fixtures

#### Must Preserve

- merge execution must continue to consume proof-carrying prepared/lowered
  merge artifacts only
- generic commit apply must not rediscover deletion semantics, topology
  semantics, or relation continuity
- the shared authoritative commit pipeline must remain shared
- fail-closed behavior for every non-admitted deletion or topology class
- replay, durability, and diagnostics parity for merge-bearing histories

#### Explicit 7D Scope Rules

Milestone 7D is not "make deletes work in merge somehow."

It must be explicit about which classes are:

- represented in the ontology
- executable in this milestone
- intentionally fail-closed in this milestone

Required ontology coverage includes:

- deletion classes:
  - source-deleted / target-live
  - source-live / target-deleted
  - deleted-on-both-sides
  - deleted-vs-modified
  - deleted-vs-rewired
- topology classes:
  - relation-local endpoint rewiring
  - rewiring escalated by current milestone policy
  - true topology-region conflict

One especially important rule must stay explicit:

- relation-local rewiring evidence must not be flattened into generic topology
  conflict before the ontology has a chance to classify it

That distinction is required so future topology execution can be widened
honestly instead of by reinterpreting old denial buckets.

#### Acceptance Requirements

This sub-milestone is complete only when:

- at least one deletion class is executable end to end with replay and recovery
  parity
- non-executable deletion classes retain typed denial and recovery-stable
  planning artifacts
- topology-bearing merge requests retain typed denial and recovery-stable
  planning artifacts
- replay and durability continue to satisfy:
  - `Hostile commit/replay equivalence test`
  - `Durable recovery and schema mismatch test`
  - `Merge-ready history shape test`
- the roadmap is paired with an explicit deletion/topology merge certification
  test or suite if
  [test-requirements.md](./test-requirements.md)
  does not already contain one

#### Must Not Fake

Milestone 7D must not:

- silently broaden one-sided deletion execution without an explicit policy
  surface
- silently treat rewiring as ordinary reconciliation
- silently treat topology-region conflict as the same thing as relation-local
  rewiring
- move merge meaning back into generic commit execution helpers

7D is the point where deletion and topology become first-class merge truth, not
special-case leftovers.

### Milestone 7E: Collaboration Merge Hardening

#### Goal

Finish the retained collaboration substrate for branch and merge truth so
published merge posture is reconstructable, replayable, support-inspectable,
and readmission-preparable from retained artifacts alone rather than from live
planner reconstruction, while lowering shared boundary meaning into
`worth-foundational` native basis, canonical, locator, compatibility,
readmission, and support surfaces.

#### Must Ship

- retained merge branch-basis artifacts
- foundational-native basis/readmission lowering for retained branch basis
- explicit merge request vocabulary beyond raw branch pairing
- foundational-native merge vocabulary lowering where honest
- retained merge proof packets rather than summary-only publication
- foundational-native canonical basis and locator lowering
- retained correspondence witnesses
- retained schema reconciliation witnesses
- retained strategy and policy witnesses
- compatibility and readmission-preparation witnesses built on foundational
  shared boundary grammar
- support inspection witnesses built on foundational support posture grammar
- replay and recovery parity for all new retained merge collaboration artifacts

#### Must Preserve

- single serialized authority for final truth commit
- canonical observability and replay
- explicit separation between relational domain truth and foundational shared
  boundary grammar
- no host-side heuristic correspondence, schema, or policy logic becoming
  accidental authority
- explicit typed denial rather than generic merge-failure flattening

#### Acceptance Requirements

This sub-milestone is complete only when:

- retained merge collaboration witnesses round-trip through durable publication
  and recovery canonically
- foundational-lowered basis, canonical, locator, compatibility, readmission,
  and support surfaces round-trip from retained relational truth without live
  reconstruction
- `Hostile commit/replay equivalence test` remains satisfied for histories
  carrying retained collaboration witnesses
- `Durable recovery and schema mismatch test` remains satisfied for histories
  carrying retained collaboration witnesses
- `Merge-ready history shape test` remains satisfied for histories carrying
  retained collaboration witnesses
- `Lineage/correspondence hardening test` is widened or paired with an explicit
  merge-correspondence witness certification requirement
- the roadmap is paired with an explicit retained collaboration merge
  certification requirement if
  [test-requirements.md](./test-requirements.md)
  does not already contain one

## Milestone 8: Parallel Read, Bulk Mutation, and Scale Query Completion

### Goal

Finish the scale side of the vision so the runtime can honestly claim
first-class industrial bulk query and bulk mutation behavior, not just strong
commit-time preparation.

### Must Ship

- deterministic packetized planning for bulk reads over immutable snapshots
- explicit parallel read execution where supported
- large-surface bulk traversal/query APIs as primary surfaces
- low-level bulk mechanical primitives for relation traversal and scans, with semantic layering remaining above them
- parity between index-assisted reads and authoritative storage fallback
- read-side locality, partition, and work-packet metrics
- partition-aware query surfaces and partitioning hints where the runtime can expose them generically
- memory stability guarantees strong enough for long-lived high-scale workloads
- bulk mutation as a scale product surface rather than only repeated single-write composition
- graph introspection surfaces strong enough to inspect read and mutation scale behavior
- snapshot-stable read behavior under active mutation and high churn

### Must Preserve

- immutable snapshot semantics
- no hidden read mutation or repair
- non-authoritative derived indexes
- authoritative storage fallback always available
- deterministic observable query order

### Acceptance Requirements

This milestone is complete only when the implementation satisfies the following
named requirements from
[test-requirements.md](./test-requirements.md):

- `Bulk query and traversal stress truth test`
- `Index non-authority corruption test`
- `Deterministic observability under hostile scheduling test`
- `Snapshot-stable concurrent read vs hot rewrite test`

## Milestone 8.5: Extensible Commit Strategies

### Goal

Replace the hard-coded mutation-only commit model with an extensible commit
strategy system so that domain-specific commit types — intent reconciliation,
constraint solving, workflow advancement, bridge-mediated evaluation — compose
with the full commit pipeline (transaction, invariant, merge, replay, publication)
without modifying the runtime.

### Adversarial Constraint

A caller-supplied commit strategy that panics, deadlocks, produces
non-deterministic mutations, or silently violates schema constraints must not
corrupt the runtime's authoritative state, break replay parity, or bypass the
invariant pipeline. The commit pipeline must treat the strategy as an untrusted
effect producer: its output is validated, its failures are contained, and its
replay determinism is verified — not assumed.

### Motivation

The mutation-only commit model ("here is what changed") does not capture
higher-level semantic goals. Many real-world operations are strategy-driven:

- A CAD assembly constraint says "these faces must be flush" → constraint solver
  produces mutations
- A deployment manifest says "this service must have 3 replicas" → intent differ
  produces mutations
- A compliance rule says "this entity must satisfy GDPR" → policy engine
  produces mutations
- A signal-relational bridge says "evaluate the signal graph and reflect results
  into relational state" → bridge evaluator produces mutations

These all share the same commit lifecycle (open transaction → produce effects →
validate invariants → commit with causal metadata → notify subscribers) but
differ only in how effects are produced. Per Architectural Law 28, the shared
lifecycle is the abstraction and the effect-production strategy is the parameter.

### Prerequisites

- Milestone 6.5 (invariant pipeline) for validating strategy-produced mutations
- Milestone 7C (merge execution) for merging branches with strategy-produced
  commits
- Causal commit metadata (from Milestone 7B) for causal ordering of
  strategy-produced commits

### Must Ship

#### Commit Strategy Trait

- a typed trait (or equivalent extension point) that allows callers to define
  custom commit strategies with a declared input type and a lowered mutation plan
  as output
- the strategy receives a read-only view of current authoritative state and
  produces a mutation batch; it does not have write access to the runtime
- strategy registration must be schema-level and freeze-at-construction, not
  ad-hoc; a strategy registered after the first commit is rejected
- each registered strategy must carry a deterministic descriptor so replay can
  verify strategy identity across sessions

#### Strategy Containment

- a strategy that returns an error must cause the transaction to fail-explicit
  without partial mutation; the runtime must not apply a partial mutation batch
  from a failed strategy
- a strategy that produces mutations violating schema constraints must be
  rejected at the invariant pipeline boundary, not at the strategy boundary;
  the strategy is not trusted to self-validate
- a strategy that produces non-deterministic output (different mutations for
  the same input state) must be detectable through replay parity verification;
  the runtime must record the strategy input alongside the mutation output so
  replay can re-invoke the strategy and compare
- strategies must not hold references into the runtime across transaction
  boundaries; the strategy is invoked, produces output, and returns — it does
  not persist state between commits

#### Strategy-Aware Merge

- merge must carry strategy metadata on commits so the merge pipeline can
  distinguish commits produced by different strategies
- when two branches carry commits from different strategies targeting the same
  records, merge must classify the conflict using both record-level and
  strategy-level metadata
- strategy-level conflict classification must be a distinct category from
  record-level conflicts, with structured diagnostics that name the conflicting
  strategies and their declared intents

#### Reference Implementation: Intent Reconciliation

- the milestone must ship an intent reconciliation strategy as a concrete
  implementation that demonstrates the full strategy lifecycle
- intent reconciliation accepts a desired-state declaration, diffs against
  current state, and produces a mutation batch
- intent reconciliation must be idempotent: applying the same intent to a state
  that already satisfies it must produce no mutations and succeed
- intent reconciliation must compose with CRDT-style merge policies: if an
  aspect has a declared merge policy and an intent targets that aspect, the
  reconciliation must respect the merge policy

#### Strategy Persistence and Replay

- persisted commit artifacts must record the strategy identity, the strategy
  input, the produced mutation batch, and the invariant validation results
- replay must re-invoke the registered strategy with the recorded input and
  verify that it produces the same mutation batch; divergence is a replay
  parity violation
- strategy inputs must be canonical artifacts with digest bases, not opaque
  blobs

### Must Preserve

- serialized authority for truth mutation (strategies produce mutations through
  the existing commit authority, not a parallel path)
- canonical observability and replay
- coherent publication
- invariant enforcement at commit boundary (strategies do not bypass invariants)
- explicit failure rather than partial strategy execution

### Acceptance Requirements

This milestone is complete only when:

- at least two distinct commit strategies (intent reconciliation and one other)
  produce canonical commit envelopes verified by the existing replay
  certification pipeline
- strategy containment is demonstrated: a deliberately failing strategy does not
  corrupt runtime state
- strategy replay parity is verified: replaying a strategy-produced commit
  re-invokes the strategy and confirms identical output
- strategy-aware merge conflict classification is demonstrated under hostile
  branching scenarios with commits from different strategies targeting the same
  records
- intent reconciliation idempotency is verified as a specific case
- `Hostile commit/replay equivalence test` is satisfied for strategy-bearing
  histories
- the roadmap is paired with an explicit extensible-commit-strategy
  certification test if
  [test-requirements.md](./test-requirements.md)
  does not already contain one

## Milestone 9: Generic Certification Program

### Goal

Run the full generic truth-grade certification program after the remaining
product milestones are implemented.

### Scope

This milestone is not for discovering missing features. It is for proving the
completed runtime under the hostile scenarios already defined in
[test-requirements.md](./test-requirements.md).

### Acceptance Requirements

This milestone is complete only when all ten generic named requirements in
[test-requirements.md](./test-requirements.md)
are satisfied:

- `Hostile commit/replay equivalence test`
- `Savepoint rollback fracture test`
- `Snapshot pinning and reclaim correctness test`
- `Deterministic observability under hostile scheduling test`
- `Index non-authority corruption test`
- `Diff/CDC truth parity test`
- `Lineage/correspondence hardening test`
- `Merge-ready history shape test`
- `Bulk query and traversal stress truth test`
- `Durable recovery and schema mismatch test`

Each certification run must emit canonical machine-checkable artifact bundles,
not only human-readable logs, exactly as required by the requirements doc.

## Milestone 9: Domain Certification Program

### Goal

Prove that the generic runtime is actually fit for the stated target domains.

### Acceptance Requirements

This milestone is complete only when all four domain-specific named
requirements in
[test-requirements.md](./test-requirements.md)
are satisfied:

- `Topology identity survival test`
- `Missing-twin / nonmanifold corruption localization test`
- `Netlist rewiring identity and history test`
- `Snapshot-stable concurrent read vs hot rewrite test`

The requirements doc remains the authoritative source for the exact CAD and
chip scenarios, verification output, and pass conditions these tests impose.

## Per-Milestone Format

For consistency and readability, every milestone in this roadmap uses the same
shape:

- `Goal`
- `Must Ship`
- `Must Preserve`
- `Acceptance Requirements`

The acceptance section deliberately references
[test-requirements.md](./test-requirements.md)
explicitly rather than trying to restate those tests in weaker or shorter form.

## Bridge Independence

Relational must not depend on `worth-runtime-bridge`. The Bridge links
Relational and Signal, so the Bridge depends on Relational.
[Bridge Milestone 20](../WORTH-runtime-bridge/milestone-20.md) moves the
adapter out of `presentation/bridge` and adds the Bridge-free change source
API that Relational owns.

## Completion Standard

WORTH Relational is roadmap-complete only when:

- all remaining product milestones are shipped
- all ten generic named requirements in
  [test-requirements.md](./test-requirements.md)
  are satisfied
- both CAD named requirements in
  [test-requirements.md](./test-requirements.md)
  are satisfied
- both chip named requirements in
  [test-requirements.md](./test-requirements.md)
  are satisfied
- all certification runs emit canonical machine-checkable artifacts for truth,
  patches, diagnostics, lineage, replay, branch heads, and query surfaces
