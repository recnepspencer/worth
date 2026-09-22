use super::*;

#[test]
fn transaction_inspection_never_projects_hypothetical_committed_truth() {
    let runtime = runtime_with_test_schema();
    let baseline = runtime
        .inspect_what_happened()
        .graph_summary(&current_graph_request(None, None, true));

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(batch_create("pending"))
        .expect("test staging stays within configured resource budgets");
    let staging = txn.inspect_staging();
    let during_staging = runtime
        .inspect_what_happened()
        .graph_summary(&current_graph_request(None, None, true));

    assert_eq!(staging.batch_count, 1);
    assert_eq!(during_staging.entity_count, baseline.entity_count);
    assert_eq!(during_staging.relation_count, baseline.relation_count);
}

#[test]
fn transaction_inspection_savepoint_rollback_scrubs_abandoned_work_and_commit_truth() {
    let runtime = runtime_with_test_schema();
    let existing = create_entity(&runtime, "existing");

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(batch_create("kept"))
        .expect("test staging stays within configured resource budgets");
    let savepoint = txn.create_savepoint().unwrap();
    txn.push_batch(
        WorkerIntentBatch::new("abandoned-update").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: existing,
                fields: crate::tests::support::single_string_aspect_field_patch(
                    crate::tests::support::aspect_key("name"),
                    crate::tests::support::field_key("name"),
                    "abandoned",
                ),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    txn.push_batch(batch_create("abandoned"))
        .expect("test staging stays within configured resource budgets");

    let before_rollback = txn.inspect_staging();
    assert_eq!(before_rollback.batch_count, 3);
    assert_eq!(before_rollback.savepoints.len(), 1);
    assert!(before_rollback
        .touched_records
        .contains(&crate::facade::transactions::RecordRef::Entity(existing)));
    assert_eq!(before_rollback.intent_counts.create_count, 2);
    assert_eq!(before_rollback.intent_counts.entity_mutation_count, 1);

    txn.rollback_to_savepoint(savepoint)
        .expect("rollback to savepoint");

    let after_rollback = txn.inspect_staging();
    assert_eq!(after_rollback.batch_count, 1);
    assert!(after_rollback.savepoints.is_empty());
    assert!(after_rollback.touched_records.is_empty());
    assert_eq!(after_rollback.intent_counts.create_count, 1);
    assert_eq!(after_rollback.intent_counts.entity_mutation_count, 0);

    let committed = txn.commit(&runtime).expect("commit surviving staged work");
    let committed_entity = changed_entities(&committed)[0];
    let commit_inspection = runtime
        .inspect_what_happened()
        .inspect_commit(committed.commit.commit_id)
        .expect("commit inspection");

    assert_eq!(
        commit_inspection.changed_records,
        vec![crate::facade::transactions::RecordRef::Entity(
            committed_entity
        )]
    );
    assert!(!commit_inspection
        .changed_records
        .contains(&crate::facade::transactions::RecordRef::Entity(existing)));
}

#[test]
fn transaction_inspection_marks_lineage_affecting_intents_without_previewing_commit_or_history() {
    let runtime = runtime_with_test_schema();
    let entity = create_entity(&runtime, "replace-target");
    let baseline_latest_commit = runtime
        .history()
        .latest_commit()
        .map(|commit| commit.commit_id);
    let baseline_window =
        runtime
            .inspect_what_happened()
            .inspect_recent_commits(&RecentCommitInspectionRequest {
                branch_id: Some(BranchId("main".to_string())),
                limit: 8,
            });

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("replace").push(MutationIntent::Entity(
            EntityMutationIntent::Replace(crate::transactions::data::ReplaceEntityIntent {
                entity_id: entity,
                replacement: crate::transactions::data::EntitySpec {
                    partition_id: crate::facade::identity::PartitionId::main(),
                    kind_id: crate::facade::identity::KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw("replacement"),
                    fields: crate::tests::support::single_string_aspect_field_patch(
                        crate::tests::support::aspect_key("name"),
                        crate::tests::support::field_key("name"),
                        "replacement",
                    ),
                },
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");

    let staging = txn.inspect_staging();
    assert!(staging.contains_lineage_affecting_intents);
    assert_eq!(staging.intent_counts.entity_mutation_count, 1);
    assert_eq!(
        staging.touched_records,
        vec![crate::facade::transactions::RecordRef::Entity(entity)]
    );

    let latest_commit_during_staging = runtime
        .history()
        .latest_commit()
        .map(|commit| commit.commit_id);
    let window_during_staging =
        runtime
            .inspect_what_happened()
            .inspect_recent_commits(&RecentCommitInspectionRequest {
                branch_id: Some(BranchId("main".to_string())),
                limit: 8,
            });
    let current = retained_record_inspection(
        &runtime,
        &BranchId("main".to_string()),
        runtime.current_version_id(),
        crate::facade::transactions::RecordRef::Entity(entity),
    );

    assert_eq!(latest_commit_during_staging, baseline_latest_commit);
    assert_eq!(window_during_staging, baseline_window);
    let current_name = match current.record_observation.value {
        Some(crate::facade::inspection::HistoricalRecordValue::Entity(ref record)) => {
            read_entity_name(record)
        }
        _ => None,
    };
    assert_eq!(current_name, Some("replace-target".into()));
}

#[test]
fn staging_inspection_counts_a_revalidation_demand_apart_from_the_mutations() {
    let runtime = runtime_with_test_schema();
    let rewritten = create_entity(&runtime, "rewritten");
    let demanded = create_entity(&runtime, "demanded");

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("rewrite-one-and-rejudge-another")
            .push(MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                UpdateEntityFieldsIntent {
                    entity_id: rewritten,
                    fields: crate::tests::support::single_string_aspect_field_patch(
                        crate::tests::support::aspect_key("name"),
                        crate::tests::support::field_key("name"),
                        "after",
                    ),
                },
            )))
            .push(MutationIntent::Entity(EntityMutationIntent::Revalidate(
                crate::transactions::data::RevalidateEntityIntent {
                    entity_id: demanded,
                },
            ))),
    )
    .expect("test staging stays within configured resource budgets");

    let staging = txn.inspect_staging();

    assert_eq!(
        staging.intent_counts.entity_mutation_count, 1,
        "one record is about to change, so the mutation count must name one record and not two"
    );
    assert_eq!(
        staging.intent_counts.entity_revalidation_count, 1,
        "the demand is carried and must be visible, not silently absent from the surface"
    );
    assert!(
        staging
            .touched_records
            .contains(&crate::facade::transactions::RecordRef::Entity(demanded)),
        "a demanded record is touched even though it is not mutated: {:?}",
        staging.touched_records
    );
}
