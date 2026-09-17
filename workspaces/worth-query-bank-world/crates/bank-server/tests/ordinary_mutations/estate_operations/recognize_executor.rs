#[path = "recognize_executor/fixture.rs"]
mod fixture;

use bank_domain::proposals::BankIdempotencyKey;
use bank_server::{
    queries, BankEstateProgressionDenial, BankExecutorRecognitionProjectionDenial,
    BankMutationCommitOutcome, BankReadControls,
};

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
    let outcome = fixture
        .world
        .runtime
        .recognize_estate_executor_with_key(
            &specialist,
            fixture.action(fixture.executor),
            &key,
            &request_scope(),
        )
        .expect("the exact legal authority should authorize one executor relation");

    let BankMutationCommitOutcome::Committed(receipt) = outcome else {
        panic!("the first exact recognition must commit: {outcome:?}");
    };
    assert_eq!(receipt.emitted_effect_count(), 0);
    assert_eq!(receipt.expected_fact_count(), 0);
    assert_eq!(receipt.decision_fact_count(), Some(8));
    assert_zero_canonical_work(receipt.canonical_work());
    assert_eq!(estate_executors(&fixture), [fixture.executor]);
    assert!(estate_authority_is_still_recognized(&fixture));

    let retry = fixture
        .world
        .runtime
        .recognize_estate_executor_with_key(
            &specialist,
            fixture.action(fixture.executor),
            &key,
            &request_scope(),
        )
        .expect("an equivalent retry should recover before duplicate-state denial");
    let BankMutationCommitOutcome::AlreadyCommitted(recovered) = retry else {
        panic!("the equivalent retry must recover the exact commit: {retry:?}");
    };
    assert_eq!(receipt.aftermath(), recovered.aftermath());
    assert_zero_canonical_work(recovered.canonical_work());
}

#[test]
fn command_authorization_does_not_make_a_foreign_authority_lawful() {
    let fixture = foreign_authority_world("recognize-executor-foreign-authority");
    let denial = recognize(&fixture, fixture.executor, 51)
        .expect_err("authority from another estate must fail invariant projection");

    assert!(matches!(
        denial,
        BankEstateProgressionDenial::ExecutorRecognitionProjection(
            BankExecutorRecognitionProjectionDenial::AuthorityEstateMismatch
        )
    ));
    assert!(estate_executors(&fixture).is_empty());
}

#[test]
fn selected_authority_must_name_the_commanded_executor() {
    let fixture = holder_mismatch_world("recognize-executor-holder-mismatch");
    let denial = recognize(&fixture, fixture.executor, 61)
        .expect_err("authority held by another principal must not create the requested edge");

    assert!(matches!(
        denial,
        BankEstateProgressionDenial::ExecutorRecognitionProjection(
            BankExecutorRecognitionProjectionDenial::AuthorityHolderMismatch
        )
    ));
    assert!(estate_executors(&fixture).is_empty());
}

#[test]
fn unrecognized_authority_and_duplicate_executor_state_are_denied() {
    let unrecognized = unrecognized_authority_world("recognize-executor-unrecognized");
    let denial = recognize(&unrecognized, unrecognized.executor, 71)
        .expect_err("an unrecognized legal record must not create executor authority");
    assert!(matches!(
        denial,
        BankEstateProgressionDenial::ExecutorRecognitionProjection(
            BankExecutorRecognitionProjectionDenial::AuthorityNotRecognized
        )
    ));
    assert!(estate_executors(&unrecognized).is_empty());

    let duplicate = duplicate_executor_world("recognize-executor-duplicate");
    let denial = recognize(&duplicate, duplicate.executor, 72)
        .expect_err("a new key must not recreate an existing executor edge");
    assert!(matches!(
        denial,
        BankEstateProgressionDenial::ExecutorRecognitionProjection(
            BankExecutorRecognitionProjectionDenial::AlreadyRecognizedExecutor
        )
    ));
    assert_eq!(estate_executors(&duplicate), [duplicate.executor]);
}

fn recognize(
    fixture: &RecognitionFixture,
    executor: bank_domain::model::BankPrincipalId,
    identity: u8,
) -> Result<BankMutationCommitOutcome, BankEstateProgressionDenial> {
    let key = idempotency(identity);
    fixture.world.runtime.recognize_estate_executor_with_key(
        &fixture.authenticate_specialist(),
        fixture.action(executor),
        &key,
        &request_scope(),
    )
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

fn assert_zero_canonical_work(phases: bank_server::BankCommitCanonicalWorkPhases) {
    for work in [
        phases.installation(),
        phases.admission(),
        phases.execution(),
        phases.provider_commit(),
        phases.projection(),
        phases.live_delivery(),
        phases.retry_resolution(),
        phases.recovery_inspection(),
        phases.publication(),
    ] {
        assert_eq!(work.basis_preparations(), 0);
        assert_eq!(work.digest_derivations(), 0);
        assert_eq!(work.canonical_encoded_bytes(), 0);
        assert_eq!(work.digest_text_materializations(), 0);
    }
}
