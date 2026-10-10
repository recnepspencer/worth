//! scope revocation after product admission denies before projection.
use super::*;

#[test]
fn scope_revocation_after_product_admission_denies_before_projection() {
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
            revoke_account_ownership(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                account.entity_id(),
            )
        })
        .expect("the fixture read owns its installed host call");
    let observer = world.application.application_query_basis_observer();
    let acquisitions_before = observer.observe().acquisitions();

    let denial = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .execute_application_query_one_shot(&phase, plan)
                .err()
                .expect("revoked scope authorization must deny before projection")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationOneShotDenialKind::Authorization(
            crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind::StaleAuthorization,
        )
    );
    assert!(!basis_is_live(&world, &basis));
    assert!(
        observer.observe().acquisitions() > acquisitions_before,
        "branch drift must use a newly registered security basis, not the selected data snapshot"
    );
}
