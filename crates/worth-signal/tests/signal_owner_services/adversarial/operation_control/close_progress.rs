use std::sync::mpsc;
use std::thread;

use worth_proof::TransitionOutcome;
use worth_signal::facade::branch::{
    AdmittedSignalBranchBasis, AdmittedSignalBranchSnapshot, SignalBranchAdvanceDenial,
    SignalBranchBasisPort, SignalBranchLifecyclePort, SignalBranchMutationPort,
    SignalBranchRetirementReason, SignalOwnerCancellationSource, SignalOwnerLifecycleObservation,
    SignalOwnerOperationBoundary,
};

use super::super::world::{AdversarialWorld, PROGRESS_BOUND};

fn assert_late_admission_denied(
    mutation: &SignalBranchMutationPort<(), (), (), (), ()>,
    root_basis: &AdmittedSignalBranchBasis,
) {
    let late = mutation.advance_exact(
        root_basis,
        &mut (),
        &SignalOwnerCancellationSource::new().token(),
        |_| panic!("close must deny late work before the caller callback"),
    );
    assert!(matches!(
        late,
        Err(SignalBranchAdvanceDenial::OwnerUnavailable(_))
    ));
}

fn assert_closed(
    lifecycle: &SignalBranchLifecyclePort<(), (), ()>,
    basis: &SignalBranchBasisPort<(), (), ()>,
) {
    assert_eq!(
        lifecycle.owner_lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closed
    );
    assert!(basis.owner_service_cost_snapshot().is_err());
}

#[test]
fn close_drains_an_admitted_fork_while_fencing_late_work() {
    let mut world = AdversarialWorld::new();
    let control = world
        .runtime
        .as_ref()
        .expect("the root remains live until close is requested")
        .owner_operation_control()
        .expect("control is issued after sealing");
    let pause = control.arm_pause_once(SignalOwnerOperationBoundary::BeforeCanonicalMovement);
    let mutation = world.mutation.clone();
    let basis = world.root_basis.clone();
    let (tx, rx) = mpsc::sync_channel(1);
    thread::scope(|scope| {
        scope.spawn(move || {
            let result = mutation
                .fork_exact(
                    worth_signal::facade::branch::validate_signal_branch_name("close-fork")
                        .expect("the close identity is valid"),
                    &basis,
                    &SignalOwnerCancellationSource::new().token(),
                )
                .map(|_| ())
                .map_err(|denial| format!("{denial:?}"));
            let _ = tx.send(result);
        });
        assert!(pause.wait_until_reached(PROGRESS_BOUND));
        world.close_root();
        assert_eq!(
            world.lifecycle.owner_lifecycle_observation(),
            SignalOwnerLifecycleObservation::Closing
        );
        assert_late_admission_denied(&world.mutation, &world.root_basis);
        pause.release();
        assert_eq!(rx.recv_timeout(PROGRESS_BOUND), Ok(Ok(())));
    });
    assert_closed(&world.lifecycle, &world.basis);
}

#[test]
fn close_drains_an_admitted_snapshot_capture_while_fencing_late_work() {
    let mut world = AdversarialWorld::new();
    let control = world
        .runtime
        .as_ref()
        .expect("the root remains live until close is requested")
        .owner_operation_control()
        .expect("control is issued after sealing");
    let pause = control.arm_pause_once(SignalOwnerOperationBoundary::BeforeCanonicalMovement);
    let mutation = world.mutation.clone();
    let basis = world.child_basis.clone();
    let (tx, rx) = mpsc::sync_channel(1);
    thread::scope(|scope| {
        scope.spawn(move || {
            let result = mutation
                .capture_exact(&basis, &SignalOwnerCancellationSource::new().token())
                .map(|_| ())
                .map_err(|denial| format!("{denial:?}"));
            let _ = tx.send(result);
        });
        assert!(pause.wait_until_reached(PROGRESS_BOUND));
        world.close_root();
        assert_eq!(
            world.lifecycle.owner_lifecycle_observation(),
            SignalOwnerLifecycleObservation::Closing
        );
        assert_late_admission_denied(&world.mutation, &world.root_basis);
        pause.release();
        assert_eq!(rx.recv_timeout(PROGRESS_BOUND), Ok(Ok(())));
    });
    assert_closed(&world.lifecycle, &world.basis);
}

#[test]
fn close_drains_an_admitted_restore_while_fencing_late_work() {
    let mut world = AdversarialWorld::new();
    let (snapshot, captured_basis): (AdmittedSignalBranchSnapshot, _) = world
        .mutation
        .capture_exact(
            &world.child_basis,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("restore starts with a real owner-issued snapshot")
        .into_parts();
    let control = world
        .runtime
        .as_ref()
        .expect("the root remains live until close is requested")
        .owner_operation_control()
        .expect("control is issued after sealing");
    let pause = control.arm_pause_once(SignalOwnerOperationBoundary::BeforeCanonicalMovement);
    let mutation = world.mutation.clone();
    let basis = captured_basis.clone();
    let worker_snapshot = snapshot.clone();
    let (tx, rx) = mpsc::sync_channel(1);
    thread::scope(|scope| {
        scope.spawn(move || {
            let result = mutation
                .restore_exact(
                    &basis,
                    &worker_snapshot,
                    &SignalOwnerCancellationSource::new().token(),
                )
                .map(|_| ())
                .map_err(|denial| format!("{denial:?}"));
            let _ = tx.send(result);
        });
        assert!(pause.wait_until_reached(PROGRESS_BOUND));
        world.close_root();
        assert_eq!(
            world.lifecycle.owner_lifecycle_observation(),
            SignalOwnerLifecycleObservation::Closing
        );
        assert_late_admission_denied(&world.mutation, &world.root_basis);
        pause.release();
        assert_eq!(rx.recv_timeout(PROGRESS_BOUND), Ok(Ok(())));
    });
    drop(snapshot);
    assert_closed(&world.lifecycle, &world.basis);
}

#[test]
fn close_drains_an_admitted_retirement_while_fencing_late_work() {
    let world = AdversarialWorld::new();
    let reference = world
        .basis
        .issue_managed_branch_reference(&world.child_basis)
        .expect("the retirement target has managed reference custody");
    let target_basis = world
        .basis
        .observe_current(&reference)
        .expect("the retirement target has exact current custody");
    let AdversarialWorld {
        mut runtime,
        basis,
        mutation,
        lifecycle,
        root_basis,
        child_basis,
    } = world;
    drop(child_basis);
    let plan = match lifecycle
        .plan_retirement_exact(target_basis, SignalBranchRetirementReason::Superseded)
    {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("the retirement plan is owner-issued: {other:?}"),
    };
    let control = runtime
        .as_ref()
        .expect("the root remains live until close is requested")
        .owner_operation_control()
        .expect("control is issued after sealing");
    let pause = control.arm_pause_once(SignalOwnerOperationBoundary::BeforeCanonicalMovement);
    let worker_lifecycle = lifecycle.clone();
    let (tx, rx) = mpsc::sync_channel(1);
    thread::scope(|scope| {
        scope.spawn(move || {
            let result = worker_lifecycle
                .retire_exact(plan, &SignalOwnerCancellationSource::new().token())
                .into_result()
                .map(|_| ())
                .map_err(|denial| format!("{denial:?}"));
            let _ = tx.send(result);
        });
        assert!(pause.wait_until_reached(PROGRESS_BOUND));
        runtime.take();
        assert_eq!(
            lifecycle.owner_lifecycle_observation(),
            SignalOwnerLifecycleObservation::Closing
        );
        assert_late_admission_denied(&mutation, &root_basis);
        pause.release();
        assert_eq!(rx.recv_timeout(PROGRESS_BOUND), Ok(Ok(())));
    });
    assert_closed(&lifecycle, &basis);
}
