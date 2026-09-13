use crate::branch::owner_services::operation_control::SignalOwnerOperationBoundary;
use crate::branch::validate_signal_branch_name;

use super::super::super::super::SignalOwnerCancellationSource;
use super::super::world::{set_dependency, MutationWorld};
use super::run_paused;

#[derive(Clone, Copy)]
struct AdvanceBoundaryCost {
    boundary: SignalOwnerOperationBoundary,
    lookups: u64,
    retention_contacts: u64,
    target_contacts: u64,
    movements: u64,
}

#[test]
fn mutation_port_reaches_every_common_boundary_at_its_exact_cost_stage() {
    for expected in [
        AdvanceBoundaryCost {
            boundary: SignalOwnerOperationBoundary::OwnerLifecycleAdmission,
            lookups: 0,
            retention_contacts: 0,
            target_contacts: 0,
            movements: 0,
        },
        AdvanceBoundaryCost {
            boundary: SignalOwnerOperationBoundary::BranchRegistryLookup,
            lookups: 0,
            retention_contacts: 0,
            target_contacts: 0,
            movements: 0,
        },
        AdvanceBoundaryCost {
            boundary: SignalOwnerOperationBoundary::TargetCellAdmission,
            lookups: 1,
            retention_contacts: 1,
            target_contacts: 0,
            movements: 0,
        },
        AdvanceBoundaryCost {
            boundary: SignalOwnerOperationBoundary::ExactBasisPreflight,
            lookups: 1,
            retention_contacts: 1,
            target_contacts: 1,
            movements: 0,
        },
        AdvanceBoundaryCost {
            boundary: SignalOwnerOperationBoundary::BeforeCanonicalMovement,
            lookups: 1,
            retention_contacts: 1,
            target_contacts: 1,
            movements: 0,
        },
        AdvanceBoundaryCost {
            boundary: SignalOwnerOperationBoundary::AfterCanonicalMovement,
            lookups: 1,
            retention_contacts: 1,
            target_contacts: 1,
            movements: 1,
        },
        AdvanceBoundaryCost {
            boundary: SignalOwnerOperationBoundary::OutcomeConstruction,
            lookups: 1,
            retention_contacts: 1,
            target_contacts: 1,
            movements: 1,
        },
    ] {
        exercise_advance_boundary(expected);
    }
}

fn exercise_advance_boundary(expected: AdvanceBoundaryCost) {
    let world = MutationWorld::<()>::new();
    let before = world.owner.cost_snapshot();
    let port = world.port.clone();
    let basis = world.source_basis.clone();
    let derived = world.derived;
    let input_b = world.input_b;
    let result = run_paused(
        &world.owner.operation_control(),
        expected.boundary,
        move || {
            port.advance_exact(
                &basis,
                &mut (),
                &SignalOwnerCancellationSource::new().token(),
                |transaction| set_dependency(transaction, derived, input_b),
            )
            .map(|outcome| outcome.advanced_basis().observation().generation().get())
            .map_err(|denial| format!("{denial:?}"))
        },
        || {
            let at_boundary = world.owner.cost_snapshot();
            assert_eq!(
                at_boundary.owner_upgrade_attempts(),
                before.owner_upgrade_attempts() + 1
            );
            assert_eq!(
                at_boundary.branch_registry_lookups(),
                before.branch_registry_lookups() + expected.lookups
            );
            assert_eq!(
                at_boundary.retention_registry_contacts(),
                before.retention_registry_contacts() + expected.retention_contacts
            );
            assert_eq!(
                at_boundary.target_cell_contacts(),
                before.target_cell_contacts() + expected.target_contacts
            );
            assert_eq!(
                at_boundary.canonical_movements(),
                before.canonical_movements() + expected.movements
            );
        },
    );
    assert_eq!(result, Ok(1));
    assert_eq!(
        world.dependency_sources(&world.source_branch),
        vec![world.input_b]
    );
}

#[derive(Clone, Copy)]
struct ForkBoundaryCost {
    boundary: SignalOwnerOperationBoundary,
    reservations: u64,
    target_contacts: u64,
    movements: u64,
    source_captures: u64,
    preparations: u64,
    installations: u64,
    live: usize,
    pending: usize,
}

#[test]
fn mutation_port_reaches_every_fork_boundary_before_returning_exact_custody() {
    for expected in [
        ForkBoundaryCost {
            boundary: SignalOwnerOperationBoundary::BranchRegistryReservation,
            reservations: 0,
            target_contacts: 0,
            movements: 0,
            source_captures: 0,
            preparations: 0,
            installations: 0,
            live: 2,
            pending: 0,
        },
        ForkBoundaryCost {
            boundary: SignalOwnerOperationBoundary::ForkSourceCapture,
            reservations: 1,
            target_contacts: 1,
            movements: 0,
            source_captures: 1,
            preparations: 1,
            installations: 0,
            live: 2,
            pending: 1,
        },
        ForkBoundaryCost {
            boundary: SignalOwnerOperationBoundary::ForkDestinationInstallation,
            reservations: 1,
            target_contacts: 1,
            movements: 0,
            source_captures: 1,
            preparations: 1,
            installations: 1,
            live: 3,
            pending: 0,
        },
    ] {
        exercise_fork_boundary(expected);
    }
}

fn exercise_fork_boundary(expected: ForkBoundaryCost) {
    let world = MutationWorld::<()>::new();
    let before = world.owner.cost_snapshot();
    let port = world.port.clone();
    let basis = world.source_basis.clone();
    let result = run_paused(
        &world.owner.operation_control(),
        expected.boundary,
        move || {
            port.fork_exact(
                validate_signal_branch_name("controlled-custody-child").expect("name validates"),
                &basis,
                &SignalOwnerCancellationSource::new().token(),
            )
            .map(|outcome| {
                (
                    outcome.created_branch().clone(),
                    outcome.created_basis().owner_branch_id(),
                )
            })
            .map_err(|denial| format!("{denial:?}"))
        },
        || {
            let at_boundary = world.owner.cost_snapshot();
            assert_eq!(
                at_boundary.branch_registry_reservations(),
                before.branch_registry_reservations() + expected.reservations
            );
            assert_eq!(
                at_boundary.target_cell_contacts(),
                before.target_cell_contacts() + expected.target_contacts
            );
            assert_eq!(
                at_boundary.canonical_movements(),
                before.canonical_movements() + expected.movements
            );
            assert_eq!(
                at_boundary.fork_source_captures(),
                before.fork_source_captures() + expected.source_captures
            );
            assert_eq!(
                at_boundary.fork_destination_preparations(),
                before.fork_destination_preparations() + expected.preparations
            );
            assert_eq!(
                at_boundary.fork_destination_installations(),
                before.fork_destination_installations() + expected.installations
            );
            assert_eq!(world.owner.live_count(), expected.live);
            assert_eq!(world.owner.reservation_count(), expected.pending);
        },
    );
    let (handle, basis_id) = result.expect("controlled fork succeeds");
    assert_eq!(handle.id, basis_id);
    assert_eq!(handle.parent_branch_id, Some(world.source_branch.id));
    assert_eq!(handle.name, "controlled-custody-child");
}
