use std::sync::mpsc;
use std::thread;

use worth_signal::facade::branch::{
    AdmittedSignalBranchBasis, AdmittedSignalBranchSnapshot, SignalOwnerCancellationSource,
    SignalOwnerOperationBoundary,
};

use super::super::world::{AdversarialWorld, PROGRESS_BOUND};

fn advance_result(
    mutation: &super::super::world::MutationPort,
    basis: &AdmittedSignalBranchBasis,
) -> Result<AdmittedSignalBranchBasis, String> {
    mutation
        .advance_exact(
            basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .map(|outcome| outcome.into_basis())
        .map_err(|denial| format!("{denial:?}"))
}

fn assert_advanced(
    result: Result<AdmittedSignalBranchBasis, String>,
    expected: &AdmittedSignalBranchBasis,
) {
    let advanced = result.expect("the unrelated branch must make real progress");
    assert_eq!(advanced.branch_id(), expected.branch_id());
    assert_eq!(
        advanced.observation().generation().get(),
        expected.observation().generation().get() + 1,
        "the sibling result must prove a canonical movement"
    );
}

fn capture_result(
    mutation: &super::super::world::MutationPort,
    basis: &AdmittedSignalBranchBasis,
) -> Result<(AdmittedSignalBranchSnapshot, AdmittedSignalBranchBasis), String> {
    mutation
        .capture_exact(basis, &SignalOwnerCancellationSource::new().token())
        .map(|outcome| outcome.into_parts())
        .map_err(|denial| format!("{denial:?}"))
}

fn restore_result(
    mutation: &super::super::world::MutationPort,
    basis: &AdmittedSignalBranchBasis,
    snapshot: &AdmittedSignalBranchSnapshot,
) -> Result<AdmittedSignalBranchBasis, String> {
    mutation
        .restore_exact(
            basis,
            snapshot,
            &SignalOwnerCancellationSource::new().token(),
        )
        .map_err(|denial| format!("{denial:?}"))
}

fn exercise_capture_boundary(boundary: SignalOwnerOperationBoundary) {
    let world = AdversarialWorld::new();
    let control = world
        .runtime
        .as_ref()
        .expect("the owner root remains live")
        .owner_operation_control()
        .expect("the sealed owner issues operation control");
    let pause = control.arm_pause_once(boundary);
    let (capture_tx, capture_rx) = mpsc::sync_channel(1);
    let (advance_tx, advance_rx) = mpsc::sync_channel(1);
    let capture_mutation = world.mutation.clone();
    let advance_mutation = world.mutation.clone();
    let capture_basis = world.child_basis.clone();
    let advance_basis = world.root_basis.clone();

    thread::scope(|scope| {
        scope.spawn(move || {
            let _ = capture_tx.send(capture_result(&capture_mutation, &capture_basis));
        });
        assert!(pause.wait_until_reached(PROGRESS_BOUND));
        scope.spawn(move || {
            let _ = advance_tx.send(advance_result(&advance_mutation, &advance_basis));
        });
        assert_advanced(
            advance_rx
                .recv_timeout(PROGRESS_BOUND)
                .expect("the sibling reports while capture is parked"),
            &world.root_basis,
        );
        pause.release();
        let (snapshot, captured) = capture_rx
            .recv_timeout(PROGRESS_BOUND)
            .expect("capture reports after release")
            .expect("the parked capture remains lawful");
        assert_eq!(captured.branch_id(), world.child_basis.branch_id());
        assert_eq!(
            captured.observation().generation().get(),
            world.child_basis.observation().generation().get() + 1
        );
        drop(snapshot);
    });
}

fn exercise_restore_boundary(boundary: SignalOwnerOperationBoundary) {
    let world = AdversarialWorld::new();
    let (snapshot, captured_basis) = capture_result(&world.mutation, &world.child_basis)
        .expect("restore starts from an owner-issued snapshot");
    let expected_restored = captured_basis.clone();
    let control = world
        .runtime
        .as_ref()
        .expect("the owner root remains live")
        .owner_operation_control()
        .expect("the sealed owner issues operation control");
    let pause = control.arm_pause_once(boundary);
    let (restore_tx, restore_rx) = mpsc::sync_channel(1);
    let (advance_tx, advance_rx) = mpsc::sync_channel(1);
    let restore_mutation = world.mutation.clone();
    let advance_mutation = world.mutation.clone();
    let restore_basis = captured_basis;
    let advance_basis = world.root_basis.clone();

    thread::scope(|scope| {
        scope.spawn(move || {
            let _ = restore_tx.send(restore_result(&restore_mutation, &restore_basis, &snapshot));
        });
        assert!(pause.wait_until_reached(PROGRESS_BOUND));
        scope.spawn(move || {
            let _ = advance_tx.send(advance_result(&advance_mutation, &advance_basis));
        });
        assert_advanced(
            advance_rx
                .recv_timeout(PROGRESS_BOUND)
                .expect("the sibling reports while restoration is parked"),
            &world.root_basis,
        );
        pause.release();
        assert_advanced(
            restore_rx
                .recv_timeout(PROGRESS_BOUND)
                .expect("restoration reports after release"),
            &expected_restored,
        );
    });
}

#[test]
fn unrelated_branch_advances_while_snapshot_capture_crosses_every_work_boundary() {
    for boundary in [
        SignalOwnerOperationBoundary::ExactBasisPreflight,
        SignalOwnerOperationBoundary::BeforeCanonicalMovement,
        SignalOwnerOperationBoundary::AfterCanonicalMovement,
        SignalOwnerOperationBoundary::OutcomeConstruction,
    ] {
        exercise_capture_boundary(boundary);
    }
}

#[test]
fn unrelated_branch_advances_while_restoration_crosses_every_work_boundary() {
    for boundary in [
        SignalOwnerOperationBoundary::ExactBasisPreflight,
        SignalOwnerOperationBoundary::BeforeCanonicalMovement,
        SignalOwnerOperationBoundary::AfterCanonicalMovement,
        SignalOwnerOperationBoundary::OutcomeConstruction,
    ] {
        exercise_restore_boundary(boundary);
    }
}
