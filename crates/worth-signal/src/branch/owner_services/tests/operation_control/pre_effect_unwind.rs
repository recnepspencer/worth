use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::branch::owner_services::lifecycle_state::MAXIMUM_IN_FLIGHT_SIGNAL_OWNER_OPERATIONS;
use crate::branch::owner_services::operation_control::SignalOwnerOperationBoundary;
use crate::branch::owner_services::{SignalOwnerAdmissionDenial, SignalOwnerCancellationSource};
use crate::branch::{validate_signal_branch_name, SignalBranchBasisObservationDenial};

use super::super::runtime_root::runtime_with_two_branches;

#[test]
fn lifecycle_and_registry_faults_release_exact_pre_effect_capacity() {
    exercise_lifecycle_admission_fault();
    exercise_registry_lookup_fault();
    exercise_registry_reservation_fault();
}

fn exercise_lifecycle_admission_fault() {
    let (mut runtime, _, _, basis) = runtime_with_two_branches();
    let (port, _, _) = runtime.owner_port_slots().expect("admission owner seals");
    let owner = port.upgrade_owner().expect("admission owner remains live");
    owner
        .operation_control()
        .inject_panic_once(SignalOwnerOperationBoundary::OwnerLifecycleAdmission);
    assert!(catch_unwind(AssertUnwindSafe(|| owner.admit())).is_err());

    let admissions = (0..MAXIMUM_IN_FLIGHT_SIGNAL_OWNER_OPERATIONS)
        .map(|_| owner.admit().expect("fault returns every admission slot"))
        .collect::<Vec<_>>();
    assert!(matches!(
        owner.admit(),
        Err(SignalOwnerAdmissionDenial::OperationCapacityExhausted {
            maximum_in_flight_operations: MAXIMUM_IN_FLIGHT_SIGNAL_OWNER_OPERATIONS,
        })
    ));
    drop(admissions);
    drop(basis);
}

fn exercise_registry_lookup_fault() {
    let (mut runtime, _, branch, _) = runtime_with_two_branches();
    let (port, _, _) = runtime.owner_port_slots().expect("lookup owner seals");
    let owner = port.upgrade_owner().expect("lookup owner remains live");
    let admission = owner.admit().expect("lookup fault admits");
    let before = owner.cost_snapshot();
    owner
        .operation_control()
        .inject_panic_once(SignalOwnerOperationBoundary::BranchRegistryLookup);
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _ = owner.lookup_cell(&admission, branch.id);
    }))
    .is_err());
    assert_eq!(owner.live_count(), 2);
    assert_eq!(owner.reservation_count(), 0);
    assert_eq!(
        owner.cost_snapshot().branch_registry_lookups(),
        before.branch_registry_lookups(),
        "the lookup fault precedes registry contact"
    );
    assert_eq!(
        owner
            .lookup_cell(&admission, branch.id)
            .expect("one-shot lookup fault leaves membership healthy")
            .branch_id(),
        branch.id
    );
}

fn exercise_registry_reservation_fault() {
    let (mut runtime, sibling, source, basis) = runtime_with_two_branches();
    let (_, mutation, _) = runtime.owner_port_slots().expect("fork owner seals");
    let owner = mutation.upgrade_owner().expect("fork owner remains live");
    let admission = owner.admit().expect("fork reservation fault admits");
    let original_children = owner
        .metadata
        .branch_children(&admission, source.id)
        .expect("source lineage is observable");
    let before = owner.cost_snapshot();
    owner
        .operation_control()
        .inject_panic_once(SignalOwnerOperationBoundary::BranchRegistryReservation);
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _ = owner.reserve_fork_destination(
            &admission,
            &basis,
            validate_signal_branch_name("faulted-registry-reservation")
                .expect("identity validates"),
        );
    }))
    .is_err());
    assert_eq!(owner.live_count(), 2);
    assert_eq!(owner.reservation_count(), 0);
    assert_eq!(
        owner
            .metadata
            .branch_children(&admission, source.id)
            .expect("fault cannot install lineage"),
        original_children
    );
    assert_eq!(
        owner.cost_snapshot().branch_registry_reservations(),
        before.branch_registry_reservations()
    );
    let healthy = owner
        .reserve_fork_destination(
            &admission,
            &basis,
            validate_signal_branch_name("healthy-registry-reservation")
                .expect("retry identity validates"),
        )
        .expect("one-shot reservation fault returns capacity");
    assert!(healthy.branch().id.0 > source.id.0.max(sibling.id.0));
    drop(healthy);
    assert_eq!(owner.reservation_count(), 0);
}

#[test]
fn target_and_basis_preflight_faults_preserve_exact_no_movement_truth() {
    for boundary in [
        SignalOwnerOperationBoundary::TargetCellAdmission,
        SignalOwnerOperationBoundary::ExactBasisPreflight,
        SignalOwnerOperationBoundary::BeforeCanonicalMovement,
    ] {
        exercise_advance_pre_effect_fault(boundary);
    }
}

fn exercise_advance_pre_effect_fault(boundary: SignalOwnerOperationBoundary) {
    let (mut runtime, sibling, branch, basis) = runtime_with_two_branches();
    let (_, mutation, _) = runtime.owner_port_slots().expect("advance owner seals");
    let owner = mutation
        .upgrade_owner()
        .expect("advance owner remains live");
    let admission = owner.admit().expect("advance fault admits");
    let cell = owner
        .lookup_cell(&admission, branch.id)
        .expect("advance target is live");
    let sibling_cell = owner
        .lookup_cell(&admission, sibling.id)
        .expect("unrelated sibling is live");
    let ledger_before = owner.retention_ledger_observation();
    let cell_before = cell.cost_snapshot();
    owner.operation_control().inject_panic_once(boundary);
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let reservation = owner
            .reserve_advance_output(&admission, &cell)
            .expect("advance output reserves");
        let _ = reservation
            .advance::<(), (), _>(
                &basis,
                &mut (),
                &SignalOwnerCancellationSource::new().token(),
                |_| Ok(()),
            )
            .into_result();
    }))
    .is_err());
    let mut released = ledger_before.clone();
    released.next_lease_id += 1;
    assert_eq!(owner.retention_ledger_observation(), released);
    assert_eq!(cell.cost_snapshot().movements(), cell_before.movements());

    if boundary == SignalOwnerOperationBoundary::TargetCellAdmission {
        assert_eq!(cell.cost_snapshot(), cell_before);
        assert_eq!(cell.poison_recovery(), None);
        assert!(cell.observe_exact(&admission).is_ok());
    } else {
        assert!(matches!(
            cell.observe_exact(&admission),
            Err(SignalBranchBasisObservationDenial::QuarantinedBranch { branch_id })
                if branch_id == branch.id
        ));
        assert!(cell.poison_recovery().is_some());
    }
    assert!(
        sibling_cell.observe_exact(&admission).is_ok(),
        "{boundary:?} remains branch-local"
    );
}
