use super::*;

#[test]
fn genuine_performed_native_custody_can_be_adopted_after_control_moves() {
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let court = CompositeSupplyChainCourt::compile();
    let current = court.bootstrap();
    let target = fork(&court, &execution, &current);
    let candidate =
        court
            .records
            .candidate(&execution, target.basis().relational_basis(), "grain", "5");
    let token = RuntimeWorldCancellationSource::new().token();
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
                &current,
            ),
            &token,
            None,
        )
        .unwrap();
    let RuntimeWorldPublicationOutcome::ProductUnpublished(effects) =
        court.world.publication_port().execute_with_signal(
            execution,
            prepared,
            &mut court.context(&execution, &target),
            &token,
            |_| {
                Err(worth_signal::facade::SignalError::InvalidInput {
                    message: "declared sibling refusal".into(),
                    context: None,
                })
            },
        )
    else {
        panic!("actual Native effect survives Signal refusal");
    };
    assert_eq!(effects.owner_effect_count(), 1);
    assert_eq!(court.observe(&target), target);
    let actual_native = direct_basis(&court, &target);
    assert_ne!(actual_native, *target.basis().relational_basis());
    let moved_control = court.publish_cargo(&current, "9");
    let current_now = court.observe(&current);
    assert_ne!(current_now, current);
    assert_ne!(
        current_now.basis().relational_basis(),
        current.basis().relational_basis()
    );
    let recovery = court.world.recovery_port();
    let adoption = recovery
        .prepare_settled_relational_adoption(&effects, &token, None)
        .unwrap();
    let RuntimeWorldPublicationOutcome::Performed(done) = court
        .world
        .publication_port()
        .execute_without_signal(execution, adoption, &token)
    else {
        panic!("original performed evidence does not require the old control head");
    };
    let selected = court.observe(&target);
    assert_eq!(selected.basis().relational_basis(), &actual_native);
    assert_eq!(grain(&court, &execution, &selected), "5");
    assert_eq!(grain(&court, &execution, &current_now), "9");
    let handle = effects.recovery_handle();
    drop(effects);
    let (_original, cleanup) = recovery
        .release_effects(
            &execution,
            &handle,
            0,
            court.observe_recovery_original(&execution, &handle),
        )
        .unwrap()
        .into_parts();
    assert!(cleanup.owner_retirement_work().is_empty());
    drop((done.consume(), moved_control));
}
