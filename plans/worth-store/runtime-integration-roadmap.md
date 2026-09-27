# Worth Store Runtime And Query Integration Roadmap

## Purpose

This is Part II of the active Worth Store program. It begins after the
[Worth Store Physical Database Roadmap](physical-database-roadmap.md) closes.
That Part I closeout includes the mandatory
[Physical Foundation Reconstruction Roadmap](physical-foundation-reconstruction-roadmap.md),
which makes the physical runtime, media path, fresh-process recovery, and
S.1-through-S.9 certification real before S.10 through S.12 finish. Part II
does not compensate for an incomplete reconstruction with an integration-side
backend or semantic persistence fallback.

Part II establishes the backend-neutral `worth-application-runtime` lifecycle
facade and its concrete `worth-application-store` composition. That composition
owns one Query participant, one physical Store instance, and their narrow
bindings.
Query owns semantic execution, Store owns physical execution, and the bindings
translate only public contracts. The application facade is the product entry;
Worth Query remains the domain language, Relational remains MVCC authority,
semantic Signal remains derived-computation authority, Runtime Bridge remains
the causal protocol layer, and Worth Store makes the joined system survive.

The previous semantic Store roadmap is deleted. Its document structure,
milestone numbering, implementation topology, and closeout claims are not
migrated, preserved, or treated as earned compatibility. Concepts that still
belong in the product are designed again against the real physical Store and
the current Query/Relational/Signal architecture.

## Roadmap Position

```text
Physical Database Roadmap S.12
  -> Milestone 1: Query integration readiness and backend contract refactor
  -> Milestone 2: semantic-to-physical integration spine
  -> Milestone 3: Store durability beneath canonical commit and publication
  -> Milestone 4: basis-pinned Store reads, hydration, and access providers
  -> Milestone 5: branch-aware concurrent MVCC over Store
  -> Milestone 6: runtime residency, drainage, cleanup, and rehydration
  -> Milestone 7: durable semantic identity and checkpoint basis
  -> Milestone 8: bounded bootstrap, recovery, PITR, and readmission
  -> Milestone 9: Store-backed Query execution and pushdown parity
  -> Milestone 10: durable historical worlds, branches, previews, diff, and merge
  -> Milestone 11: durable Query artifacts, saved queries, and continuations
  -> Milestone 12: blob-backed Query delivery and large-object semantics
  -> Milestone 13: durable live views and subscriptions
  -> Milestone 14: deterministic bulk ingest, transform, and migration
  -> Milestone 15: semantic retention, compaction, maintenance, and tiering
  -> Milestone 16: compatibility, replication, and portable capsules
  -> Milestone 17: extensible durable product artifacts
  -> Milestone 18: global admission, operations, and trust integration
  -> Milestone 19: joined runtime and Store certification
```

## Document Authority And Spec Names

The active Store planning authority is exactly:

- `physical-database-roadmap.md` for Part I
- this document for Part II

Fresh Part II milestone specs use the unambiguous
`runtime-integration-m{number}.md` family. Existing `milestone-*.md`, closeout,
or implementation-plan documents from the retired semantic roadmap are not
children of this roadmap and provide no requirement, dependency, compatibility,
or completion evidence for it.

## Governing Summaries

- `MENTALITY.md` protects adversarial-first, foundation-first construction. The
  strongest constraint is that Query composition and concurrent authority must
  be made structurally real before broad Store-backed features are added.
- Foundational Milestone 10 eight-axis profiles: Store operational and durable
  evidence names `ExecutionObjectiveProfile::Throughput` and
  `ObservationActivationProfile::OnDemand`. Throughput never weakens WAL,
  acknowledgement, recovery, MVCC, integrity, custody, or authoritative
  lineage. This roadmap does not claim a Store crash/WAL courtroom closed.
- `arch_laws.md` protects autonomous subsystems and proof-carrying transitions.
  The strongest constraint is that the monolithic mutable Query backend must
  split into separately borrowable read, submission, lifecycle, and inspection
  authorities, while commit execution consumes a typed durability progression.
- `composition_laws.md` protects one named semantic responsibility per unit. The
  strongest constraint is that application lifecycle, Query participation,
  physical Store, and Store bindings remain distinct responsibilities; the
  bindings cannot become a third runtime or duplicate orchestration.
- `domain_structure_laws.md` protects authority and dependency direction. Query
  owns public intent and semantic orchestration, Relational owns semantic
  truth, semantic Signal owns semantic derivation, Bridge owns semantic causal
  crossing, the physical Store instance owns byte survival and physical work,
  and the integration boundary owns only typed lowering and descriptive
  evidence correlation beneath those owner contracts.
- `perf_laws.md` protects locality, boundedness, and honest cost. Parallel
  admission must consume planned read/write/conflict footprints; a branch label
  alone is not proof of disjointness, and a global backend lock is not an
  acceptable concurrency implementation.
- `dx_laws.md` protects readable intent with inspectable execution. Store-backed
  construction must hide mechanical wiring while Query plans expose basis,
  consistency, concurrency footprint, cost, durability posture, and recovery
  actions before execution.
- `physical-database-roadmap.md` protects database-grade byte survival. Part II
  consumes its pages, WAL/checkpoint physics, stable physical reads, bounded
  memory, I/O pacing, integrity, backup, security, and certification evidence;
  it does not reproduce those mechanisms above the boundary.
- `physical-foundation-reconstruction-roadmap.md` protects the executable
  truth of that substrate. Part II consumes its sealed runtime and physical
  handoffs; supplied replay layouts, heap-only stores, disconnected file
  writers, or test-only physical mechanisms cannot satisfy an integration
  milestone.
- `worth-query/docs/AI_README.md` protects one canonical Query meaning across
  runtime-backed and later Store-backed operation. Part II must extend Query's
  admitted operating modes rather than create a Store-local query language,
  planner, outcome taxonomy, or lifecycle system.
- `worth-proof/README.md` protects proof-bearing progression law. Part II must
  use sealed platform witnesses and proofs for authority-bearing transitions;
  generic marker bounds, ids, digests, labels, and receipts must not mint a
  stronger phase or open a governed public door.
- `worth-foundational/docs/FOUNDATIONAL_README.md` protects shared boundary
  vocabulary. Part II may lower into Foundational identity, canonical-basis,
  profile, diagnostic, provenance, receipt, performance, and transition nouns
  only where meaning actually crosses a crate, trust, support, or export
  boundary. Query-owned and runtime-local hot state stays in its strongest
  owning type.
- `NAMING.md` and `BOUNDARIES.md` protect the platform audience and routing
  grammar. Product and platform entry crates consume Query only through
  `worth-query-decl` and `worth-query-host`; certification alone may consume
  `worth-query-replay`. The Store integration does not become a new audience
  facade by convenience.

## Global Adversarial Constraint

The joined runtime must survive this hostile condition:

> A Store larger than memory serves many users across deep, independently
> mutating branches while one-shot reads, live views, subscriptions, bulk work,
> background maintenance, replication, schema evolution, and operator recovery
> run concurrently. At minimum, five users read branch A, five users read branch
> B, two mutations submit on branch A, and two mutations submit on branch B.
> Same-branch submissions may be planned or queued concurrently, but exactly
> one mutation at a time owns active branch-write authority for a branch-head
> generation;
> branch A and branch B may each have one active writer concurrently while
> crashes are injected before durability, after durability but before semantic
> publication, after publication but before acknowledgment, and during restart.
> Every completed Query operation must retain canonical meaning, every admitted
> reader must observe one basis-exact MVCC world, every acknowledged write must
> survive, every unacknowledged durable write must resolve idempotently, and
> independent branches must make concurrent progress without a global mutable
> backend lock or full-store heap materialization.

The runtime has failed if it:

- creates a second semantic Store runtime beside the Query runtime
- turns the Store bindings into a third runtime, scheduler, cache, truth
  registry, or recovery authority
- gives Query and Store different Relational instances or runtime identities
- confuses the physical Store Signal graph with the semantic Query Signal
  graph, or persists either graph as crash authority
- persists Query receipts, digests, labels, or consumed projection authority as
  substitutes for fresh Query admission
- allows Query to decide MVCC truth or Store to decide semantic visibility
- acknowledges a mutation before physical durability and semantic publication
  have a recoverable relationship
- serializes independent branch work through one runtime-wide mutex, mutable
  backend borrow, submission queue, or copied snapshot registry
- admits two live active writers for one branch-head generation, lets queued
  same-branch work emit mutation effects, or releases a writer fence from an
  indeterminate outcome before recovery resolves it
- treats different branches as automatically disjoint when a global invariant,
  index, quota, or authority surface is shared
- lets physical WAL ordering become unnecessary semantic serialization
- requires full Store reconstruction in memory for ordinary reads or restart
- treats memory pressure as permission to break a pin, discard undurable work,
  persist ephemeral authority, or delete durable semantic truth
- lets live, bulk, retention, replication, or operator paths bypass the same
  canonical Query and durability boundaries

## Product Decision Lock

Part II freezes these decisions:

1. Worth Query is the ordinary domain runtime language; the backend-neutral
   application facade is the persistent product lifecycle surface.
2. `workspaces/application-composition/crates/worth-application-store` is the
   one Store composition. It owns one Query participant, one physical Store
   instance, and narrow bindings; there is no semantic Store runtime.
3. The Store bindings implement owner-defined provider and physical-
   lowering contracts. They translate semantic intent to physical work and physical
   evidence to semantic readmission, but it is not a runtime, scheduler, cache,
   truth registry, or lifecycle authority of its own.
   Part II defines the owner persistence ports and typed durability progression
   with Relational, Signal, and Runtime Bridge; the binding cannot define a
   Store-specific semantic authority lane. Store-
   native read, submission, lifecycle, inspection, and pushdown providers are
   additive Query capabilities, not replacements for those owner ports.
4. Query core and Query host do not depend on Store. Physical Store does not
   depend on Query. Only `worth-application-store` knows both public contract
   families; backend-neutral `worth-application-runtime` knows neither.
5. Exactly one Relational authority exists per runtime identity. Bridge consumes
   a published-read handle from that authority rather than a cloned runtime.
6. MVCC visibility, transaction legality, branch truth, and conflict decisions
   remain Relational-owned.
7. Store owns durable semantic artifact survival and physical access through
   its own sealed physical instance. Its private physical Signal graph is
   derived and reconstructible; it cannot mint Relational, Bridge, semantic
   Signal, or Query authority from persisted representations.
8. Query declarations may be persisted. Query runtime authority is reacquired
   through fresh admission after recovery.
9. Different branches may prepare and execute concurrently when the lowered
   plan carries sufficient disjointness proof. Shared authority coordinates at
   the narrowest real boundary.
10. Same-branch mutation submissions may be planned or queued concurrently,
    but exactly one mutation at a time owns active branch-write authority for
    one branch-head generation. The authority is Relational-owned, sealed,
    generation-bound, non-cloneable, and consumed into publication, a proven
    no-effect release, or an indeterminate recovery fence. A later same-branch
    submission acquires authority only after the prior mutation reaches one of
    those terminal postures and revalidates or replans against the then-current
    head.
11. Different branches may own active branch-write authority concurrently.
    Branch-local ownership does not itself prove their work independent: before
    parallel execution, lowered plans and global indexes, uniqueness, quotas,
    cross-branch invariants, physical ordering, and other shared authority
    coordinate independently at their narrowest real boundary.
12. Governed public transitions consume concrete platform authority witnesses
    and proofs. A caller-defined generic marker, persisted receipt, digest, or
    projected identity cannot authorize progression.
13. Query-owned and runtime-local artifacts remain in their strongest owning
    types. Foundational vocabulary is used at real shared boundaries, not as a
    universal internal envelope or second runtime model.
14. Product entry code opens `worth-application-runtime` with the selected Store
    composition. That runtime consumes Query only through `worth-query-host`;
    product and platform code never import Query core or Store internals.
15. Query replay and reconstruction surfaces remain certification-only.
    Operational Store restart consumes the runtime's dedicated recovery and
    readmission contracts; it does not route ordinary service through
    `worth-query-replay`.
16. This roadmap is ordered by implementation dependency, not by the repository
    containing each edit. A milestone may require changes in Query, Relational,
    Bridge, Signal, Foundational, or Proof when those changes are prerequisites
    for the one Store-backed runtime.
17. Store-gated execution, durable Query artifacts, and blob-backed delivery
    are implemented and closed here. Query retains canonical semantic contracts
    and runtime-backed proof, but does not carry a duplicate Store implementation
    sequence.
18. Query's semantic Signal and Store's private physical Signal have separate
    owners, identities, lifecycles, node vocabularies, and recovery posture.
    They communicate only through owner contracts; neither imports the other's
    nodes or completion handles. The physical Signal remains Store-owned and is
    not replaced by application composition.
19. Format, backend, residency, WAL, recovery, isolation, integrity, layout,
    and blob mechanism crates remain ignorant of Query and semantic Signal.
    Part II consumes the physical Store facade; it does not reach through it to
    raw mechanisms or physical Signal state.

## Cross-Cutting Artifact Law

Every durable artifact family introduced by any milestone must declare at
birth:

- authority class: authoritative, derived durable, or ephemeral
- accuracy class for derived material: exact, conservative, approximate,
  heuristic, or advisory
- canonical identity and version
- source authority and rebuild basis
- retention and reclamation participation
- compatibility and migration posture
- replication, export, and import participation
- tenant, key, authenticity, and custody scope
- ordinary, recovery, and diagnostic cost contracts
- support/admission row and typed unavailable posture

No later milestone exists to retrofit these declarations across an already
proliferated artifact graph.

## Operating Modes

- `RuntimeBacked`: Query runs on its existing non-Store runtime substrate. No
  Store capability or durability claim is ambient.
- `StoreBackedDurable`: `worth-application-store` binds the Query participant and
  physical Store while application runtime orders their lifecycle. Semantic working state and durable
  physical state coexist: admitted families may be hot in the Query runtime,
  hydrating, draining, or Store-only without changing their canonical meaning.
- `StoreAttachedEmbedded`: an external host supplies live runtime authorities
  while the selected application composition owns explicitly scoped physical
  Store bindings without stealing host authority.
- `Unavailable`: Store-backed vocabulary may remain visible for planning, but
  support admission fails typed before execution when the required physical,
  semantic, or deployment capability is absent.

These modes describe admitted composition and deployment posture, not an
exclusive choice between memory and database. In Store-backed modes, ordinary
semantic state may move between a bounded in-memory working set and durable
Store representations. Large blobs and other admitted physical-native families
may be written and streamed through Store without full semantic hydration.

## Target Composition And DX

The strongly preferred initial topology is:

```text
                         Product / API
                              |
                  Worth Application Runtime
                              |
                   worth-application-store
                    /                    \
          Query host participant     physical Store facade
          Relational / semantic      Signal / scheduler / WAL / media
          Signal / Runtime Bridge
```

The rough directory target is:

```text
workspaces/application-composition/crates/
  worth-application-runtime/     provider-neutral opening/readiness/close owner
  worth-application-store/       concrete Store composition, not a semantic runtime
    composition/                 construction, operating profile, provider bindings
    query_participant/           Query host and authority-handle binding
    physical_store/              physical Store facade and lifecycle binding
    adapter/
    semantic_lowering/           prepared semantic intent to physical plans
    relational_durability/       physical persist_prepared body and descriptive outcome
    physical_access/             basis requirements to Store access plans
    semantic_hydration/          verified bytes to readmission material
    artifact_association/        semantic identities to physical families
    recovery_readmission/        recovered evidence to fresh semantic admission
    facade/                      narrow Store composition handle

workspaces/worth-store/crates/
  worth-store-semantic-artifacts  persisted semantic records and codecs only
    commit_record/                canonical prepared-commit representation
    identity_record/              schema, branch, version, and lineage records
    query_artifact_record/        saved declaration and template records
    continuation_record/          cursor, checkpoint, and resume records
    blob_reference_record/        basis-bound large-object reference records
    codec/                        versioned semantic/physical translation

workspaces/application-composition/crates/
  worth-application-composition-certification  joined hostile courtroom
```

This is a rough target, not permission to create bags. Milestone 1 may change
crate cuts when ownership proves different, but it may not collapse the sibling
instances, make the adapter a third runtime, or introduce a second public
runtime. Each adapter child is one named lowering, correlation, or hydration
responsibility; `adapter/` is not permission for a generic translation bag.

The target construction experience is conceptually:

```rust
let store = WorthStorePhysicalRuntime::open(store_configuration)?;
let application = WorthApplicationStore::compose(store)?;

let runtime = WorthApplicationRuntime::open_persistent(
    application,
    domain_package,
    host_runtime_bindings,
)?;

let query = runtime.query();
let operations = runtime.operations(operator_authority);
```

The application facade owns infrastructure lifecycle, not domain meaning.
Platform code selects the concrete composition; ordinary domain work remains
behind the legal Query declaration and host facades. `operator_authority`
represents a concrete sealed platform authority, not a caller-defined marker.

Application composition orders bottom-up construction while each owner retains
recovery meaning: admit and recover physical Store; let the binding identify the
durable semantic frontier; create the one Relational authority; install semantic
Signal and Runtime Bridge from published semantic handles; consume each typed
owner readiness; then expose overall readiness. Shutdown proceeds top-down:
quiesce Query admission and live work; let the adapter settle every admitted
semantic/physical join; drain and close the physical Store instance; then
release composition authority. A failure at any step returns one typed posture
that preserves the exact authority and residue still requiring disposition.

Normal product code sees Query capabilities, not Relational, Bridge, Signal,
page, WAL, or backend wiring. Serious plans expose basis, consistency,
read/write/conflict footprints, parallel-admission proof, durability posture,
estimated physical work, residency posture, possible rehydration work, and
recovery action before execution.

## Milestone Plan

## Milestone 1: Query Integration Readiness And Backend Contract Refactor

### Goal

Extend Query's closed concurrent-read and deterministic-submission substrate so
it can host a production Store-backed runtime without one exclusive mutable
backend object becoming the execution and lifecycle owner of the whole system.

### Boundary

This is not Store persistence yet. It consumes Query Milestone 9.7 rather than
rebuilding its shared-read, generation-pinning, journal, and published-artifact
work. It refactors the remaining backend contract and submission topology so a
Store implementation can admit branch-disjoint execution without reopening the
public Query semantics or serializing through one mutable backend borrow.

### Adversarial Constraint

Adding one reader, writer, live resource, inspection request, or Store-backed
capability must not require borrowing unrelated runtime state, locking the
whole backend, cloning Relational truth, or constructing a second orchestration
surface.

### Must Ship

- split the current `WorthQueryRuntimeBackend` responsibilities into at least:
  - concurrent basis-pinned read authority
  - concurrent submission authority
  - Query-participant lifecycle authority
  - observational inspection authority
- retire the execution-critical `&mut self` backend bottleneck so ordinary
  reads, submissions, lifecycle work, and inspection do not require one
  exclusive borrow of unrelated runtime state
- cloneable or independently borrowable handles where the authority permits it
- explicit `Send`/`Sync` posture per handle rather than ambient thread-safety
  claims
- migrate the existing runtime-backed Bridge backend onto the split contract
  before the Store-backed implementation is admitted, proving the seam belongs
  to Query rather than being a Store-only adapter shape
- preserve one deterministic public submission contract while allowing the
  backend to plan or queue multiple same-branch submissions without treating
  queued work as active mutation authority or forcing different branches
  through one globally serialized execution lane
- a move-owned `PlannedBranchMutation` or equivalently precise non-authority
  packet carrying the requested branch basis and lowered read, write, conflict,
  invalidation, locality, and shared-authority footprints; planning and queueing
  this packet grants no mutation or publication authority
- a Relational-owned, sealed, move-only active branch-write authority bound to
  runtime identity, branch identity, observed branch-head generation, and
  writer generation; at most one such authority may be live per branch-head
  generation at a time while different branches may hold theirs concurrently
- a Relational branch-writer admission owner with direct branch-identity lookup
  and generation fencing rather than one runtime-wide submission mutex or FIFO;
  queued plans carry no mutation or publication authority
- consuming terminal progression for active branch-write authority: successful
  publication advances the branch generation, proven no-effect releases the
  current generation for fresh admission, and indeterminate durability or
  publication fences new writers until recovery resolves the branch fate
- explicit compatibility map from Query Milestones 9.7, 9.10, 9.11, 9.12, and
  9.13 into the provider contracts this roadmap consumes
- a Store-backed Query backend contract implemented outside Query core
- an extensible provider construction progression behind the stable
  `WorthApplicationRuntime::open_persistent` lifecycle facade, consuming Query
  only through `worth-query-host`; later Store requirements make incomplete
  provider sets fail to compile
- one `worth-application-store` composition owning one Query participant, one
  physical Store instance, and narrow bindings while exposing Query capability
  namespaces through the application runtime
- one Relational runtime identity with distinct write authority and published
  read-source handles
- Bridge construction from the published read source, not an `Arc` clone of a
  mutable Relational runtime
- compile-time prohibition of explicit-backend plus backend-parts double
  authority
- support-profile rows for runtime-backed, Store-backed durable,
  Store-attached embedded, and unavailable postures
- dependency and residue checks forbidding:
  - Query core importing Store
  - physical Store importing Query
  - semantic Signal importing physical Signal nodes or physical Signal
    importing semantic nodes
  - the adapter owning a scheduler, pending-work registry, cache, or independent
    recovery lifecycle
  - platform or product entry crates importing Query core or the Store runtime
    instead of the Query audience facades
  - ordinary crates importing `worth-query-replay`
  - a second Store query language or planner
  - runtime-wide `Mutex`/`RwLock` wrappers used as the concurrency design

### Must Preserve

- existing canonical Query declarations and typed outcomes
- Relational authority over truth and MVCC
- semantic Signal authority over derived computation
- Bridge authority over causal crossing
- absent/non-Store Query operation
- the stable Query-host audience facade and the in-memory owner authority
  completed through Query Milestone 9.17; Store integration adds physical
  durability beneath those owners without creating a separate runtime API

### Proof Obligations

- compile-fail proof that read handles cannot mutate and inspection cannot mint
  execution authority
- concurrency-shape proof that two read handles and two submission handles can
  exist without a global mutable backend borrow
- a deterministic two-branch proof must queue two mutations for branch A and
  two for branch B, admit exactly one active branch writer for each branch,
  prove A and B progress concurrently, and prove neither queued same-branch
  mutation can prepare authoritative truth, lower mutation-bearing physical
  work, or publish; basis-pinned planning reads remain independently admissible
- lifecycle proof that adding a managed runtime subsystem breaks every
  incomplete construction and fork site
- residue proof that no parallel Store runtime or local pseudo-Query API exists

### Closeout Gate

Milestone 1 is not closed until Query can host the future Store-backed backend
through autonomous authority handles, branch-disjoint work is not forced
through one execution borrow, at most one writer is active per branch generation
at a time through compiler- and runtime-enforced authority without a global
writer lock, absent mode remains clean, and a runtime-wide lock or duplicate
Relational instance cannot satisfy the public contract.

## Milestone 2: Semantic-To-Physical Integration Spine

### Goal

Build the real `worth-application-store` bindings that translate owner semantic
artifacts into Part I physical operations and translates stable physical reads
back into runtime-owned semantic material without transferring authority to the
translation layer.

### Boundary

Query and Relational own the semantic contracts presented to the seam. Store
owns physical write, read, range, stream, lease, and durability contracts. The
adapter owns the typed lowering, evidence correlation, and hydration
relationship between those public families. It does not decide Query meaning,
Relational legality, physical storage mechanics, physical work readiness, or
physical scheduling.

Part II designs the durability seam here because Query deliberately stops at
memory-resident semantic authority. Relational, Signal, Runtime Bridge, and
Query dispatch each define the smallest persistence contract required by their
own meaning; `worth-application-store` binds those contracts to Store pages,
WAL, indexes, and physical Signal work. No Store-local receipt, registry, or
publication path may bypass owner readmission or become parallel authority.

### Adversarial Constraint

One canonical mutation and one basis-pinned read must cross the actual Part I
backend, survive restart, and return the runtime's canonical result without an
in-memory shadow record, consumer-local adapter, backend-shaped query artifact,
or decoder that can mint semantic authority from bytes.

### Must Ship

- Query-owned provider contracts for Store-backed read, submission, lifecycle,
  inspection, capability admission, and typed unavailability
- owner-defined persistence ports for Relational, Signal, Runtime Bridge,
  immutable package storage, runtime-stream lifecycle, and dispatch
- exact Query commit-provider capacity binding, pre-effect atomic reservation,
  and a phase-correct backpressure/indeterminate boundary; Store-native
  scheduling cannot hide a second queue or convert post-effect ambiguity into
  backpressure
- typed semantic write lowering from prepared Relational artifacts into Store
  mutation batches with exact identity, ordering, scope, and durability basis
- typed physical read lowering from Query/Relational access requirements into
  Store point, range, streaming, and materialization plans
- versioned semantic record families and codecs in
  `worth-store-semantic-artifacts`
- hydration surfaces that produce untrusted or readmission-required semantic
  material, never current Relational authority directly
- an explicit mapping registry from semantic artifact family to physical
  capability, layout class, integrity class, and recovery participation
- capability negotiation that denies before execution when the physical Store
  cannot satisfy a semantic requirement
- the first production-path vertical specimen: declare, prepare, lower, write,
  flush, restart, read, hydrate, readmit, and compare one canonical artifact
- dependency fences proving Query core and physical Store remain mutually
  ignorant while only the integration boundary imports both public contracts
- visibility fences proving physical Signal nodes and completion handles do not
  cross the adapter as semantic authority, and semantic Signal nodes do not
  enter the physical Store instance

### Must Preserve

- semantic types remain stronger than their physical encodings
- record codecs cannot validate domain invariants or publish branch heads
- Store plans cannot become Query plans or Relational transactions
- Store-native persistence cannot create a second durability receipt,
  publication progression, outbox payload, pending-work registry, or recovery
  authority beside the owner-defined ports
- the integration crate contains named lowering and hydration responsibilities,
  not a generic adapter or translation bag

### Proof Obligations

- compile-fail proof that decoded records cannot construct admitted Query or
  Relational authority
- round-trip canonical parity through the real Part I backend and a fresh
  process
- unsupported-capability denial before allocation, I/O, or partial lowering
- mapping completeness proof for every admitted semantic artifact family
- sabotage proof for swapped identity, wrong branch, wrong schema, truncated
  record, stale codec, and backend-local fallback

### Closeout Gate

Milestone 2 is not closed until one real semantic write and one real semantic
read traverse the production physical database in opposite directions through
typed, authority-preserving contracts, and no shadow in-memory path can satisfy
the acceptance proof.

## Milestone 3: Store Durability Beneath Canonical Commit And Publication

### Goal

Prove one complete Query mutation from declared intent through Relational
transaction authority, adapter lowering, physical Store durability, descriptive
physical evidence, Relational publication, Query completion, crash, restart,
and equivalent read.

### Boundary

The Store binding owns physical lowering and evidence correlation beneath
the existing Relational backend port, but it owns no transactional join,
publication, acknowledgment, or completion authority. Relational decides
whether a mutation is legal, prepares canonical truth, verifies the descriptive
physical outcome, and alone mints publication. Store decides whether the bytes
survive. Query alone mints completion after consuming the resulting
`RelationalCommitOutcome` through its existing execution progression.

### Adversarial Constraint

A crash before Store durability, after Store durability but before Relational
publication, after Relational publication but before Query acknowledgment, or
after acknowledgment must resolve to one idempotent commit conclusion without
lost acknowledged truth, duplicate commits, or shadow branch heads.

### Must Ship

- one canonical owner progression established jointly by the semantic owners
  and Store integration, with Store implementing only its physical middle,
  equivalent to:

  ```text
  AdmittedQueryMutation
    -> PlannedBranchMutation
    -> BranchWriteAdmittedMutation
    -> RelationalPreparedPublication
    -> RelationalPreparedDurableAppendRequest
    -> PhysicalAppendNotStarted
    -> PhysicalAppendStarted
    -> PhysicalRelationalAppendOutcome
    -> RelationalDurabilityAdmittedPublication
    -> RelationalCommitOutcome
    -> QueryCompletion
  ```

- the canonical progression is carried by `worth-proof` stages, sealed
  platform authorities, and typed transition outcomes rather than a local
  Store proof vocabulary
- private constructors so no phase can be skipped or synthesized from ids,
  digests, receipts, or labels
- `PlannedBranchMutation` may be constructed and queued concurrently with other
  same-branch plans but carries no active writer or publication authority
- `BranchWriteAdmittedMutation` consumes the Relational-owned active
  branch-write authority for the exact branch-head generation; a planned or
  queued mutation cannot construct `RelationalPreparedPublication`, lower
  Store mutation work, or enter durability
- canonical commit envelope, version identity, branch identity, parent basis,
  active writer generation, mutation identity, and idempotency identity
- physical lowering through the physical Store facade into its C.5.1 work
  topology and Part I WAL/page/checkpoint contracts, never raw mechanisms
- Store physical outcome bound to the exact prepared request, branch, writer
  generation, mutation identity, and idempotency identity; it is descriptive
  evidence returned through the existing backend port, not Store authority
- Relational verification and publication that consume the admitted physical
  outcome through the existing owner progression
- group-commit-compatible durability without changing semantic commit identity
- typed `Indeterminate` result with recovery handle where a client cannot know
  whether an unacknowledged durable commit published; this posture retains the
  branch-generation fence so a later writer cannot overtake unresolved truth
- retry deduplication returning the original canonical result
- the first Store-backed Query read after fresh-process recovery

### Must Preserve

- Query does not become commit authority
- Store does not validate domain invariants or MVCC legality
- physical LSN order does not replace semantic version or branch identity
- Signal invalidation begins only after committed truth is publishable

### Proof Obligations

- crash matrix at every progression edge
- exact acknowledgment proof for every supported backend capability tier
- duplicate-request proof across fresh-process restart
- same-branch queue proof that only the active generation owner can reach
  preparation or durability, and that publication, proven no-effect, and
  indeterminate outcomes respectively advance, release, or retain the branch
  fence
- in-memory runtime versus Store-backed canonical-result parity
- controlled defect proving acknowledgment-before-durability is detected

### Closeout Gate

Milestone 3 is not closed until a Query-authored mutation can be killed at every
authority transition and fresh-process recovery produces the one allowed
canonical result with exact durability, publication, and acknowledgment
evidence.

## Milestone 4: Basis-Pinned Store Reads, Hydration, And Access Providers

### Goal

Make physical Store access an ordinary, bounded input to Query and Relational
reads so runtime state may be reconstructed on demand instead of remaining
permanently heap-resident.

### Boundary

Query owns the declared read and result meaning. Relational owns MVCC basis and
visibility. The adapter owns access-plan lowering, physical lease carriage,
record hydration, and readmission into the one Relational runtime. Store owns
stable bytes and bounded physical access; its Signal graph, scheduler state,
and frame leases remain physical implementation authority rather than Query
runtime state.

### Adversarial Constraint

A basis-pinned query whose working set is larger than memory must fault cold
semantic records from Store while commits continue, without copying a complete
snapshot, widening into an undeclared scan, observing a mixed basis, or letting
decoded bytes bypass Relational visibility.

### Must Ship

- basis-pinned read requests carrying exact branch, version, schema, tenant,
  policy, projection, ordering, and traversal requirements
- Store point, range, streaming, adjacency, and bounded materialization
  providers selected only by lowered access plans
- stable physical read leases tied to semantic basis and generation
- version-aware decoding and semantic hydration with explicit readmission
- fault and prefetch paths that preserve the same Query result semantics
- intrinsic, persistent-index-required, streaming-required, and unavailable
  access postures
- bounded decoded-record arenas and lifecycle-scoped allocation
- exact counters for pages, ranges, bytes, records, faults, prefetches,
  hydrations, readmissions, copies, allocations, and amplification
- no-N+1 and no-full-snapshot structural enforcement

### Must Preserve

- a physical lease is not a semantic snapshot capability
- hydration cannot create a second Relational runtime or mutable truth copy
- readers never evaluate or publish derived state
- physical cache hits and misses cannot change canonical Query meaning

### Proof Obligations

- greater-than-memory basis-exact reads under concurrent commits
- hot-resident versus cold-faulted canonical result and receipt parity
- stale generation, reclaimed page, wrong schema, wrong tenant, and corrupted
  record denial before semantic use
- exact access-counter proof for point, range, streaming, and bounded traversal
- sabotage proof that an in-memory fallback or whole-snapshot copy is detected

### Closeout Gate

Milestone 4 is not closed until the ordinary read path can obtain cold semantic
state from the real Store under a stable basis, hydrate it through Relational,
and return canonical Query results inside declared memory and access bounds.

## Milestone 5: Branch-Aware Concurrent MVCC Over Store

### Goal

Make concurrent branch work a structural property of the real Store-backed
runtime rather than a throughput hope above serialized or fully resident state.

### Boundary

Query declares basis and mutation intent, Relational computes MVCC visibility
and conflict sets, the lowered plan proves locality/disjointness, and Store
executes stable physical access and durable publication. A branch label alone
does not grant parallel authority.

### Adversarial Constraint

Five readers on branch A, five readers on branch B, two concurrent mutation
submissions on branch A, and two on branch B must make progress while exactly
one mutation per branch-head generation owns active branch-write authority at a
time.
Branch A and branch B must write concurrently while their queued same-branch
successors remain non-authoritative, some reads fault cold data from Store,
foreground reads remain basis-stable, and shared global authority coordinates
explicitly.

### Must Ship

- basis-pinned concurrent read handles over published Relational snapshots
- lowered read/write/conflict/invalidation/locality footprints
- proof-bearing parallel admission for structurally disjoint work
- Relational-authorized `worth-proof` disjointness evidence, with no
  caller-mintable or branch-label-only substitute
- branch-scoped planning and queueing lanes whose entries cannot mint active
  mutation or publication authority
- generation-bound active branch-write admission with exactly one move-owned
  live authority per branch-head generation at a time and independent ownership
  across branches
- concurrent preparation and validation across independent branches
- same-branch successor revalidation or replanning after the active writer
  advances or releases the branch generation
- typed outcomes for conflict, rebase required, retryable head drift,
  branch writer busy, indeterminate branch fence, and unavailable parallel
  admission
- explicit shared-authority footprints for global indexes, uniqueness,
  cross-branch invariants, tenant quotas, and other real coordination points
- physical group commit and total WAL ordering that do not serialize semantic
  preparation or independent branch publication
- branch-local Signal invalidation and live-resource maintenance after durable
  publication
- exact counters for admitted parallelism, denied parallelism, revalidation,
  active branch-writer acquisition, branch-writer busy denial, indeterminate
  branch fencing, branch-head contention, physical coordination, and global
  coordination

### Must Preserve

- existing readers continue observing their pinned pre-commit basis
- new readers observe only completely published heads
- planned or queued same-branch submissions own no active mutation authority,
  and an indeterminate writer prevents successor admission until recovery
  resolves its exact publication fate
- different branches do not imply disjointness when shared authority exists
- same-branch conflicts never become last-writer-wins folklore
- maintenance cannot revoke a protected physical or semantic read basis

### Proof Obligations

- the named ten-reader/four-writer two-branch scenario under deterministic
  schedule permutation
- same-branch queue permutation proving exactly one active writer, generation-
  ordered successor admission, and revalidation or replanning against the
  predecessor's published head
- sabotage proof that cloning an active branch writer, admitting two writers
  for one generation, or letting queued work lower mutation effects is rejected
- different-branch shared-global-index coordination proof
- no-global-lock structural audit and contention-slope test
- crash injection during concurrent group commit and branch-head publication
- forced cold-read faults and hydration during every admitted writer schedule
- exact parity between serial reference execution and every admitted concurrent
  schedule

### Closeout Gate

Milestone 5 is not closed until independent branches demonstrate real
concurrent progress, stable readers never block on ordinary writers, conflicts
localize to the authority actually shared, exactly one mutation owns active
write authority per branch generation at a time, and neither a monolithic
mutable backend nor a global writer queue can reappear as a hidden serialization
boundary.

## Milestone 6: Runtime Residency, Drainage, Cleanup, And Rehydration

### Goal

Bound the joined runtime's memory without confusing eviction, derived-state
discard, ephemeral cleanup, checkpointing, or durable retention.

### Boundary

Each runtime subsystem owns the lifecycle and semantic classification of its
resident state. A global semantic-residency admission authority allocates
memory envelopes, observes pressure, and coordinates typed plans, but it cannot
reach into subsystems and delete entries. Part I buffer-pool policy and
physical Signal lifecycle remain inside the physical Store instance; this
milestone governs semantic and Query-runtime residency above it and coordinates
cross-instance pressure only through declared adapter contracts.

### Adversarial Constraint

The ten-reader/four-writer two-branch workload must continue while the Store is
larger than memory, pressure oscillates across high and low watermarks, cold
branches rehydrate, old bases remain pinned, a slow subscriber lags, derived
state is rebuilt, configuration changes are attempted, and crashes occur during
drain and rehydration. Memory must remain bounded without lost acknowledged
truth, broken bases, false resume, global execution serialization, or pressure-
authorized durable deletion.

### Must Ship

- a complete residency inventory distinguishing at least:
  - planned or queued branch submissions without write authority
  - active generation-bound branch writers and undurable write state
  - Relational authoritative working state and reconstructible historical state
  - pinned published bases and generation leases
  - semantic Signal derived evaluation state
  - Query plans, results, indexes, and materializations
  - live/subscription resources and durable continuation support
  - diagnostics, support expansions, and semantic recovery state
  - observed physical Store budget and pressure posture, never ownership of
    physical frames or physical Signal nodes
- explicit drain actions:
  - `DropAndRebuild`
  - `EvictAndRehydrate`
  - `CheckpointThenEvict`
  - `RetainOrDenyNewWork`
- typed progression equivalent to:

  ```text
  ObservedMemoryPressure
    -> ClassifiedResidencyInventory
    -> ProposedDrainPlan
    -> LeaseValidatedDrainPlan
    -> ExecutedDrain
    -> ReclaimedCapacityReceipt
  ```

- subsystem-local residency authorities that propose candidates and execute
  only their own typed drain plans
- global hard budget, soft target, high/low watermark, tenant/branch share,
  background I/O, and pinned-basis admission policy
- rehydration cost and rebuild cost in candidate selection
- nested configuration aligned to subsystem ownership
- startup configuration validation and live reconfiguration through
  `inspect -> simulate -> admit -> apply -> observe`
- exact counters for resident, pinned, drainable, evicted, discarded,
  checkpointed, rehydrated, rebuilt, denied, and reclaimed bytes and objects
- cleanup protocols for completed transactions, cancelled work, abandoned
  previews, orphaned resources, expired leases, and stranded rehydration

### Must Preserve

- dirty or undurable authority cannot be evicted as if it were persisted
- a pin, active recovery lease, or exact-resume obligation cannot be broken by
  memory pressure
- derived state may be dropped only when its rebuild basis and admitted rebuild
  budget are known
- ephemeral runtime handles are closed or reacquired, never serialized as
  authority
- runtime drainage cannot duplicate Part I buffer-pool eviction or authorize
  durable retention deletion
- semantic pressure cannot directly cancel physical work, evict physical
  frames, or mutate the physical Signal graph; it requests typed Store actions
  through the adapter
- users configure budgets, priorities, and service objectives; they do not
  configure unsafe internal eviction order or semantic invariants

### Proof Obligations

- the named oscillating-pressure concurrent scenario with serial-oracle parity
- exact resident-memory ceiling and reclamation-slope assertions
- pin preservation, undurable-write protection, and recovery-lease protection
- hot-resident versus evict/rehydrate canonical parity
- derived drop/rebuild parity and ephemeral resource non-persistence
- live configuration simulation, rejection, atomic application, and rollback
- sabotage proof for global LRU, `clear_all`, broken pin, pressure deletion,
  hidden full-state scan, and unbounded rehydration allocation

### Closeout Gate

Milestone 6 is not closed until the joined runtime can remain inside a hard
memory envelope under real Store-backed concurrent load, every resident family
has one explicit drain or retain law, and user policy can shape service posture
without being able to weaken correctness.

## Milestone 7: Durable Semantic Identity And Checkpoint Basis

### Goal

Persist the complete semantic identity graph required to reconstruct and query
truth without making persisted representations self-authorizing.

### Boundary

This milestone owns durable semantic records and their exact physical mapping.
It does not reopen Query, Relational, Bridge, or semantic Signal authority from strings,
digests, or decoded bytes.

### Adversarial Constraint

Schema evolution, lineage changes, branch-head movement, cursor advancement,
and checkpoint publication may interleave with commits and crashes without
producing an identity that changes meaning, outranks source authority, or
cannot be re-admitted after restart.

### Must Ship

- durable schemas and schema-evolution boundaries
- canonical version DAG, branch-head, ordered-parent, and merge-parent records
- structural identity and lineage/correspondence events
- typed journal positions and journal segment identities
- durable CDC cursors and subscriber checkpoint bases
- semantic checkpoints bound to exact physical checkpoint/WAL ranges
- transactional cursor/checkpoint advancement with commit publication
- canonical identity/version rules for every persisted Query declaration family
- explicit retained-declaration records distinct from runtime authority
- Store layouts and access paths declared through the Part I layout discipline

### Must Preserve

- Relational owns interpretation of version, branch, schema, and lineage truth
- Query owns canonical declaration meaning
- Store owns record survival and lookup, not authority promotion
- cursor and checkpoint records cannot advance beyond durable truth

### Proof Obligations

- cross-backend canonical identity parity
- crash tests around schema, lineage, branch-head, cursor, and checkpoint
  publication
- compile-fail proof that persisted declarations cannot construct admitted
  Query handles
- property proof that every checkpoint range is continuous and basis-exact

### Closeout Gate

Milestone 7 is not closed until every identity required by ordinary recovery,
historical evaluation, continuation, and inspection has one durable source,
one typed interpretation boundary, and one fresh-admission path.

## Milestone 8: Bounded Bootstrap, Recovery, PITR, And Readmission

### Goal

Recover the joined runtime from a semantic checkpoint plus a bounded journal
tail, then re-enter Query through fresh typed admission.

### Boundary

The physical Store instance first recovers and verifies physical bytes without
Query. The adapter then identifies verified semantic artifacts and the durable
semantic frontier. Only afterward does Part II reconstruct Relational authority,
rebuild semantic Signal and Query state, and offer retained declarations for
fresh readmission.

### Adversarial Constraint

Recovery after corruption, PITR selection, retained-authority rollback,
replica bootstrap, or interrupted maintenance must converge to one semantic
truth world without scanning history from genesis, trusting derived residue,
or treating an old Query handle as current authority.

### Must Ship

- semantic recovery-source precedence over checkpoints, commit tail, snapshots,
  branch deltas, replicated artifacts, and rebuildable derived families
- bounded Relational reconstruction from checkpoint plus journal tail
- Store-backed access that avoids loading the full database into heap state
- rebuild plans for Signal graphs, Query indexes, live resources, and other
  derived runtime state
- independent disposal and reconstruction of physical and semantic Signal
  graphs, with neither graph's serialized state accepted as recovery authority
- retained Query declaration readmission through current support, policy,
  schema, tenant, and basis gates
- semantic PITR, complete retained-authority rollback, and replica/bootstrap
  recovery over Part I operational plans
- typed trusted, degraded, rebuild-required, quarantined, unavailable, and
  unrecoverable runtime conclusions
- idempotent resumption of durability/publication transitions stranded by crash
- recovery handles for indeterminate client-visible operations

### Must Preserve

- the offline verifier remains independent from the live runtime
- physical verification cannot mint semantic authority
- derived runtime state is disposable and rebuildable
- recovery cannot silently substitute a nearby basis for the requested basis
- operational recovery does not import Query replay or reconstruction
  surfaces; certification may compare it with a replay oracle from a cert-only
  lane

### Proof Obligations

- greater-than-memory restart with bounded resident bytes and bounded WAL tail
- crash-after-durability/before-publication resolution across fresh process
- corrupt-derived-state rebuild versus corrupt-authority quarantine
- PITR and rollback convergence against canonical replay oracle
- stale declaration, stale support row, foreign tenant, and wrong schema
  readmission denials
- dependency proof that the operational recovery graph cannot import
  `worth-query-replay`

### Closeout Gate

Milestone 8 is not closed until a fresh process can recover a larger-than-memory
Store, reconstruct one Relational authority, rebuild derivation, readmit valid
Query declarations, reject stale authority, and report exact recovery work.

## Milestone 9: Store-Backed Query Execution And Pushdown Parity

### Goal

Make ordinary Query meaning execute over persistent Store access paths with the
same canonical results and typed denials as the runtime-backed mode.

### Boundary

Query remains the declaration, admission, planning, and outcome owner.
Relational remains visibility and invariant authority. Store provides bounded
record, index, range, streaming, and materialization access selected by the
lowered plan. This milestone absorbs the ordinary execution and pushdown half
of former Query Milestone 10; implementation may change Query provider and plan
contracts, but closure belongs to this joined roadmap.

### Adversarial Constraint

A query whose data is much larger than memory must not widen into full Store
scans, caller-owned graph traversal, N+1 access, copied snapshots, or hidden
materialization merely because the Store-backed path lacks an admitted access
structure.

### Must Ship

- Store-backed parity for admitted read, aggregate, mutation, inspection, and
  projection-consumption families
- physical pushdown for admitted projection, predicate, ordering, aggregation,
  range, and bounded traversal shapes with explicit fallback or denial
- collection/detail, ordering, opaque cursor, bounded traversal, graph read,
  policy, tenant, and relationship-proof execution
- persistent adjacency, predicate, ordering, range, frontier, visited-set,
  result, streaming, and materialization capability providers where admitted
- explicit required-posture outcomes for persistent index, streaming,
  asynchronous materialization, and unsupported access capability
- basis-pinned Store readers backed by stable physical plans and leases
- no-N+1, read-amplification, resident-memory, allocation, and I/O counters on
  Query receipts
- support-matrix and Consumer Kit proof for Store-backed family adoption
- exact runtime-backed versus Store-backed canonical-result comparison

### Must Preserve

- one Query expression and result model across operating modes
- Query access plans do not become physical authority
- Store indexes and materializations remain derived and rebuildable
- policy masking and tenant narrowing occur before physical access

### Proof Obligations

- generated parity matrix over every admitted Query family
- greater-than-memory graph and collection queries with exact access counters
- persistent-index absence and budget-exceeded typed denial tests
- corruption/rebuild parity for every derived Store access family
- no direct physical-row or materialization-row reads outside admitted providers

### Closeout Gate

Milestone 9 is not closed until serious Query work runs Store-backed without
semantic drift, hidden broad access, consumer-local adapters, or an unbounded
memory assumption.

## Milestone 10: Durable Historical Worlds, Branches, Previews, Diff, And Merge

### Goal

Make Query's truth-world and branch capabilities durable without letting Store
become branch, comparison, or merge authority.

### Boundary

Relational owns branch truth, historical visibility, structural identity, and
merge execution. Query owns basis lifecycle, preview/comparison declarations,
and outcomes. Store preserves the histories, delta layers, snapshots, and
derived assistance those owners require. This milestone absorbs the historical
restore and diff half of former Query Milestone 10 after Milestones 7 through 9
have made identity, recovery, and ordinary Store-backed execution real.

### Adversarial Constraint

Deep branch histories, concurrent branch mutation, schema evolution, identity
split/merge, retention pressure, preview isolation, and restart must not make a
historical Query ambiguous, copy full state per branch, or promote merge-
assistance material into merge authority.

### Must Ship

- Store-backed Query basis lifecycle for current, historical, branch, preview,
  comparison, and materialization operations
- structural delta storage and shared immutable branch bases
- near-constant branch creation relative to total Store size
- bounded delta-stack read amplification and deterministic rewrite policy
- immutable semantic snapshots and snapshot-plus-tail historical reads
- historical diff and structural correspondence materialization
- isolated preview truth and restart-safe retained preview declarations where
  admitted
- merge-ready history and derived merge-assistance artifacts
- exact ambiguity, missing-basis, pruned-basis, and lineage-drift outcomes

### Must Preserve

- branches share storage but never authority
- previews cannot publish through ordinary committed lanes
- diff and correspondence acceleration remains derived
- merge execution and conflict resolution remain Relational-owned

### Proof Obligations

- deep-branch greater-than-memory read and mutation slopes
- historical replay versus snapshot-plus-tail parity
- preview crash/restart isolation and forbidden publication tests
- split/merge lineage ambiguity localization
- destroy-and-rebuild proof for all diff and merge-assistance artifacts

### Closeout Gate

Milestone 10 is not closed until current, historical, branch, preview, diff, and
merge journeys retain one Query grammar, one Relational truth model, and bounded
Store-backed access across restart and retention pressure.

## Milestone 11: Durable Query Artifacts, Saved Queries, And Continuations

### Goal

Persist and reload canonical Query artifacts and delivery continuations through
the Store-backed runtime without turning their physical records into Query
authority or host-local convenience state.

### Boundary

Query owns saved-query, template, scope-composition, parameter, cursor, and
continuation meaning. This roadmap may change Query when those contracts need
completion. The integration owns record lowering, persistence, checkpoint
coupling, reload, portability, and fresh admission. Store owns byte survival.

### Adversarial Constraint

Saved queries, durable cursors, delivery checkpoints, and imported artifacts
must preserve canonical query identity, binding, basis, policy, tenant, and
continuation meaning across crash, restart, transfer, schema evolution, and
retention without restoring an old runtime handle or resuming past durable
truth.

### Must Ship

- durable records for canonical saved queries, templates, named scopes,
  parameter bindings, delivery cursors, and admitted continuation families
- versioned codecs and canonical identity independent of physical layout
- transactional cursor/checkpoint advancement tied to durable publication
- reload into readmission-required Query artifacts
- restart, import, export, replication, retention, and migration participation
  declared for every artifact family
- explicit ephemeral, durable, incompatible, pruned-basis, stale-policy,
  rebuild-required, and unavailable postures
- idempotent continuation recovery after duplicate or indeterminate delivery
- exact counters for reload validation, basis checks, continuation steps,
  import/export breadth, and readmission work
- deletion of host-local saved-query and cursor shims for admitted durable
  families

### Must Preserve

- persisted declarations, cursors, and receipts cannot construct active Query
  handles directly
- a cursor cannot outlive or misidentify its durable truth basis
- physical record identity cannot replace canonical Query identity
- unsupported durability fails typed instead of becoming a local cache

### Proof Obligations

- saved-query canonical identity parity across restart and physical rewrite
- cursor continuation parity across crash, duplication, and replay
- stale schema, policy, tenant, basis, and capability readmission denials
- import/export round trip with corruption and missing-dependency rejection
- compile-fail proof against authority restoration from persisted artifacts

### Closeout Gate

Milestone 11 is not closed until every admitted durable Query artifact reloads
through fresh admission to the same canonical meaning and every continuation
resumes at exactly one durable, basis-honest position.

## Milestone 12: Blob-Backed Query Delivery And Large-Object Semantics

### Goal

Make blob and media-backed Query results first-class, basis-honest Store
integration products instead of metadata plus host-side file plumbing.

### Boundary

Query owns blob-reference projection, result, policy, and delivery meaning. The
integration owns durable reference lowering, stable-handle derivation, range
and streaming access, upload association persistence, restart, and portability.
Part I owns native chunks, integrity, encryption, and physical retrieval.

### Adversarial Constraint

Multi-gigabyte blob projections, range reads, uploads, restart, branch sharing,
policy changes, export/import, and concurrent ordinary traffic must preserve
canonical Query meaning and basis identity with constant memory, without
opaque file sidecars, bearer handles that bypass policy, or full-object
materialization.

### Must Ship

- canonical blob/content reference projections in Query results
- basis-bound, tenant-bound, policy-bound, expiring delivery capabilities
- Part I chunk-backed range and streaming providers
- upload staging distinct from committed blob authority
- transactional upload-to-semantic-record association through the canonical
  owner commit progression
- deduplication and branch sharing that preserve tenant, key, and policy scope
- saved-query, continuation, replication, capsule, retention, and rehydration
  participation for blob-bearing results
- typed expired, stale-basis, wrong-scope, unavailable-range, corrupt-chunk,
  incomplete-upload, and unsupported-delivery outcomes
- exact counters for chunk reads, ranges, streamed bytes, resident bytes,
  copies, handle admission, policy denial, upload association, and portability

### Must Preserve

- Store owns persisted blob bytes; Query owns result meaning
- delivery capabilities cannot be promoted into object or Query authority
- uploads do not create a second commit or query path
- scalar and blob-bearing results obey the same basis and policy semantics

### Proof Obligations

- constant-memory multi-gigabyte range and streaming scenarios
- scalar versus blob-reference Query identity and policy parity
- restart, export/import, replication, and branch-sharing proof
- wrong-tenant, wrong-key, stale-basis, expired-capability, and corruption
  rejection before leakage
- crash matrix across upload staging, chunk durability, semantic association,
  publication, and acknowledgment

### Closeout Gate

Milestone 12 is not closed until blob-backed Query delivery is an ordinary
basis- and policy-honest Store-backed capability with bounded memory and no
host-side file semantics.

## Milestone 13: Durable Live Views And Subscriptions

### Goal

Make Query-owned live meaning and subscription lifecycles restartable from
durable support artifacts without persisting runtime authority or rebuilding
subscription semantics inside Store.

### Boundary

Query owns live declarations, family selection, activation, sharing,
maintenance, delivery, continuation, and close. Bridge and Signal own causal
maintenance and scheduling. Store owns durable declarations, bases, cursors,
checkpoints, and family-specific resume support through the generic durable
artifact and continuation substrate closed by Milestone 11.

### Adversarial Constraint

Subscriber crashes, runtime crashes, branch-head movement, identity evolution,
retention, backpressure, version skew, and support-family drift must converge to
an exact, degraded-but-recoverable, rebuild-required, or not-resumable
conclusion without duplicate delivery or false exact-resume claims.

### Must Ship

- durable canonical live and subscription declarations
- family-aware basis, cursor, checkpoint, and support artifacts
- exact resume, degraded recovery, rebuild, and non-resumable taxonomy
- restart-safe shared fanout and equivalent-subscription identity
- checkpointed delivery with deduplication and explicit ordering
- branch/preview isolation and lineage-aware continuation
- backpressure, overflow, cancellation, close, and orphan-resource recovery
- time-aware and async/resource-backed posture where Query support admits it
- retention, compatibility, replication, and maintenance declarations attached
  to every support family at birth
- branch-local invalidation and maintenance counters

### Must Preserve

- Store does not own subscription semantics or delivery policy
- retained declarations are not active handles
- live results converge to canonical one-shot Query results on the same basis
- durable support cannot overclaim exactness after basis or family drift

### Proof Obligations

- crash/restart at every activation, delivery, checkpoint, and close transition
- live-versus-one-shot convergence under concurrent branch writes
- equivalent-subscription sharing without cross-tenant or cross-branch leakage
- overflow/backpressure and slow-consumer boundedness
- retention and compatibility degradation localization

### Closeout Gate

Milestone 13 is not closed until active resources survive or fail honestly across
restart, every admitted family has exact resume semantics, and Store contains
support artifacts rather than a shadow subscription runtime.

## Milestone 14: Deterministic Bulk Ingest, Transform, And Migration

### Goal

Make large imports, rewrites, and schema migrations first-class Query/Store
programs that preserve ordinary commit meaning and branch concurrency.

### Boundary

Bulk authoring enters through Query-owned declarations and typed scopes.
Relational owns semantic validation and transaction production. Store owns
bounded staging, durable progress, physical write efficiency, and recovery.

### Adversarial Constraint

A multi-terabyte import or migration interrupted repeatedly while ordinary
branch reads and writes continue must remain bounded in memory, deterministic
under chunk variation, resumable without duplication, and canonically
equivalent to the admitted ordinary transaction sequence.

### Must Ship

- typed bulk ingest, transform, rewrite, and migration declarations
- explicit tenant, workspace, branch, schema, and artifact-family scope
- planned batch/chunk boundaries and deterministic canonical reduction
- bounded-memory staging and Store-native streaming paths
- resumable durable progress with idempotent chunk and transaction identity
- ordinary-versus-bulk canonical commit parity
- foreground reservation and background pacing integration
- branch-aware concurrency footprints and conflict behavior
- migration versioning, compatibility, rollback, and indeterminate recovery
- operator-visible progress, cost, and recovery handles

### Must Preserve

- bulk work does not use a second commit or validation path
- chunk size and worker count cannot change semantic output
- migrations cannot bypass Query support or Relational invariants
- cancellation cannot leave unclassified durable residue

### Proof Obligations

- randomized chunk size, worker count, and schedule determinism
- kill/restart at every durable progress boundary
- ordinary traffic latency and branch-concurrency proof during bulk pressure
- serial ordinary-commit oracle parity
- bounded resident bytes and exact allocation/I/O counters

### Closeout Gate

Milestone 14 is not closed until bulk and migration programs are merely
amortized executions of canonical runtime truth, not privileged utility paths.

## Milestone 15: Semantic Retention, Compaction, Maintenance, And Tiering

### Goal

Apply semantic survival policy over the physical maintenance substrate without
allowing pressure, placement, or derived debt to erase a basis the product
promised to retain.

### Boundary

Relational and Query define which semantic worlds and continuations remain
meaningful. Store translates that policy into physical reachability,
compaction, reclaim, rebuild, and placement work. Part I remains the authority
for physical isolation and pacing. This milestone governs durable survival and
deletion; Milestone 6 remains the separate authority for memory residency,
drainage, eviction, cleanup, and rehydration.

### Adversarial Constraint

Retention expiry, deep branch history, pinned readers, resumable subscribers,
snapshot/PITR leases, compaction debt, cold-tier movement, corruption rebuild,
and foreground pressure must not create dangling bases, reclaim protected
chunks, or make placement policy semantic truth.

### Must Ship

- typed retention policies over branches, history, checkpoints, declarations,
  subscriptions, derived families, blobs, and recovery leases
- semantic liveness graph lowered to physical reachability
- derived compaction products and deterministic rewrite publication
- reclaim guarded by semantic and physical leases
- foreground/background work classes, pacing, starvation, and debt escalation
- hot/warm/cold placement and recall posture
- working-set intelligence as derived advisory material
- family-specific survival conclusions for live, historical, replication, and
  extension artifacts
- rebuild, compaction, retention, recall, and deletion debt counters
- typed archive, compact, defer, deny, degrade, and unavailable outcomes

### Must Preserve

- pressure alone cannot authorize semantic deletion
- placement and working-set intelligence remain derived
- active readers and recovery operations retain stable bases
- maintenance cannot publish through an ordinary mutation shortcut

### Proof Obligations

- concurrent read/write/live workload during compaction, reclaim, and tiering
- lease preservation and stale-generation rejection
- retention-model oracle versus physical reachability comparison
- cold-tier recall and corruption-rebuild boundedness
- crash/restart around every maintenance root publication

### Closeout Gate

Milestone 15 is not closed until semantic policy and physical reachability agree
under concurrent operation, every deletion is authorized and explainable, and
foreground work remains inside its admitted interference envelope.

## Milestone 16: Compatibility, Replication, And Portable Capsules

### Goal

Make semantic Store truth and admitted support artifacts evolvable and portable
across processes, machines, and rolling versions without inventing a second
truth format.

### Boundary

Compatibility interprets versioned semantic artifacts; replication ships
canonical and declared derived families; Store verifies physical and logical
integrity; Query and Relational freshly admit imported meaning.

### Adversarial Constraint

Mixed-version replicas, partial branch transfer, snapshot-plus-tail bootstrap,
network duplication/reordering, schema evolution, retained declarations,
key/tenant scope, and interrupted import must converge or reject before
publication without silently weakening meaning.

### Must Ship

- semantic artifact version windows and migration contracts
- rolling mixed-version support and explicit reader/writer denial
- immutable capsule manifests over canonical semantic and physical identities
- snapshot-plus-tail, partial branch, bounded range, and blob-bearing transfer
- deterministic export/import and idempotent resume
- logical digest graph composed with Part I physical integrity and authenticity
- family declarations controlling portability of derived and subscription
  support artifacts
- tenant, key, custody, and proof-of-possession admission
- imported artifact quarantine, verification, and fresh semantic readmission
- replication lag, compatibility debt, transfer breadth, and rebuild counters

### Must Preserve

- replication cannot create a parallel canonical format
- a physically valid capsule is not automatically semantically admissible
- imported Query declarations cannot restore old runtime authority
- version compatibility cannot silently downgrade guarantees

### Proof Obligations

- generated version-skew matrix across every portable family
- duplicate, reordered, truncated, corrupted, wrong-tenant, and wrong-key
  capsule scenarios
- source-versus-replica canonical Query parity
- partial-branch isolation and missing-dependency rejection
- restartable import publication and quarantine proof

### Closeout Gate

Milestone 16 is not closed until portable artifacts preserve exact semantic and
physical identity, incompatible meaning fails before publication, and replicas
reconstruct the same admitted Query worlds as their source.

## Milestone 17: Extensible Durable Product Artifacts

### Goal

Open the Store-backed runtime to product blobs, materializations, analysis,
correspondence, locality, and future durable families without weakening the
authority, rebuild, concurrency, or certification model.

### Boundary

Extensions declare domain meaning and storage requirements. Query owns public
admission and orchestration. Store owns registered persistence strategy and
physical participation. No extension may define a private query, commit,
recovery, retention, replication, or proof path.

### Adversarial Constraint

An extension with large artifacts, approximate analysis, custom indexes,
branch-local materialization, blobs, and specialized access requirements must
remain contained under crash, rebuild, retention, replication, version skew,
tenant pressure, and hostile implementation mistakes.

### Must Ship

- extension registration consuming the full cross-cutting artifact law
- product-facing blob/object references over the Part I chunk substrate
- persistent Query materializations and specialized access families
- basis-anchored analysis checkpoints
- correspondence indexes, structural fingerprints, locality clustering, and
  admitted diff/merge assistance
- explicit exact, conservative, approximate, heuristic, and advisory behavior
- storage-strategy registration against physical layout and capability tiers
- extension support rows, budgets, migration, retention, replication, and
  certification requirements
- compiler and runtime denials for shadow authority or unsupported capability
- extension-local failure containment and rebuild isolation

### Must Preserve

- derived artifacts remain destroyable and rebuildable
- product blobs use native Store chunks rather than an external file sidecar
- extensions cannot mint Query, Relational, Bridge, Signal, or Store authority
- approximate or advisory artifacts cannot enter exact execution lanes

### Proof Obligations

- malicious extension attempts to bypass each authority boundary
- destroy/rebuild parity for every reference family
- blob-scale constant-memory and branch-sharing proof
- accuracy-class enforcement and forbidden promotion tests
- extension failure isolation during concurrent ordinary traffic

### Closeout Gate

Milestone 17 is not closed until adding a durable family is an explicit,
certifiable registration rather than permission to punch through the runtime.

## Milestone 18: Global Admission, Operations, And Trust Integration

### Goal

Make whole-runtime resource risk, semantic repair, operator action, tenant
scope, and security posture visible and governable before final certification.

### Boundary

Part I owns physical operations and security mechanisms. Part II owns their
semantic interpretation, Query-facing admission, runtime readmission, and
operator journeys. Worth Proof reports evidence; it does not authorize work.

### Adversarial Constraint

Resource exhaustion, key loss, tenant-scope error, corruption, backlog growth,
operator repair, failover, and partial recovery must fail before unbounded work
or wrong-scope publication while leaving a typed recovery or denial path.

### Must Ship

- global and per-tenant budgets for branch depth/count, history, checkpoints,
  declarations, live support, derived artifacts, blobs, indexes, WAL tail,
  resident/pinned/dirty pages, I/O, compaction, replication, keys, audit, and
  recovery work
- Query plan admission consuming semantic and physical budget posture
- semantic repair and readmission over Part I repair plans
- trusted, degraded, quarantined, unavailable, and unrecoverable operator views
- tenant-scoped blast radius, quotas, export, repair, and deletion conclusions
- key lifecycle, authenticity, custody, encryption, and audit integration
- tamper-evident causal chain from Query intent to physical and operator outcome
- explicit administrative authority separate from ordinary Query authority
- inspect, simulate, authorize, execute, verify, rollback, and recover DX for
  critical operator journeys
- machine-checkable configuration/support snapshot and provenance

### Must Preserve

- operators cannot bypass semantic invariants through physical repair
- Query plans cannot promote diagnostics or model verdicts into authority
- budget degradation is explicit policy, never silent widening
- Store remains outside identity-provider ownership

### Proof Obligations

- resource-exhaustion tests proving denial before broad construction
- wrong-tenant, wrong-key, stale-custody, compromised-key, and missing-proof
  rejection across ordinary and recovery lanes
- operator crash/restart and duplicate-action idempotency
- causal audit reconstruction from Query declaration to physical outcome
- semantic repair versus clean canonical rebuild parity

### Closeout Gate

Milestone 18 is not closed until every serious resource, trust, and operator
boundary has a typed preflight, explicit blast radius, recoverable failure
topology, and auditable outcome.

## Milestone 19: Joined Runtime And Store Certification

### Goal

Certify the completed Store-backed Query runtime as one system rather than a
collection of individually green crates.

### Boundary

This milestone adds no product capability. It is the courtroom that proves the
joined authority, concurrency, durability, recovery, performance, compatibility,
security, and operator claims of Milestones 1 through 18 over the certified
physical foundation.

### Adversarial Constraint

No milestone-local fixture, copied runtime, test-only adapter, model verdict,
or bounded smoke profile may stand in for production-path execution under the
full declared workload and hardware envelope.

### Must Ship

- generated milestone-to-capability-to-proof coverage manifest
- runtime-backed versus Store-backed Query parity certification
- concurrency certification over readers, writers, branches, global
  constraints, live resources, maintenance, and recovery
- fresh-process crash and indeterminate-outcome certification
- greater-than-memory, bounded-allocation, access-amplification, and latency
  interference certification
- compatibility, replication, extension, tenant, security, and operator
  certification
- controlled mutants for duplicate runtime authority, acknowledgment inversion,
  stale basis, lost update, global-lock fallback, unbounded scan, false resume,
  derived-authority promotion, and wrong-scope readmission
- reproducible evidence bundles tied to source, binary, config, schema, format,
  backend, hardware, support profile, and workload profile
- generic and representative domain certification programs

### Must Preserve

- certification remains observation and proof, never production authority
- formal models refine production outcomes rather than replace them
- smoke, CI, soak, release, and hardware profiles remain honestly distinct
- incomplete or bound-exhausted evidence cannot promote readiness

### Proof Obligations

- all three roadmap-spanning hostile scenarios below
- mutation sensitivity proving each named defect is detected and localized
- independent oracle comparison rather than self-derived expected results
- deterministic replay and shrinkage of failing schedules
- exact evidence provenance and stale-bundle rejection

### Closeout Gate

Milestone 19 is not closed until every admitted Store-backed Query family is
covered by production-boundary hostile proof and the joined system—not merely
its components—earns the claimed operating envelope.

## Roadmap-Spanning Hostile Scenarios

### Scenario A: Two-Branch Interactive Concurrency Under Crash

- five readers remain pinned to branch A and five to branch B
- two mutations submit concurrently to each branch
- exactly one mutation per branch-head generation owns active branch-write
  authority at a time, while branch A and branch B may each write concurrently
- the queued same-branch successor cannot prepare authoritative truth or emit
  mutation effects until the active writer publishes, proves no effect, or is
  resolved from an indeterminate recovery fence; basis-pinned planning reads
  remain legal
- a global uniqueness/index obligation makes one cross-branch pair coordinate
- live views and subscriptions exist on both branches
- group commit, branch-head publication, Signal routing, and Query completion
  receive deterministic crash injection at every transition
- fresh-process recovery must converge to the serial reference truth, preserve
  every acknowledged write, classify every unacknowledged durable write, retain
  old reader bases, and resume only support-honest live resources

This scenario is mandatory for Milestones 2 through 6, 8, 9, 13, and 19.

### Scenario B: Greater-Than-Memory Evolution And Maintenance Siege

- total Store size materially exceeds the configured buffer pool
- deep branches, schema evolution, lineage split/merge, large blobs, persistent
  Query indexes, analysis artifacts, and slow subscribers coexist
- foreground reads and writes continue during checkpoint, compaction, reclaim,
  scrub, tier movement, bulk migration, and derived rebuild
- memory, pin, dirty-page, allocation, I/O, amplification, and interference
  counters must remain inside the admitted envelope
- crashes and corruption are injected during every maintenance publication
- recovery must use bounded checkpoint plus tail and rebuild derived state
  without full-store materialization

This scenario is mandatory for Milestones 4 through 10, 12 through 17, and 19.

### Scenario C: Mixed-Version Replica Disaster And Governed Readmission

- a source and replicas run across the admitted rolling-version window
- partial branches, retained declarations, subscription support, blobs, and
  extension artifacts replicate under duplication, reordering, interruption,
  corruption, key rotation, and tenant-scope pressure
- the primary is lost and a replica is promoted through Part I fencing and
  Part II semantic readmission
- PITR and rollback candidates compete with replicated heads
- Query support, policy, schema, basis, and runtime authority must be freshly
  admitted before service resumes
- incompatible, wrong-scope, stale-custody, or unverifiable artifacts remain
  quarantined and cannot be promoted through operator action alone

This scenario is mandatory for Milestones 8, 10 through 13, 16, 18, and 19.

## Critical Path And Parallel Work

Milestones 1 through 12 are the non-negotiable integration path. They establish
the Query contract refactor, the bidirectional semantic/physical spine, durable
write publication, cold Store reads and hydration, real branch concurrency,
bounded residency, checkpoint/recovery, ordinary and historical Query parity,
durable Query artifacts, and blob-backed delivery.

The dependency order inside that path is strict:

- Milestone 2 must prove a real vertical Store round trip before Milestone 3
  generalizes the durable write protocol.
- Milestone 4 must make cold Store reads ordinary before Milestone 5 claims
  joined concurrency.
- Milestone 6 must prove drainage and rehydration under that concurrency before
  Milestones 7 and 8 freeze checkpoint and recovery behavior.
- Milestone 9 closes ordinary execution/pushdown before Milestone 10 closes
  historical and branch parity.
- Milestones 11 and 12 consume stable identity, recovery, access, and Query
  parity rather than inventing durable artifact or blob side paths.

After Milestone 12:

- Milestone 13 live/subscription work consumes the durable continuation
  substrate from Milestone 11 and may not rebuild it locally.
- Milestone 14 bulk work may begin earlier after Milestones 3 through 6, but it
  closes only through the canonical commit, residency, and recovery paths.
- Milestone 15 consumes the survival and lease declarations of Milestones 10
  through 14 before durable retention or reclaim closes.
- Milestone 16 begins after every existing artifact family declares retention,
  compatibility, and portability participation.
- Milestone 17 begins after compatibility, replication, and extension
  containment contracts exist.
- Milestone 18 integrates continuously but closes only after all resource and
  trust surfaces are known.
- Milestone 19 remains last.

## Completion Standard

Part II is complete only when Worth Store can honestly say:

- Worth Query is the single ordinary runtime language in runtime-backed and
  Store-backed modes
- no second semantic Store runtime or Store-local pseudo-Query exists
- `worth-application-store` owns exactly one Query participant, one physical
  Store instance, and narrow bindings between them
- backend-neutral application composition owns selected-provider completeness,
  owner opening/closing order, typed readiness join, and overall readiness—but
  no semantic recovery meaning or physical work
- the adapter is the only semantic/physical translation boundary and owns no
  independent join authority, scheduler, cache, work registry, truth, or
  recovery lifecycle
- physical and semantic Signal graphs remain distinct, reconstructible under
  their respective owners, and incapable of minting one another's authority
- one Relational authority owns MVCC truth per runtime identity
- Query read, submission, participant-lifecycle, and inspection authorities are autonomous
- independent branches make concurrent progress without a global backend lock
- exactly one mutation at a time owns active branch-write authority per
  branch-head generation; same-branch successors remain planned or queued until
  fresh admission against the resulting head
- indeterminate mutation fate fences successor admission until recovery, while
  shared global authority coordinates independently of branch ownership
- every acknowledged Query mutation survives and every indeterminate mutation
  has a recovery path
- Store-backed reads remain basis-exact and bounded for data larger than memory
- runtime state drains, clears, or rehydrates through explicit family-specific
  policy while hard memory bounds and pins remain honest
- restart reconstructs Relational truth and freshly admits Query authority
- saved queries, continuations, and blob-bearing results remain canonical and
  restart-stable where admitted
- historical, live, subscription, bulk, retention, replication, extension,
  security, and operator paths share the same canonical boundaries
- every derived durable family is classified, rebuildable, budgeted, portable,
  and accuracy-honest
- the three roadmap-spanning hostile scenarios pass through production
  boundaries with reproducible evidence

Only then has Worth Store earned the claim that the physical database and the
Worth runtime operate as one production-grade system.
