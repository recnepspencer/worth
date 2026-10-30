#[allow(unused_imports)]
pub(crate) use {
    crate::court::ordinary_context::operation_control::combined_cancellation::combined_candidate,
    crate::court::ordinary_context::operation_control::serial_request,
    crate::court::ordinary_context::operation_control::WAIT,
    crate::court::ordinary_context::CompositeSupplyChainCourt, worth_runtime_world::facade::*,
    worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest as Boundary,
};

#[test]
fn combined_cancellation_retains_actual_owners_without_a_mixed_world() {
    for boundary in [
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::BeforeCanonicalMovement,
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::AfterCanonicalMovement,
    ] {
        let host_request = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);
        let mut court = crate::court::ordinary_context::CompositeSupplyChainCourt::compile();
        let policy = match court.world.execution_placement() {
            worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(policy)
            | worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Leased {
                policy, ..
            } => policy,
        };
        let root = court.bootstrap();
        let cancellation = RuntimeWorldCancellationSource::new();
        let token = cancellation.token();
        let prepared = combined_candidate(&court, execution, &root, &token);
        let mut context = court.context(execution, &root);
        let port = court.world.publication_port();
        let signal = court
            .signal
            .owner_component_services()
            .unwrap()
            .basis_port();
        let signal_reference = signal
            .issue_managed_branch_reference(root.basis().signal_basis())
            .unwrap();
        let control = court.signal.owner_operation_control_for_test().unwrap();
        std::thread::scope(|scope| {
            let pause = control.arm_pause_once_for_test(boundary);
            let worker = scope.spawn(move || {
                port.execute_with_signal(
                    worth_execution::ExecutionRequest::serial(
                        &crate::court::ordinary_context::operation_control::serial_request(policy),
                    ),
                    prepared,
                    &mut context,
                    &token,
                    |_| Ok(()),
                )
            });
            assert!(
                pause.wait_until_reached(WAIT),
                "combined owner reaches {boundary:?}"
            );
            let direct = court
                .records
                .runtime
                .observe_branch(&court.records.runtime.main_branch_identity())
                .unwrap()
                .1;
            assert_eq!(
                court
                    .records
                    .read(execution, &direct)
                    .records
                    .get("grain")
                    .map(String::as_str),
                Some("5")
            );
            assert_eq!(court.observe(&root), root);
            cancellation.cancel();
            pause.release();
            let RuntimeWorldPublicationOutcome::ProductUnpublished(effects) =
                worker.join().unwrap()
            else {
                panic!("Relational already moved, so cancellation cannot report no effect");
            };
            let signal_moved = boundary == worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::AfterCanonicalMovement;
            assert_eq!(
                effects.owner_effect_count(),
                if signal_moved { 2 } else { 1 }
            );
            assert_eq!(
                effects.cause(),
                ProductUnpublishedCause::CancellationAfterEffect
            );
            assert!(effects
                .component_results()
                .relational_settlement()
                .is_some());
            assert_eq!(
                effects
                    .component_results()
                    .signal()
                    .advanced_outcome()
                    .is_some(),
                signal_moved
            );
            let current_signal = signal.observe_current(&signal_reference).unwrap();
            assert_eq!(
                current_signal.admission_identity()
                    == root.basis().signal_basis().admission_identity(),
                !signal_moved
            );
            let unchanged = court.observe(&root);
            assert_eq!(unchanged, root);
            assert_eq!(
                court
                    .records
                    .read(execution, unchanged.basis().relational_basis())
                    .records
                    .get("grain")
                    .map(String::as_str),
                Some("4")
            );
            let handle = effects.recovery_handle();
            drop(effects);
            let recovery = court.world.recovery_port();
            let retained = recovery.inspect_effects(&handle).unwrap();
            assert_eq!(
                retained.owner_effect_count(),
                if signal_moved { 2 } else { 1 }
            );
            assert!(retained
                .component_results()
                .relational_settlement()
                .is_some());
            assert_eq!(
                retained
                    .component_results()
                    .signal()
                    .advanced_outcome()
                    .is_some(),
                signal_moved
            );
            drop(retained);
            recovery.release_effects(&handle, 0).unwrap();
            assert_eq!(court.observe(&root), root);
        });
        drop((signal_reference, root));
        court.finish();
    }
}
