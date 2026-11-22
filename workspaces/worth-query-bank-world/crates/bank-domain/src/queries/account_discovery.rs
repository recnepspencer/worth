use worth_query_decl::facade::{
    application_query::{
        ApplicationQueryBasisSupport, ApplicationQueryCardinality,
        ApplicationQueryDefinitionBuilder, ApplicationQueryDisclosureContract,
        ApplicationQueryLaneEligibility, ApplicationQueryOrderingDirection,
        ApplicationQueryRootPath,
    },
    worth_query_application_query,
};

use crate::{
    authorization::DiscoverOwnAccounts,
    reads::VisibleAccount,
    schema::{
        Account, AccountAuthorizedUser, AuthorizationAccount, BankSchema, BusinessAccount,
        BusinessOwner, PersonalOwner, Principal,
    },
};

pub struct AccountDiscoveryQueryParameters;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccountDiscoveryRequest;

pub const fn accounts() -> AccountDiscoveryRequest {
    AccountDiscoveryRequest
}

worth_query_decl::facade::worth_query_structured_value_binding!(pub AccountDiscoveryQueryParametersBinding for AccountDiscoveryQueryParameters { identity: "AccountDiscoveryQueryParameters" });
worth_query_decl::facade::worth_query_structured_value_binding!(pub AccountDiscoveryQueryResultBinding for VisibleAccount { identity: "VisibleAccount" });
worth_query_application_query!(
    pub AccountDiscoveryQuery for BankSchema,
    identity "AccountDiscoveryQuery",
    parameters AccountDiscoveryQueryParametersBinding,
    result AccountDiscoveryQueryResultBinding,
    scope Principal => "Principal",
    name "account_discovery"
);
worth_query_decl::facade::worth_query_structured_value_binding!(pub AccountDiscoveryRequestBinding for AccountDiscoveryRequest { identity: "AccountDiscoveryRequest" });
worth_query_decl::facade::worth_query_query_binding!(
    pub AccountDiscoveryQueryBinding for AccountDiscoveryRequest, schema BankSchema, context crate::schema::BankBindingContext,
    identity "worth.bank.account-discovery-query-binding.v1",
    input AccountDiscoveryRequestBinding,
    query AccountDiscoveryQuery,
    parameters AccountDiscoveryQueryParametersBinding => |_| worth_query_decl::facade::application_query::ApplicationQueryParameterSet::new(),
    result AccountDiscoveryQueryResultBinding,
    scope crate::schema::PrincipalIdentityField,
    principal_field crate::schema::PrincipalIdentityField::at("PrincipalIdentityField"),
    limits results 1_024

);

pub fn account_discovery_definition() -> ApplicationQueryDefinitionBuilder<
    BankSchema,
    AccountDiscoveryQuery,
    AccountDiscoveryQueryParameters,
    VisibleAccount,
    Principal,
> {
    let [personal, authorized, business] = discoverable_account_paths();
    ApplicationQueryDefinitionBuilder::declare(AccountDiscoveryQuery::reference())
        .root(Account::reference())
        .scope(Principal::reference())
        .result_shape(VisibleAccount::result_shape())
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .requires_ability(DiscoverOwnAccounts::reference())
        .root_path(personal)
        .root_path(authorized)
        .root_path(business)
        .order_by(
            VisibleAccount::account(),
            ApplicationQueryOrderingDirection::Ascending,
        )
}

/// Every account a principal may discover: owned personally, used as an
/// authorized user, or owned through a business.
pub(super) fn discoverable_account_paths(
) -> [ApplicationQueryRootPath<BankSchema, Principal, Account>; 3] {
    [
        ApplicationQueryRootPath::from(Principal::reference()).forward(PersonalOwner::reference()),
        ApplicationQueryRootPath::from(Principal::reference())
            .forward(AccountAuthorizedUser::reference())
            .forward(AuthorizationAccount::reference()),
        ApplicationQueryRootPath::from(Principal::reference())
            .reverse(BusinessOwner::reference())
            .forward(BusinessAccount::reference()),
    ]
}
