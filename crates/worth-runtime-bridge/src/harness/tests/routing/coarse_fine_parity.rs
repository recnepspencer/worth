use crate::facade::BridgeRouteRequest;

use super::super::support::{
    build_runtime_with_aspects, committed_patch, field_aspect_registration, field_slice_snapshot,
    registration, surface_widening_registration,
};
use crate::harness::fixtures::{InMemoryRelationalBridgeSource, RecordingSignalBridgeSink};

#[test]
fn coarse_and_fine_routes_remain_parity_safe_for_shared_scope() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let left_source = InMemoryRelationalBridgeSource::default();
    left_source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    left_source.insert_snapshot(field_slice_snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    let left_runtime = build_runtime_with_aspects(
        left_source,
        RecordingSignalBridgeSink::default(),
        vec![registration()],
        vec![field_aspect_registration()],
    );

    let right_source = InMemoryRelationalBridgeSource::default();
    right_source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    right_source.insert_snapshot(field_slice_snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    let right_runtime = build_runtime_with_aspects(
        right_source,
        RecordingSignalBridgeSink::default(),
        vec![surface_widening_registration()],
        vec![field_aspect_registration()],
    );

    let left_result = left_runtime
        .deliver_invalidation(
            left_runtime
                .plan_committed_patch(
                    BridgeRouteRequest::for_commit(
                        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                    ),
                    execution,
                )
                .expect("left route should plan"),
            execution,
        )
        .expect("left route should deliver");
    let right_result = right_runtime
        .deliver_invalidation(
            right_runtime
                .plan_committed_patch(
                    BridgeRouteRequest::for_commit(
                        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                    ),
                    execution,
                )
                .expect("right route should plan"),
            execution,
        )
        .expect("right route should deliver");

    assert_eq!(
        left_result.result_summary().subscription_slice_identity(),
        right_result.result_summary().subscription_slice_identity()
    );
    assert_eq!(left_result.result_summary().subscription_slice_count(), 1);
    assert_eq!(right_result.result_summary().subscription_slice_count(), 1);
}
