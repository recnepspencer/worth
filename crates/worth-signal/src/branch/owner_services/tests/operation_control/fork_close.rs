use std::sync::mpsc;
use std::thread;

use crate::branch::owner_services::operation_control::SignalOwnerOperationBoundary;
use crate::branch::{validate_signal_branch_name, SignalOwnerLifecycleObservation};

use super::super::progress_bound::PROGRESS_BOUND;
use super::super::runtime_root::runtime_with_two_branches;
use crate::branch::owner_services::SignalOwnerCancellationSource;

#[test]
fn operation_control_reaches_registry_and_both_fork_boundaries_with_exact_handle() {
    for boundary in [
        SignalOwnerOperationBoundary::BranchRegistryReservation,
        SignalOwnerOperationBoundary::ForkSourceCapture,
        SignalOwnerOperationBoundary::ForkDestinationInstallation,
    ] {
        exercise_fork_pause(boundary);
    }
}

fn exercise_fork_pause(boundary: SignalOwnerOperationBoundary) {
    let (mut runtime, _, source, basis) = runtime_with_two_branches();
    let (_, mutation, _) = runtime.owner_port_slots().expect("fork owner seals");
    let owner = mutation.upgrade_owner().expect("fork owner remains live");
    let setup = owner.admit().expect("fork setup admits");
    let source_cell = owner
        .lookup_cell(&setup, source.id)
        .expect("fork source is live");
    drop(setup);
    let before = owner.cost_snapshot();
    let pause = owner.operation_control().arm_pause_once(boundary);
    let (done_tx, done_rx) = mpsc::sync_channel(1);
    let worker_owner = owner.clone();
    thread::spawn(move || {
        let admission = worker_owner.admit().expect("fork worker admits");
        let reservation = worker_owner
            .reserve_fork_output(&admission, &source_cell)
            .expect("fork output reserves");
        let cancellation = SignalOwnerCancellationSource::new();
        let ready = reservation
            .fork(
                &basis,
                validate_signal_branch_name("operation-control-destination")
                    .expect("fork name is valid"),
                &cancellation.token(),
            )
            .expect("controlled fork completes");
        let issued = ready
            .installed()
            .cell()
            .with_state(&admission, |state, _| state.handle().clone())
            .expect("installed cell exposes its exact owner handle");
        let (handle, destination_basis) = ready.into_destination_parts();
        let _ = done_tx.send((handle, issued, destination_basis.owner_branch_id()));
    });

    assert!(pause.wait_until_reached(PROGRESS_BOUND));
    match boundary {
        SignalOwnerOperationBoundary::BranchRegistryReservation => {
            assert_eq!(owner.live_count(), 2);
            assert_eq!(owner.reservation_count(), 0);
        }
        SignalOwnerOperationBoundary::ForkSourceCapture => {
            assert_eq!(owner.live_count(), 2);
            assert_eq!(owner.reservation_count(), 1);
            assert_eq!(
                owner.cost_snapshot().fork_source_captures(),
                before.fork_source_captures() + 1
            );
        }
        SignalOwnerOperationBoundary::ForkDestinationInstallation => {
            assert_eq!(owner.live_count(), 3);
            assert_eq!(owner.reservation_count(), 0);
            assert_eq!(
                owner.cost_snapshot().fork_destination_installations(),
                before.fork_destination_installations() + 1
            );
        }
        _ => unreachable!("the fork table names only fork boundaries"),
    }
    pause.release();
    let (handle, issued, basis_id) = done_rx
        .recv_timeout(PROGRESS_BOUND)
        .expect("controlled fork returns");
    assert_eq!(handle, issued);
    assert_eq!(handle.id, basis_id);
    assert_eq!(handle.parent_branch_id, Some(source.id));
    assert_eq!(handle.name, "operation-control-destination");
}

#[test]
fn operation_control_reaches_detached_owner_close_batch() {
    let (mut runtime, _, _, basis) = runtime_with_two_branches();
    let (basis_port, _, _) = runtime.owner_port_slots().expect("close owner seals");
    let owner = basis_port
        .upgrade_owner()
        .expect("close owner remains live");
    drop(basis);
    let before = owner.cost_snapshot().close_batches();
    let pause = owner
        .operation_control()
        .arm_pause_once(SignalOwnerOperationBoundary::OwnerCloseBatch);
    let (done_tx, done_rx) = mpsc::sync_channel(1);
    let worker_owner = owner.clone();
    thread::spawn(move || {
        let _ = done_tx.send(worker_owner.close());
    });
    assert!(pause.wait_until_reached(PROGRESS_BOUND));
    assert_eq!(
        owner.lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closing
    );
    assert_eq!(owner.cost_snapshot().close_batches(), before + 1);
    pause.release();
    assert_eq!(done_rx.recv_timeout(PROGRESS_BOUND), Ok(Ok(())));
    assert_eq!(
        owner.lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closed
    );
}
