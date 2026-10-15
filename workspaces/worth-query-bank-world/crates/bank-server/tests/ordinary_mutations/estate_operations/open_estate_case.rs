#[path = "open_estate_case/fixture.rs"]
mod fixture;

use bank_domain::{
    estate::{DeathNoticeStatus, EstateCaseStatus, EstateWorkflowStage},
    proposals::BankIdempotencyKey,
};
use bank_server::{
    queries, BankAuthenticatedPrincipal, BankCommitReceipt, BankEstateCaseOpeningProjectionDenial,
    BankReadControls,
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
    let outcome = {
        let source_actor = &specialist;
        let source_scope = &request_scope();
        let source_key = &key;
        let source_intent =
            bank_domain::schema::OpenEstateCase::try_from(fixture.action(fixture.notice))
                .expect("typed estate command preserves the exact variant");
        let source_request = fixture.world.runtime.request(source_actor, source_scope);
        let source_operation = fixture
            .world
            .runtime
            .mutate(bank_server::mutations::open_estate_case(source_intent))
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
    assert_eq!(receipt.changed_record_count(), 2);
    assert_eq!(receipt.emitted_effect_count(), 0);
    assert_eq!(receipt.expected_fact_count(), 0);
    assert_eq!(receipt.decision_fact_count(), Some(7));
    assert_admission_derives_the_request_identities_once(receipt.canonical_work());
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
    let retry = {
        let source_actor = specialist;
        let source_scope = &request_scope();
        let source_key = key;
        let source_intent =
            bank_domain::schema::OpenEstateCase::try_from(fixture.action(fixture.notice))
                .expect("typed estate command preserves the exact variant");
        let source_request = fixture.world.runtime.request(source_actor, source_scope);
        let source_operation = fixture
            .world
            .runtime
            .mutate(bank_server::mutations::open_estate_case(source_intent))
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
    assert_eq!(committed.aftermath(), recovered.aftermath());
    assert_admission_derives_the_request_identities_once(recovered.canonical_work());
}

fn assert_intent_drift(
    fixture: &CaseOpeningFixture,
    specialist: &BankAuthenticatedPrincipal,
    key: &BankIdempotencyKey,
) {
    let _denial = {
        let source_actor = specialist;
        let source_scope = &request_scope();
        let source_key = key;
        let source_intent =
            bank_domain::schema::OpenEstateCase::try_from(fixture.action(fixture.foreign_notice))
                .expect("typed estate command preserves the exact variant");
        let source_request = fixture.world.runtime.request(source_actor, source_scope);
        let source_operation = fixture
            .world
            .runtime
            .mutate(bank_server::mutations::open_estate_case(source_intent))
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
fn foreign_verified_notice_reaches_projection_but_cannot_open_the_case() {
    let fixture = case_opening_world(
        "estate-case-opening-foreign-notice",
        EstateCaseStatus::PendingOpening,
        DeathNoticeStatus::Verified,
    );
    let specialist = fixture.authenticate_specialist();
    let (_denial_source_custody, denial) = {
        let source_actor = &specialist;
        let source_scope = &request_scope();
        let source_key = &idempotency(21);
        let source_intent =
            bank_domain::schema::OpenEstateCase::try_from(fixture.action(fixture.foreign_notice))
                .expect("typed estate command preserves the exact variant");
        let source_request = fixture.world.runtime.request(source_actor, source_scope);
        let source_operation = fixture
            .world
            .runtime
            .mutate(bank_server::mutations::open_estate_case(source_intent))
            .as_principal(source_actor)
            .controls(bank_server::BankMutationControls::new(
                (*source_scope).clone(),
                (*source_key).clone(),
            ))
            .retain_operation()
            .unwrap_or_else(|(_, cause)| panic!("exact declared estate construction: {cause:?}"));
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Refused { operation: source_custody, denial: worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::Handler(worth_query_host::facade::primary_graph::MutationHandlerExecutionDenial::Handler(cause))) } = source_operation.advance(&source_request) else { panic!("native pre-effect projection refusal") };
        let cause = cause
            .downcast::<bank_server::BankEstateCaseOpeningProjectionDenial>()
            .expect("actual native projection type");
        (source_custody, cause)
    };

    assert!(matches!(
        denial,
        BankEstateCaseOpeningProjectionDenial::NoticeMismatch
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
        let (_denial_source_custody, denial) = {
            let source_actor = &specialist;
            let source_scope = &request_scope();
            let source_key = &idempotency(31 + ordinal as u8);
            let source_intent =
                bank_domain::schema::OpenEstateCase::try_from(fixture.action(fixture.notice))
                    .expect("typed estate command preserves the exact variant");
            let source_request = fixture.world.runtime.request(source_actor, source_scope);
            let source_operation = fixture
                .world
                .runtime
                .mutate(bank_server::mutations::open_estate_case(source_intent))
                .as_principal(source_actor)
                .controls(bank_server::BankMutationControls::new(
                    (*source_scope).clone(),
                    (*source_key).clone(),
                ))
                .retain_operation()
                .unwrap_or_else(|(_, cause)| {
                    panic!("exact declared estate construction: {cause:?}")
                });
            let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Refused { operation: source_custody, denial: worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::Handler(worth_query_host::facade::primary_graph::MutationHandlerExecutionDenial::Handler(cause))) } = source_operation.advance(&source_request) else { panic!("native pre-effect projection refusal") };
            let cause = cause
                .downcast::<bank_server::BankEstateCaseOpeningProjectionDenial>()
                .expect("actual native projection type");
            (source_custody, cause)
        };
        assert!(matches!(
            denial,
            BankEstateCaseOpeningProjectionDenial::NoticeNotVerified
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
        let (_denial_source_custody, denial) = {
            let source_actor = &specialist;
            let source_scope = &request_scope();
            let source_key = &idempotency(41 + ordinal as u8);
            let source_intent =
                bank_domain::schema::OpenEstateCase::try_from(fixture.action(fixture.notice))
                    .expect("typed estate command preserves the exact variant");
            let source_request = fixture.world.runtime.request(source_actor, source_scope);
            let source_operation = fixture
                .world
                .runtime
                .mutate(bank_server::mutations::open_estate_case(source_intent))
                .as_principal(source_actor)
                .controls(bank_server::BankMutationControls::new(
                    (*source_scope).clone(),
                    (*source_key).clone(),
                ))
                .retain_operation()
                .unwrap_or_else(|(_, cause)| {
                    panic!("exact declared estate construction: {cause:?}")
                });
            let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Refused { operation: source_custody, denial: worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::Handler(worth_query_host::facade::primary_graph::MutationHandlerExecutionDenial::Handler(cause))) } = source_operation.advance(&source_request) else { panic!("native pre-effect projection refusal") };
            let cause = cause
                .downcast::<bank_server::BankEstateCaseOpeningProjectionDenial>()
                .expect("actual native projection type");
            (source_custody, cause)
        };
        assert!(matches!(
            denial,
            BankEstateCaseOpeningProjectionDenial::CaseNotPendingOpening
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
    assert_eq!(admission.canonical_encoded_bytes(), 210);
    assert_eq!(admission.sha256_input_bytes(), 429);
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
