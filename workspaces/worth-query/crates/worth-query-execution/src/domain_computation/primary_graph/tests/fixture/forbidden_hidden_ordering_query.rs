use worth_query_declaration::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryInfluenceContract,
    ApplicationQueryLaneEligibility, ApplicationQueryOrderingDirection,
    ApplicationQueryResultShapeBuilder,
};
use worth_query_declaration::facade::application_schema::ApplicationEncodedScalarValue;
use worth_query_declaration::worth_query_application_query;

use super::application_queries::{
    label_result_field, status_result_field, AccountSummaryParameters, AccountSummaryResult,
};
use super::{
    Account, AccountLabel, CapabilityDisclosure, CapabilityDisclosureBinding,
    IdentityExecutionSchema, TouchAccountCapability,
};

worth_query_declaration::worth_query_structured_value_binding!(pub ForbiddenHiddenOrderingQueryParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub ForbiddenHiddenOrderingQueryResultBinding for AccountSummaryResult { identity: "AccountSummaryResult" });
worth_query_application_query!(
    pub ForbiddenHiddenOrderingQuery for IdentityExecutionSchema,
    identity "ForbiddenHiddenOrderingQuery",
    parameters ForbiddenHiddenOrderingQueryParametersBinding,
    result ForbiddenHiddenOrderingQueryResultBinding,
    scope Account => "Account",
    name "forbidden_hidden_ordering"
);

pub(super) fn forbidden_hidden_ordering_definition() -> ApplicationQueryDefinition<
    IdentityExecutionSchema,
    ForbiddenHiddenOrderingQuery,
    AccountSummaryParameters,
    AccountSummaryResult,
    Account,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        ForbiddenHiddenOrderingQuery,
        Account,
        AccountSummaryResult,
        ForbiddenHiddenOrderingQueryResultBinding,
    >::new(Account::reference())
    .field(status_result_field::<ForbiddenHiddenOrderingQuery>())
    .field(label_result_field::<ForbiddenHiddenOrderingQuery>())
    .build();
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        "forbidden-hidden-ordering",
        TouchAccountCapability::reference(),
    )
    .use_field_by(
        AccountLabel::reference(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        ApplicationQueryInfluenceContract::forbid_all(),
    )
    .disclose_field_by(
        status_result_field::<ForbiddenHiddenOrderingQuery>(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        ApplicationQueryInfluenceContract::permit_all(),
    )
    .disclose_field_by(
        label_result_field::<ForbiddenHiddenOrderingQuery>(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        ApplicationQueryInfluenceContract::permit_all(),
    );
    ApplicationQueryDefinitionBuilder::declare(ForbiddenHiddenOrderingQuery::reference())
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(0, 0, 2))
        .disclosure(disclosure)
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .order_by(
            label_result_field::<ForbiddenHiddenOrderingQuery>(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .build()
        .unwrap()
}

fn encoded_disclosure(
    value: CapabilityDisclosure,
) -> ApplicationEncodedScalarValue<CapabilityDisclosureBinding> {
    ApplicationEncodedScalarValue::try_new(value).expect("fixture disclosure must encode")
}
