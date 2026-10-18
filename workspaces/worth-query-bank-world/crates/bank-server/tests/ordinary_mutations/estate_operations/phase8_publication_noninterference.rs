//! Complete Bank commit-publication noninterference across protected-fact twins.

use bank_domain::{proposals::BankIdempotencyKey, schema::AccountStatus};
use bank_server::{queries, BankReadControls};
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
    {
        let source_actor = &specialist;
        let source_scope = &request_scope();
        let source_key = &BankIdempotencyKey::new(format!("publication-freeze-{key}")).unwrap();
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
    }
}
