use std::sync::{mpsc, Arc};
use std::thread;

use crate::branch::SignalBranchAdvanceDenial;

use super::super::SignalOwnerCancellationSource;
use super::progress_bound::{wait_until_progress, worker_park, PROGRESS_BOUND};
use super::runtime_root::runtime_with_two_branches;

pub(super) mod restore;
mod retirement;

#[test]
fn cancellation_while_waiting_for_same_cell_denies_without_movement() {
    let (mut runtime, _, branch, expected) = runtime_with_two_branches();
    let (_, mutation, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = mutation.upgrade_owner().expect("owner remains live");
    let setup_admission = owner.admit().expect("setup admits");
    let cell = owner
        .lookup_cell(&setup_admission, branch.id)
        .expect("target cell is live");
    drop(setup_admission);
    let (holder_park, mut holder_control) = worker_park();
    let (holder_done_tx, holder_done_rx) = mpsc::sync_channel(1);
    let holder_cell = Arc::clone(&cell);
    let holder_owner = Arc::clone(&owner);
    thread::spawn(move || {
        let holder_admission = holder_owner.admit().expect("holder admits in its worker");
        let result = holder_cell.with_state(&holder_admission, |_, _| {
            holder_park.park("same-cell cancellation holder");
        });
        let _ = holder_done_tx.send(result);
    });
    holder_control.wait_until_parked("same-cell cancellation holder");

    let cancellation = SignalOwnerCancellationSource::new();
    let token = cancellation.token();
    let waiter_cell = Arc::clone(&cell);
    let waiter_expected = expected.clone();
    let waiter_owner = Arc::clone(&owner);
    let (waiter_done_tx, waiter_done_rx) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let waiter_admission = waiter_owner.admit().expect("waiter admits in its worker");
        let result = waiter_cell
            .advance_exact::<(), (), _>(
                &waiter_admission,
                &waiter_expected,
                &mut (),
                &token,
                |_| Ok(()),
            )
            .map(|_| ());
        let _ = waiter_done_tx.send(result);
    });
    assert!(
        wait_until_progress("waiter reaches same-cell wait", || {
            cell.cost_snapshot().waits() == 1
        }),
        "waiter did not reach the named cell wait within bound"
    );
    cancellation.cancel();
    holder_control.release();
    holder_done_rx
        .recv_timeout(PROGRESS_BOUND)
        .expect("holder completion is bounded")
        .expect("holder exits cleanly");
    let result = waiter_done_rx
        .recv_timeout(PROGRESS_BOUND)
        .expect("cancelled waiter completion is bounded");
    assert!(matches!(
        result,
        Err(SignalBranchAdvanceDenial::CancelledNoMovement)
    ));
    assert_eq!(cell.cost_snapshot().movements(), 0);
    let healthy = SignalOwnerCancellationSource::new();
    let healthy_admission = owner.admit().expect("healthy twin admits");
    cell.advance_exact::<(), (), _>(
        &healthy_admission,
        &expected,
        &mut (),
        &healthy.token(),
        |_| Ok(()),
    )
    .expect("healthy twin advances from the unchanged basis");
    assert_eq!(cell.cost_snapshot().movements(), 1);
}
