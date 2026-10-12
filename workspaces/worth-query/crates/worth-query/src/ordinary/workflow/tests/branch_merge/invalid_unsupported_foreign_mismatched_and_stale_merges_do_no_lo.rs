//! invalid unsupported foreign mismatched and stale merges do no lower work.
use super::*;

#[test]
fn invalid_unsupported_foreign_mismatched_and_stale_merges_do_no_lower_work() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let invalid = declare_branch_merge("main", "main").expect_err("same branch must deny");
    assert_eq!(
        invalid.denial_kind(),
        WorthQueryBranchMergeDeclarationDenialKind::SameBranch
    );

    let unsupported = stateful_bridge_task_runtime(request_execution)
        .workspace("unsupported-branch-merge")
        .expect("workspace should open");
    let declaration = merge_declaration();
    let unsupported = branch_merge(request_execution, &unsupported, &declaration)
        .expect_err("missing relational authority must deny");
    assert_eq!(
        unsupported
            .counters()
            .lower_runtime_execution_attempt_count(),
        0
    );

    let left = stateful_bridge_task_runtime_with_merge(request_execution)
        .workspace("foreign-branch-merge-left")
        .expect("left should open");
    let foreign_context =
        branch_merge(request_execution, &left, &declaration).expect("left should admit");
    let mut right = stateful_bridge_task_runtime_with_merge(request_execution)
        .workspace("foreign-branch-merge-right")
        .expect("right should open");
    let foreign = declaration
        .clone()
        .using(foreign_context)
        .run(request_execution, &mut right);
    let foreign = foreign.stop().expect("foreign authority must stop");
    assert_eq!(
        foreign.source(),
        WorthQueryBranchMergeStopSource::ForeignAuthority
    );
    assert_eq!(
        foreign.counters().lower_runtime_execution_attempt_count(),
        0
    );

    let captured =
        branch_merge(request_execution, &right, &declaration).expect("context should admit");
    let reverse = declare_branch_merge("candidate", "main").expect("reverse should declare");
    let mismatch = reverse.using(captured).run(request_execution, &mut right);
    let mismatch = mismatch.stop().expect("mismatched context must stop");
    assert_eq!(
        mismatch.source(),
        WorthQueryBranchMergeStopSource::MismatchedContext
    );
    assert_eq!(
        mismatch.counters().lower_runtime_execution_attempt_count(),
        0
    );

    let first =
        branch_merge(request_execution, &right, &declaration).expect("first context should admit");
    let stale =
        branch_merge(request_execution, &right, &declaration).expect("second context should admit");
    let first = declaration
        .clone()
        .using(first)
        .run(request_execution, &mut right);
    assert!(first.completed().is_some(), "first merge should execute");
    let stale = declaration.using(stale).run(request_execution, &mut right);
    let stale = stale.stop().expect("retained context must stale-deny");
    assert_eq!(
        stale.source(),
        WorthQueryBranchMergeStopSource::StaleAuthority
    );
    assert_eq!(stale.counters().lower_runtime_execution_attempt_count(), 0);
}
