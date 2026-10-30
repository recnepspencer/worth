#[allow(unused_imports)] use crate::signal_owner_services::ordinary_context::adversarial::operation_control::independent_progress::*;
use {crate::signal_owner_services::ordinary_context::execution_custody};
pub(crate) use {std::sync::mpsc};
pub(crate) use {std::thread};

pub(crate) use {
    worth_signal::facade::branch::validate_signal_branch_name,
    worth_signal::facade::branch::SignalBranchRetirementReason,
    worth_signal::facade::branch::SignalOwnerCancellationSource,
    worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest,
};

pub(crate) use {
    crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld,
    crate::signal_owner_services::ordinary_context::adversarial::world::PROGRESS_BOUND,
};

#[test]
fn unrelated_branch_advances_before_a_parked_branch_is_released() {
    let world =
        crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld::new();
    let control = control(&world);
    let pause = control.arm_pause_once_for_test(
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::BeforeCanonicalMovement,
    );
    let before = world
        .basis
        .owner_service_cost_snapshot()
        .expect("the owner is open");
    let (a_tx, a_rx) = std::sync::mpsc::sync_channel(1);
    let (b_tx, b_rx) = std::sync::mpsc::sync_channel(1);
    let mutation_a = world.mutation.clone();
    let mutation_b = world.mutation.clone();
    let basis_a = world.root_basis.clone();
    let basis_b = world.child_basis.clone();

    std::thread::scope(|scope| {
        scope.spawn(move || {
            let _ = a_tx.send(advance_result(&mutation_a, &basis_a));
        });
        assert!(pause.wait_until_reached(PROGRESS_BOUND));

        scope.spawn(move || {
            let _ = b_tx.send(advance_result(&mutation_b, &basis_b));
        });
        assert_advanced(
            b_rx.recv_timeout(PROGRESS_BOUND)
                .expect("B reports before A is released"),
            &world.child_basis,
        );
        let after_b = world
            .basis
            .owner_service_cost_snapshot()
            .expect("the parked operation leaves the owner inspectable");
        assert_eq!(
            after_b.target_cell_contacts() - before.target_cell_contacts(),
            2,
            "A has contacted its cell before the park and B contacts only its own cell"
        );
        assert_eq!(
            after_b.target_cell_waits(),
            before.target_cell_waits(),
            "unrelated branch progress does not wait on A's cell"
        );
        assert_eq!(
            after_b.canonical_movements(),
            before.canonical_movements() + 1,
            "B moved while A was parked and no A movement was reported"
        );

        pause.release();
        assert_advanced(
            a_rx.recv_timeout(PROGRESS_BOUND)
                .expect("A reports after release"),
            &world.root_basis,
        );
    });

    let after = world
        .basis
        .owner_service_cost_snapshot()
        .expect("both workers leave the owner healthy");
    assert_eq!(
        after.target_cell_contacts() - before.target_cell_contacts(),
        2,
        "each branch operation contacts exactly its own cell"
    );
    assert_eq!(after.target_cell_waits(), before.target_cell_waits());
    assert_eq!(
        after.canonical_movements(),
        before.canonical_movements() + 2
    );
}

#[test]
fn outcome_construction_pause_holds_only_the_completed_branch() {
    exercise_advance_boundary(
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::OutcomeConstruction,
    );
}
