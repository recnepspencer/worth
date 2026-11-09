use worth_query_declaration::facade::application_query::ApplicationQueryBasisSupport;
use worth_query_declaration::facade::application_query::ApplicationQueryCardinality;
use worth_query_declaration::facade::application_query::ApplicationQueryDefinitionBuilder;
use worth_query_declaration::facade::application_query::ApplicationQueryDisclosureContract;
use worth_query_declaration::facade::application_query::ApplicationQueryInfluenceContract;
use worth_query_declaration::facade::application_query::ApplicationQueryLaneEligibility;
use worth_query_declaration::facade::application_query::ApplicationQueryResultShapeBuilder;
use worth_query_declaration::facade::application_query::TypedApplicationQueryResultShape;
use worth_query_declaration::facade::application_schema::ApplicationEncodedScalarValue;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_declaration::worth_query_application_query;

use crate::domain_computation::primary_graph::tests::fixture::application_queries::label_result_field;
use crate::domain_computation::primary_graph::tests::fixture::application_queries::parameter_reference::status_parameter;
use crate::domain_computation::primary_graph::tests::fixture::application_queries::status_result_field;
use crate::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters;
use crate::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryResult;
use crate::domain_computation::primary_graph::tests::fixture::Account;
use crate::domain_computation::primary_graph::tests::fixture::AccountStatus;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDisclosure;
use crate::domain_computation::primary_graph::tests::fixture::CapabilityDisclosureBinding;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use crate::domain_computation::primary_graph::tests::fixture::TouchAccountCapability;

worth_query_declaration::worth_query_structured_value_binding!(pub IncompleteDisclosureQueryParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub IncompleteDisclosureQueryResultBinding for AccountSummaryResult { identity: "AccountSummaryResult" });
worth_query_application_query!(
    pub IncompleteDisclosureQuery for IdentityExecutionSchema,
    identity "IncompleteDisclosureQuery",
    parameters IncompleteDisclosureQueryParametersBinding,
    result IncompleteDisclosureQueryResultBinding,
    scope Account => "Account",
    name "incomplete_disclosure"
);

worth_query_declaration::worth_query_structured_value_binding!(pub ForbiddenInfluenceQueryParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub ForbiddenInfluenceQueryResultBinding for AccountSummaryResult { identity: "AccountSummaryResult" });
worth_query_application_query!(
    pub ForbiddenInfluenceQuery for IdentityExecutionSchema,
    identity "ForbiddenInfluenceQuery",
    parameters ForbiddenInfluenceQueryParametersBinding,
    result ForbiddenInfluenceQueryResultBinding,
    scope Account => "Account",
    name "forbidden_influence"
);

worth_query_declaration::worth_query_structured_value_binding!(pub ResultRulePredicateQueryParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub ResultRulePredicateQueryResultBinding for AccountSummaryResult { identity: "AccountSummaryResult" });
worth_query_application_query!(
    pub ResultRulePredicateQuery for IdentityExecutionSchema,
    identity "ResultRulePredicateQuery",
    parameters ResultRulePredicateQueryParametersBinding,
    result ResultRulePredicateQueryResultBinding,
    scope Account => "Account",
    name "result_rule_predicate"
);

pub fn incomplete_disclosure_definition() -> ApplicationQueryDefinitionBuilder<
    IdentityExecutionSchema,
    IncompleteDisclosureQuery,
    AccountSummaryParameters,
    AccountSummaryResult,
    Account,
> {
    let shape = shape::<IncompleteDisclosureQuery>();
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        "incomplete",
        TouchAccountCapability::reference(),
    )
    .disclose_field_by(
        status_result_field::<IncompleteDisclosureQuery>(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        ApplicationQueryInfluenceContract::forbid_all(),
    );
    definition(
        IncompleteDisclosureQuery::reference(),
        shape,
        disclosure,
        false,
    )
}

pub fn forbidden_influence_definition() -> ApplicationQueryDefinitionBuilder<
    IdentityExecutionSchema,
    ForbiddenInfluenceQuery,
    AccountSummaryParameters,
    AccountSummaryResult,
    Account,
> {
    let shape = shape::<ForbiddenInfluenceQuery>();
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        "forbidden-influence",
        TouchAccountCapability::reference(),
    )
    .use_field_by(
        AccountStatus::at("AccountStatus"),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        ApplicationQueryInfluenceContract::forbid_all(),
    )
    .expect("record field contract")
    .disclose_field_by(
        status_result_field::<ForbiddenInfluenceQuery>(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        ApplicationQueryInfluenceContract::forbid_all(),
    )
    .disclose_field_by(
        label_result_field::<ForbiddenInfluenceQuery>(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        ApplicationQueryInfluenceContract::forbid_all(),
    );
    definition(
        ForbiddenInfluenceQuery::reference(),
        shape,
        disclosure,
        true,
    )
}

pub fn result_rule_predicate_definition() -> ApplicationQueryDefinitionBuilder<
    IdentityExecutionSchema,
    ResultRulePredicateQuery,
    AccountSummaryParameters,
    AccountSummaryResult,
    Account,
> {
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        "result-rule-predicate",
        TouchAccountCapability::reference(),
    )
    .disclose_field_by(
        status_result_field::<ResultRulePredicateQuery>(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        ApplicationQueryInfluenceContract::permit_all(),
    )
    .disclose_field_by(
        label_result_field::<ResultRulePredicateQuery>(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        ApplicationQueryInfluenceContract::permit_all(),
    );
    definition(
        ResultRulePredicateQuery::reference(),
        shape::<ResultRulePredicateQuery>(),
        disclosure,
        true,
    )
}

pub fn shape<
    Query: worth_query_declaration::facade::application_query::ApplicationQueryMarkerIdentity<
        IdentityExecutionSchema,
    >,
>() -> TypedApplicationQueryResultShape<
    IdentityExecutionSchema,
    Query,
    Account,
    AccountSummaryResult,
    Query::ResultBinding,
>
where
    Query::ResultBinding: ApplicationStructuredValueBinding<Value = AccountSummaryResult>,
{
    ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        Query,
        Account,
        AccountSummaryResult,
        Query::ResultBinding,
    >::new(Account::reference())
    .field(status_result_field::<Query>())
    .field(label_result_field::<Query>())
    .build()
}

pub fn definition<
    Query: worth_query_declaration::facade::application_query::ApplicationQueryMarkerIdentity<
        IdentityExecutionSchema,
    >,
>(
    reference: worth_query_declaration::facade::application_query::ApplicationQueryReference<
        IdentityExecutionSchema,
        Query,
        AccountSummaryParameters,
        AccountSummaryResult,
        Account,
    >,
    shape: TypedApplicationQueryResultShape<
        IdentityExecutionSchema,
        Query,
        Account,
        AccountSummaryResult,
        Query::ResultBinding,
    >,
    disclosure: ApplicationQueryDisclosureContract,
    predicate: bool,
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
    let builder = ApplicationQueryDefinitionBuilder::declare(reference)
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(disclosure)
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public();
    if predicate {
        builder
            .parameter(status_parameter())
            .where_equal(AccountStatus::at("AccountStatus"), status_parameter())
    } else {
        builder
    }
}

pub fn encoded_disclosure(
    value: CapabilityDisclosure,
) -> ApplicationEncodedScalarValue<CapabilityDisclosureBinding> {
    ApplicationEncodedScalarValue::try_new(value).expect("fixture disclosure must encode")
}
