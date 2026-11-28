use super::{
    build_runtime, BridgeBulkWorkloadRequest, BridgeBulkWorkloadSegment, BridgeRouteRequest,
    RecordingSignalBridgeSink,
};
use crate::harness::fixtures::NamedTruth;

#[test]
fn bridge_bulk_reduction_collapses_duplicate_publications_deterministically() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (commit_a, _observation_a) = truth.commit_name(execution, "alice");
    let source = truth.source.clone();

    let runtime = build_runtime(
        source,
        RecordingSignalBridgeSink::default(),
        vec![NamedTruth::mapping()],
    );

    let planned = runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(commit_a.clone())),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(commit_a.clone())),
            ]),
            execution,
        )
        .expect("duplicate bulk workload should plan");
    let reduced_artifact = planned.execution_plan().reduced_artifact();

    assert_eq!(reduced_artifact.reduction_input_count(), 4);
    assert_eq!(reduced_artifact.reduction_output_count(), 3);
    assert_eq!(reduced_artifact.counters().bulk_reduction_input_count(), 4);
    assert_eq!(reduced_artifact.reduced_truth_views().len(), 1);
    assert_eq!(reduced_artifact.reduced_publications().len(), 1);
    assert_eq!(
        reduced_artifact.reduced_publications()[0]
            .reduced_route_identities()
            .len(),
        2
    );
    assert_eq!(
        reduced_artifact.reduced_publications()[0].reduced_route_identities(),
        planned
            .packet_set()
            .routing_packets()
            .iter()
            .map(|packet| packet.route_identity().clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        reduced_artifact.reduced_publications()[0].invalidation_target_count(),
        2
    );
}
