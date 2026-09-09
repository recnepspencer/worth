use std::time::Duration;

use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;

use super::fixture::{installed_world, live_scope};
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode;

#[test]
fn admitted_external_identity_resolves_through_certified_index_and_freshness() {
    let world = installed_world(&[("alice", WorthQueryPrincipalMappingStatus::Enabled)]);
    let scope = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &scope);

    let principal = world
        .selected_product()
        .resolve_authenticated_principal(
            &world.binding,
            external,
            &scope,
            WorthQueryPrincipalResolutionMode::Certification,
        )
        .unwrap();

    assert_eq!(principal.external_identity().subject(), "alice");
    assert_eq!(*principal.principal_identity(), 1);
    assert_eq!(principal.attributes()[0].key(), "display");
    assert_eq!(principal.attributes()[0].value(), "Test User");
    assert_eq!(principal.examined_candidate_count(), 1);
    world
        .selected_product()
        .validate_authenticated_principal(&principal, &scope)
        .unwrap();
    let debug = format!("{principal:?}");
    assert!(!debug.contains("alice"));
    assert!(!debug.contains("https://issuer.example"));
}
