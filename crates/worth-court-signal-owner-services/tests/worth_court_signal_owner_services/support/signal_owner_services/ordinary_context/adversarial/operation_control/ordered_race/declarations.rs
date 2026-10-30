pub(crate) use std::sync::mpsc;
pub(crate) use std::thread;

pub(crate) use {
    worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest,
    worth_signal::facade::branch::SignalOwnerOperationControlForTest,
};

pub(crate) use crate::signal_owner_services::ordinary_context::adversarial::world::PROGRESS_BOUND;

pub(crate) fn run<L, R>(
    runtime: &Option<crate::signal_owner_services::ordinary_context::adversarial::world::Runtime>,
    control: &SignalOwnerOperationControlForTest,
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
    let pause = control.arm_pause_once_for_test(
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::BranchRegistryLookup,
    );
    let (left_tx, left_rx) = std::sync::mpsc::sync_channel(1);
    let (right_tx, right_rx) = std::sync::mpsc::sync_channel(1);
    let result = std::thread::scope(|scope| {
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
        .owner_operation_control_for_test()
        .expect("the owner root remains live through both ordered contenders");
    result
}
