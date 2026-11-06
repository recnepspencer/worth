use std::time::Duration;

use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::parameter_reference::status_parameter;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_query::current_controls;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_query::installed_ordered_query;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

#[test]
fn path_bound_ordering_mechanism_opens_exact_plan_authority() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
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
                    &request,
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
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let query = installed_ordered_query(&world);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let plan = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
                .admit_application_query(
                    &phase,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), "open".to_string())
                        .unwrap(),
                    current_controls(&request),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    assert!(plan
        .graph_read_plan()
        .requirements()
        .requires_kind(
            worth_query_admission::facade::graph_read_access::WorthQueryGraphReadAccessRequirementKind::OrderingSupport,
        ));
    let result = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot(&phase, plan)
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(result.rows()[0].label(), "primary");
}
