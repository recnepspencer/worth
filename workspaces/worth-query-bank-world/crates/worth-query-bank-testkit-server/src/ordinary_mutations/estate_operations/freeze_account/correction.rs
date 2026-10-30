pub use super::fixture::{revocable_freeze_world, FreezeFixture};
pub use super::{estate_account_status, foreign_account_status};
pub use crate::facade::support::request_scope;
pub use bank_domain::{
    proposals::BankIdempotencyKey,
    schema::{AccountStatus, FreezeEstateAccount},
};
pub use bank_server::{mutations, BankMutationControls};
pub use worth_query_host::facade::{
    application_entry::*,
    primary_graph::{
        WorthQueryCorrectionDerivationDenial, WorthQueryOperationAuthorizationDenialKind,
        WorthQueryRecordedCorrectionSource,
    },
};

pub fn limits() -> WorthQueryCorrectionHistoryLimits {
    WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN)
}

pub fn freeze_and_close(fixture: &FreezeFixture) -> WorthQueryRecordedCorrectionSource {
    let specialist = fixture.authenticate_specialist();
    let scope = request_scope();
    let request = fixture.world.runtime.request(&specialist, &scope);
    let operation = fixture
        .world
        .runtime
        .mutate(mutations::freeze_estate_account(FreezeEstateAccount::new(
            fixture.estate,
            fixture.estate_account,
        )))
        .as_principal(&specialist)
        .controls(BankMutationControls::new(
            scope.clone(),
            BankIdempotencyKey::new("correction-original-freeze").unwrap(),
        ))
        .retain_operation()
        .unwrap_or_else(|cause| panic!("freeze construction: {cause:?}"));
    let WorthQuerySourceOnlyAdvance::Completed(completed) = operation.advance(&request) else {
        panic!("original freeze completes through its capability owner");
    };
    let WorthQuerySourceCompletion::Fresh { receipt, .. } = completed.evidence() else {
        panic!("original freeze is fresh");
    };
    let source = receipt.correction_source();
    assert!(completed.close(&request).is_ok());
    assert_eq!(estate_account_status(fixture), AccountStatus::Frozen);
    source
}
