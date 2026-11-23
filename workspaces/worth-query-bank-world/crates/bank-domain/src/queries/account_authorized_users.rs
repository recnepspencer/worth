use worth_query_decl::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinitionBuilder,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
    ApplicationQueryOrderingDirection,
};
use worth_query_decl::facade::{worth_query_application_query, WorthQueryProjection};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

use crate::authorization::ViewAccountAccess;
use crate::model::{AccountAuthorizationId, AccountId, BankPrincipalId, CustomerRole};
use crate::queries::UnitQueryResultBinding;
use crate::reads::AuthorizedAccountUser;
use crate::schema::{
    Account, AccountAuthorization, AccountAuthorizationIdentity, AccountAuthorizedUser,
    AuthorizationAccount, AuthorizationRole, BankSchema, Principal, PrincipalIdentityField,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccountAuthorizedUsersQueryParameters;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccountAuthorizedUsersRequest {
    account: AccountId,
}

impl AccountAuthorizedUsersRequest {
    pub const fn new(account: AccountId) -> Self {
        Self { account }
    }

    pub const fn account(self) -> AccountId {
        self.account
    }
}

pub const fn account_authorized_users(account: AccountId) -> AccountAuthorizedUsersRequest {
    AccountAuthorizedUsersRequest::new(account)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountAuthorizedUsersQueryResult {
    users: Vec<AuthorizedAccountUser>,
}

impl AccountAuthorizedUsersQueryResult {
    pub fn users(&self) -> &[AuthorizedAccountUser] {
        &self.users
    }

    pub fn into_users(self) -> Vec<AuthorizedAccountUser> {
        self.users
    }
}

worth_query_decl::facade::worth_query_structured_value_binding!(pub AccountAuthorizedUsersQueryParametersBinding for AccountAuthorizedUsersQueryParameters { identity: "AccountAuthorizedUsersQueryParameters" });
worth_query_decl::facade::worth_query_structured_value_binding!(pub AccountAuthorizedUsersQueryResultBinding for AccountAuthorizedUsersQueryResult { identity: "AccountAuthorizedUsersQueryResult" });
worth_query_application_query!(
    pub AccountAuthorizedUsersQuery for BankSchema,
    identity "AccountAuthorizedUsersQuery",
    parameters AccountAuthorizedUsersQueryParametersBinding,
    result AccountAuthorizedUsersQueryResultBinding,
    scope Account => "Account",
    name "account_authorized_users"
);
worth_query_decl::facade::worth_query_structured_value_binding!(pub AccountAuthorizedUsersRequestBinding for AccountAuthorizedUsersRequest { identity: "AccountAuthorizedUsersRequest" });
worth_query_decl::facade::worth_query_query_binding!(
    pub AccountAuthorizedUsersQueryBinding for AccountAuthorizedUsersRequest, schema BankSchema, context crate::schema::BankBindingContext,
    identity "worth.bank.account-authorized-users-query-binding.v1",
    input AccountAuthorizedUsersRequestBinding,
    query AccountAuthorizedUsersQuery,
    parameters AccountAuthorizedUsersQueryParametersBinding => |_| worth_query_decl::facade::application_query::ApplicationQueryParameterSet::new(),
    result AccountAuthorizedUsersQueryResultBinding,
    scope crate::schema::AccountIdentity,
    field crate::schema::AccountIdentity::at("AccountIdentity"),
    value AccountAuthorizedUsersRequest::account,
    limits results 1_024

);

#[derive(WorthQueryProjection)]
#[worth_query(schema = BankSchema, root = Account)]
struct AuthorizedUsersRow {
    #[worth_query(relation = AuthorizationAccount, reverse_from = AccountAuthorization,
        nested = UnitQueryResultBinding, slot = "AuthorizationsSlot")]
    authorizations: Vec<AuthorizationRow>,
}

#[derive(WorthQueryProjection)]
#[worth_query(schema = BankSchema, root = AccountAuthorization)]
struct AuthorizationRow {
    #[worth_query(field = AccountAuthorizationIdentity, slot = "AuthorizationIdentitySlot")]
    authorization: AccountAuthorizationId,
    #[worth_query(field = AuthorizationRole, slot = "AuthorizationRoleSlot")]
    role: CustomerRole,
    #[worth_query(relation = AccountAuthorizedUser, reverse_from = Principal,
        nested = UnitQueryResultBinding, slot = "AuthorizationPrincipalSlot")]
    principal: PrincipalRow,
}

#[derive(WorthQueryProjection)]
#[worth_query(schema = BankSchema, root = Principal)]
struct PrincipalRow {
    #[worth_query(field = PrincipalIdentityField, slot = "PrincipalIdentitySlot")]
    principal: BankPrincipalId,
}

pub fn account_authorized_users_definition() -> ApplicationQueryDefinitionBuilder<
    BankSchema,
    AccountAuthorizedUsersQuery,
    AccountAuthorizedUsersQueryParameters,
    AccountAuthorizedUsersQueryResult,
    Account,
> {
    let shape = AuthorizedUsersRow::result_shape_builder::<
        AccountAuthorizedUsersQuery,
        AccountAuthorizedUsersQueryResultBinding,
    >()
    .build();
    ApplicationQueryDefinitionBuilder::declare(AccountAuthorizedUsersQuery::reference())
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .requires_ability(ViewAccountAccess::reference())
        .order_by(
            AuthorizationRow::authorization(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .continue_by(AuthorizedUsersRow::authorizations())
}

impl WorthQueryApplicationProjection<BankSchema, AccountAuthorizedUsersQuery>
    for AccountAuthorizedUsersQueryResult
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, BankSchema, AccountAuthorizedUsersQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let users = AuthorizedUsersRow::project_row(row)?
            .authorizations
            .into_iter()
            .map(|authorization| {
                AuthorizedAccountUser::from_projection(
                    authorization.authorization,
                    authorization.principal.principal,
                    authorization.role,
                )
            })
            .collect();
        Ok(Self { users })
    }
}
