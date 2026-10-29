pub(crate) use worth_query_bank_testkit_server::facade::estate_operations::open_estate_case::*;

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
            .unwrap_or_else(|cause| panic!("exact declared estate construction: {cause:?}"));
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
                .unwrap_or_else(|cause| panic!("exact declared estate construction: {cause:?}"));
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
                .unwrap_or_else(|cause| panic!("exact declared estate construction: {cause:?}"));
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
