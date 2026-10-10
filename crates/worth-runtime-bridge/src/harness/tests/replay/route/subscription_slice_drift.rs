use super::super::super::support::{
    build_runtime_with_aspects, committed_patch, field_aspect_registration,
    field_aspect_registration_with_kind, field_slice_snapshot, registration,
};
use crate::facade::{BridgeRouteRequest, SubscriptionSliceKind, TruthDeltaSurfaceKind};
use crate::harness::fixtures::{InMemoryRelationalBridgeSource, RecordingSignalBridgeSink};

#[test]
fn bridge_replay_rejects_subscription_slice_drift() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let original_source = InMemoryRelationalBridgeSource::default();
    original_source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    original_source.insert_snapshot(field_slice_snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    let original_runtime = build_runtime_with_aspects(
        original_source,
        RecordingSignalBridgeSink::default(),
        vec![registration()],
        vec![field_aspect_registration()],
    );

    let route = original_runtime
        .plan_committed_patch(
            BridgeRouteRequest::for_commit(crate::truth_identity_fixtures::truth_commit_fixture(
                "commit-a",
            )),
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

    let restarted_source = InMemoryRelationalBridgeSource::default();
    restarted_source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    restarted_source.insert_snapshot(field_slice_snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    let restarted_runtime = build_runtime_with_aspects(
        restarted_source,
        RecordingSignalBridgeSink::default(),
        vec![registration()],
        vec![field_aspect_registration_with_kind(
            "profile-name-region",
            TruthDeltaSurfaceKind::EntityField,
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
