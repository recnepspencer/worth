use crate::harness::fixtures::NamedTruth;
#[test]
fn bridge_bulk_execution_plan_rejects_parallel_preparation_for_shared_truth_view_targets() {
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
        .expect("shared truth-view workload should plan");

    assert_eq!(
        planned.execution_plan().selected_mode(),
        BridgePreparationMode::Serial
    );
    assert_eq!(
        planned.execution_plan().parallel_admission().class(),
        BridgeParallelAdmissionClass::ParallelPreparationRejected
    );
    assert_eq!(
        planned.execution_plan().parallel_admission().reason(),
        BridgeParallelAdmissionReason::SharedTruthViewMaterializationTarget
    );
    assert_eq!(
        planned.execution_plan().legality_decision().class(),
        BridgeParallelLegalityClass::ParallelPreparationIllegal
    );
    assert_eq!(
        planned.execution_plan().legality_decision().reason(),
        BridgeParallelLegalityReason::SharedTruthViewMaterializationTarget
    );
    assert_eq!(
        planned.execution_plan().profitability_decision().class(),
        BridgeParallelProfitabilityClass::NotApplicable
    );
    assert_eq!(
        planned
            .execution_plan()
            .counters()
            .bulk_parallel_preparation_rejected_count(),
        1
    );
    assert_eq!(
        planned
            .execution_plan()
            .counters()
            .bulk_parallel_serial_reduction_count(),
        0
    );
    assert_eq!(planned.execution_plan().planning_failures().len(), 1);
    assert_eq!(
        planned.execution_plan().planning_failures()[0].kind(),
        BridgeBulkPlanningFailureKind::InvalidParallelAdmissionBasis
    );
    assert_eq!(planned.packet_set().truth_view_packets().len(), 1);
    assert!(planned
        .execution_plan()
        .legality_proof()
        .admitted_partitions()
        .partitions()
        .is_empty());
    assert!(planned
        .execution_plan()
        .legality_proof()
        .disjoint_packet_regions()
        .regions()
        .is_empty());
}

#[test]
fn bridge_bulk_execution_plan_rejects_parallel_preparation_for_continuity_remap_workloads() {
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
        RecordingSignalBridgeSink::default(),
        vec![NamedTruth::mapping()],
    );
    let lineage_context = BridgeLineageContext::new(BridgeContinuityAuthorityBasis::new(
        crate::truth_identity_fixtures::truth_branch_fixture("main"),
        observation_b.snapshot_identity().clone(),
    ));

    let planned = runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::from(
                    crate::facade::RelationalCommittedPatchRequest::at_snapshot(
                        commit_a.clone(),
                        shared_snapshot.clone(),
                    ),
                ))
                .with_mapping_context(
                    BridgeMappingContext::default().with_lineage_context(lineage_context.clone()),
                ),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::from(
                    crate::facade::RelationalCommittedPatchRequest::at_snapshot(
                        commit_b.clone(),
                        shared_snapshot.clone(),
                    ),
                ))
                .with_mapping_context(
                    BridgeMappingContext::default().with_lineage_context(lineage_context),
                ),
            ]),
            execution,
        )
        .expect("continuity remap workload should plan");

    assert_eq!(
        planned.execution_plan().selected_mode(),
        BridgePreparationMode::Serial
    );
    assert_eq!(
        planned.execution_plan().parallel_admission().class(),
        BridgeParallelAdmissionClass::ParallelPreparationRejected
    );
    assert_eq!(
        planned.execution_plan().parallel_admission().reason(),
        BridgeParallelAdmissionReason::ContinuityRemapRequiresSerialPreparation
    );
    assert_eq!(
        planned.execution_plan().legality_decision().class(),
        BridgeParallelLegalityClass::ParallelPreparationIllegal
    );
    assert_eq!(
        planned.execution_plan().legality_decision().reason(),
        BridgeParallelLegalityReason::ContinuityRemapRequiresSerialPreparation
    );
    assert_eq!(
        planned.execution_plan().profitability_decision().class(),
        BridgeParallelProfitabilityClass::NotApplicable
    );
    assert_eq!(
        planned
            .execution_plan()
            .counters()
            .bulk_parallel_preparation_rejected_count(),
        1
    );
    assert_eq!(planned.execution_plan().planning_failures().len(), 1);
    assert_eq!(
        planned.execution_plan().planning_failures()[0].kind(),
        BridgeBulkPlanningFailureKind::InvalidParallelAdmissionBasis
    );
    assert!(planned
        .execution_plan()
        .legality_proof()
        .admitted_partitions()
        .partitions()
        .is_empty());
    assert!(planned
        .execution_plan()
        .legality_proof()
        .disjoint_packet_regions()
        .regions()
        .is_empty());
}

use super::*;
