# Milestone 3.5 Engineering Spec: Inspection Evidence Expansion And Relevance Indexes

> **Status:** Planned
>
> **Roadmap parent:** [worth_ui_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/worth-ui/plans/worth-ui/worth_ui_roadmap.md)
>
> **Primary prerequisite:** `Milestone 3.4 Admission, Support, And Graph Touch Obligations`
>
> **Follow-on sequence:** `Milestone 3.6a Measurement Vocabulary, Basis Admission, And Host Evidence Boundaries`
>
> **Primary architectural driver:** make runtime truth inspectable through typed,
> indexed, relevance-scoped evidence rather than debug dumps, renderer-local
> helpers, or ad hoc explanation folklore.

## Goal

Make Worth UI inspection runtime-owned, typed, relevance-scoped, and
incrementally explorable.

Milestone 3.5 is complete when callers can target runtime truth by declaration
identity, source span, graph identity, aspect neighborhood, and obligation
scope; receive narrow, typed evidence references before asking for rich detail;
and expand only the evidence families relevant to the inspection question
without broad scans, renderer-local reconstruction, or giant mixed dumps.

This milestone closes one explanation boundary:

- what counts as inspection evidence for declaration, admission, graph, aspect,
  and obligation truth
- how runtime truth becomes stable evidence references
- how relevance is expressed as typed inspection narrowing rather than fuzzy
  filtering
- how indexes bind authored identity, graph identity, and aspect participation
  to evidence neighborhoods
- how evidence slices are derived from runtime authority without becoming a
  second truth system

It does not close measurement evidence, mounted receipts, screenshots,
hit-testing, replay, or human inspector UI.

## Non-Goals

Milestone 3.5 does not implement:

- measurement or allocation evidence
- mounted receipt evidence
- visual snapshot evidence
- hit-test evidence
- replay timeline or replay-step evidence
- host-observation explanation beyond preserving the substrate seams
- human inspector panels
- renderer overlays
- Query authority, Query causal inspection authority, or Query basis authority
- broad cross-runtime causal storytelling

3.5 closes evidence substrate and targeted inspection only:

- typed evidence-family ownership for declaration, admission, graph, aspect,
  and obligation truth
- typed relevance routing and inspection narrowing
- stable evidence references before rich materialization
- derived evidence indexes over authored and runtime identity
- narrow evidence-slice assembly
- the first real AI-facing inspection path on runtime-owned evidence

## Why This Milestone Exists

Milestone 3.1 created the inspection facade and support vocabulary but kept the
surface intentionally thin. Milestone 3.2 established canonical declaration
authority. Milestone 3.3 established graph truth and core indexes. Milestone
3.4 established admission, support, graph-touch classification, selected
obligations, and typed verdicts.

3.5 is where those runtime-owned truths first become a serious inspection
substrate instead of a promise. Without this slice, every future AI workflow,
human inspector, replay surface, and diagnostics panel would be forced to
reconstruct explanation from logs, broad snapshots, or renderer-local helper
state.

This milestone is not "add diagnostics." It is the first proof-bearing evidence
layer for the runtime:

- later measurement and allocation work needs an evidence substrate rather than
  a one-off layout explainer
- later mounted receipt and screenshot work needs stable identity-backed
  evidence refs rather than frame-local debug state
- later human inspector work needs a relevance-scoped query surface rather than
  giant graph dumps
- later AI tooling needs typed, narrow, repeatable inspection queries rather
  than prompt-specific heuristics

If 3.5 is vague, Worth UI will grow a second runtime in the tooling layer. That
would permanently weaken the platform.

## Governing Summaries

- `MENTALITY.md`
  protects adversarial-constraint-first design. 3.5 must start from large,
  churn-heavy inspection pressure where callers repeatedly ask narrow semantic
  questions across many identities, not from a single pretty inspector demo.
- `arch_laws.md`
  protects authority versus derivation, proof-bearing phase progression, and
  narrow facades. 3.5 must keep runtime truth authoritative and make evidence
  slices, indexes, and materialized detail derived from that authority.
- `composition_laws.md`
  protects named semantic steps. 3.5 must not collapse relevance admission,
  reference production, index lookup, slice assembly, and rich detail
  materialization into one giant inspection helper.
- `domain_structure_laws.md`
  protects structural separability. Evidence families, relevance routing,
  indexes, slice assembly, and AI harness code must live in distinct homes
  because they broaden, fail, and scale differently.
- `perf_laws.md`
  protects semantic-breadth honesty. 3.5 must answer local inspection questions
  through typed indexes and relevance routing rather than broad graph scans or
  mixed evidence dumps.
- `worth_ui_roadmap.md`
  protects the milestone's place in sequence. 3.5 must ship `UiEvidenceSlice`,
  `UiRelevanceFilter`, typed evidence families, stable indexes over
  declaration/admission/graph/aspect/obligation truth, and the first formal AI
  harness path before screenshots exist.
- `WORTH_UI_README.md`
  protects the actual runtime stack and artifact vocabulary. 3.5 must extend
  the existing inspection surface instead of inventing a parallel explainer:
  `UiInspectionQuery`, `UiInspectionTarget`, `UiInspectionScope`,
  `UiInspectionRelevance`, `UiEvidenceBudget`, `UiEvidenceRichness`,
  `UiInspectionReceipt`, `UiInspectionSupportReport`, and
  `UiInspectionClosureReport` are the substrate-adjacent ordinary lanes.
- `ai-diagnostics.md`
  protects one shared evidence substrate for AI and human diagnostics. 3.5 must
  make evidence family identity, provenance, causal links, relevance filtering,
  and lazy materialization real instead of collapsing back to logs or dumps.
- `crates/worth-query/docs/AI_README.md`
  protects Query-owned inspection and projection lanes. 3.5 must keep
  per-target retained Query evidence on `workspace.inspect(...)`, keep
  cross-runtime causal explanation on `admit_causal_inspection` /
  `request_causal_inspection`, and keep projection-backed domain facts on
  `consume_projection_facts(...)` plus Query basis artifacts such as
  `ResolvedSnapshotBasis` and `SnapshotResolutionReport`.

## Adversarial Constraint

3.5 must survive this hostile condition:

> Worth UI runs as a large, hot-rebinding desktop runtime where declaration
> edits, Query-backed data changes, host observations, and graph mutations keep
> changing the active truth. AI and human-facing tooling repeatedly ask narrow
> questions such as "why is this declaration denied?", "what obligations touch
> this node?", "which published aspect caused this dependent work?", and "show
> me only the evidence refs for this source span." Across that churn, the
> runtime must answer from typed runtime-owned evidence and indexes, preserve
> stable identity, reveal only the requested neighborhood, and avoid broad
> rescans, renderer-local explanation, log reconstruction, or second-graph
> folklore.

If evidence must be dumped wholesale to be useful, if relevance is fuzzy and
untyped, if references are unstable across equivalent inspection requests, if
the inspection layer recomputes truth independently, or if adding a new runtime
family later would require inventing a separate inspection architecture, 3.5 is
not closed.

## Product Decision Lock

- Inspection evidence is derived from runtime authority; it is not a second
  authority graph.
- `UiEvidenceSlice` is a projection artifact, not a writable runtime truth
  carrier.
- `UiRelevanceFilter` is typed and closed. Relevance narrowing is not keyword
  search, fuzzy ranking, or free-form log filtering.
- Evidence retrieval is two-stage by default:
  `evidence_refs` first, `materialized_detail` only when requested.
- Evidence families are explicit and additive. New runtime families extend the
  evidence substrate through typed family surfaces; they do not widen a generic
  blob.
- `UiEvidenceIdentity`, `UiEvidenceRef`, `UiEvidenceHandle`, and
  `UiEvidenceSlice` are distinct concepts. The names may vary, but the semantic
  split may not:
  - evidence identity = canonical identity of an evidence record
  - evidence ref = externally projectable, non-WORTHable handle
  - evidence handle = internal runtime handle where needed
  - evidence slice = projection containing refs, summaries, and optional detail
- Indexes are runtime-owned derived structures over declaration, admission,
  graph, aspect, and obligation truth. They may be destroyed and rebuilt from
  authority.
- Human inspector surfaces and AI tooling are consumers of the same evidence
  substrate. They must not each author separate explanation logic.
- Host adapters and renderers may expose observations and receipts when later
  milestones add them, but they do not explain meaning.
- Evidence projection must preserve stable identity handles and neighborhood
  boundaries even when rich detail remains deferred.
- Query-owned basis, projection-consumption, retained inspection, and causal
  explanation lanes remain Query-owned. Worth UI may attach UI evidence to
  those lanes through `worth-ui-query-binding`; it may not restate them as
  UI-local pseudo Query reports.

## Inspection Authority Progression

3.5 must preserve explicit phase progression:

`UiDeclarationArtifact / UiAdmissionReport / UiGraphSnapshot / UiSelectedObligationSet / UiObligationVerdict -> family-local evidence records -> stable evidence refs -> indexed evidence neighborhoods -> UiEvidenceSlice -> UiInspectionReceipt`

The exact type names may vary, but the progression law may not:

- runtime authority is not itself an evidence slice
- evidence refs are not materialized detail
- indexes are not authority
- `UiEvidenceSlice` is not a writable store
- `UiInspectionReceipt` is the public retained inspection outcome, not a debug
  log bundle

If production code can jump from raw runtime structures directly into ad hoc
debug strings, renderer overlays, or whole-graph dumps without family-local
evidence, reference production, and slice assembly, the progression has
collapsed.

## Evidence Reference Lifecycle Contract

3.5 must make evidence-reference stability precise rather than rhetorical.

`UiEvidenceRef` must carry or bind to at least:

- evidence family
- evidence identity
- owning authority artifact identity
- authority generation or snapshot identity
- world profile where relevant
- materialization posture

Evidence refs are stable handles to evidence over a specific authority
generation.

The generation law is:

- equivalent inspection queries over the same authority generation must converge
  on equivalent evidence refs
- later authority generations may produce corresponding refs, but correspondence
  must be explicit rather than assumed
- replayed or historical generations may expose equivalent or corresponding
  refs, but those refs remain generation-bound
- discarded evidence may invalidate expansion while preserving tombstone-grade
  reference identity where policy requires it

3.5 must distinguish at minimum:

- same authority generation
- new graph or runtime authority generation
- same declaration identity after equivalent lowering
- replayed historical generation
- discarded or garbage-collected evidence

If "stable ref" means only "works right now in this process," the milestone is
not closed.

## Evidence Retention And Expansion Contract

3.5 is the evidence substrate, so it must declare a minimal retention posture.

It must admit a typed retention policy such as:

- `current_generation_only`
- `retained_for_inspection`
- `retained_for_replay`
- `retained_until_closeout`
- `discarded_with_tombstone`

It must also admit a typed expansion posture such as:

- `available`
- `unavailable_discarded`
- `unavailable_wrong_generation`
- `unavailable_not_materialized`
- `unsupported`

The exact enum names may vary, but the semantics may not. AI and human tools
must have a principled answer when holding an old evidence ref after the
underlying retained detail has changed, moved generations, or been discarded.

## Evidence Family Contract

3.5 must make evidence-family shape explicit.

Each 3.5-covered family must provide:

- stable evidence identity
- provenance to the owning runtime authority artifact
- typed semantic category
- typed links to upstream or downstream related evidence
- family-local relevance participation
- typed retention posture
- reference-only posture
- materialized-detail posture
- generation binding

The first admitted family set is:

- declaration evidence
  declaration identity, authored source provenance, declaration family, aspect
  contract summaries, and declaration-to-graph correspondence posture
- admission evidence
  support posture, admission posture, denial posture, world posture, host or
  Query gating posture, and admission decision neighborhoods
- graph evidence
  graph node identity, declaration correspondence, participation posture,
  parent/slot/page/region/mosaic neighborhood, and generation-local graph
  evidence
- aspect evidence
  published aspect posture, consumed aspect posture, publisher/dependent
  neighborhoods, and aspect-local receipt references
- obligation evidence
  touch-derived obligation selection, selection reasons, dispatch-plan posture,
  verdict posture, and retained obligation evidence handles

The family law is strict:

- family-local evidence may project or summarize authority
- family-local evidence may not reopen or reinterpret the authority decision
- family-local evidence may not substitute for future families such as
  measurement, mounting, snapshots, or replay

## Evidence Link Contract

3.5 must make evidence-link vocabulary explicit enough that tools can walk the
evidence neighborhood without turning links into a second authority graph.

It must admit typed link kinds such as:

- `derived_from`
- `summarizes`
- `explains`
- `caused_by`
- `selected_by`
- `invalidated_by`
- `attached_to`
- `corresponds_to`
- `blocked_by`
- `cites_foreign_evidence`

The exact names may vary, but the law may not:

- evidence links are explanation or provenance edges
- evidence links are not authority edges
- following a link must never promote lower-authority explanation into runtime
  truth

## Relevance And Inspection Query Contract

3.5 must make the inspection request shape operationally precise.

It must extend the existing public inspection lane rather than replace it:

- `UiInspectionQuery`
- `UiInspectionTarget`
- `UiInspectionScope`
- `UiInspectionRelevance`
- `UiEvidenceBudget`
- `UiEvidenceRichness`
- `UiInspectionReceipt`

`UiRelevanceFilter` must either be the product-facing narrowing surface or the
runtime-owned narrowing artifact underneath `UiInspectionRelevance`, but the
relationship must be explicit and singular. 3.5 may not leave two competing
relevance vocabularies alive.

The narrowing contract must distinguish at least:

- target class
  - product root
  - declared surface
  - graph node
  - source span
  - obligation graph node
  - obligation touch
  - obligation evidence handle
- scope
  - graph
  - measurement
  - mounting
  - rebind
- relevance neighborhood
  - declaration-local
  - admission-local
  - graph-local
  - aspect-local
  - obligation-local
  - cross-family neighborhood
- richness posture
  - refs only
  - summary
  - materialized detail
- budget posture
  - narrow/local
  - ordinary
  - expanded

The exact enum names may evolve, but 3.5 may not leave those distinctions
implicit in helper methods or prompt instructions.

3.5 must also make "nothing found" semantically precise. It must admit a typed
relevance outcome such as:

- `matched`
- `empty_local`
- `unsupported_scope`
- `contradictory_request`
- `budget_exceeded`
- `not_applicable_to_target`

Those outcomes must stay distinct because these are different conditions:

- no obligation evidence exists for this node
- this target class cannot have obligation evidence
- this scope exists architecturally but is not admitted yet
- the request is contradictory
- the request would exceed the allowed expansion budget

3.5 must not collapse those conditions into a generic empty result.

It must also constrain cross-family expansion:

- local family request:
  may include only refs from the requested family plus direct provenance refs
- cross-family neighborhood request:
  must name allowed families or allowed link kinds
- causal trace request:
  remains deferred unless a later causal or replay milestone explicitly admits
  it

This prevents "cross-family neighborhood" from becoming "inspect everything
related."

## Inspection Receipt And Slice Contract

3.5 must make the minimal ordinary return shape explicit enough that
implementation cannot drift into ad hoc receipts.

`UiInspectionReceipt` must contain or bind to at least:

- query identity
- target
- scope
- relevance
- richness
- budget
- authority generation
- outcome
- evidence slice identity or reference
- cost receipt

`UiEvidenceSlice` must contain or bind to at least:

- slice identity
- authority generation
- families included
- evidence refs
- family summaries
- materialized detail
- omitted_by_budget
- omitted_by_scope

The exact field names may vary, but those semantic categories may not.

## Inspection Cost Contract

3.5 must make inspection cost visible as a typed receipt, not a debugging
guess.

It must admit an inspection cost receipt such as:

- `index_lookups`
- `evidence_refs_considered`
- `evidence_refs_returned`
- `materialized_records`
- `omitted_by_budget`
- `traversals_denied`
- `broad_scan_used`

The ordinary-path law is strict:

- `broad_scan_used` must be `false` on the ordinary 3.5-covered path
- if a request cannot be answered honestly inside the budgeted indexed path,
  the runtime must return a typed narrowed outcome rather than silently scanning

## Evidence Ordering Contract

Equivalent evidence slices must return refs in deterministic canonical order.

The ordering basis must include at least:

- family
- authority generation
- target identity
- evidence identity
- link kind where relevant

The exact comparator structure may vary, but "equivalent requests converge"
must not depend on incidental hash-map or traversal order.

## Query Integration Contract

3.5 must be explicit about what Worth UI may consume from Query and what it may
not redefine.

Worth UI inspection may attach UI evidence to Query-owned artifacts, but these
lanes stay Query-owned:

- snapshot-basis and world posture:
  `ResolvedSnapshotBasis`, `SnapshotResolutionReport`
- projection-backed domain facts:
  `WORTHQueryReadResult::consume_projection_facts(...)`,
  `WORTHQueryWriteReceipt::consume_projection_facts(...)`,
  `QueryContextExecutionArtifact::consume_projection_facts(...)`
- retained per-target Query evidence:
  `workspace.inspect(...)`
- cross-runtime causal explanation:
  `admit_causal_inspection`, `request_causal_inspection`

3.5 must therefore admit a formal foreign-evidence reference surface for Query
citation, such as:

- owner
- artifact kind
- artifact identity
- inspection route

Worth UI may cite Query-owned evidence through that typed foreign-evidence ref,
but it may not materialize Query-owned explanation as UI-owned truth.

3.5 therefore must not:

- treat `workspace.inspect(...)` as the Worth UI evidence substrate
- use `admit_causal_inspection` or `request_causal_inspection` as a substitute
  for local UI evidence slices
- bypass projection-consumption receipts and recover Query facts from host
  caches, graph blobs, or local materialization rows
- collapse Query basis posture into UI-local booleans or strings

## Source Provenance Contract

Source-span lookup must bind to source artifact identity and source generation,
not to naked byte ranges.

It must admit a typed source provenance reference such as:

- source artifact identity
- source generation
- span

Without that binding, a post-edit source span can silently point at the wrong
meaning.

## Evidence Index Contract

3.5 must make index promises concrete.

Derived indexes must exist at minimum for:

- `UiDeclarationIdentity -> evidence refs`
- source provenance span -> declaration / admission evidence refs
- `UiGraphNodeIdentity -> declaration / admission / obligation evidence refs`
- published aspect identity -> publishing nodes / receipts / evidence refs
- consumed aspect identity -> dependent nodes / obligations / receipts /
  evidence refs

Index rules:

- indexes are derived and rebuildable from runtime authority plus retained
  family-local evidence
- evidence index updates must be transactionally aligned with the publication of
  the owning authority artifact that introduced the evidence neighborhood
- no ordinary inspection path may require recursive graph walks when one of the
  required indexes already names the lookup
- source-span lookup must stay on typed authored provenance, not fuzzy text
  search
- future families may add indexes, but they may not weaken the rebuild law

That transaction-alignment rule applies per family:

- declaration evidence aligns with declaration artifact publication
- admission evidence aligns with admission report publication
- graph evidence aligns with graph snapshot publication
- obligation evidence aligns with selected-obligation, dispatch-plan, verdict,
  and admission publication as appropriate

Family-local evidence may not be admitted on the ordinary path until its
ordinary reference lookup path is indexed or explicitly unsupported. "We will
scan it for now and index it later" is not an allowed ordinary path.

## Phase Plan

### Phase 1: Freeze Evidence Authority And Family Boundaries

This phase freezes what counts as inspection evidence, which crate owns each
family, and what the evidence substrate is allowed to derive versus own.

**Relevant subsystems**
- `worth-ui-inspection`
- `worth-ui-runtime`
- `worth-ui`

**Relevant APIs**
- `UiEvidenceSlice`
- `UiInspectionReceipt`
- `UiInspectionQuery`
- `UiInspectionTarget`
- evidence-family root types for declaration, admission, graph, aspect, and
  obligations
- the product inspection facade exported by `worth-ui`

**Warnings**
- Do not encode evidence as generic maps, strings, or mixed debug records.
- Do not let `worth-ui-inspection` restate runtime authority as inspector-owned
  truth.
- Do not let product-facing facades export internal family storage topology.

**Test requirements**
- Adversarial equivalence test: two independently assembled but authority-equal
  evidence neighborhoods for the same declaration identity must expose the same
  evidence-family membership and stable reference set.
- Adversarial rejection test: external callers must be unable to mint or widen
  evidence-family authority outside the owning runtime or inspection boundary.

**Engineering decisions**
- Define one closed family vocabulary for 3.5-covered evidence:
  declaration, admission, graph, aspect, and obligations.
- Separate authority facts from inspection projection facts at the type level.
- Keep public evidence-family surfaces narrow and facade-oriented.
- Require every family-local evidence record to name its provenance to one of:
  `UiDeclarationArtifact`, `UiAdmissionReport`, `UiGraphSnapshot`,
  `UiSelectedObligationSet`, or `UiObligationVerdict`.

**Open questions**
- None.

### Phase 2: Define Relevance Routing As Typed Inspection Admission

This phase makes relevance a real narrowing contract rather than an ad hoc
filtering convention.

**Relevant subsystems**
- `worth-ui-inspection`
- `worth-ui-runtime`

**Relevant APIs**
- `UiInspectionQuery`
- `UiInspectionRelevance`
- `UiRelevanceFilter`
- `UiInspectionScope`
- typed inspection target/scope APIs exposed through the inspection facade
- family-local relevance selectors

**Warnings**
- Do not make relevance a free-form predicate callback surface.
- Do not conflate "requested neighborhood" with "all evidence we can find."
- Do not let later AI prompt conventions become the only definition of
  relevance semantics.

**Test requirements**
- Adversarial convergence test: equivalent requests expressed through identity,
  source span, and graph-derived routes must converge to the same relevant
  evidence-ref set when they name the same semantic neighborhood.
- Adversarial denial test: impossible or contradictory relevance combinations
  must yield typed rejection or empty-localized posture rather than silently
  widening to a broad dump.

**Engineering decisions**
- Treat relevance routing as an inspection-admission boundary with typed
  narrowing inputs.
- Freeze the core narrowing axes: identity, source span, graph scope, aspect
  neighborhood, obligation family, evidence family, and detail posture.
- Make relevance closed by default so future broadening becomes additive and
  auditable.
- Keep `UiInspectionRelevance` and `UiRelevanceFilter` in one explicit layering
  relationship so callers do not choose between two ordinary narrowing models.

**Open questions**
- None.

### Phase 3: Materialize Stable Evidence References Before Rich Detail

This phase freezes the two-step inspection model: stable reference production
first, rich materialization second.

**Relevant subsystems**
- `worth-ui-inspection`
- `worth-ui-runtime`

**Relevant APIs**
- evidence reference handle types
- `UiEvidenceBudget`
- `UiEvidenceRichness`
- `UiEvidenceSlice`
- `UiInspectionReceipt`
- detail-posture or expansion-request surfaces associated with inspection
  queries

**Warnings**
- Do not make materialized detail the default shape for every inspection
  request.
- Do not let evidence refs encode renderer-local or process-ephemeral identity
  that cannot survive equivalent runtime questions.
- Do not couple evidence reference generation to a specific AI or panel
  consumer.

**Test requirements**
- Adversarial parity test: requesting `evidence_refs` first and then expanding a
  subset must converge to the same materialized detail as requesting that same
  subset directly through an equivalent typed path.
- Adversarial leakage test: a narrow reference-only request for one node,
  declaration, or source span must not materialize unrelated family detail or
  force unrelated index traversals.

**Engineering decisions**
- Introduce stable evidence handles that can be expanded later without changing
  their semantic identity.
- Keep reference production cheap enough to support iterative inspection loops.
- Encode detail posture explicitly so callers must ask for richer payloads.
- Require `UiInspectionReceipt` to preserve the original query, selected
  relevance, and richness posture so expansions remain auditable.

**Open questions**
- None.

### Phase 4: Index Declaration Identity And Source-Provenance Evidence

This phase creates the authored-side evidence indexes so inspection can reach
runtime explanation through declaration authority and typed authored provenance
without scans.

**Relevant subsystems**
- `worth-ui-runtime`
- `worth-ui-inspection`

**Relevant APIs**
- declaration identity -> evidence index surfaces
- source span -> declaration/admission evidence index surfaces
- `UiDeclarationIdentity`

**Warnings**
- Do not collapse declaration identity and source provenance into one generic
  authored lookup lane.
- Do not collapse source-span lookup into best-effort fuzzy source matching.
- Do not let authored lookup reopen source text or declaration lowering on the
  ordinary inspection path.

**Test requirements**
- Adversarial convergence test: equivalent lookup paths through declaration
  identity and authored source provenance must converge on the same evidence
  neighborhood when they target the same declaration fact.
- Adversarial residue test: deleting and rebuilding derived indexes from
  authority must reproduce the same lookup answers and must not require
  renderer-local or inspector-local repair.

**Engineering decisions**
- Treat indexes as derived runtime structures with explicit rebuild posture.
- Preserve `UiDeclarationIdentity` lookup and typed authored provenance lookup
  as separate ordinary lanes even when they converge on the same evidence
  neighborhood.
- Keep authored lookup tied to declaration authority and admission evidence
  only; runtime graph traversal belongs to the next phase.

**Open questions**
- None.

### Phase 5: Index Graph-Identity Evidence Neighborhoods

This phase creates the runtime-side graph-identity indexes so inspection can
reach declaration, admission, and obligation neighborhoods from admitted graph
truth without rescans.

**Relevant subsystems**
- `worth-ui-runtime`
- `worth-ui-inspection`

**Relevant APIs**
- graph node identity -> declaration/admission/obligation evidence index
  surfaces
- `UiGraphNodeIdentity`
- `UiGraphSnapshot`

**Warnings**
- Do not hide broad graph walks behind cheap-looking inspection getters.
- Do not mix authoritative graph truth with derived graph-evidence index
  storage.
- Do not infer graph neighborhoods from declaration shape when admitted graph
  truth already owns the runtime topology.

**Test requirements**
- Adversarial convergence test: equivalent lookup paths through graph identity
  and declaration-correspondence bridges must converge on the same graph-local
  evidence neighborhood when they target the same runtime node fact.
- Adversarial residue test: deleting and rebuilding derived graph-identity
  indexes from `UiGraphSnapshot`-backed authority must reproduce the same
  answers without renderer-local repair.

**Engineering decisions**
- Keep graph-identity lookup separate from authored lookup because the two lanes
  answer different authority questions and scale differently.
- Treat `UiGraphNodeIdentity` as the ordinary runtime-entry key for local
  evidence neighborhoods once graph truth is already known.
- Limit this phase to graph-local neighborhood indexing; aspect and obligation
  evidence broadening belongs to later phases.

**Open questions**
- None.

### Phase 6: Add Aspect Evidence Families And Aspect-Local Indexes

This phase adds aspect evidence as its own family and makes publication and
consumption neighborhoods inspectable without reopening aspect semantics
outside the owning runtime lanes.

**Relevant subsystems**
- `worth-ui-runtime`
- `worth-ui-inspection`

**Relevant APIs**
- published aspect -> publishers/receipts evidence surfaces
- consumed aspect -> dependents/receipts evidence surfaces
- aspect-local relevance selectors

**Warnings**
- Do not let aspect evidence recompute aspect semantics outside the owning
  runtime lanes.
- Do not compress published and consumed aspect neighborhoods into one vague
  "related evidence" surface.
- Do not make aspect-local inspection depend on later measurement or mounted
  receipt families.

**Test requirements**
- Adversarial parity test: equivalent aspect neighborhoods reached from
  publisher-first and dependent-first lookup paths must agree on the same
  relationship facts and evidence refs.
- Adversarial localization test: an aspect-local request must not widen into
  obligation, mounted, or cross-frame explanation when the aspect neighborhood
  itself is sufficient.

**Engineering decisions**
- Add aspect evidence as a family-local projection over existing declaration and
  graph authority, not as a new authority.
- Keep producer-side and consumer-side aspect indexes distinct because the
  traversal directions and question shapes differ.
- Preserve aspect-local explanation vocabulary so later families can attach to
  it without weakening it.

**Open questions**
- None.

### Phase 7: Add Obligation Evidence Families On 3.4 Authority

This phase adds obligation evidence as a separate family grounded directly on
3.4 selection, dispatch, verdict, and admission artifacts.

**Relevant subsystems**
- `worth-ui-runtime`
- `worth-ui-inspection`

**Relevant APIs**
- obligation evidence family surfaces
- `UiGraphTouchDescriptor`
- `UiSelectedObligationSet`
- `UiObligationDispatchPlan`
- `UiObligationVerdict`
- `UiAdmissionReport`

**Warnings**
- Do not let obligation evidence replace or reinterpret 3.4 admission and
  verdict authority.
- Do not recover obligation meaning from graph posture or diagnostic text when
  3.4 authority artifacts already exist.
- Do not blur selection reason, dispatch posture, verdict posture, and denial
  posture into one generic obligation explanation row.

**Test requirements**
- Adversarial convergence test: equivalent obligation-local queries through
  touch-derived and graph-node-derived routes must converge on the same
  retained obligation evidence neighborhood when they target the same admitted
  work.
- Adversarial anti-reopening test: hostile consumers must be unable to reopen
  obligation selection or verdict meaning by synthesizing inspection-local
  explanations disconnected from `UiSelectedObligationSet`,
  `UiObligationDispatchPlan`, `UiObligationVerdict`, or `UiAdmissionReport`.

**Engineering decisions**
- Obligation evidence must consume 3.4 authority artifacts directly instead of
  reverse-engineering obligation meaning from graph posture.
- Keep obligation evidence separate from aspect evidence because the authority
  chain and future broadening path are different.
- Preserve retained obligation evidence handles as the ordinary local expansion
  lane.

**Open questions**
- None.

### Phase 8: Define Evidence Slice Assembly As A Derived Projection

This phase defines how narrow evidence slices are assembled from refs, indexes,
and authority-backed detail without collapsing into a god assembler.

**Relevant subsystems**
- `worth-ui-inspection`
- `worth-ui-runtime`
- `worth-ui`

**Relevant APIs**
- `UiEvidenceSlice`
- `UiInspectionReceipt`
- family-local slice contributors
- inspection facade entrypoints that return evidence slices

**Warnings**
- Do not build one monolithic "inspect everything" assembler.
- Do not let slice assembly mutate runtime authority or index state directly.
- Do not mix stable references, family summaries, and rich materialized detail
  without naming those layers separately.

**Test requirements**
- Adversarial equivalence test: independently assembled slices for the same
  typed request must converge on the same family summaries, ref ordering, and
  materialized-detail boundaries.
- Adversarial breadth test: a slice request constrained to one family or local
  neighborhood must prove that unrelated families and indexes were not dragged
  into the assembly path.

**Engineering decisions**
- Treat slice assembly as a derived projection pipeline over typed refs and
  indexes.
- Make family-local assemblers subordinate to a narrow facade orchestration
  surface.
- Preserve deterministic ordering so evidence slices remain stable across
  equivalent requests.
- Keep family summaries, evidence refs, and materialized detail as separate
  layers inside the slice so future families can broaden without breaking the
  ordinary path.

**Open questions**
- None.

### Phase 9: Ship The First AI Inspection Harness On Real Runtime Evidence

This phase makes the 3.1 inspection promise real for AI workflows before
screenshots or mounted receipts exist.

**Relevant subsystems**
- `worth-ui`
- `worth-ui-inspection`
- `worth-ui-runtime`

**Relevant APIs**
- product inspection facade entrypoints
- `UiInspectionQuery`
- `UiInspectionTarget`
- `UiInspectionScope`
- `UiInspectionRelevance`
- `UiEvidenceBudget`
- `UiEvidenceRichness`
- `UiInspectionReceipt`
- targeted inspection query surfaces by declaration identity, source span,
  graph identity, and scope
- evidence-ref expansion entrypoints

**Warnings**
- Do not make the AI harness a parallel diagnostics protocol.
- Do not leak crate-internal topology because the caller happens to be an AI.
- Do not depend on screenshots, logs, or renderer debug helpers for 3.5
  closure.

**Test requirements**
- Adversarial convergence test: an AI-targeted query path and a direct product
  inspection query for the same target must return equivalent evidence slices
  and expansion behavior.
- Adversarial narrowness test: an AI-targeted request for a local scope must
  stay local and must not require the runtime to emit a whole-frame or
  whole-graph explanation blob.

**Engineering decisions**
- Keep the first AI harness path on the same inspection facade ordinary callers
  will use.
- Require typed target selection and typed scope/relevance selection.
- Make evidence refs first-class in the AI path so iterative inspection is the
  default workflow.
- Preserve compatibility with existing support and closure reporting:
  `UiInspectionSupportReport`, `UiInspectionScopeSupportRow`, and
  `UiInspectionClosureReport` remain the support-bearing lane around the richer
  evidence substrate rather than being displaced by it.

**Open questions**
- None.

### Phase 10: Certify Boundary Purity, Narrowness, And Growth Posture

This phase closes the milestone by proving the substrate is strong enough for
later evidence families rather than merely good enough for the current five.

**Relevant subsystems**
- `worth-ui-certification`
- `worth-ui-inspection`
- `worth-ui-runtime`
- `worth-ui`

**Relevant APIs**
- certification topology audits over inspection surfaces
- public inspection facade exports
- named evidence and relevance certification suites
- anti-bypass UI compile tests over `worth-ui` and `worth-ui-inspection`

**Warnings**
- Do not certify only behavior; certify structural boundaries.
- Do not let public APIs expose internal family storage or deep module imports.
- Do not close the milestone if adding a future family would still require a
  second inspection architecture.

**Test requirements**
- Adversarial growth-path test: adding a dummy future evidence family through
  the certified extension seams must have an obvious structural home and must
  not require widening generic blobs or bypassing the relevance boundary.
- Adversarial anti-bypass test: hostile consumers must be unable to explain
  runtime meaning through renderer-local, log-local, or facade-bypassing
  inspection shortcuts.

**Engineering decisions**
- Add explicit certification for evidence-family closure, relevance narrowing,
  index rebuild honesty, and facade purity.
- Treat the growth posture for future measurement, mounted receipt, and visual
  evidence as a proof obligation now.
- Keep milestone closure tied to machine-checkable structural proof, not prose
  claims about inspectability.
- Add topology audits proving that public callers depend on the named facade
  surfaces instead of deep imports into family-local evidence internals.

**Open questions**
- None.

## Must Ship

- `UiEvidenceSlice` as the ordinary projection artifact for narrow inspection
  responses
- `UiRelevanceFilter` as a typed, closed relevance-routing surface
- stable evidence reference handles with deferred detail expansion
- typed evidence families for:
  - declaration
  - admission
  - graph
  - aspects
  - obligations
- stable derived indexes for:
  - declaration identity -> evidence sets
  - source span -> declaration / admission evidence
  - graph node identity -> declaration / admission / obligation evidence
  - published aspect -> publishing nodes / receipts
  - consumed aspect -> dependent nodes / obligations / receipts
- family-local evidence slice assembly under one narrow inspection facade
- the first formal AI inspection harness path using real runtime evidence by
  identity, source span, and scope
- certification proving relevance narrowness, facade purity, anti-bypass
  enforcement, and growth posture for future evidence families

## Must Preserve

- runtime authority remains in declaration, graph, admission, support, and
  obligation owning lanes; inspection remains derived
- inspection and diagnostics stay one substrate for AI and human consumers
- product-facing facades stay narrow and do not export internal evidence
  topology
- equivalent inspection requests converge on equivalent refs and slice shape
- cheap-looking inspection APIs remain honest about traversal and materialization
  cost
- later measurement, mounted receipt, screenshot, and replay milestones can add
  family-local evidence without reopening the substrate
- no renderer-local, host-local, or log-local explanation path becomes
  necessary for 3.5-covered truth

## Acceptance Evidence

- an agent can inspect a declaration artifact without receiving a giant mixed
  dump
- an agent can inspect a graph node by identity and request only aspect-local
  or obligation-local evidence
- an agent can request `evidence_refs` before `materialized_detail`
- the inspection facade answers real declaration, admission, graph, aspect, and
  obligation questions through typed slices
- equivalent identity, source-span, and graph-neighborhood requests converge on
  the same evidence references when they target the same semantic truth
- deleting and rebuilding derived evidence indexes from authority reproduces the
  same inspection answers
- hostile consumers cannot mint evidence authority, bypass the product facade,
  or widen local requests into whole-runtime explanation by accident
- no renderer-local debug helper is required to explain 3.5-covered runtime
  meaning

## Sequencing Notes

- This milestone belongs after 3.4 because inspection breadth should consume
  admitted support, admission reports, selected obligations, and verdict truth
  rather than reinvent those decisions.
- This milestone belongs before 3.6 because measurement and allocation need a
  pre-existing evidence substrate rather than a bespoke explainer.
- This milestone belongs before 3.8 and 3.9 because mounted receipts,
  screenshots, and hit-testing need stable evidence refs and relevance routing
  before they broaden the visible inspection surface.
- 3.5 is intentionally narrower than "full diagnostics." It proves the shared
  substrate and the first substantial family set; future milestones extend that
  substrate with new authority-backed families.
