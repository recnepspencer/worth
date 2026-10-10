#[test]
fn bridge_bulk_execution_plan_selects_serial_when_parallel_is_legal_but_not_profitable() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let source = InMemoryRelationalBridgeSource::default();
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
        RecordingSignalBridgeSink::default(),
        vec![registration()],
    );

    let planned = runtime
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
