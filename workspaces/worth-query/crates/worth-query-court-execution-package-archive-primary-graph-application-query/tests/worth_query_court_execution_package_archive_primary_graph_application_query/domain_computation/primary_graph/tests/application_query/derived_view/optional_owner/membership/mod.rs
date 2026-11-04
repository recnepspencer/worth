//! The entry's ownership relation is deliberately absent from this real census.
use crate::domain_computation::primary_graph::tests::application_query::derived_view::optional_owner::*;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::public_membership::PublicAccountMembershipResult;

pub fn read(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
) -> WorthQueryApplicationOneShotResult<PublicAccountMembershipQuery, PublicAccountMembershipResult>
{
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let selected = world.selected_product(&(phase_custody).test_host_request_for_test());
    let principal = selected
        .resolve_authenticated_principal(
            phase_custody,
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let scope = selected
        .resolve_entity(
            phase_custody,
            AccountStatus::reference(),
            "open".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .certification_query(PublicAccountMembershipQuery::reference())
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let plan = selected
        .admit_application_query(
            phase_custody,
            &query,
            &access,
            ApplicationQueryParameterSet::new(),
            current_controls(&request),
        )
        .unwrap();
    world
        .application
        .execute_application_query_one_shot(phase_custody, plan)
        .unwrap()
}
