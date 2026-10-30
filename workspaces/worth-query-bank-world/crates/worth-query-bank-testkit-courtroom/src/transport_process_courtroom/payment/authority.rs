//! The account owner's independent authority changes around callback delivery.

pub use bank_domain::model::{AccountId, BankPrincipalId, CustomerRole};
pub use bank_domain::queries::account_authorized_users;
pub use bank_domain::schema::{GrantAccountAuthorization, RevokeAccountAuthorization};
pub use bank_server::{
    mutations, BankAuthenticatedPrincipal, BankIdentityRuntime, BankMutationControls,
};
pub use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
pub use worth_query_host::facade::application_entry::{
    WorthQuerySourceCompletion, WorthQuerySourceOnlyAdvance,
};
pub use worth_query_host::facade::product::WorthQueryProductBranch;

pub use super::key;

pub fn revoke_approver(
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
    assert_eq!(
        branch,
        runtime.current_branch(),
        "this court retains its original current branch"
    );
    let request = runtime.request(owner, scope).on_branch(branch);
    let operation = runtime
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
        .retain_operation()
        .unwrap_or_else(|cause| panic!("revocation construction: {cause:?}"));
    let WorthQuerySourceOnlyAdvance::Completed(revoked) = operation.advance(&request) else {
        panic!("owner revokes before callback")
    };
    assert!(matches!(
        revoked.evidence(),
        WorthQuerySourceCompletion::Fresh { .. }
    ));
    assert!(revoked.close(&request).is_ok());
}

pub fn restore_approver(
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
        .retain_operation()
        .unwrap_or_else(|cause| panic!("grant construction: {cause:?}"));
    let request = runtime.request(owner, scope);
    let WorthQuerySourceOnlyAdvance::Completed(granted) = granted.advance(&request) else {
        panic!("owner restores current approval")
    };
    assert!(matches!(
        granted.evidence(),
        WorthQuerySourceCompletion::Fresh { .. }
    ));
    assert!(granted.close(&request).is_ok());
}
