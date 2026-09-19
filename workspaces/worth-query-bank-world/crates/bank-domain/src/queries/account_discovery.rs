use worth_query_decl::facade::{
    application_query::{
        ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
        ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
        ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
        ApplicationQueryOrderingDirection, ApplicationQueryResultFieldRef,
        ApplicationQueryResultShapeBuilder, ApplicationQueryRootPath,
    },
    application_schema::{EqualityPredicate, NoApplicationUnit, ReadOnly},
    worth_query_application_query,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

use crate::{
    authorization::DiscoverOwnAccounts,
    model::AccountId,
    reads::VisibleAccount,
    schema::{
        Account, AccountAuthorizedUser, AccountIdentity, AuthorizationAccount, BankSchema,
        BusinessAccount, BusinessOwner, Identity, PersonalOwner, Principal,
    },
};

pub struct AccountDiscoveryQueryParameters;
pub struct AccountIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(AccountIdentitySlot => "AccountIdentitySlot");

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
    pub AccountDiscoveryQueryBinding for AccountDiscoveryRequest, schema BankSchema,
    identity "worth.bank.account-discovery-query-binding.v1",
    input AccountDiscoveryRequestBinding,
    query AccountDiscoveryQuery,
    parameters AccountDiscoveryQueryParametersBinding => |_| worth_query_decl::facade::application_query::ApplicationQueryParameterSet::new(),
    result AccountDiscoveryQueryResultBinding,
    principal crate::schema::BankPrincipalBinding, mapping crate::schema::ExternalPrincipalMapping, principal_entity crate::schema::Principal,
        principal_identity crate::model::BankPrincipalId, identity_binding crate::schema::BankPrincipalIdBinding,
    scope Principal, crate::schema::PrincipalIdentity, crate::schema::PrincipalIdentityField,
        crate::model::BankPrincipalId, ReadOnly, NoApplicationUnit,
    principal_field crate::schema::PrincipalIdentityField::reference(),
    limits results 1_024, work 100_000

);

pub fn account_discovery_definition() -> ApplicationQueryDefinition<
    BankSchema,
    AccountDiscoveryQuery,
    AccountDiscoveryQueryParameters,
    VisibleAccount,
    Principal,
> {
    let identity = account_identity();
    let shape = ApplicationQueryResultShapeBuilder::new(Account::reference())
        .field(identity)
        .build();
    ApplicationQueryDefinitionBuilder::declare(AccountDiscoveryQuery::reference())
        .root(Account::reference())
        .scope(Principal::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::Many)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(2, 5, 1))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .requires_ability(DiscoverOwnAccounts::reference())
        .root_path(
            ApplicationQueryRootPath::from(Principal::reference())
                .forward(PersonalOwner::reference()),
        )
        .root_path(
            ApplicationQueryRootPath::from(Principal::reference())
                .forward(AccountAuthorizedUser::reference())
                .forward(AuthorizationAccount::reference()),
        )
        .root_path(
            ApplicationQueryRootPath::from(Principal::reference())
                .reverse(BusinessOwner::reference())
                .forward(BusinessAccount::reference()),
        )
        .order_by(identity, ApplicationQueryOrderingDirection::Ascending)
        .build()
        .expect("bank account discovery query is statically canonical")
}

impl WorthQueryApplicationProjection<BankSchema, AccountDiscoveryQuery> for VisibleAccount {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, BankSchema, AccountDiscoveryQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self::new(row.field(account_identity())?))
    }
}

fn account_identity() -> ApplicationQueryResultFieldRef<
    AccountDiscoveryQuery,
    AccountIdentitySlot,
    BankSchema,
    Account,
    Identity,
    AccountIdentity,
    AccountId,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("account", AccountIdentity::reference())
}
