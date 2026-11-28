use worth_proof::TransitionOutcome;

use crate::facade::branch::{
    AdmittedSignalBranchBasis, SignalBranchSnapshotCaptureDenial,
    SignalBranchSnapshotReconstructionDenial,
};
use crate::facade::*;

type TestRuntime = SignalRuntime<(), (), (), (), ()>;

fn runtime_with_budget(maximum_stored_snapshots: usize) -> TestRuntime {
    SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .maximum_stored_branch_snapshots(maximum_stored_snapshots)
        .build()
}

fn fork(runtime: &mut TestRuntime, name: &str, parent: SignalBranchId) -> SignalBranchHandle {
    match runtime.fork_branch(SignalBranchForkRequest::from_parent_branch_head(
        name, parent,
    )) {
        TransitionOutcome::Success(receipt) => receipt.created_branch().clone(),
        other => panic!("expected branch fork, got {other:?}"),
    }
}

fn observe(runtime: &TestRuntime, branch: SignalBranchHandle) -> AdmittedSignalBranchBasis {
    runtime
        .observe_signal_branch_basis(branch)
        .expect("branch should be observable")
}

#[test]
fn dropped_snapshot_authority_cannot_grow_live_storage_past_its_budget() {
    let mut runtime = runtime_with_budget(2);
    let branch = runtime.current_branch();
    let initial_basis = observe(&runtime, branch.clone());
    let (first_snapshot, first_basis) = runtime
        .capture_signal_branch_snapshot(&initial_basis)
        .expect("first budgeted snapshot should succeed")
        .into_parts();
    drop(initial_basis);
    drop(first_snapshot);
    let (second_snapshot, second_basis) = runtime
        .capture_signal_branch_snapshot(&first_basis)
        .expect("second budgeted snapshot should succeed")
        .into_parts();
    drop(first_basis);
    drop(second_snapshot);

    assert!(matches!(
        runtime.capture_signal_branch_snapshot(&second_basis),
        Err(
            SignalBranchSnapshotCaptureDenial::SnapshotCapacityExhausted {
                maximum_stored_snapshots: 2,
            }
        )
    ));
    let observed_after_denial = observe(&runtime, branch);
    assert_eq!(
        observed_after_denial.observation(),
        second_basis.observation()
    );
}

#[test]
fn branch_retirement_reclaims_snapshot_capacity_for_other_branches() {
    let mut runtime = runtime_with_budget(1);
    let canonical = runtime.current_branch();
    let branch = fork(&mut runtime, "capacity-owner", canonical.id);
    let pre_capture_basis = observe(&runtime, branch.clone());
    let (snapshot, branch_basis) = runtime
        .capture_signal_branch_snapshot(&pre_capture_basis)
        .expect("branch snapshot should fill the budget")
        .into_parts();
    drop(pre_capture_basis);
    drop(snapshot);

    let canonical_basis = observe(&runtime, canonical.clone());
    assert!(matches!(
        runtime.capture_signal_branch_snapshot(&canonical_basis),
        Err(SignalBranchSnapshotCaptureDenial::SnapshotCapacityExhausted { .. })
    ));
    drop(canonical_basis);

    let retirement = match runtime.plan_signal_branch_retirement(
        branch,
        branch_basis,
        SignalBranchRetirementReason::Rejected,
    ) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("snapshot-free branch should retire, got {other:?}"),
    };
    assert!(matches!(
        runtime.retire_signal_branch(retirement),
        TransitionOutcome::Success(_)
    ));

    let canonical_basis = observe(&runtime, canonical);
    runtime
        .capture_signal_branch_snapshot(&canonical_basis)
        .expect("retirement should reclaim one stored snapshot slot");
}

#[test]
fn pristine_reconstruction_denies_zero_budget_without_movement() {
    let mut source = runtime_with_budget(1);
    let source_basis = observe(&source, source.current_branch());
    let portable_snapshot = source
        .capture_signal_branch_snapshot(&source_basis)
        .expect("source capture should succeed")
        .into_parts()
        .0
        .into_snapshot();

    let mut target = runtime_with_budget(0);
    let target_branch = target.current_branch();
    let pristine_basis = observe(&target, target_branch.clone());
    assert!(matches!(
        target.reconstruct_signal_branch_snapshot(&pristine_basis, &portable_snapshot),
        Err(
            SignalBranchSnapshotReconstructionDenial::SnapshotCapacityExhausted {
                maximum_stored_snapshots: 0,
            }
        )
    ));
    let observed_after_denial = observe(&target, target_branch);
    assert_eq!(
        observed_after_denial.observation(),
        pristine_basis.observation()
    );
}
