use crate::facade::history::BranchId;
use crate::facade::merge::{MergeIntent, MergePlanningRequest};
use crate::history::data::{MergeBaseSelectionRule, RelationalMergeBranchBasis};
use crate::tests::support::{
    create_branch_from_main, create_entity, create_entity_outcome_on_branch,
    runtime_with_test_schema,
};

#[test]
fn merge_branch_basis_matches_planning_artifact_and_history_scope_authority() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );
    let request = MergePlanningRequest::new(
        BranchId("main".to_string()),
        BranchId("feature".to_string()),
        MergeIntent::ReconcileIntoTarget,
    );

    let direct_basis = runtime
        .history()
        .resolve_merge_branch_basis(&request.source_branch, &request.target_branch)
        .expect("direct branch basis");
    let history_plan = runtime
        .merge()
        .plan_history_scope_for_test(execution, request.clone())
        .expect("history-scoped merge plan");
    let artifact = runtime
        .merge()
        .inspect_planning_scope(execution, request)
        .expect("planning artifact");

    assert_exact_branch_basis(&history_plan.basis, &direct_basis);
    assert_exact_branch_basis(&artifact.branch_basis, &direct_basis);
    assert_eq!(
        artifact.ancestry.merge_base_commit_id,
        artifact.branch_basis.merge_base().commit().commit_id
    );
    assert_eq!(
        artifact.branch_basis.merge_base().commit().commit_id,
        artifact.merge_base().commit().commit_id
    );
}

fn assert_exact_branch_basis(
    actual: &RelationalMergeBranchBasis,
    expected: &RelationalMergeBranchBasis,
) {
    assert_eq!(actual, expected);
    assert_eq!(actual.basis_digest(), expected.basis_digest());
    assert_eq!(actual.source_branch(), expected.source_branch());
    assert_eq!(actual.target_branch(), expected.target_branch());
    assert_eq!(actual.source_head(), expected.source_head());
    assert_eq!(actual.target_head(), expected.target_head());
    assert_eq!(
        actual.merge_base().rule(),
        MergeBaseSelectionRule::MaxCommitIdCommonAncestor
    );
    assert_eq!(actual.merge_base().rule(), expected.merge_base().rule());
    assert_eq!(actual.merge_base().commit(), expected.merge_base().commit());
    assert_eq!(
        actual.merge_base().supporting_left_ancestors(),
        expected.merge_base().supporting_left_ancestors()
    );
    assert_eq!(
        actual.merge_base().supporting_right_ancestors(),
        expected.merge_base().supporting_right_ancestors()
    );
}
