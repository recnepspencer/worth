use super::*;

#[test]
fn advanced_cutover_requires_current_read_and_fresh_native_work_before_calling_the_caller() {
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let court = CompositeSupplyChainCourt::compile();
    let head = court.bootstrap();
    let token = RuntimeWorldCancellationSource::new().token();
    let candidate =
        court
            .records
            .candidate(&execution, head.basis().relational_basis(), "grain", "5");
    let prepared = court
        .world
        .publication_port()
        .prepare_without_signal(
            head.clone(),
            CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
                .with_prepared_relational_candidate(candidate),
            &token,
            None,
        )
        .unwrap();
    let (caller, outcome) = court
        .world
        .publication_port()
        .execute_without_signal_with_memory_cutover(execution, prepared, &token, |_| {
            panic!("missing current-read must refuse before callback")
        });
    assert!(caller.is_none());
    let RuntimeWorldPublicationOutcome::NoEffect(no) = outcome else {
        panic!("missing current-read refuses")
    };
    assert_eq!(
        no.cause(),
        NoEffectCause::CurrentReadDependency(
            CurrentProductReadDependencyDenial::CurrentReadRequired
        )
    );
    let prepared = court
        .world
        .publication_port()
        .prepare_with_signal(
            head.clone(),
            CompositePublicationIntent::with_signal(None),
            &token,
            None,
        )
        .unwrap();
    let (caller, outcome) = court
        .world
        .publication_port()
        .execute_with_signal_with_memory_cutover(
            execution,
            prepared,
            &mut court.context(&execution, &head),
            &token,
            |_| panic!("unsupported Native posture refuses before Signal"),
            |_| panic!("unsupported Native posture refuses before callback"),
        );
    assert!(caller.is_none());
    let RuntimeWorldPublicationOutcome::NoEffect(no) = outcome else {
        panic!("unsupported posture refuses")
    };
    assert_eq!(
        no.cause(),
        NoEffectCause::CurrentReadDependency(
            CurrentProductReadDependencyDenial::UnsupportedComponentPlan
        )
    );
    assert_eq!(court.observe(&head), head);
    assert_eq!(
        direct_basis(&court, &head),
        *head.basis().relational_basis()
    );
}
