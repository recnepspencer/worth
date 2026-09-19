use super::*;

#[test]
fn denied_worker_retirement_preserves_snapshot_authority() {
    let mut shell = worker_shell_with_counter_graph();
    let main = shell.current_branch();
    let main_basis = shell.worker_branch_basis(main.id.0).unwrap();
    let parent = shell
        .fork_worker_branch(WorkerForkBranchRequest {
            name: "snapshot-parent".to_owned(),
            parent_branch_id: main.id.0,
            expected_parent_basis: main_basis,
        })
        .unwrap();
    let retained_snapshot = shell.branch_snapshot(parent.branch.id.0).unwrap();
    let parent_basis = shell.worker_branch_basis(parent.branch.id.0).unwrap();
    shell
        .fork_worker_branch(WorkerForkBranchRequest {
            name: "live-child".to_owned(),
            parent_branch_id: parent.branch.id.0,
            expected_parent_basis: parent_basis.clone(),
        })
        .unwrap();

    let denial = shell
        .retire_worker_branch(WorkerRetireBranchRequest {
            branch_id: parent.branch.id.0,
            expected_basis: parent_basis,
            reason: WorkerBranchRetirementReason::Rejected,
        })
        .unwrap_err();

    assert!(denial.message.contains("LiveChildren"));
    shell
        .restore_branch_snapshot(parent.branch.id.0, retained_snapshot)
        .expect("denied retirement must preserve snapshot authority");
}

#[test]
fn worker_batch_retirement_releases_owned_snapshots_after_planning() {
    let mut shell = worker_shell_with_counter_graph();
    let main = shell.current_branch();
    let main_basis = shell.worker_branch_basis(main.id.0).unwrap();
    let parent = shell
        .fork_worker_branch(WorkerForkBranchRequest {
            name: "batch-parent".to_owned(),
            parent_branch_id: main.id.0,
            expected_parent_basis: main_basis,
        })
        .unwrap();
    let child = shell
        .fork_worker_branch(WorkerForkBranchRequest {
            name: "batch-child".to_owned(),
            parent_branch_id: parent.branch.id.0,
            expected_parent_basis: parent.created_basis,
        })
        .unwrap();
    shell.branch_snapshot(parent.branch.id.0).unwrap();
    shell.branch_snapshot(child.branch.id.0).unwrap();

    let receipt = shell
        .retire_worker_branches(WorkerRetireBranchesRequest {
            retirements: vec![
                WorkerRetireBranchRequest {
                    branch_id: child.branch.id.0,
                    expected_basis: shell.worker_branch_basis(child.branch.id.0).unwrap(),
                    reason: WorkerBranchRetirementReason::Rejected,
                },
                WorkerRetireBranchRequest {
                    branch_id: parent.branch.id.0,
                    expected_basis: shell.worker_branch_basis(parent.branch.id.0).unwrap(),
                    reason: WorkerBranchRetirementReason::DependencyCancellation,
                },
            ],
        })
        .unwrap();

    assert_eq!(receipt.retirements.len(), 2);
    assert_eq!(shell.branches(), vec![main]);
}
