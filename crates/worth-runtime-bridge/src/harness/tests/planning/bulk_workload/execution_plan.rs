use crate::harness::fixtures::NamedTruth;
#[test]
fn bridge_bulk_execution_plan_carries_canonical_legality_proof() {
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
        .expect("left bulk workload should plan");
    let right = right_runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(commit_b.clone())),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(commit_a.clone())),
            ]),
            execution,
        )
        .expect("right bulk workload should plan");

    assert_eq!(
        left.execution_plan().parallel_admission().class(),
        BridgeParallelAdmissionClass::ParallelPreparationAdmitted
    );
    assert_eq!(
        left.execution_plan().parallel_admission().reason(),
        BridgeParallelAdmissionReason::AdmittedOperational
    );
    assert_eq!(
        left.execution_plan().legality_decision().class(),
        BridgeParallelLegalityClass::ParallelPreparationLegal
    );
    assert_eq!(
        left.execution_plan().legality_decision().reason(),
        BridgeParallelLegalityReason::DisjointPacketRegionsCertified
    );
    assert_eq!(
        left.execution_plan().profitability_decision().class(),
        BridgeParallelProfitabilityClass::Profitable
    );
    assert_eq!(
        left.execution_plan().profitability_decision().reason(),
        BridgeParallelProfitabilityReason::AdmittedOperational
    );
    assert_eq!(
        left.execution_plan().selected_mode(),
        BridgePreparationMode::ParallelPreparation
    );
    assert_eq!(
        left.execution_plan()
            .legality_proof()
            .canonical_planning_identity(),
        left.canonical_planning_identity()
    );
    assert_eq!(
        left.execution_plan().legality_proof().digest(),
        right.execution_plan().legality_proof().digest()
    );
    assert_eq!(
        left.execution_plan()
            .legality_proof()
            .disjoint_packet_regions()
            .regions()
            .len(),
        4
    );
    for region in left
        .execution_plan()
        .legality_proof()
        .disjoint_packet_regions()
        .regions()
    {
        assert!(region.as_str().starts_with("bulk-packet-region:sha256:"));
        assert!(!region.as_str().contains("partition:"));
        assert!(!region.as_str().contains("snapshot-a"));
        assert!(!region.as_str().contains("snapshot-b"));
    }
    assert_eq!(
        left.execution_plan()
            .legality_proof()
            .disjoint_packet_regions()
            .regions()
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        4
    );
    assert_eq!(
        left.execution_plan()
            .legality_proof()
            .admitted_partitions()
            .partitions()
            .len(),
        4
    );
    assert_eq!(
        left.execution_plan()
            .legality_proof()
            .disjoint_packet_regions()
            .regions(),
        left.execution_plan()
            .legality_proof()
            .admitted_partitions()
            .partitions(),
    );
    assert_eq!(
        left.execution_plan().reduced_artifact().digest(),
        right.execution_plan().reduced_artifact().digest()
    );
    assert_eq!(left.packet_set().digest(), right.packet_set().digest());
    assert_eq!(left.packet_set().routing_packets().len(), 2);
    assert_eq!(left.packet_set().truth_view_packets().len(), 2);
    assert_eq!(left.packet_set().reduction_packets().len(), 2);
    assert_eq!(left.packet_set().counters().bulk_packet_count(), 8);
    assert_eq!(left.packet_set().counters().bulk_packet_entry_count(), 8);
    assert_eq!(left.packet_set().counters().bulk_reduction_input_count(), 6);
    assert_eq!(
        left.packet_set().counters().bulk_reduction_output_count(),
        6
    );
    assert_eq!(
        left.execution_plan().counters().bulk_parallel_legal_count(),
        1
    );
    assert_eq!(
        left.execution_plan()
            .counters()
            .bulk_parallel_profitable_count(),
        1
    );
    assert!(left.execution_plan().planning_failures().is_empty());
    assert_eq!(left.execution_plan().decision_log().records().len(), 3);
    assert_eq!(
        left.execution_plan().decision_log().records()[0].kind(),
        BridgeBulkDecisionRecordKind::ParallelLegality
    );
    assert_eq!(
        left.execution_plan().decision_log().records()[1].kind(),
        BridgeBulkDecisionRecordKind::ParallelProfitability
    );
    assert_eq!(
        left.execution_plan().decision_log().records()[2].kind(),
        BridgeBulkDecisionRecordKind::ParallelAdmission
    );
    assert_eq!(
        left.execution_plan()
            .locality_footprint()
            .publication_scope_count(),
        2
    );
    assert_eq!(
        left.execution_plan()
            .reduced_artifact()
            .reduced_publications()
            .len(),
        2
    );
}

use super::*;
