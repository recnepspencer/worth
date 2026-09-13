use worth_proof::TransitionOutcome;

use crate::branch::{SignalBranchRetirementDenial, SignalBranchRetirementReason};

use super::super::super::tests::runtime_root::runtime_with_two_branches;
use super::super::super::SignalOwnerCancellationSource;
use super::receipt_oracle::{expected_closeout_digest, expected_terminal_basis_digest};

#[test]
fn retire_exact_performs_one_real_cell_movement_and_preserves_exact_receipt() {
    let (mut runtime, sibling, target, basis) = runtime_with_two_branches();
    let expected_terminal_digest = expected_terminal_basis_digest(&target, basis.observation());
    let expected_parent = target
        .parent_branch_id
        .expect("the retirement target is a real fork child");
    let expected_closeout = expected_closeout_digest(
        target.id,
        expected_parent,
        target.head_snapshot_id,
        target.head_snapshot_id,
        SignalBranchRetirementReason::Rejected,
        &expected_terminal_digest,
    );
    let plan = match runtime.plan_signal_branch_retirement(
        target.clone(),
        basis,
        SignalBranchRetirementReason::Rejected,
    ) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("pre-seal owner must issue the exact linear plan: {other:?}"),
    };
    let (_, _, port) = runtime.owner_port_slots().expect("the real runtime seals");
    let owner = port.upgrade_owner().expect("the owner root remains live");
    let inspection = owner.admit().expect("cell inspection admits");
    let sibling_cell = owner
        .lookup_cell(&inspection, sibling.id)
        .expect("the sibling cell is canonical");
    let target_cell = owner
        .lookup_cell(&inspection, target.id)
        .expect("the target cell is canonical");
    drop(inspection);
    let sibling_before = sibling_cell.cost_snapshot();
    let target_before = target_cell.cost_snapshot();
    let owner_before = port
        .owner_service_cost_snapshot()
        .expect("cost inspection upgrades the live weak port");

    let outcome = port.retire_exact(plan, &SignalOwnerCancellationSource::new().token());
    let receipt = match outcome {
        TransitionOutcome::Success(receipt) => receipt,
        other => panic!("real lifecycle-port retirement must perform: {other:?}"),
    };
    let owner_after = port
        .owner_service_cost_snapshot()
        .expect("the live root retains descriptive counters");

    assert_eq!(receipt.retired_branch(), &target);
    assert_eq!(receipt.parent_branch_id(), expected_parent);
    assert_eq!(receipt.forked_from_snapshot_id(), target.head_snapshot_id);
    assert_eq!(receipt.terminal_head_snapshot_id(), target.head_snapshot_id);
    assert_eq!(receipt.reason(), SignalBranchRetirementReason::Rejected);
    assert_eq!(receipt.terminal_basis_digest(), expected_terminal_digest);
    assert_eq!(receipt.closeout_digest(), expected_closeout);
    assert_eq!(receipt.reclaimed_branch_state_count(), 1);
    assert_eq!(receipt.reclaimed_snapshot_state_count(), 0);
    assert_eq!(receipt.reclaimed_runtime_meta_count(), 0);
    assert_eq!(receipt.retained_proof_record_count(), 1);
    assert_eq!(sibling_cell.cost_snapshot(), sibling_before);
    assert_eq!(
        target_cell.cost_snapshot().contacts(),
        target_before.contacts() + 1
    );
    assert_eq!(target_cell.cost_snapshot().waits(), target_before.waits());
    assert_eq!(
        target_cell.cost_snapshot().movements(),
        target_before.movements() + 1
    );
    assert_eq!(
        owner_after.owner_upgrade_attempts(),
        owner_before.owner_upgrade_attempts() + 2
    );
    assert_eq!(
        owner_after.target_cell_contacts(),
        owner_before.target_cell_contacts() + 1
    );
    assert_eq!(
        owner_after.target_cell_waits(),
        owner_before.target_cell_waits()
    );
    assert_eq!(
        owner_after.canonical_movements(),
        owner_before.canonical_movements() + 1
    );
    assert_eq!(
        owner_after.retention_registry_contacts(),
        owner_before.retention_registry_contacts() + 1
    );
    assert_eq!(owner_after.branch_registry_entries_scanned(), 0);

    let follow_up = owner.admit().expect("the owner remains healthy");
    assert!(matches!(
        owner.lookup_cell(&follow_up, target.id),
        Err(super::super::super::SignalBranchRegistryDenial::UnknownBranch(branch_id))
            if branch_id == target.id
    ));
    assert_eq!(
        owner
            .metadata
            .retirement_receipt(&follow_up, target.id)
            .expect("receipt recovery is owner-admitted"),
        Some(receipt)
    );
    assert_eq!(owner.live_count(), 1);
}

#[test]
fn retire_exact_cancellation_releases_every_reservation_without_contact() {
    let (mut runtime, _, target, basis) = runtime_with_two_branches();
    let expected_observation = basis.observation().clone();
    let plan = match runtime.plan_signal_branch_retirement(
        target.clone(),
        basis,
        SignalBranchRetirementReason::Superseded,
    ) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("pre-seal owner must issue the exact linear plan: {other:?}"),
    };
    let (_, _, port) = runtime.owner_port_slots().expect("the real runtime seals");
    let owner = port.upgrade_owner().expect("the owner root remains live");
    let setup = owner.admit().expect("setup inspection admits");
    let target_cell = owner
        .lookup_cell(&setup, target.id)
        .expect("the target cell is canonical");
    drop(setup);
    let target_before = target_cell.cost_snapshot();
    let live_before = owner.live_count();
    let cancellation = SignalOwnerCancellationSource::new();
    cancellation.cancel();

    assert!(matches!(
        port.retire_exact(plan, &cancellation.token()),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::CancelledNoMovement)
    ));
    assert_eq!(target_cell.cost_snapshot(), target_before);
    assert_eq!(owner.live_count(), live_before);

    let follow_up = owner.admit().expect("cancelled work releases admission");
    let contract = owner
        .metadata
        .retirement_contract_observation(&follow_up, target.id)
        .expect("retirement cleanup is owner-admitted");
    assert_eq!(contract.active_reservations, 0);
    assert_eq!(contract.reserved_receipt_count, 0);
    assert_eq!(contract.retained_receipt_count, 0);
    let observed = owner
        .lookup_cell(&follow_up, target.id)
        .expect("cancellation reopens the same live cell")
        .observe_exact(&follow_up)
        .expect("the reopened cell remains healthy");
    assert_eq!(observed, expected_observation);
}

#[test]
fn retire_exact_maps_own_admission_reentry_before_target_contact() {
    let (mut runtime, _, target, basis) = runtime_with_two_branches();
    let plan = match runtime.plan_signal_branch_retirement(
        target.clone(),
        basis,
        SignalBranchRetirementReason::ProjectionRebuild,
    ) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("pre-seal owner must issue the exact linear plan: {other:?}"),
    };
    let (_, _, port) = runtime.owner_port_slots().expect("the real runtime seals");
    let owner = port.upgrade_owner().expect("the owner root remains live");
    let admission = owner.admit().expect("the executing thread admits setup");
    let cell = owner
        .lookup_cell(&admission, target.id)
        .expect("the exact target is installed");
    let target_before = cell.cost_snapshot();
    let metadata_hold = admission
        .hold_owner_metadata()
        .expect("the executing thread holds owner metadata");

    assert!(matches!(
        port.retire_exact(plan, &SignalOwnerCancellationSource::new().token()),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::OwnerReentry)
    ));
    assert_eq!(cell.cost_snapshot(), target_before);
    drop(metadata_hold);
    drop(admission);
    let follow_up = owner
        .admit()
        .expect("reentry denial releases port admission");
    cell.observe_exact(&follow_up)
        .expect("the denied target remains healthy");
}

#[test]
fn retire_exact_reports_owner_unavailable_after_root_loss() {
    let (mut runtime, _, target, basis) = runtime_with_two_branches();
    let plan = match runtime.plan_signal_branch_retirement(
        target,
        basis,
        SignalBranchRetirementReason::DependencyCancellation,
    ) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("pre-seal owner must issue the exact linear plan: {other:?}"),
    };
    let (_, _, port) = runtime.owner_port_slots().expect("the real runtime seals");
    drop(runtime);

    assert!(matches!(
        port.retire_exact(plan, &SignalOwnerCancellationSource::new().token()),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::OwnerUnavailable(_))
    ));
    assert!(port.owner_service_cost_snapshot().is_err());
    assert_eq!(
        port.owner_lifecycle_observation(),
        super::super::super::SignalOwnerLifecycleObservation::Closed
    );
}

#[test]
fn retire_exact_rechecks_sole_holder_at_execution_without_movement() {
    let (mut runtime, _, target, basis) = runtime_with_two_branches();
    let (_, _, port) = runtime.owner_port_slots().expect("the real runtime seals");
    let owner = port.upgrade_owner().expect("the owner root remains live");
    let plan =
        match port.plan_retirement_exact(basis, SignalBranchRetirementReason::ProjectionRebuild) {
            TransitionOutcome::Success(plan) => plan,
            other => panic!("the sole-holder plan must issue: {other:?}"),
        };
    let extra_holder = plan.admitted_basis().clone();
    let inspection = owner.admit().expect("cell inspection admits");
    let cell = owner
        .lookup_cell(&inspection, target.id)
        .expect("the exact target remains installed");
    drop(inspection);
    let before = cell.cost_snapshot();

    assert!(matches!(
        port.retire_exact(plan, &SignalOwnerCancellationSource::new().token()),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::SharedAdmittedBasis {
            branch_id,
            shared_holders: 2,
        }) if branch_id == target.id
    ));
    assert_eq!(cell.cost_snapshot(), before);
    assert_eq!(owner.live_count(), 2);
    drop(extra_holder);
}

#[test]
fn retire_exact_preserves_retirement_in_progress_denial_and_rollback() {
    let (mut runtime, _, target, basis) = runtime_with_two_branches();
    let plan = match runtime.plan_signal_branch_retirement(
        target.clone(),
        basis,
        SignalBranchRetirementReason::Merged,
    ) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("the pre-seal plan must issue: {other:?}"),
    };
    let (_, _, port) = runtime.owner_port_slots().expect("the real runtime seals");
    let owner = port.upgrade_owner().expect("the owner root remains live");
    let admission = owner.admit().expect("the first retirement admits");
    let reservation = owner
        .reserve_retirement(&admission, target.id)
        .expect("the first retirement reserves the exact branch");

    assert!(matches!(
        port.retire_exact(plan, &SignalOwnerCancellationSource::new().token()),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::RetirementInProgress {
            branch_id,
        }) if branch_id == target.id
    ));
    drop(reservation);
    drop(admission);
    let follow_up = owner.admit().expect("the original reservation rolls back");
    assert!(owner.lookup_cell(&follow_up, target.id).is_ok());
}
