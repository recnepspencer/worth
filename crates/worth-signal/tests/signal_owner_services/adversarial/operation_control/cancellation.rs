use std::sync::mpsc;
use std::thread;

use worth_proof::TransitionOutcome;
use worth_signal::facade::branch::{
    AdmittedSignalBranchSnapshot, SignalBranchAdvanceDenial, SignalBranchForkOperationDenial,
    SignalBranchRestoreDenial, SignalBranchRetirementDenial, SignalBranchRetirementReason,
    SignalBranchSnapshotCaptureDenial, SignalOwnerCancellationSource, SignalOwnerOperationBoundary,
};

use super::super::world::{AdversarialWorld, PROGRESS_BOUND};

#[test]
fn pre_movement_cancellation_denies_every_cancellable_public_operation() {
    let world = AdversarialWorld::new();
    let snapshot_and_basis = world
        .mutation
        .capture_exact(
            &world.child_basis,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("the restore case begins with a real snapshot")
        .into_parts();
    let (snapshot, captured_basis): (AdmittedSignalBranchSnapshot, _) = snapshot_and_basis;
    let before = world
        .basis
        .owner_service_cost_snapshot()
        .expect("the owner is open")
        .canonical_movements();

    let advance_cancel = SignalOwnerCancellationSource::new();
    advance_cancel.cancel();
    assert!(matches!(
        world.mutation.advance_exact(
            &world.root_basis,
            &mut (),
            &advance_cancel.token(),
            |_| panic!("cancellation must precede the caller callback"),
        ),
        Err(SignalBranchAdvanceDenial::CancelledNoMovement)
    ));

    let fork_cancel = SignalOwnerCancellationSource::new();
    fork_cancel.cancel();
    assert!(matches!(
        world.mutation.fork_exact(
            worth_signal::facade::branch::validate_signal_branch_name("cancelled-child")
                .expect("the cancellation fixture name is valid"),
            &world.root_basis,
            &fork_cancel.token(),
        ),
        Err(SignalBranchForkOperationDenial::CancelledNoMovement)
    ));

    let capture_cancel = SignalOwnerCancellationSource::new();
    capture_cancel.cancel();
    assert!(matches!(
        world
            .mutation
            .capture_exact(&world.root_basis, &capture_cancel.token()),
        Err(SignalBranchSnapshotCaptureDenial::CancelledNoMovement)
    ));

    let restore_cancel = SignalOwnerCancellationSource::new();
    restore_cancel.cancel();
    assert!(matches!(
        world
            .mutation
            .restore_exact(&captured_basis, &snapshot, &restore_cancel.token(),),
        Err(SignalBranchRestoreDenial::CancelledNoMovement)
    ));

    let retirement_reference = world
        .basis
        .issue_managed_branch_reference(&world.child_basis)
        .expect("the cancellation retirement target has a managed reference");
    let retirement_basis = world
        .basis
        .observe_current(&retirement_reference)
        .expect("the cancellation retirement target has independent custody");
    drop(snapshot);
    drop(captured_basis);
    drop(world.child_basis);
    let plan = match world
        .lifecycle
        .plan_retirement_exact(retirement_basis, SignalBranchRetirementReason::Superseded)
    {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("the cancellation retirement plan is valid: {other:?}"),
    };
    let retire_cancel = SignalOwnerCancellationSource::new();
    retire_cancel.cancel();
    assert!(matches!(
        world.lifecycle.retire_exact(plan, &retire_cancel.token()),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::CancelledNoMovement)
    ));

    assert_eq!(
        world
            .basis
            .owner_service_cost_snapshot()
            .expect("pre-movement cancellation leaves the owner open")
            .canonical_movements(),
        before,
        "all cancelled operations are effect-free"
    );
    world
        .mutation
        .advance_exact(
            &world.root_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("an immediate healthy twin follows every cancellation");
}

#[test]
fn cancellation_at_the_pre_movement_park_is_still_effect_free() {
    let world = AdversarialWorld::new();
    let control = world
        .runtime
        .as_ref()
        .expect("the owner root stays live")
        .owner_operation_control()
        .expect("operation control is available after sealing");
    let pause = control.arm_pause_once(SignalOwnerOperationBoundary::BeforeCanonicalMovement);
    let cancellation = SignalOwnerCancellationSource::new();
    let mutation = world.mutation.clone();
    let basis = world.child_basis.clone();
    let worker_cancellation = cancellation.clone();
    let (tx, rx) = mpsc::sync_channel(1);

    thread::scope(|scope| {
        scope.spawn(move || {
            let result = mutation
                .advance_exact(&basis, &mut (), &worker_cancellation.token(), |_| Ok(()))
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
        .owner_operation_control()
        .expect("operation control is available after sealing");
    let pause = control.arm_pause_once(SignalOwnerOperationBoundary::AfterCanonicalMovement);
    let cancellation = SignalOwnerCancellationSource::new();
    let mutation = world.mutation.clone();
    let basis = world.child_basis.clone();
    let worker_cancellation = cancellation.clone();
    let (tx, rx) = mpsc::sync_channel(1);

    thread::scope(|scope| {
        scope.spawn(move || {
            let result = mutation
                .advance_exact(&basis, &mut (), &worker_cancellation.token(), |_| Ok(()))
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
        .owner_operation_control()
        .expect("operation control is available after sealing");
    let pause = control.arm_pause_once(SignalOwnerOperationBoundary::AfterCanonicalMovement);
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
