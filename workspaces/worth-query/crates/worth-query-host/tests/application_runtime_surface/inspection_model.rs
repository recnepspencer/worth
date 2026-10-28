//! One genuine public bound query over the existing Host identity fixture.
use super::{
    Account, AccountIdentity, AccountNumber, ExternalMapping, HostIdentitySchema, IdentityBinding,
    Principal,
};
use worth_query_decl::facade::{
    application_query::{
        ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
        ApplicationQueryDefinitionBuilder, ApplicationQueryDisclosureContract,
        ApplicationQueryInspectionDefinition, ApplicationQueryLaneEligibility,
        ApplicationQueryParameterSet, ApplicationQueryResultFieldRef,
        ApplicationQueryResultShapeBuilder,
    },
    application_schema::{
        EqualityPredicate, NoApplicationUnit, ReadOnly, StringApplicationValueBinding,
    },
    worth_query_application_query, worth_query_portable_type, worth_query_query_binding,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

pub struct AccountParameters;
worth_query_structured_value_binding!(pub AccountParametersBinding for AccountParameters {
    identity: "worth.query.host.inspection.parameters.v1"
});
worth_query_structured_value_binding!(pub AccountResultBinding for String {
    identity: "worth.query.host.inspection.result.v1"
});
pub struct NumberSlot;
worth_query_portable_type!(NumberSlot => "worth.query.host.inspection.number-slot.v1");
worth_query_application_query!(
    pub AccountQuery for HostIdentitySchema,
    identity "HostInspectionAccountQuery",
    parameters AccountParametersBinding,
    result AccountResultBinding,
    scope Account => "Account",
    name "host_inspection_account_query"
);

fn number_result() -> ApplicationQueryResultFieldRef<
    AccountQuery,
    NumberSlot,
    HostIdentitySchema,
    Account,
    AccountIdentity,
    AccountNumber,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("number", AccountNumber::reference())
}

pub(super) fn definition(
) -> ApplicationQueryDefinition<HostIdentitySchema, AccountQuery, AccountParameters, String, Account>
{
    let shape = ApplicationQueryResultShapeBuilder::<
        HostIdentitySchema,
        AccountQuery,
        Account,
        String,
        AccountResultBinding,
    >::new(Account::reference())
    .field(number_result())
    .build();
    ApplicationQueryDefinitionBuilder::declare(AccountQuery::reference())
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
}

impl WorthQueryApplicationProjection<HostIdentitySchema, AccountQuery> for String {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, HostIdentitySchema, AccountQuery>,
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
    identity: "worth.query.host.inspection.read-input.v1"
});
worth_query_query_binding!(
    pub AccountReadBinding for AccountRead, schema HostIdentitySchema,
    identity "worth.query.host.inspection.read-binding.v1",
    input AccountReadInputBinding,
    query AccountQuery,
    parameters AccountParametersBinding => |_| ApplicationQueryParameterSet::new(),
    result AccountResultBinding,
    principal IdentityBinding, mapping ExternalMapping, principal_entity Principal,
        principal_identity u64, identity_binding worth_query_decl::facade::application_schema::U64ApplicationValueBinding,
    scope Account, AccountIdentity, AccountNumber, String, ReadOnly, NoApplicationUnit,
    field AccountNumber::reference(),
    value AccountRead::into_number,
    limits results 1
);

pub(super) fn inspection_definition(
) -> ApplicationQueryInspectionDefinition<HostIdentitySchema, AccountReadBinding> {
    ApplicationQueryInspectionDefinition::new(ApplicationQueryDisclosureContract::public()).unwrap()
}
