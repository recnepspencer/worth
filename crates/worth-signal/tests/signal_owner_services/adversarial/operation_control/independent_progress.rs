use std::sync::mpsc;
use std::thread;

use worth_signal::facade::branch::{
    validate_signal_branch_name, SignalBranchRetirementReason, SignalOwnerCancellationSource,
    SignalOwnerOperationBoundary,
};

use super::super::world::{AdversarialWorld, PROGRESS_BOUND};

#[path = "independent_progress/retirement_planning.rs"]
mod retirement_planning;

fn control(world: &AdversarialWorld) -> worth_signal::facade::branch::SignalOwnerOperationControl {
    world
        .runtime
        .as_ref()
        .expect("the owner root remains live")
        .owner_operation_control()
        .expect("the sealed owner issues operation control")
}

fn advance_result(
    mutation: &super::super::world::MutationPort,
    basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
) -> Result<worth_signal::facade::branch::AdmittedSignalBranchBasis, String> {
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
    result: Result<worth_signal::facade::branch::AdmittedSignalBranchBasis, String>,
    expected: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
) {
    let advanced = result.expect("the requested branch must complete while the peer is parked");
    assert_eq!(advanced.branch_id(), expected.branch_id());
    assert_eq!(
        advanced.observation().generation().get(),
        expected.observation().generation().get() + 1,
        "progress must include a real canonical movement"
    );
}

fn exercise_advance_boundary(boundary: SignalOwnerOperationBoundary) {
    let world = AdversarialWorld::new();
    let control = control(&world);
    let pause = control.arm_pause_once(boundary);
    let (a_tx, a_rx) = mpsc::sync_channel(1);
    let (b_tx, b_rx) = mpsc::sync_channel(1);
    let mutation_a = world.mutation.clone();
    let mutation_b = world.mutation.clone();
    let basis_a = world.root_basis.clone();
    let basis_b = world.child_basis.clone();
    thread::scope(|scope| {
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
        pause.release();
        assert_advanced(
            a_rx.recv_timeout(PROGRESS_BOUND)
                .expect("A reports after release"),
            &world.root_basis,
        );
    });
}

#[test]
fn unrelated_branch_advances_while_fork_registry_and_source_capture_are_parked() {
    for boundary in [
        SignalOwnerOperationBoundary::BranchRegistryReservation,
        SignalOwnerOperationBoundary::ForkSourceCapture,
    ] {
        exercise_fork_boundary(boundary);
    }
}

#[test]
fn every_reachable_advance_boundary_preserves_unrelated_progress() {
    for boundary in [
        SignalOwnerOperationBoundary::OwnerLifecycleAdmission,
        SignalOwnerOperationBoundary::BranchRegistryLookup,
        SignalOwnerOperationBoundary::ExactBasisPreflight,
        SignalOwnerOperationBoundary::TargetCellAdmission,
        SignalOwnerOperationBoundary::BeforeCanonicalMovement,
        SignalOwnerOperationBoundary::AfterCanonicalMovement,
    ] {
        exercise_advance_boundary(boundary);
    }
}

#[test]
fn unrelated_branch_advances_before_a_parked_branch_is_released() {
    let world = AdversarialWorld::new();
    let control = control(&world);
    let pause = control.arm_pause_once(SignalOwnerOperationBoundary::BeforeCanonicalMovement);
    let before = world
        .basis
        .owner_service_cost_snapshot()
        .expect("the owner is open");
    let (a_tx, a_rx) = mpsc::sync_channel(1);
    let (b_tx, b_rx) = mpsc::sync_channel(1);
    let mutation_a = world.mutation.clone();
    let mutation_b = world.mutation.clone();
    let basis_a = world.root_basis.clone();
    let basis_b = world.child_basis.clone();

    thread::scope(|scope| {
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

fn exercise_fork_boundary(boundary: SignalOwnerOperationBoundary) {
    let world = AdversarialWorld::new();
    let control = control(&world);
    let pause = control.arm_pause_once(boundary);
    let (fork_tx, fork_rx) = mpsc::sync_channel(1);
    let (advance_tx, advance_rx) = mpsc::sync_channel(1);
    let fork_mutation = world.mutation.clone();
    let advance_mutation = world.mutation.clone();
    let root_basis = world.root_basis.clone();
    let child_basis = world.child_basis.clone();

    thread::scope(|scope| {
        scope.spawn(move || {
            let result = fork_mutation
                .fork_exact(
                    validate_signal_branch_name("parked-fork-child")
                        .expect("the fork name is valid"),
                    &root_basis,
                    &SignalOwnerCancellationSource::new().token(),
                )
                .map(|_| ())
                .map_err(|denial| format!("{denial:?}"));
            let _ = fork_tx.send(result);
        });
        assert!(pause.wait_until_reached(PROGRESS_BOUND));

        scope.spawn(move || {
            let _ = advance_tx.send(advance_result(&advance_mutation, &child_basis));
        });
        assert_advanced(
            advance_rx
                .recv_timeout(PROGRESS_BOUND)
                .expect("the sibling reports while fork capture is parked"),
            &world.child_basis,
        );
        pause.release();
        assert!(fork_rx
            .recv_timeout(PROGRESS_BOUND)
            .expect("the fork reports after release")
            .is_ok());
    });
}

fn exercise_fork_post_capture_boundary(boundary: SignalOwnerOperationBoundary) {
    let world = AdversarialWorld::new();
    let control = control(&world);
    let pause = control.arm_pause_once(boundary);
    let before = world
        .basis
        .owner_service_cost_snapshot()
        .expect("the owner is open");
    let (fork_tx, fork_rx) = mpsc::sync_channel(1);
    let (advance_tx, advance_rx) = mpsc::sync_channel(1);
    let fork_mutation = world.mutation.clone();
    let advance_mutation = world.mutation.clone();
    let fork_basis = world.root_basis.clone();
    let source_basis = world.root_basis.clone();

    thread::scope(|scope| {
        scope.spawn(move || {
            let result = fork_mutation
                .fork_exact(
                    validate_signal_branch_name("parked-post-capture")
                        .expect("the fork name is valid"),
                    &fork_basis,
                    &SignalOwnerCancellationSource::new().token(),
                )
                .map(|_| ())
                .map_err(|denial| format!("{denial:?}"));
            let _ = fork_tx.send(result);
        });
        assert!(pause.wait_until_reached(PROGRESS_BOUND));
        scope.spawn(move || {
            let _ = advance_tx.send(advance_result(&advance_mutation, &source_basis));
        });
        assert_advanced(
            advance_rx
                .recv_timeout(PROGRESS_BOUND)
                .expect("the source reports before destination work is released"),
            &world.root_basis,
        );
        let after_source = world
            .basis
            .owner_service_cost_snapshot()
            .expect("the parked fork leaves the owner inspectable");
        pause.release();
        assert!(fork_rx
            .recv_timeout(PROGRESS_BOUND)
            .expect("the fork reports after release")
            .is_ok());
        assert_eq!(
            after_source.target_cell_waits(),
            before.target_cell_waits(),
            "destination installation and outcome construction hold no source-cell custody"
        );
        assert_eq!(
            after_source.canonical_movements(),
            before.canonical_movements() + 1,
            "the source advance linearizes before destination work is released"
        );
        assert_eq!(
            after_source.fork_source_captures(),
            before.fork_source_captures() + 1,
            "the fork capture boundary completed before source custody was released"
        );
    });
}

#[test]
fn source_advances_while_post_capture_fork_work_is_parked() {
    for boundary in [
        SignalOwnerOperationBoundary::ForkDestinationInstallation,
        SignalOwnerOperationBoundary::OutcomeConstruction,
    ] {
        exercise_fork_post_capture_boundary(boundary);
    }
}

#[test]
fn outcome_construction_pause_holds_only_the_completed_branch() {
    exercise_advance_boundary(SignalOwnerOperationBoundary::OutcomeConstruction);
}

#[test]
fn unrelated_branch_advances_while_retirement_cell_work_is_parked() {
    for boundary in [
        SignalOwnerOperationBoundary::ExactBasisPreflight,
        SignalOwnerOperationBoundary::BeforeCanonicalMovement,
        SignalOwnerOperationBoundary::AfterCanonicalMovement,
    ] {
        exercise_retirement_boundary(boundary);
    }
}

fn exercise_retirement_boundary(boundary: SignalOwnerOperationBoundary) {
    let world = AdversarialWorld::new();
    let reference = world
        .basis
        .issue_managed_branch_reference(&world.child_basis)
        .expect("the retirement target has an owner-issued reference");
    let retirement_basis = world
        .basis
        .observe_current(&reference)
        .expect("retirement gets independent exact custody");
    let AdversarialWorld {
        runtime,
        mutation,
        lifecycle,
        root_basis,
        child_basis,
        ..
    } = world;
    drop(child_basis);
    if boundary == SignalOwnerOperationBoundary::ExactBasisPreflight {
        return retirement_planning::exercise(
            runtime,
            lifecycle,
            mutation,
            root_basis,
            retirement_basis,
        );
    }
    let plan = match lifecycle
        .plan_retirement_exact(retirement_basis, SignalBranchRetirementReason::Superseded)
    {
        worth_proof::TransitionOutcome::Success(plan) => plan,
        other => panic!("the child plan must be owner-issued: {other:?}"),
    };
    let control = runtime
        .as_ref()
        .expect("the strong root remains live")
        .owner_operation_control()
        .expect("the sealed owner issues operation control");
    let pause = control.arm_pause_once(boundary);
    let (retire_tx, retire_rx) = mpsc::sync_channel(1);
    let (advance_tx, advance_rx) = mpsc::sync_channel(1);
    let worker_lifecycle = lifecycle.clone();
    let worker_mutation = mutation.clone();
    let worker_root_basis = root_basis.clone();

    thread::scope(|scope| {
        scope.spawn(move || {
            let result = worker_lifecycle
                .retire_exact(plan, &SignalOwnerCancellationSource::new().token())
                .into_result()
                .map(|_| ())
                .map_err(|denial| format!("{denial:?}"));
            let _ = retire_tx.send(result);
        });
        assert!(pause.wait_until_reached(PROGRESS_BOUND));
        scope.spawn(move || {
            let _ = advance_tx.send(advance_result(&worker_mutation, &worker_root_basis));
        });
        assert_advanced(
            advance_rx
                .recv_timeout(PROGRESS_BOUND)
                .expect("the unrelated branch reports while retirement is parked"),
            &root_basis,
        );
        pause.release();
        assert!(retire_rx
            .recv_timeout(PROGRESS_BOUND)
            .expect("retirement reports after release")
            .is_ok());
    });
    assert!(
        runtime.is_some(),
        "the owner root remains live through retirement"
    );
}
