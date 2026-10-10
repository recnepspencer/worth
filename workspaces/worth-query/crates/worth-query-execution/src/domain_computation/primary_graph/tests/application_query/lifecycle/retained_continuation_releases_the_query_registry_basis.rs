//! retained continuation releases the query registry basis.
use super::*;

#[test]
fn retained_continuation_releases_the_query_registry_basis() {
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
                    AccountIdentity::reference(),
                    "account-1".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let query = super::super::installed_live_query(&world);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let plan = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .selected_product(&(phase).test_host_request())
                .admit_application_query_continuation(
                    &phase,
                    &query,
                    &access,
                    live_account_parameters("account-1"),
                    crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(
                        NonZeroUsize::new(1).unwrap(),
                        NonZeroUsize::new(10_000).unwrap(),
                        &request,
                    ),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let basis = plan.basis_identity().clone();
    let session = plan.graph_work_session_identity();
    assert!(basis_is_live(&world, &basis));

    let page = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .execute_application_query_continuation_page(&phase, plan)
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let (_, continuation, receipt) = page.into_parts();
    let continuation = continuation.expect("one of two activities must retain a next page");

    assert_eq!(continuation.page_ordinal(), 2);
    assert!(receipt.basis_released());
    assert_eq!(receipt.read_completion().session_identity(), session);
    assert_eq!(receipt.read_completion().basis_identity(), &basis);
    assert_eq!(
        receipt
            .read_completion()
            .release()
            .released_reservation_count(),
        1
    );
    assert!(!basis_is_live(&world, &basis));
    drop(continuation);
    assert!(!basis_is_live(&world, &basis));
}
