use crate::branch::{
    admit_runtime_signal_branch_observation, AdmittedSignalBranchSnapshot,
    SignalBranchSnapshotCaptureDenial,
};
use crate::data::graph::SignalGraph;
use crate::logic::transaction::SignalRuntime;

use super::super::SignalOwnerCancellationSource;
use super::runtime_root::runtime_with_two_branches;

type SnapshotCapacityRuntime = SignalRuntime<(), (), (), (), ()>;

fn runtime_with_snapshot_capacity(
    maximum_stored_snapshots: usize,
) -> (
    SnapshotCapacityRuntime,
    crate::state::SignalBranchHandle,
    crate::branch::AdmittedSignalBranchBasis,
) {
    let runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .maximum_stored_branch_snapshots(maximum_stored_snapshots)
        .build();
    let branch = runtime.current_branch();
    let basis = runtime
        .observe_signal_branch_basis(branch.clone())
        .expect("the real runtime admits its initial branch");
    (runtime, branch, basis)
}

pub(super) fn caught_panic_message(panic: &(dyn std::any::Any + Send)) -> Option<&str> {
    panic
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| panic.downcast_ref::<String>().map(String::as_str))
}

#[test]
fn exact_snapshot_and_restore_contracts_move_one_cell_and_install_metadata_between_locks() {
    let (mut runtime, _, branch, starting_basis) = runtime_with_two_branches();
    let (_, mutation, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = mutation.upgrade_owner().expect("owner remains live");
    let admission = owner.admit().expect("operation admits");
    let cell = owner
        .lookup_cell(&admission, branch.id)
        .expect("target cell is live");
    let cancellation = SignalOwnerCancellationSource::new();
    let reservation = owner
        .metadata
        .reserve_snapshot(&admission, &cell)
        .expect("snapshot capacity reserves before cell work");
    let capture = cell
        .capture_snapshot_exact(&starting_basis, reservation, &cancellation.token())
        .expect("exact cell snapshot performs");
    let (capture_snapshot, capture_observation) = capture.into_parts();
    let snapshot_a_id = capture_snapshot.meta.snapshot_id;
    assert_eq!(capture_observation.generation().get(), 1);
    assert_eq!(
        capture_observation
            .target()
            .as_basis()
            .and_then(|target| target.snapshot_id()),
        Some(snapshot_a_id.0)
    );
    let captured_basis = admit_runtime_signal_branch_observation(
        capture_observation,
        branch.id,
        owner
            .acquire_admitted_retention(&admission, branch.id)
            .expect("captured basis retains its branch"),
    );
    let admitted_snapshot_a = AdmittedSignalBranchSnapshot::owner_issued(
        owner.runtime_instance_id(),
        capture_snapshot,
        owner
            .acquire_admitted_retention(&admission, branch.id)
            .expect("snapshot authority retains its branch"),
    );
    let snapshot_state = owner
        .metadata
        .snapshot_state(&admission, &admitted_snapshot_a)
        .expect("snapshot lookup is owner-admitted")
        .expect("snapshot semantic state is installed");
    let reservation = owner
        .metadata
        .reserve_snapshot(&admission, &cell)
        .expect("second snapshot capacity reserves");
    let capture_b = cell
        .capture_snapshot_exact(&captured_basis, reservation, &cancellation.token())
        .expect("second exact snapshot performs");
    let (capture_b_snapshot, capture_b_observation) = capture_b.into_parts();
    let snapshot_b_id = capture_b_snapshot.meta.snapshot_id;
    assert_ne!(snapshot_b_id, snapshot_a_id);
    let basis_b = admit_runtime_signal_branch_observation(
        capture_b_observation,
        branch.id,
        owner
            .acquire_admitted_retention(&admission, branch.id)
            .expect("second captured basis retains its branch"),
    );
    let admitted_snapshot_b = AdmittedSignalBranchSnapshot::owner_issued(
        owner.runtime_instance_id(),
        capture_b_snapshot,
        owner
            .acquire_admitted_retention(&admission, branch.id)
            .expect("second snapshot authority retains its branch"),
    );
    let snapshot_state_b = owner
        .metadata
        .snapshot_state(&admission, &admitted_snapshot_b)
        .expect("second snapshot lookup is owner-admitted")
        .expect("second snapshot semantic state is installed");
    let before_mismatch = cell.cost_snapshot();
    let mismatch = cell.restore_exact(
        &admission,
        &basis_b,
        &admitted_snapshot_b,
        snapshot_state,
        &cancellation.token(),
    );
    assert!(matches!(
        mismatch,
        Err(crate::branch::SignalBranchRestoreDenial::UnavailableSnapshot {
            branch_id,
            snapshot_id,
        }) if branch_id == branch.id && snapshot_id == snapshot_b_id
    ));
    assert_eq!(cell.cost_snapshot(), before_mismatch);
    let restore = cell
        .restore_exact(
            &admission,
            &basis_b,
            &admitted_snapshot_b,
            snapshot_state_b,
            &cancellation.token(),
        )
        .expect("exact cell restore performs");
    let restore_observation = restore.into_observation();
    assert_eq!(restore_observation.generation().get(), 3);
    assert_eq!(
        restore_observation
            .target()
            .as_basis()
            .and_then(|target| target.restore_snapshot_id()),
        Some(snapshot_b_id.0)
    );
    assert_eq!(cell.cost_snapshot().movements(), 3);
}

#[test]
fn snapshot_reservation_restores_capacity_after_denial_drop_and_unwind() {
    let (mut runtime, branch, starting_basis) = runtime_with_snapshot_capacity(1);
    let (_, mutation, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = mutation.upgrade_owner().expect("owner remains live");
    let admission = owner.admit().expect("snapshot operation admits");
    let cell = owner
        .lookup_cell(&admission, branch.id)
        .expect("target cell is live");

    let reservation = owner
        .metadata
        .reserve_snapshot(&admission, &cell)
        .expect("one snapshot slot reserves");
    assert!(matches!(
        owner.metadata.reserve_snapshot(&admission, &cell),
        Err(
            SignalBranchSnapshotCaptureDenial::SnapshotCapacityExhausted {
                maximum_stored_snapshots: 1,
            }
        )
    ));
    assert_eq!(cell.cost_snapshot(), Default::default());
    drop(reservation);

    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _reservation = owner
            .metadata
            .reserve_snapshot(&admission, &cell)
            .expect("the dropped reservation restored capacity");
        panic!("exercise snapshot reservation unwind cleanup");
    }));
    let unwind = unwind.expect_err("the reservation scope must unwind");
    assert_eq!(
        caught_panic_message(unwind.as_ref()),
        Some("exercise snapshot reservation unwind cleanup")
    );

    let cancellation = SignalOwnerCancellationSource::new();
    let mut runtime_context = ();
    let advanced = cell
        .advance_exact::<(), (), _>(
            &admission,
            &starting_basis,
            &mut runtime_context,
            &cancellation.token(),
            |_| Ok(()),
        )
        .expect("the real owner cell advances before the stale capture twin");
    let (advanced_observation, _) = advanced.into_parts();
    let advanced_basis = admit_runtime_signal_branch_observation(
        advanced_observation,
        branch.id,
        owner
            .acquire_admitted_retention(&admission, branch.id)
            .expect("the advanced basis retains its branch"),
    );
    let denied_reservation = owner
        .metadata
        .reserve_snapshot(&admission, &cell)
        .expect("the stale capture reserves before exact comparison");
    let before_stale_capture = cell.cost_snapshot();
    assert!(matches!(
        cell.capture_snapshot_exact(&starting_basis, denied_reservation, &cancellation.token(),),
        Err(SignalBranchSnapshotCaptureDenial::BasisMismatch { .. })
    ));
    let after_stale_capture = cell.cost_snapshot();
    assert_eq!(
        after_stale_capture.contacts(),
        before_stale_capture.contacts() + 1
    );
    assert_eq!(
        after_stale_capture.movements(),
        before_stale_capture.movements()
    );

    let reservation = owner
        .metadata
        .reserve_snapshot(&admission, &cell)
        .expect("stale denial restored snapshot capacity after cell release");
    let capture = cell
        .capture_snapshot_exact(&advanced_basis, reservation, &cancellation.token())
        .expect("the reserved snapshot installs through the real owner cell");
    assert_eq!(capture.observation().generation().get(), 2);
    assert!(matches!(
        owner.metadata.reserve_snapshot(&admission, &cell),
        Err(
            SignalBranchSnapshotCaptureDenial::SnapshotCapacityExhausted {
                maximum_stored_snapshots: 1,
            }
        )
    ));
}

#[test]
fn snapshot_reservation_rejects_a_different_owner_before_target_contact() {
    let (mut runtime_a, branch_a, _) = runtime_with_snapshot_capacity(1);
    let (mut runtime_b, branch_b, basis_b) = runtime_with_snapshot_capacity(1);
    let (_, mutation_a, _) = runtime_a.owner_port_slots().expect("owner A seals");
    let (_, mutation_b, _) = runtime_b.owner_port_slots().expect("owner B seals");
    let owner_a = mutation_a.upgrade_owner().expect("owner A remains live");
    let owner_b = mutation_b.upgrade_owner().expect("owner B remains live");
    let admission_a = owner_a.admit().expect("owner A admits reservation");
    let admission_b = owner_b.admit().expect("owner B admits capture");
    let cell_a = owner_a
        .lookup_cell(&admission_a, branch_a.id)
        .expect("owner A target cell is live");
    let cell_b = owner_b
        .lookup_cell(&admission_b, branch_b.id)
        .expect("owner B target cell is live");
    let reservation_a = owner_a
        .metadata
        .reserve_snapshot(&admission_a, &cell_a)
        .expect("owner A reserves its snapshot capacity");
    let before_b = cell_b.cost_snapshot();
    let cancellation = SignalOwnerCancellationSource::new();

    assert!(matches!(
        cell_b.capture_snapshot_exact(&basis_b, reservation_a, &cancellation.token(),),
        Err(SignalBranchSnapshotCaptureDenial::OwnerUnavailable(_))
    ));
    assert_eq!(cell_b.cost_snapshot(), before_b);
    let reusable_a = owner_a
        .metadata
        .reserve_snapshot(&admission_a, &cell_a)
        .expect("cross-owner denial restores owner A capacity");
    drop(reusable_a);
    let reservation_b = owner_b
        .metadata
        .reserve_snapshot(&admission_b, &cell_b)
        .expect("owner B reserves after the foreign attempt");
    let healthy_b = cell_b
        .capture_snapshot_exact(&basis_b, reservation_b, &cancellation.token())
        .expect("owner B captures through its own originating admission");
    assert_eq!(healthy_b.observation().generation().get(), 1);
}

#[test]
fn snapshot_reservation_rejects_a_sibling_cell_incarnation_before_contact() {
    let (mut runtime, source, sibling, sibling_basis) = runtime_with_two_branches();
    let (_, mutation, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = mutation.upgrade_owner().expect("owner remains live");
    let admission = owner.admit().expect("snapshot operation admits");
    let source_cell = owner
        .lookup_cell(&admission, source.id)
        .expect("source cell is live");
    let sibling_cell = owner
        .lookup_cell(&admission, sibling.id)
        .expect("sibling cell is live");
    let reservation = owner
        .metadata
        .reserve_snapshot(&admission, &source_cell)
        .expect("source reserves exact snapshot custody");
    assert_eq!(owner.metadata.pending_snapshot_reservation_count(), 1);
    let sibling_before = sibling_cell.cost_snapshot();
    let ledger_before = owner.retention_ledger_observation();

    assert!(matches!(
        sibling_cell.capture_snapshot_exact(
            &sibling_basis,
            reservation,
            &SignalOwnerCancellationSource::new().token(),
        ),
        Err(SignalBranchSnapshotCaptureDenial::OwnerCellMisuse { branch_id })
            if branch_id == sibling.id
    ));
    assert_eq!(sibling_cell.cost_snapshot(), sibling_before);
    assert_eq!(owner.metadata.pending_snapshot_reservation_count(), 0);
    assert_eq!(owner.retention_ledger_observation(), ledger_before);

    let healthy = owner
        .metadata
        .reserve_snapshot(&admission, &sibling_cell)
        .expect("the denied custody returns exact snapshot capacity");
    let captured = sibling_cell
        .capture_snapshot_exact(
            &sibling_basis,
            healthy,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("the sibling captures with its own cell incarnation");
    assert_eq!(captured.snapshot().meta.branch_id, sibling.id);
    assert_eq!(owner.metadata.pending_snapshot_reservation_count(), 0);
}

#[test]
fn snapshot_reservation_rejects_a_replaced_same_id_cell_incarnation() {
    let (mut runtime, _, branch, basis) = runtime_with_two_branches();
    let (_, mutation, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = mutation.upgrade_owner().expect("owner remains live");
    let admission = owner.admit().expect("snapshot operation admits");
    let original = owner
        .lookup_cell(&admission, branch.id)
        .expect("original cell is live");
    let reservation = owner
        .metadata
        .reserve_snapshot(&admission, &original)
        .expect("reservation binds the original incarnation");
    owner
        .replace_branch_incarnation_for_test(&admission, branch.id)
        .expect("the canonical registry replaces the exact cell");
    let replacement = owner
        .lookup_cell(&admission, branch.id)
        .expect("replacement cell is live");
    assert_ne!(original.incarnation(), replacement.incarnation());
    let before = replacement.cost_snapshot();

    assert!(matches!(
        replacement.capture_snapshot_exact(
            &basis,
            reservation,
            &SignalOwnerCancellationSource::new().token(),
        ),
        Err(SignalBranchSnapshotCaptureDenial::OwnerCellMisuse { branch_id })
            if branch_id == branch.id
    ));
    assert_eq!(replacement.cost_snapshot(), before);
    assert_eq!(owner.metadata.pending_snapshot_reservation_count(), 0);
    let healthy = owner
        .metadata
        .reserve_snapshot(&admission, &replacement)
        .expect("replaced-custody denial returns capacity");
    assert_eq!(
        replacement
            .capture_snapshot_exact(
                &basis,
                healthy,
                &SignalOwnerCancellationSource::new().token(),
            )
            .expect("replacement cell captures with replacement custody")
            .snapshot()
            .meta
            .branch_id,
        branch.id
    );
}
