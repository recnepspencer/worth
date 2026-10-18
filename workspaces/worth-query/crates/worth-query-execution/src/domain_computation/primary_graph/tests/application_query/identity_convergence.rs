use super::*;

#[test]
fn request_controls_do_not_become_query_or_parameter_identity() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let execution = &borrowed_execution;
            let principal = world
                .application
                .select_product_branch(
                    execution,
                    world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_authenticated_principal(
                    &host_custody.phase_for_application(&world.application),
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let account = world
                .application
                .select_product_branch(
                    execution,
                    world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &host_custody.phase_for_application(&world.application),
                    AccountStatus::reference(),
                    "open".to_string(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let query = installed_query(&world);
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
            let admit = |status: &str, maximum_results: usize, maximum_work: usize| {
                world
                .selected_product(
                    &(host_custody.phase_for_application(&world.application)).test_host_request(),
                )
                .admit_application_query(
                    &host_custody.phase_for_application(&world.application),
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), status.to_string())
                        .expect("fixture query parameter must encode"),
                    crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(
                        NonZeroUsize::new(maximum_results).unwrap(),
                        Some(NonZeroUsize::new(maximum_work).unwrap()),
                        &request,
                    ),
                )
            };

            let first = admit("open", 10, 10_000).unwrap();
            let query_identity = first.query_identity().clone();
            let parameter_identity = *first.parameter_binding_identity();
            assert_eq!(first.controls().maximum_result_count().get(), 10);
            assert_eq!(
                first.controls().maximum_work().map(NonZeroUsize::get),
                Some(10_000)
            );
            drop(first);

            let changed_controls = admit("open", 9, 9_999).unwrap();

            assert_eq!(changed_controls.query_identity(), &query_identity);
            assert_eq!(
                changed_controls.parameter_binding_identity(),
                &parameter_identity
            );
            assert_eq!(changed_controls.controls().maximum_result_count().get(), 9);
            assert_eq!(
                changed_controls
                    .controls()
                    .maximum_work()
                    .map(NonZeroUsize::get),
                Some(9_999)
            );
            drop(changed_controls);

            let changed_parameter = admit("closed", 10, 10_000).unwrap();

            assert_eq!(changed_parameter.query_identity(), &query_identity);
            assert_ne!(
                changed_parameter.parameter_binding_identity(),
                &parameter_identity
            );
        })
        .expect("the fixture block owns its installed host call")
}
