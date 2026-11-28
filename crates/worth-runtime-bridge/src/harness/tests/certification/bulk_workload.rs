use super::super::support::build_runtime;
use crate::facade::{
    BridgeBulkWorkloadRequest, BridgeBulkWorkloadSegment, BridgeParallelAdmissionClass,
    BridgeParallelAdmissionReason, BridgeRouteRequest,
};
use crate::harness::fixtures::NamedTruth;

#[test]
fn bridge_bulk_certifies_exact_counters_for_parallel_admitted_workload() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (commit_a, _observation_a) = truth.commit_name(execution, "alice");
    let (commit_b, _observation_b) = truth.commit_name(execution, "bob");
    let source = truth.source.clone();

    let runtime = build_runtime(
        source,
        crate::harness::fixtures::RecordingSignalBridgeSink::default(),
        vec![NamedTruth::mapping()],
    );

    let plan = runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(commit_a.clone())),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(commit_b.clone())),
            ]),
            execution,
        )
        .expect("parallel-admitted workload should plan");

    assert_eq!(
        plan.execution_plan().parallel_admission().class(),
        BridgeParallelAdmissionClass::ParallelPreparationAdmitted
    );
    assert_eq!(
        plan.execution_plan().parallel_admission().reason(),
        BridgeParallelAdmissionReason::AdmittedOperational
    );
    assert_eq!(plan.packet_set().counters().bulk_packet_count(), 8);
    assert_eq!(plan.execution_plan().counters().bulk_routed_item_count(), 2);
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_normalized_workload_width(),
        13
    );
    assert_eq!(
        plan.execution_plan().counters().bulk_packet_entry_count(),
        8
    );
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_reduction_input_count(),
        6
    );
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_reduction_output_count(),
        6
    );
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_packet_queue_depth_peak(),
        8
    );
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_reducer_input_buffer_peak(),
        6
    );
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_replay_mismatch_count(),
        0
    );
    assert_eq!(
        plan.execution_plan().counters().bulk_parallel_legal_count(),
        1
    );
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_parallel_profitable_count(),
        1
    );
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_parallel_preparation_admitted_count(),
        1
    );
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_parallel_serial_reduction_count(),
        0
    );
}

#[test]
fn bridge_bulk_certifies_exact_counters_for_serial_reduction_workload() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (commit_a, _observation_a) = truth.commit_name(execution, "alice");
    let (commit_b, observation_b) = truth.commit_name(execution, "bob");
    let shared_snapshot = observation_b.snapshot_identity().clone();
    let source = truth.source.clone();

    let runtime = build_runtime(
        source,
        crate::harness::fixtures::RecordingSignalBridgeSink::default(),
        vec![NamedTruth::mapping()],
    );

    let plan = runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::from(
                    crate::facade::RelationalCommittedPatchRequest::at_snapshot(
                        commit_a.clone(),
                        shared_snapshot.clone(),
                    ),
                )),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::from(
                    crate::facade::RelationalCommittedPatchRequest::at_snapshot(
                        commit_b.clone(),
                        shared_snapshot.clone(),
                    ),
                )),
            ]),
            execution,
        )
        .expect("serial-reduction workload should plan");

    assert_eq!(
        plan.execution_plan().parallel_admission().class(),
        BridgeParallelAdmissionClass::SerialRequired
    );
    assert_eq!(
        plan.execution_plan().parallel_admission().reason(),
        BridgeParallelAdmissionReason::SharedPublicationReductionTarget
    );
    assert_eq!(plan.packet_set().counters().bulk_packet_count(), 7);
    assert_eq!(plan.execution_plan().counters().bulk_routed_item_count(), 2);
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_normalized_workload_width(),
        11
    );
    assert_eq!(
        plan.execution_plan().counters().bulk_packet_entry_count(),
        8
    );
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_reduction_input_count(),
        5
    );
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_reduction_output_count(),
        5
    );
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_packet_queue_depth_peak(),
        7
    );
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_reducer_input_buffer_peak(),
        5
    );
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_replay_mismatch_count(),
        0
    );
    assert_eq!(
        plan.execution_plan().counters().bulk_parallel_legal_count(),
        1
    );
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_parallel_profitable_count(),
        0
    );
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_parallel_serial_reduction_count(),
        1
    );
    assert_eq!(plan.execution_plan().planning_failures().len(), 1);
}
