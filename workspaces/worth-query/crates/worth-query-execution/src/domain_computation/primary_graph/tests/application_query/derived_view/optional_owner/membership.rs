//! The entry's ownership relation is deliberately absent from this real census.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::PublicAccountMembershipResult;

pub(super) fn read(
    world: &AuthorizationWorld,
) -> WorthQueryApplicationOneShotResult<PublicAccountMembershipQuery, PublicAccountMembershipResult>
{
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let selected = world.selected_product();
    let principal = selected
        .resolve_authenticated_principal(
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let scope = selected
        .resolve_entity(
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
            &query,
            &access,
            ApplicationQueryParameterSet::new(),
            current_controls(&request),
        )
        .unwrap();
    world
        .application
        .execute_application_query_one_shot(plan)
        .unwrap()
}
