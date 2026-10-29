#[path = "disburse_estate/fixture.rs"]
pub mod fixture;
#[path = "disburse_estate/hostility.rs"]
pub mod hostility;
#[path = "disburse_estate/recovery.rs"]
pub mod recovery;

pub use bank_domain::{
    estate::EstateAction,
    proposals::{BankIdempotencyKey, BankProposalDenial},
    reads::AccountActivityItem,
    schema::PostingPurpose,
};
pub use bank_server::{queries, BankAuthenticatedPrincipal, BankCommitReceipt, BankReadControls};

pub use self::fixture::{disbursement_drift_world, disbursement_world, DisbursementFixture};
pub use crate::facade::server::support::request_scope;

pub fn disburse(
    fixture: &DisbursementFixture,
    specialist: &BankAuthenticatedPrincipal,
    action: EstateAction,
    key: BankIdempotencyKey,
) -> super::EstateSourceAdvance<
    bank_domain::schema::DisburseEstate,
    bank_server::EstateDisbursementAction,
> {
    {
        let source_actor = specialist;
        let source_scope = &request_scope();
        let source_key = &key;
        let source_intent = bank_domain::schema::DisburseEstate::try_from(action)
            .expect("typed estate command preserves the exact variant");
        let source_request = fixture.world.runtime.request(source_actor, source_scope);
        let source_operation = fixture
            .world
            .runtime
            .mutate(bank_server::mutations::disburse_estate(source_intent))
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

pub fn assert_equivalent_retry(
    fixture: &DisbursementFixture,
    specialist: &BankAuthenticatedPrincipal,
    key: &BankIdempotencyKey,
    committed: &BankCommitReceipt,
) {
    let retry = {
        let closing_actor = specialist;
        let closing_scope = crate::facade::server::support::request_scope();
        let closing_request = (fixture)
            .world
            .runtime
            .request(closing_actor, &closing_scope);
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
            done,
        ) = disburse(fixture, specialist, fixture.action(250), key.clone())
        else {
            panic!("native source completion")
        };
        assert!(matches!(
            done.evidence(),
            worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Recovered { .. }
        ));
        bank_server::BankCommitReceipt::from_closed_source(
            done.close_with_evidence(&closing_request)
                .unwrap_or_else(|(_, cause)| panic!("source close: {cause:?}")),
        )
    };
    let recovered = retry;
    assert_eq!(committed.aftermath(), recovered.aftermath());
    assert_admission_derives_the_request_identities_once(&recovered);
}

pub fn assert_authoritative_activity(fixture: &DisbursementFixture) {
    let source_owner = fixture.authenticate_source_owner();
    let beneficiary = fixture.authenticate_beneficiary();
    let source = account_activity(fixture, &source_owner, fixture.source);
    let destination = account_activity(fixture, &beneficiary, fixture.destination);
    let debit = disbursement_entry(&source);
    let credit = disbursement_entry(&destination);

    assert_eq!(debit.account(), fixture.source);
    assert_eq!(debit.amount().minor_units(), -250);
    assert_eq!(debit.account_sequence().get(), 2);
    assert_eq!(credit.account(), fixture.destination);
    assert_eq!(credit.amount().minor_units(), 250);
    assert_eq!(credit.account_sequence().get(), 1);
    assert_eq!(debit.journal(), credit.journal());
    assert_eq!(debit.reversal_of(), None);
    assert_eq!(credit.reversal_of(), None);
    assert_eq!(account_revision(fixture, &source_owner, fixture.source), 2);
    assert_eq!(
        account_revision(fixture, &beneficiary, fixture.destination),
        1
    );
}

pub fn account_activity(
    fixture: &DisbursementFixture,
    principal: &BankAuthenticatedPrincipal,
    account: bank_domain::model::AccountId,
) -> Vec<AccountActivityItem> {
    fixture
        .world
        .runtime
        .account_activity(account)
        .as_principal(principal)
        .execute(BankReadControls::current(request_scope(), 16).unwrap())
        .expect("an exact account owner should read authoritative activity")
        .rows()[0]
        .entries()
        .to_vec()
}

pub fn disbursement_entry(entries: &[AccountActivityItem]) -> AccountActivityItem {
    *entries
        .iter()
        .find(|entry| entry.purpose() == PostingPurpose::EstateDisbursement)
        .expect("the committed estate-disbursement journal must be query-visible")
}

pub fn account_revision(
    fixture: &DisbursementFixture,
    principal: &BankAuthenticatedPrincipal,
    account: bank_domain::model::AccountId,
) -> u64 {
    fixture
        .world
        .runtime
        .query(queries::account_summary(account))
        .as_principal(principal)
        .controls(BankReadControls::current(request_scope(), 1).unwrap())
        .execute()
        .expect("the exact account owner should read the authoritative revision")
        .rows()[0]
        .accounting_revision()
        .get()
}

pub fn assert_no_disbursement_effects(fixture: &DisbursementFixture) {
    let source_owner = fixture.authenticate_source_owner();
    let beneficiary = fixture.authenticate_beneficiary();
    assert!(account_activity(fixture, &source_owner, fixture.source)
        .iter()
        .all(|entry| entry.purpose() != PostingPurpose::EstateDisbursement));
    assert!(account_activity(fixture, &beneficiary, fixture.destination)
        .iter()
        .all(|entry| entry.purpose() != PostingPurpose::EstateDisbursement));
    assert_eq!(account_revision(fixture, &source_owner, fixture.source), 1);
    assert_eq!(
        account_revision(fixture, &beneficiary, fixture.destination),
        0
    );
}

pub fn assert_admission_derives_the_request_identities_once(receipt: &BankCommitReceipt) {
    let phases = receipt.canonical_work();
    // Binding-selected capability admission governs its input as well as the mutation
    // input/key identities. All three native derivations are included here; later
    // publication and domain recovery do not rebuild these identities.
    let admission = phases.admission();
    assert_eq!(admission.basis_preparations(), 0);
    assert_eq!(admission.digest_derivations(), 3);
    assert_eq!(admission.canonical_entries(), 3);
    assert_eq!(admission.canonical_encoded_bytes(), 1170);
    assert_eq!(admission.sha256_input_bytes(), 1389);
    assert_eq!(admission.sha256_compression_blocks(), 23);
    assert_eq!(admission.digest_text_materializations(), 0);
    for phase in [
        phases.installation(),
        phases.execution(),
        phases.provider_commit(),
        phases.projection(),
        phases.live_delivery(),
        phases.retry_resolution(),
        phases.recovery_inspection(),
        phases.publication(),
    ] {
        assert_eq!(phase.basis_preparations(), 0);
        assert_eq!(phase.digest_derivations(), 0);
        assert_eq!(phase.digest_text_materializations(), 0);
    }
}

pub fn idempotency(identity: u8) -> BankIdempotencyKey {
    BankIdempotencyKey::new(format!("estate-disbursement-{identity}")).unwrap()
}
