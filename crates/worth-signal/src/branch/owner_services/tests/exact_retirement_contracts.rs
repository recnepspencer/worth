use crate::branch::SignalBranchRetirementReason;
use worth_proof::TransitionOutcome;

use super::super::SignalOwnerCancellationSource;
use super::retirement_receipt_oracle::{
    expected_closeout_digest, expected_terminal_basis_digest,
    expected_terminal_basis_digest_at_generation,
};
use super::runtime_root::runtime_with_two_branches;

#[test]
fn exact_retirement_contract_consumes_a_linear_plan_before_registry_removal() {
    let (mut runtime, _, branch, basis) = runtime_with_two_branches();
    let expected_terminal_digest = expected_terminal_basis_digest(&branch, basis.observation());
    let wrong_terminal_digest = expected_terminal_basis_digest_at_generation(
        &branch,
        basis.observation(),
        basis.observation().generation().get() + 1,
    );
    let expected_parent = branch
        .parent_branch_id
        .expect("the retirement target is a fork child");
    let expected_closeout = expected_closeout_digest(
        branch.id,
        expected_parent,
        branch.head_snapshot_id,
        branch.head_snapshot_id,
        SignalBranchRetirementReason::Rejected,
        &expected_terminal_digest,
    );
    let wrong_closeout = expected_closeout_digest(
        branch.id,
        expected_parent,
        branch.head_snapshot_id,
        branch.head_snapshot_id,
        SignalBranchRetirementReason::Merged,
        &expected_terminal_digest,
    );
    let plan = match runtime.plan_signal_branch_retirement(
        branch.clone(),
        basis,
        SignalBranchRetirementReason::Rejected,
    ) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("retirement plan should be issued before sealing: {other:?}"),
    };
    let (_, _, lifecycle) = runtime.owner_port_slots().expect("runtime seals");
    let owner = lifecycle.upgrade_owner().expect("owner remains live");
    let admission = owner.admit().expect("retirement admits");
    let retirement = owner
        .reserve_retirement(&admission, branch.id)
        .expect("lineage, receipt, and registry capacity reserve pre-effect");
    let cancellation = SignalOwnerCancellationSource::new();
    let receipt = retirement
        .execute(plan, &cancellation.token())
        .expect("exact cell retirement performs into its reserved receipt");
    assert_eq!(receipt.retired_branch(), &branch);
    assert_eq!(receipt.reason(), SignalBranchRetirementReason::Rejected);
    assert_eq!(receipt.parent_branch_id(), expected_parent);
    assert_eq!(receipt.forked_from_snapshot_id(), branch.head_snapshot_id);
    assert_eq!(receipt.terminal_head_snapshot_id(), branch.head_snapshot_id);
    assert_eq!(receipt.terminal_basis_digest(), expected_terminal_digest);
    assert_eq!(receipt.closeout_digest(), expected_closeout);
    assert_ne!(receipt.terminal_basis_digest(), wrong_terminal_digest);
    assert_ne!(receipt.closeout_digest(), wrong_closeout);
    assert_eq!(receipt.reclaimed_branch_state_count(), 1);
    assert_eq!(receipt.reclaimed_snapshot_state_count(), 0);
    assert_eq!(receipt.reclaimed_runtime_meta_count(), 0);
    assert_eq!(receipt.retained_proof_record_count(), 1);
    assert_eq!(
        owner
            .metadata
            .retirement_receipt(&admission, branch.id)
            .expect("receipt recovery is owner-admitted"),
        Some(receipt)
    );
    assert_eq!(owner.live_count(), 1);
}

#[test]
fn performed_retirement_fault_recovers_exact_receipt_and_never_reopens_inert_cell() {
    let (mut runtime, _, branch, basis) = runtime_with_two_branches();
    let expected_terminal_digest = expected_terminal_basis_digest(&branch, basis.observation());
    let expected_parent = branch
        .parent_branch_id
        .expect("the retirement target is a fork child");
    let expected_closeout = expected_closeout_digest(
        branch.id,
        expected_parent,
        branch.head_snapshot_id,
        branch.head_snapshot_id,
        SignalBranchRetirementReason::Superseded,
        &expected_terminal_digest,
    );
    let plan = match runtime.plan_signal_branch_retirement(
        branch.clone(),
        basis,
        SignalBranchRetirementReason::Superseded,
    ) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("retirement plan should be issued before sealing: {other:?}"),
    };
    let (_, _, lifecycle) = runtime.owner_port_slots().expect("runtime seals");
    let owner = lifecycle.upgrade_owner().expect("owner remains live");
    let admission = owner.admit().expect("retirement admits");
    let retirement = owner
        .reserve_retirement(&admission, branch.id)
        .expect("receipt capacity reserves before movement");
    let cancellation = SignalOwnerCancellationSource::new();

    let injected = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = retirement.execute_with_post_movement_fault(plan, &cancellation.token());
    }));
    assert!(
        injected.is_err(),
        "the disputed post-movement boundary is reached"
    );
    let receipt = owner
        .metadata
        .retirement_receipt(&admission, branch.id)
        .expect("follow-up recovery is owner-admitted")
        .expect("performed movement retains its exact preconstructed receipt");
    assert_eq!(receipt.retired_branch(), &branch);
    assert_eq!(receipt.reason(), SignalBranchRetirementReason::Superseded);
    assert_eq!(receipt.parent_branch_id(), expected_parent);
    assert_eq!(receipt.forked_from_snapshot_id(), branch.head_snapshot_id);
    assert_eq!(receipt.terminal_head_snapshot_id(), branch.head_snapshot_id);
    assert_eq!(receipt.terminal_basis_digest(), expected_terminal_digest);
    assert_eq!(receipt.closeout_digest(), expected_closeout);
    assert_eq!(receipt.reclaimed_branch_state_count(), 1);
    assert_eq!(receipt.reclaimed_snapshot_state_count(), 0);
    assert_eq!(receipt.reclaimed_runtime_meta_count(), 0);
    assert_eq!(receipt.retained_proof_record_count(), 1);
    assert_eq!(owner.live_count(), 1);
    assert!(matches!(
        owner.lookup_cell(&admission, branch.id),
        Err(super::super::SignalBranchRegistryDenial::UnknownBranch(id)) if id == branch.id
    ));
}
