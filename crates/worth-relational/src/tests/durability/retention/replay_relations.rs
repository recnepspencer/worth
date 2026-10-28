use super::*;

#[test]
fn retention_plan_reports_explicit_replay_pins_for_deleted_relations_until_released() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let source = create_entity(execution, &runtime, "replay-left");
    let target = create_entity(execution, &runtime, "replay-right");
    let created = create_relation_outcome(execution, &runtime, source, target, "replay-r1");
    let relation = changed_relations(&created)[0];
    let deleted = {
        let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
        txn.push_batch(
            WorkerIntentBatch::new("delete-relation").push(MutationIntent::Relation(
                RelationMutationIntent::Delete(DeleteRelationIntent {
                    relation_id: relation,
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
        txn.commit(execution, &runtime).unwrap()
    };

    assert!(runtime
        .visibility_authority()
        .release_snapshot(&created.snapshot)
        .is_ok());
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&deleted.snapshot)
        .is_ok());
    assert!(runtime
        .history_authority()
        .retain_version_for_replay(execution, created.version_id)
        .unwrap());

    let pinned = runtime.retention().inspect_plan(execution).unwrap();
    let diagnostics = runtime.publication().diagnostics();
    let retention_artifacts = diagnostics.by_scope(DiagnosticsScope::Retention);
    let latest_retention = retention_artifacts.last().unwrap();
    let latest_entry = latest_retention.entries.last().unwrap();
    assert!(retention_artifacts
        .iter()
        .flat_map(|artifact| artifact.entries.iter())
        .any(|entry| entry.code == DiagnosticCode::ReplayRetentionPinned));
    assert!(pinned.replay_pinned_relations >= 1);
    assert_eq!(pinned.reclaimable_relations, 0);
    assert_eq!(latest_entry.code, DiagnosticCode::RetentionPlanInspected);
    assert_eq!(
        diagnostic_field(latest_entry, "replay_pinned_relations"),
        &RelationalDiagnosticValue::Unsigned(pinned.replay_pinned_relations as u64)
    );
    assert_eq!(
        diagnostic_field(latest_entry, "branch_pinned_relations"),
        &RelationalDiagnosticValue::Unsigned(pinned.branch_pinned_relations as u64)
    );
    assert_eq!(
        diagnostic_unsigned_field(latest_entry, "branch_replay_overlap_relations"),
        0
    );

    assert!(runtime
        .history_authority()
        .release_version_replay_retention(created.version_id));
    let released = runtime.retention().inspect_plan(execution).unwrap();
    assert_eq!(released.replay_pinned_relations, 0);
    assert_eq!(released.branch_pinned_relations, 0);
}
