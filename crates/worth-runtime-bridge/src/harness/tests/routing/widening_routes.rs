use crate::facade::{
    BridgeAspectRegistration, BridgeAspectRegistrationId, BridgeRouteRequest, MappingSelector,
    SliceWideningPolicy, SubscriptionSliceKind, TruthDeltaSurfaceKind, TruthPatchScope,
};

use super::super::support::{
    build_runtime_with_aspects, committed_partition_patch, surface_widening_registration,
};
use crate::harness::fixtures::{InMemoryRelationalBridgeSource, RecordingSignalBridgeSink};

#[test]
fn registered_partition_widening_routes_deterministically() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let left_source = InMemoryRelationalBridgeSource::default();
    left_source.insert_committed_patch(committed_partition_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    ));
    let left_runtime = build_runtime_with_aspects(
        left_source,
        RecordingSignalBridgeSink::default(),
        vec![surface_widening_registration()],
        vec![BridgeAspectRegistration::new(
            BridgeAspectRegistrationId::admit_bridge_owned("partition-widening"),
            TruthPatchScope::for_target(
                MappingSelector::exact("user"),
                worth_foundational::facade::AspectKey::new("profile")
                    .expect("valid native aspect key"),
                crate::facade::TruthPatchTargetSelector::partition(),
            ),
            crate::snapshot::SnapshotReadContract::scalar(
                worth_foundational::facade::AspectKey::new("profile")
                    .expect("valid native aspect key"),
                worth_foundational::facade::ScalarAspectType::String,
            ),
            TruthDeltaSurfaceKind::EntityPartition,
            SubscriptionSliceKind::SignalPartition,
            SliceWideningPolicy::RegisteredPartitionWidening,
        )],
    );

    let right_source = InMemoryRelationalBridgeSource::default();
    right_source.insert_committed_patch(committed_partition_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    ));
    let right_runtime = build_runtime_with_aspects(
        right_source,
        RecordingSignalBridgeSink::default(),
        vec![surface_widening_registration()],
        vec![BridgeAspectRegistration::new(
            BridgeAspectRegistrationId::admit_bridge_owned("partition-widening"),
            TruthPatchScope::for_target(
                MappingSelector::exact("user"),
                worth_foundational::facade::AspectKey::new("profile")
                    .expect("valid native aspect key"),
                crate::facade::TruthPatchTargetSelector::partition(),
            ),
            crate::snapshot::SnapshotReadContract::scalar(
                worth_foundational::facade::AspectKey::new("profile")
                    .expect("valid native aspect key"),
                worth_foundational::facade::ScalarAspectType::String,
            ),
            TruthDeltaSurfaceKind::EntityPartition,
            SubscriptionSliceKind::SignalPartition,
            SliceWideningPolicy::RegisteredPartitionWidening,
        )],
    );

    let left_route = left_runtime
        .plan_committed_patch(
            BridgeRouteRequest::for_commit(crate::truth_identity_fixtures::truth_commit_fixture(
                "commit-a",
            )),
            execution,
        )
        .expect("partition widening route should plan");
    let right_route = right_runtime
        .plan_committed_patch(
            BridgeRouteRequest::for_commit(crate::truth_identity_fixtures::truth_commit_fixture(
                "commit-a",
            )),
            execution,
        )
        .expect("partition widening route should plan deterministically");

    assert_eq!(left_route.subscription_slices().len(), 1);
    assert_eq!(
        left_route.subscription_slices().slices()[0].slice_kind(),
        &SubscriptionSliceKind::SignalPartition
    );
    assert_eq!(
        left_route.subscription_slices().slices()[0].match_status(),
        crate::facade::FineGrainedMatchStatus::WideningAdmitted
    );
    assert_eq!(left_route.counters().planned_slice_match_count(), 1);
    assert_eq!(left_route.counters().slice_widening_count(), 1);
    assert_eq!(left_route.counters().slice_suppression_count(), 0);
    assert_eq!(
        left_route.lowering_summary().subscription_slice_identity(),
        right_route.lowering_summary().subscription_slice_identity()
    );
}
