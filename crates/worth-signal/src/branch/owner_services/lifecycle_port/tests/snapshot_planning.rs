use worth_proof::TransitionOutcome;

use crate::branch::{SignalBranchRetirementDenial, SignalBranchRetirementReason};

use super::super::super::tests::runtime_root::runtime_with_two_branches;
use super::super::super::SignalOwnerCancellationSource;
use super::receipt_oracle::expected_terminal_basis_digest;

#[test]
fn snapshot_release_plan_counts_unique_owner_custody_and_executes_after_release() {
    let (mut runtime, _, target, initial_basis) = runtime_with_two_branches();
    let (first_snapshot, first_basis) = runtime
        .capture_signal_branch_snapshot(&initial_basis)
        .expect("the target issues its first snapshot")
        .into_parts();
    drop(initial_basis);
    let (second_snapshot, final_basis) = runtime
        .capture_signal_branch_snapshot(&first_basis)
        .expect("the target issues a distinct second snapshot")
        .into_parts();
    drop(first_basis);
    let final_target = runtime
        .branch_handle(target.id)
        .expect("the twice-captured target remains live");
    let expected_digest = expected_terminal_basis_digest(&final_target, final_basis.observation());
    let first_clone = first_snapshot.clone();
    assert_eq!(
        first_snapshot.retention_identity(),
        first_clone.retention_identity()
    );
    assert_ne!(
        first_snapshot.retention_identity(),
        second_snapshot.retention_identity()
    );
    let (_, _, port) = runtime
        .owner_port_slots()
        .expect("the snapshot fixture seals");
    let owner = port.upgrade_owner().expect("the owner remains live");
    let before = owner.cost_snapshot();
    let ledger_before = owner.retention_ledger_observation();
    assert_eq!(ledger_before.admitted_count_by_branch, vec![(target.id, 3)]);

    let plan = match port.plan_retirement_releasing_snapshots_exact(
        final_basis,
        &[&first_snapshot, &first_clone, &second_snapshot],
        SignalBranchRetirementReason::Superseded,
    ) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("unique owner custody must plan exactly: {other:?}"),
    };
    let after = owner.cost_snapshot();
    assert_eq!(plan.branch(), &final_target);
    assert_eq!(plan.terminal_basis_digest.as_str(), expected_digest);
    assert_eq!(owner.retention_ledger_observation(), ledger_before);
    assert_eq!(
        after.target_cell_contacts(),
        before.target_cell_contacts() + 1
    );
    assert_eq!(
        after.retention_registry_contacts(),
        before.retention_registry_contacts() + 1
    );

    drop(first_clone);
    drop(first_snapshot);
    drop(second_snapshot);
    assert_eq!(owner.admitted_retention_count(target.id), 1);
    let receipt = match port.retire_exact(plan, &SignalOwnerCancellationSource::new().token()) {
        TransitionOutcome::Success(receipt) => receipt,
        other => panic!("released snapshot custody must execute: {other:?}"),
    };
    assert_eq!(receipt.retired_branch(), &final_target);
    assert_eq!(receipt.terminal_basis_digest(), expected_digest);
}

#[test]
fn snapshot_release_plan_denies_foreign_runtime_before_registry_contact() {
    let (mut runtime, _, _, basis) = runtime_with_two_branches();
    let (_, _, port) = runtime
        .owner_port_slots()
        .expect("the receiving owner seals");
    let owner = port
        .upgrade_owner()
        .expect("the receiving owner remains live");
    let (mut foreign_runtime, _, _, foreign_basis) = runtime_with_two_branches();
    let (foreign_snapshot, refreshed_foreign_basis) = foreign_runtime
        .capture_signal_branch_snapshot(&foreign_basis)
        .expect("the foreign runtime issues real snapshot custody")
        .into_parts();
    drop(foreign_basis);
    drop(refreshed_foreign_basis);
    let before = owner.cost_snapshot();
    let expected_runtime_instance_id = owner.runtime_instance_id();
    let observed_runtime_instance_id = foreign_snapshot.owner_runtime_instance_id();

    assert!(matches!(
        port.plan_retirement_releasing_snapshots_exact(
            basis,
            &[&foreign_snapshot],
            SignalBranchRetirementReason::Rejected,
        ),
        TransitionOutcome::Denied(
            SignalBranchRetirementDenial::ForeignRetirementSnapshot {
                expected_runtime_instance_id: expected,
                observed_runtime_instance_id: observed,
            }
        ) if expected == expected_runtime_instance_id && observed == observed_runtime_instance_id
    ));
    assert_eq!(
        owner.cost_snapshot().branch_registry_lookups(),
        before.branch_registry_lookups(),
        "foreign snapshot custody denies before registry contact"
    );
    assert_eq!(
        owner.cost_snapshot().retention_registry_contacts(),
        before.retention_registry_contacts(),
        "foreign snapshot custody denies before retention contact"
    );
    assert_eq!(
        owner.cost_snapshot().target_cell_contacts(),
        before.target_cell_contacts(),
        "foreign snapshot custody denies before target contact"
    );
}

#[test]
fn snapshot_release_plan_denies_real_wrong_branch_custody() {
    let (mut runtime, selected, target, target_basis) = runtime_with_two_branches();
    let selected_basis = runtime
        .observe_signal_branch_basis(selected.clone())
        .expect("the selected branch admits");
    let (selected_snapshot, selected_after_capture) = runtime
        .capture_signal_branch_snapshot(&selected_basis)
        .expect("the selected branch issues real snapshot custody")
        .into_parts();
    drop(selected_basis);
    drop(selected_after_capture);
    let (_, _, port) = runtime.owner_port_slots().expect("the fixture seals");

    assert!(matches!(
        port.plan_retirement_releasing_snapshots_exact(
            target_basis,
            &[&selected_snapshot],
            SignalBranchRetirementReason::Rejected,
        ),
        TransitionOutcome::Denied(
            SignalBranchRetirementDenial::RetirementSnapshotBranchMismatch {
                branch_id,
                snapshot_branch_id,
            }
        ) if branch_id == target.id && snapshot_branch_id == selected.id
    ));
}
