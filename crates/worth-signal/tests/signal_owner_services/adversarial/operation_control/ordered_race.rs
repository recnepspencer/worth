use std::sync::mpsc;
use std::thread;

use worth_signal::facade::branch::{SignalOwnerOperationBoundary, SignalOwnerOperationControl};

use super::super::world::PROGRESS_BOUND;

pub(super) fn run<L, R>(
    runtime: &Option<super::super::world::Runtime>,
    control: &SignalOwnerOperationControl,
    left: L,
    right: R,
    left_parked: bool,
) -> (Result<(), &'static str>, Result<(), &'static str>)
where
    L: FnOnce() -> Result<(), &'static str> + Send,
    R: FnOnce() -> Result<(), &'static str> + Send,
{
    let owner_root = runtime
        .as_ref()
        .expect("the ordered contenders share a live owner root");
    let pause = control.arm_pause_once(SignalOwnerOperationBoundary::BranchRegistryLookup);
    let (left_tx, left_rx) = mpsc::sync_channel(1);
    let (right_tx, right_rx) = mpsc::sync_channel(1);
    let result = thread::scope(|scope| {
        if left_parked {
            scope.spawn(move || {
                let _ = left_tx.send(left());
            });
            assert!(pause.wait_until_reached(PROGRESS_BOUND));
            scope.spawn(move || {
                let _ = right_tx.send(right());
            });
            let right_result = right_rx
                .recv_timeout(PROGRESS_BOUND)
                .expect("the unparked right contender resolves first");
            pause.release();
            let left_result = left_rx
                .recv_timeout(PROGRESS_BOUND)
                .expect("the parked left contender resolves after release");
            (left_result, right_result)
        } else {
            scope.spawn(move || {
                let _ = right_tx.send(right());
            });
            assert!(pause.wait_until_reached(PROGRESS_BOUND));
            scope.spawn(move || {
                let _ = left_tx.send(left());
            });
            let left_result = left_rx
                .recv_timeout(PROGRESS_BOUND)
                .expect("the unparked left contender resolves first");
            pause.release();
            let right_result = right_rx
                .recv_timeout(PROGRESS_BOUND)
                .expect("the parked right contender resolves after release");
            (left_result, right_result)
        }
    });
    owner_root
        .owner_operation_control()
        .expect("the owner root remains live through both ordered contenders");
    result
}
