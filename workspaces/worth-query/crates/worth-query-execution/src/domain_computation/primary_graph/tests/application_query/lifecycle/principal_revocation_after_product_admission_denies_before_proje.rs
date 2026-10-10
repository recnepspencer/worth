//! principal revocation after product admission denies before projection.
use super::*;

#[test]
fn principal_revocation_after_product_admission_denies_before_projection() {
    let world = installed_authorization_world(true);
    let request = super::super::super::fixture::live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .select_product_branch(
                    &host_custody.borrowed_request_for_test(),
                    world.application.product_runtime().default_branch(),
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
            let host_custody = phase.test_host_request();
            world
                .application
                .select_product_branch(
                    &host_custody.borrowed_request_for_test(),
                    world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_string(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let query = installed_query(&world);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let plan = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .selected_product(&(phase).test_host_request())
                .admit_application_query(
                    &phase,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), "open".to_string())
                        .expect("fixture query parameter must encode"),
                    current_controls(&request),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let basis = plan.basis_identity().clone();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            disable_mapping(
                &host_custody.borrowed_request_for_test(),
                &world,
                principal.mapping_entity_id(),
            )
        })
        .expect("the fixture read owns its installed host call");

    let denial = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .execute_application_query_one_shot(&phase, plan)
                .err()
                .expect("revocation must deny before reading the retained data basis")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationOneShotDenialKind::StalePrincipal
    );
    assert!(!basis_is_live(&world, &basis));
}
