use super::*;

worth_query_declaration::worth_query_structured_value_binding!(pub PublicScopedAccountSummaryParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub PublicScopedAccountSummaryResultBinding for AccountSummaryResult { identity: "worth.query.test.execution.account_summary.result.v1" });
worth_query_declaration::worth_query_application_query!(
    pub PublicScopedAccountSummaryQuery for IdentityExecutionSchema,
    identity "PublicScopedAccountSummaryQuery",
    parameters PublicScopedAccountSummaryParametersBinding,
    result PublicScopedAccountSummaryResultBinding,
    scope Account => "Account",
    name "public_scoped_account_summary"
);

pub(in crate::domain_computation::primary_graph::tests::fixture) fn public_scoped_definition(
) -> ApplicationQueryDefinition<
    IdentityExecutionSchema,
    PublicScopedAccountSummaryQuery,
    AccountSummaryParameters,
    AccountSummaryResult,
    Account,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        PublicScopedAccountSummaryQuery,
        Account,
        AccountSummaryResult,
        PublicScopedAccountSummaryResultBinding,
    >::new(Account::reference())
    .field(status_result_field::<PublicScopedAccountSummaryQuery>())
    .field(label_result_field::<PublicScopedAccountSummaryQuery>())
    .build();
    ApplicationQueryDefinitionBuilder::declare(PublicScopedAccountSummaryQuery::reference())
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(0, 0, 2))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
}
