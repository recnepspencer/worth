use std::time::Duration;

use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_query::current_controls;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_query::installed_query;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::parameter_reference::status_parameter;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAdmissionDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

#[test]
fn query_graph_capacity_denial_and_drop_return_exact_reservation() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let execution = &borrowed_execution;
            let principal = world
                .application
                .select_product_branch_for_test(
                    execution,
                    world.application.product_runtime().default_branch_for_test(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_authenticated_principal(
                    &host_custody.phase_for_application_for_test(&world.application),
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let account = world
                .application
                .select_product_branch_for_test(
                    execution,
                    world.application.product_runtime().default_branch_for_test(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &host_custody.phase_for_application_for_test(&world.application),
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let query = installed_query(&world);
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
            let admit = || {
                world
                    .selected_product(
                        &(host_custody.phase_for_application_for_test(&world.application))
                            .test_host_request_for_test(),
                    )
                    .admit_application_query(
                        &host_custody.phase_for_application_for_test(&world.application),
                        &query,
                        &access,
                        ApplicationQueryParameterSet::new()
                            .bind(status_parameter(), "open".to_owned())
                            .expect("fixture query parameter must encode"),
                        current_controls(&request),
                    )
            };
            let mut retained = (0..64).map(|_| admit().unwrap()).collect::<Vec<_>>();

            let denial = match admit() {
                Err(denial) => denial,
                Ok(_) => panic!("the sixty-fifth query session must be backpressured"),
            };
            assert_eq!(
                denial.kind(),
                WorthQueryApplicationQueryAdmissionDenialKind::GraphWorkAdmissionUnavailable
            );
            drop(retained.pop());
            retained.push(admit().expect("dropping one session returns one graph reservation"));
            drop(retained);
            assert_eq!(
                world.application.provider_session_resource_count_for_test(),
                0
            );
            let completed = world
                .application
                .execute_application_query_one_shot(
                    &host_custody.phase_for_application_for_test(&world.application),
                    admit().unwrap(),
                )
                .expect("a successful read completes the exact returned reservation");
            assert_eq!(
                completed
                    .receipt()
                    .read_completion()
                    .release()
                    .released_reservation_count(),
                1
            );
            assert_eq!(
                world.application.provider_session_resource_count_for_test(),
                0
            );
        })
        .expect("the fixture block owns its installed host call")
}
