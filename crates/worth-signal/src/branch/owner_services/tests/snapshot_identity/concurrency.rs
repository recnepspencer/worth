use std::sync::Arc;

use crate::branch::admit_runtime_signal_branch_observation;
use crate::data::graph::SignalGraph;
use crate::logic::transaction::SignalRuntime;

use super::super::super::SignalOwnerCancellationSource;
use super::super::progress_bound::worker_park;

#[test]
fn concurrent_sibling_captures_keep_identity_and_target_lease_accounting_independent() {
    let mut runtime = SignalRuntime::<(), (), (), (), ()>::build_for::<()>(SignalGraph::new());
    let initial = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("the runtime admits its shared source");
    let (first_branch, first_basis) = runtime
        .fork_signal_branch("concurrent-first", &initial)
        .expect("the first sibling forks")
        .into_parts();
    let (second_branch, second_basis) = runtime
        .fork_signal_branch("concurrent-second", &initial)
        .expect("the second sibling forks")
        .into_parts();
    let (_, mutation, _) = runtime.owner_port_slots().expect("the runtime seals");
    let owner = mutation.upgrade_owner().expect("the owner remains live");
    let admission = owner.admit().expect("cell lookup admits");
    let first_cell = owner
        .lookup_cell(&admission, first_branch.id)
        .expect("the first cell is installed");
    let second_cell = owner
        .lookup_cell(&admission, second_branch.id)
        .expect("the second cell is installed");
    drop(admission);
    let (first_capture, second_capture) = std::thread::scope(|scope| {
        let (first_park, mut first_control) = worker_park();
        let (second_park, mut second_control) = worker_park();
        let first_owner = Arc::clone(&owner);
        let first = scope.spawn(move || {
            let admission = first_owner.admit().expect("first capture admits");
            let reservation = first_owner
                .metadata
                .reserve_snapshot(&admission, &first_cell)
                .expect("first identity reserves");
            first_park.park("first sibling snapshot reservation");
            first_cell
                .capture_snapshot_exact(
                    &first_basis,
                    reservation,
                    &SignalOwnerCancellationSource::new().token(),
                )
                .expect("first sibling captures")
        });
        let second_owner = Arc::clone(&owner);
        let second = scope.spawn(move || {
            let admission = second_owner.admit().expect("second capture admits");
            let reservation = second_owner
                .metadata
                .reserve_snapshot(&admission, &second_cell)
                .expect("second identity reserves");
            second_park.park("second sibling snapshot reservation");
            second_cell
                .capture_snapshot_exact(
                    &second_basis,
                    reservation,
                    &SignalOwnerCancellationSource::new().token(),
                )
                .expect("second sibling captures")
        });
        first_control.wait_until_parked("first sibling snapshot reservation");
        second_control.wait_until_parked("second sibling snapshot reservation");
        first_control.release();
        second_control.release();
        let first_capture = first.join().expect("first capture thread remains healthy");
        let second_capture = second
            .join()
            .expect("second capture thread remains healthy");
        (first_capture, second_capture)
    });

    let (first_snapshot, first_observation) = first_capture.into_parts();
    let (second_snapshot, second_observation) = second_capture.into_parts();
    assert_ne!(
        first_snapshot.meta.snapshot_id,
        second_snapshot.meta.snapshot_id
    );
    let retention_admission = owner.admit().expect("retention acquisition admits");
    let first_retained_basis = admit_runtime_signal_branch_observation(
        first_observation,
        first_branch.id,
        owner
            .acquire_admitted_retention(&retention_admission, first_branch.id)
            .expect("first target receives admitted custody"),
    );
    let second_retained_basis = admit_runtime_signal_branch_observation(
        second_observation,
        second_branch.id,
        owner
            .acquire_admitted_retention(&retention_admission, second_branch.id)
            .expect("second target receives admitted custody"),
    );
    let first_lease = owner
        .acquire_external_retention(&retention_admission, &first_retained_basis)
        .expect("the first captured target retains");
    let second_lease = owner
        .acquire_external_retention(&retention_admission, &second_retained_basis)
        .expect("the second captured target retains");
    assert_ne!(
        first_lease.retained_target(),
        second_lease.retained_target()
    );
    let first_receipt = first_lease.release();
    assert_eq!(first_receipt.remaining_target_leases(), 0);
    assert_eq!(first_receipt.remaining_branch_leases(), 0);
    let second_receipt = second_lease.release();
    assert_eq!(second_receipt.remaining_target_leases(), 0);
    assert_eq!(second_receipt.remaining_branch_leases(), 0);
}
