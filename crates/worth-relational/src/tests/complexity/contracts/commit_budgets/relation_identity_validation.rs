use super::*;

#[test]
fn complexity_budget_relation_identity_validation_avoids_partition_scan() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let source = create_entity(execution, &runtime, "source");
    let target = create_entity(execution, &runtime, "target");
    let _existing = create_relation(execution, &runtime, source, target, "existing");
    for index in 0..12 {
        let other_source = create_entity(execution, &runtime, &format!("other-source-{index}"));
        let other_target = create_entity(execution, &runtime, &format!("other-target-{index}"));
        let _ = create_relation(
            execution,
            &runtime,
            other_source,
            other_target,
            &format!("r{index}"),
        );
    }

    runtime.performance_access().reset_counters();
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("duplicate").push(MutationIntent::Create(CreateIntent::Relation(
            crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("dup"),
                source: crate::transactions::data::EntityReference::Existing(source),
                target: crate::transactions::data::EntityReference::Existing(target),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            },
        ))),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let error = txn
        .commit(
            execution,
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap_err();
    let counters = runtime.performance_access().counters();

    assert!(matches!(
        error,
        TransactionCommitError::Conflict { error: ref conflict, .. }
            if conflict.code == DiagnosticCode::DuplicateRelationIdentity
    ));
    assert_eq!(counters.relation_identity_candidates_scanned, 1);
}
