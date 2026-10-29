pub(crate) use worth_query_bank_worlds::facade::server::estate_operations::recognize_executor::*;

pub(crate) mod adoption;

#[test]
fn public_query_observes_the_exact_recognized_executor() {
    let fixture = exact_recognition_world("recognize-executor-commit");
    let specialist = fixture.authenticate_specialist();
    let key = BankIdempotencyKey::new("recognize-estate-executor-program-entry").unwrap();
    let outcome = {
        let source_actor = &specialist;
        let source_scope = &request_scope();
        let source_key = &key;
        let source_intent = bank_domain::schema::RecognizeEstateExecutor::try_from(
            fixture.action(fixture.executor),
        )
        .expect("typed estate command preserves the exact variant");
        let source_request = fixture.world.runtime.request(source_actor, source_scope);
        let source_operation = fixture
            .world
            .runtime
            .mutate(bank_server::mutations::recognize_estate_executor(
                source_intent,
            ))
            .as_principal(source_actor)
            .controls(bank_server::BankMutationControls::new(
                (*source_scope).clone(),
                (*source_key).clone(),
            ))
            .retain_operation()
            .unwrap_or_else(|cause| panic!("exact declared estate construction: {cause:?}"));
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
            source_completed,
        ) = source_operation.advance(&source_request)
        else {
            panic!("native source completion required")
        };
        assert!(matches!(
            source_completed.evidence(),
            worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh { .. }
        ));
        let closed_source = source_completed
            .close_with_evidence(&source_request)
            .unwrap_or_else(|(_, cause)| panic!("lawful native source close: {cause:?}"));
        bank_server::BankCommitReceipt::from_closed_source(closed_source)
    };

    let receipt = outcome;
    assert_eq!(receipt.emitted_effect_count(), 0);
    assert_eq!(receipt.expected_fact_count(), 0);
    assert_eq!(receipt.decision_fact_count(), Some(8));
    assert_admission_derives_the_request_identities_once(receipt.canonical_work());
    assert_eq!(estate_executors(&fixture), [fixture.executor]);
    assert!(estate_authority_is_still_recognized(&fixture));

    let retry = {
        let source_actor = &specialist;
        let source_scope = &request_scope();
        let source_key = &key;
        let source_intent = bank_domain::schema::RecognizeEstateExecutor::try_from(
            fixture.action(fixture.executor),
        )
        .expect("typed estate command preserves the exact variant");
        let source_request = fixture.world.runtime.request(source_actor, source_scope);
        let source_operation = fixture
            .world
            .runtime
            .mutate(bank_server::mutations::recognize_estate_executor(
                source_intent,
            ))
            .as_principal(source_actor)
            .controls(bank_server::BankMutationControls::new(
                (*source_scope).clone(),
                (*source_key).clone(),
            ))
            .retain_operation()
            .unwrap_or_else(|cause| panic!("exact declared estate construction: {cause:?}"));
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
            source_completed,
        ) = source_operation.advance(&source_request)
        else {
            panic!("native source completion required")
        };
        assert!(matches!(
            source_completed.evidence(),
            worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Recovered { .. }
        ));
        let closed_source = source_completed
            .close_with_evidence(&source_request)
            .unwrap_or_else(|(_, cause)| panic!("lawful native source close: {cause:?}"));
        bank_server::BankCommitReceipt::from_closed_source(closed_source)
    };
    let recovered = retry;
    assert_eq!(receipt.aftermath(), recovered.aftermath());
    assert_admission_derives_the_request_identities_once(recovered.canonical_work());
}

#[test]
fn command_authorization_does_not_make_a_foreign_authority_lawful() {
    let fixture = foreign_authority_world("recognize-executor-foreign-authority");
    let (_denial_source, denial) = {
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Refused { operation: original_source, denial: worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::Handler(worth_query_host::facade::primary_graph::MutationHandlerExecutionDenial::Handler(cause))) } = recognize(&fixture, fixture.executor, 51) else { panic!("native pre-effect refusal") };
        (
            original_source,
            cause
                .downcast::<bank_server::BankExecutorRecognitionProjectionDenial>()
                .expect("native projection cause"),
        )
    };

    assert!(matches!(
        denial,
        BankExecutorRecognitionProjectionDenial::AuthorityEstateMismatch
    ));
    assert!(estate_executors(&fixture).is_empty());
}

#[test]
fn selected_authority_must_name_the_commanded_executor() {
    let fixture = holder_mismatch_world("recognize-executor-holder-mismatch");
    let (_denial_source, denial) = {
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Refused { operation: original_source, denial: worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::Handler(worth_query_host::facade::primary_graph::MutationHandlerExecutionDenial::Handler(cause))) } = recognize(&fixture, fixture.executor, 61) else { panic!("native pre-effect refusal") };
        (
            original_source,
            cause
                .downcast::<bank_server::BankExecutorRecognitionProjectionDenial>()
                .expect("native projection cause"),
        )
    };

    assert!(matches!(
        denial,
        BankExecutorRecognitionProjectionDenial::AuthorityHolderMismatch
    ));
    assert!(estate_executors(&fixture).is_empty());
}

#[test]
fn unrecognized_authority_and_duplicate_executor_state_are_denied() {
    let unrecognized = unrecognized_authority_world("recognize-executor-unrecognized");
    let (_denial_source, denial) = {
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Refused { operation: original_source, denial: worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::Handler(worth_query_host::facade::primary_graph::MutationHandlerExecutionDenial::Handler(cause))) } = recognize(&unrecognized, unrecognized.executor, 71) else { panic!("native pre-effect refusal") };
        (
            original_source,
            cause
                .downcast::<bank_server::BankExecutorRecognitionProjectionDenial>()
                .expect("native projection cause"),
        )
    };
    assert!(matches!(
        denial,
        BankExecutorRecognitionProjectionDenial::AuthorityNotRecognized
    ));
    assert!(estate_executors(&unrecognized).is_empty());

    let duplicate = duplicate_executor_world("recognize-executor-duplicate");
    let (_denial_source, denial) = {
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Refused { operation: original_source, denial: worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::Handler(worth_query_host::facade::primary_graph::MutationHandlerExecutionDenial::Handler(cause))) } = recognize(&duplicate, duplicate.executor, 72) else { panic!("native pre-effect refusal") };
        (
            original_source,
            cause
                .downcast::<bank_server::BankExecutorRecognitionProjectionDenial>()
                .expect("native projection cause"),
        )
    };
    assert!(matches!(
        denial,
        BankExecutorRecognitionProjectionDenial::AlreadyRecognizedExecutor
    ));
    assert_eq!(estate_executors(&duplicate), [duplicate.executor]);
}
