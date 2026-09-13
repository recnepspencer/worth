use std::sync::Arc;

use crate::branch::SignalBranchBasisObservationDenial;
use crate::data::graph::SignalGraph;
use crate::logic::transaction::{SignalRuntime, SnapshotBranchState};
use crate::state::SignalBranchHandle;

use super::super::{
    SignalBranchCellState, SignalBranchExecutionCell, SignalOwner, SignalOwnerCancellationSource,
    SignalOwnerOperationAdmission,
};

type TestRuntime = SignalRuntime<(), (), (), (), ()>;

fn runtime_with_one_snapshot_slot_and_sibling() -> (
    TestRuntime,
    SignalBranchHandle,
    crate::branch::AdmittedSignalBranchBasis,
    SignalBranchHandle,
    crate::branch::AdmittedSignalBranchBasis,
) {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .maximum_stored_branch_snapshots(1)
        .build();
    let source = runtime.current_branch();
    let source_basis = runtime
        .observe_signal_branch_basis(source.clone())
        .expect("the snapshot source basis observes");
    let (sibling, sibling_basis) = runtime
        .fork_signal_branch("snapshot-ordering-sibling", &source_basis)
        .expect("the independent healthy sibling forks")
        .into_parts();
    (runtime, source, source_basis, sibling, sibling_basis)
}

fn assert_faulted_cell_and_healthy_sibling_capture(
    owner: &Arc<SignalOwner<(), (), ()>>,
    admission: &SignalOwnerOperationAdmission<'_>,
    faulted: &Arc<SignalBranchExecutionCell<SignalBranchCellState<(), (), ()>>>,
    faulted_branch: &SignalBranchHandle,
    sibling: &Arc<SignalBranchExecutionCell<SignalBranchCellState<(), (), ()>>>,
    sibling_branch: &SignalBranchHandle,
    sibling_basis: &crate::branch::AdmittedSignalBranchBasis,
) {
    assert!(matches!(
        faulted.observe_exact(admission),
        Err(SignalBranchBasisObservationDenial::QuarantinedBranch { branch_id })
            if branch_id == faulted_branch.id
    ));
    let reservation = owner
        .metadata
        .reserve_snapshot(admission, sibling)
        .expect("the fault schedule returned the single snapshot slot exactly");
    let capture = sibling
        .capture_snapshot_exact(
            sibling_basis,
            reservation,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("an independent cell captures after the faulted reservation releases");
    assert_eq!(capture.snapshot().meta.branch_id, sibling_branch.id);
}

#[test]
fn direct_out_of_order_snapshot_reservation_drop_panics_once_and_returns_capacity() {
    let (mut runtime, source, _, sibling, sibling_basis) =
        runtime_with_one_snapshot_slot_and_sibling();
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
        .expect("snapshot capacity reserves before cell admission");

    let fault = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        source_cell
            .with_state(&admission, |_, _| drop(reservation))
            .expect("the source cell admits before the sensitivity fault");
    }));
    let fault = fault.expect_err("out-of-order drop must reject");
    assert_eq!(
        super::exact_cell_contracts::caught_panic_message(fault.as_ref()),
        Some("snapshot reservation cleanup must run after target-cell release")
    );
    assert_faulted_cell_and_healthy_sibling_capture(
        &owner,
        &admission,
        &source_cell,
        &source,
        &sibling_cell,
        &sibling,
        &sibling_basis,
    );
}

#[test]
fn out_of_order_snapshot_install_rejects_in_every_build_and_returns_capacity() {
    let (mut runtime, source, _, sibling, sibling_basis) =
        runtime_with_one_snapshot_slot_and_sibling();
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
        .expect("snapshot capacity reserves before install");
    let packet = source_cell
        .with_state(&admission, |state, _| {
            SnapshotBranchState::from_branch_state(state.state()).packet(reservation.snapshot_id())
        })
        .expect("the real packet is prepared before the disputed boundary");

    let fault = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        source_cell
            .with_state(&admission, move |_, _| reservation.install(packet))
            .expect("the source cell admits before install rejects ordering");
    }));
    let fault = fault.expect_err("out-of-order install must reject");
    assert_eq!(
        super::exact_cell_contracts::caught_panic_message(fault.as_ref()),
        Some("snapshot installation must run after target-cell release")
    );
    assert_faulted_cell_and_healthy_sibling_capture(
        &owner,
        &admission,
        &source_cell,
        &source,
        &sibling_cell,
        &sibling,
        &sibling_basis,
    );
}

#[test]
fn primary_cell_unwind_preserves_original_panic_and_returns_snapshot_capacity() {
    let (mut runtime, source, _, sibling, sibling_basis) =
        runtime_with_one_snapshot_slot_and_sibling();
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
        .expect("snapshot capacity reserves before the primary unwind");

    let fault = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        source_cell
            .with_state(&admission, move |_, _| {
                let _reservation = reservation;
                panic!("primary cell unwind remains the contained failure");
            })
            .expect("the primary callback exits through unwind");
    }));
    let fault = fault.expect_err("the primary callback panic propagates");
    assert_eq!(
        super::exact_cell_contracts::caught_panic_message(fault.as_ref()),
        Some("primary cell unwind remains the contained failure")
    );
    assert_faulted_cell_and_healthy_sibling_capture(
        &owner,
        &admission,
        &source_cell,
        &source,
        &sibling_cell,
        &sibling,
        &sibling_basis,
    );
}
