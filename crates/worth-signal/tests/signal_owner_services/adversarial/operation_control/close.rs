use std::sync::mpsc;
use std::thread;

use worth_signal::facade::branch::{
    SignalBranchAdvanceDenial, SignalOwnerCancellationSource, SignalOwnerLifecycleObservation,
    SignalOwnerOperationBoundary,
};

use super::super::world::{AdversarialWorld, PROGRESS_BOUND};

#[test]
fn close_fences_new_work_but_releases_an_already_admitted_operation() {
    let mut world = AdversarialWorld::new();
    let control = world
        .runtime
        .as_ref()
        .expect("the root remains live until the close request")
        .owner_operation_control()
        .expect("the control handle comes from the sealed owner");
    let pause = control.arm_pause_once(SignalOwnerOperationBoundary::BeforeCanonicalMovement);
    let mutation = world.mutation.clone();
    let basis = world.child_basis.clone();
    let (in_flight_tx, in_flight_rx) = mpsc::sync_channel(1);

    thread::scope(|scope| {
        scope.spawn(move || {
            let result = mutation
                .advance_exact(
                    &basis,
                    &mut (),
                    &SignalOwnerCancellationSource::new().token(),
                    |_| Ok(()),
                )
                .map(|_| ())
                .map_err(|denial| format!("{denial:?}"));
            let _ = in_flight_tx.send(result);
        });
        assert!(pause.wait_until_reached(PROGRESS_BOUND));

        world.close_root();
        assert_eq!(
            world.lifecycle.owner_lifecycle_observation(),
            SignalOwnerLifecycleObservation::Closing
        );

        let late = world.mutation.advance_exact(
            &world.root_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| panic!("late work must be denied before the caller callback"),
        );
        assert!(matches!(
            late,
            Err(SignalBranchAdvanceDenial::OwnerUnavailable(_))
        ));

        pause.release();
        assert_eq!(in_flight_rx.recv_timeout(PROGRESS_BOUND), Ok(Ok(())));
    });

    assert_eq!(
        world.lifecycle.owner_lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closed
    );
    assert!(world.basis.owner_service_cost_snapshot().is_err());
}

#[test]
fn dropping_a_public_pause_is_a_release_guard() {
    let world = AdversarialWorld::new();
    let control = world
        .runtime
        .as_ref()
        .expect("the root remains live")
        .owner_operation_control()
        .expect("the control handle comes from the sealed owner");
    let pause = control.arm_pause_once(SignalOwnerOperationBoundary::BeforeCanonicalMovement);
    let mutation = world.mutation.clone();
    let basis = world.child_basis.clone();
    let (tx, rx) = mpsc::sync_channel(1);

    thread::scope(|scope| {
        scope.spawn(move || {
            let result = mutation
                .advance_exact(
                    &basis,
                    &mut (),
                    &SignalOwnerCancellationSource::new().token(),
                    |_| Ok(()),
                )
                .map(|_| ())
                .map_err(|denial| format!("{denial:?}"));
            let _ = tx.send(result);
        });
        assert!(pause.wait_until_reached(PROGRESS_BOUND));
        drop(pause);
        assert_eq!(rx.recv_timeout(PROGRESS_BOUND), Ok(Ok(())));
    });
}

#[test]
fn close_batch_pause_is_releasable_and_cannot_admit_new_work() {
    let mut world = AdversarialWorld::new();
    let control = world
        .runtime
        .as_ref()
        .expect("the strong root remains until the close thread takes it")
        .owner_operation_control()
        .expect("the sealed owner issues operation control");
    let pause = control.arm_pause_once(SignalOwnerOperationBoundary::OwnerCloseBatch);
    let runtime = world
        .runtime
        .take()
        .expect("the close worker takes the only strong runtime root");

    thread::scope(|scope| {
        scope.spawn(move || drop(runtime));
        assert!(pause.wait_until_reached(PROGRESS_BOUND));
        assert_eq!(
            world.lifecycle.owner_lifecycle_observation(),
            SignalOwnerLifecycleObservation::Closing
        );
        let late = world.mutation.advance_exact(
            &world.root_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| panic!("close fencing must happen before the callback"),
        );
        assert!(matches!(
            late,
            Err(SignalBranchAdvanceDenial::OwnerUnavailable(_))
        ));
        pause.release();
    });

    assert_eq!(
        world.lifecycle.owner_lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closed
    );
}
