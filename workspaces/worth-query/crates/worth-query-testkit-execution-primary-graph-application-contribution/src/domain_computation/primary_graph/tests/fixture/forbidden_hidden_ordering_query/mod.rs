use worth_query_declaration::facade::application_query::ApplicationQueryBasisSupport;
use worth_query_declaration::facade::application_query::ApplicationQueryCardinality;
use worth_query_declaration::facade::application_query::ApplicationQueryDefinition;
use worth_query_declaration::facade::application_query::ApplicationQueryDefinitionBuilder;
use worth_query_declaration::facade::application_query::ApplicationQueryDisclosureContract;
use worth_query_declaration::facade::application_query::ApplicationQueryInfluenceContract;
use worth_query_declaration::facade::application_query::ApplicationQueryLaneEligibility;
use worth_query_declaration::facade::application_query::ApplicationQueryOrderingDirection;
use worth_query_declaration::facade::application_query::ApplicationQueryResultShapeBuilder;
use worth_query_declaration::facade::application_schema::ApplicationEncodedScalarValue;
use worth_query_declaration::worth_query_application_query;

use crate::domain_computation::primary_graph::tests::fixture::application_queries::label_result_field;
use crate::domain_computation::primary_graph::tests::fixture::application_queries::status_result_field;
use crate::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters;
use crate::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryResult;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDisclosure;
use crate::domain_computation::primary_graph::tests::fixture::Account;
use crate::domain_computation::primary_graph::tests::fixture::AccountLabel;
use crate::domain_computation::primary_graph::tests::fixture::CapabilityDisclosureBinding;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use crate::domain_computation::primary_graph::tests::fixture::TouchAccountCapability;

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

pub fn forbidden_hidden_ordering_definition() -> ApplicationQueryDefinition<
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

pub fn encoded_disclosure(
    value: CapabilityDisclosure,
) -> ApplicationEncodedScalarValue<CapabilityDisclosureBinding> {
    ApplicationEncodedScalarValue::try_new(value).expect("fixture disclosure must encode")
}
