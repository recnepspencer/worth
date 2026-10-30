//! Complete Bank commit-publication noninterference across protected-fact twins.

pub use bank_domain::{proposals::BankIdempotencyKey, schema::AccountStatus};
pub use bank_server::{queries, BankReadControls};
pub use worth_query_host::facade::publication::application_aftermath::WorthQueryPublishedAftermathPosture;

pub use super::freeze_account::fixture::{
    freeze_world_with_protected_foreign_status, FreezeFixture,
};
pub use crate::facade::support::request_scope;

pub fn protected_foreign_status(fixture: &FreezeFixture) -> AccountStatus {
    let owner = fixture.authenticate_foreign_owner();
    fixture
        .world
        .runtime
        .query(queries::account_summary(fixture.foreign_account))
        .as_principal(&owner)
        .controls(BankReadControls::current(request_scope(), 16).unwrap())
        .execute()
        .expect("the foreign owner reads the authoritative protected account")
        .rows()[0]
        .status()
}

pub fn commit_freeze(fixture: &FreezeFixture, key: u8) -> bank_server::BankCommitReceipt {
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
