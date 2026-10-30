pub(crate) use worth_query_bank_testkit_server::facade::estate_operations::disburse_estate::*;

pub(crate) mod hostility;

pub(crate) mod recovery;

#[test]
fn public_progression_disburses_through_a_distinct_authoritative_journal() {
    let fixture = disbursement_world("estate-disbursement-commit", 1_000);
    let specialist = fixture.authenticate_actor();
    let key = idempotency(11);
    let outcome = {
        let closing_actor = &specialist;
        let closing_scope = worth_query_bank_testkit_server::facade::support::request_scope();
        let closing_request = fixture.world.runtime.request(closing_actor, &closing_scope);
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
            done,
        ) = disburse(&fixture, &specialist, fixture.action(250), key.clone())
        else {
            panic!("native source completion")
        };
        assert!(matches!(
            done.evidence(),
            worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh { .. }
        ));
        bank_server::BankCommitReceipt::from_closed_source(
            done.close_with_evidence(&closing_request)
                .unwrap_or_else(|(_, cause)| panic!("source close: {cause:?}")),
        )
    };
    let receipt = outcome;

    assert_eq!(receipt.emitted_effect_count(), 2);
    assert_eq!(receipt.decision_fact_count(), Some(36));
    assert!(
        !receipt.retained_preimage(),
        "compensation operations must not promise an inverse pre-image"
    );
    assert_admission_derives_the_request_identities_once(&receipt);
    assert_authoritative_activity(&fixture);
    assert_equivalent_retry(&fixture, &specialist, &key, &receipt);
}

#[test]
fn idempotency_rejects_a_different_authorized_disbursement_input() {
    let fixture = disbursement_drift_world("estate-disbursement-drift");
    let specialist = fixture.authenticate_actor();
    let key = idempotency(31);
    let action = fixture.action(250);
    let committed = {
        let closing_actor = &specialist;
        let closing_scope = worth_query_bank_testkit_server::facade::support::request_scope();
        let closing_request = fixture.world.runtime.request(closing_actor, &closing_scope);
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
            done,
        ) = disburse(&fixture, &specialist, action, key.clone())
        else {
            panic!("native source completion")
        };
        assert!(matches!(
            done.evidence(),
            worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh { .. }
        ));
        bank_server::BankCommitReceipt::from_closed_source(
            done.close_with_evidence(&closing_request)
                .unwrap_or_else(|(_, cause)| panic!("source close: {cause:?}")),
        )
    };
    assert!(committed.changed_record_count() > 0);

    let _drift = {
        let closing_actor = &specialist;
        let closing_scope = worth_query_bank_testkit_server::facade::support::request_scope();
        let closing_request = fixture.world.runtime.request(closing_actor, &closing_scope);
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Terminal(
            done,
        ) = disburse(&fixture, &specialist, fixture.action(251), key)
        else {
            panic!("native keyed drift")
        };
        assert!(matches!(done.native_outcome(), Some(worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift)));
        done.close(&closing_request)
            .unwrap_or_else(|(_, cause)| panic!("drift close: {cause:?}"))
    };
}

#[test]
fn malformed_postings_and_insufficient_journal_funds_deny_before_commit() {
    let malformed_fixture = disbursement_world("estate-disbursement-malformed", 1_000);
    let specialist = malformed_fixture.authenticate_actor();
    let EstateAction::DisburseEstate(mut malformed) = malformed_fixture.action(250) else {
        unreachable!("the fixture always constructs a disbursement")
    };
    malformed.postings.swap(0, 1);
    let malformed_denial = {
        let closing_actor = &specialist;
        let closing_scope = worth_query_bank_testkit_server::facade::support::request_scope();
        let closing_request = malformed_fixture
            .world
            .runtime
            .request(closing_actor, &closing_scope);
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Terminal(
            done,
        ) = disburse(
            &malformed_fixture,
            &specialist,
            EstateAction::DisburseEstate(malformed),
            idempotency(51),
        )
        else {
            panic!("native domain no-effect denial")
        };
        let Some(worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::DomainDenied(bank_domain::schema::EstateMutationDenial::Proposal(cause))) = done.native_outcome() else { panic!("original proposal denial") };
        let cause = cause.clone();
        done.close(&closing_request)
            .unwrap_or_else(|(_, denial)| panic!("terminal close: {denial:?}"));
        cause
    };
    assert!(matches!(
        malformed_denial,
        BankProposalDenial::DisbursementPostingMismatch
    ));
    assert_no_disbursement_effects(&malformed_fixture);

    let poor_fixture = disbursement_world("estate-disbursement-insufficient", 100);
    let specialist = poor_fixture.authenticate_actor();
    let denial = {
        let closing_actor = &specialist;
        let closing_scope = worth_query_bank_testkit_server::facade::support::request_scope();
        let closing_request = poor_fixture
            .world
            .runtime
            .request(closing_actor, &closing_scope);
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Terminal(
            done,
        ) = disburse(
            &poor_fixture,
            &specialist,
            poor_fixture.action(250),
            idempotency(52),
        )
        else {
            panic!("native domain no-effect denial")
        };
        let Some(worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::DomainDenied(bank_domain::schema::EstateMutationDenial::Proposal(cause))) = done.native_outcome() else { panic!("original proposal denial") };
        let cause = cause.clone();
        done.close(&closing_request)
            .unwrap_or_else(|(_, denial)| panic!("terminal close: {denial:?}"));
        cause
    };
    assert!(matches!(
        denial,
        BankProposalDenial::InsufficientFunds(account)
            if account == poor_fixture.source
    ));
}
