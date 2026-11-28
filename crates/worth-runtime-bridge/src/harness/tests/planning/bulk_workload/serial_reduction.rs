use crate::harness::fixtures::NamedTruth;
#[test]
fn bridge_bulk_execution_plan_selects_serial_when_parallel_is_legal_but_not_profitable() {
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

    let planned = runtime
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
        .expect("legal-but-unprofitable workload should plan");

    assert_eq!(
        planned.execution_plan().selected_mode(),
        BridgePreparationMode::Serial
    );
    assert_eq!(
        planned.execution_plan().legality_decision().class(),
        BridgeParallelLegalityClass::ParallelPreparationLegal
    );
    assert_eq!(
        planned.execution_plan().profitability_decision().class(),
        BridgeParallelProfitabilityClass::Unprofitable
    );
    assert_eq!(
        planned.execution_plan().profitability_decision().reason(),
        BridgeParallelProfitabilityReason::SharedPublicationReductionTarget
    );
    assert_eq!(
        planned.execution_plan().parallel_admission().class(),
        BridgeParallelAdmissionClass::SerialRequired
    );
    assert_eq!(
        planned.execution_plan().parallel_admission().reason(),
        BridgeParallelAdmissionReason::SharedPublicationReductionTarget
    );
    assert_eq!(
        planned
            .execution_plan()
            .counters()
            .bulk_parallel_legal_count(),
        1
    );
    assert_eq!(
        planned
            .execution_plan()
            .counters()
            .bulk_parallel_profitable_count(),
        0
    );
    assert_eq!(
        planned
            .execution_plan()
            .counters()
            .bulk_parallel_serial_reduction_count(),
        1
    );
    assert_eq!(planned.execution_plan().planning_failures().len(), 1);
    assert_eq!(
        planned.execution_plan().planning_failures()[0].kind(),
        BridgeBulkPlanningFailureKind::ParallelPreparationNotProfitable
    );
    assert_eq!(
        planned.execution_plan().decision_log().records()[1].kind(),
        BridgeBulkDecisionRecordKind::ParallelProfitability
    );
}

use super::*;
