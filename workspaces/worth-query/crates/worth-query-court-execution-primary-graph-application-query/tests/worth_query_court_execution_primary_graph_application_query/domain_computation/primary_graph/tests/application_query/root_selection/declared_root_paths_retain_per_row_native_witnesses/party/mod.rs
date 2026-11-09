//! Alice's principal and the open account, each resolved in its own installed host call.
use super::*;

pub fn alice_and_open_account(
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> (
    worth_query_execution::facade::domain_computation::WorthQueryAuthenticatedPrincipal<
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema,
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Principal,
        u64,
    >,
    worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityIdentity<
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema,
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account,
    >,
){
    let external = world.authenticate("alice", Duration::from_secs(60), request);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world
                .application
                .select_product_branch_for_test(
                    &host_custody.borrowed_request_for_test(),
                    world
                        .application
                        .product_runtime()
                        .default_branch_for_test(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world
                .application
                .select_product_branch_for_test(
                    &host_custody.borrowed_request_for_test(),
                    world
                        .application
                        .product_runtime()
                        .default_branch_for_test(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &phase,
                    AccountStatus::at("AccountStatus"),
                    "open".to_string(),
                    request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    (principal, account)
}
