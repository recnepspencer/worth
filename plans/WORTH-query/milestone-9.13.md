# Milestone 9.13: Declarative Query Experience And Phase-Surface Cutover

Status: Complete. Core Phases 1-12 closed on 2026-07-14. Add-on Phases 13-30
closed on 2026-07-15. The completed boundary includes the runtime-backed
declarative product surface, runtime-installed domain packages and
runtime-affine handles, one canonical domain-capability authority, and exact
Foundational-native aspect values across Query authoring, predicates, schema
semantics, materialization, projection consumption, and ordinary consumer DX.
Store-backed execution parity, durable restore, saved-artifact survival, and
durable continuation remain Milestones 10 and 11. See
[milestone-9.13-closeout.md](./milestone-9.13-closeout.md) for closure evidence
for all 30 phases.

## Goal

Make ordinary `worth-query` usage capability-oriented, declarative, and
discoverable: consumers describe the query outcome they need, Query owns the
proof-preserving phase progression, and lower-level canonicalization,
admission, planning, lowering, execution, maintenance, and inspection
mechanisms cannot become a second application framework. Domain setup must
enter through one canonical package installed into a specific Query runtime so
domain identity, operation registration, invariant posture, contributions,
diagnostics, and later execution share one runtime-owned authority from birth.
Every aspect value that enters or leaves that experience must retain
Foundational's exact scalar or struct meaning. Query may add proof-bearing
wrappers and operator capabilities, but it may not replace that meaning with a
coarser Query-owned value algebra.

## Why This Milestone Exists

Milestone 9.12 closes raw authority escape hatches, but authority safety alone
does not make a coherent product API. The ordinary facade still reflects much
of Query's internal phase topology. A consumer can be forced to understand
which request to canonicalize, which binding function to call, which planning
context to seed, which lowerer to select, and which executor or receipt builder
finishes the operation.

That ceremony creates two risks. Consumers rebuild local orchestration above
Query, and later store-backed work duplicates the same public journey for a
second substrate. This milestone closes both risks before Milestone 10.

Phases 1-12 closed the ordinary capability journeys but left the domain-
extension story split across application-facade configured handles, raw-string
contribution entry, manually supplied graph-operation registries, and a runtime
builder that does not install one complete domain definition. That split means
domain meaning begins inside Query conceptually without yet being owned by one
runtime mechanically. Phases 13-20 close that remaining authority fork before
store-backed execution inherits it.

The aspect-native substrate refactor removed JSON authority and now retains
Foundational values through mutation, materialization, and projection facts.
However, the ordinary surface still narrows mutation authoring to a few scalar
constructors, models predicate operands and schema fields through coarse Query-
owned enums, rejects complete struct values on a materialized bridge path, and
names a marker type as though it were a native row. Phases 21-30 close that
consumer-boundary fork before Milestone 10 duplicates it across another
execution substrate.

## Governing Summaries

- `MENTALITY.md` protects production-grade foundations: solve the hostile
  composition problem first, make incorrect usage mechanically unavailable,
  and do not preserve ordinary completion work as compatibility debt.
- `arch_laws.md` protects proof and authority continuity: semantic intent must
  compile into orchestration, lowered plans are the executor's only input, and
  every phase consumes the proof emitted by its predecessor.
- `composition_laws.md` protects legible responsibilities: facade modules may
  aggregate but not implement, orchestration must read as named semantic steps,
  and public phase mechanics must not form an accidental second framework.
- `domain_structure_laws.md` protects structural ownership: public capability
  surfaces must be narrower than internal topology, and authoring, authority,
  execution, diagnostics, compatibility, and certification must remain
  distinguishable in both the tree and import graph.
- `perf_laws.md` protects pre-resolved and cost-honest execution: policy and
  topology decisions happen before the hot path, proof is carried instead of
  rediscovered, and ergonomic APIs may not hide broad scans or allocation.
- `aspect_native_query_refactor.md` and the relational aspect-refactor rules
  protect Foundational carriers as the semantic substrate: removing JSON is a
  prerequisite, not proof of completion, when public authoring, schema,
  predicate, row, or projection surfaces still narrow native meaning.
- Foundational's `AspectValue`, `ScalarAspectType`, `StructAspectValue`,
  contract-validation, and canonicalization surfaces protect the one native
  value vocabulary that Query must carry rather than restate.
- `WORTH_query_roadmap.md` protects one declared meaning lowered once and
  executed against canonical truth. It positions this cutover after authority
  closure and before runtime/store execution parity; runtime-installed domain
  authority must therefore close here rather than becoming a store-backend
  concern. The same sequencing requires exact Foundational-native value
  semantics to close here before store pushdown inherits predicate, mutation,
  row, and projection contracts.

## Adversarial Constraint

An ordinary consumer must be able to express each supported read, aggregate,
live, historical, comparison, preview, mutation, workflow, inspection, and
domain-extension outcome without choosing or reconstructing Query's internal
phase order, backend, policy lowerer, execution route, or receipt assembly.

Equivalent declarations must converge on the same canonical artifact,
admitted authority, plan, result meaning, and diagnostic identity. Skipped,
reordered, cross-capability, cross-basis, stale, or backend-shaped assembly
must be unrepresentable or deny before lower-runtime contact. The ergonomic
surface must add no unbounded work and must not become a generic builder that
erases capability-specific denials.

For domain extensions, two equivalent packages must install to the same
canonical runtime meaning regardless of declaration order. Raw domain strings,
consumer-authored identity digests, foreign-runtime handles, stale installation
generations, manually paired registries, or low-level materializers must be
unable to mint, reconstruct, or compete with installed domain authority. The
runtime must resolve an installed operation in bounded indexed work without
rescanning packages or asking the consumer to supply a registry.

For aspect values, every Foundational scalar family and every admitted struct
aspect must retain exact meaning through public authoring, canonicalization,
contract validation, planning, execution, retained materialization, projection
consumption, receipt identity, and consumer refinement. A Query-owned coarse
enum, generic integer collapse, ad hoc string encoding, scalar-only bridge
branch, or marker named as a materialized row must be unable to become a
competing value authority or silently narrow a supported family.

## Product Decision Lock

- Ordinary APIs express desired Query capabilities, not pipeline mechanics.
- Query owns canonicalization, binding, validation, admission, planning,
  lowering, execution routing, receipt production, and derived diagnostics.
- Proof-bearing phase artifacts may be returned for observation or advanced
  integration, but ordinary consumers cannot construct them or independently
  advance them.
- Capability families retain distinct declarations and typed outcomes where
  their cost, failure, lifecycle, or correctness semantics differ. This is not
  a single universal query builder.
- Safe defaults may remove ceremony only when Query can derive them from
  admitted context. Authority, basis, policy, tenant, bounds, and lifecycle
  choices are never guessed.
- Runtime-backed and future store-backed execution implement the same admitted
  capability contract. Backend selection is Query-owned and observable, not
  application-authored.
- Advanced domain contribution remains possible through explicit extension
  contracts; it does not require public access to Query's internal pipeline.
- A domain package is declarative setup input. Query validates, canonicalizes,
  admits, installs, indexes, and owns the resulting runtime artifact.
- Only a specific runtime may mint an installed domain handle. The handle
  carries runtime authority and installation generation; representation,
  support snapshots, labels, or digests cannot recreate it.
- Domain identity is encoded by Query from typed components. Consumers do not
  author a canonical digest string or restate domain owner identity per
  operation.
- Invariants, graph obligations, graph-read operations, declaration families,
  and contribution posture compile from one package into one installation.
  Derived registries and indexes are rebuildable from that installed artifact.
- Domain contributions are authored from the installed handle. The raw-string
  `worth_query_domain(...)` lane and publicly callable phase materializers do
  not survive add-on closeout.
- Physical adapters remain allowed only at real external boundaries such as
  storage, schema ingress, signal delivery, or transport. They cannot become a
  second semantic domain-registration path.
- Query remains generic. Domain-specific vocabulary may extend the installed
  handle through downstream-owned typed extension contracts; Query does not
  hard-code a particular product domain into its generic facade.
- Foundational owns native aspect keys, scalar value families, struct values,
  canonical wrappers, and contract-declared value shape. Query owns ergonomic
  intent, proof-bearing admission, operator eligibility, result shaping, and
  consumer refinement over those exact carriers.
- Query-specific authored, predicate, schema, row, or consumed-value types may
  wrap Foundational carriers only to add a real proof, lifecycle, or capability
  distinction. They may not duplicate or collapse the native value algebra.
- Operator-specific narrowing is legal only when derived from the admitted
  Foundational contract. Equality and membership preserve exact native value
  identity; string, numeric, ordering, and temporal operators expose only the
  families their declared capability admits.
- Struct aspects remain structs until an explicit projection mask selects
  scalar fields. Query may expose selected leaves, a complete admitted struct,
  or both, but it may not reject or flatten struct meaning accidentally.
- Canonical aspect-value identity encoding has one authority. Debug output,
  presentation text, and subsystem-local formatting are never digest or
  equivalence authority.
- `WorthQueryNativeRow` may remain only if it becomes a real native row
  product. If it is a phantom type parameter, it must be renamed and removed
  from consumer teaching so its name cannot promise retained row data.
- Compatibility aliases, dual constructors, and deprecated phase entrypoints
  do not survive closeout.
- Certification, support discovery, inventories, and golden artifacts remain
  outside the ordinary product surface.

## Phase Plan

### Phase 1: Freeze Consumer Journeys And The Public Phase Graph

Inventory what ordinary consumers actually do and map each journey from public
entrypoint to canonical result, including every public function that exposes or
advances an internal Query phase. This freezes the refactor boundary before API
design begins.

**Relevant subsystems**

- `facade` and crate-root exports
- authoring, canonicalization, binding, validation, planning, and execution
- policy, live, historical, preview, workflow, and inspection surfaces
- Worth UI and at least one serious domain consumer

**Relevant APIs**

- public constructors and free functions in ordinary facade namespaces
- public phase artifacts and their transition functions
- consumer-local Query wrappers, coordinators, and receipt assembly
- public API snapshots, residue registries, and import audits

**Warnings**

- Export count alone cannot distinguish a rich product API from leaked
  machinery; inventory entries must record who is expected to call them.
- Existing consumer wrappers may encode missing Query capabilities rather than
  mere cleanup debt. Their semantic responsibility must be classified before
  deletion.
- Tests and examples are consumers and can perpetuate the wrong mental model.

**Test requirements**

- A seeded inventory test adds an unclassified phase transition to an ordinary
  facade and proves the audit fails with its exact source location.
- A consumer-residue test detects local canonicalize/bind/plan/execute
  orchestration even when it is renamed or split across helper functions.
- A journey-equivalence test proves facade aliases and deep imports resolve to
  distinct rows rather than being collapsed by name similarity.

**Engineering decisions**

- Each public surface is classified as ordinary declaration, ordinary outcome,
  sealed returned proof, extension contract, terminal representation,
  diagnostics, certification, compatibility, or internal mechanism.
- Each consumer journey records declared intent, required context, admitted
  capability, Query-owned phase chain, result, receipts, diagnostics, cost
  counters, and local ceremony.
- The inventory is source-backed and fails when newly public constructors or
  transitions are not classified.

**Open questions**

- None.

### Phase 2: Freeze The Capability-Oriented Facade And DX Grammar

Define the ordinary vocabulary consumers discover first. The surface is
organized by what the consumer is trying to accomplish, with progressive
disclosure for optional policy, live, history, workflow, and inspection
capabilities.

**Relevant subsystems**

- facade namespaces and re-export files
- authoring builders and typed domain declarations
- ordinary outcome and typed stop families
- API documentation and IDE-visible examples

**Relevant APIs**

- query/read declarations
- aggregate count declarations
- live subscription declarations
- history, diff, and correspondence declarations
- preview, mutation, and workflow declarations
- result, receipt, and inspection entrypoints

**Warnings**

- A single prelude containing every public type is not discoverability.
- Fluent syntax must not permit invalid phase order or conceal expensive
  traversal, unbounded collections, or implicit basis selection.
- Parallel convenience methods with slightly different defaults recreate
  semantic ambiguity.

**Test requirements**

- A compiler-backed grammar contract freezes one complete ordinary journey
  signature per capability family: facade namespace, declaration/refinement/
  terminal vocabulary, outcome, typed stop, next action, explicit context,
  cost disclosure, ceremony budget, and the later phase that owns its first
  executable transcript. Missing fields or an unowned transcript fail this
  phase.
- Compile-pass DX transcripts prove every capability implemented through
  Phase 5 uses only its frozen ordinary facade imports. Phases 6 through 9
  must add the executable transcript for each family they implement, and
  Phase 12 must reject any frozen grammar without a compiling final journey.
- Compile-fail transcripts prove capability-specific options cannot be applied
  to the wrong declaration family or after admission.
- Compile-fail tests prove a declaration cannot run without an explicit context
  handoff. Hostile context tests prove missing or mismatched basis, policy,
  tenant, or relationship authority inside that handoff produces a typed
  next-action stop instead of guessing.

**Engineering decisions**

- The public grammar follows `declare -> refine -> admit/run -> outcome`; Query
  may fuse steps internally without exposing its internal phase count.
- Phase 2 freezes the final grammar and transcript owner; it does not create
  placeholder declarations, synthetic success outcomes, or non-constructible
  context shells for capabilities whose operational semantics belong to later
  phases. Each owning phase must implement the already-frozen signature rather
  than redesigning it.
- Common refinement vocabulary is shared only where meaning, cost, denial, and
  lifecycle are genuinely identical.
- Typed stops report the missing decision and the valid next action. They do
  not expose internal stage names as consumer instructions.
- The facade has a measurable ceremony budget: reference journeys count
  imports, consumer-authored intermediate values, explicit transitions, and
  local adapters before and after cutover.

**Open questions**

- Exact namespace and method names remain implementation decisions, provided
  the frozen journey grammar and capability boundaries are preserved.

### Phase 3: Converge Ordinary Authoring On One Canonical Declaration Entry

Ensure all ordinary query shapes begin as typed declarative intent and lower
through one Query-owned canonicalization path. Eliminate parallel raw request,
artifact-first, and payload-shaped authoring lanes.

**Relevant subsystems**

- `authoring`
- `canonicalization`
- composition, scopes, templates, predicates, and result shapes
- domain graph operation declarations

**Relevant APIs**

- detail and collection authoring builders
- typed predicate, traversal, ordering, and result-shape selectors
- scope/template composition and parameter binding
- canonical query bundle construction

**Warnings**

- Ergonomic strings are acceptable only as authoring input that immediately
  lowers into native typed carriers; they cannot survive as internal authority.
- Direct canonical artifact construction is a parallel authoring lane even if
  the resulting artifact is well formed.
- A generic expression escape hatch can silently bypass family-specific
  legality and cost bounds.

**Test requirements**

- Equivalent direct, composed-scope, and template-instantiated declarations
  converge on the same canonical query and result-shape identity.
- Compile-fail tests prove ordinary consumers cannot construct canonical
  bundles, canonical entries, or validated selector carriers directly.
- Invalid field, traversal, predicate, and result-shape combinations deny with
  typed authoring context before binding or lower-runtime contact.

**Engineering decisions**

- Canonical artifacts remain inspectable, serializable where appropriate, and
  stable for replay, but are minted only by Query-owned lowering.
- Domain extensions contribute typed vocabulary through declared extension
  contracts rather than custom canonicalization forks.
- Authoring sugar is tested for canonical equivalence, not as a separate
  semantic path.

**Open questions**

- None.

### Phase 4: Bind Context, Basis, Tenant, And Policy Declaratively

Replace consumer-authored sequencing across binding, basis, tenant, policy,
and admission APIs with one declarative context handoff that produces the
sealed capability required by the requested journey.

**Relevant subsystems**

- binding and query context
- basis lifecycle
- policy basis, narrowing, planning, and delivery
- tenant and relationship-proof admission

**Relevant APIs**

- binding requirements and resolutions
- scoped basis capabilities
- policy/tenant admission and narrowing
- relationship-proof requirements and typed denials

**Warnings**

- Context ergonomics must not flatten distinct basis, tenant, policy, and
  relationship authorities into an untyped bag.
- Reusing a context is legal only under an explicit equivalence contract.
- Policy or tenant defaults cannot be inferred from ambient process state.

**Test requirements**

- Equivalent declarations with equivalent admitted context converge on the
  same scoped capability and narrowed canonical identity regardless of input
  ordering.
- Cross-basis, stale-policy, cross-tenant, and mismatched relationship proof
  combinations deny before planning or execution and produce no partial
  successor.
- Reuse tests prove changed policy epoch or basis generation invalidates only
  the affected context and cannot reuse a prior admission.

**Engineering decisions**

- Consumers supply declarations and authority-bearing context; Query derives
  binding requirements, resolves supported bindings, admits basis, narrows
  policy, and produces one sealed handoff.
- Missing context returns a typed, capability-specific stop that identifies
  what authority must supply next.
- Internal policy phase artifacts are carried through the proof chain, not
  exposed as ordinary assembly pieces.

**Open questions**

- None.

### Phase 5: Collapse Read Planning And Execution Behind Admitted Queries

Make one-shot detail, collection, graph, aggregate, and composed reads execute
from an admitted query capability. Consumers may inspect planning and execution
evidence, but do not select planners, seed plan contexts, choose execution
routes, or assemble result bundles.

**Relevant subsystems**

- validation and planning
- frontier planning and execution
- collection and graph composition
- runtime-backed read routing and result construction

**Relevant APIs**

- planning request contexts and plan seed functions
- collection planning and post-read shaping
- serial and parallel execution routes
- runtime execution and result-bundle builders

**Warnings**

- Hiding the planner does not permit hiding traversal breadth, fallback, or
  execution cost from the outcome.
- Collection, detail, graph, and aggregate families must not be forced through
  scalar or shape-erasing execution.
- Backend choice cannot leak into canonical query meaning or result shape.

**Test requirements**

- Direct ordinary execution and an instrumented internal phase-chain fixture
  produce identical canonical plan identity, result, receipt, and counters.
- Compile-fail tests prove ordinary consumers cannot seed plans, choose serial
  or parallel routes, call executors with partially proved artifacts, or build
  successful result bundles.
- Unsupported family, excessive traversal, and invalid result-shape requests
  deny before execution with exact zero lower-runtime work.

**Engineering decisions**

- The admitted query is the only ordinary input to execution.
- Planning reports, route posture, fallbacks, and counters are derived evidence
  attached to outcomes or inspection, not prerequisites callers assemble.
- Internal execution remains phase-typed and backend-extensible so Milestone 10
  can add store-backed implementations without changing the ordinary journey.

**Open questions**

- None.

### Phase 6: Make Live Queries Framework-Owned Resources

Turn live usage into a declarative lifecycle owned by Query. Consumers declare
desired live behavior and receive a managed handle; they do not independently
assemble subscription admission, activation, maintenance, continuation,
suppression, or teardown phases.

**Relevant subsystems**

- declarative live
- subscription declaration and activation
- live planning, maintenance, delivery, and continuation
- temporal, async, preview-isolation, and policy-live paths

**Relevant APIs**

- live query declarations and view-shape refinements
- subscription family selection and managed lifecycle handles
- delivery batches, continuation receipts, and teardown outcomes
- policy drift and live rebind outcomes

**Warnings**

- A managed handle must not become a mutable bag that lets consumers assert
  activation or maintenance posture.
- Delivery callbacks, streams, and UI adapters are projections of the same
  lifecycle, not separate subscription authorities.
- Resource ownership includes deterministic teardown and orphan prevention.

**Test requirements**

- A compile-pass `facade::live` journey implements the Phase 2 grammar contract
  exactly and proves declaration, context handoff, open, delivery observation,
  and deterministic close without importing lifecycle phase machinery.
- One-shot and live-promoted evaluation converge for the same admitted query,
  basis, policy, and view shape across update, suppression, and replay.
- Compile-fail tests prove consumers cannot activate an unadmitted declaration,
  author maintenance posture, fabricate continuation, or revive a disposed
  handle.
- Crash/resume, policy drift, stale basis, and preview isolation tests prove a
  handle cannot cross its admitted lifecycle or leak authoritative residue.

**Engineering decisions**

- Query registers, tracks, advances, and disposes every live resource.
- The ordinary surface exposes declaration refinements, a managed handle,
  typed delivery, observable lifecycle posture, and explicit typed stops.
- Temporal and async causes remain distinct in evidence even when the consumer
  receives them through one managed delivery experience.

**Open questions**

- None.

### Phase 7: Unify Historical, Diff, And Correspondence Journeys

Make historical reads, branch comparison, diffs, lineage, and correspondence
begin from the same declarative query vocabulary while preserving their
distinct basis, ambiguity, materialization, and cost semantics.

**Relevant subsystems**

- historical and query context
- correspondence and correspondence history
- identity evolution and lineage
- diff shaping and comparison result bundles

**Relevant APIs**

- historical basis and materialization declarations
- comparison-basis declarations
- diff, lineage, and correspondence refinements
- ambiguity, disagreement, denial, and success outcomes

**Warnings**

- Historical and current reads may share authoring grammar but not pretend to
  have identical availability or materialization cost.
- Correspondence advisory evidence must never silently promote to continuity.
- Diff and comparison must bind both bases structurally; a convenient tuple is
  not sufficient proof.

**Test requirements**

- Compile-pass `facade::history` and `facade::comparison` journeys implement
  their Phase 2 grammar contracts exactly using only capability-specific
  declaration, context, outcome, stop, and inspection vocabulary.
- Current, historical, and restored internal fixtures yield equivalent result
  meaning when they observe the same canonical truth basis.
- Cross-basis pairing, missing history, ambiguous correspondence, and stale
  lineage deny or remain advisory without partial success artifacts.
- Compile-fail tests prove consumers cannot build comparison contexts, promote
  ambiguity, or assemble historical success envelopes directly.

**Engineering decisions**

- Shared query shape and projection vocabulary remain common; basis pair,
  materialization path, correspondence posture, and typed outcome remain
  capability-specific.
- Query owns historical planning and comparison assembly.
- Durable restore remains Milestone 10/11 scope, but it must implement the same
  admitted historical capability rather than add a new public journey.

**Open questions**

- None.

### Phase 8: Converge Preview, Mutation, And Workflow Orchestration

Make preview, writeback, effect, merge-adjacent, and domain workflow usage
declarative while preserving lower-runtime mutation and branch authority.
Consumers describe the workflow and inspect its outcomes; Query owns the
admission and orchestration chain.

**Relevant subsystems**

- preview and preview-live
- workflow and query-authored mutation declarations
- effect lifecycle and writeback
- domain capability contributions and lower-runtime routing

**Relevant APIs**

- preview workflow declarations and scoped preview bindings
- mutation/effect declarations
- writeback triggers and branch workflow declarations
- execution, promotion, denial, closeout, and aftermath outcomes

**Warnings**

- Declarative mutation does not transfer mutation, merge, or preview-session
  authority into Query.
- Workflow convenience must not collapse admission, execution, promotion, and
  aftermath into an uninspectable boolean result.
- Read-only and promotion-eligible preview states require distinct proof types.

**Test requirements**

- Compile-pass `facade::preview`, `facade::mutation`, `facade::workflow`, and
  `facade::domain` journeys implement their Phase 2 grammar contracts exactly;
  each transcript must exercise its real authority handoff and terminal rather
  than a placeholder or compile-only shell.
- Equivalent explicit internal orchestration and ordinary workflow declaration
  produce identical lower-runtime request, receipt, aftermath, and inspection
  identity.
- Unauthorized mutation, stale preview, cross-session promotion, and
  unsupported writeback deny before lower-runtime execution with no partial
  authority residue.
- Compile-fail tests prove consumers cannot construct admitted effects, lowered
  plans, promotion eligibility, or successful workflow envelopes.

**Engineering decisions**

- Ordinary workflows return structured outcomes preserving advisory,
  violation, execution, and aftermath distinctions.
- Domain-defined workflow vocabulary enters through contribution contracts and
  lowers through Query-owned orchestration.
- Lower runtimes remain the only executors of the authorities they own.

**Open questions**

- None.

### Phase 9: Make Results, Receipts, And Inspection Cohesive

Give consumers one coherent way to use successful results, typed stops,
operational receipts, and optional rich inspection without requiring them to
know which internal subsystem produced each artifact.

**Relevant subsystems**

- ordinary outcome
- result bundles and projection consumption
- causal inspection and diagnostics
- support posture and execution evidence

**Relevant APIs**

- ordinary success, advisory, violation, deferred, and unavailable outcomes
- projection facts and consumed authority
- operational receipts and causal inspection requests
- planning, execution, live, history, and policy diagnostics

**Warnings**

- A unified outcome cannot erase capability-specific facts or failure
  topologies.
- Receipts are evidence of completed work, not authority for new work.
- Rich diagnostics must remain policy-controlled and must not burden the
  ordinary hot path when not requested.

**Test requirements**

- A compile-pass `facade::inspection` journey implements its Phase 2 grammar
  contract exactly, begins from authentic outcome evidence plus scoped
  inspection basis, and never imports source phase artifacts.
- Every capability family exposes a common outcome navigation contract while
  preserving its family-specific success and denial payloads.
- Receipt-only, digest-only, and diagnostic-only attempts to authorize new
  execution or inspection fail at compile time or typed admission.
- Minimal and rich diagnostic policies produce identical operational outcomes
  and receipts while differing only in derived inspection materialization.

**Engineering decisions**

- Common outcome traits or navigation vocabulary unify only shared structural
  fate; concrete payloads preserve capability semantics.
- Projection consumption is the ordinary typed fact extraction lane.
- Inspection begins from an authentic outcome/receipt plus scoped inspection
  basis; it does not reopen source authority.

**Open questions**

- None.

### Phase 10: Quarantine Phase APIs And Contract The Ordinary Facade

Move phase constructors, transition functions, route selectors, report
builders, support inventories, and certification machinery out of ordinary
consumer reach. Preserve only capability declarations, sealed returned proofs,
typed outcomes, operational receipts, and explicit extension contracts.

**Relevant subsystems**

- facade export groups and crate-root exports
- internal phase modules
- certification and Consumer Kit namespaces
- support, inventory, migration, and compatibility surfaces

**Relevant APIs**

- canonicalize, bind, validate, admit, plan, lower, execute, and envelope
  transition functions
- public artifact constructors and success-envelope builders
- support-discovery, matrix, audit, and certification exports
- deprecated aliases and compatibility adapters

**Warnings**

- Moving an API to a deeper public module is not quarantine.
- Making constructors private while retaining public free-function transitions
  may still expose the same assembly framework.
- Tests cannot justify product-public visibility; certification gets an
  explicit non-ordinary namespace or crate-local access.

**Test requirements**

- Golden public API snapshots prove ordinary namespaces contain only the
  classified product surface and fail on any newly exposed phase transition.
- Compile-fail suites prove downstream crates cannot import or call internal
  phase machinery through crate root, deep modules, re-exports, aliases, or
  generic conversion traits.
- Internal parity tests prove facade contraction does not change canonical
  artifacts, outcomes, receipts, diagnostics, or cost counters.

**Engineering decisions**

- Public visibility is minimized at the defining module, not only hidden from
  re-export lists.
- Certification and Consumer Kit tooling remain visibly separate from product
  use and cannot mint operational authority.
- Removed ordinary phase APIs receive permanent prohibition and residue rows;
  no deprecated compatibility period survives milestone closure.

**Open questions**

- None.

### Phase 11: Cut Over Reference Consumers And Rewrite Discovery

Migrate Worth UI and a serious domain consumer to the capability-oriented
surface, delete their local Query orchestration, and make documentation teach
only the resulting present-tense mental model.

**Relevant subsystems**

- Worth UI runtime/query integration
- serious domain entry and workflow integration
- Query AI discovery and feature documentation
- examples, tests, fixtures, and consumer residue audits

**Relevant APIs**

- ordinary declaration and managed lifecycle journeys
- domain contribution contracts
- result/projection consumption and inspection
- consumer-local query coordinators and wrappers

**Warnings**

- A consumer wrapper that merely renames the old phase chain is not adoption.
- Discovery docs must not teach historical methods as alternatives or as
  banned folklore; obsolete methods should simply be absent.
- Test fixtures must model the ordinary API unless their explicit purpose is
  internal certification.

**Test requirements**

- Consumer-shaped end-to-end tests exercise representative read, live,
  historical/comparison, workflow, and inspection journeys using only ordinary
  facade imports.
- Residue tests inject local canonicalization/planning/execution and local
  subscription lifecycle assembly into consumer sources and prove exact
  detection.
- Documentation examples compile, and discovery-link tests prove every
  advertised capability resolves to a current ordinary API.

**Engineering decisions**

- Adoption evidence records deleted local types, helpers, transitions, deep
  imports, and backend decisions, not only new call sites.
- DX transcripts measure before/after ceremony and local decision count.
- `AI_README.md` remains a discovery document: it explains what exists, where
  to start, and which capability family to choose without historical API
  instruction.

**Open questions**

- None.

### Phase 12: Certify The Declarative Product Boundary

Run hostile certification over the complete ordinary experience and close only
when every supported capability follows one declarative entry, one
proof-preserving internal chain, and one coherent outcome contract.

**Relevant subsystems**

- all ordinary capability facade groups
- public API, prohibition, and residue certification
- reference consumers and DX transcripts
- runtime-backed hostile certification matrix

**Relevant APIs**

- final ordinary facade
- final extension contracts
- certification bundles and support matrix
- prohibition and consumer-residue registries

**Warnings**

- Happy-path brevity is insufficient; closure requires hostile composition,
  stale proof, denial localization, lifecycle, and cost evidence.
- A phase API with no current consumer remains a competing public framework and
  blocks closure.
- Store-backed absence is allowed; a runtime-specific ordinary contract is not.

**Test requirements**

- The final compiler-backed transcript matrix proves every Phase 2 grammar row
  resolves to an executable ordinary facade journey with the same namespace,
  ordered vocabulary, typed outcome/stop/next-action contract, context
  requirement, and ceremony ceiling; a frozen-but-unimplemented or drifted
  family blocks closure.
- A hostile matrix covers equivalent declaration convergence, cross-capability
  option rejection, cross-basis and stale-context denial, one-shot/live parity,
  historical ambiguity, preview/workflow denial, receipt non-promotion, and
  diagnostic-policy equivalence.
- Sabotage tests add a public phase constructor, deep transition, backend
  selector, success-envelope builder, compatibility alias, and consumer-local
  coordinator; each must fail a named enforcement layer.
- Exact counter tests prove invalid declarations deny before planning or
  lower-runtime contact and ordinary ergonomic lowering adds no work dependent
  on unrelated workspace, history, subscriber, or consumer size.
- Reference-consumer certification proves canonical result, receipt, authority
  identity, and diagnostics match the internal instrumented phase-chain oracle.

**Engineering decisions**

- Closure evidence composes facade snapshot, prohibition, residue, DX,
  reference-consumer, semantic parity, lifecycle, and bounded-work digests.
- Every shipped capability has one declared ordinary journey and one owner for
  each internal transition.
- Any missing ordinary capability discovered during cutover expands this
  milestone unless it requires a genuinely store-dependent implementation.

**Open questions**

- None.

### Phase 13: Freeze The Domain Authority Map And Installation Grammar

Inventory every way domain identity, configuration, invariants, operations,
contributions, declaration families, and lower-runtime posture currently enter
Query. Freeze one installation grammar before moving authority so the add-on
cannot preserve several old paths under new names.

**Relevant subsystems**

- application facade domain entry and configured handles
- domain-capability contribution authoring and materialization
- runtime builder registrations and graph-read operation registries
- ordinary `facade::domain`, workflow, and extension exports
- downstream domain helpers and reference consumers

**Relevant APIs**

- `WorthQueryApplicationFacade::domain(...)`
- `WorthQueryDomainEntryMarker` and `WorthQueryDomainOperatingContext`
- `worth_query_domain(...)`
- `WorthQueryRuntimeBuilder`
- `WorthQueryGraphReadOperationRegistry`
- domain-specific family helpers and public materializers
- proposed `WorthQueryDomainPackage<D>` grammar contract

**Warnings**

- Similar use of the word `domain` does not prove shared authority. Each path
  must be classified by who can mint identity, what runtime it belongs to, and
  whether it can affect execution.
- An inventory that records exports but not downstream construction and manual
  registry passing will miss the actual competing-authority seams.
- Physical IO adapters are not domain-semantic registration and must not be
  deleted merely because their names contain `adapter`.

**Test requirements**

- A source-backed authority inventory adds a new raw-string domain constructor,
  independent operation registry, or public materializer and fails with its
  exact defining and exporting locations.
- A reference journey proves the current typed handle, raw contribution, graph-
  operation, invariant, workflow, and runtime-builder paths remain separately
  classified rather than being merged by shared names.
- A seeded physical boundary adapter remains classified as legitimate external
  translation while a seeded semantic domain adapter is rejected.

**Engineering decisions**

- Every current path is classified as package input, canonical installation,
  installed-handle capability, derived index, diagnostic projection, physical
  boundary adapter, compatibility path, or prohibited competing authority.
- The final grammar is `declare package -> validate -> admit -> install ->
  obtain runtime-affine handle -> declare capability -> execute/inspect`.
- Phase 13 freezes the package fields and transcript owners. Later phases may
  refine internal representation but may not create a second installation
  grammar.
- The inventory records all existing consumers that must be cut over before a
  raw or independent path can be removed.

**Open questions**

- Exact public type names may change during implementation if the frozen
  authority and lifecycle grammar remains intact.

### Phase 14: Canonicalize One Typed Domain Package

Create the single declarative package that describes a domain before runtime
construction. Query owns canonical ordering, identity encoding, structural
validation, support admission, and the proof-bearing package progression.

**Relevant subsystems**

- domain identity and evidence identity
- application support and configuration admission
- invariant, graph-obligation, and graph-operation declarations
- declaration-family and contribution contracts
- domain package validation and admission

**Relevant APIs**

- proposed `WorthQueryDomainPackage<D>`
- proposed `WorthQueryValidatedDomainPackage<D>`
- proposed `WorthQueryAdmittedDomainPackage<D>`
- proposed `WorthQueryDomainIdentityDeclaration`
- proposed `WorthQueryDomainPackageValidationDenial`
- proposed `WorthQueryDomainPackageAdmissionDenial`
- replacement for `context_identity_digest() -> String`

**Warnings**

- A user-supplied string called a digest is still representation, not identity
  authority.
- Package canonicalization must reject duplicate or conflicting operation,
  invariant, obligation, declaration-family, and contribution registrations;
  last-write-wins would create order-dependent runtime meaning.
- The package may describe declarative semantics but cannot contain executable
  callbacks that bypass Query or lower-runtime planning.
- Package richness must not force rich diagnostics into the runtime hot path;
  diagnostic policy remains derived.

**Test requirements**

- Property and permutation tests prove equivalent package declarations with
  different insertion order produce the same canonical package identity,
  admitted contents, and installation plan.
- Collision tests prove the same domain identity cannot admit conflicting
  operation versions, accepted relation sets, invariant rules, declaration
  families, or contribution policies.
- Compile-fail tests prove consumers cannot construct validated or admitted
  package types, supply a canonical digest, or promote a marker/string into
  package authority.
- A hostile package containing a callback-shaped execution hook, untyped owner
  string, or unsupported lower-runtime claim denies before runtime construction
  and produces no partial package successor.

**Engineering decisions**

- Domain-owned code supplies typed identity fields and declarative definitions;
  Query canonicalizes and seals their identity.
- Canonical identity includes package schema version, typed domain identity,
  required capabilities/configuration, operating requirements, operation and
  invariant definitions, and declared extension families.
- Display names, diagnostic detail, and labels are excluded from authority
  unless the spec explicitly classifies them as semantic identity.
- Validation proves structural coherence. Admission proves current Query
  support. Installation remains a later runtime-authority transition.
- The admitted package is immutable and move-only where practical; derived
  indexes are not stored as package authority.

**Open questions**

- None.

### Phase 15: Install Domain Authority Into One Runtime

Make domain installation a runtime-construction transition. The runtime owns
the canonical installation registry and alone mints handles bound to its
authority identity and installation generation.

**Relevant subsystems**

- `WorthQueryRuntimeBuilder` and runtime construction
- runtime authority identity
- installed domain registry and lifecycle
- configured domain handles and admitted world basis
- support snapshot and installation diagnostics

**Relevant APIs**

- proposed `WorthQueryRuntimeBuilder::domain_package(...)`
- proposed `WorthQueryDomainInstallationRegistry`
- proposed `WorthQueryInstalledDomainArtifact<D>`
- proposed `WorthQueryInstalledDomainHandle<'runtime, D>` or equivalent
  runtime-affine carrier
- proposed `WorthQueryRuntime::domain(...)`
- proposed `WorthQueryDomainInstallationReceipt`

**Warnings**

- Binding only to configuration or a support snapshot is insufficient. The
  installed handle must prove one concrete runtime authority.
- Runtime affinity cannot be reconstructed from a runtime label, pointer,
  digest, or copied support report.
- Installation must be atomic across all package contents; partial registries
  may not survive a failed package.
- Runtime construction must not repeatedly clone full package definitions into
  each handle.

**Test requirements**

- Two runtimes built from equivalent packages expose equivalent installed
  semantic identity but distinct runtime-authority witnesses; a handle from one
  runtime cannot execute or inspect work on the other.
- Duplicate, conflicting, unsupported, or partially invalid package
  installation fails atomically and leaves no operation, invariant,
  contribution, declaration-family, or diagnostic residue.
- Compile-fail or sealed-constructor tests prove external code cannot mint an
  installed artifact, runtime-affine handle, installation generation, or
  installation receipt.
- Installation counters prove package validation and index construction occur
  once per installed package, while handle lookup does not rescan package
  contents.

**Engineering decisions**

- The runtime installation registry is authoritative for which domains exist
  in that runtime. Operation indexes, obligation indexes, and inspection views
  are derived and rebuildable from installed artifacts.
- Runtime authority identity and installation generation are typed witnesses,
  not public digest parameters.
- The initial product supports pre-runtime installation through the builder.
  Dynamic installation is not implied; adding it later would require an
  explicit quiescence, versioning, and invalidation contract.
- Existing application-facade support checks may preflight a package, but they
  cannot mint an executable domain handle.
- Installation produces a self-describing receipt with package identity,
  runtime authority reference, admitted definition counts, derived-index
  counts, warnings, and exact construction counters.

**Open questions**

- Whether the public handle uses a lifetime, a sealed runtime token, or both is
  an implementation choice; cross-runtime use must remain unrepresentable or
  deny before work.

### Phase 16: Compile Package Semantics Into Runtime Substrates

Lower each admitted package once into the runtime-owned invariant catalog,
graph-obligation catalog, graph-read operation registry, declaration-family
registry, and contribution policy. Execution consumes the installed products
without manual registry injection or semantic adapters.

**Relevant subsystems**

- runtime invariant and graph-obligation registration
- graph-read operation resolution and access planning
- declaration-family support and orchestration
- domain contribution eligibility/materialization policy
- runtime builder backend assembly

**Relevant APIs**

- `WorthQueryGraphObligationRegistrationCatalog`
- `WorthQueryGraphReadOperationRegistry`
- `WorthQueryGraphReadOperationRegistration`
- invariant registration artifacts and relational invariant catalogs
- proposed `WorthQueryInstalledDomainExecutionIndex`
- proposed package-to-runtime lowering plans and receipts

**Warnings**

- Passing a registry into an explanation or execution function is still
  consumer-owned orchestration even if the registry was validated.
- Domain operations declare how they lower into existing Query operators; they
  do not execute callbacks, scan graph truth during registration, or select
  lower-runtime backends.
- One package may populate several runtime substrates, but package installation
  remains the single authority and the derived substrates may not disagree.
- Physical schema, source, signal, and transport adapters remain visible
  runtime boundaries and are not package-owned domain semantics.

**Test requirements**

- A domain graph-read declaration resolves identically through ordinary read,
  access explanation, live maintenance, and internal oracle paths without any
  caller-supplied operation registry.
- A package conflict spanning invariant, obligation, operation, and declaration
  family definitions denies before runtime publication and leaves every
  substrate unchanged.
- Sabotage tests add a `*_with_operation_registry` ordinary path, direct
  registration callback, or operation-owner string restamp and prove facade,
  visibility, or residue certification rejects it.
- Exact counters prove installed operation lookup is indexed by canonical
  operation identity and does not perform full package or registry scans as
  unrelated domains and operations grow.
- Rebuild tests destroy every derived domain execution index, reconstruct it
  from installed artifacts, and recover identical resolution, denial, and
  diagnostic identities.

**Engineering decisions**

- Package lowering occurs before runtime publication and produces one sealed
  installation plan consumed by the builder.
- Graph-read operation registration becomes runtime state rather than an
  explicit parameter on ordinary explanation or execution APIs.
- Runtime substrate entries retain the installed domain identity and package
  version that authorized them.
- Cross-substrate conflict detection happens against canonical identities,
  accepted semantic scope, and support requirements rather than raw strings.
- Hot execution paths consume compact derived indexes and typed installed
  witnesses; they do not retain or traverse rich package diagnostics.

**Open questions**

- None.

### Phase 17: Bind Domain Contributions To Installed Authority

Move admission, support, invariant, workflow, continuity, aftermath, and
explanation contributions beneath the installed domain handle. Every
contribution inherits domain, package, runtime, target, and world identity
without accepting a raw domain string or rebuilding the contribution pipeline.

**Relevant subsystems**

- domain-capability authoring, eligibility, admission, and materialization
- contribution targets and target binding
- contribution-composed orchestration
- explanation, continuity, aftermath, workflow, and support artifacts
- projection consumption and lower-runtime boundary evidence

**Relevant APIs**

- proposed `WorthQueryInstalledDomainHandle::contributions()`
- proposed installed-handle target-bound contribution surfaces
- `WorthQueryContributionIntent`
- `WorthQueryContributionComposedOrchestrationInput`
- existing `worth_query_domain(...)` and public materialization functions to be
  removed from ordinary reach

**Warnings**

- Merely copying the installed domain key into the existing string surface does
  not bind authority.
- Contribution diagnostics may explain or narrow installed semantics but may
  not mutate the installed package or promote a diagnostic artifact into
  runtime authority.
- Different contribution categories retain their distinct payload, failure,
  support, and materialization contracts even though they share installed
  identity and lifecycle.
- Low-level materializers needed by internal certification do not remain
  product-public for that reason.

**Test requirements**

- Equivalent handle-bound contribution declarations produce the same canonical
  contribution and materialized artifact identities as the internal oracle,
  with installed domain, package, runtime, target, and world witnesses intact.
- Raw string, foreign handle, stale generation, target mismatch, and cross-world
  contribution attempts fail before evaluation or materialization and produce
  no partial successor.
- Compile-fail tests prove ordinary consumers cannot call
  `worth_query_domain(...)`, independently evaluate/admit/prepare/materialize a
  contribution, or restamp a target with installed identity.
- Mixed contribution outcomes preserve partial-admission truth while every
  retained result remains linked to the same installed authority.

**Engineering decisions**

- The installed handle supplies domain qualification and runtime affinity;
  callers supply only contribution meaning and a proof-bearing target reachable
  from that handle.
- Query owns evaluation, admission, preparation, canonical materialization,
  receipt assembly, and optional diagnostic projection.
- Contribution target types carry installed authority structurally rather than
  compare reporting digests after construction.
- The string-authored contribution facade is deleted after reference adoption;
  no alias or compatibility constructor remains.
- Certification-only raw transitions move behind crate-private or explicitly
  non-product test support.

**Open questions**

- None.

### Phase 18: Carry Installed Authority Through Execution And Rebind

Carry the runtime-installed domain witness through declaration, planning,
execution, live maintenance, continuation, recovery, receipts, and inspection.
Define the exact stale and rebind boundary when runtime generation, package
version, basis, policy, or lower-runtime evidence changes.

**Relevant subsystems**

- domain declaration and configured-handle orchestration
- ordinary read, live, workflow, mutation, and inspection journeys
- continuation preparation/execution and recovery
- basis, policy, tenant, and lower-runtime evidence
- runtime receipts and causal diagnostics

**Relevant APIs**

- installed-handle declaration and execution capabilities
- proposed `WorthQueryInstalledDomainAuthorityWitness`
- proposed `WorthQueryDomainRebindRequest` and typed rebind outcomes
- `WorthQueryPreparedContinuationExecutionReadmission`
- ordinary capability receipts and inspection requests

**Warnings**

- Installation is not blanket authorization for a later operation. Dynamic
  basis, policy, tenant, target, and lower-runtime eligibility remain separate
  proof transitions.
- Rebind must re-admit from authoritative runtime and package state; copying a
  newer generation number or package digest is forbidden.
- A valid handle cannot silently switch to a rebuilt runtime with equivalent
  configuration. Semantic equivalence and runtime authority are distinct.
- Diagnostics may observe the full chain but cannot repair or promote stale
  execution authority.

**Test requirements**

- An end-to-end domain journey preserves one installed authority from runtime
  lookup through declaration, admitted execution, receipt, projection
  consumption, and inspection, while minimal and rich diagnostics leave the
  operational result identical.
- Foreign-runtime, stale-installation, changed-package, changed-basis, changed-
  policy, and lower-binding drift each deny at their owning boundary with a
  distinct typed next action and exact zero later-phase work.
- Rebind from unchanged authoritative state converges on equivalent semantic
  identity while minting the current runtime/generation witness; rebind from
  changed package meaning cannot masquerade as the original installation.
- Live and continuation tests prove an installed handle cannot revive a closed
  resource, cross runtime authority, or reuse a prepared artifact after its
  installation generation becomes stale.

**Engineering decisions**

- Installed authority is an upstream proof carried into capability-specific
  admission; it does not replace those proofs.
- Every operational receipt records installed package identity and runtime
  installation witness as evidence, while neither is accepted alone as future
  authority.
- Rebind is one Query-owned admission path with typed causes and next actions;
  consumers cannot assemble a replacement handle.
- The diagnostic trace links package validation, installation, capability
  admission, execution, and outcome decisions without forcing rich artifacts
  onto the hot path.
- Existing application-facade configured handles are retired as executable
  authority once installed-handle adoption closes.

**Open questions**

- None.

### Phase 19: Contract The Installed-Domain Facade And Extension DX

Expose one discoverable installed-domain facade and eliminate the competing
public vocabularies. Keep Query generic while allowing downstream crates to add
typed domain-native vocabulary that lowers through the installed handle.

**Relevant subsystems**

- facade namespaces and crate-root exports
- application facade, runtime facade, and ordinary domain workflow naming
- public domain-capability types and materializers
- family-helper and downstream extension contracts
- public API snapshots, compile-fail boundaries, and docs

**Relevant APIs**

- final `facade::domain` installation and installed-handle grammar
- `WorthQueryRuntimeBuilder` domain package entry
- `WorthQueryRuntime` installed-domain lookup
- downstream-owned extension traits over installed handles
- existing application-facade domain entry, raw contribution, geometry helper,
  and low-level materialization exports to be contracted or removed

**Warnings**

- Renaming three domain entry systems into three nearby modules does not create
  one facade.
- An extension trait may provide domain vocabulary but cannot own canonical
  identity, registration, planning, execution, receipts, or diagnostics.
- Query-specific hard-coding of geometry or another product domain is not a
  generic extension mechanism.
- Public type count and method count are not DX. Ordinary journeys must expose
  the smallest surface that preserves real cost and failure distinctions.

**Test requirements**

- Golden facade snapshots prove one ordinary domain installation/lookup/
  capability grammar and reject reintroduction of application-facade handles,
  raw-string contributions, independent registries, or low-level materializers.
- A downstream-owned domain extension compiles without Query source changes and
  produces the same canonical installed capability artifacts as the generic
  handle journey.
- Compile-fail tests prove an extension cannot mint installed authority,
  construct a successful receipt, access internal package lowering, or bypass
  family-specific admission.
- Ceremony measurements prove setup and one representative domain journey use
  one package, one runtime installation call, one installed handle lookup, and
  no manual registry or semantic adapter assembly.

**Engineering decisions**

- `facade::domain` has one meaning: declaration and use of runtime-installed
  domain capability. Domain workflow operations live beneath that handle or in
  the ordinary workflow family without a second domain root.
- The ordinary facade exports declarations, admitted/installed handles, typed
  outcomes, receipts, inspection, and sealed extension contracts; transition
  machinery stays private.
- Domain-native convenience belongs in the downstream domain crate as typed
  extension vocabulary over the generic installed handle.
- Product-specific helper code currently housed in Query moves to its owning
  domain or is replaced by generic declarations; no compatibility helper remains
  solely to preserve the old call shape.
- External adapters remain in explicit physical boundary modules and are not
  re-exported as domain semantic setup.

**Open questions**

- None.

### Phase 20: Cut Over Consumers And Certify One Domain Authority

Migrate real consumers, rewrite discovery, delete every competing domain-
authority path, and certify that runtime installation is the only route from
domain setup to execution and diagnostics.

**Relevant subsystems**

- Worth UI and serious downstream domain consumers
- Query docs and AI discovery
- runtime/domain hostile certification
- facade, prohibition, residue, identity, lifecycle, and complexity audits
- package installation and derived-index rebuild tests

**Relevant APIs**

- final domain package and installed-handle facade
- final handle-bound contribution and extension contracts
- installation, execution, recovery, and inspection receipts
- certification bundles, prohibition registries, and consumer residue scans

**Warnings**

- A wrapper that hides `worth_query_domain(...)`, manual operation registries,
  or application-facade handles is not adoption.
- Passing tests after deleting an unused path does not prove the remaining path
  is runtime-affine; hostile foreign-runtime and stale-generation tests are
  mandatory.
- The original Phase 12 closeout remains valid only for its stated ordinary
  runtime-backed boundary. Add-on closure requires a new evidence section or
  closeout amendment with exact claims.
- Store durability, restart survival, and cross-process installed-domain reload
  remain Milestones 10 and 11 unless explicitly pulled forward with their full
  substrate.

**Test requirements**

- Reference-consumer end-to-end tests install a domain package, obtain a
  runtime-affine handle, run registered read and workflow operations, attach
  contributions, consume projection facts, and inspect one linked diagnostic
  chain without semantic adapters or manual registries.
- Hostile tests cover equivalent package convergence; conflicting package
  denial; foreign runtime, stale generation, wrong world, wrong target, and
  diagnostic non-promotion; derived-index destruction/rebuild; and minimal/rich
  diagnostic equivalence.
- Sabotage tests reintroduce raw domain strings, user-authored identity digests,
  application-facade executable handles, `*_with_operation_registry`, public
  materializers, Query-owned product-domain helpers, and consumer-local semantic
  adapters; every mutation fails a named enforcement layer.
- Exact slope tests prove installed handle lookup and operation resolution do
  bounded indexed work as unrelated domain/package/operation counts grow, and
  invalid authority performs zero planning, lower-runtime, and execution work.
- Documentation examples compile exclusively through the installed-domain
  facade, and every advertised stable method is callable from an external
  consumer crate.

**Engineering decisions**

- Add-on closure composes canonical package, installation atomicity, runtime
  affinity, registry integration, contribution binding, rebind, facade,
  consumer adoption, docs, residue, sabotage, and bounded-work evidence.
- No compatibility alias, raw-string constructor, independent registry
  parameter, public phase materializer, or executable application-facade handle
  survives closure.
- Consumer adoption records deleted wrappers, registries, semantic adapters,
  raw identity composition, and manual materialization calls.
- Documentation teaches the present installed-domain mental model and does not
  preserve historical setup as a banned alternative.
- The installed-domain boundary closes when Phases 13-20 pass hostile
  certification and record their exact non-claims. The milestone returns to
  fully closed only after Phases 21-30 also close native value authority.

**Open questions**

- None.

### Phase 21: Freeze Native Aspect Value Authority And Consumer Grammar

Inventory every Query-owned type, constructor, matcher, formatter, row carrier,
and facade export that creates, narrows, carries, identifies, or exposes aspect
values. Freeze one consumer grammar before changing implementation so replacing
JSON cannot quietly leave a second, smaller Query value system in authority.

**Relevant subsystems**

- Foundational aspect contracts, scalar values, struct values, and canonical
  wrappers
- Query authoring, mutation, schema view, predicates, canonicalization, and
  typed schema traits
- retained materialization, live rows, projection consumption, receipts, and
  inspection
- facade exports, documentation examples, downstream consumers, and residue
  audits

**Relevant APIs**

- `worth_foundational::facade::{AspectValue, ScalarAspectType,
  StructAspectValue, ContractValidatedAspectValue}`
- `WorthQueryAuthoredAspectValue`
- `ScalarPredicateValue` and `SchemaFieldKind`
- `ConsumedFieldValueFact`, retained scalar/row surfaces, and
  `WorthQueryNativeRow`
- aspect-value digest, equivalence, receipt, and diagnostic encoders

**Warnings**

- Absence of `serde_json::Value` does not prove native completion if Query owns
  a reduced scalar enum or a string encoding that decides semantic identity.
- A proof-bearing Query wrapper is valid only when it adds a real admitted,
  projected, retained, or consumed guarantee; shape duplication is not proof.
- Operator capability, value family, struct shape, absence, null, default, and
  clear are separate concepts and may not be collapsed during inventory.
- Test-only full-value constructors can hide a public product gap and must be
  classified as evidence rather than accepted as consumer support.

**Test requirements**

- A source-backed inventory seeds a Query-owned scalar variant, public
  stringifier, scalar-only struct branch, or misleading row carrier and fails
  with the exact definition, export, and consumer surface.
- A facade/consumer grammar matrix records every Foundational scalar family,
  representative struct shape, public authoring path, legal predicate families,
  projection form, refinement form, and final certification owner; any missing
  or multiply authoritative cell fails.
- A seeded proof wrapper with a public constructor that bypasses contract
  validation fails compile-boundary or visibility certification.

**Engineering decisions**

- Foundational value and contract types are the only native semantic
  vocabulary. Query wrappers are cataloged by the additional proof they carry.
- The final grammar is `author native intent -> validate against Foundational
  contract/mask -> carry admitted native value -> project retained native
  value/struct -> consume proof-bearing fact -> refine without reconstruction`.
- Query-owned capability descriptors may classify which operations are legal,
  but they derive from Foundational contracts and do not restate value shape.
- The inventory freezes removal or replacement owners for coarse schema kinds,
  coarse predicate operands, duplicate encoders, scalar-only materialization,
  and the native-row marker name.

**Open questions**

- Exact facade names remain implementation decisions; authority ownership and
  complete family coverage do not.

### Phase 22: Establish Portable Native Patch And State Readmission

Give Foundational one authority-honest representation for native aspect patches
and stored aspect state crossing serialization, process, or lower-runtime trust
boundaries. Portable artifacts carry exact candidate meaning but no validation
authority. A single contract-aware readmission front door must reconstruct the
sealed authoritative patch or state, or deny before mutation, restoration, or
publication.

**Relevant subsystems**

- Foundational aspect values, contracts, mutation masks, patches, and states
- canonical portable encoding and contract-revision identity
- lower-runtime handoff, durable checkpoint, replay, and restoration ingress
- denials for stale contracts, malformed values, and illegal operation shapes

**Relevant APIs**

- `AuthoritativeRecordAspectPatch` and `AuthoritativeRecordAspectState`
- proposed non-authoritative portable patch and state candidates
- contract registry lookup and Foundational value-validation front doors
- canonical export and readmission receipts or structured denials

**Warnings**

- A serde-capable type must not deserialize directly into validated or
  authoritative proof. Serialization round-trip preserves candidate meaning,
  not trust.
- Portable whole set, whole clear, field set, field clear, absence, explicit
  null, default, mutation mask, and contract revision are distinct grammar.
- A map of field values is not an authoritative patch. It loses operation kind
  and may not be used as the portable canonical form.
- Readmission resolves the current contract once per touched aspect and rejects
  all drift before any authoritative state is mutated.

**Test requirements**

- Complete patch round-trips cover whole set, whole clear, mixed field set and
  clear, scalar and nested struct values, entity references, explicit null,
  and legal absence without semantic projection.
- Hostile readmission tests cover stale revisions, missing contracts, malformed
  canonical scalar wrappers, undeclared struct fields, illegal masks, duplicate
  operations, and forged proof-shaped serialized payloads.
- State snapshot tests prove restore revalidates every entry and publishes no
  partial state when any entry fails.
- Compile-fail or visibility tests prove portable candidates cannot satisfy
  APIs requiring contract-validated values, authoritative patches, or admitted
  states.

**Engineering decisions**

- Foundational owns the portable grammar because only Foundational can preserve
  the complete native value and patch vocabulary without a parallel model.
- Canonical export is available from sealed authoritative artifacts. Readmission
  requires an explicit current-contract resolver and yields fresh authority.
- Proof-bearing patch and state types do not implement unguarded deserialization.
- Portable encoding is the common lower-runtime, durability, and replay handoff;
  consumers do not invent substrate-specific projections.

**Open questions**

- The concrete resolver trait and portable encoding format are implementation
  choices; proof erasure at export and fresh admission at ingress are not.

### Phase 23: Add Native Record Patch Ingress To Relational

Give Relational an explicit native record-aspect mutation lane for entities and
relations, including creation and update. The intent grammar preserves the
Foundational operation kind through admission and never reconstructs an
authoritative patch from a set-only physical field map.

**Relevant subsystems**

- entity and relation creation intents
- existing-record mutation targets and transaction intent serialization
- aspect field authoring compatibility and native patch admission
- authoritative patch application and publication

**Relevant APIs**

- `CreateIntent`, `EntityMutationIntent`, and relation mutation intent grammar
- `ExistingRecordTarget` and `RecordRef`
- `AspectFieldPatch` compatibility authoring
- Foundational portable and authoritative record-aspect patches

**Warnings**

- `AspectFieldPatch` remains a legitimate field-authoring input, but it is not
  the native Query authority lane and must lower through the same Foundational
  admission choke point.
- Entity and relation, creation and update, whole and field operations must not
  diverge into separately maintained semantics.
- Clear on creation must follow explicit Foundational absence law; it may not be
  silently treated as no-op authoring.
- A serialized transaction intent carries an untrusted portable candidate and
  earns fresh authority from the transaction's current schema basis.

**Test requirements**

- Entity and relation matrices cover creation and update with scalar, struct,
  reference, whole-set, whole-clear, field-set, and field-clear operations.
- Compatibility field authoring and explicit native patch authoring converge
  only when they express the same Foundational operation; whole-set identity is
  never collapsed into a field patch.
- Wrong target kind, stale schema, illegal creation clear, and malformed
  portable intent deny before record mutation and publication.
- Serialization round-trips prove intent portability does not confer admitted
  or authoritative proof.

**Engineering decisions**

- Relational accepts one record-targeted portable patch candidate and re-admits
  it against the transaction schema before planning.
- Entity and relation routing share record-aspect admission and differ only in
  target resolution and invariant scope.
- The existing field-authoring surface lowers into this lane; it cannot remain
  a competing commit authority.
- Commit publication retains the exact re-admitted
  `AuthoritativeRecordAspectPatch`.

**Open questions**

- Exact public constructor names remain implementation decisions; the unified
  record-targeted authority lane does not.

### Phase 24: Carry Native Patch Meaning Through Transaction Semantics

Integrate the re-admitted native patch into every transaction phase that can
change its fate or identity: normalization, ordering, merge, conflict handling,
validation, invariant and uniqueness scope, commit strategy selection, touched
records, counters, receipts, and publication. No later phase may rediscover
meaning from projected fields.

**Relevant subsystems**

- transaction planning, normalization, merge, and reconciliation
- optimistic and pessimistic commit strategies
- invariant, uniqueness, graph, and touched-scope validation
- canonical intent identity, receipts, counters, and publication

**Relevant APIs**

- mutation planning and merged-intent representations
- conflict and reconciliation policy
- authoritative patch application and published patch operations
- transaction digests, touched-record summaries, and work counters

**Warnings**

- Native patches cannot bypass merge or conflict policy merely because their
  values were validated earlier.
- Whole set and field patch touching the same aspect require an explicit
  deterministic conflict law; iteration order is not policy.
- Validation work is bounded by touched contracts, fields, records, and declared
  invariants. Native admission must not scan unrelated schema or graph state.
- Receipts and canonical identity include exact operation meaning and schema
  basis, not display text or a reconstructed final value alone.

**Test requirements**

- Permutation and batching tests prove deterministic ordering and identical
  outcomes for semantically equivalent native intent batches.
- Merge-hostility tests cover whole/field collisions, set/clear collisions,
  duplicate targets, stale preconditions, mixed entity/relation batches, and
  competing compatibility/native authoring.
- Strategy-parity tests prove every supported commit strategy applies and
  publishes the same authoritative patch meaning.
- Exact counters prove denial performs zero commit/publication work and valid
  work scales only with touched patch entries and declared validation scope.

**Engineering decisions**

- The re-admitted authoritative patch is the transaction's semantic authority;
  physical field deltas and final values are derived execution views.
- Merge produces one explicit admitted record mutation or a typed conflict. It
  never relies on last-write iteration behavior.
- Touched scope is derived from target plus patch operations before invariant
  scheduling and retained through publication and receipts.
- Every strategy consumes the same planned native mutation artifact.

**Open questions**

- None.

### Phase 25: Readmit Native State Across Durability And Replay

Cut proof-shaped aspect state out of checkpoint and replay serialization. Store
portable candidate state and patches, verify integrity and schema identity, and
re-admit all native values before rebuilt runtime state becomes visible.

**Relevant subsystems**

- checkpoint creation and restore
- transaction log and published patch replay
- runtime rebuild and derived-index reconstruction
- integrity manifests, schema snapshots, and restoration atomicity

**Relevant APIs**

- durable checkpoint record-aspect payloads
- replayed published patch operations
- Foundational portable state and patch readmission
- restore denials and rebuild publication barrier

**Warnings**

- A valid checksum proves byte integrity, not contract validity or authority.
- Restore may not copy `AuthoritativeRecordAspectState` directly from serde.
- Schema migration is not implicit coercion. Unsupported contract drift denies
  or enters an explicitly governed migration lane outside ordinary restore.
- Derived-index rebuild begins only after complete state readmission succeeds.

**Test requirements**

- Restart and replay tests cover every native scalar family, representative
  structs, references, whole and field clears, and mixed entity/relation state.
- Tamper, stale-contract, removed-field, changed-mask, malformed-value, and
  valid-checksum-but-invalid-meaning tests deny atomically before publication.
- Checkpoint and log replay converge on identical authoritative states,
  canonical identities, published operations, and derived indexes.
- Sabotage tests reintroducing proof-bearing deserialization or direct restored
  state insertion fail a source, trait, or boundary prohibition.

**Engineering decisions**

- Durability owns portable bytes and integrity; Foundational owns meaning
  readmission; Relational owns atomic rebuild publication.
- Restored authority is newly admitted against the active schema and runtime
  basis. It is not authority transported through serde.
- Replay and checkpoint restore share the same readmission primitive and may
  differ only in their surrounding integrity and ordering evidence.
- Migration remains an explicit future capability unless this milestone defines
  and certifies one; silent widening or defaulting is prohibited.

**Open questions**

- None.

### Phase 26: Admit The Full Native Value Vocabulary Through Mutation

Make ordinary mutation and workflow authoring capable of expressing every
Foundational scalar family and admitted struct aspect while keeping raw authored
values strictly below contract validation and authoritative mutation proof.
Ergonomic primitive constructors remain sugar over the same native path.

**Relevant subsystems**

- ordinary mutation, preview, workflow, and domain authoring
- aspect touches, field patches, struct aspect patches, and mutation masks
- desired-value validation, existing-truth probing, lowering, and receipts
- server or external compatibility ingress that lowers into native authoring

**Relevant APIs**

- `WorthQueryAuthoredAspectValue`
- `WorthQueryAspectMutationBuilder::{aspect, set_aspect}`
- Foundational `AspectValue`, `StructAspectValue`, validation front doors, and
  contract-validated artifacts
- admitted desired-aspect and authoritative patch/state transitions

**Warnings**

- Public acceptance of `AspectValue` is an authoring convenience, not authority;
  admitted mutation APIs must still require the sealed validation successor.
- Per-family constructors must delegate to one general native path rather than
  become independently maintained semantics.
- Whole-struct authoring and field-level patching are distinct operations and
  must preserve mutation-mask, absence, null, default, and clear law.
- External JSON ingress may lower through Foundational compatibility, but no
  ordinary native journey may route through JSON for convenience.

**Test requirements**

- A table-driven public-facade test authors every Foundational scalar family,
  executes mutation through the ordinary runtime-backed lane, and reads back
  the exact same native family and canonical payload without JSON or string
  conversion.
- Representative nested-field struct tests prove whole-struct set, legal field
  patch, clear, null, and mask denial preserve Foundational struct semantics and
  produce the same authoritative patch as an internal validation oracle.
- Compile-fail tests prove raw authored values and raw structs cannot satisfy
  APIs requiring validated, admitted, lowered, or executed mutation proof.
- Hostile tests prove wrong-family values, undeclared fields, invalid canonical
  wrappers, and incompatible references deny before lower-runtime mutation and
  leave zero partial authority residue.

**Engineering decisions**

- `WorthQueryAuthoredAspectValue` either becomes a transparent authoring wrapper
  over any Foundational native value or is replaced by an equally exhaustive
  authoring carrier. It does not define a parallel value enum.
- Struct authoring uses Foundational field keys, shapes, values, validation, and
  patch law. Query owns journey ergonomics and typed stops, not struct meaning.
- Validation emits a sealed Query mutation successor carrying the exact
  Foundational validated payload and target/mask proof into lowering.
- Public primitive conversions are optional sugar and must have canonical
  equivalence tests against explicit native authoring.

**Open questions**

- Whether whole-struct mutation is exposed directly or through a dedicated
  struct declaration builder is an implementation choice; both must lower to
  the same Foundational contract and patch authority.

### Phase 27: Derive Schema And Predicate Semantics From Native Contracts

Replace coarse Query-owned field and predicate value authority with
contract-derived operator capability. Equality, membership, ordering, numeric,
string, reference, and temporal predicates preserve exact native operands and
deny unsupported operations before planning.

**Relevant subsystems**

- schema view and typed schema macro generation
- predicate authoring, canonicalization, validation, templates, and live query
  declarations
- result-shape and ordering validation
- runtime and future store-backed predicate lowering

**Relevant APIs**

- `SchemaFieldKind`
- `ScalarPredicateValue`
- equality, set-membership, integer comparison, string-contains, presence, and
  ordering declarations
- `TypedEqualityField`, numeric/string/orderable capability traits, and schema
  field declarations
- Foundational `ScalarAspectType` and aspect contracts

**Warnings**

- A generic `Integer(i64)` operand loses width and excludes unsigned, big-int,
  decimal, rational, and canonical float meaning; widening it with more local
  variants would preserve the wrong ownership.
- Exact value family and legal operator family are separate. A timestamp can
  support equality and ordering without pretending to be a generic integer.
- Symbol-backed and raw interned strings may share selected string operations
  only if the Foundational contract defines their equivalence; Query must not
  infer it from display text.
- Canonical predicate identity must include exact native operand meaning and
  admitted schema basis, not a coarse kind label.

**Test requirements**

- A complete native-family matrix proves equality and membership preserve exact
  operands for every admitted scalar family, including unsigned widths,
  canonical floats, decimal, big-int, rational, UUID, dates/times, entity refs,
  content refs, and both interned-string representations.
- Operator-hostility tests prove string-only, numeric-only, orderable,
  reference, and temporal operations admit exactly the contract-declared
  families and reject incompatible combinations before planning or execution.
- Canonicalization tests prove equivalent ergonomic and explicit native
  operands converge, while values that differ by family, width, canonical
  payload, reference category, or schema basis remain distinct.
- Runtime/internal-oracle parity tests prove predicate evaluation never uses a
  second Query-specific conversion rule, and unsupported store pushdown later
  can deny without changing canonical predicate meaning.

**Engineering decisions**

- Schema views expose or wrap Foundational value shape plus Query capability
  posture; they do not own a replacement field-kind enum.
- Equality and membership operands retain native `AspectValue`. Specialized
  operators use proof-bearing capability declarations derived from the field's
  admitted contract.
- Typed schema generation derives Rust input types and operator traits from one
  contract mapping. Adding a Foundational family must force exhaustive compiler
  or matrix updates at every supported Query operator boundary.
- Planning consumes validated native predicates. Execution cannot rediscover
  type compatibility or coerce operands into a generic scalar.

**Open questions**

- None.

### Phase 28: Preserve Native Scalars And Structs Through Results

Complete the read side so retained rows, bridge materializations, live results,
projection facts, ordinary outcomes, and consumer refinement preserve the exact
Foundational scalar or struct payload that survived admission. Scalar leaf
projection remains available only as an explicit projection decision.

**Relevant subsystems**

- bridge and relational row-set extraction
- retained materialized rows and scalar facts
- live view handles, read results, derived artifacts, and projection
  consumption
- ordinary outcomes, consumed authority, and typed consumer refinement

**Relevant APIs**

- `ContractValidatedAspectValueView::{Scalar, Struct}`
- `ConsumedFieldValueFact` and `ConsumedProjectionFactSet`
- retained row and retained scalar value surfaces
- `WorthQueryNativeRow` and `WorthQueryLiveView<T>`
- proposed native scalar/struct refinement and shape-mismatch denials

**Warnings**

- Rejecting a validated struct merely because a fact family was implemented as
  scalar-only is an incomplete projection contract, not an invalid value.
- Flattening a struct into dotted strings or an untyped field map creates a new
  object model. Field projection must retain Foundational keys and canonical
  paths and remain linked to the source struct contract.
- Returning `&AspectValue` is authority-honest but insufficient DX when every
  consumer must rebuild the same contract-aware type mismatch logic.
- A phantom marker named `NativeRow` must not be exported or documented as if
  it contains materialized values.

**Test requirements**

- End-to-end bridge, relational, live, retained-derived, and ordinary-read tests
  round-trip every scalar family and representative struct values into consumed
  authority without JSON, display text, or family collapse.
- Whole-struct and projected-leaf parity tests prove explicit leaf projection
  returns native `AspectValue` keyed by Foundational paths while complete struct
  consumption retains `StructAspectValue` and the originating contract shape.
- Typed refinement tests prove successful extraction for every supported Rust/
  Foundational carrier and structured shape-mismatch denials containing expected
  family, actual family, field path, source, and projection authority.
- Compile-fail and facade tests prove a row marker cannot be inspected as data,
  a raw row cannot be promoted into consumed authority, and consumers cannot
  construct successful consumed facts.

**Engineering decisions**

- Projection facts remain proof-bearing Query artifacts whose payload view is
  the exact Foundational scalar or struct value admitted by the projection.
- Query supplies ergonomic borrowed refinement over native values; refinement
  does not clone, stringify, revalidate, or reconstruct authority.
- Scalar-only fact families declare that requirement in their contract and
  return a typed shape denial. General field/result consumption supports both
  scalar and struct payloads.
- `WorthQueryNativeRow` becomes a real consumer-visible retained row with typed
  identity and native value access, or the phantom marker is renamed to an
  internal role-specific marker and excluded from ordinary product teaching.
- Extraction and refinement errors are typed and composable rather than
  `String`.

**Open questions**

- Whether complete structs are exposed as one consumed fact, a row field view,
  or both is an implementation choice; both forms must preserve one source
  contract and canonical value identity.

### Phase 29: Centralize Native Value Identity And Contract The Facade

Move canonical aspect-value identity to one Foundational-owned encoding basis,
make every Query digest or equivalence context compose that basis with explicit
domain separation, and contract the public facade around the exact native types
and proof-bearing Query wrappers. Delete local semantic encoders and coarse
value exports.

**Relevant subsystems**

- Foundational canonicalization and aspect-value identity
- Query canonical query, mutation intent, retained materialization, projection
  fact, receipt, replay, and certification identity
- ordinary read, mutation, live, workflow, domain, and foundation facade groups
- prohibition, public API snapshot, and source residue enforcement

**Relevant APIs**

- Foundational canonical aspect-value encoding or digest-basis front door
- Query aspect-value digest helpers in mutation, projection consumption,
  retained scalar values, and certification oracles
- Query facade exports for authored values, native values, structs, contracts,
  consumed facts, rows, and typed refinement denials
- canonical identity and equivalence receipts

**Warnings**

- One canonical value encoding does not mean one undifferentiated digest.
  Mutation, projection, retained-row, and receipt identities retain explicit
  typed domain separation around the same value bytes.
- `Debug`, display strings, JSON, and human-readable diagnostic text are not
  canonical value encodings.
- Re-exporting a native type under a Query name is justified only when the name
  adds proof or role meaning; an alias that implies alternate semantics is
  prohibited.
- A facade contraction that leaves broad `From`, public constructors, or deep
  public modules can still permit weaker-value promotion.

**Test requirements**

- Cross-context golden tests prove each native scalar and struct payload uses
  identical Foundational value bytes inside mutation, retained-row, projection,
  replay, and certification identities while typed domain separators keep the
  enclosing artifacts distinct.
- Mutation tests seed formatting drift, `Debug` changes, separator injection,
  interned-string representation differences, numeric-width differences, and
  bytes/content-ref confusion; canonical identities remain stable or diverge
  exactly according to native meaning.
- Residue tests reject production `match AspectValue` encoders outside the
  canonical owner, coarse public scalar/schema enums acting as value authority,
  scalar-only compatibility branches, misleading row exports, and string-based
  native refinement errors.
- Compile-fail tests prove ordinary consumers cannot construct validated,
  retained, consumed, or executed value proofs from raw native carriers,
  canonical bytes, digests, labels, or terminal projections.

**Engineering decisions**

- Foundational exposes the canonical value identity basis. Query owns only
  artifact-specific domain separation and proof composition around it.
- All Query identity paths delegate to that basis; certification oracles use an
  independently implemented semantic model only where independence is required
  for proof, not another production encoder.
- Ordinary facade namespaces expose the smallest native/proof surface required
  for authoring and consumption. Internal canonicalization and transition types
  remain non-public.
- Permanent prohibitions cover Query-owned duplicate value algebras, manual
  value stringification for identity, raw-to-proof promotion, accidental struct
  flattening, and phantom-row consumer teaching.

**Open questions**

- The canonical basis may be bytes, a typed canonical term, or a sealed digest
  input artifact. It must preserve every Foundational family and struct shape
  without allocating presentation text on ordinary hot paths.

### Phase 30: Cut Over Consumers And Certify Native Value Closure

Migrate real consumers, rewrite discovery, delete every reduced or reconstructed
value path, and certify one exact native value story from public authoring to
consumer use. Close only when all Foundational families and representative
structs survive every supported runtime-backed Query journey.

**Relevant subsystems**

- Worth UI and a serious domain consumer
- ordinary mutation, read, live, preview, workflow, projection, and inspection
  journeys
- Query docs and `AI_README.md`
- facade, residue, prohibition, replay, parity, complexity, and sabotage suites

**Relevant APIs**

- final native authoring and contract-derived predicate grammar
- final native result, row, struct, consumed-fact, and refinement surfaces
- canonical value identity receipts and typed denials
- Consumer Kit adoption and source audits

**Warnings**

- Converting a native value to a consumer-local enum immediately after Query is
  not adoption when that enum reconstructs Foundational meaning or authority.
- A scalar-only reference workload cannot certify struct support or the full
  value matrix.
- Documentation must teach the present native mental model without preserving
  coarse Query enums or JSON-era methods as historical alternatives.
- Store-backed absence remains allowed; differing runtime/store public value
  contracts do not.

**Test requirements**

- A generated native-value journey matrix covers every Foundational scalar
  family plus representative structs across authoring, mutation, one-shot read,
  live delivery, retained materialization, projection consumption, typed
  refinement, replay, receipt identity, and inspection where each capability is
  admitted.
- Reference-consumer tests replace local value decoding, integer/string
  coercion, struct reconstruction, and manual type mismatch errors with the
  ordinary native facade, recording exact deleted residue.
- Hostile tests cover wrong family, wrong width, unsigned/signed confusion,
  symbol/raw string distinction, bytes/content-ref distinction, stale schema,
  invalid struct field, scalar/struct mismatch, cross-contract reuse, digest
  collision attempts, and diagnostic non-promotion.
- Sabotage tests reintroduce a coarse scalar enum, test-only-only native write
  path, local value encoder, scalar-only bridge extraction, phantom native row,
  raw-to-proof conversion, and consumer-local reconstruction; each fails a
  named compiler, facade, prohibition, residue, or parity layer.
- Exact counters prove native refinement is constant time per selected value,
  predicate admission is proportional to declared predicates and contract
  fields, invalid values perform zero planning/lower-runtime work, and no
  operation scans unrelated schemas, rows, packages, or consumers.

**Engineering decisions**

- Closure evidence composes family/struct coverage, public facade snapshots,
  compile-fail proof, canonical identity parity, consumer deletion, docs,
  residue, sabotage, replay, and bounded-work digests.
- Support rows distinguish genuinely unsupported operators or source families
  from missing native value plumbing. The latter blocks closure.
- Phases 21-30 close as one add-on program after installed-domain authority;
  no consumer cutover may declare success while authoring, predicates, structs,
  rows, identity, or refinement remains on a competing value model.
- Milestone 10 consumes these exact canonical native predicate, mutation,
  result, row, and projection contracts for store-backed parity.

**Open questions**

- None.

## Must Ship

- capability-oriented ordinary facade grammar
- one typed declarative authoring entry into canonical query meaning
- declarative basis, tenant, policy, and relationship-context handoff
- admitted-query execution for read, collection, and count-aggregate families
- framework-owned live resource lifecycle
- coherent historical, diff, correspondence, preview, mutation, workflow, and
  inspection journeys
- cohesive outcomes, receipts, projection consumption, and optional diagnostics
- internalized phase transitions and permanently prohibited legacy phase APIs
- Worth UI and serious-domain adoption with measured DX evidence
- hostile facade, residue, parity, lifecycle, and bounded-work certification
- one typed canonical domain package spanning identity, operating requirements,
  invariants, graph obligations, graph-read operations, declaration families,
  and contribution policy
- atomic pre-runtime installation with one runtime-owned domain registry and
  runtime-affine installed handles
- Query-owned canonical domain identity encoding with no consumer-authored
  digest authority
- package-compiled runtime registries and indexes consumed automatically by
  execution, live maintenance, explanation, and inspection
- installed-handle-bound contributions, continuation, recovery, receipts, and
  diagnostic traceability
- one contracted installed-domain facade, downstream-owned typed extension DX,
  consumer cutover, and hostile single-authority certification
- one Foundational-owned native aspect value vocabulary across Query mutation,
  predicate, schema, retained-row, projection, receipt, and inspection surfaces
- full ordinary authoring and runtime-backed round-trip support for every
  Foundational scalar family and representative admitted struct aspects
- contract-derived operator capabilities and exact native predicate operands
  with no coarse Query-owned scalar or field-kind authority
- proof-bearing native scalar/struct result consumption, typed refinement, and
  an honest retained-row or explicitly internal row-marker contract
- one Foundational canonical value identity basis with Query artifact-specific
  domain separation, duplicate-encoder deletion, facade contraction, consumer
  cutover, and hostile native-value certification

## Must Preserve

- Query owns expression, canonicalization, admission, planning, orchestration,
  result shaping, and public developer experience
- lower runtimes retain truth, mutation, merge, reactive scheduling,
  persistence, and transport authorities
- basis, tenant, policy, branch, preview, and relationship authority remain
  explicit and proof-bearing
- family-specific cost, failure, correctness, and lifecycle distinctions remain
  visible
- diagnostics and support posture remain derived and non-authoritative
- advanced domain contribution remains possible without internal pipeline access
- canonical semantics, receipts, replay identity, and exact counters do not
  change merely because the ordinary facade becomes simpler
- domain packages remain declarative; they cannot contain callback-shaped
  execution, backend selection, or lower-authority promotion
- runtime installation authority remains distinct from semantic package
  equivalence, support snapshots, reporting identities, and diagnostic evidence
- installed artifacts remain authoritative while runtime indexes, lookup tables,
  support views, and diagnostics remain derived and rebuildable
- true storage, schema, source, signal, and transport adapters remain visible
  physical boundaries rather than being conflated with semantic domain setup
- Foundational remains the sole authority for aspect value families, struct
  shape, canonical wrappers, keys, paths, contracts, validation, and canonical
  value identity; Query adds intent, capability, proof, shaping, and DX only
- raw authored native values remain lower authority than validated, admitted,
  retained, consumed, and executed proof-bearing successors
- struct meaning survives until an explicit contract and projection select
  leaves; scalar convenience cannot flatten or reject complete structs
- operator, result, and refinement ergonomics do not coerce, stringify,
  reconstruct, or silently narrow native value meaning
- store-backed execution, durable installed-domain reload, and cross-process
  continuation remain owned by Milestones 10 and 11

## Acceptance Evidence

- ordinary consumers perform no manual canonicalize/bind/validate/plan/lower/
  execute/envelope progression
- ordinary consumers cannot construct or advance internal phase artifacts
- equivalent declarations converge across ergonomic forms and reference
  consumers
- invalid combinations deny before expensive construction or lower-runtime
  contact with typed next-action context
- live resources are registered, tracked, resumable where supported, and
  deterministically disposable
- historical, comparison, preview, workflow, and inspection journeys preserve
  their distinct proof and denial semantics
- public API snapshots, compile-fail tests, permanent prohibitions, residue
  audits, docs, and support matrices agree on the same surface
- reference journeys show materially lower ceremony without semantic defaults,
  backend choices, or authority decisions moving into consumer code
- no compatibility aliases, callable phase entrypoints, consumer-local Query
  coordinators, or backend-shaped ordinary APIs remain
- equivalent domain packages canonicalize and install identically independent
  of authoring order, while conflicting packages fail atomically
- only a concrete runtime can mint an installed domain handle, and foreign or
  stale handles fail before planning or lower-runtime work
- domain operations resolve from runtime-owned indexes without consumer-supplied
  registries or package scans
- all domain contributions inherit installed authority and cannot begin from a
  raw domain string or public materialization phase
- derived installation indexes rebuild from installed artifacts without
  changing resolution, denial, receipt, or diagnostic identity
- one facade snapshot, external-consumer compile suite, prohibition registry,
  and residue audit agree on the installed-domain surface
- every Foundational scalar family and representative struct value round-trips
  through admitted public authoring and consumption with exact native identity
- schema and predicate capability derive from Foundational contracts; wrong
  operators and wrong value families deny before planning or execution
- complete structs and explicitly selected scalar leaves preserve the same
  source contract and cannot be confused or reconstructed from field strings
- all Query identity contexts delegate to one Foundational value basis while
  retaining explicit typed domain separation
- facade, compile-fail, residue, sabotage, docs, Consumer Kit adoption, replay,
  parity, and bounded-work evidence report zero competing value authorities

## Sequencing Notes

This milestone belongs after 9.12 because authority must be sealed before the
ordinary experience can safely hide internal phase transitions. It belongs
before Milestone 10 because store-backed execution must implement an existing
admitted capability contract, not create a second set of consumer-visible
planning and execution journeys.

Phases 13-20 follow the completed Phases 1-12 because the ordinary capability
grammar must exist before domain extension can be installed into it. They still
belong before Milestone 10: runtime installation authority, canonical domain
identity, operation-registry ownership, and contribution binding are runtime-
semantic foundations. Deferring them would force store-backed execution to
inherit and multiply today's competing domain entry paths.

Phases 21-30 follow installed-domain closure because the final consumer-native
value grammar must cover domain-installed authoring, execution, projection,
and inspection rather than certify only pre-installation examples. They still
precede Milestone 10 because store-backed predicate pushdown and result
materialization must inherit one exact native value contract.

The add-on phases are strictly ordered. Phases 13-20 freeze the domain authority
inventory, create the canonical package, install runtime authority, compile
execution indexes, bind contributions and rebind, contract the facade, and
certify consumer adoption. Phases 21-30 then freeze native value authority,
complete write authoring, derive schema and predicate capability, preserve
native result/struct consumption, centralize canonical value identity and
contract the facade, and close with hostile consumer certification.

Milestone 10 may add backend-specific internal plans, pushdown evidence, and
fallback diagnostics. It may not change the declaration grammar, context
handoff, managed lifecycle, or outcome contract frozen here.

## Store Dependency

This milestone is not blocked on `worth-store`. Runtime-backed execution is
sufficient to freeze and certify the product boundary. Store-backed parity,
durable restore, saved artifact survival, and durable continuations remain
Milestones 10 and 11, but they must enter through the contracts established
here.

Runtime-installed domain packages are likewise not blocked on `worth-store`.
Phases 13-20 require atomic in-process runtime construction, runtime authority
identity, and generation-safe handles. Persistence, restart-stable package
reload, and cross-process handle restoration are not claimed here and must be
implemented later without changing the installed-domain grammar.

Native value closure is not blocked on `worth-store`. Phases 21-30 operate over
Foundational contracts and the existing runtime-backed Query substrate. Any
missing Foundational canonical identity front door is in-scope substrate work.
Milestone 10 must reuse the resulting predicate, mutation, row, result, and
projection contracts unchanged for store-backed execution and pushdown parity.
