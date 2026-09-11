use bank_domain::{
    estate::{EstateAction, EstateDisbursement, EstatePosting},
    model::{Money, SignedMoney},
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationIdempotencyBinding,
};

use crate::estate_capability_admission::fixture::{
    disbursement_world, request_scope, ACCOUNT, APPROVER, DECEASED, ESTATE, NOTICE, OTHER_ACCOUNT,
};

#[test]
fn journal_and_revision_drift_after_materialization_stales_provider_commit() {
    let fixture = disbursement_world("estate-disbursement-provider-currentness");
    let specialist = fixture.authenticate();
    let action = action(250);
    let admission = fixture
        .runtime
        .admit_estate_disbursement(&specialist, action, &request_scope())
        .expect("the exact disbursement should admit");
    let program = fixture
        .runtime
        .materialize_estate_disbursement(admission, idempotency(211))
        .expect("the exact disbursement should materialize while balances are current");

    let competing = fixture
        .runtime
        .disburse_estate(&specialist, action, idempotency(213), &request_scope())
        .expect("a competing lawful disbursement should change both account revisions");
    assert!(matches!(
        competing,
        crate::BankMutationCommitOutcome::Committed(_)
    ));

    let outcome = fixture
        .runtime
        .application_runtime()
        .compare_and_commit_application(program, idempotency(211));
    assert_product_basis_stale(outcome);
}

#[test]
fn unrelated_notice_invalidates_the_old_product_basis_and_fresh_disbursement_commits() {
    let fixture = disbursement_world("estate-disbursement-unrelated-currentness");
    let specialist = fixture.authenticate();
    let action = action(250);
    let admission = fixture
        .runtime
        .admit_estate_disbursement(&specialist, action, &request_scope())
        .expect("the exact disbursement should admit");
    let program = fixture
        .runtime
        .materialize_estate_disbursement(admission, idempotency(221))
        .expect("the exact disbursement should materialize");

    let unrelated = fixture
        .runtime
        .notify_estate_death(
            &specialist,
            EstateAction::NotifyDeath {
                estate: ESTATE,
                notice: NOTICE,
                subject: DECEASED,
            },
            idempotency(223),
            &request_scope(),
        )
        .expect("the unrelated notice transition should commit");
    assert!(matches!(
        unrelated,
        crate::BankMutationCommitOutcome::Committed(_)
    ));

    let outcome = fixture
        .runtime
        .application_runtime()
        .compare_and_commit_application(program, idempotency(221));
    assert_product_basis_stale(outcome);

    let fresh = fixture
        .runtime
        .disburse_estate(&specialist, action, idempotency(221), &request_scope())
        .expect("fresh admission must preserve progress unrelated to the disbursement");
    assert!(matches!(
        fresh,
        crate::BankMutationCommitOutcome::Committed(_)
    ));
}

fn assert_product_basis_stale(outcome: WorthQueryApplicationCommitOutcome) {
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("an old materialized program must retain its exact product basis: {outcome:?}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ProductBasisStale
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
}

/// Q8.26-C6: a `Compensation` operation retains no pre-image.
///
/// `DisburseEstate` declares `Compensation`, not `RecordedInverse` — it is
/// corrected by a compensating transfer, not by restoring a prior value. The
/// demand derivation returns `None` for that mechanism, so a retained slice here
/// would be a pre-image no correction lane consumes, and a receipt that appears
/// to promise an inverse the contract never declared.
#[test]
fn compensation_operations_retain_no_preimage() {
    let fixture = disbursement_world("estate-disbursement-retains-no-preimage");
    let specialist = fixture.authenticate();
    let admission = fixture
        .runtime
        .admit_estate_disbursement(&specialist, action(250), &request_scope())
        .expect("the exact disbursement should admit");
    let program = fixture
        .runtime
        .materialize_estate_disbursement(admission, idempotency(231))
        .expect("the exact disbursement should materialize");
    let outcome = fixture
        .runtime
        .application_runtime()
        .compare_and_commit_application(program, idempotency(231));
    let WorthQueryApplicationCommitOutcome::Committed(receipt) = outcome else {
        panic!("the disbursement must commit: {outcome:?}");
    };
    assert!(
        receipt.retained_preimage().is_none(),
        "DisburseEstate declares Compensation, so its commit must retain no \
         inverse pre-image"
    );
}

fn action(amount: i64) -> EstateAction {
    EstateAction::DisburseEstate(EstateDisbursement {
        estate: ESTATE,
        source_account: ACCOUNT,
        destination_account: OTHER_ACCOUNT,
        beneficiary: APPROVER,
        amount: Money::from_minor(amount).unwrap(),
        postings: [
            EstatePosting {
                account: ACCOUNT,
                amount: SignedMoney::from_minor(-amount),
            },
            EstatePosting {
                account: OTHER_ACCOUNT,
                amount: SignedMoney::from_minor(amount),
            },
        ],
    })
}

fn idempotency(seed: u8) -> WorthQueryApplicationIdempotencyBinding {
    WorthQueryApplicationIdempotencyBinding::new([seed; 32], [seed + 1; 32])
}
