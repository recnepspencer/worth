//! branch pair diff executes both real bases and reports added rows.
use super::*;

#[test]
fn branch_pair_diff_executes_both_real_bases_and_reports_added_rows() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = declare(task_collection_read)
        .expect("comparison declaration should build")
        .diff();
    let mut left = stateful_bridge_task_runtime(request_execution)
        .workspace("ordinary-comparison-left-branch")
        .expect("left workspace should open");
    let mut right = stateful_bridge_task_runtime(request_execution)
        .workspace("ordinary-comparison-right-branch")
        .expect("right workspace should open");
    write_task(request_execution, &mut left, "shared");
    write_task(request_execution, &mut right, "shared");
    write_task(request_execution, &mut right, "right-only");

    let context = between(
        request_execution,
        &left,
        test_session_label("comparison-left-branch"),
        &right,
        test_session_label("comparison-right-branch"),
    )
    .expect("branch bases should admit");
    let outcome = declaration
        .using(context)
        .run(request_execution, (&mut left, &mut right));
    let completion = outcome
        .completed()
        .expect("branch comparison should complete");

    assert_eq!(completion.change(), WorthQueryComparisonChange::Changed);
    assert_eq!(
        completion.basis_pair().family(),
        WorthQueryComparisonBasisFamily::BranchToBranch
    );
    assert_ne!(
        completion.basis_pair().left().branch_admission_identity(),
        completion.basis_pair().right().branch_admission_identity()
    );
    assert_eq!(
        completion.left().receipt().snapshot_identity(),
        completion.basis_pair().left().snapshot()
    );
    assert_eq!(
        completion.right().receipt().snapshot_identity(),
        completion.basis_pair().right().snapshot()
    );
    assert_eq!(completion.row_changes().len(), 1);
    assert!(completion.row_changes().iter().any(|change| {
        change.family() == WorthQueryComparisonRowChangeFamily::Added
            && change.left().is_none()
            && change.right().is_some()
    }));
    assert_eq!(
        completion.journey_counters().left_execution_attempt_count(),
        1
    );
    assert_eq!(
        completion
            .journey_counters()
            .right_execution_attempt_count(),
        1
    );
    assert_eq!(
        completion.journey_counters().left_row_scan_count(),
        completion.left().rows().len()
    );
    assert_eq!(
        completion.journey_counters().right_row_scan_count(),
        completion.right().rows().len()
    );
    assert_eq!(
        completion.journey_counters().emitted_row_change_count(),
        completion.row_changes().len()
    );
}
