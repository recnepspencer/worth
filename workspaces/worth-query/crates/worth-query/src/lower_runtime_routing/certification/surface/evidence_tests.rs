use super::evidence::*;
use crate::lower_runtime_routing::WorthQueryLowerRuntimeSeamKey;

#[test]
fn representative_surface_covers_every_crossing_row_once() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let surface = worth_query_lower_runtime_representative_surface(resource_request);
    let crossing_count =
        crate::lower_runtime_routing::worth_query_lower_runtime_crossing_inventory()
            .rows()
            .len();

    assert_eq!(surface.requests().len(), crossing_count);
    assert_eq!(surface.eligibilities().len(), crossing_count);
    assert_eq!(surface.boundary_receipts().len(), crossing_count);
    assert_eq!(surface.envelopes().len(), crossing_count);
    assert!(!surface.route_parity_digest().is_empty());
}

#[test]
fn representative_surface_uses_runtime_backed_fixtures_for_named_phase_six_seams() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let surface = worth_query_lower_runtime_representative_surface(resource_request);

    for seam_key in [
        WorthQueryLowerRuntimeSeamKey::ComposeRead,
        WorthQueryLowerRuntimeSeamKey::ExecuteReadFamily,
        WorthQueryLowerRuntimeSeamKey::ExecuteReadFamilyInBasisContext,
        WorthQueryLowerRuntimeSeamKey::ExecuteRuntimeCurrentReadGraph,
        WorthQueryLowerRuntimeSeamKey::ExecuteRuntimeBasisContextReadGraph,
        WorthQueryLowerRuntimeSeamKey::LiveViewSchemaAdmission,
        WorthQueryLowerRuntimeSeamKey::LiveViewSourceDeclaration,
        WorthQueryLowerRuntimeSeamKey::PublicLiveViewDeclaration,
        WorthQueryLowerRuntimeSeamKey::RuntimeLiveInstallationOrchestration,
        WorthQueryLowerRuntimeSeamKey::PreviewBasisAdmission,
        WorthQueryLowerRuntimeSeamKey::SubscriptionActivation,
        WorthQueryLowerRuntimeSeamKey::SubscriptionContinuity,
        WorthQueryLowerRuntimeSeamKey::BasisReadmissionFromTruthViewEvidence,
        WorthQueryLowerRuntimeSeamKey::BasisReadmissionFromSubscriptionEvidence,
        WorthQueryLowerRuntimeSeamKey::HistoricalBridgeLowering,
        WorthQueryLowerRuntimeSeamKey::EffectBackedRelationalMutation,
        WorthQueryLowerRuntimeSeamKey::EffectBackedRelationalMerge,
        WorthQueryLowerRuntimeSeamKey::EffectBackedBridgeWriteback,
        WorthQueryLowerRuntimeSeamKey::WriteAuthorityBackendExecution,
        WorthQueryLowerRuntimeSeamKey::SignalInvalidationRouting,
        WorthQueryLowerRuntimeSeamKey::RuntimeIntentAuthorityAdapter,
        WorthQueryLowerRuntimeSeamKey::IntentRuntimeExecution,
        WorthQueryLowerRuntimeSeamKey::ProjectionSourceIntakeFromQueryReceipts,
        WorthQueryLowerRuntimeSeamKey::ProjectionSourceIntakeFromRelationalArtifacts,
        WorthQueryLowerRuntimeSeamKey::ProjectionSourceIntakeFromBridgeArtifacts,
        WorthQueryLowerRuntimeSeamKey::CausalBridgeMaterialization,
        WorthQueryLowerRuntimeSeamKey::FrontierEvidenceIntake,
    ] {
        assert_eq!(
            surface.evidence_source_for(seam_key),
            Some(WorthQueryLowerRuntimeRepresentativeEvidenceSource::RuntimeBackedFixture)
        );
    }
}
