use super::*;

pub(in crate::harness::milestone_eight_certification) fn canonical_rows(
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Vec<MilestoneEightCertificationRow> {
    let direct = direct_detail_bundle();
    let template_lane = template_detail_bundle();
    let scope_lane = scope_detail_bundle();
    let grouped_control_rows = &[
        grouped_row("task-1", "Ada", "todo"),
        grouped_row("task-2", "Bea", "doing"),
    ];
    let grouped_hostile_rows = &[
        grouped_row("task-1", "Ada", "doing"),
        grouped_row("task-3", "Cy", "todo"),
    ];

    vec![
        MilestoneEightCertificationRow {
            row_name: "direct-vs-scope-parity",
            perturbation_class: MilestoneEightPerturbationClass::DirectScopeParity,
            hostile_expectation: HostileExpectation::EquivalentToControl,
            parity_anchor: ParityAnchor::Control,
            control_lane: direct,
            hostile_lane: scope_lane.clone(),
            parity_lane: scope_lane,
        },
        MilestoneEightCertificationRow {
            row_name: "direct-vs-template-parity",
            perturbation_class: MilestoneEightPerturbationClass::DirectTemplateParity,
            hostile_expectation: HostileExpectation::EquivalentToControl,
            parity_anchor: ParityAnchor::Control,
            control_lane: direct_detail_bundle(),
            hostile_lane: template_lane.clone(),
            parity_lane: template_detail_bundle(),
        },
        MilestoneEightCertificationRow {
            row_name: "scope-template-direct-parity",
            perturbation_class: MilestoneEightPerturbationClass::ScopeTemplateDirectParity,
            hostile_expectation: HostileExpectation::EquivalentToControl,
            parity_anchor: ParityAnchor::Control,
            control_lane: direct_detail_bundle(),
            hostile_lane: template_lane,
            parity_lane: scope_detail_bundle(),
        },
        MilestoneEightCertificationRow {
            row_name: "saved-query-freeze-parity",
            perturbation_class: MilestoneEightPerturbationClass::SavedQueryFreezeParity,
            hostile_expectation: HostileExpectation::DistinctFromControl,
            parity_anchor: ParityAnchor::Control,
            control_lane: saved_query_bundle(false),
            hostile_lane: saved_query_bundle(true),
            parity_lane: saved_query_bundle(false),
        },
        MilestoneEightCertificationRow {
            row_name: "view-shape-non-cosmetic-planning-live",
            perturbation_class: MilestoneEightPerturbationClass::ViewShapePlanningLiveSemantics,
            hostile_expectation: HostileExpectation::DistinctFromControl,
            parity_anchor: ParityAnchor::Hostile,
            control_lane: table_live_bundle(&direct_collection_canonical()),
            hostile_lane: grouped_live_bundle(true, resource_request),
            parity_lane: grouped_live_bundle(false, resource_request),
        },
        MilestoneEightCertificationRow {
            row_name: "kanban-desired-state-to-delta-parity",
            perturbation_class: MilestoneEightPerturbationClass::KanbanDesiredStateDeltaParity,
            hostile_expectation: HostileExpectation::DistinctFromControl,
            parity_anchor: ParityAnchor::Control,
            control_lane: grouped_live_bundle(true, resource_request),
            hostile_lane: grouped_live_bundle(false, resource_request),
            parity_lane: grouped_live_bundle(true, resource_request),
        },
        MilestoneEightCertificationRow {
            row_name: "kanban-delta-admission-boundary",
            perturbation_class: MilestoneEightPerturbationClass::KanbanDeltaAdmissionBoundary,
            hostile_expectation: HostileExpectation::DistinctFromControl,
            parity_anchor: ParityAnchor::Hostile,
            control_lane: grouped_live_bundle(true, resource_request),
            hostile_lane: grouped_live_bundle(false, resource_request),
            parity_lane: grouped_live_bundle(false, resource_request),
        },
        MilestoneEightCertificationRow {
            row_name: "grouped-delta-honesty",
            perturbation_class: MilestoneEightPerturbationClass::GroupedDeltaHonesty,
            hostile_expectation: HostileExpectation::DistinctFromControl,
            parity_anchor: ParityAnchor::Hostile,
            control_lane: grouped_live_bundle(true, resource_request),
            hostile_lane: grouped_live_bundle(false, resource_request),
            parity_lane: grouped_live_bundle(false, resource_request),
        },
        MilestoneEightCertificationRow {
            row_name: "grouped-bridge-truth-view-authority",
            perturbation_class: MilestoneEightPerturbationClass::GroupedBridgeTruthViewAuthority,
            hostile_expectation: HostileExpectation::DistinctFromControl,
            parity_anchor: ParityAnchor::Control,
            control_lane: grouped_truth_view_bundle(grouped_control_rows, resource_request),
            hostile_lane: grouped_truth_view_bundle(grouped_hostile_rows, resource_request),
            parity_lane: grouped_truth_view_bundle(grouped_control_rows, resource_request),
        },
        MilestoneEightCertificationRow {
            row_name: "grouped-query-execution-surface-authority",
            perturbation_class: MilestoneEightPerturbationClass::GroupedExecutionSurfaceAuthority,
            hostile_expectation: HostileExpectation::DistinctFromControl,
            parity_anchor: ParityAnchor::Control,
            control_lane: grouped_execution_surface_bundle(grouped_control_rows, resource_request),
            hostile_lane: grouped_execution_surface_bundle(grouped_hostile_rows, resource_request),
            parity_lane: grouped_execution_surface_bundle(grouped_control_rows, resource_request),
        },
        MilestoneEightCertificationRow {
            row_name: "grouped-proof-chain-no-payload-rediscovery",
            perturbation_class:
                MilestoneEightPerturbationClass::GroupedProofChainNoPayloadRediscovery,
            hostile_expectation: HostileExpectation::DistinctFromControl,
            parity_anchor: ParityAnchor::Control,
            control_lane: grouped_payload_rediscovery_free_bundle(
                grouped_control_rows,
                resource_request,
            ),
            hostile_lane: grouped_payload_rediscovery_free_bundle(
                grouped_hostile_rows,
                resource_request,
            ),
            parity_lane: grouped_payload_rediscovery_free_bundle(
                grouped_control_rows,
                resource_request,
            ),
        },
        MilestoneEightCertificationRow {
            row_name: "inspector-observed-focused-distinction",
            perturbation_class: MilestoneEightPerturbationClass::InspectorSemanticDistinction,
            hostile_expectation: HostileExpectation::DistinctFromControl,
            parity_anchor: ParityAnchor::Hostile,
            control_lane: inspector_bundle(ViewShapeDescriptor::inspector_detail_observed()),
            hostile_lane: inspector_bundle(ViewShapeDescriptor::inspector_detail_focused(
                worth_foundational::facade::AspectKey::new("profile").unwrap(),
            )),
            parity_lane: inspector_bundle(ViewShapeDescriptor::inspector_detail_focused(
                worth_foundational::facade::AspectKey::new("profile").unwrap(),
            )),
        },
        MilestoneEightCertificationRow {
            row_name: "identity-aware-focused-inspector-parity",
            perturbation_class: MilestoneEightPerturbationClass::IdentityAwareInspectorParity,
            hostile_expectation: HostileExpectation::DistinctFromControl,
            parity_anchor: ParityAnchor::Control,
            control_lane: inspector_bundle(
                ViewShapeDescriptor::identity_aware_inspector_detail_focused(
                    worth_foundational::facade::AspectKey::new("profile").unwrap(),
                    InspectorIdentityClassification::AuthoritativeContinuity,
                ),
            ),
            hostile_lane: inspector_bundle(
                ViewShapeDescriptor::identity_aware_inspector_detail_focused(
                    worth_foundational::facade::AspectKey::new("profile").unwrap(),
                    InspectorIdentityClassification::AdvisoryCandidates,
                ),
            ),
            parity_lane: inspector_bundle(
                ViewShapeDescriptor::identity_aware_inspector_detail_focused(
                    worth_foundational::facade::AspectKey::new("profile").unwrap(),
                    InspectorIdentityClassification::AuthoritativeContinuity,
                ),
            ),
        },
        MilestoneEightCertificationRow {
            row_name: "identity-break-inspector-explicitness",
            perturbation_class: MilestoneEightPerturbationClass::IdentityBreakInspectorExplicitness,
            hostile_expectation: HostileExpectation::DistinctFromControl,
            parity_anchor: ParityAnchor::Hostile,
            control_lane: inspector_bundle(
                ViewShapeDescriptor::identity_aware_inspector_detail_focused(
                    worth_foundational::facade::AspectKey::new("profile").unwrap(),
                    InspectorIdentityClassification::AuthoritativeContinuity,
                ),
            ),
            hostile_lane: inspector_bundle(
                ViewShapeDescriptor::identity_aware_inspector_detail_focused(
                    worth_foundational::facade::AspectKey::new("profile").unwrap(),
                    InspectorIdentityClassification::IdentityBreak,
                ),
            ),
            parity_lane: inspector_bundle(
                ViewShapeDescriptor::identity_aware_inspector_detail_focused(
                    worth_foundational::facade::AspectKey::new("profile").unwrap(),
                    InspectorIdentityClassification::IdentityBreak,
                ),
            ),
        },
        MilestoneEightCertificationRow {
            row_name: "support-profile-honesty",
            perturbation_class: MilestoneEightPerturbationClass::SupportProfileHonesty,
            hostile_expectation: HostileExpectation::DistinctFromControl,
            parity_anchor: ParityAnchor::Control,
            control_lane: support_profile_bundle(true),
            hostile_lane: support_profile_bundle(false),
            parity_lane: support_profile_bundle(true),
        },
    ]
}
