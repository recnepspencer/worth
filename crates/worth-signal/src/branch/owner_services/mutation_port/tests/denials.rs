use crate::branch::{
    validate_signal_branch_name, SignalBranchAdvanceDenial, SignalBranchForkOperationDenial,
    SignalBranchRestoreDenial, SignalBranchSnapshotCaptureDenial,
};

use super::super::super::SignalOwnerCancellationSource;
use super::world::{set_dependency, MutationWorld};

#[test]
fn stale_matrix_cleans_every_reservation_and_allows_healthy_follow_up() {
    let world = MutationWorld::<()>::new();
    let cancellation = SignalOwnerCancellationSource::new();
    let captured = world
        .port
        .capture_exact(&world.source_basis, &cancellation.token())
        .expect("snapshot setup performs");
    let mut runtime_ctx = ();
    let current = world
        .port
        .advance_exact(
            captured.captured_basis(),
            &mut runtime_ctx,
            &cancellation.token(),
            |transaction| set_dependency(transaction, world.derived, world.input_b),
        )
        .expect("the setup movement makes the captured basis stale");
    let stale = captured.captured_basis();
    let retained_before = world
        .owner
        .admitted_or_reserved_retention_count(world.source_branch.id);
    let ledger_before = world.owner.retention_ledger_observation();

    assert!(matches!(
        world.port.fork_exact(
            validate_signal_branch_name("stale-fork").expect("name validates"),
            stale,
            &cancellation.token(),
        ),
        Err(SignalBranchForkOperationDenial::BasisMismatch { .. })
    ));
    assert!(matches!(
        world
            .port
            .advance_exact(stale, &mut runtime_ctx, &cancellation.token(), |_| Ok(())),
        Err(SignalBranchAdvanceDenial::BasisMismatch { .. })
    ));
    assert!(matches!(
        world.port.capture_exact(stale, &cancellation.token()),
        Err(SignalBranchSnapshotCaptureDenial::BasisMismatch { .. })
    ));
    assert!(matches!(
        world
            .port
            .restore_exact(stale, captured.admitted_snapshot(), &cancellation.token()),
        Err(SignalBranchRestoreDenial::BasisMismatch { .. })
    ));
    assert_eq!(
        world
            .owner
            .admitted_or_reserved_retention_count(world.source_branch.id),
        retained_before,
        "stale output reservations return every pending slot"
    );
    assert_eq!(world.owner.reservation_count(), 0);
    let ledger_after = world.owner.retention_ledger_observation();
    assert_eq!(ledger_after.used_capacity, ledger_before.used_capacity);
    assert_eq!(ledger_after.reserved_admitted_lease_count, 0);

    let healthy = world
        .port
        .advance_exact(
            current.advanced_basis(),
            &mut runtime_ctx,
            &cancellation.token(),
            |_| Ok(()),
        )
        .expect("the current basis remains usable after every stale denial");
    assert_eq!(healthy.advanced_basis().observation().generation().get(), 3);
}

#[test]
fn pre_movement_cancellation_matrix_is_no_effect_and_releases_capacity() {
    let world = MutationWorld::<()>::new();
    let captured = world
        .port
        .capture_exact(
            &world.source_basis,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("snapshot setup performs");
    let current = captured.captured_basis();
    let retained_before = world
        .owner
        .admitted_or_reserved_retention_count(world.source_branch.id);
    let ledger_before = world.owner.retention_ledger_observation();
    let before = world.owner.cost_snapshot();
    let cancelled = SignalOwnerCancellationSource::new();
    cancelled.cancel();

    assert!(matches!(
        world.port.fork_exact(
            validate_signal_branch_name("cancelled-fork").expect("name validates"),
            current,
            &cancelled.token(),
        ),
        Err(SignalBranchForkOperationDenial::CancelledNoMovement)
    ));
    assert!(matches!(
        world
            .port
            .advance_exact(current, &mut (), &cancelled.token(), |_| Ok(())),
        Err(SignalBranchAdvanceDenial::CancelledNoMovement)
    ));
    assert!(matches!(
        world.port.capture_exact(current, &cancelled.token()),
        Err(SignalBranchSnapshotCaptureDenial::CancelledNoMovement)
    ));
    assert!(matches!(
        world
            .port
            .restore_exact(current, captured.admitted_snapshot(), &cancelled.token()),
        Err(SignalBranchRestoreDenial::CancelledNoMovement)
    ));
    let after = world.owner.cost_snapshot();
    assert_eq!(after.canonical_movements(), before.canonical_movements());
    assert_eq!(world.owner.reservation_count(), 0);
    let ledger_after = world.owner.retention_ledger_observation();
    assert_eq!(ledger_after.used_capacity, ledger_before.used_capacity);
    assert_eq!(ledger_after.reserved_admitted_lease_count, 0);
    assert_eq!(
        world
            .owner
            .admitted_or_reserved_retention_count(world.source_branch.id),
        retained_before
    );
    world
        .port
        .advance_exact(
            current,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("an immediate healthy twin still performs");
}

#[test]
fn advance_cancellation_requested_after_cutoff_cannot_erase_performed_truth() {
    let world = MutationWorld::<()>::new();
    let cancellation = SignalOwnerCancellationSource::new();
    let callback_cancellation = cancellation.clone();
    let before = world.owner.cost_snapshot();
    let outcome = world
        .port
        .advance_exact(
            &world.source_basis,
            &mut (),
            &cancellation.token(),
            |transaction| {
                callback_cancellation.cancel();
                set_dependency(transaction, world.derived, world.input_b)
            },
        )
        .expect("cancellation after the cell movement permit remains performed");
    let after = world.owner.cost_snapshot();
    assert_eq!(
        after.canonical_movements(),
        before.canonical_movements() + 1
    );
    assert_eq!(outcome.advanced_basis().observation().generation().get(), 1);
    assert_eq!(
        world.dependency_sources(&world.source_branch),
        vec![world.input_b]
    );
}

#[test]
fn output_retention_exhaustion_denies_every_method_pre_effect_then_recovers() {
    let world = MutationWorld::<()>::new();
    let captured = world
        .port
        .capture_exact(
            &world.source_basis,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("snapshot setup performs");
    let current = captured.captured_basis();
    let admission = world.owner.admit().expect("capacity setup admits");
    let ledger = world.owner.retention_ledger_observation();
    let available = ledger.maximum_active_leases - ledger.used_capacity;
    let capacity = world
        .owner
        .reserve_admitted_output_slots_for_test(&admission, world.source_branch.id, available)
        .expect("the test occupies exactly the remaining admitted-output capacity");
    let before = world.owner.cost_snapshot();

    assert!(matches!(
        world.port.fork_exact(
            validate_signal_branch_name("retention-full-fork").expect("name validates"),
            current,
            &SignalOwnerCancellationSource::new().token(),
        ),
        Err(SignalBranchForkOperationDenial::RetentionUnavailable { .. })
    ));
    assert!(matches!(
        world.port.advance_exact(
            current,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        ),
        Err(SignalBranchAdvanceDenial::RetentionUnavailable { .. })
    ));
    assert!(matches!(
        world
            .port
            .capture_exact(current, &SignalOwnerCancellationSource::new().token()),
        Err(SignalBranchSnapshotCaptureDenial::RetentionUnavailable { .. })
    ));
    assert!(matches!(
        world.port.restore_exact(
            current,
            captured.admitted_snapshot(),
            &SignalOwnerCancellationSource::new().token(),
        ),
        Err(SignalBranchRestoreDenial::RetentionUnavailable { .. })
    ));
    let after = world.owner.cost_snapshot();
    assert_eq!(after.canonical_movements(), before.canonical_movements());
    assert_eq!(after.target_cell_contacts(), before.target_cell_contacts());
    drop(capacity);
    drop(admission);
    world
        .port
        .advance_exact(
            current,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("capacity release permits an immediate healthy operation");
}

#[test]
fn weak_port_owner_loss_is_stable_for_all_four_methods() {
    let world = MutationWorld::<()>::new();
    let captured = world
        .port
        .capture_exact(
            &world.source_basis,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("snapshot setup performs");
    let port = world.port.clone();
    let current = captured.captured_basis().clone();
    let snapshot = captured.admitted_snapshot().clone();
    drop(world.owner);
    drop(world.runtime);

    assert!(matches!(
        port.fork_exact(
            validate_signal_branch_name("lost-owner-fork").expect("name validates"),
            &current,
            &SignalOwnerCancellationSource::new().token(),
        ),
        Err(SignalBranchForkOperationDenial::OwnerUnavailable(_))
    ));
    assert!(matches!(
        port.advance_exact(
            &current,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        ),
        Err(SignalBranchAdvanceDenial::OwnerUnavailable(_))
    ));
    assert!(matches!(
        port.capture_exact(&current, &SignalOwnerCancellationSource::new().token()),
        Err(SignalBranchSnapshotCaptureDenial::OwnerUnavailable(_))
    ));
    assert!(matches!(
        port.restore_exact(
            &current,
            &snapshot,
            &SignalOwnerCancellationSource::new().token(),
        ),
        Err(SignalBranchRestoreDenial::OwnerUnavailable(_))
    ));
}
