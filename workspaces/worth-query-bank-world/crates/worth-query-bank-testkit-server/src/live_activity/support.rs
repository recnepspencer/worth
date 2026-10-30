pub use bank_domain::model::{CustomerRole, Money};
pub use bank_domain::proposals::BankIdempotencyKey;
pub use bank_domain::schema::{Deposit, GrantAccountAuthorization, RevokeAccountAuthorization};
pub use bank_server::{mutations, queries, BankMutationControls, BankReadControls};
pub use std::num::NonZeroUsize;
pub use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
pub use worth_query_host::facade::primary_graph::WorthQueryApplicationLiveControls;

pub use crate::facade::fixture::{self, VIEWER};
pub use crate::facade::support::request_scope;

pub fn commit_authorization_toggle(
    fixture: &fixture::OrdinaryReadFixture,
    owner: &bank_server::BankAuthenticatedPrincipal,
    ordinal: usize,
) {
    let scope = request_scope();
    let request = fixture.world.runtime.request(owner, &scope);
    if ordinal.is_multiple_of(2) {
        let operation = fixture
            .world
            .runtime
            .mutate(mutations::revoke_account_access(
                RevokeAccountAuthorization {
                    account: fixture.personal_account,
                    authorization: authorized_user_id(fixture, owner, VIEWER),
                },
            ))
            .as_principal(owner)
            .controls(mutation_controls(&format!("overflow-revoke-{ordinal}")))
            .retain_operation()
            .unwrap_or_else(|cause| panic!("declared access: {cause:?}"));
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
            done,
        ) = operation.advance(&request)
        else {
            panic!("fresh access completion")
        };
        assert!(matches!(
            done.evidence(),
            worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh { .. }
        ));
        done.close(&request)
            .unwrap_or_else(|(_, cause)| panic!("access close: {cause:?}"));
    } else {
        let operation = fixture
            .world
            .runtime
            .mutate(mutations::grant_account_access(GrantAccountAuthorization {
                account: fixture.personal_account,
                principal: fixture::principal_id(VIEWER),
                role: CustomerRole::Viewer,
            }))
            .as_principal(owner)
            .controls(mutation_controls(&format!("overflow-grant-{ordinal}")))
            .retain_operation()
            .unwrap_or_else(|cause| panic!("declared access: {cause:?}"));
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
            done,
        ) = operation.advance(&request)
        else {
            panic!("fresh access completion")
        };
        assert!(matches!(
            done.evidence(),
            worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh { .. }
        ));
        done.close(&request)
            .unwrap_or_else(|(_, cause)| panic!("access close: {cause:?}"));
    }
}

pub fn authorized_user_id(
    fixture: &fixture::OrdinaryReadFixture,
    owner: &bank_server::BankAuthenticatedPrincipal,
    principal: usize,
) -> bank_domain::model::AccountAuthorizationId {
    let users = fixture
        .world
        .runtime
        .query(queries::account_authorized_users(fixture.personal_account))
        .as_principal(owner)
        .controls(BankReadControls::current(request_scope(), 16).unwrap())
        .execute()
        .expect("owner should read authorized users");
    let [users] = users.rows() else {
        panic!("authorized users query must return one account row")
    };
    users
        .users()
        .iter()
        .find(|user| user.principal() == fixture::principal_id(principal))
        .expect("fixture authorization should exist")
        .authorization()
}

pub fn activity_count(
    fixture: &fixture::OrdinaryReadFixture,
    owner: &bank_server::BankAuthenticatedPrincipal,
) -> usize {
    fixture
        .world
        .runtime
        .account_activity(fixture.personal_account)
        .as_principal(owner)
        .execute(BankReadControls::current(request_scope(), 64).unwrap())
        .expect("owner should read installed account activity")
        .rows()[0]
        .entries()
        .len()
}

pub fn commit_deposit(
    fixture: &fixture::OrdinaryReadFixture,
    teller: &bank_server::BankAuthenticatedPrincipal,
    account: bank_domain::model::AccountId,
    key: &str,
) -> worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt {
    let outcome = fixture
        .world
        .runtime
        .mutate(mutations::deposit(Deposit {
            institution: fixture.institution,
            account,
            amount: Money::from_minor(1).unwrap(),
        }))
        .as_principal(teller)
        .controls(BankMutationControls::new(
            request_scope(),
            BankIdempotencyKey::new(key).unwrap(),
        ))
        .retain_operation()
        .unwrap_or_else(|cause| panic!("deposit construction: {cause:?}"));
    let scope = request_scope();
    let request = fixture.world.runtime.request(teller, &scope);
    let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
        completed,
    ) = outcome.advance(&request, Some(NonZeroUsize::new(20_000).unwrap()))
    else {
        panic!("native retained deposit completion")
    };
    let worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh {
        receipt,
        ..
    } = completed.evidence()
    else {
        panic!("fresh deposit evidence")
    };
    let receipt = receipt.clone();
    assert!(completed.close(&request).is_ok());

    receipt
}

pub fn live_controls() -> WorthQueryApplicationLiveControls {
    live_controls_for(request_scope(), 16)
}

pub fn live_controls_for(
    request: WorthQueryRequestScope,
    buffer_capacity: usize,
) -> WorthQueryApplicationLiveControls {
    WorthQueryApplicationLiveControls::bounded(request, buffer_capacity, 8).unwrap()
}

pub fn mutation_controls(key: &str) -> BankMutationControls {
    BankMutationControls::new(
        request_scope(),
        BankIdempotencyKey::new(key).expect("test idempotency key should admit"),
    )
}
