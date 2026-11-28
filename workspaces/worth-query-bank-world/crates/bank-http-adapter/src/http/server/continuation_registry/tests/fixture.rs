//! Publish one ordinary transfer for the native activity continuation.
use super::*;
use crate::http::server::tests::fixture::{application, CausalHttpApplication};
use bank_domain::{
    model::{AccountId, BankPrincipalId, Money},
    proposals::BankIdempotencyKey,
    schema::SendMoney,
};
use bank_server::{BankAuthenticatedPrincipal, BankMutationControls};
use worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance;

pub(super) struct Fixture {
    pub application: CausalHttpApplication,
    pub principal: BankAuthenticatedPrincipal,
    }
pub(super) fn scope(
) -> worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope {
    use worth_query_host::facade::admission::authenticated_principal::*;
    WorthQueryRequestScope::new(
        Some(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system().now() + Duration::from_secs(30)),
        WorthQueryCancellationSource::new().token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
}
pub(super) async fn fixture() -> Fixture {
    let application = application(AccountId::new(100).unwrap());
    let request_scope = scope();
    let principal = application.authenticate(&request_scope).await;
    {
        let request = application.runtime.request(&principal, &request_scope);
        let operation = application
            .runtime
            .mutate(bank_server::mutations::send_money(SendMoney {
                from: AccountId::new(100).unwrap(),
                recipient: BankPrincipalId::new(2).unwrap(),
                amount: Money::from_minor(10).unwrap(),
            }))
            .as_principal(&principal)
            .controls(BankMutationControls::new(
                request_scope.clone(),
                BankIdempotencyKey::new("registry-source").unwrap(),
            ))
            .retain_operation()
            .unwrap_or_else(|_| panic!("source owner"));
        let WorthQuerySourceOnlyAdvance::Completed(completed) = operation.advance(&request, None)
        else {
            panic!("source completion");
        };
        assert!(completed.close(&request).is_ok());
    }
    Fixture { application, principal }
}
