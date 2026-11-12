//! Successor-owned balance query for the additive B application.
use super::schema::{
    Account, AccountIdentity, ExternalMapping, HomeRecords, IdentityBinding, Principal,
    SuccessorSchema,
};
use worth_query_decl::facade::{
    application_query::{
        ApplicationQueryBasisSupport, ApplicationQueryCardinality,
        ApplicationQueryDefinitionBuilder, ApplicationQueryDisclosureContract,
        ApplicationQueryLaneEligibility, ApplicationQueryParameterSet,
        ApplicationQueryResultFieldRef, ApplicationQueryResultShapeBuilder,
    },
    application_schema::fragment::{self, FieldSelection},
    worth_query_application_query, worth_query_portable_type, worth_query_query_binding,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

pub struct AccountParameters;
worth_query_structured_value_binding!(pub AccountParametersBinding for AccountParameters {
    identity: "worth.query.host.home.balance.parameters.v1"
});
worth_query_structured_value_binding!(pub AccountResultBinding for u64 {
    identity: "worth.query.host.home.balance.result.v1"
});
pub struct NumberSlot;
worth_query_portable_type!(NumberSlot => "worth.query.host.home.balance.number-slot.v1");
worth_query_application_query!(
    pub AccountQuery for SuccessorSchema,
    identity "HostHomeBalanceQuery",
    parameters AccountParametersBinding,
    result AccountResultBinding,
    scope Account => "Account",
    name "host_home_balance_query"
);

pub(super) fn number_result<Q>() -> ApplicationQueryResultFieldRef<
    Q,
    NumberSlot,
    SuccessorSchema,
    FieldSelection<HomeRecords, Account, AccountIdentity, u64>,
> {
    ApplicationQueryResultFieldRef::new(
        "balance",
        FieldSelection::<HomeRecords, Account, AccountIdentity, u64>::at("Balance"),
    )
}

pub(super) fn definition(
) -> ApplicationQueryDefinitionBuilder<SuccessorSchema, AccountQuery, AccountParameters, u64, Account>
{
    let shape = ApplicationQueryResultShapeBuilder::<
        SuccessorSchema,
        AccountQuery,
        Account,
        u64,
        AccountResultBinding,
    >::new(fragment::entity(HomeRecords, Account))
    .field(number_result())
    .build();
    ApplicationQueryDefinitionBuilder::declare(AccountQuery::reference())
        .root(fragment::entity(HomeRecords, Account))
        .scope(fragment::entity(HomeRecords, Account))
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
}

impl WorthQueryApplicationProjection<SuccessorSchema, AccountQuery> for u64 {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, SuccessorSchema, AccountQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        row.field(number_result())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountRead(pub String);
impl AccountRead {
    fn into_number(self) -> String {
        self.0
    }
}
worth_query_structured_value_binding!(pub AccountReadInputBinding for AccountRead {
    identity: "worth.query.host.home.balance.read-input.v1"
});
worth_query_query_binding!(
    pub AccountReadBinding for AccountRead, schema SuccessorSchema,
    identity "worth.query.host.home.balance.read-binding.v1",
    input AccountReadInputBinding,
    query AccountQuery,
    parameters AccountParametersBinding => |_| ApplicationQueryParameterSet::new(),
    result AccountResultBinding,
    principal IdentityBinding, mapping ExternalMapping, principal_entity Principal,
        principal_identity u64, identity_binding worth_query_decl::facade::application_schema::U64ApplicationValueBinding,
    scope FieldSelection<HomeRecords, Account, AccountIdentity, String>,
    field FieldSelection::<HomeRecords, Account, AccountIdentity, String>::at("AccountNumber"),
    value AccountRead::into_number,
    limits results 1
);
