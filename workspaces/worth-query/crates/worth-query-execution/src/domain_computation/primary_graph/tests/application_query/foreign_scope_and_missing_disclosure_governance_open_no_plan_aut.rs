//! foreign scope and missing disclosure governance open no plan authority.
use super::*;

#[test]
fn foreign_scope_and_missing_disclosure_governance_open_no_plan_authority() {
    let world = installed_authorization_world(true);
    let foreign = installed_authorization_world(true);
    let request = live_scope();
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
    let foreign_account = foreign
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            foreign
                .application
                .select_product_branch(
                    &host_custody.borrowed_request_for_test(),
                    foreign.application.product_runtime().default_branch(),
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
    let crossed = WorthQueryApplicationQueryAccessContext::new(&principal, &foreign_account);
    let denial = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .selected_product(&(phase).test_host_request())
                .admit_application_query(
                    &phase,
                    &query,
                    &crossed,
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), "open".to_string())
                        .unwrap(),
                    current_controls(&request),
                )
                .err()
                .expect("foreign scope must deny")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::ForeignScope
    );

    let governed = installed_governed_query(&world);
    let local_account = world
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
    let local = WorthQueryApplicationQueryAccessContext::new(&principal, &local_account);
    let denial = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .selected_product(&(phase).test_host_request())
                .admit_application_query(
                    &phase,
                    &governed,
                    &local,
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), "open".to_string())
                        .unwrap(),
                    current_controls(&request),
                )
                .err()
                .expect("missing disclosure governance must deny")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::DisclosureGovernanceRequired
    );
}
