use bank_domain::proposals::BankSnapshot;
use bank_domain::schema::{
    AccountIdentity, GrantAccountAuthorization, GrantAccountAuthorizationOperation,
    RevokeAccountAuthorization, RevokeAccountAuthorizationOperation,
};

use super::bounded::{AccountEntity, BoundedProjectionState, ProjectionReader};
use super::BankProjectionDenial;

pub(crate) fn project_account_authorization_grant(
    reader: &mut ProjectionReader<'_, '_, GrantAccountAuthorizationOperation>,
    account: &AccountEntity,
    input: &GrantAccountAuthorization,
) -> Result<BankSnapshot, BankProjectionDenial> {
    let mut state = BoundedProjectionState::for_capability_projection(reader)?;
    state.project_admitted_account(reader, account, input.account)?;
    reader.require_decision_field(account, AccountIdentity::at("AccountIdentity"))?;
    let principal = state.project_principal(reader, input.principal)?;
    state.project_matching_authorization(reader, &principal, input.account)?;
    state
        .finish()
        .build()
        .map_err(BankProjectionDenial::InvalidDomainState)
}

pub(crate) fn project_account_authorization_revoke(
    reader: &mut ProjectionReader<'_, '_, RevokeAccountAuthorizationOperation>,
    account: &AccountEntity,
    admitted_account: bank_domain::model::AccountId,
    input: &RevokeAccountAuthorization,
) -> Result<BankSnapshot, BankProjectionDenial> {
    let mut state = BoundedProjectionState::for_capability_projection(reader)?;
    state.project_admitted_account(reader, account, admitted_account)?;
    state.project_authorization_by_id(reader, input.authorization)?;
    state
        .finish()
        .build()
        .map_err(BankProjectionDenial::InvalidDomainState)
}
