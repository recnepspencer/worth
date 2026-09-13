use worth_proof::TransitionOutcome;

use crate::branch::SignalBranchRetirementReason;

use super::super::super::tests::runtime_root::runtime_with_two_branches;
use super::super::super::SignalOwnerCancellationSource;
use super::receipt_oracle::{expected_closeout_digest, expected_terminal_basis_digest};

#[test]
fn plan_and_retire_exact_contact_target_once_each_and_preserve_receipt() {
    let (mut runtime, sibling, target, basis) = runtime_with_two_branches();
    let expected_terminal_digest = expected_terminal_basis_digest(&target, basis.observation());
    let expected_observation = basis.observation().clone();
    let expected_parent = target
        .parent_branch_id
        .expect("the real fork child has one parent");
    let expected_closeout = expected_closeout_digest(
        target.id,
        expected_parent,
        target.head_snapshot_id,
        target.head_snapshot_id,
        SignalBranchRetirementReason::DependencyCancellation,
        &expected_terminal_digest,
    );
    let (_, _, port) = runtime.owner_port_slots().expect("the real runtime seals");
    let owner = port.upgrade_owner().expect("the owner root remains live");
    let inspection = owner.admit().expect("cost setup admits");
    let sibling_cell = owner
        .lookup_cell(&inspection, sibling.id)
        .expect("the sibling cell is canonical");
    let target_cell = owner
        .lookup_cell(&inspection, target.id)
        .expect("the target cell is canonical");
    let sibling_before = sibling_cell.cost_snapshot();
    let target_before = target_cell.cost_snapshot();
    let owner_before = owner.cost_snapshot();
    let ledger_before = owner.retention_ledger_observation();
    let metadata_before = owner
        .metadata
        .retirement_contract_observation(&inspection, target.id)
        .expect("the planning contract is observable");
    let live_before = owner.live_count();
    drop(inspection);

    let plan = match port
        .plan_retirement_exact(basis, SignalBranchRetirementReason::DependencyCancellation)
    {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("the exact lifecycle plan must issue: {other:?}"),
    };
    let owner_after_plan = owner.cost_snapshot();
    let target_after_plan = target_cell.cost_snapshot();
    assert_eq!(plan.branch(), &target);
    assert_eq!(plan.admitted_basis().observation(), &expected_observation);
    assert_eq!(
        plan.terminal_basis_digest.as_str(),
        expected_terminal_digest
    );
    assert_eq!(plan.planned_child_membership_count(), 0);
    assert_eq!(owner.live_count(), live_before);
    assert_eq!(owner.retention_ledger_observation(), ledger_before);
    let after_inspection = owner.admit().expect("post-plan inspection admits");
    assert_eq!(
        owner
            .metadata
            .retirement_contract_observation(&after_inspection, target.id)
            .expect("planning leaves the contract observable"),
        metadata_before
    );
    drop(after_inspection);
    assert_eq!(sibling_cell.cost_snapshot(), sibling_before);
    assert_eq!(target_after_plan.contacts(), target_before.contacts() + 1);
    assert_eq!(target_after_plan.movements(), target_before.movements());
    assert_eq!(
        owner_after_plan.target_cell_contacts(),
        owner_before.target_cell_contacts() + 1
    );
    assert_eq!(
        owner_after_plan.retention_registry_contacts(),
        owner_before.retention_registry_contacts() + 1
    );
    assert_eq!(
        owner_after_plan.branch_registry_lookups(),
        owner_before.branch_registry_lookups() + 1
    );
    assert_eq!(owner_after_plan.branch_registry_entries_scanned(), 0);

    let receipt = match port.retire_exact(plan, &SignalOwnerCancellationSource::new().token()) {
        TransitionOutcome::Success(receipt) => receipt,
        other => panic!("the owner-issued lifecycle plan must execute: {other:?}"),
    };
    let owner_after_retirement = owner.cost_snapshot();
    let target_after_retirement = target_cell.cost_snapshot();
    assert_eq!(receipt.retired_branch(), &target);
    assert_eq!(receipt.parent_branch_id(), expected_parent);
    assert_eq!(
        receipt.reason(),
        SignalBranchRetirementReason::DependencyCancellation
    );
    assert_eq!(receipt.terminal_basis_digest(), expected_terminal_digest);
    assert_eq!(receipt.closeout_digest(), expected_closeout);
    assert_eq!(receipt.reclaimed_branch_state_count(), 1);
    assert_eq!(receipt.retained_proof_record_count(), 1);
    assert_eq!(sibling_cell.cost_snapshot(), sibling_before);
    assert_eq!(
        target_after_retirement.contacts(),
        target_after_plan.contacts() + 1
    );
    assert_eq!(
        target_after_retirement.movements(),
        target_after_plan.movements() + 1
    );
    assert_eq!(
        owner_after_retirement.target_cell_contacts(),
        owner_after_plan.target_cell_contacts() + 1
    );
    assert_eq!(
        owner_after_retirement.canonical_movements(),
        owner_after_plan.canonical_movements() + 1
    );
    assert_eq!(
        owner_after_retirement.retention_registry_contacts(),
        owner_after_plan.retention_registry_contacts() + 1
    );
    assert_eq!(owner.live_count(), live_before - 1);
}
