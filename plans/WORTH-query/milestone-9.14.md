# Milestone 9.14: Installed Operation Semantics, Semantic Aspect Correspondence, Conditional Signal Authority, And Bound Downstream Authority

> Platform carriage successor:
> [WORTH Signal Milestone 13.1](../WORTH_signal/milestone-13.1-plan.md)

## Goal

Make the safe downstream Query path mechanically complete. Stable domain views,
workflows, and operation semantics are installed once as typed Query
definitions. Query also authors exact portable dependencies on
Relational/Foundational truth aspects and the conditional-node intent that
governs whether operation and workflow nodes are eligible to evaluate.
Relational preserves authoritative whole-aspect, field, endpoint, and lifecycle
change meaning; the runtime bridge owns the installed correspondence from those
semantic changes into exact node-local Signal aspect slots; and Signal alone
owns invalidation versions, scheduling, and the runtime evaluation decision.
One installed operating-world root owns graph
participation and is the only authority-bearing entry point. Typed operation-
family facades
borrow from that root; none can construct an independent runtime or graph
world. One non-detachable runtime-affine operation capability then carries
declaration, installation, cross-domain and cross-graph binding, basis,
workflow progression, execution trace, re-execution and cert-only replay
posture, Signal evaluation eligibility and outcome, reversal posture, lineage,
consumption, native refinement, dependency impact, sharing, and lifecycle
proof through the consumer journey without permitting callers to reassemble
authority from individually valid parts. The same capability mints its
consumer support contract, managed observation leases, invalidation deltas,
and collection/window delivery so downstream runtimes never reconstruct Query
posture, operation meaning, graph adapters, stage ledgers, replay catalogs,
undo scope, identity lineage, impact closure, shared work, or patch semantics
locally.

The ordinary authoring path must therefore be capable of declaring a real
conditional computed or workflow node without importing Signal internals. It
names semantic truth dependencies by exact aspect contract, binding, field
mask, locality, and relevant change meaning rather than by numeric Signal slot.
Those dependencies, the portable condition, trigger inputs,
comparator/equivalence posture, temporal or on-demand posture, and declared
output relationship cross the bridge once, are installed into the actual
Signal graph and aspect layout, and return to Query as proof-bearing decision
evidence. Query effect conditions remain a distinct downstream-effect contract
and cannot substitute for node evaluation conditions.

## Why This Milestone Exists

Milestone 9.13 makes ordinary Query declarative, runtime-installed, and
Foundational-native. Downstream integrations can still make serious mistakes
if stable domain operations rebuild schema, selectors, result shape, workflow
graphs, stage ledgers, replay catalogs, reversal scope, or lineage meaning in
extension methods; if operation families expose independent public roots; if
Query-authored semantic dependencies lose field, relation-endpoint, lifecycle,
contract-revision, or change-kind meaning on the Relational publication path;
if a bridge-owned string is treated as an installed Signal aspect; if slot
coalescing silently claims exactness; if Query-authored nodes silently degrade
to `Always`, if a raw Signal condition
or custom string is smuggled through the facade, if the bridge or Query
re-decides Signal eligibility, if comparator meaning is omitted from operation
identity, if skipped or reverted-clean work is reported as computed; if
hidden adapters translate between graph authorities; if independently valid
domain or graph capabilities are treated as atomic cross-domain authority; if
consumers maintain local dependency graphs
or recompute policy; if equivalent projections create duplicate execution and
subscription resources; or if separately exposed definitions, executions,
consumed facts, basis identities, support families, lifecycle receipts,
invalidation surfaces, collection patch posture, and reporting digests must be
coordinated correctly. This milestone closes those remaining assembly and
duplication hazards before Query exports its final runtime-backed
certification oracle and before Store integration multiplies the same boundary
across physical providers. It does so generically: Query owns operation-semantic
progression and runtime authority, `worth-proof` owns reusable proof progression
law, domain packages own domain algorithms and correspondence policy, and Store
continues to own durable journals, checkpoints, and restart.
One logical operating graph remains the default; separate graph participation
exists only for genuinely separate authority, lifecycle, or provider
boundaries.

## Governing Summaries

- `MENTALITY.md` protects foundation-first correctness under adversarial
  pressure; semantic aspect correspondence and conditional node meaning are
  therefore repaired across authoring, Relational publication, bridge
  installation, Signal execution, and Query re-entry before later replay,
  invalidation, sharing, cost, or patch authority is allowed to depend on it.
- `arch_laws.md` protects proof-carrying phase progression and preserved
  identity authority; portable semantic dependency intent, authoritative truth
  delta, installed aspect correspondence, conditional lowering, Signal
  decision evidence, and Query consequence are separate typed phases, and
  representations cannot promote themselves.
- `composition_laws.md` protects one predictable semantic responsibility per
  file and function; installed definitions, graph participation, entry-root
  authority, binding, support, compatibility, refinement, dependency impact,
  sharing, lifecycle, invalidation, collection delivery, identity, and
  accounting remain separately named responsibilities.
- `domain_structure_laws.md` protects physical boundaries that reveal
  authority, lifecycle, truth source, and dependency direction; Query owns
  portable dependency intent, Relational owns authoritative change meaning,
  the bridge owns installed correspondence and crossing, Signal owns local
  aspect versions and evaluation truth, and the public facade is narrower than
  the internal capability topology.
- `perf_laws.md` protects cost honesty and bounded execution breadth; truth
  delta lowering, aspect-correspondence lookup, slot fan-out and widening,
  installed operation lookup, native access, dependency closure, sharing
  admission, condition admission, eligibility checks, comparator resolution,
  invalidation, window resolution, and patch delivery expose exact counters and
  cannot hide broad scans, eager compute, or duplicate equivalent maintenance.
- `worth-proof/README.md` protects one reusable progression law: recipes,
  artifacts, proof sets, checked outcomes, assumption bases, freshness,
  readmission, concrete witness spending, structural facts, and honest fixed-
  shape collections are reused behind sealed owner-specific types rather than
  rebuilt inside Query, Bridge, Relational, or Signal. It does not become a
  runtime scheduler, evaluator, or generic public authority.
- `worth-foundational/docs/README.md` and `FOUNDATIONAL_README.md` protect the
  strongest-owner rule and one shared boundary vocabulary: aspect contracts,
  values, masks, locators, canonical basis/comparison, portable identities,
  evidence, provenance, lineage, support, and counter-backed performance
  artifacts are reused wherever those meanings cross crates, while stronger
  Query, Relational, Bridge, and Signal types remain authoritative.
- `WORTH_query_roadmap.md` protects one canonical runtime-backed Query meaning
  that later providers must preserve; 9.14 closes downstream consumer safety
  after 9.13 and before Milestone 13 exports provider-independent oracles.
- CAD `ROAD.md` protects foundation-first kernel sequencing; the entry,
  publication, lineage, and graph-participation substrate must close before
  later B-rep, boolean, feature, modification, UI, and AI roads depend on it.
- CAD `ARCHITECTURE.md` protects one authority-bearing entry root, one logical
  graph by default, projection-based ordinary handoffs, promotion-on-reference,
  and cert-only replay; 9.14 must make those claims mechanically expressible.
- CAD `BOUNDARIES.md` protects routability from real task sentences; recovery
  therefore means declared reversal or compensation, while cross-graph work
  routes through installed participation and a bound operation.
- Road 1 protects the dependency order from pure meaning through Query
  adoption, ordinary entry/publication, and certification; adapter grammar and
  the root handle precede ordinary multi-family execution.
- CAD `NAMING.md` protects authority-bearing band meaning; multiple
  `worthy-entry-*` family crates may improve routing but cannot imply multiple
  public roots or a generic adapter band.
- CAD `GLOSSARY.md` protects the distinctions agents must preserve: entry root
  versus entry family, publication versus execution trace, ordinary re-
  execution versus replay, and explicit graph participation versus hidden
  bridging.

## Adversarial Constraint

Given any number of independently valid installed-domain handles, graph
adapters, operation-family facades, stage receipts, portable or locally rebuilt
operation definitions, execution completions, replay scopes, reversal scopes,
lineage candidates, basis
projections, consumed fact receipts, support projections, dependency labels,
equivalence tokens, diagnostic digests, invalidation labels, collection
patches, portable semantic aspect dependencies, authoritative truth-delta
targets, aspect-correspondence witnesses, raw Signal aspects or masks, portable
condition declarations, raw Signal conditions, condition or comparator labels,
bridge lowering artifacts, evaluation decisions, and
lifecycle artifacts from foreign runtimes, stale generations, or
different declarations, a consumer must be unable to combine them into a
second operating-world root, atomic multi-domain or multi-graph operation
capability, legal workflow advancement, ordinary access to cert-only replay,
reversal or compensation authority, persistent naming
authority, operational projection capability, compatibility proof, shared
execution, observation lease, replacement admission, rebind admission, native
value access, impact closure, invalidation delta, collection window, patch
delivery, or lifecycle transition.

Raw `worth-proof` carriers, caller-selected authority markers, or Foundational
projections, evidence, identities, support summaries, and digests must be unable
to satisfy a stronger owner-specific admission merely because their fields or
canonical comparison agree. Conversely, an owner-specific wrapper must not
reimplement the reusable proof progression or shared boundary meaning that
those crates already own.

A truth aspect identity, contract revision, field mask, binding, change meaning,
surface, locality, installed Signal node/aspect slot, precision posture,
condition, or comparator change; a foreign or stale correspondence/lowering;
a detached Signal decision; or an evaluation outcome from another node, graph,
runtime, operation, run, basis, snapshot, or attempt must never be accepted as
equivalent operation authority. Unsupported correspondence or conditions deny
before Signal graph mutation or execution. Ineligible,
deferred, suppressed, and reverted-clean nodes perform and report exactly the
work their Signal-owned outcome allows; they cannot mint computed output,
publication, stage completion, impact, invalidation, or delivery evidence.

Conditional execution must remain a narrowing of the exact graph authority
already admitted for the bound operation, never a second way to obtain graph
power. Every declared conditional truth read must be a typed subset of one
declared operation graph-read role; every possible conditional output, touch,
or effect must be a typed subset of the operation's declared touch/effect
closure; and every realized read, touch, effect, and invalidation consequence
must be a subset of that declared conditional scope. The exact bound graph
participation, runtime generation, basis, graph adapter, and graph-touch
obligation evidence must survive correspondence, Signal lowering, evaluation,
and Query re-entry. A condition or Signal decision may narrow realized work but
can never widen admitted capability. Missing or foreign predecessor graph
authority denies before bridge lowering, Signal mutation/evaluation, or Query
consequence work even when operation identities, aspect meaning, slots, and
reporting digests otherwise match.

Equivalent installed definitions and bound capabilities must converge on one
canonical meaning and, where Query admits sharing, one framework-owned
execution resource with independently disposable consumer leases. The admitted
path must remain proportional to declared dependency, consumption, affected-
capability, window, and patch width. Foreign, stale, detached, ambiguous, and
representation-only inputs must deny before planning, sharing, lower-runtime
contact, native refinement, invalidation, or lifecycle work.

## Product Decision Lock

- Query exports a complete installed operation semantic closure, not separate
  catalog, workflow, replay, reversal, lineage, or authority ingredient lists
  that consumers must assemble correctly.
- Stable domain-native views, workflows, and operations are installed package
  definitions; downstream extension traits may expose vocabulary but may only
  resolve those definitions through an installed handle.
- One installed operating-world root is the sole authority-bearing ordinary
  entry point. Operation-family facades are typed borrowed views over it, not
  independent runtime constructors or alternate roots.
- One logical graph is the default. A separate graph participates only through
  a named, sealed, runtime-affine adapter when authority, provider, tenant, or
  lifecycle is genuinely separate.
- Cross-graph operations consume typed source projections and produce typed
  target effects. They are atomic only when Query proves shared commit
  authority; otherwise their installed contract exposes compensation and
  partial-failure posture.
- The installed operation read/touch declarations are portable semantic inputs
  to Query's existing graph-read access planning and Graph Touch Obligation
  Authority. They are not a second graph enforcement engine. Binding and
  lowering derive or retain the existing proof-bearing graph capabilities, and
  later conditional phases can only spend and narrow those capabilities.
- Installed operation definitions declare dependency, workflow, read, touch,
  effect, invariant, replay, reversal, lineage, result, failure, and cost
  posture as one semantic closure. Portable definitions never contain
  executable callbacks.
- The strongest owning type remains authoritative at every layer. Query,
  Relational, Bridge, and Signal types may retain or wrap Foundational and
  `worth-proof` carriers, but no shared lower-level carrier replaces a stronger
  runtime-owned capability, receipt, patch, decision, or correspondence.
- Foundational is the mandatory shared vocabulary for aspect contracts,
  identities, revisions, field paths, masks, locators, native values, canonical
  basis/comparison, boundary identities/evidence, and counter-backed performance
  exports. Query and adjacent runtimes must reuse those surfaces rather than
  introduce semantic mirrors.
- `worth-proof` is the mandatory substrate for proof-bearing phase progression,
  checked outcome topology, concrete authority/capability witness spending,
  assumption basis and freshness, boundary weakening/readmission, and honest
  fixed-shape collections. Query-specific opaque types retain owner meaning and
  prevent raw generic proof carriers from becoming an alternate facade.
- Public governed transitions require concrete sealed Query, Relational,
  Bridge, or Signal authority marker types inside `worth-proof` witnesses. A
  caller-selected generic `AuthorityMarker` or `CapabilityMarker` cannot satisfy
  operational admission.
- `worth-proof` does not become Query's workflow scheduler, Signal evaluator, or
  dynamic graph runtime. Foundational does not become Query state, Relational
  patch authority, Bridge correspondence authority, or Signal hot aspect state.
- Query owns a portable conditional-node authoring contract covering declared
  semantic truth dependencies, trigger inputs, evaluation condition, comparator/equivalence posture,
  temporal or on-demand posture, and output relationship. It does not expose
  Signal's runtime node objects or let Query decide runtime eligibility.
- A Query semantic truth dependency names the exact Foundational aspect key,
  identity and contract revision, Relational binding and source surface,
  projection/change mask, locality, and relevant change meaning. It never names
  or derives authority from a raw Signal aspect slot.
- Relational owns the authoritative interpretation and publication of whole-
  aspect, field-level, endpoint, structural, lifecycle, and opaque changes. A
  bridge adapter may not discard field precision, turn opaque change into a
  lifecycle fallback, or widen a source without an explicit typed posture.
- The runtime bridge owns a runtime-affine installed semantic-aspect
  correspondence binding Query intent and exact Relational truth meaning to an
  exact Signal graph, node, local aspect slot, partition, runtime generation,
  and precision posture. Stable names, digests, or equal numeric slots have no
  operational authority.
- Relational and Signal aspects are not assumed to be one-to-one. Installation
  may admit one-to-many, many-to-one, or derived-only Signal relationships only
  through explicit exact or declared-widening contracts; ambiguity, unsupported
  precision, and slot exhaustion return typed denials before graph mutation.
- Query node evaluation conditions and Query effect conditions are different
  contracts with different owners and lifecycle positions. Neither converts
  into or defaults the other.
- Dependency-version meaningfulness, condition eligibility, post-compute output
  equivalence, and artifact-reuse equivalence are distinct contracts. An
  implementation may share typed policies only where their declared semantics
  genuinely coincide; one generic comparator cannot silently decide all four.
- Query alone validates portable conditional meaning and lowers its declaration
  into a neutral Bridge conditional contract. The runtime bridge admits that
  neutral contract against installed correspondence and installs the exact
  Signal node contract. Signal alone mints evaluation
  eligibility, deferral, suppression, computation, and semantic-cleanliness
  evidence; Query consumes that evidence without restamping it.
- Multi-domain operation authority is bound atomically. A tuple of separately
  valid domain capabilities is not an operation capability.
- Workflow stages and run traces are Query-minted proof progression; domain
  packages retain ownership of stage algorithms and semantic lowering.
- Re-execution, cert-only replay, and reversal are distinct. Ordinary entry
  may retry or re-execute through the installed operation; trace-driven replay
  and reconstruction are exposed only to the certification audience; reversal
  executes a declared inverse or compensation posture and may be explicitly
  unavailable.
- Persistent naming and identity continuity consume typed lineage evidence;
  raw identities, strings, digests, coordinates, rendered values, and debug
  representations cannot authorize continuity.
- Operational compatibility is decided by Query and returns typed proof or a
  typed denial; consumers do not compare identity components.
- Native access is derived from the admitted declaration and Foundational
  contract; callers do not scan facts or reinterpret paths.
- Consumer support and required Query surfaces are minted once by Query from
  the bound capability; consumers do not rebuild them from hooks or digests.
- Query emits capability-bound invalidation deltas; consumers decide their own
  consequences but do not reinterpret which Query meaning changed.
- Query compiles semantic dependency roles and impact closure from the
  installed definition, admitted plan, conditional trigger inputs, Signal
  decision evidence, native contracts, and installed domain invariants;
  consumers do not maintain a second dependency graph.
- Equivalent capability sharing is admitted by a Query-owned equivalence
  contract and represented by framework-managed consumer leases; matching
  digests or cloned packets never justify reuse.
- Collection identity, ordering, cursors, windows, and incremental patches are
  Query-shaped authority; UI virtualization owns mounting policy only.
- Live promotion, replacement, rebind, and disposal remain attached to the
  capability that established their installation and execution authority.
- Reporting and diagnostic identities are observable derivations with zero
  operational authority.
- Raw `EvaluationCondition`, `NodeEvaluationConfig`, custom-condition strings,
  comparator names, bridge request fields, and Signal reporting summaries are
  not public Query authoring or operational authority.
- Exact public type names may be refined during implementation, but the single
  capability, proof progression, authority ownership, and prohibition grammar
  are not open.

## Shared Substrate Adoption Rule

Implementation must search and adopt the strongest existing `worth-foundational`
or `worth-proof` type before introducing a new cross-crate semantic carrier. A
new owner-specific type is justified only when it adds runtime affinity,
installation identity, sealed authority, lifecycle ownership, or a stronger
invariant that the shared type intentionally does not own. In that case it
retains or projects the shared type; it does not duplicate its semantics under a
Query-, Bridge-, Relational-, or Signal-prefixed name.

- Phases 1, 4, 7-9, 11, 14-16, 18-24, and 26 reuse Foundational aspect contracts,
  values, field paths, masks, locators, canonical basis/comparison, portable and
  boundary identities, and their exact strength distinctions wherever those
  meanings appear.
- Phases 1-3, 5-6, 8-13, 16-19, 23-25, and 26 reuse `worth-proof` recipes,
  artifacts, proof sets, checked outcomes, assumption bases, freshness and
  readmission, concrete witness spending, and structural collection facts for
  actual admission or phase progression. A phase that merely declares portable
  meaning does not manufacture a proof solely to claim reuse.
- Phases 5-6, 8-14, 20, 22, 24, and 26 reuse Foundational evidence, provenance,
  lineage/continuity, receipt, and boundary-attachment vocabulary for facts that
  cross owner boundaries. The stronger runtime receipt, trace, patch, decision,
  or capability remains owner-specific and non-reconstructible from those
  attachments.
- Phases 3, 6, 9, 18, and any later fixed-shape authority set reuse
  `NonEmpty`, `CanonicalVec`, `UniqueVec`, fixed-arity composition, and relevant
  structural proofs only after the owning admission has checked the claimed
  property. Sorting, deduplication, tuple shape, or nonempty input syntax is not
  proof.
- Phases 4 and 22, plus every phase exposing counters, reuse Foundational support
  profiles, performance claims, counter specifications, counter-backed receipts,
  and materialized reports at their documented strength. Query's embedded
  boundary-local counter snapshot remains the execution fact; a materialized
  report cannot be readmitted as one.
- Bridge and Signal continue to own dynamic mapping, slot allocation, graph
  state, scheduling, evaluation, and runtime decisions; Query continues to own
  installed operation admission, consumer contracts, workflow topology,
  lifecycle resources, impact, patches, and public authority. Shared substrate
  adoption must never pull these runtime engines into Foundational or
  `worth-proof`.

Shared-substrate use is triggered by meaning, not by phase authors remembering
an optional library. The following triggers are mandatory throughout 9.14:

- When a transition can honestly distinguish denial, deferment, stale input,
  rebind requirement, and execution failure, it uses the applicable
  `worth-proof` checked-outcome topology and preserves every reachable category.
  It must not flatten them into `Result`, booleans, strings, or one local error
  enum. Categories that cannot occur are not fabricated merely to fill the
  lattice.
- When an artifact's authority depends on an installation, runtime, graph,
  snapshot, attempt, or provider basis, that basis participates in its
  authoritative carried form through `AssumptionBasis` and the applicable
  freshness/weakening/readmission law. Rechecking copied identity fields is not
  a substitute.
- When progression has real semantic stages, it reuses `Recipe`, `Artifact`,
  proof-set, readiness, and concrete witness-spending law behind sealed owner
  wrappers. A single already-admitted value is not wrapped in fake stages just
  to mention `worth-proof`.
- When nonempty, canonical, unique, disjoint, or fixed-arity shape is required
  for authority, the owning admission checks the real collection and then
  carries the corresponding `worth-proof` form. A sorted or deduplicated `Vec`
  remains an implementation detail until that admission succeeds.
- When meaning crosses crate, provider, serialization, support, or comparison
  boundaries, it retains the strongest applicable Foundational aspect, value,
  path, mask, locator, identity, canonical-basis, evidence, provenance,
  lineage, support, receipt, or performance vocabulary. Owner-local strings,
  integer identities, digest-only equality, and semantic mirror enums are not
  acceptable intermediate authority.
- When work is claimed to have executed, the path retains owner-local exact
  counters and materializes Foundational counter-backed evidence at the
  boundary where shared performance truth is required. Claims, policy
  admission, canonical bundles, supporting rows, reports, and certified bundles
  retain their documented strength and cannot be substituted for one another.

The Phase 7-11 substrate repair has this minimum adoption ledger:

- Phase 7 stores exact Foundational contracts, bindings, identities, revisions,
  paths, masks, locators, native condition parameters, and canonical comparison
  material inside Query's portable declaration. Query installation uses checked
  `worth-proof` progression; portable authoring alone does not claim installed
  proof or runtime freshness.
- Phase 8 derives from the stronger Relational canonical patch while retaining
  Foundational patch/contract/path/locator meaning losslessly. Publication uses
  checked `worth-proof` outcomes for the non-success states the crossing can
  actually produce; a generic envelope or matching digest cannot become the
  successful Relational publication artifact.
- Phase 9 uses the full applicable `worth-proof` admission substrate: staged
  resolution/lowering/readiness, concrete Bridge and Signal witnesses, composite
  assumption basis and freshness, checked outcomes, and proven nonempty,
  canonical, unique, or disjoint target/source collections after validation.
  Foundational canonical comparison establishes portable semantic sameness but
  never installed slot authority.
- Phase 10 spends the Phase 9 owner witness through real
  resolve/lower/admit/ready/execute progression. Bridge and Signal wrappers add
  pair-bound runtime and evaluation invariants; they do not recreate a private
  recipe or outcome system beside `worth-proof`.
- Phase 11 carries Signal decision evidence into Query under the exact current
  assumption basis. Any process, provider, serialization, or reconstruction
  crossing uses explicit boundary weakening and concrete Query readmission;
  Foundational provenance and receipts describe the crossing without minting
  current Query authority.

For every later phase, this ledger is applied by the semantic triggers above,
not by copying the Phase 7-11 type choices mechanically. An implementation may
omit a listed substrate capability only when the phase has no corresponding
meaning, and the boundary review records that reason. "The local type was
simpler" and "the digest is equivalent" are not valid reasons.

Nominal adoption does not satisfy this rule. A phase has under-utilized the
shared substrate when it keeps a parallel owner-local semantic or proof model
on the authoritative path and uses Foundational or `worth-proof` only to:

- compute or compare a digest, diagnostic label, serialization form, or report;
- decorate a locally decided outcome after admission, mutation, or execution;
- wrap one generic witness while reimplementing recipe, proof-set, basis,
  freshness, checked-outcome, weakening, or readmission law beside it;
- convert into the shared type only at a facade or bridge edge and then discard
  its strength distinctions for owner-local booleans, strings, or catch-all
  enums; or
- manufacture `NonEmpty`, canonical, unique, or fixed-shape carriers before the
  owning admission has actually proved those structural facts.

Adoption is complete only when the strongest applicable shared type participates
in the authoritative stored artifact, canonical identity, comparison, checked
transition, or carried evidence for the meaning it owns. The owner-specific type
must add a named stronger invariant, keep its constructor sealed, and retain the
shared artifact or losslessly project it at the exact ownership boundary. Later
code must consume that strengthened owner type; it may not return to raw shared
ingredients and reassemble authority. This is a relevance rule, not a feature-
count rule: use every shared capability needed to preserve the phase's real
semantics, but do not invent proofs, evidence, or wrappers for meanings the phase
does not have.

Every implementation-batch boundary review for this milestone records, for each
new semantic or proof-bearing type: the strongest existing shared owner, the
specific shared capabilities used on the authoritative path, the stronger
owner-only invariant being added, the point where raw lower-strength inputs are
rejected, and any applicable shared capability intentionally not used with a
reason. An unexplained owner-local duplicate or report-only adoption is a phase
finding, not deferred cleanup.

Each phase review must identify every newly introduced proof, witness, basis,
freshness, outcome, identity, aspect, path, mask, locator, value, evidence,
receipt, provenance, lineage, support, and performance type. The implementation
is incomplete until each is either an existing shared type, an owner-specific
wrapper that visibly strengthens one, or an explicitly documented owner-only
runtime meaning. Tests prove the owner boundary and strength distinction; they
do not recertify `worth-proof` or Foundational's own internal laws.

## Phase Plan

### Phase 1-6 conditional retrofit boundary

Phases 1-6 establish the unconditional installed-operation, graph, binding,
support, execution, consumption, and workflow authority chain. Their conditional
clauses describe the final shape that Phase 11 must retrofit after Phases 7-10
have installed portable condition meaning, authoritative truth publication,
semantic-aspect correspondence, and Signal decision authority. Before that
retrofit exists, a definition may retain canonical conditional meaning, but a
conditional operation or stage must deny binding explicitly; it may not execute
as `Always`, mint a consumer contract, or enter either Phase 5 or Phase 6
progression. A Phase 1-6 implementation is complete for its ordered slice only
when the unconditional path is complete and the conditional path is an honest
typed stop. Phase 11, not an early placeholder or local evaluator, removes that
stop and completes the conditional requirements written into Phases 1-6.

### Phase 1: Installed Domain Operation Definitions

This phase extends the installed domain package from capability, invariant,
obligation, graph-operation, declaration-family, and contribution meaning to a
complete installed operation semantic closure. A definition declares its typed
identity and version, parameter contract, native aspect contract and projection
mask, canonical Query intent, result shape, collection/order posture, portable
conditional-node declarations, required installed capabilities, optional typed
workflow-stage graph, graph-read
contract, touch and effect contract, invariants, replay posture, reversal or
compensation posture, lineage and promotion-on-reference contract, derived-
publication contract, terminal result and failure classes, cost contract,
support, and lowering family. The runtime validates and installs the definition
once and returns only an installed typed operation capability.

Every portable aspect contract, native value, field path, mask, locator,
canonical basis/comparison input, and portable identity in this closure reuses
the corresponding Foundational type. Definition -> validated candidate ->
installed capability is a checked `worth-proof` progression behind opaque Query
types; only the Query installer can spend the concrete installation witness and
mint runtime-affine operation authority.

Generic ad hoc Query authoring remains legitimate. A downstream operation with
stable domain meaning, however, cannot repeatedly construct schema, selectors,
aliases, result shape, or mutation paths inside an extension method and call
that local construction domain-native authority.

**Relevant subsystems**
- domain package validation, canonical identity, and runtime installation
- installed-domain operation indexes and handle lookup
- ordinary read, collection, mutation, and workflow declarations
- Foundational-native schema contracts, masks, selectors, and result shapes
- derived publication, projection consumption, and sparse identity promotion

**Relevant APIs**
- `WorthQueryDomainPackage<D>` and installed-domain handle surfaces
- installed graph-read operation definitions and declaration-family definitions
- ordinary read, collection, mutation, and workflow authoring
- Query-owned portable node condition, trigger-input, comparator/equivalence,
  temporal/on-demand, and output-relationship declarations
- proposed typed domain view definition, workflow definition, installed
  operation key, and installed operation capability
- existing graph-read, graph-touch, authority-scoped effect, identity-
  evolution, and deterministic execution contracts
- proposed derived-publication and promotion-on-reference roles in the
  installed operation semantic closure
- Foundational aspect/value/path/mask/locator/canonical identity vocabulary and
  `worth_proof::Recipe`/`Artifact`, checked outcomes, assumption basis, and
  concrete installation witnesses behind Query-owned definition types
- runtime-bridge conditional-node lowering registration and installed Signal
  node contract identity

**Warnings**
- An extension trait that hides local reconstruction of a stable query is still
  a competing declaration authority even when its method name is ergonomic.
- Installing a closure, callback, backend plan, or executable consumer object
  would move volatile execution mechanics into portable domain meaning.
- A distinct Rust newtype whose only semantic payload is a string label is not
  by itself a typed contract. Parameter, result, collection, touch/effect,
  lineage, publication, failure, cost, and support contracts must carry the
  typed meaning consumed by their owning Query admission or lowering path, or
  an explicit typed `NotRequired` posture; canonical identity participation
  cannot be their only behavior.
- A workflow catalog, replay catalog, reversal catalog, and lineage catalog
  maintained independently would create multiple semantic authorities even if
  every catalog refers to the same operation name.
- Defaulting an omitted conditional-node declaration to Signal `Always`,
  accepting a raw `EvaluationCondition`, or treating Query effect conditions
  as evaluation gates silently changes installed operation meaning.
- Condition families, trigger inputs, comparator/equivalence posture, temporal
  or on-demand posture, and declared outputs participate in canonical identity
  and conflict detection. A custom string or diagnostic label is not a typed
  domain condition.
- The semantic closure declares what must be true; domain implementations
  register or lower algorithms separately and cannot smuggle executable
  callbacks into portable definitions.
- A raw operation name plus version is not installed authority. Portable
  definitions become operational only when resolved by the installing runtime.
- A locally mirrored aspect, identity, canonicalization, proof, freshness, or
  installation-outcome type is not an acceptable shortcut. Raw Foundational or
  `worth-proof` carriers cannot be passed through the public facade to obtain an
  installed operation capability.
- Query must not hard-code product domains. Domain crates author typed generic
  definitions against Query's contracts and may expose their own vocabulary.
- Installed domain operation definitions are versioned product semantics, not
  user-saved query instances or durable provider records.
- Completion is not publication. Downstream stages consume a separately minted
  derived-publication receipt rather than reopening the operation trace.
- An output does not acquire durable graph identity merely because execution
  produced it; promotion requires the declared reference posture and typed
  lineage evidence.

**Test requirements**
- Adversarial definition convergence test: equivalent package view and workflow
  definitions install to identical canonical meaning regardless of declaration
  order, ergonomic authoring path, or derived-index rebuild.
- Adversarial conflict and reconstruction test: duplicate versions with
  different schema, masks, parameters, result shape, support, lowering, or
  workflow meaning fail package installation atomically, while a locally
  reconstructed lookalike cannot resolve as the installed operation.
- Adversarial authority test: raw names, copied definition fields, foreign-
  runtime handles, stale installation generations, and portable definitions
  cannot execute, bind, mutate, or mint an installed operation capability.
- Adversarial lookup-bound test: resolving one installed operation performs one
  indexed lookup independent of unrelated domains and operations and performs
  no planning or lower-runtime work on a miss.
- Adversarial semantic-closure test: removing or drifting a required capability,
  workflow edge, graph-read, touch/effect, invariant, replay, reversal,
  lineage, promotion, publication, condition, trigger input, comparator,
  temporal/on-demand posture, result/failure, or cost role changes
  canonical operation meaning and conflicts atomically rather than producing a
  partially installed operation.
- Adversarial callback test: portable operation definitions cannot carry or
  recover executable callbacks, backend objects, or consumer-owned stage
  implementations.
- Adversarial contract-operability test: every installed semantic contract is
  consumed by its real Query admission, lowering, publication, or settlement
  path. Replacing an executable contract with a same-named string newtype must
  fail the public transcript or hostile runtime suite rather than continuing
  to pass because only canonical digests observe it.
- Adversarial condition-identity test: two otherwise identical operation or
  stage definitions with one-field condition, trigger, comparator, or output-
  relationship drift conflict atomically; declaration order and derived-index
  rebuild preserve identical admitted meaning.
- Adversarial substrate-adoption test: Foundational canonicalization and the
  Query wrapper produce identical portable definition meaning, while raw proof
  artifacts, caller-selected markers, validated-but-uninstalled candidates, and
  same-field local mirrors cannot resolve or execute as an installed operation.

**Engineering decisions**
- The package is the canonical portable source for stable domain operation
  meaning; the installed operation capability is the runtime-affine authority.
- Portable conditional-node declarations are Query-owned semantic inputs. The
  executable Signal node contract and its lowering provider remain volatile,
  runtime-affine installation state.
- A typed downstream extension method may resolve an installed operation and
  improve vocabulary, but it cannot construct or override Query meaning.
- Definitions describe semantic intent and lowering family, never a physical
  provider plan or executable callback.
- The operation definition is the single canonical source for workflow,
  replay, reversal, lineage, promotion, publication, dependency-impact, and
  certification semantics; later phases compile derived indexes and proof
  artifacts from it.
- The canonical execution trace and the canonical derived publication are
  distinct artifacts: certification may inspect the trace, while ordinary
  downstream stages consume the publication through projection consumption.
- Installed view and workflow indexes are derived, rebuildable, and powerless
  without the retained installed package artifact and runtime authority.
- Query owns definition admission and installed operation authority;
  Foundational owns the shared portable meanings and `worth-proof` owns the
  generic checked progression used underneath that admission.

**Open questions**
- Exact generic type names and whether view and workflow definitions share a
  common sealed definition envelope are implementation choices; their distinct
  result, cost, lifecycle, and failure contracts must remain visible.

### Phase 2: Installed Graph Participation Adapters

This phase makes every genuinely separate graph authority enter the operating
world through one named, sealed, runtime-affine participation adapter. The
default remains one logical graph with multiple domain schemas, aspects,
partitions, and packs. A separate adapter is justified only when authority,
provider, tenant, basis lifecycle, or commit ownership is actually separate;
domain taxonomy alone does not justify graph federation.

A portable graph-participation definition declares the graph's typed role,
observation and projection contracts, touch and effect capabilities, identity
and lineage posture, locality and budget contract, commit posture, compensation
requirements, and failure topology. It contains no executable callback. The
owning runtime installs the corresponding lowering provider and mints the only
operational adapter capability. Query can then prove which installed
operations that graph may satisfy without exposing provider internals or
allowing an application to bridge graphs directly.

Portable participation meaning reuses Foundational basis, identity, locality,
provenance, lineage, evidence, and boundary vocabulary. Participation
definition -> validated provider pairing -> installed adapter is a checked
`worth-proof` progression with an exact assumption basis and concrete owning-
runtime witness; the generic artifacts cannot act as the installed adapter.

When a Query-authored conditional node reads or produces meaning across that
boundary, its bridge lowering retains this exact installed graph adapter. The
adapter does not interpret the condition or comparator, and matching graph
labels cannot substitute for bound graph authority in Signal decision evidence.

**Relevant subsystems**
- operating-world installation and installed domain packages
- graph-read planning, projection consumption, graph touches, and effects
- basis, policy, tenant, branch, preview, identity, and lineage authority
- commit admission, compensation posture, and provider boundary envelopes

**Relevant APIs**
- installed domain handles and lower-runtime capability routing
- graph-read access plans and projection-consumption contracts
- graph-touch obligations and authority-scoped effect plans
- proposed graph-participation definition, installed graph adapter,
  participation denial, and adapter support receipt
- Foundational boundary/provenance/basis vocabulary and `worth-proof` checked
  installation progression behind the stronger installed adapter capability

**Warnings**
- Do not create one adapter per domain merely because topology, geometry,
  physics, or UI use different vocabulary. Multiple domains may inhabit one
  logical graph without a federation boundary.
- A generic callback, object-safe "adapter bag," raw provider pointer, or
  application-owned bridge would hide authority and failure semantics.
- A provider interface that exposes only a semantic key, while retaining its
  implementation as opaque `Any` state, is not an operational lowering
  provider. The installed adapter itself must retain the exact callable typed
  lowering capability used by binding and execution; binding may not build a
  second private provider bundle beside the installed adapter.
- An adapter may lower declared reads and effects; it may not translate raw
  identities, copy authority, or invent cross-graph correspondence.
- Shared IDs, matching digests, event-bus topics, and equal payloads do not
  establish graph compatibility or causal continuity.
- Physical persistence and transport remain provider and Store concerns; this
  phase installs semantic participation and runtime-affine routing only.
- Conditional evaluation callbacks and condition dispatch do not belong in a
  graph adapter. Query-to-Signal lowering is a separate bridge authority that
  consumes the adapter's installed graph binding.
- Foundational boundary evidence or a raw proof witness may describe or prove a
  lower-level fact, but neither can be accepted directly as graph participation
  authority or shared commit authority.

**Test requirements**
- Adversarial single-graph convergence test: domain packages installed into one
  logical graph resolve without artificial adapters and produce the same
  operation meaning as the direct single-graph path.
- Adversarial adapter convergence test: equivalent participation definitions
  and provider installations mint equivalent support and routing meaning
  regardless of declaration order or derived-index rebuild.
- Adversarial construction test: raw graph handles, provider objects, copied
  adapter fields, matching digests, and foreign or stale adapter capabilities
  cannot join an operating world or satisfy an operation requirement.
- Adversarial authority test: an adapter cannot widen observation, mutation,
  identity, lineage, basis, tenant, or commit authority beyond the capability
  installed by the owning runtime.
- Adversarial cost test: adapter resolution is bounded indexed work over the
  operation's declared graph roles and does not scan unrelated graphs, domains,
  providers, consumers, or diagnostics.
- Adversarial provider-use test: sabotage each registered provider's real
  observation, projection, touch/effect, and commit-admission entry point and
  prove the bound operation invokes the exact installed adapter once. A suite
  in which every provider can be replaced by a semantic-key-only empty object
  without changing outcomes is a milestone failure.
- Adversarial conditional-binding test: a lowered conditional node or Signal
  decision bound to a foreign, stale, or semantic-key-lookalike graph adapter
  denies before condition resolution or node execution with exact zero later
  work.
- Adversarial substrate-boundary test: equivalent Foundational projections and
  checked installation results converge, but copied evidence, raw recipes or
  witnesses, caller-selected marker types, and matching portable identities
  cannot install, substitute, or invoke an adapter.

**Engineering decisions**
- Graph adapters are explicit authority boundaries, not convenience wrappers.
- Portable participation meaning and executable lowering remain separate;
  Query installs and binds them but does not own graph semantics.
- One logical graph is the ordinary case. Multi-graph participation is an
  admitted capability used only when a real authority boundary exists.
- Cross-graph correspondence and lineage must arrive as typed domain evidence;
  the adapter transports that evidence but cannot manufacture it.
- The owning runtime controls adapter admission; `worth-proof` supplies its
  progression law and Foundational supplies shared boundary meaning without
  becoming an adapter registry or provider runtime.

**Open questions**
- Exact public names remain open until the single-root facade transcript proves
  they are discoverable without making adapters ordinary application objects.

### Phase 3: Single Runtime Entry And Non-Detachable Multi-Graph Authority

This phase introduces one installed operating-world root from which every
ordinary operation-family facade and later capability proceeds. Construct,
boolean, transform, route, recovery, and future families are typed borrowed
views over this root; they cannot install their own runtime, retain an
independent graph world, or expose an alternate authority constructor.

A bound operation capability, with projection-specialized views where
applicable, retains the exact installed-domain and graph-participation
authorities, canonical operation definition, native value contract, runtime
generation, basis, policy, tenant, preview or branch scope, exact installed
conditional-node lowering and Signal graph binding where declared, and
required cross-domain and cross-graph capability set as private state. Only Query can
construct it by atomically binding every installed capability declared by the
operation. Cross-graph execution is admitted as atomic only when the adapters
share an admitted commit authority; otherwise the bound capability carries the
installed compensation and partial-failure contract before execution starts.

Atomic binding uses a `worth-proof::AssumptionBasis` covering the exact runtime,
installation, graph, policy, tenant, branch/preview, and conditional-lowering
generations plus concrete Query/adapter capability witnesses. Required multi-
domain and multi-graph sets use proven non-empty, canonical, unique, or fixed-
arity carriers only after Query validates those properties. Foundational
canonical comparison may reject portable mismatch early, but only Query's
checked admission can mint the bound operation capability.

**Relevant subsystems**
- installed domain capability and runtime-affine handle authority
- downstream basis and projection authority
- atomic multi-domain capability binding
- single operating-world root and operation-family facade borrowing
- installed graph-participation adapter binding
- public domain/read facade

**Relevant APIs**
- `WorthQueryInstalledDomainHandle<D>`
- installed-domain view declaration surfaces
- installed graph-participation adapters
- proposed installed operating-world root, borrowed operation-family facade,
  bound operation/projection capability, commit posture, and binding denial
- `worth_proof::AssumptionBasis`, concrete capability witnesses, checked
  outcomes, and proven collection carriers behind the sealed Query binding API

**Warnings**
- A convenience struct with public fields is still an ingredient list, not a
  capability.
- Phantom tags cannot substitute for retained runtime-affine authority.
- Do not make the capability serializable or reconstructible from canonical
  definition data; persistence later readmits portable meaning through the
  owning runtime.
- Do not erase legitimate view-family or domain-family distinctions behind one
  dynamically typed bag.
- One root does not mean one untyped `execute(any)` method. Typed operation
  families and their cost/failure contracts remain visible.
- An operation-family crate may contribute definitions and lowering but cannot
  construct, own, clone, or replace the operating-world root.
- A tuple, struct, or map of independently bound topology, spatial, relational,
  or other domain capabilities is not atomic operation authority.
- Query must reject mixed runtime, installation generation, basis, policy,
  tenant, preview, and branch scope before producing the bound capability.
- Conditional operation binding must also reject a missing, extra, foreign, or
  stale condition lowerer, Signal graph, trigger contract, or comparator
  provider before minting the bound capability.
- Separate graph adapters do not imply distributed atomicity. Query must prove
  shared commit authority or expose compensated, partially committed, and
  recovery-required outcomes explicitly.
- Matching portable commit names, semantic keys, digests, or provider labels
  never prove shared commit authority. Atomic multi-graph binding requires one
  runtime-minted commit-authority witness retained by every participating
  installed adapter; independently authored equal labels have reporting power
  only.
- A raw proof set, generic capability witness, Foundational equivalence result,
  or prevalidated tuple cannot bypass Query's atomic admission or recover a
  partially bound capability after denial.

**Test requirements**
- Adversarial convergence test: equivalent installed declarations in the same
  runtime produce equivalent bound capability meaning regardless of declaration
  order or facade authoring path.
- Adversarial construction test: external code cannot mint, field-assemble,
  deserialize, or clone-and-restamp a bound capability from an installed
  handle, view definition, runtime identity, or digest.
- Adversarial separation test: independently valid domain and view artifacts
  from different runtimes or installation generations cannot bind and perform
  exact zero planning, execution, and refinement work.
- Adversarial atomicity test: individually valid required domain capabilities
  with mismatched basis, policy, tenant, preview, branch, or installation
  generation cannot be bundled, reordered, partially bound, or substituted to
  mint operation authority.
- Adversarial single-entry test: public operation-family facades cannot create
  an operating world, accept raw runtime or graph handles, or execute without a
  borrow from the one installed root.
- Adversarial cross-graph test: adapters with separate commit authorities
  cannot obtain an atomic execution capability; they receive the declared
  compensation/partial-failure posture before lowering or graph mutation.
- Adversarial same-commit convergence test: a genuinely shared commit authority
  produces one atomic bound operation regardless of operation-family facade or
  adapter registration order.
- Adversarial commit-label collision test: two foreign or independently
  installed providers declaring the same commit label cannot mint atomic
  authority and deny before planning or provider contact with exact zero-work
  counters. The corresponding positive case must prove pointer- or witness-
  bound ownership of one runtime-minted commit authority, not string equality.
- Adversarial conditional-authority test: independently valid operation,
  condition-lowering, Signal graph, comparator, and basis capabilities cannot
  be tuple-assembled or substituted to mint one bound conditional operation;
  denial performs zero condition checks or Signal execution.
- Adversarial proof-boundary test: raw proof carriers, caller-selected witness
  markers, reordered or duplicate proven-collection inputs, and Foundational-
  equivalent but runtime-foreign participants cannot mint bound authority;
  each denial leaves no reusable partial binding.

**Engineering decisions**
- The bound capability owns the complete prerequisite set for downstream
  operation and projection work and exposes read-only semantic inspection.
- A conditional bound capability privately retains the exact Query declaration
  admission, bridge lowering, installed Signal node identity, graph binding,
  and comparator authority required to request evaluation later.
- Cross-domain binding is one Query decision and one proof artifact, not a
  consumer-authored aggregation of successful independent binds.
- Cross-graph binding is the same authority decision over explicit installed
  adapters; graph-to-graph direct calls and application bridges are forbidden.
- One root centralizes authority and lifecycle while typed borrowed facades
  preserve operation-family meaning, cost, failure, and discoverability.
- Construction is sealed behind the installed-domain declaration path.
- Canonical portable declarations remain non-authoritative inputs; the bound
  capability is runtime-affine operational authority.

**Open questions**
- Exact generic parameters and final public name remain open until compiler
  inference and facade transcripts prove the smallest honest surface.

### Phase 4: Query-Minted Consumer Support Contract

This phase makes the bound projection mint one typed consumer contract that
states the Query meaning and support required downstream. The contract carries
the admitted basis capability, live and continuation posture, result-state and
async posture, projection-consumption contract, recovery and inspection
capabilities, conditional evaluation posture and admitted condition family,
dependency-impact contract, sharing posture, invalidation contract, and
collection-delivery posture that apply to this exact bound
projection.

Downstream runtimes may add presentation, graph attachment, allocation, or host
policy, but they do not enumerate Query subsystems, derive support from local
hook kinds, or fold Query reporting digests into a local posture product.

**Relevant subsystems**
- Query support and admission matrices
- bound projection declaration and installed-domain capability
- live, async/result-state, recovery, inspection, and projection consumption
- conditional-node family, bridge-lowering, comparator, trigger, and temporal
  or on-demand support
- downstream consumer contract facade

**Relevant APIs**
- Query support snapshots and admitted capability artifacts
- proposed `WorthQueryConsumerProjectionContract` or equivalent
- bound projection support and requirement inspection
- Foundational profile, support-truth, boundary-evidence attachment, and
  materialization vocabulary for derived cross-crate support reporting

**Warnings**
- A struct containing one string digest per Query subsystem is still a
  consumer-authored Query mirror.
- The consumer contract must express admitted semantic requirements, not an
  inventory of whichever internal modules currently implement them.
- Consumer-local presentation posture may not alter Query support,
  compatibility, recovery, continuation, or result-state meaning.
- Support must be evaluated by Query; a downstream default that treats every
  registered hook as supported is structurally dishonest.
- Foundational support profiles and evidence describe admitted Query support at
  a boundary; they do not replace the stronger Query consumer contract or
  perform operational support admission.
- Comparing two copies of declared requirements is not support admission. Each
  required dimension must be evaluated against the installed runtime's
  Query-owned support truth, and every dimension-specific denial required below
  must be reachable by drifting that runtime truth without rewriting the
  operation definition.
- A runtime that supports ordinary Signal evaluation does not thereby support
  every Query-authored condition family, comparator, temporal wake, on-demand
  trigger, or result-state consequence. Each required dimension is admitted
  explicitly; absent dimensions are typed `NotRequired` only when the
  installed operation truly does not declare them.

**Test requirements**
- Adversarial convergence test: equivalent bound projections mint identical
  consumer contract meaning, required capabilities, support posture, and exact
  counters regardless of declaration order or facade path.
- Adversarial spoofing test: local hook enums, copied support reports, matching
  digest strings, and consumer-authored requirement lists cannot construct or
  satisfy the consumer contract.
- Adversarial support drift test: basis, live, async/result-state, recovery,
  inspection, projection-consumption, condition family, comparator, trigger,
  temporal/on-demand lowering, dependency-impact, sharing,
  invalidation, or collection-delivery drift produces a Query-owned typed
  compatibility denial before downstream planning rather than a consumer-side
  digest comparison.
- Adversarial derived-state test: support indexes and reporting summaries can be
  destroyed and rebuilt from the bound capability and consumer contract
  without changing admission, compatibility, denial, or lifecycle outcomes.
- Adversarial Foundational support-export test: Query support can materialize a
  deterministic Foundational profile/evidence attachment with exact narrowing,
  freshness, degraded, and availability posture, while copied or readmitted
  descriptive attachments cannot satisfy the Query operational contract.

**Engineering decisions**
- Query mints exactly one consumer contract per bound projection generation.
- The contract is proof-bearing operational input; reporting and human-readable
  support summaries are derived projections with no construction power.
- Downstream-specific requirements compose beside the Query contract through a
  named consumer boundary rather than being inserted into Query authority.
- Query retains its stronger support truth and uses Foundational only for shared
  profiles, evidence attachments, and support-report materialization; rich
  reports remain off the ordinary hot path.

**Open questions**
- Exact grouping of live, continuation, recovery, and result-state views may
  follow existing Query capability families; the one-contract authority is
  fixed.

### Phase 5: Installed Execution And Consumption Progression

This phase makes execution and projection consumption a proof-widening chain
owned by the bound capability. For operations with declared downstream output,
the ordinary progression is structurally equivalent to `bound -> executed ->
published -> consumed -> settled`; each transition consumes or immutably
borrows the exact prior proof and returns the only type accepted by the next
transition. Operations without publication cannot be treated as downstream
artifact producers merely because they completed. For a conditional operation,
`execute` first consumes the exact Signal evaluation decision admitted for this
bound attempt. Only a Signal outcome that authorizes and completes computation
may contribute computed output; deferral, suppression, ineligibility, or
reverted-clean posture follows its own typed terminal or no-change path.

**Relevant subsystems**
- ordinary read execution and installed-domain execution receipts
- runtime-bridge evaluation requests and Signal-owned decision/outcome evidence
- projection consumption declarations, contracts, facts, and receipts
- derived publication definitions, artifacts, and publication receipts
- downstream authority settlement and result-state handling

**Relevant APIs**
- `WorthQueryInstalledDomainExecutionReceipt`
- `WorthQueryConsumedProjectionAuthority`
- proposed bound execution, derived publication, consumed projection, and
  settlement capabilities
- proposed Query-admitted conditional execution handoff retaining the exact
  installed Signal node, condition, comparator, graph, snapshot, and attempt
- `worth_proof::Artifact` with Query-owned phase markers, proof sets, checked
  `TransitionOutcome`, and `AssumptionBasis` as the internal progression
  substrate behind opaque Query facade types

**Warnings**
- A tuple containing the same artifacts is not a proof-bearing phase type.
- Do not implement a second Query-local proof carrier, freshness enum, or
  success/denial/deferred/stale/rebind/failure lattice. Reuse `worth-proof`
  internally, while preserving the stronger Query-specific public wrappers.
- A raw `worth_proof::Artifact`, `Proof`, `AuthorityWitness`, or
  `CapabilityWitness` is not ordinary Query authority. Only the owning Query
  transition may consume the concrete sealed witness and mint the next wrapper.
- A transition helper that destructures its predecessor and omits retained
  domain, graph, commit, basis, installation, result-state, or warning authority
  has broken the chain even if Rust still enforces move-only phase order. Every
  phase must retain the complete predecessor proof, not a hand-selected subset.
- The executor must not accept a detached definition or independently supplied
  basis to override the capability's authority.
- Advisory, partial, pending, and violation posture must remain attached to the
  same chain instead of being flattened into booleans or side-channel errors.
- Do not make consumers replay Query's internal phase graph manually under new
  method names.
- Ordinary downstream code cannot inspect a completed execution trace to
  reconstruct output. It consumes the declared publication through projection
  consumption.
- An executor-supplied packet containing a read result, result shape, and
  authorized projection is execution material, not a Query publication. Query
  must validate that material against the bound definition and basis and mint a
  distinct derived-publication receipt; merely rewrapping the packet in a
  `Published` type is forbidden.
- Query and the domain executor may not reinterpret trigger inputs or promote a
  Signal deferral, suppression, ineligibility, or reverted-clean outcome into
  successful computation. A reporting digest that says `computed` has no
  authority.

**Test requirements**
- Adversarial continuity test: installation authority, declaration identity,
  basis, execution receipt, consumption contract, materialized facts, warnings,
  and settlement remain traceable through one chain with no restamping.
- Adversarial mix-and-match test: a completion, fact receipt, or consumption
  authority from another bound capability cannot advance the chain even when
  its query text, view shape, basis label, and terminal digest match.
- Adversarial phase-order test: consumption before execution, settlement before
  consumption, and reuse of a moved single-use transition are uncallable or
  deny at the earliest trust boundary with zero later-phase work.
- Adversarial substrate-parity test: the pleasant Query facade and its internal
  `worth-proof` transition path produce identical phase, basis, proof-set,
  checked-outcome, and move-only behavior; raw proof carriers or caller-defined
  authority markers cannot enter or advance the Query chain.
- Adversarial publication test: a completion without a declared and minted
  publication receipt cannot enter projection consumption; a publication from
  another operation, run, basis, or graph adapter cannot be consumed.
- Adversarial predecessor-retention test: sabotage every authority retained by
  the bound capability independently and prove that none disappears during
  `bound -> executed -> published -> consumed -> settled`. In particular,
  executor-authored foreign read results, result shapes, authorized
  projections, or matching publication digests deny before publication with
  zero consumption work.
- Adversarial evaluation-outcome test: cross-node, cross-graph, cross-runtime,
  cross-basis, cross-snapshot, cross-attempt, or condition/comparator-drifted
  Signal evidence cannot execute or publish. Deferred, suppressed, ineligible,
  and reverted-clean outcomes mint no computed publication and report exact
  zero forbidden later-phase work.
- Adversarial ordinary-path test: a downstream operation obtains every required
  upstream fact from the typed publication and cannot reopen trace, replay, or
  provider internals.

**Engineering decisions**
- Phase types encode what has been proven, not merely which fields are present.
- Query phase wrappers use the shared `worth-proof` artifact/proof-set/basis
  machinery rather than restating progression law, but their fields and
  constructors remain private so the generic substrate does not widen Query's
  authority surface.
- Execution and consumption transitions preserve the installed authority
  privately and expose semantic outcomes through contracted views.
- Query mints its execution receipt from retained bound authority plus the
  exact Signal-owned decision and outcome; it does not mint a second local
  evaluation decision.
- Execution trace is certification evidence; derived publication is the
  ordinary inter-operation handoff. Neither can substitute for the other.
- Multi-consumer observation, when legitimate, uses an explicit shared
  authority lifecycle rather than arbitrary packet cloning.

**Open questions**
- None. Snapshot settlement remains move-only unless a later explicit
  observation lease is admitted through Phase 19.

### Phase 6: Installed Workflow Graph And Query-Minted Run Trace

This phase turns an operation's optional workflow posture into an installed,
validated graph and a proof-bearing execution trace. An installed operation may
declare typed stage identities, stage dependencies and legal successors,
required installed capabilities across domains, typed input and output
contracts, optional portable conditional-node contracts, graph-read roles,
touch and effect roles, invariant checkpoints, cost roles, and terminal result
and failure classes. Package installation
proves that the stage graph is acyclic, complete, uniquely identified, and
dependency-closed before any runtime work can begin.

Starting a run from the atomically bound operation capability mints a Query-
owned workflow-run capability. Each admitted stage consumes the exact required
predecessor receipt set and produces a Query-minted stage receipt carrying the
workflow run, installed operation, bound authority, predecessor continuity,
typed native outputs, graph reads, touch and effect evidence, invariant
outcomes, exact Signal evaluation decision/outcome evidence where declared,
and exact counters. A consumer cannot skip a stage, splice runs, or
maintain a parallel evidence ledger.

**Relevant subsystems**
- installed operation workflow definitions and package validation
- atomic multi-domain operation binding
- graph-read planning, graph-touch obligations, and authority-scoped effects
- deterministic parallel admission and exact workflow accounting
- conditional stage admission, Signal evaluation, and semantic-cleanliness
  outcomes

**Relevant APIs**
- installed domain operation capability and bound operation capability
- existing graph-read access plans, graph-touch obligations, and lowered
  authority-scoped effect plans
- existing deterministic parallel admission proofs
- proposed installed workflow graph, workflow-run capability, stage admission,
  stage receipt, and completed execution trace
- `worth_proof::NonEmpty`, `CanonicalVec`, `UniqueVec`, fixed-arity join
  carriers, structural proofs, and checked transition outcomes where the
  declared workflow shape honestly establishes those facts

**Warnings**
- A global product-specific stage enum or hard-coded stage slot map is not a
  generic workflow graph.
- Independently valid stage receipts are not composable unless Query proves
  they belong to the same run, operation, authority, and predecessor frontier.
- Portable workflow definitions declare semantic roles and dependencies; they
  never carry executable callbacks or backend objects.
- Do not rebuild graph-read, touch, effect, or parallel-admission semantics in
  this phase. The workflow trace consumes the existing proof-bearing contracts
  established by Milestones 9.10, 9.9, 9.3.3, and 5.3.
- Diagnostic stage timelines are derived views of the retained trace and have
  no authority to advance a run.
- `worth-proof` may carry stage/predecessor facts and fixed-arity joins, but it
  is not the dynamic workflow graph or scheduler. Query retains DAG validation,
  frontier selection, runtime scheduling, and run lifecycle authority.
- Do not mint canonical-order, uniqueness, disjointness, or non-empty proof
  forms merely because the desired property was declared. Package admission
  mints them only after validating the actual installed workflow graph.
- A conditional stage that does not evaluate is not a completed stage. Its
  installed workflow contract must declare whether the outcome defers the
  frontier, legally suppresses the stage, preserves a prior clean result, or
  terminates the run; host code may not guess a successor.

**Test requirements**
- Adversarial graph-convergence test: equivalent workflow definitions install
  to the same validated DAG and stage semantics regardless of declaration
  order, while cycles, missing predecessors, duplicate stage identities,
  unreachable stages, and incomplete terminal paths fail installation
  atomically.
- Adversarial progression test: stage skipping, duplicate advancement,
  predecessor omission, cross-run receipt mixing, foreign-runtime receipts,
  and wrong-basis or stale-generation receipts cannot advance a run.
- Adversarial multi-domain binding test: every required domain capability must
  share runtime, installation generation, basis, policy, tenant, and preview or
  branch scope before the first stage; partial success mints no run capability.
- Adversarial trace-convergence test: valid serial and deterministically
  parallel schedules produce the same semantic trace, outputs, touch/effect
  closure, invariant outcomes, and terminal result for the declared workflow.
- Adversarial cost test: stage admission inspects only the declared predecessor
  frontier and required capabilities, and exact counters expose graph, read,
  touch, effect, and parallel-admission work without scanning unrelated runs.
- Adversarial conditional-frontier test: each declared condition outcome
  advances only its installed legal successor posture. A detached decision,
  copied outcome label, changed trigger/comparator, or decision from another
  stage/run cannot advance the workflow, and skipped computation is not
  charged or traced as executed.
- Adversarial proof-substrate test: raw vectors, tuples, symbolic family
  references, caller-minted markers, and unvalidated predecessor sets cannot
  satisfy admitted stage progression; proven fixed-shape/static joins retain
  parity with the Query DAG result without becoming a second scheduler.

**Engineering decisions**
- The installed workflow graph is part of the operation semantic closure; the
  run trace is the single Query-minted canonical execution artifact.
- Query reuses `worth-proof` collections and structural facts only after it has
  proved their exact properties; dynamic DAG topology remains Query-owned.
- Domain implementations own stage algorithms and lower through admitted
  Query and lower-runtime capability seams. Query owns legal progression,
  predecessor continuity, trace assembly, and stage authority.
- The trace derives from existing graph-read, touch, effect, and parallel
  proofs rather than storing consumer-authored summaries of them.
- Stage receipts are proof-bearing inputs to later stages and to replay,
  reversal, lineage, dependency impact, inspection, and certification.
- Signal decision evidence remains Signal-owned inside the Query-minted stage
  receipt. The receipt proves Query admitted and consumed it for one exact
  stage attempt; it does not restamp the decision as Query-owned truth.
- A workflow terminal or declared publishable stage mints its publication from
  the trace once. Work outside that installed workflow consumes the publication
  rather than treating a stage receipt as a public handoff.

**Open questions**
- The concrete representation of joins and parallel frontiers remains an
  implementation choice, but it must preserve exact predecessor authority and
  deterministic semantic trace ordering.

### Phase 7: Query-Authored Conditional Node Contract

This phase makes conditional computed and workflow nodes first-class Query
authoring meaning. Query owns a portable, callback-free declaration that names
the typed node role, declared semantic truth dependencies and produced derived
roles or partitions, required context, trigger inputs, evaluation condition, comparator and
equivalence posture, temporal wake or on-demand trigger posture where needed,
maintenance and artifact posture, and relationship to operation outputs. The
contract is part of canonical operation or workflow-stage meaning and is not
executable until admitted by an installed runtime.

Each semantic truth dependency retains the exact Foundational aspect key,
identity, contract revision, projection/change mask and field paths, the
Relational entity-field, relation-field, relation-endpoint, structural, or
lifecycle binding, locality/partition posture, and the change meanings that may
trigger evaluation. Query declares portable semantic consumption; it does not
declare a numeric Signal aspect, assume one truth aspect equals one Signal slot,
or make runtime slot allocation part of canonical operation identity.

Each dependency also names the exact operation graph-read role from which that
truth may be consumed, including an explicit primary-logical-graph role when no
separate graph participates. Installation proves that its read scope is a
subset of that role's declared graph-read contract. Each declared output
relationship similarly identifies the operation output, touch, or effect role
it may contribute to, and installation proves that the possible consequence is
already contained by the operation's touch/effect closure. These bindings are
portable semantic requirements, not graph capability: only later binding to the
installed operating world can mint the exact runtime-affine graph authority.

Those semantic dimensions are the existing Foundational-native types and
admitted mask/locator forms, not Query-owned lookalikes. Portable dependency and
conditional declarations prepare a Foundational canonical basis for comparison
and export parity; Query installation consumes the full canonical basis and
structured mismatch result. A digest may index or report that basis but cannot
admit installation or correspondence.

The ordinary facade must express at least always-eligible, aspect-filtered,
delta-threshold, on-demand, temporal, and registered domain-specific condition
families without exposing Signal runtime objects. A domain-specific family uses
a typed Query declaration marker and a separately registered lowering; a raw
custom string, closure, `Any` payload, or consumer callback is not portable
conditional meaning. Inherited comparator or tier policy, when supported, is
an explicit canonical posture rather than an omitted field interpreted later.

Illustrative DX, with exact names left to implementation:

```rust
let operation = package
    .operation(RebuildGeometry)
    .computed_node(RebuildFaces)
    .reads_truth(
        BodyTopology::aspect()
            .fields(TopologyFields::CONNECTIVITY)
            .on(ChangeMeaning::ValueOrMembership),
    )
    .produces::<FaceGeometry>()
    .when(ChangedTruth::<BodyTopology>::new())
    .compare_with(GeometryTolerance)
    .install_into(&mut runtime)?;
```

This is node evaluation authoring. Existing Query effect conditions govern
whether an admitted downstream effect executes and remain a separate contract;
neither surface accepts or silently defaults the other.

**Relevant subsystems**
- ordinary Query operation, computed, workflow-stage, temporal, and on-demand
  authoring
- installed domain operation and workflow canonical identity
- Foundational aspect, partition, context, and native value contracts
- Relational aspect bindings, authoritative change kinds, and source surfaces
- Signal node semantic, projection, execution, authority, reuse, condition,
  comparator, maintenance, and artifact contracts

**Relevant APIs**
- ordinary Query declaration builders and domain package definitions
- Signal `NodeContract`, `NodeEvaluationConfig`, `EvaluationCondition`,
  comparator/equivalence, temporal, and reuse semantics as lowering targets,
  not public Query authoring inputs
- proposed portable conditional-node declaration, typed condition family,
  semantic truth dependency, trigger contract, comparator requirement, and
  output relationship
- Foundational `AspectContract`, `AspectKey`, `AspectIdentity`,
  `AspectContractRevision`, field path/key, projection and mutation masks,
  aspect/field locators, native value carriers, canonical basis, comparison,
  and mismatch classification

**Warnings**
- Do not copy Signal's public enums into a Query mirror or expose raw Signal
  configuration through the Query facade. Query declares stable authoring
  intent; Signal retains runtime evaluation semantics.
- `Custom(String)` is not an acceptable ordinary Query escape hatch. A custom
  domain condition requires a typed marker, canonical portable parameters,
  explicit support posture, and exact registered lowering.
- A bare `f64` delta threshold is not sufficient portable domain meaning for a
  geometry, physics, or other unitful value. Query authoring retains the
  canonical native quantity, unit/tolerance contract, comparison domain, and
  validation posture; NaN, infinity, negative values where illegal, unit drift,
  and incompatible value families deny before installation or lowering.
- Declared condition trigger inputs must be a dependency-closed subset of the
  node's admitted reads, context, temporal source, or on-demand owner. A
  produced aspect cannot become an implicit feedback trigger.
- A matching `AspectKey` is insufficient when identity, contract revision,
  field mask, Relational binding, locality, or relevant change meaning differs.
  Those dimensions are canonical semantic input, not reporting metadata.
- Do not introduce Query mirrors of Foundational contracts, masks, locators,
  scalar/struct carriers, unitful quantities, canonical basis, or mismatch
  categories. Query owns how those exact meanings participate in an installed
  operation, not a second encoding of the meanings themselves.
- Canonical basis is authoritative comparison input; canonical digest is derived
  compression. Equal digest bytes cannot satisfy dependency equivalence or
  installation admission.
- Query declarations never expose `worth_signal::Aspect`, raw numeric slots, or
  an `AspectMask` whose meaning depends on an installed Signal graph. Runtime-
  local allocation belongs to the installed correspondence phase.
- Do not require one-to-one aspect correspondence. A truth dependency may fan
  out to several Signal nodes; several truth fields may intentionally share a
  local invalidation slot; and a derived Signal output may have no authoritative
  Relational source.
- `None means inherit`, an empty aspect mask, and a default `Always` may be
  legitimate semantics only when the Query contract names them explicitly.
  Absence may not acquire meaning from bridge or host defaults.
- A condition does not authorize reads, temporal wakes, on-demand triggers, or
  outputs beyond the declaration's existing contracts.
- A dependency's graph-read role and an output's touch/effect role are
  load-bearing semantic bindings. Locality, matching aspect meaning, a Signal
  target, or a condition that evaluates true cannot infer, replace, or widen
  them. Read authority and mutation authority remain distinct typed lanes
  joined only by the same installed operation closure.
- Query effect conditions, live relevance predicates, workflow successor
  guards, and Signal node evaluation conditions must remain separately named
  even when a product feature uses several together.

**Test requirements**
- Adversarial facade transcript: external code declares, installs, binds, and
  executes representative aspect-filtered, threshold, on-demand, temporal, and
  typed domain-specific nodes without deep Query imports or Signal internals.
- Adversarial canonicality test: equivalent declarations converge across
  builder order and package order; one-field drift in reads, outputs, condition,
  trigger, comparator, equivalence, temporal, maintenance, or artifact posture
  changes canonical meaning and installation conflict results.
- Adversarial truth-dependency test: drift aspect identity, contract revision,
  field mask, entity/relation binding, endpoint role, lifecycle meaning,
  locality, or relevant change kind independently and prove exact canonical
  conflict or typed denial rather than an aspect-name mismatch.
- Adversarial graph-closure test: bind a dependency to an undeclared or foreign
  graph-read role and bind each possible output to an undeclared touch/effect
  role; installation denies before correspondence or lower-runtime work. A
  condition, locality, aspect key, or reporting digest that otherwise matches
  cannot repair the missing operation scope.
- Adversarial Foundational parity test: declarations authored through every
  legal Query facade path retain byte-for-byte/canonical-entry-equivalent
  Foundational contracts, masks, locators, and native values; no Query-local
  conversion changes field, absence, opaque, unit, or revision meaning.
- Adversarial canonical-basis test: equivalent declarations converge through
  Foundational canonical comparison while digest collision/lookalike, producer-
  shape drift, unsupported comparison, and bridged-but-not-readmitted basis fail
  before Query installation.
- Adversarial comparator-separation test: drift dependency-version comparator,
  eligibility condition, post-compute output equivalence, and artifact-reuse
  equivalence independently and prove that each changes only its declared
  decision and canonical dimension.
- Adversarial threshold test: canonical unitful thresholds converge, while NaN,
  infinity, illegal negative values, unit drift, value-family drift, and raw
  `f64` lookalikes deny before runtime lowering.
- Adversarial construction test: raw Signal configuration, custom strings,
  raw Signal aspects/masks, callbacks, `Any` payloads, copied canonical fields,
  and Query effect conditions cannot construct or satisfy a conditional-node
  declaration.
- Adversarial default-honesty test: no omitted field can silently become
  `Always`, inherit a comparator, broaden an aspect mask, or acquire a temporal
  clock or on-demand trigger from downstream defaults.
- Compile-fail evidence is selective: retain probes only for authority-bearing
  private construction and cross-condition/domain substitution that runtime
  tests cannot prove more cheaply.

**Engineering decisions**
- Query owns the portable authoring grammar and canonical operation inclusion;
  Signal owns installed evaluation behavior. The bridge is the only conversion
  boundary between them.
- Query canonical identity includes semantic truth dependency meaning but
  includes its graph-read and possible-consequence role bindings, while
  excluding runtime-local graph capability identities, Signal slot numbers, and
  allocation order.
- Foundational supplies the portable semantic/canonical payload; Query's
  installed declaration remains the stronger owner-specific artifact and does
  not lower back into a generic Foundational state map.
- Condition parameters contain portable semantic data only. Volatile resolvers,
  clocks, trigger owners, comparators, and callbacks are registered separately.
- Existing Signal contracts are reused as the runtime target; Query does not
  create a second condition evaluator or comparator algebra.
- One responsibility-named module owns each of portable condition declaration,
  comparator requirement, trigger posture, and operation/stage attachment. No
  `condition_helpers`, generic support bag, or cross-layer configuration file
  may absorb them.

**Open questions**
- Exact facade names remain open until compile-pass transcripts prove inference
  and discoverability. Owner boundaries and supported semantic families do not.

### Phase 8: Aspect-Precise Relational Change Publication

This phase repairs the authoritative Relational-to-bridge publication substrate
before Query installs any correspondence or conditional node. The one canonical
Relational authoritative patch remains the source of truth. Its bridge-facing
derivation preserves the exact aspect key and contract identity available at
the publication boundary, field sets and clears, whole-aspect operations,
relation source/target endpoint changes, structural surface, lifecycle meaning,
locality, and whether an aspect payload is opaque.

The adapter emits the narrowest self-describing truth-delta target justified by
the authoritative patch. A field-level patch remains field-level. A whole-
aspect mutation remains whole-aspect. Endpoint, structural, and lifecycle
changes remain distinct. When source evidence cannot support exact field or
semantic precision, the adapter returns a typed unsupported or declared-
widening posture; it does not silently emit a whole-aspect target, substitute a
synthetic `lifecycle` aspect, or label widened publication as exact.

The published truth delta retains the exact Relational graph authority,
transaction/commit authority, logical graph role, and source basis that produced
the patch. It is descriptive input to later correspondence and invalidation; it
does not mint Query graph-read authority, Query graph-touch authority, or a new
bridge graph identity. A semantically identical patch from another graph,
adapter, runtime, tenant, branch, or generation is foreign.

The semantic payload uses Foundational authoritative aspect state, patch,
contract, mask, field-path, and locator vocabulary. Relational retains the
stronger canonical transaction and authoritative patch. Checked publication
uses `worth_proof::TransitionOutcome`: exact and explicitly admitted widening
are successful owner-specific publication artifacts; unsupported, stale,
graph-role rebind-required, deferred, and denied outcomes remain distinct and
mint no successful envelope. The current in-memory lowering has no
infrastructure failure seam, so its failed payload is `Infallible` rather than
a ceremonial failure variant; a later fallible provider may introduce a
concrete failure only with a real producer.

**Relevant subsystems**
- Relational authoritative patch publication and aspect evaluation/emission
  evidence
- Relational presentation adapter for runtime-bridge committed patch envelopes
- Foundational aspect identity, contract revision, field path, and mask types
- runtime-bridge truth-delta locators, target surfaces, and snapshot slices

**Relevant APIs**
- Relational `PublishedAuthoritativeRecordPatch` and
  `PublishedAuthoritativePatchOperation`
- Relational declared aspect contracts and entity/relation/lifecycle bindings
- bridge `AspectLocator`, field locator, projection mask,
  `TruthDeltaSurfaceKind`, and committed patch target/envelope
- Foundational authoritative aspect state, admitted mutation/projection masks,
  field paths/locators, and patch set/clear vocabulary
- `worth_proof::TransitionOutcome` with Relational-specific success, denial,
  deferred, stale, rebind-required, and failure payloads
- proposed typed successful publication precision and typed unsupported denial

**Warnings**
- Do not derive bridge changes from `changed_aspects()` alone when the canonical
  patch retains field-level operations. That projection is too weak to justify
  a field-precise or whole-aspect claim.
- `EntityField` is a supported authoritative surface, not an unreachable bridge
  case. Production code may not panic because the adapter discarded the field
  locator earlier.
- Opaque aspect change is not lifecycle change. Opaque posture must survive as
  an explicit typed limitation or denial until a policy with authority admits
  widening.
- Structural created/updated/deleted/retained posture does not replace aspect
  binding or field-change meaning; both dimensions remain available where the
  source patch proves them.
- This phase derives one bridge envelope from the canonical patch. It must not
  introduce a second Relational change ledger or consumer-specific patch path.
- Publication carries the source graph authority forward but grants no new
  graph capability. Later phases must join it to the exact bound operation
  graph participation rather than treating aspect equality as graph admission.
- Foundational patch and locator types are boundary vocabulary, not permission
  to replace the stronger Relational authoritative patch or reopen mutation
  admission inside the bridge.
- A descriptive Foundational boundary artifact/report cannot substitute for the
  successful Relational publication artifact or advance routing.

**Test requirements**
- Adversarial operation-parity test: whole-aspect set/clear and field set/clear
  operations produce the exact corresponding bridge targets and masks from the
  real transaction, commit, publication, and presentation path.
- Adversarial binding matrix: entity field, relation field, relation source and
  target endpoint, structural region/facet, and lifecycle changes remain
  distinguishable through the bridge envelope; substituting one for another
  fails at publication admission.
- Adversarial precision test: a one-field mutation never becomes an exact
  whole-aspect change, while an explicitly admitted widening records its cause,
  source precision, target precision, and exact widening counters.
- Adversarial opaque-change test: opaque authoritative change cannot become a
  lifecycle target, empty change, exact whole-aspect target, or default `ALL`
  mask; it yields the declared typed posture before routing or Signal work.
- Adversarial convergence test: equivalent authoritative patches produced
  through different legal declaration and transaction orderings emit identical
  canonical truth-delta envelopes and exact counters.
- Adversarial source-authority test: equal aspect contracts, field targets,
  change meaning, and canonical bytes from a foreign graph, adapter, runtime,
  branch, tenant, or generation cannot satisfy the bound operation's graph role
  and deny before correspondence lookup or Signal work.
- Adversarial derived-state rebuild test: destroy all Relational presentation
  and bridge routing indexes, rebuild them from authoritative schema and patch
  truth, and prove identical targets, precision postures, denials, and counters.
- Adversarial outcome-topology test: unsupported, denied, deferred, stale, and
  graph-role rebind-required publication retain distinct `worth-proof` outcome
  categories and mint no successful bridge envelope, while exact and admitted-
  widening success remain distinguishable inside the success payload. Prove the
  failed payload is `Infallible` until a real fallible lowering provider exists.

**Engineering decisions**
- Relational owns interpretation of its canonical authoritative patch; the
  bridge consumes a self-describing truth delta and does not reopen Relational
  storage or infer change meaning from names.
- Relational reuses Foundational semantic payloads and `worth-proof` checked
  progression internally; its successful publication artifact remains the
  stronger owner-specific crossing type.
- Ordinary lowering cost is `O(o + f)` for relevant authoritative patch
  operations `o` and changed field entries `f`, independent of unrelated
  schema aspects, subscribers, Signal nodes, and diagnostic projections.
- Exact counters distinguish authoritative operations inspected, field targets
  emitted, whole-aspect targets emitted, unsupported opaque changes, and each
  admitted widening class.
- Responsibility-named modules separate authoritative patch interpretation,
  field-target lowering, structural/lifecycle lowering, and precision denial.

**Open questions**
- Exact precision-posture type names remain open, but silent widening and opaque
  fallback are prohibited.

### Phase 9: Installed Semantic-Aspect Correspondence And Signal Slot Allocation

This phase installs the exact correspondence between portable Query truth
dependencies, authoritative Relational change meaning, runtime-bridge routing,
and the local aspect layout of an installed Signal graph. The bridge mints an
opaque pair-bound correspondence witness from admitted Query operation/stage
meaning and declared graph role, the exact installed graph-participation
authority, installed Relational schema, bridge mapping, and Signal graph state.
No caller can construct the witness from component identities. This installed
correspondence is not yet Query operation authority: Phase 10 must join it to
the exact bound operation graph-read and graph-touch capability before it can
authorize conditional lowering or evaluation.

Correspondence admission is a checked `worth-proof` progression behind opaque
Bridge/Query types: portable dependency -> resolved Foundational/Relational
contracts -> lowered correspondence candidate -> admitted precision/capacity ->
execution-ready installed graph target. It carries a composite
`AssumptionBasis` covering Query installation/basis plus Bridge and Signal
runtime/graph generations. Only concrete sealed Bridge authority and installed
Signal-lowering capability witnesses may advance it.

The witness retains the exact Query dependency, Foundational aspect key,
identity and contract revision, Relational binding, field/projection mask,
change meaning, truth-delta surface and locality, declared operation graph role,
installed graph-participation authority identity and graph adapter, bridge
mapping registration, Signal runtime/graph/node
identity, node-local `Aspect` slot and mask,
partition, installation generation, basis, and an `Exact` or
`DeclaredWidening` success posture. Unsupported precision, ambiguity, capacity
exhaustion, stale basis, rebind requirement, denial, and failure are checked
`TransitionOutcome` branches that mint no installed witness. Volatile Signal
graph and slot mechanics register separately from portable semantic
declarations.

Correspondence is explicitly not one-to-one. One truth dependency may fan out
to several installed Signal nodes or slots. Several truth dependencies may
share a local Signal slot only when the installed widening contract makes the
broader invalidation honest. A Signal slot may represent derived-only state
with no Relational source. Raw slot numbers are meaningful only inside their
exact node, graph, runtime generation, and installed witness; slot `0` elsewhere
is unrelated.

Admitted one-to-many targets use proof-bearing non-empty, canonical, and unique
collections where those properties have actually been established. Many-to-one
allocation retains the canonical unique source set and explicit widening
evidence. Declaration-order sorting or deduplication is not itself proof; the
Bridge admission authority mints canonical-order, uniqueness, non-empty, and
disjointness facts only after checking the installed inputs.

The production bridge delivery path consumes the witness to mint actual Signal
invalidation seeds/masks for the installed graph. Bridge-owned stable names and
reporting scopes remain rebuildable diagnostics or lookup inputs and cannot
substitute for the witness or actual Signal target. Ambiguous overlapping
mappings, missing source precision, unsupported fan-out, and slot exhaustion
deny during installation before graph mutation or operation binding.

**Relevant subsystems**
- Query installed semantic truth dependencies and operation/stage identity
- Relational schema aspect contracts and Phase 8 truth-delta envelopes
- runtime-bridge mapping registration, overlap/widening admission, routing, and
  production Signal delivery adapter
- Signal graph/node installation, node semantic/projection contracts, aspect
  slots, masks, versions, partitions, and profile capacity

**Relevant APIs**
- proposed installed semantic-aspect correspondence and pair-bound witness
- `worth_proof::Recipe`/`Artifact`, `AssumptionBasis`, freshness/readmission,
  concrete authority/capability witnesses, `TransitionOutcome`, `NonEmpty`,
  `CanonicalVec`, `UniqueVec`, and applicable structural proofs behind
  owner-specific correspondence types
- Foundational canonical basis/comparison and boundary identity vocabulary for
  portable correspondence meaning and derived reporting, never installed-slot
  authority
- bridge `TruthPatchScope`, mapping registration, subscription slice,
  invalidation target/artifact, and Signal evaluation request
- Signal `NodeContract`, `NodeSemanticContract`, `NodeProjectionContract`,
  `Aspect`, `AspectMask`, aspect versions, and profile `MAX_ASPECTS`
- typed mapping ambiguity, unsupported precision, widening, and slot-capacity
  denials

**Warnings**
- A bridge `SignalInvalidationScope` stable name is not an installed Signal
  aspect. It may index or explain a mapping but cannot authorize invalidation.
- An unsupported, ambiguous, capacity-exhausted, stale, or rebind-required
  candidate cannot be retained inside an installed witness with a posture flag.
  It is a non-success `worth-proof` outcome and leaves no installed graph state.
- Raw `worth-proof` recipes, artifacts, proofs, bases, or witnesses cannot be
  supplied through the public Bridge or Query facade. Owner-specific wrappers
  seal concrete marker types and minting transitions.
- Never place numeric Signal slots in portable Query identity, persisted
  operation definitions, or Relational contracts. Allocation is runtime-local
  and may change after a legitimate reinstall without changing semantic intent.
- Many-to-one allocation trades precision for capacity. It is legal only as
  declared widening with observable fan-in, affected dependencies, and cost;
  equal slot numbers do not prove semantic equivalence.
- Slot exhaustion is a typed installation failure or explicit replan request.
  It must not silently collapse unrelated meanings into `ALL`, slot zero, or a
  catch-all aspect.
- Mapping overlap must be resolved before execution. Runtime first-match order,
  registration order, or specificity ties are not semantic policy.
- Query owns portable dependency intent, Relational owns truth meaning, the
  bridge owns installed correspondence, and Signal owns slot/version mechanics.
  No layer may recreate another layer's authority in a mirror registry.
- One Bridge runtime's installed correspondence registry targets exactly one
  executable Signal graph. Mixed-graph registrations fail runtime construction;
  callers bind that graph once through an opaque Bridge graph binding and may
  not reassemble loose `(bridge, graph)` pairs at each transition.
- One executable Signal graph admits exactly one opaque Bridge correspondence-
  allocation owner. Clones of that Bridge runtime retain the same owner, while
  an independently built runtime is rejected at graph binding before it can
  allocate a colliding slot or bypass declared many-to-one widening.
- Query's installed-correspondence facade exposes Query-owned inspection and
  delivery operations only. It never returns the underlying Bridge witness,
  Signal node/aspect targets, numeric slots, or a raw Signal graph handle.
- A correspondence witness authorizes only the mapped Signal target under the
  installed graph participation it retains. It is not a Query read/touch
  capability, cannot widen the operation touch descriptor, and cannot be spent
  for Query conditional execution until Phase 10 joins the exact bound
  operation capability.

**Test requirements**
- Adversarial end-to-end correspondence test: a real Relational field mutation
  flows through the authoritative patch and Phase 8 envelope, resolves one
  installed witness, and increments only the expected real Signal node/aspect
  versions through the production delivery adapter.
- Adversarial semantic-drift matrix: vary aspect key, identity, contract
  revision, field mask, Relational binding, endpoint role, lifecycle meaning,
  surface, locality, change meaning, declared graph role, installed graph-
  participation authority, graph adapter, graph,
  node, slot, partition, generation, or basis independently and assert the exact
  earliest denial with zero Signal mutation or later condition work.
- Adversarial unbound-authority test: a valid installed correspondence may exist
  before operation binding, but an installed operation plus matching dependency,
  Relational publication, Bridge mapping, Signal graph, and reporting digest
  cannot mint an execution-ready lowering or evaluate without the exact bound
  operation graph capability; a foreign equal-semantics capability also denies
  before conditional Signal mutation or evaluation.
- Adversarial slot-scope test: equal numeric slots across different nodes,
  graphs, runtimes, generations, or partitions are unrelated and cannot satisfy
  correspondence even when names and reporting digests match.
- Adversarial graph-cohesion test: one Bridge runtime rejects correspondence
  registrations spanning two Signal graph instances at construction, and its
  opaque graph binding rejects a cloned or foreign graph before admission,
  allocation, publication loading, or Signal mutation. The same graph rejects
  a second independently built Bridge runtime before either allocation registry
  can claim overlapping node/aspect ownership.
- Adversarial cardinality test: exact one-to-one, one-to-many, admitted many-to-
  one widening, and derived-only Signal aspects retain their distinct meaning;
  forbidden reverse promotion from derived-only state to truth authority fails.
- Adversarial capacity test: fill every supported Signal aspect slot, then prove
  that one additional independent dependency receives a typed capacity denial
  or explicit replan without overwriting, catch-all collapse, or graph residue.
- Adversarial non-success construction test: unsupported precision, ambiguity,
  capacity exhaustion, stale basis, rebind requirement, denial, and failure each
  retain their exact `worth-proof` outcome category and cannot expose,
  serialize, or later recover an installed correspondence witness.
- Adversarial ambiguity test: overlapping exact/wildcard mappings, equal-
  specificity candidates, and registration-order changes either converge on one
  canonical admitted result or fail atomically before graph mutation.
- Adversarial widening test: field-to-whole, entity, aspect, and surface
  widening each preserve exact cause and counters; no widened correspondence is
  observationally or operationally reported as exact.
- Adversarial rebuild test: destroy mapping, lookup, slot-allocation, and
  reporting indexes and rebuild them from authoritative installed inputs;
  admission, denial, actual Signal targets, outcomes, identities, and exact
  counters remain identical even if non-authoritative allocation iteration
  order changes.
- Adversarial proven-collection test: empty fan-out, duplicate targets,
  noncanonical order, overlapping targets falsely claiming disjointness, raw
  vectors, and raw tuples cannot satisfy correspondence admission; equivalent
  admitted inputs produce identical non-empty/canonical/unique proof-bearing
  collections independent of declaration order.
- Compile-fail evidence is selective: prove only that external callers cannot
  construct a correspondence witness, inject a raw Signal aspect, or substitute
  operation/family/domain marker types; runtime matrices own combinatorial drift.

**Engineering decisions**
- Portable semantic identity is stable across equivalent installations; the
  actual Signal slot layout is runtime-affine. The witness binds them without
  pretending the mechanical layout is portable meaning.
- The Bridge owns correspondence admission while `worth-proof` owns generic
  progression law. The Bridge does not implement private recipe, proof-set,
  freshness, or outcome machinery in parallel.
- Foundational canonical basis and boundary identity describe portable or
  reportable correspondence meaning; only the stronger installed Bridge witness
  can authorize a Signal target.
- The successful installed correspondence is a narrowing projection of the
  installed graph-participation authority, not of a not-yet-bound operation.
  Bridge admission preserves that predecessor proof; Phase 10 later joins the
  bound Query graph capability. Neither step duplicates Query's Graph Touch
  Obligation Authority or promotes a Signal invalidation target into graph
  power.
- Ordinary resolution is bounded by the emitted truth-delta targets and matched
  installed correspondence fan-out. It does not scan unrelated schema aspects,
  Query operations, Signal nodes, consumers, or reporting projections.
- Exact counters distinguish truth targets admitted, correspondence lookups,
  exact matches, widened matches by class, ambiguity denials, capacity denials,
  actual Signal seeds emitted, node fan-out, and slots touched.
- Authoritative installed inputs rebuild every correspondence and allocation
  index. Neither stable-name maps nor report digests become a second authority.
- The raw Signal graph crosses the Bridge boundary only when creating the
  opaque graph binding. Installation and authoritative-change delivery consume
  that binding, while Query retains an opaque owner-specific correspondence
  wrapper rather than projecting Bridge or Signal operational ingredients.
- Predictive modules separate semantic correspondence admission, slot
  allocation, production Signal target construction, widening classification,
  and typed denial; no generic aspect bridge bag owns all five.

**Open questions**
- The exact slot-allocation strategy may vary by Signal storage profile, but its
  capacity, equivalence, widening, and deterministic-observable contracts are
  fixed here.

### Phase 10: Query Conditional Translation, Runtime-Bridge Admission, And Signal Decision Authority

This phase has Query translate an admitted conditional-node declaration exactly
once into a neutral Bridge contract, which the runtime bridge admits and lowers
into an installed Signal node contract. Portable
condition definitions remain callback-free. Volatile lowerers, condition
resolvers, comparator providers, temporal wake sources, and on-demand trigger
providers register separately under the neutral Bridge contract and Signal
target marker types. Query retains exact declaration and authority continuity;
runtime construction rejects missing, extra, foreign, stale, or
semantic-key-lookalike registrations before an operation can bind.

Lowering consumes the Phase 9 installed semantic-aspect correspondence for
every declared truth trigger and read. It does not translate an `AspectKey` or
bridge stable name directly into a Signal slot. The installed node projection,
dependency versions, invalidation mask, and condition trigger basis are derived
from the exact pair-bound correspondences retained by the operation or stage.

Lowering joins each installed correspondence to, and spends, the exact bound
operation graph-read capability and existing Query graph-touch obligation plan.
The Signal invalidation mask is a derived narrowing for reactive evaluation,
not a replacement graph capability. Prepared execution may reduce realized
reads or suppress all later work, but neither the bridge, a condition resolver,
nor Signal may add a graph role, read scope, touch scope, effect family, or
operating-world authority absent from the bound operation plan.

The owner-specific lowering path reuses `worth-proof` recipe/artifact stages for
resolved, lowered, admitted, execution-ready, and executed progression. The
Phase 9 correspondence is spent as a concrete lowering capability witness; the
installed Signal runtime mints readiness and execution evidence. Query and the
Bridge do not define parallel readiness stages or flatten checked non-success
outcomes into a generic error.

The bridge mints a pair-bound lowering witness retaining the exact Query
declaration, installed operation and stage where applicable, runtime and graph
generation, installed graph adapter, bound graph participation, admitted
graph-read plan, graph-touch obligation evidence, Signal node identity, lowered
semantic and projection contract, condition, comparator/equivalence policy,
trigger owner, and support admission. The execution request extends the
existing invalidation, read-packet, and snapshot boundary with that witness;
callers cannot provide detached graph authority, condition, or comparator
alongside an otherwise valid request.

Signal applies invalidation and dependency narrowing, resolves the installed
condition and dependency-version comparator, and determines eligibility before
invoking node compute. After compute it applies the separately declared output
equivalence and reuse contracts. Signal alone mints the decision and outcome
evidence that distinguishes
eligible computation, condition-based ineligibility or suppression, temporal
or on-demand deferral, prepared/computed output, and semantically reverted-clean
output. The evidence is bound to the exact node, graph, runtime generation,
snapshot, dependency versions, condition, comparator, trigger or wake, and
evaluation attempt. The bridge transports it but cannot reinterpret or restamp
it.

**Relevant subsystems**
- runtime-bridge Query-to-Signal request admission and lowering
- Signal graph installation, invalidation planning, dependency capture,
  precompute condition resolution, comparator resolution, temporal lowering,
  on-demand triggering, and evaluation outcome
- installed graph-participation adapters and lower-runtime routing
- installed semantic-aspect correspondences and real Signal aspect masks
- exact evaluation and denial counters

**Relevant APIs**
- existing bridge Signal evaluation request, invalidation artifact, read packet,
  and snapshot request boundary
- Signal condition resolver, comparator resolver, prepared evaluation,
  temporal eligibility, evaluation outcome, and execution counters
- proposed conditional lowering registration, installed lowering witness,
  bound evaluation request, Signal decision evidence, and typed lowering denial
- Phase 9 correspondence witness as the only semantic-aspect-to-Signal target
  authority
- `worth_proof::Recipe`/`Artifact`, lowering and readiness capability witnesses,
  assumption basis, execution-ready/executed forms, and checked
  `TransitionOutcome` behind Bridge- and Signal-owned wrappers

**Warnings**
- A semantic key plus `Any`-stored resolver is not typed lowering authority.
  The installed witness retains the exact callable provider used by evaluation.
- A bridge stable-name scope, copied numeric `Aspect`, or locally assembled
  `AspectMask` cannot satisfy trigger or projection lowering. The Phase 9
  correspondence supplies the exact installed node-local targets.
- Raw `worth-proof` readiness, witness, or executed carriers cannot bypass the
  installed Bridge/Signal wrapper. Concrete owner marker types are sealed and
  no public API accepts a caller-selected authority/capability generic.
- The bridge does not evaluate conditions, compare versions, manufacture a
  temporal wake, or turn an on-demand label into a trigger. Signal owns those
  runtime decisions.
- Signal must not import Query declarations as its semantic truth. The bridge
  lowers into Signal-owned contracts and retains the pair-bound provenance.
- Signal invalidation is not Query graph-touch authority. A valid Signal node,
  mask, decision, or derived dependency packet cannot authorize a read or
  mutation outside the exact predecessor graph plan, even when it would be safe
  according to Signal's local graph.
- If Signal's existing `Custom(String)` or `DeltaThreshold(f64)` representation
  cannot preserve the admitted typed custom-condition identity or canonical
  unitful threshold without string/numeric reinterpretation, repair that Signal
  substrate first. Encoding a Query marker or unit into the string is forbidden.
- Eligibility is decided before compute. Post-compute equivalence may yield a
  reverted-clean outcome, but it cannot erase the fact or cost of computation.
- A skipped/deferred node must not invoke the domain compute callback. A
  reverted-clean node must not be reported as skipped computation.

**Test requirements**
- Adversarial lowering convergence test: equivalent packages, registration
  order, facade entry paths, and rebuilt lowering indexes produce identical
  installed Signal contracts, witnesses, decisions, outcomes, and counters.
- Adversarial registration test: missing, extra, wrong-marker, foreign-runtime,
  stale-generation, and semantic-key-lookalike condition/comparator/trigger
  providers reject runtime construction or operation binding before Signal
  graph mutation or evaluation.
- Adversarial dimension sabotage test: drift declaration, installed operation,
  stage, bound graph participation, admitted graph-read plan, graph-touch
  obligation evidence, graph adapter, semantic-aspect correspondence, truth
  contract, Signal node/aspect slot, graph generation, basis, snapshot,
  dependency versions, condition, comparator, trigger, wake, or attempt one at
  a time and assert the earliest typed denial with exact zero later work.
- Adversarial scope-widening test: make the condition true and provide a valid
  local Signal target whose read, invalidation, touch, or effect scope exceeds
  the bound operation plan; lowering denies before Signal mutation/evaluation.
  Narrowing to any admitted subset succeeds with identical predecessor graph
  authority and exact bounded-work counters.
- Adversarial comparator-role test: substitute dependency comparator, output
  equivalence, or reuse equivalence for one another and prove typed rejection;
  no same-named provider or equal reporting digest satisfies another role.
- Adversarial execution test: ineligible, suppressed, temporal-deferred, and
  on-demand-deferred nodes invoke compute zero times; eligible nodes invoke the
  exact installed compute provider once; reverted-clean output records compute
  once and downstream semantic change zero times.
- Adversarial ordering test: condition and comparator resolution happen after
  admitted invalidation/dependency narrowing and before compute, independent of
  serial or parallel scheduling; parallelism cannot change semantic outcomes.
- Adversarial decision-construction test: copied outcome labels, matching
  reporting digests, resolver return values, or component tuples cannot mint
  Signal decision evidence or satisfy Query re-entry.
- Adversarial progression-substrate test: unresolved, merely resolved, lowered-
  but-unadmitted, stale, rebind-required, or caller-constructed raw proof forms
  cannot execute; the ordinary facade and checked `worth-proof` path converge on
  identical readiness, decision, outcome, basis, and counters.

**Engineering decisions**
- Runtime-bridge owns admission and transport; Signal owns installed node
  evaluation truth; Query owns neither lower-runtime decision.
- `worth-proof` owns the generic lower/admit/ready/execute law; the Bridge and
  Signal own the stronger payloads, concrete witnesses, runtime state, and
  evaluation semantics carried through that law.
- The pair-bound lowering witness is derived from authoritative installed state
  and rebuildable indexes. Destroying an index cannot change admission or
  outcomes.
- Conditional lowering consumes one predecessor graph authority chain. Query's
  existing graph-read and graph-touch systems remain the only capability
  minting boundaries; Bridge and Signal produce scoped execution evidence, not
  parallel graph authority.
- Condition checks and comparator checks have separate exact counters from
  domain compute, semantic-change, and downstream delivery work.
- Predictive modules separate conditional registration, lowering admission,
  Signal request construction, decision transport, and denial. No generic
  bridge helper bucket is permitted.

**Open questions**
- Exact representation of the pair-bound witness may follow existing bridge
  capability envelopes, but it must remain runtime-affine, opaque, non-
  serializable, and non-reconstructable.

### Phase 11: Query Re-entry And Conditional Consequence Propagation

This phase admits Signal-owned decision and outcome evidence back into the
exact bound Query operation or workflow-stage attempt. Query validates retained
operation, stage, runtime, graph, basis, snapshot, condition, comparator,
trigger, attempt, bound graph participation, graph-read plan, and graph-touch
obligation continuity, then mints the Query execution or stage receipt that
later phases already consume. Query records what Signal decided; it does not
produce an equivalent local eligibility enum or conditional graph capability
with independent authority.

Query re-entry uses `worth-proof` checked admission and assumption-basis law.
An in-process opaque evidence chain may retain its current basis when the exact
owner-issued carrier remains intact. Evidence that crosses serialization,
reconstruction, provider, process, or another trust boundary becomes an
explicit boundary-bridged freshness form and must be readmitted by concrete
Query authority; it cannot deserialize directly into current Query authority.
Foundational provenance/boundary evidence may describe that crossing and its
freshness, but cannot perform the operational readmission.

Conditional meaning then remains attached through publication, projection
consumption, settlement, workflow traces, ordinary re-execution, cert-only
replay, reversal inspection, lineage, native access where relevant,
compatibility, lifecycle, dependency impact, sharing, invalidation, opaque
identity, exact cost evidence, collection windows, and patch delivery. Each
later phase consumes the same retained condition and decision authority only
where it affects that phase. A non-evaluated node cannot invent output or a
publication; a reverted-clean outcome may preserve an admitted prior result
without claiming new semantic change or delivery.

Re-entry separates authorized potential scope from realized work. The retained
bound operation capability remains the sole authority for graph reads, touches,
and effects. Signal evidence can prove that an admitted subset was read,
invalidated, computed, unchanged, or changed; Query may advance the ordinary
touch/effect progression only for actual changed consequences already declared
by the operation. Ineligible, suppressed, and deferred outcomes mint no touch
or effect. Reverted-clean outcomes retain admitted read and compute cost but
mint no new mutation touch, effect, semantic invalidation, or delivery.

**Relevant subsystems**
- installed execution and workflow stage progression
- result-state, warning, publication, consumption, and settlement authority
- live promotion, dependency impact, invalidation, sharing, replay, lifecycle,
  collection delivery, and cost evidence
- inspection and causal explanation projections

**Relevant APIs**
- bound operation and workflow-run capabilities
- Query execution, stage, publication, consumption, trace, impact, invalidation,
  patch, and settlement receipts
- proposed Query admission of Signal decision evidence and conditional outcome
  view with no independent construction or restamping path
- `worth_proof::AssumptionBasis`, checked outcomes, current/stale/rebind/
  authority-revalidation freshness, boundary bridging, and readmission behind
  Query-owned evidence/receipt wrappers
- Foundational provenance, boundary evidence, freshness, and receipt attachments
  for derived cross-crate explanation and support surfaces

**Warnings**
- Do not map Signal evidence into a free-standing Query enum and discard the
  retained bridge witness. A presentation enum may be derived for inspection
  only and has zero progression authority.
- Do not weaken every ordinary in-process call merely because it crosses a
  crate boundary; trust weakening is required when current owner-issued basis
  cannot be preserved. Conversely, serialization or reconstructed component
  tuples can never preserve current basis implicitly.
- Foundational provenance or receipt attachments are descriptive boundary
  meaning. They cannot replace the retained Signal decision, Bridge witness, or
  Query-owned readmission result.
- `not computed`, `no semantic change`, `not delivered`, and `failed` are
  different outcomes. They retain distinct cause, result-state, warning, and
  counter posture.
- Replay must compare the installed conditional contract and decision path, not
  merely terminal values. A different condition that happened to compute the
  same value is semantic divergence.
- Dependency impact and invalidation consume declared trigger inputs plus the
  realized Signal decision; consumers do not infer them from output presence.
- Dependency impact and consumer invalidation are derived consequences, not
  operational graph authority. Neither may be fed back as a graph-touch
  capability or used to widen the retained operation plan.
- Sharing cannot coalesce conditional nodes unless Query proves equality of
  condition, comparator, trigger/wake basis, installed Signal graph, lifecycle,
  and all existing capability-equivalence dimensions.

**Test requirements**
- Adversarial full-journey transcript: install a real conditional operation,
  bind through the single root, lower through the bridge, obtain a real Signal
  decision, execute or defer, and reach the legal Query terminal state without
  raw authority ingredients or test-only constructors.
- Adversarial re-entry test: cross-operation, cross-stage, cross-run, cross-
  runtime, cross-graph, cross-basis, cross-snapshot, cross-attempt, stale, and
  condition/comparator-drifted Signal evidence, plus matching Signal evidence
  carrying a foreign or missing predecessor graph-read/touch capability, denies
  before Query execution, workflow advancement, publication, or impact work.
- Adversarial freshness/readmission test: intact in-process evidence preserves
  its exact current basis; serialized, reconstructed, foreign-provider, stale-
  generation, and boundary-bridged evidence requires explicit Query readmission
  and preserves denied/stale/rebind-required/authority-revalidation outcomes
  without minting a receipt early.
- Adversarial outcome matrix: always, aspect-filtered, threshold, temporal,
  on-demand, and typed domain-specific conditions exercise eligible,
  ineligible/suppressed, deferred, computed-changed, and reverted-clean paths;
  every posture survives through its legal terminal receipt unchanged.
- Adversarial consequence test: non-evaluated nodes mint no new publication,
  lineage, operational touch/effect, impact, invalidation, patch, or computed
  counters; reverted-clean nodes retain admitted read and compute cost but mint
  no mutation touch, effect, or semantic-change delivery; changed outputs take
  the ordinary pre-admitted touch/effect, publication, and downstream path
  exactly once.
- Adversarial one-authority trace test: the operation's ordinary and conditional
  paths retain the same graph capability identity, obligation evidence, adapter,
  generation, and basis from binding through terminal trace. No conditional-
  only touch receipt exists, and destroying derived conditional/impact indexes
  cannot change admission or realized touch/effect evidence.
- Adversarial replay and sharing test: equivalent installed condition paths
  converge across ordinary re-execution and cert replay, while any condition,
  comparator, trigger, wake basis, or Signal graph drift prevents equivalence
  and sharing even when terminal values and reporting digests match.
- Adversarial rebuild test: destroy and rebuild Query condition, bridge lowering,
  Signal node, dependency, and execution indexes, then prove identical
  admissions, denials, phase outcomes, identities, and exact counters.

**Engineering decisions**
- Query re-entry is a validation and proof-widening boundary, not an evaluation
  boundary. Signal decision authority remains retained and inspectable.
- Query re-entry joins realized conditional evidence to the existing bound
  graph progression; it never mints, reconstructs, or restamps graph authority
  from correspondence, invalidation, decision, impact, or reporting artifacts.
- `worth-proof` owns basis/freshness/readmission progression; Query owns the
  concrete admission authority, semantic checks, and stronger execution/stage
  receipt. Foundational owns only the portable descriptive attachment.
- Later phases consume a common retained conditional provenance record rather
  than maintaining replay-, invalidation-, sharing-, or delivery-specific
  condition mirrors.
- Causal inspection may explain why a node evaluated, deferred, suppressed, or
  reverted clean, but its report cannot advance lifecycle or authorize reuse.
- This phase completes the retrofit of Phases 1-6 before later 9.14 authorities
  build on conditional execution.

**Open questions**
- None. Any implementation that leaves Query authoring unable to create the
  real Signal condition, or leaves Query unable to distinguish Signal outcomes,
  is incomplete even if the bridge can execute an always-eligible node.

### Phase 12: Re-execution Semantics And Cert-Only Replay Authority

This phase separates ordinary retry and re-execution from certification-only
replay. It distinguishes four mechanisms: idempotent retry of one stage
attempt, full ordinary re-execution from installed intent against the same
basis, certification replay against an admitted same or historical basis, and
durable restart reconstruction from a persisted journal. Ordinary entry
facades may expose the first two without access to the original execution
trace. Trace-driven replay is exported only through Query's certification
audience. Durable journal persistence, checkpoint recovery, and restart
reconstruction remain Store-owned.

A completed trace mints certification replay capability only when the installed
operation declares an admitted replay posture and the caller holds the cert
audience capability. Certification replay consumes normalized intent, the
installed operation definition, exact bound context, and the original trace,
then creates a new run and attempt identity. Equivalence is determined by the
operation's declared semantic comparator over typed results, publications,
effects, lineage, installed conditional-node meaning, realized Signal decision
path, and explicitly allowed nondeterministic noise—not by raw
execution identity, digest equality, or a consumer-maintained parity checklist.

**Relevant subsystems**
- installed operation replay posture and normalized intent
- workflow run trace and terminal execution outcome
- basis lifecycle, historical evaluation, and correspondence
- result, effect, lineage, divergence, and exact-counter inspection

**Relevant APIs**
- completed workflow execution trace and bound operation capability
- Query basis use and historical evaluation capabilities
- ordinary retry and re-execution through the installed operation facade
- cert-audience replay admission, replay run capability, semantic equivalence
  contract, replay comparison, and typed divergence result
- `worth-proof` concrete cert capability witness, assumption-basis freshness,
  boundary readmission, and checked replay-admission outcome
- Foundational canonical comparison, provenance, lineage/continuity,
  executed/completed receipt, and boundary-evidence attachments for portable
  replay explanation and provider parity

**Warnings**
- Replay is not reversal and does not promise to undo the original operation.
- Ordinary entry, derived, UI, app, resolver, solver, and pack crates cannot
  import or receive the completed-trace replay capability.
- An ordinary downstream stage that needs upstream meaning consumes the
  declared publication; reopening the upstream trace is a boundary failure.
- Raw run identifiers, trace digests, output digests, and byte equality cannot
  define semantic equivalence unless the installed operation explicitly names
  byte identity as its domain guarantee.
- Foundational canonical/lineage/receipt artifacts describe and compare replay
  meaning across providers; they cannot grant cert audience access, mint the
  replay run, or replace the stronger Query replay capability.
- A caller-defined generic proof authority cannot act as the cert witness.
  Replay admission consumes one sealed concrete Query certification capability.
- Replay may re-evaluate an installed condition against its admitted replay
  basis, but it cannot reuse the original runtime decision as current
  eligibility. Comparison covers both condition semantics and realized
  decision paths and reports typed divergence where they differ.
- A new replay attempt must never reuse the original run or attempt identity,
  even when every semantic result converges.
- This phase must not imply durable journal storage, checkpoint survival, or
  restart reconstruction; those remain Store contracts.
- Allowed nondeterminism must be typed and bounded. An open-ended "ignore
  differences" callback would make replay unfalsifiable.

**Test requirements**
- Adversarial same-basis convergence test: an admitted semantic re-execution
  creates distinct run and attempt identities while proving equivalent typed
  intent, result, effect, lineage, and exact-counter posture.
- Adversarial replay-fence test: ordinary entry and downstream consumer crates
  cannot import, receive, or reconstruct trace-driven replay authority, while
  the cert audience can replay the same installed operation through the sealed
  facade.
- Adversarial publication-path test: the ordinary downstream path converges
  with certification replay outcomes while consuming only the typed derived
  publication and never accessing the execution trace.
- Adversarial historical-basis test: admitted certification replay records
  the exact basis relationship and correspondence evidence; a foreign,
  unrelated, stale, or unsupported basis denies before stage execution.
- Adversarial divergence test: omitted stages, changed semantic outputs,
  changed effects, broken lineage, or meaningful ordering changes localize a
  typed divergence even when reporting digests collide.
- Adversarial allowed-noise test: explicitly declared ordering or diagnostic
  noise converges without masking semantic output, effect, lineage, or
  invariant drift.
- Adversarial path-parity test: snapshot, live-triggered re-execution, and
  admitted replay paths consume the same installed operation semantics and
  produce comparable traces without a replay-specific operation catalog.
- Adversarial conditional replay test: condition, comparator, trigger/wake
  basis, or decision-path drift localizes semantic divergence even when
  terminal output matches; appropriately changed runtime inputs remain an
  explicit outcome difference rather than false execution-identity equality.
- Adversarial substrate-boundary test: Foundational replay evidence and
  `worth-proof` freshness/outcome categories converge across providers, while
  copied attachments, canonical digests, bridged-but-unreadmitted basis, and
  caller-defined witnesses cannot obtain Query replay authority.

**Engineering decisions**
- Ordinary re-execution derives from installed intent and current bound
  authority; certification replay admission additionally requires the original
  completed trace and cert-audience capability.
- A replay result carries original and replay identities, shared normalized
  intent, basis relationship, trace comparison, output/effect/lineage
  equivalence, divergence locus, and exact counters.
- Idempotent retry, semantic re-execution, historical re-execution, and durable
  restart replay remain separately named because their authority, cost, and
  failure contracts differ.
- Query owns the audience fence, replay admission, semantic comparison
  orchestration, and owner-specific replay artifacts; `worth-proof` owns their
  generic progression law, domains own the comparator semantics they declare,
  and Store owns persistence and recovery mechanics.

**Open questions**
- None. Any future cross-process replay transport must readmit portable meaning
  through the runtime rather than serializing operational replay authority.

### Phase 13: Reversal, Compensation, And Irreversibility Contracts

This phase makes aftermath posture explicit and binds any reversal path to the
original installed operation and completed execution trace. The operation
definition must distinguish provisional discard, exact inverse, compensating
operation, rebuild or recovery procedure, and irreversible completion. Query
does not infer inverse behavior from touched scope and does not pretend every
operation is undoable.

When reversal or compensation is declared, the domain supplies its semantic
lowering and postcondition contract. Query admits it only against the exact
original operation, execution trace, effect and touch closure, basis,
installation generation, and current lifecycle. A successful inverse or
compensation is a new ordinary authoritative operation with its own run and
receipt referencing the original; it is never a mutation of history or a
consumer-authored "undo completed" label.

Aftermath admission reuses `worth-proof` recipes/artifacts, exact assumption
bases, checked outcomes, and concrete Query/domain capability witnesses for the
declared -> bound-to-completion -> admitted -> executed progression.
Foundational provenance, lineage, boundary evidence, and executed/completed
receipt vocabulary describe the relationship to the original run, but only the
stronger Query aftermath capability can authorize inverse or compensation work.

**Relevant subsystems**
- installed operation aftermath and irreversibility posture
- completed workflow trace, graph touches, effects, and invariants
- authority-scoped operation execution and lifecycle
- recovery, inspection, and exact accounting

**Relevant APIs**
- completed execution trace and installed effect/touch contracts
- ordinary bound operation execution and authority-scoped effects
- proposed reversal admission, exact-inverse capability, compensation
  capability, irreversible denial, and aftermath receipt
- `worth_proof::TransitionOutcome`, assumption basis, freshness/readmission,
  and concrete capability witnesses plus Foundational provenance/lineage/receipt
  vocabulary behind owner-specific aftermath types

**Warnings**
- Touched scope proves where an operation acted; it does not prove how to undo
  that action.
- Replay repeats semantic intent. Reversal or compensation changes truth under
  a separately declared semantic contract. The two capabilities must not share
  a misleading generic "recovery" entrypoint.
- Compensation may restore a declared invariant or business postcondition
  without restoring byte-identical prior state. Its guarantee must state which.
- Irreversibility is a typed operational posture, not a missing callback or a
  runtime string error.
- Rebuild and Store recovery must not be presented as exact inverse operations.
- A Foundational receipt or raw `worth-proof` artifact cannot be restamped as an
  inverse/compensation capability. Non-success admission branches mint no
  aftermath authority that can be recovered later.
- A condition that prevented evaluation produced no new operation effect to
  reverse. Reverted-clean computation may still have cost and trace evidence,
  but cannot acquire an inverse merely because a stage attempt occurred.

**Test requirements**
- Adversarial scope-forgery test: copied touch sets, effect labels, operation
  names, or matching digests cannot mint exact inverse or compensation
  authority.
- Adversarial continuity test: a reversal capability from another execution,
  runtime, installation generation, basis, or lifecycle cannot act on the
  target operation even when touched scope and outputs match.
- Adversarial posture test: exact inverse, compensation, provisional discard,
  rebuild/recovery, and irreversible operations expose distinct callable
  surfaces, results, postconditions, failures, and exact counters.
- Adversarial semantic test: exact inverse proves its declared restoration
  semantics; compensation proves its declared postcondition without claiming
  byte equality; irreversible operations deny before lowering or mutation.
- Adversarial partial-failure test: failed reversal or compensation preserves
  the original trace, reports partial effects and required recovery posture,
  and cannot mint a successful aftermath receipt.
- Adversarial conditional-aftermath test: deferred, suppressed, and ineligible
  outcomes cannot mint reversal authority; changed conditional executions bind
  reversal only to the exact resulting trace and effects.
- Adversarial substrate-boundary test: stale assumption bases, raw proof
  artifacts, caller-selected witness markers, and matching Foundational receipts
  retain their proper checked outcome or descriptive strength but cannot mint or
  recover an owner-specific aftermath capability.

**Engineering decisions**
- Domain packages own inverse and compensation semantics and lowerings. Query
  owns their installed declaration, binding to original authority, progression,
  denial, receipt, and inspection contracts.
- Every reversal or compensation executes as a new operation with distinct
  identity and an explicit relation to the original execution.
- Touch/effect closure is necessary evidence but never sufficient authority.
- `worth-proof` owns the reusable progression law and Foundational owns shared
  receipt/provenance vocabulary; neither replaces Query's aftermath admission or
  the domain's inverse/compensation semantics.
- Durable rollback journals, checkpoints, and restart recovery remain Store-
  owned and consume these semantic contracts without redefining them.

**Open questions**
- None. Operations whose honest aftermath posture is not yet known must install
  as explicitly irreversible or unsupported rather than exposing a weak undo.

### Phase 14: Typed Lineage And Identity-Evolution Binding

This phase binds Query's existing lineage, correspondence, and identity-
evolution semantics to installed operation outputs and the Query-minted
execution trace. Operations declare how produced, preserved, split, merged,
retired, or ambiguous identities are evidenced. Query then carries typed
lineage outcomes from the exact stage and effect that established them rather
than allowing consumers to reconstruct continuity from geometry, labels,
digests, array position, or reporting representations.

The standard outcome family distinguishes preserved identity, singular
successor, plural split successors, merged predecessor set, generated identity,
retired identity, ambiguous or advisory correspondence, and continuity break.
Persistent naming, replay comparison, dependency impact, compatibility, UI
selection continuity, and derived-product reuse consume this same lineage
contract. Ambiguous or advisory correspondence may be presented for
adjudication but cannot silently authorize persistent naming.

Promotion on reference is the ordinary sparse-identity path. An artifact
subelement remains payload until a durable declared reference requests graph
identity. Query admits promotion only from the operation's installed promotion
contract, exact carrying publication, typed reference intent, and trace-bound
lineage evidence. Merely producing, selecting, displaying, or caching a
subelement does not promote it.

Query reuses Foundational provenance, lineage, continuity/divergence,
promotion, freshness, receipt, and boundary-evidence attachment vocabulary when
this trace-bound meaning crosses crate or support boundaries. The Query trace
and identity-evolution result remain the stronger operational artifacts;
Foundational attachments describe them without minting continuity authority.

**Relevant subsystems**
- existing lineage, structural correspondence, and identity evolution
- installed operation output and effect contracts
- workflow execution trace and typed native result carriers
- persistent naming, compatibility, invalidation, replay, and consumer
  continuity
- derived publications, durable reference intent, and sparse promotion

**Relevant APIs**
- lineage traversal and correspondence artifacts from Milestones 5.4 and 7
- runtime authoritative mutation evidence and existing-truth binding
- Foundational native identity and value types
- Foundational boundary-evidence provenance, lineage, continuity/divergence,
  promotion, freshness, receipt, attachment, and readmission vocabulary
- proposed operation-lineage declaration, trace-bound lineage evidence,
  identity-evolution result, and persistent-naming admission
- graph-schema promotion grammar and proposed promotion-on-reference admission

**Warnings**
- Coordinates, topology summaries, rendered values, debug strings, raw IDs,
  labels, and digests are observations or representations, not continuity
  authority.
- Similar geometry or equal output values may support correspondence but cannot
  promote advisory evidence into preserved identity.
- Persistent naming policy remains domain-owned; Query owns the proof-bearing
  binding and typed evolution result, not product-specific naming rules.
- Do not create separate lineage stories for replay, invalidation, UI
  continuity, and derived-product reuse. They must consume the same trace-bound
  evidence.
- Plural split and merge outcomes must not collapse into an optional singular
  successor.
- Dense artifact payload must not acquire graph identity eagerly. Promotion is
  caused by an admitted durable reference, not by publication or observation.
- Ineligible, deferred, suppressed, and reverted-clean outcomes cannot mint new
  output lineage. Preserving an admitted prior result preserves only the
  already-authoritative lineage attached to that result.
- A Foundational provenance row, lineage attachment, canonical digest, or
  support-basis readmission is not Query persistent-naming authority. Only the
  exact trace-bound Query admission may produce the operational result.

**Test requirements**
- Adversarial representation test: raw strings, raw identifiers, digests,
  coordinates, rendered geometry, debug output, and copied correspondence rows
  cannot authorize persistent naming or operational identity continuity.
- Adversarial split/merge test: one-to-many splits, many-to-one merges,
  generated identities, retirements, and breaks produce exact typed outcomes
  with no silent singularization or predecessor loss.
- Adversarial trace-continuity test: lineage evidence from another execution,
  stage, operation, runtime, basis, or generation cannot bind to the target
  output even when its reporting representation matches.
- Adversarial replay-parity test: semantically equivalent replay produces the
  declared lineage-equivalent outcome under a distinct execution identity;
  lineage divergence localizes replay failure.
- Adversarial boundary-evidence test: Query trace lineage materializes the same
  Foundational attachment across equivalent facade/provider paths, while copied,
  stale, reconstructed, support-only, or digest-only attachments cannot promote
  identity or satisfy persistent naming.
- Adversarial ambiguity test: ambiguous or advisory correspondence remains
  inspectable but cannot mint persistent naming, compatibility, reuse, or
  preservation authority without an admitted domain adjudication.
- Adversarial boundedness test: lineage binding is proportional to declared
  output and correspondence width and does not scan unrelated execution
  traces, identities, or consumer state.
- Adversarial promotion test: producing, projecting, rendering, caching, or
  selecting an artifact subelement cannot mint graph identity; the same
  subelement gains identity only when an admitted durable reference presents
  matching carrying-publication and lineage evidence.
- Adversarial sparse-identity test: promotion work is proportional to newly
  referenced subelements and does not enumerate unreferenced artifact payload.
- Adversarial conditional-lineage test: copied prior output, a no-change label,
  or a detached Signal decision cannot mint fresh lineage or promotion evidence
  for a non-evaluated or reverted-clean attempt.

**Engineering decisions**
- Existing Query identity-evolution semantics are reused and bound to installed
  operation traces; this milestone does not invent a parallel lineage engine.
- Foundational owns native identity and value carriers; domain packages own
  naming and correspondence semantics; Query owns admission, trace binding,
  typed evolution outcomes, and proof-preserving transport.
- Persistent naming accepts only typed lineage evidence from an admitted
  operation trace or an explicitly adjudicated correspondence capability.
- Promotion-on-reference consumes the graph schema's promotion grammar; Query
  binds and records the authority but does not invent product naming semantics.
- The same lineage result feeds replay, reversal inspection, dependency impact,
  compatibility, reuse, invalidation, and consumer continuity.

**Open questions**
- The exact generic vocabulary for domain-specific persistent names remains an
  implementation choice, but no public surface may accept representation-level
  substitutes for typed lineage evidence.

### Phase 15: Declaration-Indexed Native Access

This phase derives native value access from the admitted projection declaration
and Foundational contract. Declaring a projected field produces a typed access
key or materially equivalent proof that can refine the exact selected fact in
constant time without consumer-side path matching, fact scans, string parsing,
or local value-family switches.

**Relevant subsystems**
- Foundational native aspect contracts and canonical scalar/struct carriers
- Query projection declaration and materialized fact consumption
- native refinement and absence posture

**Relevant APIs**
- `ProjectionFactFieldPath`
- `ConsumedFieldValueFact` native refinement methods
- `AspectValue`, `StructAspectValue`, and canonical Foundational wrappers
- proposed declaration-derived native access key and access denial

**Warnings**
- A raw field path is location vocabulary, not proof that a value was declared,
  admitted, materialized, and retained by this capability.
- Do not introduce a Query-owned mirror of Foundational scalar or struct
  families to improve autocomplete.
- Absence, null, missing materialization, unsupported family, and struct shape
  mismatch are distinct typed outcomes.
- Typed access must not conceal a linear scan of all facts or source rows.

**Test requirements**
- Adversarial native parity test: every Foundational scalar family, complete
  multi-field values under Foundational's current flat `StructAspectValue`
  law, dotted atomic aspect keys, canonical floats, and admitted absence
  postures observed through the bound capability equal Query's native
  consumption oracle exactly. Nested structs become a requirement only when
  Foundational owns and admits a nested carrier; Query must not invent one.
- Adversarial access rejection test: a key from another declaration, field,
  contract revision, domain installation, or value family cannot access or
  refine a fact, even when its printable field path is identical.
- Adversarial bounded lookup test: accessing `k` declared native values performs
  exactly `k` indexed accesses and refinements as unrelated facts, rows, views,
  and installed domains grow.

**Engineering decisions**
- Query derives access keys from the admitted declaration and contract; callers
  never manufacture them from paths.
- Native carriers remain Foundational-owned and are borrowed or moved without
  stringify/reparse or JSON-shaped intermediates.
- Foundational currently defines struct fields as `AspectValue` leaves, so
  recursive struct nesting is not representable. This phase proves complete
  flat structs and atomic dotted aspect keys and deliberately does not create a
  Query-owned recursive value family.
- Unsupported refinement preserves field, contract, source, and projection
  authority context in a typed denial.

**Open questions**
- The final accessor grammar may use generated typed selectors, declaration
  return values, or both; the no-scan and no-forging properties are fixed.

### Phase 16: Authority-Native Compatibility Decisions

This phase moves downstream sameness and transition decisions into Query.
Consumers ask semantic questions such as whether two capabilities share an
installation, may replace one another, may rebind, may share execution, or
belong to a compatible basis. Query returns a sealed typed witness for the
admitted relationship or a typed denial identifying the first incompatible
authority dimension.

Portable/cross-provider semantic comparison reuses Foundational canonical basis,
equivalent/mismatched/unsupported comparison, producer-shape, and structured
mismatch vocabulary. Query then performs the stronger runtime-affine admission
and returns an opaque pair-bound witness backed by `worth-proof` progression and
concrete Query authority. Foundational comparison or digest evidence alone can
never mint that operational witness.

For conditional capabilities, compatibility also includes the portable node
contract, installed bridge lowering, Signal graph and node generation,
condition family and parameters, comparator/equivalence posture, trigger or
wake basis, and lifecycle state relevant to the requested relationship.

**Relevant subsystems**
- installed-domain authority and generation compatibility
- basis capability lifecycle and projection contract equivalence
- replacement, rebind, reuse, and execution-sharing admission

**Relevant APIs**
- installed-domain authority witnesses
- basis capability envelopes and use receipts
- proposed installation, replacement, rebind, and basis compatibility proofs
- Foundational canonical basis, comparison, producer-shape, identity-boundary,
  and structured mismatch surfaces
- `worth-proof` checked outcomes and concrete Query authority witness behind the
  pair-bound operational compatibility result

**Warnings**
- Boolean helpers discard the proof later work needs and encourage callers to
  repeat the comparison.
- Equality of labels, canonical definitions, fact-set digests, or reporting
  projections cannot prove operational compatibility.
- Foundational canonical equivalence proves portable semantic sameness only at
  its named comparison boundary. Query must still prove runtime, installation,
  generation, basis, lifecycle, and relationship-specific admission.
- Do not collapse replacement, rebind, reuse, execution sharing, and same-
  installation into one generic compatibility enum; their proofs and failure
  modes differ.
- Compatibility checks may inspect retained authority but may not reopen lower
  runtimes or re-execute Query work.

**Test requirements**
- Adversarial equivalence test: every admitted compatibility witness agrees
  with the canonical Query authority oracle across equivalent facade paths,
  declaration order, and derived-index rebuild.
- Adversarial collision test: matching labels, definitions, digest text,
  reporting output, and copied metadata cannot admit foreign-runtime,
  stale-generation, wrong-basis, wrong-contract, or wrong-lifecycle pairs.
- Adversarial conditional drift test: one-field condition, comparator, trigger,
  temporal basis, installed Signal node, or bridge-lowering drift returns its
  exact typed denial before reuse, replacement, rebind, or sharing work even
  when current outputs and reporting digests match.
- Adversarial proof-use test: replacement, rebind, reuse, and execution-sharing
  entrypoints reject a witness for the wrong relationship or capability pair at
  compile time or before operational work.
- Adversarial canonical-comparison test: equivalent, mismatched, and unsupported
  Foundational comparisons retain their exact category; only equivalent meaning
  proceeds to Query authority admission, and equal digests or bridged identities
  cannot skip that second boundary.

**Engineering decisions**
- Compatibility witnesses bind both exact capabilities and the named semantic
  relationship they prove.
- Foundational owns shared comparison/mismatch vocabulary and `worth-proof` owns
  checked progression; Query owns the concrete operational compatibility law,
  pair binding, and witness minting.
- Denials preserve machine-readable installation, generation, basis, contract,
  lifecycle, and view-shape mismatch categories without leaking forgeable
  authority components.
- Derived compatibility indexes may accelerate lookup but are rebuildable and
  never sufficient without retained authority.

**Open questions**
- None.

### Phase 17: Capability-Bound Lifecycle Transitions

This phase attaches live promotion, replacement, rebind, cancellation, and
disposal to the bound projection capability. The framework tracks every managed
resource created from it, while transition-specific capability states make
illegal lifecycle calls unavailable.

**Relevant subsystems**
- live query promotion and managed resource lifecycle
- installed-domain execution drift and explicit rebind
- replacement, cancellation, disposal, and continuation

**Relevant APIs**
- managed live query resources
- installed-domain rebind receipts and drift counters
- proposed live-bound, rebound, replaced, and disposed capability states
- `worth-proof` artifact phases, assumption basis, current/stale-readable/rebind-
  required/authority-revalidation freshness, checked transitions, and concrete
  Query lifecycle witnesses behind owner-specific capability states

**Warnings**
- A live subscription identifier beside a snapshot capability is detached
  lifecycle state.
- Disposal must invalidate operational use without pretending to erase durable
  or diagnostic evidence.
- Automatic retargeting across installation generations, bases, or runtimes is
  forbidden; drift requires a typed explicit transition.
- Do not create Query-local freshness or rebind enums that duplicate
  `worth-proof`. Query supplies the concrete basis payload, semantic validation,
  and lifecycle wrapper; shared progression law remains reused.
- Rebind and replacement of a conditional capability must readmit its lowering,
  Signal graph/node, condition support, comparator, and trigger or temporal
  basis. A live resource cannot carry an old Signal decision into a new
  lifecycle generation.
- Lifecycle convenience must not erase the distinct costs or failure modes of
  snapshot execution, live maintenance, rebind, and replacement.

**Test requirements**
- Adversarial lifecycle convergence test: one-shot, promoted-live, explicitly
  rebound, and replacement paths preserve canonical declaration and authority
  meaning where the respective transition is admitted.
- Adversarial drift test: stale generation, foreign runtime, disposed resource,
  wrong basis, and changed contract deny before planning, lower-runtime,
  maintenance, or delivery work with exact-zero counters.
- Adversarial conditional lifecycle test: stale trigger ownership, temporal
  wake basis, Signal node generation, or condition lowering prevents promotion,
  replacement, and rebind before maintenance; an admitted transition mints a
  fresh evaluation attempt and never reuses prior eligibility evidence.
- Adversarial ownership test: dropping, cancelling, replacing, or disposing a
  managed capability leaves no orphan subscription or resource, and double
  disposal or use-after-disposal is uncallable or typed-denied.
- Adversarial freshness-topology test: current, stale-readable, rebind-required,
  authority-revalidation-required, rebound, replaced, and disposed states retain
  their distinct `worth-proof` progression and cannot be interconverted by
  copying the Query basis payload or constructing a generic witness.

**Engineering decisions**
- The framework owns resource registration, continuation, and disposal.
- Each lifecycle transition consumes the preceding state and returns the next
  proof-bearing state plus retained inspection evidence.
- Query-specific lifecycle wrappers reuse `worth-proof` basis/freshness/
  transition machinery and seal concrete markers; `worth-proof` does not own
  resource registration, cancellation, or subscription mechanics.
- Explicit rebind requires Query-minted compatibility proof from Phase 16.

**Open questions**
- None.

### Phase 18: Query-Compiled Semantic Dependency And Impact Closure

This phase makes every bound projection carry the complete semantic dependency
contract that determines how changes affect its meaning. Query compiles the
contract from the installed operation definition, validated workflow graph,
Query-minted stage trace, admitted graph-read plan, touch and effect closure,
portable conditional trigger inputs, installed condition and comparator
meaning, realized Signal decision evidence, lineage outcomes,
Foundational-native contracts and masks, result shape,
collection ordering, window policy, support posture, and installed domain
invariants. Consumers may inspect the resulting roles and impact but cannot
author a competing graph or recompute policy.

Dependency roles remain explicit: operational identity, selection or
membership, ordering, projected value, grouping, window boundary, support and
lifecycle, conditional eligibility or semantic cleanliness, installed domain
invariant, and advisory-only context do not have
the same invalidation consequence. The compiled closure converts a lower change
into the narrowest honest impact class before consumer invalidation or patch
delivery begins.

**Relevant subsystems**
- installed view and workflow definitions
- validated workflow graph, stage trace, touch/effect closure, and lineage
- read planning, projection consumption, result shape, ordering, and grouping
- native aspect contracts, masks, and installed domain invariants
- live relevance, invalidation narrowing, and patch classification

**Relevant APIs**
- declaration aspect and projection consumption contracts
- query-shaped live relevance and region-scoped narrowing artifacts
- installed domain invariant and graph-obligation definitions
- installed operation semantic closure and completed execution trace
- proposed projection dependency contract, dependency role, compiled impact
  closure, and impact decision
- Foundational aspect contracts, masks, field paths/locators, canonical basis,
  and native value identity as the shared dependency payload vocabulary
- `worth_proof::NonEmpty`, `CanonicalVec`, `UniqueVec`, structural proofs, and
  checked outcomes for admitted dependency sets and closure facts actually
  established by Query

**Warnings**
- A flat list of changed fields does not explain whether identity, membership,
  ordering, value, grouping, or window meaning changed.
- Output absence does not explain whether a condition suppressed evaluation,
  deferred it, or computation reverted clean. Impact consumes the retained
  Signal outcome and cannot infer it from rows or result digests.
- Consumers may widen their own consequences, but they may not reinterpret the
  Query impact class or declare a narrower dependency closure than Query proved.
- Domain semantic prerequisites may enter through installed definitions and
  invariants; domain crates may not install executable invalidation callbacks
  or consumer-specific recompute policy.
- Replay scope, reversal scope, lineage scope, and invalidation scope must not
  be maintained as parallel graphs. They are derived roles and closures over
  the same installed operation and execution trace.
- Cycles, ambiguity, unsupported dependency roles, and incomplete lower-runtime
  locality must deny or conservatively escalate explicitly, never silently
  produce an exact local impact.
- Sorting, deduplication, or hashing a dependency vector does not establish
  canonicality, uniqueness, disjointness, or closure. Query admission mints the
  matching `worth-proof` fact only after validating the real graph.

**Test requirements**
- Adversarial closure convergence test: direct recomparison, live maintenance,
  and replay compile identical dependency roles, transitive closure, impact
  class, and counters for the same installed capability and change.
- Adversarial trace-derivation test: the compiled impact closure contains every
  declared stage read, conditional trigger, touch, effect, invariant, output,
  comparator/equivalence, and lineage dependency
  and cannot be narrowed by omitting a stage receipt or substituting a
  consumer-authored replay or invalidation scope.
- Adversarial role matrix test: identity, membership, ordering, projected
  value, grouping, window-boundary, support/lifecycle, invariant, and advisory
  changes produce exactly their specified impact and never silently promote
  advisory context into operational authority.
- Adversarial foreign and ambiguity test: a closure from another runtime,
  installation, generation, view definition, basis, or plan cannot classify
  impact for the target capability; cyclic, conflicting, or incomplete
  declarations fail before live or consumer work.
- Adversarial boundedness test: compiling or applying an impact for `d`
  declared dependency edges and `a` affected edges performs work proportional
  to `d` at binding and `a` at change time, independent of unrelated views,
  aspects, consumers, graph nodes, and diagnostic projections.
- Adversarial conditional impact test: ineligible and deferred outcomes produce
  no computed-output impact; reverted-clean outcomes record evaluation cost but
  no semantic output change; a changed eligible output enters the ordinary
  narrow impact closure exactly once.
- Adversarial proven-closure test: empty required closures, duplicate roles,
  noncanonical order, false disjointness, raw vectors, and consumer-authored
  proof markers cannot satisfy impact admission; equivalent legal graphs yield
  identical proof-bearing dependency sets and impact outcomes.

**Engineering decisions**
- Installed definitions and plans are authoritative for Query dependency
  meaning; Query-minted traces carry the realized dependency evidence, and
  compiled closure indexes are derived and rebuildable.
- Query emits the narrowest proven impact class: unaffected or suppressed,
  value patch, membership splice, reorder or regroup, window shift, reexecute,
  explicit rebind, replacement, retirement, or unsupported escalation.
- Foundational owns native aspect shape and contract meaning; Query owns how an
  admitted query consumes that meaning and how changes affect its result.
- `worth-proof` owns reusable structural facts and checked progression; Query
  owns dependency semantics, graph traversal, closure computation, and impact
  classification.
- Impact closure is computed before invalidation fan-out so consumers receive a
  decision, not ingredients from which they must rediscover one.
- Replay, reversal inspection, lineage, invalidation, patching, and reuse
  consume the same dependency/effect closure instead of maintaining separate
  authority maps.

**Open questions**
- Exact eager versus borrowed closure representation is an implementation
  choice governed by the same semantic and exact-counter contract.

### Phase 19: Equivalent Capability Coalescing And Managed Consumer Leases

This phase turns canonical equivalence into framework-owned shared work. Query
may coalesce bound projections only when a sealed equivalence decision proves
the exact runtime, installed package and generation, operating context, basis,
policy and tenant posture, canonical view definition, support contract, native
projection contract, conditional-node and comparator contract, installed
Signal graph/node generation, trigger or wake basis, dependency closure, and
lifecycle state required by the
shared execution family. Each consumer receives an independently disposable
lease over the shared resource rather than a clone of execution authority.

Sharing is an admitted optimization and lifecycle contract, never a change in
canonical Query meaning. A capability that is not proven share-compatible runs
independently or returns a typed unsupported-sharing decision; matching labels,
digests, definitions, or output rows cannot authorize coalescing.

**Relevant subsystems**
- canonical declaration and bound-capability equivalence
- snapshot execution, live subscription, continuation, and managed lifecycle
- shared observer registry, delivery fan-out, cancellation, and disposal
- exact sharing, suppression, fan-out, and resource counters

**Relevant APIs**
- compatibility witnesses and managed live resources
- subscription-family sharing and lifecycle contracts
- bound execution and settlement capabilities
- proposed share-admission proof, shared execution owner, consumer observation
  lease, lease disposal receipt, and sharing counter snapshot
- Foundational canonical comparison and structured mismatch as portable semantic
  input, plus `worth-proof` pair-bound checked admission and move-only lifecycle
  progression behind Query-owned sharing/lease types

**Warnings**
- Cloning a packet, handle, subscription identifier, or shared pointer does not
  establish semantic equivalence or managed lifecycle.
- Automatic sharing across basis, policy, tenant, runtime, installation,
  generation, contract, condition, comparator, trigger/wake basis, installed
  Signal graph/node, dependency closure, result shape, or lifecycle
  boundaries is forbidden even if current results happen to match.
- Foundational equivalence is necessary where portable meaning is compared but
  never sufficient for operational sharing. Query still proves every runtime-
  affine dimension and mints the concrete pair-bound sharing witness.
- Lease disposal must not cancel work still observed by another admitted lease,
  and a leaked consumer must not make resource ownership unknowable.
- Reference counting or global locking on every delivery can hide coordination
  cost; the sharing design must expose fan-out and lifecycle work structurally.

**Test requirements**
- Adversarial equivalence convergence test: direct and independently authored
  share-eligible equivalent capabilities admit one shared execution meaning
  and identical lease semantics regardless of declaration order or sharing-
  index rebuild.
- Adversarial non-equivalence test: foreign runtime, stale generation, changed
  basis, policy, tenant, support, projection, condition, comparator, trigger,
  temporal wake, Signal graph/node, dependency closure, window, or
  lifecycle posture refuses sharing before execution or resource mutation.
- Adversarial lifecycle test: arbitrary lease create/drop/cancel/dispose order
  leaves exactly one resource while any lease remains, disposes it exactly once
  after the last lease, and makes use-after-disposal or cross-owner release
  uncallable or typed-denied.
- Adversarial fan-out and contention test: one shared change delivered to `l`
  leases performs exactly one underlying maintenance pass plus `l` admitted
  deliveries, exposes coordination counters, and does not scan unrelated
  capabilities, resources, or consumers.
- Adversarial lower-substrate test: Foundational equivalent comparison with
  foreign runtime authority, or a raw/copied `worth-proof` witness with matching
  payloads, cannot admit sharing; only the Query-owned checked transition mints
  the shared owner and independent move-only leases.

**Engineering decisions**
- Query alone decides whether capabilities may share and retains the shared
  execution owner; consumers own only their lease and downstream consequences.
- Foundational and `worth-proof` supply canonical comparison and progression
  law respectively; neither owns the shared resource registry, equivalence
  dimensions, lease lifecycle, or fan-out mechanics.
- Sharing indexes are rebuildable projections over retained active capability
  and resource authority and cannot mint or prolong a resource by themselves.
- Base projection work may share independently of consumer presentation.
  Window-specific maintenance shares only when the later bound window contract
  proves equivalent window, cursor, ordering, and continuation meaning.
- Snapshot observation, live maintenance, and continuation sharing retain
  distinct failure and cost contracts even when they reuse lifecycle machinery.

**Open questions**
- Exact registry and allocation strategy must be selected from measured
  observer topology, but it must preserve the phase, authority, and counter
  contracts above.

### Phase 20: Capability-Bound Consumer Invalidation Delta

This phase makes Query emit one typed invalidation delta from the managed bound
capability or shared execution owner whenever Query meaning relevant to a
consumer changes. The delta carries the Phase 18 impact decision and names the
exact capability generation, shared-resource generation where applicable,
target lease, affected declared access keys, result-state or lifecycle cause,
retained conditional decision cause where applicable, locality/region posture
where admitted, continuation posture, and the Query-
owned compatibility evidence required for preservation, patching, reexecution,
rebind, replacement, or retirement.

Consumers retain authority over their consequences. Worth UI maps the delta to
UI graph nodes, allocation work, preservation policy, and mounted receipts; it
does not rediscover which Query subsystem changed or manufacture a local list
of Query invalidation surfaces.

**Relevant subsystems**
- query-shaped live invalidation and region-scoped narrowing
- bound projection lifecycle and compatibility
- compiled dependency impact and managed shared-execution fan-out
- declared native access keys and projection-consumption facts
- downstream invalidation subscription and consumer indexes

**Relevant APIs**
- query-shaped live delivery and invalidation metadata
- projection impact decisions and consumer observation leases
- proposed `WorthQueryConsumerInvalidationDelta` or equivalent
- capability-bound invalidation subscription/observation surface
- Foundational aspect/field locators, projection masks, boundary-artifact role,
  provenance, and freshness vocabulary inside the stronger Query delta and its
  derived cross-crate representation

**Warnings**
- A consumer enum that mirrors live, signal, async, recovery, inspection, and
  projection subsystems is a second interpretation of Query change meaning.
- A derived lookup key may locate a candidate, but the delta must retain the
  bound authority that proves the candidate is current.
- Consumers may widen their own consequence scope explicitly; Query may not
  silently broaden a narrow declared change to a whole-view invalidation.
- A consumer may not downgrade a reexecute, rebind, replace, retire, or
  unsupported impact into a local value patch.
- A condition-based deferral or suppression is not a computed invalidation.
  Reverted-clean evaluation may update inspection/cost posture without emitting
  a semantic result delta or consumer patch.
- Invalidation lookup must not scan all bound projections or all consumer
  bindings to rediscover an exact capability match.
- A Foundational locator/mask identifies shared semantic scope but is not a
  Query invalidation delta. A Foundational derived-projection or support-only
  artifact cannot preserve, patch, reexecute, rebind, replace, or retire a
  consumer binding.

**Test requirements**
- Adversarial replay-convergence test: the same canonical Query evolution emits
  byte-equivalent semantic impact, invalidation deltas, and compatibility
  outcomes across one-shot recomparison, live maintenance, and replayed
  delivery.
- Adversarial foreign/stale test: a delta from another runtime, installation,
  capability generation, declaration, or lifecycle cannot invalidate,
  preserve, rebind, replace, or retire the target consumer binding.
- Adversarial boundedness test: delivering changes to `k` affected bound
  projections and `l` admitted leases performs exactly `k` indexed capability
  lookups plus `l` targeted deliveries and does not grow with unrelated
  projections, consumers, graph nodes, rows, or diagnostics.
- Adversarial authority-separation test: two consumers may derive different
  downstream consequences from the same delta without changing its Query
  meaning, compatibility evidence, identity, or counters.
- Adversarial conditional invalidation test: condition-relevant input changes
  target only admitted affected capabilities; the Signal decision determines
  whether compute and semantic delivery occur, and no consumer hook can widen
  a no-change outcome into a Query-authored patch.
- Adversarial Foundational projection test: equivalent Query deltas materialize
  identical Foundational locators, masks, provenance, and derived boundary role,
  while copied locators, masks, support artifacts, and stale attachments cannot
  drive any Query lifecycle or patch consequence.

**Engineering decisions**
- Query owns change meaning through a capability-bound delta; consumers own the
  semantic consequences inside their domains.
- Foundational supplies shared scope and boundary-description vocabulary; Query
  retains the stronger operational delta, authority, lifecycle generation, and
  consequence classification.
- The compiled impact decision is preserved, not re-derived, across shared-
  resource fan-out and per-lease delivery.
- Delta identity is operational only while paired with retained bound
  authority and lifecycle generation.
- Subscription and continuation are framework-managed extensions of the bound
  capability, not consumer-owned hook tables.

**Open questions**
- The exact balance between eager affected-key materialization and a bounded
  borrowed iterator must be decided from allocation and delivery cost, with
  identical semantic and counter contracts.

### Phase 21: Operational Identity Opacity

This phase separates operational identity authority from reporting and
diagnostic representation. Operational handles and witnesses expose semantic
comparison only through Query-owned methods. Digests, labels, formatted traces,
portable operation names, dependency fingerprints, equivalence tokens, lease
labels, condition/comparator fingerprints, Signal decision summaries, and
diagnostic fingerprints remain observable but cannot key, compare,
install, bind, share, admit, replace, rebind, reuse, release, or resume
operational state.

The migration is executed through the inventoried vertical slices in
[`milestone-9.14-phase-21-identity-inventory.md`](./milestone-9.14-phase-21-identity-inventory.md).
Each slice records its source owner, current authority leaks, intended
cross-runtime handoff, and hostile closure evidence before production migration
begins.

**Relevant subsystems**
- canonical Query identity and evidence identity
- facade reporting, diagnostics, and inspection projections
- installed-operation, dependency, sharing, replacement, and rebind indexes

**Relevant APIs**
- typed Query authority and basis identities
- evidence/reporting digest projections
- installed-operation lookup, compatibility, sharing, and lease entrypoints
- Foundational authority, admitted, boundary-bridged, readmitted, projection,
  digest-evidence, and external-token identity categories for shared boundary
  representation

**Warnings**
- Hiding a raw integer behind a newtype is insufficient if consumers can copy
  it into operational constructors or compare it as authority.
- Reporting digests and equivalence explanations may be stable for diagnostics
  without becoming installed-operation, sharing, or compatibility contracts.
- Query operational identities stay stronger owner types. Foundational identity
  categories are used when identity crosses or is projected at a shared
  boundary, and their explicit readmission does not bypass Query admission.
- Internal indexes must retain or revalidate the authority their derived keys
  locate; a key hit alone opens no door.
- Do not remove useful inspection merely to prevent misuse; separate power from
  observability structurally.

**Test requirements**
- Adversarial representation test: copied digest bytes, debug output, labels,
  portable operation names, dependency/equivalence tokens, lease labels,
  condition/comparator fingerprints, Signal outcome summaries, serialized
  diagnostics, and colliding test fingerprints cannot satisfy any
  operational constructor, lookup result, sharing admission, lease release, or
  compatibility requirement.
- Adversarial observability parity test: reporting and inspection projections
  remain deterministically derivable from retained authority and may be
  destroyed and rebuilt without changing operational outcomes.
- Adversarial API test: external consumers cannot extract raw operational key
  material or call equality/ordering traits that substitute for the named
  Query compatibility decisions.
- Adversarial identity-boundary test: authoritative Query identity projects,
  bridges, and readmits through the matching Foundational categories without
  collapsing into one newtype; projection identity, digest evidence, external
  tokens, and merely bridged identity cannot satisfy operational Query use.

**Engineering decisions**
- Operational identity types remain opaque and non-promotable.
- Reporting identity types are explicitly named as projections and have no
  conversion into operational authority.
- Shared identity exports reuse Foundational identity-boundary vocabulary;
  Query does not create a parallel projection/bridging/readmission ontology.
- Operational indexes return candidates that must remain paired with retained
  authority and a Query-owned validation step.

**Completion evidence**
- The seven inventoried vertical slices are closed in
  [`milestone-9.14-phase-21-identity-inventory.md`](./milestone-9.14-phase-21-identity-inventory.md),
  including their source owner, boundary weakening/readmission path, and
  hostile representation attacks.
- Query current read identity is minted from admitted retained Bridge truth.
  Query current mutation commit, snapshot, and entity identity is minted only
  when one Bridge-owned causality bundle matches the receipt's exact commit,
  snapshot, collection, Relational record, mutation kind, and canonical
  Foundational patch/touch meaning. Bridge denies subject/effect patch drift
  before authority execution. Equal raw projections and mismatched mutation
  bundles fail Query admission, including the empty-runtime snapshot.
- Query operation progression, aftermath, compatibility, shared lease, and
  invalidation artifacts retain private owner identities alongside generic
  Proof progression where progression is useful.
- Replay and lineage use domain-owned semantic comparison. Reporting identity
  cannot satisfy generic lineage equality or become ordinary replay authority.
- Compiler certification denies raw identity-part extraction, raw
  Relational-to-Query promotion, owner-marker facade imports, forged sharing
  and invalidation construction, and replay-to-ordinary promotion.
- Equality, ordering, and hashing remain only on identities that must
  participate in legitimate representation-level collections. Operational
  decisions require named owner methods or proof-bearing artifacts.

**Open questions**
- None.

### Phase 22: Consumption Cost Evidence And Exact Counters

This phase makes the bound capability's costs visible at every downstream
boundary. Installed operation lookup, binding, support-contract minting,
execution handoff, compatibility, native access, dependency compilation and
impact resolution, authoritative truth-delta interpretation, field-target
lowering, aspect-correspondence lookup, mapping widening, slot allocation and
fan-out, conditional lowering admission, invalidation/dependency
narrowing, condition resolution, comparator resolution, domain compute,
semantic-change classification, sharing admission and lease fan-out,
lifecycle transition,
invalidation delivery, collection-window resolution, patch delivery, and
denial outcomes carry structural counters sufficient to prove their declared
bounds without timing-based guesses or consumer instrumentation.

**Relevant subsystems**
- projection fact extraction and native refinement
- installed-domain operation lookup and compatibility admission
- dependency closure and impact decision
- shared execution, lease lifecycle, and delivery fan-out
- managed capability transitions and downstream settlement receipts
- consumer invalidation and collection window/patch delivery
- Relational publication precision and installed bridge-to-Signal aspect
  correspondence
- Query complexity contract registry and certification counters

**Relevant APIs**
- `ProjectionFactExtractionCounters`
- installed-domain drift and execution counters
- proposed downstream capability, dependency-impact, sharing, and native-access
  counter snapshots
- Foundational performance contract names, counter specifications, counter rows,
  policy-admission receipts, canonical comparison bundles, counter-backed
  execution receipts, and explicitly materialized reports

**Warnings**
- End-to-end elapsed time cannot prove bounded semantic breadth.
- Counters added only to diagnostics are optional observability, not a
  performance contract visible to the consumer.
- Foundational performance claims and policy-admission receipts do not prove
  execution. Query produces real boundary-local rows first; only then may it
  materialize a Foundational counter-backed performance receipt.
- Rich Foundational performance reports are deliberate support widening and
  remain off the ordinary execution/consumption hot path.
- Do not charge unrelated source execution work to the downstream consumption
  boundary; each boundary accounts for the work it owns.
- Exact-zero denial claims require counters for planning, lower-runtime contact,
  fact access, refinement, sharing/resource mutation, lifecycle mutation,
  invalidation, window/patch application, and authority reopen.
- Condition checks, comparator checks, temporal/on-demand deferrals, domain
  compute, reverted-clean outcomes, semantic changes, and deliveries are
  separate counters. Collapsing them into `evaluated_nodes` makes both cost and
  behavior unverifiable.
- Exact and widened correspondence, ambiguity/capacity denial, Signal node fan-
  out, and actual slots touched are separate counters; a generic routing count
  cannot prove semantic breadth.

**Test requirements**
- Adversarial slope test: resolving an installed operation, binding it,
  compiling `d` dependencies, and accessing `k` declared facts performs one
  indexed operation lookup plus work proportional to `d + k` and remains
  unchanged as unrelated rows, fields, views, domains, installations,
  diagnostic projections, and consumers grow.
- Adversarial denial-budget test: foreign, stale, detached, wrong-contract, and
  wrong-lifecycle inputs report exact zero downstream planning, execution,
  access, refinement, and lifecycle work beyond the minimum indexed authority
  check.
- Adversarial accounting parity test: ordinary facade and internal oracle paths
  produce identical counter snapshots for the same capability journey.
- Adversarial incremental breadth test: `a` affected dependency edges, `k`
  affected capabilities, `l` admitted leases, window width `w`, and patch width
  `p` report exact independent counters and remain insensitive to unrelated
  registry, collection, consumer, and graph size.
- Adversarial conditional accounting test: each outcome family proves exact
  invalidation-narrowing, condition-check, comparator-check, compute,
  semantic-change, and delivery counts; unrelated graph nodes and consumers do
  not affect them.
- Adversarial aspect-breadth test: lowering `t` authoritative truth targets
  through `m` admitted correspondence matches touches exactly the declared
  Signal fan-out and remains insensitive to unrelated schema aspects, mappings,
  nodes, slots, consumers, and reporting scopes.
- Adversarial performance-strength test: descriptive claims, policy-admission
  receipts, canonical bundles, counter-backed execution receipts, materialized
  reports, and certified/readmitted bundles retain their distinct Foundational
  strength; no weaker artifact can satisfy a stronger Query cost claim.
- Adversarial counter-export parity test: Query's exact operational snapshot
  materializes the expected Foundational counter rows with no missing,
  duplicate, unexpected, or mismatched rows, while report materialization does
  not alter the operational result or counters.

**Engineering decisions**
- Every critical result or denial embeds its boundary-local counter snapshot.
- Native access is `O(k)` for `k` requested access keys and `O(1)` per key after
  declaration admission; compatibility is bounded by the named retained proof
  dimensions rather than global registry size; impact resolution is bounded by
  affected dependency edges; shared maintenance plus delivery is bounded by
  semantic maintenance breadth plus admitted lease fan-out; invalidation and
  patch work is bounded by affected capability and semantic patch width.
- Rich inspection may derive summaries from counters but may not alter or
  become the source of operational accounting.
- Query owns measurement and hot-path counter capture; Foundational owns shared
  performance claim/receipt/report vocabulary. The export is derived from the
  Query snapshot and cannot be readmitted as operational execution authority.

**Open questions**
- None.

### Phase 23: Bound Collection Identity And Window Declaration

This phase specializes the bound projection for collection-shaped results. A
bound collection capability retains canonical result shape, entity and view-
local row identity, ordering authority, membership semantics, cursor basis,
and admitted window policy. Consumers request semantic windows through that
capability resolved from its installed view definition rather than assembling
ranges from offset pagination, result digests, copied row keys, or host-local
collection caches.

The consumer may declare presentation concerns such as viewport overscan or
mounting budget. Query owns which ordered, cursor-bound result window those
concerns select from canonical Query meaning.

Entity/view-local identity projections, canonical basis/comparison, field paths,
locators, masks, and native row values reuse Foundational vocabulary. Window
admission uses checked `worth-proof` progression and an exact assumption basis
behind the Query-owned, capability-bound request; neither a canonical cursor
projection nor a raw proof artifact is window authority.

**Relevant subsystems**
- collection result shape, ordering, membership, and cursor authority
- installed collection-view definition, bound projection, and declaration-
  indexed access
- view-local identity and materialized projection facts
- downstream virtualization/window request boundary

**Relevant APIs**
- collection query results and opaque cursor surfaces
- entity identity and view-local identity projection facts
- proposed bound collection capability and admitted window declaration
- Foundational identity/basis/path/locator/mask/value vocabulary and
  `worth-proof` checked admission behind the bound Query window types

**Warnings**
- Array position, UI node order, display text, and host widget identity cannot
  become row identity.
- A numeric offset is not a stable cursor or window authority under live
  membership and ordering change.
- Consumer overscan policy may affect requested breadth but not Query ordering,
  membership, or continuation meaning.
- The collection capability must not require the consumer to scan all rows to
  recover stable identity or locate a declared window.
- Foundational identity equivalence and raw `worth-proof` witnesses cannot
  bypass ordering, membership, cursor-basis, capability-generation, or consumer-
  breadth admission.

**Test requirements**
- Adversarial identity convergence test: direct collection execution, bound
  collection execution from the installed view, and equivalent window requests
  produce identical canonical row identity, ordering, membership, and cursor
  meaning.
- Adversarial reorder test: insertion, removal, ordering-key change, grouping
  movement, and unrelated row change preserve stable entity/view-local identity
  without relying on prior array position.
- Adversarial construction test: offset ranges, copied cursor bytes, consumer-
  authored row keys, reporting digests, and foreign window declarations cannot
  construct an admitted window.
- Adversarial bounded request test: declaring and resolving a window of width
  `w` performs work proportional to the admitted access plan and `w`, with no
  consumer-side full collection scan.
- Adversarial substrate-boundary test: Foundational projections converge with
  Query's portable identity/basis meaning, but raw proof carriers, caller-
  selected markers, and canonically equal foreign cursors cannot mint or recover
  an admitted window.

**Engineering decisions**
- Collection identity and ordering remain Query-owned parts of the bound
  capability; consumers receive typed row handles and declared native access.
- Window admission binds cursor, ordering, basis, capability generation, and
  consumer breadth policy into one proof-bearing request and extends Phase 18's
  dependency roles with collection membership, ordering, grouping, and window-
  boundary impact.
- Unsupported grouping, cursor, or live-window neighbors deny explicitly rather
  than degrading into offset pagination or full materialization.
- Query owns collection membership, ordering, cursor, and window authority;
  Foundational and `worth-proof` supply shared meaning and checked progression
  without becoming a collection runtime.

**Open questions**
- Whether the ordinary DX exposes cursor-first, anchor-first, or viewport-first
  authoring remains open to transcript testing; all must lower to the same
  admitted window contract.

### Phase 24: Query-Shaped Window And Patch Delivery

This phase delivers collection evolution as capability-bound Query patches.
Insert, remove, move, update, reset-required, result-state, warning,
continuation, and window-shift posture remain attached to the exact bound
collection authority, cursor basis, compiled impact decision, and target lease
where sharing is admitted. The consumer receives a patch that can be applied to
its own graph or mounted representation without reinterpreting raw CDC, diffing
full collections, or hashing a local Query patch posture.

Worth UI translates admitted row and patch meaning into UI graph touches,
virtualized node preservation, measurement invalidation, and mounted receipts.
Query does not own UI mounting, overscan policy, or allocation.

The Query patch retains Foundational row identities, aspect/field paths, masks,
locators, native values, provenance, boundary evidence, and canonical basis
where those meanings cross crates. Those shared components never replace the
stronger capability-, cursor-, impact-, and lease-bound Query patch authority.

**Relevant subsystems**
- query-shaped live collection patches and suppression
- bound collection windows, result state, and continuation
- dependency impact, managed leases, consumer invalidation deltas, and native
  row access
- Worth UI virtualized data and graph-touch handoff

**Relevant APIs**
- live collection patch artifacts and delivery receipts
- proposed bound window delivery and consumer patch capability
- exact patch-width, suppression, and continuation counters
- Foundational patch component, identity, value, provenance, evidence, and basis
  vocabulary retained inside the stronger Query delivery artifact

**Warnings**
- A bundle of support, live, async, inspection, consumption, and recovery
  digests is not a collection patch contract.
- Raw CDC or source-row events cannot be exposed as consumer patch meaning.
- Full reset is an explicit typed posture with a reason and cost contract, not
  a silent fallback when incremental handling is inconvenient.
- Patch application must validate capability generation and cursor basis before
  touching consumer state, and shared delivery must validate the exact target
  lease without repeating underlying maintenance.
- Conditional deferral, suppression, or reverted-clean evaluation cannot emit
  an invented empty update or reset patch. Only a retained semantic impact may
  produce collection delivery.
- A generic Foundational patch, matching boundary evidence, or canonical digest
  cannot be applied through the Query delivery path without the exact bound
  capability, cursor basis, impact decision, generation, and target lease.

**Test requirements**
- Adversarial fresh-execution parity test: applying every admitted patch
  sequence yields exactly the same ordered window, native values, result state,
  and continuation meaning as fresh execution at the resulting basis.
- Adversarial delivery-order test: duplicate, stale, reordered, foreign,
  wrong-window, and wrong-generation patches cannot corrupt the admitted
  consumer window and produce typed next actions.
- Adversarial bounded patch test: a semantic patch of width `p` performs work
  proportional to `p` plus declared window-boundary effects, independent of
  total collection, UI graph, and unrelated consumer size.
- Adversarial virtualization convergence test: Worth UI consumes bound windows
  and patches without local Query posture digests, row-identity reconstruction,
  full-collection diffing, or raw CDC interpretation while preserving its own
  graph and mounting authority.
- Adversarial shared-window test: equivalent admitted windows may share exactly
  one maintenance pass while distinct leases receive independently applicable
  patches; a different cursor, ordering, basis, or window contract cannot join
  that shared resource.
- Adversarial conditional patch test: deferred and suppressed nodes deliver no
  patch, reverted-clean nodes deliver no semantic patch while preserving cost
  evidence, and changed eligible output produces the same patch as fresh
  execution at the resulting basis.
- Adversarial Foundational-projection test: every shared patch component round-
  trips with identical Foundational meaning, while a reconstructed generic patch
  or same-field evidence bundle cannot authorize Query or consumer mutation.

**Engineering decisions**
- Query patches carry stable row identity, operation meaning, compiled impact,
  affected declared facts, result-state posture, continuation, authority,
  target lease where applicable, and exact counters.
- The consumer applies patches through a capability-validated handoff and owns
  only domain-local consequences.
- Suppression and reset admission are Query decisions; mounting and preservation
  policy remain consumer decisions.
- Foundational owns shared patch-component meaning; Query owns semantic patch
  assembly and delivery authority; the consumer owns application consequences.

**Open questions**
- None.

### Phase 25: Public Facade, DX, And Reference-Consumer Cutover

This phase makes one installed operating-world root, its typed operation-family
views, and the bound capability the ordinary discoverable path. The facade lets
a host install domains and any genuinely required graph adapters once, then
lets consumers borrow construct, boolean, transform, route, recovery, and
future family views from that root. Through those views they resolve a domain-
native operation, author or resolve its typed conditional nodes, bind it
atomically across required domains, graphs, and installed Signal lowering,
advance its typed workflow, consume its declared publication, request ordinary
re-execution or reversal, consume typed lineage, execute or promote it, access
declared native values, ask named compatibility questions, inspect compiled
impact, acquire or dispose managed observation leases, consume support
contracts and invalidation deltas, declare collection windows, apply query-
shaped patches, and dispose managed resources without importing internal phase
topology or manually carrying authority ingredients. Trace-driven replay is
absent from this ordinary facade and remains certification-only.

Worth UI is the primary hostile reference consumer because its allocation,
rebind, and virtualization boundaries exercise installation, projection,
native refinement, support, compatibility, lifecycle, invalidation, collection
identity, and patch delivery together. The cutover removes its remaining need
to defend against recombined Query authority or reconstructed Query posture
locally.

**Relevant subsystems**
- Query domain/read/live/foundation facade modules
- operating-world installation and graph-participation adapter facade
- portable conditional-node authoring and runtime-bridge/Signal support facade
- consumer kit and external compile transcripts
- `worth-ui-query-binding` and other serious downstream reference consumers
- Query discovery documentation

**Relevant APIs**
- installed-domain capability facade
- installed view/workflow resolution, bound projection facade, and declaration-
  derived access grammar
- workflow-run, ordinary re-execution, reversal/compensation, lineage, and
  persistent-naming admission grammar
- separate cert-audience replay facade
- consumer support/invalidation contract, collection window, patch delivery,
  named compatibility, dependency impact, sharing lease, and lifecycle methods
- conditional-node declaration, typed condition/comparator/trigger grammar,
  and derived inspection of retained Signal decision evidence

**Warnings**
- A facade that reexports every ingredient and internal phase type has not
  simplified or sealed the boundary.
- The facade must not reexport raw Signal node configuration, condition enums,
  custom strings, resolver traits, decision constructors, or bridge requests.
- The ordinary facade also must not reexport raw Foundational artifacts or
  generic `worth-proof` recipes, artifacts, bases, proofs, witnesses, markers, or
  collection carriers as alternate inputs to governed transitions. It exposes
  only owner-specific contracts and safe descriptive projections.
- One entry root must not collapse into an untyped operation bag. Typed family
  views remain the discoverable grammar and share only root authority.
- Operation-family facades and graph adapters cannot expose public root,
  runtime, provider, or independent execution constructors.
- Do not preserve detached constructors for tests; hostile fixtures belong in
  certification-only support.
- Examples teach only the positive current path, not historical methods framed
  as forbidden alternatives.
- DX measurements must include imports, authority decisions, lifecycle cleanup,
  and denial routing—not only line count.

**Test requirements**
- Adversarial DX transcript: an external crate completes install, declaration,
  operating-world construction, graph participation where genuinely required,
  typed family borrowing, installed operation resolution, binding, snapshot
  execution, native access, live promotion, compatibility, shared observation,
  explicit rebind, workflow advancement, publication consumption, ordinary re-
  execution, aftermath handling, lineage-aware continuity, invalidation,
  collection window delivery, patch application, inspection, lease release,
  and disposal using ordinary facade modules with no local schema
  reconstruction, graph bridge, stage ledger, replay or undo catalog, lineage
  string, dependency graph, manual digest, raw path, hook enum, deep import, or
  detached receipt plumbing.
- Adversarial conditional DX transcript: the same external crate authors each
  supported condition family, registers volatile lowerers at runtime assembly,
  executes a real Signal-backed conditional node, and observes its Query result
  without raw Signal configuration, bridge requests, local trigger tests,
  decision restamping, or deep imports.
- Adversarial substrate-facade transcript: ordinary callers can use the complete
  owner-specific journey without importing `worth-proof` internals or manually
  assembling Foundational boundary artifacts; attempts to substitute either for
  a Query/Bridge/Relational/Signal capability fail at the public boundary.
- Adversarial single-root transcript: construct, boolean, transform, and a
  representative cross-domain operation all borrow from one installed root;
  none can create or accept an alternate root or raw adapter collection.
- Adversarial reference convergence test: Worth UI and at least one non-UI
  consumer produce the same canonical results, authority decisions, denials,
  and counters through the bound capability as Query's internal oracle.
- Adversarial deletion test: consumer-local authority bundles, basis
  comparators, stable domain-query builders, schema/selector/result-shape
  copies, dependency graphs, recompute policies, sharing registries, support-
  hook mirrors, digest folds, fact scans, invalidation surface enums,
  collection patch posture, lifecycle joins, workflow stage ledgers, replay
  catalogs, undo-scope registries, persistent-naming strings, and compatibility
  mirrors can be deleted without preserving fragments elsewhere.

**Engineering decisions**
- Progressive disclosure begins with one installed operating-world root, one
  typed operation-family view, one installed operation, and one bound
  capability; advanced inspection does not expose construction power.
- Worth UI retains ownership of Query-to-allocation semantic refinement but no
  longer reconstructs Query installation, basis, compatibility, or lifecycle
  authority, support posture, dependency impact, sharing eligibility,
  invalidation meaning, row identity, or patch semantics.
- Compiler diagnostics and typed denials route callers to the nearest legal
  facade transition.

**Open questions**
- Final method names remain open to golden transcript review; the authority
  grammar and required journey are fixed.

### Phase 26: Mechanical Prohibitions And Hostile Certification

This phase makes regression structurally expensive. Compile-fail probes,
facade snapshots, boundary checks, source residue audits, sabotage fixtures,
and provider-independent semantic oracles prove that authority ingredients
cannot reappear as an alternate public path.

**Relevant subsystems**
- Query facade and module visibility
- compile-pass/compile-fail consumer harnesses
- boundary checker and source/API residue certification
- Milestone 13 provider-independent certification oracle

**Relevant APIs**
- public facade snapshots and semver surface checks
- certification-only hostile fixture constructors
- downstream capability parity bundle
- installed-operation, dependency-impact, sharing, and lease parity bundles
- semantic truth dependency, Relational publication precision, installed
  aspect-correspondence, conditional lowering, Signal decision, and Query re-
  entry parity bundle
- reusable installed-operation/workflow certification kit with generic
  compile-fail proofs and domain-supplied semantic scenario fixtures

**Warnings**
- Source scans alone cannot prove runtime-affine authority continuity.
- Runtime rejection alone is weaker than making detached assembly uncallable.
- Certification support must not leak hostile constructors into production
  facade modules.
- Do not freeze filenames or incidental topology; enforce authority direction,
  forbidden construction, and responsibility boundaries.
- Do not duplicate hundreds of consumer-local compile harnesses. Query owns the
  generic authority-impossibility matrix; domain packages contribute narrow
  semantic fixtures and runtime scenarios for their installed operations.
- Runtime-generated compile-fail claims are dishonest. Compile-fail proof is a
  central static harness; the reusable consumer kit supplies runtime matrices,
  bounded fixtures, sabotage cases, and semantic parity oracles.
- Do not re-certify `worth-proof` or Foundational internally. 9.14 proves that
  its owner-specific wrappers preserve and cannot bypass those shipped laws;
  the lower crates retain their own proof-of-substrate certification.

**Test requirements**
- Adversarial sabotage matrix: foreign installation, stale generation,
  second operating-world root, hidden or forged graph adapter, independently
  constructed operation-family facade, locally reconstructed installed
  operation, copied semantic aspect key, drifted contract revision or field
  mask, forged truth-delta target, raw or copied Signal aspect/mask, forged
  aspect-correspondence witness, stable-name scope substitution, raw or copied
  Signal condition, custom-condition string, foreign or stale
  condition/comparator/trigger provider, forged bridge lowering,
  detached or restamped Signal decision, detached completion, copied
  consumed facts, wrong declaration key, forged dependency closure, reporting
  digest collision, forged support projection, false sharing-equivalence token,
  wrong compatibility witness, foreign lease or invalidation delta, copied
  cursor, stale collection patch, disposed lifecycle, cross-run stage receipt,
  forged replay or reversal scope, representation-derived lineage, and cross-
  provider artifacts all fail at the earliest boundary with exact counters.
- Adversarial replay-fence test: ordinary entry, derived, app, UI, resolver,
  solver, and pack consumers cannot import or obtain trace-driven replay;
  certification can replay through the dedicated audience facade and cannot
  leak that capability back into ordinary code.
- Adversarial graph-boundary test: direct graph-to-graph calls, raw provider
  access, shared-ID joins, digest-authorized correspondence, and application-
  owned bridges cannot substitute for installed graph participation and a
  bound cross-graph operation.
- Adversarial facade/prohibition test: public API snapshots and compile-fail
  probes report zero consumer-executable portable operation definitions,
  detached capability constructors, raw operational identity extractors,
  consumer-mintable access/window/dependency/share keys, support/invalidation
  constructors, patch constructors, alternate operating-world roots,
  operation-family runtime constructors, public adapter bags, cert replay on
  ordinary facades, raw Signal node/aspect/mask/configuration or bridge request
  exposure, consumer-mintable semantic-aspect correspondence,
  Query-owned eligibility constructors, or lifecycle entrypoints that skip
  bound predecessors.
- Adversarial residue test: production source and reference consumers contain
  zero stable domain-operation reconstruction, operational digest comparisons,
  consumer-side authority assembly, local dependency or recompute authority,
  consumer-owned sharing registries, broad fact/collection scans, local support
  or invalidation mirrors, local basis compatibility logic, patch-posture
  hashes, raw CDC interpretation, raw Signal-slot interpretation, bridge stable-
  name authorization, silent field-to-whole widening, duplicate Foundational
  aspect/canonical/identity/evidence/performance vocabulary, private
  proof/recipe/basis/freshness/outcome/witness/proven-collection substrates,
  raw shared carriers accepted as owner authority, or orphan lifecycle joins.
- Adversarial substrate-adoption test: representative operation, publication,
  correspondence, re-entry, compatibility, lifecycle, lineage, invalidation,
  and performance paths use the real Foundational and `worth-proof` public
  surfaces behind opaque owner wrappers; raw lower carriers cannot progress the
  public facade, and no tests merely prove the lower crates' own tests.
- Adversarial conditional residue test: production Query and consumers contain
  zero raw Signal condition construction, custom-string dispatch, local
  condition evaluation, comparator reimplementation, eligibility restamping,
  bridge-request assembly, output-presence inference, or condition-specific
  replay/invalidation/sharing mirrors.
- Adversarial oracle-export test: Milestone 13 can drive the complete bound
  capability journey against a second admitted provider without weakening or
  reinterpreting Query semantics.
- Adversarial certification-kit test: a representative downstream domain can
  register operation semantics and contribute small workflow, replay,
  conditional-node, semantic-aspect correspondence, reversal, lineage, impact,
  and counter scenarios without rebuilding generic
  authority compile-fail tests or a product-specific evidence ledger.

**Engineering decisions**
- The certification matrix combines compile-time impossibility, typed runtime
  denial, exact counter proof, and reference-consumer deletion evidence.
- Boundary rules name allowed production owners and reject dependency or
  reexport bypasses mechanically.
- Boundary rules enforce the single-root and replay-audience contracts across
  Query and downstream entry, derived, app/UI, solver, resolver, pack, and cert
  bands.
- The provider-independent parity bundle becomes a required Milestone 13 input,
  not a separate interpretation of Query meaning.
- Generic construction impossibility is certified once at Query's facade;
  downstream certification is proportional to domain semantics rather than the
  cross-product of authority ingredients.

**Open questions**
- None.

## Must Ship

- complete typed installed operation semantic closure with indexed runtime
  resolution and no consumer-local reconstruction of stable operation meaning
- portable Query-authored conditional-node contracts covering typed trigger
  inputs, evaluation conditions, comparator/equivalence, temporal/on-demand,
  maintenance, artifact, and output posture without raw Signal configuration
- portable Query semantic truth dependencies retaining exact Foundational
  contract, Relational binding/surface, field mask, locality, and relevant
  change meaning without runtime-local Signal slots
- aspect-precise Relational authoritative publication preserving field, whole-
  aspect, endpoint, structural, lifecycle, and opaque-change posture
- runtime-affine installed semantic-aspect correspondences binding exact Query
  dependencies and Relational truth deltas to actual Signal graph/node/aspect
  targets; successful witnesses retain exact or declared-widening posture while
  unsupported, ambiguity, capacity, stale, rebind, denial, and failure remain
  checked non-success outcomes that mint no witness
- systematic reuse of Foundational aspect/value/mask/locator/canonicalization/
  identity/evidence/performance vocabulary and `worth-proof` artifact/recipe/
  proof-set/basis/freshness/witness/outcome/proven-collection progression behind
  stronger owner-specific Query, Relational, Bridge, and Signal types
- exact runtime-bridge lowering into installed Signal node contracts with
  pair-bound lowering witnesses and separately registered volatile providers
- Signal-minted eligibility, deferral/suppression, computation, and reverted-
  clean decision evidence admitted back into Query without restamping
- conditional dependency and consequence scopes proven as typed subsets of the
  same bound operation graph-read, graph-touch-obligation, and effect authority;
  Bridge/Signal invalidation can narrow but never mint or widen graph capability
- one installed operating-world root as the sole ordinary authority-bearing
  entry point, with typed borrowed operation-family facades
- sealed installed graph-participation adapters for genuinely separate graph
  authorities, with explicit atomic or compensated commit posture
- one sealed runtime-affine bound projection capability
- atomic multi-domain and admitted multi-graph operation binding
- one Query-minted consumer support contract
- proof-bearing installed execution, derived publication, consumption, and
  settlement progression
- installed typed workflow DAGs, Query-minted run/stage traces, and legal
  predecessor and conditional-outcome progression
- ordinary retry/re-execution plus cert-audience-only trace replay with distinct
  attempt identity and typed convergence or divergence
- exact-inverse, compensation, rebuild/recovery, provisional-discard, and
  irreversibility contracts with no scope-only undo
- trace-bound typed lineage and identity-evolution evidence for persistent
  naming, promotion-on-reference, and consumer continuity
- declaration-indexed Foundational-native value access and typed denials
- Query-owned installation, basis, replacement, rebind, and reuse decisions
- framework-owned live, replacement, rebind, cancellation, and disposal states
- Query-compiled semantic dependency roles, impact closure, and typed impact
  decisions
- explicit equivalent-capability sharing admission, framework-owned shared
  execution resources, and independently disposable consumer leases
- capability-bound consumer invalidation deltas
- opaque operational identities separated from reporting projections
- embedded exact structural counters and named complexity contracts
- bound collection identity/window declarations and query-shaped incremental
  patch delivery
- contracted facade, positive discovery path, Worth UI reference adoption, and
  reusable operation/workflow certification kit with a hostile certification
  matrix

## Must Preserve

- Foundational ownership of native scalar, struct, absence, contract, and
  canonical value meaning
- Foundational ownership of shared aspect contracts/masks/locators, canonical
  basis/comparison, boundary identity/evidence, provenance/lineage/support, and
  performance claim/receipt/report vocabulary wherever 9.14 crosses those
  shared boundaries
- `worth-proof` ownership of reusable proof-bearing progression, checked outcome
  topology, assumption basis/freshness/readmission, concrete witness spending,
  structural facts, and honest fixed-shape collections
- stronger runtime ownership remains intact: shared Foundational or
  `worth-proof` carriers cannot replace Query authority, Relational canonical
  patches, Bridge installed correspondence, Signal decisions/slots, or their
  owner-specific receipts and lifecycle state
- Query ownership of declaration, installation, basis, execution, projection
  consumption, installed operation meaning, consumer support, compatibility,
  dependency impact, sharing admission, invalidation, collection identity/
  ordering/cursors, patch meaning, and managed query lifecycle
- Query ownership of portable conditional authoring and canonical operation
  inclusion, runtime-bridge ownership of exact crossing and lowering, and
  Signal ownership of condition/comparator resolution and evaluation decisions
- Query ownership of portable semantic truth dependency intent, Relational
  ownership of authoritative aspect-change interpretation, runtime-bridge
  ownership of installed truth-to-Signal correspondence, and Signal ownership
  of node-local aspect slots, versions, invalidation, and scheduling
- Query's existing graph-read access planning and Graph Touch Obligation
  Authority remain the sole graph capability minting boundaries; conditional
  declarations, correspondences, Signal masks/decisions, dependency impact, and
  consumer invalidation remain scoped consumers or derived evidence
- separation of stable Relational/Foundational semantic aspects from runtime-
  local Signal aspects; numeric slots, stable-name scopes, and diagnostic
  digests never become portable or operational semantic authority
- strict separation between Query node evaluation conditions and Query effect
  conditions
- Query ownership of atomic cross-domain binding, workflow progression and
  trace assembly, cert replay admission, reversal binding, lineage evidence
  transport, graph-participation binding, and typed identity-evolution outcomes
- one-root authority with operation-family facades retaining distinct domain,
  cost, failure, and lifecycle contracts
- domain ownership of stage algorithms, inverse and compensation semantics,
  persistent naming policy, correspondence policy, and product meaning
- Store ownership of durable journals, checkpoints, restart replay, and
  recovery persistence
- single logical graph as the default and multi-graph participation only for
  explicit independent authority boundaries
- lower-runtime ownership of truth, reactive scheduling, and physical provider
  semantics
- exact retention of installed Signal node, graph, condition, comparator,
  trigger/wake, snapshot, attempt, and decision provenance through every Query
  phase that consumes conditional meaning
- backend-independent canonical Query meaning and provider parity
- distinct cost, failure, correctness, and lifecycle boundaries for snapshot,
  live, replacement, rebind, reuse, inspection, and disposal
- distinct Query change meaning and consumer-local consequence authority
- canonical equivalence and shared execution remain distinct: equivalence may
  admit reuse, but derived sharing indexes and leases never become declaration
  or execution authority
- portable domain operation definitions remain distinct from installed
  runtime-affine operation capabilities
- replay remains distinct from reversal, execution identity remains distinct
  from semantic equivalence, and touch/effect scope remains distinct from
  inverse authority
- execution traces remain certification evidence while derived publications
  remain the ordinary inter-operation consumption path
- Query remains a generic semantic framework rather than becoming a geometry
  kernel, product workflow registry, or executable callback host
- consumer ownership of viewport, overscan, graph, mounting, allocation, and
  presentation policy
- portable declarations and reporting projections remain non-authoritative and
  readmittable rather than secretly runtime-affine

## Acceptance Evidence

- equivalent installed view and workflow definitions converge, conflicting
  definitions fail atomically, and local lookalikes cannot execute
- one installed operating-world root is required by every ordinary operation-
  family facade; no family or application can construct an alternate root
- same-graph domain composition requires no artificial federation, while
  foreign, stale, or forged graph adapters cannot satisfy operation authority
- multi-graph work is atomic only with shared commit authority and otherwise
  exposes declared compensation, partial failure, and recovery posture
- invalid workflow graphs fail package installation, and independently valid
  stage receipts or domain capabilities cannot be combined into legal
  progression or atomic operation authority
- ordinary Query authoring creates real aspect-filtered, threshold, temporal,
  on-demand, and typed domain-specific Signal-backed nodes without raw Signal
  internals or custom-string dispatch
- field-level, whole-aspect, relation-endpoint, structural, lifecycle, and
  opaque Relational changes retain their exact or explicitly widened posture
  through the canonical bridge truth-delta path
- installed semantic-aspect correspondence supports exact one-to-one, one-to-
  many, admitted many-to-one widening, and derived-only Signal relationships;
  ambiguity, unsupported precision, and slot pressure deny atomically without
  catch-all collapse or graph residue
- exact and declared-widening correspondence are the only successful installed
  witness postures; unsupported, ambiguous, capacity-exhausted, stale, rebind-
  required, denied, and failed candidates retain distinct `worth-proof`
  outcomes and cannot later recover a witness
- Query, Relational, Bridge, and Signal reuse Foundational shared semantics and
  `worth-proof` progression without exposing raw generic carriers, caller-
  selected authority markers, semantic mirrors, duplicate outcome/freshness
  lattices, or lower-strength artifacts accepted as owner authority
- real authoritative Relational changes reach only their correspondence-bound
  Signal node/aspect versions through the production adapter; bridge stable
  names and equal numeric slots cannot authorize delivery
- equivalent conditional declarations and lowerings converge; one-field drift
  in condition, comparator, trigger/wake basis, Signal graph/node, snapshot, or
  attempt denies at the earliest owner with exact zero forbidden later work
- Signal alone decides eligibility and semantic cleanliness; ineligible,
  suppressed, and deferred nodes invoke compute zero times, reverted-clean
  nodes retain compute cost but emit no semantic delivery, and changed eligible
  output enters Query publication and downstream consequences exactly once
- conditional decision provenance survives installed execution, workflow,
  replay, compatibility, impact, sharing, lifecycle, invalidation, cost, and
  patch paths without Query or consumers restamping it
- every Phase 7-11 artifact retains the exact predecessor bound graph
  participation, read plan, touch-obligation evidence, adapter, generation, and
  basis; foreign/missing authority and attempted scope widening deny before
  lower-runtime work, while skipped/deferred/reverted-clean outcomes mint exact
  zero mutation touches or effects
- cert-audience replay creates a distinct run/attempt identity while proving
  the installed operation's declared semantic equivalence; ordinary crates
  cannot import replay and meaningful divergence localizes to typed trace,
  result, publication, effect, or lineage evidence
- scope-only undo is impossible; exact inverse, compensation, rebuild/recovery,
  and irreversibility remain typed and separately testable
- persistent naming cannot be authorized from raw strings, identifiers,
  digests, coordinates, rendered values, or advisory correspondence
- promotion-on-reference is proportional to newly durable references and never
  eagerly promotes unreferenced artifact payload
- independent valid artifacts cannot be recombined into operational authority
- the ordinary facade carries one exact installation-to-settlement authority
  chain with no consumer restamping
- declared native access preserves complete Foundational parity and bounded
  indexed cost
- portable canonical comparison, identity projection/bridging, lineage/support
  attachments, and counter-backed performance exports retain Foundational
  strength distinctions while operational Query admissions remain stronger
- named compatibility decisions return pair-bound witnesses or typed denials
- live and rebind lifecycle states are framework-owned and leave no orphans
- compiled dependency roles and impact decisions converge across one-shot,
  live, and replay paths without consumer-local closure or recompute authority
- equivalent capabilities share only through Query admission; one underlying
  maintenance pass fans out through independently disposable typed leases with
  exact counters
- support requirements and invalidation meaning come from Query-minted typed
  artifacts rather than consumer hook enums or digest bundles
- collection windows and query-shaped patches preserve row identity, ordering,
  cursor, result-state, continuation, and fresh-execution parity
- conditional no-change outcomes never manufacture empty/reset patches, fresh
  lineage, invalidation, or computed publications
- reporting representations have zero operational effect
- invalid, stale, foreign, detached, and disposed inputs deny before expensive
  work with exact counter proof
- Worth UI deletes local Query authority, support, invalidation, row-identity,
  stable operation reconstruction, dependency/recompute policy, sharing
  registry, workflow stage ledger, replay and undo catalogs, lineage-string
  reconstruction, and patch-posture reconstruction while preserving its graph,
  virtualization, allocation, mounting, and presentation semantics
- a representative operation/workflow certification kit proves generic
  authority impossibility centrally and accepts domain-owned semantic fixtures
  without duplicating a product-specific compile-test matrix
- compile-fail, facade, residue, sabotage, parity, and provider-oracle suites all
  agree on the single capability path

## Sequencing Notes

This milestone follows 9.13 because it hardens the installed-domain and
Foundational-native product boundary that 9.13 establishes; it must not invent
parallel declaration or value semantics while 9.13 remains open. Installed
domain operation definitions come first because graph participation, single-
root authority, atomic binding, workflow
progression, replay, reversal, lineage, dependency impact, sharing equivalence,
invalidation, and patch delivery must all consume one canonical operation
meaning rather than reconcile locally authored lookalikes. Graph participation
closes before binding so the root cannot accept raw providers or hidden
application bridges. Phases 1-6 establish and retain the operation, graph,
support, execution, and workflow positions that conditional meaning must enter.
Phases 7-11 then close portable semantic dependency and conditional authoring,
aspect-precise Relational publication, installed Relational-to-Signal
correspondence, exact runtime-bridge conditional lowering, Signal-owned decision
authority, and Query re-entry before replay or any later consumer authority is
built. This is an explicit substrate repair:
later phases may not proceed using always-eligible placeholders or detached
Signal configuration. Every phase reuses Foundational shared boundary meaning
and `worth-proof` progression law where applicable before introducing an owner-
specific wrapper; the wrapper may strengthen ownership but may not recreate the
lower substrate. Workflow graph and trace, conditional re-entry, replay,
reversal, and lineage close before native access and downstream
lifecycle work because every later compatibility, impact, reuse, invalidation,
collection, and certification decision must consume the trace rather than
retrofit a parallel ledger.

This milestone consumes the graph-read planning of Milestone 9.10, graph-touch
authority of Milestone 9.9, deterministic parallel admission of Milestone 5.3,
authority-scoped effect execution of Milestone 9.3.3, and existing lineage and
identity-evolution contracts from Milestones 5.4 and 7. It binds and composes
those capabilities; it does not rebuild them. It also consumes Relational's
canonical authoritative patch and aspect-contract meaning, plus Signal's
existing node, aspect-slot, condition, comparator, temporal eligibility,
prepared evaluation, and semantic-cleanliness contracts through the runtime
bridge. It does not create a Query-owned truth-change interpreter, Signal-slot
allocator, or evaluator, and it does not expose Signal internals as Query
authoring.

It precedes Milestone 13 because Query's provider-independent certification
oracle must export the non-detachable capability journey, not the lower-level
ingredient surfaces that this milestone removes. It must include consumer
support, workflow trace, replay, reversal, lineage, dependency impact, sharing
and lease lifecycle, semantic-aspect correspondence, conditional
authoring/lowering/decision provenance,
invalidation, collection windows, and patch delivery so
providers do not reinterpret them.
Store integration consumes that same oracle and capability contract through
the Store roadmap. The milestone is not blocked on Store and makes no physical
persistence claim.
