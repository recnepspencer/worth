#[path = "recognize_executor/adoption.rs"]
mod adoption;
#[path = "recognize_executor/fixture.rs"]
mod fixture;

use bank_domain::proposals::BankIdempotencyKey;
use bank_server::{queries, BankExecutorRecognitionProjectionDenial, BankReadControls};

use self::fixture::{
    duplicate_executor_world, exact_recognition_world, foreign_authority_world,
    holder_mismatch_world, unrecognized_authority_world, RecognitionFixture,
};
use crate::support::request_scope;

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
            .unwrap_or_else(|(_, cause)| panic!("exact declared estate construction: {cause:?}"));
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
            .unwrap_or_else(|(_, cause)| panic!("exact declared estate construction: {cause:?}"));
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

fn recognize(
    fixture: &RecognitionFixture,
    executor: bank_domain::model::BankPrincipalId,
    identity: u8,
) -> worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance<
    worth_query_host::facade::application_entry::WorthQuerySourceOnlyOperation<
        bank_domain::schema::BankSchema,
        bank_domain::schema::RecognizeEstateExecutor,
        bank_server::BankApplication,
        bank_server::ExecutorRecognitionAction,
        worth_query_host::facade::declaration::application_program::ApplicationSourceCompletion,
    >,
    worth_query_host::facade::application_entry::WorthQueryCompletedSourceOperation<
        bank_domain::schema::BankSchema,
        bank_domain::schema::RecognizeEstateExecutor,
        bank_server::BankApplication,
        bank_server::ExecutorRecognitionAction,
        worth_query_host::facade::declaration::application_program::ApplicationSourceCompletion,
    >,
    worth_query_host::facade::application_entry::WorthQueryTerminalSourceOperation<
        bank_domain::schema::BankSchema,
        bank_domain::schema::RecognizeEstateExecutor,
        bank_server::BankApplication,
        bank_server::ExecutorRecognitionAction,
        worth_query_host::facade::declaration::application_program::ApplicationSourceCompletion,
    >,
> {
    let key = idempotency(identity);
    {
        let source_actor = &fixture.authenticate_specialist();
        let source_scope = &request_scope();
        let source_key = &key;
        let source_intent =
            bank_domain::schema::RecognizeEstateExecutor::try_from(fixture.action(executor))
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
            .unwrap_or_else(|(_, cause)| panic!("exact declared estate construction: {cause:?}"));
        source_operation.advance(&source_request)
    }
}

fn estate_executors(fixture: &RecognitionFixture) -> Vec<bank_domain::model::BankPrincipalId> {
    estate_overview(fixture).executors().to_vec()
}

fn estate_authority_is_still_recognized(fixture: &RecognitionFixture) -> bool {
    estate_overview(fixture)
        .legal_authorities()
        .iter()
        .any(|authority| authority.id() == fixture.authority && authority.recognized())
}

fn estate_overview(fixture: &RecognitionFixture) -> bank_domain::reads::EstateCaseOverview {
    fixture
        .world
        .runtime
        .query(queries::estate_case(fixture.estate))
        .as_principal(&fixture.authenticate_specialist())
        .controls(BankReadControls::current(request_scope(), 16, 20_000).unwrap())
        .execute()
        .expect("the assigned specialist should observe the estate")
        .rows()[0]
        .clone()
}

fn idempotency(identity: u8) -> BankIdempotencyKey {
    BankIdempotencyKey::new(format!("recognize-estate-executor-{identity}")).unwrap()
}

fn assert_admission_derives_the_request_identities_once(
    phases: bank_server::BankCommitCanonicalWorkPhases,
) {
    // Binding-selected capability admission governs its input as well as the mutation
    // input/key identities. All three native derivations are included here; later
    // publication and domain recovery do not rebuild these identities.
    let admission = phases.admission();
    assert_eq!(admission.basis_preparations(), 0);
    assert_eq!(admission.digest_derivations(), 3);
    assert_eq!(admission.canonical_entries(), 3);
    assert_eq!(admission.canonical_encoded_bytes(), 295);
    assert_eq!(admission.sha256_input_bytes(), 514);
    assert_eq!(admission.sha256_compression_blocks(), 11);
    assert_eq!(admission.digest_text_materializations(), 0);
    for work in [
        phases.installation(),
        phases.execution(),
        phases.provider_commit(),
        phases.projection(),
        phases.retry_resolution(),
        phases.publication(),
    ] {
        assert_eq!(work.basis_preparations(), 0);
        assert_eq!(work.digest_derivations(), 0);
        assert_eq!(work.canonical_encoded_bytes(), 0);
        assert_eq!(work.digest_text_materializations(), 0);
    }
}
