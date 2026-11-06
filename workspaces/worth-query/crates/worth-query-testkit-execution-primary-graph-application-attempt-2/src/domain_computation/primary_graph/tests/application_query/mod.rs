use std::num::NonZeroUsize;
use std::time::Duration;
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::parameter_reference::status_parameter;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountSummaryQuery;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryResult;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ForgedSelectorQuery;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::forged_selector::ForgedSelectorResult;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::GovernedAccountSummaryQuery;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::LiveAccountActivityQuery;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::live_account_query::LiveAccountActivityResult;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::NestedAccountQuery;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::nested_account::NestedAccountResult;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::OrderedAccountSummaryQuery;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
pub use crate::domain_computation::primary_graph::tests::application_query::observed_source_fixture::observed_account_source;
pub fn installed_query(
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
) -> worth_query_installation::facade::WorthQueryInstalledApplicationQuery<
    IdentityExecutionSchema,
    AccountSummaryQuery,
    AccountSummaryParameters,
    AccountSummaryResult,
    Account,
> {
    world
        .application
        .installed_schema()
        .certification_query(AccountSummaryQuery::reference())
        .unwrap()
}
pub fn installed_governed_query(
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
) -> worth_query_installation::facade::WorthQueryInstalledApplicationQuery<
    IdentityExecutionSchema,
    GovernedAccountSummaryQuery,
    AccountSummaryParameters,
    AccountSummaryResult,
    Account,
> {
    world
        .application
        .installed_schema()
        .certification_query(GovernedAccountSummaryQuery::reference())
        .unwrap()
}
pub fn installed_ordered_query(
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
) -> worth_query_installation::facade::WorthQueryInstalledApplicationQuery<
    IdentityExecutionSchema,
    OrderedAccountSummaryQuery,
    AccountSummaryParameters,
    AccountSummaryResult,
    Account,
> {
    world
        .application
        .installed_schema()
        .certification_query(OrderedAccountSummaryQuery::reference())
        .unwrap()
}
pub fn installed_live_query(
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
) -> worth_query_installation::facade::WorthQueryInstalledApplicationQuery<
    IdentityExecutionSchema,
    LiveAccountActivityQuery,
    AccountSummaryParameters,
    LiveAccountActivityResult,
    Account,
> {
    world
        .application
        .installed_schema()
        .certification_query(LiveAccountActivityQuery::reference())
        .unwrap()
}
pub fn installed_nested_query(
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
) -> worth_query_installation::facade::WorthQueryInstalledApplicationQuery<
    IdentityExecutionSchema,
    NestedAccountQuery,
    AccountSummaryParameters,
    NestedAccountResult,
    Account,
> {
    world
        .application
        .installed_schema()
        .certification_query(NestedAccountQuery::reference())
        .unwrap()
}
pub fn installed_forged_selector_query(
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
) -> worth_query_installation::facade::WorthQueryInstalledApplicationQuery<
    IdentityExecutionSchema,
    ForgedSelectorQuery,
    AccountSummaryParameters,
    ForgedSelectorResult,
    Account,
> {
    world
        .application
        .installed_schema()
        .certification_query(ForgedSelectorQuery::reference())
        .unwrap()
}
pub fn current_controls(
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls<'_> {
    worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls::new(
        NonZeroUsize::new(10).unwrap(),
        request,
    )
}

#[path = "observed_source_fixture/mod.rs"]
pub mod observed_source_fixture;
