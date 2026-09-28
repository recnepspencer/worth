# Milestone 3.9.1: Query 9.14 Consumer-Path Modernization

Status: Closed on 2026-07-21.

Exit-trigger amendment, 2026-07-23: Query 9.14 Phases 17, 19, 23, and 24 are
complete, and the bounded managed-live compatibility seam described by this
historical plan has been replaced by Query's ordinary lifecycle, lease,
invalidation, collection-window, and patch surfaces. Milestone 3.9.2 owns the
remaining declaration-indexed native-access, opaque operational-identity, and
UI-consequence cutover before Milestone 3.10. The phase descriptions below
record the boundary that 3.9.1 actually closed; they are not rewritten as
though the later Query surface already existed.

## Goal

Modernize the Query-facing portion of the closed Milestone 3.9 runtime so
Worth UI declares its stable Query operations once, enters through Query's one
installed operating-world gateway for each operation attempt, consumes one
Query-minted consumer contract per bound execution, retains Query-minted
settlement proof, and owns only
downstream UI binding, allocation, replacement, and presentation meaning.

## Why This Milestone Exists

Milestone 3.9 correctly closed execution-plan ownership and real application
lifecycle proof against the Query surface available at the time. Query
Milestone 9.14 Phases 1 through 14 now provide a stronger ordinary consumer
path. Leaving Worth UI on its older local operation, support, basis, settlement,
and equivalence models would turn compatibility code into a second Query
runtime before Milestones 3.10, 3.12, 3.13, and 6 build on it.

This is a modernization add-on, not a rewrite of Milestone 3.9 history. It
supersedes only the old Query-bound assumptions; 3.9's application authority,
regional plan storage, allocation locality, real filesystem ingress, atomic
replacement, and frame-cost truths remain closed.

## Governing Summaries

- `MENTALITY.md` protects foundation-first authority under adversarial
  ambiguity. The migration must solve the competing-authority problem, not
  merely rename it until a search is clean.
- `arch_laws.md` requires Query's typed progression and exact identity to carry
  forward. UI may derive from those proofs but may not reconstruct them.
- `composition_laws.md` requires each surviving responsibility to have one
  predictable home. A giant modernization adapter or helper bag is not a
  compliant cutover.
- `domain_structure_laws.md` requires Query authority, UI derivation,
  diagnostics, and compatibility to remain physically distinguishable.
- `perf_laws.md` forbids repeated semantic construction, broad rediscovery, and
  hidden migration cost on ordinary build or frame paths.
- `worth_ui_roadmap.md` requires Worth UI to consume stronger Query runtime
  lanes instead of maintaining UI-local support, admission, result, recovery,
  or explanation models.

## Adversarial Constraint

Two bindings can have the same UI declaration text, local digest, result shape,
and hook count while belonging to different Query operating worlds, installed
operations, required-support admission, bases, or progression receipts. They must never
compare equivalent, activate as a no-op, share settlement, or preserve a live
resource. Conversely, a broad search must not delete legitimate UI-owned
derivations merely because their names contain `Query`.

The hostile scale case is simultaneous fan-in and fan-out: one application
candidate joins Query authority with UI identity, host, allocation, and
replacement authority, while one admitted Query settlement may feed many UI
plan rows. Neither direction may create a parallel publication point for the
same truth, clone a move-only Query phase value per row, or let a compact index
become authority.
Application replacement and in-generation fact refresh must use their distinct
existing transactions rather than partially publishing through each other.

The milestone therefore has two simultaneous obligations:

1. exact Query authority must survive every ordinary bind, execute, publish,
   consume, settle, replacement, and diagnostic boundary; and
2. every mechanically discovered subsystem must receive a human semantic
   disposition rather than being judged by token absence.

## Product Decision Lock

- The manually adjudicated
  [Query modernization inventory](./milestone-3.9.1-query-modernization-inventory.csv)
  is the subsystem migration reference. The separately adjudicated
  [boundary and edge matrix](./milestone-3.9.1-boundary-edge-matrix.csv) is the
  authority-flow reference.
- Search discovers candidates. A human resolves authority and ownership.
- The allowed inventory classifications are `replace`, `retain`,
  `transitional`, `diagnostic-only`, and `unrelated`.
- A clean grep is not acceptance evidence. No checked-in test may claim the
  migration is correct merely because old names, files, or tokens disappeared.
- Compiler boundaries, real Query progression, real application lifecycle
  scenarios, exact identities, typed denials, and structural counters remain
  valid proof.
- Query-free Worth UI applications remain Query-free and ceremony-free.
- Query replay, aftermath, and lineage do not enter the ordinary UI binding
  lane. UI binding succession is not Query operation lineage.
- The existing managed-live lane is explicitly transitional until Query 9.14
  Phases 17, 19, 23, and 24 provide the required ordinary lifecycle, lease,
  window, and patch-delivery surfaces.
- This milestone modernizes the existing measurement and allocation consumer
  path. It does not pre-build Milestone 3.13's general scalar, collection,
  product, continuation, or result-state binding substrate.

## Boundary Contract

The ordinary ownership chain is:

```text
authored UI binding reference
-> installed operation reference retained by the application generation
-> attempt-scoped borrow of Query's installed operating world
-> one Query-bound operation for replacement admission or fact refresh
-> one mint-once Query consumer contract plus separate UI requirements
-> execute -> publish -> consume -> settle
-> one generation-owned retained settlement slot
-> one UI-derived measurement-fact batch
-> compact generation-scoped plan references
-> requested allocation-frame ingress
```

- `WorthQueryInstalledOperatingWorld` is an attempt-scoped borrowed
  control-plane entry. The one binding owner enters it during preparation or
  replacement admission and again when an admitted in-generation Query refresh
  needs a fresh move-only operation chain. The world is never stored in active
  application or frame state.
- A bound operation may mint its consumer contract once. The contract and its
  temporary `WorthQueryConsumerBoundary` wrapper are preparation/execution
  authority, not durable application identity. The contract moves into
  Query's `consume` transition and cannot also be retained or copied into a UI
  posture product.
- The durable upstream artifact is the resulting
  `WorthQuerySettledDomainProjection`. The binding owner retains it once per
  active or candidate settlement slot; derived indexes and plan rows carry
  opaque, application-generation-scoped references to that slot.
- Candidate and active settlement stores remain disjoint until the existing
  Milestone 3.9 application transaction publishes the whole candidate. A
  denial drops only candidate-owned bound operations, contracts, settlements,
  facts, and compatibility resources. It cannot partially mutate active state.
- Within one active application generation, a refreshed Query settlement and
  its derived UI fact batch replace one existing slot only through the closed
  framework-turn source transaction. Failure preserves the prior slot. Fact
  refresh does not become an application replacement, and application
  replacement does not become an incremental fact-publication side door.
- `worth-ui-query-binding` is the only Worth UI production crate permitted to
  import `worth-query`. Runtime and inspection crates consume binding-owned
  opaque artifacts; inspection receives observation, never execution or
  replacement authority. Public facades expose lifecycle-ordered UI
  capabilities, not raw operating-world constructors or proof minting.
- Cost narrows at every edge: binding work scales with affected bindings,
  settlement derivation occurs once per executed binding, plan fan-out scales
  with affected dependents, and steady frames touch only requested fact
  references. No downstream phase may rediscover operation support, world
  identity, or affected scope.

## Phase Plan

### Phase 1: Adjudicated Subsystem Migration Authority

Freeze the migration's semantic scope before production edits begin. The CSV
is a curated register of subsystems discovered through broad search, not a raw
match dump and not a generated source of runtime truth.

**Relevant subsystems**

- all Worth UI production and test crates that mention Query operations,
  support, basis, projection, settlement, replacement, live resources, or
  diagnostics
- crate manifests and the canonical road-1 dependency contract
- the 3.9 certification targets that already own real application lifecycle
  proof

**Relevant artifacts**

- `milestone-3.9.1-query-modernization-inventory.csv`
- `milestone-3.9.1-boundary-edge-matrix.csv`
- search seeds recorded per inventory row
- `tools/boundary-check/config/road1.toml`

**Warnings**

- One row represents one semantic subsystem, not one file or one token match.
- A search hit may be correct UI ownership, obsolete Query mirroring, a bounded
  compatibility seam, diagnostic-only representation, or unrelated vocabulary.
  Its spelling cannot decide which.
- Search is rerun during implementation to discover missed candidates. A new
  ambiguous subsystem adds or revises a row; it does not fail a permanent
  zero-match sentinel.
- `resolved` means the row's semantic disposition is decided. It does not mean
  its implementation is complete.

**Test requirements**

- As phase QA, parse the CSV and reject duplicate IDs, missing owners, missing
  classifications, empty manual-resolution reasons, or transitional rows
  without an exit trigger. This is document integrity verification, not a
  product test.
- Parse the edge matrix and reject duplicate edge IDs, missing producers or
  consumers, unstated cardinality or lifetime, missing failure owners, absent
  cost contracts, and any edge without an explicit forbidden shortcut.
- Rerun broad searches for `WorthQuery`, `query_`, `projection`, `support`,
  `basis`, `settlement`, `rebind`, `live`, `replay`, `aftermath`, and `lineage`.
  Manually sample every unmatched semantic cluster and either map it to an
  existing row or add a new adjudicated row.
- Deliberately include a legitimate UI-derived Query surface and an obsolete
  local Query mirror in the same search result set. The review must retain the
  former and replace the latter, proving that search count is not the decision
  rule.

**Engineering decisions**

- The CSV is versioned design evidence and implementation routing authority.
  It does not generate Rust, tests, or allowlists.
- The two CSVs describe different axes: subsystem disposition and authority
  flow. Neither is a generated migration sentinel, and neither substitutes for
  compiler or behavioral proof.
- Implementation completion is recorded row by row only after the target code
  and its real boundary proof land.
- Permanent enforcement targets structural facts: dependency direction,
  sealed construction, exact authority flow, typed progression, and observable
  runtime behavior. It does not target historical migration vocabulary.

**Open questions**

- None.

### Phase 2: Installed Operation Semantic Closure

Install Worth UI's stable Query meaning once. Separate its semantic closure
from the volatile read and workflow executors that realize it in an operating
world.

**Relevant subsystems**

- QMOD-001 installed operation semantic closure
- QMOD-002 installed operation executors
- the Worth UI domain marker and domain package

**Relevant APIs**

- `WorthQueryDomainOperationDefinition<D, O, F>`
- `worth_ui_domain_package()`
- the Query installation surfaces for typed operation definitions and executor
  registration
- typed Worth UI operation and family markers for snapshot measurement read
  and measurement-record workflow meaning

**Warnings**

- `measurement_schema()`, selectors, result-shape fields, operation support,
  reexecution posture, aftermath posture, and mutation meaning may not be
  reconstructed inside convenience extension methods.
- Stable meaning and volatile executors must not be hidden behind one trait
  method merely because it shortens a caller.
- The measurement operation is unconditional. Condition and correspondence
  posture must be explicitly `NotRequired`; the modernization must not invent
  conditional semantics to demonstrate new Query features.
- Ordinary UI execution does not consume replay or aftermath. The operation
  closure declares the honest reexecution and aftermath posture Query requires
  without importing cert-only replay machinery into Worth UI.

**Test requirements**

- Install the real Worth UI domain package twice with equivalent semantic
  closure and prove Query produces equivalent installed operation identity;
  change the selector, result shape, or workflow touch meaning and prove the
  installed identity changes.
- Attempt real runtime construction with the semantic operation present but
  its required executor absent, extra, or registered under the wrong marker or
  family. Query's exact registration-set invariant must deny construction
  before an operating world, UI allocation, or replacement authority exists.
- Execute the registered snapshot read and measurement-record workflow through
  their real executors. Prove the installed read shape yields the exact native
  value, the installed workflow declares the mutation effect family, and the
  resulting Query mutation evidence names both recorded aspect touches rather
  than relying on a per-call builder.

**Engineering decisions**

- Snapshot-measurement read and measurement-record workflow each own one
  semantic family directory under the installed-domain boundary. Stable
  definition and volatile executor are named sibling modules inside that
  family; generic `operations/` and `execution/` category buckets are
  forbidden.
- UI-authored view declarations reference installed operation meaning; they do
  not carry independent schema or selector authority.
- QMOD-001 and QMOD-002 must close before any consumer-path rewiring so later
  phases have one real operation to bind.

**Open questions**

- None.

### Phase 3: One Operating World and Bound UI Declarations

Make Query's installed operating world the only ordinary root for Query
binding and execution. Preserve Worth UI application and declaration identity
without preserving its local mini-runtime.

**Relevant subsystems**

- QMOD-003 installed domain reference
- QMOD-004 application Query binding plan
- QMOD-005 runtime Query binding subsystem
- QMOD-006 installed UI view declaration
- QMOD-026 authored Query binding and view registration

**Relevant APIs**

- `WorthQueryInstalledOperatingWorld`
- installed domain handle and rebind receipt
- Query bound domain operation
- Worth UI application preparation, replacement candidate, authored binding
  semantics, and view registration surfaces

**Warnings**

- A `WorthUiInstalledQueryDomain` may retain an exact Query handle, but it may
  not become an alternate operating-world root or reconstruct authority from
  installation receipts.
- The Worth UI binding plan may own UI declaration references and Query-free
  posture. It may not own a parallel installed domain plus executable local
  definitions plus settlement registry.
- Old snapshot/live declaration stops must not collapse into
  `QueryDeclarationUnavailable`. Preserve Query's typed progression topology.
- File-authored and Rust-authored view declarations must converge before Query
  binding; neither authoring path may receive a private activation route.

**Test requirements**

- Start from the public Worth UI application builder, install a real Query
  workspace and domain, resolve a file-authored view to an installed operation
  reference, and enter the one binding-owned operating-world gateway to bind
  the Phase-2 operation. No deep import or second runtime binding activation
  may be required.
- Mix a UI declaration from one installed world with a handle, bound operation,
  or application generation from another. Denial must occur before execution,
  settlement storage, allocation mutation, or active-generation publication.
- Build the same binding through file-authored and Rust-authored composition.
  Both must resolve to the same UI declaration identity and exact Query bound
  operation while preserving distinct authored provenance.
- Build a Query-free application and prove it constructs, lowers, replaces,
  and frames without installing Query or carrying dummy consumer contracts.

**Engineering decisions**

- `worth-ui-query-binding` owns the single UI/Query operation gateway.
  Application preparation resolves authored bindings to installed operation
  references; replacement admission and in-generation fact refresh invoke the
  same gateway with an attempt-scoped operating-world borrow to produce fresh
  bound operations. No other UI crate binds or executes Query operations.
- The operating world and move-only bound operation never enter steady frame
  state. Active application state may retain the exact installed-domain and
  operation references needed to request the next attempt through the gateway.
- Runtime binding state retains downstream UI facts and explicitly classified
  compatibility resources only. Query execution progression stays in Query's
  proof types.
- The exact installed handle wrapper survives only as reference retention and
  UI-domain ergonomics; authority-increasing behavior routes back through
  Query.

**Open questions**

- None.

### Phase 4: Query-Minted Consumer Contract

Replace UI-local support, basis, and Query posture products with the exact
consumer contract Query mints for a bound projection. Add only independently
named UI consumer requirements at the downstream boundary.

**Relevant subsystems**

- QMOD-007 UI binding contract digest
- QMOD-008 Query prerequisite basis model
- QMOD-011 local Query support receipt
- QMOD-012 local Query binding posture

**Relevant APIs**

- `WorthQueryConsumerProjectionContract`
- `WorthQueryConsumerBoundary`
- Query consumer-boundary requirements
- a Worth UI binding-requirements type limited to allocation detail,
  result-shape presentation, denial presentation, and UI inspection relevance

**Warnings**

- `support_status_for_runtime_hook()` cannot be fixed by adding more local
  branches. The local support fold itself is competing Query authority.
- Support, basis, live compatibility, async result state, recovery, inspection,
  and projection consumption may not be recombined into a UI-local Query
  posture product.
- Denial presentation is UI meaning. It must remain distinct from Query denial
  identity and may not upgrade, flatten, or replace Query recovery posture.
- Only Query's presentation and allocation postures enter
  `WorthQueryConsumerBoundaryRequirements`. Richer UI allocation detail,
  result-shape presentation, denial presentation, and inspection relevance stay
  in the adjacent UI-owned artifact and cannot rewrite the Query boundary.
- Local definition or artifact digests may remain diagnostic summaries only.
  They may not admit execution, support replacement, or prove sameness.

**Test requirements**

- Bind the same UI declaration against two real Query consumer contracts whose
  UI labels and local diagnostic digests collide but whose installed support or
  authority differs. The contracts must remain distinct and the foreign one
  must deny before execution.
- Change only the UI denial-presentation requirement. Query support, basis,
  progression denial, and Query-minted binding identity must remain unchanged
  while the UI-only requirement changes explicitly.
- Present an unsupported or deferred Query operation through an otherwise
  valid UI hook. No UI hook, hook count, local result shape, or test constructor
  may turn the Query contract into supported.
- Attempt to mint a second consumer contract from the same bound operation.
  Query's `AlreadyMinted` denial must remain visible, and no UI retry or helper
  may silently rebind merely to obtain another contract.

**Engineering decisions**

- Each bound operation mints exactly one Query consumer contract. The binding
  owner attaches only Query's downstream presentation/allocation posture
  through `WorthQueryConsumerBoundary`, then returns that same contract to
  Query's progression rather than minting a second contract or retaining a
  copy. Richer UI-only requirements remain adjacent.
- Before settlement, attempt state may own the bound operation, the one
  unconsumed contract, and UI requirements. A Query-minted observational
  support projection is materialized only under explicit diagnostic policy;
  ordinary execution and equivalence do not pay for or depend on it. After
  `consume`, active state retains the settled projection; it does not claim to
  retain the consumed contract.
- UI-only requirements remain an adjacent artifact after Query support and its
  consumer boundary. They may be inspected together but never merged into one
  authority type or UI-authored contract identity.
- QMOD-007 survives only if inspection has a concrete need for a diagnostic
  digest; otherwise it is deleted.

**Open questions**

- None.

### Phase 5: Settled Projection to UI Measurement Fact

Consume Query's full ordinary progression and make its settled projection the
single upstream artifact from which Worth UI derives measurement and allocation
facts.

**Relevant subsystems**

- QMOD-009 consumed projection authority wrapper and index
- QMOD-010 Query measurement fact settlement
- the existing UI allocation-source generation, order, invalidation, and
  measurement family derivations

**Relevant APIs**

- Query bound operation `execute`, `publish`, `consume`, and `settle`
- `WorthQuerySettledDomainProjection`
- `WorthQueryProgressionDenial`
- Worth UI measurement fact receipt and allocation invalidation basis

**Warnings**

- UI may not join a projection outcome, execution receipt, copied basis, and
  warnings to synthesize the Query settlement that Query now owns.
- The settled projection must be retained exactly. A digest, index key, source
  label, or copied receipt is not a replacement for it.
- One retained settlement may have many UI dependents. The owner stores the
  non-cloneable Query phase artifact once; fan-out uses compact
  application-generation-scoped references and never performs per-frame
  `Arc` cloning, proof reconstruction, or global settlement lookup.
- UI allocation source generation and order begin after Query settlement. They
  describe downstream scheduling and invalidation, not Query basis authority.
- Partial results, warnings, result state, counters, publication proof, and
  conditional provenance must survive the UI boundary without flattening.

**Test requirements**

- Through the public application path, record real measurement data, execute
  the installed read, publish it, consume it under the Query contract, settle
  it, and derive the UI measurement fact used by allocation. Every transition
  must consume the prior proof type.
- Mix an execution receipt, publication receipt, consumer contract, or settled
  projection from different worlds or operation identities. Query or the UI
  join boundary must deny before a measurement receipt or allocation source is
  minted.
- Exercise a settled projection with warnings or partial result posture and
  prove UI inspection and allocation eligibility preserve the exact Query
  warnings and result state while deriving only the admitted subset of UI
  facts.
- Destroy all UI-derived authority index keys and rebuild them from retained
  settled projections. The rebuilt lookup and invalidation behavior must be
  equivalent, demonstrating that the indexes are derived.
- Refresh one Query binding inside an active application generation. The
  binding gateway must mint and consume a fresh operation chain, derive the new
  fact batch completely, and replace one settlement slot through the existing
  framework-turn source transaction. Injected denial preserves the prior slot
  and does not initiate application replacement.

**Engineering decisions**

- `WorthQuerySettledDomainProjection` replaces the old UI authority wrapper as
  the retained Query artifact.
- `WorthUiQueryMeasurementFactSettlement` remains only if renamed and shaped
  as a downstream UI derivation whose constructor requires a settled
  projection.
- Active and candidate generations own disjoint settlement slots. An admitted
  preserve transfers or reuses the existing slot only through the application
  replacement transaction; ordinary equality of keys cannot share proof
  across generations.
- In-generation refresh preserves the slot's application-generation identity
  while advancing its Query settlement and UI source generation/order through
  the framework-turn transaction. Readers see either the complete predecessor
  slot value or the complete successor value.
- The ordinary path stores no independently reconstructed Query basis identity,
  projection contract identity, or execution evidence reference.

**Open questions**

- None.

### Phase 6: Exact Replacement Equivalence and No-Op Admission

Rebase Query-aware replacement, preservation, rebind, retirement, and no-op
classification on Query-minted bound-operation identity and currentness plus
independently owned UI binding meaning. The single-use consumer contract proves
admission and progression; it is not durable replacement identity.

**Relevant subsystems**

- QMOD-013 UI binding identity
- QMOD-014 Query binding comparison and no-op equivalence
- QMOD-015 replacement admission and application no-op
- the 3.9 active/candidate application generation and atomic publication chain

**Relevant APIs**

- exact installed-domain authority plus Query-minted bound-operation binding
  identity, installation generation, and currentness
- the mint-once Query consumer contract while the candidate remains
  unexecuted, and the settled projection after progression
- Worth UI binding identity, authored result-shape requirement, and consumer
  boundary requirements
- admitted replacement candidate, comparison evidence, affected predecessor
  closure, and activation outcome

**Warnings**

- Equal artifact, definition, posture, support-projection, or rebind digests
  are insufficient when Query binding authority differs.
- Query operation or contract sameness is insufficient when the UI binding ID,
  result-shape requirement, allocation consumption, or presentation meaning
  differs.
- Diagnostic hashes may appear in receipts for compact comparison evidence.
  They must never be the only input to a preserve or no-op decision.
- Query's canonical binding identity is necessary comparison evidence, not a
  transferable capability. Preservation also requires the exact retained
  installed-domain authority to be current and belong to the candidate's
  admitted operation reference.
- This phase must reuse the closed 3.9 replacement transaction. It must not add
  a second Query-specific publication or rollback path.

**Test requirements**

- Construct equal local digests and identical UI declarations over different
  real Query bound-operation identities or installation generations.
  Replacement must classify rebind or deny, not preserve or no-op.
- Reuse the exact current Query binding identity with equivalent UI binding
  meaning across a valid replacement. The existing settled slot and unaffected
  plan rows must preserve without re-execution, broad allocation replanning,
  duplicate consumer-contract minting, or live-resource churn.
- Keep the Query binding exact while changing only UI result-shape or
  presentation requirements. The candidate must re-lower the affected UI rows
  without pretending Query support changed.
- Interrupt after Query-aware candidate admission but before final publication.
  The prior active application, settled projection, allocation catalog, and
  compatibility resource ownership must remain complete and retryable.

**Engineering decisions**

- Replacement equivalence is a product of exact installed-domain authority,
  Query-minted binding identity, installation currentness, and UI binding
  meaning. A canonical identity string alone, UI digest, or copied
  consumer-contract field matrix cannot stand in for the Query inputs.
- Candidate authority carries exact installed references and any Query phase
  values required for a genuinely changed binding into the existing 3.9
  lowering and publication chain. Exact preservation uses the retained active
  binding identity plus the candidate's equal current installed reference, so
  it does not mint a duplicate consumer contract merely to prove the same
  operation again. A no-op drops candidate-owned phase values when present; a
  changed candidate consumes them exactly once into its own settlement before
  publication.
- Installed-reference currentness is revalidated at final Query succession
  admission. Comparison evidence cannot authorize publication after an
  installation-generation turnover between lowering and activation.
- No-op classification remains application-wide and atomic; Query comparison
  contributes proof but does not independently publish state.

**Open questions**

- None.

### Phase 7: Downstream Planning, Invalidation, and Frame Ingress

Rewire the legitimate UI-owned consumers of Query meaning to the new bound
operation progression, retained settlement slot, and replacement evidence.
Delete copied Query posture while preserving UI-local plan, invalidation, and
allocation-ingress responsibilities.

**Relevant subsystems**

- QMOD-016 Query dependency invalidation and topology impact
- QMOD-017 execution-plan Query links and frame gateway
- QMOD-026 authored Query binding and view registration

**Relevant APIs**

- UI query dependency invalidation and affected-node narrowing
- execution-plan query binding input and lane support links
- allocation-frame Query fact gateway
- generation-scoped settlement references and UI measurement fact receipts

**Warnings**

- `Query` in a filename does not make a subsystem Query-owned. UI dependency
  invalidation, plan-row selection, and allocation ingress remain UI
  responsibilities.
- Retained subsystems must receive proof-bearing inputs. Renaming copied local
  posture to sound more upstream does not modernize it.
- Steady frames must not reopen operation declarations, consumer support,
  result-shape planning, or broad application artifacts.
- Plan rows may identify a retained settlement slot and UI fact slice. They may
  not own the operating world, bound operation, consumer contract, entire
  settlement store, or a reference whose validity is independent of the
  active application generation.

**Test requirements**

- Change one Query binding in a large real application generation. Query
  comparison, invalidation, plan lowering, and allocation ingress must touch
  only the exact affected binding and dependent UI rows; exact counters must
  remain independent of unrelated application width.
- Change an unrelated appearance or source declaration while preserving the
  exact current Query binding. No Query re-execution, support recomputation,
  settlement replacement, or Query-driven invalidation may occur.
- Feed a settled projection from the wrong application generation or Query
  binding into the frame gateway. It must deny before allocation ingress or
  plan-row execution.

**Engineering decisions**

- Retained UI derivations are organized by their UI responsibility, not placed
  in a generic Query modernization module.
- The frame gateway accepts only UI-derived facts backed by a retained settled
  projection and the exact active application generation.
- Settlement storage owns heavyweight Query proof; plan storage owns compact
  links. Resolving a requested link is direct or region-indexed and performs no
  global search, proof clone, or support recomputation.

**Open questions**

- None.

### Phase 8: Exact Diagnostics and Lifecycle-Ordered Facades

Project the modernized Query boundary for operators and external callers
without flattening Query outcomes or exporting its operating-world and
compatibility internals.

**Relevant subsystems**

- QMOD-018 Query diagnostics and inspection projection
- QMOD-021 public Query binding facades

**Relevant APIs**

- Query progression denials, warnings, result state, counters, Query-minted
  binding/support observations, and settled projection
- compact Worth UI inspection links and richer diagnostic projections
- curated Worth UI, runtime, and query-binding facades

**Warnings**

- Compact diagnostics may summarize, but every summary must link to the exact
  Query stop, warning, binding observation, or settlement artifact it projects.
- A UI status enum that repeats Query stop classes is still a competing model
  even when its variants happen to map one-to-one today.
- Public ergonomics do not justify exporting an operating-world constructor,
  local support mint, settlement constructor, compatibility resource, or
  diagnostic digest as authority.
- Diagnostics are derived and policy-governed. Richness changes must not alter
  execution, replacement, or allocation outcomes.

**Test requirements**

- Drive Query success, partial-with-warning, unsupported, wrong-world, stale,
  and rebind-required outcomes through compact inspection and rich diagnostics.
  Each remains structurally distinguishable and links to the exact upstream
  artifact.
- Request minimal and rich evidence for the same settled projection. Both must
  report the same operational outcome and identity while materialization cost
  changes only according to the evidence policy.
- Compile an external public-facade consumer that completes the ordinary
  binding and inspection journey, and a negative consumer that attempts to
  mint local support or reach managed-live compatibility internals.

**Engineering decisions**

- Facades expose lifecycle order and typed outcomes while hiding operating-
  world internals, compatibility mechanics, and diagnostic-only digests.
- Inspection projects upstream proof plus UI relevance; it stores no second
  support, basis, settlement, or recovery authority.
- Inspection references cannot be passed back into execution, replacement, or
  allocation admission. Observation is a one-way edge.

**Open questions**

- None.

### Phase 9: Managed-Live Compatibility Containment

Contain the existing managed-live declaration, resource, and succession path
behind one honest compatibility boundary until Query supplies the remaining
ordinary lifecycle and collection-delivery primitives. Preserve real behavior
without pretending snapshot-operation modernization completed live delivery.

**Relevant subsystems**

- QMOD-019 managed live projection resource
- QMOD-020 live binding succession and rebind
- QMOD-024 replay, aftermath, and operation lineage non-applicability

**Relevant APIs**

- existing Query-managed live projection and resource receipts
- Worth UI live admission, read, succession, retirement, and exact-once close
- Query-minted snapshot binding identity, currentness, and retained settlement
  proof; any fresh snapshot execution still uses a new mint-once contract
- future Query 9.14 Phase 17 capability lifecycle, Phase 19 consumer leases,
  Phase 23 collection windows, and Phase 24 patch delivery

**Warnings**

- The compatibility boundary is not permitted to mint a
  `WorthQueryConsumerProjectionContract`, settled projection, operation
  lineage, replay receipt, or aftermath receipt.
- UI replacement succession is the lifecycle of a mounted UI consumer. It is
  not Query data identity evolution and must not use Phase-14 lineage types to
  make the old live lane look modern.
- Replay and aftermath are cert-only Query concerns and remain outside ordinary
  UI crates.
- `transitional` is not an indefinite label. The inventory names the upstream
  exit trigger, the bounded surfaces, and the owner of removal.

**Test requirements**

- Open a real managed-live Query resource through the compatibility boundary,
  perform preserve, rebind, retirement, rollback, and final disposal across
  public application replacements, and prove exact-once close with no orphan or
  stale active resource.
- Attempt to feed a compatibility receipt or UI succession receipt into the
  operation-native consumer, settlement, equivalence, or lineage APIs. The
  type boundary must make promotion unavailable or the owning boundary must
  return a typed denial before use.
- Prove a snapshot-only Query binding never constructs the live compatibility
  subsystem, registers a live resource, or pays live succession cost.
- Prove a live compatibility denial preserves the prior complete application
  generation and exact settled snapshot projection without partial retirement.

**Engineering decisions**

- The compatibility code lives under a path named by its constraint, such as
  `compatibility/managed_live`, and is absent from the ordinary snapshot
  consumer facade.
- The modernization does not block on unfinished upstream phases because the
  existing real resource behavior remains contained and truthful.
- Once the inventory exit trigger is satisfied, a follow-on cutover replaces
  this boundary rather than widening it.

**Open questions**

- None.

### Phase 10: Real Consumer-Path Certification and Closure

Certify the modernized boundary through the real public application and Query
mechanisms, update every inventory row with its implemented disposition, and
close without adding migration-shaped tests or compile-cost islands.

**Relevant subsystems**

- QMOD-022 Query certification fixtures and lifecycle scenarios
- QMOD-023 runtime identity-state-Query residue certification
- QMOD-025 crate dependency boundary
- all rows in the modernization inventory
- all rows in the boundary and edge matrix
- the existing `worth-ui-certification` application-contract and compile-
  contract owners

**Relevant APIs and evidence**

- real temporary `.wui` file ingress and production application preparation
- real Query domain installation, operating world, operation binding,
  execution, publication, consumption, and settlement
- public replacement, frame, allocation, inspection, and cleanup lifecycle
- boundary checker, generated agent-context checker, line-cap gate, workspace
  tests, clippy, and existing compile-contract aggregates

**Warnings**

- Do not create one integration target, fixture workspace, nested Cargo build,
  or cold-start program per inventory row or phase.
- Do not manufacture Query support, basis, consumer contracts, settlements, or
  local digests in order to make a lifecycle test convenient.
- Do not write a source sentinel whose claim is that the migration succeeded
  because old names or paths are absent.
- Runtime residue certification may inspect actual active state. That is
  behavioral proof and should remain distinct from source-token scanning.
- Closing timing evidence must be regenerated only after the implementation and
  test topology stabilize.

**Test requirements**

- Extend the existing compiled-once application-contract target with one real
  vertical journey: write actual `.wui` bytes, let the production ingress
  prepare the application, install and bind the real Query operation, execute,
  publish, consume, settle, lower, activate, frame, inspect, replace, and clean
  up through public facades.
- In that same compiled scenario family, exercise foreign-world, stale-handle,
  mixed-receipt, equal-diagnostic-digest, unsupported-contract, partial-warning,
  duplicate-contract-mint, interrupted-publication, and live-compatibility
  failures. Each must deny at its owning boundary while prior active truth
  remains complete.
- Assert exact structural counters for operation binding, Query execution,
  retained settlement slots, derived fact batches, compact link resolutions,
  affected bindings, affected plan rows, allocation ingress, replacement
  publication, and resource cleanup. Scale both dependents per settlement and
  unrelated UI width independently to prove fan-out and delta-bounded work.
- Run Query-free and snapshot-only variants through the same application owner
  and prove they carry no dummy Query world or managed-live cost.
- Rerun the inventory searches as a review aid, manually adjudicate any new
  subsystem, and set each row's implementation status to its honest final
  disposition. No test asserts a target match count or zero old tokens.
- Review every edge against the implemented producer, consumer, lifetime,
  failure, and counter evidence; update its implementation status only after
  the real crossing proves the recorded contract.

**Engineering decisions**

- Real behavioral scenarios are consolidated by shared application lifecycle,
  not mechanically one test per migration row.
- Existing compile targets and checked-in compile owners absorb new proof. The
  milestone adds no private build graph or generated fixture workspace.
- Dependency direction is mechanically enforced because it protects present
  authority. Historical migration vocabulary remains a review concern recorded
  in the CSV.

**Open questions**

- None.

## Must Ship

- the 26-row manually adjudicated subsystem inventory, updated if implementation
  search discovers another semantic subsystem
- the manually adjudicated boundary and edge matrix, updated when implementation
  discovers another authority crossing
- typed Worth UI installed operation definitions with separate executors
- one Query operating-world entry for prepared Query-backed applications
- exactly-once Query consumer-contract minting and consumer-boundary
  progression
- retained `WorthQuerySettledDomainProjection` authority feeding UI-owned
  measurement and allocation facts through generation-owned settlement slots
- replacement and no-op equivalence based on Query-minted binding identity and
  currentness plus UI binding meaning
- proof-bearing downstream planning, invalidation, frame-ingress, diagnostic,
  and facade surfaces
- one explicitly bounded managed-live compatibility seam with an upstream exit
  trigger
- real public application/Consumer Kit certification inside existing compile
  and integration targets

## Must Preserve

- all closed Milestone 3.9 truths outside the obsolete Query assumptions:
  canonical application authority, real `.wui` ingress, atomic replacement,
  regional plan and allocation locality, sealed host output, exact cleanup, and
  frame-cost accounting
- UI-owned binding identity, authored result-shape and presentation meaning,
  dependency invalidation, allocation ingress, and diagnostic relevance
- Query-free applications with no Query ceremony
- prior active application truth on every denial or interrupted replacement
- Query-owned warnings, result state, denials, counters, and resource lifecycle
  without lossy local status translation
- the build-topology and compiler-session cost discipline closed by Milestones
  3.8.1 and 3.9

## Acceptance Evidence

- Every inventory row has a resolved classification, manual reason, target
  owner, phase, completion evidence, and—where transitional—exit trigger.
- Every edge-matrix row names its producer, consumer, cardinality, lifetime,
  failure owner, cost contract, allowed dependency, and forbidden shortcut.
- Production no longer constructs stable Query operation meaning per call,
  folds Query support from UI hooks, reconstructs Query basis or settlement, or
  uses a local digest as Query admission/equivalence authority.
- One public real-mechanism scenario proves the full Query operation progression
  and downstream UI frame lifecycle without manufactured Query authority.
- Equal-representation/different-authority hostility and
  exact-authority/different-UI-meaning hostility both produce the correct
  typed replacement outcome.
- One-settlement/many-dependent hostility retains one heavyweight settlement,
  rejects cross-generation references, and keeps steady-frame work bounded by
  requested facts rather than total fan-out.
- Runtime-state certification finds no orphan settlement, stale active live
  resource, mixed application generation, or local Query status authority.
- Boundary, agent-context, line-cap, format, clippy, test, and compile-contract
  gates pass without a new nested build, fixture workspace, or integration-
  target explosion.
- The migration search is retained as review provenance in the CSV. No passing
  test depends on an old symbol or path being absent.

## Sequencing Notes

- Milestone 3.9 remains historically closed. Milestone 3.9.1 immediately
  follows it because the modernization changes the Query authority substrate
  consumed by Milestones 3.10, 3.12, 3.13, and 6.
- Phase order is mandatory: inventory before edits; stable operation meaning
  before operating-world binding; binding before consumer support; consumer
  support before settlement; settlement before replacement equivalence;
  equivalence before downstream rewiring; ordinary snapshot closure before
  compatibility containment; all of it before certification closeout.
- Query 9.14 Phases 1 through 14 are the available modernization foundation.
  This milestone does not claim the unfinished live lifecycle, lease, window,
  or patch-delivery phases.
- When Query 9.14 Phases 17, 19, 23, and 24 are complete and public, the
  QMOD-019/QMOD-020 exit trigger requires a named follow-on cutover. It does not
  silently broaden Milestone 3.9.1 after closure.
