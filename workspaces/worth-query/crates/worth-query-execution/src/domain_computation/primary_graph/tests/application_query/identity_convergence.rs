use super::*;

#[test]
fn request_controls_do_not_become_query_or_parameter_identity() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &world.binding,
            external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let account = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = installed_query(&world);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let admit = |status: &str, maximum_results: usize, maximum_work: usize| {
        world.selected_product().admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new().bind(status_parameter(), status.to_string()),
            crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(
                NonZeroUsize::new(maximum_results).unwrap(),
                NonZeroUsize::new(maximum_work).unwrap(),
                &request,
            ),
        )
    };

    let first = admit("open", 10, 10_000).unwrap();
    let query_identity = first.query_identity().clone();
    let parameter_identity = *first.parameter_binding_identity();
    assert_eq!(first.controls().maximum_result_count().get(), 10);
    assert_eq!(first.controls().maximum_work().get(), 10_000);
    drop(first);

    let changed_controls = admit("open", 9, 9_999).unwrap();

    assert_eq!(changed_controls.query_identity(), &query_identity);
    assert_eq!(
        changed_controls.parameter_binding_identity(),
        &parameter_identity
    );
    assert_eq!(changed_controls.controls().maximum_result_count().get(), 9);
    assert_eq!(changed_controls.controls().maximum_work().get(), 9_999);
    drop(changed_controls);

    let changed_parameter = admit("closed", 10, 10_000).unwrap();

    assert_eq!(changed_parameter.query_identity(), &query_identity);
    assert_ne!(
        changed_parameter.parameter_binding_identity(),
        &parameter_identity
    );
}
