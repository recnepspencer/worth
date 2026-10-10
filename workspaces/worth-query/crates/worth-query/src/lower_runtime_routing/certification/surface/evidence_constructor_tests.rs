use super::evidence::*;
use super::fixtures::{
    representative_basis_subscription_readmission_row,
    representative_basis_truth_view_readmission_row,
    representative_causal_bridge_materialization_row, representative_compose_read_row,
    representative_effect_bridge_writeback_row, representative_effect_relational_merge_row,
    representative_effect_relational_mutation_row,
    representative_execute_read_family_in_basis_context_row,
    representative_execute_read_family_row, representative_frontier_evidence_row,
    representative_historical_bridge_lowering_row, representative_intent_runtime_execution_row,
    representative_live_view_schema_row, representative_live_view_source_row,
    representative_preview_basis_row, representative_projection_bridge_row,
    representative_projection_query_receipts_row, representative_projection_relational_row,
    representative_public_live_view_declaration_row,
    representative_runtime_basis_context_read_graph_row,
    representative_runtime_current_read_graph_row, representative_runtime_intent_authority_row,
    representative_runtime_live_installation_orchestration_row,
    representative_signal_invalidation_row, representative_subscription_activation_row,
    representative_subscription_continuity_row, representative_write_authority_row,
};
use crate::lower_runtime_routing::WorthQueryLowerRuntimeSeamKey;

#[test]
fn representative_surface_runtime_backed_seams_match_real_boundary_artifact_constructors() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let surface = worth_query_lower_runtime_representative_surface(resource_request);
    let compose_row = representative_compose_read_row();
    let read_family_row = representative_execute_read_family_row();
    let read_family_basis_row = representative_execute_read_family_in_basis_context_row();
    let runtime_current_row = representative_runtime_current_read_graph_row();
    let runtime_basis_row = representative_runtime_basis_context_read_graph_row();
    let live_row = representative_live_view_schema_row();
    let source_row = representative_live_view_source_row();
    let public_live_row = representative_public_live_view_declaration_row();
    let orchestration_row = representative_runtime_live_installation_orchestration_row();
    let activation_row = representative_subscription_activation_row();
    let continuity_row = representative_subscription_continuity_row(resource_request);
    let preview_row = representative_preview_basis_row();
    let truth_view_readmission_row =
        representative_basis_truth_view_readmission_row(resource_request);
    let subscription_readmission_row =
        representative_basis_subscription_readmission_row(resource_request);
    let historical_row = representative_historical_bridge_lowering_row(resource_request);
    let mutation_row = representative_effect_relational_mutation_row();
    let merge_row = representative_effect_relational_merge_row();
    let writeback_row = representative_effect_bridge_writeback_row();
    let write_row = representative_write_authority_row();
    let signal_row = representative_signal_invalidation_row();
    let runtime_intent_authority_row = representative_runtime_intent_authority_row();
    let intent_runtime_execution_row = representative_intent_runtime_execution_row();
    let query_receipt_row = representative_projection_query_receipts_row();
    let relational_row = representative_projection_relational_row();
    let bridge_row = representative_projection_bridge_row(resource_request);
    let causal_row = representative_causal_bridge_materialization_row();
    let frontier_row = representative_frontier_evidence_row();

    assert_eq!(
        surface
            .route_plan_for(WorthQueryLowerRuntimeSeamKey::ComposeRead)
            .unwrap()
            .route_digest(),
        compose_row.route_plan.as_ref().unwrap().route_digest()
    );
    assert_eq!(
        surface
            .route_plan_for(WorthQueryLowerRuntimeSeamKey::ExecuteReadFamily)
            .unwrap()
            .route_digest(),
        read_family_row.route_plan.as_ref().unwrap().route_digest()
    );
    assert_eq!(
        surface
            .route_plan_for(WorthQueryLowerRuntimeSeamKey::ExecuteReadFamilyInBasisContext)
            .unwrap()
            .route_digest(),
        read_family_basis_row
            .route_plan
            .as_ref()
            .unwrap()
            .route_digest()
    );
    assert_eq!(
        surface
            .route_plan_for(WorthQueryLowerRuntimeSeamKey::ExecuteRuntimeCurrentReadGraph)
            .unwrap()
            .route_digest(),
        runtime_current_row
            .route_plan
            .as_ref()
            .unwrap()
            .route_digest()
    );
    assert_eq!(
        surface
            .route_plan_for(WorthQueryLowerRuntimeSeamKey::ExecuteRuntimeBasisContextReadGraph)
            .unwrap()
            .route_digest(),
        runtime_basis_row
            .route_plan
            .as_ref()
            .unwrap()
            .route_digest()
    );
    assert_eq!(
        surface
            .request_for(WorthQueryLowerRuntimeSeamKey::LiveViewSchemaAdmission)
            .unwrap()
            .request_digest(),
        live_row.request.request_digest()
    );
    assert_eq!(
        surface
            .boundary_receipt_for(WorthQueryLowerRuntimeSeamKey::LiveViewSchemaAdmission)
            .unwrap()
            .boundary_execution_identity(),
        live_row.boundary_receipt.boundary_execution_identity()
    );
    assert_eq!(
        surface
            .route_plan_for(WorthQueryLowerRuntimeSeamKey::LiveViewSourceDeclaration)
            .unwrap()
            .route_digest(),
        source_row.route_plan.as_ref().unwrap().route_digest()
    );
    assert_eq!(
        surface
            .route_plan_for(WorthQueryLowerRuntimeSeamKey::PublicLiveViewDeclaration)
            .unwrap()
            .route_digest(),
        public_live_row.route_plan.as_ref().unwrap().route_digest()
    );
    assert_eq!(
        surface
            .boundary_receipt_for(WorthQueryLowerRuntimeSeamKey::PublicLiveViewDeclaration)
            .unwrap()
            .boundary_execution_identity(),
        public_live_row
            .boundary_receipt
            .boundary_execution_identity()
    );
    assert_eq!(
        surface
            .route_plan_for(WorthQueryLowerRuntimeSeamKey::RuntimeLiveInstallationOrchestration)
            .unwrap()
            .route_digest(),
        orchestration_row
            .route_plan
            .as_ref()
            .unwrap()
            .route_digest()
    );
    assert_eq!(
        surface
            .route_plan_for(WorthQueryLowerRuntimeSeamKey::SubscriptionActivation)
            .unwrap()
            .route_digest(),
        activation_row.route_plan.as_ref().unwrap().route_digest()
    );
    assert_eq!(
        surface
            .route_plan_for(WorthQueryLowerRuntimeSeamKey::SubscriptionContinuity)
            .unwrap()
            .route_digest(),
        continuity_row.route_plan.as_ref().unwrap().route_digest()
    );
    assert_eq!(
        surface
            .boundary_receipt_for(WorthQueryLowerRuntimeSeamKey::PreviewBasisAdmission)
            .unwrap()
            .boundary_execution_identity(),
        preview_row.boundary_receipt.boundary_execution_identity()
    );
    assert_eq!(
        surface
            .boundary_receipt_for(
                WorthQueryLowerRuntimeSeamKey::BasisReadmissionFromTruthViewEvidence
            )
            .unwrap()
            .boundary_execution_identity(),
        truth_view_readmission_row
            .boundary_receipt
            .boundary_execution_identity()
    );
    assert_eq!(
        surface
            .boundary_receipt_for(
                WorthQueryLowerRuntimeSeamKey::BasisReadmissionFromSubscriptionEvidence
            )
            .unwrap()
            .boundary_execution_identity(),
        subscription_readmission_row
            .boundary_receipt
            .boundary_execution_identity()
    );
    assert_eq!(
        surface
            .route_plan_for(WorthQueryLowerRuntimeSeamKey::HistoricalBridgeLowering)
            .unwrap()
            .route_digest(),
        historical_row.route_plan.as_ref().unwrap().route_digest()
    );
    assert_eq!(
        surface
            .route_plan_for(WorthQueryLowerRuntimeSeamKey::EffectBackedRelationalMutation)
            .unwrap()
            .route_digest(),
        mutation_row.route_plan.as_ref().unwrap().route_digest()
    );
    assert_eq!(
        surface
            .route_plan_for(WorthQueryLowerRuntimeSeamKey::EffectBackedRelationalMerge)
            .unwrap()
            .route_digest(),
        merge_row.route_plan.as_ref().unwrap().route_digest()
    );
    assert_eq!(
        surface
            .route_plan_for(WorthQueryLowerRuntimeSeamKey::EffectBackedBridgeWriteback)
            .unwrap()
            .route_digest(),
        writeback_row.route_plan.as_ref().unwrap().route_digest()
    );
    assert_eq!(
        surface
            .route_plan_for(WorthQueryLowerRuntimeSeamKey::WriteAuthorityBackendExecution)
            .unwrap()
            .route_digest(),
        write_row.route_plan.as_ref().unwrap().route_digest()
    );
    assert_eq!(
        surface
            .boundary_receipt_for(WorthQueryLowerRuntimeSeamKey::WriteAuthorityBackendExecution)
            .unwrap()
            .boundary_execution_identity(),
        write_row.boundary_receipt.boundary_execution_identity()
    );
    assert_eq!(
        surface
            .route_plan_for(WorthQueryLowerRuntimeSeamKey::SignalInvalidationRouting)
            .unwrap()
            .route_digest(),
        signal_row.route_plan.as_ref().unwrap().route_digest()
    );
    assert_eq!(
        surface
            .route_plan_for(WorthQueryLowerRuntimeSeamKey::RuntimeIntentAuthorityAdapter)
            .unwrap()
            .route_digest(),
        runtime_intent_authority_row
            .route_plan
            .as_ref()
            .unwrap()
            .route_digest()
    );
    assert_eq!(
        surface
            .route_plan_for(WorthQueryLowerRuntimeSeamKey::IntentRuntimeExecution)
            .unwrap()
            .route_digest(),
        intent_runtime_execution_row
            .route_plan
            .as_ref()
            .unwrap()
            .route_digest()
    );
    assert_eq!(
        surface
            .boundary_receipt_for(
                WorthQueryLowerRuntimeSeamKey::ProjectionSourceIntakeFromQueryReceipts
            )
            .unwrap()
            .boundary_execution_identity(),
        query_receipt_row
            .boundary_receipt
            .boundary_execution_identity()
    );
    assert_eq!(
        surface
            .boundary_receipt_for(
                WorthQueryLowerRuntimeSeamKey::ProjectionSourceIntakeFromRelationalArtifacts
            )
            .unwrap()
            .boundary_execution_identity(),
        relational_row
            .boundary_receipt
            .boundary_execution_identity()
    );
    assert_eq!(
        surface
            .boundary_receipt_for(
                WorthQueryLowerRuntimeSeamKey::ProjectionSourceIntakeFromBridgeArtifacts
            )
            .unwrap()
            .boundary_execution_identity(),
        bridge_row.boundary_receipt.boundary_execution_identity()
    );
    assert_eq!(
        surface
            .route_plan_for(WorthQueryLowerRuntimeSeamKey::CausalBridgeMaterialization)
            .unwrap()
            .route_digest(),
        causal_row.route_plan.as_ref().unwrap().route_digest()
    );
    assert_eq!(
        surface
            .route_plan_for(WorthQueryLowerRuntimeSeamKey::FrontierEvidenceIntake)
            .unwrap()
            .route_digest(),
        frontier_row.route_plan.as_ref().unwrap().route_digest()
    );
    assert_eq!(
        surface
            .envelope_for(WorthQueryLowerRuntimeSeamKey::SignalInvalidationRouting)
            .unwrap()
            .envelope_identity(),
        signal_row.envelope.envelope_identity()
    );
    assert_eq!(
        surface
            .envelope_for(WorthQueryLowerRuntimeSeamKey::RuntimeLiveInstallationOrchestration)
            .unwrap()
            .envelope_identity(),
        orchestration_row.envelope.envelope_identity()
    );
}
