# Milestone 3.2 Engineering Spec: Canonical Declaration Artifacts And Aspect Contracts

> **Status:** Planned
>
> **Roadmap parent:** [worth_ui_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/worth-ui/plans/worth-ui/worth_ui_roadmap.md)
>
> **Primary prerequisite:** `Milestone 3.1 Inspection Boundary, DSL Ownership, And Certification Topology`
>
> **Follow-on sequence:** `Milestone 3.3 UI Authority Graph, Identity, Participation, And Core Indexes`
>
> **Primary architectural driver:** make authored UI meaning lower once into
> runtime-owned declaration authority rather than being rediscovered from source
> text, renderer behavior, or family-local helper logic.

## Goal

Make Worth UI declarations lower exactly once into a canonical, runtime-owned,
typed declaration artifact whose contracts are rich enough that later graph,
Query, diagnostics, inspection, and hot-reload lanes can consume authored
meaning without reopening source, inventing renderer-local semantic tables, or
smuggling meaning through support-only blobs.

Milestone 3.2 is complete when authored declarations have one authoritative
artifact identity, one admitted family authority surface, one aspect contract
surface, one declared topology-role surface, one declared Query or service or
touch or measurement or host-capability posture surface, and one typed support
snapshot for architecturally-owned but not-yet-admitted declaration semantics.

## Non-Goals

Milestone 3.2 does not implement:

- runtime UI graph truth
- graph node identity
- repeated runtime instance identity
- participation truth
- mounted receipts
- measurement execution
- host observations
- Query execution or projection consumption
- service execution
- obligation dispatch
- hot rebind planning
- visual snapshots
- replay
- inspector panel UI

3.2 closes canonical declaration authority only:

- authored semantic identity
- family authority
- aspect contracts
- declared structural intent
- declared Query or service or touch or measurement or host-capability posture
- support posture
- the sealed handoff surface 3.3 may consume

## Why This Sequence Exists

Milestone 2 gave Worth UI canonical source lowering. Milestone 3.1 gave the
runtime boundary, inspection facade, DSL crate ownership, host separation, and
certification topology. Milestone 3.2 is the next load-bearing slice: once the
source boundary exists and the public/runtime boundary is honest, authored UI
meaning itself must become runtime-owned authority.

This is not “more metadata on the parser result.” It is the semantic substrate
that 3.3, 3.4, and 3.5 depend on:

- 3.3 needs declaration-owned structural meaning before it can build graph
  identity, participation, and bounded indexes honestly.
- 3.4 needs declaration-owned semantic contracts before it can select touched
  obligations without source-local heuristics.
- 3.5 needs declaration-owned families, aspects, and support posture before it
  can operationalize inspection over real semantic evidence families.

If 3.2 stays thin, later milestones will rebuild authored meaning from source
text, helper branches, renderer logic, or graph-local inference. That would
violate the series architecture before the graph even lands.

## Governing Summaries

- `MENTALITY.md`
  protects adversarial-constraint-first design. 3.2 must solve the future
  semantic-authority problem now instead of shipping a pleasant parser output
  that later milestones are forced to reinterpret.
- `arch_laws.md`
  protects declared contracts over runtime discovery, proof-bearing
  progression, authoritative-vs-derived separation, and facade-only public
  surfaces. Declaration meaning must be typed, sealed, and runtime-owned.
- `composition_laws.md`
  protects named semantic steps. 3.2 must not collapse identity admission,
  family admission, aspect projection, topology projection, support posture,
  and certification into one declaration helper or god module.
- `domain_structure_laws.md`
  protects structural separability. Source lowering, declaration authority,
  graph truth, host projection, inspection support, and certification must live
  in distinct homes because they fail, scale, and evolve differently.
- `perf_laws.md`
  protects hot-path honesty. Later graph, obligation, and inspection lanes must
  consume already-lowered declaration contracts instead of repeatedly
  rediscovering semantic intent from source.
- `worth_ui_roadmap.md`
  places 3.2 between boundary setup and graph truth. This milestone must turn
  Milestone 2’s canonical lowering into `UiDeclarationArtifact`,
  `UiDeclarationIdentity`, `UiDeclarationFamily`, `UiAspectContract`, support
  scaffolding, and the first admitted family set before 3.3 or 3.4 begin.

## Adversarial Constraint

3.2 must survive this hostile condition:

> Worth UI grows a larger declaration family set, accepts repeated hot-reload
> edits, broadens Query binding, diagnostics, and inspection, and adds new host
> adapters and graph families. Across that growth, later runtime phases must
> derive authored semantic meaning from one canonical declaration artifact
> rather than reopening source text, reading renderer code, interpreting helper
> branches, or consulting family-specific folklore registries. Equivalent
> authored meaning must converge to equivalent declaration authority; changed
> meaning must change authority on the contract axis that actually changed; and
> unsupported declaration semantics must stay visible as typed support posture
> rather than falling out of the model.

If a later phase needs to read DSL source text to determine topology role, if a
renderer decides what a declaration publishes, if a family can bypass typed
admission by widening a generic blob, if a support row can certify declaration
truth, or if declaration identity depends on incidental tree position or source
ordering noise, 3.2 is not closed.

## Product Decision Lock

- 3.2 is an authority milestone, not a parser-polish milestone.
- `UiDeclarationArtifact` is the one canonical authored semantic artifact at
  this boundary.
- Declaration authority is distinct from graph truth, mounted truth, measured
  truth, and diagnostic projection.
- Declaration family admission is closed and typed; there is no generic
  “miscellaneous declaration” fallback.
- Aspect publication and consumption are typed declaration contracts, not
  freeform string maps.
- Query posture, service usage, touch meaning, and measurement policy are
  declaration-owned intent contracts, not execution receipts.
- Support posture may describe architecturally-owned but not-yet-admitted
  declaration semantics, but may not become a second semantic truth model.
- Public declaration surfaces must stay facade-curated and must not mirror
  internal topology one-for-one.

## Declaration Equivalence Contract

3.2 must make declaration equivalence explicit rather than leaving it to
implementation taste.

The declaration identity and equivalence basis must follow these rules:

- Identity-bearing inputs:
  - admitted declaration family
  - admitted aspect publication and consumption contracts
  - admitted structural role and structural semantics
  - admitted Query, service, touch, and measurement intent contracts
  - any family-declared authored ordering whose semantics are explicitly claimed
    by that family contract
- Non-identity-bearing inputs unless a family contract explicitly says
  otherwise:
  - formatting
  - whitespace
  - comments
  - token span locations
  - parser-local node ids
  - renderer labels
  - diagnostic wording
  - source ordering details that do not carry a declared semantic guarantee

This milestone does not allow “equivalent enough” identity. Every admitted
family contract must state whether authored ordering matters and on which exact
lane it matters. If the family does not claim authored ordering as semantic
meaning, ordering noise must normalize away before declaration identity or
equivalence is computed.

## Declaration Artifact Shape Lock

`UiDeclarationArtifact` is the sole authoritative declaration record for this
milestone.

The implementation may decompose its internal storage, but the architecture
must preserve these rules:

- family-specific contracts are contained within or sealed behind the canonical
  declaration artifact boundary
- no sibling registry, support table, renderer cache, host adapter, or graph
  helper may carry richer declaration semantic truth than the declaration
  artifact
- every public or internal consumer outside the declaration authority lane must
  consume declaration meaning through sealed declaration projections derived
  from `UiDeclarationArtifact`
- provenance, diagnostics, and support posture may annotate declaration
  authority, but may not become alternate semantic sources

If a later subsystem can recover richer declaration meaning from somewhere other
than the canonical artifact boundary, 3.2 has failed.

## Declaration Digest Structure

3.2 must split declaration digests by lane so contract-local changes are
mechanically visible.

The milestone requires an overall artifact digest plus lane-specific digest
projections, for example:

- `UiDeclarationArtifactDigest`
- `UiDeclarationIdentityDigest`
- `UiDeclarationFamilyDigest`
- `UiDeclarationAspectDigest`
- `UiDeclarationStructuralDigest`
- `UiDeclarationPostureDigest`
- `UiDeclarationSupportDigest`

The exact names may vary, but the separation may not. This is what allows the
milestone to prove:

- formatting changes do not mutate semantic digests
- diagnostic wording changes do not mutate semantic identity
- structural changes do not silently mutate Query posture digest
- posture changes do not silently mutate structural digest

If the implementation can only answer “the declaration changed somehow,” it is
too coarse for later runtime lanes to trust.

## Declaration Artifact Laws

3.2 must encode these laws structurally:

1. Authored declaration meaning lowers once. Later runtime phases may consume
   declaration authority, but may not reopen source text to rediscover meaning.
2. Declaration identity is authored-semantic identity, not graph position,
   mounted position, renderer path, or repeated-instance identity.
3. Family authority is explicit and closed. If a declaration belongs to a new
   semantic family, the milestone must admit a new family contract rather than
   widening an old family blob.
4. Aspect publication and consumption are separate typed contracts even when
   they share vocabulary.
5. Declared topology role is authoritative intent only. It may describe page,
   region, mosaic, local-composition, control, or related structural roles, but
   it may not fabricate graph node existence or participation truth.
6. Declared Query, service, touch, and measurement semantics are intent
   contracts only. They may guide later runtime lanes, but may not masquerade
   as executed evidence.
7. Support reporting is derivative and bounded. It may say “belongs here but
   not yet admitted,” but it may not carry richer authority than the admitted
   declaration artifact itself.
8. Every later milestone must be able to state exactly which declaration
   contract lane it consumes rather than “whatever the DSL meant.”

## DSL Semantic Input Artifact

3.2 must make the declaration input boundary explicit so “source reopening
rejection” is mechanically meaningful.

The required lowering progression is:

`DSL source -> parsed source AST -> UiDslSemanticArtifact -> UiDeclarationArtifact`

This milestone therefore requires a named semantic DSL input artifact and
lowering provenance surface, for example:

- `UiDslSemanticArtifact`
- `UiDslSourceProvenance`
- `UiDslLoweringReceipt`

The exact names may differ, but the architectural roles may not:

- parsed source AST is not declaration authority
- `UiDslSemanticArtifact` is the resolved, sugar-normalized, semantically
  admitted DSL-owned input to runtime declaration lowering
- `UiDslLoweringReceipt` is the proof-bearing handoff that the runtime consumed
  semantic DSL input rather than raw source text

This is the boundary that replaces raw source as declaration input. Runtime
declaration authority may consume semantic DSL artifacts and their lowering
receipts, but may not consume raw source text once lowering completed.

## Declaration Access And Projection Cost Law

3.2 must make the cost claims around declaration authority explicit.

- lookup by `UiDeclarationIdentity` must be bounded and must not require source
  reopening, renderer consultation, or broad artifact scans
- family classification must be derivable from admitted declaration authority,
  not rediscovered from source text
- aspect, structural, posture, and support projections must be produced from
  canonical declaration authority rather than reparsing or rerunning lowering
- declaration support and handoff surfaces must expose structural counters or
  exact proof tests sufficient to show that steady-state access stays bounded

This milestone does not require the full 3.3 index set, but it does require
that declaration authority be consumable as a stable bounded source rather than
an opaque blob that later milestones must repeatedly reopen.

## Planned Directory Skeleton

3.2 should force the runtime tree toward a declaration-owned responsibility
shape rather than one declaration blob or a facade dump. The exact file names
may evolve, but the topology should look like this:

```text
workspaces/worth-ui/crates/worth-ui-runtime/src/
  declaration/
    artifact/
      identity.rs
      artifact.rs
      artifact_digest.rs
      provenance.rs
      mod.rs
    family/
      family.rs
      admission.rs
      denial.rs
      catalog.rs
      contracts/
        page.rs
        page_set.rs
        region.rs
        mosaic.rs
        local_composition.rs
        control.rs
        query_binding.rs
        intent.rs
        diagnostic_surface.rs
        mod.rs
      mod.rs
    aspect_contract/
      published.rs
      consumed.rs
      coverage_report.rs
      mod.rs
    structural_semantics/
      topology_role.rs
      structure_contract.rs
      mod.rs
    declaration_handoff/
      structural_payload.rs
      posture_payload.rs
      handoff_artifact.rs
      mod.rs
    declared_posture/
      query.rs
      service.rs
      touch.rs
      measurement.rs
      host_capability.rs
      mod.rs
    support/
      snapshot.rs
      row.rs
      unsupported_posture.rs
      mod.rs
    mod.rs
```

Public surfaces should stay curated under the Worth UI facade. Certification
should own hostile construction fixtures, topology audits, residue scans, and
public-surface proof rather than leaving those in runtime-local tests. This
directory skeleton is normative about responsibility shape, not a requirement
that every file name above land exactly as written.

## Declaration Certification Matrix

Every major 3.2 contract surface must map to a named proof family:

- `declaration_identity_suite`
  proves stable identity, semantic equivalence, and anti-WORTHry construction
  rules.
  owning crate: `worth-ui-certification`
  enforcement: compile-fail plus equivalence-runtime proof
- `declaration_family_admission_suite`
  proves the admitted family catalog, family-specific admission, and unknown or
  partial family denials.
  owning crate: `worth-ui-certification`
  enforcement: runtime certification plus topology audit
- `aspect_contract_suite`
  proves typed publication/consumption, coverage projection, and anti-stringly
  boundaries.
  owning crate: `worth-ui-certification`
  enforcement: compile-fail plus runtime certification
- `structural_semantics_suite`
  proves topology-role projection, structural-lane locality, and graph-handoff
  readiness without graph fabrication.
  owning crate: `worth-ui-certification`
  enforcement: runtime certification plus hostile handoff proof
- `declared_posture_suite`
  proves Query, service, touch, measurement, and host-capability intent
  contracts with typed denials and no executed-evidence leakage.
  owning crate: `worth-ui-certification`
  enforcement: compile-fail plus runtime certification
- `support_snapshot_suite`
  proves unsupported or not-yet-admitted declaration semantics remain visible
  through typed support rows without becoming authority.
  owning crate: `worth-ui-certification`
  enforcement: runtime certification plus residue scan
- `public_facade_boundary_suite`
  proves public callers cannot reach through the facade into internal family or
  artifact topology.
  owning crate: `worth-ui-certification`
  enforcement: compile-fail plus topology audit
- `declaration_source_reopening_rejection_suite`
  proves runtime production code does not re-read source text or renderer-local
  helpers to recover declaration meaning once lowering completed.
  owning crate: `worth-ui-certification`
  enforcement: residue scan and dependency audit
- `milestone_3_3_handoff_suite`
  proves 3.3 consumes proof-bearing declaration contract projections, not raw
  source products or support-only rows.
  owning crate: `worth-ui-certification`
  enforcement: compile-fail plus hostile handoff-runtime proof

Every suite must name its hostile lane, its anti-bypass lane, its exact surface
under test, and the declaration contract law it is certifying.

## Test Topology Requirements

3.2 tests must obey the same structure law as production:

- declaration identity fixtures belong with identity proof, not in generic
  helpers
- family admission fixtures belong with family authority proof, not under
  parser-world test support
- aspect contract hostile cases belong under aspect boundary tests, not broad
  “milestone 3” test files
- topology-role proof belongs under structural semantics, not future graph
  tests
- support snapshot proof belongs under support posture, not inspection-only
  broad suites

Required hostile topology:

- compile-fail fixtures for raw construction of artifacts, identities,
  families, aspect contracts, and family-owned contract wrappers
- residue scans proving runtime or host code does not reinterpret declaration
  meaning from source text or renderer-local tables
- facade-boundary tests proving public code cannot deep-import internal family
  topology
- convergence or equivalence tests proving semantically equivalent authored
  declarations lower to equivalent authority
- localization tests proving a change in one contract lane does not silently
  mutate unrelated declaration authority lanes
- support-row schema proof showing support rows are produced from a declared
  schema rather than ad hoc family-local formatting logic

## Declaration World-Sensitivity Law

3.2 must make world-sensitivity explicit.

- declaration artifacts are authored-semantic and mostly world-neutral
- admission projections over those artifacts may be world-sensitive
- if authored declaration semantics differ by world, that difference must appear
  as a declared posture lane and must participate in equivalence

This prevents accidental conflation between:

- the same authored declaration admitted differently in different worlds
- two authored declarations that are semantically different because their
  world-sensitive meaning changed

## Phases

### Phase 1: Freeze Canonical Artifact Envelope And Authored Identity Law

Phase 1 defines the one authoritative declaration artifact shape and the stable
identity basis every later phase must consume.

**Relevant subsystems**

- `worth-ui-dsl`
- runtime declaration artifact lane
- Worth UI public facade
- certification compile-fail boundary suite

**Relevant APIs**

- `UiDeclarationArtifact`
- `UiDeclarationIdentity`
- declaration artifact admission entry point
- artifact read-only projection surface

**Warnings**

- Do not make the artifact a parser-shaped debug object with incidental fields.
- Do not derive identity from tree position, renderer path, or repeated
  instance count.
- Do not let graph truth, mounted truth, or support truth leak into the
  authoritative artifact envelope.

**Test requirements**

- Equivalence test: semantically equivalent declarations with formatting,
  lexical, or ordering noise lower to equivalent artifact identity and contract
  digest.
- Rejection test: raw strings, parsed source fragments, renderer-local labels,
  or graph-local identities cannot mint `UiDeclarationArtifact` or
  `UiDeclarationIdentity`.
- Localization test: changing a declaration semantic lane changes only the
  relevant artifact digest or projection basis rather than silently preserving
  stale identity.

**Engineering decisions**

- Artifact identity is authored-semantic identity only.
- The artifact envelope is sealed and read-only outside its owning admission
  boundary.
- Artifact provenance may preserve source-lowering lineage, but provenance is
  not semantic authority.

**Open questions**

- None.

### Phase 2: Admit Closed Declaration Family Authority

Phase 2 turns the initial family set into explicit runtime authority rather
than parser branches or optional blob fields.

**Relevant subsystems**

- declaration family admission lane
- family catalog
- family-specific contract modules
- certification family-boundary suites

**Relevant APIs**

- `UiDeclarationFamily`
- family admission report or denial surface
- family-specific contract wrappers
- initial family catalog surface

**Warnings**

- Do not encode family differences by adding many nullable fields to
  `UiDeclarationArtifact`.
- Do not create a catch-all family that can absorb undeclared semantics.
- Do not let later lanes pattern-match on DSL source kinds instead of admitted
  family authority.

**Test requirements**

- Catalog parity test: every roadmap family (`page`, `page-set`, `region`,
  `mosaic`, `local-composition`, `control`, `query-binding`, `intent`,
  `diagnostic-surface`) appears exactly once in the admitted family catalog.
- Rejection test: unknown, partial, or contradictory family claims deny through
  typed family admission, not through late downstream failure.
- Boundary test: external callers cannot directly construct family-owned
  contract wrappers or bypass the catalog.

**Engineering decisions**

- Each family owns its typed contract shape.
- Shared semantic projection is a projection layer, not a giant family-unifying
  blob.
- The admitted family catalog is authority; support posture may mention future
  families but may not admit them.
- `query-binding` and `intent` may appear in two roles:
  - as standalone declaration families when they define retained reusable
    declaration artifacts
  - as declared posture projections attached to structural or control families
- the family catalog and the posture lanes must distinguish those two roles
  explicitly so one does not duplicate or silently replace the other.

**Open questions**

- None.

### Phase 3: Define Typed Aspect Publication And Consumption Contracts

Phase 3 makes semantic publication and semantic consumption first-class
declaration contracts.

**Relevant subsystems**

- declaration aspect contract lane
- declaration coverage reporting
- Query-facing contract projection seam
- certification anti-stringly contract suites

**Relevant APIs**

- `UiAspectContract`
- declaration published-aspect projection
- declaration consumed-aspect projection
- aspect coverage report

**Warnings**

- Do not model aspect contracts as string bags or renderer-owned labels.
- Do not hide richer semantic meaning in source helpers than the contract can
  express.
- Do not let support or inspection surfaces become the only authoritative view
  of aspect meaning.

**Test requirements**

- Equivalence test: equivalent publication or consumption semantics converge to
  equivalent typed aspect contracts regardless of authored spelling details.
- Rejection test: raw strings, ad hoc Query labels, or renderer-local semantic
  tags cannot satisfy `UiAspectContract`.
- Coverage test: the coverage report explains what a declaration publishes and
  consumes without source reopening.

**Engineering decisions**

- Publication and consumption are separate typed claims, even when they share
  vocabulary.
- Later graph, obligation, and inspection lanes consume the declaration aspect
  contract rather than derive their own semantic tables.
- Coverage reporting is derivative of admitted contract authority.

**Open questions**

- None.

### Phase 4: Carry Structural Declaration Semantics Without Fabricating Graph Truth

Phase 4 defines the structural role surfaces 3.3 will later instantiate into
graph truth.

**Relevant subsystems**

- declaration structural semantics lane
- topology-role projection
- future graph handoff contract
- certification structural-role suites

**Relevant APIs**

- declaration topology-role projection
- page or region or mosaic or local-composition structure contracts
- graph-consumable structural handoff surface

**Warnings**

- Do not instantiate graph nodes, repeated instances, or participation truth in
  3.2.
- Do not collapse page, region, mosaic, and local composition into one generic
  “container” contract.
- Do not leave slot or membership meaning implicit in source traversal order.

**Test requirements**

- Handoff parity test: every admitted structural family projects graph-consumable
  structural meaning without source reopening or renderer-local interpretation.
- Rejection test: incomplete or contradictory structural declarations fail here
  rather than escaping to 3.3 as half-formed graph truth.
- Localization test: non-structural declaration changes do not silently change
  structural-role projections.

**Engineering decisions**

- Declaration artifact owns structural intent only.
- 3.3 owns admitted graph existence, repeated-instance identity, and
  participation posture.
- Structural semantics may declare slot or membership intent but cannot produce
  graph node identities.
- 3.2 must hand 3.3 one sealed declaration handoff artifact composed only of:
  - declaration identity
  - declaration family
  - admitted structural role
  - declared containment or membership intent
  - declared slot ownership or slot participation intent
  - declared authored ordering guarantee only where the family contract says
    ordering is semantically meaningful
  - declared repetition posture only where the family contract admits it

3.3 must not need any additional source reopening to recover structural intent.

**Open questions**

- None.

### Phase 5: Admit Declared Query, Service, Touch, Measurement, And Host-Capability Intent Contracts

Phase 5 closes the non-graph semantic postures later runtime lanes require.

**Relevant subsystems**

- declared Query posture lane
- declared service usage lane
- declared touch or interaction posture lane
- declared measurement policy lane
- declared host-capability posture lane

**Relevant APIs**

- declaration Query binding posture projection
- declaration service usage projection
- declaration touch meaning projection
- declaration measurement policy projection
- declaration host-capability posture projection

**Warnings**

- Do not let Query posture fall back to source strings or host-local logic.
- Do not confuse declared measurement policy with measured runtime observation.
- Do not push touch or service meaning into family-local helper code for later
  recovery.
- Do not let host capability requirements first appear in host-adapter code.
- Do not leave applicability classification implicit.

**Test requirements**

- Convergence test: later-facing Query, service, touch, and measurement
  projections are derivable from declaration authority alone.
- Rejection test: unsupported Query posture, contradictory service usage,
  impossible touch meaning, or invalid measurement policy deny through typed
  local surfaces.
- Separation test: declared posture cannot be promoted into executed Query
  receipts, host observations, or allocation outcomes.
- Host-boundary test: text input, IME, accessibility, font metrics, visual
  capture prerequisites, or other host capability gates must appear first as
  declared host-capability posture rather than host-adapter inference.

**Engineering decisions**

- These are intent contracts only.
- Query posture belongs in 3.2 because later Query/runtime lanes must inherit a
  typed authored basis.
- Measurement policy belongs in 3.2 only as authored policy, not observed fact.
- Host capability posture belongs in 3.2 because later host adapters must admit
  or deny against authored requirements rather than invent those requirements
  locally.
- Every admitted family must classify each posture lane as exactly one of:
  - `required`
  - `optional`
  - `not_applicable`
  - `architecturally_owned_but_not_yet_admitted`
- `not_applicable` means the family does not semantically participate in that
  posture lane.
- `architecturally_owned_but_not_yet_admitted` means the lane belongs to the
  family’s long-term runtime contract and must appear through typed support
  posture until admitted.

**Open questions**

- None.

### Phase 6: Add Support Snapshot For Architecturally-Owned But Not-Yet-Admitted Semantics

Phase 6 makes future declaration semantics visible without creating a second
semantic authority model.

**Relevant subsystems**

- declaration support snapshot lane
- declaration support rows
- inspection-facing support projection
- certification unsupported-coverage suites

**Relevant APIs**

- declaration support snapshot
- declaration support row
- typed unsupported or not-yet-admitted posture

**Warnings**

- Do not let support rows carry richer authority than admitted declaration
  contracts.
- Do not use support posture as a compatibility escape hatch.
- Do not report unsupported semantics through strings alone.

**Test requirements**

- Coverage parity test: every architecturally-owned but not-yet-admitted
  declaration semantic lane appears as typed support posture instead of falling
  out of the model.
- Rejection test: support rows cannot be promoted into declaration authority,
  graph truth, or runtime evidence.
- Localization test: unsupported posture for one semantic lane does not imply
  the entire declaration subsystem is unavailable.

**Engineering decisions**

- Support snapshot is derivative of declaration authority plus roadmap-owned
  expectation.
- Unsupported posture is honest only when it names the specific semantic lane
  that remains outside current admission.
- This phase exists to keep the public contract stable while later milestones
  broaden semantic coverage.
- Support rows may only project:
  - admitted declaration contract facts
  - exact lane classification (`required`, `optional`, `not_applicable`,
    `architecturally_owned_but_not_yet_admitted`)
  - explicit roadmap-declared future declaration lanes
  - typed unsupported or unavailable posture
- Support rows must be generated from a declared support-row schema rather than
  ad hoc family-local assembly logic.
- Support rows may not invent new semantic families, new semantic categories, or
  richer semantics than the admitted declaration authority carries.

**Open questions**

- None.

### Phase 7: Mechanize Boundary Enforcement And Residue Rejection

Phase 7 turns the declaration architecture into compile-time and certification
proof rather than prose.

**Relevant subsystems**

- `worth-ui-certification`
- compile-fail fixture catalog
- topology and dependency audits
- public facade export review

**Relevant APIs**

- declaration compile-fail boundary suites
- topology audit
- dependency audit
- curated public declaration facade surface

**Warnings**

- Do not rely on review comments to prevent raw construction or source
  reopening.
- Do not mirror internal declaration topology through public exports.
- Do not bury hostile boundary proof inside broad integration test files.

**Test requirements**

- Compile-fail test: public callers cannot mint artifacts, identities,
  families, aspect contracts, or family-owned contract wrappers from raw
  values.
- Residue test: production source rejects renderer-local, host-local,
  graph-local, or source-reopening reinterpretation of declaration meaning
  outside the owning lanes.
- Facade test: declaration public surfaces route through curated facade exports
  rather than giant export mirrors.

**Engineering decisions**

- Certification owns the anti-cheating proof for 3.2.
- Public declaration surfaces must stay narrow and capability-shaped.
- Residue rejection is part of ordinary completion, not a later cleanup lane.

**Open questions**

- None.

### Phase 8: Close Declaration Authority And Hand 3.3 A Proof-Bearing Contract Surface

Phase 8 publishes the exact handoff 3.3 may consume and the exact non-goals 3.2
must not pretend to have solved.

**Relevant subsystems**

- declaration closeout proof lane
- graph handoff contract surface
- support continuity surface
- certification closeout audit

**Relevant APIs**

- `UiDeclarationCloseoutReport`
- `UiDeclarationGraphHandoff`
- the exact 3.3 graph-consumable declaration handoff surface

**Warnings**

- Do not hand 3.3 raw source products, parser-local blobs, or support-only rows.
- Do not claim graph node identity, participation truth, mounted truth, or
  measured truth from 3.2.
- Do not certify milestone completion from roadmap text alone.

**Test requirements**

- Handoff-equivalence test: 3.3-facing contract inputs are fully derivable from
  admitted declaration authority without source reopening.
- Rejection test: raw source products, support-only rows, and graph-shaped
  receipts are denied as 3.3 declaration inputs.
- Coverage test: closeout proof enumerates the admitted family set and the
  declaration semantic lanes actually closed in 3.2.

**Engineering decisions**

- 3.2 closes at the declaration-authority boundary only.
- `UiDeclarationGraphHandoff` is the one proof-bearing sealed declaration handoff artifact, not
  an open-ended set of ad hoc projections.
- Closeout must make the remaining graph work explicit while preserving the
  truth that declaration meaning is already canonical.

**Open questions**

- None.

## Must Ship

- `UiDeclarationArtifact`
- `UiDslSemanticArtifact` or an equivalent semantic DSL input artifact
- `UiDslSourceProvenance`
- `UiDslLoweringReceipt` or an equivalent semantic-source-to-declaration
  lowering receipt
- `UiDeclarationIdentity`
- `UiDeclarationFamily`
- `UiAspectContract`
- `UiDeclarationArtifactDigest` with lane-specific digest projections
- typed declaration contract lanes for:
  - structural role
  - Query binding posture
  - service usage
  - touch or interaction meaning
  - measurement policy
  - host capability posture
- support snapshot and support-row scaffolding for declaration families
- initial admitted declaration families:
  - `page`
  - `page-set`
  - `region`
  - `mosaic`
  - `local-composition`
  - `control`
  - `query-binding`
  - `intent`
  - `diagnostic-surface`
- `UiDeclarationGraphHandoff` as the graph-consumable declaration handoff surface for 3.3
- exact 3.3 sealed handoff artifact carrying only admitted structural and
  declared posture contracts, never raw source or support-only authority
- certification proof that declaration authority cannot be WORTHd or replaced by
  source, renderer, host, or graph-local reinterpretation

## Must Preserve

- Milestone 3.1’s single public facade discipline
- runtime ownership of truth and inspection authority
- DSL ownership as a first-class crate boundary
- host neutrality through `worth-ui-host-contract`
- strict separation between authoritative declaration meaning and derived graph,
  mount, observation, and diagnostic truth
- one canonical lowering path from authored source into runtime-owned authority
- typed unsupported posture instead of missing-API or string failure

## Acceptance Evidence

3.2 is complete only when all of these are true:

- authored declarations lower once into a canonical artifact with stable
  identity, family, aspect contract, topology role, Query posture, service
  posture, touch meaning, measurement policy, and host-capability posture
- runtime declaration lowering consumes a semantic DSL input artifact and
  lowering receipt rather than raw source text
- no later runtime phase needs to reopen source text or consult renderer-local
  logic to understand declaration meaning
- equivalent authored meaning converges to equivalent declaration authority
- changed authored meaning changes the correct contract lane without hidden
  equivalence drift
- lane-specific digests prove that semantic changes mutate the correct digest
  family and do not create unrelated digest churn
- support reporting explains architecturally-owned but not-yet-admitted
  declaration semantics without becoming an alternate authority model
- 3.3 can consume declaration structural meaning through a proof-bearing handoff
  surface rather than raw source products
- compile-fail, topology, dependency, and residue suites prove public callers
  cannot mint or bypass declaration authority

## Allowed Debt

3.2 may reserve richer future declaration families, broader semantic lanes, and
later graph-participation-specific support surfaces for later milestones when
the ordinary declaration authority path already exists.

Any allowed debt must satisfy the standard from `MENTALITY.md`: it must be a
named blocker, major enough to justify deferral, mechanically contained so it
cannot be mistaken for the ordinary lane, and attached to an explicit follow-on
milestone. “We can flesh this out later” is not allowed debt.

3.2 may not mark these as debt:

- canonical declaration artifact admission
- stable authored identity law
- semantic DSL input artifact and lowering receipt boundary
- closed family authority and family catalog admission
- typed aspect publication and consumption contracts
- lane-specific declaration digest structure
- declared structural role projection for 3.3 handoff
- declared Query, service, touch, measurement, and host-capability posture
  contracts
- typed support snapshot for not-yet-admitted declaration semantics
- schema-limited support row generation
- compile-fail proof against raw declaration artifact or contract construction
- residue rejection for source reopening and renderer-local reinterpretation
- curated public facade rather than export-mirror topology
- concrete 3.3 handoff contract surface

## Sequencing Notes

3.2 belongs immediately after 3.1 because boundary honesty alone is not enough:
the runtime needs canonical authored semantic authority before graph truth,
obligation selection, and richer inspection can proceed honestly.

3.2 belongs before:

- 3.3, because graph node identity, bounded indexes, and participation cannot
  be truthfully built on parser-local or renderer-local declaration meaning
- 3.4, because touched obligations need declaration-owned semantic contracts
  before obligation selection can be runtime-owned
- 3.5, because substantial inspection evidence families need declaration
  authority to explain what a declaration means before they can explain what the
  runtime did with it

The DSL must co-develop with 3.2. Source sugar may only express declaration
semantics that this milestone can admit honestly.

## Required Self-Check

Before closeout, answer these with evidence:

- Does 3.2 make declaration meaning runtime-owned authority rather than a
  parser result with better names?
- Can every later consumer name the declaration contract lane it depends on
  rather than “whatever the DSL meant”?
- Can equivalent authored declarations converge without source-sensitive drift?
- Can public callers, host adapters, and renderer code only observe
  declaration authority through the facade and read-only projections?
- Does the 3.3 handoff avoid raw source products, graph fabrication, and
  support-only pseudo-authority?

Reopen 3.2 if any of these become true:

- later runtime phases reopen source text to recover declaration meaning
- renderer or host code owns semantic interpretation that should belong to the
  declaration artifact
- a new declaration family can appear without explicit family admission
- support rows become richer or more authoritative than the declaration
  artifact they describe
- graph, mount, or measured truth is conflated with declared intent
- public exports mirror internal declaration topology so deeply that refactoring
  the internal tree would become a breaking change
