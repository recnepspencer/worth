use worth_query_decl::facade::worth_query_application_query;
use worth_query_decl::facade::{
    application_query::{
        ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
        ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
        ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
        ApplicationQueryOrderingDirection, ApplicationQueryResultFieldRef,
        ApplicationQueryResultRelationRef, ApplicationQueryResultShapeBuilder, ExactlyOneResult,
        ManyResults, ReverseResultTraversal,
    },
    application_schema::{EqualityPredicate, NoApplicationUnit, ReadOnly, ReadWrite},
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

use crate::authorization::ViewAccountAccess;
use crate::model::{AccountAuthorizationId, AccountId, BankPrincipalId, CustomerRole};
use crate::reads::AuthorizedAccountUser;
use crate::schema::{
    Account, AccountAuthorization, AccountAuthorizationIdentity, AccountAuthorizedUser,
    AuthorizationAccount, AuthorizationIdentity, AuthorizationRole, AuthorizationScope, BankSchema,
    Principal, PrincipalIdentity, PrincipalIdentityField,
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

struct AuthorizationsSlot;
worth_query_decl::facade::worth_query_portable_type!(AuthorizationsSlot => "AuthorizationsSlot");
struct AuthorizationIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(AuthorizationIdentitySlot => "AuthorizationIdentitySlot");
struct AuthorizationRoleSlot;
worth_query_decl::facade::worth_query_portable_type!(AuthorizationRoleSlot => "AuthorizationRoleSlot");
struct AuthorizationPrincipalSlot;
worth_query_decl::facade::worth_query_portable_type!(AuthorizationPrincipalSlot => "AuthorizationPrincipalSlot");
struct PrincipalIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(PrincipalIdentitySlot => "PrincipalIdentitySlot");

type AuthorizationIdentitySelector = ApplicationQueryResultFieldRef<
    AccountAuthorizedUsersQuery,
    AuthorizationIdentitySlot,
    BankSchema,
    AccountAuthorization,
    AuthorizationIdentity,
    AccountAuthorizationIdentity,
    AccountAuthorizationId,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
>;

type AuthorizationRoleSelector = ApplicationQueryResultFieldRef<
    AccountAuthorizedUsersQuery,
    AuthorizationRoleSlot,
    BankSchema,
    AccountAuthorization,
    AuthorizationScope,
    AuthorizationRole,
    CustomerRole,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
>;

type PrincipalIdentitySelector = ApplicationQueryResultFieldRef<
    AccountAuthorizedUsersQuery,
    PrincipalIdentitySlot,
    BankSchema,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityField,
    BankPrincipalId,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
>;

pub fn account_authorized_users_definition() -> ApplicationQueryDefinition<
    BankSchema,
    AccountAuthorizedUsersQuery,
    AccountAuthorizedUsersQueryParameters,
    AccountAuthorizedUsersQueryResult,
    Account,
> {
    let principal = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        AccountAuthorizedUsersQuery,
        Principal,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(Principal::reference())
    .field(principal_identity());
    let authorization = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        AccountAuthorizedUsersQuery,
        AccountAuthorization,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(AccountAuthorization::reference())
    .field(authorization_identity())
    .field(authorization_role())
    .relation(authorization_principal(), principal);
    let shape = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        AccountAuthorizedUsersQuery,
        Account,
        AccountAuthorizedUsersQueryResult,
        AccountAuthorizedUsersQueryResultBinding,
    >::new(Account::reference())
    .relation(account_authorizations(), authorization)
    .build();
    ApplicationQueryDefinitionBuilder::declare(AccountAuthorizedUsersQuery::reference())
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(2, 2, 3))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .requires_ability(ViewAccountAccess::reference())
        .order_by(
            authorization_identity(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .continue_by(account_authorizations())
        .build()
        .expect("bank authorized account users query is statically canonical")
}

impl WorthQueryApplicationProjection<BankSchema, AccountAuthorizedUsersQuery>
    for AccountAuthorizedUsersQueryResult
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, BankSchema, AccountAuthorizedUsersQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let users = row
            .many(account_authorizations())?
            .iter()
            .map(|authorization| {
                Ok(AuthorizedAccountUser::from_projection(
                    authorization.field(authorization_identity())?,
                    authorization
                        .one(authorization_principal())?
                        .field(principal_identity())?,
                    authorization.field(authorization_role())?,
                ))
            })
            .collect::<Result<Vec<_>, WorthQueryApplicationProjectionDenial>>()?;
        Ok(Self { users })
    }
}

fn authorization_identity() -> AuthorizationIdentitySelector {
    ApplicationQueryResultFieldRef::new("authorization", AccountAuthorizationIdentity::reference())
}

fn authorization_role() -> AuthorizationRoleSelector {
    ApplicationQueryResultFieldRef::new("role", AuthorizationRole::reference())
}

fn principal_identity() -> PrincipalIdentitySelector {
    ApplicationQueryResultFieldRef::new("principal", PrincipalIdentityField::reference())
}

fn account_authorizations() -> ApplicationQueryResultRelationRef<
    AccountAuthorizedUsersQuery,
    AuthorizationsSlot,
    BankSchema,
    AuthorizationAccount,
    AccountAuthorization,
    Account,
    ReverseResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::reverse_many(
        "authorizations",
        AuthorizationAccount::reference(),
    )
}

fn authorization_principal() -> ApplicationQueryResultRelationRef<
    AccountAuthorizedUsersQuery,
    AuthorizationPrincipalSlot,
    BankSchema,
    AccountAuthorizedUser,
    Principal,
    AccountAuthorization,
    ReverseResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::reverse_one("principal", AccountAuthorizedUser::reference())
}
