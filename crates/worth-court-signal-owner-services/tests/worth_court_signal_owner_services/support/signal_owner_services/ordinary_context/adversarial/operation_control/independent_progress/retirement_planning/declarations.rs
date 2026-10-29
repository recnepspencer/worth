pub(crate) use {std::sync::mpsc};
pub(crate) use {std::thread};

pub(crate) use {worth_signal::facade::branch::AdmittedSignalBranchBasis, worth_signal::facade::branch::SignalBranchRetirementReason, worth_signal::facade::branch::SignalOwnerCancellationSource, worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest};

pub(crate) use {crate::signal_owner_services::ordinary_context::adversarial::world::PROGRESS_BOUND};

pub(crate) fn exercise(
    runtime: Option<crate::signal_owner_services::ordinary_context::adversarial::world::Runtime>,
    lifecycle: crate::signal_owner_services::ordinary_context::adversarial::world::LifecyclePort,
    mutation: crate::signal_owner_services::ordinary_context::adversarial::world::MutationPort,
    root_basis: AdmittedSignalBranchBasis,
    retirement_basis: AdmittedSignalBranchBasis,
) {
    let control = runtime
        .as_ref()
        .expect("the strong root remains live")
        .owner_operation_control_for_test()
        .expect("the sealed owner issues operation control");
    let pause =
        control.arm_pause_once_for_test(worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::ExactBasisPreflight);
    let worker_lifecycle = lifecycle.clone();
    let worker_mutation = mutation.clone();
    let worker_root_basis = root_basis.clone();
    let (plan_tx, plan_rx) = std::sync::mpsc::sync_channel(1);
    let (advance_tx, advance_rx) = std::sync::mpsc::sync_channel(1);

    std::thread::scope(|scope| {
        scope.spawn(move || {
            let result = worker_lifecycle
                .plan_retirement_exact(retirement_basis, worth_signal::facade::branch::SignalBranchRetirementReason::Superseded);
            let _ = plan_tx.send(result);
        });
        assert!(pause.wait_until_reached(PROGRESS_BOUND));
        scope.spawn(move || {
            let _ = advance_tx.send(crate::signal_owner_services::ordinary_context::adversarial::operation_control::independent_progress::advance_result(&worker_mutation, &worker_root_basis));
        });
        crate::signal_owner_services::ordinary_context::adversarial::operation_control::independent_progress::assert_advanced(
            advance_rx
                .recv_timeout(PROGRESS_BOUND)
                .expect("the unrelated branch advances while planning is parked"),
            &root_basis,
        );
        pause.release();
        let plan = match plan_rx
            .recv_timeout(PROGRESS_BOUND)
            .expect("retirement planning reports after release")
        {
            worth_proof::TransitionOutcome::Success(plan) => plan,
            other => panic!("the parked retirement plan remains lawful: {other:?}"),
        };
        assert!(matches!(
            lifecycle.retire_exact(plan, &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token()),
            worth_proof::TransitionOutcome::Success(_)
        ));
    });
    runtime
        .as_ref()
        .expect("the owner root remains live through retirement planning")
        .owner_operation_control_for_test()
        .expect("the owner control remains available after contenders finish");
}