#[path = "notify_death/fixture.rs"]
pub(super) mod fixture;

use bank_domain::{
    estate::DeathNoticeStatus, model::BankPrincipalId, proposals::BankIdempotencyKey,
};
use bank_server::{
    queries, BankDeathNotificationProjectionDenial, BankEstateProgressionDenial,
    BankMutationCommitOutcome, BankReadControls,
};

use self::fixture::{notification_world, NotificationFixture};
use crate::support::request_scope;

#[test]
fn public_query_observes_one_committed_notification_request() {
    let fixture = notification_world("notify-death-commit", DeathNoticeStatus::Reported);
    let specialist = fixture.authenticate_specialist();
    let key = BankIdempotencyKey::new("notify-death-program-entry").unwrap();
    let outcome = fixture
        .world
        .runtime
        .notify_estate_death_with_key(
            &specialist,
            fixture.action(fixture.notice, fixture.deceased),
            &key,
            &request_scope(),
        )
        .expect("the exact reported notice should commit one notification request");

    let BankMutationCommitOutcome::Committed(receipt) = outcome else {
        panic!("the exact death notification must commit: {outcome:?}");
    };
    assert_eq!(
        receipt.changed_record_count(),
        3,
        "the notice, its estate, and the co-committed dispatch outbox row change together"
    );
    assert!(receipt.co_committed_dispatch_outbox());
    assert!(
        receipt.external_dispatch_posture().is_none(),
        "a bank with no installed rail commits the outbox and dispatches nothing"
    );
    assert_eq!(receipt.emitted_effect_count(), 1);
    assert_eq!(receipt.expected_fact_count(), 0);
    assert_eq!(receipt.decision_fact_count(), Some(8));
    assert_zero_canonical_work(receipt.canonical_work());
    assert_eq!(
        notice_status(&fixture),
        DeathNoticeStatus::NotificationRequested
    );

    let retry = fixture
        .world
        .runtime
        .notify_estate_death_with_key(
            &specialist,
            fixture.action(fixture.notice, fixture.deceased),
            &key,
            &request_scope(),
        )
        .expect("equivalent retry should recover before poststate inspection");
    let BankMutationCommitOutcome::AlreadyCommitted(recovered) = retry else {
        panic!("equivalent notification retry must recover: {retry:?}");
    };
    assert_eq!(receipt.aftermath(), recovered.aftermath());
    assert!(recovered.co_committed_dispatch_outbox());
    assert_eq!(recovered.emitted_effect_count(), 1);
    assert_zero_canonical_work(recovered.canonical_work());

    let drift = fixture
        .world
        .runtime
        .notify_estate_death_with_key(
            &specialist,
            fixture.action(fixture.foreign_notice, fixture.deceased),
            &key,
            &request_scope(),
        )
        .expect_err("the same key with a different input must retain typed drift");
    assert!(matches!(
        drift,
        BankEstateProgressionDenial::IdempotencyIntentDrift
    ));
}

#[test]
fn command_authority_cannot_substitute_a_foreign_notice_or_subject() {
    let fixture = notification_world("notify-death-hostile-input", DeathNoticeStatus::Reported);
    let foreign = notify(
        &fixture,
        fixture.foreign_notice,
        fixture.foreign_deceased,
        91,
    )
    .expect_err("an unrelated real notice must fail exact estate projection");
    assert!(matches!(
        foreign,
        BankEstateProgressionDenial::DeathNotificationProjection(
            BankDeathNotificationProjectionDenial::NoticeMismatch
        )
    ));
    assert_eq!(notice_status(&fixture), DeathNoticeStatus::Reported);

    let wrong_subject = notify(&fixture, fixture.notice, fixture.other_subject, 92)
        .expect_err("another real principal must not replace the notice subject");
    assert!(matches!(
        wrong_subject,
        BankEstateProgressionDenial::DeathNotificationProjection(
            BankDeathNotificationProjectionDenial::NoticeSubjectMismatch
        )
    ));
    assert_eq!(notice_status(&fixture), DeathNoticeStatus::Reported);
}

#[test]
fn only_reported_notices_may_request_notification_causality() {
    let mut first_public_debug = None;
    for (ordinal, status) in [
        DeathNoticeStatus::NotificationRequested,
        DeathNoticeStatus::Verified,
        DeathNoticeStatus::Rejected,
    ]
    .into_iter()
    .enumerate()
    {
        let fixture = notification_world(&format!("notify-death-{status:?}"), status);
        let denial = notify(
            &fixture,
            fixture.notice,
            fixture.deceased,
            101 + ordinal as u8,
        )
        .expect_err("non-reported posture must not emit another request");
        assert!(matches!(
            denial,
            BankEstateProgressionDenial::DeathNotificationProjection(
                BankDeathNotificationProjectionDenial::NoticeNotReported
            )
        ));
        let public_debug = format!("{denial:?}");
        if let Some(first) = &first_public_debug {
            assert_eq!(first, &public_debug);
        } else {
            first_public_debug = Some(public_debug);
        }
        assert_eq!(notice_status(&fixture), status);
    }
}

fn notify(
    fixture: &NotificationFixture,
    notice: bank_domain::estate::DeathNoticeId,
    subject: BankPrincipalId,
    identity: u8,
) -> Result<BankMutationCommitOutcome, BankEstateProgressionDenial> {
    let key = idempotency(identity);
    fixture.world.runtime.notify_estate_death_with_key(
        &fixture.authenticate_specialist(),
        fixture.action(notice, subject),
        &key,
        &request_scope(),
    )
}

fn notice_status(fixture: &NotificationFixture) -> DeathNoticeStatus {
    fixture
        .world
        .runtime
        .query(queries::estate_case(fixture.estate))
        .as_principal(&fixture.authenticate_specialist())
        .controls(BankReadControls::current(request_scope(), 16, 20_000).unwrap())
        .execute()
        .expect("the assigned specialist should observe the death notice")
        .rows()[0]
        .death_notice()
        .status()
}

fn idempotency(identity: u8) -> BankIdempotencyKey {
    BankIdempotencyKey::new(format!("notify-death-{identity}")).unwrap()
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
