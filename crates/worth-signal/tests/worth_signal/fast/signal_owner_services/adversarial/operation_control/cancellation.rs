use crate::fast::signal_owner_services::execution_custody;
pub(crate) use std::sync::mpsc;
pub(crate) use std::thread;

pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_signal::facade::branch::{
    AdmittedSignalBranchSnapshot, SignalBranchAdvanceDenial, SignalBranchForkOperationDenial,
    SignalBranchRestoreDenial, SignalBranchRetirementDenial, SignalBranchRetirementReason,
    SignalBranchSnapshotCaptureDenial, SignalOwnerCancellationSource,
    SignalOwnerOperationBoundaryForTest,
};

pub(crate) use super::super::world::{AdversarialWorld, PROGRESS_BOUND};

#[test]
fn cancellation_at_the_pre_movement_park_is_still_effect_free() {
    let world = AdversarialWorld::new();
    let control = world
        .runtime
        .as_ref()
        .expect("the owner root stays live")
        .owner_operation_control_for_test()
        .expect("operation control is available after sealing");
    let pause = control
        .arm_pause_once_for_test(SignalOwnerOperationBoundaryForTest::BeforeCanonicalMovement);
    let cancellation = SignalOwnerCancellationSource::new();
    let mutation = world.mutation.clone();
    let basis = world.child_basis.clone();
    let worker_cancellation = cancellation.clone();
    let (tx, rx) = mpsc::sync_channel(1);

    thread::scope(|scope| {
        scope.spawn(move || {
            let result = mutation
                .advance_exact(
                    worth_execution::ExecutionRequest::serial(
                        &execution_custody::operational_serial_request(),
                    ),
                    &basis,
                    &mut (),
                    &worker_cancellation.token(),
                    |_| Ok(()),
                )
                .map(|_| ())
                .map_err(|denial| format!("{denial:?}"));
            let _ = tx.send(result);
        });
        assert!(pause.wait_until_reached(PROGRESS_BOUND));
        cancellation.cancel();
        pause.release();
        assert!(matches!(
            rx.recv_timeout(PROGRESS_BOUND),
            Ok(Err(message)) if message.contains("CancelledNoMovement")
        ));
    });

    world
        .mutation
        .advance_exact(
            worth_execution::ExecutionRequest::serial(
                &execution_custody::operational_serial_request(),
            ),
            &world.root_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("a sibling remains immediately usable after cancellation");
}

#[test]
fn cancellation_after_canonical_movement_keeps_the_performed_advance() {
    let world = AdversarialWorld::new();
    let control = world
        .runtime
        .as_ref()
        .expect("the owner root stays live")
        .owner_operation_control_for_test()
        .expect("operation control is available after sealing");
    let pause = control
        .arm_pause_once_for_test(SignalOwnerOperationBoundaryForTest::AfterCanonicalMovement);
    let cancellation = SignalOwnerCancellationSource::new();
    let mutation = world.mutation.clone();
    let basis = world.child_basis.clone();
    let worker_cancellation = cancellation.clone();
    let (tx, rx) = mpsc::sync_channel(1);

    thread::scope(|scope| {
        scope.spawn(move || {
            let result = mutation
                .advance_exact(
                    worth_execution::ExecutionRequest::serial(
                        &execution_custody::operational_serial_request(),
                    ),
                    &basis,
                    &mut (),
                    &worker_cancellation.token(),
                    |_| Ok(()),
                )
                .map(|_| ())
                .map_err(|denial| format!("{denial:?}"));
            let _ = tx.send(result);
        });
        assert!(pause.wait_until_reached(PROGRESS_BOUND));
        cancellation.cancel();
        pause.release();
        assert_eq!(rx.recv_timeout(PROGRESS_BOUND), Ok(Ok(())));
    });
}

#[test]
fn cancellation_after_fork_movement_keeps_the_owner_issued_child() {
    let world = AdversarialWorld::new();
    let control = world
        .runtime
        .as_ref()
        .expect("the owner root stays live")
        .owner_operation_control_for_test()
        .expect("operation control is available after sealing");
    let pause = control
        .arm_pause_once_for_test(SignalOwnerOperationBoundaryForTest::AfterCanonicalMovement);
    let cancellation = SignalOwnerCancellationSource::new();
    let mutation = world.mutation.clone();
    let basis = world.root_basis.clone();
    let worker_cancellation = cancellation.clone();
    let (tx, rx) = mpsc::sync_channel(1);

    thread::scope(|scope| {
        scope.spawn(move || {
            let result = mutation
                .fork_exact(
                    worth_signal::facade::branch::validate_signal_branch_name("performed-fork")
                        .expect("the fork name is valid"),
                    &basis,
                    &worker_cancellation.token(),
                )
                .map(|outcome| outcome.created_branch().id)
                .map_err(|denial| format!("{denial:?}"));
            let _ = tx.send(result);
        });
        assert!(pause.wait_until_reached(PROGRESS_BOUND));
        cancellation.cancel();
        pause.release();
        let child = rx
            .recv_timeout(PROGRESS_BOUND)
            .expect("the fork resolves after the cancellation request")
            .expect("post-movement cancellation cannot erase the child");
        assert_ne!(child, world.root_basis.branch_id());
    });
}
