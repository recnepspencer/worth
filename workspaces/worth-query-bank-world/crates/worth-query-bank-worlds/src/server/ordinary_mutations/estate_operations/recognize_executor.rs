#[path = "recognize_executor/adoption.rs"]
pub mod adoption;
#[path = "recognize_executor/fixture.rs"]
pub mod fixture;

pub use bank_domain::proposals::BankIdempotencyKey;
pub use bank_server::{queries, BankExecutorRecognitionProjectionDenial, BankReadControls};

pub use self::fixture::{
    duplicate_executor_world, exact_recognition_world, foreign_authority_world,
    holder_mismatch_world, unrecognized_authority_world, RecognitionFixture,
};
pub use crate::facade::server::support::request_scope;

pub fn recognize(
    fixture: &RecognitionFixture,
    executor: bank_domain::model::BankPrincipalId,
    identity: u8,
) -> super::EstateSourceAdvance<
    bank_domain::schema::RecognizeEstateExecutor,
    bank_server::ExecutorRecognitionAction,
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
            .unwrap_or_else(|cause| panic!("exact declared estate construction: {cause:?}"));
        source_operation.advance(&source_request)
    }
}

pub fn estate_executors(fixture: &RecognitionFixture) -> Vec<bank_domain::model::BankPrincipalId> {
    estate_overview(fixture).executors().to_vec()
}

pub fn estate_authority_is_still_recognized(fixture: &RecognitionFixture) -> bool {
    estate_overview(fixture)
        .legal_authorities()
        .iter()
        .any(|authority| authority.id() == fixture.authority && authority.recognized())
}

pub fn estate_overview(fixture: &RecognitionFixture) -> bank_domain::reads::EstateCaseOverview {
    fixture
        .world
        .runtime
        .query(queries::estate_case(fixture.estate))
        .as_principal(&fixture.authenticate_specialist())
        .controls(BankReadControls::current(request_scope(), 16).unwrap())
        .execute()
        .expect("the assigned specialist should observe the estate")
        .rows()[0]
        .clone()
}

pub fn idempotency(identity: u8) -> BankIdempotencyKey {
    BankIdempotencyKey::new(format!("recognize-estate-executor-{identity}")).unwrap()
}

pub fn assert_admission_derives_the_request_identities_once(
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
