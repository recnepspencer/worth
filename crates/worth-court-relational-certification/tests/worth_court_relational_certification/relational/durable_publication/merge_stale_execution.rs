//! Executing a prepared merge is refused once either actual branch head has advanced.
use super::branch_history_publications::{create_entity, create_entity_outcome_on_branch};
use super::merge_prepared_verification::prepare_feature_merge;
use super::merge_proof_journey::merge_ready_runtime;
use worth_relational::facade::{history::BranchId, merge::MergeExecutionError};
use worth_testkit_relational::durable_medium::TestDurableMedium;

#[test]
fn authoritative_merge_execution_certification_rejects_stale_prepared_merge_after_target_advances()
{
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = merge_ready_runtime(execution, &medium);

    let prepared = prepare_feature_merge(execution, &runtime);
    create_entity(execution, &runtime, "main-advance");

    match runtime.execute_prepared_merge(execution, prepared) {
        Err(MergeExecutionError::StaleBranchHead { branch, .. }) => {
            assert_eq!(branch, BranchId("main".to_string()));
        }
        other => panic!("expected target stale-head rejection, got {other:?}"),
    }
}

#[test]
fn authoritative_merge_execution_certification_rejects_stale_prepared_merge_after_source_advances()
{
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = merge_ready_runtime(execution, &medium);

    let prepared = prepare_feature_merge(execution, &runtime);
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-advance",
        BranchId("feature".to_string()),
    );

    match runtime.execute_prepared_merge(execution, prepared) {
        Err(MergeExecutionError::StaleBranchHead { branch, .. }) => {
            assert_eq!(branch, BranchId("feature".to_string()));
        }
        other => panic!("expected source stale-head rejection, got {other:?}"),
    }
}
