use super::*;

#[test]
fn foreign_and_signal_only_sources_refuse_and_duplicate_attachment_preserves_the_first_source() {
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let court = CompositeSupplyChainCourt::compile();
    let foreign = CompositeSupplyChainCourt::compile();
    let current = court.bootstrap();
    let other = foreign.bootstrap();
    let target = fork(&court, &execution, &current);
    let token = RuntimeWorldCancellationSource::new().token();
    let candidate =
        court
            .records
            .candidate(&execution, target.basis().relational_basis(), "grain", "5");
    let denied = court
        .world
        .publication_port()
        .prepare_without_signal(
            target.clone(),
            intent(
                CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
                    .with_prepared_relational_candidate(candidate),
                &other,
            ),
            &token,
            None,
        )
        .unwrap_err();
    assert_eq!(
        denied.cause(),
        NoEffectCause::CurrentReadDependency(CurrentProductReadDependencyDenial::Branch(
            RuntimeWorldBranchAdmissionDenial::ForeignOwner
        ))
    );
    let denied = court
        .world
        .publication_port()
        .prepare_with_signal(
            target.clone(),
            intent(CompositePublicationIntent::with_signal(None), &current),
            &token,
            None,
        )
        .unwrap_err();
    assert_eq!(
        denied.cause(),
        NoEffectCause::CurrentReadDependency(
            CurrentProductReadDependencyDenial::UnsupportedComponentPlan
        )
    );
    let candidate =
        court
            .records
            .candidate(&execution, target.basis().relational_basis(), "grain", "5");
    let mut original = intent(
        CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
            .with_prepared_relational_candidate(candidate),
        &current,
    );
    assert_eq!(
        original.require_current_read(target.clone()),
        Err(CurrentProductReadDependencyDenial::AlreadyRequired)
    );
    let prepared = court
        .world
        .publication_port()
        .prepare_without_signal(target.clone(), original, &token, None)
        .unwrap();
    let moved = move_signal(&court, &execution, &current);
    let RuntimeWorldPublicationOutcome::NoEffect(denied) = court
        .world
        .publication_port()
        .execute_without_signal(execution, prepared, &token)
    else {
        panic!("duplicate cannot replace the original control source with the target");
    };
    assert_eq!(
        denied.cause(),
        NoEffectCause::CurrentReadDependency(
            CurrentProductReadDependencyDenial::WorldHeadUnavailable
        )
    );
    assert_eq!(court.observe(&target), target);
    assert_eq!(
        direct_basis(&court, &target),
        *target.basis().relational_basis()
    );
    let fresh = prepare(&court, &execution, &target, &moved);
    let RuntimeWorldPublicationOutcome::Performed(done) = court
        .world
        .publication_port()
        .execute_without_signal(execution, fresh, &token)
    else {
        panic!("lawful source after the refusal matrix must still perform");
    };
    drop(done.consume());
}
