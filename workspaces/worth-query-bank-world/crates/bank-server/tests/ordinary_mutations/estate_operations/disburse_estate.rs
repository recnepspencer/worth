#[path = "disburse_estate/fixture.rs"]
pub(super) mod fixture;
#[path = "disburse_estate/hostility.rs"]
mod hostility;

use bank_domain::{
    estate::{EstateAction, EstateDisbursement},
    model::SignedMoney,
    proposals::BankProposalDenial,
    reads::AccountActivityItem,
    schema::PostingPurpose,
};
use bank_server::{
    queries, BankAuthenticatedPrincipal, BankCommitDenialKind, BankCommitDenialStage,
    BankCommitReceipt, BankEstateProgressionDenial, BankMutationCommitOutcome, BankReadControls,
};
use worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyBinding;

use self::fixture::{disbursement_drift_world, disbursement_world, DisbursementFixture};
use crate::support::request_scope;

#[test]
fn public_progression_disburses_through_a_distinct_authoritative_journal() {
    let fixture = disbursement_world("estate-disbursement-commit", 1_000);
    let specialist = fixture.authenticate_actor();
    let binding = idempotency(11);
    let outcome = disburse(&fixture, &specialist, fixture.action(250), binding)
        .expect("the exact lawful estate disbursement should execute through Query");
    let BankMutationCommitOutcome::Committed(receipt) = outcome else {
        panic!("the first disbursement must commit: {outcome:?}");
    };

    assert_eq!(receipt.emitted_effect_count(), 2);
    assert_eq!(receipt.decision_fact_count(), Some(36));
    assert_single_admission_digest(&receipt);
    assert_authoritative_activity(&fixture);
    assert_equivalent_retry(&fixture, &specialist, binding, &receipt);
}

#[test]
fn idempotency_binds_raw_intent_and_all_nine_disbursement_dimensions() {
    let fixture = disbursement_drift_world("estate-disbursement-drift");
    let specialist = fixture.authenticate_actor();
    let binding = idempotency(31);
    let action = fixture.action(250);
    let committed = disburse(&fixture, &specialist, action, binding)
        .expect("the baseline disbursement should execute");
    assert!(matches!(committed, BankMutationCommitOutcome::Committed(_)));

    assert_drift(
        &fixture,
        &specialist,
        action,
        WorthQueryApplicationIdempotencyBinding::new(*binding.key_identity(), [99; 32]),
    );
    for drifted in payload_drifts(action) {
        assert_drift(&fixture, &specialist, drifted, binding);
    }
}

#[test]
fn malformed_postings_and_insufficient_journal_funds_deny_before_commit() {
    let malformed_fixture = disbursement_world("estate-disbursement-malformed", 1_000);
    let specialist = malformed_fixture.authenticate_actor();
    let EstateAction::DisburseEstate(mut malformed) = malformed_fixture.action(250) else {
        unreachable!("the fixture always constructs a disbursement")
    };
    malformed.postings.swap(0, 1);
    let malformed_denial = disburse(
        &malformed_fixture,
        &specialist,
        EstateAction::DisburseEstate(malformed),
        idempotency(51),
    )
    .expect_err("posting order is part of the governed command shape");
    assert!(matches!(
        malformed_denial,
        BankEstateProgressionDenial::Proposal(BankProposalDenial::DisbursementPostingMismatch)
    ));
    assert_no_disbursement_effects(&malformed_fixture);

    let poor_fixture = disbursement_world("estate-disbursement-insufficient", 100);
    let specialist = poor_fixture.authenticate_actor();
    let denial = disburse(
        &poor_fixture,
        &specialist,
        poor_fixture.action(250),
        idempotency(52),
    )
    .expect_err("a journal-derived balance below the requested amount must deny");
    assert!(matches!(
        denial,
        BankEstateProgressionDenial::Proposal(BankProposalDenial::InsufficientFunds(account))
            if account == poor_fixture.source
    ));
}

fn disburse(
    fixture: &DisbursementFixture,
    specialist: &BankAuthenticatedPrincipal,
    action: EstateAction,
    binding: WorthQueryApplicationIdempotencyBinding,
) -> Result<BankMutationCommitOutcome, BankEstateProgressionDenial> {
    fixture
        .world
        .runtime
        .disburse_estate(specialist, action, binding, &request_scope())
}

fn assert_equivalent_retry(
    fixture: &DisbursementFixture,
    specialist: &BankAuthenticatedPrincipal,
    binding: WorthQueryApplicationIdempotencyBinding,
    committed: &BankCommitReceipt,
) {
    let retry = disburse(fixture, specialist, fixture.action(250), binding)
        .expect("an equivalent authorized retry must resolve before reprojection");
    let BankMutationCommitOutcome::AlreadyCommitted(recovered) = retry else {
        panic!("the retry must recover the authoritative commit: {retry:?}");
    };
    assert_eq!(committed.aftermath(), recovered.aftermath());
}

fn assert_authoritative_activity(fixture: &DisbursementFixture) {
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

fn account_activity(
    fixture: &DisbursementFixture,
    principal: &BankAuthenticatedPrincipal,
    account: bank_domain::model::AccountId,
) -> Vec<AccountActivityItem> {
    fixture
        .world
        .runtime
        .account_activity(account)
        .as_principal(principal)
        .execute(BankReadControls::current(request_scope(), 16, 1_024).unwrap())
        .expect("an exact account owner should read authoritative activity")
        .rows()[0]
        .entries()
        .to_vec()
}

fn disbursement_entry(entries: &[AccountActivityItem]) -> AccountActivityItem {
    *entries
        .iter()
        .find(|entry| entry.purpose() == PostingPurpose::EstateDisbursement)
        .expect("the committed estate-disbursement journal must be query-visible")
}

fn account_revision(
    fixture: &DisbursementFixture,
    principal: &BankAuthenticatedPrincipal,
    account: bank_domain::model::AccountId,
) -> u64 {
    fixture
        .world
        .runtime
        .query(queries::account_summary(account))
        .as_principal(principal)
        .controls(BankReadControls::current(request_scope(), 1, 1_024).unwrap())
        .execute()
        .expect("the exact account owner should read the authoritative revision")
        .rows()[0]
        .accounting_revision()
        .get()
}

fn assert_no_disbursement_effects(fixture: &DisbursementFixture) {
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

fn assert_single_admission_digest(receipt: &BankCommitReceipt) {
    let phases = receipt.canonical_work();
    assert_eq!(phases.admission().basis_preparations(), 1);
    assert_eq!(phases.admission().digest_derivations(), 1);
    assert_eq!(phases.admission().canonical_entries(), 10);
    assert!(phases.admission().canonical_encoded_bytes() <= 4_096);
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

fn assert_drift(
    fixture: &DisbursementFixture,
    specialist: &BankAuthenticatedPrincipal,
    action: EstateAction,
    binding: WorthQueryApplicationIdempotencyBinding,
) {
    let outcome = disburse(fixture, specialist, action, binding)
        .expect("governed-input drift should remain a typed commit outcome");
    assert!(matches!(
        outcome,
        BankMutationCommitOutcome::Denied {
            kind: BankCommitDenialKind::IdempotencyIntentDrift,
            stage: BankCommitDenialStage::Idempotency,
        }
    ));
}

fn payload_drifts(action: EstateAction) -> [EstateAction; 9] {
    let EstateAction::DisburseEstate(input) = action else {
        unreachable!("the fixture always constructs a disbursement")
    };
    let alternate_estate = bank_domain::estate::EstateCaseId::new(input.estate.get() + 15).unwrap();
    [
        changed(input, |value| value.estate = alternate_estate),
        changed(input, |value| {
            value.source_account = input.destination_account
        }),
        changed(input, |value| {
            value.destination_account = input.source_account
        }),
        changed(input, |value| {
            value.beneficiary =
                bank_domain::model::BankPrincipalId::new(input.beneficiary.get() + 1).unwrap()
        }),
        changed(input, |value| {
            value.amount = bank_domain::model::Money::from_minor(251).unwrap()
        }),
        changed(input, |value| {
            value.postings[0].account = input.destination_account
        }),
        changed(input, |value| {
            value.postings[0].amount = SignedMoney::from_minor(-251)
        }),
        changed(input, |value| {
            value.postings[1].account = input.source_account
        }),
        changed(input, |value| {
            value.postings[1].amount = SignedMoney::from_minor(251)
        }),
    ]
}

fn changed(
    mut input: EstateDisbursement,
    change: impl FnOnce(&mut EstateDisbursement),
) -> EstateAction {
    change(&mut input);
    EstateAction::DisburseEstate(input)
}

fn idempotency(identity: u8) -> WorthQueryApplicationIdempotencyBinding {
    WorthQueryApplicationIdempotencyBinding::new([identity; 32], [identity + 1; 32])
}
