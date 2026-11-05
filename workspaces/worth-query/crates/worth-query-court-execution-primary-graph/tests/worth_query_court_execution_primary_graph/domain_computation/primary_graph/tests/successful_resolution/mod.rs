use std::time::Duration;

use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

#[test]
fn admitted_external_identity_resolves_through_certified_index_and_freshness() {
    let world = installed_world(&[("alice", WorthQueryPrincipalMappingStatus::Enabled)]);
    let scope = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &scope);

    let principal = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &scope,
                    WorthQueryPrincipalResolutionMode::Certification,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(principal.external_identity().subject(), "alice");
    assert_eq!(*principal.principal_identity(), 1);
    assert_eq!(principal.attributes()[0].key(), "display");
    assert_eq!(principal.attributes()[0].value(), "Test User");
    assert_eq!(principal.examined_candidate_count(), 1);
    world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
                .validate_authenticated_principal(&phase, &principal, &scope)
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let debug = format!("{principal:?}");
    assert!(!debug.contains("alice"));
    assert!(!debug.contains("https://issuer.example"));
}
