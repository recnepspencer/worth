//! Concurrent duplicate forks through cloned fork ports.
use super::*;

#[test]
fn concurrent_duplicate_fork_has_one_winner_and_no_second_target() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity(execution, &runtime, "phase2-duplicate-fork-anchor");
    let source_branch = BranchId("main".to_owned());
    let port = runtime.fork_port();
    let (_, first_source) = port
        .observe_fork_source(&source_branch)
        .expect("first exact source");
    let (_, second_source) = port
        .observe_fork_source(&source_branch)
        .expect("second exact source");
    let branch_count_before = runtime.history().branch_cells_snapshot().len();
    let start = Arc::new(Barrier::new(3));

    let first_port = port.clone();
    let first_start = Arc::clone(&start);
    let first = std::thread::spawn(move || {
        first_start.wait();
        let host_request = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);
        first_port.fork_branch(
            execution,
            BranchId("duplicate-target".to_owned()),
            first_source,
        )
    });
    let second_port = port.clone();
    let second_start = Arc::clone(&start);
    let second = std::thread::spawn(move || {
        second_start.wait();
        let host_request = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);
        second_port.fork_branch(
            execution,
            BranchId("duplicate-target".to_owned()),
            second_source,
        )
    });
    start.wait();

    let outcomes = [
        first.join().expect("first fork worker joins"),
        second.join().expect("second fork worker joins"),
    ];
    assert_eq!(outcomes.iter().filter(|outcome| outcome.is_ok()).count(), 1);
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(
                outcome,
                Err(crate::branch::RelationalBranchTransitionStop::Denied(
                    crate::branch::RelationalForkDenial::DuplicateTarget
                ))
            ))
            .count(),
        1,
    );
    assert_eq!(
        runtime.history().branch_cells_snapshot().len(),
        branch_count_before + 1,
        "the losing reservation leaves no second branch cell",
    );
    let source_root = runtime
        .history
        .branch_cell(&source_branch)
        .and_then(|cell| cell.root())
        .expect("source root");
    let target_root = runtime
        .history
        .branch_cell(&BranchId("duplicate-target".to_owned()))
        .and_then(|cell| cell.root())
        .expect("winning target root");
    assert!(Arc::ptr_eq(&source_root, &target_root));
}
