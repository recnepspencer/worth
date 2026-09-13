use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::branch::owner_services::operation_control::SignalOwnerOperationBoundary;
use crate::branch::validate_signal_branch_name;

use super::super::super::super::SignalOwnerCancellationSource;
use super::super::world::{set_dependency, MutationWorld};
use super::assert_no_pending_reservations;

#[test]
fn outcome_panics_preserve_every_performed_move_and_release_fork_custody() {
    panic_after_performed_fork();
    panic_after_performed_advance();
    panic_after_performed_capture();
    panic_after_performed_restore();
}

fn panic_after_performed_fork() {
    let world = MutationWorld::<()>::new();
    let current = world
        .port
        .advance_exact(
            &world.source_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |transaction| set_dependency(transaction, world.derived, world.input_b),
        )
        .expect("fork rollback fixture creates source journal truth");
    let admission = world.owner.admit().expect("fork rollback proof admits");
    let source_cell = world
        .owner
        .lookup_cell(&admission, world.source_branch.id)
        .expect("fork rollback source cell is live");
    let source_before = source_cell.fork_source_state_truth_after_fault();
    assert!(!source_before
        .mutation_ledger
        .structural_merge_journal()
        .records
        .is_empty());
    drop(admission);
    let before = world.owner.cost_snapshot();
    let destination_id = crate::state::SignalBranchId(
        world
            .source_branch
            .id
            .0
            .max(world.sibling_basis.owner_branch_id().0)
            + 1,
    );
    world
        .owner
        .operation_control()
        .inject_panic_once(SignalOwnerOperationBoundary::OutcomeConstruction);
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = world.port.fork_exact(
            validate_signal_branch_name("outcome-panic-fork").expect("name validates"),
            current.advanced_basis(),
            &SignalOwnerCancellationSource::new().token(),
        );
    }));

    assert!(panic.is_err());
    let after = world.owner.cost_snapshot();
    assert_eq!(after.canonical_movements(), before.canonical_movements());
    assert_eq!(
        after.fork_source_captures(),
        before.fork_source_captures() + 1
    );
    assert_eq!(
        after.fork_destination_installations(),
        before.fork_destination_installations() + 1
    );
    assert_eq!(world.owner.live_count(), 3);
    let destination = world
        .owner
        .lookup_cell(
            &world.owner.admit().expect("destination observation admits"),
            destination_id,
        )
        .expect("outcome panic preserves the performed destination");
    assert_eq!(destination.branch_id(), destination_id);
    let mut expected_source = source_before;
    expected_source
        .mutation_ledger
        .clear_all(world.source_branch.head_snapshot_id);
    assert_eq!(
        source_cell.fork_source_state_truth_after_fault(),
        expected_source,
        "outcome panic cannot erase the performed source capture"
    );
    assert_no_pending_reservations(&world);
}

fn panic_after_performed_advance() {
    let world = MutationWorld::<()>::new();
    let before = world.owner.cost_snapshot();
    world
        .owner
        .operation_control()
        .inject_panic_once(SignalOwnerOperationBoundary::OutcomeConstruction);
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = world.port.advance_exact(
            &world.source_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |transaction| set_dependency(transaction, world.derived, world.input_b),
        );
    }));

    assert!(panic.is_err());
    assert_eq!(
        world.owner.cost_snapshot().canonical_movements(),
        before.canonical_movements() + 1
    );
    assert_eq!(
        world.dependency_sources(&world.source_branch),
        vec![world.input_b]
    );
    assert_no_pending_reservations(&world);
}

fn panic_after_performed_capture() {
    let world = MutationWorld::<()>::new();
    let before = world.owner.cost_snapshot();
    world
        .owner
        .operation_control()
        .inject_panic_once(SignalOwnerOperationBoundary::OutcomeConstruction);
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = world.port.capture_exact(
            &world.source_basis,
            &SignalOwnerCancellationSource::new().token(),
        );
    }));

    assert!(panic.is_err());
    assert_eq!(
        world.owner.cost_snapshot().canonical_movements(),
        before.canonical_movements() + 1
    );
    assert!(
        world
            .canonical_handle(&world.source_branch)
            .head_snapshot_id
            .is_some(),
        "snapshot head movement survives outcome construction panic"
    );
    assert_no_pending_reservations(&world);
}

fn panic_after_performed_restore() {
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
    world
        .owner
        .operation_control()
        .inject_panic_once(SignalOwnerOperationBoundary::OutcomeConstruction);
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = world.port.restore_exact(
            current.advanced_basis(),
            captured.admitted_snapshot(),
            &SignalOwnerCancellationSource::new().token(),
        );
    }));

    assert!(panic.is_err());
    assert_eq!(
        world.owner.cost_snapshot().canonical_movements(),
        before.canonical_movements() + 1
    );
    assert_eq!(
        world.dependency_sources(&world.source_branch),
        vec![world.input_a]
    );
    assert_no_pending_reservations(&world);
}

#[test]
fn fork_installation_panic_preserves_performed_destination_and_releases_source_custody() {
    let world = MutationWorld::<()>::new();
    let current = world
        .port
        .advance_exact(
            &world.source_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |transaction| set_dependency(transaction, world.derived, world.input_b),
        )
        .expect("fork rollback fixture creates source journal truth");
    let admission = world.owner.admit().expect("fork rollback proof admits");
    let source_cell = world
        .owner
        .lookup_cell(&admission, world.source_branch.id)
        .expect("fork rollback source cell is live");
    let source_before = source_cell.fork_source_state_truth_after_fault();
    assert!(!source_before
        .mutation_ledger
        .structural_merge_journal()
        .records
        .is_empty());
    drop(admission);
    let before = world.owner.cost_snapshot();
    let destination_id = crate::state::SignalBranchId(
        world
            .source_branch
            .id
            .0
            .max(world.sibling_basis.owner_branch_id().0)
            + 1,
    );
    world
        .owner
        .operation_control()
        .inject_panic_once(SignalOwnerOperationBoundary::ForkDestinationInstallation);
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = world.port.fork_exact(
            validate_signal_branch_name("installation-panic-fork").expect("name validates"),
            current.advanced_basis(),
            &SignalOwnerCancellationSource::new().token(),
        );
    }));

    assert!(panic.is_err());
    let after = world.owner.cost_snapshot();
    assert_eq!(after.canonical_movements(), before.canonical_movements());
    assert_eq!(
        after.fork_source_captures(),
        before.fork_source_captures() + 1
    );
    assert_eq!(
        after.fork_destination_installations(),
        before.fork_destination_installations() + 1
    );
    assert_eq!(world.owner.live_count(), 3);
    let destination = world
        .owner
        .lookup_cell(
            &world.owner.admit().expect("destination observation admits"),
            destination_id,
        )
        .expect("installation panic preserves the performed destination");
    assert_eq!(destination.branch_id(), destination_id);
    let mut expected_source = source_before;
    expected_source
        .mutation_ledger
        .clear_all(world.source_branch.head_snapshot_id);
    assert_eq!(
        source_cell.fork_source_state_truth_after_fault(),
        expected_source,
        "installation panic cannot erase the performed source capture"
    );
    assert_no_pending_reservations(&world);
}
