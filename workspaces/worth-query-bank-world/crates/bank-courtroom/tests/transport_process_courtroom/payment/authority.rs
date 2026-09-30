//! The account owner's independent authority changes around callback delivery.

use bank_domain::model::{AccountId, BankPrincipalId, CustomerRole};
use bank_domain::queries::account_authorized_users;
use bank_domain::schema::{GrantAccountAuthorization, RevokeAccountAuthorization};
use bank_server::{
    mutations, BankAuthenticatedPrincipal, BankIdentityRuntime, BankMutationControls,
};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;
use worth_query_host::facade::product::WorthQueryProductBranch;

use super::key;

pub(super) fn revoke_approver(
    runtime: &BankIdentityRuntime,
    owner: &BankAuthenticatedPrincipal,
    scope: &WorthQueryRequestScope,
    branch: WorthQueryProductBranch,
    account: AccountId,
    approver: BankPrincipalId,
) {
    let authorized = runtime
        .request(owner, scope)
        .on_branch(branch)
        .query(account_authorized_users(account))
        .execute()
        .expect("owner reads the exact current account grant");
    let [account_row] = authorized.rows() else {
        panic!("expected one business account authorization row");
    };
    let grant = account_row
        .users()
        .iter()
        .find(|user| user.principal() == approver && user.role() == CustomerRole::Approver)
        .expect("payment approver has the live account grant");
    let revoked = runtime
        .mutate(mutations::revoke_account_access(
            RevokeAccountAuthorization {
                account,
                authorization: grant.authorization(),
            },
        ))
        .as_principal(owner)
        .controls(BankMutationControls::new(
            scope.clone(),
            key("payment-court:revoke-approver"),
        ))
        .execute()
        .expect("owner revokes approval authority before callback delivery");
    assert!(matches!(
        revoked,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
}

pub(super) fn restore_approver(
    runtime: &BankIdentityRuntime,
    owner: &BankAuthenticatedPrincipal,
    scope: &WorthQueryRequestScope,
    account: AccountId,
    approver: BankPrincipalId,
) {
    let granted = runtime
        .mutate(mutations::grant_account_access(GrantAccountAuthorization {
            account,
            principal: approver,
            role: CustomerRole::Approver,
        }))
        .as_principal(owner)
        .controls(BankMutationControls::new(
            scope.clone(),
            key("payment-court:restore-approver"),
        ))
        .execute()
        .expect("owner restores current approval authority");
    assert!(matches!(
        granted,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
}
