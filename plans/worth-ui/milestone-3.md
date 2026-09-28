# Milestone 3: Hot Runtime, Stable Identity, Execution Plans, And Frame-Cost Certification

## Goal

Make canonical Worth UI artifacts become active, frame-executable runtime plans
that can be hot-replaced inside a running app without losing identity, durable
interaction state, Query-owned binding posture, diagnostics truth, or frame-cost
honesty.

## Why This Milestone Exists

Milestone 1 closed the capability registration and snapshot authority boundary.
Milestone 2 closes the source-to-artifact authority boundary.

Those are still not enough for a UI platform.

The next foundation must prove that the running Worth UI runtime can consume
canonical artifacts as executable platform truth. Hot reload, stable identity,
execution-plan lowering, execution-lane specialization, and frame-cost counters
are one foundation because later shell, command, Query-bound view, component,
canvas, plugin, and tooling work all depend on the same active-plan substrate.

If this milestone is weak, later milestones inherit the wrong runtime:

- hot reload becomes file watcher theater over non-executable artifacts
- execution plans invent identity, equivalence, and replacement rules after
  reload already claimed to preserve state
- shell and widget work depend on app-local state carry-forward folklore
- data-heavy and real-time surfaces retrofit performance lanes after broad
  frame-path coupling exists
- Query-bound UI state collapses into local subscription or loading wrappers
- performance certification remains profiling narrative instead of a platform
  contract

This milestone therefore merges the original hot-reload and execution-plan
roadmap boundaries into one product-grade runtime activation milestone.

## Governing Summaries

- `MENTALITY.md`: protects Worth UI from MVP reload and widget-first drift; this
  milestone must build the runtime substrate for the product Worth UI is meant
  to become, not a narrow demo reload loop.
- `arch_laws.md`: protects proof-bearing phase chains and facade authority; raw
  source, candidate artifacts, admitted replacements, reconciliation plans,
  active execution plans, and frame receipts must be distinct typed states.
- `composition_laws.md`: protects the implementation from god reload functions;
  watch, admission, equivalence, impact narrowing, identity matching,
  reconciliation, plan lowering, lane execution, counters, and certification
  need named responsibilities and predictable files.
- `domain_structure_laws.md`: protects the tree from hiding authority; active
  runtime truth, durable UI state, source ingress, Query-owned live posture,
  derived diagnostics, and frame-cost evidence must not share structural space.
- `perf_laws.md`: protects frame-path honesty; semantic richness must lower
  before frames run, reuse must have explicit equivalence, and execution breadth
  must be explained by counters at named boundaries.
- `worth_ui_roadmap.md`: protects the platform sequence; hot iteration, stable
  identity, frame-efficient execution lanes, and cost certification are
  foundation-first work that must land before shell and component breadth.
- `worth-ui-vision.md`: protects the platform thesis; Worth UI must give
  desktop developers runtime-owned hot iteration and frame-efficient native
  execution without becoming a web runtime or a thin egui widget bundle.
- `milestone-1.md`: protects frozen capability snapshots and facade-only app
  construction; this milestone must consume snapshots and public facade
  artifacts rather than reopening mutable registry authority.
- `milestone-2.md`: protects canonical artifact authority; this milestone must
  consume artifact identity, provenance, equivalence, and dependency metadata
  instead of rediscovering meaning from source text, strings, or builder state.
- `crates/worth-query/docs/AI_README.md`: protects Query's runtime authority;
  Worth UI must carry Query-owned support, admission, live view, async/result,
  recovery, inspection, projection-consumption, and causal explanation posture
  instead of building UI-local pseudo Query layers.
- WORTH Foundational performance docs: protect performance language from
  becoming local profiling folklore; Worth UI should produce its own runtime
  counters and then lower them into Foundational performance claims, canonical
  bundles, counter-backed receipts, planned reports, and certification envelopes
  at explicit boundaries.

## Adversarial Constraint

A running Worth UI app must survive a hostile reload storm over a multi-file
sample app with at least 200 artifact nodes and mixed ordinary, virtualized
data, canvas/spatial, and real-time overlay surfaces: equivalent, valid,
invalid, identity-changing, Query-drifted, and lane-changing candidates must be
admitted or denied through typed phases; valid replacements must lower into
frame-executable plans and swap only at safe frame boundaries; invalid
replacements must preserve the last active plan; durable state must carry
forward only where identity and eligibility prove it; Query bindings must
preserve or rebind only through Query-owned posture; and steady-state frames
must prove through counters that they do not parse source, resolve registry
strings, broad-scan artifact topology, or inherit diagnostic richness by
default.

## Product Decision Lock

- Hot reload and execution-plan lowering are one runtime activation foundation,
  not separate best-effort features.
- The active execution plan is the runtime's frame-executable UI truth; the
  canonical artifact remains the semantic authority it lowers from.
- Reload swaps active runtime plans, not arbitrary Rust code.
- File-authored and Rust-authored artifact candidates use the same replacement,
  reconciliation, lowering, lane, counter, and activation pipeline.
- Last valid active plan preservation is mandatory on every invalid candidate.
- Identity carry-forward is admitted only by typed identity/replacement law, not
  by tree position, display text, source path, or widget-local guesses.
- Durable UI state is not authoritative domain truth and must not masquerade as
  Query, relational, signal, or bridge state.
- Query-bound surfaces preserve Query-owned identity, support, admission,
  live-view, async/result, recovery, inspection, and projection posture.
- Execution lanes specialize mechanics and cost, not UI meaning.
- Ordinary widgets, virtualized data surfaces, canvas/spatial surfaces, and
  real-time overlay/HUD surfaces all belong in the first real execution-plan
  substrate because Worth UI is the UI platform we are building.
- Frame counters are part of product correctness, not optional profiling.
- Rich diagnostics are derived observation artifacts and must not change active
  UI meaning or steady-state frame cost unless explicitly requested by policy.
- WORTH Foundational performance surfaces are boundary vocabulary and proof
  envelopes for Worth UI evidence; they are not the Worth UI frame runtime,
  counter store, lane executor, or plan topology.
- Extension hooks are allowed only where this milestone names an admission
  boundary; hooks may contribute source ingress, state-family rules, identity
  seeds, component lowering, lane adapters, diagnostics presentation, counters,
  or report materialization, but they cannot own active plan truth, Query
  posture, state carry-forward, lane taxonomy, or performance certification.

## Hostile Test Design Rule

Milestone 3 tests must try to break the runtime, not merely prove the happy path
can be typed.

Every phase-level test suite should include at least one case that attempts to
WORTH, omit, reorder, stale, duplicate, drift, broaden, or bypass the proof that
the phase claims to establish. If a test only checks that the obvious accepted
path returns `Ok`, it is not sufficient for this milestone. Tests should prefer
exact receipts, typed denial kinds, counter values, digest comparisons, and
post-failure active-state assertions over substring matching or broad boolean
success.

## Phase Plan

### Phase 1: Active Runtime Authority Boundary

Freeze the runtime-owned authority that holds the active canonical artifact,
active execution plan, last valid replacement point, reload status, and derived
diagnostics references.

**Relevant subsystems**

- runtime activation host
- active artifact and active plan ownership
- last-valid preservation
- reload status model

**Relevant APIs**

- `WorthUiRuntimeHost`
- `WorthUiActiveArtifact`
- `WorthUiActiveExecutionPlan`
- `WorthUiLastValidRuntimeState`
- `WorthUiRuntimeActivationStatus`
- `WorthUiRuntimeLifecycle`
- `WorthUiRuntimeFrameEpoch`
- `WorthUiPendingActivation`
- `WorthUiRuntimeShutdownReceipt`

**Warnings**

- Do not let app code own the active plan outside the Worth UI runtime host.
- Do not collapse active artifact, active plan, and diagnostics into one mutable
  runtime object.
- Do not make last-valid preservation an error-handler side effect.
- Do not let lifecycle hooks mutate active plan state outside startup, pending
  activation, safe-frame swap, pause, resume, or shutdown phases.

**Test requirements**

- `equivalent_runtime_hosts_start_with_equivalent_active_state`: equivalent
  initial artifacts and snapshots produce equivalent runtime host activation
  state.
- `active_runtime_state_not_constructible_from_app_local_parts`: compile-fail
  or visibility coverage proves downstream code cannot mint active runtime
  state directly.
- `last_valid_state_exists_before_first_reload_candidate`: the runtime records
  a preservation point before any candidate can replace it.
- `active_state_rejects_WORTHd_last_valid_receipt`: a WORTHd or stale
  last-valid preservation receipt cannot be installed as active runtime truth.
- `frame_epoch_mismatch_rejects_pending_activation`: a pending activation tied
  to an older frame epoch cannot swap into a newer active runtime state.

**Engineering decisions**

- The runtime host is the only owner of active artifact and active plan
  transition state.
- Last-valid state is a first-class runtime artifact, not a cached copy hidden
  inside diagnostics.
- Active runtime status is structured enough for later diagnostics projection
  without becoming the diagnostics surface itself.
- Runtime lifecycle covers startup, pause/resume around replacement,
  safe-frame activation, shutdown, and failed-activation recovery without
  granting app code direct active-plan mutation.

**Open questions**

- None.

### Phase 2: Replaceable Artifact Candidate Boundary

Freeze the candidate envelope that carries a potential replacement artifact,
its source cause, digest/equivalence basis, provenance handle, dependency
metadata, and authoring lane.

**Relevant subsystems**

- reload candidate envelope
- file-authored candidate ingress
- Rust-authored candidate ingress
- source/change cause capture

**Relevant APIs**

- `WorthUiReplacementCandidate`
- `WorthUiReplacementCause`
- `WorthUiCandidateAuthoringLane`
- `WorthUiCandidateArtifactBundle`
- `WorthUiCandidateDependencyMetadata`

**Warnings**

- Do not pass loose source paths or raw artifact references into replacement
  logic.
- Do not treat file-authored and Rust-authored candidates as separate runtime
  lanes.
- Do not let diagnostics richness become part of candidate semantic identity.

**Test requirements**

- `equivalent_file_and_rust_candidates_with_same_artifact_share_candidate_basis`:
  equivalent file-authored and Rust-authored artifact candidates produce the
  same replacement candidate basis where authoring lane is observational.
- `candidate_without_artifact_digest_or_dependency_metadata_rejected`: a
  candidate cannot enter admission without M2 artifact identity and impact
  metadata.
- `candidate_cause_does_not_change_artifact_equivalence`: source cause
  differences do not alter canonical artifact equivalence.
- `candidate_with_stale_dependency_metadata_rejected_even_when_digest_matches`:
  candidate admission rejects bundles whose artifact digest and dependency
  metadata come from different lowering runs.

**Engineering decisions**

- Candidate envelopes are the only input to replacement admission.
- Candidate authoring lane is preserved for diagnostics and certification, not
  as a fork in semantic runtime behavior.
- Dependency metadata from Milestone 2 is required before impact narrowing can
  be attempted.

**Open questions**

- None.

### Phase 3: Candidate Admission Boundary

Freeze the first denial gate for replacement candidates so unsupported
snapshots, malformed lowering results, missing artifact proof, and unsupported
runtime posture fail before comparison or plan lowering.

**Relevant subsystems**

- candidate admission
- snapshot compatibility
- support posture checks
- replacement denial diagnostics

**Relevant APIs**

- `WorthUiCandidateAdmission`
- `WorthUiAdmittedReplacementCandidate`
- `WorthUiCandidateAdmissionDenial`
- `WorthUiCandidateAdmissionReport`
- WORTH Query `Support Matrix And Admission`
- WORTH Query `Query Operating Modes`
- WORTH Query `Ordinary Outcomes`
- WORTH Query declaration entry readiness and support posture surfaces

**Warnings**

- Do not compare or lower a candidate that has not been admitted.
- Do not read mutable registries to compensate for snapshot mismatch.
- Do not downgrade unsupported posture into a warning when replacement would
  change active runtime truth.

**Test requirements**

- `same_candidate_and_same_active_basis_admit_equivalently`: replaying the same
  candidate against the same active runtime basis produces equivalent admission
  results.
- `snapshot_mismatch_rejected_before_equivalence_comparison`: candidates bound
  to an incompatible capability snapshot fail before artifact comparison.
- `unsupported_runtime_posture_rejected_before_plan_lowering`: unsupported or
  deferred support posture cannot enter execution-plan lowering.
- `admitted_candidate_cannot_swap_query_support_receipts_after_admission`:
  replacing Query support/admission receipts after candidate admission is
  detected before comparison or plan lowering.

**Engineering decisions**

- Admission consumes the active runtime basis and candidate envelope and
  produces a stronger admitted candidate type.
- Support posture remains typed and machine-readable.
- Admission denials are derived diagnostics over candidate and active runtime
  basis; they do not mutate active state.

**Open questions**

- None.

### Phase 4: Artifact Equivalence Consumption Boundary

Freeze how runtime replacement consumes Milestone 2 artifact digest and
equivalence contracts instead of inventing a reload-local comparison heuristic.

**Relevant subsystems**

- artifact equivalence consumption
- digest comparison
- semantic sameness classification
- replay parity

**Relevant APIs**

- `WorthUiArtifactEquivalence`
- `WorthUiArtifactDigest`
- `WorthUiRuntimeArtifactComparison`
- `WorthUiRuntimeEquivalenceBasis`

**Warnings**

- Do not use tree pointer identity, source order, file paths, or debug output
  as reload equivalence.
- Do not make diagnostics/provenance richness alter canonical replacement
  sameness.
- Do not compare active plans before artifact equivalence has been consumed.

**Test requirements**

- `same_artifact_equivalence_basis_produces_same_runtime_comparison`: the same
  active and candidate artifacts compare equivalently under replay.
- `diagnostic_richness_does_not_change_runtime_artifact_comparison`: richer
  candidate diagnostics or provenance do not change semantic equivalence.
- `meaningful_artifact_difference_classified_before_impact_narrowing`: a real
  semantic artifact difference is visible before dependency impact logic runs.
- `same_digest_with_mismatched_equivalence_basis_rejected`: a candidate cannot
  rely on digest text alone when the equivalence basis or support posture does
  not match the active runtime basis.

**Engineering decisions**

- Runtime comparison consumes the M2 equivalence basis and may add runtime
  activation context, but cannot redefine artifact sameness.
- No-op replacement is a first-class outcome so reload storms can avoid useless
  plan work.
- Artifact comparison produces typed evidence that later impact narrowing and
  diagnostics consume.

**Open questions**

- None.

### Phase 5: Replacement Impact Classification Boundary

Freeze the runtime vocabulary for no-op, local subtree, structural replacement,
broad replacement, lane-affecting replacement, and unsupported impact.

**Relevant subsystems**

- impact classification
- replacement scope
- lane-impact posture
- broad replacement denials

**Relevant APIs**

- `WorthUiReplacementImpact`
- `WorthUiReplacementScope`
- `WorthUiLaneImpactClassification`
- `WorthUiUnsupportedReplacementImpact`
- `WorthUiCommandImpact`
- `WorthUiTokenThemeImpact`
- `WorthUiAccessibilityImpact`
- `WorthUiRendererResourceImpact`

**Warnings**

- Do not let every semantic change imply a full runtime replacement by default.
- Do not classify lane-affecting changes as local subtree edits.
- Do not let broad replacement silently drop durable state without receipts.
- Do not ignore non-tree impacts: command bindings, token/theme references,
  accessibility metadata, renderer resources, diagnostics policy, Query
  bindings, and durable state families all need named impact posture.

**Test requirements**

- `equivalent_artifact_changes_classify_to_noop`: equivalent artifacts produce
  a no-op impact classification without plan rebuild work.
- `lane_affecting_change_classified_before_plan_lowering`: changes that move a
  surface between execution lanes are identified before lane plans are built.
- `unsupported_impact_denied_without_mutating_active_state`: unsupported impact
  classes fail closed and preserve the active plan.
- `broad_replacement_without_state_drop_receipts_rejected`: a broad replacement
  cannot proceed unless every affected durable state family has explicit
  preserve, replace, drop, or create evidence.
- `theme_or_command_only_change_does_not_force_full_tree_replacement`:
  command-binding and token/theme changes classify to their narrowest honest
  impact rather than defaulting to broad replacement.

**Engineering decisions**

- Impact classification is a replacement-planning phase, not a plan-lowering
  implementation detail.
- Lane-affecting replacement is admitted only when later lane parity and state
  replacement rules can prove safe semantics.
- Broad replacement is allowed, but it must be explicit and receipt-backed.
- Unsupported impact classes are named denials, including unsupported lane,
  Query posture, state family, source cause, capability snapshot, renderer
  resource, and plan-node family.

**Open questions**

- None.

### Phase 6: Dependency Impact Narrowing Boundary

Freeze how runtime replacement consumes Milestone 2 dependency metadata to
avoid broad tree rediscovery and to explain which artifact subtrees, source
modules, bindings, and lanes are affected.

**Relevant subsystems**

- dependency impact metadata
- subtree invalidation basis
- source-module impact lookup
- runtime narrowing counters

**Relevant APIs**

- `WorthUiIncrementalInvalidationBasis`
- `WorthUiRuntimeImpactNarrowing`
- `WorthUiArtifactSubtreeDigest`
- `WorthUiImpactLookupCounters`
- `WorthUiCommandBindingInvalidation`
- `WorthUiTokenInvalidation`
- `WorthUiAccessibilityInvalidation`
- `WorthUiRendererResourceInvalidation`
- WORTH Query `Aspects And Authority Lanes`
- WORTH Query `Signal Compatibility And Continuation`
- WORTH Query `Region-Scoped Live Invalidation And Stream Contracts`
- WORTH Query `Live Views And Live Promotion`
- WORTH Query `Async Resources And Result State`

**Warnings**

- Do not use filesystem diffs as the authoritative impact graph.
- Do not infer dependency impact through recursive full-artifact scans when M2
  metadata already states the relationship.
- Do not collapse UI artifact dependency truth with Query/signal dependency
  truth.
- Do not treat affected source modules, artifact subtrees, command bindings,
  tokens/themes, accessibility refs, Query bindings, state families, lane
  assignments, and renderer resources as one undifferentiated invalidation set.

**Test requirements**

- `equivalent_dependency_metadata_produces_equivalent_runtime_impact`: replayed
  candidates with equivalent dependency metadata produce equivalent impact
  narrowing.
- `changed_module_impact_lookup_does_not_scan_full_artifact`: representative
  impact lookup exposes counters proving it does not traverse the entire
  artifact.
- `runtime_owned_query_dependency_links_preserved_during_impact_narrowing`:
  Query/signal linkage metadata survives narrowing as typed references rather
  than UI-local edges.
- `query_bound_change_cannot_be_narrowed_by_ui_subtree_only`: a Query-bound
  artifact change is rejected when impact narrowing ignores Query-owned
  invalidation or live-view linkage.
- `renderer_resource_change_cannot_broaden_unrelated_widget_subtrees`:
  renderer-resource invalidation narrows to admitted canvas/real-time lane
  surfaces instead of rebuilding unrelated ordinary widgets.

**Engineering decisions**

- Runtime narrowing consumes candidate dependency metadata and active artifact
  metadata; it does not author dependency truth.
- Narrowing counters are emitted as proof evidence because this phase directly
  protects the hot iteration experience.
- Query-owned dependency or invalidation posture is referenced, not rebuilt.
- Dependency narrowing may consume custom source-provider metadata only after
  it has been lowered into the same candidate/dependency envelope as ordinary
  file-authored source.

**Open questions**

- None.

### Phase 7: Identity Match Graph Boundary

Freeze the match graph between active artifact node identity and candidate node
identity before any durable state can be considered for carry-forward.

**Relevant subsystems**

- stable identity matching
- artifact node correspondence
- replacement graph construction
- identity match diagnostics

**Relevant APIs**

- `WorthUiIdentityMatchGraph`
- `WorthUiIdentityMatchNode`
- `WorthUiArtifactIdentitySeed`
- `WorthUiIdentityMatchReport`
- `WorthUiIdentitySeedContribution`
- `WorthUiRepeatedTemplateIdentity`
- `WorthUiMovedNodeIdentity`

**Warnings**

- Do not match nodes by source order, display label, widget type alone, or
  geometry.
- Do not allow multiple candidate nodes to claim the same active durable state.
- Do not preserve state before identity matching has produced typed evidence.
- Do not let components override match results; component hooks may contribute
  identity seeds, but the runtime owns matching and ambiguity denial.

**Test requirements**

- `same_identity_seeds_produce_same_match_graph`: replaying active/candidate
  artifacts with the same identity seeds produces the same match graph.
- `duplicate_candidate_identity_rejected_before_state_reconciliation`: duplicate
  stable identities fail before state can be carried forward.
- `source_reordering_does_not_change_identity_match_graph`: legal source/module
  reordering does not alter node correspondence.
- `same_label_same_component_different_identity_does_not_preserve_state`: two
  visually similar nodes cannot match merely because label and component family
  are equal.
- `repeated_template_items_preserve_only_keyed_identity`: repeated or templated
  nodes preserve identity only through stable item keys and template identity,
  not through list position.
- `moved_node_identity_survives_parent_or_region_change_when_seed_is_stable`:
  a node moved between regions can match only when stable identity and movement
  posture prove it is the same runtime entity.

**Engineering decisions**

- The identity match graph consumes M2 identity seeds and runtime active
  artifact identity, not UI state.
- Match graph construction produces proof for later replacement and
  reconciliation phases.
- Ambiguous identity is a denial candidate, not a best-effort warning.
- Identity seed contribution is the narrow hook for special components; it is
  admitted before matching and cannot directly preserve state.

**Open questions**

- None.

### Phase 8: Replacement Classification Boundary

Freeze the per-node replacement vocabulary: preserve, replace, drop, create,
move, rebind, lane-change, and ambiguous.

**Relevant subsystems**

- replacement classification
- node lifecycle transitions
- lane transition markers
- ambiguity denial

**Relevant APIs**

- `WorthUiNodeReplacementClassification`
- `WorthUiNodeReplacementPlan`
- `WorthUiNodeLifecycleTransition`
- `WorthUiAmbiguousReplacementDenial`

**Warnings**

- Do not treat move and preserve as the same state transition when layout or
  lane semantics change.
- Do not let dropped nodes leak durable state.
- Do not let created nodes inherit state unless an explicit restoration
  authority says so.

**Test requirements**

- `same_match_graph_produces_same_replacement_classification`: equivalent match
  graphs and impact classifications produce equivalent node replacement plans.
- `ambiguous_node_replacement_denied_before_reconciliation`: ambiguity in
  replacement classification blocks durable state carry-forward.
- `lane_change_classified_separately_from_structural_move`: movement within a
  lane and movement across lanes produce distinct classifications.
- `drop_then_create_same_structural_path_does_not_claim_preserve`: deleting and
  recreating a node at the same parent/slot path cannot masquerade as a
  preserve transition without matching identity evidence.

**Engineering decisions**

- Replacement classification is the bridge between identity matching and state
  reconciliation.
- The classification plan is typed and inspectable because later diagnostics
  and certification depend on it.
- Lane changes are explicit even when semantic artifact meaning is preserved.

**Open questions**

- None.

### Phase 9: Durable State Inventory Boundary

Freeze the platform-owned inventory of durable UI state families and their
ownership, eligibility, persistence, replacement, and lane constraints.

**Relevant subsystems**

- durable UI state registry
- focus state ownership
- scroll and selection state ownership
- shell-local interaction state classification

**Relevant APIs**

- `WorthUiDurableStateFamily`
- `WorthUiDurableStateInventory`
- `WorthUiDurableStateEligibility`
- `WorthUiStateOwnershipClass`
- `WorthUiDurableStateFamilyHook`
- `WorthUiTransientInteractionState`
- `WorthUiStatePersistencePosture`

**Warnings**

- Do not let arbitrary widgets claim durable state by convention.
- Do not treat durable UI state as authoritative domain truth.
- Do not collapse focus, scroll, selection, text input, splitter, tab, and panel
  visibility state into one generic state bag.
- Do not preserve hover, pressed, drag capture, pointer capture, animation tick,
  or in-flight gesture state unless a state-family hook explicitly admits a
  bounded transient restoration rule.

**Test requirements**

- `durable_state_inventory_replay_is_deterministic`: the same platform state
  family inventory produces the same eligibility report under replay.
- `state_family_without_owner_identity_rejected`: a durable state family cannot
  be admitted without explicit owner identity and replacement posture.
- `domain_truth_state_cannot_enter_durable_ui_state_inventory`: compile-fail or
  typed denial coverage proves authoritative runtime truth cannot be stored as
  UI-local durable state.
- `durable_state_family_without_replacement_policy_cannot_be_registered`:
  state families must declare preserve/drop/replace behavior before they can
  participate in reload.
- `transient_interaction_state_defaults_to_drop`: hover, pressed, capture,
  drag, animation, and gesture state drop by default unless an admitted family
  rule proves safe bounded restoration.

**Engineering decisions**

- Durable UI state inventory is platform law and must exist before per-state
  reconciliation.
- State families carry explicit owner identity and replacement rules.
- Persistence posture is recorded but full persistence belongs to the later
  settings/project/workspace milestone.
- M3 explicitly supports focus chain, scroll anchors, selection ranges, text
  editing buffers, splitter positions, tab state, panel visibility, and
  bounded custom state families; domain truth and open-ended widget bags are
  rejected.

**Open questions**

- None.

### Phase 10: Durable State Reconciliation Boundary

Freeze the reconciliation rules for carrying focus, scroll position, selection,
panel visibility, splitter position, tab state, and text input state across
admitted replacements.

**Relevant subsystems**

- state reconciliation planner
- state preservation receipts
- per-family reconciliation rules
- state replacement diagnostics

**Relevant APIs**

- `WorthUiDurableStateReconciliationPlan`
- `WorthUiDurableStateCarryForward`
- `WorthUiDurableStateReplacement`
- `WorthUiDurableStateReconciliationReceipt`
- `WorthUiFocusChainReconciliation`
- `WorthUiScrollAnchorReconciliation`
- `WorthUiSelectionRangeReconciliation`
- `WorthUiTextEditStateReconciliation`

**Warnings**

- Do not preserve state when replacement classification says replace, drop, or
  ambiguous.
- Do not apply one generic carry-forward rule to state families with different
  failure modes.
- Do not let dropped nodes leave orphan state.
- Do not preserve drag/capture/gesture state through a reload by default; most
  pointer-derived state is intentionally transient.

**Test requirements**

- `equivalent_identity_and_state_inputs_preserve_equivalent_state`: equivalent
  replacement plans and state inventories produce equivalent carry-forward
  receipts.
- `ambiguous_or_replaced_identity_drops_state_with_receipt`: ambiguous,
  replaced, or dropped nodes do not preserve durable state and produce explicit
  replacement evidence.
- `text_input_state_not_preserved_across_incompatible_component_shape`: input
  state fails closed when component or prop shape makes preservation unsafe.
- `orphan_state_removed_after_node_drop`: dropped nodes leave no focus, scroll,
  selection, text, splitter, tab, or visibility state residue in active runtime
  state.
- `scroll_anchor_survives_reorder_only_with_stable_target_identity`: scroll
  preservation follows admitted anchor identity, not numeric offset alone.
- `selection_range_rejected_when_backing_collection_identity_changes`:
  selection state does not survive when the selected item basis changed without
  an admitted Query or component selection posture.

**Engineering decisions**

- State reconciliation consumes replacement classification; it never performs
  its own identity matching.
- Every preserved, replaced, dropped, or created state entry must be receipt
  visible.
- State carry-forward counters are required so hostile reload certification can
  detect accidental blanket preservation or blanket loss.
- Custom state-family reconciliation hooks can only choose among preserve,
  replace, drop, or recreate outcomes declared by the family inventory.

**Open questions**

- None.

### Phase 11: Query Binding Comparison Boundary

Freeze the comparison of Query-bound artifact references before any subscription
or live binding preservation is planned.

**Relevant subsystems**

- Query binding reference comparison
- view binding identity
- runtime posture comparison
- projection-consumption linkage

**Relevant APIs**

- `WorthUiQueryBindingComparison`
- `WorthUiQueryBindingIdentity`
- `WorthUiQueryBindingPosture`
- WORTH Query `Support Matrix And Admission`
- WORTH Query `Basis Capability Lifecycle`
- WORTH Query `Typed Binding And Retained Artifact Reuse`
- WORTH Query `Live Views And Live Promotion`
- WORTH Query `Projection Consumption`
- WORTH Query `Async Resources And Result State`
- WORTH Query `Recovery`
- WORTH Query `Inspection`

**Warnings**

- Do not compare Query bindings by rendered labels or local widget state.
- Do not flatten Query support, admission, async/result, recovery, or inspection
  posture into booleans.
- Do not treat changed UI placement as changed Query meaning unless the typed
  binding identity changed.

**Test requirements**

- `same_query_owned_binding_identity_preserves_binding_comparison`: equivalent
  Query-owned binding identities compare equivalently across reload.
- `query_binding_posture_drift_detected_before_subscription_reuse`: support,
  admission, async/result, or recovery posture drift is detected before live
  subscription reuse.
- `query_binding_comparison_does_not_use_ui_local_status_enums`: compile-fail
  or API proof prevents local loading/retry/cancelled enums from replacing
  Query posture.
- `same_rendered_query_label_with_different_query_basis_rebinds_or_denies`:
  matching UI labels cannot hide changed Query basis, support, projection, or
  async/result posture.

**Engineering decisions**

- Query binding comparison is a separate phase because Query authority is not UI
  state reconciliation.
- Worth UI may preserve typed references, but Query owns the meaning and support
  posture of those references.
- Projection-consumption and inspection links remain typed and are not
  re-derived from displayed rows.

**Open questions**

- None.

### Phase 12: Query Live Rebind Planning Boundary

Freeze whether Query-bound surfaces preserve an existing live binding,
rebind through Query-owned handles, or deny activation because the candidate
would require a UI-local pseudo-runtime.

**Relevant subsystems**

- Query live binding preservation
- subscription rebind planning
- async/result posture carry-forward
- recovery and inspection linkage

**Relevant APIs**

- `WorthUiQueryLiveRebindPlan`
- `WorthUiQueryBindingPreservation`
- `WorthUiQueryBindingRebind`
- `WorthUiQueryBindingDriftDenial`
- WORTH Query `Live Views And Live Promotion`
- WORTH Query `Subscription Selection And Diagnostics`
- WORTH Query `Basis Capability Lifecycle`
- WORTH Query `Projection Consumption`
- WORTH Query `Async Resources And Result State`
- WORTH Query `Recovery`
- WORTH Query `Inspection`
- WORTH Query `Continuation Pipeline`

**Warnings**

- Do not keep stale subscriptions alive after the binding identity or basis
  changed.
- Do not rebind by reaching into lower runtime crates directly.
- Do not create Worth-UI-owned loading, retry, recovery, or explanation models.

**Test requirements**

- `same_query_binding_basis_preserves_live_binding`: equivalent Query binding
  basis preserves live binding through a typed preservation receipt.
- `query_basis_drift_requires_rebind_or_denial`: changed Query basis produces
  an explicit rebind plan or typed denial.
- `ui_local_subscription_recovery_path_rejected`: attempts to recover by local
  UI subscription or status state fail closed.
- `stale_query_subscription_handle_cannot_be_preserved_after_basis_drift`:
  reload cannot keep an old live handle when Query basis or support posture has
  changed.

**Engineering decisions**

- Rebind planning consumes Query binding comparison and runtime support posture.
- Query owns live, async/result, recovery, projection-consumption, and
  inspection semantics; Worth UI owns presentation and active-plan references.
- Rebind planning output is part of activation staging, not a frame-time choice.

**Open questions**

- None.

### Phase 13: Activation Staging Boundary

Freeze the staging state that holds an admitted replacement candidate,
replacement impact, identity/reconciliation plans, Query rebind plans, and
pending execution-plan lowering input before a frame-boundary swap is allowed.

**Relevant subsystems**

- pending activation staging
- staged replacement bundle
- activation readiness checks
- staging diagnostics

**Relevant APIs**

- `WorthUiPendingActivation`
- `WorthUiStagedReplacement`
- `WorthUiActivationReadiness`
- `WorthUiActivationStagingReport`
- WORTH Query support/admission receipts carried by Query rebind plans
- WORTH Query basis, live-view, async/result, recovery, and inspection
  artifacts carried as staged activation inputs

**Warnings**

- Do not mutate active runtime state during staging.
- Do not let the frame loop pull partially staged replacement data.
- Do not allow activation staging to re-run earlier admission or reconciliation
  logic by convention.

**Test requirements**

- `equivalent_replacement_inputs_produce_equivalent_pending_activation`: the
  same admitted candidate, impact, state, and Query plans produce equivalent
  staged activation bundles.
- `partial_replacement_bundle_cannot_be_activated`: missing reconciliation,
  Query rebind, or plan-lowering inputs prevent activation.
- `staging_does_not_mutate_active_runtime_state`: failed staging leaves active
  artifact, plan, and durable state untouched.
- `staging_rejects_reconciliation_plan_from_different_candidate`: activation
  staging fails if the state or Query rebind plan was produced for a different
  candidate digest.

**Engineering decisions**

- Staging is the lifecycle boundary between replacement planning and plan
  lowering/activation.
- Pending activation is move-only where practical so partial clones do not
  create false observers.
- Staging reports are diagnostics inputs, not active runtime truth.

**Open questions**

- None.

### Phase 14: Execution-Plan Input Boundary

Freeze the typed input that execution-plan lowering consumes from a staged
replacement so plan lowering cannot read raw source, mutable registries, or
unproven artifact state.

**Relevant subsystems**

- plan lowering entry
- canonical artifact consumption
- capability handle consumption
- staged replacement consumption

**Relevant APIs**

- `WorthUiExecutionPlanInput`
- `WorthUiPlanLoweringContext`
- `WorthUiPlanLoweringBasis`
- `WorthUiPlanLoweringDenial`
- `WorthUiPlanNodeInput`
- `WorthUiComponentLoweringHook`
- `WorthUiEguiBoundaryInput`
- WORTH Query typed binding/resolver artifacts carried by Query-bound plan
  inputs
- WORTH Query projection-consumption receipts carried by Query-bound plan inputs
- WORTH Query async/result and recovery posture carried by Query-bound plan
  inputs

**Warnings**

- Do not let plan lowering inspect source text or parse diagnostics.
- Do not resolve commands, components, tokens, or bindings from strings at plan
  lowering time.
- Do not accept canonical artifacts without staged replacement proof when the
  plan is intended for active runtime replacement.
- Do not let component lowering hooks emit arbitrary executable nodes; they
  lower into admitted plan-node families and denial types.

**Test requirements**

- `same_staged_artifact_produces_same_plan_input`: equivalent staged
  replacements produce equivalent plan-lowering input.
- `raw_source_or_unresolved_capability_cannot_enter_plan_lowering`: compile-fail
  or typed denial coverage proves plan lowering cannot consume raw source,
  mutable registries, or unresolved strings.
- `plan_input_preserves_query_owned_binding_handles`: Query-owned binding
  handles survive into plan input without local wrapper substitution.
- `plan_lowering_rejects_candidate_missing_activation_readiness`: a candidate
  cannot enter plan lowering with only an admitted artifact and no activation
  readiness bundle.
- `component_lowering_hook_cannot_emit_unregistered_plan_node_family`: custom
  component lowering fails when it tries to branch outside the admitted M3 plan
  node families.

**Engineering decisions**

- Plan-lowering input is the bridge from semantic artifact authority to
  executable runtime mechanics.
- Plan lowering may consume compact resolved handles, but it must not perform
  capability resolution.
- Replacement context is carried because plan output must preserve activation
  and reconciliation evidence.
- M3 plan input admits component invocation, child ranges, command handles,
  token/style handles, layout region handles, Query/view handles,
  accessibility handles, diagnostic refs, lane partition refs, egui boundary
  refs, and render-resource refs.

**Open questions**

- None.

### Phase 15: Compact Runtime Handle Boundary

Freeze compact runtime handles for components, commands, tokens, child ranges,
view bindings, lanes, and state slots so the frame path does not resolve strings
or scan broad registries.

**Relevant subsystems**

- runtime handle allocation
- command and component handles
- token and style handles
- child range and binding handles

**Relevant APIs**

- `WorthUiRuntimeHandle`
- `WorthUiComponentHandle`
- `WorthUiCommandHandle`
- `WorthUiTokenHandle`
- `WorthUiChildRangeHandle`
- `WorthUiViewBindingHandle`

**Warnings**

- Do not expose raw handle internals as public semantic identity.
- Do not allocate handles per frame.
- Do not use hash lookups where plan-local index handles are the honest cost
  model.

**Test requirements**

- `equivalent_plan_inputs_allocate_equivalent_runtime_handles`: equivalent
  plan-lowering inputs allocate equivalent compact handles.
- `frame_path_cannot_resolve_component_or_command_by_string`: compile-fail,
  API, or counter proof prevents string resolution on the steady frame path.
- `handle_allocation_reports_cardinality_and_collision_denials`: handle
  allocation reports family widths and rejects duplicate or colliding plan-local
  claims.
- `handle_reuse_after_lane_change_requires_new_plan_receipt`: a lane-changing
  replacement cannot reuse stale plan-local handles without fresh allocation
  evidence.

**Engineering decisions**

- Runtime handles are execution mechanics derived from canonical artifact
  meaning.
- Handles can be rebuilt from active artifact and plan input; they are not
  semantic authority.
- Handle allocation counters are required for performance certification.

**Open questions**

- None.

### Phase 16: Plan Topology Assembly Boundary

Freeze the executable plan topology: node order, child ranges, region
structure, traversal groups, lane partitioning, and activation-local lookup
tables.

**Relevant subsystems**

- execution-plan topology
- traversal order
- lane partitioning
- plan-local indexes

**Relevant APIs**

- `WorthUiExecutionPlan`
- `WorthUiPlanNode`
- `WorthUiPlanTopology`
- `WorthUiPlanLanePartition`
- `WorthUiPlanLookupIndex`
- `WorthUiPlanNodeFamily`
- `WorthUiEguiPlanBoundary`
- `WorthUiRenderResourceRef`

**Warnings**

- Do not let frame traversal rediscover child relationships from artifact
  trees.
- Do not collapse lane partitioning into renderer-time branches.
- Do not make plan-local lookup tables mutable from app code.
- Do not hide egui integration as ambient access; `Context`, `Ui`, `Response`,
  `Id`, input, layout allocation, paint submission, memory/state bridge, and
  frame timing contact must be represented as plan or lane boundary inputs.

**Test requirements**

- `equivalent_plan_inputs_assemble_equivalent_topology`: equivalent plan inputs
  produce equivalent plan topology and traversal order.
- `plan_topology_assembly_rejects_missing_child_or_lane_links`: missing child
  ranges, invalid region links, or unsupported lane partitions fail before
  activation.
- `frame_traversal_uses_plan_topology_without_artifact_tree_scan`: counters or
  API proof show steady frames consume plan topology rather than scanning the
  canonical artifact tree.
- `plan_topology_rejects_orphaned_child_range_handles`: child range handles that
  point outside the assembled topology fail before activation.
- `egui_boundary_contact_is_plan_declared_not_ambient`: plan topology rejects
  nodes that require egui context, input, memory, paint, or frame timing access
  without an admitted egui boundary declaration.

**Engineering decisions**

- Plan topology is executable structure derived from canonical artifact
  structure.
- Lane partitioning is assembled before activation so lane execution is not a
  speculative frame-time decision.
- Plan-local lookup indexes are private mechanics with counters for proof.
- The egui boundary is a lower runtime contact surface, not Worth UI semantic
  authority; it must remain typed enough for frame-cost and diagnostics proof.

**Open questions**

- None.

### Phase 17: Plan Equivalence And Digest Boundary

Freeze execution-plan equivalence and digest semantics so plan reuse, no-op
activation, lane parity, and certification have an explicit sameness contract.

**Relevant subsystems**

- plan digesting
- plan equivalence
- plan reuse posture
- active plan comparison

**Relevant APIs**

- `WorthUiExecutionPlanDigest`
- `WorthUiExecutionPlanEquivalence`
- `WorthUiExecutionPlanEquivalenceBasis`
- `WorthUiPlanReuseClassification`

**Warnings**

- Do not infer plan sameness from artifact digest alone when execution lane,
  handle, or topology mechanics changed.
- Do not let diagnostics richness or profiling policy change plan digest.
- Do not let pointer identity stand in for plan equivalence.

**Test requirements**

- `same_plan_meaning_and_lane_basis_produces_same_plan_digest`: equivalent plan
  topology, handles, and lanes produce equivalent plan digests.
- `lane_or_handle_meaning_change_changes_plan_equivalence`: meaningful lane or
  handle changes alter plan equivalence even if visual output might look
  similar.
- `diagnostic_policy_does_not_change_execution_plan_digest`: diagnostics
  richness does not affect plan semantic/execution equivalence.
- `same_artifact_different_lane_partition_changes_plan_digest`: artifact
  equality alone cannot hide execution-lane partition changes.

**Engineering decisions**

- Plan equivalence is separate from artifact equivalence because mechanics and
  cost matter at the execution boundary.
- Plan digest covers executable runtime meaning, not observation detail.
- Later shell and tooling milestones must consume this contract instead of
  inventing local active-plan comparisons.

**Open questions**

- None.

### Phase 18: Plan Provenance And Inspection Boundary

Freeze the derived observation lane that explains how artifact nodes,
capabilities, identity matches, state reconciliation, Query bindings, and lane
lowering produced execution-plan nodes.

**Relevant subsystems**

- plan inspection
- artifact-to-plan provenance
- lane inspection
- replacement diagnostics linkback

**Relevant APIs**

- `WorthUiExecutionPlanInspection`
- `WorthUiPlanNodeInspection`
- `WorthUiArtifactToPlanProvenance`
- `WorthUiLaneInspection`
- WORTH Query `Inspection`
- WORTH Query `Cross-Runtime Causal Inspection`
- WORTH Query `Projection Consumption`
- WORTH Query `Ordinary Outcomes`
- WORTH Query lower-runtime boundary envelopes where plan diagnostics explain
  admitted lower-runtime contact

**Warnings**

- Do not make plan inspection authoritative runtime meaning.
- Do not require source or Rust control-flow archaeology to explain active
  plan nodes.
- Do not duplicate Query inspection or cross-runtime explanation where typed
  Query artifacts already exist.

**Test requirements**

- `plan_inspection_explains_artifact_and_capability_origin`: representative
  plan nodes can name the artifact nodes and registered capabilities that
  produced them.
- `plan_provenance_replay_is_deterministic`: replaying the same staged
  replacement produces the same provenance relationships.
- `query_owned_inspection_links_are_preserved_not_reauthored`: Query inspection
  and projection-consumption references are preserved as typed links instead
  of UI-local explanation records.
- `plan_inspection_rejects_unlinked_query_explanation_records`: plan inspection
  cannot attach free-form Query explanations that are not backed by Query-owned
  inspection or causal-inspection artifacts.

**Engineering decisions**

- Plan inspection is a derived view over active execution-plan truth.
- Inspection must be rich enough for developer tooling and certification but
  must not add frame-path cost by default.
- Provenance links replacement, artifact, plan, and lane facts without turning
  diagnostics into authority.

**Open questions**

- None.

### Phase 19: Execution Lane Taxonomy And Hook Admission Boundary

Freeze the execution lane taxonomy, support posture, and admitted hook points
for ordinary widget and shell surfaces, virtualized data surfaces,
canvas/spatial surfaces, real-time overlay/HUD surfaces, and typed special-case
extensions.

**Relevant subsystems**

- execution lane taxonomy
- lane support posture
- lane admission
- lane diagnostics
- hook admission

**Relevant APIs**

- `WorthUiExecutionLane`
- `WorthUiExecutionLaneSupport`
- `WorthUiLaneAdmission`
- `WorthUiLaneSupportDiagnostic`
- `WorthUiExtensionHookAdmission`
- `WorthUiLaneAdapterHook`
- `WorthUiUnsupportedHookDenial`
- WORTH Query `Support Matrix And Admission`
- WORTH Query `Query Operating Modes`
- WORTH Query runtime-backed support rows for Query-bound lane inputs

**Warnings**

- Do not make lanes visual categories; they are cost and failure-mode regimes.
- Do not let every custom component invent a private lane.
- Do not treat unsupported lane posture as a renderer fallback.
- Do not call hook admission an escape hatch unless it produces typed inputs,
  denials, receipts, and counters required by the phase it extends.

**Test requirements**

- `equivalent_lane_descriptors_produce_equivalent_lane_support`: equivalent
  lane support declarations produce equivalent lane support posture.
- `unsupported_lane_reference_rejected_before_plan_activation`: plan nodes
  referencing unsupported lanes fail before activation.
- `lane_taxonomy_distinguishes_cost_and_failure_modes`: ordinary, virtualized
  data, canvas/spatial, and real-time lanes produce distinct typed posture.
- `private_component_lane_claim_rejected_without_lane_support`: component
  descriptors cannot smuggle a new execution lane through a string or custom
  tag.
- `hook_admission_rejects_active_plan_or_query_authority_override`: hooks that
  try to own active plan truth, Query posture, state carry-forward, lane
  taxonomy, or performance certification fail before activation.
- `admitted_lane_adapter_hook_preserves_lane_counter_contract`: a custom lane
  adapter can branch mechanics only while preserving lane support posture,
  forbidden-work counters, and certification receipts.

**Engineering decisions**

- The first execution-plan substrate includes all four platform lane families.
- Lanes specialize mechanics and counters, not canonical UI meaning.
- Lane support posture is machine-checkable because later component and shell
  work depends on it.
- Supported hooks in M3 are source ingress, debounce policy, identity seed
  contribution, durable state-family admission, component lowering, lane
  adapter mechanics, canvas/spatial draw and hit-test, real-time overlay
  mechanics, diagnostics projection, counter families, and report
  materialization.

**Open questions**

- None.

### Phase 20: Ordinary Widget And Shell Lane Boundary

Freeze the ordinary lane for standard widgets, panels, shell regions, command
surfaces, and non-virtualized layout/content traversal.

**Relevant subsystems**

- ordinary widget execution
- shell-region execution
- command-surface projection
- ordinary layout traversal

**Relevant APIs**

- `WorthUiOrdinaryExecutionLane`
- `WorthUiOrdinaryLanePlan`
- `WorthUiOrdinaryLaneFrameReceipt`
- `WorthUiOrdinaryLaneCounters`

**Warnings**

- Do not put virtualized data, canvas, or real-time behavior into ordinary lane
  fallback branches.
- Do not resolve commands, tokens, or components by string during ordinary lane
  execution.
- Do not let shell surface behavior become app-local execution logic.

**Test requirements**

- `equivalent_ordinary_lane_plans_execute_with_equivalent_traversal_counters`:
  equivalent ordinary plans produce equivalent lane traversal receipts and
  counters.
- `ordinary_lane_rejects_virtualized_or_canvas_surface_claims`: data-heavy and
  canvas/spatial surfaces cannot silently downgrade into ordinary execution.
- `ordinary_frame_path_does_not_parse_or_resolve_source`: counters prove the
  ordinary lane executes from handles and topology only.
- `ordinary_lane_counters_fail_when_widget_execution_scans_all_plan_nodes`:
  ordinary lane certification catches implementations that traverse unrelated
  plan nodes for a local surface.

**Engineering decisions**

- The ordinary lane is the baseline for shell and standard component execution.
- It remains execution mechanics over active plans, not a place to redefine
  shell semantics.
- Ordinary lane receipts are required even before the full application shell
  milestone.

**Open questions**

- None.

### Phase 21: Virtualized Data Lane Boundary

Freeze the virtualized data lane for table, grid, list, and similar high-density
surfaces that must execute by visible/touched range rather than full data or
full node breadth.

**Relevant subsystems**

- virtualized row/column execution
- visible range planning
- query-shaped patch handoff
- data lane counters

**Relevant APIs**

- `WorthUiVirtualizedDataLane`
- `WorthUiVirtualizedDataPlan`
- `WorthUiVisibleRange`
- `WorthUiVirtualizedDataFrameReceipt`
- `WorthUiVirtualizedDataCounters`
- WORTH Query `Collections, Cursors, Ordering, And Aggregation Reads`
- WORTH Query `Scopes, Templates, View Shapes, And Saved Queries`
- WORTH Query `Read Composition`
- WORTH Query `Live Views And Live Promotion`
- WORTH Query `Projection Consumption`
- WORTH Query `Async Resources And Result State`
- WORTH Query `Subscription Selection And Diagnostics`

**Warnings**

- Do not loop over full collections on the frame path.
- Do not implement pagination, ordering, cursor, async/result, or live patch
  semantics locally when Query owns those meanings.
- Do not let virtualization change table or list semantic artifact meaning.

**Test requirements**

- `equivalent_visible_range_inputs_produce_equivalent_data_lane_receipts`:
  equivalent data plans and visible ranges produce equivalent virtualized frame
  receipts.
- `data_lane_rejects_full_collection_frame_scan`: hostile data surfaces fail
  certification if frame counters show full collection traversal.
- `query_shaped_patch_posture_preserved_in_data_lane`: Query-owned live patch,
  async/result, and projection posture survive lane execution without UI-local
  wrappers.
- `virtualized_data_lane_rejects_offset_pagination_as_cursor_substitute`:
  table/grid execution cannot replace Query cursor or ordering posture with
  UI-local offset pagination.

**Engineering decisions**

- Virtualization is a first-platform lane because Worth UI targets serious data
  consoles and workbenches.
- Data lane execution consumes Query-owned shape and runtime posture where data
  comes from Query-bound artifacts.
- Visible/touched range counters are mandatory acceptance evidence.

**Open questions**

- None.

### Phase 22: Canvas And Spatial Lane Boundary

Freeze the canvas/spatial lane for pan, zoom, hit-test, overlay, selection,
tool-state, and command-integrated surfaces without making Worth UI own a full
scene renderer.

**Relevant subsystems**

- canvas lane planning
- spatial hit-test execution
- overlay planning
- tool-state attachment

**Relevant APIs**

- `WorthUiCanvasSpatialLane`
- `WorthUiCanvasSpatialPlan`
- `WorthUiCanvasViewportRequest`
- `WorthUiSpatialHitTestRequest`
- `WorthUiLaneHandle` for overlay execution
- `WorthUiCanvasSpatialFrameReceipt`
- `WorthUiCanvasDrawHook`
- `WorthUiSpatialHitTestHook`
- `WorthUiSpatialToolStateHook`
- WORTH Query `Graph Composition Authoring`
- WORTH Query `Structural Correspondence And Historical Materialization`
- WORTH Query `Projection Consumption`
- WORTH Query `Basis Capability Lifecycle`
- WORTH Query `Inspection`
- WORTH Query `Cross-Runtime Causal Inspection`
- WORTH Query preview/branch session identity surfaces where spatial workflows
  bind to speculative or branch-local meaning

**Warnings**

- Do not make the canvas lane a disconnected side runtime.
- Do not let spatial tool state bypass command, focus, accessibility, or
  diagnostics posture.
- Do not make Worth UI the owner of volatile scene truth or domain geometry
  authority.
- Do not allow draw or hit-test hooks to read arbitrary domain geometry or
  renderer internals outside admitted lane inputs.

**Test requirements**

- `equivalent_canvas_plans_produce_equivalent_spatial_lane_receipts`:
  equivalent canvas/spatial plans produce equivalent hit-test and overlay
  receipts under replay.
- `canvas_lane_rejects_domain_truth_or_scene_renderer_ownership`: attempts to
  store domain geometry truth or renderer internals in Worth UI lane state fail
  closed.
- `spatial_lane_preserves_command_and_selection_identity`: command and
  selection identity survive canvas execution through typed platform handles.
- `canvas_hit_test_cannot_read_domain_geometry_truth_directly`: hit-test
  execution must consume admitted lane/tool inputs and cannot bypass Query or
  domain authority by fishing in lower truth state.
- `custom_canvas_draw_hook_preserves_platform_identity_and_counters`: custom
  draw mechanics can attach only through admitted lane inputs and must produce
  frame receipts that preserve command, selection, diagnostics, and cost
  accounting.

**Engineering decisions**

- Canvas/spatial lane is included now because CAD/topology/editor products are
  part of the platform target, not later ornaments.
- Worth UI owns the UI-facing lane mechanics: interaction, overlays, hit-test
  posture, command integration, and counters.
- Domain geometry, scene renderer internals, and authoritative spatial truth
  remain outside Worth UI authority.
- Canvas/spatial hooks are intentionally narrow: draw, hit-test, overlay, and
  tool-state mechanics, never scene truth ownership.

**Open questions**

- None.

### Phase 23: Real-Time Overlay And HUD Lane Boundary

Freeze the real-time overlay/HUD lane for high-frequency visual surfaces that
must avoid ordinary widget mechanics while still participating in platform
identity, command, diagnostics, and frame-cost accounting.

**Relevant subsystems**

- real-time overlay planning
- HUD lane execution
- renderer-facing surface handoff
- high-frequency frame counters

**Relevant APIs**

- `WorthUiRealtimeOverlayLane`
- `WorthUiHudPlan`
- `WorthUiRendererSurfaceHandle`
- `WorthUiRealtimeFrameReceipt`
- `WorthUiRealtimeLaneCounters`
- `WorthUiRealtimeOverlayHook`
- `WorthUiRendererSurfaceAdmission`
- `WorthUiHighFrequencyFramePolicy`

**Warnings**

- Do not route real-time overlays through ordinary widget traversal.
- Do not let renderer-facing handles become untracked native integration
  backdoors.
- Do not allow high-frequency surfaces to skip diagnostics and cost receipts.
- Do not let real-time hooks opt out of identity, command, accessibility, or
  diagnostics posture just because they are high frequency.

**Test requirements**

- `equivalent_realtime_plans_produce_equivalent_hud_receipts`: equivalent
  real-time overlay plans produce equivalent high-frequency frame receipts.
- `realtime_lane_rejects_ordinary_widget_fallback`: real-time surfaces cannot
  silently execute through ordinary lane mechanics.
- `renderer_surface_handle_does_not_bypass_platform_identity`: renderer-facing
  handles remain bound to platform identity and diagnostics posture.
- `realtime_lane_counter_detects_hidden_ordinary_layout_pass`: real-time lane
  certification fails if overlay execution secretly performs ordinary layout
  traversal.
- `custom_realtime_hook_cannot_suppress_forbidden_work_counters`: a real-time
  overlay hook cannot hide source parsing, registry lookup, ordinary layout,
  allocation, or diagnostic materialization counters.

**Engineering decisions**

- Real-time overlays and HUD surfaces are part of the platform foundation
  because Worth UI targets simulation, visualization, and high-frequency tools.
- This lane owns UI-facing execution contracts, not full renderer ownership.
- Real-time cost counters must be separate from ordinary lane counters.
- Renderer-facing surface admission is allowed only as a typed handoff with
  frame policy, handle identity, and certification evidence.

**Open questions**

- None.

### Phase 24: Cross-Lane Meaning Parity Boundary

Freeze the proof that lane specialization changes execution mechanics and cost,
not canonical UI meaning, command meaning, Query binding meaning, accessibility
posture, or diagnostics identity.

**Relevant subsystems**

- cross-lane parity
- lane transition validation
- shared semantic references
- lane-specific mechanics comparison

**Relevant APIs**

- `WorthUiLaneMeaningParity`
- `WorthUiLaneTransitionParity`
- `WorthUiCrossLaneSemanticReference`
- `WorthUiLaneParityReport`
- WORTH Query command/readiness, support/admission, binding, projection
  consumption, async/result, recovery, and inspection artifacts referenced by
  lane-shared semantic handles

**Warnings**

- Do not let each lane become a shadow UI runtime.
- Do not duplicate command readiness, Query binding, focus identity, or
  diagnostics meaning per lane.
- Do not claim parity from visual similarity.

**Test requirements**

- `same_artifact_meaning_preserved_across_admitted_lane_transition`: admitted
  lane transitions preserve canonical semantic references where meaning is
  unchanged.
- `lane_specific_command_or_query_semantics_rejected`: lane implementations
  cannot redefine command or Query binding semantics.
- `visual_similarity_without_semantic_parity_does_not_certify_lane_transition`:
  visual output equivalence alone is not accepted as lane parity evidence.
- `lane_transition_with_changed_query_binding_denied_without_query_rebind`:
  cross-lane parity cannot certify a transition if Query binding posture
  changed and no Query-owned rebind receipt exists.

**Engineering decisions**

- Cross-lane parity is required before lane-changing replacement can activate.
- Shared semantic handles remain upstream of lane mechanics.
- Lane parity reports are certification artifacts for future shell, component,
  and canvas work.

**Open questions**

- None.

### Phase 25: Safe Frame-Boundary Activation Boundary

Freeze the frame-boundary gate that decides when a staged, lowered, reconciled,
and lane-partitioned replacement may become the active execution plan.

**Relevant subsystems**

- frame-boundary activation
- pending plan readiness
- egui frame coordination
- activation gate diagnostics

**Relevant APIs**

- `WorthUiFrameActivationGate`
- `WorthUiFrameBoundary`
- `WorthUiReadyActivation`
- `WorthUiActivationGateReceipt`
- WORTH Query rebind receipts and basis posture carried by ready activations
- WORTH Query ordinary outcomes or checked stops for activation blockers tied
  to Query-owned posture

**Warnings**

- Do not activate in the middle of frame traversal.
- Do not let frame activation re-run semantic admission, identity matching, or
  Query rebind planning.
- Do not expose partially active plans to lane execution.

**Test requirements**

- `ready_activation_commits_only_at_safe_frame_boundary`: staged replacements
  become active only at declared safe frame boundaries.
- `mid_frame_activation_attempt_denied_without_state_mutation`: attempts to
  activate during frame traversal fail without mutating active runtime state.
- `activation_gate_receipt_names_plan_state_and_reconciliation_basis`: gate
  receipts bind active plan, state reconciliation, and Query rebind basis.
- `activation_gate_rejects_ready_plan_with_stale_frame_epoch`: a ready
  activation cannot commit against a frame boundary older than the plan
  readiness receipt.

**Engineering decisions**

- Frame-boundary activation is a lifecycle phase separate from plan lowering.
- Activation gates consume readiness receipts, not raw candidate data.
- Safe boundary semantics must align with egui's immediate frame model while
  remaining Worth-owned platform behavior.

**Open questions**

- None.

### Phase 26: Atomic Plan Swap And Prior-Valid Preservation Boundary

Freeze the atomic transition that moves the ready activation into active runtime
state and preserves the previous active plan as the rollback/diagnostic basis.

**Relevant subsystems**

- atomic active-plan swap
- previous active state preservation
- activation receipts
- failed swap rollback

**Relevant APIs**

- `WorthUiAtomicPlanSwap`
- `WorthUiPlanSwapReceipt`
- `WorthUiPriorValidPlan`
- `WorthUiPlanSwapRollback`
- WORTH Query binding preservation/rebind receipts carried across plan swaps
- WORTH Query live-view, async/result, recovery, and inspection references
  preserved or replaced by the atomic swap receipt

**Warnings**

- Do not update artifact, plan, durable state, and Query bindings in separate
  externally observable steps.
- Do not discard the prior valid plan until the new active state is complete.
- Do not treat rollback as recomputing from source.

**Test requirements**

- `atomic_swap_replaces_artifact_plan_state_and_bindings_together`: successful
  swaps update all active runtime surfaces atomically.
- `swap_failure_restores_prior_valid_plan_without_source_reparse`: forced swap
  failure restores prior valid active state without reparsing source or
  rebuilding from mutable registries.
- `plan_swap_receipt_binds_previous_and_next_active_digests`: swap receipts
  identify previous and next active artifact/plan digests.
- `partial_swap_injection_leaves_no_mixed_active_state`: injected failure after
  artifact update but before plan/state/binding update cannot leave mixed
  previous/new active state observable.

**Engineering decisions**

- Swap is atomic at Worth UI runtime state boundaries even though lower host
  mechanics may execute in steps internally.
- Prior valid state is retained as a typed runtime artifact and diagnostic
  source.
- Rollback consumes prior valid runtime state, not source inputs.

**Open questions**

- None.

### Phase 27: Reload Failure Preservation Boundary

Freeze failure behavior for invalid candidate reloads, denied activations,
failed plan lowering, failed reconciliation, and failed swaps so the active app
never blanks, corrupts, or partially mutates.

**Relevant subsystems**

- failure preservation
- reload denial result
- active state immutability on failure
- failure diagnostics basis

**Relevant APIs**

- `WorthUiReloadFailure`
- `WorthUiReloadPreservationReceipt`
- `WorthUiReloadDenial`
- `WorthUiFailedActivationReport`
- WORTH Query `Ordinary Outcomes`
- WORTH Query `Recovery`
- WORTH Query support/admission and checked-stop posture for Query-caused
  reload denials

**Warnings**

- Do not let invalid reloads clear active UI surfaces.
- Do not mutate durable state when candidate validation fails.
- Do not recover from failure by falling back to app-local rendering paths.

**Test requirements**

- `invalid_candidate_preserves_previous_active_plan`: invalid candidates leave
  active artifact, plan, state, and Query bindings unchanged.
- `failed_reconciliation_or_plan_lowering_preserves_prior_valid_state`: failures
  after candidate admission still preserve the prior valid active plan.
- `reload_failure_does_not_create_fallback_ui_runtime`: failure paths cannot
  route to local fallback rendering outside active plan authority.
- `repeated_invalid_reload_does_not_accumulate_state_or_subscription_residue`:
  repeated failures leave durable state and Query binding sets equal to the last
  valid active runtime state.

**Engineering decisions**

- Failure preservation is its own boundary because it is a product guarantee,
  not an exception handler detail.
- Every failure emits a typed preservation receipt.
- Diagnostics may update, but active UI meaning does not change on failure.

**Open questions**

- None.

### Phase 28: Reload And Plan Diagnostics Boundary

Freeze typed diagnostics for candidate admission, equivalence, impact
narrowing, identity matching, state reconciliation, Query rebind, plan lowering,
lane admission, activation, and swap failure.

**Relevant subsystems**

- reload diagnostics
- plan diagnostics
- replacement diagnostics
- diagnostic ordering and richness policy

**Relevant APIs**

- `WorthUiRuntimeDiagnostic`
- `WorthUiReloadDiagnostic`
- `WorthUiPlanDiagnostic`
- `WorthUiDiagnosticRichnessPolicy`
- `WorthUiRuntimeDiagnosticReport`
- `WorthUiDiagnosticRichnessTier`
- `WorthUiSupportReportPolicy`
- `WorthUiDiagnosticProjectionHook`
- WORTH Query `Inspection`
- WORTH Query `Cross-Runtime Causal Inspection`
- WORTH Query `Projection Consumption`
- WORTH Query `Recovery`
- WORTH Query `Ordinary Outcomes`
- WORTH Query lower-runtime boundary envelopes where diagnostics explain
  admitted lower-runtime contact

**Warnings**

- Do not flatten reload failures into strings.
- Do not let richer diagnostics change active artifact, active plan, or digest.
- Do not put diagnostics construction on the steady frame path unless policy
  explicitly admits it.
- Do not let custom diagnostics projection create new runtime truth or
  substitute for typed phase diagnostics.

**Test requirements**

- `same_reload_failure_produces_same_diagnostic_codes_and_ordering`: equivalent
  failures produce deterministic typed diagnostics.
- `diagnostic_richness_does_not_change_active_plan_or_digest`: rich versus
  minimal diagnostics leave active runtime truth unchanged.
- `every_replacement_phase_denial_maps_to_specific_diagnostic_family`:
  admission, identity, Query, lane, plan, activation, and swap failures produce
  distinct diagnostic families.
- `diagnostics_never_depend_on_error_message_substrings`: diagnostic assertions
  use typed codes, identities, receipts, and stop classes rather than
  presentation wording.
- `diagnostic_richness_tiers_gate_report_materialization`: off, minimal,
  standard, full, and support tiers deterministically decide which diagnostic
  and Foundational report sections may materialize.

**Engineering decisions**

- Diagnostics are derived observation artifacts over runtime phases.
- Diagnostic reports carry phase-local typed references and can link to Query
  inspection where applicable.
- Diagnostic richness is policy-selected and separate from active plan
  semantics.
- Diagnostics tiers are `off`, `minimal`, `standard`, `full`, and `support`;
  only `full` and `support` may request rich support expansion by default.

**Open questions**

- None.

### Phase 29: In-App Diagnostics Projection Boundary

Freeze the runtime-owned projection that lets a running Worth UI app present
reload, plan, lane, state, and frame-cost diagnostics without inventing a second
diagnostics model in app code.

**Relevant subsystems**

- in-app diagnostics projection
- diagnostics surface binding
- reload status presentation
- frame-cost report presentation

**Relevant APIs**

- `WorthUiDiagnosticsProjection`
- `WorthUiReloadStatusSurface`
- `WorthUiPlanInspectionSurface`
- `WorthUiFrameCostSurface`
- `WorthUiDiagnosticsProjectionHook`
- WORTH Foundational `performance_api::lower_lane::reports`
- WORTH Foundational `plan_performance_report`
- WORTH Foundational `FoundationalPerformanceReportRequest`
- WORTH Foundational `FoundationalMaterializedPerformanceReport`
- WORTH Query `Inspection`
- WORTH Query `Cross-Runtime Causal Inspection`
- WORTH Query `Projection Consumption`
- WORTH Query `Recovery`
- WORTH Query `Async Resources And Result State`
- WORTH Query ordinary outcome and checked-stop presentation artifacts

**Warnings**

- Do not make the diagnostics UI authoritative.
- Do not require apps to scrape logs or inspect private runtime state.
- Do not let diagnostics projection become a hidden imperative editing runtime.
- Do not let projection hooks invent Query status rows, performance rows, or
  reload facts not present in typed runtime diagnostics.

**Test requirements**

- `diagnostics_projection_preserves_runtime_diagnostic_identity`: projected
  diagnostics preserve typed diagnostic identity and phase references.
- `diagnostics_projection_cannot_mutate_active_plan`: diagnostics surfaces
  cannot alter active artifact, plan, or durable state.
- `failed_reload_visible_without_blank_active_app`: invalid reload diagnostics
  are presentable while the prior active UI remains active.
- `diagnostics_projection_rejects_freeform_query_status_rows`: diagnostics
  projection cannot display Query-bound status unless it is backed by Query
  ordinary outcomes, inspection, recovery, async/result, or checked-stop
  posture.

**Engineering decisions**

- In-app diagnostics projection is included now because failed reloads are part
  of the hot iteration product loop.
- The projection consumes runtime diagnostic reports and plan/frame receipts.
- Later tooling can build richer inspectors over this same projection contract.
- Projection hooks choose presentation shape only; they cannot change
  diagnostic identity, ordering, severity, Query posture, or performance
  materialization boundaries.

**Open questions**

- None.

### Phase 30: File Watch And Debounce Ingress Boundary

Freeze file watching, debouncing, source package reload triggering, and
replaceable artifact-input watcher behavior as ingress into the already-defined
candidate pipeline.

**Relevant subsystems**

- file watcher
- debounce pipeline
- source package reload trigger
- artifact-input watcher

**Relevant APIs**

- `WorthUiSourceWatcher`
- `WorthUiReloadDebounce`
- `WorthUiWatchedArtifactInput`
- `WorthUiWatcherEvent`
- `WorthUiSourceProvider`
- `WorthUiSourceIngressHook`
- `WorthUiSourcePackageRevision`
- `WorthUiCandidateOrderingReceipt`

**Warnings**

- Do not let the watcher own reload semantics.
- Do not treat filesystem events as authoritative dependency impact.
- Do not run source parsing, validation, or plan swap on the frame path.
- Do not assume the filesystem is the only source ingress; editor buffers,
  generated source, test harnesses, and Rust-authored artifact inputs must enter
  through the same candidate envelope.
- Do not trust raw event order from temp files, partial writes, atomic renames,
  or multi-root saves without a candidate ordering receipt.

**Test requirements**

- `equivalent_file_event_bursts_debounce_to_equivalent_candidates`: equivalent
  event bursts produce equivalent candidate submissions after debounce.
- `watcher_event_without_lowered_candidate_cannot_mutate_active_runtime`: raw
  file events cannot alter active runtime state.
- `file_watcher_uses_candidate_pipeline_for_file_and_rust_artifact_inputs`:
  watched file-authored and replaceable Rust-authored artifact inputs enter the
  same replacement candidate pipeline.
- `watcher_event_reorder_does_not_change_final_candidate_sequence`: reordered
  filesystem bursts with the same final source package produce deterministic
  candidate ordering and debounce evidence.
- `partial_write_and_atomic_rename_emit_one_ordered_candidate`: temp-file,
  partial-write, and atomic-rename patterns debounce into one source package
  revision with deterministic candidate cause evidence.
- `in_memory_source_provider_uses_same_candidate_admission`: editor-buffer and
  test-harness source ingress cannot bypass candidate admission, lowering, or
  activation staging.

**Engineering decisions**

- File watching is deliberately late in the milestone because candidate
  admission and activation are the real runtime authority.
- Watchers create candidate causes and trigger source/lowering work; they do
  not decide replacement impact.
- Debounce policy must be observable enough for iteration latency diagnostics.
- Source ingress hooks are supported for filesystem, editor buffer, generated,
  in-memory test, and Rust-authored artifact sources, but every path produces
  the same candidate ordering and admission evidence.

**Open questions**

- None.

### Phase 31: Counter Taxonomy And Measurement Boundary

Freeze the named counter taxonomy and measurement boundaries for reload,
lowering, reconciliation, plan assembly, lane execution, and frame rendering.

**Relevant subsystems**

- counter taxonomy
- measurement boundary registry
- complexity contract labels
- frame-cost evidence

**Relevant APIs**

- `WorthUiRuntimeCounterFamily`
- `WorthUiMeasurementBoundary`
- `WorthUiFrameCostCounter`
- `WorthUiComplexityContract`
- WORTH Foundational `performance_api::common_path`
- WORTH Foundational `performance_api::lower_lane::basis`
- WORTH Foundational `FoundationalPerformanceBoundary`
- WORTH Foundational `FoundationalPerformanceEvidenceStrength`
- WORTH Foundational layout, breadth/locality, allocation, access-pattern,
  execution-temperature, freshness/retention, fallback/debt, and work-class
  definitions
- WORTH Query `Subscription Selection And Diagnostics`
- WORTH Query `Signal Compatibility And Continuation`
- WORTH Query `Planner Parallel Admission And Scale Posture`
- WORTH Query `Async Resources And Result State`

**Warnings**

- Do not ship performance claims without named counters.
- Do not use elapsed time alone as proof of frame-path cost.
- Do not let counter collection alter active UI meaning.
- Do not treat WORTH Foundational vocabulary as runtime instrumentation; Worth
  UI counters are produced by Worth UI and only then lowered into Foundational
  performance claims, bundles, and receipts.

**Test requirements**

- `counter_taxonomy_replay_is_deterministic`: equivalent runtime operations
  expose equivalent counter family names and measurement boundaries.
- `hot_path_without_counter_boundary_rejected_by_certification`: declared hot
  paths cannot enter acceptance without a named counter boundary.
- `counter_richness_does_not_change_active_plan_digest`: richer counter capture
  policy does not change active artifact or plan digests.
- `counter_taxonomy_rejects_unattributed_work_bucket`: every nonzero counter
  must belong to a named phase, lane, boundary, or Query-owned posture source.
- `foundational_performance_claim_without_worth_ui_counter_denied`: a
  Foundational performance claim or layout intent cannot certify Worth UI work
  unless Worth UI emitted matching phase/lane counter evidence.

**Engineering decisions**

- Counters are product-visible proof artifacts, not hidden profiler data.
- Measurement boundaries exist at reload candidate admission, impact narrowing,
  state reconciliation, Query rebind planning, plan lowering, lane execution,
  activation, and steady frame rendering.
- Counter names must preserve phase, lane, and authority boundaries.
- Foundational performance posture names are used for shared boundary meaning;
  they do not decide Worth UI lane execution strategy.

**Open questions**

- None.

### Phase 32: Reload And Lowering Counter Boundary

Freeze counters for candidate admission, equivalence comparison, impact
narrowing, identity matching, state reconciliation, Query rebind planning,
handle allocation, topology assembly, and plan equivalence.

**Relevant subsystems**

- reload counters
- lowering counters
- reconciliation counters
- plan assembly counters

**Relevant APIs**

- `WorthUiReloadCounters`
- `WorthUiImpactNarrowingCounters`
- `WorthUiReconciliationCounters`
- `WorthUiPlanLoweringCounters`
- WORTH Foundational `performance_api::lower_lane::basis`
- WORTH Foundational `performance_api::lower_lane::receipts`
- WORTH Foundational `performance_bundle`
- WORTH Foundational `counter_backed_performance_receipt`
- WORTH Foundational `FoundationalPerformanceCounterSpec`
- WORTH Foundational `FoundationalPerformanceCounterRow`
- WORTH Query support/admission, basis lifecycle, subscription diagnostics,
  signal-compatibility, and continuation receipts referenced by reload and
  lowering counters where Query-bound surfaces participate

**Warnings**

- Do not hide broad scans inside reload or plan lowering.
- Do not rely on small sample apps to justify missing counters.
- Do not let invalid reloads skip counter evidence.
- Do not use a policy-admission receipt as executed performance evidence.

**Test requirements**

- `equivalent_reload_work_produces_equivalent_reload_counters`: equivalent
  candidate replacement work produces equivalent reload and lowering counters.
- `impact_narrowing_counter_detects_full_artifact_scan_regression`: hostile
  changed-module lookup fails if counters show broad artifact traversal.
- `invalid_candidate_emits_admission_and_preservation_counters`: failed reloads
  still emit enough counter evidence to prove where work stopped.
- `reload_counter_detects_repeated_query_support_rediscovery`: reload/lowering
  counters fail if Query support/admission posture is rediscovered repeatedly
  instead of carried as typed receipts.
- `foundational_counter_receipt_rejects_missing_duplicate_or_unexpected_rows`:
  lowering Worth UI reload counters into a Foundational counter-backed receipt
  fails when required rows are missing, duplicated, unexpected, or attached to
  the wrong counter spec.

**Engineering decisions**

- Reload and lowering counters are separate from steady-frame counters because
  they protect different cost surfaces.
- Invalid paths emit counters at the boundary they reached.
- Counter receipts must be usable by hostile certification without parsing
  debug logs.
- Foundational counter-backed receipts wrap Worth UI counter evidence after the
  fact; they are not allowed to trigger extra reload, comparison, or lowering
  work.

**Open questions**

- None.

### Phase 33: Steady Frame Counter Boundary

Freeze steady-frame counters for nodes visited, layout recompute breadth,
hit-test breadth, text shaping, glyph uploads, allocations, draw batches,
render passes, virtualized rows/columns touched, canvas hit-test breadth, and
real-time overlay work.

**Relevant subsystems**

- frame counters
- lane execution receipts
- allocation and draw accounting
- render-pass accounting

**Relevant APIs**

- `WorthUiSteadyFrameCounters`
- `WorthUiLaneFrameReceipt`
- `WorthUiFrameExecutionReceipt`
- `WorthUiRenderCostReceipt`
- WORTH Foundational `performance_api::lower_lane::receipts`
- WORTH Foundational `performance_api::lower_lane::reports`
- WORTH Foundational `FoundationalCounterBackedPerformanceReceipt`
- WORTH Foundational `FoundationalPerformanceCounterRow`
- WORTH Foundational `FoundationalPerformanceReportMaterializationBoundary`
- WORTH Query subscription diagnostics, live-view delivery posture,
  async/result posture, and projection-consumption receipts referenced by
  Query-bound lane frame receipts

**Warnings**

- Do not let steady frames parse source, validate artifacts, resolve registry
  strings, or broad-scan artifact topology.
- Do not hide allocation or text shaping costs behind lane-local helpers.
- Do not merge virtualized, canvas, real-time, and ordinary lane counters into
  one ambiguous count.
- Do not materialize Foundational performance reports on the ordinary steady
  frame path.

**Test requirements**

- `steady_frame_counters_replay_for_equivalent_active_plan`: equivalent active
  plans and frame inputs produce equivalent steady-frame counter families.
- `steady_frame_source_parse_or_registry_lookup_counter_must_be_zero`:
  certification fails if steady frames parse source or resolve capability
  strings.
- `lane_specific_frame_counters_expose_expected_work_breadth`: ordinary,
  virtualized data, canvas/spatial, and real-time lanes expose distinct work
  breadth counters.
- `steady_frame_counters_fail_on_diagnostic_materialization_by_default`:
  steady-frame certification catches implementations that materialize rich
  diagnostics when diagnostic policy is minimal/off.
- `foundational_receipt_counter_rows_match_steady_frame_specs_exactly`:
  steady-frame receipt lowering fails when a Foundational counter row omits,
  renames, duplicates, or miscounts a Worth UI frame counter spec.

**Engineering decisions**

- Steady-frame counters are required active-plan output in test and diagnostic
  modes.
- Allocation, shaping, draw, and render-pass counters must be explicit enough
  to catch broad regressions.
- Counter collection policy may vary, but the counter boundary must exist.
- Foundational report planning is a diagnostics/support boundary; ordinary
  frame execution emits evidence without expanding reports by default.

**Open questions**

- None.

### Phase 34: File And Rust Replacement Parity Boundary

Freeze the proof that a running app can accept valid replacement artifacts from
file-authored source or Rust-authored composition through the same candidate,
admission, reconciliation, plan, lane, counter, and activation pipeline.

**Relevant subsystems**

- authoring-lane parity
- Rust composition replacement
- file-authored replacement
- active runtime replacement parity

**Relevant APIs**

- `WorthUiCandidateAuthoringLane`
- `WorthUiReplacementCandidate`
- `WorthUiRuntimeArtifactComparison`
- `WorthUiPlanSwapReceipt`

**Warnings**

- Do not give Rust-authored composition privileged access to active plan or
  canonical artifact constructors.
- Do not certify parity by visual similarity or demo behavior alone.
- Do not treat authoring lane as a replacement semantics fork.

**Test requirements**

- `file_and_rust_replacements_with_same_meaning_activate_equivalent_plans`:
  equivalent file-authored and Rust-authored replacements produce equivalent
  active execution plans and swap receipts.
- `rust_replacement_cannot_bypass_candidate_admission_or_snapshot_support`:
  Rust-authored composition cannot bypass admission, snapshot support, or
  replacement diagnostics.
- `authoring_lane_difference_preserved_only_as_diagnostic_provenance`: authoring
  lane remains observable without changing semantic replacement or plan
  equivalence.
- `rust_authored_candidate_cannot_inject_active_plan_nodes_directly`: Rust
  composition parity fails if it bypasses artifact lowering and provides plan
  nodes or handles directly.

**Engineering decisions**

- Rust composition remains an authoring escape hatch with no special runtime
  privilege.
- Parity is proven at artifact, plan, lane, and activation receipt boundaries.
- Authoring lane provenance is diagnostic, not authority.

**Open questions**

- None.

### Phase 35: Hostile Reload Storm Certification Boundary

Close the reload product loop with hostile certification over repeated valid,
invalid, equivalent, identity-changing, lane-changing, and Query-drifted
candidate storms.

**Relevant subsystems**

- reload certification
- hostile candidate generation
- preservation proof
- iteration latency counters

**Relevant APIs**

- `WorthUiReloadStormCertification`
- `WorthUiReloadStormScenario`
- `WorthUiReloadCertificationBundle`
- `WorthUiReloadLatencyCounters`
- WORTH Foundational `performance_api::lower_lane::basis`
- WORTH Foundational `performance_api::lower_lane::receipts`
- WORTH Foundational `performance_api::lower_lane::reports`
- WORTH Foundational `performance_api::stronger_lane::certified`
- WORTH Foundational `certify_hot_path_counter_backed_performance_receipt`
- WORTH Foundational `certify_support_expansion_performance_report`
- WORTH Query support/admission, live-view, subscription diagnostics,
  async/result, recovery, and inspection receipts included in reload storm
  certification bundles where Query-bound surfaces participate

**Warnings**

- Do not close the milestone with a single happy-path reload.
- Do not certify only source watcher behavior.
- Do not allow repeated invalid reloads to degrade active state or diagnostics
  determinism.
- Do not certify a reload storm from elapsed time or policy admission alone;
  certification consumes Worth UI counters lowered into Foundational receipts.

**Test requirements**

- `hostile_reload_storm_preserves_last_valid_active_plan`: mixed candidate
  storms preserve the last valid plan across invalid candidates.
- `reload_storm_equivalent_edits_do_not_rebuild_or_swap_needlessly`: equivalent
  candidates classify as no-op and avoid unnecessary activation work.
- `reload_storm_latency_counters_remain_iteration_shaped`: reload work exposes
  counters proving iteration is not Rust-build-shaped or full-runtime-shaped by
  default.
- `reload_storm_with_interleaved_invalid_and_valid_candidates_preserves_ordered_truth`:
  valid candidates after invalid ones activate in deterministic order without
  replaying stale invalid state.
- `reload_storm_rejects_WORTHd_receipt_reuse_across_candidates`: receipts from
  an earlier candidate cannot certify a later candidate with a different digest,
  basis, impact, or Query posture.
- `reload_storm_foundational_bundle_comparison_uses_full_meaning`: canonical
  Foundational bundle comparison fails when two reload runs share elapsed-time
  shape but differ in counter specs, evidence rows, layout posture, boundary,
  or freshness/debt posture.

**Engineering decisions**

- Reload storm certification is the first end-to-end proof of the candidate to
  active-plan pipeline.
- Certification bundles must be inspectable offline.
- The storm must include both file-authored and Rust-authored candidate paths.
- Foundational certified bundles are emitted only after Worth UI has produced
  candidate, activation, and counter receipts for the storm.

**Open questions**

- None.

### Phase 36: Identity, State, And Query Drift Certification Boundary

Close hostile certification for identity ambiguity, state carry-forward,
state replacement/drop, Query binding drift, live rebind, and UI-local
pseudo-runtime rejection.

**Relevant subsystems**

- identity hostile certification
- durable state hostile certification
- Query binding drift certification
- pseudo-runtime guard tests

**Relevant APIs**

- `WorthUiIdentityStateCertification`
- `WorthUiQueryDriftCertification`
- `WorthUiStateCarryForwardReceipt`
- `WorthUiQueryBindingDriftDenial`
- WORTH Query `Support Matrix And Admission`
- WORTH Query `Basis Capability Lifecycle`
- WORTH Query `Live Views And Live Promotion`
- WORTH Query `Subscription Selection And Diagnostics`
- WORTH Query `Projection Consumption`
- WORTH Query `Async Resources And Result State`
- WORTH Query `Recovery`
- WORTH Query `Inspection`
- WORTH Query `Cross-Runtime Causal Inspection`

**Warnings**

- Do not treat state preservation as success unless the receipt proves why it
  was eligible.
- Do not let ambiguous identity preserve state.
- Do not accept Query drift recovery through local UI state or lower-runtime
  bypasses.

**Test requirements**

- `ambiguous_identity_storm_never_preserves_durable_state`: repeated ambiguous
  identity changes always deny carry-forward.
- `state_replacement_and_drop_receipts_match_actual_runtime_state`: receipts
  for preserved, replaced, dropped, and created state exactly match active
  runtime state after activation.
- `query_drift_certification_rejects_ui_local_loading_or_subscription_model`:
  Query drift cannot be patched with UI-local status or subscription wrappers.
- `query_drift_certification_uses_query_stop_classes_not_messages`: Query drift
  denials assert typed Query stop classes/outcomes rather than message text.
- `state_and_query_residue_scan_clean_after_failed_and_successful_reload_mix`:
  mixed failure/success storms leave no orphan durable state, stale live
  binding, or UI-local Query status residue.

**Engineering decisions**

- This certification proves the most dangerous stateful reload failure modes.
- State and Query certification consume receipts produced by earlier phases.
- Query-owned artifacts and posture are the only admissible binding authority.

**Open questions**

- None.

### Phase 37: Lane And Frame-Cost Certification Boundary

Close hostile certification for ordinary, virtualized data, canvas/spatial, and
real-time overlay lanes under frame-cost pressure, including proof that steady
frames do not parse, validate, resolve strings, broad-scan artifact topology, or
pay diagnostic richness by default.

**Relevant subsystems**

- lane certification
- frame-cost certification
- no-source frame proof
- broad-scan regression proof

**Relevant APIs**

- `WorthUiLaneCertification`
- `WorthUiFrameCostCertification`
- `WorthUiNoSourceFrameProof`
- `WorthUiBroadScanRegressionDenial`
- WORTH Foundational `performance_api::lower_lane::basis`
- WORTH Foundational `performance_api::lower_lane::receipts`
- WORTH Foundational `performance_api::lower_lane::reports`
- WORTH Foundational `performance_api::stronger_lane::certified`
- WORTH Foundational `performance_api::stronger_lane::readiness`
- WORTH Foundational `compare_performance_bundles`
- WORTH Foundational `prepare_counter_backed_performance_receipt_for_canonical_basis`
- WORTH Foundational `foundational_performance_milestone8_readiness_report`
- WORTH Foundational `require_foundational_performance_milestone8_production_test_readiness`
- WORTH Query collection/read, view-shape, live-view, subscription diagnostics,
  projection-consumption, async/result, signal-compatibility, and continuation
  receipts referenced by lane certification when data or runtime posture comes
  from Query-bound surfaces

**Warnings**

- Do not certify lanes through screenshots or visual plausibility.
- Do not accept frame elapsed time without explanatory counters.
- Do not leave real-time or data-heavy lanes as debt; they are platform
  foundation surfaces in this milestone.
- Do not use Foundational readiness as a shortcut around Worth UI hostile lane
  and frame-cost certification.

**Test requirements**

- `data_heavy_lane_touches_visible_range_not_full_collection`: hostile data
  surfaces prove virtualized breadth through exact counters.
- `realtime_lane_avoids_ordinary_widget_traversal`: hostile real-time overlays
  prove they do not execute through ordinary widget mechanics.
- `steady_frame_no_source_no_registry_no_broad_scan_certification`: every lane
  proves steady frames do not parse source, validate artifacts, resolve
  registry strings, or broad-scan canonical artifacts.
- `cross_lane_parity_certifies_mechanics_not_meaning_changes`: lane
  specialization proof shows mechanics and counters change without changing
  canonical UI meaning.
- `data_and_realtime_certification_use_scale_variation_not_single_fixture`:
  hostile data-heavy and real-time tests must run at multiple widths/densities
  so accidental constant-size shortcuts do not pass.
- `frame_cost_certification_fails_on_any_positive_source_or_registry_counter`:
  any nonzero source parse, artifact validation, registry string lookup, or
  broad artifact scan counter on steady frames fails certification.
- `foundational_readiness_cannot_pass_with_uncertified_worth_ui_lane_evidence`:
  production-readiness proof fails if any lane lacks Worth UI counter evidence
  lowered into canonical Foundational bundles and counter-backed receipts.

**Engineering decisions**

- Lane/frame certification is the acceptance gate for the merged hot-runtime
  and execution-plan foundation.
- Data-heavy and real-time certification are mandatory because Worth UI is a UI
  platform for those product classes.
- The final proof is counter-backed, not visual or narrative.
- Foundational readiness is a final closure inventory over Worth UI lane
  evidence, not the source of that evidence.

**Open questions**

- None.

## Must Ship

- runtime host authority over active artifact, active execution plan, last valid
  state, reload status, and diagnostics references
- replaceable candidate envelopes for file-authored and Rust-authored artifact
  inputs
- runtime lifecycle, frame epoch, pending activation, pause/resume, shutdown,
  and failed-activation recovery receipts
- candidate admission, artifact equivalence, impact classification, and
  dependency impact narrowing
- impact surfaces for commands, tokens/themes, accessibility metadata, Query
  bindings, durable state families, lane assignment, renderer resources, and
  diagnostics policy
- identity match graph and per-node replacement classification
- durable UI state inventory and reconciliation for focus, scroll, selection,
  panel visibility, splitters, tabs, and text input state
- admitted state-family, identity-seed, component-lowering, lane-adapter,
  diagnostics-projection, counter-family, and report-materialization hooks
- Query binding comparison and live rebind planning that preserve Query-owned
  posture
- activation staging, safe frame-boundary activation, atomic plan swap, prior
  valid preservation, and failure preservation
- execution-plan lowering input, compact runtime handles, plan topology, plan
  equivalence/digest, and plan inspection/provenance
- plan-node families for component invocation, child ranges, commands, tokens,
  layout regions, Query/view bindings, accessibility, diagnostics, lane
  partitions, egui boundary refs, and render-resource refs
- execution lane taxonomy and real ordinary, virtualized data, canvas/spatial,
  and real-time overlay/HUD lanes
- cross-lane meaning parity proof
- typed reload, plan, lane, reconciliation, Query, activation, and frame-cost
  diagnostics
- in-app diagnostics projection for reload and frame-cost issues
- file-watch and debounce ingress into the candidate pipeline
- reload/lowering counters and steady-frame lane counters
- WORTH Foundational performance claim, bundle, counter-backed receipt, report
  planning, certified bundle, and readiness integration at the explicit
  performance boundaries
- hostile reload, identity/state, Query drift, lane, and frame-cost
  certification suites

## Must Preserve

- Worth UI remains a UI platform over egui, not a fork of egui or a renderer
  runtime.
- The canonical artifact remains semantic UI authority; the execution plan is
  frame-executable active runtime truth derived from it.
- Extension hooks remain admitted contributions, never alternate authority
  paths.
- File-authored and Rust-authored composition converge on the same active
  replacement path.
- Active plan replacement never reopens mutable registry authority or source
  parsing on the steady frame path.
- Invalid reloads preserve the prior active plan and durable state.
- Identity carry-forward stays proof-bearing and cannot be guessed from
  display, layout, source order, or widget type alone.
- Durable UI state remains separate from authoritative runtime truth.
- Query-facing surfaces preserve Query-owned support, admission, live,
  async/result, projection, recovery, inspection, and explanation posture.
- Execution lanes specialize cost and mechanics without creating shadow UI
  runtimes or redefining command/Query/accessibility meaning.
- Diagnostics and counters observe; they do not change artifact or plan truth.
- WORTH Foundational performance surfaces preserve shared boundary meaning and
  certification shape; they do not own Worth UI runtime execution, live counter
  storage, plan topology, or lane mechanics.
- Unsupported hook, lane, state, Query, renderer, source, capability snapshot,
  and plan-node families fail closed before activation.

## Acceptance Evidence

- a running app can accept valid file-authored and Rust-authored replacement
  artifacts through one candidate-to-active-plan pipeline
- equivalent replacements classify as no-op and avoid unnecessary swaps
- valid replacements preserve eligible durable state and explicitly replace or
  drop ineligible state
- invalid candidates, failed reconciliation, failed plan lowering, and failed
  swaps preserve the prior active runtime plan
- Query binding drift produces typed rebind or denial instead of UI-local
  pseudo-runtime repair
- active execution plans expose compact handles, topology, lane partitions,
  equivalence, provenance, and inspection
- admitted hooks can branch special mechanics while preserving candidate,
  identity, state, lane, diagnostic, counter, and certification receipts
- ordinary, virtualized data, canvas/spatial, and real-time overlay lanes all
  execute through their own admitted lane mechanics
- frame counters prove steady frames do not parse source, validate artifacts,
  resolve registry strings, broad-scan artifact topology, or pay diagnostics
  cost by default
- Worth UI counter evidence lowers into WORTH Foundational canonical bundles,
  counter-backed receipts, planned reports, certified bundles, and readiness
  closure without moving report materialization into the steady frame path
- hostile reload storm, identity/state, Query drift, data-heavy, real-time, and
  cross-lane parity certification suites pass

## Sequencing Notes

This milestone belongs immediately after Milestone 2 because source lowering
has produced canonical artifact truth, but the running platform still needs an
active runtime substrate that can execute, replace, reconcile, and certify that
truth.

It deliberately folds the original roadmap's separate execution-plan and
frame-cost milestone into Milestone 3 because hot reload cannot be honestly
proven unless replacement reaches the frame-executable plan boundary, and
execution plans cannot be honestly designed after reload has already invented
identity, state, and swap semantics.

It belongs before application shell, command routing, Query-bound views, forms,
professional components, canvas workflows, plugins, tooling, and native
integration because all of those later product milestones must consume one
platform-owned active-plan runtime instead of inventing local reload, state,
lane, or frame-cost behavior.

## Self-Check

- Does the milestone solve a real structural problem or just package work
  cosmetically? Yes: it closes the active runtime artifact-to-plan foundation.
- Is the adversarial constraint precise and load-bearing? Yes: hostile reload,
  identity/state, Query drift, lane, and frame-cost pressure all drive the
  phase plan.
- Does the roadmap justify this milestone now? Yes: foundation-first work must
  close hot iteration and frame-efficient execution before shell/component
  breadth.
- Does the spec preserve crate authority boundaries? Yes: Worth UI owns UI
  activation and execution plans while Query, truth, signal, bridge, renderer,
  and native authority remain separate.
- Are the phases carrying most of the real design information? Yes.
- Is each phase centered on one conceptual detail or boundary? Yes.
- Does each phase contain at least 2 adversarial tests by default? Yes.
- Could a competent engineer map this spec into honest types, modules, and
  tests? Yes.
- Does the milestone belong in this roadmap sequence, or is it out of order?
  It belongs here, directly after canonical artifacts and before shell/product
  breadth.
