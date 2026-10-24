use super::{fixture, key, ReadyPaymentWorld};
use worth_query_host::facade::application_entry::{
    WorthQuerySourceCompletion, WorthQuerySourceOnlyAdvance,
};

pub(super) fn grant_teller_payment_disclosure(fixture: &fixture::OrdinaryReadFixture) {
    let runtime = &fixture.world.runtime;
    let owner = fixture.authenticate(fixture::OWNER);
    let scope = crate::support::request_scope();
    let request = runtime.request(&owner, &scope);
    let operation = runtime
        .mutate(bank_server::mutations::grant_account_access(
            bank_domain::schema::GrantAccountAuthorization {
                account: fixture.business_account,
                principal: fixture::principal_id(fixture::TELLER),
                role: bank_domain::model::CustomerRole::Viewer,
            },
        ))
        .as_principal(&owner)
        .controls(bank_server::BankMutationControls::new(
            scope.clone(),
            key("approved-payment:correction:grant-view"),
        ))
        .retain_operation()
        .unwrap_or_else(|cause| panic!("ordinary disclosure grant: {cause:?}"));
    let WorthQuerySourceOnlyAdvance::Completed(completed) = operation.advance(&request) else {
        panic!("ordinary disclosure grant completes");
    };
    assert!(matches!(
        completed.evidence(),
        WorthQuerySourceCompletion::Fresh { .. }
    ));
    completed
        .close(&request)
        .unwrap_or_else(|_| panic!("ordinary grant closes"));
}

pub(super) fn balances(ready: &ReadyPaymentWorld) -> (i64, i64) {
    let owner = ready.fixture.authenticate(fixture::OWNER);
    let recipient = ready.fixture.authenticate(fixture::RECIPIENT);
    let balance = |account, principal| {
        ready
            .fixture
            .world
            .runtime
            .request(principal, &ready.scope)
            .query(bank_domain::queries::account_summary(account))
            .execute()
            .unwrap()
            .rows()[0]
            .current_balance()
            .minor_units()
    };
    (
        balance(ready.fixture.business_account, &owner),
        balance(ready.fixture.recipient_account, &recipient),
    )
}
