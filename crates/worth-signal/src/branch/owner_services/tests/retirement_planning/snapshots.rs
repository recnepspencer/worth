use worth_proof::TransitionOutcome;

use crate::branch::{SignalBranchRetirementDenial, SignalBranchRetirementReason};

use super::super::super::SignalOwnerCancellationSource;
use super::super::retirement_receipt_oracle::expected_terminal_basis_digest;
use super::{populated_runtime_with_two_branches, seal_populated_target};

#[test]
fn owner_snapshot_release_plan_counts_unique_owner_issued_custody_exactly() {
    let (mut runtime, _, target, initial_basis) = populated_runtime_with_two_branches();
    let (first_snapshot, first_basis) = runtime
        .capture_signal_branch_snapshot(&initial_basis)
        .expect("the populated target produces its first owner-issued snapshot")
        .into_parts();
    drop(initial_basis);
    let (second_snapshot, final_basis) = runtime
        .capture_signal_branch_snapshot(&first_basis)
        .expect("the populated target produces a distinct second snapshot")
        .into_parts();
    drop(first_basis);
    let final_target = runtime
        .branch_handle(target.id)
        .expect("the twice-captured target remains live");
    let expected_digest = expected_terminal_basis_digest(&final_target, final_basis.observation());
    let first_snapshot_clone = first_snapshot.clone();
    assert_eq!(
        first_snapshot.retention_identity(),
        first_snapshot_clone.retention_identity(),
        "snapshot clones share one owner retention identity"
    );
    assert_ne!(
        first_snapshot.retention_identity(),
        second_snapshot.retention_identity(),
        "separate captures retain separate owner identities"
    );

    let (port, _, _) = runtime
        .owner_port_slots()
        .expect("the populated snapshot fixture seals");
    let owner = port.upgrade_owner().expect("the snapshot owner upgrades");
    let admission = owner
        .admit()
        .expect("snapshot-release planning admits once");
    let before = owner.retention_ledger_observation();
    assert_eq!(before.admitted_count_by_branch, vec![(target.id, 3)]);
    let plan = match owner.plan_retirement_releasing_snapshots_exact(
        &admission,
        final_basis,
        &[&first_snapshot, &first_snapshot_clone, &second_snapshot],
        SignalBranchRetirementReason::Superseded,
    ) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("unique snapshot custody should plan exactly: {other:?}"),
    };
    assert_eq!(plan.branch(), &final_target);
    assert_eq!(plan.terminal_basis_digest.as_str(), expected_digest);
    assert_eq!(owner.retention_ledger_observation(), before);

    drop(first_snapshot_clone);
    drop(first_snapshot);
    drop(second_snapshot);
    assert_eq!(owner.admitted_retention_count(target.id), 1);
    let reservation = owner
        .reserve_retirement(&admission, target.id)
        .expect("declared snapshot custody is released before execution");
    let cancellation = SignalOwnerCancellationSource::new();
    let receipt = reservation
        .execute(plan, &cancellation.token())
        .expect("the snapshot-aware owner plan executes canonically");
    assert_eq!(receipt.retired_branch(), &final_target);
    assert_eq!(receipt.terminal_basis_digest(), expected_digest);
}

#[test]
fn owner_snapshot_release_plan_denies_foreign_runtime_before_registry_contact() {
    let (_runtime, owner, _, _, basis) = seal_populated_target();
    let (mut foreign_runtime, _, _, foreign_basis) = populated_runtime_with_two_branches();
    let (foreign_snapshot, refreshed_foreign_basis) = foreign_runtime
        .capture_signal_branch_snapshot(&foreign_basis)
        .expect("the foreign runtime issues real snapshot custody")
        .into_parts();
    drop(foreign_basis);
    drop(refreshed_foreign_basis);

    let admission = owner.admit().expect("the receiving owner admits");
    let before = owner.cost_snapshot();
    let expected_runtime_instance_id = owner.runtime_instance_id();
    let observed_runtime_instance_id = foreign_snapshot.owner_runtime_instance_id();
    assert!(matches!(
        owner.plan_retirement_releasing_snapshots_exact(
            &admission,
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
}

#[test]
fn owner_snapshot_release_plan_denies_real_wrong_branch_custody() {
    let (mut runtime, selected, target, target_basis) = populated_runtime_with_two_branches();
    let selected_basis = runtime
        .observe_signal_branch_basis(selected.clone())
        .expect("the populated selected branch admits");
    let (selected_snapshot, selected_after_capture) = runtime
        .capture_signal_branch_snapshot(&selected_basis)
        .expect("the selected branch issues real snapshot custody")
        .into_parts();
    drop(selected_basis);
    drop(selected_after_capture);
    let (port, _, _) = runtime
        .owner_port_slots()
        .expect("the wrong-branch snapshot fixture seals");
    let owner = port.upgrade_owner().expect("the snapshot owner upgrades");
    let admission = owner.admit().expect("wrong-branch planning admits");
    assert!(matches!(
        owner.plan_retirement_releasing_snapshots_exact(
            &admission,
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
