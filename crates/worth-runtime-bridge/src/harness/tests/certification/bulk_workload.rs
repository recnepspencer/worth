use super::super::support::{build_runtime, committed_patch, registration, snapshot};
use crate::facade::{
    BridgeBulkWorkloadRequest, BridgeBulkWorkloadSegment, BridgeParallelAdmissionClass,
    BridgeParallelAdmissionReason, BridgeRouteRequest,
};

#[test]
fn bridge_bulk_certifies_exact_counters_for_parallel_admitted_workload() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let source = crate::harness::fixtures::InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-b"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    source.insert_snapshot(snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    source.insert_snapshot(snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
        "bob",
    ));
    let runtime = build_runtime(
        source,
        crate::harness::fixtures::RecordingSignalBridgeSink::default(),
        vec![registration()],
    );

    let plan = runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(
                    crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                )),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(
                    crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
                )),
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
    assert_eq!(plan.packet_set().counters().bulk_packet_count(), 6);
    assert_eq!(plan.execution_plan().counters().bulk_routed_item_count(), 2);
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_normalized_workload_width(),
        13
    );
    assert_eq!(
        plan.execution_plan().counters().bulk_packet_entry_count(),
        6
    );
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_reduction_input_count(),
        4
    );
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_reduction_output_count(),
        4
    );
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_packet_queue_depth_peak(),
        6
    );
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_reducer_input_buffer_peak(),
        4
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

    let source = crate::harness::fixtures::InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-b"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    source.insert_snapshot(snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    let runtime = build_runtime(
        source,
        crate::harness::fixtures::RecordingSignalBridgeSink::default(),
        vec![registration()],
    );

    let plan = runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(
                    crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                )),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(
                    crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
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
    assert_eq!(plan.packet_set().counters().bulk_packet_count(), 5);
    assert_eq!(plan.execution_plan().counters().bulk_routed_item_count(), 2);
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_normalized_workload_width(),
        11
    );
    assert_eq!(
        plan.execution_plan().counters().bulk_packet_entry_count(),
        6
    );
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_reduction_input_count(),
        3
    );
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_reduction_output_count(),
        3
    );
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_packet_queue_depth_peak(),
        5
    );
    assert_eq!(
        plan.execution_plan()
            .counters()
            .bulk_reducer_input_buffer_peak(),
        3
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
