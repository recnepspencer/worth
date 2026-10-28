use crate::facade::history::BranchCreateErrorClass;
use crate::tests::support::*;

#[test]
fn branch_creation_and_branch_targeted_commits_build_a_version_graph() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let main_outcome = create_entity_outcome(execution, &runtime, "main-a");
    runtime
        .history_authority()
        .fork_branch_from(
            execution,
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let feature_outcome = create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-a",
        BranchId("feature".to_string()),
    );
    let main_second = create_entity_outcome_on_branch(
        execution,
        &runtime,
        "main-b",
        BranchId("main".to_string()),
    );
    let branch_cells = runtime.history().branch_cells_snapshot();

    assert_eq!(
        runtime
            .history()
            .branch_head(&BranchId("feature".to_string()))
            .unwrap(),
        feature_outcome.commit
    );
    assert_eq!(
        runtime
            .history()
            .branch_head(&BranchId("main".to_string()))
            .unwrap(),
        main_second.commit
    );
    assert_eq!(
        feature_outcome.commit.parents,
        vec![main_outcome.commit.commit_id]
    );
    assert_eq!(
        main_second.commit.parents,
        vec![main_outcome.commit.commit_id]
    );
    assert_eq!(branch_cells.len(), 2);
    assert_eq!(runtime.history().immutable_commit_count(), 3);
}

#[test]
fn branch_history_helpers_expose_ancestor_and_merge_base_reasoning() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let main = create_entity_outcome(execution, &runtime, "main");
    runtime
        .history_authority()
        .fork_branch_from(
            execution,
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let feature = create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature",
        BranchId("feature".to_string()),
    );
    let chain = runtime
        .history()
        .ancestor_closure_by_commit_id_order(feature.commit.commit_id);
    let merge_base = runtime.history().latest_common_ancestor_between_branches(
        &BranchId("main".to_string()),
        &BranchId("feature".to_string()),
    );

    assert_eq!(chain, vec![main.commit.commit_id, feature.commit.commit_id]);
    assert_eq!(merge_base, Some(main.commit.commit_id));
    assert!(runtime.history().can_merge_branch_into(
        &BranchId("feature".to_string()),
        &BranchId("main".to_string())
    ));
}

#[test]
fn duplicate_branch_creation_is_rejected() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "main-seed");
    runtime
        .history_authority()
        .fork_branch_from(
            execution,
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let error = runtime
        .history_authority()
        .fork_branch_from(
            execution,
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap_err();

    assert_eq!(error.class, BranchCreateErrorClass::BranchAlreadyExists);
}
