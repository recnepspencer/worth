use super::*;

#[test]
fn bridge_bulk_explanation_reconstructs_canonical_bulk_plan_truth() {
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
        RecordingSignalBridgeSink::default(),
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
        .expect("bulk workload should plan before explanation reconstruction");
    let record = runtime.canonicalize_bulk_workload_plan(&plan);

    let explanation = runtime.diagnostics().explain_bulk_record(&record);

    assert_eq!(explanation.workload_identity(), plan.workload_identity());
    assert_eq!(
        explanation.canonical_planning_identity(),
        plan.canonical_planning_identity()
    );
    assert_eq!(
        explanation.admission_profile_identity(),
        plan.admission_profile_identity()
    );
    assert_eq!(
        explanation.selected_mode(),
        BridgePreparationMode::ParallelPreparation
    );
    assert_eq!(explanation.request_segment_count(), 2);
    assert_eq!(explanation.packet_set_digest(), plan.packet_set().digest());
    assert_eq!(
        explanation.execution_plan_digest(),
        plan.execution_plan().digest()
    );
    assert_eq!(
        explanation.reduced_artifact_digest(),
        plan.execution_plan().reduced_artifact().digest()
    );
    assert_eq!(
        explanation.decision_log_digest(),
        plan.execution_plan().decision_log().digest()
    );
    assert_eq!(
        explanation.decision_log(),
        plan.execution_plan().decision_log()
    );
    assert_eq!(
        explanation
            .counters()
            .bulk_parallel_preparation_admitted_count(),
        1
    );
    assert!(explanation.planning_failures().is_empty());
    assert_eq!(explanation.planning_failure_count(), 0);
}
#[test]
fn bridge_bulk_explanation_retains_typed_parallel_serial_reduction_failures() {
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
        .expect("bulk workload should plan before explanation reconstruction");
    let explanation = runtime
        .diagnostics()
        .explain_bulk_record(&runtime.canonicalize_bulk_workload_plan(&plan));

    assert_eq!(explanation.selected_mode(), BridgePreparationMode::Serial);
    assert_eq!(
        explanation.decision_log(),
        plan.execution_plan().decision_log()
    );
    assert_eq!(
        explanation.planning_failures(),
        plan.execution_plan().planning_failures()
    );
    assert_eq!(explanation.planning_failure_count(), 1);
    assert_eq!(
        explanation.planning_failures()[0].kind(),
        crate::facade::BridgeBulkPlanningFailureKind::ParallelPreparationNotProfitable
    );
}
