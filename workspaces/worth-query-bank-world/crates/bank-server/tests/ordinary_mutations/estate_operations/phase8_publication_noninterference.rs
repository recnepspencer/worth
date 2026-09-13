//! Complete Bank commit-publication noninterference across protected-fact twins.

use bank_domain::schema::AccountStatus;
use bank_server::{queries, BankMutationCommitOutcome, BankReadControls};
use worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyBinding;
use worth_query_host::facade::publication::application_aftermath::WorthQueryPublishedAftermathPosture;

use super::freeze_account::fixture::{freeze_world_with_protected_foreign_status, FreezeFixture};
use crate::support::request_scope;

#[test]
fn paired_protected_account_detail_worlds_publish_equal_complete_commit_surfaces() {
    let left = freeze_world_with_protected_foreign_status(
        "r848-protected-twin",
        AccountStatus::Open,
        AccountStatus::Open,
    );
    let right = freeze_world_with_protected_foreign_status(
        "r848-protected-twin",
        AccountStatus::Open,
        AccountStatus::Closed,
    );
    assert_eq!(protected_foreign_status(&left), AccountStatus::Open);
    assert_eq!(protected_foreign_status(&right), AccountStatus::Closed);

    let left = commit_freeze(&left, 31);
    let right = commit_freeze(&right, 31);

    // Each receipt carries its real World publication identity. Compare the
    // Bank-visible semantics rather than pretending two Worlds share authority.
    assert_eq!(
        left.publication().inspect().kind(),
        right.publication().inspect().kind()
    );
    assert_eq!(left.changed_record_count(), right.changed_record_count());
    assert_eq!(left.emitted_effect_count(), right.emitted_effect_count());
    assert_eq!(
        left.expected_version_count(),
        right.expected_version_count()
    );
    assert_eq!(left.expected_fact_count(), right.expected_fact_count());
    assert_eq!(left.decision_fact_count(), right.decision_fact_count());
    assert_eq!(left.canonical_work(), right.canonical_work());
    assert_eq!(left.aftermath(), right.aftermath());
    assert_eq!(
        left.external_dispatch_posture(),
        right.external_dispatch_posture()
    );
    assert_eq!(
        left.co_committed_dispatch_outbox(),
        right.co_committed_dispatch_outbox()
    );
    assert_eq!(left.retained_preimage(), right.retained_preimage());
    assert_eq!(
        left.performed_preimage_retention_work(),
        right.performed_preimage_retention_work()
    );
    assert_eq!(
        left.aftermath().posture(),
        Some(WorthQueryPublishedAftermathPosture::Reversible)
    );
}

fn protected_foreign_status(fixture: &FreezeFixture) -> AccountStatus {
    let owner = fixture.authenticate_foreign_owner();
    fixture
        .world
        .runtime
        .query(queries::account_summary(fixture.foreign_account))
        .as_principal(&owner)
        .controls(BankReadControls::current(request_scope(), 16, 20_000).unwrap())
        .execute()
        .expect("the foreign owner reads the authoritative protected account")
        .rows()[0]
        .status()
}

fn commit_freeze(fixture: &FreezeFixture, key: u8) -> bank_server::BankCommitReceipt {
    let specialist = fixture.authenticate_specialist();
    let outcome = fixture
        .world
        .runtime
        .freeze_estate_account(
            &specialist,
            fixture.action(fixture.estate_account),
            WorthQueryApplicationIdempotencyBinding::new([key; 32], [key.wrapping_add(1); 32]),
            &request_scope(),
        )
        .expect("freeze must admit under both protected-fact twins");
    let BankMutationCommitOutcome::Committed(receipt) = outcome else {
        panic!("freeze must commit: {outcome:?}");
    };

    receipt
}
