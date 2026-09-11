# WORTH Query Future Roadmap

## Purpose

This document defines the future work for `worth-query`.

It is a future-only roadmap. It does not assume the query layer is already
productized, and it does not treat query as thin convenience syntax over
runtime reads. It exists to sequence the work required to make asking for
truth as rigorous, typed, replay-honest, and live-promotable as the rest of
the WORTH stack.

The operating rule for this roadmap is:

`declare query intent once, lower it once, execute it against canonical truth`

That rule governs every milestone:

1. query meaning must be expressed as typed structures rather than strings,
   ad hoc host closures, or runtime-only conventions
2. planning, narrowing, and legality checks must happen before the hot path
   executes reads or live maintenance
3. `worth-query` owns canonical query declaration, planning, provider, result,
   and runtime-backed execution contracts; Store integration composes those
   contracts and may require prerequisite changes in Query without moving
   durable implementation work back into this roadmap
4. live delivery, historical reads, and persisted query artifacts must remain
   derived from canonical truth rather than inventing shadow read models

## Adversarial Constraint

`worth-query` must survive the following hostile condition:

> A large branch-bearing truth graph with aspect-rich entities, lineage-heavy
> identity evolution, schema drift, live subscriptions, historical reads,
> query-shaped diffs, and policy-aware masking must produce the same typed
> query result, the same narrowing decision, and the same explanation of why
> that result changed regardless of whether the read came from in-memory truth,
> store-backed historical restore, or replayed live maintenance.

If any supported path:

- lets execution rediscover semantics that should have been lowered into the
  query plan
- forces whole-entity scans when the query declared narrower aspect or scope
- makes live subscriptions interpret raw CDC instead of query-shaped intent
- changes query meaning depending on backend/storage path
- makes historical or saved-query features depend on opaque host glue instead
  of canonical query artifacts
- allows policy masking, branch targeting, or lineage traversal to drift
  between one-shot reads and live promotion

then `worth-query` has failed.

## Roadmap Rules

Rules for every remaining query item:

- each milestone must describe a real query capability boundary, not just
  "add some builders" or "wire one more adapter"
- each milestone must preserve the ownership split:
  `worth-relational` owns truth semantics, `worth-store` owns durable
  persistence, `worth-signal` owns reactive evaluation, and `worth-query`
  owns typed query expression, lowering, and result shaping
- every milestone must distinguish canonical query artifacts from derived
  runtime conveniences
- no milestone is complete until it has machine-checkable acceptance evidence
  through typed plan assertions, parity scenarios, replay checks, or hostile
  subscription/history cases
- sequence numbers express logical dependency order, not a promise that every
  later milestone must wait for every earlier integration detail to land
- every milestone must identify Store-facing contracts or deferred durable
  claims so the Store integration roadmap can close them without duplicating
  Query meaning
- every milestone must declare its own adversarial constraint
- every hot-path milestone must declare named complexity contracts and exact
  counter proof obligations
- any knowingly incomplete first ship must be marked as explicit debt rather
  than implied completeness
- named certification suites in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/worth-query/test-requirements.md)
  are the authoritative acceptance source for milestone closure

## Operating Modes

The roadmap preserves these query operating modes explicitly:

- `Runtime-backed mode`: queries plan and execute directly against
  `worth-relational` snapshot-backed reads
- `Store-backed mode`: queries execute against admitted `worth-store` surfaces
  without changing canonical query meaning; its implementation sequence and
  closure evidence live in the Store runtime integration roadmap
- `Live-promoted mode`: the same canonical query meaning is maintained through
  query-shaped incremental updates
- `Ephemeral artifact mode`: saved queries, templates, or host-bound bindings
  may exist without durable persistence before `worth-store` completes; this
  mode must remain explicit debt rather than implied product completion

## Obligation Surface Convention

Every milestone in this roadmap must be read as four separate obligation
surfaces, even when a section heading still says `Must Ship` for readability:

- `surface primitives`: the concrete query types, plan artifacts, metadata, or
  protocol-facing structures that must exist
- `semantic guarantees`: the meaning that those primitives must preserve under
  execution, live maintenance, history, policy, and replay
- `proof obligations`: the parity checks, hostile scenarios, and machine-
  checkable evidence that must exist before the milestone is honest
- `store handoff`: the exact provider, artifact, or parity contract consumed by
  the Store runtime integration roadmap, without pretending Query locally owns
  the durable implementation

If a roadmap line item only names API surface but does not also name semantic
or proof obligations, it is incomplete.

## Store Integration Handoff

This roadmap is organized around Query's semantic and runtime-backed build
order. The physical database, joined runtime, and any Query changes required
to integrate them are sequenced in the
[Worth Store Runtime And Integration Roadmap](../worth-store/runtime-integration-roadmap.md).
That roadmap is intentionally organized by implementation and dependency order,
not by a rigid source-ownership rule.

The former Store-gated Query milestones move as follows:

- former Query Milestone 10 ordinary execution and pushdown work moves to Store
  Milestone 9; historical, branch, preview, diff, and merge parity moves to
  Store Milestone 10
- former Query Milestone 11 durable saved queries, named artifacts,
  continuations, cursors, and restart-stable reload move to Store Milestone 11
- former Query Milestone 12 blob-backed delivery and large-object semantics
  move to Store Milestone 12
- joined Query/Store production certification closes in Store Milestone 19

Query still owns the canonical declaration, plan, result, provider, and
semantic parity contracts those Store milestones consume. If integration
reveals that Query must change, that change belongs in the Store roadmap's
implementation sequence rather than being mirrored here as a second milestone.

## Platform Framework Stance

`worth-query` is not a read-only helper crate. It is the intended
platform-level framework surface for ordinary domain and application code.

That means:

- ordinary developers should be able to stay inside `worth-query` for the
  majority of read, live, branch-workflow, mutation-orchestration, and
  delivery-shape work they perform
- `worth-query` may expose pass-through, lowering, orchestration, and unified
  configuration surfaces over `worth-relational`, `worth-signal`, and the
  runtime bridge
- lower crates remain the semantic authorities for truth mutation, merge
  semantics, reactive scheduling, preview-session lifecycle, writeback safety,
  persistence, and durable transport contracts
- "platform-level facade" therefore means one daily-driver import with
  authority-preserving lowering, not one giant crate that steals ownership
  from the runtimes below it

The roadmap must stay honest to both halves of that claim:

- if a domain developer would reasonably expect to do it through the query
  framework, it needs an explicit roadmap home here
- if the lower crate remains authoritative for the semantics, the roadmap must
  say so explicitly instead of implying that `worth-query` became a second
  truth, merge, or writeback engine

## Early Cross-Feature Proof Gates

The hardest failures in `worth-query` live at feature intersections rather than
inside isolated feature families. The roadmap therefore requires these
cross-feature proof gates before final certification:

- `Milestone 5` must prove live + ordering + projection parity for admitted
  query families, and must preserve the architectural seams that let
  `Milestone 9` add live + policy masking parity without redefining live query
  meaning
- `Milestone 5.1` must prove region-scoped invalidation narrowing and
  change-stream-backed delivery contracts without collapsing live query meaning
  into raw partition events or transport-local stream glue
- `Milestone 5.2` must prove preview-session query contexts preserve branch
  workflow basis identity, preview lifecycle identity, and promoted-result
  comparison semantics without ambient host orchestration
- `Milestone 5.3` must prove frontier-aware planning and deterministic parallel
  admission preserve canonical execution meaning while making planning cost
  posture explicit
- `Milestone 5.4` must prove structural correspondence and historical
  materialization-path metadata remain explicit and ambiguity-honest
- `Milestone 5.5` must prove query-authored mutation, merge, and writeback
  declarations lower into lower-crate authorities without duplicating mutation
  semantics or hiding branch-workflow truth behind host glue
- `Milestone 5.6` must prove the unified facade and unified runtime
  configuration remain authority-preserving rather than bag-shaped or
  semantics-erasing
- `Milestone 6` must prove historical + diff + result-shape parity for the
  same declared query shape
- `Milestone 7` must prove lineage + correspondence + branch-scoped comparison
  parity for admitted identity-evolution scenarios
- `Milestone 8` must prove view-shape-specific live semantics for at least
  table/detail plus one grouped or temporal view
- `Milestone 9` must prove tenant schema variation + validation + delivery-
  shape parity, and policy masking + historical reads + saved/scope-composed
  queries where those surfaces already exist
- `Milestone 9.1` must prove query-owned subscription declaration families and
  lowering preserve the same canonical query meaning across policy, tenant,
  basis, and view-shape variations without inventing a second live-query
  semantics path or a fake one-size-fits-all subscription kind
- `Milestone 9.2` must prove subscription-family-backed live delivery,
  sharing, continuation, and preview isolation preserve query-shaped parity
  for the same canonical query and admitted live family
- `Milestone 9.3` must prove Query's automatic subscription-family selection
  path remains bridge-honest, diagnostically sufficient, and
  certification-ready rather than smuggling capabilities above the bridge or
  host runtime
- `Milestone 9.3.1` must prove Query inspection can expose one bridge-owned
  cross-runtime causal explanation envelope joining relational authority,
  bridge routing/evaluation, signal invalidation/evaluation, lineage,
  provenance, and replay posture without requiring domains to spelunk lower
  runtimes
- `Milestone 9.3.2` must prove Query basis is a capability lifecycle, not a
  raw branch/snapshot identifier, so observation, mutation, replay, inspection,
  and materialization all consume phase-typed basis proofs
- `Milestone 9.3.3` must prove Query effects execute through one
  authority-scoped effect pipeline that lowers intent once and prevents
  executors from re-deciding authority, basis, strategy, or artifact policy
- `Milestone 9.3.4` must prove materialized projections expose declared,
  typed, receipt-backed fact consumption so consumers can use projection facts
  without reopening source authority
- `Milestone 9.3.5` must prove every Query-crossing intent resolves through a
  structured admission decision lattice with success, advisory, and violation
  traces before construction, lowering, or covered bridge-backed execution
- `Milestone 9.3.6` must prove lower-runtime contact is capability-routed
  through contractual boundary envelopes rather than scattered direct bridge,
  relational, signal, or store access
- `Milestone 9.3.7` must prove downstream domains can contribute typed domain
  capability posture through one public Query-owned contribution seam across
  admission, support, traceability, invariant, workflow, continuity, aftermath,
  and explanation categories, so Query can materialize canonical runtime
  artifacts without forcing domains to mint local pseudo-Query layers
- `Milestone 9.3.8` must prove serious downstream domains can enter Query as a
  first-class platform boundary through one Query-owned declaration,
  progression, routing, inspection, orchestration, and certification seam
  rather than rebuilding local pre-Query declaration, preparation, and handoff
  worlds above relational, the runtime bridge, signal, `worth-proof`, and
  `worth-foundational`
- the `Runtime API Public Stabilization Gate` must freeze the ordinary public
  workspace/handle/state/aspect/effect/intent/inspection contract after the
  runtime API facade consumes 9.1 through 9.3.8, so domain runtimes can build
  now and temporal/async milestones extend the same model later rather than
  adding parallel APIs
- the `Runtime Authoritative Mutation Evidence Gate` must freeze the ordinary
  public mutation-evidence contract after aspect-native mutation and runtime
  facade stabilization are in place, so write-heavy downstream domains do not
  rebuild target recovery, existing-truth identity binding, causality /
  provenance recovery, or authority explanation above Query, and so the Query
  public contract and bridge carry-forward contract remain one end-to-end
  evidence story rather than two drifting halves
- `Milestone 9.4` must prove the merged runtime-backed temporal/async query
  surface as one ordinary Query product lane: basis binding, declaration
  families, result-state, mixed-cause delivery, downstream delivery contract,
  and hostile certification all have to preserve canonical query meaning
  without making Query the owner of clocks, async lifecycle execution, or
  lower-cause ordering authority
- `Milestone 9.5` must prove the reusable query composition, core view-shape,
  grouped composition, projection-consumption, and preserved temporal/async
  reuse debts are closed as production-ready Query productization lanes before
  store-backed and durable milestones build on top of them
- `Milestone 9.6` must prove evidence identity, stop-class matching, and
  session label identity are runtime-owned structural contracts â€” digests
  survive formatting drift, every covered stop class is matchable without
  string operations, and session labels carry canonical identity with typed
  collision posture
- `Milestone 9.7` must prove N concurrent shared read contexts under commit
  pressure produce byte-identical results and receipts to serialized
  execution, that journal replay reconstructs identical truth and published
  artifacts, and that no reader can acquire a lock on the committed-read hot
  path or trigger derived evaluation
- `Milestone 9.8` must prove a downstream crate can author evidence reports,
  enforce the no-bypass contract, pin support posture, and obtain a valid
  honestly-postured in-memory test runtime using only Query-shipped kit
  surfaces, with closure proven by reference-consumer adoption and deletion
  of the hand-rolled equivalents
- `Milestone 9.9` must prove complete graph touch obligation authority: every
  obligation kind executable, compose/batch/read lanes dispatching, policy-aware
  mutation gates live, explicit relational graph-composition execution point,
  exact-zero duplicate enforcement, and full reference adoption in `worth-topo`
  and `worth-kernel` construction with architectural certification closure
- `Milestone 9.10` must prove graph read access planning and declarative index
  admission: declared graph reads derive operation resolutions, access shapes,
  access requirements,
  budgets, and typed execution postures before execution, making hidden N+1 and
  unbounded background indexing impossible on covered read lanes
- `Milestone 9.11` must prove downstream basis and projection authority is one
  Query-owned canonical artifact: consumers declare required meaning through
  fluent facade DX and cannot pair, reconstruct, restamp, or promote authority
  from independently valid bases, receipts, facts, labels, or digests
- `Milestone 9.12` must prove Query's public facade contains one sealed
  authority path per capability: consumers cannot mint authority from raw
  digests or identities, skip scoped basis admission, invoke the raw intent
  engine, assert subscription posture, or carry unscoped preview/inspection
  artifacts into operational work
- `Milestone 9.13` must prove ordinary Query usage is capability-oriented and
  declarative, then prove serious domain setup is installed into one concrete
  Query runtime, and finally prove exact Foundational-native aspect value
  meaning survives the complete consumer journey: consumers describe desired
  read, live, historical, workflow, inspection, and domain-extension outcomes
  while Query owns canonicalization, admission, planning, lowering, execution
  routing, lifecycle, receipt assembly, canonical domain identity, runtime
  installation, derived operation registries, and proof-bearing value DX
  without creating a second scalar or struct authority
- `Milestone 9.13.1` must establish a livable Query iteration foundation in
  independently executable slices: bulk the selected compiler denials, remove
  Worth UI coupling, isolate cold certification, dismantle the manually
  assembled library-test binary, repair repeated reconstruction hotspots, and
  extract declaration and installation as permanent production authorities;
  each slice inventories only the boundary it immediately changes and no proof-
  management platform is introduced
- `Milestone 9.13.2` must complete the covered Query authority split by
  extracting admission, execution, and publication one semantic surface at a
  time, retargeting certification, and cutting audience facades with parity so
  Cargo package boundaries, dependency direction, and naturally local tests
  replace hand-maintained authority selection; `worth-query` remains the
  product composition root for unrelated behavior and may lower only one way
  into each sole destination authority
- `Milestone 9.14` must prove downstream projection authority is one
  non-detachable runtime-affine capability rooted in one installed operating-
  world entry root, explicit graph participation, and installed typed domain
  operation definitions: consumers cannot locally reconstruct stable operation
  meaning, semantic truth dependencies, Relational change meaning, installed
  truth-to-Signal aspect correspondence, alternate entry roots, graph bridges,
  workflow progression, replay, reversal, publication, or lineage, or recombine independently valid
  installations, graph adapters, domain capabilities, stage receipts,
  replay/reversal scopes, definitions, completions, bases, facts, receipts,
  support projections, dependency labels, aspect keys, truth-delta targets,
  Signal slots/masks, correspondence scopes, equivalence tokens, reporting
  digests, invalidation labels, collection patches, leases, or lifecycle
  artifacts into operational power; atomic cross-domain and admitted cross-
  graph binding, Query-minted workflow traces, ordinary re-execution, cert-only
  replay, typed aftermath, derived publication, lineage and promotion-on-
  reference, native access,
  consumer support, dependency impact, shared execution admission, managed
  leases, compatibility, invalidation, collection windows, patch delivery,
  replacement, rebind, and disposal remain Query-owned, proof-bearing, and
  exactly accounted
- `Milestone 9.15` closes the honest pre-commit foundation: governed artifacts
  and occurrences, bounded native access, real resource admission and managed
  runs, sealed provider sessions, basis-complete decision read-sets, isolated
  proposed state, and installed invariant execution
- `Milestone 9.16` proves the ordinary front door through a real authenticated
  asynchronous bank world: schema-derived typed references, Authentik identity,
  capability-, purpose-, conflict-, and disclosure-aware graph admission,
  double-entry and estate effects, provider-proven compare-and-commit,
  installed application queries bound to the existing no-N+1 graph-read access
  plans, public read/mutation/workflow/history/live facades, governed
  break-glass, actionable recovery, accepted aftermath/external-effect
  publication, provisional undo/redo implementation evidence, and separate
  user-node processes
- `Milestone 9.16.1.1` repairs the installed graph-contract integrity exposed
  after 9.16.1 closure: declaration-owned application aspect identity and
  revision, one installed native schema catalog, exact typed operation read
  and touch scopes, execution consumption of those same contracts, and
  complete typed aftermath inspection
- `Milestone 9.16.2` establishes stable portable package identity, complete
  typed export, bounded reconstruction with fresh Query validation, and a
  deterministic neutral release archive without introducing application-state
  persistence or a physical runtime
- the `Milestone 9.17` umbrella closes through `9.17.1`, corrective
  `9.17.1.1`, corrective `9.17.1.2`, `9.17.2`, and `9.17.3`: first exact in-memory owner component
  bases and Relational branch-local MVCC, then independently borrowable
  Relational ports plus lifecycle-total settlement/retention, then final
  Relational basis/lifecycle and Signal services, then dedicated Runtime World
  composite history/currentness, then complete Query carriage, facade cutover,
  and hostile certification
- `Milestone 9.17.4` hardens the application API after that certified foundation:
  entry-owned value and intent bindings, explicit contributions, request-bound
  execution, retained/page/live resources, installed domain handlers, exact
  recovery, full Bank adoption, and primary invariant/bulk-construction proof;
  it replaces application-owned Query pipelines before 9.18 begins
- `Milestone 9.18` accepts tree-based semantic undo and redo as freshly
  admitted composite-history operations over exact source world commits and
  target product-branch heads; it replaces the provisional linear Phase 8
  experiment without creating a Query-owned history stack
- `Milestones 9.19` through `9.22` add advanced generic computation through
  the same front door: managed access and verified footprints, correlated and
  set-oriented execution, governed decision evidence, and occurrence-safe
  reuse; each milestone closes its own reference adoption, public facade,
  documentation, and provider-independent hostile evidence
- Query certification must export reusable semantic parity oracles for ordinary,
  historical, live, policy, artifact, and delivery contracts; Store Milestones
  9 through 13 and Store Milestone 19 consume those oracles against physical
  boundaries
- any Store-driven Query refactor must preserve the same canonical declaration,
  plan, result, provider, authority, and explanation semantics already proven
  by the runtime-backed milestones

## Critical Path And Store Handoffs

The Query critical path closes the canonical runtime-backed framework and the
contracts required by physical integration:

- `Milestone 1` -> `Milestone 2` -> `Milestone 3` -> `Milestone 4` ->
  `Milestone 5` -> `Milestone 5.1` -> `Milestone 5.2` -> `Milestone 5.3` ->
  `Milestone 5.4` -> `Milestone 5.5` -> `Milestone 5.6` -> `Milestone 6` ->
  `Milestone 7` -> `Milestone 8` -> `Milestone 9` -> `Milestone 9.1` ->
  `Milestone 9.2` -> `Milestone 9.3` -> `Milestone 9.3.1` through
  `Milestone 9.3.8` -> `Runtime API Public Stabilization Gate` ->
  `Runtime Authoritative Mutation Evidence Gate` -> `Milestone 9.4` through
  `Milestone 9.13` -> `Milestone 9.13.1` -> `Milestone 9.13.2` ->
  `Milestone 9.14` -> `Milestone 9.15` -> `Milestone 9.16` ->
  `Milestone 9.16.1.1` -> `Milestone 9.16.2` ->
  `Milestone 9.17.1` -> `Milestone 9.17.1.1` ->
  `Milestone 9.17.1.2` -> `Milestone 9.17.2` ->
  `Milestone 9.17.3` -> `Milestone 9.17.4` ->
  `Milestone 9.18` -> `Milestone 9.19` ->
  `Milestone 9.20` -> `Milestone 9.21` -> `Milestone 9.22` ->
  `Milestone 13`

The numbered order remains the semantic dependency order. The condensed ranges
above do not weaken any intervening milestone, acceptance gate, or proof
obligation.

Store handoffs are explicit:

- runtime-backed execution, access planning, authority sealing, deterministic
  submission, publication evidence, and exact Foundational-native value meaning
  close in Query before Store Milestone 1 begins integration refactoring
- installed domain operation resolution, downstream projection capability
  binding, dependency-impact compilation, equivalent-work sharing and lease
  lifecycle, native access, compatibility, invalidation, and operational-
  identity opacity close in Query before Store integration or provider
  certification can inherit those boundaries
- managed domain artifact carriage, provider execution sessions, real post-state
  invariant execution, basis-complete compare-and-commit, resource-bounded
  execution, access-product lifecycle, membership coverage, realized
  footprints, and set-oriented partition admission close in Query before Store
  integration or provider certification can inherit those boundaries
- Store Milestones 1 through 4 consume those contracts to establish the backend
  seam, semantic/physical lowering, durable publication join, and cold reads
- Store Milestones 5 through 10 close joined concurrency, residency, recovery,
  ordinary pushdown, and historical parity
- Store Milestones 11 through 13 close durable query artifacts, continuations,
  blob delivery, and live restart semantics
- Store Milestone 19 closes production certification across runtime and physical
  boundaries using Query's exported semantic parity oracles

Until those Store milestones close, Query documentation and APIs must label
application-state durability, Store-native graph execution, physical fetch,
residency, replication, distributed recovery, blob, and physical pushdown
claims as external handoffs rather than local completion. Milestone 9.16.2
establishes package portability only, and Milestones 9.17 through 9.21 keep
their authoritative application worlds resident in memory. Query must not grow
a temporary PostgreSQL or shadow Store model to make later Store-native claims
appear complete.

## Milestone 1: Typed Query Expression And Result Shape Foundation

### Goal

Make query intent a first-class typed artifact before execution, live
promotion, or persistence concerns are allowed to sprawl.

### Adversarial Constraint

Different builder paths, helper layers, and host binding surfaces must
normalize to the same canonical query artifact and result-shape meaning for the
same declared intent, with no downstream phase allowed to recover missing
meaning from ambient host context.

### Why This Milestone Exists

`worth-query` cannot honestly plan, validate, optimize, or subscribe to reads
until it has one canonical representation of:

- what is being queried
- which aspects are being projected
- what result shape the caller expects
- what scope the query is allowed to traverse

Without this milestone, every later feature would be forced to reverse-engineer
host-specific query builders or execution closures.

### Must Ship

- one public `worth-query` facade and crate boundary
- typed query expression families for:
  - entity/detail reads
  - collection reads
  - aspect projection
  - bounded relation traversal
  - typed result shapes
- composable query fragments rather than string-based query construction
- canonical query identity/digest surfaces for structurally equal queries
- explicit query context shell that can later carry branch, history, policy,
  and live-promotion parameters without changing query meaning
- diagnostics that explain how a query expression normalized into its canonical
  typed form

### Must Preserve

- query is expression authority, not truth authority
- result shapes stay typed rather than degrading into dynamic maps
- structurally equal queries normalize identically
- host code does not bypass the facade and invent alternate query ASTs

### Complexity / Proof Obligations

- name canonicalization and query-identity contracts in terms of clause count,
  projection width, traversal clause count, and result-shape width
- expose exact counters for normalized clause count, projection entry count,
  result-shape field count, and canonicalization fallback count
- prove canonical query digest parity for equivalent construction paths

### Allowed Debt

- ergonomic builder sugar may remain `Debt` if canonical query identity and
  facade-normalized meaning are already frozen and parity-proven
- alternate host-local query ASTs may not exist as debt

### Sequencing Notes

This belongs first because every later milestone depends on one canonical query
artifact and one canonical result-shape artifact rather than host-specific
construction paths.

### Parallelization Notes

Once canonical query identity is frozen, Milestone 2 validation work and early
Milestone 3 planning work can proceed in parallel.

### Store Dependency

This milestone is not blocked on `worth-store`.

### Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- the `Canonical Query Normalization Parity Test` in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/worth-query/test-requirements.md)
  passes with canonical machine-checkable artifacts

- equivalent query builder paths normalize into the same canonical query
  artifact
- typed result shapes remain structurally aligned with the declared projection
- query identity survives round-tripping through the public facade without
  host-specific loss of meaning

## Milestone 2: Schema-Aware Validation, Aspect Projection, And Predicate Semantics

### Goal

Make query legality explicit so invalid, over-broad, or schema-dishonest
queries fail before execution.

### Adversarial Constraint

No illegal, schema-mismatched, over-broad, or structurally ambiguous query may
advance into planning by relying on runtime fallback, host-side repair, or
implicit whole-entity widening.

### Why This Milestone Exists

The vision only works if queries are constrained enough for the runtime to
optimize and narrow. That requires the query layer to understand schema,
aspects, field types, relation kinds, and bounded traversal legality before it
attempts execution.

### Must Ship

- schema-aware query validation at construction time where compile-time proof
  is not available
- typed predicate expressions over aspect fields
- workflow-aware predicate families as first-class typed predicates rather than
  host-local post-filters
- explicit aspect projection legality checks
- structured content aspect query legality over schema-declared content blocks
  where the schema admits queryable structured content
- bounded traversal legality checks over declared relation kinds and depth
- typed ordering declarations over query-visible fields
- validation diagnostics for:
  - unknown aspects
  - incompatible field predicates
  - illegal traversal edges
  - invalid result-shape bindings
  - schema-version mismatch at query construction

### Must Preserve

- validation must consume authoritative schema semantics from
  `worth-relational`
- query construction must not execute storage reads to decide legality
- aspect-aware reads remain the default instead of silently widening to
  whole-entity fetches
- illegal queries fail explicitly before hot execution

### Complexity / Proof Obligations

- name validation contracts for predicate validation, projection validation,
  traversal validation, and result-shape binding validation
- expose exact counters for validated predicates, validated projection items,
  rejected clauses, traversal legality checks, and whole-entity widening
  denials
- prove that schema-invalid queries fail before any execution-path admission

### Allowed Debt

- compile-time proof coverage may remain `Debt` where construction-time proof
  already exists and is canonical
- any silent widening from illegal or unsupported projection/predicate forms
  may not ship as debt

### Sequencing Notes

This belongs before planning because the planner must consume already-legal
query artifacts instead of rediscovering legality during execution lowering.

### Parallelization Notes

Can proceed in parallel with the canonical-artifact hardening of late
Milestone 1. It should finish before core Milestone 3 plan lowering closes.

### Store Dependency

This milestone is not blocked on `worth-store`.

### Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- the `Schema-Aware Rejection And Projection Legality Test` in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/worth-query/test-requirements.md)
  passes with canonical machine-checkable artifacts

- schema-invalid queries fail during construction with typed diagnostics
- workflow-aware predicates normalize and validate like any other predicate
  family instead of bypassing the canonical query artifact
- structured content queries fail explicitly when the schema/content contract
  does not admit the requested projection or predicate
- legal aspect projections and predicates lower deterministically
- traversal legality remains bounded and explainable rather than becoming
  host-defined graph walking

## Milestone 3: Query Planning And Snapshot-Backed Execution

Status:
Closed on 2026-04-14 for runtime-backed one-shot execution. Store-backed
parity, store pushdown, and durable snapshot-plus-tail semantics remain
explicit debt until `worth-store` can support them honestly.

### Goal

Lower typed query intent into executable plans that read canonical truth
through stable snapshots instead of ad hoc runtime access.

### Adversarial Constraint

Runtime-backed execution, store-backed execution where admitted, and type-bound
host binding paths must all converge to the same query result meaning for the
same canonical query and basis, without the executor rediscovering legality,
projection, or scope semantics that the planner should already have fixed.

### Why This Milestone Exists

Typed query expressions are not enough on their own. The query layer needs an
explicit planner that decides:

- what truth surfaces must be read
- what aspects and relations can be narrowed
- what execution path is legal for this query shape
- what result-shaping work belongs before or after runtime reads

This is the milestone where `worth-query` stops being syntax and becomes an
actual query subsystem.

### Must Ship

- query planner that lowers typed expressions into proof-carrying execution
  plans
- snapshot-backed execution contracts for one-shot reads
- explicit separation between planning, execution, and result shaping
- authoritative-runtime execution path against `worth-relational`
- type-bound execution descriptors that let higher layers bind consumer inputs
  to canonical query plans without redefining query semantics in route or
  handler glue
- result metadata showing what aspects, scopes, and ordering bases were used
- execution diagnostics for fallback, widening, and unsupported plan shapes
- counters for planned read breadth, projected aspect count, traversal breadth,
  and fallback path selection

### Must Preserve

- execution must read from stable snapshots rather than live mutable truth
- executor must not rediscover legality or narrowing decisions already solved
  by the planner
- query results remain derived from canonical truth surfaces
- type-bound binding descriptors remain query-owned metadata while route,
  server, or UI plumbing stays outside `worth-query`
- unsupported plan shapes fail explicitly instead of silently widening

### Complexity / Proof Obligations

- name planning and execution contracts for plan build, plan normalization,
  snapshot-backed execution, and fallback-path admission
- expose exact counters for planned read breadth, projected aspect count,
  traversal breadth, fallback broadening count, and snapshot basis resolution
- prove runtime-backed/store-backed parity where both paths are admitted
- prove an admitted runtime route/plan distinction through certification so
  planner-owned route semantics do not collapse into one happy-path lane

### Allowed Debt

- store-backed pushdown may remain `Debt` while runtime-backed execution is
  canonical and store parity is still blocked on `worth-store`
- executor rediscovery of planner-owned semantics may not ship as debt

### Sequencing Notes

This belongs before collection scale semantics, live promotion, and historical
reads because all of those depend on proof-carrying plans and snapshot-honest
execution.

### Parallelization Notes

Core runtime-backed planning should finish before the rest of the roadmap
builds on it. Store-backed plan variants can advance in parallel as
`worth-store` matures.

### Store Dependency

- Core runtime-backed execution is not blocked on `worth-store`.
- Full completion of store-aware execution parity, snapshot-plus-tail restore
  parity, and honest store pushdown is blocked on `worth-store` milestones for
  canonical commit persistence, snapshots, and durable restore.

### Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- the `Planner / Executor / Binding Parity Test` in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/worth-query/test-requirements.md)
  passes with canonical machine-checkable artifacts

- identical query/context input lowers to identical execution plans
- one-shot execution reads through stable snapshots and returns the declared
  typed result shape
- type-bound execution descriptors round-trip to the same canonical plan as
  direct query invocation
- planner and executor diagnostics can explain when a query used a narrowed
  path versus an authoritative fallback path
- runtime-backed execution and any admitted store-backed execution agree for
  the same truth basis where store support exists

## Milestone 4: Collection Semantics, Ordering, Pagination, And Bounded Traversal

Status:
Closed on 2026-04-14 for runtime-backed collection semantics. Live
maintenance, historical collection replay, durable cursor continuation, and
store-backed collection parity remain intentionally deferred to later
milestones.

### Goal

Make large-surface reads honest and product-grade rather than leaving
collection behavior as host-local loops.

### Adversarial Constraint

Large collections, ordered pages, bounded traversals, aggregations, rollups,
and CDC-shaped outputs must remain proportional to declared query breadth and
must not secretly devolve into unbounded scans, unstable cursor semantics, or
host-side recomputation.

### Why This Milestone Exists

A query layer that can only do detail reads is not a real product surface.
WORTH Query needs first-class collection semantics that keep cardinality,
ordering, and traversal cost explicit enough for both the runtime and the
caller to reason about.

### Must Ship

- collection query planning as a first-class path rather than repeated
  detail-query composition
- typed ordering semantics
- opaque cursor-based pagination
- bounded result-set declarations
- bounded relation materialization and eager loading contracts
- aggregation query families with explicit grouping and tolerance declarations
- relational rollup query families over declared relation edges
- query-time derived field declarations that are validated and planned as part
  of the canonical query artifact rather than post-processing host code
- CDC-shaped output/result families for integration-facing query consumers
- collection/result metadata that explains cursor position, ordering basis,
  and truncation behavior
- counters for collection width, page width, traversal depth, and
  materialization breadth

### Must Preserve

- offset/limit must not masquerade as stable pagination
- query APIs must reveal cardinality and traversal cost honestly
- bounded traversal must stay bounded by declared depth/scope rather than
  drifting into arbitrary graph walks
- rollups and query-time derived fields must remain derived result semantics
  rather than accidental stored authority
- CDC-shaped output must remain query-shaped and projection-honest rather than
  degenerating into raw runtime CDC
- collection execution remains snapshot-stable under active mutation

### Complexity / Proof Obligations

- name collection planning, cursor-advance, bounded traversal, aggregation,
  rollup, and CDC-shaped rendering contracts
- expose exact counters for collection width, page width, traversal depth,
  materialization breadth, aggregate input breadth, and CDC output width
- prove cursor stability and query-shaped CDC parity for the same basis

### Allowed Debt

- admitted fast paths for specific aggregation or rollup families may remain
  `Debt` if the exact fallback path is explicit and parity-proven
- unstable cursor semantics or host-postprocessed derived fields may not ship
  as debt

### Sequencing Notes

This belongs after core planning because collection semantics are the first
place where cost dishonesty and query-shaped output drift become large-scale
product risks.

### Parallelization Notes

Collection execution, rollup/aggregation semantics, and CDC-shaped output
formatting can advance in parallel once Milestone 3 has frozen plan and basis
artifacts.

### Store Dependency

- Core collection semantics are not blocked on `worth-store`.
- Restart-stable cursor durability and persisted page-resume semantics are
  blocked on later `worth-store` durability work and should not be claimed in
  this milestone.

### Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- the `Collection, Cursor, Rollup, And CDC Shape Parity Test` in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/worth-query/test-requirements.md)
  passes with canonical machine-checkable artifacts

- ordered collection queries return stable cursor-advancable pages for one
  snapshot basis
- bounded traversal and eager materialization stay within declared scope
- aggregation, rollup, and query-time derived-field results remain tied to the
  declared truth basis and declared projection rather than host recomputation
- CDC-shaped output matches the same canonical query meaning as ordinary result
  execution for the same query and basis
- collection counters and diagnostics explain why a query touched the breadth
  it touched

## Milestone 5: Live Query Promotion And Incremental Result Maintenance

Status:
Closed on 2026-04-15 for runtime-backed live promotion, query-shaped patching,
replay parity, suppression, and policy-hardening. Historical/policy-composed
live semantics, durable continuation, and store-backed live parity remain
intentionally deferred to later milestones.

### Goal

Make any admitted read query promotable to a live query without changing the
query expression, while keeping live maintenance query-shaped instead of
event-shaped.

### Adversarial Constraint

A live-promoted query that starts from a stable basis and consumes truth
changes incrementally must converge to the same result as repeated fresh
re-execution of the canonical query, regardless of update order, suppression,
or subscription longevity.

### Why This Milestone Exists

The WORTH Query vision breaks if reads, subscriptions, and reactive refreshes
are separate products. This milestone makes live promotion a property of query
execution context rather than a parallel API surface.

### Must Ship

- live-promotion context for admitted query families
- query-to-signal lowering for incremental maintenance
- bridge-aware invalidation metadata sufficient to decide query relevance
- query-shaped incremental result patches for:
  - detail reads
  - ordered collections
  - bounded materialized relations
- suppression of irrelevant truth changes before result delivery
- diagnostics explaining why a truth change did or did not affect a live query
- counters for invalidation breadth, patch width, and suppressed updates

### Must Preserve

- live promotion must not change query meaning
- signal scheduling remains owned by `worth-signal`
- patch-to-invalidation routing remains owned by the runtime bridge
- consumers must receive query-shaped changes rather than raw CDC they have to
  reinterpret
- live maintenance must remain snapshot- and basis-honest

### Complexity / Proof Obligations

- name invalidation-relevance, live patch construction, and suppression
  contracts
- expose exact counters for invalidation breadth, live patch width, suppressed
  updates, and full re-execution fallbacks
- prove live + ordering + projection parity for admitted query families, and
  prove that later live + policy masking semantics can compose without
  redefining live query meaning

### Allowed Debt

- some query families may remain non-live-promotable as explicit `Debt` while
  admitted families are fully parity-proven
- raw CDC delivery disguised as query-shaped maintenance may not ship as debt

### Sequencing Notes

This belongs before historical/diff and view-shape semantics because it is the
first proof that query meaning survives time rather than just one-shot reads.

### Parallelization Notes

Can progress in parallel with early Milestone 6 context work once Milestone 3
planning and Milestone 4 collection semantics are stable.

### Store Dependency

- Core live promotion is not blocked on `worth-store`.
- Durable subscription resume across restart is blocked on `worth-store`
  durability for persisted cursors/checkpoints and must not be claimed here.

### Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- the `Live Promotion Convergence And Suppression Test` in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/worth-query/test-requirements.md)
  passes with canonical machine-checkable artifacts

- the same query expression can execute as one-shot or live without semantic
  drift
- irrelevant truth changes are suppressed before query-shaped patch delivery
- query-shaped live patches preserve ordering, membership, and projection
  semantics
- replaying the same canonical truth changes yields the same live query result
  evolution

## Milestone 5.1: Region-Scoped Live Invalidation And Delivery Contracts

Platform integration companion:
[WORTH Signal Milestone 13.1](../WORTH_signal/milestone-13.1-plan.md) governs
the completed production Runtime Bridge-to-Signal-to-Query cutover. The
cross-runtime closeout is recorded in
[Milestone 13.1 closeout](../WORTH_signal/milestone-13.1-closeout.md); local
region-scoped artifacts remain only one owner inside that composed path.

### Goal

Make live query narrowing and delivery contracts region-aware, stream-honest,
and explicitly query-shaped instead of broad aspect-only invalidation or ad hoc
CDC transport glue.

### Adversarial Constraint

When a truth change only affects a semantically bounded region or partition of
the query's declared scope, live maintenance must narrow to that region and
emit delivery metadata that preserves the same query meaning without widening
to full-aspect or full-collection recomputation.

### Why This Milestone Exists

Milestone 5 closed the first honest live substrate, but it still leaves one
important production-grade gap: the lower runtimes can already speak in region-
and partition-scoped invalidation and change-stream contracts, while the query
roadmap only guarantees broad live relevance and query-shaped patches.

That is not enough for geometry-grade live delivery, integration-grade stream
contracts, or query surfaces that want to narrow below whole-aspect scope.

### Must Ship

- region- or partition-aware live invalidation metadata for admitted live query
  families
- query-declared locality predicates or materially equivalent plan-owned region
  narrowing surfaces
- change-stream-backed delivery contract lowering for query-shaped CDC/live
  output where the bridge admits it
- diagnostics and counters explaining region matches, region suppressions, and
  stream-contract admission or denial

### Must Preserve

- region-scoped narrowing remains derived from planner-owned query semantics and
  lower-runtime locality contracts rather than host heuristics
- delivery contracts stay query-shaped instead of exposing raw partition events
  as the consumer contract
- durable stream continuation remains deferred until later durable milestones

### Complexity / Proof Obligations

- name region-match, partition-narrowing, and change-stream lowering contracts
- expose exact counters for matched regions, suppressed region changes,
  stream-contract admissions, and region-widening denials
- prove region-scoped live suppression and change-stream-backed delivery remain
  parity-safe with the same canonical live query meaning

### Allowed Debt

- unsupported region families may remain explicit `Debt`
- raw partition or raw CDC events masquerading as query delivery may not ship
  as debt

### Sequencing Notes

This belongs immediately after Milestone 5 because it is live-maintenance
hardening, not a later historical or policy feature.

### Parallelization Notes

Can progress in parallel with early preview-context and planning-hardening work
once the Milestone 5 live substrate is frozen.

### Store Dependency

- Runtime-backed region narrowing and stream-contract semantics are not blocked
  on `worth-store`.
- Durable stream resume and persisted checkpoints are handed to Store
  Milestone 11.

### Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- region-scoped live invalidation stays narrower than broad aspect invalidation
  when lower-runtime locality contracts admit that narrowing
- query-shaped delivery contracts can lower into formal stream contracts
  without changing query meaning
- unsupported region or stream-contract combinations fail typed and early

## Milestone 5.2: Preview Session Query Contexts And Branch Workflow Foundations

Status:
Closed on 2026-04-16 for runtime-backed preview-session query contexts,
promotion-parity comparison, preview-live composition, and workflow-foundation
artifacts. Durable preview replay reload, persisted workflow artifacts, and
store-backed preview parity remain intentionally deferred.

### Goal

Make speculation and preview sessions first-class query contexts so branch-
native preview workflows stay inside the query framework instead of devolving
into raw bridge orchestration.

### Adversarial Constraint

A query bound to a preview or speculative session must preserve explicit basis,
preview-lifecycle identity, and preview-versus-promoted comparison semantics
without ambient host glue deciding what session it really targeted or how the
result should be interpreted.

### Why This Milestone Exists

The vision already wants AI, workflow, and geometry users to operate against
branch-local truth and speculative branches. The runtime bridge already has a
real preview-session lifecycle. If the query roadmap does not expose that as a
native basis context, developers will drop out of query-land for one of the
most important branch-native workflows in the product.

### Must Ship

- query contexts that can bind to `BridgePreviewSession` or materially
  equivalent preview-session artifacts
- explicit preview-session basis metadata and preview-lifecycle metadata on
  query plans/results
- distinction between read-only preview evaluation and promotable preview
  evaluation
- query-native comparison surfaces for preview result versus promoted result
- preview-live admission, maintenance, drift denial, and explicit rebind over
  the corresponding admitted Milestone 5 and 5.1 live families
- branch-workflow foundation artifacts that later mutation/merge milestones can
  extend without redefining preview semantics

### Must Preserve

- preview-session lifecycle authority remains owned by the runtime bridge
- preview contexts do not become host-local branch aliases
- preview queries preserve ordinary canonical query meaning apart from the
  explicitly declared preview basis
- preview-live may not silently fall back to authoritative live truth

### Complexity / Proof Obligations

- name preview-basis resolution, preview-lifecycle identity, and promoted-
  result comparison contracts
- name preview-live admission, drift, and explicit-rebind contracts
- expose exact counters for preview-session admissions, preview-basis
  resolutions, preview/promotion comparison runs, preview-live admissions,
  preview-live drift denials, preview-live explicit rebinds, and invalid
  preview-context denials
- prove preview-session query contexts remain parity-safe with the same
  canonical query shape and declared preview basis

### Allowed Debt

- unsupported preview families may remain explicit `Debt`
- ambient host orchestration that silently selects or rewrites the preview
  basis may not ship as debt

### Sequencing Notes

This belongs before general branch/history expansion because preview workflows
are a special but load-bearing form of basis identity that later branch and
merge work must inherit.

### Parallelization Notes

Can progress in parallel with region-scoped live hardening and planning
hardening once Milestone 5 proof-bearing live artifacts are stable.

### Store Dependency

- Runtime-backed preview-session query contexts are not blocked on
  `worth-store`.
- Durable preview replay and persisted branch-workflow artifacts remain later
  durable work.

### Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- preview-session-bound queries preserve explicit basis and lifecycle identity
- preview-versus-promoted comparison remains query-native and typed
- preview-live remains basis-explicit and either maintained, denied, or
  explicitly rebound without silent retargeting
- unsupported preview-session query combinations fail before semantic drift

## Milestone 5.3: Frontier-Aware Planning And Deterministic Parallel Admission

### Goal

Make planning consume signal-frontier cost posture and deterministic parallel-
admission knowledge instead of planning in isolation and hoping the executor
figures it out later.

### Adversarial Constraint

Planning decisions for bulk queries, live maintenance, and multi-query bundles
must preserve the same canonical query meaning whether they execute serially or
in parallel, while breadth and parallel admission decisions remain explicit
rather than rediscovered by runtime heuristics.

### Why This Milestone Exists

Milestone 3 established proof-bearing plans, but not yet the stronger planning
story that a platform framework should have when `worth-signal` already owns
frontier and deterministic parallel-admission machinery. Query should consume
that structural knowledge at plan time instead of acting as if every cost
decision is local and serial by default.

### Must Ship

- frontier-aware planning metadata for admitted query families
- deterministic parallel-admission metadata on planned execution routes for
  admitted bulk/live families
- diagnostics that explain why a route admitted parallel execution or fell back
  to serial execution
- counters for predicted breadth, realized breadth, parallel admissions, and
  serial fallbacks

### Must Preserve

- `worth-signal` remains authoritative for frontier and parallel-admission
  semantics
- the executor consumes lowered admission decisions instead of speculating
  about parallel safety at runtime
- serial and parallel lanes must preserve identical canonical query meaning

### Complexity / Proof Obligations

- name frontier-breadth, parallel-admission, and serial-fallback contracts
- expose exact counters for frontier lookups, predicted breadth, realized
  breadth, admitted parallel batches, and serial fallback decisions
- prove deterministic serial/parallel parity for admitted planned families

### Allowed Debt

- unsupported query families may remain serial-only as explicit `Debt`
- executor-side speculative parallel admission may not ship as debt

### Sequencing Notes

This belongs after Milestone 5 because it hardens planning using already-frozen
live and query artifacts without reopening Milestone 3 itself.

### Parallelization Notes

Can progress in parallel with preview-session and correspondence/historical
hardening once the required lower-runtime planning inputs are stable.

### Store Dependency

This milestone is not blocked on `worth-store`.

### Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- frontier-aware planning decisions are explicit and digest-bearing
- admitted serial and parallel lanes remain semantically identical
- unsupported parallel families fail closed or remain explicit debt

## Milestone 5.4: Structural Correspondence And Historical Evaluation Contracts

### Goal

Strengthen the branch/history/identity story so structural correspondence and
historical materialization-path honesty become explicit query artifacts instead
of implied bridge details.

### Adversarial Constraint

Correspondence and historical queries must stay explicit about whether they
were resolved through lineage, structural fingerprinting, retained snapshots,
delta replay, or full reconstruction, without silently collapsing ambiguity or
materialization-path differences into one vague "comparison result."

### Why This Milestone Exists

The vision already mentions structural fingerprints and rich historical reads,
but the current roadmap reads too lineage-centric and too generic about
historical basis. This milestone closes that precision gap before later branch,
history, and workflow milestones build more surface area on top.

### Must Ship

- structural-fingerprint-based correspondence as a first-class query artifact
  beside lineage-based correspondence
- query result metadata describing historical materialization path for admitted
  historical reads
- explicit compatibility/admission contracts for historical evaluation where
  the lower runtimes cannot serve a request honestly
- diagnostics for structural ambiguity, lineage/structural disagreement, and
  unsupported historical materialization paths

### Must Preserve

- lineage remains authoritative continuity; structural correspondence remains
  advisory unless explicitly promoted by lower-truth semantics
- historical evaluation authority remains in lower runtimes, not host caches
- ambiguity and materialization-path differences remain explicit in results

### Complexity / Proof Obligations

- name structural-correspondence, historical-materialization-path, and
  compatibility-admission contracts
- expose exact counters for structural candidates considered, ambiguity
  denials, historical-path admissions, and path-compatibility denials
- prove correspondence and historical artifacts remain typed, replay-safe, and
  ambiguity-honest

### Allowed Debt

- unsupported structural families or historical materialization paths may
  remain explicit `Debt`
- silent collapse of ambiguity or hidden materialization-path substitution may
  not ship as debt

### Sequencing Notes

This belongs before the broader current Milestone 6 and Milestone 7 work
because those milestones should build on explicit correspondence and historical
contracts rather than implying them.

### Parallelization Notes

Can progress in parallel with planning hardening and early branch-workflow
surfaces once the required lower-runtime artifacts are stable.

### Store Dependency

- Runtime-backed structural correspondence and admitted historical-path
  metadata are not blocked on `worth-store`.
- Durable historical restore remains later store-backed work.

### Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- structural correspondence is explicit and distinct from lineage continuity
- historical query results expose materialization-path meaning where admitted
- unsupported or ambiguous cases fail typed and early

## Milestone 5.5: Query-Orchestrated Mutation, Merge, And Writeback Declarations

### Goal

Make `worth-query` a real platform workflow surface for domain developers by
letting query-authored mutation, merge, and writeback declarations lower into
relational and bridge authorities without forcing developers to drop into raw
lower-crate APIs for common branch-native workflows.

### Adversarial Constraint

Mutation intents, merge intents, conflict preview, post-merge inspection, and
query-triggered writeback declarations must all preserve explicit authority
boundaries, conflict meaning, and replay/delivery honesty without turning
`worth-query` into a second mutation engine or hiding branch-workflow truth
behind host glue.

### Why This Milestone Exists

If `worth-query` is the daily-driver framework surface, it cannot stop at reads
and live subscriptions. Domain developers need to stay inside the query facade
for context resolution, preview, merge inspection, commit lowering, and
writeback-trigger declaration. Otherwise the platform fractures precisely at
the workflows branch-native products use most.

### Must Ship

- query-authored mutation intents that lower into relational commit strategy
  requests
- query-authored branch-workflow declarations for at least:
  - preview / compare
  - conflict inspection
  - merge intent
  - post-merge result inspection
- query-triggered writeback declarations that lower into bridge writeback
  declarations where admitted
- diagnostics and counters for mutation-intent lowering, merge admission,
  conflict classification, and writeback admission or denial

### Must Preserve

- `worth-relational` remains authoritative for commit strategy, merge
  semantics, and mutation truth
- the runtime bridge remains authoritative for preview-session lifecycle,
  writeback safety, idempotence, causality, and replay artifacts
- `worth-query` owns declaration, lowering, orchestration surface, and result
  shaping, not a second mutation engine

### Complexity / Proof Obligations

- name mutation-intent lowering, merge-intent lowering, conflict-preview, and
  writeback-declaration contracts
- expose exact counters for lowered mutation intents, merge previews, merge
  denials, writeback admissions, and writeback denials
- prove branch-native workflow declarations lower into lower-crate authorities
  without semantic drift or hidden host orchestration

### Allowed Debt

- unsupported mutation or writeback families may remain explicit `Debt`
- host-local branch workflow glue that bypasses canonical lowering may not ship
  as debt for any admitted workflow family

### Sequencing Notes

This belongs before later history/policy/store milestones because it corrects
the biggest platform-level omission in the current roadmap: query as the
application-facing workflow framework, not just the read surface.

### Parallelization Notes

Can progress in parallel with unified-facade/config work once the authority
boundaries for commit, merge, preview, and writeback lowering are frozen.

### Store Dependency

- Runtime-backed mutation/merge/writeback lowering is not blocked on
  `worth-store`.
- Durable workflow continuation and persisted workflow artifacts remain later
  work.

### Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- admitted mutation and merge workflow declarations lower into relational
  authorities without semantic drift
- admitted writeback declarations lower into bridge authorities without hiding
  causality or safety semantics
- unsupported workflow families fail typed and early

## Milestone 5.6: Unified Application Facade And Unified Runtime Configuration

### Goal

Make `worth-query` the explicit daily-driver facade and configuration surface
for ordinary domain/application code without erasing lower-crate authority
boundaries or collapsing configuration into a bag.

### Adversarial Constraint

A unified facade and unified runtime configuration must let developers use the
platform through one coherent surface while preserving subsystem ownership,
typed capability boundaries, and structurally sectioned configuration rather
than flattening the stack into ambiguous pass-through glue.

### Why This Milestone Exists

The product story for `worth-query` only fully lands when developers can treat
it as the main framework import instead of shopping among `worth-relational`,
`worth-signal`, and the runtime bridge. But that facade must stay authority-
preserving and architecture-shaped, or it just becomes a bag of convenience
APIs and config fields.

### Must Ship

- one explicit application-facing facade posture for `worth-query`
- pass-through or composed public surfaces for admitted lower-runtime
  capabilities that application developers should access through query
- unified `WORTHQueryConfig` or materially equivalent configuration surface
  sectioned by subsystem ownership
- capability advertisement and diagnostics explaining which composed surfaces
  are admitted, deferred, or unsupported

### Must Preserve

- the facade is unified for developers but not semantics-erasing for the
  underlying runtimes
- configuration must mirror subsystem boundaries rather than becoming a flat
  bag
- unsupported composed capabilities remain explicit rather than implied by one
  broad "platform config" type

### Complexity / Proof Obligations

- name facade composition, capability advertisement, and configuration-section
  resolution contracts
- expose exact counters for capability lookups, configuration-section
  resolutions, and unsupported-composition denials
- prove unified facade and unified configuration surfaces preserve lower-crate
  authority and admitted capability boundaries

### Allowed Debt

- unsupported composed surfaces may remain explicit `Debt`
- flat bag-shaped unified configuration and semantics-erasing facade shortcuts
  may not ship as debt

### Sequencing Notes

This belongs after the workflow-composition milestone because the facade and
config surfaces should compose real admitted platform capabilities, not merely
promise them.

### Parallelization Notes

Can progress in parallel with the earliest branch/history/policy preparation as
long as unsupported capabilities remain explicitly gated.

### Store Dependency

- Core unified facade and unified configuration work is not blocked on
  `worth-store`.
- Any config fields that claim durable resume, store-backed parity, or durable
  artifact support remain gated by the later store-backed milestones.

### Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- application-facing facade composition is explicit and authority-preserving
- unified runtime configuration stays sectioned by subsystem responsibility
- capability advertisement and support metadata remain in sync with admitted
  composed surfaces

## Milestone 6: Branch-Scoped, Historical, And Diff Query Contexts

### Goal

Make branch targeting, time-travel, and comparison queries first-class context
and query capabilities rather than separate host APIs.

### Adversarial Constraint

Current-state reads, branch-scoped reads, historical reads, and diff queries
over the same canonical query must preserve identical query meaning apart from
their declared basis difference, with no hidden basis substitution, host cache
repair, or result-shape drift.

### Why This Milestone Exists

WORTH systems are branch-native and history-native. If `worth-query` cannot
express "read this branch," "read this commit," and "show me the difference
between these two truth bases" through the same query model, then developers
will be pushed back into ad hoc runtime calls that fracture the stack.

### Must Ship

- branch-scoped query contexts
- historical query contexts targeting branch heads, commits, or admitted
  snapshots
- compatibility with preview-session-derived bases and admitted historical
  materialization-path metadata established by Milestones 5.2 and 5.4
- diff-query expression families for comparing two declared truth bases
- result metadata that names the basis used for every branch/historical/diff
  query
- diagnostics for unsupported historical bases, ambiguous comparison bases, and
  scope mismatch
- parity surfaces between present-state reads and historical reads for the same
  declared query shape

### Must Preserve

- branch and history meaning remain owned by `worth-relational` and
  `worth-store`
- historical queries do not mutate truth or fabricate history through host
  caches
- preview-session basis identity and historical materialization-path identity
  remain explicit rather than ambient host context
- diff queries return query-shaped comparison artifacts rather than raw storage
  deltas
- basis identity remains explicit end-to-end

### Complexity / Proof Obligations

- name branch-basis resolution, historical-basis resolution, and diff-shaping
  contracts
- expose exact counters for historical basis lookups, diff input breadth,
  comparison-scope width, and unsupported-basis denials
- prove historical + diff + result-shape parity for the same canonical query

### Allowed Debt

- durable store-backed historical execution may remain `Debt` until
  `worth-store` can support it honestly
- hidden basis substitution or history reconstruction through ambient host
  caches may not ship as debt

### Sequencing Notes

This belongs after Milestones 5.2 and 5.4 because basis identity across time,
preview lifecycle, and historical materialization path are the next major
places where query meaning can fracture.

### Parallelization Notes

Branch/head context work can begin earlier, but full historical/diff closure
should follow stable Milestone 3 planning plus the preview-basis and
historical-contract hardening from Milestones 5.2 and 5.4.

### Store Dependency

- Branch-head reads and any retained-history reads already exposed by the
  runtime are not blocked on `worth-store`.
- Full completion for durable point-in-time restore, snapshot-targeted
  execution, restart-stable historical parity, and store-backed diff execution
  is blocked on `worth-store` milestones for snapshots, delta layering, and
  replication-safe artifact identity.

### Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- the `Historical / Diff / Basis Parity Test` in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/worth-query/test-requirements.md)
  passes with canonical machine-checkable artifacts

- the same declared query shape can run against current and historical truth
  bases without changing its structural meaning
- diff queries produce structured, typed change sets aligned to the declared
  projection and scope
- where store-backed historical execution exists, it matches runtime-backed
  historical truth for the same basis

## Milestone 7: Lineage, Correspondence, And Identity-Evolution Queries

### Goal

Make identity evolution queryable without exposing raw lineage internals or
forcing consumers to hand-assemble history walks.

### Adversarial Constraint

Identity-evolving queries must preserve explicit continuity, ambiguity, and
rejection semantics across branch-local, historical, and comparison reads
without silently promoting correspondence guesses into authoritative identity.

### Why This Milestone Exists

Branch and history alone are not enough for truth systems where entities can be
replaced, split, merged, or corresponded across versions. Query consumers need
first-class lineage-aware reads if they are going to compare, inspect, and
subscribe to evolving truth honestly.

### Must Ship

- lineage traversal query expressions
- correspondence-aware query expressions for cross-branch or cross-version
  comparison where the runtime admits them, including structural-fingerprint-
  backed correspondence where admitted
- identity-evolution result shapes that distinguish:
  - current entity truth
  - historical antecedents
  - replacements/splits
  - ambiguous or rejected correspondences
- diagnostics for unsupported lineage traversals and ambiguous correspondence
- query metadata that exposes the lineage/correspondence basis used for a
  result

### Must Preserve

- lineage semantics remain owned by `worth-relational`
- correspondence must not silently become authoritative identity
- structural correspondence must remain explicit about when it is advisory,
  ambiguous, or rejected
- ambiguous identity evolution fails or reports ambiguity explicitly
- lineage-aware reads remain query-shaped and replay-honest

### Complexity / Proof Obligations

- name lineage traversal and correspondence-resolution contracts
- expose exact counters for lineage steps traversed, correspondence candidates
  considered, ambiguous correspondence denials, and fallback identity breaks
- prove lineage + correspondence + branch-scoped comparison parity for
  admitted identity-evolution cases

### Allowed Debt

- unsupported lineage or correspondence classes may remain explicit `Debt`
  while admitted classes are fully typed and parity-proven
- silent continuity through ambiguous correspondence may not ship as debt

### Sequencing Notes

This belongs after Milestones 5.4 and 6 because identity evolution only
becomes meaningful once basis-aware reads and explicit correspondence
contracts already exist.

### Parallelization Notes

Can progress in parallel with early view-shape and scope work once Milestone 6
has frozen branch/history basis artifacts and Milestone 5.4 has frozen the
explicit correspondence vocabulary.

### Store Dependency

- Core lineage-aware query semantics are not blocked on `worth-store`.
- Restart-stable lineage/correspondence parity across persisted history is
  blocked on `worth-store` durable lineage artifact support and should be
  treated as completion debt until store lands that support.

### Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- the `Lineage And Correspondence Query Parity Test` in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/worth-query/test-requirements.md)
  passes with canonical machine-checkable artifacts

- lineage traversal yields typed, explainable results across admitted identity
  evolution classes
- correspondence-aware comparison stays explicit about ambiguity and rejection
- lineage-aware reads over replayed or restored truth match original query
  meaning for the same lineage basis

## Milestone 8: Scopes, Templates, And View Shapes

### Goal

Turn query composition into a reusable product surface rather than leaving every
host to rebuild domain vocabulary and presentation intent locally.

### Adversarial Constraint

Reusable scopes, templates, and admitted view shapes must expand
to the same canonical query meaning as direct construction, and view-shape
semantics must affect planning and live maintenance structurally rather than
surviving only as cosmetic type tags.

### Why This Milestone Exists

Typed query primitives are necessary, but not sufficient. Developers need
schema-validated reuse surfaces such as named scopes, parameterized templates,
and intent-driven view shapes if WORTH Query is going to be the normal way
people ask for truth.

### Must Ship

#### Surface Primitives

- composable named scopes as first-class query fragments
- query templates with typed parameter slots
- intent-driven view shapes for at least:
  - table/detail
  - grouped/kanban-style collections
  - timeline/chart-style results where admitted
  - inspector-style detail projection with live aspect focus
- diagnostics for invalid parameter binding, incompatible scope composition,
  and unsupported view-shape/query combinations
- query-shape metadata that higher layers can use for delivery and live-patch
  formatting

#### Semantic Guarantees

- scopes must preserve the same canonical query meaning as their expanded form
- templates must bind parameters without introducing host-local semantic drift
- each admitted view shape must affect planning, invalidation narrowing,
  delivery formatting, and live patch semantics explicitly rather than acting
  as display-only sugar
- inspector/detail view behavior must preserve narrow aspect-focused live
  projection instead of widening to whole-entity reads

#### Proof Obligations

- scope-composed queries and directly-authored queries must normalize to the
  same canonical artifact
- at least one grouped or temporal view must prove view-shape-specific live
  patch semantics rather than only type-surface existence
- inspector/detail live projection must prove aspect-focused invalidation
  narrowing under change

#### Store Handoff

- durable saved-query persistence, portability, and restart-stable workspace
  artifacts are handed to Store Milestone 11

### Must Preserve

- scopes and templates remain typed query artifacts rather than macro-like
  string substitution
- view shapes do not become presentation-owned shadow query languages
- host layers consume query-shape metadata instead of redefining the query

### Complexity / Proof Obligations

- name scope expansion, template binding, and view-shape lowering contracts
- expose exact counters for scope fragments expanded, template parameters
  bound, and view-shape-specific patch events
- prove view-shape-specific live semantics for at least table/detail plus one
  grouped or temporal view

### Allowed Debt

- absent view-shape families may remain explicit `Debt`
- any shipped view shape that lacks planning, invalidation, delivery, and live
  semantics may not ship as debt
- durable saved-query semantics are intentionally handed to Store Milestone 11

### Sequencing Notes

This belongs after lineage/history and the workflow/facade insertions because
composition and presentation intent must sit on top of already-honest query
meaning and platform workflow surfaces rather than inventing either.

### Parallelization Notes

Scopes/templates and view-shape semantics can progress in parallel once
Milestones 4 through 6 plus 5.5 and 5.6 have stabilized collection, live,
basis, and platform-facade behavior.

### Store Dependency

- Scopes, templates, and view-shape semantics are not blocked on `worth-store`.
- Durable saved-query persistence, portability, and restart-stable workspace
  artifacts are blocked on `worth-store`; until then, any saved-query-like
  surfaces may exist only as ephemeral or host-local artifacts and must not be
  marketed as complete.

### Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- the `Scope / Template / View-Shape Semantic Parity Test` in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/worth-query/test-requirements.md)
  passes with canonical machine-checkable artifacts

- scopes and templates compose into the same canonical query artifacts as
  equivalent direct construction
- admitted view shapes affect planning, invalidation narrowing, delivery
  formatting, and live patch semantics rather than only surfacing as type tags
- inspector/detail live projection remains aspect-focused under change

## Milestone 9: Policy-Aware Narrowing, Tenant Scope, And Delivery Contracts

Status:
Closed on 2026-04-21 for runtime-backed policy-aware narrowing, tenant
truth/schema basis admission, relationship-proof admission/denial, policy-aware
execution seam lowering, live admission, delivery contracts, and certification.
Store-backed execution parity, durable policy cursors, durable artifact reload,
durable delivery metadata reload, restart-stable subscription metadata, and
durable tenant/query artifact portability remain intentionally deferred to
WORTH Store and later milestones. See
[milestone-9-closeout.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/worth-query/milestone-9-closeout.md).

### Goal

Make policy, masking, and multi-tenant narrowing structural query concerns
rather than after-the-fact filtering glued on by hosts.

### Adversarial Constraint

Policy masks, tenant branch narrowing, tenant-scoped schema variation, and
relationship-proof denials must never leak masked truth, alter query meaning
across one-shot/live/historical modes, or depend on post-read redaction.

### Why This Milestone Exists

The query layer is the last place where over-broad reads can still be stopped
before truth is touched. If policy-aware narrowing and tenant scoping are not
structural here, then higher layers will either over-read and redact later or
invent incompatible policy behavior across read, live, and historical surfaces.

### Must Ship

#### Surface Primitives

- aspect-level policy masking in query planning
- branch-level access scoping in query validation/execution context
- tenant-scoped branch narrowing where the platform resolves tenant truth by
  branch
- tenant-scoped schema awareness so query validation and projection use the
  tenant's active schema rather than assuming one global schema
- graph-native relationship-proof predicate/query families for admitted
  relationship-calculus style access or legality proofs
- delivery-shape metadata for server-facing transport layers
- policy composition rules for admitted mutation, merge, writeback, and
  streamed-delivery declarations exposed through the query framework
- diagnostics for masked aspects, denied branches, ambiguous tenant context,
  tenant-schema mismatch, relationship-proof denial, and policy/query
  incompatibility

#### Semantic Guarantees

- policy masking must happen before execution so masked aspects are never read
- tenant scoping must narrow both truth basis and schema basis explicitly
- relationship-proof queries must remain typed query semantics rather than
  host-local authorization callbacks
- delivery-shape metadata must preserve the exact masked/projected result
  meaning seen by the caller
- policy and tenant context must compose with admitted mutation, merge,
  writeback, and stream-contract declarations without post-read repair
- one-shot, live, historical, and saved/scope-composed queries must honor the
  same policy and tenant basis

#### Proof Obligations

- policy-masked and unmasked variants of the same query must prove that masked
  aspects never enter the plan or live-maintenance path
- tenant-specific schema variation must prove validation and result-shape
  parity across at least two divergent tenant schema states
- relationship-proof queries must prove explicit denial and non-leakage when
  the proof chain is broken
- delivery-shape metadata must stay parity-safe across one-shot, live, and
  historical execution for the same policy basis

#### Store Handoff

- durable delivery cursors, restart-stable subscription metadata, and persisted
  tenant/query artifacts remain incomplete until `worth-store` lands the
  required durable support

### Must Preserve

- policy authority remains owned by schema/platform layers rather than by query
  host code
- masked aspects must not be read and then discarded later
- tenant scoping must narrow truth basis explicitly rather than through ambient
  hidden filters
- delivery metadata remains derived from the canonical query and policy basis

### Complexity / Proof Obligations

- name policy masking, tenant basis resolution, tenant-schema validation, and
  delivery-shape derivation contracts
- expose exact counters for masked projection entries, tenant basis resolutions,
  tenant-schema validation branches, relationship-proof denials, and delivery
  metadata derivations
- prove tenant schema variation + validation + delivery-shape parity, and
  policy masking parity across one-shot, live, and historical execution

### Allowed Debt

- durable tenant/query artifact persistence may remain `Debt` until
  `worth-store` supports it
- policy masking by post-read redaction or host-local authorization callbacks
  may not ship as debt

### Sequencing Notes

This belongs after scopes, view shapes, and the workflow/facade insertions
because policy and tenant narrowing must govern the full composed query and
platform surface, not just primitive query forms.

### Parallelization Notes

Relationship-proof and tenant-schema work can progress in parallel, but final
closure should wait until scopes, saved queries, delivery shapes, and admitted
workflow/facade surfaces are stable enough to prove parity across them.

### Store Dependency

- Core policy-aware narrowing is not blocked on `worth-store`.
- Store-backed policy execution parity is Store Milestone 9 and 10 scope.
- Durable delivery cursors, restart-stable subscription metadata, and
  persisted tenant/query artifacts are blocked on `worth-store` and remain
  Store Milestone 11 scope.

### Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- the `Policy, Tenant Schema, And Relationship-Proof Boundary Test` in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/worth-query/test-requirements.md)
  passes with canonical machine-checkable artifacts

- masked aspects never appear in the execution plan or result
- denied branch or tenant access fails before storage/runtime reads execute
- tenant-scoped schema variation changes validation/projection behavior
  explicitly and deterministically
- broken relationship-proof chains fail closed without leaking masked or
  unauthorized truth
- the same policy basis produces the same query narrowing for one-shot, live,
  and historical execution

## Milestone 9.1: Query-Owned Subscription Declaration Families, Lowering, And Admission

### Goal

Make subscriptions first-class query artifacts by lowering admitted live query
meaning into explicit subscription declaration families and bridge-facing
subscription plans rather than treating long-lived observation as host-side
glue around live queries.

### Adversarial Constraint

The same canonical query, live family, policy basis, tenant basis, and
view-shape intent must lower into the same subscription declaration family and
bridge subscription plan regardless of whether the caller authored it
directly, through a scope/template/saved artifact, or through a runtime facade
helper, with no path allowed to infer subscription meaning from ambient host
observer state or a fake default subscription kind.

### Why This Milestone Exists

Milestone 9 froze policy-aware narrowing, tenant/schema basis resolution,
relationship-proof admission, and caller-visible delivery shape. Those
artifacts now define what a live query is allowed to mean.

What still does not exist is one query-owned answer to:

- what subscription identity corresponds to this canonical live query
- what basis does that subscription bind to
- what bridge subscription contract should it lower into
- what makes two live requests the same subscription versus different
  subscriptions
- why was a subscription admitted or denied

Without this milestone, WORTH Query can continue to speak about "live" while
still relying on hidden runtime conventions for the actual subscription
mechanism.

### Must Ship

- canonical query-owned subscription declaration-family artifacts derived from
  admitted live query meaning
- subscription family selection, identity, and equivalence surfaces distinct
  from raw query identity, delivery shape, and bridge stream identity
- lowering from live query plans into bridge-native subscription declarations
  and admission requests
- basis-bound subscription declaration for current, branch-local, historical,
  and admitted preview contexts where the live family supports them
- policy-aware, tenant-aware, and relationship-proof-aware subscription
  admission
- view-shape-aware subscription shaping for admitted detail, collection,
  grouped, and inspector-style live families
- explicit lowering from query declaration families into admitted bridge
  declaration families and admitted `worth-signal` observation and delivery
  strategies
- diagnostics that explain subscription declaration, lowering, basis binding,
  and denial

### Must Preserve

- `worth-query` remains the owner of query semantics and result shaping, not
  the owner of bridge subscription protocol semantics
- subscription lowering must consume the same canonical policy/tenant/basis
  artifacts as one-shot and historical execution
- unsupported subscription combinations fail typed and early instead of
  widening into raw CDC or host observer callbacks
- equivalent live requests normalize to one subscription-family meaning before
  bridge activation
- `worth-query` does not invent its own observer semantics; it chooses among
  admitted family lowerings that already map into bridge and `worth-signal`
  strategy space

### Complexity / Proof Obligations

- name subscription declaration, lowering, basis binding, and admission
  contracts
- expose exact counters for admitted subscription declarations, denied
  declarations, grouped/detail lowering variants, basis-bound declarations, and
  bridge-lowering fallback count
- prove that equivalent live query inputs lower to equivalent
  subscription-family artifacts and bridge declarations

### Allowed Debt

- durable subscription artifact persistence and reload may remain `Debt` until
  `worth-store` supports them
- host-local subscription assembly, ambient observer inference, or CDC-shaped
  fallback may not ship as debt

### Sequencing Notes

This belongs immediately after Milestone 9 because policy narrowing, tenant
basis, and caller-visible delivery shape must be frozen before Query can lower
subscriptions honestly.

### Parallelization Notes

Bridge-facing lowering and query-owned declaration identity can progress in
parallel, but final closure should wait until both agree on canonical
subscription equivalence and denial semantics.

### Store Dependency

- Core runtime-backed subscription declaration and lowering are not blocked on
  `worth-store`.
- Store-backed restart parity remains Store Milestone 13 scope.
- Durable subscription artifacts and reload semantics remain Store Milestone 11
  scope.

### Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- the `Query Subscription Declaration And Lowering Parity Test` in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/worth-query/test-requirements.md)
  passes with canonical machine-checkable artifacts

- equivalent live query inputs lower to the same canonical
  subscription-family declaration and bridge-facing subscription plan
- policy, tenant, basis, and view-shape differences that change live meaning
  also change subscription meaning explicitly
- unsupported or ambiguous subscription bindings fail before activation
- no admitted path interprets raw CDC or one fixed baked-in subscription kind
  as a substitute for query-shaped subscription intent

## Milestone 9.2: Subscription-Family-Backed Live Delivery, Sharing, And Lifecycle Parity

### Goal

Make active query subscriptions first-class runtime objects with explicit
lifecycle, sharing, continuation, and preview isolation so live maintenance is
honestly lowered through admitted subscription families rather than merely
"live-promoted" in name.

### Adversarial Constraint

One-shot reads, live-promoted queries, shared-equivalent subscriptions,
continuation after identity evolution, and preview-scoped subscriptions must
all preserve the same canonical query meaning and caller-visible query-shaped
delivery for the admitted live family, without consumer pacing, fanout shape,
or preview churn redefining what the query means.

### Why This Milestone Exists

Milestone 9.1 can declare and lower subscriptions, but declaration alone is
not enough for the product surface we actually want.

WORTH Query still needs to own:

- active subscription handles and lifecycle
- sharing and deduplication for equivalent query subscriptions
- query-shaped delivery over active subscriptions
- continuation and remap semantics when identity evolves
- preview-scoped subscription behavior and discard/promotion boundaries

Without this milestone, Query would have subscription declarations on paper but
still no honest runtime story for active long-lived subscriptions.

### Must Ship

- query subscription handles and active lifecycle surfaces
- query-shaped delivery contracts for subscription-backed maintenance
- equivalent-subscription sharing and multi-consumer fanout semantics
- continuation and remap handling for admitted identity-evolution scenarios
- explicit preview-scoped query subscription behavior, discard semantics, and
  promotion-boundary interaction
- grouped-baseline, derived-view, and admitted view-shape integration for
  subscription-backed live maintenance
- active family-aware delivery behavior that preserves which admitted
  subscription family and `worth-signal` strategy were selected
- diagnostics and counters for fanout, continuation, preview discard, and
  subscription delivery behavior

### Must Preserve

- one-shot and subscription-backed lanes preserve the same canonical query
  meaning apart from the explicitly declared live lifecycle
- sharing does not redefine query meaning or delivery semantics
- preview subscriptions remain isolated from authoritative subscriptions unless
  promoted through an explicit authority boundary
- continuation remains explicit under lineage, correspondence, and branch
  divergence rather than being inferred from host cache coincidence
- active lifecycle semantics remain family-aware rather than collapsing all
  admitted live families into one generic runtime lane

### Complexity / Proof Obligations

- name subscription lifecycle, sharing, continuation, preview isolation, and
  delivery contracts
- expose exact counters for active subscriptions, shared fanout width,
  continuation remaps, preview discard residue checks, and delivery batches or
  patch groups
- prove parity across one-shot, subscription-backed live, shared-consumer, and
  preview/discard lanes for admitted families
- prove at least two admitted subscription families remain parity-safe under
  sharing, continuation, and query-shaped delivery

### Allowed Debt

- durable continuation checkpoints and restart-stable subscription metadata may
  remain `Debt` until `worth-store` supports them
- consumer-local caches, hidden fanout heuristics, or preview residue after
  discard may not ship as debt

### Sequencing Notes

This belongs after Milestone 9.1 because Query needs canonical subscription
declarations before it can honestly own active subscription lifecycle and
sharing.

### Parallelization Notes

Lifecycle and sharing work can progress in parallel with preview isolation and
continuation work, but final closure should wait until grouped/derived
delivery, preview discard, and continuation all prove parity for the same
admitted subscription families.

### Store Dependency

- Core runtime-backed subscription lifecycle, sharing, continuation, and
  preview isolation are not blocked on `worth-store`.
- Store-backed restart and snapshot-plus-tail continuation remain Store
  Milestone 13 scope.
- Durable subscription checkpoints, reload, and restart-stable metadata remain
  Store Milestone 11 scope.

### Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- the `Subscription Lifecycle Sharing And Preview Parity Test` in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/worth-query/test-requirements.md)
  passes with canonical machine-checkable artifacts

- active subscription delivery remains query-shaped and parity-safe with
  one-shot meaning for the same admitted live family
- equivalent subscriptions can share one active maintenance path without
  changing meaning
- continuation across admitted identity-evolution scenarios remains explicit
  and typed
- discarded preview subscriptions leave no authoritative query residue

## Milestone 9.3: Subscription Family Diagnostics, Bridge Parity, And Runtime Certification

### Goal

Make Query's automatic subscription-family selection path bridge-honest,
diagnosable, and certified so first-class subscriptions are not just
implemented, but explainable as an authority-preserving lowering into the
bridge and signal layers.

### Adversarial Constraint

For every admitted query subscription family, Query's automatic subscription
path must be explainable as the same semantic request a careful manual host
could assemble through canonical query artifacts plus bridge subscription
contracts plus admitted `worth-signal` observation and delivery strategies,
with diagnostics and certification sufficient to prove that Query is not
inventing hidden semantics above the bridge.

### Why This Milestone Exists

Milestones 9.1 and 9.2 can make subscriptions real, but they still leave one
dangerous gap:

- Query could automate subscriptions in a way that is impossible to explain in
  bridge terms
- diagnostics could stop at "live handle exists" instead of exposing lowered
  subscription meaning
- runtime support claims could outrun actual certified subscription families

This milestone exists to close that honesty gap before store-backed and durable
milestones build on top of it.

### Must Ship

- query-owned subscription inspector and diagnostics artifacts
- explicit bridge-parity explanation surfaces showing how query subscription
  declarations lower into bridge subscription declarations and admitted
  lifecycle behavior
- support and admission reporting for subscription-capable query families
- runtime certification rows for declaration, lifecycle, sharing, continuation,
  preview isolation, and bridge parity
- canonical subscription bundle artifacts sufficient for offline diagnosis of
  admitted runtime-backed subscription paths
- diagnostics that report which admitted query family, bridge family, and
  `worth-signal` strategy lowering were selected

### Must Preserve

- bridge remains the authority for bridge subscription protocol semantics
- signal remains the authority for observation execution and scheduling
- diagnostics richness may change retained detail but not subscription meaning
- unsupported subscription families remain explicit non-admitted surfaces
  rather than "experimental magic"
- certification must prove family selection and lowering honesty, not merely
  one generic subscription lifecycle

### Complexity / Proof Obligations

- name bridge-parity explanation, subscription diagnostics, support reporting,
  and runtime certification contracts
- expose exact counters for bridge-parity comparisons, unsupported family
  denials, diagnostics bundle emissions, and certified subscription family
  coverage
- prove that every admitted automatic subscription family has a bridge-facing
  explanation and at least one hostile certification path
- prove that admitted family variation is visible and mechanically distinct in
  diagnostics and certification artifacts

### Allowed Debt

- durable subscription artifact replay and store-backed restart certification
  remain explicit handoffs until Store Milestones 11 and 13 close
- undocumented hidden lowering paths or uncertified supported subscription
  families may not ship as debt

### Sequencing Notes

This belongs after Milestone 9.2 because Query needs actual subscription
lifecycle behavior before it can certify bridge parity and diagnostic
sufficiency honestly.

### Parallelization Notes

Inspector work, support reporting, and certification bundle work can progress
in parallel, but final closure should wait until the same admitted
subscription-family matrix is covered by diagnostics, capability reporting, and
hostile certification.

### Store Dependency

- Runtime-backed subscription diagnostics, bridge parity, and certification are
  not blocked on `worth-store`.
- Store-backed subscription execution parity remains Store Milestone 13 scope.
- Durable subscription continuation and replay remain Store Milestone 11 scope.

### Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- the `Query Subscription Bridge Parity And Diagnostic Sufficiency Test` in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/worth-query/test-requirements.md)
  passes with canonical machine-checkable artifacts

- every admitted automatic subscription family can be explained through
  canonical query-owned subscription artifacts and bridge-facing lowering
- diagnostics can localize declaration, basis, lifecycle, continuation,
  preview, and bridge-parity failures mechanically
- support metadata and admitted runtime behavior stay in sync for
  subscription-capable query families
- diagnostics can distinguish declaration-family changes from ordinary
  lifecycle-instance changes

## Milestone 9.3.1: Cross-Runtime Causal Diagnostics And Query Inspection

### Goal

Make Query inspection the ordinary public surface for explaining why a
query-observed result changed, did not change, failed, was denied, or replayed
differently across relational, runtime bridge, and signal boundaries.

### Adversarial Constraint

A downstream domain must be able to ask Query inspection for a causal
explanation of a query observation and receive one typed, machine-checkable
artifact anchored to the Query operational receipt that produced the
observation. Its digests and evidence references must join relational
authority, bridge routing/evaluation/source/structural/stream/preview/writeback
records, signal invalidation/evaluation/forensic availability, lineage,
provenance, and replay posture without direct imports from runtime-bridge
diagnostics, relational internals, or signal graph internals.

### Why This Milestone Exists

Milestone 9.3 proves subscription-family diagnostics and bridge parity, but it
does not yet close the general "why did this happen?" boundary. If domains have
to assemble explanations by reaching into the runtime bridge, relational, and
signal layers themselves, Query has failed to expose the explanation surface
that its public inspection contract needs.

This milestone gives that boundary a Query-owned roadmap home while preserving
lower-runtime authority: the bridge owns the cross-runtime causal envelope,
relational owns truth authority, signal owns invalidation/evaluation evidence,
and Query owns inspection admission, redaction, and public materialization.

### Specification

The governing milestone spec is
[milestone-9.3.1.md](./milestone-9.3.1.md).

### Must Ship

- Query inspection APIs and artifacts for cross-runtime causal explanations
- causal observation anchors and evidence-reference contracts derived from
  existing Query operational artifacts rather than post-hoc lower-runtime
  searches
- ordered implementation gates for anchor adapters, evidence-reference indexes,
  Query admission, bridge envelope assembly, Query materialization, and
  certification; each gate must close before the next production phase consumes
  it
- a bridge-owned causal explanation envelope carrying relational authority,
  bridge route/evaluation/source/structural/stream/preview/writeback/replay,
  signal invalidation/evaluation/forensic availability, lineage, provenance,
  replay posture, and materialization-policy digests
- success/advisory/violation admission artifacts so redacted or narrowed
  explanations do not collapse into a binary admitted/denied wall
- worth-proof-backed or equivalent proof-bearing progression for phase ordering,
  sealed witness minting, fixed-shape evidence-reference collections, and
  Query/bridge trust-boundary readmission
- first-class performance contracts and slope counters for anchor derivation,
  evidence-reference resolution, admission, bridge envelope assembly,
  redaction, materialization, and public artifact serialization
- typed denial artifacts for missing bridge route evidence, missing signal
  evidence, incompatible relational authority, policy-redacted diagnostics, and
  unsupported explanation families
- cold-path richness controls so expanded explanation detail never widens
  hot-path query execution or signal invalidation
- support metadata and certification rows for runtime-backed causal
  explanation families, with durable/store-backed replay called out as later
  milestone debt

### Must Preserve

- relational remains the authority for truth, commits, snapshots, and
  relational decision evidence
- runtime bridge remains the authority for bridge protocol, route/evaluation,
  writeback, preview, historical materialization, and cross-runtime envelope
  assembly
- signal remains the authority for observation, invalidation, scheduling,
  lineage, and provenance evidence
- Query remains the authority for canonical query intent, inspection admission,
  redaction, result-shape context, and public artifacts
- downstream domains consume explanations through Query inspection rather than
  stitching lower-runtime diagnostics locally

### Complexity / Proof Obligations

- prove changed, suppressed, denied, branch/preview, and replayed observations
  all produce typed causal inspection artifacts
- prove bridge, relational, and signal digests agree with the lower-runtime
  records they summarize
- prove Query consumes existing lower-runtime diagnostic, forensic, causality,
  provenance, and authority records rather than building a parallel diagnostics
  authority
- prove each phase emits exact performance counters and slope digests before the
  next phase consumes its artifact
- prove worth-proof shape digests or equivalent proof-shape artifacts prevent
  phase skipping, raw collection substitution, stale proof reuse, and WORTHd
  lower-runtime authority witnesses
- prove missing evidence fails as typed diagnostic denial and redacted or
  narrowed evidence becomes typed advisory detail rather than best-effort
  narrative output
- prove diagnostic richness affects only cold-path materialization, not query
  meaning, planning, subscription semantics, or signal invalidation
- prove Worth-style consumers can delete direct explanation stitching across
  runtime bridge, relational, and signal internals once this artifact exists

### Allowed Debt

- durable causal explanation archives, restart-stable expanded narratives, and
  store-backed replay reconstruction remain Store Milestones 8, 11, and 13
  handoffs
- domain-specific prose renderers may remain domain-owned if they consume the
  Query causal inspection artifact instead of lower-runtime internals

### Sequencing Notes

This belongs after Milestone 9.3 because bridge-honest subscription diagnostics
prove the narrow live-query explanation lane first. It belongs before the
Runtime API Public Stabilization Gate because inspection is part of the public
runtime API contract and should not be frozen while this boundary is still
being handled as domain glue.

### Store Dependency

Runtime-backed causal diagnostics are not blocked on `worth-store`. Durable
causal archives, persisted expanded inspection narratives, store-backed replay
reconstruction, and restart-stable causal envelope reload remain Store
Milestones 8, 11, and 13 scope.

### Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- the `Cross-Runtime Causal Explanation Envelope Test` in
  [test-requirements.md](./test-requirements.md)
  passes with canonical machine-checkable artifacts
- Query inspection can explain query-observed changed, suppressed, denied,
  branch/preview, and replayed outcomes through one typed public artifact
- bridge-owned causal envelopes carry digests for relational authority, bridge
  route/evaluation/source/structural/stream/preview/writeback/replay, signal
  invalidation/evaluation/forensic availability, lineage, provenance, replay
  posture, and materialization policy
- public Query artifacts preserve lower-runtime authority names instead of
  flattening them into Query-owned narrative facts
- downstream domains can consume causal explanations without direct imports
  from runtime bridge diagnostics, relational runtime internals, or signal graph
  internals

## Milestone 9.3.2: Query Basis Capability Lifecycle

### Goal

Make every Query basis a phase-typed capability lifecycle rather than a raw
branch, head, preview, snapshot, historical, tenant, or policy identifier.

### Specification

The governing milestone spec is
[milestone-9.3.2.md](./milestone-9.3.2.md). The closeout record is
[milestone-9.3.2-closeout.md](./milestone-9.3.2-closeout.md).

### Adversarial Constraint

A consumer must not be able to observe, mutate, replay, inspect, or materialize
against a basis unless Query has proven that the requested basis family is
eligible for that operation and has emitted a self-describing basis envelope
that names its authority, scope, visibility, lifecycle, and permitted next
transitions.

### Phase Contract

```text
RawBasisIntent
  -> NormalizedBasisIntent
  -> BasisEligibility | DeniedBasisCapability
  -> AdmittedBasisCapability
  -> ScopedExecutionOrObservationBasis
  -> LowerRuntimeBoundBasis
  -> BasisUseReceipt
  -> SelfDescribingBasisEnvelope
  -> BasisLifecycleCertificationBundle
```

### Must Ship

- phase-typed basis capability families for current head, branch, preview,
  snapshot, historical, tenant-scoped, and policy-scoped usage
- normalized basis intent, typed denied capability artifacts, lower-runtime
  readmission, and lifecycle certification closure
- basis eligibility decisions before read, mutation, replay, inspection, or
  materialization surfaces can be constructed
- basis use receipts that distinguish observation, mutation, replay,
  inspection, and materialization permission
- typed denials for stale, inaccessible, incompatible, or operation-ineligible
  basis requests

### Must Preserve

- relational remains the authority for branch, commit, snapshot, and head truth
- Query owns public basis capability admission and use receipts
- raw lower-runtime basis identifiers do not become public capability tokens
- temporal/resource basis extensions in Milestone 9.4 and the follow-on reuse
  hardening in Milestone 9.5 extend the same lifecycle rather than inventing
  parallel basis APIs

### Acceptance Evidence

This milestone is complete only when Query can prove equivalent basis intents
normalize to the same capability envelope, illegal transitions are
unrepresentable or typed failures, and downstream domains can read, inspect,
and prepare mutation surfaces without raw relational branch or snapshot IDs.

## Milestone 9.3.3: Authority-Scoped Effect Execution Pipeline

### Goal

Make Query-authored effects execute through one authority-scoped pipeline that
lowers semantic intent once and requires the executor to consume only lowered,
proof-bearing plans.

### Specification

The governing milestone spec is
[milestone-9.3.3.md](./milestone-9.3.3.md). The closeout record is
[milestone-9.3.3-closeout.md](./milestone-9.3.3-closeout.md).

### Adversarial Constraint

No executor may re-decide authority scope, basis posture, invariant scope,
preview policy, route strategy, artifact policy, or diagnostic richness. If
those decisions are needed, they must be proven before execution and carried in
the lowered effect execution plan.

### Phase Contract

```text
RawEffectIntent
  -> AuthorityEligibility
  -> AuthorityScopedEffectPlan
  -> LoweredEffectExecutionPlan
  -> EffectExecutionReceipt
  -> SelfDescribingEffectEnvelope
```

### Must Ship

- authority-scoped effect plans for current-head, branch, preview, workflow,
  topology, and query-authored writeback families where admitted
- lowered execution plans that carry basis, authority, invariant, artifact, and
  diagnostic policy proofs
- effect execution receipts that expose primary result, decision trace,
  structural deltas, integrity markers, and performance counters
- compile-fail or facade-boundary tests proving executors cannot accept raw
  intents or weaker plan types

### Must Preserve

- domain handlers produce declarative effects rather than framework ceremony
- runtime bridge and relational remain authorities for their execution
  semantics
- Query owns public effect admission, lowering, and receipt shaping
- branch mutation and preview mutation are parameters of the shared lifecycle,
  not separate `execute_on_branch`-style APIs

### Acceptance Evidence

This milestone is complete only when admitted effect families execute through
the same proof-widening pipeline, rejected families fail before construction or
lowering, and execution counters prove strategy decisions were resolved
upstream instead of rediscovered in the executor.

## Milestone 9.3.4: Declared Projection Consumption And Materialized Fact Receipts

### Goal

Make consumption of materialized Query projections a declared, typed,
receipt-backed contract so consumers can use projection facts without reopening
the source authority.

### Specification

The governing milestone spec is
[milestone-9.3.4.md](./milestone-9.3.4.md).

### Adversarial Constraint

A consumer that has received a materialized projection must not fish in
relational truth, bridge internals, signal internals, or domain-specific caches
to discover IDs, memberships, labels, topology facts, workflow facts, table
facts, or geometry facts that should have been declared as consumed projection
facts.

### Phase Contract

```text
ProjectionConsumptionDeclaration
  -> ProjectionConsumptionEligibility | DeniedProjectionConsumption
  -> MaterializedProjectionContract
  -> ConsumedProjectionFactSet
  -> ProjectionConsumptionReceipt
  -> SelfDescribingProjectionConsumptionEnvelope
  -> ProjectionConsumptionCertificationBundle
```

### Must Ship

- projection consumption declarations for materialized query views
- typed consumed fact sets for entity identities, relation identities,
  memberships, labels, derived facts, shape facts, and view-local identities
  where the projection admits them
- materialized projection contracts that bind consumed facts to the query,
  basis, policy, view shape, and materialization digest that produced them
- topology/Worth certification as the first hostile lane, without making the
  milestone topology-specific

### Must Preserve

- relational owns authoritative truth; materialized projections are derived
- Query owns projection contracts, declared consumption, and receipts
- domain consumers own how they use admitted projection facts, not how those
  facts are rediscovered from authority
- future geometry, workflow, table, and design projections reuse the same
  lifecycle instead of adding local lookup helpers

### Acceptance Evidence

This milestone is complete only when a certification program can declare the
projection facts it consumes, receive typed fact receipts bound to one
materialization, and avoid direct source-authority reads for fact discovery.

## Milestone 9.3.5: Intent Admission Decision Lattice And Decision Trace

The governing milestone spec is
[milestone-9.3.5.md](./milestone-9.3.5.md).

Status:

- Closed on 2026-05-18 via
  [milestone-9.3.5-closeout.md](./milestone-9.3.5-closeout.md)

### Goal

Make every Query-crossing intent resolve through a structured admission
decision lattice before construction, command lowering, execution, or
diagnostic materialization, and make covered admitted paths cross into real
bridge-backed execution through one canonical typed handoff.

### Adversarial Constraint

No Query surface may collapse admission into a binary `Result` wall that loses
actionable context. Success, advisory, and violation outcomes must all carry
structured decision traces, machine-readable context, and enough phase proof
for downstream code to proceed, adapt, or fail closed without reconstructing
the decision.

### Phase Contract

```text
RawIntent
  -> IntentEligibility
  -> AdmissionDecision
  -> AdmittedIntentPlan | AdvisoryDecision | ViolationDecision
  -> AdmittedExecutionHandoff | AdvisoryStop | ViolationStop
  -> DecisionTraceEnvelope
```

### Must Ship

- a shared admission decision lattice for reads, basis use, projection
  consumption, effect execution, inspection, diagnostic materialization, and
  lower-runtime capability routing
- structured success, advisory, and violation decision variants with typed
  context
- typed execution handoffs for every covered family whose admitted form already
  binds to a real bridge/runtime execution seam
- decision traces that record policy, capability, invariant, basis, projection,
  and lower-runtime routing decisions where applicable
- compile-fail or construction-boundary tests proving rejected or advisory-only
  intents cannot be lowered as admitted plans

### Must Preserve

- eligibility precedes expensive construction and domain object assembly
- diagnostics can enrich admission traces without changing the operational
  result
- each authority keeps ownership of its decision evidence while Query owns the
  public admission envelope
- binary convenience results may exist only as derived summaries, not as the
  canonical admission artifact

### Acceptance Evidence

This milestone is complete only when all admitted 9.3.x surfaces share the
decision lattice, failure and advisory cases are as inspectable as successful
cases, lower phases consume proof-bearing admitted plans rather than
revalidating raw intents, and covered execution paths consume typed admitted
handoffs rather than rediscovering admission from raw requests.

## Milestone 9.3.6: Lower-Runtime Capability Routing And Boundary Envelopes

### Goal

Make all Query contact with relational, runtime bridge, signal, and later store
surfaces pass through capability-routed lower-runtime boundary envelopes rather
than scattered direct imports or compatibility shortcuts.

### Specification

The governing milestone spec is
[milestone-9.3.6.md](./milestone-9.3.6.md).

### Adversarial Constraint

If direct bridge, relational, signal, and Query-runtime bridge paths coexist,
they must share one lifecycle abstraction or be marked as explicit
compatibility debt. A domain or certification program must not be able to
choose a lower-runtime path by convenience and silently bypass Query's basis,
admission, projection, effect, or inspection contracts.

### Phase Contract

```text
LowerRuntimeCapabilityRequest
  -> CapabilityEligibility
  -> LowerRuntimeRoutePlan
  -> BoundaryExecutionReceipt
  -> LowerRuntimeBoundaryEnvelope
```

### Must Ship

- lower-runtime capability routing for admitted relational, bridge, signal, and
  store-adjacent contacts
- boundary envelopes that name authority, route, capability, cost posture,
  failure topology, and retained evidence
- compatibility-debt records for any remaining direct lower-runtime paths,
  including exit criteria and certification coverage
- facade and compile-boundary tests proving ordinary consumers use Query's
  routed capability lane rather than lower-runtime internals

### Must Preserve

- lower runtimes remain autonomous subsystems with contractual facades
- Query routes capabilities; it does not absorb lower-runtime truth,
  scheduling, storage, or bridge protocol authority
- shared lifecycle is the abstraction; traversal, data topology, failure mode,
  and cost posture remain explicit parameters
- cost and failure boundaries are not flattened by a generic adapter bag

### Acceptance Evidence

This milestone is complete only when every 9.3.x public capability can name its
lower-runtime route, receipt, and boundary envelope, and any remaining direct
path is intentionally tracked compatibility debt rather than an accidental
escape hatch.

## Milestone 9.3.7: Domain Capability Contributions And Canonical Runtime Materialization

### Goal

Allow downstream domains to contribute typed semantic capability posture to
Query through one public contribution seam, while Query remains the sole owner
of canonical runtime artifacts across admission, support, traceability,
workflow, continuity, aftermath, and explanation surfaces.

### Specification

The governing milestone spec is
[milestone-9.3.7.md](./milestone-9.3.7.md).

### Adversarial Constraint

A serious downstream domain must be able to tell Query "this declaration is
advisory", "this declaration is violating", "this declaration carries
declaration-scoped support or traceability", "this declaration has workflow
promotion or discard posture", "this declaration preserves or splits
continuity", "this declaration establishes aftermath facts", or "this
declaration requires explanation context" without minting local pseudo-Query
artifacts, without calling crate-private constructors, and without flattening
semantic posture into generic strings or ad hoc JSON.

### Phase Contract

```text
DomainCapabilityContributionRequest
  -> DomainCapabilityContributionEligibility
  -> AdmittedDomainCapabilityContribution
  -> CanonicalRuntimeMaterialization
  -> QueryAdmissionDecision
   | QuerySupportTraceability
   | QueryWorkflowArtifacts
   | QueryContinuityArtifacts
   | QueryAftermathArtifacts
   | QueryExplanationArtifacts
```

### Must Ship

- one public Query-owned domain capability contribution lifecycle
- typed domain contribution families for:
  - admission posture
  - declaration-scoped support and traceability posture
  - invariant and capability posture
  - workflow / preview posture
  - continuity / lineage posture
  - consequence / aftermath posture
  - explanation / inspection posture
- canonical materializers that turn admitted domain contributions into:
  - `WORTHQueryIntentAdvisoryDecision`
  - `WORTHQueryIntentViolationDecision`
  - declaration-scoped support/traceability artifacts
  - graph-composition capability / invariant-facing artifacts where applicable
- fully closed workflow, continuity, aftermath, and explanation contribution
  families on top of the shared contribution substrate
- compile-boundary and certification proof that canonical Query decision
  artifacts remain Query-owned and cannot be minted directly by domains

### Must Preserve

- Query remains owner of canonical runtime admission artifacts
- downstream domains own semantic meaning, not canonical artifact authority
- public constructors for canonical advisory and violation artifacts do not
  become a free-for-all bypass
- declaration-scoped support and invariant posture stays typed and
  machine-checkable rather than collapsing into free-form messages

### Acceptance Evidence

This milestone is complete only when semantically equivalent domain-authored
advisory or violation posture materializes into the same canonical Query
decision/support artifacts regardless of builder path, while semantically
different posture diverges predictably and illegal direct artifact minting
fails closed.

## Milestone 9.3.8: Query-As-Beginning Platform Entry

### Specification

The governing milestone spec is
[milestone-9.3.8.md](./milestone-9.3.8.md). The closeout record is
[milestone-9.3.8-closeout.md](./milestone-9.3.8-closeout.md).

### Goal

Make `worth-query` the true first-class platform entry for serious downstream
domain work so declarations, progression, authority routing, preparation,
continuation, inspection, ergonomics, and certification all begin inside one
Query-owned public seam rather than being split across local pseudo-Query
layers above relational, the runtime bridge, signal, `worth-proof`, and
`worth-foundational`.

### Adversarial Constraint

A geometry-kernel-grade domain must be able to enter WORTH through Query once
and stay inside one honest public lifecycle while expressing declaration
meaning, support posture, continuity posture, preparation readiness, runtime
continuation, and inspection needs without reconstructing a second semantic
world in host glue or domain-local adapters.

### Must Ship

- one Query-owned platform-entry seam that begins at domain entry and extends
  through the full boundary stack captured in
  [milestone-9.3.8.md](./milestone-9.3.8.md)
- a phase-locked implementation that follows the milestone's explicit
  boundary order rather than re-splitting the work into disconnected
  declaration, preparation, and runtime-handoff mini-products
- public route-plan, boundary-receipt, and boundary-envelope artifacts for the
  covered lower-authority crossings
- a framework-quality ordinary lane that compiles onto lower authorities
  without caller-owned choreography
- unified inspection, support/readiness, documentation, and certification for
  the resulting platform-entry lifecycle

### Must Preserve

- Query remains the front door and orchestration surface rather than a second
  truth, continuation, or derived-execution engine
- relational, the runtime bridge, signal, `worth-proof`, and
  `worth-foundational` remain the authorities for the semantics they already
  own
- each internal phase remains a real capability boundary rather than a vague
  implementation bucket
- the milestone may be large, but it must still close as one coherent product
  capability instead of shipping fragmented half-seams

### Acceptance Evidence

This milestone is complete only when serious downstream domains can enter
Query once, progress through the covered boundary phases without rebuilding
local pseudo-Query layers, and receive public route/receipt/envelope,
inspection, and certification artifacts that converge across equivalent paths
while divergent posture and illegal shortcuts fail typed and early.

The late collaboration extension inside `9.3.8` also depends on shared
lower-authority hardening work in `worth-signal`, `worth-relational`, and
`worth-runtime-bridge` so Query can consume retained branch, merge, lineage,
preview, policy, and strategy posture instead of reconstructing collaboration
meaning from host-local glue. The first of those shared hardening specs is
[`../worth_signal/collaboration_branching_hardening_plan.md`](../worth_signal/collaboration_branching_hardening_plan.md).

## Runtime API Public Stabilization Gate

### Goal

Freeze the ordinary public runtime API contract after the runtime facade has
consumed Milestones 9.1 through 9.3.8, so downstream domain runtimes can build
against named surfaces and typed handles now while Milestones 9.4 and 9.5
later extend the same API model with temporal/async semantics and the related
productization cleanup.

### Adversarial Constraint

A serious domain runtime must be able to build workflow, geometry, table, or
application features against the public Query facade now, without lower-runtime
plumbing, and without any later temporal/async milestone forcing a parallel API
or sync-to-async rewrite.

### Specification

The governing stabilization spec is
[runtime-api-public-stabilization-plan.md](./runtime-api-public-stabilization-plan.md).

### Must Ship

- golden DX transcript tests for workflow, geometry, table, and composed
  adversarial runtime surfaces
- final public vocabulary for workspace, durable surfaces, handles, state,
  aspects, computeds, effects, intents, branch/preview reuse, and inspection
- support matrix rows distinguishing stable runtime-backed surfaces from
  deferred temporal/async/store/durable surfaces
- compile-fail rejection of lower-runtime plumbing and temporal/async pre-claim
  shortcuts from ordinary public API usage

### Must Preserve

- no domain semantics move into `worth-query`
- temporal/async behavior remains deferred until Milestones 9.4 and 9.5
- lower runtimes remain authorities for truth, signal execution, bridge
  protocol, temporal scheduling, async lifecycle, store parity, and durability
- later temporal/async milestones extend the stabilized handle/state/aspect
  contract rather than adding sibling public APIs

### Store Dependency

This gate is not blocked on `worth-store`. It must explicitly mark store-backed
and durable claims as later milestone debt.

### Acceptance Evidence

This gate is complete only when `worth-query` can prove that the final public
facade supports the golden DX transcripts through meaningful assertions over
receipts, lanes, aspects, delivery, residue, support posture, and inspection,
while unsupported temporal/async neighbors fail typed and early.

## Runtime Authoritative Mutation Evidence Gate

### Goal

Freeze the ordinary public mutation-evidence contract after runtime facade
stabilization so downstream write-heavy domains can build on aspect-native
authority surfaces without rebuilding target recovery, existing-truth identity
binding, naming-writeback evidence, or continuity-sensitive inspection locally.

### Adversarial Constraint

Direct writes, ordered batches, authoritative imports, preview-local mutation,
projected naming attachment, continuity-sensitive updates, and domain-authored
writeback lowering must preserve the same canonical target-class meaning, the
same target identity evidence, and the same typed explanation of what was
actually targeted regardless of whether the target was new, preexisting,
referenced earlier in the same batch, or bound through admitted naming or
continuity evidence.

### Specification

The governing hardening spec is
[runtime-authoritative-mutation-evidence-plan.md](./runtime-authoritative-mutation-evidence-plan.md).

The required follow-on hardening spec for generic graph-shaped authoring and
identity-preserving existing-target mutation is
[runtime-generic-graph-authoring-plan.md](./runtime-generic-graph-authoring-plan.md).

The admitted graph-authoring and geometry-pressure closure for that follow-on
hardening is now frozen in
[runtime-generic-graph-authoring-closeout.md](./runtime-generic-graph-authoring-closeout.md).

### Must Ship

- explicit declared-versus-resolved target evidence in public receipts and
  inspection
- honest aggregate batch/session authority evidence for multi-write mutation
  sessions
- first-class runtime-carried causality and provenance so lineage and source
  explanation come through the public authority lane by construction
- admitted existing-truth identity binding with typed denial for unresolved or
  incompatible bindings
- admitted naming-aware and continuity-aware mutation evidence families that
  either preserve explicit outcome meaning or fail typed and early
- domain-agnostic public authoring surfaces for:
  - identity-preserving existing-target relation updates
  - first-class same-batch graph composition with a widening mixed-shape
    capability contract
  - bridge-backed backend-verified existing-truth checks on admitted runtime
    families
- support-matrix and certification rows covering the hardened mutation surface

### Must Preserve

- aspect-native CRUD remains the ordinary public mutation story
- touched-aspect fallout meaning stays explicit and auditable alongside target
  evidence
- lower runtimes remain authoritative for truth, naming, writeback, and
  lineage semantics
- unsupported identity-binding, naming, or continuity families fail closed
  rather than degrading into best-effort target recovery

### Store Dependency

This gate is not blocked on `worth-store`. Durable restart, store-backed replay,
and persisted mutation artifact reload remain later-milestone debt.

### Acceptance Evidence

This gate is complete only when `worth-query` can prove that public receipts,
inspection bundles, support metadata, and executable admission behavior agree
on target evidence, existing-truth binding, and admitted naming/continuity
neighbors, while downstream domains can delete local target-recovery glue
instead of merely wrapping it. The gate is not fully closed until the authoring
surfaces and certification obligations in
[runtime-generic-graph-authoring-plan.md](./runtime-generic-graph-authoring-plan.md)
are also satisfied. That follow-on closure now exists in
[runtime-generic-graph-authoring-closeout.md](./runtime-generic-graph-authoring-closeout.md).

## Milestone 9.4: Runtime-Backed Temporal And Async Query Surface

### Goal

Close the full runtime-backed temporal and async Query product surface so
application code can declare, admit, inspect, execute, and consume temporal
wakes, async/resource lifecycle, and mixed truth/time/async delivery through
one canonical Query facade before store-backed and durable follow-on work
begins.

### Adversarial Constraint

For the same canonical Query declaration, truth-view basis, temporal posture,
tenant/policy context, preview posture, and async source family, Query must
produce the same admitted Query basis, the same result-state meaning, the same
mixed-cause delivery ordering, and the same explanation artifacts regardless
of whether the observed change came from a relational truth patch, a time-only
wake, an async completion, a retry or revalidation path, replay, restart or
resume, or preview promotion or discard.

### Why This Milestone Exists

Bridge Milestone 17 now closes the lower-authority temporal and async law:
temporal bridge basis, time-aware subscription admission, async source
declaration identity, completion causality, mixed-cause ordering,
restart/resume posture, preview residue law, and offline certification
bundles. Query now has to project that into one ordinary product surface
instead of splitting temporal, async, mixed-cause, and certification into
separate roadmap milestones.

Without this milestone:

- downstream domains would have to reach around Query for temporal or async
  behavior
- app surfaces would invent local pending, fulfilled, stale, cancelled, or
  superseded meaning that drifts from bridge-closed causality law
- time-only changes would be treated as diagnostics noise or fake truth patches
- `worth-server` and later consumers would inherit half-Query, half-host
  delivery semantics instead of one typed contract

### Must Ship

- temporal query basis descriptors and time-aware subscription lowering through
  the stabilized Query runtime facade
- async/resource query families with typed result-state and completion-cause
  semantics
- mixed truth/time/async delivery ordering, coalescing, and replay-equivalent
  query-shaped delivery metadata
- restart-aware runtime-backed temporal and async continuation posture where
  admitted
- typed support metadata, diagnostics, and hostile certification for the full
  merged temporal/async surface

### Must Preserve

- Query owns public product meaning and result shape, not clock truth, wake
  scheduling, async lifecycle truth, retry policy, or mixed-cause authority
- `worth-runtime-bridge` remains authoritative for temporal basis, async
  identity, completion causality, mixed-cause ordering, restart/resume basis,
  preview residue law, and certification bundle shape
- `worth-signal` remains authoritative for temporal eligibility,
  previous-value semantics, wake readiness, async lifecycle, retry, timeout,
  cancellation, supersession, and revalidation policy
- historical truth basis remains distinct from temporal execution basis at
  every public Query boundary

### Sequencing Notes

This milestone absorbs the old roadmap `9.4`, `9.5`, `9.6`, and `9.7` split
into one Query milestone. The detailed internal execution plan now lives in
[milestone-9.4.md](./milestone-9.4.md).

It belongs after the runtime API stabilization and authoritative mutation
evidence gates because Query must extend one already-stabilized ordinary
runtime facade.

It belongs before Store Milestone 1 because store-backed integration should
not be forced to discover temporal, async, mixed-cause, and certification
semantics while also closing durable backend parity.

### Store Dependency

- Runtime-backed temporal basis, async families, mixed-cause delivery, and
  certification are not blocked on `worth-store`.
- Store-backed historical restore, persisted temporal replay, and durable
  async continuation remain later scope for Milestones `10` and `11`.

### Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- the merged temporal/async certification suites in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/worth-query/test-requirements.md)
  pass with canonical machine-checkable artifacts
- temporal query basis, time-only delivery, async/resource result-state, and
  mixed truth/time/async ordering all remain Query-shaped and replay-equivalent
- unsupported ambient clocks, unbound timers, stale async completions, raw
  timer folklore, and unsupported mixed-cause neighbors fail typed and early
- the milestone-close proof matches the merged [milestone-9.4.md](./milestone-9.4.md)
  closure instead of leaving separate old `9.5` through `9.7` gaps behind

## Milestone 9.5: Query Productization Debt Cleanup For Reuse, View Shapes, And Typed Consumption

### Goal

Close the remaining runtime-backed Query productization debt in reusable
composition, core view-shape families, grouped composition, retained-artifact
projection consumption, and preserved temporal/async reuse so the ordinary
product facade stops carrying visible-but-not-fully-hardened lanes into the
store-backed and durable milestones.

### Adversarial Constraint

For the same canonical query declaration, scope/template expansion, view-shape
family, retained result artifact, basis/remask posture, and preserved
temporal/async reuse posture, Query must produce the same canonical
declaration identity, the same support/admission posture, the same
fact-consumption contract, and the same delivery/reuse meaning regardless of
whether the caller reaches the lane through direct composition, grouped
composition, inspector/grouped reuse, or retained artifact consumption.

### Why This Milestone Exists

The old roadmap split temporal/async work into `9.4` through `9.7`, but that
semantic work now closes inside one merged [milestone-9.4.md](./milestone-9.4.md).
That leaves a different remaining problem: several ordinary productization
lanes are admitted, documented, and in daily use, but still carry explicit
debt markers or structurally unfinished reuse/productization work.

Without this milestone:

- store-backed and durable milestones will freeze half-hardened composition and
  view-shape semantics
- direct consumers will keep hitting special-case pack/bind/decode seams where
  Query claims a typed fact-consumption lane
- support and profile surfaces will keep advertising core product families as
  admitted-but-debt instead of actually closed

### Must Ship

- hardened named-scope expansion and template-instantiation support profiles
  that can move from `debt` to production-ready runtime-backed closure
- hardened core view-shape families for `table`, `detail`,
  `inspector_detail_observed`, `inspector_detail_focused`, and
  `kanban_grouped`, including their support/profile and ordinary product
  semantics
- grouped template/composition closure so grouped planning is no longer an
  admitted public lane carrying explicit composition debt
- projection-consumption source-family closure for retained derived artifact
  bindings and live artifact bindings where Query intends them to participate
  as first-class typed fact sources
- hardened runtime-backed preserved reuse across the inspector/grouped
  temporal/async neighbors so the covered reuse surface carries merged `9.4`
  meaning end to end
- simple public raw runtime bootstrap for a valid bridge-backed read runtime
  so hostile runtime-backed tests do not need custom assembly just to reach the
  ordinary read lane

### Must Preserve

- scope/template expansion remains canonical declaration composition rather
  than string substitution or host-local query rewriting
- view shape remains part of planning, delivery, and reuse semantics rather
  than display-only sugar
- projection consumption remains the typed fact lane; it must not collapse back
  into row-bag decoding folklore
- temporal/async reuse must preserve the merged `9.4` runtime-backed meaning
  rather than erasing time/async posture during scope/template/view reuse

### Sequencing Notes

The detailed debt-close execution plan for this milestone now lives in
[milestone-9.5.md](./milestone-9.5.md).

This milestone belongs immediately after the merged
[milestone-9.4.md](./milestone-9.4.md) closure so runtime-backed temporal and
async meaning is already frozen before reusable composition, retained
artifacts, and preserved view-shape reuse try to carry it forward.

It belongs before Milestones `10` and `11` because store-backed and durable
milestones should not inherit half-hardened runtime-backed productization
surfaces as if they were final.

### Store Dependency

- This milestone is not blocked on `worth-store`.
- Durable saved-query reload, restart-stable continuation, and store-backed
  temporal/async reuse remain later scope for Milestones `10` and `11`.

### Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- the explicit `debt` markers for named-scope expansion, template
  instantiation, and the admitted core view-shape families are removed
- grouped composition docs and support/profile surfaces no longer describe the
  admitted grouped planning lane as composition debt
- retained derived artifact bindings and live artifact bindings no longer force
  product code through special-case runtime-owned fact seams where Query claims
  a first-class projection-consumption lane
- inspector/grouped temporal/async reuse neighbors carry canonical
  runtime-backed semantics across the covered reuse surface
- hostile runtime-backed read tests no longer need a custom minimal
  bridge-backed harness just to obtain a valid raw runtime read path

## Milestone 9.6: Product Boundary Debt Closure For Evidence Identity, Typed Stop Classes, And Session Label Identity

> **Status:** Closed for non-spatial identity-boundary scope on `query-repair`;
> `worth-spatial public_api_contract` remains a named postponed external gate.
> Non-spatial Phases 8â€“12 are reconciled in
> [milestone-9.6-attack-plan.md](./milestone-9.6-attack-plan.md), and Milestone
> 9.7 is unblocked for worth-query identity-boundary sequencing.

### Goal

Make evidence identity, stop-class matching, and session label identity
runtime-owned structural contracts so consumers never format runtime values
into digests, string-match error messages in decision paths, or mint
free-form session labels against a runtime built on canonical identity.

### Adversarial Constraint

For the same runtime fact â€” admission denial, basis admission, receipt,
support row, session identity â€” Query must produce the same canonical
evidence identity and the same typed stop-class meaning under `Debug` derive
reordering, field renaming, message rewording, separator injection inside
field values, and session label collision pressure.

### Why This Milestone Exists

The first serious downstream consumer hashes `format!("{:?}", value)` strings
joined with `|` for evidence identity, matches runtime denials with
`message.contains(...)`, and opens preview/branch sessions with free-form
string labels. Each is a Query-owned contract carried by consumer folklore.
Concurrency receipts (`9.7`) and the consumer kit (`9.8`) must be born on
canonical identity rather than migrated onto it later.

### Must Ship

- one sealed, scheme-versioned canonical evidence-identity primitive
- migration of covered Query-owned digest surfaces onto that primitive with
  zero format-string residue
- typed stop-class matching across covered denial paths, including typed
  family payloads on admission denials
- canonical session label identity for preview/branch entry with explicit
  collision posture
- support/profile, docs, and hostile certification closure for all three
  boundaries

### Must Preserve

- the existing rich error topology, extended into matchability rather than
  flattened
- the existing public facade shape; no parallel digest, error, or label APIs
- human-readable diagnostics as presentation atop typed contracts

### Complexity / Proof Obligations

- name the canonical encoding and digest contracts and prove digest stability
  under formatting drift and separator injection with exact assertions
- prove typed stop-class coverage with a consumer-shaped zero-string-ops
  matching suite that survives message rewording
- prove session label collision posture with typed collision stops

### Allowed Debt

- durable digest archives and restart-stable identity reload remain explicit
  handoffs to Store Milestones 7 and 11
- no covered surface may keep the format-string digest scheme as debt

### Sequencing Notes

The detailed execution plan lives in [milestone-9.6.md](./milestone-9.6.md).
This milestone belongs immediately after `Milestone 9.5` and before
`Milestone 9.7`, whose receipts, journal identity, and published-artifact
digests must use the canonical scheme from birth.

### Parallelization Notes

Phases are mostly independent per boundary; digest migration must complete
before `9.7` begins emitting new receipt families.

### Store Dependency

- This milestone is not blocked on `worth-store`.

### Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- covered digest surfaces emit scheme-versioned canonical digests with zero
  format-string construction
- a consumer-shaped matching suite handles every covered stop class without
  string operations, surviving hostile message rewording
- preview/branch entry flows through canonical label identity with typed
  collision posture
- docs, support profiles, and certification agree the boundaries are closed

## Milestone 9.7: Concurrent Read Authority And Deterministic Submission

### Goal

Decompose the Query runtime into authority-typed subsystems so
committed-snapshot reads scale concurrently across consumers while truth
mutation and derived maintenance remain single-owner and deterministic,
without changing canonical query meaning or adding a second semantics path
beside the workspace.

### Adversarial Constraint

N concurrent shared read contexts under sustained commit pressure, preview
and branch churn, and live maintenance load must produce byte-identical
results and receipts for the same canonical declaration and basis capability
as fully serialized execution â€” while journal replay reconstructs identical
truth, receipts, and published derived artifacts, with zero locks on the
committed-read hot path and zero derived evaluations triggered by readers.

### Why This Milestone Exists

Every workspace operation takes `&mut self`, so the borrow checker enforces
one operation in flight per workspace regardless of MVCC immutability
underneath. Server-grade consumers would otherwise improvise a global lock or
branch-per-connection â€” both prohibited folklore. Store-backed shapes in
Store Milestone 1 must inherit lane-correct contracts rather than retrofit
`Send` boundaries later.

### Must Ship

- backend adapter contracts decomposed by authority lane with `Send + Sync`
  read lanes (Phases 1â€“2)
- runtime-owned published-artifact registry authority with registry/mint
  inventory and scans (Phase 11)
- generation-indexed pinning with lock-free hot path, pin/retire inventory, and
  runtime-owned residue counters (Phase 12)
- shared read contexts with full pinning-boundary closure in-phase (Phase 13)
- typed journal position identity with journal inventory and scans (Phase 14)
- consumer-facing journal-segment replay with journal-boundary closure (Phase 15)
- the published derived-artifact rule: readers consume digest-stamped
  published results through projection consumption; only the maintenance
  owner evaluates
- the re-expressed workspace facade with unchanged existing consumer surface
  and fail-closed admission rows for the new families (Phase 9)
- real concurrent hostile certification with in-phase sabotage proof (Phase 16)
- public-bridge projection-consumption honesty (Phase 17)
- derived milestone closure posture and closeout doc (Phase 18)

### Must Preserve

- canonical query meaning across serialized and concurrent execution
- lower-crate authority boundaries; only access topology moves
- the single-owner workspace as a first-class consumer surface
- merged `9.4` temporal/async meaning and `9.5` projection-consumption
  semantics inside the published-artifact lane

### Complexity / Proof Obligations

- name the read-context mint, submission intake, and publication contracts
- expose exact counters for lock acquisitions (zero), reader evaluations
  (zero), snapshot generation pins/retirements, journal positions, and
  publication breadth
- prove byte-identical concurrent-versus-serialized receipts and journal
  replay parity

### Allowed Debt

- durable journal persistence, store-backed replay reconstruction, and
  restart-stable published-artifact reload remain explicit handoffs to Store
  Milestones 3, 8, and 11
- lock-based or evaluation-leaking read paths may not ship as debt

### Sequencing Notes

The detailed execution plan lives in [milestone-9.7.md](./milestone-9.7.md).
This milestone belongs after `Milestone 9.6` so its receipts and digests are
born canonical, and before Store Milestone 1 as a hard gate so store-backed
shapes inherit the concurrency topology.

### Parallelization Notes

Phases 1â€“10 may overlap at the topology layer (adapter decomposition,
read-context scaffold, submission seam, facade families, interim hostile
schedule). Phases 11â€“18 are the mandatory honesty end-cap: each phase owns its
substrate and proof together â€” inventory slices, scans, hostile schedules, and
sabotage close inside the phase that ships the work. Sequence:
**11 â†’ 12 â†’ 13** (pinning closes in Phase 13), **14 â†’ 15** (journal closes in
Phase 15), **16 â†’ 17** (certification with in-phase sabotage, public-bridge
honesty), **18** (aggregated closeout only). Milestone `9.7` may not report
`Closed` until Phase 18.

### Store Dependency

- This milestone is not blocked on `worth-store`.

### Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- concurrent readers under write pressure produce byte-identical receipts and
  results to serialized execution with exact-zero lock and reader-evaluation
  counters
- journal replay reconstructs identical truth, receipts, and published
  artifacts
- existing downstream consumers compile unchanged against the re-expressed
  facade
- the new facade families carry honest fail-closed support/admission rows

## Milestone 9.8: Downstream Consumer Product Kit For Evidence Reports, Boundary Audits, And Support Pinning

### Goal

Ship the runtime-owned kit that eliminates consumer-side folklore around
Query's product contracts â€” declarative evidence-report scaffolding, a
shipped boundary-bypass audit, and exportable, pinnable support snapshots â€”
proven by reference-consumer adoption rather than API presence.

### Adversarial Constraint

A downstream domain crate must be able to author a digest-bearing evidence
report, enforce the no-bypass contract, and pin its support-posture
dependencies using only Query-shipped kit surfaces, with every divergence
class â€” escaped digest fields, prohibited seam usage, pinned posture
regression, folklore resurrection â€” failing mechanically in the consumer's
build.

### Why This Milestone Exists

The reference consumer pays roughly 250 hand-rolled lines per evidence
report, enforces the hard prohibitions with `include_str!` source greps, and
re-derives support posture as hand-built gap rows. Each is a runtime-owned
contract materialized by consumer folklore, and every future consumer would
reinvent or skip it. Milestones `9.6` and `9.7` harden what Query says; this
milestone hardens what Query gives consumers to build with.

### Must Ship

- the declarative evidence-report composition kit over the `9.6` canonical
  evidence-identity primitive
- runtime-owned bypass enforcement: sealed or visibility-tightened seams plus
  one shipped audit artifact derived from a single prohibition registry
- the serialized, versioned support snapshot and typed consumer pinning
  contract with build-failing drift detection
- the shipped in-memory consumer test backend with honest fail-closed support
  posture, replacing hand-implemented adapter assemblies and hand-fabricated
  receipts in consumer test suites
- reference-consumer adoption with deletion of `worth-kernel`'s hand-rolled
  report plumbing, grep audit, and gap-row assembly in covered surfaces
- support/profile, docs, and hostile certification closure for the kit
  families

### Must Preserve

- the `9.6` canonical evidence-identity scheme as the only digest authority
  the kit can express
- one support truth: the snapshot is a digest-bound derived projection of the
  live matrix
- the reference consumer's evidence semantics through migration
- the Query facade as the only consumer surface

### Complexity / Proof Obligations

- name the kit report, audit, and pinning contracts
- prove kit-versus-hand-rolled report parity, structural (non-textual) bypass
  detection with zero false positives on comments and literals, and
  pin-localized build failure under posture regression
- prove adoption residue with exact-zero assertions on covered consumer
  surfaces

### Allowed Debt

- persisted support snapshots, durable audit archives, and store-backed kit
  artifacts remain explicit handoffs to Store Milestones 11 and 18
- shipping the kit without reference-consumer adoption may not be claimed as
  closure

### Sequencing Notes

The detailed execution plan lives in [milestone-9.8.md](./milestone-9.8.md).
This milestone belongs after `Milestone 9.7` so the kit covers the
concurrency-era facade families, and before Store Milestone 1 so real
consumer adoption pressure-tests the frozen runtime-backed surface.

### Parallelization Notes

Kit phases may overlap early Store Milestone 1 work where staffing allows, since
store execution does not consume kit surfaces; reference adoption and
certification close strictly last.

### Store Dependency

- This milestone is not blocked on `worth-store`.

### Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- a kit-authored report reproduces a hand-rolled report's semantics with
  canonical-scheme digests, and misuse fails typed or fails to compile
- the shipped audit detects seeded bypasses structurally from a downstream
  crate's test suite with zero textual false positives
- a pinned posture regression fails exactly the pinned consumers' builds with
  typed findings
- a downstream-shaped test suite obtains a valid workspace from the shipped
  in-memory backend with zero hand-implemented adapter traits and zero
  hand-fabricated receipts
- covered `worth-kernel` surfaces carry zero remaining hand-rolled digest,
  audit, or gap-row folklore

## Milestone 9.9: Graph Touch Obligation Authority

### Goal

Establish graph touch obligation dispatch as a complete Query authority
boundary â€” typed obligation kinds, three-state verdicts, canonical dispatch
artifacts, index-backed selection, relational execution bridge, duplicate-rule
elimination, and mechanical consumer anti-folklore â€” certified
architecturally and proven by reference-consumer deletion of parallel
legality in `worth-topo` and `worth-kernel`.

### Adversarial Constraint

Obligation dispatch must be a pure function of touch descriptor, operating
world, and assembly index â€” on every lane that reaches authoritative
execution: write-batch intent admission, declaration-entry orchestration,
read-family execution, and preview/branch mutation where applicable â€” with
exact-zero false negatives, false positives, duplicate rule implementations,
manual pre-check residue, and dispatch-plan drift on covered surfaces under
property-test certification.

### Why This Milestone Exists

Query teaches register invariants and consume typed graph-composition
denials, but the product surface still trains manual invariant-pack callbacks
and duplicate enforcement across pre-checks, materialized-view validators,
and commit-boundary invariants. `worth-topo` calls `compose_graph` yet
enforces loop wiring in three places; `worth-kernel` carries host-local
layout legality and motion sequencing guarded by `unreachable!`. This
milestone ships obligation authority as foundation infrastructure before
store-backed execution inherits another legality layer.

### Must Ship

- complete obligation authority model with multi-obligation dispatch envelopes
- graph touch descriptors for mutation and read lanes
- registration, auto-indexing, assembly index with complexity contracts
- relational graph-composition execution point and rule migration
- authoritative write-batch intent admission integration (canonical dispatch seam)
- declaration-entry and contribution-orchestration dispatch
- read-family and read-composition dispatch (all four entry points)
- preview and branch mutation obligation parity
- policy-aware graph mutation execution and operating-context gate dispatch
- advisory, capability-gap, and preflight-sequencing executors
- envelope attachment to receipts, decision traces, and mutation evidence
- derived read validation re-homed; consumer obligation bypass audit
- kernel construction operating context wiring
- primitive construction birth compose execution (all covered families)
- full reference adoption in `worth-topo` and `worth-kernel` construction
- public docs and AI_README category
- architectural hostile certification matrix closure

### Must Preserve

- relational invariant execution authority
- typed graph-composition domain-invariant denial contracts on block paths
- declaration legality and support admission as upstream lanes obligations consume

### Complexity / Proof Obligations

- every obligation kind Ã— representative touch in certification matrix
- false-fire/false-miss, replay equivalence, complexity contracts
- exact-zero duplicate rule implementations and adoption manifest residue
- policy-aware mutation gate parity with operating context changes

### Allowed Debt

- store-backed obligation envelope durability remains Milestone `10`/`11` scope
- shipping authority surfaces without full reference adoption may not be claimed
  as closure

### Sequencing Notes

The detailed execution plan lives in [milestone-9.9.md](./milestone-9.9.md).
Twenty phases: vocabulary and relational execution point (1â€“5); intent
admission integration before surface-specific wiring (6â€“9); remaining
executors and envelope attachment (10â€“12); re-homing and bypass audit (13â€“14);
kernel operating context before birth compose (15â€“16); adoption (17â€“18);
docs then certification close (19â€“20).

### Parallelization Notes

Relational execution point and policy-aware mutation work may overlap where
staffing allows; adoption and certification close strictly last.

### Store Dependency

- Runtime-backed obligation authority is not blocked on `worth-store`.
- Durable obligation envelope persistence is Milestone `10`/`11` scope.

### Acceptance Evidence

- every obligation kind executes in certification matrix across representative
  touches and lanes
- write-batch intent admission carries obligation dispatch â€” manual
  invariant-pack pre-hook eliminated on covered paths
- primitive construction birth executes compose_graph with obligation routing
- compose, batch, read-family, preview/branch, and declaration-entry lanes
  dispatch with canonical envelopes on receipts and decision traces
- policy-aware mutation gates and preflight sequencing certified
- full topo milestone-one catalog and kernel phase-chain adoption residue
- bypass audit and architectural certification matrix pass

## Milestone 9.10: Graph Read Access Planning And Declarative Index Admission

### Goal

Make graph read access planning a Query authority boundary: declared graph
reads derive canonical access shapes, required indexes, budget estimates,
admission postures, and receipts before execution. Covered graph read lanes must
not perform hidden N+1 traversal, must not allocate unbounded background
indexes, and must not teach AI agents a graph/index mental model that the
runtime cannot enforce.

### Adversarial Constraint

Broad boolean predicates, multi-relation edge walks, dense frontier expansion,
policy/tenant narrowed reads, relationship-proof reads, reusable read families,
live-promoted reads, and preview/branch reads must all choose one explicit
posture before expensive execution begins: admitted inline indexed, admitted
paged streaming, persistent-index-required, store-backed-index-required,
async-materialization-required, access-capability-registration-required, or
typed denial. No covered surface may discover after the fact that it performed
N+1 relation lookups or built a RAM-expansive derived graph structure.

### Why This Milestone Exists

The roadmap already points developers away from caller-owned graph loops, and
Milestone `9.9` gives Query touch/read descriptors and obligation envelopes.
That is not enough to make graph reads robust in the wild. Query also needs a
performance authority boundary that derives access needs from declarations,
admits only bounded execution postures, and records the work through
machine-checkable counters before store-backed execution tries to push the same
semantics into durable indexes.

### Must Ship

- graph read access shape vocabulary derived from canonical read graphs
- boolean predicate normalization and selectivity inputs
- required index set derivation for directional adjacency, predicate, ordering,
  lifecycle, policy/tenant, and relationship-proof support
- access cost estimates, memory/breadth budgets, and complexity contracts
- typed access admission outcomes including budget denial, persistent-index
  requirement, store-backed capability requirement, paged streaming, async
  materialization, access-capability registration requirement, and typed denial
- exhaustive graph read access-case registry covering admitted,
  required-capability, materialized, streaming, store-owned, and typed-denial
  cases
- existing index inventory and support rows
- bounded ephemeral index provisioning with lifecycle receipts
- ordinary read, reusable read-family, intent, live, preview/branch,
  policy/tenant, and relationship-proof integration
- graph-read consumer bypass audit for N+1 and manual access folklore
- reference adoption in `worth-topo` and `worth-kernel`
- docs and AI_README category for graph read access planning
- hostile certification matrix for no-N+1, memory budgets, and replayable
  access-plan receipts

### Must Preserve

- `worth-relational` remains truth authority; Query owns read declaration,
  access planning, admission, receipts, and support posture
- indexes and materialized access structures remain derived and rebuildable
  from authoritative truth
- graph touch obligation authority remains separate from graph read access
  planning
- store-backed persistent index durability is represented as a typed
  `RequiresStoreBackedPersistentIndex` capability posture with Milestone
  `10`/`11` ownership

### Complexity / Proof Obligations

- exact counters for candidate roots, touched nodes, touched edges, frontier
  width, visited/dedup size, resident bytes, allocated bytes, and page count
- proof that over-budget broad reads reject, stream, require persistent index,
  or require async materialization before edge-scan counters increment
- exact-zero covered caller-owned N+1 graph read loops
- access-plan digest replay equivalence across repeated execution on the same
  truth basis

### Allowed Debt

- no runtime-backed graph read edge case may remain generic debt; every case
  needs an access-case registry row, admission posture, support-row posture,
  receipt field, and test requirement
- durable persistent index storage and restart-stable access artifacts are
  explicit Milestone `10`/`11` capability requirements surfaced through typed
  store-owned postures
- a graph shape without current execution support must return a named
  required-capability posture or typed denial with owner and evidence
- hidden N+1 loops, hidden broad scans, and unbounded automatic index builds may
  not ship as debt

### Sequencing Notes

The detailed execution plan lives in [milestone-9.10.md](./milestone-9.10.md).
This milestone belongs after `9.9` because it consumes the read/touch descriptor
mental model, and before `10` because store-backed execution should inherit a
complete access planning contract rather than inventing one during pushdown.

### Parallelization Notes

Planner vocabulary, cost modeling, and support-row work may overlap. Execution
postures and reference adoption must follow admission surfaces. Certification
closes last.

### Store Dependency

- Runtime-backed access planning, bounded ephemeral indexes, streaming posture,
  and async-materialization admission are not blocked on `worth-store`.
- Durable persistent index storage, restart-stable access artifacts, and
  store-backed pushdown parity are Milestone `10`/`11` capability owners and
  must be visible as typed store-owned postures before execution.

### Acceptance Evidence

- every covered graph read produces an access-plan receipt or typed denial
- broad boolean and dense traversal reads refuse unsafe inline execution before
  expensive work begins
- hidden N+1 traversal is mechanically detected and absent from covered
  reference-consumer surfaces
- docs, AI_README, support rows, and certification agree on admitted,
  required-capability, materialization, store-owned, and typed-denial graph read
  access postures

## Milestone 9.11: Declarative Downstream Basis Authority And Consumer DX

Status:
Closed on 2026-07-12. See
[milestone-9.11-closeout.md](./milestone-9.11-closeout.md) for the shipped
artifact, deletion, consumer-adoption, and verification record.

### Goal

Make Query mint one canonical downstream authority artifact that binds the
exact scoped basis, projection contract, consumption receipt, source lineage,
settlement posture, and admitted typed facts. Consumers declare required
meaning through one fluent facade path instead of reconstructing Query
authority from separately pairable lifecycle artifacts.

### Adversarial Constraint

Two evaluations may expose equal labels, digests, target values, or projection
shapes while differing in basis generation, source lineage, contract,
settlement, or consumption receipt. No downstream crate may construct or admit
a hybrid of those evaluations, promote evidence into authority, or pay work
proportional to unrelated Query or consumer state.

### Why This Milestone Exists

Basis capability lifecycle, declared projection consumption, and Consumer Kit
proof already exist, but demanding consumers still have to pair their outputs.
Worth UI demonstrated that individually typed artifacts can still become an
unsafe aggregate when their exact shared provenance is not owned by one Query
product. Query needs the same declarative, authority-first product standard for
downstream basis handoff that `9.10` establishes for graph access admission.

### Must Ship

- sealed Query-owned consumed-projection authority artifact
- structural basis, contract, receipt, source, settlement, and fact binding
- closed declarative authority contract and typed denial taxonomy
- result-attached fluent facade DX and equivalent explicit phase API
- converged public basis/downstream-authority facade vocabulary
- support rows, inspection projections, exact operation counters, and docs
- Consumer Kit prohibition, compile-fail, adoption, and residue proof
- Worth UI Query-binding adoption and deletion of local basis reconstruction
- hostile cross-basis, collision, stale-settlement, replay, and bounded-work
  certification

### Must Preserve

- Query remains Query basis and projection-consumption authority
- lower runtimes and downstream domains retain their own truth and execution
  authority
- evidence labels and digests remain derived, inspectable, and non-promotable
- unsupported source and fact families fail closed through typed support posture
- convenience and explicit lifecycle paths share one semantic transition

### Complexity / Proof Obligations

- admission is `O(declared_requirements + consumed_facts)`
- exact counters cover normalization, requirement checks, source binding,
  settlement checks, fact visits, canonicalization, and authority construction
- unrelated workspace rows, historical basis inventory, and consumer graph size
  do not affect Query authority-admission counters
- replayed equivalent consumption produces equivalent structural authority and
  evidence

### Allowed Debt

- no authority-capable tuple, digest promotion, raw identity re-entry, local
  compatibility scan, or legacy constructor may survive closeout
- store-backed and durable sources may remain typed Milestone `10`/`11`
  postures, but the authority product must already have explicit extension
  points and fail-closed support rows for them
- documentation-only migration history may remain; callable compatibility
  authority may not

### Sequencing Notes

The detailed execution plan lives in [milestone-9.11.md](./milestone-9.11.md).
This milestone belongs after `9.10` because it applies the declarative-admission
product standard to downstream basis authority, and before `10` because
store-backed execution must inherit one canonical handoff rather than multiply
consumer-side pairing folklore.

### Parallelization Notes

Closure inventory and DX prototyping may overlap. The canonical artifact and
denial taxonomy freeze before public fluent DX. Query integration precedes
consumer cutover; architectural certification and legacy deletion close last.

### Store Dependency

- Runtime-backed canonical downstream authority and Worth UI adoption are not
  blocked on `worth-store`.
- Store-backed source admission and durable authority reload remain Milestone
  `10`/`11` owners and must enter through explicit typed postures.

### Acceptance Evidence

- cross-basis and cross-receipt hybrid construction is unrepresentable or
  uncompilable
- fluent, explicit, replayed, and serialized contract paths are equivalent
- every admitted source yields one authority; every failed relationship yields
  one typed denial and no partial successor
- Worth UI scroll and portal paths preserve exact Query authority without
  local reconstruction
- Consumer Kit residue and public-boundary audits report zero legacy authority
  seams
- docs, AI orientation, support rows, DX transcripts, and implementation agree

## Milestone 9.12: Query Public Authority Surface Cutover

Status: Closed on 2026-07-13. The ordinary facade now exposes one sealed
authority-preserving path per supported capability; legacy authority minting,
parallel basis lifecycle, raw admission, and ordinary-facade tooling seams are
deleted or mechanically prohibited. The milestone spec records the complete
package, hostile, compile-fail, reference-consumer, server-consumer, workspace,
formatting, and hygiene evidence.

### Goal

Make Query's ordinary public facade expose one sealed authority-preserving path
per capability, with raw representations, lifecycle internals, compatibility
adapters, and certification machinery unable to act as parallel authority APIs.

### Adversarial Constraint

A downstream consumer must be unable to mint, restamp, pair, or route Query
authority through a raw digest, string identity, posture enum, raw admission
request, legacy unscoped lifecycle, or certification-only artifact—even when
each supplied component is individually well-formed or collides textually with
a legitimate Query artifact.

### Why This Milestone Exists

Milestone 9.11 closes the canonical downstream authority product, but Query's
broader facade still contains older and lower-level construction paths capable
of recreating competing authority. Those paths must be removed before
store-backed execution extends the same contracts across another substrate.

### Specification

The governing milestone spec is
[milestone-9.12.md](./milestone-9.12.md).

### Must Ship

- sealed digest and identity authority construction
- one scoped public basis lifecycle
- declarative intent admission with raw admission machinery internal
- scoped subscription, causal inspection, and preview-live follow-on paths
- contracted ordinary facade separated from certification and migration tools
- consumer cutover, named prohibitions, compile-fail and residue enforcement,
  public API snapshots, and hostile certification

### Must Preserve

- Query and every lower runtime retain their existing semantic authorities
- equivalent ordinary declarations converge on one sealed capability chain
- evidence labels, digests, diagnostics, and serialized projections remain
  useful but non-promotable
- ordinary developer ergonomics remain declarative
- rejection precedes expensive construction and lower-runtime contact

### Complexity / Proof Obligations

- admission work remains proportional to declared inputs and admitted facts,
  independent of unrelated workspace, history, subscription, or consumer state
- exact counters prove invalid relationships deny before construction or
  lower-runtime contact
- hostile collision, replay, stale-generation, cross-basis, cross-receipt, and
  phase-skipping tests prove representation cannot become authority
- compile-fail, facade snapshot, prohibition, residue, and sabotage suites prove
  every removed path is mechanically non-resurrectable

### Allowed Debt

- store-backed and durable implementations remain Store roadmap scope
- no runtime-backed raw constructor, unscoped operational entrypoint, callable
  compatibility adapter, or ordinary-facade certification leak may remain debt

### Sequencing Notes

This belongs after 9.11 because it contracts Query's complete public authority
surface around the canonical downstream product. It belongs before 10 because
store-backed execution must inherit one sealed lifecycle rather than multiply
legacy and canonical paths across runtime and store substrates.

### Store Dependency

This milestone is not blocked on `worth-store`.

### Acceptance Evidence

- raw representation cannot mint operational Query authority
- ordinary basis, intent, subscription, preview, and inspection paths consume
  sealed proof-bearing predecessors
- covered compatibility-debt rows and callable migration APIs are absent
- public facade, support posture, docs, reference consumers, and hostile
  certification agree on one authority path per capability

## Milestone 9.13: Declarative Query Experience And Phase-Surface Cutover

Status: Core Phases 1-12 closed on 2026-07-14 for the runtime-backed
declarative product boundary. Add-on Phases 13-30 are open. Phases 13-20 close
runtime-installed domain packages and single domain-capability authority;
Phases 21-30 close Foundational-native aspect value authority, portable
readmission, Relational transaction integration, durability, and consumer DX.
The ten-family grammar, ordinary/internal parity, managed lifecycle,
facade/prohibition/residue enforcement, and reference-consumer adoption remain
certified at the original boundary. Store-backed execution and durable
artifact/continuation claims remain Store Milestones 9 through 13. See
[milestone-9.13-closeout.md](./milestone-9.13-closeout.md) for the historical
Phases 1-12 evidence and [milestone-9.13.md](./milestone-9.13.md) for the open
add-on phase contract.

### Goal

Make ordinary Query usage capability-oriented and declarative so consumers
describe desired outcomes while Query owns its proof-preserving internal phase
progression, managed lifecycles, backend routing, receipts, and diagnostics.
Extend that product boundary so a typed domain package is installed into one
concrete runtime, which alone mints runtime-affine domain handles and derives
operation, invariant, obligation, declaration-family, and contribution indexes.
Complete the boundary by carrying Foundational's exact scalar, struct, patch,
and state meaning through trust-boundary readmission, Relational transactions,
durability and replay, Query authoring, predicates, schema capability,
materialization, projection consumption, identity, and typed refinement.

### Adversarial Constraint

An ordinary consumer must be unable to skip, reorder, reproduce, or become the
owner of Query's canonicalization, binding, validation, admission, planning,
lowering, execution, maintenance, or envelope phases. Equivalent declarations
must converge on the same canonical meaning and outcome without guessing basis,
policy, tenant, bound, lifecycle, or backend decisions.

Equivalent domain packages must install to the same semantic meaning regardless
of declaration order. Raw domain strings, consumer-authored identity digests,
foreign-runtime handles, stale installation generations, independent operation
registries, and public materializers must be unable to create competing domain
authority or reach lower-runtime work.

Every Foundational scalar family and admitted struct aspect must survive the
complete Query journey without a coarse Query-owned enum, generic integer
collapse, local string encoder, scalar-only bridge path, or misleading row
marker becoming competing value authority. Invalid value/operator/contract
combinations must deny before planning or lower-runtime work.

### Why This Milestone Exists

Milestone 9.12 makes authority-preserving phase artifacts safe, but the public
surface can still require consumers to assemble safe low-level stages into
local Query coordinators. That is both poor DX and an architectural fork point.
The ordinary experience must close before store-backed execution would
otherwise multiply phase-shaped journeys across backends.

The core closeout also revealed that typed configured handles, string-authored
contributions, graph-operation registries, and actual runtime construction do
not yet share one mechanical authority. The add-on phases close that fork before
store-backed execution can multiply it.

The native substrate also retains Foundational values internally while ordinary
mutation, predicate, schema, struct, row, and refinement surfaces expose a
smaller or inconsistent language. Closing only JSON residue would let Store
Milestone 1 inherit a second value ontology. The final add-on phases close that
fork at the consumer boundary.

### Specification

The governing milestone spec is
[milestone-9.13.md](./milestone-9.13.md).

### Must Ship

- capability-oriented facade and progressive-disclosure DX grammar
- one declarative authoring path into canonical Query meaning
- declarative context handoff for basis, binding, tenant, policy, and
  relationship proof
- admitted-query read execution and framework-owned live resource lifecycle
- coherent history, diff, correspondence, preview, workflow, result, receipt,
  projection-consumption, and inspection journeys
- internalized phase transitions and contracted ordinary facade
- reference-consumer cutover, DX measurements, permanent prohibitions,
  residue enforcement, and hostile certification
- canonical typed domain package and Query-owned domain identity encoding
- atomic pre-runtime domain installation and runtime-affine installed handles
- package-compiled invariant, graph-obligation, graph-read operation,
  declaration-family, and contribution substrates
- installed-handle contribution, continuation, recovery, receipt, and
  inspection authority
- removal of raw-string contribution entry, manual operation registries,
  executable application-facade handles, and Query-owned product-domain helpers
- downstream extension DX, consumer recutover, and hostile single-authority
  certification
- full native authoring for every Foundational scalar family and representative
  admitted struct aspects through ordinary mutation and workflow journeys
- contract-derived schema and predicate capability with exact native operands
  rather than coarse Query-owned value or field-kind authority
- native scalar/struct retained results, projection facts, typed refinement,
  and an honest row product or explicitly internal marker
- one Foundational canonical value identity basis, Query-specific typed domain
  separation, duplicate-encoder deletion, facade contraction, consumer cutover,
  and hostile native-value certification

### Must Preserve

- Query and lower-runtime authority boundaries
- family-specific cost, failure, correctness, and lifecycle distinctions
- explicit basis, policy, tenant, branch, preview, and relationship authority
- backend-independent canonical meaning and ordinary outcomes
- advanced domain contribution without public internal-pipeline access
- semantic domain setup remains distinct from physical storage, schema, source,
  signal, and transport adapters
- installed domain artifacts remain authoritative while runtime lookup indexes,
  support views, and diagnostics remain derived and rebuildable
- Foundational remains the sole authority for aspect value families, structs,
  contracts, canonical wrappers, validation, and canonical value identity;
  Query adds proof-bearing intent, capability, result shaping, and DX
- struct meaning survives until an explicit projection selects native leaves,
  and raw authored values cannot promote into stronger Query proof states

### Complexity / Proof Obligations

- ergonomic lowering adds work only proportional to declared inputs and
  admitted context
- invalid declarations deny before planning or lower-runtime contact
- one-shot/live and ordinary/internal-oracle parity are exact for canonical
  identity, results, receipts, diagnostics, and counters
- facade snapshots, compile-fail, prohibition, residue, sabotage, lifecycle,
  and reference-consumer suites prove phase machinery cannot reappear
- installed handle lookup and operation resolution remain bounded indexed work
  as unrelated package and operation counts grow
- invalid, foreign, or stale installed authority performs exact zero planning,
  lower-runtime, and execution work
- native refinement is constant time per selected value; predicate admission is
  proportional to declared predicates and contract fields; invalid native
  values perform exact zero planning and lower-runtime work
- complete scalar-family and representative-struct parity, canonical identity,
  facade, residue, sabotage, and reference-consumer suites prove a second value
  authority cannot reappear

### Allowed Debt

- store-backed execution, durable restore, saved-artifact survival, and durable
  continuation remain Store Milestones 8 through 13
- no callable ordinary phase transition, backend-shaped ordinary API,
  compatibility alias, or consumer-local Query coordinator may remain debt
- no raw-string domain entry, consumer-authored identity digest, independent
  operation registry parameter, public contribution materializer, or executable
  application-facade handle may remain debt
- no coarse Query-owned scalar/schema enum acting as value authority, duplicate
  production value encoder, scalar-only struct rejection, raw-to-proof
  conversion, or phantom native-row teaching may remain debt

### Sequencing Notes

This belongs after 9.12 because the proof and authority chain must be sealed
before its internal transitions are hidden behind a simpler product surface.
It belongs before 10 because runtime-backed and store-backed execution must
implement the same admitted capability contract and ordinary journey.

Add-on Phases 13-20 follow the closed core because they install domain
extensions into the already-frozen ordinary capability grammar. Phases 21-30
then close exact native value semantics across Foundational readmission,
Relational transactions, durability, and those installed and ordinary
journeys. All remain before Milestone 10 because runtime domain authority,
registry ownership, native predicate meaning, and native result/materialization
contracts are runtime-semantic foundations, not store implementation details.

### Store Dependency

This milestone is not blocked on `worth-store`. Add-on closure requires atomic
in-process runtime installation, generation-safe handles, Foundational-native
contracts, and the existing runtime-backed materialization/projection substrate,
not durable package reload or cross-process restoration. Store Milestone 1 must reuse
the resulting value, predicate, row, and projection contracts unchanged.

### Acceptance Evidence

- ordinary consumers declare capabilities without manual phase progression
- phase artifacts are observable where useful but not constructible or
  independently advanceable
- invalid combinations yield typed next-action stops without guessed defaults
- live resources are framework-owned and deterministically disposable
- reference consumers delete local Query orchestration and preserve exact
  canonical outcomes
- ordinary facade, docs, support posture, prohibitions, residue audits, and
  hostile certification agree
- equivalent packages install identically while conflicts fail atomically
- only the installing runtime can mint and accept an installed domain handle
- operations and contributions resolve from installed authority without raw
  strings, manual registries, semantic adapters, or public phase materializers
- derived installation indexes rebuild from installed artifacts without
  changing resolution, denial, receipt, or diagnostic identity
- every Foundational scalar family and representative struct value round-trips
  through admitted public authoring, runtime-backed execution, materialization,
  projection consumption, typed refinement, receipt identity, and inspection
- schema/operator admission derives from Foundational contracts, native value
  identity has one canonical basis, and consumer/facade/residue/sabotage proof
  reports zero competing value authorities

## Milestone 9.13.1: Query Iteration Foundation

Status: Phases 1-3 closed on 2026-07-18. Phases 4-8 are open.

### Goal

Restore livable Query iteration through direct structural cuts: remove repeated
compiler setup and consumer coupling, dismantle the giant library-test binary,
isolate cold certification, eliminate reconstructive test hotspots, and
extract declaration and installation as the first permanent authority packages.

### Adversarial Constraint

Declaration and installation edits must compile and test without the remaining
monolith, replay, or certification. Ordinary Query behavior must not share a
manually assembled library-test binary with public journeys, exhaustive
convergence matrices, or cert-only reconstruction. One compiler invocation
retains selected compiler-owned denials. Every slice inventories only the cost
boundary it immediately removes or extracts; a repository-wide inventory,
proof platform, or timing framework cannot become a prerequisite for obvious
work.

### Why This Milestone Exists

The first cut reduced warm compiler certification from roughly 399 seconds to
roughly 4 seconds, exposing the next structural floor: one approximately 2,981-
test library binary still takes roughly 118 seconds warm and can take roughly a
minute to rebuild after production edits. Manually injected integration suites,
cold certification, repeated installed-package reconstruction, and multiple
production authorities still share artifacts Cargo cannot select independently.
These defects have direct target, logic, and package-boundary fixes.

### Specification

The governing milestone spec is
[milestone-9.13.1.md](./milestone-9.13.1.md).

### Must Ship

- ordinary Query library tests with no trybuild execution
- one direct compile-fail certification target containing only load-bearing
  authority, substitution, phase-ordering, ownership, and facade denials
- positive public journeys owned by ordinary integration tests or doctests,
  with no compile-pass trybuild loop
- deletion of historical tombstones, generic certification-artifact privacy
  probes, orphan diagnostics, and production-owned fixture registries
- no Query certification that reads Worth UI source
- dedicated `workspaces/worth-query` ownership for the engine, audience
  facades, and cold certification, with the repository root remaining an
  orchestrator rather than the Query package owner
- permanent cold `worth-query-certification` package with no ordinary reverse
  dependency
- deletion of the manual library-test integration aggregator and same-slice
  classification of its directly injected suites
- repair of observed reconstructive test hotspots without parallelism, fakes,
  ignored tests, or timing infrastructure
- permanent `worth-query-declaration` and `worth-query-installation` packages
  with owner-local tests and machine-enforced dependency direction
- direct documented Cargo commands with no custom runner, proof inventory,
  manifest language, cache system, receipts, shards, or timing framework

### Must Preserve

- every distinct compiler-visible authority-minting, substitution,
  phase-ordering, ownership, move-only, facade, and Query authority invariant
- canonical Query meaning, public behavior, runtime authority, support posture,
  lower-runtime ownership, and Store handoffs
- the exact intended diagnostic boundary of each retained compiler fixture
- Query and Worth UI ownership direction
- canonical declaration and installed-package meaning, runtime affinity,
  conflict atomicity, and rebuildable-index truth
- the final authority DAG in which declaration precedes installation and no
  ordinary package depends on certification

### Complexity / Proof Obligations

- compile-fail registration enters trybuild's bulk `--bins --keep-going` path
- positive journeys compile and execute through ordinary Cargo test ownership
- one before/after observation is enough to decide whether the cut worked;
  repeated performance sampling is forbidden here
- owner-local declaration and installation commands omit later and cold Query
  authorities and are observed after representative owner-local invalidation
- ordinary remaining-monolith behavior omits compiler, replay, source-audit,
  and consumer-workspace work

### Allowed Debt

- admission, execution, publication, and parity-gated final consumer cutover
  with exact predecessor-authority retirement remain for Milestone 9.13.2
- cold certification may temporarily depend on the shrinking monolith until
  9.13.2 retargets it; no ordinary package may depend back on certification
- repeated harness construction and Query-to-Worth-UI test coupling may not
  remain debt
- the manual library-test aggregator, repeated reconstructive hotspot, and
  failure to extract declaration or installation may not remain debt

### Sequencing Notes

This follows 9.13 because test and package breadth already block ordinary work.
It establishes the upstream half of the authority graph so 9.13.2 can complete
the covered authority split with useful inner loops. Milestone 9.14 follows the
completed cutovers so new installed-operation semantics enter the intended
package graph.

### Store Dependency

This milestone is not blocked on Store and changes no provider, durability,
replay, or Store-facing semantic contract.

### Acceptance Evidence

- one successful ordinary library run with no trybuild output
- one successful direct compiler-certification run over the selected denials
- one successful Worth UI Query-binding run owned by the Worth UI workspace
- one successful cold-certification run absent from ordinary package closures
- the manual library-test aggregator is absent and every directly injected
  suite has an explicit product, certification, or deletion disposition
- repaired package-validation scenarios preserve exact convergence, conflict,
  rebuild, and counter outcomes without repeated broad setup
- declaration and installation owner commands omit later/cold authorities and
  one same-machine observation per slice shows the loop is measured in tens of
  seconds rather than minutes
- one before/after elapsed observation for each real slice command
- repository review confirms the deleted runner-platform components are absent

## Milestone 9.13.2: Query Authority Crate Decomposition

Status: Open. Begins after Milestone 9.13.1 closes.

### Goal

Complete the legal Road 1 Query authority graph established by 9.13.1: extract
admission, execution, and publication one semantic authority surface at a
time; retarget cold certification; and cut each covered consumer to an audience
facade with full parity and atomic predecessor retirement. This removes
competing authority while retaining the `worth-query` product composition root
and unrelated feature behavior.

### Adversarial Constraint

A local change to one remaining Query authority must compile and test without
building later or cold authorities. Every slice inventories and moves only its
owned mixed modules and consumers. No facade, dependency edge, shared test
support crate, compatibility re-export, or migration package may bypass the
finished graph or create a second meaning source.

### Why This Milestone Exists

Milestone 9.13.1 establishes cold certification and permanent declaration and
installation packages, but admission, execution, and publication remain in a
shrinking migration package. The completed production graph is the honest long-
term selection mechanism: code and tests become local because authority and
dependencies are physically local, not because a runner interprets manifests.

### Specification

The governing milestone spec is
[milestone-9.13.2.md](./milestone-9.13.2.md).

### Must Ship

- a reviewed authority and dependency graph derived from production meaning
- exact reviewed Query-framework naming amendments for admission, execution,
  and publication completing the declaration, installation, and certification
  amendments established by 9.13.1
- narrow audience facades preserving the current ordinary public journey
- authority-local unit, integration, and compiler tests living with their
  production owner
- same-slice deletion of any residual source-topology audit, proof-of-proof
  test, or shared test-platform temptation encountered at an authority boundary
- parity-gated retirement of each authority-capable predecessor surface after
  its covered consumers cut over; lawful one-way product composition remains

### Must Preserve

- Query's canonical declaration, admission, planning, lowering, execution,
  result, lifecycle, and certification meaning
- lower-runtime authority and Store handoffs
- every compiler-owned public denial

### Acceptance Evidence

- boundary-check and agent-context enforcement over the new graph
- ordinary consumer transcripts compile through intended facades
- each authority package runs its owned tests without unrelated Query packages
- full Query and workspace certification remain green after every covered
  authority cutover
- every covered predecessor authority and authority-capable compatibility re-
  export is absent; the `worth-query` composition root remains green and lowers
  one way into the sole destination authority

## Milestone 9.14: Installed Operation Semantics, Semantic Aspect Correspondence, Conditional Signal Authority, And Bound Downstream Authority

Cross-runtime carriage companion:
[WORTH Signal Milestone 13.1](../WORTH_signal/milestone-13.1-plan.md) consumes
the installed operation, correspondence, conditional Signal, impact, and
downstream-authority contracts established here. Its completed platform
carriage preserves those owners and creates no second Query authority lane;
see the [cross-runtime closeout](../WORTH_signal/milestone-13.1-closeout.md).

### Goal

Make Query's downstream projection path mechanically complete by carrying
installed domain operation meaning, declaration, installation, one operating-
world entry root, explicit graph participation, basis, atomic cross-domain and
admitted cross-graph binding, workflow progression, execution trace, ordinary
re-execution, Query-authored portable semantic truth dependencies and
conditional-node meaning, aspect-precise Relational publication, installed
runtime-bridge correspondence into actual Signal node-local aspect slots, exact
conditional lowering, Signal-owned evaluation decisions, Query re-entry, cert-only
replay, reversal/compensation posture, typed lineage,
derived publication, execution, consumption, native refinement, consumer
support, compiled dependency impact, equivalent-capability sharing, managed
consumer leases, compatibility, invalidation, collection windows, query-shaped patch delivery,
and managed lifecycle through one sealed runtime-affine capability.

### Adversarial Constraint

Independently valid handles, alternate entry roots, graph adapters, operation-
family facades, portable or locally reconstructed definitions, domain
capabilities, stage receipts, replay or reversal scopes, lineage candidates,
completions, bases, fact receipts, support projections, dependency labels,
equivalence tokens, reporting digests, invalidation labels, cursors,
collection patches, portable semantic aspect dependencies, authoritative
truth-delta targets, aspect-correspondence witnesses, raw Signal aspects/masks,
raw or copied Signal conditions, condition/comparator or trigger labels, bridge
lowerings, detached Signal decisions, leases, and
lifecycle artifacts from foreign runtimes,
stale generations, or different declarations must be impossible to recombine
into operational projection authority. Equivalent work may share only when
Query proves the complete authority and semantic equivalence contract and
mints independently disposable leases. Invalid combinations deny before
planning, sharing, lower-runtime contact, refinement, invalidation, patch
application, or lifecycle work with exact counters.

Truth aspect identity/revision, field mask, binding, change meaning, surface,
locality, installed Signal node/aspect slot, precision posture, condition, or
comparator drift and foreign node, graph, runtime, basis, snapshot, trigger/wake,
or attempt evidence deny before graph mutation, evaluation, or Query consequence
work. Ineligible, suppressed, deferred, and reverted-clean outcomes cannot be
reported or delivered as new computed output.

### Why This Milestone Exists

Milestone 9.13 closes declarative installation and Foundational-native value
meaning, but a downstream consumer can still fail if Query exports safe
ingredients that the consumer must coordinate correctly, if stable domain
operations locally rebuild schema and result meaning, if workflows maintain
product-specific stage ledgers, replay and undo catalogs, or representation-
derived lineage, if operation families construct separate runtime roots, if
Query cannot express exact Relational truth dependencies, if field-level or
endpoint change meaning is widened or discarded before the bridge, if bridge
stable names masquerade as installed Signal aspects, if slot coalescing hides
precision loss, if Query cannot author Signal's real conditional nodes, if
Query or the bridge re-decides eligibility, or if skipped and reverted-clean work is conflated,
if applications bridge graphs through raw handles or hidden adapters, if local
dependency graphs reinterpret change impact, or if
equivalent capabilities create duplicate execution and subscription resources.
Query must install one complete operation semantic closure and export the
single operating-world root, explicit graph participation, atomic bound
capability, workflow trace, portable conditional-node authoring, exact bridge
semantic-aspect correspondence, conditional lowering, Signal decision
provenance, Query re-entry, cert-fenced replay,
aftermath, lineage, derived publication, consumer support contract, dependency
impact, sharing and lease contract, invalidation delta,
collection window/patch contract, and named compatibility decisions before
Milestone 13 freezes provider-independent certification and before Store
integration multiplies the boundary.

### Specification

The governing milestone spec is
[milestone-9.14.md](./milestone-9.14.md).

### Must Ship

- complete installed operation semantic closure with indexed resolution
- Query-authored portable conditional-node contracts for typed trigger inputs,
  conditions, comparators/equivalence, temporal/on-demand posture, and outputs
- Query-authored semantic truth dependencies retaining exact Foundational
  contract, Relational binding/surface, field mask, locality, and relevant
  change meaning
- aspect-precise Relational authoritative publication plus runtime-affine
  installed correspondence into actual Signal graph/node/aspect targets with
  exact or declared-widening successful witnesses and checked unsupported,
  ambiguity, capacity, stale, rebind, denial, and failure outcomes that mint no
  witness
- mandatory reuse of Foundational shared aspect/canonical/identity/evidence/
  performance vocabulary and `worth-proof` progression/basis/freshness/witness/
  outcome/proven-collection law behind stronger owner-specific runtime types
- pair-bound runtime-bridge lowering into installed Signal node contracts, with
  separately registered volatile providers
- Signal-minted eligibility, deferral/suppression, computation, and reverted-
  clean evidence admitted back into Query without restamping
- one installed operating-world root with typed borrowed operation-family
  facades and no alternate authority-bearing entry points
- installed graph-participation adapters only for genuinely separate graph
  authority, with explicit atomic or compensated commit posture
- atomic multi-domain and admitted multi-graph operation binding
- installed workflow DAGs and Query-minted workflow run/stage traces
  retaining legal conditional-outcome progression
- ordinary retry/re-execution and cert-only trace replay with distinct attempt
  identity and declared equivalence
- typed exact-inverse, compensation, recovery, and irreversibility posture
- trace-bound lineage and identity-evolution evidence for persistent naming and
  promotion-on-reference
- one non-detachable bound projection capability
- one Query-minted consumer support contract
- proof-bearing installed execution, derived publication, consumption, and
  settlement progression
- declaration-indexed Foundational-native access without consumer fact scans
- Query-owned installation, basis, replacement, rebind, and reuse decisions
- capability-bound live, replacement, rebind, cancellation, and disposal
- Query-compiled dependency roles, impact closure, and typed impact decisions
- Query-admitted equivalent-capability coalescing with framework-owned shared
  execution resources and independently disposable consumer leases
- capability-bound consumer invalidation deltas
- bound collection identity/window declarations and query-shaped patch delivery
- opaque operational identities separated from reporting projections
- exact boundary-local counters, facade cutover, reference-consumer deletion,
  reusable operation/workflow certification kit, permanent prohibitions, and
  hostile certification

### Must Preserve

- Foundational remains shared native aspect/value/mask/locator, canonical
  comparison, boundary identity/evidence, provenance/lineage/support, and
  performance vocabulary authority
- `worth-proof` remains proof-bearing progression, checked outcome, assumption-
  basis/freshness/readmission, concrete witness spending, structural-fact, and
  honest fixed-shape collection authority
- Query, Relational, Bridge, and Signal retain stronger owner-specific runtime
  authority; raw Foundational or `worth-proof` carriers cannot replace their
  capabilities, patches, correspondences, decisions, receipts, or lifecycle
  states
- Query remains installed operation, declaration, installation, basis,
  projection consumption, consumer support, compatibility, dependency impact,
  sharing admission, invalidation, collection identity/ordering, cursor/patch
  meaning, and managed lifecycle authority
- Query owns portable conditional authoring, the runtime bridge owns exact
  crossing/lowering, and Signal owns runtime condition/comparator resolution
  and evaluation decisions; Query effect conditions remain separate
- Query owns portable semantic truth dependency intent, Relational owns
  authoritative aspect-change interpretation, the runtime bridge owns installed
  truth-to-Signal correspondence, and Signal owns node-local aspect slots,
  versions, invalidation, and scheduling
- Relational/Foundational semantic aspects remain distinct from runtime-local
  Signal aspects; stable names, equal numeric slots, and reporting digests have
  zero correspondence authority
- dependency-version comparison, condition eligibility, post-compute output
  equivalence, and artifact-reuse equivalence remain distinct; unitful domain
  thresholds retain canonical native value and unit/tolerance meaning
- Query remains atomic cross-domain binding, workflow progression and trace,
  graph-participation and cross-graph binding, cert replay admission, reversal
  binding, lineage transport, and identity-evolution authority while domains
  own algorithms, compensation/inverse meaning, and naming/correspondence policy
- operation-family entry facades remain typed borrowed views over one root;
  typed family distinctions do not create separate runtime authority
- one logical graph remains the default; graph participation is introduced only
  for a genuine independent authority boundary
- Store remains durable journal, checkpoint, restart replay, and recovery
  authority
- lower runtimes remain truth, scheduling, and physical-provider authorities
- Signal decision provenance remains bound to the exact node, graph, runtime,
  condition, comparator, trigger/wake, basis, snapshot, and attempt through
  every later Query authority that consumes it
- downstream runtimes retain graph, mounting, allocation, viewport, overscan,
  and presentation consequence authority
- portable declarations and reporting projections remain non-authoritative
- portable operation definitions remain non-operational until installed, and
  derived sharing indexes remain rebuildable rather than authoritative
- snapshot, live, replacement, rebind, reuse, inspection, and disposal retain
  distinct cost and failure contracts

### Complexity / Proof Obligations

- installed operation resolution is one bounded indexed lookup independent of
  unrelated domains and operations
- graph-participation and operation-family resolution inspect only the
  operation's declared roles and never scan unrelated graphs or entry families
- native access is `O(k)` for `k` declared access keys and `O(1)` per admitted
  key after declaration binding
- compatibility cost is bounded by retained proof dimensions and independent of
  registry, consumer, or diagnostic projection counts
- dependency compilation and impact resolution are bounded by declared and
  affected dependency edges; shared maintenance plus delivery is bounded by
  semantic maintenance breadth plus admitted lease fan-out
- condition admission, invalidation narrowing, condition/comparator checks,
  domain compute, reverted-clean classification, semantic change, and delivery
  expose separate exact counters and never scan unrelated nodes or consumers
- authoritative truth-delta lowering and semantic-aspect correspondence are
  bounded by emitted patch targets plus admitted mapping fan-out; exact/widened
  matches, ambiguity/capacity denials, Signal seeds, node fan-out, and slots
  touched expose separate exact counters and never scan unrelated schema,
  mappings, nodes, consumers, or reporting scopes
- invalidation of `k` capabilities and `l` leases, window width `w`, and patch
  width `p` expose exact independent work counters and do not scale with
  unrelated collections, consumers, graph nodes, or diagnostic projections
- foreign, stale, detached, wrong-contract, and disposed inputs perform exact
  zero downstream planning, lower-runtime, refinement, invalidation, patch, and
  lifecycle work
- ordinary facade, internal oracle, and provider-independent certification
  report identical outcomes and exact counter snapshots

### Allowed Debt

- physical persistence and provider integration remain in the Store roadmap
- no local stable-operation reconstruction, detached authority assembly,
  consumer-owned compatibility or dependency closure, consumer-owned sharing
  registry, operational digest comparison, support/invalidation mirror, broad
  fact/collection scan, local patch-posture hash, raw CDC interpretation, or
  orphan lifecycle join, alternate entry root, hidden graph adapter, direct
  graph-to-graph call, shared-ID authority, or ordinary replay import may
  remain debt
- no raw Signal configuration in the Query facade, custom-string condition
  dispatch, local condition evaluation, comparator mirror, eligibility
  restamping, detached bridge request, raw unitless `f64` domain threshold, or
  output-presence inference may remain debt
- no silent field-to-whole Relational widening, opaque-to-lifecycle fallback,
  bridge stable-name authorization, raw Signal-slot interpretation, consumer-
  minted correspondence, ambiguous runtime mapping, or catch-all slot-pressure
  collapse may remain debt
- no Query/Relational/Bridge/Signal mirror of Foundational aspect, canonical,
  identity, provenance, support, or performance vocabulary; no private proof,
  recipe, basis, freshness, outcome, witness, or proven-collection substrate
  parallel to `worth-proof`; and no raw shared carrier accepted as stronger
  owner authority may remain debt

### Sequencing Notes

This follows 9.13.2 because it consumes the installed-domain and native-value
authority model established by 9.13 through the deterministic, authority-local
package and proof topology established by 9.13.1 and completed by 9.13.2. Graph
participation and one-root authority close before operation binding;
Phases 1-6 retain conditional identity and authority positions, then Phases
7-11 close portable semantic dependency and conditional authoring, aspect-
precise Relational publication, installed Relational-to-Signal correspondence,
bridge/Signal evaluation, and Query re-entry
before replay and later downstream authorities. Publication and the cert replay
fence close before workflow consumers. It
precedes Milestone 13 because the provider-independent oracle must certify the
completed downstream capability, not ingredient surfaces that consumers can
misassemble.

### Store Dependency

This milestone is not blocked on Store. Store integration must consume the
same installed operation definition and resolution, bound capability, consumer
support, graph-participation, single-root, publication, cert-replay,
conditional authoring/lowering/decision provenance, dependency-impact, sharing
and lease, compatibility, invalidation,
collection window/patch, lifecycle, and accounting contracts without reopening
them.

### Acceptance Evidence

- equivalent installed operation definitions converge while conflicts fail
  atomically and local lookalikes cannot execute
- every operation family borrows one installed operating-world root and cannot
  construct an alternate runtime or accept raw graph handles
- same-graph domain composition needs no adapter; separate graph authority
  participates only through an installed adapter, and multi-graph atomicity
  requires shared commit authority
- invalid workflow graphs fail installation, independently valid stage
  receipts cannot be mixed, and required multi-domain capabilities bind only
  as one atomic authority
- ordinary Query authoring creates real aspect-filtered, threshold, temporal,
  on-demand, and typed domain-specific Signal-backed nodes without raw Signal
  configuration or custom-string dispatch
- field-level, whole-aspect, relation-endpoint, structural, lifecycle, and
  opaque Relational changes retain exact or explicitly widened posture through
  the canonical bridge truth-delta path
- installed semantic-aspect correspondence admits exact one-to-one, one-to-
  many, declared many-to-one widening, and derived-only Signal relationships;
  ambiguity, unsupported precision, and slot pressure deny atomically without
  catch-all collapse or graph residue
- exact and declared-widening are the only successful correspondence-witness
  postures; unsupported, ambiguous, capacity-exhausted, stale, rebind-required,
  denied, and failed candidates retain distinct `worth-proof` outcomes and can
  never recover a witness
- owner-specific Query, Relational, Bridge, and Signal types reuse Foundational
  shared meaning and `worth-proof` progression without semantic mirrors,
  duplicate freshness/outcome lattices, raw generic facade authority, or
  caller-selected witness marker types
- real Relational changes reach only correspondence-bound Signal node/aspect
  versions through production delivery; bridge stable names, matching digests,
  and equal numeric slots cannot authorize or redirect invalidation
- equivalent conditional declarations and bridge lowerings converge; one-field
  condition, comparator, trigger/wake, Signal graph/node, basis, snapshot, or
  attempt drift denies before forbidden later work
- Signal alone decides eligibility and semantic cleanliness: skipped/deferred
  nodes compute zero times, reverted-clean nodes retain compute cost but deliver
  no semantic change, and changed output enters Query consequences once
- cert-only replay uses distinct run/attempt identity while proving declared
  result/publication/effect/lineage equivalence or localizing typed divergence;
  ordinary consumers cannot import replay and consume publications instead
- touched scope cannot mint undo; inverse, compensation, recovery, and
  irreversibility remain explicit and separately certified
- persistent naming cannot be authorized from raw IDs, strings, digests,
  geometry, rendered values, or advisory correspondence
- promotion-on-reference never eagerly gives graph identity to unreferenced
  artifact payload
- external construction and sabotage probes cannot mint or recombine authority
- complete native scalar, struct, absence, and refinement parity survives the
  bound capability path
- support contracts and invalidation deltas are Query-minted rather than
  rebuilt from consumer hook enums or digest bundles
- dependency impact converges across one-shot, live, and replay paths, and
  equivalent capability sharing performs one maintenance pass with exact
  per-lease fan-out and disposal evidence
- collection windows and patches preserve identity, ordering, cursor,
  continuation, result-state, and fresh-execution parity
- replay, compatibility, dependency impact, sharing, lifecycle, invalidation,
  accounting, and patch delivery consume retained Signal decision provenance
  without Query or consumers restamping it
- Worth UI and another reference consumer delete local stable-operation
  builders, Query authority, dependency/recompute policy, sharing registries,
  support, invalidation, and patch-posture assembly and converge with Query's
  internal oracle
- facade snapshots, compile-fail probes, residue scans, exact-counter tests,
  lifecycle tests, reusable operation/workflow certification fixtures, and
  provider-oracle tests agree on one capability path

## Milestone 9.15: Managed Domain Computation, Proposed State, And Invariant Execution

**Status:** Complete through Phase 10.

### Goal

Close the honest pre-commit foundation for installed domain computation:
governed artifacts and occurrences, bounded native access, real resource
admission and managed runs, provider-bound sessions, basis-complete decision
read-sets, isolated proposed state, and installed invariant execution.

### Specification

The governing milestone spec is
[milestone-9.15.md](./milestone-9.15.md).

### Must Ship

- installed artifact, occurrence, reproducibility, search, convergence,
  transformation, counter, decision, and invariant contracts
- runtime-affine managed artifacts and in-memory yield continuations
- live capacity reservation, saturation, backpressure, cancellation,
  readmission, exhaustion, degradation, and exact cleanup
- sealed provider plans and sessions that require installed-operation authority
- complete positive, negative, membership, ordering, traversal, artifact, and
  structural decision facts
- isolated provisional attempts and exact proposed-state inspection
- real invariant execution tied to exact session, attempt, and proposal
- permanent authority, governance, counter-unit, memory-accounting, lifecycle,
  and consumer proof

### Boundary

Milestone 9.15 does not claim commit, public application authoring,
authentication, authorization, or advanced access products. Those capabilities
have one governing home in Milestone 9.16 and Milestones 9.19 through 9.22.

## Milestone 9.16: Authenticated Async Bank World And The Ordinary Query Front Door

**Status:** Runtime Hardening Phases 1-10 and Bank World Phases 1-5 are closed.
The accepted aftermath, external-effect, recovery, retention, and publication
foundation is proved through corrections C1-C8 under the
[Runtime Phase 8 finish plan](./milestone-9.16-runtime-phase-8-finish-plan.md).
Runtime Phase 9 closes host-installed conditional providers, managed clocks,
Signal-owned temporal wakes, and reconstruction from authoritative
Relational/domain truth. Runtime Phase 10 closes the public-policy cutover,
developer guidance, facade contract, reference-consumer classification, and
workaround residue under their
[Phase 9](./milestone-9.16-runtime-phase-9-closure-ledger.md) and
[Phase 10](./milestone-9.16-runtime-phase-10-closure-ledger.md) ledgers.
The existing undo/redo lane is provisional and is excluded from Bank Phases 5
and 6, Closure Phase 1, and current milestone acceptance evidence; Milestone
9.18 owns its product contract and proof. Developer guidance is
[Application Aftermath, External Effects, And Recovery](../../workspaces/worth-query/crates/worth-query/docs/execution/application-aftermath-and-recovery.md).
Bank Phase 5 is closed by the real Docker-backed, separate-process transport
court (2026-08-13). Milestone 9.16 remains open: Bank Phase 6 and Closure Phase
1 still precede the Milestone 9.17 handoff. Milestone 9.16.1.1 is the installed
graph-contract prerequisite for Milestone 9.16.2. After it closes, 9.16.2 is an
additional portable-package prerequisite before 9.17.1 and may proceed beside
the remaining 9.16 phases where the touched boundaries do not overlap.

### Goal

Prove the ordinary Query front door with a legitimate in-memory bank and
person-to-person payment world using real Authentik OIDC, separate asynchronous
user-node processes, capability- and purpose-scoped permissions, field-level
disclosure, conflict-of-interest, governed break-glass, typed schema and
application-query references, double-entry and estate effects,
compare-and-commit, actionable recovery, accepted aftermath/external-effect
publication, and live delivery. The current linear undo/redo implementation may
remain as a provisional experiment, but a future separately governed Query Undo/Redo
Semantics milestone must accept or replace it before it becomes a product
promise.

### Adversarial Constraint

Dynamic personal and business customers, authorized business users, tellers,
auditors, executors, beneficiaries, branch managers, and compliance officers
race transfers, retry after response loss, approve payments, administer a
deceased customer's estate, encounter conflicting personal and employee
authority, request and revoke emergency access, compensate and redo eligible
operations, lose permissions during live delivery, and operate over real TCP
boundaries. Authentication, capability, purpose, disclosure, elevation,
touched-graph admission, invariants, commit, recovery, aftermath, idempotency,
and publication remain distinct typed authorities.

### Specification

The governing milestone spec is
[milestone-9.16.md](./milestone-9.16.md).

### Phase Ownership

- the Runtime Hardening Track uses normal Phases 1–N for generic Query product
  work
- the Bank World Track uses normal Phases 1–N for the bank domain, Authentik
  adapter, asynchronous topology, and complete consumer journeys
- the Closure Track uses normal Phases 1–N for cross-cutting certification and
  permanent prohibitions
- a discovered generic runtime gap adds the next corrective phase or an
  interstitial milestone before implementation; completed milestones and
  ledger rows remain immutable history, bank-specific and adapter-specific
  discoveries remain in the Bank World Track, and independent advanced
  computation belongs to Milestones 9.19 through 9.22

### Must Ship

- schema-derived typed entity, relation, aspect, field, operation, policy, and
  exact money references
- a real Authentik adapter whose `(issuer, subject)` proof authenticates but
  does not authorize
- relationship-scoped customer and employee permissions composed through
  Relational facts, runtime-bridge lowering, Signal decisions, and Query
  admission
- purpose-bound capability grants, field-level disclosure,
  conflict-of-interest, delegation, and governed break-glass with expiry,
  revocation, audit, and mandatory review
- distinct internal-computation and consumer-disclosure authority with
  noninterference evidence for protected facts that affect membership,
  ordering, counts, cursors, summaries, explanations, or live delivery
- exact touched-graph enforcement for reads, mutations, explanations, activity,
  and live delivery
- balanced immutable postings, available-funds and account invariants,
  distinct-actor business and estate approval, and request idempotency
- provider-proven compare-and-commit with honest typed terminal outcomes
- installed application-query identity, Query-owned continuations, explicit
  basis and precondition controls, one-shot/history/live/preview parity, and
  canonical lowering of filters, sorts, traversal, and nested expansion into
  the existing Milestone 9.10 graph-read access-plan and receipt pipeline
- ordinary read, mutation, workflow, history, live, explanation, recovery,
  compensation, and accepted aftermath/publication facades; existing linear
  undo and redo facades remain provisional pending their owning follow-up
  milestone
- a `worth-query-host` conditional-operation contract that binds a domain
  predicate provider to an exact installed application operation and node,
  exposes Query-owned observations rather than raw Signal decisions, binds a
  named host clock and accepts clock observations, and re-enters the same
  installed operation through fresh Query admission
- Signal-owned wake scheduling, eligibility, coalescing, suppression, and
  provenance plus application-runtime reinstall reconstruction from current
  authoritative Relational/domain temporal intent; Workflow Editor and other
  hosts own neither a Signal graph nor a local temporal scheduler
- a temporary Axum boundary, one authoritative bank process, and one independent
  async user-node process per participant
- consumer-real certification, workaround deletion, facade/docs cutover, and
  permanent prohibitions

### Must Preserve

- Milestone 9.15 prepared-state and invariant authority
- authentication distinct from graph-backed authorization
- roles and relationships distinct from scoped capability authority, ordinary
  capability distinct from elevation, and entity visibility distinct from
  field disclosure
- internal access distinct from consumer disclosure and incapable of leaking
  protected facts through query shape or metadata
- Relational ownership of authoritative facts and transaction mechanics
- runtime-bridge ownership of installed lowering and Signal ownership of policy
  evaluation truth, temporal wake scheduling, eligibility, and suppression
- Relational/domain ownership of reconstructible temporal intent, Query
  ownership of exact host-provider and operation binding, and volatile Signal
  wake state remaining derived rather than durable authority
- cert-only replay and exact Foundational values
- Milestone 9.10 selectivity, access-requirement, support-inventory, budget,
  admitted-plan, receipt, and no-N+1 authority
- committed history preserved by compensation; any accepted future redo must
  require fresh execution rather than replay
- accepted tree-based semantic undo/redo is owned by Milestone 9.18 over the
  exact composite runtime-world history established by Milestone 9.17;
  semantic merge, rebase, offline synchronization, multi-parent publication,
  and distributed recovery remain owned by the
  [cross-runtime merging-and-branching roadmap](../cross-runtime/merging-and-branching-roadmap.md)
- transport as adaptation rather than policy or runtime authority

### Acceptance Evidence

The dynamic bank courtroom runs over real OIDC and TCP boundaries, contains no
semantic aspect strings or internal Query imports in consumer code, survives
concurrency, conflict-of-interest, disclosure narrowing, break-glass expiry,
  revocation, retries, response loss, external-effect faults, and provider
  failures, and closes every accepted high- or critical-impact ledger row. A
  host-only conditional courtroom additionally installs providers and clocks
  through `worth-query-host`, proves reinstallable temporal wakes and
  idempotent invocation of the exact installed operation, and contains no
  `worth_signal`, `worth_runtime_bridge`, raw Signal decision, or local
  scheduler residue.
  Existing undo/redo journeys remain provisional regression evidence.

## Milestone 9.16.1: Canonical Graph Obligation And Provider Session Convergence

### Placement

Milestone 9.16 pauses after Runtime Hardening Phase 7.2 while this corrective
milestone is open. Milestones 9.9, 9.10, 9.11, and 9.15 and the completed 9.16
rows retain their recorded historical statuses. After 9.16.1 closes, 9.16
resumes at Runtime Phase 7.3. The corrective scope is authority convergence for
the named graph-obligation, graph-read-planning, and provider-session surfaces.
It does not authorize crate-wide deletion or decomposition of the
`worth-query` monolith.

### Goal

Establish exactly one installed obligation, graph-read requirement, support
inventory, cost, budget, capacity, sealed plan, managed provider session,
lower-owner execution, decision read-set, terminal, and receipt progression for
every ordinary Query read and mutation.

### Adversarial Constraint

A hostile external consumer attempts to construct planning or execution proof,
invoke review as authority, execute raw Relational state, begin authorization
before the provider session, report a selected invariant as executed, combine
cross-runtime, cross-branch, or cross-basis products, and invoke a predecessor
authority after its covered surface cuts over.
A real Bank read and mutation must instead traverse one session-bound path,
invoke the exact Relational/Runtime Bridge/Signal owners, release all resources,
preserve the complete predecessor feature behavior, and retain exact-zero warm
canonical and SHA work as unrelated population and consumer fan-out grow.

### Specification

The governing milestone spec is
[milestone-9.16.1.md](./milestone-9.16.1.md), with closure governed by
[milestone-9.16.1-closure-ledger.md](./milestone-9.16.1-closure-ledger.md).

### Must Ship

- one destination-package installed graph-obligation set per query or operation
- one sealed selection, planning, admission, capacity, and provider-session
  progression
- typed branch affinity carried by every plan, session, read set, proposal,
  invariant, commit, retry, receipt, and publication transition
- one graph-read planning authority consumed by every application-query lane
- session-bound Relational observations, Runtime Bridge correspondence, Signal
  decisions, Query decision facts, proposed state, invariants, and terminals
- honest read-only and mutation branches with exact lifecycle release
- receipts minted only from actual terminal owner evidence
- per-surface parity, atomic cutover, and exact retirement of the monolith
  obligation authority, parallel graph-read authority, manual invariant
  authority, and public proof constructors covered by the milestone; unrelated
  Query features and the ordinary monolith facade remain intact
- hostile consumer, dependency, mutation-sensitive, scale, documentation, and
  residue evidence proving the single path

### Must Preserve

- every recorded historical milestone and phase status
- Relational graph truth and mechanics, Runtime Bridge correspondence/lowering,
  Signal policy evidence, and Query legal composition
- Milestone 9.15 managed lifecycle, provider session, complete read-set,
  proposed-state, and invariant guarantees
- Milestone 9.16 Phase 6 query and warm-path guarantees and Phase 7.1-7.2 typed
  capability, exact-grant, trusted-time, purpose, and currentness guarantees
- cert-only replay and the boundary between ordinary and reconstructive work

### Handoff

Milestone 9.16 resumes at Runtime Phase 7.3 only after the 9.16.1 ledger closes.
Disclosure and every later capability phase must consume the canonical
session-bound decision and may not create another selector, planner, executor,
or receipt path. Every later 9.16 phase carries the admitted branch unchanged;
  per-Relational-branch MVCC and concurrent independent-branch writers begin in
  9.17.1; the Relational service and shared lifecycle correction closes in
  9.17.1.1; final owner services and Signal progress close in 9.17.1.2;
  composite product branches begin in 9.17.2 and become the public Query lane
  in 9.17.3.
  Phase 7.3 also inherits the
  existing Phase 6 query identity, parameter, basis,
continuation, history, preview, live, result, and publication contracts; 9.16.1
changes their graph authority path without deleting or redefining those
features.

## Milestone 9.16.1.1: Installed Graph Contract Integrity Repair

**Status:** Closed on 2026-08-23. Declaration now owns stable application
aspect identity and revision; installation retains one native schema catalog
and exact typed read, touch, emission, external-effect, and aftermath meaning;
execution consumes those installed contracts through Relational-owned read and
validated-mutation evidence; and Host exposes the complete read-only contract
surface. Milestone 9.16.2 may consume this contract without a package- or
NCR-local compatibility representation.

### Placement

Milestone 9.16.1 remains historical. This corrective sub-milestone repairs a
later-discovered mismatch in the current installed application-operation
contract without reopening or amending the prior closure record. Milestone
9.16.2 consumes the repaired installed contract rather than creating a package-
or NCR-local workaround.

### Goal

Make one typed installed schema and operation contract the exact shared
authority for declared graph reads, declared graph touches, native application
aspect contracts, graph-obligation selection, execution lowering, aftermath
inspection, Host adoption, and successor package export.

### Adversarial Constraint

A Host consumer installs same-named fields and aspects across distinct entity
and schema loci, reads one field, declares graph touches, and inspects an
emit-only operation and external-owner aftermath. Query must expose exact typed
scopes and masks, register the installed native contracts in Relational, keep
declared and performed touches distinct, and retain complete typed aftermath
meaning.

### Specification

The governing specification is
[milestone-9.16.1.1.md](./milestone-9.16.1.1.md). Historical closure artifacts
are not reopened or used as current implementation authority.

### Must Ship

- declaration-owned stable application aspect identity and revision
- one installed native application-schema contract catalog
- exact typed entity, native projection, and relation read scopes
- exact typed create, delete, write, link, and unlink declared touch scopes
- graph mutation separated from external-effect emission
- execution registration and performed-evidence comparison from the same
  installed contracts, with no application contract reconstruction
- typed installed reconciliation and correlation-family inspection through
  `worth_query_host::facade::domain`
- focused owner, Host-consumer, compile-boundary, and execution integration
  tests plus required repository checks

### Must Preserve

- the recorded closure and all still-valid guarantees of Milestone 9.16.1
- Foundational aspect/mask/canonical ownership, Relational graph/touch truth,
  Query declaration/installation meaning, and Query execution/session owners
- provider-internal runtime schema in execution
- every unrelated Query application, capability, conditional, aftermath,
  publication, recovery, and cert-only replay behavior
- exact lifecycle and warm-path guarantees

### Handoff

Milestone 9.16.2 Phase 1 extends the stable identity vocabulary delivered here
to the remaining package axes. Its package export and reconstruction phases
carry and freshly readmit the retained native schema, typed operation read and
touch, external-effect, and aftermath contracts. NCR uses only the repaired
declaration and Host facades. No successor may add string touch grammar,
package-local aspect identities, correlation summaries, reconciliation
summaries, or a second operation-contract authority.

## Milestone 9.16.2: Portable Query Packages And Fresh Readmission

### Placement

Milestone 9.16.2 is an additive corrective interstitial discovered by the
Workflow Editor release boundary. It begins after Milestone 9.16.1.1 closes
and may execute beside the remaining Milestone 9.16 Bank Phase 6 and Closure
Phase 1 work, but 9.17.1 begins only after both 9.16 and 9.16.2 close. Prior
milestone and phase statuses remain unchanged.

### Goal

Let an exact Query workflow release cross process, build, and storage
boundaries as complete typed meaning. Ship stable identities, declaration-
minted provenance, complete typed export, bounded untrusted reconstruction,
fresh Query validation against independently expected identity, and one
deterministic neutral release archive with a host-owned signing boundary.

### Adversarial Constraint

A hostile host corrupts, omits, duplicates, reorders, or cross-splices package
records; trusts claimed identity; selects latest by name; forges same-spelled
member references; supplies unsupported versions; and exceeds every decode or
reconstruction budget. The valid package must round-trip to the exact freshly
validated identity, while every hostile form fails before installation or
effects. Records, archive bytes, checksums, signatures, repository output, and
release metadata may mint no Query authority.

### Specification

The governing milestone spec is
[milestone-9.16.2.md](./milestone-9.16.2.md).

Implementation closes through four authority gates: stable identity and
declared provenance; complete typed export; bounded reconstruction with fresh
validation; and deterministic archive plus host release boundary. No fifth
durability phase belongs to this milestone.

### Must Ship

- stable semantic identities for every package-relevant Rust type axis, with
  no portable canonical dependence on `type_name` or module paths
- declaration-minted operation and effect provenance
- a complete public typed package manifest, record, record-view, and record-set
  vocabulary through `worth-query-host::facade::domain`
- bounded logical reconstruction into a non-authoritative candidate followed
  by fresh Query validation and expected-identity comparison
- one narrow `worth-query-package-archive` audience crate with deterministic,
  versioned, bounded encoding and decoding
- a host-owned signing and provenance envelope that contains no private key,
  provider, credential, proof, or runtime state
- an optional immutable exact-archive repository contract that stores
  descriptive bytes but cannot validate, activate, install, or execute them
- deterministic, corruption, compatibility, budget, cross-splice,
  coexistence, facade, docs, release-tool, and residue proof

### Must Preserve

- Query ownership of portable meaning, validation, and semantic digest
- host ownership of Git provenance, signing, registry revision, and activation
- private validation proofs and runtime authority
- the ordinary/reconstructive work boundary
- later Store-roadmap ownership of application-state durability, physical
  fetch, residency, recovery, graph topology, replication, retention,
  distributed recovery, physical capsules, and extension families

### Handoff

Workflow Editor may use the neutral archive as the Query member of an exact
signed release. Milestone 9.17 consumes the identity and fresh-readmission
contracts while keeping owner and composite state in memory. Worth Store later
owns continuous application-state durability and may retain the same archive
bytes without inheriting a PostgreSQL topology or minting Query authority.

## Milestone 9.17: Composite Runtime Branching And Branch-Local MVCC

### Goal

Govern the five-step foundation that establishes one Runtime World-owned
product branch as a reference to an exact composite Relational-plus-Signal
world commit while preserving owner-local component authority and base Bridge
correspondence, replacing the conservative global Relational coordinator with
branch-local MVCC, closing independently borrowable owner services, and
carrying the finished authority through Query. The complete 9.17 world is intentionally
memory-resident; physical durability and restart recovery begin with Worth
Store integration.

### Adversarial Constraint

Two product branches share one exact immutable Signal basis while their
Relational branches diverge; other operations advance Signal alone or both
components. Equal-version branches challenge substitution, a blocked writer
must not stop an unrelated branch, same-head races must preserve exact conflict
posture, and partial preparation must never expose a half-current product world.
Cancellation and failure at every owner/publication transfer must settle or
retain bounded typed state without exposing a half-current product world or
dispatching an owner-local outbox whose composite publication did not succeed.

### Specification

The governing milestone spec is
[milestone-9.17.md](./milestone-9.17.md). Implementation is divided by
authority boundary into:

- [Milestone 9.17.1](./milestone-9.17.1.md): owner component bases and
  Relational branch-local MVCC;
- [Milestone 9.17.1.1](./milestone-9.17.1.1.md): corrective owner-port
  concurrency, settlement recovery, exact retention, lifecycle, facade, and
  evidence closure;
- [Milestone 9.17.1.2](./milestone-9.17.1.2.md): final owner-service completion
  and Signal independent-branch progress;
- [Milestone 9.17.2](./milestone-9.17.2.md): composite runtime-world history
  and coordinated publication; and
- [Milestone 9.17.3](./milestone-9.17.3.md): Query product-branch carriage,
  public facade, and end-to-end certification.

The order is strict. The umbrella closes only after 9.17.3. The separate
post-closure [9.17.4 application API hardening](./milestone-9.17.4.md) now
follows that foundation before 9.18; it does not reopen the certified umbrella.

### Submilestone Sequence

#### Milestone 9.17.1: Exact Owner Bases And Relational Branch-Local MVCC

Status: Closed on 2026-08-28. The executable Relational branch-local MVCC flow
is documented in
`crates/worth-relational/BRANCH_LOCAL_MVCC.md`; the closed owner-basis and
Relational handoff is frozen in
`crates/worth-relational/OWNER_COMPONENT_PORT.md` and
`crates/worth-signal/BRANCH_BASES.md`. Milestone 9.17.1.2 must freeze the final
concrete owner bundles before Runtime World begins.
Runtime World may compose only owner-issued observations, bases, retention
obligations, ports, and typed outcomes.

Foundational first freezes one descriptive vocabulary that separates immutable
target bases from mutable branch references and names exact reference
observations, generations, forks, comparisons, and movements. Relational and
Signal then cut their public branch/basis surfaces over that shared language
while retaining private owner authority. Relational separates immutable commits
from mutable branch references and replaces the global ordinary commit
coordinator with branch-qualified observation, detached transaction state,
exact-head validation, atomic branch-root publication, canonical history, and
obligation-bound retention. Before the MVCC cutover, Relational ships the
Supply Chain certification world: a public-facade causal compiler, named
semantic deltas, independent pure oracle, and Court/Standard/Scale profiles
retained for later merge certification. Closure requires no sibling semantic
crossover, exact shared ancestry, zero-copy fork, touched-region copy-on-write,
controlled independent-branch progress, one-winner same-reference races, exact
owner admission, bounded lifecycle, Signal zero-work basis reuse, and no
composite product claim.

#### Milestone 9.17.1.1: Owner-Port Concurrency And Lifecycle Closure

Status: Closed on 2026-08-29. The corrected Relational, retention, and
lifecycle handoff is frozen in
`crates/worth-relational/OWNER_COMPONENT_PORT.md` and
`crates/worth-signal/BRANCH_BASES.md`; the executable owner flows are
`crates/worth-relational/BRANCH_LOCAL_MVCC.md` and its `branch_local_mvcc`
example, and the `branch_bases` example beside the Signal guide. Milestone
9.17.1 remains closed and historical. Independent post-closure review had found
that the owner handoff still required exclusive Relational runtime borrowing
around preparation and settlement, could lose the only recoverable
performed-settlement state, rejected exact historical Signal retention as stale,
and could leak a dropped Signal external lease. It also found a missing concrete
facade export, unexecuted scheduled and feature-gated evidence, an untested
contention outcome, and owner documentation that did not match the implemented
lifecycle.

This corrective milestone replaced those seams with independently borrowable
Relational preparation, fork, publication, and settlement services; installed
bounded runtime-owned settlement recovery before component movement; retained
exact Signal targets independently of currentness; made ordinary lease drop
terminal; and closed the facade, defensive failure, executable example, CI,
cost-scope, naming, and documentation contracts. Every documented feature-gated
and scheduled proof now executes under its real compiled name through one named
selection authority that fails a lane reaching zero executable tests. Runtime
World 9.17.2 may consume only this corrected owner surface and may not repair
it in the composition layer.

#### Milestone 9.17.1.2: Final Owner Services And Signal Independent Progress

Status: Closed on 2026-09-01 at accepted production revision `95c9aa7455`.
The governing specification and closure record are in
[milestone-9.17.1.2.md](./milestone-9.17.1.2.md). The closed 9.17.1 and
9.17.1.1 meanings remain historical and unchanged.

This milestone completes Relational basis/lifecycle ports and replaces whole-
`SignalRuntime` exclusive access with concrete Signal basis, mutation, and
lifecycle ports. It partitions canonical Signal mutation behind one
execution cell per live Signal branch, preserves one owner graph, makes old
runtime convenience methods delegate to the same cells, and proves unrelated
branches progress synchronously without a global mutex or worker queue. Owner
closure, same-branch races, cancellation, panic, capacity denial, and caller-
capability loss receive explicit typed posture. Runtime World 9.17.2 may
consume only this concrete facade and may not repair the gap with an adapter,
shadow graph, or `Arc<Mutex<SignalRuntime<_>>>`.

#### Milestone 9.17.2: Composite Runtime-World History And Coordinated Publication

Status: Implementation and scoped closure complete on 2026-09-05. The persistent
Astra high gate approved Phases 5-7, including the real owner court, executable
example and closure checks. The [specification](./milestone-9.17.2.md) records the
untouched dependency lint debt and Signal batch-read follow-up. Query carriage
and public facade cutover remain in 9.17.3.

The dedicated `worth-runtime-world` composition owner consumes exact
Relational, Signal, and installed Bridge-correspondence artifacts. It owns
occurrence-identified immutable single-parent composite commits, product
branch references, explicit exact-basis root bootstrap, managed observations,
deduplicated exact component pins,
bounded history, and one typed preflight/reserve/owner-work/compare-and-publish
progression. The final product-head CAS is the composition-currentness
linearization point. Owner effects that occur without product movement remain
in bounded `ProductUnpublishedOwnerEffects` recovery records and are never
misnamed rollback or a commit. Closure requires no mixed product observation,
no global composition or Signal lock, no silent ancestry pruning, and honest
cost scopes. It does not claim public Query cutover or physical persistence.
The owner remains generic over the frozen
`SignalOwnerServicePorts<D, I, E, Ctx, T>` contract, admits Bridge meaning only
from a real installed correspondence witness, and honors Relational's distinct
fresh fork-source-token protocol. Branch creation is the explicit two-by-two
reuse/fork matrix; component change is a separate publication. The current
Query `BridgeOwnedSignalRuntime` graph-access topology is cut over in 9.17.3,
without a second Signal graph or an erased compatibility lane.

#### Milestone 9.17.3: Query Product-Branch Carriage, Facade, And Certification

Status: Closed on 2026-09-10. Phases 1-6 and cumulative certification are
complete. The ordinary and advanced examples and the bounded and scheduled
model/cost courts run through the public Query/Bridge/Signal/Runtime World
composition root. The scoped strict lint, line-cap, facade, boundary, and
generated-context gates pass; the existing unrelated Signal unit-test compile
debt remains outside this milestone's dirty set.

Query carries the exact admitted composite basis and performed successor
through every plan/session/read-set/proposal/invariant/effect/terminal/receipt/
history/live/inspection boundary, publishes the public product-branch
workflow, deletes the Relational-only and ambient-Signal lane, and runs the
cumulative hostile court through the real composition root. It permits
existing-outbox dispatch only from performed composite publication in the live
runtime. Its closure closes the umbrella. The subsequently specified 9.17.4
hardens consumer adoption before 9.18 extends the ordinary application API.

The [9.17.3 specification](./milestone-9.17.3.md) requires the one-sealed-graph
Signal-owned conditional service and complete installation/re-entry lifecycle
before broad carriage migration. Phase 1 establishes shared host/Query/Bridge
operation entry, Bridge-only conditional-service issuance, bounded source-affine
evaluation reuse, and authoritative post-seal definition publication. The World
Signal bundle retains its three existing ports. Query strengthens its existing
provider-session affinity without adding a second phase protocol. Bridge observations and Signal
derived reuse bind the exact source world even when distinct product branches
share a Signal basis. A surviving loser outbox row stays ineligible for its
original occurrence even inside a later performed
descendant or fresh adoption after settlement; both dispatch and redispatch bind
the original operation's exact performed publication attempt. Partial terminals
cannot construct committed receipts or committed recovery bindings.
The six phases close on real public-root scenario evidence, bounded resource and
cost contracts, and deletion of each migrated legacy authority path. 9.18 adds
fresh correction meaning and admission to this same publication/terminal path;
it does not move history ownership or introduce persistence.

### Must Ship

- 9.17.1 first ships the Supply Chain causal certification world, independent
  semantic oracle, reusable merge-ready deltas, and structural-sharing
  observations; then ships a shared Foundational immutable-basis/mutable-
  reference grammar;
  owner-issued Relational and Signal branch bases that remain distinct and
  non-substitutable; a Relational split between immutable commits and mutable
  references; and Relational-owned branch-local versions, repeatable snapshots,
  detached transactions, exact-head conflicts, atomic branch roots, retention,
  prepared candidates, performed owner commits, and concurrent independent-
  branch progress without an ordinary global lock
- 9.17.1.1 then ships independently borrowable Relational preparation, fork,
  publication, and settlement services; pre-effect bounded settlement
  recovery; exact non-current Signal retention; terminal external-lease drop;
  the complete concrete publication authority facade; typed defensive
  publication failure; and real contention, cancellation-feature, scheduled-
  scale, executable-example, documentation, and cost-scope evidence
- 9.17.1.2 then completes the Relational service bundle and ships weak Signal
  ports, canonical per-branch cells, independent progress, and terminal
  closure/cancellation/panic/capacity evidence
- 9.17.2 ships a dedicated Runtime World composition owner with exact Bridge
  correspondence, explicit exact-basis root bootstrap,
  occurrence-identified immutable single-parent commits,
  product branch references and managed observations, deduplicated exact
  component pins, bounded ancestry-preserving history, explicit reuse/fork
  posture, coordinated compare-and-publish, and retained product-unpublished
  owner effects with no mixed or half-current product world
- 9.17.3 carries the admitted product branch and component bases through every
  Query plan, session, authority, effect, proposal, invariant, publication,
  terminal, receipt, history, live, inspection, and aftermath phase; cuts over
  the public facade; gates the existing
  outbox on performed composite publication; and deletes the Relational-only/
  ambient-Signal lane; workflow-visible dispatch aftermath re-enters as a new
  composite publication rather than sideband Relational mutation
- all operational transitions consume compiler-visible, private-minted
  predecessor types using concrete `worth-proof` carriers specialized to
  owner-sealed markers; governed facades expose no caller-selected generic
  authority parameter; and `worth-foundational` supplies portable canonical
  and boundary vocabulary without becoming operational authority

### Must Preserve

- the ordinary typed and authorized application boundary established by 9.16
- the branch-affine provider-session contract established by 9.16.1
- Relational and Signal ownership of their respective component branch truth
- base Runtime Bridge ownership of installed semantic correspondence and the
  dedicated Runtime World owner's composition-history/currentness authority,
  without either absorbing component truth
- Query carriage without Query-owned component or composite history authority
- explicit memory residency through 9.21 without a temporary physical-runtime
  abstraction or hidden persistence promise
- cross-runtime ownership of semantic merge, rebase, multi-parent publication,
  offline synchronization, and distributed recovery

### Acceptance Evidence

Independent component and composite-history probes prove a causally installed
Supply Chain baseline against a non-production semantic oracle, no branch
crossover, shared immutable history without duplicated truth, touched-region copy-on-write,
shared immutable Signal basis reuse, Relational-only and Signal-only
advancement, combined publication, immutable-commit/mutable-reference
separation, distinct equal-version authority, cross-branch progress, same-head
conflict, atomic branch-root visibility, exact ancestry and retention, no half
publication, and exact lifecycle cleanup.
The final 9.17.3 court proves the same facts through the real Query composition
root, public facade, failed-composite outbox ineligibility, default/admitted
parallel lanes, compiler denials, exact counters, executable docs, and exact-
zero legacy residue.
The 9.17.1.1 correction court first proves simultaneous Relational preparation
and settlement are expressible without exclusive runtime borrowing, performed
settlement survives capability loss, a historical exact Signal basis remains
retainable, lease drop releases its obligation, and every documented feature or
scheduled proof executes under its real compiled name.

The 9.17.1.2 court proves complete owner bundles, deterministic unrelated
Signal progress, one-winner same-branch races, one canonical graph, and
lifecycle-total
close/cancellation/panic/capacity behavior. The exact executable targets and
profiles are governed by the individual submilestone specifications;
[test-requirements.md](./test-requirements.md) governs the shared evidence
quality rules rather than serving as a suite-name registry.

## Milestone 9.17.4: Application API Hardening

### Goal

Make typed application intent the ordinary Query experience over the certified
9.17.3 runtime. Query owns reusable binding and execution; consumers supply
domain meaning and explicitly registered handlers rather than copying the
select/resolve/admit/execute/publish pipeline. No legacy compatibility layer
or competing ordinary application language remains after cutover.

### Specification And Scope

The governing specification is
[milestone-9.17.4.md](./milestone-9.17.4.md). It freezes the target call shapes,
current-to-destination replacement table, owner boundaries, directory skeleton,
documentation obligations, and six phases requiring certification:

1. Pure-value bindings and explicit cross-crate contribution composition.
2. Requests and current, retained, historical, paged, and live reads.
3. Installed operation execution, primary candidate/domain integrity, exact
   recovery, and complete Bank adoption.
4. Admitted runtime-cardinality construction, geometry-shaped integrity and
   bounded synchronous output-demand settlement.
5. Existing `worth-ui-query-binding` and `crates/worth-server` consumer migration.
6. Remaining audience closure and cumulative certification.

The Bank application and real HTTP/user-node journeys complete before the
geometry-shaped construction phase. The latter proves the public primary
foundation without implementing the proprietary kernel. House M0 consumes its
bounded synchronous group-demand contract; M3 extends it to transitive
regeneration and off-thread numerical completion. Native Save/Open remains
house M7E Query/Store work. No placeholder strategy supplies missing completion.
Advanced access and correlated execution remain 9.19 and 9.20.

Bank's provisional linear undo/redo commands and transport consumers retire
in Phase 3 while accepted aftermath and recovery remain. 9.18 supplies the
accepted tree-correction product. Capability/elevation admission is explicitly
bound into the new entry; publication retention is an opt-in pre-effect lease,
never an implicit cost attached to every descriptive commit receipt. Bank's
broken bindings are repaired directly into the destination API, without a
preliminary restoration of superseded public contracts.

### Acceptance And Handoff

Use Bank accounting, permission revocation during retained/live/paged reads,
stale mutations, idempotency, losing-outbox publication, exact settlement,
process transport, sibling progress, and cleanup to convict a facade which
caches or reconstructs authority. A separate pure-value/contribution consumer
proves lawful binding and actual primary cyclic construction, mandatory
candidate-neighborhood invariants, and pre-allocation resource admission.
1/10/100 geometry-shaped groups expose per-entity sessions or publications.

Generation removes duplicate wiring, not explicit semantic identity or
registration. Retained data does not retain permission. Prepared attempts
carry one exact selection; mutation outcomes retain all actual effects and
recovery custody. Preserve existing World/Signal ownership and reuse valid
certification evidence and compiled binaries. Phase certification, commit, and
push precede the next phase; the old CI pipeline is not reinstated.

9.18 consumes this hardened request/handler/history/aftermath contract.
9.19-9.22 extend it without another ordinary consumer language. The certified
9.17 branching umbrella remains a completed predecessor.

## Milestone 9.18: Tree-Based Semantic Undo And Redo

### Goal

Replace the provisional Milestone 9.16 linear undo/redo experiment with an
accepted tree-based product over the composite history completed by Milestone
9.17.3 under the Milestone 9.17 umbrella, through the application API hardened
in 9.17.4. Every
reversal or reapplication selects an exact source world commit and target
product branch/head, re-enters current authority and policy, coordinates the
required component plans, and publishes a new composite commit without
rewriting history.

### Adversarial Constraint

Disjoint and conflicting descendants, equal-version foreign component branches,
stale composite heads, incompatible Signal definitions, expired authority,
replacement inputs, irreversible external effects, partial component
preparation, and concurrent corrections challenge the product. An implicit
stack, copied receipt, cached result, ambient Signal head, or replayed authority
must open no door.

### Specification

The governing milestone spec is
[milestone-9.18.md](./milestone-9.18.md).

### Must Ship

- exact committed-occurrence, source composite commit, component bases, and
  target product-branch correction identity
- installed recorded-inverse, compensation, reconciliation, reapplication, and
  irreversible contracts
- explicit per-component retain, inverse, compensate, reconcile/rebuild,
  reapply, or deny posture
- typed applicability over intervening composite/component history, current
  definitions, policy, authority, conflicts, and invariants
- fresh reversal and reapplication through ordinary owner-local execution and
  Runtime Bridge coordinated compare-and-publish
- preserved alternative descendants rather than a mutable undo/redo stack
- public branch-history/aftermath facade, executable documentation, complete
  provisional-lane cutover, and independent hostile certification

### Must Preserve

- the completed Milestone 9.17 umbrella: 9.17.1 component authority and
  branch-local MVCC, 9.17.2 composite correspondence/publication/history, and
  9.17.3 Query carriage and public product-branch authority, followed by
  9.17.4 application bindings, request execution, and consumer cutover
- Milestone 9.16 aftermath, retained-truth, external-effect, recovery, and
  publication contracts
- original history, fresh authorization, and cert-only replay
- cross-runtime ownership of merge, rebase, offline synchronization,
  multi-parent publication, and distributed recovery

### Acceptance Evidence

An independent history oracle proves every successful correction is a new
commit, stale or hostile attempts apply nothing, divergence remains explicit,
compensation does not claim reversal, alternatives remain navigable, and
ordinary commits pay exact-zero correction work.

## Milestone 9.19: Managed Advanced Access And Verified Footprints

### Goal

Add installed-query-bound managed search and access products, complete positive
and negative membership, exact refinement, and verified realized footprints
through the existing Milestone 9.10 and 9.16 authority paths.

### Specification

The governing milestone spec is
[milestone-9.19.md](./milestone-9.19.md).

### Must Ship

- provider-backed access-product lifecycle with honest memory and work evidence
- typed search and spatial strategies bound to the existing graph-read plan
- complete membership across insertion, deletion, motion, maintenance,
  rebuild, eviction, authorization, and disclosure change
- verified footprints that narrow but never widen declared authority
- bank/compliance and geometry public-facade adoption, executable docs,
  independent full-scan/resource courts, and workaround deletion

### Acceptance Evidence

Independent oracles expose incomplete candidates, protected-fact leaks,
underreported resource work, stale lifecycle use, and lying footprints while
alternate providers converge on canonical semantics.

## Milestone 9.20: Correlated Paths And Conflict-Proof Set Execution

### Goal

Install typed correlated heterogeneous path programs and execute large batches
through verified conflict partitions and real provider set operations.

### Specification

The governing milestone spec is
[milestone-9.20.md](./milestone-9.20.md).

### Must Ship

- typed path steps, bindings, bounds, identity, and admitted provider lowering
- exact-zero covered per-binding and per-result lookup work
- domain-installed conflict meaning and verified canonical partitions
- capability-safe provider set execution with separate planning/execution cost
- chip/netlist, geometry, and bank reference adoption and hostile parity/slope
  certification

### Acceptance Evidence

An independent interpreter and partition oracle prove semantic parity,
boundedness, negative dependencies, complete conflict-free coverage, no hidden
scalar loops, no quadratic all-pairs admission, and no unauthorized member
processing.

## Milestone 9.21: Governed Decision Attachments And Summaries

### Goal

Attach domain decisions to exact executions under classification, disclosure,
retention, deletion, and occurrence governance, then expose rebuildable
incremental summaries without promoting derived evidence to authority.

### Specification

The governing milestone spec is
[milestone-9.21.md](./milestone-9.21.md).

### Must Ship

- typed attachment schemas and exact execution/occurrence binding
- nested classification, redaction, retention, deletion, capability, purpose,
  disclosure, elevation, and review governance
- dependency- and invalidation-backed incremental summaries
- bank/compliance and research facade/docs adoption with independent privacy,
  deletion, rebuild, stale-summary, and authority-spoofing courts

### Acceptance Evidence

Restricted content cannot escape through a permissive container, summaries
update and rebuild exactly, deletion cannot be defeated by derived copies, and
no attachment or summary opens admission, commit, approval, branch, correction,
or recovery authority.

## Milestone 9.22: Occurrence-Safe Stage And Subartifact Reuse

### Goal

Reuse eligible stages and subartifacts while preserving installed semantic
equivalence, dependencies, provider and governance posture, lifecycle,
consumer purpose, and distinct production/acquisition and independent-
certification occurrences.

### Specification

The governing milestone spec is
[milestone-9.22.md](./milestone-9.22.md).

### Must Ship

- installed equivalence, reproducibility, dependency, provider, governance,
  and consumer-purpose contracts
- framework-owned shared executions and move-only consumer leases
- exact invalidation, maintenance, stale, eviction, rebuild, and disposal
  lifecycle
- research reference adoption, public facade/docs, bounded resource evidence,
  alternate-provider parity, and occurrence-sensitive hostile certification

### Acceptance Evidence

Eligible consumers share exactly once, ineligible occurrences never share,
stale or incompatible artifacts deny before use, leases dispose exactly,
resources remain bounded, and cache hits cannot manufacture production or
certification occurrences.

## Store-Gated Implementation Moved

The former Query Milestones 10 through 12 are now Store Milestones 9 through 12
in the [Worth Store Runtime And Integration Roadmap](../worth-store/runtime-integration-roadmap.md).
They were removed here because keeping duplicate milestone bodies would create
two authorities for dependency order, closeout evidence, and integration DX.
Their Query-facing contract obligations remain governed by the handoff above.

## Milestone 13: Runtime-Backed Generic And Domain Query Certification

### Goal

Prove the completed runtime-backed Query framework and export semantic parity
oracles that Store integration can reuse against physical boundaries.

### Adversarial Constraint

Every admitted runtime-backed capability must survive hostile replay, basis
variation, policy variation, live maintenance, and domain-specific workloads
without changing canonical query meaning, authority, result shape, or
certification identity. A provider may not pass by weakening, widening, or
reinterpreting the declared semantics.

### Why This Milestone Exists

`worth-query` is the main consumer-facing semantic surface for the stack.
Store integration needs a trustworthy oracle, not a second interpretation of
what Query plans, results, histories, live changes, policies, artifacts, or
delivery shapes mean.

### Must Ship

- a dedicated `worth-query` certification matrix or
  `test-requirements.md` equivalent if one does not yet exist
- generic certification suites covering at minimum:
  - canonical query normalization parity
  - schema-aware validation rejection
  - snapshot-backed execution parity
  - collection/pagination stability
  - live-promotion equivalence
  - region-scoped live narrowing and stream-contract parity
  - preview-session basis and promotion parity
  - frontier-aware planning and deterministic parallel parity
  - structural correspondence and historical materialization-path parity
  - query-authored workflow/mutation lowering parity
  - unified facade/configuration boundary parity
  - live + policy masking parity
  - historical/diff parity
  - historical + diff + result-shape parity
  - lineage/correspondence query parity
  - lineage + correspondence + branch-scoped comparison parity
  - policy-masking and tenant-scope correctness
  - tenant schema variation + validation + delivery-shape parity
- domain certification suites covering at minimum:
  - geometry/topology neighborhood query truth
  - AI/speculative-branch comparison reads
  - geometry and workflow branch-preview/merge reads plus query-authored merge
    lowering
  - web collection/detail/live workflow reads
  - chip/netlist cone and historical diff reads
- provider-independent semantic parity oracles for canonical plans and result
  shapes, basis-exact reads, policy masking, live evolution, saved-artifact
  semantic freeze, continuation meaning, and blob-reference projection meaning
- provider-independent bound-projection oracles for exact installed-operation
  resolution, installation, single-root entry, graph participation, atomic
  multi-domain and admitted multi-graph binding, workflow progression and
  trace, ordinary re-execution, cert-only replay, reversal/compensation
  posture, publication, lineage, promotion and persistent-naming admission,
  consumer support, execution, consumption,
  native access, dependency impact, shared execution and lease lifecycle,
  compatibility, invalidation, collection window/patch delivery, denial, and
  counter parity established by Milestone 9.14
- provider-independent domain-computation oracles for managed artifacts,
  semantic/occurrence/certification identity separation, substitution policy,
  exact and non-bitwise reproducibility classes, bulk/chunk native access,
  resource admission, cancellation/yield/resume, provider sessions, complete
  decision read-sets, proposed post-state, invariant execution,
  compare-and-commit, authenticated touched-graph authorization,
  access-product completeness and membership, realized
  footprints, correlated paths, conflict partitions, structural counters,
  decision attachments, and reuse lifecycle established by Milestones 9.15
  through 9.22
- machine-checkable artifact bundles for plans, results, diagnostics, and live
  evolution that Store Milestones 9 through 13 and 19 can execute unchanged
  against the physical provider

### Must Preserve

- certification must prove existing capability boundaries rather than smuggling
  in missing features
- provider-independent oracles may observe provider receipts but may not encode
  a Store-specific shortcut or tolerate semantic widening
- query artifacts remain canonical and typed across original execution and
  replay/certification re-run
- beta support claims must not outrun admitted-family, fallback-honesty, and
  certification-matrix proof

### Complexity / Proof Obligations

- name certification bundle construction and replay-verification contracts
- expose exact counters for certification scenarios executed, parity checks
  performed, and capability rows covered from the Vision Coverage Appendix
- prove complete runtime-backed appendix coverage rather than milestone-local
  spot checks only
- prove the exported oracle can be driven by a second test provider without
  embedding runtime-instance assumptions in expected results

### Allowed Debt

- physical persistence, restart, portability, durable continuation, blob
  transport, and Store pushdown evidence remain Store roadmap work
- no debt is allowed in Query semantic coverage or machine-checkable oracle
  artifacts for capabilities claimed here

### Sequencing Notes

This closes after Milestone 9.22 because Store integration must inherit a
certified public surface, non-detachable downstream capability, execution-grade
domain-computation substrate, and semantic oracle. It does not wait for Store.

### Parallelization Notes

Provider-harness extraction may proceed alongside late runtime-backed
milestones once their public contracts stabilize. Joined physical-provider
execution belongs to the Store roadmap.

### Store Dependency

- this milestone is not blocked on Store
- Store Milestones 9 through 13 consume its provider-independent oracles
- Store Milestone 19 owns joined runtime/physical production certification

### Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- the `Query Certification Matrix Sufficiency Test` in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/worth-query/test-requirements.md)
  passes with canonical machine-checkable artifacts
- the `Admitted Query Family Boundary Test`, `Fallback Non-Leakage / No Silent
  Widening Test`, `Cross-Feature Composition Matrix Test`, `Reference
  Semantics Test`, `Saved Artifact Semantic Freeze Test`, `Schema Evolution
  Compatibility Test`, `Diagnostic Sufficiency Test`, and `Beta Support Matrix
  Enforcement Test` in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/worth-query/test-requirements.md)
  pass for the surfaces `worth-query` claims as shipped or beta-supported

- every shipped query capability has at least one hostile certification path
- machine-checkable certification bundles can localize planning, execution,
  policy, live-maintenance, and history failures
- a deliberately widening or basis-drifting test provider is rejected before
  Store integration is allowed to consume the oracle

## Per-Milestone Format

Every milestone in this roadmap uses the same shape:

- `Goal`
- `Adversarial Constraint`
- `Why This Milestone Exists`
- `Must Ship`
  interpreted as:
  - `Surface Primitives`
  - `Semantic Guarantees`
  - `Proof Obligations`
  - `Store Handoff`
- `Must Preserve`
- `Complexity / Proof Obligations`
- `Allowed Debt`
- `Sequencing Notes`
- `Parallelization Notes`
- `Store Dependency`
- `Acceptance Evidence`

The store-dependency section records the exact handoff consumed by the Store
roadmap. It does not make physical implementation part of local Query closure.

## Completion Standard

`worth-query` is roadmap-complete only when:

- typed query expression, validation, planning, execution, collection
  semantics, live promotion, region-scoped live narrowing, preview-session
  query contexts, frontier-aware planning, structural correspondence,
  query-authored workflow/mutation lowering, unified facade/configuration,
  authoritative mutation evidence, historical reads, lineage traversal,
  composition, policy-aware narrowing, temporal query basis semantics, async
  resource query families, mixed truth/time/async delivery semantics, and
  temporal/async query certification are all shipped
- Foundational-native scalar and struct meaning survives Query authoring,
  predicate/schema admission, execution, materialization, projection
  consumption, typed refinement, receipt identity, and inspection without a
  competing Query-owned value algebra or semantic encoder
- Query's proof portfolio is authority-owned and naturally selectable through
  Cargo packages and responsibility-named targets: local authority lanes omit
  later and cold work, full certification composes real public journeys,
  retained compiler denials stay selective, and no proof inventory, runner,
  source audit, shared fixture platform, or test-of-test topology exists
- downstream projection consumers resolve installed domain operations and
  receive one non-detachable runtime-affine capability with Query-owned
  consumer support, native access, dependency impact, equivalent-work sharing
  and leases, compatibility, invalidation, collection windows and patches,
  lifecycle, opaque operational identity, and exact cost evidence rather than
  authority ingredients they must assemble correctly
- installed domain computations carry typed managed artifacts; execute through
  resource-bounded cancellable provider sessions; preserve distinct semantic,
  occurrence, and certification identity with explicit substitution policy;
  compare exact and non-bitwise results only through installed reproducibility
  classes; retain complete decision read-sets, membership coverage, and
  verified realized footprints; validate exact proposed post-state through real
  domain invariant execution; commit, abort, or reconcile through honest
  transaction outcomes; and scale through managed access products, correlated
  paths, and conflict-proof set-oriented execution without domain semantics
  entering Query
- the runtime-backed framework closes at Milestone 13 with provider-independent
  semantic parity oracles and machine-checkable generic/domain evidence
- Store-native graph execution, durable query extensions, distributed restart,
  blob delivery, and joined parity remain explicit handoffs to Store Milestones
  9 through 13 and 19; Milestone 9.16.1.1 first closes installed graph-contract
  integrity, then Milestone 9.16.2 independently closes portable package
  identity, reconstruction, and release carriage

## Vision Coverage Appendix

This appendix is the traceability layer for the rule in
`worth_query_vision.md`: if a capability is named in the vision and not yet
built, it is roadmap work. Every named capability must have a roadmap home,
canonical artifact boundary, proof path, and certification path, even when the
answer is "store-gated" or "shared with another subsystem."

| Vision Capability | Roadmap Home | Canonical Artifact(s) | Acceptance Proof | Certification Path |
| --- | --- | --- | --- | --- |
| Typed composable query expressions | Milestone 1 | Canonical query AST, query digest, facade-normalized query artifact | Equivalent builders normalize identically | Milestone 13 query normalization parity |
| Aspect-aware projection | Milestones 1-2 | Projection descriptors, validated aspect masks | Legal projection lowers deterministically | Milestone 13 normalization + execution parity |
| Filter and predicate expressions | Milestone 2 | Typed predicate nodes | Schema-invalid predicates reject early | Milestone 13 validation rejection |
| Workflow-aware predicates | Milestone 2 | Workflow predicate nodes in canonical query artifact | Workflow predicates validate and normalize canonically | Milestone 13 validation rejection + web workflow domain suite |
| Ordering | Milestones 2 and 4 | Ordering descriptors in plan/result metadata | Ordered reads preserve declared basis | Milestone 13 collection/pagination stability |
| Pagination and opaque cursors | Milestone 4; durable continuation in Store Milestone 11 | Cursor descriptors, page metadata, durable cursors | Page advancement is stable for one basis | Milestone 13 runtime parity; Store Milestone 19 joined parity |
| Bounded result sets | Milestone 4 | Bound/limit descriptors in plan metadata | Truncation stays explicit and basis-honest | Milestone 13 collection stability |
| Bounded relational materialization | Milestone 4 | Relation materialization descriptors, traversal bounds | Eager materialization stays within declared scope | Milestone 13 execution parity + geometry/chip domain suites |
| Subgraph-scoped queries | Milestone 4 | Scope/traversal boundary descriptors | Traversal breadth remains bounded and explainable | Milestone 13 geometry/chip domain suites |
| Relation traversal expressions | Milestones 1, 2, and 4 | Traversal nodes, validated relation-edge constraints | Illegal traversals reject; legal traversals stay bounded | Milestone 13 validation rejection + domain suites |
| Graph read access planning and declarative index admission | Milestone 9.10 | Operation-resolution descriptors, access-shape descriptors, access requirement sets, intrinsic/supported budget estimates, access admission envelopes, graph access receipts | Covered graph reads execute through admitted access postures, with no hidden N+1 traversal and no unbounded background indexing | Milestone 9.10 no-N+1, memory-budget, and access-plan replay certification + Milestone 13 execution parity |
| Declarative downstream basis authority and consumer DX | Milestone 9.11 | Consumed projection authority artifacts, declarative authority contracts, typed authority denials, DX transcripts, consumer adoption receipts | Downstream runtimes consume one Query-owned basis/projection authority product and cannot reconstruct or recombine authority from receipts, facts, labels, or digests | Milestone 9.11 cross-basis, facade-DX, consumer-residue, and Worth UI adoption certification + Milestone 13 projection/domain parity |
| Query public authority surface cutover | Milestone 9.12 | Sealed identity handles, scoped basis and follow-on proofs, declarative admission handoffs, contracted facade snapshots, prohibition and residue artifacts | Ordinary consumers cannot mint authority from representation, skip Query lifecycles, invoke raw admission machinery, assert operational posture, or import certification internals as product APIs | Milestone 9.12 collision, phase-skipping, compile-fail, facade-snapshot, prohibition, residue, sabotage, and reference-consumer certification + Milestone 13 authority-boundary parity |
| Declarative Query experience and phase-surface cutover | Milestone 9.13 | Capability declarations, admitted-query handles, managed live resources, typed outcomes and next-action stops, DX transcripts, contracted facade and residue artifacts | Ordinary consumers describe desired outcomes while Query exclusively owns canonicalization, binding, validation, admission, planning, lowering, execution routing, lifecycle, receipt assembly, and derived diagnostics | Milestone 9.13 ordinary/internal parity, compile-fail, facade-snapshot, prohibition, residue, lifecycle, bounded-work, and reference-consumer certification + Milestone 13 product-surface parity |
| Runtime-installed domain capability authority | Milestone 9.13 add-on Phases 13-20 | Canonical domain packages, admitted package artifacts, runtime installation registry, runtime-affine installed handles, package-compiled operation/invariant/obligation indexes, installation and execution receipts | Equivalent packages install identically; conflicting packages fail atomically; only the installing runtime can mint usable handles; registered operations and contributions resolve without raw strings, manual registries, or semantic adapters | Milestone 9.13 package convergence, installation atomicity, runtime-affinity, derived-index rebuild, facade/residue/sabotage, bounded-lookup, and reference-consumer certification + Milestone 13 domain parity |
| Foundational-native aspect value authority and consumer DX | Milestone 9.13 add-on Phases 21-30 | Portable non-authoritative patch/state candidates, Foundational contract readmission, native entity/relation transaction intents, authoritative patch planning and publication, durable readmission, exact scalar and struct values, contract-derived operator capabilities, proof-bearing mutation/result/consumption artifacts, native retained rows or honest internal markers, typed refinement denials, canonical value identity basis | Every native patch operation, scalar family, and representative struct round-trips through portable boundaries, Relational planning/merge/commit, checkpoint/replay, Query authoring, validation, execution, materialization, projection, refinement, receipts, and inspection; incompatible operators and weaker-proof promotion fail before work; no substrate owns a competing value, patch, state, or semantic encoder | Milestone 9.13 patch/state readmission, entity/relation transaction parity, restart/replay, native-family/struct matrix, canonical identity parity, facade/compile-fail/prohibition/residue/sabotage, bounded-work, and reference-consumer deletion certification + Milestone 13 schema, execution, projection, and domain parity |
| Livable Query iteration foundation | Milestone 9.13.1 | Selected bulk compile-fail group, responsibility-named ordinary targets, cold certification leaf, permanent declaration and installation packages, consumer-owned Worth UI adoption tests | Load-bearing compiler evidence remains selective; the manual library-test aggregator, repeated reconstruction, historical/privacy/meta fixtures, and Query-to-Worth-UI coupling are removed; declaration and installation edits omit later and cold authorities; each slice inventories only the boundary it immediately changes | Milestone 9.13.1 direct compiler and cold-certification runs, target-ownership migration review, package convergence/rebuild proof, declaration/installation owner commands, Worth UI binding run, and one before/after observation per slice |
| Query authority crate decomposition | Milestone 9.13.2 | Completed production authority graph, admission/execution/publication packages, retargeted certification, narrow audience facades, authority-local tests, enforced dependency direction | Remaining authority work compiles and tests without later or cold Query authorities; no product composition root, facade, shared support crate, or compatibility re-export reconstructs, duplicates, or bypasses a covered authority | Milestone 9.13.2 boundary-check, consumer transcript, authority-local test, compiler-denial, certification-retargeting, per-surface parity/cutover, exact predecessor-retirement, and full downstream feature-preservation proof |
| Installed operation semantics, semantic aspect correspondence, conditional Signal authority, and bound downstream authority | Milestone 9.14 | Complete installed operation semantic closures, one installed operating-world root, typed borrowed operation-family facades, sealed graph-participation adapters, atomic multi-domain and admitted multi-graph operation capabilities, portable Query-authored semantic truth dependencies and conditional-node declarations, aspect-precise Relational publication, installed truth-to-Signal aspect correspondences, pair-bound runtime-bridge lowerings, installed Signal node contracts, Signal-minted decision evidence, Query re-entry receipts, installed workflow DAGs, Query-minted run/stage traces, ordinary re-execution and cert-only replay results, typed reversal/compensation posture, derived-publication receipts, trace-bound lineage and promotion evidence, runtime-affine bound projections, Query-minted consumer support contracts, proof-bearing execution/consumption states, declaration-indexed native access keys, pair-bound compatibility witnesses, compiled dependency-impact closures, shared execution owners and consumer leases, capability-bound invalidation deltas, bound collection/window capabilities, query-shaped patches, managed lifecycle states, opaque operational identities, and exact counter snapshots | Stable domain operations, semantic truth dependencies, aspect publication, installed aspect correspondence, entry authority, graph participation, workflow progression, conditional authoring, lowering, Signal decisions, Query re-entry, replay, reversal, publication, promotion, and lineage cannot be reconstructed locally and independently valid authority ingredients cannot be recombined; Query owns portable semantic dependency and conditional intent, Relational owns authoritative aspect-change meaning, the runtime bridge owns exact installed correspondence and lowering, and Signal owns local aspect slots and evaluation truth; node evaluation and effect conditions remain distinct; skipped, deferred, and reverted-clean outcomes cannot masquerade as new computed delivery; one logical graph is the default; separate graphs bind atomically only with shared commit authority and otherwise expose compensation; ordinary downstream work consumes typed publications while replay remains cert-only; native access remains Foundational-exact and bounded; Query alone binds operation authority, advances workflow, mints traces and publications, admits replay, binds aftermath and lineage, compiles impact, admits sharing and compatibility/lifecycle, mints leases and support, states invalidation meaning, and preserves collection identity, ordering, cursors, continuation, and patch semantics; reporting representations and derived indexes have zero operational power | Milestone 9.14 operation-definition, single-root, graph-adapter, atomic/compensated multi-graph, semantic-dependency canonicality, Relational publication precision, aspect-correspondence/slot-capacity admission, conditional-authoring canonicality, bridge-lowering admission, Signal-decision authority, conditional outcome/counter matrix, Query re-entry, workflow-graph convergence/conflict, stage-progression, publication-consumption, replay-fence/equivalence/divergence, reversal-posture, lineage/promotion/persistent-naming, construction, support-spoofing, mix-and-match, native parity, dependency-impact replay, sharing-equivalence/lease lifecycle, collision, invalidation replay, collection-window identity, shared patch/fresh-execution parity, exact-counter, facade, residue, sabotage, reusable-certification-kit, and reference-consumer certification + Milestone 13 provider-independent bound-projection parity |
| Managed pre-commit domain computation | Milestone 9.15 | Installed artifact/counter/decision/invariant contracts, occurrence and reproducibility identity, move-only managed artifacts, resource reservations, managed runs and continuations, sealed provider plans and sessions, basis-complete decision read-sets, provisional post-state attempts, and real invariant-execution results | Large products cross stages without blobs; actual provider work is bounded, cancellable, yieldable, resumable in memory, and backpressured; proposals remain isolated; selected obligations cannot masquerade as executed invariants; prepared work has no commit power | Milestone 9.15 artifact/occurrence, native-memory, resource-saturation, lifecycle, provider-session, read-set, proposed-state, invariant-execution, facade, residue, and hostile-provider certification |
| Authenticated ordinary Query front door | Milestone 9.16 | Schema-derived typed references, installed application queries and Query-owned continuations, Authentik principal proof, capability/purpose/disclosure/conflict/elevation authority, Milestone 9.10-bound filter/sort/traversal access plans, touched-graph admission, double-entry and estate effects, provider compare-and-commit, actionable recovery, accepted aftermath/external-effect publication, ordinary read/mutation/workflow/history/live facades, host-installed conditional providers and named clocks, reconstructible temporal wakes, cross-process HTTP adaptation, and explicitly provisional linear undo/redo implementation evidence | Authentication cannot imply authorization; roles cannot imply unconstrained capability; entity visibility cannot imply field disclosure; break-glass cannot become superuser or self-benefit authority; policy composes through Relational facts, runtime-bridge lowering, Signal decision evidence, and Query admission; application queries cannot bypass graph-read planning or fabricate no-N+1 proof; host predicates cannot return raw Signal decisions; hosts cannot own Signal graphs or temporal schedulers; wake eligibility cannot imply application-operation authority; cursors carry no authority; compensation preserves history; provisional undo/redo owns no canonical history; concurrent money movement commits once or returns an honest outcome | Milestone 9.16 OIDC, capability/touched-graph, conflict-of-interest, disclosure, break-glass, canonical-query/cursor/basis, graph-access/no-N+1, accounting, concurrency, idempotency, recovery, aftermath, conditional host/reinstall, live-revocation, cross-process consumer, facade, residue, and certification-only replay parity; Milestone 9.18 for tree-based undo/redo product acceptance |
| Installed graph contract integrity repair | Milestone 9.16.1.1 | Declaration-owned application aspect identity/revision; installed native schema catalog; typed entity/projection/relation reads; typed create/delete/write/link/unlink touches; typed reconciliation and correlation-family inspection | Installed operation inspection, graph-obligation selection, execution lowering, performed-evidence comparison, Host adoption, and successor package export consume one exact typed contract; no empty semantic-read posture, structured touch grammar, application contract reconstruction, or aftermath summary survives | Focused owner, Host-consumer, compile-boundary, and execution integration tests plus required repository checks |
| Portable Query packages and fresh readmission | Milestone 9.16.2 | Stable identities and declaration-minted references extending the 9.16.1.1 application aspect/correlation identities; complete typed package records; bounded reconstruction and fresh Query validation; deterministic neutral archive; host-owned release envelope and signing boundary | Records, bytes, checksums, signatures, repositories, filenames, and release names remain descriptive; only fresh Query validation against independently expected identity can recover package meaning; no application state, provider, secret, proof, handle, callback, or physical topology enters the archive; warm execution performs no package work | Identity/provenance compiler evidence; exact export/reconstruction equality; record omission/duplication/reorder/cross-splice mutants; archive golden vectors, corruption, compatibility, and budget courts; same-name coexistence; release-tool, facade, docs, boundary, and residue certification |
| In-memory composite runtime branching and branch-local MVCC | Milestone 9.17 umbrella: 9.17.1 owner bases/MVCC, 9.17.1.1 Relational service/lifecycle correction, 9.17.1.2 final owner-service/Signal-progress correction, 9.17.2 composite history/publication, 9.17.3 Query carriage/outbox gate/facade/certification | Owner-issued Relational/Signal bases; independently borrowable Relational and Signal owner services; lifecycle-total owner settlement and retention; branch-local MVCC; base Bridge-owned correspondence; dedicated Runtime World-owned single-parent composite commits and product-head comparison; Query-carried product-world affinity; live-runtime outbox admission bound to performed composite publication | Branch identities remain distinct; unrelated branches progress without whole-owner or composition locks; lost capabilities do not strand owner state; exact residency is independent of currentness; owner effects without product movement remain bounded and non-current; an owner-local outbox never dispatches after failed composite publication; application worlds remain memory-resident and no temporary physical-runtime abstraction is introduced | 9.17.1 owner-basis and publication-locality courts + 9.17.1.1 preparation/settlement independence, capability-loss recovery, exact-retention, lease-terminal, contention, feature, scheduled-proof, facade, and docs courts + 9.17.1.2 owner-bundle, same-branch, independent-progress, lifecycle, panic, capacity, facade, and scale courts + 9.17.2 identity, product-head comparison, retained-owner-effects, retention, ancestry, contention, and no-mixed-publication courts + 9.17.3 end-to-end shared-basis, component-divergence, same-head-race, substitution, lifecycle, outbox-gating, facade, compiler, docs, residue, and later cross-runtime semantic merge suites |
| Application API hardening | Milestone 9.17.4 | Entry-owned bindings/contributions, capability/elevation requests, installed handlers, opt-in retained publication, page/live resources, exact recovery, Bank/UI/server adoption, candidate integrity, runtime-cardinality construction and bounded output demand | No cached permission or hidden pins; stable portable identity; actual owner validation; no erased partial effects, per-entity publications, consumer-owned phase pipeline or compatibility route | Bank process/authority/recovery courts, UI/server integration, cross-crate values, cyclic candidate/resource/demand courts, and preserved 9.17.3 occurrence/Signal/World evidence |
| Tree-based semantic undo and redo | Milestone 9.18 | Exact source composite commit and target product branch/head, explicit per-component correction posture, installed inverse/compensation/reconciliation/reapplication contracts, applicability against intervening history, fresh Query admission, owner-local preparation, Runtime World coordinated publication, correction causality, and typed next actions | Reversal and reapplication create new composite commits; unchanged components retain exact bases; Signal reconciliation remains Signal-owned; history and alternatives remain intact; copied receipts and prior authority open no door; stale/conflicting divergence is typed before effects; external effects retain honest compensation/irreversibility posture; Query owns no history head | Milestone 9.18 composite-divergence, stale-head, component-basis, Signal-reconciliation, authority, compensation, external-effect, partial-preparation, zero-ordinary-work, facade, documentation, and residue certification + later cross-runtime merge/rebase/recovery suites |
| Managed advanced access and verified footprints | Milestone 9.19 | Installed-query-bound search and access products, Milestone 9.10 requirement/inventory/plan extensions, lifecycle products, coverage/membership witnesses, exact refinement, and verified realized footprints | Search preserves capability, purpose, exact composite product-world basis, disclosure, cursor, recovery, and aftermath; membership remains complete under negative-space change; protected candidates do not leak; footprints narrow but never widen authority | Milestone 9.19 bank/geometry search, disclosure, no-N+1, membership, footprint, lifecycle, memory, alternate-provider, facade/docs, and prohibition certification + Milestone 13 parity |
| Correlated paths and set execution | Milestone 9.20 | Typed heterogeneous path programs, admitted provider lowering, complete path dependencies, installed conflict relations, verified partitions, provider set operations, canonical reductions, and structural-cost evidence | Paths remain bounded and schema-typed; correlated reads consume one admitted graph plan; partitions are complete and conflict-free; bulk work is truly set-oriented; planning is not quadratic; unauthorized members are not processed or leaked | Milestone 9.20 chip/geometry/bank interpreter, no-N+1, partition-parity, slope, authority, facade/docs, and prohibition certification + Milestone 13 parity |
| Governed decision evidence | Milestone 9.21 | Schema-bound attachments, exact execution/occurrence binding, nested governance, retention/deletion, derived incremental summaries, typed omissions, and rebuild evidence | Attachments and summaries preserve classification, disclosure, purpose, capability, review, and deletion while opening no graph, operation, branch, correction, approval, or recovery authority | Milestone 9.21 bank/research privacy, deletion, invalidation, rebuild, stale-summary, authority, facade/docs, and prohibition certification + Milestone 13 parity |
| Occurrence-safe reuse and declared reproducibility | Milestones 9.15 and 9.22 | Distinct semantic artifact, production/acquisition occurrence, and independent-certification identities; installed substitution policy; exact, seeded, canonical-reduction, bound/comparator, distributional, and observational/non-replayable reproducibility contracts; purpose-aware stage reuse | Equal content cannot erase a required occurrence or manufacture independent evidence; cache reuse obeys consumer purpose; stale dependencies and incompatible provider/governance posture deny before use; leases and retained resources remain bounded | Milestone 9.15 occurrence-substitution and reproducibility matrix + Milestone 9.22 research-reference, reuse/lifecycle, facade/docs, alternate-provider, and prohibition certification + Milestone 13 parity |
| Honest candidate search, single-basis convergence, and transformation evidence | Milestone 9.15; durable resolution in cross-runtime Milestones 10-11 | Installed search-universe/completeness/optimality contracts, managed convergence epochs, transformation occurrences, correspondence cardinality, loss/disposition evidence, and basis-bound proposals | Heuristic or incomplete work cannot claim uniqueness or optimality; convergence, stability, feasibility, oscillation, and exhaustion remain distinct; derived evidence cannot become conflict, decision, approval, session, admission, or publication authority | Milestone 9.15 search-claim, convergence, transformation-authority, scale, and residue certification + cross-runtime Milestones 10-11 governed-resolution and session proof |
| Aggregation queries | Milestone 4 | Aggregation descriptors, grouping metadata | Aggregates stay tied to declared basis | Milestone 13 execution parity |
| Tolerance-aware aggregation | Milestones 4 and 5 | Tolerance policy metadata, live suppression metadata | Suppression does not change aggregate meaning | Milestone 13 live + policy masking parity and aggregation cases |
| Relational rollups | Milestone 4 | Rollup descriptors over relation edges | Rollups remain derived from declared truth basis | Milestone 13 execution parity + domain suites |
| Query-time derived fields | Milestone 4 | Derived-field declarations in canonical query/result shape | Derived fields are planned, not host-postprocessed | Milestone 13 execution parity |
| CDC-shaped output | Milestone 4; durable portability in Store Milestone 11 | Query-shaped CDC result families, delivery metadata | CDC-shaped output matches ordinary query meaning | Milestone 13 semantic parity; Store Milestone 19 joined parity |
| Snapshot-backed execution | Milestone 3 | Proof-carrying execution plan, snapshot basis metadata | Same query/context lowers to same plan and result | Milestone 13 snapshot-backed execution parity |
| Type-bound execution / generalized route-model binding | Milestone 3, shared integration with server/cloud | Type-bound execution descriptors tied to canonical plans | Bound descriptors round-trip to same plan as direct execution | Milestone 13 normalization parity; server/cloud integration suites later |
| Live read-to-subscribe promotion | Milestone 5 | Live execution context, query-to-signal lowering metadata | One-shot and live execution preserve semantics | Milestone 13 live-promotion equivalence |
| Incremental result maintenance | Milestone 5 | Query-shaped live patch artifacts, suppression metadata | Live patches preserve ordering/membership/projection | Milestone 13 live-promotion equivalence |
| Query-to-signal bridging | Milestone 5, shared with runtime bridge | Query relevance metadata, bridge-facing invalidation descriptors | Truth changes map to query-shaped maintenance honestly | Milestone 13 live equivalence + bridge-adjacent suites |
| Cross-runtime causal diagnostics | Milestone 9.3.1, shared with runtime bridge, relational, and signal | Bridge causal explanation envelopes, Query causal inspection artifacts, causal materialization receipts | Query inspection explains why an observation changed, was suppressed, was denied, or replayed without direct lower-runtime access | Milestone 9.3.1 causal explanation certification + Milestone 13 diagnostics suites |
| Query basis capability lifecycle | Milestone 9.3.2, shared with relational and runtime bridge | Basis capability envelopes, basis eligibility records, basis use receipts | Observation, mutation, replay, inspection, and materialization consume phase-typed basis proofs instead of raw branch/snapshot identifiers | Milestone 9.3.2 basis lifecycle certification + Milestone 13 branch/history suites |
| Authority-scoped effect execution | Milestone 9.3.3, shared with runtime bridge and relational | Authority-scoped effect plans, lowered effect execution plans, effect execution receipts | Query effects execute only from lowered proof-bearing plans; executors do not re-decide authority, basis, strategy, or artifact policy | Milestone 9.3.3 effect execution certification + Milestone 13 workflow/mutation suites |
| Declared projection consumption | Milestone 9.3.4 | Projection consumption declarations, materialized projection contracts, consumed fact receipts | Consumers use typed facts from declared materializations without reopening source authority | Milestone 9.3.4 projection consumption certification + Milestone 13 projection/domain suites |
| Intent admission decision lattice | Milestone 9.3.5 | Admission decisions, advisory/violation variants, admitted execution handoffs, decision trace envelopes | Query-crossing intents resolve before construction/lowering, and covered runtime-backed paths cross into execution through typed admitted handoffs with structured success, advisory, and violation context | Milestone 9.3.5 admission lattice certification + Milestone 13 diagnostics suites |
| Lower-runtime capability routing | Milestone 9.3.6, shared with runtime bridge, relational, signal, and store | Lower-runtime route plans, boundary execution receipts, lower-runtime boundary envelopes | Lower-runtime contact is capability-routed and receipt-backed; remaining direct paths are explicit compatibility debt | Milestone 9.3.6 boundary routing certification + Milestone 13 support-matrix suites |
| Domain-authored capability contributions | Milestone 9.3.7 | Domain capability contribution requests, admitted domain capability artifacts, canonical runtime materializers, declaration-scoped support traceability artifacts, workflow/continuity/aftermath/explanation contribution families | Domains contribute semantic capability posture through one public Query seam while Query keeps canonical runtime artifact ownership across major category families | Milestone 9.3.7 domain capability certification + Milestone 13 diagnostics/support suites |
| Query-as-beginning platform entry for serious downstream domains | Milestone 9.3.8, shared with worth-proof, worth-foundational, worth-relational, worth-runtime-bridge, and worth-signal | Typed domain entry surfaces, canonical declaration artifacts, progression states, route plans, boundary receipts, boundary envelopes, support/readiness snapshots, orchestration artifacts, certification bundles, and collaboration-entry prerequisites from shared lower-authority hardening specs | Serious downstream domains enter WORTH through one Query-owned seam that covers declaration, preparation, continuation, inspection, and lower-authority routing without rebuilding local pseudo-Query layers; later collaboration-facing phases consume retained lower-authority branch, merge, lineage, preview, policy, and strategy posture instead of reopening host glue | Milestone 9.3.8 platform-entry certification + Milestone 13 diagnostics/support/workflow suites |
| Temporal query basis and time-aware subscriptions | Milestone 9.4, shared with runtime bridge and signal temporal execution | Temporal query context descriptors, temporal subscription declaration metadata, bridge temporal basis requests | Truth basis and temporal execution basis stay distinct; time-only deliveries remain query-shaped | Milestone 9.4 temporal/async certification + Milestone 13 live/history suites |
| Time-only query result delivery | Milestone 9.4 | Temporal cause metadata, time-aware delivery batches, previous-value comparison basis | Clock wakes can change admitted query results without raw signal events or ambient timers | Milestone 9.4 temporal/async certification |
| Async/resource query families | Milestone 9.4, shared with runtime bridge and signal async resources | Async resource query declarations, result-state descriptors, completion-causality artifacts | Stale, cancelled, retried, failed, and superseded completions remain query-shaped and basis-bound | Milestone 9.4 temporal/async certification |
| Mixed truth/time/async delivery | Milestone 9.4 | Mixed-cause delivery metadata, cause ordering receipts, coalescing/suppression diagnostics | Host event arrival order cannot change canonical query-shaped delivery meaning | Milestone 9.4 temporal/async certification + Milestone 13 cross-feature suites |
| Temporal/async query certification | Milestone 9.4 | Temporal/async certification bundles, support matrix rows, diagnostic sufficiency bundles, reference workload artifacts | Every advertised runtime-backed temporal/async query family has hostile proof and fail-closed unsupported-neighbor coverage | Milestone 9.4 completion itself + Milestone 13 query certification matrix |
| Scope/template composition hardening | Milestone 9.5 | Canonical scope-expansion artifacts, template-instantiation artifacts, scope/template support rows | Reusable composition stays canonical and support-typed instead of remaining admitted debt | Milestone 9.5 debt-close certification + Milestone 13 composition suites |
| Core view-shape family hardening | Milestone 9.5 | Production-ready `table`, `detail`, inspector-detail, and grouped-view-shape artifacts | Core view families stop advertising admitted-but-debt runtime-backed posture | Milestone 9.5 debt-close certification + Milestone 13 view-shape suites |
| Grouped composition closure | Milestone 9.5 | Grouped template/composition artifacts, grouped planning support profiles | Grouped planning no longer carries explicit composition debt in ordinary product docs or support surfaces | Milestone 9.5 debt-close certification |
| Retained-artifact projection consumption hardening | Milestone 9.5 | Retained derived-artifact and live-artifact source-family bindings, typed fact receipts | Projection consumption remains the ordinary typed fact lane rather than special-case pack/bind/decode folklore | Milestone 9.5 debt-close certification + Milestone 13 projection/domain suites |
| Preserved temporal/async reuse-neighbor closure | Milestone 9.5 | Inspector/grouped preserved reuse artifacts, preserved runtime-backed semantics, reuse digests | Covered temporal/async reuse neighbors carry merged `9.4` meaning across the full runtime-backed reuse surface | Milestone 9.5 debt-close certification + Milestone 13 cross-feature suites |
| Raw runtime read bootstrap hardening | Milestone 9.5 | Valid bridge-backed read-runtime bootstrap artifact, raw runtime bootstrap support posture, hostile read-runtime harness entry surface | Hostile runtime-backed read tests can reach the ordinary raw read lane without custom bridge-backed assembly folklore | Milestone 9.5 debt-close certification + Milestone 13 runtime/read harness suites |
| Region-scoped live invalidation | Milestone 5.1 | Region/partition-aware invalidation metadata, locality predicates, region-scoped suppression metadata | Live narrowing stays below broad aspect scope where lower-runtime locality contracts admit it | Milestone 13 live equivalence + geometry domain suites |
| Change-stream-backed delivery contracts | Milestones 5.1, 9, and 11 | Stream-lowered delivery declarations, delivery metadata, durable stream checkpoints | Query-shaped delivery lowers into formal stream contracts without semantic drift | Milestone 13 delivery-shape + durable continuation parity |
| Preview-session query contexts | Milestone 5.2 | Preview-session basis metadata, preview-lifecycle metadata, preview/promotion comparison artifacts, preview-live admission and drift artifacts | Preview-bound queries preserve explicit basis and lifecycle meaning, and preview-live remains basis-explicit under maintenance, denial, and explicit rebind | Milestone 13 branch/history/workflow suites |
| Frontier-aware planning | Milestone 5.3 | Frontier-derived planning metadata, breadth posture, parallel-admission posture | Planner consumes lower-runtime frontier posture without executor rediscovery | Milestone 13 planning parity + performance suites |
| Deterministic parallel admission | Milestone 5.3 | Parallel-admission decisions on planned routes, serial fallback diagnostics | Serial and parallel admitted lanes remain semantically identical | Milestone 13 planning parity + performance suites |
| Branch-scoped reads | Milestone 6 | Branch-targeting query context metadata | Same query shape runs against different branches honestly | Milestone 13 historical/diff parity |
| Time-travel reads | Milestone 6; physical completion in Store Milestone 10 | Historical basis descriptors, snapshot/commit targets | Historical basis is explicit and parity-safe | Milestone 13 semantic parity; Store Milestones 10 and 19 physical parity |
| Diff queries | Milestone 6 | Structured diff query artifacts, comparison-basis metadata | Diff results align with declared projection/scope | Milestone 13 historical + diff + result-shape parity |
| Branch comparison views | Milestone 6 with Milestone 8 view-shape semantics | Comparison basis metadata, view-shape metadata | Branch comparisons preserve basis identity and result meaning | Milestone 13 historical + diff parity |
| Historical evaluation contracts | Milestone 5.4 with completion in Milestone 6 | Historical materialization-path metadata, compatibility/admission artifacts | Historical reads stay explicit about how truth was materialized and whether the request was honestly admissible | Milestone 13 historical/diff parity + diagnostics suites |
| Lineage traversal queries | Milestone 7 | Lineage traversal descriptors, lineage-basis metadata | Lineage results stay typed and explainable | Milestone 13 lineage/correspondence parity |
| Correspondence queries | Milestones 5.4 and 7 | Lineage-backed and structural-fingerprint-backed correspondence descriptors, ambiguity/rejection metadata | Ambiguous correspondence stays explicit and advisory correspondence never silently becomes continuity | Milestone 13 lineage/correspondence parity |
| Inspector-pattern detail with live aspect projection | Milestones 5 and 8 | Inspector/detail view-shape metadata, aspect-focused live patch artifacts | Inspector live projection proves narrow invalidation | Milestone 13 live equivalence + view-shape cross-feature suites |
| View shapes: table/detail | Milestone 8 | View-shape descriptors, delivery/live-patch metadata | View shape affects planning and live semantics, not only typing | Milestone 13 web/detail workflow suites |
| View shapes: kanban/grouped | Milestone 8 | Grouped view metadata, group-splice patch artifacts | Group membership changes preserve view semantics | Milestone 13 view-shape cross-feature suites |
| View shapes: timeline/chart | Milestones 4 and 8 | Temporal/grouping metadata, tolerance/suppression metadata | Temporal/chart semantics stay explicit and basis-honest | Milestone 13 historical/diff + aggregation suites |
| Named scopes | Milestone 8 | Scope fragments in canonical query artifact | Scope expansion equals direct construction | Milestone 13 normalization parity |
| Query templates with parameter slots | Milestone 8 | Template descriptors, parameter binding metadata | Parameterized instantiation preserves canonical meaning | Milestone 13 normalization parity |
| Saved and named query definitions | Milestone 8; durable completion in Store Milestone 11 | Saved-query canonical artifact, durable saved-query records | Reloaded saved query preserves identity and meaning | Milestone 13 semantic-freeze oracle; Store Milestones 11 and 19 joined parity |
| Result shape declarations for delivery contracts | Milestones 1 and 9 | Typed result shapes, delivery-shape metadata | Delivery metadata remains identical to canonical masked/projected result | Milestone 13 delivery-shape parity |
| Policy-aware aspect masking | Milestone 9 | Policy masks in plan metadata | Masked aspects never enter execution plan | Milestone 13 live + policy masking parity |
| Branch-level access scoping | Milestone 9 | Branch-access validation metadata | Denied branches fail before reads execute | Milestone 13 policy/tenant correctness |
| Automatic tenant branch scoping | Milestone 9 | Tenant branch-resolution metadata | Tenant context narrows truth basis explicitly | Milestone 13 policy/tenant correctness |
| Tenant-scoped schema awareness | Milestone 9 | Tenant schema-basis metadata, tenant-aware validation artifacts | Validation uses tenant schema rather than a global default | Milestone 13 tenant schema variation + validation parity |
| Graph-native relationship proofs | Milestone 9, shared with schema/platform policy authority | Relationship-proof predicate/query nodes, denial metadata | Broken proof chains deny explicitly without data leakage | Milestone 13 policy/tenant correctness |
| Multi-tenant query architecture | Milestone 9; durable completion in Store Milestone 11 | Tenant basis metadata, durable tenant/query artifacts | Tenant-scoped reads remain parity-safe across restart where supported | Milestone 13 policy/tenant correctness; Store Milestone 19 joined parity |
| Structured content aspect queries | Milestone 2; live/update consequences in Milestone 5 and Store Milestones 9-10 | Structured content projection/predicate descriptors | Structured content legality and live narrowing stay explicit | Milestone 13 validation/live oracle; Store Milestone 19 joined parity |
| Query planning and optimization | Milestone 3; store-aware completion in Store Milestone 9 | Proof-carrying execution plans, store pushdown diagnostics | Planner lowers once; executor does not rediscover semantics | Milestone 13 plan oracle; Store Milestones 9 and 19 physical parity |
| Delivery contracts for integrations | Milestones 4 and 9; durability in Store Milestone 11 | CDC/result delivery metadata, durable delivery cursors | Delivery contracts remain query-shaped and basis-honest | Milestone 13 delivery oracle; Store Milestones 11 and 19 joined parity |
| Query-authored mutation intents | Milestone 5.5 | Mutation-intent declarations, lowered commit-strategy request descriptors, context-derived observation artifacts | Query-authored mutation workflows lower into relational authorities without semantic drift | Milestone 13 workflow/mutation suites |
| Branch-native workflow orchestration | Milestones 5.2 and 5.5 | Preview/compare/merge workflow declarations, conflict inspection artifacts, post-merge inspection artifacts | Branch workflows stay inside the query framework while preserving lower-crate authority boundaries | Milestone 13 workflow/mutation + branch suites |
| Query-triggered writeback declarations | Milestone 5.5 | Writeback-trigger declarations, lowered bridge writeback descriptors, causality/admission metadata | Query-triggered writeback stays declaration-owned by query and execution-owned by the bridge | Milestone 13 workflow/mutation + diagnostics suites |
| Runtime authoritative mutation evidence | Runtime Authoritative Mutation Evidence Gate | Declared/resolved target evidence, batch/session authority evidence, existing-truth binding descriptors, naming/continuity evidence bundles | Downstream write-heavy domains receive authority evidence through the public facade without local target-recovery glue | Milestone 13 workflow/mutation + diagnostics suites |
| Unified application facade | Milestone 5.6 | Authority-preserving public facade surface, capability registry, support metadata | Domain developers can use query as the daily-driver import without erasing lower-crate ownership | Milestone 13 support-matrix + certification suites |
| Unified runtime configuration | Milestone 5.6 | Sectioned `WORTHQueryConfig`, subsystem-owned config sections, capability-gated config metadata | Unified config remains architecture-shaped rather than bag-shaped | Milestone 13 support-matrix + diagnostics suites |
| Store-backed pushdown and execution parity | Store Milestone 9 | Store-backed plan variants, fallback diagnostics | Store-backed results equal Query's semantic oracle | Store Milestones 9 and 19 |
| Durable saved queries and cursors | Store Milestone 11 | Durable saved-query records, durable cursor/checkpoint records | Restart preserves canonical identity and continuation point | Store Milestones 11 and 19 |
| Import/export portability of query artifacts | Store Milestones 11 and 16 | Portable query artifact bundles and basis identity | Imported/exported artifacts preserve canonical meaning | Store Milestones 11, 16, and 19 |
| Blob/media-backed query delivery | Store Milestone 12 | Blob/media reference projections, durable delivery handles, upload-associated result metadata | Blob-backed results preserve canonical query meaning and policy masking | Store Milestones 12 and 19 |
| Query certification matrix | Milestone 13 | Runtime-backed certification bundles and provider-independent semantic parity oracles | Every Query capability has hostile semantic proof, not just local demos | Milestone 13; Store Milestone 19 consumes the oracle |

If a future query capability is added to `worth_query_vision.md`, this appendix
must gain a row in the same patch or the roadmap is incomplete.

## Companion Documents

- [worth_query_vision.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/worth-query/worth_query_vision.md)
- [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/worth-query/test-requirements.md)
- [milestone-9.3.1.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/worth-query/milestone-9.3.1.md)
- [milestone-9.3.2.md](./milestone-9.3.2.md)
- [milestone-9.3.3.md](./milestone-9.3.3.md)
- [milestone-9.3.4.md](./milestone-9.3.4.md)
- [milestone-9.3.5.md](./milestone-9.3.5.md)
- [milestone-9.3.6.md](./milestone-9.3.6.md)
- [milestone-9.3.7.md](./milestone-9.3.7.md)
- [milestone-9.4.md](./milestone-9.4.md)
- [milestone-9.5.md](./milestone-9.5.md)
- [milestone-9.10.md](./milestone-9.10.md)
- [milestone-9.11.md](./milestone-9.11.md)
- [milestone-9.11-closeout.md](./milestone-9.11-closeout.md)
- [milestone-9.12.md](./milestone-9.12.md)
- [milestone-9.13.md](./milestone-9.13.md)
- [milestone-9.13-closeout.md](./milestone-9.13-closeout.md)
- [milestone-9.13.1.md](./milestone-9.13.1.md)
- [milestone-9.13.2.md](./milestone-9.13.2.md)
- [milestone-9.14.md](./milestone-9.14.md)
- [milestone-9.15.md](./milestone-9.15.md)
- [milestone-9.16.md](./milestone-9.16.md)
- [milestone-9.16.1.md](./milestone-9.16.1.md)
- [milestone-9.16.1.1.md](./milestone-9.16.1.1.md)
- [milestone-9.16.2.md](./milestone-9.16.2.md)
- [milestone-9.17.md](./milestone-9.17.md)
- [milestone-9.17.1.md](./milestone-9.17.1.md)
- [milestone-9.17.1.1.md](./milestone-9.17.1.1.md)
- [milestone-9.17.1.2.md](./milestone-9.17.1.2.md)
- [milestone-9.17.2.md](./milestone-9.17.2.md)
- [milestone-9.17.3.md](./milestone-9.17.3.md)
- [milestone-9.17.4.md](./milestone-9.17.4.md)
- [milestone-9.18.md](./milestone-9.18.md)
- [milestone-9.19.md](./milestone-9.19.md)
- [milestone-9.20.md](./milestone-9.20.md)
- [milestone-9.21.md](./milestone-9.21.md)
- [milestone-9.22.md](./milestone-9.22.md)
- [runtime-api-public-stabilization-plan.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/worth-query/runtime-api-public-stabilization-plan.md)
- [runtime-authoritative-mutation-evidence-plan.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/worth-query/runtime-authoritative-mutation-evidence-plan.md)
- [worth_runtime_bridge_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/worth-runtime-bridge/worth_runtime_bridge_roadmap.md)
- [worth_relational_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/worth-relational/worth_relational_roadmap.md)
- [Worth Store Runtime And Integration Roadmap](../worth-store/runtime-integration-roadmap.md)
- [MENTALITY.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/coding_guidelines/MENTALITY.md)
- [architectural_guidelines.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/coding_guidelines/architectural_guidelines.md)
- [domain_standards.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/coding_guidelines/domain_standards.md)
- [performance_guidelines.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/_docs/coding_guidelines/performance_guidelines.md)
