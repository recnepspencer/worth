use crate::branch::owner_services::operation_control::SignalOwnerOperationBoundary;
use crate::branch::validate_signal_branch_name;

use super::super::super::super::SignalOwnerCancellationSource;
use super::super::world::{set_dependency, MutationWorld};
use super::{assert_no_pending_reservations, run_paused};

#[test]
fn cancellation_after_movement_cannot_erase_any_performed_port_truth() {
    cancel_after_performed_fork();
    cancel_after_performed_advance();
    cancel_after_performed_capture();
    cancel_after_performed_restore();
}

fn cancel_after_performed_fork() {
    let world = MutationWorld::<()>::new();
    let before = world.owner.cost_snapshot();
    let cancellation = SignalOwnerCancellationSource::new();
    let worker_cancellation = cancellation.clone();
    let port = world.port.clone();
    let basis = world.source_basis.clone();
    let performed = run_paused(
        &world.owner.operation_control(),
        SignalOwnerOperationBoundary::AfterCanonicalMovement,
        move || {
            port.fork_exact(
                validate_signal_branch_name("post-cutoff-fork").expect("name validates"),
                &basis,
                &worker_cancellation.token(),
            )
            .map(|outcome| outcome.created_basis().owner_branch_id())
            .map_err(|denial| format!("{denial:?}"))
        },
        || {
            assert_eq!(
                world.owner.cost_snapshot().canonical_movements(),
                before.canonical_movements()
            );
            assert_eq!(
                world.owner.cost_snapshot().fork_source_captures(),
                before.fork_source_captures() + 1
            );
            assert_eq!(world.owner.live_count(), 2);
            assert_eq!(world.owner.reservation_count(), 1);
            cancellation.cancel();
        },
    )
    .expect("performed fork ignores later cancellation");
    assert_ne!(performed, world.source_branch.id);
    assert_eq!(world.owner.live_count(), 3);
    assert_eq!(
        world.owner.cost_snapshot().canonical_movements(),
        before.canonical_movements()
    );
    assert_eq!(
        world.owner.cost_snapshot().fork_destination_installations(),
        before.fork_destination_installations() + 1
    );
    assert_no_pending_reservations(&world);
}

fn cancel_after_performed_advance() {
    let world = MutationWorld::<()>::new();
    let before = world.owner.cost_snapshot();
    let cancellation = SignalOwnerCancellationSource::new();
    let worker_cancellation = cancellation.clone();
    let port = world.port.clone();
    let basis = world.source_basis.clone();
    let derived = world.derived;
    let input_b = world.input_b;
    let generation = run_paused(
        &world.owner.operation_control(),
        SignalOwnerOperationBoundary::AfterCanonicalMovement,
        move || {
            port.advance_exact(
                &basis,
                &mut (),
                &worker_cancellation.token(),
                |transaction| set_dependency(transaction, derived, input_b),
            )
            .map(|outcome| outcome.advanced_basis().observation().generation().get())
            .map_err(|denial| format!("{denial:?}"))
        },
        || {
            assert_eq!(
                world.owner.cost_snapshot().canonical_movements(),
                before.canonical_movements() + 1
            );
            cancellation.cancel();
        },
    )
    .expect("performed advance ignores later cancellation");
    assert_eq!(generation, 1);
    assert_eq!(
        world.dependency_sources(&world.source_branch),
        vec![world.input_b]
    );
    assert_no_pending_reservations(&world);
}

fn cancel_after_performed_capture() {
    let world = MutationWorld::<()>::new();
    let before = world.owner.cost_snapshot();
    let cancellation = SignalOwnerCancellationSource::new();
    let worker_cancellation = cancellation.clone();
    let port = world.port.clone();
    let basis = world.source_basis.clone();
    let generation = run_paused(
        &world.owner.operation_control(),
        SignalOwnerOperationBoundary::AfterCanonicalMovement,
        move || {
            port.capture_exact(&basis, &worker_cancellation.token())
                .map(|outcome| outcome.captured_basis().observation().generation().get())
                .map_err(|denial| format!("{denial:?}"))
        },
        || {
            assert_eq!(
                world.owner.cost_snapshot().canonical_movements(),
                before.canonical_movements() + 1
            );
            cancellation.cancel();
        },
    )
    .expect("performed capture ignores later cancellation");
    assert_eq!(generation, 1);
    assert_no_pending_reservations(&world);
}

fn cancel_after_performed_restore() {
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
    let generation = run_paused(
        &world.owner.operation_control(),
        SignalOwnerOperationBoundary::AfterCanonicalMovement,
        move || {
            port.restore_exact(&basis, &snapshot, &worker_cancellation.token())
                .map(|basis| basis.observation().generation().get())
                .map_err(|denial| format!("{denial:?}"))
        },
        || {
            assert_eq!(
                world.owner.cost_snapshot().canonical_movements(),
                before.canonical_movements() + 1
            );
            cancellation.cancel();
        },
    )
    .expect("performed restore ignores later cancellation");
    assert_eq!(generation, 3);
    assert_eq!(
        world.dependency_sources(&world.source_branch),
        vec![world.input_a]
    );
    assert_no_pending_reservations(&world);
}
