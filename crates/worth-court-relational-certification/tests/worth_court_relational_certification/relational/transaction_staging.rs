use worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request;

use worth_relational::facade::{
    history::BranchId,
    inspection::{
        GraphInspectionBudget, GraphInspectionRequest, HistoricalInspectionMode, InspectionScope,
        RecentCommitInspectionRequest,
    },
    transactions::{
        EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
    },
};
use worth_testkit_relational::{
    aspect_storage::publications::{begin_main_transaction, changed_entities},
    durable_publication::{batch_create, field_patch, memory_runtime},
    ordinary_records::create_entity,
};

#[test]
fn transaction_inspection_never_projects_hypothetical_committed_truth() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = memory_runtime();
    let baseline = runtime
        .inspect_what_happened()
        .graph_summary(execution, &current_graph_request())
        .expect("read request remains live");

    let mut txn = begin_main_transaction(&runtime);
    txn.push_batch(
        batch_create("pending"),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let staging = txn.inspect_staging();
    let during_staging = runtime
        .inspect_what_happened()
        .graph_summary(execution, &current_graph_request())
        .expect("read request remains live");

    assert_eq!(staging.batch_count, 1);
    assert_eq!(during_staging.entity_count, baseline.entity_count);
    assert_eq!(during_staging.relation_count, baseline.relation_count);
}

#[test]
fn transaction_inspection_savepoint_rollback_scrubs_abandoned_work_and_commit_truth() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = memory_runtime();
    let existing = create_entity(execution, &runtime, "existing");

    let mut txn = begin_main_transaction(&runtime);
    txn.push_batch(
        batch_create("kept"),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let savepoint = txn.create_savepoint().unwrap();
    txn.push_batch(
        WorkerIntentBatch::new("abandoned-update").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: existing,
                fields: field_patch("name", "name", "abandoned"),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    txn.push_batch(
        batch_create("abandoned"),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");

    let before_rollback = txn.inspect_staging();
    assert_eq!(before_rollback.batch_count, 3);
    assert_eq!(before_rollback.savepoints.len(), 1);
    assert!(before_rollback.touched_records.contains(
        &worth_relational::facade::transactions::RecordRef::Entity(existing)
    ));
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

    let committed = txn
        .commit(execution, &runtime)
        .expect("commit surviving staged work");
    let committed_entity = changed_entities(&committed)[0];
    let commit_inspection = runtime
        .inspect_what_happened()
        .inspect_commit(execution, committed.commit.commit_id)
        .unwrap()
        .expect("commit inspection");

    assert_eq!(
        commit_inspection.changed_records,
        vec![worth_relational::facade::transactions::RecordRef::Entity(
            committed_entity
        )]
    );
    assert!(!commit_inspection.changed_records.contains(
        &worth_relational::facade::transactions::RecordRef::Entity(existing)
    ));
}

#[test]
fn transaction_inspection_marks_lineage_affecting_intents_without_previewing_commit_or_history() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = memory_runtime();
    let entity = create_entity(execution, &runtime, "replace-target");
    let inspect_entity = || {
        let identity = runtime.main_branch_identity();
        let (_, basis) = runtime.observe_branch(&identity).unwrap();
        runtime
            .inspect_what_happened()
            .inspect_historical_record(
                execution,
                &BranchId("main".to_owned()),
                basis.observation().version_id(),
                worth_relational::facade::transactions::RecordRef::Entity(entity),
                HistoricalInspectionMode::RetainedOnly,
            )
            .expect("read request remains live")
            .record_observation
            .value
            .expect("the committed entity is available")
    };
    let baseline_entity = inspect_entity();
    let baseline_latest_commit = runtime
        .history()
        .historical_latest_commit()
        .map(|commit| commit.commit_id);
    let baseline_window = runtime
        .inspect_what_happened()
        .inspect_recent_commits(
            execution,
            &RecentCommitInspectionRequest {
                branch_id: Some(BranchId("main".to_string())),
                limit: 8,
            },
        )
        .unwrap();

    let mut txn = begin_main_transaction(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("replace").push(MutationIntent::Entity(
            EntityMutationIntent::Replace(
                worth_relational::facade::transactions::ReplaceEntityIntent {
                    entity_id: entity,
                    replacement: worth_relational::facade::transactions::EntitySpec {
                        partition_id: worth_relational::facade::identity::PartitionId::main(),
                        kind_id: worth_relational::facade::identity::KindId(1),
                        client_key: worth_relational::facade::symbols::ClientKey::raw(
                            "replacement",
                        ),
                        fields: field_patch("name", "name", "replacement"),
                    },
                },
            ),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");

    let staging = txn.inspect_staging();
    assert!(staging.contains_lineage_affecting_intents);
    assert_eq!(staging.intent_counts.entity_mutation_count, 1);
    assert_eq!(
        staging.touched_records,
        vec![worth_relational::facade::transactions::RecordRef::Entity(
            entity
        )]
    );

    let latest_commit_during_staging = runtime
        .history()
        .historical_latest_commit()
        .map(|commit| commit.commit_id);
    let window_during_staging = runtime
        .inspect_what_happened()
        .inspect_recent_commits(
            execution,
            &RecentCommitInspectionRequest {
                branch_id: Some(BranchId("main".to_string())),
                limit: 8,
            },
        )
        .unwrap();
    assert_eq!(latest_commit_during_staging, baseline_latest_commit);
    assert_eq!(window_during_staging, baseline_window);
    assert_eq!(inspect_entity(), baseline_entity);
}

#[test]
fn staging_inspection_counts_a_revalidation_demand_apart_from_the_mutations() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = memory_runtime();
    let rewritten = create_entity(execution, &runtime, "rewritten");
    let demanded = create_entity(execution, &runtime, "demanded");

    let mut txn = begin_main_transaction(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("rewrite-one-and-rejudge-another")
            .push(MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                UpdateEntityFieldsIntent {
                    entity_id: rewritten,
                    fields: field_patch("name", "name", "after"),
                },
            )))
            .push(MutationIntent::Entity(EntityMutationIntent::Revalidate(
                worth_relational::facade::transactions::RevalidateEntityIntent {
                    entity_id: demanded,
                },
            ))),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
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
        staging.touched_records.contains(
            &worth_relational::facade::transactions::RecordRef::Entity(demanded)
        ),
        "a demanded record is touched even though it is not mutated: {:?}",
        staging.touched_records
    );
}

fn current_graph_request() -> GraphInspectionRequest {
    GraphInspectionRequest {
        scope: InspectionScope::Current,
        partition_scope: None,
        relation_kind_scope: None,
        summary_only: true,
        budget: GraphInspectionBudget {
            max_entities: 1_024,
            max_relations: 1_024,
            max_work_units: 4_096,
        },
    }
}
