//! The principal's visible accounts whose display name contains a needle,
//! ignoring ASCII case: a residual where filter over account discovery.
use worth_query_decl::facade::{
    application_query::{
        ApplicationQueryBasisSupport, ApplicationQueryCardinality,
        ApplicationQueryDefinitionBuilder, ApplicationQueryDisclosureContract,
        ApplicationQueryLaneEligibility, ApplicationQueryOrderingDirection,
        ApplicationQueryParameterRef, ApplicationQueryParameterSet,
        ApplicationQueryWhereFilterOperands,
    },
    application_schema::StringApplicationValueBinding,
    worth_query_application_query, worth_query_query_binding, worth_query_structured_value_binding,
};

use super::account_discovery::{
    discoverable_account_paths, AccountDiscoveryQueryParameters,
    AccountDiscoveryQueryParametersBinding, AccountDiscoveryQueryResultBinding,
};
use crate::authorization::DiscoverOwnAccounts;
use crate::reads::VisibleAccount;
use crate::schema::{Account, AccountDisplayName, BankSchema, Principal, PrincipalIdentityField};

/// Text an account's display name must contain, compared ignoring ASCII case.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountSearchRequest {
    needle: String,
}

pub fn account_search(needle: impl Into<String>) -> AccountSearchRequest {
    AccountSearchRequest {
        needle: needle.into(),
    }
}

/// The `needle` query parameter.
pub struct AccountSearchNeedle;

fn needle() -> ApplicationQueryParameterRef<
    AccountSearchQuery,
    AccountSearchNeedle,
    StringApplicationValueBinding,
> {
    ApplicationQueryParameterRef::from_query_identifier("needle")
}

fn parameters(request: &AccountSearchRequest) -> ApplicationQueryParameterSet<AccountSearchQuery> {
    ApplicationQueryParameterSet::new()
        .bind(needle(), request.needle.clone())
        .expect("every string encodes as a text parameter")
}

worth_query_application_query!(
    pub AccountSearchQuery for BankSchema,
    identity "worth.bank.account-search-query.v1",
    parameters AccountDiscoveryQueryParametersBinding,
    result AccountDiscoveryQueryResultBinding,
    scope Principal => "Principal",
    name "account_search"
);
worth_query_structured_value_binding!(pub AccountSearchRequestBinding for AccountSearchRequest {
    identity: "worth.bank.account-search-request.v1"
});
worth_query_query_binding!(
    pub AccountSearchQueryBinding for AccountSearchRequest, schema BankSchema, context crate::schema::BankBindingContext,
    identity "worth.bank.account-search-query-binding.v1",
    input AccountSearchRequestBinding,
    query AccountSearchQuery,
    parameters AccountDiscoveryQueryParametersBinding => parameters,
    result AccountDiscoveryQueryResultBinding,
    scope PrincipalIdentityField,
    principal_field PrincipalIdentityField::at("PrincipalIdentityField"),
    limits results 1_024
);

/// Account discovery's root paths, narrowed by a case-insensitive display-name filter.
pub fn account_search_definition() -> ApplicationQueryDefinitionBuilder<
    BankSchema,
    AccountSearchQuery,
    AccountDiscoveryQueryParameters,
    VisibleAccount,
    Principal,
> {
    let filter = ApplicationQueryWhereFilterOperands::new()
        .field("display_name", AccountDisplayName::at("AccountDisplayName"))
        .and_then(|operands| operands.parameter("needle", needle()))
        .and_then(|operands| operands.parse("contains_ignore_ascii_case(display_name, needle)"))
        .expect("the account search filter admits statically");
    let [personal, authorized, business] = discoverable_account_paths();
    ApplicationQueryDefinitionBuilder::declare(AccountSearchQuery::reference())
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
        .parameter(needle())
        .where_filter(filter)
        .order_by(
            VisibleAccount::account(),
            ApplicationQueryOrderingDirection::Ascending,
        )
}
