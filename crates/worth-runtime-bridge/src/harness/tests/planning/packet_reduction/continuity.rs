#[test]
fn bridge_bulk_packet_set_tracks_continuity_remap_packets() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (commit_a, _first) = truth.commit_name(execution, "alice");
    let (commit_b, second) = truth.commit_name(execution, "bob");
    let snapshot = second.snapshot_identity();
    let runtime = build_runtime(
        truth.source.clone(),
        RecordingSignalBridgeSink::default(),
        vec![NamedTruth::mapping()],
    );
    let lineage_context = BridgeLineageContext::new(BridgeContinuityAuthorityBasis::new(
        crate::facade::TruthBranchIdentity::from_relational_branch_id("main"),
        snapshot.clone(),
    ));
    let route_at_snapshot = |commit| {
        crate::facade::RelationalCommittedPatchRequest::at_snapshot(commit, snapshot.clone()).into()
    };

    let planned = runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(route_at_snapshot(commit_a)).with_mapping_context(
                    BridgeMappingContext::default().with_lineage_context(lineage_context.clone()),
                ),
                BridgeBulkWorkloadSegment::new(route_at_snapshot(commit_b)).with_mapping_context(
                    BridgeMappingContext::default().with_lineage_context(lineage_context),
                ),
            ]),
            execution,
        )
        .expect("continuity-bearing bulk workload should plan");

    assert_eq!(planned.packet_set().continuity_packets().len(), 2);
    assert_eq!(
        planned
            .packet_set()
            .continuity_packets()
            .iter()
            .map(|packet| packet.continuity_member_identity().clone())
            .collect::<std::collections::BTreeSet<_>>(),
        planned
            .canonical_request()
            .continuity_members()
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>()
    );
    assert_eq!(
        planned
            .packet_set()
            .continuity_packets()
            .iter()
            .map(|packet| packet.originating_route_identity().clone())
            .collect::<Vec<_>>(),
        planned
            .packet_set()
            .routing_packets()
            .iter()
            .map(|packet| packet.route_identity().clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        planned
            .packet_set()
            .continuity_packets()
            .iter()
            .map(|packet| packet.snapshot_identity())
            .collect::<Vec<_>>(),
        vec![snapshot.canonical_label(), snapshot.canonical_label()]
    );
    assert_eq!(
        planned
            .packet_set()
            .continuity_packets()
            .iter()
            .map(|packet| packet.prior_slice_count())
            .sum::<usize>(),
        2
    );
}

#[test]
fn bridge_bulk_reduction_artifact_carries_continuity_remaps() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = ProfileTruth::new(execution);
    let build = || {
        build_runtime(
            truth.native.source.clone(),
            RecordingSignalBridgeSink::default(),
            vec![ProfileTruth::mapping()],
        )
    };
    let left_runtime = build();
    let right_runtime = build();
    let request = BridgeBulkWorkloadRequest::new(vec![BridgeBulkWorkloadSegment::new(
        BridgeRouteRequest::for_commit(truth.commit.clone()),
    )
    .with_mapping_context(
        BridgeMappingContext::default().with_lineage_context(truth.lineage_context()),
    )]);

    let left = left_runtime
        .plan_bulk_workload(request.clone(), execution)
        .expect("left continuity-bearing workload should plan");
    let right = right_runtime
        .plan_bulk_workload(request, execution)
        .expect("right continuity-bearing workload should plan");

    assert_eq!(
        left.execution_plan()
            .reduced_artifact()
            .reduced_continuity_remaps()
            .len(),
        1
    );
    assert_eq!(
        left.execution_plan()
            .reduced_artifact()
            .reduced_continuity_remaps(),
        right
            .execution_plan()
            .reduced_artifact()
            .reduced_continuity_remaps()
    );
    assert_eq!(
        left.execution_plan()
            .reduced_artifact()
            .reduced_continuity_remaps()[0]
            .continuity_member_identity(),
        left.packet_set().continuity_packets()[0].continuity_member_identity()
    );
    assert_eq!(
        left.execution_plan()
            .reduced_artifact()
            .reduced_continuity_remaps()[0]
            .snapshot_identity(),
        truth.observation.snapshot_identity().canonical_label()
    );
    assert_eq!(
        left.execution_plan()
            .reduced_artifact()
            .reduced_continuity_remaps()[0]
            .prior_slice_count(),
        1
    );
}

use super::*;

use crate::harness::fixtures::{NamedTruth, ProfileTruth};
