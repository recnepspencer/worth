use super::*;

#[test]
fn combined_cancellation_retains_actual_owners_without_a_mixed_world() {
    for boundary in [
        Boundary::BeforeCanonicalMovement,
        Boundary::AfterCanonicalMovement,
    ] {
        let host_request = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);
        let mut court = CompositeSupplyChainCourt::compile();
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
                    worth_execution::ExecutionRequest::serial(&super::serial_request(policy)),
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
            let signal_moved = boundary == Boundary::AfterCanonicalMovement;
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

#[test]
fn uninterrupted_combined_candidate_publishes_both_owner_successors() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let court = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let token = RuntimeWorldCancellationSource::new().token();
    let prepared = combined_candidate(&court, execution, &root, &token);
    let RuntimeWorldPublicationOutcome::Performed(done) =
        court.world.publication_port().execute_with_signal(
            *execution,
            prepared,
            &mut court.context(execution, &root),
            &token,
            |_| Ok(()),
        )
    else {
        panic!("the same combined candidate succeeds without cancellation");
    };
    let head = court.observe(&root);
    assert_eq!(head.selected_commit(), done.commit().identity());
    assert_ne!(
        head.basis().relational_basis().admission_identity(),
        root.basis().relational_basis().admission_identity()
    );
    assert_ne!(
        head.basis().signal_basis().admission_identity(),
        root.basis().signal_basis().admission_identity()
    );
    assert_eq!(
        court
            .records
            .read(execution, head.basis().relational_basis())
            .records
            .get("grain")
            .map(String::as_str),
        Some("5")
    );
    drop(done.consume());
    drop((head, root));
    court.finish();
}

fn combined_candidate(
    court: &CompositeSupplyChainCourt,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    root: &ProductBranchObservation,
    token: &RuntimeWorldCancellationToken,
) -> PreparedCompositePublicationWithSignal {
    let candidate =
        court
            .records
            .candidate(execution, root.basis().relational_basis(), "grain", "5");
    court
        .world
        .publication_port()
        .prepare_with_signal(
            root.clone(),
            CompositePublicationIntent::with_signal(Some(RelationalTransactionIntent::ordinary()))
                .with_prepared_relational_candidate(candidate),
            token,
            None,
        )
        .unwrap()
}
