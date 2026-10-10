use super::super::support::{
    build_pricing_runtime, capture_pricing_certification_matrix,
    capture_pricing_missing_snapshot_failure_bundle, generated_pricing_scenario, pricing_patch,
    pricing_patch_envelope_identity, BridgeDeliveryErrorKind, BridgeFailureClass,
    BridgePreviewSessionIdentity, BridgeRuntimePolicy, InMemoryRelationalBridgeSource,
    RecordingSignalBridgeSink,
};

#[test]
fn pricing_shock_certification_matrix_distinguishes_control_replay_and_hostile_lanes() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let scenario = generated_pricing_scenario();
    let control = capture_pricing_certification_matrix(
        BridgeRuntimePolicy::development(),
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-certification-control"),
        resource_request,
    );

    let hostile_source = InMemoryRelationalBridgeSource::default();
    hostile_source.insert_committed_patch(pricing_patch(
        pricing_patch_envelope_identity(
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-missing-snapshot"),
            crate::truth_identity_fixtures::truth_patch_fixture("patch:steel-missing-snapshot"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-missing"),
        ),
        "steel",
    ));
    let hostile_runtime =
        build_pricing_runtime(hostile_source, RecordingSignalBridgeSink::default());
    let hostile =
        capture_pricing_missing_snapshot_failure_bundle(&hostile_runtime, resource_request);

    assert_eq!(
        control.reference.route_snapshot,
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main")
    );
    assert_eq!(
        control.reference.source_branch,
        crate::truth_identity_fixtures::truth_branch_fixture("main")
    );
    assert_eq!(
        control.reference.source_commit,
        crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-main")
    );
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
    assert_eq!(
        control.replay.source_commit,
        crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-main")
    );
    assert_eq!(
        hostile.failure_class,
        BridgeFailureClass::Delivery(BridgeDeliveryErrorKind::SnapshotAcquisitionFailure)
    );
    assert_eq!(
        hostile.source_commit,
        crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-missing-snapshot")
    );
    assert_eq!(
        hostile.source_snapshot,
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-missing")
    );
}
