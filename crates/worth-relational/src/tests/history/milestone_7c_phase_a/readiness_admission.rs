use crate::facade::history::BranchId;
use crate::facade::merge::{MergeExecutionRequest, MergeIntent};
use crate::tests::support::{
    create_branch_from_main, create_entity, create_entity_outcome_on_branch,
};

#[test]
fn prepare_merge_execution_admits_fully_ready_source_only_addition() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = crate::tests::support::runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );

    let request = MergeExecutionRequest {
        target_branch: BranchId("main".to_string()),
        source_branch: BranchId("feature".to_string()),
        merge_intent: MergeIntent::ReconcileIntoTarget,
    };

    let prepared = runtime
        .merge()
        .prepare_merge_execution(execution, request.clone())
        .expect("merge execution should prepare");
    let normalized_request = runtime
        .merge()
        .normalize_merge_request(request.clone())
        .expect("normalized request");

    assert_eq!(prepared.request(), &normalized_request);
    assert!(prepared.artifact().lowered_plan.fully_execution_ready);
    assert_eq!(prepared.artifact().lowered_plan.blocked_count, 0);
    assert_eq!(prepared.artifact().lowered_plan.rejected_count, 0);
    assert!(prepared
        .execution_ready_plan()
        .lowered_records
        .iter()
        .all(|record| matches!(
            record.record_decision,
            crate::merge::data::LoweredRecordDecision::Execute(_)
        )));
}
