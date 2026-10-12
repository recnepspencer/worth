//! Reads the exact product occurrence through the installed query and its authority.
use super::*;
pub(super) fn assert_product_input(
    world: &CourtroomWorld,
    request: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
    principal: &primary_graph::WorthQueryAuthenticatedPrincipal<TemporalHostSchema, Principal, u64>,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    expected: &str,
) {
    let query = world
        .application
        .installed_schema()
        .certification_query(TemporalIntentQuery::reference())
        .unwrap();

    let selected = world
        .application
        .with_host_advancement(|phase| world.application.on_branch(branch).select(&phase).unwrap())
        .expect("installed host call opens");
    let scope = world
        .application
        .with_host_advancement(|phase| {
            selected
                .resolve_entity(
                    &phase,
                    IntentIdentityField::reference(),
                    "intent-1".to_string(),
                    request,
                    primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let access = primary_graph::WorthQueryApplicationQueryAccessContext::new(principal, &scope);
    let plan = world
        .application
        .with_host_advancement(|phase| {
            selected
                .admit_application_query(
                    &phase,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    controls(request),
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let result = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot(&phase, plan)
                .unwrap()
        })
        .expect("installed host call opens");
    assert_eq!(result.rows()[0].input, expected);
    assert_eq!(product_identity(result.receipt()).product_branch(), branch);
    assert_security_work(result.receipt());
}
