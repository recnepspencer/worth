//! Governed where filters: a filter field decides row presence and count,
//! so its internal rule must permit every membership surface.
use super::invalid_disclosure_queries::{definition, encoded_disclosure, shape};
use crate::domain_computation::primary_graph::tests::fixture::application_queries::parameter_reference::status_parameter;
use crate::domain_computation::primary_graph::tests::fixture::application_queries::{
    label_result_field, status_result_field, AccountSummaryParameters, AccountSummaryResult,
};
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDisclosure;
use crate::domain_computation::primary_graph::tests::fixture::{
    Account, AccountStatus, IdentityExecutionSchema, TouchAccountCapability,
};
use worth_query_declaration::facade::application_query::{
    ApplicationDisclosureClassification,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDisclosureContract,
    ApplicationQueryInfluenceContract, ApplicationQueryMarkerIdentity,
    ApplicationQueryObservableInfluence as Influence, ApplicationQueryReference,
    ApplicationQueryWhereFilterOperands,
};
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_declaration::worth_query_application_query;

worth_query_declaration::worth_query_disclosure_classification!(FilterMembershipDisclosure);
worth_query_declaration::worth_query_disclosure_classification!(FilterRowPresenceOnlyDisclosure);

worth_query_declaration::worth_query_structured_value_binding!(pub FilterRowPresenceOnlyParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub FilterRowPresenceOnlyResultBinding for AccountSummaryResult { identity: "AccountSummaryResult" });
worth_query_application_query!(
    pub FilterRowPresenceOnlyQuery for IdentityExecutionSchema,
    identity "FilterRowPresenceOnlyQuery",
    parameters FilterRowPresenceOnlyParametersBinding,
    result FilterRowPresenceOnlyResultBinding,
    scope Account => "Account",
    name "filter_row_presence_only"
);
worth_query_declaration::worth_query_structured_value_binding!(pub FilterMembershipParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub FilterMembershipResultBinding for AccountSummaryResult { identity: "AccountSummaryResult" });
worth_query_application_query!(
    pub FilterMembershipQuery for IdentityExecutionSchema,
    identity "FilterMembershipQuery",
    parameters FilterMembershipParametersBinding,
    result FilterMembershipResultBinding,
    scope Account => "Account",
    name "filter_membership"
);

/// Permits only row presence: the filter's count influence is undeclared.
pub fn filter_row_presence_only_definition() -> ApplicationQueryDefinitionBuilder<
    IdentityExecutionSchema,
    FilterRowPresenceOnlyQuery,
    AccountSummaryParameters,
    AccountSummaryResult,
    Account,
> {
    filtered(
        FilterRowPresenceOnlyQuery::reference(),
        FilterRowPresenceOnlyDisclosure,
        [Influence::RowPresence],
    )
}

/// Permits every membership surface a one-shot filter exposes.
pub fn filter_membership_definition() -> ApplicationQueryDefinitionBuilder<
    IdentityExecutionSchema,
    FilterMembershipQuery,
    AccountSummaryParameters,
    AccountSummaryResult,
    Account,
> {
    filtered(
        FilterMembershipQuery::reference(),
        FilterMembershipDisclosure,
        [Influence::RowPresence, Influence::Count],
    )
}

fn filtered<
    Query: ApplicationQueryMarkerIdentity<IdentityExecutionSchema> + 'static,
    Classification: ApplicationDisclosureClassification,
    const N: usize,
>(
    reference: ApplicationQueryReference<
        IdentityExecutionSchema,
        Query,
        AccountSummaryParameters,
        AccountSummaryResult,
        Account,
    >,
    classification: Classification,
    influence: [Influence; N],
) -> ApplicationQueryDefinitionBuilder<
    IdentityExecutionSchema,
    Query,
    AccountSummaryParameters,
    AccountSummaryResult,
    Account,
>
where
    Query::ResultBinding: ApplicationStructuredValueBinding<Value = AccountSummaryResult>,
{
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        classification,
        TouchAccountCapability::reference(),
    )
    .use_field_by(
        AccountStatus::at("AccountStatus"),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        ApplicationQueryInfluenceContract::permit(influence),
    )
    .expect("record field contract")
    .disclose_field_by(
        status_result_field::<Query>(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        ApplicationQueryInfluenceContract::permit_all(),
    )
    .disclose_field_by(
        label_result_field::<Query>(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        ApplicationQueryInfluenceContract::permit_all(),
    );
    let filter = ApplicationQueryWhereFilterOperands::<Query>::new()
        .field("account_status", AccountStatus::at("AccountStatus"))
        .and_then(|operands| operands.parameter("status", status_parameter()))
        .and_then(|operands| operands.parse("account_status == status"))
        .expect("the governed filter admits statically");
    definition(reference, shape::<Query>(), disclosure, false)
        .parameter(status_parameter())
        .where_filter(filter)
}
