use std::time::Duration;

use super::super::WorthQueryPrincipalResolutionMode;
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, live_scope,
};

#[test]
fn admitted_selected_principal_matches_ordinary_and_certification() {
    let world = installed_authorization_world(true);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let execution = &borrowed_execution;
            let selected = world.selected_product(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
            );
            let request = live_scope();
            let external = world.authenticate("alice", Duration::from_secs(60), &request);
            let legacy = selected
                .resolve_authenticated_principal(
                    &host_custody.phase_for_application(&world.application),
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .expect("legacy selected principal resolves");
            let _owner = &world
                .application
                .primary_provider
                .graph
                .source_owner
                .invalidation_owner;

            for mode in [
                WorthQueryPrincipalResolutionMode::Ordinary,
                WorthQueryPrincipalResolutionMode::Certification,
            ] {
                let fresh = selected
                    .resolve_authenticated_principal_admitted(
                        execution,
                        &world.binding,
                        &external,
                        &request,
                        mode,
                    )
                    .expect("admitted principal uses the same selected truth");
                assert_eq!(fresh.principal_entity_id(), legacy.principal_entity_id());
                assert_eq!(fresh.mapping_entity_id(), legacy.mapping_entity_id());
                assert_eq!(fresh.target_relation_id(), legacy.target_relation_id());
                assert_eq!(fresh.principal_identity(), legacy.principal_identity());
                assert_eq!(
                    fresh.examined_candidate_count(),
                    legacy.examined_candidate_count()
                );
            }
        })
        .expect("the fixture block owns its installed host call")
}
