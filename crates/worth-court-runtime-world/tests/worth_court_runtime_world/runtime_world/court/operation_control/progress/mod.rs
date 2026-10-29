pub(crate) use worth_signal::facade::branch::validate_signal_branch_name;
pub(crate) use {
    crate::court::ordinary_context::operation_control::clean,
    crate::court::ordinary_context::operation_control::serial_request,
    crate::court::ordinary_context::operation_control::signal_prepared,
    crate::court::ordinary_context::operation_control::WAIT,
    crate::court::ordinary_context::CompositeSupplyChainCourt, std::sync::mpsc,
    worth_runtime_world::facade::*,
    worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest as Boundary,
};
#[test]
fn unrelated_signal_branch_progresses_at_each_owner_park() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for boundary in [
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::OwnerLifecycleAdmission,
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::BranchRegistryLookup,
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::ExactBasisPreflight,
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::TargetCellAdmission,
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::BeforeCanonicalMovement,
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::AfterCanonicalMovement,
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::OutcomeConstruction,
    ] {
        let court = crate::court::ordinary_context::CompositeSupplyChainCourt::compile();
        let policy = match court.world.execution_placement() {
            worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(policy)
            | worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Leased {
                policy, ..
            } => policy,
        };
        let root = court.bootstrap();
        let token = RuntimeWorldCancellationSource::new().token();
        let intent = ProductBranchCreationIntent::from_source(
            "independent",
            ProductBranchCreationPlans::new(
                RelationalBranchCreationPlan::ReuseExact,
                SignalBranchCreationPlan::ForkExact {
                    target: validate_signal_branch_name("independent").unwrap(),
                },
            ),
        )
        .unwrap();
        let RuntimeWorldBranchCreationOutcome::Performed(other) = court
            .world
            .branch_port()
            .create_product_branch(request_execution, root.clone(), intent, &token)
            .unwrap()
        else {
            panic!("real independent branch")
        };
        let first = signal_prepared(&court, &root, &token);
        let second = signal_prepared(&court, &other, &token);
        let mut first_context = court.context(request_execution, &root);
        let mut second_context = court.context(request_execution, &other);
        let control = court.signal.owner_operation_control_for_test().unwrap();
        let port = court.world.publication_port();
        let second_port = port.clone();
        let first_token = token.clone();
        let (send, receive) = std::sync::mpsc::sync_channel(1);
        std::thread::scope(|scope| {
            // This guard is inside the scope so assertion unwinding releases
            // the parked owner before scope's implicit worker joins.
            let pause = control.arm_pause_once_for_test(boundary);
            let first_worker = scope.spawn(move || {
                port.execute_with_signal(
                    worth_execution::ExecutionRequest::serial(
                        &crate::court::ordinary_context::operation_control::serial_request(policy),
                    ),
                    first,
                    &mut first_context,
                    &first_token,
                    |_| Ok(()),
                )
            });
            assert!(pause.wait_until_reached(WAIT), "owner park {boundary:?}");
            let second_worker = scope.spawn(move || {
                let result = second_port.execute_with_signal(
                    worth_execution::ExecutionRequest::serial(
                        &crate::court::ordinary_context::operation_control::serial_request(policy),
                    ),
                    second,
                    &mut second_context,
                    &token,
                    |_| Ok(()),
                );
                send.send(result).unwrap();
            });
            let progressed = receive
                .recv_timeout(WAIT)
                .expect("unrelated product branch must progress while owner is parked");
            assert!(
                matches!(progressed, RuntimeWorldPublicationOutcome::Performed(_)),
                "unrelated branch at {boundary:?}"
            );
            clean(&court, progressed);
            pause.release();
            let resumed = first_worker.join().unwrap();
            assert!(
                matches!(resumed, RuntimeWorldPublicationOutcome::Performed(_)),
                "healthy parked operation performs after release at {boundary:?}"
            );
            clean(&court, resumed);
            second_worker.join().unwrap();
        });
        let report = court
            .world
            .branch_port()
            .retire_product_branch(&other)
            .unwrap();
        assert_eq!(report.owner_retirement_work().len(), 1);
        drop((root, other, report));
        court.finish();
    }
}
