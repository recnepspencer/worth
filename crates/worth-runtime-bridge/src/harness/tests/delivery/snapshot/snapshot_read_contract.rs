use super::super::{
    build_runtime_with_aspects, commit_a, committed_patch, field_aspect_registration, patch_a,
    registration, snapshot_a,
};
use crate::facade::{BridgeDeliveryErrorKind, BridgeRouteRequest, TruthSnapshotIdentity};
use crate::harness::fixtures::{
    InMemoryRelationalBridgeSource, RecordingSignalBridgeSink, SnapshotFixture,
};

#[test]
fn bridge_snapshot_contract_rejects_missing_required_reads() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let source = InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(committed_patch(
        commit_a(),
        patch_a(),
        snapshot_a(),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    source.insert_snapshot(SnapshotFixture::new(snapshot_a(), vec![]));
    let runtime = build_runtime_with_aspects(
        source,
        RecordingSignalBridgeSink::default(),
        vec![registration()],
        vec![field_aspect_registration()],
    );

    let route = runtime
        .plan_committed_patch(BridgeRouteRequest::for_commit(commit_a()), execution)
        .expect("bridge should plan before validating snapshot reads");
    let expected_target_identity = route.read_packet().reads()[0].target_identity().clone();

    let error = runtime
        .deliver_invalidation(route, execution)
        .expect_err("bridge should reject incomplete snapshot read results");

    assert!(
        matches!(error.kind(), BridgeDeliveryErrorKind::SnapshotReadContractViolation(cause) if cause.kind() == crate::snapshot::BridgeSnapshotReadErrorKind::RecordCountMismatch)
    );
    assert_eq!(
        error
            .context()
            .snapshot_identity()
            .and_then(TruthSnapshotIdentity::relational_snapshot_parts),
        Some(crate::facade::RelationalBridgeSnapshotIdentityParts::new(
            1, 1
        ))
    );
    let snapshot_read = error
        .context()
        .snapshot_read()
        .expect("snapshot contract violation should retain read coordinate");
    assert_eq!(snapshot_read.entity_identity(), "user");
    assert_eq!(snapshot_read.aspect_key().as_str(), "profile");
    assert_eq!(
        snapshot_read
            .target_identity()
            .expect("subscription-slice read coordinate should retain target identity"),
        &expected_target_identity
    );
    assert!(snapshot_read
        .target_identity()
        .expect("subscription-slice read coordinate should retain target identity")
        .as_str()
        .starts_with("snapshot-read-target:sha256:"));
}

#[test]
fn leased_route_surfaces_cancellation_before_snapshot_reads() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let source = InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(committed_patch(
        commit_a(),
        patch_a(),
        snapshot_a(),
        worth_foundational::facade::FieldKey::new("name".to_owned()).unwrap(),
    ));
    source.insert_snapshot(SnapshotFixture::new(snapshot_a(), vec![]));
    let runtime = build_runtime_with_aspects(
        source,
        RecordingSignalBridgeSink::default(),
        vec![registration()],
        vec![field_aspect_registration()],
    );
    let route = runtime
        .plan_committed_patch(BridgeRouteRequest::for_commit(commit_a()), execution)
        .unwrap();
    let cancellation = worth_execution::CancellationSource::new();
    cancellation.cancel();
    let lease = crate::snapshot::test_execution_lease(cancellation.token());

    let error = runtime
        .deliver_invalidation(route, worth_execution::ExecutionRequest::leased(&lease))
        .expect_err("cancelled lease must deny the read");
    assert_eq!(
        error.kind(),
        BridgeDeliveryErrorKind::ExecutionDenied(crate::error::BridgeExecutionDenial::Cancelled)
    );
}
