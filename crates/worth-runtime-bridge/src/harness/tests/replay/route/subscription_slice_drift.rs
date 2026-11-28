use super::super::super::support::build_runtime_with_aspects;
use crate::facade::{BridgeRouteRequest, SubscriptionSliceKind};
use crate::harness::fixtures::{ProfileTruth, RecordingSignalBridgeSink};

#[test]
fn bridge_replay_rejects_subscription_slice_drift() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = ProfileTruth::new(execution);
    let original_source = truth.native.source.clone();
    let original_runtime = build_runtime_with_aspects(
        original_source,
        RecordingSignalBridgeSink::default(),
        vec![ProfileTruth::mapping()],
        vec![ProfileTruth::aspect_mapping()],
    );

    let route = original_runtime
        .plan_committed_patch(
            BridgeRouteRequest::for_commit(truth.commit.clone()),
            execution,
        )
        .expect("original route should plan before replay certification");
    original_runtime
        .deliver_invalidation(route, execution)
        .expect("original route should deliver before replay certification");
    let canonical_record = original_runtime
        .diagnostics()
        .last_canonical_route_record()
        .expect("original runtime should expose a canonical route record");

    let restarted_source = truth.native.source.clone();
    let restarted_runtime = build_runtime_with_aspects(
        restarted_source,
        RecordingSignalBridgeSink::default(),
        vec![ProfileTruth::mapping()],
        vec![ProfileTruth::aspect_mapping_with_kind(
            "profile-title-region",
            SubscriptionSliceKind::SignalRegion,
        )],
    );

    let error = restarted_runtime
        .replay_canonical_record(&canonical_record, execution)
        .expect_err("replay should reject subscription slice identity drift");
    let canonical_route_record = canonical_record
        .decode()
        .expect("test canonical route record should decode");

    assert_eq!(
        error.kind(),
        crate::facade::BridgeReplayErrorKind::SubscriptionSliceMismatch
    );
    assert_eq!(
        error.context().route_identity(),
        Some(canonical_route_record.route_identity())
    );
    assert_eq!(
        error.context().snapshot_identity(),
        Some(canonical_route_record.source_snapshot())
    );
    assert_eq!(
        error.context().subscription_slice_identity(),
        Some(canonical_route_record.subscription_slice_identity())
    );
}
