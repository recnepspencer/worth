use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::branch::admit_runtime_signal_branch_observation;
use crate::branch::owner_services::operation_control::SignalOwnerOperationBoundary;
use crate::branch::owner_services::SignalOwnerCancellationSource;
use crate::state::SignalSnapshotId;

use super::super::runtime_root::runtime_with_two_branches;

#[test]
fn snapshot_post_movement_faults_preserve_cell_metadata_and_release_pending_custody() {
    for boundary in [
        SignalOwnerOperationBoundary::AfterCanonicalMovement,
        SignalOwnerOperationBoundary::OutcomeConstruction,
    ] {
        exercise_snapshot_post_movement_fault(boundary);
    }
}

fn exercise_snapshot_post_movement_fault(boundary: SignalOwnerOperationBoundary) {
    let (mut runtime, _, branch, basis) = runtime_with_two_branches();
    let (_, mutation, _) = runtime.owner_port_slots().expect("snapshot owner seals");
    let owner = mutation
        .upgrade_owner()
        .expect("snapshot owner remains live");
    let admission = owner.admit().expect("snapshot fault admits");
    let cell = owner
        .lookup_cell(&admission, branch.id)
        .expect("snapshot target is live");
    let ledger_before = owner.retention_ledger_observation();
    let cell_before = cell.cost_snapshot();
    owner.operation_control().inject_panic_once(boundary);

    let fault = catch_unwind(AssertUnwindSafe(|| {
        let reservation = owner
            .reserve_snapshot_outputs(&admission, &cell)
            .expect("snapshot output custody reserves");
        let ready = reservation
            .capture(&basis, &SignalOwnerCancellationSource::new().token())
            .expect("snapshot movement reaches outcome construction");
        let _ = ready.into_outcome();
    }));
    assert!(
        fault.is_err(),
        "{boundary:?} must inject the controlled fault"
    );

    let observation = cell
        .observe_exact(&admission)
        .expect("post-publication snapshot cell remains observable");
    let snapshot_id = SignalSnapshotId(
        observation
            .target()
            .as_basis()
            .and_then(|target| target.snapshot_id())
            .expect("the committed cell publishes one snapshot identity"),
    );
    assert_eq!(snapshot_id, SignalSnapshotId(0));
    assert!(owner
        .metadata
        .has_snapshot_state(&admission, branch.id, snapshot_id)
        .expect("metadata observation is owner-admitted"));
    assert_eq!(owner.metadata.pending_snapshot_reservation_count(), 0);
    assert_eq!(cell.poison_recovery(), None);
    assert_eq!(
        cell.cost_snapshot().movements(),
        cell_before.movements() + 1
    );
    let mut released = ledger_before.clone();
    released.next_lease_id += 2;
    assert_eq!(
        owner.retention_ledger_observation(),
        released,
        "the lost caller receives no lease while every reserved identity returns"
    );

    let refreshed = admit_runtime_signal_branch_observation(
        observation,
        branch.id,
        owner
            .acquire_admitted_retention(&admission, branch.id)
            .expect("the complete published head can be readmitted"),
    );
    let retry = owner
        .reserve_snapshot_outputs(&admission, &cell)
        .expect("post-fault output capacity is reusable")
        .capture(&refreshed, &SignalOwnerCancellationSource::new().token())
        .expect("post-fault snapshot retry moves")
        .into_outcome();
    assert_eq!(retry.snapshot().meta.snapshot_id, SignalSnapshotId(1));
    assert_eq!(retry.captured_basis().owner_branch_id(), branch.id);
}
