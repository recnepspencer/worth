use std::sync::{mpsc, Arc};
use std::thread;

use worth_foundational::{FoundationalBranchTargetBasis, FoundationalBranchTargetEncoding};
use worth_proof::TransitionOutcome;

use crate::branch::{
    AdmittedSignalBranchBasis, SignalBranchAdvanceDenial, SignalBranchRetentionAcquisitionDenial,
    SignalBranchRetentionTerminalOutcome, SignalBranchRetirementDenial,
    SignalBranchRetirementReason,
};
use crate::state::SignalBranchId;

use super::super::{
    SignalBranchCellState, SignalBranchExecutionCell, SignalOwner, SignalOwnerCancellationSource,
    SignalOwnerOperationAdmission,
};
use super::progress_bound::PROGRESS_BOUND;
use super::runtime_root::runtime_with_two_branches;

type TestOwner = SignalOwner<(), (), ()>;
type TestCell = SignalBranchExecutionCell<SignalBranchCellState<(), (), ()>>;

struct ReservedFenceWorkerWorld<'a> {
    owner: Arc<TestOwner>,
    retired_id: SignalBranchId,
    sibling_id: SignalBranchId,
    retired_basis: &'a AdmittedSignalBranchBasis,
    sibling_basis: AdmittedSignalBranchBasis,
    sibling_target: FoundationalBranchTargetEncoding,
}

struct ReservedFenceWorkerControl {
    ready: mpsc::SyncSender<()>,
    attempt: mpsc::Receiver<()>,
    attempted: mpsc::SyncSender<crate::branch::retention::SignalRetentionLedgerObservation>,
}

struct RetentionFenceBaseline {
    ledger: crate::branch::retention::SignalRetentionLedgerObservation,
    contacts: u64,
}

fn run_reserved_fence_worker(
    world: ReservedFenceWorkerWorld<'_>,
    control: ReservedFenceWorkerControl,
) {
    let admission = world
        .owner
        .admit()
        .expect("the independent thread admits before retirement reserves");
    let retired_cell = world
        .owner
        .lookup_cell(&admission, world.retired_id)
        .expect("the target cell is looked up before retirement");
    let sibling_cell = world
        .owner
        .lookup_cell(&admission, world.sibling_id)
        .expect("the sibling cell remains independently addressable");
    control
        .ready
        .send(())
        .expect("the valid admission reports ready");
    control
        .attempt
        .recv_timeout(PROGRESS_BOUND)
        .expect("the retirement reservation releases the attempts");
    let baseline = assert_exact_branch_acquisitions_are_fenced(&world, &admission, &retired_cell);
    let after_progress =
        exercise_unrelated_branch_progress(&world, &admission, &sibling_cell, baseline);
    control
        .attempted
        .send(after_progress)
        .expect("the bounded worker reports exact progress");
}

fn assert_exact_branch_acquisitions_are_fenced(
    world: &ReservedFenceWorkerWorld,
    admission: &SignalOwnerOperationAdmission<'_>,
    retired_cell: &Arc<TestCell>,
) -> RetentionFenceBaseline {
    let ledger = world.owner.retention_ledger_observation();
    let contacts = world.owner.cost_snapshot().retention_registry_contacts();
    assert!(matches!(
        world
            .owner
            .acquire_external_retention(admission, world.retired_basis),
        Err(SignalBranchRetentionAcquisitionDenial::RetiredBranch { branch_id })
            if branch_id == world.retired_id
    ));
    assert!(matches!(
        world.owner.reserve_advance_output(admission, retired_cell),
        Err(SignalBranchAdvanceDenial::RetentionUnavailable {
            denial: SignalBranchRetentionAcquisitionDenial::RetiredBranch { branch_id },
        }) if branch_id == world.retired_id
    ));
    assert_eq!(world.owner.retention_ledger_observation(), ledger);
    assert_eq!(
        world.owner.cost_snapshot().retention_registry_contacts(),
        contacts,
        "both exact-branch denials precede retention-ledger contact"
    );
    RetentionFenceBaseline { ledger, contacts }
}

fn exercise_unrelated_branch_progress(
    world: &ReservedFenceWorkerWorld,
    admission: &SignalOwnerOperationAdmission<'_>,
    sibling_cell: &Arc<TestCell>,
    baseline: RetentionFenceBaseline,
) -> crate::branch::retention::SignalRetentionLedgerObservation {
    let sibling_lease = world
        .owner
        .acquire_external_retention(admission, &world.sibling_basis)
        .expect("the unrelated exact target retains");
    let retained = world.owner.retention_ledger_observation();
    assert_eq!(
        retained.external_lease_identities,
        vec![(
            baseline.ledger.next_lease_id + 1,
            world.sibling_id,
            world.sibling_target.clone(),
        )]
    );
    assert_eq!(
        sibling_lease.release().outcome(),
        SignalBranchRetentionTerminalOutcome::Released
    );
    let sibling_output = world
        .owner
        .reserve_advance_output(admission, sibling_cell)
        .expect("the unrelated admitted output reserves");
    assert_eq!(
        world
            .owner
            .retention_ledger_observation()
            .reserved_count_by_branch,
        vec![(world.sibling_id, 1)]
    );
    drop(sibling_output);
    sibling_cell
        .with_state(admission, |state, _| {
            assert_eq!(state.branch_id(), world.sibling_id);
        })
        .expect("unrelated canonical cell work progresses");
    let after_progress = world.owner.retention_ledger_observation();
    assert_eq!(
        after_progress.next_lease_id,
        baseline.ledger.next_lease_id + 2
    );
    assert_eq!(after_progress.used_capacity, baseline.ledger.used_capacity);
    assert_eq!(
        after_progress.admitted_lease_identities,
        baseline.ledger.admitted_lease_identities
    );
    assert!(after_progress.external_lease_identities.is_empty());
    assert!(after_progress.reserved_count_by_branch.is_empty());
    assert_eq!(
        world.owner.cost_snapshot().retention_registry_contacts(),
        baseline.contacts + 2
    );
    after_progress
}

#[test]
fn external_retention_inserted_before_retirement_reservation_denies_exactly_then_releases() {
    let (mut runtime, _, branch, basis) = runtime_with_two_branches();
    let target_key = basis
        .descriptor()
        .observation()
        .target()
        .as_basis()
        .expect("the production-issued descriptor carries a basis target")
        .canonical_encoding();
    let plan = match runtime.plan_signal_branch_retirement(
        branch.clone(),
        basis,
        SignalBranchRetirementReason::Rejected,
    ) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("the exact retirement plan issues before sealing: {other:?}"),
    };
    let (_, _, lifecycle) = runtime.owner_port_slots().expect("the runtime seals");
    let owner = lifecycle.upgrade_owner().expect("the owner remains live");
    let admission = owner.admit().expect("retention and retirement admit");
    let before = owner.retention_ledger_observation();

    let external = owner
        .acquire_external_retention(&admission, plan.admitted_basis())
        .expect("the saved real target retains after planning");
    let retained = owner.retention_ledger_observation();
    assert_eq!(retained.next_lease_id, before.next_lease_id + 1);
    assert_eq!(retained.used_capacity, before.used_capacity + 1);
    assert_eq!(
        retained.external_lease_identities,
        vec![(before.next_lease_id + 1, branch.id, target_key.clone())]
    );
    assert_eq!(retained.external_count_by_branch, vec![(branch.id, 1)]);
    assert_eq!(retained.external_count_by_target, vec![(target_key, 1)]);

    assert!(matches!(
        owner.reserve_retirement(&admission, branch.id),
        Err(SignalBranchRetirementDenial::RetainedComponentBasis {
            branch_id,
            active_leases: 1,
        }) if branch_id == branch.id
    ));
    assert_eq!(
        owner.retention_ledger_observation(),
        retained,
        "the denial neither consumes nor duplicates the exact obligation"
    );
    let contract = owner
        .metadata
        .retirement_contract_observation(&admission, branch.id)
        .expect("the denied contract remains observable");
    assert_eq!(contract.active_reservations, 0);
    assert_eq!(contract.reserved_receipt_count, 0);
    assert_eq!(contract.retained_receipt_count, 0);
    assert_eq!(owner.live_count(), 2);

    let released = external.release();
    assert_eq!(
        released.outcome(),
        SignalBranchRetentionTerminalOutcome::Released
    );
    assert_eq!(released.branch_id(), branch.id);
    assert_eq!(released.remaining_branch_leases(), 0);
    assert_eq!(released.remaining_target_leases(), 0);
    let mut after_release = before.clone();
    after_release.next_lease_id += 1;
    assert_eq!(owner.retention_ledger_observation(), after_release);

    let receipt = owner
        .reserve_retirement(&admission, branch.id)
        .expect("release reopens exact retirement eligibility")
        .execute(plan, &SignalOwnerCancellationSource::new().token())
        .expect("the healthy retirement completes");
    assert_eq!(receipt.retired_branch(), &branch);
    assert_eq!(receipt.reason(), SignalBranchRetirementReason::Rejected);
    assert_eq!(owner.live_count(), 1);
    let completed = owner
        .metadata
        .retirement_contract_observation(&admission, branch.id)
        .expect("the completed contract remains observable");
    assert_eq!(completed.active_reservations, 0);
    assert_eq!(completed.reserved_receipt_count, 0);
    assert_eq!(completed.retained_receipt_count, 1);
}

#[test]
fn retirement_reservation_fences_exact_branch_acquisitions_while_sibling_progresses() {
    let (mut runtime, sibling, retired, retired_basis) = runtime_with_two_branches();
    let sibling_basis = runtime
        .observe_signal_branch_basis(sibling.clone())
        .expect("the unrelated branch issues a real basis");
    let sibling_target = sibling_basis
        .descriptor()
        .observation()
        .target()
        .as_basis()
        .expect("the sibling descriptor carries a basis target")
        .canonical_encoding();
    let plan = match runtime.plan_signal_branch_retirement(
        retired.clone(),
        retired_basis,
        SignalBranchRetirementReason::Superseded,
    ) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("the exact retirement plan issues before sealing: {other:?}"),
    };
    let (_, _, lifecycle) = runtime.owner_port_slots().expect("the runtime seals");
    let owner = lifecycle.upgrade_owner().expect("the owner remains live");
    let retirement_admission = owner.admit().expect("retirement admits");

    let (ready_tx, ready_rx) = mpsc::sync_channel(1);
    let (attempt_tx, attempt_rx) = mpsc::sync_channel(1);
    let (attempted_tx, attempted_rx) = mpsc::sync_channel(1);
    let (retirement, after_progress) = thread::scope(|scope| {
        let worker_world = ReservedFenceWorkerWorld {
            owner: Arc::clone(&owner),
            retired_id: retired.id,
            sibling_id: sibling.id,
            retired_basis: plan.admitted_basis(),
            sibling_basis,
            sibling_target,
        };
        let worker_control = ReservedFenceWorkerControl {
            ready: ready_tx,
            attempt: attempt_rx,
            attempted: attempted_tx,
        };
        let worker = scope.spawn(move || run_reserved_fence_worker(worker_world, worker_control));
        assert_eq!(ready_rx.recv_timeout(PROGRESS_BOUND), Ok(()));
        let retirement = owner
            .reserve_retirement(&retirement_admission, retired.id)
            .expect("metadata reservation wins before either exact-branch acquisition");
        attempt_tx
            .send(())
            .expect("the bounded worker begins its fenced attempts");
        let after_progress = attempted_rx
            .recv_timeout(PROGRESS_BOUND)
            .expect("the unrelated branch progresses while retirement remains reserved");
        worker.join().expect("the bounded worker remains healthy");
        (retirement, after_progress)
    });
    let receipt = retirement
        .execute(plan, &SignalOwnerCancellationSource::new().token())
        .expect("denied acquisitions leave retirement executable");
    assert_eq!(receipt.retired_branch(), &retired);
    assert_eq!(receipt.reason(), SignalBranchRetirementReason::Superseded);
    assert_eq!(owner.live_count(), 1);
    let final_ledger = owner.retention_ledger_observation();
    assert_eq!(final_ledger.next_lease_id, after_progress.next_lease_id);
    assert_eq!(final_ledger.used_capacity, 0);
    assert!(final_ledger.admitted_lease_identities.is_empty());
    assert!(final_ledger.external_lease_identities.is_empty());
    assert!(final_ledger.admitted_count_by_branch.is_empty());
    assert!(final_ledger.reserved_count_by_branch.is_empty());
    assert!(final_ledger.external_count_by_branch.is_empty());
    let contract = owner
        .metadata
        .retirement_contract_observation(&retirement_admission, retired.id)
        .expect("the completed contract remains observable");
    assert_eq!(contract.active_reservations, 0);
    assert_eq!(contract.reserved_receipt_count, 0);
    assert_eq!(contract.retained_receipt_count, 1);
}
