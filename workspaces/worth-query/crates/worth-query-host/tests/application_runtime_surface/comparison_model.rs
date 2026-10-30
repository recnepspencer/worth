//! Governed one-field tree for the ordinary Host paired inspection journey.
use super::inspection_capability::{text, InspectAccountCapability};
use super::inspection_model::{
    number_result, AccountParameters, AccountParametersBinding, AccountResultBinding,
};
use super::{
    Account, AccountIdentity, AccountNumber, ExternalMapping, HostIdentitySchema, IdentityBinding,
    Principal,
};
use worth_query_decl::facade::{
    application_query::{
        ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
        ApplicationQueryDefinitionBuilder, ApplicationQueryDisclosureContract,
        ApplicationQueryInfluenceContract, ApplicationQueryInspectionDefinition,
        ApplicationQueryLaneEligibility, ApplicationQueryParameterSet,
        ApplicationQueryResultShapeBuilder,
    },
    application_schema::{NoApplicationUnit, ReadOnly},
    worth_query_application_query, worth_query_query_binding, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};
worth_query_application_query!(pub GovernedAccountQuery for HostIdentitySchema,
    identity "HostGovernedInspectionAccountQuery", parameters AccountParametersBinding,
    result AccountResultBinding, scope Account => "Account", name "host_governed_inspection_account_query");

pub(super) fn definition() -> ApplicationQueryDefinition<
    HostIdentitySchema,
    GovernedAccountQuery,
    AccountParameters,
    String,
    Account,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        HostIdentitySchema,
        GovernedAccountQuery,
        Account,
        String,
        AccountResultBinding,
    >::new(Account::reference())
    .field(number_result())
    .build();
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        "host-number-diagnostic",
        InspectAccountCapability::reference(),
    )
    .disclose_field_by(
        number_result::<GovernedAccountQuery>(),
        text("number"),
        ApplicationQueryInfluenceContract::permit_all(),
    );
    ApplicationQueryDefinitionBuilder::declare(GovernedAccountQuery::reference())
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(disclosure)
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
}
impl WorthQueryApplicationProjection<HostIdentitySchema, GovernedAccountQuery> for String {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, HostIdentitySchema, GovernedAccountQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        row.field(number_result())
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GovernedAccountRead(pub String);
impl GovernedAccountRead {
    fn into_number(self) -> String {
        self.0
    }
}
worth_query_structured_value_binding!(pub GovernedAccountReadInputBinding for GovernedAccountRead {
    identity: "worth.query.host.inspection.governed-read-input.v1"
});
worth_query_query_binding!(pub GovernedAccountReadBinding for GovernedAccountRead,
    schema HostIdentitySchema, identity "worth.query.host.inspection.governed-read-binding.v1",
    input GovernedAccountReadInputBinding, query GovernedAccountQuery,
    parameters AccountParametersBinding => |_| ApplicationQueryParameterSet::new(),
    result AccountResultBinding,
    principal IdentityBinding, mapping ExternalMapping, principal_entity Principal,
        principal_identity u64, identity_binding worth_query_decl::facade::application_schema::U64ApplicationValueBinding,
    scope Account, AccountIdentity, AccountNumber, String, ReadOnly, NoApplicationUnit,
    field AccountNumber::reference(), value GovernedAccountRead::into_number, limits results 1
);
pub(super) fn inspection_definition(
) -> ApplicationQueryInspectionDefinition<HostIdentitySchema, GovernedAccountReadBinding> {
    ApplicationQueryInspectionDefinition::new(ApplicationQueryDisclosureContract::public()).unwrap()
}
