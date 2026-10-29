pub(crate) use {crate::signal_owner_services::ordinary_context::execution_custody};
pub(crate) use {std::sync::mpsc};
pub(crate) use {std::thread};

pub(crate) use {worth_signal::facade::branch::validate_signal_branch_name, worth_signal::facade::branch::SignalBranchRetirementReason, worth_signal::facade::branch::SignalOwnerCancellationSource, worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest};

pub(crate) use {crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld, crate::signal_owner_services::ordinary_context::adversarial::world::PROGRESS_BOUND};


pub(crate) fn control(
    world: &AdversarialWorld,
) -> worth_signal::facade::branch::SignalOwnerOperationControlForTest {
    world
        .runtime
        .as_ref()
        .expect("the owner root remains live")
        .owner_operation_control_for_test()
        .expect("the sealed owner issues operation control")
}

pub(crate) fn advance_result(
    mutation: &crate::signal_owner_services::ordinary_context::adversarial::world::MutationPort,
    basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
) -> Result<worth_signal::facade::branch::AdmittedSignalBranchBasis, String> {
    mutation
        .advance_exact(
            worth_execution::ExecutionRequest::serial(
                &crate::signal_owner_services::ordinary_context::execution_custody::operational_serial_request(),
            ),
            basis,
            &mut (),
            &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .map(|outcome| outcome.into_basis())
        .map_err(|denial| format!("{denial:?}"))
}

pub(crate) fn assert_advanced(
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

pub(crate) fn exercise_advance_boundary(boundary: SignalOwnerOperationBoundaryForTest) {
    let world = crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld::new();
    let control = control(&world);
    let pause = control.arm_pause_once_for_test(boundary);
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
        pause.release();
        assert_advanced(
            a_rx.recv_timeout(PROGRESS_BOUND)
                .expect("A reports after release"),
            &world.root_basis,
        );
    });
}

pub(crate) fn exercise_fork_boundary(boundary: SignalOwnerOperationBoundaryForTest) {
    let world = crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld::new();
    let control = control(&world);
    let pause = control.arm_pause_once_for_test(boundary);
    let (fork_tx, fork_rx) = std::sync::mpsc::sync_channel(1);
    let (advance_tx, advance_rx) = std::sync::mpsc::sync_channel(1);
    let fork_mutation = world.mutation.clone();
    let advance_mutation = world.mutation.clone();
    let root_basis = world.root_basis.clone();
    let child_basis = world.child_basis.clone();

    std::thread::scope(|scope| {
        scope.spawn(move || {
            let result = fork_mutation
                .fork_exact(
                    validate_signal_branch_name("parked-fork-child")
                        .expect("the fork name is valid"),
                    &root_basis,
                    &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
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

pub(crate) fn exercise_fork_post_capture_boundary(boundary: SignalOwnerOperationBoundaryForTest) {
    let world = crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld::new();
    let control = control(&world);
    let pause = control.arm_pause_once_for_test(boundary);
    let before = world
        .basis
        .owner_service_cost_snapshot()
        .expect("the owner is open");
    let (fork_tx, fork_rx) = std::sync::mpsc::sync_channel(1);
    let (advance_tx, advance_rx) = std::sync::mpsc::sync_channel(1);
    let fork_mutation = world.mutation.clone();
    let advance_mutation = world.mutation.clone();
    let fork_basis = world.root_basis.clone();
    let source_basis = world.root_basis.clone();

    std::thread::scope(|scope| {
        scope.spawn(move || {
            let result = fork_mutation
                .fork_exact(
                    validate_signal_branch_name("parked-post-capture")
                        .expect("the fork name is valid"),
                    &fork_basis,
                    &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
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

pub(crate) fn exercise_retirement_boundary(boundary: SignalOwnerOperationBoundaryForTest) {
    let world = crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld::new();
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
    if boundary == worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::ExactBasisPreflight {
        return crate::signal_owner_services::ordinary_context::adversarial::operation_control::independent_progress::retirement_planning::exercise(
            runtime,
            lifecycle,
            mutation,
            root_basis,
            retirement_basis,
        );
    }
    let plan = match lifecycle
        .plan_retirement_exact(retirement_basis, worth_signal::facade::branch::SignalBranchRetirementReason::Superseded)
    {
        worth_proof::TransitionOutcome::Success(plan) => plan,
        other => panic!("the child plan must be owner-issued: {other:?}"),
    };
    let control = runtime
        .as_ref()
        .expect("the strong root remains live")
        .owner_operation_control_for_test()
        .expect("the sealed owner issues operation control");
    let pause = control.arm_pause_once_for_test(boundary);
    let (retire_tx, retire_rx) = std::sync::mpsc::sync_channel(1);
    let (advance_tx, advance_rx) = std::sync::mpsc::sync_channel(1);
    let worker_lifecycle = lifecycle.clone();
    let worker_mutation = mutation.clone();
    let worker_root_basis = root_basis.clone();

    std::thread::scope(|scope| {
        scope.spawn(move || {
            let result = worker_lifecycle
                .retire_exact(plan, &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token())
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