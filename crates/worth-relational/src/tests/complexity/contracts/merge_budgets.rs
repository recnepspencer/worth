use crate::facade::history::BranchId;
use crate::facade::merge::{MergeExecutionRequest, MergeIntent};
use crate::facade::transactions::{CreateIntent, MutationIntent, WorkerIntentBatch};
use crate::tests::support::{
    changed_entities, create_branch_from_main, create_entity, persisted_runtime_with_test_schema,
    update_entity, update_entity_on_branch,
};

#[test]
fn complexity_budget_merge_planning_reports_request_shaped_work() {
    let runtime = persisted_runtime_with_test_schema();
    let shared = create_entity(&runtime, "shared");
    create_branch_from_main(&runtime, "feature");
    update_entity(&runtime, shared, "main-value");
    update_entity_on_branch(
        &runtime,
        shared,
        "feature-value",
        BranchId("feature".to_string()),
    );

    runtime.performance_access().reset_counters();
    let artifact = runtime
        .merge()
        .inspect_planning_scope(crate::merge::data::MergePlanningRequest::new(
            BranchId("main".to_string()),
            BranchId("feature".to_string()),
            MergeIntent::ReconcileIntoTarget,
        ))
        .expect("merge planning artifact");

    let counters = runtime.performance_access().counters();
    assert!(runtime
        .performance_access()
        .contracts()
        .iter()
        .any(|contract| contract.id == "runtime.merge.planning"));
    assert_eq!(counters.merge_planning_requests, 1);
    assert!(counters.merge_planning_schema_kinds_snapshotted >= 1);
    assert!(counters.merge_planning_target_commits_scoped >= 1);
    assert!(counters.merge_planning_source_commits_scoped >= 1);
    assert!(counters.merge_planning_target_records_scoped >= 1);
    assert!(counters.merge_planning_source_records_scoped >= 1);
    assert_eq!(
        counters.merge_identity_candidates_discovered,
        artifact.identity_discovery.candidate_count
    );
    assert!(counters.merge_identity_target_records_scanned >= 1);
    assert!(counters.merge_identity_target_records_indexed >= 1);
    assert_eq!(
        counters.merge_conflict_records_classified,
        artifact.conflict_classification.classified_record_count
    );
    assert_eq!(
        counters.merge_causal_records_annotated,
        artifact.causal_annotation.classified_record_count
    );
    assert_eq!(
        counters.merge_policy_records_resolved,
        artifact.policy_resolution.resolved_record_count
    );
    assert_eq!(
        counters.merge_lowered_records_emitted,
        artifact.lowered_plan.record_count
    );
    assert_eq!(
        counters.merge_decision_log_width,
        artifact.decision_log.decisions.len()
    );
    assert!(counters.merge_planning_elapsed_nanos > 0);
}

#[test]
fn complexity_budget_merge_execution_reports_admitted_records_and_emitted_mutations() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity(&runtime, "main-anchor");
    create_branch_from_main(&runtime, "feature");
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_branch(
        &runtime,
        BranchId("feature".to_string()),
    );
    txn.push_batch(
        WorkerIntentBatch::new("create-feature-only").push(MutationIntent::Create(
            CreateIntent::Entity(crate::transactions::data::EntitySpec {
                partition_id: crate::facade::identity::PartitionId::main(),
                kind_id: crate::facade::identity::KindId(1),
                client_key: crate::symbols::data::ClientKey::raw("feature-only"),
                fields: crate::tests::support::single_string_aspect_field_patch(
                    crate::tests::support::aspect_key("name"),
                    crate::tests::support::field_key("name"),
                    "feature-only",
                ),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    let feature_only = changed_entities(&txn.commit(&runtime).expect("feature-only create"))[0];

    let prepared = runtime
        .prepare_merge_execution(MergeExecutionRequest {
            target_branch: BranchId("main".to_string()),
            source_branch: BranchId("feature".to_string()),
            merge_intent: MergeIntent::ReconcileIntoTarget,
        })
        .expect("prepared merge execution");

    runtime.performance_access().reset_counters();
    let outcome = runtime
        .execute_prepared_merge(prepared)
        .expect("executed merge");
    let counters = runtime.performance_access().counters();

    assert!(runtime
        .performance_access()
        .contracts()
        .iter()
        .any(|contract| contract.id == "runtime.merge.execution_commit"));
    assert_eq!(counters.merge_execution_attempts, 1);
    assert_eq!(
        counters.merge_execution_requests,
        outcome.structural_summary.executed_record_count
    );
    assert_eq!(
        counters.merge_execution_records_admitted,
        outcome.structural_summary.executed_record_count
    );
    assert_eq!(
        counters.merge_execution_mutation_intents_emitted,
        outcome.structural_summary.emitted_mutation_intent_count
    );
    assert_eq!(outcome.structural_summary.adopted_source_record_count, 1);
    assert_eq!(outcome.structural_summary.emitted_entity_create_count, 1);
    assert_eq!(changed_entities(&outcome.commit).len(), 1);
    assert_ne!(changed_entities(&outcome.commit)[0], feature_only);
}
