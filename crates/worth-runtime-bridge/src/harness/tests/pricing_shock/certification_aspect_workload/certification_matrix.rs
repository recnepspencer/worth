use super::super::support::{
    capture_pricing_certification_matrix, capture_pricing_missing_snapshot_failure_bundle,
    generated_pricing_scenario, BridgeDeliveryErrorKind, BridgeFailureClass,
    BridgePreviewSessionIdentity, BridgeRuntimePolicy, RelationalPricing,
};

#[test]
fn pricing_shock_certification_matrix_distinguishes_control_replay_and_hostile_lanes() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let scenario = generated_pricing_scenario();
    let truth = RelationalPricing::reference(resource_request, &scenario, None);
    let control = capture_pricing_certification_matrix(
        BridgeRuntimePolicy::development(),
        &truth,
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-certification-control"),
        resource_request,
    );

    let hostile = capture_pricing_missing_snapshot_failure_bundle(
        &truth,
        BridgeRuntimePolicy::development(),
        resource_request,
    );

    assert_eq!(
        control.reference.route_snapshot,
        truth.steel_snapshot().clone()
    );
    assert_eq!(
        control.reference.source_branch.relational_branch_id(),
        Some("main")
    );
    assert_eq!(control.reference.source_commit, truth.steel_commit.clone());
    assert_eq!(control.reference.route_entry_count, 2);
    assert_eq!(
        control.reference.main_rubber_cost_cents,
        scenario.main_rubber_cost
    );
    assert_eq!(
        control.reference.speculative_rubber_cost_cents,
        scenario.speculative_rubber_cost
    );
    assert!(!control
        .reference
        .evaluation_record_identity
        .as_str()
        .is_empty());
    assert!(!control
        .reference
        .evaluation_selector_identity
        .as_str()
        .is_empty());
    assert_eq!(
        control.replay.source_snapshot,
        control.reference.route_snapshot
    );
    assert_eq!(control.replay.source_commit, truth.steel_commit.clone());
    assert!(
        matches!(hostile.failure_class, BridgeFailureClass::Delivery(BridgeDeliveryErrorKind::SnapshotAcquisitionFailure(ref cause)) if cause.kind() == crate::adapter::RelationalBridgeSourceErrorTag::ExternalSourceFailure)
    );
    assert_eq!(hostile.source_commit, truth.steel_commit.clone());
    assert_eq!(hostile.source_snapshot, truth.steel_snapshot().clone());
}
