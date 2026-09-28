# Milestone 3.6b Engineering Spec: Allocation Neighborhood Planning And Constraint Propagation

> **Status:** Planned
>
> **Roadmap parent:** [worth_ui_roadmap.md](./worth_ui_roadmap.md)
>
> **Primary prerequisite:** `Milestone 3.6a Measurement Vocabulary, Basis Admission, And Host Evidence Boundaries`
>
> **Follow-on sequence:** `Milestone 3.8 bounded measurement invalidation under resize, scroll, portal, and drag churn`
>
> **Primary architectural driver:** turn admitted measurement basis into a deterministic, bounded allocation-planning kernel before committed allocation receipts, resize streams, splitter drag loops, and scroll/portal churn broaden the runtime.

## Goal

Freeze the second half of Worth UI measurement as a runtime-owned planning lane.

Milestone 3.6b is complete when Worth UI can consume 3.6a
`UiMeasurementBasis` artifacts, construct explicit allocation neighborhoods,
propagate constraints through those neighborhoods deterministically, deny
contradictory or cyclic planning neighborhoods before host execution, and hand
one typed allocation plan forward into the existing execution-plan/runtime-host
pipeline without rediscovering declaration, Query, or host meaning.

This milestone closes the pre-receipt planning side of layout:

- what an allocation neighborhood is
- how parent, child, and sibling constraints propagate
- how equal-share and bounded reconciliation work
- how viewport-, scroll-, and portal-derived basis inputs enter planning
- how user-resizable structure becomes a planning input without making drag
  loops or host gestures the planner authority
- how a local measurement change maps to a local allocation-planning
  neighborhood instead of broad whole-page replanning

It does not close:

- committed `UiAllocationReceipt` or mounted geometry truth
- per-frame resize/drag/scroll churn handling
- drag-stream cadence or every-frame commit policy
- portal reposition loops
- scroll extent maintenance after planning
- hit testing
- paint-time visual geometry evaluation
- host-local gesture ownership

## Why This Milestone Exists

3.6a froze measurement vocabulary, support posture, host evidence intake, Query
fact eligibility, and `UiMeasurementBasis`. That gave the runtime a typed input
to planning, but not yet the planning kernel itself.

3.6b exists so later resize, splitter, scroll, and portal work do not
improvise allocation semantics in host adapters, mosaic widgets, or activation
helpers.

Without this slice, the runtime would still be missing the answers to the
hard questions:

- what is the bounded unit of allocation replanning?
- how do parent-to-child and child-to-parent constraints flow without hidden
  recursion folklore?
- when siblings negotiate width/height, where does that semantics live?
- how do equal-share, min/max, and fill/hug interact mechanically?
- how does viewport-relative, portal-anchor, or scroll-container basis alter
  planning without collapsing everything into one global layout pass?
- how does user-resizable structure participate in planning without making
  transient drag state authoritative?

If those remain implicit, later churn milestones will encode them in
gesture-local code, host widgets, or ad hoc layout helpers. That would leave
Worth UI with measurement artifacts but no actual architecture for using them.

## Governing Summaries

- `MENTALITY.md`
  protects adversarial-constraint-first design. 3.6b must be shaped by hostile
  resize/sibling/viewport/portal pressure, not by a static stack-layout demo.
- `arch_laws.md`
  protects authority versus derivation and proof-bearing phase transitions.
  3.6b must keep measurement basis, allocation neighborhood, allocation plan,
  and later receipts as separate artifacts.
- `composition_laws.md`
  protects named semantic steps. 3.6b must not collapse neighborhood
  construction, constraint propagation, denial, handle binding, and inspection
  into one planner blob.
- `domain_structure_laws.md`
  protects separate ownership for basis intake, neighborhood derivation,
  propagation, runtime handoff, and diagnostics. 3.6b must give each one a
  visible home.
- `perf_laws.md`
  protects bounded breadth and locality-first execution. 3.6b must make local
  allocation replanning structurally narrow rather than a later heuristic.
- `worth_ui_roadmap.md`
  protects the sequence. Allocation planning must consume frozen measurement
  basis before mounted receipts and continuous interaction churn broaden the
  runtime.
- `WORTH_UI_README.md`
  protects the real runtime stack. 3.6b must keep allocation meaning in
  Worth UI runtime, Query facts in Query-owned lanes, and host code as
  observation/mechanics only.
- `worth-ui-dsl-vision.md`
  protects explicit layout operators and runtime-owned measurement meaning.
  3.6b must give stack/row/grid/mosaic/split/scroll/portal authoring real
  planning semantics instead of parent magic.
- `ai-diagnostics.md`
  protects one shared evidence substrate. 3.6b allocation neighborhoods and
  denials must inspect through typed evidence, not a layout debugger side lane.
- `crates/worth-query/docs/AI_README.md`
  protects Query-owned basis, projection consumption, retained inspection, and
  cross-runtime causal explanation. 3.6b must consume `consume_projection_facts(...)`,
  `workspace.inspect(...)`, `ResolvedSnapshotBasis`, `SnapshotResolutionReport`,
  `admit_causal_inspection`, and `request_causal_inspection` only through the
  admitted Query/Worth binding seam when Query-backed content still participates
  in allocation planning.

## Adversarial Constraint

3.6b must survive this hostile condition:

> A running Worth UI app contains nested shell mosaics, local composition
> regions, scroll-owned inspectors, viewport-relative workbench areas,
> portal-anchored dropdowns, Query-backed content whose intrinsic size changes,
> and user-resizable splitters whose stable positions may carry forward across
> hot reload. Source edits, Query fact changes, viewport observations, portal
> anchor observations, and durable splitter-position state all arrive while the
> app remains live. The runtime must derive the same allocation neighborhood and
> the same constraint-propagation result from equivalent measurement basis
> inputs, must keep replanning local to the affected neighborhood, must deny
> contradictions before mounted receipt production, must preserve the
> safe-frame-boundary activation model, and must never let host gesture code,
> renderer branches, or transient drag state become the authority for layout
> meaning.

If equivalent declaration + graph + measurement basis inputs can produce
different allocation plans, if resize and sibling negotiation semantics only
exist in host code, if transient interaction state can silently become layout
truth, or if a local change still requires whole-page planning by default,
3.6b is not closed.

## Product Decision Lock

- Allocation planning is a runtime semantic lane, not a host helper and not a
  component-local layout routine.
- This is the first milestone where the DSL layout algebra must gain concrete
  runtime planning meaning. `stack`, `row`, `grid`, `split`, `mosaic`,
  `overlay`, `scroll`, and `portal_anchor` may differ in planning rules, but
  they may not each invent private allocation semantics outside the shared
  planning lane.
- 3.6b consumes 3.6a `UiMeasurementBasis`; it does not redefine measurement
  vocabulary, host evidence policy, or Query basis policy.
- Allocation neighborhood, allocation plan, execution plan, and mounted
  receipt are distinct artifacts:
  - neighborhood = bounded planning scope
  - allocation plan = solved planning result over that scope
  - execution plan = runtime-host/lane lowering artifact
  - mounted receipt = committed visible output truth
- Parent/child propagation, sibling negotiation, and bounded reconciliation
  must be explicit typed planning rules, not recursion hidden inside layout
  operators.
- Allocation neighborhoods may contain only nodes that are admitted as
  `participation.layout`. Hidden, paint-only, hit-test-only, or
  accessibility-only nodes may influence other runtime families, but they may
  not silently participate in allocation planning unless their declaration
  posture explicitly says they publish layout participation.
- User resize is not "drag state owns layout." Durable splitter position or
  similar state may participate as a planning input when admitted; transient
  interaction state remains non-authoritative.
- 3.6b closes deterministic planning semantics only. Continuous resize, drag,
  scroll, and portal motion churn remain a later milestone.
- The safe-frame-boundary activation model remains intact. Planning output may
  broaden what is activated, but not when activation may commit.
- Scroll-owned and portal-anchored planning remain narrowed to planning-time
  neighborhoods and constraints here. Continuous extent maintenance and anchor
  reposition loops remain out of scope.
- Repeated-instance and conditional-instance identity must remain canonical at
  the graph/declaration layer. 3.6b may not use sibling order, current
  visibility, or current slot position as a substitute identity when
  constructing neighborhoods or peer negotiation groups.
- Query-backed intrinsic content remains Query-consumed evidence in the plan
  input. Worth UI must not create UI-local content caches that outrun Query.
- Unsupported, cyclic, contradictory, stale, and underconstrained planning
  outcomes must remain distinct typed denials. 3.6b may not collapse them into
  a generic "layout failed" path.
- Allocation solving must preserve 3.6a unit, coordinate-space, and rounding
  posture. If two inputs look numerically similar but differ in those semantic
  postures, the planner must either normalize them explicitly or deny the
  combination rather than quietly merging them.

## Existing API Anchors

This milestone must build from the real seams already present in the repo.

Current measurement-basis and neighborhood anchors:

- `UiMeasurementBasis`
- `UiMeasurementBasisGeneration`
- `UiMeasurementBasisPosture`
- `UiMeasurementDependencyLineage`
- `UiMeasurementDependencyMap`
- `UiMeasurementNeighborhoodClassHint`
- `admit_measurement_basis(...)`
- `UiGraphMeasurementNeighborhoodHint::from_basis(...)`
- `UiGraphTouchMeasurementNeighborhoodHint::from_touch(...)`

Current declaration/admission/touch anchors:

- `UiDeclaredMeasurementPolicyPosture`
- `UiGraphTouchDescriptor`
- `UiSelectedObligationSet`
- `UiQueryMeasurementEligibility`
- `UiProjectionFactReceipt`
- `UiAdmissionBoundary::select_obligations(...)`
- `UiAdmissionBoundary::lower_obligation_dispatch(...)`
- `UiAdmissionBoundary::dispatch_selected_obligations(...)`
- `UiAdmissionBoundary::admit_selected_obligations(...)`
- `UiAdmissionBoundary::admit_measurement_requirement(...)`
- `UiAdmissionBoundary::admit_query_measurement_eligibility_for_touch_from_projection_consumption(...)`
- `UiAdmissionBoundary::admit_query_measurement_eligibility_from_projection_consumption(...)`

Current host / Query input anchors that planning must consume rather than
replace:

- `UiMeasurementRequest`
- `UiMeasurementResult`
- `freeze_measurement_request(...)`
- `request_host_measurement(...)`
- `collect_host_measurement_evidence(...)`
- `admit_current_host_measurement_evidence(...)`
- `normalize_host_measurement_evidence(...)`
- `invalidate_stale_host_measurement_evidence(...)`
- `workspace.inspect(...)`
- `consume_projection_facts(...)`
- `ResolvedSnapshotBasis`
- `SnapshotResolutionReport`
- `admit_causal_inspection`
- `request_causal_inspection`

Current runtime-plan / activation anchors that 3.6b must hand off into:

- `WorthUiRuntimeHandleAllocationBasis`
- `WorthUiRuntimeHandleAllocation`
- `WorthUiRuntimeHandleAllocationReceipt`
- `WorthUiRuntimeHost::admit_execution_lanes(...)`
- `WorthUiRuntimeHost::assemble_execution_plan_topology(...)`
- `WorthUiRuntimeHost::assemble_execution_plan_topology_with_lane_admission(...)`
- `WorthUiPlanTopologyAssembler::assemble_with_lane_admission(...)`
- `WorthUiExecutionPlan`
- `WorthUiPlanTopology`
- `WorthUiPlanLanePartition`
- `WorthUiPlanLookupIndex`
- `WorthUiPlanTopologyCounters`
- `WorthUiRuntimeHost::prepare_ready_activation(...)`
- `WorthUiRuntimeHost::activate_ready_at_frame_boundary(...)`
- `WorthUiRuntimeHost::swap_ready_activation_at_frame_boundary(...)`

Current resize/preservation anchors that planning must respect:

- `MosaicResizePermission`
- `WorthUiSplitterPositionReconciliation`
- `WorthUiDurableStateFamily::splitter_position()`
- `WorthUiTransientInteractionState`
- `WorthUiTransientInteractionPolicy`

Current evidence / inspection / certification anchors this milestone should
extend rather than bypass:

- `certify_measurement_basis_determinism(...)`
- `certify_measurement_basis_determinism_for_scenarios(...)`
- `UiMeasurementBasisCertificationScenario`
- `UiMeasurementBasisCertificationOutcome`
- `UiInspectionScope::Measurement`
- `UiInspectionReceipt`
- `UiEvidenceSlice`
- `project_measurement_inspection_view(...)`
- `project_measurement_inspection_denial_view(...)`

Operationally, 3.6b should extend these seams first. New names such as
`UiAllocationNeighborhood` and `UiAllocationPlan` are justified only when they
sit cleanly between the existing measurement basis and execution-plan/runtime
handoff boundaries.

## Required Artifact Shapes

3.6b should not rely on a generic "layout plan" with hidden semantics. It
needs explicit planning artifacts that later receipt and churn milestones can
consume honestly.

`UiAllocationNeighborhood` should minimally preserve:

- `layout_operator_contract_identity`
- `neighborhood_identity`
- `neighborhood_generation`
- `root_graph_node_identity`
- `world_profile`
- `measurement_basis_identity`
- `member_node_identities`
- `dependency_map`
- `neighborhood_class`
- `propagation_edges`
- `special_inputs`
- `durable_planning_inputs`
- `denial_posture`

`UiAllocationNeighborhoodIdentity` should minimally preserve:

- `root_graph_node_identity`
- `graph_generation`
- `measurement_basis_identity`
- `layout_operator_contract_identity`
- `neighborhood_class`
- `participating_member_identity_set`
- `dependency_map_digest`
- `world_profile`

`UiAllocationNeighborhoodIdentity` must explicitly exclude:

- current sibling order unless declaration/runtime semantics explicitly admit it
- current visibility unless layout participation depends on it
- host handles
- transient drag state
- solver execution order
- debug labels

`UiLayoutOperatorPlanningContract` should minimally preserve:

- `operator_identity`
- `operator_family`
- `primary_axis`
- `cross_axis`
- `child_participation_rule`
- `allowed_sizing_modes`
- `allowed_propagation_edges`
- `sibling_grouping_rule`
- `intrinsic_return_policy`
- `overflow_policy`
- `special_input_requirements`
- `denial_policy`

`UiAllocationConstraintSet` should minimally preserve:

- `incoming_available_space`
- `intrinsic_contribution_requirements`
- `sibling_negotiation_mode`
- `equal_share_group`
- `bounded_min_max_requirements`
- `viewport_requirement`
- `scroll_owner_requirement`
- `portal_anchor_requirement`
- `resize_permission_posture`
- `unit_posture`
- `coordinate_space`
- `rounding_posture`

`UiConstraintPropagationEdge` should minimally preserve:

- `edge_identity`
- `source_node_identity`
- `target_node_identity`
- `edge_family`
  - `parent_available_space`
  - `child_intrinsic_contribution`
  - `sibling_negotiation`
  - `equal_share_distribution`
  - `bounded_reconciliation`
  - `viewport_basis`
  - `scroll_owner_basis`
  - `portal_anchor_basis`
  - `durable_resize_input`
- `constraint_payload_digest`
- `cycle_participation_posture`

`UiAllocationSolvePlan` should minimally preserve:

- `solve_plan_identity`
- `allocation_neighborhood_identity`
- `solve_order`
- `solve_passes`
- `normalization_posture`
- `fixed_point_posture`
- `cycle_posture`
- `trace_digest`

`UiAllocationSolvePass` should minimally preserve:

- `pass_index`
- `pass_family`
- `input_edge_set_digest`
- `output_fact_digest`
- `denial_posture`

`UiAllocationSolveTrace` should minimally preserve:

- `solve_plan_identity`
- `pass_receipts`
- `edge_applications`
- `normalization_receipts`
- `remainder_distribution_receipts`
- `bound_reconciliation_receipts`
- `denial_or_convergence_outcome`

`UiConstraintCyclePosture` must distinguish at least:

- `acyclic`
- `admitted_fixed_point`
- `denied_cycle`
- `denied_unsupported_convergence`

`UiEqualShareDistributionPolicy` must distinguish at least:

- `exact_fractional`
- `deterministic_remainder_left_to_right_by_stable_peer_identity`
- `deterministic_remainder_center_out_by_stable_peer_identity`
- `deny_if_non_integral_required`

`UiBoundReconciliationPosture` must distinguish at least:

- `satisfied_without_clamp`
- `satisfied_with_declared_clamp`
- `underconstrained`
- `overconstrained`
- `contradictory_min_max`
- `unsupported_unit_mix`
- `unsupported_rounding_mix`

`UiAllocationPlan` should minimally preserve:

- `plan_identity`
- `plan_generation`
- `measurement_basis_identity`
- `allocation_neighborhood_identity`
- `graph_node_identity`
- `constraint_solution_posture`
- `planned_node_constraints`
- `planned_lane_partitioning_hint`
- `handle_allocation_basis_digest`
- `participation_basis`
- `normalization_posture`
- `denial_posture`

`UiAllocationPlan` may solve abstract constraints and lane-partitioning hints.
It may not claim final mounted boxes, visible geometry, hit-test regions,
committed scroll extents, or host paint geometry.

`UiAllocationPlanningCostReceipt` should minimally preserve:

- `neighborhood_identity`
- `nodes_considered`
- `nodes_admitted`
- `edges_emitted`
- `propagation_passes`
- `special_inputs_loaded`
- `query_fact_refs_consumed`
- `host_evidence_refs_consumed`
- `denied_broadening_reason`
- `cost_class`

`UiAllocationPlanningCostReceipt.cost_class` must distinguish at least:

- `local`
- `container`
- `viewport`
- `scroll_container`
- `portal_anchor`
- `durable_resize_group`
- `denied_unbounded`

These shapes are milestone contracts, not implementation suggestions. Field
spelling may change, but the semantic payload may not collapse.

## Operational Entry And Handoff Path

3.6b should be implemented on one explicit ordinary path rather than on
parallel helper seams.

The intended operational flow is:

1. A graph-touch enters through `UiGraphTouchDescriptor`.
2. Admission selects and lowers obligations through:
   - `UiAdmissionBoundary::select_obligations(...)`
   - `UiAdmissionBoundary::admit_measurement_requirement(...)`
   - `UiAdmissionBoundary::admit_query_measurement_eligibility_for_touch_from_projection_consumption(...)`
3. Query-backed content facts are admitted through
   `consume_projection_facts(...)` and materialized as `UiProjectionFactReceipt`.
4. Host evidence is admitted through:
   - `freeze_measurement_request(...)`
   - `request_host_measurement(...)`
   - `collect_host_measurement_evidence(...)`
   - `admit_current_host_measurement_evidence(...)`
5. Basis assembly remains the sole ordinary planner input through
   `admit_measurement_basis(...)`.
6. 3.6b derives one allocation neighborhood and one allocation plan from that
   basis.
7. The solved plan binds into runtime lowering through:
   - `WorthUiRuntimeHost::admit_execution_lanes(...)`
   - `WorthUiRuntimeHandleAllocationBasis`
   - `WorthUiRuntimeHandleAllocation`
   - `WorthUiRuntimeHost::assemble_execution_plan_topology_with_lane_admission(...)`
8. Candidate activation remains gated through:
   - `WorthUiRuntimeHost::prepare_ready_activation(...)`
   - `WorthUiRuntimeHost::activate_ready_at_frame_boundary(...)`
   - `WorthUiRuntimeHost::swap_ready_activation_at_frame_boundary(...)`
9. Evidence, denial posture, and certification project through:
   - `UiInspectionScope::Measurement`
   - `UiInspectionReceipt`
   - `UiEvidenceSlice`
   - `certify_measurement_basis_determinism(...)`
   - `certify_measurement_basis_determinism_for_scenarios(...)`

Any implementation path that skips this chain and instead mutates mounted
state directly, reads host helpers as authority, or makes topology assembly
re-solve layout semantics is out of bounds for 3.6b.

## Implementation Map

The implementation path for 3.6b should be explicit:

1. Consume admitted `UiMeasurementBasis` and `UiGraphMeasurementNeighborhoodHint`
   instead of rediscovering dependencies from declarations.
2. Freeze `UiLayoutOperatorPlanningContract` so stack/row/grid/split/mosaic/
   overlay/scroll/portal-anchor families publish explicit planning contracts
   rather than private helper semantics.
3. Freeze one explicit allocation-neighborhood artifact between basis intake
   and execution-plan lowering.
4. Admit parent/child, child/parent, sibling negotiation, equal-share, and
   bounded-reconciliation edges as typed propagation families.
5. Execute one deterministic `UiAllocationSolvePlan` with explicit pass order,
   cycle posture, remainder policy, and solve trace rather than hidden
   recursion or opportunistic fixed-point behavior.
6. Admit viewport-, scroll-, and portal-derived planning inputs as typed
   special-input edges without broadening into continuous churn.
7. Admit durable resize input posture such as splitter position as planning
   input, while keeping transient interaction state non-authoritative.
8. Produce one deterministic `UiAllocationPlan` plus one
   `UiAllocationPlanningCostReceipt` per admitted neighborhood.
9. Bind allocation-plan identity into `WorthUiRuntimeHandleAllocationBasis` /
   `WorthUiRuntimeHandleAllocationReceipt` rather than letting later topology
   assembly rediscover plan semantics.
10. Prove execution topology carries solved planning identity forward without
    re-solving committed geometry or mounted output truth.
11. Keep safe-frame-boundary activation and atomic plan swap as the future
    commit boundary for candidate plans.
12. Route allocation-plan denials, inspection, and certification through the
    shared evidence and diagnostics substrate before 3.8 consumes the result.

If implementation begins with per-frame resize loops, host drag handlers, or
mounted receipt mutation before these twelve steps exist, the milestone has
started in the wrong place.

## Phase Plan

### Phase 1: Freeze Allocation Planning As A Runtime-Owned Lane

This phase freezes allocation planning as a first-class runtime category that
consumes measurement basis rather than re-deriving layout meaning locally.

**Relevant subsystems**
- `worth-ui-runtime/evidence`
- `worth-ui-runtime/graph`
- `worth-ui-runtime/runtime`

**Relevant APIs**
- `UiMeasurementBasis`
- `UiGraphMeasurementNeighborhoodHint::from_basis(...)`
- `UiGraphTouchMeasurementNeighborhoodHint::from_touch(...)`
- `UiAdmissionBoundary::select_obligations(...)`
- `UiAdmissionBoundary::admit_measurement_requirement(...)`
- `UiAdmissionBoundary::admit_query_measurement_eligibility_for_touch_from_projection_consumption(...)`
- `WorthUiRuntimeHandleAllocationReceipt`

**Warnings**
- Do not let allocation planning start life as a hidden helper under mounted
  receipts or host frame activation.
- Do not redefine 3.6a basis meaning inside the planner.
- Do not let execution-plan topology assembly become the de facto allocation
  planner.

**Test requirements**
- Adversarial equivalence test: equivalent measurement basis and graph inputs
  must converge on the same allocation-neighborhood/planning identity.
- Adversarial boundary test: planning code must not reread source text, host
  adapters, or Query internals to recover meaning already present in basis
  artifacts.

**Engineering decisions**
- Introduce one explicit allocation-planning lane between 3.6a basis intake and
  runtime execution-plan lowering.
- Keep planning artifacts derived and reproducible from admitted inputs.
- Require planning output to be inspectable and denial-bearing before mounted
  receipts exist.

**Open questions**
- None.

### Phase 2: Define Allocation Neighborhood Authority And Membership

This phase turns the 3.6a neighborhood hint into a real planning scope with
typed membership rules.

**Relevant subsystems**
- `worth-ui-runtime/graph`
- `worth-ui-runtime/obligations`
- `worth-ui-runtime/evidence`

**Relevant APIs**
- `UiMeasurementDependencyMap`
- `UiMeasurementNeighborhoodClassHint`
- `UiGraphMeasurementNeighborhoodHint`
- `UiGraphTouchDescriptor`
- `UiSelectedObligationSet`

**Warnings**
- Do not treat "the whole page" or "the whole mosaic" as the default planning
  neighborhood.
- Do not infer membership heuristically from renderer topology.
- Do not conflate structural neighborhood membership with later receipt churn
  breadth.
- Do not let currently hidden or non-layout-participating nodes silently join a
  layout neighborhood just because they share ancestry.
- Do not let repeated-instance order or current slot index become the peer
  identity basis for sibling negotiation.

**Test requirements**
- Adversarial localization test: one local basis change must resolve to a typed
  local allocation neighborhood rather than an unconditional broad planning
  region.
- Adversarial membership-rejection test: unrelated nodes with no dependency-map
  relation must stay outside the neighborhood even when they share ancestors.
- Adversarial participation test: nodes denied `participation.layout` must stay
  out of allocation neighborhoods even when they remain mounted or visible for
  other runtime families.
- Adversarial identity test: reordering repeated instances without semantic
  identity change must preserve neighborhood membership and peer-group identity.

**Engineering decisions**
- Introduce `UiAllocationNeighborhood` as the bounded planning authority
  artifact.
- Freeze `UiAllocationNeighborhoodIdentity` as an explicit equivalence contract
  rather than an emergent digest over whatever the current solver touched.
- Make neighborhood membership derive from graph identity plus measurement
  dependency map, not from widget-local recursion.
- Preserve class-specific membership posture for local intrinsic, container,
  viewport, scroll-container, and portal-anchor neighborhoods.

**Open questions**
- None.

### Phase 3: Define Constraint Artifacts And Propagation Edges

This phase freezes the typed language of allocation propagation before solving
it.

**Relevant subsystems**
- `worth-ui-runtime/evidence`
- `worth-ui-runtime/graph`
- `worth-ui-runtime/runtime`

**Relevant APIs**
- `UiAllocationNeighborhood`
- `UiAllocationConstraintSet`
- `UiConstraintPropagationEdge`
- `UiMeasurementBasis`
- `admit_measurement_basis(...)`

**Warnings**
- Do not smuggle multiple propagation families through one opaque "solve"
  helper.
- Do not let parent-to-child, child-to-parent, and sibling negotiation reuse
  one unlabeled edge kind.
- Do not let layout operators invent private propagation semantics that bypass
  one shared planning contract lane.
- Do not hide cycle posture until denial time.

**Test requirements**
- Adversarial shape test: equivalent planning neighborhoods must emit the same
  propagation-edge set in canonical order.
- Adversarial rejection test: impossible edge families or duplicate edge
  authorities must deny before solve-time recursion begins.
- Adversarial operator-contract test: stack, row, grid, split, mosaic,
  overlay, scroll, and portal-anchor must lower through an explicit
  `UiLayoutOperatorPlanningContract` rather than operator-local solve helpers.

**Engineering decisions**
- Freeze a closed propagation-edge vocabulary in 3.6b.
- Introduce `UiLayoutOperatorPlanningContract` so each layout operator declares
  allowed propagation families, sizing modes, intrinsic return posture,
  sibling grouping rule, special-input requirements, and denial policy.
- Keep edge identity and payload digest explicit so planning reuse and
  diagnostics can name what changed.
- Separate special-input edges from ordinary parent/child/sibling edges.

**Open questions**
- None.

### Phase 4: Propagate Parent Available Space To Children

This phase closes the downward constraint flow for container-driven layout.

**Relevant subsystems**
- `worth-ui-runtime/evidence`
- `worth-ui-runtime/graph`

**Relevant APIs**
- `UiDeclaredMeasurementPolicyPosture`
- `UiMeasurementBasis`
- `UiAllocationConstraintSet`
- `UiConstraintPropagationEdge`

**Warnings**
- Do not reinterpret parent available space from host pixels after 3.6a basis
  is admitted.
- Do not let downward propagation silently override child min/max posture.
- Do not hide axis-specific propagation inside operator-specific branches.
- Do not mix coordinate spaces or rounding postures in one downward constraint
  path without an explicit normalization rule.

**Test requirements**
- Adversarial convergence test: equivalent parent basis and child declaration
  posture must produce equivalent child available-space constraints.
- Adversarial denial test: contradictory parent available-space inputs or
  missing required downward constraints must deny structurally before planning
  continues.
- Adversarial normalization test: logically equivalent available-space inputs in
  normalized unit/coordinate posture must converge, while mismatched posture
  must deny or remain explicitly distinguishable.

**Engineering decisions**
- Make parent available-space propagation an explicit edge family.
- Preserve axis-specific propagation posture as typed planning data.
- Keep downward propagation separate from later sibling negotiation and
  intrinsic return flow.

**Open questions**
- None.

### Phase 5: Propagate Child Intrinsic Contributions Back Upward

This phase closes the return path for hug/content-measured/intrinsic child
requirements.

**Relevant subsystems**
- `worth-ui-runtime/evidence`
- `worth-ui-query-binding`
- `worth-ui-runtime/host`

**Relevant APIs**
- `UiMeasurementBasis`
- `UiMeasurementResult`
- `UiProjectionFactReceipt`
- `UiAllocationConstraintSet`
- `consume_projection_facts(...)`
- `UiAdmissionBoundary::admit_query_measurement_eligibility_from_projection_consumption(...)`

**Warnings**
- Do not recompute intrinsic meaning directly from controls or host widgets.
- Do not flatten Query-backed and host-backed intrinsic contributions into one
  anonymous number path.
- Do not let intrinsic return flow bypass generation compatibility posture.
- Do not merge intrinsic contributions that disagree on unit, coordinate-space,
  or rounding posture as though they were the same evidence.

**Test requirements**
- Adversarial parity test: equivalent intrinsic evidence from Query/host inputs
  must yield the same upward contribution for the same neighborhood.
- Adversarial stale-input test: stale projection facts or stale host evidence
  must deny or remain explicitly stale instead of entering the solved plan.

**Engineering decisions**
- Treat upward intrinsic contribution as a separate propagation edge family.
- Preserve whether the contribution came from Query facts, host evidence, or
  both.
- Require upward contributions to remain basis-generation-compatible before
  neighborhood solve.

**Open questions**
- None.

### Phase 6: Solve Sibling Negotiation

This phase closes peer-level negotiation semantics before any equal-share
distribution logic broadens the surface.

**Relevant subsystems**
- `worth-ui-runtime/evidence`
- `worth-ui-runtime/graph`
- `worth-ui-runtime/capability`

**Relevant APIs**
- `UiAllocationConstraintSet`
- `UiConstraintPropagationEdge`
- `UiAllocationNeighborhood`
- `MosaicSizingContractDescriptor`

**Warnings**
- Do not hide sibling negotiation inside row/stack/grid/mosaic helpers.
- Do not let one sibling's intrinsic contribution silently erase another
  sibling's bounded posture.
- Do not let sibling negotiation depend on transient visual order when stable
  peer identity is the real semantic basis.

**Test requirements**
- Adversarial equivalence test: equivalent sibling groups with the same
  constraints must converge on the same negotiation result.
- Adversarial rejection test: contradictory sibling contracts or impossible
  peer requirements must deny before any equal-share or bounded reconciliation
  step executes.
- Adversarial fixed-point test: if a sibling group enters a supported
  bidirectional solve posture, the planner must either converge through an
  explicit admitted fixed-point policy or deny as unsupported convergence.

**Engineering decisions**
- Make sibling negotiation its own explicit propagation family.
- Preserve group identity for peer negotiation so local sibling changes do not
  widen beyond the negotiation set.
- Require solve order to record whether sibling negotiation ran before or after
  equal-share and bound reconciliation so replay and inspection do not infer it
  heuristically.

**Open questions**
- None.

### Phase 7: Solve Equal-Share Distribution

This phase closes equal-share planning as its own peer-distribution step rather
than a leftover arithmetic detail inside sibling negotiation.

**Relevant subsystems**
- `worth-ui-runtime/evidence`
- `worth-ui-runtime/graph`
- `worth-ui-runtime/capability`

**Relevant APIs**
- `UiAllocationConstraintSet`
- `UiConstraintPropagationEdge`
- `UiAllocationNeighborhood`
- `MosaicSizingContractDescriptor`

**Warnings**
- Do not let equal-share fall back to broad leftover-space folklore.
- Do not let zero-sized, hidden, or non-layout-participating peers distort an
  equal-share group unless the declaration semantics explicitly admit them.
- Do not treat equal-share as a synonym for generic sibling negotiation.
- Do not distribute rounding remainder by incidental iteration order.

**Test requirements**
- Adversarial equivalence test: equivalent equal-share groups with the same
  admitted peer set and the same available space must converge on the same
  distribution.
- Adversarial rejection test: overcommitted equal-share groups must deny before
  bounded reconciliation or runtime handoff.
- Adversarial edge-case test: zero-share, zero-available-space, and
  single-surviving-peer cases must resolve through typed equal-share posture
  rather than divide-by-leftover folklore.
- Adversarial remainder test: non-integral leftover space must resolve through
  one declared `UiEqualShareDistributionPolicy` keyed by stable peer identity
  or deny when integral distribution is required.

**Engineering decisions**
- Make equal-share distribution its own explicit propagation family.
- Preserve equal-share group identity independently from broader sibling
  negotiation groups.
- Keep equal-share semantics aligned with declaration/runtime meaning rather
  than raw arithmetic convenience.
- Record deterministic remainder handling in solve trace artifacts so later
  inspection and certification can explain why one peer received the extra unit.

**Open questions**
- None.

### Phase 8: Reconcile Min/Max Bounds Without Hidden Fallbacks

This phase closes bounded constraint reconciliation and contradiction posture.

**Relevant subsystems**
- `worth-ui-runtime/evidence`
- `worth-ui-runtime/declaration`
- `worth-ui-runtime/diagnostics`

**Relevant APIs**
- `UiDeclaredMeasurementPolicyPosture`
- `UiAllocationConstraintSet`
- `UiAllocationPlan`

**Warnings**
- Do not silently clamp impossible plans and pretend they solved cleanly.
- Do not collapse underconstrained, overconstrained, and cyclic neighborhoods
  into one denial family.
- Do not let host adapters invent fallback bounds after runtime planning fails.
- Do not hide contradictions caused by mixed unit, coordinate-space, or
  rounding posture behind ordinary min/max clamping.
- Do not treat declared bounded clamp and contradiction denial as the same
  outcome.

**Test requirements**
- Adversarial contradiction test: impossible min/max combinations must deny
  with typed contradiction posture before any receipt path executes.
- Adversarial determinism test: equivalent bounded-neighborhood inputs must
  converge on the same reconciliation result or the same typed denial.
- Adversarial clamp-honesty test: when bounded reconciliation permits clamp,
  the resulting `UiBoundReconciliationPosture` must record declared clamp
  explicitly instead of pretending the plan satisfied bounds naturally.

**Engineering decisions**
- Keep bounded reconciliation as an explicit propagation family.
- Distinguish at least:
  - underconstrained
  - overconstrained
  - cyclic
  - stale-input
  - unsupported-special-input
- Distinguish declared clamp from contradiction denial with a dedicated
  `UiBoundReconciliationPosture`.
- Treat contradiction posture as part of the solved plan artifact, not as a
  string-only diagnostic emitted later.

**Open questions**
- None.

### Phase 9: Admit Viewport-Derived Planning Inputs

This phase closes viewport-derived planning as its own special-input family.

**Relevant subsystems**
- `worth-ui-runtime/evidence`
- `worth-ui-runtime/host`

**Relevant APIs**
- `UiMeasurementNeighborhoodClassHint`
- `UiMeasurementResult`
- `UiMeasurementRequest`
- `UiAllocationConstraintSet`
- `UiConstraintPropagationEdge`
- `freeze_measurement_request(...)`
- `request_host_measurement(...)`
- `collect_host_measurement_evidence(...)`
- `admit_current_host_measurement_evidence(...)`

**Warnings**
- Do not treat viewport-relative requirements as ordinary parent-child
  propagation.
- Do not broaden this phase into scroll extent or portal-anchor work.
- Do not let raw viewport host observations bypass 3.6a normalization posture.

**Test requirements**
- Adversarial locality test: viewport-derived planning inputs must map to their
  own typed special-input planning edge and neighborhood.
- Adversarial denial test: missing viewport evidence must deny through typed
  planning posture rather than hidden best effort.

**Engineering decisions**
- Preserve viewport basis as its own special planning edge family.
- Keep viewport inputs as planning-time inputs only in 3.6b.
- Require viewport planning to retain source evidence identity so later churn
  work can invalidate only the neighborhoods that actually consumed it.

**Open questions**
- None.

### Phase 10: Admit Scroll-Owner Planning Inputs

This phase closes scroll-container planning inputs as their own special-input
family without broadening into continuous scroll behavior.

**Relevant subsystems**
- `worth-ui-runtime/evidence`
- `worth-ui-runtime/host`
- `worth-ui-runtime/services`

**Relevant APIs**
- `UiMeasurementNeighborhoodClassHint`
- `UiMeasurementResult`
- `UiMeasurementRequest`
- `UiAllocationConstraintSet`
- `UiConstraintPropagationEdge`
- `freeze_measurement_request(...)`
- `request_host_measurement(...)`
- `collect_host_measurement_evidence(...)`
- `admit_current_host_measurement_evidence(...)`

**Warnings**
- Do not treat scroll-owned planning requirements as ordinary parent-child
  propagation.
- Do not broaden this phase into continuous scroll extent maintenance or scroll
  position behavior.
- Do not let scroll container host mechanics decide semantic scroll ownership.

**Test requirements**
- Adversarial locality test: scroll-container planning inputs must map to their
  own typed special-input planning edge and neighborhood.
- Adversarial denial test: missing scroll-container basis must deny through
  typed planning posture rather than hidden best effort.

**Engineering decisions**
- Preserve scroll-owner basis as its own special planning edge family.
- Keep scroll inputs as planning-time inputs only in 3.6b.
- Require scroll-owner planning to retain source evidence identity so later
  churn work can invalidate only the neighborhoods that actually consumed it.

**Open questions**
- None.

### Phase 11: Admit Portal-Anchor Planning Inputs

This phase closes portal-anchor planning inputs as their own special-input
family without broadening into continuous anchor reposition behavior.

**Relevant subsystems**
- `worth-ui-runtime/evidence`
- `worth-ui-runtime/host`
- `worth-ui-runtime/services`

**Relevant APIs**
- `UiMeasurementNeighborhoodClassHint`
- `UiMeasurementResult`
- `UiMeasurementRequest`
- `UiAllocationConstraintSet`
- `UiConstraintPropagationEdge`
- `freeze_measurement_request(...)`
- `request_host_measurement(...)`
- `collect_host_measurement_evidence(...)`
- `admit_current_host_measurement_evidence(...)`

**Warnings**
- Do not treat portal-anchor requirements as ordinary parent-child propagation.
- Do not broaden this phase into continuous anchor reposition loops.
- Do not let portal host mechanics decide anchor planning semantics.

**Test requirements**
- Adversarial locality test: portal-anchor planning inputs must map to their
  own typed special-input planning edge and neighborhood.
- Adversarial denial test: missing portal-anchor evidence must deny through
  typed planning posture rather than hidden best effort.

**Engineering decisions**
- Preserve portal-anchor basis as its own special planning edge family.
- Keep portal-anchor inputs as planning-time inputs only in 3.6b.
- Require portal-anchor planning to retain source evidence identity so later
  churn work can invalidate only the neighborhoods that actually consumed it.

**Open questions**
- None.

### Phase 12: Admit Durable Resize Inputs Without Making Drag State Authoritative

This phase freezes the resize seam so splitter and resizable layout work can
grow later without corrupting authority boundaries.

**Relevant subsystems**
- `worth-ui-runtime/runtime/state_inventory`
- `worth-ui-runtime/runtime/reconciliation`
- `worth-ui-runtime/capability`

**Relevant APIs**
- `MosaicResizePermission`
- `WorthUiSplitterPositionReconciliation`
- `WorthUiDurableStateFamily::splitter_position()`
- `WorthUiTransientInteractionState`
- `WorthUiTransientInteractionPolicy`

**Warnings**
- Do not let transient drag state become the source of persistent layout truth.
- Do not ignore durable splitter-position state when resize permission makes it
  a declared planning input.
- Do not broaden this phase into defining per-frame drag commit cadence.
- Do not let stale carried-forward splitter state silently survive when the
  declaration identity, sibling set, or resize contract changed incompatibly.

**Test requirements**
- Adversarial authority test: durable splitter-position input may influence the
  allocation plan only when admitted through the durable-state/planning seam;
  transient interaction state alone must not.
- Adversarial preservation test: stable-identity carry-forward of splitter
  position must survive rebind where admitted, while invalid or absent durable
  input must deny or fall back through typed posture.
- Adversarial shape-change test: carry-forwarded splitter state must deny or
  remap explicitly when repeated-instance identity, slot membership, or sizing
  contract changed incompatibly.

**Engineering decisions**
- Treat durable resize input as a planning edge family distinct from host drag
  observations.
- Preserve resize-permission posture in the plan input so user-resizable and
  fixed neighborhoods remain mechanically distinct.
- Leave continuous drag-stream planning and activation cadence to 3.8.

**Open questions**
- None.

### Phase 13: Bind Allocation Plan Identity Into Runtime Handle Allocation Basis

This phase closes the first runtime handoff from semantic planning into
handle-allocation authority without claiming committed geometry.

**Relevant subsystems**
- `worth-ui-runtime/runtime/handle_allocation`
- `worth-ui-runtime/runtime/plan_topology`
- `worth-ui-runtime/runtime`

**Relevant APIs**
- `WorthUiRuntimeHandleAllocationBasis`
- `WorthUiRuntimeHandleAllocation`
- `WorthUiRuntimeHandleAllocationReceipt`
- `WorthUiRuntimeHost::admit_execution_lanes(...)`

**Warnings**
- Do not let handle allocation digest only raw topology if allocation-plan
  semantics are now part of correctness.
- Do not treat handle-allocation binding as license to compute mounted boxes or
  host-consumable geometry in 3.6b.

**Test requirements**
- Adversarial receipt-binding test: changing the solved allocation plan while
  leaving raw node topology constant must still change the relevant plan/handle
  binding identity where correctness requires it.
- Adversarial scope test: handle-allocation basis may carry planning identity,
  lane partition hints, and denial posture, but it must not claim final mounted
  geometry or receipt truth.

**Engineering decisions**
- Extend handle-allocation basis/handoff to carry allocation-plan identity
  honestly.
- Preserve the distinct roles of allocation plan and handle allocation receipt.

**Open questions**
- None.

### Phase 14: Prove Execution Topology Consumes Planning Identity Without Rediscovery

This phase closes the second runtime handoff from handle allocation into
execution-plan topology while proving topology lowering does not rediscover
allocation semantics.

**Relevant subsystems**
- `worth-ui-runtime/runtime/plan_topology`
- `worth-ui-runtime/runtime`

**Relevant APIs**
- `WorthUiRuntimeHost::assemble_execution_plan_topology_with_lane_admission(...)`
- `WorthUiPlanTopologyAssembler::assemble_with_lane_admission(...)`
- `WorthUiExecutionPlan`
- `WorthUiPlanTopology`
- `WorthUiPlanLanePartition`
- `WorthUiPlanLookupIndex`
- `WorthUiPlanTopologyCounters`

**Warnings**
- Do not ask topology assembly to rediscover solved neighborhood meaning.
- Do not let execution-plan lowering collapse back into raw node-input
  interpretation once allocation plan identity exists.
- Do not hide lane partitioning semantics inside renderer-facing helpers.
- Do not broaden this phase into full mounted-geometry production or visual
  output truth.

**Test requirements**
- Adversarial anti-rediscovery test: topology assembly must consume already
  solved allocation planning artifacts rather than reconstructing them from node
  inputs or host assumptions.
- Adversarial topology test: equivalent solved allocation plans must lower to
  equivalent execution-plan topology and lane partitions.
- Adversarial boundary test: execution topology may carry planning identity and
  lane partition structure, but final visible geometry must remain a later
  receipt/commit concern.

**Engineering decisions**
- Preserve the distinct roles of handle allocation receipt, topology assembly,
  and execution plan.
- Keep topology counters and plan counters evidence-bearing so later churn work
  can prove bounded replanning cost.

**Open questions**
- None.

### Phase 15: Preserve Safe Activation As The Future Commit Boundary

This phase prevents the new planner from corrupting the existing activation and
bounded-change runtime posture while keeping activation as the later commit
boundary rather than a planning shortcut.

**Relevant subsystems**
- `worth-ui-runtime/runtime/activation_staging`
- `worth-ui-runtime/runtime/atomic_plan_swap`
- `worth-ui-runtime/runtime`

**Relevant APIs**
- `WorthUiRuntimeHost::prepare_ready_activation(...)`
- `WorthUiRuntimeHost::activate_ready_at_frame_boundary(...)`
- `WorthUiRuntimeHost::swap_ready_activation_at_frame_boundary(...)`
- `WorthUiReadyActivation`
- `WorthUiPlanSwapReceipt`
- `WorthUiPlanSwapRollback`
- `WorthUiActivationGateReceipt`

**Warnings**
- Do not let local allocation replanning bypass the safe frame-boundary
  activation gate.
- Do not let resize-oriented planning broaden into direct active-state mutation.
- Do not hide plan-widening behind "activation convenience" helpers.
- Do not let candidate planning artifacts masquerade as committed mounted truth
  before future receipt work lands.

**Test requirements**
- Adversarial boundary test: a changed allocation neighborhood must still enter
  the runtime through ready activation and atomic plan swap rather than direct
  active-state mutation.
- Adversarial narrowness test: equivalent active state plus one local changed
  allocation neighborhood must not widen into unrelated plan rebuilds unless
  the neighborhood graph actually overlaps.

**Engineering decisions**
- Keep allocation planning upstream of activation.
- Preserve safe-frame-boundary semantics as the only ordinary commit boundary.
- Make local replanning breadth explicit in the plan artifact and counters so
  activation need not guess why a candidate widened.

**Open questions**
- None.

### Phase 16: Make Allocation Planning Inspectable

This phase closes the evidence and inspection surface for allocation planning
before certification broadens the proof program.

**Relevant subsystems**
- `worth-ui-inspection`
- `worth-ui-runtime/evidence`

**Relevant APIs**
- `UiInspectionScope::Measurement`
- `UiInspectionReceipt`
- `UiEvidenceSlice`
- `UiAllocationNeighborhood`
- `UiAllocationPlan`
- `workspace.inspect(...)`
- `request_causal_inspection`

**Warnings**
- Do not wait for mounted receipts before exposing planning denials and
  neighborhood evidence.
- Do not build a special layout debugger that bypasses the shared evidence
  substrate.
- Do not flatten neighborhood identity or propagation edges into presentation
  strings.

**Test requirements**
- Adversarial inspection parity test: public inspection of an allocation
  neighborhood or allocation plan must converge on the same retained evidence as
  direct runtime evidence assembly.
- Adversarial narrowness test: inspecting one neighborhood must not widen into
  whole-frame explanation unless the declared budget/relevance settings ask for
  it.
- Adversarial explanation test: inspection must answer neighborhood membership,
  propagation edge origin, equal-share remainder distribution, bound
  reconciliation posture, special-input identity, and durable resize input
  posture through typed evidence rather than presentation-only strings.

**Engineering decisions**
- Extend the shared inspection substrate rather than adding a planner-local
  tool lane.
- Preserve neighborhood identity, propagation edges, denial posture, and
  handle-binding evidence as inspectable artifacts.
- Expose explicit planner questions such as:
  - `inspect_allocation_neighborhood(neighborhood_id)`
  - `inspect_allocation_plan(plan_id)`
  - `explain_neighborhood_membership(node_id)`
  - `explain_propagation_edge(edge_id)`
  - `explain_equal_share_group(group_id)`
  - `explain_bound_reconciliation(plan_id)`
  - `explain_special_input(plan_id, viewport/scroll/portal)`
  - `explain_durable_resize_input(plan_id)`

**Open questions**
- None.

### Phase 17: Certify Allocation Planning Determinism And Anti-Bypass Boundaries

This phase closes the milestone by proving allocation planning is a real
architecture seam and not just better helper code.

**Relevant subsystems**
- `worth-ui-certification`
- `worth-ui-runtime/evidence`

**Relevant APIs**
- `certify_measurement_basis_determinism(...)`
- `certify_measurement_basis_determinism_for_scenarios(...)`
- `UiMeasurementBasisCertificationScenario`
- `UiMeasurementBasisCertificationOutcome`
- `UiAllocationPlanningCostReceipt`
- `UiAllocationSolveTrace`

**Warnings**
- Do not wait for mounted receipts before exposing planning denials and
  neighborhood evidence.
- Do not build a special layout debugger that bypasses the shared evidence
  substrate.
- Do not certify only stack/happy-path neighborhoods.

**Test requirements**
- Adversarial certification test: hostile viewport, sibling, portal, scroll,
  and resize-input scenarios must prove deterministic planning, typed denial,
  and bounded neighborhood width.
- Adversarial anti-bypass test: host adapters, topology assemblers, and
  operator-local helpers must fail certification if they can mint planning
  semantics or committed-geometry claims outside the admitted planning lane.

**Engineering decisions**
- Close 3.6b with a named certification matrix:
  - `allocation_neighborhood_suite`
  - `constraint_edge_suite`
  - `parent_child_propagation_suite`
  - `intrinsic_return_flow_suite`
  - `sibling_negotiation_suite`
  - `equal_share_suite`
  - `bounded_reconciliation_suite`
  - `special_input_suite`
  - `durable_resize_input_suite`
  - `plan_handoff_suite`
  - `activation_boundary_suite`
  - `allocation_inspection_suite`
  - `allocation_anti_bypass_suite`
- Treat 3.6b as closed only when 3.8 can consume its artifacts without
  reopening neighborhood or propagation semantics.

**Open questions**
- None.

## Must Ship

- `milestone-3.6b.md` as the allocation-planning spec
- one explicit allocation-planning lane between 3.6a measurement basis and
  runtime execution-plan lowering
- `UiLayoutOperatorPlanningContract` for:
  - `stack`
  - `row`
  - `grid`
  - `split`
  - `mosaic`
  - `overlay`
  - `scroll`
  - `portal_anchor`
- `UiAllocationNeighborhood` with explicit:
  - identity
  - generation
  - root graph node identity
  - measurement basis identity
  - layout operator contract identity
  - dependency map
  - neighborhood class
  - propagation edges
  - special planning inputs
  - durable planning inputs
  - denial posture
- explicit `UiAllocationNeighborhoodIdentity` equivalence basis that includes:
  - root graph node identity
  - graph generation
  - measurement basis identity
  - layout operator contract identity
  - neighborhood class
  - participating member identity set
  - dependency map digest
  - world profile
  and excludes:
  - incidental sibling order
  - incidental current visibility
  - host handles
  - transient drag state
- `UiAllocationConstraintSet` with explicit:
  - parent available-space posture
  - intrinsic contribution requirements
  - sibling negotiation mode
  - equal-share grouping
  - bounded min/max requirements
  - viewport / scroll / portal requirements
  - resize-permission posture
- `UiConstraintPropagationEdge` with closed typed edge families for:
  - `parent_available_space`
  - `child_intrinsic_contribution`
  - `sibling_negotiation`
  - `equal_share_distribution`
  - `bounded_reconciliation`
  - `viewport_basis`
  - `scroll_owner_basis`
  - `portal_anchor_basis`
  - `durable_resize_input`
- `UiAllocationSolvePlan`, `UiAllocationSolvePass`, and `UiAllocationSolveTrace`
  with one declared solve order covering:
  - normalization
  - special-input admission
  - neighborhood membership freeze
  - propagation-edge emission
  - parent available-space propagation
  - child intrinsic return flow
  - sibling negotiation
  - equal-share distribution
  - bounded min/max reconciliation
  - final denial-or-plan classification
- `UiConstraintCyclePosture`
- `UiEqualShareDistributionPolicy` with deterministic remainder handling by
  stable peer identity
- `UiBoundReconciliationPosture` distinguishing declared clamp from
  contradiction
- `UiAllocationPlan` with explicit:
  - identity
  - generation
  - measurement basis identity
  - allocation neighborhood identity
  - graph node identity
  - solved node constraints
  - plan posture / denial posture
  - lane-partitioning hint
  - handle-allocation basis digest
  - solve-trace identity
- explicit statement that `UiAllocationPlan` solves abstract constraints and
  lane partitioning only, not committed mounted geometry
- `UiAllocationPlanningCostReceipt` with boundedness counters and cost class
- explicit typed denial posture for:
  - underconstrained neighborhoods
  - overconstrained neighborhoods
  - cyclic neighborhoods
  - denied unsupported convergence
  - stale-input neighborhoods
  - incompatible participation posture
  - incompatible repeated-instance identity carry-forward
  - incompatible unit / coordinate-space / rounding posture
  - missing special-input evidence
  - unsupported resize/viewport/portal/scroll planning posture
- deterministic parent-to-child propagation semantics
- deterministic child intrinsic return-flow semantics
- deterministic sibling negotiation and equal-share semantics
- deterministic bounded min/max reconciliation semantics
- special planning-input handling for:
  - viewport-derived basis
  - scroll-container basis
  - portal-anchor basis
- durable resize-input planning posture for admitted splitter-position style
  state without making transient interaction state authoritative
- runtime handoff compatibility that binds solved allocation-plan identity into:
  - `WorthUiRuntimeHandleAllocationBasis`
  - `WorthUiRuntimeHandleAllocationReceipt`
  - `WorthUiExecutionPlan`
  without claiming final mounted geometry
- inspection and diagnostics for allocation neighborhoods, propagation edges,
  and allocation-plan denials through the shared evidence substrate
- named certification matrix proving deterministic planning, anti-bypass
  boundaries, and bounded neighborhood width

## Must Preserve

- 3.6a remains the owner of measurement vocabulary, basis admission, host
  evidence policy, and Query fact eligibility
- allocation planning remains derived from admitted basis and graph truth
- Query remains the owner of Query basis, projection consumption, retained
  inspection, and cross-runtime causal explanation
- host adapters remain mechanics/observation translators rather than layout
  semantic owners
- allocation neighborhood, allocation plan, execution plan, and mounted receipt
  remain distinct artifacts
- transient drag/resize interaction state remains non-authoritative
- durable resize input participates only through admitted planning-state seams
- safe-frame-boundary activation and atomic plan swap remain the ordinary
  commit boundary
- layout-participation posture remains the gate for neighborhood membership
- repeated-instance identity remains canonical and may not collapse into order
  or visibility folklore during planning
- unit, coordinate-space, and rounding posture remain explicit wherever special
  planning inputs or intrinsic evidence are normalized
- scroll and portal behavior remain narrowed to planning-time semantics in
  3.6b rather than continuous behavior ownership
- inspection remains one shared substrate for AI and human consumers

## Acceptance Evidence

- equivalent declaration + graph + measurement basis inputs converge to the
  same allocation neighborhood and the same allocation plan
- equivalent admitted operator contracts converge on the same solve order,
  solve trace, and remainder-policy outcome
- parent/child, sibling, equal-share, and bounded min/max semantics resolve
  through typed propagation rules rather than host helpers or local recursion
- local basis changes identify a typed affected allocation neighborhood instead
  of forcing unconditional whole-page replanning
- hidden or otherwise non-layout-participating nodes do not silently enter
  allocation neighborhoods or equal-share groups
- viewport-, scroll-, and portal-derived planning inputs remain distinct typed
  planning edges and denials
- durable splitter-position style state can influence admitted planning while
  transient drag state alone cannot become layout authority
- repeated-instance reorder without semantic identity change preserves peer
  negotiation identity, while incompatible shape changes deny or remap
  explicitly
- mixed unit, coordinate-space, or rounding posture either normalizes through a
  named path or denies explicitly rather than quietly merging contradictory
  evidence
- allowed clamp posture remains explicitly recorded in the solved plan, while
  contradictory min/max posture denies rather than silently clamping
- topology assembly and runtime handle allocation consume solved planning
  artifacts rather than rediscovering planning semantics from raw node inputs
- runtime handoff carries planning identity and lane partition hints without
  claiming committed mounted boxes or host paint geometry
- changed allocation neighborhoods still commit only through ready activation
  plus atomic plan swap at a safe frame boundary
- inspection can explain allocation neighborhood membership, propagation edges,
  denial posture, and handle-binding identity without renderer-local helpers
- planning cost receipts prove bounded neighborhood width, emitted edge count,
  and propagation pass count for hostile local replanning scenarios
- certification proves no host adapter, component helper, or transient
  interaction path can mint layout authority locally

## Sequencing Notes

- 3.6b belongs after 3.6a because allocation planning must consume frozen
  measurement basis rather than co-defining measurement semantics while
  planning.
- 3.6b belongs before 3.8 because resize, drag, scroll, and portal churn need
  a stable planning kernel and bounded neighborhood model before continuous
  invalidation broadens the runtime.
- 3.6b also belongs before committed allocation receipt work because receipt
  truth should consume solved planning artifacts instead of inventing
  plan-shaped semantics after the fact.
