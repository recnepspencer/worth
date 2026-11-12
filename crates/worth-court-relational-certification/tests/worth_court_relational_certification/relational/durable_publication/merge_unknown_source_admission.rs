//! Typed owner-binding refusal of a merge execution request naming an unknown source branch.
use super::branch_history_publications::{create_entity_outcome_on_branch, fork};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::merge::{
    MergeExecutionRequest, MergeIntent, RelationalMergeRequestBindingDenial,
};
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::{create_entity_outcome, durable_runtime};

#[test]
fn unknown_merge_source_refuses_while_a_valid_preparation_remains_held() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity_outcome(execution, &runtime, "root");
    fork(execution, &runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );

    let _prepared = runtime
        .prepare_merge_execution(
            execution,
            runtime
                .bind_merge_execution_request(MergeExecutionRequest::new(
                    BranchId("main".to_string()),
                    BranchId("feature".to_string()),
                    MergeIntent::ReconcileIntoTarget,
                ))
                .expect("known branches bind to their owner cells"),
        )
        .expect("prepared merge execution");

    let denial = runtime
        .bind_merge_execution_request(MergeExecutionRequest::new(
            BranchId("main".to_string()),
            BranchId("wrong".to_string()),
            MergeIntent::ReconcileIntoTarget,
        ))
        .expect_err("unknown source must refuse even with another valid preparation held");

    match denial {
        RelationalMergeRequestBindingDenial::UnknownBranch(branch_id) => {
            assert_eq!(branch_id, BranchId("wrong".to_string()))
        }
        other => panic!("expected the unknown source branch refusal, got {other:?}"),
    }
}
