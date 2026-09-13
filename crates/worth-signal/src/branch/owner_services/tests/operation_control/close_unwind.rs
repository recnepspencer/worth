use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{mpsc, Arc};
use std::thread;

use crate::branch::owner_services::operation_control::SignalOwnerOperationBoundary;
use crate::branch::SignalOwnerLifecycleObservation;

use super::super::progress_bound::{wait_until_progress, PROGRESS_BOUND};
use super::super::runtime_root::runtime_with_two_branches;

#[test]
fn close_batch_fault_releases_cleanup_claim_for_waiting_retry() {
    let (mut runtime, _, _, basis) = runtime_with_two_branches();
    let captured = runtime
        .capture_signal_branch_snapshot(&basis)
        .expect("close fault fixture installs independently populated metadata");
    let (basis_port, _, _) = runtime.owner_port_slots().expect("close owner seals");
    let owner = basis_port
        .upgrade_owner()
        .expect("close owner remains live");
    drop(captured);
    drop(basis);
    let before = owner.cost_snapshot();
    let control = owner.operation_control();
    let first_batch = control.arm_pause_once(SignalOwnerOperationBoundary::OwnerCloseBatch);
    control.inject_panic_once(SignalOwnerOperationBoundary::OwnerCloseBatch);
    let (first_tx, first_rx) = mpsc::sync_channel(1);
    let first_owner = Arc::clone(&owner);
    thread::spawn(move || {
        let panicked = catch_unwind(AssertUnwindSafe(|| first_owner.close())).is_err();
        let _ = first_tx.send(panicked);
    });
    assert!(first_batch.wait_until_reached(PROGRESS_BOUND));
    assert_eq!(
        owner.lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closing
    );
    assert_eq!(
        owner.live_count(),
        0,
        "the first registry batch is detached"
    );

    let (waiter_tx, waiter_rx) = mpsc::sync_channel(1);
    let waiter_owner = Arc::clone(&owner);
    thread::spawn(move || {
        let _ = waiter_tx.send(waiter_owner.close());
    });
    assert!(wait_until_progress(
        "a concurrent closer waits on the held cleanup claim",
        || { owner.cleanup_waiter_count() == 1 }
    ));

    first_batch.release();
    assert_eq!(first_rx.recv_timeout(PROGRESS_BOUND), Ok(true));
    assert_eq!(waiter_rx.recv_timeout(PROGRESS_BOUND), Ok(Ok(())));
    assert_eq!(
        owner.lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closed
    );
    assert_eq!(owner.cleanup_waiter_count(), 0);
    assert_eq!(owner.live_count(), 0);
    assert_eq!(owner.reservation_count(), 0);
    assert_eq!(owner.metadata.pending_snapshot_reservation_count(), 0);
    assert_eq!(owner.retention_ledger_observation().used_capacity, 0);
    assert_eq!(
        owner.cost_snapshot().close_batches(),
        before.close_batches() + 2,
        "both actually detached registry and metadata batches count once"
    );
    assert_eq!(owner.close(), Ok(()), "completed close remains idempotent");
}
