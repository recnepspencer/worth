use super::*;

mod conditional_definition;
mod deadline;
mod panic_recovery;
mod refusals;

#[test]
fn unused_cutover_preserves_both_heads_and_the_same_write_can_then_perform() {
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let court = CompositeSupplyChainCourt::compile();
    let current = court.bootstrap();
    let target = fork(&court, &execution, &current);
    let token = RuntimeWorldCancellationSource::new().token();
    let (caller, outcome) = court
        .world
        .publication_port()
        .execute_without_signal_with_memory_cutover(
            execution,
            prepare(&court, &execution, &target, &current),
            &token,
            |cutover| {
                assert_eq!(cutover.current_read(), &current);
                assert_eq!(cutover.expected_target(), &target);
                "declined"
            },
        );
    assert_eq!(caller, Some("declined"));
    let RuntimeWorldPublicationOutcome::NoEffect(no) = outcome else {
        panic!("unused token must refuse")
    };
    assert_eq!(
        no.cause(),
        NoEffectCause::CurrentReadDependency(
            CurrentProductReadDependencyDenial::CutoverNotPerformed
        )
    );
    assert_eq!(court.observe(&current), current);
    assert_eq!(court.observe(&target), target);
    assert_eq!(
        direct_basis(&court, &target),
        *target.basis().relational_basis()
    );
    let (caller, outcome) = court
        .world
        .publication_port()
        .execute_without_signal_with_memory_cutover(
            execution,
            prepare(&court, &execution, &target, &current),
            &token,
            |cutover| {
                cutover.perform();
                Err::<(), _>("caller status")
            },
        );
    assert_eq!(caller, Some(Err("caller status")));
    let RuntimeWorldPublicationOutcome::Performed(done) = outcome else {
        panic!("caller error cannot erase actual publication")
    };
    assert_eq!(grain(&court, &execution, &court.observe(&target)), "5");
    assert_eq!(grain(&court, &execution, &court.observe(&current)), "4");
    drop(done.consume());
}

#[test]
fn callback_can_move_control_before_the_guard_and_signal_runs_after_the_callback() {
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let court = CompositeSupplyChainCourt::compile();
    let current = court.bootstrap();
    let target = fork(&court, &execution, &current);
    let token = RuntimeWorldCancellationSource::new().token();
    let prepared = prepare(&court, &execution, &target, &current);
    let (_, outcome) = court
        .world
        .publication_port()
        .execute_without_signal_with_memory_cutover(execution, prepared, &token, |cutover| {
            assert!(!court
                .world
                .observation_port()
                .product_branch_writers_are_locked_out_for_test(&current)
                .unwrap());
            let moved = move_signal(&court, &execution, &current);
            assert_ne!(moved, current);
            cutover.perform();
        });
    let RuntimeWorldPublicationOutcome::NoEffect(no) = outcome else {
        panic!("changed control refuses at the effect")
    };
    assert_eq!(
        no.cause(),
        NoEffectCause::CurrentReadDependency(
            CurrentProductReadDependencyDenial::WorldHeadUnavailable
        )
    );
    assert_eq!(court.observe(&target), target);
    assert_eq!(
        direct_basis(&court, &target),
        *target.basis().relational_basis()
    );
    let fresh = court.observe(&current);
    let candidate =
        court
            .records
            .candidate(&execution, target.basis().relational_basis(), "grain", "5");
    let prepared = court
        .world
        .publication_port()
        .prepare_with_signal(
            target.clone(),
            intent(
                CompositePublicationIntent::with_signal(Some(
                    RelationalTransactionIntent::ordinary(),
                ))
                .with_prepared_relational_candidate(candidate),
                &fresh,
            ),
            &token,
            None,
        )
        .unwrap();
    let callback_finished = std::cell::Cell::new(false);
    let (caller, outcome) = court
        .world
        .publication_port()
        .execute_with_signal_with_memory_cutover(
            execution,
            prepared,
            &mut court.context(&execution, &target),
            &token,
            |_| {
                assert!(callback_finished.get());
                assert!(!court
                    .world
                    .observation_port()
                    .product_branch_writers_are_locked_out_for_test(&fresh)
                    .unwrap());
                Ok(())
            },
            |cutover| {
                cutover
                    .perform_before(std::time::Instant::now() + std::time::Duration::from_secs(30));
                callback_finished.set(true);
                Err::<(), _>("caller status")
            },
        );
    assert_eq!(caller, Some(Err("caller status")));
    let RuntimeWorldPublicationOutcome::Performed(done) = outcome else {
        panic!("both actual owner effects must publish")
    };
    let selected = court.observe(&target);
    assert_ne!(
        selected.basis().signal_basis().admission_identity(),
        target.basis().signal_basis().admission_identity()
    );
    assert_eq!(grain(&court, &execution, &selected), "5");
    assert_eq!(court.observe(&current), fresh);
    drop(done.consume());
}
