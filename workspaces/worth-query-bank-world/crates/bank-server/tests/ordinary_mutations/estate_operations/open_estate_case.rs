#[path = "open_estate_case/fixture.rs"]
mod fixture;

use bank_domain::{
    estate::{DeathNoticeStatus, EstateCaseStatus, EstateWorkflowStage},
    proposals::BankIdempotencyKey,
};
use bank_server::{
    queries, BankAuthenticatedPrincipal, BankCommitReceipt, BankEstateCaseOpeningProjectionDenial,
    BankEstateProgressionDenial, BankMutationCommitOutcome, BankReadControls,
};

use self::fixture::{case_opening_world, CaseOpeningFixture};
use crate::support::request_scope;

#[test]
fn public_query_progression_opens_the_exact_verified_estate_case() {
    let fixture = case_opening_world(
        "estate-case-opening-commit",
        EstateCaseStatus::PendingOpening,
        DeathNoticeStatus::Verified,
    );
    let specialist = fixture.authenticate_specialist();
    let key = BankIdempotencyKey::new("open-estate-case-program-entry").unwrap();
    let outcome = fixture
        .world
        .runtime
        .open_estate_case_with_key(
            &specialist,
            fixture.action(fixture.notice),
            &key,
            &request_scope(),
        )
        .expect("the exact verified notice should open its pending estate case");

    let BankMutationCommitOutcome::Committed(receipt) = outcome else {
        panic!("the first exact opening must commit: {outcome:?}");
    };
    assert_eq!(receipt.changed_record_count(), 2);
    assert_eq!(receipt.emitted_effect_count(), 0);
    assert_eq!(receipt.expected_fact_count(), 0);
    assert_eq!(receipt.decision_fact_count(), Some(7));
    assert_zero_canonical_work(receipt.canonical_work());
    assert_case_posture(
        &fixture,
        EstateCaseStatus::Open,
        DeathNoticeStatus::Verified,
    );

    assert_equivalent_retry(&fixture, &specialist, &key, &receipt);
    assert_intent_drift(&fixture, &specialist, &key);
}

fn assert_equivalent_retry(
    fixture: &CaseOpeningFixture,
    specialist: &BankAuthenticatedPrincipal,
    key: &BankIdempotencyKey,
    committed: &BankCommitReceipt,
) {
    let retry = fixture
        .world
        .runtime
        .open_estate_case_with_key(
            specialist,
            fixture.action(fixture.notice),
            key,
            &request_scope(),
        )
        .expect("an equivalent authorized retry should inspect Query idempotency");
    let BankMutationCommitOutcome::AlreadyCommitted(recovered) = retry else {
        panic!("the equivalent retry must recover the exact commit: {retry:?}");
    };
    assert_eq!(committed.aftermath(), recovered.aftermath());
    assert_zero_canonical_work(recovered.canonical_work());
}

fn assert_intent_drift(
    fixture: &CaseOpeningFixture,
    specialist: &BankAuthenticatedPrincipal,
    key: &BankIdempotencyKey,
) {
    let denial = fixture
        .world
        .runtime
        .open_estate_case_with_key(
            specialist,
            fixture.action(fixture.foreign_notice),
            key,
            &request_scope(),
        )
        .expect_err("intent drift must stop before poststate projection");
    assert!(matches!(
        denial,
        BankEstateProgressionDenial::IdempotencyIntentDrift
    ));
}

#[test]
fn foreign_verified_notice_reaches_projection_but_cannot_open_the_case() {
    let fixture = case_opening_world(
        "estate-case-opening-foreign-notice",
        EstateCaseStatus::PendingOpening,
        DeathNoticeStatus::Verified,
    );
    let specialist = fixture.authenticate_specialist();
    let denial = fixture
        .world
        .runtime
        .open_estate_case_with_key(
            &specialist,
            fixture.action(fixture.foreign_notice),
            &idempotency(21),
            &request_scope(),
        )
        .expect_err("a real notice from another estate must fail exact projection");

    assert!(matches!(
        denial,
        BankEstateProgressionDenial::CaseOpeningProjection(
            BankEstateCaseOpeningProjectionDenial::NoticeMismatch
        )
    ));
    assert_case_posture(
        &fixture,
        EstateCaseStatus::PendingOpening,
        DeathNoticeStatus::Verified,
    );
}

#[test]
fn only_a_verified_notice_may_enter_case_opening() {
    let mut first_public_debug = None;
    for (ordinal, notice_status) in [
        DeathNoticeStatus::Reported,
        DeathNoticeStatus::NotificationRequested,
        DeathNoticeStatus::Rejected,
    ]
    .into_iter()
    .enumerate()
    {
        let fixture = case_opening_world(
            &format!("estate-case-opening-notice-{notice_status:?}"),
            EstateCaseStatus::PendingOpening,
            notice_status,
        );
        let specialist = fixture.authenticate_specialist();
        let denial = fixture
            .world
            .runtime
            .open_estate_case_with_key(
                &specialist,
                fixture.action(fixture.notice),
                &idempotency(31 + ordinal as u8),
                &request_scope(),
            )
            .expect_err("only external verified truth may support case opening");
        assert!(matches!(
            denial,
            BankEstateProgressionDenial::CaseOpeningProjection(
                BankEstateCaseOpeningProjectionDenial::NoticeNotVerified
            )
        ));
        let public_debug = format!("{denial:?}");
        if let Some(first) = &first_public_debug {
            assert_eq!(first, &public_debug);
        } else {
            first_public_debug = Some(public_debug);
        }
        assert_case_posture(&fixture, EstateCaseStatus::PendingOpening, notice_status);
    }
}

#[test]
fn only_a_pending_case_may_enter_case_opening() {
    let mut first_public_debug = None;
    for (ordinal, case_status) in [
        EstateCaseStatus::Open,
        EstateCaseStatus::Released,
        EstateCaseStatus::Closed,
    ]
    .into_iter()
    .enumerate()
    {
        let fixture = case_opening_world(
            &format!("estate-case-opening-status-{case_status:?}"),
            case_status,
            DeathNoticeStatus::Verified,
        );
        let specialist = fixture.authenticate_specialist();
        let denial = fixture
            .world
            .runtime
            .open_estate_case_with_key(
                &specialist,
                fixture.action(fixture.notice),
                &idempotency(41 + ordinal as u8),
                &request_scope(),
            )
            .expect_err("a fresh intent cannot repeat or reverse case opening");
        assert!(matches!(
            denial,
            BankEstateProgressionDenial::CaseOpeningProjection(
                BankEstateCaseOpeningProjectionDenial::CaseNotPendingOpening
            )
        ));
        let public_debug = format!("{denial:?}");
        if let Some(first) = &first_public_debug {
            assert_eq!(first, &public_debug);
        } else {
            first_public_debug = Some(public_debug);
        }
        assert_case_posture(&fixture, case_status, DeathNoticeStatus::Verified);
    }
}

fn assert_case_posture(
    fixture: &CaseOpeningFixture,
    expected_case: EstateCaseStatus,
    expected_notice: DeathNoticeStatus,
) {
    let specialist = fixture.authenticate_specialist();
    let result = fixture
        .world
        .runtime
        .query(queries::estate_case(fixture.estate))
        .as_principal(&specialist)
        .controls(BankReadControls::current(request_scope(), 16, 20_000).unwrap())
        .execute()
        .expect("the assigned specialist should observe the estate case");
    let overview = &result.rows()[0];
    assert_eq!(overview.stage(), EstateWorkflowStage::Administration);
    assert_eq!(overview.status(), expected_case);
    assert_eq!(overview.death_notice().id(), fixture.notice);
    assert_eq!(overview.death_notice().status(), expected_notice);
}

fn idempotency(identity: u8) -> BankIdempotencyKey {
    BankIdempotencyKey::new(format!("open-estate-case-{identity}")).unwrap()
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
