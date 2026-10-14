#[path = "notify_death/fixture.rs"]
pub(super) mod fixture;

use bank_domain::{
    estate::DeathNoticeStatus, model::BankPrincipalId, proposals::BankIdempotencyKey,
};
use bank_server::{queries, BankDeathNotificationProjectionDenial, BankReadControls};

use self::fixture::{notification_world, NotificationFixture};
use crate::support::request_scope;

#[test]
fn public_query_observes_one_committed_notification_request() {
    let fixture = notification_world("notify-death-commit", DeathNoticeStatus::Reported);
    let specialist = fixture.authenticate_specialist();
    let key = BankIdempotencyKey::new("notify-death-program-entry").unwrap();
    let outcome = {
        let source_actor = &specialist;
        let source_scope = &request_scope();
        let source_key = &key;
        let source_intent = bank_domain::schema::NotifyEstateDeath::try_from(
            fixture.action(fixture.notice, fixture.deceased),
        )
        .expect("typed estate command preserves the exact variant");
        let source_request = fixture.world.runtime.request(source_actor, source_scope);
        let source_operation = fixture
            .world
            .runtime
            .mutate(bank_server::mutations::notify_estate_death(source_intent))
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
    assert_admission_derives_the_request_identities_once(receipt.canonical_work());
    assert_eq!(
        notice_status(&fixture),
        DeathNoticeStatus::NotificationRequested
    );

    let retry = {
        let source_actor = &specialist;
        let source_scope = &request_scope();
        let source_key = &key;
        let source_intent = bank_domain::schema::NotifyEstateDeath::try_from(
            fixture.action(fixture.notice, fixture.deceased),
        )
        .expect("typed estate command preserves the exact variant");
        let source_request = fixture.world.runtime.request(source_actor, source_scope);
        let source_operation = fixture
            .world
            .runtime
            .mutate(bank_server::mutations::notify_estate_death(source_intent))
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
    assert!(recovered.co_committed_dispatch_outbox());
    assert_eq!(recovered.emitted_effect_count(), 1);
    assert_admission_derives_the_request_identities_once(recovered.canonical_work());

    let _drift = {
        let source_actor = &specialist;
        let source_scope = &request_scope();
        let source_key = &key;
        let source_intent = bank_domain::schema::NotifyEstateDeath::try_from(
            fixture.action(fixture.foreign_notice, fixture.deceased),
        )
        .expect("typed estate command preserves the exact variant");
        let source_request = fixture.world.runtime.request(source_actor, source_scope);
        let source_operation = fixture
            .world
            .runtime
            .mutate(bank_server::mutations::notify_estate_death(source_intent))
            .as_principal(source_actor)
            .controls(bank_server::BankMutationControls::new(
                (*source_scope).clone(),
                (*source_key).clone(),
            ))
            .retain_operation()
            .unwrap_or_else(|(_, cause)| panic!("exact declared estate construction: {cause:?}"));
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Terminal(
            source_terminal,
        ) = source_operation.advance(&source_request)
        else {
            panic!("exact keyed intent drift is terminal")
        };
        assert!(matches!(source_terminal.native_outcome(), Some(worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift)));
        source_terminal
            .close(&source_request)
            .unwrap_or_else(|(_, cause)| panic!("lawful drift close: {cause:?}"))
    };
}

#[test]
fn command_authority_cannot_substitute_a_foreign_notice_or_subject() {
    let fixture = notification_world("notify-death-hostile-input", DeathNoticeStatus::Reported);
    let (_foreign_source, foreign) = {
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Refused { operation: original_source, denial: worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::Handler(worth_query_host::facade::primary_graph::MutationHandlerExecutionDenial::Handler(cause))) } = notify(
        &fixture,
        fixture.foreign_notice,
        fixture.foreign_deceased,
        91,
    ) else { panic!("native pre-effect refusal") };
        (
            original_source,
            cause
                .downcast::<bank_server::BankDeathNotificationProjectionDenial>()
                .expect("native projection cause"),
        )
    };
    assert!(matches!(
        foreign,
        BankDeathNotificationProjectionDenial::NoticeMismatch
    ));
    assert_eq!(notice_status(&fixture), DeathNoticeStatus::Reported);

    let (_wrong_subject_source, wrong_subject) = {
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Refused { operation: original_source, denial: worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::Handler(worth_query_host::facade::primary_graph::MutationHandlerExecutionDenial::Handler(cause))) } = notify(&fixture, fixture.notice, fixture.other_subject, 92) else { panic!("native pre-effect refusal") };
        (
            original_source,
            cause
                .downcast::<bank_server::BankDeathNotificationProjectionDenial>()
                .expect("native projection cause"),
        )
    };
    assert!(matches!(
        wrong_subject,
        BankDeathNotificationProjectionDenial::NoticeSubjectMismatch
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
        let (_denial_source, denial) = {
            let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Refused { operation: original_source, denial: worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::Handler(worth_query_host::facade::primary_graph::MutationHandlerExecutionDenial::Handler(cause))) } = notify(
            &fixture,
            fixture.notice,
            fixture.deceased,
            101 + ordinal as u8,
        ) else { panic!("native pre-effect refusal") };
            (
                original_source,
                cause
                    .downcast::<bank_server::BankDeathNotificationProjectionDenial>()
                    .expect("native projection cause"),
            )
        };
        assert!(matches!(
            denial,
            BankDeathNotificationProjectionDenial::NoticeNotReported
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
) -> worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance<
    worth_query_host::facade::application_entry::WorthQuerySourceOnlyOperation<
        bank_domain::schema::BankSchema,
        bank_domain::schema::NotifyEstateDeath,
        bank_server::BankApplication,
        bank_server::DeathNotificationAction,
        worth_query_host::facade::declaration::application_program::ApplicationSourceCompletion,
    >,
    worth_query_host::facade::application_entry::WorthQueryCompletedSourceOperation<
        bank_domain::schema::BankSchema,
        bank_domain::schema::NotifyEstateDeath,
        bank_server::BankApplication,
        bank_server::DeathNotificationAction,
        worth_query_host::facade::declaration::application_program::ApplicationSourceCompletion,
    >,
    worth_query_host::facade::application_entry::WorthQueryTerminalSourceOperation<
        bank_domain::schema::BankSchema,
        bank_domain::schema::NotifyEstateDeath,
        bank_server::BankApplication,
        bank_server::DeathNotificationAction,
        worth_query_host::facade::declaration::application_program::ApplicationSourceCompletion,
    >,
> {
    let key = idempotency(identity);
    {
        let source_actor = &fixture.authenticate_specialist();
        let source_scope = &request_scope();
        let source_key = &key;
        let source_intent =
            bank_domain::schema::NotifyEstateDeath::try_from(fixture.action(notice, subject))
                .expect("typed estate command preserves the exact variant");
        let source_request = fixture.world.runtime.request(source_actor, source_scope);
        let source_operation = fixture
            .world
            .runtime
            .mutate(bank_server::mutations::notify_estate_death(source_intent))
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
    assert_eq!(admission.canonical_encoded_bytes(), 256);
    assert_eq!(admission.sha256_input_bytes(), 475);
    assert_eq!(admission.sha256_compression_blocks(), 9);
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
