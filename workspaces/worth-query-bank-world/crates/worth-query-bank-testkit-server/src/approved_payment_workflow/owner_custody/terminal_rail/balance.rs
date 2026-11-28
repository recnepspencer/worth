use super::{fixture, ReadyPaymentWorld};

pub fn balances(ready: &ReadyPaymentWorld) -> (i64, i64) {
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
