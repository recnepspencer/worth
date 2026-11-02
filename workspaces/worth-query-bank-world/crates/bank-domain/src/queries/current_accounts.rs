//! Current-only managed access to the unchanged principal-specific v1 Source.
use super::account_discovery::{
    account_identity, AccountDiscoveryQueryBinding, AccountDiscoveryQueryParameters,
    AccountDiscoveryQueryParametersBinding, AccountDiscoveryQueryResultBinding,
};
use crate::{authorization::DiscoverOwnAccounts, reads::VisibleAccount, schema::*};
use worth_query_decl::facade::{application_query::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrentAccountsRequest;
pub const fn current_accounts() -> CurrentAccountsRequest {
    CurrentAccountsRequest
}
worth_query_structured_value_binding!(pub CurrentAccountsRequestBinding for CurrentAccountsRequest {
    identity: "worth.bank.current-accounts-request.v1"
});
worth_query_application_query!(
    pub CurrentAccountsQuery for BankSchema,
    identity "worth.bank.current-accounts-query.v1",
    parameters AccountDiscoveryQueryParametersBinding,
    result AccountDiscoveryQueryResultBinding,
    scope Principal => "Principal",
    name "current_accounts"
);
#[derive(Clone, Copy, Debug, worth_foundational::facade::WorthValue)]
pub struct AllAccountsProbe {}
worth_query_value_binding!(pub AllAccountsProbeBinding for AllAccountsProbe {
    identity: "worth.bank.all-accounts-probe.v1", scalar: String,
    canonical: ValueDecodeBounds::new(64, 64, 4, 4),
});
pub struct AllAccountsParameter;
fn all_accounts(
) -> ApplicationQueryParameterRef<CurrentAccountsQuery, AllAccountsParameter, AllAccountsProbeBinding>
{
    ApplicationQueryParameterRef::from_query_identifier("all")
}
worth_query_query_binding!(
    pub CurrentAccountsQueryBinding for CurrentAccountsRequest, schema BankSchema,
    identity "worth.bank.current-accounts-query-binding.v1",
    input CurrentAccountsRequestBinding,
    query CurrentAccountsQuery,
    parameters AccountDiscoveryQueryParametersBinding => |_| ApplicationQueryParameterSet::new().bind(all_accounts(), AllAccountsProbe {}).unwrap(),
    result AccountDiscoveryQueryResultBinding,
    principal BankPrincipalBinding, mapping ExternalPrincipalMapping, principal_entity Principal,
    principal_identity crate::model::BankPrincipalId, identity_binding BankPrincipalIdBinding,
    scope Principal, PrincipalIdentity, PrincipalIdentityField,
        crate::model::BankPrincipalId, ReadOnly, NoApplicationUnit,
    principal_field PrincipalIdentityField::reference(),
    limits results 1_024
);

pub struct CurrentAccountsIndex;
impl ApplicationManagedIndexBinding<BankSchema> for CurrentAccountsIndex {
    type SourceBinding = AccountDiscoveryQueryBinding;
    type ProbeBinding = AllAccountsProbeBinding;
    type Entity = Account;
    const NAME: &'static str = "current_accounts_roster";
    const PROVIDER: &'static str = "worth.bank.account-roster.v1";
    const PROVIDER_VERSION: u32 = 1;
    const MAINTENANCE: ApplicationManagedIndexMaintenance =
        ApplicationManagedIndexMaintenance::Native {
            cold: ApplicationManagedIndexColdSource::AdmitRebuild,
        };
    const STRUCTURE: ApplicationManagedIndexSourceStructure =
        ApplicationManagedIndexSourceStructure::BinarySourceTree;
    const MEMBERSHIP: ApplicationManagedIndexMembership =
        ApplicationManagedIndexMembership::ExhaustiveSourceTree;
}

pub fn current_accounts_definition() -> ApplicationQueryDefinition<
    BankSchema,
    CurrentAccountsQuery,
    AccountDiscoveryQueryParameters,
    VisibleAccount,
    Principal,
> {
    let identity = account_identity::<CurrentAccountsQuery>();
    ApplicationQueryDefinitionBuilder::declare(CurrentAccountsQuery::reference())
        .root(Account::reference())
        .scope(Principal::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::new(Account::reference())
                .field(identity)
                .build(),
        )
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_only())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .requires_ability(DiscoverOwnAccounts::reference())
        .managed_index(CurrentAccountsIndex::reference(), all_accounts())
        .order_by(identity, ApplicationQueryOrderingDirection::Ascending)
        .build()
        .expect("current accounts has one exact installed Source and all probe")
}
impl WorthQueryApplicationProjection<BankSchema, CurrentAccountsQuery> for VisibleAccount {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, BankSchema, CurrentAccountsQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self::new(row.field(account_identity())?))
    }
}
