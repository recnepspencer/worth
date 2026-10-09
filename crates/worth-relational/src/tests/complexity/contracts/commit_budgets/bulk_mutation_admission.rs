use super::*;

#[test]
fn complexity_budget_bulk_mutation_planning_reports_identity_scope_and_batch_evidence() {
    let runtime = runtime_with_test_schema();
    let source = create_entity_in_partition(&runtime, "source", PartitionId(7));
    let target = create_entity_in_partition(&runtime, "target", PartitionId(11));

    runtime.performance_access().reset_counters();
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("entities").push(MutationIntent::Create(
            CreateIntent::BulkEntities(BulkEntityCreateIntent {
                partition_id: PartitionId(7),
                kind_id: KindId(1),
                client_keys: vec![
                    crate::symbols::data::ClientKey::raw("alpha"),
                    crate::symbols::data::ClientKey::raw("beta"),
                ],
                field_patches: vec![
                    crate::tests::support::single_string_aspect_field_patch(
                        crate::tests::support::aspect_key("name"),
                        crate::tests::support::field_key("name"),
                        "alpha",
                    ),
                    crate::tests::support::single_string_aspect_field_patch(
                        crate::tests::support::aspect_key("name"),
                        crate::tests::support::field_key("name"),
                        "beta",
                    ),
                ],
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    txn.push_batch(
        WorkerIntentBatch::new("relations").push(MutationIntent::Create(
            CreateIntent::BulkRelations(BulkRelationCreateIntent {
                partition_id: PartitionId(13),
                kind_id: KindId(2),
                client_keys: vec![crate::symbols::data::ClientKey::raw("edge")],
                endpoints: vec![(
                    crate::transactions::data::EntityReference::Existing(source),
                    crate::transactions::data::EntityReference::Existing(target),
                )],
                field_patches: vec![crate::transactions::data::AspectFieldPatch::default()],
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");

    let plan = txn
        .plan_bulk_mutation_batch(&runtime)
        .expect("planning succeeds")
        .expect("planned batch");
    let counters = runtime.performance_access().counters();

    assert_eq!(plan.locality.entity_target_count, 2);
    assert_eq!(plan.locality.relation_target_count, 1);
    assert_eq!(plan.locality.cross_partition_relation_count, 1);
    assert_eq!(plan.naming.normalized_client_keys.len(), 3);
    assert_eq!(plan.lineage.transitions.len(), 3);
    assert_eq!(plan.provenance.worker_batch_names.len(), 2);
    assert_eq!(counters.bulk_mutation_batch_count, 0);
    assert_eq!(counters.bulk_mutation_naming_normalization_count, 0);
    assert_eq!(counters.bulk_mutation_lineage_transition_count, 0);
    assert_eq!(counters.bulk_mutation_provenance_record_count, 0);
}

#[test]
fn complexity_budget_bulk_mutation_admission_remains_side_effect_free_until_commit() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");

    runtime.performance_access().reset_counters();
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("relation-batch").push(MutationIntent::Create(
            CreateIntent::BulkRelations(BulkRelationCreateIntent {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_keys: vec![crate::symbols::data::ClientKey::raw("edge")],
                endpoints: vec![(
                    crate::transactions::data::EntityReference::Existing(source),
                    crate::transactions::data::EntityReference::Existing(target),
                )],
                field_patches: vec![crate::transactions::data::AspectFieldPatch::default()],
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");

    let admitted = txn
        .admit_provenance_complete_bulk_mutation_batch(&runtime)
        .expect("admission should succeed");
    let preflight_counters = runtime.performance_access().counters();

    assert!(admitted.is_some());
    assert_eq!(preflight_counters.bulk_mutation_batch_count, 0);
    assert_eq!(
        preflight_counters.bulk_mutation_naming_normalization_count,
        0
    );
    assert_eq!(preflight_counters.bulk_mutation_lineage_transition_count, 0);
    assert_eq!(preflight_counters.bulk_mutation_provenance_record_count, 0);

    let mut commit_txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    commit_txn
        .push_batch(
            WorkerIntentBatch::new("relation-batch").push(MutationIntent::Create(
                CreateIntent::BulkRelations(BulkRelationCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_keys: vec![crate::symbols::data::ClientKey::raw("edge-commit")],
                    endpoints: vec![(
                        crate::transactions::data::EntityReference::Existing(source),
                        crate::transactions::data::EntityReference::Existing(target),
                    )],
                    field_patches: vec![crate::transactions::data::AspectFieldPatch::default()],
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let _ = commit_txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("commit should succeed");
    let committed_counters = runtime.performance_access().counters();

    assert_eq!(committed_counters.bulk_mutation_batch_count, 1);
    assert_eq!(committed_counters.bulk_mutation_relation_target_count, 1);
    assert_eq!(
        committed_counters.bulk_mutation_naming_normalization_count,
        1
    );
    assert_eq!(committed_counters.bulk_mutation_lineage_transition_count, 1);
    assert_eq!(committed_counters.bulk_mutation_provenance_record_count, 1);
}
