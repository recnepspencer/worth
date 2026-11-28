use super::{
    build_runtime, BridgeBulkWorkloadRequest, BridgeBulkWorkloadSegment, BridgeRouteRequest,
    RecordingSignalBridgeSink,
};
use crate::harness::fixtures::NamedTruth;

#[test]
fn bridge_bulk_reduction_artifact_is_stable_across_input_order() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (commit_a, _observation_a) = truth.commit_name(execution, "alice");
    let (commit_b, _observation_b) = truth.commit_name(execution, "bob");
    let left_source = truth.source.clone();

    let left_runtime = build_runtime(
        left_source,
        RecordingSignalBridgeSink::default(),
        vec![NamedTruth::mapping()],
    );

    let right_source = truth.source.clone();

    let right_runtime = build_runtime(
        right_source,
        RecordingSignalBridgeSink::default(),
        vec![NamedTruth::mapping()],
    );

    let left = left_runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(commit_a.clone())),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(commit_b.clone())),
            ]),
            execution,
        )
        .expect("left workload should plan");
    let right = right_runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(commit_b.clone())),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(commit_a.clone())),
            ]),
            execution,
        )
        .expect("right workload should plan");

    assert_eq!(
        left.execution_plan().reduced_artifact().digest(),
        right.execution_plan().reduced_artifact().digest()
    );
    assert_eq!(
        left.execution_plan()
            .reduced_artifact()
            .reduced_publications()[0]
            .publication_identity(),
        right
            .execution_plan()
            .reduced_artifact()
            .reduced_publications()[0]
            .publication_identity()
    );
    assert_eq!(left.packet_set().digest(), right.packet_set().digest());
}
