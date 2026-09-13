use crate::branch::owner_services::operation_control::SignalOwnerOperationBoundary;
use crate::branch::{
    validate_signal_branch_name, SignalBranchAdvanceDenial, SignalBranchForkOperationDenial,
    SignalBranchRestoreDenial, SignalBranchSnapshotCaptureDenial,
};

use super::super::super::super::SignalOwnerCancellationSource;
use super::super::world::{set_dependency, MutationWorld};
use super::{assert_no_pending_reservations, run_paused};

#[test]
fn before_movement_cancellation_denies_every_port_method_without_performed_truth() {
    cancel_fork_before_movement();
    cancel_advance_before_movement();
    cancel_capture_before_movement();
    cancel_restore_before_movement();
}

fn cancel_fork_before_movement() {
    let world = MutationWorld::<()>::new();
    let before = world.owner.cost_snapshot();
    let cancellation = SignalOwnerCancellationSource::new();
    let worker_cancellation = cancellation.clone();
    let port = world.port.clone();
    let basis = world.source_basis.clone();
    let denied = run_paused(
        &world.owner.operation_control(),
        SignalOwnerOperationBoundary::BeforeCanonicalMovement,
        move || {
            matches!(
                port.fork_exact(
                    validate_signal_branch_name("controlled-cancel-fork").expect("name validates"),
                    &basis,
                    &worker_cancellation.token(),
                ),
                Err(SignalBranchForkOperationDenial::CancelledNoMovement)
            )
        },
        || {
            assert_eq!(
                world.owner.cost_snapshot().canonical_movements(),
                before.canonical_movements()
            );
            cancellation.cancel();
        },
    );
    assert!(denied);
    assert_eq!(world.owner.live_count(), 2);
    assert_eq!(
        world.owner.cost_snapshot().canonical_movements(),
        before.canonical_movements()
    );
    assert_no_pending_reservations(&world);
}

fn cancel_advance_before_movement() {
    let world = MutationWorld::<()>::new();
    let before = world.owner.cost_snapshot();
    let cancellation = SignalOwnerCancellationSource::new();
    let worker_cancellation = cancellation.clone();
    let port = world.port.clone();
    let basis = world.source_basis.clone();
    let denied = run_paused(
        &world.owner.operation_control(),
        SignalOwnerOperationBoundary::BeforeCanonicalMovement,
        move || {
            matches!(
                port.advance_exact(&basis, &mut (), &worker_cancellation.token(), |_| panic!(
                    "cancelled advance must not enter its callback"
                ),),
                Err(SignalBranchAdvanceDenial::CancelledNoMovement)
            )
        },
        || cancellation.cancel(),
    );
    assert!(denied);
    assert_eq!(
        world.owner.cost_snapshot().canonical_movements(),
        before.canonical_movements()
    );
    assert_eq!(
        world.dependency_sources(&world.source_branch),
        vec![world.input_a]
    );
    assert_no_pending_reservations(&world);
}

fn cancel_capture_before_movement() {
    let world = MutationWorld::<()>::new();
    let before = world.owner.cost_snapshot();
    let cancellation = SignalOwnerCancellationSource::new();
    let worker_cancellation = cancellation.clone();
    let port = world.port.clone();
    let basis = world.source_basis.clone();
    let denied = run_paused(
        &world.owner.operation_control(),
        SignalOwnerOperationBoundary::BeforeCanonicalMovement,
        move || {
            matches!(
                port.capture_exact(&basis, &worker_cancellation.token()),
                Err(SignalBranchSnapshotCaptureDenial::CancelledNoMovement)
            )
        },
        || cancellation.cancel(),
    );
    assert!(denied);
    assert_eq!(
        world.owner.cost_snapshot().canonical_movements(),
        before.canonical_movements()
    );
    assert_no_pending_reservations(&world);
}

fn cancel_restore_before_movement() {
    let world = MutationWorld::<()>::new();
    let captured = world
        .port
        .capture_exact(
            &world.source_basis,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("restore fixture captures input A");
    let current = world
        .port
        .advance_exact(
            captured.captured_basis(),
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |transaction| set_dependency(transaction, world.derived, world.input_b),
        )
        .expect("restore fixture advances to input B");
    let before = world.owner.cost_snapshot();
    let cancellation = SignalOwnerCancellationSource::new();
    let worker_cancellation = cancellation.clone();
    let port = world.port.clone();
    let basis = current.advanced_basis().clone();
    let snapshot = captured.admitted_snapshot().clone();
    let denied = run_paused(
        &world.owner.operation_control(),
        SignalOwnerOperationBoundary::BeforeCanonicalMovement,
        move || {
            matches!(
                port.restore_exact(&basis, &snapshot, &worker_cancellation.token()),
                Err(SignalBranchRestoreDenial::CancelledNoMovement)
            )
        },
        || cancellation.cancel(),
    );
    assert!(denied);
    assert_eq!(
        world.owner.cost_snapshot().canonical_movements(),
        before.canonical_movements()
    );
    assert_eq!(
        world.dependency_sources(&world.source_branch),
        vec![world.input_b]
    );
    assert_no_pending_reservations(&world);
}
