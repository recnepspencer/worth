use std::sync::mpsc;
use std::thread;

use crate::branch::owner_services::operation_control::SignalOwnerOperationBoundary;

use super::super::progress_bound::PROGRESS_BOUND;
use super::super::runtime_root::runtime_with_two_branches;
use crate::branch::owner_services::SignalOwnerCancellationSource;

#[test]
fn operation_control_reaches_cell_basis_movement_and_outcome_boundaries() {
    for (boundary, expected_movements) in [
        (SignalOwnerOperationBoundary::TargetCellAdmission, 0),
        (SignalOwnerOperationBoundary::ExactBasisPreflight, 0),
        (SignalOwnerOperationBoundary::BeforeCanonicalMovement, 0),
        (SignalOwnerOperationBoundary::AfterCanonicalMovement, 1),
        (SignalOwnerOperationBoundary::OutcomeConstruction, 1),
    ] {
        exercise_advance_pause(boundary, expected_movements);
    }
}

fn exercise_advance_pause(boundary: SignalOwnerOperationBoundary, expected_movements: u64) {
    let (mut runtime, _, branch, basis) = runtime_with_two_branches();
    let (_, mutation, _) = runtime.owner_port_slots().expect("movement owner seals");
    let owner = mutation
        .upgrade_owner()
        .expect("movement owner remains live");
    let setup = owner.admit().expect("movement setup admits");
    let cell = owner
        .lookup_cell(&setup, branch.id)
        .expect("movement target is live");
    drop(setup);
    let pause = owner.operation_control().arm_pause_once(boundary);
    let (done_tx, done_rx) = mpsc::sync_channel(1);
    let worker_owner = owner.clone();
    let worker_cell = cell.clone();
    thread::spawn(move || {
        let admission = worker_owner.admit().expect("movement worker admits");
        let reservation = worker_owner
            .reserve_advance_output(&admission, &worker_cell)
            .expect("movement output reserves");
        let cancellation = SignalOwnerCancellationSource::new();
        let result = reservation
            .advance::<(), (), _>(&basis, &mut (), &cancellation.token(), |_| Ok(()))
            .into_result()
            .map(|ready| ready.into_parts().0.observation().generation().get());
        let _ = done_tx.send(result);
    });

    assert!(pause.wait_until_reached(PROGRESS_BOUND));
    assert_eq!(
        cell.cost_snapshot().movements(),
        expected_movements,
        "the pause exposes the exact movement side of {boundary:?}"
    );
    pause.release();
    let completed = done_rx.recv_timeout(PROGRESS_BOUND);
    assert!(matches!(completed, Ok(Ok(1))), "completion: {completed:?}");
}
