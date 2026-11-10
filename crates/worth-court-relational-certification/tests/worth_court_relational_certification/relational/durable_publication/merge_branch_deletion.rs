//! Branch-local entity deletion: mutual-deletion classification and stale-delete denial.
use super::branch_history_publications::{
    create_entity, delete_entity, delete_entity_on_branch, fork, transaction,
};
use super::merge_proof_journey::merge_request;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::merge::{
    DeletionExecutionClass, MergeExecutableClass, MergeResolutionClass,
};
use worth_relational::facade::transactions::{
    DeleteEntityIntent, EntityMutationIntent, MutationIntent, TransactionCommitError,
    WorkerIntentBatch,
};
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::durable_runtime;

#[test]
fn real_feature_branch_delete_after_main_delete_is_authorable_and_classifies_as_deleted_on_both_sides(
) {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let entity = create_entity(execution, &runtime, "shared");
    fork(execution, &runtime, "feature");
    delete_entity(execution, &runtime, entity);
    delete_entity_on_branch(execution, &runtime, entity, BranchId("feature".to_string()));

    let artifact = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            runtime
                .bind_merge_planning_request(merge_request().into())
                .unwrap(),
        )
        .expect("planning artifact");

    let lowered = artifact
        .lowered_plan
        .records
        .iter()
        .find(|record| {
            record.resolution_class
                == MergeResolutionClass::Deletion(DeletionExecutionClass::DeletedOnBothSides)
        })
        .expect("lowered record");

    assert_eq!(
        lowered.resolution_class,
        MergeResolutionClass::Deletion(DeletionExecutionClass::DeletedOnBothSides)
    );
    assert_eq!(
        lowered.executable_class,
        Some(MergeExecutableClass::ConvergeDeletedOnBothSides)
    );
}

#[test]
fn branch_local_delete_allowance_does_not_make_same_branch_stale_delete_legal() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let entity = create_entity(execution, &runtime, "shared");
    delete_entity(execution, &runtime, entity);
    let mut txn = transaction(&runtime, &BranchId("main".to_string()));
    txn.push_batch(
        WorkerIntentBatch::new("stale-delete").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: entity }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");

    match txn.commit(execution, &runtime) {
        Err(TransactionCommitError::Conflict { error, .. }) => {
            assert_eq!(
                error.code(),
                worth_relational::facade::diagnostics::DiagnosticCode::StaleHandle
            );
        }
        other => panic!("expected stale delete rejection, got {other:?}"),
    }
}
