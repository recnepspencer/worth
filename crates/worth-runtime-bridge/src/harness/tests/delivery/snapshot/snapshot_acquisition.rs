use super::super::{
    build_runtime, commit_a, commit_b, committed_patch, patch_a, patch_b, registration, snapshot,
    snapshot_a, snapshot_b,
};
use crate::facade::{BridgeDeliveryErrorKind, BridgeRouteRequest, TruthSnapshotIdentity};
use crate::harness::fixtures::{InMemoryRelationalBridgeSource, RecordingSignalBridgeSink};

#[test]
fn bridge_delivery_fails_when_newer_truth_arrives_without_required_snapshot() {
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
    let runtime = build_runtime(
        source.clone(),
        RecordingSignalBridgeSink::default(),
        vec![registration()],
    );

    let route = runtime
        .plan_committed_patch(BridgeRouteRequest::for_commit(commit_a()), execution)
        .expect("bridge should plan the route");

    source.insert_committed_patch(committed_patch(
        commit_b(),
        patch_b(),
        snapshot_b(),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    source.insert_snapshot(snapshot(snapshot_b(), "bob"));

    let error = runtime
        .deliver_invalidation(route, execution)
        .expect_err("delivery should still require the original planned snapshot");

    assert_eq!(
        error.kind(),
        BridgeDeliveryErrorKind::SnapshotAcquisitionFailure
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
}
