#[path = "operation_control/advance_unwind.rs"]
mod advance_unwind;
#[path = "operation_control/close_unwind.rs"]
mod close_unwind;
#[path = "operation_control/fork_close.rs"]
mod fork_close;
#[path = "operation_control/fork_unwind.rs"]
mod fork_unwind;
#[path = "operation_control/movement.rs"]
mod movement;
#[path = "operation_control/pre_effect_unwind.rs"]
mod pre_effect_unwind;
#[path = "operation_control/restore_unwind.rs"]
mod restore_unwind;
#[path = "operation_control/snapshot_unwind.rs"]
mod snapshot_unwind;

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc;
use std::thread;

use super::super::operation_control::SignalOwnerOperationBoundary;
use super::progress_bound::PROGRESS_BOUND;
use super::runtime_root::runtime_with_two_branches;

#[test]
fn operation_control_is_unarmed_equivalent_owner_local_and_one_shot() {
    let (mut runtime_a, _, branch_a, _basis_a) = runtime_with_two_branches();
    let (port_a, _, _) = runtime_a.owner_port_slots().expect("owner A seals");
    let owner_a = port_a.upgrade_owner().expect("owner A remains live");
    let (mut runtime_b, _, branch_b, _basis_b) = runtime_with_two_branches();
    let (port_b, _, _) = runtime_b.owner_port_slots().expect("owner B seals");
    let owner_b = port_b.upgrade_owner().expect("owner B remains live");

    let control = owner_a.operation_control();
    assert_eq!(owner_a.cost_snapshot(), owner_b.cost_snapshot());
    let admission_a = owner_a.admit().expect("unarmed A admits");
    let admission_b = owner_b.admit().expect("unarmed B admits");
    owner_a
        .lookup_cell(&admission_a, branch_a.id)
        .expect("unarmed A lookup succeeds");
    owner_b
        .lookup_cell(&admission_b, branch_b.id)
        .expect("unarmed B lookup succeeds");
    drop(admission_a);
    drop(admission_b);
    assert_eq!(
        owner_a.cost_snapshot(),
        owner_b.cost_snapshot(),
        "obtaining an unarmed controller changes no owner result or counter"
    );

    let pause = control.arm_pause_once(SignalOwnerOperationBoundary::BranchRegistryLookup);
    let (done_tx, done_rx) = mpsc::sync_channel(1);
    let worker_owner = owner_a.clone();
    thread::spawn(move || {
        let admission = worker_owner.admit().expect("parked lookup admits");
        let result = worker_owner
            .lookup_cell(&admission, branch_a.id)
            .map(|cell| cell.branch_id());
        let _ = done_tx.send(result);
    });
    assert!(pause.wait_until_reached(PROGRESS_BOUND));
    let unrelated = owner_b.admit().expect("unrelated owner admits");
    assert_eq!(
        owner_b
            .lookup_cell(&unrelated, branch_b.id)
            .expect("unrelated owner progresses")
            .branch_id(),
        branch_b.id
    );
    pause.release();
    assert_eq!(done_rx.recv_timeout(PROGRESS_BOUND), Ok(Ok(branch_a.id)));

    let second = owner_a.admit().expect("one-shot follow-up admits");
    assert_eq!(
        owner_a
            .lookup_cell(&second, branch_a.id)
            .expect("one-shot pause is consumed")
            .branch_id(),
        branch_a.id
    );
    control.inject_panic_once(SignalOwnerOperationBoundary::BranchRegistryLookup);
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _ = owner_a.lookup_cell(&second, branch_a.id);
    }))
    .is_err());
    assert!(owner_a.lookup_cell(&second, branch_a.id).is_ok());
}

#[test]
fn operation_control_pause_drop_releases_and_lifecycle_admission_is_reachable() {
    let (mut runtime, _, branch, _basis) = runtime_with_two_branches();
    let (port, _, _) = runtime.owner_port_slots().expect("owner seals");
    let owner = port.upgrade_owner().expect("owner remains live");
    let control = owner.operation_control();

    let pause = control.arm_pause_once(SignalOwnerOperationBoundary::OwnerLifecycleAdmission);
    let (admitted_tx, admitted_rx) = mpsc::sync_channel(1);
    let worker_owner = owner.clone();
    thread::spawn(move || {
        let admitted = worker_owner.admit().is_ok();
        let _ = admitted_tx.send(admitted);
    });
    assert!(pause.wait_until_reached(PROGRESS_BOUND));
    drop(pause);
    assert_eq!(admitted_rx.recv_timeout(PROGRESS_BOUND), Ok(true));

    let pause = control.arm_pause_once(SignalOwnerOperationBoundary::BranchRegistryLookup);
    let (lookup_tx, lookup_rx) = mpsc::sync_channel(1);
    let worker_owner = owner.clone();
    thread::spawn(move || {
        let admission = worker_owner.admit().expect("drop-safe lookup admits");
        let result = worker_owner
            .lookup_cell(&admission, branch.id)
            .map(|cell| cell.branch_id());
        let _ = lookup_tx.send(result);
    });
    assert!(pause.wait_until_reached(PROGRESS_BOUND));
    drop(pause);
    assert_eq!(lookup_rx.recv_timeout(PROGRESS_BOUND), Ok(Ok(branch.id)));
}
