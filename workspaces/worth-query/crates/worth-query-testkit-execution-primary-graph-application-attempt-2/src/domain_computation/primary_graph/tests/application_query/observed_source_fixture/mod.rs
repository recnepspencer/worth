//! An ordinary public query supplies the source descriptor for scheduling proofs.
use crate::domain_computation::primary_graph::tests::application_query::*;

pub fn observed_account_source(
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
) -> worth_query_execution::facade::domain_computation::WorthQueryObservedSource<AccountSummaryQuery>
{
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    world
        .application
        .with_host_advancement(|phase| {
            let selected = world.selected_product(&phase.test_host_request());
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let account = selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let query = installed_query(world);
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
            let plan = selected
                .admit_application_query(
                    &phase,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), "open".to_owned())
                        .unwrap(),
                    current_controls(&request),
                )
                .unwrap();
            let result = world
                .application
                .execute_application_query_one_shot(&phase, plan)
                .unwrap();
            result.observed_sources()[0].clone()
        })
        .unwrap()
}
