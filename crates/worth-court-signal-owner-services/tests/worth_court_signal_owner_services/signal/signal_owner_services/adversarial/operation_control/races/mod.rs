#[allow(unused_imports)] use crate::signal_owner_services::ordinary_context::adversarial::operation_control::races::*;
pub(crate) use {crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld, crate::signal_owner_services::ordinary_context::adversarial::world::PROGRESS_BOUND};
use {crate::signal_owner_services::ordinary_context::execution_custody};
pub(crate) use {std::sync::mpsc, std::sync::Arc, std::sync::Barrier};
pub(crate) use {std::thread};
pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_signal::facade::branch::AdmittedSignalBranchBasis, worth_signal::facade::branch::AdmittedSignalBranchSnapshot, worth_signal::facade::branch::SignalBranchAdvanceDenial, worth_signal::facade::branch::SignalBranchRestoreDenial, worth_signal::facade::branch::SignalBranchRetirementDenial, worth_signal::facade::branch::SignalBranchRetirementReason, worth_signal::facade::branch::SignalBranchSnapshotCaptureDenial, worth_signal::facade::branch::SignalOwnerCancellationSource};

#[test]
fn same_branch_advance_advance_has_one_winner_and_both_orderings() {
    prove_advance_is_effectful();
    for pause_left in [true, false] {
        let world = crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld::new();
        let control = world
            .runtime
            .as_ref()
            .expect("the race retains its owner root")
            .owner_operation_control_for_test()
            .expect("operation control is issued after sealing");
        let pause = control.arm_pause_once_for_test(
            worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::BranchRegistryLookup,
        );
        let left_mutation = world.mutation.clone();
        let right_mutation = world.mutation.clone();
        let left_basis = world.child_basis.clone();
        let right_basis = world.child_basis.clone();
        let (left_tx, left_rx) = std::sync::mpsc::sync_channel(1);
        let (right_tx, right_rx) = std::sync::mpsc::sync_channel(1);

        std::thread::scope(|scope| {
            let left = move || advance(&left_mutation, &left_basis);
            let right = move || advance(&right_mutation, &right_basis);
            if pause_left {
                scope.spawn(move || {
                    let _ = left_tx.send(left());
                });
                assert!(pause.wait_until_reached(PROGRESS_BOUND));
                scope.spawn(move || {
                    let _ = right_tx.send(right());
                });
            } else {
                scope.spawn(move || {
                    let _ = right_tx.send(right());
                });
                assert!(pause.wait_until_reached(PROGRESS_BOUND));
                scope.spawn(move || {
                    let _ = left_tx.send(left());
                });
            }
            let first = if pause_left {
                right_rx
                    .recv_timeout(PROGRESS_BOUND)
                    .expect("the unparked right operation wins")
            } else {
                left_rx
                    .recv_timeout(PROGRESS_BOUND)
                    .expect("the unparked left operation wins")
            };
            pause.release();
            let second = if pause_left {
                left_rx
                    .recv_timeout(PROGRESS_BOUND)
                    .expect("the parked left operation resolves")
            } else {
                right_rx
                    .recv_timeout(PROGRESS_BOUND)
                    .expect("the parked right operation resolves")
            };
            assert_one_success(first, second, "advance/advance");
        });
    }
}

#[test]
fn same_branch_advance_restore_has_one_winner() {
    prove_advance_is_effectful();
    prove_capture_and_restore_are_effectful();
    let world = crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld::new();
    let (snapshot, current) = capture(&world.mutation, &world.child_basis)
        .expect("the race has a real admitted snapshot");
    let mutation_a = world.mutation.clone();
    let mutation_b = world.mutation.clone();
    let basis_a = current.clone();
    let basis_b = current.clone();
    let snapshot_b = snapshot.clone();
    run_pair(
        &world.runtime,
        move || advance(&mutation_a, &basis_a),
        move || restore(&mutation_b, &basis_b, &snapshot_b),
        "advance/restore",
    );
}

#[test]
fn same_branch_restore_restore_has_one_winner() {
    prove_capture_and_restore_are_effectful();
    let world = crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld::new();
    let (snapshot, current) = capture(&world.mutation, &world.child_basis)
        .expect("the race has a real admitted snapshot");
    let mutation_a = world.mutation.clone();
    let mutation_b = world.mutation.clone();
    let basis_a = current.clone();
    let basis_b = current.clone();
    let snapshot_a = snapshot.clone();
    let snapshot_b = snapshot.clone();
    run_pair(
        &world.runtime,
        move || restore(&mutation_a, &basis_a, &snapshot_a),
        move || restore(&mutation_b, &basis_b, &snapshot_b),
        "restore/restore",
    );
}

#[test]
fn same_branch_snapshot_advance_has_one_winner() {
    prove_capture_and_restore_are_effectful();
    prove_advance_is_effectful();
    let world = crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld::new();
    let mutation_a = world.mutation.clone();
    let mutation_b = world.mutation.clone();
    let basis_a = world.child_basis.clone();
    let basis_b = world.child_basis.clone();
    run_pair(
        &world.runtime,
        move || capture(&mutation_a, &basis_a).map(|_| ()),
        move || advance(&mutation_b, &basis_b),
        "snapshot/advance",
    );
}
#[path = "retirement_fences/mod.rs"]
pub(crate) mod retirement_fences;
