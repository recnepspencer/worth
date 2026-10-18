#[path = "freeze_account/fixture.rs"]
pub(crate) mod fixture;

use bank_domain::{
    estate::EstateWorkflowStage, proposals::BankIdempotencyKey, schema::AccountStatus,
};
use bank_server::{queries, BankEstateFreezeProjectionDenial, BankReadControls};

use self::fixture::{exact_freeze_world, foreign_account_freeze_world, FreezeFixture};
use crate::support::request_scope;

#[test]
fn public_query_progression_freezes_the_exact_estate_account() {
    let fixture = exact_freeze_world("estate-freeze-commit", AccountStatus::Open);
    let specialist = fixture.authenticate_specialist();
    let key = BankIdempotencyKey::new("freeze-estate-account-program-entry").unwrap();
    let outcome = {
        let source_actor = &specialist;
        let source_scope = &request_scope();
        let source_key = &key;
        let source_intent = bank_domain::schema::FreezeEstateAccount::try_from(
            fixture.action(fixture.estate_account),
        )
        .expect("typed estate command preserves the exact variant");
        let source_request = fixture.world.runtime.request(source_actor, source_scope);
        let source_operation = fixture
            .world
            .runtime
            .mutate(bank_server::mutations::freeze_estate_account(source_intent))
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
    assert_eq!(receipt.decision_fact_count(), Some(5));
    assert_freeze_canonical_work(receipt.canonical_work());
    assert_eq!(estate_account_status(&fixture), AccountStatus::Frozen);

    let retry = {
        let source_actor = &specialist;
        let source_scope = &request_scope();
        let source_key = &key;
        let source_intent = bank_domain::schema::FreezeEstateAccount::try_from(
            fixture.action(fixture.estate_account),
        )
        .expect("typed estate command preserves the exact variant");
        let source_request = fixture.world.runtime.request(source_actor, source_scope);
        let source_operation = fixture
            .world
            .runtime
            .mutate(bank_server::mutations::freeze_estate_account(source_intent))
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
            worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Recovered { .. }
        ));
        let closed_source = source_completed
            .close_with_evidence(&source_request)
            .unwrap_or_else(|(_, cause)| panic!("lawful native source close: {cause:?}"));
        bank_server::BankCommitReceipt::from_closed_source(closed_source)
    };
    let recovered = retry;
    assert_eq!(receipt.aftermath(), recovered.aftermath());
    assert!(recovered.retained_preimage());
    assert!(
        recovered.performed_preimage_retention_work(),
        "the installed recorded inverse must pay its explicit retention work"
    );
    assert_freeze_canonical_work(recovered.canonical_work());
    assert_eq!(estate_account_status(&fixture), AccountStatus::Frozen);

    let _drift = {
        let source_actor = &specialist;
        let source_scope = &request_scope();
        let source_key = &key;
        let source_intent = bank_domain::schema::FreezeEstateAccount::try_from(
            fixture.action(fixture.foreign_account),
        )
        .expect("typed estate command preserves the exact variant");
        let source_request = fixture.world.runtime.request(source_actor, source_scope);
        let source_operation = fixture
            .world
            .runtime
            .mutate(bank_server::mutations::freeze_estate_account(source_intent))
            .as_principal(source_actor)
            .controls(bank_server::BankMutationControls::new(
                (*source_scope).clone(),
                (*source_key).clone(),
            ))
            .retain_operation()
            .unwrap_or_else(|cause| panic!("exact declared estate construction: {cause:?}"));
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

    let committed = {
        let source_actor = &specialist;
        let source_scope = &request_scope();
        let source_key = &idempotency(230);
        let source_intent = bank_domain::schema::FreezeEstateAccount::try_from(
            fixture.action(fixture.estate_account),
        )
        .expect("typed estate command preserves the exact variant");
        let source_request = fixture.world.runtime.request(source_actor, source_scope);
        let source_operation = fixture
            .world
            .runtime
            .mutate(bank_server::mutations::freeze_estate_account(source_intent))
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
    assert!(committed.changed_record_count() > 0);
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
    let (_denial_source_custody, denial) = {
        let source_actor = &specialist;
        let source_scope = &request_scope();
        let source_key = &idempotency(21);
        let source_intent = bank_domain::schema::FreezeEstateAccount::try_from(
            fixture.action(fixture.foreign_account),
        )
        .expect("typed estate command preserves the exact variant");
        let source_request = fixture.world.runtime.request(source_actor, source_scope);
        let source_operation = fixture
            .world
            .runtime
            .mutate(bank_server::mutations::freeze_estate_account(source_intent))
            .as_principal(source_actor)
            .controls(bank_server::BankMutationControls::new(
                (*source_scope).clone(),
                (*source_key).clone(),
            ))
            .retain_operation()
            .unwrap_or_else(|cause| panic!("exact declared estate construction: {cause:?}"));
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Refused { operation: source_custody, denial: worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::Handler(worth_query_host::facade::primary_graph::MutationHandlerExecutionDenial::Handler(cause))) } = source_operation.advance(&source_request) else { panic!("native pre-effect projection refusal") };
        let cause = cause
            .downcast::<bank_server::BankEstateFreezeProjectionDenial>()
            .expect("actual native projection type");
        (source_custody, cause)
    };

    assert!(matches!(
        denial,
        BankEstateFreezeProjectionDenial::RelatedAccountMismatch
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
        let (_denial_source_custody, denial) = {
            let source_actor = &specialist;
            let source_scope = &request_scope();
            let source_key = &idempotency(31 + ordinal as u8);
            let source_intent = bank_domain::schema::FreezeEstateAccount::try_from(
                fixture.action(fixture.estate_account),
            )
            .expect("typed estate command preserves the exact variant");
            let source_request = fixture.world.runtime.request(source_actor, source_scope);
            let source_operation = fixture
                .world
                .runtime
                .mutate(bank_server::mutations::freeze_estate_account(source_intent))
                .as_principal(source_actor)
                .controls(bank_server::BankMutationControls::new(
                    (*source_scope).clone(),
                    (*source_key).clone(),
                ))
                .retain_operation()
                .unwrap_or_else(|cause| panic!("exact declared estate construction: {cause:?}"));
            let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Refused { operation: source_custody, denial: worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::Handler(worth_query_host::facade::primary_graph::MutationHandlerExecutionDenial::Handler(cause))) } = source_operation.advance(&source_request) else { panic!("native pre-effect projection refusal") };
            let cause = cause
                .downcast::<bank_server::BankEstateFreezeProjectionDenial>()
                .expect("actual native projection type");
            (source_custody, cause)
        };

        assert!(matches!(
            denial,
            BankEstateFreezeProjectionDenial::AccountNotOpen
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
    // Binding-selected capability admission governs its input as well as the mutation
    // input/key identities. All three native derivations are included here; later
    // publication and domain recovery do not rebuild these identities.
    let admission = phases.admission();
    assert_eq!(admission.basis_preparations(), 0);
    assert_eq!(admission.digest_derivations(), 3);
    assert_eq!(admission.canonical_entries(), 3);
    assert_eq!(admission.canonical_encoded_bytes(), 269);
    assert_eq!(admission.sha256_input_bytes(), 488);
    assert_eq!(admission.sha256_compression_blocks(), 9);
    assert_eq!(admission.digest_text_materializations(), 0);
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
