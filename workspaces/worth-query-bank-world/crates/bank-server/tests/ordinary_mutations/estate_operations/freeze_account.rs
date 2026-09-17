#[path = "freeze_account/fixture.rs"]
pub(crate) mod fixture;

use bank_domain::{
    estate::EstateWorkflowStage, proposals::BankIdempotencyKey, schema::AccountStatus,
};
use bank_server::{
    queries, BankEstateFreezeProjectionDenial, BankEstateProgressionDenial,
    BankMutationCommitOutcome, BankReadControls,
};

use self::fixture::{exact_freeze_world, foreign_account_freeze_world, FreezeFixture};
use crate::support::request_scope;

#[test]
fn public_query_progression_freezes_the_exact_estate_account() {
    let fixture = exact_freeze_world("estate-freeze-commit", AccountStatus::Open);
    let specialist = fixture.authenticate_specialist();
    let key = BankIdempotencyKey::new("freeze-estate-account-program-entry").unwrap();
    let outcome = fixture
        .world
        .runtime
        .freeze_estate_account_with_key(
            &specialist,
            fixture.action(fixture.estate_account),
            &key,
            &request_scope(),
        )
        .expect("the exact estate account should reach Query commit");

    let BankMutationCommitOutcome::Committed(receipt) = outcome else {
        panic!("the first exact freeze must commit: {outcome:?}");
    };
    assert_eq!(receipt.changed_record_count(), 2);
    assert_eq!(receipt.emitted_effect_count(), 0);
    assert_eq!(receipt.expected_fact_count(), 0);
    assert_eq!(receipt.decision_fact_count(), Some(5));
    assert_freeze_canonical_work(receipt.canonical_work());
    assert_eq!(estate_account_status(&fixture), AccountStatus::Frozen);

    let retry = fixture
        .world
        .runtime
        .freeze_estate_account_with_key(
            &specialist,
            fixture.action(fixture.estate_account),
            &key,
            &request_scope(),
        )
        .expect("an equivalent authorized retry should inspect Query idempotency");
    let BankMutationCommitOutcome::AlreadyCommitted(recovered) = retry else {
        panic!("the equivalent retry must recover the exact commit: {retry:?}");
    };
    assert_eq!(receipt.aftermath(), recovered.aftermath());
    assert!(recovered.retained_preimage());
    assert!(
        recovered.performed_preimage_retention_work(),
        "the installed recorded inverse must pay its explicit retention work"
    );
    assert_freeze_canonical_work(recovered.canonical_work());
    assert_eq!(estate_account_status(&fixture), AccountStatus::Frozen);

    let drift = fixture
        .world
        .runtime
        .freeze_estate_account_with_key(
            &specialist,
            fixture.action(fixture.foreign_account),
            &key,
            &request_scope(),
        )
        .expect_err("the same key with a different account must retain typed drift");
    assert!(
        matches!(drift, BankEstateProgressionDenial::IdempotencyIntentDrift),
        "unexpected freeze drift denial: {drift:?}"
    );
    assert_eq!(foreign_account_status(&fixture), AccountStatus::Open);
}

#[test]
fn estate_preview_keeps_its_exact_account_status_after_a_freeze_commit() {
    let fixture = exact_freeze_world("estate-preview-before-freeze", AccountStatus::Open);
    let specialist = fixture.authenticate_specialist();
    let session = fixture
        .world
        .runtime
        .open_preview(&request_scope())
        .unwrap();
    let before = fixture
        .world
        .runtime
        .query(queries::estate_case(fixture.estate))
        .as_principal(&specialist)
        .controls(read_controls())
        .preview(&session)
        .expect("preview must read the selected open account");
    assert_eq!(before.rows()[0].account().status(), AccountStatus::Open);

    let committed = fixture
        .world
        .runtime
        .freeze_estate_account_with_key(
            &specialist,
            fixture.action(fixture.estate_account),
            &idempotency(230),
            &request_scope(),
        )
        .expect("the freeze must commit after preview selection");
    assert!(matches!(committed, BankMutationCommitOutcome::Committed(_)));
    assert_eq!(estate_account_status(&fixture), AccountStatus::Frozen);

    let retained = fixture
        .world
        .runtime
        .query(queries::estate_case(fixture.estate))
        .as_principal(&specialist)
        .controls(read_controls())
        .preview(&session)
        .expect("preview must retain the pre-freeze product occurrence");
    assert_eq!(retained.rows()[0].account().status(), AccountStatus::Open);
    assert!(session.discard().unwrap().discarded());
}

#[test]
fn foreign_grant_account_reaches_projection_but_cannot_receive_effect_authority() {
    let fixture = foreign_account_freeze_world("estate-freeze-foreign-account");
    let specialist = fixture.authenticate_specialist();
    let denial = fixture
        .world
        .runtime
        .freeze_estate_account_with_key(
            &specialist,
            fixture.action(fixture.foreign_account),
            &idempotency(21),
            &request_scope(),
        )
        .expect_err("a grant-bound foreign account must fail after graph observation");

    assert!(matches!(
        denial,
        BankEstateProgressionDenial::FreezeProjection(
            BankEstateFreezeProjectionDenial::RelatedAccountMismatch
        )
    ));
    assert_eq!(estate_account_status(&fixture), AccountStatus::Open);
    assert_eq!(foreign_account_status(&fixture), AccountStatus::Open);
}

#[test]
fn non_open_accounts_cannot_begin_a_second_freeze_transition() {
    let mut first_public_debug = None;
    for (ordinal, status) in [AccountStatus::Frozen, AccountStatus::Closed]
        .into_iter()
        .enumerate()
    {
        let fixture = exact_freeze_world(&format!("estate-freeze-{status:?}"), status);
        let specialist = fixture.authenticate_specialist();
        let denial = fixture
            .world
            .runtime
            .freeze_estate_account_with_key(
                &specialist,
                fixture.action(fixture.estate_account),
                &idempotency(31 + ordinal as u8),
                &request_scope(),
            )
            .expect_err("only an open account may enter the freeze transition");

        assert!(matches!(
            denial,
            BankEstateProgressionDenial::FreezeProjection(
                BankEstateFreezeProjectionDenial::AccountNotOpen
            )
        ));
        let public_debug = format!("{denial:?}");
        if let Some(first) = &first_public_debug {
            assert_eq!(first, &public_debug);
        } else {
            first_public_debug = Some(public_debug);
        }
        assert_eq!(estate_account_status(&fixture), status);
    }
}

fn estate_account_status(fixture: &FreezeFixture) -> AccountStatus {
    let specialist = fixture.authenticate_specialist();
    let result = fixture
        .world
        .runtime
        .query(queries::estate_case(fixture.estate))
        .as_principal(&specialist)
        .controls(read_controls())
        .execute()
        .expect("the assigned specialist should observe the estate account");
    let overview = &result.rows()[0];
    assert_eq!(overview.stage(), EstateWorkflowStage::Administration);
    assert_eq!(overview.account().id(), fixture.estate_account);
    overview.account().status()
}

fn foreign_account_status(fixture: &FreezeFixture) -> AccountStatus {
    let owner = fixture.authenticate_foreign_owner();
    fixture
        .world
        .runtime
        .query(queries::account_summary(fixture.foreign_account))
        .as_principal(&owner)
        .controls(read_controls())
        .execute()
        .expect("the foreign account owner should observe their account")
        .rows()[0]
        .status()
}

fn read_controls() -> BankReadControls {
    BankReadControls::current(request_scope(), 16, 20_000).unwrap()
}

fn idempotency(identity: u8) -> BankIdempotencyKey {
    BankIdempotencyKey::new(format!("freeze-estate-account-{identity}")).unwrap()
}

fn assert_freeze_canonical_work(phases: bank_server::BankCommitCanonicalWorkPhases) {
    let input_identity = phases.admission();
    assert_eq!(input_identity.basis_preparations(), 1);
    assert_eq!(input_identity.digest_derivations(), 1);
    assert_eq!(input_identity.canonical_encoded_bytes(), 821);
    assert_eq!(input_identity.digest_text_materializations(), 0);
    for work in [
        phases.installation(),
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
