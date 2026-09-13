use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use worth_proof::TransitionOutcome;

use crate::branch::{SignalBranchRetirementDenial, SignalBranchRetirementReason};

use super::super::super::tests::runtime_root::runtime_with_two_branches;
use super::super::super::SignalOwnerLifecycleObservation;

const PROGRESS_BOUND: Duration = Duration::from_secs(2);

#[test]
fn weak_port_observes_open_closing_closed_and_owner_loss() {
    let (mut runtime, _, _, _) = runtime_with_two_branches();
    let (_, _, port) = runtime.owner_port_slots().expect("the real runtime seals");
    assert_eq!(
        port.owner_lifecycle_observation(),
        SignalOwnerLifecycleObservation::Open
    );
    let owner = port.upgrade_owner().expect("the owner root remains live");
    let admission = owner.admit().expect("the owner admits work before close");
    let closing_owner = owner.clone();
    let (closed_tx, closed_rx) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let _ = closed_tx.send(closing_owner.close());
    });

    assert!(observe_within(|| {
        port.owner_lifecycle_observation() == SignalOwnerLifecycleObservation::Closing
    }));
    assert_eq!(closed_rx.try_recv(), Err(TryRecvError::Empty));
    drop(admission);
    assert_eq!(
        closed_rx.recv_timeout(PROGRESS_BOUND),
        Ok(Ok(())),
        "close drains the admitted call within the bound"
    );
    assert_eq!(
        port.owner_lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closed
    );
    assert!(port.owner_service_cost_snapshot().is_err());
    drop(owner);
    drop(runtime);
    assert_eq!(
        port.owner_lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closed
    );
    assert!(port.owner_service_cost_snapshot().is_err());
}

#[test]
fn root_drop_during_admitted_work_closes_without_hidden_port_owner() {
    let (mut runtime, _, _, _) = runtime_with_two_branches();
    let (_, _, port) = runtime.owner_port_slots().expect("the real runtime seals");
    let owner = port.upgrade_owner().expect("the owner root remains live");
    let admission = owner.admit().expect("the owner admits synchronous work");

    drop(runtime);
    assert_eq!(
        port.owner_lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closing
    );
    drop(admission);
    assert_eq!(
        port.owner_lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closed
    );
    drop(owner);
    assert!(port.owner_service_cost_snapshot().is_err());
}

#[test]
fn lifecycle_and_cost_inspection_account_for_their_weak_upgrades_only() {
    let (mut runtime, _, _, _) = runtime_with_two_branches();
    let (_, _, port) = runtime.owner_port_slots().expect("the real runtime seals");
    let before = port
        .owner_service_cost_snapshot()
        .expect("the first inspection upgrades the live owner");
    assert_eq!(
        port.owner_lifecycle_observation(),
        SignalOwnerLifecycleObservation::Open
    );
    let after = port
        .owner_service_cost_snapshot()
        .expect("the second cost inspection upgrades the live owner");

    assert_eq!(
        after.owner_upgrade_attempts(),
        before.owner_upgrade_attempts() + 2
    );
    assert_eq!(
        after.branch_registry_lookups(),
        before.branch_registry_lookups()
    );
    assert_eq!(
        after.branch_registry_reservations(),
        before.branch_registry_reservations()
    );
    assert_eq!(
        after.branch_registry_entries_scanned(),
        before.branch_registry_entries_scanned()
    );
    assert_eq!(after.target_cell_contacts(), before.target_cell_contacts());
    assert_eq!(after.target_cell_waits(), before.target_cell_waits());
    assert_eq!(after.canonical_movements(), before.canonical_movements());
    assert_eq!(
        after.retention_registry_contacts(),
        before.retention_registry_contacts()
    );
}

#[test]
fn planning_denies_during_close_and_after_owner_loss_without_hidden_ownership() {
    let (mut closing_runtime, _, _, closing_basis) = runtime_with_two_branches();
    let (_, _, closing_port) = closing_runtime
        .owner_port_slots()
        .expect("the closing fixture seals");
    let closing_owner = closing_port
        .upgrade_owner()
        .expect("the closing owner remains live");
    let admitted = closing_owner.admit().expect("work admits before close");
    let worker_owner = closing_owner.clone();
    let (closed_tx, closed_rx) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let _ = closed_tx.send(worker_owner.close());
    });
    assert!(observe_within(|| {
        closing_port.owner_lifecycle_observation() == SignalOwnerLifecycleObservation::Closing
    }));
    assert!(matches!(
        closing_port.plan_retirement_exact(closing_basis, SignalBranchRetirementReason::Rejected,),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::OwnerUnavailable(_))
    ));
    assert!(closing_port.owner_service_cost_snapshot().is_err());
    drop(admitted);
    assert_eq!(closed_rx.recv_timeout(PROGRESS_BOUND), Ok(Ok(())));

    let (mut lost_runtime, _, _, lost_basis) = runtime_with_two_branches();
    let (_, _, lost_port) = lost_runtime
        .owner_port_slots()
        .expect("the owner-loss fixture seals");
    drop(lost_runtime);
    assert!(matches!(
        lost_port.plan_retirement_exact(
            lost_basis,
            SignalBranchRetirementReason::DependencyCancellation,
        ),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::OwnerUnavailable(_))
    ));
    assert_eq!(
        lost_port.owner_lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closed
    );
    assert!(lost_port.owner_service_cost_snapshot().is_err());
}

fn observe_within(mut observation_reached: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + PROGRESS_BOUND;
    while Instant::now() < deadline {
        if observation_reached() {
            return true;
        }
        thread::yield_now();
    }
    observation_reached()
}
