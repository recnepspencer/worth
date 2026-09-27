# Milestone 3.4 Engineering Spec: Admission, Support, And Graph Touch Obligations

> **Status:** Planned
>
> **Roadmap parent:** [worth_ui_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/worth-ui/plans/worth-ui/worth_ui_roadmap.md)
>
> **Primary prerequisite:** `Milestone 3.3 UI Authority Graph, Identity, Participation, And Core Indexes`
>
> **Follow-on sequence:** `Later Milestone 3.x inspection-evidence and runtime-explanation slices`
>
> **Primary architectural driver:** make touched UI meaning select typed
> runtime obligations through one admission boundary instead of caller-memory,
> validator folklore, renderer-local legality checks, or broad post-mutation
> graph scans.

## Goal

Make Worth UI obligation selection, support posture, and legality admission
runtime-owned, typed, world-aware, and mechanically inspectable.

Milestone 3.4 is complete when callers can describe touched runtime meaning
through a typed graph-touch artifact; the runtime can combine that touch with
graph authority, declaration contracts, operating world, support posture, and
admitted obligation-family metadata; and one canonical admission boundary can
select, dispatch, and verdict the required obligations without broad scans,
validator folklore, renderer-local legality, or UI-local pseudo Query/runtime
reconstruction.

This milestone closes one authority boundary:

- what touched runtime meaning is being proposed
- what support posture applies to that touch in the current world
- which obligation families become eligible from that touched meaning
- which obligations are selected for ordinary admission
- what typed verdict each selected obligation returned
- what typed admission report, denial posture, and evidence neighborhood later
  inspection or diagnostics may consume

It does not close Query execution, measurement execution, host observation
capture, intent routing, service execution, mounted receipt production, or
human inspector UI.

## Non-Goals

Milestone 3.4 does not implement:

- declaration lowering
- runtime graph construction or graph identity
- measurement planning or host measurement exchange
- Query execution or projection materialization
- intent execution or mutation submission
- service execution for portal, focus, motion, or scroll lanes
- mounted receipt planning
- replay UI
- visual snapshot capture
- human inspector panels
- full cross-runtime explanation breadth

3.4 closes selection and legality only:

- graph-touch declaration
- support posture for touched meaning
- obligation-family admission
- selected-obligation planning
- typed obligation verdicts
- typed admission reports and denial posture
- obligation evidence hooks and inspection readiness for later slices

## Why This Sequence Exists

Milestone 3.1 created the runtime boundary, support vocabulary, and inspection
facade. Milestone 3.2 created canonical declaration authority and typed
semantic lanes. Milestone 3.3 created graph truth, participation truth, and
bounded indexes. Milestone 3.4 is the next load-bearing slice: once the
runtime knows what authored meaning exists and what runtime nodes currently
exist, it must decide which checks follow from touched meaning without forcing
callers to remember validator packs or broad graph legality walks.

This is not "a validation layer." It is the runtime's semantic selection
boundary:

- later measurement and allocation slices need touched meaning to admit
  measurement requirements without rediscovering them from renderer behavior
- later Query-binding slices need touched meaning to admit Query-bound work
  without UI-local pseudo Query status models
- later intent and service slices need touched meaning to admit operability,
  portal, focus, and service requirements through the same runtime contract
- later inspection slices need selected-obligation and verdict evidence as
  first-class runtime artifacts instead of logs or strings

If 3.4 stays vague, later milestones will rebuild legality through command
helpers, renderer-local prechecks, host heuristics, or app-specific validator
tables. That would turn one of the most important runtime boundaries into
folklore.

## Governing Summaries

- `MENTALITY.md`
  protects adversarial-constraint-first design. 3.4 must start from hostile
  touch churn, world changes, Query-backed change, host observations, and
  partial participation rather than from a pleasant "validate this node" demo.
- `arch_laws.md`
  protects compile-time contracts, authority-vs-derivation separation, proof
  progression, and structured outcomes. 3.4 must separate support from
  admission, touch declaration from execution, and verdict structure from
  presentation text.
- `composition_laws.md`
  protects named semantic steps. 3.4 must not collapse support posture, touch
  classification, obligation selection, dispatch planning, verdict mapping, and
  diagnostics into one admission facade or god module.
- `domain_structure_laws.md`
  protects structural separability. Touch descriptors, support posture,
  obligation-family catalogs, dispatch planning, verdict artifacts, inspection
  evidence, and certification must live in distinct homes because they fail,
  scale, and broaden differently.
- `perf_laws.md`
  protects breadth honesty and upstream planning. 3.4 must select obligations
  from typed touch facts and graph/index authority rather than broad rescans,
  repeated rediscovery, or broad post-mutation validation.
- `worth_ui_roadmap.md`
  requires `UiGraphTouchDescriptor`, `UiSelectedObligationSet`,
  `UiObligationDispatchPlan`, `UiObligationVerdict`, `UiAdmissionReport`,
  world-aware support/admission posture, and the initial obligation families.
  The roadmap explicitly says this milestone closes the "who selects checks?"
  boundary.
- `worth-ui-vision.md`
  protects Worth UI as a desktop runtime platform rather than a widget bundle.
  3.4 therefore has to make support, legality, and runtime posture platform
  capabilities instead of app-local callback or validator glue.
- `WORTH_UI_README.md`
  protects the runtime-owned flow:
  declaration artifact -> support/admission -> graph -> selected obligations ->
  Query binding/projection consumption -> measurement -> mounted receipts.
  3.4 must own the obligation selection step as its own authority boundary.
- `worth-ui-dsl-vision.md`
  protects semantic-lane authoring. 3.4 must consume authored touch meaning,
  aspect contracts, and lane-local semantics rather than treating touched work
  as generic component change.
- `ai-diagnostics.md`
  protects typed evidence and one runtime-owned explanation substrate. 3.4
  therefore must emit typed obligation/admission evidence and support posture
  artifacts alongside the runtime family it introduces.
- `crates/worth-query/docs/AI_README.md`
  protects Query as the ordinary domain/runtime facade. For 3.4 that means
  Query-backed support, basis, projection consumption, inspection, and causal
  explanation stay Query-owned:
  `ResolvedSnapshotBasis`, `SnapshotResolutionReport`,
  `consume_projection_facts(...)`, `workspace.inspect(...)`,
  `admit_causal_inspection`, and `request_causal_inspection` remain the
  admitted public lanes.

## Adversarial Constraint

3.4 must survive this hostile condition:

> Worth UI repeatedly processes declaration edits, graph mutations, Query-backed
> fact changes, host observations, and service-triggered updates across
> authoritative, preview, branch, diagnostic, and host-observation worlds.
> Those touches affect repeated instances, slot occupancy, participation axes,
> Query bindings, measurement requirements, host capabilities, and future
> service attachments. Across that churn, the runtime must classify touched
> meaning precisely, select only the obligation families actually justified by
> the touch, deny unsupported or illegal work through typed posture, preserve
> the distinction between support and admission, and expose retained evidence
> without broad graph scans, renderer-local interpretation, caller-remembered
> validator packs, or UI-local pseudo Query/runtime models.

If a caller must remember which checks to run, if a node-level touch silently
widens into graph-global validation, if world posture is implicit, if
Query-backed denials are reconstructed locally, if selected obligations cannot
be explained structurally, or if unsupported versus denied work collapses into
one generic failure, 3.4 is not closed.

## Product Decision Lock

- `UiGraphTouchDescriptor` is the one canonical touched-meaning artifact for
  this boundary.
- Support posture and admission posture are separate runtime artifacts. Support
  says whether a lane belongs and is admitted now; admission says whether a
  specific touched operation passes now.
- Obligation selection is runtime-owned. Callers declare touch; callers do not
  select validator packs.
- Obligation families are explicit and closed. New semantic work admits a new
  family or a new family-local lane; it does not widen a catch-all validator.
- Touch origin, operating world, and affected aspect lanes are first-class
  classification inputs, not optional metadata.
- `UiSelectedObligationSet` is a lowered runtime plan artifact, not an
  implementation detail or transient local list.
- `UiObligationVerdict` is typed and tri-state-capable. Binary pass/fail is
  illegal for the ordinary runtime lane.
- Query-owned support, basis, projection-consumption, inspection, and
  explanation surfaces remain Query-owned. Worth UI may require or attach to
  them; it may not restate them as UI-local booleans, labels, or ad hoc report
  structs.
- Host adapters must not decide support, legality, or obligation selection.
- Diagnostics and later inspection consume obligation evidence; they do not
  reconstruct it from logs or host behavior.

## Touch And Obligation Authority Progression

3.4 must preserve explicit phase progression:

`UiDeclarationArtifact -> UiDeclarationGraphHandoff -> UiGraphSnapshot -> UiGraphTouchDescriptor -> UiSupportSnapshot / UiAdmissionPosture -> UiSelectedObligationSet -> UiObligationDispatchPlan -> UiObligationVerdict -> UiAdmissionReport`

The exact type names may vary, but the progression law may not:

- declarations are not graph touch
- graph truth is not support posture
- support posture is not selected obligations
- selected obligations are not dispatch execution
- verdicts are not user-facing strings
- admission reports are the canonical retained outcome for this boundary

If production code can jump from graph mutation or source edit directly into
ad hoc local validators, the phase progression has collapsed.

## Touch Precision Contract

`UiGraphTouchDescriptor` must be precise enough that the runtime can select
obligations without widening to folklore.

The touch contract must distinguish at least:

- touched graph target class
  - node
  - edge or slot occupancy
  - page/region/mosaic membership
  - graph-owned attachment lane
- touch origin
  - declaration change
  - Query-backed fact change
  - host observation
  - service event
  - intent submission
  - diagnostic-only touch
- operating world
  - authoritative
  - preview
  - branch
  - hot-reload-candidate
  - diagnostic
  - host-observation
  - test-certification
- affected aspect posture
  - read
  - written
  - invalidated
  - preserved
- affected runtime lanes
  - structural
  - participation
  - measurement
  - query binding
  - intent operability
  - service
  - host capability
  - diagnostic

The exact field names may differ, but the semantic precision may not.

`UiGraphTouchDescriptor` must also carry typed touch timing:

- `pre_mutation`
- `post_mutation`
- `reactive_observation`
- `diagnostic_projection`
- `replay_evaluation`

This timing is not optional metadata. It determines whether an obligation is
screening a proposed mutation, reacting to already-admitted change, consuming a
host/runtime observation, or projecting diagnostic/replay posture.

3.4 does not allow "node changed" as the ordinary touch boundary. That shape is
too coarse to support bounded obligation selection, later rebind breadth
control, or later inspection explanation.

`UiGraphTouchDescriptor` may be constructed only from admitted upstream sources:

- declaration change receipt
- graph mutation receipt
- Query projection/change receipt
- host observation receipt
- service event receipt
- intent submission receipt
- diagnostic-only request receipt

The exact wrapper names may vary, but the law may not: callers must not mint
"something changed near node X" touches from raw intent or ambient memory.
Touch is a projection of a real upstream event/receipt whenever the runtime has
one.

## Support And Admission Contract

3.4 must make support and admission different kinds of truth.

Support answers:

- does this obligation family belong architecturally?
- is it admitted in this runtime profile and operating world?
- is it available on the ordinary path, deferred, diagnostic-only, or
  unsupported?

Admission answers:

- given this exact touch, graph posture, and operating world, does the touched
  operation pass now?
- which selected obligations returned success, advisory, or violation?
- what evidence and denial posture explains the result?

Required posture distinctions include at least:

- supported
- unsupported
- deferred
- diagnostic-only
- wrong-world
- wrong-host-capability
- wrong-query-basis
- stale
- ambiguous
- rebind-required
- budget-exceeded

The exact naming may vary, but the separation may not. A caller must be able to
tell the difference between:

- "this lane belongs here but is not admitted yet"
- "this lane is admitted but this specific touch violated it"
- "this lane is admitted only in another world or basis posture"

## Obligation Selection Law

Obligation selection must be derived from typed touch meaning plus admitted
support posture and graph/index authority.

At minimum, 3.4 must admit one closed family taxonomy that can express:

- `structural-legality`
- `participation-legality`
- `slot-contract`
- `measurement-requirement`
- `query-binding-requirement`
- `intent-operability-requirement`
- `portal-host-requirement`
- `focus-route-requirement`
- `motion-support-requirement`
- `accessibility-requirement`
- `host-capability-requirement`
- `diagnostic-surface-requirement`

Not every family must be fully ordinary-path-supported in 3.4, but every family
that belongs architecturally to touched UI meaning must have typed support
posture and typed selection/admission behavior. Unsupported families must fail
through support posture, not disappear from the model.

The ordinary path is:

`touch descriptor + graph truth + support posture + obligation-family catalog -> selected obligation set`

Each selected obligation must also carry:

- `UiSelectedObligationIdentity`
  - `touch_identity`
  - `obligation_family_identity`
  - `target_identity`
  - `aspect_scope`
  - `world_profile`
  - `support_row_identity`
- `UiObligationCheckKind`
  - `blocking_invariant`
  - `prerequisite_requirement`
  - `capability_gap_screen`
  - `world_gate`
  - `advisory_check`
  - `diagnostic_only_check`
  - `deferred_backstop`
- `UiObligationSelectionReason`
  - selected by touch target
  - selected by touch origin
  - selected by aspect posture
  - selected by graph participation
  - selected by declaration contract
  - selected by support row
  - selected by world profile
  - selected by Query prerequisite
  - selected by host-capability prerequisite

`UiObligationCheckKind` is separate from obligation family. Family names the
semantic domain; check kind names how the selected obligation behaves at this
boundary. Without that split, obligation family becomes overloaded and later
runtime slices will have to reconstruct semantics from family names alone.

Selection must not depend on:

- renderer-local classification
- host-local prevalidation
- caller-supplied validator arrays
- broad graph walks hidden behind scalar APIs
- reopened source text
- UI-local reclassification of Query readiness or support

3.4 must also ship one explicit starter selection matrix so the ordinary path
is mechanically visible rather than prose-only. The initial matrix may evolve,
but it must include representative rows such as:

- touch lane: structural + slot occupancy
  - origin: declaration change
  - world: hot-reload-candidate
  - selected:
    - `structural-legality`
    - `slot-contract`
    - `participation-legality` when participation aspects were touched
    - `diagnostic-surface-requirement` when denial presentation is required
- touch lane: measurement
  - origin: host observation
  - world: host-observation
  - selected:
    - `measurement-requirement`
    - `host-capability-requirement`
- touch lane: query binding
  - origin: Query-backed fact change
  - world: authoritative
  - selected:
    - `query-binding-requirement`
    - `participation-legality` when participation/presence aspects were touched
    - `diagnostic-surface-requirement` when stale/denied posture changed
- touch lane: diagnostic
  - origin: diagnostic-only touch
  - selected:
    - `diagnostic-surface-requirement`

This matrix is not the whole future planner. It is the initial executable law
that proves selection is runtime-owned and touch-shaped.

## Query And Host Boundary Law

3.4 must consume Query-owned and host-owned lanes without restating them.

For Query-backed obligation work:

- basis posture comes from Query-owned basis artifacts such as
  `ResolvedSnapshotBasis` and `SnapshotResolutionReport`
- typed materialized facts come from `consume_projection_facts(...)`
- retained inspection stays on `workspace.inspect(...)`
- cross-runtime why stays on `admit_causal_inspection` and
  `request_causal_inspection`

Worth UI may:

- require that those Query-owned facts or postures exist
- attach graph/node/obligation identity to them
- report typed denials when those Query-owned prerequisites are unsupported or
  unavailable

Worth UI may not:

- rebuild Query support posture locally
- replace basis artifacts with labels or booleans
- replace projection consumption with local cache spelunking
- treat `workspace.inspect(...)` as support or admission

For host-backed obligation work:

- host capability posture is a typed prerequisite
- host adapters report capabilities and observations
- host adapters do not decide obligation legality

This milestone must therefore make Query-prerequisite and host-prerequisite
requirements explicit obligation inputs, not implementation-side guesses.

3.4 dispatch executes only obligation-checking work admitted for this
milestone. It does not execute the runtime family whose requirement is being
checked.

- a `measurement-requirement` verdict may say measurement support is required,
  unsupported, diagnostic-only, deferred, or prerequisite-satisfied; it may not
  perform measurement planning
- a `service`-related requirement verdict may say portal/focus/motion support
  is required, unsupported, diagnostic-only, deferred, or
  prerequisite-satisfied; it may not execute the service
- a `query-binding-requirement` verdict may cite Query-owned basis, projection,
  inspection, or support evidence; it may not execute Query or recreate Query
  admission

This guard keeps 3.4 as selection/admission infrastructure rather than an early
implementation of later measurement, Query, intent, or service milestones.

## Evidence And Inspection Readiness Law

Every runtime family introduced in 3.4 must ship typed evidence and inspection
hooks at its own boundary.

At minimum, the obligation/admission family must retain:

- touch identity
- selected-obligation identity
- obligation-family identity
- support posture evidence
- admission posture evidence
- verdict identity and verdict class
- upstream graph/declaration identity
- Query evidence refs where Query-backed posture mattered
- host capability evidence refs where host posture mattered
- denial/advisory/violation rationale artifacts

The first serious 3.4 evidence lane does not need the final human inspector UI,
but it must leave later inspection slices with real retained artifacts instead
of only log lines or strings.

Selection evidence must be strong enough to answer:

- why did `focus-route-requirement` run?
- why did `measurement-requirement` not run?
- why did an appearance-only change avoid `structural-legality`?

If later inspection can only answer those through logs or source archaeology,
3.4 has left the AI/human inspection architecture underpowered.

## Obligation Budget And Cost Law

3.4 must make obligation-selection and obligation-dispatch cost explicit.

At minimum, the milestone requires:

- `UiObligationSelectionBudget`
- `UiObligationSelectionCostReceipt`
- `UiObligationDispatchBudget`
- `UiObligationBudgetVerdict`

Minimum counters include:

- `graph_nodes_considered`
- `index_lookups_consumed`
- `obligations_selected`
- `families_considered`
- `families_denied_by_support`
- `query_prereq_refs_loaded`
- `host_capability_refs_loaded`

The exact field names may vary, but the budget/cost law may not. Large or broad
operations must deny honestly through typed budget posture rather than hiding
unbounded graph walks behind "validation" or "selection" language.

## Planned Directory Skeleton

3.4 should force the runtime tree toward an admission-owned and
obligation-owned responsibility shape rather than one validation blob. The
exact filenames may evolve, but the topology should look like this:

```text
workspaces/worth-ui/crates/worth-ui-runtime/src/
  admission/
    support/
      support_snapshot.rs
      support_row.rs
      world_support.rs
      unsupported_posture.rs
      mod.rs
    admission/
      admission_posture.rs
      admission_report.rs
      denial.rs
      advisory.rs
      mod.rs
    mod.rs
  obligations/
    touch/
      touch_descriptor.rs
      touch_origin.rs
      touch_world.rs
      touch_aspect_posture.rs
      mod.rs
    catalog/
      obligation_family.rs
      family_support.rs
      family_selection_rules.rs
      mod.rs
    selection/
      selection_context.rs
      selected_obligation_set.rs
      selection_report.rs
      mod.rs
    dispatch/
      obligation_dispatch_plan.rs
      dispatch_input.rs
      mod.rs
    verdict/
      obligation_verdict.rs
      verdict_class.rs
      verdict_evidence.rs
      mod.rs
    diagnostics/
      obligation_diagnostic.rs
      admission_diagnostic.rs
      mod.rs
    inspection/
      obligation_inspection_refs.rs
      admission_inspection_refs.rs
      mod.rs
    mod.rs
```

Public surfaces should stay curated through the Worth UI facade.
`worth-ui-inspection` should own formal query/receipt contracts over retained
evidence. `worth-ui-certification` should own hostile residue scans,
compile-fail proof, world-matrix proof, and ordinary-path anti-cheating suites.

## Obligation Certification Matrix

Every major 3.4 contract surface must map to a named proof family:

- `graph_touch_descriptor_suite`
  proves typed touch origin/world/aspect classification, touch equivalence, and
  anti-coarsening proof.
  owning crate: `worth-ui-certification`
  enforcement: compile-fail plus runtime certification
- `support_and_admission_posture_suite`
  proves support/admission separation, world-aware posture, and typed denial
  classes.
  owning crate: `worth-ui-certification`
  enforcement: runtime certification plus support-profile audit
- `obligation_family_catalog_suite`
  proves the admitted family taxonomy, family-local support posture, and
  no-catch-all-family boundary.
  owning crate: `worth-ui-certification`
  enforcement: runtime certification plus compile-fail boundary proof
- `obligation_selection_suite`
  proves selected obligations derive from typed touch meaning plus graph/index
  authority rather than caller-supplied validator sets or broad scans.
  owning crate: `worth-ui-certification`
  enforcement: hostile runtime certification plus residue scan
- `dispatch_and_verdict_suite`
  proves selected obligations lower to typed dispatch plans and typed verdicts
  with tri-state-capable outcome structure.
  owning crate: `worth-ui-certification`
  enforcement: runtime certification plus compile-fail boundary proof
- `query_boundary_requirement_suite`
  proves Query-backed requirements consume Query-owned basis, projection, and
  inspection lanes rather than UI-local substitutes.
  owning crate: `worth-ui-certification`
  enforcement: runtime certification plus dependency audit
- `host_capability_requirement_suite`
  proves host-capability requirements stay explicit and host adapters do not own
  legality or support decisions.
  owning crate: `worth-ui-certification`
  enforcement: runtime certification plus dependency audit
- `obligation_evidence_suite`
  proves selected-obligation and admission evidence is retained as typed runtime
  artifacts and is inspection-ready.
  owning crate: `worth-ui-certification`
  enforcement: runtime certification plus inspection-boundary proof
- `public_facade_boundary_suite`
  proves callers cannot bypass the facade to WORTH touch descriptors, selected
  obligations, or verdict identities from raw values.
  owning crate: `worth-ui-certification`
  enforcement: compile-fail plus topology audit

Every suite must name its hostile lane, anti-bypass lane, exact public surface,
and the architectural law it is proving.

## Test Topology Requirements

3.4 tests must obey the same structure law as production:

- touch classification proof belongs with touch authority, not in a generic
  runtime helper file
- support/admission proof belongs under admission/support, not inspection-only
  broad suites
- obligation-family proof belongs with the family catalog, not future service
  or intent tests
- Query-backed requirement proof belongs with Query-boundary tests, not UI-local
  fixture piles
- host-capability proof belongs with host-contract boundary tests, not adapter
  implementation smoke suites

Required hostile topology:

- compile-fail fixtures for raw construction of touch descriptors, selected
  obligation sets, dispatch plans, verdict wrappers, and typed denial surfaces
- residue scans proving production code does not select obligations from
  renderer-local validators, host-local validator tables, or reopened source
  text
- world-matrix tests proving the same touch can admit differently across worlds
  without collapsing posture into one boolean
- Query-boundary tests proving Query-backed requirements consume Query-owned
  basis/projection/inspection artifacts instead of local cache or row access
- breadth-honesty tests proving ordinary selection does not recurse through the
  entire graph for local touches
- localization tests proving an appearance-only or diagnostic-only touch does
  not silently trigger unrelated structural or intent requirements
- support-vs-admission tests proving supported-but-denied and unsupported lanes
  remain distinguishable in reports and diagnostics
- evidence tests proving selected-obligation and admission artifacts are
  retained, typed, and inspectable without logs

Phase-local adversarial proof must be the ordinary bar, not a later cleanup
exercise.

## World-Sensitivity Law

3.4 must make world-sensitivity explicit.

- support posture is world-aware
- admission posture is world-aware
- touch descriptors carry operating world explicitly
- obligation selection may differ by world, but that difference must be typed
  and explainable
- verdicts must preserve whether denial came from world posture, support
  posture, Query basis posture, host capability posture, or ordinary legality

This prevents accidental conflation between:

- a touch that is unsupported in all worlds
- a touch that is supported only in preview or diagnostic worlds
- a touch that is supported but denied in the current world
- a touch whose Query basis or host capability posture is wrong for the current
  world

## Phases

### Phase 1: Freeze The Admission Boundary And Split Support From Legality

Phase 1 defines the one authoritative admission boundary and prevents "support"
and "did this operation pass?" from collapsing into one vague status surface.

**Relevant subsystems**

- runtime admission lane
- support snapshot lane
- obligation family catalog
- inspection/support posture lane
- certification support-vs-admission suites

**Relevant APIs**

- `UiSupportSnapshot`
- support row / world support posture surface
- `UiAdmissionReport`
- typed admission posture / denial surface
- selected-obligation result envelope
- admission report aggregation lattice

**Warnings**

- Do not make support a preflight string helper and admission a later boolean.
- Do not let admission reports hide whether the failure came from unsupported
  family posture, wrong world, wrong Query basis, wrong host capability, or
  ordinary legality violation.
- Do not let public support posture imply that every visible public type is
  admitted on the ordinary path.

**Test requirements**

- Separation test: the same obligation family can be `supported` in a world yet
  still yield an admitted touch that returns advisory or violation verdicts.
- Rejection test: an unsupported or deferred family fails through typed support
  posture before ordinary legality dispatch rather than pretending to be a
  local denial.
- World-posture test: the same touch can be supported in preview and denied by
  wrong-world posture in authoritative mode without changing the family
  identity.

**Engineering decisions**

- Support posture and admission posture are distinct retained artifacts.
- Support posture is family/world/runtime-profile truth; admission posture is
  touch-local legality truth.
- Admission reports summarize selected obligation work; they do not replace
  underlying verdict artifacts.
- Admission report aggregation must be explicit, not emergent from presentation
  code. The initial aggregation law is:
  - any blocking violation => denied
  - unsupported required family => unsupported
  - wrong-world required family => wrong-world
  - all satisfied + advisories => admitted-with-advisory
  - all satisfied => admitted
  - diagnostic-only selected families do not make ordinary work admitted
  - deferred required family => deferred
- Diagnostic projection is downstream of typed posture, not the authority lane.

**Open questions**

- None.

### Phase 2: Define A Typed Graph Touch Descriptor With Origin, World, And Aspect Precision

Phase 2 defines the touched-meaning artifact the caller may provide and the
runtime may trust.

**Relevant subsystems**

- graph touch lane
- declaration/graph handoff consumption
- aspect contract consumption
- world classification lane
- certification touch-precision suites

**Relevant APIs**

- `UiGraphTouchDescriptor`
- touch origin classification surface
- touch world surface
- touch timing surface
- touch aspect posture surface
- touch target classification surface

**Warnings**

- Do not make the ordinary touch shape "node changed" or "rerender this
  subtree."
- Do not omit touch origin. Query-backed fact change, declaration edit, and
  host observation are not interchangeable.
- Do not let aspect posture degrade into a generic changed-flag bag.
- Do not infer world from ambient runtime state after the touch descriptor is
  constructed.

**Test requirements**

- Equivalence test: semantically equivalent touches with the same target,
  origin, world, and aspect posture produce equivalent touch authority even if
  caller-local construction details differ.
- Coarsening rejection test: a descriptor that omits required origin, world,
  target, or aspect-local meaning is denied before obligation selection.
- Localization test: an appearance-only touch does not classify as structural,
  Query-binding, or intent-operability work unless a declared aspect contract
  says it must.

**Engineering decisions**

- Touch descriptors are sealed artifacts, not open bags callers may widen with
  arbitrary string metadata.
- Target identity is graph-owned and typed; source-span or display labels are
  not substitute identity lanes.
- Touch origin is part of selection truth, not optional diagnostics context.
- Touch timing is part of selection truth, not an implementation detail of the
  caller.
- Aspect posture must distinguish read, written, invalidated, and preserved
  slices so later rebind and explanation work inherit honest breadth.
- Touches originate from admitted upstream receipts, not raw user guesswork or
  ambient runtime intent.

**Open questions**

- None.

### Phase 3: Make World-Aware Support And Admission Typed Runtime Artifacts

Phase 3 gives touch selection and legality one typed world-aware posture model
instead of scattered gate checks.

**Relevant subsystems**

- support snapshot lane
- world support posture lane
- admission posture lane
- denial/advisory topology
- certification posture suites

**Relevant APIs**

- support row / world support row projection
- typed unsupported/deferred/diagnostic-only posture
- typed wrong-world/wrong-query-basis/wrong-host-capability posture
- `UiAdmissionReport`

**Warnings**

- Do not flatten support/admission into a single enum whose variants mix
  readiness and verdict outcome.
- Do not encode world-sensitive posture only in log text or diagnostic strings.
- Do not treat diagnostic-only posture as ordinary support.

**Test requirements**

- Matrix test: the support snapshot can distinguish `supported`,
  `unsupported`, `deferred`, and `diagnostic-only` per family and per world.
- Denial test: the same supported family can deny a touch through wrong-world,
  wrong-host-capability, or wrong-Query-basis posture without being reported as
  unsupported.
- Reconstruction test: an admission report is fully interpretable from typed
  posture artifacts without requiring string parsing or host memory.

**Engineering decisions**

- Support rows carry current-world and family-local posture, not user-facing
  prose as authority.
- Admission reports carry typed verdict neighborhoods and posture references.
- Wrong-world and unsupported are distinct ordinary-path outcomes.
- Query-basis and host-capability posture become first-class denial families
  instead of hidden implementation branches.

**Open questions**

- None.

### Phase 4: Admit Obligation Families Through A Closed Selection Matrix

Phase 4 closes the family taxonomy and makes selection derive from admitted
family rules instead of ad hoc validators.

**Relevant subsystems**

- obligation family catalog
- family support posture lane
- family selection matrix
- certification family-boundary suites

**Relevant APIs**

- obligation family identity surface
- `UiObligationCheckKind`
- family support posture projection
- family-local selection rules
- `UiSelectedObligationSet`
- `UiSelectedObligationIdentity`
- `UiObligationSelectionReason`

**Warnings**

- Do not create a catch-all "validation" family.
- Do not let one broad family silently own structure, participation,
  measurement, Query, intent, and host legality at once.
- Do not let family selection depend on caller-remembered lists.

**Test requirements**

- Catalog parity test: every roadmap family appears exactly once in the admitted
  family catalog.
- Rejection test: unknown or contradictory obligation-family claims fail
  through family admission rather than late dispatch.
- Drift test: adding a new family or expanding one familyâ€™s support posture
  forces explicit compiler and certification updates at every selection site.

**Engineering decisions**

- The family catalog is closed and typed.
- Family-local support posture is separate from touch-local admission verdicts.
- Selection rules are data the runtime owns, not helper conditionals callers
  replicate.
- Selected obligations carry stable selected-obligation identity and typed
  selection reasons.
- Check kind is separate from family identity so blocking invariants,
  prerequisite checks, capability-gap screens, world gates, advisories,
  diagnostic-only checks, and deferred backstops do not collapse into family
  names.
- Future families may exist as support-row posture before ordinary support
  closes, but they may not disappear from the model.

**Open questions**

- None.

### Phase 5: Lower Selected Obligations Into Dispatch Plans And Typed Verdicts

Phase 5 turns family selection into a proof-bearing dispatch artifact and a
typed verdict lane.

**Relevant subsystems**

- obligation selection lane
- dispatch planning lane
- verdict lane
- admission-report construction lane
- certification dispatch/verdict suites

**Relevant APIs**

- `UiSelectedObligationSet`
- `UiObligationDispatchPlan`
- `UiObligationVerdict`
- verdict class surface / stop posture surface
- selected-obligation result envelope

**Warnings**

- Do not dispatch directly from touch descriptors without a selected-obligation
  artifact.
- Do not use binary pass/fail verdicts for the ordinary lane.
- Do not force unsupported, wrong-world, or wrong-host-capability posture to
  masquerade as advisory or violation.
- Do not let dispatch execution rediscover which families were eligible.
- Do not let verdicts carry only display text.

**Test requirements**

- Progression test: the only ordinary path to execution is
  touch -> selected obligations -> dispatch plan -> verdicts.
- Tri-state test: success, advisory, and violation remain typed and distinct
  through verdict and admission-report construction.
- Boundary test: dispatch cannot widen or swap obligation families that were
  not present in the selected-obligation set.
- Localization test: a family-local advisory does not get flattened into a
  global failure when the admission report still needs structured context.
- Determinism test: same touch + same graph generation + same support snapshot
  produces the same selected obligations, same dispatch plan shape, and same
  verdict classes.

**Engineering decisions**

- `UiSelectedObligationSet` is the proof-bearing execution input for this
  milestone.
- `UiObligationDispatchPlan` carries only admitted selected work; it is not a
  second selection engine.
- `UiObligationVerdict` owns machine-readable result class or equivalent stop
  posture, structured context, and evidence refs.
- If verdict class is kept small, each verdict must also carry a separate
  typed stop posture strong enough to distinguish unsupported, deferred,
  diagnostic-only, wrong-world, wrong-Query-basis, wrong-host-capability,
  stale, ambiguous, and budget-exceeded from ordinary advisory/violation
  results.
- Admission reports summarize verdict neighborhoods; they do not replace them.

**Open questions**

- None.

### Phase 6: Consume Query, Host, Measurement, And Service Prerequisites Without Rebuilding Their Runtimes

Phase 6 integrates external prerequisite lanes while preserving owner
boundaries.

**Relevant subsystems**

- Query-binding prerequisite lane
- host-capability prerequisite lane
- measurement/service requirement posture lane
- obligation family support posture
- certification boundary suites

**Relevant APIs**

- `ResolvedSnapshotBasis`
- `SnapshotResolutionReport`
- `consume_projection_facts(...)`
- `workspace.inspect(...)`
- `admit_causal_inspection`
- `request_causal_inspection`
- host capability report surface

**Warnings**

- Do not rebuild Query support/admission with UI-local booleans or labels.
- Do not inspect materialization rows or local caches when projection
  consumption owns the public lane.
- Do not let host adapters decide that a touch is legal because the adapter
  happened to support a mechanic.
- Do not treat measurement requirements as measured facts or layout execution.

**Test requirements**

- Query-boundary test: Query-backed obligation requirements consume Query-owned
  basis and projection artifacts rather than local cache or reopened source.
- Host-boundary test: host capability denials arise from typed capability
  posture, not renderer-local heuristics.
- Separation test: measurement and service requirement families remain intent
  or prerequisite contracts only; they do not masquerade as executed receipts.

**Engineering decisions**

- Query-owned basis, projection, inspection, and causal explanation stay on the
  Query lane.
- Worth UI obligations may require or cite those artifacts, not replace them.
- Host capability posture is explicit and typed before execution.
- Measurement, portal, focus, motion, and service-related families may appear
  as support posture before later execution slices close fully.
- Portal/focus/motion families should default toward
  `architecturally_owned_but_not_yet_admitted`, `diagnostic_only`, or
  `unsupported` posture until their service milestones land, rather than
  encouraging half-service implementations inside 3.4.

**Open questions**

- None.

### Phase 7: Emit Evidence, Diagnostics, And Inspection Hooks For Obligation Work

Phase 7 makes the new runtime family explainable on day one.

**Relevant subsystems**

- obligation evidence lane
- admission evidence lane
- diagnostics lane
- inspection query/receipt lane
- certification evidence suites

**Relevant APIs**

- obligation evidence ref / evidence handle surface
- admission evidence ref / evidence handle surface
- selection-reason evidence projection
- obligation diagnostic artifact
- admission diagnostic artifact
- inspection-ready projection surface

**Warnings**

- Do not make logs the public evidence surface.
- Do not require later inspection to reconstruct selected obligations from
  dispatch code or host behavior.
- Do not flatten structured denial/advisory posture into one string because a
  later UI might render it.

**Test requirements**

- Evidence retention test: every selected obligation and every verdict can be
  inspected through typed retained evidence rather than logs.
- Relevance test: later inspection can scope evidence by graph node, touch,
  family, or denial posture without broad dumps.
- Diagnostic parity test: user-facing diagnostic projections remain derivable
  from the same evidence the AI/human inspection lanes consume.
- Reason test: later inspection can answer why an obligation was selected or
  not selected from retained selection-reason evidence instead of logs.

**Engineering decisions**

- Evidence is an ordinary runtime artifact, not a debug optional.
- Admission diagnostics project retained evidence; they do not author new truth.
- 3.4 must leave later inspection slices with typed identity-bearing handles,
  not giant materialized report blobs as the only path.
- Support posture, selected obligations, verdicts, and diagnostics remain
  causally linked by typed evidence refs.

**Open questions**

- None.

### Phase 8: Mechanize Boundary Enforcement And Close A Proof-Bearing Handoff To Later Runtime Slices

Phase 8 turns the milestone into compiler and certification proof and publishes
the exact handoff later runtime families may consume.

**Relevant subsystems**

- compile-fail boundary suites
- dependency/residue audits
- public facade export review
- closeout handoff lane
- certification closeout audit

**Relevant APIs**

- compile-fail boundary suites for touch/support/verdict construction
- topology/dependency audit
- curated public admission/obligation facade surface
- `UiAdmissionCloseoutReport` or equivalent closeout artifact

**Warnings**

- Do not rely on review comments to prevent caller-selected validators or
  renderer-local legality helpers.
- Do not mirror internal topology through public exports.
- Do not hand later runtime slices raw graph/declaration artifacts and call that
  the obligation handoff.

**Test requirements**

- Compile-fail test: public callers cannot mint touch descriptors,
  selected-obligation sets, dispatch plans, verdict wrappers, or denial
  witnesses from raw values.
- Residue test: production code rejects local validator packs, renderer-local
  legality helpers, source reopening, and host-owned admission shortcuts.
- Handoff test: later runtime slices can consume touch/admission/verdict
  authority through sealed artifacts without reopening graph or declaration
  semantics.
- Coverage test: closeout proof enumerates the touch, support, family,
  selection, dispatch, verdict, Query-boundary, host-boundary, and evidence
  lanes actually closed in 3.4.
- Determinism proof: equivalent touch descriptors converge to equivalent
  admission reports under the same graph generation, support snapshot, and
  prerequisite evidence set.

**Engineering decisions**

- Certification owns the anti-cheating proof for 3.4.
- Public surfaces stay narrow and capability-shaped.
- The handoff to later runtime slices is selected-obligation and
  admission-evidence authority, not ad hoc helper access.
- 3.4 closes at obligation selection and legality evidence only.

**Open questions**

- None.

## Must Ship

- `UiGraphTouchDescriptor`
- typed touch origin classification
- typed operating-world classification for touch/admission work
- typed touch timing classification
- typed touch aspect posture for read/write/invalidate/preserve slices
- admitted-source-only touch construction law
- `UiSupportSnapshot` and world-aware support posture for obligation families
- `UiSelectedObligationSet`
- `UiSelectedObligationIdentity`
- `UiObligationCheckKind`
- `UiObligationSelectionReason`
- `UiObligationDispatchPlan`
- `UiObligationVerdict`
- typed verdict class or equivalent typed obligation stop posture strong enough
  to distinguish satisfied, advisory, violation, unsupported, deferred,
  diagnostic-only, wrong-world, wrong-Query-basis, wrong-host-capability,
  stale, ambiguous, and budget-exceeded posture
- `UiAdmissionReport`
- explicit admission report aggregation lattice
- typed denial/advisory/violation posture for ordinary admission
- `UiObligationSelectionBudget`
- `UiObligationSelectionCostReceipt`
- `UiObligationDispatchBudget`
- `UiObligationBudgetVerdict`
- closed initial obligation family catalog for:
  - `structural-legality`
  - `participation-legality`
  - `slot-contract`
  - `measurement-requirement`
  - `query-binding-requirement`
  - `intent-operability-requirement`
  - `portal-host-requirement`
  - `focus-route-requirement`
  - `motion-support-requirement`
  - `accessibility-requirement`
  - `host-capability-requirement`
  - `diagnostic-surface-requirement`
- first closed selection matrix table for representative touch classes
- Query-boundary requirement integration through Query-owned public artifacts
- host-capability requirement integration through typed host-contract posture
- retained evidence handles and diagnostic artifacts for selected obligations,
  verdicts, and admission reports
- certification proof that callers declare touched meaning but do not select
  validators manually
- certification proof that ordinary selection avoids broad graph scans and
  source reopening
- certification proof that support and admission remain distinct typed runtime
  artifacts
- deterministic selection proof for same touch + same graph generation + same
  support snapshot
- deterministic admission-report proof for equivalent touch descriptors under
  equivalent prerequisite evidence

## Must Preserve

- Milestone 3.1's single public facade discipline
- Milestone 3.1's inspection-authority boundary
- Milestone 3.2's declaration authority and aspect contracts
- Milestone 3.3's graph authority, participation truth, and bounded indexes
- strict separation between support posture and touch-local admission posture
- strict separation between requirement selection and later execution families
- strict separation between obligation family identity and obligation check kind
- Query-owned basis, projection-consumption, inspection, and causal explanation
  ownership
- host neutrality through `worth-ui-host-contract`
- typed outcomes instead of booleans or string-only failures
- touch construction from real upstream authority instead of caller-local
  intuition
- bounded, touch-scoped selection instead of broad post-mutation graph walks
- evidence-first diagnostics instead of log-first explanation

## Acceptance Evidence

3.4 is complete only when all of these are true:

- callers declare touched meaning through a typed `UiGraphTouchDescriptor`
  rather than selecting validators manually
- touch descriptors carry explicit origin, world, target, and aspect-local
  meaning precise enough to avoid catch-all "node changed" selection
- touch descriptors also carry explicit timing and originate only from admitted
  upstream receipt families
- support posture and admission posture remain separate runtime-owned artifacts
- the same family can be supported yet still deny a specific touch through typed
  legality posture
- world-sensitive differences are explicit at the support/admission boundary
  rather than ambient runtime drift
- selected obligations derive from typed touch meaning plus graph/index
  authority and support posture rather than broad scans
- each selected obligation has stable identity, typed check kind, and typed
  selection-reason evidence
- selected obligations lower to typed dispatch plans and typed verdicts
- verdicts preserve at least success, advisory, and violation posture with
  machine-readable context
- unsupported, deferred, wrong-world, wrong-Query-basis, wrong-host-capability,
  stale, ambiguous, and budget-exceeded posture remain structurally distinct
  from ordinary advisory/violation outcomes
- dispatch executes only obligation-checking work for 3.4 and does not
  prematurely execute measurement, Query, intent, or service runtime families
- Query-backed requirement families cite Query-owned basis/projection/inspection
  artifacts rather than UI-local substitutes
- host-capability requirement families cite typed host posture rather than
  renderer-local legality
- selection and dispatch expose typed budget/cost receipts with named counters
  for breadth honesty
- a starter closed selection matrix makes the ordinary path mechanically visible
- later inspection can explain what obligations were selected and why without
  logs or source reopening
- compile-fail, runtime, residue, and topology suites prove public callers
  cannot bypass the admission boundary or reintroduce validator folklore

## Allowed Debt

3.4 may defer richer execution of later measurement, service, mounted-receipt,
or human-inspector families when the ordinary obligation-selection and typed
admission path already exists and the deferred work is mechanically contained.

Any allowed debt must satisfy `MENTALITY.md`: it must be named, major enough to
justify deferral, bounded so it cannot be mistaken for the ordinary lane, and
attached to an explicit follow-on milestone.

3.4 may not mark these as debt:

- `UiGraphTouchDescriptor`
- explicit touch origin/world/aspect precision
- explicit touch timing precision
- admitted-source-only touch construction
- separation between support posture and admission posture
- closed obligation-family catalog
- `UiSelectedObligationSet`
- `UiSelectedObligationIdentity`
- `UiObligationCheckKind`
- `UiObligationSelectionReason`
- `UiObligationDispatchPlan`
- `UiObligationVerdict`
- `UiAdmissionReport`
- typed verdict/stop posture distinction between unsupported, wrong-world,
  wrong-Query-basis, wrong-host-capability, and ordinary legality outcomes
- explicit admission aggregation lattice
- obligation-selection and obligation-dispatch budget/cost receipts
- world-aware support/admission posture
- typed wrong-world, wrong-Query-basis, and wrong-host-capability posture
- Query-owned prerequisite consumption for Query-backed requirement families
- typed host-capability prerequisite posture
- retained evidence and inspection-ready handles for selected obligations and
  verdicts
- residue rejection for caller-local validator packs and broad ordinary-path
  scans
- sealed handoff to later runtime slices

Unnamed or cosmetic "we can flesh this out later" debt is not allowed.

## Sequencing Notes

3.4 belongs immediately after 3.3 because graph truth is not enough. The
runtime still needs one authority for deciding which legality/support checks
follow from touched meaning.

3.4 belongs before:

- later inspection-evidence slices, because those slices need selected
  obligations, verdicts, and admission reports as real evidence families
- measurement/allocation slices, because measurement requirements must already
  be selected from touched meaning instead of from renderer-local code
- Query-binding/runtime-state slices, because Query-bound work must already be
  admitted through a typed prerequisite boundary
- intent and service slices, because operability, portal, focus, and service
  legality must inherit the same selected-obligation contract
- mounted receipt and rebind slices, because they need typed obligation and
  denial evidence to explain what changed and what was blocked

This sequencing is what keeps later runtime work from rebuilding its own local
admission systems.

## Required Self Check

Before closeout, answer these with evidence:

- Does 3.4 make touched-work selection runtime-owned authority rather than a
  validator convention?
- Can the runtime name why each obligation was selected from typed touch facts
  and support posture?
- Can the runtime distinguish family identity, check kind, verdict class, and
  support/admission posture without reconstructing those semantics from names or
  strings?
- Can the same family be supported while a specific touch is still denied
  through typed legality posture?
- Are Query-backed and host-backed prerequisites consumed through their owning
  public lanes rather than rebuilt locally?
- Can later inspection explain selection and denial without logs or source
  reopening?

Reopen 3.4 if any of these become true:

- callers must remember which validators or checks to run
- touch descriptors collapse into generic changed-node or rerender surfaces
- touch descriptors can be minted from caller-local guesswork rather than real
  upstream authority artifacts
- support and admission collapse into one vague status model
- verdict classes or stop posture collapse unsupported/deferred/wrong-world into
  advisory or violation
- selected obligations widen by hidden broad graph scans
- Query-backed denials are reconstructed from local cache or materialization-row
  access
- host adapters decide legality or support
- dispatch starts performing measurement, Query execution, or half-service work
  instead of checking admitted prerequisites
- verdicts flatten to pass/fail or text-only errors
- inspection must reconstruct selected obligations from logs or host behavior
- public exports mirror internal obligation topology deeply enough that
  refactors would become breaking changes
