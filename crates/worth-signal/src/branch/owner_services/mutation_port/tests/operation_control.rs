mod boundaries;
mod cancellation;
mod panic_truth;
mod performed_truth;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::branch::owner_services::operation_control::{
    SignalOwnerOperationBoundary, SignalOwnerOperationControl,
};

use super::world::MutationWorld;

const CONTROL_BOUND: Duration = Duration::from_secs(3);

fn run_paused<R>(
    control: &SignalOwnerOperationControl,
    boundary: SignalOwnerOperationBoundary,
    operation: impl FnOnce() -> R + Send + 'static,
    at_boundary: impl FnOnce(),
) -> R
where
    R: Send + 'static,
{
    let pause = control.arm_pause_once(boundary);
    let (send, receive) = mpsc::sync_channel(1);
    let worker = thread::spawn(move || {
        let _ = send.send(operation());
    });
    assert!(
        pause.wait_until_reached(CONTROL_BOUND),
        "mutation port did not reach {boundary:?}"
    );
    at_boundary();
    pause.release();
    let result = receive
        .recv_timeout(CONTROL_BOUND)
        .expect("controlled mutation returns within the progress bound");
    worker.join().expect("controlled mutation worker exits");
    result
}

fn assert_no_pending_reservations(world: &MutationWorld<()>) {
    assert_eq!(world.owner.reservation_count(), 0);
    assert_eq!(
        world
            .owner
            .retention_ledger_observation()
            .reserved_admitted_lease_count,
        0
    );
}
