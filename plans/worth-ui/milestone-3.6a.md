# Milestone 3.6a Engineering Spec: Measurement Vocabulary, Basis Admission, And Host Evidence Boundaries

> **Status:** Planned
>
> **Roadmap parent:** [worth_ui_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/worth-ui/plans/worth-ui/worth_ui_roadmap.md)
>
> **Primary prerequisite:** `Milestone 3.5 Inspection Evidence Expansion And Relevance Indexes`
>
> **Follow-on sequence:** `Milestone 3.6b Allocation Neighborhood Planning And Constraint Propagation`
>
> **Primary architectural driver:** make measurement a runtime-owned semantic and proof-bearing planning input before allocation receipts, resize churn, scroll extent changes, portal anchoring, and drag-heavy replanning broaden the surface.

## Goal

Freeze the first half of Worth UI measurement as a runtime authority boundary.

Milestone 3.6a is complete when Worth UI can accept canonical declaration
measurement meaning, admit or deny that meaning through typed support posture,
exchange host-supplied measurement evidence without letting the host own layout
semantics, and produce a deterministic measurement basis that later allocation
planning can consume without rediscovering declaration, Query, or host facts.

This milestone closes the pre-allocation side of measurement:

- what measurement meaning exists at declaration time
- which measurement modes are admitted in which worlds and host profiles
- which host observations count as evidence rather than layout truth
- how Query-backed content facts participate in measurement basis without
  turning Worth UI into a second Query runtime
- how measurement support, evidence, and inspection remain typed before
  committed allocation receipts or continuous interaction pressure arrive

It does not close:

- committed `UiAllocationReceipt` truth
- viewport-resize churn handling
- splitter drag replanning
- scroll-owned extent maintenance
- portal-anchor continuous repositioning
- mounted receipt projection
- hit testing
- visual geometry evaluation
- frame-lane lowering

## Why This Milestone Exists

Milestone 3.2 made declarations rich enough to carry measurement posture.
Milestone 3.3 made structure and participation runtime-owned. Milestone 3.4
made graph-touch obligations explicit. Milestone 3.5 made runtime evidence
typed and inspectable.

3.6a is where measurement stops being a declaration adjective and becomes a
runtime-owned semantic lane.

Without this slice, later resize, scroll, portal, and drag work would be
forced to improvise around unresolved questions:

- does `hug`, `fill`, `bounded`, or `viewport-relative` mean anything precise
  beyond a DSL token?
- which host-returned values are evidence and which are authority?
- when Query-backed content shape changes, what measurement fact actually
  changed?
- what is the typed input to later allocation planning?
- how does inspection explain measurement denials before mounted receipts exist?

If those questions are left implicit, later milestones will encode measurement
semantics in host adapters, canvas helpers, mosaic widgets, or scroll/portal
special cases. That would create folklore instead of architecture.

## Governing Summaries

- `MENTALITY.md`
  protects adversarial-constraint-first design. 3.6a must be shaped by hostile
  resize/content/host churn pressure rather than by a static happy-path layout
  demo.
- `arch_laws.md`
  protects authority versus derivation, phase-typed progression, and explicit
  boundary crossings. 3.6a must keep declaration meaning, measurement basis,
  host evidence, and later allocation outputs as distinct artifacts.
- `composition_laws.md`
  protects named semantic steps. 3.6a must not collapse admission, basis
  assembly, host evidence intake, Query fact consumption, and inspection into
  one layout helper.
- `domain_structure_laws.md`
  protects separate ownership for declaration posture, measurement basis,
  host-contract evidence exchange, and later allocation planning. 3.6a must
  give each one a visible home.
- `perf_laws.md`
  protects breadth honesty and proof-carrying cost claims. 3.6a must narrow
  future replanning to typed measurement neighborhoods rather than broad tree
  recomputation.
- `worth_ui_roadmap.md`
  protects the sequence. Measurement planning semantics must exist before
  allocation receipts, resize churn, scroll-owned extents, and continuous
  interaction measurement broaden the runtime.
- `WORTH_UI_README.md`
  protects the actual runtime stack. 3.6a must keep measurement in Worth UI,
  host observations in `worth-ui-host-contract`, and Query-backed state/basis
  in Query-owned lanes rather than inventing a second truth runtime.
- `worth-ui-vision.md`
  protects Worth UI as a real desktop platform rather than widget-local layout
  folklore. 3.6a must support dense shell/layout work, inspection, and
  hot-lowered iteration without web-style overflow magic.
- `worth-ui-dsl-vision.md`
  protects lane-oriented authoring. 3.6a must give layout and measurement
  operators admitted runtime meaning instead of modifier-chain or parent-magic
  semantics.
- `ai-diagnostics.md`
  protects one shared evidence substrate. 3.6a measurement posture and host
  evidence must inspect through typed evidence and support rows, not logs.
- `crates/worth-query/docs/AI_README.md`
  protects Query-owned basis, retained inspection, projection consumption, and
  cross-runtime causal inspection. 3.6a must consume `ResolvedSnapshotBasis`,
  `SnapshotResolutionReport`, `workspace.inspect(...)`,
  `consume_projection_facts(...)`, `admit_causal_inspection`, and
  `request_causal_inspection` through `worth-ui-query-binding` where relevant,
  not restate them as UI-local pseudo Query APIs.

## Adversarial Constraint

3.6a must survive this hostile condition:

> A running Worth UI app contains nested mosaic regions, local composition,
> text/content-driven controls, Query-backed inspector fields, portal anchors,
> and viewport-relative shell regions. Source edits, Query-backed content
> changes, viewport observations, font/text metrics, and host capability
> changes all arrive while the app remains live. The runtime must classify
> measurement meaning once, admit or deny that meaning structurally, consume
> only the host evidence and Query facts required for measurement, and produce
> a deterministic measurement basis for later planning without letting host
> code decide layout truth, without re-reading source or Query internals on the
> hot path, without reusing stale basis inputs across generation boundaries, and
> without collapsing future resize/scroll/drag churn into broad whole-graph
> replanning.

If equivalent declaration + Query basis + host evidence inputs can produce
different measurement basis artifacts, if measurement meaning is implicit in
operators or host widgets, if host metrics can silently override declaration
semantics, or if later churn must rediscover the same facts through scans,
3.6a is not closed.

## Product Decision Lock

- Measurement is a runtime semantic lane, not a host service and not a widget
  convenience helper.
- Hosts may report measurement evidence only. Hosts may not decide layout
  meaning, overflow meaning, scroll ownership, or anchor semantics.
- Query may provide projected facts and basis posture only through Query-owned
  lanes. Worth UI may consume those facts for measurement; it may not recreate
  Query basis or result-state models.
- `UiMeasurementRequest`, `UiMeasurementResult`, and the 3.6a measurement basis
  artifact are distinct concepts:
  - request = runtime-owned ask for host evidence
  - result = typed host-returned evidence
  - basis = normalized runtime input to later allocation planning
- The 3.6a measurement vocabulary is not one flat list. It must be classified
  into explicit axes:
  - measurement mode
  - constraint modifier
  - basis source
  - ownership/lifecycle posture
  - evidence requirement
- Declaration measurement posture, host evidence, and later committed
  allocation receipts are different phases and must stay different types.
- 3.6a closes static and generation-local measurement meaning only.
  Continuous interaction churn remains a later milestone.
- Scroll-owned and portal-anchored semantics are especially dangerous. 3.6a
  may close only planning-time basis meaning such as scroll-container basis,
  scroll-viewport basis, scroll-content-extent-required posture, and
  portal-anchor-measurement-basis-required posture. Continuous extent
  maintenance, scroll position, anchor repositioning, and portal behavior stay
  out of scope.
- Measurement inspection must flow through the shared inspection substrate and
  `UiInspectionScope::Measurement`; it must not create a special layout-only
  debugger path.
- The ordinary path must be deterministic: equivalent declaration authority,
  Query basis, world, host capability profile, and host evidence must converge
  on the same measurement basis.
- Measurement basis assembly must be generation-aware. Host evidence, Query
  fact receipts, viewport facts, and declaration support posture may not be
  combined across incompatible generations and then treated as a valid basis.
- Unsupported, stale, contradictory, partial, and capability-gated measurement
  outcomes must remain distinct typed states. 3.6a may not compress them into
  one generic "measurement failed" branch.
- Unit, coordinate-space, and rounding posture must be explicit wherever host
  evidence can otherwise look equivalent while differing structurally.
- The existing `worth-ui-runtime/src/runtime/measurement/` subsystem remains the
  performance-counter and certification subsystem
  (`WorthUiMeasurementBoundary`, `WorthUiMeasurementCounterPacket`,
  `WorthUiMeasurementCertificationDenial`). 3.6a must not overload that module
  into UI layout semantic authority. If new UI measurement semantics need a
  runtime home, they must live in a distinct semantic lane and bridge to the
  counter subsystem only where later performance claims require it.

## Existing API Anchors

This milestone must build from the real seams already present in the repo
rather than naming a fresh imaginary runtime.

Current declaration-time measurement anchors:

- public facade re-exports in
  `workspaces/worth-ui/crates/worth-ui/src/facade/registry.rs`
  and `workspaces/worth-ui/crates/worth-ui-runtime/src/facade/mod.rs`
- `NamedMeasurementDefinition`
- `NamedMeasurementToken`
- `MeasurementValue`
- `MeasurementConstraint`
- `MosaicMeasurementAuthority`
- `MosaicViewportConstraint`
- `MosaicScrollOwnership`
- `RawLayoutMeasurementForDiagnostics`
- `RawLayoutMeasurementKind`

Current declaration-posture and support anchors:

- `UiDeclaredMeasurementPolicyPosture`
  in `worth-ui-runtime/src/declaration/declared_posture/measurement_policy_posture.rs`
- `admit_declared_posture_contract(...)`
  in `worth-ui-runtime/src/declaration/declared_posture/admission.rs`
- `UiDeclarationSupportRow`
  and `declared_measurement_policy_posture()`
  in `worth-ui-runtime/src/declaration/support/support_row.rs`
- `UiDeclarationSupportRowSchemaKind::MeasurementPolicy`
  in `worth-ui-runtime/src/declaration/support/schema_kind.rs`
- `WorthUiApp::inspection_support_report(...)`
  and `WorthUiApp::inspection_support_report_for(...)`

Current inspection anchors:

- `UiInspectionScope::measurement()`
- `UiInspectionQuery`
- `UiInspectionReceipt`
- `UiEvidenceSlice`
- `UiInspectionSupportReport`
- measurement-scope support projection in
  `worth-ui-runtime/src/facade/app_inspection_support.rs`

Current host-boundary anchors:

- `worth-ui-host-contract::WorthUiHostCapability`
- host-capability posture admission inside
  `worth-ui-runtime/src/declaration/declared_posture/admission.rs`
- current runtime host seam in `worth-ui-runtime/src/host/mod.rs`

Current Query-owned anchors this milestone must consume rather than replace:

- `ResolvedSnapshotBasis`
- `SnapshotResolutionReport`
- `workspace.inspect(...)`
- `WORTHQueryReadResult::consume_projection_facts(...)`
- `WORTHQueryWriteReceipt::consume_projection_facts(...)`
- `QueryContextExecutionArtifact::consume_projection_facts(...)`
- `admit_causal_inspection`
- `request_causal_inspection`

Operationally, 3.6a should extend these existing seams first. New type names
such as `UiMeasurementRequest` and `UiMeasurementResult` are justified only
when they sit cleanly between these anchors instead of bypassing them.

## Required Artifact Shapes

3.6a should not rely on an abstract "basis artifact" with unspecified shape.
The runtime needs one explicit artifact family that later phases can consume
without reinterpretation.

`UiMeasurementBasis` should minimally preserve:

- `basis_identity`
- `basis_generation`
- `declaration_identity`
- `graph_node_identity`
- `world_profile`
- `support_snapshot_identity`
- `host_capability_profile_identity`
- `query_basis_identity`
- `measurement_mode_posture`
- `constraint_modifier_posture`
- `basis_source_posture`
- `ownership_lifecycle_posture`
- `evidence_requirement_posture`
- `evidence_inputs`
- `generation_compatibility`
- `dependency_lineage`
- `neighborhood_class_hint`
- `denial_posture`

`MeasurementEvidenceInput` should be a typed input-ref family rather than an
anonymous tuple. It should minimally preserve:

- `source_family`
  - `declaration_posture`
  - `query_projection_fact`
  - `host_measurement_result`
  - `viewport_observation`
  - `host_capability_report`
- `input_identity`
- `input_generation`
- `evidence_category`
- `unit_posture`
- `coordinate_space`
- `rounding_posture`

`UiMeasurementGenerationCompatibility` should be a named compatibility posture
rather than a generic stale/error branch. It should minimally distinguish:

- `compatible`
- `stale_declaration_support`
- `stale_query_basis`
- `stale_query_fact_receipt`
- `stale_host_capability`
- `stale_host_evidence`
- `stale_viewport_evidence`
- `incompatible_world`
- `incompatible_host_profile`

These shapes are milestone contracts, not implementation suggestions. Field
spelling may change, but the semantic payload may not collapse.

## Implementation Map

The implementation path for 3.6a should be explicit:

1. Extend declaration-time measurement posture and support admission.
2. Consume the 3.4 `measurement-requirement` obligation as the entry path into
   the measurement lane rather than creating a parallel admission trigger.
3. Admit declaration measurement modes by world and host capability before
   any evidence exchange begins.
4. Admit Query-backed measurement posture and fact eligibility before ordinary
   measurement consumption begins.
5. Introduce a dedicated semantic-measurement runtime home that is distinct
   from `runtime/measurement/` performance counters.
6. Add host measurement request contracts at the host-contract boundary.
7. Add host evidence normalization and invalidation rules at the runtime intake
   seam.
8. Add Query-fact consumption adapters in `worth-ui-query-binding` for
   measurement-required projected facts.
9. Assemble one deterministic measurement basis artifact from:
   declaration posture + world/support posture + Query basis/facts + host
   evidence.
10. Freeze dependency lineage and neighborhood classes before 3.6b planning and
   3.8 churn work broaden the surface.
11. Route basis inspection, denial posture, and certification through the
    existing inspection/support lanes and the certification seam before 3.6b
    consumes the result.

If implementation begins with allocation planners, viewport churn handlers, or
scroll/portal interaction loops before these eleven steps exist, the milestone
has started in the wrong place.

## Phase Plan

### Phase 1: Freeze Measurement As A Declaration-Owned Semantic Lane

This phase freezes what declaration-time measurement meaning is allowed to
exist before any host evidence or allocation planning broadens it.

**Relevant subsystems**
- `worth-ui-dsl`
- `worth-ui-runtime/declaration`
- `worth-ui-runtime/capability`

**Relevant APIs**
- `admit_declared_posture_contract(...)`
- `UiDeclaredMeasurementPolicyPosture`
- `UiDeclarationSupportRow`
- `UiDeclarationSupportRowSchemaKind::MeasurementPolicy`
- `UiDeclarationSupportRow::declared_measurement_policy_posture()`
- `UiInspectionScope::measurement()`
- `NamedMeasurementDefinition`
- `MeasurementValue`
- `MeasurementConstraint`
- `MosaicMeasurementAuthority`
- declaration support posture APIs such as
  `declared_measurement_policy_posture()`

**Warnings**
- Do not let measurement meaning hide inside widget kind, renderer branches, or
  local mosaic helpers.
- Do not let raw numeric dimensions count as canonical measurement semantics.
- Do not let declaration syntax outrun admitted runtime modes.

**Test requirements**
- Adversarial equivalence test: semantically equivalent declaration inputs for
  measurement posture must converge on the same admitted canonical
  measurement-policy meaning.
- Adversarial rejection test: unsupported or malformed measurement declarations
  must deny through typed posture and diagnostics rather than falling through to
  host-local best effort.

**Engineering decisions**
- Freeze one closed 3.6a measurement vocabulary surface for admitted planning
  modes.
- Keep `NamedMeasurementDefinition` and measurement authority descriptors as
  declaration-time artifacts, not allocation outputs.
- Preserve explicit measurement support rows on declarations so measurement
  support remains machine-checkable before execution.
- Treat the current admitted posture tokens
  (`measurement:hug-height`, `measurement:font-metrics-required`) as the seed
  lane to broaden, not as proof that the full 3.6a vocabulary already exists.

**Open questions**
- None.

### Phase 2: Define The Admitted Measurement Vocabulary And Its Semantic Axes

This phase turns roadmap bullet names into real runtime semantics instead of
leaving them as decorative labels.

**Relevant subsystems**
- `worth-ui-runtime/declaration`
- `worth-ui-runtime/capability`
- `worth-ui-runtime/evidence`

**Relevant APIs**
- measurement vocabulary admitted for the roadmap:
  `available-space`, `fixed`, `hug`, `fill`, `equal-share`, `min`, `max`,
  `bounded`, `content-measured`, `viewport-relative`, `scroll-owned`,
  `portal-anchored`
- declaration support and inspection projections for measurement

**Warnings**
- Do not define vocabulary by analogy to DOM percentage/overflow behavior.
- Do not let `scroll-owned` mean both planning-time ownership and later
  continuous extent churn.
- Do not collapse `content-measured` and `host-measured` into the same
  semantic bucket.

**Test requirements**
- Adversarial convergence test: equivalent authored forms of the same
  measurement mode must produce the same semantic mode, bounds posture, and
  authority classification.
- Adversarial denial test: contradictory vocabulary combinations such as
  incompatible bounds or impossible ownership mixtures must fail before later
  planning.

**Engineering decisions**
- Define the measurement vocabulary as explicit axes rather than one flat list:
  - measurement mode:
    `fixed`, `hug`, `fill`, `equal-share`, `content-measured`
  - constraint modifier:
    `min`, `max`, `bounded`
  - basis source:
    `available-space`, `viewport-relative`, `intrinsic-content`,
    `query-content`, `host-text-metrics`, `portal-anchor`,
    `scroll-container`
  - ownership/lifecycle posture:
    `scroll-container-basis`, `scroll-viewport-basis`,
    `scroll-content-extent-required`, `portal-anchor-measurement-basis-required`,
    `viewport-derived`
  - evidence requirement:
    `no-host-evidence`, `host-text-measurement-required`,
    `host-viewport-evidence-required`, `query-fact-required`,
    `portal-anchor-evidence-required`
- Separate planning-time mode meaning from later churn behavior.
- Make ownership axes explicit: intrinsic-content, available-space,
  viewport-derived, scroll-owned, and portal-anchor-derived are distinct.
- Broaden the declaration admission path in
  `declaration/declared_posture/admission.rs` from the current token pair into
  a typed admitted vocabulary instead of creating a parallel parser or
  host-local registry.
- Treat `scroll-owned` and `portal-anchored` as legacy roadmap labels that
  must lower into narrower 3.6a planning-time posture names rather than
  remaining broad semantic buckets.

**Open questions**
- None.

### Phase 3: Admit Measurement By World And Host Capability

This phase closes the non-Query admission boundary for measurement so world and
capability support claims stop being implicit before Query-backed content joins
the lane.

**Relevant subsystems**
- `worth-ui-runtime/admission`
- `worth-ui-runtime/declaration`
- `worth-ui-runtime/capability`

**Relevant APIs**
- `UiAdmissionReport`
- `WorthUiApp::inspection_support_report(...)`
- `WorthUiApp::inspection_support_report_for(...)`
- declaration support snapshot/report projection surfaces
- `worth-ui-host-contract::WorthUiHostCapability`

**Warnings**
- Do not assume a measurement mode is supported in every world because it is
  declared.
- Do not let host capability gaps silently degrade into renderer-local layout.
- Do not treat stale declaration support posture or stale capability posture as
  "close enough" for measurement admission.

**Test requirements**
- Adversarial parity test: the same declaration measured in two identical world
  and capability profiles must admit identically.
- Adversarial denial test: wrong-world, missing-host-capability, or
  unsupported measurement cases must produce typed measurement denial rather
  than fallback sizing.
- Adversarial staleness test: when the declaration support generation or host
  capability posture changes, older admitted posture must not be silently
  reused as if it were still current.

**Engineering decisions**
- Make the 3.4 path explicit:
  `UiGraphTouchDescriptor`
  -> selected `measurement-requirement` obligation
  -> measurement support/admission posture
  -> measurement request/evidence/basis assembly.
- Keep measurement admission world-aware and host-capability-aware.
- Preserve distinct denial classes for unsupported, wrong-world,
  stale-support-posture, and capability-gated measurement.
- Require admission outputs to carry enough identity and generation posture so
  later phases can reject stale support or capability mixing deterministically.

**Open questions**
- None.

### Phase 4: Admit Query Measurement Posture And Fact Eligibility

This phase closes the Query-side admission boundary so Query-backed measurement
depends on admitted basis posture and typed fact families rather than informal
content reads.

**Relevant subsystems**
- `worth-ui-query-binding`
- `worth-ui-runtime/admission`
- `worth-ui-runtime/evidence`

**Relevant APIs**
- `ResolvedSnapshotBasis`
- `SnapshotResolutionReport`
- `WORTHQueryReadResult::consume_projection_facts(...)`
- `WORTHQueryWriteReceipt::consume_projection_facts(...)`
- `QueryContextExecutionArtifact::consume_projection_facts(...)`
- `workspace.inspect(...)`
- `admit_causal_inspection`
- `request_causal_inspection`

**Warnings**
- Do not let Query-backed content measurement bypass Query basis admission.
- Do not consume projected facts for measurement without preserving which
  declaration or measurement mode depended on which fact family.
- Do not treat retained inspection as a substitute for ordinary projection-fact
  admission.

**Test requirements**
- Adversarial convergence test: equivalent Query-backed measurement inputs
  reached through the ordinary binding/consumption path must yield the same
  typed fact-eligibility posture for later basis assembly.
- Adversarial denial test: unsupported Query posture, stale basis generation,
  or unavailable projected fact families must deny before basis assembly rather
  than widening into best-effort content sizing.
- Adversarial dependency test: when projected facts change outside a
  declaration's admitted measurement dependency set, Query measurement
  eligibility must stay local or remain unchanged.

**Engineering decisions**
- Consume Query basis posture through Query-owned artifacts rather than raw
  branch/snapshot strings.
- Separate "Query-backed measurement dependency is admitted in principle" from
  "the needed Query fact receipts were consumed for this declaration
  generation."
- Preserve `workspace.inspect(...)`, `admit_causal_inspection`, and
  `request_causal_inspection` for retained/cross-runtime explanation rather
  than the ordinary measurement input path.

**Open questions**
- None.

### Phase 5: Define Host Measurement Request Contracts

This phase freezes what Worth UI is allowed to ask the host for, before any
returned evidence is normalized or admitted into basis assembly.

**Relevant subsystems**
- `worth-ui-host-contract`
- `worth-ui-runtime/host`
- `worth-ui-runtime/evidence`

**Relevant APIs**
- `UiMeasurementRequest`
- `UiHostObservation`
- `worth-ui-host-contract::WorthUiHostCapability`
- current runtime host seam in `worth-ui-runtime/src/host/mod.rs`
- closed request-family posture for 3.6a:
  - `text_intrinsic_size`
  - `text_baseline_metrics`
  - `font_metrics`
  - `native_control_intrinsic_size`
  - `viewport_extent`
  - `dpi_scale_factor`
  - `portal_anchor_rect`
  - `scroll_container_viewport`

**Warnings**
- Do not let host request types smuggle layout strategy, overflow semantics, or
  final sizing decisions across the host boundary.
- Do not conflate viewport requests, intrinsic text measurement requests,
  native control sizing requests, and portal-anchor observation requests.
- Do not hide capability requirements inside request payload folklore.
- Do not admit host request families for forbidden authority asks such as:
  `final_layout_size`, `overflow_decision`, `scroll_extent_authority`,
  `portal_position_decision`, or `allocation_box`.

**Test requirements**
- Adversarial equivalence test: semantically equivalent runtime needs must
  produce equivalent `UiMeasurementRequest` shapes regardless of adapter or
  call path.
- Adversarial rejection test: requests that require unavailable host
  capabilities or mix incompatible evidence families must fail through typed
  request/admission posture rather than reaching the adapter.
- Adversarial boundary test: external callers must be unable to construct a
  request that already encodes final layout meaning.

**Engineering decisions**
- Model host participation first as a typed request contract, not merely as a
  loosely shaped adapter callback.
- Require request identity, evidence family, and capability posture to be
  explicit before host execution begins.
- Keep host-neutral request meaning in `worth-ui-host-contract`; adapter
  translation stays in `worth-ui-host-*`.
- Close the first request taxonomy in 3.6a so later milestones extend an
  admitted family rather than smuggling new host authority in as payload shape.

**Open questions**
- None.

### Phase 6: Normalize Host Measurement Evidence And Invalidation

This phase freezes how host-returned observations become normalized runtime
evidence and how that evidence becomes stale.

**Relevant subsystems**
- `worth-ui-host-contract`
- `worth-ui-runtime/host`
- `worth-ui-runtime/evidence`

**Relevant APIs**
- `UiMeasurementRequest`
- `UiMeasurementResult`
- `UiHostObservation`
- current runtime host seam in `worth-ui-runtime/src/host/mod.rs`

**Warnings**
- Do not let host adapters decide final constraints, overflow ownership, or
  region strategy.
- Do not let host measurement caches become authoritative truth.
- Do not let egui-specific measurement quirks leak into public runtime meaning.
- Do not let viewport samples, text metrics, or intrinsic-size samples outlive
  the capability/profile generation under which they were collected.

**Test requirements**
- Adversarial equivalence test: equivalent host evidence returned for the same
  request must converge on the same normalized runtime evidence artifact.
- Adversarial anti-bypass test: hostile host adapters must be unable to smuggle
  final layout decisions back as if they were measurement facts.
- Adversarial invalidation test: host evidence captured before a capability,
  viewport, DPI, font, or adapter-profile change must not remain valid unless
  the runtime can prove the request/evidence contract still matches.

**Engineering decisions**
- Separate request issuance from evidence normalization so transport and
  semantic intake remain different responsibilities.
- Separate viewport evidence, text/intrinsic evidence, native sizing
  observations, and portal-anchor observations into typed evidence categories
  even if they share transport machinery.
- Require `UiMeasurementResult` to preserve request identity, evidence
  category, and evidence generation so basis assembly can audit staleness
  instead of trusting timing.
- Preserve unit, coordinate-space, and rounding posture on normalized host
  evidence. 3.6a should minimally distinguish:
  - units: `logical_px`, `physical_px`
  - coordinate spaces: `viewport`, `window`, `graph_node_local`,
    `host_surface`, `portal_layer`
  - rounding posture: `exact_float`, `host_rounded`, `runtime_rounded`,
    `deferred_to_allocation`

**Open questions**
- None.

### Phase 7: Consume Query Projection Facts For Measurement Without Reopening Query Authority

This phase closes the ordinary downstream-runtime boundary where measurement
depends on projected content shape, label text, collection size, or state
posture.

**Relevant subsystems**
- `worth-ui-query-binding`
- `worth-ui-runtime/declaration`
- `worth-ui-runtime/evidence`

**Relevant APIs**
- `WORTHQueryReadResult::consume_projection_facts(...)`
- `WORTHQueryWriteReceipt::consume_projection_facts(...)`
- `QueryContextExecutionArtifact::consume_projection_facts(...)`
- `UiProjectionBinding`
- `UiProjectionFactReceipt`
- `workspace.inspect(...)`
- `admit_causal_inspection`
- `request_causal_inspection`

**Warnings**
- Do not rebuild local Query caches for content measurement.
- Do not use cross-runtime causal inspection as the ordinary path for
  measurement inputs.
- Do not confuse retained per-target Query inspection with projection-fact
  consumption.
- Do not consume projected facts for measurement without preserving which
  declaration or measurement mode depended on which fact family.

**Test requirements**
- Adversarial convergence test: equivalent Query-backed measurement inputs
  reached through the ordinary binding/consumption path must yield the same
  typed fact receipts for measurement basis assembly.
- Adversarial boundary test: Worth UI measurement code must not read raw Query
  internals or invent UI-local query result-state just to size content.
- Adversarial dependency test: when projected facts change outside a
  declaration's declared measurement dependency set, measurement basis
  invalidation must stay local or remain unchanged.

**Engineering decisions**
- Consume projected measurement inputs through `consume_projection_facts(...)`
  on the ordinary path.
- Use `workspace.inspect(...)` only for retained per-target evidence, not as a
  substitute for projection consumption.
- Preserve typed dependency identity from Query fact consumption so later
  basis assembly and neighborhood planning do not have to rediscover the
  dependency set heuristically.

**Open questions**
- None.

### Phase 8: Assemble A Deterministic Measurement Basis Artifact

This phase creates the runtime-owned artifact that later allocation planning
consumes.

**Relevant subsystems**
- `worth-ui-runtime/evidence`
- `worth-ui-runtime/declaration`
- `worth-ui-runtime/host`
- `worth-ui-query-binding`

**Relevant APIs**
- declaration measurement posture artifacts
- `UiMeasurementRequest`
- `UiMeasurementResult`
- Query fact receipts consumed for measurement
- `UiInspectionReceipt` and measurement-scope inspection projections
- `UiMeasurementBasis`
- `MeasurementEvidenceInput`
- `UiMeasurementGenerationCompatibility`

**Warnings**
- Do not let later allocation planning rediscover declaration posture or Query
  facts from scratch.
- Do not mix host evidence and declaration semantics into anonymous tuples or
  helper structs with no semantic role.
- Do not let the basis artifact masquerade as a committed allocation receipt.
- Do not let one changed input force total-basis invalidation unless the typed
  dependency graph actually widened.

**Test requirements**
- Adversarial determinism test: equivalent declaration authority, Query fact
  receipts, world posture, host capability profile, and host evidence must
  produce equivalent measurement basis artifacts.
- Adversarial generation test: when one measurement input changes, the new
  basis generation must change only where the typed dependency set changed.
- Adversarial contradiction test: impossible combinations such as
  viewport-relative measurement without viewport evidence, portal-anchored
  measurement without anchor evidence, or content-measured posture without the
  required Query or host evidence must produce typed basis denial rather than a
  partial basis with hidden holes.

**Engineering decisions**
- Introduce one explicit measurement basis artifact between evidence exchange
  and later allocation planning.
- Bind the basis to declaration identity, world profile, Query basis posture,
  and host evidence generation.
- Keep the basis derived and reproducible from authority plus evidence.
- Require `UiMeasurementBasis` to preserve explicit identity, generation, input
  refs, compatibility posture, dependency lineage, neighborhood-class hint,
  and denial posture rather than collapsing into a helper struct that later
  phases widen opportunistically.

**Open questions**
- None.

### Phase 9: Preserve Measurement Dependency Lineage

This phase freezes how a measurement basis remembers what it depended on before
the later replanning and churn milestones begin consuming it.

**Relevant subsystems**
- `worth-ui-runtime/evidence`
- `worth-ui-runtime/graph`
- `worth-ui-query-binding`
- `worth-ui-runtime/host`

**Relevant APIs**
- measurement basis artifact surfaces introduced in Phase 8
- `UiProjectionFactReceipt`
- `UiMeasurementResult`
- `UiGraphTouchDescriptor`

**Warnings**
- Do not force later planning to rediscover dependency sets by scanning
  declarations, Query receipts, or host evidence again.
- Do not flatten viewport, content, portal-anchor, scroll-container, and
  intrinsic-text dependencies into one generic "measurement changed" marker.
- Do not let dependency lineage masquerade as a committed receipt or mounted
  geometry artifact.

**Test requirements**
- Adversarial lineage test: equivalent authority and evidence inputs must
  produce equivalent dependency lineage inside the basis artifact.
- Adversarial narrowness test: changing one dependency family must only taint
  the lineage slices that actually depended on it.
- Adversarial replay test: rebuilding basis lineage from authority plus
  evidence must reproduce the same dependency structure without consulting
  later allocation or receipt artifacts.

**Engineering decisions**
- Require the basis artifact to preserve typed dependency lineage for viewport,
  content, portal-anchor, scroll-container, and intrinsic-text evidence.
- Keep lineage as a derived proof attached to basis assembly rather than as a
  separate heuristic index reconstructed later.
- Separate lineage identity from neighborhood classification so later phases can
  consume one without redefining the other.

**Open questions**
- None.

### Phase 10: Define Measurement Neighborhood Classes For Future Replanning

This phase freezes how measurement dependencies are grouped so later resize,
scroll, and drag work has a bounded planning neighborhood to inherit.

**Relevant subsystems**
- `worth-ui-runtime/graph`
- `worth-ui-runtime/obligations`
- `worth-ui-runtime/evidence`

**Relevant APIs**
- `UiGraphTouchDescriptor`
- graph participation/index surfaces
- measurement support and evidence projections

**Warnings**
- Do not equate measurement neighborhood with the whole page or whole mosaic.
- Do not leave neighborhood semantics implicit until the resize milestone.
- Do not confuse structural topology neighborhoods with later mounted receipt
  churn neighborhoods.
- Do not let viewport-relative, scroll-owned, portal-anchored, and
  drag-adjacent surfaces all collapse into the same invalidation radius.

**Test requirements**
- Adversarial localization test: a local measurement-input change must resolve
  to a typed local neighborhood rather than an unconditional broad replan set.
- Adversarial separation test: appearance-only or Query-only changes that do
  not affect declared measurement dependencies must not enter the measurement
  neighborhood.
- Adversarial pressure test: viewport resize, scroll-container extent change,
  portal-anchor movement, and splitter-drag inputs must each map to the
  narrowest typed neighborhood class that can explain their future replanning
  fallout.

**Engineering decisions**
- Define neighborhood classes only as lineage-derived hints over graph identity
  and declared consumed facts now, before continuous interaction broadens the
  system.
- Keep neighborhood derivation separate from actual replanning execution.
- Preserve enough structure that later viewport resize, splitter drag, and
  scroll extent work can stay bounded.
- Freeze at least distinct neighborhood classes for local intrinsic-content
  dependency, container-available-space dependency, viewport dependency,
  scroll-container dependency, and portal-anchor dependency even though 3.6a
  does not yet execute continuous churn.
- Treat 3.6a neighborhood output as basis metadata only. 3.6b owns turning
  those classes into actual allocation neighborhoods and constraint
  propagation.

**Open questions**
- None.

### Phase 11: Make Measurement Inspection And Diagnostics First-Class

This phase ensures measurement is inspectable through the same evidence
substrate already established in 3.5.

**Relevant subsystems**
- `worth-ui-inspection`
- `worth-ui-runtime/evidence`
- `worth-ui-runtime/declaration`

**Relevant APIs**
- `UiInspectionScope::Measurement`
- `UiInspectionQuery`
- `UiEvidenceSlice`
- `UiInspectionReceipt`
- measurement support reports and closure reports
- `WorthUiApp::inspection_support_report(...)`
- `WorthUiApp::inspection_support_report_for(...)`

**Warnings**
- Do not build a layout debugger that bypasses typed evidence.
- Do not flatten unsupported, denied, stale, contradictory, and
  capability-gated measurement into one message.
- Do not wait for mounted receipts before making measurement denial inspectable.
- Do not make operators infer whether a failure came from declaration posture,
  Query facts, host evidence, or basis-generation mismatch.

**Test requirements**
- Adversarial parity test: measurement inspection queries through the public
  inspection facade must converge on the same basis/evidence answers as direct
  runtime inspection assembly.
- Adversarial narrowness test: measurement inspection for one declaration or
  graph node must not widen into whole-frame explanation.

**Engineering decisions**
- Extend the existing inspection substrate rather than adding a separate layout
  inspection path.
- Preserve support-report and closure-report posture around richer measurement
  evidence.
- Make measurement evidence expandable through `UiEvidenceSlice` instead of
  defaulting to giant detail bundles.
- Ensure inspection can surface dependency lineage and generation mismatch in a
  typed way so operators can tell whether resize/scroll/drag fallout is a real
  semantic dependency or just stale evidence.

**Open questions**
- None.

### Phase 12: Certify Host Purity, Basis Determinism, And Future-Growth Posture

This phase closes the milestone by proving 3.6a is a real architecture seam,
not just preliminary plumbing.

**Relevant subsystems**
- `worth-ui-certification`
- `worth-ui-runtime`
- `worth-ui-host-contract`
- `worth-ui-inspection`

**Relevant APIs**
- certification topology audits
- measurement inspection and support facades
- host evidence exchange surfaces

**Warnings**
- Do not certify only happy-path measurement.
- Do not close the milestone if future 3.6b/3.8 work would still need to
  redefine basic vocabulary or boundary ownership.
- Do not let host adapters or measurement helpers leak deep imports across the
  intended seam.

**Test requirements**
- Adversarial anti-bypass test: host adapters and external callers must be
  unable to manufacture measurement authority, committed allocation truth, or
  Query basis authority through local helpers.
- Adversarial growth-path test: adding future allocation-planning or
  continuous-interaction families must have an obvious structural home without
  widening the 3.6a basis artifact into a god object.

**Engineering decisions**
- Add certification for vocabulary closure, host-boundary purity, basis
  determinism, and measurement inspection narrowness.
- Treat 3.6a as closed only when 3.6b and 3.8 have an obvious consumer path
  from the shipped artifacts.
- Keep measurement proof machine-checkable rather than narrative-only.

**Open questions**
- None.

## Must Ship

- `milestone-3.6a.md` as the measurement-foundation spec
- roadmap sequencing that splits old 3.6 scope into at least:
  - `3.6a` measurement vocabulary, basis admission, and host evidence
  - `3.6b` allocation neighborhood planning and constraint propagation
- a closed 3.6a measurement vocabulary taxonomy with explicit axes for:
  - measurement mode
  - constraint modifier
  - basis source
  - ownership/lifecycle posture
  - evidence requirement
- admitted 3.6a planning-time vocabulary mapped from the roadmap labels for:
  - `fixed`
  - `hug`
  - `fill`
  - `equal-share`
  - `content-measured`
  - `min`
  - `max`
  - `bounded`
  - `available-space`
  - `viewport-relative`
  - `scroll-container-basis`
  - `scroll-viewport-basis`
  - `scroll-content-extent-required`
  - `portal-anchor-measurement-basis-required`
- explicit 3.4 entry posture:
  - `UiGraphTouchDescriptor`
  - selected `measurement-requirement` obligation
  - measurement support/admission posture
- typed admission posture for measurement by world, host capability, and
  Query-backed content posture
- closed host request-family posture for:
  - `text_intrinsic_size`
  - `text_baseline_metrics`
  - `font_metrics`
  - `native_control_intrinsic_size`
  - `viewport_extent`
  - `dpi_scale_factor`
  - `portal_anchor_rect`
  - `scroll_container_viewport`
- explicit forbidden host-authority request families for:
  - `final_layout_size`
  - `overflow_decision`
  - `scroll_extent_authority`
  - `portal_position_decision`
  - `allocation_box`
- typed host measurement evidence exchange through:
  - `UiMeasurementRequest`
  - `UiMeasurementResult`
  - host observation / host contract evidence lanes
- `UiMeasurementBasis` with explicit:
  - identity
  - generation
  - input refs
  - compatibility posture
  - dependency lineage
  - neighborhood-class hint
  - denial posture
- `MeasurementEvidenceInput` with explicit typed source family, input identity,
  input generation, evidence category, unit posture, coordinate space, and
  rounding posture
- `UiMeasurementGenerationCompatibility` distinguishing:
  - `compatible`
  - `stale_declaration_support`
  - `stale_query_basis`
  - `stale_query_fact_receipt`
  - `stale_host_capability`
  - `stale_host_evidence`
  - `stale_viewport_evidence`
  - `incompatible_world`
  - `incompatible_host_profile`
- typed staleness and denial posture covering:
  - unsupported vocabulary
  - wrong-world admission
  - missing host capability
  - stale Query basis or stale support posture
  - contradictory or partial basis inputs
- a deterministic measurement basis artifact that later 3.6b planning consumes
- typed dependency lineage inside the basis artifact for:
  - viewport-derived evidence
  - content/query-derived evidence
  - portal-anchor-derived evidence
  - scroll-container-derived evidence
  - intrinsic text/native measurement evidence
- explicit Query integration rules naming:
  - `ResolvedSnapshotBasis`
  - `SnapshotResolutionReport`
  - `workspace.inspect(...)`
  - `consume_projection_facts(...)`
  - `admit_causal_inspection`
  - `request_causal_inspection`
- measurement-scope inspection and diagnostics through the shared evidence
  substrate
- certification proving host purity, basis determinism, and anti-bypass
  boundaries

## Must Preserve

- declaration meaning remains the source of measurement semantics
- Worth UI remains the owner of UI measurement meaning
- Query remains the owner of basis, retained inspection, projection-fact
  consumption, and cross-runtime causal explanation
- host adapters remain evidence suppliers, not layout semantic owners
- measurement basis remains distinct from committed allocation receipts
- scroll and portal semantics stay narrowed to planning-time basis meaning in
  3.6a rather than broad behavior ownership
- unit, coordinate-space, and rounding posture remain explicit wherever host
  evidence could otherwise look equivalent
- inspection remains one shared substrate for AI and human consumers
- future 3.6b and 3.8 work can build on shipped artifacts instead of reopening
  vocabulary or boundary ownership

## Acceptance Evidence

- equivalent declaration + world + host capability + Query basis + host
  evidence inputs converge to the same measurement basis
- unsupported, contradictory, wrong-world, or capability-gated measurement
  modes deny through typed posture rather than heuristic fallback
- host supplies measurement evidence only and cannot decide final layout
  meaning
- Query-backed measurement inputs are consumed through typed Query lanes rather
  than local caches or raw lower-runtime imports
- equivalent basis inputs with the same unit, coordinate-space, and rounding
  posture converge to the same basis, while mismatched posture denies or stays
  explicitly distinguishable
- measurement inspection can explain support posture, denial posture, and basis
  inputs without mounted receipts or renderer-local helpers
- measurement neighborhoods are typed narrowly enough that later resize/drag
  work can stay bounded
- viewport resize, scroll extent change, portal-anchor movement, and splitter
  drag can each be invalidated through typed dependency lineage rather than
  broad whole-graph recomputation
- certification proves no host, facade, or test helper can mint measurement
  authority or Query basis authority locally

## Sequencing Notes

- 3.6a belongs after 3.5 because measurement needs the shared evidence and
  inspection substrate before it can be explained honestly.
- 3.6a belongs before 3.6b because allocation-neighborhood planning should
  consume a frozen measurement basis rather than co-defining basic measurement
  semantics while planning.
- 3.6a belongs before 3.8 because continuous resize, scroll, portal, and drag
  churn should broaden a stable measurement kernel instead of forcing the kernel
  to form under churn pressure.
- 3.6a also belongs before later mounted-receipt and rebind milestones because
  those milestones need typed measurement inputs and measurement evidence rather
  than host folklore.
