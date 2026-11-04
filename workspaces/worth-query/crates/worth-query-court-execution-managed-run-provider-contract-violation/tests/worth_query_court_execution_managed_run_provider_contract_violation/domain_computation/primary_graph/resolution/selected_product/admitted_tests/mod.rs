use std::time::Duration;

use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;

#[test]
fn admitted_selected_principal_matches_ordinary_and_certification() {
    let world = installed_authorization_world(true);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let execution = &borrowed_execution;
            let selected = world.selected_product(
                &(host_custody.phase_for_application_for_test(&world.application)).test_host_request_for_test(),
            );
            let request = live_scope();
            let external = world.authenticate("alice", Duration::from_secs(60), &request);
            let legacy = selected
                .resolve_authenticated_principal(
                    &host_custody.phase_for_application_for_test(&world.application),
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .expect("legacy selected principal resolves");

            for mode in [
                WorthQueryPrincipalResolutionMode::Ordinary,
                WorthQueryPrincipalResolutionMode::Certification,
            ] {
                let fresh = selected
                    .resolve_authenticated_principal_admitted_for_test(
                        execution,
                        &world.binding,
                        &external,
                        &request,
                        mode,
                    )
                    .expect("admitted principal uses the same selected truth");
                assert_eq!(fresh.principal_entity_id_for_test(), legacy.principal_entity_id_for_test());
                assert_eq!(fresh.mapping_entity_id_for_test(), legacy.mapping_entity_id_for_test());
                assert_eq!(fresh.target_relation_id_for_test(), legacy.target_relation_id_for_test());
                assert_eq!(fresh.principal_identity(), legacy.principal_identity());
                assert_eq!(
                    fresh.examined_candidate_count(),
                    legacy.examined_candidate_count()
                );
            }
        })
        .expect("the fixture block owns its installed host call")
}
